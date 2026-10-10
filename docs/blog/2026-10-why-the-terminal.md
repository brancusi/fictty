---
title: "Why the terminal? Agents, throwaway screens and where fictty fits"
description: Terminal UIs are having their biggest year in decades while a well-known engineer tells everyone to stop making them. We looked at why the boom is happening, what the terminal really costs, who else is building screens for agents, and what that leaves for fictty.
date: 2026-10-10T18:00:00Z
kicker: research
---

*Update, 10 October 2026: two corrections from our second pass. OSC 7501 is Superlogical's Program
Status Protocol, made for its Rex terminal and implemented by TUIOS and others; it isn't TUIOS's
own. And Google's flagship terminal agent is no longer the open-source Gemini CLI: in May 2026
Google moved consumers to Antigravity CLI, which Google doesn't describe as open source (reports
call it closed) and which is tied to a desktop app. The text below is corrected, and the list of
what we're taking from this research now says what we're considering, except where it's already in
[the plan](../PLAN.md). The [follow-up](2026-10-disposable-interfaces.md) has the rest of what
changed.*

In August, Thomas Ptacek published an essay called
[Stop Making TUIs](https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/). His argument is
short: coding agents now write a decent native app as cheaply as a terminal one, so "building a
CLI is almost always a good idea. Building a TUI almost never is." TUIs, he says, "exist for just
two reasons: modems, and because Unix nerds didn't want to learn Motif." It drew
[539 comments on Hacker News](https://news.ycombinator.com/item?id=49384210) and a link from
[Simon Willison](https://simonwillison.net/2026/Aug/21/stop-making-tuis/), who admitted he was
"running out of excuses."

We're building [fictty](https://github.com/brancusi/fictty), a terminal UI runtime for agents. So
we owe ourselves a straight answer: is the terminal the right place for this, or are we riding a
wave that's about to break? This post is what we found. It covers why the terminal boom is
happening, what a terminal really costs compared with the web, Electron and native, who else is
building quick, disposable screens for agents, and where that leaves fictty.

## The boom, in numbers

Stars are a vanity metric, but they show direction. GitHub, on 10 October 2026:

| project | what it is | stars |
|---|---|---|
| opencode | open-source coding agent, TUI on OpenTUI | 212k |
| Claude Code | Anthropic's coding agent | 150k |
| Codex CLI | OpenAI's coding agent, TUI on ratatui | 128k |
| Gemini CLI | Google's open-source coding agent, TUI on Ink (consumers moved to Antigravity CLI in May) | 107k |
| Bubble Tea | Go TUI framework (Charm) | 45k |
| herdr | a multiplexer built for coding agents | 43k |
| Ink | React for the terminal | 40k |
| Textual | Python TUI framework | 37k |
| zellij | terminal workspace | 36k |
| Crush | Charm's coding agent | 29k |
| ratatui | Rust TUI library | 23k |
| OpenTUI | Zig-core TUI library with React and Solid bindings | 13.5k |
| TUIOS | a terminal window manager for agents | 5.1k |

All four labs ship a coding agent as a terminal program, though Google's flagship is no longer the
open one in this table: on 19 May 2026 it moved consumers from Gemini CLI to
[Antigravity CLI](https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/),
a Go binary Google doesn't describe as open source (reports call it closed) that shares its
harness with the Antigravity desktop app. The two most-starred
multiplexer-style projects of the year (herdr and TUIOS) exist to manage terminals full of agents.
Something real is going on.

## Why now

We found five reasons that hold up. Most coverage of the "TUI renaissance" is opinion, and some of
it is unsourced (one widely shared piece claims TUIs start in under 50 ms and use 10–50 MB against
"200–500 MB for a comparable GUI," with no measurement behind it). These are the reasons with
evidence.

**1. The model reads the terminal natively.** A CHI 2026 workshop paper,
[Terminal Is All You Need](https://arxiv.org/abs/2603.10664) (De Masi, University of Geneva),
argues that terminal agents won on three properties: the interface and the model share a
representation (text), the agent's actions are visible in the same medium the person watches, and
the barrier to entry is low. A terminal frame is a grid of characters. An agent can read it back
exactly, with no screenshot and no vision model in between. That's not true of any other UI.

**2. People want the agent beside the editor, not inside it.** Arnór Heiðar Sigurðsson's
[Why are coding agents all going to the terminal?](https://arnorhs.dev/posts/2025-10-11/why-are-coding-agents-all-going-to-the-terminal/)
makes the case that developers don't love terminals so much as they love an agent that lives
*apart* from their editor, and the terminal is the cheapest way to get that. He also lists the
pain: the mouse barely works, resizing is finicky, copy and paste is awkward, and a chat interface
strains a medium built for scrollback. He expects these tools to become desktop apps within a few
years. OpenAI already shipped one: the [Codex app](https://openai.com/index/introducing-the-codex-app/)
(February 2026) is a desktop command centre for several agents at once. The terminal isn't the only
place agents live. It's where they started and where most of them still run.

**3. Agents run somewhere else, for a long time.** Agents run on remote boxes, in containers, in
worktrees, for hours. A terminal crosses SSH with nothing extra installed. The two breakout
projects of the year are about exactly this: [herdr](https://herdr.dev) ("the runtime your coding
agents live on") keeps sessions alive across disconnects and shows local and SSH machines in one
window; [TUIOS](https://tuios.dev) restores sessions after a reboot. Neither would make sense as a
browser tab.

**4. The toolkits got fast.** Rendering a busy, streaming UI in a terminal without flicker used to
be hard. Three of the big agents rebuilt their rendering in the past eighteen months:

- **Codex CLI** moved from TypeScript to Rust, largely to ship
  [standalone binaries without Node](https://github.com/openai/codex/tree/main/codex-rs), and draws
  with ratatui.
- **opencode** left Go and Bubble Tea for [OpenTUI](https://opentui.com), a Zig core with Yoga
  flexbox and React or Solid on top, built by the same team.
- **Claude Code** kept React as its component model but
  [replaced Ink's renderer](https://steipete.me/posts/2025/signature-flicker) with its own: build
  the UI, lay it out, turn it into a screen of cells, diff against the last frame, and emit the
  minimum ANSI. Secondary reports say Anthropic also sent patches to tmux and VS Code's terminal for
  synchronized output (DEC mode 2026); we couldn't confirm that from a primary source.

Cell diffing is the same idea that makes ratatui fast, and the one fictty leans on: a full screen
is about 12,000 cells, so diffing it every frame is cheap.

**5. Agents write TUIs well.** The same thing Ptacek points at cuts in both directions. If an agent
can write a SwiftUI app in an afternoon, it can write a Bubble Tea app in an hour. Cheap to build
is a reason for more TUIs as much as fewer.

## TUIOS, herdr, and the window-manager layer

TUIOS (the *Terminal UI Operating System*) is worth a close look, because the name suggests
something it isn't.

It's a terminal window manager by [Gaurav Gosain](https://github.com/Gaurav-Gosain/tuios), written
in Go on Bubble Tea v2 and Lipgloss v2, MIT, about 5,100 stars. It started in September 2025 as a
floating and tiling window manager (v0.1.0 in October 2025), grew a daemon with detach and reattach
in v0.5.0 (December 2025), and in v0.8.0 (27 September 2026) turned toward agents. Its tagline is
now "a terminal window manager that knows what your agents are doing." It has:

- BSP, master-stack and scrolling layouts, nine workspaces and a vim-style modal interface
- sessions that survive restarts, an SSH server mode and a browser terminal (`tuios-web`)
- per-pane agent status (working, waiting, done, errored) and one **Inbox** for every approval and
  question, with push notifications to your phone
- agent-to-agent messaging, fleets of agents in git worktrees, and checkpoints of each agent's turn
- a JSON control protocol over a socket, a `.tape` scripting language with recording and replay,
  and support for OSC 7501, an escape sequence any program can emit to report its status (the
  [Program Status Protocol](https://www.superlogical.com/rex/docs/build/program-status), which
  Superlogical made for its Rex terminal)
- full kitty graphics support, including video

[herdr](https://github.com/herdrdev/herdr) (Rust, Apache-2.0, 43,000 stars) is the larger
neighbour with the same idea: tmux-style keys, per-pane agent status, sessions that survive a
dropped SSH connection, and a socket API that lets agents spawn panes, prompt each other and wait
on each other. TUIOS ships a herdr compatibility layer, which says a lot about who set the shape.

These tools manage **terminals that contain agents**. They decide where panes go and tell you
which agent needs you. None of them gives an agent a way to put a *screen* in front of you: a
table you can sort, a form, a dashboard bound to live data. The panes are the frame; what's
inside a pane is still whatever program happens to run there. That's the layer fictty is for, and
it sits *inside* their panes rather than competing with them. A fictty screen in a herdr or TUIOS
pane, reporting "waiting for you" through OSC 7501, is a combination worth building.

Other projects in this layer: [zellij](https://github.com/zellij-org/zellij) (with WASM plugins),
tmux (still the thing agents reach for when told to "use tmux to test"), kitty's own sessions
(LWN called kitty
"[a kind of TUI construction kit](https://lwn.net/Articles/1080821/)"), and
[Wave](https://github.com/wavetermdev/waveterm), an Electron terminal whose `wsh` CLI lets an agent
open a web page or preview beside itself.

## The real trade-offs

Ptacek's essay is right more often than TUI fans would like. Here's our honest table:

| | terminal (TUI) | web app | Electron / Tauri | native |
|---|---|---|---|---|
| **Distribution** | one binary, or nothing new if it's data in a runtime | a URL; best of all | an installer; Electron ships Chromium (tens of MB) | per platform, signing, stores |
| **Startup, memory** | small, but published numbers are mostly unsourced | depends on the tab | measured gaps vary a lot by study; Tauri usually smaller | small and fast |
| **Remote use** | SSH, no setup; the strongest argument | needs a server and a port | poor over remote | poor over remote |
| **Agent can read it** | exactly, as text | DOM or accessibility tree, often a screenshot | same as web, harder to reach | accessibility APIs or a screenshot |
| **Agent can drive it** | keys and mouse over a PTY; every program | browser automation | harder | OS automation |
| **Accessibility** | poor in practice | best tooling | inherits the web's | good, platform-native |
| **Input** | keyboard-first; mouse is limited; the kitty keyboard protocol helps | full | full | full |
| **Graphics** | cells, plus pixels on kitty-protocol terminals | anything | anything | anything |
| **Design ceiling** | a character grid | none | none | none |

Three rows deserve a sentence each.

**Accessibility is the terminal's real weakness.** Casey Reeves's
[The text mode lie](https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility)
argues that modern TUI frameworks (Ink, Bubble Tea, tcell among them) are worse for blind users
than badly built GUIs, because they redraw the screen constantly and use the terminal as a canvas
rather than a stream. GitHub's team wrote about the same problem
[making `gh` accessible](https://github.blog/engineering/user-experience/building-a-more-accessible-github-cli/).
A UI that is data has an advantage here (the runtime knows a row is a row and can say so), but
nobody gets that for free, us included.

**Graphics are better than people think, and patchier.** The
[kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) now runs in kitty,
Ghostty, WezTerm, Konsole, Warp and iTerm2, among others, and it supports alpha blending and
images above or below text. Sixel is wider but can't layer. Multiplexers complicate it: tmux needs
passthrough and Unicode placeholders, and zellij speaks sixel only. So pixels are a progressive
enhancement, never the floor.

**Agent-readability is the row nobody else wins.** A web page can be read by an agent, but
through a DOM, an accessibility tree or a picture. A terminal frame is the same text the person
sees. For a screen an agent builds and then checks, that's decisive.

Our take: **Ptacek is right about apps people keep.** If you're building a tool you'll use every
day for a year, build a native app or a web app. The terminal wins somewhere narrower: **the
screen an agent puts up for ten minutes, next to where it's already working, which it needs to
read back.** That's not a consolation prize. It's most of what agents put in front of people.

## Throwaway UIs: who's doing what

Agents making quick, disposable interfaces is now a category of its own, and the labs have taken
sides. They split into three camps.

### HTML: let the model write the page

- **"HTML is the new markdown."** Thariq Shihipar, who works on Claude Code at Anthropic, argued in
  May 2026 that for most non-trivial output, agents should write self-contained HTML rather than
  Markdown, with a [gallery of twenty examples](https://thariqs.github.io/html-effectiveness/): spec
  comparisons, annotated code reviews, custom editors. "Diffs and call-graphs are spatial
  information; markdown flattens them."
- **Claude Code Artifacts** (beta from 18 June 2026) publish one HTML page from a session to a
  private claude.ai link that updates as the session goes on
  ([overview](https://noqta.tn/en/blog/claude-code-artifacts-live-shareable-pages-guide-2026)).
  The chat-side ancestor, Claude Artifacts, and the
  [Imagine with Claude](https://www.datacamp.com/tutorial/imagine-with-claude) research preview
  (September 2025), which generated software on the fly on a virtual desktop, push the same idea
  further.
- **Lavish.** This is the one we were asked about. [lavish-axi](https://github.com/kunchenguid/lavish-axi)
  by Kun Chen (MIT, about 4,000 stars, first commit May 2026, v0.1.85 on 9 October) bills itself
  as "the new editor for your HTML artifacts." The agent writes HTML; `lavish-axi <file>` opens it
  in a local browser; the person annotates elements, edits Mermaid diagrams as whiteboards, and
  queues feedback; the agent collects it with `lavish-axi poll <file>` (a long poll) and revises
  the file, which live-reloads. It's an *AXI*, Chen's term from [axi.md](https://axi.md) for an
  "Agent eXperience Interface": ten rules for CLIs agents use, such as token-efficient output,
  definitive empty states and "content first" (a bare command shows live data, not help).
  Lavish is the closest thing we found to fictty's loop in the browser: agent writes, person
  responds in place, agent waits cheaply for the response.
- **Google's generative UI.** Gemini's [dynamic view and visual layout](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/)
  experiments write a custom interactive page per prompt, and AI Mode in Search builds tools and
  simulations on the fly.
- **MCP Apps** became the first official MCP extension on
  [26 January 2026](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/): a tool can
  return an HTML UI that the host renders in a sandboxed iframe. Claude, ChatGPT, VS Code and Goose
  support it. It grew out of [MCP-UI](https://github.com/MCP-UI-Org/mcp-ui) and OpenAI's Apps SDK,
  and OpenAI's own [ChatKit](https://developers.openai.com/api/docs/guides/chatkit) carries on
  while its Agent Builder is retired in November.

### Data: let the model fill in a catalogue

- **[A2UI](https://github.com/a2ui-project/a2ui)** (Google, Apache-2.0, 16,700 stars): agents
  stream declarative component descriptions; each client renders them with its own native widgets.
  v0.9.1 is the stable release and 1.0 is a release candidate.
- **[json-render](https://github.com/vercel-labs/json-render)** (Vercel Labs, 18,600 stars): a
  catalogue of components with Zod schemas, a flat JSON spec, streamed patches, and renderers for
  React, Vue, Svelte, React Native, PDF, email, video, 3D and, notably,
  **Ink in the terminal**.
- **[OpenUI](https://github.com/thesysdev/openui)** (Thesys, 10,600 stars): a compact streaming
  language rather than JSON, claiming about half the tokens of json-render or Thesys's own C1 JSON
  on their benchmark.
- **[AG-UI](https://github.com/ag-ui-protocol/ag-ui)** and CopilotKit: the event protocol between an
  agent backend and an app's frontend, with generative UI on top.
- **The Vercel AI SDK**: tool calls that render React components.

### Terminal: let the model drive a screen

- **json-render's Ink renderer** and a Rust crate, [a2ui](https://docs.rs/crate/a2ui) by Liangdi
  (June 2026), which renders A2UI on ratatui with 18 components. A2UI in the terminal is no longer
  open ground.
- **[claude-canvas](https://github.com/dvdsgl/claude-canvas)**: prebuilt Ink canvases Claude Code
  opens in a tmux split. It proved the demand in a week, then stayed a demo.
- **[Raxol](https://github.com/DROOdotFOO/raxol)** (Elixir): one Elm-architecture module rendered
  to a terminal, LiveView, SSH and MCP, with MCP tools derived from the live component tree. It's
  the project that took the same bet as us most seriously, code-first rather than data-first.
- **Screen drivers** (agent-tty, tui-test, the many tmux skills): run any TUI in a PTY and let the
  agent read the screen and send keys. They work with every program ever written and know nothing
  about any of them.
- GitHub's experimental [TUIKit](https://github.com/github/tuikit) goes the other way: component
  specs in Markdown that an LLM compiles into Bubble Tea, Ink, OpenTUI and ratatui code.

### The map

Put them on two axes: **what the agent writes** (code, or data a runtime renders) and **where it
shows up** (a browser or chat, or the terminal). Then mark which ones let the agent read the live
UI back.

| | browser, chat, native | terminal |
|---|---|---|
| **agent writes code** | HTML artifacts, Claude Code Artifacts, Lavish, Imagine with Claude, Gemini dynamic view, MCP Apps, v0 | Ink, Bubble Tea, ratatui, Textual, OpenTUI apps; claude-canvas; Raxol; TUIKit |
| **agent writes data** | A2UI, json-render, OpenUI/C1, AG-UI, AI SDK generative UI | json-render/ink, the a2ui crate, **fictty** |
| **manages the frame, not the content** | Codex app, Wave | TUIOS, herdr, zellij, tmux |

The bottom-right cell is crowded with renderers. What none of them has is the loop: a long-running
screen that an agent can push to, patch by id, drive with keys and read back exactly, while the
numbers on it stream from commands that never pass through the model.

## Where fictty fits

So, narrowly:

> **fictty is a terminal screen an agent writes as one JSON value, edits live by id, and reads
> back exactly, while its data streams straight from your commands, never through the model.**

What that rules out:

- **It isn't a window manager.** TUIOS and herdr arrange agents; fictty is what an agent shows you
  inside one of their panes.
- **It isn't an HTML artifact.** If the output is a report to share or keep, publish HTML. fictty
  is for the screen you use for ten minutes while the agent works.
- **It isn't a new standard.** A2UI and json-render already define formats. fictty's own format is
  a means; an A2UI adapter is on the plan, and A2UI's terminal renderers mean that alone isn't a
  moat.
- **It isn't a framework for shipping apps.** If you're building a TUI people will install, use
  ratatui, Bubble Tea, Textual or OpenTUI directly.
- **It isn't a screen scraper.** It doesn't drive other programs; it runs its own screen and knows
  what every cell means.

The claims it has to win on:

1. **The loop is exact.** Push, patch by id, send keys, then read the frame as text and the state
   as JSON. An agent checks its work without a screenshot.
2. **Data skips the model.** Sources are commands and streams bound to nodes. The prototype draws a
   500-update-a-second ticker at over 100 fps; the model writes the screen once and never relays a
   number.
3. **Behaviour runs at frame rate.** Scrolling, selection, focus and mouse run in the runtime, so a
   person navigating never waits for a model.
4. **It runs where the agent already is.** A pane, a remote box, an SSH session. Pixels where the
   terminal has them, cells everywhere else.
5. **It can be replayed.** Every input is a message and the state is serializable, so history an
   agent can rewind and fork is cheap to build. It isn't built yet, and until it is, this is a
   promise, not a claim.

What this round of research changes. Two items are already in [the plan](../PLAN.md):

- **`watch` is the Lavish lesson.** Lavish's best idea is a cheap long poll for the person's
  response. fictty's planned `watch` is the same verb; it moves up the list.
- **Don't count on the A2UI adapter to differentiate.** It's on the plan, and it's table stakes
  now.

And what we're considering:

- **Report status through the window manager.** Emit OSC 7501 (which TUIOS reads, and whatever
  herdr reads) when a screen is waiting for the person, so fictty shows up in the Inbox people
  already watch.
- **Follow the AXI rules for the CLI.** Compact output, definitive empty states, next-step hints.
  `fictty screen` with no screen running should say so in one line.
- **Treat accessibility as a feature of the data model,** not an afterthought: a plain linear
  rendering of the same UI value, for screen readers and for agents with small context.

## What we couldn't settle

- **Is the terminal a phase?** Arnór expects agent CLIs to become desktop apps; OpenAI already
  built one. If the agents leave the terminal, a terminal runtime for their screens goes with them.
  Our bet is that remote, long-running agents keep the terminal relevant even if the chat moves to
  a window.
- **Startup and memory.** We couldn't find a careful, sourced comparison of TUI, Electron, Tauri
  and native. The numbers in circulation conflict by factors of five.
- **Does HTML win anyway?** The HTML camp has the lab behind Claude Code and a fast-growing tool in
  Lavish. If agents get good enough at self-contained HTML and a browser is always one keystroke
  away, the terminal's advantage shrinks to remote and read-back. We think those two are enough.
  We'll find out.

## Sources

- Thomas Ptacek, [Stop Making TUIs](https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/), and the [Hacker News thread](https://news.ycombinator.com/item?id=49384210); Simon Willison's [link post](https://simonwillison.net/2026/Aug/21/stop-making-tuis/)
- Alexandre De Masi, [Terminal Is All You Need](https://arxiv.org/abs/2603.10664), CHI 2026 workshop CUCHI'26
- Arnór Heiðar Sigurðsson, [Why are coding agents all going to the terminal?](https://arnorhs.dev/posts/2025-10-11/why-are-coding-agents-all-going-to-the-terminal/)
- [The TUI renaissance](https://byteiota.com/tui-renaissance-2026-why-terminal-uis-are-back/) (byteiota; read with care, several figures are unsourced)
- OpenAI, [Introducing the Codex app](https://openai.com/index/introducing-the-codex-app/); [codex-rs](https://github.com/openai/codex/tree/main/codex-rs)
- Peter Steinberger, [Claude Code's signature flicker](https://steipete.me/posts/2025/signature-flicker); [Claude Code uses React to render its TUI](https://analyticsindiamag.com/ai-news-updates/claude-code-uses-react-to-render-its-tui)
- [OpenTUI](https://opentui.com), [ratatui](https://github.com/ratatui/ratatui), [Bubble Tea](https://github.com/charmbracelet/bubbletea), [Textual](https://github.com/Textualize/textual), [Ink](https://github.com/vadimdemedes/ink)
- [TUIOS](https://github.com/Gaurav-Gosain/tuios) and [tuios.dev](https://tuios.dev); Superlogical, [Program Status Protocol (OSC 7501)](https://www.superlogical.com/rex/docs/build/program-status); Google, [Transitioning Gemini CLI to Antigravity CLI](https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/); [herdr](https://github.com/herdrdev/herdr); [zellij](https://github.com/zellij-org/zellij); [Wave](https://github.com/wavetermdev/waveterm)
- Lee Phillips, [Kitty chases the mouse](https://lwn.net/Articles/1080821/), LWN; the [kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- Casey Reeves, [The text mode lie](https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility); GitHub, [Building a more accessible GitHub CLI](https://github.blog/engineering/user-experience/building-a-more-accessible-github-cli/)
- [Tauri vs Electron vs Deno vs Electrobun](https://betterstack.com/community/guides/scaling-nodejs/tauri-vs-electron-vs-deno-vs-electrobun/) (Better Stack); [Tauri vs Electron 2026](https://tech-insider.org/tauri-vs-electron-2026/)
- Thariq Shihipar, [HTML effectiveness gallery](https://thariqs.github.io/html-effectiveness/); coverage of [HTML vs Markdown](https://pasqualepillitteri.it/en/news/2243/html-vs-markdown-claude-code-thariq-anthropic)
- [Claude Code Artifacts guide](https://noqta.tn/en/blog/claude-code-artifacts-live-shareable-pages-guide-2026); [Imagine with Claude](https://www.datacamp.com/tutorial/imagine-with-claude)
- Kun Chen, [lavish-axi](https://github.com/kunchenguid/lavish-axi) and [axi.md](https://axi.md)
- Google Research, [Generative UI](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/)
- [MCP Apps announcement](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/), [SEP-1865](https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp), [MCP-UI](https://github.com/MCP-UI-Org/mcp-ui), [ChatKit](https://developers.openai.com/api/docs/guides/chatkit)
- [A2UI](https://github.com/a2ui-project/a2ui) and [InfoQ on v0.9](https://infoq.com/news/2026/07/google-a2ui-genui); [json-render](https://github.com/vercel-labs/json-render); [OpenUI](https://github.com/thesysdev/openui); [AG-UI](https://github.com/ag-ui-protocol/ag-ui)
- [a2ui crate](https://docs.rs/crate/a2ui) (ratatui); [Raxol](https://github.com/DROOdotFOO/raxol); [TUIKit](https://github.com/github/tuikit); [tui-test](https://docs.rs/crate/tui-test-rs/latest)
- Our earlier posts: [a UI that is one value](https://thoughtcontrol.app/blog/a-ui-is-one-value/), [mapping 33 tools](https://thoughtcontrol.app/blog/mapping-the-landscape/), [fictty](https://thoughtcontrol.app/blog/fictty/)
