# A2UI (Agent-to-User Interface)

- **Maker:** Google (the A2UI project; Opal, Flutter and Gemini Enterprise teams contribute), with
  CopilotKit as the most active outside partner
- **URL:** https://github.com/a2ui-project/a2ui, https://a2ui.org
- **Status (10 October 2026):** Apache-2.0, 16,657 stars. Repository created 24 September 2025;
  publicly launched as v0.8 in mid-December 2025 (secondary coverage gives 15 December). The
  README calls v0.9.1 "the current production release", v1.0 a **release candidate** ("previously
  known as 0.10"), and v0.8 legacy. Python packages `a2ui-agent-sdk` v0.8.0 and `a2ui-core` v0.3.0
  were released 8 October 2026; pushed to daily. The README still says "Early stage public
  preview ... Expect changes." `@a2ui/react` had about 511,000 npm downloads in the month to
  8 October 2026.

## What it is

A wire format plus reference renderers for UI that an agent describes and a client draws with its
own native widgets. The agent sends a stream of JSON envelopes; each envelope creates a
**surface**, adds or replaces **components** on it, updates its **data model**, or deletes it. The
client holds a **catalog** of trusted components and draws only what the catalog allows. Its slogan
is "safe like data, but expressive like code."

## The problem it's solving

An agent, often a remote one across a trust boundary (one agent delegating to another over A2A),
needs to show a person something richer than text, without the host executing code the agent
wrote and without every host re-implementing every agent's UI. A2UI's answer is the old
server-driven-UI answer: send a description, let the host render it in its own design system.

## Its path / bet

Be the **portable contract** under everyone else. Stay transport-agnostic (AG-UI, A2A, MCP, SSE,
WebSockets, REST), framework-agnostic (Lit, Angular, React, Flutter, Jetpack Compose alpha;
SwiftUI planned) and model-agnostic. Let hosts own the look: v1.0 removed the theme properties so
styling "defers entirely to the target framework's native theme." Grow through renderers written
by others rather than one blessed client.

## How it works

- **Envelopes (v1.0).** Each line is a JSON object with exactly one of `createSurface`,
  `updateComponents`, `updateDataModel`, `deleteSurface`, `callRendererFunction` or
  `agentFunctionResponse`. v1.0 lets `createSurface` carry the whole initial component list and
  data model in one message.
- **Flat adjacency list.** Components are a flat list with `id`s; containers reference children by
  id; the one with `"id": "root"` mounts under the surface. An update replaces components by id,
  and missing references render as placeholders, which is what makes progressive streaming work.
- **Data model.** A JSON object per surface; components bind with JSON Pointer (`{"path":
  "/user/name"}`), with relative scopes inside list templates and `@index`. `updateDataModel`
  replaces the value at a pointer. Inputs are two-way bound; `sendDataModel: true` makes the
  renderer attach the whole data model to every message it sends back.
- **Actions.** A button either sends an `event` (name plus context) to the agent or runs a
  catalog-registered local `functionCall` (`openUrl` and the like).
- **Basic catalog.** 18 components (Text, Image, Icon, Video, AudioPlayer, Row, Column, List, Card,
  Tabs, Divider, Modal, Button, CheckBox, TextField, DateTimeInput, ChoicePicker, Slider) and
  functions for validation (`required`, `regex`, `email`...), formatting (`formatString`,
  `formatNumber`, `formatDate`, `pluralize`) and logic (`and`, `or`, `not`). Custom catalogs are
  JSON Schema with strict naming rules.
- **Prompt, generate, validate.** The spec prescribes a loop: put the schema and catalog in the
  prompt, generate, validate, and feed structured errors (`VALIDATION_FAILED`, `UNALLOWED_CHILD`,
  with a JSON Pointer) back to the model.
- **A2UI Express (proposal).** A compact, Python-like DSL (`root = Column([header, ...])`), with an
  ANTLR grammar, that a host compiles into v1.0 JSON. It is gated behind an environment variable
  and exists because JSON is expensive for models to emit. This is A2UI conceding OpenUI's point.

## Strengths

- The broadest backing in the camp: Google products in production (Opal, Gemini Enterprise via
  A2A, ADK Web, Flutter's GenUI SDK), CopilotKit and AG-UI as the default transport, Android's
  Compose team, and a long tail of community renderers (Swift, MUI, Lynx, React Native).
- The surface/id/data-model split is clean: structure and data update separately, by id and by
  pointer, which is the right shape for incremental edits.
- Security story is coherent: no code crosses the wire; the catalog is the allow-list.
- Terminal renderers already exist: `a2ui-ink` (Ink, v0.9) and the Rust `a2ui` crate (ratatui,
  Slint, egui and others). Both are tiny (the crate shows about 150 downloads; `a2ui-ink` 4
  stars), but they prove the mapping.

## Weaknesses / limits

- **Not stable.** Three spec versions in ten months with breaking changes (v0.8 to v0.9 renamed
  messages and changed the schema; v1.0 removed theming). The main renderers mark v1.0
  "planned". Anyone adopting it today adopts a moving target.
- **Verbose for models.** A flat list of fully-specified JSON objects costs tokens; OpenUI's own
  benchmark puts A2UI at about 3x OpenUI Lang (a competitor's number, but A2UI's Express proposal
  suggests Google agrees with the direction).
- **One-way render, no read-back of what was drawn.** The agent learns about the UI through
  actions and the data model, never through the rendered result. There is no notion of focus,
  selection or scroll position the agent can query.
- **Data flows through the agent.** Live values arrive as `updateDataModel` messages the agent
  sends. There is no declarative data source the renderer polls on its own. (The "agent" here is
  a process, not necessarily the model, but the format gives no way to bind a command or stream.)
- **Logic is creeping in.** `and`/`or`/`not`, validation checks and `formatString` interpolation
  are small, but they are the start of an expression language.
- Criticism in public threads: catalogs make "every UI the same"; trusting a model to lay out UI
  invites impersonation (InfoQ's roundup of HN and Reddit).

## Relation to fictty

**Adopt (as an input), and partly compete.** A2UI is the format fictty is most likely to be asked
to speak, and its terminal renderers occupy the "agent writes data, shows up in the terminal" cell
fictty sits in. But A2UI is a format and a renderer contract; fictty is a running process with a
loop. They overlap on the UI value; they don't overlap on read-back, local behaviour, bound data
sources or replay.

## Could fictty adopt it instead of building?

Partly, and not yet as the internal model.

- **What maps cleanly:** surface ≈ a fictty screen (or a layer); flat components with ids ≈ nodes
  patched by id; the data model with JSON Pointer ≈ fictty's `data`; actions ≈ fictty actions;
  custom catalogs ≈ fictty's alphabet plus project components.
- **What A2UI has no place for:** data sources bound to commands and streams (fictty's central
  claim), key maps, layers (callouts, spotlights, arrows), pixels, a theme (v1.0 removed it on
  purpose), and view state the agent can read. These could ride in a custom catalog and extension
  fields, but then the payload is "A2UI-shaped fictty," readable by no other renderer.
- **Cost of switching the internal value:** flat adjacency lists are worse to hand-write and to
  read back than fictty's nested tree; the spec is still breaking things; and fictty's rule of
  "substitution only" conflicts mildly with A2UI's logic functions.

Verdict: keep fictty's own value as the internal model, write an **A2UI v1.0 input adapter** that
lowers onto assembly (already on the plan), and use A2UI's vocabulary (surface, catalog, data
model, pointer) where fictty has an equivalent, so the mapping stays lossless one way. Revisit if
v1.0 goes stable with renderer adoption and an extension for bound data sources appears.

## What fictty should take from it

- **Separate structure from data in the patch verbs.** `updateComponents` by id and
  `updateDataModel` by JSON Pointer are two different verbs; fictty should keep that split explicit
  in its patch dialect (and the plan's RFC 6902 item fits the data side).
- **Structured validation errors with a pointer,** fed back to the model. fictty's `push` should
  reject with `{code, path, message}` in the same shape.
- **Placeholders for missing references** so a half-streamed UI still draws.
- **`sendDataModel`**: on every action, hand the agent the state it needs. fictty's `watch` should
  return the relevant state with the event, not just the event.
- **A compact authoring syntax is coming everywhere** (Express, OpenUI Lang). fictty's JSON can
  stay the wire format, but agents may want a terser way to write it.

## Sources

- https://github.com/a2ui-project/a2ui (README, read 10 October 2026)
- https://github.com/a2ui-project/a2ui/blob/main/specification/v1_0/docs/a2ui_protocol.md
- https://github.com/a2ui-project/a2ui/tree/main/specification/proposals/express
- https://github.com/a2ui-project/a2ui/blob/main/docs/public/reference/renderers.md
- https://github.com/a2ui-project/a2ui/blob/main/docs/public/ecosystem/renderers.md
- https://github.com/a2ui-project/a2ui/blob/main/docs/public/ecosystem/a2ui-in-the-world.md
- https://infoq.com/news/2026/07/google-a2ui-genui
- https://sdtimes.com/ai/google-launches-a2ui-project-to-enable-agents-to-build-contextually-relevant-uis/
- https://www.blogdumoderateur.com/a2ui-google-permet-agents-ia-composer-interfaces-utilisateur/ (launch date)
- https://github.com/kokoro-ele/a2ui-ink ; https://crates.io/crates/a2ui ; https://github.com/Liangdi/a2ui
- npm downloads: https://api.npmjs.org/downloads/point/last-month/@a2ui/react

Not verified: the exact launch date (from secondary coverage; we didn't find Google's own launch
post), and how much production traffic Opal and Gemini Enterprise put through A2UI (the claims are
the project's own).
