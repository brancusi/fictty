# MCP Apps (and MCP-UI)

- **Maker:** the Model Context Protocol project; SEP-1865 authored by Ido Salomon and Liad Yosef
  (MCP-UI), Olivier Chafik and Jerome Swannack (Anthropic), Jonathan Hefner, Anton Pidkuiko, Nick
  Cooper, Bryan Ashley and Alexi Christakis (OpenAI and others).
- **URL:** https://github.com/modelcontextprotocol/ext-apps ; https://github.com/MCP-UI-Org/mcp-ui
- **Status (10 October 2026):** SEP-1865 is Final, Extensions Track; the first official MCP
  extension, announced 26 January 2026 with spec version 2026-01-26 (stable) and a draft in
  progress. `ext-apps` repo: about 2,900 stars, SDK at v2.0.3 (25 September 2026), last push 25
  September 2026, licence shown by GitHub as "Other" (README says Apache-2.0). MCP-UI: about 5,200
  stars, Apache-2.0, created May 2025, last push 16 September 2026, now positioned as an MCP Apps
  implementation. Hosts listed as supporting it: Claude, ChatGPT (fully compatible from 22 February
  2026), VS Code, Goose, Postman, MCPJam and others; support varies by host.

## What it is

A standard way for an MCP server's tool to come with an interactive HTML interface. The server
predeclares UI resources under a `ui://` URI; a tool references one in its metadata; the host
fetches it, renders it in a sandboxed iframe, and passes the tool's input and result into it. The
UI talks back to the host over MCP's own JSON-RPC.

## The problem it's solving

Tool results were text or JSON the model then had to describe. Many tools (a map, a booking flow, a
chart, a product picker) need a real interface, and before this each host (MCP-UI hosts, OpenAI's
Apps SDK) did it differently. The spec unifies them so one server works in every host.

## Its path / bet

HTML in an iframe, written by the *tool's developer* ahead of time, not by the model. The model
decides when to call the tool; the template is fixed; data flows in through tool results. That
separation of template and data is deliberate: hosts can prefetch and security-review templates.
The spec's authors considered external URLs and remote-DOM and deferred both, partly over "model
visibility" and the inability to screenshot.

## How it works

- Resource MIME type `text/html;profile=mcp-app`; tool metadata `_meta.ui.resourceUri`.
- View to host requests: `ui/initialize`, `tools/call` and `resources/read` (proxied to the server),
  `ui/open-link`, `ui/message` (add a chat message), `ui/request-display-mode` (inline,
  fullscreen, picture-in-picture), `ui/update-model-context`.
- Host to view: `ui/notifications/tool-input` (and `-partial` for streaming), `tool-result`,
  `tool-cancelled`, `host-context-changed` (theme, styles), `ui/resource-teardown`.
- `_meta.ui.csp` declares `connectDomains`, `resourceDomains`, `frameDomains`; the default is
  `connect-src 'none'`. Permissions (camera, microphone, geolocation, clipboard) are requests the
  host may honour.
- **What the model sees:** tool `content` goes to the model; `structuredContent` does not. The
  View can call `ui/update-model-context`, which overwrites its previous context and is given to the
  model on later turns. There is no defined way to read back the rendered UI as text or image;
  screenshot APIs are listed as a future consideration.

## Strengths

- A real, multi-vendor standard with the two largest chat hosts behind it. The closest thing the
  field has to "write the UI once, show it in any agent host."
- Clean security story: predeclared templates, sandboxing, auditable JSON-RPC, declared CSP.
- Template/data separation means data can reach the UI without passing through the model: a tool
  result with `structuredContent` goes to the iframe, not the context window.
- The UI can call tools itself, so interaction doesn't need a model turn.

## Weaknesses / limits

- HTML only. A terminal host can't render it without a browser engine, so terminal agents (Claude
  Code, Codex CLI, opencode) get the tool but not the interface.
- The model doesn't write the UI; a developer does. That is the right call for products, and the
  wrong one for a throwaway screen an agent wants for ten minutes.
- Read-back is opt-in and app-defined (`update-model-context`), not exact.
- Host support varies; ChatGPT keeps a `window.openai` layer of extras on top.

## Relation to fictty

**Complement.** MCP Apps is for chat hosts and developer-authored widgets; fictty is for terminal
panes and agent-authored screens. Two real touchpoints: fictty could be reached *as an MCP server*
(push, patch, screen, get as tools, which is cheap and probably overdue), and a fictty UI value
could be rendered as an MCP App by a small HTML renderer, so the same screen shows in a chat host.
That second one is worth noting but not worth building yet.

## Could fictty adopt it instead of building?

No. It has no terminal renderer and no notion of an agent-authored, live-patched screen. Adopting
its *vocabulary* is cheap and sensible: tool-input/tool-result, display modes, declared CSP and
`update-model-context` map onto things fictty already has or needs.

## What fictty should take from it

- **Template versus data, enforced.** MCP Apps separates the reviewable template from the data that
  fills it. fictty's UI value already does this; say so in those words.
- **Declared network and command scope.** `connectDomains` with a deny-by-default is the model for
  declaring which commands a fictty screen may run.
- **Expose fictty as an MCP server** with tools mirroring the CLI. Every major agent speaks MCP;
  not all of them will shell out to a CLI well.
- **Model context as an explicit channel.** `ui/update-model-context` is the app deciding what the
  model should know. fictty's equivalent is exact and automatic (`screen`, `get --state`), which is
  a genuine advantage; keep it compact (AXI-style) so it doesn't cost more context than it saves.

## Sources

- [SEP-1865: MCP Apps](https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp) (Final)
- [modelcontextprotocol/ext-apps](https://github.com/modelcontextprotocol/ext-apps) and its [2026-01-26 specification](https://github.com/modelcontextprotocol/ext-apps/blob/main/specification/2026-01-26/apps.mdx); [releases](https://github.com/modelcontextprotocol/ext-apps/releases)
- [MCP Apps announcement](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/) (26 January 2026)
- [MCP-UI](https://github.com/MCP-UI-Org/mcp-ui), [mcpui.dev](https://mcpui.dev/)
- OpenAI, [Apps SDK changelog](https://developers.openai.com/plugins/changelog) (ChatGPT MCP Apps compatibility, 22 February 2026)

Not verified: secondary claims that the extension was folded into an extensions framework in a
2026-07-28 MCP spec release.
