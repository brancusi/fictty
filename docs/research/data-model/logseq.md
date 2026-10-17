# Logseq: the data model

Logseq is an outliner: everything is a tree of blocks, and pages are named roots of those trees.
It has had two data models, and in 2026 they became two products. The original, now **Logseq OG**
(github.com/logseq/og, version 1.0.0, April 2026), keeps a folder of Markdown or Org files as the
source of truth and parses them into an in-memory DataScript database that the app queries. The
new one, the **DB version** (github.com/logseq/logseq, the 2.0 betas: 2.0.1 on 13 July 2026, 2.0.2
on 7 October 2026), makes the DataScript database itself the source of truth, stored in SQLite,
with typed properties, tags that act as classes, and a server-ordered sync that lets several people
edit one graph. The move from the first to the second took three years (the "Database version"
pull request was opened in July 2023 and merged in May 2025) and ended with the team splitting the
app in two rather than carrying both. This note describes both models, says which is which
throughout, and sets them against fictty's UI value and store. Checked against the sources on
17 October 2026.

- **Maker:** Logseq Inc. (Tienson Qin, Zhiyuan Chen and team); AGPL-3.0, about 45,000 GitHub stars
- **Kind:** local-first outliner and personal knowledge base; Electron desktop, web, iOS and
  Android; plugins; a paid sync service (self-hostable for the DB version)
- **Storage:** OG: `pages/*.md` and `journals/*.md` in a folder, with `logseq/config.edn`; DB
  version: one `db.sqlite` per graph under `~/logseq/graphs/<name>/`, plus `assets/`
- **Unit of data:** the block (in the DB version, the *node*: a block or a page, which now behave
  alike), an entity of datoms with a UUID, a parent, an order and properties
- **Source of truth:** OG: the files; the database is a derived, re-indexable cache. DB version:
  the SQLite database; Markdown is an export or a one-way mirror

## The unit of data

Both models store the same kind of thing, a **block**: one bullet of an outline, with its text,
its place in a tree and its properties. A **page** is a block with a name that other blocks hang
under. A **journal** is a page for a day. What differs is where a block's fields live.

**OG (file graphs).** A block is a list item in a Markdown file. Its properties are `key:: value`
lines written inside the item, under its first line:

```markdown
- How to Take Smart Notes
  id:: 6512f0c4-3b7b-4cb2-8c3e-6a904c813c40
  type:: [[book]]
  author:: [[Sönke Ahrens]]
  rating:: 8
	- TODO Write a summary
	  SCHEDULED: <2026-10-20 Tue>
```

The parser (mldoc, an OCaml parser compiled to JavaScript) turns each file into blocks and
transacts them into DataScript. The schema in `logseq/og` (`deps/db/src/logseq/db/schema.cljs`)
shows what a block becomes: `:block/content` (the full text, properties included),
`:block/properties` (a map from keyword to value, where a value that contains references becomes a
set of page names), `:block/properties-text-values`, `:block/marker` (`"TODO"`, `"DOING"`),
`:block/priority`, `:block/scheduled`, `:block/deadline`, `:block/parent`, `:block/left` (the
sibling before it), `:block/page`, `:block/refs` and `:block/path-refs` (references inherited
from ancestors), `:block/file`. Page properties are the first block of the page, its
"frontmatter". Property values cannot hold newlines, since a newline ends them; `tags::` and
`alias::` split on commas; quoting a value stops it linking. Values are untyped text: the DB
version's docs contrast its numbers, "Unlike file graph properties, these are stored as actual
numbers".

**DB version (database graphs).** A **node** is a page or a block; the docs say "the two now
behave similarly". It is an entity in DataScript whose core attributes are fixed by
`deps/db/src/logseq/db/frontend/schema.cljs` (schema version 65.35 on 17 October 2026):
`:block/uuid` (unique), `:block/title` (the text; it replaced `:block/content` and
`:block/original-name`), `:block/name` (lower-cased title, for pages), `:block/parent`,
`:block/order` (a fractional-index key replacing `:block/left`), `:block/page`, `:block/refs`,
`:block/tags`, `:block/link` (embeds), `:block/created-at`, `:block/updated-at`,
`:block/journal-day`. Properties are no longer text in the block. Each property is itself an
entity (a page tagged `#Property`) with a `:db/ident`, and a node holds a property by having that
ident as an attribute:

