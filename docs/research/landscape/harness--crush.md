# Crush

- **Maker:** Charm (charmbracelet)
- **URL:** https://github.com/charmbracelet/crush
- **Status (2026-10-10):** about 28,600 stars. Licence FSL-1.1-MIT (Functional Source License,
  converting to MIT after two years), not OSI open source today. v0.98.1 on 9 October 2026, with
  nightlies. Created May 2025.

## What it is

Charm's coding agent ("Glamourous agentic coding for all"), written in Go on Charm's own v2
stack: Bubble Tea v2.1.0, Bubbles v2, Lip Gloss v2, Glamour v2 for Markdown, and Ultraviolet
(Charm's cell-level rendering primitives, MIT, about 390 stars). Multi-model, LSP-aware, MCP over
stdio, HTTP and SSE, sessions per project in SQLite.

## The problem it's solving

A terminal agent that looks good and works in every terminal on every OS (macOS, Linux, Windows,
Android, the BSDs), from the company that makes the most widely used Go TUI toolkit. Commercially,
it's also a front end for Charm's model service (Hyper).

## Its path / bet

- **The Elm Architecture in Go.** Bubble Tea is Model/Update/View, the same pattern fictty uses
  in Rust; Crush is the largest showcase of it in an agent.
- **Configuration as a shell.** `crushrc` "is just Bash with some Crush-specific builtins"
  (`mcp add`, `lsp add`, model registration), run by Crush's built-in Bash. The agent configures
  itself through a built-in skill.
- **Claude Code-compatible hooks,** as shell commands. Only `PreToolUse` is supported today.
- **Cooperate with the window manager.** Crush has a native herdr client: inside a herdr pane it
  reports idle, working and blocked state and its session identity over herdr's Unix socket, "so
  herdr can display accurate status without screen scraping."
- **A server.** Crush runs a local server with a versioned HTTP API (`/v1`, self-documenting) over
  a socket, with multi-client tests: the TUI isn't the only possible front end.

## How it works

- `internal/ui` holds Bubble Tea components: chat, dialogs, completions, a diff view, an image
  component, notifications, animation.
- Extensibility is MCP servers, LSPs, skills and hooks. We found no API that lets a plugin or the
  agent draw a custom component in the TUI.

## Strengths

- Same architecture fictty chose (Elm), at scale, with years of Charm polish on terminal
  compatibility, colour profiles and keyboard handling.
- Concrete, shipping integration with herdr: proof that the frame layer and the agent layer can
  talk without screen scraping.
- Portable: the widest OS list of any agent here.

## Weaknesses / limits

- No custom UI from agents or plugins; read-back is text only.
- Source-available rather than open source for two years after each release.
- Hooks are early (one event).

## Relation to fictty

**Complement.** Crush gives an agent nowhere to put a screen, and lives in herdr panes; a fictty
screen beside it is a fit. Crush's herdr client is the template for the integration the first
landscape post recommended ("report status through the window manager").

## Could fictty adopt it instead of building?

No: there's no UI runtime to adopt. Bubble Tea and Ultraviolet are Go; fictty is Rust on ratatui,
and switching languages would buy nothing fictty lacks.

## What fictty should take from it

- **Copy the herdr client.** A small, herdr-specific event vocabulary (idle, working, blocked,
  session id) sent over herdr's socket, decoupled from fictty's own types. fictty's equivalent:
  "blocked" when a screen waits on the person (a form, a picker), "working" while data streams.
  Do the same for TUIOS (OSC 7501).
- **A self-documenting socket API.** Crush's server renders its own endpoint docs. `fictty
  help --json` or a `describe` verb listing every socket operation would help agents.
- **Built-in self-configuration skill.** "Tell Crush how to configure Crush" is the AXI idea;
  fictty's skill should let an agent set up fictty for a project the same way.

## Sources

- Repository, README and licence: https://github.com/charmbracelet/crush
- `go.mod` (Bubble Tea v2.1.0, Ultraviolet), read 2026-10-10
- Hooks: https://github.com/charmbracelet/crush/blob/main/docs/hooks/README.md
- herdr integration: https://github.com/charmbracelet/crush/tree/main/internal/herdr
- Server API: https://github.com/charmbracelet/crush/tree/main/internal/server
- Ultraviolet: https://github.com/charmbracelet/ultraviolet
- Bubble Tea: https://github.com/charmbracelet/bubbletea

Couldn't verify: user numbers beyond stars, and whether Charm plans a plugin UI API (none
announced that we found).
