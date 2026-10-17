//! The screen as an agent reads it: the shown nodes in order, each with its kind, id, the text it
//! shows, focus and selection, and where it was laid out when a frame says so. Structure, not a
//! grid of characters: it reads the same whatever draws the state (a terminal, a window).

use crate::engine::Rect;
use crate::model::{Kind, Node, fill, scalar, segments, shows};
use crate::state::{self, State};
use crate::style::{self, Resolved};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One shown node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shown {
    /// How deep in the tree (the root is 0).
    pub depth: usize,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Its title, filled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// What it shows: a text's lines, a button's label, an editor's text, a list's selected row,
    /// a tab bar's titles.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub focus: bool,
    /// A list's, tree's or table's rows, and which is selected; a tab bar's tabs and which shows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<usize>,
    /// Where a list's view is and how many lines it has, while it has more than it shows: lines
    /// scrolled past (fractions too) and the lines in all. A scrolling container's is in cells
    /// along it: scrolled past, and what it holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll: Option<(f32, usize)>,
    /// How many rows a list lays out to a line, while it's more than one (its `cols`, counted
    /// for the width it had when they fit as many as there's room for).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cols: Option<usize>,
    /// How many entries an editor over a multi-selection edits, and whether they differ.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub entries: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mixed: bool,
    /// A `multi` list's picked rows, by index, while more than one is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub picked: Vec<usize>,
    /// Where the last frame laid it out (nodes with an id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<Rect>,
    /// Whether the last frame cut some of its text to fit, ending it in `…`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cut: bool,
    /// Its style under the active theme (see `style::resolve`).
    #[serde(default, skip_serializing_if = "plain")]
    pub style: Resolved,
}

fn plain(r: &Resolved) -> bool {
    *r == Resolved::default()
}

/// The shown nodes of `st`, in order, with the rectangles the last frame laid them out in and
/// whether it cut their text.
pub fn readback(st: &State, rects: &[(String, Rect)], cut: &[String]) -> Vec<Shown> {
    let mut out = Vec::new();
    walk(st, st.tree(), 0, "root", (rects, cut), &mut out);
    out
}

type Frame<'a> = (&'a [(String, Rect)], &'a [String]);

fn walk(st: &State, n: &Node, depth: usize, key: &str, frame: Frame, out: &mut Vec<Shown>) {
    if !state::shown(st, n) {
        return;
    }
    // A node without an id goes by where it sits, as the window names it: the nearest id (or
    // `root`) and the way down from there, so a cut in it shows too.
    let key = n.id.as_deref().unwrap_or(key);
    out.push(shown(st, n, depth, key, frame));
    match &n.kind {
        Kind::Tabs { tabs, .. } => {
            let at = state::tab_of(st, n).min(tabs.len().saturating_sub(1));
            if let Some(c) = n.page(at) {
                walk(st, c, depth + 1, &format!("{key}.{at}"), frame, out);
            }
        }
        _ => {
            for (k, c) in n.children().iter().enumerate() {
                walk(st, c, depth + 1, &format!("{key}.{k}"), frame, out);
            }
        }
    }
}

/// A template's lines filled from `v`, leaving out lines whose placeholders all came out empty.
/// A tab stays a tab: what's after it is the line's right side.
fn lines(template: &str, v: &Value) -> String {
    let shown: Vec<String> = template.split('\n').filter(|l| shows(l, v)).map(|l| segments(l, v).into_iter().map(|s| if s.tab { "\t".to_string() } else { s.text }).collect()).collect();
    shown.join("\n")
}

