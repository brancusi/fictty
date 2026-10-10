# Tambo

- **Maker:** Tambo AI (startup)
- **URL:** https://github.com/tambo-ai/tambo, https://tambo.co
- **Status (10 October 2026):** MIT, 11,186 stars, repository created June 2024, pushed to daily
  (web-v0.136.0 on 9 October 2026). **Tambo Cloud is shutting down:** signups are closed, the
  hosted API stops on 31 October 2026 and user data is deleted on 30 November 2026. The open-source
  toolkit stays self-hostable; the team has moved on to a new product (Charming). Added to this
  cluster because it was one of the larger "generative UI SDK" projects and its shutdown is a
  signal about the camp.

## What it is

A React toolkit plus backend for agents that render the app's own components. Developers register
components with Zod prop schemas; the schemas become tool definitions; the agent picks a component
and streams its props.

## The problem it's solving

Making an existing React app's components available to an agent, with streaming, cancellation and
conversation state handled for you.

## Its path / bet

Full-stack convenience: the SDK plus a hosted agent loop and state store. Two kinds of component:
**generative** (rendered once in reply to a message) and **interactable** (persist, and the agent
can keep updating their props as the person refines the request, like a task board or a cart).

## How it works

- `TamboProvider` with a list of `{name, description, component, propsSchema}`.
- `withInteractable(Component, {...})` wraps a component so the agent can read and update its props
  over the conversation.
- Backend (Tambo Cloud or self-hosted Docker) runs the LLM loop with the developer's API key; MCP
  servers can be attached.

## Strengths

- **Interactable components** are a real idea: a component on screen the agent keeps editing,
  rather than a new card per message. That's closer to fictty's patch-by-id than anything else in
  the React camp.
- Easy start; good developer experience by most accounts.

## Weaknesses / limits

- The hosted business didn't last. A generative UI SDK with a hosted agent loop was squeezed
  between the AI SDK, CopilotKit and the labs' own offerings.
- React only, developer-built components only; no generative layout.
- Agent read-back is limited to the props the agent itself set.

## Relation to fictty

**Watch.** Not a competitor on surface or approach. Its interactable components validate the
"agent keeps editing a live screen" loop, and its shutdown is a reminder that a thin layer over
everyone else's pieces is hard to sustain.

## Could fictty adopt it instead of building?

No: React, browser, developer-built components, and a backend in wind-down.

## What fictty should take from it

- **Name the "interactable" idea** in fictty's docs: every fictty node is interactable by default
  (patch by id, read back), which is the thing Tambo had to add as a wrapper.
- **Don't build a hosted agent loop.** Stay the runtime the agent already running in the terminal
  talks to; the agent loop belongs to Claude Code, Codex and the rest.

## Sources

- https://github.com/tambo-ai/tambo (README, shutdown notice, read 10 October 2026)
- https://tambo.co/blog/posts/tambo-is-shutting-down (linked from the README; not read directly)
- https://docs.tambo.co/concepts/generative-interfaces/interactable-components

Not verified: the reasons for the Cloud shutdown beyond the README notice.
