//! The runtime: everything that isn't state. It owns the terminal, the control socket, child
//! processes and the watched file, turns what happens into `Msg`s for `update`, and carries out
//! the `Effect`s that come back.

use crate::host::{Host, NoHost, Notifier};
use crate::model::{Source, ThemeSpec, Ui};
use crate::state::{Effect, Msg, State, Stats, narrow, update};
use crate::view;
use anyhow::{Context, Result, bail};
use crate::view::Hit;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton,
    MouseEventKind,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

/// One request line on the control socket.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Request {
    Push { ui: Ui },
    Patch { id: String, node: crate::model::Node },
    Key { key: String },
    /// Write one store entry (`put`, `merge` or `edit`), refused when `if_rev` is stale.
    Write {
        key: String,
        #[serde(flatten)]
        write: crate::store::Write,
        #[serde(default)]
        if_rev: Option<u64>,
        #[serde(default = "agent")]
        by: String,
    },
    /// One store entry: its revision, who wrote it last, and its text or value.
    Read { key: String },
    /// Add themes (by name) and choose one: a name, `next` or `prev`.
    Theme {
        #[serde(default)]
        add: BTreeMap<String, crate::model::ThemeSpec>,
        #[serde(default)]
        choose: Option<String>,
    },
    /// Move one node's view state: `selected` (lists, tables) or `tab`.
    Set {
        id: String,
        #[serde(default)]
        selected: Option<usize>,
        #[serde(default)]
        tab: Option<usize>,
    },
    /// Replace the layers only (callouts, rings, spotlights).
    Layers { layers: Vec<crate::model::LayerSpec> },
    /// A message in the corner for a while.
    Toast {
        text: String,
        #[serde(default)]
        tone: Option<String>,
        #[serde(default)]
        ms: Option<u64>,
    },
    /// The mouse at a cell, as the terminal would report it: `move`, `click`, `up` or `down`
    /// (the wheel).
    Mouse { kind: String, x: u16, y: u16 },
    /// The current frame, as text.
    Screen,
    /// Save the whole state, become `binary` (default: this one) and carry on from that state.
    Upgrade {
        #[serde(default)]
        binary: Option<String>,
    },
    /// Click a node as the mouse would: a row of it, or a tab.
    Click {
        id: String,
        #[serde(default)]
        row: Option<usize>,
        #[serde(default)]
        tab: Option<usize>,
    },
    Get,
    State,
    Stats,
}

/// Everything the loop reacts to arrives on one queue: terminal input, data, socket requests.
enum Event {
    Msg(Msg),
    Request(Request, Sender<Value>),
    Term(TermEvent),
}

pub fn default_socket() -> PathBuf {
    std::env::var_os("FICTTY_SOCKET").map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("fictty.sock"))
}

pub fn load(path: &Path) -> Result<Ui> {
    let raw = if path == Path::new("-") {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?
    };
    serde_json::from_str(&raw).with_context(|| format!("{} is not a UI", path.display()))
}

/// Fetch a source now (blocking).
pub fn fetch(src: &Source, host: &dyn Host) -> Result<Value, String> {
    if let Some(name) = &src.host {
        let v = host.fetch(name, &src.args)?;
        return match src.path.as_deref() {
            Some(p) => crate::model::lookup(&v, p).cloned().ok_or_else(|| format!("{name}: no {p} in its value")),
            None => Ok(v),
        };
    }
    if let Some(v) = &src.value {
        return narrow(&v.to_string(), src.path.as_deref());
    }
    if let Some(argv) = &src.stream {
        let mut child = spawn_stream(argv)?;
        let mut line = String::new();
        let read = BufReader::new(child.stdout.take().ok_or("no stdout")?).read_line(&mut line);
        let _ = child.kill();
        let _ = child.wait();
        read.map_err(|e| e.to_string())?;
        return narrow(&line, src.path.as_deref());
    }
    let mut cmd = match (&src.cmd, &src.shell) {
        (Some(argv), _) if !argv.is_empty() => {
            let mut c = Command::new(&argv[0]);
            c.args(&argv[1..]);
            c
        }
        (_, Some(sh)) => {
            let mut c = Command::new("sh");
            c.args(["-c", sh]);
            c
        }
        _ => return Err("a source needs cmd, shell or value".into()),
    };
    let out = cmd.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().next().unwrap_or("failed").to_string());
    }
    narrow(&String::from_utf8_lossy(&out.stdout), src.path.as_deref())
}

fn spawn_stream(argv: &[String]) -> Result<Child, String> {
    let (bin, args) = argv.split_first().ok_or("stream: empty command")?;
    Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{bin}: {e}"))
}

fn exec(argv: &[String]) -> Result<(), String> {
    let (bin, args) = argv.split_first().ok_or("run: empty command")?;
    let out = Command::new(bin).args(args).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("failed").to_string())
    }
}

/// Runs `update` and carries out its effects. Fetches and runs go to threads that report back as
/// messages; `blocking` (for headless renders) does them inline instead.
struct Driver {
    tx: Sender<Event>,
    in_flight: BTreeSet<String>,
    last_fetch: BTreeMap<String, Instant>,
    /// Running `stream` sources, with the definition each was started from.
    streams: BTreeMap<String, (Source, Child)>,
    quit: bool,
    blocking: bool,
    /// Set by an upgrade request: the binary to become.
    upgrade: Option<String>,
    /// Zero on the logical clock (`State::now_ms`).
    started: Instant,
    host: Arc<dyn Host>,
}

