//! fictty embedded as a library: an app builds the UI as a value, serves its data and actions in
//! process through a `Host`, and tests the screen without a terminal.

use fictty::{App, Host, Notifier};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// A todo app's world: the host owns it, the screen only reads it through `today`.
#[derive(Clone, Default)]
struct Todos {
    done: Arc<Mutex<Vec<String>>>,
    notifier: Arc<Mutex<Option<Notifier>>>,
}

impl Host for Todos {
    fn start(&self, notifier: Notifier) {
        *self.notifier.lock().unwrap() = Some(notifier);
    }

    fn fetch(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "today" => {
                let done = self.done.lock().unwrap();
                let limit = args["limit"].as_u64().unwrap_or(10) as usize;
                let rows: Vec<Value> = ["water the plants", "call the bank", "write the post"]
                    .iter()
                    .take(limit)
                    .enumerate()
                    .map(|(i, t)| json!({"id": format!("t{i}"), "text": t, "done": done.contains(&format!("t{i}"))}))
                    .collect();
                Ok(json!({"rows": rows}))
            }
            other => Err(format!("no source {other}")),
        }
    }

    fn act(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "done" => {
                let id = args["id"].as_str().ok_or("done needs an id")?.to_string();
                self.done.lock().unwrap().push(id.clone());
                Ok(json!({"done": id}))
            }
            other => Err(format!("can't {other}")),
        }
    }
}

fn ui() -> fictty::model::Ui {
    serde_json::from_value(json!({
        "root": {"type": "list", "id": "todos", "bind": "today",
                 "item": "{done|true=[x];false=[ ]} {text}",
                 "keys": {"x": {"host": "done", "args": {"id": "{id}"}}, "z": {"host": "nope"}}},
        "data": {"today": {"host": "today", "args": {"limit": 2}, "path": "rows"}}
    }))
    .unwrap()
}

#[test]
fn a_host_source_fills_the_screen_and_a_host_action_changes_it() {
    let host = Todos::default();
    let mut screen = App::new(ui()).host(host.clone()).headless(40, 6).unwrap();
    let text = screen.screen();
    assert!(text.contains("▸ [ ] water the plants"), "{text}");
    assert!(!text.contains("write the post"), "args limit the rows: {text}");

    screen.keys(&["j", "x"]);
    assert_eq!(*host.done.lock().unwrap(), vec!["t1".to_string()]);
    let text = screen.screen();
    assert!(text.contains("[x] call the bank"), "the action refetches host sources: {text}");
    let log = &screen.state().data["$log"];
    assert!(serde_json::to_string(log).unwrap().contains("\"op\":\"host\""), "{log:?}");
}

#[test]
fn a_failed_host_action_says_why() {
    let mut screen = App::new(ui()).host(Todos::default()).headless(60, 6).unwrap();
    screen.keys(&["z"]);
    assert_eq!(screen.state().status.as_deref(), Some("nope: can't nope"));
}

#[test]
fn the_host_tells_the_screen_when_its_world_changed() {
    let host = Todos::default();
    let mut screen = App::new(ui()).host(host.clone()).headless(40, 6).unwrap();
    assert!(screen.screen().contains("[ ] water the plants"));
    // The app's world changes behind the screen's back (the daemon saw a file change).
    host.done.lock().unwrap().push("t0".into());
    assert!(screen.screen().contains("[ ] water the plants"), "nothing refetches by itself");
    host.notifier.lock().unwrap().as_ref().unwrap().changed("today");
    assert!(screen.screen().contains("[x] water the plants"));
}

#[test]
fn a_screen_without_a_host_says_so() {
    let mut screen = App::new(ui()).headless(60, 6).unwrap();
    assert!(screen.screen().contains("today: this screen has no host source \"today\""));
}

/// A vault: pages the host keeps, which the screen edits.
#[derive(Clone, Default)]
struct Vault {
    pages: Arc<Mutex<std::collections::BTreeMap<String, String>>>,
    loads: Arc<Mutex<Vec<String>>>,
    notifier: Arc<Mutex<Option<Notifier>>>,
}

impl Host for Vault {
    fn start(&self, notifier: Notifier) {
        *self.notifier.lock().unwrap() = Some(notifier);
    }
    fn keys(&self) -> Vec<String> {
        vec!["page/".into()]
    }
    fn load(&self, key: &str) -> Result<Value, String> {
        self.loads.lock().unwrap().push(key.into());
        let name = key.strip_prefix("page/").unwrap();
        self.pages.lock().unwrap().get(name).cloned().map(Value::String).ok_or_else(|| format!("no page {name}"))
    }
    fn save(&self, key: &str, value: &Value) -> Result<(), String> {
        let name = key.strip_prefix("page/").unwrap().to_string();
        self.pages.lock().unwrap().insert(name, value.as_str().unwrap().into());
        Ok(())
    }
}

