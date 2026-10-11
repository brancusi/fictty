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
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase");
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


