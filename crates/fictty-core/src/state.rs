//! The screen's whole state, the messages that change it and a pure `update`. The runtime owns
//! the terminal, the socket and child processes; it feeds messages in and carries effects out.
//! `State` serializes, so `fictty get --state` shows exactly what is drawn.

use crate::model::{Action, Kind, LayerSpec, Node, ThemeSpec, Ui, lookup};
use crate::store::{Body, Entry, Write};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// What the screen can draw: pictures, and the cell size in pixels. Found by whatever draws it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Caps {
    pub graphics: bool,
    /// Pixels per cell (0 when unknown).
    pub cell_w: f32,
    pub cell_h: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub ui: Ui,
    /// Per-node view state, by node id. Survives a push for every id still in the new tree.
    #[serde(default)]
    pub local: BTreeMap<String, Local>,
    #[serde(default)]
    pub focus: Option<String>,
    /// The last value fetched for each source.
    #[serde(default)]
    pub data: BTreeMap<String, Slot>,
    /// The app's data: written by the person and by agents, never by a push (see `store`).
    #[serde(default)]
    pub store: BTreeMap<String, Entry>,
    /// Writes to the store so far, so the runtime knows when to save it.
    #[serde(default)]
    pub store_writes: u64,
    /// The themes the person or project loaded (`--themes`), by name. A push doesn't change them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub themes: BTreeMap<String, ThemeSpec>,
    /// The active theme's name: the person's choice, like a selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// What's wrong with the UI against the store and the theme, in sentences an agent can act
    /// on: a bound value of the wrong shape, an unknown variant or tone. Checked after every
    /// message; the status line shows the first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub problems: Vec<String>,
    /// One line at the bottom: the last error or message.
    #[serde(default)]
    pub status: Option<String>,
    /// How many UIs have been pushed (the first load counts).
    #[serde(default)]
    pub version: u64,
    /// The runtime's measurements, also bindable as the `$stats` source.
    #[serde(default)]
    pub stats: Stats,
    /// What the mouse is over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover: Option<Hover>,
    /// Rows being dragged to a new place in their list, while the button is down.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag: Option<Drag>,
    /// The logical clock (ms), moved only by `Msg::Tick`; animations read it.
    #[serde(default)]
    pub now_ms: u64,
    /// What the terminal can draw (pictures, the cell size in pixels, larger text).
    #[serde(default)]
    pub caps: Caps,
    /// Whether larger text (OSC 66) is on: the terminal's to say, or `FICTTY_TEXT_SIZE=1`.
    #[serde(default)]
    pub text_sizing: bool,
    /// The font files themes name that whatever draws couldn't load, and why, by path.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fonts_failed: BTreeMap<String, String>,
    /// The rows a node shows when they aren't its bound array as it is: a tree's open rows in
    /// order, a filtered list's matches. Worked out from the rest after each message (so never
    /// saved); `items` reads them.
    #[serde(skip)]
    pub rows: BTreeMap<String, Vec<Value>>,
    /// Messages showing in the corner until their time is up, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub toasts: Vec<Toast>,
    /// The last click in an editor's text and how many came in a row there (a double click
    /// selects a word, a triple the line).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clicks: Option<Clicks>,
    /// Store keys the host keeps (by prefix: `page/`): loaded from it when something on screen
    /// first needs them, never seeded, and saved back to it after they change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosted: Vec<String>,
    /// Hosted entries asked for and not here yet, with why one failed to load (a failed one
    /// isn't asked for again until the host says it changed).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub loading: BTreeMap<String, Option<String>>,
    /// Each host source's `args` with their templates filled from the state (the selection, the
    /// store): what it was last fetched with. When they come out different, it's fetched again.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, Value>,
    /// The keys of a chord typed so far (`p` of `p h`), waiting for the rest. Bindable as
    /// `$keys` (`pending`, and `next`: the keys that would finish or continue it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chord: Option<String>,
    /// Where the focus was before it last moved (`{"focus": "$prev"}` goes back there).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_focus: Option<String>,
    /// The node the focus moved to (or whose selected row moved), until the scrollers around it
    /// have brought it into view, a list's selected row rather than all of it:
    /// the engine scrolls them from where the next frame laid it out (`Msg::Pan`), then says
    /// it's done (`Msg::Revealed`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal: Option<String>,
    /// Keys typed into an editor whose hosted text is still loading: (entry, editor, key), typed
    /// in order once it lands.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub typed_ahead: Vec<(String, String, String)>,
    /// What keys wait for: host acts with `wait` (`act:<name>`), then the host sources they
    /// refresh (`data:<name>`). While any is here, keys go into `held`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waiting: Vec<String>,
    /// Keys typed while something in `waiting` hadn't landed, typed in order once it all has.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub held: Vec<String>,
    /// Writes that last a while (`for_ms`): what goes, and when on the logical clock.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expiring: Vec<Expiry>,
    /// The UI's tree with every `each` expanded into its instances, when it has one: what the
    /// view draws and focus, keys and binds look in (`tree()`). Worked out after each message;
    /// a push or a patch changes `ui`, never this. Shown in `get --state`, never read back.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub tree: Option<Node>,
    /// What `tree` was expanded from: the UI's revision and each `each`'s bound value. When
    /// neither changed, it isn't expanded again.
    #[serde(skip)]
    pub expanded_from: Option<(u64, Vec<Option<Value>>)>,
    /// Goes up with every push and patch, so derived trees know the UI changed.
    #[serde(skip)]
    pub ui_rev: u64,
}

/// `State::tree`, as a borrow of its fields alone, so the rest of the state can change beside it.
macro_rules! tree {
    ($st:expr) => {
        $st.tree.as_ref().unwrap_or(&$st.ui.root)
    };
}

impl State {
    /// The node with this id in the drawn tree: of several (one per width, each with its own
    /// `when`), the one that shows, so keys and actions act on what the person sees.
    pub fn node(&self, id: &str) -> Option<&Node> {
        let mut all = Vec::new();
        tree!(self).walk(&mut all);
        let mut same = all.into_iter().filter(|n| n.id.as_deref() == Some(id));
        let first = same.next()?;
        if shown(self, first) {
            return Some(first);
        }
        Some(same.find(|n| shown(self, n)).unwrap_or(first))
    }

    /// The tree on screen: the UI's, with each `each` expanded.
    pub fn tree(&self) -> &Node {
        self.tree.as_ref().unwrap_or(&self.ui.root)
    }

    /// A state read back (`get --state`, a test's copy) has what it was, but not what's worked
    /// out from it: the expanded tree and the rows trees and filters show. This works them out
    /// again, so `tree()`, `focus_order` and binds read as they do on the live screen.
    pub fn derive(&mut self) {
        self.expanded_from = None;
        expand(self);
        derive_rows(self, None);
    }
}

/// Whether a node or any under it is an `each`.
fn has_each(n: &Node) -> bool {
    matches!(n.kind, Kind::Each { .. }) || n.children().iter().any(has_each)
}

/// `n` with every `each` under it expanded: a column (or row) of its template, once per item of
/// its bound array.
fn expand_node(st: &State, n: &Node) -> Node {
    match &n.kind {
        Kind::Each { template, row, stack, wrap, cols, gap, justify, items: across } => {
            let items = n.bind.as_deref().and_then(|b| value_of(st, b)).and_then(Value::as_array).cloned().unwrap_or_default();
            let count = items.len();
            let children: Vec<Node> = items.iter().enumerate().map(|(k, it)| expand_node(st, &crate::model::instance(template, it, (k, count)))).collect();
            let (gap, justify, items, wrap) = (gap.clone(), justify.clone(), across.clone(), *wrap);
            let kind = match cols {
                Some(cols) => Kind::Grid { children, cols: cols.clone(), gap, items },
                None if *stack => Kind::Stack { children },
                None if *row => Kind::Row { children, gap, justify, items, wrap },
                None => Kind::Col { children, gap, justify, items, wrap },
            };
            Node { kind, ..n.clone() }
        }
        _ if !has_each(n) => n.clone(),
        _ => {
            let mut out = n.clone();
            if let Some(cs) = out.children_mut() {
                for c in cs.iter_mut() {
                    *c = expand_node(st, c);
                }
            }
            out
        }
    }
}

/// The bound values every `each` in the UI reads (templates included), or None when one's bind
/// depends on its item (a nested `each`), so the inputs can't be told apart without expanding.
fn each_inputs(st: &State) -> Option<Vec<Option<Value>>> {
    fn binds<'a>(n: &'a Node, out: &mut Vec<&'a str>, dynamic: &mut bool) {
        if let Kind::Each { template, .. } = &n.kind {
            match n.bind.as_deref() {
                Some(b) if b.contains("{$item") => *dynamic = true,
                Some(b) => out.push(b),
                None => {}
            }
            binds(template, out, dynamic);
        }
        for c in n.children() {
            binds(c, out, dynamic);
        }
    }
    let (mut out, mut dynamic) = (Vec::new(), false);
    binds(&st.ui.root, &mut out, &mut dynamic);
    (!dynamic).then(|| out.into_iter().map(|b| value_of(st, b).cloned()).collect())
}

/// Work out `State::tree`, unless nothing it's made from changed. True when it changed.
fn expand(st: &mut State) -> bool {
    if !has_each(&st.ui.root) {
        st.expanded_from = None;
        return st.tree.take().is_some();
    }
    let inputs = each_inputs(st);
    if let (Some(now), Some((rev, was))) = (&inputs, &st.expanded_from)
        && *rev == st.ui_rev
        && now == was
    {
        return false;
    }
    st.expanded_from = inputs.map(|i| (st.ui_rev, i));
    let tree = Some(expand_node(st, &st.ui.root));
    if tree == st.tree {
        return false;
    }
    st.tree = tree;
    true
}

/// Part of the store that goes at `at` (ms): the entry `key` when `path` is empty, else the
/// value at that JSON Pointer in it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Expiry {
    pub key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    pub at: u64,
}

/// A run of clicks on one cell of an editor (see `State::clicks`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Clicks {
    pub id: String,
    pub col: u16,
    pub row: u16,
    pub at: u64,
    pub count: u8,
}

/// How soon a click must follow the last to count as a double (or triple) click, in ms.
const MULTI_CLICK_MS: u64 = 400;

/// A message in the corner (`Action::Toast`), shown until `until` on the logical clock.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Toast {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    pub until: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hover {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
}

/// Rows of a list (or tree or table) with a `drop` lifted to go somewhere else in it: the row
/// pressed and every row carried with it (a picked row carries all the picked ones), and the row
/// it would land beside if let go now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drag {
    pub id: String,
    /// The row pressed, by place and by key.
    pub row: usize,
    pub key: String,
    /// Every row carried, by key, in the list's order.
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub over: Option<Over>,
}

/// Where carried rows would land: beside row `row`, on its `side`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Over {
    pub row: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub side: Side,
}

/// Which side of a row something dropped goes: before it (above, or left of it in a grid of
/// rows) or after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Before,
    After,
}

impl Side {
    pub fn name(self) -> &'static str {
        match self {
            Side::Before => "before",
            Side::After => "after",
        }
    }
}

/// Frames and messages over the last measuring window (the runtime reports twice a second).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    /// Frames drawn per second.
    pub fps: f64,
    /// Mean time to build and draw one frame, in milliseconds.
    pub draw_ms: f64,
    /// Slowest frame in the window, in milliseconds.
    pub max_ms: f64,
    /// Messages applied per second (keys, data, pushes, patches).
    pub msgs: f64,
    /// Frames drawn since start.
    pub frames: u64,
}

/// Sources whose names start with `$` are the runtime's own (`$stats`); a push keeps them.
pub fn builtin(name: &str) -> bool {
    name.starts_with('$')
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Local {
    pub selected: usize,
    /// The shown child of a tabs node.
    #[serde(default)]
    pub tab: usize,
    /// An editor's or input's views of the text entries it has shown, by entry: carets,
    /// scroll, size. The text is in the store.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub editors: BTreeMap<String, caretline::View>,
    /// A tree's rows (by key) folded or opened against its default.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub toggled: BTreeSet<String>,
    /// The selected row's key (a list's or tree's `key`, default `id`), so the selection stays
    /// on that row when rows come and go above it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Where the view of a list, tree or table starts, in lines from its first (a line is a row,
    /// or `cols` rows), when the wheel or the trackpad has moved it away from following the
    /// selection; none while it follows. A fraction is part of a line scrolled past, so a window
    /// scrolls smoothly; a terminal shows whole lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,
    /// How tall the box its rows show in is, in cells, as the last frame laid it out (the view
    /// reports it): what `Lines` fits rows into.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rows: u16,
    /// The heights of the `fit` rows it has built, in cells, by row key (see `Lines`); dropped
    /// when its width changes, since what wraps does too.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub heights: BTreeMap<String, f32>,
    /// The cells its rows were laid out across in the last frame (the view reports it), for
    /// `cols` that fit as many as there's room for.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub width: u16,
    /// A `multi` node's picked rows, by key, in the order they were picked; empty while only
    /// the selected row is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub picked: Vec<String>,
    /// Where a range of picked rows starts (a row's key): the row last clicked or ⌘-clicked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// A row asked for by key (`Set { key }`, the `select` action) that isn't there yet: it's
    /// selected when it comes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub want: Option<String>,
    /// The size of the area its parent laid it out in, last frame, for a `when` that reads
    /// `$width` or `$height` (the view reports it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<(u16, u16)>,
    /// How far a scrolling container is scrolled, in cells along it (rows down a column, columns
    /// across a row), fractions too.
    #[serde(default, skip_serializing_if = "is_nothing")]
    pub offset: f32,
    /// A scrolling container's view and what it holds, in cells along it, as the last frame laid
    /// them out (the view reports it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extent: Option<(f32, f32)>,
}

fn is_nothing(n: &f32) -> bool {
    *n == 0.
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

/// A list's (tree's, table's) lines as its view lays them out: rows `cols` to a line, each line as
/// tall as its tallest row in cells (the template's height, or as measured for `fit` rows), and
/// the view's height in cells (`Local::rows`, from the frame). `top` is in lines, fractions too.
/// Following the selection, the wheel, paging, what's in view, what the window builds and the
/// readback's scroll all ask here, so every list scrolls by the same arithmetic.
pub struct Lines<'a> {
    pub count: usize,
    pub cols: usize,
    pub view: f32,
    tall: Tall<'a>,
}

enum Tall<'a> {
    Each(f32),
    /// `fit` rows: each line's height worked out when it's asked for (only those near the view
    /// are, but for a list as long as all its lines), from the heights measured by row key, a
    /// row not yet measured taken as `guess`.
    Measured { node: &'a Node, rows: &'a [Value], heights: Option<&'a BTreeMap<String, f32>>, guess: f32 },
}

/// What a `fit` row's measured height is kept by: its key (`key`, default `id`), else where it is.
pub fn height_key(n: &Node, r: &Value, k: usize) -> String {
    key_of(n, r).unwrap_or_else(|| format!("#{k}"))
}

impl<'a> Lines<'a> {
    pub fn of(st: &'a State, n: &'a Node) -> Lines<'a> {
        let cols = per_line(st, n);
        let rows = items(st, n);
        let count = rows.len().div_ceil(cols);
        let local = n.id.as_deref().and_then(|id| st.local.get(id));
        let view = local.map_or(0., |l| l.rows as f32);
        let per = n.rows().map_or(1, crate::model::Rows::height) as f32;
        let tall = match n.rows().filter(|r| r.fits()) {
            // A row not built yet is as tall as those measured, on average.
            Some(_) => {
                let heights = local.map(|l| &l.heights).filter(|h| !h.is_empty());
                let guess = heights.map_or(per, |h| h.values().sum::<f32>() / h.len() as f32);
                Tall::Measured { node: n, rows, heights, guess }
            }
            None => Tall::Each(per),
        };
        Lines { count, cols, view, tall }
    }

    /// How tall line `line` is, in cells.
    pub fn height(&self, line: usize) -> f32 {
        match &self.tall {
            Tall::Each(h) => *h,
            Tall::Measured { node, rows, heights, guess } => {
                let row = |k: usize| heights.and_then(|h| h.get(&height_key(node, &rows[k], k))).copied().unwrap_or(*guess);
                (line * self.cols..((line + 1) * self.cols).min(rows.len())).map(row).fold(0., f32::max)
            }
        }
        .max(0.05)
    }

    /// How tall all the lines are, in cells: what a `fit` list would take with room.
    pub fn total(&self) -> f32 {
        match &self.tall {
            Tall::Each(h) => h * self.count as f32,
            Tall::Measured { .. } => (0..self.count).map(|l| self.height(l)).sum(),
        }
    }

    /// The view's height, or before any frame has said, enough for a screenful.
    fn room(&self) -> f32 {
        if self.view > 0. { self.view } else { 200. }
    }

    /// How many lines show whole from line `from` (one at least).
    pub fn fitting_from(&self, from: usize) -> usize {
        match &self.tall {
            Tall::Each(h) => ((self.room() / h.max(0.05)).floor() as usize).max(1),
            Tall::Measured { .. } => {
                let (mut sum, mut n) = (0., 0);
                for l in from..self.count {
                    sum += self.height(l);
                    if sum > self.room() + 0.01 {
                        break;
                    }
                    n += 1;
                }
                n.max(1)
            }
        }
    }

    /// The furthest `top` goes: the last line ending at the view's bottom.
    pub fn most(&self) -> f32 {
        match &self.tall {
            Tall::Each(_) => self.count.saturating_sub(self.fitting_from(0)) as f32,
            Tall::Measured { .. } => {
                let mut sum = 0.;
                for l in (0..self.count).rev() {
                    let h = self.height(l);
                    if sum + h > self.room() {
                        return l as f32 + (sum + h - self.room()) / h;
                    }
                    sum += h;
                }
                0.
            }
        }
    }

    /// The `top` that shows line `line` whole at the view's bottom (its start, if it's taller).
    fn follow(&self, line: usize) -> f32 {
        match &self.tall {
            Tall::Each(_) => (line + 1).saturating_sub(self.fitting_from(0)) as f32,
            Tall::Measured { .. } => {
                let (mut sum, mut k) = (0., line + 1);
                while k > 0 {
                    let h = self.height(k - 1);
                    if sum + h > self.room() + 0.01 && k <= line {
                        break;
                    }
                    sum += h;
                    k -= 1;
                }
                k as f32
            }
        }
    }

    /// Where the view starts, in lines: where the wheel left it, else where it keeps the
    /// selection in view, never past the last whole view.
    pub fn offset(&self, local: Option<&Local>) -> f32 {
        let line = local.map_or(0, |l| l.selected) / self.cols.max(1);
        match local.and_then(|l| l.top) {
            Some(top) => top.clamp(0., self.most()),
            None => self.follow(line).min(self.most()),
        }
    }

    /// The lines to build from `top`: the one it's in, to the last the view reaches.
    pub fn shown(&self, top: f32) -> std::ops::Range<usize> {
        let from = (top.floor().max(0.) as usize).min(self.count);
        let mut y = -top.fract() * self.height(from);
        let mut to = from;
        while to < self.count && y < self.room() {
            y += self.height(to);
            to += 1;
        }
        from..to
    }

    /// `top` moved by `cells`, inside the lines, and the cells it couldn't move.
    pub fn moved(&self, top: f32, cells: f32) -> (f32, f32) {
        let most = self.most();
        let (mut line, mut frac, mut left) = (top.floor().max(0.), top.fract().max(0.), cells);
        while left.abs() > 0.0001 {
            let h = self.height(line as usize);
            if left > 0. {
                let rest = (1. - frac) * h;
                if line + frac >= most || line as usize >= self.count {
                    break;
                }
                if left < rest {
                    frac += left / h;
                    left = 0.;
                } else {
                    left -= rest;
                    line += 1.;
                    frac = 0.;
                }
            } else {
                if line + frac <= 0. {
                    break;
                }
                let back = frac * h;
                if -left <= back {
                    frac += left / h;
                    left = 0.;
                } else if line == 0. {
                    left += back;
                    frac = 0.;
                    break;
                } else {
                    left += back;
                    line -= 1.;
                    frac = 1.;
                }
            }
        }
        let to = (line + frac).clamp(0., most);
        // What clamping took back is what's left over too.
        let over = (line + frac - to) * self.height(to.floor() as usize);
        (to, left + over)
    }

    /// Where line `line` starts in a view starting at `top`, in cells from the view's top (less
    /// than nothing above it).
    pub fn y(&self, top: f32, line: usize) -> f32 {
        let from = top.floor().max(0.) as usize;
        let start = -top.fract() * self.height(from);
        match &self.tall {
            Tall::Each(h) => start + (line as f32 - from as f32) * h,
            Tall::Measured { .. } if line >= from => start + (from..line).map(|l| self.height(l)).sum::<f32>(),
            Tall::Measured { .. } => start - (line..from).map(|l| self.height(l)).sum::<f32>(),
        }
    }

    /// Whether line `line` shows whole in a view starting at `top` (a view that follows the
    /// selection always shows it).
    pub fn in_view(&self, top: Option<f32>, line: usize) -> bool {
        let Some(top) = top else { return true };
        let y = self.y(top, line);
        y >= -0.01 && y + self.height(line) <= self.room() + 0.01
    }
}

/// Move the view under the mouse by `lines` cells: a list's or a scrolling container's, and what it
/// can't take, at its end, the next scroller out.
fn pan_from(st: &mut State, id: &str, lines: f32) {
    let Some(n) = st.node(id) else { return };
    let left = if n.rows().is_some() && n.focusable() && st.local.get(id).is_some_and(|l| l.rows > 0) {
        pan(st, id, lines)
    } else if n.scrolls_down().is_some() {
        scroll_by(st, id, lines)
    } else {
        lines
    };
    if left.abs() < 0.001 {
        return;
    }
    if let Some(out) = scrollers_around(st, id).into_iter().find(|s| s != id) {
        let left = scroll_by(st, &out, left);
        if left.abs() >= 0.001 {
            pan_from(st, &out, left);
        }
    }
}

/// Scroll a container by `cells` along it, kept inside what it holds: what it couldn't take.
fn scroll_by(st: &mut State, id: &str, cells: f32) -> f32 {
    let local = st.local.entry(id.to_string()).or_default();
    let Some((view, content)) = local.extent else { return cells };
    let to = (local.offset + cells).clamp(0., (content - view).max(0.));
    let took = to - local.offset;
    local.offset = to;
    cells - took
}

/// The scrolling containers around node `id` in the tree shown, innermost first (itself first
/// if it scrolls).
pub fn scrollers_around(st: &State, id: &str) -> Vec<String> {
    fn find<'a>(st: &State, n: &'a Node, id: &str, path: &mut Vec<&'a Node>) -> bool {
        if !shown(st, n) {
            return false;
        }
        path.push(n);
        if n.id.as_deref() == Some(id) {
            return true;
        }
        let found = match &n.kind {
            Kind::Tabs { .. } => n.page(tab_of(st, n)).is_some_and(|c| find(st, c, id, path)),
            _ => n.children().iter().any(|c| find(st, c, id, path)),
        };
        if !found {
            path.pop();
        }
        found
    }
    let mut path = Vec::new();
    find(st, tree!(st), id, &mut path);
    path.iter().rev().filter(|n| n.scrolls_down().is_some()).filter_map(|n| n.id.clone()).collect()
}

/// Whether `key` pages the scroller around the focus rather than acting on the focus (whose
/// list would page its own selection), so it doesn't bring the focus back into view.
pub fn pages_a_scroller(st: &State, key: &str) -> bool {
    let paging = matches!(resolve(st, key), Some(Action::Named(n)) if n == "page_down" || n == "page_up");
    paging && !st.focus.as_deref().and_then(|f| st.node(f)).is_some_and(|n| n.rows().is_some() || matches!(n.kind, Kind::Table { .. }))
}

