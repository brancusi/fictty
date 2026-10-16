//! The native window as a test drives it (`App::in_window`): laid out by GPUI's test platform,
//! its mouse going through the window's own handlers, read back as an agent would. A cell is
//! 8.4 × 24 px there, whatever the machine's fonts.
#![cfg(target_os = "macos")]

use fictty::App;
use fictty::readback::Shown;
use fictty::state::Msg;
use serde_json::{Value, json};

fn window(ui: Value, w: u16, h: u16) -> fictty::Headless {
    App::new(serde_json::from_value(ui).unwrap()).in_window(w, h).unwrap()
}

/// The shown node with id `id`.
fn node(s: &mut fictty::Headless, id: &str) -> Shown {
    s.readback().into_iter().find(|n| n.id.as_deref() == Some(id)).unwrap_or_else(|| panic!("no node {id}"))
}

/// The editor's text with its selection marked: `⟦` `⟧` around it, `▮` at the caret.
fn marked(s: &mut fictty::Headless, id: &str) -> String {
    let st = s.state();
    let view = &st.local[id].editors[id];
    let r = view.selection.primary();
    let text = st.store[id].read()["text"].as_str().unwrap().to_string();
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

/// The column an editor's text starts at: the last one, from its edge, where a press puts the
/// caret first (a press left of the text puts it there too).
fn text_start(s: &mut fictty::Headless, id: &str) -> u16 {
    let e = node(s, id).rect.unwrap();
    let firsts: Vec<u16> = (e.x..e.x + 4)
        .filter(|&x| {
            s.mouse("press", x, e.y).mouse("release", x, e.y);
            let now = s.state().now_ms;
            // Apart on the clock, so the presses don't count as a double click.
            s.msg(Msg::Tick { now_ms: now + 1000 });
            marked(s, id).starts_with('▮')
        })
        .collect();
    *firsts.last().expect("a press at the editor's left edge puts the caret first")
}

fn rows(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("row {i}")).collect()
}

#[test]
fn the_window_lays_nodes_out_side_by_side_and_reads_them_back() {
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "l", "size": 20, "items": rows(5)},
        {"type": "editor", "id": "e", "text": "hello"}
    ]}});
    let mut s = window(ui, 80, 12);
    let (l, e) = (node(&mut s, "l").rect.unwrap(), node(&mut s, "e").rect.unwrap());
    assert_eq!(l.width, 20, "a size in cells is that many columns: {l:?}");
    assert_eq!(l.x + l.width, e.x, "the editor starts where the list ends: {l:?} {e:?}");
    assert!(e.width >= 55 && e.height >= 8, "the editor takes the rest: {e:?}");
    // The engine learned the editor's size from the window, so its view is that big.
    let vp = s.state().local["e"].editors["e"].viewport;
    assert!(vp.width > 50 && vp.height > 7 && vp.width <= e.width && vp.height <= e.height, "{vp:?} in {e:?}");
    let caps = s.state().caps;
    assert!((caps.cell_w - 8.4).abs() < 0.01 && caps.cell_h == 24.0, "the window's cell, from its font: {caps:?}");
}

#[test]
fn a_press_on_a_row_selects_it_and_a_tab_switches() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "tabs", "id": "t", "size": 3, "tabs": ["One", "Two"], "children": [
            {"type": "text", "text": "first"}, {"type": "text", "text": "second"}
        ]},
        {"type": "list", "id": "l", "items": rows(10)}
    ]}});
    let mut s = window(ui, 60, 16);
    let l = node(&mut s, "l").rect.unwrap();
    s.mouse("press", l.x + 2, l.y + 3).mouse("release", l.x + 2, l.y + 3);
    assert_eq!(s.state().local["l"].selected, 3, "the fourth row: {l:?}");
    assert_eq!(s.state().focus.as_deref(), Some("l"));
    // The second tab's title, a few columns in from the first.
    let t = node(&mut s, "t").rect.unwrap();
    let mut chose = None;
    for x in t.x..t.x + 20 {
        s.mouse("press", x, t.y).mouse("release", x, t.y);
        if s.state().local.get("t").is_some_and(|l| l.tab == 1) {
            chose = Some(x);
            break;
        }
    }
    assert!(chose.is_some(), "some press along the tab bar chose the second tab");
    assert!(s.screen().contains("second"), "{}", s.screen());
}

#[test]
fn a_press_on_a_picture_runs_the_click_of_the_card_around_it() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "row", "id": "card", "size": 6, "click": {"put": "picked", "value": "card"}, "children": [
            {"type": "image", "id": "pic", "size": 10, "src": "nothing.png"},
            {"type": "text", "id": "name", "text": "a name"}
        ]},
        {"type": "list", "id": "l", "click": {"put": "picked", "value": "list"}, "items": rows(3)}
    ]}});
    let mut s = window(ui, 60, 16);
    let p = node(&mut s, "pic").rect.unwrap();
    s.mouse("press", p.x + 1, p.y + 1).mouse("release", p.x + 1, p.y + 1);
    assert_eq!(s.state().store["picked"].read()["value"], "card");
    // A row takes its own press, once: the list's click runs after it selects, not the card's.
    let l = node(&mut s, "l").rect.unwrap();
    s.mouse("press", l.x + 2, l.y + 1).mouse("release", l.x + 2, l.y + 1);
    assert_eq!(s.state().local["l"].selected, 1);
    assert_eq!(s.state().store["picked"].read()["value"], "list");
}

#[test]
fn an_image_follows_the_row_its_src_is_filled_from() {
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "l", "size": 20, "item": "{name}",
         "items": [{"name": "a", "file": "/pics/a.png"}, {"name": "b", "file": "/pics/b.jpg"}]},
        {"type": "image", "id": "pic", "bind": "@l", "src": "{file}", "fit": "cover"}
    ]}});
    let mut s = window(ui, 60, 12);
    assert_eq!(node(&mut s, "pic").text, "/pics/a.png");
    s.keys(&["j"]);
    assert_eq!(node(&mut s, "pic").text, "/pics/b.jpg");
    assert!(node(&mut s, "pic").rect.is_some_and(|r| r.width > 30), "it fills the rest of the row");
}

/// The picked rows of `id`, by index, as the readback has them (empty while one row is).
fn picked(s: &mut fictty::Headless, id: &str) -> Vec<usize> {
    node(s, id).picked
}

#[test]
fn a_multi_list_picks_with_command_shift_and_the_keys() {
    let items: Vec<Value> = (0..8).map(|i| json!({"id": format!("r{i}"), "name": format!("row {i}")})).collect();
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "list", "id": "l", "multi": true, "item": "{name}", "items": items},
        {"type": "text", "id": "n", "bind": "$picked.l", "size": 1, "text": "{count} picked"}
    ]}});
    let mut s = window(ui, 60, 16);
    let l = node(&mut s, "l").rect.unwrap();
    let press = |s: &mut fictty::Headless, kind: &str, row: u16| {
        s.mouse(kind, l.x + 2, l.y + row).mouse("release", l.x + 2, l.y + row);
    };
    press(&mut s, "press", 1);
    assert_eq!(picked(&mut s, "l"), Vec::<usize>::new(), "a click is one row");
    press(&mut s, "d-press", 3);
    press(&mut s, "d-press", 5);
    assert_eq!(picked(&mut s, "l"), vec![1, 3, 5]);
    assert_eq!(node(&mut s, "n").text, "3 picked");
    assert_eq!(s.state().local["l"].selected, 5, "the row last picked is the one the keys move");
    press(&mut s, "d-press", 3);
    assert_eq!(picked(&mut s, "l"), vec![1, 5], "⌘ again picks it out");
    // ⇧ picks from the anchor (the row last ⌘-pressed, 3) to the row pressed.
    press(&mut s, "s-press", 6);
    assert_eq!(picked(&mut s, "l"), vec![3, 4, 5, 6]);
    s.keys(&["s-up", "s-up"]);
    assert_eq!(picked(&mut s, "l"), vec![3, 4]);
    s.keys(&["esc"]);
    assert_eq!(picked(&mut s, "l"), Vec::<usize>::new(), "Esc goes back to one row");
    assert_eq!(s.state().local["l"].selected, 4);
    s.keys(&["d-a"]);
    assert_eq!(picked(&mut s, "l").len(), 8);
    s.keys(&["j"]);
    assert_eq!(picked(&mut s, "l"), Vec::<usize>::new(), "a plain move is one row again");
    // Every picked row is drawn selected; the lead is also active.
    press(&mut s, "press", 0);
    press(&mut s, "s-press", 2);
    let st = s.state();
    let tree = st.ui.root.clone();
    let n = tree.find("l").unwrap();
    let states = |k| fictty::style::row_states(st, n, k);
    assert!(states(0).contains(&"selected") && states(1).contains(&"selected") && !states(3).contains(&"selected"));
    assert!(states(2).contains(&"active") && !states(0).contains(&"active"));
}