impl Drop for Driver {
    fn drop(&mut self) {
        for (_, (_, mut c)) in std::mem::take(&mut self.streams) {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Driver {
    fn new(tx: Sender<Event>, blocking: bool, host: Arc<dyn Host>) -> Driver {
        Driver { tx, in_flight: BTreeSet::new(), last_fetch: BTreeMap::new(), streams: BTreeMap::new(), quit: false, blocking, upgrade: None, started: Instant::now(), host }
    }

    fn apply(&mut self, st: &mut State, msg: Msg) {
        if let Msg::Data { name, .. } = &msg {
            self.in_flight.remove(name);
        }
        // Messages carry the time they arrived (an editor groups typing into undo steps by it; a
        // toast lasts from then). Headless renders keep the clock still, so they come out the
        // same every time.
        if !matches!(msg, Msg::Tick { .. }) && !self.blocking {
            update(st, Msg::Tick { now_ms: self.started.elapsed().as_millis() as u64 });
        }
        let mut queue: Vec<Effect> = update(st, msg);
        while let Some(fx) = queue.pop() {
            match fx {
                Effect::Quit => self.quit = true,
                Effect::Reply(_) => {}
                // The terminal puts it on the clipboard (OSC 52); a headless render has none.
                Effect::Copy(text) if !self.blocking => {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                    let mut out = std::io::stdout();
                    let _ = write!(out, "\x1b]52;c;{b64}\x07");
                    let _ = out.flush();
                }
                Effect::Copy(_) => {}
                Effect::Load(path) => match load(Path::new(&path)) {
                    Ok(ui) => queue.extend(update(st, Msg::Push { ui })),
                    Err(e) => st.status = Some(format!("{e:#}")),
                },
                Effect::Fetch(name) if self.blocking => {
                    let Some(src) = st.ui.data.get(&name) else { continue };
                    let result = fetch(src, &*self.host);
                    queue.extend(update(st, Msg::Data { name, result }));
                }
                Effect::Fetch(name) if st.ui.data.get(&name).is_some_and(|s| s.stream.is_some()) => {
                    self.start_stream(st, name)
                }
                Effect::Fetch(name) => self.spawn_fetch(st, name),
                Effect::Host { name, args } => {
                    if self.blocking {
                        let result = self.host.act(&name, &args);
                        queue.extend(update(st, Msg::Acted { name, result, at: String::new() }));
                    } else {
                        let (tx, host) = (self.tx.clone(), self.host.clone());
                        std::thread::spawn(move || {
                            let result = host.act(&name, &args);
                            let _ = tx.send(Event::Msg(Msg::Acted { name, result, at: clock() }));
                        });
                    }
                }
                Effect::Run(argv) => {
                    if self.blocking {
                        let result = exec(&argv);
                        queue.extend(update(st, Msg::Ran { argv, result }));
                    } else {
                        let tx = self.tx.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(Event::Msg(Msg::Ran { result: exec(&argv), argv }));
                        });
                    }
                }
            }
        }
        self.reap(st);
    }

    /// Start a stream source unless it's already running from the same definition.
    fn start_stream(&mut self, st: &mut State, name: String) {
        let Some(src) = st.ui.data.get(&name).cloned() else { return };
        if self.streams.get(&name).is_some_and(|(def, _)| *def == src) {
            return;
        }
        let argv = src.stream.clone().unwrap_or_default();
        let mut child = match spawn_stream(&argv) {
            Ok(c) => c,
            Err(e) => {
                update(st, Msg::Data { name, result: Err(e) });
                return;
            }
        };
        let out = child.stdout.take().expect("piped");
        let (tx, path, n) = (self.tx.clone(), src.path.clone(), name.clone());
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if line.trim().is_empty() {
                    continue;
                }
                let msg = Msg::Data { name: n.clone(), result: narrow(&line, path.as_deref()) };
                if tx.send(Event::Msg(msg)).is_err() {
                    return;
                }
            }
        });
        if let Some((_, mut old)) = self.streams.insert(name, (src, child)) {
            let _ = old.kill();
            let _ = old.wait();
        }
    }

    /// Stop every stream (before an upgrade: the new process starts its own).
    fn stop_streams(&mut self) {
        for (_, (_, mut c)) in std::mem::take(&mut self.streams) {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    /// Stop streams the UI no longer has (or has changed).
    fn reap(&mut self, st: &State) {
        let gone: Vec<String> =
            self.streams.iter().filter(|(k, (def, _))| st.ui.data.get(*k) != Some(def)).map(|(k, _)| k.clone()).collect();
        for k in gone {
            if let Some((_, mut c)) = self.streams.remove(&k) {
                let _ = c.kill();
                let _ = c.wait();
            }
        }
    }

    /// Sources with `every` that are due.
    fn tick(&mut self, st: &mut State) {
        let due: Vec<String> = st
            .ui
            .data
            .iter()
            .filter(|(k, s)| {
                s.stream.is_none()
                    && s.every.is_some_and(|secs| {
                        self.last_fetch.get(*k).is_none_or(|t| t.elapsed() >= Duration::from_secs_f64(secs.max(0.01)))
                    })
            })
            .map(|(k, _)| k.clone())
            .collect();
        for name in due {
            self.spawn_fetch(st, name);
        }
    }

    /// Fetch on a thread; the value comes back as `Msg::Data`. One fetch per source at a time.
    fn spawn_fetch(&mut self, st: &State, name: String) {
        let Some(src) = st.ui.data.get(&name).cloned() else { return };
        self.last_fetch.insert(name.clone(), Instant::now());
        if !self.in_flight.insert(name.clone()) {
            return;
        }
        let (tx, host) = (self.tx.clone(), self.host.clone());
        std::thread::spawn(move || {
            let _ = tx.send(Event::Msg(Msg::Data { result: fetch(&src, &*host), name }));
        });
    }

    fn request(&mut self, st: &mut State, req: Request) -> Value {
        // What changed the screen goes in `$log`, so a UI can show what its agent is doing.
        // A write is logged after it's tried, so a refused one shows as refused.
        let logged = activity(st, &req);
        let log = |st: &mut State, op: &str, what: String| {
            update(st, Msg::Tick { now_ms: self.started.elapsed().as_millis() as u64 });
            update(st, Msg::Activity { op: op.into(), what, at: clock() });
        };
        let is_write = matches!(req, Request::Write { .. });
        if let (Some((op, what)), false) = (&logged, is_write) {
            log(st, op, what.clone());
        }
        let msg = match req {
            Request::Get => return serde_json::to_value(&st.ui).unwrap_or_default(),
            Request::State => return serde_json::to_value(&*st).unwrap_or_default(),
            Request::Stats => return serde_json::to_value(&st.stats).unwrap_or_default(),
            Request::Push { ui } => Msg::Push { ui },
            Request::Patch { id, node } => Msg::Patch { id, node },
            Request::Key { key } => Msg::Key { key },
            Request::Read { key } => {
                return st.store.get(&key).map_or_else(|| json!({"ok": false, "status": format!("no entry {key}")}), |e| e.read());
            }
            Request::Write { key, write, if_rev, by } => {
                let reply = update(st, Msg::Write { key, write, if_rev, by }).into_iter().find_map(|fx| match fx {
                    Effect::Reply(r) => Some(r),
                    _ => None,
                });
                let mut reply = reply.unwrap_or_else(|| json!({"ok": false}));
                if let Some((op, what)) = logged {
                    match reply["ok"].as_bool() {
                        Some(true) => log(st, op, what),
                        _ => log(st, "refused", format!("{op} {what} ({})", refusal(&reply))),
                    }
                }
                if !st.problems.is_empty() {
                    reply["problems"] = json!(st.problems);
                }
                return reply;
            }
            Request::Set { id, selected, tab } => Msg::Set { id, selected, tab },
            Request::Theme { add, choose } => Msg::Theme { add, choose },
            Request::Layers { layers } => Msg::Layers { layers },
            Request::Toast { text, tone, ms } => Msg::Toast { text, tone, ms },
            Request::Click { id, row, tab } => Msg::Click { id, row, tab },
            // The loop answers these itself (they need the last frame).
            Request::Mouse { .. } | Request::Screen | Request::Upgrade { .. } => return json!({"ok": false}),
        };
        self.apply(st, msg);
        let mut reply = json!({"ok": st.status.is_none(), "version": st.version, "status": st.status});
        if !st.problems.is_empty() {
            reply["problems"] = json!(st.problems);
        }
        reply
    }
}

/// The store from its file, if there is one yet.
fn load_store(path: &Path) -> Result<BTreeMap<String, crate::store::Entry>> {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).with_context(|| format!("{} is not a fictty store", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// The store to its file: written beside it, then renamed over it, so a crash never leaves half.
fn save_store(path: &Path, st: &State) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&st.store)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Themes from a file: a JSON object of theme specs by name.
pub fn load_themes(path: &Path) -> Result<BTreeMap<String, crate::model::ThemeSpec>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("{} is not a set of themes", path.display()))
}

