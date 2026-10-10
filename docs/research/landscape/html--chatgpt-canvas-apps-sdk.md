# ChatGPT Canvas and the Apps SDK

- **Maker:** OpenAI
- **URL:** https://developers.openai.com/apps-sdk/ ; https://help.openai.com/en/articles/9624314-model-release-notes
- **Status (10 October 2026):**
  - **Canvas** launched 3 October 2024 (GPT-4o with Canvas). OpenAI's model release notes say that
    from 28 May 2026 "canvas will no longer be available in GPT-5.5 Instant or GPT-5.5 Thinking";
    writing and coding moved into chat as "writing blocks" and "code blocks". It remains on legacy
    models until they are retired. No product-wide retirement has been announced.
  - **Apps SDK** launched in preview at DevDay in October 2025 (SEP-1865 says November 2025),
    built on MCP. ChatGPT became "fully
    compatible" with MCP Apps on 22 February 2026; `window.openai` is now described as a
    compatibility layer plus ChatGPT-specific extras. Apps can be distributed as Codex plugins
    (March 2026). Closed host, open SDK docs.
  - **ChatKit** (embeddable chat UI) continues; Agent Builder is being retired in November 2026
    (from the first landscape pass; not re-verified here).

## What it is

Two different answers from one lab. Canvas was a side panel where the model wrote and edited a
document or code with the person, including running simple HTML/React previews. The Apps SDK lets
developers ship apps that appear inside ChatGPT: an MCP server whose tools come with HTML widgets
rendered inline, picture-in-picture or fullscreen.

## The problem it's solving

Canvas: chat is a bad place to co-edit a long object. Apps SDK: ChatGPT as a platform, where
third-party services show real interfaces in the conversation instead of text descriptions of them.

## Its path / bet

OpenAI has stepped back from the "model writes a page beside the chat" path (Canvas dropped from
the current models in favour of inline blocks) and doubled down on the developer-authored widget
path (Apps SDK, now converged on MCP Apps). That's the opposite of Anthropic's 2026 direction, where
model-written pages (Claude Code Artifacts) grew. The disagreement between the two labs is one of
the clearest signals in this cluster.

## How it works

- An app is an MCP server. Tools declare a UI template (`_meta.ui.resourceUri`, alias
  `openai/outputTemplate`) and CSP.
- ChatGPT renders the widget in a sandboxed iframe and supplies `toolInput`, `toolOutput`
  (`structuredContent`; "the model reads it as-is", so keep it concise) and `toolResponseMetadata`
  (hidden from the model).
- The widget keeps `widgetState` via `setWidgetState`, calls tools with `callTool`, posts into the
  chat with `sendFollowUpMessage`, requests display modes, uploads and selects files.
- Users choose how often connected apps ask permission (June 2026).

## Strengths

- The largest consumer chat surface; distribution through an app directory.
- Converged on the open standard, so the same server works in Claude and VS Code.
- Clear control over what the model sees (`structuredContent`) versus what only the widget sees
  (`_meta`), which is the same "data that skips the model" idea fictty has, applied to tool results.

## Weaknesses / limits

- Widgets are code a developer ships, reviewed ahead of time. Not a medium for an agent's
  throwaway screen.
- Chat-only surface; nothing for terminal agents (Codex CLI gets plugins, not widgets).
- Canvas's withdrawal from current models suggests OpenAI found model-written side panels less
  valuable than inline blocks, or too costly to maintain; OpenAI hasn't said why.

## Relation to fictty

**Watch.** No direct overlap with a terminal runtime. The Canvas reversal is a data point for the
blog post: one lab is betting the model should write pages, the other pulled back to inline
structured blocks and developer-built apps.

## Could fictty adopt it instead of building?

No. It's a host feature of ChatGPT.

## What fictty should take from it

- **Inline blocks beat side panels for most answers.** OpenAI's retreat to writing and code blocks
  says the heavy surface should be summoned, not default. fictty screens should be opt-in and
  short-lived, and an agent should be told when *not* to open one.
- **Separate model-visible and UI-only data in the format.** The `structuredContent`/`_meta` split
  is a clean pattern; fictty's `get --state` could offer a compact model view versus the full state.
- **Display modes** (inline, PiP, fullscreen) map neatly onto a pane, a callout layer and a full
  screen; worth borrowing the names.

## Sources

- OpenAI, [Model release notes](https://help.openai.com/en/articles/9624314-model-release-notes) (28 May 2026 entry on canvas)
- OpenAI, [Apps SDK reference](https://developers.openai.com/apps-sdk/reference) and [changelog](https://developers.openai.com/plugins/changelog)
- [OpenAI launches Canvas](https://www.bolnews.com/technology/openai-launches-new-canvas-interface-for-chatgpt) (October 2024, secondary)
- [OpenAI silently drops Canvas from GPT-5.5](https://aiweekly.co/alerts/openai-silently-drops-canvas-from-gpt-55-update) (secondary)
- [MCP Apps vs OpenAI Apps SDK](https://mcp.directory/blog/mcp-apps-standard-vs-openai-apps-sdk-2026) (secondary)

Not verified: whether o3 (the last legacy model with Canvas) was in fact retired on 26 August 2026,
and whether the Apps SDK is still labelled preview.
