//! Editing keys and the mouse in editors and inputs: the Mac text-field conventions, each with a
//! twin for terminals that don't send ⌘. The semantics are caretline's; these check that fictty
//! delivers every key and click to it and carries its answers out (the clipboard, notices).
//!
//! A golden is the text with the selection in ⟦ ⟧ and the caret as ▮: `Hello ⟦wor▮⟧ld`.

use fictty::App;
use fictty::state::Msg;
use serde_json::json;

/// A screen with one editor, `e`, holding the golden's text with its selection made by keys.
fn editor(golden: &str) -> fictty::Headless {
    let (text, anchor, head) = parse(golden);
    let ui = json!({"root": {"type": "editor", "id": "e", "text": text}});
    let mut s = App::new(serde_json::from_value(ui).unwrap()).headless(60, 8).unwrap();
    s.keys(&["c-home"]);
    for _ in 0..anchor {
        s.keys(&["right"]);
    }
    let (k, n) = if head >= anchor { ("s-right", head - anchor) } else { ("s-left", anchor - head) };
    for _ in 0..n {
        s.keys(&[k]);
    }
    assert_eq!(shown(&mut s), golden, "setting up");
    s
}

fn parse(golden: &str) -> (String, usize, usize) {
    let (mut text, mut anchor, mut head, mut open) = (String::new(), None, None, None);
    for c in golden.chars() {
        let at = text.chars().count();
        match c {
            '⟦' => open = Some(at),
            '⟧' => anchor = Some(if head == open { at } else { open.unwrap() }),
            '▮' => head = Some(at),
            c => text.push(c),
        }
    }
    let head = head.expect("a caret");
    (text, anchor.unwrap_or(head), head)
}

/// The editor's text and selection as a golden.
fn shown(s: &mut fictty::Headless) -> String {
    let st = s.state();
    let view = &st.local["e"].editors["e"];
    let r = view.selection.primary();
    let text = st.store["e"].read()["text"].as_str().unwrap().to_string();
    let (lo, hi) = (r.anchor.min(r.head), r.anchor.max(r.head));
    let mut out = String::new();
    for (i, c) in text.chars().chain(std::iter::once('\0')).enumerate() {
        if lo != hi && i == lo {
            out.push('⟦');
        }
        if i == r.head {
            out.push('▮');
        }
        if lo != hi && i == hi {
            out.push('⟧');
        }
        if c != '\0' {
            out.push(c);
        }
    }
    out
}

/// Each key (and its twins) from `from` gives `to`.
fn check(from: &str, keys: &[&str], to: &str) {
    for k in keys {
        let mut s = editor(from);
        s.keys(&[k]);
        assert_eq!(shown(&mut s), to, "{from} then {k}");
    }
}

#[test]
fn row_and_document_ends() {
    check("one tw▮o\nthree", &["d-left", "home", "c-a"], "▮one two\nthree");
    check("one tw▮o\nthree", &["d-right", "end", "c-e"], "one two▮\nthree");
    check("one\ntw▮o", &["d-up", "c-home"], "▮one\ntwo");
    check("o▮ne\ntwo", &["d-down", "c-end"], "one\ntwo▮");
}

#[test]
fn words() {
    check("one two thr▮ee", &["a-left", "a-b"], "one two ▮three");
    check("on▮e two three", &["a-right", "a-f"], "one▮ two three");
}

#[test]
fn shift_extends_any_motion_and_never_moves_the_anchor() {
    check("one tw▮o", &["s-left"], "one t⟦▮w⟧o");
    check("one tw▮o", &["a-s-left"], "one ⟦▮tw⟧o");
    check("one tw▮o", &["d-s-left", "s-home"], "⟦▮one tw⟧o");
    check("o▮ne two", &["d-s-right", "s-end"], "o⟦ne two▮⟧");
    // E6
    check("ab ⟦▮c⟧ d", &["s-right"], "ab c▮ d");
}

#[test]
fn an_arrow_collapses_a_selection_toward_its_side() {
    // E1
    check("Hello ⟦wor▮⟧ld", &["left"], "Hello ▮world");
    check("Hello ⟦wor▮⟧ld", &["right"], "Hello wor▮ld");
    // E10
    check("one two ⟦thr▮⟧ee", &["a-left", "a-b"], "one ▮two three");
}

#[test]
fn esc_drops_a_selection_before_it_leaves() {
    let mut s = editor("Hello ⟦wor▮⟧ld");
    s.keys(&["esc"]);
    assert_eq!(shown(&mut s), "Hello wor▮ld");
    assert_eq!(s.state().focus.as_deref(), Some("e"), "still editing");
}

#[test]
fn deletes() {
    // E29
    check("one two thr▮ee", &["d-bs", "c-u"], "▮ee");
    check("one tw▮o three", &["d-del"], "one tw▮");
    check("one two thr▮ee", &["a-bs", "c-w"], "one two ▮ee");
    check("one tw▮o three", &["a-del", "a-d"], "one tw▮ three");
}

#[test]
fn with_a_selection_every_delete_deletes_just_the_selection() {
    for k in ["bs", "del", "a-bs", "c-w", "a-del", "a-d", "d-bs", "c-u", "d-del"] {
        check("one ⟦two▮⟧ three", &[k], "one ▮ three");
    }
}

