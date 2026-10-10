# AG-UI and CopilotKit

- **Maker:** CopilotKit (company), which created and stewards the AG-UI protocol
- **URL:** https://github.com/ag-ui-protocol/ag-ui, https://docs.ag-ui.com,
  https://github.com/CopilotKit/CopilotKit
- **Status (10 October 2026):** AG-UI: MIT, 16,433 stars, repository created 7 May 2025, dated
  releases (latest `release/2026-10-10`); `@ag-ui/core` about 10.3 million npm downloads in the
  month to 8 October 2026. CopilotKit: MIT, 37,903 stars, created June 2023, v1.78.0 on
  9 October 2026; `@copilotkit/react-core` about 2.2 million downloads a month. CopilotKit sells a
  hosted layer ("CopilotKit Intelligence": persistent threads, memories, analytics).

## What it is

**AG-UI** is an event protocol between an agent backend and a user-facing app: a stream of about
16 standard event types (plus reasoning and sub-agent events) over SSE, WebSockets or anything
else. **CopilotKit** is the frontend stack on top: React (and Angular, Vue, React Native, Slack,
Teams) components and hooks for chat, frontend tools, shared state, human-in-the-loop and
generative UI.

## The problem it's solving

Every agent framework (LangGraph, CrewAI, ADK, Mastra, Pydantic AI, Strands, Microsoft Agent
Framework...) streams differently, and every app re-wires it. AG-UI is "the third protocol":
MCP gives agents tools, A2A connects agents to agents, AG-UI connects agents to people's apps.

## Its path / bet

Own the **transport and runtime layer** and stay neutral on the UI format. CopilotKit frames
generative UI as three patterns and supports all of them: **controlled** (the app pre-builds
components, the agent picks one and passes data, via AG-UI tool calls), **declarative** (A2UI and
Open-JSON-UI), and **open-ended** (MCP Apps, arbitrary HTML). Win by integrating with everyone:
the AG-UI README lists first-party support from Google ADK, Microsoft Agent Framework, AWS
Strands and Bedrock AgentCore, Mastra, LangGraph, CrewAI and more.

## How it works

- **Events.** Lifecycle (`RunStarted`, `RunFinished`, `RunError`, steps), text messages
  (start/content/end), tool calls (start/args/end/result), **state** (`StateSnapshot`,
  `StateDelta` as RFC 6902 JSON Patch, `MessagesSnapshot`), **activity** (`ActivitySnapshot`,
  `ActivityDelta`, structured in-progress updates between messages), reasoning, sub-agents, and
  `Custom`/`Raw` for extensions.
- **Shared state.** The agent and the app both read and write one state object, synchronised by
  snapshots and patches. This is AG-UI's real contribution to generative UI.
- **Frontend tools.** The app registers tools (`useFrontendTool`) the agent can call; the app
  renders each phase of the call (in progress, complete) with its own components.
- **A2UI on top.** CopilotKit's runtime renders A2UI when you pass `a2ui: {}`; A2UI names AG-UI as
  its standard transport binding.
- **Clients.** CopilotKit is the first-party client; the README also lists a community "Terminal +
  Agent" client and Slack/Teams channels.

## Strengths

- The widest adoption of any protocol in this camp, by a distance, and real cross-vendor buy-in.
- Shared state with JSON Patch deltas is a sound, general mechanism, and it flows both ways.
- Pragmatic taxonomy (controlled, declarative, open-ended) that matches how teams actually ship.

## Weaknesses / limits

- **It's plumbing, not a UI.** AG-UI says nothing about what a component is; generative UI is
  whatever runs on top. "Generative UI" doesn't appear in the event spec itself.
- **Chat-shaped.** Runs, messages and tool calls assume a conversation; a long-lived screen that
  outlives a run is not the model.
- The protocol is stewarded by one company whose business is the frontend and hosted layer above
  it; neutrality is a claim, not a governance structure.
- Rendering is React-first; the terminal client is community-maintained.

## Relation to fictty

**Complement.** AG-UI is a transport fictty could speak, not a competitor: a fictty screen could
consume an AG-UI stream (state snapshots and deltas into fictty's data; A2UI payloads into its UI
value) and emit actions back. The overlap is the shared-state idea, which fictty already has in a
stricter form.

## Could fictty adopt it instead of building?

Not as a substitute: there's no UI or runtime in AG-UI to adopt. As a **second wire protocol**
next to fictty's socket it is plausible and possibly valuable, since it would let any AG-UI agent
framework drive a fictty screen. The cost is a chat-and-run-shaped event model that doesn't match
fictty's push/patch/read loop; an adapter, not a rewrite.

## What fictty should take from it

- **State snapshots plus JSON Patch deltas** is the standard shape for syncing state with an agent.
  fictty's `get --state` and the planned `watch` could emit exactly that (a snapshot, then
  deltas), so AG-UI tooling can consume fictty without translation.
- **The three-pattern taxonomy** is a useful way to explain fictty: it is "declarative" in
  CopilotKit's terms, with the twist that behaviour and data run locally.
- **Render tool-call phases.** Showing "in progress" while an agent works is table stakes; fictty
  should have a cheap, standard way to show an agent's pending action on screen.

## Sources

- https://github.com/ag-ui-protocol/ag-ui (README, read 10 October 2026)
- https://docs.ag-ui.com/concepts/events
- https://github.com/CopilotKit/CopilotKit (README)
- https://github.com/CopilotKit/generative-ui (the three-pattern guide)
- https://docs.copilotkit.ai/generative-ui ; https://docs.copilotkit.ai/generative-ui-specs/open-json-ui
- https://api.npmjs.org/downloads/point/last-month/@ag-ui/core ;
  https://api.npmjs.org/downloads/point/last-month/@copilotkit/react-core

Not verified: the depth of the "1st party" integrations (some may be thin adapters), and the
status of the community terminal client.