/// Entry `key`'s text.
fn text_of(s: &mut fictty::Headless, key: &str) -> String {
    s.state().store.get(key).map(|e| e.read()["text"].as_str().unwrap_or_default().to_string()).unwrap_or_default()
}

#[test]
fn a_field_over_picked_rows_writes_to_each_and_a_field_left_alone_to_none() {
    let items: Vec<Value> = (0..4).map(|i| json!({"id": format!("p{i}"), "name": format!("pic {i}")})).collect();
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "files", "multi": true, "size": 20, "item": "{name}", "items": items},
        {"type": "col", "children": [
            {"type": "editor", "id": "caption", "bind": "@files", "doc": "caption/{id}", "size": 4, "placeholder": "Caption"},
            {"type": "editor", "id": "date", "bind": "@files", "doc": "date/{id}", "size": 3}
        ]}
    ]}, "store": {"caption/p0": "a red door", "caption/p1": "a blue gate", "caption/p2": "a green wall"}, "focus": "files"});
    let mut s = window(ui, 70, 16);
    // The first three picked: their captions differ, their dates are all empty.
    s.keys(&["s-down", "s-down"]);
    assert_eq!(node(&mut s, "files").picked, vec![0, 1, 2]);
    let caption = node(&mut s, "caption");
    assert!(caption.mixed && caption.entries == 3 && caption.text.is_empty(), "{caption:?}");
    let st = s.state();
    let tree = st.ui.root.clone();
    assert_eq!(fictty::state::placeholder(st, tree.find("caption").unwrap()).as_deref(), Some("Multiple values"));
    assert!(!node(&mut s, "date").mixed);
    // A date typed once lands in all three; the captions are left as they were.
    s.keys(&["tab", "tab"]);
    assert_eq!(s.state().focus.as_deref(), Some("date"));
    s.keys(&["2", "0", "2", "6"]);
    for i in 0..3 {
        assert_eq!(text_of(&mut s, &format!("date/p{i}")), "2026");
    }
    assert_eq!(text_of(&mut s, "date/p3"), "", "the row not picked");
    assert_eq!(text_of(&mut s, "caption/p1"), "a blue gate");
    // A caption typed over the mixed ones makes them one, and undo gives each its own back.
    s.keys(&["s-tab"]);
    s.keys(&["d", "o", "o", "r"]);
    for i in 0..3 {
        assert_eq!(text_of(&mut s, &format!("caption/p{i}")), "door");
    }
    assert!(!node(&mut s, "caption").mixed);
    s.keys(&["d-z", "d-z"]);
    assert_eq!(text_of(&mut s, "caption/p0"), "a red door");
    assert_eq!(text_of(&mut s, "caption/p2"), "a green wall");
    assert!(node(&mut s, "caption").mixed, "different again");
    assert!(!s.state().store.keys().any(|k| k.starts_with("$mixed/")) || node(&mut s, "caption").mixed);
    // One row picked: the editor it always was.
    s.keys(&["s-tab", "esc"]);
    assert_eq!(node(&mut s, "caption").entries, 1);
    assert_eq!(node(&mut s, "caption").text, "a green wall");
}

#[test]
fn a_list_of_rows_drawn_from_a_node_lays_them_out_in_cols_and_picks_from_a_picture() {
    let items: Vec<Value> = (0..30).map(|i| json!({"id": format!("p{i}"), "name": format!("pic {i}"), "thumb": format!("/thumbs/p{i}.png")})).collect();
    let row = json!({"type": "col", "size": 4, "children": [
        {"type": "image", "src": "{$item.thumb}"},
        {"type": "text", "size": 1, "text": "{$item.name}"}
    ]});
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "g", "multi": true, "cols": 3, "size": 36, "row": row, "items": items},
        {"type": "text", "id": "n", "bind": "$picked.g", "text": "{count} picked"}
    ]}, "focus": "g"});
    let mut s = window(ui, 60, 14);
    let g = node(&mut s, "g");
    assert_eq!(g.text, "/thumbs/p0.png · pic 0", "the selected row reads back as what it shows");
    let r = g.rect.unwrap();
    // A press on the picture in the second column of the first line, then ⌘ on the third.
    let col = |c: u16| r.x + r.width * c / 3 + r.width / 6;
    s.mouse("press", col(1), r.y + 1).mouse("release", col(1), r.y + 1);
    assert_eq!(s.state().local["g"].selected, 1);
    s.mouse("d-press", col(2), r.y + 1).mouse("release", col(2), r.y + 1);
    // ⇧ on the first of the second line: the range from the anchor (2) to it (3).
    s.mouse("s-press", col(0), r.y + 5).mouse("release", col(0), r.y + 5);
    assert_eq!(node(&mut s, "g").picked, vec![2, 3]);
    assert_eq!(node(&mut s, "n").text, "2 picked");
    // ↓ goes a line, → a row.
    s.keys(&["esc", "down"]);
    assert_eq!(s.state().local["g"].selected, 6);
    s.keys(&["right"]);
    assert_eq!(s.state().local["g"].selected, 7);
    // Far down, the view follows by whole lines: the first row shown starts a line.
    s.keys(&["down", "down", "down", "down", "down"]);
    assert_eq!(s.state().local["g"].selected, 22);
    let fit = s.state().local["g"].rows as usize;
    assert!(fit >= 6 && fit.is_multiple_of(3), "whole lines of rows fit: {fit}");
    let st = s.state();
    let from = fictty::state::first_shown(st.local.get("g"), 30, fit, 3);
    assert!(from.is_multiple_of(3));
    assert!((from..from + fit).contains(&22), "{from} + {fit}");
}

#[test]
fn quit_ends_the_screen_where_c_c_would_copy() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "hi"}, "focus": "e"});
    let mut s = window(ui, 40, 8);
    s.keys(&["c-c"]);
    assert!(!s.quit(), "⌃C in an editor is copy");
    s.request(json!({"op": "quit"}));
    assert!(s.quit());
}

#[test]
fn a_press_on_a_list_that_isnt_multi_with_a_modifier_is_a_click() {
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows(6)}});
    let mut s = window(ui, 40, 12);
    let l = node(&mut s, "l").rect.unwrap();
    s.mouse("d-press", l.x + 2, l.y + 2).mouse("release", l.x + 2, l.y + 2);
    s.mouse("s-press", l.x + 2, l.y + 4).mouse("release", l.x + 2, l.y + 4);
    assert_eq!(s.state().local["l"].selected, 4);
    assert!(picked(&mut s, "l").is_empty());
}

#[test]
fn a_press_in_an_editor_places_the_caret_and_a_drag_selects() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "alpha beta gamma\nsecond line"}});
    let mut s = window(ui, 60, 8);
    let e = node(&mut s, "e").rect.unwrap();
    let x0 = text_start(&mut s, "e");
    s.mouse("press", x0 + 6, e.y).mouse("release", x0 + 6, e.y);
    assert_eq!(marked(&mut s, "e"), "alpha ▮beta gamma\nsecond line");
    s.mouse("press", x0 + 3, e.y + 1).mouse("release", x0 + 3, e.y + 1);
    assert_eq!(marked(&mut s, "e"), "alpha beta gamma\nsec▮ond line", "the second row");
    // A drag from the start of `beta` to the end of `gamma`.
    s.mouse("press", x0 + 6, e.y).mouse("drag", x0 + 10, e.y).mouse("drag", x0 + 16, e.y).mouse("release", x0 + 16, e.y);
    assert_eq!(marked(&mut s, "e"), "alpha ⟦beta gamma▮⟧\nsecond line");
}

#[test]
fn a_double_press_selects_a_word_and_a_triple_the_line() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "alpha beta gamma\nsecond line"}});
    let mut s = window(ui, 60, 8);
    let e = node(&mut s, "e").rect.unwrap();
    let x0 = text_start(&mut s, "e");
    // Far enough apart on the clock that this starts a new run of clicks.
    let now = s.state().now_ms;
    s.msg(Msg::Tick { now_ms: now + 2000 });
    for _ in 0..2 {
        s.mouse("press", x0 + 7, e.y).mouse("release", x0 + 7, e.y);
    }
    assert!(marked(&mut s, "e").starts_with("alpha ⟦beta"), "{}", marked(&mut s, "e"));
    s.mouse("press", x0 + 7, e.y).mouse("release", x0 + 7, e.y);
    assert!(marked(&mut s, "e").starts_with("⟦alpha beta gamma"), "{}", marked(&mut s, "e"));
}

