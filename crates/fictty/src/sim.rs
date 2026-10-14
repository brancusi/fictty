//! A screen in a pseudo-terminal, with a terminal emulator on the other end: the layer under
//! `Headless`. Keys go in as the bytes a terminal sends for them, encoded under the keyboard
//! flags the app pushed (the kitty protocol, or legacy escapes), so crossterm's own decoder reads
//! them. The mouse goes in as SGR reports, pastes bracketed when the app asked for it. What comes
//! out is read by vt100, and the app's socket says what it drew, so a test can check that the
//! terminal shows exactly the frame.
//!
//! It runs any binary that is a fictty screen (`fictty run ui.json`, or an app built on fictty),
//! with `FICTTY_SOCKET` set to its own socket. Each step waits until the screen has gone quiet
//! and has answered on its socket, so a step's effects are on screen when it returns.
//!
//! ```no_run
//! # use fictty::sim::Sim;
//! let mut s = Sim::spawn(&["fictty", "run", "ui.json"], 80, 24).unwrap();
//! s.keys(&["j", "d-a", "s-tab"]).mouse("press", 3, 4).paste("text");
//! s.check().unwrap(); // the terminal shows the frame
//! ```

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// What the app turned on, read from what it wrote.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Modes {
    /// The kitty keyboard flags pushed, innermost last.
    pub kitty: Vec<u32>,
    pub bracketed_paste: bool,
    pub sgr_mouse: bool,
}

/// A screen in a pty.
pub struct Sim {
    child: Child,
    master: std::fs::File,
    vt: vt100::Parser,
    socket: PathBuf,
    pub modes: Modes,
    /// Whether this terminal answers the kitty keyboard query (a terminal that speaks the
    /// protocol); off, the app falls back to legacy keys.
    kitty: bool,
    /// The end of what was read last, for an escape split across reads.
    tail: Vec<u8>,
    /// How long output must be still before a step counts as done.
    pub quiet: Duration,
    /// What the app does for a host act (`fuzz::Act::Host`).
    #[allow(clippy::type_complexity)]
    host_act: Option<Box<dyn FnMut(&mut Sim, &str)>>,
}

impl Sim {
    /// Runs `argv` in a `width` × `height` pty, as a terminal that speaks the kitty keyboard
    /// protocol, and waits until its screen is up.
    pub fn spawn(argv: &[&str], width: u16, height: u16) -> Result<Sim> {
        Sim::spawn_with(argv, width, height, true, &[])
    }

