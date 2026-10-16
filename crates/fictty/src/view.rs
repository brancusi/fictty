//! The view: `State` in, a frame out. It reads nothing else (no store, clock, env or files).
//! Besides the frame it returns the hit regions (what is under each cell, for the mouse), and
//! it places the UI's layers with caretline-layers over what it drew.

use crate::model::{Float, Kind, LayerSpec, Len, Lens, Look, Measure, Node, Seg, Size, Style as NodeStyle, ThemeSpec, Variant, fill, lookup, number, scalar, segments};
use crate::state::{Slot, State, bound, field_view, items, selected, shown, shown_in, sized, split_bind, tab_of, value_of};
use crate::gfx::{Pic, Placed, Rgb};
use caretline_layers as cl;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Map, MapResolution, Points};
use ratatui::widgets::{Axis, Block, BorderType, Chart, Dataset, Gauge, GraphType, LegendPosition, Paragraph, Sparkline, Wrap};
use serde_json::{Value, json};

pub use crate::engine::Hit;

/// A ratatui rectangle as the engine's.
fn cells(r: Rect) -> crate::engine::Rect {
    crate::engine::Rect::new(r.x, r.y, r.width, r.height)
}

/// The screen's colours: the UI's `theme`, over these defaults.
struct Theme {
    bg: Option<Color>,
    fg: Option<Color>,
    border: Color,
    accent: Color,
    dim: Color,
    pos: Color,
    neg: Color,
    hover: Color,
    panel: Color,
    ring: Color,
}

struct Cx<'a> {
    st: &'a State,
    /// The active theme, if one is loaded and chosen.
    active: Option<&'a ThemeSpec>,
    th: Theme,
    hits: Vec<Hit>,
    rects: Vec<(String, crate::engine::Rect)>,
    anchors: cl::AnchorMap,
    pics: Vec<Placed>,
    scaled: Vec<Scaled>,
    sizes: Vec<(String, u16, u16)>,
    wake: Option<u64>,
    /// A `toasts` node showed them, so the corner doesn't.
    toasts_placed: bool,
    /// The node about to be drawn was already checked against its `when` by its row or column.
    laid_out: bool,
    areas: Vec<(String, u16, u16)>,
}

/// Text the terminal draws larger (OSC 66): its cell, the text, the scale and the style.
#[derive(Clone, Debug, PartialEq)]
pub struct Scaled {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub scale: u8,
    pub style: Style,
}

/// What a frame needs besides its cells: what's under each cell, pictures, larger text.
#[derive(Default)]
pub struct Out {
    pub hits: Vec<Hit>,
    /// Where each node with an id was laid out.
    pub rects: Vec<(String, crate::engine::Rect)>,
    pub pics: Vec<Placed>,
    pub scaled: Vec<Scaled>,
    /// The cells each editor got: (id, width, height). The runtime tells an editor when its
    /// own size differs, so its scroll keeps the caret in view.
    pub sizes: Vec<(String, u16, u16)>,
    /// When (on the logical clock) the frame changes by itself next: a toast going, a spinner
    /// turning. The runtime ticks and draws again then.
    pub wake: Option<u64>,
    /// The area each node whose `when` reads `$width` or `$height` was laid out in: (id, width,
    /// height). The runtime tells the state where it differs.
    pub areas: Vec<(String, u16, u16)>,
}

impl Cx<'_> {
    /// A colour by theme name, CSS-ish name or `#rrggbb`.
    fn color(&self, name: &str) -> Option<Color> {
        token(self.st, self.active, name).unwrap_or(name).parse().ok()
    }

    /// A part of a component, styled by the active theme (`table.header`, `row.selected`), or
    /// `fallback` when the theme doesn't say.
    fn look(&self, part: &str, fallback: Style) -> Style {
        match self.active.and_then(|t| t.parts.get(part)) {
            Some(spec) => self.spec(&spec.spec(), Style::default()),
            None => fallback,
        }
    }

    fn style(&self, s: Option<&NodeStyle>) -> Style {
        let mut out = Style::default();
        let Some(s) = s else { return out };
        if let Some(c) = s.fg.as_deref().and_then(|c| self.color(c)) {
            out = out.fg(c);
        }
        if let Some(c) = s.bg.as_ref().and_then(|c| self.color(c.colour())) {
            out = out.bg(c);
        }
        for (on, m) in [(s.bold, Modifier::BOLD), (s.dim, Modifier::DIM), (s.italic, Modifier::ITALIC)] {
            if on {
                out = out.add_modifier(m);
            }
        }
        out
    }

    fn rgb(&self, name: &str) -> Option<Rgb> {
        match self.color(name)? {
            Color::Rgb(r, g, b) => Some((r, g, b)),
            _ => None,
        }
    }

    fn mark(&mut self, id: &str, row: Option<usize>, tab: Option<usize>, rect: Rect) {
        self.hits.push(Hit { rect: cells(rect), id: id.to_string(), row, tab, text: false });
        if row.is_none() && tab.is_none() {
            self.rects.push((id.to_string(), cells(rect)));
        }
        let key = match row {
            Some(r) => cl::AnchorKey::host("row", &format!("{id}#{r}")),
            None if tab.is_some() => return,
            None => cl::AnchorKey::host("node", id),
        };
        self.anchors.put(key, cl::Rect::new(rect.x, rect.y, rect.width, rect.height));
    }

    fn hovered(&self, id: &str, row: usize) -> bool {
        self.st.hover.as_ref().is_some_and(|h| h.id == id && h.row == Some(row))
    }

    /// Spans from a template: `{±x}` coloured by sign.
    /// Spans from a template (tabs and fills dropped): `{±x}` coloured by sign, `<spec>` styled.
    fn spans(&self, template: &str, v: &Value, base: Style) -> Vec<Span<'static>> {
        self.parts(template, v, base)
            .into_iter()
            .filter_map(|p| match p {
                Part::Text(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    fn parts(&self, template: &str, v: &Value, base: Style) -> Vec<Part> {
        segments(template, v).into_iter().map(|g| self.part(g, base)).collect()
    }

    fn part(&self, g: Seg, base: Style) -> Part {
        let mut st = match &g.style {
            Some(spec) => self.spec(spec, base),
            None => base,
        };
        match g.sign {
            Some(true) => st = st.fg(self.th.pos),
            Some(false) => st = st.fg(self.th.neg),
            None => {}
        }
        if g.tab {
            Part::Tab
        } else if let Some(c) = g.fill {
            Part::Fill(c, st)
        } else {
            Part::Text(Span::styled(g.text, st))
        }
    }

    /// A style spec over `base`: `accent+b`, `muted`, `text+on-selection`, `#ff8800+u`. A name
    /// that's no colour but one of the theme's parts takes that part's style (`<link>`).
    fn spec(&self, spec: &str, base: Style) -> Style {
        self.spec_at(spec, base, 0)
    }

    fn spec_at(&self, spec: &str, base: Style, depth: u8) -> Style {
        let mut st = base;
        for t in spec.split('+') {
            st = match t {
                "b" => st.add_modifier(Modifier::BOLD),
                "d" => st.add_modifier(Modifier::DIM),
                "i" => st.add_modifier(Modifier::ITALIC),
                "u" => st.add_modifier(Modifier::UNDERLINED),
                "r" => st.add_modifier(Modifier::REVERSED),
                "s" => st.add_modifier(Modifier::CROSSED_OUT),
                t => match t.strip_prefix("on-") {
                    Some(c) => self.color(c).map_or(st, |c| st.bg(c)),
                    None => match (self.color(t), self.active.and_then(|a| a.parts.get(t))) {
                        (Some(c), _) => st.fg(c),
                        (None, Some(part)) if depth < 4 => self.spec_at(&part.spec(), st, depth + 1),
                        _ => st,
                    },
                },
            };
        }
        st
    }
}

/// A run of a drawn line: text, the point where the right-aligned rest starts, or a fill.
enum Part {
    Text(Span<'static>),
    Tab,
    Fill(char, Style),
}

/// One line of parts into `r`: the left side from `r.x`, anything after a tab right-aligned to
/// the end (the left side is cut to leave a cell between them), and a fill taking what's left.
fn draw_parts(buf: &mut Buffer, r: Rect, parts: Vec<Part>, keep: u16) {
    let (mut left, mut right, mut seen_tab) = (Vec::new(), Vec::new(), false);
    for p in parts {
        match p {
            Part::Tab if !seen_tab => seen_tab = true,
            Part::Tab => {}
            p if seen_tab => right.push(p),
            p => left.push(p),
        }
    }
    let width = |ps: &[Part]| ps.iter().map(|p| if let Part::Text(s) = p { s.width() } else { 0 }).sum::<usize>();
    // The right side yields to `keep` cells of the left side (and the cell between them).
    let rw = width(&right).min(r.width as usize).min(r.width.saturating_sub(keep.saturating_add(1)) as usize);
    let room = (r.width as usize).saturating_sub(if rw > 0 { rw + 1 } else { 0 });
    let fixed = width(&left);
    let fills = left.iter().any(|p| matches!(p, Part::Fill(..)));
    let mut x = r.x;
    let end = r.x + room as u16;
    let cut = !fills && fixed > room && room > 1;
    for p in left {
        if x >= end {
            break;
        }
        match p {
            Part::Text(s) => {
                let (nx, _) = buf.set_span(x, r.y, &s, end - x);
                x = nx;
            }
            Part::Fill(c, st) => {
                let n = room.saturating_sub(fixed) as u16;
                let n = n.min(end - x);
                buf.set_string(x, r.y, c.to_string().repeat(n as usize), st);
                x += n;
            }
            Part::Tab => {}
        }
    }
    // Text that didn't fit ends in an ellipsis, so a cut is never mistaken for the whole.
    if cut {
        buf[(end - 1, r.y)].set_symbol("…");
    }
    let cut_right = width(&right) > rw && rw > 1;
    let mut x = r.x + r.width - rw as u16;
    for p in right {
        if let Part::Text(s) = p {
            let (nx, _) = buf.set_span(x, r.y, &s, r.right().saturating_sub(x));
            x = nx;
        }
    }
    if cut_right {
        buf[(r.right() - 1, r.y)].set_symbol("…");
    }
}

/// A token's colour: the active theme's, else the UI's own `theme` (a fallback under every theme).
fn token<'a>(st: &'a State, active: Option<&'a ThemeSpec>, name: &str) -> Option<&'a str> {
    active.and_then(|t| t.tokens.get(name)).or_else(|| st.ui.theme.get(name)).map(String::as_str)
}

/// The theme the person chose, if it's loaded.
pub fn active(st: &State) -> Option<&ThemeSpec> {
    st.theme.as_ref().and_then(|n| st.themes.get(n))
}

fn theme(st: &State, active: Option<&ThemeSpec>) -> Theme {
    let tok = |k: &str| token(st, active, k).and_then(|c| c.parse().ok());
    let get = |k: &str, d: Color| tok(k).unwrap_or(d);
    Theme {
        bg: tok("bg"),
        fg: tok("fg"),
        border: get("border", Color::Rgb(0x4a, 0x50, 0x60)),
        accent: get("accent", Color::Rgb(0x5f, 0xb3, 0xff)),
        dim: get("dim", Color::Rgb(0x55, 0x58, 0x60)),
        pos: get("pos", Color::Rgb(0x4c, 0xd1, 0x7f)),
        neg: get("neg", Color::Rgb(0xf2, 0x5f, 0x5c)),
        hover: get("hover", Color::Rgb(0x26, 0x2c, 0x3a)),
        panel: get("panel", Color::Rgb(0x1f, 0x24, 0x30)),
        ring: get("ring", Color::Rgb(0x26, 0x3a, 0x55)),
    }
}

/// Draw the whole screen; returns what's under each cell and the pictures to place.
/// A wide character (漢, 🙂) that something was drawn over (a toast, a float, a layer covering
/// its second cell), or that the right edge cuts, goes: what's on top wins. A terminal can't show
/// both, and ratatui never sends a cell it thinks a wide character covers, so the thing on top
/// would be the part that's lost.
fn whole_wide(buf: &mut Buffer) {
    let a = buf.area;
    for y in a.top()..a.bottom() {
        let mut x = a.left();
        while x < a.right() {
            let w = Span::raw(buf[(x, y)].symbol()).width().max(1) as u16;
            let cut = x + w > a.right();
            let covered = (1..w).any(|i| x + i < a.right() && !matches!(buf[(x + i, y)].symbol(), " " | ""));
            if w > 1 && (cut || covered) {
                buf[(x, y)].set_symbol(" ");
                x += 1;
            } else {
                x += w;
            }
        }
    }
}

pub fn draw(st: &State, f: &mut Frame) -> Out {
    let act = active(st);
    let mut cx = Cx { st, active: act, th: theme(st, act), hits: Vec::new(), rects: Vec::new(), anchors: cl::AnchorMap::new(), pics: Vec::new(), scaled: Vec::new(), sizes: Vec::new(), wake: None, toasts_placed: false, laid_out: false, areas: Vec::new() };
    let mut base = Style::default();
    if let Some(bg) = cx.th.bg {
        base = base.bg(bg);
    }
    if let Some(fg) = cx.th.fg {
        base = base.fg(fg);
    }
    let all = f.area();
    f.buffer_mut().set_style(all, base);
    let bar = st.ui.status_bar.is_none_or(|b| b.shown());
    let [body, status] = Layout::vertical([Constraint::Fill(1), Constraint::Length(bar as u16)]).areas(f.area());
    node(&mut cx, st.tree(), f, body);
    let shade = layers(&cx, f.buffer_mut(), body);
    cx.pics.extend(shade);
    if !cx.toasts_placed {
        toasts(&mut cx, f.buffer_mut(), body);
    }
    // A lasting write going changes the frame too.
    if let Some(at) = st.expiring.iter().map(|e| e.at).min() {
        cx.wake = Some(cx.wake.map_or(at, |w| w.min(at)));
    }
    if !bar {
        whole_wide(f.buffer_mut());
        return Out { hits: cx.hits, rects: cx.rects, pics: cx.pics, scaled: cx.scaled, sizes: cx.sizes, wake: cx.wake, areas: cx.areas };
    }
    let line = match st.status.as_ref().or(st.problems.first()) {
        Some(s) => Line::styled(s.clone(), cx.look("status.error", Style::default().fg(cx.th.neg))),
        None => Line::styled(
            format!(" ui v{} · tab focus · j/k move · ←/→ tabs · click, wheel, hover · q quit", st.version),
            cx.look("status", Style::default().fg(cx.th.dim)),
        ),
    };
    let s = &st.stats;
    let meter = format!("{:>5.1} fps · draw {:.2}ms (max {:.1}) · {:>6.0} msg/s ", s.fps, s.draw_ms, s.max_ms, s.msgs);
    let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(meter.chars().count() as u16)]).areas(status);
    f.render_widget(Paragraph::new(line), left);
    f.render_widget(Paragraph::new(Line::styled(meter, Style::default().fg(Color::Yellow))), right);
    whole_wide(f.buffer_mut());
    Out { hits: cx.hits, rects: cx.rects, pics: cx.pics, scaled: cx.scaled, sizes: cx.sizes, wake: cx.wake, areas: cx.areas }
}

/// The toasts, newest at the bottom, in the body's bottom-right corner, a row up from its edge (a
/// UI's own footer often sits there).
fn toasts(cx: &mut Cx, buf: &mut Buffer, body: Rect) {
    let mut bottom = body.bottom().saturating_sub(1);
    for t in cx.st.toasts.iter().rev() {
        cx.wake = Some(cx.wake.map_or(t.until, |w| w.min(t.until)));
        let w = (t.text.chars().count() as u16 + 4).min(body.width.saturating_sub(2)).max(5);
        if bottom < body.y + 3 || body.width < 6 {
            break;
        }
        let r = Rect::new(body.right().saturating_sub(w + 1), bottom - 3, w, 3);
        let tone = t.tone.as_deref().and_then(|c| cx.color(c)).unwrap_or(cx.th.accent);
        let fill = toast_look(cx, t.tone.as_deref());
        ratatui::widgets::Widget::render(ratatui::widgets::Clear, r, buf);
        buf.set_style(r, fill);
        let block = Block::bordered().border_type(BorderType::Rounded).border_style(fill.fg(tone));
        let inner = block.inner(r);
        ratatui::widgets::Widget::render(block, r, buf);
        buf.set_stringn(inner.x + 1, inner.y, &t.text, inner.width.saturating_sub(1) as usize, fill);
        bottom -= 3;
    }
}

/// The spinner's glyph now, and a wake for the next one.
fn spin(cx: &mut Cx) -> &'static str {
    let (glyph, next) = crate::state::spinner(cx.st.now_ms);
    cx.wake = Some(cx.wake.map_or(next, |w| w.min(next)));
    glyph
}

