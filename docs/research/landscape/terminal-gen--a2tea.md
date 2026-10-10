# a2tea (A2UI to Bubble Tea bridge)

- **Maker:** Joe Stump (`joestump-agent`); mirrored into the `charmbracelet` GitHub organisation
  as a read-only copy of `gitea.stump.rocks/stump.wtf/a2tea`
- **URL:** https://github.com/charmbracelet/a2tea, docs at https://joestump-agent.github.io/a2tea/
- **Status (2026-10-10):** 4 stars, Apache-2.0 per the README (GitHub shows no detected licence),
  created 15 August 2026, pushed 10 October 2026. Pre-1.0, no tagged release yet. Its stated
  consumer is "Joe's fork of charmbracelet/crush". We couldn't confirm whether Charm itself plans
  to ship it in Crush; the mirror in Charm's organisation is the only signal.

## What it is

A Go library that finds A2UI messages inside a model's reply (interleaved with prose), and renders
each surface as an embeddable Bubble Tea model inside a chat host.

## The problem it's solving

A terminal coding agent that receives structured UI from a model today prints raw JSON. a2tea lets
the chat host draw it instead, inline in the conversation, with input that round-trips to the
agent.

## Its path / bet

Generative UI belongs inline in the agent's own chat transcript, rendered by the agent's own TUI,
using the standard (A2UI v0.9 via `github.com/tmc/a2ui`) rather than a custom format.

## How it works

- `a2tea.Scan(reply)` splits a reply into text parts and A2UI message parts; `Render(msgs)`
  returns a `tea.Model` with `SetSize`, `Focus`, `Blur`; it never quits the program.
- Core catalogue rendered: Text variants, Card, Row, Column, List, Divider, Button, all five
  inputs, Tabs, Modal; media as one-line placeholders. Monochrome chrome so the host theme wins.
- Tab cycles focus; Enter on a button sends a protocol-native `a2ui.ClientMessage` with context
  back to the agent.

## Strengths

- Right integration point for chat-centric agents: inside the transcript, no second pane.
- Standards-first: no private component types.
- Designed to embed, which is what agent TUIs need.

## Weaknesses / limits

- Very early and effectively one person's project.
- Inline surfaces in a scrolling transcript are short-lived by design; no long-running screen, no
  data binding beyond what the agent sends, no read-back by an outside agent.
- Go only; usable by Bubble Tea hosts.

## Relation to fictty

Watch. It shows the other place generative terminal UI can live: inside the agent's chat, not
beside it. If Crush or another big agent ships inline A2UI, the "small form or picker" use case
moves into the agent itself and fictty's room narrows to screens that outlive a message: live
dashboards, review queues, long walkthroughs.

## Could fictty adopt it instead of building?

No. Different language, different job (inline renderer in someone else's TUI), and it lacks every
part of fictty's loop.

## What fictty should take from it

- The inline case is real. fictty should be able to render a value once to stdout (it has
  `render`), so an agent CLI can show a static snapshot inline and open a live screen only when
  it's needed.
- Monochrome-by-default chrome that defers to the host theme is a good rule for anything embedded.

## Sources

- https://github.com/charmbracelet/a2tea (README)
- https://api.github.com/repos/charmbracelet/a2tea
- https://pkg.go.dev/github.com/tmc/a2ui
