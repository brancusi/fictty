//! Fuzzing a screen: random sequences of keys, the mouse, pastes and resizes, driven through
//! `Headless`, with properties checked after every step. A failure shrinks to a short sequence and
//! prints as a scenario: the acts, and the focused editor before and after in the editing
//! notation (`Hello ⟦wor▮⟧ld`). `replay` runs a printed sequence again.
//!
//! Built in: nothing panics, every step stays inside its time budget, the same sequence on a
//! fresh screen gives the same state and frame, and, for a screen drawn as a grid, the frame
//! drawn as escapes (diffed frame to frame as a running screen draws) into a terminal emulator
//! shows exactly the frame. `editing()` adds the editing invariants; an app
//! adds its own with `check`.
//!
//! ```no_run
//! # use fictty_core::{App, fuzz};
//! # let ui: fictty_core::model::Ui = serde_json::from_str("{}").unwrap();
//! let make = || App::new(ui.clone()).headless(60, 12).unwrap();
//! if let Err(f) = fuzz::run(make, &mut fuzz::editing(), &fuzz::Config::default()) {
//!     panic!("{f}");
//! }
//! ```

use crate::engine::Headless;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

/// One thing the person does.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// A key by name (`a`, `s-left`, `d-z`).
    Key(String),
    /// The mouse (`press`, `s-press`, `drag`, `release`, `move`, `wheel-up`, `wheel-down`) at a cell.
    Mouse(String, u16, u16),
    Paste(String),
    Resize(u16, u16),
    /// The clock moved on by this many ms (toasts go, expiring writes lapse).
    Wait(u64),
    /// Something from outside the person: what the app's handler (`Headless::on_host_act`)
    /// does with this argument, an agent's write say.
    Host(String),
}

impl fmt::Display for Act {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Act::Key(k) => write!(f, "key {k}"),
            Act::Mouse(k, x, y) => write!(f, "{k} {x} {y}"),
            Act::Paste(t) => write!(f, "paste {}", Value::String(t.clone())),
            Act::Resize(w, h) => write!(f, "resize {w} {h}"),
            Act::Wait(ms) => write!(f, "wait {ms}"),
            Act::Host(a) => write!(f, "host {}", Value::String(a.clone())),
        }
    }
}

impl Act {
    /// An act as `Display` writes it.
    pub fn parse(s: &str) -> Option<Act> {
        let (verb, rest) = s.trim().split_once(' ')?;
        let nums = || rest.split_whitespace().map(|n| n.parse::<u16>().ok()).collect::<Option<Vec<_>>>();
        Some(match verb {
            "key" => Act::Key(rest.trim().into()),
            "paste" => Act::Paste(serde_json::from_str(rest).ok()?),
            "host" => Act::Host(serde_json::from_str(rest).ok()?),
            "resize" => match nums()?[..] {
                [w, h] => Act::Resize(w, h),
                _ => return None,
            },
            "wait" => Act::Wait(rest.trim().parse().ok()?),
            kind => match nums()?[..] {
                [x, y] => Act::Mouse(kind.into(), x, y),
                _ => return None,
            },
        })
    }

    /// Do it to the screen.
    pub fn apply(&self, s: &mut Headless) {
        match self {
            Act::Key(k) => {
                s.keys(&[k]);
            }
            Act::Mouse(k, x, y) => {
                s.mouse(k, *x, *y);
            }
            Act::Paste(t) => {
                s.paste(t);
            }
            Act::Resize(w, h) => {
                s.resize(*w, *h);
            }
            Act::Wait(ms) => {
                let now_ms = s.state().now_ms + ms;
                s.msg(crate::state::Msg::Tick { now_ms });
            }
            Act::Host(a) => {
                s.host_act(a);
            }
        }
    }
}

/// A sequence as one line: acts separated by `; `.
pub fn line(acts: &[Act]) -> String {
    acts.iter().map(|a| a.to_string()).collect::<Vec<_>>().join("; ")
}

