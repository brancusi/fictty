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