#[test]
fn the_wheel_scrolls_a_list_from_the_state_and_leaves_the_selection() {
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows(80)}});
    let mut s = window(ui, 30, 14);
    let l = node(&mut s, "l").rect.unwrap();
    let fit = s.state().local["l"].rows;
    assert!(fit >= 8 && fit <= l.height, "the rows the window had room for: {fit} in {l:?}");
    for _ in 0..3 {
        s.mouse("wheel-down", l.x + 2, l.y + 2);
    }
    assert_eq!(s.state().local["l"].top, Some(9.0), "three notches, three rows each");
    assert_eq!(s.state().local["l"].selected, 0, "the selection stays");
    // The rows drawn are the ones the state says: a press lands on row 9 + 1.
    s.mouse("press", l.x + 2, l.y + 1).mouse("release", l.x + 2, l.y + 1);
    assert_eq!(s.state().local["l"].selected, 10);
    s.mouse("wheel-up", l.x + 2, l.y + 2);
    assert_eq!(s.state().local["l"].top, Some(6.0));
    assert!(node(&mut s, "l").scroll.is_some_and(|(at, of)| at == 6.0 && of == 80), "{:?}", node(&mut s, "l").scroll);
}

#[test]
fn a_trackpad_moves_a_list_by_part_of_a_row_and_a_press_still_lands_on_the_row_under_it() {
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows(80)}});
    let mut s = window(ui, 30, 14);
    let l = node(&mut s, "l").rect.unwrap();
    // Two and three quarter rows down, as a trackpad would, in small moves.
    for _ in 0..11 {
        s.msg(Msg::Pan { id: "l".into(), lines: 0.25 });
    }
    assert_eq!(s.state().local["l"].top, Some(2.75));
    // The rows are drawn three quarters of a row up: the middle of the list's second cell row
    // is on row 4 (it would be row 3 if only whole rows moved). A frame first, as a window
    // draws one before the next press.
    s.screen();
    s.mouse("press", l.x + 2, l.y + 1).mouse("release", l.x + 2, l.y + 1);
    assert_eq!(s.state().local["l"].selected, 4);
    // Never past the start or the last whole view.
    s.msg(Msg::Pan { id: "l".into(), lines: -40.0 });
    assert_eq!(s.state().local["l"].top, Some(0.0));
    s.msg(Msg::Pan { id: "l".into(), lines: 500.0 });
    let fit = s.state().local["l"].rows as f32;
    assert_eq!(s.state().local["l"].top, Some(80.0 - fit));
}

#[test]
fn the_wheel_scrolls_an_editor() {
    let text: Vec<String> = (0..60).map(|i| format!("line {i}")).collect();
    let ui = json!({"root": {"type": "editor", "id": "e", "text": text.join("\n")}});
    let mut s = window(ui, 40, 10);
    let e = node(&mut s, "e").rect.unwrap();
    s.mouse("wheel-down", e.x + 3, e.y + 2).mouse("wheel-down", e.x + 3, e.y + 2);
    let scroll = s.state().local["e"].editors["e"].scroll.line;
    assert_eq!(scroll, 6, "two notches, three lines each");
}

#[test]
fn a_resize_reaches_the_screen_and_the_editors_size() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "hello"}});
    let mut s = window(ui, 40, 10);
    let before = s.state().local["e"].editors["e"].viewport;
    s.resize(70, 20);
    let after = s.state().local["e"].editors["e"].viewport;
    assert_eq!(s.state().data["$screen"], fictty::state::Slot::Ready { value: json!({"width": 70, "height": 20}) });
    assert!(after.width >= before.width + 30 && after.height >= before.height + 10, "{before:?} → {after:?}");
}

#[test]
fn the_fuzz_drivers_mouse_acts_reach_the_window() {
    // The window fuzzes clean; this makes sure that's not because its mouse does nothing: over
    // a run of generated acts, about as many change selections, focus and carets as in the
    // terminal's grid (most are moves, releases and presses on what's already chosen).
    use fictty::fuzz::{Act, Config, generate};
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "l", "items": rows(40)},
        {"type": "editor", "id": "e", "text": "alpha beta gamma\nsecond line\nthird"}
    ]}});
    let cfg = Config { steps: 400, ..Config::default() };
    let count = |mut s: fictty::Headless| {
        let (mut acts, mut changed) = (0, 0);
        for act in generate(&cfg, 7, s.size()).into_iter().filter(|a| matches!(a, Act::Mouse(..))) {
            let before = serde_json::to_value(&s.state().local).unwrap();
            act.apply(&mut s);
            acts += 1;
            changed += (serde_json::to_value(&s.state().local).unwrap() != before) as usize;
        }
        (acts, changed)
    };
    let win = count(window(ui.clone(), 60, 12));
    let grid = count(App::new(serde_json::from_value(ui).unwrap()).headless(60, 12).unwrap());
    assert!(win.0 > 20 && win.1 >= 5 && win.1 * 4 >= grid.1 * 3, "{} of {} mouse acts changed the view state in the window, {} in the grid", win.1, win.0, grid.1);
}

#[test]
fn keys_reach_the_window_as_keystrokes() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": ""}, "focus": "e"});
    let mut s = window(ui, 40, 6);
    s.keys(&["H", "i", "space", "!", "漢"]);
    assert_eq!(marked(&mut s, "e"), "Hi !漢▮", "typed, shifted and wide characters");
    s.keys(&["s-left", "s-left"]);
    assert_eq!(marked(&mut s, "e"), "Hi ⟦▮!漢⟧", "a modifier with a motion");
    s.keys(&["bs"]);
    assert_eq!(marked(&mut s, "e"), "Hi ▮");
    s.keys(&["d-z"]);
    assert_eq!(marked(&mut s, "e"), "Hi ⟦▮!漢⟧", "⌘Z undoes, the selection as it was");
}

#[test]
fn a_paste_comes_through_the_clipboard_and_a_copy_goes_onto_it() {
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "one two"}, "focus": "e"});
    let mut s = window(ui, 40, 6);
    s.keys(&["d-right"]).paste(" three");
    assert_eq!(marked(&mut s, "e"), "one two three▮", "pasted as ⌘V with the text on the clipboard");
    // Copy `one`, then paste it at the end: ⌘C puts it on the clipboard, ⌘V takes it from there.
    s.keys(&["d-left", "s-a-right", "d-c", "d-right", "d-v"]);
    assert_eq!(marked(&mut s, "e"), "one two threeone▮");
    assert_eq!(s.copied().last().map(String::as_str), Some("one"));
}

#[test]
fn moving_over_a_row_hovers_it() {
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows(10)}});
    let mut s = window(ui, 30, 14);
    let (x, y) = s.at("l", Some(4), None).expect("row 4 was drawn");
    s.mouse("move", x, y);
    assert_eq!(s.state().hover, Some(fictty::state::Hover { id: "l".into(), row: Some(4) }));
    let (x, y) = s.at("l", Some(2), None).unwrap();
    s.mouse("move", x, y).mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().hover.as_ref().and_then(|h| h.row), Some(2));
    assert_eq!(s.state().local["l"].selected, 2, "pressed where it pointed");
}

#[test]
fn the_finders_point_at_what_was_drawn() {
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "list", "id": "l", "size": 20, "items": rows(5)},
        {"type": "tabs", "id": "t", "tabs": ["One", "Two", "Three"], "children": [
            {"type": "editor", "id": "a", "text": "first"}, {"type": "text", "text": "second"}, {"type": "text", "text": "third"}
        ]}
    ]}});
    let mut s = window(ui, 70, 10);
    for k in [2, 0, 1] {
        let (x, y) = s.at("t", None, Some(k)).unwrap_or_else(|| panic!("tab {k}"));
        s.mouse("press", x, y).mouse("release", x, y);
        assert_eq!(s.state().local["t"].tab, k);
    }
    let (x, y) = s.at("l", Some(3), None).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().local["l"].selected, 3);
    // Back on the first tab, the editor's text: a press four columns in puts the caret there.
    let (x, y) = s.at("t", None, Some(0)).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    let text = s.text_at("a").expect("the editor's text was drawn");
    s.mouse("press", text.x + 4, text.y).mouse("release", text.x + 4, text.y);
    assert_eq!(marked(&mut s, "a"), "firs▮t");
}

// ---- layout ---------------------------------------------------------------------------------