/// A toast's style: the theme's part for its tone (`toast.neg`), else `toast`, else the panel.
fn toast_look(cx: &Cx, tone: Option<&str>) -> Style {
    let base = cx.look("toast", Style::default().bg(cx.th.panel).fg(cx.th.fg.unwrap_or(Color::White)));
    match tone.and_then(|t| cx.active?.parts.get(&format!("toast.{t}"))) {
        Some(spec) => cx.spec(&spec.spec(), base),
        None => base,
    }
}

/// A floating node's place in its stack, with the cell's size in pixels.
fn float_rect(fl: &Float, area: Rect, (cw, ch): (f32, f32)) -> Rect {
    let m = fl.margin.as_ref().map_or(0, |l| len_cells(l.sides()[3]).unwrap_or(0));
    let dim = |s: Option<&Size>, full: u16, cell: f32| -> u16 {
        let n = match s.map(Size::measure) {
            Some(Measure::Cells(n)) => n,
            Some(Measure::Px(p)) => in_cells(p, cell),
            Some(Measure::Percent(p)) => (full as f32 * p / 100.0) as u16,
            None | Some(Measure::Share(_)) => full,
        };
        n.min(full)
    };
    let w = dim(fl.width.as_ref(), area.width.saturating_sub(2 * m), cw);
    let h = dim(fl.height.as_ref(), area.height.saturating_sub(2 * m), if ch > 0.0 { ch } else { 16.0 });
    let at = fl.at.as_deref().unwrap_or("center");
    let x = if at.contains("left") {
        area.x + m
    } else if at.contains("right") {
        area.right().saturating_sub(m + w)
    } else {
        area.x + (area.width - w) / 2
    };
    let y = if at.contains("top") {
        area.y + m
    } else if at.contains("bottom") {
        area.bottom().saturating_sub(m + h)
    } else {
        area.y + (area.height - h) / 2
    };
    Rect::new(x, y, w, h)
}

/// How an editor or input looks: its text, the caret, a selection, the placeholder, and its
/// Markdown styles when it has them.
struct FieldLook {
    text: Style,
    caret: Style,
    selection: Style,
    placeholder: Style,
    markdown: Option<Markdown>,
}

struct Markdown {
    heading: Style,
    bullet: Style,
    quote: Style,
    code: Style,
    task: Style,
    done: Style,
    link: Style,
    /// The kinds of link the editor has (`wiki`, `tag`, `url`), styled as `link`.
    links: Vec<String>,
}

/// A text cell's Markdown style: its line's kind (a heading, a quote), the list marker at the
/// line's start, or inline `code`. `col` is the char's column in its line.
#[cfg(test)]
fn md_style(md: &Markdown, line: &str, col: usize, base: Style) -> Style {
    md_styles(md, line, base).get(col).copied().unwrap_or(base)
}

/// The look of each character of a Markdown line, by its role (`model::md_roles`).
fn md_styles(md: &Markdown, line: &str, base: Style) -> Vec<Style> {
    use crate::model::Md;
    let kinds: Vec<&str> = md.links.iter().map(String::as_str).collect();
    crate::model::md_roles(line, &kinds)
        .into_iter()
        .map(|r| match r {
            Md::Text => base,
            Md::Heading => md.heading,
            Md::Bullet => md.bullet,
            Md::Quote => md.quote,
            Md::Code => md.code,
            Md::Task => md.task,
            Md::Done => md.done,
            Md::Link => md.link,
        })
        .collect()
}

/// An editor's cells, from caretline's own view of it. Until the runtime has told it its size
/// (the first frame, or just after a resize), a copy sized to the area is drawn instead.
fn editor(buf: &mut Buffer, area: Rect, doc: &caretline::Document, view: &caretline::View, look: &FieldLook, placeholder: Option<&str>, focused: bool) {
    use caretline::view::Role;
    if area.width == 0 || area.height == 0 {
        return;
    }
    buf.set_style(area, look.text);
    if doc.text.len_chars() == 0 {
        if let Some(p) = placeholder {
            buf.set_stringn(area.x, area.y, p, area.width as usize, look.placeholder);
        }
        if focused {
            buf[(area.x, area.y)].set_style(look.caret);
        }
        return;
    }
    let vp = view.viewport;
    let frame = if (vp.width, vp.height) == (area.width, area.height) {
        caretline::view::render(doc, view)
    } else {
        let (mut d, mut vs) = (doc.clone(), [view.clone()]);
        caretline::views::update_doc(&mut d, &mut vs, 0, caretline::Msg::resize(area.width, area.height));
        caretline::view::render(&d, &vs[0])
    };
    // For Markdown: the document line a row shows, where it starts, and each character's look,
    // worked out once for all the rows a wrapped line takes.
    let mut styled: Option<(usize, usize, Vec<Style>)> = None;
    for y in 0..frame.height.min(area.height) {
        let line = match (&look.markdown, frame.rows.get(y as usize)) {
            (Some(md), Some(caretline::view::RowInfo::Text { line, .. })) => {
                if styled.as_ref().is_none_or(|(l, ..)| l != line) {
                    styled = Some((*line, doc.text.line_to_char(*line), md_styles(md, &d_line(doc, *line), look.text)));
                }
                styled.as_ref().map(|(_, start, styles)| (start, styles))
            }
            _ => None,
        };
        for x in 0..frame.width.min(area.width) {
            let cell = frame.cell(x, y);
            if cell.symbol.is_empty() {
                continue; // the second half of a wide grapheme
            }
            let text = match (line, cell.char_idx) {
                (Some((start, styles)), Some(i)) if i as usize >= *start => styles.get(i as usize - start).copied().unwrap_or(look.text),
                _ => look.text,
            };
            let s = match cell.role {
                Role::Selection => look.selection,
                Role::Caret => look.caret,
                Role::Status | Role::StatusAccent | Role::Hang => look.text.add_modifier(Modifier::DIM),
                _ => text,
            };
            buf.set_string(area.x + x, area.y + y, &cell.symbol, s);
        }
    }
    // The primary caret: a block, drawn only while the editor has focus.
    if let Some((x, y)) = frame.cursor.filter(|&(x, y)| focused && x < area.width && y < area.height) {
        let c = &mut buf[(area.x + x, area.y + y)];
        c.set_style(look.caret);
        if c.symbol().trim().is_empty() {
            c.set_symbol(" ");
        }
    }
}