```clojure
;; a block tagged #Book, as `logseq debug pull --uuid …` would show it (ids illustrative)
{:db/id 4211
 :block/uuid #uuid "6512f0c4-3b7b-4cb2-8c3e-6a904c813c40"
 :block/title "How to Take Smart Notes"
 :block/page {:db/id 310}                        ; the page it is on
 :block/parent {:db/id 310}
 :block/order "a1"
 :block/tags #{{:db/id 905}}                     ; the class :user.class/Book-Kq3vX9aB
 :user.property/author-Mx81fQ2z #{{:db/id 4380}} ; a Node property: refs to other nodes
 :user.property/rating-Zp0aW7cd {:db/id 4381}    ; a Number property: a ref to a value entity
 :block/created-at 1792224000000
 :block/updated-at 1792310400000}

;; the property itself, also an entity
{:db/ident :user.property/rating-Zp0aW7cd
 :block/title "rating"
 :block/tags #{{:db/ident :logseq.class/Property}}
 :logseq.property/type :number
 :db/cardinality :db.cardinality/one}
```

Property types a person can choose (`property/type.cljs`): `:default` (Text), `:number`, `:date`,
`:datetime`, `:checkbox`, `:url`, `:node`, `:asset`; internal ones add `:keyword`, `:map`, `:json`,
`:string`, `:entity`, `:class`, `:page`, `:property`. Text, Number and Url values are themselves
small entities (a Text value is a block, so it can hold references and children; a Number value
holds `:logseq.property/value`), which is how one value can be shared, chosen from a closed list
("choices") or given history. Checkbox and DateTime values sit on the node directly. Tasks are
nodes tagged `#Task` with the built-in `:logseq.property/status` (closed values Backlog, Todo,
Doing, In Review, Done, Canceled), `priority`, `scheduled` and `deadline` properties, where OG had
`TODO` keywords in the text.

## Identity and references

**OG.** A block has a `:block/uuid` in the database, but it is only stable if it is written down.
When a block is first referenced, Logseq writes an `id:: <uuid>` property into its file; until
then the uuid is regenerated on each parse. Pages are identified by name: `:block/name` (lower
case) is unique, so two pages cannot share a title, and renaming a page rewrites the text of every
file that links to it. References are text:

- `[[Page name]]` and `#tag` (a tag is a page reference, nothing more) link to pages;
- `((6512f0c4-…))` links to a block by its uuid;
- `{{embed [[Page]]}}` and `{{embed ((uuid))}}` show a page or block inline, editable in place;
- a property value with references links each of them.

The parser records every reference in `:block/refs`, and backlinks ("Linked references") are a
query over that attribute. "Unlinked references" are a text search for the page's name.

**DB version.** Every node has a uuid from birth, and every entity a `:db/id`; schema entities
(properties, classes, built-in values) also have a `:db/ident` keyword. A user property's ident is
made from its name plus a random suffix (`create-db-ident-from-name` appends `-` and eight
characters), so renaming a property never changes its ident, and two properties can share a
title. References are by uuid underneath and shown by title: blocks are referenced with `[[ ]]`
like pages ("blocks no longer use `(())`"), and the stored title holds the uuid
(`[[6512f0c4-…]]`), which the editor shows as the current title, so a rename is a change to one
entity. Page names are unique per tag ("there can be `Apple #Company` and `Apple #Fruit`"). The
cost shows at the edges: the semantic REST API takes "UUIDs only" in paths, so an agent resolves
every name before it can write; the Markdown mirror (below) prints references as `[[Foo]]` and
accepts that duplicate titles make them ambiguous.

