// What each system actually keeps, for the same bike job: the bytes on disk or the rows in a
// table, the smallest piece of each, and what that makes fast, slow or impossible. The /data-model/
// page draws its "What's actually stored", "The atom" and "Speed and queries" sections from this.
// The sources are the pages in docs/research/data-model/; ids, keys and names here are made up.

export type Block =
  | { kind: 'code'; label: string; note?: string; lang: string; body: string }
  | { kind: 'table'; label: string; note?: string; cols: string[]; rows: string[][] };

export interface Stored {
  key: string; name: string; tint: string;
  focus: string;      // what the model is built for
  atom: string;       // the smallest piece of content
  blocks: Block[];    // what is stored, literally
  write: string;      // what one change rewrites
  ask: { code: string; lang: string; how: string }; // "which jobs are due by Friday?"
}

export const STORED: Stored[] = [
  {
    key: 'fictty', name: 'fictty', tint: 'fictty',
    focus: 'A live screen that an agent and a person share. The data is whatever JSON that screen needs, kept beside it.',
    atom: 'The store entry: a key holding one JSON value (or one text), with a revision and an author. Rows inside it are just JSON; they have ids only if the data gives them one.',
    blocks: [
      { kind: 'code', label: 'kestrel.json · the store file (fictty run --store)', lang: 'json', body:
`{
  "jobs_booked": {
    "rev": 9, "by": "agent",
    "value": [
      { "id": "j1", "bike": "Gravel · Priya N.", "work": "Brake pads, rotor clean",
        "due": "2026-10-15", "who": "Sam" },
      { "id": "j3", "bike": "Cargo · Lena M.", "work": "Rear hub", "due": "2026-10-16", "who": "Ana" }
    ]
  },
  "note/j1": {
    "rev": 3, "by": "person",
    "doc": { "text": "Squeals after rain.", "history": { "…": "the undo tree" } }
  }
}` },
      { kind: 'code', label: 'the UI value · pushed by the agent, never saved with the data', lang: 'json', note: 'The list names the entry it shows; it never holds the rows.', body:
`{ "type": "list", "id": "jobs", "bind": "jobs_booked", "row": { "…": "a card template" } }` },
    ],
    write: 'In memory, one entry changes and its rev goes up by one. On disk, the whole store file is written again (to a .tmp beside it, then renamed over it).',
    ask: { lang: 'sh', code:
`fictty read jobs_booked | jq '.value[]
  | select(.who == "Sam")
  | select(.due <= "2026-10-16")'`,
      how: 'There is no query language, on purpose. An agent (or a cmd source) reads the entry and filters it itself; on screen, a list\'s filter only matches typed words against its rows. Fast because it is all in memory and small, but nothing is indexed.' },
  },
  {
    key: 'obsidian', name: 'Obsidian', tint: 'obsidian',
    focus: 'Your notes, as plain files in a folder you own, forever. The app is a fast reader and editor over them.',
    atom: 'The note: one Markdown file, named by its path. Inside it, structure is text: YAML at the top for properties, ^ids for blocks you want to link to.',
    blocks: [
      { kind: 'code', label: 'vault/Jobs/Gravel brakes.md', lang: 'markdown', body:
`---
customer: "[[Priya Natarajan]]"
bike: Gravel
due: 2026-10-15
mechanic: Sam
tags: [job]
---
# Gravel brakes

Brake pads, rotor clean.
Squeals after rain. ^b7f2` },
      { kind: 'code', label: 'the metadata cache · IndexedDB, derived, rebuilt from the files', lang: 'json', note: 'Positions in the text, never the truth. "Rebuild vault cache" throws it away.', body:
`"Jobs/Gravel brakes.md": {
  "frontmatter": { "customer": "[[Priya Natarajan]]", "due": "2026-10-15", "tags": ["job"] },
  "links":  [{ "link": "Priya Natarajan", "position": { "start": { "line": 1 } } }],
  "blocks": { "b7f2": { "position": { "start": { "line": 10 } } } }
}` },
    ],
    write: 'The whole file is written again. Any other program can write it too; Obsidian sees the change and re-indexes that file.',
    ask: { lang: 'dataview', code:
`TABLE customer, due
FROM #job
WHERE due <= date(2026-10-16) AND mechanic = "Sam"`,
      how: 'Bases (built in) or Dataview (a plugin) answer this from an in-memory index of every note\'s frontmatter. Fast once indexed; values are only as typed as the YAML.' },
  },
  {
    key: 'roam', name: 'Roam Research', tint: 'roam',
    focus: 'Networked thought: every block can be linked, embedded and queried, so ideas connect below the level of a page.',
    atom: 'The block: an entity with a uid and a string of text. Underneath, the real atom is the datom, one fact of entity, attribute and value.',
    blocks: [
      { kind: 'table', label: 'the graph · DataScript datoms, in the browser (hosted on Roam\'s servers)', note: 'Each row also carries the transaction that wrote it, and blocks have :edit/time and :edit/user datoms too.', cols: ['e', 'attribute', 'value'], rows: [
        ['410', ':node/title', '"Gravel brakes"'],
        ['410', ':block/children', '413'],
        ['410', ':block/children', '414'],
        ['410', ':block/children', '416'],
        ['413', ':block/uid', '"x8Kd2fQ1a"'],
        ['413', ':block/string', '"Customer:: [[Priya Natarajan]]"'],
        ['413', ':block/order', '0'],
        ['413', ':block/refs', '412  (the page "Priya Natarajan")'],
        ['413', ':block/refs', '415  (the page "Customer")'],
        ['414', ':block/string', '"Due:: [[October 15th, 2026]]"'],
        ['414', ':block/refs', '418  (that day\'s daily note)'],
        ['416', ':block/string', '"Squeals after rain"'],
      ] },
    ],
    write: 'A transaction of a few datoms: change a block\'s text and its :block/string and :block/refs are replaced. Applied in your browser at once, then sent to the graph.',
    ask: { lang: 'roam', code:
`{{query: {and: [[job]] [[Sam]]
           {between: [[October 12th, 2026]]
                     [[October 16th, 2026]]}}}}`,
      how: 'Roam\'s query blocks (or Datalog with data.q) run against the whole graph, which is in your browser\'s memory. Joins and backlinks are cheap; but "due" and "Sam" are words in block strings, not typed fields.' },
  },
  {
    key: 'logseq-og', name: 'Logseq OG', tint: 'logseq',
    focus: 'Roam\'s outliner, kept as plain Markdown files on your own disk, with a database parsed from them for speed.',
    atom: 'The block: one bullet in a Markdown file. Its properties and its id are lines of text written under it.',
    blocks: [
      { kind: 'code', label: 'graph/pages/Gravel brakes.md', lang: 'markdown', note: 'id:: is written into the file only once something refers to the block.', body:
`- Brake job
  id:: 6710a2c4-3b7b-4cb2-8c3e-6a904c809e1f
  customer:: [[Priya Natarajan]]
  due:: [[Oct 15th, 2026]]
  mechanic:: Sam
	- Brake pads, rotor clean
	- Squeals after rain` },
      { kind: 'code', label: 'DataScript, parsed from the files · derived', lang: 'clojure', body:
`{:block/uuid       #uuid "6710a2c4-…-9e1f"
 :block/content    "Brake job\\nid:: 6710…\\ncustomer:: [[Priya Natarajan]]\\n…"
 :block/properties {:customer #{"priya natarajan"} :due #{"oct 15th, 2026"} :mechanic "Sam"}
 :block/page       {:db/id 310}
 :block/refs       #{{:db/id 412} {:db/id 418}}}` },
    ],
    write: 'The page\'s file is written again. A file changed outside the app is re-parsed; if the two drift apart, "Re-index" starts over from the files.',
    ask: { lang: 'logseq', code:
`{{query (property mechanic "Sam")}}`,
      how: 'Simple queries and Datalog run against DataScript in memory. Property values are text (a date is a page name, a number a string of digits), so "due by Friday" needs an advanced Datalog query and some parsing.' },
  },
  {
    key: 'logseq', name: 'Logseq 2.0', tint: 'logseq',
    focus: 'The same outliner rebuilt on a database: one truth in SQLite, typed properties, tags as classes, real-time sync.',
    atom: 'The node: a block or a page, an entity with a uuid from birth. A property is a node too, with a type, and a tag is a class that brings its properties with it.',
    blocks: [
      { kind: 'table', label: 'graphs/kestrel/db.sqlite · one table, kvs', note: 'Illustrative: each row is a node of DataScript\'s sorted index, encoded. Nothing here is meant to be read by hand.', cols: ['addr', 'content', 'addresses'], rows: [
        ['0', '(the root of the index)', '[17, 18, 19, …]'],
        ['17', '[4211 :block/title "Brake job"] [4211 :block/uuid #uuid "6710…"] [4211 :block/tags 905] …', 'null'],
        ['18', '[4211 :user.property/customer-Mx81 4380] [4211 :user.property/due-Zp0a 3120] …', 'null'],
      ] },
      { kind: 'code', label: 'the same node, read back (logseq debug pull)', lang: 'clojure', body:
`{:block/uuid  #uuid "6710a2c4-…-9e1f"
 :block/title "Brake job"
 :block/tags  #{{:db/id 905}}                   ; the class #Job
 :user.property/customer-Mx81 {:db/id 4380}    ; a node: Priya Natarajan
 :user.property/due-Zp0a      {:db/id 3120}    ; a date: the journal Oct 15th, 2026
 :user.property/mechanic-Q7cd {:db/id 4390}    ; a text value, itself a block
 :block/created-at 1792224000000}` },
    ],
    write: 'A transaction of datoms; a few index nodes are rewritten in SQLite. With sync, the write names the log position it was based on, and a stale one is replayed.',
    ask: { lang: 'clojure', code:
`{:query [:find (pull ?b [*])
         :where
         [?b :block/tags ?t]
         [?t :block/title "Job"]
         [?b :user.property/mechanic-Q7cd ?m]
         [?m :block/title "Sam"]]}`,
      how: 'Datalog over a typed graph: the class tells you which nodes are jobs, and properties have types. The most expressive query of the six, and the hardest to type by hand.' },
  },
  {
    key: 'notion', name: 'Notion', tint: 'notion',
    focus: 'A shared team workspace where a document and a database are the same tree, and any page can be a row.',
    atom: 'The block: a record with a UUID, a type, a map of properties, an ordered list of children and a parent. A database row is a page, and a page is a block.',
    blocks: [
      { kind: 'table', label: 'Postgres · the block table (simplified)', note: 'Every row also has space_id, the workspace, which picks one of 480 shards. Notion\'s internal tables aren\'t public: this follows its engineering posts; keys and ids are made up.', cols: ['id', 'type', 'parent_id', 'properties', 'content', 'version'], rows: [
        ['a91f…', 'page', '4f2a… (Jobs)', '{"title":[["Gravel brakes"]], "Wq>~":[["‣",[["p","5c0e…"]]]], "dU3;":[["‣",[["d",{"start_date":"2026-10-15"}]]]], "mE#c":[["Sam"]]}', '["c02f…","d13a…"]', '42'],
        ['c02f…', 'text', 'a91f…', '{"title":[["Brake pads, rotor clean"]]}', '[]', '3'],
        ['d13a…', 'text', 'a91f…', '{"title":[["Squeals after rain"]]}', '[]', '5'],
      ] },
      { kind: 'table', label: 'Postgres · the collection table · the database\'s schema', cols: ['id', 'name', 'schema'], rows: [
        ['4f2a…', 'Jobs', '{"title":{"name":"Job","type":"title"}, "Wq>~":{"name":"Customer","type":"relation"}, "dU3;":{"name":"Due","type":"date"}, "mE#c":{"name":"Mechanic","type":"text"}}'],
      ] },
    ],
    write: 'A few operations ("set Due on record a91f…") in one transaction. Shown at once in your app, checked and written on the server, then pushed to everyone with the page open.',
    ask: { lang: 'http', code:
`POST /v1/data_sources/4f2a…/query
{ "filter": { "and": [
  { "property": "Due",
    "date": { "on_or_before": "2026-10-16" } },
  { "property": "Mechanic",
    "rich_text": { "equals": "Sam" } } ] } }`,
      how: 'The server runs it over typed properties and returns rows (pages), sorted, a page of results at a time. Typed and quick; capped at 10,000 results, and only what the integration was shared.' },
  },
];

