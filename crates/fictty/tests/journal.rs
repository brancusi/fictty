//! Daybook (`examples/journal/`), a journal written as data, in the native window: the flows a
//! person runs every day, as macOS keystrokes, read back as an agent would. Today is fixed here
//! (the example asks `date`).
#![cfg(target_os = "macos")]

use fictty::App;
use fictty::state::Msg;
use serde_json::{Value, json};

fn journal() -> fictty::Headless {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/journal/journal.json");
    let mut ui: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    ui["data"]["today"] = json!({"value": {"id": "2026-10-13", "short": "Tue 13 Oct", "long": "Tuesday, October 13 2026"}});
    App::new(serde_json::from_value(ui).unwrap()).in_window(140, 30).unwrap()
}

fn value(s: &mut fictty::Headless, key: &str) -> Value {
    s.state().store[key].read()["value"].clone()
}

fn text(s: &mut fictty::Headless, key: &str) -> String {
    s.state().store.get(key).map(|e| e.read()["text"].as_str().unwrap_or_default().to_string()).unwrap_or_default()
}

/// The open entry's text with `▮` at the caret.
fn caret(s: &mut fictty::Headless) -> String {
    let st = s.state();
    let here = &st.store["here"].read()["value"];
    let key = format!("{}/{}", here["kind"].as_str().unwrap(), here["id"].as_str().unwrap());
    let r = st.local["page"].editors[&key].selection.primary();
    let mut t = text(s, &key);
    let at = t.char_indices().nth(r.head).map_or(t.len(), |(i, _)| i);
    t.insert(at, '▮');
    t
}

fn ids(s: &mut fictty::Headless, key: &str) -> Vec<String> {
    value(s, key).as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_string()).collect()
}

/// The id of the row list `id` has selected.
fn selected(s: &mut fictty::Headless, id: &str) -> String {
    let st = s.state();
    let at = st.local.get(id).map_or(0, |l| l.selected);
    st.store[id].read()["value"][at]["id"].as_str().unwrap().to_string()
}

fn shown(s: &mut fictty::Headless, id: &str) -> bool {
    s.readback().iter().any(|n| n.id.as_deref() == Some(id))
}

#[test]
fn today_opens_while_writing_and_is_listed_once() {
    let mut s = journal();
    assert_eq!(s.state().focus.as_deref(), Some("page"), "it opens writing");
    s.keys(&["d-t"]);
    assert_eq!(value(&mut s, "here")["id"], "2026-10-13");
    s.keys(&["H", "i"]);
    assert_eq!(text(&mut s, "day/2026-10-13"), "Hi", "typing goes into today");
    assert!(!text(&mut s, "page/Welcome").contains("Hi"), "⌘T didn't type into the page it left");
    s.keys(&["d-t"]);
    assert_eq!(ids(&mut s, "days"), ["2026-10-11", "2026-10-12", "2026-10-13"], "today once");
    assert_eq!(selected(&mut s, "days"), "2026-10-13");
    assert_eq!(caret(&mut s), "Hi▮", "back where it was");
}

#[test]
fn the_day_before_and_after() {
    let mut s = journal();
    s.keys(&["d-t", "a-["]);
    assert_eq!(value(&mut s, "here")["id"], "2026-10-12");
    assert_eq!(selected(&mut s, "days"), "2026-10-12", "the list follows the open day");
    assert_eq!(s.state().focus.as_deref(), Some("page"));
    s.keys(&["a-["]);
    assert_eq!(value(&mut s, "here")["id"], "2026-10-11");
    s.keys(&["a-]", "a-]"]);
    assert_eq!(value(&mut s, "here")["id"], "2026-10-13");
    assert_eq!(selected(&mut s, "days"), "2026-10-13");
}

#[test]
fn a_new_page_from_anywhere() {
    let mut s = journal();
    s.keys(&["d-n"]);
    assert!(shown(&mut s, "newname"));
    assert_eq!(s.state().focus.as_deref(), Some("newname"));
    s.keys(&["P", "l", "a", "n", "s", "enter"]);
    assert!(!shown(&mut s, "newname"), "the box goes");
    assert_eq!(ids(&mut s, "pages"), ["Welcome", "Reading list", "Garden", "Plans"]);
    assert_eq!(selected(&mut s, "pages"), "Plans");
    assert_eq!(s.state().focus.as_deref(), Some("page"), "straight into writing");
    s.keys(&["x"]);
    assert_eq!(text(&mut s, "page/Plans"), "x");
    // A blank name makes no page.
    s.keys(&["d-n", "space", "enter"]);
    assert!(shown(&mut s, "newname"), "still asking");
    assert_eq!(ids(&mut s, "pages").len(), 4);
    s.keys(&["esc"]);
    // Esc lets it go, back to where it was.
    s.keys(&["d-n", "Q", "esc"]);
    assert!(!shown(&mut s, "newname"));
    assert_eq!(s.state().focus.as_deref(), Some("page"));
    assert_eq!(ids(&mut s, "pages").len(), 4);
}

