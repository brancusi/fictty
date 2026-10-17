# The wringer

Nine screens a designer might hand over, built from what fictty has today, and six themes they
can wear. They are the test of done for [docs/design/expressive.md](../../docs/design/expressive.md):
each screen gets rebuilt without tricks as that plan lands, and the themes get pushed until three
of them make the same screen look like three different apps.

```sh
mise exec -- cargo build --release
examples/window/open.sh mail                 # one screen, the first theme (night-city)
examples/window/open.sh kanban brutal        # in another theme
examples/window/open.sh all                  # every screen, a window each
fictty --socket /tmp/fw-kanban.sock theme paper          # switch a running one
fictty --socket /tmp/fw-kanban.sock screen --nodes       # read it back
python3 examples/window/screens.py           # regenerate the seven generated screens
```

`mail.json` and `ops.json` are written by hand; `screens.py` writes the other seven, and
`themes.py` writes `themes.json`. What the styling study found is in
[docs/design/styling-study.md](../../docs/design/styling-study.md).
`ops` streams live numbers and a log from `feed.py` (a `merge` and a `put` every 0.7 s).
Themes: `night-city` (hard edges, acid yellow on near-black), `glass`, `paper`, `phosphor`,
`brutal`, `soft` (`themes.json`).

Windows only draw while they're in front (macOS stops drawing covered windows). For a screenshot,
bring the window forward and use `screencapture -x -o` (or `-R x,y,w,h` for just the window),
until `fictty screenshot` lands in phase 2.

## The screens, their tricks, and what retires each trick

| screen | what it is | tricks today | done when (expressive.md step) |
|---|---|---|---|
| `mail` | folders, a message list, a reader, a command palette | none | it looks finished in all six themes after step 2 |
| `ops` | live stats, services, deploys, a streaming log | none | the same, with its numbers in a large type role (step 2) |
| `settings` | a form: fields, toggles, a radio group, a select, a slider, a save bar | checkboxes are list rows that toggle; the select is a pop-up list; the slider is a gauge that can't move | ~~labels beside fields~~ (`items: center`, `fit`) and ~~the form scrolls~~ (`scroll`), done in step 3; ~~the radio group~~ is tabs without pages; the checkbox and select are written-down recipes; the slider is `slider` with `add` (phase 2) |
| `chat` | rooms, a thread with bubbles left and right, a composer | none since step 4 | done: each message is a row template, a bubble as wide as its text up to 44 cells (`fit`, `max`), justified to its side and wearing `hero` or `card` from its data (`{$item.side|me=end;*=start}`), as tall as it wraps to (`fit` rows) |
| `kanban` | four columns of cards | none since step 4; moving cards is phase 2's | done: each column is a list of cards, a row template with a title, a tag and a size, as tall as its title wraps to (step 4); they move between columns by mouse and key with `drag`, `drop` and `move` (phase 2) |
| `player` | album art, title, progress, transport, up next | the art is an `svg` placeholder in the window; the title is plain text | the art draws (`svg`, native step 5 / phase 3); the title is a `display` type role (step 2); progress is a `slider` |
| `landing` | a hero, calls to action, a feature row, pricing | none since step 3 | done: `display` type (step 2); the page scrolls, the calls to action and nav are justified, and the features and prices are grids of as many as fit, up to three (step 3) |
| `calendar` | a week, with events at times and of different lengths | none since step 3 | done: each day's events are data, an `each` with `stack`, floated at `y` from the start, as tall as they last (step 3) |
| `sheet` | a dense table with heat colours and totals | none since step 4 | done: column widths and alignment the same over the header, rows and totals, heat tints from the theme's `neg` to `pos`, totals in a footer (4c) |

## Pushing the styling

The point of the six themes isn't six palettes. It's to find out how far a theme can move a
screen's character, and which small things move it most. Step 2 of expressive.md (themes as whole
styles) is where this happens: as each part property lands, try it across the six themes on
`mail`, `settings` and `landing`, and keep a before and after for anything that changes the feel.

What to try, in rough order of how much it tends to matter:

- **Type.** A font per role (a display face, a UI face, a mono for numbers). Tabular numbers in
  tables and stats. Letter spacing on small caps labels (`MESSAGES`, `TO DO`). Line height: looser
  for body text, tighter for headings. Weight contrast between a title and its body.
- **Text colour levels.** Three steps (text, secondary, faint), not two. Neutrals tinted towards
  the accent's hue rather than pure grey.
- **Spacing rhythm.** One base unit (4 px) for every pad and gap. More space between groups than
  inside them. A panel's title on the same left edge as its content.
- **Edges.** Hairline borders at low contrast versus none at all, with surfaces told apart by
  colour alone. Nested radii (an inner radius is the outer one minus the padding). Square for
  brutal and night-city.
- **Depth.** Two soft shadows (one wide and faint, one close) instead of one. A glow for neon. A
  hard offset shadow with no blur for brutal. Nothing at all for paper.
- **States.** Hover as a small shift in the surface, not a new colour. A focus ring with an offset
  and its own colour. Selection that keeps the text readable (in some of the wringer's themes it
  didn't). Disabled as reduced opacity.
- **Surfaces.** How far a panel sits from the background. A gradient of a few percent. The scrim's
  colour and opacity under a dialog.
- **Density.** Row heights and padding as a theme setting: compact versus comfortable.
- **Details.** Scrollbars thin and over the content. The caret's width and colour. Dividers inset
  from the edge. Empty states in italics and the faint colour.
- **Motion** (once phase 3 lands). Hover at about 120 ms, a press at about 80 ms, a toast that
  slides 8 px while it fades in.

Write each finding down where the next person will find it: what changed, in which themes, and
the screenshots. Keep the theme in `themes.json` that shows it best.
