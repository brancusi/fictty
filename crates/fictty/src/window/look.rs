//! The active theme as the window's: a resolved style (`style::Resolved`, worked out in the core)
//! drawn on a GPUI element by `boxed`, and `<spec>` runs in text (`accent+b`, `fg+on-ring`) as
//! inks. Read from the state each frame, so switching themes is a message like any other.

use crate::model::Len;
use crate::state::State;
use crate::style::{self, Resolved, Role, Theme};
use std::collections::HashMap;
use gpui::{BoxShadow, CursorStyle, FontStyle, FontWeight, HighlightStyle, Hsla, StrikethroughStyle, Styled, UnderlineStyle, linear_color_stop, linear_gradient, point, px, rgb, rgba};

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
    /// The family each font file the window loaded turned out to be.
    faces: &'a HashMap<String, String>,
    pub bg: Hsla,
    pub fg: Hsla,
    pub dim: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    /// A selected row's or text's background.
    pub ring: Hsla,
    /// What a dialog dims the screen behind it with: the background, part see-through.
    pub scrim: Hsla,
    pub pos: Hsla,
    pub neg: Hsla,
}

/// A heat cell's tint at `t` (0 at its `min`, 1 at its `max`): the theme's `neg` below the
/// middle and `pos` above it, stronger towards the ends, over whatever is under the cell (the
/// panel at the middle).
pub fn heat(st: &State, t: f32) -> Option<Hsla> {
    let (name, k) = if t < 0.5 { ("neg", (0.5 - t) * 2.) } else { ("pos", (t - 0.5) * 2.) };
    let mut c = parse(&style::Theme::of(st).colour(name)?)?;
    c.a *= 0.6 * k;
    Some(c)
}

/// A colour by name: `#rrggbb`, `#rgb`, or one of the named colours.
pub fn parse(name: &str) -> Option<Hsla> {
    let hex = style::hex(name)?;
    let c = u32::from_str_radix(&hex[1..], 16).ok()?;
    Some(if hex.len() == 9 { rgba(c).into() } else { rgb(c).into() })
}

impl<'a> Look<'a> {
    pub fn new(st: &'a State, faces: &'a HashMap<String, String>) -> Look<'a> {
        let th = Theme::of(st);
        let get = |k: &str| th.colour(k).as_deref().and_then(parse).unwrap_or_default();
        let bg = get("bg");
        Look {
            th,
            faces,
            bg,
            scrim: th.colour("scrim").as_deref().and_then(parse).unwrap_or(Hsla { a: 0.6, ..bg }),
            fg: get("fg"),
            dim: get("dim"),
            border: get("border"),
            accent: get("accent"),
            ring: get("ring"),
            pos: get("pos"),
            neg: get("neg"),
        }
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

    /// The type role a `<spec>` names, if it names one.
    pub fn role(&self, spec: &str) -> Option<Role> {
        self.th.role_in(spec)
    }

    /// A role's font as a family: a family's name as it is, a font file as the family it
    /// loaded as (the default font until it has).
    pub fn fonts(&self) -> impl Fn(&str) -> String + '_ {
        |f: &str| if style::is_font_file(f) { self.faces.get(f).cloned().unwrap_or_default() } else { f.to_string() }
    }

    /// A resolved ink as the window's colours.
    pub fn ink(&self, i: &style::Ink) -> Ink {
        Ink {
            fg: i.fg.as_deref().and_then(parse),
            bg: i.bg.as_deref().and_then(parse),
            bold: i.bold,
            dim: i.dim,
            italic: i.italic,
            underline: i.underline,
            reverse: i.reverse,
            strike: i.strike,
        }
    }

    /// A resolved style's pad in pixels, `[top, right, bottom, left]`: cells are the window's
    /// cells.
    pub fn pad(r: &Resolved, cell: (f32, f32)) -> [f32; 4] {
        let Some([t, rt, b, l]) = &r.pad else { return [0.; 4] };
        let px_of = |l: &Len, per: f32| l.px(per).unwrap_or(0.);
        [px_of(t, cell.1), px_of(rt, cell.0), px_of(b, cell.1), px_of(l, cell.0)]
    }

    /// A resolved style on a box: its background (a colour or a gradient), its text, `extra`
    /// pixels of pad over its own (`[top, right, bottom, left]`), its corners, border, shadows
    /// and ring, opacity and cursor. The one place the theme's looks become GPUI's.
    pub fn boxed<E: Styled>(&self, mut el: E, r: &Resolved, cell: (f32, f32), extra: [f32; 4]) -> E {
        el = inked(el, self.ink(&r.ink));
        if let Some(g) = &r.gradient
            && let (Some(a), Some(b)) = (parse(&g.from), parse(&g.to))
        {
            el = el.bg(linear_gradient(g.angle, linear_color_stop(a, 0.), linear_color_stop(b, 1.)));
        }
        let [t, rt, b, l] = Self::pad(r, cell);
        el = padded(el, [t + extra[0], rt + extra[1], b + extra[2], l + extra[3]]);
        if let Some(rad) = r.radius {
            el = el.rounded(px(rad));
        }
        if let Some(b) = &r.border {
            el = el.border(px(b.width)).border_color(parse(&b.color).unwrap_or(self.border));
            if b.style == "dashed" {
                el = el.border_dashed();
            }
        }
        let mut shadows: Vec<BoxShadow> = r
            .shadow
            .iter()
            .map(|s| BoxShadow {
                color: parse(&s.color).map_or(self.bg, |c| Hsla { a: c.a * s.alpha.unwrap_or(1.), ..c }),
                offset: point(px(s.x), px(s.y)),
                blur_radius: px(s.blur),
                spread_radius: px(s.spread),
            })
            .collect();
        // A ring is a hard shadow its width wide, `offset` out, with the page's colour between.
        if let Some(ring) = &r.ring {
            let at = |spread: f32, color: Hsla| BoxShadow { color, offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(spread) };
            shadows.push(at(ring.offset + ring.width, parse(&ring.color).unwrap_or(self.accent)));
            if ring.offset > 0. {
                shadows.push(at(ring.offset, self.bg));
            }
        }
        if !shadows.is_empty() {
            el.style().box_shadow = Some(shadows);
        }
        if let Some(o) = r.opacity {
            el = el.opacity(o);
        }
        match r.cursor.as_deref() {
            Some("pointer") => el.cursor(CursorStyle::PointingHand),
            Some("text") => el.cursor(CursorStyle::IBeam),
            Some("grab") => el.cursor(CursorStyle::OpenHand),
            _ => el,
        }
    }
}

/// Ink on an element: its colours, weight and slant.
pub fn inked<E: Styled>(mut el: E, ink: Ink) -> E {
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

/// Pad in pixels, `[top, right, bottom, left]`.
pub fn padded<E: Styled>(el: E, p: [f32; 4]) -> E {
    if p == [0.; 4] {
        return el;
    }
    el.pt(px(p[0])).pr(px(p[1])).pb(px(p[2])).pl(px(p[3]))
}
