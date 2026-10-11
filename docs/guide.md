# The guide

Everything the prototype does today, with the demos and the numbers. The [README](../README.md)
is the short version.

## The model in one picture

A terminal UI that is one serializable value. The whole screen (a tree of components, the data
sources they bind to and the keys) is plain JSON. A running screen holds no other state, so you can
read the UI out, push a whole new one in, or replace one node, and it re-renders.

```
world (a market feed, any command that prints JSON)      agents and the person
   │  sources: {"cmd": [...], "stream": [...]}              │  put / merge / edit, typing, store actions
   ▼                                                        ▼
State { ui, data, store (versioned entries), local (view state by id), focus, themes, theme }
   │  update(state, msg) -> effects        (pure)          ← serializable, `get --state`
   │  view(state) -> frame                 (pure)
   ▼
terminal
```

Who owns what: the **UI** is the agent's (a push replaces it), the **store** is shared (only
writes change it, each with a revision), **view state** is the person's (selection, carets,
scroll), and the **theme** is the person's or the project's. The reasons are in
[design/state-and-theme.md](design/state-and-theme.md).

## Try it

The quickest way to see everything is the showcase: a menu of screens (notes, a ledger, a live
market, a gallery of components, an agent's log), each one JSON, sharing one store and three
themes.

```sh
examples/showcase/play.sh       # builds fictty if needed; t switches theme, esc goes back
examples/showcase/tour.sh       # in another terminal: an agent drives the showcase
```

And by hand:

```sh
cargo build --release                                     # binary: target/release/fictty
fictty run examples/tasks.json                            # in one terminal
fictty push examples/trading.json                         # in another: the whole UI swaps
fictty patch header node.json                             # replace one node by id
fictty key j x                                            # type keys
fictty get --state                                        # everything on screen, as JSON
fictty run --watch my.json                                # hot reload while you edit
fictty render examples/tasks.json --size 100x30 --keys j  # no terminal
fictty read ledger                                        # one store entry: rev, by, value or text
fictty put ledger rows.json --if-rev 12                   # write it, unless someone wrote since 12
fictty theme paper                                        # choose a loaded theme
```

## The model

- **Components** (a closed set): `col`, `row`, `text`, `list`, `table` (with column `total`s),
  `sparkline`, `chart`, `bars` (up, or `horizontal`), `gauge`, `big`, `tabs`, `editor`, `input`,
  and the pixel ones below. Every node can have `id`, `size` (`12`, `"30%"`, `"*"`, `"2*"`),
  `title`, `border`, `pad`, `variant`, `tone`, `style`, `bind` and `keys`.
- **Sources** in `data`: `cmd` (argv), `shell`, `stream` (a command printing JSON lines) or a
  literal `value`; `path` narrows the JSON, `every` re-fetches. Sources are disposable: a push
  fetches them again. Built in: `$stats`, `$caps`, `$theme`, `$store` (every entry, by revision)
  and `$log` (what agents did over the socket, newest first, at the local clock time; a write
  that was refused shows as `refused`, with why). `$self` says where the screen runs: `bin`,
  `socket`, `dir` (its UI's folder), `cli` (a command line that reaches this screen from any
  terminal) and `env` (a line that sets a terminal up so `fictty`, and the scripts beside the UI,
  reach it).
- **Binds and templates**: `bind` names a store entry or a source, then a path (`books.running`);
  `@id` is the selected row of another list. Text and columns are templates over the bound
  value: `"{text} · {due}"`, `<accent+b>styled</>`, `{±change}` coloured by sign.
- **Actions** on keys: `quit`, `up`, `down`, `top`, `bottom`, `focus_next`, `focus_prev`, `refresh`,
  `{"run": [argv with {field} of the selected row]}`, `{"load": "{file}"}`, `{"focus": "id"}`,
  `{"theme": "next"}`, `{"copy": "text"}` (onto the system clipboard, through the terminal's
  OSC 52; a template over the selected row, `store` and `$self`), and store actions: `append`,
  `put`, `remove`, `do`. A node's `click` runs an action too, after the click selects the row, so a
  list of commands people click to copy is `"click": {"copy": "{$self.env} && {cmd}"}`. A focused
  node's keys win over the UI's, which win over the defaults. While fictty has the mouse, hold
  Shift (Option in some macOS terminals) to select text the usual way.
- **Identity**: view state (selection, carets, scroll) and focus are keyed by node id, so they
  survive a push for any id that's still there (React's `key`).
- **Problems**: a bound value of the wrong shape, an unknown variant or tone, comes back as a
  sentence in `problems` (in `get --state`, in replies, on the status line).

