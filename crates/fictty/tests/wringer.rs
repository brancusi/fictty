//! The wringer's screens (`examples/window/`) rebuilt without tricks in steps 3 and 4, in the native
//! window: each in every theme at a big window and a small one, with no problems and nothing cut,
//! and what each rebuild was for.
#![cfg(target_os = "macos")]

use fictty::App;
use fictty::readback::Shown;
use std::path::PathBuf;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/window")
}

fn screen(name: &str, theme: &str, w: u16, h: u16) -> fictty::Headless {
    let ui = serde_json::from_str(&std::fs::read_to_string(dir().join(format!("{name}.json"))).unwrap()).unwrap();
    App::new(ui).themes_file(&dir().join("themes.json")).unwrap().theme(theme).in_window(w, h).unwrap()
}

fn node(s: &mut fictty::Headless, id: &str) -> Shown {
    s.readback().into_iter().find(|n| n.id.as_deref() == Some(id)).unwrap_or_else(|| panic!("no node {id}"))
}

#[test]
fn the_rebuilt_screens_wear_every_theme_with_nothing_cut() {
    let themes: Vec<String> = fictty::engine::load_themes(&dir().join("themes.json")).unwrap().into_keys().collect();
    for name in ["settings", "landing", "calendar", "chat", "kanban"] {
        for theme in &themes {
            for (w, h) in [(160, 44), (110, 30)] {
                let mut s = screen(name, theme, w, h);
                assert_eq!(s.state().problems, Vec::<String>::new(), "{name} in {theme} at {w}x{h}");
                let cut: Vec<String> = s.readback().into_iter().filter(|n| n.cut).map(|n| format!("{} {:?}: {}", n.kind, n.id, n.text)).collect();
                assert!(cut.is_empty(), "{name} in {theme} at {w}x{h} cut {cut:?}");
            }
        }
    }
}

/// The form scrolls on a short window, and Tab down it brings each field into view.
#[test]
fn settings_scrolls_its_form_to_the_field_in_focus() {
    let mut s = screen("settings", "glass", 110, 18);
    assert!(node(&mut s, "form").scroll.is_some(), "a short window holds less than the form");
    let form = node(&mut s, "form").rect.unwrap();
    // A label is centred on its field: on its row.
    s.keys(&["tab"; 8]);
    let focus = s.state().focus.clone().unwrap();
    let f = node(&mut s, &focus).rect.unwrap_or_else(|| panic!("{focus} shows"));
    assert!(f.y >= form.y && f.y + f.height <= form.y + form.height, "{focus} {f:?} in the form {form:?}");
}

/// The page scrolls, and the features go as many to a line as fit: three on a wide window, fewer
/// on a narrow one, lined up.
#[test]
fn landing_wraps_its_features_and_scrolls() {
    let mut wide = screen("landing", "paper", 160, 44);
    assert_eq!(node(&mut wide, "features").cols, Some(3));
    let mut narrow = screen("landing", "paper", 70, 30);
    assert!(node(&mut narrow, "features").cols.is_none_or(|c| c < 3), "fewer to a line on a narrow window");
    assert!(node(&mut narrow, "page").scroll.is_some(), "and the page scrolls");
}

/// Each event floats where its data puts it: two rows an hour from the day's top, as tall as it
/// lasts.
#[test]
fn calendar_events_sit_at_their_times() {
    let mut s = screen("calendar", "glass", 160, 30);
    let (standup, review) = (node(&mut s, "ev-0-1").rect.unwrap(), node(&mut s, "ev-0-4").rect.unwrap());
    assert_eq!(review.y - standup.y, 6, "10:00 to 13:00 is three hours: {standup:?} {review:?}");
    assert_eq!((standup.height, review.height), (4, 6), "two hours and three");
    let thu = node(&mut s, "ev-3-6").rect.unwrap();
    assert!(thu.x > review.x, "Thursday's to the right of Monday's");
}

/// A message is as tall as it wraps to: the long one takes more lines than a short one.
#[test]
fn chat_messages_are_as_tall_as_their_text() {
    let mut s = screen("chat", "glass", 140, 34);
    let h = s.state().local["thread"].heights.clone();
    assert!(h["3"] > h["5"] + 0.5, "the long message is taller: {h:?}");
}

/// A card is as tall as its title wraps to, in its own column's list.
#[test]
fn kanban_cards_are_as_tall_as_their_titles() {
    let mut s = screen("kanban", "paper", 140, 34);
    let h = s.state().local["col-todo"].heights.clone();
    assert!(h["todo1"] > h["todo0"] + 0.5, "the long title's card is taller: {h:?}");
    assert_eq!(node(&mut s, "col-doing").rows, Some(2), "each column lists its own cards");
}
