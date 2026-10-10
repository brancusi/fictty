# Gemini CLI (and its successor, Antigravity CLI)

- **Maker:** Google
- **URL:** https://github.com/google-gemini/gemini-cli, https://geminicli.com
- **Status (2026-10-10):** about 107,300 stars, Apache-2.0, still releasing (v0.64.0-preview.1
  on 9 October 2026, nightlies of 0.65). But on **19 May 2026** Google announced it is
  "transitioning Gemini CLI to Antigravity CLI": consumer Gemini CLI (Google AI Pro and Ultra, and
  free Gemini Code Assist users) stopped serving requests on **18 June 2026**. It remains
  available to Standard and Enterprise licence holders and through paid API keys. Antigravity CLI
  (`agy`) is written in Go, shares a harness with the Antigravity 2.0 desktop app, and is not
  described as open source in Google's post; several reports call it a closed binary.

## What it is

Google's open-source terminal coding agent (launched June 2025), a TypeScript monorepo whose TUI
is React on Ink. Its packages include an A2A server (`a2a-server`), a VS Code companion and an
SDK. Extensions (October 2025) bundle MCP servers, commands, context files, themes, hooks,
sub-agents and skills.

## The problem it's solving

The same as the other harnesses: an agent beside the editor, in the terminal. Its rendering work
in late 2025 targeted the flicker, scroll-jump and resize problems every Ink app hits.

## Its path / bet

- **Stay on Ink, but fork it.** `packages/cli` depends on `ink` aliased to `@jrichman/ink@6.6.9`,
  a fork maintained by Jacob Richman, the Google engineer who wrote the rendering post.
- **Alternate screen by default** since v0.15.0 (November 2025): sticky headers for tool
  confirmations, a prompt anchored at the bottom, mouse clicks to place the cursor, and the full
  chat written back to normal scrollback on exit.
- **Interop through protocols,** not UI: A2A for agent-to-agent, MCP for tools. No extension
  point draws custom UI in the terminal that we could find.
- **Then, a different product.** Google's bet moved to Antigravity: a desktop app plus a Go CLI
  on the same harness, with "Antigravity plugins" replacing extensions.

## How it works

- React components rendered by the forked Ink to an alternate screen buffer; static history
  committed through Ink's `Static` pattern in normal mode. (A Qwen Code design doc says Gemini
  CLI later replaced parts of the fork's approach with its own memoised static render; secondary,
  unverified.)
- Extensions install from a Git URL or path (`gemini extensions install`). They add capabilities
  for the model; they don't draw.

## Strengths

- Open source with a large community, many forks (Qwen Code, about 28,400 stars, is the best
  known, and documents its own Ink 7 virtual-viewport design).
- A2A server out of the box: the clearest "agent as a service" packaging among the four.
- A documented, user-visible rendering overhaul with honest notes on what was still missing.

## Weaknesses / limits

- **The product was demoted.** For most people, Gemini CLI is no longer Google's terminal
  agent; Antigravity CLI is, and it isn't open.
- Ink's model (React reconciling to strings, full-line redraws) is the one Anthropic and opencode
  both left; a fork is a maintenance burden.
- No agent-authored or plugin-authored UI, no read-back beyond text.

## Relation to fictty

**Watch.** Gemini CLI users (now mostly enterprise) get nothing inside the harness for live
screens, so a fictty pane is a fit, as with Codex. The more important fact is strategic: the
first landscape post said "all four labs ship their flagship coding agent as a terminal
program"; Google's flagship terminal agent is now a closed Go binary tied to a desktop app. That's
the "is the terminal a phase?" question becoming concrete, and it should be corrected in the
post.

## Could fictty adopt it instead of building?

No. There's nothing here to adopt: no UI runtime, and its renderer is a fork of a library fictty
deliberately doesn't use.

## What fictty should take from it

- **Write the session back to scrollback on exit.** Gemini CLI's alternate screen still leaves a
  readable transcript behind. A fictty screen should leave its last frame (or a summary) in
  scrollback when it closes, so the person and the agent keep a record.
- **Mouse is a progressive enhancement.** Ctrl-S to drop out of mouse mode for native selection
  is the pattern fictty needs too.
- **Labs move fast and close things.** Depending on a harness's extension API (see the Claude
  Code dossier) carries this risk; a standalone CLI and socket don't.

## Sources

- Repository and releases: https://github.com/google-gemini/gemini-cli
- `packages/cli/package.json` (the `@jrichman/ink` alias), read 2026-10-10
- Jacob Richman and Jack Wotherspoon, Making the terminal beautiful one pixel at a time (13 November 2025):
  https://developers.googleblog.com/en/making-the-terminal-beautiful-one-pixel-at-a-time/
- An important update: transitioning Gemini CLI to Antigravity CLI (19 May 2026):
  https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/
- Extensions launch: https://blog.google/technology/developers/gemini-cli-extensions/ and https://geminicli.com/docs/extensions/
- Coverage of the transition: https://virtualizationreview.com/articles/2026/05/19/google-moves-gemini-cli-into-antigravity-cli-as-agent-platform-expands.aspx
- Qwen Code virtual viewport design (secondary on Gemini's renderer): https://qwenlm.github.io/qwen-code-docs/en/design/virtual-viewport/README/

Couldn't verify: how Antigravity CLI draws (Go is stated; whether it uses Bubble Tea or anything
else is not), whether it has any UI extension point, and whether it's open source.
