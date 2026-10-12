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
    assert_eq!(s.state().local["l"].top, Some(9), "three notches, three rows each");
    assert_eq!(s.state().local["l"].selected, 0, "the selection stays");
    // The rows drawn are the ones the state says: a press lands on row 9 + 1.
    s.mouse("press", l.x + 2, l.y + 1).mouse("release", l.x + 2, l.y + 1);
    assert_eq!(s.state().local["l"].selected, 10);
    s.mouse("wheel-up", l.x + 2, l.y + 2);
    assert_eq!(s.state().local["l"].top, Some(6));
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
