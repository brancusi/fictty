# Architecture

How fictty is layered, what belongs where, and the rules that keep it that way, as it stands on
17 October 2026. fictty is moving from the terminal into a native window: the engine is done
and shared, the window is growing to parity step by step, and the terminal's grid is kept only
until the window has everything ([native.md](native.md)). What's coming is in
[the plan](../PLAN.md); every component, and the ones to build next, are in
[the components](../components.md); what has changed so far is in [the changelog](../CHANGELOG.md).

## The direction

On 13 October 2026 the terminal was dropped as a target ([expressive.md](expressive.md)). It held
the look back: one font at one size, whole cells, no real images and no motion. The window is
drawn by GPUI (the framework behind the Zed editor), laid out by its Taffy flexbox, and painted
on Metal. Nothing about the model changed: a screen is still one value, an agent still pushes,
patches and reads it back, and the core is still an Elm loop. What changed is what draws it, and
that the layout and style vocabulary can now be as rich as a designer's.

| decided | what |
|---|---|
| 11 October | leave the terminal for a native window; split the engine out first ([native.md](native.md)) |
| 13 October | the terminal is dropped (no new terminal work); themes become whole styles; a layout vocabulary on Taffy is its own step; rows become nodes ([expressive.md](expressive.md)) |
| 13 October | the public demo is one app an agent builds live in the window ([demo.md](demo.md)) |
| 17 October | demos show only the building blocks, no workarounds ([building-blocks.md](building-blocks.md)) |

## Where fictty sits

| layer | owns | depends on |
|---|---|---|
| **caretline** | text: the editing engine, callouts (`caretline-layers`) | nothing that draws |
| **fictty-core** | a UI as one value, the template language, the Elm loop, the store, data sources, the socket, the host, style resolution, readback, the fuzz driver: the engine, with nothing that draws | caretline, serde |
| **fictty** | drawing the state: the native window (GPUI, macOS) and, until it goes, the terminal's grid (ratatui); pixels; the CLI; `App` for embedding. Re-exports the core | fictty-core, gpui, ratatui |
| **apps** | Thought Control, Daybook, anything else: their data, actions and screens | fictty |

fictty knows nothing about any app. An app's screens are fictty UIs (JSON, or Rust that builds
the JSON), and its world reaches them as data sources and actions through a `Host`.

## Inside a running screen

```
agents ── socket ──┐                     host (an embedding app) ── Notifier ──┐
people ── keys, mouse, paste (the OS) ──┐                                      │
                   ▼                    ▼                                      ▼
        ┌──────────────────── engine thread (fictty-core: Live) ─────────────────────┐
        │  every input is a Msg → update(&mut State, Msg) -> Vec<Effect>      pure   │
        │  effects run outside update: fetch, run, load, save, toast timers, quit    │
        │  sources and streams, the host, the store file, the logical clock          │
        └──────────────────────────┬─────────────────────────────────────────────────┘
                                   │ rings when the state changed
                                   ▼
        ┌──────────────────── main thread (fictty: the window, GPUI) ────────────────┐
        │  lock the state only while building a frame                                │
        │  shown tree → resolved styles → GPUI elements → Taffy layout → Metal       │
        │  hands back Drawn: hits, rectangles, sizes, scrolls, row heights, cuts     │
        └──────────────────────────┬─────────────────────────────────────────────────┘
                                   ▼
            readback: fictty screen --nodes [--style]     fictty get --state
```

- **The engine runs on its own thread** (`Live::spawn`): it takes in sources, socket requests
  and input as fast as they arrive and rings whatever draws when the state changed. The window
  posts its input to the engine's `Inbox`, so a key and an agent's `put` go through the same
  `update`.
- **State is serializable**: the UI value, the store, view state keyed by node id (`Local`:
  selection, scroll, folds, measured row heights), focus, stats and the time (`now_ms`). If it
  isn't in the state, it can't be read back, replayed or tested.
