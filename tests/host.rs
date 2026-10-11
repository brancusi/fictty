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