/// A screen to run: its UI (a value, or a file), and what it starts from. The `fictty` binary is
/// one user of this; an app that embeds fictty is another, with a `Host` for its own data.
///
/// ```no_run
/// # let ui: fictty::model::Ui = serde_json::from_str(r#"{"root": {"type": "text", "text": "hi"}}"#).unwrap();
/// fictty::App::new(ui).socket("/tmp/my.sock").run().unwrap();
/// ```
pub struct App {
    ui: Option<Ui>,
    file: Option<PathBuf>,
    watch: bool,
    socket: PathBuf,
    restore: Option<PathBuf>,
    store: Option<PathBuf>,
    themes: BTreeMap<String, ThemeSpec>,
    theme: Option<String>,
    host: Arc<dyn Host>,
}

impl App {
    /// A screen showing `ui`.
    pub fn new(ui: Ui) -> App {
        App { ui: Some(ui), ..App::empty() }
    }

    /// A screen showing the UI in `path` (`-` reads stdin), read when it starts.
    pub fn file(path: impl Into<PathBuf>) -> App {
        App { file: Some(path.into()), ..App::empty() }
    }

    /// A screen waiting for a UI to be pushed.
    pub fn empty() -> App {
        App {
            ui: None,
            file: None,
            watch: false,
            socket: default_socket(),
            restore: None,
            store: None,
            themes: BTreeMap::new(),
            theme: None,
            host: Arc::new(NoHost),
        }
    }

    /// Reload the UI file whenever it changes.
    pub fn watch(mut self, on: bool) -> App {
        self.watch = on;
        self
    }

    /// The control socket agents reach the screen on (default: `$FICTTY_SOCKET`, else
    /// `fictty.sock` in the temp dir).
    pub fn socket(mut self, path: impl Into<PathBuf>) -> App {
        self.socket = path.into();
        self
    }

