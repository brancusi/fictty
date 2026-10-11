//! Markdown lists in an editor's text: ordered markers kept in number (`1.`, `a.`, `i.`), a style
//! per depth for a level that's new, and a key that changes one list's style. Pure functions from
//! text to edits (`[from, to, text]` in chars); the editor applies them as the person's edit.
//!
//! A list's style is its own: the first item at a level says what its siblings are. The depth
//! sequence (`"ordered": ["1.", "a.", "i."]`) only names the style of a level an item has just
//! moved into with no siblings there.

/// How an ordered marker counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Count {
    Number,
    Lower,
    Upper,
    Roman,
    RomanUpper,
}

/// An ordered marker's style: how it counts, and what follows the count (`.` or `)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub count: Count,
    pub delim: char,
}

/// A list item's marker: a bullet, or an ordered one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Bullet(char),
    Ordered(Style),
}

impl Style {
    /// `"1."`, `"a)"`, `"i."`, `"A."`, `"I."`.
    pub fn parse(s: &str) -> Option<Style> {
        let delim = s.chars().last().filter(|c| matches!(c, '.' | ')'))?;
        let body = &s[..s.len() - 1];
        let count = match body {
            b if !b.is_empty() && b.chars().all(|c| c.is_ascii_digit()) => Count::Number,
            "i" => Count::Roman,
            "I" => Count::RomanUpper,
            b if b.len() == 1 && b.chars().all(|c| c.is_ascii_lowercase()) => Count::Lower,
            b if b.len() == 1 && b.chars().all(|c| c.is_ascii_uppercase()) => Count::Upper,
            _ => return None,
        };
        Some(Style { count, delim })
    }

    /// The marker for the `n`th item (from 1).
    pub fn format(self, n: usize) -> String {
        let body = match self.count {
            Count::Number => n.to_string(),
            Count::Lower => letters(n),
            Count::Upper => letters(n).to_uppercase(),
            Count::Roman => roman(n),
            Count::RomanUpper => roman(n).to_uppercase(),
        };
        format!("{body}{}", self.delim)
    }
}

/// 1 → a, 26 → z, 27 → aa.
fn letters(mut n: usize) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
    }
    out.iter().rev().collect()
}

