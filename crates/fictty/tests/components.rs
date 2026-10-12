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
    assert!(text.starts_with("▾ [ ] Launch"), "{text}");
    assert!(text.contains("\n  ▾ [ ] Record the demo"), "{text}");
    assert!(text.contains("\n    · [ ] Script it"), "open by default: {text}");
    // ← on an open row folds it; the rows under it go.
    s.keys(&["left"]);
    let text = s.screen();
    assert!(!text.contains("Write the post"), "{text}");
    assert!(text.starts_with("▸ [ ] Launch"), "{text}");
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
    s.msg(Msg::Write { key: "plan".into(), write: serde_json::from_value(json!({"write": "put", "value": plan})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
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

#[test]
fn a_list_keeps_its_row_variants_and_its_look_through_json() {
    let ui = json!({"root": {"type": "list", "id": "l", "variant": "card", "row_variant": "kind",
                              "variants": {"head": "<b>{text}</>"}, "skip": ["head"],
                              "items": [{"kind": "head", "text": "Today"}, {"text": "a"}]}});
    let parsed: fictty::model::Ui = serde_json::from_value(ui.clone()).unwrap();
    let back = serde_json::to_value(&parsed).unwrap();
    assert_eq!(back["root"]["variant"], "card");
    assert_eq!(back["root"]["row_variant"], "kind");
    let mut s = screen(ui, 20, 4);
    assert_eq!(s.state().local["l"].selected, 1, "the header can't be selected");
}

#[test]
fn a_selection_stays_on_its_row_when_rows_arrive_above_it() {
    let ui = json!({"root": {"type": "list", "id": "l", "bind": "rows", "item": "{text}"},
                    "store": {"rows": [{"id": "a", "text": "A"}, {"id": "b", "text": "B"}, {"id": "c", "text": "C"}]}});
    let mut s = screen(ui, 20, 6);
    s.keys(&["j"]);
    let rows = json!([{"id": "z", "text": "Z"}, {"id": "y", "text": "Y"}, {"id": "a", "text": "A"}, {"id": "b", "text": "B"}, {"id": "c", "text": "C"}]);
    s.msg(Msg::Write { key: "rows".into(), write: serde_json::from_value(json!({"write": "put", "value": rows})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
    assert_eq!(s.state().local["l"].selected, 3, "still on B");
    // Without a key it's the place that stays.
    let ui = json!({"root": {"type": "list", "id": "l", "bind": "rows"}, "store": {"rows": ["a", "b"]}});
    let mut s = screen(ui, 20, 6);
    s.keys(&["j"]);
    s.msg(Msg::Write { key: "rows".into(), write: serde_json::from_value(json!({"write": "put", "value": ["z", "a", "b"]})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
    assert_eq!(s.state().local["l"].selected, 1);
}

#[test]
fn keep_holds_room_for_the_text_before_the_meta() {
    let ui = json!({"root": {"type": "list", "id": "l", "keep": 12, "mark": "", "lead": "",
                              "item": "{t}\t{meta}", "items": [{"t": "Water the plants", "meta": "Tue 14:00 · tilde · from the agent"}]}});
    let mut s = screen(ui, 30, 2);
    let line = s.screen().lines().next().unwrap().to_string();
    assert!(line.starts_with("Water the p…"), "{line:?}");
}

#[test]
fn rows_can_take_two_lines() {
    let ui = json!({"root": {"type": "list", "id": "l", "lines": 2, "item": "<b>{at}</> {op}\n{what}",
                              "items": [{"at": "09:00", "op": "put", "what": "notes"}, {"at": "09:01", "op": "edit", "what": "page/home"}]}});
    let mut s = screen(ui, 30, 6);
    let text = s.screen();
    assert!(text.starts_with("▸ 09:00 put\n  notes\n  09:01 edit\n  page/home"), "{text}");
    s.keys(&["j"]);
    assert_eq!(s.state().local["l"].selected, 1);
}

#[test]
fn a_tree_takes_a_mark_and_a_leaf_glyph_like_a_list() {
    let ui = json!({"root": {"type": "tree", "id": "t", "mark": "▌", "lead": " ", "leaf": "", "item": "{text}",
                              "items": [{"id": "a", "text": "A", "children": [{"id": "b", "text": "B"}]}]}});
    let mut s = screen(ui, 20, 3);
    assert!(s.screen().starts_with("▌▾ A\n   B"), "{}", s.screen());
}

#[test]
fn templates_fit_fields_escape_brackets_and_take_tabs_from_values() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "text", "size": 1, "text": "a << b <b>bold</>"},
        {"type": "text", "size": 1, "bind": "kv", "text": "{k:6}|{v:>4}|{long:5}|"},
        {"type": "text", "size": 1, "bind": "kv", "text": "{row|markup}"}
    ]}, "data": {"kv": {"value": {"k": "ab", "v": "7", "long": "abcdefgh", "row": "left\tright"}}}});
    let mut s = screen(ui, 20, 4);
    let text = s.screen();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "a < b bold");
    assert_eq!(lines[1], "ab    |   7|abcd…|");
    assert_eq!(lines[2], format!("left{}right", " ".repeat(11)));
}

#[test]
fn a_markup_tag_can_name_a_theme_part() {
    let themes: std::collections::BTreeMap<String, fictty::model::ThemeSpec> =
        serde_json::from_value(json!({"t": {"parts": {"link": "#ff0000+u"}}})).unwrap();
    let ui: fictty::model::Ui = serde_json::from_value(json!({"root": {"type": "text", "text": "see <link>the docs</>"}})).unwrap();
    let mut s = App::new(ui).themes(themes).headless(30, 2).unwrap();
    let ansi = s.ansi();
    assert!(ansi.contains("\u{1b}[0;4;38;2;255;0;0mt"), "{ansi:?}");
}

#[test]
fn chords_wait_for_their_last_key() {
    let ui = json!({"root": {"type": "list", "id": "l", "items": ["a"]},
                    "keys": {"p h": {"put": "prio", "value": "high"}, "p l": {"put": "prio", "value": "low"}, "x": {"put": "prio", "value": "x"}},
                    "store": {"prio": ""}});
    let mut s = screen(ui, 20, 3);
    s.keys(&["p"]);
    assert_eq!(s.state().chord.as_deref(), Some("p"));
    assert_eq!(s.state().data["$keys"], fictty::state::Slot::Ready { value: json!({"pending": "p", "next": ["h", "l"]}) });
    s.keys(&["h"]);
    assert_eq!(entry(&mut s, "prio")["value"], "high");
    // A key that finishes no chord drops it, and does nothing itself.
    s.keys(&["p", "x"]);
    assert_eq!(entry(&mut s, "prio")["value"], "high");
    assert!(s.state().chord.is_none());
    s.keys(&["x"]);
    assert_eq!(entry(&mut s, "prio")["value"], "x");
}

#[test]
fn tab_titles_are_templates_with_a_lead_and_a_trail() {
    let ui = json!({"root": {"type": "tabs", "id": "t", "bind": "counts", "tabs": ["Inbox {inbox}", "Done"], "lead": "<b>thc</>", "trail": "Sat 11",
                              "children": [{"type": "text", "text": "one"}, {"type": "text", "text": "two"}]},
                    "data": {"counts": {"value": {"inbox": 4}}}});
    let mut s = screen(ui, 40, 3);
    let first = s.screen().lines().next().unwrap().to_string();
    assert_eq!(first, format!("thc  Inbox 4   Done{}Sat 11", " ".repeat(15)));
}

#[test]
fn min_and_max_bound_a_share() {
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "text", "text": "L", "size": "60%", "max": 10, "border": true},
        {"type": "text", "text": "R", "border": true}]}});
    let mut s = screen(ui, 60, 3);
    let top = s.screen().lines().next().unwrap().to_string();
    assert!(top.starts_with("╭────────╮╭"), "{top}");
}

