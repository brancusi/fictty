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
use look::{Ink, Look};
use crate::model::{Float, Kind, Md, Measure, Node, Size, md_roles};
use crate::state::{self, Msg, State};
use anyhow::Result;
use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, DefiniteLength, DispatchPhase, FocusHandle, FontWeight, HighlightStyle, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    MouseUpEvent, Point, ScrollWheelEvent, SharedString, StyledText, Window, WindowBounds, WindowOptions, canvas, div, font, prelude::*, px, relative, size,
};
use serde_json::{Value, json};
use std::cell::RefCell;
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
    /// Each editor's text area and each list's rows, as bounds to fit cells into, with the
    /// rows a list's item takes (1 for an editor).
    sizes: Vec<(String, Bounds<Pixels>, u16)>,
    /// The area each node whose `when` reads `$width` or `$height` was laid out in.
    areas: Vec<(String, Bounds<Pixels>)>,
}

type Shared = Rc<RefCell<Laid>>;

/// A painted frame's hit regions, in pixels, innermost last.
type DrawnHits = Vec<(Bounds<Pixels>, Hit)>;

/// What's under the mouse in `id`: the whole node, or one row or tab of it, or an editor's text.
fn hit(id: String, row: Option<usize>, tab: Option<usize>, text: bool) -> Hit {
    Hit { rect: Rect::default(), id, row, tab, text }
}

/// A canvas that notes, as it's laid out, the bounds it filled: absolute, so it takes no room.
/// What's noted is what shows: the bounds clipped to what holds it (a node squeezed out of its
/// parent's room is clipped away, and so is its hit).
fn measure(laid: &Shared, note: impl FnOnce(&mut Laid, Bounds<Pixels>) + 'static) -> AnyElement {
    let laid = laid.clone();
    canvas(move |b, window, _| note(&mut laid.borrow_mut(), b.intersect(&window.content_mask().bounds)), |_, _, _, _| {}).absolute().inset_0().into_any_element()
}

