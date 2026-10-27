//! The native window (`fictty run --window`): the core's engine on its own thread, the state drawn
//! by GPUI in pixels each frame, every input going in as a fictty `Msg`. This is the spike of
//! 11 October brought onto `main` so each step of the move (docs/design/native.md) is visible:
//! a subset of the components, one palette, no themes yet. Phase 1 brings it to parity.
//!
//! What a frame laid out goes back to the engine as the terminal's did (`Drawn`): each node's
//! rectangle, the cells each editor and list got, what's under the mouse, and when the frame
//! changes by itself. A measuring canvas in each node collects its bounds while GPUI prepaints;
//! a last canvas hands them over once the frame has painted. Sizes are in cells of the editors'
//! monospace font, as in a terminal.

mod look;
#[cfg(feature = "window-test")]
pub mod test;
mod text;

use crate::engine::{Drawn, Hit, Live, Rect};
use look::{Ink, Look, inked, padded};
use crate::style;
use crate::model::{Align, Aspect, Filled, Fit, Float, Justify, Kind, Len, Lens, Md, Measure, Node, Size, md_roles};
use crate::state::{self, Msg, State};
use anyhow::Result;
use gpui::{
    AnyElement, App, Application, Bounds, CursorStyle, ObjectFit, StyledImage, img, ClipboardItem, Context, DefiniteLength, DispatchPhase, FocusHandle, HighlightStyle, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    MouseUpEvent, ExternalPaths, Point, ScrollWheelEvent, SharedString, StyledText, Window, WindowBounds, WindowOptions, canvas, div, font, prelude::*, px, relative, size,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A fictty row, in pixels: what a size in cells turns into, and every line's height.
const ROW: f32 = 24.0;
/// The type size.
const TYPE: f32 = 14.0;
/// The monospace font editors and inputs use, whose character width is a fictty column.
const MONO: &str = "Menlo";

/// A fictty cell in pixels: a column (the monospace font's character width) by a row.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    w: f32,
    h: f32,
}

impl Cell {
    /// Window pixels as whole cells, from the window's top-left corner.
    fn rect(&self, b: Bounds<Pixels>) -> Rect {
        let (x, y) = (f32::from(b.origin.x) / self.w, f32::from(b.origin.y) / self.h);
        let (w, h) = (f32::from(b.size.width) / self.w, f32::from(b.size.height) / self.h);
        let c = |v: f32| v.round().clamp(0.0, u16::MAX as f32) as u16;
        Rect::new(c(x), c(y), c(x + w).saturating_sub(c(x)), c(y + h).saturating_sub(c(y)))
    }

    /// How many whole columns and rows fit in `b`.
    fn fit(&self, b: Bounds<Pixels>) -> (u16, u16) {
        ((f32::from(b.size.width) / self.w).floor() as u16, (f32::from(b.size.height) / self.h).floor() as u16)
    }

    /// The cell `at` falls in, relative to `b`, clamped to it (a drag past the edge stays on it;
    /// the row may be one past the last, as a terminal's drag below an editor is).
    fn within(&self, b: Bounds<Pixels>, at: Point<Pixels>) -> (u16, u16) {
        let (cols, rows) = self.fit(b);
        let col = (f32::from(at.x - b.origin.x) / self.w).floor().max(0.0) as u16;
        let row = (f32::from(at.y - b.origin.y) / self.h).floor().max(0.0) as u16;
        (col.min(cols.saturating_sub(1)), row.min(rows))
    }
}

/// What the frame being built laid out, in window pixels, collected while GPUI prepaints.
#[derive(Default)]
struct Laid {
    rects: Vec<(String, Bounds<Pixels>)>,
    /// Hit regions, innermost last: a node, maybe a row or a tab, maybe an editor's text. Their
    /// own rectangles are filled in from the bounds when the frame is handed over.
    hits: DrawnHits,
    /// Each editor's text area, each list's and table's rows' box, and each fitting grid's
    /// bounds, to fit cells into: the room each has.
    sizes: Vec<(String, Bounds<Pixels>)>,
    /// How tall each `fit` row a list built is: the list, the row's key, its bounds.
    heights: Vec<(String, String, Bounds<Pixels>)>,
    /// The area each node whose `when` reads `$width` or `$height` was laid out in.
    areas: Vec<(String, Bounds<Pixels>)>,
    /// The nodes it cut text in.
    cut: Vec<String>,
    /// The font files it couldn't load, and why.
    fonts: Vec<(String, String)>,
    /// Each scroller's view (whether it scrolls down), and what it holds, by id.
    views: Vec<(String, bool, Bounds<Pixels>)>,
    contents: Vec<(String, Bounds<Pixels>)>,
    /// Where the focused node was laid out, unclipped: it may be scrolled out of sight.
    focused: Option<Bounds<Pixels>>,
}

type Shared = Rc<RefCell<Laid>>;

/// A painted frame's hit regions, in pixels, innermost last.
type DrawnHits = Vec<(Bounds<Pixels>, Hit)>;

/// What's under the mouse in `id`: the whole node, or one row or tab of it, or an editor's text.
fn hit(id: String, row: Option<usize>, tab: Option<usize>, text: bool) -> Hit {
    Hit { rect: Rect::default(), id, row, tab, text, part: None }
}

/// The part a list's group heading is hit as: its row is the group's first.
const SECTION: &str = "$section";

/// Whether a drag or a drop can aim at what `h` is: a row, or a group's heading (before its
/// first row), not a button in a row.
fn droppable_on(h: &Hit) -> bool {
    h.part.as_deref().is_none_or(|p| p == SECTION)
}

/// A canvas that notes, as it's laid out, the bounds it filled: absolute, so it takes no room.
/// What's noted is what shows: the bounds clipped to what holds it (a node squeezed out of its
/// parent's room is clipped away, and so is its hit).
fn measure(laid: &Shared, note: impl FnOnce(&mut Laid, Bounds<Pixels>) + 'static) -> AnyElement {
    let laid = laid.clone();
    canvas(move |b, window, _| note(&mut laid.borrow_mut(), b.intersect(&window.content_mask().bounds)), |_, _, _, _| {}).absolute().inset_0().into_any_element()
}

/// A canvas that notes the whole of its bounds, shown or not: what a scroller holds, where the
/// focus is when it's scrolled out of sight, and the room a list or an editor was given (its
/// box, not the part a scroller around it shows).
fn measure_whole(laid: &Shared, note: impl FnOnce(&mut Laid, Bounds<Pixels>) + 'static) -> AnyElement {
    let laid = laid.clone();
    canvas(move |b, _, _| note(&mut laid.borrow_mut(), b), |_, _, _, _| {}).absolute().inset_0().into_any_element()
}

/// Where the window's state comes from and where its messages go.
#[derive(Clone)]
enum Engine {
    /// fictty's engine, on its own thread (`fictty run --window`): it takes in sources and
    /// requests as fast as they come; the window reads its state when it draws.
    Live(Arc<Mutex<Live>>),
    /// A test's `Headless`, whose frame the window is (`App::in_window`).
    #[cfg(feature = "window-test")]
    Test(Rc<RefCell<test::Side>>),
}

impl Engine {
    fn send(&self, msg: Msg) {
        match self {
            Engine::Live(live) => live.lock().expect("the engine").msg(msg),
            #[cfg(feature = "window-test")]
            Engine::Test(side) => side.borrow_mut().sent.push(msg),
        }
    }

    /// Whether node `id` is a list that takes files dragged in (its `drop` has `files`).
    fn takes_files(&self, id: &str) -> bool {
        let takes = |st: &State| st.node(id).is_some_and(|n| n.drop.as_ref().is_some_and(|d| d.files.is_some()));
        match self {
            Engine::Live(live) => takes(live.lock().expect("the engine").state()),
            #[cfg(feature = "window-test")]
            Engine::Test(side) => takes(&side.borrow().st),
        }
    }

    /// The row the state has the mouse over.
    fn hover(&self) -> Option<state::Hover> {
        match self {
            Engine::Live(live) => live.lock().expect("the engine").state().hover.clone(),
            #[cfg(feature = "window-test")]
            Engine::Test(side) => side.borrow().st.hover.clone(),
        }
    }

    /// A painted frame laid this out: true when the state changed because of it (an editor
    /// learned its size), so another frame should follow.
    fn drew(&self, took: Duration, drawn: Drawn) -> bool {
        match self {
            Engine::Live(live) => live.lock().expect("the engine").drew(took, drawn),
            #[cfg(feature = "window-test")]
            Engine::Test(side) => {
                side.borrow_mut().drawn = Some(drawn);
                false
            }
        }
    }
}

struct Portal {
    engine: Engine,
    focus: FocusHandle,
    cell: Cell,
    /// The window's size in cells, as last told to the engine.
    screen: (u16, u16),
    /// What the frame being built lays out, handed to the engine once it has painted.
    laid: Shared,
    /// The last painted frame's hit regions, in pixels, innermost last: for what GPUI doesn't
    /// route itself (an editor's cells, the row under the mouse).
    shown: Rc<RefCell<DrawnHits>>,
    /// The editor a press in its text started a drag in, until the button comes up.
    drag: Option<String>,
    /// A `resize` handle being dragged, until the button comes up: the node, where the mouse
    /// went down, how tall the node was then and how tall the drag has made it, in pixels.
    sizing: Option<(String, f32, f32, f32)>,
    /// The wheel's travel over a node, not yet a whole scroll step.
    wheel: (String, f32),
    /// When (on the logical clock) the frame being built changes by itself next.
    wake: Option<u64>,
    /// The font files the themes name, by path, and the family each loaded as (empty when it
    /// couldn't be).
    faces: HashMap<String, String>,
    /// How many `fit` nodes the node being built is inside: their cut lines say how wide
    /// they'd be.
    fitting: usize,
    /// Set while a scroller builds its children: the way it scrolls (true across a row), where
    /// they have all the room they want, so a share is as long as what it shows.
    loose: Option<bool>,
    /// The list and row whose template is being built, so its buttons press on that row.
    row: Option<(String, usize)>,
    /// A press on a row of a list with a `drop`, until the button comes up: a click then, or a
    /// drag once it has moved far enough.
    press: Option<Pressed>,
    /// How many rows each list laid out to a line, last frame: a drag over a grid of rows goes
    /// before or after a row by its halves across, over a column of them by its halves down.
    per_line: HashMap<String, usize>,
    /// The window's title as last set (the UI's `title`, filled).
    title: Option<String>,
    /// Files dragged in from outside, while they are: the list and row (and side) last told to
    /// the engine, so a move only tells it when that changes.
    brought: Option<Aim>,
}

/// Where files brought in would land: list `id`, beside row `row` on its side, or (`row` none)
/// after its last row.
type Aim = (Option<String>, Option<(usize, state::Side)>);

/// A press on row `row` of list `id`, where the mouse went down, the click it is if it doesn't
/// move, and once it has (`lifted`), where the rows would land.
struct Pressed {
    id: String,
    row: usize,
    at: Point<Pixels>,
    click: Msg,
    lifted: bool,
    over: Option<(usize, state::Side)>,
}

/// How far a press moves before it's a drag, in pixels.
const LIFT: f32 = 4.0;