/// A page down (or up, `dir` -1): the focused list's selection by the rows it shows, else the
/// innermost scroller around the focus by most of its view.
fn page(st: &mut State, dir: isize) -> Vec<Effect> {
    let Some(focus) = st.focus.clone() else { return vec![] };
    if let Some(n) = st.node(&focus).filter(|n| n.rows().is_some() || matches!(n.kind, Kind::Table { .. })) {
        let lines = Lines::of(st, n);
        let by = lines.fitting_from(lines.offset(st.local.get(&focus)).floor() as usize) * lines.cols;
        return step(st, move |sel, len| if dir > 0 { (sel + by).min(len.saturating_sub(1)) } else { sel.saturating_sub(by) });
    }
    if let Some(s) = scrollers_around(st, &focus).into_iter().next() {
        let view = st.local.get(&s).and_then(|l| l.extent).map_or(0., |(v, _)| v);
        scroll_by(st, &s, dir as f32 * (view - 1.).max(1.));
    }
    vec![]
}

/// Move a list's view by `cells`, from where it is now: the cells it couldn't move. A view that
/// didn't move goes on following the selection.
fn pan(st: &mut State, id: &str, cells: f32) -> f32 {
    let Some(n) = st.node(id) else { return cells };
    let lines = Lines::of(st, n);
    let local = st.local.get(id);
    let at = lines.offset(local);
    let (to, left) = lines.moved(at, cells);
    if to != at || local.is_some_and(|l| l.top.is_some()) {
        st.local.entry(id.to_string()).or_default().top = Some(to);
    }
    left
}

/// How many to a line node `n` lays out: a list's rows or a grid's children by its `cols`, a
/// count, a column each, or as many as fit across the width the last frame laid it out in.
/// Keys, the view and the readback all ask here.
pub fn per_line(st: &State, n: &Node) -> usize {
    let Some(cols) = n.cols() else { return 1 };
    let width = n.id.as_deref().and_then(|id| st.local.get(id)).map_or(0., |l| l.width as f32 * st.caps.cell_w);
    let th = crate::style::Theme::of(st);
    cols.count(width, |l| th.px(l, st.caps.cell_w))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Slot {
    Loading,
    Ready { value: Value },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
pub enum Msg {
    /// Replace the whole UI.
    Push { ui: Ui },
    /// Replace one node by id.
    Patch { id: String, node: Node },
    /// A key, by name (`j`, `tab`, `s-tab`, `c-c`, `enter`).
    Key { key: String },
    /// A source finished fetching.
    Data { name: String, result: Result<Value, String> },
    /// A `run` finished.
    Ran { argv: Vec<String>, result: Result<(), String> },
    /// The host says its source `host` changed: the sources that read it fetch again.
    Changed { host: String },
    /// A host action finished, at the local clock time (`HH:MM:SS`, from the runtime).
    Acted { name: String, result: Result<Value, String>, at: String },
    /// The runtime's measurements.
    Stats { stats: Stats },
    /// A click on a node (a row of it, or a tab title).
    Click { id: String, row: Option<usize>, tab: Option<usize> },
    /// A click on a row with a modifier held: ⌘ (or ⌥) picks it in or out, ⇧ picks the range
    /// from the anchor. On a node that isn't `multi` it's a click.
    Pick { id: String, row: usize, how: Pick },
    /// The mouse moved over a node (a row of it), or off everything.
    Hover { hover: Option<Hover> },
    /// The wheel over a node: rows to move (negative is up).
    Scroll { id: String, by: isize },
    /// The view of a list moved by `lines` (fractions too), as a trackpad moves it; the
    /// selection stays where it is. On a scrolling container it's cells along it. What the
    /// node can't take (a list at its end, a node that doesn't scroll) goes to the scroller
    /// around it.
    Pan { id: String, lines: f32 },
    /// A scrolling container's view and what it holds, in cells along it, as a frame laid them
    /// out.
    Extent { id: String, view: f32, content: f32 },
    /// The focused node is in view in every scroller around it (see `State::reveal`).
    Revealed,
    /// How tall a frame built a list's `fit` rows, in cells, by row key.
    Heights { id: String, heights: Vec<(String, f32)> },
    /// A press on row `row` of list `id` (one with a `drop`) moved far enough to be a drag: the row
    /// lifts, with every picked row if it's one of them.
    Lift { id: String, row: usize },
    /// Lifted rows moved over row `row` of their list, to go on its `side`; or off its rows
    /// (`row` none), where letting go puts them nowhere.
    Carry {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        row: Option<usize>,
        #[serde(default = "after")]
        side: Side,
    },
    /// The button came up on a drag: over a row, the list's `drop` runs; anywhere else (or after
    /// Esc) nothing moves.
    Release,
    /// A press on a part of row `row` of list, tree or table `id`: a button by its path in the
    /// row template (`0.2`; in a table, the column first: `3.0`), or a tree row's fold glyph
    /// (`fold`). The row is selected as a click
    /// selects it, without the list's own `click`, then the button's `click` runs on it
    /// (`{"toggle": "store.loved/{id}"}` loves that row's album), or the row folds or opens.
    Press { id: String, row: usize, part: String },
    /// A click inside an editor's or input's text, at a cell relative to it: focus it and put the
    /// caret there, or with `extend` (Shift held) stretch the selection to it. A second click on
    /// the same cell soon after selects a word, a third the line.
    Point {
        id: String,
        col: u16,
        row: u16,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        extend: bool,
    },
    /// The mouse dragged with the button held, from a click in an editor's text: the selection
    /// stretches to this cell (relative to the editor; the runtime clamps it).
    Drag { id: String, col: u16, row: u16 },
    /// Text pasted into the terminal (bracketed paste): into the focused editor or input.
    Paste { text: String },
    /// A hosted store entry from the host: a string is text, anything else a value. When the
    /// entry is already here (the host says it changed), it's written as an edit by `host`.
    Loaded { key: String, result: Result<Value, String> },
    /// A hosted entry changed at the host: load it again.
    Reload { key: String },
    /// Animation time.
    Tick { now_ms: u64 },
    /// Move a node's view state from outside: a list's or table's `selected` row (by place, or
    /// by its `key`: the row whose key that is, once it's there), a tabs node's `tab`.
    Set {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selected: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tab: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },
    /// Write one store entry. With `if_rev`, refused unless the entry is still at that
    /// revision (someone else wrote in between). The answer comes back as `Effect::Reply`.
    Write {
        key: String,
        #[serde(flatten)]
        write: Write,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        if_rev: Option<u64>,
        by: String,
        /// The write lasts this long (ms, on the screen's clock): a `put` takes its entry away
        /// again, a `merge` the keys it set. A row marked live for three seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        for_ms: Option<u64>,
    },
    /// Add or replace themes (by name), and choose one: a name, or `next` / `prev` to cycle.
    Theme {
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        add: BTreeMap<String, ThemeSpec>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        choose: Option<String>,
    },
    /// Something an agent did over the socket, for the `$log` source (newest first), with the
    /// local clock time it happened at (`HH:MM:SS`, from the runtime).
    Activity { op: String, what: String, at: String },
    /// The cells an editor node got in the last frame (the view reports them; the runtime sends
    /// this when they differ from what the editor has).
    Size { id: String, width: u16, height: u16 },
    /// Replace the layers only.
    Layers { layers: Vec<LayerSpec> },
    /// Move the focus to a node (one that takes it), and nothing else: no click action runs.
    Focus { id: String },
    /// The area a node's parent laid it out in, last frame (the view reports it, for `when`s
    /// that read `$width` and `$height`).
    Area { id: String, width: u16, height: u16 },
    /// Show a message in the corner for `ms` (default 3000), in a `tone`.
    Toast {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tone: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ms: Option<u64>,
    },
    /// The terminal's size in cells, bindable as `$screen` (`width`, `height`).
    Screen { width: u16, height: u16 },
    /// What the terminal can draw.
    Caps { caps: Caps, text_sizing: bool },
    /// A font file a theme names couldn't be loaded by whatever draws, and why.
    Font { file: String, error: String },
    /// Where this screen runs: its binary, its socket, the folder its UI came from. Bindable as
    /// `$self`, with `cli` (a command line that reaches this screen from any terminal), `env` (a
    /// line that sets a terminal up so `fictty`, and the scripts beside the UI, reach it) and `dir`.
    Host { bin: String, socket: String, dir: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Fetch a source (the runtime reads its definition from `ui.data`).
    Fetch(String),
    Run(Vec<String>),
    Load(String),
    Quit,
    /// Put this text on the system clipboard.
    Copy(String),
    /// The answer to the request that sent the message (a write: its new revision, or why not).
    Reply(Value),
    /// A host action, its args filled.
    Host { name: String, args: Value },
    /// Load a hosted store entry from the host (it comes back as `Msg::Loaded`).
    Read(String),
}

fn after() -> Side {
    Side::After
}

/// How a modified click picks rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pick {
    Toggle,
    Range,
}

pub fn update(st: &mut State, msg: Msg) -> Vec<Effect> {
    let checks = matches!(msg, Msg::Push { .. } | Msg::Patch { .. } | Msg::Data { .. } | Msg::Write { .. } | Msg::Theme { .. } | Msg::Key { .. } | Msg::Font { .. });
    let writes = st.store_writes;
    let quiet = matches!(msg, Msg::Tick { .. } | Msg::Hover { .. } | Msg::Lift { .. } | Msg::Carry { .. } | Msg::Stats { .. } | Msg::Activity { .. } | Msg::Caps { .. } | Msg::Size { .. } | Msg::Extent { .. } | Msg::Revealed | Msg::Heights { .. });
    // A source's new value changes only the rows of the nodes that read it.
    let only = match &msg {
        Msg::Data { name, .. } => Some(name.clone()),
        _ => None,
    };
    // Messages from the person or an agent move selections on purpose: note the rows they're
    // on now, before re-deriving rows makes a selection follow its old row instead.
    let moves = matches!(msg, Msg::Key { .. } | Msg::Click { .. } | Msg::Pick { .. } | Msg::Set { .. } | Msg::Scroll { .. } | Msg::Press { .. });
    let was = st.focus.clone();
    let was_row = st.focus.as_deref().and_then(|f| st.local.get(f)).map(|l| l.selected);
    let keyed = matches!(&msg, Msg::Key { key } if !pages_a_scroller(st, key)) || matches!(msg, Msg::Paste { .. });
    let fx = apply(st, msg);
    // A tick that took a lasting write away changed the store like any write.
    let quiet = quiet && st.store_writes == writes;
    // The `each` nodes' instances follow their data.
    if !quiet && expand(st) {
        settle(st);
    }
    if moves {
        remember(st);
    }
    // Rows a tree or a filter shows can change with anything but the quiet messages; when they
    // do, selections are kept inside them again.
    if !quiet && derive_rows(st, only.as_deref()) {
        settle(st);
    }
    // Anything that shows or hides nodes (a write, a source, the screen's size) can leave the
    // focus on one that's gone: it moves, whatever the message.
    if !quiet && let Some(to) = moved_focus(st, &focus_order(st)) {
        st.focus = to;
    }
    // Wherever the focus went, or the row it's on in a list, it's brought into view once the
    // next frame says where it is, and on any key or paste, which acts on it (typing in a field
    // scrolled away brings it back); not where a UI starts it (a screen opens at its top), nor for the
    // wheel, which leaves the focus where it is.
    let row = st.focus.as_deref().and_then(|f| st.local.get(f)).map(|l| l.selected);
    if keyed || (was.is_some() && st.focus != was) || (st.focus == was && row != was_row) {
        st.reveal = st.focus.clone();
    }
    // A selection, a write or a new row can point an editor at an entry it hasn't shown yet.
    fields(st);
    if !quiet {
        let value = picked_summary(st);
        if st.data.get("$picked").is_none_or(|s| !matches!(s, Slot::Ready { value: v } if *v == value)) {
            st.data.insert("$picked".into(), Slot::Ready { value });
        }
    }
    if st.focus != was && was.is_some() {
        st.prev_focus = was;
    }
    let mut fx = fx;
    fx.extend(wanted(st));
    if !quiet {
        fx.extend(refill_args(st));
    }
    if checks {
        st.problems = problems(st);
    }
    if st.store_writes != writes || !st.data.contains_key("$store") {
        let value = store_summary(st);
        st.data.insert("$store".into(), Slot::Ready { value });
    }
    fx
}

fn apply(st: &mut State, msg: Msg) -> Vec<Effect> {
    match msg {
        Msg::Push { ui } => {
            let before = std::mem::take(&mut st.ui.data);
            st.ui = ui;
            st.ui_rev += 1;
            st.version += 1;
            st.status = None;
            // Seeds start entries that don't exist yet; the store is never cleared by a push.
            for (k, v) in &st.ui.store {
                if !st.store.contains_key(k) {
                    st.store.insert(k.clone(), Entry::value(v.clone(), "seed"));
                    st.store_writes += 1;
                }
            }
            if let Some(f) = st.ui.focus.clone() {
                st.focus = Some(f);
            }
            // A source whose definition changed starts over (its args filled after this message).
            st.args.retain(|k, _| before.get(k) == st.ui.data.get(k));
            settle(st);
            // Sources that are new or changed are fetched; one that's the same as before keeps its
            // value, its stream and its clock (`refresh` fetches everything again).
            st.data.retain(|k, _| builtin(k) || st.ui.data.contains_key(k));
            let stale: Vec<String> = st.ui.data.iter().filter(|(k, s)| before.get(*k) != Some(*s) || !st.data.contains_key(*k)).map(|(k, _)| k.clone()).collect();
            stale.into_iter().map(Effect::Fetch).collect()
        }
        Msg::Patch { id, node } => {
            if st.ui.root.replace(&id, &node) {
                st.ui_rev += 1;
                st.status = None;
                settle(st);
            } else {
                st.status = Some(format!("patch: no node with id {id:?}"));
            }
            vec![]
        }
        Msg::Data { name, result } => {
            if st.ui.data.contains_key(&name) {
                let slot = match result {
                    Ok(value) => Slot::Ready { value },
                    Err(message) => Slot::Error { message },
                };
                st.data.insert(name.clone(), slot);
                settle(st);
            }
            st.waiting.retain(|w| *w != format!("data:{name}"));
            release(st)
        }
        Msg::Stats { stats } => {
            // Rounded for showing: `$stats` is for templates.
            let r = |x: f64, d: i32| (x * 10f64.powi(d)).round() / 10f64.powi(d);
            let value = serde_json::json!({"fps": r(stats.fps, 1), "draw_ms": r(stats.draw_ms, 2), "max_ms": r(stats.max_ms, 1),
                                           "msgs": r(stats.msgs, 0), "frames": stats.frames});
            st.data.insert("$stats".into(), Slot::Ready { value });
            st.stats = stats;
            vec![]
        }
        Msg::Ran { argv, result } => {
            st.status = result.err().map(|e| format!("{}: {e}", argv.join(" ")));
            // A failure shows even where the UI hides the status line.
            if let Some(why) = st.status.clone() {
                toast(st, why, Some("neg".into()), Some(6000));
            }
            refresh(st)
        }
        Msg::Changed { host } => st.ui.data.iter().filter(|(_, s)| s.host.as_deref() == Some(&host)).map(|(k, _)| Effect::Fetch(k.clone())).collect(),
        Msg::Acted { name, result, at } => {
            match result {
                Ok(_) => log(st, "host".into(), name.clone(), at),
                Err(e) => {
                    st.status = Some(format!("{name}: {e}"));
                    toast(st, format!("{name}: {e}"), Some("neg".into()), Some(6000));
                    log(st, "refused".into(), format!("{name} ({e})"), at);
                }
            }
            // What the host did may have changed what its sources say.
            let refresh: Vec<String> = st.ui.data.iter().filter(|(_, s)| s.host.is_some()).map(|(k, _)| k.clone()).collect();
            // An act keys wait for: now they wait for what it refreshes.
            if let Some(i) = st.waiting.iter().position(|w| *w == format!("act:{name}")) {
                st.waiting.remove(i);
                // Only sources that will answer (a template with nothing to fill from fetches nothing).
                let answering: Vec<String> = refresh.iter().filter(|k| source_now(st, k).is_some()).map(|k| format!("data:{k}")).collect();
                st.waiting.extend(answering);
            }
            let mut fx: Vec<Effect> = refresh.into_iter().map(Effect::Fetch).collect();
            fx.extend(release(st));
            fx
        }
        Msg::Click { id, .. } | Msg::Pick { id, .. } | Msg::Point { id, .. } | Msg::Scroll { id, .. } | Msg::Press { id, .. } if !reachable(st, &id) => vec![],
        Msg::Press { id, row, part } => {
            let Some(n) = st.node(&id) else { return vec![] };
            let click = if part == "fold" && matches!(n.kind, Kind::Tree { .. }) {
                None
            } else {
                let Some(click) = n.row_part(&part).filter(|b| matches!(b.kind, Kind::Button { .. })).and_then(|b| b.click.clone()) else {
                    return vec![];
                };
                Some(click)
            };
            if row >= items(st, n).len() {
                return vec![];
            }
            let anchor = items(st, n).get(row).and_then(|v| key_of(n, v));
            if n.focusable() {
                st.focus = Some(id.clone());
            }
            let local = st.local.entry(id).or_default();
            local.selected = row;
            local.key = anchor.clone();
            local.picked.clear();
            local.anchor = anchor;
            match click {
                Some(click) => act(st, click),
                None => fold(st, Fold::Toggle),
            }
        }
        Msg::Lift { id, row } => {
            st.drag = None;
            let Some(n) = st.node(&id).filter(|n| n.drop.is_some()) else { return vec![] };
            let Some(key) = row_key(st, n, row) else { return vec![] };
            let picked: Vec<String> = picked(st, &id).into_iter().filter_map(|k| row_key(st, n, k)).collect();
            let keys = if picked.len() > 1 && picked.contains(&key) { picked } else { vec![key.clone()] };
            st.drag = Some(Drag { id, row, key, keys, over: None });
            vec![]
        }
        Msg::Carry { row, side } => {
            let Some(d) = st.drag.as_ref() else { return vec![] };
            let n = st.node(&d.id);
            let over = row.filter(|r| n.is_some_and(|n| *r < items(st, n).len())).map(|r| Over { row: r, key: n.and_then(|n| row_key(st, n, r)), side });
            if let Some(d) = st.drag.as_mut() {
                d.over = over;
            }
            vec![]
        }
        Msg::Release => {
            let Some(d) = st.drag.take() else { return vec![] };
            let (Some(over), Some(on)) = (d.over.clone(), st.node(&d.id).and_then(|n| n.drop.clone())) else { return vec![] };
            dropped(st, d, over, on.on)
        }
        Msg::Pick { id, row, how } => {
            if !st.node(&id).is_some_and(|n| n.multi) {
                return apply(st, Msg::Click { id, row: Some(row), tab: None });
            }
            let click = st.node(&id).and_then(|n| n.click.clone());
            st.focus = Some(id.clone());
            pick(st, &id, row, how);
            match click {
                Some(a) => act(st, a),
                None => vec![],
            }
        }
        Msg::Click { id, row, tab } => {
            let Some(n) = st.node(&id) else { return vec![] };
            // A node with no click of its own (a picture or a text in a clickable row) passes it
            // to the nearest one around it that has one.
            let click = match &n.click {
                Some(c) => Some(c.clone()),
                None if row.is_none() && tab.is_none() && !n.focusable() => {
                    tree!(st).path_to(&id).into_iter().rev().find_map(|a| a.click.clone())
                }
                None => None,
            };
            let anchor = row.and_then(|r| items(st, n).get(r).and_then(|v| key_of(n, v)));
            if n.focusable() {
                st.focus = Some(id.clone());
            }
            let local = st.local.entry(id.clone()).or_default();
            if let Some(r) = row {
                local.selected = r;
                // Its key now, as `step` does: a store write by the click would have the
                // selection follow the row it left.
                local.key = anchor.clone();
                local.picked.clear();
                local.anchor = anchor;
            }
            if let Some(t) = tab {
                local.tab = t;
                settle(st);
                into_tab(st, &id);
            }
            match click {
                Some(a) => act(st, a),
                None => vec![],
            }
        }
        Msg::Hover { hover } => {
            st.hover = hover;
            vec![]
        }
        Msg::Point { id, col, row, extend } => {
            if !st.node(&id).is_some_and(|n| n.field().is_some()) {
                return vec![];
            }
            st.focus = Some(id.clone());
            let now = st.now_ms;
            let count = match &st.clicks {
                Some(c) if !extend && c.id == id && (c.col, c.row) == (col, row) && now.saturating_sub(c.at) <= MULTI_CLICK_MS => c.count % 3 + 1,
                _ => 1,
            };
            st.clicks = Some(Clicks { id: id.clone(), col, row, at: now, count });
            // Where the caret was: a click on a task's box ticks it and leaves the caret there.
            let before = field_view(st, &id).map(|(k, v)| (k, v.selection.clone(), v.scroll));
            let mut fx = on_doc(st, &id, caretline::Msg::Click { col, row, extend }, "person");
            match count {
                2 => {
                    if let Some((_, view)) = field_view(st, &id) {
                        let pos = view.caret();
                        fx.extend(on_doc(st, &id, caretline::Msg::SelectWordAt { pos }, "person"));
                    }
                }
                3 => {
                    use caretline::{By, Dir};
                    fx.extend(on_doc(st, &id, caretline::Msg::Move { dir: Dir::Backward, by: By::LineStart, extend: false }, "person"));
                    fx.extend(on_doc(st, &id, caretline::Msg::Move { dir: Dir::Forward, by: By::LineEnd, extend: true }, "person"));
                }
                _ => {}
            }
            // A click on a link runs what its kind says (Shift: its `s-click`).
            if count == 1
                && let Some((kind, _)) = link_under(st, &id, true)
                && let Some(spec) = st.node(&id).and_then(|n| n.field()).and_then(|f| f.links.iter().find(|l| l.kind == kind).cloned())
                && let Some(a) = if extend { spec.s_click } else { spec.click }
            {
                let mut out = editor_effects(st, fx);
                out.extend(act(st, a));
                return out;
            }
            // A click on a task's box ticks it.
            if count == 1
                && !extend
                && let Some((line, at)) = caret_place(st, &id)
                && let Some((b, _)) = task_line(st, &id, line)
                && (b.saturating_sub(1)..=b + 1).contains(&at)
            {
                toggle_task(st, &id, line);
                // The tick changed one character for one, so where the caret was is still there.
                if let Some((key, sel, scroll)) = before
                    && let Some(v) = st.local.get_mut(&id).and_then(|l| l.editors.get_mut(&key))
                {
                    v.selection = sel;
                    v.scroll = scroll;
                }
            }
            editor_effects(st, fx)
        }
        Msg::Drag { id, col, row } => {
            if st.focus.as_deref() != Some(id.as_str()) {
                return vec![];
            }
            let fx = on_doc(st, &id, caretline::Msg::Drag { col, row }, "person");
            editor_effects(st, fx)
        }
        Msg::Loaded { key, result } => {
            let value = match result {
                Ok(v) => v,
                Err(e) => {
                    st.loading.insert(key, Some(e));
                    return vec![];
                }
            };
            st.loading.remove(&key);
            match (st.store.contains_key(&key), value) {
                (false, v) => {
                    st.store.insert(key.clone(), Entry::value(v, "host"));
                    st.store_writes += 1;
                    fields(st);
                    // What was typed while it loaded, typed now.
                    let (now, later): (Vec<_>, Vec<_>) = std::mem::take(&mut st.typed_ahead).into_iter().partition(|(k, ..)| *k == key);
                    st.typed_ahead = later;
                    let mut fx = Vec::new();
                    for (_, id, k) in now {
                        if matches!(k.as_str(), "tab" | "s-tab") && st.node(&id).and_then(|n| n.field()).is_some_and(|f| f.indents) {
                            indent(st, &id, k == "tab");
                        } else {
                            let e = type_into(st, &id, &k);
                            fx.extend(editor_effects(st, e));
                        }
                    }
                    settle(st);
                    return fx;
                }
                (true, v) => {
                    let same = match (&st.store[&key].body, &v) {
                        (Body::Text { doc }, Value::String(t)) => doc.text == t.as_str(),
                        (Body::Value { value }, v) => value == v,
                        _ => false,
                    };
                    if !same && let Err(e) = write_entry(st, &key, Write::Put { value: v }, None, "host") {
                        st.status = Some(format!("{key}: {e}"));
                    }
                }
            }
            settle(st);
            vec![]
        }
        Msg::Reload { key } => {
            if !hosted(st, &key) {
                return vec![];
            }
            // A page that failed is asked for again; one never asked for waits until it shows.
            let failed = st.loading.get(&key).is_some_and(Option::is_some);
            if failed {
                st.loading.insert(key.clone(), None);
            }
            if st.store.contains_key(&key) || failed { vec![Effect::Read(key)] } else { vec![] }
        }
        Msg::Paste { text } => {
            let Some(id) = st.focus.clone().filter(|f| st.node(f).is_some_and(|n| n.field().is_some())) else { return vec![] };
            let now_ms = st.now_ms;
            on_doc(st, &id, caretline::Msg::Tick { now_ms }, "person");
            let fx = on_doc(st, &id, caretline::Msg::Paste { text: Some(text) }, "person");
            renumber(st, &id, &[]);
            editor_effects(st, fx)
        }
        Msg::Scroll { id, by } if st.node(&id).is_some_and(|n| n.field().is_some()) => {
            on_doc(st, &id, caretline::Msg::Scroll { rows: by as i32 * 3 }, "person");
            vec![]
        }
        Msg::Scroll { id, by } => {
            if !st.node(&id).is_some_and(|n| n.focusable()) {
                return vec![];
            }
            let rows = st.local.get(&id).map(|l| l.rows).unwrap_or(0) as usize;
            st.focus = Some(id.clone());
            if rows == 0 {
                // Not drawn yet (or not a list): the wheel moves the selection.
                return step(st, |sel, len| sel.saturating_add_signed(by).min(len.saturating_sub(1)));
            }
            // The wheel scrolls the view three lines a notch; the selection stays where it is.
            let line = st.node(&id).map_or(1., |n| {
                let lines = Lines::of(st, n);
                lines.height(lines.offset(st.local.get(&id)).floor() as usize)
            });
            pan(st, &id, by as f32 * 3.0 * line);
            vec![]
        }
        Msg::Pan { id, lines } => {
            pan_from(st, &id, lines);
            vec![]
        }
        Msg::Extent { id, view, content } => {
            let local = st.local.entry(id).or_default();
            local.extent = Some((view, content));
            local.offset = local.offset.clamp(0., (content - view).max(0.));
            vec![]
        }
        Msg::Revealed => {
            st.reveal = None;
            vec![]
        }
        Msg::Heights { id, heights } => {
            st.local.entry(id).or_default().heights.extend(heights);
            vec![]
        }
        Msg::Tick { now_ms } => {
            st.now_ms = now_ms;
            st.toasts.retain(|t| t.until > now_ms);
            toasting(st);
            expire(st);
            vec![]
        }
        Msg::Toast { text, tone, ms } => {
            toast(st, text, tone, ms);
            vec![]
        }
        Msg::Focus { id } => {
            if focus_order(st).contains(&id) {
                st.focus = Some(id);
            } else {
                st.status = Some(format!("focus: {id} doesn't take the focus now"));
            }
            vec![]
        }
        Msg::Font { file, error } => {
            st.fonts_failed.insert(file, error);
            vec![]
        }
        Msg::Area { id, width, height } => {
            st.local.entry(id).or_default().area = Some((width, height));
            settle(st);
            vec![]
        }
        Msg::Screen { width, height } => {
            st.data.insert("$screen".into(), Slot::Ready { value: serde_json::json!({"width": width, "height": height}) });
            vec![]
        }
        Msg::Set { id, selected, tab, key } => {
            if st.node(&id).is_none() {
                st.status = Some(format!("set: no node with id {id:?}"));
                return vec![];
            }
            let local = st.local.entry(id).or_default();
            if let Some(r) = selected {
                local.selected = r;
                local.top = None;
                // The row it was on is forgotten, or the selection would follow it back there;
                // the new row's key is remembered after.
                local.key = None;
            }
            if let Some(t) = tab {
                local.tab = t;
            }
            if key.is_some() {
                local.want = key;
            }
            settle(st);
            vec![]
        }
        Msg::Write { key, write, if_rev, by, for_ms } => {
            let lasting = for_ms.map(|ms| (ms, expiry_paths(&write)));
            let reply = match write_entry(st, &key, write, if_rev, &by) {
                Ok(()) => {
                    if let Some((ms, paths)) = lasting {
                        expire_later(st, &key, paths, ms);
                    }
                    // A deleted entry has no revision left to report.
                    match st.store.get(&key) {
                        Some(e) => serde_json::json!({"ok": true, "key": key, "rev": e.rev}),
                        None => serde_json::json!({"ok": true, "key": key, "deleted": true}),
                    }
                }
                Err(status) => {
                    let mut r = serde_json::json!({"ok": false, "status": status});
                    if let Some(e) = st.store.get(&key) {
                        r["current"] = e.read();
                    }
                    r
                }
            };
            settle(st);
            vec![Effect::Reply(reply)]
        }
        Msg::Theme { add, choose } => {
            let first = add.keys().next().cloned();
            st.themes.extend(add);
            if let Some(c) = choose.or(if st.theme.is_none() { first } else { None }) {
                choose_theme(st, &c);
            }
            vec![]
        }
        Msg::Activity { op, what, at } => {
            log(st, op, what, at);
            vec![]
        }
        Msg::Size { id, width, height } => {
            if st.node(&id).is_some_and(|n| n.field().is_some()) {
                on_doc(st, &id, caretline::Msg::resize(width.max(1), height.max(1)), "person");
            } else {
                let local = st.local.entry(id).or_default();
                if local.width != width {
                    local.heights.clear();
                }
                (local.rows, local.width) = (height, width);
            }
            vec![]
        }
        Msg::Layers { layers } => {
            st.ui.layers = layers;
            vec![]
        }
        Msg::Caps { caps, text_sizing } => {
            st.caps = caps;
            st.text_sizing = text_sizing;
            // Bindable as `$caps`, so a UI can say what it's drawing with.
            let mut value = serde_json::to_value(caps).unwrap_or_default();
            value["text_sizing"] = text_sizing.into();
            value["cell_w"] = (caps.cell_w.round() as i64).into();
            value["cell_h"] = (caps.cell_h.round() as i64).into();
            st.data.insert("$caps".into(), Slot::Ready { value });
            vec![]
        }
        Msg::Host { bin, socket, dir } => {
            let q = |s: &str| format!("'{}'", s.replace('\'', r"'\''"));
            let bin_dir = std::path::Path::new(&bin).parent().map(|p| p.display().to_string()).unwrap_or_default();
            let value = serde_json::json!({
                "bin": bin, "socket": socket, "dir": q(&dir),
                "cli": format!("FICTTY_SOCKET={} {}", q(&socket), q(&bin)),
                "env": format!("export PATH={}:{}:\"$PATH\" FICTTY_SOCKET={}", q(&bin_dir), q(&dir), q(&socket)),
            });
            st.data.insert("$self".into(), Slot::Ready { value });
            vec![]
        }
        Msg::Key { key } if !st.waiting.is_empty() => {
            st.held.push(key);
            vec![]
        }
        Msg::Key { key } if key == "esc" && st.drag.is_some() => {
            st.drag = None;
            vec![]
        }
        Msg::Key { key } => {
            if key == "c-t"
                && let Some(id) = editing(st, &key)
                && st.node(&id).and_then(|n| n.field()).is_some_and(|f| f.markdown && !f.readonly)
            {
                tasks_key(st, &id);
                return vec![];
            }
            if let Some(why) = not_ready(st, &key) {
                st.status = Some(why);
                return vec![];
            }
            if let Some(id) = editing(st, &key) {
                // Text still on its way from the host: the key waits for it.
                if let Some(k) = st.node(&id).and_then(|n| doc_key(st, n)).filter(|k| st.loading.get(k) == Some(&None)) {
                    st.typed_ahead.push((k, id, key));
                    return vec![];
                }
                if matches!(key.as_str(), "tab" | "s-tab") {
                    let moved = indent(st, &id, key == "tab");
                    renumber(st, &id, &moved);
                    return vec![];
                }
                if key == "enter" && continue_list(st, &id) {
                    renumber(st, &id, &[]);
                    return vec![];
                }
                // Undo and redo put back a text that was numbered when it was written; renumbering
                // it again would be a new edit, and the next undo would have nowhere to go.
                let history = editor_key(&key).and_then(|k| caretline::keymap(&k)).is_some_and(|m| matches!(m, caretline::Msg::Undo | caretline::Msg::Redo));
                let fx = type_into(st, &id, &key);
                if !history {
                    renumber(st, &id, &[]);
                }
                return editor_effects(st, fx);
            }
            // Chords: a key binding of several keys (`"p h"`) waits for the rest of them.
            let pending = st.chord.take();
            let seq = match &pending {
                Some(c) => format!("{c} {key}"),
                None => key.clone(),
            };
            let next = chord_next(st, &seq);
            if !next.is_empty() {
                st.chord = Some(seq.clone());
                st.data.insert("$keys".into(), Slot::Ready { value: serde_json::json!({"pending": seq, "next": next}) });
                return vec![];
            }
            st.data.insert("$keys".into(), Slot::Ready { value: serde_json::json!({"pending": null, "next": []}) });
            if pending.is_some() {
                // The chord is done, or it went nowhere (and the key with it).
                return match chord_action(st, &seq) {
                    Some(a) => act(st, a),
                    None => vec![],
                };
            }
            match resolve(st, &key) {
                Some(a) => act(st, a),
                None => vec![],
            }
        }
    }
}

/// What a store action's templates read: the selected row's fields, `store` (every entry: a
/// value, or a text's contents) and `$id`, an id no write has used (the sum of all revisions only
/// goes up, and survives a restart with the store).
fn action_context(st: &State) -> Value {
    let mut ctx = match selected_item(st) {
        Some(Value::Object(m)) => Value::Object(m),
        _ => Value::Object(Default::default()),
    };
    let store: serde_json::Map<String, Value> = st
        .store
        .iter()
        .map(|(k, e)| {
            let v = match &e.body {
                Body::Value { value } => value.clone(),
                Body::Text { doc } => doc.text.to_string().into(),
            };
            (k.clone(), v)
        })
        .collect();
    ctx["store"] = Value::Object(store);
    // The UI's sources by name (`{today}`), where the selected row has no field of that name.
    for (name, slot) in &st.data {
        if let (Slot::Ready { value }, Some(m)) = (slot, ctx.as_object_mut())
            && !name.starts_with('$')
            && !m.contains_key(name)
        {
            m.insert(name.clone(), value.clone());
        }
    }
    let next: u64 = st.store.values().map(|e| e.rev).sum::<u64>() + 1;
    ctx["$id"] = format!("{next:x}").into();
    if let Some(Slot::Ready { value }) = st.data.get("$self") {
        ctx["$self"] = value.clone();
    }
    // The link at the focused editor's caret: `{$link}` for a key, `{$match}` for a click.
    if let Some((_, m)) = st.focus.as_deref().and_then(|id| link_at(st, id)) {
        ctx["$link"] = m.clone().into();
        ctx["$match"] = m.into();
    }
    with_drag(st, ctx)
}

/// Rows let go over row `over` of their list: its `drop` runs on the row dropped on, as a click
/// would run on it (`{id}`), with the carried rows as `$drag` and the side as `$drop`. The
/// selection and picks stay with the rows carried, so they follow them to their new place.
fn dropped(st: &mut State, d: Drag, over: Over, on: Action) -> Vec<Effect> {
    let Some(n) = st.node(&d.id) else { return vec![] };
    if over.row >= items(st, n).len() {
        return vec![];
    }
    let focusable = n.focusable();
    let target = over.key.clone();
    let was = st.local.get(&d.id).cloned().unwrap_or_default();
    if focusable {
        st.focus = Some(d.id.clone());
    }
    let local = st.local.entry(d.id.clone()).or_default();
    local.selected = over.row;
    local.key = target;
    local.picked.clear();
    st.drag = Some(Drag { over: Some(over), ..d.clone() });
    let fx = act(st, on);
    st.drag = None;
    let local = st.local.entry(d.id.clone()).or_default();
    local.selected = d.row;
    local.key = Some(d.key.clone());
    local.picked = was.picked;
    local.anchor = was.anchor;
    fx
}

/// `$drag` (the row carried first, its fields and `keys`, every carried row's key) and `$drop`
/// (`side`, `before` or `after`) in an action's context, while rows are dragged.
fn with_drag(st: &State, mut ctx: Value) -> Value {
    let Some(d) = &st.drag else { return ctx };
    let Some(m) = ctx.as_object_mut() else { return ctx };
    let row = st.node(&d.id).and_then(|n| items(st, n).iter().find(|r| key_of(n, r).as_deref() == Some(d.key.as_str())).cloned());
    let mut carried = match row {
        Some(Value::Object(r)) => Value::Object(r),
        _ => serde_json::json!({}),
    };
    carried["key"] = d.key.clone().into();
    carried["keys"] = serde_json::json!(d.keys);
    m.insert("$drag".into(), carried);
    if let Some(o) = &d.over {
        m.insert("$drop".into(), serde_json::json!({"side": o.side.name(), "row": o.row, "key": o.key}));
    }
    ctx
}

/// The link under editor `id`'s caret, if its text has one there: its kind and what it names.
fn link_at(st: &State, id: &str) -> Option<(String, String)> {
    link_under(st, id, false)
}

/// `link_at`, but for a click: a wiki link's brackets aren't the link, only its title is.
fn link_under(st: &State, id: &str, click: bool) -> Option<(String, String)> {
    let f = st.node(id)?.field()?;
    if f.links.is_empty() {
        return None;
    }
    let kinds: Vec<&str> = f.links.iter().map(|l| l.kind.as_str()).collect();
    let (key, view) = field_view(st, id)?;
    let Body::Text { doc } = &st.store.get(&key)?.body else { return None };
    let at = view.caret().min(doc.text.len_chars());
    let line = doc.text.char_to_line(at);
    let col = at - doc.text.line_to_char(line);
    let text = doc.text.line(line).to_string();
    crate::model::links_in(&text, &kinds)
        .into_iter()
        .find(|(a, b, k, _)| {
            let inset = if click && k == "wiki" { 2 } else { 0 };
            (*a + inset..*b - inset).contains(&col)
        })
        .map(|(_, _, k, m)| (k, m))
}

/// A store action's `value` filled from `ctx`: strings are templates. A string that is one
/// placeholder takes the value itself, so a number stays a number and a list a list; text is
/// never guessed into a number (components that add up read numbers from strings).
fn fill_value(v: &Value, ctx: &Value) -> Value {
    match v {
        Value::String(t) if t.contains('{') => {
            let one = t.strip_prefix('{').and_then(|r| r.strip_suffix('}')).filter(|p| !p.contains(['{', '}', '|']));
            if let Some(found) = one.and_then(|p| lookup(ctx, p)) {
                return found.clone();
            }
            Value::String(crate::model::fill(t, ctx))
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| fill_value(x, ctx)).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), fill_value(x, ctx))).collect()),
        other => other.clone(),
    }
}