/// A line back into its acts. A paste's text is JSON, so it can hold `; `.
pub fn parse_line(mut s: &str) -> Result<Vec<Act>, String> {
    let mut out = Vec::new();
    loop {
        s = s.trim_start_matches([';', ' ', '\n']);
        if s.is_empty() {
            return Ok(out);
        }
        let verb = ["paste ", "host "].into_iter().find(|v| s.starts_with(v));
        let len = match verb {
            Some(v) => {
                let mut it = serde_json::Deserializer::from_str(&s[v.len()..]).into_iter::<String>();
                it.next().ok_or("an act with no text")?.map_err(|e| format!("an act's text: {e}"))?;
                v.len() + it.byte_offset()
            }
            None => s.find("; ").unwrap_or(s.len()),
        };
        out.push(Act::parse(&s[..len]).ok_or_else(|| format!("not an act: {}", &s[..len]))?);
        s = &s[len..];
    }
}

/// What to generate, and how much.
#[derive(Clone, Debug)]
pub struct Config {
    /// Each run's sequence comes from this seed and the run's number, so a failure reproduces.
    pub seed: u64,
    pub runs: u32,
    /// Acts in a run (gestures count each of their acts).
    pub steps: usize,
    /// The longest one act may take.
    pub budget: Duration,
    /// Keys to choose from, by name.
    pub keys: Vec<String>,
    /// Sequences of keys that go in together, so properties that span them get a chance
    /// (`["d-x", "d-v"]`).
    pub combos: Vec<Vec<String>>,
    /// Texts a paste chooses from.
    pub pastes: Vec<String>,
    /// Arguments a host act chooses from (`Act::Host`, run by the app's handler).
    pub host_acts: Vec<String>,
    /// How often (out of 100) an act is the mouse, a paste, a resize or a wait; the rest are keys.
    pub mouse: u32,
    pub paste: u32,
    pub resize: u32,
    pub wait: u32,
    /// How often (out of 100) an act is a host act, when there are any.
    pub host: u32,
    /// Run each passing sequence again on a fresh screen, and compare the state and frame.
    pub replay: bool,
    /// Draw every frame through the crossterm backend into a terminal emulator too, and check
    /// that the emulator shows the frame (`Headless::terminal`).
    pub terminal: bool,
    /// Applied to the state before two runs of a sequence are compared: masks what's meant to
    /// differ (a random id, a timestamp from the host).
    pub normalize: Option<fn(&mut Value)>,
    /// What makes two failures of a property the same one, from the property and the message:
    /// shrinking keeps to the class it started with. Unset, a property is one class.
    pub class: Option<fn(&str, &str) -> String>,
}

impl Default for Config {
    fn default() -> Config {
        let mut keys: Vec<String> = ["a", "b", "x", "space", "1", "-", "[", "]", ".", "enter", "tab", "s-tab", "bs", "del", "esc"].map(String::from).into();
        for k in ["left", "right", "up", "down", "home", "end", "pgup", "pgdn"] {
            for m in ["", "s-", "a-", "a-s-", "c-", "c-s-", "d-", "d-s-"] {
                keys.push(format!("{m}{k}"));
            }
        }
        keys.extend(["d-a", "d-c", "d-x", "d-v", "d-z", "d-s-z", "c-a", "c-c", "c-x", "c-v", "c-z", "a-bs", "d-bs", "c-w", "c-k", "c-t"].map(String::from));
        let combos = [["d-x", "d-v"], ["d-c", "d-v"], ["d-z", "d-s-z"]].map(|c| c.map(String::from).into()).into();
        let mut pastes: Vec<String> =
            ["word", "two words", "line one\nline two", "- [ ] a task", "1. one\n2. two", "  indented", "émoji 🙂 wide 漢字", ""].map(String::from).into();
        // A long paragraph: work that grows with a line's length shows up as a slow act.
        pastes.push("Words with [[a link]], #a-tag, `code` and https://example.com in a long paragraph. ".repeat(20));
        Config { seed: 1, runs: 64, steps: 60, budget: Duration::from_millis(250), keys, combos, pastes, host_acts: vec![], mouse: 12, paste: 6, resize: 2, wait: 3, host: 0, replay: true, terminal: true, normalize: None, class: None }
    }
}

/// What a property compares between acts: the frame, the focus and the text. Anything else it
/// reads from the screen's state (`Step::screen`).
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub screen: String,
    pub focus: Option<String>,
    /// Each editor's text, anchor and caret (chars), by id.
    pub editors: BTreeMap<String, (String, usize, usize)>,
    /// The store entry each editor shows, by id.
    pub shows: BTreeMap<String, String>,
    /// Every text entry in the store, by key.
    pub docs: BTreeMap<String, String>,
}

