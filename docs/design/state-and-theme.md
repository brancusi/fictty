# State and theme: who owns what

A proposal, 10 October 2026. Status: **built**, merged into `main` on 11 October 2026. What we learned building it is in
[prototype-notes.md](prototype-notes.md).

fictty mixed three kinds of state in one place, and an agent could wipe two of them by
accident. A `push` replaced the UI, and with it every data value an agent had sent and the text of
any editor whose id the new UI left out. A second agent rewriting a note could erase what the
person had just typed. This proposal splits the state by owner, gives each owner its own rules, and
does the same for the look of the screen.

## The rule

**Agents own the UI. The store is shared, and every write says which revision it was based on.
People own their view. The theme decides the look, and the UI only says what things mean.**

| Layer | Owner | Changed by | Survives a push? |
|---|---|---|---|
| UI (`push`, `patch`) | the agent | replacing it | it *is* the push |
| Store (the app's data) | shared | writes: typing, `put`, `merge`, `edit`, store actions | always |
| View state (selection, carets, scroll, focus) | the person | keys, mouse, `set` | for every node id still there |
| Theme (tokens, parts, variants) | the person or project | `--theme`, the `theme` op, a theme key | always |
| Sources (commands, streams) | the runtime | their commands | refetched; disposable by design |

## The store

The store is a flat map of named, versioned **entries**. Each one is either a JSON value or a text
document (a caretline document: the text and its undo history).

```jsonc
"store": {
  "ledger":  {"rev": 12, "by": "person", "value": [{"date": "2026-10-09", "payee": "Rent", "amount": -1450}]},
  "standup": {"rev": 77, "by": "agent",  "doc": {"text": "# Standup…", "history": {…}}}
}
```

- **`rev`** goes up with every write, from anyone. **`by`** says who wrote last: `seed`, `person`,
  or an agent's name (`--by`).
- **Writes** come from typing in an editor or input, from store actions on keys (below), and from
  agents over the socket: `put` (a value; a string replaces a text's contents as one undoable
  edit), `merge` (RFC 7396 into a value), `edit` (`[from, to, "text"]` ranges of a text).
- **`--if-rev N`** makes a write conditional. If anyone wrote since revision N, the write is refused
  and the reply carries the entry as it is now, so the agent reads it and tries again. This is
  caretline's stale-revision rule, applied to every entry.
- **An agent's text edit is an edit, not a swap.** It goes through a view of its own, so the
  person's carets move with the text instead of jumping, and one undo takes it back.
- **Seeds.** A UI can carry starting values (`"store": {"ledger": […]}`), and an editor or input
  its starting `text`. A seed is written only when its entry doesn't exist yet. A push never
  overwrites or deletes an entry.
- **Persistence.** `fictty run --store file.json` reads the store at the start and writes it
  after every change, atomically (written beside the file, then renamed over it).

### Shapes belong to components, not to the store

The store doesn't have a schema. Each component says what it reads at its `bind`:

| Component | Reads |
|---|---|
| `list`, `table`, `bars` | an array (of objects, for templates and columns) |
| `sparkline` | an array of numbers, or `field` of each object |
| `text`, `gauge`, `big` | any value; templates pick fields from it |
| `editor`, `input` | a text entry (a string becomes one the first time an editor binds to it) |
| `@id` | derived: the selected row of another list, never stored |

When a bound value doesn't have the shape its component reads, the runtime says so in words an
agent can act on: `table ledger: expected an array, got an object`. Checks run after every push
and every write, and the errors come back in the reply and in the status line. They used to draw
nothing and say nothing.

### Documents, not tables

Entries are **denormalised documents**: each is whatever shape the agent or command produced,
self-contained. That suits agents (a command's JSON drops straight in; reading an entry back gives
the whole thing) and matches A2UI's data model (one JSON object per surface, addressed by pointer).

What we don't do: copy a record into two entries (they drift), or store derived views (a sorted
list, a total, the selected row). Those are computed from the one copy: `sort` on a node, `@`
binds, a table's column totals. That's the "one source of truth" rule.