/// Make a theme active: a name, or `next` / `prev` through the loaded ones in name order.
fn choose_theme(st: &mut State, choice: &str) {
    let names: Vec<&String> = st.themes.keys().collect();
    let at = st.theme.as_ref().and_then(|t| names.iter().position(|n| *n == t));
    let pick = match choice {
        "next" | "prev" if names.is_empty() => None,
        "next" => Some(names[at.map_or(0, |i| (i + 1) % names.len())].clone()),
        "prev" => Some(names[at.map_or(0, |i| (i + names.len() - 1) % names.len())].clone()),
        name if st.themes.contains_key(name) => Some(name.to_string()),
        name => {
            st.status = Some(format!("theme: no theme named {name:?} (loaded: {})", names.iter().map(|n| n.as_str()).collect::<Vec<_>>().join(", ")));
            None
        }
    };
    if pick.is_some() {
        st.theme = pick;
    }
    // Bindable as `$theme`: which one is on, and what else there is.
    let value = serde_json::json!({"name": st.theme, "names": st.themes.keys().collect::<Vec<_>>()});
    st.data.insert("$theme".into(), Slot::Ready { value });
}

/// The parts of an entry a lasting write sets, as JSON Pointers: the whole entry for a put, each
/// value a merge sets (not the ones it removes).
fn expiry_paths(write: &Write) -> Vec<String> {
    fn leaves(v: &Value, at: String, out: &mut Vec<String>) {
        match v {
            Value::Object(m) if !m.is_empty() => {
                for (k, x) in m {
                    leaves(x, format!("{at}/{}", k.replace('~', "~0").replace('/', "~1")), out);
                }
            }
            Value::Null => {}
            _ => out.push(at),
        }
    }
    match write {
        Write::Merge { value } => {
            let mut out = Vec::new();
            leaves(value, String::new(), &mut out);
            out
        }
        _ => vec![String::new()],
    }
}

/// Note that these parts of `key` go `ms` from now; a later lasting write to the same part
/// replaces its time.
fn expire_later(st: &mut State, key: &str, paths: Vec<String>, ms: u64) {
    let at = st.now_ms + ms;
    for path in paths {
        st.expiring.retain(|e| !(e.key == key && e.path == path));
        st.expiring.push(Expiry { key: key.into(), path, at });
    }
}

/// Take away what's due. True when the store changed.
fn expire(st: &mut State) -> bool {
    let now = st.now_ms;
    let (due, keep): (Vec<Expiry>, Vec<Expiry>) = std::mem::take(&mut st.expiring).into_iter().partition(|e| e.at <= now);
    st.expiring = keep;
    let mut changed = false;
    for e in due {
        if e.path.is_empty() {
            if st.store.remove(&e.key).is_some() {
                st.store_writes += 1;
                changed = true;
            }
            continue;
        }
        let (parent, last) = e.path.rsplit_once('/').unwrap_or(("", ""));
        let last = last.replace("~1", "/").replace("~0", "~");
        let gone = match st.store.get_mut(&e.key).map(|x| &mut x.body) {
            Some(Body::Value { value }) => match value.pointer_mut(parent) {
                Some(Value::Object(m)) => m.remove(&last).is_some(),
                _ => false,
            },
            _ => false,
        };
        if gone {
            wrote(st, &e.key, "clock");
            changed = true;
        }
    }
    changed
}

/// How many toasts show at once (the oldest goes first).
const TOASTS: usize = 4;

fn toast(st: &mut State, text: String, tone: Option<String>, ms: Option<u64>) {
    st.toasts.push(Toast { text, tone, until: st.now_ms + ms.unwrap_or(3000) });
    let over = st.toasts.len().saturating_sub(TOASTS);
    st.toasts.drain(..over);
    toasting(st);
}

/// `$toasting`: whether a toast shows, so a UI can hide what a `toasts` node covers.
fn toasting(st: &mut State) {
    let on = !st.toasts.is_empty();
    if !matches!(st.data.get("$toasting"), Some(Slot::Ready { value: Value::Bool(b) }) if *b == on) {
        st.data.insert("$toasting".into(), Slot::Ready { value: Value::Bool(on) });
    }
}

/// How many entries `$log` keeps.
const LOG_LEN: usize = 40;

/// Adds to `$log`: `{at, op, what}`, newest first, `at` as minutes and seconds on the clock.
/// Keys in a row join one entry, so an agent typing a sentence reads as the sentence.
fn log(st: &mut State, op: String, what: String, at: String) {
    let mut entries = match st.data.remove("$log") {
        Some(Slot::Ready { value: Value::Array(a) }) => a,
        _ => Vec::new(),
    };
    let typed = |k: &str| match k {
        "space" => " ".to_string(),
        "enter" => "⏎".to_string(),
        k if k.chars().count() == 1 => k.to_string(),
        k => format!("‹{k}›"),
    };
    match entries.first_mut() {
        Some(last) if op == "key" && last["op"] == "key" => {
            let joined = format!("{}{}", last["what"].as_str().unwrap_or(""), typed(&what));
            let n = joined.chars().count();
            last["what"] = joined.chars().skip(n.saturating_sub(48)).collect::<String>().into();
            last["at"] = at.into();
        }
        _ => {
            let what = if op == "key" { typed(&what) } else { what };
            entries.insert(0, serde_json::json!({"at": at, "op": op, "what": what}));
            entries.truncate(LOG_LEN);
        }
    }
    st.data.insert("$log".into(), Slot::Ready { value: Value::Array(entries) });
}

/// Why the focused field's own `key` can't run yet: its text isn't a number, or it's required and blank.
fn not_ready(st: &State, key: &str) -> Option<String> {
    let n = st.node(st.focus.as_ref()?)?;
    let f = n.field()?;
    if !(f.number || f.required) || !n.keys.contains_key(key) {
        return None;
    }
    let (doc_key, _) = field_view(st, n.id.as_ref()?)?;
    let text = match &st.store.get(&doc_key)?.body {
        Body::Text { doc } => doc.text.to_string(),
        Body::Value { .. } => return None,
    };
    let name = n.title.as_deref().or(n.id.as_deref()).unwrap_or("input");
    if f.required && text.trim().is_empty() {
        return Some(format!("{name}: it needs some text first"));
    }
    (f.number && !crate::model::number_done(&text)).then(|| format!("{name}: {text:?} isn't a number"))
}

/// The focused editor or input that takes this key, if any: everything but Tab, Shift-Tab, Esc
/// and the node's own `keys`. Ctrl-C copies there; quitting is from outside.
fn editing(st: &State, key: &str) -> Option<String> {
    let id = st.focus.as_ref()?;
    let n = st.node(id)?;
    // Esc leaves an editor, unless there's a selection to drop first.
    let leaves = match key {
        "tab" | "s-tab" => !n.field().is_some_and(|f| f.indents),
        "esc" => !field_view(st, id).is_some_and(|(_, v)| {
            let r = v.selection.primary();
            r.anchor != r.head
        }),
        _ => false,
    };
    // A selection is dropped first, even where the node binds Esc itself.
    let selecting = key == "esc" && field_view(st, id).is_some_and(|(_, v)| v.selection.primary().anchor != v.selection.primary().head);
    // A key with Ctrl, Alt or ⌘ that the editor does nothing with goes on to the UI's keys
    // (⌘T while writing), so an app's shortcuts work wherever the focus is.
    let mut modified = false;
    let mut rest = key;
    while let Some((m, r)) = rest.split_once('-').filter(|(m, r)| m.len() == 1 && !r.is_empty()) {
        modified |= matches!(m, "c" | "a" | "d");
        rest = r;
    }
    let unused = modified && key != "c-t" && editor_key(key).and_then(|k| caretline::keymap(&k)).is_none();
    let taken = n.field().is_some() && (!n.keys.contains_key(key) || selecting) && !leaves && !unused;
    taken.then(|| id.clone())
}

/// What an editor asked for, as the screen's effects: its clipboard goes to the system's (OSC 52),
/// its notices ("copied 3 chars", "nothing selected") to a short toast.
fn editor_effects(st: &mut State, fx: Vec<caretline::Effect>) -> Vec<Effect> {
    let mut out = Vec::new();
    for e in fx {
        match e {
            caretline::Effect::ClipboardSet { text } => out.push(Effect::Copy(text)),
            caretline::Effect::Notice { text } => toast(st, text, None, Some(1500)),
            _ => {}
        }
    }
    out
}

/// A key into an editor: the clock first (typing groups into undo steps by time), then the key
/// through caretline's keymap. Its effects come back for `editor_effects`.
fn type_into(st: &mut State, id: &str, key: &str) -> Vec<caretline::Effect> {
    let Some(msg) = editor_key(key).and_then(|k| caretline::keymap(&k)) else { return vec![] };
    let now_ms = st.now_ms;
    on_doc(st, id, caretline::Msg::Tick { now_ms }, "person");
    on_doc(st, id, msg, "person")
}

