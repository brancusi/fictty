# Adaptive Cards

- **Maker:** Microsoft
- **URL:** https://adaptivecards.io, https://github.com/microsoft/AdaptiveCards
- **Status (10 October 2026):** MIT, 1,965 stars, repository created December 2016. Schema
  versions 1.0 to 1.6; 1.6 is the latest. The JavaScript renderer `adaptivecards` is at 3.0.6
  (April 2026 tag) with about 4.2 million npm downloads in the month to 8 October 2026. Recent
  commits are dependency bumps and fixes (latest 6 October 2026); the README's release table was
  last updated in 2024. Effectively in **maintenance mode** as an open project, while remaining
  the card format of Teams, Outlook, Copilot Studio and Microsoft 365 Copilot (Teams renders up to
  1.5).

## What it is

A JSON format for platform-agnostic UI snippets ("cards"): text blocks, images, fact sets,
containers, column sets, inputs and actions. The **host** decides the look through a host config;
the card only says what goes where. Renderers exist for JavaScript, .NET (WPF, HTML), UWP/WinUI,
Android and iOS, plus a visual designer and a separate **templating** language that binds a card
template to a data object.

## The problem it's solving

In 2017: letting bots and services send interactive content into many hosts (Teams, Outlook,
Windows, Bot Framework channels) that each keep their own visual style, without sending HTML.
Today it's the default answer inside Microsoft's agent products, and models are routinely asked to
write card JSON.

## Its path / bet

The original "safe like data" bet, eight years before A2UI: a declarative payload, a host-owned
look, renderers per platform, graceful degradation (`fallback`, versioning) when a host is older
than the card.

## How it works

- A card: `{"type": "AdaptiveCard", "version": "1.5", "body": [...], "actions": [...]}` with
  elements such as `TextBlock`, `Image`, `ColumnSet`, `FactSet`, `Input.Text`,
  `Action.Submit`, `Action.Execute` (with server-side refresh).
- **Host config** maps semantic styles (`"emphasis"`, `"good"`, `"large"`) to the host's fonts,
  colours and spacing.
- **Templating** (`${$root.name}`, `$data` for repetition, `$when` for conditions) separates
  layout from data; the template is expanded before rendering.
- **Fallback and versioning:** an element can name a fallback for hosts that don't support it.

## Strengths

- Proven at very large scale, with a decade of hosts and renderers, and native renderers on every
  major platform.
- The cleanest separation of content from look (host config) in the camp; A2UI v1.0 moved to the
  same position.
- Fallback and version negotiation, which the newer formats mostly lack.
- Models know it well from training data.

## Weaknesses / limits

- Designed for small cards in a message, not screens: no tables with selection, no tabs, no live
  binding, no layout beyond columns and containers.
- One-shot: a card is replaced, not patched by id (Action.Execute refresh swaps the whole card).
- The open project has little momentum; the energy is inside Microsoft's products.
- Templating has conditions and repetition (`$when`, `$data` arrays): small, but logic.

## Relation to fictty

**Inspire.** Not a competitor (no terminal, no running screen) and not a good format for fictty's
scope, but the reference design for "host owns the look" and for graceful degradation.

## Could fictty adopt it instead of building?

No. Its component model is too small for fictty's screens (lists and tables with behaviour,
layers, pixels, live data), and there's no patch-by-id. An Adaptive Card *input adapter* (render a
card as a fictty node) is cheap and would let fictty show what Microsoft-ecosystem agents already
emit, but it's low priority.

## What fictty should take from it

- **Host config is fictty's theme, and it should stay data owned by the screen,** not the agent's
  payload. The agent says "emphasis"; the theme decides what emphasis looks like.
- **Fallback.** fictty's pixels-to-cells fallback is the same idea; extend it to unknown component
  types (render a labelled placeholder with the node's text instead of failing the push).
- **Version the format in the value.** A `version` field on every pushed UI, checked by the runtime,
  would make the coming format changes survivable.

## Sources

- https://github.com/microsoft/AdaptiveCards (README, schemas directory, commit history)
- https://adaptivecards.io ; https://adaptivecards.io/explorer/
- https://learn.microsoft.com/en-us/microsoft-copilot-studio/guidance/adaptive-cards-overview
- https://api.npmjs.org/downloads/point/last-month/adaptivecards

Not verified: Microsoft's plans for a schema beyond 1.6, and whether Microsoft 365 Copilot has a
runtime feature that has models generate cards (we found only developer aids and third-party
tools).
