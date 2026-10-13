//! The active theme as the window's colours and text styles: tokens are colours, and parts and
//! `<spec>` runs (`accent+b`, `fg+on-ring`) are inks. Read from the state each frame, as the
//! terminal's view reads it, so switching themes is a message like any other.

use crate::model::Style;
use crate::state::State;
use crate::style::{self, Theme};
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
    th: Theme<'a>,
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
    /// The theme's `edge` token: how borders that don't name their own are drawn. Anything but
    /// `rounded` (the default) squares every corner: panels, fields, buttons, tabs, rows.
    pub edge: Option<&'a str>,
    pub pos: Hsla,
    pub neg: Hsla,
}

/// A colour by name: `#rrggbb`, `#rgb`, or one of the named colours.
pub fn parse(name: &str) -> Option<Hsla> {
    let hex = style::hex(name)?;
    u32::from_str_radix(&hex[1..], 16).ok().map(|c| rgb(c).into())
}

impl<'a> Look<'a> {
    pub fn new(st: &'a State) -> Look<'a> {
        let th = Theme::of(st);
        let get = |k: &str| th.colour(k).as_deref().and_then(parse).unwrap_or_default();
        let bg = get("bg");
        Look {
            th,
            bg,
            scrim: th.colour("scrim").as_deref().and_then(parse).unwrap_or(Hsla { a: 0.6, ..bg }),
            edge: th.token("edge"),
            fg: get("fg"),
            dim: get("dim"),
            border: get("border"),
            accent: get("accent"),
            panel: get("panel"),
            ring: get("ring"),
            hover: get("hover"),
            pos: get("pos"),
            neg: get("neg"),
        }
    }

    /// Whether corners are square: the theme's edge isn't `rounded`.
    pub fn square(&self) -> bool {
        self.edge.is_some_and(|e| e != "rounded")
    }

    /// `el`'s corners: `r` pixels, or square under a square-edged theme.
    pub fn round<E: gpui::Styled>(&self, el: E, r: f32) -> E {
        if self.square() { el } else { el.rounded(gpui::px(r)) }
    }

    /// A theme token (`accent`, `violet`) or a colour name.
    pub fn color(&self, name: &str) -> Option<Hsla> {
        parse(&self.th.colour(name)?)
    }

    /// A style spec over `base`: `accent+b`, `muted`, `fg+on-ring`, `#ff8800+u`. A name that's no
    /// colour but one of the theme's parts takes that part's ink (`<link>`).
    pub fn spec(&self, spec: &str, base: Ink) -> Ink {
        let i = self.th.ink(spec, style::Ink::default());
        Ink {
            fg: i.fg.as_deref().and_then(parse).or(base.fg),
            bg: i.bg.as_deref().and_then(parse).or(base.bg),
            bold: base.bold || i.bold,
            dim: base.dim || i.dim,
            italic: base.italic || i.italic,
            underline: base.underline || i.underline,
            reverse: base.reverse || i.reverse,
            strike: base.strike || i.strike,
        }
    }

    /// A part of a component, inked by the active theme (`table.header`, `row.selected`), or
    /// `fallback` when the theme doesn't say.
    pub fn part(&self, part: &str, fallback: Ink) -> Ink {
        match self.th.active.and_then(|t| t.parts.get(part)) {
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