**Later, when a real example asks for it:** collections keyed by id with an `order`, so selection
follows an id instead of a row number; writes by JSON Pointer path with a revision per record, so
two agents editing different rows don't collide. References between entries (foreign keys) wait
until something needs a join that a path or an `@` bind can't do.

## Store actions: what keys can do to the store

Keys and submits are data, not code. These actions only substitute values into templates and
write; there are no conditions, loops or expressions.

```jsonc
{"append": "ledger", "value": {"id": "{$id}", "date": "{store.date}", "payee": "{store.payee}", "amount": "{store.amount}"}}
{"put": "payee", "value": ""}          // clear an input
{"remove": "ledger"}                   // the focused list's selected row
{"do": [ … ]}                          // several, in order
```

Templates read `{field}` from the selected row, `{store.key}` (and `{store.key.path}`) from the
store, and `{$id}`, a fresh id. A string that is a single placeholder takes the value with its type
(a number stays a number); text is never guessed into a number, because a payee called "2024" or
an id like "5" must stay text. Components that add up read numbers from strings. Each action is a
write by `person`.

## Components added

- **`input`**: a one-line caretline field on a text entry. Enter runs its `keys.enter`.
  `placeholder` shows while it's empty.
- **`editor` with `doc`**: `"bind": "@notes", "doc": "note/{id}"` edits the entry named by the
  selected row, so a list of notes and one editor make a notes app. Each note keeps its own
  carets.
- **Table totals**: a column with `"total": "amount"` sums that path under the rows.
- Commands a screen runs (`run`, sources) get `FICTTY_SOCKET` and `FICTTY`, so a helper script
  can read the store and answer with a stream.

## The theme

Today the theme is a map of colour tokens inside the UI, so every push replaces it, and agents can
write any literal colour. A2UI removed theming in v1.0 so the host owns the look; json-render
leaves the look to each catalog's components and exposes only semantic props (`variant:
"success"`). fictty takes the middle path: **the theme is its own layer, owned by the person or
project, and nodes ask for meanings, not colours.**

```jsonc
// themes.json: fictty run --themes themes.json
{
  "glass": {
    "tokens":   {"bg": "#0a0b10", "fg": "#d6dae3", "accent": "#39d6e6", "warn": "#ffbd4f", "neg": "#ff7aa2"},
    "parts":    {"table.header": "accent+b", "row.selected": "on-ring+b", "border.focused": "accent",
                 "title": "fg+b", "editor.selection": "on-ring", "input.placeholder": "dim+i"},
    "variants": {"card": {"edge": "rounded", "style": {"bg": "panel"}},
                 "quiet": {"edge": "plain", "style": {"fg": "dim"}}}
  },
  "paper": { … }
}
```

- **Tokens** are named colours. Styles and template markup refer to them (`"fg": "accent"`,
  `<warn+b>…</>`).
- **Parts** style the pieces of components the runtime draws: headers, the selected row, borders
  (focused or not), titles, the caret and selection, placeholders, the status line. A part's value
  is the same style spec as template markup.
- **Variants** are named looks a node asks for with `"variant": "card"`: its style, border and
  backdrop. **`tone`** (`"tone": "warn"`) is shorthand for the foreground token.
- **Precedence:** the node's own `style`, then its variant, then the theme's parts, then tokens.
  Nothing else cascades.
- **Switching:** the active theme is view state (the person's). `{"theme": "next"}` on a key
  cycles, `fictty theme <name>` sets it, `fictty theme --load file.json` adds themes. A UI's own
  `theme` map still works, as tokens under every theme (a fallback, like a seed).
- **Unknown names are errors**, reported like shape errors.

## What an agent sees

- `fictty get --state`: the store (with revisions), view state, the active theme.
- `fictty read <key>`: one entry, ready to base a conditional write on.
- Errors (stale writes, bad shapes, unknown tokens) as sentences with the current value attached.
- `$log`, a built-in source of everything agents did over the socket, newest first, which a UI can
  bind a list to.

## Not in this round

Collections keyed by id, path writes and per-record revisions; flexbox layout; project components;
`watch`. The plan lists them.