    /// Start from a saved state (what `upgrade` hands the new process).
    pub fn restore(mut self, path: Option<PathBuf>) -> App {
        self.restore = path;
        self
    }

    /// Keep the store in this JSON file: read at the start, written after every change.
    pub fn store(mut self, path: impl Into<PathBuf>) -> App {
        self.store = Some(path.into());
        self
    }

    /// Add themes (by name). The first is chosen unless `theme` names one.
    pub fn themes(mut self, themes: BTreeMap<String, ThemeSpec>) -> App {
        self.themes.extend(themes);
        self
    }

    /// Add the themes in a JSON file (theme specs by name).
    pub fn themes_file(self, path: &Path) -> Result<App> {
        Ok(self.themes(load_themes(path)?))
    }

    /// The theme to start with.
    pub fn theme(mut self, name: impl Into<String>) -> App {
        self.theme = Some(name.into());
        self
    }

    /// The app embedding the screen: its sources and actions.
    pub fn host(mut self, host: impl Host) -> App {
        self.host = Arc::new(host);
        self
    }

    /// The state a screen starts from: restored, or new with the store file and the themes.
    fn start(&self) -> Result<State> {
        let mut st: State = match &self.restore {
            Some(p) => serde_json::from_str(&std::fs::read_to_string(p)?)?,
            None => State::default(),
        };
        if let (None, Some(p)) = (&self.restore, &self.store) {
            st.store = load_store(p)?;
        }
        if !self.themes.is_empty() || self.theme.is_some() {
            update(&mut st, Msg::Theme { add: self.themes.clone(), choose: self.theme.clone().filter(|_| self.restore.is_none()) });
        }
        Ok(st)
    }

    /// The UI to show first: a restored state's own, the given one, the file's, or a placeholder.
    fn first_ui(&self, st: &State) -> Result<Ui> {
        Ok(match (&self.restore, &self.ui, &self.file) {
            (Some(_), _, _) => st.ui.clone(),
            (None, Some(ui), _) => ui.clone(),
            (None, None, Some(p)) => load(p)?,
            (None, None, None) => serde_json::from_value(json!({"root": {"type": "text",
                "text": "Waiting for a UI.\n\nPush one:  fictty push ui.json"}}))?,
        })
    }