impl Look {
    pub fn of(s: &mut Headless) -> Look {
        let screen = s.screen();
        let focus = s.state().focus.clone();
        let ids: Vec<String> = s.state().local.keys().cloned().collect();
        let editors = ids.iter().filter_map(|id| Some((id.clone(), editor(s, id)?))).collect();
        let st = s.state();
        let shows = ids.iter().filter_map(|id| Some((id.clone(), crate::state::field_view(st, id)?.0))).collect();
        let docs = st
            .store
            .iter()
            .filter_map(|(k, e)| match &e.body {
                crate::store::Body::Text { doc } => Some((k.clone(), doc.text.to_string())),
                _ => None,
            })
            .collect();
        Look { screen, focus, editors, shows, docs }
    }

    /// Whether editor `id` shows the same entry in both looks.
    pub fn same_doc(&self, other: &Look, id: &str) -> bool {
        self.shows.get(id).is_some_and(|k| other.shows.get(id) == Some(k))
    }

    /// The focused editor: its id, text, anchor and caret.
    pub fn focused(&self) -> Option<(&str, &str, usize, usize)> {
        let id = self.focus()?;
        let (t, a, h) = self.editors.get(id)?;
        Some((id, t, *a, *h))
    }

    /// The focused node's id.
    pub fn focus(&self) -> Option<&str> {
        self.focus.as_deref()
    }
}

/// What a property is shown after each act: the screen (to read more from it, or to act on a
/// copy of what it learned), the act, and the look before and after.
pub struct Step<'a> {
    pub screen: &'a mut Headless,
    pub act: &'a Act,
    pub before: &'a Look,
    pub after: &'a Look,
    /// The acts so far, this one last.
    pub acts: &'a [Act],
}

/// A named property: checked after every act, and once when a run ends.
pub struct Check {
    pub name: String,
    #[allow(clippy::type_complexity)]
    step: Box<dyn FnMut(&mut Step) -> Result<(), String>>,
    #[allow(clippy::type_complexity)]
    end: Box<dyn FnMut(&mut Headless, &Look) -> Result<(), String>>,
    reset: Box<dyn FnMut()>,
}

/// A property checked after every act.
pub fn check(name: &str, step: impl FnMut(&mut Step) -> Result<(), String> + 'static) -> Check {
    Check { name: name.into(), step: Box::new(step), end: Box::new(|_, _| Ok(())), reset: Box::new(|| {}) }
}

impl Check {
    /// Also checked once at the end of a run, with the look at its start. It may act on the
    /// screen (undo everything, say).
    pub fn at_end(mut self, end: impl FnMut(&mut Headless, &Look) -> Result<(), String> + 'static) -> Check {
        self.end = Box::new(end);
        self
    }

    /// Called before each run, for a property that remembers across acts.
    pub fn on_reset(mut self, reset: impl FnMut() + 'static) -> Check {
        self.reset = Box::new(reset);
        self
    }
}

/// A failed property, its sequence shrunk.
#[derive(Clone, Debug)]
pub struct Failure {
    pub property: String,
    pub message: String,
    pub seed: u64,
    /// The shrunk sequence; it fails at its last act (or at the end, for an end check).
    pub acts: Vec<Act>,
    /// The focused editor, in the editing notation, at the start and after each act.
    pub goldens: Vec<Option<String>>,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "property \"{}\" failed (seed {}, {} acts after shrinking)", self.property, self.seed, self.acts.len())?;
        writeln!(f, "  {}", self.message)?;
        let g = |i: usize| self.goldens.get(i).cloned().flatten().unwrap_or_else(|| "(no editor focused)".into());
        writeln!(f, "  start   {}", g(0))?;
        for (i, a) in self.acts.iter().enumerate() {
            writeln!(f, "  {:<28}→ {}", a.to_string(), g(i + 1))?;
        }
        write!(f, "  replay: fuzz::replay(make, {:?})", line(&self.acts))
    }
}

/// splitmix64: a small seeded generator, so a run is the same everywhere.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }

    fn pick<'a, T>(&mut self, of: &'a [T]) -> Option<&'a T> {
        of.get(self.below(of.len() as u64) as usize)
    }
}