fn views() -> Value {
    json!({"root": {"type": "tabs", "id": "views", "tab": 1, "tabs": ["Today", "Inbox <b>{n}</>"], "bind": "counts", "children": [
        {"type": "list", "id": "today", "items": ["a"]},
        {"type": "col", "children": [
            {"type": "list", "id": "inbox", "items": ["x", "y"], "keys": {"/": {"do": [{"put": "asking", "value": true}, {"focus": "ask"}]}}},
            {"type": "input", "id": "ask", "when": "asking", "keys": {"enter": {"do": [{"put": "asking", "value": false}, {"focus": "$prev"}]}}}
        ]}
    ]}, "keys": {"1": {"tab": 0, "of": "views"}, "2": {"tab": 1, "of": "views"}},
    "data": {"counts": {"value": {"n": 2}}}, "store": {"asking": false}})
}

#[test]
fn tabs_start_where_they_say_and_choosing_one_moves_the_focus_into_it() {
    let mut s = screen(views(), 30, 5);
    assert!(s.screen().contains("x"), "starts on Inbox: {}", s.screen());
    assert_eq!(s.state().focus.as_deref(), Some("inbox"), "the content, not the tab bar");
    s.keys(&["1"]);
    assert_eq!(s.state().focus.as_deref(), Some("today"));
    s.msg(Msg::Click { id: "views".into(), row: None, tab: Some(1) });
    assert_eq!(s.state().focus.as_deref(), Some("inbox"));
    assert!(s.screen().starts_with(" Today   Inbox 2"), "{}", s.screen());
}

