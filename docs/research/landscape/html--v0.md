# v0

- **Maker:** Vercel
- **URL:** https://v0.app ; API https://api.v0.dev/v2
- **Status (10 October 2026):** commercial product, closed source. Launched in 2023 as a UI
  generator; now an app-building agent that edits files and runs a dev server in a Vercel Sandbox.
  The v0 API (v2) became generally available on 5 August 2026 per a third-party write-up. Pricing
  is credit-based (Free with $5/month, Plus $30, Business $100 per user). Vercel's June 2026 Ship
  event added agent infrastructure around it.

## What it is

An agent that turns a prompt (or a repo, a ZIP, a screenshot) into a working web app, with a live
preview URL and one-click deploy to Vercel. Historically it generated React with Tailwind and
shadcn/ui; we couldn't confirm the current framework from the API announcement, though Vercel's
ecosystem makes Next.js the likely default.

## The problem it's solving

Building a real front end is slow. v0 makes the first version of an app nearly free and keeps it on
a path to production (Git, deploys, a real framework).

## Its path / bet

The durable end of the HTML camp: the model writes *real application code* that people keep,
deploy and maintain. It's in this cluster because it's the reference point for "the agent writes
the UI as code", not because it does throwaway screens.

## How it works

- `chats.create` / `createAsync` / `createStream` run the agent; the stream emits text, thinking,
  file edits, bash commands and tool calls.
- The agent works in an isolated sandbox, runs a dev server and checks its work against it.
- `createVercelProject` and `deploy` ship it. Preview tokens are short-lived and should be proxied.

## Strengths

- Production-grade output on a mainstream stack; the generated code is the asset.
- A headless API, so other agents and products can embed the generator.
- The agent verifies against a running app, which is a form of read-back.

## Weaknesses / limits

- Heavy for a ten-minute screen: a sandbox, a dev server, a framework, seconds to minutes per turn.
- Cloud and Vercel bound; no terminal surface; credits cost money.
- The agent writes code, so every UI is a project to maintain.

## Relation to fictty

**Watch.** Barely overlaps. It's useful as the far "durable" pole on the ephemerality axis: it shows
what the HTML camp looks like when the UI is meant to last, and therefore what fictty is not.

## Could fictty adopt it instead of building?

No. Different problem.

## What fictty should take from it

- **Stream the agent's work as typed parts** (text, file edits, commands). fictty's `watch` output
  and history should be typed events too, not free text.
- **Graduation path.** If a fictty screen turns out to be worth keeping, the honest next step is a
  real app; a "describe this screen as a spec for v0 or a ratatui app" export could be cheap and
  useful. Low priority.

## Sources

- [Vercel v0 API GA](https://www.developersdigest.tech/blog/vercel-v0-api-ga-2026) (third-party, August 2026)
- [Vercel unveils agent tools](https://itbrief.co.uk/story/vercel-unveils-agent-tools-as-deployments-turn-ai-led) (third-party, June 2026)
- [v0.app](https://v0.app)

Not verified: the GA date against Vercel's own changelog, and the current default framework.
