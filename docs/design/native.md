# fictty in its own window

Status: **decided, 11 October 2026.** fictty leaves the terminal. A screen is a native window drawn
by GPUI (Zed's GPU framework), laid out in pixels with real type, and the terminal target goes.
The Elm core, the UI as data, the socket and the host stay exactly as they are. This file is the
implementation plan; [PLAN.md](../PLAN.md) carries its phases as the order of work.

## Why

Aram's ask: a portal that stays open on his Mac, that agents fill with screens he works in (text,
images, video, files), updating as fast as they write, beautiful, with no hoops between the app
and the machine. The terminal is what's in the way:

- it keeps keys for itself (⌘C, ⌘Z in WezTerm) and encodes the rest differently per terminal;
- dropping a file types its path; the clipboard is OSC 52, refused silently by some terminals;
- images work only where the kitty protocol does; video not at all;
- one font, one size, whole cells: no real typography, layout or motion;
- a terminal emulator sits between the grid we test and what's shown.

The spike (branch `spike/native`, 11 October) answered the questions that could have stopped it:

| | GPUI window (spike) | terminal (guide's numbers) |
|---|---|---|
| ticker, 500 values/s | 120 fps | — |
| plasma, a full-screen frame at 120 Hz | 120 fps | — |
| patch storm, 1,000 patches/s | 120 fps, 1,000/s | 116 fps, 1,000/s |
| pushes as fast as a client sends | 120 fps, ~5,900/s | 108 fps, ~6,200/s |
| 50,000-row list replaced 20×/s | 0.02 ms to build a frame | — |

Built on an M-series Mac, release, gpui 0.2.2. The view rebuilds its elements from the state
every frame (an Elm view, literally) in well under a millisecond; GPUI draws at the display's
rate. The engine runs on its own thread, so how fast agents can write doesn't depend on the frame
rate. Keys (⌘ included), clicks, file drops and `fictty screen` all worked through fictty's own
messages.

Iced (the same Elm shape, wgpu) stays the fallback if GPUI's churn ever costs more than it gives;
the core doesn't depend on either.

## What stays, what changes, what goes

| stays | changes | goes |
|---|---|---|
| The UI value: nodes, templates, maps, markup, `when`, `each`, themes as data | `view`: GPUI elements from the state, not cells | ratatui, crossterm |
| `State`, `Msg`, `update`, `Effect`; serializable, replayable | Layout: flexbox in pixels; `size` keeps meaning cells (a row's height, a column's width) and gains `px` | the kitty graphics protocol, OSC 66 (`gfx.rs` sending) |
| The store, writes with revisions, expiring writes, `$log` | Pixels: cards, plots, SVG and images drawn natively, not rasterised and sent | `fictty::sim`, vt100, the escape-code differential |
| Sources (command, stream, file, host), actions, the command policy | `fictty screen`: a readback of the node tree (kind, id, text, rectangle) | `fictty setup wezterm`, the terminal key-forwarding notes |
| The socket and every op; `push`, `patch`, `write`, `read`, `key`, `get` | `Headless`: the engine and the readback, without pixels | terminal probes (cell size, text sizing, keyboard flags) |
| The `Host` trait, `Notifier`, kept entries | `App::run`: opens a window, on the main thread | `render --ansi` |
| caretline as the text engine | Themes: parts gain radius, shadow, padding, font | |
| `fictty::fuzz` on the core | Key names: the same, now from the OS | |

## Architecture

```
agents ── socket ──┐        host (an embedding app) ── Notifier ──┐
                   ▼                                              ▼
             ┌──────────────── engine thread (one per tab) ────────────────┐
             │  events → Msg → update(&mut State) → effects → driver       │
             │  sources, streams, the host, saves, the clock               │
             └──────────────┬──────────────────────────────────────────────┘
                            │ rings when the state changed
                            ▼
             ┌──────────────── main thread (GPUI) ──────────────────────────┐
             │  view(&State) → elements → layout → Metal, once per frame    │
             │  OS input → fictty Msg → engine                              │
             └──────────────────────────────────────────────────────────────┘
```

- **One engine per tab**, on its own thread, holding the `State` behind a lock: it takes in
  messages as fast as they arrive and rings the window. The window locks the state only while it
  builds a frame (a fraction of a millisecond). This is `Live` from the spike, grown up.
- **The view is a pure function of the state**, as now. Its only memory is GPUI's own element
  state (a list's scroll offset), which is view state the terminal also kept outside `State`.
  Anything an agent must see (selection, focus, caret, drag-hover, IME composition) is in `State`.
- **Layout is GPUI's** (taffy flexbox). The `view` returns, besides elements, each node's laid-out
  rectangle, for hit-testing what the OS doesn't route itself and for the readback.

### Crates

Split now, so the core never compiles GPUI and the core's tests stay fast:

| crate | what | depends on |
|---|---|---|
| `fictty-core` | model, template language, state, update, store, lists, host, the engine (`Live`), the socket, `Headless`, `fuzz` | caretline, serde |
| `fictty` | the window: view, input, themes to styles, media, tabs; `App` for embedding; the `fictty` binary and CLI | fictty-core, gpui |

`fictty` re-exports the core, so an embedding app (Thought Control) depends on one crate as today.

### Units and type

- A number `size` stays in cells: rows are the base line height (24 px at 14 px type), columns the
  width of a character of the UI font. Existing UIs lay out as before, just smoother.
- New: `"120px"`, and the existing `"30%"`, `"*"`, `"2*"`. `min`/`max` the same.
- Text uses a proportional UI font (Inter, bundled, OFL) outside editors and a monospace one
  (JetBrains Mono, bundled, OFL) in editors, inputs and `code`. Themes can name others.
- `scale` (headings drawn larger) becomes a font size; `Big` uses real large type.

### Editors

caretline lays text out in cells: its viewport is columns by rows, it wraps by column and moves
up and down by column. Editors therefore stay **monospace** in the window: the caret, selection,
wrap and clicks map exactly as now, at the mono font's cell size. Proportional editors (a
document feel) need caretline to take line measurements from the renderer; that's a caretline
change, made there first and released, and it comes after parity (phase 4).

IME goes through GPUI's input handler on the focused editor: the composing text is a `Msg`
(`Compose { text, range }`) and shown by the view from `State`, so `screen` sees it too.

### Callouts, rings and spotlights

`caretline-layers` places callouts in cells. In the window, layers are placed with GPUI's
`anchored` element against the anchor's laid-out rectangle, and spotlights become a translucent
scrim with soft holes (the `Shade` picture today). Pulses animate on the clock.

### Readback and screenshots

- `fictty screen` returns the shown nodes in order: kind, id, the text it shows, its rectangle,
  focus and selection. `--text` prints it as an outline. This replaces the character grid: an
  agent reads structure instead of positions.
- `fictty screenshot out.png` captures the window. On macOS, capturing one's own window may need
  the Screen Recording permission; if so, GPUI's offscreen rendering (or the test platform) is
  the fallback. Settled in phase 2.

## Phases

Each ends with something Aram runs. Sizes: S one session, M a few, L a week or more of sessions.

### Phase 0 · The seam (M)

The core stands alone, and nothing breaks.

- [x] Workspace: `crates/fictty-core` and `crates/fictty`; `fictty` re-exports the core
- [x] The engine (`Live`) in the core: setup, `pump`, `spawn` (its own thread, ringing whatever
      draws); input through an `Inbox`. `App::run` is "start the engine, then draw" for both the
      terminal (still there) and the window. Same speed as the old loop in a terminal (stress
      scenarios, release, in a pty: 108 fps ticker and storms, ~5,600 pushes/s flat out)
- [x] Readback in the core (`fictty_core::readback`): the shown nodes in order with text, focus,
      selection and rectangle; `fictty screen --nodes` (`--json`), the `readback` op,
      `Headless::readback()`. `Headless::screen()` draws through a `Frame` (the terminal's
      `Grid`) until phase 1 ends; without one it prints the readback
- [x] Laid-out rectangles out of the view (`Drawn::rects`), in the engine's own `Rect` and `Hit`
      types, so hit regions no longer depend on ratatui
- [x] The spike's window behind `fictty run --window` (macOS): the engine's thread rings it;
      keys, clicks, the clipboard, `screen` (as the readback), `upgrade` and quit go through
      fictty's own messages. Drop zones wait for phase 2
- Done when: `cargo test` passes with the core building without ratatui, and the spike runs from
  `main`.

### Phase 1 · Parity in the window (L)

Every component, every theme, every input the terminal had; then the terminal goes.

- [x] Layout: `size` in cells and px, `%`, `*`, `min`/`max`, `gap`, `pad`, `align`, `float`,
      `modal` (with a scrim), `$width`/`$height` areas, `stack` (step 1, below)
- [ ] Text with markup: styled runs (`<b>`, colours, signs), tabs and fills laid out as columns,
      maps, conditionals; wrapping. Done, except a cut line's `…` (GPUI's `text_ellipsis` breaks
      under taffy's measuring, so lines clip) and `keep`
- [ ] Lists, trees, tables: virtualised, row templates, multi-line rows, `mark`/`lead`,
      variants, skip rows, filters, heat colours, footers, scrollbars, hover. Done: drawn from the
      state's scroll (only the rows in view), row templates, multi-line rows, hover, the wheel.
      Left: a tree's indent and folds, `mark`/`lead`, row variants, skip rows, heat colours,
      footers and totals, column widths and alignment, scrollbars
- [ ] Editors and inputs: caretline at mono cell size; caret, selections, Markdown styles, links,
      task boxes, ordered lists, placeholders, read-only; click, drag, double and triple click,
      wheel; `accept: number`. Done: all of it drawn, and click, drag, double and triple press and
      the wheel. Left to prove with tests: a click on a link or a task box, read-only, `accept`
- [ ] Charts drawn with `canvas`: sparkline, chart, bars, gauge, plot, map, dot, big
- [ ] Pictures native: cards (rounded, gradient, shadow, glow) as styles; SVG and images as
      elements
- [ ] Tabs, buttons, spinners, toasts, the status bar, problems. Done: tabs, buttons, spinners,
      toasts (on time). Left: the status bar as the UI's (its status, problems and hints, and
      `status_bar: false`), a `toasts` node's placement