#[test]
fn focus_goes_back_where_it_was() {
    let mut s = screen(views(), 30, 6);
    s.keys(&["/"]);
    assert_eq!(s.state().focus.as_deref(), Some("ask"));
    s.keys(&["enter"]);
    assert_eq!(s.state().focus.as_deref(), Some("inbox"), "$prev");
    // A template names the node too.
    let ui = json!({"root": {"type": "col", "children": [{"type": "list", "id": "a", "items": ["1"]}, {"type": "list", "id": "b", "items": ["2"]}]},
                    "keys": {"g": {"focus": "{store.to}"}}, "store": {"to": "b"}});
    let mut s = screen(ui, 20, 4);
    s.keys(&["g"]);
    assert_eq!(s.state().focus.as_deref(), Some("b"));
}

#[test]
fn a_cut_meta_ends_in_an_ellipsis_too() {
    let ui = json!({"root": {"type": "list", "id": "l", "keep": 12, "mark": "", "lead": "",
                              "item": "{t}\t{meta}", "items": [{"t": "Water the plants", "meta": "due Oct 18 (in 7d) · !"}]}});
    let mut s = screen(ui, 30, 2);
    let line = s.screen().lines().next().unwrap().to_string();
    assert!(line.ends_with('…') && line.starts_with("Water the p…"), "{line:?}");
}

#[test]
fn the_wheel_scrolls_a_list_and_leaves_the_selection() {
    let rows: Vec<String> = (0..80).map(|i| format!("row {i}")).collect();
    let ui = json!({"root": {"type": "list", "id": "l", "items": rows}});
    let mut s = screen(ui, 30, 21);
    s.screen();
    for _ in 0..3 {
        s.msg(Msg::Scroll { id: "l".into(), by: 1 });
    }
    let frame = s.screen();
    assert_eq!(s.state().local["l"].selected, 0, "the selection stays");
    assert!(frame.starts_with("  row 9 "), "three notches, three rows each: {frame}");
    // A key moves the selection, and the view goes back to it once it's out of sight.
    s.keys(&["j"]);
    assert!(s.screen().starts_with("▸ row 1\n") || s.screen().contains("▸ row 1"), "{}", s.screen());
    assert_eq!(s.state().local["l"].top, None);
}

#[test]
fn tab_indents_in_an_editor_that_asks_for_it() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "editor", "id": "e", "tab": "indent", "syntax": "markdown", "text": "- one\n- two\n- three\nplain"},
        {"type": "list", "id": "l", "items": ["x"]}
    ]}});
    let mut s = screen(ui, 30, 8);
    s.keys(&["down", "tab"]);
    assert_eq!(text(&mut s, "e"), "- one\n  - two\n- three\nplain");
    assert_eq!(s.state().focus.as_deref(), Some("e"), "Tab stays in the editor");
    // A selection over two lines indents both, one undo takes it back.
    s.keys(&["c-home", "s-down", "s-down", "tab"]);
    assert_eq!(text(&mut s, "e"), "  - one\n    - two\n- three\nplain");
    s.keys(&["s-tab"]);
    assert_eq!(text(&mut s, "e"), "- one\n  - two\n- three\nplain");
    s.keys(&["d-z"]);
    assert_eq!(text(&mut s, "e"), "  - one\n    - two\n- three\nplain");
    // Plain text takes two spaces; Esc leaves.
    s.keys(&["c-end", "tab"]);
    assert!(text(&mut s, "e").ends_with("plain  "));
    s.keys(&["esc"]);
    assert_eq!(s.state().focus.as_deref(), Some("l"));
}

#[test]
fn a_put_into_text_leaves_the_caret_after_it() {
    // A find starts by putting the first letter typed into the finder, then focusing it.
    let ui = json!({"root": {"type": "col", "children": [{"type": "input", "id": "find", "size": 1}, {"type": "list", "id": "l", "items": ["a"]}]},
                    "focus": "l", "keys": {"3": {"do": [{"put": "find", "value": "3"}, {"focus": "find"}]}}});
    let mut s = screen(ui, 20, 3);
    s.keys(&["3", "q"]);
    assert_eq!(text(&mut s, "find"), "3q");
}