/// Tab in an editor that indents: the lines of the selection, or a list item, two spaces in (or
/// out, with `inward` false, as far as two spaces go); elsewhere Tab types two spaces, unless the
/// editor indents lines only (`indent-lines`). One edit
/// by the person, one undo step, every caret carried along.
fn indent(st: &mut State, id: &str, inward: bool) -> Vec<usize> {
    let Some((key, view)) = field_view(st, id) else { return vec![] };
    let r = view.selection.primary();
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return vec![] };
    let text = &doc.text;
    let (lo, hi) = (r.anchor.min(r.head).min(text.len_chars()), r.anchor.max(r.head).min(text.len_chars()));
    let first = text.char_to_line(lo);
    let mut last = text.char_to_line(hi);
    // A selection that ends at a line's start doesn't take that line.
    if hi > lo && last > first && hi == text.line_to_char(last) {
        last -= 1;
    }
    let line = |l: usize| text.line(l).to_string();
    let listy = |t: &str| {
        let t = t.trim_start();
        t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ") || {
            let d = t.chars().take_while(char::is_ascii_digit).count();
            d > 0 && t[d..].starts_with(". ")
        }
    };
    let lines_only = st.node(id).and_then(|n| n.field()).is_some_and(|f| f.indent_lines);
    let changes: Vec<(usize, usize, String)> = if inward && lo == hi && !lines_only && !listy(&line(first)) {
        vec![(lo, lo, "  ".into())]
    } else if inward {
        (first..=last).map(|l| text.line_to_char(l)).map(|at| (at, at, "  ".to_string())).collect()
    } else {
        (first..=last)
            .filter_map(|l| {
                let n = line(l).chars().take(2).take_while(|c| *c == ' ').count();
                let at = text.line_to_char(l);
                (n > 0).then(|| (at, at + n, String::new()))
            })
            .collect()
    };
    let moved = if lo == hi && changes.first().is_some_and(|c| c.0 == lo && !c.2.is_empty() && !lines_only && inward && !listy(&line(first))) {
        vec![]
    } else {
        (first..=last).collect()
    };
    edit_lines(st, id, changes);
    moved
}

/// Keep an editor's ordered lists in number after an edit (when it asks for it, with `ordered`),
/// as part of the same undo step. `moved` lines take the style of their new level.
fn renumber(st: &mut State, id: &str, moved: &[usize]) {
    let Some(f) = st.node(id).and_then(|n| n.field()) else { return };
    if f.ordered.is_empty() || f.readonly {
        return;
    }
    renumber_lists(st, id, moved);
}

/// Every ordered list in editor `id` in number, as part of the undo step before it.
fn renumber_lists(st: &mut State, id: &str, moved: &[usize]) {
    let Some(f) = st.node(id).and_then(|n| n.field()) else { return };
    let styles: Vec<crate::lists::Style> = f.ordered.iter().filter_map(|s| crate::lists::Style::parse(s)).collect();
    let Some((key, _)) = field_view(st, id) else { return };
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return };
    let changes = crate::lists::renumber(&doc.text.to_string(), &styles, moved);
    edit_lines_with(st, id, changes, true);
}

/// Edits to the start of lines (an indent, a task's box) by the person, as one undo step, with
/// editor `id`'s carets and selections moving with their text: one at a line's start stays with
/// that line's text, not in front of what was added.
fn edit_lines(st: &mut State, id: &str, changes: Vec<(usize, usize, String)>) {
    edit_lines_with(st, id, changes, false);
}

/// `edit_lines`; with `join`, the edit joins the undo step before it (a renumbering that follows
/// what the person typed).
fn edit_lines_with(st: &mut State, id: &str, changes: Vec<(usize, usize, String)>, join: bool) {
    if changes.is_empty() {
        return;
    }
    let Some((key, view)) = field_view(st, id) else { return };
    let before = view.selection.clone();
    let shift = |x: usize| -> usize {
        let mut d: isize = 0;
        for (f, t, s) in &changes {
            let n = s.chars().count() as isize;
            if x >= *t {
                d += n - (t - f) as isize;
            } else if x > *f {
                d -= (x - f) as isize;
            }
        }
        (x as isize + d).max(0) as usize
    };
    let now_ms = st.now_ms;
    on_doc(st, id, caretline::Msg::Tick { now_ms }, "person");
    on_doc(st, id, caretline::Msg::Edit { changes: changes.clone(), join }, "person");
    if let Some(v) = st.local.get_mut(id).and_then(|l| l.editors.get_mut(&key)) {
        v.selection = before.transform(|mut r| {
            r.anchor = shift(r.anchor);
            r.head = shift(r.head);
            r.old_visual_position = None;
            r
        });
    }
}

/// Enter in a Markdown list item: the next line starts as the next item (`- `, `2. `, an open
/// `[ ] ` after a task, at the same indent), the text after the caret going with it. Enter on an
/// empty item outdents it, or at the left edge ends the list. False when it isn't a list item
/// (or there's a selection), for Enter to do what it always does.
fn continue_list(st: &mut State, id: &str) -> bool {
    let Some(f) = st.node(id).and_then(|n| n.field()) else { return false };
    if !f.markdown || f.readonly {
        return false;
    }
    let Some((key, view)) = field_view(st, id) else { return false };
    let r = view.selection.primary();
    if r.anchor != r.head {
        return false;
    }
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return false };
    let text = &doc.text;
    let at = r.head.min(text.len_chars());
    let l = text.char_to_line(at);
    let start = text.line_to_char(l);
    let line = text.line(l).to_string();
    let line = line.trim_end_matches(['\n', '\r']);
    let chars: Vec<char> = line.chars().collect();
    let indent = chars.iter().take_while(|c| **c == ' ').count();
    let rest: String = chars[indent..].iter().collect();
    let (marker, next) = if let Some(b) = ["- ", "* ", "+ "].iter().find(|b| rest.starts_with(**b)) {
        (b.len(), b.to_string())
    } else {
        let Some(it) = crate::lists::item(line) else { return false };
        let width = it.end - it.start + 1;
        let d = rest.chars().take_while(char::is_ascii_digit).count();
        let next = match rest[..d].parse::<u64>() {
            Ok(n) if d > 0 => format!("{}{} ", n + 1, &rest[d..d + 1]),
            // A letter or roman numeral: the same marker, for renumbering to count on.
            _ => format!("{} ", it.text),
        };
        (width, next)
    };
    let tasky = rest[marker..].starts_with("[ ] ") || rest[marker..].starts_with("[x] ") || rest[marker..].starts_with("[X] ");
    let prefix = indent + marker + if tasky { 4 } else { 0 };
    let col = at - start;
    if col < prefix {
        return false;
    }
    let content_empty = chars.len() == prefix;
    let changes = if content_empty {
        if indent >= 2 {
            vec![(start, start + 2, String::new())]
        } else {
            vec![(start, start + prefix, String::new())]
        }
    } else {
        let task = if tasky { "[ ] " } else { "" };
        vec![(at, at, format!("\n{}{next}{task}", " ".repeat(indent)))]
    };
    edit_lines(st, id, changes);
    true
}

/// Where editor `id`'s caret is: its line, and its column in that line.
fn caret_place(st: &State, id: &str) -> Option<(usize, usize)> {
    let (key, view) = field_view(st, id)?;
    let Body::Text { doc } = &st.store.get(&key)?.body else { return None };
    let at = view.caret().min(doc.text.len_chars());
    let line = doc.text.char_to_line(at);
    Some((line, at - doc.text.line_to_char(line)))
}

/// Line `line` of a Markdown editor's text, when it's a task item: its box's column and whether
/// it's ticked. Never for a read-only editor.
fn task_line(st: &State, id: &str, line: usize) -> Option<(usize, bool)> {
    let f = st.node(id)?.field()?;
    if !f.markdown || f.readonly {
        return None;
    }
    let (key, _) = field_view(st, id)?;
    let Body::Text { doc } = &st.store.get(&key)?.body else { return None };
    if line >= doc.text.len_lines() {
        return None;
    }
    crate::model::task_box(&doc.text.line(line).to_string())
}

/// Ctrl-T in a Markdown editor: each line the caret or selection is on becomes a task (`text` →
/// `- [ ] text`, `- item` → `- [ ] item`), an open task is ticked, and a ticked one is unticked,
/// or with `tasks: "cycle"` made plain text again. One edit by the person, carets kept on their
/// text.
fn tasks_key(st: &mut State, id: &str) {
    let cycle = st.node(id).and_then(|n| n.field()).is_some_and(|f| f.task_cycle);
    let Some((key, view)) = field_view(st, id) else { return };
    let r = view.selection.primary();
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return };
    let text = &doc.text;
    let (lo, hi) = (r.anchor.min(r.head).min(text.len_chars()), r.anchor.max(r.head).min(text.len_chars()));
    let first = text.char_to_line(lo);
    let mut last = text.char_to_line(hi);
    if hi > lo && last > first && hi == text.line_to_char(last) {
        last -= 1;
    }
    let mut changes = Vec::new();
    for l in first..=last {
        let line = text.line(l).to_string();
        let line = line.trim_end_matches(['\n', '\r']);
        let at = text.line_to_char(l);
        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let rest: String = line.chars().skip(indent).collect();
        let marker = if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
            Some(2)
        } else {
            let d = rest.chars().take_while(char::is_ascii_digit).count();
            (d > 0 && rest[d..].starts_with(". ")).then_some(d + 2)
        };
        match crate::model::task_box(line) {
            Some((b, false)) => changes.push((at + b, at + b + 1, "x".to_string())),
            // `- [x] done` back to `done`: the marker, the box and the space after it go.
            Some((b, true)) if cycle => changes.push((at + indent, (at + b + 3).min(at + line.chars().count()), String::new())),
            Some((b, true)) => changes.push((at + b, at + b + 1, " ".to_string())),
            None => match marker {
                Some(m) => changes.push((at + indent + m, at + indent + m, "[ ] ".to_string())),
                None => changes.push((at + indent, at + indent, "- [ ] ".to_string())),
            },
        }
    }
    edit_lines(st, id, changes);
}

/// Keys held while something they wait for landed: typed in order once nothing is left to wait
/// for (a key that starts another wait holds the ones after it again).
fn release(st: &mut State) -> Vec<Effect> {
    if !st.waiting.is_empty() {
        return vec![];
    }
    let mut fx = Vec::new();
    for key in std::mem::take(&mut st.held) {
        fx.extend(update(st, Msg::Key { key }));
    }
    fx
}

/// Tick or untick the task on line `line` of editor `id`: an edit by the person, one undo step.
fn toggle_task(st: &mut State, id: &str, line: usize) {
    let Some((b, done)) = task_line(st, id, line) else { return };
    let Some((key, _)) = field_view(st, id) else { return };
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return };
    let at = doc.text.line_to_char(line) + b;
    let mark = if done { " " } else { "x" };
    let now_ms = st.now_ms;
    on_doc(st, id, caretline::Msg::Tick { now_ms }, "person");
    on_doc(st, id, caretline::Msg::Edit { changes: vec![(at, at + 1, mark.to_string())], join: false }, "person");
}

/// The store entry an editor or input shows: its `doc` template over the bound value, else its
/// `bind`, else its id. None while a `doc` has nothing to fill from (no row selected). An
/// editor over a multi-selection whose entries differ shows `$mixed/<id>`, an empty text of its
/// own, until its first edit makes them the same (see `doc_keys`).
pub fn doc_key(st: &State, n: &Node) -> Option<String> {
    let keys = doc_keys(st, n);
    if keys.len() > 1 && !keys.iter().all(|k| entry_text(st, k) == entry_text(st, &keys[0])) {
        return Some(format!("{MIXED}{}", n.id.as_deref()?));
    }
    lead_key(st, n)
}

/// What an empty field shows: `Multiple values` over picked rows whose entries differ, else its
/// own `placeholder`.
pub fn placeholder(st: &State, n: &Node) -> Option<String> {
    if doc_key(st, n).is_some_and(|k| k.starts_with(MIXED)) {
        return Some("Multiple values".into());
    }
    n.field()?.placeholder.map(String::from)
}

/// What a mixed editor's own text is called: never saved, gone once it's no longer mixed.
pub const MIXED: &str = "$mixed/";

/// Every entry an editor edits: its `doc` filled from each picked row when it's bound (`@id`)
/// to a `multi` node with several picked, else the one it shows. The selected row's comes first.
pub fn doc_keys(st: &State, n: &Node) -> Vec<String> {
    let lead = lead_key(st, n);
    let many = || -> Option<Vec<String>> {
        let doc = n.field()?.doc?;
        let (source, path) = split_bind(n.bind.as_deref()?);
        let id = source.strip_prefix('@')?;
        let list = st.node(id).filter(|l| l.multi)?;
        let picks = picked(st, id);
        if picks.len() < 2 {
            return None;
        }
        let rows = items(st, list);
        let mut keys: Vec<String> = lead.iter().cloned().collect();
        for r in picks {
            let k = crate::model::fill(doc, lookup(rows.get(r)?, path)?);
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        Some(keys)
    };
    many().unwrap_or_else(|| lead.into_iter().collect())
}

/// The bound value of each picked row after the selected one, in `doc_keys`' order, for seeds.
fn picked_rows<'a>(st: &'a State, n: &Node) -> Vec<&'a Value> {
    let Some((source, path)) = n.bind.as_deref().map(split_bind) else { return vec![] };
    let Some(id) = source.strip_prefix('@') else { return vec![] };
    let Some(list) = st.node(id).filter(|l| l.multi) else { return vec![] };
    let lead = selected(st, id);
    let rows = items(st, list);
    picked(st, id).into_iter().filter(|r| *r != lead).filter_map(|r| lookup(rows.get(r)?, path)).collect()
}

/// An entry's text, a string value as text, nothing as empty.
fn entry_text(st: &State, key: &str) -> String {
    match st.store.get(key).map(|e| &e.body) {
        Some(Body::Text { doc }) => doc.text.to_string(),
        Some(Body::Value { value: Value::String(s) }) => s.clone(),
        _ => String::new(),
    }
}

/// The entry an editor shows for its selected row (or its bind, or its id).
fn lead_key(st: &State, n: &Node) -> Option<String> {
    let f = n.field()?;
    match (f.doc, n.bind.as_deref()) {
        (Some(doc), Some(bind)) => Some(crate::model::fill(doc, value_of(st, bind)?)),
        (Some(doc), None) => Some(doc.to_string()),
        (None, Some(bind)) if !bind.starts_with('@') => Some(bind.to_string()),
        _ => n.id.clone(),
    }
}

/// The view editor `id` has of the entry it shows now.
pub fn field_view<'a>(st: &'a State, id: &str) -> Option<(String, &'a caretline::View)> {
    let key = doc_key(st, st.node(id)?)?;
    let v = st.local.get(id)?.editors.get(&key)?;
    Some((key, v))
}

/// Every editor showing `key`, with its view of it taken out of the state (put back by
/// `put_views`).
fn take_views(st: &mut State, key: &str) -> Vec<(String, caretline::View)> {
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let ids: Vec<String> =
        all.iter().filter(|n| n.field().is_some() && doc_key(st, n).as_deref() == Some(key)).filter_map(|n| n.id.clone()).collect();
    ids.into_iter().filter_map(|id| Some((id.clone(), st.local.get_mut(&id)?.editors.remove(key)?))).collect()
}

fn put_views(st: &mut State, key: &str, views: Vec<(String, caretline::View)>) {
    for (id, v) in views {
        st.local.entry(id).or_default().editors.insert(key.to_string(), v);
    }
}

/// A caretline message through editor `id`'s view of its text. Every other editor on the same
/// text follows the change. A change of text is a store write, by `by`. An editor over several
/// rows' entries makes the change in each: the same message through the same view while they're
/// the same, or, while they differ, what its own empty text then holds put into every one.
fn on_doc(st: &mut State, id: &str, msg: caretline::Msg, by: &str) -> Vec<caretline::Effect> {
    let Some(node) = st.node(id) else { return vec![] };
    let keys = doc_keys(st, node);
    if keys.len() < 2 {
        return on_one_doc(st, id, msg, by);
    }
    let Some(key) = doc_key(st, node) else { return vec![] };
    if key.starts_with(MIXED) && matches!(msg, caretline::Msg::Undo | caretline::Msg::Redo) {
        // Entries that differ each step back (or on) through their own history.
        for k in &keys {
            let view = st.local.get(id).and_then(|l| l.editors.get(k)).cloned().unwrap_or_else(|| caretline::View::new(caretline::Viewport { width: 1, height: 1 }));
            mirror_into(st, id, k, msg.clone(), view, by);
        }
        return vec![];
    }
    if key.starts_with(MIXED) {
        let fx = on_one_doc(st, id, msg, by);
        let text = entry_text(st, &key);
        if !text.is_empty() {
            let view = st.local.get(id).and_then(|l| l.editors.get(&key)).cloned();
            for k in &keys {
                let _ = write_entry(st, k, Write::Put { value: Value::String(text.clone()) }, None, by);
            }
            st.store.remove(&key);
            if let Some(l) = st.local.get_mut(id) {
                l.editors.remove(&key);
                if let Some(v) = view {
                    l.editors.insert(keys[0].clone(), v);
                }
            }
        }
        return fx;
    }
    let before = st.local.get(id).and_then(|l| l.editors.get(&key)).cloned();
    let fx = on_one_doc(st, id, msg.clone(), by);
    // Every message, not only edits, goes to every other picked row's entry from where the caret
    // was in this one: a document keeps its own clock and clipboard, and they decide how typing
    // groups into undo steps and what a paste puts in, so the entries stay alike step by step.
    let Some(before) = before else { return fx };
    for k in keys.iter().filter(|k| **k != key) {
        mirror_into(st, id, k, msg.clone(), before.clone(), by);
    }
    fx
}

/// A caretline message into entry `key` through `view` as editor `id`'s view of it; every other
/// editor on it follows, and a change of text is a write by `by`.
fn mirror_into(st: &mut State, id: &str, key: &str, msg: caretline::Msg, view: caretline::View, by: &str) {
    let mut views = take_views(st, key);
    views.retain(|(v, _)| v != id);
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get_mut(key) else {
        put_views(st, key, views);
        return;
    };
    let rev = doc.rev;
    let mut vs: Vec<caretline::View> = views.iter().map(|(_, v)| v.clone()).collect();
    vs.push(view);
    let acting = vs.len() - 1;
    caretline::views::update_doc(doc, &mut vs, acting, msg);
    let mine = vs.pop().expect("pushed");
    let changed = doc.rev != rev;
    for ((_, v), new) in views.iter_mut().zip(vs) {
        *v = new;
    }
    views.push((id.to_string(), mine));
    put_views(st, key, views);
    if changed {
        wrote(st, key, by);
    }
}

/// `on_doc` for the one entry editor `id` shows.
fn on_one_doc(st: &mut State, id: &str, msg: caretline::Msg, by: &str) -> Vec<caretline::Effect> {
    let Some(node) = st.node(id) else { return vec![] };
    let number = node.field().is_some_and(|f| f.number);
    let Some(key) = doc_key(st, node) else { return vec![] };
    let mut fx = Vec::new();
    let mut views = take_views(st, &key);
    let acting = views.iter().position(|(v, _)| v == id);
    if let (Some(acting), Some(Entry { body: Body::Text { doc }, .. })) = (acting, st.store.get_mut(&key)) {
        let before = doc.rev;
        let mut vs: Vec<caretline::View> = views.iter().map(|(_, v)| v.clone()).collect();
        // A number field keeps what it had when an edit would make it something else.
        let undo = number.then(|| (doc.clone(), vs.clone()));
        fx = caretline::views::update_doc(doc, &mut vs, acting, msg);
        // What the editor would say on its own status bar ("copied 3 chars", "nothing selected")
        // becomes a notice.
        if let Some(text) = vs[acting].status.take() {
            fx.push(caretline::Effect::Notice { text });
        }
        if let Some((d, v)) = undo
            && !crate::model::number_so_far(&doc.text.to_string())
        {
            *doc = d;
            vs = v;
        }
        for ((_, v), new) in views.iter_mut().zip(vs) {
            *v = new;
        }
        if doc.rev != before {
            wrote(st, &key, by);
        }
    }
    put_views(st, &key, views);
    fx
}

/// Count a write to `key`: its revision goes up, and it records who.
fn wrote(st: &mut State, key: &str, by: &str) {
    if let Some(e) = st.store.get_mut(key) {
        e.rev += 1;
        e.by = by.into();
    }
    st.store_writes += 1;
}

/// A write from outside (an agent, over the socket). Text is edited, never swapped out: the
/// person's carets and undo history carry on through it, and one undo takes it back.
fn write_entry(st: &mut State, key: &str, write: Write, if_rev: Option<u64>, by: &str) -> Result<(), String> {
    if let (Some(want), Some(e)) = (if_rev, st.store.get(key))
        && e.rev != want
    {
        return Err(format!("stale: {key} is at rev {}, not {want}; read it and try again", e.rev));
    }
    if if_rev.is_some_and(|r| r > 0) && !st.store.contains_key(key) {
        return Err(format!("stale: {key} doesn't exist"));
    }
    if matches!(write, Write::Delete {}) {
        if st.store.remove(key).is_none() {
            return Err(format!("no entry {key}"));
        }
        for l in st.local.values_mut() {
            l.editors.remove(key);
        }
        st.store_writes += 1;
        return Ok(());
    }
    let is_text = matches!(st.store.get(key), Some(Entry { body: Body::Text { .. }, .. }));
    match write {
        Write::Put { value } if !is_text => {
            match st.store.get_mut(key) {
                Some(e) => e.body = Body::Value { value },
                None => {
                    st.store.insert(key.into(), Entry { rev: 0, by: by.into(), body: Body::Value { value } });
                }
            }
            wrote(st, key, by);
        }
        Write::Merge { value } if !is_text => {
            let e = st.store.entry(key.into()).or_insert_with(|| Entry { rev: 0, by: by.into(), body: Body::Value { value: Value::Null } });
            if let Body::Value { value: v } = &mut e.body {
                crate::store::merge(v, &value);
            }
            wrote(st, key, by);
        }
        Write::Put { value: Value::String(text) } => {
            // Only what differs changes, so carets, selections and undo outside it stay put.
            let changes = match &st.store[key].body {
                Body::Text { doc } => caretline::diff::changes(&doc.text.to_string(), &text),
                Body::Value { .. } => vec![(0, 0, text)],
            };
            text_edit(st, key, changes, by)?;
        }
        Write::Edit { changes } if is_text => text_edit(st, key, changes, by)?,
        Write::Put { .. } => return Err(format!("{key} is text: put a string, or send an edit")),
        Write::Merge { .. } => return Err(format!("{key} is text: merge is for values")),
        Write::Edit { .. } => return Err(format!("{key} isn't text: put or merge a value")),
        Write::Delete {} => unreachable!("handled above"),
    }
    Ok(())
}

/// An agent's edit of a text entry: one undo step, made through a view of its own so every
/// editor's carets map through it and none of them jumps.
fn text_edit(st: &mut State, key: &str, changes: Vec<(usize, usize, String)>, by: &str) -> Result<(), String> {
    let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(key) else { return Err(format!("{key} isn't text")) };
    let len = doc.text.len_chars();
    let mut at = 0;
    for (from, to, _) in &changes {
        if from < &at || from > to || *to > len {
            return Err(format!("edit: ranges must be in order, apart and within the text (0..{len})"));
        }
        at = *to;
    }
    let now_ms = st.now_ms;
    let mut views = take_views(st, key);
    if let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get_mut(key) {
        let mut vs: Vec<caretline::View> = views.iter().map(|(_, v)| v.clone()).collect();
        vs.push(caretline::View::new(caretline::Viewport { width: 80, height: 24 }));
        let agent = vs.len() - 1;
        caretline::views::update_doc(doc, &mut vs, agent, caretline::Msg::Tick { now_ms });
        caretline::views::update_doc(doc, &mut vs, agent, caretline::Msg::Edit { changes, join: false });
        vs.pop();
        for ((_, v), new) in views.iter_mut().zip(vs) {
            *v = new;
        }
    }
    put_views(st, key, views);
    wrote(st, key, by);
    Ok(())
}

