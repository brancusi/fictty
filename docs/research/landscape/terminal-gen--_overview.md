# Agent-generated UI in the terminal: the closest neighbours

Research as of 10 October 2026. This extends the "Terminal: let the model drive a screen" section
of [the landscape post](../../blog/2026-10-why-the-terminal.md). One dossier per entry, in this
directory, named `terminal-gen--<slug>.md`.

## What's really being solved

Every project here is answering one of two questions:

1. **How does an agent put something in front of a person, in the terminal, and get an answer
   back?** (claude-canvas, Laura, json-render's Ink renderer, the A2UI renderers, Raxol, fictty)
2. **How does an agent use a terminal screen it didn't write?** (tui-use, agent-tui, tui-test,
   agent-tty, tmux and terminal MCP servers, ht, VHS)

Underneath both is the same gap that ConductorOne's Paul Querna put in one line: "A terminal
program is already a state machine. The problem is that the state is presented as a screen."
The first group tries to give the agent a screen it understands because it described it. The second
tries to recover understanding from screens that were never meant to be read by machines.

## The paths people are going down

**1. Prebuilt canvases.** claude-canvas: a few hand-designed screens (calendar, document, flight)
the agent fills with JSON and opens in a tmux split; the person's click comes back over a socket.
Proved the demand in a week (about 1,500 stars), then stopped. Lesson: people want the agent to
open a screen beside itself and get an answer back; a fixed set of screens doesn't scale.

**2. A format plus a terminal renderer, as a library.** json-render's Ink renderer (Vercel Labs),
the Rust a2ui crate on ratatui, a2ui-ink, a2tea for Bubble Tea. The agent (or the host app's
model) writes a declarative spec; a renderer embedded in someone's app draws it. A2UI now renders
in a terminal in three languages, and json-render has the adoption. None of them is a process an
outside agent attaches to: no socket, no patch by id from outside, no read-back of the frame, no
data sources besides the agent itself. They answer "what should a UI value look like", not "where
does it live while the agent works".

**3. Inline in the agent's own chat.** a2tea's target (a fork of Charm's Crush) and json-render's
`ink-chat` example render generated UI inside the transcript. This is the most natural home for a
one-shot form or picker, and if a major agent ships it, that use case leaves every standalone tool.

**4. An agent-driven workspace.** Laura (laura-tui): the agent tiles panes of files, diffs, plans
and logs over an NDJSON socket by pane id, highlights lines and comments; the person answers with
an inline review that arrives in the chat. The loop is the same as fictty's; the content is
documents rather than components. Very new and unknown (0 stars), but the most fictty-like thing
in the terminal.

**5. Code-first frameworks with an agent surface.** Raxol (Elixir) derives MCP tools from its
component tree, so an agent operates an app structurally; Melker (Deno) makes apps readable,
permissioned documents. Both are for apps a developer writes; an agent can't conjure a screen in
them without writing code.

**6. Drive any screen.** A flood of near-identical PTY drivers in 2026 (tui-use, pilotty, two
unrelated agent-tuis, agent-tty, PTY-Agent, agentic-tui), the older MCP servers (tmux-mcp,
iterm-mcp, terminalcp) and ht, Microsoft's tui-test rewritten in Rust with a Playwright-style
locator API, and VHS for recording. They work with every program and know nothing about any. The
best of them try to recover structure: ConductorOne's agent-tui adds roles and refs through
per-program adapters; tui-test reads cells and styles; tui-use reads inverse-video highlights as
"selection". The number of clones says the plumbing is a commodity.

## The open debate

- **Where does generated UI live: in the chat, beside it, or in a browser?** a2tea and ink-chat
  say inline; claude-canvas and Laura say a split beside the agent; the HTML camp (other cluster)
  says a browser tab. fictty bets on beside, for screens that outlive one message.
- **Format or runtime?** The format question is close to settled (A2UI and json-render, both with
  terminal renderers). The runtime question (a long-lived, attachable screen with exact read-back
  and data that skips the model) has almost no takers: Laura for documents, Raxol for code-first
  apps, fictty for data-first screens.
- **Retrofit or publish structure?** Drivers retrofit a DOM onto programs with adapters. A screen
  that is data publishes its own. Both will coexist; fictty should make itself the easiest screen
  for drivers to read, not compete with them.
- **Code or data in the value?** json-render allows expressions (`$cond`, `$computed`); Melker
  allows handlers; fictty and A2UI keep logic out. The trade is expressiveness against safety and
  read-back.

## Where this leaves fictty

Nothing here does all of fictty's loop: a process the agent pushes to, patches by id, drives with
keys and mouse, reads back exactly as text and state, with live data bound to commands and
streams. The closest are Laura (same loop, documents only) and the format renderers (same
values, no loop). That position is real but narrow, and the risks are concrete:

- If json-render or A2UI ships an attachable terminal host with a socket, the format renderers
  close most of the gap quickly, with far more adoption.
- If the big agents render generated UI inline in their own TUIs, short-lived forms and pickers
  stop needing a separate screen.
- The driver crowd can read any screen "well enough" for many tasks.

What to take, in priority order (each is argued in its dossier):

1. **Overflow and visibility read-back** after every push (Laura).
2. **`watch` with `selected`/`cancelled` results and screen predicates** (claude-canvas,
   tui-use, agent-tui).
3. **Accept json-render (Ink catalogue) specs as input alongside A2UI, use RFC 6902 patches,
   and test against the official A2UI v0.9.1 examples** (json-render, a2ui-ink, a2ui crate).
4. **An outline view in roles and refs, and an agent-tui adapter,** so drivers see fictty
   screens exactly (agent-tui).
5. **Emulator-level conformance tests with tui-test,** and `fictty agent-context` as a JSON
   command schema (tui-test).
6. **Visible, approvable command policy** for pushed UIs (Melker), and **tree-derived MCP tools
   with a focus lens** (Raxol).
7. **History exports that double as proof** (agent-tty) **in a readable, tape-like text form**
   (VHS).

## Entities checked

Covered with dossiers: claude-canvas, json-render's Ink renderer, the Rust a2ui crate, a2tea,
a2ui-ink, Laura, Raxol, Melker, tui-use (with pilotty, pproenca/agent-tui, PTY-Agent,
agentic-tui), ConductorOne agent-tui, Microsoft tui-test, Coder agent-tty, terminal MCP servers
and ht, VHS.

Added beyond the brief: Laura, a2tea, a2ui-ink, Melker, ConductorOne agent-tui, Microsoft
tui-test's Rust rewrite.

Looked at and left out: GitHub's TUIKit (an LLM compiles component specs into framework code; see
`framework--tuikit.md`), chicago-desktop/tui-desktop (a terminal compositor on the Wippy runtime
with an agent command channel; 0 stars, two weeks of commits), agent session managers such as
agent-deck and agent-manager (frame layer, not content), Toad (an agent front end, not generated
UI). We found no project named "tui-use" other than onesuper's, and no evidence that "terminalcp"
is maintained beyond August 2025.

Couldn't verify: how much of `@json-render/ink`'s 591,000 monthly npm downloads is real terminal
use; whether Charm officially backs a2tea; Raxol's performance and agent claims (README only).