/// A run's sequence.
pub fn generate(cfg: &Config, seed: u64, size: (u16, u16)) -> Vec<Act> {
    let mut r = Rng(seed);
    let (mut w, mut h) = size;
    let mut out = Vec::new();
    while out.len() < cfg.steps {
        let roll = r.below(100) as u32;
        let (x, y) = (r.below(w as u64) as u16, r.below(h as u64) as u16);
        if roll < cfg.mouse {
            match r.below(6) {
                0 => out.extend([Act::Mouse("press".into(), x, y), Act::Mouse("release".into(), x, y)]),
                1 => {
                    // ⇧ or ⌘ held: extending a caret, picking rows of a `multi` list.
                    let kind = if r.below(2) == 0 { "s-press" } else { "d-press" };
                    out.extend([Act::Mouse(kind.into(), x, y), Act::Mouse("release".into(), x, y)]);
                }
                2 => {
                    // A double or triple click.
                    for _ in 0..2 + r.below(2) {
                        out.extend([Act::Mouse("press".into(), x, y), Act::Mouse("release".into(), x, y)]);
                    }
                }
                3 => {
                    out.push(Act::Mouse("press".into(), x, y));
                    for _ in 0..1 + r.below(3) {
                        out.push(Act::Mouse("drag".into(), r.below(w as u64) as u16, r.below(h as u64) as u16));
                    }
                    out.push(Act::Mouse("release".into(), x, y));
                }
                4 => out.push(Act::Mouse(if r.below(2) == 0 { "wheel-up" } else { "wheel-down" }.into(), x, y)),
                _ => out.push(Act::Mouse("move".into(), x, y)),
            }
        } else if roll < cfg.mouse + cfg.paste {
            if let Some(t) = r.pick(&cfg.pastes) {
                out.push(Act::Paste(t.clone()));
            }
        } else if roll < cfg.mouse + cfg.paste + cfg.resize {
            (w, h) = (20 + r.below(140) as u16, 4 + r.below(50) as u16);
            out.push(Act::Resize(w, h));
        } else if roll < cfg.mouse + cfg.paste + cfg.resize + cfg.wait {
            out.push(Act::Wait([100, 1000, 5000][r.below(3) as usize]));
        } else if roll < cfg.mouse + cfg.paste + cfg.resize + cfg.wait + cfg.host && !cfg.host_acts.is_empty() {
            out.push(Act::Host(r.pick(&cfg.host_acts).expect("host acts").clone()));
        } else if r.below(10) == 0 && !cfg.combos.is_empty() {
            let c = r.pick(&cfg.combos).expect("combos");
            out.extend(c.iter().map(|k| Act::Key(k.clone())));
        } else if let Some(k) = r.pick(&cfg.keys) {
            out.push(Act::Key(k.clone()));
        }
    }
    out
}

/// Where a sequence failed: the property, why, and how many acts it took (`acts.len()` + 1 for
/// an end check).
struct Fail {
    property: String,
    message: String,
    at: usize,
}

/// Plays `acts` on a fresh screen, checking every property; the first failure.
fn play(make: &dyn Fn() -> Headless, acts: &[Act], checks: &mut [Check], cfg: &Config, goldens: Option<&mut Vec<Option<String>>>) -> Result<(Look, Value), Fail> {
    for c in checks.iter_mut() {
        (c.reset)();
    }
    let fail = |property: &str, message: String, at| Fail { property: property.into(), message, at };
    let mut s = catch_unwind(AssertUnwindSafe(make)).map_err(|p| fail("no panic", format!("making the screen panicked: {}", panic_text(&p)), 0))?;
    if cfg.terminal {
        s.terminal();
    }
    let start = Look::of(&mut s);
    let mut before = start.clone();
    let mut goldens = goldens;
    if let Some(g) = goldens.as_deref_mut() {
        g.push(focused_golden(&mut s));
    }
    for (i, act) in acts.iter().enumerate() {
        let t = Instant::now();
        catch_unwind(AssertUnwindSafe(|| act.apply(&mut s))).map_err(|p| fail("no panic", format!("{act} panicked: {}", panic_text(&p)), i + 1))?;
        let took = t.elapsed();
        if took > cfg.budget {
            return Err(fail("inside the budget", format!("{act} took {took:?}, over {:?}", cfg.budget), i + 1));
        }
        let after = Look::of(&mut s);
        if cfg.terminal {
            let shown = s.terminal();
            if shown != after.screen {
                return Err(fail("the terminal shows the frame", differs(&after.screen, &shown), i + 1));
            }
        }
        if let Some(g) = goldens.as_deref_mut() {
            g.push(focused_golden(&mut s));
        }
        for c in checks.iter_mut() {
            let mut step = Step { screen: &mut s, act, before: &before, after: &after, acts: &acts[..=i] };
            let r = catch_unwind(AssertUnwindSafe(|| (c.step)(&mut step)));
            match r {
                Ok(Ok(())) => {}
                Ok(Err(m)) => return Err(fail(&c.name, m, i + 1)),
                Err(p) => return Err(fail(&c.name, format!("the check panicked: {}", panic_text(&p)), i + 1)),
            }
        }
        before = after;
        // A running screen would have ended here: nothing after it happens.
        if s.quit() {
            break;
        }
    }
    let end = before;
    // The state the acts left, before an end check acts on the screen.
    let mut state = serde_json::to_value(s.state()).unwrap_or_default();
    // Measurements, not state: they differ between two runs of the same sequence.
    if let Some(o) = state.as_object_mut() {
        o.remove("stats");
    }
    if let Some(n) = cfg.normalize {
        n(&mut state);
    }
    for c in checks.iter_mut() {
        let r = catch_unwind(AssertUnwindSafe(|| (c.end)(&mut s, &start)));
        match r {
            Ok(Ok(())) => {}
            Ok(Err(m)) => return Err(fail(&c.name, m, acts.len() + 1)),
            Err(p) => return Err(fail(&c.name, format!("the end check panicked: {}", panic_text(&p)), acts.len() + 1)),
        }
    }
    Ok((end, state))
}