/// fictty's key names as caretline keys: `a`, `space`, `enter`, `bs`, `del`, arrows, `home`,
/// `end`, `pgup`, `pgdn`, `esc`, with `c-` (Ctrl), `a-` (Alt) and `s-` (Shift) in front.
fn editor_key(name: &str) -> Option<caretline::Key> {
    use caretline::KeyCode as C;
    let mut mods = caretline::Mods::default();
    let mut rest = name;
    while let Some((m, r)) = rest.split_once('-').filter(|(m, r)| m.len() == 1 && !r.is_empty()) {
        match m {
            "c" => mods.ctrl = true,
            "a" => mods.alt = true,
            "s" => mods.shift = true,
            "d" => mods.cmd = true,
            _ => return None,
        }
        rest = r;
    }
    let code = match rest {
        "space" => C::Char(' '),
        "enter" => C::Enter,
        "bs" => C::Backspace,
        "del" => C::Delete,
        "left" => C::Left,
        "right" => C::Right,
        "up" => C::Up,
        "down" => C::Down,
        "home" => C::Home,
        "end" => C::End,
        "pgup" => C::PageUp,
        "pgdn" => C::PageDown,
        "esc" => C::Esc,
        _ => {
            let mut cs = rest.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => C::Char(c),
                _ => return None,
            }
        }
    };
    Some(caretline::Key { code, mods })
}

/// The keys that would continue the chord `seq` (`p` → `h`, `m`), from the focused node's keys and
/// the UI's. Empty when `seq` starts none.
fn chord_next(st: &State, seq: &str) -> Vec<String> {
    let focused = st.focus.as_deref().and_then(|id| st.node(id));
    let prefix = format!("{seq} ");
    let mut next: Vec<String> = focused
        .map(|n| n.keys.keys())
        .into_iter()
        .flatten()
        .chain(st.ui.keys.keys())
        .filter_map(|k| k.strip_prefix(&prefix).and_then(|r| r.split(' ').next()).map(str::to_string))
        .collect();
    next.sort();
    next.dedup();
    next
}

/// The action a whole chord is bound to: the focused node's, then the UI's.
fn chord_action(st: &State, seq: &str) -> Option<Action> {
    let focused = st.focus.as_deref().and_then(|id| st.node(id));
    focused.and_then(|n| n.keys.get(seq)).or_else(|| st.ui.keys.get(seq)).cloned()
}

/// The action for a key: the focused node's own, then the UI's, then the defaults.
fn resolve(st: &State, key: &str) -> Option<Action> {
    let focused = st.focus.as_deref().and_then(|id| st.node(id));
    if let Some(a) = focused.and_then(|n| n.keys.get(key)) {
        return Some(a.clone());
    }
    if let Some(a) = st.ui.keys.get(key) {
        return Some(a.clone());
    }
    // A button presses on Enter and Space.
    if let Some(n) = focused.filter(|n| matches!(n.kind, Kind::Button { .. }) && matches!(key, "enter" | "space")) {
        return n.click.clone();
    }
    // Esc leaves a text field (one with nothing selected; see `editing`).
    if key == "esc" && focused.is_some_and(|n| n.field().is_some()) {
        return Some(Action::Named("leave".into()));
    }
    // Rows in a grid: ←/→ go a row, as ↑/↓ go a line.
    if focused.is_some_and(|n| per_line(st, n) > 1) {
        match key {
            "h" | "left" => return Some(Action::Named("prev_row".into())),
            "l" | "right" => return Some(Action::Named("next_row".into())),
            _ => {}
        }
    }
    // A list that picks several rows: ⇧ with a move extends the range, ⌘A (⌥A) takes them all,
    // and Esc goes back to the selected row alone.
    if let Some(n) = focused.filter(|n| n.multi) {
        let picking = n.id.as_deref().and_then(|id| st.local.get(id)).is_some_and(|l| !l.picked.is_empty());
        match key {
            "s-down" | "J" => return Some(Action::Named("extend_down".into())),
            "s-up" | "K" => return Some(Action::Named("extend_up".into())),
            "d-a" | "a-a" => return Some(Action::Named("pick_all".into())),
            "esc" if picking => return Some(Action::Named("unpick".into())),
            _ => {}
        }
    }
    // A tree folds where a list would switch tabs.
    if focused.is_some_and(|n| matches!(n.kind, Kind::Tree { .. })) {
        match key {
            "h" | "left" => return Some(Action::Named("collapse".into())),
            "l" | "right" => return Some(Action::Named("expand".into())),
            "space" => return Some(Action::Named("fold".into())),
            _ => {}
        }
    }
    let name = match key {
        "q" | "c-c" => "quit",
        "j" | "down" => "down",
        "k" | "up" => "up",
        "g" | "home" => "top",
        "G" | "end" => "bottom",
        "pgdn" => "page_down",
        "pgup" => "page_up",
        "tab" => "focus_next",
        "s-tab" => "focus_prev",
        "h" | "left" => "tab_prev",
        "l" | "right" => "tab_next",
        "r" => "refresh",
        _ => return None,
    };
    Some(Action::Named(name.into()))
}

fn act(st: &mut State, a: Action) -> Vec<Effect> {
    match a {
        Action::Run { run } => {
            let sel = with_drag(st, selected_item(st).unwrap_or(Value::Null));
            vec![Effect::Run(run.iter().map(|s| crate::model::fill(s, &sel)).collect())]
        }
        Action::Load { load } => {
            // `{file}` reads the selected row, so a menu of screens is a list.
            let sel = with_drag(st, selected_item(st).unwrap_or(Value::Null));
            vec![Effect::Load(crate::model::fill(&load, &sel))]
        }
        Action::Focus { focus } => {
            let to = match focus.as_str() {
                "$prev" => st.prev_focus.clone().unwrap_or_default(),
                f => crate::model::fill(f, &action_context(st)),
            };
            if focus_order(st).contains(&to) {
                st.focus = Some(to);
            }
            vec![]
        }
        Action::Select { select, of } if select == "next" || select == "prev" => {
            if st.node(&of).is_none() {
                st.status = Some(format!("select: no node {of}"));
                return vec![];
            }
            if select == "next" {
                step_in(st, &of, |sel, len| (sel + 1).min(len.saturating_sub(1)))
            } else {
                step_in(st, &of, |sel, _| sel.saturating_sub(1))
            }
        }
        Action::Select { select, of } => {
            let key = crate::model::fill(&select, &action_context(st));
            if st.node(&of).is_none() {
                st.status = Some(format!("select: no node {of}"));
                return vec![];
            }
            st.local.entry(of).or_default().want = Some(key);
            settle(st);
            vec![]
        }
        Action::ListStyle { list_style } => {
            let Some(id) = st.focus.clone().filter(|f| st.node(f).is_some_and(|n| n.field().is_some())) else { return vec![] };
            if list_style != "next" {
                st.status = Some(format!("list_style: {list_style:?} isn't known (only \"next\")"));
                return vec![];
            }
            let Some((line, _)) = caret_place(st, &id) else { return vec![] };
            let Some((key, _)) = field_view(st, &id) else { return vec![] };
            let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { return vec![] };
            let changes = crate::lists::cycle_style(&doc.text.to_string(), line);
            edit_lines(st, &id, changes);
            // Bullets turned into numbers right after a numbered list carry it on (as Markdown
            // reads them), so the numbers follow on from it.
            renumber_lists(st, &id, &[]);
            vec![]
        }
        Action::ShowTab { tab, of } => {
            match st.node(&of).map(|n| &n.kind) {
                Some(Kind::Tabs { tabs, .. }) if tab < tabs.len() => {
                    st.local.entry(of.clone()).or_default().tab = tab;
                    settle(st);
                    into_tab(st, &of);
                }
                _ => st.status = Some(format!("tab: {of} has no tab {tab}")),
            }
            vec![]
        }
        Action::Theme { theme } => {
            choose_theme(st, &theme);
            vec![]
        }
        Action::Append { append, value } => {
            let v = fill_value(&value, &action_context(st));
            match st.store.get_mut(&append).map(|e| &mut e.body) {
                Some(Body::Value { value: Value::Array(a) }) => a.push(v),
                None => {
                    st.store.insert(append.clone(), Entry { rev: 0, by: "person".into(), body: Body::Value { value: Value::Array(vec![v]) } });
                }
                Some(_) => {
                    st.status = Some(format!("append: {append} isn't a list"));
                    return vec![];
                }
            }
            wrote(st, &append, "person");
            settle(st);
            vec![]
        }
        Action::Put { put, value, for_ms } => {
            let value = fill_value(&value, &action_context(st));
            let done = write_entry(st, &put, Write::Put { value }, None, "person");
            if let (Ok(()), Some(ms)) = (&done, for_ms) {
                expire_later(st, &put, vec![String::new()], ms);
            }
            match done {
                Err(e) => st.status = Some(format!("put: {e}")),
                // The person put it there: every editor showing it carries on after it.
                Ok(()) if matches!(st.store.get(&put).map(|e| &e.body), Some(Body::Text { .. })) => {
                    let mut all = Vec::new();
                    tree!(st).walk(&mut all);
                    let ids: Vec<String> = all.iter().filter(|n| n.field().is_some() && doc_key(st, n).as_deref() == Some(put.as_str())).filter_map(|n| n.id.clone()).collect();
                    for id in ids {
                        use caretline::{By, Dir};
                        on_doc(st, &id, caretline::Msg::Move { dir: Dir::Forward, by: By::DocEnd, extend: false }, "person");
                    }
                }
                Ok(()) => {}
            }
            settle(st);
            vec![]
        }
        Action::Copy { copy } => {
            let text = crate::model::fill(&copy, &action_context(st));
            vec![Effect::Copy(text)]
        }
        Action::Toast { toast: text, tone, ms } => {
            let text = crate::model::fill(&text, &action_context(st));
            toast(st, text, tone, ms);
            vec![]
        }
        Action::Host { host, args, wait } => {
            let args = fill_value(&args, &action_context(st));
            if wait {
                st.waiting.push(format!("act:{host}"));
            }
            vec![Effect::Host { name: host, args }]
        }
        Action::Remove { remove, matching: Some(want) } => {
            let want = fill_value(&want, &action_context(st));
            let Some(Body::Value { value: Value::Array(rows) }) = st.store.get_mut(&remove).map(|e| &mut e.body) else {
                st.status = Some(format!("remove: {remove} isn't a list"));
                return vec![];
            };
            let fits = |r: &Value| match (&want, r) {
                (Value::Object(w), Value::Object(r)) => w.iter().all(|(k, v)| r.get(k) == Some(v)),
                (w, r) => w == r,
            };
            let before = rows.len();
            rows.retain(|r| !fits(r));
            if rows.len() != before {
                wrote(st, &remove, "person");
                settle(st);
            }
            vec![]
        }
        Action::Remove { remove, matching: None } => {
            let target = row_targets(st).filter(|(key, _)| *key == remove);
            let Some((key, mut ptrs)) = target else {
                st.status = Some(format!("remove: the focused list doesn't show a row of {remove}"));
                return vec![];
            };
            // Every picked row, the last first, so the indexes of the rest hold.
            let at = |p: &String| p.rsplit_once('/').and_then(|(_, i)| i.parse::<usize>().ok()).unwrap_or(0);
            ptrs.sort_by_key(|p| std::cmp::Reverse(at(p)));
            for ptr in &ptrs {
                let (parent, last) = ptr.rsplit_once('/').unwrap_or(("", ""));
                let removed = match st.store.get_mut(&key).map(|e| &mut e.body) {
                    Some(Body::Value { value }) => match (value.pointer_mut(parent), last.parse::<usize>()) {
                        (Some(Value::Array(a)), Ok(i)) if i < a.len() => {
                            a.remove(i);
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                };
                if !removed {
                    st.status = Some(format!("remove: no row at {ptr} in {key}"));
                    return vec![];
                }
            }
            if let Some(l) = st.focus.clone().and_then(|f| st.local.get_mut(&f)) {
                l.picked.clear();
            }
            wrote(st, &key, "person");
            settle(st);
            vec![]
        }
        // `store.zen`: a true-or-false store entry (or a path into one), not a row's field.
        // `store.flag/{id}`: an entry of each picked row of the focused list, named from it, all
        // taking the selected row's new value.
        Action::Toggle { toggle } if toggle.starts_with("store.") && toggle.contains('{') => {
            let template = &toggle["store.".len()..];
            let Some(id) = st.focus.clone() else { return vec![] };
            let Some(n) = st.node(&id) else { return vec![] };
            let keys: Vec<String> = acted_on(st, n).iter().map(|r| crate::model::fill(template, r)).collect();
            let Some(lead) = keys.first().cloned() else { return vec![] };
            let on = |st: &State, k: &str| matches!(st.store.get(k).map(|e| &e.body), Some(Body::Value { value: Value::Bool(true) }));
            let to = !on(st, &lead);
            for k in keys {
                let _ = write_entry(st, &k, Write::Put { value: Value::Bool(to) }, None, "person");
            }
            settle(st);
            vec![]
        }
        Action::Toggle { toggle } if toggle.starts_with("store.") => {
            let path = &toggle["store.".len()..];
            let (key, inner) = split_bind(path);
            let ptr: String = if inner.is_empty() { String::new() } else { inner.split('.').map(|p| format!("/{p}")).collect() };
            let flipped = match st.store.get_mut(key).map(|e| &mut e.body) {
                None if ptr.is_empty() => {
                    st.store.insert(key.to_string(), Entry { rev: 0, by: "person".into(), body: Body::Value { value: Value::Bool(true) } });
                    true
                }
                Some(Body::Value { value }) => match value.pointer_mut(&ptr) {
                    Some(Value::Bool(b)) => {
                        *b = !*b;
                        true
                    }
                    Some(v @ Value::Null) => {
                        *v = Value::Bool(true);
                        true
                    }
                    _ => false,
                },
                _ => false,
            };
            if !flipped {
                st.status = Some(format!("toggle: {path} isn't true or false"));
                return vec![];
            }
            wrote(st, key, "person");
            settle(st);
            vec![]
        }
        Action::Toggle { toggle } => {
            let Some((key, ptrs)) = row_targets(st) else {
                st.status = Some(format!("toggle: the focused list doesn't show a store entry, so {toggle} can't change"));
                return vec![];
            };
            let rel = toggle.replace('.', "/");
            // Every picked row takes the selected row's new value, so a mixed set all flips one way.
            let to = match st.store.get(&key).map(|e| &e.body) {
                Some(Body::Value { value }) => !value.pointer(&format!("{}/{rel}", ptrs[0])).and_then(Value::as_bool).unwrap_or(false),
                _ => true,
            };
            let flipped = match st.store.get_mut(&key).map(|e| &mut e.body) {
                Some(Body::Value { value }) => ptrs.iter().all(|ptr| match value.pointer_mut(&format!("{ptr}/{rel}")) {
                    Some(v @ (Value::Bool(_) | Value::Null)) => {
                        *v = Value::Bool(to);
                        true
                    }
                    Some(_) => false,
                    None => match value.pointer_mut(ptr) {
                        Some(Value::Object(m)) => {
                            m.insert(toggle.clone(), Value::Bool(to));
                            true
                        }
                        _ => false,
                    },
                }),
                _ => false,
            };
            if !flipped {
                st.status = Some(format!("toggle: {toggle} of the selected row isn't true or false"));
                return vec![];
            }
            wrote(st, &key, "person");
            vec![]
        }
        Action::Do { steps } => steps.into_iter().flat_map(|a| act(st, a)).collect(),
        Action::Named(n) => match n.as_str() {
            "quit" => vec![Effect::Quit],
            "refresh" => refresh(st),
            "down" => {
                let by = focused_cols(st);
                step(st, move |sel, len| if sel + by < len { sel + by } else { sel })
            }
            "up" => {
                let by = focused_cols(st);
                step(st, move |sel, _| sel.checked_sub(by).unwrap_or(sel))
            }
            "next_row" => step(st, |sel, len| (sel + 1).min(len.saturating_sub(1))),
            "prev_row" => step(st, |sel, _| sel.saturating_sub(1)),
            "page_down" => page(st, 1),
            "page_up" => page(st, -1),
            "top" => step(st, |_, _| 0),
            "bottom" => step(st, |_, len| len.saturating_sub(1)),
            "tab_next" => switch(st, 1),
            "tab_prev" => switch(st, -1),
            "fold" => fold(st, Fold::Toggle),
            "collapse" => fold(st, Fold::Close),
            "expand" => fold(st, Fold::Open),
            "leave" => leave(st),
            "focus_next" => cycle(st, 1),
            "focus_prev" => cycle(st, -1),
            "extend_down" => {
                let by = focused_cols(st) as isize;
                extend(st, by)
            }
            "extend_up" => {
                let by = focused_cols(st) as isize;
                extend(st, -by)
            }
            "pick_all" => pick_all(st),
            "unpick" => {
                if let Some(l) = st.focus.clone().and_then(|f| st.local.get_mut(&f)) {
                    l.picked.clear();
                }
                vec![]
            }
            other => {
                st.status = Some(format!("unknown action {other:?}"));
                vec![]
            }
        },
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fold {
    Toggle,
    Open,
    Close,
}

/// Fold or unfold the focused tree's selected row. Closing a row that's closed (or has nothing
/// under it) goes to its parent; opening one that's open goes to its first child.
fn fold(st: &mut State, how: Fold) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    if !st.node(&id).is_some_and(|n| matches!(n.kind, Kind::Tree { .. })) {
        return vec![];
    }
    let rows = st.rows.get(&id).map(Vec::as_slice).unwrap_or(&[]);
    let at = selected(st, &id);
    let Some(row) = rows.get(at) else { return vec![] };
    let kids = row["$kids"].as_u64().unwrap_or(0) > 0;
    let open = row["$open"].as_bool().unwrap_or(false);
    let depth = row["$depth"].as_u64().unwrap_or(0);
    let key = row["$key"].as_str().unwrap_or_default().to_string();
    let flip = match how {
        Fold::Toggle => kids,
        Fold::Open => kids && !open,
        Fold::Close => kids && open,
    };
    let local = st.local.entry(id.clone()).or_default();
    if flip {
        if !local.toggled.remove(&key) {
            local.toggled.insert(key);
        }
    } else if how == Fold::Close && depth > 0 {
        let parent = rows[..at].iter().rposition(|r| r["$depth"].as_u64() == Some(depth - 1));
        local.selected = parent.unwrap_or(at);
    } else if how == Fold::Open && kids && open {
        local.selected = at + 1;
    }
    vec![]
}

/// The focused list's (or tree's) selected row as a place in the store: the entry its bind names,
/// and a JSON Pointer to the row in that entry's value. None unless the node shows a store entry.
/// The store entry the focused list shows and where its picked rows sit in it (JSON pointers),
/// the selected row first: the rows a store action on a row changes.
fn row_targets(st: &State) -> Option<(String, Vec<String>)> {
    let id = st.focus.as_ref()?;
    let n = st.node(id)?;
    let bind = n.bind.as_deref().filter(|b| !b.starts_with('@'))?;
    let (key, path) = split_bind(bind);
    if !matches!(st.store.get(key)?.body, Body::Value { .. }) {
        return None;
    }
    let rows = items(st, n);
    let lead = selected(st, id);
    let prefix: String = if path.is_empty() { String::new() } else { path.split('.').map(|p| format!("/{p}")).collect() };
    let ptr = |at: usize| -> Option<String> {
        let inner = match rows.get(at)?.get("$ptr").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => format!("/{at}"),
        };
        Some(format!("{prefix}{inner}"))
    };
    let mut ptrs = vec![ptr(lead)?];
    ptrs.extend(picked(st, id).into_iter().filter(|r| *r != lead).filter_map(ptr));
    Some((key.to_string(), ptrs))
}

/// Row `k` of node `n`'s key (its `key` field, default `id`).
pub fn row_key(st: &State, n: &Node, k: usize) -> Option<String> {
    key_of(n, items(st, n).get(k)?)
}

/// The rows of `id` that are picked, by index, in order: the picked keys still there, or the
/// selected row alone.
pub fn picked(st: &State, id: &str) -> Vec<usize> {
    let sel = selected(st, id);
    let Some(n) = st.node(id) else { return vec![sel] };
    let Some(keys) = st.local.get(id).map(|l| &l.picked).filter(|k| n.multi && !k.is_empty()) else { return vec![sel] };
    let out: Vec<usize> = items(st, n).iter().enumerate().filter(|(_, r)| key_of(n, r).is_some_and(|k| keys.contains(&k))).map(|(i, _)| i).collect();
    if out.is_empty() { vec![sel] } else { out }
}

/// Pick row `row` of `id` in or out (`Toggle`), or the range from the anchor to it (`Range`).
/// The row becomes the selected one; a row picked out hands that to the nearest still picked.
fn pick(st: &mut State, id: &str, row: usize, how: Pick) {
    let Some(n) = st.node(id) else { return };
    let keys: Vec<Option<String>> = items(st, n).iter().map(|r| key_of(n, r)).collect();
    let Some(Some(key)) = keys.get(row).cloned() else { return };
    let now = picked(st, id);
    let lead = selected(st, id);
    let anchor = st.local.get(id).and_then(|l| l.anchor.clone()).or_else(|| keys.get(lead).cloned().flatten());
    let (set, to): (Vec<usize>, usize) = match how {
        Pick::Toggle if now.contains(&row) && now.len() > 1 => {
            let rest: Vec<usize> = now.into_iter().filter(|r| *r != row).collect();
            let near = *rest.iter().min_by_key(|r| r.abs_diff(row)).expect("one is left");
            (rest, near)
        }
        Pick::Toggle if now.contains(&row) => (now, row),
        Pick::Toggle => {
            let mut set = now;
            set.push(row);
            set.sort_unstable();
            (set, row)
        }
        Pick::Range => {
            let from = anchor.as_ref().and_then(|a| keys.iter().position(|k| k.as_ref() == Some(a))).unwrap_or(lead);
            ((from.min(row)..=from.max(row)).collect(), row)
        }
    };
    let lines = Lines::of(st, n);
    let shows = lines.in_view(st.local.get(id).and_then(|l| l.top), to / lines.cols.max(1));
    let local = st.local.entry(id.to_string()).or_default();
    local.picked = if set.len() > 1 { set.iter().filter_map(|r| keys.get(*r).cloned().flatten()).collect() } else { vec![] };
    local.anchor = if how == Pick::Toggle { Some(key) } else { anchor };
    local.selected = to;
    local.key = keys.get(to).cloned().flatten();
    if !shows {
        local.top = None;
    }
}

/// How many rows to a line the focused node lays out.
fn focused_cols(st: &State) -> usize {
    st.focus.as_deref().and_then(|f| st.node(f)).map_or(1, |n| per_line(st, n))
}

/// ⇧↓ and ⇧↑ on a `multi` list: the range from the anchor grows or shrinks by a row.
fn extend(st: &mut State, dir: isize) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    let Some(n) = st.node(&id) else { return vec![] };
    let len = items(st, n).len();
    let at = selected(st, &id);
    let Some(to) = at.checked_add_signed(dir).filter(|t| *t < len) else { return vec![] };
    pick(st, &id, to, Pick::Range);
    vec![]
}

/// ⌘A on a `multi` list: every row, the selected one staying where it is.
fn pick_all(st: &mut State) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    let Some(n) = st.node(&id) else { return vec![] };
    let keys: Vec<String> = items(st, n).iter().filter(|r| !n.skips(r)).filter_map(|r| key_of(n, r)).collect();
    let local = st.local.entry(id).or_default();
    local.picked = if keys.len() > 1 { keys } else { vec![] };
    vec![]
}

/// `$picked`: each `multi` node's picked rows, `{"count", "keys", "rows"}`, by node id.
fn picked_summary(st: &State) -> Value {
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let mut out = serde_json::Map::new();
    for n in all.into_iter().filter(|n| n.multi) {
        let Some(id) = n.id.as_deref() else { continue };
        let rows = items(st, n);
        let at: Vec<&Value> = picked(st, id).into_iter().filter_map(|r| rows.get(r)).collect();
        let keys: Vec<Value> = at.iter().filter_map(|r| key_of(n, r)).map(Value::String).collect();
        // Where the selected row is among them all, for "6 of 47".
        let index = if rows.is_empty() { 0 } else { selected(st, id) + 1 };
        out.insert(id.to_string(), serde_json::json!({"count": at.len(), "keys": keys, "rows": at, "index": index, "total": rows.len()}));
    }
    Value::Object(out)
}

