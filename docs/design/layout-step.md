# Step 3: the layout vocabulary

The plan for native.md's step 3, written on 13 October 2026 when step 2 merged. The vocabulary
itself was decided in [expressive.md](expressive.md) ("The vocabulary": layout on nodes, on
containers, floats); this says where we start, what to settle first, the order, and how we know
it's done. Everything here is window-only: the grid view isn't extended.

## Start here

Rewritten on 14 October 2026, when 3a merged to `main` (88fd94e), for whoever picks up 3b.

**Since then (14 October, to 84f5c47 and the tidy-up after it):** the gallery came first, by
Aram's call: [selection.md](selection.md) is done and [gallery-notes.md](gallery-notes.md) has
what it taught. Three things from it matter to 3b to 3d:

- **A list already scrolls in fractions of a line** (`top: f32`, `Msg::Pan`, scrollbar parts,
  `(scroll a/b)` in the readback). 3d's container scroll should share that shape: an offset in
  `State` in the container's own unit, the same parts, the same readback.
- **A list's `row` is a node template** with `cols`. `size: "fit"` on a row template is where
  `fit` row heights land; do it in 3b's `fit` commit if it falls out, else leave it to step 4.
- **Fill every new field from the bound value the same way** `doc`, `src` and `when` are, and
  read it back as written.

**3a is done:** `pad` and `margin` (one length, `[x, y]` or four sides), `gap` (one or
`[x, y]`), `min` and `max`, all as `Len` in any unit; float `margin` the same type; problems
per node and field for a bad length; `tabs.selected`; font files that don't load as problems.
The Kestrel demo plays in a real window on it.

**What 3a taught:**

- **Never send a `Msg` from inside the window's draw.** The engine is locked while a frame is
  built, so a send from there deadlocks. Whatever the draw learns (a font that didn't load,
  rectangles, hits) goes back with the frame in `Drawn`, and the engine turns it into messages
  after (`report_fonts`). 3b's measured `fit` sizes, if the core ever needs them, go the same way.
- **Measure in the window at one refresh rate, and A/B headless for small differences.** The
  window's `draw_ms` moved by 2× between a 120 Hz and a 60 Hz session for the same code, plasma
  (three nodes) as much as biglist, so it's no good for a before-and-after across sessions. A
  throwaway test in the window's test platform (release, a few hundred frames of plasma, the
  50k-row list and many padded cards, `update`, frame and readback timed together) was steady
  to 2% between runs. Use it to compare two builds, and the stress windows for absolute numbers.
- **A box written in shorthand reads back in shorthand.** `Len` serializes whole cells as
  integers and `Lens`/`Gap` keep the form they were written in (`written()`), so `get --state`
  and `screen --nodes` show `pad 14px 8px` rather than four expanded sides. Keep that for
  `self`, `justify` and `aspect`: read back what was written.
- **A float's margin is padding on its wrapper**, so it works at every anchor and the planned
  "margin on an anchor that can't take it" problem wasn't needed. Look for that kind of answer
  before adding a problem.
- **A renamed part matches silently.** A theme still saying `tabs.active` now styles `tabs` in
  the `active` state, with no problem reported, by choice: nothing outside the repo used it.
  Rename parts early, as this one was.
- **The release fuzz at 400 runs takes about five and a half minutes** (the tour and the long
  list are the slow ones). Run it in the background while writing the guide.

**Next, 3b (fit and alignment), as these commits, each with its tests in `tests/window.rs`:**