## Structure and hierarchy

Both models have one tree per page: a block's `:block/parent` is its parent block or its page, and
siblings are ordered (`:block/left` in OG, a sibling pointer that every insert and move has to
repair; `:block/order`, a sortable string key, in the DB version). Collapsing is a property of the
block (`collapsed:: true` in OG's file, `:block/collapsed?`).

Hierarchies of pages differ more:

- **OG namespaces** are in the name: `[[project/alpha/notes]]` creates pages `project`,
  `project/alpha` and `project/alpha/notes`, linked by `:block/namespace`, and the file is
  `pages/project___alpha___notes.md`. Renaming `project` renames every page under it.
- **DB version** namespaces are parent links between pages, managed in a `Library` page that edits
  them as an outline; `[[foo/bar/baz]]` still creates the chain but the reference becomes
  `[[baz]]`, and renaming a parent touches nothing below it. A block tagged `#Page` becomes a page
  under its parent.
- **DB version classes** form a second hierarchy: a tag can extend several parent tags
  (`:logseq.property.class/extends`, default `Root Tag`), and a node tagged `#Audiobook` inherits
  the tag properties of `#Book` and `#MediaObject`. Tag properties (`:logseq.property.class/properties`)
  make every node with that tag show and validate those fields, which turns a tag into a table
  schema ("known in other applications as types, classes or supertags").

Journals are pages for dates: OG files `journals/2026_10_17.md`, with `:block/journal?` and
`:block/journal-day`; DB version pages tagged `#Journal` (a class that extends `#Page`).

## Storage and the source of truth

**OG: files first, database derived.** The folder is the truth. On open, Logseq parses every file
into DataScript and caches the result (on desktop, a transit file under `~/.logseq/graphs/`);
a file watcher re-parses files changed outside the app; "Re-index" throws the cache away and parses
again. Edits in the app are written back to the file of the page. This gives plain text that any
editor, `git` or `grep` can read, and it gives two sources that can disagree: the cache can go
stale, a file edited by two programs can lose one side (`logseq/bak/` keeps the losing copy), and
anything not expressible in the text (a uuid nobody referenced, a property type) does not survive
a re-parse.

**DB version: the database is the truth.** A graph is `~/logseq/graphs/<name>/db.sqlite` (on
desktop; OPFS in the browser), holding DataScript's storage as rows of one table,
`kvs (addr INTEGER primary key, content TEXT, addresses JSON)`: DataScript's persistent B-tree
nodes, stored by address (`deps/db/src/logseq/db/common/sqlite.cljs`). There is no re-index. Search
and sync bookkeeping live in separate files (`search-db.sqlite`, `client-ops-db.sqlite`), which
backups deliberately skip because they can be rebuilt. One process writes a graph: a `db-worker-node`
daemon bound to `127.0.0.1`, found by a lock file, shared by the desktop app and the CLI ("A graph
must not have two active writer daemons"). Validation is part of the model: a Malli schema checks
entities, and `logseq graph validate` checks a whole graph.

What is derived in the DB version: Markdown. Exports exist (standard Markdown, EDN, SQLite), and
an optional, desktop-only **Markdown Mirror** (ADR 0016, accepted May 2026; the worker code is
in `src/main/frontend/worker/markdown_mirror.cljs`) writes `mirror/markdown/pages/*.md` and
`journals/*.md` after each change. The ADR is explicit: "The mirror must not become another graph
source of truth"; files under `mirror/markdown/**` are ignored by import, watchers and parsing so
"the mirror never feeds back into the graph"; editing them does nothing and later edits overwrite
them. The only lossless, editable text form is **Build EDN** (`logseq.db.sqlite.build`), a
readable description of pages, blocks, properties and classes without uuids or db ids:

```clojure
{:properties {:rating {:logseq.property/type :number}
              :author {:logseq.property/type :node
                       :db/cardinality :db.cardinality/many}}
 :classes {:Book {:build/class-properties [:author :rating]}}
 :pages-and-blocks
 [{:page {:block/title "Reading"}
   :blocks [{:block/title "How to Take Smart Notes"
             :build/tags [:Book]
             :build/properties {:rating 8
                                :author #{[:build/page {:block/title "Sönke Ahrens"}]}}}]}]}
```

## Queries and derived views

Both models are queried in Datalog against DataScript, with a simpler DSL on top.

**Simple queries** are an expression of filters: `{{query (and (task TODO DOING) (property type
book))}}` in OG; in the DB version a block tagged `#Query`, made with `/Query` or the query builder,
with the filters renamed in places (`(page-property)` became `(property)`, `(priority A)` became
`(priority high)`). Results show as a list or a table; in the DB version, as a view (table, list
or gallery) with sorting and filtering in the view rather than the query.

**Advanced queries** are EDN maps with a Datalog `:query`, `:inputs` (`:today`, `:current-page`,
`:-7d` and so on) and rules. In OG:

```clojure
#+BEGIN_QUERY
{:title "Open tasks on journals"
 :query [:find (pull ?b [*])
         :where
         [?b :block/marker ?m]
         [(contains? #{"TODO" "DOING"} ?m)]
         [?b :block/page ?p]
         [?p :block/journal? true]]}
#+END_QUERY
```

The same in the DB version, where task fields are properties and journals a class, queried through
the built-in rules in `logseq.db.frontend.rules` (`task`, `property`, `has-ref`, `tags` and others,
which the app adds when a query uses them; from the CLI a query appends `%` to `:in` to use them):

```clojure
{:query [:find (pull ?b [*])
         :where
         (task ?b #{"Todo" "Doing"})
         [?b :block/page ?p]
         [?p :block/tags ?t]
         [?t :db/ident :logseq.class/Journal]]}
```

Old advanced queries break across the move: `:block/content` is `:block/title`,
`:block/marker` is `:logseq.property/status`, `:block/journal?` and `:block/path-refs` are gone
(use `(has-ref ?b ?ref)`).

Advanced queries are not only data. OG's `:view` and `:result-transform` take ClojureScript
functions (`:view (fn [rows] [:div …])`), evaluated in the app. The DB version deprecates some
options (`:title`, `:group-by-page?`, `:collapsed?`); the docs reviewed here do not say whether
`:view` functions are kept (not verified).

Other derived views: backlinks and unlinked references, the graph view, the `Pages` and tag tables
(every tag page is a table of its nodes, with its tag properties as columns), flashcards and the
journals page are all queries over the same database.

## Change, history and sync

**OG.** Change is file writes. Undo and redo are in memory, per session. History is whatever the
files have: optional git auto-commit, `logseq/bak/` copies when the file on disk and the page in
the app disagree, and Logseq Sync's version history. **Logseq Sync** for file graphs (paid, beta,
end-to-end encrypted with `age`, stored on AWS) syncs *files*: it compares by md5, keeps a
transaction id per graph in `logseq/graphs-txid.edn`, and on a conflict merges by block. The
`frontend.fs.diff-merge` namespace in `logseq/og` keeps a base copy of each synced file under
`logseq/version-files/base/` and runs a three-way merge of base, incoming and local, matching
blocks by uuid (the `@logseq/diff-merge` library), on by default (`:feature/enable-sync-diff-merge?`).
Without a base it merges against an empty one. Users were told not to combine Logseq Sync with
iCloud, Dropbox or Syncthing, and forum reports of overwritten edits ("newer first" by file time)
were common before the merge landed. Collaboration on one graph was never supported.

**DB version.** Change is DataScript transactions, each tagged with an *outliner op*: the intent
(`:save-block`, `:insert-blocks`, `:move-blocks`, `:delete-blocks`, `:create-page`,
`:upsert-property`, or a raw `:transact`). Everything else is built on those ops:

- **Sync** (called RTC, "Real Time Collaboration", in the docs and UI; the legacy RTC code was
  replaced by **db-sync** on Cloudflare Durable Objects in February 2026, pull request #12315,
  with a Node.js server for self-hosting). The server holds the graph and a `tx_log`
  (`t INTEGER primary key, tx, outliner_op`). A client sends `tx/batch` with `t-before`, which
  "must equal the server's current t"; otherwise the server answers `tx/reject` with the current
  t, and the client must "pull/rebase after stale rejection instead of retrying the same batch
  blindly". Rebasing (ADR 0010) reverses the pending local transactions, applies the remote ones,
  then **replays the local ops by intent** against the new state, dropping a whole user action if
  any of its ops no longer makes sense ("Rebase reasons about user intent instead of datom
  accidents"). Presence (who is editing which block) goes over the same WebSocket. A checksum over
  each block's uuid, title, name, parent and page (ADR 0006) is compared when both sides claim to
  be at the same t, and a mismatch fails fast. Graphs are end-to-end encrypted by default; only
  `:block/title` and `:block/name` are encrypted, so "orders, tags, property values, and refs
  remain plaintext to the server".
- **Undo** (ADRs 0002, 0011): only local transactions enter the undo stack (`:local-tx? true`);
  remote changes never do. Undo replays persisted inverse semantic ops, and is "best effort": if a
  remote change (a deleted parent, a move) makes an undo unsafe, it is dropped and the stack
  cleared rather than producing invalid data.
- **History in the data:** every node has `created-at` and `updated-at`; `created-by-ref` and
  `deleted-by-ref` record who; properties with `:logseq.property/enable-history?` (Status is one)
  keep a history entity per change (block, property, value), which replaced OG's logbook.
  Deletion is mixed by kind (ADR 0009): ordinary pages and blocks go to a Recycle for 30 days;
  tags, properties and choices are retracted outright.
- **Backups:** desktop keeps the last 12 automatic `db.sqlite` copies; `logseq graph backup`
  manages them from the CLI.

I found no per-node version history (an older text of a block) in the DB version's docs; the
history is per property and the sync log (not verified further).

## Extensibility and agent access

- **Plugin API** (both models, `@logseq/libs`): `logseq.Editor` (`getBlock`, `insertBlock`,
  `updateBlock`, `upsertBlockProperty`, `getBlockProperties`, and for DB graphs `createTag`,
  `addBlockTag`, `addTagProperty`, `addTagExtends`, `upsertProperty`, `getTagObjects`),
  `logseq.DB` (`q` for a simple query, `datascriptQuery` for Datalog with inputs, `onChanged` and
  `onBlockChanged`, which hand a plugin the changed blocks and the transaction's datoms). Plugin
  properties in DB graphs are namespaced (`:plugin.property._api/…`). In the browser, only plugins
  without "effects" are allowed.

  ```js
  const books = await logseq.DB.datascriptQuery(`
    [:find (pull ?b [:block/uuid :block/title])
     :where [?b :block/tags ?t] [?t :block/title "Book"]]`)
  logseq.DB.onChanged(({ blocks, txData }) => { /* react to edits */ })
  ```

- **HTTP API server** (desktop, both models): `POST http://127.0.0.1:12315/api` with a bearer
  token and `{"method": "logseq.Editor.insertBlock", "args": [...]}`; any plugin method can be
  called this way.
- **MCP:** the DB version's desktop app serves an MCP endpoint at `/mcp` on the same HTTP server
  (tools `listPages`, `getPage`, `listTags`, `listProperties`, `search`, `upsertNodes` with batched
  operations and a dry-run "pretend" mode; changes are undoable in the app). The hosted db-sync
  service exposes a **semantic REST API** under `/api/v1` (39 operations: pages, blocks and trees,
  typed properties, tasks, tags, assets, search; graph-scoped personal access tokens, 120 reads
  and 20 writes a minute) and an MCP server and ChatGPT app over it. The engineering guide is firm
  that both go through outliner semantics: "It must not implement mutations as arbitrary raw
  DataScript transactions", and "Do not create a second independent mutation API for MCP tools".
- **CLI** (DB version, `logseq`): `graph create|export|import|validate|backup`, `list`,
  `search`, `show`, `query --query <edn>` (and named queries in `cli.edn`), `upsert
  block|page|task|tag|property|asset`, `move`, `remove`, `sync start|status|upload|download`,
  `debug pull`, `--output json|edn`, `--dry-run` on creates. `logseq skill install` writes an
  agent skill to `.agents/skills/logseq-cli/SKILL.md`. Example:
  `logseq upsert block --graph demo --target-page Home --blocks $'- Parent\n  - Child'`.
- **AgentBridge** (CLI, 2026): routes tasks in a graph to coding agents; a `#Task` with Todo
  status, a matching assignee and no `:logseq.property.agent/session-id` is dispatched to a master
  agent session, which writes the child session id back on the task. Its own guide notes that the
  "only the master writes" rule is a prompt, not enforced.
- **Scripting:** `nbb-logseq` scripts can read and, for DB graphs, write a graph's database.

OG's agent access is weaker: the files themselves (an agent can write Markdown, and the watcher
picks it up), the plugin API and the HTTP server; the old `@logseq/cli` npm package could read
graphs.

## Against fictty

fictty's model, for comparison: a **UI value** an agent pushes and patches (nodes with ids, a
closed set of components), a **store** of named entries (each a JSON value or a caretline text
with its undo history, each with a `rev` and a `by`), **view state** per node id (selection,
folds, carets, scroll, owned by the person), and **sources** (commands, streams, host data) that
are refetched and disposable. Agents write over a socket (`push`, `patch`, `put`, `merge`, `edit`,
`--if-rev`) and read the screen back (`screen --nodes`, `get --state`).

| | Logseq OG (files) | Logseq DB version | fictty |
|---|---|---|---|
| **Unit** | a block: a list item in a Markdown file, properties as `key:: value` text | a node (block or page): a DataScript entity with typed property attributes | a store entry: one whole JSON value or one text; the UI value as a separate whole |
| **Identity** | page names (unique, lower-cased); block uuids only once written as `id::` | uuid on every node, `:db/id`, `:db/ident` with a random suffix for schema | the entry's key (a readable string, `note/3`); node ids for view state; a list's `key` field for rows |
| **References** | `[[page]]`, `#tag`, `((uuid))`, embeds, in the text; backlinks from `:block/refs` | `[[ ]]` to any node by uuid, shown by title; refs and tags as ref attributes | binds by key and path, `@id` for another list's selected row, `doc` templates naming an entry; no references between entries, no backlinks |
| **Schema** | none; property values are text | typed properties, closed choices, cardinality, classes with inherited properties, Malli validation, schema version 65.35 | none in the store; each component says what shape it reads, and a mismatch is a `problems` sentence |
| **Storage** | files are the truth; DataScript a derived cache, re-indexable | SQLite `kvs` rows of DataScript storage; one writer daemon per graph; Markdown a derived mirror | in memory; `--store file.json` written atomically after each change, one screen per store file |
| **Queries** | Datalog and a simple-query DSL; advanced queries can carry ClojureScript `:view` functions | the same, renamed, with built-in rules; views (table, list, gallery) over results | none: templates, paths, `filter`, `when`, built-in sources; computation happens in commands outside |
| **History** | in-memory undo; git, `logseq/bak`, sync version files | tx log on the server; undo of local actions by inverse ops; per-property history; created/updated/deleted by and at; Recycle | `rev` and `by` per entry; a text's undo history in the entry; `$log` of agents' writes; the state is replayable |
| **Concurrency** | per file; sync merges three ways by block uuid; no co-editing | one ordered log per graph: `t-before` must equal the server's `t`, else reject, pull, rebase by replaying intent | one process; `--if-rev N` per entry refuses a stale write and returns the entry; the agent decides what to do |
| **Agent access** | write files; plugin API; local HTTP `/api` | CLI with JSON output and an agent skill; local HTTP and MCP; hosted semantic REST and MCP with scoped tokens; AgentBridge | the socket and CLI: write the store, push and patch the UI, press keys, read the screen node by node and the state as JSON |

### What fictty could take from it

- **Writes as intents, from a small closed set.** db-sync's core lesson is that merging and
  undoing raw datoms kept failing ("the helper set keeps growing with special cases"), and
  replaying a handful of canonical ops (insert, save, move, delete, transact) fixed it. fictty's
  `Write` is already small (`Put`, `Merge`, `Edit`, `Delete`), and its store actions (`append`,
  `remove`, `toggle`) are intents. When fictty adds the planned path writes and per-record
  revisions, it should keep them intent-shaped (append a row, remove the row with key k, set field
  f of row k) rather than "replace this JSON Pointer", so a stale write can be rebased by meaning
  instead of refused.
- **A stable id on every record from birth, never derived from its text.** OG made uuids real
  only when someone referenced a block, and identified pages by name; the DB version had to make
  identity universal. fictty's lists already key rows by `id` and actions can mint `{$id}`; it
  could make that the rule for any collection an agent or the person adds to, so selection, `doc`
  entries (`note/{id}`) and future references never hang on a title.
- **Granularity is what makes concurrency possible.** OG could merge because it matched blocks by
  uuid inside a file; the DB version checks parity over block attributes. fictty's revision is per
  entry, so two agents editing different rows of one list collide. The plan already names the fix
  (collections keyed by id, a revision per record); Logseq is evidence it is the right one.
- **Only the person's own actions in their undo.** ADR 0002 keeps remote changes out of the local
  undo stack and makes undo best-effort when the world moved. fictty deliberately puts an agent's
  text edit into the person's undo ("one undo takes it back"); that is a choice worth keeping
  explicit, with Logseq's rule as the alternative if several agents write at once.
- **Derived files must never feed back.** The Markdown Mirror is written atomically, ignored by
  every reader and documented as not a source. If fictty ever writes a store out as Markdown or
  CSV for other tools, the same rule applies.
- **Dry runs and who-did-what in the data.** The CLI's `--dry-run` and MCP's "pretend" let an agent
  see what a write would do; `created-by-ref` and property history record authorship. fictty has
  `by` per entry and `$log`; a dry-run reply for `put`, `merge` and store actions would be cheap.
- **Ship the agent instructions with the tool.** `logseq skill install` puts a skill where agents
  look. fictty's `fictty-ui` skill lives outside the repo; a `fictty skill` command would keep it
  in step with the binary.

### What it should not take

- **Code in queries.** Advanced queries' `:view` and `:result-transform` run ClojureScript from
  the data. That breaks "data, never code", and it is where a pushed UI would stop being safe to
  accept from an agent.
- **Two sources of truth.** OG's file-plus-cache design needed re-indexing, a watcher, backup
  folders and a three-way merge to keep them in step, and still lost edits. fictty's store file is
  persistence of one truth, locked to one screen; it should stay that way, and anything else
  written out should be a mirror.
- **Data inside prose.** OG's `key:: value` lines mixed the record into the text: values could
  not hold newlines, commas meant lists for some keys and not others, and quoting changed meaning.
  fictty keeps texts and values as separate entries, which is cleaner.
- **Opaque identifiers on the agent's path.** The DB version's property idents
  (`:user.property/rating-Zp0aW7cd`) and the REST API's "UUIDs only" make an agent look up every
  name before writing. fictty's readable keys are one of its agent-friendly traits; internal ids,
  if they come, should not replace names in the socket protocol.
- **An ontology in the store.** Typed properties, classes, inheritance and Malli validation are
  right for a personal database that lasts years; for fictty, where components declare the shape
  they read and agents produce the data, a schema layer would duplicate what the components
  already say.

### Trade-offs

The file model bought portability and trust (the notes are files you own, readable without the
app) and paid with consistency: uuids that came and went, renames that rewrote many files, sync
that merged text, and no typed data. The DB model bought consistency, types, real-time
collaboration and fast queries, and paid with portability (the truth is a SQLite file of B-tree
rows), with an agent API that has to go through outliner semantics to be safe, and with the
migration itself.

That migration is the lesson. It took three years from the first pull request to a 2.0 beta;
a DB schema that has reached version 65.35; an importer with stated limits (several queries or embeds in one block
keep only the first; task keywords remapped; flashcard history dropped; Org mode dropped; old
advanced queries rewritten by hand); changed syntax (`(())` to `[[ ]]`, `TODO` to a Status
property); and, in the end, two products, because "Every feature, bug fix, and UX change needs to
be considered twice". Markdown, the original source of truth, became a one-way mirror, and the
team's own roadmap lists two-way Markdown sync as still "under research". The decision of what
the truth is, and at what granularity it is identified and versioned, is the one that is hardest
to change later. fictty has made it early (a store of entries, one truth, files as persistence),
and the next step it is planning, records with ids and revisions inside entries, is the step
Logseq found it could not retrofit cheaply.

## Sources

- Logseq repository (DB version): https://github.com/logseq/logseq
- DB schema: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/schema.cljs
- Properties and types: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/property.cljs,
  https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/property/type.cljs
- Classes: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/class.cljs
- Idents: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/db_ident.cljc
- Query rules: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/frontend/rules.cljc
- SQLite storage: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/common/sqlite.cljs
- Build EDN: https://github.com/logseq/logseq/blob/master/deps/db/src/logseq/db/sqlite/build.cljs
- DB version docs: https://github.com/logseq/docs/blob/master/db-version.md
- Changes in the DB version: https://github.com/logseq/docs/blob/master/db-version-changes.md
- ADRs (sync, undo, rebase, checksum, deletion, Markdown Mirror): https://github.com/logseq/logseq/tree/master/docs/adr
- Runtime and engineering guide (2026-08-24): https://github.com/logseq/logseq/blob/master/docs/agent-guide/implemented/architecture/2026-08-24-logseq-runtime-and-engineering-guide.md
- CLI: https://github.com/logseq/logseq/blob/master/docs/cli/logseq-cli.md
- Semantic REST API skill: https://github.com/logseq/logseq/blob/master/.agents/skills/logseq-rest-api/SKILL.md
- Desktop MCP server: https://github.com/logseq/logseq/blob/master/src/electron/electron/mcp_server.cljs
- Plugin API types: https://github.com/logseq/logseq/blob/master/libs/src/LSPlugin.ts
- "Sync with Cloudflare DO" (db-sync replaces RTC): https://github.com/logseq/logseq/pull/12315
- "Database version": https://github.com/logseq/logseq/pull/9858
- Releases 2.0.1 and 2.0.2: https://github.com/logseq/logseq/releases
- "Big update: Logseq is splitting into two versions" (24 April 2026): https://logseq.io/p/e3YDyX5AYr
- Logseq OG repository and file schema: https://github.com/logseq/og,
  https://github.com/logseq/og/blob/version/file/deps/db/src/logseq/db/schema.cljs
- OG file sync and merge: https://github.com/logseq/og/blob/version/file/src/main/frontend/fs/sync.cljs,
  https://github.com/logseq/og/blob/version/file/src/main/frontend/fs/diff_merge.cljs
- File-graph docs: https://github.com/logseq/docs/blob/master/pages/Properties.md,
  https://github.com/logseq/docs/blob/master/pages/Block%20Reference.md,
  https://github.com/logseq/docs/blob/master/pages/Advanced%20Queries.md,
  https://github.com/logseq/docs/blob/master/pages/Logseq%20Sync.md,
  https://github.com/logseq/docs/blob/master/pages/Local%20Http%20server.md
- Forum, sync conflicts: https://discuss.logseq.com/t/logseq-sync-content-got-overwritten-when-edit-without-fully-synced/15041,
  https://discuss.logseq.com/t/can-i-delete-the-contents-within-the-bak-folder/8068