/// One sequence, with the replay check: the failure, if any.
fn trial(make: &dyn Fn() -> Headless, acts: &[Act], checks: &mut [Check], cfg: &Config) -> Option<Fail> {
    let first = match play(make, acts, checks, cfg, None) {
        Ok(look) => look,
        Err(f) => return Some(f),
    };
    if cfg.replay {
        match play(make, acts, &mut [], cfg, None) {
            Ok(again) if again != first => {
                let what = if again.0 != first.0 { "frame" } else { "state" };
                return Some(Fail { property: "replays the same".into(), message: format!("the same acts on a fresh screen gave a different {what}"), at: acts.len() + 1 });
            }
            Ok(_) => {}
            Err(f) => return Some(f),
        }
    }
    None
}

/// Generate `cfg.runs` sequences and check each; the first failure, shrunk.
pub fn run(make: impl Fn() -> Headless, checks: &mut [Check], cfg: &Config) -> Result<(), Failure> {
    let size = make().size();
    for n in 0..cfg.runs {
        let seed = cfg.seed.wrapping_mul(1_000_003).wrapping_add(n as u64);
        let acts = generate(cfg, seed, size);
        if let Some(f) = trial(&make, &acts, checks, cfg) {
            return Err(shrink(&make, acts, f, checks, cfg, seed));
        }
    }
    Ok(())
}

/// Play a printed sequence (`"key a; press 3 4; paste \"x\""`) and check it.
pub fn replay(make: impl Fn() -> Headless, acts: &str, checks: &mut [Check], cfg: &Config) -> Result<(), Failure> {
    let acts = parse_line(acts).unwrap_or_else(|e| panic!("{e}"));
    match trial(&make, &acts, checks, cfg) {
        Some(f) => Err(described(&make, acts, f, checks, cfg, 0)),
        None => Ok(()),
    }
}

/// The shortest sequence that still fails the same way (the same property, and the same class
/// when `Config::class` says): drop chunks of acts, halving the
/// chunk size (ddmin), until no single act can go.
fn shrink(make: &dyn Fn() -> Headless, mut acts: Vec<Act>, mut f: Fail, checks: &mut [Check], cfg: &Config, seed: u64) -> Failure {
    acts.truncate(f.at.min(acts.len()));
    let mut chunk = acts.len().div_ceil(2).max(1);
    loop {
        let mut i = 0;
        let mut dropped = false;
        while i < acts.len() {
            let mut fewer = acts.clone();
            fewer.drain(i..(i + chunk).min(acts.len()));
            match trial(make, &fewer, checks, cfg) {
                Some(g) if class(cfg, &g) == class(cfg, &f) => {
                    acts = fewer;
                    acts.truncate(g.at.min(acts.len()));
                    f = g;
                    dropped = true;
                }
                _ => i += chunk,
            }
        }
        if chunk == 1 && !dropped {
            break;
        }
        if !dropped {
            chunk = chunk.div_ceil(2);
        }
    }
    described(make, acts, f, checks, cfg, seed)
}

