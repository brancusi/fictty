//! The terminal: a screen drawn as a grid of cells. It starts the core's engine on its own thread,
//! turns the terminal's input into the engine's, and draws the state whenever the engine rings.
//! `Grid` draws the same frames for headless screens, so tests read the grid.

pub use crate::engine::{Inbox, Input, Live, Pointer, Request, default_socket, fetch, load, load_themes, send};
use crate::engine::{Drawn, Frame};
use crate::host::Host;
use crate::model::{ThemeSpec, Ui};
use crate::state::{Msg, State};
use crate::view;
use anyhow::Result;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event as TermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton,
    MouseEventKind,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(feature = "fuzz")]
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A screen to run: its UI (a value, or a file), and what it starts from. The `fictty` binary is
/// one user of this; an app that embeds fictty is another, with a `Host` for its own data.
///
/// ```no_run
/// # let ui: fictty::model::Ui = serde_json::from_str(r#"{"root": {"type": "text", "text": "hi"}}"#).unwrap();
/// fictty::App::new(ui).socket("/tmp/my.sock").run().unwrap();
/// ```
pub struct App(crate::engine::App);

impl App {
    /// A screen showing `ui`.
    pub fn new(ui: Ui) -> App {
        App(crate::engine::App::new(ui))
    }

    /// A screen showing the UI in `path` (`-` reads stdin), read when it starts.
    pub fn file(path: impl Into<PathBuf>) -> App {
        App(crate::engine::App::file(path))
    }

    /// A screen waiting for a UI to be pushed.
    pub fn empty() -> App {
        App(crate::engine::App::empty())
    }

    /// Whether `run` actions and command sources run (they do by default). Off, each fails with
    /// "commands are off" and nothing is started: for tests and fuzzing a UI whose buttons run
    /// scripts.
    pub fn commands(self, on: bool) -> App {
        App(self.0.commands(on))
    }

    /// Reload the UI file whenever it changes.
    pub fn watch(self, on: bool) -> App {
        App(self.0.watch(on))
    }

    /// The control socket agents reach the screen on (default: `$FICTTY_SOCKET`, else
    /// `fictty.sock` in the temp dir).
    pub fn socket(self, path: impl Into<PathBuf>) -> App {
        App(self.0.socket(path))
    }

    /// Start from a saved state (what `upgrade` hands the new process).
    pub fn restore(self, path: Option<PathBuf>) -> App {
        App(self.0.restore(path))
    }

    /// Keep the store in this JSON file: read at the start, written after every change.
    pub fn store(self, path: impl Into<PathBuf>) -> App {
        App(self.0.store(path))
    }

    /// Add themes (by name). The first is chosen unless `theme` names one.
    pub fn themes(self, themes: BTreeMap<String, ThemeSpec>) -> App {
        App(self.0.themes(themes))
    }

    /// Add the themes in a JSON file (theme specs by name).
    pub fn themes_file(self, path: &Path) -> Result<App> {
        Ok(App(self.0.themes_file(path)?))
    }

    /// The theme to start with.
    pub fn theme(self, name: impl Into<String>) -> App {
        App(self.0.theme(name))
    }

    /// The app embedding the screen: its sources and actions.
    pub fn host(self, host: impl Host) -> App {
        App(self.0.host(host))
    }

    /// The core's screen, to start an engine for something else to draw.
    pub fn into_core(self) -> crate::engine::App {
        self.0
    }

    /// Without a terminal: a screen of `width`×`height` cells that renders to text, for tests
    /// and `fictty render`. Sources are fetched and host actions done inline.
    pub fn headless(self, width: u16, height: u16) -> Result<crate::engine::Headless> {
        self.0.headless_with(width, height, Box::new(Grid::new(width, height)?))
    }

