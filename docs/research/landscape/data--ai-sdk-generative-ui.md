# Vercel AI SDK generative UI

- **Maker:** Vercel
- **URL:** https://github.com/vercel/ai, https://ai-sdk.dev/docs/ai-sdk-ui/generative-user-interfaces
- **Status (10 October 2026):** Apache-2.0 (GitHub shows NOASSERTION; the LICENSE file is Apache
  2.0), 27,228 stars. AI SDK 5 shipped 31 July 2025, 6 on 22 December 2025, and 7 is the current
  `latest` (7.0.137, 9 October 2026), with v6 and v5 still patched. The `ai` package had about
  124 million npm downloads in the month to 8 October 2026, by far the most of anything in this
  camp. The older React Server Components path (`ai/rsc`, `streamUI`) is labelled "currently
  experimental. We recommend using AI SDK UI for production."

## What it is

The AI SDK's generative UI is a pattern, not a format: "connecting the results of a tool call to a
React component." The model calls a typed tool; the tool runs; the result arrives in the message's
`parts` as `tool-<name>` with a state (`input-available`, `output-available`, `output-error`); the
developer's React code renders the matching component.

## The problem it's solving

Getting structured, interactive output into a chat UI with full type safety, using the tool-calling
the model already does well, without inventing a UI language.

## Its path / bet

Keep the model on rails it's already good at (tool calls with schemas) and leave the UI entirely to
the developer's components. The earlier, bolder bet (the server streams React components with
`streamUI`) was walked back to "experimental"; the mainstream path is now "tool parts rendered by
the client." Vercel's more ambitious generative UI work moved to json-render (Labs).

## How it works

- Server: `streamText({ tools })`, streamed with `toUIMessageStream`.
- Client: `useChat`, map over `messages[].parts`, switch on `part.type === 'tool-displayWeather'`,
  render a loading state, the component with `part.output`, or an error.
- The model chooses *which* component (by calling that tool) and supplies its props (the tool
  arguments). It never composes layout.

## Strengths

- Enormous adoption and a stable, typed, well-documented API.
- The safest possible shape: the model never describes UI, only calls functions.
- Composes with everything else (json-render, MCP, AG-UI adapters).

## Weaknesses / limits

- **Not generative layout.** Every UI must be pre-built by a developer; the agent can't make a
  screen nobody anticipated. For throwaway screens, that's the whole problem.
- Chat-message-shaped: a component lives inside a message, not on a long-lived surface the agent
  can revise by id.
- Browser and React only. Nothing for the terminal.
- No read-back of rendered state beyond what the app chooses to send back.

## Relation to fictty

**Watch.** Different problem: typed tool results in a web chat. It matters to fictty only as the
default pattern most agent developers learn first, and as evidence that the mainstream retreated
from streaming components to "model picks, app renders."

## Could fictty adopt it instead of building?

No. There is no format or runtime to adopt; it's a TypeScript SDK pattern for React apps.

## What fictty should take from it

- **The controlled pattern is the baseline.** fictty's project components (JSON compositions with
  named parameters) are the terminal equivalent of "register a component, let the model pass
  props." Make that path at least as easy as writing a full UI value, because most agents will use
  it most of the time.
- **Typed phases of a tool call** (pending, done, error) are a good model for how a fictty node
  bound to a running command should look.
- **The `streamUI` retreat is a warning.** Streaming arbitrary UI from a server into a client
  framework was harder to make reliable than expected; fictty's "data, never code" stance avoids
  the same trap.

## Sources

- https://github.com/vercel/ai
- https://ai-sdk.dev/docs/ai-sdk-ui/generative-user-interfaces
- https://ai-sdk.dev/docs/ai-sdk-rsc/overview
- https://registry.npmjs.org/ai (release dates) ; https://api.npmjs.org/downloads/point/last-month/ai

Not verified: whether AI SDK 7 changed the generative UI guidance beyond the AI SDK 5 tool-part
naming the docs page cites.