/// A failure's class, which shrinking keeps to.
fn class(cfg: &Config, f: &Fail) -> String {
    match cfg.class {
        Some(c) => format!("{}: {}", f.property, c(&f.property, &f.message)),
        None => f.property.clone(),
    }
}

fn described(make: &dyn Fn() -> Headless, acts: Vec<Act>, f: Fail, checks: &mut [Check], cfg: &Config, seed: u64) -> Failure {
    let mut goldens = Vec::new();
    let _ = play(make, &acts, checks, cfg, Some(&mut goldens));
    Failure { property: f.property, message: f.message, seed, acts, goldens }
}

/// The first line where the frame and the terminal differ.
fn differs(frame: &str, terminal: &str) -> String {
    let (f, t): (Vec<&str>, Vec<&str>) = (frame.lines().collect(), terminal.lines().collect());
    let y = (0..f.len().max(t.len())).find(|&y| f.get(y) != t.get(y)).unwrap_or(0);
    format!("row {y}:\n    frame    {:?}\n    terminal {:?}", f.get(y).unwrap_or(&""), t.get(y).unwrap_or(&""))
}

fn panic_text(p: &Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "a panic".into())
}

/// An editor's text, anchor and caret (chars): its own view of the entry it shows.
pub fn editor(s: &mut Headless, id: &str) -> Option<(String, usize, usize)> {
    let st = s.state();
    let (key, view) = crate::state::field_view(st, id)?;
    let text = st.store.get(&key)?.read()["text"].as_str()?.to_string();
    let r = view.selection.primary();
    Some((text, r.anchor, r.head))
}

/// The focused editor's text in the editing notation: the selection in ⟦ ⟧, the caret as ▮.
pub fn focused_golden(s: &mut Headless) -> Option<String> {
    let id = s.state().focus.clone()?;
    let (text, anchor, head) = editor(s, &id)?;
    Some(format!("{id}: {}", golden(&text, anchor, head)))
}

/// `Hello ⟦wor▮⟧ld`.
pub fn golden(text: &str, anchor: usize, head: usize) -> String {
    let (lo, hi) = (anchor.min(head), anchor.max(head));
    let mut out = String::new();
    for (i, c) in text.chars().map(Some).chain([None]).enumerate() {
        if lo != hi && i == lo {
            out.push('⟦');
        }
        if i == head {
            out.push('▮');
        }
        if lo != hi && i == hi {
            out.push('⟧');
        }
        if let Some(c) = c {
            out.push(c);
        }
    }
    out.replace('\n', "⏎")
}

/// Whether a key moves the caret without extending the selection.
fn plain_motion(key: &str) -> bool {
    let (mods, base) = match key.rsplit_once('-') {
        Some((m, b)) if !b.is_empty() => (m, b),
        _ => ("", key),
    };
    !mods.split('-').any(|m| m == "s") && ["left", "right", "up", "down", "home", "end", "pgup", "pgdn"].contains(&base)
}

/// What a frame laid out nests: each node with a rectangle sits inside its nearest ancestor with
/// one, and two that aren't one inside the other don't overlap, unless they part at a stack,
/// whose children lie over each other by design (layers, dialogs, floats).
pub fn laid_out() -> Check {
    check("laid out without overlap", |s| nested(&s.screen.readback()))
}