1. **`self`, `justify`, `items`.** Done 16 October 2026: `Align` and `Justify` in the core,
   `justify` and `items` beside `gap` on `col`, `row` and `each` (a node-level `items` would
   take a list's), `self` on every node; four tests in the window. No measuring, so it's the safe first commit: `self` on any
   node maps to Taffy's `align_self`; `justify` and `items` on `col`, `row` and `each` to
   `justify_content` and `align_items`. Unknown words fail to parse with a clear error, as
   `Lens` does. Read back as written. The guide's layout section.
2. **`aspect`** (`"16/9"` or a number) to Taffy's `aspect_ratio`, on any node. Done 16 October
   2026. Taffy 0.9 turns only a side the style gives into the other, never one it stretched a
   node to, so an aspect node sits in a place of its own (`place-<key>`, taking its share,
   bounds, margin and `self`) and is 100% of the side it's given there: its `size` along, else
   the room across. Neither (not stretched, no `size`) has nothing to work from; `fit` may give
   it its content's size.
3. **`size: "fit"`.** Done 16 October 2026: `Measure::Fit`, flexbox's `0 1 auto` in the
   window. Text, buttons, titles, wrapped paragraphs and lists measured as they were; three
   things needed more. A cut line is a canvas with no size, so inside a `fit` node it carries an
   unseen, zero-height copy of the whole line to be measured (everywhere, it cost a 50k-row list
   about a quarter of its frame, so `Portal::fitting` counts the `fit` nodes being built). An
   editor's height is its wrapped rows from caretline's `Layout` at its view's width
   (`wrapped_rows`), and since that width comes back from the frame before, `Headless` now draws
   until the sizes settle (a `fit` editor takes two), up to four times. The fuzz has `fitted`: a
   cut `fit` node with room past its row's children, or a share beside it holding some, fails.
   A row template's `fit` is left to step 4. `Size` (`model.rs`, `Size::measure`) gains `Fit`; in the window it's
   Taffy's `auto` basis with no grow, and text is measured by GPUI's own layout. This is the
   risky one (GPUI's ellipsis broke under Taffy's measuring in step 2d): test a `fit` text, button,
   list and editor at many widths in `in_window`, and add a fuzz property that a `fit` node isn't
   cut while its parent has room. A `fit` list is as tall as its rows up to its `max`; an editor
   as tall as its lines.
4. **Daybook**: the spacers either side of the writing column go, replaced by `max` and
   `self: "center"`. (Zen's top margin is already possible with 3a's `margin`; do it here too.)
   Done 16 October 2026, differently: `max` bounds a node along its parent and `self` places it
   across, so neither gives a column a width in a column. The column is `size: 72` in a row
   with `justify: "center"` (a fixed size gives way as `max` would), in the same place as the
   spacers put it at three sizes, zen and not. Zen's margin stays an empty text shown `when:
   "zen"`: a margin can't depend on the store. Filling `margin` (and `pad`) from the bound
   value, as `when` is, would retire it; that, or a cross-axis `max`, is for 3f to decide.
5. **Look** at the wringer's `landing` and `settings` and the Kestrel demo in a real window
   (ask Aram to bring it forward), and run `FUZZ_RUNS=400` in release. The fuzz is clean at 400 runs in release
   (16 October 2026, 315 s, `fitted` included, with a screen of `fit` nodes); the look is
   still to do.

**The baseline**, taken in a release build at 0300ead on an Apple M5 Max, in the window
(`FICTTY_SOCKET=/tmp/fstress.sock fictty run --window …`, then `HOLD=6 examples/stress/play.sh`).
`build` is the window's frame build time (`draw_ms` in `get --stats`). Two runs gave:

| scenario | fps | build ms | max ms |
|---|---|---|---|
| ticker | 104–112 | 0.14 | 0.2 |
| plasma | 106–118 | 0.02–0.03 | 0.0–0.1 |
| biglist | 20 | 0.11 | 0.1 |
| patch storm 1000/s | 110–120 | 0.13–0.14 | 0.2 |
| push storm 60/s | 60 | 0.17–0.22 | 0.4–0.6 |
| push storm flat out | 106–120 | 0.11–0.14 | 0.3 |

**After 3a** (14 October 2026, 762caf3, same machine), two runs each in one session, 3a then
the baseline rebuilt from adae188:

| scenario | 3a fps | 3a build ms | adae188 fps | adae188 build ms |
|---|---|---|---|---|
| ticker | 120 | 0.18–0.20 | 60–62 | 0.14–0.16 |
| plasma | 120 | 0.05–0.06 | 60–61 | 0.02 |
| biglist | 20 | 0.15–0.16 | 20 | 0.08–0.09 |
| patch storm 1000/s | 60–120 | 0.19 | 60 | 0.14–0.18 |
| push storm 60/s | 58–60 | 0.26–0.29 | 38–50 | 0.18–0.21 |
| push storm flat out | 60–122 | 0.26–0.31 | 58–60 | 0.12–0.15 |

The window's numbers say 3a builds slower, but the two ran at different refresh rates (120 Hz
then 60), and plasma, three nodes, moved as much as biglist did, which isn't a per-node cost.
A headless A/B in the window's test platform (release, 400 frames each of plasma, the 50k-row
list and 96 padded cards, update and frame and readback) put both builds within 2% of each other,
twice. So no regression is shown; measure again in the window at one refresh rate before step
3 is called done.

## Where we start

What layout the window has today (step 1, `crates/fictty/src/window/mod.rs`, `sized`):

- **`size`**: cells, `"48px"`, `"30%"`, or shares (`"*"`, `"2*"`), read once in the core
  (`Size::measure`). A fixed size is a flex basis that shrinks when there's no room. No size is
  one share: **nothing sizes to its content.**
- **`min`, `max`**: cells only, along the parent's direction.
- **`pad`**: cells only, `1` (left and right) or `[x, y]`, inside a panel's own inset. A
  style's `pad` (step 2b) is one length for every side, in any unit, so the same word means
  two things today.
- **`gap`**: cells, one number, along the container.
- **`align`**: text alignment, and where a button's face sits.
- **Floats**: a box at one of nine anchors, `margin` in cells, sized by `width` and `height`.
- **Every row and column clips** (`overflow_hidden`). Removing it made the fuzz fail, since a
  squeezed row's children overlapped. A card's shadow is cut at its row's edge, so rows of cards
  take `pad`.
- **Only lists, tables, trees and editors scroll**, each with its own offset in `State`.

What each example does without the vocabulary, and what retires it:

| where | the trick | retired by |
|---|---|---|
| wringer `landing` | pricing and feature cards stretch to fill the screen | `fit`, `wrap`, `justify` |
| wringer `settings` | labels beside fields at fixed sizes; the form is cut off at the bottom | `items: "center"`, `fit`, `scroll` |
| wringer `calendar` | a script computes the gaps between events | floats with `y` and `size` from data |
| Kestrel demo, Floor | four stat cards share the row whatever their content; `pad` on the row for shadows | `fit`, `wrap`, `margin` |
| Kestrel demo, Notes and Stock | long content is cut off | `scroll` |
| Daybook | the writing column is centred by empty texts either side; zen's top margin is an empty text shown `when: "zen"` | `max` in any unit, `self` or `justify: "center"`, `margin` |

## Settle first

Three decisions change the shape of everything after them, so they come before any code. My
recommendation is first in each. **Aram confirmed all three as recommended on 14 October 2026.**

1. **One meaning for a single number: every side.** `pad: 1`, `margin: 1` and a style's `pad`
   all mean every side, as CSS does; `[x, y]` and `[top, right, bottom, left]` say more. The
   25 places in `examples/` that use `"pad": 1` meaning left and right become `[1, 0]` in the
   same commit (a script, checked by rendering each one before and after). The alternative,
   keeping a node's `pad: 1` horizontal, leaves a node's and a style's `pad` disagreeing for
   good.
2. **Lengths are one type everywhere.** The core's `Len` from step 2b (cells, `"8px"`, a `space`
   name) is what `pad`, `margin`, `gap`, `min` and `max` take, so `"min": "240px"` and
   `"gap": "s3"` work. `size` keeps its own type (it alone has shares, `%` and `fit`). A
   `space` name a theme doesn't have is a problem, as in styles.
3. **A scroll offset lives in `State`, in rows for lists and logical pixels for containers.**
   (Revised with Aram on 14 October 2026: a list's offset is lines with fractions, `top: f32`,
   so a trackpad scrolls a list smoothly in the window; a line is whole cells, so the fraction
   is exact to the pixel without the core knowing the font. Lists draw a scrollbar from the
   `scrollbar` parts. 3d's containers keep logical pixels.)
   Lists keep the offset they have (`top`, by row). A scrolling container's offset is pixels,
   the window's unit, now that the terminal is dropped. It's view state by node id like
   selection, so it survives a push, replays, and shows in `get --state`. The readback reports
   each scroller's offset and its content's size, so an agent knows there's more.

## The order

Each sub-step lands with tests in `tests/window.rs` (one per field at least), clean window fuzz
runs (`FUZZ_RUNS=400` in release, `fuzz::laid_out` included), the grid twins it retires deleted,
and the screens that use it looked at in a real window.

### 3a · Units and the box (S) · done 14 October 2026

- Decisions 1 and 2: `pad` four-sided in any unit, `margin` (new) on any node, `min`/`max` in
  any unit, `gap` as one length or `[x, y]`.
- `margin` maps to Taffy's margin. A float's `margin` becomes the same type.
- Rows and columns still clip. A margin gives a card's shadow room. (Kestrel's Floor row has no
  `pad` to drop by now; its cards get their margins with the rest of the demo in 3f.)
- Problems: a malformed length, an unknown `space` name, `margin` on a float's anchor that
  can't take it.

### 3b · Fit and alignment (M) · next

- `size: "fit"`: as big as the content along the parent's direction. Text is measured with the
  window's own text layout (as step 2d's ellipsis does). A `fit` list is as tall as its rows
  up to its `max`, and an editor as tall as its lines.
- `self` (`start`, `center`, `end`, `stretch`), and on containers `justify` (`start`, `center`,
  `end`, `between`, `around`) and `items` (`start`, `center`, `end`, `stretch`, `baseline`).
- `aspect` (`"16/9"`, `1`).
- The risk is `fit` under Taffy's measuring, where GPUI's own ellipsis broke. Test it at many
  widths in `in_window`, and add a fuzz check that a `fit` node is never cut while its parent
  has room.

### 3c · Wrap and grid (M)

- `wrap: true` on a row: children flow onto the next line.
- `{"type": "grid", "cols": 3}` or `"cols": ["200px", "*", "2*"]`, with `span` on children.
  Rows come from the children; no `rows` field until a screen needs one.
- The readback gives each child's rectangle as now, so an agent sees the reflow.

### 3d · Scroll (M, the one new behaviour)

- `scroll: true` on a row or column: it scrolls along its direction, with its offset in `State`
  (decision 3).
- The wheel scrolls it. Page Up and Page Down scroll it when the focus is inside it. **The
  focus scrolled into view**: Tab onto a field below the fold brings it up.
- `Msg::Scroll` is shared with lists; a nested list scrolls first, then its container.
- The readback shows `scroll 120/840` on a scroller. The JSON has `scroll` and `content`.
- A fuzz property: the focused node is always inside its scroller's view after a key.

### 3e · Data placement (S)

- `$index` and `$count` in `each` templates.
- Float `x` and `y` in any unit, as templates (`"y": "{$item.start}"`), with `size` from data
  too, for the calendar.

### 3f · Rebuild without tricks (M)

- The wringer's `settings`, `landing` and `calendar` rebuilt in `screens.py` with no tricks,
  each noted in examples/window/README.md, with screenshots in three themes, as in the styling
  study.
- The Kestrel demo: the Floor's cards `fit` and `wrap`; Notes and Stock scroll; a settings
  dialog laid out without tricks (docs/design/demo.md, step 3's row). The demo plays clean in a
  real window, and its log's commands still work as typed.
- Daybook: the writing column is `max: 72` with `self: "center"`, and zen's top margin is a
  `margin`, not empty texts. Its README drops those two lines from "still open".

## Done when

- Every field above has a test in the window, and the fuzz is clean at 400 runs in release.
- `settings`, `landing` and `calendar` run with no tricks, in three themes, in screenshots.
- The Kestrel demo shows step 3 in a real window, and Daybook loses its spacers.
- `fictty screen --nodes` shows scroll offsets and content sizes, and the guide documents every
  field.
- The release build's frame build time is measured with `examples/stress/play.sh` and set
  against the baseline under "Start here". Step 2 took it from about 0.06 to 0.11 ms; if step 3 adds more,
  style resolution gets its cache (per theme revision and node, as expressive.md's risks say)
  before step 3 is called done.

## Carried with it

Small things found during step 2 and Daybook, picked up where they fit:

- ~~**Font files that fail to load** aren't problems yet (with 3a's problems).~~ Done with 3a:
  the window hands them back with the frame (`Drawn::fonts`), and the state keeps them.
- ~~**`tabs.active` and the `active` state** share a word for two things; rename the part
  `tabs.selected` before any theme outside the repo uses it (with 3a).~~ Done with 3a.
- **A map's catch-all can't read its value** (`{kind|done=✓;*=<accent>{kind}</>}`); a template
  language fix, any time.
- **A task action** (`{"task": "toggle"}`) so ⌘Enter can tick a task: editors, step 5.
- **Days as a calendar**, not a list: Daybook's ⌥[ walks rows. Date arithmetic is a host's or
  an agent's, so this stays out of the UI value.

## After step 3

- **Step 4, rows as nodes (L)**, as expressive.md has it: list, table and tree rows as node
  templates, virtualised, `fit` heights. It's done when the kanban and chat are real cards and
  bubbles, and the Kestrel demo's Jobs board and inbox use them.
- **A decision for Aram: proportional editors, sooner?** Daybook shows the editor's monospace
  text is the gap between a journal that works and one that's beautiful. It's phase 4 today,
  and it needs caretline to take line measurements from the renderer (released there first).
  That work is in caretline, so it can run beside steps 3 and 4 rather than after phase 1.