/// A list of files, a detail beside it on a wide screen, and a modal dialog over both.
fn dialog() -> Value {
    json!({
        "root": {"type": "stack", "id": "stack", "children": [
            {"type": "row", "children": [
                {"type": "list", "id": "files", "items": ["a.txt", "b.txt"],
                 "keys": {"d": {"put": "confirm", "value": true}}},
                {"type": "text", "id": "detail", "text": "wide detail", "when": "$screen.width>=60"}
            ]},
            {"type": "col", "id": "ask", "title": "Delete?", "modal": true, "when": "confirm",
             "float": {"width": 30, "height": 5},
             "children": [
                {"type": "text", "text": "This can't be undone.", "size": 1},
                {"type": "row", "size": 1, "children": [
                    {"type": "button", "id": "yes", "label": "Delete",
                     "click": {"do": [{"put": "confirm", "value": false}, {"toast": "Deleted", "tone": "neg"}]}},
                    {"type": "button", "id": "no", "label": "Keep", "click": {"put": "confirm", "value": false}}
                ]}
            ]}
        ]},
        "store": {"confirm": false}
    })
}

/// Whether node `id` shows.
fn shows(s: &mut fictty::Headless, id: &str) -> bool {
    s.readback().iter().any(|n| n.id.as_deref() == Some(id))
}

#[test]
fn when_hides_a_node_and_gives_its_room_away() {
    let mut wide = window(dialog(), 80, 8);
    assert!(shows(&mut wide, "detail"));
    let (files, detail) = (node(&mut wide, "files").rect.unwrap(), node(&mut wide, "detail").rect.unwrap());
    assert_eq!(files.x + files.width, detail.x, "side by side: {files:?} {detail:?}");
    let mut narrow = window(dialog(), 50, 8);
    assert!(!shows(&mut narrow, "detail"));
    let (files, stack) = (node(&mut narrow, "files").rect.unwrap(), node(&mut narrow, "stack").rect.unwrap());
    assert_eq!(files.width, stack.width, "the list takes the detail's room: {files:?} in {stack:?}");
}

#[test]
fn a_modal_floats_over_the_screen_and_keeps_the_focus() {
    let mut s = window(dialog(), 80, 14);
    assert!(!shows(&mut s, "ask"));
    s.keys(&["d"]);
    let (ask, stack) = (node(&mut s, "ask"), node(&mut s, "stack").rect.unwrap());
    assert_eq!(ask.title.as_deref(), Some("Delete?"));
    let r = ask.rect.unwrap();
    assert_eq!((r.width, r.height), (30, 5), "the float's size: {r:?}");
    assert!((2 * r.x + r.width).abs_diff(2 * stack.x + stack.width) <= 1, "across the middle: {r:?} in {stack:?}");
    assert!((2 * r.y + r.height).abs_diff(2 * stack.y + stack.height) <= 1, "down the middle: {r:?} in {stack:?}");
    assert!(shows(&mut s, "yes") && shows(&mut s, "no"));
    assert_eq!(s.state().focus.as_deref(), Some("yes"), "focus moves into the dialog");
    s.keys(&["tab"]);
    assert_eq!(s.state().focus.as_deref(), Some("no"));
    s.keys(&["tab"]);
    assert_eq!(s.state().focus.as_deref(), Some("yes"), "and stays there");
    // A press behind the dialog, on the list's second row, goes nowhere.
    let (x, y) = s.at("files", Some(1), None).expect("the list's rows are drawn under the scrim");
    assert!(!(r.x..r.x + r.width).contains(&x) || !(r.y..r.y + r.height).contains(&y), "outside the dialog");
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().focus.as_deref(), Some("yes"));
    assert_eq!(s.state().local["files"].selected, 0, "the row under the scrim wasn't chosen");
    // A press on the dialog's button does what it says.
    let (x, y) = s.at("yes", None, None).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert!(!shows(&mut s, "ask"));
    assert_eq!(s.state().toasts.last().map(|t| t.text.as_str()), Some("Deleted"), "the toast shows");
    assert_eq!(s.state().focus.as_deref(), Some("files"), "focus goes back");
}

#[test]
fn a_float_sits_where_it_says_with_its_margin() {
    let ui = json!({"root": {"type": "stack", "id": "s", "children": [
        {"type": "list", "id": "under", "items": rows(20)},
        {"type": "text", "id": "badge", "text": "3 new", "float": {"width": 10, "height": 2, "at": "top-right", "margin": 1}},
        {"type": "text", "id": "foot", "text": "bottom", "float": {"width": "50%", "height": 1, "at": "bottom-left"}}
    ]}});
    let mut s = window(ui, 60, 14);
    let (st, badge, foot) = (node(&mut s, "s").rect.unwrap(), node(&mut s, "badge").rect.unwrap(), node(&mut s, "foot").rect.unwrap());
    assert_eq!(badge, fictty::engine::Rect::new(st.x + st.width - 11, st.y + 1, 10, 2), "in {st:?}");
    assert_eq!(foot, fictty::engine::Rect::new(st.x, st.y + st.height - 1, st.width / 2, 1), "in {st:?}");
    // A float covers what's under it: a press on it doesn't reach the list.
    s.mouse("press", badge.x + 1, badge.y).mouse("release", badge.x + 1, badge.y);
    assert_eq!(s.state().local.get("under").map_or(0, |l| l.selected), 0);
}

#[test]
fn gaps_space_out_a_row() {
    let ui = json!({"root": {"type": "row", "gap": 3, "children": [
        {"type": "text", "id": "a", "text": "A", "size": 1}, {"type": "text", "id": "b", "text": "B", "size": 1}]}});
    let mut s = window(ui, 20, 3);
    let (a, b) = (node(&mut s, "a").rect.unwrap(), node(&mut s, "b").rect.unwrap());
    assert_eq!((a.width, b.x), (1, a.x + 4), "{a:?} {b:?}");
}

#[test]
fn min_and_max_bound_a_share() {
    let ui = json!({"root": {"type": "row", "id": "r", "children": [
        {"type": "text", "id": "l", "text": "L", "size": "60%", "max": 10, "border": true},
        {"type": "text", "id": "m", "text": "M", "min": 30, "border": true},
        {"type": "text", "id": "n", "text": "N", "border": true}]}});
    let mut s = window(ui, 60, 5);
    let (r, l, m, n) = (node(&mut s, "r").rect.unwrap(), node(&mut s, "l").rect.unwrap(), node(&mut s, "m").rect.unwrap(), node(&mut s, "n").rect.unwrap());
    assert_eq!(l.width, 10, "60% held to 10: {l:?}");
    assert!(m.width >= 30, "a share of what's left, at least 30: {m:?}");
    assert_eq!(l.width + m.width + n.width, r.width, "the rest is shared: {l:?} {m:?} {n:?} in {r:?}");
}

#[test]
fn a_size_can_be_in_pixels() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "row", "size": "48px", "children": [
            {"type": "text", "id": "a", "text": "A", "size": "84px"}, {"type": "text", "id": "b", "text": "B"}]},
        {"type": "text", "id": "c", "text": "C"}]}});
    let mut s = window(ui, 40, 8);
    let (a, b, c) = (node(&mut s, "a").rect.unwrap(), node(&mut s, "b").rect.unwrap(), node(&mut s, "c").rect.unwrap());
    // 84 px is ten columns of 8.4; 48 px two rows of 24.
    assert_eq!((a.width, a.height, b.x, c.y), (10, 2, a.x + 10, a.y + 2), "{a:?} {b:?} {c:?}");
}

#[test]
fn pad_takes_every_side_or_four_in_any_unit() {
    let ui = |pad: Value| json!({"root": {"type": "col", "children": [
        {"type": "col", "id": "box", "pad": pad, "border": true, "children": [{"type": "list", "id": "inner", "items": ["x"]}]}]}});
    let theme = json!({"space": {"wide": "84px", "tall": 2}});
    let was = node(&mut window(ui(json!(0)), 40, 12), "inner").rect.unwrap();
    // Top, right, bottom and left in, from where it was: columns of 8.4 px, rows of 24.
    for (pad, [t, r, b, l]) in [
        (json!(1), [1, 1, 1, 1]),
        (json!([3, 1]), [1, 3, 1, 3]),
        (json!([1, 2, 3, 4]), [1, 2, 3, 4]),
        (json!(["84px", "48px"]), [2, 10, 2, 10]),
        (json!(["tall", "wide", 0, "wide"]), [2, 10, 0, 10]),
    ] {
        let i = node(&mut themed(theme.clone(), ui(pad.clone()), 40, 12), "inner").rect.unwrap();
        assert_eq!((i.x, i.y, i.width, i.height), (was.x + l, was.y + t, was.width - l - r, was.height - t - b), "pad {pad}: {i:?}, from {was:?}");
    }
}