    /// Run in this terminal until `quit`.
    pub fn run(self) -> Result<()> {
        let (width, height) = ratatui::crossterm::terminal::size().unwrap_or((80, 24));
        let mut live = self.0.live(width, height)?;
        let inbox = live.inbox();
        let caps = |sizing: bool| Msg::Caps { caps: crate::gfx::detect(), text_sizing: sizing };
        let mut kitty = crate::gfx::Kitty::default();

        generic_tty()?;
        let mut term = ratatui::init();
        // Larger text: the terminal's to say (a probe), unless FICTTY_TEXT_SIZE forces it.
        let sizing = match std::env::var("FICTTY_TEXT_SIZE").as_deref() {
            Ok("1") => true,
            Ok("0") => false,
            _ => crate::gfx::probe_text_sizing(),
        };
        // (The probe clears what it printed; ratatui's first frame draws every cell.)
        live.msg(caps(sizing));
        if let Ok(size) = term.size() {
            live.msg(Msg::Screen { width: size.width, height: size.height });
        }
        let _ = ratatui::crossterm::execute!(std::io::stdout(), EnableMouseCapture, EnableBracketedPaste);
        // Say a fictty screen runs here (a terminal user var, OSC 1337), so a terminal set up for
        // it (`fictty setup wezterm`) sends ⌘ keys here and nowhere else. Others ignore it.
        let _ = write!(std::io::stdout(), "\x1b]1337;SetUserVar=fictty=MQ==\x07");
        // Keys with every modifier told apart (⌘ as `d-`, Ctrl-Home, Shift-Enter), where the
        // terminal speaks the kitty keyboard protocol.
        let enhanced = ratatui::crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
        if enhanced {
            use ratatui::crossterm::event::{KeyboardEnhancementFlags as F, PushKeyboardEnhancementFlags};
            let _ = ratatui::crossterm::execute!(std::io::stdout(), PushKeyboardEnhancementFlags(F::DISAMBIGUATE_ESCAPE_CODES | F::REPORT_ALTERNATE_KEYS));
        }
        // The engine runs on its own thread and rings when the screen may have changed.
        let (ring, rung) = mpsc::channel::<()>();
        let live = live.spawn(move || ring.send(()).is_ok());
        // Terminal input joins the engine's queue, in order, with everything else.
        std::thread::spawn(move || {
            while let Ok(ev) = event::read() {
                let sent = match ev {
                    TermEvent::Key(k) if k.kind == KeyEventKind::Press => key_name(k).is_none_or(|key| inbox.input(Input::Key(key))),
                    TermEvent::Mouse(m) => pointer(m.kind, m.modifiers).is_none_or(|kind| inbox.input(Input::Pointer { kind, x: m.column, y: m.row })),
                    TermEvent::Paste(text) => inbox.input(Input::Paste(text)),
                    // The cell size can change with the window (a font size change, another display).
                    TermEvent::Resize(width, height) => inbox.input(Input::Resize { width, height }) && inbox.msg(caps(sizing)),
                    _ => true,
                };
                if !sent {
                    return;
                }
            }
        });
        let res = (|| -> Result<()> {
            // Draw at most once per frame budget (120 fps), whenever the engine rang.
            let budget = Duration::from_micros(8_333);
            let mut last_draw = Instant::now() - budget;
            let mut pending = true;
            let mut scaled_last: Vec<view::Scaled> = Vec::new();
            let mut size = term.size()?;
            loop {
                if !pending {
                    match rung.recv_timeout(Duration::from_millis(250)) {
                        Ok(()) => {}
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
                    }
                }
                while rung.try_recv().is_ok() {}
                // Wait out the frame budget on the queue: rings in the meantime join this frame.
                // (A timed wait on the queue wakes on time; a sleep here oversleeps by
                // milliseconds.)
                while let Some(left) = budget.checked_sub(last_draw.elapsed()).filter(|d| !d.is_zero()) {
                    if let Err(mpsc::RecvTimeoutError::Disconnected) = rung.recv_timeout(left) {
                        return Ok(());
                    }
                }
                let mut l = live.lock().map_err(|_| anyhow::anyhow!("the engine stopped"))?;
                if l.quit() {
                    return Ok(());
                }
                let copied = l.take_copied();
                if !copied.is_empty() {
                    use base64::Engine;
                    let mut out = std::io::stdout().lock();
                    for text in copied {
                        // The terminal puts it on the clipboard (OSC 52).
                        let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                        let _ = write!(out, "\x1b]52;c;{b64}\x07");
                    }
                    let _ = out.flush();
                }
                if let Some(bin) = l.take_upgrade() {
                    let mut out = std::io::stdout().lock();
                    out.write_all(&kitty.clear())?;
                    out.flush()?;
                    drop(out);
                    // Only returns if the exec failed; the screen carries on.
                    l.upgrade(&bin)?;
                }
                let screens = l.take_screens();
                let t = Instant::now();
                let mut text = None;
                let mut out = None;
                term.draw(|f| {
                    out = Some(view::draw(l.state(), f));
                    if !screens.is_empty() {
                        text = Some(buffer_text(f.buffer_mut()));
                    }
                })?;
                let out = out.expect("a frame");
                let now = term.size()?;
                if now != size {
                    size = now;
                    scaled_last.clear();
                }
                // Pictures and larger text go out after the cells, in one write.
                let mut extra = kitty.sync(&out.pics, l.state().caps);
                if out.scaled != scaled_last {
                    for t in &out.scaled {
                        extra.extend(osc66(t));
                    }
                    scaled_last = out.scaled;
                }
                if !extra.is_empty() {
                    let mut o = std::io::stdout().lock();
                    o.write_all(&extra)?;
                    o.flush()?;
                }
                if let Some(text) = text {
                    for s in screens {
                        let _ = s.send(json!({"ok": true, "screen": text}));
                    }
                }
                let drawn = Drawn { hits: out.hits, rects: out.rects, sizes: out.sizes, areas: out.areas, wake: out.wake };
                pending = l.drew(t.elapsed(), drawn);
                last_draw = Instant::now();
            }
        })();
        let _ = std::io::stdout().write_all(&kitty.clear());
        if enhanced {
            let _ = ratatui::crossterm::execute!(std::io::stdout(), ratatui::crossterm::event::PopKeyboardEnhancementFlags);
        }
        let _ = ratatui::crossterm::execute!(std::io::stdout(), DisableMouseCapture, DisableBracketedPaste);
        let _ = write!(std::io::stdout(), "\x1b]1337;SetUserVar=fictty=\x07");
        ratatui::restore();
        // The engine's thread ends with the queue; the socket goes with the screen.
        drop(live);
        res
    }
}