#[test]
fn a_modal_layer_covers_what_is_under_it() {
    let ui = json!({"root": {"type": "stack", "children": [
        {"type": "text", "text": "XXXXXXXXXXXXXXXXXXXX\nXXXXXXXXXXXXXXXXXXXX\nXXXXXXXXXXXXXXXXXXXX"},
        {"type": "text", "id": "m", "modal": true, "pad": 1, "text": "hi"}
    ]}});
    let mut s = screen(ui, 20, 4);
    let frame = s.screen();
    assert!(!frame.contains('X'), "{frame}");
    assert!(frame.contains("hi"));
}

#[test]
fn keys_typed_while_a_page_loads_land_when_it_does() {
    use fictty::state::{Effect, State, update};
    let mut st = State { hosted: vec!["page/".into()], ..State::default() };
    let ui = serde_json::from_value(json!({"root": {"type": "editor", "id": "e", "doc": "page/a"}})).unwrap();
    let fx = update(&mut st, Msg::Push { ui });
    assert!(fx.contains(&Effect::Read("page/a".into())));
    for k in ["h", "i"] {
        update(&mut st, Msg::Key { key: k.into() });
    }
    assert_eq!(st.typed_ahead.len(), 2);
    update(&mut st, Msg::Loaded { key: "page/a".into(), result: Ok(json!("# Page")) });
    assert!(st.typed_ahead.is_empty());
    assert_eq!(st.store["page/a"].read()["text"], "hi# Page");
}

#[test]
fn a_push_fetches_only_the_sources_that_changed() {
    use fictty::state::{Effect, State, update};
    let ui = |n: u32| -> fictty::model::Ui {
        serde_json::from_value(json!({"root": {"type": "text", "text": "x"},
                                      "data": {"same": {"cmd": ["date"]}, "moved": {"cmd": ["echo", n.to_string()]}}})).unwrap()
    };
    let mut st = State::default();
    let first = update(&mut st, Msg::Push { ui: ui(1) });
    assert_eq!(first.len(), 2);
    update(&mut st, Msg::Data { name: "same".into(), result: Ok(json!(1)) });
    update(&mut st, Msg::Data { name: "moved".into(), result: Ok(json!(1)) });
    let again = update(&mut st, Msg::Push { ui: ui(2) });
    assert_eq!(again, vec![Effect::Fetch("moved".into())]);
}

#[test]
fn a_lasting_write_goes_on_the_screens_clock() {
    let ui = json!({"root": {"type": "list", "id": "l", "bind": "rows", "item": "{id}{live|*=}"},
                    "store": {"rows": [{"id": "a"}, {"id": "b"}], "live": {}}});
    let mut s = screen(ui, 20, 4);
    let merge = |v: Value| serde_json::from_value(json!({"write": "merge", "value": v})).unwrap();
    s.msg(Msg::Write { key: "live".into(), write: merge(json!({"a": true})), if_rev: None, by: "agent".into(), for_ms: Some(3000) });
    s.msg(Msg::Tick { now_ms: 1000 });
    s.msg(Msg::Write { key: "live".into(), write: merge(json!({"b": true})), if_rev: None, by: "agent".into(), for_ms: Some(3000) });
    assert_eq!(entry(&mut s, "live")["value"], json!({"a": true, "b": true}));
    s.msg(Msg::Tick { now_ms: 3000 });
    assert_eq!(entry(&mut s, "live")["value"], json!({"b": true}), "a went at 3000, b lasts until 4000");
    assert_eq!(entry(&mut s, "live")["by"], "clock");
    s.msg(Msg::Tick { now_ms: 4000 });
    assert_eq!(entry(&mut s, "live")["value"], json!({}));
    // A put with for_ms takes the entry away again; a when on it comes and goes with it.
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "text", "text": "undo?", "when": "undoable", "size": 1},
        {"type": "list", "id": "l", "items": ["x"], "keys": {"d": {"put": "undoable", "value": true, "for_ms": 500}}}]}});
    let mut s = screen(ui, 20, 3);
    s.keys(&["d"]);
    assert!(s.screen().contains("undo?"));
    s.msg(Msg::Tick { now_ms: 600 });
    assert!(!s.screen().contains("undo?"));
    assert!(!s.state().store.contains_key("undoable"));
}