fn shown(st: &State, n: &Node, depth: usize, key: &str, (rects, cut): Frame) -> Shown {
    let scope = state::scope(st, n);
    let v = scope.as_ref();
    let id = n.id.clone();
    let mut s = Shown {
        depth,
        kind: n.kind_name().into(),
        title: n.title.as_deref().map(|t| fill(t, v)),
        text: String::new(),
        focus: id.is_some() && id == st.focus,
        rows: None,
        selected: None,
        picked: s_picked(st, n),
        scroll: s_scroll(st, n),
        cols: Some(state::per_line(st, n)).filter(|c| *c > 1),
        entries: if n.field().is_some() { state::doc_keys(st, n).len() } else { 0 },
        mixed: state::doc_key(st, n).is_some_and(|k| k.starts_with(state::MIXED)),
        rect: id.as_deref().and_then(|i| rects.iter().find(|(r, _)| r == i)).map(|(_, r)| *r),
        style: style::resolve(st, n),
        cut: cut.iter().any(|c| c == key),
        id,
    };
    match &n.kind {
        Kind::Text { text, .. } | Kind::Big { text } | Kind::Spinner { text } => s.text = lines(text, v),
        Kind::Button { label } => s.text = fill(label, v),
        Kind::Editor { .. } | Kind::Input { .. } => {
            s.text = s
                .id
                .as_deref()
                .and_then(|i| state::field_view(st, i))
                .and_then(|(k, _)| st.store.get(&k))
                .map(|e| e.read()["text"].as_str().unwrap_or_default().to_string())
                .unwrap_or_default();
        }
        Kind::List { rows, .. } | Kind::Tree { rows, .. } => {
            let items = state::items(st, n);
            let at = s.id.as_deref().map_or(0, |i| state::selected(st, i));
            s.rows = Some(items.len());
            if !items.is_empty() {
                s.selected = Some(at);
                s.text = match (&rows.item, &rows.row, items.get(at)) {
                    (Some(t), _, Some(r)) => lines(t, r),
                    (None, Some(row), Some(r)) => row_text(st, row, r, (at, items.len())),
                    (None, _, r) => scalar(r),
                    _ => String::new(),
                };
            }
        }
        Kind::Table { columns, .. } => {
            let items = state::items(st, n);
            let at = s.id.as_deref().map_or(0, |i| state::selected(st, i));
            s.rows = Some(items.len());
            if let Some(r) = items.get(at) {
                s.selected = Some(at);
                s.text = columns.iter().map(|c| fill(&c.value, r)).collect::<Vec<_>>().join(" · ");
            }
        }
        Kind::Tabs { tabs, lead, trail, .. } => {
            s.rows = Some(tabs.len());
            s.selected = Some(state::tab_of(st, n).min(tabs.len().saturating_sub(1)));
            // The titles, between the bar's lead and trail when it has them.
            let titles = tabs.iter().map(|t| fill(t, v)).collect::<Vec<_>>().join(" | ");
            s.text = [lead.as_deref().map(|t| fill(t, v)), Some(titles), trail.as_deref().map(|t| fill(t, v))].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        }
        Kind::Gauge { value, of, label } => s.text = crate::model::gauge_label(value, of.as_ref(), label.as_deref(), v),
        Kind::Image { .. } => s.text = crate::state::image_src(st, n).unwrap_or_default(),
        Kind::Toasts {} => s.text = st.toasts.iter().map(|t| t.text.clone()).collect::<Vec<_>>().join("\n"),
        _ => {}
    }
    s
}

