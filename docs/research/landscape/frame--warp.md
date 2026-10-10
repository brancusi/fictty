# Warp

- **Maker:** Warp (venture-funded company; CEO Zach Lloyd)
- **URL:** https://www.warp.dev, https://github.com/warpdotdev/warp
- **Status (10 October 2026):** client open source since **28 April 2026** (blog post by Zach
  Lloyd; changelog entry 27 April). AGPL-3.0 for the client, MIT for its UI framework crates
  (`warpui_core`, `warpui`); the server stays closed. 65,408 stars on the repository (most of
  which predate open-sourcing, when the repo held issues only). OpenAI is the "founding sponsor"
  of the open-source repository and its agent workflows run on GPT models. Latest tagged stable
  in the repo: 3 June 2026; the app itself auto-updates on a faster channel.

## What it is

"An agentic development environment, born out of the terminal": a Rust, GPU-rendered terminal
app with its own built-in coding agent, support for third-party CLI agents (Claude Code, Codex,
Gemini CLI and about a dozen others), code review, Warp Drive for shared workflows, and Oz, a
cloud platform for orchestrating agents ("Warp Factories").

## The problem it's solving

The classic terminal is a stream of bytes with no structure and no help. Warp groups each command
and its output into a **block**, adds a real input editor, and now treats the terminal as the
control room for agents, local and in the cloud.

## Its path / bet

Own the terminal app and turn it into the agent workspace, then sell the cloud orchestration
(Oz, Factories). Open-sourcing the client is a bet that community plus agents can out-ship larger
closed rivals; Warp says plainly it can't compete on price.

## How it works

- **Blocks.** Commands and output are grouped as units you can select, copy, share and give to
  the agent as context.
- **Third-party CLI agents.** Warp detects supported agents and adds a rich input editor, agent
  notifications (for Claude Code, Codex and OpenCode; others not yet) and inline code review.
- **Oz.** Cloud agents with skills and verification loops; Claude Code and Codex are available as
  harnesses there. Warp's own repository is developed this way (build.warp.dev shows the agents).

## Strengths

- A large installed base and a real company.
- The block model is the closest any mainstream terminal got to structured, addressable output.
- Open client and MIT UI framework: the code is now readable.

## Weaknesses / limits

- A desktop app with its own renderer; doesn't run over SSH as a TUI does.
- AI features depend on Warp's closed backend; building from source isn't a self-hosted setup.
- We found no way for an agent to put its own structured UI in a Warp pane beyond terminal output
  and Warp's own panels.
- We didn't verify Warp's current pricing or usage numbers.

## Relation to fictty

**Watch.** Warp is a frame with ambitions to be the whole environment. A fictty screen runs in a
Warp pane like any TUI (Warp supports the kitty graphics protocol per our earlier pass). Warp's
blocks are a reminder that structure in the terminal is valuable and that a terminal *app* can
impose it; fictty imposes it from inside the pane instead, which works in every terminal.

## Could fictty adopt it instead of building?

No. AGPL client of a commercial product, desktop-only, no content model for agents.

## What fictty should take from it

- **Blocks as the unit of read-back.** Warp's insight is that "command plus its output" is the
  unit people and agents want to address. fictty's equivalent is the node id; keep every
  read-back addressable by id.
- **Notifications through the host.** Warp shows CLI-agent notifications from standard
  sequences; fictty should emit OSC 9/777-style notifications as well as OSC 7501 so hosts like
  Warp and cmux can surface "waiting for you".
- **warpui** (MIT) is worth a look as a Rust UI framework if fictty ever needs a native window,
  not before.

## Sources

- https://github.com/warpdotdev/warp (README, licences, releases), read 10 October 2026
- https://www.warp.dev/blog/warp-is-now-open-source (28 April 2026)
- https://www.helpnetsecurity.com/2026/04/30/warp-open-source-client/
- https://blog.kilo.ai/p/warp-finally-went-open-source (critique: backend dependence)
- https://openai.com/index/warp (OpenAI customer story)
- https://docs.warp.dev/agent-platform/cli-agents/overview/
