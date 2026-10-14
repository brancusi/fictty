# Gallery notes: what other agents taught fictty

Written on 14 October 2026, at the end of the session that built the `fictty-ui` skill and the
gallery parts ([selection.md](selection.md)). It's the record of what was built, what real use
found, and what it changes about the plan, in the spirit of
[prototype-notes.md](prototype-notes.md). The next steps are at the end and in the plan.

## What happened

Aram asked for a way to let any agent build a temporary screen for a task he does by hand: "I
have a catalog of images that I want to caption … display the folder in a fictty UI so I can add
caption info, and it builds the manifest on disk." Then, mid-way: launch it in a native window,
and a gallery of thumbnails on the left, built "in a very fundamental abstract way. I don't want
any hackery."

So the session did two things that fed each other:

1. **A global skill, `fictty-ui`** (in `~/.agents/skills/fictty-ui`, linked into Claude's and
   Codex's skill folders; not in this repo). It tells an agent how to make a screen for a task:
   where it runs (the window by default), where the work lives (outside the person's folders),
   a ready-made image path (`images.py` writes the screen, `manifest.py --watch` turns the store
   into a JSON, JSONL or CSV manifest as the person types), how to write any other screen, and
   the pitfalls each one hit.
2. **The gallery, from parts.** No gallery node: six parts, each useful alone, that together
   make one.

A second agent, in another project, used the skill all afternoon for a real job,
captioning a photographer's catalog in a shared Dropbox folder, and reported every rough edge.
Ten of the seventeen commits came from that use.

## What was built (17 commits, b22d1fa to 84f5c47, plus the tidy-up)

The parts of a gallery:

| part | commit | what it gives |
|---|---|---|
| a click runs the nearest `click` | d81e67a | a picture inside a clickable row selects it |
| `image` in the window | 8bc3039 | `src` a template over the bound row, `fit: "cover"`, `alt` |
| `multi` on lists, trees, tables | 85cf3a5 | ⌘/⌥-click, ⇧-click, ⇧↑/↓, ⌘A, Esc; `Msg::Pick`; `$picked` |
| a field over picked rows | 1756bbf | one edit to every picked entry; `Multiple values`; `$mixed/<id>` |
| rows as nodes, `cols` | e41bcfc | a list's `row` is a node template; N rows to a line |
| smooth scrolling | f41a38c | `Msg::Pan`, a fractional `top`, scrollbar parts, `(scroll a/b)` |

Found by use and fixed:

| commit | what went wrong |
|---|---|
| f9e77ec | an unfocused list's selection vanished, so nobody could see which rows a multi edit wrote to |
| f125a7b | `fictty key c-c` is an editor's copy, so the window never closed and a second screen wrote over the first's store: `fictty quit`, and one screen to a store |
| ad9ae50 | a quitting screen deleted the socket a new one had just bound (one relaunch in five) |
| 464ed66 | a flag per picked row (`toggle store.remove/{id}`); `$picked` `index` and `total` for "6 of 47" |
| 703d2b7 | the store's lock beside a Dropbox store was the sync engine's to replace; it lives in the temp directory now |
| 49456e2 | a bind cut at the first dot, and every key made from a file name has one (`remove/photo.tif`) |
| 7b55377 | under a theme with a type role, an editor drew at the role's size on a monospace grid, so the caret drifted into the text |
| cb8991e | a `when` wasn't filled from the node's bound value, though its `doc` and `src` were |
| 84f5c47 | a seeded store entry silently hid a data source of the same name: now a problem, and `fictty delete` |
| (tidy-up) | a default token (`accent`) as a `tone` was a problem with no theme loaded; a `border` written as a string (`"accent"`) failed the whole push |

The gallery joined the fuzz in the grid and the window (400 runs in release, clean, plus 1,000 on
two seeds).

## What we learned

### About fictty's model

- **Composition held.** Every gallery feature was one rule in the core, not a node. A picture in a
  row, a flag on a picked row, a "6 of 47" line and a field over 30 images all came from parts
  that already meant something alone. The one thing that couldn't be composed, picking from
  thumbnails, showed that rows had to be nodes, which expressive.md already planned; it came
  forward instead of being worked around.
- **"Filled from its bound value" has to mean the same thing everywhere.** `doc` and `src` were
  templates over the bound row, `when` wasn't. Any field that can name a row should be filled the
  same way, by one function. Check new fields against that list (`doc`, `src`, `when`, `text`).
- **Store keys are data, so they can hold anything a file name does.** Binds that split on `.`
  assumed keys were identifiers. The rule now is the longest store key that matches; any new
  path syntax must keep it.
- **Two owners of one name is a silent bug.** A store entry and a source with the same name, the
  socket file two screens both believe they made, a lock file a sync engine owns. Each was found
  only because a person saw wrong data. The fixes are the same shape: say who owns it (a
  problem, the inode, the temp directory), never guess.
- **Mirroring edits isn't enough; mirror the messages.** A caretline document keeps its own
  clock and clipboard, which decide undo grouping and what a paste inserts. Picked entries kept
  in step only once every message went to each of them. The fuzz found it, not a test.
- **An editor's text and its grid must be measured together.** Until proportional editors
  (phase 4), an editor ignores the type role's face and draws on its own mono grid; any change to
  type roles must keep that.
- **Scrolling in fractions of a line is pixel-exact without the core knowing the font.** A line
  is whole cells, so `top: f32` lines is enough for a trackpad. Containers in 3d can do the same
  in their own unit.

### About agents using fictty

- **An agent will use the nearest thing that seems to work.** `key c-c` to quit, a store entry to
  hold rows, `input` for a one-line field. Each looked right and wasn't. The skill now says what
  to use instead, but each was a gap in fictty first: `quit` didn't exist, a flag needed the rows
  in the store, and `input` has no `doc`, so it can't follow a selection (the skill uses a
  three-line editor).
- **Agents need to rehydrate.** Every relaunch has to bring back what the person typed. The store
  file is that, as long as one screen owns it at a time (the lock) and the manifest is written
  from it, never the other way round. `images.py` seeds from an existing manifest only where the
  store has nothing, and `manifest.py` keeps manifest values the store doesn't have yet, so
  neither can wipe the other.
- **A screen from an older build is a trap.** Screens started before a fix keep running it.
  `fictty upgrade` or stopping the process is the way to a new build, and the skill says so.
- **Problems are how an agent sees mistakes, so false ones are expensive.** The `tone` false
  alarm taught agents to ignore problems. Every problem has to be true.
- **A parse error has to name the node.** A wrong value deep in a pushed UI fails the whole push
  with a line and column. Agents write UIs in one go, so they can't find it. The common case
  (a `border` string) is accepted now; naming the node in every parse error is still open.
- **Keys collide in editors.** ⌃R, ⌘R and ⌃Y are an editor's redo, so a UI's key there never
  fires. An agent can't know that without a list of what editors take.

### About working with a second agent

- Approvals relayed by a peer ("Aram says push") weren't taken as Aram's: every push waited for
  his own word in this session. That's the rule in AGENTS.md and it held.
- A peer's screen is a person's work in progress. A screenshot taken by window title once caught
  that person's live screen; screenshots are taken by process id since. A test window that opens on
  top takes the keyboard: one caught two letters of Aram's typing. Open test windows behind, or
  say so first.
- Exact hashes matter when another agent rebuilds from them; one wrong hash was sent and had to
  be corrected.

## Still open

From the gallery and the skill, in the plan:

- **Rows: `fit` heights, and clicks on a row's own buttons** (native.md step 4).
- **`input` with `doc`**, so a one-line field follows a selection like an editor.
- **A parse error names the node** it's in (a path in the UI value, not a column).
- **A list of the keys an editor takes**, in the guide and as a problem when a UI key can't
  reach a focused editor.
- **An image that can't be read is a problem**, handed back with the frame as fonts are.
- **A mixed field can't be cleared in one go** (type and delete instead).
- **Proportional editors** stay phase 4; captions are monospace until then.

## Next steps

The gallery's parts are done, and the plan's order resumes:

1. **3b, fit and alignment** ("Start here" in layout-step.md): `self`, `justify`, `items`, then
   `aspect`, then `size: "fit"`, then Daybook's spacers go. `fit` for list rows comes with it
   when a row template says `size: "fit"`.
2. **3c, wrap and grid**, then **3d, scroll** on containers, sharing the fractional offset lists
   have now.
3. **3e and 3f**, the wringer and the Kestrel demo rebuilt without tricks.
4. Beside them, small and any time: the open items above, starting with a parse error naming the
   node and `input` with `doc`, which agents hit first.