fn roman(mut n: usize) -> String {
    const R: [(usize, &str); 13] =
        [(1000, "m"), (900, "cm"), (500, "d"), (400, "cd"), (100, "c"), (90, "xc"), (50, "l"), (40, "xl"), (10, "x"), (9, "ix"), (5, "v"), (4, "iv"), (1, "i")];
    let mut out = String::new();
    for (v, s) in R {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

/// A list item in a line: its indent, where its marker is (chars, the marker without the space
/// after it), the marker's text, and what kind it is. A first item's count tells its style apart
/// (`i.` is roman, `a.` a letter); later items are renumbered to their list's style anyway.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub indent: usize,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub marker: Marker,
}

pub fn item(line: &str) -> Option<Item> {
    let chars: Vec<char> = line.chars().collect();
    let indent = chars.iter().take_while(|c| **c == ' ').count();
    let rest = &chars[indent..];
    if let Some(&b) = rest.first()
        && matches!(b, '-' | '*' | '+')
        && rest.get(1) == Some(&' ')
    {
        return Some(Item { indent, start: indent, end: indent + 1, text: b.to_string(), marker: Marker::Bullet(b) });
    }
    let n = rest.iter().take_while(|c| c.is_ascii_alphanumeric()).count();
    if n == 0 || n > 6 || !matches!(rest.get(n), Some('.' | ')')) || rest.get(n + 1) != Some(&' ') {
        return None;
    }
    let token: String = rest[..n].iter().collect();
    let delim = rest[n];
    let count = if token.chars().all(|c| c.is_ascii_digit()) {
        Count::Number
    } else if token.chars().all(|c| "ivxlcdm".contains(c)) {
        Count::Roman
    } else if token.chars().all(|c| "IVXLCDM".contains(c)) {
        Count::RomanUpper
    } else if token.chars().all(|c| c.is_ascii_lowercase()) {
        Count::Lower
    } else if token.chars().all(|c| c.is_ascii_uppercase()) {
        Count::Upper
    } else {
        return None;
    };
    let text: String = rest[..=n].iter().collect();
    Some(Item { indent, start: indent, end: indent + n + 1, text, marker: Marker::Ordered(Style { count, delim }) })
}

/// One level of the list being walked: its indent, its style (None for bullets), how many
/// ordered items it has had, and which run of siblings it is.
struct Level {
    indent: usize,
    style: Option<Style>,
    count: usize,
    run: usize,
}

/// For each line, the run of siblings it's an item of (None for a line that isn't an item).
fn walk(lines: &[&str], styles: &[Style], moved: &[usize], mut visit: impl FnMut(usize, &Item, &Level)) {
    let mut stack: Vec<Level> = Vec::new();
    let mut runs = 0;
    for (l, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some(it) = item(line) else {
            // Text inside an item (indented under it) keeps the list going; anything else ends
            // the levels it isn't inside.
            let indent = line.chars().take_while(|c| *c == ' ').count();
            stack.retain(|lv| lv.indent < indent);
            continue;
        };
        stack.retain(|lv| lv.indent <= it.indent);
        let fresh = stack.last().is_none_or(|lv| lv.indent != it.indent);
        if fresh {
            let depth = stack.len();
            let style = match it.marker {
                // An item moved into a level of its own takes the style of that depth.
                Marker::Ordered(_) if moved.contains(&l) && !styles.is_empty() => Some(styles[depth % styles.len()]),
                Marker::Ordered(s) => Some(s),
                Marker::Bullet(_) => None,
            };
            runs += 1;
            stack.push(Level { indent: it.indent, style, count: 0, run: runs });
        }
        let lv = stack.last_mut().expect("a level");
        if matches!(it.marker, Marker::Ordered(_)) && lv.style.is_some() {
            lv.count += 1;
        }
        visit(l, &it, lv);
    }
}

/// The edits that renumber every ordered list in `text`: each ordered item takes its list's style
/// and its place among its siblings. `moved` lines (just indented or outdented) take the style of
/// their new siblings, or of their depth when they have none there.
pub fn renumber(text: &str, styles: &[Style], moved: &[usize]) -> Vec<(usize, usize, String)> {
    let lines: Vec<&str> = text.split('\n').collect();
    let starts = line_starts(&lines);
    let mut out = Vec::new();
    walk(&lines, styles, moved, |l, it, lv| {
        if let (Marker::Ordered(_), Some(style)) = (it.marker, lv.style) {
            let want = style.format(lv.count);
            if want != it.text {
                out.push((starts[l] + it.start, starts[l] + it.end, want));
            }
        }
    });
    out
}

/// The edits that give the list at `line` the next style in number → letter → roman → bullet,
/// every sibling at that level at once.
pub fn cycle_style(text: &str, line: usize) -> Vec<(usize, usize, String)> {
    let lines: Vec<&str> = text.split('\n').collect();
    let starts = line_starts(&lines);
    // Which run of siblings the line is in, and its style now.
    let mut target = None;
    walk(&lines, &[], &[], |l, _, lv| {
        if l == line {
            target = Some((lv.run, lv.style));
        }
    });
    let Some((run, now)) = target else { return vec![] };
    let next: Option<Style> = match now.map(|s| s.count) {
        Some(Count::Number) => Some(Style { count: Count::Lower, delim: '.' }),
        Some(Count::Lower | Count::Upper) => Some(Style { count: Count::Roman, delim: '.' }),
        Some(Count::Roman | Count::RomanUpper) => None,
        None => Some(Style { count: Count::Number, delim: '.' }),
    };
    let mut out = Vec::new();
    let mut n = 0;
    walk(&lines, &[], &[], |l, it, lv| {
        if lv.run != run {
            return;
        }
        n += 1;
        let want = match next {
            Some(s) => s.format(n),
            None => "-".to_string(),
        };
        if want != it.text {
            out.push((starts[l] + it.start, starts[l] + it.end, want));
        }
    });
    out
}

fn line_starts(lines: &[&str]) -> Vec<usize> {
    let mut at = 0;
    lines
        .iter()
        .map(|l| {
            let s = at;
            at += l.chars().count() + 1;
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(text: &str, changes: Vec<(usize, usize, String)>) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        for (f, t, s) in changes.into_iter().rev() {
            chars.splice(f..t, s.chars());
        }
        chars.into_iter().collect()
    }

    fn depth() -> Vec<Style> {
        ["1.", "a.", "i."].iter().map(|s| Style::parse(s).unwrap()).collect()
    }

    #[test]
    fn counts_in_numbers_letters_and_roman() {
        let s = |x| Style::parse(x).unwrap();
        assert_eq!(s("a.").format(27), "aa.");
        assert_eq!(s("i.").format(4), "iv.");
        assert_eq!(s("I)").format(9), "IX)");
        assert_eq!(s("1.").format(12), "12.");
    }

    #[test]
    fn renumbers_each_level_in_its_own_style() {
        let text = "1. one\n1. two\n  a. sub\n  a. sub two\n5. three\n\npara\n\n3. again";
        let want = "1. one\n2. two\n  a. sub\n  b. sub two\n3. three\n\npara\n\n1. again";
        assert_eq!(apply(text, renumber(text, &depth(), &[])), want);
    }

    #[test]
    fn a_moved_item_takes_its_depths_style_or_its_siblings() {
        // Indented under `one`, alone at its new level: the depth's style.
        let text = "1. one\n  2. two\n3. three";
        assert_eq!(apply(text, renumber(text, &depth(), &[1])), "1. one\n  a. two\n2. three");
        // Joining siblings there: theirs.
        let text = "1. one\n  i. sub\n  3. two";
        assert_eq!(apply(text, renumber(text, &depth(), &[2])), "1. one\n  i. sub\n  ii. two");
    }

    #[test]
    fn a_key_cycles_one_lists_style() {
        let text = "1. one\n2. two\n  a. sub\n3. three";
        let s1 = apply(text, cycle_style(text, 0));
        assert_eq!(s1, "a. one\nb. two\n  a. sub\nc. three");
        let s2 = apply(&s1, cycle_style(&s1, 3));
        assert_eq!(s2, "i. one\nii. two\n  a. sub\niii. three");
        let s3 = apply(&s2, cycle_style(&s2, 1));
        assert_eq!(s3, "- one\n- two\n  a. sub\n- three");
        let s4 = apply(&s3, cycle_style(&s3, 0));
        assert_eq!(s4, "1. one\n2. two\n  a. sub\n3. three");
    }
}