    /// Without a terminal: a screen of `width`×`height` cells that renders to text, for tests
    /// and `fictty render`. Sources are fetched and host actions done inline.
    pub fn headless(self, width: u16, height: u16) -> Result<Headless> {
        let mut st = self.start()?;
        let ui = self.first_ui(&st)?;
        let (tx, _) = mpsc::channel();
        let (ntx, notes) = mpsc::channel();
        self.host.start(Notifier::new(ntx));
        let mut d = Driver::new(tx, true, self.host.clone());
        d.apply(&mut st, Msg::Screen { width, height });
        d.apply(&mut st, Msg::Push { ui });
        let term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))?;
        let mut h = Headless { st, d, term, notes };
        // One frame to learn each editor's size, so keys scroll it as they would on screen.
        h.frame();
        Ok(h)
    }

    /// Run in this terminal until `quit`.
    pub fn run(mut self) -> Result<()> {
        if self.restore.is_none() {
            self.restore = std::env::var_os("FICTTY_RESTORE").map(PathBuf::from);
        }
        let App { file, watch, socket, store, host, .. } = &self;
        let (watch, socket, host) = (*watch, socket.clone(), host.clone());
        // Restoring (after an upgrade): the saved state, every selection and focus as it was.
        let mut st = self.start()?;
        let mut saved_writes = st.store_writes;
        let ui = self.first_ui(&st)?;
        let (tx, rx) = mpsc::channel();
        // Commands the screen runs (sources, `run`) can talk back to it: a helper script reads the
        // store with `$FICTTY read …`. Set before any thread starts.
        // SAFETY: no other thread exists yet, so nothing reads the environment concurrently.
        unsafe {
            std::env::set_var("FICTTY_SOCKET", &socket);
            std::env::remove_var("FICTTY_RESTORE");
            if let Ok(me) = std::env::current_exe() {
                std::env::set_var("FICTTY", me);
            }
        }
        serve(&socket, tx.clone())?;
        // The host's notices join the queue like everything else.
        let (ntx, nrx) = mpsc::channel::<Msg>();
        let fwd = tx.clone();
        std::thread::spawn(move || {
            for m in nrx {
                if fwd.send(Event::Msg(m)).is_err() {
                    return;
                }
            }
        });
        host.start(Notifier::new(ntx));
        let mut d = Driver::new(tx, false, host);
        // A restored state's clock carries on from where it was.
        d.started = Instant::now().checked_sub(Duration::from_millis(st.now_ms)).unwrap_or_else(Instant::now);
        d.apply(&mut st, Msg::Push { ui });
        let here = match file {
            Some(f) => std::fs::canonicalize(f).ok().and_then(|f| f.parent().map(Path::to_path_buf)).unwrap_or_default(),
            None => std::env::current_dir().unwrap_or_default(),
        };
        let bin = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "fictty".into());
        let sock = std::fs::canonicalize(&socket).unwrap_or_else(|_| socket.clone());
        d.apply(&mut st, Msg::Host { bin, socket: sock.display().to_string(), dir: here.display().to_string() });
        let caps = |sizing: bool| Msg::Caps { caps: crate::gfx::Caps::detect(), text_sizing: sizing };
        let mut kitty = crate::gfx::Kitty::default();
        let mut seen = file.as_deref().and_then(mtime);

        generic_tty()?;
        let mut term = ratatui::init();
        // Larger text: the terminal's to say (a probe), unless FICTTY_TEXT_SIZE forces it.
        let sizing = match std::env::var("FICTTY_TEXT_SIZE").as_deref() {
            Ok("1") => true,
            Ok("0") => false,
            _ => crate::gfx::probe_text_sizing(),
        };
        // (The probe clears what it printed; ratatui's first frame draws every cell.)
        d.apply(&mut st, caps(sizing));
        if let Ok(size) = term.size() {
            d.apply(&mut st, Msg::Screen { width: size.width, height: size.height });
        }
        let _ = ratatui::crossterm::execute!(std::io::stdout(), EnableMouseCapture);
        let started = d.started;
        let res = (|| -> Result<()> {
            // Draw only when something changed, at most once per frame budget (120 fps).
            let budget = Duration::from_micros(8_333);
            // Terminal input joins the same queue as everything else.
            let input = d.tx.clone();
            std::thread::spawn(move || {
                while let Ok(ev) = event::read() {
                    if input.send(Event::Term(ev)).is_err() {
                        return;
                    }
                }
            });
            let mut dirty = true;
            let mut last_draw = Instant::now() - budget;
            let mut window = Window::new();
            // What's under each cell in the last frame: the view's, for turning the mouse into messages.
            let mut hits: Vec<Hit> = Vec::new();
            let mut screens: Vec<Sender<Value>> = Vec::new();
            let mut scaled_last: Vec<view::Scaled> = Vec::new();
            // When the last frame said it would change by itself (a toast going, a spinner).
            let mut wake: Option<u64> = None;
            while !d.quit {
                // Animations (a pulsing ring) need time to move: tick every frame while one shows.
                let animating = st.ui.layers.iter().any(|l| l.pulse.is_some());
                let now = started.elapsed().as_millis() as u64;
                if (animating && last_draw.elapsed() >= budget) || wake.is_some_and(|w| now >= w) {
                    wake = None;
                    d.apply(&mut st, Msg::Tick { now_ms: started.elapsed().as_millis() as u64 });
                    dirty = true;
                }
                dirty |= !screens.is_empty();
                if dirty && last_draw.elapsed() >= budget {
                    let t = Instant::now();
                    let mut text = None;
                    let mut pics = Vec::new();
                    let mut scaled = Vec::new();
                    let mut sizes = Vec::new();
                    term.draw(|f| {
                        let out = view::draw(&st, f);
                        (hits, pics, scaled, sizes, wake) = (out.hits, out.pics, out.scaled, out.sizes, out.wake);
                        if !screens.is_empty() {
                            text = Some(buffer_text(f.buffer_mut()));
                        }
                    })?;
                    // Pictures and larger text go out after the cells, in one write.
                    let mut extra = kitty.sync(&pics, st.caps);
                    if scaled != scaled_last {
                        for t in &scaled {
                            extra.extend(osc66(t));
                        }
                        scaled_last = scaled;
                    }
                    if !extra.is_empty() {
                        let mut out = std::io::stdout().lock();
                        out.write_all(&extra)?;
                        out.flush()?;
                    }
                    if let Some(text) = text {
                        for s in screens.drain(..) {
                            let _ = s.send(json!({"ok": true, "screen": text}));
                        }
                    }
                    window.frame(t.elapsed());
                    last_draw = Instant::now();
                    dirty = resize_editors(&mut d, &mut st, sizes);
                }
                // Sleep until the next frame is due (when there's something to draw) or a while.
                let mut wait = if dirty || animating { budget.saturating_sub(last_draw.elapsed()) } else { Duration::from_millis(50) };
                if let Some(w) = wake {
                    wait = wait.min(Duration::from_millis(w.saturating_sub(started.elapsed().as_millis() as u64)));
                }
                let mut batch = Vec::new();
                if let Ok(ev) = rx.recv_timeout(wait) {
                    batch.push(ev);
                    batch.extend(rx.try_iter().take(10_000));
                }
                let resized = batch.iter().any(|e| matches!(e, Event::Term(TermEvent::Resize(..))));
                let n = apply_batch(batch, &mut d, &mut st, &hits, &mut screens);
                window.msgs += n;
                dirty |= n > 0;
                if let Some(p) = store.as_deref().filter(|_| st.store_writes != saved_writes) {
                    match save_store(p, &st) {
                        Ok(()) => saved_writes = st.store_writes,
                        Err(e) => st.status = Some(format!("saving the store: {e:#}")),
                    }
                }
                if resized {
                    if let Ok(size) = term.size() {
                        d.apply(&mut st, Msg::Screen { width: size.width, height: size.height });
                    }
                    // The cell size can change with the window (a font size change, another display).
                    d.apply(&mut st, caps(sizing));
                    scaled_last.clear();
                }
                if let Some(bin) = d.upgrade.take() {
                    let path = std::env::temp_dir().join(format!("fictty-state-{}.json", std::process::id()));
                    std::fs::write(&path, serde_json::to_vec(&st)?)?;
                    let mut out = std::io::stdout().lock();
                    out.write_all(&kitty.clear())?;
                    out.flush()?;
                    drop(out);
                    d.stop_streams();
                    let _ = std::fs::remove_file(&socket);
                    use std::os::unix::process::CommandExt;
                    // The same command line, as the new binary, told where the state is.
                    let mut cmd = Command::new(&bin);
                    cmd.args(std::env::args_os().skip(1)).env("FICTTY_RESTORE", &path);
                    // Only returns if the exec failed.
                    let err = cmd.exec();
                    st.status = Some(format!("upgrade to {bin} failed: {err}"));
                    serve(&socket, d.tx.clone())?;
                    dirty = true;
                }
                d.tick(&mut st);
                if watch && let Some(p) = &file {
                    let now = mtime(p);
                    if now != seen {
                        seen = now;
                        match load(p) {
                            Ok(ui) => d.apply(&mut st, Msg::Push { ui }),
                            Err(e) => st.status = Some(format!("{e:#}")),
                        }
                        dirty = true;
                    }
                }
                if let Some(stats) = window.report(st.stats.frames) {
                    d.apply(&mut st, Msg::Stats { stats });
                    dirty = true;
                }
            }
            Ok(())
        })();
        let _ = std::io::stdout().write_all(&kitty.clear());
        let _ = ratatui::crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        ratatui::restore();
        let _ = std::fs::remove_file(&socket);
        res
    }
}

