# Step 4: rows as nodes

The plan for native.md's step 4, written on 17 October 2026 when step 3's code was done (its
look in a real window still to come). The vocabulary was decided in
[expressive.md](expressive.md) ("Content"); this says where we start, the order, and how we know
it's done. Window-only, as step 3 was.

## Where we start

A list's `row` is already a node template (the gallery brought it forward on 14 October): filled
from `$item`, `{$index}` and `{$count}`, laid out `cols` to a line (a count, as many as fit, or a
size per column), virtualised (only the lines in view are built), its readback the text its
nodes show. What it can't do:

- **Rows are all one height**, the template's `size` in cells. A chat bubble, a kanban card with
  a long title, Nocturne's covers (whose height follows their width) need rows as tall as what
  they show.
- **A row's own buttons don't take clicks.** A press anywhere in a row is a press on the row, so
  a card can't have a ♡ or a ⋯ of its own.
- **Trees and tables** draw their rows from `item` templates and columns only.

And one thing step 3 taught: a list learns its room from the frame (`Local::rows`) as a count of
rows worked out from that one height, and each place that scrolls, pages or follows the
selection does its own arithmetic on it (`view_offset`, `pan`, `in_view`, `page`, the window's
`shown_rows`, the readback's scroll). Variable heights would mean doing all of that twice.

## The order

Each lands with tests in `tests/window.rs`, clean release fuzz runs at 400, and the screens that
use it looked at.

### 4a · One model of a list's view, and `fit` rows (M) · done 17 October 2026

- **`Lines`**, in the state: a list's lines (rows `cols` to a line), each line's height in cells,
  and the view's height in cells. `top` stays in lines, fractions too. Following the selection,
  the wheel, paging, what's in view, what the window builds and the readback's `(scroll a/b)`
  all ask it, so there is one piece of arithmetic for every list.
- **`Local::rows` is the view's height in cells**, as the frame laid the rows' box out (not a
  count of rows); a uniform list's lines are its template's height.
- **`"size": "fit"` on a row template**: each row as tall as what it shows. The frame reports
  the height of each row it built (`Drawn::heights`, by the row's key), the state keeps them in
  `Local::heights` (`Msg::Heights`), and a row not yet built is taken to be as tall as the rows
  measured so far on average (else the template's `lines`). A line is as tall as its tallest
  row. New width, new heights: they're dropped when the list's width changes.
- The wheel moves a list by cells, which `Lines` turns into lines.
- Tests: rows of different heights scroll and follow the selection by their own heights; a
  measured row keeps its height when rows come and go above it; the readback's scroll counts
  lines. Nocturne's shelf drops its fixed card height for `fit`.
- How it landed: `state::Lines` works each line's height out when it's asked for (a list asks
  only about the lines near its view; a list as long as all its lines sums them), since
  working all 50,000 out every time cost a 50k-row list three times its frame. A `fit` list,
  or a list in a scroller, takes its lines' total height as its own, instead of the rows it
  built last: that was a loop (rows built from the height, the height from the rows built)
  that left Nocturne's Up Next one row tall at 152×31. A `cols` line's slots are its height as
  last measured and the cards inside say their own, so Taffy doesn't measure each card twice.
  `in_view` measures where a line sits (a fractional `top` used to be taken as whole).
- Headless, release, best of three, 200 frames of ↓ (layout-step.md's way): a 50,000-row list
  of three-cell rows 10.9 ms a frame, of `fit` rows 12.6; Nocturne with `fit` cards 3.7 ms
  (3.4 with its fixed ones). The fuzz has a screen of `fit` rows and `fit` cards in a scroller,
  clean at 400 release runs.
- Still slow, for later: a row template is filled by turning the node into JSON and back for
  every row built, every frame, which is why a list of node rows costs twice a list of `item`
  text (10.9 ms against 5.4).

### 4b · A row's own buttons (S) · done 17 October 2026

- A button in a row template keeps its `click`. A press on it selects the row, then runs the
  click with that row as the selected one (so `{"toggle": "store.loved/{id}"}` loves that row's
  album). Its hit is the row's with the button's path in the template, so the readback and the
  mouse can name it.
- Keys still go to the list: a row's buttons aren't in the focus order (Enter on a row is the
  list's).
- Tests: a ♡ in a card loves that card's row and not the selected one; a press beside it selects.
- How it landed: `Msg::Press { id, row, part }`; a filled row template keeps its buttons'
  `click` (only theirs); the window sends a press from a button built inside a row, and leaves
  a hit naming the part (`Hit::part`, `Headless::at_part`); the socket's `click` takes `part`.
  It found a bug in clicks: a click on a row didn't set the selection's key, so a click whose
  action wrote the store sent the selection back to the row it left (`step` already guarded
  against that). `a_click_that_writes_keeps_the_row_it_chose` covers it.

### 4c · Trees and tables (M)

- A tree's `row` template, with its indent and fold glyph drawn by the runtime around it.
- A table's cells as templates per column (`cell`), with column widths and alignment.

### 4d · Rebuild (M) · chat and kanban done; Kestrel next

- The wringer's `chat` (bubbles: a row template with `pad`, a radius, `fit` width, `self: end`
  for mine) and `kanban` (cards with a title, a tag chip and a size badge), in every theme, in
  `tests/wringer.rs`.
- Kestrel's inbox and Jobs board as row templates.
- `chat` and `kanban` landed on 17 October 2026, in every theme in `tests/wringer.rs`. What they
  needed and found:
  - **A template's typed fields from data.** `justify`, `items` and `self` are `Filled<T>`: a
    value, or a placeholder kept as written until a template fills it
    (`"justify": "{$item.side|me=end;*=start}"`); unfilled it's a problem. `{$item.x}` takes
    filters as any placeholder does (`picked`, one implementation for both).
  - **A bug in nested templates:** an `each`'s instance filled the `$item` inside a list's `row`
    (and a nested `each`'s `template`) with its own item, so every kanban card's title was
    empty. Filling stops at a template inside the template now.
  - **A `fit` node held by `max`, in a row whose height is `fit`, is measured one line tall.**
    Taffy measures a leaf under a max-content width without the `max` around it and clamps the
    width afterwards, so the height comes from one line; GPUI's text cache answers a
    widthless question with whatever it measured last, which happened to hide it for a share
    with a `max`. A paragraph element of our own measuring each width apart fixed GPUI's half
    and exposed Taffy's, so it's not in; the bubble is a share held to 44 cells, its text only
    as wide as it is with `self`. Fixing it for real is a Taffy change (or measuring `fit` nodes
    ourselves); it's written here so the next screen that meets it knows why.
- Still to do in 4d: Kestrel's inbox and Jobs board.

## Done when

- `chat` and `kanban` run without tricks in every theme, tested, and in screenshots.
- A 50,000-row list of `fit` rows scrolls at the frame time of a uniform one (each row measured
  once, as it first shows), measured headless as in layout-step.md.
- The fuzz is clean at 400 release runs with a screen of `fit` rows and row buttons.
