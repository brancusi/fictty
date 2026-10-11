# Plan

The order of work. Each phase ends with something a person can run. Decisions and their reasons
are in [design/architecture.md](design/architecture.md) and
[design/state-and-theme.md](design/state-and-theme.md); what we tried, built and learned before
this order was set is in [design/prototype-notes.md](design/prototype-notes.md). This file is the
sequence.

## Where we are (11 October 2026)

fictty stays a runtime on ratatui. json-render, OpenTUI and the ratatui ecosystem were tried
first; none has the loop an outside agent needs (push, patch, read back, answer in place), so we
build that and borrow formats and components from them. The store-and-theme prototype works: a
notes app, a ledger, a live market and a gallery, all JSON over one store and three themes
(`examples/showcase/`). It was merged into `main` on 11 October, and anyone can run it in one
line (`curl -fsSL https://fictty.com/try.sh | sh`) from the top of fictty.com.

The order below follows what the prototype showed: first land it, then close the agent loop
(`watch`, rebased text edits), then make records addressable, then layout and the kit.

## Done

### Extraction (2026-10-10)

The prototype grew on a branch of Thought Control (`crates/scene`, called thc-scene) and moved
here with its history.

- [x] Own repo, crate and binary: `fictty`, MIT
- [x] Renamed throughout (`fictty` binary, `FICTTY_*` environment variables, `fictty.sock`)
- [x] README, guide, architecture, time-travel proposal, identity explorations, agent files
- [x] fictty.com: the site in `web/`, with the posts and the landscape research

### The store and the theme (merged 2026-10-11)

- [x] Themes as their own layer: tokens, parts (a skin per primitive) and variants, loaded with
      `--themes`, switched by key or `fictty theme`; nodes ask for `variant` and `tone`
- [x] The store: the app's data apart from the UI, versioned entries, conditional writes
      (`put`, `merge`, `edit` with `--if-rev`), seeds, `--store` persistence, store actions on keys
- [x] `problems`: wrong shapes at binds and unknown variants, tones or `accept`, in words, in replies
- [x] `input` and `editor` on caretline (single-line for inputs), their text in the store;
      `doc` templates make an editor follow a list's selection
- [x] Markdown styling in editors (headings, lists, quotes, code), and clicking to place the caret
- [x] `accept: "number"` inputs; `$log` with clock times and refused writes; `$store`; `$theme`
- [x] The showcase: notes, a ledger with a helper process, a live market, a gallery, an agent's log

### Shipping it (2026-10-11)

- [x] fictty.com pages for [state-and-theme](design/state-and-theme.md) and
      [prototype-notes](design/prototype-notes.md); the first milestone post, with screenshots
- [x] Demos in one line: `try.sh` (the latest, on the landing page) and a frozen copy per post,
      pinned to its commit (`scripts/freeze-demo.sh`), tested like a reader runs them
      (`scripts/try-test.py`); a demo link component (name, date, version, copy)
- [x] A `copy` action (OSC 52) and `$self`: the showcase's Agent screen hands out commands that
      reach it from any terminal
- [x] A Tokens panel on posts (`scripts/blog-tokens.py`), and the `blog-post` and `ship` skills
- [x] A release workflow that builds macOS and Linux binaries on a version tag

### The kit, the library and the tour (2026-10-11)

Thought Control started building its screens on fictty in a second session, and asked for what it
needed as behaviour; what was generic landed here, what was app-specific stayed there.

- [x] fictty as a library: `App` (a UI value or file, a socket, a store, themes), `Host` (sources
      and actions in process, a `Notifier` to say the world changed), `Headless` for tests;
      upgrade re-runs the same command line, so an embedding app survives it
- [x] Store entries the host keeps (pages in a vault): loaded when shown, saved when settled,
      reloaded as an edit when they change at the host
- [x] `tree` (folds, filters, `$ptr` so store actions reach nested rows), `toggle`, `filter` on
      lists, tables and trees; lists and trees share their row options (`Rows`): marks, row
      variants, `lines`, `keep`, and a selection that follows its row's `key`
- [x] `stack` with `float` and `modal` (dialogs), `when` on nodes and layers (`$screen` makes
      panes responsive), `button`, toasts, `spinner`, scroll bars, `gap`, `min`/`max`
- [x] Editors: Markdown task items, `readonly`; the Mac editing keys with twins, the kitty
      keyboard protocol, bracketed paste, the mouse (shift-click, drag, double and triple click),
      the system clipboard; `tests/editing.rs` is the contract Thought Control wrote
- [x] Templates: `{key:11}` widths, `<<`, theme parts as tags, tabs from `|markup` values; tab
      titles as templates with a lead and a trail; chords (`"p h"`, `$keys`); a UI's `focus`;
      Esc leaves a field; `remove` with `where`; host source `args` from the selection and store
- [x] The tour: the one-liner opens nine chapters with a quest each, an autopilot and a firehose
      (114 fps at 12,000 messages a second beside a filtered 100,000-row list, release build)

## 1 · Land the prototype (now)

- [x] Review and merge `store-theme-prototype` into `main`
- [ ] Errors about a field show on the field (its border and title), not only on the status line,
      which screens may hide
- [x] An escape for `<` in template markup (`<<`)
- [ ] Problem checks only for nodes whose bind or data changed, to win back the 13% of flat-out
      push speed they cost
- [x] The first release, `v0.1.0`: binaries for macOS and Linux (Intel Macs cross-built), so the
      one-liners need no Rust; how to release is [releasing.md](releasing.md)
- [ ] Say when a copy didn't reach the clipboard (tmux, or a terminal that refuses OSC 52); today
      the screen says "copied" either way