impl Portal {
    /// Open on `engine`, measuring the cell: a column is the monospace font's character width.
    fn new(engine: Engine, window: &mut Window, cx: &mut Context<Self>) -> Portal {
        let ts = window.text_system();
        let w = ts.ch_advance(ts.resolve_font(&font(MONO)), px(TYPE)).map(f32::from).unwrap_or(8.4);
        let cell = Cell { w, h: ROW };
        // Pictures and charts size themselves by the cell, as in a terminal.
        engine.send(Msg::Caps { caps: state::Caps { graphics: false, cell_w: cell.w, cell_h: cell.h }, text_sizing: false });
        let focus = cx.focus_handle();
        focus.focus(window);
        Portal { engine, focus, cell, screen: (0, 0), laid: Shared::default(), shown: Rc::default(), drag: None, sizing: None, wheel: (String::new(), 0.0), wake: None, faces: HashMap::new(), fitting: 0, loose: None, row: None, press: None, per_line: HashMap::new(), brought: None, title: None }
    }

    fn send(&mut self, msg: Msg, cx: &mut Context<Self>) {
        self.engine.send(msg);
        cx.notify();
    }

    /// The frame changes by itself at `at` (a spinner turning, a toast going).
    fn soon(&mut self, at: u64) {
        self.wake = Some(self.wake.map_or(at, |w| w.min(at)));
    }

    /// The wheel over `id`: a mouse wheel's notch is a step (three rows, as in a terminal), and
    /// a trackpad's travel a step each three rows of it.
    /// The wheel or the trackpad over a list: its view moves by as much as the fingers did, a
    /// pixel at a time (a line is `line_h` pixels); a mouse wheel's notch is three lines.
    fn pan(&mut self, id: &str, e: &ScrollWheelEvent, line_h: f32, cx: &mut Context<Self>) {
        // The list takes the wheel; what it can't take goes on to its scroller in the state.
        cx.stop_propagation();
        let lines = match e.delta {
            gpui::ScrollDelta::Lines(l) => -l.y * 3.0,
            gpui::ScrollDelta::Pixels(p) => -f32::from(p.y) / line_h.max(1.0),
        };
        if lines != 0.0 {
            self.send(Msg::Pan { id: id.to_string(), lines }, cx);
        }
    }

    fn wheel(&mut self, id: &str, e: &ScrollWheelEvent, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if self.wheel.0 != id {
            self.wheel = (id.to_string(), 0.0);
        }
        let step = 3.0 * self.cell.h;
        self.wheel.1 -= match e.delta {
            gpui::ScrollDelta::Lines(l) => l.y * step,
            gpui::ScrollDelta::Pixels(p) => f32::from(p.y),
        };
        let by = (self.wheel.1 / step).trunc();
        if by != 0.0 {
            self.wheel.1 -= by * step;
            self.send(Msg::Scroll { id: id.to_string(), by: by as isize }, cx);
        }
    }

    /// The mouse moved: a drag that started in an editor's text follows it anywhere in the
    /// window; otherwise what's under it is hovered (a node, or a row of one), told only when it
    /// changes, so its `.hover` parts hold. Under it means the innermost thing there: not a row a
    /// dialog covers.
    fn moved(&mut self, e: &MouseMoveEvent, cx: &mut Context<Self>) {
        // Files that left the window without being let go send no event of their own.
        if !e.dragging() && self.brought.is_some() {
            self.let_go(false, cx);
        }
        if e.dragging() && self.press.is_some() {
            return self.carry(e.position, cx);
        }
        if let Some((id, from, was, _)) = self.sizing.clone().filter(|_| e.dragging()) {
            let height = was + (f32::from(e.position.y) - from);
            if let Some(s) = self.sizing.as_mut() {
                s.3 = height;
            }
            return self.send(Msg::Resize { id, height, done: false }, cx);
        }
        if let Some(id) = self.drag.clone().filter(|_| e.dragging()) {
            if let Some((col, row)) = self.in_text(&id, e.position) {
                self.send(Msg::Drag { id, col, row }, cx);
            }
            return;
        }
        self.drag = None;
        let hover = self.shown.borrow().iter().rev().find(|(b, _)| b.contains(&e.position)).filter(|(_, h)| !h.id.is_empty()).map(|(_, h)| state::Hover { id: h.id.clone(), row: h.row });
        if hover != self.engine.hover() {
            self.send(Msg::Hover { hover }, cx);
        }
    }

    /// A press on a row of a list with a `drop`: a plain one waits to see whether it's a click or
    /// a drag; a modified one picks rows as it always does.
    fn row_down(&mut self, id: &str, k: usize, droppable: bool, e: &MouseDownEvent, cx: &mut Context<Self>) {
        let msg = row_press(id, k, e);
        if droppable && matches!(msg, Msg::Click { .. }) {
            self.press = Some(Pressed { id: id.to_string(), row: k, at: e.position, click: msg, lifted: false, over: None });
            return;
        }
        self.send(msg, cx);
    }

    /// The mouse moved with a row pressed: past `LIFT` pixels the row lifts, and then where it
    /// would land is the row under the mouse, on the side of it the mouse is on.
    fn carry(&mut self, at: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(p) = self.press.as_mut() else { return };
        if !p.lifted {
            let (dx, dy) = (f32::from(at.x - p.at.x), f32::from(at.y - p.at.y));
            if dx.hypot(dy) < LIFT {
                return;
            }
            p.lifted = true;
            let lift = Msg::Lift { id: p.id.clone(), row: p.row };
            self.send(lift, cx);
        }
        let Some(p) = self.press.as_ref() else { return };
        let across = self.per_line.get(&p.id).is_some_and(|c| *c > 1);
        let over = self.shown.borrow().iter().rev().find(|(b, h)| b.contains(&at) && h.id == p.id && droppable_on(h) && h.row.is_some()).map(|(b, h)| {
            // Over a group's heading: before the group's first row.
            let before = h.part.is_some() || if across { at.x < b.center().x } else { at.y < b.center().y };
            (h.row.unwrap_or_default(), if before { state::Side::Before } else { state::Side::After })
        });
        if over != p.over {
            if let Some(p) = self.press.as_mut() {
                p.over = over;
            }
            let (row, side) = over.map_or((None, state::Side::After), |(r, s)| (Some(r), s));
            self.send(Msg::Carry { row, side }, cx);
        }
    }

    /// Files dragged in from outside (Finder) moved to `at`: they'd land in the innermost list
    /// under the mouse that takes files, beside the row under it on the side the mouse is on,
    /// or after its last row over the rest of it.
    pub(super) fn bring(&mut self, files: Vec<String>, at: Point<Pixels>, cx: &mut Context<Self>) {
        let aim: Aim = {
            let shown = self.shown.borrow();
            let list = shown.iter().rev().filter(|(b, h)| b.contains(&at) && !h.id.is_empty() && droppable_on(h)).find(|(_, h)| self.engine.takes_files(&h.id));
            match list {
                Some((b, h)) => {
                    let across = self.per_line.get(&h.id).is_some_and(|c| *c > 1);
                    let side = |b: &Bounds<Pixels>| if h.part.is_some() || (across && at.x < b.center().x) || (!across && at.y < b.center().y) { state::Side::Before } else { state::Side::After };
                    (Some(h.id.clone()), h.row.map(|r| (r, side(b))))
                }
                None => (None, None),
            }
        };
        if self.brought.as_ref() == Some(&aim) {
            return;
        }
        self.brought = Some(aim.clone());
        let (id, over) = aim;
        let (row, side) = over.map_or((None, state::Side::After), |(r, s)| (Some(r), s));
        self.send(Msg::Bring { id, files, row, side }, cx);
    }

    /// Files brought in are let go (`drop`), or have left the window: either way the drag ends,
    /// and let go over a list that takes them, its `drop.files` runs.
    pub(super) fn let_go(&mut self, drop: bool, cx: &mut Context<Self>) {
        if self.brought.take().is_none() {
            return;
        }
        self.send(if drop { Msg::Release } else { Msg::Bring { id: None, files: vec![], row: None, side: state::Side::After } }, cx);
    }

    /// The button came up: a press that never moved is the click it was; a drag lets go.
    fn up(&mut self, cx: &mut Context<Self>) {
        self.drag = None;
        if let Some((id, _, _, height)) = self.sizing.take() {
            self.send(Msg::Resize { id, height, done: true }, cx);
        }
        if let Some(p) = self.press.take() {
            self.send(if p.lifted { Msg::Release } else { p.click }, cx);
        }
    }

    /// Row `k` of `n` as a drag shows it: dimmed (the `drag.ghost` part's opacity) while it's
    /// carried, and with a bar (the `drop.hover` part) on the side carried rows would go, marked
    /// with the place they'd have in the list, counted from 1 (the `drop.index` part).
    fn carried(&self, st: &State, lk: &Look, n: &Node, k: usize, across: bool, row: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
        // A row a drop put there before the app answered (`drop.optimistic`): the `row.pending` look.
        let row = if state::items(st, n).get(k).and_then(|r| r.get("$pending")) == Some(&Value::Bool(true)) {
            let look = style::inner(st, "row.pending", &[]);
            let row = lk.boxed(row, &look, (self.cell.w, self.cell.h), [0.; 4]);
            match look.opacity {
                Some(o) => row.opacity(o),
                None => row,
            }
        } else {
            row
        };
        let Some(d) = st.drag.as_ref().filter(|d| n.id.as_deref() == Some(d.id.as_str())) else { return row };
        let carried = state::row_key(st, n, k).is_some_and(|key| d.keys.contains(&key));
        let row = match style::inner(st, "drag.ghost", &[]).opacity.filter(|_| carried) {
            Some(o) => row.opacity(o),
            None => row,
        };
        let Some(side) = d.over.as_ref().filter(|o| o.row == k).map(|o| o.side) else { return row };
        let bar = lk.boxed(div().absolute(), &style::inner(st, "drop.hover", &[]), (self.cell.w, self.cell.h), [0.; 4]);
        let w = px(3.);
        let bar = match (across, side) {
            (false, state::Side::Before) => bar.top_0().left_0().right_0().h(w),
            (false, state::Side::After) => bar.bottom_0().left_0().right_0().h(w),
            (true, state::Side::Before) => bar.top_0().bottom_0().left_0().w(w),
            (true, state::Side::After) => bar.top_0().bottom_0().right_0().w(w),
        };
        // The place the first one carried (or brought) would have: rows carried from before
        // the bar leave their places.
        let at = if side == state::Side::Before { k } else { k + 1 };
        let gone = (0..at).filter(|i| state::row_key(st, n, *i).is_some_and(|key| d.keys.contains(&key))).count();
        let mark = lk.boxed(div().absolute().text_xs().line_height(px(16.)), &style::inner(st, "drop.index", &[]), (self.cell.w, self.cell.h), [0.; 4]).child(SharedString::from(format!("{}", at - gone + 1)));
        let mark = match (across, side) {
            (false, state::Side::Before) => mark.top_0().right_0(),
            (false, state::Side::After) => mark.bottom_0().right_0(),
            (true, state::Side::Before) => mark.top_0().left(px(4.)),
            (true, state::Side::After) => mark.top_0().right(px(4.)),
        };
        row.child(bar).child(mark)
    }

    /// The cell `at` falls on in editor `id`'s text, in the last painted frame.
    fn in_text(&self, id: &str, at: Point<Pixels>) -> Option<(u16, u16)> {
        self.shown.borrow().iter().rev().find(|(_, h)| h.text && h.id == id).map(|(b, _)| self.cell.within(*b, at))
    }
}