/// A document line's text, without its line break.
fn d_line(doc: &caretline::Document, line: usize) -> String {
    let l = doc.text.line(line).to_string();
    l.trim_end_matches(['\n', '\r']).to_string()
}

fn border_type(edge: Option<&str>) -> BorderType {
    match edge {
        Some("double") => BorderType::Double,
        Some("thick") => BorderType::Thick,
        Some("plain") => BorderType::Plain,
        _ => BorderType::Rounded,
    }
}

fn node(cx: &mut Cx, n: &Node, f: &mut Frame, area: Rect) {
    let st = cx.st;
    let laid_out = std::mem::take(&mut cx.laid_out);
    if let (Some(id), Some(w)) = (&n.id, n.when.as_deref())
        && sized(w)
        && !laid_out
    {
        cx.areas.push((id.clone(), area.width, area.height));
    }
    if !laid_out && !shown_in(st, n, Some((area.width, area.height))) {
        return;
    }
    let focused = n.id.is_some() && n.id == st.focus;
    if let Some(id) = &n.id {
        cx.mark(id, None, None, area);
    }
    // A variant gives a style under the node's own, and a border and backdrop when it has none.
    let var: Option<&Variant> = n.variant.as_ref().and_then(|v| cx.active?.variants.get(v));
    let mut style = cx.style(var.and_then(|v| v.style.as_ref())).patch(cx.style(n.style.as_ref()));
    if let Some(c) = n.tone.as_deref().and_then(|t| cx.color(t)) {
        style = style.fg(c);
    }
    let edge_name = n.edge.as_deref().or(var.and_then(|v| v.edge.as_deref()));
    let backdrop = n.backdrop.as_ref().or(var.and_then(|v| v.backdrop.as_ref()));
    let bordered = n.title.is_some() || n.border || edge_name.is_some() || backdrop.is_some() || var.is_some_and(|v| v.border);
    let title_style = |cx: &Cx| {
        let fallback = if focused { Style::default().fg(cx.th.accent).add_modifier(Modifier::BOLD) } else { Style::default().fg(cx.th.fg.unwrap_or(Color::Reset)) };
        let base = cx.look(if focused { "title.focused" } else { "title" }, fallback);
        match var.and_then(|v| v.title.as_deref()) {
            Some(spec) => cx.spec(spec, base),
            None => base,
        }
    };
    let pixels = st.caps.graphics;
    let card = backdrop.filter(|_| pixels).and_then(|b| {
        Some(Pic::Card {
            fill: cx.rgb(&b.fill)?,
            fill2: b.fill2.as_deref().and_then(|c| cx.rgb(c)),
            border: b.border.as_deref().and_then(|c| cx.rgb(c)),
            radius: b.radius,
            shadow: b.shadow,
            glow: b.glow.as_deref().and_then(|c| cx.rgb(c)),
            page: cx.th.bg.and_then(|c| match c {
                Color::Rgb(r, g, b) => Some((r, g, b)),
                _ => None,
            }),
        })
    });
    let inner = if let Some(pic) = card {
        // A pixel card stands in for the border: the title sits on its top edge, the content one
        // cell in from each side.
        // Under every cell background (kitty draws z below -2^30 beneath them), with the cells
        // over it left clear, so the card shows through and a selected row's colour stays on top.
        cx.pics.push(Placed { x: area.x, y: area.y, w: area.width, h: area.height, z: crate::gfx::UNDER, pic });
        f.buffer_mut().set_style(area, Style::default().bg(Color::Reset));
        if let Some(t) = &n.title {
            let ts = title_style(cx);
            f.buffer_mut().set_string(area.x + 2, area.y + 1, t, ts);
            Rect::new(area.x + 2, area.y + 3, area.width.saturating_sub(4), area.height.saturating_sub(4))
        } else {
            Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(4), area.height.saturating_sub(2))
        }
    } else if bordered {
        let edge = if focused { cx.look("border.focused", Style::default().fg(cx.th.accent)) } else { cx.look("border", Style::default().fg(cx.th.border)) };
        let mut b = Block::bordered().border_type(border_type(edge_name)).border_style(edge);
        if let Some(bg) = style.bg {
            b = b.style(Style::default().bg(bg));
        }
        if let Some(t) = &n.title {
            b = b.title(Span::styled(format!(" {t} "), title_style(cx)));
        }
        let inner = b.inner(area);
        f.render_widget(b, area);
        inner
    } else {
        if let Some(bg) = style.bg {
            f.buffer_mut().set_style(area, Style::default().bg(bg));
        }
        area
    };
    let inner = match n.pad.as_ref().map(Lens::sides) {
        Some([top, _, _, left]) => inner.inner(ratatui::layout::Margin { horizontal: len_cells(left).unwrap_or(0), vertical: len_cells(top).unwrap_or(0) }),
        None => inner,
    };
    // A bound node whose source hasn't arrived (or failed) says so instead of drawing.
    if let Some(bind) = n.bind.as_ref().filter(|b| !b.starts_with('@')) {
        let name = split_bind(bind).0;
        match st.data.get(name) {
            Some(Slot::Error { message }) => {
                let p = Paragraph::new(format!("{name}: {message}")).style(Style::default().fg(cx.th.neg));
                return f.render_widget(p.wrap(Wrap { trim: true }), inner);
            }
            Some(Slot::Ready { .. }) => {}
            _ if st.ui.data.contains_key(name) => {
                let g = spin(cx);
                return f.render_widget(Paragraph::new(format!("{g} {name}…")).style(Style::default().fg(cx.th.dim)), inner);
            }
            _ => {}
        }
    }
    let align = match n.align.as_deref() {
        Some("center") => Alignment::Center,
        Some("right") => Alignment::Right,
        _ => Alignment::Left,
    };
    let id = n.id.clone().unwrap_or_default();
    match &n.kind {
        Kind::Col { children, gap, .. } | Kind::Row { children, gap, .. } => {
            // `$width` and `$height` in a child's `when` are this node's inner size.
            let here = (inner.width, inner.height);
            for c in children {
                if let (Some(id), Some(w)) = (&c.id, c.when.as_deref())
                    && sized(w)
                {
                    cx.areas.push((id.clone(), here.0, here.1));
                }
            }
            let showing: Vec<&Node> = children.iter().filter(|c| shown_in(st, c, Some(here))).collect();
            let along = if matches!(n.kind, Kind::Col { .. }) { inner.height } else { inner.width };
            let cell = if matches!(n.kind, Kind::Col { .. }) { if st.caps.cell_h > 0.0 { st.caps.cell_h } else { 16.0 } } else { st.caps.cell_w };
            let cs: Vec<Constraint> = showing.iter().map(|c| bounded(constraint(c.size.as_ref(), cell), c.min.as_ref().and_then(len_cells), c.max.as_ref().and_then(len_cells), along)).collect();
            let areas = match n.kind {
                Kind::Col { .. } => Layout::vertical(cs).spacing(gap.as_ref().map_or(0, |g| len_cells(g.xy()[1]).unwrap_or(0))).split(inner),
                _ => Layout::horizontal(cs).spacing(gap.as_ref().map_or(0, |g| len_cells(g.xy()[0]).unwrap_or(0))).split(inner),
            };
            for (c, a) in showing.into_iter().zip(areas.iter()) {
                cx.laid_out = true;
                node(cx, c, f, *a);
            }
        }
        Kind::Stack { children } => {
            for c in children.iter().filter(|c| shown(st, c)) {
                let at = c.float.as_ref().map_or(inner, |fl| float_rect(fl, inner, (st.caps.cell_w, st.caps.cell_h)));
                if c.modal {
                    // What's behind a dialog dims, and a click there goes nowhere.
                    let buf = f.buffer_mut();
                    for y in inner.top()..inner.bottom() {
                        for x in inner.left()..inner.right() {
                            let cell = &mut buf[(x, y)];
                            cell.modifier.insert(Modifier::DIM);
                        }
                    }
                    cx.hits.push(Hit { rect: cells(inner), id: String::new(), row: None, tab: None, text: false });
                }
                // A floating or modal layer covers what's under it, characters and all.
                if c.float.is_some() || c.modal {
                    f.render_widget(ratatui::widgets::Clear, at);
                    let mut base = Style::default();
                    if let Some(bg) = cx.th.bg {
                        base = base.bg(bg);
                    }
                    if let Some(fg) = cx.th.fg {
                        base = base.fg(fg);
                    }
                    f.buffer_mut().set_style(at, base);
                }
                node(cx, c, f, at);
            }
        }
        // Expanded into a column or a row before drawing (`State::tree`).
        Kind::Each { .. } => {}
        Kind::Toasts {} => {
            cx.toasts_placed = true;
            // While a toast shows, the node covers its whole area, whatever was drawn there.
            if let Some(newest) = st.toasts.last() {
                f.render_widget(ratatui::widgets::Clear, inner);
                f.buffer_mut().set_style(inner, toast_look(cx, newest.tone.as_deref()));
            }
            for (k, t) in st.toasts.iter().rev().take(inner.height as usize).enumerate() {
                cx.wake = Some(cx.wake.map_or(t.until, |w| w.min(t.until)));
                let line = Rect::new(inner.x, inner.y + k as u16, inner.width, 1);
                let mut ts = toast_look(cx, t.tone.as_deref());
                // Without a part for its tone, the tone colours the text.
                if t.tone.as_deref().is_some_and(|tn| cx.active.is_none_or(|a| !a.parts.contains_key(&format!("toast.{tn}"))))
                    && let Some(c) = t.tone.as_deref().and_then(|c| cx.color(c))
                {
                    ts = ts.fg(c);
                }
                f.buffer_mut().set_style(line, ts);
                f.buffer_mut().set_stringn(line.x + 1, line.y, &t.text, line.width.saturating_sub(2) as usize, ts);
            }
        }
        Kind::Spinner { text } => {
            let v = bound(st, n).unwrap_or(&Value::Null);
            let mut parts = vec![Part::Text(Span::styled(format!("{} ", spin(cx)), cx.look("spinner", style.fg(cx.th.accent))))];
            parts.extend(cx.parts(text, v, style));
            draw_parts(f.buffer_mut(), Rect { height: 1.min(inner.height), ..inner }, parts, 0);
        }
        Kind::Button { label } => {
            let v = bound(st, n).unwrap_or(&Value::Null);
            let text: String = segments(label, v).into_iter().map(|g| g.text).collect();
            let bs = if focused {
                cx.look("button.focused", Style::default().fg(Color::Black).bg(cx.th.accent).add_modifier(Modifier::BOLD))
            } else {
                cx.look("button", Style::default().fg(cx.th.fg.unwrap_or(Color::White)).bg(cx.th.panel))
            };
            let w = (text.chars().count() as u16 + 4).min(inner.width);
            let x = match align {
                Alignment::Center => inner.x + (inner.width - w) / 2,
                Alignment::Right => inner.right() - w,
                Alignment::Left => inner.x,
            };
            let r = Rect::new(x, inner.y + inner.height.saturating_sub(1) / 2, w, 1.min(inner.height));
            f.buffer_mut().set_style(r, bs);
            f.buffer_mut().set_stringn(r.x + 2, r.y, &text, w.saturating_sub(4) as usize, bs);
        }
        Kind::Text { text, .. } if n.scale.is_some_and(|k| k > 1) && st.text_sizing => {
            // Larger text: the terminal draws it over `scale` rows; the cells under it stay blank.
            let v = bound(st, n).unwrap_or(&Value::Null);
            let k = n.scale.unwrap_or(1);
            for (i, l) in text.split('\n').enumerate() {
                let y = inner.y + i as u16 * k as u16;
                if y + k as u16 > inner.bottom() {
                    break;
                }
                let segs = segments(l, v);
                let st_ = segs.iter().find_map(|g| g.style.as_deref()).map_or(style, |sp| cx.spec(sp, style));
                let t: String = segs.into_iter().map(|g| g.text).collect();
                cx.scaled.push(Scaled { x: inner.x, y, text: t, scale: k, style: st_ });
            }
        }
        Kind::Text { text, .. } if n.scale.is_some_and(|k| k >= 3) => {
            // No larger text here: three times the size falls back to the block-digit font.
            let v = bound(st, n).unwrap_or(&Value::Null);
            let segs = segments(text.split('\n').next().unwrap_or(""), v);
            let st_ = segs.iter().find_map(|g| g.style.as_deref()).map_or(style, |sp| cx.spec(sp, style));
            let t: String = segs.into_iter().map(|g| g.text).collect();
            let lines: Vec<Line> = big(&t).into_iter().map(|l| Line::styled(l, st_)).collect();
            f.render_widget(Paragraph::new(lines).alignment(align), inner);
        }
        Kind::Text { text, .. } => {
            let v = bound(st, n).unwrap_or(&Value::Null);
            // Lines with a tab or a fill are laid out by hand (no wrapping); the rest wrap.
            // Lines whose placeholders all came out empty are left out (optional details).
            let shown: Vec<&str> = text.split('\n').filter(|l| crate::model::shows(l, v)).collect();
            // (A tab can come from a `|markup` value as well as the template.)
            let hand = text.contains('\t') || text.contains("{*") || (text.contains("|markup") && shown.iter().any(|l| segments(l, v).iter().any(|g| g.tab)));
            if hand {
                for (k, l) in shown.iter().enumerate().take(inner.height as usize) {
                    let r = Rect::new(inner.x, inner.y + k as u16, inner.width, 1);
                    draw_parts(f.buffer_mut(), r, cx.parts(l, v, style), 0);
                }
            } else {
                let lines: Vec<Line> = shown.iter().map(|l| Line::from(cx.spans(l, v, style))).collect();
                f.render_widget(Paragraph::new(lines).style(style).alignment(align).wrap(Wrap { trim: false }), inner);
            }
        }
        Kind::Editor { .. } | Kind::Input { .. } => {
            cx.sizes.push((id.clone(), inner.width, inner.height));
            let field = n.field().expect("a text field");
            let shown = field_view(st, &id).and_then(|(key, view)| match st.store.get(&key).map(|e| &e.body) {
                Some(crate::store::Body::Text { doc }) => Some((&**doc, view)),
                _ => None,
            });
            let Some((doc, view)) = shown else {
                // A hosted entry on its way, or one that couldn't load.
                let key = crate::state::doc_key(st, n);
                match key.as_ref().and_then(|k| st.loading.get(k).map(|l| (k, l))) {
                    Some((k, Some(why))) => {
                        let p = Paragraph::new(format!("{k}: {why}")).style(Style::default().fg(cx.th.neg));
                        return f.render_widget(p.wrap(Wrap { trim: true }), inner);
                    }
                    Some((k, None)) => {
                        let g = spin(cx);
                        return f.render_widget(Paragraph::new(format!("{g} {k}…")).style(Style::default().fg(cx.th.dim)), inner);
                    }
                    None => {}
                }
                let e = field.placeholder.unwrap_or("Nothing to show");
                return f.render_widget(Paragraph::new(e).style(cx.look("empty", Style::default().fg(cx.th.dim))), inner);
            };
            let looks = FieldLook {
                text: style,
                caret: cx.look("editor.caret", style.add_modifier(Modifier::REVERSED)),
                selection: cx.look("editor.selection", style.add_modifier(Modifier::REVERSED)),
                placeholder: cx.look("input.placeholder", style.fg(cx.th.dim).add_modifier(Modifier::ITALIC)),
                markdown: field.markdown.then(|| Markdown {
                    heading: cx.look("md.heading", style.fg(cx.th.accent).add_modifier(Modifier::BOLD)),
                    bullet: cx.look("md.bullet", style.fg(cx.th.accent)),
                    quote: cx.look("md.quote", style.fg(cx.th.dim).add_modifier(Modifier::ITALIC)),
                    code: cx.look("md.code", style.fg(cx.th.pos)),
                    task: cx.look("md.task", style.fg(cx.th.accent).add_modifier(Modifier::BOLD)),
                    done: cx.look("md.done", style.fg(cx.th.dim).add_modifier(Modifier::CROSSED_OUT)),
                    link: cx.look("md.link", style.fg(cx.th.accent).add_modifier(Modifier::UNDERLINED)),
                    links: field.links.iter().map(|l| l.kind.clone()).collect(),
                }),
            };
            // An input is one row, in the middle of what it's given.
            let area = if field.line && inner.height > 1 { Rect::new(inner.x, inner.y + (inner.height - 1) / 2, inner.width, 1) } else { inner };
            cx.hits.push(Hit { rect: cells(area), id: id.clone(), row: None, tab: None, text: true });
            editor(f.buffer_mut(), area, doc, view, &looks, field.placeholder, focused);
        }
        Kind::Rule { glyph } => {
            let g = glyph.clone().unwrap_or_else(|| if inner.height > inner.width { "│".into() } else { "─".into() });
            let ls = if style.fg.is_some() { style } else { style.fg(cx.th.border) };
            for y in inner.y..inner.bottom() {
                f.buffer_mut().set_string(inner.x, y, g.repeat(inner.width as usize), ls);
            }
        }
        Kind::List { rows: look, .. } | Kind::Tree { rows: look, .. } => {
            // A tree's rows start with their indent and a fold glyph (or `leaf`).
            let leaf = match &n.kind {
                Kind::Tree { leaf, .. } => Some(leaf.as_deref().unwrap_or("· ")),
                _ => None,
            };
            let rows = items(st, n);
            if rows.is_empty() {
                let e = look.empty.clone().unwrap_or_else(|| "(nothing)".into());
                let es = cx.look("empty", Style::default().fg(cx.th.dim));
                return f.render_widget(Paragraph::new(e).style(es).wrap(Wrap { trim: true }), inner);
            }
            let per = look.lines.unwrap_or(1).max(1) as usize;
            let fit = (inner.height as usize / per).max(1);
            let sel = n.id.as_ref().map(|id| selected(st, id));
            let from = crate::state::first_row(n.id.as_ref().and_then(|i| st.local.get(i)), rows.len(), fit);
            if !id.is_empty() {
                cx.sizes.push((id.clone(), inner.width, fit as u16));
            }
            scrollbar(cx, f.buffer_mut(), Rect { height: (fit * per).min(inner.height as usize) as u16, ..inner }, from * per, rows.len() * per);
            let over = (rows.len() > fit) as u16;
            let guide = cx.look("tree.guide", style.fg(cx.th.dim));
            let (mark, lead) = match leaf {
                Some(_) => (look.mark.as_deref().unwrap_or(""), look.lead.as_deref().unwrap_or("")),
                None => (look.mark.as_deref().unwrap_or("▸ "), look.lead.as_deref().unwrap_or("  ")),
            };
            let keep = look.keep.unwrap_or(0);
            for (k, r) in rows.iter().enumerate().skip(from).take(fit) {
                let y = inner.y + ((k - from) * per) as u16;
                if y >= inner.bottom() {
                    break;
                }
                let block = Rect::new(inner.x, y, inner.width.saturating_sub(over), (per as u16).min(inner.bottom() - y));
                let mut rs = style;
                if cx.hovered(&id, k) {
                    rs = rs.bg(cx.th.hover);
                }
                let picked = look.variant.as_ref().and_then(|p| look.variants.get(&scalar(lookup(r, p))));
                let is_sel = sel == Some(k) && !n.skips(r);
                if is_sel {
                    rs = match &look.selected {
                        Some(spec) => cx.spec(spec, rs),
                        None if focused => rs.patch(cx.look("row.selected", Style::default().add_modifier(Modifier::REVERSED))),
                        None => rs.patch(cx.look("row.selected.blur", Style::default().add_modifier(Modifier::BOLD))),
                    };
                }
                f.buffer_mut().set_style(block, rs);
                let mut first = Vec::new();
                if picked.is_none() || !n.skips(r) {
                    first.extend(cx.parts(if is_sel { mark } else { lead }, r, rs));
                }
                if let Some(leaf) = leaf {
                    let depth = r["$depth"].as_u64().unwrap_or(0) as usize;
                    let glyph = match (r["$kids"].as_u64().unwrap_or(0) > 0, r["$open"].as_bool().unwrap_or(false)) {
                        (true, true) => "▾ ",
                        (true, false) => "▸ ",
                        _ => leaf,
                    };
                    let gs = if is_sel { rs } else { guide.patch(Style { bg: rs.bg, ..Style::default() }) };
                    first.push(Part::Text(Span::styled(format!("{}{glyph}", "  ".repeat(depth)), gs)));
                }
                // Lines after the first line up under the first's text.
                let indent: usize = first.iter().map(|p| if let Part::Text(s) = p { s.width() } else { 0 }).sum();
                let template = picked.or(look.item.as_ref());
                let plain = match template {
                    None if leaf.is_some() => scalar(lookup(r, "text").or(Some(r))),
                    None => scalar(Some(r)),
                    Some(_) => String::new(),
                };
                let lines: Vec<&str> = match template {
                    Some(t) => t.split('\n').collect(),
                    None => vec![plain.as_str()],
                };
                for (i, line) in lines.iter().enumerate().take(block.height as usize) {
                    let mut parts = if i == 0 { std::mem::take(&mut first) } else { vec![Part::Text(Span::styled(" ".repeat(indent), rs))] };
                    match template {
                        Some(_) => parts.extend(cx.parts(line, r, rs)),
                        None => parts.push(Part::Text(Span::styled(line.to_string(), rs))),
                    }
                    draw_parts(f.buffer_mut(), Rect::new(block.x, block.y + i as u16, block.width, 1), parts, keep);
                }
                if !id.is_empty() {
                    cx.mark(&id, Some(k), None, block);
                }
            }
        }
        Kind::Table { columns, .. } => {
            let rows = items(st, n);
            let widths: Vec<Constraint> = columns.iter().map(|c| constraint(c.size.as_ref(), st.caps.cell_w)).collect();
            if inner.height == 0 {
                return;
            }
            let totals = columns.iter().any(|c| c.total.is_some());
            let visible = (inner.height as usize).saturating_sub(1 + totals as usize);
            let over = (rows.len() > visible) as u16;
            // Column positions once; each row reuses them at its own y.
            let spans = Layout::horizontal(widths).spacing(1).split(Rect::new(inner.x + 2, inner.y, inner.width.saturating_sub(2 + over), 1));
            let cols = |y: u16| spans.iter().map(move |a| Rect { y, ..*a });
            let head = cx.look("table.header", Style::default().fg(cx.th.accent).add_modifier(Modifier::BOLD));
            for (c, a) in columns.iter().zip(cols(inner.y)) {
                let al = if c.align.as_deref() == Some("right") { Alignment::Right } else { Alignment::Left };
                f.render_widget(Paragraph::new(c.title.clone()).style(head).alignment(al), a);
            }
            let sel = n.id.as_ref().map(|id| selected(st, id));
            let from = crate::state::first_row(n.id.as_ref().and_then(|i| st.local.get(i)), rows.len(), visible);
            if !id.is_empty() {
                cx.sizes.push((id.clone(), inner.width, visible as u16));
            }
            scrollbar(cx, f.buffer_mut(), Rect::new(inner.x, inner.y + 1, inner.width, visible as u16), from, rows.len());
            for (k, r) in rows.iter().enumerate().skip(from).take(visible) {
                let y = inner.y + 1 + (k - from) as u16;
                let line = Rect::new(inner.x, y, inner.width.saturating_sub(over), 1);
                let mut rs = style;
                if cx.hovered(&id, k) {
                    rs = rs.bg(cx.th.hover);
                }
                if sel == Some(k) {
                    rs = if focused {
                        rs.patch(cx.look("row.selected", Style::default().bg(cx.th.accent).fg(Color::Black)))
                    } else {
                        rs.patch(cx.look("row.selected.blur", Style::default().add_modifier(Modifier::BOLD)))
                    };
                }
                f.buffer_mut().set_style(line, rs);
                if sel == Some(k) {
                    f.buffer_mut().set_string(inner.x, y, "▸", rs);
                }
                for (c, a) in columns.iter().zip(cols(y)) {
                    let mut cs = rs;
                    if let Some(h) = &c.heat
                        && let Some(v) = number(&fill(&h.value, r))
                    {
                        cs = cs.bg(heat(v, h.min, h.max, cx.th.neg, cx.th.pos, cx.th.panel)).fg(Color::White);
                        f.buffer_mut().set_style(a, cs);
                    }
                    let al = if c.align.as_deref() == Some("right") { Alignment::Right } else { Alignment::Left };
                    let line = Line::from(cx.spans(&c.value, r, cs)).alignment(al);
                    f.buffer_mut().set_line(line_x(&line, a), y, &line, a.width);
                }
                if !id.is_empty() {
                    cx.mark(&id, Some(k), None, line);
                }
            }
            if totals && inner.height >= 3 {
                // The footer: each total under its column, on the last row, over a rule.
                let y = inner.bottom() - 1;
                let ts = cx.look("table.total", style.add_modifier(Modifier::BOLD));
                let rule = cx.look("border", Style::default().fg(cx.th.border));
                f.buffer_mut().set_string(inner.x, y, " ".repeat(inner.width as usize), ts);
                for (c, a) in columns.iter().zip(cols(y)) {
                    let Some(path) = &c.total else { continue };
                    let sum: f64 = rows.iter().filter_map(|r| lookup(r, path)).filter_map(|v| v.as_f64().or_else(|| v.as_str().and_then(number))).sum();
                    let text = money(sum);
                    let al = if c.align.as_deref() == Some("right") { Alignment::Right } else { Alignment::Left };
                    let mut cs = ts;
                    if sum < 0.0 {
                        cs = cs.fg(cx.th.neg);
                    }
                    let line = Line::from(Span::styled(text, cs)).alignment(al);
                    f.buffer_mut().set_line(line_x(&line, a), y, &line, a.width);
                    if y > inner.y + 1 {
                        f.buffer_mut().set_string(a.x, y - 1, "─".repeat(a.width as usize), rule);
                    }
                }
                if columns.first().is_some_and(|c| c.total.is_none()) {
                    f.buffer_mut().set_string(inner.x + 2, y, "Total", ts);
                }
            }
        }
        Kind::Sparkline { values, field } => {
            let nums = numbers(values, bound(st, n), field.as_deref());
            let (lo, hi) = nums.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let span = if hi > lo { hi - lo } else { 1.0 };
            let data: Vec<u64> = nums.iter().map(|x| (((x - lo) / span) * 100.0).round() as u64 + 1).collect();
            let skip = data.len().saturating_sub(inner.width as usize);
            f.render_widget(Sparkline::default().data(&data[skip..]).style(style), inner);
        }
        Kind::Chart { series, labels } => {
            let data: Vec<Vec<(f64, f64)>> = series
                .iter()
                .map(|s| {
                    numbers(&[], value_of(st, &s.bind), s.field.as_deref()).into_iter().enumerate().map(|(i, y)| (i as f64, y)).collect()
                })
                .collect();
            let all = data.iter().flatten();
            let (lo, hi) = all.clone().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
            let xmax = data.iter().map(|d| d.len()).max().unwrap_or(1).saturating_sub(1).max(1) as f64;
            if lo > hi {
                return;
            }
            let pad = ((hi - lo) * 0.08).max(1e-9);
            let (lo, hi) = (lo - pad, hi + pad);
            let sets: Vec<Dataset> = series
                .iter()
                .zip(data.iter())
                .map(|(s, d)| {
                    let c = s.color.as_deref().and_then(|c| cx.color(c)).unwrap_or(cx.th.accent);
                    Dataset::default().name(s.name.clone()).marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(c)).data(d)
                })
                .collect();
            let axis = Style::default().fg(cx.th.dim);
            let ylabels = vec![compact(lo), compact((lo + hi) / 2.0), compact(hi)];
            let chart = Chart::new(sets)
                .style(style)
                .legend_position(if series.len() > 1 { Some(LegendPosition::TopLeft) } else { None })
                .x_axis(Axis::default().style(axis).bounds([0.0, xmax]).labels(labels.clone()))
                .y_axis(Axis::default().style(axis).bounds([lo, hi]).labels(ylabels));
            f.render_widget(chart, inner);
        }
        Kind::Bars { label, value, horizontal: true, .. } => across(cx, n, f, inner, label, value, style, &id),
        Kind::Bars { label, value, width, .. } => bars(cx, n, f, inner, label, value, *width, style, &id),
        Kind::Gauge { value, max, label } => {
            let v = bound(st, n).unwrap_or(&Value::Null);
            let x = number(&fill(value, v)).unwrap_or(0.0);
            let ratio = (x / max.unwrap_or(1.0)).clamp(0.0, 1.0);
            let text = label.as_ref().map(|l| fill(l, v)).unwrap_or_else(|| format!("{:.0}%", ratio * 100.0));
            let fg = style.fg.unwrap_or(cx.th.accent);
            let g = Gauge::default()
                .gauge_style(Style::default().fg(fg).bg(cx.th.panel))
                .ratio(ratio)
                .use_unicode(true)
                .label(Span::styled(text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
            f.render_widget(g, inner);
        }
        Kind::Big { text } => {
            let v = bound(st, n).unwrap_or(&Value::Null);
            let segs = segments(text, v);
            let sign = segs.iter().find_map(|g| g.sign);
            let s: String = segs.into_iter().map(|g| g.text).collect();
            let mut bs = style;
            if bs.fg.is_none() {
                bs = bs.fg(match sign {
                    Some(true) => cx.th.pos,
                    Some(false) => cx.th.neg,
                    None => cx.th.fg.unwrap_or(Color::White),
                });
            }
            let lines: Vec<Line> = big(&s).into_iter().map(|l| Line::styled(l, bs)).collect();
            let top = inner.y + inner.height.saturating_sub(3) / 2;
            let at = Rect::new(inner.x, top, inner.width, 3.min(inner.height));
            f.render_widget(Paragraph::new(lines).alignment(align), at);
        }
        Kind::Tabs { tabs, children, lead, trail, .. } => {
            let shown = tab_of(st, n).min(children.len().saturating_sub(1));
            let v = bound(st, n).unwrap_or(&Value::Null);
            let bar = Rect { height: 1.min(inner.height), ..inner };
            // The trail right-aligned, the lead first, the titles between.
            let mut end = inner.right();
            if let Some(t) = trail {
                let parts = cx.parts(t, v, style);
                let w = parts.iter().map(|p| if let Part::Text(s) = p { s.width() } else { 0 }).sum::<usize>() as u16;
                draw_parts(f.buffer_mut(), bar, std::iter::once(Part::Tab).chain(parts).collect(), 0);
                end = end.saturating_sub(w + 1);
            }
            let mut x = inner.x;
            if let Some(l) = lead {
                let parts = cx.parts(l, v, style);
                let w = parts.iter().map(|p| if let Part::Text(s) = p { s.width() } else { 0 }).sum::<usize>() as u16;
                draw_parts(f.buffer_mut(), Rect { width: w.min(bar.width), ..bar }, parts, 0);
                x += w + 1;
            }
            let mut active = None;
            for (k, t) in tabs.iter().enumerate() {
                // A title is a template, markup and all, padded a cell either side.
                let w = cx.parts(t, v, Style::default()).iter().map(|p| if let Part::Text(s) = p { s.width() } else { 0 }).sum::<usize>() as u16 + 2;
                if x + w > end {
                    break;
                }
                let r = Rect::new(x, inner.y, w, 1);
                let ts = if k == shown {
                    cx.look("tabs.selected", Style::default().fg(Color::Black).bg(cx.th.accent).add_modifier(Modifier::BOLD))
                } else if cx.st.hover.as_ref().is_some_and(|h| h.id == id && h.row == Some(1000 + k)) {
                    Style::default().fg(cx.th.accent).bg(cx.th.hover)
                } else {
                    cx.look("tabs.idle", Style::default().fg(cx.th.dim))
                };
                f.buffer_mut().set_style(r, ts);
                draw_parts(f.buffer_mut(), Rect { x: r.x + 1, width: w - 1, ..r }, cx.parts(t, v, ts), 0);
                if !id.is_empty() {
                    cx.hits.push(Hit { rect: cells(r), id: id.clone(), row: Some(1000 + k), tab: Some(k), text: false });
                }
                if k == shown {
                    active = Some(r);
                }
                x += w + 1;
            }
            let rule = Rect::new(inner.x, inner.y + 1, inner.width, 1);
            f.buffer_mut().set_string(rule.x, rule.y, "─".repeat(inner.width as usize), cx.look("border", Style::default().fg(cx.th.border)));
            // The active tab's label underlined on the rule.
            if let Some(r) = active {
                let u = cx.look("tabs.underline", Style::default().fg(cx.th.accent));
                f.buffer_mut().set_string(r.x, rule.y, "━".repeat(r.width as usize), u);
            }
            if let Some(c) = children.get(shown) {
                let body = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(2));
                node(cx, c, f, body);
            }
        }
        Kind::Plot { series, fill, width } => {
            let data: Vec<(Vec<f64>, Rgb)> = series
                .iter()
                .map(|s| {
                    let c = s.color.as_deref().and_then(|c| cx.rgb(c)).unwrap_or((0x5f, 0xb3, 0xff));
                    (numbers(&[], value_of(st, &s.bind), s.field.as_deref()), c)
                })
                .collect();
            if pixels {
                let pic = Pic::Plot { series: data, fill: *fill, width: width.unwrap_or(0.12) };
                cx.pics.push(Placed { x: inner.x, y: inner.y, w: inner.width, h: inner.height, z: 0, pic });
            } else {
                // Cells only: the same lines in braille.
                let pts: Vec<Vec<(f64, f64)>> = data.iter().map(|(v, _)| v.iter().enumerate().map(|(i, y)| (i as f64, *y)).collect()).collect();
                let (lo, hi) = pts.iter().flatten().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
                let xmax = pts.iter().map(|p| p.len()).max().unwrap_or(1).max(2) as f64 - 1.0;
                let sets: Vec<Dataset> = pts
                    .iter()
                    .zip(data.iter())
                    .map(|(p, (_, c))| Dataset::default().marker(Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Rgb(c.0, c.1, c.2))).data(p))
                    .collect();
                if lo <= hi {
                    f.render_widget(Chart::new(sets).x_axis(Axis::default().bounds([0.0, xmax])).y_axis(Axis::default().bounds([lo, hi])), inner);
                }
            }
        }
        Kind::Image { .. } => {
            let src = crate::state::image_src(st, n).unwrap_or_default();
            if pixels && !src.is_empty() {
                cx.pics.push(Placed { x: inner.x, y: inner.y, w: inner.width, h: inner.height, z: 0, pic: Pic::File(src) });
            } else {
                f.render_widget(Paragraph::new(format!("[image {src}]")).style(Style::default().fg(cx.th.dim)), inner);
            }
        }
        Kind::Svg { svg, src, alt } => {
            let markup = match (svg, src) {
                (Some(t), _) => Some(crate::model::fill_raw(t, bound(st, n).unwrap_or(&Value::Null))),
                (None, Some(p)) => Some(format!("file:{p}")),
                _ => None,
            };
            match markup {
                Some(m) if pixels => cx.pics.push(Placed { x: inner.x, y: inner.y, w: inner.width, h: inner.height, z: 0, pic: Pic::Svg(m) }),
                _ => {
                    let a = alt.as_deref().map(|a| fill(a, bound(st, n).unwrap_or(&Value::Null))).unwrap_or_else(|| "[svg]".into());
                    f.render_widget(Paragraph::new(a).style(Style::default().fg(cx.th.dim)).alignment(align), inner);
                }
            }
        }
        Kind::Dot { color, glow } => {
            let c = cx.rgb(color).unwrap_or((0xe8, 0x83, 0x4f));
            if pixels {
                cx.pics.push(Placed { x: inner.x, y: inner.y, w: inner.width, h: inner.height, z: 0, pic: Pic::Dot { color: c, glow: *glow } });
            } else {
                let at = Rect::new(inner.x + inner.width / 2, inner.y + inner.height / 2, 1, 1);
                f.buffer_mut().set_string(at.x, at.y, "•", Style::default().fg(Color::Rgb(c.0, c.1, c.2)).add_modifier(Modifier::BOLD));
            }
        }
        Kind::Map { lat, lon, label } => {
            let rows = match bound(st, n) {
                Some(Value::Array(a)) => a.as_slice(),
                _ => &[],
            };
            let pts: Vec<(f64, f64, String)> = rows
                .iter()
                .map(|r| {
                    let la = lookup(r, lat).and_then(Value::as_f64).unwrap_or(0.0);
                    let lo = lookup(r, lon).and_then(Value::as_f64).unwrap_or(0.0);
                    (lo, la, label.as_ref().map(|l| fill(l, r)).unwrap_or_default())
                })
                .collect();
            let coords: Vec<(f64, f64)> = pts.iter().map(|p| (p.0, p.1)).collect();
            let (land, dot) = (cx.th.border, style.fg.unwrap_or(cx.th.accent));
            let hover_row = cx.st.hover.as_ref().filter(|h| h.id == id).and_then(|h| h.row);
            let canvas = Canvas::default()
                .marker(Marker::Braille)
                .x_bounds([-180.0, 180.0])
                .y_bounds([-60.0, 85.0])
                .paint(|ctx| {
                    ctx.draw(&Map { resolution: MapResolution::High, color: land });
                    ctx.layer();
                    ctx.draw(&Points { coords: &coords, color: dot });
                    for (k, (x, y, l)) in pts.iter().enumerate() {
                        let s = if hover_row == Some(k) { Style::default().fg(Color::Black).bg(dot) } else { Style::default().fg(dot) };
                        ctx.print(*x, *y, Line::styled(format!("●{l}"), s));
                    }
                });
            f.render_widget(canvas, inner);
            for (k, (x, y, _)) in pts.iter().enumerate() {
                let cx_ = inner.x as f64 + (x + 180.0) / 360.0 * (inner.width.saturating_sub(1)) as f64;
                let cy = inner.y as f64 + (85.0 - y) / 145.0 * (inner.height.saturating_sub(1)) as f64;
                if !id.is_empty() {
                    cx.mark(&id, Some(k), None, Rect::new(cx_.round() as u16, cy.round() as u16, 1, 1));
                }
            }
        }
    }
}

