# Roam Research: the data model

Roam Research is a hosted outliner whose graph is a DataScript database: an in-memory store of
`[entity attribute value]` facts with Datomic's query and pull languages, loaded whole into each
client. Every page and every block is an entity. A block's text is a single string in Roam's own
Markdown, and links, tags, attributes and block references are written into that string and parsed
out into reference attributes, so the text is what people edit and the graph is derived from it.
Roam publishes the schema it uses (the "Data Model" page of its developer documentation, itself a
public Roam graph), lets code query it with Datalog from the browser, a desktop HTTP API, a hosted
HTTP API and, since 2026, an official MCP server, and keeps a server-side record of every hosted
graph's past from which a graph can be opened "exactly as it was at any past moment". This note
describes the model as Roam documents it, says what I could not verify, and sets it against
fictty's UI value and store. Checked against the sources on 17 October 2026.

- **Maker:** Roam Research, Inc. (founded by Conor White-Sullivan); proprietary, subscription
  only (Pro at $15 a month or $165 a year, Believer at $500 for five years)
- **Kind:** networked-thought outliner and personal knowledge base; web app, desktop app (macOS,
  Windows, Linux), native mobile app; real-time multiplayer on hosted graphs; extensions in
  JavaScript and ClojureScript
- **Storage:** hosted graphs on Roam's servers ("Google Firebase", per the help graph), loaded in
  full into a DataScript database in each client; encrypted hosted graphs; local (offline) graphs
  kept on one device; exports in Markdown, JSON, EDN and MessagePack
- **Unit of data:** the block, an entity with a `:block/uid` (nine random characters), a
  `:block/string`, an order among its siblings and its children; pages are entities with a unique
  `:node/title` and children but no string
- **Source of truth:** the hosted graph on Roam's servers, of which every client holds a full
  replica; within the graph, each block's string is the truth for its references, and
  `:block/refs`, backlinks and attribute triples are recomputed from it

## The unit of data