/// A GPUI keystroke as a fictty key name (`a`, `A`, `s-left`, `d-s-z`, `c-a`).
fn key_name(e: &KeyDownEvent) -> Option<String> {
    let k = &e.keystroke;
    let m = &k.modifiers;
    let base = match k.key.as_str() {
        "backspace" => "bs".to_string(),
        "delete" => "del".into(),
        "escape" => "esc".into(),
        "pageup" => "pgup".into(),
        "pagedown" => "pgdn".into(),
        "space" if !(m.control || m.alt || m.platform || m.shift) => "space".into(),
        "space" => "space".into(),
        "enter" | "tab" | "left" | "right" | "up" | "down" | "home" | "end" => k.key.clone(),
        f if f.starts_with('f') && f.len() > 1 && f[1..].chars().all(|c| c.is_ascii_digit()) => f.into(),
        // A plain character: what it typed (`A`, `!`), with no Shift of its own.
        _ if !(m.control || m.alt || m.platform) => return k.key_char.clone().or_else(|| Some(k.key.clone())),
        other => other.to_lowercase(),
    };
    let mut name = String::new();
    for (on, p) in [(m.control, "c-"), (m.alt, "a-"), (m.platform, "d-"), (m.shift, "s-")] {
        if on {
            name.push_str(p);
        }
    }
    name.push_str(&base);
    Some(name)
}

/// A node's share of its row or column, as flexbox: cells and pixels a basis it may shrink from
/// (as a terminal's layout gives a fixed size up when there isn't room), `30%` a fraction, `*`
/// and `2*` a grow factor; `min` and `max` bound it, and `margin` keeps it off its neighbours,
/// as lengths.
fn sized(el: gpui::Stateful<gpui::Div>, n: &Node, along_row: bool, loose: bool, cell: Cell, th: &style::Theme) -> gpui::Stateful<gpui::Div> {
    let per = if along_row { cell.w } else { cell.h };
    let fixed = |mut el: gpui::Stateful<gpui::Div>, l: DefiniteLength| {
        el.style().flex_grow = Some(0.);
        el.flex_shrink().flex_basis(l).min_w(px(0.)).min_h(px(0.))
    };
    // As big as what it shows, giving way when there isn't room.
    let fit = |el: gpui::Stateful<gpui::Div>| {
        let mut el = el.flex_shrink().min_w(px(0.)).min_h(px(0.));
        el.style().flex_grow = Some(0.);
        el
    };
    let mut el = match n.size.as_ref().map(Size::measure) {
        Some(Measure::Cells(c)) => fixed(el, px(c as f32 * per).into()),
        Some(Measure::Px(p)) => fixed(el, px(p).into()),
        Some(Measure::Percent(p)) => fixed(el, relative(p / 100.0)),
        // In a scroller there's no room to share along it: as long as what it shows.
        Some(Measure::Share(_)) | None if loose => fit(el),
        Some(Measure::Share(w)) => {
            let mut el = el.flex_basis(px(0.)).min_w(px(0.)).min_h(px(0.)).flex_shrink();
            el.style().flex_grow = Some(w as f32);
            el
        }
        // An aspect with no size is as long as how big it is across makes it.
        Some(Measure::Fit) => fit(el),
        None if n.aspect.is_some() => fit(el),
        None => el.flex_1().min_w(px(0.)).min_h(px(0.)),
    };
    if let Some(m) = n.min.as_ref().and_then(|m| th.px(m, per)) {
        el = if along_row { el.min_w(px(m)) } else { el.min_h(px(m)) };
    }
    if let Some(m) = n.max.as_ref().and_then(|m| th.px(m, per)) {
        el = if along_row { el.max_w(px(m)) } else { el.max_h(px(m)) };
    }
    let [t, r, b, l] = sides(th, n.margin.as_ref(), cell);
    if [t, r, b, l] != [0.; 4] {
        el = el.mt(px(t)).mr(px(r)).mb(px(b)).ml(px(l));
    }
    el.style().align_self = n.align_self.as_ref().and_then(Filled::get).map(across);
    el
}

/// A slot on a line of columns, as wide as `tracks` (a column, or the columns a child spans) and
/// the `gap`s between them: its cells and pixels as a basis, its shares growing into what's left.
/// Every line's slots add up to the same columns, so each grows alike and they line up. A slot
/// lays its child out as a column would. A percentage or `fit` is no column size (a problem), so
/// it's taken as a share.
fn track_slot(tracks: &[Measure], gap: f32, cell: Cell) -> gpui::Div {
    let (mut basis, mut grow) = (gap * tracks.len().saturating_sub(1) as f32, 0.);
    for t in tracks {
        match *t {
            Measure::Cells(c) => basis += c as f32 * cell.w,
            Measure::Px(p) => basis += p,
            Measure::Share(w) => grow += w as f32,
            Measure::Percent(_) | Measure::Fit => grow += 1.,
        }
    }
    let mut slot = div().flex().flex_col().flex_basis(px(basis)).flex_shrink().min_w(px(0.)).min_h(px(0.));
    slot.style().flex_grow = Some(grow);
    slot
}

/// A table column's slot on each of its lines: as wide as its `size` says (cells, `px`, a
/// percentage of the line, or a share of what's left, one by default), giving way when there's
/// no room; its text where `align` puts it.
fn column_slot(col: &crate::model::Column, cell: Cell) -> gpui::Div {
    let slot = div().flex().flex_row().items_center().min_w(px(0.)).overflow_hidden();
    let slot = match col.size.as_ref().map(|s| s.measure()) {
        Some(Measure::Cells(n)) => slot.flex_basis(px(n as f32 * cell.w)).flex_shrink(),
        Some(Measure::Px(p)) => slot.flex_basis(px(p)).flex_shrink(),
        Some(Measure::Percent(p)) => slot.flex_basis(relative(p / 100.)).flex_shrink(),
        Some(Measure::Share(w)) => {
            let mut slot = slot.flex_basis(px(0.)).flex_shrink();
            slot.style().flex_grow = Some(w as f32);
            slot
        }
        // `fit` is a problem for a column (its lines are laid out apart): one share.
        None | Some(Measure::Fit) => slot.flex_basis(px(0.)).flex_grow().flex_shrink(),
    };
    match col.align.as_deref() {
        Some("right") => slot.justify_end(),
        Some("center") => slot.justify_center(),
        _ => slot,
    }
}

/// `self` and `items` as flexbox's alignment across.
fn across(a: Align) -> gpui::AlignItems {
    match a {
        Align::Start => gpui::AlignItems::FlexStart,
        Align::Center => gpui::AlignItems::Center,
        Align::End => gpui::AlignItems::FlexEnd,
        Align::Stretch => gpui::AlignItems::Stretch,
        Align::Baseline => gpui::AlignItems::Baseline,
    }
}

/// `justify` as flexbox's along.
fn along(j: Justify) -> gpui::JustifyContent {
    match j {
        Justify::Start => gpui::JustifyContent::FlexStart,
        Justify::Center => gpui::JustifyContent::Center,
        Justify::End => gpui::JustifyContent::FlexEnd,
        Justify::Between => gpui::JustifyContent::SpaceBetween,
        Justify::Around => gpui::JustifyContent::SpaceAround,
    }
}

/// A press on row `k` of `id`: ⌘ (or ⌥) picks it in or out, ⇧ picks the range to it, and
/// otherwise it's a click that selects it.
fn row_press(id: &str, k: usize, e: &MouseDownEvent) -> Msg {
    let m = &e.modifiers;
    match (m.platform || m.alt, m.shift) {
        (true, _) => Msg::Pick { id: id.to_string(), row: k, how: state::Pick::Toggle },
        (false, true) => Msg::Pick { id: id.to_string(), row: k, how: state::Pick::Range },
        _ => Msg::Click { id: id.to_string(), row: Some(k), tab: None },
    }
}

/// Four sides in pixels, `[top, right, bottom, left]`: across in cells' widths, down in their
/// heights. A length that isn't one is nothing (and a problem).
fn sides(th: &style::Theme, l: Option<&Lens>, cell: Cell) -> [f32; 4] {
    let Some(l) = l else { return [0.; 4] };
    let [t, r, b, lf] = l.sides();
    let at = |l: &Len, per: f32| th.px(l, per).unwrap_or(0.);
    [at(t, cell.h), at(r, cell.w), at(b, cell.h), at(lf, cell.w)]
}

/// One side of a float, as a length: cells, pixels or a fraction of its stack; the whole stack
/// by default.
fn float_side(s: Option<&Size>, per: f32) -> DefiniteLength {
    match s.map(Size::measure) {
        Some(Measure::Cells(c)) => px(c as f32 * per).into(),
        Some(Measure::Px(p)) => px(p).into(),
        Some(Measure::Percent(p)) => relative(p / 100.0),
        None | Some(Measure::Share(_) | Measure::Fit) => relative(1.0),
    }
}

