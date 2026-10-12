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

use crate::engine::{Drawn, Hit, Input, Live, Rect};
use crate::model::{Kind, Node, Size};
use crate::state::{self, Msg, State};
use anyhow::Result;
use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, DefiniteLength, FocusHandle, FontWeight, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Point, ScrollStrategy, ScrollWheelEvent, SharedString, UniformListScrollHandle, Window, WindowBounds, WindowOptions, canvas, div, font, prelude::*, px, relative, rgb, size,
    uniform_list,
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
    hits: Vec<(Bounds<Pixels>, Hit)>,
    /// Each editor's text area and each list's rows, as bounds to fit cells into.
    sizes: Vec<(String, Bounds<Pixels>)>,
    /// The area each node whose `when` reads `$width` or `$height` was laid out in.
    areas: Vec<(String, Bounds<Pixels>)>,
}

type Shared = Rc<RefCell<Laid>>;

/// What's under the mouse in `id`: the whole node, or one row or tab of it, or an editor's text.
fn hit(id: String, row: Option<usize>, tab: Option<usize>, text: bool) -> Hit {
    Hit { rect: Rect::default(), id, row, tab, text }
}

/// A canvas that notes, as it's laid out, the bounds it filled: absolute, so it takes no room.
fn measure(laid: &Shared, note: impl FnOnce(&mut Laid, Bounds<Pixels>) + 'static) -> AnyElement {
    let laid = laid.clone();
    canvas(move |b, _, _| note(&mut laid.borrow_mut(), b), |_, _, _, _| {}).absolute().inset_0().into_any_element()
}

#[derive(Clone, Copy)]
struct Palette {
    bg: Hsla,
    panel: Hsla,
    fg: Hsla,
    dim: Hsla,
    accent: Hsla,
    border: Hsla,
    selected: Hsla,
}

fn palette() -> Palette {
    Palette {
        bg: rgb(0x0f1418).into(),
        panel: rgb(0x161d22).into(),
        fg: rgb(0xe3eaee).into(),
        dim: rgb(0x93a3ad).into(),
        accent: rgb(0x5fd1c6).into(),
        border: rgb(0x2a353d).into(),
        selected: rgb(0x1d3b3a).into(),
    }
}

/// A fictty colour name (`cyan`, `#rrggbb`) as GPUI's.
fn color(name: &str, p: &Palette) -> Option<Hsla> {
    if let Some(hex) = name.strip_prefix('#') {
        return u32::from_str_radix(hex, 16).ok().map(|c| rgb(c).into());
    }
    let c = match name {
        "accent" => return Some(p.accent),
        "dim" | "muted" => return Some(p.dim),
        "red" | "neg" => 0xf07178,
        "green" | "pos" => 0x8bd49c,
        "yellow" => 0xf0c674,
        "blue" => 0x6cb6ff,
        "magenta" => 0xd291e4,
        "cyan" => 0x5fd1c6,
        "white" => 0xffffff,
        "gray" | "grey" | "darkgray" => 0x7a8791,
        "black" => 0x000000,
        _ => return None,
    };
    Some(rgb(c).into())
}

struct Portal {
    /// fictty's engine, on its own thread: it takes in sources and requests as fast as they
    /// come; the window reads its state when it draws.
    live: Arc<Mutex<Live>>,
    focus: FocusHandle,
    cell: Cell,
    /// The window's size in cells, as last told to the engine.
    screen: (u16, u16),
    /// What the frame being built lays out, handed to the engine once it has painted.
    laid: Shared,
    /// Each editor's text area in the last painted frame, for the mouse.
    texts: Rc<RefCell<HashMap<String, Bounds<Pixels>>>>,
    /// The editor a press in its text started a drag in, until the button comes up.
    drag: Option<String>,
    /// The wheel's travel over an editor not yet a whole scroll step.
    wheel: f32,
    /// When (on the logical clock) the frame being built changes by itself next.
    wake: Option<u64>,
    /// Each list's scroll, by id: the window's own view state, as a terminal's scrollback is.
    scrolls: HashMap<String, UniformListScrollHandle>,
    /// The last selection each list was scrolled to.
    shown: HashMap<String, usize>,
}