/// Where a line starts in its cell, by its alignment.
fn line_x(line: &Line, a: Rect) -> u16 {
    let w = (line.width() as u16).min(a.width);
    match line.alignment {
        Some(Alignment::Right) => a.x + a.width - w,
        Some(Alignment::Center) => a.x + (a.width - w) / 2,
        _ => a.x,
    }
}

/// Bars across: a row per item, the label, a bar in eighths of a cell, the value.
#[allow(clippy::too_many_arguments)]
fn across(cx: &mut Cx, n: &Node, f: &mut Frame, inner: Rect, label: &str, value: &str, style: Style, id: &str) {
    let rows = items(cx.st, n);
    if rows.is_empty() || inner.width < 12 {
        return;
    }
    let vals: Vec<f64> = rows.iter().map(|r| lookup(r, value).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(number))).unwrap_or(0.0)).collect();
    let max = vals.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1e-9);
    let labels: Vec<String> = rows.iter().map(|r| fill(label, r)).collect();
    let lw = labels.iter().map(|l| l.chars().count()).max().unwrap_or(0).min(16) as u16;
    let nums: Vec<String> = vals.iter().map(|v| money(v.round())).collect();
    let nw = nums.iter().map(|s| s.chars().count()).max().unwrap_or(0) as u16;
    let room = inner.width.saturating_sub(lw + nw + 2) as f64;
    let eighths = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    let ls = cx.look("bars.label", Style::default().fg(cx.th.dim));
    for (k, ((l, v), num)) in labels.iter().zip(&vals).zip(&nums).enumerate().take(inner.height as usize) {
        let y = inner.y + k as u16;
        let hovered = cx.hovered(id, k);
        let color = if hovered { cx.th.accent } else if *v < 0.0 { cx.th.neg } else { style.fg.unwrap_or(cx.th.pos) };
        let cells = v.abs() / max * room;
        let bar = format!("{}{}", "█".repeat(cells.floor() as usize), eighths[((cells - cells.floor()) * 8.0).round().min(7.0) as usize]);
        let name: String = l.chars().take(lw as usize).collect();
        f.buffer_mut().set_string(inner.x, y, &name, if hovered { ls.fg(cx.th.accent) } else { ls });
        f.buffer_mut().set_string(inner.x + lw + 1, y, &bar, Style::default().fg(color));
        let nx = inner.x + lw + 1 + bar.chars().count() as u16 + 1;
        f.buffer_mut().set_string(nx.min(inner.right().saturating_sub(nw)), y, num, style.add_modifier(Modifier::BOLD));
        if !id.is_empty() {
            cx.mark(id, Some(k), None, Rect::new(inner.x, y, inner.width, 1));
        }
    }
}

