# Architecture

How fictty is layered, what belongs where, and the rules that keep it that way. The reasoning
behind each decision is in the dev blog post
[fictty: splitting out the layer agents build on](https://thoughtcontrol.app/blog/fictty/).

## Where fictty sits

| layer | owns | depends on |
|---|---|---|
| **caretline** | text: the editing engine, callouts (`caretline-layers`), tours (`caretline-tour`) | nothing terminal-specific |
| **fictty-core** | a UI as one value, the Elm loop, the store, data sources, the socket, the host, readback: the engine, with nothing that draws | caretline |
| **fictty** | drawing the state: the terminal's grid today, a native GPUI window next ([native.md](native.md)); pixels; the CLI; `App` for embedding. Re-exports the core | fictty-core, ratatui, caretline-layers, gpui (macOS) |
| **apps** | Thought Control, ownpurse, anything else: their data, commands and screens | fictty, caretline |

fictty knows nothing about any app. An app's screens are fictty UIs (JSON, or Rust that builds
the JSON), and its commands are the data sources and actions those UIs bind to.

## Inside a running screen

```
sources (commands, JSON-line streams, values)        agents and people (socket, keys, mouse)
          │ Data                                                 │ Push, Patch, Key, Click, …
          └──────────────────────────┬───────────────────────────┘
                                     ▼
                     update(&mut State, Msg) -> Vec<Effect>      pure
                                     │
                     view(&State) -> frame + hit regions + pictures   pure
                                     │
            ratatui: draw into a cell buffer, diff with the last frame, send only changed cells
            kitty graphics: rasterise pictures, send each once (cached by content), re-place what moved
```

- **The engine runs on its own thread** (`Live::spawn`, in the core): it takes in sources,
  socket requests and input as fast as they arrive, does its chores (saving, timers, the watched
  file) and rings whatever draws when the state changed. The terminal and the window are both
  "start the engine, then draw": each posts its input to the engine's `Inbox`, locks the state
  only while it builds a frame, and hands back what the frame laid out (`Drawn`: hit regions,
  each node's rectangle, editor sizes) so the mouse lands where things are and the readback
  says where they were drawn.

- **State** is serializable: the UI value, data, view state keyed by node id, focus, stats and
  the time (`now_ms`). If it isn't in the state, it can't be read back, replayed or tested.
- **Effects** (fetch, run, load, quit) are run by the runtime, outside `update`.
- **ratatui's cell diff** does the job a virtual DOM does in a browser. Every frame is drawn from
  the state; only the cells that changed reach the terminal. So pushing a whole new UI costs no
  more than drawing a frame.

## The vocabulary agents build with

```
 assembly            the UI value: what goes where, bound to what data
 theme               the look: tokens, type, a skin for every primitive
 project components  compositions as data, built once per project
 alphabet            primitives with behaviour, maintained here
 escape hatch        a custom widget in Rust, compiled into the app
```

- **Alphabet.** A small set of primitives that need behaviour or speed: layout, text, list and
  table, input and textarea (on caretline), tabs, layers, pixels, and keys, clicks, hover and
  focus for every node. A primitive is a ratatui widget plus what ratatui leaves to apps: data
  binding, behaviour, theme and a plain-cell fallback.
- **Theme.** Data in the state, so it can be pushed, patched and rewound. It owns the look only;
  layout stays with assembly.
- **Project components.** JSON compositions with named parameters, written once per project
  (an agent can write them). Substitution only: no loops or conditions.
- **Assembly.** What an agent writes most of the time. A2UI maps onto this layer.
- **Escape hatch.** New behaviour is Rust, registered as a widget and compiled into the app;
  `upgrade` swaps the binary and keeps the screen's state.

Patterns move down: a recipe that recurs becomes a project component; a component that needs
new behaviour is proposed for the alphabet.

## Rules

1. **Data, never code.** A pushed UI is data. The only way it reaches the outside world is an
   action (`run`, `load`) or a data source, and those will sit behind a command policy.
2. **Behaviour lives in the runtime.** Don't add a component that only arranges or styles other
   components; that's a project component or a recipe.
3. **One source of truth.** The UI value, the data and the view state each live in one place;
   everything else refers to them by id.
4. **Agents read back what they write.** Every feature must be visible through `screen` (text;
   `screen --nodes` for the shown nodes, whatever draws them) or `state` (JSON).
5. **Use ratatui where it fits.** Write our own drawing only for behaviour or speed.
6. **Engine bugs go to caretline.** Text-editing problems are fixed there, released, then pulled in.