// the atom: six levels from the whole collection down to one character, and where each system's
// writes, names, queries and conflicts sit on them
export const LEVELS = ['the whole store or vault', 'a file or entry', 'a page', 'a block or row', 'a field', 'a character'];
export const MARKS = [
  { key: 'w', name: 'one save rewrites', short: 'save' },
  { key: 'n', name: 'smallest thing with an id', short: 'id' },
  { key: 'q', name: 'a query returns', short: 'query' },
  { key: 'c', name: 'a conflict covers', short: 'clash' },
] as const;
type Mark = typeof MARKS[number]['key'];
// at: the level; also: a second level that holds sometimes (dashed)
export const LADDER: Record<string, Record<Mark, { at: number; also?: number; note: string }>> = {
  fictty: {
    w: { at: 0, note: 'the whole store file is written again' },
    n: { at: 1, also: 3, note: 'the entry key; a row only if the data has an id field' },
    q: { at: 3, note: 'a list\'s filter shows rows of one entry' },
    c: { at: 1, note: 'the revision is per entry' },
  },
  obsidian: {
    w: { at: 1, note: 'the note\'s file' },
    n: { at: 1, also: 3, note: 'the path; a block once you give it a ^id' },
    q: { at: 1, also: 3, note: 'notes; Dataview can also list tasks and list items' },
    c: { at: 1, note: 'the file; Sync merges the text after the fact' },
  },
  roam: {
    w: { at: 4, note: 'a few datoms' },
    n: { at: 3, note: 'every block has a uid' },
    q: { at: 3, note: 'blocks (Datalog can return any value)' },
    c: { at: 3, note: 'one block; the merge isn\'t documented' },
  },
  'logseq-og': {
    w: { at: 1, note: 'the page\'s file' },
    n: { at: 2, also: 3, note: 'the page by name; a block once it is referenced' },
    q: { at: 3, note: 'blocks' },
    c: { at: 1, note: 'the file, when synced as files' },
  },
  logseq: {
    w: { at: 4, note: 'datoms, in a few index nodes' },
    n: { at: 3, note: 'every node has a uuid' },
    q: { at: 3, note: 'nodes' },
    c: { at: 4, note: 'checked against the log, replayed by intent' },
  },
  notion: {
    w: { at: 3, note: 'the record\'s row' },
    n: { at: 3, note: 'every block has a UUID' },
    q: { at: 3, note: 'rows of one data source' },
    c: { at: 4, also: 5, note: 'a property; text merges by character' },
  },
};

