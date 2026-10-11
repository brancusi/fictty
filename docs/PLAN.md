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
(`examples/showcase/`). It was merged into `main` on 11 October.

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

## 1 · Land the prototype (now)

- [x] Review and merge `store-theme-prototype` into `main`
- [ ] Errors about a field show on the field (its border and title), not only on the status line,
      which screens may hide
- [ ] An escape for `<` in template markup
- [ ] Problem checks only for nodes whose bind or data changed, to win back the 13% of flat-out
      push speed they cost
- [ ] fictty.com pages for [state-and-theme](design/state-and-theme.md) and
      [prototype-notes](design/prototype-notes.md), as designed pages
- [ ] A pull request to json-render for its Tab focus bug (one line in the Ink renderer)
- [ ] Record the showcase (and the tour) for the site's front page
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
- [ ] Keep unchanged data sources on push (today every push restarts all of them)
- [ ] Validated batches: several writes applied together, or none, naming the bad ones
- [ ] The skill: the alphabet, the store, one full example, and the rules ("check with `screen`",
      "read before you write", "bind data to commands, never relay it")
- [ ] MCP server: a thin adapter over the socket; semantic tools first (`read`, `put`, `edit`,
      `watch`, `push`), raw keys and mouse as the fallback; the screen as an image
- [ ] Command policy: data-only by default for pushed UIs; commands from an allowlist; allow/ask/deny
- [ ] The test: rebuild the ownpurse walkthrough on fictty

## 3 · Records

- [ ] Collections keyed by id with an `order`; lists and tables bind to them, and selection
      follows the id, so an insert or a removal doesn't move it
- [ ] Writes by JSON Pointer path, with a revision per record, so two writers on different rows
      don't collide
- [ ] Store actions that address a record (`put` and `remove` by id, not the selected row only)
- [ ] More `accept` kinds where a form needs them: a date, one of a list, required
- [ ] JSON Patch (RFC 6902) as the patch dialect for values

## 4 · Layout and the kit

OpenTUI's component list is the target; each piece wraps the best crate behind one spec and the
theme.

- [ ] Layout on flexbox (a subset, via `taffy`): direction, size, grow, gap, padding, alignment,
      so the dense screens work at 80×24 as well as 160×46
- [ ] Popups and dialogs on layers (confirm, a picker, a toast)
- [ ] Scroll views for long text and logs; scrollbars
- [ ] Markdown rendered (not only styled in an editor), code with syntax colour, a diff view
- [ ] An embedded terminal for live command output (tui-term)
- [ ] Motion: transitions on push and theme change (tachyonfx)
- [ ] Editors: dragging to select, the system clipboard (OSC 52), search
- [ ] Project components: JSON with named parameters, loaded from the project, `{"use": "kpi", …}`
- [ ] Review the drawing-only components (`gauge`, `big`, `bars`, `sparkline`, `chart`, `map`):
      keep what's compact and degrades well, move the rest to recipes
- [ ] A catalog: what this screen can render (alphabet, project components, registered widgets)
- [ ] Widget registry: any ratatui widget, registered by an embedding app (the escape hatch)
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
- [ ] Thought Control builds a screen on fictty
- [ ] ownpurse's walkthrough runs on fictty
- [ ] fictty as a library crate an app embeds, not only a binary
- [ ] Publish the crate