#[test]
fn toasts_go_where_a_toasts_node_is() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "text", "text": "body"},
        {"type": "toasts", "size": 1}]}});
    let mut s = screen(ui, 30, 4);
    s.msg(Msg::Toast { text: "saved by agent".into(), tone: Some("pos".into()), ms: None });
    let frame = s.screen();
    let lines: Vec<&str> = frame.lines().collect();
    assert_eq!(lines[2].trim(), "saved by agent", "{frame}");
    assert!(!frame.contains('╭'), "no corner toast: {frame}");
}

#[test]
fn a_toasts_node_covers_what_is_under_it_while_one_shows() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "text", "text": "body"},
        {"type": "stack", "size": 1, "children": [
            {"type": "text", "text": "the bar's own text, quite long"},
            {"type": "toasts"}]}]}});
    let mut s = screen(ui, 40, 4);
    assert_eq!(s.state().data.get("$toasting"), None, "no toast yet");
    s.msg(Msg::Toast { text: "hi".into(), tone: None, ms: Some(100) });
    let bar = s.screen().lines().nth(2).unwrap().to_string();
    assert_eq!(bar.trim(), "hi", "{bar:?}");
    assert_eq!(s.state().data["$toasting"], fictty::state::Slot::Ready { value: json!(true) });
    s.msg(Msg::Tick { now_ms: 100 });
    assert!(s.screen().contains("the bar's own text"));
    assert_eq!(s.state().data["$toasting"], fictty::state::Slot::Ready { value: json!(false) });
}

#[test]
fn an_editor_can_start_with_the_caret_at_the_end() {
    use fictty::state::{State, update};
    let mut st = State { hosted: vec!["page/".into()], ..State::default() };
    let ui = serde_json::from_value(json!({"root": {"type": "editor", "id": "e", "doc": "page/a", "caret": "end"}})).unwrap();
    update(&mut st, Msg::Push { ui });
    update(&mut st, Msg::Key { key: "x".into() });
    update(&mut st, Msg::Loaded { key: "page/a".into(), result: Ok(json!("# Page")) });
    assert_eq!(st.store["page/a"].read()["text"], "# Pagex");
}

#[test]
fn keys_type_into_a_field_a_key_just_showed() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "input", "id": "find", "size": 1, "when": "finding"},
        {"type": "list", "id": "l", "items": ["a"]}]},
        "focus": "l", "store": {"finding": false},
        "keys": {"/": {"do": [{"put": "finding", "value": true}, {"focus": "find"}]}}});
    let mut s = screen(ui, 30, 4);
    s.keys(&["/", "m", "q", "4"]);
    assert!(s.screen().starts_with("mq4"), "{}", s.screen());
}

