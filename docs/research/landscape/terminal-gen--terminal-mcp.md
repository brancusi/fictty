# Terminal MCP servers and headless terminals (tmux-mcp, iterm-mcp, terminalcp, ht)

- **tmux-mcp** (Nick G., `nickgnd`): https://github.com/nickgnd/tmux-mcp, 303 stars, MIT,
  created March 2025, last push February 2026.
- **iterm-mcp** (`ferrislucas`): https://github.com/ferrislucas/iterm-mcp, 569 stars, MIT,
  created January 2025, last push September 2025.
- **terminalcp** (Mario Zechner, `badlogic`): https://github.com/badlogic/terminalcp, 128 stars,
  no licence file detected, created and last pushed August 2025; on npm as
  `@mariozechner/terminalcp`.
- **ht** (Andy Konwinski, `andyk`): https://github.com/andyk/ht, 909 stars, Apache-2.0, created
  April 2024, last push July 2025.
- Status as of 2026-10-10: all four are working, older (2024-25) and mostly quiet. Dozens of
  smaller terminal MCP servers exist on registries (glama, mcpmarket); we did not survey them
  individually.

## What they are

The first generation of "let the model use a terminal", before the 2026 CLI drivers:

- **tmux-mcp** exposes tmux sessions, windows and panes as MCP resources and tools: list, capture
  a pane's content, run a command in a pane, split, kill.
- **iterm-mcp** does the same for the active iTerm2 tab on macOS.
- **terminalcp** spawns processes in PTYs with full emulation and offers two read modes: the
  rendered screen with scrollback (for TUIs) or the raw stream (for logs). People can attach to an
  agent's process from their own terminal, like screen. Also a CLI and a Node library.
- **ht** wraps any binary in a PTY plus a VT emulator and speaks JSON over stdin/stdout:
  `input`, `sendKeys`, `takeSnapshot`, `resize`, plus a live web preview. Its README: "needed
  something like a headless browser but for terminals."

## The problem they're solving

The same as tui-use: give a model the keyboard and the screen of an interactive program.

## Their path / bet

MCP (or a JSON pipe) as the transport, the multiplexer or a PTY as the substrate. Mario Zechner's
own later benchmark post (August 2025) argued that many MCP servers flood the context window and
that CLIs often beat them, though stateful tools are easier as MCP servers; the 2026 drivers mostly
went the CLI-plus-skill route, which matches that.

## Strengths

- tmux-mcp and iterm-mcp work on the terminal the person already has open: the agent sees exactly
  what the person sees.
- ht is tiny, a single Rust binary, and its JSON protocol is a clean model of a terminal.

## Weaknesses / limits

- No semantics; pane captures are raw text.
- Several are effectively unmaintained.
- MCP tool overhead in context, per Zechner.

## Relation to fictty

Complement. They are how agents reach existing terminals. fictty's planned MCP server will sit in
the same registries, and should be judged against these on context cost.

## Could fictty adopt them instead of building?

No for the screen. fictty could borrow ht's JSON protocol shape for its own stdin/stdout mode
(useful where sockets are awkward), but it doesn't need ht itself.

## What fictty should take from it

- Keep the MCP adapter thin and the tool list short; CLI plus skill first (as the plan says).
- terminalcp's "person can attach to the agent's process" is the right default: a fictty screen an
  agent starts headless should be attachable from any terminal.
- A stdin/stdout JSON mode, like ht, for hosts that can't open sockets.

## Sources

- https://github.com/nickgnd/tmux-mcp, https://github.com/ferrislucas/iterm-mcp,
  https://github.com/badlogic/terminalcp, https://github.com/andyk/ht (READMEs)
- https://mariozechner.at/posts/2025-08-15-mcp-vs-cli/
- GitHub API data fetched 2026-10-10