/// A terminal's mouse event as the engine's pointer.
fn pointer(kind: MouseEventKind, mods: KeyModifiers) -> Option<Pointer> {
    Some(match kind {
        MouseEventKind::Down(MouseButton::Left) => Pointer::Press { shift: mods.contains(KeyModifiers::SHIFT) },
        MouseEventKind::Drag(MouseButton::Left) => Pointer::Drag,
        MouseEventKind::Up(MouseButton::Left) => Pointer::Release,
        MouseEventKind::ScrollUp => Pointer::WheelUp,
        MouseEventKind::ScrollDown => Pointer::WheelDown,
        MouseEventKind::Moved | MouseEventKind::Drag(_) => Pointer::Move,
        _ => return None,
    })
}

/// Frames drawn into a grid of cells in memory, for headless screens: what a terminal would show.
pub struct Grid {
    term: ratatui::Terminal<ratatui::backend::TestBackend>,
    /// The last frame as drawn. Not the backend's buffer: that one only takes the cells a frame
    /// changed, and keeps a stale character under a wide one's second cell, which no terminal
    /// shows.
    last: ratatui::buffer::Buffer,
    /// Once asked for (`wire`): every frame drawn again the way a running screen draws it, as
    /// escapes from the crossterm backend, diffed against the frame before, into an emulator.
    #[cfg(feature = "fuzz")]
    wire: Option<Wire>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Result<Grid> {
        Ok(Grid {
            term: ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))?,
            last: ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, width, height)),
            #[cfg(feature = "fuzz")]
            wire: None,
        })
    }
}

impl Frame for Grid {
    fn draw(&mut self, st: &State) -> Drawn {
        let mut drawn = Drawn::default();
        let last = &mut self.last;
        let _ = self.term.draw(|f| {
            let out = view::draw(st, f);
            drawn = Drawn { hits: out.hits, rects: out.rects, sizes: out.sizes, areas: out.areas, wake: out.wake };
            *last = f.buffer_mut().clone();
        });
        #[cfg(feature = "fuzz")]
        if let Some(w) = &mut self.wire {
            w.draw(st);
        }
        drawn
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.term.backend_mut().resize(width, height);
        let _ = self.term.resize(ratatui::layout::Rect::new(0, 0, width, height));
        // The terminal resized: the emulator's grid too, and ratatui draws the whole frame again.
        #[cfg(feature = "fuzz")]
        if let Some(w) = &mut self.wire {
            w.vt.screen_mut().set_size(height, width);
            let _ = w.term.resize(ratatui::layout::Rect::new(0, 0, width, height));
        }
    }

    fn text(&self) -> String {
        buffer_text(&self.last)
    }

    fn ansi(&self) -> String {
        buffer_ansi(&self.last)
    }

    #[cfg(feature = "fuzz")]
    fn wire(&mut self) -> bool {
        if self.wire.is_some() {
            return false;
        }
        let a = self.term.backend().buffer().area;
        self.wire = Some(Wire::new(a.width, a.height));
        true
    }

