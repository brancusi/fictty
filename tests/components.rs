//! The components, driven as a person would (keys, clicks) and read back as an agent would (the
//! frame as text, the state as JSON).

use fictty::App;
use fictty::state::Msg;
use serde_json::{Value, json};

fn screen(ui: Value, w: u16, h: u16) -> fictty::Headless {
    App::new(serde_json::from_value(ui).unwrap()).headless(w, h).unwrap()
}

fn entry(s: &mut fictty::Headless, key: &str) -> Value {
    s.state().store[key].read()
}

fn outline() -> Value {
    json!({
        "root": {"type": "tree", "id": "t", "bind": "plan", "item": "{done|true=[x];*=[ ]} {text}",
                 "keys": {"x": {"toggle": "done"}, "d": {"remove": "plan"}}},
        "store": {"plan": [
            {"id": "a", "text": "Launch", "children": [
                {"id": "a1", "text": "Write the post"},
                {"id": "a2", "text": "Record the demo", "children": [{"id": "a2x", "text": "Script it"}]}
            ]},
            {"id": "b", "text": "Rest"}
        ]}
    })
}

#[test]
fn a_tree_shows_nested_rows_and_folds_them() {
    let mut s = screen(outline(), 40, 8);
    let text = s.screen();
    assert!(text.contains(" ▾ [ ] Launch"), "{text}");
    assert!(text.contains("     ▸ [ ] Record the demo") || text.contains("   ▾ [ ] Record the demo"), "{text}");
    assert!(text.contains("     · [ ] Script it"), "open by default: {text}");
    // ← on an open row folds it; the rows under it go.
    s.keys(&["left"]);
    let text = s.screen();
    assert!(!text.contains("Write the post"), "{text}");
    assert!(text.contains(" ▸ [ ] Launch"), "{text}");
    // → opens it again, and a second → goes to its first child.
    s.keys(&["right", "right"]);
    assert_eq!(s.state().local["t"].selected, 1);
    // ← on a leaf goes to its parent.
    s.keys(&["left"]);
    assert_eq!(s.state().local["t"].selected, 0);
}

#[test]
fn a_tree_row_is_addressable_in_the_store() {
    let mut s = screen(outline(), 40, 8);
    // Down to "Script it" (Launch, Write, Record, Script) and tick it.
    s.keys(&["j", "j", "j", "x"]);
    let plan = entry(&mut s, "plan");
    assert_eq!(plan["value"][0]["children"][1]["children"][0]["done"], json!(true), "{plan}");
    assert!(s.screen().contains("[x] Script it"));
    s.keys(&["x"]);
    assert_eq!(entry(&mut s, "plan")["value"][0]["children"][1]["children"][0]["done"], json!(false));
    // Remove takes the row out where it sits: up to "Record the demo", and its rows go with it.
    s.keys(&["k", "d"]);
    let plan = entry(&mut s, "plan");
    assert_eq!(plan["value"][0]["children"].as_array().unwrap().len(), 1, "{plan}");
    assert_eq!(plan["value"][0]["children"][0]["id"], "a1");
}

#[test]
fn folds_survive_an_update_from_outside() {
    let mut s = screen(outline(), 40, 8);
    s.keys(&["j", "j", "space"]);
    assert!(!s.screen().contains("Script it"));
    let mut plan = entry(&mut s, "plan")["value"].clone();
    plan[1]["text"] = "Rest a lot".into();
    s.msg(Msg::Write { key: "plan".into(), write: serde_json::from_value(json!({"write": "put", "value": plan})).unwrap(), if_rev: None, by: "agent".into() });
    let text = s.screen();
    assert!(!text.contains("Script it") && text.contains("Rest a lot"), "{text}");
}

