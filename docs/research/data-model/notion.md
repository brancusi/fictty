# Notion: the data model

Notion is a single kind of record repeated billions of times. A paragraph, a to-do, an image, a
page and a database row are all *blocks*: a row with a UUID, a type, a bag of properties, an
ordered list of child ids and a pointer to its parent. Everything else (pages, databases, views,
permissions, synced content) is built by giving blocks particular types and by following those two
pointers. Since 2025 the picture has two important additions: a database is now a container of one
or more *data sources*, each with its own schema, and the text inside blocks is merged with a CRDT
instead of last-write-wins. This note reads Notion's own engineering posts, its API reference and
its help pages as of October 2026, and sets the model beside fictty's.

- **Maker:** Notion Labs, Inc. (San Francisco)
- **Kind:** a hosted, collaborative workspace (documents, wikis, databases), with a public REST API
  and a hosted MCP server
- **Storage:** server-side Postgres, sharded by workspace into 480 logical shards; on devices, a
  SQLite (or IndexedDB) record cache that offline mode turned into a guaranteed store
- **Unit of data:** the block (one record per paragraph, row, page)
- **Source of truth:** Notion's servers; clients apply edits optimistically and the server accepts
  or rejects each transaction

## The unit of data

Notion's engineering post "The data model behind Notion's flexibility" opens with "Everything you
see in Notion is a block." Every block has the same five attributes:

- **id**: a random UUID (v4). A page's id is the tail of its URL.
- **type**: how the block is shown and how its properties and content are read (`to_do`,
  `heading_2`, `page`, `toggle` and so on).
- **properties**: a map of attributes. The common one is `title`, the block's text; a database row
  adds the properties its schema defines.
- **content**: an ordered array of child block ids (a toggle's contents, nested bullets, a page's
  body).
- **parent**: the id of the block above it. The post is explicit that "the parent block is only
  used for permissions".

The type is a lens, not a conversion. Turning a checked to-do into a heading and back leaves it
checked, because the `checked` property was never removed, only not shown. Properties a type does
not read survive the round trip.

The public API shows a block in a flatter, typed form. The children are not inlined: `has_children`
says whether to ask for them.

```json
{
  "object": "block",
  "id": "c02fc1d3-db8b-45c5-a222-27595b15aea7",
  "parent": { "type": "page_id", "page_id": "59833787-2cf9-4fdf-8782-e53db20768a5" },
  "created_time": "2026-10-09T14:12:00.000Z",
  "last_edited_time": "2026-10-09T14:13:00.000Z",
  "created_by": { "object": "user", "id": "ee5f0f84-409a-440f-983a-a5315961c6e4" },
  "last_edited_by": { "object": "user", "id": "ee5f0f84-409a-440f-983a-a5315961c6e4" },
  "has_children": false,
  "in_trash": false,
  "type": "to_do",
  "to_do": {
    "rich_text": [
      { "type": "text", "text": { "content": "Order brake pads", "link": null },
        "annotations": { "bold": false, "italic": false, "strikethrough": false,
                         "underline": false, "code": false, "color": "default" },
        "plain_text": "Order brake pads", "href": null }
    ],
    "checked": false,
    "color": "default"
  }
}
```

(The shape is from the API reference; the ids and text are made up.) The type's name is also the
key of its payload, so a client reads `block[block.type]`. The block types in the API enum include
paragraph, the headings (1 to 4), the list items, `to_do`, `toggle`, `quote`, `callout`, `code`,
`equation`, `table` and `table_row`, `column_list` and `column`, `child_page`, `child_database`,
`synced_block`, the media and file blocks, `embed`, `bookmark`, `link_preview`, `breadcrumb`,
`table_of_contents` and `template`. A type the API cannot show comes back as `unsupported` with a
`block_type` string (`"form"`, say), and the reference warns that new ones appear. The 2026-03-11
version renamed `transcription` to `meeting_notes` and replaced `archived` with `in_trash`.

Text is not a string but an array of rich text objects, each a run with annotations, a link, or a
mention. Each run's `content` is limited to 2,000 characters in the API.

## Identity and references

**UUIDs everywhere.** Blocks, pages, databases, data sources, users, comments and views each have a
UUID. Ids are stable for the life of the record: moving a block changes its parent and the parent's
content array, never its id. Database ids and data source ids are distinct and, the upgrade guide
stresses, "not interchangeable".

