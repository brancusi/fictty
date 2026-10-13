//! Text with markup in the window: a template's lines, filled, as styled runs. As in the
//! terminal, a tab right-aligns the rest of its line, `{*─}` fills the room between, and a line
//! with neither wraps unless it's told not to.
//!
//! A line that stays on one is ours to lay out: it's shaped once it knows its width, and what
//! doesn't fit is cut and ends in `…`, keeping a tab's right side (the meta: a time, a count)
//! and cutting it too only past the room `keep` holds for the left. GPUI's own `text_ellipsis`
//! couldn't do this: taffy measures a text's smallest size and keeps that, so it showed "…" for
//! whole lines. A cut is noted for the readback, so an agent knows the text it reads isn't all
//! on screen.
//!
//! A line takes one type role, the first its markup names (`<display>…</>`), since a line is
//! shaped at one size.

use super::look::{Ink, Look};
use super::Shared;
use crate::model::{Seg, segments, shows};
use crate::style::Role;
use gpui::{
    AnyElement, App, Bounds, FontFeatures, FontWeight, HighlightStyle, Pixels, ShapedLine, SharedString, StyledText, TextRun, Window, canvas, div, point,
    prelude::*, px,
};
use serde_json::Value;
use std::ops::Range;
use std::sync::Arc;

/// What text is drawn with: the look, where the frame notes cuts, the node it's in, the pixels
/// `keep` holds for a line's left side, and the node's line height.
#[derive(Clone, Copy)]
pub struct Tx<'a> {
    pub lk: &'a Look<'a>,
    pub laid: &'a Shared,
    pub id: &'a str,
    pub keep: f32,
    pub line_h: f32,
}

/// Runs of one line, joined: the text and each run's highlight.
#[derive(Clone, Default)]
struct Runs {
    text: String,
    marks: Vec<(Range<usize>, HighlightStyle)>,
}

impl Runs {
    fn push(&mut self, text: &str, h: HighlightStyle) {
        let at = self.text.len();
        self.text.push_str(text);
        if h != HighlightStyle::default() {
            self.marks.push((at..self.text.len(), h));
        }
    }

    fn element(self) -> StyledText {
        StyledText::new(SharedString::from(self.text)).with_highlights(self.marks)
    }

    /// Its first `len` bytes, then `tail` in the last run's look.
    fn cut(&self, len: usize, tail: &str) -> Runs {
        let mut r = Runs { text: self.text[..len].to_string(), marks: Vec::new() };
        for (range, h) in &self.marks {
            if range.start < len {
                r.marks.push((range.start..range.end.min(len), *h));
            }
        }
        let last = self.marks.iter().find(|(range, _)| range.start < len && range.end >= len).map(|(_, h)| *h).unwrap_or_default();
        r.push(tail, last);
        r
    }

    /// The text runs GPUI shapes it with, over `base`.
    fn runs(&self, base: &gpui::TextStyle) -> Vec<TextRun> {
        let mut out = Vec::new();
        let mut at = 0;
        for (range, h) in &self.marks {
            if range.start > at {
                out.push(base.to_run(range.start - at));
            }
            out.push(base.clone().highlight(*h).to_run(range.len()));
            at = range.end;
        }
        if at < self.text.len() || out.is_empty() {
            out.push(base.to_run(self.text.len() - at));
        }
        out
    }
}

