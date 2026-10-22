# Changelog

Everything that has changed in fictty, newest first, from the prototype on 9 October to today. All
dates are 2026. Four releases were tagged, all on 11 October; everything since is unreleased work
on `main`, broken up by day and theme. Since 11 October fictty has been moving from the terminal
into a native window, and since 13 October the terminal is dropped: the phases are in
[native.md](design/native.md), the layout and style stack in [expressive.md](design/expressive.md),
step 3 in [layout-step.md](design/layout-step.md), step 4 in [rows-step.md](design/rows-step.md),
and what comes next in [building-blocks.md](design/building-blocks.md) and [the plan](PLAN.md).
How to use what's listed here is in [the guide](guide.md); how a release is cut is in
[releasing.md](releasing.md).

## Unreleased (since v0.2.2)

Six days of work after the last tag: a fuzz driver and a pseudo-terminal sim, then the move to a
native GPUI window on macOS (phase 0, then phase 1's steps 1 to 4), a public demo, two dogfooding
apps (Daybook and Nocturne) and the fixes they found.

### 22 October: a handle to resize, and editors' scrollbars

- `"resize": "<store key>"` on a node: a grip on its bottom edge (the `resize.handle` part, a
  `resize` cursor) drags it taller or shorter. The node follows the mouse (`Msg::Resize`, the
  height held in its view state as `sizing`), and when it lets go the height goes into the store
  under the key, in pixels, so it outlives the session and, with a key that isn't a row's, holds
  across rows (Caption Genius's caption fields). Down a column the height stands for `size`,
  within `min` and `max`. The readback shows `(resize 180px)`.
- An editor with more text than it shows draws the scrollbar lists have, and the readback shows
  its `(scroll 3/12)` in rows (`state::editor_rows`).

### 17 October: rows as nodes (step 4) and the demo audit

**Lists, trees and tables**

- A list's view has one model, `state::Lines`: its lines (rows `cols` to a line), each line's
  height in cells and the view's height, with `top` in lines. Following the selection, the
  wheel, paging, what's in view, what the window builds and the readback's scroll all ask it,
  where each did its own arithmetic on a count of rows (step 4a).
- A row template with `size: "fit"` makes rows as tall as what they show. The frame reports the
  height of each row it built, the state keeps them by row key (dropped on a new width), rows
  not built yet count as the average, and a line is as tall as its tallest row. A fit list, or
  a list in a scroller, takes its lines' total as its height. A 50,000-row list of fit rows
  takes 12.6 ms a frame headless against 10.9 for fixed rows (4a).
- A button in a list's row template keeps its click: a press on it selects that row, without
  the list's own click, and runs the button's click on it (`Msg::Press`, by the button's path in
  the row template), so a card's ♡ loves that card. Keys stay the list's. Tests press it with
  `at_part`, and an agent with the socket's `click` and a part (4b).
- Fixed: a click on a row whose action wrote the store sent the selection back to the row it
  left, because the click didn't note the new row's key (4b).
- Trees in the window have their indent (two cells a level) and fold glyph (▾, ▸, or the tree's
  leaf), styled by `tree.guide` and worked out once in the core (`model::fold_lead`) for the
  window and the readback. A press on the glyph is a press on the row's part `fold`: it selects
  the row and folds or opens it. A tree's rows can be a row template, as a list's can, with its
  buttons reaching their own row. The window drew trees flat before (4c).
- Tables in the window lay every line (header, rows, a footer of totals) on the same columns:
  `size` in cells, px, a percentage or a share, and `align` for titles and cells. They were
  equal shares that ignored size, align, heat and total. A `fit` column is a problem rather than
  a guess, since each line is laid out apart (4c).
- `heat` on a table column has one meaning, where the value sits from `min` to `max`
  (`Heat::at`), drawn as the theme's `neg` or `pos` over the cell, stronger towards the ends
  (4c).
- Table totals are `Column::total`, formatted by `model::grouped` (the terminal's money format,
  moved to the core) and read back as the table's footer (4c).
- A table column's `cell` is a node template filled from `$item`; its buttons are the row's
  parts, the column first (`Node::row_part`), so a ✕ in a row removes that row (4c).
- A list, table or tree's title takes `{$rows}`, how many rows it shows after its filter. Inside
  an `each` it's the list's own count, where `{$count}` is the each's.
- Tabs count their choices by their titles, not their pages, and a tab without a page shows
  nothing, so tabs alone are a bar of choices (a radio group). More pages than tabs is a problem.
- Tab titles, and the tab bar's lead and trail, take `{$store.key}` as texts do.

**Styling and themes**

- `on` is a condition, read as `when` is: while it holds the node is in its on state, styled by
  the theme's parts (`button.on`) and its variant's, and the readback says `(on)`. A toggle is
  one node, where Nocturne had Play and Pause, ♡ and ♥ as pairs swapped by `when`.
- A variant takes `parts` of its own over the theme's, name by name with their states, so one
  list's rows can be cards while another's stay rows.
- `justify`, `items` and `self` take a placeholder (a value, or kept as written until a template
  fills it; a problem if it's left unfilled), so a chat bubble sits on its message's side from
  its data.

**Templates and actions**

- `{$store.key}` in a template reads a store entry beside the bound value, so one text can say
  NOW PLAYING or PAUSED where two were swapped.
- `{$item.x}` takes filters as any placeholder does.
- `{"select": "next", "in": "tracks"}` (and `prev`) moves another list's selection a row,
  without moving the focus there and back.
- Fixed: an `each` instance filled the `$item` inside a list's row template with its own item,
  so every kanban card's title was empty. Filling now stops at a template inside the template.
- Fixed: a whole list's height rounded down, so it was half a cell short and scrolled.

**Window**

- The node renderer's stack, table and tabs arms are methods of their own, as the grid's is:
  Nocturne's deeper tree overflowed a debug test thread's stack.
- Known: a `fit` node held by `max` inside a fit row is measured one line tall (Taffy measures
  under max-content without the max). [rows-step.md](design/rows-step.md) records it, and the chat
  bubble is built from shapes that measure right.

**Examples and demos**

- The wringer's chat is a thread of bubbles, a row template on each message's side with a look
  from its data, and kanban's columns are lists of cards as tall as their titles. Both run in
  every theme without problems or cuts in `tests/wringer.rs` (4d, which landed before 4c).
- The wringer's sheet wrote its heat value as a path (`"q1"`) where it's a template (`"{q1}"`),
  so its heat never showed; fixed, and it joins the every-theme test.
- Nocturne without workarounds: its toggles are single nodes with `on`, its heading reads the
  store, n and p use `select next`, its shelf's rows are cards through variant parts, its spacers
  are justified rows, and its player reads `playing` and `shuffle` from the store rather than
  from which button shows.
- The other demos audited the same way: empty spacer texts are gone from the player, Kestrel's
  and mail's button rows; the settings radio group has no empty pages; kanban no longer stores
  each column's count beside its cards; Kestrel no longer binds its tabs to a store entry only
  to show an unread count; chat's Send button sends, sharing the composer's Enter action.

**Docs**

- [rows-step.md](design/rows-step.md) plans step 4, rows as nodes: one model of a list's view and
  fit rows, a row's own buttons, trees and tables, then chat and kanban rebuilt.
- [building-blocks.md](design/building-blocks.md) writes up the demo audit: Aram's rule for demos
  (only the building blocks, no workarounds), the signs of a workaround, what each demo needed
  and the blocks it got, what was kept and why, the gaps left, and the next steps in order.

### 17 October: wrap, grid and scroll (steps 3c to 3f)

**Layout**

- A list's `cols` takes `{"min": len}` beside a count: as many rows to a line as fit, each at
  least that wide. The width the frame laid the rows across is kept in the list's view state,
  `state::per_line` is the one place the count is worked out, and the readback says
  `(N to a line)`. Nocturne's shelf went from six copies of one list, one per width, to one.
- `cols` also takes a `max` beside its `min` (`{"min": "240px", "max": 3}`), so three features
  on a wide window don't become three and two empty columns.
- `wrap: true` on a row, col or each flows the children that don't fit onto the next line, by
  their own sizes (3c).
- A `grid` lays its children out a line at a time in columns that line up: `cols` is a count,
  `{"min": len}` for as many as fit, or a size per column (`["200px", "*", "2*"]`), and a child
  takes `span` columns. An `each` with `cols` is a grid of its instances. A list's `cols` takes
  sizes per column too (3c).
- A grid column that's a percentage or `fit`, `cols {min}` without an id to keep its width by,
  and `span` outside a grid are problems.
- `scroll: true` on a col, grid or each scrolls down, on a row across. Its offset and extent are
  in the state by id. The wheel pans the innermost node under it and what a list or scroller
  can't take goes to the scroller around it. Page Up and Page Down page a focused list, else the
  scroller around the focus. The readback says `(scroll 12/40)` (3d).
- The focus is brought into view whenever it moves, its list's row moves, a key comes or a paste
  arrives (not on a page of a scroller, and not on a UI's first focus). The wheel doesn't, since
  the fuzz caught it scrolling the focus away (3d).
- `{$index}` and `{$count}` in each and row templates; a float's `x` and `y` are lengths, so data
  can fill them; and an `each` with `stack: true` floats its instances where their data puts
  them, as a calendar's events (3e).
- A node clipped away by a scroller has no rectangle or hit, and a list's or editor's room is its
  whole box, not the part a scroller shows (or a list inside one scrolled itself instead of
  letting the page move).
- Fixed: since 3c each row of a list with `cols` sat in a slot whose height Taffy found by laying
  the card out again. The slot takes the row's known height now, and Nocturne's frame went from
  4.4 ms to 3.4 ms headless.
- The grid is drawn by its own method: inside the recursive node renderer its locals made every
  frame bigger and overflowed a debug test thread on Nocturne.

**Engine and CLI**

- `fictty run --watch` reloads the `--themes` file as well as the UI, replacing themes of the same
  names and keeping the one shown chosen.

**Templates**

- A map's catch-all can read the value as `$` (`{kind|done=✓;*=<accent>$</>}`); it always could,
  and is now documented and tested with the `?` and `!` forms beside it.

**Examples and demos**

- The wringer's settings, landing and calendar, and Kestrel's Floor, are rebuilt without tricks
  (3f): settings' labels are centred beside their fields and its save bar is justified; landing
  scrolls and its features and prices are grids of as many as fit, up to three; the calendar's
  hours are an `each` and its events an `each` with `stack`, placed at `y` from their data
  instead of a script working out the gaps; Kestrel's Floor cards are a grid of up to four that
  wraps on a narrow window. `tests/wringer.rs` and `tests/demo.rs` check them in every theme at
  two sizes.
- Nocturne's room scrolls: the hero is fixed, the shelf as tall as its covers, and the album
  chosen is brought into view; its test runs at 1280x800 too.
- Nocturne's player keeps time with Up Next hidden: `nocturne.py` writes `library.json` with each
  album's track lengths, which the player reads instead of the hidden list.

### 16 October: alignment and fit (step 3b), and Nocturne

**Layout**

- `justify` (start, center, end, between, around) and `items` (stretch, start, center, end,
  baseline) on a col, row or each place its children along and across it, and a node's own
  `self` overrides `items` for it. Unknown words fail to parse, naming the ones they take.
- `"aspect": "16/9"` (or a number) keeps a node's shape: it takes the side it's given and works
  out the other.
- `size: "fit"` makes a node as big as what it shows (flexbox's `0 1 auto`), held by `max` and
  giving way when there isn't room. An editor in a fit node is as tall as its rows wrapped at
  its width, so Headless now draws until the sizes it learns settle, up to four times. A cut line
  carries an unseen copy of itself for measuring only inside fit nodes, so other lists pay
  nothing for it.
- Fixed: the window drew a hidden node (its `when` false) as an empty box, so its row or column
  still put gaps on either side of it.
- Fixed: rows in a list with `cols` grew from nothing, so a short last line's rows came out wider
  than the rest. Each row is now a fixed share of its line.
- Fixed: of several nodes with one id (one shown per width), keys and clicks acted on the first
  in the tree, shown or not. Lookups by id now take the node that shows.

**Styling and themes**

- Colours take an alpha: a token, part or style may be `#rrggbbaa` (or `#rgba`), drawn see-through
  in the window; shadows multiply their own alpha into it, and an opaque alpha reads back as
  `#rrggbb`.
- The window draws a `gauge`: its style is the track and a new part, `gauge.fill`, the bar, with a
  label over it; the readback says its label or percent.
- A gauge's maximum is now `of`, a number or a template. Fixed: its old `max` never reached it,
  since every node's own `max` (how big it can grow) took the key first, so `"max": 100` drew
  100% in the terminal too.
- A type role named like a part (a role called `title`) is a theme problem naming both, since
  `<title>` also took the title part's ink.
- A token `edge` that isn't `rounded`, `square`, `plain`, `thick` or `double` is a theme problem;
  a theme that used `edge` as a border colour lost every corner.

**Actions and readback**

- An action from a node that isn't a list, tree or table reads the value it's bound to, so a
  love button bound to `@albums` with `{"toggle": "store.loved/{id}"}` loves the album it shows.
- The readback says a node was cut even when it has no id, naming it by the nearest id (or root)
  and the way down from there.

**Examples and demos**

- Daybook centres its writing column with `justify: "center"` instead of empty texts either side.
- Nocturne (`examples/nocturne/`): a listening room of twelve made-up albums whose covers and
  rooms `nocturne.py` draws with Pillow into the temporary directory, so nothing binary is
  checked in. Its theme sets three faces, gradients and coloured shadows, and the whole screen
  follows the album chosen. It plays: space, the transport, ♡, ⇄, a click on a track, search and
  Browse all work as data, and `player.py` keeps time over the socket. It fits a laptop: the
  sidebar and rail go when there's no room and the shelf holds three to eight covers a line.
- `tests/nocturne.rs` writes it with `--no-art` and checks no cuts and no problems at three sizes,
  that the room follows the album, and the controls.
- The release fuzz is clean after 3b.

**Docs**

- [native.md](design/native.md) no longer points at the spike's branch, which is removed; its
  numbers stay.
- The plan marks 3b done and 3c next, and places Nocturne's gaps in step 3's finish line.

### 14 October: lengths and boxes (step 3a), and a gallery from parts

**Layout**

- `pad`, `margin` (new, on any node), `gap`, `min` and `max` take one length type: cells, `"8px"`,
  or a name from the theme's `space` scale. `pad` and `margin` are one length for every side,
  `[x, y]`, or `[top, right, bottom, left]`, as in CSS, so a node's pad and a style's pad mean the
  same thing; a `gap` is one length or `[x, y]`. A length that isn't one is a problem, per node
  and field (3a).
- Lists scroll smoothly and show where they are: `top` is in lines and may be a fraction, the
  window turns a trackpad's pixels into `Msg::Pan`, a mouse wheel's notch is still three lines,
  and a scrollbar is drawn from the `scrollbar` and `scrollbar.thumb` parts (now defaults; shape
  `none` hides it). The readback says `(scroll 6/16)`.

**Lists, trees and tables**

- `multi: true` lets a list, tree or table pick several rows as a Mac list does: ⌘-click (⌥ in a
  terminal), ⇧-click and ⇧↑/↓ for a range, ⌘A for all, Esc or a plain move back to one. Picked
  keys and the anchor live in the node's view state, so they replay and survive rows coming and
  going. `$picked` gives each node's count, keys and rows to templates and `when`, and `toggle`
  and `remove` act on every picked row. Over the socket it's `s-click` and `d-click`.
- `$picked.<id>` has `index` and `total`, for "6 of 47".
- A list's row can be a node template, filled from `$item` as an each's is, so thumbnails are
  rows that select, pick, scroll and take keys. `cols` lays rows N to a line, with `next_row` and
  `prev_row` keys for one row on or back.
- Fixed: `row.selected.blur` painted the panel colour, so a list's selection vanished once the
  focus moved away. It's the hover grey now, as a Mac list out of focus is.

**Editors**

- An editor bound to a multi list's selection, with a `doc` per row, edits every picked row's
  entry. While they hold the same text, every edit goes to each (so undo steps back in each);
  while they differ it shows an empty text of its own with "Multiple values", and its first edit
  puts what it holds into every entry. A field left alone writes nothing. The readback counts the
  entries and says when they're mixed.
- Fixed: mirroring only edits let picked entries drift apart; every message now goes to each,
  and undo on a mixed field steps back in each entry's own history.
- Fixed: under a theme whose `screen` part names a type role, an editor drew in its monospace face
  at the role's size, so the caret sat inside the last word. The editor sets its grid's own face,
  size, line height and weight.

**Window**

- Images draw in the window with GPUI's `img` (PNG, JPEG, GIF, WebP, BMP, TIFF), whole inside the
  node or with `fit: "cover"`. `src` is a template over the bound value, so a picture follows a
  list's selection; `alt` or the path shows when the file can't be drawn.
- A click on a node without a click runs the nearest click around it, so a picture inside a
  clickable row no longer swallows the click.

**Styling and themes**

- The shown tab's part is `tabs.selected`, not `tabs.active`, matching `row.selected`; every theme
  in the repo is renamed.
- A font file a theme names that doesn't load is a problem naming the font, the file and why
  (`fonts_failed` in `get --state`); its text still shows in the default font.
- A default token (`accent` with no theme loaded) is no longer reported as an unknown tone; the
  check asks the same `Theme::knows` the resolver uses.
- A style's `border` can be written short, `"[width] [solid|dashed|double] [colour]"`; a string
  failed the whole push before, naming no node.

**Store, binds and templates**

- `toggle` of `store.remove/{id}` flips an entry named from each picked row, so a flag lives
  beside a row's fields and a row template shows it with a `when` on that entry.
- A `when` is filled from its node's bound value before it's checked, as `doc` and `src` already
  were, so a detail pane can show the selected row's flag.
- A bind names a store key with dots in it (`remove/photo.tif`): it reads the longest store key it
  starts with, and the rest is the path into it. It cut at the first dot before.
- A name that's both a store entry and a data source is a problem saying which one binds read;
  a stale seeded entry had silently hidden a source's new rows.
- `fictty delete` (the socket's write `delete`, with `--if-rev`) deletes a store entry; the log
  shows it and a replay repeats it.

**Socket and CLI**

- `fictty quit` (and the `quit` request) ends a screen wherever the focus is; `key c-c` is an
  editor's copy while a field has the focus.
- A running screen holds a lock on its store file, and a second screen on the same store refuses
  to start, naming the pid that has it. The lock lives in the temporary directory, named from a
  hash of the store's full path, so nothing is written beside a store in a synced folder.
- Fixed: a quitting window removed its socket path after a new screen had already bound a new file
  there, leaving it unreachable (one relaunch in five). A screen now removes the path only while
  it's still the file it made.

**Docs**

- [layout-step.md](design/layout-step.md) gains "Start here": the decisions to confirm, 3a as a
  list of commits, a release-build baseline of frame build time, measurements after 3a, and what
  3a taught for 3b.
- AGENTS.md records what CI taught (no terminal on CI, slow debug runners, two clocks in one test,
  Linux-only clippy), how to close a window with `fictty quit`, and to leave other people's
  screens alone.
- [gallery-notes.md](design/gallery-notes.md) records the session that built the fictty-ui skill
  and the gallery from parts (Aram's call: no gallery node). The plan ticks the gallery and the
  skill and drops to-dos the move to the window superseded; the guide names `next_row` and
  `prev_row`.

### 13 October: themes as whole styles (step 2), the public demo and Daybook

**Styling and themes**

- The layout and style stack is decided in [expressive.md](design/expressive.md): adopt what GPUI
  already has (Taffy layout, text, paint), themes as whole styles resolved in the core and read
  back by agents, a layout vocabulary, rows as nodes, and drag and motion as data. The terminal
  is dropped and the theme kit pulled forward.
- A theme token `edge` sets every node's border shape: anything but `rounded` squares the corners,
  and `thick` or `double` draws a panel's border heavier.
- Style resolution in the core (`style.rs`, step 2a): a node's part, the parts of the states that
  hold layered over it, then its variant, tone and own style, with tokens filled in as `#rrggbb`;
  its border, title, edge and backdrop resolve with it. The readback carries it:
  `fictty screen --nodes --style`, `fictty render --nodes --style`, and `style` in the JSON.
- Wrong names in a theme are problems: parts no component has, spec words that mean nothing,
  colours that aren't tokens, skips that aren't row variants, unknown scale entries, roles,
  shapes, cursors, border styles and fields no style has.
- Parts as whole styles (2b): a part, a node's `style` and a variant's `style` are a short ink
  (`"accent+b"`) or a whole style with `fg`, `bg` (a colour or a gradient), ink modifiers, a type
  role, `pad`, `radius`, `border`, `shadow`, `ring`, `shape`, `opacity` and `cursor`. Themes gain
  scales (`space`, `radius`, `depth`, `type`), fonts, and `motion` (kept for later).
- The window draws the resolved style (2c) through one mapping: background or gradient, ink, pad,
  corners, borders (dashed too), shadows, a ring, opacity and cursor. What it hard-coded (a
  button's pad, a panel's inset, tab pills, row hover, the input's box, Markdown's inks) is now
  default parts in the core, so the readback shows what's drawn. Tabs take the shape their part
  gives: pill, underline, segment or plain. Frames build in about twice the time (0.06 to 0.11 ms
  on the ticker) with the frame rate unchanged.
- The window reports any node under the mouse, not only rows, so `.hover` parts hold for buttons.
- Type roles and theme fonts (2d): `"style": {"type": "display"}` or `<display>…</>` in markup
  sets the family (or a theme's font file), size, weight, line height and tabular numbers.
  Tracking reads back but isn't drawn (GPUI 0.2.2 has no letter spacing).
- Our own ellipsis (2d): a one-line text is shaped once it knows its width, and what doesn't fit
  is cut with `…`, keeping the right side whole and `keep` cells of the left. `lines: n` shows at
  most n wrapped lines. The readback says `(cut)`. GPUI's own clamp kept Taffy's smallest measure
  and showed a single word.
- Cards and the status line (2e): a `backdrop` resolves into a card (fill or gradient, border,
  corners, a shadow or glow). The status line is the UI's: the status, else the first problem in
  the `status.error` part, else hints; `"status_bar": "speed"` adds the speed meter and `false`
  hides the line.
- The styling study (2f): `examples/window/themes.py` writes the wringer's six themes as whole
  styles sharing one vocabulary, so any screen wears any theme, and a test checks every example
  screen under every theme. [styling-study.md](design/styling-study.md) found type moves a
  screen's character most, then edges and depth, then selection and focus, palette last; its
  before-and-after screenshots were taken in the window.

**Editors and input**

- A ⌃, ⌥ or ⌘ key a focused editor has no use for (by caretline's keymap) goes on to the UI's keys,
  so ⌘T works while writing.
- Fixed: a click on a task's box ticked it and moved the caret into the box; the caret stays.
- An input's `required` holds its keys while it's blank.

**Templates and actions**

- Action templates read sources by name (`{today.id}`), behind the selected row's fields.
- `{"toggle": "store.zen"}` flips a store entry, not only a field of a selected row.
- Fixed: a `do` that stepped a list's selection and then wrote the store had the selection follow
  its old row back.
- Fixed: `fictty set <list> {"selected": n}` was undone at once, since the list still remembered
  the old row's key.

**Testing and CI**

- CI is green again, after failing since 12 October: the terminal fuzz builds a fresh emulator on
  a resize (ratatui's `Terminal::resize` asks the real terminal, which CI hasn't got), Linux's
  `openpty` signature is handled, and a debug build's fuzz time budget is four times a release
  build's (`FUZZ_BUDGET_MS` overrides it).
- The pty test spaces presses half a second apart on both clocks, so timing never turns them into
  double or triple clicks on one screen and not the other.

**Examples and demos**

- The wringer (`examples/window/`): nine designed screens, six themes and `open.sh`, the test of
  done for each step.
- The public demo (`examples/demo/`): `open.sh` opens the window and `agent.py` builds Kestrel
  Cycles, a made-up bike workshop, one logged command at a time (`push`, `put`, `screen --nodes`,
  `set`, `patch`, `merge`, `theme`, `screen --nodes --style`), then hands the app over with its
  live numbers running. One beat patches a stat card with a misspelt variant, reads the problem
  back and fixes it. [demo.md](design/demo.md) gives its rules.
- Fixed in the demo: the floor log's events carried markup in their data, which isn't parsed, so
  events now carry a kind the template maps to a tone; the logged `set` and `put` commands show
  reading stdin, as they really run.
- Daybook (`examples/journal/`): a journal after Thought Control's, one JSON value with no host
  (today on ⌘T, pages on ⌘N, ⌥[ ⌥] for days, zen on ⌥Z, tasks that cycle on ⌃T).
  `tests/journal.rs` runs its flows in the window, and the window fuzz runs it.

**Docs**

- [layout-step.md](design/layout-step.md) plans step 3, the layout vocabulary, and native.md's
  steps are reordered from 2 on.

### 12 October: the core crate, the engine thread and the window (phase 0, phase 1 step 1)

**Engine and embedding**

- The repo is a workspace: `crates/fictty-core` holds the UI value, state, store, lists, host, fuzz
  driver and engine, and never compiles ratatui or GPUI; `crates/fictty` draws it and re-exports
  the core, so embedding apps keep their imports.
- The engine (`Live`, grown from the runtime) owns the socket, sources, host, saves, the watched
  file and upgrade, on its own thread with `spawn`, ringing whatever draws; input arrives through
  an `Inbox`. In a pty it runs as fast as the old loop: 108 fps on the ticker and patch storms.
- Readback: `fictty screen --nodes` (`--json`), the `readback` socket request and
  `Headless::readback()` list the shown nodes in order with kind, id, text, focus, selection and
  the rectangle each was laid out in.

**Window**

- `fictty run --window` opens a GPUI window on macOS, driven by the engine's thread: keys, clicks,
  the system clipboard, `screen`, upgrade and quit go through fictty's own messages.
- The window reports what it laid out (rectangles, hits, editor and list sizes, `$width` areas,
  the wake time), so spinners turn and toasts go on their own.
- Editors draw caretline's own frame (caret, selection, wrap, scroll), and presses, drags and the
  wheel in their text select as in the terminal. Each row is one shaped run with selections drawn
  behind it, so glyphs no longer shift as a selection changes.
- The window draws the active theme (tokens as colours, parts and `<spec>` runs as inks) and
  template markup: colours, bold, italic, underline, strike, dim, reverse, signed numbers, a tab
  right-aligning the rest and a fill. Markdown roles move to the core (`model::md_roles`).
- The window pastes the system clipboard on ⌘V and hovers the row under the mouse.
- The window lays out like the terminal (step 1): sizes read once in the core (`Size::measure`:
  cells, `"48px"`, `"30%"`, shares); a fixed size shrinks when there's no room; `min`, `max`,
  `pad` and `align` work; a float sits at its edge or corner; a modal puts a scrim over its stack
  that takes presses and the wheel. What a frame notes is clipped to what holds it.

**Testing**

- `App::in_window(w, h)` is a headless screen drawn by the window under GPUI's test platform, with
  keys arriving as macOS keystrokes, pastes through the clipboard and ⌘V, and the window's own
  mouse handlers. `Headless::at(id, row, tab)` and `text_at(id)` say where something was drawn.
- `tests/window.rs` drives layout, presses, carets, drags, multi-clicks, the wheel, resizes, keys,
  paste and copy; the fuzz runs editors, lists, tables and every tour chapter through the window.
  `fuzz::laid_out` checks that nodes nest in their parents and don't overlap but over a stack.
- `.github/workflows/test.yml` runs clippy and the tests on macOS and Linux on every push, as a
  signal, not a gate.

**Docs**

- native.md's phase 1 checklist and its next steps in order; AGENTS.md lets an agent launch the
  native window itself on its own socket, drive it and close it.

### 11 October (after v0.2.2): fuzzing, the pty sim and the native plan

**Testing and fuzzing**

- `fictty::fuzz` (the `fuzz` feature) generates keys, clicks, double clicks, drags, the wheel,
  pastes, resizes and waits on fresh headless screens, with built-in properties (no panic, a time
  budget per act, the same acts replaying to the same state and frame, and the terminal showing
  the frame through a vt100 emulator). `fuzz::editing()` adds the editing invariants, an app adds
  its own with `fuzz::check`, failures shrink to the fewest acts, and `fuzz::replay` runs a printed
  sequence again. The generator is a seeded splitmix, so there's no new dependency.
- `Headless` takes the mouse, pastes and resizes, answers `screen` and `mouse` requests, says when
  its screen has quit, and skips frames when nothing changed.
- `App::commands(false)` turns off `run` actions and command sources, so tests never start a
  script.
- `fictty::sim` (the `sim` feature) runs any fictty binary in a pseudo-terminal with vt100 on the
  other end: it answers the terminal queries, sends keys as kitty CSI u or legacy bytes, the mouse
  as SGR reports, pastes bracketed, and resizes the pty. `tests/sim.rs` checks the binary against
  a headless twin. `FICTTY_SIM_TRACE=1` prints every byte.
- Driver host acts: `Act::Host` runs the screen's host handler inside an act, with
  `Config::host_acts`, `Config::normalize` and `Config::class` for apps' own fuzzing.

**Editors**

- `ordered` (say `["1.", "a.", "i."]`) keeps an editor's Markdown ordered lists in number after
  every edit, in numbers, letters or roman numerals, as part of the same undo step;
  `{"list_style": "next"}` changes one list's style.
- Fixed (found by the fuzzer): undo renumbered the text it had just put back, so it could never
  get past it; a paste didn't renumber; numbers after bullets at the same indent were counted as
  the bullet list; and bullets made numbers after a numbered list now carry its count on.
- Fixed: a key at the end of a 1,200-character paragraph took 573 ms in a release build; a line's
  Markdown looks and links are now worked out once, and the key takes well under a millisecond.
- Fixed: a whole-text put (an agent's or a host reload) moved every caret past it; it now goes
  through caretline's diff and edits only what changed.

**Engine and state**

- The expanded tree of `each` nodes is kept until the UI or a value an each reads changes: 21
  eaches went from 0.236 to 0.126 ms a key.
- `Msg::Focus` (and `Notifier::focus`) moves the focus and nothing else.
- `"wait": true` on a host action holds keys typed meanwhile until it has answered and its sources
  have landed.
- A running screen knows its size before the first push, so a `when` on `$screen` starts right.
- `State::derive` works out the expanded tree and rows of a state read back from `get --state`.
- Fixed: a list bound to its own selection recursed until the stack overflowed; it reads nothing
  and a problem says why.
- Fixed: the focus now moves off a node that hid, whatever hid it.
- Fixed: a host reload with unsaved typing saves the typing first, and a headless reload marks
  what it loaded as kept.
- Fixed: a click on a wiki link's `[[` or `]]` places the caret; Esc drops a selection first even
  where a node binds Esc; while a modal shows, a click or the wheel outside it goes nowhere.
- Fixed: a wide character with something drawn over its second cell (a toast's corner) kept both
  in the frame; the view drops the wide character so what's on top wins.
- A headless screen's clock stays still through agent requests, and its log has no wall-clock
  times, so the same writes replay to the same state.

**Docs and site**

- The demo for "Built for a real app" is frozen on v0.2.2, and the post is published.
- [native.md](design/native.md) plans the move to a native GPUI window: why, what stays and goes,
  the architecture, six phases, risks. Aram's answers: the name stays, macOS first, one app with
  tabs.

## v0.2.2 · 11 October 2026

**Examples and demos**

- Fixed: the tour's Pages chapter started with the focus in its editor and nothing else took
  keys, so Esc couldn't leave the page and `n` typed into it. A list of the pages now starts with
  the focus: Enter goes to a page, `o` opens it beside, and Esc from the editor comes back to it.

## v0.2.1 · 11 October 2026

**Examples and demos**

- A tenth tour chapter, Pages: a notebook of store entries where a click on a `[[link]]` opens the
  page beside it as another panel of a sidebar made with `each`, Shift-click goes there, a tag
  link raises a toast, Tab nests a line, Enter continues a list, and the main pane switches layout
  on its own `$width`. Opening a page marks it for a moment with a write that expires (`for_ms`),
  and every chapter's toasts sit in the bottom bar.

**Docs**

- The README, the guide and the plan say ten chapters and record the milestone.

## v0.2.0 · 11 October 2026

The kit, the library and the tour, then what Thought Control asked for from a real terminal.

**Embedding**

- fictty is a library: `App` builds a screen from a `Ui` value or a file, with a socket, a store,
  themes and a `Host`. A source `{"host": name, "args"}` calls `Host::fetch` and an action
  `Host::act`, off the drawing thread, with answers coming back as messages. `Host::start` gets a
  `Notifier` whose `changed(name)` refetches the sources that read it.
- `Headless` replaces `render`: a terminal-less screen to send keys and messages to and read back
  as text or state. `Headless::request` answers a socket request without a socket.
- Upgrade re-runs the same command line with `FICTTY_RESTORE`, so it works for an embedding app's
  binary too.
- Store entries a host keeps (`Host::keys`, by prefix) are loaded when first shown (with a
  spinner, or the reason it failed), saved with `Host::save` once changed and still for half a
  second, and left out of the `--store` file. `Notifier::reload(key)` reloads one as an edit by
  the host.
- A host source's `args` are templates over the state (`{@list.id}`, `{store.key}`), fetched
  again only when they come out different.
- `fictty::setup` exposes the WezTerm module as `wezterm_lua()` and `wezterm()`.

**Components**

- `tree` shows nested rows that fold with left, right and space; folds are view state by row id.
- Lists, tables and trees take a `filter`: an input's text narrows the rows to those matching
  every word, and a filtered tree opens to its matches.
- A shown row carries `$ptr`, a JSON Pointer to where it sits, so `remove` and the new `toggle`
  action (a checkbox) reach it through a filter or deep in a tree.
- `when` shows a node only while a condition holds; `$screen` gives the terminal's size; `stack`
  draws children over each other, a child with `float` sits at an edge or corner, and a `modal`
  child dims what's behind, swallows clicks there and keeps focus inside; `button` runs its
  click on Enter, Space or a click.
- Toasts (an action, the `toast` socket request and `fictty toast`) show until their time is up;
  the view tells the runtime when the frame next changes by itself, so nothing redraws in
  between. A `toasts` node shows them where a UI wants, with a part per tone (`toast.<tone>`), and
  `$toasting` says whether one shows.
- `spinner`, and a spinner in place of a source that hasn't arrived; scroll bars on long lists,
  tables and trees (`scrollbar`, `scrollbar.thumb`).
- `gap` on rows and columns, an accent underline under the active tab, and an ellipsis where a
  line's left side is cut.
- Lists and trees share their row options (`Rows`): mark, lead, selected style, `row_variant`,
  skip, a leaf glyph, `lines` for multi-line rows and `keep` for the cells kept before a cut. A
  selection follows its row's key when rows come and go above it.
- `each` draws a template once per item of a bound list, filling only `{$item}` and
  `{$item.path}`; the expanded tree is in `State::tree`.
- Container queries: `$width` and `$height` in a `when` are the size of the area the parent lays
  the node out in.
- Choosing a tab (a click, or `{"tab": N, "of": id}`) moves the focus into what it shows; a tabs
  node's `tab` is where it starts; tab titles are templates over the bind, take markup, and the
  bar takes a lead and trail.
- `min` and `max` bound a node's share of its row or column.
- Fixed: a list's row field was `variant`, shared with the node's own, so row variants were lost
  through every push and restore; it's `row_variant` now.

**Editors**

- Markdown task items (`- [ ]`, `- [x]`) are styled (`md.task`, `md.done`) and tick with Ctrl-T or
  a click on the box; Ctrl-T also makes any line a task, and `"tasks": "cycle"` cycles text, task,
  done, text.
- `readonly` shows a text the person can select and copy but not change; agents can still write
  it.
- Editing keys and the mouse follow the Mac text-field conventions: keys name every modifier in
  one order (`c-`, `a-`, `d-` for ⌘, `s-`), the kitty keyboard protocol is turned on where the
  terminal has it, bracketed paste types into the focused editor, a drag stretches the selection,
  Shift-click extends it, double and triple clicks select a word and a line, and Esc drops a
  selection before it leaves. An editor's copies go out over OSC 52 and its messages show as
  toasts. `tests/editing.rs` is the acceptance suite from Thought Control's editing contract.
- `"tab": "indent"` makes Tab and Shift-Tab indent and outdent the selection or a list item;
  `"tab": "indent-lines"` indents the lines the caret is on, never typing spaces.
- Enter continues a Markdown list (bullet, next number, an open task), and Enter on an empty item
  outdents it or ends the list.
- Links in editor text (`[[Page]]`, `#tag`, URLs) run their kind's click or Shift-click action
  with `{$match}`, and an action run with the caret on one reads `{$link}`; styled with `md.link`.
- `"caret": "end"` starts an editor's caret at the end of an entry the first time it shows it.
- Keys typed into an editor whose hosted text is still loading wait (`typed_ahead`) and land once
  it does; a put by the person leaves every editor after what it wrote.

**Templates and actions**

- `{key:11}` fits a value to 11 cells and `{key:>11}` right-aligns it; `<<` is a literal `<` in
  markup; a markup tag can name a theme part (`<link>`); a tab in a `|markup` value right-aligns.
- Chords: a binding of several keys (`"p h"`) waits for the rest, and `$keys` shows the keys so far
  and what could follow.
- A UI's `focus` sets where the focus starts; Esc in a text field with nothing selected moves to
  the next node that isn't one; `{"focus": …}` takes a template or `$prev`.
- Layers take `when`, and a `when` can join conditions with ` & `; `remove` with `where` takes
  matching rows from an array entry.
- `{"select": "{store.k}", "in": id}` and `set` with `key` pick a row by key, even one that hasn't
  arrived yet.
- `for_ms` on a write (`fictty put/merge --for-ms`, the put action, `Notifier::write_for`) takes
  it away again on the screen's clock, kept in `State::expiring`, so it replays.

**Engine and state**

- Fixed: a failed `run` or host action raises a toast as well as the status line.
- Derived rows settle only when a tree's or filter's rows changed (update back to 8.2 µs a message),
  and a source's new value re-derives only what reads it: a 100,000-row filtered list beside a
  stream went from 43 to 114 fps.
- A push fetches only sources that are new or changed, so the same definition keeps its value,
  stream and timer.
- A refetch asked for while one is under way runs again when it lands.
- A write over the socket goes through the driver, so its effects (a host source refetch) happen
  at once; an expiring write refreshes what reads it.
- A floating or modal stack child clears what's under it.
- Headless draws a frame after each key, as a running screen does.

**Socket and CLI**

- The wheel over a list, tree or table scrolls its view three rows a notch and leaves the
  selection where it is.
- `fictty setup wezterm` installs a WezTerm module that sends ⌘ keys as kitty bytes only to fictty
  panes (found by OSC 1337 `SetUserVar`); an empty `XDG_CONFIG_HOME` counts as unset.

**Releases**

- The release workflow cross-builds Intel Mac binaries on Apple silicon (GitHub retired its Intel
  runners) and rebuilds a tag's binaries by hand with `workflow_dispatch`.
- `try.sh` with no cargo runs the newest release; the stack post's demo moves to v0.1.0, so readers
  get a binary. [releasing.md](releasing.md) says how to release, with CI as a background signal.

**Examples and docs**

- The tour: nine chapters, each with a lesson, something to try and a quest kept in the store,
  including an autopilot driving the screen over the socket and a speed chapter of 100,000
  filtered rows beside a firehose. `tour.py` writes the chapters, and the one-liner opens the
  tour.
- The demo for "A kit for apps, and a tour" is frozen and the post published; the guide, plan,
  README and AGENTS.md cover the kit, the library and the tour.

## v0.1.0 · 11 October 2026

fictty as its own project: the store, editors, themes, the showcase, the site, and a one-line
demo.

**Engine and store**

- The prototype moves from Thought Control with its history, renamed to fictty throughout (the
  binary, `FICTTY_*` variables, `fictty.sock`).
- A store for the app's data, apart from the UI: versioned entries (a JSON value, or a text with
  its undo history) that a push never touches. Writes (`put`, `merge`, `edit`) take `--if-rev` and
  are refused when someone wrote in between; `fictty run --store file.json` keeps it, written
  atomically.
- `fictty set` moves view state; `$log` is what agents did over the socket, newest first. A
  refused write shows as `refused` with why, and entries carry the local clock time.
- Themes as their own layer: `fictty run --themes themes.json` loads named themes (tokens, parts,
  variants); `{"theme": "next"}` or `fictty theme <name>` switches; nodes ask for a `variant` and a
  `tone` instead of colours.
- Store actions on keys: `append`, `put`, `remove` and `do`; templates read the selected row,
  `{store.key}` and `{$id}`.
- `problems` lists what's wrong in sentences (a value of the wrong shape, an unknown variant), in
  the state, replies and the status line.
- Built-in sources `$theme`, `$store` and `$self` (the screen's binary, socket and folder).
- Commands the screen runs get `FICTTY_SOCKET` and `FICTTY`.

**Components and editors**

- `editor` on caretline: carets and scroll are view state, and an agent's edit goes through its
  own view, so the person's carets move with the text and one undo takes it back.
- `input`, a one-line field whose Enter runs `keys.enter`; `"accept": "number"`; an editor with
  `doc` edits the entry its template names.
- `"syntax": "markdown"` styles headings, list markers, quotes and inline code (`md.*` parts); a
  click places the caret and the wheel scrolls.
- `pad`, bars across, a table column's `total`, and `load` reading its file from the selected row.
- `{"copy": "…"}` puts text on the clipboard over OSC 52.

**Socket and CLI**

- `fictty render` takes `--store`, `--themes` and `--theme`.
- Fixed: under `curl | sh` the screen spun on macOS's generic `/dev/tty`; `try.sh` hands over the
  terminal's own device, and fictty refuses `/dev/tty` with a sentence.

**Examples and demos**

- The showcase: notes, a ledger, a live market, a gallery of every component and an agent's log,
  sharing one store and three themes (glass, paper, phosphor), with `tour.sh` driving it.
- `curl -fsSL https://fictty.com/try.sh | sh` runs it; `try.sh` takes a version, gives each install
  its own socket, and writes `fictty` and `fictty-tour` for a second terminal. The agent screen's
  commands copy on a click.
- `scripts/freeze-demo.sh` pins a demo to a commit for a post, and `scripts/try-test.py` runs a
  one-liner like a reader.

**Docs and site**

- The README, guide, architecture, plan, time-travel proposal, identity explorations and the
  preflight check; [scope.md](design/scope.md), [adopt-or-build.md](design/adopt-or-build.md),
  [state-and-theme.md](design/state-and-theme.md) and
  [prototype-notes.md](design/prototype-notes.md); 82 research dossiers on the agent UI landscape.
- fictty.com, an Astro site in the glass identity, with the blog, the guide, the landscape board,
  a demo link component and the latest demo in the hero.
- The first posts: "Why the terminal?", "Interfaces you throw away" and "Kicking the tyres", with
  graphics, Tilde as a speaker and a Tokens panel (`scripts/blog-tokens.py`).
- The `blog-post` and `ship` skills, and a release workflow for macOS and Linux binaries.

## Before v0.1.0

The prototype, built on 9 October inside Thought Control as `thc-scene`.

- The proof of concept: the whole screen as JSON, a tree of components (col, row, text, list,
  table, sparkline) bound to data sources, with state, update and view in the Elm shape;
  `run`, `push`, `patch`, `key`, `get --state`, `render` and `--watch`.
- Stream sources (one long-running command, each JSON line a new value), binds that reach into a
  source, drawing only when something changed (capped at 120 fps), lists that build only the rows
  that fit (a 50,000-row list in 0.18 ms), an fps meter, `get --stats` and a stress suite.
- The mouse (click, wheel, hover, `tip` callouts), layers on caretline-layers (callouts, arrows,
  rings, spotlights), charts, bars, gauges, big digits, tabs, a world map, `{±x}`, themes,
  `@node` binds and a live finance dashboard.
- Thought Control's Today screen rebuilt from components, identical cell for cell across 13
  states, which brought templates with `<style>` runs, value maps, conditionals, fills and
  per-kind list rows.
- Pixels through the kitty graphics protocol (cards, plots, PNG images), larger text over OSC 66,
  SVG with live placeholders, and upgrade in place.