impl Portal {
    fn node(&mut self, st: &State, lk: &Look, n: &Node, along_row: bool, path: &str, cx: &mut Context<Self>) -> AnyElement {
        if !state::shown(st, n) {
            return div().into_any_element();
        }
        // A height its handle set stands for its `size` down a column (`min` and `max` still hold).
        let held;
        let n = match state::resized(st, n).filter(|_| !along_row) {
            Some(h) => {
                held = Node { size: Some(Size::Spec(format!("{h}px"))), ..n.clone() };
                &held
            }
            None => n,
        };
        let id = n.id.clone().unwrap_or_default();
        let key = if id.is_empty() { path.to_string() } else { id.clone() };
        let focused = !id.is_empty() && st.focus.as_deref() == Some(id.as_str());
        let loose = self.loose.take() == Some(along_row);
        let cell = self.cell;
        let th = style::Theme::of(st);
        // Everything clips what it holds, rows and columns too: a squeezed row's children would
        // spill over each other otherwise. A card's shadow is clipped where its row ends, so a
        // card takes a `margin` to give its shadow room.
        let el = div().id(SharedString::from(format!("n-{key}"))).relative().flex().flex_col().overflow_hidden();
        // A node with an `aspect` sits in a place of its own that takes its share, bounds,
        // margin and `self`, and it's all of the side it's given there: its `size` along, else
        // the room across. Taffy turns a side the style gives into the other, never a side it
        // stretched the node to.
        let (mut el, place) = match n.aspect.as_ref().map(Aspect::ratio) {
            Some(ratio) => {
                let place = sized(div().id(SharedString::from(format!("place-{key}"))).flex().flex_col(), n, along_row, loose, cell, &th);
                let given = n.size.as_ref().is_some_and(|s| s.measure() != Measure::Fit);
                let mut el = if given == along_row { el.w(relative(1.)) } else { el.h(relative(1.)) }.flex_none();
                el.style().aspect_ratio = Some(ratio);
                (el, Some(place))
            }
            None => (sized(el, n, along_row, loose, cell, &th), None),
        };
        // Where it was laid out, for the readback and the mouse: noted before its children, so
        // the innermost hit comes last.
        if !id.is_empty() {
            let id = id.clone();
            el = el.child(measure(&self.laid, move |l, b| {
                l.rects.push((id.clone(), b));
                l.hits.push((b, hit(id, None, None, false)));
            }));
        }
        // The focus, wherever it is, so the scrollers around it can bring it into view.
        if focused {
            el = el.child(measure_whole(&self.laid, |l, b| l.focused = Some(b)));
        }
        let fits = n.size.as_ref().is_some_and(|s| s.measure() == Measure::Fit);
        self.fitting += fits as usize;
        let rs = style::resolve(st, n);
        let scope = state::scope(st, n);
        let v = scope.as_ref();
        let c = (cell.w, cell.h);
        // `pad` inside, over the style's own; a button's style is its face's.
        let extra = sides(&th, n.pad.as_ref(), cell);
        let button = matches!(n.kind, Kind::Button { .. });
        el = if button { padded(el, extra) } else { lk.boxed(el, &rs, c, extra) };
        el = match n.align.as_deref() {
            Some("center") => el.text_center(),
            Some("right") => el.text_right(),
            _ => el,
        };
        // Its type role, for its text and everything in it; `keep` holds room on a row's left.
        let role = rs.role.as_ref();
        el = text::typed(el, role, &lk.fonts());
        let laid = self.laid.clone();
        let keep = n.rows().and_then(|r| r.keep).map_or(0., |k| k as f32 * cell.w);
        let tx = text::Tx { lk, laid: &laid, id: &key, keep, line_h: text::line_height(role, cell.h), fitting: self.fitting > 0 };
        if let (Some(t), Some(ink)) = (&n.title, &rs.title) {
            el = el.gap_1().child(div().flex_none().w_full().text_sm().child(text::line(&tx, t, v, lk.ink(ink), false)));
        }
        let body: AnyElement = match &n.kind {
            Kind::Col { children, gap, justify, items, wrap } | Kind::Row { children, gap, justify, items, wrap } => {
                let row = matches!(n.kind, Kind::Row { .. });
                let scrolls = n.scroll && !id.is_empty();
                // A hidden child takes no room, its gap neither.
                let mut kids: Vec<AnyElement> = Vec::new();
                for (k, c) in children.iter().enumerate().filter(|(_, c)| state::shown(st, c)) {
                    self.loose = scrolls.then_some(row);
                    kids.push(self.node(st, lk, c, row, &format!("{key}.{k}"), cx));
                }
                self.loose = None;
                let [gx, gy] = gap.as_ref().map_or([0.; 2], |g| {
                    let [x, y] = g.xy();
                    [th.px(x, cell.w).unwrap_or(0.), th.px(y, cell.h).unwrap_or(0.)]
                });
                let d = div().relative().flex().min_h(px(0.)).min_w(px(0.)).gap_x(px(gx)).gap_y(px(gy));
                // A scroller's content is as long as what it holds, along it.
                let d = match (scrolls, row) {
                    (false, _) => d.size_full(),
                    (true, true) => d.h_full().flex_none(),
                    (true, false) => d.w_full().flex_none(),
                };
                let mut d = if row { d.flex_row() } else { d.flex_col() };
                (d.style().justify_content, d.style().align_items) = (justify.as_ref().and_then(Filled::get).map(along), items.as_ref().and_then(Filled::get).map(across));
                // Wrapped lines pack from the start, each as tall (or wide) as its biggest child.
                if *wrap {
                    d = d.flex_wrap().content_start();
                }
                // `$width` and `$height` in a child's `when` are this node's inner size.
                let sized: Vec<String> = children.iter().filter(|c| c.when.as_deref().is_some_and(state::sized)).filter_map(|c| c.id.clone()).collect();
                let d = if sized.is_empty() { d } else { d.child(measure(&self.laid, move |l, b| l.areas.extend(sized.into_iter().map(|id| (id, b))))) };
                let d = d.children(kids);
                if scrolls { self.scroller(st, lk, &id, !row, d, cx) } else { d.into_any_element() }
            }
            Kind::Grid { .. } => self.grid(st, lk, n, &id, &key, &th, cx),
            Kind::Stack { .. } => self.stack(st, lk, n, &key, cx),
            Kind::Text { text, wrap, lines } => text::lines(&tx, text, v, Ink::default(), wrap.unwrap_or(true), *lines),
            Kind::Spinner { text } => {
                let (glyph, next) = state::spinner(st.now_ms);
                self.soon(next);
                let ink = lk.ink(&style::inner(st, "spinner", &[]).ink);
                div().flex().flex_row().gap_2().child(inked(div(), ink).child(glyph)).child(text::line(&tx, text, v, Ink::default(), false)).into_any_element()
            }
            Kind::Button { label } => {
                // A tab right-aligns what follows it across the button, as in a text ("All
                // images\t101"), so the face is as wide as the button then.
                let tabbed = label.contains('\t');
                let label: AnyElement = if tabbed {
                    div().w_full().child(text::line(&tx, label, v, Ink::default(), false)).into_any_element()
                } else {
                    text::plain(label, v).into_any_element()
                };
                let target = id.clone();
                // The face, as wide as its label, where `align` puts it, down the middle.
                el = match n.align.as_deref() {
                    Some("center") => el.items_center(),
                    Some("right") => el.items_end(),
                    _ => el.items_start(),
                }
                .justify_center();
                // In a list's row, it presses on that row by its path in the row template.
                let press = self.row.clone().filter(|_| id.is_empty() && n.click.is_some()).map(|(list, k)| {
                    let part = key.strip_prefix(&format!("{list}/row{k}")).unwrap_or_default().trim_start_matches('.').to_string();
                    (list, k, part)
                });
                let face = match &press {
                    Some((list, k, part)) => {
                        let under = Hit { part: Some(part.clone()), ..hit(list.clone(), Some(*k), None, false) };
                        Some(measure(&self.laid, move |l, b| l.hits.push((b, under))))
                    }
                    None => (!id.is_empty()).then(|| {
                        let under = id.clone();
                        measure(&self.laid, move |l, b| l.hits.push((b, hit(under, None, None, false))))
                    }),
                };
                let msg = match press {
                    Some((id, row, part)) => Msg::Press { id, row, part },
                    None => Msg::Click { id: target.clone(), row: None, tab: None },
                };
                // As tall as its row, the label in the middle, so a border stays inside the row.
                let face_box = div().id(SharedString::from(format!("btn-{key}"))).relative().flex_none().h_full().flex().items_center();
                lk.boxed(if tabbed { face_box.w_full() } else { face_box }, &rs, c, [0.; 4])
                    .children(face)
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.send(msg.clone(), cx)
                    }))
                    .child(label)
                    .into_any_element()
            }
            Kind::Rule { .. } => {
                let colour = lk.ink(&rs.ink).fg.unwrap_or(lk.border);
                // Down the middle when it's taller than wide (in a row), else across, as the
                // terminal picks its glyph; a pixel thick either way.
                canvas(|_, _, _| {}, move |b, _, window, _| {
                    let line = if b.size.height > b.size.width {
                        Bounds::new(gpui::point(b.origin.x + (b.size.width - px(1.)) / 2., b.origin.y), size(px(1.), b.size.height))
                    } else {
                        Bounds::new(gpui::point(b.origin.x, b.origin.y + (b.size.height - px(1.)) / 2.), size(b.size.width, px(1.)))
                    };
                    window.paint_quad(gpui::fill(line, colour));
                })
                .size_full()
                .into_any_element()
            }
            Kind::List { .. } | Kind::Tree { .. } => self.list(st, lk, n, &id, fits || loose, cx),
            Kind::Table { .. } => self.table(st, lk, n, &id, &key, tx, cx),
            Kind::Editor { .. } | Kind::Input { .. } => self.editor(st, lk, n, &id, focused, cx),
            Kind::Tabs { .. } => self.tabs(st, lk, n, &id, &key, &rs, v, cx),
            Kind::Toasts {} => {
                if let Some(at) = st.toasts.iter().map(|t| t.until).min() {
                    self.soon(at);
                }
                let ts: Vec<AnyElement> = st.toasts.iter().rev().map(|t| toast(st, lk, t, c)).collect();
                div().flex().flex_col().gap_1().children(ts).into_any_element()
            }
            Kind::Image { fit, alt, .. } => {
                let fit = match fit.unwrap_or_default() {
                    Fit::Contain => ObjectFit::Contain,
                    Fit::Cover => ObjectFit::Cover,
                };
                match state::image_src(st, n) {
                    Some(src) => {
                        let (dim, why) = (lk.dim, alt.clone().unwrap_or_else(|| format!("[image {src}]")));
                        img(std::path::PathBuf::from(src))
                            .size_full()
                            .object_fit(fit)
                            .with_fallback(move || div().text_color(dim).child(why.clone()).into_any_element())
                            .into_any_element()
                    }
                    None => div().into_any_element(),
                }
            }
            Kind::Gauge { value, of, label } => {
                // The node's own box is the track; the `gauge.fill` part fills it from the start.
                let ratio = crate::model::gauge_ratio(value, of.as_ref(), v) as f32;
                let bar = lk.boxed(div().flex_none().h_full().w(relative(ratio)), &style::inner(st, "gauge.fill", &[]), c, [0.; 4]);
                let track = div().relative().size_full().flex().flex_row().child(bar);
                match label {
                    Some(l) => track
                        .child(div().absolute().inset_0().flex().items_center().justify_center().child(text::line(&tx, l, v, Ink::default(), false)))
                        .into_any_element(),
                    None => track.into_any_element(),
                }
            }
            _ => div().text_color(lk.dim).child(format!("[{}]", n.kind_name())).into_any_element(),
        };
        self.fitting -= fits as usize;
        el = el.child(body);
        if n.resize.is_some() && !id.is_empty() && !along_row {
            el = el.child(self.handle(st, lk, &id, cx));
        }
        // A press on a node with an id that nothing inside it took (its rows, tabs, text and
        // buttons take their own): the core passes it on to the nearest `click`.
        if !id.is_empty() {
            let target = id.clone();
            el = el.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.send(Msg::Click { id: target.clone(), row: None, tab: None }, cx)
                }),
            );
        }
        match place {
            Some(place) => place.child(el).into_any_element(),
            None => el.into_any_element(),
        }
    }

    /// A node floating in its stack where its `float` says: its own size (the stack's by default),
    /// at a side, a corner or the middle, `margin` in from the edges. It covers what's under
    /// it, the mouse too.
    fn float(&mut self, st: &State, lk: &Look, n: Node, fl: &Float, path: &str, cx: &mut Context<Self>) -> AnyElement {
        let cell = self.cell;
        let at = fl.at.as_deref().unwrap_or("center");
        let wrap = padded(div().absolute().inset_0().flex(), sides(&style::Theme::of(st), fl.margin.as_ref(), cell));
        let wrap = if at.contains("left") {
            wrap.justify_start()
        } else if at.contains("right") {
            wrap.justify_end()
        } else {
            wrap.justify_center()
        };
        let wrap = if at.contains("top") {
            wrap.items_start()
        } else if at.contains("bottom") {
            wrap.items_end()
        } else {
            wrap.items_center()
        };
        // `x` and `y` put its edge that far in from the stack's, where `at` would place it.
        let th = style::Theme::of(st);
        let wrap = match fl.x.as_ref().and_then(|x| th.px(x, cell.w)) {
            Some(x) => wrap.justify_start().pl(px(x)),
            None => wrap,
        };
        let wrap = match fl.y.as_ref().and_then(|y| th.px(y, cell.h)) {
            Some(y) => wrap.items_start().pt(px(y)),
            None => wrap,
        };
        let (w, h) = (float_side(fl.width.as_ref(), cell.w), float_side(fl.height.as_ref(), cell.h));
        // Its float sizes it, not `size`: it fills the box.
        let n = Node { size: None, min: None, max: None, ..n };
        // It covers what's under it with the screen's background, unless it has one of its own:
        // then only that, so a colour with alpha (`"#24252299"`, a scrim over a picture) lets
        // what's under it show.
        let own = style::resolve(st, &n);
        let covers = own.ink.bg.is_none() && own.gradient.is_none();
        let shown = self.node(st, lk, &n, false, path, cx);
        let face = div().id(SharedString::from(format!("float-{path}"))).flex().flex_col().flex_none().w(w).h(h).max_w_full().max_h_full().when(covers, |d| d.bg(lk.bg)).occlude().child(shown);
        wrap.child(face).into_any_element()
    }

    /// A stack: its children over each other, a float where it says, a modal over a scrim.
    /// (This and the next two are methods of their own so `node`'s frame stays small: it
    /// recurses as deep as the tree.)
    fn stack(&mut self, st: &State, lk: &Look, n: &Node, key: &str, cx: &mut Context<Self>) -> AnyElement {
        let Kind::Stack { children } = &n.kind else { unreachable!("a stack") };
        let shown: Vec<(usize, Node)> = children.iter().enumerate().filter(|(_, c)| state::shown(st, c)).map(|(k, c)| (k, c.clone())).collect();
        let mut kids: Vec<AnyElement> = Vec::new();
        // Sized to `fit`, the first child that neither floats nor is a dialog is the stack's
        // base: in the flow, so the stack is as big as it is; the rest go over it. Any other
        // stack has every child over its own box.
        let fits = n.size.as_ref().is_some_and(|s| s.measure() == Measure::Fit);
        let base = shown.iter().find(|(_, c)| fits && c.float.is_none() && !c.modal).map(|(k, _)| *k);
        for (k, c) in shown {
            let path = format!("{key}.{k}");
            if c.modal {
                // What's behind a dialog dims, and a press there goes nowhere.
                kids.push(
                    div()
                        .id(SharedString::from(format!("scrim-{path}")))
                        .absolute()
                        .inset_0()
                        .bg(lk.scrim)
                        .occlude()
                        .child(measure(&self.laid, |l, b| l.hits.push((b, hit(String::new(), None, None, false)))))
                        .into_any_element(),
                );
            }
            kids.push(match c.float.clone() {
                Some(fl) => self.float(st, lk, c, &fl, &path, cx),
                None if base == Some(k) => {
                    // Down, as a `col` holds it, so a picture's `aspect` gives the height.
                    let shown = self.node(st, lk, &c, false, &path, cx);
                    div().relative().w_full().flex().flex_col().child(shown).into_any_element()
                }
                None => {
                    let covers = c.modal;
                    let shown = self.node(st, lk, &c, false, &path, cx);
                    div().absolute().inset_0().flex().when(covers, |d| d.bg(lk.bg).occlude()).child(shown).into_any_element()
                }
            });
        }
        div().relative().size_full().children(kids).into_any_element()
    }

    /// A table: its header over its rows, as many as it has room for.
    #[allow(clippy::too_many_arguments)]
    /// A table: a header, its rows, and a footer of totals when a column has one, every line
    /// on the same columns (`size` in any unit but `fit`, a cell between them, `align`). A cell
    /// is its column's `value` text, or its `cell` template's nodes; `heat` tints it from the
    /// theme's `neg` to `pos`.
    #[allow(clippy::too_many_arguments)]
    fn table(&mut self, st: &State, lk: &Look, n: &Node, id: &str, key: &str, tx: text::Tx, cx: &mut Context<Self>) -> AnyElement {
        let Kind::Table { columns, .. } = &n.kind else { unreachable!("a table") };
        let cell = self.cell;
        let c = (cell.w, cell.h);
        let rows = state::items(st, n).to_vec();
        let columns = columns.clone();
        let line = |d: gpui::Div| d.flex().flex_row().gap(px(cell.w)).h(px(cell.h));
        // The header and the footer sit over the rows' text: the rows' pad, their own look.
        let [_, row_r, _, row_l] = Look::pad(&style::inner_in(st, n, "row", &[]), c);
        let head = lk.boxed(line(div()), &style::inner(st, "table.header", &[]), c, [0., row_r, 0., row_l]).children(columns.iter().map(|col| column_slot(col, cell).child(col.title.clone())));
        let mut body: Vec<AnyElement> = Vec::new();
        for k in shown_rows(st, n).collect::<Vec<_>>() {
            let r = &rows[k];
            let (target, under) = (id.to_string(), id.to_string());
            let mut cells: Vec<AnyElement> = Vec::new();
            for (ci, col) in columns.iter().enumerate() {
                let shows = match &col.cell {
                    // A cell template's nodes, its buttons the row's parts: the column, then the path.
                    Some(t) => {
                        let was = self.row.replace((id.to_string(), k));
                        let built = self.node(st, lk, &crate::model::row_instance(t, r, (k, rows.len())), false, &format!("{id}/row{k}.{ci}"), cx);
                        self.row = was;
                        built
                    }
                    None => text::line(&tx, &col.value, r, Ink::default(), false),
                };
                let tint = col.heat.as_ref().and_then(|h| h.at(r)).and_then(|t| look::heat(st, t));
                cells.push(column_slot(col, cell).when_some(tint, |d, t| d.bg(t)).child(shows).into_any_element());
            }
            let droppable = n.drop.is_some();
            let row = lk
                .boxed(line(div()).id(k).relative(), &style::inner_in(st, n, "row", &style::row_states(st, n, k)), c, [0.; 4])
                .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, Some(k), None, false)))))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.row_down(&target, k, droppable, e, cx)
                }))
                .children(cells);
            body.push(self.carried(st, lk, n, k, false, row).into_any_element());
        }
        // Each total under its column, a negative one in `neg`.
        let foot = columns.iter().any(|col| col.total.is_some()).then(|| {
            let neg = style::Theme::of(st).colour("neg").and_then(|h| look::parse(&h));
            let cells = crate::model::table_footer(&columns, &rows);
            lk.boxed(line(div().flex_none()), &style::inner(st, "table.total", &[]), c, [0., row_r, 0., row_l]).children(columns.iter().zip(cells).map(|(col, t)| {
                let below = col.total(&rows).is_some_and(|(sum, _)| sum < 0.);
                column_slot(col, cell).when(below, |d| d.when_some(neg, |d, n| d.text_color(n))).child(t)
            }))
        });
        // The rows it has room for, below the header.
        let fits = id.to_string();
        let wheel = id.to_string();
        let rows_area = div()
            .id(SharedString::from(format!("rows-{key}")))
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .overflow_hidden()
            .child(measure_whole(&self.laid, move |l, b| l.sizes.push((fits, b))))
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.wheel(&wheel, e, cx)))
            .children(body);
        div().flex().flex_col().size_full().child(head).child(rows_area).children(foot).into_any_element()
    }

    /// A tab bar over the shown tab, in the shape the theme gives it.
    #[allow(clippy::too_many_arguments)]
    fn tabs(&mut self, st: &State, lk: &Look, n: &Node, id: &str, key: &str, rs: &style::Resolved, v: &Value, cx: &mut Context<Self>) -> AnyElement {
        let Kind::Tabs { tabs, lead, trail, .. } = &n.kind else { unreachable!("tabs") };
        let c = (self.cell.w, self.cell.h);
        let at = state::tab_of(st, n).min(tabs.len().saturating_sub(1));
        let tabs = tabs.clone();
        let (mut active, mut idle) = (style::inner(st, "tabs.selected", &[]), style::inner(st, "tabs.idle", &[]));
        // The shape the theme gives the tabs: pills (the default), an underline under the
        // shown one, segments joined in one box, or plain text.
        let shape = rs.shape.as_deref().unwrap_or("pill");
        let line = style::inner(st, "tabs.underline", &[]).ink.fg.or_else(|| active.ink.bg.clone());
        if matches!(shape, "underline" | "plain") {
            for t in [&mut active, &mut idle] {
                t.ink.bg = None;
                t.gradient = None;
                t.radius = None;
            }
            active.ink.fg = line.clone();
        }
        let mut bar = div().flex().flex_row().gap_2();
        if shape == "segment" {
            let mut frame = style::inner(st, "panel", &[]);
            frame.pad = None;
            frame.border = style::inner(st, "input", &[]).border;
            bar = lk.boxed(bar.gap_0().flex_none().overflow_hidden(), &frame, c, [0.; 4]);
            active.radius = None;
            idle.radius = None;
        }
        let underline = (shape == "underline").then(|| line.as_deref().and_then(look::parse)).flatten();
        let child = n.page(at).cloned();
        let (lead, trail) = (lead.as_deref().map(|t| text::whole(lk, t, v, Ink::default())), trail.as_deref().map(|t| text::whole(lk, t, v, Ink::default())));
        let bar = bar.children(tabs.iter().enumerate().map(|(k, t)| {
            let target = id.to_string();
            let under = id.to_string();
            lk.boxed(div().id(SharedString::from(format!("tab-{key}-{k}"))).relative(), if k == at { &active } else { &idle }, c, [0.; 4])
                .when_some(underline.filter(|_| k == at), |d, u| d.border_b_2().border_color(u))
                .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, None, Some(k), false)))))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.send(Msg::Click { id: target.clone(), row: None, tab: Some(k) }, cx)
            }))
                .child(text::plain(t, v))
        }));
        let shown = child.map(|c| self.node(st, lk, &c, false, &format!("{key}.{at}"), cx)).unwrap_or_else(|| div().into_any_element());
        // In a row of its own, so a segmented bar is as wide as its tabs.
        div().flex().flex_col().size_full().gap_1().child(div().flex().flex_row().items_center().gap_4().children(lead).child(bar).child(div().flex_1()).children(trail)).child(div().flex().flex_1().min_h(px(0.)).child(shown)).into_any_element()
    }

    /// A grid: lines of columns. The core places each child (`grid_lines`), and each line is a
    /// row of slots, one a column or a child's span of them, the same sizes on every line, so the
    /// columns line up. A slot lays its child out as a column would. (Its own method, as
    /// `list` is, so `node`'s frame stays small: it recurses as deep as the tree.)
    #[allow(clippy::too_many_arguments)]
    fn grid(&mut self, st: &State, lk: &Look, n: &Node, id: &str, key: &str, th: &style::Theme, cx: &mut Context<Self>) -> AnyElement {
        let Kind::Grid { children, cols, gap, items } = &n.kind else { unreachable!("a grid") };
        let cell = self.cell;
        let kids: Vec<(usize, &Node)> = children.iter().enumerate().filter(|(_, c)| state::shown(st, c)).collect();
        let tracks = cols.tracks(state::per_line(st, n));
        let [gx, gy] = gap.as_ref().map_or([0.; 2], |g| {
            let [x, y] = g.xy();
            [th.px(x, cell.w).unwrap_or(0.), th.px(y, cell.h).unwrap_or(0.)]
        });
        let spans: Vec<usize> = kids.iter().map(|(_, c)| c.span.unwrap_or(1) as usize).collect();
        let mut lines = Vec::new();
        for line in crate::model::grid_lines(&spans, tracks.len()) {
            let end = line.last().map_or(0, |(_, from, span)| from + span);
            let mut slots: Vec<AnyElement> = Vec::new();
            for (i, from, span) in line {
                let (k, c) = kids[i];
                let shown = self.node(st, lk, c, false, &format!("{key}.{k}"), cx);
                slots.push(track_slot(&tracks[from..from + span], gx, cell).child(shown).into_any_element());
            }
            slots.extend((end..tracks.len()).map(|t| track_slot(&tracks[t..t + 1], gx, cell).into_any_element()));
            let mut line = div().flex().flex_row().flex_none().w_full().gap_x(px(gx));
            line.style().align_items = items.as_ref().and_then(Filled::get).map(across);
            lines.push(line.children(slots).into_any_element());
        }
        // The width it was laid out across, for `cols` that fit as many as there's room for.
        let width = (!id.is_empty() && matches!(cols, crate::model::Cols::Fill { .. })).then(|| {
            let fits = id.to_string();
            measure_whole(&self.laid, move |l, b| l.sizes.push((fits, b)))
        });
        let sized: Vec<String> = children.iter().filter(|c| c.when.as_deref().is_some_and(state::sized)).filter_map(|c| c.id.clone()).collect();
        let areas = (!sized.is_empty()).then(|| measure(&self.laid, move |l, b| l.areas.extend(sized.into_iter().map(|id| (id, b)))));
        let d = div().relative().flex().flex_col().min_h(px(0.)).min_w(px(0.)).gap_y(px(gy)).children(width).children(areas).children(lines);
        // Its lines are as tall as what they hold, so a scrolling grid's content is too.
        if n.scroll && !id.is_empty() { self.scroller(st, lk, id, true, d.w_full().flex_none(), cx) } else { d.size_full().overflow_hidden().into_any_element() }
    }

    /// A scrolling container: `content` (as long as what it holds, along it) in a view that
    /// clips it, moved back by the state's offset. It tells the engine how long the view and
    /// the content are, takes the wheel (what it can't take goes to the scroller around it, in
    /// the state), and shows a scrollbar while it holds more than it shows.
    fn scroller(&mut self, st: &State, lk: &Look, id: &str, down: bool, content: gpui::Div, cx: &mut Context<Self>) -> AnyElement {
        let cell = self.cell;
        let along = if down { cell.h } else { cell.w };
        let local = st.local.get(id);
        let offset = local.map_or(0., |l| l.offset) * along;
        let (held, seen, wheel) = (id.to_string(), id.to_string(), id.to_string());
        let content = if down { content.mt(px(-offset)) } else { content.ml(px(-offset)) }.child(measure_whole(&self.laid, move |l, b| l.contents.push((held, b))));
        let bar = match local.and_then(|l| l.extent) {
            Some((view, held)) if down => self.scrollbar(st, lk, offset / along, held, view),
            _ => None,
        };
        let view = div().id(SharedString::from(format!("scroll-{id}"))).relative().flex().size_full().min_h(px(0.)).min_w(px(0.)).overflow_hidden();
        let view = if down { view.flex_col() } else { view.flex_row() };
        view.child(measure_whole(&self.laid, move |l, b| l.views.push((seen, down, b))))
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| {
                let px_moved = match e.delta {
                    gpui::ScrollDelta::Lines(l) => -(if down || l.x == 0. { l.y } else { l.x }) * 3.0 * along,
                    gpui::ScrollDelta::Pixels(p) => -f32::from(if down || p.x == px(0.) { p.y } else { p.x }),
                };
                if px_moved != 0. {
                    cx.stop_propagation();
                    this.send(Msg::Pan { id: wheel.clone(), lines: px_moved / along }, cx);
                }
            }))
            .child(content)
            .children(bar)
            .into_any_element()
    }

    /// A tree's row: its indent, its fold glyph (a press on it folds or opens the row, its hit
    /// the row's with the part `fold`), then what the row shows beside them.
    #[allow(clippy::too_many_arguments)]
    fn tree_row(&self, st: &State, lk: &Look, id: &str, k: usize, r: &Value, leaf: Option<&str>, body: AnyElement, cx: &mut Context<Self>) -> AnyElement {
        let cell = self.cell;
        let (depth, glyph) = crate::model::fold_lead(r, leaf);
        // The guide's look, but a selected row's own ink, as the terminal drew it.
        let guide = if state::selected(st, id) == k { style::Resolved::default() } else { style::inner(st, "tree.guide", &[]) };
        let under = Hit { part: Some("fold".into()), ..hit(id.to_string(), Some(k), None, false) };
        let target = id.to_string();
        let fold = lk
            .boxed(div().flex_none().relative(), &guide, (cell.w, cell.h), [0.; 4])
            .child(measure(&self.laid, move |l, b| l.hits.push((b, under))))
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.send(Msg::Press { id: target.clone(), row: k, part: "fold".into() }, cx)
            }))
            .child(glyph.to_string());
        div().flex().flex_row().items_start().w_full().pl(px(depth as f32 * 2. * cell.w)).child(fold).child(div().flex_1().min_w_0().child(body)).into_any_element()
    }

    /// A list's rows, from where the state says it's scrolled: as many as fit, each the rows its
    /// item template takes. The wheel scrolls the state, as the terminal's does.
    /// `whole`: it's as long as its lines (`fit`, or in a scroller along it), not the room it's
    /// given.
    fn list(&mut self, st: &State, lk: &Look, n: &Node, id: &str, whole: bool, cx: &mut Context<Self>) -> AnyElement {
        let (Kind::List { rows: look, .. } | Kind::Tree { rows: look, .. }) = &n.kind else { unreachable!("a list or tree") };
        let cell = self.cell;
        let rows = state::items(st, n);
        let per = look.height();
        let lines = state::Lines::of(st, n);
        let cols = lines.cols;
        // `fit` rows are as tall as what they show, measured as they're built; others the
        // template's height.
        let fits = look.fits();
        let row_h = (!fits).then(|| px(cell.h * per as f32));
        let tracks = look.cols.as_ref().map_or_else(|| vec![Measure::Share(1)], |c| c.tracks(cols));
        let laid = self.laid.clone();
        self.per_line.insert(id.to_string(), cols);
        let droppable = n.drop.is_some();
        if rows.is_empty() {
            return lk.boxed(div(), &style::inner(st, "empty", &[]), (cell.w, cell.h), [0., 8., 0., 8.]).child(look.empty.clone().unwrap_or_else(|| "(nothing)".into())).into_any_element();
        }
        // Where the view is, to the pixel: the part of its first line scrolled past moves the
        // rows up by that much.
        let at = lines.offset(st.local.get(id));
        let mut shown: Vec<AnyElement> = Vec::new();
        for l in lines.shown(at) {
            let on_line = match lines.line(l) {
                state::Line::Heading { first, rows: count } => {
                    if let Some(s) = look.section.as_ref() {
                        let heading = self.heading(st, lk, n, id, s, (first, count), cx);
                        shown.push(heading);
                    }
                    continue;
                }
                state::Line::Rows { from, to } => from..to,
            };
            let mut line: Vec<AnyElement> = Vec::new();
            for k in on_line.clone() {
                let r = &rows[k];
                // A row is its template's nodes, or its item's text.
                let body = match (&look.row, &look.item) {
                    (Some(t), _) => {
                        let was = self.row.replace((id.to_string(), k));
                        let built = self.node(st, lk, &crate::model::row_instance(t, r, (k, rows.len())), false, &format!("{id}/row{k}"), cx);
                        self.row = was;
                        built
                    }
                    (None, Some(t)) => {
                        let tx = text::Tx { lk, laid: &laid, id, keep: look.keep.map_or(0., |k| k as f32 * cell.w), line_h: cell.h, fitting: self.fitting > 0 };
                        text::lines(&tx, t, r, Ink::default(), false, None)
                    }
                    // A tree's row without a template shows its `text`.
                    (None, None) if matches!(n.kind, Kind::Tree { .. }) => div().child(crate::model::scalar(r.get("text").or(r.get("value")))).into_any_element(),
                    (None, None) => div().child(crate::model::scalar(Some(r))).into_any_element(),
                };
                let body = match &n.kind {
                    Kind::Tree { leaf, .. } => self.tree_row(st, lk, id, k, r, leaf.as_deref(), body, cx),
                    _ => body,
                };
                let (target, under) = (id.to_string(), id.to_string());
                // A focused list's place is its selected row: that's what a scroller brings into view.
                let here = (st.focus.as_deref() == Some(id) && k == state::selected(st, id)).then(|| measure_whole(&self.laid, |l, b| l.focused = Some(b)));
                let tall = fits.then(|| {
                    let (list, key) = (id.to_string(), state::height_key(n, r, k));
                    measure_whole(&self.laid, move |l, b| l.heights.push((list, key, b)))
                });
                let row = lk
                    .boxed(div().id(k).relative().flex_none(), &style::inner_in(st, n, "row", &style::row_states(st, n, k)), (cell.w, cell.h), [0.; 4])
                    .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, Some(k), None, false)))))
                    .children(here)
                    .children(tall)
                    .when_some(row_h, |d, h| d.h(h))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            this.row_down(&target, k, droppable, e, cx)
                        }),
                    )
                    .child(body);
                let row = self.carried(st, lk, n, k, cols > 1, row);
                // In its column's slot on the line, a plain box: a slot's own padding would count
                // before the shares, so a short last line's rows would come out wider. The slot is
                // the row's height, known, so the line isn't measured from what each row holds
                // (that measuring cost Nocturne's shelf a quarter of its frame).
                // A `fit` row's slot is its line's height as last measured, so the line isn't measured
                // from what's in it either; the row inside is its own height, and says so.
                let slot_h = row_h.unwrap_or_else(|| px(lines.height(l) * cell.h));
                let t = k - on_line.start;
                let row = if cols > 1 { track_slot(&tracks[t..t + 1], 0., cell).h(slot_h).child(row).into_any_element() } else { row.into_any_element() };
                line.push(row);
            }
            // Rows `cols` to a line, each line as tall as a row and each row an equal share of it.
            if cols > 1 {
                for t in line.len()..cols {
                    line.push(track_slot(&tracks[t..t + 1], 0., cell).into_any_element());
                }
                shown.push(div().flex().flex_row().flex_none().w_full().children(line).into_any_element());
            } else {
                shown.extend(line);
            }
        }
        let first = at.floor() as usize;
        let body = div().flex().flex_col().flex_none().w_full().mt(px(-(at - at.floor()) * lines.height(first) * cell.h)).children(shown);
        let bar = self.scrollbar(st, lk, at, lines.count as f32, lines.fitting_from(first) as f32);
        let (room, wheel) = (id.to_string(), id.to_string());
        // A `fit` list's rows (or a list's in a scroller) want room for all its lines (its `max`
        // and the room around it hold it to less), whatever the view built last; any other
        // list's take what it's given.
        let rows_box = div().id(SharedString::from(format!("rows-{id}"))).relative().flex().flex_col().min_h(px(0.)).overflow_hidden();
        let rows_box = if whole {
            // Whole cells, since the frame tells the view's height in whole cells: else a
            // list as tall as its lines is part of one short and scrolls by it.
            let mut b = rows_box.flex_basis(px(lines.total().ceil() * cell.h)).flex_shrink();
            b.style().flex_grow = Some(0.);
            b
        } else {
            rows_box.flex_1()
        };
        rows_box
            // The room it has for rows: its whole box, not the part a scroller around it shows
            // (scrolling that is the scroller's).
            .child(measure_whole(&self.laid, move |l, b| l.sizes.push((room, b))))
            // The wheel moves it by cells.
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.pan(&wheel, e, cell.h, cx)))
            .child(body)
            .children(bar)
            .into_any_element()
    }

    /// A list's group heading (its `section`), across the whole width before the group's first
    /// row, filled from that row (`$rows`: how many the group has). It takes no selection; rows
    /// dragged or files brought over it land before the group's first row.
    #[allow(clippy::too_many_arguments)]
    fn heading(&mut self, st: &State, lk: &Look, n: &Node, id: &str, s: &crate::model::Section, (first, count): (usize, usize), cx: &mut Context<Self>) -> AnyElement {
        let cell = self.cell;
        let rows = state::items(st, n);
        let item = s.item(&rows[first..first + count]);
        let was = self.row.take();
        let body = self.node(st, lk, &crate::model::row_instance(&s.row, &item, (first, rows.len())), false, &format!("{id}/section{first}"), cx);
        self.row = was;
        let (list, key) = (id.to_string(), state::heading_key(&s.of(&rows[first])));
        let tall = s.height().is_none().then(|| measure_whole(&self.laid, move |l, b| l.heights.push((list, key, b))));
        let under = Hit { part: Some(SECTION.into()), ..hit(id.to_string(), Some(first), None, false) };
        lk.boxed(div().relative().flex_none().w_full(), &style::inner_in(st, n, "section", &[]), (cell.w, cell.h), [0.; 4])
            .child(measure(&self.laid, move |l, b| l.hits.push((b, under))))
            .children(tall)
            .when_some(s.height(), |d, h| d.h(px(h * cell.h)).overflow_hidden())
            .flex()
            .flex_col()
            .justify_end()
            .child(body)
            .into_any_element()
    }

    /// A `resize` node's handle: a grip (the `resize.handle` part) on its bottom edge. A press
    /// there starts a drag that sets how tall the node is (`Msg::Resize`), from how tall it was.
    fn handle(&self, st: &State, lk: &Look, id: &str, cx: &mut Context<Self>) -> AnyElement {
        let grip = lk.boxed(div().w(px(28.)).h(px(3.)), &style::inner(st, "resize.handle", &[]), (self.cell.w, self.cell.h), [0.; 4]);
        let target = id.to_string();
        div()
            .id(SharedString::from(format!("resize-{id}")))
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(8.))
            .flex()
            .items_end()
            .justify_center()
            .pb(px(1.))
            .cursor(CursorStyle::ResizeUpDown)
            .child(grip)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    // How tall it is now: its own box as the last frame laid it out.
                    let was = this.shown.borrow().iter().find(|(_, h)| h.id == target && h.row.is_none() && !h.text && h.part.is_none()).map(|(b, _)| f32::from(b.size.height));
                    if let Some(was) = was {
                        this.sizing = Some((target.clone(), f32::from(e.position.y), was, was));
                    }
                }),
            )
            .into_any_element()
    }

    /// A list's scrollbar while it has more lines than it shows: a thumb as tall as the share
    /// shown, where the view is, from the `scrollbar` and `scrollbar.thumb` parts (`shape:
    /// "none"` draws none).
    fn scrollbar(&self, st: &State, lk: &Look, at: f32, total: f32, shown: f32) -> Option<AnyElement> {
        if total <= shown + 0.01 || total <= 0. || style::inner(st, "scrollbar", &[]).shape.as_deref() == Some("none") {
            return None;
        }
        let (size, from) = (shown / total, at / total);
        let thumb = lk.boxed(div().absolute().left_0().right_0().top(relative(from)).h(relative(size)), &style::inner(st, "scrollbar.thumb", &[]), (self.cell.w, self.cell.h), [0.; 4]);
        Some(div().absolute().top_0().bottom_0().right(px(2.)).w(px(5.)).child(thumb).into_any_element())
    }

    /// An editor or input, drawn from caretline's own frame of it: the text at the monospace
    /// font's cell size, so the caret, selections, wrapping and clicks land where caretline puts
    /// them. The frame is the size the last frame laid the editor out in (`Msg::Size`).
    fn editor(&mut self, st: &State, lk: &Look, n: &Node, id: &str, focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let marked = lk.ink(&style::inner(st, "editor.selection", &[]).ink).bg.unwrap_or(lk.ring);
        let caret = lk.ink(&style::inner(st, "editor.caret", &[]).ink).bg.unwrap_or(lk.accent);
        let placeholder = lk.ink(&style::inner(st, "input.placeholder", &[]).ink);
        let cell = self.cell;
        let field = n.field().expect("a text field");
        let single = field.line;
        let doc = state::field_view(st, id).and_then(|(key, view)| match st.store.get(&key).map(|e| &e.body) {
            Some(crate::store::Body::Text { doc }) => Some((caretline::view::render(doc, view), doc)),
            _ => None,
        });
        let mut text = div().relative().size_full().overflow_hidden();
        if single {
            text = text.h(px(cell.h)).flex_none();
        } else if n.size.as_ref().is_some_and(|s| s.measure() == Measure::Fit) {
            // As tall as its text wraps to at its width, a line at least.
            let rows = doc.as_ref().map_or(1, |(_, d)| state::field_view(st, id).map_or(1, |(_, view)| wrapped_rows(d, view)));
            text = text.h(px(rows.max(1) as f32 * cell.h)).flex_none();
        }
        let (fits, under) = (id.to_string(), id.to_string());
        text = text.child(measure_whole(&self.laid, move |l, b| l.sizes.push((fits, b))));
        text = text.child(measure(&self.laid, move |l, b| {
            l.hits.push((b, hit(under, None, None, true)));
        }));
        match doc {
            None => {
                let empty = "…".to_string();
                let ink = placeholder;
                text = text.child(inked(div().h(px(cell.h)), ink).child(empty));
                if focused {
                    text = text.child(div().absolute().left(px(0.)).top(px(3.)).w(px(2.)).h(px(cell.h - 6.)).bg(caret));
                }
            }
            Some((_, d)) if d.text.len_chars() == 0 => {
                let ink = placeholder;
                text = text.child(inked(div().h(px(cell.h)), ink).child(state::placeholder(st, n).unwrap_or_default()));
                if focused {
                    text = text.child(div().absolute().left(px(0.)).top(px(3.)).w(px(2.)).h(px(cell.h - 6.)).bg(caret));
                }
            }
            Some((frame, d)) => {
                use caretline::view::{Role, RowInfo};
                let md = field.markdown.then(|| md_inks(st, lk));
                let kinds: Vec<&str> = field.links.iter().map(|l| l.kind.as_str()).collect();
                // A document line's Markdown roles, worked out once for all the rows it wraps to.
                let mut roles: Option<(usize, usize, Vec<Md>)> = None;
                // Each row is one run of text, shaped once, so selecting never moves a glyph;
                // selections sit behind it on the cell grid that clicks are mapped on.
                for y in 0..frame.height {
                    let mut row = String::new();
                    let mut marks: Vec<(u16, u16)> = Vec::new();
                    let mut runs: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
                    if let (Some(_), Some(RowInfo::Text { line, .. })) = (&md, frame.rows.get(y as usize))
                        && roles.as_ref().is_none_or(|(l, ..)| l != line)
                    {
                        let text: String = d.text.line(*line).chars().filter(|c| !matches!(c, '\n' | '\r')).collect();
                        roles = Some((*line, d.text.line_to_char(*line), md_roles(&text, &kinds)));
                    }
                    for x in 0..frame.width {
                        let c = frame.cell(x, y);
                        let at = row.len();
                        row.push_str(&c.symbol);
                        if let (Some(inks), Some((_, start, rs)), Some(i)) = (&md, &roles, c.char_idx) {
                            let role = (i as usize).checked_sub(*start).and_then(|k| rs.get(k)).copied().unwrap_or(Md::Text);
                            if role != Md::Text {
                                let h = lk.highlight(inks(role));
                                match runs.last_mut() {
                                    Some((r, hl)) if r.end == at && *hl == h => r.end = row.len(),
                                    _ => runs.push((at..row.len(), h)),
                                }
                            }
                        }
                        if matches!(c.role, Role::Selection | Role::Caret) {
                            match marks.last_mut() {
                                Some((_, end)) if *end == x => *end = x + 1,
                                _ => marks.push((x, x + 1)),
                            }
                        }
                    }
                    let top = y as f32 * cell.h;
                    for (a, b) in marks {
                        text = text.child(div().absolute().left(px(a as f32 * cell.w)).top(px(top)).w(px((b - a) as f32 * cell.w)).h(px(cell.h)).bg(marked));
                    }
                    let shown = row.trim_end().len();
                    runs.retain_mut(|(r, _)| {
                        r.end = r.end.min(shown);
                        r.start < r.end
                    });
                    row.truncate(shown);
                    text = text.child(div().absolute().left(px(0.)).top(px(top)).h(px(cell.h)).whitespace_nowrap().child(StyledText::new(row).with_highlights(runs)));
                }
                if let Some((x, y)) = frame.cursor.filter(|_| focused) {
                    let (left, top) = (x as f32 * cell.w, y as f32 * cell.h);
                    text = text.child(div().absolute().left(px(left)).top(px(top + 3.)).w(px(2.)).h(px(cell.h - 6.)).bg(caret));
                }
            }
        }
        // More text than it shows: the scrollbar lists have, where the view is in it.
        if !single {
            text = text.children(state::editor_rows(st, id).and_then(|(at, total, shown)| self.scrollbar(st, lk, at as f32, total as f32, shown as f32)));
        }
        let (press, wheel) = (id.to_string(), id.to_string());
        div()
            .id(SharedString::from(format!("ed-{id}")))
            .flex()
            .flex_col()
            .size_full()
            // A column in, so the text sits on the cell grid wherever the editor does.
            .px(px(cell.w))
            // The grid's own face, size, line and weight, whatever type role the screen or a
            // part around it sets: caretline places the caret, selections and clicks on that
            // grid, so text drawn any other size would part from them. (Proportional editors
            // are phase 4 in native.md.)
            .font_family(MONO)
            .text_size(px(TYPE))
            .line_height(px(ROW))
            .font_weight(gpui::FontWeight::NORMAL)
            .when(single, |d| d.justify_center())
            // A press in the text puts the caret there (twice a word, three times the line) and
            // starts a drag; the wheel scrolls it. The same messages a terminal's mouse makes.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                    let Some((col, row)) = this.in_text(&press, e.position) else { return };
                    cx.stop_propagation();
                    this.drag = Some(press.clone());
                    this.send(Msg::Point { id: press.clone(), col, row, extend: e.modifiers.shift }, cx);
                }),
            )
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.wheel(&wheel, e, cx)))
            .child(text)
            .into_any_element()
    }
}

