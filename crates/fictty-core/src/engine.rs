//! The engine: everything that runs a screen but doesn't draw it. It owns the control socket,
//! child processes, the host and the watched file, turns what happens into `Msg`s for `update`,
//! and carries out the `Effect`s that come back. What draws (a terminal, a window) starts an
//! engine (`Live`), posts its input (`Inbox`) and reads the state when it draws a frame; tests and
//! `fictty render` drive one without a thread (`Headless`).

use crate::host::{Host, NoHost, Notifier};
use crate::model::{Source, ThemeSpec, Ui};
use crate::state::{Effect, Msg, State, Stats, narrow, update};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// One request line on the control socket.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Request {
    Push { ui: Ui },
    Patch { id: String, node: crate::model::Node },
    Key { key: String },
    /// End the screen, wherever the focus is (a key like `c-c` is an editor's copy while one
    /// has the focus).
    Quit,
    /// Write one store entry (`put`, `merge` or `edit`), refused when `if_rev` is stale.
    Write {
        key: String,
        #[serde(flatten)]
        write: crate::store::Write,
        #[serde(default)]
        if_rev: Option<u64>,
        #[serde(default = "agent")]
        by: String,
        /// The write lasts this long (ms): then the entry, or the keys a merge set, go.
        #[serde(default)]
        for_ms: Option<u64>,
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
        /// Select the row with this key, once it's there.
        #[serde(default)]
        key: Option<String>,
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
    /// The shown nodes, in order: kind, id, the text each shows, focus, selection, rectangle.
    Readback,
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

/// A rectangle on the screen, in cells: where a frame laid a node out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect { x, y, width, height }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && y >= self.y && x < self.x.saturating_add(self.width) && y < self.y.saturating_add(self.height)
    }
}

/// What a cell belongs to: a node, and maybe one of its rows (or bars, or points) or tabs.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub rect: Rect,
    pub id: String,
    pub row: Option<usize>,
    pub tab: Option<usize>,
    /// The text area of an editor or input: a click there places the caret.
    pub text: bool,
}

/// What a frame drew, besides what's shown: what the engine needs back from it.
#[derive(Clone, Debug, Default)]
pub struct Drawn {
    /// What's under each cell, for turning the mouse into messages.
    pub hits: Vec<Hit>,
    /// Where each node with an id was laid out, for the readback.
    pub rects: Vec<(String, Rect)>,
    /// The cells each editor got (id, width, height), and the rows each list showed: an editor
    /// told its size keeps the caret in view.
    pub sizes: Vec<(String, u16, u16)>,
    /// The area each node whose `when` reads `$width` or `$height` was laid out in.
    pub areas: Vec<(String, u16, u16)>,
    /// When (on the logical clock) the frame changes by itself next: a toast going, a spinner
    /// turning.
    pub wake: Option<u64>,
    /// The nodes some of whose text the frame cut to fit (ending it in `…`).
    pub cut: Vec<String>,
    /// The font files it couldn't load, and why.
    pub fonts: Vec<(String, String)>,
}

/// The mouse, as whatever draws reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    /// The left button down, with Shift or not, and with ⌘ (⌥ in a terminal) or not: a press
    /// on a row with either picks rows (`Msg::Pick`).
    Press { shift: bool, toggle: bool },
    /// Moved with the button held.
    Drag,
    Release,
    Move,
    WheelUp,
    WheelDown,
}

impl Pointer {
    /// A pointer by the names `Headless::mouse` takes: `press`, `s-press`, `d-press` (⌘),
    /// `drag`, `release`, `move`, `wheel-up`, `wheel-down`.
    pub fn named(kind: &str) -> Pointer {
        match kind {
            "press" => Pointer::Press { shift: false, toggle: false },
            "s-press" => Pointer::Press { shift: true, toggle: false },
            "d-press" | "a-press" => Pointer::Press { shift: false, toggle: true },
            "drag" => Pointer::Drag,
            "release" => Pointer::Release,
            "wheel-up" => Pointer::WheelUp,
            "wheel-down" => Pointer::WheelDown,
            _ => Pointer::Move,
        }
    }

    /// A pointer by the socket's names: `click` (`s-click` with ⇧, `d-click` with ⌘), `up`,
    /// `down`, `move`.
    fn requested(kind: &str) -> Pointer {
        match kind {
            "click" => Pointer::Press { shift: false, toggle: false },
            "s-click" => Pointer::Press { shift: true, toggle: false },
            "d-click" | "a-click" => Pointer::Press { shift: false, toggle: true },
            "up" => Pointer::WheelUp,
            "down" => Pointer::WheelDown,
            _ => Pointer::Move,
        }
    }
}

/// Input from whatever draws the screen: keys by name, the mouse at a cell, pastes, a new size.
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Key(String),
    Pointer { kind: Pointer, x: u16, y: u16 },
    Paste(String),
    Resize { width: u16, height: u16 },
}

/// Everything the engine reacts to arrives on one queue: input, data, socket requests.
enum Event {
    Msg(Msg),
    Request(Request, Sender<Value>),
    Input(Input),
}

/// Where whatever draws a running screen posts its input: it joins the engine's queue, in order,
/// with everything else.
#[derive(Clone)]
pub struct Inbox(Sender<Event>);

impl Inbox {
    /// False once the engine has stopped.
    pub fn input(&self, input: Input) -> bool {
        self.0.send(Event::Input(input)).is_ok()
    }

    pub fn msg(&self, msg: Msg) -> bool {
        self.0.send(Event::Msg(msg)).is_ok()
    }
}

/// A screen's way of drawing itself for `Headless`: the terminal's grid, or the window under
/// GPUI's test platform. Without one, a headless screen reads back its nodes as an outline.
pub trait Frame {
    /// Draw the state.
    fn draw(&mut self, st: &State) -> Drawn;
    /// Input for a frame that takes it itself, as a window does (keys as the OS's keystrokes,
    /// the mouse to whatever element is under it, a paste through the clipboard): what it sent
    /// comes back from `messages`. False leaves it to the engine (keys by name, the mouse on the
    /// last frame's hit regions).
    fn input(&mut self, _input: &Input) -> bool {
        false
    }
    /// The messages the frame sent since it was last asked: a window's clicks, its cell size.
    fn messages(&mut self) -> Vec<Msg> {
        Vec::new()
    }
    /// The screen copied `text`: a window puts it on its clipboard.
    fn copied(&mut self, _text: &str) {}
    /// The screen is now `width` × `height` cells.
    fn resize(&mut self, width: u16, height: u16);
    /// The last frame as text, one line per row.
    fn text(&self) -> String;
    /// The last frame with its colours, as ANSI escapes.
    fn ansi(&self) -> String {
        self.text()
    }
    /// Draw every frame from now on as a terminal would show it too (the fuzz driver's check):
    /// true when that starts now.
    fn wire(&mut self) -> bool {
        false
    }
    /// The last frame as the terminal shows it, once `wire` started.
    fn terminal(&self) -> Option<String> {
        None
    }
}