#[test]
fn a_row_can_be_selected_by_key_before_it_arrives() {
    let ui = json!({"root": {"type": "list", "id": "l", "bind": "rows", "item": "{id}"},
                    "store": {"rows": [{"id": "a"}, {"id": "b"}], "pick": "d"},
                    "keys": {"s": {"select": "{store.pick}", "in": "l"}}});
    let mut s = screen(ui, 20, 6);
    s.msg(Msg::Set { id: "l".into(), selected: None, tab: None, key: Some("b".into()) });
    assert_eq!(s.state().local["l"].selected, 1);
    // Asked for before it's there: selected when it comes.
    s.keys(&["s"]);
    assert_eq!(s.state().local["l"].selected, 1, "d isn't there yet");
    let rows = json!([{"id": "c"}, {"id": "a"}, {"id": "b"}, {"id": "d"}]);
    s.msg(Msg::Write { key: "rows".into(), write: serde_json::from_value(json!({"write": "put", "value": rows})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
    assert_eq!(s.state().local["l"].selected, 3);
    assert!(s.state().local["l"].want.is_none());
}

#[test]
fn each_repeats_a_template_per_item() {
    let ui = json!({
        "root": {"type": "each", "id": "panels", "bind": "open", "template": {
            "type": "editor", "id": "ed-{$item.id}", "title": "{$item.title}", "doc": "page/{$item.id}",
            "text": "{$item.title} notes", "size": "{$item.h}"}},
        "store": {"open": [{"id": "a", "title": "Alpha", "h": 4}, {"id": "b", "title": "Beta", "h": 4}]}
    });
    let mut s = screen(ui, 30, 12);
    let frame = s.screen();
    assert!(frame.contains("Alpha ") && frame.contains("Beta "), "{frame}");
    assert!(frame.contains("Alpha notes") && frame.contains("Beta notes"), "{frame}");
    assert_eq!(s.state().focus.as_deref(), Some("ed-a"));
    s.keys(&["tab", "x"]);
    assert_eq!(text(&mut s, "page/b"), "xBeta notes", "the second instance has the focus");
    // The template's own placeholders are left for it: ui keeps the template as pushed.
    assert_eq!(s.state().ui.root.kind_name(), "each");
    // A new item, a new instance.
    let open = json!([{"id": "a", "title": "Alpha", "h": 3}, {"id": "b", "title": "Beta", "h": 3}, {"id": "c", "title": "Gamma", "h": 3}]);
    s.msg(Msg::Write { key: "open".into(), write: serde_json::from_value(json!({"write": "put", "value": open})).unwrap(), if_rev: None, by: "agent".into(), for_ms: None });
    assert!(s.screen().contains("Gamma"), "{}", s.screen());
    assert_eq!(s.state().focus.as_deref(), Some("ed-b"), "focus stays on its instance");
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
    let mut s = screen(ui, 100, 4);
    let frame = s.screen();
    assert!(frame.contains("NARROW LIST") && !frame.contains("WIDE"), "beside the sidebar: {frame}");
    assert_eq!(s.state().focus.as_deref(), Some("narrow"), "focus goes where it shows");
    s.keys(&["x"]);
    let frame = s.screen();
    assert!(frame.contains("WIDE TABLE") && !frame.contains("NARROW"), "alone: {frame}");
    assert_eq!(s.state().focus.as_deref(), Some("wide"));
}

#[test]
fn links_in_an_editor_act_on_a_click_and_a_key() {
    let ui = json!({
        "root": {"type": "editor", "id": "page", "syntax": "markdown",
                 "text": "See [[Home Page]] and #ideas\nnothing here",
                 "links": [
                     {"kind": "wiki", "click": {"put": "opened", "value": "{$match}"}, "s-click": {"put": "aside", "value": "{$match}"}},
                     {"kind": "tag", "click": {"put": "tag", "value": "{$match}"}}],
                 "keys": {"a-o": {"put": "opened", "value": "{$link}"}}},
        "store": {"opened": "", "aside": "", "tag": ""}
    });
    let mut s = screen(ui, 40, 4);
    let value = |s: &mut fictty::Headless, k: &str| s.state().store[k].read()["value"].clone();
    s.msg(Msg::Point { id: "page".into(), col: 8, row: 0, extend: false });
    assert_eq!(value(&mut s, "opened"), "Home Page");
    s.msg(Msg::Tick { now_ms: 5000 });
    s.msg(Msg::Point { id: "page".into(), col: 10, row: 0, extend: true });
    assert_eq!(value(&mut s, "aside"), "Home Page");
    s.msg(Msg::Tick { now_ms: 10000 });
    s.msg(Msg::Point { id: "page".into(), col: 24, row: 0, extend: false });
    assert_eq!(value(&mut s, "tag"), "ideas");
    // A click elsewhere places the caret and nothing more.
    s.msg(Msg::Tick { now_ms: 15000 });
    s.msg(Msg::Point { id: "page".into(), col: 2, row: 1, extend: false });
    assert_eq!(value(&mut s, "tag"), "ideas");
    // ⌥O opens the link at the caret.
    s.keys(&["c-home", "right", "right", "right", "right", "right"]);
    s.msg(Msg::Write { key: "opened".into(), write: serde_json::from_value(json!({"write": "put", "value": ""})).unwrap(), if_rev: None, by: "t".into(), for_ms: None });
    s.keys(&["a-o"]);
    assert_eq!(value(&mut s, "opened"), "Home Page");
}

#[test]
fn focus_moves_without_a_click() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "list", "id": "a", "items": ["1"]},
        {"type": "list", "id": "b", "items": ["2"], "click": {"put": "clicked", "value": true}}]},
        "store": {"clicked": false}});
    let mut s = screen(ui, 20, 4);
    s.msg(Msg::Focus { id: "b".into() });
    assert_eq!(s.state().focus.as_deref(), Some("b"));
    assert_eq!(entry(&mut s, "clicked")["value"], false, "no click action");
}

#[test]
fn a_wiki_links_brackets_only_place_the_caret() {
    let ui = json!({"root": {"type": "editor", "id": "e", "syntax": "markdown", "text": "go [[Home]] now",
                    "links": [{"kind": "wiki", "click": {"put": "opened", "value": "{$match}"}}]},
                    "store": {"opened": ""}});
    let mut s = screen(ui, 30, 3);
    s.msg(Msg::Point { id: "e".into(), col: 4, row: 0, extend: false });
    assert_eq!(entry(&mut s, "opened")["value"], "", "a bracket");
    s.msg(Msg::Tick { now_ms: 5000 });
    s.msg(Msg::Point { id: "e".into(), col: 6, row: 0, extend: false });
    assert_eq!(entry(&mut s, "opened")["value"], "Home", "the title");
}