/// The rows a document wraps to in its view, as far as a thousand: a `fit` editor's height.
fn wrapped_rows(doc: &caretline::Document, view: &caretline::View) -> usize {
    let l = caretline::layout::Layout::of(doc, view);
    let mut rows = 0;
    for line in 0..=l.last_line() {
        rows += l.line_rows(line);
        if rows >= 1000 {
            break;
        }
    }
    rows
}

impl Render for Portal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Instant::now();
        // The window's size in cells: `$screen`, as a terminal's resize sends it.
        let vp = window.viewport_size();
        let screen = ((f32::from(vp.width) / self.cell.w) as u16, (f32::from(vp.height) / self.cell.h) as u16);
        if screen != self.screen {
            self.screen = screen;
            self.engine.send(Msg::Screen { width: screen.0, height: screen.1 });
        }
        *self.laid.borrow_mut() = Laid::default();
        self.wake = None;
        match self.engine.clone() {
            Engine::Live(live) => {
                let mut live = live.lock().expect("the engine");
                // What the screen copied goes on the system clipboard.
                if let Some(text) = live.take_copied().pop() {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
                // Agents' `screen` requests get a readback of the last frame.
                let asked = live.take_screens();
                if !asked.is_empty() {
                    let text = crate::readback::outline(&live.readback());
                    for r in asked {
                        let _ = r.send(json!({"ok": true, "screen": text}));
                    }
                }
                self.titled(live.state(), window);
                self.frame(live.state(), t, cx)
            }
            #[cfg(feature = "window-test")]
            Engine::Test(side) => {
                self.titled(&side.borrow().st, window);
                self.frame(&side.borrow().st, t, cx)
            }
        }
    }
}