impl Portal {
    fn send(&mut self, msg: Msg, cx: &mut Context<Self>) {
        self.live.lock().unwrap().msg(msg);
        cx.notify();
    }

    /// The frame changes by itself at `at` (a spinner turning, a toast going).
    fn soon(&mut self, at: u64) {
        self.wake = Some(self.wake.map_or(at, |w| w.min(at)));
    }

    /// The cell `at` falls on in editor `id`'s text, in the last painted frame.
    fn in_text(&self, id: &str, at: Point<Pixels>) -> Option<(u16, u16)> {
        self.texts.borrow().get(id).map(|b| self.cell.within(*b, at))
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

/// What a template shows, filled from `v`.
fn filled(template: &str, v: &Value) -> String {
    crate::model::segments(template, v).into_iter().map(|s| s.text).collect()
}

/// A node's share of its row or column, as flexbox: cells become pixels, `30%` a fraction,
/// `*` and `2*` a grow factor.
fn sized(el: gpui::Stateful<gpui::Div>, n: &Node, along_row: bool, cell: Cell) -> gpui::Stateful<gpui::Div> {
    match &n.size {
        Some(Size::Cells(c)) if along_row => el.flex_none().w(px(*c as f32 * cell.w)),
        Some(Size::Cells(c)) => el.flex_none().h(px(*c as f32 * cell.h)),
        Some(Size::Spec(s)) if s.ends_with('%') => {
            let f = s.trim_end_matches('%').parse::<f32>().unwrap_or(100.0) / 100.0;
            let l: DefiniteLength = relative(f);
            if along_row { el.flex_none().w(l) } else { el.flex_none().h(l) }
        }
        Some(Size::Spec(s)) => {
            let grow = s.trim_end_matches('*').parse::<f32>().unwrap_or(1.0);
            let mut el = el.flex_basis(px(0.)).min_w(px(0.)).min_h(px(0.)).flex_shrink();
            el.style().flex_grow = Some(grow);
            el
        }
        None => el.flex_1().min_w(px(0.)).min_h(px(0.)),
    }
}

impl Portal {
    fn node(&mut self, st: &State, n: &Node, along_row: bool, path: &str, cx: &mut Context<Self>) -> AnyElement {
        if !state::shown(st, n) {
            return div().into_any_element();
        }
        let p = palette();
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
        if let Some(s) = &n.style {
            if let Some(c) = s.fg.as_deref().and_then(|c| color(c, &p)) {
                el = el.text_color(c);
            }
            if let Some(c) = s.bg.as_deref().and_then(|c| color(c, &p)) {
                el = el.bg(c);
            }
            if s.bold {
                el = el.font_weight(FontWeight::BOLD);
            }
        }
        let v = state::bound(st, n).unwrap_or(&Value::Null);
        // A title or a border: a panel with rounded corners, the title on top.
        if n.title.is_some() || n.border {
            el = el.bg(p.panel).rounded_lg().border_1().border_color(if focused { p.accent } else { p.border }).p_2().gap_1();
            if let Some(t) = &n.title {
                el = el.child(div().text_sm().text_color(if focused { p.accent } else { p.dim }).font_weight(FontWeight::SEMIBOLD).child(filled(t, v)));
            }
        }
        let body: AnyElement = match &n.kind {
            Kind::Col { children, gap } | Kind::Row { children, gap } => {
                let row = matches!(n.kind, Kind::Row { .. });
                let kids: Vec<AnyElement> = children.iter().enumerate().map(|(k, c)| self.node(st, c, row, &format!("{key}.{k}"), cx)).collect();
                let d = div().relative().flex().size_full().min_h(px(0.)).min_w(px(0.)).gap(px(gap.unwrap_or(0) as f32 * if row { cell.w } else { cell.h }));
                let d = if row { d.flex_row() } else { d.flex_col() };
                // `$width` and `$height` in a child's `when` are this node's inner size.
                let sized: Vec<String> = children.iter().filter(|c| c.when.as_deref().is_some_and(state::sized)).filter_map(|c| c.id.clone()).collect();
                let d = if sized.is_empty() { d } else { d.child(measure(&self.laid, move |l, b| l.areas.extend(sized.into_iter().map(|id| (id, b))))) };
                d.children(kids).into_any_element()
            }
            Kind::Stack { children } => {
                let shown: Vec<(usize, Node)> = children.iter().enumerate().filter(|(_, c)| state::shown(st, c)).map(|(k, c)| (k, c.clone())).collect();
                let kids: Vec<AnyElement> = shown.iter().map(|(k, c)| div().absolute().inset_0().flex().child(self.node(st, c, false, &format!("{key}.{k}"), cx)).into_any_element()).collect();
                div().relative().size_full().children(kids).into_any_element()
            }
            Kind::Text { text } => {
                let lines: Vec<AnyElement> = text.split('\n').filter(|l| crate::model::shows(l, v)).map(|l| div().child(filled(l, v)).into_any_element()).collect();
                div().flex().flex_col().children(lines).into_any_element()
            }
            Kind::Spinner { text } => {
                let (glyph, next) = state::spinner(st.now_ms);
                self.soon(next);
                div().flex().flex_row().gap_2().child(div().text_color(p.accent).child(glyph)).child(filled(text, v)).into_any_element()
            }
            Kind::Button { label } => {
                let label = filled(label, v);
                let target = id.clone();
                div()
                    .id(SharedString::from(format!("btn-{key}")))
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(if focused { p.accent } else { p.panel })
                    .text_color(if focused { p.bg } else { p.fg })
                    .cursor_pointer()
                    .hover(|s| s.bg(p.selected))
                    .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: None, tab: None }, cx)))
                    .child(label)
                    .into_any_element()
            }
            Kind::Rule { .. } => div().h(px(1.)).w_full().bg(p.border).into_any_element(),
            Kind::List { rows, .. } | Kind::Tree { rows, .. } => self.list(st, n, &id, rows.item.clone(), focused, cx),
            Kind::Table { columns, .. } => {
                let rows = state::items(st, n).to_vec();
                let sel = state::selected(st, &id);
                let columns = columns.clone();
                let head = div().flex().flex_row().h(px(cell.h)).text_sm().text_color(p.dim).children(columns.iter().map(|c| div().flex_1().child(c.title.clone())));
                let body: Vec<AnyElement> = rows
                    .iter()
                    .enumerate()
                    .take(200)
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
                            .when(k == sel, |d| d.bg(if focused { p.selected } else { p.panel }))
                            .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: Some(k), tab: None }, cx)))
                            .children(columns.iter().map(|c| div().flex_1().child(filled(&c.value, r))))
                            .into_any_element()
                    })
                    .collect();
                // The rows it has room for, below the header.
                let fits = id.clone();
                let rows_area = div().relative().flex().flex_col().flex_1().min_h(px(0.)).child(measure(&self.laid, move |l, b| l.sizes.push((fits, b)))).children(body);
                div().flex().flex_col().size_full().child(head).child(rows_area).into_any_element()
            }
            Kind::Editor { .. } | Kind::Input { .. } => self.editor(st, n, &id, focused, cx),
            Kind::Tabs { tabs, children, .. } => {
                let at = state::tab_of(st, n).min(children.len().saturating_sub(1));
                let tabs = tabs.clone();
                let child = children.get(at).cloned();
                let bar = div().flex().flex_row().gap_2().children(tabs.iter().enumerate().map(|(k, t)| {
                    let target = id.clone();
                    let under = id.clone();
                    div()
                        .id(SharedString::from(format!("tab-{key}-{k}")))
                        .relative()
                        .child(measure(&self.laid, move |l, b| l.hits.push((b, hit(under, None, Some(k), false)))))
                        .px_2()
                        .rounded_md()
                        .cursor_pointer()
                        .when(k == at, |d| d.bg(p.selected).text_color(p.accent))
                        .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: None, tab: Some(k) }, cx)))
                        .child(filled(t, v))
                }));
                let shown = child.map(|c| self.node(st, &c, false, &format!("{key}.{at}"), cx)).unwrap_or_else(|| div().into_any_element());
                div().flex().flex_col().size_full().gap_1().child(bar).child(div().flex().flex_1().min_h(px(0.)).child(shown)).into_any_element()
            }
            Kind::Toasts {} => {
                if let Some(at) = st.toasts.iter().map(|t| t.until).min() {
                    self.soon(at);
                }
                let ts: Vec<AnyElement> = st.toasts.iter().rev().map(|t| div().px_2().rounded_md().bg(p.selected).child(t.text.clone()).into_any_element()).collect();
                div().flex().flex_col().gap_1().children(ts).into_any_element()
            }
            _ => div().text_color(p.dim).child(format!("[{}]", n.kind_name())).into_any_element(),
        };
        el = el.child(body);
        el.into_any_element()
    }

    fn list(&mut self, st: &State, n: &Node, id: &str, item: Option<String>, focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let p = palette();
        let count = state::items(st, n).len();
        let sel = state::selected(st, id);
        let handle = self.scrolls.entry(id.to_string()).or_default().clone();
        if self.shown.get(id) != Some(&sel) {
            handle.scroll_to_item(sel, ScrollStrategy::Top);
            self.shown.insert(id.to_string(), sel);
        }
        let target = id.to_string();
        let fits = id.to_string();
        let cell = self.cell;
        let list = uniform_list(
            SharedString::from(format!("list-{id}")),
            count,
            // Only the rows in view are built, read from the state where they are.
            cx.processor(move |this: &mut Portal, range: std::ops::Range<usize>, _window, cx| {
                let laid = this.laid.clone();
                let live = this.live.lock().unwrap();
                let st = live.state();
                let Some(n) = st.tree().find(&target) else { return Vec::new() };
                let rows = state::items(st, n);
                range
                    .filter(|k| *k < rows.len())
                    .map(|k| {
                        let r = &rows[k];
                        let text = match &item {
                            Some(t) => filled(t, r),
                            None => crate::model::scalar(Some(r)),
                        };
                        let target = target.clone();
                        let under = target.clone();
                        div()
                            .id(k)
                            .relative()
                            .child(measure(&laid, move |l, b| l.hits.push((b, hit(under, Some(k), None, false)))))
                            .h(px(cell.h))
                            .px_2()
                            .rounded_sm()
                            .when(k == sel, |d| d.bg(if focused { p.selected } else { p.panel }).text_color(if focused { p.accent } else { p.fg }))
                            .hover(|s| s.bg(p.panel))
                            .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: Some(k), tab: None }, cx)))
                            .child(text)
                    })
                    .collect()
            }),
        )
        .track_scroll(handle)
        .size_full();
        // The rows it has room for: the wheel and paging move by them.
        div().relative().flex_1().min_h(px(0.)).child(measure(&self.laid, move |l, b| l.sizes.push((fits, b)))).child(list).into_any_element()
    }

    /// An editor or input, drawn from caretline's own frame of it: the text at the monospace
    /// font's cell size, so the caret, selections, wrapping and clicks land where caretline puts
    /// them. The frame is the size the last frame laid the editor out in (`Msg::Size`).
    fn editor(&mut self, st: &State, n: &Node, id: &str, focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let p = palette();
        let cell = self.cell;
        let field = n.field().expect("a text field");
        let single = field.line;
        let doc = state::field_view(st, id).and_then(|(key, view)| match st.store.get(&key).map(|e| &e.body) {
            Some(crate::store::Body::Text { doc }) => Some((caretline::view::render(doc, view), doc.text.len_chars() == 0)),
            _ => None,
        });
        let mut text = div().relative().size_full().overflow_hidden();
        if single {
            text = text.h(px(cell.h)).flex_none();
        }
        let (fits, under) = (id.to_string(), id.to_string());
        text = text.child(measure(&self.laid, move |l, b| {
            l.sizes.push((fits, b));
            l.hits.push((b, hit(under, None, None, true)));
        }));
        match doc {
            Some((_, true)) | None => {
                let empty = field.placeholder.unwrap_or(if doc.is_none() { "…" } else { "" }).to_string();
                text = text.child(div().h(px(cell.h)).text_color(p.dim).italic().child(empty));
                if focused {
                    text = text.child(div().absolute().left(px(0.)).top(px(3.)).w(px(2.)).h(px(cell.h - 6.)).bg(p.accent));
                }
            }
            Some((frame, false)) => {
                use caretline::view::Role;
                for y in 0..frame.height {
                    // Runs of cells with one look, as spans.
                    let mut runs: Vec<(Role, String)> = Vec::new();
                    for x in 0..frame.width {
                        let c = frame.cell(x, y);
                        let role = match c.role {
                            Role::Selection | Role::Caret => c.role,
                            _ => Role::Text,
                        };
                        match runs.last_mut() {
                            Some((r, s)) if *r == role => s.push_str(&c.symbol),
                            _ => runs.push((role, c.symbol.to_string())),
                        }
                    }
                    let spans = runs.into_iter().map(|(role, s)| match role {
                        Role::Selection | Role::Caret => div().bg(p.selected).child(s),
                        _ => div().child(s),
                    });
                    text = text.child(div().flex().flex_row().h(px(cell.h)).whitespace_nowrap().children(spans));
                }
                if let Some((x, y)) = frame.cursor.filter(|_| focused) {
                    let (left, top) = (x as f32 * cell.w, y as f32 * cell.h);
                    text = text.child(div().absolute().left(px(left)).top(px(top + 3.)).w(px(2.)).h(px(cell.h - 6.)).bg(p.accent));
                }
            }
        }
        let (press, wheel) = (id.to_string(), id.to_string());
        div()
            .id(SharedString::from(format!("ed-{id}")))
            .flex()
            .flex_col()
            .size_full()
            .px_2()
            .font_family(MONO)
            .when(single, |d| d.justify_center().bg(p.panel).rounded_md().border_1().border_color(if focused { p.accent } else { p.border }))
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
            .on_scroll_wheel(cx.listener(move |this, e: &ScrollWheelEvent, _, cx| {
                // Each step scrolls three rows, as a terminal's wheel notch does.
                this.wheel -= f32::from(e.delta.pixel_delta(px(this.cell.h)).y);
                let step = 3.0 * this.cell.h;
                let by = (this.wheel / step).trunc();
                if by != 0.0 {
                    this.wheel -= by * step;
                    this.send(Msg::Scroll { id: wheel.clone(), by: by as isize }, cx);
                }
            }))
            .child(text)
            .into_any_element()
    }
}