/// Ink on an element: its colours, weight and slant.
fn inked<E: Styled>(mut el: E, ink: Ink) -> E {
    let (fg, bg) = if ink.reverse { (ink.bg, ink.fg) } else { (ink.fg, ink.bg) };
    if let Some(c) = fg {
        el = el.text_color(c);
    }
    if let Some(c) = bg {
        el = el.bg(c);
    }
    if ink.bold {
        el = el.font_weight(FontWeight::BOLD);
    }
    if ink.italic {
        el = el.italic();
    }
    if ink.dim {
        el = el.opacity(0.6);
    }
    el
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
    /// The wheel's travel over a node, not yet a whole scroll step.
    wheel: (String, f32),
    /// When (on the logical clock) the frame being built changes by itself next.
    wake: Option<u64>,
}

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
        Portal { engine, focus, cell, screen: (0, 0), laid: Shared::default(), shown: Rc::default(), drag: None, wheel: (String::new(), 0.0), wake: None }
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
    fn wheel(&mut self, id: &str, e: &ScrollWheelEvent, cx: &mut Context<Self>) {
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
    /// window; otherwise the row under it is hovered, as the terminal's hover, told only when it
    /// changes. Under it means the innermost thing there: not a row a dialog covers.
    fn moved(&mut self, e: &MouseMoveEvent, cx: &mut Context<Self>) {
        if let Some(id) = self.drag.clone().filter(|_| e.dragging()) {
            if let Some((col, row)) = self.in_text(&id, e.position) {
                self.send(Msg::Drag { id, col, row }, cx);
            }
            return;
        }
        self.drag = None;
        let hover = self.shown.borrow().iter().rev().find(|(b, _)| b.contains(&e.position)).filter(|(_, h)| h.row.is_some()).map(|(_, h)| state::Hover { id: h.id.clone(), row: h.row });
        if hover != self.engine.hover() {
            self.send(Msg::Hover { hover }, cx);
        }
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
/// and `2*` a grow factor; `min` and `max` bound it, in cells.
fn sized(el: gpui::Stateful<gpui::Div>, n: &Node, along_row: bool, cell: Cell) -> gpui::Stateful<gpui::Div> {
    let per = if along_row { cell.w } else { cell.h };
    let fixed = |mut el: gpui::Stateful<gpui::Div>, l: DefiniteLength| {
        el.style().flex_grow = Some(0.);
        el.flex_shrink().flex_basis(l).min_w(px(0.)).min_h(px(0.))
    };
    let mut el = match n.size.as_ref().map(Size::measure) {
        Some(Measure::Cells(c)) => fixed(el, px(c as f32 * per).into()),
        Some(Measure::Px(p)) => fixed(el, px(p).into()),
        Some(Measure::Percent(p)) => fixed(el, relative(p / 100.0)),
        Some(Measure::Share(w)) => {
            let mut el = el.flex_basis(px(0.)).min_w(px(0.)).min_h(px(0.)).flex_shrink();
            el.style().flex_grow = Some(w as f32);
            el
        }
        None => el.flex_1().min_w(px(0.)).min_h(px(0.)),
    };
    if let Some(m) = n.min {
        el = if along_row { el.min_w(px(m as f32 * per)) } else { el.min_h(px(m as f32 * per)) };
    }
    if let Some(m) = n.max {
        el = if along_row { el.max_w(px(m as f32 * per)) } else { el.max_h(px(m as f32 * per)) };
    }
    el
}

/// One side of a float, as a length: cells, pixels or a fraction of its stack; the whole stack
/// by default.
fn float_side(s: Option<&Size>, per: f32) -> DefiniteLength {
    match s.map(Size::measure) {
        Some(Measure::Cells(c)) => px(c as f32 * per).into(),
        Some(Measure::Px(p)) => px(p).into(),
        Some(Measure::Percent(p)) => relative(p / 100.0),
        None | Some(Measure::Share(_)) => relative(1.0),
    }
}

impl Portal {
    fn node(&mut self, st: &State, lk: &Look, n: &Node, along_row: bool, path: &str, cx: &mut Context<Self>) -> AnyElement {
        if !state::shown(st, n) {
            return div().into_any_element();
        }
        let id = n.id.clone().unwrap_or_default();
        let key = if id.is_empty() { path.to_string() } else { id.clone() };
        let focused = !id.is_empty() && st.focus.as_deref() == Some(id.as_str());
        let cell = self.cell;
        let mut el = sized(div().id(SharedString::from(format!("n-{key}"))).relative().flex().flex_col().overflow_hidden(), n, along_row, cell);
        // Where it was laid out, for the readback and the mouse: noted before its children, so
        // the innermost hit comes last.
        if !id.is_empty() {
            let id = id.clone();
            el = el.child(measure(&self.laid, move |l, b| {
                l.rects.push((id.clone(), b));
                l.hits.push((b, hit(id, None, None, false)));
            }));
        }
        let own = lk.style(n.style.as_ref());
        el = inked(el, own);
        let v = state::bound(st, n).unwrap_or(&Value::Null);
        let panel = n.title.is_some() || n.border;
        // `pad` cells and rows inside the border (a panel's own inset besides).
        let (x, y) = n.pad.map_or((0, 0), |p| p.xy());
        let inset = if panel { 8.0 } else { 0.0 };
        if panel || (x, y) != (0, 0) {
            el = el.px(px(inset + x as f32 * cell.w)).py(px(inset + y as f32 * cell.h));
        }
        el = match n.align.as_deref() {
            Some("center") => el.text_center(),
            Some("right") => el.text_right(),
            _ => el,
        };
        // A title or a border: a panel with rounded corners, the title on top.
        if panel {
            let edge = if focused { lk.part("border.focused", Ink { fg: Some(lk.accent), ..Ink::default() }) } else { lk.part("border", Ink { fg: Some(lk.border), ..Ink::default() }) };
            el = el.bg(lk.panel).border_color(edge.fg.unwrap_or(lk.border)).gap_1();
            // Its edge, or the theme's: rounded (the default), or square, `thick` and `double`
            // heavier.
            el = match n.edge.as_deref().or(lk.edge) {
                None | Some("rounded") => el.rounded_lg().border_1(),
                Some("thick" | "double") => el.border_2(),
                Some(_) => el.border_1(),
            };
            if let Some(t) = &n.title {
                let ink = if focused { lk.part("title.focused", Ink { fg: Some(lk.accent), bold: true, ..Ink::default() }) } else { lk.part("title", Ink { fg: Some(lk.dim), bold: true, ..Ink::default() }) };
                el = el.child(div().flex_none().w_full().text_sm().child(text::line(lk, t, v, ink, false)));
            }
        }
        let body: AnyElement = match &n.kind {
            Kind::Col { children, gap } | Kind::Row { children, gap } => {
                let row = matches!(n.kind, Kind::Row { .. });
                let kids: Vec<AnyElement> = children.iter().enumerate().map(|(k, c)| self.node(st, lk, c, row, &format!("{key}.{k}"), cx)).collect();
                let d = div().relative().flex().size_full().min_h(px(0.)).min_w(px(0.)).gap(px(gap.unwrap_or(0) as f32 * if row { cell.w } else { cell.h }));
                let d = if row { d.flex_row() } else { d.flex_col() };
                // `$width` and `$height` in a child's `when` are this node's inner size.
                let sized: Vec<String> = children.iter().filter(|c| c.when.as_deref().is_some_and(state::sized)).filter_map(|c| c.id.clone()).collect();
                let d = if sized.is_empty() { d } else { d.child(measure(&self.laid, move |l, b| l.areas.extend(sized.into_iter().map(|id| (id, b))))) };
                d.children(kids).into_any_element()
            }
            Kind::Stack { children } => {
                let shown: Vec<(usize, Node)> = children.iter().enumerate().filter(|(_, c)| state::shown(st, c)).map(|(k, c)| (k, c.clone())).collect();
                let mut kids: Vec<AnyElement> = Vec::new();
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
                        None => {
                            let covers = c.modal;
                            let shown = self.node(st, lk, &c, false, &path, cx);
                            div().absolute().inset_0().flex().when(covers, |d| d.bg(lk.bg).occlude()).child(shown).into_any_element()
                        }
                    });
                }
                div().relative().size_full().children(kids).into_any_element()
            }
            Kind::Text { text } => text::lines(lk, text, v, Ink::default()),
            Kind::Spinner { text } => {
                let (glyph, next) = state::spinner(st.now_ms);
                self.soon(next);
                let ink = lk.part("spinner", Ink { fg: Some(lk.accent), ..Ink::default() });
                div().flex().flex_row().gap_2().child(inked(div(), ink).child(glyph)).child(text::line(lk, text, v, Ink::default(), false)).into_any_element()
            }
            Kind::Button { label } => {
                let label = text::plain(label, v);
                let ink = if focused { lk.part("button.focused", Ink { fg: Some(lk.bg), bg: Some(lk.accent), bold: true, ..Ink::default() }) } else { lk.part("button", Ink { fg: Some(lk.fg), bg: Some(lk.ring), ..Ink::default() }) };
                let target = id.clone();
                let hover = lk.hover;
                // The face, as wide as its label, where `align` puts it, down the middle.
                el = match n.align.as_deref() {
                    Some("center") => el.items_center(),
                    Some("right") => el.items_end(),
                    _ => el.items_start(),
                }
                .justify_center();
                let face = (!id.is_empty()).then(|| {
                    let under = id.clone();
                    measure(&self.laid, move |l, b| l.hits.push((b, hit(under, None, None, false))))
                });
                inked(lk.round(div().id(SharedString::from(format!("btn-{key}"))).relative().flex_none().px_3().cursor_pointer(), 6.), ink)
                    .children(face)
                    .when(!focused, |d| d.hover(move |s| s.bg(hover)))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: None, tab: None }, cx)))
                    .child(label)
                    .into_any_element()
            }
            Kind::Rule { .. } => div().h(px(1.)).w_full().bg(own.fg.unwrap_or(lk.border)).into_any_element(),
            Kind::List { .. } | Kind::Tree { .. } => self.list(st, lk, n, &id, cx),
            Kind::Table { columns, .. } => {
                let rows = state::items(st, n).to_vec();
                let sel = state::selected(st, &id);
                let columns = columns.clone();
                let head_ink = lk.part("table.header", Ink { fg: Some(lk.accent), bold: true, ..Ink::default() });
                let head = inked(div().flex().flex_row().h(px(cell.h)), head_ink).children(columns.iter().map(|c| div().flex_1().child(c.title.clone())));
                let chosen = selected_ink(lk, focused);
                let body: Vec<AnyElement> = shown_rows(st, &id, rows.len(), 1)
                    .map(|k| (k, &rows[k]))
                    .map(|(k, r)| {
                        let target = id.clone();
                        let under = id.clone();
                        div()
                            .id(k)
                            .relative()
                            .flex()
                            .flex_row()
                            .h(px(cell.h))
                            .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, Some(k), None, false)))))
                            .when(k == sel, |d| inked(d, chosen))
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: Some(k), tab: None }, cx)))
                            .children(columns.iter().map(|c| div().flex_1().min_w_0().child(text::line(lk, &c.value, r, Ink::default(), false))))
                            .into_any_element()
                    })
                    .collect();
                // The rows it has room for, below the header.
                let fits = id.clone();
                let wheel = id.clone();
                let rows_area = div()
                    .id(SharedString::from(format!("rows-{key}")))
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(measure(&self.laid, move |l, b| l.sizes.push((fits, b, 1))))
                    .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.wheel(&wheel, e, cx)))
                    .children(body);
                div().flex().flex_col().size_full().child(head).child(rows_area).into_any_element()
            }
            Kind::Editor { .. } | Kind::Input { .. } => self.editor(st, lk, n, &id, focused, cx),
            Kind::Tabs { tabs, children, .. } => {
                let at = state::tab_of(st, n).min(children.len().saturating_sub(1));
                let tabs = tabs.clone();
                let active = lk.part("tabs.active", Ink { fg: Some(lk.bg), bg: Some(lk.accent), bold: true, ..Ink::default() });
                let idle = lk.part("tabs.idle", Ink { fg: Some(lk.dim), ..Ink::default() });
                let child = children.get(at).cloned();
                let bar = div().flex().flex_row().gap_2().children(tabs.iter().enumerate().map(|(k, t)| {
                    let target = id.clone();
                    let under = id.clone();
                    div()
                        .id(SharedString::from(format!("tab-{key}-{k}")))
                        .relative()
                        .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, None, Some(k), false)))))
                        .px_2()
                        .map(|d| lk.round(d, 6.))
                        .cursor_pointer()
                        .map(|d| if k == at { inked(d, active) } else { inked(d, idle) })
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: None, tab: Some(k) }, cx)))
                        .child(text::plain(t, v))
                }));
                let shown = child.map(|c| self.node(st, lk, &c, false, &format!("{key}.{at}"), cx)).unwrap_or_else(|| div().into_any_element());
                div().flex().flex_col().size_full().gap_1().child(bar).child(div().flex().flex_1().min_h(px(0.)).child(shown)).into_any_element()
            }
            Kind::Toasts {} => {
                if let Some(at) = st.toasts.iter().map(|t| t.until).min() {
                    self.soon(at);
                }
                let ts: Vec<AnyElement> = st.toasts.iter().rev().map(|t| toast(lk, t)).collect();
                div().flex().flex_col().gap_1().children(ts).into_any_element()
            }
            _ => div().text_color(lk.dim).child(format!("[{}]", n.kind_name())).into_any_element(),
        };
        el = el.child(body);
        el.into_any_element()
    }

    /// A node floating in its stack where its `float` says: its own size (the stack's by default),
    /// at a side, a corner or the middle, `margin` cells in from the edge. It covers what's under
    /// it, the mouse too.
    fn float(&mut self, st: &State, lk: &Look, n: Node, fl: &Float, path: &str, cx: &mut Context<Self>) -> AnyElement {
        let cell = self.cell;
        let m = fl.margin.unwrap_or(0) as f32;
        let at = fl.at.as_deref().unwrap_or("center");
        let wrap = div().absolute().inset_0().flex().px(px(m * cell.w)).py(px(m * cell.h));
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
        let (w, h) = (float_side(fl.width.as_ref(), cell.w), float_side(fl.height.as_ref(), cell.h));
        // Its float sizes it, not `size`: it fills the box.
        let n = Node { size: None, min: None, max: None, ..n };
        let shown = self.node(st, lk, &n, false, path, cx);
        let face = div().id(SharedString::from(format!("float-{path}"))).flex().flex_col().flex_none().w(w).h(h).max_w_full().max_h_full().bg(lk.bg).occlude().child(shown);
        wrap.child(face).into_any_element()
    }

    /// A list's rows, from where the state says it's scrolled: as many as fit, each the rows its
    /// item template takes. The wheel scrolls the state, as the terminal's does.
    fn list(&mut self, st: &State, lk: &Look, n: &Node, id: &str, cx: &mut Context<Self>) -> AnyElement {
        let (Kind::List { rows: look, .. } | Kind::Tree { rows: look, .. }) = &n.kind else { unreachable!("a list or tree") };
        let focused = st.focus.as_deref() == Some(id);
        let cell = self.cell;
        let rows = state::items(st, n);
        let sel = state::selected(st, id);
        let per = look.lines.unwrap_or(1).max(1);
        if rows.is_empty() {
            let ink = lk.part("empty", Ink { fg: Some(lk.dim), italic: true, ..Ink::default() });
            return inked(div().px_2(), ink).child(look.empty.clone().unwrap_or_else(|| "(nothing)".into())).into_any_element();
        }
        let chosen = selected_ink(lk, focused);
        let hover = lk.hover;
        let shown: Vec<AnyElement> = shown_rows(st, id, rows.len(), per)
            .map(|k| {
                let r = &rows[k];
                let text = match &look.item {
                    Some(t) => text::lines(lk, t, r, Ink::default()),
                    None => div().child(crate::model::scalar(Some(r))).into_any_element(),
                };
                let (target, under) = (id.to_string(), id.to_string());
                div()
                    .id(k)
                    .relative()
                    .flex_none()
                    .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, Some(k), None, false)))))
                    .h(px(cell.h * per as f32))
                    .px_2()
                    .map(|d| lk.round(d, 2.))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .overflow_hidden()
                    .map(|d| if k == sel { inked(d, chosen) } else { d.hover(move |s| s.bg(hover)) })
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: Some(k), tab: None }, cx)))
                    .child(text)
                    .into_any_element()
            })
            .collect();
        let (fits, wheel) = (id.to_string(), id.to_string());
        div()
            .id(SharedString::from(format!("rows-{id}")))
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.))
            .overflow_hidden()
            // The items it has room for: the wheel and paging move by them.
            .child(measure(&self.laid, move |l, b| l.sizes.push((fits, b, per))))
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.wheel(&wheel, e, cx)))
            .children(shown)
            .into_any_element()
    }

    /// An editor or input, drawn from caretline's own frame of it: the text at the monospace
    /// font's cell size, so the caret, selections, wrapping and clicks land where caretline puts
    /// them. The frame is the size the last frame laid the editor out in (`Msg::Size`).
    fn editor(&mut self, st: &State, lk: &Look, n: &Node, id: &str, focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let marked = lk.part("editor.selection", Ink { bg: Some(lk.ring), ..Ink::default() }).bg.unwrap_or(lk.ring);
        let caret = lk.part("editor.caret", Ink { bg: Some(lk.accent), ..Ink::default() }).bg.unwrap_or(lk.accent);
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
        }
        let (fits, under) = (id.to_string(), id.to_string());
        text = text.child(measure(&self.laid, move |l, b| {
            l.sizes.push((fits, b, 1));
            l.hits.push((b, hit(under, None, None, true)));
        }));
        match doc {
            None => {
                let empty = "…".to_string();
                let ink = lk.part("input.placeholder", Ink { fg: Some(lk.dim), italic: true, ..Ink::default() });
                text = text.child(inked(div().h(px(cell.h)), ink).child(empty));
                if focused {
                    text = text.child(div().absolute().left(px(0.)).top(px(3.)).w(px(2.)).h(px(cell.h - 6.)).bg(caret));
                }
            }
            Some((_, d)) if d.text.len_chars() == 0 => {
                let ink = lk.part("input.placeholder", Ink { fg: Some(lk.dim), italic: true, ..Ink::default() });
                text = text.child(inked(div().h(px(cell.h)), ink).child(field.placeholder.unwrap_or_default().to_string()));
                if focused {
                    text = text.child(div().absolute().left(px(0.)).top(px(3.)).w(px(2.)).h(px(cell.h - 6.)).bg(caret));
                }
            }
            Some((frame, d)) => {
                use caretline::view::{Role, RowInfo};
                let md = field.markdown.then(|| md_inks(lk));
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
        let (press, wheel) = (id.to_string(), id.to_string());
        div()
            .id(SharedString::from(format!("ed-{id}")))
            .flex()
            .flex_col()
            .size_full()
            // A column in, so the text sits on the cell grid wherever the editor does.
            .px(px(cell.w))
            .font_family(MONO)
            .when(single, |d| lk.round(d, 6.).justify_center().bg(lk.bg).border_1().border_color(if focused { lk.accent } else { lk.border }))
            // A press in the text puts the caret there (twice a word, three times the line) and
            // starts a drag; the wheel scrolls it. The same messages a terminal's mouse makes.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                    let Some((col, row)) = this.in_text(&press, e.position) else { return };
                    this.drag = Some(press.clone());
                    this.send(Msg::Point { id: press.clone(), col, row, extend: e.modifiers.shift }, cx);
                }),
            )
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| this.wheel(&wheel, e, cx)))
            .child(text)
            .into_any_element()
    }
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
                self.frame(live.state(), t, cx)
            }
            #[cfg(feature = "window-test")]
            Engine::Test(side) => self.frame(&side.borrow().st, t, cx),
        }
    }
}