#[test]
fn each_follows_its_data_and_keeps_its_tree_otherwise() {
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "each", "id": "e", "bind": "open", "template": {"type": "text", "id": "t-{$item}", "text": "{$item}!", "size": 1}},
        {"type": "list", "id": "l", "items": ["x", "y"]}]},
        "store": {"open": ["a"]}});
    let mut s = screen(ui, 20, 5);
    assert!(s.screen().contains("a!"));
    s.keys(&["j"]);
    assert!(s.screen().contains("a!"), "a key that changes nothing it reads leaves it");
    s.msg(Msg::Write { key: "open".into(), write: serde_json::from_value(json!({"write": "put", "value": ["a", "b"]})).unwrap(), if_rev: None, by: "t".into(), for_ms: None });
    assert!(s.screen().contains("b!"), "{}", s.screen());
}

#[test]
fn with_commands_off_nothing_runs_and_the_screen_says_why() {
    let dir = std::env::temp_dir().join(format!("fictty-commands-off-{}", std::process::id()));
    let mark = dir.join("ran");
    let ui = json!({
        "root": {"type": "col", "children": [
            {"type": "text", "id": "out", "bind": "@listing", "text": "{.}"},
            {"type": "button", "id": "go", "label": "go", "click": {"run": ["sh", "-c", format!("mkdir -p {0} && touch {1}", dir.display(), mark.display())]}}
        ]},
        "data": {"listing": {"shell": "echo hello"}},
        "focus": "go"
    });
    let mut s = App::new(serde_json::from_value(ui).unwrap()).commands(false).headless(60, 8).unwrap();
    s.keys(&["enter"]);
    assert!(!mark.exists(), "the button's command ran");
    let status = s.state().status.clone().unwrap_or_default();
    assert!(status.ends_with("commands are off"), "{status}");
    let listing = serde_json::to_value(&s.state().data["listing"]).unwrap();
    assert!(listing.to_string().contains("commands are off"), "{listing}");
}

#[test]
fn headless_takes_the_mouse_and_resizes_like_a_terminal() {
    let ui = json!({
        "root": {"type": "col", "children": [
            {"type": "list", "id": "l", "items": ["one", "two", "three"], "min": 3},
            {"type": "editor", "id": "e", "text": "hello world"}
        ]},
        "focus": "l"
    });
    let mut s = screen(ui, 30, 10);
    let at = |s: &mut fictty::Headless, text: &str| {
        let screen = s.screen();
        let (y, line) = screen.lines().enumerate().find(|(_, l)| l.contains(text)).unwrap_or_else(|| panic!("{text} in {screen}"));
        (line[..line.find(text).unwrap()].chars().count() as u16, y as u16)
    };
    // A press on a row selects it.
    let (x, y) = at(&mut s, "three");
    s.mouse("press", x, y).mouse("release", x, y);
    assert_eq!(s.state().local["l"].selected, 2);
    // A drag across the editor's text selects what it crossed.
    let (x, y) = at(&mut s, "hello");
    s.mouse("press", x, y).mouse("drag", x + 5, y).mouse("release", x + 5, y);
    assert_eq!(s.state().focus.as_deref(), Some("e"));
    let (_, v) = fictty::state::field_view(s.state(), "e").unwrap();
    let r = v.selection.primary();
    assert_eq!((r.anchor.min(r.head), r.anchor.max(r.head)), (0, 5));
    // A paste replaces the selection; a resize redraws at the new size.
    s.paste("HOWDY");
    assert!(s.screen().contains("HOWDY world"));
    s.resize(20, 6);
    assert_eq!(s.size(), (20, 6));
    assert_eq!(s.screen().lines().count(), 6);
}

#[test]
fn a_key_at_the_end_of_a_long_paragraph_stays_quick() {
    // Found by Thought Control: each cell's look scanned its whole line for links, so a key in a
    // 1,200-character paragraph took over half a second in a release build.
    let para = "Words and [[a link]] and #tag here, ".repeat(34);
    let ui = json!({"root": {"type": "editor", "id": "e", "text": para, "syntax": "markdown", "caret": "end",
                             "links": [{"kind": "wiki"}, {"kind": "tag"}]}, "focus": "e"});
    let mut s = screen(ui, 100, 30);
    s.keys(&["d-end"]);
    let t = std::time::Instant::now();
    for _ in 0..10 {
        s.keys(&["x"]);
        s.screen();
    }
    let each = t.elapsed() / 10;
    let limit = if cfg!(debug_assertions) { 300 } else { 30 };
    assert!(each.as_millis() < limit, "{each:?} a key");
}

