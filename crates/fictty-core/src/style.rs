//! Each shown node's style, worked out from the active theme: its part (the component's name),
//! with the parts for the states that hold layered over it (`button`, then `button.focused`),
//! then its variant, then its `tone`, then its own `style`, each a whole style laid over the
//! last, with tokens filled in as colours and scale names (`s2`, `r2`, `e1`, `display`) as
//! what they name.
//! Whatever draws reads this one value, the readback prints it, and tests assert on it without a
//! window. There's no cascade beyond that: nothing else applies.

use crate::model::{Backdrop, Fill, Len, Lens, Node, Part, Ring, Rows, Shadow, Style, ThemeSpec, TypeRole, Variant, is_colour};
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

/// The shapes a style can ask a component to take.
const SHAPES: [&str; 11] = ["underline", "pill", "segment", "plain", "thin", "overlay", "none", "rounded", "square", "double", "thick"];

const CURSORS: [&str; 4] = ["default", "pointer", "text", "grab"];

const LINES: [&str; 3] = ["solid", "dashed", "double"];

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

/// A gradient background, its colours filled in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub from: String,
    pub to: String,
    pub angle: f32,
}

/// A type role, filled in: its name, and the font a `fonts` name stands for.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    #[serde(flatten)]
    pub role: TypeRole,
}

/// A border as drawn: width in pixels, colour, `solid`, `dashed` or `double`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub color: String,
    pub style: String,
}

/// A focus ring as drawn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Halo {
    pub width: f32,
    pub offset: f32,
    pub color: String,
}

/// A node's style, resolved: every name filled in, nothing left to look up.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Resolved {
    /// The theme part it starts from: its `type`, or none for a container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// The states that hold, whose parts layer over it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
    /// Its text, and its background (a gradient's first colour).
    #[serde(default, skip_serializing_if = "is_plain")]
    pub ink: Ink,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Gradient>,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Space inside, `[x, y]`: cells (numbers) or pixels (`"8px"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pad: Option<[Len; 2]>,
    /// Corner radius in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    /// Its border, when it draws one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shadow: Vec<Shadow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring: Option<Halo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// How its border and corners are drawn, when not `rounded`: its own, its variant's or the
    /// theme's `edge`. Only for a node that draws a border, or a face (a button, a field, tabs,
    /// rows).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
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

fn len_text(l: &Len) -> String {
    match l {
        Len::Cells(n) => format!("{n}"),
        Len::Spec(s) => s.clone(),
    }
}

impl Resolved {
    /// One line: `button.focused · #0f1418+on-#5fd1c6+b · radius 6 · border 1 #5fd1c6 · edge double`.
    pub fn line(&self) -> String {
        let mut out = vec![];
        if let Some(p) = &self.part {
            out.push(std::iter::once(p.as_str()).chain(self.states.iter().map(String::as_str)).collect::<Vec<_>>().join("."));
        }
        if !is_plain(&self.ink) {
            out.push(self.ink.spec());
        }
        out.extend(self.gradient.as_ref().map(|g| format!("gradient {}→{} {}°", g.from, g.to, g.angle)));
        out.extend(self.role.as_ref().map(|r| {
            let mut t = format!("type {}", r.name);
            t.extend(r.role.font.as_ref().map(|f| format!(" {f}")));
            t.extend(r.role.size.map(|s| format!(" {s}px")));
            t.extend(r.role.weight.map(|w| format!(" {w}")));
            t
        }));
        out.extend(self.pad.as_ref().map(|[x, y]| format!("pad {} {}", len_text(x), len_text(y))));
        out.extend(self.radius.map(|r| format!("radius {r}")));
        out.extend(self.border.as_ref().map(|b| format!("border {} {}{}", b.width, b.color, if b.style == "solid" { String::new() } else { format!(" {}", b.style) })));
        if !self.shadow.is_empty() {
            out.push(format!("shadow ×{}", self.shadow.len()));
        }
        out.extend(self.ring.as_ref().map(|r| format!("ring {} {}", r.width, r.color)));
        out.extend(self.shape.as_ref().map(|s| format!("shape {s}")));
        out.extend(self.opacity.map(|o| format!("opacity {o}")));
        out.extend(self.cursor.as_ref().map(|c| format!("cursor {c}")));
        out.extend(self.title.as_ref().map(|t| format!("title {}", t.spec())));
        out.extend(self.edge.as_ref().map(|e| format!("edge {e}")));
        out.extend(self.backdrop.as_ref().map(|b| format!("backdrop {}", b.fill2.as_ref().map_or(b.fill.clone(), |f2| format!("{}→{f2}", b.fill)))));
        out.join(" · ")
    }
}

