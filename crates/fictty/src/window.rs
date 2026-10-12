//! The native window (`fictty run --window`): the core's engine on its own thread, the state drawn
//! by GPUI in pixels each frame, every input going in as a fictty `Msg`. This is the spike of
//! 11 October brought onto `main` so each step of the move (docs/design/native.md) is visible:
//! a subset of the components, one palette, no themes yet. Phase 1 brings it to parity.

use crate::engine::{Drawn, Live};
use crate::model::{Kind, Node, Size};
use crate::state::{self, Msg, State};
use anyhow::Result;
use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, DefiniteLength, FocusHandle, FontWeight, Hsla, KeyDownEvent, ScrollStrategy, SharedString, UniformListScrollHandle,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, relative, rgb, size, uniform_list,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A fictty row, in pixels: what a size in cells turns into.
const ROW: f32 = 24.0;
/// A fictty column, in pixels.
const COL: f32 = 9.0;

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
fn sized(el: gpui::Stateful<gpui::Div>, n: &Node, along_row: bool) -> gpui::Stateful<gpui::Div> {
    match &n.size {
        Some(Size::Cells(c)) if along_row => el.flex_none().w(px(*c as f32 * COL)),
        Some(Size::Cells(c)) => el.flex_none().h(px(*c as f32 * ROW)),
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
        let mut el = sized(div().id(SharedString::from(format!("n-{key}"))).flex().flex_col().overflow_hidden(), n, along_row);
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
                let d = div().flex().size_full().min_h(px(0.)).min_w(px(0.)).gap(px(gap.unwrap_or(0) as f32 * if row { COL } else { ROW }));
                let d = if row { d.flex_row() } else { d.flex_col() };
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
            Kind::Spinner { text } => div().text_color(p.dim).child(format!("◌ {}", filled(text, v))).into_any_element(),
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
                let head = div().flex().flex_row().text_sm().text_color(p.dim).children(columns.iter().map(|c| div().flex_1().child(c.title.clone())));
                let body: Vec<AnyElement> = rows
                    .iter()
                    .enumerate()
                    .take(200)
                    .map(|(k, r)| {
                        let target = id.clone();
                        div()
                            .id(k)
                            .flex()
                            .flex_row()
                            .when(k == sel, |d| d.bg(if focused { p.selected } else { p.panel }))
                            .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Click { id: target.clone(), row: Some(k), tab: None }, cx)))
                            .children(columns.iter().map(|c| div().flex_1().child(filled(&c.value, r))))
                            .into_any_element()
                    })
                    .collect();
                div().flex().flex_col().child(head).children(body).into_any_element()
            }
            Kind::Editor { .. } | Kind::Input { .. } => self.editor(st, n, &id, focused, cx),
            Kind::Tabs { tabs, children, .. } => {
                let at = state::tab_of(st, n).min(children.len().saturating_sub(1));
                let tabs = tabs.clone();
                let child = children.get(at).cloned();
                let bar = div().flex().flex_row().gap_2().children(tabs.iter().enumerate().map(|(k, t)| {
                    let target = id.clone();
                    div()
                        .id(SharedString::from(format!("tab-{key}-{k}")))
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
        uniform_list(
            SharedString::from(format!("list-{id}")),
            count,
            // Only the rows in view are built, read from the state where they are.
            cx.processor(move |this: &mut Portal, range: std::ops::Range<usize>, _window, cx| {
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
                        div()
                            .id(k)
                            .h(px(ROW))
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
        .flex_1()
        .into_any_element()
    }

    fn editor(&mut self, st: &State, n: &Node, id: &str, focused: bool, cx: &mut Context<Self>) -> AnyElement {
        let p = palette();
        let Some((key, view)) = state::field_view(st, id) else { return div().text_color(p.dim).child("…").into_any_element() };
        let text = st.store.get(&key).map(|e| e.read()["text"].as_str().unwrap_or_default().to_string()).unwrap_or_default();
        let caret = view.selection.primary().head;
        let target = id.to_string();
        let single = n.field().is_some_and(|f| f.line);
        let mut at = 0;
        let lines: Vec<AnyElement> = text
            .split('\n')
            .map(|line| {
                let len = line.chars().count();
                let here = focused && caret >= at && caret <= at + len;
                let el = if here {
                    let (a, b): (String, String) = (line.chars().take(caret - at).collect(), line.chars().skip(caret - at).collect());
                    div().flex().flex_row().h(px(ROW)).child(a).child(div().w(px(2.)).h(px(ROW - 6.)).mt(px(3.)).bg(p.accent)).child(b)
                } else {
                    div().h(px(ROW)).child(if line.is_empty() { " ".to_string() } else { line.to_string() })
                };
                at += len + 1;
                el.into_any_element()
            })
            .collect();
        div()
            .id(SharedString::from(format!("ed-{id}")))
            .flex()
            .flex_col()
            .size_full()
            .px_2()
            .when(single, |d| d.bg(p.panel).rounded_md().border_1().border_color(if focused { p.accent } else { p.border }))
            .overflow_y_scroll()
            .on_click(cx.listener(move |this, _, _, cx| this.send(Msg::Focus { id: target.clone() }, cx)))
            .children(lines)
            .into_any_element()
    }
}

impl Render for Portal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = Instant::now();
        let p = palette();
        let live = self.live.clone();
        let mut live = live.lock().unwrap();
        let asked = live.take_screens();
        // What the screen copied goes on the system clipboard.
        if let Some(text) = live.take_copied().pop() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        let st = live.state();
        let body = self.node(st, st.tree(), false, "root", cx);
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
        let out = div()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| {
                if let Some(key) = key_name(e) {
                    this.send(Msg::Key { key }, cx);
                }
            }))
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(p.bg)
            .text_color(p.fg)
            .text_size(px(14.))
            .child(div().flex().flex_1().min_h(px(0.)).p_3().child(body))
            .child(div().absolute().bottom(px(32.)).right(px(16.)).flex().flex_col().gap_1().children(toasts))
            .child(div().h(px(24.)).px_3().text_xs().text_color(p.dim).bg(p.panel).child(status));
        // Rectangles and editor sizes come from layout in phase 1; until then the readback has
        // none.
        live.drew(t.elapsed(), Drawn::default());
        out
    }
}

/// The window's size to start with, in pixels.
const START: (f32, f32) = (1200.0, 800.0);

/// Run `app` in a native window until it quits or the window closes.
pub fn run(app: crate::App) -> Result<()> {
    let cells = ((START.0 / COL) as u16, (START.1 / ROW) as u16);
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
                Portal { live, focus, scrolls: HashMap::new(), shown: HashMap::new() }
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