impl Render for Portal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Instant::now();
        let p = palette();
        let live = self.live.clone();
        let mut live = live.lock().unwrap();
        // The window's size in cells: `$screen`, as a terminal's resize sends it.
        let vp = window.viewport_size();
        let screen = ((f32::from(vp.width) / self.cell.w) as u16, (f32::from(vp.height) / self.cell.h) as u16);
        if screen != self.screen {
            self.screen = screen;
            live.input(Input::Resize { width: screen.0, height: screen.1 });
        }
        *self.laid.borrow_mut() = Laid::default();
        self.wake = None;
        let asked = live.take_screens();
        // What the screen copied goes on the system clipboard.
        if let Some(text) = live.take_copied().pop() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        let st = live.state();
        let body = self.node(st, st.tree(), false, "root", cx);
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
        let toasts: Vec<AnyElement> = st.toasts.iter().rev().take(3).map(|t| div().px_3().py_1().rounded_md().bg(p.selected).text_color(p.fg).child(t.text.clone()).into_any_element()).collect();
        // Agents' `screen` requests get a readback of what's drawn.
        if !asked.is_empty() {
            let text = crate::readback::outline(&live.readback());
            for r in asked {
                let _ = r.send(json!({"ok": true, "screen": text}));
            }
        }
        // Once the frame has painted, what it laid out goes to the engine; if that changed the
        // state (an editor learned its size), another frame follows.
        let handover = {
            let (laid, texts, live, cell, wake) = (self.laid.clone(), self.texts.clone(), self.live.clone(), self.cell, self.wake);
            let took = t.elapsed();
            canvas(|_, _, _| {}, move |_, _, window, _| handover(&laid, &texts, &live, cell, wake, took, window)).absolute().size_0()
        };
        div()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                if let Some(key) = key_name(e) {
                    this.send(Msg::Key { key }, cx);
                }
            }))
            // A drag that started in an editor's text follows the mouse anywhere in the window.
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                let Some(id) = this.drag.clone() else { return };
                if !e.dragging() {
                    this.drag = None;
                    return;
                }
                if let Some((col, row)) = this.in_text(&id, e.position) {
                    this.send(Msg::Drag { id, col, row }, cx);
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _: &gpui::MouseUpEvent, _, _| this.drag = None))
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(p.bg)
            .text_color(p.fg)
            .text_size(px(TYPE))
            .line_height(px(ROW))
            .child(div().flex().flex_1().min_h(px(0.)).p_3().child(body))
            .child(div().absolute().bottom(px(32.)).right(px(16.)).flex().flex_col().gap_1().children(toasts))
            .child(div().h(px(ROW)).px_3().text_xs().text_color(p.dim).bg(p.panel).child(status))
            .child(handover)
    }
}

