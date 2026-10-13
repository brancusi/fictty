//! Each shown node's style, worked out from the active theme: its part (the component's name),
//! with the parts for the states that hold layered over it (`button`, then `button.focused`),
//! then its variant, then its `tone`, then its own `style`, with tokens filled in as colours.
//! Whatever draws reads this one value, the readback prints it, and tests assert on it without a
//! window. There's no cascade beyond that: nothing else applies.

use crate::model::{Backdrop, Node, Rows, Style, ThemeSpec, Variant, is_colour};
use crate::state::{self, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The states a part can be suffixed with, in the order their parts layer: `row.selected.blur`
/// is `row`, then `row.selected`, then `row.blur`, then `row.selected.blur`. `active`,
/// `selected` and `disabled` don't hold for a node yet (nothing in `State` says so).
pub const STATES: [&str; 6] = ["hover", "focused", "active", "selected", "disabled", "blur"];

/// The parts the runtime draws inside a node, besides each component's own (its `type`).
const INNER: [&str; 27] = [
    "border", "title", "row", "empty", "status", "status.error", "toast", "table.header", "table.total", "tabs.active", "tabs.idle",
    "tabs.underline", "editor.caret", "editor.selection", "input.placeholder", "md.heading", "md.bullet", "md.quote", "md.code",
    "md.task", "md.done", "md.link", "scrollbar", "scrollbar.thumb", "tree.guide", "bars.label", "spinner",
];

/// The components that draw something of their own, so have a part: every `type` but the
/// containers, which only arrange (`row` is a list's row).
const COMPONENTS: [&str; 20] = [
    "text", "big", "spinner", "button", "list", "tree", "table", "tabs", "editor", "input", "toasts", "rule", "sparkline", "chart",
    "bars", "gauge", "plot", "image", "svg", "map",
];

/// Parts under every theme, where it doesn't say.
const DEFAULT_PARTS: [(&str, &str); 7] = [
    ("button", "fg+on-ring"),
    ("button.focused", "bg+on-accent+b"),
    ("title", "fg"),
    ("title.focused", "accent+b"),
    ("border", "border"),
    ("border.focused", "accent"),
    ("spinner", "accent"),
];

/// Tokens every theme has, where it doesn't say.
const DEFAULT_TOKENS: [(&str, u32); 10] = [
    ("bg", 0x0f1418),
    ("fg", 0xe3eaee),
    ("dim", 0x93a3ad),
    ("border", 0x2a353d),
    ("accent", 0x5fd1c6),
    ("panel", 0x161d22),
    ("ring", 0x1d3b3a),
    ("hover", 0x1a2329),
    ("pos", 0x8bd49c),
    ("neg", 0xf07178),
];

/// How a run of text looks: colours (`#rrggbb`) and the `b d i u r s` modifiers of a spec.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ink {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dim: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reverse: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strike: bool,
}

impl Ink {
    /// The ink written back as a spec: `#0f1418+on-#5fd1c6+b`.
    pub fn spec(&self) -> String {
        let mut words: Vec<String> = Vec::new();
        words.extend(self.fg.clone());
        words.extend(self.bg.as_ref().map(|b| format!("on-{b}")));
        for (on, m) in [(self.bold, "b"), (self.dim, "d"), (self.italic, "i"), (self.underline, "u"), (self.reverse, "r"), (self.strike, "s")] {
            if on {
                words.push(m.into());
            }
        }
        words.join("+")
    }
}

/// A node's style, resolved.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Resolved {
    /// The theme part it starts from: its `type`, or none for a container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// The states that hold, whose parts layer over it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
    /// Its text.
    #[serde(default, skip_serializing_if = "is_plain")]
    pub ink: Ink,
    /// How its border and corners are drawn, when not `rounded`: its own, its variant's or the
    /// theme's `edge`. Only for a node that draws a border, or a face (a button, a field, tabs,
    /// rows).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
    /// Its border's colour, when it draws one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    /// Its title's ink, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Ink>,
    /// Its backdrop, its own or its variant's, with colours filled in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop: Option<Backdrop>,
}

