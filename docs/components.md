# Components

A component is a node `type` in the UI value: `{"type": "list", …}`. The set is closed (the
`Kind` enum in `crates/fictty-core/src/model.rs`), and layouts arrange its members rather than
add new ones. A component exists because it needs behaviour or speed that the runtime has to own:
a selection, a caret, a fold, a scroll offset, a clock. Behaviour belongs in the runtime;
arrangement belongs in data, so a card, a toolbar or a form row is written from the parts below,
not added to the list. Every node also takes the shared fields in the next section. This page
lists what each component is, its fields and whether the native window draws it yet. The full
reference, with every template rule and action, is [the guide](guide.md).

The window is replacing the terminal ([design/native.md](design/native.md), "Next, step by
step"). Each component below has a **Window** line: *done* (the window draws it and its tests
run there), *partial* (it draws, with what's missing named) or *terminal only* (the window shows
a `[type]` placeholder until the step named).

## Fields every node takes

| field | what it does |
|---|---|
| `id` | Stable identity: selection, folds, carets and scroll survive a push; `patch` replaces the node alone; focusable nodes need one |
| `bind` | The store entry or data source it reads (`"books.running"`, `"@notes"` for another list's selected row) |
| `when` | Shown only while a condition holds: `"dialog"`, `"!dialog"`, `"mode=edit"`, `"$width>=80"`, joined with ` & ` (all) or ` | ` (any). A hidden node takes no room or focus |
| `on` | A condition, read as `when` is, that puts the node in its `on` state (theme part `button.on`): a toggle as one node |
| `size`, `min`, `max` | Its share of its row or column: cells (`12`), `"48px"`, `"30%"`, `"*"`, `"2*"`, `"fit"`; `min` and `max` bound it in any length |
| `pad`, `margin` | Space inside and outside: one length, `[x, y]` or `[top, right, bottom, left]`, in cells, `"8px"` or a theme `space` name (`"s2"`) |
| `align` | `left`, `center` or `right` for its text, and where a button's face sits |
| `self` | Where it sits across its container, over the container's `items`: `start`, `center`, `end`, `stretch`, `baseline` |
| `aspect` | Width over height, `1.5` or `"16/9"` |
| `span` | In a `grid`, how many columns it takes |
| `scroll` | A `col`, `grid` or `each` scrolls down, a `row` across, with its offset kept in the state by id |
| `resize` | A store key (a template): a drag handle on the node's bottom edge sets its height, kept there in pixels when the drag lets go; down a column that height stands for `size`, within `min` and `max` (window only) |
| `variant`, `tone`, `style` | A named look from the theme, a meaning for its text colour (`warn`, `neg`, `accent`), and its own whole style (`fg`, `bg`, `type`, `pad`, `radius`, `border`, `shadow`, `ring`, `opacity`, `cursor` and the rest) |
| `border`, `edge`, `title` | A border, its shape (`rounded`, `double`, `thick`, `plain`); a `title` draws one with the title in it |
| `backdrop` | A card behind the node: `fill`, `fill2` (a gradient down), `border`, `radius`, `shadow`, `glow` |
| `keys`, `click` | Actions on keys while it has focus (`{"enter": …}`), and on a press |
| `multi` | A `list`, `tree` or `table` whose rows can be picked several at a time |
| `drop` | A `list`, `tree` or `table` whose rows can be dragged into a new place among its own: `{"on": <action>}` runs when one is let go over a row, reading that row (`{id}`), the row carried (`{$drag.id}`, `{$drag.keys}` for every picked row carried) and `{$drop.side}` (`before` or `after`). **Window only** |
| `float`, `modal` | In a `stack`: where it floats (`at`, `width`, `height`, `margin`, or `x` and `y` from data), and whether it is a dialog that keeps the focus over a scrim |
| `tip` | A callout (`title`, `text`) that follows the mouse over its rows. **Window:** terminal only (step 7) |
| `scale` | Text drawn larger with the terminal's text sizing (OSC 66). **Window:** terminal only; the window uses a type role (`"style": {"type": "display"}`) instead |

The UI value around the root node has its own fields: `root`, `data` (sources: `cmd`, `shell`,
`stream`, `value`, `host`, with `path` and `every`), `store` (seeds), `keys` (keys that work
everywhere), `theme`, `layers`, `focus`, `status_bar` and `title` (the window's title, a
template whose paths name store entries, sources or `@list` as binds do:
`"{project.name} · Caption Genius"`; `get --state` shows it filled in as `title`).

## Layout

Containers hold `children` (or, for `each`, a `template`) and place them. They map onto the
window's flexbox and grid layout (GPUI's Taffy).

