# Daybook: a journal written as data

A page a day, pages for everything else, tasks in the text, and a zen mode that leaves only the
writing. It follows Thought Control's journal (the keys, the `§`/`¶` headings, tasks that cycle,
Tab that nests), but where thc builds its screen in Rust behind a host, Daybook is one JSON value
with no host: `journal.py` writes `journal.json`. It's here to use fictty the way a daily app
does, and to catch what that finds. The entries are made up.

```sh
mise exec -- cargo build --release
examples/journal/open.sh                                   # in paper
examples/journal/open.sh glass                             # another theme
DAYBOOK_STORE=~/daybook.json examples/journal/open.sh      # keep what you write
fictty --socket /tmp/fjournal.sock screen --nodes          # read it back while you use it
```

## The flows

| key | what it does |
|---|---|
| ⌘T | today's page, from anywhere, even mid-sentence: added to the journal once, selected, the caret where you left it |
| ⌥[ ⌥] | the day before and after the open one (the journal's rows) |
| ⌘N | a new page: a name, then Enter (a blank name waits); Esc goes back |
| ⌘1 ⌘2 | the journal, the pages; Enter opens a row and goes straight to writing |
| ⌥Z (or ⌘.) | zen: the sidebar, heading and hints go, the text stays centred with the caret where it was |
| ⌃T | a line becomes a task, is ticked, then is plain again; the caret stays on the same letter |
| Enter, Tab | Enter after a task starts the next one; Tab nests it; Enter on an empty one outdents, then ends the list |
| a click | on a box ticks it, leaving the caret where you were writing; on a `[[link]]` opens that page, adding it to the list |
| Esc | leaves the text for the lists |

`crates/fictty/tests/journal.rs` runs each flow in the window under GPUI's test platform, with
real macOS keystrokes, and `tests/fuzz.rs` fuzzes it with these keys in the mix.

## What building it found

Fixed in fictty:

- **Shortcuts didn't work while writing.** A focused editor took every key, so ⌘T typed nothing
  and did nothing. Now a key with ⌃, ⌥ or ⌘ that the editor has no use for goes on to the UI's
  keys; the editor keeps the ones it uses (⌘C, ⌥←, ⌘Z and the rest).
- **A key couldn't name today.** Action templates read the selected row and the store, not the
  UI's sources; now `{today.id}` reads the `today` source (a row's own field of that name wins).
- **Zen needed a toggle for a plain flag.** `toggle` flipped only a field of a list's selected
  row; `{"toggle": "store.zen"}` flips a store entry.
- **The selection jumped back.** A `do` that moved a list's selection (`up`) and then wrote the
  store had the selection follow its old row back, so ⌥[ opened yesterday with today still
  selected. A step now notes the row it moved to at once.
- **A click on a task's box moved the caret into the box.** It now ticks the box and leaves the
  caret where it was.
- **A blank name made a nameless page.** An input's `required` holds its keys while it's blank.

Still open:

- **Editors are monospace** (caretline lays text out in cells; proportional editors are phase 4
  of native.md). Zen is quiet, but not yet beautiful: the text can't take the theme's serif or a
  larger size.
- **Ctrl-T is wired in.** There's no action that ticks a task, so ⌘Enter or ⌘L can't be bound to
  it.
- **Days are a list, not a calendar.** ⌥[ walks the journal's rows; a day nobody wrote in can't
  be reached, and the list is oldest first (`append` only adds at the end). Date arithmetic is a
  host's, or an agent's.
- **A map's catch-all can't use the value** (`{kind|done=✓;*=<accent>{kind}</>}` doesn't
  parse), found by the Kestrel demo.
- **Zen's top margin is an empty text** shown only in zen. The column is centred with
  `justify` now, but a `margin` can't depend on the store, so the six rows above the text in
  zen still come from a spacer.
- **Two lists show a selection** (the open day and the last page); the theme's
  `row.selected.blur` decides how loud the unfocused one is.
- **Open tasks from other days** ("also today" in thc) need a query across entries: a host's or
  an agent's job, not the UI's.