/// Bars over zero: positive up, negative down, eighth blocks at the tips, labels underneath.
#[allow(clippy::too_many_arguments)]
fn bars(cx: &mut Cx, n: &Node, f: &mut Frame, inner: Rect, label: &str, value: &str, width: Option<u16>, style: Style, id: &str) {
    let rows = items(cx.st, n);
    if rows.is_empty() || inner.height < 3 {
        return;
    }
    let vals: Vec<f64> = rows.iter().map(|r| lookup(r, value).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(number))).unwrap_or(0.0)).collect();
    let (lo, hi) = vals.iter().fold((0.0f64, 0.0f64), |(a, b), &x| (a.min(x), b.max(x)));
    let span = (hi - lo).max(1e-9);
    let h = (inner.height - 2) as f64; // bars, then the zero line and the labels below
    let zero = inner.y + ((hi / span) * h).round() as u16;
    let n_ = rows.len() as u16;
    let w = width.unwrap_or(((inner.width / n_.max(1)).saturating_sub(1)).clamp(1, 9));
    let total = n_ * (w + 1);
    let x0 = inner.x + inner.width.saturating_sub(total) / 2;
    let base = style;
    let eighths = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    for (k, (r, v)) in rows.iter().zip(vals.iter()).enumerate() {
        let x = x0 + k as u16 * (w + 1);
        if x + w > inner.right() {
            break;
        }
        let mut color = if *v >= 0.0 { base.fg.unwrap_or(cx.th.pos) } else { cx.th.neg };
        let hovered = cx.hovered(id, k);
        if hovered {
            color = cx.th.accent;
        }
        let cells = (v.abs() / span) * h;
        let full = cells.floor() as u16;
        let part = ((cells - cells.floor()) * 8.0).round() as usize;
        let bar = Style::default().fg(color);
        // Bars stay between the panel's top and the label row.
        let room = |y: u16| y >= inner.y && y + 1 < inner.bottom();
        for i in 0..full {
            let y = if *v >= 0.0 { zero.wrapping_sub(1 + i) } else { zero + 1 + i };
            if room(y) {
                f.buffer_mut().set_string(x, y, "█".repeat(w as usize), bar);
            }
        }
        if part > 0 {
            if *v >= 0.0 && zero > inner.y + full {
                f.buffer_mut().set_string(x, zero - 1 - full, eighths[part].repeat(w as usize), bar);
            } else if *v < 0.0 && part >= 4 && room(zero + 1 + full) {
                f.buffer_mut().set_string(x, zero + 1 + full, "▀".repeat(w as usize), bar);
            }
        }
        let l: String = fill(label, r).chars().take(w as usize).collect();
        let ls = if hovered { Style::default().fg(cx.th.accent).add_modifier(Modifier::BOLD) } else { Style::default().fg(cx.th.dim) };
        f.buffer_mut().set_string(x, inner.bottom() - 1, &l, ls);
        if !id.is_empty() {
            let top = if *v >= 0.0 { zero.saturating_sub(full + 1) } else { zero + 1 };
            cx.mark(id, Some(k), None, Rect::new(x, top.max(inner.y), w, (full + 1).max(1)));
        }
    }
    f.buffer_mut().set_string(inner.x, zero, "┈".repeat(inner.width as usize), Style::default().fg(cx.th.border));
}

