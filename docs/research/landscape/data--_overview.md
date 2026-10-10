# The data camp: agents fill in declarative UI

Research as of 10 October 2026. Dossiers: [A2UI](data--a2ui.md),
[json-render](data--json-render.md), [OpenUI / Thesys C1](data--openui.md),
[AG-UI and CopilotKit](data--ag-ui-copilotkit.md),
[AI SDK generative UI](data--ai-sdk-generative-ui.md),
[ChatKit widgets and Open-JSON-UI](data--chatkit-widgets.md),
[Adaptive Cards](data--adaptive-cards.md), and [Tambo](data--tambo.md) (added: a large generative UI
SDK whose hosted service is shutting down). Nothing on the original list turned out not to exist;
"Open-JSON-UI" exists only as CopilotKit's name for OpenAI's widget schema, so it's covered inside
the ChatKit dossier.

## What's really being solved

Everyone in this camp is solving the same problem: **how to let a model put interactive UI in front
of a person without running code the model wrote.** The answer is always some version of
server-driven UI: the host keeps a catalogue of trusted components and draws them its own way; the
model sends a description. Adaptive Cards did this in 2017 for bots. A2UI, json-render and the rest
are doing it again for agents, with three new pressures:

1. **Streaming.** A person shouldn't wait for the whole description; formats are flat lists with
   ids or JSON Patch lines so a half-finished UI can draw.
2. **Tokens.** Every brace costs latency. OpenUI built a terse language; A2UI followed with its
   Express proposal.
3. **Editing over turns.** A UI is revised as the conversation goes on, so everything is
   addressable by id (A2UI components, json-render elements, OpenUI statement names, Tambo's
   interactable components).

Almost all of it lives in **a chat, in a browser**, and most of it treats the UI as **a message**:
a card in a thread that a later message replaces. Long-lived screens are the exception.

## The paths

| path | who | the bet |
|---|---|---|
| A portable format, host-native rendering | A2UI, Adaptive Cards | be the contract under every client; the host owns the look |
| A schema-agnostic toolkit with many renderers | json-render | be the library everyone renders with, whatever the format |
| A compact language with logic and data | OpenUI (Thesys) | tokens and power matter more than "pure data"; sell the gateway |
| The transport and shared state | AG-UI, CopilotKit | stay neutral on format, own the pipe and the frontend stack |
| Model picks, developer renders | AI SDK, ChatKit, Tambo | the model never lays out UI; it calls typed tools or fills templates |

## The open debates

- **How much logic belongs in the data?** The spectrum runs from Adaptive Cards and A2UI (small
  functions for formatting, validation and `and`/`or`/`not`) through json-render (`$cond`,
  `visible`, `watch`, `$computed`) to OpenUI (variables, ternaries, loops, filters, queries). Every
  format that started as "safe like data" has added logic. fictty's rule (substitution only, logic
  in the runtime or in commands) is at the strict end, and that's a position to argue, not a
  default.
- **JSON or a language?** OpenUI's benchmark claims about half the tokens of JSON formats;
  A2UI's Express proposal concedes the point. JSON stays the wire format; what the model types may
  not be JSON for long.
- **Catalogue sameness.** Critics say fixed catalogues make every generated UI look alike; the
  defenders say that's the point (the host's design system).
- **Standard or toolkit?** A2UI wants to be the standard; json-render treats standards as
  catalogues; OpenUI calls its language a standard; CopilotKit supports all of them. Consolidation
  has started: Thesys renamed C1 to OpenUI Cloud, OpenAI is retiring Agent Builder, Tambo Cloud is
  shutting down, the AI SDK walked `streamUI` back to experimental.

## What almost nobody does (and fictty does)

- **Read back what was drawn.** These formats are one-way: the agent knows what it sent and what
  actions came back, not what the person sees, where the cursor is, or what is selected. ChatKit's
  `copy_text` and A2UI's `sendDataModel` are the closest. A frame as text plus the full state as
  JSON is unclaimed.
- **Data that skips the model.** Values usually arrive through the agent (A2UI
  `updateDataModel`, AG-UI state deltas). OpenUI's `Query(..., refreshSeconds)` is the single
  exception and the nearest cousin to fictty's data sources.
- **A long-running screen an outside agent attaches to.** All of these are libraries inside
  someone's app. None is a process an agent in a terminal can push to, patch, drive with keys and
  read back.
- **The terminal.** Only json-render's Ink renderer is real there (about 590,000 npm downloads a
  month, though downloads overstate users); the A2UI terminal renderers (`a2ui-ink`, the Rust
  `a2ui` crate) are tiny.

## Should fictty adopt one of these as its format?

Hard-nosed answer: **no single format replaces fictty's value, and A2UI is the only one worth
speaking natively.**

- **A2UI** maps best: surfaces, components by id, a data model by JSON Pointer, custom catalogues.
  It lacks data sources, key maps, layers, pixels, theme (removed in v1.0) and any read-back, and
  v1.0 is still a release candidate. Keep fictty's value internal, write the A2UI v1.0 adapter onto
  assembly as planned, and align vocabulary so the mapping is lossless. If A2UI goes stable and
  grows an extension for bound data sources, re-run this decision.
- **json-render**: adopt its patch dialect (RFC 6902, plus RFC 7396 merge), not its spec; the spec
  carries logic fictty forbids.
- **OpenUI Lang**: incompatible with "data, never code." Take its data-source details (defaults,
  re-run on input change) and its token discipline.
- **AG-UI**: a possible second transport, not a format. Emitting state as snapshot plus JSON Patch
  deltas would let AG-UI tooling consume fictty unchanged.
- **AI SDK, ChatKit, Tambo, Adaptive Cards**: nothing to adopt; ideas only (controlled pattern,
  plain-text twin per node, host config, fallback, versioning).

## What this means for fictty's position

The data camp has settled the format question well enough that fictty shouldn't compete on it.
What's left open is everything after the first render: **a screen that stays up, runs its own
behaviour at frame rate, pulls its own data, and can be read back exactly.** That's fictty's
ground. The risk is json-render/ink (or an A2UI terminal renderer) growing a socket and a
read-back verb; that would be a weekend's work for a motivated maintainer, so fictty's lead is in
the loop's quality and speed, not in the idea.

For the bigger question (UI that's cheap enough to throw away, and visual learning rather than
reading an agent's text), this camp is mostly silent. Its products are chat cards and enterprise
forms. Nobody here is designing for "an agent builds a screen to help a person understand
something, and then the screen goes away." That framing is open.