### col

Children stacked top to bottom. Fields: `children`, `gap` (a length or `[x, y]`), `justify`
(`start`, `center`, `end`, `between`, `around`), `items` (`stretch`, `start`, `center`, `end`,
`baseline`), `wrap` (on into a column beside).

```json
{"type": "col", "gap": 1, "justify": "between", "children": [
  {"type": "text", "size": "fit", "text": "<b>Settings</>"},
  {"type": "button", "id": "save", "size": "fit", "label": "Save"}
]}
```

**Window:** done.

### row

Children side by side. The same fields as `col`; with `wrap`, those that don't fit go on to the
next line, each line as tall as its tallest.

```json
{"type": "row", "wrap": true, "gap": [1, 0], "children": [
  {"type": "text", "size": "fit", "text": "<accent>#bikes</>"},
  {"type": "text", "size": "fit", "text": "<accent>#parts</>"}
]}
```

**Window:** done.

### grid

Children in columns that line up from line to line. Fields: `children`, `cols` (a count,
`{"min": "200px", "max": 3}` for as many as fit, or a size per column `["200px", "*", "2*"]`),
`gap`, `items`; a child takes `span` columns.

```json
{"type": "grid", "id": "features", "cols": {"min": "220px", "max": 3}, "gap": 1, "children": [
  {"type": "text", "variant": "card", "text": "Push"},
  {"type": "text", "variant": "card", "text": "Patch"},
  {"type": "text", "variant": "card", "text": "Read back"}
]}
```

**Window:** done.

### stack

