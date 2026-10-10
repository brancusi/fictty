# json-render and its Ink renderer

- **Maker:** Vercel Labs
- **URL:** https://github.com/vercel-labs/json-render, https://json-render.dev,
  `@json-render/ink` on npm
- **Status (2026-10-10):** about 18,600 stars, Apache-2.0, 231 commits on main, last commit
  1 October 2026. `@json-render/ink` was added in 0.15.0 (first published 23 March 2026); latest
  0.21.0 (18 September 2026). npm reports about 591,000 downloads of `@json-render/ink` and 8.2
  million of `@json-render/core` in the month to 8 October 2026. We couldn't tell how much of the
  Ink figure is real terminal use rather than installs pulled in by tooling or CI. The format itself is covered
  in [data--json-render.md](data--json-render.md); this dossier is about the terminal renderer.

## What it is

A "generative UI framework": you define a catalogue of components and actions (with Zod
schemas), the model writes a flat JSON spec that may only use that catalogue, and a renderer draws
it. There are about 30 packages: React, Vue, Svelte, Solid, React Native, PDF, email, image,
video (Remotion), 3D, Next.js, MCP Apps, state adapters, devtools and **Ink for the terminal**.

## The problem it's solving

Letting a model produce UI without letting it produce arbitrary code: "AI generates JSON, you
render it safely." The catalogue is the guardrail, and it also generates the system prompt.

## Its path / bet

One format, every renderer. The spec is the portable thing; the terminal is one more target
alongside web, mobile, PDF and video. It lives inside the developer's app: the app calls the
model, streams the spec, and renders it in-process.

## How it works

- **Spec:** `{ "root": "id", "elements": { "id": { "type", "props", "children": [ids] } } }`,
  a flat map keyed by id, so elements can be addressed and patched individually.
- **Streaming:** the model emits RFC 6902 JSON Patch lines (`{"op":"add","path":"/elements/card-1",
  ...}`); `createSpecStreamCompiler` turns chunks into patches and a growing spec.
- **State and logic in the spec:** `$state` reads, `$bindState` two-way binds, `$cond`,
  `$template`, `$computed`, `visible`, `repeat` and `watch` (run an action when a value changes).
  This is a small expression language.
- **Ink renderer:** about 25 standard components: Box, Text, Heading, Divider, Badge, Spinner,
  ProgressBar, Sparkline, BarChart, Table, List, Card, KeyValue, Link, StatusLine, Markdown,
  TextInput, Select, MultiSelect, ConfirmInput, Tabs. `useUIStream` streams a spec from an HTTP
  endpoint. The `ink-chat` example is a 644-line terminal chat app that calls tools, then renders
  the model's spec inline in the conversation, with long hand-written design rules in the prompt.
- There is no CLI and no standalone process: every package is a library you embed.

## Strengths

- Real adoption and a large team behind it; the format is becoming a default for "LLM writes UI
  as JSON".
- The flat, id-keyed spec plus JSON Patch is exactly the right shape for incremental edits.
- Catalogue-generated prompts and schema validation are a mature answer to "how does the model
  know what it may use".
- The Ink renderer already covers forms, tables, charts and markdown in a terminal.

## Weaknesses / limits

- It is a library inside someone's app, not a screen an outside agent can attach to. A coding
  agent in a terminal can't push a spec to a running json-render screen, patch it, send keys or
  read the frame back without the host app building all of that.
- Data comes from the host app's state store or from the model. Nothing binds a node to a
  command or a stream on its own.
- The expression language (`$cond`, `$computed`) is logic in the UI value, which fictty rules out
  on purpose.
- Ink is React in Node: fine for chat-style output, not built for thousands of updates a second.

## Relation to fictty

Complement and partial competitor. json-render owns the format question for web-first teams and
has a terminal renderer, so "a JSON UI in the terminal" is not fictty's differentiator. What
json-render doesn't have is the long-running, attachable process: push, patch by id, drive, read
back, data sources that skip the model.

## Could fictty adopt it instead of building?

Not as the runtime: it's a TypeScript library with no process, socket or read-back, and Ink is not
what fictty's speed claims rest on. fictty could adopt it as an **input format**: a json-render
spec with the Ink catalogue maps closely onto fictty's components, and its JSON Patch stream maps
onto `patch`. That is cheaper than inventing a second dialect, and it lets json-render users point
the same spec at a terminal screen an agent can drive.

## What fictty should take from it

- Accept a json-render spec (Ink catalogue) as input next to A2UI, and adopt RFC 6902 JSON Patch
  as the patch dialect (already on the plan).
- Generate the agent's instructions from the catalogue, as `catalog.prompt()` does, instead of
  hand-writing the skill.
- The `ink-chat` prompt is a free lesson in terminal design rules that work for models (one
  representation per number, explicit column widths, colour with restraint). Steal it for
  fictty's skill.
- Don't copy the expression language.

## Sources

- https://github.com/vercel-labs/json-render (README, `packages/ink/README.md`,
  `packages/ink/CHANGELOG.md`, `packages/core/README.md`, `examples/ink-chat/src/app.tsx`, read
  from a clone on 2026-10-10)
- https://json-render.dev/docs/api/ink
- https://registry.npmjs.org/@json-render/ink and https://api.npmjs.org/downloads/point/last-month/@json-render/ink