#[test]
fn an_editor_and_a_preview_of_its_text_start_with_the_editors_text() {
    // The preview comes first and names no text: the editor's seeds the entry.
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "editor", "id": "preview", "doc": "page", "readonly": true},
        {"type": "editor", "id": "body", "doc": "page", "text": "Some text"}
    ]}});
    let mut s = App::new(serde_json::from_value(ui).unwrap()).headless(40, 4).unwrap();
    assert_eq!(s.request(json!({"op": "read", "key": "page"}))["text"], "Some text");
}

#[test]
fn a_state_read_back_works_its_tree_out_again() {
    // Thought Control's harness read `get --state` back into a State: its tree is left out, so
    // the focus order had no `each` instances until it's worked out again.
    let ui = json!({
        "root": {"type": "each", "id": "panels", "bind": "open", "template": {"type": "list", "id": "l-{$item}", "items": ["x"]}},
        "store": {"open": ["a", "b"]}
    });
    let mut s = App::new(serde_json::from_value(ui).unwrap()).headless(30, 8).unwrap();
    let live = fictty::state::focus_order(s.state());
    assert_eq!(live, ["l-a", "l-b"]);
    let mut back: fictty::state::State = serde_json::from_value(s.request(json!({"op": "state"}))).unwrap();
    assert!(fictty::state::focus_order(&back).is_empty(), "read back, the tree isn't there");
    back.derive();
    assert_eq!(fictty::state::focus_order(&back), live);
}


#[test]
fn the_readback_has_each_nodes_rectangle_from_the_grid() {
    let ui = json!({"root": {"type": "row", "children": [
        {"type": "input", "id": "name", "size": 20},
        {"type": "button", "id": "go", "label": "Go"}
    ]}, "focus": "name", "status_bar": false});
    let mut s = App::new(serde_json::from_value(ui).unwrap()).headless(40, 3).unwrap();
    s.keys(&["h", "i"]);
    let nodes = s.readback();
    let name = nodes.iter().find(|n| n.id.as_deref() == Some("name")).unwrap();
    assert_eq!((name.text.as_str(), name.focus), ("hi", true));
    assert_eq!(name.rect, Some(fictty::engine::Rect::new(0, 0, 20, 3)));
    let go = nodes.iter().find(|n| n.id.as_deref() == Some("go")).unwrap();
    assert_eq!(go.rect, Some(fictty::engine::Rect::new(20, 0, 20, 3)));
    // The socket's op answers the same.
    let r = s.request(json!({"op": "readback"}));
    assert_eq!(r["nodes"][1]["id"], "name");
}

#[test]
fn a_live_screen_runs_its_engine_on_its_own_thread() {
    let sock = std::env::temp_dir().join(format!("fl-{}.sock", std::process::id()));
    let ui = json!({"root": {"type": "text", "id": "t", "text": "n={n}", "bind": "n"}, "data": {"n": {"value": {"n": 1}}}});
    let live = App::new(serde_json::from_value(ui).unwrap()).socket(&sock).into_core().live(40, 5).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let live = live.spawn(move || tx.send(()).is_ok());
    // An agent's request goes through the engine's thread, which rings.
    let r = fictty::runtime::send(&sock, json!({"op": "write", "key": "k", "write": "put", "value": "v"})).unwrap();
    assert_eq!(r["ok"], true);
    rx.recv_timeout(std::time::Duration::from_secs(5)).expect("a ring");
    assert_eq!(live.lock().unwrap().state().store["k"].read()["value"], "v");
    // The source's value arrives from a thread of its own.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while fictty::runtime::send(&sock, json!({"op": "readback"})).unwrap()["nodes"][0]["text"] != "n=1" {
        assert!(std::time::Instant::now() < deadline, "the source's value should show");
        let _ = rx.recv_timeout(std::time::Duration::from_millis(50));
    }
    // Input from whatever draws it joins the same queue.
    let inbox = live.lock().unwrap().inbox();
    inbox.input(fictty::engine::Input::Key("c-c".into()));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !live.lock().unwrap().quit() {
        assert!(std::time::Instant::now() < deadline, "the screen should quit");
        let _ = rx.recv_timeout(std::time::Duration::from_millis(50));
    }
    drop(live);
}