- [ ] Layers: callouts, arrows, rings, spotlights, pulses, the hover tip
- [ ] Themes: every token and part mapped to styles; parts gain radius, shadow, padding, font.
      Done: tokens, parts and `<spec>` runs (`look.rs`). Left: `variant`, `tone`, `edge`,
      `backdrop`, and the new part properties
- [ ] Input: every key with every modifier from the OS (`d-` is ⌘), IME, the system clipboard
      (copy as text, ⌘V pastes, owned by the runtime), mouse and wheel, hover, focus rings.
      Done: keys, the clipboard, the mouse, the wheel, hover. Left: IME (`Msg::Compose`), focus
      rings
- [ ] Tests: the component tests rewritten on readback and state; a GPUI test-platform test per
      component that it lays out without overlap. Done: the harness (`App::in_window`), every
      input through it, the fuzz driver through it, the no-overlap property (`fuzz::laid_out`).
      Left: the grid-reading tests, moved step by step below (32 to go)
- [ ] The tour's ten chapters and every example run in the window and look right (screenshots
      reviewed)
- [ ] Thought Control runs in the window, unchanged but for `App::run` on the main thread
- [ ] **The terminal goes**: ratatui, crossterm, vt100, `sim`, kitty and OSC 66 code, `setup`,
      `render --ansi`; the fuzz driver loses "the terminal shows the frame"
