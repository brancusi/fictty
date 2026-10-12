//! The fuzz driver over screens with editors and the components around them: random keys, the
//! mouse, pastes and resizes, with the built-in properties and the editing invariants.
//! `FUZZ_RUNS=2000 cargo test --release --test fuzz` runs longer; `FUZZ_SEED` picks another seed.

use fictty::App;
use fictty::fuzz::{self, Config};
use serde_json::{Value, json};

fn cfg() -> Config {
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse().ok());
    // A debug build is slow: a few runs there, more in release, many with FUZZ_RUNS.
    let runs = if cfg!(debug_assertions) { 6 } else { 40 };
    Config { runs: env("FUZZ_RUNS").unwrap_or(runs) as u32, seed: env("FUZZ_SEED").unwrap_or(1), ..Config::default() }
}

fn screen(ui: Value, w: u16, h: u16) -> impl Fn() -> fictty::Headless {
    move || App::new(serde_json::from_value(ui.clone()).unwrap()).headless(w, h).unwrap()
}

fn fuzzed(ui: Value, w: u16, h: u16) {
    if let Err(f) = fuzz::run(screen(ui, w, h), &mut fuzz::editing(), &cfg()) {
        panic!("{f}");
    }
}

#[test]
fn an_editor() {
    let text = "# Notes\n\nHello world, a line\n- one\n  - nested\n- [ ] a task\n\n1. first\n2. second\n\nwide 漢字 and 🙂\n";
    let links = json!([{"kind": "wiki"}, {"kind": "tag"}, {"kind": "url"}]);
    fuzzed(json!({"root": {"type": "editor", "id": "e", "text": text, "syntax": "markdown", "links": links}, "focus": "e"}), 40, 10);
}

#[test]
fn an_outliner_with_ordered_lists() {
    let text = "- one\n  - two\n    - three\n1. a\n   a. b\n2. c\n- [x] done\n";
    let ed = json!({"type": "editor", "id": "e", "text": text, "tab": "indent-lines", "ordered": ["1.", "a.", "i."], "tasks": "cycle"});
    fuzzed(json!({"root": ed, "focus": "e", "keys": {"c-l": {"list_style": "next"}}}), 40, 10);
}

#[test]
fn editors_beside_lists_and_inputs() {
    let ui = json!({
        "root": {"type": "row", "children": [
            {"type": "list", "id": "pages", "items": ["alpha", "beta", "gamma", "delta"]},
            {"type": "col", "children": [
                {"type": "input", "id": "title", "text": "a title"},
                {"type": "editor", "id": "body", "text": "Some text with [[a link]] and #tag\n- item\n", "links": [{"kind": "wiki", "click": {"put": "opened", "value": "{$match}"}}, {"kind": "tag", "click": {"toast": "#{$match}"}}]},
                {"type": "tabs", "id": "t", "tabs": ["one", "two"], "children": [{"type": "text", "text": "first"}, {"type": "text", "text": "second"}]}
            ]}
        ]},
        "focus": "body"
    });
    fuzzed(ui, 60, 14);
}

#[test]
fn an_editor_beside_a_read_only_preview_of_its_text() {
    // Undoing everything goes through the editor, not the preview (Thought Control's fuzzer: the
    // check focused the preview, where undo does nothing).
    fuzzed(json!({"root": {"type": "row", "children": [
        {"type": "editor", "id": "a_preview", "doc": "page", "readonly": true},
        {"type": "editor", "id": "body", "doc": "page", "text": "Some text\n- a list\n"}
    ]}, "focus": "body"}), 60, 10);
}

#[test]
fn a_failure_shrinks_and_prints_a_scenario() {
    // A property that fails once anything is typed: the shrunk sequence is one key.
    let mut checks = vec![fuzz::check("never typed", |s| match s.after.focused() {
        Some((_, text, ..)) if text != "start" => Err(format!("typed: {text:?}")),
        _ => Ok(()),
    })];
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "start"}, "focus": "e"});
    let cfg = Config { keys: vec!["left".into(), "right".into(), "a".into()], combos: vec![], mouse: 0, paste: 0, resize: 0, wait: 0, ..cfg() };
    let f = fuzz::run(screen(ui.clone(), 30, 4), &mut checks, &cfg).unwrap_err();
    assert_eq!(f.property, "never typed");
    assert_eq!(fuzz::line(&f.acts), "key a", "{f}");
    let shown = f.to_string();
    assert!(shown.contains("start   e: start▮") || shown.contains("start   e: ▮start"), "{shown}");
    assert!(shown.contains("key a"), "{shown}");
    // And the printed sequence replays to the same failure.
    let again = fuzz::replay(screen(ui, 30, 4), &fuzz::line(&f.acts), &mut checks, &cfg).unwrap_err();
    assert_eq!(again.property, "never typed");
}