#[test]
fn sections_by_number_and_back_to_writing() {
    let mut s = journal();
    s.keys(&["d-2"]);
    assert_eq!(s.state().focus.as_deref(), Some("pages"));
    s.keys(&["down", "down", "enter"]);
    assert_eq!(value(&mut s, "here")["id"], "Garden");
    assert_eq!(s.state().focus.as_deref(), Some("page"));
    assert!(caret(&mut s).ends_with("[[Reading list]].\n▮"), "the caret at the end: {}", caret(&mut s));
    s.keys(&["d-1"]);
    assert_eq!(s.state().focus.as_deref(), Some("days"));
    s.keys(&["enter"]);
    assert_eq!(value(&mut s, "here")["id"], "2026-10-11");
}

#[test]
fn zen_leaves_only_the_writing() {
    let mut s = journal();
    s.keys(&["d-t", "H", "i"]);
    s.keys(&["a-z"]);
    assert!(!shown(&mut s, "side"), "the sidebar goes");
    assert!(!s.readback().iter().any(|n| n.text.contains('§')), "the heading goes");
    assert_eq!(s.state().focus.as_deref(), Some("page"), "still writing");
    s.keys(&["!"]);
    assert_eq!(caret(&mut s), "Hi!▮");
    s.keys(&["a-z"]);
    assert!(shown(&mut s, "side"));
    // A section key leaves zen to show it.
    s.keys(&["d-.", "d-1"]);
    assert!(shown(&mut s, "side"));
    assert_eq!(s.state().focus.as_deref(), Some("days"));
}

#[test]
fn tasks_tick_and_the_caret_stays_on_its_text() {
    let mut s = journal();
    s.keys(&["d-t"]);
    for k in "Buy bread".chars() {
        s.keys(&[&k.to_string()]);
    }
    s.keys(&["left", "left", "left", "left", "left"]);
    assert_eq!(caret(&mut s), "Buy ▮bread");
    s.keys(&["c-t"]);
    assert_eq!(caret(&mut s), "- [ ] Buy ▮bread", "a task, the caret on the same letter");
    s.keys(&["c-t"]);
    assert_eq!(caret(&mut s), "- [x] Buy ▮bread", "ticked");
    s.keys(&["c-t"]);
    assert_eq!(caret(&mut s), "Buy ▮bread", "and plain again");
    // Enter after a task starts the next one; Tab nests it.
    s.keys(&["c-t", "d-right", "enter"]);
    assert_eq!(caret(&mut s), "- [ ] Buy bread\n- [ ] ▮");
    s.keys(&["tab", "B", "u", "t", "t", "e", "r"]);
    assert_eq!(caret(&mut s), "- [ ] Buy bread\n  - [ ] Butter▮");
    assert_eq!(s.state().focus.as_deref(), Some("page"), "Tab nests; it doesn't leave");
    // Enter on an empty task outdents it, then ends the list.
    s.keys(&["enter", "enter"]);
    assert_eq!(caret(&mut s), "- [ ] Buy bread\n  - [ ] Butter\n- [ ] ▮");
}

#[test]
fn a_click_on_a_box_ticks_it_and_leaves_the_caret() {
    let mut s = journal();
    let r = s.text_at("page").expect("the editor's text");
    // `- [ ] Write today's entry` is the third line; its box is at columns 2–4.
    s.mouse("press", r.x + 3, r.y + 2).mouse("release", r.x + 3, r.y + 2);
    let now = s.state().now_ms;
    s.msg(Msg::Tick { now_ms: now + 1000 });
    assert!(text(&mut s, "page/Welcome").contains("- [x] Write today's entry"), "{}", text(&mut s, "page/Welcome"));
    assert!(caret(&mut s).ends_with("(⌥Z)\n▮"), "the caret stays where it was writing: {}", caret(&mut s));
}

#[test]
fn a_link_opens_its_page_and_a_new_one_joins_the_list() {
    let mut s = journal();
    s.keys(&["d-t"]);
    for k in "Read [[Moths]]".chars() {
        s.keys(&[&k.to_string()]);
    }
    let r = s.text_at("page").expect("the editor's text");
    s.mouse("press", r.x + 9, r.y).mouse("release", r.x + 9, r.y);
    assert_eq!(value(&mut s, "here")["id"], "Moths");
    assert_eq!(ids(&mut s, "pages").last().unwrap(), "Moths");
}