#[test]
fn a_filter_narrows_a_list_and_store_actions_still_find_the_row() {
    let ui = json!({
        "root": {"type": "col", "children": [
            {"type": "input", "id": "q", "size": 1},
            {"type": "list", "id": "l", "bind": "fruit", "filter": "q", "item": "{name}",
             "keys": {"d": {"remove": "fruit"}}}
        ]},
        "store": {"fruit": [{"name": "Apple"}, {"name": "Banana"}, {"name": "Blueberry"}, {"name": "Cherry"}]}
    });
    let mut s = screen(ui, 30, 8);
    s.keys(&["B", "e"]);
    let text = s.screen();
    assert!(text.contains("Blueberry") && !text.contains("Banana") && !text.contains("Apple"), "{text}");
    s.keys(&["tab", "d"]);
    let fruit = entry(&mut s, "fruit");
    let names: Vec<&str> = fruit["value"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Apple", "Banana", "Cherry"]);
}

#[test]
fn a_filtered_tree_opens_to_show_what_matches() {
    let mut ui = outline();
    ui["root"]["filter"] = "q".into();
    ui["root"] = json!({"type": "col", "children": [{"type": "input", "id": "q", "size": 1}, ui["root"].clone()]});
    let mut s = screen(ui, 40, 8);
    s.keys(&["s", "c", "r", "i", "p", "t"]);
    let text = s.screen();
    assert!(text.contains("Launch") && text.contains("Record the demo") && text.contains("Script it"), "{text}");
    assert!(!text.contains("Write the post") && !text.contains("Rest"), "{text}");
}

fn dialog() -> Value {
    json!({
        "root": {"type": "stack", "children": [
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

#[test]
fn when_hides_a_node_and_gives_its_room_away() {
    let mut wide = screen(dialog(), 80, 6);
    assert!(wide.screen().contains("wide detail"));
    let mut narrow = screen(dialog(), 50, 6);
    let text = narrow.screen();
    assert!(!text.contains("wide detail"), "{text}");
}

#[test]
fn a_modal_floats_over_the_screen_and_keeps_the_focus() {
    let mut s = screen(dialog(), 80, 12);
    assert!(!s.screen().contains("Delete?"));
    s.keys(&["d"]);
    let text = s.screen();
    assert!(text.contains("╭ Delete? ") && text.contains("Delete") && text.contains("Keep"), "{text}");
    assert_eq!(s.state().focus.as_deref(), Some("yes"), "focus moves into the dialog");
    s.keys(&["tab"]);
    assert_eq!(s.state().focus.as_deref(), Some("no"));
    s.keys(&["tab"]);
    assert_eq!(s.state().focus.as_deref(), Some("yes"), "and stays there");
    // A click behind the dialog goes nowhere.
    s.msg(Msg::Click { id: String::new(), row: None, tab: None });
    assert_eq!(s.state().focus.as_deref(), Some("yes"));
    s.keys(&["enter"]);
    let text = s.screen();
    assert!(!text.contains("Delete?"), "{text}");
    assert!(text.contains("Deleted"), "the toast shows: {text}");
    assert_eq!(s.state().focus.as_deref(), Some("files"), "focus goes back");
}

#[test]
fn toasts_go_when_their_time_is_up() {
    let mut s = screen(dialog(), 60, 10);
    s.msg(Msg::Toast { text: "Saved".into(), tone: Some("pos".into()), ms: Some(500) });
    assert!(s.screen().contains("Saved"));
    s.msg(Msg::Tick { now_ms: 499 });
    assert!(s.screen().contains("Saved"));
    s.msg(Msg::Tick { now_ms: 500 });
    assert!(!s.screen().contains("Saved"));
}

#[test]
fn a_cut_line_ends_in_an_ellipsis_and_keeps_its_right_side() {
    let ui = json!({"root": {"type": "list", "id": "l", "item": "{t}\t<d>{when}</>",
                              "items": [{"t": "a title far too long to fit", "when": "Tue · tilde"}]}});
    let mut s = screen(ui, 30, 3);
    let line = s.screen().lines().next().unwrap().to_string();
    assert!(line.ends_with("Tue · tilde"), "{line:?}");
    assert!(line.contains("…"), "{line:?}");
}

#[test]
fn gaps_space_out_a_row() {
    let ui = json!({"root": {"type": "row", "gap": 3, "children": [
        {"type": "text", "text": "A", "size": 1}, {"type": "text", "text": "B", "size": 1}]}});
    let mut s = screen(ui, 20, 2);
    assert!(s.screen().starts_with("A   B"));
}

fn text(s: &mut fictty::Headless, key: &str) -> String {
    s.state().store[key].read()["text"].as_str().unwrap().to_string()
}

#[test]
fn an_editor_ticks_markdown_tasks() {
    let ui = json!({"root": {"type": "editor", "id": "page", "syntax": "markdown",
                              "text": "# Today\n- [ ] water the plants\n- [x] call the bank\nplain"}});
    let mut s = screen(ui, 40, 6);
    // The caret starts at the top; down a line and tick it.
    s.keys(&["down", "c-t"]);
    assert_eq!(text(&mut s, "page"), "# Today\n- [x] water the plants\n- [x] call the bank\nplain");
    s.keys(&["c-t"]);
    assert!(text(&mut s, "page").contains("- [ ] water"));
    // A click on a box ticks it (row 2, the box at column 3).
    s.msg(Msg::Point { id: "page".into(), col: 3, row: 2, extend: false });
    assert!(text(&mut s, "page").contains("- [ ] call the bank"));
    // Not a task: Ctrl-T does what the editor does with it, and the text keeps its tasks.
    s.keys(&["down", "c-t"]);
    assert!(text(&mut s, "page").ends_with("plain"));
}

#[test]
fn a_readonly_editor_scrolls_but_never_changes() {
    let long: Vec<String> = (1..=40).map(|i| format!("line {i}")).collect();
    let ui = json!({"root": {"type": "editor", "id": "log", "readonly": true, "text": long.join("\n")}});
    let mut s = screen(ui, 30, 6);
    s.keys(&["a", "b", "bs", "enter"]);
    assert_eq!(text(&mut s, "log"), long.join("\n"));
    s.keys(&["pgdn", "pgdn"]);
    let frame = s.screen();
    assert!(!frame.contains("line 1\n"), "{frame}");
}

#[test]
fn a_spinner_turns_with_the_clock() {
    let ui = json!({"root": {"type": "spinner", "text": "syncing"}});
    let mut s = screen(ui, 20, 2);
    let a = s.screen();
    assert!(a.starts_with("⠋ syncing"), "{a}");
    s.msg(Msg::Tick { now_ms: 85 });
    assert!(s.screen().starts_with("⠙ syncing"));
}

#[test]
fn a_long_list_shows_a_scroll_bar_and_keeps_its_rows_clear_of_it() {
    let rows: Vec<String> = (0..40).map(|i| format!("row {i:02} xxxxxxxxxxxxxxxxxxxxxxxx")).collect();
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows}});
    let mut s = screen(ui, 20, 6);
    let frame = s.screen();
    let first = frame.lines().next().unwrap();
    assert!(first.ends_with('┃'), "{frame}");
    assert!(first.contains('…'), "{frame}");
    s.keys(&["G"]);
    let frame = s.screen();
    assert!(frame.lines().nth(4).unwrap().ends_with('┃'), "the thumb at the bottom: {frame}");
}

#[test]
fn remove_where_takes_matching_rows_from_anywhere() {
    let ui = json!({
        "root": {"type": "col", "children": [
            {"type": "list", "id": "l", "bind": "files", "item": "{name}"},
            {"type": "button", "id": "b", "label": "Delete b.txt", "click": {"remove": "files", "where": {"name": "{store.doomed}"}}}
        ]},
        "store": {"files": [{"name": "a.txt"}, {"name": "b.txt"}, {"name": "c.txt"}], "doomed": "b.txt"},
        "focus": "b"
    });
    let mut s = screen(ui, 30, 6);
    assert_eq!(s.state().focus.as_deref(), Some("b"), "the UI's focus");
    s.keys(&["enter"]);
    let names: Vec<String> = entry(&mut s, "files")["value"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(names, ["a.txt", "c.txt"]);
}

#[test]
fn esc_leaves_a_field_for_the_next_thing_that_takes_keys() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "input", "id": "q", "size": 1},
        {"type": "list", "id": "l", "items": ["a", "b"]}
    ]}, "keys": {"n": {"toast": "next"}}});
    let mut s = screen(ui, 30, 6);
    assert_eq!(s.state().focus.as_deref(), Some("q"));
    s.keys(&["n"]);
    assert!(s.state().toasts.is_empty(), "n types in the field");
    s.keys(&["esc", "n"]);
    assert_eq!(s.state().focus.as_deref(), Some("l"));
    assert_eq!(s.state().toasts.len(), 1, "now n is the screen's");
}

#[test]
fn a_layer_shows_while_its_when_holds() {
    let ui = json!({
        "root": {"type": "list", "id": "l", "items": ["a"], "keys": {"x": {"put": "done", "value": true}}},
        "layers": [{"on": "l", "title": "Hint", "text": "press x", "when": "!done & !never"}],
        "store": {"done": false}
    });
    let mut s = screen(ui, 40, 10);
    assert!(s.screen().contains("press x"));
    s.keys(&["x"]);
    assert!(!s.screen().contains("press x"));
}
