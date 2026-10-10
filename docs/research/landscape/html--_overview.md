# The HTML camp: agents write pages

Research as of 10 October 2026. This extends the HTML section of
[Why the terminal?](../../blog/2026-10-why-the-terminal.md). One dossier per entity:

| dossier | what | relation | threat (0-5) |
|---|---|---|---|
| [Claude Code Artifacts](html--claude-code-artifacts.md) | agent publishes an HTML page from a session to a live claude.ai URL | complement | 3 |
| [Lavish (lavish-axi) and AXI](html--lavish-axi.md) | local browser review loop for agent HTML; long-poll feedback | inspire | 2 |
| ["HTML is the new markdown"](html--html-is-the-new-markdown.md) | the essay and twenty-example gallery behind the camp | inspire | 2 |
| [MCP Apps / MCP-UI](html--mcp-apps.md) | standard for tool-attached HTML UIs in chat hosts | complement | 2 |
| [Cursor Canvas](html--cursor-canvas.md) (new) | agent canvases from a first-party component library in the IDE | inspire | 2 |
| [Claude Artifacts](html--claude-artifacts.md) | the chat-side original, now with templates, storage, connectors | inspire | 2 |
| [Gemini generative UI](html--gemini-generative-ui.md) | per-prompt generated pages in Gemini and Search | inspire | 1 |
| [ChatGPT Canvas and Apps SDK](html--chatgpt-canvas-apps-sdk.md) | Canvas dropped from current models; developer widgets via MCP Apps | watch | 1 |
| [v0](html--v0.md) | agent builds and deploys real apps | watch | 1 |
| [Imagine with Claude](html--imagine-with-claude.md) | five-day demo of software generated per click | inspire | 1 |

Nothing on the brief turned out not to exist. Two corrections to the first pass: ChatGPT Canvas is
no longer available in OpenAI's current models (since 28 May 2026), and Cursor Canvas, which we
missed, belongs here.

## What is really being solved

Everyone in this cluster starts from the same observation: **a UI used to be expensive, so agents
answered in text; now a purpose-built interface costs one model turn.** That changes what a UI is
for. It no longer has to be a durable product that pays back its cost over years. It can be a
throwaway made for one question, one review, one explanation, and then discarded or kept as a link.

Under that, three problems keep recurring:

1. **Text is the wrong shape for spatial and interactive things.** Diffs, call graphs, options side
   by side, timelines, simulations. This is Thariq's argument, Google's research result (raters
   strongly preferred generated pages to text and Markdown, speed aside) and the reason every lab
   built a page surface.
2. **Understanding, not just output.** The examples people reach for first are explainers: how a
   feature works, a concept explainer, a module map, a fractal you can zoom. Generated interactive UI
   is quietly becoming a teaching medium, and Google's consumer work is explicitly about it. Almost
   nobody measures it.
3. **The way back.** A page the person can't answer through is a broadcast. The camp spent 2026
   building return paths: copy-as-prompt buttons, Lavish's long poll with element-level
   annotations, Artifact comments sent to Claude with the session watching, MCP Apps'
   `update-model-context`, widget state in ChatGPT.

## The paths

- **The model writes the page, the host keeps it** (Claude Artifacts, Claude Code Artifacts, Gemini
  dynamic view, Imagine with Claude). Free-form HTML/JS, sandboxed, the full web as design ceiling.
  The bet: models write good-enough HTML and a browser is always near.
- **The developer writes the template, the model calls it** (MCP Apps, MCP-UI, OpenAI Apps SDK).
  Reviewed HTML widgets attached to tools; data arrives through tool results. The bet: products and
  security need fixed templates; the model only decides when.
- **The model writes code against a catalogue** (Cursor Canvas's component library, Claude's
  Slides/Docs/Design/Dashboards templates). A drift toward constrained output for recurring shapes.
  This is the HTML camp walking toward the data camp.
- **A local loop around the page** (Lavish). Keep HTML, but make review precise and waiting cheap,
  with no account and any agent.
- **The model writes the app** (v0). Durable code that ships. The far end of the ephemerality axis.

## The open debate

- **Code or data?** The HTML camp's evidence is that models are good at HTML and people like the
  results. Its own behaviour says something else: Anthropic added typed templates, Cursor uses a
  component library, MCP Apps fixes templates in advance, Google pairs free-form HTML for consumers
  with A2UI (data) for developers, and OpenAI pulled Canvas back to inline blocks. Nobody is
  defending "regenerate the whole page every time" for recurring shapes.
- **Where does the data come from?** In most of the camp the numbers are frozen into the page when
  the model writes it. Live data is arriving through hosted connectors (Artifacts, claude.ai) and
  tool calls (MCP Apps), always through a vendor's proxy. Nobody binds a local command or stream to
  a page.
- **Can the agent see what it made?** Read-back is source code, DOM snapshots, app-chosen context or
  database rows. Nobody offers the rendered frame as exact text. MCP Apps explicitly deferred
  screenshots.
- **Who owns the surface?** Each lab's page surface is tied to its own host and account. MCP Apps is
  the only cross-vendor piece, and it is for developer widgets, not agent pages. Lavish is the only
  agent-neutral page loop, and it's local and single-maintainer.

## What this means for fictty

The HTML camp owns **pages to keep and share**, and with comments, watching, connectors and a shared
database, Claude Code Artifacts now reaches into ten-minute working screens for anyone with a
browser and a claude.ai account. That's the real competitive pressure, and it should be said
plainly in the blog post. fictty should not build hosting, sharing, comments or HTML review: export
to HTML and hand off to Artifacts or Lavish instead.

What the camp does not do, and fictty does:

- data bound to **local** commands and streams at frame rate, never through the model or a vendor
  proxy;
- **exact read-back** of what the person sees, as text and as state;
- behaviour (keys, focus, selection) in a runtime, so it is consistent across every screen rather
  than re-implemented in every page;
- any agent, no account, in the pane, over SSH.

The camp also hands fictty its to-do list: a `watch` at least as good as Artifact comments and
Lavish's poll, point-at-a-node feedback, layout warnings reported to the agent, declared and
reviewable data-source scope (like MCP Apps' CSP and Artifacts' declared connectors), an MCP server
face, and the Thariq gallery rebuilt as fictty recipes as the honest test of the terminal's limits.