## The store

The app's data, apart from the UI: named entries, each a JSON value or a text (a caretline
document with its undo history), each with a revision (`rev`) and who wrote it last (`by`).

- **A push never changes it.** A UI can seed entries that don't exist yet (`"store": {…}`, or an
  editor's `text`); a seed never overwrites.
- **Writes**: typing in an editor or input; store actions on keys; and from agents,
  `fictty put` (a value, or a string as a text's new contents), `merge` (RFC 7396) and `edit`
  (`[[from, to, "text"]]` ranges of a text). With `--if-rev N` a write is refused if anyone wrote
  since revision N, and the reply carries the entry as it is now.
- **An agent's text edit is an edit**: the person's carets move with the text, and one undo takes
  it back.
- **`fictty run --store file.json`** keeps it across runs, written atomically after each change.
- **Editors**: `{"type": "editor", "bind": "@notes", "doc": "note/{id}", "text": "# {title}"}` edits
  the entry its template names from the selected note, so a list and an editor make a notes app.
  An editor takes every key but Tab, Shift-Tab and Esc; `input` is the one-line version, and
  Enter runs its `keys.enter`. `"accept": "number"` makes an input refuse typing that can't make
  a number, and its keys wait until the text is a whole one (the status says why). A click places the caret; `"syntax": "markdown"` styles headings,
  list markers, quotes and inline code (theme parts `md.heading`, `md.bullet`, `md.quote`,
  `md.code`) while the text stays plain.

## Themes

`fictty run --themes themes.json` loads named themes; the first is chosen, `{"theme": "next"}` on
a key cycles, `fictty theme <name>` picks one. A push doesn't change them. A theme has `tokens`
(colours), `parts` (how the runtime draws `table.header`, `row.selected`, `border.focused`,
`title`, `editor.caret`, `input.placeholder`, `status` and the rest, as style specs like
`accent+b`) and `variants` (named looks: a style, an edge, a backdrop, a title style). Nodes ask
for a `variant` and a `tone`, never a colour. A UI's own `theme` map still works, as tokens under
every theme. `examples/showcase/themes.json` has three: glass, paper and phosphor.

## Not yet

A policy on what a pushed UI may `run` (today: anything you could run yourself, over a socket
only you can open), flexbox layout, collections keyed by id and writes by path, `watch`,
history. The order is in [PLAN.md](PLAN.md).

## Stress demos

`examples/stress/`: a 500 updates/s ticker, a 160×50 text plasma at 120 Hz and a
50,000-row list replaced 20×/s (all `stream` sources fed by `feed.py`), plus `storm.py`, which
drives the screen over its socket with 1,000 patches/s or whole random UIs as fast as it can.
`play.sh` runs them all on the running screen and prints the counters. The status bar always
shows fps, the mean and worst draw time and messages/s; UIs can bind them as `$stats`.

Release build, a 170×52 terminal, measured on one machine:

| demo | fps | draw | msgs/s |
|---|---|---|---|
| idle | 2 | 0.4 ms | 0 |
| ticker (500 updates/s) | 114 | 0.23 ms | 500 |
| plasma (120 frames/s of 8,000 cells) | 110 | 0.24 ms | 122 |
| 50k-row list replaced 20×/s | 20 | 0.18 ms | 20 |
| patch storm, 1,000 patches/s | 116 | 0.22 ms | 1,000 |
| push storm, whole UIs as fast as a client can send | 108 | 0.84 ms | ~6,200 |

With the store, editors and problem checks (10 October 2026, the same machine, a 170×52
pseudo-terminal, both builds measured back to back): the ticker, plasma, list and patch storm
are unchanged; whole-UI pushes as fast as a client can send went from about 5,800 to 5,000 a
second, the cost of checking every push for problems.

## Finance dashboard, mouse and layers

`examples/finance/`: `fin.py` streams a made-up book (KPIs, a P&L statement, 16
positions, sector returns, regions, cash bridge) at 10 Hz; `gen.py` writes `dashboard.json`, four
tabs on that one stream; `tour.py` is an "analyst" agent that walks through it over the socket.

```sh
cargo run --release -- run examples/finance/dashboard.json
python3 examples/finance/tour.py        # from another terminal
```

- **Components** added: `chart` (lines on shared axes), `bars` (negatives hang below zero),
  `gauge`, `big` (block-digit KPIs), `tabs`, `map` (a world map with points). Table columns take
  `align` and `heat` (a cell's background from its value); `{±x}` in any template colours a value
  by its sign. `theme` names the colours; `edge` picks the border.
- **Mouse**: click focuses, selects a row or switches a tab; the wheel scrolls; hover highlights
  the row under the pointer. A node's `tip` is a callout that follows the hovered row, filled
  from it. `click` on a node runs an action. The view returns each frame's hit regions; the
  runtime turns pointer events into `Click`, `Scroll` and `Hover` messages.
- **Layers** (caretline-layers 0.2): `layers` in the UI, or `fictty layers file.json` /
  `{"op":"layers"}`, places callouts, arrows, rings (`pulse` animates them) and spotlights on any
  node (`on`) or row (`row`), each with its own `look` (colours and `edge`), signed by `by`. `bind`
  fills a callout from live data.
- **Master-detail**: `bind: "@positions"` is the selected row of node `positions`.
- **Agents drive it** like a person: `click`, `mouse` (move, click, wheel at a cell) and `screen`
  (the current frame as text, to see what's there).

## thc's Today, as a scene UI

`examples/thc/`: `today.json` (from `gen.py`) is thc's Today screen built from
scene components with the ember-dark tokens; `thcfeed.py` turns `thc today`, `thc q`,
`thc show` and `thc history` JSON into a view model (rows, meta, details). `compare.sh` renders
thc's own TUI and the scene UI after the same keys and compares them cell by cell (character,
colours, attributes): all 13 states, 78,000 cells, are identical.

The template language that took: `<accent+b>…</>` styles (theme tokens, `on-<colour>`, `b d i u
r s`), `{status|todo=[ ];done=<done>[x]</>}` maps, `{x|?then $}` and `{x|!else}`, `{x|markup}` for
values that carry their own styles (layered on the surrounding style; `<=spec>` replaces it), a
tab to right-align the rest of a line, `{*─}` to fill it, and lines whose placeholders all come
out empty are left out. Lists take `variant`/`variants` (a template per row kind), `skip` (rows
the cursor passes over), `mark`, `lead` and `selected`; `rule` draws a line; `status_bar: false`
gives the UI every row.

## Pixels

`examples/pixels/`: thc's Today on pixel cards. Where the terminal speaks the kitty
graphics protocol (Ghostty, kitty, WezTerm), a node's `backdrop` is a rounded, shadowed, gradient
panel drawn under the text, `plot` draws anti-aliased lines with gradient fills, `image` places a
PNG and `dot` a lit, glowing sphere; elsewhere they fall back to borders, braille and text. The
view stays pure (it asks for pictures in cells, colours and proportions); the runtime rasterises
them at the real cell size, sends each once (cached by content) and re-places only what moved.
`scale` asks for larger text (OSC 66) where a startup probe finds the terminal draws it.

The pixel demo shows thc's Today screen, so it needs [thc](https://thoughtcontrol.app) installed.

```sh
cargo build --release
python3 examples/pixels/mark.py examples/pixels/mark.png   # once
target/release/fictty run examples/pixels/pixels.json
target/release/fictty upgrade     # after a rebuild: same screen, same state, new binary
```

### Graphics: what to reach for (guidance for agents)

1. **Components first.** `backdrop` (cards), `plot`, `dot`, `big`/`scale`, tables, lists: the
   engine draws them at the terminal's real pixel size, caches them, keeps them in the theme and
   falls back to cells on terminals without graphics.
2. **SVG for anything custom** (icons, diagrams, a progress ring): an `svg` node with inline
   markup, `{field}` placeholders filled from its `bind`, so data drives the drawing. The engine
   renders it (resvg, system fonts) fitted to the node; `alt` is the text fallback. Compute
   geometry in the data source (an arc's length, say): templates fill values, they don't do maths.
3. **PNG only for real bitmaps** (photos, screenshots), as `image` with a path; the terminal reads
   the file itself.
4. **Animate data, not pictures.** Change a source or `patch` one node; the engine re-renders only
   what changed. A patch replaces the whole node, so send it with its `backdrop`.

How pixels travel: the kitty graphics protocol (Ghostty, kitty, WezTerm) takes RGB, RGBA or PNG,
never SVG, so vector art is rasterised here. Generated pictures go through a temporary file the
terminal reads and deletes (`FICTTY_PIXELS=direct` sends them in-band, for SSH); cards sit
under every cell background so selections paint over them; a spotlight is a translucent sheet
over everything.

Measured (210x60, release): about 0.9 ms a frame without pictures; with a 15 Hz plot, about 7 ms,
nearly all of it rasterising and compressing on the main thread. Next: rasterise on a worker
thread and present pictures when they're ready.

`examples/pixels/tour.py` drives all of it over the socket on a seeded sample vault with its own
config (thc's Today spans every registered vault; the demo shouldn't see yours).
