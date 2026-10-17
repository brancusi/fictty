# fictty: the data model

fictty keeps an app's data in a **store**: a flat map from string keys to versioned entries, held
in memory by one running screen and changed only by writes. Each entry is either a JSON value or
a text document (a caretline document with its own undo history), and carries a revision number
and the name of whoever wrote it last. The UI that shows the data is a separate JSON value owned
by agents (a push replaces it, and never touches the store), and what the person is doing (the
selected row, carets, scroll, folds) is a third layer, keyed by node id. There is no schema, no
query language, no links between entries and no history beyond each text's undo. What fictty
offers instead is a small, inspectable surface that agents and a person can write to at the same
time without losing each other's work: conditional writes by revision, RFC 7396 merges, and text
edits that are edits rather than replacements. This page describes the model as the code has it
on 17 October 2026, and marks what is only planned.

- **Maker:** the fictty project (Aram, @brancusi, with Claude as Tilde); open source, Rust.
- **Kind:** a UI runtime whose screen is one serialisable value, driven over a Unix socket by
  agents, a person and an embedding app; not a notes or knowledge tool.
- **Storage:** in memory, in `State.store`; optionally a single JSON file (`fictty run --store
  file.json`) rewritten atomically after every change; entries under host prefixes are kept by
  the embedding app (`Host::load` / `Host::save`) instead.
- **Unit of data:** a store **entry**: `{rev, by, value}` or `{rev, by, doc}` under one string key.
- **Source of truth:** the running screen's `State` while it runs; the store file or the host is
  only where it lasts between runs.

## The unit of data

The unit is the entry (`crates/fictty-core/src/store.rs`):

```rust
pub struct Entry {
    pub rev: u64,        // goes up by one with every write, from anyone
    pub by: String,      // who wrote last: "seed", "person", "host", "agent" or an agent's name
    #[serde(flatten)]
    pub body: Body,
}
pub enum Body {
    Text { doc: Box<caretline::Document> },  // text an editor shows, with its undo history
    Value { value: serde_json::Value },      // any JSON: rows, numbers, a record, a flag
}
```

Serialised, the store is a JSON object of entries:

```json
{
  "ledger": {"rev": 12, "by": "person", "value": [{"id": "a1", "payee": "Rent", "amount": -1450}]},
  "zen":    {"rev": 3,  "by": "person", "value": false},
  "page/Welcome": {"rev": 77, "by": "agent", "doc": {"text": "# Welcome\n…", "history": {"…": "…"}}}
}
```

Read back over the socket (`fictty read <key>`, `Entry::read`), an entry is simpler:
`{"rev": 12, "by": "person", "value": [...]}`, or `{"rev": 77, "by": "agent", "text": "# Welcome…"}`
for a text entry, whose history is not included.

The two bodies behave differently:

- **A value entry** is any JSON value. It is written whole (`put`), merged into (`merge`, RFC
  7396), or changed in place by the store actions (`append`, `remove`, `toggle`). The revision
  counts writes to the entry as a whole, never to a part of it.
- **A text entry** is a caretline `Document`: the text (a rope), its undo history (a revision
  tree), block marks and editing config. It is written by typing, by `edit` (character ranges
  `[from, to, "text"]` of the current text), or by `put` of a string, which fictty turns into
  the minimal edit (`caretline::diff::changes`) so carets and undo outside the changed span stay
  put. `merge` on a text entry is refused (`is text: merge is for values`), as is `put` of a
  non-string and `edit` on a value entry.
- **A string can become text.** An entry created by `put` of a string is a value entry holding a
  string; the first time an editor or input binds to it, fictty converts it into a text entry
  with the same `rev` and `by`. An editor bound to an entry holding any other value shows a
  problem (`holds a value, not text`) rather than converting it.

The carets, selection and scroll of each editor are not part of the entry: they live in the
editor node's view state (`Local.editors`, one `caretline::View` per entry it has shown), so two
editors on one text keep their own carets.

## Identity and references

There are four kinds of identifier, and none of them is a reference in the database sense.

- **Store keys** are free strings. Slashes are a convention, not structure: `note/{id}`,
  `page/Welcome`, `remove/photo.tif`. A key can contain dots; a bind resolves the longest store
  key it starts with (ending at a dot or the end) and treats the rest as a path into its value,
  so `remove/photo.tif` is one entry and `ledger.0.payee` is a path into `ledger`.