/// What a painted frame laid out, to the engine, in cells.
fn handover(laid: &Shared, texts: &RefCell<HashMap<String, Bounds<Pixels>>>, live: &Mutex<Live>, cell: Cell, wake: Option<u64>, took: Duration, window: &mut Window) {
    let l = std::mem::take(&mut *laid.borrow_mut());
    let drawn = Drawn {
        rects: l.rects.into_iter().map(|(id, b)| (id, cell.rect(b))).collect(),
        hits: l.hits.iter().map(|(b, h)| Hit { rect: cell.rect(*b), ..h.clone() }).collect(),
        sizes: l.sizes.into_iter().map(|(id, b)| { let (w, h) = cell.fit(b); (id, w, h) }).collect(),
        areas: l.areas.into_iter().map(|(id, b)| { let (w, h) = cell.fit(b); (id, w, h) }).collect(),
        wake,
    };
    *texts.borrow_mut() = l.hits.into_iter().filter(|(_, h)| h.text).map(|(b, h)| (h.id, b)).collect();
    if live.lock().expect("the engine").drew(took, drawn) {
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
            // A column is the monospace font's character width.
            let ts = window.text_system();
            let w = ts.ch_advance(ts.resolve_font(&font(MONO)), px(TYPE)).map(f32::from).unwrap_or(8.4);
            let cell = Cell { w, h: ROW };
            // Pictures and charts size themselves by the cell, as in a terminal.
            live.lock().expect("the engine").msg(Msg::Caps { caps: state::Caps { graphics: false, cell_w: cell.w, cell_h: cell.h }, text_sizing: false });
            cx.new(|cx| {
                let focus = cx.focus_handle();
                focus.focus(window);
                cx.spawn(async move |this, cx| {
                    use futures::StreamExt;
                    while rung.next().await.is_some() {
                        let go_on = this.update(cx, |p: &mut Portal, cx| {
                            let mut l = p.live.lock().expect("the engine");
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
                Portal {
                    live,
                    focus,
                    cell,
                    screen: (0, 0),
                    laid: Shared::default(),
                    texts: Rc::default(),
                    drag: None,
                    wheel: 0.0,
                    wake: None,
                    scrolls: HashMap::new(),
                    shown: HashMap::new(),
                }
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