impl Portal {
    /// The window's title, when the UI's has changed.
    fn titled(&mut self, st: &State, window: &mut Window) {
        if st.title != self.title {
            self.title.clone_from(&st.title);
            window.set_window_title(st.title.as_deref().unwrap_or("fictty"));
        }
    }

    /// The whole window for `st`, started at `t`.
    fn frame(&mut self, st: &State, t: Instant, cx: &mut Context<Self>) -> gpui::Stateful<gpui::Div> {
        self.load_fonts(st, cx);
        let faces = std::mem::take(&mut self.faces);
        let lk = Look::new(st, &faces);
        let body = self.node(st, &lk, st.tree(), false, "root", cx);
        // A lasting write going, or a toast, changes the frame too.
        for at in st.expiring.iter().map(|e| e.at).chain(st.toasts.iter().map(|t| t.until)) {
            self.soon(at);
        }
        // The status line: the status, else the first problem, else what keys do; the speed
        // meter when the UI asks for it.
        let c = (self.cell.w, self.cell.h);
        let bar = st.ui.status_bar.is_none_or(|b| b.shown()).then(|| {
            let said = st.status.as_ref().or(st.problems.first());
            let text = match said {
                Some(s) => lk.boxed(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap(), &style::inner(st, "status.error", &[]), c, [0.; 4]).child(s.clone()),
                None => div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().child("tab moves the focus · ↑ ↓ move · ← → tabs · q quits"),
            };
            let speed = (st.ui.status_bar == Some(crate::model::StatusBar::Speed)).then(|| {
                let s = &st.stats;
                div().flex_none().child(format!("{} fps · build {:.2} ms (max {:.1}) · {} msg/s", s.fps as u64, s.draw_ms, s.max_ms, s.msgs as u64))
            });
            lk.boxed(div().flex().flex_row().gap_4().items_center().h(px(ROW)).text_xs(), &style::inner(st, "status", &[]), c, [0.; 4]).child(text).children(speed)
        });
        let toasts: Vec<AnyElement> = st.toasts.iter().rev().take(3).map(|t| toast(st, &lk, t, (self.cell.w, self.cell.h))).collect();
        // Once the frame has painted, what it laid out goes to the engine; if that changed the
        // state (an editor learned its size), another frame follows.
        // The mouse's moves and releases are the window's, wherever they land: over a dialog or a
        // float too, which keep presses from what's under them.
        let handover = {
            let (laid, shown, engine, cell, wake) = (self.laid.clone(), self.shown.clone(), self.engine.clone(), self.cell, self.wake);
            let took = t.elapsed();
            let (moves, ups) = (cx.entity().downgrade(), cx.entity().downgrade());
            canvas(|_, _, _| {}, move |_, _, window, _| {
                handover(&laid, &shown, &engine, cell, wake, took, window);
                window.on_mouse_event(move |e: &MouseMoveEvent, phase, _, cx| {
                    if phase == DispatchPhase::Bubble {
                        let _ = moves.update(cx, |this, cx| this.moved(e, cx));
                    }
                });
                window.on_mouse_event(move |e: &MouseUpEvent, phase, _, cx| {
                    if phase == DispatchPhase::Bubble && e.button == MouseButton::Left {
                        let _ = ups.update(cx, |this: &mut Portal, cx| this.up(cx));
                    }
                });
            })
            .absolute()
            .size_0()
        };
        // The `screen` part is the window's own: a theme's body type (its font for everything),
        // over the theme's colours.
        let screen = style::inner(st, "screen", &[]);
        let frame = div()
            .id("portal")
            .track_focus(&self.focus)
            // Files dragged in from Finder: GPUI makes them a drag of their paths.
            .on_drag_move::<ExternalPaths>(cx.listener(|this, e: &gpui::DragMoveEvent<ExternalPaths>, _, cx| {
                let files = e.drag(cx).paths().iter().map(|p| p.to_string_lossy().into_owned()).collect();
                this.bring(files, e.event.position, cx);
            }))
            .on_drop::<ExternalPaths>(cx.listener(|this, _: &ExternalPaths, _, cx| this.let_go(true, cx)))
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                let Some(key) = key_name(e) else { return };
                // ⌘V pastes what's on the system clipboard; caretline knows its own copies when
                // they come back. With nothing there, the key goes in as itself.
                if key == "d-v"
                    && let Some(text) = cx.read_from_clipboard().and_then(|c| c.text())
                {
                    return this.send(Msg::Paste { text }, cx);
                }
                this.send(Msg::Key { key }, cx);
            }))
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(lk.bg)
            .text_color(lk.fg)
            .text_size(px(TYPE))
            .line_height(px(ROW))
            .map(|d| text::typed(lk.boxed(d, &screen, (self.cell.w, self.cell.h), [0.; 4]), screen.role.as_ref(), &lk.fonts()))
            // A cell's margin, so the body starts on the cell grid: a cell's middle never falls on
            // the edge between two rows.
            .child(div().flex().flex_1().min_h(px(0.)).px(px(self.cell.w)).py(px(self.cell.h)).child(body))
            .child(div().absolute().bottom(px(32.)).right(px(16.)).flex().flex_col().gap_1().children(toasts))
            .children(bar)
            .child(handover);
        self.faces = faces;
        frame
    }

    /// Load the font files the themes name that aren't loaded yet, noting the family each
    /// turned out to be (what the text system knows after that it didn't before).
    fn load_fonts(&mut self, st: &State, cx: &mut Context<Self>) {
        let files: Vec<String> = st.themes.values().flat_map(|t| t.fonts.values()).filter(|f| style::is_font_file(f) && !self.faces.contains_key(*f)).cloned().collect();
        for f in files {
            let ts = cx.text_system().clone();
            let before = ts.all_font_names();
            let loaded = std::fs::read(&f).map_err(|e| e.to_string()).and_then(|bytes| ts.add_fonts(vec![std::borrow::Cow::Owned(bytes)]).map_err(|e| e.to_string()));
            let family = loaded.as_ref().ok().and_then(|_| ts.all_font_names().into_iter().find(|n| !before.contains(n)));
            // What failed goes back with the frame, to show as a problem.
            if let Err(why) = loaded {
                self.laid.borrow_mut().fonts.push((f.clone(), why));
            }
            self.faces.insert(f, family.unwrap_or_default());
        }
    }
}