/// Numbers from literal `values`, or an array (each item's `field`).
fn numbers(values: &[f64], v: Option<&Value>, field: Option<&str>) -> Vec<f64> {
    if !values.is_empty() {
        return values.to_vec();
    }
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| match field {
                Some(p) => lookup(x, p).and_then(Value::as_f64),
                None => x.as_f64(),
            })
            .collect(),
        _ => vec![],
    }
}

/// `1234567` → `1.23M`.
/// A sum for a footer: thousands separated, two decimals unless it's whole.
fn money(x: f64) -> String {
    let neg = x < 0.0;
    let cents = (x.abs() * 100.0).round() as u64;
    let (whole, frac) = (cents / 100, cents % 100);
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let sign = if neg { "-" } else { "" };
    if frac == 0 { format!("{sign}{grouped}") } else { format!("{sign}{grouped}.{frac:02}") }
}

fn compact(x: f64) -> String {
    let a = x.abs();
    match a {
        a if a >= 1e9 => format!("{:.2}B", x / 1e9),
        a if a >= 1e6 => format!("{:.2}M", x / 1e6),
        a if a >= 1e4 => format!("{:.1}k", x / 1e3),
        a if a >= 100.0 => format!("{x:.0}"),
        _ => format!("{x:.2}"),
    }
}

