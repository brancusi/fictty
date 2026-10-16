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

The quickest way to see everything is the showcase. It opens on a tour of ten chapters, each a
lesson, something to play with and a quest: a screen and its JSON side by side, every component,
the store with live revisions, a dialog over a stack, themes, an autopilot driving the screen
while you watch, a notebook of linked pages with a sidebar, and a firehose beside 100,000 rows that filter on every key. The apps (notes, a
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
fictty quit                                               # end it, wherever the focus is
fictty run --watch my.json                                # hot reload while you edit
fictty screen --nodes                                     # the shown nodes: kind, id, focus, text, where
fictty screen --nodes --style                             # and each one's style under the theme
fictty run --window my.json                               # in a native window (macOS, early)
fictty render examples/tasks.json --size 100x30 --keys j  # no terminal
fictty read ledger                                        # one store entry: rev, by, value or text
fictty put ledger rows.json --if-rev 12                   # write it, unless someone wrote since 12
fictty delete ledger                                      # the entry goes (a seed can start it again)
fictty theme paper                                        # choose a loaded theme
```

## The model

- **Components** (a closed set): layout `col`, `row` (both with a `gap`) and `stack` (children
  over each other); `text`, `big`, `spinner`; `list`, `tree`, `table` (with column `total`s);
  `editor`, `input`, `button`; `tabs`; `sparkline`, `chart`, `bars` (up, or `horizontal`), `gauge`;
  and the pixel ones below. Every node can have `id`, `size` (`12` cells, `"48px"`, `"30%"`, `"*"`,
  `"2*"`, `"fit"`; a terminal rounds pixels to its cells) with `min` and `max`, `title`, `border`,
  `pad`, `margin`, `self`, `aspect`, `align`, `variant`, `tone`, `style`, `bind`, `keys`, `click`, `when`,
  and in a stack `float` and `modal`.
- **Lengths**: `pad`, `margin`, `gap`, `min` and `max` take cells (`1`), pixels (`"8px"`) or a
  name from the theme's `space` (`"s3"`), the same lengths a style's `pad` takes. `pad` (space
  inside, over its style's) and `margin` (space outside, between a node and what's beside it)
  are one length for every side, `[x, y]` (across, then down), or `[top, right, bottom, left]`,
  as CSS: `"pad": [1, 0]` is a cell either side. A `gap` is one length or `[x, y]`. A length
  that isn't one, or names a `space` the theme doesn't have, is a problem (`get --state`'s
  `problems`, the status line). The window takes every unit; a terminal takes cells only.
- **Alignment** (the window's): a `col`, `row` or `each` puts its children along it with
  `justify`, in the room they leave: `start` (the default), `center`, `end`, `between` (the room
  between them) or `around` (around each). It puts them across with `items`: `stretch` (the
  default, as wide as a column or as tall as a row), `start`, `center`, `end` or `baseline`. A
  node's own `self` takes the same words and overrides its container's `items` for it. A node
  that isn't stretched is as big across as its content. Only children with a fixed `size` leave
  room along a container (a share takes it all): `{"type": "col", "justify": "center", "items":
  "center", "children": [{"type": "text", "size": 2, "max": 40, …}]}` sits in the middle both
  ways (or `"size": "fit"`, below). An unknown word fails to parse, naming the ones it takes.
  All read back as written.
- **Fit** (the window's): `"size": "fit"` makes a node as big as what it shows along its row or
  column: a text as wide as its line (or as tall as it wraps to, in a column), a button as wide
  as its face, a box as wide as its title, a list as tall as its rows, an editor as tall as its
  lines at its width, growing as it's typed in. `max` holds it (a long `fit` list scrolls past
  its `max`), and where there isn't room it gives way, the others with it, as a fixed size
  does. Shares take what's left, so `[{"size": "fit"}, {}]` is a label and the rest. A cut
  line in a `fit` node only says how wide it would be inside one, so it costs nothing
  elsewhere.
- **Aspect** (the window's): `"aspect": "16/9"` (or a number, width over height) keeps a node's
  shape. It takes the side it's given and works out the other: its `size` along its row or
  column, else all the room across, less its `margin` (a picture as wide as its column, as tall
  as that makes it). A node that has neither, one not stretched across and with no fixed
  `size`, has nothing to work from, so give it one. Anything but a number above nothing or `"w/h"`
  fails to parse.
- **Lists and trees** share their row options: `item` (a template), `row_variant` and `variants`
  (a template per kind of row) with `skip` (rows that can't be selected), `mark` and `lead`,
  `selected` (a style), `lines` (rows of several lines), `keep` (cells the left side keeps before
  the right side, after a tab, is cut), `key` (default `id`: the selection follows its row when
  rows come and go) and `filter` (a store entry whose words every shown row must contain). A
  `tree` nests rows under each row's `nest` (default `children`), folds with ←/→ and space, keeps
  its folds by key, and gives templates `{$depth}`, `{$kids}` and `{$open}`; its `leaf` is the
  glyph for rows with nothing under them. A shown row carries `$ptr`, where it sits in the bound
  entry, so store actions reach it through a filter or deep in a tree. **`"multi": true`** (a
  list, tree or table) lets the person pick several rows: ⌘-click (⌥-click in a terminal) picks
  one in or out, ⇧-click picks the range from the row last clicked, ⇧↑/⇧↓ (`J`/`K`) grow it,
  ⌘A (⌥A) takes every row, Esc and any plain move or click go back to one. The selected row is
  still the one `@id` reads; every picked row is drawn `row.selected` and the selected one also
  `row.active`. `$picked.<id>` is `{"count", "keys", "rows", "index", "total"}` (`index` the selected
  row's place from 1, `total` the rows: `"{index} of {total}"`), `toggle` and `remove` act on every
  picked row (all take the selected row's new value), and the readback shows `(picked 3)`
  (`"picked"` in the JSON). `fictty mouse s-click X Y` and `d-click` press with ⇧ or ⌘.
  An **editor over picked rows** (`"bind": "@files", "doc": "date/{id}"`, `files` multi) edits
  every picked row's entry: while they're the same it shows the text and makes each edit in
  each (so undo steps back in each); while they differ it shows nothing, with the placeholder
  `Multiple values`, and its first edit puts what it then holds into every one. A field left
  alone writes nothing, so a caption that differs per row keeps its own while a date is typed
  once for all. The readback shows `(3 entries)` or `(3 entries, mixed)`; the mixed field's own
  text is the entry `$mixed/<id>`, never saved. ⌘Z on a mixed field steps back in each entry's
  own history.
- **Rows as nodes**: a list's `row` is a node drawn for each row instead of `item`, its
  `{$item}` and `{$item.path}` filled from the row as an `each` template's are (a picture over
  a name: `{"type": "col", "size": 8, "children": [{"type": "image", "src": "{$item.thumb}"},
  {"type": "text", "size": 1, "text": "{$item.name}"}]}`). It's as tall as its `size` in
  cells. A press anywhere in it is a press on the row, so selection, `multi` picking, keys and
  scrolling are the list's as ever; its nodes take no ids or clicks of their own. `cols: 3`
  lays rows three to a line: a grid, where ↑/↓ move a line and ←/→ a row, and the view scrolls
  by whole lines. The readback's text for such a row is what its nodes show (`/thumbs/a.png ·
  a name`). The window draws them; a terminal shows the row's value.
- **Repeating**: `{"type": "each", "bind": "open", "template": {…}}` draws the template once per
  item of the bound array, top to bottom (`"row": true` side by side). In the template,
  `{$item}` and `{$item.path}` are filled from the item (`"id": "panel-{$item.id}"`,
  `"doc": "page/{$item.ref}"`, `"size": "{$item.h}"`); the template's own placeholders are left
  for it. The expanded tree is what's drawn and focused (`tree` in `get --state`); a push or a
  patch changes the template. A `State` read back from `get --state` leaves the tree out (it's
  worked out, not kept): call `derive()` on it before `focus_order` or `tree()`.
- **Overlays**: a `stack` draws its children over each other, the last on top; one with
  `float` (`width` and `height` in cells, pixels or `%`, `at` an edge or corner, `margin` in
  from the edges, as a node's) sits
  where it says, and a `modal` one dims what's behind it and keeps the focus: while it shows, a
  click or the wheel anywhere outside it goes nowhere. Both cover what's under them. **`when`**
  shows a node only while a condition holds: a bind that's set (`"dialog"`), not set
  (`"!dialog"`), compared (`"mode=edit"`, `"$screen.width>=120"`), or several joined by ` & `.
  Its `{field}`s are filled from the node's bound value first (`"bind": "@files", "when":
  "remove/{id}"` reads the selected row's flag). `$width` and `$height`
  are the size of the area the node's parent lays it out in (a container query): a pane beside a
  sidebar switches on its own width, not the screen's. A hidden node takes no
  room and no focus. Layers take `when` too. **Toasts** show for a while: the `toast` action,
  `fictty toast`, and failed commands and host actions. They sit in the corner, or wherever a
  `{"type": "toasts"}` node is (a line each, in a UI's own bar), styled by the theme part
  `toast.<tone>`, else `toast`; while one shows, a `toasts` node covers its area, and
  `$toasting` is true (for a `when` on what's under it).
- **Sources** in `data`: `cmd` (argv), `shell`, `stream` (a command printing JSON lines), a
  literal `value`, or `host` (the embedding app's, with `args` that are templates over the state,
  so `{"host": "detail", "args": {"id": "{@list.id}"}}` refetches as the selection moves); `path`
  narrows the JSON, `every` re-fetches. A push fetches the sources that are new or changed; one
  that's the same keeps its value, stream and timer (`refresh` fetches them all). Built
  in: `$stats`, `$caps`, `$theme`, `$screen` (the terminal's `width` and `height`), `$keys` (a
  chord's keys so far), `$picked` (each `multi` node's picked rows), `$store` (every entry, by revision) and `$log` (what agents did over the
  socket, newest first, at the local clock time; a write that was refused shows as `refused`,
  with why). `$self` says where the screen runs: `bin`, `socket`, `dir` (its UI's folder), `cli`
  (a command line that reaches this screen from any terminal) and `env` (a line that sets a
  terminal up so `fictty`, and the scripts beside the UI, reach it).
- **Binds and templates**: `bind` names a store entry or a source, then a path (`books.running`;
  a store key may hold dots, `remove/photo.tif`: the longest key the bind starts with is the
  entry);
  `@id` is the selected row of another list. Text and columns are templates over the bound
  value: `"{text} · {due}"`, `<accent+b>styled</>` (or any theme part: `<link>`), `<<` for a
  literal `<`, `{±change}` coloured by sign, `{name:12}` fitted to 12 cells (`{n:>6}`
  right-aligned), maps (`{done|true=☑;*=☐}`), and a tab right-aligns the rest of a line.
- **Actions** on keys: `quit`, `up`, `down` (a line, in a list with `cols`), `next_row` and
  `prev_row` (one row whatever the `cols`, on across line ends), `top`, `bottom`, `focus_next`, `focus_prev`,
  `refresh`, `{"run": [argv with {field} of the selected row]}`, `{"load": "{file}"}`,
  `{"focus": "id"}`, `{"select": "{store.k}", "in": "list"}` (a row by its key, now or once it
  arrives), `{"theme": "next"}`, `{"copy": "text"}` (onto the system clipboard, through
  the terminal's OSC 52), `{"toast": "Saved", "tone": "pos"}`, `{"host": "name", "args": {…}}`,
  and store actions: `append`, `put`, `remove` (the selected row, or with `where` every row that
  matches), `toggle` (a true/false field of the selected row: a checkbox; `"store.zen"` flips a store
  entry instead, and `"store.remove/{id}"` an entry named from each picked row, all taking the
  selected row's new value, so a flag lives per row the way an editor's `doc` does; a row
  template shows it with `"when": "remove/{$item.id}"`), `do`. An action's templates read the selected row's fields, `{store.key}` and
  the UI's sources by name (`{today.id}`). A node's `click`
  runs an action too, after the click selects the row; a `button` runs its click on Enter and
  Space. A focused node's keys win over the UI's, which win over the defaults; a binding of
  several keys (`"p h"`) is a chord. While fictty has the mouse, hold Shift (Option in some macOS
  terminals) to select text the usual way.
- **Focus**: Tab and Shift-Tab move it, a UI's `focus` says where it starts when pushed, and Esc
  leaves a text field (dropping a selection first). Esc drops a selection first, even where a node binds Esc. `{"focus": …}` takes an id, a template
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
  editor's `text`); a seed never overwrites. A bind reads the store before the sources, so an
  entry named like a source hides it, which is a problem until `fictty delete <name>` takes
  the entry away (a UI that moved its rows from the store to a source, say).
- **Writes**: typing in an editor or input; store actions on keys; and from agents,
  `fictty put` (a value, or a string as a text's new contents), `merge` (RFC 7396) and `edit`
  (`[[from, to, "text"]]` ranges of a text). With `--if-rev N` a write is refused if anyone wrote
  since revision N, and the reply carries the entry as it is now.
- **Writes that last a while**: `for_ms` on a write (`fictty put … --for-ms 3000`, `merge`, the
  `put` action, `Notifier::write_for`) takes it away again on the screen's own clock: a put's
  entry, or the keys a merge set. A row marked live for three seconds, or a `when` that holds
  only while an undo is on offer; the expiry is in the state, so it replays.
- **An agent's text edit is an edit**: the person's carets move with the text, and one undo takes
  it back.
- **`fictty run --store file.json`** keeps it across runs, written atomically after each change.
  One running screen holds a store file at a time (a lock the system lets go of however the
  screen ends, on a file in the temporary directory named from the store's full path, so
  nothing is written beside a store in a synced folder); a second refuses to start and says
  which pid has it.
- **Editors**: `{"type": "editor", "bind": "@notes", "doc": "note/{id}", "text": "# {title}"}` edits
  the entry its template names from the selected note, so a list and an editor make a notes app.
  An editor takes every key but Tab, Shift-Tab and Esc, and the ⌃, ⌥ and ⌘ keys it has no use
  for, which go on to the UI's keys (⌘T while writing); `input` is the one-line version, and
  Enter runs its `keys.enter`; with `"required": true` its keys wait while it's blank. `"accept": "number"` makes an input refuse typing that can't make
  a number, and its keys wait until the text is a whole one (the status says why).
  `"syntax": "markdown"` styles headings, list markers, quotes, inline code and task items
  (theme parts `md.heading`, `md.bullet`, `md.quote`, `md.code`, `md.task`, `md.done`) while the
  text stays plain. Ctrl-T makes the lines it's on tasks (`text` → `- [ ] text`, `- item` →
  `- [ ] item`) and ticks open ones; a ticked one is unticked, or with `"tasks": "cycle"` made
  plain text again. A click on a `[ ]` ticks it and leaves the caret where it was. `"links"` makes spans act on a click: `{"kind": "wiki" | "tag" | "url", "click": action,
  "s-click": action}`, with `{$match}` the page, tag or address; a key reads the link at the
  caret as `{$link}`; links take the theme part `md.link`.
  `"caret": "end"` starts the caret at the end of an entry the editor shows for the first time.
  `"ordered": ["1.", "a.", "i."]` keeps ordered lists in number after every edit (numbers,
  letters, roman), each list in its own style; an item moved to a level of its own by Tab takes
  that depth's style, and `{"list_style": "next"}` changes one list's style (number, letter,
  roman, bullet). Enter in a list item starts the next one (`- `, `2. `, an open
  `[ ] ` after a task, at the same indent, the rest of the line going with it); Enter on an empty
  item outdents it, or ends the list. `"readonly": true` shows a text to
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
- **The wheel** scrolls a list, tree or table three lines a notch and leaves the selection where
  it is; in the window a trackpad moves it as far as the fingers do, part of a row at a time
  (the view's start is `top` in lines, fractions too). A key that moves the selection out of
  view brings the view back to it. While a list has more than it shows, the window draws a
  scrollbar (the `scrollbar` and `scrollbar.thumb` parts; `shape: "none"` hides it), and the
  readback shows `(scroll 6/16)`, lines scrolled past and lines in all.

## Themes

`fictty run --themes themes.json` loads named themes; the first is chosen, `{"theme": "next"}` on
a key cycles, `fictty theme <name>` picks one. A push doesn't change them. A theme has `tokens`
(colours), `parts` (how the runtime draws `table.header`, `row.selected`, `border.focused`,
`title`, `editor.caret`, `input.placeholder`, `status` and the rest, as style specs like
`accent+b`) and `variants` (named looks: a style, an edge, a backdrop, a title style). Nodes ask
for a `variant` and a `tone`, never a colour. A UI's own `theme` map still works, as tokens under
every theme. `examples/showcase/themes.json` has three: glass, paper and phosphor. In the window, a
token `edge` sets the border of every node that doesn't name its own: anything but `rounded`
squares the corners of panels, buttons, fields, tabs, rows and toasts, and `thick` or `double`
draws them heavier. `examples/window/themes.json` has six more for the window, written by
`themes.py` and worn by the wringer's nine screens and the demo (`examples/window/open.sh`). They
share one vocabulary, so a screen can name it and wear any of them: type roles `display` and
`stat` (sized in pixels), `heading`, `label`, `body`, `caption`, `num`, `mono`; scales `s1`–`s6`,
`r1`–`r3`, `e1`–`e2`; variants `card`, `quiet`, `hero`, `alert`, `bare`. The `screen` part is the
window's own (`{"type": "body"}` gives everything the theme's body font).

A part is a short ink (`"accent+b"`) or a whole style, an object; so is a node's own `style` and a
variant's `style`:

```json
"space":  {"s2": "8px", "s3": "14px"},
"radius": {"r2": 6},
"depth":  {"e1": [{"y": 1, "blur": 2, "color": "black", "alpha": 0.3}]},
"type":   {"label": {"font": "ui", "size": 13, "weight": 600, "tracking": 0.2}},
"fonts":  {"ui": "Inter"},
"parts": {
  "button":         {"fg": "fg", "bg": "panel", "type": "label", "pad": ["s3", "s2"], "radius": "r2",
                     "border": {"color": "border"}, "shadow": "e1", "cursor": "pointer"},
  "button.focused": {"bg": {"from": "accent", "to": "glow", "angle": 160}, "ring": {"width": 2, "offset": 2}},
  "tabs":           {"shape": "pill"}
}
```

A style's fields: `fg`; `bg` (a colour or a gradient `{"from", "to", "angle"}`); `bold`, `dim`,
`italic`, `underline`, `strike`, `reverse`; `type` (a role from the theme's `type`); `pad` (one
length for every side, `[x, y]` or `[top, right, bottom, left]`: cells, `"8px"` or a `space`
name); `radius` (pixels or a
`radius` name); `border` (`width`, `color`, `style`: `solid`, `dashed`, `double`, or in short
`"2px dashed accent"`, `"accent"`); `shadow` (a
`depth` name, or `none`); `ring` (`width`, `offset`, `color`); `shape` (tabs `underline`, `pill`,
`segment`, `plain`; a scrollbar `thin`, `overlay`, `none`; an edge `rounded`, `square`, `double`,
`thick`); `opacity`; `cursor` (`default`, `pointer`, `text`, `grab`). A style never arranges: no
size, gap or position. The scales let a theme change its whole feel in a few lines; `fonts`
names families or font files that type roles use. The window draws all of it.

**Type.** A theme's `type` names roles: `{"font": "serif", "size": 44, "weight": 700, "line":
54, "tracking": -1, "tabular": true}`. `font` is a `fonts` name or a family; a `fonts` entry is
a family (`"Georgia"`) or a font file beside the themes (`"fonts/Anton.ttf"`), loaded the first
time it's needed; a file that doesn't load is a problem, and its text falls back to the default. A node asks for a role through its style, `"style": {"type": "display"}`
(a node's own `type` is its kind), and markup asks with `<display>…</>`: a line takes the first
role it names. `tabular` lines numbers up; `tracking` reads back but isn't drawn yet. Text
wraps; `"wrap": false` keeps each line on one, and `"lines": 2` shows at most two. A line that
doesn't fit ends in `…`: a row keeps the part after its tab (a time, a count) whole while it
can, and `keep` (cells) holds room for the left before the right is cut too. The readback marks
a node whose text was cut, `(cut)` in the outline and `"cut": true` in the JSON.

**Cards and the status line.** A `backdrop` (`fill`, `fill2` for a gradient down, `border`,
`radius` and `shadow` as fractions of a cell's height, `glow` for a lit edge) draws as a card in
the window. A row or column clips what it holds, so give each card a `margin` to leave its
shadow room. The status line at the bottom shows the status, else the first problem, else
hints; `"status_bar": false` gives the UI every row, and `"status_bar": "speed"` adds the
speed meter (`$stats` has the numbers either way). Its look is the `status` and `status.error`
parts.

Every component's look comes from parts, including the window's own: when a theme doesn't say,
default parts do (`button` has a 12px pad and a 6px radius, a bordered node is a `panel` with
the `panel` colour and 8px of pad, `row.hover`, `row.selected`, `tabs.selected`, `input`,
`toast`…), and `fictty screen --nodes --style` shows them. A theme's part of the same name owns
its ink and keeps the default's box unless it says otherwise. Rows are `row` with the states
that hold for that row; tabs are `tabs.selected` and `tabs.idle`, in the shape the `tabs` part
gives (`pill`, `underline`, `segment`, `plain`).

A node's style is worked out in one place, and nothing else applies: its part (the component's
`type`, so `button`; containers have none), with the parts for the states that hold layered over
it, then its variant, then its `tone`, then its own `style`. States are suffixes, layered in this
order, every combination fewer before more: `.hover`, `.focused`, `.active`, `.selected`,
`.disabled`, `.blur` (`row.selected.blur` is `row`, `row.selected`, `row.blur`, then itself).
Today `hover`, `focused` and `blur` (focusable, without the focus) hold for a node; the rest come
with the state that says so. Its border and title take `border` and `title` with the same states.
`fictty screen --nodes --style` (or `render --nodes --style`) prints it under each node, with
tokens as colours (`┊ button.focused · #0f1418+on-#5fd1c6+b`); the JSON has it as `style`.
Each part, the variant and the node's own style lay over the last: what one says replaces, the
rest stays. Wrong names are `problems`, on the status line and in the reply to a push: a part no
component has (`button.focsed`; a name outside the components' families, like `link`, is the
theme's own, for markup), a word in a spec that means nothing, a colour that's no token, a
`space`, `radius`, `depth` or `type` name the theme doesn't have, a shape or cursor that isn't
one, a field no style has (`colour`), a row variant a list skips but doesn't have.

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
  the status line too), and the host sources fetch again after it. With `"wait": true`, keys
  typed meanwhile wait (in the state, as `held`) until the action has answered and those sources
  have landed, then go in order: for an action that closes or opens what the next key acts on.
- **`Notifier`**: `Host::start` gets one, for any thread. `changed("today")` refetches every source
  that reads `today` (the app's world changed; no polling), `write(key, write, by)` writes the
  store, `focus(id)` moves the focus without running a click, and `msg` sends any message.
- **Everything the host says arrives as a message**, so `get --state`, `$log` and replays see it;
  the view never reads the host.
- **Tests without a terminal**: `App::new(ui).host(h).headless(80, 24)?` gives a screen to send
  `keys(&["j", "x"])`, the mouse (`mouse("press", x, y)`, also `s-press`, `drag`, `release`,
  `move`, `wheel-up`, `wheel-down`), `paste(text)`, `resize(w, h)` and messages to, and to read
  back as text (`screen()`, `ansi()`), as nodes (`readback()`: each shown node's kind, id, text,
  focus, selection and rectangle) or state (`state()`). Sources are fetched and actions done
  inline. `App::commands(false)` stops `run` actions and command sources (each fails with
  "commands are off"), so a test never starts a script. See `tests/host.rs`.
- **Fuzzing** (the `fuzz` feature): `fictty::fuzz::run(make, &mut checks, &config)` plays random
  keys, clicks, drags, the wheel, pastes, resizes and waits on fresh headless screens, checking
  after every act. Built in: nothing panics, each act stays inside a time budget, the same acts
  replay to the same state and frame, and the frame drawn as a running screen draws it (escapes
  from ratatui's crossterm backend, diffed frame to frame, into a vt100 emulator) shows exactly
  the frame. `fuzz::editing()` adds the editing invariants (a plain motion leaves no selection,
  copy changes nothing, cut then paste gives the text back, undoing everything gets back to the
  start); an app adds its own with `fuzz::check(name, |step| …)`. A failure shrinks to the
  fewest acts and prints them with the focused editor in the editing notation
  (`Hello ⟦wor▮⟧ld`), and `fuzz::replay(make, "key d-a; paste \"x\"", …)` runs a printed one
  again. **Host acts** are what happens from outside the person, an agent's write say:
  `Config { host_acts: vec!["write".into()], host: 5, .. }` mixes in `Act::Host("write")`
  (printed `host "write"`), which runs the handler the screen was given
  (`headless(…).on_host_act(|s, arg| …)`, `Sim::on_host_act` for a pty) inside the act, so the
  look after it has what it did. `Config::normalize` masks what's meant to differ before two runs
  of a sequence are compared (a random id), and `Config::class` (from the property and message)
  keeps shrinking to the failure it started with. See `tests/fuzz.rs`, which fuzzes every chapter
  of the tour.
- **A real terminal, without one** (the `sim` feature): `fictty::sim::Sim::spawn(&["fictty",
  "run", "ui.json"], 80, 24)` runs any fictty screen in a pseudo-terminal with a vt100 emulator on
  the other end. Keys go in as the bytes a terminal sends (`keys(&["d-z"])` is `ESC[122;9u` once
  the screen has pushed the kitty keyboard flags; legacy escapes in a terminal that doesn't speak
  the protocol), so crossterm's own decoder reads them. The mouse goes in as SGR reports, pastes
  bracketed, and `resize` changes the pty's size. Each step waits until the screen is still and
  has answered on its socket. `check()` says whether the terminal shows exactly the frame;
  `state()` and `request()` reach the socket. `FICTTY_SIM_TRACE=1` prints every byte each way.
  `tests/sim.rs` plays random acts on a headless screen and through the pty, and checks that both
  end the same.
- **Store entries the host keeps**: `Host::keys` names prefixes (`page/`). Such an entry is never
  seeded: when something first shows it the screen asks `Host::load` (a spinner meanwhile, the
  reason if it fails), and once it has changed and been still for half a second it goes back
  through `Host::save`. `Notifier::reload(key)` loads it again when it changed at the host; text
  comes in as an edit, so the person's carets and undo carry on. Typing not saved yet is saved
  first, so the host can merge it with its change before the load. A host whose `save` merges
  (the person's text with an agent's line written meanwhile) calls `reload` when what it keeps
  differs from what it was given, or the editor never sees the merge. The `--store` file leaves
  these entries out.
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
  from it. `click` on a node runs an action; a click on a node without one (a picture in a card) runs
  the nearest `click` around it. The view returns each frame's hit regions; the
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
3. **`image` for real bitmaps** (photos, screenshots), with a path. `src` is a template over the
   node's bound value (`"bind": "@files", "src": "{preview}"` follows a selection), `fit:
   "cover"` fills the node (cropping) instead of fitting inside it, and `alt` shows where it
   can't be drawn. The window draws PNG, JPEG, GIF, WebP, BMP and TIFF; a terminal reads PNG
   files itself. The readback's text for an image is the file it shows.
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
