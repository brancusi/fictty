//! The active theme as the window's colours and text styles: tokens are colours, and parts and
//! `<spec>` runs (`accent+b`, `fg+on-ring`) are inks. Read from the state each frame, as the
//! terminal's view reads it, so switching themes is a message like any other.

use crate::model::{Style, ThemeSpec};
use crate::state::State;
use gpui::{FontStyle, FontWeight, HighlightStyle, Hsla, StrikethroughStyle, UnderlineStyle, px, rgb};

/// How a run of text looks: colours and the `b d i u r s` modifiers of a spec.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ink {
    pub fg: Option<Hsla>,
    pub bg: Option<Hsla>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub strike: bool,
}

/// The theme the window draws a frame with.
pub struct Look<'a> {
    st: &'a State,
    active: Option<&'a ThemeSpec>,
    pub bg: Hsla,
    pub fg: Hsla,
    pub dim: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub panel: Hsla,
    /// A selected row's or text's background.
    pub ring: Hsla,
    pub hover: Hsla,
    /// What a dialog dims the screen behind it with: the background, part see-through.
    pub scrim: Hsla,
    pub pos: Hsla,
    pub neg: Hsla,
}

/// A colour by name: `#rrggbb`, or one of the terminal's named colours.
pub fn parse(name: &str) -> Option<Hsla> {
    if let Some(hex) = name.strip_prefix('#') {
        let hex = match hex.len() {
            3 => hex.chars().flat_map(|c| [c, c]).collect(),
            _ => hex.to_string(),
        };
        return u32::from_str_radix(&hex, 16).ok().filter(|_| hex.len() == 6).map(|c| rgb(c).into());
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
    Some(rgb(c).into())
}

impl<'a> Look<'a> {
    pub fn new(st: &'a State) -> Look<'a> {
        let active = st.theme.as_ref().and_then(|n| st.themes.get(n));
        let token = |k: &str| active.and_then(|t| t.tokens.get(k)).or_else(|| st.ui.theme.get(k)).and_then(|c| parse(c));
        let get = |k: &str, d: u32| token(k).unwrap_or_else(|| rgb(d).into());
        let bg = get("bg", 0x0f1418);
        Look {
            st,
            active,
            bg,
            scrim: token("scrim").unwrap_or(Hsla { a: 0.6, ..bg }),
            fg: get("fg", 0xe3eaee),
            dim: get("dim", 0x93a3ad),
            border: get("border", 0x2a353d),
            accent: get("accent", 0x5fd1c6),
            panel: get("panel", 0x161d22),
            ring: get("ring", 0x1d3b3a),
            hover: get("hover", 0x1a2329),
            pos: get("pos", 0x8bd49c),
            neg: get("neg", 0xf07178),
        }
    }

    /// A theme token (`accent`, `violet`) or a colour name.
    pub fn color(&self, name: &str) -> Option<Hsla> {
        let token = self.active.and_then(|t| t.tokens.get(name)).or_else(|| self.st.ui.theme.get(name));
        match name {
            "fg" => Some(self.fg),
            "bg" => Some(self.bg),
            "dim" | "muted" => Some(self.dim),
            "accent" => Some(self.accent),
            "border" => Some(self.border),
            "panel" => Some(self.panel),
            "ring" => Some(self.ring),
            "hover" => Some(self.hover),
            "pos" => Some(self.pos),
            "neg" => Some(self.neg),
            _ => parse(token.map_or(name, String::as_str)),
        }
    }

    /// A style spec over `base`: `accent+b`, `muted`, `fg+on-ring`, `#ff8800+u`. A name that's no
    /// colour but one of the theme's parts takes that part's ink (`<link>`).
    pub fn spec(&self, spec: &str, base: Ink) -> Ink {
        self.spec_at(spec, base, 0)
    }

    fn spec_at(&self, spec: &str, base: Ink, depth: u8) -> Ink {
        let mut ink = base;
        for t in spec.split('+') {
            match t {
                "b" => ink.bold = true,
                "d" => ink.dim = true,
                "i" => ink.italic = true,
                "u" => ink.underline = true,
                "r" => ink.reverse = true,
                "s" => ink.strike = true,
                t => match t.strip_prefix("on-") {
                    Some(c) => ink.bg = self.color(c).or(ink.bg),
                    None => match (self.color(t), self.active.and_then(|a| a.parts.get(t))) {
                        (Some(c), _) => ink.fg = Some(c),
                        (None, Some(part)) if depth < 4 => ink = self.spec_at(part, ink, depth + 1),
                        _ => {}
                    },
                },
            }
        }
        ink
    }

    /// A part of a component, inked by the active theme (`table.header`, `row.selected`), or
    /// `fallback` when the theme doesn't say.
    pub fn part(&self, part: &str, fallback: Ink) -> Ink {
        match self.active.and_then(|t| t.parts.get(part)) {
            Some(spec) => self.spec(spec, Ink::default()),
            None => fallback,
        }
    }

    /// A node's own `style`, as ink.
    pub fn style(&self, s: Option<&Style>) -> Ink {
        let Some(s) = s else { return Ink::default() };
        Ink {
            fg: s.fg.as_deref().and_then(|c| self.color(c)),
            bg: s.bg.as_deref().and_then(|c| self.color(c)),
            bold: s.bold,
            dim: s.dim,
            italic: s.italic,
            ..Ink::default()
        }
    }

    /// Ink as GPUI's text highlight. Reversed ink swaps its colours, defaulting to the theme's.
    pub fn highlight(&self, ink: Ink) -> HighlightStyle {
        let (mut fg, mut bg) = (ink.fg, ink.bg);
        if ink.reverse {
            (fg, bg) = (Some(bg.unwrap_or(self.bg)), Some(fg.unwrap_or(self.fg)));
        }
        HighlightStyle {
            color: fg,
            background_color: bg,
            font_weight: ink.bold.then_some(FontWeight::BOLD),
            font_style: ink.italic.then_some(FontStyle::Italic),
            underline: ink.underline.then(|| UnderlineStyle { thickness: px(1.), color: fg, wavy: false }),
            strikethrough: ink.strike.then(|| StrikethroughStyle { thickness: px(1.), color: fg }),
            fade_out: ink.dim.then_some(0.4),
        }
    }
}