/// A colour between `neg` (at `min`), `mid` (at zero) and `pos` (at `max`).
fn heat(v: f64, min: f64, max: f64, neg: Color, pos: Color, mid: Color) -> Color {
    if v >= 0.0 {
        mix(mid, pos, (v / max.max(1e-9)).clamp(0.0, 1.0))
    } else {
        mix(mid, neg, (v / min.min(-1e-9)).clamp(0.0, 1.0))
    }
}

fn rgb(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

fn mix(a: Color, b: Color, t: f64) -> Color {
    match (rgb(a), rgb(b)) {
        (Some(a), Some(b)) => {
            let l = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
            Color::Rgb(l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
        }
        _ if t < 0.5 => a,
        _ => b,
    }
}

/// Only the rows that fit are built: the first one shown, and the selection within them. The
/// selection sits on the last visible row once it passes the bottom.
/// A thin bar on the right edge of `r` when `len` rows don't fit: where the shown ones (from
/// `from`) are among them. Drawn first; the rows draw over all but its column.
fn scrollbar(cx: &Cx, buf: &mut Buffer, r: Rect, from: usize, len: usize) {
    let h = r.height as usize;
    if len <= h || h == 0 || r.width < 2 {
        return;
    }
    let size = (h * h / len).max(1);
    let at = from * (h - size) / (len - h).max(1);
    let x = r.right() - 1;
    let track = cx.look("scrollbar", Style::default().fg(cx.th.border));
    let thumb = cx.look("scrollbar.thumb", Style::default().fg(cx.th.accent));
    for y in 0..h {
        let on = y >= at && y < at + size;
        buf.set_string(x, r.y + y as u16, if on { "┃" } else { "│" }, if on { thumb } else { track });
    }
}

/// A child's share with its `min` and `max` (cells) applied, along `along` cells of its parent.
/// A length as whole cells, for the grid: pixels and `space` names are none. The grid isn't
/// extended for lengths in other units; the window has them.
fn len_cells(l: &Len) -> Option<u16> {
    match l {
        Len::Cells(n) => Some(n.max(0.) as u16),
        Len::Spec(_) => None,
    }
}

fn bounded(c: Constraint, min: Option<u16>, max: Option<u16>, along: u16) -> Constraint {
    if min.is_none() && max.is_none() {
        return c;
    }
    let clamp = |n: u16| n.max(min.unwrap_or(0)).min(max.unwrap_or(u16::MAX));
    match c {
        Constraint::Length(n) => Constraint::Length(clamp(n)),
        Constraint::Percentage(p) => Constraint::Length(clamp((along as u32 * p as u32 / 100) as u16)),
        // A share of what's left can't be known here: the bound that matters wins.
        _ => match (min, max) {
            (_, Some(m)) => Constraint::Max(m),
            (Some(m), None) => Constraint::Min(m),
            (None, None) => c,
        },
    }
}

pub fn constraint(s: Option<&Size>, cell: f32) -> Constraint {
    match s.map(Size::measure) {
        None => Constraint::Fill(1),
        Some(Measure::Cells(n)) => Constraint::Length(n),
        Some(Measure::Px(p)) => Constraint::Length(in_cells(p, cell)),
        Some(Measure::Percent(p)) => Constraint::Percentage(p.round().clamp(0.0, 100.0) as u16),
        Some(Measure::Share(w)) => Constraint::Fill(w),
    }
}

/// Pixels as whole cells of `cell` pixels (a cell of 8 × 16 where the terminal didn't say).
fn in_cells(p: f32, cell: f32) -> u16 {
    (p / if cell > 0.0 { cell } else { 8.0 }).round().clamp(0.0, u16::MAX as f32) as u16
}

// ---- big digits ---------------------------------------------------------------------------

/// A glyph: five rows of pixels, drawn as three rows of half blocks.
fn glyph(c: char) -> &'static [&'static str; 5] {
    match c.to_ascii_uppercase() {
        '0' => &["###", "#.#", "#.#", "#.#", "###"],
        '1' => &[".#.", "##.", ".#.", ".#.", "###"],
        '2' => &["###", "..#", "###", "#..", "###"],
        '3' => &["###", "..#", ".##", "..#", "###"],
        '4' => &["#.#", "#.#", "###", "..#", "..#"],
        '5' => &["###", "#..", "###", "..#", "###"],
        '6' => &["###", "#..", "###", "#.#", "###"],
        '7' => &["###", "..#", "..#", ".#.", ".#."],
        '8' => &["###", "#.#", "###", "#.#", "###"],
        '9' => &["###", "#.#", "###", "..#", "###"],
        '$' => &[".##", "##.", ".#.", ".##", "##."],
        '.' => &[".", ".", ".", ".", "#"],
        ',' => &[".", ".", ".", "#", "#"],
        '-' => &["...", "...", "###", "...", "..."],
        '+' => &["...", ".#.", "###", ".#.", "..."],
        '%' => &["#.#", "..#", ".#.", "#..", "#.#"],
        'K' => &["#.#", "#.#", "##.", "#.#", "#.#"],
        'M' => &["#.#", "###", "###", "#.#", "#.#"],
        'B' => &["##.", "#.#", "##.", "#.#", "##."],
        'X' => &["...", "#.#", ".#.", "#.#", "..."],
        'D' => &["##.", "#.#", "#.#", "#.#", "##."],
        'Y' => &["#.#", "#.#", ".#.", ".#.", ".#."],
        ':' => &[".", "#", ".", "#", "."],
        _ => &["..", "..", "..", "..", ".."],
    }
}

/// Text in the big font: three lines.
pub fn big(s: &str) -> [String; 3] {
    let mut out = [String::new(), String::new(), String::new()];
    for (i, c) in s.chars().enumerate() {
        let g = glyph(c);
        let w = g[0].chars().count();
        if i > 0 {
            for l in out.iter_mut() {
                l.push(' ');
            }
        }
        for (row, l) in out.iter_mut().enumerate() {
            let top: Vec<char> = g[row * 2].chars().collect();
            let bot: Vec<char> = g.get(row * 2 + 1).map(|r| r.chars().collect()).unwrap_or_else(|| vec!['.'; w]);
            for x in 0..w {
                l.push(match (top[x] == '#', bot[x] == '#') {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    _ => ' ',
                });
            }
        }
    }
    out
}

// ---- layers ---------------------------------------------------------------------------------

/// The `callout` renderer: measures a box of a title and wrapped text.
struct Callout;

fn wrap(s: &str, w: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for para in s.split('\n') {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > w {
                lines.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
        lines.push(cur);
    }
    lines
}

fn callout_lines(data: &Value, inner: usize) -> Vec<String> {
    let mut lines = wrap(data["text"].as_str().unwrap_or(""), inner);
    if let Some(t) = data["title"].as_str() {
        lines.insert(0, t.to_string());
    }
    lines
}

impl cl::Renderer for Callout {
    fn measure(&self, cx: &cl::MeasureCtx) -> cl::Size {
        let inner = cx.avail.w.saturating_sub(4).max(8) as usize;
        let lines = callout_lines(cx.data, inner);
        let label = cx.owner.actor().map_or(0, |a| a.chars().count() + 5);
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0).max(label).min(inner);
        cl::Size::new(w as u16 + 4, lines.len() as u16 + 2)
    }
}

/// The layers this frame shows: the UI's, plus the tip of whatever row the mouse is over.
fn specs(st: &State) -> Vec<LayerSpec> {
    let mut out: Vec<LayerSpec> = st.ui.layers.iter().filter(|l| l.when.as_deref().is_none_or(|w| crate::state::holds(st, w))).cloned().collect();
    if let Some(h) = &st.hover
        && let Some(row) = h.row.filter(|r| *r < 1000)
        && let Some(n) = st.tree().find(&h.id)
        && let Some(tip) = &n.tip
    {
        let rows = items(st, n);
        let rows = if rows.is_empty() {
            match bound(st, n) {
                Some(Value::Array(a)) => a.as_slice(),
                _ => &[],
            }
        } else {
            rows
        };
        if let Some(r) = rows.get(row) {
            out.push(LayerSpec {
                on: h.id.clone(),
                row: Some(row),
                title: tip.title.as_ref().map(|t| fill(t, r)),
                text: Some(fill(&tip.text, r)),
                arrow: true,
                place: vec!["right".into(), "left".into(), "below".into(), "above".into()],
                width: Some(44),
                look: Some(Look { edge: Some("plain".into()), ..Look::default() }),
                ..LayerSpec::default()
            });
        }
    }
    out
}

fn layers(cx: &Cx, buf: &mut Buffer, body: Rect) -> Vec<Placed> {
    let st = cx.st;
    let specs = specs(st);
    if specs.is_empty() {
        return vec![];
    }
    let mut ls = cl::Layers::default();
    for (i, s) in specs.iter().enumerate() {
        let (kind, key) = match s.row {
            Some(r) => ("row", format!("{}#{}", s.on, r)),
            None => ("node", s.on.clone()),
        };
        let mut l = cl::Layer::new(cl::Anchor::Host { kind: kind.into(), key });
        l.id = format!("s{i}");
        let v = s.bind.as_deref().and_then(|b| value_of(st, b)).unwrap_or(&Value::Null);
        if s.title.is_some() || s.text.is_some() {
            let data = json!({"title": s.title.as_ref().map(|t| fill(t, v)), "text": s.text.as_ref().map(|t| fill(t, v)).unwrap_or_default()});
            l = l.with_content(cl::Content::new("callout", data));
        }
        l.arrow = s.arrow;
        if s.ring || s.pulse.is_some() {
            l.ring = Some(cl::Ring { pulse: s.pulse.map(|p| cl::Pulse { period_ms: p, cycles: 0 }) });
        }
        if s.spotlight {
            l.spotlight = Some(cl::Spotlight::default());
        }
        l.place = s
            .place
            .iter()
            .filter_map(|p| match p.as_str() {
                "below" => Some(cl::Side::Below),
                "above" => Some(cl::Side::Above),
                "right" => Some(cl::Side::Right),
                "left" => Some(cl::Side::Left),
                _ => None,
            })
            .collect();
        l.max_width = s.width;
        let _ = cl::apply(&mut ls, cl::LayerOp::Push(l), s.by.as_deref(), 0, &cl::Limits::default());
    }
    let grid = cl::Grid::new(buf.area.width, buf.area.height).with_area(cl::Rect::new(body.x, body.y, body.width, body.height));
    let renderers = cl::Renderers::new().register("callout", Callout);
    let plan = cl::plan(&ls, &cx.anchors, &grid, &renderers);

    // Spotlights: with pictures, one translucent sheet over everything with soft holes; in cells,
    // the text outside the holes fades.
    let mut pics = Vec::new();
    if !plan.spots.is_empty() && st.caps.graphics {
        let holes = plan.spots.iter().flat_map(|s| s.holes.iter().map(|h| (h.x - body.x, h.y - body.y, h.w, h.h))).collect();
        pics.push(Placed { x: body.x, y: body.y, w: body.width, h: body.height, z: 10, pic: Pic::Shade { holes, alpha: 0.62 } });
    } else if !plan.spots.is_empty() {
        for y in body.y..body.bottom() {
            for x in body.x..body.right() {
                if plan.dimmed(x, y) {
                    let c = &mut buf[(x, y)];
                    c.set_style(Style::default().fg(cx.th.dim).remove_modifier(Modifier::BOLD | Modifier::REVERSED));
                }
            }
        }
    }
    for p in &plan.layers {
        let i: usize = p.id.trim_start_matches('s').parse().unwrap_or(0);
        let s = &specs[i];
        let look = s.look.clone().unwrap_or_default();
        let c = |n: &Option<String>, d: Color| n.as_deref().and_then(|x| cx.color(x)).unwrap_or(d);
        let accent = c(&look.accent, cx.th.accent);
        let fill_bg = c(&look.fill, cx.th.panel);
        let fg = c(&look.fg, cx.th.fg.unwrap_or(Color::Rgb(0xd8, 0xdc, 0xe6)));
        let mut ring = c(&look.ring, cx.th.ring);
        if let Some(period) = s.pulse.filter(|p| *p > 0) {
            let t = (st.now_ms % period as u64) as f64 / period as f64;
            ring = mix(ring, accent, (1.0 - (t * std::f64::consts::TAU).cos()) / 2.0);
        }
        let g = Glyphs::of(look.edge.as_deref());
        for r in &p.ring {
            for x in r.x..r.x + r.w {
                for y in r.y..r.y + r.h {
                    if x < buf.area.width && y < buf.area.height {
                        buf[(x, y)].set_bg(ring);
                    }
                }
            }
        }
        let line = Style::default().fg(accent);
        if let Some(rt) = &p.route {
            let n = rt.steps.len();
            for (k, st_) in rt.steps.iter().enumerate() {
                let sym = if k + 1 == n {
                    match st_.leave {
                        cl::Dir::Up => "▲",
                        cl::Dir::Down => "▼",
                        cl::Dir::Left => "◀",
                        cl::Dir::Right => "▶",
                    }
                } else if st_.enter == st_.leave {
                    if matches!(st_.enter, cl::Dir::Up | cl::Dir::Down) { g.v } else { g.h }
                } else {
                    match (st_.enter, st_.leave) {
                        (cl::Dir::Right, cl::Dir::Down) | (cl::Dir::Up, cl::Dir::Left) => g.tr,
                        (cl::Dir::Left, cl::Dir::Down) | (cl::Dir::Up, cl::Dir::Right) => g.tl,
                        (cl::Dir::Right, cl::Dir::Up) | (cl::Dir::Down, cl::Dir::Left) => g.br,
                        _ => g.bl,
                    }
                };
                put(buf, st_.x, st_.y, sym, line);
            }
        }
        let (Some(r), Some(content)) = (p.rect, ls.get(&p.id).and_then(|l| l.content.as_ref())) else { continue };
        let fill_s = Style::default().fg(fg).bg(fill_bg);
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                put(buf, x, y, " ", fill_s);
            }
        }
        let lines = callout_lines(&content.data, r.w.saturating_sub(4) as usize);
        if p.mode == Some(cl::Mode::Strip) {
            buf.set_stringn(r.x + 1, r.y, lines.join(" "), r.w as usize - 1, fill_s);
            continue;
        }
        let border = Style::default().fg(accent).bg(fill_bg);
        let (x1, y1) = (r.x + r.w - 1, r.y + r.h - 1);
        for x in r.x + 1..x1 {
            put(buf, x, r.y, g.h, border);
            put(buf, x, y1, g.h, border);
        }
        for y in r.y + 1..y1 {
            put(buf, r.x, y, g.v, border);
            put(buf, x1, y, g.v, border);
        }
        put(buf, r.x, r.y, g.tl, border);
        put(buf, x1, r.y, g.tr, border);
        put(buf, r.x, y1, g.bl, border);
        put(buf, x1, y1, g.br, border);
        if let Some(actor) = p.owner.actor() {
            let label = format!(" ◆ {actor} ");
            let n = label.chars().count() as u16;
            let mut at = 2;
            if let Some(a) = p.route.as_ref().map(|rt| rt.attach)
                && a.edge == cl::Edge::Top
                && (at..at + n).contains(&a.offset)
            {
                at = a.offset + 2;
            }
            if at + n < r.w {
                buf.set_string(r.x + at, r.y, label, border.add_modifier(Modifier::ITALIC));
            }
        }
        if let (Some(rt), Some(side)) = (&p.route, p.side) {
            let j = match side {
                cl::Side::Above => g.jt,
                cl::Side::Below => g.jb,
                cl::Side::Right => g.jr,
                cl::Side::Left => g.jl,
            };
            put(buf, rt.junction.0, rt.junction.1, j, border);
        }
        let has_title = content.data["title"].is_string();
        for (k, l) in lines.iter().enumerate().take(r.h as usize - 2) {
            let s = if k == 0 && has_title { border.add_modifier(Modifier::BOLD) } else { fill_s };
            buf.set_line(r.x + 2, r.y + 1 + k as u16, &Line::from(Span::styled(l.clone(), s)), r.w - 4);
        }
    }
    pics
}