- Done when: the tour runs in the window end to end, Thought Control's screens run in it, and the
  repo has no terminal code.

#### Done so far (12 October 2026)

Commits `dc868b9` to `6c6b07b`.

- **The window reports what it laid out.** A measuring canvas in each node notes its bounds as
  GPUI prepaints; once the frame paints, they go to the engine as `Drawn`: rectangles (the
  readback has them), hits (`fictty mouse` lands), editor and list sizes, `$width` areas and the
  wake time (spinners turn and toasts go on their own). A column is the monospace font's advance
  (Menlo, 8.43 px at 14 px; `ch_width` is the glyph's ink, not its advance), a row 24 px; the
  window sends that as `Msg::Caps` and its size in cells as `Msg::Screen`.
- **Editors draw caretline's own frame.** Caret, selections, wrap and scroll are caretline's;
  each row is one shaped run with selections behind it on the cell grid (splitting a row into
  spans moved glyphs as the selection changed). A press, drag or wheel in the text is `Point`,
  `Drag` and `Scroll`, the terminal's messages. Markdown roles come from the core
  (`model::md_roles`) and are inked from the theme's `md.*` parts.
- **Themes and markup.** `look.rs` reads the active theme each frame: tokens are colours, parts
  and `<spec>` runs are inks. `text.rs` draws a template's styled runs, signs, tabs and fills.
- **Lists and tables scroll from the state.** They draw the rows `state::first_row` says, as
  many as fit; the wheel sends `Msg::Scroll` (a notch is a step). The window keeps no scroll,
  hover or selection of its own. Clicks act on press, as the terminal's do.
- **The window under test.** `App::in_window(w, h)` (the `window-test` feature,
  `gpui/test-support`) is a `Headless` drawn by the window under GPUI's test platform: no screen,
  fixed text metrics (a cell is 8.4 × 24 px), real layout, prepaint and handover. `Frame::input`
  takes keys as the keystrokes macOS delivers (`window::test::keystroke`, checked against
  `key_name` for every key the fuzz driver presses), the mouse as GPUI events in a cell's middle,
  and pastes through the clipboard and ⌘V; `Frame::copied` puts copies on the clipboard.
  `Headless::at(id, row, tab)` and `text_at(id)` find what to press.
- **What it proved.** `tests/window.rs` (12 tests) covers layout, rows, tabs, carets, drags,
  multi-presses, the wheel, resizes, keys, paste and copy, hover and the finders. The fuzz driver
  runs editors, lists, tables and all ten tour chapters through the window, clean at 400 runs in
  release; a test checks its mouse changes state as often in the window as in the grid (14 of
  77 acts, both).
- **In the app.** ⌘V pastes the system clipboard (caretline knows its own copies when they come
  back); rows hover; the body and editors' text sit a whole cell in, on the cell grid.
- **CI.** `.github/workflows/test.yml`: clippy (warnings denied) and the tests on macOS and
  Linux on every push, as a signal.
- **Speed, engine side.** With the window off screen (so drawing nothing), the engine took ~6,060
  pushes a second flat out, as the spike did (~5,900). Frame times need the window on screen.

#### Next, step by step

Step 1 landed on 12 October (branch `window-layout`): sizes are read once in the core
(`Size::measure`: cells, `"48px"`, `%`, shares) for both renderers; a fixed size is a flex basis
that shrinks when there's no room, as a terminal's layout gives one up; `min`/`max` bound it in
cells; `pad` insets a node by cells and rows (inside a panel's own inset); `align` aligns text
and places a button's face (now as wide as its label, its hit the face); a `float` sits in a box
of its size at its edge or corner, `margin` in, covering what's under it; a `modal` puts a
scrim (the background, part see-through, or the theme's `scrim`) over its stack, which takes
presses and the wheel and sends the engine's `mouse` nowhere. The mouse's moves and releases
became window-level listeners, since a float occludes the root; hover is the innermost hit
under the mouse, so a row under a dialog isn't hovered. What a frame notes is clipped to what
holds it: the tour at 21 columns squeezed a panel out of its parent, and its rectangle and hit
were where it didn't show. `fuzz::laid_out` (nodes nest in their parents and don't overlap but
over a stack) runs with the window's fuzz tests, clean at 400 runs in release over the tour, a
dialog with floats, editors, lists and tables. The test window is a quarter cell over its size
(a half cell put far edges on a rounding tie). Ten grid tests moved to `tests/window.rs`, plus
floats, `px`, `pad` and `align`. Not yet done: looking at the tour's chapters in a real window.

**The order changed on 13 October** ([expressive.md](expressive.md), decided with Aram): the
terminal is dropped (no more terminal work; new fields are window-only), themes become whole
styles with the theme kit pulled forward from phase 4, a layout vocabulary on GPUI's Taffy is a
step of its own, and rows become nodes. [examples/window/](../../examples/window/README.md) (the
wringer: nine screens, six themes, `open.sh`) is what each step is measured against, with the
public demo ([demo.md](demo.md), `examples/demo/`): each step lands with the demo showing it. **The next
step is 2, and it starts with 2a.**

Step 2a landed on 13 October: `crates/fictty-core/src/style.rs` resolves each shown node's
style: its part (its `type`; containers have none, which also keeps a layout `row` apart from a
list's `row` part), with the parts of the states that hold layered over it (every combination,
fewer before more, so `row.selected.blur` is `row`, `row.selected`, `row.blur`, itself), then its
variant, its `tone` and its own `style`, with tokens filled in as `#rrggbb`. Its border, title,
edge (only where it draws a border or a face) and backdrop resolve with it. The defaults the
window and the grid used as fallbacks (`button`, `title`, `border` and their `.focused`) are
default parts under every theme. `hover`, `focused` and `blur` hold for a node today; `active`,
`selected` and `disabled` are named but wait for the state that says so (a press, a node's
`disabled`). The window's `look.rs` reads colours and specs through the core's `Theme`, so there
is one resolver; it draws from `Resolved` in 2c. The readback carries it (`screen --nodes
--style`, `render --nodes --style`, `style` in the JSON). Theme problems: parts no component has
(a name outside the components' families is the theme's own, for markup), spec words that mean
nothing, colours that aren't tokens, `skip`s that aren't row variants; scale entries and roles
join in 2b and 2d. Every example screen in every theme comes back clean. Open for 2b: the
window tells the engine only a row's hover, so `.hover` holds for a button only once 2c sends it;
and `tabs.active` (the shown tab) reads as `tabs` in the `active` state, which 2b should settle
(a `tab` part with `.selected`).

Step 2b landed on 13 October: a part is a short ink or a whole style (`model::Part`), and a
node's and a variant's `style` is the same whole style (`model::Style`: `fg`, `bg` as a colour or
a gradient, the ink modifiers, `type`, `pad`, `radius`, `border`, `shadow`, `ring`, `shape`,
`opacity`, `cursor`). Each layer lays over the last (`Style::over`). The theme has scales
(`space`, `radius`, `depth`, `type`), `fonts`, and `motion` (read and kept for phase 3). The
resolved style fills every name in: pad as cells or pixels, radius in pixels, shadows and ring
with colours, the type role with its font. A part's border draws even where the node has none
of its own (a button's). Problems name unknown scale entries, roles, shapes, cursors, border
styles, opacities out of range and fields no style has (kept, not dropped, so a typo is never
silent); a malformed part, length or background fails to load with what was expected. Variants
are whole styles through their `style` (their `edge`, `border`, `backdrop` and `title` stay as
they were). A style's single `pad` length pads every side, where a node's `pad: 1` pads only the
sides: the layout step settles node pads with four-sided `pad`. Fonts are names in the core;
the window loads font files in 2d. Not done yet: the window draws only colours from all this
(2c).

Step 2c landed on 13 October: the window draws every node from its resolved style through one
mapping, `Look::boxed` (look.rs): background or gradient, ink, pad, corners, border (dashed
too), shadows, a ring (hard shadows, with the page's colour between for its offset), opacity and
cursor. A button's style goes on its face. The inner parts resolve the same way
(`style::inner`): a list's or table's row with `selected`, `hover`, `focused` and `blur` as they
hold for that row (`style::row_states`), the tabs, the table header, empty lists, placeholders,
the caret and selection, Markdown's roles, toasts. What the window used to hard-code (a button's
pad and radius, a panel's inset and background, tab pills, row hover and selection, the input's
box, the Markdown inks) is now default parts in the core, so the readback shows what's drawn,
and a theme's part of the same name owns its ink and keeps the default's box unless it says
otherwise. The theme's `edge` folded in: anything but `rounded` squares corners (the radius
goes), `thick` and `double` draw two pixels. The window now tells the engine what any node the
mouse is over is, not only rows, so `.hover` parts hold for buttons and the rest, and GPUI's own
hover styling is gone. Tabs take the shape their part gives them: `pill` (the default),
`underline` (the `tabs.underline` part's colour under the shown tab), `segment` (joined in one
box) or `plain`. Tests in the window: a button's hover part, a part's pad widening a button's
face and its press, every tab shape taking clicks. The six wringer themes look as they did, with
square edges where they ask. Not yet: type roles and fonts (2d).

Its cost, measured in a release build with the window in front (`examples/stress/play.sh`,
5 s a stage, against main before 2a): frames build in about twice the time (ticker 0.06 → 0.11
ms, patch storm 0.07 → 0.13 ms, a 60/s push storm 0.12 → 0.20 ms), the frame rate is unchanged
(120 fps; 60 under the 60/s storm), and pushes flat out are 5,600–6,000 a second before and
5,000–5,800 after over repeated runs. Resolution isn't cached yet; when it matters, cache it
per theme revision and node, as expressive.md says.

Step 2d landed on 13 October. A node asks for a type role through its style (`"style":
{"type": "display"}`): a node's own `type` is its kind, so expressive.md's `"type": "display"`
on a node couldn't be. Markup names a role like a part (`<display>…</>`), and a line takes the
first role it names, since GPUI shapes a line at one size. A role sets the font (a family, or
a `fonts` file loaded with GPUI's `add_fonts` the first time a theme names it, its family found
by what the text system knows after that it didn't before; font paths are made absolute beside
the themes file), the size, the weight, the line height and tabular numbers (`tnum`). Tracking
resolves and reads back but isn't drawn: GPUI 0.2.2 has no letter spacing. A line that stays
on one (a row, a table cell, a title, `wrap: false` text) is our own element, `text::line`: a
canvas that shapes its runs once it knows its width, arranges the left side, a fill and the
right side (`text::arrange`), and cuts what doesn't fit with `…` (`text::fit`): the right side
whole while the left keeps `keep`, then the right cut too. `lines: n` wraps and shows at most
`n` lines, the last cut with `…`: also ours, by shaping prefixes, because GPUI's clamp has the
trouble its ellipsis had (taffy keeps the smallest size it measured, and it showed one word). A
frame notes the nodes it cut (`Drawn::cut`), and the readback says `(cut)`, so an agent knows
it isn't reading what's all on screen; a tab in read-back text stays a tab. A tab bar draws its
`lead` and `trail` in the window now, and reads them back (`thc · Inbox 4 | Done · Sat 11`).
Moved: the cut, keep, tab-title and tab-focus grid tests to the window, the template and
markup-part tests to the core (`text::fit` and `text::arrange` are unit-tested on a monospace
measure); their grid twins and view.rs's variant-and-pad test are deleted.

Step 2e landed on 13 October. A `backdrop` (a node's or its variant's) resolves into the card
it stands for: its fill, or a gradient down to `fill2`, its border colour, its corners (a
fraction of the cell's height, from the cell size the window reports) and its shadow (a glow in
its `glow` colour, else a dark shadow down). A `tone` draws through the resolved ink. The status
line is the UI's: the status, else the first problem (the `status.error` part), else what keys
do, in the `status` part; `"status_bar": false` hides it, and `"status_bar": "speed"` adds the
speed meter, which the window showed always before. A row or column still clips what it holds
(the window fuzz found children of a squeezed row spilling over each other without it), so a
card's shadow stops where its row does: a row of cards takes `pad` for room. The demo gained its
beat: the agent patches a card with a misspelled variant, reads the problem back from `get
--state` (it shows on the status bar too), and fixes it.

Each step builds a slice, moves the grid tests that slice owns onto `App::in_window`
(readback, state and the finders, not characters), and ends with the tour's chapters that use it
looked at in a real window. A step is done when its tests pass in the window, its grid twins are
deleted, and the window fuzz runs stay clean.

1. **Layout (S–M), done.** `pad`, `align`, `min`/`max`, `px` sizes, `float`, `modal` with a scrim (a
   press outside goes nowhere). A no-overlap property in the fuzz driver for the window: siblings'
   rectangles never overlap and every child sits inside its parent (stacks and floats excepted).
   Tests: `when_hides_a_node_and_gives_its_room_away`,
   `a_modal_floats_over_the_screen_and_keeps_the_focus`, `gaps_space_out_a_row`,
   `min_and_max_bound_a_share`, `when_can_read_the_width_a_node_is_laid_out_in`,
   `each_repeats_a_template_per_item`, `each_follows_its_data_and_keeps_its_tree_otherwise`,
   `the_readback_has_each_nodes_rectangle_from_the_grid`,
   `headless_takes_the_mouse_and_resizes_like_a_terminal`,
   `keys_type_into_a_field_a_key_just_showed`.
2. **Text and themes as whole styles (M–L). Next: 2f.** Native.md's text-and-themes work
   plus expressive.md's "Looks", "Type" and the theme kit, in this order:
   - **2a. Style resolution in the core.** A pure function in `fictty-core` (a new `style.rs`):
     for each shown node, its theme part plus the state suffixes that hold (`.hover`, `.focused`,
     `.active`, `.selected`, `.disabled`, `.blur`, read from `State`, which already has hover),
     then its variant, then its own `style`, with tokens and scale names filled in. The readback
     carries it (`fictty screen --nodes --style`, and the JSON); unknown parts, tokens, scale
     entries, roles, variants and `row_variant`s become `problems`. Tested in the core, no
     window.
   - **2b. Parts as whole styles.** A part stays a short ink (`"accent+b"`) or becomes an object:
     `fg`, `bg` (a colour or `{"from", "to", "angle"}`), `type`, `pad`, `radius`,
     `border` (`width`, `color`, `style`), `shadow`, `ring`, `shape`, `opacity`, `cursor`. The
     theme gains scales (`space`, `radius`, `depth`, `type`; `motion` reserved for phase 3) and
     `fonts` (names, or font files loaded with GPUI's `add_fonts`). Variants become whole styles
     too (they style, never arrange).
   - **2c. The window draws the resolved style.** `look.rs` becomes the mapping from it onto GPUI
     styles, for panels, buttons, inputs, tabs, rows, toasts. The `edge` token (13 October,
     window only: square corners unless `rounded`, `thick`/`double` heavier) folds into it.
   - **2d. Type.** Type roles (`"type": "display"` on a node, `<display>…</>` in markup, `scale`
     as a shorthand); `wrap` and `lines` on text; our own ellipsis (measure, then cut with `…`,
     keeping the right side after a tab; GPUI's `text_ellipsis` breaks under taffy's measuring)
     and `keep`; tabular numbers where a part asks.
   - **2e. The rest of native step 2.** `tone`, `backdrop` (cards: radius, gradient, shadow); the
     status bar as the UI's (status, problems, hints, `status_bar: false`), the speed meter only
     when asked.
   - **2f. The styling study.** Rewrite the wringer's six themes as whole styles and push them
     (examples/window/README.md, "Pushing the styling"): which subtle things change a screen's
     character most. Findings and before/after screenshots go in a short write-up beside
     expressive.md.
   Tests: `a_cut_line_ends_in_an_ellipsis_and_keeps_its_right_side`,
   `a_cut_meta_ends_in_an_ellipsis_too`, `keep_holds_room_for_the_text_before_the_meta`,
   `templates_fit_fields_escape_brackets_and_take_tabs_from_values`,
   `a_markup_tag_can_name_a_theme_part`, `tab_titles_are_templates_with_a_lead_and_a_trail`,
   `tabs_start_where_they_say_and_choosing_one_moves_the_focus_into_it`, and view.rs's
   `a_variant_and_pad_come_from_the_theme_and_the_node`.
   Done when the six themes are whole styles and three of them make `mail`, `settings` and
   `landing` look like three different apps, in screenshots.
3. **The layout vocabulary (M).** New (expressive.md, "Layout"): `size: "fit"`, `margin`,
   four-sided `pad` and `min`/`max` in any unit, `self`, `aspect`; on containers `justify`,
   `items`, `wrap`, `gap` as `[x, y]`; `grid` with `cols` and `span`; float `x`/`y` from data;
   `$index` and `$count` in `each`; `scroll: true` with its offset in `State` (the wheel, page
   keys, focus scrolled into view, the offset in the readback). Each maps onto GPUI's Taffy
   style; a test per field in `tests/window.rs`. Done when `settings`, `landing` and `calendar`
   are rebuilt in `screens.py` without their tricks.
4. **Lists, trees, tables, and rows as nodes (L).** Row templates as nodes (expressive.md,
   "Content"): a list, table or tree's `row` is a node template filled from `$item`, virtualised,
   its height the template's `size` or `fit`; clicks on a row's own buttons reach them. Done when
   `chat` and `kanban` are rebuilt with real bubbles and cards. Then the rest: a tree's indent,
   fold glyphs and clicks on them; `mark`/`lead`; row variants; skip rows; filters; heat colours; column widths and alignment; footers and
   totals; scrollbars. Tests: `a_tree_shows_nested_rows_and_folds_them`,
   `a_tree_row_is_addressable_in_the_store`, `folds_survive_an_update_from_outside`,
   `a_filter_narrows_a_list_and_store_actions_still_find_the_row`,
   `a_filtered_tree_opens_to_show_what_matches`,
   `a_long_list_shows_a_scroll_bar_and_keeps_its_rows_clear_of_it`, `rows_can_take_two_lines`,
   `a_tree_takes_a_mark_and_a_leaf_glyph_like_a_list`,
   `the_wheel_scrolls_a_list_and_leaves_the_selection`, and view.rs's
   `a_table_totals_its_columns`.
5. **Editors, the rest (S).** Prove in the window what's drawn: links (style and click), task
   boxes (click toggles), read-only, `accept: number`; then IME as `Msg::Compose` in `State`,
   shown by the view. Tests: `a_readonly_editor_scrolls_but_never_changes`,
   `a_key_at_the_end_of_a_long_paragraph_stays_quick`, and the cell-addressed ones that only
   worked through the grid's sizes: `an_editor_ticks_markdown_tasks`,
   `links_in_an_editor_act_on_a_click_and_a_key`, `a_wiki_links_brackets_only_place_the_caret`,
   editing.rs's `the_mouse_places_extends_and_selects_words_and_lines`.
6. **Charts, big type and icons (M).** `icon` (a bundled open set, Lucide, ISC) joins.
   `sparkline`, `chart`, `bars`, `gauge`, `plot` (and its map and dots), `big`, drawn with `canvas` from the same data; hits on bars and points. Tests: view.rs's
   `big_digits_are_three_rows` and `bars_across_label_bar_and_value`, rewritten on readback,
   plus a no-panic fuzz over the speed and gallery chapters.
7. **Toasts, layers and the clock (M).** `enter`/`leave` motion for floats, layers and toasts
   joins. A `toasts` node's placement and covering; layers placed with `anchored`: callouts,
   arrows, rings, spotlights as a scrim with soft holes, pulses on the logical clock; the hover tip. Tests: `toasts_go_when_their_time_is_up`,
   `a_spinner_turns_with_the_clock`, `a_layer_shows_while_its_when_holds`,
   `a_modal_layer_covers_what_is_under_it`, `a_lasting_write_goes_on_the_screens_clock`,
   `toasts_go_where_a_toasts_node_is`, `a_toasts_node_covers_what_is_under_it_while_one_shows`.
8. **Embedding and the examples (M).** The wringer's nine screens in screenshots beside the
   tour. host.rs's seven grid tests on `in_window` (the shape Thought Control's tests have);
   every example and tour chapter looked at in a real window, with screenshots; `examples/stress/play.sh` against a window on screen for frame times beside
   the spike's; Thought Control's screens in the window, with Aram.
9. **The terminal goes (S–M).** ratatui, crossterm, vt100, `sim`, `Grid`, `gfx.rs` sending,
   `setup`, `render --ansi`; `App::headless` becomes the window under test (or frameless
   readback where GPUI can't build); the fuzz driver drops "the terminal shows the frame".

Notes that still hold:

- **Each frontend gets the same messages.** GPUI routes clicks on rows, tabs and buttons itself;
  editors map pixels to cells in the window. Either way the messages are the terminal's, so
  state, replay and the fuzz driver don't change.
- **Copy and screens are drained by the frontend.** The engine queues `Effect::Copy` text
  (`take_copied`) and `screen` requests (`take_screens`); the window answers screens with the
  readback outline. A screenshot answer (phase 2) slots in the same place.
- **Waiting: use the channel, not `sleep`.** The first threaded terminal loop lost 20 fps
  because `thread::sleep` overslept by up to 4 ms; waiting on the ring channel with a timeout
  fixed it.
- **Windows off screen don't draw.** macOS stops a hidden or covered window's frames: screen
  captures of it are stale and its fps reads 0. Look at it in front, or test it in `in_window`.
- **The debug suite is slower.** The tour fuzzed in the window takes most of `tests/fuzz.rs`'s
  ~66 s in a debug build; trim its runs there if it gets in the way.
- **One flaky test goes with the terminal.** `random_acts_leave_the_real_screen_where_they_leave_a_headless_one`
  (the pty sim) sometimes misses a triple click under load; it leaves with `sim`.
- **One screen per socket.** A second `fictty run` on the default socket refuses
  (`FICTTY_SOCKET` for another); tabs (phase 2) are the real answer.
- **To look into:** `fictty set notes` with `{"selected": k}` answers ok but the notes list's
  selection doesn't move (it follows its `key`); find out whether that's meant.

### Phase 2 · The agent's portal (M)

What the window makes possible for agents.

- [ ] `watch`: an agent waits for events (a store write by the person, a submit, a click on a named
      action, a drop) with filters and revisions; `emit` actions name events
- [ ] Drop zones: `accept` and `on_drop` on any node; hover state in `State` (`$drop`), files as
      `{$file.path}`, `{$file.name}`, `{$file.type}`, `{$file.size}`; a theme part `drop.hover`
- [ ] Tabs: one app, many screens. `fictty open --tab <name> ui.json` opens a tab (starting the
      app if it isn't running); each tab its own engine, store and socket
      (`$FICTTY_DIR/<tab>.sock`); a tab bar; ⌘1–9, ⌘W, ⌘T
- [ ] `screenshot` (readback with rectangles landed in phase 0; the window fills them in phase 1)
- [ ] The command policy for UIs an agent shares: data only by default, commands asked for
- Done when: the receipts walkthrough runs: an agent opens a tab with a drop zone, Aram drops
  files, the agent hears it through `watch`, files them and the table updates.

### Phase 3 · Media (M)

- [ ] `image`: a path or URL, `fit` (contain, cover), rounded corners; decoded off the main thread
- [ ] `video`: a path or URL in a node, hardware-decoded (AVFoundation) frames drawn as a
      `surface`; play, pause, seek and position in `State`, so agents control and read it
- [ ] Pasting and dropping images: the clipboard's image types arrive as files
- [ ] Animation: transitions on focus, selection and layers, on the logical clock (replayable)
- Done when: an agent puts a video in one tab and an image gallery in another, and both play and
  scroll at the display's rate.

### Phase 4 · Typography (M–L)

- [ ] Proportional text where it reads better: headings, prose, tables' text columns
- [ ] caretline measures lines through the renderer (a caretline release), so editors can be
      proportional; Markdown documents get real heading sizes
- [ ] The theme kit: type scale, spacing scale, radii, elevation

### Phase 5 · Ship (M)

- [ ] `fictty.app`: a bundle with an icon, menus (from the UI's keys), a Dock presence; signed and
      notarised in the release workflow
- [ ] `try.sh` downloads the app and opens the tour in a window; frozen demos keep working for
      the posts that pinned them (they pin terminal builds and stay as they are)
- [ ] README, guide, architecture, site: fictty as a window, not a terminal; screenshots
- [ ] A post: why fictty left the terminal, with the spike's numbers
- [ ] Release v0.3.0
- Later: Linux (GPUI runs on Wayland and X11), Windows.

## Thought Control

Thought Control embeds fictty. What its move costs:

- **Unchanged:** its UIs (JSON), its `Host` (sources, actions, kept entries, `Notifier`), its
  store use, its Headless tests that assert on state, store text and the focused editor.
- **Changes:** `App::run` opens a window and must be called on the main thread (it already is, in
  `thc screen`). Tests that read the character grid move to `readback()` (phase 1). Sizes in its
  UIs keep working; it can use `px` and the new theme parts when it wants.
- **Goes:** its terminal key forwarding (`thc_keys.lua`, `fictty.lua`), its pty harness, terminal
  image tuning. Already paused (11 October).
- It pins a fictty commit per phase; each phase's end is a build it can take.

## Risks

| risk | what we do |
|---|---|
| GPUI's API moves and its crates.io release lags Zed | Pin `gpui = "=0.2.2"`; only `crates/fictty` touches it; move versions deliberately, one commit each. Iced is the fallback, the core untouched. |
| Shaders need Xcode's `metal` compiler | `runtime_shaders` for development; the release workflow builds with Xcode and precompiled shaders. |
| caretline lays text out in cells | Editors stay monospace until caretline measures through the renderer (phase 4, a caretline release first). |
| `caretline-layers` places callouts in cells | Placement moves to GPUI's `anchored`; the layer data stays the same. |
| Readback loses the character grid's exactness | Readback is structural (nodes, text, rectangles); screenshots for looks; GPUI's test platform for layout. |
| Screenshots may need Screen Recording permission | Offscreen rendering or the test platform as the fallback; decided in phase 2. |
| 70 tests read the grid | Rewritten on readback and state in phase 1, component by component, before the grid goes. |
| Frozen terminal demos and posts | They pin terminal builds and keep working; new posts pin window builds. |

## Decided with Aram (11 October 2026)

1. **The name stays fictty.**
2. **macOS first.** Linux and Windows come after v0.3.0.
3. **One app with tabs.** An agent opens a tab (`fictty open --tab`); a separate window only on
   request (⌘N, or `fictty open --window`).
