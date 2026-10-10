# OpenAI ChatKit widgets (and Open-JSON-UI)

- **Maker:** OpenAI
- **URL:** https://developers.openai.com/api/docs/guides/chatkit,
  https://github.com/openai/chatkit-js, https://github.com/openai/chatkit-python,
  https://widgets.chatkit.studio
- **Status (10 October 2026):** Apache-2.0. `chatkit-js` 1,962 stars, last release
  `@openai/chatkit-react` 1.6.1 on 31 July 2026; `chatkit-python` 392 stars, v1.6.5 on 19 May 2026.
  Both repositories created 4 October 2025 (launched with AgentKit at DevDay 2025).
  `@openai/chatkit-react` about 310,000 npm downloads in the month to 8 October 2026. **Agent
  Builder**, the hosted workflow product ChatKit launched with, is scheduled to shut down on
  30 November 2026; the docs say "ChatKit is still available" and steer new work to the
  self-hosted server integration. Release cadence has slowed since mid-2026.

## What it is

A drop-in chat UI (a web component plus React bindings, served from OpenAI's CDN) with **widgets**:
structured cards and lists the assistant streams into the conversation. Widgets are designed in
ChatKit Studio's Widget Builder in a "strict, simplified JSX dialect", exported as `.widget` files
(layout, data schema and sample data), hydrated on the server with Jinja-style `{{ }}`
placeholders, and streamed to the client as JSON.

**Open-JSON-UI** is CopilotKit's name for "an open standardization of OpenAI's internal declarative
Generative UI schema"; CopilotKit lists it next to A2UI as a declarative spec. We found no
standalone repository or spec document for it, only CopilotKit's docs pages.

## The problem it's solving

Giving teams a production chat front end with rich, interactive responses without building one,
and keeping the model's output safe by constraining it to designed templates.

## Its path / bet

**Templates designed by people, filled by agents.** The layout is authored ahead of time (visually,
in Studio); at runtime the server builds a widget from a template plus data. The model decides
when to show a widget and supplies data; it can generate widget JSON directly, but the documented
path is template plus data.

## How it works

- **Containers:** `Card` (with confirm and cancel actions), `ListView` (rows of `ListViewItem`),
  `Basic` (entity previews). **Components:** Box, Row, Col, Text, Title, Caption, Markdown, Badge,
  Image, Button, Form, Select, DatePicker, charts and more.
- **Python server:** `WidgetTemplate.from_file("x.widget").build(data)` returns a Pydantic model;
  `stream_widget` sends it. Yielding successive versions from a generator makes the SDK **diff**
  them and emit update events; only `Text` and `Markdown` nodes with an `id` stream text smoothly,
  other diffs re-render that part.
- **Actions:** `onClickAction`, `onSubmitAction`, `onChangeAction` with a `type` and `payload`,
  handled on the server by default or on the client with `handler: "client"`.
- **Back to the model:** a widget converts into model input (`ThreadItemConverter.widget_to_input`),
  and each widget carries `copy_text`, a plain-text rendering.

## Strengths

- Polished, opinionated, and backed by the largest model vendor.
- The template-plus-data split is sound: design once, hydrate many times.
- **`copy_text` and `widget_to_input`**: a plain-text version of every widget, for the clipboard
  and for the model. A small, practical form of read-back.
- Diffing successive widget states on the server makes "just re-render the whole thing" cheap for
  the developer.

## Weaknesses / limits

- Tied to OpenAI's hosted script and, until November, to Agent Builder; the product it launched
  with is being retired, which makes its long-term priority unclear.
- Chat-embedded only: widgets live in messages, not on a standing screen.
- Jinja templating and a JSX dialect are two more languages; the template, not the model, holds the
  layout.
- Browser only. Open-JSON-UI's status as a "standard" rests on one vendor's description.

## Relation to fictty

**Watch.** A different surface and a vendor-locked client. The ideas worth noting are the
template-plus-data split (fictty's project components) and the plain-text twin of every widget
(fictty's `screen`).

## Could fictty adopt it instead of building?

No. The renderer is OpenAI's hosted web component; the widget schema has no published standalone
spec we could find; and there's no terminal story.

## What fictty should take from it

- **A plain-text twin for every node,** designed for the model as much as for the clipboard.
  fictty's `screen` gives the frame; a per-node "copy text" in `get --state` would let an agent
  quote one card without parsing the whole frame.
- **Diff successive whole values on the server side** so producers can just re-push; fictty
  already makes a whole push as cheap as a frame, which is the stronger version of the same idea.
- **Retired hosting is a risk lesson:** formats bound to one vendor's hosted runtime inherit its
  product decisions. fictty's runtime is local and MIT; say so.

## Sources

- https://developers.openai.com/api/docs/guides/chatkit (and the .md version, shutdown notice)
- https://developers.openai.com/api/docs/guides/chatkit-widgets
- https://github.com/openai/chatkit-python/blob/main/docs/concepts/widgets.md
- https://github.com/openai/chatkit-python/blob/main/docs/guides/build-interactive-responses-with-widgets.md
- https://github.com/openai/chatkit-js
- https://openai.com/index/introducing-agentkit/ ; https://mcp.directory/blog/openai-agentkit-deprecation-2026
- https://github.com/CopilotKit/generative-ui ; https://docs.copilotkit.ai/generative-ui-specs/open-json-ui

Not verified: the exact JSON shape of a serialized widget (the docs we read describe components
and props but don't show a full payload), and whether Open-JSON-UI has any spec beyond CopilotKit's
pages.