/// A mouse event as a message, by what was drawn under it last frame (the innermost hit).
/// A press in an editor's text starts a drag (`drag`), and moves with the button held stretch its
/// selection until the button comes up.
fn pointer(kind: Pointer, x: u16, y: u16, hits: &[Hit], st: &State, drag: &mut Option<(String, Rect)>) -> Option<Msg> {
    let at = |want_row: bool| hits.iter().rev().find(|h| h.rect.contains(x, y) && (!want_row || h.row.is_some()));
    match kind {
        Pointer::Drag if drag.is_some() => {
            let (id, r) = drag.as_ref()?;
            let col = x.saturating_sub(r.x).min(r.width.saturating_sub(1));
            let row = y.saturating_sub(r.y).min(r.height);
            Some(Msg::Drag { id: id.clone(), col, row })
        }
        Pointer::Release => {
            *drag = None;
            None
        }
        Pointer::Press { shift, toggle } => {
            let h = at(false)?;
            if h.text {
                *drag = Some((h.id.clone(), h.rect));
                return Some(Msg::Point { id: h.id.clone(), col: x - h.rect.x, row: y - h.rect.y, extend: shift });
            }
            if let Some(row) = h.row.filter(|r| *r < 1000 && (shift || toggle)) {
                let how = if toggle { crate::state::Pick::Toggle } else { crate::state::Pick::Range };
                return Some(Msg::Pick { id: h.id.clone(), row, how });
            }
            Some(Msg::Click { id: h.id.clone(), row: h.row.filter(|r| *r < 1000), tab: h.tab })
        }
        Pointer::WheelDown | Pointer::WheelUp => {
            let by = if kind == Pointer::WheelDown { 1 } else { -1 };
            let h = hits.iter().rev().find(|h| h.row.is_none() && h.rect.contains(x, y))?;
            Some(Msg::Scroll { id: h.id.clone(), by })
        }
        Pointer::Move | Pointer::Drag => {
            let hover = at(true).map(|h| crate::state::Hover { id: h.id.clone(), row: h.row });
            (hover != st.hover).then_some(Msg::Hover { hover })
        }
    }
}