fn is_plain(ink: &Ink) -> bool {
    *ink == Ink::default()
}

impl Resolved {
    /// One line: `button.focused · #0f1418+on-#5fd1c6+b · border #5fd1c6 · edge double`.
    pub fn line(&self) -> String {
        let mut out = vec![];
        if let Some(p) = &self.part {
            out.push(std::iter::once(p.as_str()).chain(self.states.iter().map(String::as_str)).collect::<Vec<_>>().join("."));
        }
        if !is_plain(&self.ink) {
            out.push(self.ink.spec());
        }
        out.extend(self.border.as_ref().map(|b| format!("border {b}")));
        out.extend(self.title.as_ref().map(|t| format!("title {}", t.spec())));
        out.extend(self.edge.as_ref().map(|e| format!("edge {e}")));
        out.extend(self.backdrop.as_ref().map(|b| format!("backdrop {}", b.fill2.as_ref().map_or(b.fill.clone(), |f2| format!("{}→{f2}", b.fill)))));
        out.join(" · ")
    }
}

/// The active theme as the core reads it: its tokens over the UI's own `theme` map over the
/// defaults, and its parts over the default parts.
#[derive(Clone, Copy)]
pub struct Theme<'a> {
    pub active: Option<&'a ThemeSpec>,
    ui: &'a BTreeMap<String, String>,
}

impl<'a> Theme<'a> {
    pub fn of(st: &'a State) -> Theme<'a> {
        Theme { active: st.theme.as_ref().and_then(|n| st.themes.get(n)), ui: &st.ui.theme }
    }

    /// A token's value as written (`#5fd1c6`, `rounded`): the theme's, else the UI's.
    pub fn token(&self, name: &str) -> Option<&'a str> {
        self.active.and_then(|t| t.tokens.get(name)).or_else(|| self.ui.get(name)).map(String::as_str)
    }

    /// A token (`accent`, `muted` for `dim`) or a colour (`red`, `#ff8800`), as `#rrggbb`.
    pub fn colour(&self, name: &str) -> Option<String> {
        let name = if name == "muted" { "dim" } else { name };
        if let Some(c) = self.token(name).and_then(hex) {
            return Some(c);
        }
        match DEFAULT_TOKENS.iter().find(|(k, _)| *k == name) {
            Some((_, c)) => Some(format!("#{c:06x}")),
            None => hex(name),
        }
    }

    /// Whether `name` means a colour here: a token or a colour by itself.
    fn knows(&self, name: &str) -> bool {
        name == "muted" || self.token(name).is_some() || DEFAULT_TOKENS.iter().any(|(k, _)| *k == name) || is_colour(name) || hex(name).is_some()
    }