- [ ] A pull request to json-render for its Tab focus bug (one line in the Ink renderer)
- [ ] Record the showcase (and the tour) for the site's front page
- [ ] The phosphor theme's heat colours: the market's watchlist is unreadable in it
- [ ] Close the Thought Control draft PR that carried the prototype, and drop `crates/scene` there
- [ ] Move the Thought Control examples (`examples/thc/`, the Today screens in `examples/pixels/`)
      to the Thought Control repo as a consumer; the showcase replaces them here

## 2 · Close the agent loop

The live session showed both gaps: the agent couldn't know the person had typed until told, and
`--if-rev` on text is refused by every keystroke in between.

- [ ] `watch`: wait for events (a store write by the person, a submit, a click on a named action,
      a key) without polling, with the revision to read from
- [ ] Text edits rebased onto the person's typing (caretline maps edits through, as it does for
      live editors); `--if-rev` stays strict for values
- [x] Keep unchanged data sources on push (a push fetches only new and changed ones)
- [ ] Validated batches: several writes applied together, or none, naming the bad ones
- [ ] The skill: the alphabet, the store, one full example, and the rules ("check with `screen`",
      "read before you write", "bind data to commands, never relay it")
- [ ] MCP server: a thin adapter over the socket; semantic tools first (`read`, `put`, `edit`,
      `watch`, `push`), raw keys and mouse as the fallback; the screen as an image
- [ ] Command policy: data-only by default for pushed UIs; commands from an allowlist; allow/ask/deny
- [ ] The test: rebuild the ownpurse walkthrough on fictty

## 3 · Records

- [ ] Collections keyed by id with an `order`; lists and tables bind to them (selection already
      follows a row's `key`, so an insert or a removal doesn't move it)
- [ ] Writes by JSON Pointer path, with a revision per record, so two writers on different rows
      don't collide
- [ ] Store actions that address a record: `remove` with `where` is done; `put` by id next
- [ ] More `accept` kinds where a form needs them: a date, one of a list, required
- [ ] JSON Patch (RFC 6902) as the patch dialect for values

## 4 · Layout and the kit

OpenTUI's component list is the target; each piece wraps the best crate behind one spec and the
theme.

- [ ] Layout on flexbox (a subset, via `taffy`): direction, size, grow, gap, padding, alignment,
      so the dense screens work at 80×24 as well as 160×46
- [x] Popups and dialogs (a `stack` with a floating, `modal` child), toasts
- [x] Scroll bars on lists, tables and trees; `readonly` editors for long text and logs
- [ ] Markdown rendered (not only styled in an editor), code with syntax colour, a diff view
- [ ] An embedded terminal for live command output (tui-term)
- [ ] Motion: transitions on push and theme change (tachyonfx)
- [x] A `copy` action onto the system clipboard (OSC 52), and `$self`, so a screen can hand out
      commands that reach it from any terminal
- [x] Editors: dragging to select, copy and paste with the system clipboard
- [ ] Editors: search
- [ ] Project components: JSON with named parameters, loaded from the project, `{"use": "kpi", …}`
- [ ] Review the drawing-only components (`gauge`, `big`, `bars`, `sparkline`, `chart`, `map`):
      keep what's compact and degrades well, move the rest to recipes
- [ ] A catalog: what this screen can render (alphabet, project components, registered widgets)
- [ ] Widget registry: any ratatui widget, registered by an embedding app (the escape hatch);
      waits until Thought Control finds something the kit can't do
- [ ] Rasterise pictures on a worker thread (a 15 Hz plot costs ~7 ms a frame on the main thread)

## 5 · Design system

The identity is the glass direction: every stroke a pane of tinted glass, overlapping like the
panes and layers of a terminal UI, generated on the terminal's cell grid from a seed. Explorations
are in [design/identity/](design/identity/) (open `explorations.html`); the light version is the
reference, the dark version isn't there yet. The showcase's `glass`, `paper` and `phosphor`
themes are the first in-terminal drafts.

- [ ] Push the glass direction further: the mark, the wordmark's legibility at full scatter, motion
- [ ] A dark version that works as well as the light one (`screen` blending on night, edge light)
- [ ] The wordmark generator as code: one seeded generator that emits SVG and PNG, and the chosen seed
- [ ] Favicon and small sizes; the social card
- [ ] Brand tokens: grounds, tints, neutrals, semantic colours, type for docs and site
- [ ] **fictty's default theme**: the same palette as an in-terminal theme (ANSI 16 and truecolor),
      in the theme format, so the brand and the product's default look are one thing
- [ ] Where themes live: the person's, the project's, or both merged
- [ ] Vocabulary: names for theme, throw/shape (push/patch) and the escape hatch, if the glass
      direction wants different words than the clay ones

## 6 · History

As proposed in [design/time-travel.md](design/time-travel.md): record inputs, snapshot sparsely,
then `history`, `state_at`, `screen_at`, `diff`, `seek`, `fork`, `replay`, marks, export and
import; `if_seq` on every write. First release: reproducible agent sessions. Tours come after.
The store's revisions and the `$log` are the first pieces of it.

## 7 · Standards

- [ ] json-render specs as an input format, mapped onto our nodes (agents already write them)
- [ ] A2UI input adapter onto assembly, once the spec reaches 1.0

## 8 · Apps and release

- [ ] Put the showcase beside a browser artifact for the same task, in front of a few people who
      supervise agents: do they want this in the terminal?
- [ ] Thought Control builds a screen on fictty (under way: nine views on `App` and `Host`)
- [ ] ownpurse's walkthrough runs on fictty
- [x] fictty as a library crate an app embeds, not only a binary
- [ ] Publish the crate
