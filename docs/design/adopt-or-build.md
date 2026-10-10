# Adopt or build

Should fictty exist as its own runtime, or should we adopt, build on, or fold into something that
already exists? A hard look, as of 10 October 2026. The evidence is in the
[landscape dossiers](../research/landscape/); the positioning is in
[Interfaces you throw away](../blog/2026-10-disposable-interfaces.md); the scope is in
[scope.md](scope.md).

## What we'd be replacing

To judge an option, be clear about what fictty actually does. Strip out the things other tools
already do well and five jobs remain:

1. **The UI is one serializable value** in the runtime's state, pushed whole or patched by node id,
   with view state (selection, scroll, focus) kept by id across pushes.
2. **Data sources bound to nodes**: commands and streams the runtime runs itself, so data never
   passes through the model.
3. **Behaviour in the runtime** at frame rate: keys, focus, selection, scrolling, mouse, layers.
4. **Exact read-back**: the frame as text (`screen`) and the state as JSON (`get --state`).
5. **A long-lived process an outside agent attaches to** over a socket or CLI, from any harness,
   in any terminal, over SSH.

Everything else (drawing cells, layout, pixels, text editing) we already take from ratatui,
caretline and resvg. The question is whether any tool does jobs 1 to 5, or most of them, well
enough that building them is a waste.

## The options

Scores are our judgement from the dossiers. "Deciding question" is the one fact that would flip the
answer.

### A2UI as the internal format

**Gain:** a standard with Google behind it, 16,700 stars, renderers in Lit, Angular, React,
Flutter and three terminal languages; agents increasingly trained on it; a clean split between
components by id and a data model by JSON Pointer.

**Lose:** A2UI has no place for data sources bound to commands, key maps, layers, pixels or theme
(v1.0 removed theming on purpose), and no notion of read-back. All of that would ride in custom
catalogues and extension fields, producing "A2UI-shaped fictty" that no other renderer can draw.
Its flat adjacency list is worse for an agent to hand-write and to read back than a nested tree.
It's had three spec versions in ten months, and v1.0 is still a release candidate.

**Deciding question:** does A2UI 1.0 go stable *and* grow an extension for bound data sources? If
both, re-run this.

**Verdict:** don't adopt as the internal model. **Speak it as an input**: an A2UI v1.0 adapter that
lowers onto assembly, tested against the official v0.9.1 and v1.0 examples. Align vocabulary
(surface, catalogue, data model, pointer) so the mapping is lossless one way. Copy its structured
validation errors (`{code, path, message}`), placeholders for missing references, and
`sendDataModel` (state attached to every event in `watch`).

### json-render (and its Ink renderer)

**Gain:** the most adoption in the camp (about 591,000 monthly npm downloads for the Ink renderer,
inflated by transitive installs), thirty renderers, RFC 6902 JSON Patch streaming, a catalogue that
generates its own prompt, and a terminal renderer that already does tables, forms, charts and
Markdown.

**Lose:** it's a TypeScript library embedded in someone's Ink app. No process, no socket, no
patch-by-id from outside, no data sources, no read-back. Ink is React in Node; fictty's measured
numbers (1,000 patches a second with frames under 1 ms) rest on ratatui's cell diff in one Rust
process. Its spec carries an expression language (`$cond`, `$computed`, `watch`) that breaks our
"data, never code" rule.

**Deciding question:** does json-render ship an attachable host (a long-lived process with a
socket and read-back)? If it does, most of fictty's distinct ground becomes theirs, with far more
distribution.

**Verdict:** don't adopt as the runtime. **Adopt its patch dialect** (RFC 6902, plus RFC 7396
merge), **generate the agent skill from the schema** the way `catalog.prompt()` does, and accept
the logic-free subset of a json-render Ink-catalogue spec as a second input after A2UI. State our
no-logic rule in contrast to theirs.

### Textual (and textual-serve)

**Gain:** the richest widget set and app model of any TUI framework, CSS-like layout and theming,
the best headless test story (Pilot plus snapshots), and a browser path today.

**Lose:** a Python rewrite; startup and per-frame cost our stress numbers wouldn't survive; no
single static binary for SSH boxes; caretline and resvg gone; state in widget objects, not one
value. The company behind it wound down in 2025, and the sole maintainer wrote on 9 October 2026
that he can't always make time for issues.

**Deciding question:** none realistic. The bus factor alone rules it out as a foundation.

**Verdict:** no. Take ideas: theme expressiveness at TCSS level (as data), Pilot-style tests,
streaming Markdown as a primitive.

### OpenTUI

**Gain:** the only serious alternative substrate. A Zig core with images, kitty graphics, mouse
pointers, an embedded terminal, code, diff and Markdown views, SSH serving, a headless test
renderer, and a TypeScript home next to A2UI, json-render and MCP Apps (adapters would be
shorter). Proven at scale by opencode.

**Lose:** a rewrite of the runtime in TypeScript; a Bun or recent Node dependency plus a native
library per platform; caretline only through FFI or a port; a pre-1.0 API that changes weekly and
is steered by one product's needs. ratatui already gives us more speed than we need.