fn put(buf: &mut Buffer, x: u16, y: u16, s: &str, style: Style) {
    if x < buf.area.width && y < buf.area.height {
        buf[(x, y)].set_symbol(s).set_style(style);
    }
}

/// Box and arrow glyphs for an edge style.
struct Glyphs {
    h: &'static str,
    v: &'static str,
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    /// Junctions for an arrow leaving a box placed above, below, left of or right of its anchor
    /// (`┬`, `┴`, `├`, `┤` in the plain set).
    jt: &'static str,
    jb: &'static str,
    jl: &'static str,
    jr: &'static str,
}

impl Glyphs {
    fn of(edge: Option<&str>) -> Glyphs {
        match edge {
            Some("double") => Glyphs { h: "═", v: "║", tl: "╔", tr: "╗", bl: "╚", br: "╝", jt: "╦", jb: "╩", jl: "╠", jr: "╣" },
            Some("thick") => Glyphs { h: "━", v: "┃", tl: "┏", tr: "┓", bl: "┗", br: "┛", jt: "┳", jb: "┻", jl: "┣", jr: "┫" },
            Some("plain") => Glyphs { h: "─", v: "│", tl: "┌", tr: "┐", bl: "└", br: "┘", jt: "┬", jb: "┴", jl: "├", jr: "┤" },
            _ => Glyphs { h: "─", v: "│", tl: "╭", tr: "╮", bl: "╰", br: "╯", jt: "┬", jb: "┴", jl: "├", jr: "┤" },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_character_drawn_over_gives_way() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
        buf.set_string(0, 0, "漢字", Style::default());
        // Something drawn over 字's second cell, and 漢 left alone.
        buf.set_string(3, 0, "╭", Style::default());
        whole_wide(&mut buf);
        let row: Vec<&str> = (0..6).map(|x| buf[(x, 0)].symbol()).collect();
        assert_eq!(row, ["漢", " ", " ", "╭", " ", " "]);
        // Cut by the right edge.
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        buf[(2, 0)].set_symbol("字");
        whole_wide(&mut buf);
        assert_eq!(buf[(2, 0)].symbol(), " ");
    }

    #[test]
    fn big_digits_are_three_rows() {
        let [a, b, c] = big("1.5");
        assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("▄█    █▀▀", " █    ▀▀█", "▀▀▀ ▀ ▀▀▀"));
    }

    fn screen(ui: serde_json::Value, w: u16, h: u16) -> Vec<String> {
        use crate::state::{Msg, update};
        let mut st = State::default();
        update(&mut st, Msg::Push { ui: serde_json::from_value(ui).unwrap() });
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
        term.draw(|f| {
            draw(&st, f);
        })
        .unwrap();
        let buf = term.backend().buffer();
        (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>().trim_end().to_string()).collect()
    }

    #[test]
    fn a_table_totals_its_columns() {
        let rows = screen(
            serde_json::json!({
                "root": {"type": "table", "id": "t", "bind": "ledger", "columns": [
                    {"title": "payee", "value": "{payee}"},
                    {"title": "amount", "value": "{amount}", "align": "right", "size": 10, "total": "amount"}
                ]},
                "store": {"ledger": [{"payee": "Rent", "amount": -1450}, {"payee": "Pay", "amount": "3200.5"}, {"payee": "Tea", "amount": "-4.25"}]},
                "status_bar": false
            }),
            30,
            7,
        );
        assert_eq!(rows[6], "  Total               1,746.25", "{rows:#?}");
        assert!(rows[5].ends_with("──────────"), "{rows:#?}");
    }

    #[test]
    fn bars_across_label_bar_and_value() {
        let rows = screen(
            serde_json::json!({
                "root": {"type": "bars", "horizontal": true, "label": "{k}", "value": "v",
                         "items": [{"k": "rent", "v": 1450}, {"k": "food", "v": 362.5}]},
                "status_bar": false
            }),
            30,
            2,
        );
        assert_eq!(rows[0], "rent ███████████████████ 1,450");
        assert_eq!(rows[1], "food ████▊ 363", "a quarter of the width: 4.75 cells");
    }

        #[test]
    fn markdown_styles_headings_markers_quotes_and_code() {
        let md = Markdown {
            heading: Style::default().fg(Color::Cyan),
            bullet: Style::default().fg(Color::Blue),
            quote: Style::default().fg(Color::Gray),
            code: Style::default().fg(Color::Green),
            task: Style::default().fg(Color::Yellow),
            done: Style::default().fg(Color::DarkGray),
            link: Style::default().fg(Color::Magenta),
            links: vec!["wiki".into()],
        };
        let base = Style::default();
        assert_eq!(md_style(&md, "see [[Home]] now", 6, base), md.link, "a link");
        assert_eq!(md_style(&md, "- [x] shipped", 3, base), md.task, "the box");
        assert_eq!(md_style(&md, "- [x] shipped", 7, base), md.done, "a ticked task's text");
        assert_eq!(md_style(&md, "- [ ] to do", 7, base), base);
        assert_eq!(md_style(&md, "## Done", 4, base), md.heading);
        assert_eq!(md_style(&md, "  - item", 2, base), md.bullet, "the marker");
        assert_eq!(md_style(&md, "  - item", 4, base), base, "not the item");
        assert_eq!(md_style(&md, "12. step", 2, base), md.bullet);
        assert_eq!(md_style(&md, "> said", 3, base), md.quote);
        assert_eq!(md_style(&md, "run `cargo test` now", 6, base), md.code);
        assert_eq!(md_style(&md, "run `cargo test` now", 4, base), md.code, "the opening tick");
        assert_eq!(md_style(&md, "run `cargo test` now", 17, base), base);
    }

    #[test]
    fn an_editor_draws_its_text_and_reports_its_size() {
        use crate::state::{Msg, update};
        let ui = serde_json::from_value(serde_json::json!({
            "root": {"type": "editor", "id": "note", "title": "Note", "text": "line one\nline two"},
            "status_bar": false
        }))
        .unwrap();
        let mut st = State::default();
        update(&mut st, Msg::Push { ui });
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(24, 5)).unwrap();
        let mut sizes = Vec::new();
        term.draw(|f| sizes = draw(&st, f).sizes).unwrap();
        assert_eq!(sizes, vec![("note".to_string(), 22, 3)], "inside the border");
        let buf = term.backend().buffer();
        let row = |y: u16| (0..24).map(|x| buf[(x, y)].symbol()).collect::<String>();
        assert_eq!(row(1), "│line one              │");
        assert_eq!(row(2), "│line two              │");
        // The caret (focused, at the start) is a reversed cell.
        assert!(buf[(1, 1)].modifier.contains(Modifier::REVERSED));
    }
}