    /// A part's spec: the theme's, else the default.
    pub fn part(&self, name: &str) -> Option<&'a str> {
        self.active.and_then(|t| t.parts.get(name)).map(String::as_str).or_else(|| DEFAULT_PARTS.iter().find(|(k, _)| *k == name).map(|(_, v)| *v))
    }

    /// A spec over `base`: `accent+b`, `muted`, `fg+on-ring`, `#ff8800+u`. A word that's no colour
    /// but a part takes that part's ink (`<link>`).
    pub fn ink(&self, spec: &str, base: Ink) -> Ink {
        self.ink_at(spec, base, 0)
    }

    fn ink_at(&self, spec: &str, base: Ink, depth: u8) -> Ink {
        let mut ink = base;
        for w in spec.split('+').map(str::trim).filter(|w| !w.is_empty()) {
            match w {
                "b" => ink.bold = true,
                "d" => ink.dim = true,
                "i" => ink.italic = true,
                "u" => ink.underline = true,
                "r" => ink.reverse = true,
                "s" => ink.strike = true,
                w => match w.strip_prefix("on-") {
                    Some(c) => ink.bg = self.colour(c).or(ink.bg),
                    None => match (self.colour(w), self.part(w)) {
                        (Some(c), _) => ink.fg = Some(c),
                        (None, Some(part)) if depth < 4 => ink = self.ink_at(part, ink, depth + 1),
                        _ => {}
                    },
                },
            }
        }
        ink
    }

    /// The words of a spec that mean nothing here.
    fn unknown<'s>(&self, spec: &'s str) -> Vec<&'s str> {
        let modifier = |w: &str| matches!(w, "b" | "d" | "i" | "u" | "r" | "s");
        spec.split('+')
            .map(str::trim)
            .filter(|w| !w.is_empty() && !modifier(w))
            .filter(|w| match w.strip_prefix("on-") {
                Some(c) => !self.knows(c),
                None => !self.knows(w) && self.part(w).is_none(),
            })
            .collect()
    }

    /// `part` with the parts for `states` layered over it, over `base`.
    pub fn layered(&self, part: &str, states: &[&str], base: Ink) -> Ink {
        let mut ink = base;
        for name in names(part, states) {
            if let Some(spec) = self.part(&name) {
                ink = self.ink(spec, ink);
            }
        }
        ink
    }

    /// A node's own `style`, over `base`.
    fn style(&self, s: &Style, base: Ink) -> Ink {
        Ink {
            fg: s.fg.as_deref().and_then(|c| self.colour(c)).or(base.fg),
            bg: s.bg.as_deref().and_then(|c| self.colour(c)).or(base.bg),
            bold: base.bold || s.bold,
            dim: base.dim || s.dim,
            italic: base.italic || s.italic,
            ..base
        }
    }

    fn variant(&self, n: &Node) -> Option<&'a Variant> {
        self.active?.variants.get(n.variant.as_deref()?)
    }
}

/// A colour by itself as `#rrggbb`: `#rgb`, `#rrggbb`, or one of the named colours.
pub fn hex(name: &str) -> Option<String> {
    if let Some(h) = name.strip_prefix('#') {
        let h: String = match h.len() {
            3 => h.chars().flat_map(|c| [c, c]).collect(),
            _ => h.to_string(),
        };
        return (h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit())).then(|| format!("#{}", h.to_ascii_lowercase()));
    }
    let c = match name.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
        "black" => 0x000000,
        "red" => 0xe06c75,
        "green" => 0x8bd49c,
        "yellow" => 0xf0c674,
        "blue" => 0x6cb6ff,
        "magenta" => 0xd291e4,
        "cyan" => 0x5fd1c6,
        "gray" | "grey" => 0xa0aab2,
        "darkgray" | "darkgrey" => 0x5c6670,
        "lightred" => 0xff8f8f,
        "lightgreen" => 0xa8f0b8,
        "lightyellow" => 0xffe08a,
        "lightblue" => 0x9fd0ff,
        "lightmagenta" => 0xf0b0ff,
        "lightcyan" => 0x9ff0e8,
        "white" => 0xffffff,
        _ => return None,
    };
    Some(format!("#{c:06x}"))
}

/// The part names a part and its states layer in: every combination of the states, in their
/// order, fewer before more (`row`, `row.selected`, `row.blur`, `row.selected.blur`).
fn names(part: &str, states: &[&str]) -> Vec<String> {
    let mut sets: Vec<u32> = (0..1u32 << states.len()).collect();
    sets.sort_by_key(|s| s.count_ones());
    sets.into_iter()
        .map(|s| {
            let mut name = part.to_string();
            for (i, st) in states.iter().enumerate() {
                if s & (1 << i) != 0 {
                    name.push('.');
                    name.push_str(st);
                }
            }
            name
        })
        .collect()
}