/// An input as a message for `update`, by the last frame's hits.
fn input_msg(input: Input, hits: &[Hit], st: &State, drag: &mut Option<(String, Rect)>) -> Option<Msg> {
    match input {
        Input::Key(key) => Some(Msg::Key { key }),
        Input::Pointer { kind, x, y } => pointer(kind, x, y, hits, st, drag),
        Input::Paste(text) => Some(Msg::Paste { text }),
        Input::Resize { width, height } => Some(Msg::Screen { width, height }),
    }
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

/// Whether a source runs a command to get its value.
fn is_command(src: &Source) -> bool {
    src.stream.is_some() || src.cmd.is_some() || src.shell.is_some()
}

/// Why a command didn't run on a screen whose commands are off.
const OFF: &str = "commands are off";

/// Runs `update` and carries out its effects. Fetches and runs go to threads that report back as
/// messages; `blocking` (for headless renders) does them inline instead.
struct Driver {
    tx: Sender<Event>,
    in_flight: BTreeSet<String>,
    /// Sources asked for while a fetch of them was under way (a push changed their args): fetched
    /// again when it lands, so the screen ends on what the newest definition gives.
    again: BTreeSet<String>,
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
    /// What the screen put on the clipboard and nobody has taken yet, oldest first.
    copied: Vec<String>,
    /// Each hosted entry's revision as the host has it (loaded or saved), and when one that
    /// differs last changed.
    kept: BTreeMap<String, u64>,
    /// The answer to the request being handled (a write's new revision, or why not).
    reply: Option<Value>,
    changed: BTreeMap<String, (u64, Instant)>,
    /// Whether `run` actions and command sources (`cmd`, `shell`, `stream`) run. Off, each
    /// fails with "commands are off", as a test or a fuzzer wants.
    commands: bool,
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
        Driver { tx, in_flight: BTreeSet::new(), again: BTreeSet::new(), last_fetch: BTreeMap::new(), streams: BTreeMap::new(), quit: false, blocking, upgrade: None, started: Instant::now(), host, copied: Vec::new(), reply: None, kept: BTreeMap::new(), changed: BTreeMap::new(), commands: true }
    }

    fn apply(&mut self, st: &mut State, msg: Msg) {
        let mut refetch = None;
        if let Msg::Data { name, .. } = &msg {
            self.in_flight.remove(name);
            if self.again.remove(name) {
                refetch = Some(name.clone());
            }
        }
        // What the host just gave is what it has: nothing to save back.
        let loaded = match &msg {
            Msg::Loaded { key, result: Ok(_) } => Some(key.clone()),
            _ => None,
        };
        // Messages carry the time they arrived (an editor groups typing into undo steps by it; a
        // toast lasts from then). Headless renders keep the clock still, so they come out the
        // same every time.
        if !matches!(msg, Msg::Tick { .. }) && !self.blocking {
            update(st, Msg::Tick { now_ms: self.started.elapsed().as_millis() as u64 });
        }
        let mut queue: Vec<Effect> = update(st, msg);
        while let Some(fx) = queue.pop() {
            match fx {
                Effect::Run(argv) if !self.commands => queue.extend(update(st, Msg::Ran { argv, result: Err(OFF.into()) })),
                Effect::Fetch(name) if !self.commands && crate::state::source_now(st, &name).is_some_and(|s| is_command(&s)) => {
                    queue.extend(update(st, Msg::Data { name, result: Err(OFF.into()) }))
                }
                Effect::Quit => self.quit = true,
                Effect::Reply(r) => self.reply = Some(r),
                // Whatever draws puts it on the clipboard (`Live::take_copied`); a headless
                // screen keeps it, for tests to read.
                Effect::Copy(text) => self.copied.push(text),
                Effect::Load(path) => match load(Path::new(&path)) {
                    Ok(ui) => queue.extend(update(st, Msg::Push { ui })),
                    Err(e) => st.status = Some(format!("{e:#}")),
                },
                Effect::Fetch(name) if self.blocking => {
                    let Some(src) = crate::state::source_now(st, &name) else { continue };
                    let result = fetch(&src, &*self.host);
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
                Effect::Read(key) => {
                    // What the person typed and the screen hasn't saved yet goes to the host
                    // first, so the load that follows has both sides (the host merges); loading
                    // first would put the host's text over the typing.
                    let unsaved = self.unsaved(st, &key);
                    if self.blocking {
                        let saved = unsaved.map_or(Ok(()), |v| self.host.save(&key, &v));
                        let result = saved.map_err(|e| format!("saving before a reload: {e}")).and_then(|()| self.host.load(&key));
                        let ok = result.is_ok();
                        queue.extend(update(st, Msg::Loaded { key: key.clone(), result }));
                        // What the host just gave is what it has, as for a load from a thread.
                        if let Some(e) = st.store.get(&key).filter(|_| ok) {
                            self.kept.insert(key, e.rev);
                        }
                    } else {
                        let (tx, host) = (self.tx.clone(), self.host.clone());
                        std::thread::spawn(move || {
                            let saved = unsaved.map_or(Ok(()), |v| host.save(&key, &v));
                            let result = saved.map_err(|e| format!("saving before a reload: {e}")).and_then(|()| host.load(&key));
                            let _ = tx.send(Event::Msg(Msg::Loaded { key, result }));
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
        if let Some(key) = loaded
            && let Some(e) = st.store.get(&key)
        {
            self.kept.insert(key, e.rev);
        }
        if let Some(name) = refetch {
            self.spawn_fetch(st, name);
        }
        self.reap(st);
    }

    /// A hosted entry's value when it has changed since it was last saved or loaded, noted as
    /// saved (the caller saves it).
    fn unsaved(&mut self, st: &State, key: &str) -> Option<Value> {
        let e = st.store.get(key).filter(|_| crate::state::hosted(st, key))?;
        if self.kept.get(key) == Some(&e.rev) {
            return None;
        }
        self.kept.insert(key.into(), e.rev);
        self.changed.remove(key);
        Some(match &e.body {
            crate::store::Body::Text { doc } => Value::String(doc.text.to_string()),
            crate::store::Body::Value { value } => value.clone(),
        })
    }

    /// Save hosted entries that changed and have been still for `settle` (a headless screen
    /// passes zero and saves at once).
    fn keep(&mut self, st: &mut State, settle: Duration) {
        if st.hosted.is_empty() {
            return;
        }
        let mut failed = Vec::new();
        for (key, e) in st.store.iter().filter(|(k, _)| crate::state::hosted(st, k)) {
            if self.kept.get(key) == Some(&e.rev) {
                self.changed.remove(key);
                continue;
            }
            let since = match self.changed.get(key) {
                Some((rev, t)) if *rev == e.rev => *t,
                _ => {
                    self.changed.insert(key.clone(), (e.rev, Instant::now()));
                    Instant::now()
                }
            };
            if since.elapsed() < settle {
                continue;
            }
            let value = match &e.body {
                crate::store::Body::Text { doc } => Value::String(doc.text.to_string()),
                crate::store::Body::Value { value } => value.clone(),
            };
            self.kept.insert(key.clone(), e.rev);
            self.changed.remove(key);
            if self.blocking {
                if let Err(err) = self.host.save(key, &value) {
                    failed.push(format!("saving {key}: {err}"));
                }
            } else {
                let (tx, host, key) = (self.tx.clone(), self.host.clone(), key.clone());
                std::thread::spawn(move || {
                    if let Err(err) = host.save(&key, &value) {
                        let _ = tx.send(Event::Msg(Msg::Toast { text: format!("saving {key}: {err}"), tone: Some("neg".into()), ms: Some(6000) }));
                    }
                });
            }
        }
        if let Some(why) = failed.pop() {
            st.status = Some(why);
        }
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
        let Some(src) = crate::state::source_now(st, &name) else { return };
        // Its first fetch said why (see `apply`); a refresh says nothing more.
        if !self.commands && is_command(&src) {
            return;
        }
        self.last_fetch.insert(name.clone(), Instant::now());
        if !self.in_flight.insert(name.clone()) {
            self.again.insert(name);
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
        let (started, blocking) = (self.started, self.blocking);
        // A headless screen's clock stands still, and its log has no wall-clock times, so the
        // same requests leave the same state.
        let log = |st: &mut State, op: &str, what: String| {
            if !blocking {
                update(st, Msg::Tick { now_ms: started.elapsed().as_millis() as u64 });
            }
            let at = if blocking { String::new() } else { clock() };
            update(st, Msg::Activity { op: op.into(), what, at });
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
            Request::Quit => {
                self.quit = true;
                return json!({"ok": true, "version": st.version, "status": "quitting"});
            }
            Request::Read { key } => {
                return st.store.get(&key).map_or_else(|| json!({"ok": false, "status": format!("no entry {key}")}), |e| e.read());
            }
            Request::Write { key, write, if_rev, by, for_ms } => {
                // Through the driver, so what the write sets off (a refetch) happens too.
                self.apply(st, Msg::Write { key, write, if_rev, by, for_ms });
                let mut reply = self.reply.take().unwrap_or_else(|| json!({"ok": false}));
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
            Request::Set { id, selected, tab, key } => Msg::Set { id, selected, tab, key },
            Request::Theme { add, choose } => Msg::Theme { add, choose },
            Request::Layers { layers } => Msg::Layers { layers },
            Request::Toast { text, tone, ms } => Msg::Toast { text, tone, ms },
            Request::Click { id, row, tab } => Msg::Click { id, row, tab },
            // The loop answers these itself (they need the last frame).
            Request::Mouse { .. } | Request::Screen | Request::Readback | Request::Upgrade { .. } => return json!({"ok": false}),
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
    // The host keeps its own entries, and a mixed editor's own text is never kept.
    let mine: BTreeMap<&String, &crate::store::Entry> =
        st.store.iter().filter(|(k, _)| !crate::state::hosted(st, k) && !k.starts_with(crate::state::MIXED)).collect();
    std::fs::write(&tmp, serde_json::to_vec_pretty(&mine)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Themes from a file: a JSON object of theme specs by name.
pub fn load_themes(path: &Path) -> Result<BTreeMap<String, crate::model::ThemeSpec>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut themes: BTreeMap<String, crate::model::ThemeSpec> = serde_json::from_str(&raw).with_context(|| format!("{} is not a set of themes", path.display()))?;
    // A font file is found beside the themes, wherever the screen runs from.
    let dir = path.parent().unwrap_or(Path::new("."));
    for t in themes.values_mut() {
        for f in t.fonts.values_mut() {
            if crate::style::is_font_file(f) && Path::new(f.as_str()).is_relative() {
                *f = dir.join(&*f).to_string_lossy().into_owned();
            }
        }
    }
    Ok(themes)
}

/// A screen to run: its UI (a value, or a file), and what it starts from. Whatever draws it starts
/// its engine (`live`); tests drive it without one (`headless`).
///
/// ```no_run
/// # let ui: fictty_core::model::Ui = serde_json::from_str(r#"{"root": {"type": "text", "text": "hi"}}"#).unwrap();
/// let live = fictty_core::App::new(ui).socket("/tmp/my.sock").live(80, 24).unwrap();
/// let screen = live.spawn(|| true);
/// println!("{}", fictty_core::readback::outline(&screen.lock().unwrap().readback()));
/// ```
pub struct App {
    ui: Option<Ui>,
    file: Option<PathBuf>,
    watch: bool,
    socket: PathBuf,
    restore: Option<PathBuf>,
    store: Option<PathBuf>,
    themes: BTreeMap<String, ThemeSpec>,
    /// The file the themes came from, reloaded with the UI's when it's watched.
    themes_path: Option<PathBuf>,
    theme: Option<String>,
    host: Arc<dyn Host>,
    commands: bool,
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

    /// Whether `run` actions and command sources run (they do by default). Off, each fails with
    /// "commands are off" and nothing is started: for tests and fuzzing a UI whose buttons run
    /// scripts.
    pub fn commands(mut self, on: bool) -> App {
        self.commands = on;
        self
    }

    /// A screen waiting for a UI to be pushed.
    pub fn empty() -> App {
        App {
            ui: None,
            file: None,
            watch: false,
            socket: default_socket(),
            restore: None,
            commands: true,
            store: None,
            themes: BTreeMap::new(),
            themes_path: None,
            theme: None,
            host: Arc::new(NoHost),
        }
    }

    /// Reload the UI file whenever it changes, and the themes file too.
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

    /// Add the themes in a JSON file (theme specs by name). A watched screen reloads them when
    /// the file changes.
    pub fn themes_file(mut self, path: &Path) -> Result<App> {
        self.themes_path = Some(path.to_path_buf());
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
        st.hosted = self.host.keys();
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

    /// Without a thread or a socket: a screen of `width`×`height` cells, for tests and `fictty
    /// render`. Sources are fetched and host actions done inline. Its `screen` is the readback
    /// as an outline; `headless_with` gives it a frame to draw.
    pub fn headless(self, width: u16, height: u16) -> Result<Headless> {
        self.headless_frame(width, height, None)
    }

    /// As `headless`, drawn by `frame` (the terminal's grid).
    pub fn headless_with(self, width: u16, height: u16, frame: Box<dyn Frame>) -> Result<Headless> {
        self.headless_frame(width, height, Some(frame))
    }

    fn headless_frame(self, width: u16, height: u16, frame: Option<Box<dyn Frame>>) -> Result<Headless> {
        let mut st = self.start()?;
        let ui = self.first_ui(&st)?;
        let (tx, _) = mpsc::channel();
        let (ntx, notes) = mpsc::channel();
        self.host.start(Notifier::new(ntx));
        let mut d = Driver::new(tx, true, self.host.clone());
        d.commands = self.commands;
        d.apply(&mut st, Msg::Screen { width, height });
        d.apply(&mut st, Msg::Push { ui });
        let mut h = Headless {
            st,
            d,
            frame,
            size: (width, height),
            notes,
            drawn: false,
            hits: Vec::new(),
            rects: Vec::new(),
            cut: Vec::new(),
            drag: None,
            #[cfg(feature = "fuzz")]
            host_act: None,
        };
        // One frame to learn each editor's size, so keys scroll it as they would on screen.
        h.frame();
        Ok(h)
    }

    /// Start the screen's engine: the socket answers, sources run, the host starts, a restored
    /// state (after an upgrade, `$FICTTY_RESTORE`) carries on. `width` × `height` is the screen's
    /// size in cells to start with. Whatever draws it pumps it (`Live::pump`), or hands it its
    /// own thread (`Live::spawn`).
    pub fn live(mut self, width: u16, height: u16) -> Result<Live> {
        if self.restore.is_none() {
            self.restore = std::env::var_os("FICTTY_RESTORE").map(PathBuf::from);
        }
        // One running screen to a store file: a second would write over the first's changes.
        let held = self.store.as_deref().map(hold_store).transpose()?;
        // Restoring (after an upgrade): the saved state, every selection and focus as it was.
        let mut st = self.start()?;
        let saved_writes = st.store_writes;
        let ui = self.first_ui(&st)?;
        // A restored screen's streams and timers died with the old process: its first push
        // fetches every source again.
        st.ui.data.clear();
        let (tx, rx) = mpsc::channel();
        // Commands the screen runs (sources, `run`) can talk back to it: a helper script reads the
        // store with `$FICTTY read …`. Set before any thread starts.
        // SAFETY: called before the screen starts threads of its own, and before whatever draws
        // it starts any that read the environment.
        unsafe {
            std::env::set_var("FICTTY_SOCKET", &self.socket);
            std::env::remove_var("FICTTY_RESTORE");
            if let Ok(me) = std::env::current_exe() {
                std::env::set_var("FICTTY", me);
            }
        }
        let bound = serve(&self.socket, tx.clone())?;
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
        self.host.start(Notifier::new(ntx));
        let mut d = Driver::new(tx, false, self.host.clone());
        d.commands = self.commands;
        // A restored state's clock carries on from where it was.
        d.started = Instant::now().checked_sub(Duration::from_millis(st.now_ms)).unwrap_or_else(Instant::now);
        // The size before the first push, as a headless screen has it: a UI whose `when` reads
        // `$screen` starts on the right layout, and its starting focus on a node that's shown.
        d.apply(&mut st, Msg::Screen { width, height });
        d.apply(&mut st, Msg::Push { ui });
        let here = match &self.file {
            Some(f) => std::fs::canonicalize(f).ok().and_then(|f| f.parent().map(Path::to_path_buf)).unwrap_or_default(),
            None => std::env::current_dir().unwrap_or_default(),
        };
        let bin = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "fictty".into());
        let sock = std::fs::canonicalize(&self.socket).unwrap_or_else(|_| self.socket.clone());
        d.apply(&mut st, Msg::Host { bin, socket: sock.display().to_string(), dir: here.display().to_string() });
        let seen = self.file.as_deref().filter(|_| self.watch).and_then(mtime);
        let themes_file = self.themes_path.filter(|_| self.watch).map(|p| {
            let seen = mtime(&p);
            (p, seen)
        });
        Ok(Live {
            st,
            d,
            rx: Some(rx),
            screens: Vec::new(),
            meter: Meter::new(),
            socket: self.socket,
            hits: Vec::new(),
            rects: Vec::new(),
            cut: Vec::new(),
            drag: None,
            wake: None,
            last_tick: Instant::now(),
            file: self.file.filter(|_| self.watch),
            seen,
            themes_file,
            store: self.store,
            saved_writes,
            held,
            bound,
        })
    }
}

/// Hold the store file's lock for as long as the screen runs, or say which screen holds it. The
/// lock is an advisory one the system lets go of however the process ends, on a file in the
/// temporary directory named from the store's full path: not beside the store, where a synced
/// folder (Dropbox) may replace the file under the lock, and the store itself is replaced on
/// every save.
fn hold_store(store: &Path) -> Result<std::fs::File> {
    let path = lock_path(store);
    let f = std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(&path).with_context(|| format!("opening {}", path.display()))?;
    match f.try_lock() {
        Ok(()) => {
            use std::io::{Seek, Write};
            let mut f = f;
            f.set_len(0)?;
            f.seek(std::io::SeekFrom::Start(0))?;
            write!(f, "{}\n{}\n", std::process::id(), store.display())?;
            Ok(f)
        }
        Err(_) => {
            let pid = std::fs::read_to_string(&path).unwrap_or_default();
            let pid = pid.lines().next().unwrap_or_default().to_string();
            bail!("{} is in use by another fictty screen (pid {}): end it first (`fictty quit` on its socket)", store.display(), pid.trim())
        }
    }
}

/// The lock file for a store: in the temporary directory, named from a hash (FNV-1a, the same
/// in every build) of the store's full path, so every screen on that store finds the same one.
fn lock_path(store: &Path) -> PathBuf {
    let full = match (store.parent().filter(|p| !p.as_os_str().is_empty()).map(std::fs::canonicalize), store.file_name()) {
        (Some(Ok(dir)), Some(name)) => dir.join(name),
        (None, Some(name)) => std::env::current_dir().map(|d| d.join(name)).unwrap_or_else(|_| store.to_path_buf()),
        _ => store.to_path_buf(),
    };
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in full.as_os_str().as_encoded_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    std::env::temp_dir().join(format!("fictty-store-{h:016x}.lock"))
}
/// A running screen: the state and the engine (sources, streams, the host, the socket, the
/// watched file, the store file), with no drawing. Whatever draws it reads `state` for each frame,
/// tells it what the frame drew (`drew`), and answers `screen` requests from it.
pub struct Live {
    st: State,
    d: Driver,
    /// The queue, until `spawn` hands it to the engine's thread.
    rx: Option<Receiver<Event>>,
    /// `screen` requests waiting for the next frame.
    screens: Vec<Sender<Value>>,
    meter: Meter,
    socket: PathBuf,
    /// The last frame's hit regions and node rectangles.
    hits: Vec<Hit>,
    rects: Vec<(String, Rect)>,
    /// The nodes the last frame cut text in.
    cut: Vec<String>,
    /// An editor a press started a drag in.
    drag: Option<(String, Rect)>,
    /// When the last frame said it would change by itself.
    wake: Option<u64>,
    /// The last tick for an animation.
    last_tick: Instant,
    /// The UI file, when it's watched, and when it last changed.
    file: Option<PathBuf>,
    seen: Option<SystemTime>,
    /// The themes file, when the UI's is watched, and when it last changed.
    themes_file: Option<(PathBuf, Option<SystemTime>)>,
    /// The store's file, and the writes it holds.
    store: Option<PathBuf>,
    saved_writes: u64,
    /// The socket file this screen made (its device and inode): the one it removes, and no
    /// other screen's that took the path since.
    bound: (u64, u64),
    /// The store file's lock, held while the screen runs.
    #[allow(dead_code)]
    held: Option<std::fs::File>,
}

/// How often an animation moves (120 fps).
const FRAME: Duration = Duration::from_micros(8_333);

impl Live {
    /// Where input for this screen goes.
    pub fn inbox(&self) -> Inbox {
        Inbox(self.d.tx.clone())
    }

    /// Take in everything that has arrived (sources, the socket, the host, input) without
    /// waiting, and do the engine's chores. Whether the screen may have changed.
    pub fn pump(&mut self) -> bool {
        let batch: Vec<Event> = self.rx.as_ref().map(|rx| rx.try_iter().take(10_000).collect()).unwrap_or_default();
        self.step(batch)
    }

    /// A batch of events, then the chores: hosted entries saved, the store file written, sources
    /// that are due fetched, the watched file reloaded, the clock moved for whatever animates.
    fn step(&mut self, batch: Vec<Event>) -> bool {
        let n = apply_batch(batch, &mut self.d, &mut self.st, (&self.hits, &self.rects, &self.cut), &mut self.screens, &mut self.drag);
        self.meter.msgs += n;
        let mut changed = n > 0;
        self.d.keep(&mut self.st, Duration::from_millis(500));
        if let Some(p) = self.store.as_deref().filter(|_| self.st.store_writes != self.saved_writes) {
            match save_store(p, &self.st) {
                Ok(()) => self.saved_writes = self.st.store_writes,
                Err(e) => self.st.status = Some(format!("saving the store: {e:#}")),
            }
        }
        self.d.tick(&mut self.st);
        if let Some(p) = &self.file {
            let now = mtime(p);
            if now != self.seen {
                self.seen = now;
                match load(p) {
                    Ok(ui) => self.d.apply(&mut self.st, Msg::Push { ui }),
                    Err(e) => self.st.status = Some(format!("{e:#}")),
                }
                changed = true;
            }
        }
        // The themes are reloaded the same way, replacing those of the same names; the one
        // shown stays chosen, so a theme being tuned redraws in place.
        if let Some((p, seen)) = &mut self.themes_file {
            let now = mtime(p);
            if now != *seen {
                *seen = now;
                match load_themes(p) {
                    Ok(add) => self.d.apply(&mut self.st, Msg::Theme { add, choose: None }),
                    Err(e) => self.st.status = Some(format!("{e:#}")),
                }
                changed = true;
            }
        }
        // Animations (a pulsing ring) need time to move: a tick every frame while one shows, and
        // one when the last frame said it would change (a toast going, a spinner turning).
        let now = self.now();
        if (self.animating() && self.last_tick.elapsed() >= FRAME) || self.wake.is_some_and(|w| now >= w) {
            self.wake = None;
            self.last_tick = Instant::now();
            self.d.apply(&mut self.st, Msg::Tick { now_ms: now });
            changed = true;
        }
        if let Some(stats) = self.meter.report(self.st.stats.frames) {
            self.d.apply(&mut self.st, Msg::Stats { stats });
            changed = true;
        }
        changed
    }

    fn now(&self) -> u64 {
        self.d.started.elapsed().as_millis() as u64
    }

    fn animating(&self) -> bool {
        self.st.ui.layers.iter().any(|l| l.pulse.is_some())
    }

    /// How long the engine can wait for the next event before a chore is due.
    fn idle(&self) -> Duration {
        let mut wait = if self.animating() { FRAME.saturating_sub(self.last_tick.elapsed()) } else { Duration::from_millis(50) };
        if let Some(w) = self.wake {
            wait = wait.min(Duration::from_millis(w.saturating_sub(self.now())));
        }
        wait
    }

    /// Run the engine on its own thread: it waits for events, takes them in as fast as they
    /// arrive and calls `ring` whenever the screen may have changed, so whatever draws it draws
    /// then (at most once a frame, however often it rings). The thread ends when the screen quits
    /// (after a last ring) or `ring` says to stop by returning false.
    pub fn spawn(mut self, ring: impl Fn() -> bool + Send + 'static) -> Arc<Mutex<Live>> {
        let rx = self.rx.take().expect("a screen's engine starts once");
        let live = Arc::new(Mutex::new(self));
        let shared = live.clone();
        std::thread::spawn(move || {
            loop {
                let wait = shared.lock().map_or(Duration::from_millis(50), |l| l.idle());
                let mut batch = Vec::new();
                match rx.recv_timeout(wait) {
                    Ok(ev) => {
                        batch.push(ev);
                        batch.extend(rx.try_iter().take(10_000));
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
                let Ok(mut l) = shared.lock() else { return };
                let changed = l.step(batch);
                let quit = l.d.quit;
                drop(l);
                if (changed || quit) && !ring() {
                    return;
                }
                if quit {
                    return;
                }
            }
        });
        live
    }

    /// A message now (an input from a window, which routes its own clicks), through the engine
    /// like any other.
    pub fn msg(&mut self, msg: Msg) {
        self.d.apply(&mut self.st, msg);
    }

    /// An input now, landing on the last frame.
    pub fn input(&mut self, input: Input) {
        if let Some(m) = input_msg(input, &self.hits, &self.st, &mut self.drag) {
            self.d.apply(&mut self.st, m);
        }
    }

    pub fn state(&self) -> &State {
        &self.st
    }

    /// The shown nodes, with the last frame's rectangles.
    pub fn readback(&self) -> Vec<crate::readback::Shown> {
        crate::readback::readback(&self.st, &self.rects, &self.cut)
    }

    /// The frame drew, taking `took`: what it drew goes back to the engine (editors learn their
    /// size, the mouse lands on what's there). True when the state changed because of it, so
    /// another frame should follow.
    pub fn drew(&mut self, took: Duration, drawn: Drawn) -> bool {
        self.meter.frame(took);
        let Drawn { hits, rects, sizes, areas, wake, cut, fonts } = drawn;
        (self.hits, self.rects, self.wake, self.cut) = (hits, rects, wake, cut);
        resize_editors(&mut self.d, &mut self.st, sizes) | report_areas(&mut self.d, &mut self.st, areas) | report_fonts(&mut self.d, &mut self.st, fonts)
    }

    /// `screen` requests waiting for a frame; answer each with what it shows.
    pub fn take_screens(&mut self) -> Vec<Sender<Value>> {
        std::mem::take(&mut self.screens)
    }

    /// What the screen put on the clipboard since the last call, oldest first.
    pub fn take_copied(&mut self) -> Vec<String> {
        std::mem::take(&mut self.d.copied)
    }

    /// Whether the screen has quit.
    pub fn quit(&self) -> bool {
        self.d.quit
    }

    /// The binary an `upgrade` request asked the screen to become, once.
    pub fn take_upgrade(&mut self) -> Option<String> {
        self.d.upgrade.take()
    }

    /// Become `bin`, carrying on from this state: it's saved, the streams stop, the socket goes,
    /// and the same command line runs as `bin`, told where the state is (`$FICTTY_RESTORE`).
    /// Returns only if that failed, with the screen serving again.
    pub fn upgrade(&mut self, bin: &str) -> Result<()> {
        let path = std::env::temp_dir().join(format!("fictty-state-{}.json", std::process::id()));
        std::fs::write(&path, serde_json::to_vec(&self.st)?)?;
        self.d.stop_streams();
        unbind(&self.socket, self.bound);
        use std::os::unix::process::CommandExt;
        let mut cmd = Command::new(bin);
        cmd.args(std::env::args_os().skip(1)).env("FICTTY_RESTORE", &path);
        // Only returns if the exec failed.
        let err = cmd.exec();
        self.st.status = Some(format!("upgrade to {bin} failed: {err}"));
        self.bound = serve(&self.socket, self.d.tx.clone())?;
        Ok(())
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        self.d.stop_streams();
        unbind(&self.socket, self.bound);
    }
}

/// A socket file's device and inode.
fn identity(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(path).ok().map(|m| (m.dev(), m.ino()))
}

/// Remove the socket file at `path` if it's still the one this screen made: a screen that
/// started on the same path while this one was ending keeps its own.
fn unbind(path: &Path, bound: (u64, u64)) {
    if identity(path) == Some(bound) {
        let _ = std::fs::remove_file(path);
    }
}

/// Apply a batch of queued events. A source's value replaces the last one, so of several queued
/// values for one source only the newest is applied (the rest still count as received). Returns
/// how many events arrived.
fn apply_batch(batch: Vec<Event>, d: &mut Driver, st: &mut State, (hits, rects, cut): (&[Hit], &[(String, Rect)], &[String]), screens: &mut Vec<Sender<Value>>, drag: &mut Option<(String, Rect)>) -> usize {
    let n = batch.len();
    let mut newest: BTreeMap<String, usize> = BTreeMap::new();
    for (i, ev) in batch.iter().enumerate() {
        if let Event::Msg(Msg::Data { name, .. }) = ev {
            newest.insert(name.clone(), i);
        }
    }
    for (i, ev) in batch.into_iter().enumerate() {
        match ev {
            Event::Msg(Msg::Data { name, .. }) if newest.get(&name) != Some(&i) => {
                d.in_flight.remove(&name);
            }
            Event::Msg(m) => d.apply(st, m),
            Event::Request(Request::Upgrade { binary }, reply) => {
                let bin = binary.or_else(|| std::env::current_exe().ok().map(|p| p.display().to_string())).unwrap_or_default();
                let _ = reply.send(json!({"ok": true, "upgrading": bin}));
                d.upgrade = Some(bin);
            }
            Event::Request(Request::Screen, reply) => screens.push(reply),
            Event::Request(Request::Readback, reply) => {
                let _ = reply.send(json!({"ok": true, "nodes": crate::readback::readback(st, rects, cut)}));
            }
            Event::Request(Request::Mouse { kind, x, y }, reply) => {
                let msg = pointer(Pointer::requested(&kind), x, y, hits, st, &mut None);
                let what = msg.as_ref().map(|m| serde_json::to_value(m).unwrap_or_default());
                if let Some(m) = msg {
                    d.apply(st, m);
                }
                let _ = reply.send(json!({"ok": true, "msg": what}));
            }
            Event::Request(req, reply) => {
                let _ = reply.send(d.request(st, req));
            }
            Event::Input(input) => {
                if let Some(m) = input_msg(input, hits, st, drag) {
                    d.apply(st, m);
                }
            }
        }
    }
    n
}
/// Counts frames and messages over half-second windows.
struct Meter {
    start: Instant,
    frames: u64,
    draw: Duration,
    max: Duration,
    msgs: usize,
}

impl Meter {
    fn new() -> Meter {
        Meter { start: Instant::now(), frames: 0, draw: Duration::ZERO, max: Duration::ZERO, msgs: 0 }
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
        *self = Meter::new();
        Some(stats)
    }
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Listen for request lines; each gets one reply line.
/// Listen on `path`, answering each request line through the engine's queue, and say which file
/// that made (its device and inode).
fn serve(path: &Path, tx: Sender<Event>) -> Result<(u64, u64)> {
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
    identity(path).with_context(|| format!("{} went as it was made", path.display()))
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

/// A screen without a thread or a socket (`App::headless`): send it keys and messages, read its
/// frame and its state. Everything the host sends arrives before the next read.
pub struct Headless {
    st: State,
    d: Driver,
    /// What draws it, if anything: the terminal's grid.
    frame: Option<Box<dyn Frame>>,
    size: (u16, u16),
    notes: Receiver<Msg>,
    /// Whether the last frame is the state's as it is now.
    drawn: bool,
    /// The last frame's hit regions and node rectangles, so the mouse lands where a running
    /// screen's would.
    hits: Vec<Hit>,
    rects: Vec<(String, Rect)>,
    /// The nodes the last frame cut text in.
    cut: Vec<String>,
    /// An editor a press started a drag in, as the engine keeps it.
    drag: Option<(String, Rect)>,
    /// What the app does for the fuzz driver's host acts (`fuzz::Act::Host`): an agent's write,
    /// a file changing under a source.
    #[cfg(feature = "fuzz")]
    #[allow(clippy::type_complexity)]
    host_act: Option<Box<dyn FnMut(&mut Headless, &str)>>,
}

impl Headless {
    /// Keys, by name (`j`, `tab`, `enter`, `c-s`), as if typed.
    pub fn keys(&mut self, keys: &[&str]) -> &mut Headless {
        for k in keys {
            self.drain();
            if !self.routed(Input::Key((*k).into())) {
                self.msg(Msg::Key { key: (*k).into() });
            }
            // A frame after each, as a running screen draws one: a node a key brought on screen
            // learns its size before the next key types into it.
            self.frame();
        }
        self
    }

    /// Any message, as if it came from the socket.
    pub fn msg(&mut self, msg: Msg) -> &mut Headless {
        self.drain();
        self.drawn = false;
        let copies = self.d.copied.len();
        self.d.apply(&mut self.st, msg);
        self.d.keep(&mut self.st, Duration::ZERO);
        // What it copied goes on the frame's clipboard, as a running window puts it.
        if let Some(f) = &mut self.frame {
            for text in &self.d.copied[copies..] {
                f.copied(text);
            }
        }
        self
    }

    /// The mouse at a cell, as the terminal reports it: `press` (the left button), `s-press`,
    /// `drag` (with the button held), `release`, `move`, `wheel-up` or `wheel-down`. It lands on
    /// the last frame, as on a running screen.
    pub fn mouse(&mut self, kind: &str, x: u16, y: u16) -> &mut Headless {
        self.drain();
        let kind = Pointer::named(kind);
        // A window routes the mouse itself; a grid leaves it to the last frame's hit regions.
        if !self.routed(Input::Pointer { kind, x, y })
            && let Some(m) = pointer(kind, x, y, &self.hits, &self.st, &mut self.drag)
        {
            self.msg(m);
        }
        self.frame();
        self
    }

    /// Input through the frame, if it takes it (a window): what it sent, applied. False when
    /// the frame leaves it to the engine.
    fn routed(&mut self, input: Input) -> bool {
        if !self.frame.as_mut().is_some_and(|f| f.input(&input)) {
            return false;
        }
        self.absorb();
        true
    }

    /// The middle cell of what the last frame drew for node `id`, or one of its rows or tabs:
    /// where a test points the mouse to press it.
    pub fn at(&mut self, id: &str, row: Option<usize>, tab: Option<usize>) -> Option<(u16, u16)> {
        self.frame();
        let h = self.hits.iter().rev().find(|h| h.id == id && h.row == row && h.tab == tab && (row.is_some() || tab.is_some() || !h.text))?;
        Some((h.rect.x + h.rect.width.saturating_sub(1) / 2, h.rect.y + h.rect.height.saturating_sub(1) / 2))
    }

    /// Where the last frame drew editor `id`'s text: a test presses at its cells.
    pub fn text_at(&mut self, id: &str) -> Option<Rect> {
        self.frame();
        self.hits.iter().rev().find(|h| h.id == id && h.text).map(|h| h.rect)
    }

    /// What the frame sent (a window's clicks), applied in order.
    fn absorb(&mut self) -> bool {
        let sent = self.frame.as_mut().map(|f| f.messages()).unwrap_or_default();
        let any = !sent.is_empty();
        for m in sent {
            self.msg(m);
        }
        any
    }

    /// Text pasted (bracketed), as the terminal delivers it.
    pub fn paste(&mut self, text: &str) -> &mut Headless {
        self.drain();
        if !self.routed(Input::Paste(text.into())) {
            self.msg(Msg::Paste { text: text.into() });
        }
        self.frame();
        self
    }

    /// The screen resized to `width` × `height` cells.
    pub fn resize(&mut self, width: u16, height: u16) -> &mut Headless {
        self.drawn = false;
        self.size = (width, height);
        if let Some(f) = &mut self.frame {
            f.resize(width, height);
        }
        self.msg(Msg::Screen { width, height });
        self.frame();
        self
    }

    /// The frame as a terminal shows it: drawn the way a running screen draws (diffed against
    /// the frame before) into an emulator, read back as text. It should always equal `screen`;
    /// where it doesn't, the escapes are wrong. Frames from the first call on go through it.
    #[cfg(feature = "fuzz")]
    pub fn terminal(&mut self) -> String {
        if self.frame.as_mut().is_some_and(|f| f.wire()) {
            self.drawn = false;
        }
        self.frame();
        match self.frame.as_ref().and_then(|f| f.terminal()) {
            Some(t) => t,
            None => self.screen(),
        }
    }

    /// Whether the screen has quit (a `quit` action, or Ctrl-C outside a text field): a running
    /// one would have ended here.
    pub fn quit(&self) -> bool {
        self.d.quit
    }

    /// The screen's size in cells.
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    /// A request as the control socket takes it (`{"op": "write", "key": …}`), answered as the
    /// socket would answer it: an agent's view of the screen, without a socket.
    pub fn request(&mut self, req: Value) -> Value {
        self.drain();
        self.drawn = false;
        let reply = match serde_json::from_value::<Request>(req) {
            // A running screen answers these from its last frame.
            Ok(Request::Screen) => return json!({"ok": true, "screen": self.screen()}),
            Ok(Request::Readback) => return json!({"ok": true, "nodes": self.readback()}),
            Ok(Request::Mouse { kind, x, y }) => {
                let kind = match kind.as_str() {
                    "click" => "press",
                    "up" => "wheel-up",
                    "down" => "wheel-down",
                    _ => "move",
                };
                self.mouse(kind, x, y);
                return json!({"ok": true});
            }
            Ok(r) => self.d.request(&mut self.st, r),
            Err(e) => json!({"ok": false, "status": format!("bad request: {e}")}),
        };
        self.d.keep(&mut self.st, Duration::ZERO);
        reply
    }

    /// What a host act (`fuzz::Act::Host`) does: the app's handler, given the act's argument.
    #[cfg(feature = "fuzz")]
    pub fn on_host_act(mut self, f: impl FnMut(&mut Headless, &str) + 'static) -> Headless {
        self.host_act = Some(Box::new(f));
        self
    }

    /// Run a host act through the handler `on_host_act` gave.
    #[cfg(feature = "fuzz")]
    pub fn host_act(&mut self, arg: &str) -> &mut Headless {
        let mut f = self.host_act.take().expect("a host act, but no handler: Headless::on_host_act");
        f(self, arg);
        self.host_act = Some(f);
        self
    }

    /// What it has put on the clipboard so far, oldest first.
    pub fn copied(&self) -> &[String] {
        &self.d.copied
    }

    /// The state, as `get --state` shows it.
    pub fn state(&mut self) -> &State {
        self.drain();
        &self.st
    }

    /// The frame as text, one line per row (trailing spaces trimmed); without a frame to draw
    /// it, the readback as an outline.
    pub fn screen(&mut self) -> String {
        self.frame();
        match &self.frame {
            Some(f) => f.text(),
            None => crate::readback::outline(&self.readback()),
        }
    }

    /// The frame with its colours and attributes, as ANSI (truecolour) escapes.
    pub fn ansi(&mut self) -> String {
        self.frame();
        match &self.frame {
            Some(f) => f.ansi(),
            None => self.screen(),
        }
    }

    /// The shown nodes, in order, with where the last frame laid each out.
    pub fn readback(&mut self) -> Vec<crate::readback::Shown> {
        self.frame();
        crate::readback::readback(&self.st, &self.rects, &self.cut)
    }

    fn drain(&mut self) {
        while let Ok(m) = self.notes.try_recv() {
            self.drawn = false;
            self.d.apply(&mut self.st, m);
            self.d.keep(&mut self.st, Duration::ZERO);
        }
    }

    fn frame(&mut self) {
        self.drain();
        // The frame is the state's: nothing new since the last one, nothing to draw.
        if self.drawn {
            return;
        }
        self.drawn = true;
        let Some(f) = &mut self.frame else { return };
        let Drawn { hits, rects, sizes, areas, cut, fonts, .. } = f.draw(&self.st);
        (self.hits, self.rects, self.cut) = (hits, rects, cut);
        // What drawing sent (a window's cell size) and what it learned (an editor's size) change
        // the state: again until they don't, so the frame is the state's. A `fit` editor takes
        // two: its width, then the rows its text wraps to at that width.
        let mut changed = resize_editors(&mut self.d, &mut self.st, sizes) | report_areas(&mut self.d, &mut self.st, areas) | report_fonts(&mut self.d, &mut self.st, fonts) | self.absorb();
        for _ in 0..4 {
            let Some(f) = self.frame.as_mut().filter(|_| changed) else { break };
            let Drawn { hits, rects, sizes, areas, cut, fonts, .. } = f.draw(&self.st);
            (self.hits, self.rects, self.cut) = (hits, rects, cut);
            changed = resize_editors(&mut self.d, &mut self.st, sizes) | report_areas(&mut self.d, &mut self.st, areas) | report_fonts(&mut self.d, &mut self.st, fonts) | self.absorb();
        }
        self.drawn = true;
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
                Write::Delete {} => ("delete", format!("{key}{rev}")),
            }
        }
        Request::Set { id, selected, tab, key } => {
            let what = match (selected, tab, key) {
                (Some(r), _, _) => format!("{id} row {r}"),
                (_, Some(t), _) => format!("{id} tab {t}"),
                (_, _, Some(k)) => format!("{id} row {k}"),
                _ => id.clone(),
            };
            ("set", what)
        }
        Request::Key { key } => ("key", key.clone()),
        Request::Quit => ("quit", String::new()),
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

/// Tells each editor the size it was drawn at, and each list how many rows it showed, where that
/// differs from what they have. True when any changed (the next frame should draw).
fn resize_editors(d: &mut Driver, st: &mut State, sizes: Vec<(String, u16, u16)>) -> bool {
    let mut changed = false;
    for (id, width, height) in sizes {
        let own = crate::state::field_view(st, &id).map(|(_, v)| (v.viewport.width, v.viewport.height));
        // An editor's size, or the rows a list showed (for the wheel) and the width it laid
        // them out across (for `cols` that fit what there's room for).
        let differs = match own {
            Some(o) => o != (width.max(1), height.max(1)),
            None => st.local.get(&id).map(|l| (l.rows, l.width)) != Some((height, width)) && st.node(&id).is_some_and(|n| n.field().is_none()),
        };
        if differs {
            d.apply(st, Msg::Size { id, width, height });
            changed = true;
        }
    }
    changed
}

/// Tells the state the area each sized node was laid out in, where it differs from what it has.
/// True when any changed.
fn report_areas(d: &mut Driver, st: &mut State, areas: Vec<(String, u16, u16)>) -> bool {
    let mut changed = false;
    for (id, width, height) in areas {
        if st.local.get(&id).and_then(|l| l.area) != Some((width, height)) {
            d.apply(st, Msg::Area { id, width, height });
            changed = true;
        }
    }
    changed
}

/// Font files the frame couldn't load, to the state (and its problems), once each.
fn report_fonts(d: &mut Driver, st: &mut State, fonts: Vec<(String, String)>) -> bool {
    let mut changed = false;
    for (file, error) in fonts {
        if st.fonts_failed.get(&file) != Some(&error) {
            d.apply(st, Msg::Font { file, error });
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn one_screen_holds_a_store_however_its_path_is_written_and_nothing_goes_beside_it() {
        let dir = std::env::temp_dir().join(format!("fictty-lock-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = dir.join("store.json");
        let held = hold_store(&store).unwrap();
        // The same store by another spelling of its path is the same lock.
        let again = dir.join(".").join("store.json");
        let err = hold_store(&again).unwrap_err().to_string();
        assert!(err.contains(&format!("pid {}", std::process::id())), "{err}");
        assert!(hold_store(&dir.join("other.json")).is_ok(), "another store has its own");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "no lock file beside the store");
        drop(held);
        assert!(hold_store(&store).is_ok(), "free once the screen holding it lets go");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A host that keeps one page and merges what's saved into it, as a vault does: it notes
    /// every save and load.
    #[derive(Default)]
    struct Merging {
        page: Mutex<String>,
        calls: Mutex<Vec<String>>,
    }

    impl Host for Merging {
        fn keys(&self) -> Vec<String> {
            vec!["page".into()]
        }
        fn load(&self, _: &str) -> std::result::Result<Value, String> {
            self.calls.lock().unwrap().push("load".into());
            Ok(Value::String(self.page.lock().unwrap().clone()))
        }
        fn save(&self, _: &str, value: &Value) -> std::result::Result<(), String> {
            self.calls.lock().unwrap().push(format!("save {value}"));
            // The agent's line stays; the person's typing joins it.
            let mut page = self.page.lock().unwrap();
            let typed = value.as_str().unwrap().trim_start_matches("base").to_string();
            *page = format!("base{typed}{}", page.trim_start_matches("base"));
            Ok(())
        }
    }

    #[test]
    fn a_reload_saves_unsaved_typing_first() {
        // Thought Control's pty run: an agent's write landed while the person's typing wasn't
        // saved yet. Loading over it would lose the typing; saving first lets the host merge.
        let host = Arc::new(Merging::default());
        *host.page.lock().unwrap() = "base".into();
        let (tx, _rx) = mpsc::channel();
        let mut d = Driver::new(tx, true, host.clone());
        let mut st = State { hosted: host.keys(), ..State::default() };
        let ui: Ui = serde_json::from_value(json!({"root": {"type": "editor", "id": "e", "doc": "page", "caret": "end"}, "focus": "e"})).unwrap();
        d.apply(&mut st, Msg::Push { ui });
        d.apply(&mut st, Msg::Key { key: "!".into() });
        // The agent writes; the host says so before the screen has saved.
        *host.page.lock().unwrap() = "base\nagent".into();
        d.apply(&mut st, Msg::Reload { key: "page".into() });
        assert_eq!(*host.calls.lock().unwrap(), vec!["load", "save \"base!\"", "load"]);
        assert_eq!(st.store["page"].read()["text"], "base!\nagent");
        // Nothing left to save.
        d.keep(&mut st, Duration::ZERO);
        assert_eq!(host.calls.lock().unwrap().len(), 3);
    }
}