/// Whether the readback's rectangles nest (see `laid_out`).
pub fn nested(nodes: &[crate::readback::Shown]) -> Result<(), String> {
    use crate::engine::Rect;
    // Each node's parent, from the depths of the nodes before it.
    let mut parent: Vec<Option<usize>> = Vec::with_capacity(nodes.len());
    let mut open: Vec<usize> = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        while open.last().is_some_and(|&o| nodes[o].depth >= n.depth) {
            open.pop();
        }
        parent.push(open.last().copied());
        open.push(i);
    }
    let ancestors = |mut i: usize| {
        let mut up = Vec::new();
        while let Some(p) = parent[i] {
            up.push(p);
            i = p;
        }
        up
    };
    let name = |i: usize| format!("{} {}", nodes[i].kind, nodes[i].id.as_deref().unwrap_or("?"));
    let laid = |i: usize| nodes[i].rect.filter(|r| r.width > 0 && r.height > 0);
    let inside = |a: Rect, b: Rect| a.x >= b.x && a.y >= b.y && a.x + a.width <= b.x + b.width && a.y + a.height <= b.y + b.height;
    let apart = |a: Rect, b: Rect| a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y;
    let ups: Vec<Vec<usize>> = (0..nodes.len()).map(ancestors).collect();
    for i in 0..nodes.len() {
        let Some(r) = laid(i) else { continue };
        if let Some((a, outer)) = ups[i].iter().find_map(|&a| laid(a).map(|o| (a, o)))
            && !inside(r, outer)
        {
            return Err(format!("{} at {r:?} sticks out of {} at {outer:?}", name(i), name(a)));
        }
        for j in i + 1..nodes.len() {
            let Some(q) = laid(j) else { continue };
            if ups[j].contains(&i) || apart(r, q) {
                continue;
            }
            // Where their lines part: the nearest node both are under.
            let fork = ups[j].iter().find(|a| ups[i].contains(a));
            if fork.is_some_and(|&f| nodes[f].kind == "stack") {
                continue;
            }
            return Err(format!("{} at {r:?} overlaps {} at {q:?}", name(i), name(j)));
        }
    }
    Ok(())
}