    #[cfg(feature = "fuzz")]
    fn terminal(&self) -> Option<String> {
        self.wire.as_ref().map(Wire::text)
    }
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

/// A frame's way to a terminal: ratatui's crossterm backend writing escapes into a buffer, and
/// vt100 reading them.
#[cfg(feature = "fuzz")]
struct Wire {
    term: ratatui::Terminal<ratatui::backend::CrosstermBackend<Tap>>,
    out: Arc<std::sync::Mutex<Vec<u8>>>,
    vt: vt100::Parser,
}

/// What the backend writes, kept for the emulator to read.
#[cfg(feature = "fuzz")]
struct Tap(Arc<std::sync::Mutex<Vec<u8>>>);

#[cfg(feature = "fuzz")]
impl Write for Tap {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("the tap").extend_from_slice(b);
        Ok(b.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "fuzz")]
impl Wire {
    fn new(w: u16, h: u16) -> Wire {
        let out = Arc::new(std::sync::Mutex::new(Vec::new()));
        let backend = ratatui::backend::CrosstermBackend::new(Tap(out.clone()));
        let viewport = ratatui::Viewport::Fixed(ratatui::layout::Rect::new(0, 0, w, h));
        let term = ratatui::Terminal::with_options(backend, ratatui::TerminalOptions { viewport }).expect("a terminal on a buffer");
        Wire { term, out, vt: vt100::Parser::new(h, w, 0) }
    }

    fn draw(&mut self, st: &State) {
        let _ = self.term.draw(|f| {
            view::draw(st, f);
        });
        let bytes = std::mem::take(&mut *self.out.lock().expect("the tap"));
        self.vt.process(&bytes);
    }

    /// The emulator's grid as text, as `buffer_text` writes a frame.
    fn text(&self) -> String {
        let screen = self.vt.screen();
        let (h, w) = screen.size();
        let mut out = String::new();
        for y in 0..h {
            let mut line = String::new();
            for x in 0..w {
                match screen.cell(y, x) {
                    // `buffer_text` writes a wide character's second cell as a space.
                    Some(c) if c.has_contents() && !c.is_wide_continuation() => line.push_str(c.contents()),
                    _ => line.push(' '),
                }
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out
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

/// A key's name: the key (`a`, `A`, `enter`, `left`, `f5`), after its modifiers in the order `c-`
/// (Ctrl), `a-` (Alt/Option), `d-` (Super/⌘), `s-` (Shift). Shift on a letter alone is its case
/// (`A`); with another modifier it's `s-` and the letter is lowercase (`s-d-z`, ⇧⌘Z).
pub fn key_name(k: KeyEvent) -> Option<String> {
    let m = k.modifiers;
    let (ctrl, alt, sup, mut shift) = (m.contains(KeyModifiers::CONTROL), m.contains(KeyModifiers::ALT), m.contains(KeyModifiers::SUPER), m.contains(KeyModifiers::SHIFT));
    let base = match k.code {
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(c) if ctrl || alt || sup => {
            shift |= c.is_uppercase();
            c.to_lowercase().to_string()
        }
        KeyCode::Char(c) => {
            shift = false;
            c.to_string()
        }
        KeyCode::Tab => "tab".into(),
        KeyCode::BackTab => {
            shift = true;
            "tab".into()
        }
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
        KeyCode::F(n) => format!("f{n}"),
        _ => return None,
    };
    let mut name = String::new();
    for (on, p) in [(ctrl, "c-"), (alt, "a-"), (sup, "d-"), (shift, "s-")] {
        if on {
            name.push_str(p);
        }
    }
    name.push_str(&base);
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_put_modifiers_in_one_order() {
        let k = |code, mods| key_name(KeyEvent::new(code, mods)).unwrap();
        assert_eq!(k(KeyCode::Char('a'), KeyModifiers::NONE), "a");
        assert_eq!(k(KeyCode::Char('A'), KeyModifiers::SHIFT), "A");
        assert_eq!(k(KeyCode::Char('c'), KeyModifiers::CONTROL), "c-c");
        assert_eq!(k(KeyCode::Char('b'), KeyModifiers::ALT), "a-b");
        assert_eq!(k(KeyCode::Char('a'), KeyModifiers::SUPER), "d-a");
        assert_eq!(k(KeyCode::Char('Z'), KeyModifiers::SUPER | KeyModifiers::SHIFT), "d-s-z");
        assert_eq!(k(KeyCode::Char('z'), KeyModifiers::SUPER | KeyModifiers::SHIFT), "d-s-z");
        assert_eq!(k(KeyCode::Left, KeyModifiers::SHIFT), "s-left");
        assert_eq!(k(KeyCode::Left, KeyModifiers::ALT | KeyModifiers::SHIFT), "a-s-left");
        assert_eq!(k(KeyCode::BackTab, KeyModifiers::SHIFT), "s-tab");
        assert_eq!(k(KeyCode::Home, KeyModifiers::CONTROL), "c-home");
        assert_eq!(k(KeyCode::Delete, KeyModifiers::SUPER), "d-del");
        assert_eq!(k(KeyCode::Char(' '), KeyModifiers::NONE), "space");
    }
}