/// The text a filter reads: a text entry's contents, or a bound string.
fn query_text(st: &State, bind: &str) -> String {
    match st.store.get(bind).map(|e| &e.body) {
        Some(Body::Text { doc }) => doc.text.to_string(),
        _ => crate::model::scalar(value_of(st, bind)),
    }
}

/// Whether every term is somewhere in the row's values (its text, numbers; not its nested rows).
fn row_matches(r: &Value, terms: &[String], nest: Option<&str>) -> bool {
    fn text(v: &Value, nest: Option<&str>, out: &mut String) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    if Some(k.as_str()) != nest {
                        text(x, None, out);
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| text(x, None, out)),
            Value::String(s) => {
                out.push_str(&s.to_lowercase());
                out.push(' ');
            }
            Value::Number(n) => {
                out.push_str(&n.to_string());
                out.push(' ');
            }
            _ => {}
        }
    }
    let mut hay = String::new();
    text(r, nest, &mut hay);
    terms.iter().all(|t| hay.contains(t.as_str()))
}

/// A row as a tree or a filter shows it: the row (an object; anything else under `value`), its
/// nested rows left out, with where it is (`$ptr`) and, in a tree, `$depth`, `$kids`, `$open`
/// and `$key`.
fn shown_row(r: &Value, nest: Option<&str>, ptr: String) -> Value {
    let mut m = match r {
        Value::Object(m) => m.iter().filter(|(k, _)| Some(k.as_str()) != nest).map(|(k, v)| (k.clone(), v.clone())).collect(),
        other => serde_json::Map::from_iter([("value".to_string(), other.clone())]),
    };
    m.insert("$ptr".into(), ptr.into());
    Value::Object(m)
}

struct TreeCx<'a> {
    nest: &'a str,
    key: &'a str,
    open: bool,
    toggled: Option<&'a BTreeSet<String>>,
    terms: &'a [String],
}

/// A tree's shown rows, depth first: every row whose parents are open, or, while filtering, every
/// row that matches or has a match under it (opened to show it). True when any row was shown.
fn flatten(rows: &[Value], at: &str, depth: usize, t: &TreeCx, out: &mut Vec<Value>) -> bool {
    let mut any = false;
    let nest_ptr: String = t.nest.split('.').map(|p| format!("/{p}")).collect();
    for (i, r) in rows.iter().enumerate() {
        let ptr = format!("{at}/{i}");
        let kids = lookup(r, t.nest).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let key = Some(crate::model::scalar(lookup(r, t.key))).filter(|k| !k.is_empty()).unwrap_or_else(|| ptr.clone());
        let filtering = !t.terms.is_empty();
        let open = filtering || t.open != t.toggled.is_some_and(|s| s.contains(&key));
        let mut under = Vec::new();
        let hit_under = if open || filtering { flatten(kids, &format!("{ptr}{nest_ptr}"), depth + 1, t, &mut under) } else { false };
        if filtering && !hit_under && !row_matches(r, t.terms, Some(t.nest)) {
            continue;
        }
        let mut row = shown_row(r, Some(t.nest), ptr);
        row["$depth"] = depth.into();
        row["$kids"] = kids.len().into();
        row["$open"] = (open && !kids.is_empty()).into();
        row["$key"] = key.into();
        out.push(row);
        if open {
            out.extend(under);
        }
        any = true;
    }
    any
}

/// Work out `State::rows`: each tree's shown rows, and each filtered list's or table's matches.
/// With `only`, just the nodes that read that source (by their bind or their filter). True when
/// any changed.
fn derive_rows(st: &mut State, only: Option<&str>) -> bool {
    let mut old = std::mem::take(&mut st.rows);
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let mut out = BTreeMap::new();
    let mut changed = false;
    let reads = |n: &Node, name: &str| [n.bind.as_deref(), n.filter()].into_iter().flatten().any(|b| split_bind(b).0 == name);
    for n in all {
        let Some(id) = &n.id else { continue };
        if let Some(name) = only
            && !reads(n, name)
        {
            if let Some(rows) = old.remove(id) {
                out.insert(id.clone(), rows);
            }
            continue;
        }
        let query = n.filter().map(|f| query_text(st, f)).unwrap_or_default();
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let rows = match &n.kind {
            Kind::Tree { nest, open, .. } => {
                let t = TreeCx {
                    nest: nest.as_deref().unwrap_or("children"),
                    key: n.row_key(),
                    open: *open,
                    toggled: st.local.get(id).map(|l| &l.toggled),
                    terms: &terms,
                };
                let mut rows = Vec::new();
                flatten(raw_items(st, n), "", 0, &t, &mut rows);
                Some(rows)
            }
            _ if !terms.is_empty() => {
                Some(raw_items(st, n).iter().enumerate().filter(|(_, r)| row_matches(r, &terms, None)).map(|(i, r)| shown_row(r, None, format!("/{i}"))).collect())
            }
            _ => None,
        };
        let before = old.remove(id);
        changed |= before != rows;
        if let Some(rows) = rows {
            out.insert(id.clone(), rows);
        }
    }
    // Nodes that went (or stopped filtering) change the rows too.
    changed |= !old.is_empty();
    st.rows = out;
    changed
}

/// Move the focus into the shown child of tabs node `id` (its first node that takes focus), so
/// keys go to what the person just chose to see.
fn into_tab(st: &mut State, id: &str) {
    let Some(n) = st.node(id) else { return };
    let Some(child) = n.page(tab_of(st, n)) else { return };
    let mut inside = Vec::new();
    child.walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut inside);
    let order = focus_order(st);
    if let Some(f) = inside.into_iter().filter(|n| n.focusable()).filter_map(|n| n.id.clone()).find(|i| order.contains(i)) {
        st.focus = Some(f);
    }
}

/// Show the next or previous child of the focused tabs node.
fn switch(st: &mut State, dir: isize) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    let n = match st.node(&id).map(|n| &n.kind) {
        Some(Kind::Tabs { tabs, .. }) => tabs.len() as isize,
        _ => return vec![],
    };
    let local = st.local.entry(id).or_default();
    local.tab = ((local.tab as isize + dir).rem_euclid(n.max(1))) as usize;
    settle(st);
    vec![]
}

/// Move the focus out of a text field: to the next node that takes focus and isn't one, so plain
/// keys reach the screen again. Stays put when there's none.
fn leave(st: &mut State) -> Vec<Effect> {
    let ids = focus_order(st);
    let at = st.focus.as_ref().and_then(|f| ids.iter().position(|i| i == f)).unwrap_or(0);
    let next = (1..ids.len()).map(|k| &ids[(at + k) % ids.len()]).find(|id| st.node(id).is_some_and(|n| n.field().is_none()));
    if let Some(id) = next.cloned() {
        st.focus = Some(id);
    }
    vec![]
}

fn cycle(st: &mut State, dir: isize) -> Vec<Effect> {
    let ids = focus_order(st);
    if ids.is_empty() {
        return vec![];
    }
    let at = st.focus.as_ref().and_then(|f| ids.iter().position(|i| i == f)).unwrap_or(0) as isize;
    let n = ids.len() as isize;
    st.focus = Some(ids[((at + dir).rem_euclid(n)) as usize].clone());
    vec![]
}

fn step(st: &mut State, f: impl Fn(usize, usize) -> usize) -> Vec<Effect> {
    let Some(id) = st.focus.clone() else { return vec![] };
    step_in(st, &id, f)
}

/// Move list `id`'s selection to `f(at, len)`, past rows that can't be selected, wherever the
/// focus is.
fn step_in(st: &mut State, id: &str, f: impl Fn(usize, usize) -> usize) -> Vec<Effect> {
    let id = id.to_string();
    let Some(n) = st.node(&id) else { return vec![] };
    let rows = items(st, n);
    let at = st.local.get(&id).map(|l| l.selected).unwrap_or(0);
    let mut to = f(at, rows.len());
    // Past rows that can't be selected (section headers), in the direction of travel.
    let dir: isize = if to >= at { 1 } else { -1 };
    while rows.get(to).is_some_and(|r| n.skips(r)) {
        match to.checked_add_signed(dir).filter(|t| *t < rows.len()) {
            Some(t) => to = t,
            None => {
                to = at;
                break;
            }
        }
    }
    // Its key now, not once the message is done: a store write later in the same `do` would
    // have the selection follow the row it left.
    let key = rows.get(to).and_then(|r| key_of(n, r));
    let lines = Lines::of(st, n);
    let shows = lines.in_view(st.local.get(&id).and_then(|l| l.top), to / lines.cols.max(1));
    let local = st.local.entry(id).or_default();
    local.selected = to;
    local.key = key;
    local.picked.clear();
    local.anchor = None;
    // A view the wheel moved stays put while the selection is in it, and follows it again
    // once it leaves.
    if !shows {
        local.top = None;
    }
    vec![]
}

fn refresh(st: &State) -> Vec<Effect> {
    st.ui.data.keys().map(|k| Effect::Fetch(k.clone())).collect()
}

/// After any change: drop view state for ids that are gone, keep focus on something focusable,
/// and keep each selection inside its list.
/// Where the focus goes when it has to move (its node went or hid): the first node that takes
/// it, past a tab bar to the content when there is any. None when it can stay.
fn moved_focus(st: &State, order: &[String]) -> Option<Option<String>> {
    if st.focus.as_ref().is_some_and(|f| order.contains(f)) {
        return None;
    }
    let tabs = |id: &String| st.node(id).is_some_and(|n| matches!(n.kind, Kind::Tabs { .. }));
    Some(order.iter().find(|id| !tabs(id)).or(order.first()).cloned())
}

fn settle(st: &mut State) {
    let order = focus_order(st);
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let ids: Vec<String> = all.iter().filter_map(|n| n.id.clone()).collect();
    st.local.retain(|k, _| ids.contains(k));
    // A tabs node's starting tab, until something picks another.
    let starts: Vec<(String, usize)> =
        all.iter().filter_map(|n| match (&n.id, &n.kind) { (Some(id), Kind::Tabs { tab: Some(t), .. }) if !st.local.contains_key(id) => Some((id.clone(), *t)), _ => None }).collect();
    let refocus = !starts.is_empty();
    for (id, t) in starts {
        st.local.entry(id).or_default().tab = t;
    }
    let order = if refocus { focus_order(st) } else { order };
    if let Some(to) = moved_focus(st, &order) {
        st.focus = to;
    }
    // A row asked for by key is selected once it's there.
    let wanted: Vec<(String, usize)> = all
        .iter()
        .filter(|n| n.rows().is_some())
        .filter_map(|n| {
            let id = n.id.clone()?;
            let want = st.local.get(&id)?.want.as_ref()?;
            items(st, n).iter().position(|r| key_of(n, r).as_ref() == Some(want)).map(|k| (id, k))
        })
        .collect();
    for (id, k) in wanted {
        let l = st.local.entry(id).or_default();
        l.selected = k;
        l.top = None;
        l.key = l.want.take();
    }
    // A selection follows its row's key, wherever the row is now.
    let moved: Vec<(String, usize)> = all
        .iter()
        .filter(|n| n.focusable() && n.rows().is_some())
        .filter_map(|n| {
            let id = n.id.clone()?;
            let want = st.local.get(&id)?.key.as_ref()?;
            let rows = items(st, n);
            let at = selected(st, &id);
            if rows.get(at).and_then(|r| key_of(n, r)).as_ref() == Some(want) {
                return None;
            }
            rows.iter().position(|r| key_of(n, r).as_ref() == Some(want)).map(|k| (id, k))
        })
        .collect();
    for (id, k) in moved {
        st.local.entry(id).or_default().selected = k;
    }
    let lens: Vec<(String, usize)> =
        all.iter().filter(|n| n.focusable()).map(|n| (n.id.clone().unwrap(), items(st, n).len())).collect();
    for (id, len) in lens {
        if let Some(l) = st.local.get_mut(&id) {
            l.selected = l.selected.min(len.saturating_sub(1));
        }
    }
    // A selection on a row that can't be selected moves to the next one that can.
    let fix: Vec<(String, usize)> = all
        .iter()
        .filter(|n| n.focusable())
        .filter_map(|n| {
            let id = n.id.clone()?;
            let rows = items(st, n);
            let at = st.local.get(&id).map(|l| l.selected).unwrap_or(0);
            if !rows.get(at).is_some_and(|r| n.skips(r)) {
                return None;
            }
            (at..rows.len()).chain(0..at).find(|k| !n.skips(&rows[*k])).map(|k| (id, k))
        })
        .collect();
    for (id, k) in fix {
        st.local.entry(id).or_default().selected = k;
    }
    remember(st);
}

/// A row's key in a list or tree: its `key` path's value (a tree's own `$key`), if it has one.
fn key_of(n: &Node, r: &Value) -> Option<String> {
    let k = match &n.kind {
        Kind::Tree { .. } => lookup(r, "$key"),
        _ => lookup(r, n.row_key()),
    };
    Some(crate::model::scalar(k)).filter(|k| !k.is_empty())
}

/// Note each list's and tree's selected row's key, for `settle` to follow.
fn remember(st: &mut State) {
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let keys: Vec<(String, Option<String>)> = all
        .iter()
        .filter(|n| n.focusable() && n.rows().is_some())
        .filter_map(|n| {
            let id = n.id.clone()?;
            let k = items(st, n).get(selected(st, &id)).and_then(|r| key_of(n, r));
            Some((id, k))
        })
        .collect();
    for (id, k) in keys {
        if st.local.get(&id).is_some_and(|l| l.key != k) || (k.is_some() && !st.local.contains_key(&id)) {
            st.local.entry(id).or_default().key = k;
        }
    }
}

/// Fill the templated `args` of every host source, and fetch again the ones whose args changed.
fn refill_args(st: &mut State) -> Vec<Effect> {
    let templated: Vec<(String, Value)> =
        st.ui.data.iter().filter(|(_, s)| s.host.is_some() && s.args.to_string().contains('{')).map(|(k, s)| (k.clone(), s.args.clone())).collect();
    st.args.retain(|k, _| templated.iter().any(|(n, _)| n == k));
    let mut fx = Vec::new();
    for (name, args) in templated {
        let filled = fill_from_state(st, &args);
        if st.args.get(&name) != Some(&filled) {
            let first = !st.args.contains_key(&name);
            st.args.insert(name.clone(), filled);
            // A push fetches every source anyway; this is for args that moved since.
            if !first {
                fx.push(Effect::Fetch(name));
            }
        }
    }
    fx
}

/// A source's definition as it's fetched now: its `args` filled from the state.
pub fn source_now(st: &State, name: &str) -> Option<crate::model::Source> {
    let mut src = st.ui.data.get(name)?.clone();
    if let Some(a) = st.args.get(name) {
        src.args = a.clone();
    }
    Some(src)
}

/// A value whose strings are templates over the state: `{@list.id}` (a node's selected row),
/// `{store.key}` (a store entry; a text's contents) or `{source.path}`. A string that is one
/// placeholder takes the value with its type.
fn fill_from_state(st: &State, v: &Value) -> Value {
    let read = |path: &str| -> Option<Value> {
        let path = path.trim();
        let key = path.strip_prefix("store.").unwrap_or(path);
        if let Some(Body::Text { doc }) = st.store.get(key).map(|e| &e.body) {
            return Some(Value::String(doc.text.to_string()));
        }
        value_of(st, key).cloned()
    };
    match v {
        Value::String(t) if t.contains('{') => {
            if let Some(p) = t.strip_prefix('{').and_then(|r| r.strip_suffix('}')).filter(|p| !p.contains(['{', '}'])) {
                return read(p).unwrap_or(Value::Null);
            }
            let mut out = String::new();
            let mut rest = t.as_str();
            while let Some(i) = rest.find('{') {
                out.push_str(&rest[..i]);
                let Some(j) = rest[i..].find('}') else {
                    out.push_str(&rest[i..]);
                    rest = "";
                    break;
                };
                out.push_str(&crate::model::scalar(read(&rest[i + 1..i + j]).as_ref()));
                rest = &rest[i + j + 1..];
            }
            out.push_str(rest);
            Value::String(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| fill_from_state(st, x)).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), fill_from_state(st, x))).collect()),
        other => other.clone(),
    }
}

/// Whether the host keeps this store key.
pub fn hosted(st: &State, key: &str) -> bool {
    st.hosted.iter().any(|p| key.starts_with(p.as_str()))
}

/// Hosted entries something on screen needs (an editor's text, a node's bind) that aren't here
/// and haven't been asked for: ask the host for them.
fn wanted(st: &mut State) -> Vec<Effect> {
    if st.hosted.is_empty() {
        return vec![];
    }
    let mut all = Vec::new();
    tree!(st).walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut all);
    let mut keys: Vec<String> = Vec::new();
    for n in all {
        let key = match (n.field(), n.bind.as_deref()) {
            (Some(_), _) => doc_key(st, n),
            (None, Some(b)) if !b.starts_with('@') => Some(split_bind(b).0.to_string()),
            _ => None,
        };
        if let Some(k) = key.filter(|k| hosted(st, k) && !st.store.contains_key(k) && !st.loading.contains_key(k) && !keys.contains(k)) {
            keys.push(k);
        }
    }
    st.loading.extend(keys.iter().map(|k| (k.clone(), None)));
    keys.into_iter().map(Effect::Read).collect()
}

/// Every editor and input shows a text entry in the store: its seed when the entry is new, or a
/// string an agent put there before. Its view of each entry is its own, by id.
fn fields(st: &mut State) {
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    // Every entry each field edits (each picked row's, over a multi-selection), seeded from its
    // own row; then a mixed field's own empty text.
    let mut found: Vec<(String, String, String, bool)> = Vec::new();
    for n in &all {
        let (Some(f), Some(id)) = (n.field(), n.id.clone()) else { continue };
        let keys = doc_keys(st, n);
        let rows = picked_rows(st, n);
        for (i, key) in keys.into_iter().enumerate() {
            let row = if i == 0 { n.bind.as_deref().and_then(|b| value_of(st, b)) } else { rows.get(i - 1).copied() };
            let seed = match row {
                Some(v) if f.doc.is_some() => crate::model::fill(f.seed, v),
                _ => f.seed.to_string(),
            };
            found.push((id.clone(), key, seed, f.line));
        }
    }
    let mixed: Vec<(String, String, bool)> = all
        .iter()
        .filter_map(|n| Some((n.id.clone()?, doc_key(st, n)?, n.field()?.line)))
        .filter(|(_, k, _)| k.starts_with(MIXED))
        .collect();
    // A mixed text no editor shows any more goes.
    let stale: Vec<String> = st.store.keys().filter(|k| k.starts_with(MIXED) && !mixed.iter().any(|(_, m, _)| m == *k)).cloned().collect();
    for k in stale {
        st.store.remove(&k);
        for l in st.local.values_mut() {
            l.editors.remove(&k);
        }
    }
    found.extend(mixed.into_iter().map(|(id, k, line)| (id, k, String::new(), line)));
    // Of editors that show the same new entry, one with a seed seeds it (a preview beside the
    // editor names no text of its own).
    found.sort_by_key(|f| f.2.is_empty());
    for (id, key, seed, line) in found {
        let fresh = |text: &str| {
            let mut e = Entry::text(text, "seed");
            if let Body::Text { doc } = &mut e.body {
                doc.config.single_line = line;
                doc.sanitize();
            }
            e
        };
        match st.store.get(&key).map(|e| &e.body) {
            // The host's: it comes from the host, never from a seed.
            None if hosted(st, &key) => continue,
            None => {
                st.store.insert(key.clone(), fresh(&seed));
                st.store_writes += 1;
            }
            Some(Body::Value { value: Value::String(s) }) => {
                let text = s.clone();
                let e = st.store.get_mut(&key).expect("present");
                *e = Entry { rev: e.rev, by: e.by.clone(), ..fresh(&text) };
            }
            Some(Body::Value { .. }) => {
                st.status = Some(format!("{id}: {key} holds a value, not text"));
                continue;
            }
            Some(Body::Text { .. }) => {}
        }
        let Some(Entry { body: Body::Text { doc }, .. }) = st.store.get(&key) else { continue };
        let (readonly, at_end) = st.node(&id).and_then(|n| n.field()).map_or((false, false), |f| (f.readonly, f.caret_end));
        let len = doc.text.len_chars();
        let view = st.local.entry(id).or_default().editors.entry(key).or_insert_with(|| {
            let mut v = caretline::View::new(caretline::Viewport { width: 1, height: 1 });
            v.config.status_bar = false;
            // The first time this editor shows this entry, the caret starts where it says.
            if at_end {
                v.selection = v.selection.clone().transform(|mut r| {
                    r.anchor = len;
                    r.head = len;
                    r
                });
            }
            v
        });
        view.read_only = readonly;
        view.fit(doc);
    }
}

/// The `$store` source: one row per entry (key, rev, by, what it holds), for a UI to show.
fn store_summary(st: &State) -> Value {
    let rows: Vec<Value> = st
        .store
        .iter()
        .map(|(k, e)| {
            let holds = match &e.body {
                Body::Text { doc } => format!("text, {} chars", doc.text.len_chars()),
                Body::Value { value: Value::String(t) } => format!("text, {} chars", t.chars().count()),
                Body::Value { value: Value::Array(a) } => format!("a list of {}", a.len()),
                Body::Value { value: Value::Object(o) } => format!("an object of {}", o.len()),
                Body::Value { .. } => "a value".to_string(),
            };
            serde_json::json!({"key": k, "rev": e.rev, "by": e.by, "holds": holds})
        })
        .collect();
    Value::Array(rows)
}

/// What's wrong with the UI against the store, the sources and the theme (see `problems`).
pub fn problems(st: &State) -> Vec<String> {
    let mut all = Vec::new();
    tree!(st).walk(&mut all);
    let active = st.theme.as_ref().and_then(|t| st.themes.get(t));
    let mut out = Vec::new();
    // A bind reads the store before the sources, so a source named like an entry is never read.
    for name in st.ui.data.keys().filter(|k| st.store.contains_key(k.as_str())) {
        out.push(format!("{name} is a store entry and a data source: binds read the store's (`fictty delete {name}` to read the source)"));
    }
    let kind = |v: &Value| match v {
        Value::Array(_) => "a list",
        Value::Object(_) => "an object",
        Value::String(_) => "text",
        Value::Number(_) => "a number",
        Value::Bool(_) => "true or false",
        Value::Null => "nothing",
    };
    for n in &all {
        let name = n.id.clone().unwrap_or_else(|| n.kind_name().to_string());
        if let Some(v) = &n.variant
            && active.is_some_and(|t| !t.variants.contains_key(v))
        {
            out.push(format!("{name}: the theme has no variant {v:?}"));
        }
        if let Some(t) = &n.tone
            && !crate::style::Theme::of(st).knows(t)
        {
            out.push(format!("{name}: tone {t:?} isn't a theme token or a colour"));
        }
        if let Some(id) = n.bind.as_deref().and_then(|b| split_bind(b).0.strip_prefix('@'))
            && selects_in_a_loop(st, id)
        {
            out.push(format!("{name}: its bind {:?} comes back round to itself through selected rows, so it reads nothing", n.bind.as_deref().unwrap_or_default()));
        }
        if let Kind::Input { accept: Some(a), .. } = &n.kind
            && a != "number"
        {
            out.push(format!("input {name}: accept {a:?} isn't known (only \"number\")"));
        }
        if let Kind::Table { columns, .. } = &n.kind {
            for c in columns.iter().filter(|c| c.size.as_ref().is_some_and(|s| s.measure() == crate::model::Measure::Fit)) {
                out.push(format!("table {name}: column {:?} can't be `fit` (each line is laid out apart): give it cells, px, a % or a share", c.title));
            }
        }
        if let Kind::Tabs { tabs, children, .. } = &n.kind
            && children.len() > tabs.len()
        {
            out.push(format!("tabs {name}: {} pages for {} tabs, so the last {} never show", children.len(), tabs.len(), children.len() - tabs.len()));
        }
        // `span` is a grid's: anywhere else it would be quietly ignored.
        if !matches!(n.kind, Kind::Grid { .. }) {
            for c in n.children().iter().filter(|c| c.span.is_some()) {
                let who = c.id.clone().unwrap_or_else(|| c.kind_name().to_string());
                out.push(format!("{who}: span takes columns in a grid, and it's in a {}", n.kind_name()));
            }
        }
        let Some(bind) = n.bind.as_deref().filter(|b| !b.starts_with('@')) else { continue };
        let Some(v) = value_of(st, bind) else { continue };
        let wants_list = matches!(n.kind, Kind::List { .. } | Kind::Table { .. } | Kind::Tree { .. } | Kind::Bars { .. } | Kind::Sparkline { .. });
        let literal = !items(st, n).is_empty() && !v.is_array();
        if wants_list && !v.is_array() && !literal {
            out.push(format!("{} {name}: {bind} should be a list, but it's {}", n.kind_name(), kind(v)));
        }
        if let (Kind::Sparkline { field: None, .. }, Value::Array(a)) = (&n.kind, v)
            && a.iter().any(|x| !x.is_number())
        {
            out.push(format!("sparkline {name}: {bind} should be a list of numbers (or set `field`)"));
        }
    }
    out.extend(crate::style::problems(st, &all));
    out
}

