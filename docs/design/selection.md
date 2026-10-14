# Galleries: pictures, clicks, and choosing many

Written on 14 October 2026. Agents using the `fictty-ui` skill to make screens for people (a
folder of 47 images to caption, with a date and credit shared by many of them) asked for a
gallery: thumbnails on the left, several picked at once, a field that writes to every one picked.
Aram's call: no gallery node and no tricks. Build it from small parts, each useful on its own.
This doc covers what comes ahead of the layout steps; the rest is already in the plan.

## The parts

A gallery is a `list` with `multi`, whose row is a node template holding an `image`, laid out
in `cols`, beside editors that follow its selection. Each word is one of these parts:

| part | what it gives | where |
|---|---|---|
| clicks reach the nearest `click` | a picture inside a clickable row selects it | here, 1 |
| `image` in the window | pictures without a terminal | here, 2 |
| `multi` on lists, trees and tables | click, ⇧-click, ⌘-click, ⇧↑/↓, ⌘A | here, 3 |
| editors over a multi-selection | one field writes to every picked row | here, 4 |
| `grid` and `cols` | rows and cards in columns | layout-step.md 3c |
| `scroll: true` | a column of thumbnails longer than the window | layout-step.md 3d |
| rows as nodes | a list's row is any node (a picture, a name) | expressive.md, "Content" |

Aram chose (14 October 2026) to bring images and rows as nodes forward. 1 to 4 come first
because they're behaviour, not layout, and a screen in use today needs them. Then 3b, 3c and
3d, then rows as nodes, ahead of 3e and 3f.

## 1. A click runs the nearest `click`

A click lands on the innermost node under the pointer. If that node has no `click` (and isn't
a row, a tab or a text field), it goes to its nearest ancestor that has one. A text, image or
card inside a clickable row no longer swallows the click. This is one rule in `update`, not a
case per kind, and the terminal and the window both get it.

## 2. `image` in the window

- `src` is a template over the node's bound value (`"src": "{preview}"`, bound to `@files`), as
  `svg`'s markup is, so a picture follows a selection.
- The window draws any file GPUI decodes (PNG, JPEG, GIF, WebP, BMP, TIFF), scaled to fit
  inside the node while keeping its proportions, and centred.
- `fit: "cover"` fills the node instead, cropping. That's the only option; it maps onto GPUI's
  `object_fit`.
- A file that can't be read shows the node's `alt`, or `[image <path>]`. The readback gives the
  path an image shows as its text. (Telling the core a file didn't load, the way fonts do through
  `Drawn`, waits until a screen needs it.)

## 3. Choosing many

`"multi": true` on a `list`, `tree` or `table` lets the person pick several rows:

| input | effect |
|---|---|
| click, ↑/↓, j/k | one row: the picked set is that row; it's the anchor |
| ⌘-click | adds or removes a row; that row is the anchor |
| ⇧-click | the range from the anchor to that row |
| ⇧↑ / ⇧↓ | extends the range by a row |
| ⌘A (`d-a`, and `a-a` for terminals without ⌘) | every row |
| Esc | back to the lead row alone |

In the terminal, where a click can't carry ⌘, ⌥-click picks a row in or out.

- **State.** A node's `Local` keeps `picked` (row keys, in the order they were picked) and
  `anchor` (a key). The lead row is still `selected`/`key`, so `@files` means what it always
  did. Keys, not indexes, so the set survives rows coming and going, as the selection does.
- **Reading it.** `$picked` is a built-in source: `$picked.files` is `{"count": 3, "keys":
  [...], "rows": [...]}` (rows in list order), so `"{count} picked"`, `"when":
  "$picked.files.count>1"` and an `each` over `$picked.files.rows` need nothing new.
- **Looks.** Every picked row takes `row.selected`; the lead row also takes `row.active`.
- **Actions.** `toggle` and `remove` on a list with a multi-selection act on every picked row.
  `run` and the other templates read the lead row, as today.
- **Messages.** `Msg::Click` gains `mods` (the modifiers held). The window passes them, and so
  does `fictty mouse click X Y --mods shift,cmd`. The readback shows `(picked 3)` on the list,
  with `"picked"` in the JSON.

## 4. A field over many rows

An editor bound to a multi-selection (`"bind": "@files"`, `"doc": "date/{id}"`, with `files`
`multi`) edits the entry of every picked row:

- **All the same:** it shows that text, and every edit is made to every entry, the same
  caretline message applied to each with the same view. The entries stay equal, and each undo
  takes the step back in each.
- **Different:** it shows nothing, with the placeholder `Multiple values` (the theme's
  `input.placeholder`). The first edit that changes the text replaces every entry with what the
  field now holds: one write per entry, `by: person`. After that they're equal, as above. Moving
  the caret or selecting writes nothing.
- **Fields left alone write nothing**, so a caption that differs per image keeps its value while
  the date shared by the same images is typed once.
- With one row picked it's the editor it has always been.
- The readback shows `(3 entries)` on such an editor, or `(mixed)` when they differ.

Known limit: a mixed field can't be cleared in one go. Typing anything and deleting it does
that.

## Done when

- `tests/window.rs` covers each input above, a click through an image to its row, an image
  drawn from a template, and multi-edit's three cases (same, mixed, untouched), with undo.
- The fuzz runs clean at 400 runs in release with `multi` lists in the tour or the wringer.
- `fictty-ui`'s `images.py` makes the captioning screen in the window with `multi`, and the
  skill no longer sends anyone to a terminal for pictures.