/// Apply a batch of queued events. A source's value replaces the last one, so of several queued
/// values for one source only the newest is applied (the rest still count as received). Returns
/// how many events arrived.
fn apply_batch(batch: Vec<Event>, d: &mut Driver, st: &mut State, hits: &[Hit], screens: &mut Vec<Sender<Value>>) -> usize {
    let n = batch.len();
    let mut newest: BTreeMap<String, usize> = BTreeMap::new();
    for (i, ev) in batch.iter().enumerate() {
        if let Event::Msg(Msg::Data { name, .. }) = ev {
            newest.insert(name.clone(), i);
        }
    }
    for (i, ev) in batch.into_iter().enumerate() {
        match ev {
            Event::Msg(Msg::Data { name, .. }) if newest.get(&name) != Some(&i) => d.in_flight.remove(&name),
            Event::Msg(m) => {
                d.apply(st, m);
                true
            }
            Event::Request(Request::Upgrade { binary }, reply) => {
                let bin = binary.or_else(|| std::env::current_exe().ok().map(|p| p.display().to_string())).unwrap_or_default();
                let _ = reply.send(json!({"ok": true, "upgrading": bin}));
                d.upgrade = Some(bin);
                true
            }
            Event::Request(Request::Screen, reply) => {
                screens.push(reply);
                true
            }
            Event::Request(Request::Mouse { kind, x, y }, reply) => {
                let kind = match kind.as_str() {
                    "click" => MouseEventKind::Down(MouseButton::Left),
                    "up" => MouseEventKind::ScrollUp,
                    "down" => MouseEventKind::ScrollDown,
                    _ => MouseEventKind::Moved,
                };
                let msg = mouse(kind, x, y, hits, st);
                let what = msg.as_ref().map(|m| serde_json::to_value(m).unwrap_or_default());
                if let Some(m) = msg {
                    d.apply(st, m);
                }
                let _ = reply.send(json!({"ok": true, "msg": what}));
                true
            }
            Event::Request(req, reply) => {
                let _ = reply.send(d.request(st, req));
                true
            }
            Event::Term(TermEvent::Key(k)) if k.kind == KeyEventKind::Press => {
                if let Some(key) = key_name(k) {
                    d.apply(st, Msg::Key { key });
                }
                true
            }
            Event::Term(TermEvent::Mouse(m)) => {
                if let Some(msg) = mouse(m.kind, m.column, m.row, hits, st) {
                    d.apply(st, msg);
                }
                true
            }
            Event::Term(_) => true,
        };
    }
    n
}

/// A mouse event as a message, by what was drawn under it last frame (the innermost hit).
fn mouse(kind: MouseEventKind, x: u16, y: u16, hits: &[Hit], st: &State) -> Option<Msg> {
    let at = |want_row: bool| {
        hits.iter().rev().find(|h| h.rect.contains(ratatui::layout::Position { x, y }) && (!want_row || h.row.is_some()))
    };
    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let h = at(false)?;
            if h.text {
                return Some(Msg::Point { id: h.id.clone(), col: x - h.rect.x, row: y - h.rect.y });
            }
            Some(Msg::Click { id: h.id.clone(), row: h.row.filter(|r| *r < 1000), tab: h.tab })
        }
        MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
            let by = if kind == MouseEventKind::ScrollDown { 1 } else { -1 };
            let h = hits.iter().rev().find(|h| h.row.is_none() && h.rect.contains(ratatui::layout::Position { x, y }))?;
            Some(Msg::Scroll { id: h.id.clone(), by })
        }
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            let hover = at(true).map(|h| crate::state::Hover { id: h.id.clone(), row: h.row });
            (hover != st.hover).then_some(Msg::Hover { hover })
        }
        _ => None,
    }
}

/// Counts frames and messages over half-second windows.
struct Window {
    start: Instant,
    frames: u64,
    draw: Duration,
    max: Duration,
    msgs: usize,
}

impl Window {
    fn new() -> Window {
        Window { start: Instant::now(), frames: 0, draw: Duration::ZERO, max: Duration::ZERO, msgs: 0 }
    }

    fn frame(&mut self, took: Duration) {
        self.frames += 1;
        self.draw += took;
        self.max = self.max.max(took);
    }