- **What a frame laid out goes back as a message.** The window measures; the engine decides.
  `Drawn` carries each node's rectangle, the mouse's hit regions (a row, a row's own button, a
  tree's fold glyph), each editor's and list's size, each scroller's extent, the height of every
  `fit` row it built and the nodes whose text it cut. The engine folds these into `State`, so the
  next `update` (a page down, the focus scrolled into view) works from real sizes.
- **One engine, whatever draws.** `Headless` drives the same engine inline for tests; given the
  window under GPUI's test platform as its frame (`App::in_window`), it lays out exactly as the
  real window.

## From a value to pixels

Each stage is pure and lives in the core, except the last two.

1. **The shown tree.** `when` hides and shows nodes; `each` expands its template once per item,
   filled from `$item`, `$index` and `$count` (`State::tree`, worked out again only when what it
   reads changed). Templates are substitution only: `{path}`, filters (`{x|a=…;*=…}`, `?`, `!`),
   `{±x}`, `{x:12}`, `{$store.key}`, `{$rows}`, and `<style>…</>` markup.
2. **Rows.** A list, tree or table shows its bound array, filtered, with a tree's folds. Its
   `row` (and a table column's `cell`) is a node template filled per item, built only for the
   lines in view. `Lines` is the one piece of arithmetic for a list's view: lines of rows (`cols`
   to a line), each line's height (fixed, or `fit` as measured), and what's in view. Following
   the selection, the wheel, paging and the readback's `(scroll a/b)` all ask it.
3. **Style resolution** (`style.rs`). A node's style is its part (its `type`) with the parts of
   the states that hold for it (`hover`, `focused`, `selected`, `on`, `blur`, every combination,
   fewer before more), then its variant (with the variant's own `parts` for what's inside it),
   then its `tone`, then its own `style`. Tokens become colours, scale names become lengths. No
   cascade and no selectors: nothing else applies. Wrong names are problems, never silence.
4. **Layout** (the window). Sizes are read once in the core (`Size::measure`: cells, `px`, `%`,
   shares, `fit`) and mapped onto Taffy: `min`/`max`, four-sided `pad` and `margin`, `self`,
   `aspect`, `justify`, `items`, `wrap`, `gap`, `grid` with `cols` and `span`, floats placed from
   data, `scroll`. A number is cells: rows are the line height, columns a character of the UI font.
5. **Paint** (the window). `Look::boxed` maps a resolved style onto GPUI: background or
   gradient, ink, pad, corners, border, shadows, ring, opacity, cursor. Text with markup becomes
   styled runs (`text.rs`), cut with our own ellipsis. Editors draw caretline's frame.

## The two crates

| module | what it is |
|---|---|
| `fictty-core/src/model.rs` | the UI value (nodes, kinds, sources, actions, theme, layers) and the template language |
| `fictty-core/src/state.rs` | `State`, `Msg`, `Effect`, `update`, keys, focus, selection, `Lines` |
| `fictty-core/src/engine.rs` | the socket and its requests, sources, the host, saving, `upgrade`; `App`, `Live`, `Headless`, `Drawn` |
| `fictty-core/src/style.rs` | each shown node's resolved style under the theme, and what's wrong with the theme |
| `fictty-core/src/readback.rs` | the screen as an agent reads it, node by node |
| `fictty-core/src/store.rs`, `lists.rs`, `host.rs`, `fuzz.rs` | the store, Markdown list editing, what an embedding app provides, the fuzz driver |
| `fictty/src/window/` | the native window: `mod.rs` (view, input, `Drawn`), `look.rs` (styles to GPUI), `text.rs` (markup to runs), `test.rs` (GPUI's test platform) |
| `fictty/src/view.rs`, `runtime.rs`, `gfx.rs`, `sim.rs` | the terminal: grid view, draw loop, kitty graphics, a pty emulator. These go in step 9 |
| `fictty/src/main.rs` | the CLI: one subcommand per socket request |

## The vocabulary agents build with

```
 assembly            the UI value: what goes where, bound to what data
 theme               the look: tokens, scales, fonts, parts as whole styles with states, variants
 project components  compositions as data, built once per project (planned)
 alphabet            primitives with behaviour, maintained here
 escape hatch        a custom widget in Rust, compiled into the app (planned)
```

- **Alphabet.** A small set of primitives that need behaviour or speed: layout containers, text,
  list, tree and table, input and editor (on caretline), tabs, buttons, images, layers, and keys,
  clicks, hover and focus on every node ([the components](../components.md)).
- **Theme.** Data in the state, so it can be pushed, patched and rewound. It owns the look only;
  layout stays with assembly. The six themes in `examples/window/themes.json` change type, edges, depth
  and colour on the same screens.
- **Project components.** JSON compositions with named parameters (`{"use": "kpi", …}`),
  substitution only. Planned (the plan's section 4).
- **Assembly.** What an agent writes most of the time.
- **Escape hatch.** A widget registered by an embedding app. Planned, after the window.

Patterns move down: a recipe that recurs becomes a project component; a component that needs
new behaviour is proposed for the alphabet.

## How it's tested

- **`update` and the readback are pure**, so most tests build a `State`, apply messages and look.
- **The window under test**: `App::new(ui).in_window(w, h)` is the real window on GPUI's test
  platform, its keys macOS keystrokes and its mouse the window's own (`tests/window.rs`).
- **The screens as tests**: the wringer's screens in every theme, nothing cut and no problems
  (`tests/wringer.rs`); the public demo as its agent leaves it (`tests/demo.rs`); Daybook's and
  Nocturne's flows (`tests/journal.rs`, `tests/nocturne.rs`).
- **The fuzz** (`fuzz.rs`): random keys, clicks, drags, the wheel, pastes and resizes over every
  example, checking that nothing panics, every act stays in its time budget, a replay gives the
  same state, and laid-out nodes nest in their parents without overlapping. Clean at 400 runs in
  release before a step lands.

## Rules

1. **Elm, strictly.** A serializable `State`, every input a `Msg`, a pure `update` and a pure
   view. Runtime handles stay outside the state; time comes from `State::now_ms`.
2. **Data, never code.** A pushed UI is data. It reaches the outside world only through an action
   (`run`, `load`, a host action) or a data source, and those will sit behind a command policy.
3. **Behaviour lives in the runtime; arrangement lives in data.** Don't add a component that only
   arranges or styles others; that's a project component or a recipe.
4. **One source of truth.** The UI value, the data and the view state each live in one place;
   everything else refers to them by id.
5. **Agents read back what they write.** Every feature shows in `fictty screen --nodes` or
   `fictty get --state`.
6. **Adopt what GPUI has.** Taffy lays out, GPUI paints; write our own only for behaviour or
   speed (our ellipsis, for instance, because GPUI's breaks under Taffy's measuring).
7. **Special cases are a smell.** A branch for one UI means the model is wrong; fix the model.
8. **Engine bugs go to caretline.** Text-editing problems are fixed there, released, then pulled
   in.
