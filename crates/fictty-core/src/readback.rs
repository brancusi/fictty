//! The screen as an agent reads it: the shown nodes in order, each with its kind, id, the text it
//! shows, focus and selection, and where it was laid out when a frame says so. Structure, not a
//! grid of characters: it reads the same whatever draws the state (a terminal, a window).

use crate::engine::Rect;
use crate::model::{Kind, Node, fill, scalar, segments, shows};
use crate::state::{self, State};
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
    /// Where the last frame laid it out (nodes with an id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<Rect>,
}

/// The shown nodes of `st`, in order, with the rectangles the last frame laid them out in.
pub fn readback(st: &State, rects: &[(String, Rect)]) -> Vec<Shown> {
    let mut out = Vec::new();
    walk(st, st.tree(), 0, rects, &mut out);
    out
}

fn walk(st: &State, n: &Node, depth: usize, rects: &[(String, Rect)], out: &mut Vec<Shown>) {
    if !state::shown(st, n) {
        return;
    }
    out.push(shown(st, n, depth, rects));
    match &n.kind {
        Kind::Tabs { children, .. } => {
            if let Some(c) = children.get(state::tab_of(st, n).min(children.len().saturating_sub(1))) {
                walk(st, c, depth + 1, rects, out);
            }
        }
        _ => {
            for c in n.children() {
                walk(st, c, depth + 1, rects, out);
            }
        }
    }
}

/// A template's lines filled from `v`, leaving out lines whose placeholders all came out empty.
fn lines(template: &str, v: &Value) -> String {
    let shown: Vec<String> = template.split('\n').filter(|l| shows(l, v)).map(|l| segments(l, v).into_iter().map(|s| s.text).collect()).collect();
    shown.join("\n")
}

fn shown(st: &State, n: &Node, depth: usize, rects: &[(String, Rect)]) -> Shown {
    let v = state::bound(st, n).unwrap_or(&Value::Null);
    let id = n.id.clone();
    let mut s = Shown {
        depth,
        kind: n.kind_name().into(),
        title: n.title.as_deref().map(|t| fill(t, v)),
        text: String::new(),
        focus: id.is_some() && id == st.focus,
        rows: None,
        selected: None,
        rect: id.as_deref().and_then(|i| rects.iter().find(|(r, _)| r == i)).map(|(_, r)| *r),
        id,
    };
    match &n.kind {
        Kind::Text { text } | Kind::Big { text } | Kind::Spinner { text } => s.text = lines(text, v),
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
                s.text = match (&rows.item, items.get(at)) {
                    (Some(t), Some(r)) => lines(t, r),
                    (None, r) => scalar(r),
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
        Kind::Tabs { tabs, .. } => {
            s.rows = Some(tabs.len());
            s.selected = Some(state::tab_of(st, n).min(tabs.len().saturating_sub(1)));
            s.text = tabs.iter().map(|t| fill(t, v)).collect::<Vec<_>>().join(" | ");
        }
        Kind::Toasts {} => s.text = st.toasts.iter().map(|t| t.text.clone()).collect::<Vec<_>>().join("\n"),
        _ => {}
    }
    s
}

/// The readback as an outline, a line per node, indented by depth:
/// `button save (focus) [3,10 8×1]: Save`.
pub fn outline(shown: &[Shown]) -> String {
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
        match (s.selected, s.rows) {
            (Some(k), Some(n)) => line.push_str(&format!(" ({}/{n})", k + 1)),
            (None, Some(n)) => line.push_str(&format!(" (0/{n})")),
            _ => {}
        }
        if let Some(r) = s.rect {
            line.push_str(&format!(" [{},{} {}×{}]", r.x, r.y, r.width, r.height));
        }
        if !s.text.is_empty() {
            line.push_str(": ");
            line.push_str(&s.text.replace('\n', " ⏎ "));
        }
        out.push_str(&line);
        out.push('\n');
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
}