**Two pointers per block.** `content` points down and gives order; `parent` points up and gives
permissions. Notion keeps them mirrored (the post admits a few edge cases it is still cleaning up)
and gives two reasons for not deriving permissions from `content`: a block can sit in more than one
content array, which would make inheritance ambiguous, and walking down to find ancestors is slow,
especially on the client. In the API, `parent` is a tagged object: `page_id`, `block_id`,
`database_id`, `data_source_id` or `workspace`.

**Mentions** are inline references inside rich text: a run of `"type": "mention"` whose `mention`
is a `user`, `page`, `database`, `date`, `link_preview` or a `template_mention` (`today`, `now`,
`me`). A page mention carries the page's id; its `plain_text` is the title at the time of reading,
and "Untitled" when the connection cannot see it. A database mention names the database, not a
data source.

```json
{ "type": "mention",
  "mention": { "type": "page", "page": { "id": "3c612f56-fdd0-4a30-a4d6-bda7d7426309" } },
  "plain_text": "Spring service plan", "href": "https://www.notion.so/3c612f56fdd04a30a4d6bda7d7426309" }
```

**Relations** are the typed, schema-level reference: a property whose values are page ids in a
target data source. A relation is `single_property` (one way) or `dual_property` (two way, which
creates a matching property on the other side, named in `synced_property_name`). Since the 2025
split, requests name the target by `data_source_id`; responses carry both ids. A relation value
holds at most 100 pages per API write.

**Backlinks** are derived, not stored on the target as a property: Notion shows on each page the
pages that link to or mention it. The public REST API has no backlinks endpoint that I could find;
the MCP server added `notion-list-page-backlinks` on 13 to 14 October 2026.

**Synced blocks** are the one place a single piece of content appears in more than one place. The
API's `synced_block` has a `synced_from` field: `null` on the original, the original's block id on
each copy. The help pages say an edit in any copy updates all of them, that a viewer needs access
to the original to see a copy, and that unsyncing turns a copy into an independent block. The API
can create copies but cannot update a synced block's content. Read against the data model post, a
synced copy is the case the parent pointer exists for: the content is in the tree in several
places, but it has one parent and one set of permissions. (How a copy is stored internally, as a
pointer block whose children are the original's, is my reading of `synced_from`; Notion does not
document it.)

## Structure and hierarchy

**The block tree.** A workspace is a forest of blocks under its root. The content arrays make a
"render tree": a list block draws its children indented, a toggle draws them only when open, a page
draws them on a page of their own rather than inline. Indenting is structural: Tab moves a block
into the content array of the previous sibling, and only if that sibling is a list-type block.