#[test]
fn a_margin_keeps_a_node_off_what_is_beside_it() {
    let ui = |margin: Value| json!({"root": {"type": "row", "id": "r", "children": [
        {"type": "text", "id": "a", "text": "A", "size": 10},
        {"type": "text", "id": "b", "text": "B", "size": 10, "margin": margin},
        {"type": "text", "id": "c", "text": "C", "size": 10}]}});
    for (margin, [t, r, b, l]) in [(json!(0), [0, 0, 0, 0]), (json!(1), [1, 1, 1, 1]), (json!([2, "16.8px", 0, "s"]), [2, 2, 0, 5])] {
        let mut s = themed(json!({"space": {"s": "42px"}}), ui(margin.clone()), 60, 6);
        let (row, a, bn, c) = (node(&mut s, "r").rect.unwrap(), node(&mut s, "a").rect.unwrap(), node(&mut s, "b").rect.unwrap(), node(&mut s, "c").rect.unwrap());
        assert_eq!((bn.x, bn.y, bn.height), (a.x + a.width + l, row.y + t, row.height - t - b), "margin {margin}: {bn:?} in {row:?}");
        assert_eq!(c.x, bn.x + bn.width + r, "margin {margin}: what's after it moves over too");
    }
}

#[test]
fn min_max_and_gap_take_any_unit() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "row", "id": "r", "children": [
            {"type": "text", "id": "l", "text": "L", "size": "60%", "max": "84px", "border": true},
            {"type": "text", "id": "m", "text": "M", "min": "252px", "border": true},
            {"type": "text", "id": "n", "text": "N", "border": true}]},
        {"type": "row", "gap": "s", "size": 1, "children": [
            {"type": "text", "id": "a", "text": "A", "size": 1}, {"type": "text", "id": "b", "text": "B", "size": 1}]},
        {"type": "col", "gap": [5, "48px"], "size": 4, "children": [
            {"type": "text", "id": "x", "text": "X", "size": 1}, {"type": "text", "id": "y", "text": "Y", "size": 1}]}]}});
    let mut s = themed(json!({"space": {"s": "84px"}}), ui, 60, 12);
    let (l, m) = (node(&mut s, "l").rect.unwrap(), node(&mut s, "m").rect.unwrap());
    assert_eq!(l.width, 10, "60% held to 84 px: {l:?}");
    assert!(m.width >= 30, "a share of what's left, at least 252 px: {m:?}");
    let (a, b) = (node(&mut s, "a").rect.unwrap(), node(&mut s, "b").rect.unwrap());
    assert_eq!(b.x, a.x + 1 + 10, "a gap named in the theme's space: {a:?} {b:?}");
    let (x, y) = (node(&mut s, "x").rect.unwrap(), node(&mut s, "y").rect.unwrap());
    assert_eq!(y.y, x.y + 1 + 2, "a column's gap is the [x, y]'s y: {x:?} {y:?}");
}

#[test]
fn justify_puts_children_along_a_row_in_the_room_they_leave() {
    let ui = |justify: &str| json!({"root": {"type": "row", "id": "r", "justify": justify, "children": [
        {"type": "text", "id": "a", "text": "A", "size": 10},
        {"type": "text", "id": "b", "text": "B", "size": 10},
        {"type": "text", "id": "c", "text": "C", "size": 10}]}});
    // Thirty cells of children in a row of sixty leave thirty.
    for (justify, [a, b, c]) in [("start", [0, 10, 20]), ("center", [15, 25, 35]), ("end", [30, 40, 50]), ("between", [0, 25, 50]), ("around", [5, 25, 45])] {
        let mut s = window(ui(justify), 62, 3);
        let r = node(&mut s, "r").rect.unwrap();
        assert_eq!(r.width, 60);
        let xs = ["a", "b", "c"].map(|id| node(&mut s, id).rect.unwrap().x - r.x);
        assert_eq!(xs, [a, b, c], "justify {justify}");
    }
}

#[test]
fn items_puts_children_across_a_row_and_self_overrides_it() {
    let ui = |items: &str| json!({"root": {"type": "row", "id": "r", "items": items, "children": [
        {"type": "text", "id": "a", "text": "A", "size": 10},
        {"type": "text", "id": "b", "text": "B", "size": 10, "self": "end"},
        {"type": "text", "id": "c", "text": "C", "size": 10, "self": "stretch"}]}});
    for (items, a_y, a_h) in [("start", 0, 1), ("center", 3, 1), ("end", 6, 1), ("stretch", 0, 7)] {
        let mut s = window(ui(items), 30, 10);
        let r = node(&mut s, "r").rect.unwrap();
        assert_eq!(r.height, 7);
        let (a, b, c) = (node(&mut s, "a").rect.unwrap(), node(&mut s, "b").rect.unwrap(), node(&mut s, "c").rect.unwrap());
        assert_eq!((a.y - r.y, a.height), (a_y, a_h), "items {items}: {a:?} in {r:?}");
        assert_eq!((b.y - r.y, b.height), (6, 1), "items {items}: self end sits at the bottom: {b:?}");
        assert_eq!((c.y, c.height), (r.y, r.height), "items {items}: self stretch fills it: {c:?}");
    }
}

#[test]
fn a_column_centres_its_children_both_ways() {
    let ui = json!({"root": {"type": "col", "id": "c", "justify": "center", "items": "center", "children": [
        {"type": "text", "id": "t", "text": "middle", "size": 2, "max": 20}]}});
    let mut s = window(ui, 40, 11);
    let (c, t) = (node(&mut s, "c").rect.unwrap(), node(&mut s, "t").rect.unwrap());
    assert_eq!((t.y - c.y, t.height), ((c.height - 2) / 2, 2), "down the middle: {t:?} in {c:?}");
    assert_eq!(t.x - c.x + t.width / 2, c.width / 2, "across the middle: {t:?} in {c:?}");
}

#[test]
fn an_each_keeps_its_justify_and_items() {
    let ui = json!({"store": {"xs": [1, 2]}, "root": {"type": "each", "id": "e", "bind": "xs", "row": true, "justify": "end", "items": "end",
        "template": {"type": "text", "id": "x{$item}", "text": "{$item}", "size": 5}}});
    let mut s = window(ui, 30, 7);
    let (e, x1, x2) = (node(&mut s, "e").rect.unwrap(), node(&mut s, "x1").rect.unwrap(), node(&mut s, "x2").rect.unwrap());
    assert_eq!((e.x + e.width - x1.x, x2.x - x1.x), (10, 5), "at the row's end: {x1:?} {x2:?} in {e:?}");
    assert_eq!((e.y + e.height - x1.y, x1.height), (1, 1), "at its bottom: {x1:?} in {e:?}");
}

#[test]
fn an_aspect_gives_a_node_its_length_from_how_big_it_is_across() {
    // Columns are 8.4 px and rows 24: sixty columns at 7/2 are six rows; five rows at 1.4 are
    // twenty columns.
    let ui = json!({"root": {"type": "col", "id": "c", "children": [
        {"type": "text", "id": "wide", "text": "W", "aspect": "7/2"},
        {"type": "row", "id": "r", "size": 5, "children": [
            {"type": "text", "id": "tall", "text": "T", "aspect": 1.4},
            {"type": "text", "id": "rest", "text": "R"}]},
        {"type": "text", "id": "after", "text": "A"}]}});
    let mut s = window(ui, 62, 16);
    let (c, wide, r, tall, rest) = ["c", "wide", "r", "tall", "rest"].map(|id| node(&mut s, id).rect.unwrap()).into();
    assert_eq!(c.width, 60);
    assert_eq!((wide.width, wide.height, r.y), (60, 6, wide.y + 6), "as tall as its width makes it: {wide:?}");
    assert_eq!((tall.width, tall.height, rest.x), (20, 5, tall.x + 20), "as wide as its height makes it: {tall:?} {rest:?}");
}

#[test]
fn an_aspect_takes_the_side_it_is_given_less_its_margin() {
    let ui = json!({"root": {"type": "col", "id": "c", "children": [
        {"type": "text", "id": "m", "text": "M", "aspect": "7/2", "margin": [5, 0]},
        {"type": "text", "id": "z", "text": "Z", "aspect": "7/2", "size": 3, "self": "center"},
        {"type": "text", "id": "after", "text": "A"}]}});
    let mut s = window(ui, 62, 20);
    let [c, m, z, after] = ["c", "m", "z", "after"].map(|id| node(&mut s, id).rect.unwrap());
    assert_eq!((m.x - c.x, m.width, m.height), (5, 50, 5), "fifty columns across at 7/2 are five rows: {m:?}");
    // Three rows are 72 px, at 7/2 252 px: thirty columns, in the middle.
    assert_eq!((z.y, z.width, z.height, z.x - c.x), (m.y + 5, 30, 3, 15), "its size, and as wide as that makes it: {z:?}");
    assert_eq!(after.y, z.y + 3);
}