    /// The window's numbers once it's half a second old (and a fresh window).
    fn report(&mut self, total: u64) -> Option<Stats> {
        let secs = self.start.elapsed().as_secs_f64();
        if secs < 0.5 {
            return None;
        }
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let stats = Stats {
            fps: self.frames as f64 / secs,
            draw_ms: if self.frames > 0 { ms(self.draw) / self.frames as f64 } else { 0.0 },
            max_ms: ms(self.max),
            msgs: self.msgs as f64 / secs,
            frames: total + self.frames,
        };
        *self = Window::new();
        Some(stats)
    }
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Listen for request lines; each gets one reply line.
fn serve(path: &Path, tx: Sender<Event>) -> Result<()> {
    if UnixStream::connect(path).is_ok() {
        bail!("a screen is already running on {} (set FICTTY_SOCKET for another)", path.display());
    }
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path).with_context(|| format!("binding {}", path.display()))?;
    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let mut w = match conn.try_clone() {
                    Ok(w) => w,
                    Err(_) => return,
                };
                for line in BufReader::new(conn).lines().map_while(Result::ok) {
                    let reply = match serde_json::from_str::<Request>(&line) {
                        Ok(req) => {
                            let (rtx, rrx) = mpsc::channel();
                            if tx.send(Event::Request(req, rtx)).is_err() {
                                return;
                            }
                            rrx.recv_timeout(Duration::from_secs(5)).unwrap_or(json!({"ok": false, "status": "timed out"}))
                        }
                        Err(e) => json!({"ok": false, "status": format!("bad request: {e}")}),
                    };
                    if writeln!(w, "{reply}").is_err() {
                        return;
                    }
                }
            });
        }
    });
    Ok(())
}

/// Send one request to a running screen.
pub fn send(socket: &Path, req: Value) -> Result<Value> {
    let mut s = UnixStream::connect(socket)
        .with_context(|| format!("no screen running on {} (start one: fictty run)", socket.display()))?;
    writeln!(s, "{req}")?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line)?;
    Ok(serde_json::from_str(&line)?)
}

/// A frame as ANSI text: an SGR (truecolour) wherever the style changes.
fn buffer_ansi(buf: &ratatui::buffer::Buffer) -> String {
    use ratatui::style::{Color, Modifier};
    let sgr = |c: &ratatui::buffer::Cell| {
        let mut p = vec!["0".to_string()];
        for (m, code) in [(Modifier::BOLD, "1"), (Modifier::DIM, "2"), (Modifier::ITALIC, "3"), (Modifier::UNDERLINED, "4"), (Modifier::REVERSED, "7"), (Modifier::CROSSED_OUT, "9")] {
            if c.modifier.contains(m) {
                p.push(code.into());
            }
        }
        if let Color::Rgb(r, g, b) = c.fg {
            p.push(format!("38;2;{r};{g};{b}"));
        }
        if let Color::Rgb(r, g, b) = c.bg {
            p.push(format!("48;2;{r};{g};{b}"));
        }
        format!("\x1b[{}m", p.join(";"))
    };
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut last = String::new();
        for x in 0..buf.area.width {
            let c = &buf[(x, y)];
            let s = sgr(c);
            if s != last {
                out.push_str(&s);
                last = s;
            }
            out.push_str(c.symbol());
        }
        out.push_str("\x1b[0m\n");
    }
    out
}

/// Larger text at a cell (kitty's text sizing, OSC 66), in its colours.
fn osc66(t: &view::Scaled) -> Vec<u8> {
    use ratatui::style::{Color, Modifier};
    let mut sgr = vec!["0".to_string()];
    if t.style.add_modifier.contains(Modifier::BOLD) {
        sgr.push("1".into());
    }
    if let Some(Color::Rgb(r, g, b)) = t.style.fg {
        sgr.push(format!("38;2;{r};{g};{b}"));
    }
    format!("\x1b[{};{}H\x1b[{}m\x1b]66;s={};{}\x07\x1b[0m", t.y + 1, t.x + 1, sgr.join(";"), t.scale, t.text).into_bytes()
}