#[test]
fn every_chapter_of_the_tour() {
    // The tour's chapters together use every component. Their buttons run scripts and the speed
    // chapter streams from awk, so commands are off: a fuzzer never runs anything.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase");
    let mut keys = Config::default().keys;
    keys.extend(["n", "p", "j", "k", "o", "t", "g", "s-g", "/", "?", "1", "2", "3"].map(String::from));
    let cfg = Config { keys, runs: cfg().runs / 4 + 1, ..cfg() };
    for n in 1..=10 {
        let path = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.file_name().and_then(|f| f.to_str()).is_some_and(|f| f.starts_with(&format!("tour-{n}-"))))
            .expect("a chapter");
        let themes = dir.join("themes.json");
        let make = || fictty::App::file(&path).themes_file(&themes).unwrap().commands(false).headless(100, 32).unwrap();
        if let Err(f) = fuzz::run(make, &mut fuzz::editing(), &cfg) {
            panic!("{}: {f}", path.display());
        }
    }
}

#[test]
fn host_acts_run_inside_the_step_and_replay() {
    // An agent writes to the text the person is typing in: the app's handler runs inside the
    // act, so the look after it already has the write.
    let ui = json!({"root": {"type": "editor", "id": "e", "doc": "note", "text": "typed by a person"}, "focus": "e"});
    let make = move || {
        App::new(serde_json::from_value(ui.clone()).unwrap()).headless(40, 6).unwrap().on_host_act(|s, arg| {
            let req = match arg {
                "put" => json!({"op": "write", "key": "note", "write": "put", "value": "from an agent"}),
                _ => json!({"op": "write", "key": "note", "write": "edit", "changes": [[0, 0, "agent: "]]}),
            };
            assert_eq!(s.request(req)["ok"], true, "{arg}");
        })
    };
    let mut checks = fuzz::editing();
    checks.push(fuzz::check("a put shows at once", |s| match (&s.act, s.after.editors.get("e")) {
        (fuzz::Act::Host(a), Some((text, ..))) if a == "put" && text != "from an agent" => Err(format!("after the put: {text:?}")),
        _ => Ok(()),
    }));
    let cfg = Config { host_acts: vec!["put".into(), "prepend".into()], host: 10, ..cfg() };
    if let Err(f) = fuzz::run(&make, &mut checks, &cfg) {
        panic!("{f}");
    }
    // A host act prints and parses back, its argument as JSON.
    let acts = vec![fuzz::Act::Key("a".into()), fuzz::Act::Host("put; \"x\"".into())];
    assert_eq!(fuzz::line(&acts), r#"key a; host "put; \"x\"""#);
    assert_eq!(fuzz::parse_line(&fuzz::line(&acts)).unwrap(), acts);
}

#[test]
fn a_shrink_keeps_to_its_failures_class() {
    // One property, two ways to fail it: typing `a`, or typing `b`. With a class from the
    // message, the shrunk sequence keeps to the way it first failed.
    let mut checks = vec![fuzz::check("no letters", |s| match s.after.focused() {
        Some((_, text, ..)) if text.contains('a') => Err("typed an a".into()),
        Some((_, text, ..)) if text.contains('b') => Err("typed a b".into()),
        _ => Ok(()),
    })];
    let ui = json!({"root": {"type": "editor", "id": "e", "text": ""}, "focus": "e"});
    let base = Config { keys: vec!["a".into(), "b".into()], combos: vec![], mouse: 0, paste: 0, resize: 0, wait: 0, ..cfg() };
    for seed in 0..20 {
        let cfg = Config { seed, class: Some(|_, m| m.to_string()), ..base.clone() };
        let f = fuzz::run(screen(ui.clone(), 20, 3), &mut checks, &cfg).unwrap_err();
        let first = fuzz::generate(&cfg, seed.wrapping_mul(1_000_003), (20, 3));
        let wanted = if first[0] == fuzz::Act::Key("a".into()) { "typed an a" } else { "typed a b" };
        assert_eq!(f.message, wanted, "seed {seed}: {f}");
    }
}

#[test]
fn a_normalizer_masks_what_differs_between_runs() {
    // A screen whose state differs from run to run (a host's random id) replays the same once
    // the id is masked.
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let ui = json!({"root": {"type": "editor", "id": "e", "text": "x"}, "focus": "e"});
    let make = move || {
        let mut s = App::new(serde_json::from_value(ui.clone()).unwrap()).headless(20, 3).unwrap();
        let id = N.fetch_add(1, Ordering::Relaxed);
        s.request(json!({"op": "write", "key": "session", "write": "put", "value": {"id": id}}));
        s
    };
    let cfg = Config { runs: 2, steps: 3, keys: vec!["left".into()], combos: vec![], ..cfg() };
    let f = fuzz::run(&make, &mut [], &cfg).unwrap_err();
    assert_eq!(f.property, "replays the same");
    let cfg = Config { normalize: Some(|st| st["store"]["session"] = json!("masked")), ..cfg };
    fuzz::run(&make, &mut [], &cfg).unwrap();
}


/// The same screens drawn by the native window under GPUI's test platform: the mouse goes
/// through the window's own handlers, and editors and lists learn their sizes from its layout.
#[cfg(target_os = "macos")]
mod window {
    use super::*;

    fn fuzzed(ui: Value, w: u16, h: u16) {
        let make = move || App::new(serde_json::from_value(ui.clone()).unwrap()).in_window(w, h).unwrap();
        // No terminal to show the frame: the window is the frame.
        let cfg = Config { terminal: false, ..cfg() };
        if let Err(f) = fuzz::run(make, &mut fuzz::editing(), &cfg) {
            panic!("{f}");
        }
    }

    #[test]
    fn an_editor_in_the_window() {
        let text = "# Notes\n\nHello world, a line\n- one\n  - nested\n- [ ] a task\n\n1. first\n2. second\n\nwide 漢字 and 🙂\n";
        let links = json!([{"kind": "wiki"}, {"kind": "tag"}, {"kind": "url"}]);
        fuzzed(json!({"root": {"type": "editor", "id": "e", "text": text, "syntax": "markdown", "links": links}, "focus": "e"}), 40, 10);
    }

    #[test]
    fn editors_beside_lists_and_inputs_in_the_window() {
        let ui = json!({
            "root": {"type": "row", "children": [
                {"type": "list", "id": "pages", "items": ["alpha", "beta", "gamma", "delta"]},
                {"type": "col", "children": [
                    {"type": "input", "id": "title", "text": "a title"},
                    {"type": "editor", "id": "body", "text": "Some text with [[a link]] and #tag\n- item\n", "links": [{"kind": "wiki", "click": {"put": "opened", "value": "{$match}"}}, {"kind": "tag", "click": {"toast": "#{$match}"}}]},
                    {"type": "tabs", "id": "t", "tabs": ["one", "two"], "children": [{"type": "text", "text": "first"}, {"type": "text", "text": "second"}]}
                ]}
            ]},
            "focus": "body"
        });
        fuzzed(ui, 60, 14);
    }

    #[test]
    fn every_chapter_of_the_tour_in_the_window() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase");
        let mut keys = Config::default().keys;
        keys.extend(["n", "p", "j", "k", "o", "t", "g", "s-g", "/", "?", "1", "2", "3"].map(String::from));
        let cfg = Config { keys, runs: cfg().runs / 4 + 1, terminal: false, ..cfg() };
        for n in 1..=10 {
            let path = std::fs::read_dir(&dir)
                .unwrap()
                .filter_map(|e| e.ok().map(|e| e.path()))
                .find(|p| p.file_name().and_then(|f| f.to_str()).is_some_and(|f| f.starts_with(&format!("tour-{n}-"))))
                .expect("a chapter");
            let themes = dir.join("themes.json");
            let make = || fictty::App::file(&path).themes_file(&themes).unwrap().commands(false).in_window(100, 32).unwrap();
            if let Err(f) = fuzz::run(make, &mut fuzz::editing(), &cfg) {
                panic!("{}: {f}", path.display());
            }
        }
    }

    #[test]
    fn a_long_list_and_a_table_in_the_window() {
        let rows: Vec<Value> = (0..300).map(|i| json!({"name": format!("row {i}"), "n": i})).collect();
        fuzzed(json!({"root": {"type": "row", "children": [
            {"type": "list", "id": "l", "bind": "rows", "item": "{name}\t{n}"},
            {"type": "table", "id": "t", "bind": "rows", "columns": [{"title": "Name", "value": "{name}"}, {"title": "N", "value": "{n}"}]}
        ]}, "store": {"rows": rows}, "focus": "l"}), 60, 14);
    }
}