/// The focusable nodes on screen, in tree order (hidden tabs and nodes left out). While a modal
/// node shows, only the ones inside it (the last one, if several show).
pub fn focus_order(st: &State) -> Vec<String> {
    let mut all = Vec::new();
    tree!(st).walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut all);
    if let Some(m) = all.iter().rev().find(|n| n.modal) {
        let mut inside = Vec::new();
        m.walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut inside);
        all = inside;
    }
    all.into_iter().filter(|n| n.focusable()).filter_map(|n| n.id.clone()).collect()
}

/// Whether the pointer reaches node `id`: it shows, and while a modal node shows, it's inside it.
fn reachable(st: &State, id: &str) -> bool {
    let mut all = Vec::new();
    tree!(st).walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut all);
    if let Some(m) = all.iter().rev().find(|n| n.modal) {
        let mut inside = Vec::new();
        m.walk_shown(&|n| tab_of(st, n), &|n| shown(st, n), &mut inside);
        return inside.iter().any(|n| n.id.as_deref() == Some(id));
    }
    all.iter().any(|n| n.id.as_deref() == Some(id))
}

/// How long each spinner frame shows (ms).
const SPIN_MS: u64 = 80;

/// A spinner's glyph at `now_ms`, and when (on the clock) the next one shows.
pub fn spinner(now_ms: u64) -> (&'static str, u64) {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    (FRAMES[(now_ms / SPIN_MS % FRAMES.len() as u64) as usize], (now_ms / SPIN_MS + 1) * SPIN_MS)
}

/// Whether a node shows: its `when` holds (see `Node::when`).
pub fn shown(st: &State, n: &Node) -> bool {
    let area = n.id.as_ref().and_then(|id| st.local.get(id)).and_then(|l| l.area);
    shown_in(st, n, area)
}

/// Whether a node shows when its parent lays it out in an area of `size` (for `$width`,
/// `$height`). Not knowing the size yet, a node that asks for it shows. Its `{field}`s are
/// filled from its bound value first, as its `doc` and `src` are (`"when": "remove/{id}"`
/// bound to `@files` reads the selected row's flag).
pub fn shown_in(st: &State, n: &Node, size: Option<(u16, u16)>) -> bool {
    let Some(w) = n.when.as_deref() else { return true };
    let w: std::borrow::Cow<str> = if w.contains('{') { crate::model::fill_raw(w, bound(st, n).unwrap_or(&Value::Null)).into() } else { w.into() };
    if size.is_none() && sized(&w) {
        return true;
    }
    holds_in(st, &w, size)
}

/// Whether node `n` is in its `on` state: its `on` condition holds, `{field}`s filled from its
/// bound value as a `when`'s are.
pub fn is_on(st: &State, n: &Node) -> bool {
    let Some(c) = n.on.as_deref() else { return false };
    let c: std::borrow::Cow<str> = if c.contains('{') { crate::model::fill_raw(c, bound(st, n).unwrap_or(&Value::Null)).into() } else { c.into() };
    holds(st, &c)
}

/// The value node `n`'s templates fill from: its bound value, under `$store` the store entries
/// they name (`{$store.playing}`), so a text can say what it shows and how things stand, and for
/// a list, table or tree `$rows`, how many rows it shows (after its filter: `"TO DO · {$rows}"`).
pub fn scope<'a>(st: &'a State, n: &Node) -> std::borrow::Cow<'a, Value> {
    let v = bound(st, n).unwrap_or(&Value::Null);
    let mut named: Vec<&str> = Vec::new();
    let mut rows = false;
    for t in n.templates() {
        rows |= t.contains("{$rows");
        let mut rest = t;
        while let Some(i) = rest.find("{$store.") {
            rest = &rest[i + "{$store.".len()..];
            let end = rest.find(['}', '|', ':', '.']).unwrap_or(rest.len());
            named.push(&rest[..end]);
        }
    }
    let rows = rows && matches!(n.kind, Kind::List { .. } | Kind::Table { .. } | Kind::Tree { .. });
    if named.is_empty() && !rows {
        return std::borrow::Cow::Borrowed(v);
    }
    let mut out = match v {
        Value::Object(m) => m.clone(),
        Value::Null => serde_json::Map::new(),
        other => [("$".to_string(), other.clone())].into_iter().collect(),
    };
    if !named.is_empty() {
        let entries: serde_json::Map<String, Value> = named
            .into_iter()
            .map(|k| {
                let value = match st.store.get(k).map(|e| &e.body) {
                    Some(Body::Value { value }) => value.clone(),
                    Some(Body::Text { doc }) => Value::String(doc.text.to_string()),
                    None => Value::Null,
                };
                (k.to_string(), value)
            })
            .collect();
        out.insert("$store".into(), Value::Object(entries));
    }
    if rows {
        out.insert("$rows".into(), items(st, n).len().into());
    }
    std::borrow::Cow::Owned(Value::Object(out))
}

/// Whether a condition reads the size it's laid out in.
pub fn sized(cond: &str) -> bool {
    cond.contains("$width") || cond.contains("$height")
}

/// A `when` condition against the state.
pub fn holds(st: &State, cond: &str) -> bool {
    holds_in(st, cond, None)
}

/// A `when` condition, with `$width` and `$height` the size of the area it's laid out in.
pub fn holds_in(st: &State, cond: &str, size: Option<(u16, u16)>) -> bool {
    // Several conditions, all of which must hold: `"!trash & !confirm"`.
    if cond.contains(" & ") {
        return cond.split(" & ").all(|c| holds_in(st, c, size));
    }
    let cond = cond.trim();
    if let Some(rest) = cond.strip_prefix('!') {
        return !holds_in(st, rest, size);
    }
    let found = ["!=", ">=", "<=", "=", ">", "<"].iter().find_map(|op| cond.split_once(op).map(|(a, b)| (a.trim(), *op, b.trim())));
    let read = |bind: &str| -> Option<Value> {
        match (bind, size) {
            ("$width", Some((w, _))) => return Some(w.into()),
            ("$height", Some((_, h))) => return Some(h.into()),
            _ => {}
        }
        match st.store.get(bind).map(|e| &e.body) {
            Some(Body::Text { doc }) => Some(Value::String(doc.text.to_string())),
            _ => value_of(st, bind).cloned(),
        }
    };
    let Some((bind, op, want)) = found else {
        return match read(cond) {
            None | Some(Value::Null) | Some(Value::Bool(false)) => false,
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Number(n)) => n.as_f64() != Some(0.0),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
            Some(Value::Bool(true)) => true,
        };
    };
    let have = crate::model::scalar(read(bind).as_ref());
    let want = want.trim_matches('"');
    match (crate::model::number(&have), crate::model::number(want)) {
        (Some(a), Some(b)) => match op {
            "=" => a == b,
            "!=" => a != b,
            ">=" => a >= b,
            "<=" => a <= b,
            ">" => a > b,
            _ => a < b,
        },
        _ => match op {
            "=" => have == want,
            "!=" => have != want,
            ">=" => have.as_str() >= want,
            "<=" => have.as_str() <= want,
            ">" => have.as_str() > want,
            _ => have.as_str() < want,
        },
    }
}

/// The shown child of a tabs node.
pub fn tab_of(st: &State, n: &Node) -> usize {
    n.id.as_ref().and_then(|id| st.local.get(id)).map(|l| l.tab).unwrap_or(0)
}

/// The value a node is bound to, if it has one and it has arrived.
pub fn bound<'a>(st: &'a State, n: &Node) -> Option<&'a Value> {
    value_of(st, n.bind.as_deref()?)
}

/// The file an `image` shows: its `src` filled from its bound value. Empty when the template
/// comes out empty (nothing selected yet), so there's nothing to draw.
pub fn image_src(st: &State, n: &Node) -> Option<String> {
    let Kind::Image { src, .. } = &n.kind else { return None };
    let path = crate::model::fill_raw(src, bound(st, n).unwrap_or(&Value::Null));
    (!path.trim().is_empty()).then_some(path)
}

/// A bind's value: a source (and a path into it), or `@id` (and a path), the selected row of
/// node `id`, so a detail panel follows a list's selection.
pub fn value_of<'a>(st: &'a State, bind: &str) -> Option<&'a Value> {
    let (source, path) = split_bind(bind);
    if let Some(id) = source.strip_prefix('@') {
        let n = st.node(id)?;
        if selects_in_a_loop(st, id) {
            return None;
        }
        return lookup(items(st, n).get(selected(st, id))?, path);
    }
    // A store key can hold dots (`remove/photo.tif`): the longest one the bind starts with, up
    // to a dot or its end, is the entry, and the rest a path into it.
    if let Some((key, rest)) = store_key(st, bind) {
        return match &st.store[key].body {
            Body::Value { value } => lookup(value, rest),
            Body::Text { .. } => None,
        };
    }
    match st.data.get(source)? {
        Slot::Ready { value } => lookup(value, path),
        _ => None,
    }
}

/// The store entry a bind names and the path after it: the longest key the bind starts with,
/// ending where the bind does or at a dot.
fn store_key<'b>(st: &State, bind: &'b str) -> Option<(&'b str, &'b str)> {
    let ends = bind.match_indices('.').map(|(i, _)| i).chain(std::iter::once(bind.len()));
    let mut found = None;
    for end in ends {
        if st.store.contains_key(&bind[..end]) {
            found = Some((&bind[..end], bind.get(end + 1..).unwrap_or("")));
        }
    }
    found
}

/// Whether node `id`'s rows come, through `@` binds (another node's selected row), back to
/// `id` itself: a list bound to `@` its own selection. Such a bind reads nothing.
fn selects_in_a_loop(st: &State, id: &str) -> bool {
    let mut seen = vec![id];
    let mut at = id;
    while let Some(next) = st.node(at).and_then(|n| n.bind.as_deref()).and_then(|b| split_bind(b).0.strip_prefix('@')) {
        if seen.contains(&next) {
            return true;
        }
        seen.push(next);
        at = next;
    }
    false
}

/// `bind` is a source name, then optionally a path into its value: `"tick.rows"` reads `rows` of
/// the `tick` source, so one source can feed many nodes.
pub fn split_bind(bind: &str) -> (&str, &str) {
    bind.split_once('.').unwrap_or((bind, ""))
}

/// A list's or table's rows: its literal `items`, else its bound array. Borrowed: a 50,000-row
/// source is read in place, never copied per message or per frame.
pub fn items<'a>(st: &'a State, n: &'a Node) -> &'a [Value] {
    if let Some(rows) = n.id.as_ref().and_then(|id| st.rows.get(id)) {
        return rows;
    }
    raw_items(st, n)
}

/// A node's rows as given: its literal `items`, else its bound array.
fn raw_items<'a>(st: &'a State, n: &'a Node) -> &'a [Value] {
    let literal = match &n.kind {
        Kind::List { items, .. } | Kind::Table { items, .. } | Kind::Tree { items, .. } | Kind::Bars { items, .. } => items,
        _ => return &[],
    };
    if !literal.is_empty() {
        return literal;
    }
    match bound(st, n) {
        Some(Value::Array(a)) => a,
        _ => &[],
    }
}

pub fn selected(st: &State, id: &str) -> usize {
    st.local.get(id).map(|l| l.selected).unwrap_or(0)
}

/// The focused list's selected row, for `run` templates.
pub fn selected_item(st: &State) -> Option<Value> {
    let id = st.focus.as_ref()?;
    let n = st.node(id)?;
    acted_on(st, n).first().cloned()
}

/// What an action run from `n` reads its templates from, the selected row first: the selected
/// and picked rows of a list, tree or table, else the value `n` is bound to (a button bound to
/// `@albums` acts on the album it shows).
fn acted_on(st: &State, n: &Node) -> Vec<Value> {
    let Some(id) = n.id.as_deref() else { return vec![] };
    if !matches!(n.kind, Kind::List { .. } | Kind::Tree { .. } | Kind::Table { .. } | Kind::Bars { .. }) {
        return bound(st, n).cloned().into_iter().collect();
    }
    let rows = items(st, n);
    let lead = selected(st, id);
    let mut out: Vec<Value> = rows.get(lead).cloned().into_iter().collect();
    out.extend(picked(st, id).into_iter().filter(|r| *r != lead).filter_map(|r| rows.get(r).cloned()));
    out
}

