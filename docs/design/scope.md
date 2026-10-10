# Scope

What fictty is for, what it deliberately isn't, and the claims it has to win on. The research
behind this is in [the landscape post](../blog/2026-10-why-the-terminal.md) (October 2026).

## The pitch

**A terminal screen an agent writes as one JSON value, edits live by id, and reads back exactly,
while its data streams straight from your commands, never through the model.**

## Who it's for

- **Agents working in a terminal** (Claude Code, Codex, opencode, Gemini CLI and the rest) that
  need to put a screen in front of a person for minutes, not months: a walkthrough, a review queue,
  a live dashboard, a picker, a form.
- **People supervising those agents,** often over SSH or in a herdr or TUIOS pane, who want to
  navigate at frame rate and answer without waiting for a model.
- **App authors** (Thought Control, ownpurse) who describe screens as data and bind them to their
  own commands.

## What it is

A runtime. One process owns the terminal; the UI is a serializable value in its state; agents
push, patch, drive and read it over a socket or the CLI; data sources feed it directly; the
runtime does drawing, scrolling, selection, focus, mouse and pixels.

## What it deliberately is not

- **Not a window manager or multiplexer.** TUIOS, herdr, zellij and tmux arrange agents. fictty is
  what runs inside one of their panes.
- **Not an HTML artifact tool.** Reports to keep or share belong in HTML (Claude Code Artifacts,
  Lavish). fictty is for the screen used while the agent works.
- **Not a new UI standard.** A2UI and json-render define formats; fictty speaks them as inputs.
- **Not a framework for apps people install.** Use ratatui, Bubble Tea, Textual or OpenTUI.
- **Not a screen scraper.** It doesn't drive other programs; it runs its own screen.
- **Not a programming language.** A pushed UI is data: substitution only, no loops or conditions.

## The claims it must win on

1. **Exact read-back.** After any push, patch, key or click, `screen` returns the frame as text and
   `get --state` returns everything as JSON. No screenshot, no vision model.
2. **Data that skips the model.** Commands and streams bound to nodes at thousands of updates a
   second; the model writes the screen once and never relays a number.
3. **Behaviour at frame rate.** Navigation, selection, focus and mouse run locally; a person never
   waits on a model to move a cursor.
4. **Runs where the agent is.** A pane, a remote box, SSH. Pixels on kitty-protocol terminals,
   cells everywhere else.
5. **Replayable by construction** (planned): every input a message, the state serializable, so an
   agent can rewind, fork and replay a session.

## Nearest neighbours

| neighbour | what it does | how fictty differs |
|---|---|---|
| TUIOS, herdr | terminal window managers that track agent status, with socket APIs | they manage panes; fictty is the content of a pane (and should report status to them, e.g. OSC 7501, Superlogical's status protocol, which TUIOS reads) |
| Lavish (lavish-axi) | browser editor for agent HTML; the person annotates, the agent long-polls for feedback | same loop, in the terminal, with live data and exact read-back; the `watch` verb is our long poll |
| Claude Code Artifacts, MCP Apps | HTML pages published or rendered in a chat host | HTML is code in a browser; fictty is data in the terminal the agent already runs in |
| A2UI, json-render (incl. its Ink renderer), the a2ui crate | declarative UI formats and renderers, some in the terminal | renderers, not a live process an agent patches by id, drives and reads back; fictty adopts A2UI as an input |
| Raxol | Elixir Elm-architecture UI with MCP tools from the component tree | code-first; fictty is data-first, so any agent can write a screen with no build step |
| claude-canvas, PTY screen drivers | prebuilt Ink canvases; scraping any TUI over a PTY | fixed canvases or no semantics; fictty knows what every cell means |
| ratatui | the drawing library | fictty sits on it and adds the loop, data binding, focus, theme and the socket |