/// The readback as an outline, a line per node, indented by depth:
/// What a row drawn from a node template shows: its texts, labels and pictures' files, in order,
/// leaving out what a `when` hides.
fn row_text(st: &State, template: &Node, row: &Value, at: (usize, usize)) -> String {
    let n = crate::model::row_instance(template, row, at);
    let mut all = Vec::new();
    n.walk_shown(&|n| state::tab_of(st, n), &|n| state::shown(st, n), &mut all);
    let parts: Vec<String> = all
        .into_iter()
        .filter_map(|n| match &n.kind {
            Kind::Text { text, .. } => Some(lines(text, &Value::Null)),
            Kind::Button { label } => Some(fill(label, &Value::Null)),
            Kind::Image { src, .. } => Some(crate::model::fill_raw(src, &Value::Null)),
            _ => None,
        })
        .filter(|t| !t.is_empty())
        .collect();
    parts.join(" · ")
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// A list's view and its lines, while it has more lines than it shows; a scroller's offset and
/// content, while it holds more than its view.
fn s_scroll(st: &State, n: &Node) -> Option<(f32, usize)> {
    let local = st.local.get(n.id.as_deref()?)?;
    if n.scrolls_down().is_some() {
        let (view, content) = local.extent?;
        return (content > view + 0.5).then(|| ((local.offset * 10.).round() / 10., content.round() as usize));
    }
    n.rows()?;
    let lines = state::Lines::of(st, n);
    let at = lines.offset(Some(local));
    (local.rows > 0 && lines.count > lines.fitting_from(at.floor() as usize)).then(|| ((at * 100.).round() / 100., lines.count))
}

/// A `multi` node's picked rows while it has more than one.
fn s_picked(st: &State, n: &Node) -> Vec<usize> {
    let Some(id) = n.id.as_deref().filter(|_| n.multi) else { return vec![] };
    let p = state::picked(st, id);
    if p.len() > 1 { p } else { vec![] }
}

/// `button save (focus) [3,10 8×1]: Save`.
pub fn outline(shown: &[Shown]) -> String {
    outline_with(shown, false)
}

/// The outline with each node's resolved style on a line under it:
/// `  ┊ button.focused · #0f1418+on-#5fd1c6+b`.
pub fn outline_styled(shown: &[Shown]) -> String {
    outline_with(shown, true)
}

fn outline_with(shown: &[Shown], styled: bool) -> String {
    let mut out = String::new();
    for s in shown {
        let mut line = format!("{}{}", "  ".repeat(s.depth), s.kind);
        if let Some(id) = &s.id {
            line.push(' ');
            line.push_str(id);
        }
        if let Some(t) = &s.title {
            line.push_str(&format!(" \"{t}\""));
        }
        if s.focus {
            line.push_str(" (focus)");
        }
        if s.cut {
            line.push_str(" (cut)");
        }
        if s.style.states.iter().any(|x| x == "on") {
            line.push_str(" (on)");
        }
        match (s.selected, s.rows) {
            (Some(k), Some(n)) => line.push_str(&format!(" ({}/{n})", k + 1)),
            (None, Some(n)) => line.push_str(&format!(" (0/{n})")),
            _ => {}
        }
        if let Some(c) = s.cols {
            line.push_str(&format!(" ({c} to a line)"));
        }
        if let Some((at, of)) = s.scroll {
            line.push_str(&format!(" (scroll {at}/{of})"));
        }
        if s.entries > 1 {
            line.push_str(&format!(" ({} entries{})", s.entries, if s.mixed { ", mixed" } else { "" }));
        }
        if !s.picked.is_empty() {
            line.push_str(&format!(" (picked {})", s.picked.len()));
        }
        if let Some(r) = s.rect {
            line.push_str(&format!(" [{},{} {}×{}]", r.x, r.y, r.width, r.height));
        }
        if !s.text.is_empty() {
            line.push_str(": ");
            line.push_str(&s.text.replace('\n', " ⏎ ").replace('\t', "   "));
        }
        out.push_str(&line);
        out.push('\n');
        let style = s.style.line();
        if styled && !style.is_empty() {
            out.push_str(&format!("{}  ┊ {style}\n", "  ".repeat(s.depth)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::App;
    use serde_json::json;

    #[test]
    fn a_screen_without_a_frame_reads_back_its_nodes() {
        let ui = serde_json::from_value(json!({
            "data": {"fruit": {"value": [{"name": "apple"}, {"name": "pear"}]}},
            "root": {"type": "col", "children": [
                {"type": "text", "text": "Fruit\n{missing}"},
                {"type": "list", "id": "l", "bind": "fruit", "item": "{name}", "title": "Pick"},
                {"type": "button", "id": "go", "label": "Go"}
            ]},
            "focus": "l"
        }))
        .unwrap();
        let mut s = App::new(ui).headless(40, 10).unwrap();
        s.keys(&["j"]);
        assert_eq!(s.screen(), "col\n  text: Fruit\n  list l \"Pick\" (focus) (2/2): pear\n  button go: Go\n");
        let nodes = s.readback();
        assert_eq!(nodes[2].selected, Some(1));
        assert_eq!(nodes[2].rect, None);
    }

    #[test]
    fn the_readback_carries_each_nodes_style() {
        let ui = serde_json::from_value(json!({
            "root": {"type": "col", "children": [
                {"type": "button", "id": "go", "label": "Go"},
                {"type": "text", "title": "Note", "tone": "pos", "text": "saved"}
            ]}
        }))
        .unwrap();
        let mut s = App::new(ui).headless(40, 10).unwrap();
        assert_eq!(s.readback()[1].style.ink.bg.as_deref(), Some("#5fd1c6"), "a focused button, on the accent");
        assert_eq!(
            crate::readback::outline_styled(&s.readback()),
            "col\n  button go (focus): Go\n    ┊ button.focused · #0f1418+on-#5fd1c6+b · pad 12px 0 · radius 6 · cursor pointer\n  text \"Note\": saved\n    ┊ text · #8bd49c+on-#161d22 · pad 8px · radius 8 · border 1 #2a353d · title #93a3ad+b\n"
        );
    }
}