impl Portal {
    /// The whole window for `st`, started at `t`.
    fn frame(&mut self, st: &State, t: Instant, cx: &mut Context<Self>) -> gpui::Stateful<gpui::Div> {
        let lk = Look::new(st);
        let body = self.node(st, &lk, st.tree(), false, "root", cx);
        // A lasting write going, or a toast, changes the frame too.
        for at in st.expiring.iter().map(|e| e.at).chain(st.toasts.iter().map(|t| t.until)) {
            self.soon(at);
        }
        let status = format!(
            "{} fps · build {:.2} ms (max {:.1}) · {} msg/s{}",
            st.stats.fps as u64,
            st.stats.draw_ms,
            st.stats.max_ms,
            st.stats.msgs as u64,
            st.status.as_ref().map(|s| format!(" · {s}")).unwrap_or_default()
        );
        let toasts: Vec<AnyElement> = st.toasts.iter().rev().take(3).map(|t| toast(&lk, t)).collect();
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
                        let _ = ups.update(cx, |this: &mut Portal, _| this.drag = None);
                    }
                });
            })
            .absolute()
            .size_0()
        };
        div()
            .id("portal")
            .track_focus(&self.focus)
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
            // A cell's margin, so the body starts on the cell grid: a cell's middle never falls on
            // the edge between two rows.
            .child(div().flex().flex_1().min_h(px(0.)).px(px(self.cell.w)).py(px(self.cell.h)).child(body))
            .child(div().absolute().bottom(px(32.)).right(px(16.)).flex().flex_col().gap_1().children(toasts))
            .child(div().h(px(ROW)).px_3().text_xs().text_color(lk.dim).bg(lk.panel).child(status))
            .child(handover)
    }
}