**Deciding question:** do we find ourselves rebuilding what OpenTUI already has (Markdown, diff and
code views, an embedded terminal, SSH serving), or do the TypeScript adapters become most of the
work? If yes to either, revisit.

**Verdict:** not now. Stay on ratatui. Copy OpenTUI's test API shape (`renderOnce`,
`captureCharFrame`, mocked input) for a fictty library API, and make code, diff and Markdown
first-class primitives.

### Lavish (lavish-axi)

**Gain:** the right loop, already working: the agent writes, the person points at the exact
thing, the agent waits cheaply with a long poll, then revises. Local, no account, any agent, about
4,000 stars and daily releases.

**Lose:** it's a browser editor for static HTML files. Data is frozen at write time; read-back is
a DOM snapshot plus annotations; every revision regenerates the page; it needs a browser and Node
(over SSH, a tunnel or a share link). Adopting it means abandoning the terminal, live data and
exact state read-back, which is all of fictty's distinct ground.

**Deciding question:** do the people we're building for actually prefer reviewing in a browser
tab? That's a demand question, answered with users, not architecture.

**Verdict:** don't adopt; **interoperate**. `fictty export --html` hands a screen that became a
document to Lavish or an Artifact. Take its `watch` shape (structured feedback anchored to a
locator, with explicit "ended" and "everyone gone" exits), delivery receipts, layout warnings sent
to the agent, and the AXI rules for the CLI. Don't rebuild annotation or whiteboards.

### TUIOS or herdr plugins

**Gain:** distribution. herdr has 43,000 stars and a plugin marketplace; TUIOS has an Inbox,
popups, OSC 7501 and a JSON socket. Panes, sessions, remote attach and agent status for free.

**Lose:** neither has a UI model an agent can write, patch by id or read back as state. herdr's
plugin v1 is terminal UI only (a plugin is a program in a pane); TUIOS's `ask-human` is a question
with fixed answers. Building fictty *as* a plugin means building fictty anyway, in Go against a
single-author API (TUIOS) or inside one frame's plugin contract (herdr), and losing every other
frame.

**Deciding question:** does herdr ship declarative plugin UI, or does TUIOS grow `ask-human` into
forms and tables bound to commands? Then the content layer gets claimed from above, and the right
move is to become their renderer or their format.

**Verdict:** don't build on them; **be their content**. Emit OSC 7501 and OSC 9/777 when a screen
waits; call `herdr pane report-agent` when inside herdr; ship a herdr plugin manifest and recipes
for `tuios popup -- fictty run`, tmux 3.8 modal panes and zellij floating panes. Build **no frame
features**: no sessions, detach, remote attach or multi-agent status.

### MCP Apps

**Gain:** the first official MCP extension, supported by Claude, ChatGPT, VS Code and Goose; a
clean split between a reviewed template and the data that fills it; declared, deny-by-default CSP.

**Lose:** it's HTML in a sandboxed iframe in a chat host. No terminal renderer, no
agent-authored screen (templates are written by a developer ahead of time), no live patching by
id.

**Deciding question:** none. It's a different surface.

**Verdict:** don't adopt as a surface. **Expose fictty as an MCP server** (semantic tools derived
from the current UI value, with a focus lens to keep the list short, as Raxol does; raw keys and
mouse as fallbacks). Borrow the vocabulary: tool-input and tool-result, display modes, declared
scope as the model for a command policy. Later, a small HTML renderer could show a fictty value as
an MCP App.

### Claude Code mods

**Gain:** the biggest single audience of terminal agents, already able to draw panes, bands and
rows with live data; Claude writes and hot-reloads mods itself. For a Claude Code user, a mod plus
a good skill covers much of fictty's job with nothing to install.

**Lose:** proprietary, Claude Code only, code-first, redraws capped at 30 a second, and no read-back
of what a mod drew for the model. Building fictty as a mod would give up every other harness, SSH
boxes, and the frames.

**Deciding question:** does Anthropic ship a first-party "draw this JSON" mod with a read-back
tool? If so, fictty's Claude Code story becomes "use that", and fictty lives on outside one
harness.

**Verdict:** don't build *on* it; **bridge into it**, soon. A small mod that renders a fictty value
in a pane (degrading layers and pixels to Box, Text and Raster) with a `screen` tool for
read-back turns the biggest substitute into a distribution channel. Then opencode and pi
equivalents.

### Claude Code Artifacts

**Gain:** hosting, versions, sharing, comments delivered to a watching session, connector data,
a shared database. A real answer to the ten-minute screen for anyone with a browser and a
claude.ai login.

**Lose:** no pane, no local command as a source, no frame read-back, one vendor and one account,
not available through Bedrock or Vertex or API keys.

**Deciding question:** same as Lavish: do our users always have a browser beside the agent and
prefer it?

**Verdict:** **cede share-and-keep entirely.** Export to HTML and hand off. Treat its
comment-to-agent loop as the minimum bar for `watch`, and copy declared capabilities for data
sources.

### Laura, Raxol, claude-canvas