The developer documentation states the model in one line: "Your graph is a DataScript database."
DataScript is Nikita Prokopov's in-memory triple store for ClojureScript, modelled on Datomic:
every fact is a datom `[e a v]` (with a transaction id that Roam says is "not useful through the
API", since DataScript keeps no history). Roam's entities are pages, blocks, users, windows, and a
few newer kinds (diagrams, attribute assertions), plus internal families it asks developers not
to build on (`:graph/*`, `:version/*`, `:token/*`, `:harc/*`, `:vc/*`, `:restrictions/*`,
`:append-api/*`).

A **block** is "an entity with a `:block/string`" that "lives under a parent block or page via
the parent's `:block/children`". Its documented attributes:

| Attribute | Type | Meaning |
|---|---|---|
| `:block/uid` | string, unique | the block's stable id |
| `:block/string` | string | the block's text, in Roam-flavoured Markdown |
| `:block/order` | number | position among siblings, from 0 |
| `:block/open` | boolean | collapsed or not |
| `:block/children` | refs, many | direct children |
| `:block/parents` | refs, many | every ancestor, from the page down |
| `:block/page` | ref | the page the block lives on |
| `:block/refs` | refs, many | every page or block this block references |
| `:block/heading`, `:block/text-align`, `:children/view-type`, `:block/view-type` | optional | display |
| `:block/props` | map, optional | per-block component state (image sizes, slider positions, query and table settings); "treat as internal" |
| `:create/time`, `:create/user`, `:edit/time`, `:edit/user` | ms since epoch, user ref | who made it and who last changed it, when |
| `:edit/seen-by`, `:create/seen-by` | refs, many | users who have seen it since its last edit (unread markers) |
| `:ent/emojis` | vector | reactions |

The documentation's own example, a block pulled with every attribute:

```javascript
window.roamAlphaAPI.data.pull("[*]", [":block/uid", "6LMkuKL-Y"])
// => {":db/id": 16810,
//     ":block/uid": "6LMkuKL-Y",
//     ":block/string": "[[Developer Notes]] 🧑‍💻",
//     ":block/order": 1, ":block/open": true, ":block/heading": 3,
//     ":block/children": [{":db/id": 16811}],
//     ":block/parents": [{":db/id": 16287}, {":db/id": 16289}, {":db/id": 16609}, {":db/id": 16804}],
//     ":block/page": {":db/id": 16287},
//     ":block/refs": [{":db/id": 17691}],
//     ":create/user": {":db/id": 1}, ":create/time": 1692372881926,
//     ":edit/user": {":db/id": 1}, ":edit/time": 1692372881926}
```

A **page** is "an entity with a unique `:node/title`"; "its content lives in `:block/children`.
There is no `:block/string`." Pages carry the same create and edit metadata, plus rollups Roam
maintains for them: `:page/edit-time` and `:page/edit-user` (the latest edit anywhere on the page,
which powers "recently edited") and `:page/word-count`. A daily note page also carries `:log/id`,
which the docs say to treat only as a flag. A **user** is an entity with `:user/uid`,
`:user/display-name`, `:user/email`, `:user/color` and `:user/settings`, referenced from every
block's `:create/user` and `:edit/user`.

Two things follow from this shape. First, the unit of editing is small: a block is usually one
line or paragraph, and an outline of a hundred lines is a hundred entities. Second, the content is
a string, not a structure. Bold, links, tags, `{{components}}` and attributes are all characters
in `:block/string`, parsed when the block is shown or saved; the attributes of the entity are the
outline's skeleton and its bookkeeping, not the text's formatting.

**View state lives in the graph too.** `:block/open` (whether a block is folded) is a block
attribute, so it is shared by everyone who edits the graph. Filters and the settings of
linked-reference panels are kept in **window** entities whose `:window/id` embeds the viewing
user, the kind of window and its target (`"<user-uid>-mentions-page-<page-uid>"`), with
`:window/filters` and `:window/mentions-state`; this is why filters are per user. Component state
(a slider's position, a table's settings) is in `:block/props`. The graph is therefore the store
for data, for per-user view state and for shared view state alike, separated by naming
convention rather than by owner.

## Identity and references

**Uids.** Every block and page has a `:block/uid`: "random 9-character strings", generated by
`roamAlphaAPI.util.generateUID` (the import schema says Roam uses nanoid). "uids can technically be
any length and they don't have to be random, but some extensions might rely on it being 9
chars." Daily note pages are the exception: their uid is the date, `MM-DD-YYYY` (`10-17-2026`), and
their title is the long form (`October 17th, 2026`), so a client can address today's page without
looking it up. The internal `:db/id` is "stable within a live graph, but not across export/import
— persist `:block/uid`s, never entity ids."

**Lookup refs.** Anything with a unique attribute can be addressed by it wherever an entity id is
accepted: `[:block/uid "abc123xyz"]`, `[:node/title "My Page"]`, `[:user/uid "…"]`,
`[:window/id "…"]`. A pull by lookup ref is direct index access, which the docs recommend over a
query whenever the entity is known.

**Page titles are identity.** `:node/title` is unique, and a page reference in a block is the
title written in the text: `[[Project Apollo]]`, `#tag`, `#[[two words]]`, `Status:: Active`
(which links the `Status` page). The parsed `:block/refs` hold entity references, but the string
holds the name. So renaming a page has to rewrite text: the help graph's FAQ shows that renaming a
page to a title that already exists offers to **merge** the two pages, and the official MCP's
change log (0.11.0, 26 August 2026) says that deleting a page de-links its referrers "in place —
`[[Page Name]]` becomes plain text `Page Name`", including references in other pages' titles. That
references to a renamed page are rewritten to the new title is long-standing behaviour, and the
change log's fixes ("renaming a page to an emoji, previously it broke tag references") imply it,
but I did not find it stated in the current API documentation. The API refuses to rename onto an
existing title (`data.page.update` "rejects when renaming to a title that already exists").
Namespaces are only a convention in titles (`Project/Apollo`), not a structure.

**Block references** are `((uid))` in the text: a live view of another block's text, so editing
the original changes every reference. Variants are also text: `{{embed: ((uid))}}` (the block with
its children, editable in place), `{{embed-path: ((uid))}}` (with its ancestors shown),
`{{embed-children: ((uid))}}`, and an alias `[label](((uid)))`. A reference can be "replaced with"
the original (the block moves to the reference's place and leaves a reference behind), with plain
text (the link is cut), or with text and an alias.

**Refs are derived and uniform.** The Data Model page: "`[[links]]`, `#tags`, `attribute::`, and
`((block refs))` all create identical `:block/refs` datoms — the syntax only changes rendering.
Check refs; don't parse strings." Backlinks are the reverse attribute, `:block/_refs`, which
DataScript indexes for free:

```javascript
// backlinks: every block referencing a page
window.roamAlphaAPI.data.q(
  `[:find ?s
    :in $ ?title
    :where [?page :node/title ?title]
           [?b :block/refs ?page]
           [?b :block/string ?s]]`,
  "My Page")
```

Backlinks are never stored as such: they are an index over refs, which are themselves recomputed
from strings. Comments use the same machinery: a comment thread is ordinary blocks under a
collapsed `[[roam/comments]]` block at the bottom of the commented block's page, and each thread
starts with a `((ref))` to the block it discusses, which is how the block shows a reference count.

## Structure and hierarchy

A graph is a forest. Pages are the roots; blocks nest under a page or another block through
`:block/children`, with `:block/order` giving the sibling position (0-indexed, recalculated when a
sibling is deleted). Each block also carries `:block/page` and `:block/parents` (every ancestor),
denormalised so that "all descendants of a block" is one clause rather than a recursion:

```clojure
[:find ?uid
 :in $ ?ancestor-uid
 :where [?a :block/uid ?ancestor-uid]
        [?d :block/parents ?a]
        [?d :block/uid ?uid]]
```

Order is a 0-based integer per sibling, not a fractional key, so as far as the API shows,
inserting at position 0 renumbers the siblings after it; the write API takes `order` as a number or `"first"`/`"last"`, and
`data.block.reorderBlocks` takes the whole list of a parent's children in their new order.

**Daily notes** are pages like any other, distinguished by the date uid and `:log/id`. They are
where Roam expects writing to start, and the APIs treat them specially: the backend API can
create a block at `{"page-title": {"daily-note-page": "07-29-2023"}}`, the Append API and MCP have
"append to today's daily note", and the `{between: …}` query clause and the `:q` symbols
(`ms/today-start`, `in-dnp-between`) work on daily-note dates. A `:q` block on a daily note page
resolves "today" to that page's day, not the calendar's.

**Attributes** (`Status:: Active`) are typed relationships written as text. Two models are
documented, and the documentation itself is in transition:

- The Data Model page says the triples "live in `:entity/attrs` and `:attrs/lookup`", explains that
  model on the "Attributes Data Model" page, and warns that it "will be removed and replaced by a
  new attributes system, so don't build on it". In that model each described entity holds a set
  of `[e a v]` triples whose positions are `{:source <block> :value <node-or-string>}` maps, and a
  flat `:attrs/lookup` ref list makes them findable.
- "Attributes Data Model (new)" describes the replacement: each assertion is its own entity, a
  **harc** (hyperedge-arc) with a `:block/uid` and six many-cardinality refs, `:harc/e`,
  `:harc/a`, `:harc/v` (the triple) and `:harc/e-source`, `:harc/a-source`, `:harc/v-source` (the
  blocks each position was derived from). The attribute is always a page; an inline text value
  becomes an owned text entity `{:block/uid "v-<harc-uid>" :harc/v-string "Active"}`; attributes
  nested under an attribute block describe the relationship itself (their `:harc/e` is the parent
  harc), so statements about statements are plain graph walks.

The new page is explicit that harcs are **derived data**: "Roam recomputes them from the source
blocks whenever those blocks change. There is no API for writing them (for now). To change a harc,
edit the blocks it derives from." It also explains why Roam does not map attributes onto
DataScript's own attributes: an edge must be an addressable node, attribute names must be pages
that can be linked and renamed, value type and cardinality are decided per instance by what was
typed, native datoms carry no provenance, and a schema keyword per user-coined name would explode
the schema. The `:harc/*` family is also listed under "Internal" on the Data Model page; I could
not confirm whether a given graph today runs the old model, the new one, or both.

Querying the new model uses reverse refs:

```javascript
// every attribute of a page, with its values
window.roamAlphaAPI.data.pull(
  "[{:harc/_e [{:harc/a [:node/title]} {:harc/v [:block/uid :node/title :block/string :harc/v-string]}]}]",
  [":node/title", "Project Apollo"])
```

There is no other schema. The DataScript schema (which attributes are refs, which are unique, which
are many-valued) is fixed by Roam; users add structure only through pages, tags and attributes,
which are data.

## Storage and the source of truth

**In the client.** Opening a graph loads it entirely into an in-browser DataScript database, and
every read the front-end API offers (`q`, `pull`, `pull_many`, `search`) runs against that local
copy, synchronously, with a 20-second timeout. The help graph's warning about page sharing shows
how literal "entirely" is: "Sharing individual pages loads the rest of your graph's data in every
visitor's browser. Roam only displays the page you shared, but a technically skilled visitor could
dig out the others." Expensive queries can be sent off the main thread with `data.backend.q`,
which runs on Roam's servers and "can be a few changes behind the frontend while local changes are
still syncing".

**On the server.** Hosted graphs live on Roam's servers; the help graph's "Hosted Graph" page says
"Google Firebase". The backend API is sharded by graph: `api.roamresearch.com` redirects each
request to the peer machine holding that graph (`peer-N.api.roamresearch.com`), which is why its
clients must follow redirects with the token attached. How the server stores a graph, and whether
it keeps the graph as a log of transactions, is not documented; "Graph History" (below) shows that
it keeps enough to reconstruct any past moment.

**Graph kinds.** A **hosted** graph syncs across devices and can be shared (private, read or edit
by named people, readable or editable by anyone, or single pages). An **encrypted** hosted graph is
encrypted on the client; the server can store and relay it but not read it, so the backend API and
the hosted MCP cannot read it either (they can append to it), and some metadata stays in the clear
(the encrypted keys are listed: `:node/title`, `:block/string`, `:block/props`, `:entity/attrs`,
user names and emails among them; headings and alignment are not). A **local** or offline graph
never leaves the device: in the browser it lives in browser storage (lost if site data is cleared,
hence Roam's push for daily automatic backups), and in the desktop app on disk.

**Exports and imports.** "Export All" offers Markdown, JSON, EDN and MessagePack, plus automatic
backups. JSON is a tree of pages and blocks; the import schema is:

```clojure
[{:title        "December 10th 2018"
  :create-time  1576025237000
  :children     [{:string   "[[Meeting]] with [[Tim]]"
                  :children [{:string "Meeting went well"}]}
                 {:string "[[Call]] with [[John]]"}]}]
```

Pages take `title`, `children`, `create-time`, `edit-time`, `edit-user`; blocks take `string`,
`uid`, `children`, the times, `edit-user`, `heading` and `text-align`. Order is the array's order.
A title that already exists merges into the existing page; a `uid` that clashes used to fail the
import (a 2026 change log entry says conflicts are "now automatically remapped instead of failing
the import"). Exported JSON has historically used `create-email` and `edit-email` for authors. EDN
is the high-fidelity format the help graph recommends for moving a local graph, and it and
MessagePack are what **Restore Graph** reads; I understand EDN to be a dump of the DataScript
database's datoms, but I did not inspect a current export to confirm its exact shape. Markdown
exports resolve block embeds to text.

So the source of truth has two layers. Between machines it is the hosted graph, with every client a
full replica. Within the graph it is the block strings: refs, backlinks and attribute triples are
recomputed from them, and an export to Markdown or JSON carries only strings and the outline, from
which an import re-derives the rest.

## Queries and derived views

**Datalog.** `roamAlphaAPI.data.q` takes a Datomic-flavoured Datalog query as a string, with `:in`
parameters, and returns tuples; `data.pull` takes a pull pattern and one entity. Pull patterns
nest and recurse, which is how a client reads an outline:

```javascript
// a page's top-level blocks, with text and order
window.roamAlphaAPI.data.pull(
  "[{:block/children [:block/uid :block/string :block/order]}]",
  '[:node/title "My Page"]')

// a block and all its descendants
window.roamAlphaAPI.data.pull("[:block/string {:block/children ...}]", '[:block/uid "xyz"]')
```

The docs give performance advice that is really advice about the model: anchor the first
`:where` clause on a constant or lookup ref, because unbound clauses scan an index; put predicates
last; prefer reverse-ref pulls to joins for backlinks and parents; narrow pull patterns rather
than pulling `[*]` with recursive children. Roam adds its own symbols and rules for use in queries
(`current/page-title`, `current/block-uid`, `ms/this-month-start`, `in-or-refs-dnp-between`),
exposed to `data.q` since March 2026; the backend API does not get them.

**Query blocks.** Two kinds of query live in the graph as blocks:

- `{{[[query]]: {and: [[Project]] {not: [[Archived]]}}}}` is Roam's own query language, with
  `and`, `or`, `not`, `search`, `between` (daily-note dates), `daily-notes`, and `by`,
  `created-by`, `edited-by`. A page or block in a condition matches everything that links to it,
  and **references are inherited down the outline**: "a block counts as mentioning everything its
  parent blocks mention, and the page it lives on", so `{and: [[Acme]] [[pricing]]}` matches a
  block about pricing nested under one that mentions Acme. Results can be grouped by page, shown
  with paths, sorted, and nested under parent results. Queries do not re-run on their own unless
  "Reactive" is turned on (off by default since May 2023, so that open queries no longer slow
  large graphs down). The same
  query runs from code with `data.roamQuery`.
- `:q [:find ?p :where [?p :node/title]]` is a Datalog query block (revamped in December 2024 and
  March 2025), with an optional title, the query in a referenced code block, pagination, sorting,
  a five-second timeout, and since 16 October 2026 generation from a description by AI.

**Linked and unlinked references.** Every page shows its **linked references**, the blocks whose
refs include it (with the same path inheritance, filters by other pages in the path, and per-user
filter state in a window entity), and its **unlinked references**, blocks whose text mentions the
title without linking it, which can be linked in one click. Linked references are an index over
refs; unlinked references are a text search; neither is stored.

**Watches.** `data.addPullWatch(pattern, entity, callback)` calls back with the pulled value before
and after each change to it (debounced by about 100 ms), which is how extensions react to edits
without polling.

Everything in this section is derived: query results, backlinks, unlinked mentions, page rollups
and attribute triples are computed from the one copy of the strings and the outline, and Roam
never asks anyone to keep them in step by hand.

## Change, history and sync

**Writes.** The write API is a small closed set of actions, the same in every surface (front end,
local API, backend API): `create-block`, `update-block`, `move-block`, `delete-block`,
`create-page`, `update-page`, `delete-page`, and in the backend API `batch-actions`. An update
changes only the keys passed, and a block's text is replaced whole (`"string"`); there is no
range edit. A delete removes the block and its children and returns `{"deleted": true}` or
`{"deleted": false, "reason": "not-found"}` (a change of 25 August 2026, made because agents
should be able to tell a real delete from a no-op). Markdown can be parsed into blocks
(`block.fromMarkdown`, `page.fromMarkdown`), which the docs recommend for adding a lot at once.

```json
{"action": "batch-actions",
 "actions": [
   {"action": "create-page",  "page": {"title": "Batch action test page", "uid": -1}},
   {"action": "create-block", "location": {"parent-uid": -1, "order": "last"},
                              "block": {"string": "Second", "uid": -2}},
   {"action": "update-block", "block": {"uid": -2, "string": "Second (updated)"}}]}
// => {"tempids-to-uids": {"-1": "EBiw8LzPb", "-2": "X4DelKvsP"}}
```

Negative integers are **tempids**, usable across the batch and mapped to real uids in the reply.
A batch is ordered but **not atomic**: "if an action fails, the actions before it stay
transacted and the rest are skipped", and the error says how many succeeded.

**No conditional writes.** None of the documented write actions takes a revision, an expected
value or an edit time to compare against, so two writers to the same block cannot detect each
other through the API; the official MCP's syntax guide asks agents to fetch a block's full text
before updating it, but only to avoid truncation and broken references, not races. The nearest
thing to an ownership rule is the "Immutable blocks" sharing setting: "editors can only change the
blocks they created themselves".

**Edit metadata.** Every block and page carries `:create/time`, `:create/user`, `:edit/time` and
`:edit/user`; pages add rollups of the latest edit anywhere beneath them. The front end can
attribute a write to another user with `user-uid` (officially only from AI tools), and since
February 2025 queries can select blocks created or edited by a person or by an API token
(`{by: [[API Token: Baibhav's Raycast]]}`). The hosted MCP writes as a distinct "(AI)" author per
connection, so "a query like `{{query: {by: [[Your Name (AI)]]}}}` collects everything an assistant
has written". Only the latest editor is kept per entity; there is no per-entity revision count.

**Real-time multiplayer.** Hosted graphs are edited by several people at once, with presence
colours (`:user/color`) and unread markers (`:edit/seen-by`). Roam does not document its sync
protocol or how it resolves two clients changing the same block at once; I could not verify
whether it is last-writer-wins per attribute, operational transformation or something else, and
I would not assume more than "the server orders the writes". What is documented: the front end's
write promises resolve "once Roam has applied the operation" locally, the backend can lag local
changes, and large pastes once caused "thousands of unsynced changes" (fixed 9 June 2026).

**Offline.** A hosted graph keeps working offline and syncs when back online ("your changes won't
be synced to our servers until you're back online when the sync button becomes green"), with the
pending changes held in the browser in the meantime. Mobile Quick Capture saves offline captures
and syncs them later. The Append API is built for capture from outside: appends to the same place
are applied in order, "exactly-once" even with several clients open, stamped with the time of the
request rather than of the sync, and for an encrypted graph a 200 means "saved in Roam's servers"
and applied "the next time a full Roam client is opened".

**History.** DataScript keeps none, and the API gives none: no `as-of`, no per-block version list.
What Roam offers instead:

- **Undo** in the session (`data.undo`, `data.redo`, the same as ⌘Z).
- **Graph History** (Settings → Graph): open a hosted graph "exactly as it was at any past moment,
  all the way back to the day it was created", read-only ("Graph is in history mode"), with search,
  to copy lost content back. "History is a view, not an undo"; local graphs "don't keep the
  server-side record it reads from"; who may look is a sharing setting. I could not find when it
  shipped or whether any API exposes it.
- **Restore Graph** from a backup or an EDN or MessagePack export, which overwrites the whole graph.
- **Block versions** (`Ctrl+,`, "Version Control" in the help graph), which are alternative drafts
  of one block that a reader can flip between, not a change history.

The official MCP's README is blunt about the gap this leaves for agents: "Roam does not have a
traditional undo history that can reverse bulk operations or deletions made through the API",
with the advice to back up first; the help graph's "Working with AI Tools" points at Graph History
"if something disastrous happens". A request for per-page version history (GitHub issue #64, May
2020) was never answered before that repository was archived in April 2021.

## Extensibility and agent access

Roam's surfaces, from inside the app outwards:

- **`window.roamAlphaAPI`**, "provided in the browser, letting you read and write data and drive
  the UI" (`apiVersion` 1.1.3 in its example; "no longer really alpha — we just can't change the
  name"). `data.*` reads (`q`, `pull`, `pull_many`, `search`, `semanticSearch`, `roamQuery`,
  `async.*` promise versions, which Roam says to prefer), writes (`block.*`, `page.*`,
  `block.addComment`, `user.upsert`) and watches; `ui.*` drives windows, sidebars, filters, menus
  and renders Roam components into other DOM; `file.*`; `util.*` for uids and dates. Writes share a
  budget of 1,500 calls a minute.
- **Code in the graph**: `[[roam/js]]` and `[[roam/cljs]]` code blocks run as scripts, `{{roam/render:
  …}}` blocks render React or Reagent components inline, `[[roam/css]]` styles the graph. These
  are pages of the graph like any other, so code is data in the graph and runs with full access to
  the DOM and the API (encrypted graphs warn that such code sees the decrypted data).
- **Roam Depot extensions**, reviewed and installed from a marketplace, with an extension API on
  top of `roamAlphaAPI` (settings, command palette, lifecycle).
- **The Local API** (desktop app only, "just released, breaking changes might still happen"): an
  HTTP server on port 3333 or the next free one, advertised in `~/.roam-local-api.json`, which
  calls any `roamAlphaAPI` method by path: `POST /api/<graph>` with
  `{"action": "data.block.create", "args": [...]}`. It needs a local API token that a person
  approves in the app.
- **The Backend API** (`api.roamresearch.com`, since September 2022): `q`, `pull`, `pull-many`,
  `search`, `roam-query` and `write`, with graph tokens (`roam-graph-token-…`) that only the
  graph's admin can create, in read-only or read+edit roles. Quotas are per graph per minute: 500
  pulls, 200 queries and searches, 100 writes, with a batch counting as its number of actions.
  Encrypted graphs "are not supported (and will not be in the future)". Official SDKs exist for
  TypeScript (`@roam-research/roam-api-sdk`), Python, Clojure, Java and Rust.
- **The Append API** (`append-api.roamresearch.com`): one route, `append-blocks`, which appends a
  nested tree under a page (by title, created if missing) and optionally under a `nest-under` block
  (also created if missing). Append-only tokens "cannot read your graph nor can they modify
  existing blocks", work on encrypted graphs, and are limited to 30 requests a minute and 20 MB an
  hour per token.
- **MCP and CLI.** Roam publishes an official MCP server and CLI, `Roam-Research/roam-tools`
  (repository created 15 January 2026, "Alpha Software", 0.11.0 on 26 August 2026), that talks to
  the desktop app's local API, so it works on encrypted and local graphs. Its tools include
  `get_page` and `get_block` (Markdown with `<roam uid="…"/>` tags), `search`, `semantic_search`,
  `datalog_query`, `roam_query`, `get_backlinks`, `create_block`, `update_block`, `move_block`,
  `delete_block`, `create_page`, `append_to_daily_note`, `add_comment`, window and selection tools,
  and `call_extension_tool`. Access levels are `full`, `read-append` and `read-only`, enforced by
  Roam, not the client. A hosted **Roam MCP** at `https://mcp.roamresearch.com/mcp` (in Claude's
  connector directory and as a ChatGPT app) needs no desktop app, signs in with a Roam account,
  grants per-graph access levels, and can only append to an encrypted graph's daily note.
- **Guidance in the data.** A page called `[[roam/agent guidelines]]` holds a graph's conventions
  for AI tools, which the MCP returns from `get_graph_guidelines`. Blocks tagged `#.rm-hide` or
  `#.rm-private` are left out of the MCP's read tools ("a convenience filter, not a security
  guarantee", since `datalog_query` reads the database directly).
- **Extension tools for agents.** `roamAlphaAPI.ai.addTool` (experimental) registers a JavaScript
  function, with a JSON Schema for its arguments and a scope (`read`, `append`, `edit`), that
  agents connected through the local MCP can discover and call; a handler can attribute its writes
  to the calling AI with `context.tokenUserUid` or `context.asTokenUser(fn)`.

Agents therefore see Roam's model directly: the same entities, the same Datalog, the same seven
write actions, with an identity of their own in the edit metadata. What they do not get is any
way to make a write conditional on what they last read, or to undo what they did except through
the session's undo or Graph History.

## Against fictty

fictty's model, for comparison: a **UI value** an agent pushes and patches (nodes with ids, a
closed set of components), a **store** of named entries (each a JSON value or a caretline text
with its undo history, each with a `rev` that goes up on every write and a `by` naming the last
writer), **view state** per node id (selection, folds, carets, scroll; the person's, surviving a
push for every id still there), and **sources** (commands, streams, host data) that are refetched
and disposable. Agents write over a socket (`push`, `patch`, `put`, `merge`, `edit`, with
`--if-rev N` to refuse a stale write) and read the screen back (`fictty screen --nodes`,
`fictty get --state`).

| | Roam Research | fictty |
|---|---|---|
| **Unit** | the block: an entity with a uid, one Markdown string, an order and children; pages are titled roots | a store entry: one whole JSON value or one text; the UI value as a separate whole |
| **Identity** | random nine-character `:block/uid` on every block and page; daily notes keyed by date; page titles unique and used as names in text; `:db/id` internal and unstable | the entry's key (a readable string, `note/3`); node ids for view state; a list's `key` field (default `id`) for rows |
| **References** | `[[title]]`, `#tag`, `attr::` and `((uid))` in the text, parsed into `:block/refs`; backlinks as the reverse index; embeds; refs inherited down the outline in queries | binds by key and path, `@id` for another list's selected row, `doc` templates naming an entry from a row; no references between entries, no backlinks |
| **Schema** | a fixed DataScript schema owned by Roam; user structure only as pages, tags and attributes, which are data (harcs or `:entity/attrs`) | none in the store; each component says what shape it reads at its `bind`, and a mismatch is a `problems` sentence |
| **Storage** | hosted graph on Roam's servers, a full replica in every client's memory; encrypted and local variants; JSON, Markdown, EDN, MessagePack exports | in memory in one process; `--store file.json` written atomically after each change, one screen per store file |
| **Queries** | Datalog and pull from code; `{{query}}` and `:q` blocks stored in the graph; linked and unlinked references; pull watches | none: templates, paths, `filter`, `when`, `sort`, built-in sources (`$store`, `$log`, `$picked`); computation happens in commands outside |
| **History** | create and edit time and user per entity; session undo; Graph History (a read-only view of any past moment, hosted only); Restore Graph; no API for either | `rev` and `by` per entry; a text's undo history inside its entry; `$log` of agents' socket writes; the whole state is serialisable and replayable |
| **Concurrency** | real-time multiplayer with an undocumented merge; no conditional writes; batches ordered but not atomic; "Immutable blocks" as an ownership rule | one process, one ordered `update`; `--if-rev N` per entry refuses a stale write and returns the entry as it is; an agent's text edit moves the person's carets and is one undo |
| **Agent access** | `roamAlphaAPI` in the page, the desktop local API, the hosted backend and append APIs with scoped tokens, the official local and hosted MCP, extension tools; writes attributed to a distinct AI author | the socket and CLI: write the store, push and patch the UI, press keys and click, read the screen node by node and the state as JSON |

### What fictty could take from it

- **Short stable ids, separate from names, and lookup by any unique key.** Roam never lets a
  human-readable name be the only handle on a record: every block has a uid, a page's title is
  unique but its uid is what persists, and the internal id is explicitly not to be kept. fictty's
  entry keys are readable names, which suits entries, but the rows inside an entry are addressed
  by index or by whatever the list's `key` names. The planned "collections keyed by id" should make
  an id mandatory per record and treat it the way Roam treats `:block/uid`: the thing selection,
  `doc` templates and agents' writes refer to.
- **Ids made from meaning where meaning is unique.** A daily note's uid is its date, and a harc's
  text value is `v-<harc-uid>`, so two clients creating the same thing at once converge on one
  record. fictty already does this when a `doc` template names an entry from a row
  (`date/{id}`); it is worth keeping as a stated rule for anything an agent and the person might
  both create.
- **Derive, never store, what can be computed.** Roam's refs, backlinks, attribute triples and page
  rollups are all recomputed from the strings, and its docs tell developers to "check refs; don't
  parse strings" rather than keep their own copy. This is fictty's "one source of truth" rule in
  practice. When fictty gets references between entries, backlinks should be a derived index (a
  built-in source, like `$picked`) over the one place a reference is written.
- **Batch writes with tempids.** Creating a record and referring to it in the same request (`-1`
  as a parent, mapped to the real id in the reply) is what an agent needs to append a row and then
  select it or open its editor. fictty's `do` sequences store actions on keys; the socket could take
  a list of writes with tempids the same way, and, unlike Roam's, the list should apply all or
  nothing, since a half-applied batch is the hardest state for an agent to reason about.
- **Watches with before and after.** `addPullWatch` hands the callback both values. fictty's
  planned `watch` could do the same over the socket, with the `rev` and `by` of the write that
  caused it, so an agent sees what the person changed without diffing.
- **Scoped write capabilities.** Roam's append-only tokens, read-only tokens and per-graph access
  levels, and its "(AI)" author per connection, let a person connect a tool that can capture but
  not rewrite. fictty's socket is all or nothing, and `by` is a name the writer chooses (`--by`).
  If fictty ever accepts writers it did not start, a socket per scope (read, append to named
  entries, write) and a `by` the screen assigns per connection are the shape to copy.
- **Guidance kept with the data.** `[[roam/agent guidelines]]` is a page in the graph that every
  agent reads first. A fictty screen could carry the same thing as a store entry that
  `get --state` surfaces, so an agent arriving at a running screen learns its conventions from the
  screen rather than from a skill file.

### What it should not take

- **Queries and code in the data.** `{{query}}` and `:q` blocks are queries stored in the graph,
  and `roam/js`, `roam/render` and `ai.addTool` are code stored in or registered against it. They
  are why Roam is extensible, and they are exactly what fictty's "data, never code" rule forbids: a
  pushed UI may substitute values into templates, not evaluate expressions. Filtering, sorting and
  totals belong in components and recipes; anything more belongs in a command or a host source.
- **View state in the shared data.** Roam keeps fold state on the block (shared by all editors),
  component state in `:block/props`, and per-user filters in window entities named by convention.
  fictty separates these by owner (the store is shared, view state is the person's and keyed by
  node id), and that separation is what lets a push or an agent's write leave the person's folds,
  carets and selection alone. It should stay.
- **Names as references in text.** Because `[[Page]]` is a title in a string, a rename rewrites
  every referencing block, a rename onto an existing title becomes a merge, and a delete turns
  links into plain text. fictty's binds name entries by key, and a future reference between entries
  should hold the target's id, with the name shown, not stored.
- **Writes without a precondition.** Roam's write actions replace a block's string whole and cannot
  say what they were based on, so a person and an agent editing the same block cannot detect each
  other. fictty's `--if-rev` on every entry, and range edits that move the person's carets, are
  stronger and should be extended (a revision per record with path writes) rather than relaxed.
- **The whole dataset in every reader.** Roam's choice to replicate the full graph into each
  client makes queries instant and offline work possible, and also means sharing one page ships
  the whole graph. fictty runs one process per screen with nothing to replicate, and that should be
  kept until a real case needs more.

### Trade-offs

- **A general graph against a closed set.** Roam's EAV store can hold anything and answer any
  Datalog question about it, at the price of a 20-second timeout, rate limits, advice on clause
  order, and queries that are not reactive by default because reactivity made large graphs slow.
  fictty's store answers only what a bind, a path and a template can ask, which keeps every frame
  cheap and every screen readable by an agent, and pushes real computation into commands where it
  can be tested.
- **Attributes as data, not schema.** Roam rejected per-attribute schema for the reasons fictty
  has no store schema: names coined by users, per-instance types and cardinality, no schema
  explosion. Roam then had to build a reified triple (with provenance) to make attributes
  queryable. fictty gets by because components declare the shape they read; if it ever needs
  typed fields across entries, Roam's harc is a worked example of doing it without a schema.
- **History as a server-side view against history as replay.** Roam can open a hosted graph at any
  past moment but cannot undo through the API; fictty keeps a revision and a writer per entry and a
  replayable state but no time travel over the store. A read-only "store as of rev N" view, built
  from the state fictty already keeps, would give agents and people Roam's safety net for bulk
  edits without adding an undo across owners.
- **Fine-grained units against whole entries.** Roam's unit (a block) is small enough that two
  people rarely edit the same one, which is part of why it can get away with an undocumented merge.
  fictty's unit (a whole entry) is coarse, so its revision check is coarse too: two agents editing
  different rows of one list collide. Roam's granularity argues for the planned per-record
  revisions; its lack of preconditions argues for keeping the check when the unit shrinks.

## Sources

Roam's own documentation (the public `developer-documentation` and `help` graphs, read on
17 October 2026; the help graph pages were read through roamdocs.fyi, a third-party mirror of both
public graphs that serves their Markdown; the developer pages were also read in the live graph):

- Developer Hub: https://roamresearch.com/#/app/developer-documentation/page/49715b-M2
- Data Model: https://roamresearch.com/#/app/developer-documentation/page/oAdYx3oyf
- Attributes Data Model (new): https://roamresearch.com/#/app/developer-documentation/page/2ZhT2Vi46
- Attributes Data Model (old), as mirrored: https://roamdocs.fyi/developer-documentation/attributes-data-model
- Roam Alpha API: https://roamresearch.com/#/app/developer-documentation/page/tIaOPdXCj
- Roam Backend API: https://roamresearch.com/#/app/developer-documentation/page/W4Po8pcHQ
- Roam Append API: https://roamresearch.com/#/app/developer-documentation/page/eb8OVhaFC
- Local API: https://roamresearch.com/#/app/developer-documentation/page/8ikgtLSXz
- Yet-to-release updates: https://roamresearch.com/#/app/developer-documentation/page/9cnS3QNDu
- The mirror's index and full-text files: https://roamdocs.fyi/ (`/help/llms-full.txt`,
  `/developer-documentation/llms-full.txt`)
- Help pages, as mirrored: JSON Schema https://roamdocs.fyi/help/json-schema, Export
  https://roamdocs.fyi/help/export, Import https://roamdocs.fyi/help/import, Query
  https://roamdocs.fyi/help/query, Block References https://roamdocs.fyi/help/block-references,
  Linked References https://roamdocs.fyi/help/linked-references, Comments
  https://roamdocs.fyi/help/comments, Graph History https://roamdocs.fyi/help/graph-history,
  Version Control https://roamdocs.fyi/help/version-control, Hosted Graph
  https://roamdocs.fyi/help/hosted-graph, Encrypted Graphs
  https://roamdocs.fyi/help/encrypted-graphs, FAQ https://roamdocs.fyi/help/faq, Change Log
  https://roamdocs.fyi/help/change-log (also the "Roam MCP", "Local MCP", "Working with AI Tools",
  "Sharing", "Roam-specific :q additions" and "Settings" pages in `/help/llms-full.txt`)