/// The inks of Markdown's roles in an editor, from the theme's `md.*` parts.
fn md_inks<'a>(lk: &'a Look<'a>) -> impl Fn(Md) -> Ink + 'a {
    move |role| {
        let ink = |fg: Hsla| Ink { fg: Some(fg), ..Ink::default() };
        match role {
            Md::Text => Ink::default(),
            Md::Heading => lk.part("md.heading", Ink { bold: true, ..ink(lk.accent) }),
            Md::Bullet => lk.part("md.bullet", ink(lk.accent)),
            Md::Quote => lk.part("md.quote", Ink { italic: true, ..ink(lk.dim) }),
            Md::Code => lk.part("md.code", ink(lk.pos)),
            Md::Task => lk.part("md.task", Ink { bold: true, ..ink(lk.accent) }),
            Md::Done => lk.part("md.done", Ink { strike: true, ..ink(lk.dim) }),
            Md::Link => lk.part("md.link", Ink { underline: true, ..ink(lk.accent) }),
        }
    }
}

/// The rows of a list or table to draw: from its first shown row (`state::first_row`), as many
/// as the last frame said fit, and one more for a row partly in view. Before any frame has said,
/// up to 200 rows (clipped to what shows).
fn shown_rows(st: &State, id: &str, len: usize, per: u16) -> std::ops::Range<usize> {
    let local = st.local.get(id);
    let fit = local.map_or(0, |l| l.rows as usize);
    let fit = if fit == 0 { 200 / per.max(1) as usize } else { fit };
    let from = state::first_row(local, len, fit);
    from..(from + fit + 1).min(len)
}