/// The inks of Markdown's roles in an editor, from the theme's `md.*` parts.
fn md_inks<'a>(st: &'a State, lk: &'a Look<'a>) -> impl Fn(Md) -> Ink + 'a {
    move |role| {
        let part = match role {
            Md::Text => return Ink::default(),
            Md::Heading => "md.heading",
            Md::Bullet => "md.bullet",
            Md::Quote => "md.quote",
            Md::Code => "md.code",
            Md::Task => "md.task",
            Md::Done => "md.done",
            Md::Link => "md.link",
        };
        lk.ink(&style::inner(st, part, &[]).ink)
    }
}

/// The rows of a list or table to build: the lines its view reaches from where it starts
/// (`state::Lines`), a part-shown one at either end included. Before any frame has said how tall
/// the view is, a screenful (clipped to what shows).
fn shown_rows(st: &State, n: &Node) -> std::ops::Range<usize> {
    let lines = state::Lines::of(st, n);
    let shown = lines.shown(lines.offset(n.id.as_deref().and_then(|id| st.local.get(id))));
    if shown.is_empty() {
        return 0..0;
    }
    lines.rows_in(shown.start).start..lines.rows_in(shown.end - 1).end
}

/// A selected row's ink, focused or not.
/// A toast: the `toast` part, its border in its tone's colour (or the `toast.<tone>` part).
fn toast(st: &State, lk: &Look, t: &state::Toast, cell: (f32, f32)) -> AnyElement {
    let mut r = match t.tone.as_deref() {
        Some(tone) if style::Theme::of(st).part(&format!("toast.{tone}")).is_some() => style::inner(st, &format!("toast.{tone}"), &[]),
        _ => style::inner(st, "toast", &[]),
    };
    if let (Some(b), Some(c)) = (r.border.as_mut(), t.tone.as_deref().and_then(|c| style::Theme::of(st).colour(c))) {
        b.color = c;
    }
    lk.boxed(div(), &r, cell, [0.; 4]).child(t.text.clone()).into_any_element()
}