**Pages** are blocks of type `page`. In the API a page object has `properties` (just `title`
outside a database, the schema's properties inside one), `icon`, `cover`, `parent`, `url` and
`in_trash`; its body is its block children, read with `GET /v1/blocks/{id}/children` (paginated)
or, since February 2026, as Markdown with `GET /v1/pages/{id}/markdown`.

**Databases and data sources.** API version 2025-09-03 (announced 26 August 2025) split what used
to be one object:

- a **database** is the container: title, icon, cover, parent, whether it is inline. It lists its
  data sources.
- a **data source** holds the schema (`properties`) and the rows. A database may have several, each
  with its own schema; a row is a page whose parent is a `data_source_id`.

```json
{ "object": "database", "id": "{database_id}",
  "data_sources": [ { "id": "{data_source_id}", "name": "Workshop jobs" } ] }
```

The guide is blunt that this "is not backward-compatible": an integration on an older version fails
to create pages in, query or write relations to a database once someone adds a second data source.
Search now returns data source objects, one per source.

**Rows are pages.** A database row is a full page: it has property values *and* a block body. So the
same tree serves documents and records, and a row can hold a long write-up.

**Schemas and property types.** Each data source property has an `id` (stable across renames; the
title's is always `title`), a `name`, an optional `description`, a `type` and a settings object
keyed by the type. The types:

| Group | Types |
|---|---|
| Scalars | `title` (exactly one, cannot be removed), `rich_text`, `number` (with a display `format` only), `checkbox`, `date` (a date, a time or a range), `url`, `email`, `phone_number`, `place` |
| Choices | `select`, `multi_select` (options with `id`, `name`, `color`), `status` (options in the fixed groups To-do, In progress, Complete) |
| References | `relation`, `people`, `files` |
| Derived, read-only | `formula`, `rollup`, `created_time`, `created_by`, `last_edited_time`, `last_edited_by`, `unique_id` (a number with an optional `prefix`) |
| App-only | `button` (its actions cannot be read or pressed through the API), `verification` (wikis) |

A schema in the API looks like this (abridged):

```json
"properties": {
  "Job":      { "id": "title", "type": "title", "title": {} },
  "Status":   { "id": "Zqd%3F", "type": "status", "status": { "options": [...], "groups": [...] } },
  "Bike":     { "id": "a%40Bc", "type": "relation",
                "relation": { "data_source_id": "...", "type": "dual_property",
                              "dual_property": { "synced_property_name": "Jobs" } } },
  "Hours":    { "id": "h1", "type": "number", "number": { "format": "number" } },
  "Parts":    { "id": "p9", "type": "rollup",
                "rollup": { "relation_property_name": "Parts used", "rollup_property_name": "Price",
                            "function": "sum" } },
  "Total":    { "id": "t4", "type": "formula",
                "formula": { "expression": "prop(\"Hours\") * 60 + prop(\"Parts\")" } }
}
```

**Views** are records of their own, attached to a database: a layout (table, board, list,
calendar, timeline, gallery, chart and so on) with its filter, sorts, grouping and shown
properties. A *linked view* is a view of a database placed elsewhere in the tree. The public API
had no views until 19 March 2026, when `/v1/views` arrived with eight endpoints (create, retrieve,
update, delete, list, query and two more I did not check individually) and `view.*` webhook
events. Some reference pages still say views cannot be managed through the API, so the docs
disagree with themselves here.

## Storage and the source of truth

**Server.** Blocks live in Postgres. In 2021 Notion moved from one database to 480 logical shards
(each a Postgres schema with its own copy of every sharded table) on 32 physical databases, keyed by
**workspace id**, because every block belongs to exactly one workspace and so most queries never
cross shards. Every table reachable from `block` by foreign key is sharded with it (`space`,
`discussion`, `comment`, `collection`, ...). The migration double-wrote through an audit log, backfilled
in about three days, verified with sampled "dark reads", and switched over in a five-minute window.
480 was chosen because it divides by many numbers; in 2023 "The great re-shard" moved the same 480
shards onto 96 machines (five each) without downtime. Secondary write-ups of Notion's data lake post
put the block count at about 20 billion at the start of 2021 and over 200 billion in 2024, doubling
every six to twelve months, and describe a Debezium, Kafka and Hudi pipeline from Postgres to S3; I
read those figures in secondary sources only. (In the internal model a database is a `collection`
and a view a `collection_view`; the public API's names differ.)

**Client.** The native apps keep a **RecordCache**, an LRU cache of records on SQLite (IndexedDB in
the browser), and a durable **TransactionQueue** of edits the server has not yet accepted. Reads try
memory, then the cache, then `loadPageChunk`, which walks down the content tree from a block and
returns the blocks and the records they depend on.

**Offline.** "How we made Notion available offline" (11 December 2025) describes turning the
best-effort cache into a guarantee. An `offline_page` table is the set of pages kept offline and an
`offline_action` table records each reason (`toggled`, `favorite`, a recent visit, inherited from a
parent); a page leaves the set only when its last reason goes. Notion would rather not open a page
than open one with blocks missing. Pages kept offline are migrated to the CRDT text model, and
clients catch up by comparing a `lastDownloadedTimestamp` with the page's `lastUpdatedTime` after a
per-page change message, rather than polling.

The source of truth is the server. The client is optimistic: it shows an edit at once and rolls back
if the transaction is rejected.

## Queries and derived views

**Queries** go to a data source:

```http
POST /v1/data_sources/{data_source_id}/query
Notion-Version: 2026-03-11
```

```json
{
  "filter": {
    "and": [
      { "property": "Status", "status": { "equals": "In progress" } },
      { "or": [
          { "property": "Hours", "number": { "greater_than": 2 } },
          { "property": "Due", "date": { "before": "2026-10-20" } }
      ] }
    ]
  },
  "sorts": [
    { "property": "Due", "direction": "ascending" },
    { "timestamp": "created_time", "direction": "descending" }
  ],
  "page_size": 50
}
```

Filters are typed by the property (`select.equals`, `number.greater_than`, `date.before`,
`relation.contains`, `checkbox.equals`, `formula.number.*`, `rollup.any/every/none` and so on).
`and` and `or` take up to 100 items and nest one level. Sorts apply in order. Results are pages, in
pages of results with `has_more` and `next_cursor`; `filter_properties` trims what each row
carries. Since April 2026 a query returns at most 10,000 results, marking itself `incomplete` past
that; the advice is to partition by `created_time`.

**Formulas** are expressions over a row's own properties (`prop("Hours") * 60`), stored by property
id so renames do not break them, and read-only. **Rollups** follow a relation and aggregate one
property of the related pages with a fixed function (`count`, `sum`, `average`, `median`, `min`,
`max`, `range`, `earliest_date`, `percent_checked`, `show_original`, ...). Both are computed by Notion,
not stored by the writer. The query reference warns that a formula over a relation with more than
25 references evaluates only 25, and that rollups and formulas through several layers of relations
"may return incorrect results"; for exact values it says to read the property item on its own.

**Views** store a filter, sorts and grouping as data, so a board and a table over the same rows are
two view records and one data source. The MCP server also offers read-only SQL over one data source
or across 2 to 100 of them (`notion-query-data-sources`, `notion-query-multiple-data-sources`),
which the REST API does not; the SQL dialect is not documented on the tools page.

## Change, history and sync

**Operations and transactions.** A UI action becomes one or more *operations*, each creating or
updating one record; related operations go together in a *transaction* that commits or fails as a
whole. Pressing Enter in a to-do list, in the post's example, makes two operations in one
transaction: create a `to_do` block with a new UUID and an empty title, and insert its id into the
parent's content array. The client applies it locally and re-renders within milliseconds, queues it
durably, and posts it to `/saveTransactions`. The server loads the affected records and their
parents ("before"), applies the operations to a copy ("after"), checks permissions and consistency
against both, commits, and schedules the follow-on work: version history snapshots, search indexing
and a notice to MessageStore.

**Real time.** Each client holds a WebSocket to **MessageStore** and subscribes to the records it
shows. After a commit, MessageStore pushes which records changed; a client whose cached version is
older calls `syncRecordValues` and re-renders. Records carry versions, and freshness is decided by
comparing them.

**Conflicts.** Until 2025 the rule was last-write-wins per block record: two people could edit
different blocks at once, but concurrent edits to the same block could lose one person's text. "How
Notion handles concurrent editing with CRDTs" (published 18 September 2026; in production since July
2025) replaces that for text with a CRDT based on RGA: every character inserted has a stable id (a
session id and a Lamport clock), inserts name their origin, deletions leave tombstones because
offline operations may still point at them, and runs from one session share an id and a length.
Because Notion text crosses blocks (Enter splits a block, Backspace merges two), each block's text is
a "text slice tree" of slices belonging to a "text instance" that survives splits and moves.
Formatting uses Peritext-style marks with start and end anchors (bold grows when typing at its edge,
a link does not). The post says it handles "millions of CRDT operations every minute" and concedes
that a CRDT never loses changes but does not always preserve intent when two people rewrite the same
sentence. It covers only text; I could not confirm whether properties and tree structure are still
last-write-wins per record, though nothing published says otherwise.

**Page history.** Notion snapshots a page every ten minutes while it is being edited and two minutes
after the last edit. Retention is 7 days on Free, 30 on Plus, 90 on Business and unlimited on
Enterprise, which also shows a full change log of who changed what. Restoring a database restores
its rows and properties but not the text inside those rows. Trash keeps pages for 30 days. Undo is
per client; history is per page; there is no per-block revision a client can name in a write.

**Permissions** ride on the same tree. Access is checked by walking parent pointers to the
workspace root; sub-pages inherit and can be given their own sharing; Notion "respects the broadest
level of access given to a user", so a restriction lower down does not beat a grant higher up. Levels
are Full access, Can edit, Can edit content (rows and values, not the schema or views), Can create
(Business and Enterprise), Can comment and Can view. Databases can add page-level rules keyed by a
person or created-by property, per data source.

## Extensibility and agent access

**The public REST API** (`api.notion.com/v1`, versioned by a `Notion-Version` header; current
versions 2025-09-03 and 2026-03-11) covers pages, blocks, databases, data sources, views (since March
2026), users, comments, search and file uploads, with Markdown read and write for page bodies since
February 2026 (`update_content` with exact-match `old_str` edits since March, an opt-in "Notion-flavored
Markdown" `markdown_version: v2` since September). Integrations authenticate as an internal token or
by OAuth and see only what a person has shared with them. What it cannot do: move an existing block
(append cannot relocate one), update a synced block's content, read or press a button, edit status
groups, or create a verification property.

**Limits.** 180 requests a minute per connection on most plans and 600 on Business and Enterprise,
with a shared per-workspace limit on top (numbers not published); 429 `rate_limited` with a retry
hint. Payloads: 1,000 blocks and 500 KB a request, any array (rich text, children) 100 items, two
levels of nesting per append, 2,000 characters per text run, 100 relations, people or multi-select
options per value.

**Webhooks.** A connection subscribes a public HTTPS endpoint, verified by a one-time token and
signed with `X-Notion-Signature` (HMAC-SHA256 of the body). Events are notices with ids and a type,
not content: `page.created`, `page.content_updated`, `page.properties_updated`, `page.moved`,
`page.deleted`, `page.locked`, `comment.created`, `data_source.content_updated`,
`data_source.schema_updated`, `database.*`, `view.*` and so on. Some, like `page.content_updated`,
are batched and can arrive a minute or two late; the handler then reads the record through the API.
(The full event list and the retry policy are on reference pages I did not read in full.)

**The Notion MCP server** is hosted by Notion (the endpoint listed by third parties is
`https://mcp.notion.com/mcp`, streamable HTTP; I did not find the URL on Notion's own pages) and
authorised with OAuth. Its tools speak Markdown, not block JSON: `notion-fetch` returns a page,
database, data source or saved view as Notion Markdown, with content it did not load marked
`<unknown>`; `notion-update-page` takes search-and-replace edits with exact `old_str` matches; it
can create and move pages, create databases, data sources and views, query rows or run SQL, read
backlinks and comments, and (since August 2026) start and steer Notion's Custom Agent sessions. It
draws on the same rate limits as the API and retries a 429 only when the wait is two seconds or less.

**Notion's own agents.** Notion 3.0 (September 2025) made the in-app AI an agent that edits pages
and databases on a person's behalf. Custom Agents (public beta 24 February 2026, Business and
Enterprise) run on schedules and triggers (a database change, a Slack message, an email), with
permissions scoped to the pages, databases and MCP connections they are given, and usage metered in
credits since May 2026. These dates come from news coverage and third-party write-ups, not from
Notion's release notes, which I did not read.

## Against fictty

fictty's UI value is a tree of typed nodes with ids, as Notion's is; its store is a flat map of
named, versioned JSON or text entries. Notion folds both into one tree: the document *is* the data,
and a database row is a page. fictty keeps them apart on purpose (the UI is the agent's and a push
replaces it; the store is shared and only writes change it), so Notion's block tree is a cousin of
each half. Read as a UI, a Notion page is a fictty `col` of typed nodes, where `type` picks the
component and `content` is `children`. Read as data, a Notion data source is a fictty store entry
holding an array of records, and a Notion view is a fictty `table` or `list` node with a `filter` over a `bind`
(fictty has no `sort` on a node yet; rows show in the order the data has them).

| | Notion | fictty |
|---|---|---|
| **Unit** | The block: id, type, properties, content (child ids), parent. Documents, rows and pages are all blocks. | Two units: a UI node (`type`, `id`, props, `children`), and a store entry (`rev`, `by`, a JSON value or a caretline text). |
| **Identity** | A random UUID on every record, stable across moves and type changes. | Node ids are names the agent picks, unique in one UI and optional; store entries are named keys (`note/{id}`, `ledger`). Rows have no ids of their own unless the data gives them one (`key`). |
| **References** | Parent pointer (permissions), content array (order), mentions in text, typed relations between data sources, synced blocks, derived backlinks. | By name: a `bind` names an entry or source and a path; `@id` reads another node's selected row; templates name entries (`{$store.key}`). No references between entries; `patch` addresses a node by id. |
| **Schema** | Per data source, typed properties with stable ids; formulas and rollups are schema. A block type fixes its payload. | No schema in the store; each component says what shape it reads, and a mismatch is a problem sentence. The component set is closed, so the UI value has a schema of sorts. |
| **Storage** | Postgres sharded by workspace (480 logical shards), record cache in SQLite on devices, offline set with reasons. | One process's memory; `--store file.json` written atomically after each change, one screen holding the file's lock. The UI is whatever was last pushed. |
| **Queries** | Typed filters and sorts per data source, saved as views; formulas, rollups; SQL through MCP; 10,000-row cap. | `filter` on a node over its bound value (no `sort`: rows keep the data's order), `when` conditions, `@` binds, table totals; computed in `view`, never stored. No query language, by rule. |
| **History** | Operations in transactions; page snapshots every ten minutes, kept 7 to unlimited days; trash for 30; per-client undo. | `rev` and `by` per entry; caretline's undo history inside text entries; `$log` of what agents did; the state is serializable, so a run replays from its messages. No snapshots of the store. |
| **Concurrency** | Optimistic client, server validates each transaction; LWW per record until 2025, CRDT (RGA, Peritext marks) for text since; versions compared on sync. | One writer at a time through `update`; `--if-rev N` refuses a stale write and returns the entry; an agent's text edit goes through its own caretline view, so the person's carets move with it. One screen, one store, no merge. |
| **Agent access** | REST API with rate limits and payload caps, webhooks that announce ids, a hosted MCP server that speaks Markdown with exact-match edits, in-app agents. Agents see what was shared with them. | A local socket with no rate limit: `push`, `patch`, `put`, `merge`, `edit`, `read`, `key`, `mouse`, and the screen read back (`screen --nodes`, `get --state`). An agent sees everything, including what the person sees. |

### What fictty could take from it

- **Stable ids on rows.** Notion's selection, mentions and relations all work because every row has
  a UUID. fictty's state-and-theme proposal already lists "collections keyed by id with an `order`"
  as the next step when an example asks; Notion is the strongest evidence that it pays. A list's
  `key` already plays this part for view state; making it a property of the data (an entry of
  records keyed by id) would let selection, folds and an agent's writes follow a row across sorts.
- **The type as a lens.** Changing a block's type keeps the properties the new type does not read.
  fictty's `patch` could keep a node's view state, and a component switch (list to table, `col`
  to `grid`) could keep props it ignores, so an agent trying a layout and switching back loses
  nothing. This is a rule about the model, not a special case.
- **Writes per record, not per entry.** Notion conflicts at the record, so two people editing
  different rows never collide. fictty's `--if-rev` is per entry, so two agents writing different
  rows of one `ledger` refuse each other. The planned JSON Pointer writes with a revision per record
  are the same idea; Notion's 2025 move away from last-write-wins on even a single block says the
  grain matters.
- **Notices that carry ids, not content.** Notion's webhooks say "this changed" and let the reader
  fetch. A fictty `watch` that streams `{entry, rev, by}` would let an agent react to the person
  without polling `get --state`, and the reply stays small.
- **Edits as exact-match replacements.** The MCP server's `update_content` (an `old_str` that must
  match once) is the agent-facing twin of fictty's `edit` ranges with `--if-rev`: it is robust to
  offsets moving and refuses when the text is not what the agent read. fictty could accept either.
- **Derived values declared, not stored.** Rollups and formulas are part of the schema and computed
  on read, which is fictty's one-source-of-truth rule; a table's column totals already do this.
  Rollups through an `@` bind (sum a field of the rows another list points at) are a natural next
  derived value, if a demo asks for one.

### What it should not take

- **Folding the UI and the data into one tree.** In Notion the page is the data, so moving a block
  moves content and changing a layout is an edit to the document. fictty's split (an agent replaces
  the UI freely; nothing but a write touches the store) is what lets an agent rewrite a screen
  without risking the person's work. Keep it.
- **Formulas.** Notion's formula language is code in the data: expressions, conditionals, functions.
  fictty's rule is "data, never code"; its templates substitute and pick, nothing more. A fixed set
  of named aggregates (as rollups are) fits; an expression language does not.
- **A CRDT.** Notion needs one because many clients edit the same text offline. A fictty screen has
  one store in one process and one `update` loop, so every write is already ordered; caretline's
  stale-revision rule plus transformed carets covers an agent and a person on one text. A CRDT
  would add ids per character and tombstones for a case fictty does not have.
- **Permissions on the tree.** Parent-walking permissions are the reason Notion has a parent
  pointer at all. A fictty screen is one person's, and agents reach it over a local socket; access
  control belongs to whoever starts the screen, not to nodes.
- **Hidden server-side state and limits.** Notion's 25-reference formula cutoff, the 10,000-row
  cap and batched webhooks mean an agent can read a value that is quietly incomplete. fictty's
  rule that every feature shows up in `screen` or `get --state`, with problems as sentences, is the
  better contract; where fictty must cap something, it should say so in `problems`.

### Trade-offs

- **One record type versus two layers.** Notion's single block makes everything composable
  (any block can hold any block, any page can become a row) at the cost of a model where structure,
  content and permissions are tangled in one tree. fictty's UI-plus-store is less uniform but each
  owner has one set of rules.
- **UUIDs versus names.** UUIDs make moves, renames and merges safe and make references unreadable;
  fictty's named ids and entries are readable by an agent and a person (`@albums`, `note/42`) and
  rely on the agent keeping names unique. A store keyed by ids should keep readable names for
  entries and use ids for the records inside them.
- **Schema versus shape checks.** Notion's typed properties let it validate writes, index and
  query; the price is a schema to migrate (the 2025 data source split broke older integrations).
  fictty's shapes-belong-to-components keeps the store schemaless and pushes the checking to read
  time, which suits agents dropping a command's JSON straight in, and loses validation on write.
- **Snapshots versus replay.** Notion keeps periodic page snapshots on a plan-dependent clock;
  fictty's serializable state and pure `update` mean a whole run can be replayed, but nothing keeps
  old store values once overwritten. A bounded history of writes per entry (`rev`, `by`, value)
  would give fictty Notion's "restore this version" without snapshots.

## Sources

- Notion engineering, "The data model behind Notion's flexibility":
  https://www.notion.com/blog/data-model-behind-notion
- Notion engineering, "Herding elephants: lessons learned from sharding Postgres at Notion":
  https://www.notion.com/blog/sharding-postgres-at-notion
- Notion engineering, "The great re-shard": https://www.notion.com/blog/the-great-re-shard
  (introduction only read; shard counts confirmed in secondary sources)
- Notion engineering, "How we made Notion available offline" (11 December 2025):
  https://www.notion.com/blog/how-we-made-notion-available-offline
- Notion engineering, "How Notion handles concurrent editing with CRDTs" (18 September 2026):
  https://www.notion.com/blog/how-notion-handles-concurrent-editing-with-crdts
- Data lake figures (secondary): https://blog.bytebytego.com/p/storing-200-billion-entities-notions
- API reference, block object: https://developers.notion.com/reference/block
- API reference, rich text and mentions: https://developers.notion.com/reference/rich-text
- API reference, data source properties: https://developers.notion.com/reference/property-object
- API reference, query a data source: https://developers.notion.com/reference/query-a-data-source
- API reference, append block children: https://developers.notion.com/reference/patch-block-children
- API reference, request limits: https://developers.notion.com/reference/request-limits
- API reference, webhooks: https://developers.notion.com/reference/webhooks
- Upgrade guide for 2025-09-03 (databases and data sources):
  https://developers.notion.com/docs/upgrade-guide-2025-09-03
- Developer changelog: https://developers.notion.com/page/changelog
- Working with views: https://developers.notion.com/guides/data-apis/working-with-views
- Notion MCP: https://developers.notion.com/docs/mcp and supported tools:
  https://developers.notion.com/docs/mcp-supported-tools
- Help, synced blocks: https://www.notion.com/help/synced-blocks
- Help, sharing and permissions: https://www.notion.com/help/sharing-and-permissions
- Help, duplicate, delete and restore content (version history, trash):
  https://www.notion.com/help/duplicate-delete-and-restore-content
- Custom Agents coverage (secondary):
  https://alternativeto.net/news/2026/2/notion-3-3-launches-custom-agents-for-autonomous-team-automation
  and https://matthiasfrank.de/en/notion-custom-agents/
- fictty: `AGENTS.md`, `docs/guide.md` ("The model", "The store"),
  `docs/design/state-and-theme.md`, `crates/fictty-core/src/store.rs`