/// A selected row's ink, focused or not.
fn selected_ink(lk: &Look, focused: bool) -> Ink {
    if focused {
        lk.part("row.selected", Ink { fg: Some(lk.accent), bg: Some(lk.ring), ..Ink::default() })
    } else {
        lk.part("row.selected.blur", Ink { bg: Some(lk.panel), ..Ink::default() })
    }
}

/// A toast, in its tone's colour.
fn toast(lk: &Look, t: &state::Toast) -> AnyElement {
    let ink = lk.part("toast", Ink { fg: Some(lk.fg), bg: Some(lk.panel), ..Ink::default() });
    let edge = t.tone.as_deref().and_then(|c| lk.color(c)).unwrap_or(lk.accent);
    inked(lk.round(div().px_3().py_1().border_1().border_color(edge), 6.), ink).child(t.text.clone()).into_any_element()
}

/// What a painted frame laid out, to the engine, in cells.
fn handover(laid: &Shared, shown: &RefCell<DrawnHits>, engine: &Engine, cell: Cell, wake: Option<u64>, took: Duration, window: &mut Window) {
    let l = std::mem::take(&mut *laid.borrow_mut());
    let drawn = Drawn {
        rects: l.rects.into_iter().map(|(id, b)| (id, cell.rect(b))).collect(),
        hits: l.hits.iter().map(|(b, h)| Hit { rect: cell.rect(*b), ..h.clone() }).collect(),
        sizes: l.sizes.into_iter().map(|(id, b, per)| { let (w, h) = cell.fit(b); (id, w, h / per.max(1)) }).collect(),
        areas: l.areas.into_iter().map(|(id, b)| { let (w, h) = cell.fit(b); (id, w, h) }).collect(),
        wake,
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