/// The states that hold for `n`, in their order.
pub fn states(st: &State, n: &Node) -> Vec<&'static str> {
    let focused = n.id.is_some() && n.id == st.focus;
    let hover = n.id.is_some() && st.hover.as_ref().map(|h| &h.id) == n.id.as_ref();
    STATES
        .into_iter()
        .filter(|s| match *s {
            "hover" => hover,
            "focused" => focused,
            "blur" => n.focusable() && !focused,
            _ => false,
        })
        .collect()
}

/// The part a node starts from: its `type`, unless it's a container.
pub fn part_of(n: &Node) -> Option<&'static str> {
    Some(n.kind_name()).filter(|k| COMPONENTS.contains(k))
}

/// `n`'s style under the active theme.
pub fn resolve(st: &State, n: &Node) -> Resolved {
    let th = Theme::of(st);
    let states = states(st, n);
    let var = th.variant(n);
    let part = part_of(n);
    let mut ink = part.map_or_else(Ink::default, |p| th.layered(p, &states, Ink::default()));
    if let Some(s) = var.and_then(|v| v.style.as_ref()) {
        ink = th.style(s, ink);
    }
    if let Some(c) = n.tone.as_deref().and_then(|t| th.colour(t)) {
        ink.fg = Some(c);
    }
    if let Some(s) = &n.style {
        ink = th.style(s, ink);
    }
    let own_edge = n.edge.as_deref().or(var.and_then(|v| v.edge.as_deref()));
    let backdrop = n.backdrop.as_ref().or(var.and_then(|v| v.backdrop.as_ref()));
    let bordered = n.title.is_some() || n.border || own_edge.is_some() || backdrop.is_some() || var.is_some_and(|v| v.border);
    let fill = |c: &str| th.colour(c).unwrap_or_else(|| c.to_string());
    Resolved {
        part: part.map(str::to_string),
        states: states.iter().map(|s| s.to_string()).collect(),
        ink,
        edge: own_edge.or(th.token("edge")).filter(|e| *e != "rounded" && (bordered || n.focusable())).map(str::to_string),
        border: bordered.then(|| th.layered("border", &states, Ink::default()).fg).flatten(),
        title: n.title.as_ref().map(|_| {
            let t = th.layered("title", &states, Ink::default());
            match var.and_then(|v| v.title.as_deref()) {
                Some(spec) => th.ink(spec, t),
                None => t,
            }
        }),
        backdrop: backdrop.map(|b| Backdrop {
            fill: fill(&b.fill),
            fill2: b.fill2.as_deref().map(fill),
            border: b.border.as_deref().map(fill),
            glow: b.glow.as_deref().map(fill),
            ..b.clone()
        }),
    }
}

/// What's wrong with the active theme and the styles nodes ask for, in words: parts it can't
/// have, words in a spec that mean nothing, colours that aren't tokens, row variants a list
/// skips but doesn't have.
pub fn problems(st: &State, nodes: &[&Node]) -> Vec<String> {
    let th = Theme::of(st);
    let mut out = Vec::new();
    let name = st.theme.as_deref().unwrap_or_default();
    if let Some(active) = th.active {
        for (part, spec) in &active.parts {
            if !known_part(&th, part) {
                out.push(format!("theme {name}: no part is called {part:?}"));
            }
            for w in th.unknown(spec) {
                out.push(format!("theme {name}: part {part}: {w:?} isn't a token, a colour, a part or b, d, i, u, r, s"));
            }
        }
        for (v, var) in &active.variants {
            if let Some(spec) = &var.title {
                for w in th.unknown(spec) {
                    out.push(format!("theme {name}: variant {v}'s title: {w:?} isn't a token, a colour, a part or b, d, i, u, r, s"));
                }
            }
            for c in var.style.iter().flat_map(colours).chain(var.backdrop.iter().flat_map(backdrop_colours)) {
                if !th.knows(c) {
                    out.push(format!("theme {name}: variant {v}: {c:?} isn't a token or a colour"));
                }
            }
        }
    }
    for n in nodes {
        let who = n.id.clone().unwrap_or_else(|| n.kind_name().to_string());
        for c in n.style.iter().flat_map(colours).chain(n.backdrop.iter().flat_map(backdrop_colours)) {
            if !th.knows(c) {
                out.push(format!("{who}: {c:?} isn't a theme token or a colour"));
            }
        }
        if let Some(Rows { variants, skip, .. }) = n.rows() {
            for s in skip.iter().filter(|s| !variants.contains_key(*s)) {
                out.push(format!("{} {who}: it skips {s:?}, which isn't one of its row variants", n.kind_name()));
            }
        }
    }
    out
}