/// How long a prefix of `text` to keep (in bytes, at a char boundary) so that it and `…` fit in
/// `room`, and whether it had to be cut: the whole text when its `width` fits. `cut_width(len)`
/// measures the first `len` bytes followed by `…`.
pub fn fit(text: &str, room: f32, width: f32, mut cut_width: impl FnMut(usize) -> f32) -> (usize, bool) {
    if width <= room || text.is_empty() {
        return (text.len(), false);
    }
    // Where each prefix ends: k characters are `text[..ends[k]]`. The most that fit, by halves.
    let ends: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let (mut lo, mut hi) = (0, ends.len() - 1);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if cut_width(ends[mid]) <= room {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    (ends[lo], true)
}

/// The room a line's left and right sides get in `room`, `gap` apart: what they need, when it
/// fits. When it doesn't, the right side keeps what it can while the left keeps `keep` (or all
/// of itself, if less), and the left takes what's left.
pub fn arrange(left: f32, right: f32, gap: f32, room: f32, keep: f32) -> (f32, f32) {
    if right <= 0. {
        return (room, 0.);
    }
    if left + gap + right <= room {
        return (room - gap - right, right);
    }
    let right_room = right.min((room - gap - left.min(keep)).max(0.));
    ((room - gap - right_room).max(0.), right_room)
}

/// A run's ink: its `<spec>` over `base`, and a signed number's colour.
fn ink(lk: &Look, g: &Seg, base: Ink) -> Ink {
    let mut ink = g.style.as_deref().map_or(base, |s| lk.spec(s, base));
    match g.sign {
        Some(true) => ink.fg = Some(lk.pos),
        Some(false) => ink.fg = Some(lk.neg),
        None => {}
    }
    ink
}

/// A type role on an element: its font, size, weight, line height and tabular numbers.
pub fn typed<E: Styled>(mut el: E, role: Option<&Role>, fonts: &dyn Fn(&str) -> String) -> E {
    let Some(r) = role else { return el };
    if let Some(f) = r.role.font.as_deref().map(fonts).filter(|f| !f.is_empty()) {
        el = el.font_family(SharedString::from(f));
    }
    if let Some(s) = r.role.size {
        el = el.text_size(px(s)).line_height(px(r.role.line.unwrap_or(s * 1.3)));
    }
    if let Some(l) = r.role.line {
        el = el.line_height(px(l));
    }
    if let Some(w) = r.role.weight {
        el = el.font_weight(FontWeight(w as f32));
    }
    if r.role.tabular {
        el.text_style().get_or_insert_with(Default::default).font_features = Some(FontFeatures(Arc::new(vec![("tnum".into(), 1)])));
    }
    el
}

/// A role's line height, or `fallback`.
pub fn line_height(role: Option<&Role>, fallback: f32) -> f32 {
    role.and_then(|r| r.role.line.or(r.role.size.map(|s| s * 1.3))).unwrap_or(fallback)
}

/// One line of a template, filled from `v`. With `wrap`, a line with no tab or fill wraps;
/// otherwise it stays on one line and is cut to fit.
pub fn line(tx: &Tx, template: &str, v: &Value, base: Ink, wrap: bool) -> AnyElement {
    let lk = tx.lk;
    let segs = segments(template, v);
    // The line's type role: the first its markup names.
    let role = segs.iter().filter_map(|g| g.style.as_deref()).find_map(|s| lk.role(s));
    let line_h = line_height(role.as_ref(), tx.line_h);
    let (mut left, mut right, mut fill) = (Runs::default(), Runs::default(), None);
    let mut tabbed = false;
    for g in &segs {
        let h = lk.highlight(ink(lk, g, base));
        if g.tab {
            tabbed = true;
        } else if let Some(c) = g.fill.filter(|_| fill.is_none() && !tabbed) {
            fill = Some((c, h, left.text.len()));
        } else if tabbed {
            right.push(&g.text, h);
        } else {
            left.push(&g.text, h);
        }
    }
    let fonts = lk.fonts();
    if wrap && !tabbed && fill.is_none() {
        return typed(div().w_full().min_w_0().child(left.element()), role.as_ref(), &fonts).into_any_element();
    }
    // A fill splits the left side where it stood: what's before it, the fill, what's after.
    let (after, filler) = match fill {
        Some((c, h, at)) => {
            let after = Runs {
                text: left.text[at..].to_string(),
                marks: left.marks.iter().filter(|(r, _)| r.end > at).map(|(r, h)| (r.start.max(at) - at..r.end - at, *h)).collect(),
            };
            left.text.truncate(at);
            left.marks.retain_mut(|(r, _)| {
                r.end = r.end.min(at);
                r.start < r.end
            });
            let mut f = Runs::default();
            f.push(&c.to_string(), h);
            (after, Some(f))
        }
        None => (Runs::default(), None),
    };
    let (laid, id, keep) = (tx.laid.clone(), tx.id.to_string(), tx.keep);
    let gap = if right.text.is_empty() { 0. } else { 8. };
    let el = canvas(
        move |b: Bounds<Pixels>, window: &mut Window, _: &mut App| {
            let style = window.text_style();
            let size = style.font_size.to_pixels(window.rem_size());
            let shape = |r: &Runs, window: &mut Window| window.text_system().shape_line(SharedString::from(r.text.clone()), size, &r.runs(&style), None);
            let width = |r: &Runs, window: &mut Window| if r.text.is_empty() { 0. } else { f32::from(shape(r, window).width) };
            let room = f32::from(b.size.width);
            let (lw, aw, rw) = (width(&left, window), width(&after, window), width(&right, window));
            let (left_room, right_room) = arrange(lw + aw, rw, gap, room, keep);
            let mut cut = false;
            let mut fit_in = |r: &Runs, room: f32, w: f32, window: &mut Window| -> Runs {
                let (len, was) = fit(&r.text, room, w, |len| width(&r.cut(len, "…"), window));
                cut |= was;
                if was { r.cut(len, "…") } else { r.clone() }
            };
            let mut placed: Vec<(ShapedLine, f32)> = Vec::new();
            // The left side, then the fill and what follows it, then the right side.
            let l = fit_in(&left, left_room - aw, lw, window);
            let lw = width(&l, window);
            // A line alone on its row takes the node's `align`.
            let alone = right.text.is_empty() && after.text.is_empty() && filler.is_none();
            let x = match style.text_align {
                gpui::TextAlign::Center if alone => ((room - lw) / 2.).max(0.),
                gpui::TextAlign::Right if alone => (room - lw).max(0.),
                _ => 0.,
            };
            placed.push((shape(&l, window), x));
            if let Some(f) = &filler {
                let fw = width(f, window).max(1.);
                let n = ((left_room - lw - aw) / fw).floor().max(0.) as usize;
                let run = Runs { text: f.text.repeat(n), marks: f.marks.iter().map(|(_, h)| (0..f.text.len() * n, *h)).collect() };
                placed.push((shape(&run, window), lw));
                placed.push((shape(&after, window), left_room - aw));
            } else if !after.text.is_empty() {
                placed.push((shape(&after, window), lw));
            }
            if !right.text.is_empty() {
                let r = fit_in(&right, right_room, rw, window);
                let w = width(&r, window);
                placed.push((shape(&r, window), room - w));
            }
            if cut && !id.is_empty() {
                laid.borrow_mut().cut.push(id.clone());
            }
            placed
        },
        move |b, placed, window, cx| {
            for (line, x) in placed {
                let _ = line.paint(point(b.origin.x + px(x), b.origin.y), b.size.height, window, cx);
            }
        },
    )
    .w_full()
    .min_w_0()
    .h(px(line_h));
    typed(div().w_full().min_w_0().overflow_hidden().child(el), role.as_ref(), &fonts).into_any_element()
}

/// A template's shown lines, filled from `v`: lines whose placeholders all came out empty go.
/// With `wrap`, lines with no tab or fill wrap. `clamp` shows at most that many lines as
/// they wrap, the last ending in `…` when there's more.
pub fn lines(tx: &Tx, template: &str, v: &Value, base: Ink, wrap: bool, clamp: Option<u16>) -> AnyElement {
    let shown: Vec<&str> = template.split('\n').filter(|l| shows(l, v)).collect();
    let Some(n) = clamp else {
        return div().flex().flex_col().min_w_0().children(shown.iter().map(|l| line(tx, l, v, base, wrap))).into_any_element();
    };
    // Clamped: the lines as one text GPUI wraps and clamps; a tab is a gap.
    let mut runs = Runs::default();
    for (k, l) in shown.iter().enumerate() {
        if k > 0 {
            runs.push("\n", HighlightStyle::default());
        }
        for g in segments(l, v) {
            runs.push(if g.tab { "  " } else { &g.text }, tx.lk.highlight(ink(tx.lk, &g, base)));
        }
    }
    // Shaped at the width it gets: when it wraps to more than `n` lines, the longest start of
    // it that, with `…`, wraps to `n`. GPUI's own clamp has the trouble its ellipsis has.
    let (laid, id, line_h) = (tx.laid.clone(), tx.id.to_string(), tx.line_h);
    let el = canvas(
        move |b: Bounds<Pixels>, window: &mut Window, _: &mut App| {
            let style = window.text_style();
            let size = style.font_size.to_pixels(window.rem_size());
            let shape = |r: &Runs, window: &mut Window| {
                let lines = window.text_system().shape_text(SharedString::from(r.text.clone()), size, &r.runs(&style), Some(b.size.width), None).unwrap_or_default();
                let count: usize = lines.iter().map(|l| l.wrap_boundaries().len() + 1).sum();
                (lines, count)
            };
            let (whole, count) = shape(&runs, window);
            if count <= n as usize {
                return whole;
            }
            let ends: Vec<usize> = runs.text.char_indices().map(|(i, _)| i).collect();
            let (mut lo, mut hi) = (0, ends.len().saturating_sub(1));
            while lo < hi {
                let mid = (lo + hi).div_ceil(2);
                if shape(&runs.cut(ends[mid], "…"), window).1 <= n as usize {
                    lo = mid;
                } else {
                    hi = mid - 1;
                }
            }
            if !id.is_empty() {
                laid.borrow_mut().cut.push(id.clone());
            }
            shape(&runs.cut(ends.get(lo).copied().unwrap_or(0), "…"), window).0
        },
        move |b, lines, window, cx| {
            let mut y = b.origin.y;
            for l in lines {
                let _ = l.paint(point(b.origin.x, y), px(line_h), gpui::TextAlign::Left, Some(b), window, cx);
                y += px(line_h) * (l.wrap_boundaries().len() + 1) as f32;
            }
        },
    )
    .w_full()
    .min_w_0()
    .h(px(line_h * n as f32));
    div().w_full().min_w_0().overflow_hidden().child(el).into_any_element()
}

/// A template filled from `v` on one line, as wide as it is, never cut: a tab bar's lead and
/// trail.
pub fn whole(lk: &Look, template: &str, v: &Value, base: Ink) -> AnyElement {
    let mut runs = Runs::default();
    for g in segments(template, v) {
        runs.push(&g.text, lk.highlight(ink(lk, &g, base)));
    }
    div().flex_none().whitespace_nowrap().child(runs.element()).into_any_element()
}

/// A template filled from `v` as plain text (a title, a tab's label, a button's).
pub fn plain(template: &str, v: &Value) -> String {
    segments(template, v).into_iter().map(|s| s.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A monospace measure: one unit a character.
    fn chars(t: &str) -> f32 {
        t.chars().count() as f32
    }

    /// `fit` over a monospace measure.
    fn cut(text: &str, room: f32) -> (usize, bool) {
        fit(text, room, chars(text), |len| chars(&text[..len]) + 1.)
    }

    #[test]
    fn a_line_that_fits_is_whole_and_one_that_doesnt_ends_in_an_ellipsis() {
        assert_eq!(cut("short", 10.), (5, false));
        let (len, was) = cut("a title far too long to fit", 10.);
        assert_eq!((&"a title far too long to fit"[..len], was), ("a title f", true), "nine characters and the ellipsis make ten");
        assert_eq!(cut("héllo wörld", 6.), (6, true), "cut at a character, not a byte: \"héllo\" and …");
        assert_eq!(cut("abc", 0.5), (0, true), "no room: only the ellipsis");
    }

    #[test]
    fn a_cut_line_keeps_its_right_side() {
        // 30 cells: "a title far too long to fit" (27) and "Tue · tilde" (11), a cell apart.
        let (left, right) = arrange(27., 11., 1., 30., 0.);
        assert_eq!((left, right), (18., 11.), "the meta whole, the title in what's left");
        let (len, _) = cut("a title far too long to fit", left);
        assert_eq!(format!("{}…", &"a title far too long to fit"[..len]), "a title far too l…");
    }

    #[test]
    fn keep_holds_room_for_the_left_and_cuts_the_meta_too() {
        // "Water the plants" (16) and a long meta in 30 cells, keeping 12 for the title.
        let (left, right) = arrange(16., 34., 1., 30., 12.);
        assert_eq!((left, right), (12., 17.));
        let (len, _) = cut("Water the plants", left);
        assert_eq!(format!("{}…", &"Water the plants"[..len]), "Water the p…");
        let (len, was) = cut("due Oct 18 (in 7d) · !", right);
        assert!(was && len == 16, "the meta ends in an ellipsis too");
        assert_eq!(arrange(8., 30., 1., 30., 12.), (8., 21.), "a left shorter than keep keeps only itself");
        assert_eq!(arrange(10., 0., 0., 30., 12.), (30., 0.), "no right side: all the room is the left's");
    }
}