/// The active theme as the core reads it: its tokens over the UI's own `theme` map over the
/// defaults, its parts over the default parts, and its scales.
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

    /// A part: the theme's, else the default.
    pub fn part(&self, name: &str) -> Option<Part> {
        self.active
            .and_then(|t| t.parts.get(name))
            .cloned()
            .or_else(|| DEFAULT_PARTS.iter().find(|(k, _)| *k == name).map(|(_, v)| Part::Ink(v.to_string())))
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
                        (None, Some(part)) if depth < 4 => ink = self.ink_at(&part.spec(), ink, depth + 1),
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

    /// A part as a whole style: an ink becomes its colours and modifiers.
    fn whole(&self, part: &Part) -> Style {
        match part {
            Part::Whole(s) => (**s).clone(),
            Part::Ink(spec) => {
                let i = self.ink(spec, Ink::default());
                Style {
                    fg: i.fg,
                    bg: i.bg.map(Fill::Colour),
                    bold: i.bold,
                    dim: i.dim,
                    italic: i.italic,
                    underline: i.underline,
                    reverse: i.reverse,
                    strike: i.strike,
                    ..Style::default()
                }
            }
        }
    }

    /// `part` with the parts for `states` layered over it.
    pub fn layered(&self, part: &str, states: &[&str]) -> Style {
        names(part, states).iter().filter_map(|n| self.part(n)).fold(Style::default(), |s, p| self.whole(&p).over(&s))
    }

    /// A style's colours and modifiers as ink, colours filled in.
    pub fn ink_of(&self, s: &Style) -> Ink {
        Ink {
            fg: s.fg.as_deref().and_then(|c| self.colour(c)),
            bg: s.bg.as_ref().and_then(|b| self.colour(b.colour())),
            bold: s.bold,
            dim: s.dim,
            italic: s.italic,
            underline: s.underline,
            reverse: s.reverse,
            strike: s.strike,
        }
    }

    /// A length with a `space` name filled in: cells, or `"8px"`.
    fn len(&self, l: &Len) -> Option<Len> {
        match l {
            Len::Cells(_) => Some(l.clone()),
            Len::Spec(s) if s.trim().strip_suffix("px").is_some_and(|n| n.trim().parse::<f32>().is_ok()) => Some(l.clone()),
            Len::Spec(s) if s.trim().parse::<f32>().is_ok() => Some(Len::Cells(s.trim().parse().ok()?)),
            Len::Spec(s) => match self.active?.space.get(s.trim())? {
                Len::Spec(n) if !n.trim().ends_with("px") && n.trim().parse::<f32>().is_err() => None,
                v => self.len(v),
            },
        }
    }

    /// A radius in pixels: a number, `"6px"`, or a `radius` name.
    fn radius(&self, l: &Len) -> Option<f32> {
        match l {
            Len::Cells(n) => Some(*n),
            Len::Spec(s) => s.trim().trim_end_matches("px").trim().parse().ok().or_else(|| self.active?.radius.get(s.trim()).copied()),
        }
    }

    /// The shadows a `depth` name stands for, colours filled in; `none` is none.
    fn shadows(&self, name: &str) -> Option<Vec<Shadow>> {
        if name == "none" {
            return Some(vec![]);
        }
        let list = self.active?.depth.get(name)?;
        Some(list.iter().map(|s| Shadow { color: self.colour(&s.color).unwrap_or_else(|| s.color.clone()), ..s.clone() }).collect())
    }

    /// A type role, its font filled in from `fonts`.
    fn role(&self, name: &str) -> Option<Role> {
        let r = self.active?.types.get(name)?;
        let font = r.font.as_ref().map(|f| self.active.and_then(|t| t.fonts.get(f)).unwrap_or(f).clone());
        Some(Role { name: name.to_string(), role: TypeRole { font, ..r.clone() } })
    }

    fn variant(&self, n: &Node) -> Option<&'a Variant> {
        self.active?.variants.get(n.variant.as_deref()?)
    }

    /// What's wrong with a style here, in words.
    fn style_problems(&self, s: &Style) -> Vec<String> {
        let mut out = Vec::new();
        let bg = match &s.bg {
            Some(Fill::Colour(c)) => vec![c.as_str()],
            Some(Fill::Gradient { from, to, .. }) => vec![from.as_str(), to.as_str()],
            None => vec![],
        };
        let colours = s.fg.iter().map(String::as_str).chain(bg).chain(s.border.as_ref().and_then(|b| b.color.as_deref())).chain(s.ring.as_ref().and_then(|r| r.color.as_deref()));
        for c in colours.filter(|c| !self.knows(c)) {
            out.push(format!("{c:?} isn't a token or a colour"));
        }
        if let Some(r) = s.role.as_deref().filter(|r| self.role(r).is_none()) {
            out.push(format!("type {r:?} isn't one of the theme's type roles"));
        }
        let pads = match &s.pad {
            Some(Lens::One(l)) => vec![l],
            Some(Lens::XY([x, y])) => vec![x, y],
            None => vec![],
        };
        for l in pads.into_iter().filter(|l| self.len(l).is_none()) {
            out.push(format!("pad {:?} isn't a length or one of the theme's space names", len_text(l)));
        }
        if let Some(r) = s.radius.as_ref().filter(|r| self.radius(r).is_none()) {
            out.push(format!("radius {:?} isn't pixels or one of the theme's radius names", len_text(r)));
        }
        if let Some(d) = s.shadow.as_deref().filter(|d| self.shadows(d).is_none()) {
            out.push(format!("shadow {d:?} isn't one of the theme's depth names (or none)"));
        }
        if let Some(b) = s.border.as_ref().and_then(|b| b.style.as_deref()).filter(|b| !LINES.contains(b)) {
            out.push(format!("border style {b:?} isn't solid, dashed or double"));
        }
        if let Some(sh) = s.shape.as_deref().filter(|sh| !SHAPES.contains(sh)) {
            out.push(format!("shape {sh:?} isn't one of {}", SHAPES.join(", ")));
        }
        if let Some(c) = s.cursor.as_deref().filter(|c| !CURSORS.contains(c)) {
            out.push(format!("cursor {c:?} isn't one of {}", CURSORS.join(", ")));
        }
        if let Some(o) = s.opacity.filter(|o| !(0.0..=1.0).contains(o)) {
            out.push(format!("opacity {o} isn't between 0 and 1"));
        }
        for k in s.other.keys() {
            out.push(format!("a style has no {k:?}"));
        }
        out
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
    let mut s = part.map_or_else(Style::default, |p| th.layered(p, &states));
    if let Some(v) = var.and_then(|v| v.style.as_ref()) {
        s = v.over(&s);
    }
    if let Some(t) = n.tone.as_deref().filter(|t| th.knows(t)) {
        s.fg = Some(t.to_string());
    }
    if let Some(own) = &n.style {
        s = own.over(&s);
    }
    let own_edge = n.edge.as_deref().or(var.and_then(|v| v.edge.as_deref()));
    let backdrop = n.backdrop.as_ref().or(var.and_then(|v| v.backdrop.as_ref()));
    let bordered = n.title.is_some() || n.border || own_edge.is_some() || backdrop.is_some() || var.is_some_and(|v| v.border);
    // The border's colour comes from the `border` part (its border colour, or its ink), under
    // the node's own style's border.
    let edge_part = th.layered("border", &states);
    let border = (bordered || s.border.is_some()).then(|| {
        let line = s.border.clone().unwrap_or_default();
        let color = line.color.or(edge_part.border.and_then(|b| b.color)).or(edge_part.fg).unwrap_or_else(|| "border".into());
        Border { width: line.width.unwrap_or(1.), color: th.colour(&color).unwrap_or(color), style: line.style.unwrap_or_else(|| "solid".into()) }
    });
    let fill = |c: &str| th.colour(c).unwrap_or_else(|| c.to_string());
    Resolved {
        part: part.map(str::to_string),
        states: states.iter().map(|s| s.to_string()).collect(),
        ink: th.ink_of(&s),
        gradient: match &s.bg {
            Some(Fill::Gradient { from, to, angle }) => Some(Gradient { from: fill(from), to: fill(to), angle: *angle }),
            _ => None,
        },
        role: s.role.as_deref().and_then(|r| th.role(r)),
        pad: match &s.pad {
            Some(Lens::One(l)) => th.len(l).map(|l| [l.clone(), l]),
            Some(Lens::XY([x, y])) => th.len(x).zip(th.len(y)).map(|(x, y)| [x, y]),
            None => None,
        },
        radius: s.radius.as_ref().and_then(|r| th.radius(r)),
        border,
        shadow: s.shadow.as_deref().and_then(|d| th.shadows(d)).unwrap_or_default(),
        ring: s.ring.as_ref().map(|r: &Ring| Halo {
            width: r.width.unwrap_or(2.),
            offset: r.offset.unwrap_or(0.),
            color: fill(r.color.as_deref().unwrap_or("accent")),
        }),
        shape: s.shape.clone(),
        opacity: s.opacity,
        cursor: s.cursor.clone(),
        edge: own_edge.or(th.token("edge")).filter(|e| *e != "rounded" && (bordered || n.focusable())).map(str::to_string),
        title: n.title.as_ref().map(|_| {
            let t = th.ink_of(&th.layered("title", &states));
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
/// have, words in a spec that mean nothing, colours that aren't tokens, scale names it doesn't
/// have, fields no style has, row variants a list skips but doesn't have.
pub fn problems(st: &State, nodes: &[&Node]) -> Vec<String> {
    let th = Theme::of(st);
    let mut out = Vec::new();
    let name = st.theme.as_deref().unwrap_or_default();
    let ink_words = "isn't a token, a colour, a part or b, d, i, u, r, s";
    if let Some(active) = th.active {
        for (part, p) in &active.parts {
            if !known_part(&th, part) {
                out.push(format!("theme {name}: no part is called {part:?}"));
            }
            match p {
                Part::Ink(spec) => out.extend(th.unknown(spec).into_iter().map(|w| format!("theme {name}: part {part}: {w:?} {ink_words}"))),
                Part::Whole(s) => out.extend(th.style_problems(s).into_iter().map(|p| format!("theme {name}: part {part}: {p}"))),
            }
        }
        for (v, var) in &active.variants {
            if let Some(spec) = &var.title {
                out.extend(th.unknown(spec).into_iter().map(|w| format!("theme {name}: variant {v}'s title: {w:?} {ink_words}")));
            }
            out.extend(var.style.iter().flat_map(|s| th.style_problems(s)).map(|p| format!("theme {name}: variant {v}: {p}")));
            for c in var.backdrop.iter().flat_map(backdrop_colours).filter(|c| !th.knows(c)) {
                out.push(format!("theme {name}: variant {v}: {c:?} isn't a token or a colour"));
            }
        }
        for (k, l) in &active.space {
            if th.len(l).is_none() {
                out.push(format!("theme {name}: space {k}: {:?} isn't cells or pixels", len_text(l)));
            }
        }
        for (k, list) in &active.depth {
            for c in list.iter().map(|s| s.color.as_str()).filter(|c| !th.knows(c)) {
                out.push(format!("theme {name}: depth {k}: {c:?} isn't a token or a colour"));
            }
        }
    }
    for n in nodes {
        let who = n.id.clone().unwrap_or_else(|| n.kind_name().to_string());
        out.extend(n.style.iter().flat_map(|s| th.style_problems(s)).map(|p| format!("{who}: {p}")));
        for c in n.backdrop.iter().flat_map(backdrop_colours).filter(|c| !th.knows(c)) {
            out.push(format!("{who}: {c:?} isn't a token or a colour"));
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
        assert_eq!(t.border.as_ref().map(|b| (b.width, b.color.as_str())), Some((1., "#2a353d")), "a title draws a border");
        assert_eq!(t.edge.as_deref(), Some("square"), "the theme's edge");
        assert_eq!(t.backdrop.as_ref().map(|b| (b.fill.as_str(), b.fill2.as_deref())), Some(("#161d22", Some("#0f1418"))));
        assert_eq!(of(&st, "u").ink.spec(), "#e06c75+on-#161d22+i", "the node's own style last");
        let c = of(&st, "c");
        assert_eq!((c.part.as_deref(), c.border.is_none(), c.line()), (None, true, String::new()), "a container has no part, and no border for the edge");
    }

    #[test]
    fn a_selected_row_that_lost_the_focus_layers_blur_over_selected() {
        let th = ThemeSpec { parts: [("row.selected".into(), Part::Ink("fg+on-ring".into())), ("row.selected.blur".into(), Part::Ink("d".into()))].into(), ..ThemeSpec::default() };
        let ui = BTreeMap::new();
        let th = Theme { active: Some(&th), ui: &ui };
        assert_eq!(th.ink_of(&th.layered("row", &["selected", "blur"])).spec(), "#e3eaee+on-#1d3b3a+d");
        assert_eq!(names("row", &["selected", "blur"]), vec!["row", "row.selected", "row.blur", "row.selected.blur"]);
    }

    #[test]
    fn a_part_can_be_a_whole_style_named_from_the_theme_scales() {
        let st = screen(
            json!({
                "tokens": {"glow": "#39d6e6"},
                "space": {"s2": "8px", "s3": "14px", "s1": 1},
                "radius": {"r2": 6},
                "depth": {"e1": [{"y": 1, "blur": 2, "color": "black", "alpha": 0.3}], "e2": [{"y": 4, "blur": 12, "color": "black", "alpha": 0.3}, {"blur": 8, "color": "glow", "alpha": 0.5}]},
                "type": {"body": {"size": 14}, "label": {"font": "ui", "size": 13, "weight": 600, "tracking": 0.2}},
                "fonts": {"ui": "Inter"},
                "parts": {
                    "button": {"fg": "fg", "bg": "panel", "type": "label", "pad": ["s3", "s2"], "radius": "r2", "border": {"color": "border"}, "shadow": "e1", "cursor": "pointer"},
                    "button.focused": {"bg": {"from": "accent", "to": "glow", "angle": 160}, "fg": "bg", "shadow": "e2", "ring": {"width": 2, "offset": 2}},
                    "tabs": {"shape": "pill", "pad": "s1"}
                }
            }),
            json!({"type": "col", "children": [
                {"type": "button", "id": "go", "label": "Go"},
                {"type": "tabs", "id": "t", "tabs": ["A"], "children": [{"type": "text", "text": "a"}]}
            ]}),
        );
        let go = of(&st, "go");
        assert_eq!(go.ink.spec(), "#0f1418+on-#5fd1c6", "the focused part's colours; a gradient's first is the background");
        assert_eq!(go.gradient, Some(Gradient { from: "#5fd1c6".into(), to: "#39d6e6".into(), angle: 160. }));
        let role = go.role.clone().unwrap();
        assert_eq!((role.name.as_str(), role.role.font.as_deref(), role.role.size, role.role.weight), ("label", Some("Inter"), Some(13.), Some(600)));
        assert_eq!(go.pad, Some([Len::Spec("14px".into()), Len::Spec("8px".into())]));
        assert_eq!(go.radius, Some(6.));
        assert_eq!(go.border, Some(Border { width: 1., color: "#2a353d".into(), style: "solid".into() }), "a border from the part, though the node has none of its own");
        assert_eq!(go.shadow.iter().map(|s| s.color.as_str()).collect::<Vec<_>>(), vec!["#000000", "#39d6e6"], "e2, focused, over e1");
        assert_eq!(go.ring, Some(Halo { width: 2., offset: 2., color: "#5fd1c6".into() }));
        assert_eq!(go.cursor.as_deref(), Some("pointer"));
        assert_eq!(
            go.line(),
            "button.focused · #0f1418+on-#5fd1c6 · gradient #5fd1c6→#39d6e6 160° · type label Inter 13px 600 · pad 14px 8px · radius 6 · border 1 #2a353d · shadow ×2 · ring 2 #5fd1c6 · cursor pointer"
        );
        let t = of(&st, "t");
        assert_eq!((t.shape.as_deref(), t.pad.clone()), (Some("pill"), Some([Len::Cells(1.), Len::Cells(1.)])), "one length pads every side");
        assert!(st.problems.is_empty(), "{:?}", st.problems);
    }

    #[test]
    fn wrong_names_in_a_theme_are_problems() {
        let st = screen(
            json!({"tokens": {"warn": "#ffbd4f"},
                   "space": {"s1": "8px", "big": "huge"},
                   "parts": {"button.focsed": "fg", "button.focused": "acent+b", "md.heding": "b", "row.selected.blur": "d", "toast.warn": "warn", "link": "u",
                             "tabs": {"shape": "pills", "pad": "s9", "radius": "r9", "shadow": "e9", "type": "huge", "colour": "red", "border": {"style": "dotted", "color": "bordr"}}},
                   "variants": {"card": {"title": "brite", "style": {"bg": "pannel"}}}}),
            json!({"type": "col", "children": [
                {"type": "list", "id": "l", "items": [], "item": "{name}", "row_variant": "kind", "variants": {"head": "<b>{name}</>"}, "skip": ["header"]},
                {"type": "text", "id": "x", "style": {"fg": "violet", "cursor": "hand"}, "text": "hi"}
            ]}),
        );
        let ours: Vec<&String> = st.problems.iter().filter(|p| !p.contains("should be a list")).collect();
        assert_eq!(ours, vec![
            "theme t: no part is called \"button.focsed\"",
            "theme t: part button.focused: \"acent\" isn't a token, a colour, a part or b, d, i, u, r, s",
            "theme t: no part is called \"md.heding\"",
            "theme t: part tabs: \"bordr\" isn't a token or a colour",
            "theme t: part tabs: type \"huge\" isn't one of the theme's type roles",
            "theme t: part tabs: pad \"s9\" isn't a length or one of the theme's space names",
            "theme t: part tabs: radius \"r9\" isn't pixels or one of the theme's radius names",
            "theme t: part tabs: shadow \"e9\" isn't one of the theme's depth names (or none)",
            "theme t: part tabs: border style \"dotted\" isn't solid, dashed or double",
            "theme t: part tabs: shape \"pills\" isn't one of underline, pill, segment, plain, thin, overlay, none, rounded, square, double, thick",
            "theme t: part tabs: a style has no \"colour\"",
            "theme t: variant card's title: \"brite\" isn't a token, a colour, a part or b, d, i, u, r, s",
            "theme t: variant card: \"pannel\" isn't a token or a colour",
            "theme t: space big: \"huge\" isn't cells or pixels",
            "list l: it skips \"header\", which isn't one of its row variants",
            "x: \"violet\" isn't a token or a colour",
            "x: cursor \"hand\" isn't one of default, pointer, text, grab",
        ]);
    }
}