    /// `spawn`, saying whether the terminal speaks the kitty keyboard protocol, with more
    /// environment for the app.
    pub fn spawn_with(argv: &[&str], width: u16, height: u16, kitty: bool, env: &[(&str, &str)]) -> Result<Sim> {
        let (bin, args) = argv.split_first().context("an empty command")?;
        let (mut master, mut slave) = (0, 0);
        let mut ws = libc::winsize { ws_row: height, ws_col: width, ws_xpixel: width * 8, ws_ypixel: height * 16 };
        // SAFETY: openpty writes two descriptors it opened; the winsize outlives the call.
        if unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), &raw mut ws) } != 0 {
            bail!("openpty: {}", std::io::Error::last_os_error());
        }
        // SAFETY: both were just opened and are owned here alone.
        let (master, slave) = unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) };
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Short: a socket path has room for about a hundred bytes.
        let socket = std::env::temp_dir().join(format!("fsim-{}-{n}.sock", std::process::id()));
        let mut cmd = Command::new(bin);
        cmd.args(args)
            .env("FICTTY_SOCKET", &socket)
            .env("TERM", "xterm-256color")
            // No pictures and no larger text: vt100 draws neither.
            .env("FICTTY_GRAPHICS", "0")
            .env("FICTTY_TEXT_SIZE", "0")
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));
        for (k, v) in env {
            cmd.env(k, v);
        }
        // SAFETY: only async-signal-safe calls between fork and exec.
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                // Its own session, with the pty as its terminal (what /dev/tty opens).
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = cmd.spawn().with_context(|| format!("running {bin}"))?;
        let master = std::fs::File::from(master);
        // SAFETY: a valid descriptor; O_NONBLOCK changes only how reads wait.
        unsafe {
            let fd = master.as_raw_fd();
            libc::fcntl(fd, libc::F_SETFL, libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK);
        }
        let mut s = Sim { child, master, vt: vt100::Parser::new(height, width, 0), socket, modes: Modes::default(), kitty, tail: Vec::new(), quiet: Duration::from_millis(30), host_act: None };
        // Up once its socket answers.
        let start = Instant::now();
        loop {
            s.read(Duration::from_millis(20));
            if s.socket.exists() && s.ask(json!({"op": "stats"})).is_ok() {
                break;
            }
            if let Some(status) = s.child.try_wait()? {
                bail!("{bin} exited ({status}) before its screen was up:\n{}", s.text());
            }
            if start.elapsed() > Duration::from_secs(10) {
                bail!("{bin}'s screen wasn't up after 10s:\n{}", s.text());
            }
        }
        s.settle();
        Ok(s)
    }

    /// Reads what's there (waiting up to `wait` for the first bytes), feeding the emulator and
    /// answering the terminal queries the app asks. Whether anything came.
    fn read(&mut self, wait: Duration) -> bool {
        let mut fds = libc::pollfd { fd: self.master.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        // SAFETY: one valid pollfd.
        if unsafe { libc::poll(&mut fds, 1, wait.as_millis() as i32) } <= 0 {
            return false;
        }
        let mut buf = [0u8; 65536];
        let mut any = false;
        loop {
            match self.master.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    any = true;
                    trace("out", &buf[..n]);
                    self.vt.process(&buf[..n]);
                    self.scan(&buf[..n]);
                }
            }
        }
        any
    }

    /// Answers queries and notes modes in what the app wrote.
    fn scan(&mut self, bytes: &[u8]) {
        let mut seen = std::mem::take(&mut self.tail);
        seen.extend_from_slice(bytes);
        let mut replies: Vec<u8> = Vec::new();
        let mut i = 0;
        while let Some(at) = seen[i..].iter().position(|b| *b == 0x1b).map(|p| p + i) {
            if seen.get(at + 1).is_some_and(|b| *b != b'[') {
                i = at + 1;
                continue;
            }
            // A CSI's final byte; none yet (or not even its `[`): keep it for the next read.
            let Some(end) = seen[at..].iter().skip(2).position(|b| (0x40..=0x7e).contains(b)).map(|e| at + e + 3).filter(|_| at + 1 < seen.len()) else {
                if seen.len() - at < 64 {
                    self.tail = seen[at..].to_vec();
                }
                break;
            };
            let csi = &seen[at + 2..end];
            match csi {
                b"6n" => {
                    let (row, col) = self.vt.screen().cursor_position();
                    replies.extend(format!("\x1b[{};{}R", row + 1, col + 1).bytes());
                }
                b"?u" if self.kitty => {
                    let flags = self.modes.kitty.last().copied().unwrap_or(0);
                    replies.extend(format!("\x1b[?{flags}u").bytes());
                }
                b"c" | b"0c" => replies.extend(b"\x1b[?62;22c"),
                b"?2004h" => self.modes.bracketed_paste = true,
                b"?2004l" => self.modes.bracketed_paste = false,
                b"?1006h" => self.modes.sgr_mouse = true,
                b"?1006l" => self.modes.sgr_mouse = false,
                _ if csi.first() == Some(&b'>') && csi.last() == Some(&b'u') && self.kitty => {
                    let n = std::str::from_utf8(&csi[1..csi.len() - 1]).ok().and_then(|s| s.parse().ok()).unwrap_or(0);
                    self.modes.kitty.push(n);
                }
                _ if csi.first() == Some(&b'<') && csi.last() == Some(&b'u') => {
                    let n = std::str::from_utf8(&csi[1..csi.len() - 1]).ok().and_then(|s| s.parse().ok()).unwrap_or(1usize);
                    let keep = self.modes.kitty.len().saturating_sub(n.max(1));
                    self.modes.kitty.truncate(keep);
                }
                _ => {}
            }
            i = end;
        }
        if !replies.is_empty() {
            trace("reply", &replies);
            // Short, and into an input buffer the screen is reading: written whole.
            let _ = self.master.write_all(&replies);
        }
    }

    /// Reads until the screen has been still for `quiet`, has answered on its socket (it handles
    /// input and requests in order, and answers `screen` once a frame is drawn), and is still
    /// again: what the last input did is on screen.
    pub fn settle(&mut self) -> &mut Sim {
        let start = Instant::now();
        let still = |s: &mut Sim| {
            while s.read(s.quiet) && start.elapsed() < Duration::from_secs(5) {}
        };
        still(self);
        let _ = self.ask(json!({"op": "screen"}));
        still(self);
        self
    }

    /// Bytes, as the terminal would send them.
    pub fn bytes(&mut self, b: &[u8]) -> &mut Sim {
        trace("in", b);
        self.write(b);
        self.settle()
    }

    /// Writes all of `b`. The pty's input buffer is small (a long paste fills it), so this reads
    /// what the screen writes back meanwhile and tries again until it has all gone.
    fn write(&mut self, mut b: &[u8]) {
        let start = Instant::now();
        while !b.is_empty() && start.elapsed() < Duration::from_secs(10) {
            match self.master.write(b) {
                Ok(n) => b = &b[n..],
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::Interrupted => {
                    self.read(Duration::from_millis(5));
                }
                Err(_) => return,
            }
        }
    }

    /// Keys by name (`a`, `s-left`, `d-z`, `c-a-x`), each encoded as this terminal would send it
    /// under the flags the app pushed.
    pub fn keys(&mut self, keys: &[&str]) -> &mut Sim {
        for k in keys {
            match encode(k, self.modes.kitty.last().copied().unwrap_or(0)) {
                Some(b) => {
                    self.bytes(&b);
                }
                None => panic!("this terminal can't send {k} (no kitty keyboard protocol)"),
            }
        }
        self
    }

    /// The mouse at a cell, as an SGR report: `press`, `s-press`, `d-press` (sent with ⌥, the
    /// terminal's stand-in for ⌘, which a mouse report can't carry), `drag`, `release`, `move`,
    /// `wheel-up`, `wheel-down`.
    pub fn mouse(&mut self, kind: &str, x: u16, y: u16) -> &mut Sim {
        let (b, end) = match kind {
            "press" => (0, 'M'),
            "s-press" => (4, 'M'),
            "d-press" | "a-press" => (8, 'M'),
            "drag" => (32, 'M'),
            "release" => (0, 'm'),
            "wheel-up" => (64, 'M'),
            "wheel-down" => (65, 'M'),
            _ => (35, 'M'),
        };
        let b = format!("\x1b[<{b};{};{}{end}", x + 1, y + 1);
        self.bytes(b.as_bytes())
    }

    /// Text pasted: bracketed if the app asked for it, as typed otherwise.
    pub fn paste(&mut self, text: &str) -> &mut Sim {
        let b = if self.modes.bracketed_paste { format!("\x1b[200~{text}\x1b[201~") } else { text.to_string() };
        self.bytes(b.as_bytes())
    }

    /// The terminal resized: the pty's size changes and the app gets SIGWINCH.
    pub fn resize(&mut self, width: u16, height: u16) -> &mut Sim {
        let ws = libc::winsize { ws_row: height, ws_col: width, ws_xpixel: width * 8, ws_ypixel: height * 16 };
        // SAFETY: a valid descriptor and winsize.
        unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &ws) };
        self.vt.screen_mut().set_size(height, width);
        self.settle()
    }

    /// What the terminal shows, as text: one line per row, trailing spaces trimmed, a wide
    /// character's second cell as a space (as `fictty screen` writes a frame).
    pub fn screen(&mut self) -> String {
        self.settle();
        self.text()
    }

    fn text(&self) -> String {
        let screen = self.vt.screen();
        let (h, w) = screen.size();
        let mut out = String::new();
        for y in 0..h {
            let mut line = String::new();
            for x in 0..w {
                match screen.cell(y, x) {
                    Some(c) if c.has_contents() && !c.is_wide_continuation() => line.push_str(c.contents()),
                    _ => line.push(' '),
                }
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out
    }

    /// The frame the app says it drew (`fictty screen`).
    pub fn frame(&mut self) -> String {
        self.settle();
        let reply = self.request(json!({"op": "screen"}));
        let text = reply["screen"].as_str().unwrap_or_default().to_string();
        self.settle();
        text
    }

    /// Whether the terminal shows exactly the frame the app says it drew; where not, the first
    /// row that differs.
    pub fn check(&mut self) -> Result<(), String> {
        self.settle();
        let mut last = String::new();
        // A live part of the frame (the status bar's meter) can change between the frame the app
        // reports and the next one it draws: then ask again.
        for _ in 0..5 {
            let frame = self.request(json!({"op": "screen"}))["screen"].as_str().unwrap_or_default().to_string();
            while self.read(self.quiet) {}
            let shown = self.text();
            if frame == shown {
                return Ok(());
            }
            let (f, t): (Vec<&str>, Vec<&str>) = (frame.lines().collect(), shown.lines().collect());
            let y = (0..f.len().max(t.len())).find(|&y| f.get(y) != t.get(y)).unwrap_or(0);
            last = format!("row {y}:\n  frame    {:?}\n  terminal {:?}", f.get(y).unwrap_or(&""), t.get(y).unwrap_or(&""));
        }
        Err(last)
    }

    /// The app's state (`fictty get --state`).
    pub fn state(&mut self) -> Value {
        self.request(json!({"op": "state"}))
    }

    /// Any socket request, answered.
    pub fn request(&mut self, req: Value) -> Value {
        self.ask(req).unwrap_or_else(|e| json!({"ok": false, "status": e.to_string()}))
    }

    /// A socket request, sent from a thread while this keeps reading the pty: the screen may be
    /// waiting on an answer from its terminal (a query) before it gets to the request.
    fn ask(&mut self, req: Value) -> Result<Value> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = self.socket.clone();
        std::thread::spawn(move || {
            let _ = tx.send(crate::runtime::send(&socket, req));
        });
        loop {
            if let Ok(r) = rx.try_recv() {
                return r;
            }
            self.read(Duration::from_millis(5));
        }
    }

    /// The terminal's cell grid (styles, the cursor) for checks beyond text.
    pub fn vt(&self) -> &vt100::Screen {
        self.vt.screen()
    }

    /// Whether the app is still running.
    pub fn running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for Sim {
    fn drop(&mut self) {
        let _ = self.child.kill();
        // A process exiting with output nobody has read waits for it to drain (macOS's tty close
        // does), so read and drop what's there until it's gone.
        let mut buf = [0u8; 65536];
        let until = Instant::now() + Duration::from_secs(10);
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < until {
            let mut fds = libc::pollfd { fd: self.master.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY: one valid pollfd.
            if unsafe { libc::poll(&mut fds, 1, 10) } > 0 {
                while matches!(self.master.read(&mut buf), Ok(n) if n > 0) {}
            }
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.socket);
    }
}

/// With `FICTTY_SIM_TRACE=1`, every byte each way on stderr, escapes shown.
fn trace(what: &str, b: &[u8]) {
    if std::env::var_os("FICTTY_SIM_TRACE").is_some() {
        eprintln!("{what:>5} {}", b.escape_ascii());
    }
}

/// The bytes a terminal sends for a key name under the kitty keyboard `flags` the app pushed
/// (0: legacy escapes). None for a key legacy escapes can't carry (⌘ anything).
pub fn encode(name: &str, flags: u32) -> Option<Vec<u8>> {
    let (mut ctrl, mut alt, mut sup, mut shift) = (false, false, false, false);
    let mut rest = name;
    while let Some((m, r)) = rest.split_once('-').filter(|(m, r)| m.len() == 1 && !r.is_empty()) {
        match m {
            "c" => ctrl = true,
            "a" => alt = true,
            "d" => sup = true,
            "s" => shift = true,
            _ => return None,
        }
        rest = r;
    }
    let m = 1 + shift as u32 + 2 * alt as u32 + 4 * ctrl as u32 + 8 * sup as u32;
    let plain = m == 1;
    let kitty = flags & 1 != 0;
    let csi = |s: String| Some(format!("\x1b[{s}").into_bytes());
    // Keys with a CSI letter (arrows, Home, End) or a number and `~`.
    let letter = match rest {
        "up" => Some('A'),
        "down" => Some('B'),
        "right" => Some('C'),
        "left" => Some('D'),
        "home" => Some('H'),
        "end" => Some('F'),
        _ => None,
    };
    if let Some(l) = letter {
        if sup && !kitty {
            return None;
        }
        return if plain { csi(l.to_string()) } else { csi(format!("1;{m}{l}")) };
    }
    let tilde = match rest {
        "pgup" => Some(5),
        "pgdn" => Some(6),
        "del" => Some(3),
        "ins" => Some(2),
        _ => None,
    };
    if let Some(n) = tilde {
        if sup && !kitty {
            return None;
        }
        return if plain { csi(format!("{n}~")) } else { csi(format!("{n};{m}~")) };
    }
    let (code, legacy): (u32, Option<Vec<u8>>) = match rest {
        "enter" => (13, Some(b"\r".to_vec())),
        "tab" if shift && !ctrl && !alt && !sup => (9, Some(b"\x1b[Z".to_vec())),
        "tab" => (9, Some(b"\t".to_vec())),
        "bs" => (127, Some(b"\x7f".to_vec())),
        "esc" => (27, Some(b"\x1b".to_vec())),
        "space" => (32, Some(b" ".to_vec())),
        s => {
            let mut cs = s.chars();
            let (Some(c), None) = (cs.next(), cs.next()) else { return None };
            let shown = if shift { c.to_uppercase().collect::<String>() } else { c.to_string() };
            // Text: a character alone (or with Shift) is itself, in both.
            if !ctrl && !alt && !sup {
                return Some(shown.into_bytes());
            }
            let legacy = if sup {
                None
            } else if ctrl && c.is_ascii_alphabetic() {
                let b = (c.to_ascii_lowercase() as u8) & 0x1f;
                Some(if alt { vec![0x1b, b] } else { vec![b] })
            } else if alt && !ctrl {
                let mut v = vec![0x1b];
                v.extend(shown.bytes());
                Some(v)
            } else {
                None
            };
            (c.to_ascii_lowercase() as u32, legacy)
        }
    };
    if kitty {
        // Enter, Tab and Backspace alone stay legacy under "disambiguate"; Esc doesn't.
        if plain && code != 27 {
            return legacy;
        }
        return csi(if plain { format!("{code}u") } else { format!("{code};{m}u") });
    }
    match (plain, legacy) {
        (true, l) => l,
        // Alt as an Escape in front, Ctrl only on letters, Shift-Tab as CSI Z.
        (false, Some(l)) if rest.len() == 1 || rest == "tab" => Some(l),
        (false, Some(l)) if alt && !ctrl && !sup && !shift => Some([b"\x1b".as_slice(), &l].concat()),
        _ => None,
    }
}

#[cfg(feature = "fuzz")]
impl Sim {
    /// One of the fuzzer's acts, as a terminal delivers it. A wait does nothing here: a real
    /// screen has a real clock.
    pub fn act(&mut self, a: &crate::fuzz::Act) -> &mut Sim {
        use crate::fuzz::Act;
        match a {
            Act::Key(k) => self.keys(&[k]),
            Act::Mouse(k, x, y) => self.mouse(k, *x, *y),
            Act::Paste(t) => self.paste(t),
            Act::Resize(w, h) => self.resize(*w, *h),
            Act::Wait(_) => self,
            Act::Host(arg) => self.host_act(arg),
        }
    }

    /// What a host act (`fuzz::Act::Host`) does here: the app's handler, given the act's argument.
    pub fn on_host_act(mut self, f: impl FnMut(&mut Sim, &str) + 'static) -> Sim {
        self.host_act = Some(Box::new(f));
        self
    }

    /// Run a host act through the handler `on_host_act` gave, then wait for the screen to settle.
    pub fn host_act(&mut self, arg: &str) -> &mut Sim {
        let mut f = self.host_act.take().expect("a host act, but no handler: Sim::on_host_act");
        f(self, arg);
        self.host_act = Some(f);
        self.settle();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_encode_as_a_kitty_terminal_and_a_legacy_one_send_them() {
        let k = |n: &str| String::from_utf8(encode(n, 1).unwrap()).unwrap();
        assert_eq!(k("a"), "a");
        assert_eq!(k("s-a"), "A");
        assert_eq!(k("d-z"), "\x1b[122;9u");
        assert_eq!(k("d-s-z"), "\x1b[122;10u");
        assert_eq!(k("c-a"), "\x1b[97;5u");
        assert_eq!(k("esc"), "\x1b[27u");
        assert_eq!(k("enter"), "\r");
        assert_eq!(k("s-enter"), "\x1b[13;2u");
        assert_eq!(k("s-tab"), "\x1b[9;2u");
        assert_eq!(k("left"), "\x1b[D");
        assert_eq!(k("a-s-left"), "\x1b[1;4D");
        assert_eq!(k("d-bs"), "\x1b[127;9u");
        assert_eq!(k("d-del"), "\x1b[3;9~");
        let l = |n: &str| encode(n, 0).map(|b| String::from_utf8(b).unwrap());
        assert_eq!(l("c-a").as_deref(), Some("\x01"));
        assert_eq!(l("a-b").as_deref(), Some("\x1bb"));
        assert_eq!(l("s-tab").as_deref(), Some("\x1b[Z"));
        assert_eq!(l("esc").as_deref(), Some("\x1b"));
        assert_eq!(l("c-left").as_deref(), Some("\x1b[1;5D"));
        assert_eq!(l("d-z"), None, "⌘ needs the kitty protocol");
    }
}
