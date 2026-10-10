# Wave Terminal

- **Maker:** Command Line Inc. (wavetermdev); lead developer Mike Sawka (`sawka`)
- **URL:** https://www.waveterm.dev, https://github.com/wavetermdev/waveterm
- **Status (10 October 2026):** v0.14.5, released 16 April 2026; commits continue (last push
  9 October 2026) but there has been no release for almost six months. Electron + Go, Apache-2.0,
  22,465 stars, repository since June 2022.

## What it is

An open-source, AI-integrated terminal app for macOS, Linux and Windows. The window is a tiling
layout of **blocks**: terminals, a built-in editor, web browsers, file previews (Markdown, images,
PDF, CSV), system widgets such as a process viewer, and a Wave AI chat panel. A CLI, `wsh`, lets
anything running in a terminal block open, list and control blocks.

## The problem it's solving

The terminal shows only text, so the moment you need to see a file, a web page or a chart you
leave it. Wave keeps those beside the shell in one layout, reachable from the command line, and
gives an AI assistant context over all of it. Durable SSH sessions (v0.14, February 2026) cover
the remote case.

## Its path / bet

The terminal app *is* the frame, and the frame should hold graphical blocks, not only PTYs. Then
add AI that can see every block. The next step on its roadmap is **Tsunami**: small apps,
written by AI, that run as Wave widgets.

## How it works

- **Blocks and `wsh`.** `wsh view <file|url>` opens a preview or web block beside the shell;
  `wsh edit` opens the editor; `wsh blocks list` lists blocks; `wsh ai` pipes output or files to
  Wave AI; `wsh file` syncs between hosts.
- **Wave AI.** Reads terminal scrollback, takes widget screenshots, navigates web widgets, reads
  and writes files with diff previews and rollback. BYOK and local models since v0.13. Command
  execution is "in progress" on the roadmap.
- **Tsunami (in the repo, roadmap "planned").** A Go framework with a React-like virtual DOM
  (`vdom.H("div", ...)`). The AI builder design (`aiprompts/tsunami-builder.md`) is a split
  screen: chat on the left; preview, code and files on the right. The model edits `app.go` with a
  str_replace tool; every edit runs `go build`; errors go back into the model's context; a
  successful build restarts the app in a preview iframe. Draft and Published versions; the person
  can share a screenshot of the preview with the AI. Demos include a CPU chart, a todo list, a
  table test and a pomodoro timer.

## Strengths

- The clearest example of a frame that hosts graphical content next to terminals, controlled from
  the shell.
- A working idea of AI-generated widgets with a compile-and-retry loop.
- Open source, local-first, no account required.

## Weaknesses / limits

- Electron: a desktop app, not something that crosses SSH to where the agent runs (the remote
  story is the app SSHing out).
- Wave AI is its own assistant; it isn't designed around Claude Code or Codex running in a pane.
- Tsunami is code (Go) compiled per change: slow loop, and the AI reads a screenshot, not state.
- Release cadence has slowed sharply since April 2026; we couldn't find an explanation. The
  Tsunami builder's shipping status is unclear (in repo, "planned" on the roadmap).

## Relation to fictty

**Inspire / watch.** Wave is betting the same thing fictty is (people need to *see* things beside
the agent, built quickly) from the opposite corner: graphical desktop frame, code-first widgets,
read back by screenshot. If Wave ships Tsunami and an agent in a terminal block can call
`wsh` to create a widget, Wave owns "agent puts a live UI beside itself" for its users.

## Could fictty adopt it instead of building?

No. It's a whole desktop terminal app; fictty would become a Wave widget format with no life
outside Wave and no SSH story.

## What fictty should take from it

- **`wsh view` is the right gesture:** one command from inside a shell opens a view beside it.
  `fictty run` should be just as easy to place, and fictty should document "open it beside me"
  recipes for herdr, TUIOS, tmux and zellij.
- **The compile-and-retry loop is the cost fictty avoids.** Tsunami needs a Go build per edit and a
  screenshot to check; fictty's push is validated data and its read-back is exact. Say so plainly
  in the blog post; it's the clearest contrast in this cluster.
- **Draft vs published** is a useful idea for screens an agent iterates on while a person watches.

## Sources

- https://github.com/wavetermdev/waveterm (README, ROADMAP.md, releases v0.12.0–v0.14.5,
  `tsunami/`, `aiprompts/tsunami-builder.md`), read 10 October 2026
- https://docs.waveterm.dev/