#[test]
fn typing_replaces_the_selection_and_undo_brings_it_back() {
    let mut s = editor("one ⟦two▮⟧ three");
    s.keys(&["X"]);
    assert_eq!(shown(&mut s), "one X▮ three");
    s.keys(&["d-z"]);
    assert_eq!(shown(&mut s), "one ⟦two▮⟧ three");
    s.keys(&["d-s-z"]);
    assert_eq!(shown(&mut s), "one X▮ three");
    s.keys(&["c-z"]);
    assert_eq!(shown(&mut s), "one ⟦two▮⟧ three", "c-z");
    for redo in ["c-y", "c-r"] {
        let mut s = editor("one ⟦two▮⟧ three");
        s.keys(&["X", "c-z", redo]);
        assert_eq!(shown(&mut s), "one X▮ three", "{redo}");
    }
}

#[test]
fn copy_cut_and_paste_use_the_system_clipboard() {
    // E33: copy keeps the selection and sends the text out.
    for k in ["d-c", "c-c"] {
        let mut s = editor("one ⟦two▮⟧ three");
        s.keys(&[k]);
        assert_eq!(shown(&mut s), "one ⟦two▮⟧ three", "{k}");
        assert_eq!(s.copied(), ["two"], "{k}");
    }
    // E36: nothing selected, nothing copied, and it says so.
    let mut s = editor("one tw▮o");
    s.keys(&["d-c"]);
    assert!(s.copied().is_empty());
    assert_eq!(s.state().toasts.last().map(|t| t.text.as_str()), Some("nothing selected"));
    // Cut, then paste at the same place, is where it started.
    for (cut, paste) in [("d-x", "d-v"), ("c-x", "c-v")] {
        let mut s = editor("one ⟦two▮⟧ three");
        s.keys(&[cut]);
        assert_eq!(shown(&mut s), "one ▮ three", "{cut}");
        assert_eq!(s.copied(), ["two"]);
        s.keys(&[paste]);
        assert_eq!(shown(&mut s), "one two▮ three", "{paste}");
    }
}

#[test]
fn select_all() {
    // E41
    for k in ["d-a", "a-a"] {
        let mut s = editor("one tw▮o");
        s.keys(&[k]);
        assert_eq!(shown(&mut s), "⟦one two▮⟧", "{k}");
        s.keys(&["left"]);
        assert_eq!(shown(&mut s), "▮one two");
    }
    // E42
    let mut s = editor("one tw▮o");
    s.keys(&["d-a", "bs"]);
    assert_eq!(shown(&mut s), "▮");
    s.keys(&["d-z"]);
    assert_eq!(shown(&mut s), "⟦one two▮⟧");
}

#[test]
fn pasting_from_the_terminal_types_the_text() {
    let mut s = editor("one ⟦two▮⟧ three");
    s.msg(Msg::Paste { text: "2".into() });
    assert_eq!(shown(&mut s), "one 2▮ three");
}

#[test]
fn the_mouse_places_extends_and_selects_words_and_lines() {
    let mut s = editor("▮hello world\nsecond line");
    s.msg(Msg::Point { id: "e".into(), col: 2, row: 0, extend: false });
    assert_eq!(shown(&mut s), "he▮llo world\nsecond line");
    s.msg(Msg::Point { id: "e".into(), col: 8, row: 0, extend: true });
    assert_eq!(shown(&mut s), "he⟦llo wo▮⟧rld\nsecond line", "shift-click keeps the anchor");
    s.msg(Msg::Drag { id: "e".into(), col: 3, row: 1 });
    assert_eq!(shown(&mut s), "he⟦llo world\nsec▮⟧ond line", "a drag stretches it");
    // Double, then triple click on one cell.
    let mut s = editor("▮hello world\nsecond line");
    for _ in 0..2 {
        s.msg(Msg::Point { id: "e".into(), col: 8, row: 0, extend: false });
    }
    assert_eq!(shown(&mut s), "hello ⟦world▮⟧\nsecond line");
    s.msg(Msg::Point { id: "e".into(), col: 8, row: 0, extend: false });
    assert_eq!(shown(&mut s), "⟦hello world▮⟧\nsecond line");
}

#[test]
fn invariants_hold_over_a_run_of_keys() {
    let motions = ["left", "right", "up", "down", "a-left", "a-right", "home", "end", "d-up", "d-down"];
    let start = "alpha beta\ngamma ⟦delta▮⟧\nepsilon";
    for m in motions {
        // A motion without shift leaves no selection.
        let mut s = editor(start);
        s.keys(&[m]);
        let g = shown(&mut s);
        assert!(!g.contains('⟦'), "{m}: {g}");
        // With shift it never moves the anchor.
        let mut s = editor(start);
        let before = s.state().local["e"].editors["e"].selection.primary().anchor;
        s.keys(&[&format!("s-{m}").replace("s-a-", "a-s-").replace("s-d-", "d-s-")]);
        let after = s.state().local["e"].editors["e"].selection.primary().anchor;
        assert_eq!(before, after, "s-{m}");
    }
    // Copy changes nothing but the clipboard.
    let mut s = editor(start);
    let text = shown(&mut s);
    s.keys(&["d-c", "d-c"]);
    assert_eq!(shown(&mut s), text);
    s.keys(&["d-z"]);
    assert_eq!(shown(&mut s), text, "nothing to undo");
    // N edits, N undos.
    let mut s = editor("▮");
    let typed = ["a", "b", "enter", "c"];
    // Far apart in time, so each is its own undo step.
    let mut now = 0;
    for k in typed {
        s.keys(&[k]);
        now += 10_000;
        s.msg(Msg::Tick { now_ms: now });
    }
    for _ in typed {
        s.keys(&["d-z"]);
    }
    assert_eq!(shown(&mut s), "▮");
}
