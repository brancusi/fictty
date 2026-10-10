# Plan

The order of work. Each phase ends with something a person can run. Decisions and their reasons
are in [design/architecture.md](design/architecture.md); this file is the sequence.

## 0 · Extraction (done, 2026-10-10)

The prototype grew on a branch of Thought Control (`crates/scene`, called thc-scene) and moved
here with its history.

- [x] Own repo, crate and binary: `fictty`, MIT
- [x] Renamed throughout (`fictty` binary, `FICTTY_*` environment variables, `fictty.sock`)
- [x] README, guide, architecture, time-travel proposal, identity explorations, agent files
- [ ] Close the Thought Control draft PR that carried the prototype, and drop `crates/scene` there
- [ ] Move the Thought Control examples (`examples/thc/`, the Today screens in `examples/pixels/`)
      to the Thought Control repo as a consumer; replace them here with app-neutral demos

## 1 · Design system (next)

The identity is the glass direction: every stroke a pane of tinted glass, overlapping like the
panes and layers of a terminal UI, generated on the terminal's cell grid from a seed. Explorations
are in [design/identity/](design/identity/) (open `explorations.html`); the light version is the
reference, the dark version isn't there yet.

- [ ] Push the glass direction further: the mark, the wordmark's legibility at full scatter, motion
- [ ] A dark version that works as well as the light one (`screen` blending on night, edge light)
- [ ] The wordmark generator as code: one seeded generator that emits SVG and PNG, and the chosen seed
- [ ] Favicon and small sizes; the social card
- [ ] Brand tokens: grounds, tints, neutrals, semantic colours, type for docs and site
- [ ] **fictty's default theme**: the same palette as an in-terminal theme (ANSI 16 and truecolor),
      written in the theme format of phase 2, so the brand and the product's default look are one thing
- [ ] fictty.com: a one-page site with the live wordmark and the README's story
- [ ] Vocabulary: names for theme, throw/shape (push/patch) and the escape hatch, if the glass
      direction wants different words than the clay ones

## 2 · Alphabet, theme and layout

- [x] Themes as their own layer: tokens, parts (a skin per primitive) and variants, loaded with
      `--themes`, switched by key or `fictty theme`; nodes ask for `variant` and `tone`
      ([design/state-and-theme.md](design/state-and-theme.md))
- [x] The store: the app's data apart from the UI, versioned entries, conditional writes
      (`put`, `merge`, `edit` with `--if-rev`), seeds, `--store` persistence, store actions on keys
- [x] `problems`: wrong shapes at binds and unknown variants or tones, in words, in replies
- [ ] Layout on flexbox (a subset, via `taffy`): direction, size, grow, gap, padding, alignment
- [x] `input` and `editor` on caretline (single-line for inputs), their text in the store;
      `doc` templates make an editor follow a list's selection
- [ ] Collections keyed by id, writes by JSON Pointer path, a revision per record
- [ ] Markdown styling in editors (headings, lists, code), and clicking to place the caret
- [ ] Project components: JSON with named parameters, loaded from the project, `{"use": "kpi", …}`
- [ ] Review the drawing-only components (`gauge`, `big`, `bars`, `sparkline`, `chart`, `map`):
      keep what's compact and degrades well, move the rest to recipes
- [ ] A catalog: what this screen can render (alphabet, project components, registered widgets)
- [ ] Widget registry: any ratatui widget, registered by an embedding app (the escape hatch)
- [ ] fictty as a library crate an app embeds, not only a binary

## 3 · The agent loop

- [ ] `watch`: wait for events (a submit, a click on a named action, a key) without polling
- [ ] Keep unchanged data sources on push (today every push restarts all of them)
- [ ] Validated batches: several edits applied together, or none, naming the bad ones
- [ ] Command policy: data-only by default for pushed UIs; commands from an allowlist; allow/ask/deny
- [ ] The skill: the alphabet, one full example, and the rules ("check with `screen`", "bind data to
      commands, never relay it")
- [ ] Rasterise pictures on a worker thread (a 15 Hz plot costs ~7 ms a frame on the main thread)
- [ ] The test: rebuild the ownpurse walkthrough on fictty

## 4 · History

As proposed in [design/time-travel.md](design/time-travel.md): record inputs, snapshot sparsely,
then `history`, `state_at`, `screen_at`, `diff`, `seek`, `fork`, `replay`, marks, export and
import; `if_seq` on every write. First release: reproducible agent sessions. Tours come after.

## 5 · Standards

- [ ] MCP server: a thin adapter over the socket; semantic tools first, raw keys and mouse as the
      fallback; the screen as an image
- [ ] A2UI input adapter onto assembly, once the spec reaches 1.0
- [ ] JSON Patch (RFC 6902) as the patch dialect

## 6 · Apps

- [ ] Thought Control builds a screen on fictty
- [ ] ownpurse's walkthrough runs on fictty
- [ ] Publish the crate