/// The editing invariants, for any screen with editors:
/// - a motion without Shift leaves no selection (a key the editor ignores isn't one);
/// - copy changes neither the text nor the selection;
/// - cut then paste gives back the text it started with (but in an editor that renumbers its
///   lists, which may renumber what the cut left);
/// - undoing everything (in each editor the run typed in) gives back the text it started with.
pub fn editing() -> Vec<Check> {
    use std::cell::RefCell;
    use std::rc::Rc;
    let key = |a: &Act| match a {
        Act::Key(k) => k.clone(),
        _ => String::new(),
    };
    let motion = check("a plain motion leaves no selection", move |s| {
        let k = key(s.act);
        match (s.before.focused(), s.after.focused()) {
            // A key the editor doesn't bind changes nothing, and isn't a motion.
            (Some(b), Some((id, text, a, h))) if b.0 == id && s.before.same_doc(s.after, id) && plain_motion(&k) && a != h && b != (id, text, a, h) => {
                Err(format!("{k} in {id} left {}", golden(text, a, h)))
            }
            _ => Ok(()),
        }
    });
    let copy = check("copy changes nothing", move |s| {
        let k = key(s.act);
        match (s.before.focused(), s.after.focused()) {
            (Some(b), Some(a)) if matches!(k.as_str(), "d-c" | "c-c") && b.0 == a.0 && s.before.same_doc(s.after, a.0) && b != a => {
                Err(format!("{k} in {}: {} became {}", a.0, golden(b.1, b.2, b.3), golden(a.1, a.2, a.3)))
            }
            _ => Ok(()),
        }
    });
    // The editor and its text just before a cut, until the next act.
    let cut: Rc<RefCell<Option<(String, String)>>> = Rc::default();
    let reset = cut.clone();
    let roundtrip = check("cut then paste gives the text back", move |s| {
        let k = key(s.act);
        let pending = cut.borrow_mut().take();
        let (Some(b), Some(a)) = (s.before.focused(), s.after.focused()) else { return Ok(()) };
        // An editor that keeps its lists in number renumbers what a cut leaves behind; that's
        // an edit of its own, which the paste doesn't take back.
        if s.screen.state().tree().find(a.0).and_then(|n| n.field()).is_some_and(|f| !f.ordered.is_empty()) {
            return Ok(());
        }
        // A cut with nothing selected cuts nothing, and the paste after it is an older copy.
        if matches!(k.as_str(), "d-x" | "c-x") && b.0 == a.0 && s.before.same_doc(s.after, a.0) && b.2 != b.3 {
            *cut.borrow_mut() = Some((b.0.to_string(), b.1.to_string()));
        } else if let Some((id, text)) = pending
            && matches!(k.as_str(), "d-v" | "c-v")
            && id == a.0
            && s.before.same_doc(s.after, a.0)
            && text != a.1
        {
            return Err(format!("cut then paste in {id}: {text:?} became {}", golden(a.1, a.2, a.3)));
        }
        Ok(())
    })
    .on_reset(move || *reset.borrow_mut() = None);
    let undo = check("undoing everything gives the text back", |_| Ok(())).at_end(|s, start| {
        // Every text that differs from the start and an editor shows now: focus that editor and
        // undo until nothing changes.
        for (key, was) in &start.docs {
            let now = Look::of(s);
            if now.docs.get(key) == Some(was) {
                continue;
            }
            // An editor that can change it: a read-only one (a preview of the same text) has no
            // undo of its own.
            let tree = s.state().tree();
            let editable = |id: &str| tree.find(id).and_then(|n| n.field()).is_some_and(|f| !f.readonly);
            let Some(id) = now.shows.iter().find(|(id, k)| *k == key && editable(id)).map(|(id, _)| id.clone()) else { continue };
            s.msg(crate::state::Msg::Focus { id: id.clone() });
            if s.state().focus.as_deref() != Some(&id) {
                continue;
            }
            // Where the document's undo history is, and whether a change from elsewhere (an
            // agent's write) is a floor undo stops at.
            let at = |s: &mut Headless| match s.state().store.get(key).map(|e| &e.body) {
                Some(crate::store::Body::Text { doc }) => Some((doc.history.current_revision(), doc.undo_floor, doc.text.to_string())),
                _ => None,
            };
            let mut prev = at(s);
            for _ in 0..5000 {
                s.keys(&["d-z"]);
                let now = at(s);
                let stuck = now.as_ref().map(|n| n.0) == prev.as_ref().map(|p| p.0);
                prev = now;
                if stuck {
                    break;
                }
            }
            if prev.as_ref().is_some_and(|p| p.1) {
                continue;
            }
            let prev = prev.map(|p| p.2);
            if prev.as_ref() != Some(was) {
                return Err(format!("undoing everything in {key} (in {id}) left {prev:?}; it started as {was:?}"));
            }
        }
        Ok(())
    });
    vec![motion, copy, roundtrip, undo]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nesting_finds_what_sticks_out_and_what_overlaps() {
        use crate::engine::Rect;
        use crate::readback::Shown;
        let n = |depth, kind: &str, id: &str, r: Option<(u16, u16, u16, u16)>| Shown {
            depth,
            kind: kind.into(),
            id: Some(id.into()),
            title: None,
            text: String::new(),
            focus: false,
            rows: None,
            selected: None,
            picked: vec![],
            rect: r.map(|(x, y, w, h)| Rect::new(x, y, w, h)),
            style: Default::default(),
            cut: false,
        };
        let row = |a: (u16, u16, u16, u16), b: (u16, u16, u16, u16)| vec![n(0, "row", "r", Some((0, 0, 20, 5))), n(1, "list", "a", Some(a)), n(1, "col", "c", None), n(2, "list", "b", Some(b))];
        assert_eq!(nested(&row((0, 0, 10, 5), (10, 0, 10, 5))), Ok(()));
        let over = nested(&row((0, 0, 11, 5), (10, 0, 10, 5))).unwrap_err();
        assert!(over.contains("list a") && over.contains("overlaps list b"), "{over}");
        let out = nested(&row((0, 0, 10, 5), (10, 0, 11, 5))).unwrap_err();
        assert!(out.contains("list b") && out.contains("sticks out of row r"), "through a node with no rectangle: {out}");
        // Over a stack, children lie over each other.
        let stack = vec![n(0, "stack", "s", Some((0, 0, 20, 5))), n(1, "list", "a", Some((0, 0, 20, 5))), n(1, "text", "f", Some((5, 1, 10, 2)))];
        assert_eq!(nested(&stack), Ok(()));
    }

    #[test]
    fn acts_print_and_parse_back() {
        let acts = vec![Act::Key("d-s-z".into()), Act::Mouse("s-press".into(), 3, 4), Act::Paste("a; b\n\"c\"".into()), Act::Resize(80, 24), Act::Wait(1000)];
        assert_eq!(parse_line(&line(&acts)).unwrap(), acts);
    }

    #[test]
    fn plain_motions_are_unshifted_moves() {
        assert!(plain_motion("left") && plain_motion("a-left") && plain_motion("d-down"));
        assert!(!plain_motion("s-left") && !plain_motion("a-s-left") && !plain_motion("a") && !plain_motion("-"));
    }

    #[test]
    fn a_sequence_is_the_same_for_a_seed() {
        let cfg = Config::default();
        assert_eq!(generate(&cfg, 7, (80, 24)), generate(&cfg, 7, (80, 24)));
        assert_ne!(generate(&cfg, 7, (80, 24)), generate(&cfg, 8, (80, 24)));
    }
}