/// Whether a theme can have this part: a component's or an inner part, then states, or a toast's
/// tone (`toast.warn`). A name outside those families (`link`) is the theme's own, for markup to
/// ask for (`<link>…</>`).
fn known_part(th: &Theme, part: &str) -> bool {
    let mut words = part.split('.').collect::<Vec<_>>();
    while words.last().is_some_and(|w| STATES.contains(w)) {
        words.pop();
    }
    let base = words.join(".");
    let family = |w: &str| COMPONENTS.contains(&w) || INNER.iter().any(|p| p.split('.').next() == Some(w));
    COMPONENTS.contains(&base.as_str())
        || INNER.contains(&base.as_str())
        || base.strip_prefix("toast.").is_some_and(|t| th.knows(t))
        || !family(words.first().copied().unwrap_or_default())
}

fn colours(s: &Style) -> impl Iterator<Item = &str> {
    s.fg.as_deref().into_iter().chain(s.bg.as_deref())
}

fn backdrop_colours(b: &Backdrop) -> impl Iterator<Item = &str> {
    [Some(b.fill.as_str()), b.fill2.as_deref(), b.border.as_deref(), b.glow.as_deref()].into_iter().flatten()
}

/// Every shown node's style, in readback order.
pub fn shown(st: &State) -> Vec<Resolved> {
    let mut all = Vec::new();
    st.tree().walk_shown(&|n| state::tab_of(st, n), &|n| state::shown(st, n), &mut all);
    all.into_iter().map(|n| resolve(st, n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Hover, Msg, update};
    use serde_json::json;

    fn screen(theme: serde_json::Value, root: serde_json::Value) -> State {
        let mut st = State::default();
        let theme: ThemeSpec = serde_json::from_value(theme).unwrap();
        update(&mut st, Msg::Theme { add: [("t".to_string(), theme)].into(), choose: None });
        update(&mut st, Msg::Push { ui: serde_json::from_value(json!({"root": root})).unwrap() });
        st
    }

    fn of(st: &State, id: &str) -> Resolved {
        resolve(st, st.tree().find(id).unwrap())
    }

    #[test]
    fn a_part_takes_the_parts_of_its_states_over_it() {
        let mut st = screen(
            json!({"tokens": {"accent": "#ff8800"}, "parts": {"button": "fg+on-panel", "button.hover": "on-hover", "button.focused": "b", "button.hover.focused": "u"}}),
            json!({"type": "row", "children": [{"type": "button", "id": "a", "label": "A"}, {"type": "button", "id": "b", "label": "B"}]}),
        );
        let a = of(&st, "a");
        assert_eq!((a.part.as_deref(), a.states.clone()), (Some("button"), vec!["focused".to_string()]), "the first button has the focus");
        assert_eq!(a.ink.spec(), "#e3eaee+on-#161d22+b", "button, then button.focused: its own bold over the button's colours");
        assert_eq!(of(&st, "b").states, vec!["blur"], "a button without the focus is blurred");
        update(&mut st, Msg::Hover { hover: Some(Hover { id: "a".into(), row: None }) });
        let a = of(&st, "a");
        assert_eq!(a.states, vec!["hover", "focused"]);
        assert_eq!(a.ink.spec(), "#e3eaee+on-#1a2329+b+u", "hover, then focused, then both");
    }

    #[test]
    fn a_variant_then_a_tone_then_the_nodes_own_style() {
        let st = screen(
            json!({"tokens": {"warn": "#ffbd4f", "edge": "square"}, "parts": {"title": "dim"},
                   "variants": {"card": {"style": {"bg": "panel", "fg": "accent"}, "title": "b", "backdrop": {"fill": "panel", "fill2": "bg"}}}}),
            json!({"type": "col", "children": [
                {"type": "text", "id": "t", "title": "T", "variant": "card", "tone": "warn", "text": "hi"},
                {"type": "text", "id": "u", "variant": "card", "tone": "warn", "style": {"fg": "red", "italic": true}, "text": "hi"},
                {"type": "col", "id": "c", "children": []}
            ]}),
        );
        let t = of(&st, "t");
        assert_eq!(t.ink.spec(), "#ffbd4f+on-#161d22", "the variant's background, the tone's colour");
        assert_eq!(t.title.as_ref().unwrap().spec(), "#93a3ad+b", "the theme's title, then the variant's");
        assert_eq!(t.border.as_deref(), Some("#2a353d"), "a title draws a border");
        assert_eq!(t.edge.as_deref(), Some("square"), "the theme's edge");
        assert_eq!(t.backdrop.as_ref().map(|b| (b.fill.as_str(), b.fill2.as_deref())), Some(("#161d22", Some("#0f1418"))));
        assert_eq!(of(&st, "u").ink.spec(), "#e06c75+on-#161d22+i", "the node's own style last");
        let c = of(&st, "c");
        assert_eq!((c.part.as_deref(), c.border.as_deref(), c.line()), (None, None, String::new()), "a container has no part, and no border for the edge");
    }

    #[test]
    fn a_selected_row_that_lost_the_focus_layers_blur_over_selected() {
        let th = ThemeSpec { parts: [("row.selected".into(), "fg+on-ring".into()), ("row.selected.blur".into(), "d".into())].into(), ..ThemeSpec::default() };
        let ui = BTreeMap::new();
        let th = Theme { active: Some(&th), ui: &ui };
        assert_eq!(th.layered("row", &["selected", "blur"], Ink::default()).spec(), "#e3eaee+on-#1d3b3a+d");
        assert_eq!(names("row", &["selected", "blur"]), vec!["row", "row.selected", "row.blur", "row.selected.blur"]);
    }

    #[test]
    fn wrong_names_in_a_theme_are_problems() {
        let st = screen(
            json!({"tokens": {"warn": "#ffbd4f"}, "parts": {"button.focsed": "fg", "button.focused": "acent+b", "md.heding": "b", "row.selected.blur": "d", "toast.warn": "warn", "link": "u"},
                   "variants": {"card": {"title": "brite", "style": {"bg": "pannel"}}}}),
            json!({"type": "col", "children": [
                {"type": "list", "id": "l", "items": [], "item": "{name}", "row_variant": "kind", "variants": {"head": "<b>{name}</>"}, "skip": ["header"]},
                {"type": "text", "id": "x", "style": {"fg": "violet"}, "text": "hi"}
            ]}),
        );
        let ours: Vec<&String> = st.problems.iter().filter(|p| !p.contains("should be a list")).collect();
        assert_eq!(ours, vec![
            "theme t: no part is called \"button.focsed\"",
            "theme t: part button.focused: \"acent\" isn't a token, a colour, a part or b, d, i, u, r, s",
            "theme t: no part is called \"md.heding\"",
            "theme t: variant card's title: \"brite\" isn't a token, a colour, a part or b, d, i, u, r, s",
            "theme t: variant card: \"pannel\" isn't a token or a colour",
            "list l: it skips \"header\", which isn't one of its row variants",
            "x: \"violet\" isn't a theme token or a colour",
        ]);
    }
}
