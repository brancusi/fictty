# Obsidian: the data model

Obsidian is a note-taking app whose data model is a folder of plain Markdown files. Everything else
it shows, from backlinks and tags to tables, graphs and canvases, is either read out of those files
into a derived index or written as another plain file beside them (a `.canvas` in JSON, a `.base`
in YAML). The app owns no database that matters: delete its cache and it rebuilds the cache from
the files; delete the app and the files still read as text. This note looks at what that choice
gives and costs, as of Obsidian 1.14.4 (5 October 2026), and sets it against fictty's UI value and
store.

- **Maker:** Obsidian (Dynalist Inc.), a small independent company; closed-source app, open file
  formats (Markdown, JSON Canvas)
- **Kind:** local-first personal knowledge base; desktop (Electron) and mobile apps, with a paid
  Sync service and an official CLI (since 1.12, February 2026)
- **Storage:** a vault is a folder on disk; notes are `.md` files, config is `.obsidian/` inside it,
  the metadata cache is in IndexedDB outside it
- **Unit of data:** the file (a note, with optional YAML frontmatter); within a note, headings and
  blocks are addressable
- **Source of truth:** the files on disk; every index is derived and rebuildable

## The unit of data

The unit is a file. A note is a `.md` file whose name (without the extension) is its title. A note
may start with a YAML block of **properties** (frontmatter), which is the only structured data
Obsidian itself understands per note:

```markdown
---
title: Ride report
date: 2026-10-12
distance: 42.5
done: true
tags:
  - cycling
  - journal
aliases:
  - Sunday ride
related: "[[Kestrel tune-up]]"
---
# Ride report

Climbed the long hill twice. ^climb

- Chain needs oil [mood:: tired]
```

Below the frontmatter is free Markdown. Obsidian parses it into sections, headings, list items,
links, embeds, tags, footnotes and block ids, but none of that is a record: it is text with
positions. Attachments (images, PDFs, audio) are files too, linked or embedded from notes. Two
further kinds of file are Obsidian's own formats, both plain text: `.canvas` (JSON Canvas, a
spatial board) and `.base` (YAML, a saved database view over notes). Neither holds note content of
its own beyond what it says on cards; a base holds none at all.

Properties are deliberately flat. The help says nested properties are not supported in the
properties editor (they survive in the YAML and show in source mode). JSON frontmatter is accepted
but read and saved back as YAML.

## Identity and references

**A note's identity is its path.** There is no hidden id: `Projects/Kestrel tune-up.md` is the
note. The developer API's `TFile` is a handle on a path, and the metadata cache is keyed by path
(`resolvedLinks: Record<string, Record<string, number>>`, source path to destination path to a
count). Two notes in different folders may share a name; a link then needs enough of the path to
be unambiguous.

**References are text in the note.** The main forms:

