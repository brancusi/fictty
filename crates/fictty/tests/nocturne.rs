//! Nocturne (`examples/nocturne/`), the listening room, in the native window: written by its
//! script without the art (`--no-art`, no Pillow), then driven as a person would and read back.
#![cfg(target_os = "macos")]

use fictty::App;
use fictty::readback::Shown;
use std::path::PathBuf;

fn nocturne(w: u16, h: u16) -> fictty::Headless {
    let out = std::env::temp_dir().join(format!("fictty-nocturne-test-{}-{w}x{h}", std::process::id()));
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/nocturne/nocturne.py");
    let done = std::process::Command::new("python3").arg(script).arg(&out).arg("--no-art").output().expect("python3");
    assert!(done.status.success(), "{}", String::from_utf8_lossy(&done.stderr));
    let ui = serde_json::from_str(&std::fs::read_to_string(out.join("nocturne.json")).unwrap()).unwrap();
    App::new(ui).themes_file(&out.join("themes.json")).unwrap().in_window(w, h).unwrap()
}

fn shown(s: &mut fictty::Headless) -> Vec<Shown> {
    s.readback()
}

fn text(s: &mut fictty::Headless, id: &str) -> String {
    shown(s).into_iter().find(|n| n.id.as_deref() == Some(id)).map(|n| n.text).unwrap_or_default()
}

fn has(s: &mut fictty::Headless, id: &str) -> bool {
    shown(s).iter().any(|n| n.id.as_deref() == Some(id))
}

fn press(s: &mut fictty::Headless, id: &str) {
    let (x, y) = s.at(id, None, None).unwrap_or_else(|| panic!("no {id} to press"));
    s.mouse("press", x, y).mouse("release", x, y);
}

#[test]
fn it_fits_a_laptop_and_a_big_screen_with_nothing_cut() {
    for (w, h) in [(300, 56), (200, 45), (171, 36)] {
        let mut s = nocturne(w, h);
        assert_eq!(s.state().problems, Vec::<String>::new(), "{w}x{h}");
        let cut: Vec<String> = shown(&mut s).into_iter().filter(|n| n.cut).map(|n| format!("{} {:?}: {}", n.kind, n.id, n.text)).collect();
        assert!(cut.is_empty(), "{w}x{h} cut {cut:?}");
        assert!(has(&mut s, "tracks") && has(&mut s, "hero") && has(&mut s, "albums"), "{w}x{h}");
        assert_eq!(has(&mut s, "side"), w >= 230, "the sidebar only where there's room for it");
        // One shelf, as many covers to a line as fit at 200 px each.
        let albums = shown(&mut s).into_iter().find(|n| n.id.as_deref() == Some("albums")).unwrap();
        let fit = (albums.rect.unwrap().width as f32 * 8.4 / 200.) as usize;
        assert!(albums.cols.is_some_and(|c| c >= 3 && c.abs_diff(fit) <= 1), "{w}x{h}: {:?} to a line, about {fit} fit", albums.cols);
    }
}

#[test]
fn the_whole_room_follows_the_album_chosen() {
    let mut s = nocturne(300, 56);
    assert!(text(&mut s, "room").ends_with("harbour-lights-room.png"));
    s.keys(&["right", "right"]);
    assert!(text(&mut s, "room").ends_with("paper-moons-room.png"), "the room");
    assert!(text(&mut s, "art").ends_with("paper-moons.png"), "the hero");
    assert!(text(&mut s, "tracks").contains("Kite Season"), "up next");
    assert!(text(&mut s, "lyrics").contains("we cut the moon"), "the lyrics");
}

#[test]
fn space_plays_love_loves_and_search_narrows_the_shelf() {
    let mut s = nocturne(300, 56);
    assert!(has(&mut s, "play") && !has(&mut s, "pause"));
    s.keys(&["space"]);
    assert!(has(&mut s, "pause"), "space plays");
    assert_eq!(text(&mut s, "progress"), "40%", "1:38 of 4:08, as seeded");
    s.keys(&["down"]);
    let line = shown(&mut s).into_iter().find(|n| n.id.as_deref() == Some("albums")).and_then(|n| n.cols).unwrap();
    assert_eq!(s.state().local["albums"].selected, line, "↓ goes a line of covers");
    let room = text(&mut s, "room");
    let slug = room.rsplit('/').next().unwrap().trim_end_matches("-room.png");
    press(&mut s, "love");
    assert!(has(&mut s, "loved"), "the album shown is loved");
    assert_eq!(s.state().store[&format!("loved/{slug}")].read()["value"], true, "the one a line down");
    press(&mut s, "next");
    assert!(text(&mut s, "tracks").starts_with("02"), "next moves Up Next on");
    assert_eq!(s.state().focus.as_deref(), Some("albums"), "and gives the focus back to the shelf");
    s.msg(fictty::state::Msg::Focus { id: "find".into() });
    s.keys(&["S", "o", "d"]);
    let albums = shown(&mut s).into_iter().find(|n| n.id.as_deref() == Some("albums")).unwrap();
    assert_eq!(albums.rows, Some(1), "Sodium alone");
}

#[test]
fn browse_puts_the_shelf_where_the_hero_was() {
    let mut s = nocturne(300, 56);
    let (x, y) = s.at("nav", Some(1), None).unwrap();
    s.mouse("press", x, y).mouse("release", x, y);
    assert!(!has(&mut s, "hero"), "no hero");
    let albums = shown(&mut s).into_iter().find(|n| n.id.as_deref() == Some("albums")).unwrap();
    assert!(albums.rect.unwrap().height > 40, "the shelf takes the room: {:?}", albums.rect);
}