#[test]
fn a_floats_margin_takes_four_sides() {
    let ui = json!({"root": {"type": "stack", "id": "s", "children": [
        {"type": "list", "id": "under", "items": rows(20)},
        {"type": "text", "id": "badge", "text": "3 new", "float": {"width": 10, "height": 2, "at": "top-right", "margin": [2, "84px", 0, 0]}},
        {"type": "text", "id": "foot", "text": "bottom", "float": {"width": 10, "height": 1, "at": "bottom-left", "margin": [0, 0, 1, 3]}}
    ]}});
    let mut s = window(ui, 60, 14);
    let (st, badge, foot) = (node(&mut s, "s").rect.unwrap(), node(&mut s, "badge").rect.unwrap(), node(&mut s, "foot").rect.unwrap());
    assert_eq!((badge.x, badge.y), (st.x + st.width - 10 - 10, st.y + 2), "two rows down, 84 px in: {badge:?} in {st:?}");
    assert_eq!((foot.x, foot.y), (st.x + 3, st.y + st.height - 1 - 1), "{foot:?} in {st:?}");
}

#[test]
fn a_length_that_isnt_one_is_a_problem() {
    let ui = json!({"root": {"type": "col", "id": "c", "gap": "s9", "children": [
        {"type": "text", "id": "t", "text": "T", "pad": [1, "8pz"], "margin": "wide", "min": "s", "max": "huge",
         "float": {"margin": "far"}}]}});
    let mut s = themed(json!({"space": {"s": "8px"}}), ui, 40, 6);
    let problems = s.state().problems.join("\n");
    for p in ["c: gap \"s9\"", "t: pad \"8pz\"", "t: margin \"wide\"", "t: max \"huge\"", "t: float margin \"far\""] {
        assert!(problems.contains(&format!("{p} isn't a length or one of the theme's space names")), "{p} in:\n{problems}");
    }
    assert!(!problems.contains("min"), "a space name the theme has is a length: {problems}");
}

#[test]
fn a_font_file_that_doesnt_load_is_a_problem() {
    let ui = json!({"root": {"type": "text", "id": "t", "text": "T", "style": {"type": "display"}}});
    let theme = json!({"fonts": {"gone": "/no/such/dir/Gone.ttf"}, "type": {"display": {"font": "gone", "size": 30}}});
    let mut s = themed(theme, ui, 40, 4);
    let problems = s.state().problems.clone();
    assert!(problems.iter().any(|p| p.starts_with("theme t: font gone: /no/such/dir/Gone.ttf didn't load (")), "{problems:?}");
    assert_eq!(node(&mut s, "t").text, "T", "the text still shows, in the default font");
}

#[test]
fn pad_and_align_place_what_a_node_holds() {
    let ui = |pad: Value| json!({"root": {"type": "col", "children": [
        {"type": "col", "id": "box", "pad": pad, "border": true, "children": [{"type": "list", "id": "inner", "items": ["x"]}]},
        {"type": "button", "id": "go", "label": "Go", "align": "right", "size": 1},
        {"type": "button", "id": "mid", "label": "Mid", "align": "center", "size": 1}]}});
    let mut plain = window(ui(json!(0)), 40, 12);
    let mut s = window(ui(json!([3, 1])), 40, 12);
    let (was, i) = (node(&mut plain, "inner").rect.unwrap(), node(&mut s, "inner").rect.unwrap());
    // A border in the window is a line, not a cell: pad moves what's inside in from where it was.
    assert_eq!((i.x, i.y, i.width, i.height), (was.x + 3, was.y + 1, was.width - 6, was.height - 2), "three columns and a row in: {i:?}, from {was:?}");
    // A button's hit is its face: right-aligned, the face ends at the node's right edge; centred,
    // it sits in the middle.
    let (go, (gx, _)) = (node(&mut s, "go").rect.unwrap(), s.at("go", None, None).unwrap());
    assert!(gx > go.x + go.width / 2, "the face is on the right: {gx} in {go:?}");
    let (mid, (mx, _)) = (node(&mut s, "mid").rect.unwrap(), s.at("mid", None, None).unwrap());
    assert!(mx.abs_diff(mid.x + mid.width / 2) <= 1, "the face is in the middle: {mx} in {mid:?}");
}

#[test]
fn when_can_read_the_width_a_node_is_laid_out_in() {
    let ui = json!({
        "root": {"type": "row", "children": [
            {"type": "text", "text": "SIDEBAR", "size": 50, "when": "aside"},
            {"type": "col", "children": [
                {"type": "list", "id": "wide", "items": ["WIDE TABLE"], "when": "$width>=60"},
                {"type": "list", "id": "narrow", "items": ["NARROW LIST"], "when": "$width<60"}
            ]}
        ]},
        "store": {"aside": true},
        "keys": {"x": {"put": "aside", "value": false}}
    });
    let mut s = window(ui, 100, 6);
    assert!(shows(&mut s, "narrow") && !shows(&mut s, "wide"), "beside the sidebar");
    assert_eq!(s.state().focus.as_deref(), Some("narrow"), "focus goes where it shows");
    s.keys(&["x"]);
    assert!(shows(&mut s, "wide") && !shows(&mut s, "narrow"), "alone");
    assert_eq!(s.state().focus.as_deref(), Some("wide"));
}

#[test]
fn each_repeats_a_template_per_item() {
    let ui = json!({
        "root": {"type": "each", "id": "panels", "bind": "open", "template": {
            "type": "editor", "id": "ed-{$item.id}", "title": "{$item.title}", "doc": "page/{$item.id}",
            "text": "{$item.title} notes", "size": "{$item.h}"}},
        "store": {"open": [{"id": "a", "title": "Alpha", "h": 4}, {"id": "b", "title": "Beta", "h": 4}]}
    });
    let mut s = window(ui, 30, 14);
    let (a, b) = (node(&mut s, "ed-a"), node(&mut s, "ed-b"));
    assert_eq!((a.title.as_deref(), a.text.as_str()), (Some("Alpha"), "Alpha notes"));
    assert_eq!((b.title.as_deref(), b.text.as_str()), (Some("Beta"), "Beta notes"));
    assert_eq!((a.rect.unwrap().height, b.rect.unwrap().y), (4, a.rect.unwrap().y + 4), "each its size, one under the other");
    assert_eq!(s.state().focus.as_deref(), Some("ed-a"));
    s.keys(&["tab", "x"]);
    assert_eq!(s.state().store["page/b"].read()["text"], "xBeta notes", "the second instance has the focus");
    // The template's own placeholders are left for it: ui keeps the template as pushed.
    assert_eq!(s.state().ui.root.kind_name(), "each");
    // A new item, a new instance.
    let open = json!([{"id": "a", "title": "Alpha", "h": 3}, {"id": "b", "title": "Beta", "h": 3}, {"id": "c", "title": "Gamma", "h": 3}]);
    s.msg(Msg::Write { key: "open".into(), write: serde_json::from_value(json!({"write": "put", "value": open})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
    assert_eq!(node(&mut s, "ed-c").title.as_deref(), Some("Gamma"));
    assert_eq!(s.state().focus.as_deref(), Some("ed-b"), "focus stays on its instance");
}

#[test]
fn each_follows_its_data_and_keeps_its_tree_otherwise() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "each", "id": "e", "bind": "open", "template": {"type": "text", "id": "t-{$item}", "text": "{$item}!", "size": 1}},
        {"type": "list", "id": "l", "items": ["x", "y"]}]},
        "store": {"open": ["a"]}});
    let mut s = window(ui, 20, 6);
    assert_eq!(node(&mut s, "t-a").text, "a!");
    s.keys(&["j"]);
    assert!(shows(&mut s, "t-a"), "a key that changes nothing it reads leaves it");
    s.msg(Msg::Write { key: "open".into(), write: serde_json::from_value(json!({"write": "put", "value": ["a", "b"]})).unwrap(), if_rev: None, by: "t".into(), for_ms: None });
    assert_eq!(node(&mut s, "t-b").text, "b!");
    let (a, b) = (node(&mut s, "t-a").rect.unwrap(), node(&mut s, "t-b").rect.unwrap());
    assert_eq!(b.y, a.y + 1, "{a:?} {b:?}");
}