Children over each other, the last on top, each over the stack's own box. Sized `"fit"`, the
first that neither floats nor is a `modal` is the base instead, laid out down as a `col` holds it: the stack is as
big as it is (a card's square picture, `aspect: 1`, with badges floating over it). A child with `float` sits where it says; a `modal`
one dims what's behind it and keeps the focus. With `when`, a layer comes and goes: this is how
dialogs and pop-ups are written.

```json
{"type": "stack", "children": [
  {"type": "text", "text": "The page"},
  {"type": "col", "id": "confirm", "when": "asking", "modal": true, "title": "Delete?",
   "float": {"width": 40, "height": 6, "at": "center"},
   "children": [{"type": "button", "id": "yes", "label": "Delete", "click": {"put": "asking", "value": false}}]}
]}
```

**Window:** done.

### each

A `template` node repeated once for each item of the bound array. `{$item}`, `{$item.path}`,
`{$index}` and `{$count}` fill the template (ids, binds, sizes and floats too). Fields: `row`
(side by side), `wrap`, `cols` (a grid of instances), `stack` (instances float in a stack, placed
from data), `gap`, `justify`, `items`. The expanded tree is what's drawn and focused.

```json
{"type": "each", "bind": "events", "stack": true, "template":
  {"type": "text", "variant": "card", "text": "{$item.title}",
   "float": {"y": "{$item.start}", "height": "{$item.len}", "at": "top"}}}
```

**Window:** done.

### tabs

One child at a time, picked by a bar of tab titles (a click, or ←/→ while focused). Fields:
`tabs` (titles, templates that read the bound value and `{$store.key}`), `children` (a page per
tab), `tab` (the one shown first), `lead` and `trail` (templates at the bar's ends). Tabs
without pages are a bar of choices: a segmented control or a radio group. The theme's `tabs`
part picks the shape: `pill`, `underline`, `segment`, `plain`.

```json
{"type": "tabs", "id": "views", "tabs": ["Inbox {$store.unread}", "Done"], "trail": "Sat 11",
 "children": [{"type": "list", "id": "inbox", "bind": "inbox", "item": "{subject}"},
              {"type": "list", "id": "done", "bind": "done", "item": "{subject}"}]}
```

**Window:** done.

### rule

A line filling the node: `glyph` (default `│` when taller than wide, else `─`).

```json
{"type": "rule", "size": 1}
```

**Window:** a one-pixel line in the border colour (or its style's `fg`), across the middle,
or down it when the rule is taller than wide (in a `row`, between a rail and what's beside
it); `glyph` is the terminal's only.

## Text and content

### text

Text from a template over the bound value: `{field}`, `{a.b}`, `<accent+b>markup</>` (any theme
part or type role, `<display>…</>`), maps (`{done|true=☑;*=☐}`), `{±change}` coloured by sign,
and a tab to right-align the rest of a line. Fields: `text`, `wrap` (`false` keeps each line on
one, cut with `…`), `lines` (at most that many, the last cut).

```json
{"type": "text", "bind": "order", "lines": 2, "text": "<b>{customer}</>\t<dim>{due}</>\n{notes}"}
```

**Window:** done (type roles, fonts, the ellipsis and `keep` are the window's own; tracking is
read back but not drawn, as GPUI 0.2.2 has no letter spacing).

### button

A labelled control: Enter or Space while focused, or a press, runs its `click`. Field: `label`
(a template). With `on`, one button is both looks of a toggle. In a row template, a button's
press acts on that row. A tab in the label right-aligns what follows across the button, as in
a text (`"All images\t101"`: a sidebar entry with its count).

```json
{"type": "button", "id": "play", "size": "fit", "on": "playing",
 "label": "{$store.playing|true=❚❚ Pause;*=▶︎ Play}", "click": {"toggle": "store.playing"}}
```

**Window:** done.

### spinner

A turning glyph before `text` (a template): something is under way. It turns on the state's
clock, so it replays. A bound source that hasn't arrived shows one too.

```json
{"type": "spinner", "text": "Fetching {count} orders"}
```

**Window:** done.

## Input

Both fields edit a text entry in the store through caretline: the text and its undo history are
the store's, the carets the editor's. Editors are monospace until caretline measures through the
renderer (phase 4 of [design/native.md](design/native.md)).

### editor

A multi-line text editor. Fields: `text` (seeds the entry), `bind` (the entry, default the id),
`doc` (the entry's name as a template over the bound value: `"note/{id}"`), `placeholder`,
`syntax: "markdown"` (styled headings, lists, quotes, code, task boxes), `readonly`, `tab`
(`indent`, `indent-lines`), `tasks` (`toggle`, `cycle`), `caret` (`start`, `end`), `links`
(`wiki`, `tag`, `url`, each with `click` and `s-click`), `ordered` (`["1.", "a.", "i."]`).

```json
{"type": "editor", "id": "body", "bind": "@notes", "doc": "note/{id}", "syntax": "markdown",
 "placeholder": "Write…",
 "links": [{"kind": "wiki", "click": {"select": "{$match}", "in": "notes"}}]}
```

While its text is longer than it shows, it draws the scrollbar lists have and the readback shows
`(scroll 3/12)`, rows scrolled past and rows its text wraps to. With `"resize": "ui.caption_h"`
its bottom edge has a handle that drags it taller or shorter; the height is kept under that key,
not the row's, so it holds whichever row the editor shows.

**Window:** partial: it draws, types, selects, scrolls and styles Markdown and links, and
Daybook's box and link clicks run in the window (`tests/journal.rs`). Step 5 moves the last grid
tests (read-only, links, tasks) to the window and adds IME composition as a message in the state.

### input

A one-line field: an editor whose text never holds a line break. Enter runs the node's
`keys.enter`. Fields: `text`, `placeholder`, `accept: "number"` (typing that can't make a number
is refused, and its keys wait until the text is one), `required` (its keys wait while it's
blank).

```json
{"type": "input", "id": "qty", "accept": "number", "required": true, "placeholder": "Quantity",
 "keys": {"enter": {"put": "order.qty", "value": "{store.qty}"}}}
```

**Window:** done (step 5 moves its `accept: "number"` test to the window).

## Collections

A `list`, `tree` and `table` show the bound array (or `items`), keep a selection that follows
its row's `key` (default `id`), scroll with a scrollbar, take `multi`, and share a `filter` (a
store entry whose words every shown row must contain). A list and a tree draw each row from
`item` (a template) or `row` (a node template filled from `{$item}`, as an `each`'s is, as tall
as its `size` or `"fit"`). Their titles take `{$rows}`, how many rows they show.

### list

A selectable list. Row fields: `item`, `row`, `cols` (rows to a line: a count or
`{"min": "180px"}`, the minimum a template over the store if it likes: `"{ui.size|S=140px;*=200px}"`), `lines`, `mark` and `lead` (what starts the selected and other rows),
`selected` (its style), `row_variant` with `variants` (a template per kind of row) and `skip`
(rows that can't be selected), `keep`, `key`, `filter`, `empty`, `section` (a heading across
the width wherever the rows' `by` value changes: `{"by": "grp", "row": <node>}`, filled from the
group's first row, `{$item.$rows}` its count; headings take no selection).

```json
{"type": "list", "id": "jobs", "bind": "jobs", "title": "TO DO · {$rows}", "row":
  {"type": "col", "size": "fit", "variant": "card", "children": [
    {"type": "text", "size": "fit", "text": "<b>{$item.title}</>"},
    {"type": "text", "size": 1, "text": "<accent>#{$item.tag}</>"}]}}
```

A gallery in groups, a heading before each:

```json
{"type": "list", "id": "files", "multi": true, "bind": "images", "cols": {"min": "210px"},
 "row": {"type": "col", "size": 9, "children": [{"type": "image", "src": "{$item.file}"}]},
 "section": {"by": "grp", "row": {"type": "col", "size": "fit", "children": [
   {"type": "text", "text": "<b>{$item.grp_name}</> · {$item.$rows}"},
   {"type": "text", "text": "<dim>{$item.grp_note}</>"}]}}}
```

**Window:** done.

### tree

An outline: each row's `nest` (default `children`) holds the rows under it. Rows fold with ←/→,
Space or a press on the fold glyph. Fields: the list's row fields, plus `nest`, `open` (`false`
starts folded) and `leaf` (the glyph for a row with nothing under it). Templates read
`{$depth}`, `{$kids}` and `{$open}`; store actions reach a row where it sits (`$ptr`).

```json
{"type": "tree", "id": "outline", "bind": "outline", "item": "{text}\t<dim>{$kids}</>", "open": false}
```

**Window:** done (indent, fold glyphs and presses on them, step 4c).

### table

A selectable table. Fields: `columns`, `items`, `filter`. A column has `title`, `value` (a
template over the row) or `cell` (a node template: a chip, a badge, a button), `size`, `align`,
`heat` (`{"value": "{chg}", "min": -2, "max": 2}`, tinted from the theme's `neg` to `pos`) and
`total` (a path summed into a footer).

```json
{"type": "table", "id": "stock", "bind": "stock", "columns": [
  {"title": "Part", "value": "{name}", "size": "2*"},
  {"title": "Qty", "value": "{qty}", "align": "right", "total": "qty"},
  {"title": "Change", "value": "{±chg}", "align": "right", "heat": {"value": "{chg}", "min": -5, "max": 5}}
]}
```

**Window:** done (column widths, alignment, heat, totals and cell templates, step 4c).

## Data display

PLAN.md's "Layout and the kit" asks for these to be reviewed as they move into the window: keep
what's worth drawing, move the rest to recipes. Step 6 of [design/native.md](design/native.md)
draws the ones that stay with GPUI's `canvas`, with hits on bars and points.

### gauge

A filled bar: `value` (a template giving a number) out of `of` (a number or a template, default
1), with an optional `label` (default the percent). Its own style is the track and the theme's
`gauge.fill` part the bar, so `"size": "4px"` is a progress line.

```json
{"type": "gauge", "bind": "track", "size": "4px", "value": "{at}", "of": "{length}"}
```

**Window:** done.

### sparkline

A small line over numbers: `values`, or `field` of each bound item.

```json
{"type": "sparkline", "bind": "takings", "field": "total", "size": 3}
```

**Window:** terminal only (step 6).

### chart

Lines over shared axes. Fields: `series` (each `name`, `bind`, `field`, `color`), `labels`
spread along the x axis.

```json
{"type": "chart", "labels": ["Mon", "Wed", "Fri"], "series": [
  {"name": "Repairs", "bind": "week", "field": "repairs", "color": "accent"},
  {"name": "Sales", "bind": "week", "field": "sales", "color": "pos"}
]}
```

**Window:** terminal only (step 6).

### bars

Bars over the bound array (or `items`): `label` (a template), `value` (a path), `width` (of each
bar), `horizontal` (one row per item, the label before and the value after). Negative values
hang below a zero line.

```json
{"type": "bars", "bind": "by_day", "label": "{day}", "value": "total", "horizontal": true}
```

**Window:** terminal only (step 6).

### big

Large digits for a KPI: the filled `text` template in a three-row block font.

```json
{"type": "big", "bind": "floor", "text": "{open}"}
```

**Window:** terminal only (step 6). In the window today a `stat` or `display` type role on a
`text` does this job, as the demo's Floor does.

### plot

Lines drawn in pixels, anti-aliased with a gradient under each, where the terminal draws
images; braille where it doesn't. Fields: `series` (as a chart's), `fill` (default true),
`width` (line width as a fraction of a cell's height).

```json
{"type": "plot", "size": 8, "series": [{"name": "Load", "bind": "load", "field": "cpu"}]}
```

**Window:** terminal only (step 6).

### map

A world map with a point per bound item. Fields: `lat` and `lon` (paths), `label` (a template).

```json
{"type": "map", "bind": "sites", "lat": "lat", "lon": "lon", "label": "{name}"}
```

**Window:** terminal only (step 6).

### dot

A lit, glowing dot: `color` and `glow` (0 for none).

```json
{"type": "dot", "size": 2, "color": "#e8834f", "glow": 0.6}
```

**Window:** terminal only (step 6).

## Media

There is no `pixels` component: the terminal's pixel work is the `backdrop` field, `plot`, `dot`
and the two below, drawn through the kitty graphics protocol. In the window a `backdrop` is a
card from the resolved style.

### image

An image file, scaled into the node keeping its proportions. Fields: `src` (a template, so
`"{preview}"` bound to `@files` follows a selection), `fit` (`contain`, the default, or `cover`),
`alt` (shown where it can't be drawn). The readback's text is the file it shows.

```json
{"type": "image", "bind": "@files", "src": "{path}", "fit": "cover", "aspect": "4/3", "alt": "[no preview]"}
```

**Window:** done for PNG, JPEG, GIF, WebP, BMP and TIFF from a path. Phase 3 adds URLs, rounded
corners, decoding off the main thread and an unreadable file as a problem.

### svg

Vector art: inline markup in `svg` (with `{field}` placeholders, so it can draw live data) or an
`.svg` file in `src`, and `alt`.

```json
{"type": "svg", "size": 6, "alt": "[ring]",
 "svg": "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><circle cx='5' cy='5' r='4' fill='none' stroke='#ff2079'/></svg>"}
```

**Window:** terminal only. The wringer's `player` shows its art as a placeholder in the window;
its README puts the fix with media (phase 3). No numbered step in native.md names it yet.

## Layers and feedback

### toasts

Where toasts show, newest first, a line each, instead of the window's corner: a bar of the UI's
own. Toasts come from the `toast` action (`{"toast": "Saved", "tone": "pos", "ms": 3000}`),
`fictty toast` and failed commands; each takes the theme part `toast.<tone>`, else `toast`, and
`$toasting` is true while one shows.

```json
{"type": "toasts", "size": 1}
```

**Window:** partial: corner toasts and a `toasts` node draw and go on the clock. Step 7 settles
the node's placement and covering what's under it, and adds `enter`/`leave` motion.

### layers (on the UI, not a node)

Callouts, arrows, rings, pulses and spotlights over the screen, each anchored to a node (`on`) or
one of its rows (`row`). Fields: `title`, `text`, `bind` (live values in a callout), `arrow`,
`ring`, `pulse` (ms), `spotlight`, `place` (`below`, `above`, `right`, `left`), `width`, `look`,
`by`, `when`. Set in the UI's `layers` or with `fictty layers file.json`.

```json
{"layers": [{"on": "jobs", "row": 2, "title": "This one needs you", "text": "Parts arrive {eta}",
             "bind": "job3", "ring": true, "pulse": 1200, "by": "agent"}]}
```

**Window:** terminal only (step 7: placed with GPUI's `anchored`, spotlights as a scrim with soft
holes, pulses on the logical clock).

Dialogs and pop-ups are not components: they are a `stack` with a `float` and `modal` child (see
`stack` above).

## Roadmap: components to build next

In the order the plan takes them. Each says whether the plan has decided it or it is proposed
here. Sizes are the plan's for its step or phase; where the plan gives none, the size is an
estimate and says so.

### 1. The editor's last slice in the window

Proving in the window what the grid still guards: links, task boxes, `readonly`,
`accept: "number"`, then IME composition as a `Msg` held in `State`. Why: the grid's editor
tests are among the last that keep the terminal alive, and Daybook
([examples/journal/README.md](../examples/journal/README.md)) is used daily through them.
Decided: step 5 of [design/native.md](design/native.md). Size: S.

### 2. Charts and big type in the window

`sparkline`, `chart`, `bars`, `gauge`, `plot` (with `map` and `dot`) and `big`, drawn with
GPUI's `canvas` from the same data, with hits on bars and points. Why: they show as placeholders
in the window today, and the public demo's Floor wants takings as a sparkline and bars
([design/demo.md](design/demo.md)). PLAN.md asks for the drawing-only ones to be reviewed on the
way: keep what's worth drawing, move the rest to recipes. Decided: step 6. Size: M (the step's).

### 3. Icons (Lucide)

`icon`: a name from a bundled open set (Lucide, ISC licence), coloured and sized by the theme, so
a UI never ships SVG paths. Why: the demo wants icons on its tabs ([design/demo.md](design/demo.md)),
and rows with avatars and badges need small marks that aren't glyph tricks. Decided:
[design/expressive.md](design/expressive.md) ("Content"), placed in step 6 of native.md. Size:
part of step 6's M.

### 4. Layers, callouts and rings in the window

Callouts, arrows, rings, spotlights and pulses placed with GPUI's `anchored`; the hover `tip`;
a `toasts` node's placement and covering; `enter` and `leave` motion for floats, layers and
toasts. Why: the agent in the demo points with callouts and rings ("this one needs you"), the way
the terminal tour did ([design/demo.md](design/demo.md)); `layers` and `tip` draw only in the
terminal. Decided: step 7. Size: M.

### 5. Slider, with an `add` action

`slider`: dragging along a track sets a number in the store; an `add` store action takes a step,
a minimum and a maximum, so ←/→ move it. Why: the wringer's `settings` form fakes its slider
with a gauge that can't move, and says so ("a slider: ←/→ can't add 5 without the host",
`examples/window/screens.py`); [design/expressive.md](design/expressive.md) lists "a key can't
change a number" among what couldn't be done. Decided: a new primitive, justified by behaviour,
in phase 2 ([design/expressive.md](design/expressive.md), "Numbers and controls";
[design/building-blocks.md](design/building-blocks.md)). Size: part of phase 2 (M).

### 6. Checkbox and switch, as recipes

A checkbox or switch written from what exists: a `button` with `on` and `{"toggle": "store.x"}`
for a flag, or a list row with `toggle` for a field of each row, its look from the theme's
`button.on` part or a variant. Why: the wringer's settings form still draws its checkboxes as
list rows that toggle, a trick its comment names. Decided: a recipe, not a primitive
([design/expressive.md](design/expressive.md): "Checkbox, radio and select become project
recipes on `toggle`, `put` and a float"); the radio group is already tabs without pages. What's
left is writing the recipe down and using it in `settings`. Size: S (an estimate).

### 7. Select and dropdown, as a recipe

A field that opens a floating list over a `stack` (`float`, `when`), whose `click` `put`s the
chosen value and closes it. Why: the wringer's select is a pop-up list built by hand, named as a
trick in `screens.py` and in [design/building-blocks.md](design/building-blocks.md). Decided: a
recipe, as above. Size: S (an estimate). If writing it shows behaviour composition can't give
(typing to jump, the pop-up placed against its field), that is the case for a primitive.

### 8. Drag and drop, and drop zones

**Started (19 October 2026):** rows dragged within one list, tree or table, the `drop` field
above, for Caption Genius's gallery (a Folio app). A press that moves 4 pixels lifts the row
(or every picked row); `$drag` is in the state and the readback (`dragging 2 → before 5`); the
window dims what's carried (`drag.ghost`) and draws a bar where it would land (`drop.hover`),
across a grid of rows by halves left and right; Esc, or letting go off the rows, drops nothing;
an agent drags with `fictty drag <id> <row> <to> [--before]`. Files from Finder (`drop.files`,
20 October); the bar's place number (`drop.index`) and `drop.optimistic` (21 October: a drop
shows at once while its `run` goes, placeholder rows for files in `row.pending`, until the app's
rows change or the run ends). The rest below is still to come.

`drag` (what a node carries), `drop` (`accept` and the action on a drop), `$drag` in the state,
a `move` store action between arrays, theme parts `drag.ghost` and `drop.hover`; files dropped
from the system through the same mechanism (`{$file.path}`, `{$file.name}`). Why: the kanban's
cards can't move between columns by mouse; phase 2's receipts walkthrough needs a drop zone
an agent hears about; the demo wants the agent and the person to move a card by drag. Decided:
phase 2 ([design/native.md](design/native.md), [design/expressive.md](design/expressive.md)).
Size: part of phase 2 (M).

### 9. More `accept` kinds for form fields

An `input` that takes a date, or one of a list, beside `number`. `required` has landed. Why:
forms today check only numbers and blanks; Daybook found that dates are a host's or an agent's
job ("Date arithmetic is a host's, or an agent's",
[examples/journal/README.md](../examples/journal/README.md)). Decided as a want, unscheduled:
PLAN.md, "Records" ("More `accept` kinds where a form needs them: a date, one of a list,
required"). Size: S (an estimate).

### 10. Avatars, badges and chips, as recipes

Written from row templates, `cell` templates, `variant` and (once it lands) `icon`: an avatar
as an `image` with an `aspect` of 1 and a round variant, a badge as a `fit` text in a variant, a
chip as a `fit` button. Why: the demo's inbox rows want avatars and badges and an unread dot
([design/demo.md](design/demo.md), [design/building-blocks.md](design/building-blocks.md)
step 2); rows-step.md already names a chip and a badge as what a column's `cell` holds. Proposed
here: the plan names the screens, not these as components. Size: S (an estimate).

### 11. Project components (`{"use": …}`)

A component written as JSON with named parameters, loaded from the project:
`{"use": "kpi", "label": "Open jobs", "bind": "floor.open"}`. Why: it is where recipes (the
checkbox, the select, a stat card) live once they are written down, so agents stop copying the
same nodes between screens; it keeps the closed set closed. Decided as a want, unscheduled:
PLAN.md, "Layout and the kit". Size: M (an estimate).

### 12. Markdown rendered, code with syntax colour, a diff view

Markdown shown as a document (real headings, lists, links), not only styled while edited; code
coloured by its syntax; a diff of two texts. Why: agents hand people plans, code and changes to
read, and today those are a `readonly` editor with Markdown styling. Real heading sizes wait on
proportional type (phase 4). Decided as a want, unscheduled: PLAN.md, "Layout and the kit".
Size: M (an estimate).

### 13. An embedded terminal

A node that shows a live command's output, colours and all, drawn in the window. Why: once the
terminal goes, an agent's long-running command has nowhere to show but a log editor. Decided as
a want, unscheduled: PLAN.md, "Layout and the kit". Size: L (an estimate: a pty and an emulator
in the window).

### 14. Video and the rest of media

`video` (a path or URL, hardware-decoded, its play, pause, seek and position in `State`),
`image` from URLs with rounded corners and decoding off the main thread, pasted and dropped
images, and `svg` drawn in the window. Why: the wringer's `player` art is a placeholder in the
window; phase 3's test of done is a video in one tab and a gallery in another. Decided: phase 3
of [design/native.md](design/native.md) (video, images); the window's `svg` is named by the
wringer's README, not by a step. Size: M (phase 3's).

### 15. A catalogue and a widget registry

A catalogue: what this screen can render (the components, project components, registered
widgets), so an agent can ask. A widget registry: an element an embedding app registers, the
escape hatch for what the kit can't do. Why: an agent writing a screen needs to know what's
there; Thought Control may find something no composition gives. Decided as wants: PLAN.md,
"Layout and the kit"; the registry "waits until Thought Control finds something the kit can't
do". Size: S for the catalogue, unsized for the registry (estimates).

### 16. Time travel

Not a component but a way to read any of them back in time: record inputs, snapshot sparsely,
then `history`, `state_at`, `screen_at`, `diff`, `seek`, `fork`, `replay`. Why: an agent can
address a live UI's history, which nothing else offers; `update` and `view` are already pure, so
it is cheap. Proposed: [design/time-travel.md](design/time-travel.md), carried in PLAN.md's
"History" without a phase. Size: L (an estimate).

## How a new component gets in

The rule is AGENTS.md's: a new primitive is justified only by behaviour or speed that
composition can't give. A slider earns one because dragging a number along a track is state the
runtime has to own; a checkbox doesn't, because `on`, `toggle` and a theme part already make one.
Anything that only arranges or styles other components is a project component or a recipe, not
Rust. Special cases are a smell: a branch for one UI means the model is wrong.

A component that gets in is data like every other: a `type` and fields in `model.rs`, its state
in `State` (never a hidden field or thread), its time from `State::now_ms`, and never anything
that lets a UI value run logic. It must be visible to agents: in `fictty screen --nodes` (its
kind, id, text, rectangle and resolved style) and in `fictty get --state`. It lands in the window
first, with tests in `crates/fictty/tests/window.rs`, and the terminal never learns it.
