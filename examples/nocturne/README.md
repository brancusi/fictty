# Nocturne

A listening room, made to see how far the window can be pushed: twelve albums with their own
covers, a room behind them in each album's colours, a hero, a shelf, and a rail with what's up
next, the lyrics and a note on the record. Everything follows the album chosen on the shelf,
through binds to `@albums`; the UI is data, with no code in it.

```sh
mise exec -- cargo build --release
examples/nocturne/open.sh                  # needs Pillow (python3 -m pip install pillow)
fictty --socket /tmp/fnoct.sock screen --nodes
```

←/→ and ↑/↓ (or a click) choose an album, space plays and pauses, n and p skip, a click on a
track plays it, ♡ loves the album shown, ⇄ shuffles, the search narrows the shelf, and Browse
gives the shelf the hero's room. `player.py` stands in for an audio engine: it plays no sound,
but keeps time over the socket, reading the screen back as an agent does, moving the progress
on while `playing` holds and selecting the next track at the end of one. It knows the track
lengths from `library.json`, as an engine knows its files, so it keeps time with Up Next hidden.
`open.sh` starts it beside the window, which watches its files: run `nocturne.py` again while
it's open and the new UI and theme show in place.

`nocturne.py` draws every cover and room with Pillow, and writes the theme (`nocturne`) and the
UI beside them, into `$TMPDIR/fictty-nocturne`, so nothing binary is checked in. Every artist,
title, lyric and note is made up.

It fits a big screen and a laptop's: the sidebar goes below 230 cells, the rail below 150,
the room scrolls (`scroll` on the main column: the wheel, Page Down, and the album chosen
brought into view), and the shelf holds as many covers a line as fit at 200
pixels or more each (`cols: {"min": "200px"}`). `tests/nocturne.rs` drives it in the window,
written with `--no-art` (no Pillow).

What it leans on: type roles in three faces (Bodoni 72, Avenir Next, Big Caslon), gradients,
shadows in two layers and in colour, glass (colours with an alpha) over the room, a gauge for
the progress, `aspect` for the art, `fit` for titles, buttons and the rail's cards, `justify`
and `items` for the hero, and a list `row` template with `cols` that fit what there's room for,
for the shelf.