#[test]
fn the_readback_has_each_nodes_rectangle_from_the_window() {
    let ui = json!({"root": {"type": "row", "id": "r", "children": [
        {"type": "input", "id": "name", "size": 20},
        {"type": "button", "id": "go", "label": "Go"}
    ]}, "focus": "name"});
    let mut s = window(ui, 40, 4);
    s.keys(&["h", "i"]);
    let nodes = s.readback();
    let name = nodes.iter().find(|n| n.id.as_deref() == Some("name")).unwrap();
    assert_eq!((name.text.as_str(), name.focus), ("hi", true));
    let (r, name, go) = (node(&mut s, "r").rect.unwrap(), name.rect.unwrap(), node(&mut s, "go").rect.unwrap());
    assert_eq!((name.x, name.y, name.width), (r.x, r.y, 20), "{name:?} in {r:?}");
    assert_eq!((go.x, go.width), (r.x + 20, r.width - 20), "{go:?} in {r:?}");
    // The socket's op answers the same.
    let back = s.request(json!({"op": "readback"}));
    assert_eq!(back["nodes"][1]["id"], "name");
}

#[test]
fn the_window_takes_the_mouse_and_resizes() {
    let ui = json!({
        "root": {"type": "col", "children": [
            {"type": "list", "id": "l", "items": ["one", "two", "three"], "min": 3},
            {"type": "editor", "id": "e", "text": "hello world"}
        ]},
        "focus": "l"
    });
    let mut s = window(ui, 30, 10);
    // A press on a row selects it.
    let (x, y) = s.at("l", Some(2), None).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().local["l"].selected, 2);
    // A drag across the editor's text selects what it crossed.
    let x0 = text_start(&mut s, "e");
    let y = s.text_at("e").unwrap().y;
    s.mouse("press", x0, y).mouse("drag", x0 + 5, y).mouse("release", x0 + 5, y);
    assert_eq!(s.state().focus.as_deref(), Some("e"));
    assert_eq!(marked(&mut s, "e"), "⟦hello▮⟧ world");
    // A paste replaces the selection; a resize lays it out again at the new size.
    s.paste("HOWDY");
    assert_eq!(node(&mut s, "e").text, "HOWDY world");
    let before = node(&mut s, "e").rect.unwrap();
    s.resize(20, 8);
    assert_eq!(s.size(), (20, 8));
    let after = node(&mut s, "e").rect.unwrap();
    assert!(after.width < before.width && after.height < before.height, "{before:?} → {after:?}");
}

#[test]
fn keys_type_into_a_field_a_key_just_showed() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "input", "id": "find", "size": 1, "when": "finding"},
        {"type": "list", "id": "l", "items": ["a"]}]},
        "focus": "l", "store": {"finding": false},
        "keys": {"/": {"do": [{"put": "finding", "value": true}, {"focus": "find"}]}}});
    let mut s = window(ui, 30, 5);
    s.keys(&["/", "m", "q", "4"]);
    assert_eq!(node(&mut s, "find").text, "mq4");
}

/// A screen in the window under one theme.
fn themed(theme: Value, ui: Value, w: u16, h: u16) -> fictty::Headless {
    let theme: fictty::model::ThemeSpec = serde_json::from_value(theme).unwrap();
    App::new(serde_json::from_value(ui).unwrap()).themes([("t".to_string(), theme)].into()).in_window(w, h).unwrap()
}

#[test]
fn the_mouse_over_a_button_holds_its_hover_part() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "list", "id": "l", "items": ["a"], "size": 2},
        {"type": "button", "id": "go", "label": "Go", "size": 1}
    ]}});
    let mut s = themed(json!({"tokens": {"lit": "#ff8800"}, "parts": {"button.hover": {"bg": "lit"}}}), ui, 40, 8);
    assert_eq!(node(&mut s, "go").style.ink.bg.as_deref(), Some("#1d3b3a"), "the default button, not hovered");
    let (x, y) = s.at("go", None, None).unwrap();
    s.mouse("move", x, y);
    assert_eq!(s.state().hover.as_ref().map(|h| (h.id.as_str(), h.row)), Some(("go", None)), "the window tells the engine a button is hovered, not only rows");
    let go = node(&mut s, "go");
    assert_eq!((go.style.states.clone(), go.style.ink.bg.as_deref()), (vec!["hover".to_string(), "blur".to_string()], Some("#ff8800")));
    s.mouse("move", 39, 7);
    assert_eq!(s.state().hover, None, "and when it leaves");
}

#[test]
fn a_parts_pad_widens_a_buttons_face_and_its_press() {
    let ui = json!({"root": {"type": "col", "children": [{"type": "button", "id": "go", "label": "Go", "size": 1}]}});
    let middle = |pad: &str| {
        let mut s = themed(json!({"parts": {"button": {"pad": [pad, 0]}}}), ui.clone(), 40, 4);
        s.at("go", None, None).unwrap().0
    };
    let (narrow, wide) = (middle("4px"), middle("60px"));
    assert!(wide >= narrow + 6, "60px a side moves the face's middle out ({narrow} → {wide})");
    let mut s = themed(json!({"parts": {"button": {"pad": ["60px", 0]}}}), ui.clone(), 40, 4);
    let (x, y) = s.at("go", None, None).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().focus.as_deref(), Some("go"), "the press lands on the wider face");
}

#[test]
fn tabs_take_the_shape_their_part_gives_them_and_still_take_clicks() {
    for shape in ["pill", "underline", "segment", "plain"] {
        let ui = json!({"root": {"type": "tabs", "id": "t", "tabs": ["One", "Two"], "children": [{"type": "text", "text": "1"}, {"type": "text", "text": "2"}]}});
        let mut s = themed(json!({"parts": {"tabs": {"shape": shape}}}), ui, 40, 6);
        assert_eq!(node(&mut s, "t").style.shape.as_deref(), Some(shape));
        let (x, y) = s.at("t", None, Some(1)).unwrap_or_else(|| panic!("{shape}: the second tab is drawn"));
        s.mouse("press", x, y).mouse("release", x, y);
        assert_eq!(node(&mut s, "t").selected, Some(1), "{shape}: a click picks the tab");
    }
}

#[test]
fn a_cut_line_ends_in_an_ellipsis_and_the_readback_says_so() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "list", "id": "long", "size": 2, "item": "{t}\t{when}", "items": [{"t": "a title far too long to fit in thirty cells", "when": "Tue · tilde"}]},
        {"type": "list", "id": "short", "size": 2, "item": "{t}\t{when}", "items": [{"t": "fits", "when": "Tue"}]}
    ]}});
    let mut s = window(ui, 30, 6);
    assert!(node(&mut s, "long").cut, "the long row is cut");
    assert!(!node(&mut s, "short").cut, "the short one isn't");
    let screen = s.screen();
    assert!(screen.contains("list long (focus) (cut)"), "the outline says so: {screen}");
    assert!(screen.contains("thirty cells   Tue · tilde"), "a tab reads as a gap: {screen}");
    s.resize(120, 6);
    assert!(!node(&mut s, "long").cut, "with room, nothing is cut");
}

#[test]
fn keep_holds_room_for_a_rows_left_so_its_meta_is_cut_too() {
    let row = |keep: Option<u16>| {
        let mut l = json!({"type": "list", "id": "l", "item": "{t}\t{meta}", "items": [{"t": "Water the plants", "meta": "Tue 14:00 · tilde · from the agent, by the window"}]});
        if let Some(k) = keep {
            l["keep"] = json!(k);
        }
        let mut s = window(json!({"root": l}), 30, 3);
        node(&mut s, "l").cut
    };
    assert!(row(None) && row(Some(12)), "too long either way: cut");
}

#[test]
fn wrap_false_and_lines_keep_text_to_its_lines_and_cut_it() {
    let long = "a sentence long enough to need a second line at thirty cells, and a third";
    let text = |extra: Value| {
        let mut t = json!({"type": "text", "id": "t", "text": long});
        t.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        let mut s = window(json!({"root": t}), 30, 6);
        node(&mut s, "t").cut
    };
    assert!(!text(json!({})), "text wraps by default, so nothing is cut");
    assert!(text(json!({"wrap": false})), "on one line, it's cut");
    assert!(text(json!({"lines": 1})), "one line shown, the rest cut");
}

