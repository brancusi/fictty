# Step 3: the layout vocabulary

The plan for native.md's step 3, written on 13 October 2026 when step 2 merged. The vocabulary
itself was decided in [expressive.md](expressive.md) ("The vocabulary": layout on nodes, on
containers, floats); this says where we start, what to settle first, the order, and how we know
it's done. Everything here is window-only: the grid view isn't extended.

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
recommendation is first in each.

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
   Lists keep the offset they have (`top`, by row). A scrolling container's offset is pixels,
   the window's unit, now that the terminal is dropped. It's view state by node id like
   selection, so it survives a push, replays, and shows in `get --state`. The readback reports
   each scroller's offset and its content's size, so an agent knows there's more.

## The order

Each sub-step lands with tests in `tests/window.rs` (one per field at least), clean window fuzz
runs (`FUZZ_RUNS=400` in release, `fuzz::laid_out` included), the grid twins it retires deleted,
and the screens that use it looked at in a real window.

### 3a · Units and the box (S)

- Decisions 1 and 2: `pad` four-sided in any unit, `margin` (new) on any node, `min`/`max` in
  any unit, `gap` as one length or `[x, y]`.
- `margin` maps to Taffy's margin. A float's `margin` becomes the same type.
- Rows and columns still clip. A margin gives a card's shadow room, so Kestrel's Floor row drops
  its `pad`.
- Problems: a malformed length, an unknown `space` name, `margin` on a float's anchor that
  can't take it.

### 3b · Fit and alignment (M)

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
- The release build's frame build time is measured before and after with
  `examples/stress/play.sh`. Step 2 took it from about 0.06 to 0.11 ms; if step 3 adds more,
  style resolution gets its cache (per theme revision and node, as expressive.md's risks say)
  before step 3 is called done.

## Carried with it

Small things found during step 2 and Daybook, picked up where they fit:

- **Font files that fail to load** aren't problems yet (with 3a's problems).
- **`tabs.active` and the `active` state** share a word for two things; rename the part
  `tabs.selected` before any theme outside the repo uses it (with 3a).
- **A map's catch-all can't read its value** (`{kind|done=✓;*=<accent>{kind}</>}`); a template
  language fix, any time.
- **A task action** (`{"task": "toggle"}`) so ⌘Enter can tick a task: editors, step 5.
- **The pty diff test is timing-bound** (`tests/sim.rs`, `random_acts_…`): the real binary runs
  on the wall clock, so a triple click can be three single clicks to it. The headless twin now
  takes the real screen's clock before each act, which makes CI's three seeds pass; with
  `SIM_RUNS=10`, seed 4 still differs now and then. It goes with the terminal (step 9), so no
  test clock is built for it.
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