// speed and queries: what each model makes cheap, costly or impossible
export type Grade = 'good' | 'ok' | 'poor' | 'none';
export interface Op { name: string; question: string; cells: Record<string, [Grade, string, string]> }
export const OPS: Op[] = [
  { name: 'Open it', question: 'From cold, until you can read the job', cells: {
    fictty: ['ok', 'reads it all', 'Loads the whole store file into memory. Quick at a screen\'s size.'],
    obsidian: ['ok', 'indexes once', 'The index is kept between runs; the first open (or a rebuild) parses every file.'],
    roam: ['poor', 'loads it all', 'The whole graph is downloaded into the browser before anything shows.'],
    'logseq-og': ['poor', 'parses it all', 'Every file is parsed into the database (cached); a re-index starts over.'],
    logseq: ['ok', 'opens a file', 'Opens the SQLite file: nothing to parse.'],
    notion: ['good', 'one page', 'Fetches only the page you open, from the local cache first.'],
  } },
  { name: 'Save a change', question: 'Fix the due date', cells: {
    fictty: ['ok', 'whole file', 'Instant in memory, but the save rewrites the whole store, so it grows with it.'],
    obsidian: ['good', 'one file', 'Rewrites that one note.'],
    roam: ['good', 'a few facts', 'A transaction of a few datoms.'],
    'logseq-og': ['good', 'one file', 'Rewrites the page\'s file; the watcher re-parses it.'],
    logseq: ['good', 'a few facts', 'A transaction; a few index nodes rewritten.'],
    notion: ['good', 'one field', 'One operation, shown at once, confirmed by the server.'],
  } },
  { name: 'Find by id', question: 'Get job j1 exactly', cells: {
    fictty: ['ok', 'key, then scan', 'An entry by its key at once; a row means scanning that entry\'s array for its id.'],
    obsidian: ['good', 'by path', 'A note by its path; a block by its ^id, through the index.'],
    roam: ['good', 'indexed uid', 'uid is a unique, indexed attribute.'],
    'logseq-og': ['ok', 'if it has one', 'By uuid once one has been written; pages by name.'],
    logseq: ['good', 'indexed uuid', 'Every node has an indexed uuid.'],
    notion: ['good', 'by UUID', 'From the cache or the server.'],
  } },
  { name: 'Search the text', question: 'Which notes mention "squeal"?', cells: {
    fictty: ['none', 'no search', 'A list\'s filter matches typed words in its own rows, nothing wider.'],
    obsidian: ['good', 'built in', 'Searches every file, with operators and regular expressions.'],
    roam: ['ok', 'in memory', 'Searches every block string in the browser.'],
    'logseq-og': ['ok', 'an index', 'A search index beside the graph.'],
    logseq: ['ok', 'an index', 'A separate full-text index (search-db.sqlite), rebuilt if lost.'],
    notion: ['good', 'server side', 'A workspace-wide search index.'],
  } },
  { name: 'Ask a question', question: 'Sam\'s jobs due by Friday', cells: {
    fictty: ['none', 'by rule', 'No query language: an agent or a source computes it and writes the answer.'],
    obsidian: ['ok', 'Bases, Dataview', 'Over frontmatter; values are as typed as the YAML.'],
    roam: ['ok', 'Datalog', 'Any shape of question, but "due" and "Sam" are words in block text.'],
    'logseq-og': ['ok', 'Datalog', 'Same, over properties that are text.'],
    logseq: ['good', 'typed Datalog', 'Classes and typed properties; the most expressive of the six.'],
    notion: ['good', 'typed filters', 'Typed filters and sorts, saved as views; at most 10,000 results.'],
  } },
  { name: 'Follow links back', question: 'Every job for Priya', cells: {
    fictty: ['none', 'no links', 'An id in one entry is just a string to another.'],
    obsidian: ['good', 'in the index', 'Backlinks are kept in the index as files change.'],
    roam: ['good', 'reverse refs', 'A reverse lookup on :block/refs, inherited down the outline.'],
    'logseq-og': ['good', 'reverse refs', 'refs and path-refs in the database.'],
    logseq: ['good', 'all refs', 'References, tags and node properties are all edges.'],
    notion: ['ok', 'relations', 'Relations are two-way; rollups follow them, 25 at a time in a formula.'],
  } },
  { name: 'Grow large', question: 'Ten years of jobs', cells: {
    fictty: ['poor', 'in memory', 'All in memory, saved whole: sized for a screen\'s data, not an archive.'],
    obsidian: ['ok', 'thousands fine', 'Thousands of notes are fine; the index lives in memory.'],
    roam: ['poor', 'in the browser', 'The whole graph lives in the tab; reactive queries were turned off by default because they slowed large graphs.'],
    'logseq-og': ['poor', 'slow re-index', 'Big graphs are slow to parse and re-index: one reason 2.0 exists.'],
    logseq: ['ok', 'one database', 'Built to fix that; still one local database per graph.'],
    notion: ['good', 'sharded', 'Hundreds of billions of blocks over 480 shards, one workspace to a shard.'],
  } },
  { name: 'Two writers', question: 'You and an agent, at once', cells: {
    fictty: ['poor', 'per entry', 'A stale write is refused (with --if-rev), but two rows of one list still collide.'],
    obsidian: ['poor', 'per file', 'Sync merges text or keeps a conflict file.'],
    roam: ['ok', 'per block', 'Live, per block; the merge rules aren\'t documented.'],
    'logseq-og': ['poor', 'per file', 'File sync: whole-file conflicts.'],
    logseq: ['good', 'an ordered log', 'A stale write is rebased and its intent replayed.'],
    notion: ['good', 'per field', 'Per record and property; text merges by character.'],
  } },
  { name: 'Without the app', question: 'Read it with any tool', cells: {
    fictty: ['good', 'plain JSON', 'A JSON file; one screen holds it at a time.'],
    obsidian: ['good', 'plain Markdown', 'Any editor, git, grep.'],
    roam: ['poor', 'export or API', 'Only through an export or the API.'],
    'logseq-og': ['good', 'plain Markdown', 'The files are the truth.'],
    logseq: ['ok', 'CLI or mirror', 'An encoded SQLite file: use the CLI, an export or the Markdown mirror.'],
    notion: ['poor', 'API only', 'Through the API, an export, or pages kept offline.'],
  } },
];
