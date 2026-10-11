# The guide

Everything the prototype does today, with the demos and the numbers. The [README](../README.md)
is the short version. To see it first: `curl -fsSL https://fictty.com/try.sh | sh` runs the
showcase in your terminal (it fetches fictty into `~/.fictty`, building it or using a release
binary; delete that folder to undo). Releases are described in [releasing.md](releasing.md).

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

The quickest way to see everything is the showcase. It opens on a tour of nine chapters, each a
lesson, something to play with and a quest: a screen and its JSON side by side, every component,
the store with live revisions, a dialog over a stack, themes, an autopilot driving the screen
while you watch, and a firehose beside 100,000 rows that filter on every key. The apps (notes, a
ledger, a live market, a gallery of components, an agent's log) are one key away. Every screen is
JSON, sharing one store and three themes.

```sh
examples/showcase/play.sh       # builds fictty if needed; n/p move through the tour, a the apps
examples/showcase/tour.sh       # in another terminal: an agent drives the apps
python3 examples/showcase/tour.py   # rewrites the tour's chapters from one script
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

- **Components** (a closed set): layout `col`, `row` (both with a `gap`) and `stack` (children
  over each other); `text`, `big`, `spinner`; `list`, `tree`, `table` (with column `total`s);
  `editor`, `input`, `button`; `tabs`; `sparkline`, `chart`, `bars` (up, or `horizontal`), `gauge`;
  and the pixel ones below. Every node can have `id`, `size` (`12`, `"30%"`, `"*"`, `"2*"`) with
  `min` and `max`, `title`, `border`, `pad`, `variant`, `tone`, `style`, `bind`, `keys`, `click`,
  `when`, and in a stack `float` and `modal`.
- **Lists and trees** share their row options: `item` (a template), `row_variant` and `variants`
  (a template per kind of row) with `skip` (rows that can't be selected), `mark` and `lead`,
  `selected` (a style), `lines` (rows of several lines), `keep` (cells the left side keeps before
  the right side, after a tab, is cut), `key` (default `id`: the selection follows its row when
  rows come and go) and `filter` (a store entry whose words every shown row must contain). A
  `tree` nests rows under each row's `nest` (default `children`), folds with ←/→ and space, keeps
  its folds by key, and gives templates `{$depth}`, `{$kids}` and `{$open}`; its `leaf` is the
  glyph for rows with nothing under them. A shown row carries `$ptr`, where it sits in the bound
  entry, so store actions reach it through a filter or deep in a tree.
- **Overlays**: a `stack` draws its children over each other, the last on top; one with
  `float` (`width`, `height`, `at` an edge or corner, `margin`) sits where it says, and a `modal`
  one dims what's behind it, swallows clicks there and keeps the focus; both cover what's under
  them. **`when`** shows a node only
  while a condition holds: a bind that's set (`"dialog"`), not set (`"!dialog"`), compared
  (`"mode=edit"`, `"$screen.width>=120"`), or several joined by ` & `. A hidden node takes no
  room and no focus. Layers take `when` too. **Toasts** show in the corner for a while: the
  `toast` action, `fictty toast`, and failed commands and host actions.
- **Sources** in `data`: `cmd` (argv), `shell`, `stream` (a command printing JSON lines), a
  literal `value`, or `host` (the embedding app's, with `args` that are templates over the state,
  so `{"host": "detail", "args": {"id": "{@list.id}"}}` refetches as the selection moves); `path`
  narrows the JSON, `every` re-fetches. A push fetches the sources that are new or changed; one
  that's the same keeps its value, stream and timer (`refresh` fetches them all). Built
  in: `$stats`, `$caps`, `$theme`, `$screen` (the terminal's `width` and `height`), `$keys` (a
  chord's keys so far), `$store` (every entry, by revision) and `$log` (what agents did over the
  socket, newest first, at the local clock time; a write that was refused shows as `refused`,
  with why). `$self` says where the screen runs: `bin`, `socket`, `dir` (its UI's folder), `cli`
  (a command line that reaches this screen from any terminal) and `env` (a line that sets a
  terminal up so `fictty`, and the scripts beside the UI, reach it).
- **Binds and templates**: `bind` names a store entry or a source, then a path (`books.running`);
  `@id` is the selected row of another list. Text and columns are templates over the bound
  value: `"{text} · {due}"`, `<accent+b>styled</>` (or any theme part: `<link>`), `<<` for a
  literal `<`, `{±change}` coloured by sign, `{name:12}` fitted to 12 cells (`{n:>6}`
  right-aligned), maps (`{done|true=☑;*=☐}`), and a tab right-aligns the rest of a line.
- **Actions** on keys: `quit`, `up`, `down`, `top`, `bottom`, `focus_next`, `focus_prev`,
  `refresh`, `{"run": [argv with {field} of the selected row]}`, `{"load": "{file}"}`,
  `{"focus": "id"}`, `{"theme": "next"}`, `{"copy": "text"}` (onto the system clipboard, through
  the terminal's OSC 52), `{"toast": "Saved", "tone": "pos"}`, `{"host": "name", "args": {…}}`,
  and store actions: `append`, `put`, `remove` (the selected row, or with `where` every row that
  matches), `toggle` (a true/false field of the selected row: a checkbox), `do`. A node's `click`
  runs an action too, after the click selects the row; a `button` runs its click on Enter and
  Space. A focused node's keys win over the UI's, which win over the defaults; a binding of
  several keys (`"p h"`) is a chord. While fictty has the mouse, hold Shift (Option in some macOS
  terminals) to select text the usual way.
- **Focus**: Tab and Shift-Tab move it, a UI's `focus` says where it starts when pushed, and Esc
  leaves a text field (dropping a selection first). `{"focus": …}` takes an id, a template
  (`"{store.from}"`) or `$prev` (where it was before it last moved). Choosing a tab (a click, or
  `{"tab": 2, "of": "views"}`) moves the focus into it; a tabs node's `tab` is the one it starts
  on, and tab titles take markup.
- **Identity**: view state (selection, folds, carets, scroll) and focus are keyed by node id, so
  they survive a push for any id that's still there (React's `key`).
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
  a number, and its keys wait until the text is a whole one (the status says why).
  `"syntax": "markdown"` styles headings, list markers, quotes, inline code and task items
  (theme parts `md.heading`, `md.bullet`, `md.quote`, `md.code`, `md.task`, `md.done`) while the
  text stays plain. Ctrl-T makes the lines it's on tasks (`text` → `- [ ] text`, `- item` →
  `- [ ] item`) and ticks open ones; a ticked one is unticked, or with `"tasks": "cycle"` made
  plain text again. A click on a `[ ]` ticks it. `"readonly": true` shows a text to
  read, scroll, select and copy, never change. `"tab": "indent"` keeps Tab and Shift-Tab to
  indent and outdent the selected lines or a list item (Esc leaves instead); `"indent-lines"`
  always indents the caret's line, as an outliner does. A `put` by the person
  into a text leaves the caret after it, and keys typed while a host's text is loading land once
  it has.
- **Editing keys** follow the Mac text field, each with a twin for terminals that don't send ⌘:
  ⌘C/⌘X/⌘V (Ctrl), ⌘A (⌥A), ⌘Z and ⇧⌘Z (Ctrl-Z, Ctrl-Y), ⌘←/→ (Home/End, Ctrl-A/E), ⌘↑/↓
  (Ctrl-Home/End), ⌥←/→ (⌥B/F), Shift with any of them to select, ⌘⌫ (Ctrl-U), ⌥⌫ (Ctrl-W),
  ⌥Del (⌥D). The screen turns on the kitty keyboard protocol where the terminal has it, so ⌘ keys
  arrive (as `d-`); copy and cut go to the system clipboard; a paste types. The mouse places the
  caret, Shift-click extends, a drag selects, a double click takes a word and a triple the line.
  `tests/editing.rs` is the contract. WezTerm doesn't pass ⌘ to programs (⌘Z arrives as `z`, and
  ⌘C copies without the program seeing it): `fictty setup wezterm` installs a small module that
  sends those keys only to panes running a fictty screen (it says so with a terminal user var),
  for any app built on fictty. Ghostty and kitty need nothing.
- **The wheel** scrolls a list, tree or table three rows a notch and leaves the selection where it
  is; a key that moves the selection out of view brings the view back to it.

## Themes

`fictty run --themes themes.json` loads named themes; the first is chosen, `{"theme": "next"}` on
a key cycles, `fictty theme <name>` picks one. A push doesn't change them. A theme has `tokens`
(colours), `parts` (how the runtime draws `table.header`, `row.selected`, `border.focused`,
`title`, `editor.caret`, `input.placeholder`, `status` and the rest, as style specs like
`accent+b`) and `variants` (named looks: a style, an edge, a backdrop, a title style). Nodes ask
for a `variant` and a `tone`, never a colour. A UI's own `theme` map still works, as tokens under
every theme. `examples/showcase/themes.json` has three: glass, paper and phosphor.

## Embedding fictty in an app

fictty is a library too: the `fictty` binary is one user of `fictty::App`, and an app is another.

```rust
let ui: fictty::model::Ui = build_my_ui();          // a value; no file needed
fictty::App::new(ui)
    .socket(my_socket)                               // agents reach it as they reach any screen
    .themes(my_themes).theme("night")
    .host(MyHost::new())                             // the app's data and actions, in process
    .run()?;
```

- **`Host`** gives the screen what a standalone screen gets from commands. A source
  `{"host": "today", "args": {"limit": 20}, "path": "rows"}` calls `Host::fetch("today", args)`;
  an action `{"host": "done", "args": {"id": "{id}"}}` (a template, like the store actions) calls
  `Host::act`. Both run off the drawing thread. An action's answer goes in `$log` (a failure on
  the status line too), and the host sources fetch again after it.
- **`Notifier`**: `Host::start` gets one, for any thread. `changed("today")` refetches every source
  that reads `today` (the app's world changed; no polling), `write(key, write, by)` writes the
  store, and `msg` sends any message.
- **Everything the host says arrives as a message**, so `get --state`, `$log` and replays see it;
  the view never reads the host.
- **Tests without a terminal**: `App::new(ui).host(h).headless(80, 24)?` gives a screen to send
  `keys(&["j", "x"])` and messages to, and to read back as text (`screen()`, `ansi()`) or state
  (`state()`). Sources are fetched and actions done inline. See `tests/host.rs`.
- **Store entries the host keeps**: `Host::keys` names prefixes (`page/`). Such an entry is never
  seeded: when something first shows it the screen asks `Host::load` (a spinner meanwhile, the
  reason if it fails), and once it has changed and been still for half a second it goes back
  through `Host::save`. `Notifier::reload(key)` loads it again when it changed at the host; text
  comes in as an edit, so the person's carets and undo carry on. The `--store` file leaves these
  entries out.
- **Upgrade** re-runs the same command line as the new binary with `FICTTY_RESTORE` pointing at
  the saved state, so an embedding app keeps its state across `fictty upgrade` like the binary.

## Not yet

A policy on what a pushed UI may `run` (today: anything you could run yourself, over a socket
only you can open), flexbox layout, writes by path with a revision per record, `watch`, a widget
registry for embedding apps, history. The order is in [PLAN.md](PLAN.md).

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
