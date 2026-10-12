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

- [ ] Layout: `size` in cells and px, `%`, `*`, `min`/`max`, `gap`, `float`, `modal` (with a scrim)
- [ ] Text with markup: styled runs (`<b>`, colours, signs), tabs and fills laid out as columns,
      maps, conditionals; wrapping
- [ ] Lists, trees, tables: virtualised, row templates, multi-line rows, `mark`/`lead`,
      variants, skip rows, filters, heat colours, footers, scrollbars, hover
- [ ] Editors and inputs: caretline at mono cell size; caret, selections, Markdown styles, links,
      task boxes, ordered lists, placeholders, read-only; click, drag, double and triple click,
      wheel; `accept: number`
- [ ] Charts drawn with `canvas`: sparkline, chart, bars, gauge, plot, map, dot, big
- [ ] Pictures native: cards (rounded, gradient, shadow, glow) as styles; SVG and images as
      elements
- [ ] Tabs, buttons, spinners, toasts, the status bar, problems
- [ ] Layers: callouts, arrows, rings, spotlights, pulses, the hover tip
- [ ] Themes: every token and part mapped to styles; parts gain radius, shadow, padding, font
- [ ] Input: every key with every modifier from the OS (`d-` is ⌘), IME, the system clipboard
      (copy as text, ⌘V pastes, owned by the runtime), mouse and wheel, hover, focus rings
- [ ] Tests: the component tests rewritten on readback and state (42 read the character grid
      today); a GPUI test-platform test per component that it lays out without overlap. The
      harness is in: `App::in_window` draws a headless screen with the window under GPUI's test
      platform, and the fuzz driver runs editors, lists, tables and the tour through it
- [ ] The tour's ten chapters and every example run in the window and look right (screenshots
      reviewed)
- [ ] Thought Control runs in the window, unchanged but for `App::run` on the main thread
- [ ] **The terminal goes**: ratatui, crossterm, vt100, `sim`, kitty and OSC 66 code, `setup`,
      `render --ansi`; the fuzz driver loses "the terminal shows the frame"
- Done when: the tour runs in the window end to end, Thought Control's screens run in it, and the
  repo has no terminal code.

Where it starts (what phase 0 left, 12 October 2026):

- **The window is still the spike.** `crates/fictty/src/window.rs` (450 lines) draws col, row,
  stack, text, spinner, button, rule, list and tree (row template only), table, editor and input
  (lines and a caret, no selection or styles), tabs and toasts. Every other kind is a `[kind]`
  placeholder. Colours come from a fixed palette, not the theme. `size` in cells uses fixed
  `COL` (9 px) and `ROW` (24 px), not the font's measured cell.
- **Done (12 October): the window reports its layout.** A measuring canvas in each node notes its
  bounds as GPUI prepaints; once the frame paints they go to the engine as `Drawn`: rectangles
  (the readback has them), hits (`fictty mouse` lands), editor and list sizes, `$width` areas and
  the wake time (spinners turn, toasts go on their own). A column is the monospace font's
  advance (Menlo, 8.43 px at 14 px), a row 24 px; the window sends that as `Msg::Caps` and its
  size in cells as a resize. Editors draw caretline's own frame of them (caret, selection, wrap,
  scroll), and a press, drag or wheel in their text is `Point`, `Drag` and `Scroll`, so double
  and triple clicks work. Still to come for editors: Markdown styles, links, task boxes.
- **Done (12 October): lists and tables scroll from the state.** They draw the rows
  `state::first_row` says, as many as the window reported fit, and the wheel sends `Msg::Scroll`
  (a mouse wheel's notch is a step, a trackpad's travel a step each three rows); the window keeps
  no scroll of its own. Clicks act on press, as the terminal's do. The body and an editor's text
  sit a whole cell in, so they're on the cell grid.
- **Done (12 October): the window under test.** `App::in_window(w, h)` (the `window-test`
  feature: `gpui/test-support`) is a `Headless` whose frame is the window under GPUI's test
  platform: no screen, fixed text metrics (a cell is 8.4 × 24 px), layout, prepaint and the
  handover all real. A `Frame` may route the mouse itself (`Frame::pointer`) and send messages
  (`Frame::messages`), so a press at a cell is a GPUI mouse-down in that cell's middle, through
  the window's own handlers. `tests/window.rs` drives it; `tests/fuzz.rs` fuzzes editors, lists,
  tables and every tour chapter through it (clean at 400 runs each in release). Keys still go in
  as names, not as GPUI keystrokes, so `key_name` is untested.
- **Input bypasses `Input`.** Keys go straight to `Msg::Key` (fine: GPUI routes them), and clicks
  are GPUI `on_click` handlers sending `Msg`s. The terminal's way, `Input::Pointer` resolved
  against `Drawn::hits`, is there for whatever GPUI doesn't route itself (drags across nodes,
  wheel over editors, multi-click timing); decide per input which way it goes, and keep the
  messages the same either way so state, replay and the fuzz driver don't change.
- **`Caps` is a terminal idea.** The window sends its own cell size; the graphics flag goes with
  the terminal.
- **Copy and screens are drained by the frontend.** The engine queues `Effect::Copy` text
  (`take_copied`) and `screen` requests (`take_screens`); the window answers screens with the
  readback outline. A screenshot answer (phase 2) slots in the same place.
- **Grid-reading tests: 75 calls to `screen()`.** 54 in `tests/components.rs`, 16 in
  `tests/host.rs` (the shape Thought Control's tests have), 5 in the core. `Headless` draws
  through a `Frame`; without one, `screen()` already returns the readback outline. Move each
  test to `readback()` or state as its component lands in the window, then drop `Grid` and
  `headless_with` together with the terminal.
- **Waiting: use the channel, not `sleep`.** The first threaded terminal loop lost 20 fps
  because `thread::sleep` overslept by up to 4 ms; waiting on the ring channel with a timeout
  fixed it. The window doesn't pace itself (GPUI draws at most once a display frame), but any
  loop that waits should do the same.
- **Measure the window first.** No release numbers for the window from `main` yet: run
  `examples/stress/play.sh` against `fictty run --window` for a baseline beside the spike's.
- **One flaky test goes with the terminal.** `random_acts_leave_the_real_screen_where_they_leave_a_headless_one`
  (the pty sim) sometimes misses a triple click: four settles of ~80 ms each squeeze the 400 ms
  multi-click window under load. It fails on the old `main` too; it leaves with `sim`.
- **One screen per socket.** A second `fictty run` on the default socket refuses
  (`FICTTY_SOCKET` for another); tabs (phase 2) are the real answer.

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