None of them is a foundation. **Laura** (0 stars, Rust, very active) has the same loop for
documents, not components; take its overflow reports after a push, dry runs, holding writes while
the person types, and never-reused ids. **Raxol** (80 stars, Elixir, one maintainer) is code-first;
take tree-derived MCP tools with a focus lens and test verbs that match agent verbs.
**claude-canvas** was abandoned after a week; take the proof of demand and an "answer by pointing"
example that returns `selected` or `cancelled`.

### Screen drivers (tui-test, agent-tui, tui-use)

**Gain:** they work with every program ever written.

**Lose:** they recover structure from cells with adapters; fictty publishes its structure.

**Verdict:** don't build a driver, and don't compete with them. **Make fictty the easiest screen
for them to read**: an outline view in roles and refs, an agent-tui adapter that reads
`get --state`, and use **tui-test** for emulator-level conformance tests (does `screen` match what
a real terminal shows, cell for cell?).

### ratatui and Ratzilla

**Keep ratatui.** Its immediate mode matches our pure `view` and our one-source-of-truth rule;
everything fictty's speed claims rest on comes from it. **Ratzilla** is the cheapest browser
surface later: compile model, state and view to WASM and replace only `runtime.rs`. Keep `state` and
`view` WASM-clean now. (Unverified: whether caretline and resvg build for `wasm32`.)

## Summary table

| option | adopt as foundation? | what we do instead |
|---|---|---|
| A2UI | no (not yet) | input adapter, shared vocabulary, its error and event shapes |
| json-render | no | RFC 6902/7396 patches, schema-generated skill, logic-free spec as input |
| Textual | no | ideas only |
| OpenTUI | not now | revisit on the stated trigger; copy its test API |
| Lavish | no | HTML export and handoff; its `watch` shape; AXI rules |
| TUIOS / herdr | no | be their content: OSC 7501, report-agent, plugin manifest, popups |
| MCP Apps | no | fictty as an MCP server; borrow the vocabulary |
| Claude Code mods | no | a bridge mod with a read-back tool |
| Claude Code Artifacts | no | cede share-and-keep; HTML export |
| screen drivers | no | be readable by them; tui-test for conformance |
| ratatui | **yes, keep** | plus Ratzilla later, Taffy for flexbox |
| caretline, resvg | **yes, keep** | |

## Recommendation

**Build the runtime, and only the runtime.** Nothing in the landscape does jobs 1 to 5 together.
The closest projects each do one or two: Laura has the loop for documents, the format renderers
have the values without the loop, the frames have read-back of text without state, Artifacts have
the return path without the terminal or local data. That position is real, narrow, and probably
temporary, so build it small and make it easy to embed.

**Build:**

- `watch`, with structured, node-anchored events and state attached; a checkpoint or question
  component that waits for an answer.
- Semantic read-back that's better than an accessibility tree: `get --state`, an outline view in
  roles and refs, overflow and clipping reports after every push, health per node and source.
- A declared, reviewable command policy for data sources.
- Code, diff and Markdown primitives; instruments (step-through, scrubber, side-by-side) as recipes.
- History and replay, as already proposed.

**Adopt:** ratatui, caretline, resvg, Taffy (for flexbox), RFC 6902 and 7396, OSC 7501 and
OSC 9/777, tui-test for conformance, VHS for demo captures, `codex-mermaid` (Apache-2.0) if we want
a Mermaid node.

**Interoperate with:** A2UI and json-render (inputs), herdr, TUIOS, tmux and zellij (hosts),
Claude Code, opencode and pi (bridges), MCP (a server face), Artifacts and Lavish (HTML export),
AG-UI (an optional second transport, emitting snapshot plus JSON Patch deltas).

**Don't build:**

- frame features: sessions, detach, remote attach, multi-agent status;
- hosting, sharing, comments or HTML review;
- an expression language, or any logic in the UI value beyond substitution;
- a new UI standard;
- a screen driver for other programs;
- a JavaScript renderer, or a rewrite onto another substrate;
- an education product. Teaching screens are recipes and a skill, not a product line.

## What would change the verdict

| if this happens | then |
|---|---|
| json-render or an A2UI renderer ships an attachable terminal host with read-back | stop competing on the loop; make fictty a renderer or adapter for their format, or contribute the loop upstream |
| Anthropic ships a first-party JSON mod with model read-back | point Claude Code users at it; fictty's audience is everyone else |
| herdr ships declarative plugin UI, or TUIOS grows forms and tables | offer fictty's value as their plugin UI format, or render inside their contract |
| A2UI 1.0 goes stable with bound data sources | consider A2UI as the internal value, with fictty's extras as an extension |
| we start rebuilding OpenTUI's features, or TypeScript adapters dominate | re-run the OpenTUI decision |
| user tests show our users always prefer a browser tab | shrink fictty to remote and headless use, and lean on HTML export |
| a small study shows interactive screens don't improve understanding over text | keep fictty for supervision and live data; drop the teaching claim |

The cheapest of these to check is the last but one. Before building much more, put fictty and an
Artifact side by side in front of five people supervising a real agent task, and watch which one
they use.
