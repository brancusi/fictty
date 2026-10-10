# json-render

- **Maker:** Vercel Labs (Vercel's experimental arm)
- **URL:** https://github.com/vercel-labs/json-render, https://json-render.dev
- **Status (10 October 2026):** Apache-2.0, 18,598 stars. Repository created 14 January 2026;
  v0.21.0 released 18 September 2026 (v0.16 to v0.21 between April and September). Still 0.x and
  still branded a Labs product. npm downloads in the month to 8 October 2026: `@json-render/core`
  about 8.2 million, `@json-render/ink` about 591,000. Download counts include CI and transitive
  installs, so treat them as direction, not users.

## What it is

A "Generative UI framework": you define a **catalog** of components and actions with Zod schemas,
the model generates a JSON **spec** constrained to that catalog, and a renderer draws it. One core
and many renderers: React, Vue, Svelte, Solid, React Native, shadcn/ui (36 components), Next.js and
TanStack Start (whole apps with routes), Remotion (video), React PDF, React Email, Satori images,
React Three Fiber (3D), MCP Apps, and **Ink for the terminal**.

## The problem it's solving

Letting a model produce UI reliably: output that always validates, streams as it arrives, and can
be pointed at whatever surface a product already has, without the model writing React.

## Its path / bet

Be the **schema-agnostic toolkit**, not a standard. The core doesn't impose a format (its docs show
A2UI modelled as just another catalog). Win on breadth of renderers, developer ergonomics and the
Vercel/AI SDK ecosystem; add devtools, state-library adapters and codegen so a generated spec can
graduate into real code.

## How it works

- **Spec.** A flat map: `{ "root": "card-1", "elements": { "card-1": { "type": "Card", "props":
  {...}, "children": ["button-1"] } } }`.
- **Catalog to prompt.** `catalog.prompt()` turns component and action descriptions into a system
  prompt.
- **SpecStream.** The model emits JSONL lines of RFC 6902 JSON Patch (`{"op":"add","path":
  "/elements/btn-1","value":{...}}`); `createSpecStreamCompiler` applies them as they arrive. Edit
  modes for later turns: `patch` (6902), `merge` (RFC 7396) and `diff`. A YAML wire format exists
  too.
- **State and logic in the spec.** A state model with `$state` reads, `$bindState` two-way binding,
  `visible` conditions, `$cond`/`$then`/`$else`, `$template` interpolation, `$computed` (registered
  functions), a `setState` action, and `watch` (fire an action when a state path changes).
  `@json-render/directives` adds `$format`, `$math`, `$concat`, `$count` and more.
- **Ink renderer.** About 25 terminal components: Box, Text, Heading, Divider, Badge, Spinner,
  ProgressBar, Sparkline, BarChart, Table, List, Card, KeyValue, StatusLine, Markdown, TextInput,
  Select, MultiSelect, ConfirmInput, Tabs. It runs inside the developer's own Ink (React) program.
- **Devtools.** A panel with the spec tree, a state editor, action and stream logs, and a picker.

## Strengths

- The most renderers in the camp, and the only one with a maintained, widely-installed terminal
  renderer.
- RFC 6902 JSON Patch as the streaming and editing dialect: standard, diffable, and easy for a
  model to emit.
- Pragmatic: it treats formats as interchangeable and A2UI as one of them.
- Strong distribution through Vercel and the AI SDK.

## Weaknesses / limits

- **It's a library, not a process.** The renderer lives inside an app the developer writes. There
  is no running screen an outside agent can attach to, patch, drive or read back.
- **Logic in the data.** `$cond`, `visible` rules, `$computed`, `watch` and directives make the spec
  a small reactive program. Powerful, but it's the slope toward code the "data" camp said it was
  avoiding.
- **Data passes through the model or the host.** State is set by the spec, by actions, or by the
  host app; there's no declarative binding to a command or stream.
- Labs status and 0.x versions: breaking changes are normal. Its own A2UI example targets the old
  v0.8 message names.
- Token cost: OpenUI's benchmark puts json-render at about twice OpenUI Lang (a competitor's
  number).

## Relation to fictty

**Compete (in the terminal) and inspire.** `@json-render/ink` is the nearest neighbour in fictty's
own cell: agent writes data, a terminal draws it. The difference is the loop: json-render renders a
spec inside someone's Ink app; fictty is the long-running screen the agent pushes to, patches,
drives with keys and reads back, with data bound to commands. A team that only needs "model emits a
TUI form once" will pick json-render/ink and never need fictty.

## Could fictty adopt it instead of building?

No, not as the runtime. It's TypeScript on Ink/React; fictty's speed claims (thousands of patches a
second, sub-millisecond frames) rest on ratatui's cell diff in one Rust process, and its read-back,
local behaviour and data sources have no equivalent in json-render. Adopting its **spec format** is
possible but unattractive: it carries reactive logic fictty's rules forbid, and it's a moving 0.x
target. Accepting a json-render spec (the logic-free subset) as an input dialect is cheap and worth
considering after A2UI.

## What fictty should take from it

- **RFC 6902 JSON Patch** as the patch dialect (already on the plan), plus RFC 7396 merge for
  coarse edits. The same dialect as json-render and AG-UI means agents already know it.
- **Catalog to prompt.** Generate the agent-facing description of fictty's components from the same
  schema the runtime validates against, so docs and validation can't drift.
- **Devtools as a feature:** fictty's `screen` and `get --state` are the read-back; a stream log and
  action log (planned history) would match what json-render's devtools show humans.
- **A clear line on logic.** json-render shows where "just a little conditional" leads. fictty's
  "substitution only" rule is a real differentiator worth stating in the docs.

## Sources

- https://github.com/vercel-labs/json-render (README, read 10 October 2026)
- https://github.com/vercel-labs/json-render/tree/main/packages/ink
- https://github.com/vercel-labs/json-render/tree/main/packages/core
- https://json-render.dev/docs/a2ui
- https://github.com/thesysdev/openui#token-efficiency-benchmarks (competitor benchmark)
- https://api.npmjs.org/downloads/point/last-month/@json-render/core ;
  https://api.npmjs.org/downloads/point/last-month/@json-render/ink

Not verified: how many real projects use the Ink renderer (download counts are inflated by
transitive installs), and whether Vercel intends to move json-render out of Labs.
