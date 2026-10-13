//! The real binary in a pseudo-terminal: keys as the bytes a terminal sends (decoded by crossterm,
//! as on a real screen), the mouse as SGR reports, pastes bracketed, and what it draws read by a
//! terminal emulator.

use fictty::sim::Sim;
use serde_json::{Value, json};

fn ui_file(name: &str, ui: Value) -> String {
    let dir = std::env::temp_dir().join(format!("fictty-sim-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, ui.to_string()).unwrap();
    path.display().to_string()
}

fn sim(name: &str, ui: Value, kitty: bool) -> Sim {
    let path = ui_file(name, ui);
    Sim::spawn_with(&[env!("CARGO_BIN_EXE_fictty"), "run", &path], 60, 12, kitty, &[]).unwrap()
}

fn text(s: &mut Sim) -> String {
    s.request(json!({"op": "read", "key": "e"}))["text"].as_str().unwrap_or_default().to_string()
}

fn editor() -> Value {
    json!({"root": {"type": "col", "children": [
        {"type": "editor", "id": "e", "text": "hello world"},
        {"type": "list", "id": "l", "items": ["one", "two", "three"]}
    ]}, "focus": "e"})
}

#[test]
fn keys_reach_the_screen_through_the_terminals_bytes() {
    let mut s = sim("keys", editor(), true);
    assert_eq!(s.modes.kitty.last().map(|f| f & 1), Some(1), "the screen pushed the kitty flags: {:?}", s.modes);
    // ⌘→ to the end, ⌥⇧← selects a word, typing replaces it, ⌘Z takes it back.
    s.keys(&["d-right", "a-s-left", "s-t", "h", "e", "r", "e"]);
    assert_eq!(text(&mut s), "hello There");
    s.keys(&["d-z"]);
    assert_eq!(text(&mut s), "hello world");
    // ⌘A then a paste replaces everything.
    s.keys(&["d-a"]).paste("pasted 漢字");
    assert_eq!(text(&mut s), "pasted 漢字");
    s.check().unwrap();
}

#[test]
fn a_terminal_without_the_kitty_protocol_gets_legacy_keys() {
    let mut s = sim("legacy", editor(), false);
    assert!(s.modes.kitty.is_empty(), "{:?}", s.modes);
    s.keys(&["end", "a-b", "c-a"]);
    s.keys(&["x"]);
    assert_eq!(text(&mut s), "xhello world", "Ctrl-A went to the line's start");
}

#[test]
fn the_mouse_and_a_resize_arrive_as_a_terminal_sends_them() {
    let mut s = sim("mouse", editor(), true);
    let screen = s.screen();
    let y = screen.lines().position(|l| l.contains("three")).unwrap() as u16;
    let x = screen.lines().nth(y as usize).unwrap().find("three").unwrap() as u16;
    s.mouse("press", x, y).mouse("release", x, y);
    let st = s.state();
    assert_eq!(st["focus"], "l", "{st}");
    s.resize(40, 8);
    s.check().unwrap();
    assert_eq!(s.vt().size(), (8, 40));
}

#[test]
fn random_acts_leave_the_real_screen_where_they_leave_a_headless_one() {
    // Through the pty, keys are bytes crossterm decodes; headless, they're names. The same acts
    // must leave the same text and focus, with an agent's writes landing between keys. Undo is
    // left out: it groups typing by the time between keys, and a headless screen's clock stands
    // still.
    use fictty::fuzz::{self, Config};
    let ui = json!({"root": {"type": "col", "children": [
        {"type": "editor", "id": "e", "text": "# Title\nhello world, a line\n- one\n  - two\nwide 漢字 🙂\n"},
        {"type": "list", "id": "l", "items": ["one", "two", "three"]}
    ]}, "focus": "e"});
    let keys = Config::default().keys.into_iter().filter(|k| !k.ends_with("-z") && k != "c-t").collect();
    // An agent edits the text now and then, while the person types.
    let host_acts = vec!["0".into(), "8".into(), "put".into()];
    let cfg = Config { keys, combos: vec![], steps: 30, wait: 0, resize: 0, host_acts, host: 8, ..Config::default() };
    let write = |arg: &str| match arg.parse::<usize>() {
        Ok(at) => json!({"op": "write", "key": "e", "write": "edit", "changes": [[at, at, "AGENT"]]}),
        Err(_) => json!({"op": "write", "key": "e", "write": "put", "value": "# Put\nby an agent\n"}),
    };
    let runs = std::env::var("SIM_RUNS").ok().and_then(|v| v.parse().ok()).unwrap_or(3u64);
    for seed in 0..runs {
        let acts = fuzz::generate(&cfg, 100 + seed, (60, 12));
        let mut h = fictty::App::new(serde_json::from_value(ui.clone()).unwrap()).headless(60, 12).unwrap().on_host_act(move |h, a| {
            h.request(write(a));
        });
        let mut s = sim(&format!("diff{seed}"), ui.clone(), true).on_host_act(move |s, a| {
            s.request(write(a));
        });
        for (i, a) in acts.iter().enumerate() {
            // One clock for both: the real screen runs on the wall clock, and the test's own
            // round trips make its presses land further apart than a headless screen's, so three
            // presses were a triple click to one and three single clicks to the other.
            if let Some(now_ms) = s.state()["now_ms"].as_u64() {
                h.msg(fictty::state::Msg::Tick { now_ms });
            }
            a.apply(&mut h);
            s.act(a);
            if h.quit() {
                assert!(!s.running(), "seed {seed}: headless quit after {}, the real screen didn't", fuzz::line(&acts[..=i]));
                break;
            }
            let headless = (h.state().focus.clone(), fuzz::editor(&mut h, "e").map(|e| e.0));
            let real = (s.state()["focus"].as_str().map(String::from), Some(text(&mut s)));
            assert_eq!(real, headless, "seed {seed}, after {}", fuzz::line(&acts[..=i]));
            if let Err(e) = s.check() {
                panic!("seed {seed}, after {}: the terminal doesn't show the frame: {e}", fuzz::line(&acts[..=i]));
            }
        }
    }
}