/// A source's value from a command's stdout, narrowed by its `path`.
pub fn narrow(raw: &str, path: Option<&str>) -> Result<Value, String> {
    let v: Value = serde_json::from_str(raw.trim()).map_err(|e| format!("not JSON: {e}"))?;
    match path {
        None => Ok(v),
        Some(p) => lookup(&v, p).cloned().ok_or_else(|| format!("no {p:?} in the output")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ui(v: Value) -> Ui {
        serde_json::from_value(v).unwrap()
    }

    fn two_lists() -> Ui {
        ui(json!({
            "root": {"type": "row", "children": [
                {"type": "list", "id": "a", "items": ["one", "two", "three"]},
                {"type": "list", "id": "b", "bind": "tasks", "item": "{text}",
                 "keys": {"x": {"run": ["thc", "done", "{id}"]}}}
            ]},
            "data": {"tasks": {"value": [{"id": "t1", "text": "Buy milk"}, {"id": "t2", "text": "Essay"}]}}
        }))
    }

    fn note() -> Ui {
        ui(json!({
            "root": {"type": "row", "children": [
                {"type": "editor", "id": "note", "text": "hi"},
                {"type": "list", "id": "side", "items": ["a", "b"]}
            ]}
        }))
    }

    fn text_of(st: &State, key: &str) -> String {
        match &st.store[key].body {
            Body::Text { doc } => doc.text.to_string(),
            Body::Value { value } => panic!("{key} is a value: {value}"),
        }
    }

    fn caret(st: &State, id: &str) -> usize {
        field_view(st, id).unwrap().1.caret()
    }

    fn keys(st: &mut State, ks: &[&str]) -> Vec<Effect> {
        ks.iter().flat_map(|k| update(st, Msg::Key { key: (*k).into() })).collect()
    }

    fn write(st: &mut State, key: &str, w: Value, if_rev: Option<u64>) -> Value {
        let write: Write = serde_json::from_value(w).unwrap();
        match update(st, Msg::Write { key: key.into(), write, if_rev, by: "agent".into(), for_ms: None }).pop() {
            Some(Effect::Reply(r)) => r,
            other => panic!("no reply: {other:?}"),
        }
    }

    #[test]
    fn an_editor_takes_the_keys_while_focused() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        assert_eq!(st.focus.as_deref(), Some("note"), "the first focusable node");
        update(&mut st, Msg::Size { id: "note".into(), width: 20, height: 5 });
        let fx = keys(&mut st, &["end", "space", "q", "enter", "j", "c-c"]);
        assert!(fx.is_empty(), "q and Ctrl-C don't quit while editing");
        assert_eq!(text_of(&st, "note"), "hi q\nj");
        assert_eq!(st.store["note"].by, "person");
        // Tab leaves the editor; then the keys are the screen's again.
        keys(&mut st, &["tab", "j"]);
        assert_eq!(st.focus.as_deref(), Some("side"));
        assert_eq!(selected(&st, "side"), 1);
        assert_eq!(text_of(&st, "note"), "hi q\nj");
        assert_eq!(keys(&mut st, &["q"]), vec![Effect::Quit]);
    }

    #[test]
    fn typing_groups_into_undo_steps_by_time() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        keys(&mut st, &["end", "space", "a", "b"]);
        update(&mut st, Msg::Tick { now_ms: 60_000 });
        keys(&mut st, &["c"]);
        keys(&mut st, &["c-z"]);
        assert_eq!(text_of(&st, "note"), "hi ab", "the later burst undoes alone");
        keys(&mut st, &["c-y"]);
        assert_eq!(text_of(&st, "note"), "hi abc");
    }

    #[test]
    fn a_push_never_loses_what_was_written() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        keys(&mut st, &["end", "!"]);
        // A UI without the editor: the note is still in the store.
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "side", "items": ["a"]}})) });
        assert_eq!(text_of(&st, "note"), "hi!");
        assert!(!st.local.contains_key("note"), "its view went with it");
        // The editor back, with a different seed: the note wins over the seed.
        let mut again = note();
        if let Some(c) = again.root.children_mut() {
            c[0].kind = Kind::Editor { text: "a new seed".into(), doc: None, placeholder: None, syntax: None, readonly: false, tab: None, tasks: None, caret: None, links: vec![], ordered: vec![] };
        }
        update(&mut st, Msg::Push { ui: again });
        assert_eq!(text_of(&st, "note"), "hi!");
        // Seeds for values likewise only start entries.
        let seeded = |v: Value| ui(json!({"root": {"type": "list", "id": "l", "bind": "rows"}, "store": {"rows": v}}));
        update(&mut st, Msg::Push { ui: seeded(json!(["seed"])) });
        write(&mut st, "rows", json!({"write": "put", "value": ["agent's"]}), None);
        update(&mut st, Msg::Push { ui: seeded(json!(["another seed"])) });
        assert_eq!(st.store["rows"].read()["value"], json!(["agent's"]));
        let back: State = serde_json::from_value(serde_json::to_value(&st).unwrap()).unwrap();
        assert_eq!(back.store, st.store, "texts with their history, values, revisions");
    }

    #[test]
    fn a_stale_write_is_refused_and_says_what_is_there() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "text", "text": "x"}, "store": {"todo": {"done": 1}}})) });
        let r = write(&mut st, "todo", json!({"write": "merge", "value": {"open": 3}}), Some(1));
        assert_eq!((r["ok"].clone(), r["rev"].clone()), (json!(true), json!(2)));
        let r = write(&mut st, "todo", json!({"write": "put", "value": {}}), Some(1));
        assert_eq!(r["ok"], json!(false), "another agent wrote rev 2");
        assert_eq!(r["current"]["value"], json!({"done": 1, "open": 3}));
        assert_eq!(st.store["todo"].rev, 2);
    }

    #[test]
    fn an_agent_edit_lands_around_the_persons_typing() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        let rev = st.store["note"].rev;
        // The agent read "hi" at `rev`, then the person typed.
        keys(&mut st, &["end", " ", "t", "h", "e", "r", "e"]);
        let r = write(&mut st, "note", json!({"write": "edit", "changes": [[0, 0, "> "]]}), Some(rev));
        assert_eq!(r["ok"], json!(false), "the agent's view of the text is out of date");
        // Read again, edit again: it lands, and the person's caret moves with the text.
        let rev = st.store["note"].rev;
        assert_eq!(caret(&st, "note"), 8);
        let r = write(&mut st, "note", json!({"write": "edit", "changes": [[0, 0, "> "]]}), Some(rev));
        assert_eq!(r["ok"], json!(true));
        assert_eq!(text_of(&st, "note"), "> hi there");
        assert_eq!(caret(&st, "note"), 10, "still after what they typed");
        keys(&mut st, &["c-z"]);
        assert_eq!(text_of(&st, "note"), "hi there", "one undo takes the agent's edit back");
        // A whole new text is a put of a string; a value write to text is refused.
        write(&mut st, "note", json!({"write": "put", "value": "rewritten"}), None);
        assert_eq!(text_of(&st, "note"), "rewritten");
        assert_eq!(write(&mut st, "note", json!({"write": "merge", "value": {}}), None)["ok"], json!(false));
    }

    #[test]
    fn two_editors_on_one_text_share_it() {
        let mut st = State::default();
        update(&mut st, Msg::Push {
            ui: ui(json!({"root": {"type": "row", "children": [
                {"type": "editor", "id": "a", "bind": "doc", "text": "shared"},
                {"type": "editor", "id": "b", "bind": "doc"}
            ]}})),
        });
        // b's caret at the end, then type at the start in a.
        keys(&mut st, &["tab", "end", "s-tab", "x"]);
        assert_eq!(text_of(&st, "doc"), "xshared");
        assert_eq!((caret(&st, "a"), caret(&st, "b")), (1, 7), "b's caret moved with its text");
    }

    #[test]
    fn a_notes_app_is_a_list_and_an_editor_on_the_selected_note() {
        let mut st = State::default();
        update(&mut st, Msg::Push {
            ui: ui(json!({
                "root": {"type": "col", "children": [
                    {"type": "input", "id": "title", "bind": "draft", "placeholder": "New note",
                     "keys": {"enter": {"do": [
                         {"append": "notes", "value": {"id": "{$id}", "title": "{store.draft}"}},
                         {"put": "draft", "value": ""}
                     ]}}},
                    {"type": "list", "id": "list", "bind": "notes", "item": "{title}", "keys": {"x": {"remove": "notes"}}},
                    {"type": "editor", "id": "body", "bind": "@list", "doc": "note/{id}", "text": "# {title}\n"}
                ]},
                "store": {"notes": [{"id": "a", "title": "Groceries"}]}
            })),
        });
        assert_eq!(text_of(&st, "note/a"), "# Groceries\n", "seeded from the selected row");
        // Type a title into the input and press Enter: a new note, the input cleared.
        keys(&mut st, &["H", "i", "enter"]);
        let notes = st.store["notes"].read()["value"].clone();
        assert_eq!(notes[1]["title"], json!("Hi"));
        assert_eq!(text_of(&st, "draft"), "", "the input is cleared");
        assert!(notes[1]["id"].is_string());
        // Select it: the editor now shows (and seeds) that note.
        keys(&mut st, &["tab", "j", "tab", "end", "!"]);
        let id = notes[1]["id"].as_str().unwrap();
        assert_eq!(text_of(&st, &format!("note/{id}")), "# Hi!\n");
        assert_eq!(text_of(&st, "note/a"), "# Groceries\n", "the first note is untouched");
        // Remove the first note from the list; its text stays in the store.
        keys(&mut st, &["s-tab", "k", "x"]);
        assert_eq!(st.store["notes"].read()["value"].as_array().unwrap().len(), 1);
        assert!(st.store.contains_key("note/a"));
    }

    #[test]
    fn an_input_is_one_line_and_esc_leaves_it() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "row", "children": [
            {"type": "input", "id": "q"}, {"type": "list", "id": "l", "items": [1]}
        ]}, "keys": {"esc": "focus_next"}})) });
        keys(&mut st, &["a", "enter", "b"]);
        assert_eq!(text_of(&st, "q"), "ab", "enter adds no line");
        keys(&mut st, &["esc"]);
        assert_eq!(st.focus.as_deref(), Some("l"));
    }

    #[test]
    fn copy_puts_a_command_for_this_screen_on_the_clipboard() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "l", "items": [{"cmd": "fictty theme paper"}],
            "keys": {"enter": {"copy": "{$self.cli} {cmd}"}}}})) });
        update(&mut st, Msg::Host { bin: "/opt/it's/fictty".into(), socket: "/tmp/f.sock".into(), dir: "/srv/ui".into() });
        assert_eq!(update(&mut st, Msg::Key { key: "enter".into() }), vec![Effect::Copy("FICTTY_SOCKET='/tmp/f.sock' '/opt/it'\\''s/fictty' fictty theme paper".into())]);
        let Some(Slot::Ready { value }) = st.data.get("$self") else { panic!("no $self") };
        assert_eq!(value["env"], "export PATH='/opt/it'\\''s':'/srv/ui':\"$PATH\" FICTTY_SOCKET='/tmp/f.sock'");
    }

    #[test]
    fn a_number_input_takes_only_numbers_and_submits_only_a_whole_one() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "input", "id": "amt", "title": "Amount", "accept": "number",
            "keys": {"enter": {"append": "rows", "value": "{store.amt}"}}}, "store": {"rows": []}})) });
        keys(&mut st, &["-", "enter"]);
        assert_eq!(st.status.as_deref(), Some("Amount: \"-\" isn't a number"));
        assert_eq!(st.store["rows"].read()["value"], json!([]), "nothing booked");
        keys(&mut st, &["3", "-", "x", ".", "5", ".", "0"]);
        assert_eq!(text_of(&st, "amt"), "-3.50", "the second '-', the 'x' and the second '.' were refused");
        keys(&mut st, &["enter"]);
        assert_eq!(st.store["rows"].read()["value"], json!(["-3.50"]));
    }

    #[test]
    fn store_actions_keep_types_and_say_what_went_wrong() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({
            "root": {"type": "list", "id": "l", "bind": "rows", "keys": {
                "a": {"append": "rows", "value": {"amount": "{store.amt}", "n": "{store.n}", "note": "paid {store.amt}"}},
                "b": {"append": "amt", "value": 1}
            }},
            "store": {"rows": [], "amt": "-12.50", "n": 3}
        })) });
        keys(&mut st, &["a"]);
        assert_eq!(st.store["rows"].read()["value"], json!([{"amount": "-12.50", "n": 3, "note": "paid -12.50"}]));
        keys(&mut st, &["b"]);
        assert_eq!(st.status.as_deref(), Some("append: amt isn't a list"));
    }

    #[test]
    fn themes_are_loaded_chosen_and_survive_a_push() {
        let mut st = State::default();
        let theme = |accent: &str| serde_json::from_value::<ThemeSpec>(json!({"tokens": {"accent": accent}})).unwrap();
        update(&mut st, Msg::Theme { add: [("glass".to_string(), theme("#39d6e6")), ("paper".to_string(), theme("#0f8fa0"))].into(), choose: None });
        assert_eq!(st.theme.as_deref(), Some("glass"), "the first loaded is chosen");
        update(&mut st, Msg::Push { ui: note() });
        assert_eq!(st.theme.as_deref(), Some("glass"));
        keys(&mut st, &["tab"]);
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "l", "items": [1], "keys": {"t": {"theme": "next"}}}})) });
        keys(&mut st, &["t"]);
        assert_eq!(st.theme.as_deref(), Some("paper"));
        keys(&mut st, &["t"]);
        assert_eq!(st.theme.as_deref(), Some("glass"), "next wraps around");
        update(&mut st, Msg::Theme { add: BTreeMap::new(), choose: Some("neon".into()) });
        assert!(st.status.as_deref().unwrap().starts_with("theme: no theme named \"neon\""));
    }

    #[test]
    fn problems_say_what_is_wrong_in_words() {
        let mut st = State::default();
        let theme: ThemeSpec = serde_json::from_value(json!({"tokens": {"warn": "#ffbd4f"}, "variants": {"card": {}}})).unwrap();
        update(&mut st, Msg::Theme { add: [("t".to_string(), theme)].into(), choose: None });
        update(&mut st, Msg::Push { ui: ui(json!({
            "root": {"type": "col", "children": [
                {"type": "table", "id": "t", "bind": "rows", "columns": [{"title": "a", "value": "{a}"}]},
                {"type": "sparkline", "id": "s", "bind": "nums"},
                {"type": "text", "id": "x", "variant": "cardd", "tone": "warn", "text": "hi"},
                {"type": "text", "id": "y", "variant": "card", "tone": "warning", "text": "hi"}
            ]},
            "store": {"rows": {"a": 1}, "nums": [1, "two"]}
        })) });
        assert_eq!(st.problems, vec![
            "table t: rows should be a list, but it's an object".to_string(),
            "sparkline s: nums should be a list of numbers (or set `field`)".to_string(),
            "x: the theme has no variant \"cardd\"".to_string(),
            "y: tone \"warning\" isn't a theme token or a colour".to_string(),
        ]);
        // Fixing the data clears it.
        let r = write(&mut st, "rows", json!({"write": "put", "value": [{"a": 1}]}), None);
        assert_eq!(r["ok"], json!(true));
        assert_eq!(st.problems.len(), 3);
    }

    #[test]
    fn a_default_token_is_a_tone_with_no_theme_loaded() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "col", "children": [
            {"type": "text", "id": "a", "tone": "accent", "text": "hi"},
            {"type": "text", "id": "m", "tone": "muted", "text": "hi"},
            {"type": "text", "id": "w", "tone": "warning", "text": "hi"}
        ]}})) });
        assert_eq!(st.problems, vec!["w: tone \"warning\" isn't a theme token or a colour".to_string()]);
    }

    #[test]
    fn a_click_in_an_editor_focuses_it_and_places_the_caret() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "l", "items": [1]},
            {"type": "editor", "id": "e", "text": "first line\nsecond"}
        ]}})) });
        assert_eq!(st.focus.as_deref(), Some("l"));
        update(&mut st, Msg::Size { id: "e".into(), width: 20, height: 4 });
        update(&mut st, Msg::Point { id: "e".into(), col: 3, row: 1, extend: false });
        assert_eq!(st.focus.as_deref(), Some("e"));
        assert_eq!(caret(&st, "e"), 14, "line two, column three");
    }

    #[test]
    fn a_list_bound_to_its_own_selection_reads_nothing() {
        // Through another node too. It used to recurse until the stack ran out.
        for ui in [
            serde_json::json!({"root": {"type": "list", "id": "a", "bind": "@a"}}),
            serde_json::json!({"root": {"type": "col", "children": [
                {"type": "list", "id": "a", "bind": "@b.kids"}, {"type": "list", "id": "b", "bind": "@a.kids"}]}}),
        ] {
            let mut st = State::default();
            update(&mut st, Msg::Push { ui: serde_json::from_value(ui).unwrap() });
            update(&mut st, Msg::Key { key: "j".into() });
            assert!(st.problems.iter().any(|p| p.starts_with("a: its bind") && p.ends_with("reads nothing")), "{:?}", st.problems);
        }
    }

    #[test]
    fn keys_wait_for_an_act_that_says_so_and_for_what_it_refreshes() {
        let ui: Ui = serde_json::from_value(serde_json::json!({
            "root": {"type": "list", "id": "panels", "bind": "panels", "item": "{.}",
                     "keys": {"w": {"host": "close", "wait": true}, "v": {"host": "log"}}},
            "data": {"panels": {"host": "panels"}},
            "focus": "panels"
        }))
        .unwrap();
        let mut st = State::default();
        update(&mut st, Msg::Push { ui });
        update(&mut st, Msg::Data { name: "panels".into(), result: Ok(serde_json::json!(["a", "b", "c"])) });
        // An act without `wait` holds nothing.
        update(&mut st, Msg::Key { key: "v".into() });
        update(&mut st, Msg::Key { key: "j".into() });
        assert_eq!(selected(&st, "panels"), 1);
        // With it, the keys after it wait for its answer, then for the sources it refreshes.
        let fx = update(&mut st, Msg::Key { key: "w".into() });
        assert_eq!(fx, vec![Effect::Host { name: "close".into(), args: Value::Null }]);
        update(&mut st, Msg::Key { key: "j".into() });
        update(&mut st, Msg::Key { key: "j".into() });
        assert_eq!((selected(&st, "panels"), st.held.len()), (1, 2));
        let fx = update(&mut st, Msg::Acted { name: "close".into(), result: Ok(Value::Null), at: String::new() });
        assert_eq!(fx, vec![Effect::Fetch("panels".into())]);
        assert_eq!(st.waiting, vec!["data:panels".to_string()]);
        update(&mut st, Msg::Data { name: "panels".into(), result: Ok(serde_json::json!(["a", "b", "c", "d"])) });
        assert!(st.waiting.is_empty() && st.held.is_empty());
        assert_eq!(selected(&st, "panels"), 3, "both keys, after it landed");
    }

    #[test]
    fn set_moves_view_state() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: note() });
        update(&mut st, Msg::Set { id: "side".into(), selected: Some(1), tab: None, key: None });
        assert_eq!(selected(&st, "side"), 1);
        update(&mut st, Msg::Set { id: "nope".into(), selected: Some(1), tab: None, key: None });
        assert_eq!(st.status.as_deref(), Some("set: no node with id \"nope\""));
    }

    #[test]
    fn set_moves_a_selection_off_a_row_it_remembers_by_key() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: ui(json!({"root": {"type": "list", "id": "l", "bind": "rows", "item": "{id}"},
            "store": {"rows": [{"id": "a"}, {"id": "b"}, {"id": "c"}]}})) });
        assert_eq!(st.local["l"].key.as_deref(), Some("a"));
        update(&mut st, Msg::Set { id: "l".into(), selected: Some(2), tab: None, key: None });
        assert_eq!((selected(&st, "l"), st.local["l"].key.as_deref()), (2, Some("c")), "on c, and remembered there");
    }

    #[test]
    fn the_log_joins_typed_keys_and_keeps_the_newest_first() {
        let mut st = State::default();
        let act = |op: &str, what: &str| Msg::Activity { op: op.into(), what: what.into(), at: "09:41:00".into() };
        for m in [act("push", "a new UI"), act("key", "h"), act("key", "i"), act("key", "space"), act("key", "enter"), act("set", "note")] {
            update(&mut st, m);
        }
        let Some(Slot::Ready { value }) = st.data.get("$log") else { panic!("no log") };
        let ops: Vec<(&str, &str)> = value.as_array().unwrap().iter().map(|e| (e["op"].as_str().unwrap(), e["what"].as_str().unwrap())).collect();
        assert_eq!(ops, vec![("set", "note"), ("key", "hi ⏎"), ("push", "a new UI")]);
    }

    #[test]
    fn push_focuses_the_first_list_and_fetches_sources() {
        let mut st = State::default();
        let fx = update(&mut st, Msg::Push { ui: two_lists() });
        assert_eq!(st.focus.as_deref(), Some("a"));
        assert_eq!(fx, vec![Effect::Fetch("tasks".into())]);
    }

    #[test]
    fn keys_move_the_selection_and_run_templates_the_selected_row() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        update(&mut st, Msg::Data { name: "tasks".into(), result: Ok(json!([{"id": "t1"}, {"id": "t2"}])) });
        for k in ["j", "j", "j"] {
            update(&mut st, Msg::Key { key: k.into() });
        }
        assert_eq!(selected(&st, "a"), 2, "clamped to the last row");
        update(&mut st, Msg::Key { key: "tab".into() });
        update(&mut st, Msg::Key { key: "j".into() });
        let fx = update(&mut st, Msg::Key { key: "x".into() });
        assert_eq!(fx, vec![Effect::Run(vec!["thc".into(), "done".into(), "t2".into()])]);
    }

    #[test]
    fn view_state_survives_a_push_for_ids_that_remain() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        update(&mut st, Msg::Key { key: "j".into() });
        let mut next = two_lists();
        next.root.children_mut().unwrap().remove(1);
        update(&mut st, Msg::Push { ui: next });
        assert_eq!(selected(&st, "a"), 1);
        assert!(!st.local.contains_key("b"));
        assert_eq!(st.version, 2);
    }

    #[test]
    fn a_bind_can_reach_into_its_source() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "list", "id": "l", "bind": "feed.rows.all"},
                           "data": {"feed": {"value": null}}}));
        update(&mut st, Msg::Push { ui });
        update(&mut st, Msg::Data { name: "feed".into(), result: Ok(json!({"rows": {"all": [1, 2, 3]}})) });
        assert_eq!(items(&st, st.node("l").unwrap()).len(), 3);
    }

    #[test]
    fn clicks_focus_select_and_switch_tabs_and_focus_skips_hidden_tabs() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "tabs", "id": "t", "tabs": ["A", "B"], "children": [
            {"type": "list", "id": "a", "items": [1, 2, 3]},
            {"type": "list", "id": "b", "items": [1, 2, 3], "click": {"run": ["echo", "{.}"]}}
        ]}}));
        update(&mut st, Msg::Push { ui });
        assert_eq!(focus_order(&st), vec!["t", "a"]);
        update(&mut st, Msg::Click { id: "t".into(), row: None, tab: Some(1) });
        assert_eq!(focus_order(&st), vec!["t", "b"]);
        let fx = update(&mut st, Msg::Click { id: "b".into(), row: Some(2), tab: None });
        assert_eq!(st.focus.as_deref(), Some("b"));
        assert_eq!(fx, vec![Effect::Run(vec!["echo".into(), "3".into()])]);
        update(&mut st, Msg::Scroll { id: "b".into(), by: -5 });
        assert_eq!(selected(&st, "b"), 0);
    }

    #[test]
    fn focus_leaves_a_node_a_write_hides() {
        // Thought Control's pty run: a write hid the focused node's column, and the focus stayed
        // on it, out of the focus order, where no key reached anything.
        let mut st = State::default();
        let first = ui(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "main", "items": ["a"]},
            {"type": "col", "when": "side", "children": [{"type": "list", "id": "head", "items": ["h"]}]}
        ]}, "store": {"side": true}, "focus": "head"}));
        update(&mut st, Msg::Push { ui: first });
        assert_eq!(st.focus.as_deref(), Some("head"));
        let write = crate::store::Write::Put { value: json!(false) };
        update(&mut st, Msg::Write { key: "side".into(), write, if_rev: None, by: "agent".into(), for_ms: None });
        assert_eq!(st.focus.as_deref(), Some("main"));
        // And when a source's value hides it.
        let mut st = State::default();
        let pushed = ui(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "main", "items": ["a"]},
            {"type": "col", "when": "sidebar.open", "children": [{"type": "list", "id": "head", "items": ["h"]}]}
        ]}, "data": {"sidebar": {"host": "sidebar"}}, "focus": "main"}));
        update(&mut st, Msg::Push { ui: pushed });
        update(&mut st, Msg::Data { name: "sidebar".into(), result: Ok(json!({"open": true})) });
        update(&mut st, Msg::Focus { id: "head".into() });
        assert_eq!(st.focus.as_deref(), Some("head"));
        update(&mut st, Msg::Data { name: "sidebar".into(), result: Ok(json!({"open": false})) });
        assert_eq!(st.focus.as_deref(), Some("main"));
        // And when the screen narrows past its `when`.
        let mut st = State::default();
        update(&mut st, Msg::Screen { width: 130, height: 30 });
        let pushed = ui(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "main", "items": ["a"]},
            {"type": "col", "when": "$screen.width>=120", "children": [{"type": "list", "id": "head", "items": ["h"]}]}
        ]}, "focus": "head"}));
        update(&mut st, Msg::Push { ui: pushed });
        assert_eq!(st.focus.as_deref(), Some("head"));
        update(&mut st, Msg::Screen { width: 100, height: 30 });
        assert_eq!(st.focus.as_deref(), Some("main"));
    }

    #[test]
    fn a_click_outside_a_modal_goes_nowhere() {
        // Thought Control's fuzzer: with the palette open, a click on the header above it took
        // the focus out of the palette.
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "list", "id": "header", "items": ["a"], "click": {"toast": "clicked"}},
            {"type": "stack", "children": [
                {"type": "list", "id": "body", "items": [1, 2]},
                {"type": "input", "id": "q", "modal": true, "when": "palette"}
            ]}
        ]}, "store": {"palette": true}, "focus": "q"}));
        update(&mut st, Msg::Push { ui });
        assert_eq!(st.focus.as_deref(), Some("q"));
        for msg in [Msg::Click { id: "header".into(), row: Some(0), tab: None }, Msg::Scroll { id: "body".into(), by: 1 }] {
            assert_eq!(update(&mut st, msg), vec![]);
            assert_eq!(st.focus.as_deref(), Some("q"));
        }
        assert!(st.toasts.is_empty());
        update(&mut st, Msg::Click { id: "q".into(), row: None, tab: None });
        assert_eq!(st.focus.as_deref(), Some("q"));
    }

    #[test]
    fn toggle_and_remove_act_on_every_picked_row() {
        let mut st = State::default();
        let rows: Vec<Value> = (0..5).map(|i| json!({"id": format!("r{i}"), "ok": i == 1})).collect();
        let ui = ui(json!({"root": {"type": "list", "id": "l", "multi": true, "bind": "rows",
            "keys": {"x": {"toggle": "ok"}, "d": {"remove": "rows"}}}, "store": {"rows": rows}, "focus": "l"}));
        update(&mut st, Msg::Push { ui });
        update(&mut st, Msg::Click { id: "l".into(), row: Some(0), tab: None });
        update(&mut st, Msg::Pick { id: "l".into(), row: 2, how: Pick::Range });
        assert_eq!(picked(&st, "l"), vec![0, 1, 2]);
        let oks = |st: &State| match &st.store["rows"].body {
            Body::Value { value } => value.as_array().unwrap().iter().map(|r| r["ok"].as_bool().unwrap_or(false)).collect::<Vec<_>>(),
            _ => vec![],
        };
        // The selected row (2) was false, so every picked row becomes true, the one that was too.
        keys(&mut st, &["x"]);
        assert_eq!(oks(&st), vec![true, true, true, false, false]);
        keys(&mut st, &["x"]);
        assert_eq!(oks(&st), vec![false, false, false, false, false]);
        keys(&mut st, &["d"]);
        let ids = match &st.store["rows"].body {
            Body::Value { value } => value.as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_string()).collect::<Vec<_>>(),
            _ => vec![],
        };
        assert_eq!(ids, vec!["r3", "r4"]);
        assert_eq!(picked(&st, "l").len(), 1, "what's left is one row again");
        let Some(Slot::Ready { value }) = st.data.get("$picked") else { panic!("no $picked") };
        assert_eq!(value["l"]["count"], 1);
    }

    #[test]
    fn a_toggle_of_a_store_entry_named_from_the_row_flips_one_per_picked_row() {
        let mut st = State::default();
        let rows: Vec<Value> = (0..4).map(|i| json!({"id": format!("r{i}")})).collect();
        let ui = ui(json!({"root": {"type": "list", "id": "l", "multi": true, "items": rows,
            "keys": {"x": {"toggle": "store.remove/{id}"}}}, "focus": "l"}));
        update(&mut st, Msg::Push { ui });
        keys(&mut st, &["x"]);
        assert_eq!(st.store["remove/r0"].body, Body::Value { value: json!(true) });
        update(&mut st, Msg::Pick { id: "l".into(), row: 2, how: Pick::Range });
        keys(&mut st, &["x"]);
        let flags: Vec<Value> = (0..4).map(|i| match st.store.get(&format!("remove/r{i}")).map(|e| &e.body) {
            Some(Body::Value { value }) => value.clone(),
            _ => Value::Null,
        }).collect();
        // The selected row (2) wasn't flagged, so all three picked are now.
        assert_eq!(flags, vec![json!(true), json!(true), json!(true), Value::Null]);
        let Some(Slot::Ready { value }) = st.data.get("$picked") else { panic!("no $picked") };
        assert_eq!((value["l"]["index"].clone(), value["l"]["total"].clone()), (json!(3), json!(4)));
    }

    #[test]
    fn a_source_hidden_by_a_store_entry_is_a_problem_until_the_entry_is_deleted() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "list", "id": "l", "bind": "rows"},
            "data": {"rows": {"value": [{"id": "new"}]}}, "store": {"rows": [{"id": "old"}]}}));
        update(&mut st, Msg::Push { ui });
        // The source's value, as the engine fetches it.
        update(&mut st, Msg::Data { name: "rows".into(), result: Ok(json!([{"id": "new"}])) });
        let first = |st: &State| bound(st, &st.node("l").cloned().unwrap()).cloned();
        assert_eq!(first(&st), Some(json!([{"id": "old"}])), "the store's");
        assert!(st.problems.iter().any(|p| p.starts_with("rows is a store entry and a data source")), "{:?}", st.problems);
        update(&mut st, Msg::Write { key: "rows".into(), write: Write::Delete {}, if_rev: None, by: "agent".into(), for_ms: None });
        assert_eq!(first(&st), Some(json!([{"id": "new"}])), "the source's, once the entry is gone");
        assert!(st.problems.is_empty(), "{:?}", st.problems);
    }

    #[test]
    fn a_when_is_filled_from_the_nodes_bound_row() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "list", "id": "files", "items": [{"id": "a.tif"}, {"id": "b.tif"}]},
            {"type": "text", "id": "mark", "bind": "@files", "when": "remove/{id}", "text": "flagged"}
        ]}, "store": {"remove/b.tif": true}, "focus": "files"}));
        update(&mut st, Msg::Push { ui });
        let mark = |st: &State| shown(st, &st.node("mark").cloned().unwrap());
        assert!(!mark(&st), "a.tif isn't flagged");
        keys(&mut st, &["j"]);
        assert!(mark(&st), "b.tif is");
    }

    #[test]
    fn a_store_key_with_dots_is_bound_and_shown_whole() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "text", "id": "flag", "when": "remove/photo.tif", "text": "flagged"},
            {"type": "text", "id": "size", "bind": "meta/photo.tif.size", "text": "{w}"}
        ]}, "store": {"remove/photo.tif": true, "meta/photo.tif": {"size": {"w": 640}}}}));
        update(&mut st, Msg::Push { ui });
        assert!(holds(&st, "remove/photo.tif"));
        assert!(!holds(&st, "remove/other.tif"));
        let size = st.node("size").cloned().unwrap();
        assert_eq!(bound(&st, &size), Some(&json!({"w": 640})), "the key, then a path into it");
    }

    #[test]
    fn a_click_on_a_node_without_one_runs_the_nearest_around_it() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "row", "id": "card", "click": {"put": "picked", "value": "card"}, "children": [
                {"type": "image", "id": "pic", "src": "a.png"},
                {"type": "text", "id": "own", "text": "x", "click": {"put": "picked", "value": "own"}}
            ]}
        ]}}));
        update(&mut st, Msg::Push { ui });
        update(&mut st, Msg::Click { id: "pic".into(), row: None, tab: None });
        assert_eq!(st.store.get("picked").map(|e| &e.body), Some(&Body::Value { value: json!("card") }));
        update(&mut st, Msg::Click { id: "own".into(), row: None, tab: None });
        assert_eq!(st.store.get("picked").map(|e| &e.body), Some(&Body::Value { value: json!("own") }), "its own click wins");
    }

    #[test]
    fn an_at_bind_follows_another_nodes_selection() {
        let mut st = State::default();
        let ui = ui(json!({"root": {"type": "col", "children": [
            {"type": "list", "id": "l", "items": [{"n": "a"}, {"n": "b"}]},
            {"type": "text", "id": "d", "bind": "@l", "text": "{n}"}
        ]}}));
        update(&mut st, Msg::Push { ui });
        let detail = |st: &State| crate::model::fill("{n}", bound(st, st.node("d").unwrap()).unwrap());
        assert_eq!(detail(&st), "a");
        update(&mut st, Msg::Key { key: "j".into() });
        assert_eq!(detail(&st), "b");
    }

    #[test]
    fn patch_replaces_one_node() {
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: two_lists() });
        let node: Node = serde_json::from_value(json!({"type": "list", "id": "a", "items": ["only"]})).unwrap();
        update(&mut st, Msg::Patch { id: "a".into(), node });
        assert_eq!(items(&st, st.node("a").unwrap()), &[json!("only")]);
        update(&mut st, Msg::Patch { id: "zz".into(), node: Node::default() });
        assert!(st.status.unwrap().contains("zz"));
    }
}