#[test]
fn a_cut_shows_in_the_readback_without_an_id() {
    let long = "a title far too long for twenty cells";
    let mut s = window(
        json!({"root": {"type": "col", "id": "c", "children": [
            {"type": "text", "text": "short"},
            {"type": "tabs", "tabs": ["One"], "children": [{"type": "row", "children": [{"type": "text", "text": long, "wrap": false}]}]},
        ]}}),
        20,
        6,
    );
    let cut: Vec<(String, bool)> = s.readback().into_iter().filter(|n| n.kind == "text").map(|n| (n.text, n.cut)).collect();
    assert_eq!(cut, [("short".to_string(), false), (long.to_string(), true)], "named by where they sit, inside tabs too");
    assert!(s.screen().contains("(cut)"), "the outline says so");
}

#[test]
fn a_type_role_sets_a_nodes_text_and_a_bigger_one_takes_more_room() {
    let theme = json!({"type": {"display": {"size": 40, "weight": 800, "line": 48}, "body": {"size": 14}}, "fonts": {"ui": "Helvetica"}});
    let cut = |role: &str| {
        let ui = json!({"root": {"type": "text", "id": "t", "wrap": false, "style": {"type": role}, "text": "Kestrel Cycles"}});
        let mut s = themed(theme.clone(), ui, 30, 4);
        let t = node(&mut s, "t");
        (t.style.role.map(|r| (r.name, r.role.size)), t.cut)
    };
    assert_eq!(cut("body"), (Some(("body".to_string(), Some(14.))), false), "fourteen pixels fit in thirty cells");
    assert_eq!(cut("display"), (Some(("display".to_string(), Some(40.))), true), "forty don't");
    // Markup names a role for its line.
    let ui = json!({"root": {"type": "text", "id": "t", "wrap": false, "text": "<display>Kestrel Cycles</>"}});
    let mut s = themed(theme, ui, 30, 4);
    assert!(node(&mut s, "t").cut, "<display> sizes its line");
    assert!(s.state().problems.is_empty(), "a role is a known word in a spec: {:?}", s.state().problems);
}

#[test]
fn tab_titles_are_templates_with_a_lead_and_a_trail() {
    let ui = json!({"root": {"type": "tabs", "id": "t", "bind": "counts", "tabs": ["Inbox {inbox}", "Done"], "lead": "<b>thc</>", "trail": "Sat 11",
                              "children": [{"type": "text", "text": "one"}, {"type": "text", "text": "two"}]},
                    "data": {"counts": {"value": {"inbox": 4}}}});
    let mut s = window(ui, 40, 4);
    assert_eq!(node(&mut s, "t").text, "thc · Inbox 4 | Done · Sat 11");
    let (x, y) = s.at("t", None, Some(1)).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(node(&mut s, "t").selected, Some(1), "the tabs still take clicks beside a lead");
}

#[test]
fn tabs_start_where_they_say_and_choosing_one_moves_the_focus_into_it() {
    let ui = json!({"root": {"type": "tabs", "id": "views", "tab": 1, "tabs": ["Today", "Inbox <b>{n}</>"], "bind": "counts", "children": [
        {"type": "list", "id": "today", "items": ["a"]},
        {"type": "list", "id": "inbox", "items": ["x", "y"]}
    ]}, "keys": {"1": {"tab": 0, "of": "views"}, "2": {"tab": 1, "of": "views"}},
    "data": {"counts": {"value": {"n": 2}}}});
    let mut s = window(ui, 30, 5);
    assert_eq!((node(&mut s, "views").selected, s.state().focus.as_deref()), (Some(1), Some("inbox")), "starts on Inbox, in its content");
    s.keys(&["1"]);
    assert_eq!(s.state().focus.as_deref(), Some("today"));
    let (x, y) = s.at("views", None, Some(1)).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().focus.as_deref(), Some("inbox"), "a click on a tab moves the focus into it too");
}

#[test]
fn fit_makes_a_node_as_wide_as_what_it_shows_and_gives_way_when_there_isnt_room() {
    let ui = json!({"root": {"type": "row", "id": "r", "children": [
        {"type": "text", "id": "t", "text": "twelve chars", "size": "fit"},
        {"type": "text", "id": "cut", "text": "twenty letters here.", "size": "fit", "wrap": false},
        {"type": "button", "id": "b", "label": "Save", "size": "fit"},
        {"type": "text", "id": "boxed", "text": "x", "title": "a long title", "size": "fit", "border": true},
        {"type": "text", "id": "rest", "text": "rest"}]}});
    for w in [30u16, 40, 50, 60, 70, 80, 100] {
        let mut s = window(ui.clone(), w, 6);
        let [r, t, cut, b, boxed, rest] = ["r", "t", "cut", "b", "boxed", "rest"].map(|id| node(&mut s, id));
        let [r, t, c, b, x, rest] = [r.rect, t.rect, cut.rect, b.rect, boxed.rect, rest.rect].map(Option::unwrap);
        // The texts' widths, the button's face (its label and padding) and the box's title
        // inside its border: 12 + 20 + 7 + 14 = 53 columns.
        if r.width >= 53 {
            assert_eq!((t.width, c.width, b.width, x.width), (12, 20, 7, 14), "at {w}: as wide as they show");
            assert!(!cut.cut, "at {w}: whole while there's room");
            assert_eq!(rest.width, r.width - 53, "at {w}: a share takes what's left");
        } else {
            assert!(cut.cut && t.width + c.width + b.width + x.width <= r.width, "at {w}: they give way: {t:?} {c:?} {b:?} {x:?} in {r:?}");
        }
    }
}

#[test]
fn a_fit_paragraph_is_as_tall_as_it_wraps_to() {
    let text = "a paragraph that wraps over a few lines when the column is narrow enough to need it";
    let mut was = u16::MAX;
    for w in [20u16, 30, 44, 60, 100] {
        let ui = json!({"root": {"type": "col", "children": [{"type": "text", "id": "p", "text": text, "size": "fit"}, {"type": "text", "id": "after", "text": "A"}]}});
        let mut s = window(ui, w, 12);
        let (p, after) = (node(&mut s, "p").rect.unwrap(), node(&mut s, "after").rect.unwrap());
        assert!(p.height >= 1 && p.height <= was, "at {w}: no taller than narrower: {p:?}");
        assert_eq!(after.y, p.y + p.height, "at {w}: what follows starts below it");
        if w == 20 {
            assert!(p.height >= 4, "narrow, it wraps: {p:?}");
        }
        if w == 100 {
            assert_eq!(p.height, 1, "with room, one line");
        }
        was = p.height;
    }
}

#[test]
fn a_fit_list_is_as_tall_as_its_rows_up_to_its_max() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "list", "id": "three", "items": rows(3), "size": "fit"},
        {"type": "list", "id": "titled", "items": rows(3), "size": "fit", "title": "three", "border": true},
        {"type": "list", "id": "long", "items": rows(50), "size": "fit", "max": 4},
        {"type": "text", "id": "after", "text": "A"}]}});
    let mut s = window(ui, 40, 30);
    let [three, titled, long, after] = ["three", "titled", "long", "after"].map(|id| node(&mut s, id));
    let [a, b, c] = [three.rect.unwrap(), titled.rect.unwrap(), long.rect.unwrap()];
    // A title and a border take two rows between them.
    assert_eq!((a.height, b.height, c.height, after.rect.unwrap().y), (3, 5, 4, c.y + 4), "{a:?} {b:?} {c:?}");
    assert_eq!(long.scroll, Some((0., 50)), "a list held to its max scrolls");
}

#[test]
fn a_fit_editor_is_as_tall_as_its_lines_and_grows_with_them() {
    let text = "word ".repeat(30);
    // Thirty words of five wrap to six rows of a 26-column editor (a column in either side of
    // 28), five words a row, and three of 56, eleven a row.
    for (w, rows) in [(30u16, 6), (60, 3)] {
        let ui = json!({"root": {"type": "col", "children": [
            {"type": "editor", "id": "e", "text": text.trim_end(), "size": "fit"}, {"type": "text", "id": "after", "text": "A"}]}, "focus": "e"});
        let mut s = window(ui, w, 20);
        assert_eq!(node(&mut s, "e").rect.unwrap().height, rows, "at {w}");
    }
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "editor", "id": "e", "text": "one", "size": "fit"}, {"type": "text", "id": "after", "text": "A"}]}, "focus": "e"});
    let mut s = window(ui, 40, 20);
    assert_eq!(node(&mut s, "e").rect.unwrap().height, 1);
    s.keys(&["end", "enter", "t"]);
    let (e, after) = (node(&mut s, "e").rect.unwrap(), node(&mut s, "after").rect.unwrap());
    assert_eq!((e.height, after.y), (2, e.y + 2), "a new line makes it a row taller");
}