| Form | Syntax |
|---|---|
| Wikilink (the default) | `[[Kestrel tune-up]]`, `[[Projects/Kestrel tune-up]]` |
| Markdown link | `[Kestrel tune-up](Projects/Kestrel%20tune-up.md)` |
| Display text | `[[Kestrel tune-up\|the tune-up]]` |
| Heading | `[[Kestrel tune-up#Brakes]]`, nested `[[Note#Heading#Subheading]]`, same note `[[#Brakes]]` |
| Block | `[[Ride report#^climb]]` |
| Embed | `![[Ride report]]`, `![[Ride report#^climb]]`, `![[map.png\|300]]`, `![[manual.pdf#page=3]]` |
| In a property | `related: "[[Kestrel tune-up]]"` (quoted, so YAML doesn't read it as a list) |

A link names a target by text. Settings choose whether new links are written as the shortest path
that is unique, a path relative to the note, or the absolute path in the vault;
`MetadataCache.getFirstLinkpathDest(linkpath, sourcePath)` resolves a link to "the best match".
`aliases` in a note's properties give it other names that link suggestions offer; the link still
points at the file.

**Block ids** are the only ids inside a note. A `^id` at the end of a paragraph (or on its own line
after a list, quote, callout or table, or on a list item) marks that block; ids may contain Latin
letters, numbers and dashes, and Obsidian generates a random six-character one (`^37066d`) when you
link to a block that has none. The id is text in the file, so it moves with the block and survives
edits around it, but nothing stops two blocks having the same one or a sync merge duplicating it.
Heading links are by heading text: rename the heading and the link breaks unless it is updated.

**Backlinks are derived.** No note records what links to it. The metadata cache resolves every
note's outgoing links into `resolvedLinks` and `unresolvedLinks`; the Backlinks core plugin, the
graph and the CLI's `backlinks` invert that. "Unlinked mentions" is a text search for the note's
name. A link to a note that doesn't exist is legal and stays unresolved until someone creates the
note (clicking it creates it at the linked path).

**Renames rewrite other files.** Because identity is the path and references are path text,
renaming a note means editing every note that links to it. With *Automatically update internal
links* on (Settings, Files and links), a rename through Obsidian does that;
`FileManager.renameFile(file, newPath)` is the API ("Rename or move a file safely, and update all
links to it depending on the user's preferences"), and the CLI's `move` and `rename` go through the
running app. A rename made outside Obsidian (Finder, `git mv`, an agent's `mv`) reaches it only as
file-system events and, by common report, leaves the old links unresolved. Renaming a heading
does not update heading links.

## Structure and hierarchy

- **Folders** are the only containment. A note is in exactly one folder; there is no other parent.
  Folders carry no metadata of their own.
- **Notes** are flat text with an outline: `CachedMetadata.headings` and `sections` (root-level
  Markdown blocks) give the shape, and `listItems` carries each item's parent and task status. The
  structure is recomputed from the text on every change.
- **Properties** have types that are vault-wide by name: once `due` is a Date, every note's `due`
  is shown and edited as a date. The types are Text, List, Number, Checkbox, Date, Date & time and
  Tags (the last only for `tags`). The type is a hint to the editor and to Bases, not a constraint:
  a note can still hold `due: soon`, and Obsidian shows it as invalid rather than refusing it. (The
  type assignments are kept in the vault's config, `.obsidian/types.json`; the help page does not
  document the file.) Default properties are `tags`, `aliases` and `cssclasses`; the singular
  `tag`, `alias` and `cssclass` were deprecated in 1.4 and dropped as defaults in 1.9.
- **Tags** cut across folders: `#cycling` in the text or `tags:` in properties, nested with
  slashes (`#project/kestrel`), with no registry. A tag exists because some note uses it.
- **Bases** (core plugin; early access in 1.9.0, 21 May 2025, public in 1.9.10, 18 August 2025) are
  saved views: the data stays in the notes' properties, and a `.base` file (or a ```` ```base ````
  code block in a note) says which notes to take and how to show them. Editing a cell in a base's
  table writes the property into that note's frontmatter. Layouts are table, cards, list, map and,
  since 1.14.0 (September 2026), kanban; plugins can add more.

  ```yaml
  filters:
    and:
      - file.hasTag("book")
  formulas:
    ppu: "(price / age).toFixed(2)"
  properties:
    formula.ppu:
      displayName: "Price per use"
  views:
    - type: table
      name: "Reading"
      filters:
        and:
          - 'status != "done"'
      order: [file.name, status, formula.ppu]
      groupBy: {property: status, direction: ASC}
  ```

  Filters and formulas are an expression language (`file.hasTag(...)`, comparisons, method calls,
  `now()`, `this` for the note embedding the base), so a base is data that holds small programs.
- **Canvas** (core plugin) is a spatial board stored as JSON Canvas 1.0, an open spec at
  jsoncanvas.org (March 2024). A canvas is two arrays: `nodes` (each with `id`, `type`, `x`, `y`,
  `width`, `height`, optional `color`; types `text`, `file`, `link`, `group`) in ascending
  z-order, and `edges` (`id`, `fromNode`, `toNode`, optional sides, ends, `color` and `label`).

  ```json
  {
    "nodes": [
      {"id": "a1", "type": "text", "text": "# Tune-up\nBrakes, chain", "x": 0, "y": 0, "width": 260, "height": 120},
      {"id": "b2", "type": "file", "file": "Projects/Kestrel tune-up.md", "subpath": "#Brakes",
       "x": 340, "y": 0, "width": 400, "height": 300, "color": "4"}
    ],
    "edges": [
      {"id": "e1", "fromNode": "a1", "fromSide": "right", "toNode": "b2", "toSide": "left", "label": "see"}
    ]
  }
  ```

  Canvas nodes and edges do have ids, unlike notes, and a `file` node refers to a note by path.
- **Inline fields** (`rating:: 9`, `[mood:: tired]`) are a Dataview plugin convention, not core
  Obsidian: core sees them as text.

## Storage and the source of truth

- **The vault is a folder.** Obsidian "automatically refreshes your vault to keep up with any
  external changes", so any program that writes a file in it is a first-class writer.
- **`.obsidian/`** inside the vault holds that vault's config as JSON: settings, hotkeys, enabled
  plugins and their `data.json`, themes and snippets, property types, and the workspace layout
  (`workspace.json`). It is per vault and syncs with it (if the sync service includes it). Global
  settings, the vault list and File recovery's snapshots live in the app's own folder
  (`~/Library/Application Support/obsidian` on macOS).
- **The metadata cache** is the derived index: per file, its `frontmatter` (as an object),
  `frontmatterPosition`, `frontmatterLinks`, `headings`, `sections`, `links`, `embeds`, `tags`,
  `listItems`, `blocks`, `footnotes`, `footnoteRefs` and `referenceLinks`, each with positions in
  the text; and across files, `resolvedLinks` and `unresolvedLinks`. It is kept in IndexedDB so it
  survives restarts, it can fall out of step with the files, and *Rebuild vault cache* (Files and
  links settings) throws it away and reads every file again. Plugins get it with
  `metadataCache.getFileCache(file)` and the `changed`, `resolve` and `resolved` events (`changed`
  doesn't fire on rename; listen on the vault for that).
- **Reads come in two speeds.** `vault.cachedRead()` may return Obsidian's copy of a file that was
  changed on disk a moment ago; `vault.read()` goes to disk. Hidden folders (including `.obsidian`)
  are reachable only through the lower-level `vault.adapter`.

## Queries and derived views

All of these read the files or the cache and write nothing unless the person edits through them.

- **Search** (core): text with operators: `path:`, `file:`, `tag:`, `line:`, `block:`,
  `section:`, `task:`, `[status:done]` for a property, and `/regex/`. A ```` ```query ```` code
  block embeds a search's results in a note.
- **Bases** (core, above): a declarative view with filters, formulas, grouping, sorting and
  summaries over notes and their properties, including file fields (`file.name`, `file.mtime`,
  `file.tags`, `file.backlinks`, the last noted as slow and not refreshed live). The CLI's
  `base:query` returns a base's rows as JSON, CSV, TSV, Markdown or paths.
- **Dataview** (community plugin, by Michael Brenan): the older and still widely used query layer.
  It indexes frontmatter, inline fields and implicit fields (`file.cday`, `file.outlinks`,
  `file.tasks`) in memory and answers a query language:

  ````markdown
  ```dataview
  TABLE rating, file.mtime AS "edited"
  FROM #book AND "Reading"
  WHERE rating > 7
  SORT rating DESC
  ```
  ````

  Results render in place and are not stored. DataviewJS blocks run arbitrary JavaScript against
  the index. Queries are read-only by design; editing values means editing the notes (Bases, by
  contrast, writes back).
- **Graph view** (core): notes as nodes and resolved links as edges, from the cache, filtered by
  search terms, tags and folders. The local graph is the same around one note.
- **Outline, Tags, Backlinks, Properties** panes: each is a view of the cache.

## Change, history and sync

- **Writes are whole files.** The editor holds a note's text in memory and saves it to disk shortly
  after typing stops. `vault.modify()` replaces a file's contents; `vault.process(file, fn)` is an
  atomic read-modify-write ("Always prefer `Vault.process()` over `Vault.read()`/`Vault.modify()`
  to avoid unintentional loss of data"), synchronous only; `fileManager.processFrontMatter(file,
  fn)` does the same for the frontmatter as an object. There is no revision number on a file and
  no record of who wrote it; the file's modification time is the only stamp.
- **External edits** are picked up by the file watcher and reloaded into open editors. The
  editor's undo history is in memory and per editor; it is not saved with the file.
- **File recovery** (core plugin) snapshots `.md` and `.canvas` files at most every 5 minutes and
  keeps them 7 days by default (both configurable). Snapshots are kept outside the vault, are
  per device and never sync. It is "not a complete backup solution".
- **Sync** (paid service, optionally end-to-end encrypted) moves files between devices. It is file
  sync, not a shared document:
  - Markdown files that changed on two devices are merged with Google's diff-match-patch, which
    can leave duplicated text for the person to clean up.
  - Every other file, canvases included, is last-modified-wins.
  - Settings JSON is merged by key, the local keys over the remote ones.
  - Since 1.9.7 a per-device *Conflict resolution* setting can create a conflict file instead
    (`Note (Conflicted copy device-name 202610121530).md`), keeping the remote version in place.
  - **Version history** keeps notes for 1 month on the Standard plan and 12 months on Plus, and
    attachments' old versions for two weeks; settings and deleted files have their own history
    and restore.
  - **Shared vaults** allow up to 20 collaborators, all with the owner's permissions bar
    inviting. There is no real-time co-editing: "You will not see the other user's cursor, and
    their edits will only appear once the changes are synced." Live cursors exist only through
    community plugins (Live Share, Peerdraft, screen.garden and others).
  - **Obsidian Headless** (open beta, Node.js; `npm install -g obsidian-headless`) runs Sync from
    the command line without the app, with `ob sync --continuous` to keep a folder in step; the
    help page names CI pipelines and agents as its uses and warns against running it beside the
    desktop app's Sync on the same device.
- **Other sync** (iCloud, Dropbox, Syncthing, git through the community Obsidian Git plugin) works
  because the vault is a folder; each brings its own conflict behaviour, usually conflict copies.

## Extensibility and agent access

- **Plugin API** (TypeScript, in-process, full access): `app.vault` for files, `app.metadataCache`
  for the index, `app.fileManager` for link-aware renames and frontmatter, `app.workspace` for
  panes and views, plus commands, settings tabs, editor extensions (CodeMirror 6), Markdown
  post-processors and, for Bases, custom view types. Community plugins run unsandboxed in the
  app.
- **URI scheme**: `obsidian://open?vault=…&file=…`, `obsidian://new?…&content=…&append`,
  `obsidian://search?query=…`, `daily`, `unique`, `choose-vault` and `hook-get-address`, with
  `x-success`/`x-error` callbacks on some. The approval dialog for URI actions was removed in
  1.13.6 (August 2026).
- **The official CLI** (since 1.12, February 2026; the help asks for installer 1.12.7 or later) is
  enabled in Settings, General and talks to the running app (starting it if needed). It covers
  files (`read`, `create`, `append`, `move`, `rename`), properties (`property:set name=status
  value=done file=Recipe`), search, bases (`base:query … format=json`), tasks, links and backlinks,
  history and sync (`diff`, `sync:history`, `sync:restore`), plugins, and `eval code="…"` to run
  JavaScript in the app. Because it goes through the app, its renames update links and its
  property writes use the frontmatter API.
- **Local REST API** (community plugin, by Adam Coddington): an HTTPS server on
  `127.0.0.1:27124` with a bearer key; `/vault/{path}` to read, write and delete, `/active/` for
  the open note, search (simple and JsonLogic), commands, and `PATCH` that targets a heading, a
  block id or a frontmatter key (`{"targetType": "heading", "target": ["My Section"], "operation":
  "append", "content": "…"}`). It supports `ETag`/`If-Match` for conditional writes and now carries
  its own MCP endpoint at `/mcp/` (tools such as `vault_read`, `vault_patch`, `search_query`).
- **MCP servers**: many third-party ones, mostly wrappers over the Local REST API (for example
  `mcp-obsidian`) or readers of the vault folder directly. None is first-party.
- **Agents editing files directly**: the commonest route, since a vault is a folder of Markdown.
  An agent with file access can read and write notes with no API at all, and Obsidian picks the
  changes up. What it gives up: link updates on rename, the frontmatter API's safety, and any
  conflict check. Steph Ango (Obsidian's CEO) publishes `kepano/obsidian-skills`, Agent Skills that
  teach an agent Obsidian Flavored Markdown, Bases, JSON Canvas and the CLI: the official stance is
  that agents edit the open formats.

## Against fictty

fictty's model, for comparison: a **UI value** an agent pushes and patches (nodes with ids, a
closed set of components), a **store** of named entries (each a JSON value or a caretline text
with its undo history, each with a `rev` and a `by`), **view state** per node id (selection,
folds, carets, scroll, owned by the person), and **sources** (commands, streams, host data) that
are refetched and disposable. Agents write over a socket (`push`, `patch`, `put`, `merge`, `edit`,
`--if-rev`) and read the screen back (`screen --nodes`, `get --state`).

| | Obsidian | fictty |
|---|---|---|
| **Unit** | a file: a Markdown note (frontmatter plus text), a `.canvas`, a `.base`, an attachment | a store entry: one JSON value or one text document; plus the UI value as a separate whole |
| **Identity** | the vault path; block ids (`^id`) inside notes; ids on canvas nodes | the entry's key (a string, which may hold `/` and `.`); node ids for view state |
| **References** | wikilinks and Markdown links by path text, to notes, headings and blocks; backlinks derived from the cache; renames rewrite the linking files | binds by key and path (`books.running`), `@id` for another list's selection, `doc` templates naming an entry from a row (`note/{id}`); no back-references, no rename |
| **Schema** | none enforced; property types are vault-wide hints by name; bases and Dataview read whatever is there | none; each component says what shape it reads at its bind, and a mismatch is a `problems` sentence |
| **Storage** | many plain files in a folder; config as JSON in `.obsidian/`; a derived cache in IndexedDB | in memory; with `--store file.json` one JSON file, written atomically after each change, locked to one screen |
| **Queries** | search operators, Bases (declarative, with an expression language), Dataview (a query language, and JavaScript), the graph | none as such: binds, a list's `filter` on a store entry, templates, `when` conditions, and sources (`cmd`, `stream`) that compute outside |
| **History** | File recovery snapshots (local, 7 days); Sync version history (1 or 12 months); per-editor undo in memory only | a `rev` and a `by` on every entry; a text's undo history kept in the entry; `$log` of agents' socket writes; no older versions kept |
| **Concurrency** | file-level; Sync merges Markdown with diff-match-patch, others last-write-wins, optionally a conflict file; `vault.process` atomic within the app; no real-time co-editing | one process owns the store; `--if-rev N` refuses a stale write and returns the current entry; an agent's text edit is an edit, so the person's carets move with it |
| **Agent access** | edit files directly; the CLI through the running app; the Local REST API and MCP servers (plugins); `obsidian://` URIs | the socket: write the store, push and patch the UI, press keys, read back the screen node by node and the state as JSON |

### What fictty could take from it

- **A derived index beside the source of truth.** Obsidian's cache is rebuildable and never
  authoritative. If fictty needs back-references between entries ("what binds or names this
  entry?", "which rows point at `note/3`?"), it should derive them from the store and the UI on
  demand, not store them, which is the one-source-of-truth rule already.
- **Types by name, as hints.** Vault-wide property types tell editors how to show and edit a field
  without refusing data. A theme-like map from entry key patterns or field names to a type (date,
  number, tags) could let fictty pick input behaviour and report a `problems` sentence, with no
  schema that rejects a write.
- **Stable anchors inside text.** Block ids give a link target that survives edits around it. A
  caretline text entry could carry anchors (marks that move with the text) so an agent can say
  "append under this block" without counting characters, as the Local REST API's heading and block
  `PATCH` does.
- **A merge rule per kind of content.** Obsidian merges prose with diff-match-patch, JSON settings
  by key, and everything else by last write. fictty already has the pieces (text edits as ranges,
  RFC 7396 merge for values); if two stores ever need reconciling, say so per body kind, and let a
  conflict copy be an option rather than a silent merge.
- **Views as files of data.** A `.base` separates the view from the records it shows, and a
  `.canvas` is a small open format for positioned cards. Both are close to fictty's UI value as
  data over a store; JSON Canvas in particular maps onto a `stack` of `float` nodes and could be
  read as a source.
- **Snapshots outside the store.** File recovery keeps history apart from the files and per
  device. fictty could keep old revisions of entries outside the store file, so the store stays
  small and history stays the screen's.

### What it should not take

- **The path as identity.** A rename that has to rewrite every referring file, and that breaks
  links when another program does it, is the cost of identity living in a human name. fictty's
  keys are names too; if renaming entries ever matters, give entries a stable id and treat the key
  as a label, rather than rewriting binds.
- **Expressions in views.** Bases' formulas and filters, and DataviewJS above all, put logic in the
  data. fictty's rule is substitution only, with computation in sources and the host.
- **Silent merges of prose.** diff-match-patch can produce duplicated text that nobody asked for;
  fictty's refusal on a stale `rev` is safer for agents, who can read and retry.
- **Writes without attribution.** Obsidian cannot say who changed a note, only when. fictty's `by`
  per write, and `$log`, are what let a person see what an agent did; keep them.
- **Two read paths that can disagree.** `cachedRead` versus `read` is a stale-read hazard pushed
  onto every plugin author; fictty's single in-memory state avoids it.

### Trade-offs

Obsidian's choice buys longevity and openness: the data outlives the app, any tool can read it,
and an agent with a shell needs no API at all. It pays with weak write semantics: no revisions,
no attribution, no transactions across files, renames that are many writes, and conflicts that
are merged after the fact. fictty's store buys the opposite: every write is counted, attributed
and can be made conditional, a text edit is a real edit with undo, and the screen can be read
back exactly. It pays in reach: the store is one JSON file held by one running screen, readable
by other tools only as a whole, and an agent must go through the socket to change it safely.
A middle path for fictty, if Daybook or the demos ask for it, is Obsidian's: keep the store
authoritative, but export (or mirror) text entries as plain Markdown files that other tools can
read, treating edits that come back from disk as writes by `file` with a revision like any other.

## Sources

- Obsidian help, storage: https://obsidian.md/help/data-storage
- Internal links: https://obsidian.md/help/links
- Embeds: https://obsidian.md/help/embeds
- Properties: https://obsidian.md/help/properties
- Bases: https://obsidian.md/help/bases and syntax https://obsidian.md/help/bases/syntax
- Bases release: https://obsidian.md/changelog/2025-05-21-desktop-v1.9.0 and
  https://obsidian.md/changelog/2025-08-18-desktop-v1.9.10
- Changelog (1.13 and 1.14, August to October 2026): https://obsidian.md/changelog/
- JSON Canvas 1.0: https://jsoncanvas.org/spec/1.0/
- File recovery: https://obsidian.md/help/plugins/file-recovery
- Sync conflicts: https://obsidian.md/help/sync/troubleshoot
- Sync version history: https://obsidian.md/help/sync/version-history
- Shared vaults: https://obsidian.md/help/sync/collaborate
- Obsidian Headless: https://obsidian.md/help/sync/headless
- CLI: https://obsidian.md/help/cli
- URI scheme: https://obsidian.md/help/uri
- Developer docs, Vault: https://docs.obsidian.md/Plugins/Vault
- MetadataCache: https://docs.obsidian.md/Reference/TypeScript+API/MetadataCache
- CachedMetadata: https://docs.obsidian.md/Reference/TypeScript+API/CachedMetadata
- FileManager.processFrontMatter:
  https://docs.obsidian.md/Reference/TypeScript+API/FileManager/processFrontMatter
- FileManager.renameFile: https://docs.obsidian.md/Reference/TypeScript+API/FileManager/renameFile
- Dataview, metadata: https://blacksmithgu.github.io/obsidian-dataview/annotation/add-metadata/
- Local REST API: https://github.com/coddingtonbear/obsidian-local-rest-api
- Obsidian skills for agents: https://github.com/kepano/obsidian-skills