Code and packages:

- Backend SDKs (TypeScript, Python, Clojure, Java): https://github.com/Roam-Research/backend-sdks
- `@roam-research/roam-api-sdk`: https://www.npmjs.com/package/@roam-research/roam-api-sdk
- Official MCP and CLI, README, CHANGELOG and architecture notes:
  https://github.com/Roam-Research/roam-tools
- DataScript: https://github.com/tonsky/datascript
- Datomic query and pull: https://docs.datomic.com/query/query-data-reference.html,
  https://docs.datomic.com/query/query-pull.html

Other:

- Roam Research home page (platforms, pricing): https://roamresearch.com/
- David Bieber, "Roam Research's JSON Export Format" (2020):
  https://davidbieber.com/snippets/2020-04-25-roam-json-export/
- "Version history per page", Roam-Research/issues #64 (2020, repository archived 2021):
  https://github.com/Roam-Research/issues/issues/64
- A community MCP server on the backend API, for comparison:
  https://github.com/2b3pro/roam-research-mcp

Not verified: Roam's sync protocol and how it resolves concurrent edits to one block; how the
server stores a graph and since when Graph History has existed; whether a given graph runs the
`:entity/attrs` or the harc attribute model today; the exact shape of a current EDN export; and
that renames rewrite references (implied by the change log and by the documented delete
behaviour, not stated in the API docs).
