//! Text with markup in the window: a template's lines, filled, as styled runs. As in the
//! terminal, a tab right-aligns the rest of its line, `{*─}` fills the room between, and a line
//! with neither wraps. A line that doesn't fit is clipped: GPUI's `text_ellipsis` truncates
//! while taffy measures a text's smallest size and keeps that, so it showed "…" for whole lines.

use super::look::{Ink, Look};
use crate::model::{Seg, segments, shows};
use gpui::{AnyElement, HighlightStyle, SharedString, StyledText, div, prelude::*};
use serde_json::Value;
use std::ops::Range;

/// Runs of one line, joined: the text and each run's highlight.
#[derive(Default)]
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

/// One line of a template, filled from `v`. `wrap` lets a line with no tab or fill wrap.
pub fn line(lk: &Look, template: &str, v: &Value, base: Ink, wrap: bool) -> AnyElement {
    let segs = segments(template, v);
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
    if !tabbed && fill.is_none() {
        let el = div().w_full().min_w_0().child(left.element());
        return if wrap { el.into_any_element() } else { el.whitespace_nowrap().overflow_hidden().into_any_element() };
    }
    // A fill splits the left side where it stood: what's before, the fill, what's after.
    let mut row = div().flex().flex_row().w_full().min_w_0().whitespace_nowrap().overflow_hidden();
    match fill {
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
            let filler = Runs { text: c.to_string().repeat(400), marks: vec![(0..c.len_utf8() * 400, h)] };
            row = row.child(div().flex_none().child(left.element())).child(div().flex_1().min_w_0().overflow_hidden().child(filler.element()));
            if !after.text.is_empty() {
                row = row.child(div().flex_none().child(after.element()));
            }
        }
        None => row = row.child(div().flex_1().min_w_0().overflow_hidden().child(left.element())),
    }
    if !right.text.is_empty() {
        row = row.child(div().flex_none().pl_2().child(right.element()));
    }
    row.into_any_element()
}

/// A template's shown lines, filled from `v`: lines whose placeholders all came out empty go.
pub fn lines(lk: &Look, template: &str, v: &Value, base: Ink) -> AnyElement {
    let shown = template.split('\n').filter(|l| shows(l, v)).map(|l| line(lk, l, v, base, true));
    div().flex().flex_col().min_w_0().children(shown).into_any_element()
}

/// A template filled from `v` as plain text (a title, a tab's label, a button's).
pub fn plain(template: &str, v: &Value) -> String {
    segments(template, v).into_iter().map(|s| s.text).collect()
}