- **Node ids** (`"id": "notes"`) identify UI nodes, not data. View state and focus are keyed by
  them and survive a push for any id still in the new tree (as React's `key`). Nodes repeated by
  `each` take ids from their item (`"id": "panel-{$item.id}"`).
- **Row keys** identify rows inside an array entry for the view only. A `list`, `tree` or `table`
  has `key` (default `id`): the selection is remembered as that row's key (`Local.key`), so it
  follows its row when rows are inserted or removed above it, and the `select` action and
  `fictty set` can pick a row by key, even one that has not arrived yet (`Local.want`). The store
  itself does not know about these fields; an `id` is an ordinary field in an object.
- **`$ptr`** is a JSON Pointer to a shown row within its bound entry (`/3`, or `/0/children/2`
  in a tree), computed after each message. Store actions on "the selected row" (`remove`,
  `toggle`) find their target through it, so they reach the right row through a filter or deep in
  a tree. It is an address by position, valid until the array changes.

**Binds** connect a node to data: `"bind": "ledger"` (an entry), `"bind": "books.running"` (a
path with dots into an entry or a source), or `"bind": "@notes"` (the selected row of node
`notes`, and `@notes.title` a field of it). An `@` bind is derived, never stored: a detail pane
follows a list's selection without anything being copied. A bind reads the store before sources,
so an entry named like a source hides it (which is reported as a problem).

The nearest thing to a reference between records is a **key built from a row**. An editor's
`doc` is a template that names the entry it edits from the row it is bound to:

```json
{"type": "editor", "bind": "@notes", "doc": "note/{id}", "text": "# {title}"}
```

Selecting a note opens the text entry `note/<that id>`. Daybook (the journal example) does the
same with `"doc": "{kind}/{id}"`. This is a naming convention resolved at the moment of display;
nothing records that `note/a1` belongs to row `a1`, nothing stops the two from drifting apart, and
nothing cleans up `note/a1` when the row goes.

There are **no links and no backlinks**. An editor's `links` option makes `[[wiki]]`, `#tag` and
URL spans in a text clickable, with `{$match}` filled into an action (Daybook opens or appends a
page); that is behaviour on text, not a relation in the store. Nothing indexes which texts mention
which pages, and there is no backlink view. Foreign keys between entries are explicitly deferred
in [state-and-theme.md](../../design/state-and-theme.md) "until something needs a join that a path
or an `@` bind can't do".

## Structure and hierarchy

A running screen's `State` (`crates/fictty-core/src/state.rs`) holds three kinds of thing with
three owners:

| Layer | Field | Owner | Changed by | Survives a push |
|---|---|---|---|---|
| UI | `ui: Ui` | agents | `push`, `patch` | it is the push |
| Store | `store: BTreeMap<String, Entry>` | shared | writes | always |
| View state | `local: BTreeMap<String, Local>`, `focus` | the person | keys, mouse, `set` | per node id |
| Sources | `data: BTreeMap<String, Slot>` | the runtime | fetches, streams | kept if unchanged |
| Theme | `themes`, `theme` | person or project | `theme` op, keys | always |

- **The UI tree** is the only real hierarchy: `Ui { root: Node, data, store, keys, theme, layers,
  focus, status_bar }`, nodes nesting through layout kinds (`col`, `row`, `stack`, `grid`,
  `tabs`, `each`). It is presentation, not data, and agents replace it freely.
- **The store is flat.** Entries do not contain other entries. Hierarchy lives inside a value:
  arrays of objects for lists and tables, nested arrays for trees (each row's `nest` field,
  default `children`), objects for records.
- **View state** (`Local`) per node id: `selected` (an index), `key` (the selected row's key),
  `tab`, `toggled` (tree rows folded or opened, by key), `picked` (a multi-selection, by key),
  `anchor`, `top` and `offset` (scroll), `heights`, and `editors` (carets and scroll per text
  entry). It is serialised and readable, but it is not the app's data.

There is **no schema**. The store accepts any JSON value under any key. Shape is checked only
where it is read: each component says what it expects at its `bind` (a `table` an array of
objects, a `sparkline` an array of numbers or a `field` of each object, an `editor` a text
entry), and a mismatch becomes a sentence in `State.problems`, such as `table ledger: expected an
array, got an object`, checked after every push, patch, write and fetch and shown in replies, the
status line and `get --state`. Store actions keep types: a value that is one placeholder
(`"{store.amount}"`) takes the referenced value with its type, and text is never guessed into a
number. `input` with `"accept": "number"` refuses non-numeric typing, but stores the text.

The design is deliberately **denormalised documents**: each entry is whatever shape the agent or
command produced, self-contained, so a command's JSON drops straight in and reading an entry gives
all of it. Copies are forbidden by convention (the "one source of truth" rule in
[AGENTS.md](../../../AGENTS.md)); derived things such as a selected row, a filtered list or a
column total are computed, not stored.

## Storage and the source of truth

- **In memory.** While a screen runs, `State.store` is the truth. Every reader (the view, the
  socket, the host) sees the same map, and every writer goes through `update`.
- **Seeds.** A UI's `store` map (`"store": {"zen": false, "days": [...]}`) and an editor's or
  input's `text` are starting values: each is written, as `by: "seed"`, only when its entry does
  not exist yet. A push never overwrites or deletes an entry, so pushing a revised UI keeps
  everything the person typed.
- **The store file.** `fictty run --store file.json` loads the entry map at start and, after any
  batch of messages that bumped `store_writes`, writes the whole map beside the file
  (`file.json.tmp`) and renames it over the original. Text entries are saved with their caretline
  history, so undo survives a restart. One screen holds a given store file at a time, through a
  lock file in the temporary directory named from a hash of the store's path; a second screen
  refuses to start.
- **Host-kept entries.** An embedding app's `Host::keys()` returns prefixes (`["page/"]`). Entries
  under them are never seeded and are left out of the store file. When something on screen first
  needs one, the engine calls `Host::load(key)` (a string becomes text, anything else a value;
  `by: "host"`); after it changes and stays unchanged for half a second, the engine calls
  `Host::save(key, value)` off the drawing thread. `Notifier::reload(key)` brings in a change made
  at the host; text arrives as an edit, so carets and undo carry on, and unsaved typing is saved
  first so the host can merge. This is how a vault of Markdown files or a database can sit behind
  the store, but the merging is the host's job.
- **Not saved:** the UI (an agent re-pushes it, or `run ui.json` reloads it), view state, source
  values (refetched; "disposable by design"), `$log`, the expanded tree and derived rows, and the
  internal `$mixed/<id>` entries a multi-row editor uses. The whole `State`, view state included,
  is carried only across `fictty upgrade`, which serialises it and hands it to the new binary
  (`FICTTY_RESTORE`, `run --restore`).
- **Sources** are not the store. `data` holds named sources: `cmd` (argv), `shell`, `stream` (a
  command printing JSON lines), a literal `value`, or `host` (with templated `args`), narrowed by
  `path` and refetched by `every`. Their latest results sit in `State.data` as `Slot`s, read-only
  to the UI. Built-in sources expose the engine itself: `$store` (each entry's `key`, `rev`, `by`
  and what it `holds`), `$log`, `$picked`, `$screen`, `$stats`, `$caps`, `$theme`, `$keys`,
  `$self`.

## Queries and derived views

There is no query language. What a screen can derive from stored data is a fixed set of view
features, all declared in the UI value and all recomputed after each message:

- **Paths and `@` binds**: a bind picks a sub-value (`books.running`) or another list's selected
  row (`@notes`, `@notes.tags`).
- **Filters**: a list, tree or table's `filter` names a store entry (usually an `input`'s text);
  every shown row must contain each of its whitespace-separated words, matched case-insensitively
  against the row's strings and numbers. The matches are held in `State.rows` with their `$ptr`.
- **Repetition**: `each` instantiates a node template once per item of an array, with `{$item}`,
  `{$item.path}`, `{$index}` and `{$count}`; with `cols`, a grid of cards from data.
- **Trees from nested data**: a `tree` walks each row's `nest` field and gives templates
  `{$depth}`, `{$kids}` and `{$open}`; folds are view state, by row key.
- **Templates**: `{field}`, `{field.path}`, `{name:12}` (fitted), `{±change}` (coloured by sign),
  maps (`{done|true=☑;*=☐}`), presence tests (`{x|?then $}`, `{x|!otherwise}`), `{$store.key}`
  beside the bound value, and `{$rows}`, the number of rows a list shows after its filter.
- **Conditions**: `when` and `on` take a bind that is set, not set (`!dialog`), compared
  (`mode=edit`, `$screen.width>=120`), or several joined by ` & `.
- **Aggregates**: only a table column's `total` (a sum of a path over the rows) and the counts in
  `$picked` and `$rows`. There is no sort: the order shown is the order stored (Daybook's notes
  say `append` only adds at the end, so its journal is oldest first). The
  [state-and-theme.md](../../design/state-and-theme.md) note that sorting is computed "with `sort`
  on a node" describes an intent; no `sort` field exists in `model.rs`.

The rule in [AGENTS.md](../../../AGENTS.md) ("Data, never code") keeps it this way on purpose: no
expressions, loops or conditionals beyond substitution. Anything heavier is done outside fictty,
by a `cmd` or `stream` source, a host source, or an agent that reads entries and writes a result.

## Change, history and sync

**Writers.** Everything that changes the store is a `Msg` applied by the pure
`update(&mut State, Msg) -> Vec<Effect>` on the screen's one engine thread, so writes are
serialised in arrival order. The writers are:

- **The person**, by typing in an `editor` or `input` (`by: "person"`), and through **store
  actions** bound to keys and clicks, all also `by: "person"`:
  - `{"put": "newname", "value": ""}` (with optional `for_ms`)
  - `{"append": "ledger", "value": {"id": "{$id}", "payee": "{store.payee}"}}`, which creates the
    array if missing
  - `{"remove": "ledger"}` (the focused list's selected row, or every picked row, by `$ptr`), or
    `{"remove": "pages", "where": {"id": "{store.newname}"}}` (every row whose fields match)
  - `{"toggle": "done"}` (a field of the selected or picked rows), `{"toggle": "store.zen"}` (an
    entry or a path into one), `{"toggle": "store.loved/{id}"}` (an entry named from each picked
    row)
  - `{"do": [...]}`, several in order. There is no `merge` action and no upsert by id: Daybook
    adds a page without duplicating it by removing `where` the id matches and then appending.
- **Agents**, over the socket: `write` with `put`, `merge`, `edit` or `delete` (`by` defaults to
  `agent`, set with `--by`).
- **The host**: `Notifier::write(key, write, by)` and `write_for`, and `Loaded` values from
  `Host::load` (`by: "host"`).

**Revisions.** `rev` goes up by one with every write to an entry, whoever made it; a new entry
starts at 1. `by` records only the last writer, not a list of contributors.

**Conditional writes.** A write may carry `if_rev`:

```sh
fictty read settings                         # {"rev": 12, "by": "person", "value": {"currency": "EUR"}}
echo '{"month": "2026-10"}' | fictty merge settings - --if-rev 12 --by claude
```

On the socket that is `{"op": "write", "key": "settings", "write": "merge", "value": {"month":
"2026-10"}, "if_rev": 12, "by": "claude"}`. A write's `key` is always a whole entry name, never a
path (a merge to `settings.month` would create an entry of that name). If the entry's `rev` is not
12, the write is refused with `stale: settings is at rev 13, not 12; read it and try again`, and
the reply carries the entry as it
now is (`current`). `if_rev: 0` succeeds only while the entry does not exist, a create-only write.
A refused write shows in `$log` as `refused`, with the reason. Store actions and host writes are
never conditional. Because the revision is per entry, two agents changing different rows of the
same list collide.

**Merges.** `merge` follows RFC 7396 (`store::merge`): object keys replace, `null` removes, objects
merge deeply, and anything that is not an object replaces the target whole. Arrays cannot be
patched element by element; a change to one row means a `put` of the whole array, or a store
action.

**Text.** An agent's edit to a text entry is an edit, not a swap: `text_edit` applies the ranges
through a temporary caretline view of its own, so every editor's carets map through the change and
none jumps, and the change is one undo step. Ranges must be in order, apart and within the text.
`if_rev` on text is strict, so an agent racing a person who is typing is refused on every
keystroke in between; rebasing an agent's edit onto the person's typing is planned (PLAN.md §2).

**Undo** exists only for text entries, as caretline's per-document revision tree, shared by all
writers: the person's ⌘Z steps back through the latest change to that text, an agent's included.
Value entries have no undo and no previous versions.

**Expiring writes.** `for_ms` on a `put` or `merge` (`fictty put … --for-ms 3000`, the `put`
action, `Notifier::write_for`) schedules its reversal on the logical clock (`State.expiring`): a
put's entry goes, or the leaf keys a merge set. Because the expiry is in the state, it replays.

**History today** is thin: per-entry revisions, the text undo trees, and `$log`, the last 40
things agents did over the socket (`{at, op, what}`, newest first, consecutive keys joined). Since
every input is a `Msg` and `update` is pure, a recorded message log would reproduce a session
exactly, but nothing records one yet.

**Sync** is not a feature: there is one writer process per store, holding its file under a lock.
Agents and the person are concurrent writers only in the sense that they all send messages to
that process. There is no replication, no offline merge and no CRDT.

**Planned** ([PLAN.md](../../PLAN.md) §2, §3, §6; [time-travel.md](../../design/time-travel.md)):

- `watch`: wait for events (a person's store write, a submit, a click, a key) with the revision
  to read from, instead of polling. (`fictty run --watch` today only reloads the UI file.)
- Text edits rebased onto the person's typing; `if_rev` stays strict for values.
- Validated batches: several writes applied together or not at all.
- Collections keyed by id with an `order`, which lists and tables bind to.
- Writes by JSON Pointer path, with a revision per record, so writers on different rows do not
  collide; store actions that address a record by id (`put` by id next).
- JSON Patch (RFC 6902) as the patch dialect for values.
- History: record inputs and sparse snapshots in a bounded ring, then `history`, `state_at`,
  `screen_at`, `diff`, `seek`, `fork`, `replay`, marks, export and import, with `if_seq` on every
  write.

## Extensibility and agent access

**The socket.** A screen listens on a Unix socket (`FICTTY_SOCKET`), one JSON request per line,
tagged by `op` (`engine.rs`, `Request`): `push`, `patch`, `key`, `quit`, `write`, `read`, `theme`,
`set`, `layers`, `toast`, `mouse`, `click`, `screen`, `readback`, `get`, `state`, `stats`,
`upgrade`. Each connection gets its own thread, but every request becomes a message to the one
engine.

**The CLI** has one subcommand per operation: `fictty push`, `patch`, `put`, `merge`, `edit`,
`read`, `delete` (each write with `--if-rev`, `--by`, and `--for-ms` for put and merge), `key`,
`mouse`, `set`, `theme`, `layers`, `toast`, `get` (the UI, or with `--state` the whole state:
store with revisions, view state, sources, problems), `screen` (the frame as text, `--nodes` for
the readback), `render` (a UI file to text without a screen) and `upgrade`. Commands a screen runs
get `FICTTY_SOCKET` and `FICTTY` in their environment, so a helper script can read the store and
answer.

**Readback.** `readback.rs` gives the screen as an agent reads it: each shown node in order, with
its kind, id, depth, title, what it shows, its rows and which is selected or picked, focus, and
rectangle. It reads the same whatever draws the state.

**The host** (`host.rs`): an app embedding fictty implements `Host`, with `fetch` (host sources),
`act` (host actions, whose answers go to `$log`), `keys`, `load` and `save` (entries it keeps),
and `start`, which receives a `Notifier` (`changed`, `reload`, `write`, `write_for`, `focus`,
`msg`). Whatever the host contributes arrives as a message, so it shows in `get --state` and
`$log`; the view never reads the host.

**What an agent can do:** read any entry or the whole state; write any entry, conditionally;
seed entries through a UI; replace or patch the UI, including the actions the person's keys will
perform on the store; drive the screen as the person would (keys, mouse, `set`); read the screen
back as text or as nodes.

**What it cannot do:** subscribe to changes (it polls `read` or `get --state` until `watch`
lands); write part of an array by path or address a record by id; group writes atomically; query
or search entries on the engine side; list who has written an entry beyond the last writer; read
an entry's previous values; or extend the component set or the action vocabulary from data. The
command policy for pushed UIs is also still open: a pushed UI may `run` anything its user could.

## Strengths and gaps

What the model does well, for agent-built screens:

- **It is all inspectable.** The UI, the store with revisions, the view state and the problems are
  one JSON value an agent can read with `get --state`, and every entry can be read alone with its
  revision ready for a conditional write.
- **Ownership is clear.** Agents can replace the UI as often as they like without losing data;
  the person's selection and carets survive pushes; the store changes only through writes.
- **Concurrent writers do not silently clobber each other.** `if_rev` turns a lost update into a
  refusal with the current value attached, and agent text edits merge with the person's carets
  and join the undo history instead of replacing the text.
- **Any JSON drops in.** Without a schema, a command's output or an API response can be stored
  and shown at once; shape errors come back as sentences an agent can act on.
- **Deterministic by construction.** Pure `update` over messages, the logical clock, and expiries
  kept in the state make sessions replayable, which the planned history builds on.

Where it is weak compared with a notes or knowledge tool:

- **No record identity across entries.** A record is either a whole entry or an element of an
  array; the `id` field is a convention that lists use for selection, not something the store
  enforces or indexes. Keys like `note/{id}` tie a text to a row by name only, with no integrity
  and no cleanup when the row goes. `{$id}` for new rows is the sum of all revisions plus one, in
  hex; from reading `action_context`, deleting an entry lowers that sum, so an id can repeat after
  a `delete`.
- **Whole-entry writes and revisions.** One revision per entry means any two writes to the same
  list conflict under `if_rev`, and an agent changing one row must `put` the whole array. Path
  writes and per-record revisions are planned, not built.
- **No links or backlinks.** `[[wiki]]` spans are clickable but not indexed; there is no graph,
  no transclusion, no block references (caretline has block marks internally, but fictty does
  not expose them as identifiers).
- **No schema or types** beyond what each component expects at display time; no property types,
  no validation on write, no relations.
- **No query.** Filtering is substring words over one bound array; there is no sort, grouping,
  join, or cross-entry search, and nothing like a saved query or database view.
- **No history yet.** Value entries keep no past versions; text undo is per document and shared
  between writers; `$log` keeps 40 socket actions. Recording, `diff`, `seek` and `fork` are
  proposals.
- **One process, one file.** The store is a single JSON file rewritten whole on each change,
  owned by one running screen; there is no multi-device sync, and scale is bounded by memory and
  by rewriting the file.

The fair summary is that fictty's store is a shared scratch space for a live screen, built to make
concurrent agent and person writes safe and visible, while the durable, linked, queryable
knowledge model that Obsidian, Roam, Logseq or Notion provide is left to a host (such as Thought
Control) behind `Host::keys`, `load` and `save`.

## Sources

- [store.rs](../../../crates/fictty-core/src/store.rs): `Entry`, `Body`, `Write`, `merge`
- [state.rs](../../../crates/fictty-core/src/state.rs): `State`, `Local`, `Msg`, `update`,
  `write_entry`, `text_edit`, store actions, `value_of`, `row_targets`, `action_context`,
  `row_matches`, `store_summary`, `log`
- [model.rs](../../../crates/fictty-core/src/model.rs): `Ui`, `Node`, `Kind`, `Rows`, `Source`,
  `Action`
- [engine.rs](../../../crates/fictty-core/src/engine.rs): `Request`, `load_store`, `save_store`,
  the store lock, hosted entries (`keep`, `unsaved`), sources and streams
- [host.rs](../../../crates/fictty-core/src/host.rs): `Host`, `Notifier`
- [readback.rs](../../../crates/fictty-core/src/readback.rs): the screen as an agent reads it
- [main.rs](../../../crates/fictty/src/main.rs): the CLI subcommands and their flags
- [state-and-theme.md](../../design/state-and-theme.md): the ownership rule, the store, store
  actions
- [time-travel.md](../../design/time-travel.md): the history proposal
- [guide.md](../../guide.md): "The model", "The store", "Embedding fictty in an app"
- [PLAN.md](../../PLAN.md): §2 Close the agent loop, §3 Records, §6 History
- [AGENTS.md](../../../AGENTS.md): the architecture rules
- [examples/journal/README.md](../../../examples/journal/README.md) and
  [journal.py](../../../examples/journal/journal.py): Daybook, a notes app built on the store
- caretline 0.5.0 (crates.io), `Document`: text, history, marks
