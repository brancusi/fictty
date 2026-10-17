//! The public demo (`examples/demo/`), Kestrel Cycles, as its agent leaves it: the app with all
//! five tabs, written by `agent.py`'s own `app`, in the native window under every theme.
#![cfg(target_os = "macos")]

use fictty::App;
use fictty::readback::Shown;
use serde_json::{Value, json};
use std::path::PathBuf;

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// The app as the agent builds it, with the Floor's numbers in.
fn kestrel() -> Value {
    let demo = examples().join("demo");
    let script = format!(
        "import sys, json; sys.argv = ['agent', 'fictty', 'sock']; sys.path.insert(0, {:?}); import agent\n\
         ui = agent.app(5)\n\
         ui['store']['floor'] = {{'takings': '1,240', 'dtakings': 4, 'stands': 3, 'waiting': 2, 'avg': 47, 'davg': -2, 'walkins': 5}}\n\
         print(json.dumps(ui))",
        demo.display().to_string()
    );
    let out = std::process::Command::new("python3").arg("-c").arg(script).output().expect("python3");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap()
}

fn on_the_floor(theme: &str, w: u16, h: u16) -> fictty::Headless {
    let ui = serde_json::from_value(kestrel()).unwrap();
    let mut s = App::new(ui).themes_file(&examples().join("window/themes.json")).unwrap().theme(theme).in_window(w, h).unwrap();
    s.request(json!({"op": "set", "id": "main", "tab": 3}));
    s
}

fn node(s: &mut fictty::Headless, id: &str) -> Shown {
    s.readback().into_iter().find(|n| n.id.as_deref() == Some(id)).unwrap_or_else(|| panic!("no node {id}"))
}

/// The Floor's cards go as many to a line as fit, up to four, and wrap on a narrow window, whole
/// in every theme.
#[test]
fn the_floors_cards_wrap_and_nothing_is_cut() {
    let themes: Vec<String> = fictty::engine::load_themes(&examples().join("window/themes.json")).unwrap().into_keys().collect();
    for theme in &themes {
        for (w, h) in [(180, 44), (110, 34)] {
            let mut s = on_the_floor(theme, w, h);
            assert_eq!(s.state().problems, Vec::<String>::new(), "{theme} at {w}x{h}");
            let cut: Vec<String> = s.readback().into_iter().filter(|n| n.cut && n.kind != "list").map(|n| format!("{} {:?}: {}", n.kind, n.id, n.text)).collect();
            assert!(cut.is_empty(), "{theme} at {w}x{h} cut {cut:?}");
        }
    }
    let mut wide = on_the_floor("glass", 180, 44);
    assert_eq!(node(&mut wide, "stats").cols, Some(4), "four to a line on a wide window");
    let mut narrow = on_the_floor("glass", 110, 34);
    let per = node(&mut narrow, "stats").cols.unwrap_or(1);
    assert!(per < 4, "fewer on a narrow one: {per}");
    let (a, b) = (node(&mut narrow, "stat0").rect.unwrap(), node(&mut narrow, &format!("stat{per}")).rect.unwrap());
    assert!(b.x == a.x && b.y > a.y, "the card after a full line starts the next: {a:?} {b:?}");
}
