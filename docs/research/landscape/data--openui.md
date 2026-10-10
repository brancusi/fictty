# OpenUI (and Thesys C1, now OpenUI Cloud)

- **Maker:** Thesys (thesys.dev), a venture-backed startup
- **URL:** https://github.com/thesysdev/openui, https://openui.com, https://www.thesys.dev
- **Status (10 October 2026):** MIT, 10,645 stars. The repository dates from December 2024 (it
  began as Thesys's earlier open-source work and was relaunched as OpenUI; Product Hunt lists the
  OpenUI launch on 11 March 2026). `@openuidev/cli` 0.5.1 released 8 October 2026; pushed to
  daily. `@openuidev/react-lang` had about 675,000 npm downloads in the month to 8 October 2026.
  Thesys's commercial **C1 API** (launched 2025; directories give April or September) is now sold
  as **OpenUI Cloud**, "formerly the C1 API", alongside OpenUI Gateway (generative UI across
  models, with validation and repair). Pricing we found is from aggregators only (a free tier,
  then about $49 and $499 a month plus model tokens); unverified.

## What it is

A generative UI framework built around **OpenUI Lang**, a compact line-oriented language a model
writes instead of JSON. A framework-agnostic core parses it as it streams; runtimes for React, Vue,
Svelte and Angular render it; React gets prebuilt component libraries and a full chat UI. There's
also an A2UI v1.0 client that carries OpenUI Lang component payloads, a LangChain/AG-UI bridge, an
email library and a browser bundle.

## The problem it's solving

Latency and cost. A model generating JSON UI spends most of its tokens on braces, quotes and key
names, and a person waits for them. OpenUI's claim is that a terse language halves the tokens and
therefore the wait, while staying constrained to a component library.

## Its path / bet

Make the **language** the standard and sell the infrastructure around it (Cloud, Gateway,
observability). Unlike A2UI and json-render, OpenUI accepts that a UI needs state, data fetching
and logic, and puts them in the language.

## How it works

- **Statements.** Each line assigns a component or value to a name: `root = Stack([header,
  tbl])`, `header = CardHeader("Tickets")`. References can be forward, so the streaming parser
  renders what it has.
- **Reactive state.** `$days = "7"` declares a variable; passing it to an input two-way binds it;
  any expression that reads it re-evaluates.
- **Queries and mutations.** `data = Query("list_tickets", {}, {rows: []}, 30)` calls a tool the
  host exposes, renders the default until it returns, and **re-fetches every 30 seconds**;
  `Mutation(...)` runs only from an action. The runtime executes these, not the model.
- **Expressions and built-ins.** Ternaries (`$showEdit ? editForm : null`), string concatenation,
  `@Filter`, `@Sort`, `@Count`, `@Sum`, `@Each` (iteration), math, and action steps (`@Run`,
  `@Set`, `@Reset`, `@ToAssistant`, `@OpenUrl`).
- **Incremental edits.** `mergeStatements(original, patch)` merges a later turn's statements by
  name, so a model edits by re-stating only the lines that change.
- **Benchmark.** Seven scenarios measured with tiktoken: OpenUI Lang 4,800 tokens against 10,180
  for json-render and 9,948 for C1 JSON (-52%). Run by Thesys on its own scenarios.

## Strengths

- The token argument is real and has been conceded by others (A2UI's Express proposal is the same
  idea in a different syntax).
- **Data that skips the model.** `Query` with a refresh interval is the only declarative,
  model-free data binding in this camp, the closest thing to fictty's data sources.
- Edits by name are cheap and natural for a model.
- Ships complete products (chat UI, Cloud, Gateway) as well as a spec.

## Weaknesses / limits

- **It's a programming language.** Variables, ternaries, iteration, filters and aggregation: the
  model now writes small reactive programs. That buys power and loses the "safe like data" story;
  validation and repair become a product (Gateway) because generated programs fail in more ways.
- **Read-back is the source text, not the result.** An agent can re-read what it wrote but not what
  rendered, what the person selected, or what a query returned, except through actions.
- Benchmarks are self-reported, scenario choice is Thesys's, and the comparison table in the README
  (e.g. "A2UI: custom components only") is a competitor's framing.
- A single company's language; the "open standard" has no outside governance we could find.
- Browser-only renderers; nothing for the terminal.

## Relation to fictty

**Inspire.** Different surface, different philosophy on logic, but OpenUI is the one entry here
that shares fictty's view that live data should be bound by the runtime, not relayed by the model.
It's also the clearest evidence that terse authoring syntax matters to agents.

## Could fictty adopt it instead of building?

No. Adopting OpenUI Lang as fictty's format would break fictty's first rule (a pushed UI is data:
no expressions, loops or conditionals), there is no terminal runtime to adopt, and the parser and
runtime are TypeScript. An OpenUI Lang input adapter is technically possible for a logic-free
subset, but the subset would drop exactly what makes OpenUI distinctive.

## What fictty should take from it

- **`Query(tool, args, default, refreshSeconds)` is a near-twin of fictty's data source**
  (`{"cmd": [...], "every": 3}`). Copy two details: a **default value** that renders before the
  first result, and **re-running when a bound input changes** (fictty's `@tasks` selection binding
  is the same idea; extend it to source arguments).
- **Edit by restating names.** fictty's patch-by-id is the same move; make sure a multi-node patch
  in one call is as cheap as OpenUI's merge.
- **Measure tokens.** If fictty claims agents write it well, publish a token count for its examples
  against A2UI and json-render on the same screens.
- **Draw the line on logic out loud.** OpenUI shows the alternative path; fictty's stance (logic in
  the runtime or in commands, never in the UI value) should be argued, not assumed.

## Sources

- https://github.com/thesysdev/openui (README, read 10 October 2026)
- https://github.com/thesysdev/openui/tree/main/packages/lang-core
- https://github.com/thesysdev/openui/blob/main/docs/content/docs/openui-lang/queries-mutations.mdx
- https://github.com/thesysdev/openui/blob/main/docs/content/docs/openui-lang/reactive-state.mdx
- https://github.com/thesysdev/openui/blob/main/docs/content/docs/openui-lang/builtins.mdx
- https://github.com/thesysdev/openui/tree/main/benchmarks
- https://www.thesys.dev/ ("OpenUI Cloud: formerly the C1 API")
- https://www.producthunt.com/products/thesys/launches/openui-2 ; https://toolradar.com/tools/thesys (pricing, unverified)

Not verified: C1's original launch date (sources disagree), current pricing, and Thesys's funding.