fn pages_ui() -> fictty::model::Ui {
    serde_json::from_value(json!({
        "root": {"type": "row", "children": [
            {"type": "list", "id": "list", "size": 12, "item": "{name}", "items": [{"name": "home"}, {"name": "ideas"}, {"name": "gone"}]},
            {"type": "editor", "id": "page", "bind": "@list", "doc": "page/{name}", "text": "never a seed"}
        ]}
    }))
    .unwrap()
}

#[test]
fn hosted_entries_load_when_shown_and_save_when_changed() {
    let vault = Vault::default();
    vault.pages.lock().unwrap().insert("home".into(), "# Home".into());
    vault.pages.lock().unwrap().insert("ideas".into(), "- fly".into());
    let mut screen = App::new(pages_ui()).host(vault.clone()).headless(50, 6).unwrap();
    assert!(screen.screen().contains("# Home"));
    assert_eq!(*vault.loads.lock().unwrap(), vec!["page/home".to_string()], "only what shows loads");

    // Typing saves back (a headless screen saves at once; a running one after half a second).
    screen.keys(&["tab", "c-end", "!"]);
    assert_eq!(vault.pages.lock().unwrap()["home"], "# Home!");

    // The next page loads when selected; the first stays in the store.
    screen.keys(&["tab", "j"]);
    assert!(screen.screen().contains("- fly"));
    assert_eq!(vault.loads.lock().unwrap().len(), 2);

    // The file changes on disk: the host says so, and the text comes in as an edit.
    vault.pages.lock().unwrap().insert("ideas".into(), "- fly\n- swim".into());
    vault.notifier.lock().unwrap().as_ref().unwrap().reload("page/ideas");
    assert!(screen.screen().contains("- swim"));
    assert_eq!(screen.state().store["page/ideas"].by, "host");
    assert_eq!(vault.pages.lock().unwrap()["ideas"], "- fly\n- swim", "nothing saved back");

    // A page the host can't load says why, once.
    screen.keys(&["j"]);
    let text = screen.screen();
    assert!(text.contains("page/gone: no page gone"), "{text}");
    assert!(!text.contains("never a seed"), "{text}");
    screen.keys(&["k", "j"]);
    assert_eq!(vault.loads.lock().unwrap().iter().filter(|k| *k == "page/gone").count(), 1);
}

/// A host whose detail source answers for the id it's asked about.
#[derive(Clone, Default)]
struct Detail {
    asked: Arc<Mutex<Vec<Value>>>,
}

impl Host for Detail {
    fn fetch(&self, name: &str, args: &Value) -> Result<Value, String> {
        self.asked.lock().unwrap().push(args.clone());
        match name {
            "detail" => Ok(json!({"about": format!("all about {} ({})", args["id"].as_str().unwrap_or("?"), args["q"])})),
            _ => Err("no".into()),
        }
    }
}

#[test]
fn a_host_source_reads_its_args_from_the_selection_and_the_store() {
    let ui: fictty::model::Ui = serde_json::from_value(json!({
        "root": {"type": "row", "children": [
            {"type": "list", "id": "l", "item": "{id}", "items": [{"id": "a"}, {"id": "b"}], "size": 6},
            {"type": "col", "children": [{"type": "input", "id": "q", "size": 1}, {"type": "text", "bind": "detail", "text": "{about}"}]}
        ]},
        "data": {"detail": {"host": "detail", "args": {"id": "{@l.id}", "q": "{store.q}", "n": "{l}"}}},
        "focus": "l"
    }))
    .unwrap();
    let host = Detail::default();
    let mut s = App::new(ui).host(host.clone()).headless(50, 4).unwrap();
    assert!(s.screen().contains("all about a (\"\")"), "{}", s.screen());
    s.keys(&["j"]);
    assert!(s.screen().contains("all about b"), "{}", s.screen());
    s.keys(&["k"]);
    let n = host.asked.lock().unwrap().len();
    s.keys(&["k"]);
    assert_eq!(host.asked.lock().unwrap().len(), n, "same args, no fetch");
    s.keys(&["tab", "h", "i"]);
    assert!(s.screen().contains("all about a (\"hi\")"), "{}", s.screen());
    assert_eq!(s.state().args["detail"]["id"], "a");
}