fn buffer_text(buf: &ratatui::buffer::Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let line: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// A screen without a terminal (`App::headless`): send it keys and messages, read its frame and
/// its state. Everything the host sends arrives before the next read.
pub struct Headless {
    st: State,
    d: Driver,
    term: ratatui::Terminal<ratatui::backend::TestBackend>,
    notes: Receiver<Msg>,
}

impl Headless {
    /// Keys, by name (`j`, `tab`, `enter`, `c-s`), as if typed.
    pub fn keys(&mut self, keys: &[&str]) -> &mut Headless {
        for k in keys {
            self.msg(Msg::Key { key: (*k).into() });
        }
        self
    }

    /// Any message, as if it came from the socket.
    pub fn msg(&mut self, msg: Msg) -> &mut Headless {
        self.drain();
        self.d.apply(&mut self.st, msg);
        self
    }

    /// The state, as `get --state` shows it.
    pub fn state(&mut self) -> &State {
        self.drain();
        &self.st
    }

    /// The frame as text, one line per row (trailing spaces trimmed).
    pub fn screen(&mut self) -> String {
        self.frame();
        buffer_text(self.term.backend().buffer())
    }

    /// The frame with its colours and attributes, as ANSI (truecolour) escapes.
    pub fn ansi(&mut self) -> String {
        self.frame();
        buffer_ansi(self.term.backend().buffer())
    }

    fn drain(&mut self) {
        while let Ok(m) = self.notes.try_recv() {
            self.d.apply(&mut self.st, m);
        }
    }

    fn frame(&mut self) {
        self.drain();
        let mut sizes = Vec::new();
        let st = &self.st;
        let _ = self.term.draw(|f| sizes = view::draw(st, f).sizes);
        if resize_editors(&mut self.d, &mut self.st, sizes) {
            let st = &self.st;
            let _ = self.term.draw(|f| {
                view::draw(st, f);
            });
        }
    }
}

fn agent() -> String {
    "agent".into()
}

/// A socket request as a `$log` entry (op, what), for the requests that change the screen.
fn activity(st: &State, req: &Request) -> Option<(&'static str, String)> {
    let short = |s: &str| {
        let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
        if one.chars().count() > 40 { format!("{}…", one.chars().take(39).collect::<String>()) } else { one }
    };
    Some(match req {
        Request::Push { ui } => ("push", format!("a new UI (v{}), {} sources", st.version + 1, ui.data.len())),
        Request::Patch { id, .. } => ("patch", format!("node {id}")),
        Request::Write { key, write, if_rev, .. } => {
            use crate::store::Write;
            let rev = if_rev.map(|r| format!(" @{r}")).unwrap_or_default();
            match write {
                Write::Put { value: Value::String(t) } => ("put", format!("{key}{rev}: \"{}\"", short(t))),
                Write::Put { value: Value::Array(a) } => ("put", format!("{key}{rev}: {} items", a.len())),
                Write::Put { .. } => ("put", format!("{key}{rev}")),
                Write::Merge { .. } => ("merge", format!("{key}{rev}")),
                Write::Edit { changes } => {
                    let first = changes.first().map(|c| format!(": \"{}\"", short(&c.2))).unwrap_or_default();
                    ("edit", format!("{key}{rev}{first}"))
                }
            }
        }
        Request::Set { id, selected, tab } => {
            let what = match (selected, tab) {
                (Some(r), _) => format!("{id} row {r}"),
                (_, Some(t)) => format!("{id} tab {t}"),
                _ => id.clone(),
            };
            ("set", what)
        }
        Request::Key { key } => ("key", key.clone()),
        Request::Theme { add, choose } => ("theme", choose.clone().unwrap_or_else(|| format!("{} loaded", add.len()))),
        Request::Layers { layers } => ("layers", format!("{} layers", layers.len())),
        Request::Toast { text, .. } => ("toast", short(text)),
        Request::Click { id, .. } => ("click", id.clone()),
        _ => return None,
    })
}

/// Why a write was refused, in a few words for the log: "at rev 3" for a stale one.
fn refusal(reply: &Value) -> String {
    match reply["current"]["rev"].as_u64() {
        Some(rev) => format!("stale: at rev {rev}"),
        None => reply["status"].as_str().unwrap_or("refused").to_string(),
    }
}

/// macOS can't wait on input from the generic `/dev/tty` (kqueue refuses it), so a screen whose
/// input is `/dev/tty` would spin on its first read. Say so instead; the terminal's own device
/// (`/dev/ttys003`) works.
fn generic_tty() -> Result<()> {
    if cfg!(target_os = "macos") {
        // SAFETY: ttyname returns null or a pointer to a static string, read before any other call.
        let name = unsafe { libc::ttyname(0) };
        if !name.is_null() && unsafe { std::ffi::CStr::from_ptr(name) }.to_bytes() == b"/dev/tty" {
            anyhow::bail!("input is /dev/tty, which macOS can't wait on; run fictty with the terminal's own device (`tty` in that terminal names it) as its input");
        }
    }
    Ok(())
}

/// The local wall-clock time, `HH:MM:SS`, for the activity log.
fn clock() -> String {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as libc::time_t;
    // SAFETY: localtime_r writes only into `tm`, which outlives the call.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&now, &mut tm) }.is_null() {
        return format!("{:02}:{:02}:{:02}", now / 3600 % 24, now / 60 % 60, now % 60);
    }
    format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
}

/// Tells each editor the size it was drawn at, where that differs from its own. True when any
/// changed (the next frame should draw).
fn resize_editors(d: &mut Driver, st: &mut State, sizes: Vec<(String, u16, u16)>) -> bool {
    let mut changed = false;
    for (id, width, height) in sizes {
        let own = crate::state::field_view(st, &id).map(|(_, v)| (v.viewport.width, v.viewport.height));
        if own.is_some_and(|o| o != (width.max(1), height.max(1))) {
            d.apply(st, Msg::Size { id, width, height });
            changed = true;
        }
    }
    changed
}

pub fn key_name(k: KeyEvent) -> Option<String> {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    let shift = k.modifiers.contains(KeyModifiers::SHIFT);
    let alt = k.modifiers.contains(KeyModifiers::ALT);
    // Shift with a movement key extends a selection in an editor: `s-left`, `s-end`.
    let moves = matches!(k.code, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End);
    if shift && moves {
        let base = key_name(KeyEvent::new(k.code, KeyModifiers::NONE))?;
        return Some(format!("s-{base}"));
    }
    if let KeyCode::Char(c) = k.code
        && alt
        && !ctrl
    {
        return Some(format!("a-{c}"));
    }
    Some(match k.code {
        KeyCode::Char(c) if ctrl => format!("c-{c}"),
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Tab => "tab".into(),
        KeyCode::BackTab => "s-tab".into(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "esc".into(),
        KeyCode::Backspace => "bs".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Home => "home".into(),
        KeyCode::End => "end".into(),
        KeyCode::Delete => "del".into(),
        KeyCode::PageUp => "pgup".into(),
        KeyCode::PageDown => "pgdn".into(),
        _ => return None,
    })
}
