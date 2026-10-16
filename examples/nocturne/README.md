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

`nocturne.py` draws every cover and room with Pillow, and writes the theme (`nocturne`) and the
UI beside them, into `$TMPDIR/fictty-nocturne`, so nothing binary is checked in. Every artist,
title, lyric and note is made up.

What it leans on: type roles in three faces (Bodoni 72, Avenir Next, Big Caslon), gradients,
shadows in two layers and in colour, `aspect` for the art, `fit` for titles, buttons and the
rail's cards, `justify` and `items` for the hero, a list `row` template with `cols` for the
shelf, and `when` on `$screen.width` for how many covers a line holds.