/// What a painted frame laid out, to the engine, in cells.
fn handover(laid: &Shared, shown: &RefCell<DrawnHits>, engine: &Engine, cell: Cell, wake: Option<u64>, took: Duration, window: &mut Window) {
    let l = std::mem::take(&mut *laid.borrow_mut());
    let drawn = Drawn {
        // Nothing shows of a node clipped away (scrolled out of its view): it has no rectangle
        // and nothing lands on it.
        rects: l.rects.into_iter().filter(|(_, b)| !b.is_empty()).map(|(id, b)| (id, cell.rect(b))).collect(),
        hits: l.hits.iter().filter(|(b, _)| !b.is_empty()).map(|(b, h)| Hit { rect: cell.rect(*b), ..h.clone() }).collect(),
        sizes: l.sizes.into_iter().map(|(id, b)| { let (w, h) = cell.fit(b); (id, w, h) }).collect(),
        heights: l.heights.into_iter().map(|(id, key, b)| (id, key, f32::from(b.size.height) / cell.h)).collect(),
        areas: l.areas.into_iter().map(|(id, b)| { let (w, h) = cell.fit(b); (id, w, h) }).collect(),
        wake,
        cut: l.cut,
        fonts: l.fonts,
        scrolls: l
            .views
            .iter()
            .filter_map(|(id, down, v)| {
                let c = l.contents.iter().find(|(c, _)| c == id)?.1;
                let along = |b: Bounds<Pixels>| if *down { (f32::from(b.origin.y) / cell.h, f32::from(b.size.height) / cell.h) } else { (f32::from(b.origin.x) / cell.w, f32::from(b.size.width) / cell.w) };
                let ((start, view), (_, content)) = (along(*v), along(c));
                Some(crate::engine::Scrolled { id: id.clone(), start, view, content })
            })
            .collect(),
        focused: l.focused.map(|b| [f32::from(b.origin.x) / cell.w, f32::from(b.origin.y) / cell.h, f32::from(b.size.width) / cell.w, f32::from(b.size.height) / cell.h]),
    };
    *shown.borrow_mut() = l.hits;
    if engine.drew(took, drawn) {
        window.refresh();
    }
}

/// The window's size to start with, in pixels.
const START: (f32, f32) = (1200.0, 800.0);

/// Run `app` in a native window until it quits or the window closes.
pub fn run(app: crate::App) -> Result<()> {
    // A first guess at the size in cells; the first frame corrects it with the measured font.
    let cells = ((START.0 / 8.4) as u16, (START.1 / ROW) as u16);
    let live = app.into_core().live(cells.0, cells.1)?;
    // The engine's thread rings the window when the screen may have changed; GPUI draws at most
    // once a display frame, however often it's told.
    let (ring, rung) = futures::channel::mpsc::unbounded::<()>();
    let live = live.spawn(move || ring.unbounded_send(()).is_ok());
    let mut start = Some((live, rung));
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(START.0), px(START.1)), cx);
        let (live, mut rung) = start.take().expect("one window");
        let opened = cx.open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, |window, cx| {
            cx.new(|cx| {
                let ringing = live.clone();
                cx.spawn(async move |this, cx| {
                    use futures::StreamExt;
                    while rung.next().await.is_some() {
                        let go_on = this.update(cx, |_: &mut Portal, cx| {
                            let mut l = ringing.lock().expect("the engine");
                            if l.quit() {
                                cx.quit();
                                return false;
                            }
                            if let Some(bin) = l.take_upgrade() {
                                // Only returns if the exec failed; the screen carries on.
                                let _ = l.upgrade(&bin);
                            }
                            drop(l);
                            cx.notify();
                            true
                        });
                        if !go_on.unwrap_or(false) {
                            break;
                        }
                    }
                })
                .detach();
                Portal::new(Engine::Live(live), window, cx)
            })
        });
        if let Err(e) = opened {
            eprintln!("fictty: no window: {e:#}");
            cx.quit();
        }
        cx.activate(true);
    });
    Ok(())
}
