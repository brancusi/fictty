---
title: "Interfaces you throw away: a conversation about what agent UI is trying to solve"
description: For forty years a UI was durable and expensive; now an agent makes one in seconds. Aram and Claude argue through about ninety projects, products, essays and studies to work out what everyone building agent interfaces is trying to solve, what the evidence says about learning by seeing, and whether fictty should exist at all.
date: 2026-10-10T20:00:00Z
kicker: research
---

*This is a conversation between Aram ([@brancusi](https://github.com/brancusi)), who is building
[fictty](https://github.com/brancusi/fictty) (a terminal UI runtime for agents), and Claude,
Anthropic's model, which did the landscape research with him. It's put together from their working
sessions over the second pass of that research and made public: Aram's lines are his questions and
positions from those sessions, lightly edited; Claude's carry the evidence and the questions back.
Neither of them ends up quite where they started. The
[first post](2026-10-why-the-terminal.md) asked whether the terminal is the right place for this;
the [research notes](../research/landscape/), one dossier per project, are in the repo.*

## Why a terminal at all?

**Aram:** Start with the boom. Everyone is building terminal UIs again. Why a terminal UI? What's
the benefit over just building an Electron app or a web app? What's driving it, and what are the
real trade-offs?

**Claude:** The first post found five reasons that hold up. The model reads a terminal frame
natively, as the same text the person sees. People want the agent beside the editor rather than
inside it. Agents run remotely for hours, and a terminal crosses SSH with nothing installed. The
toolkits got fast: every big agent rewrote its renderer in the last eighteen months. And agents
write TUIs as cheaply as they write anything else. Against that, the terminal is weak on
accessibility, limited on input, patchy on graphics, and its design ceiling is a grid of
characters.

**Aram:** Is there a real debate, though? Is it just better to build a web app?

**Claude:** There is, and the strongest voice is on the other side. Thomas Ptacek's
[Stop Making TUIs](https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/) says that if an agent
can write a native app as cheaply as a terminal one, you should build the native app. For tools
people keep and use every day, I think he's right. The best replies don't refute him on durable
apps. They argue for remote use, keyboard speed, and Jane Street's
[finding](https://blog.janestreet.com/strace-ui-bonsai-term-and-the-tui-renaissance/) that when
agents can check screens as text, their UI work is "more often correct on the first try." Ptacek
himself names a semantic "New Terminal", a complete break from VT100, as the strongest rebuttal to
his own essay.

**Aram:** And what is Anthropic doing about these quick, ephemeral UIs? Where is Claude in this?

**Claude:** Anthropic is betting on both sides at once. In the terminal, Claude Code went fullscreen
with its own cell-diffing renderer, and since v2.1.287 on 1 October,
[mods](https://code.claude.com/docs/en/plugins/mods/overview) are on by default: plugin JavaScript
that draws panes beside the transcript, a band above the prompt, and replacement rows, with
buttons, inputs, images and live data from processes. In the browser there are three page
surfaces: Artifacts in chat, [Claude Code Artifacts](https://code.claude.com/docs/en/artifacts)
(live, versioned pages with comments delivered back to a watching session), and
[inline visuals](https://claude.com/blog/claude-builds-visuals) that appear mid-conversation and go
away. Anthropic is also the clearest of the labs about the distinction you care about. Its own
words: Artifacts are "permanent tools and documents", inline visuals "are temporary", made "to aid
users' understanding."

**Aram:** How much do mods change?

**Claude:** More than anything else in this pass. It's the thing the first post missed that matters
most, so I should list everything we got wrong before we go further.

## What we got wrong last time

**Claude:** Several of the first post's facts aged within weeks.

- "All four labs ship their flagship coding agent as a terminal program" is no longer true. On
  19 May 2026 Google moved consumers from the open-source Gemini CLI to
  [Antigravity CLI](https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/),
  a Go binary tied to a desktop app and not described as open source. OpenAI's Codex sends rich
  visuals to the browser or the ChatGPT desktop app, which absorbed the Codex app on 9 July.
- Claude Code can now be drawn into by code, through mods. A Claude Code user who wants a screen
  beside the agent may never need anything else.
- [pi](https://github.com/earendil-works/pi) belongs in the table: about 114,000 stars and an
  extension API that draws overlays and widgets.
- Exact text read-back isn't unique to fictty. herdr's `pane.read`, zellij's
  `subscribe`/`dump-screen`, tmux's `capture-pane` and `tuios screenshot --text` all give an agent
  the text of any pane. What's still unclaimed is reading back *state*: which row is selected, what
  was typed, by node id.
- OSC 7501, the "I'm waiting for you" escape sequence, is
  [Superlogical's](https://www.superlogical.com/rex/docs/build/program-status) Program Status
  Protocol, written for its Rex terminal. TUIOS implements it; Bubble Tea v2.1 added it on
  8 October. We called it TUIOS's.
- ChatGPT Canvas is gone from current models (since GPT-5.5, 28 May 2026), and Cursor Canvas,
  which we missed, belongs in the HTML camp.
- We overstated Raxol. Its idea is good, but it has about 80 stars and one maintainer.

**Aram:** Read-back was one of the claims we led with. What's left of it?

**Claude:** The narrower version, and I think it's the stronger claim anyway. Any multiplexer can
give an agent the text of a pane. Knowing that the person moved to row
fourteen and typed "no" into the field called `reason` is a different kind of knowledge, and nobody
else in the landscape offers it for a screen the agent wrote.

## What's ultimately being solved

**Aram:** Then we need the unique position. Not a feature list. What is everyone in this space
ultimately trying to solve?

**Claude:** Can I turn that around first? When you picture someone using fictty, what are they
doing in the minute before they need a screen?

**Aram:** Reading text from the agent. That's the problem I keep coming back to.

**Claude:** Then that's close to the answer. Across all of it, the problem is the gap between what
the computer knows and what the person understands, at the moment they need it. Different groups
close that gap in different places. I'd put them on six paths:

| path | the question it answers | who | what the content is |
|---|---|---|---|
| **The frame** | Which of my fifty agents needs me? | herdr, TUIOS, tmux, zellij; the Codex app, Warp, cmux, Superset | whatever runs in the pane; in desktop frames, increasingly HTML |
| **The harness** | How do I watch and steer one agent without the screen fighting me? | Claude Code, Codex CLI, opencode, pi, Crush, Gemini CLI | the transcript, plus plugin code (mods, TUI plugins, extensions) |
| **The page** | How do I show something text can't? | Claude Artifacts and Claude Code Artifacts, Claude's inline visuals, Gemini's generative UI, Lavish, MCP Apps, Cursor Canvas, v0 | HTML and JavaScript the model or a developer wrote |
| **The data** | How do I show UI without running code the model wrote? | A2UI, json-render, OpenUI, AG-UI, ChatKit widgets, Adaptive Cards | a declarative description a trusted renderer draws |
| **The terminal runtime** | How does an agent put a screen in front of a person, in the terminal, and get an answer back? | claude-canvas, Laura, json-render's Ink renderer, the A2UI terminal renderers, Raxol, fictty; and on the reading side, a flood of PTY drivers | prebuilt screens, documents, or data |
| **The malleable and the instrument** | How does a person understand, and change, the tools they use? | Ink & Switch, Robin Sloan, Maggie Appleton, Geoffrey Litt, Bret Victor, Observable, marimo | adaptable tools over shared data; instruments built for one question |

**Aram:** Walk me through the ones nearest us. The frame layer first: herdr, TUIOS.

**Claude:** The frame layer is about attention, not UI. [herdr](https://herdr.dev) went from
nothing to 43,000 stars in six months. It, TUIOS, tmux and zellij answer four questions for someone
running many long-lived agents: which one needs me (per-pane status and an inbox), will it survive
a dropped connection (persistent sessions), can agents coordinate without me (a socket API), and
where do I review (diffs, previews). Desktop frames like the Codex app,
[Superset](https://superset.sh), [cmux](https://cmux.com) and Warp answer the same questions in a
window, and they're steadily filling their panes with HTML: browsers you can comment on, Pages,
MCP Apps.

Terminal frames haven't done that yet. herdr's plugin v1 is terminal UI only. TUIOS has a small
`ask-human`, a question with fixed answers, and dock components. Zellij has four host-drawn
components, reached from compiled WASM. The content of a terminal pane is still unclaimed. I'd add
one warning: the frame is hard to make a business of. Vibe Kanban's company
[shut down in April](https://www.vibekanban.com/blog/shutdown) with 28,000 stars.

**Aram:** And the harnesses?

**Claude:** They all rebuilt their renderers, then split on who else may draw. Claude Code left Ink
for its own renderer, opencode built [OpenTUI](https://opentui.com), Codex draws with ratatui and
keeps native scrollback, pi diffs lines inside synchronized output. Flicker is a solved problem
now, solved separately by every team. On drawing, Claude Code, opencode and pi let third-party
*code* draw. Codex and Google send rich visuals out to a browser or desktop app. Crush and Gemini
CLI let nobody draw. Every harness that opened its screen chose code, and none gives the model an
exact read-back of what a plugin drew.

**Aram:** Isn't the page camp going to win? Models are good at HTML.

**Claude:** They are, and the camp's founding line is a good one. Thariq Shihipar, who works on
Claude Code, put it this way in ["HTML is the new markdown"](https://thariqs.github.io/html-effectiveness/):
"Diffs and call-graphs are spatial information; markdown flattens them."
Google's [generative UI](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/)
writes a page per prompt in Gemini and Search. [Lavish](https://github.com/kunchenguid/lavish-axi)
builds a local review loop around agent HTML: the person annotates elements, and the agent collects
the feedback with a long poll.

What interests me is that the camp's behaviour is drifting toward data. Anthropic added typed
templates (Docs, Slides, Design, Dashboards) in September. Cursor Canvas constrains the agent to a
first-party component library.
[MCP Apps](https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp)
fixes templates ahead of time and lets the model fill them. OpenAI dropped Canvas for inline
blocks. Nobody is defending "regenerate the whole page every time" for shapes that recur.

**Aram:** Isn't that the data camp's whole argument?

**Claude:** Yes. The data camp is solving one problem: let a model put interactive UI in front of a
person without the host running code the model wrote. It's server-driven UI, which
[Adaptive Cards](https://adaptivecards.io) did for bots in 2017, redone under three new pressures:
streaming (flat lists with ids, or JSON Patch lines, so a half-sent UI still draws), tokens
(OpenUI's terse language, A2UI's Express proposal), and editing over several turns (everything
addressable by id). [A2UI](https://github.com/a2ui-project/a2ui) bets on a portable format where the
host owns the look. [json-render](https://github.com/vercel-labs/json-render) bets on a toolkit with
thirty renderers, including the only widely used terminal one, on Ink.
[OpenUI](https://github.com/thesysdev/openui) went all the way to a language with variables, loops
and data queries. Consolidation has started: Tambo Cloud shuts down on 31 October, OpenAI retires
Agent Builder on 30 November, and the AI SDK moved `streamUI` back to experimental. Almost all of it
lives in a chat in a browser, and treats the UI as a message: a card in a thread that a later
message replaces.

**Aram:** And who's actually doing what we're doing, in a terminal?

**Claude:** Two groups. The first is about how an agent puts something in front of a person.
[claude-canvas](https://github.com/dvdsgl/claude-canvas) got about 1,500 stars in a week for three
prebuilt Ink screens in a tmux split, then stopped. A2UI now renders in a terminal in three
languages: the Rust `a2ui` crate on ratatui, `a2ui-ink`, and `a2tea` on Bubble Tea.
[Laura](https://github.com/thomaseleff/laura-tui) gives an agent a socket to tile panes of files,
diffs and logs by id, and returns the person's inline review to the chat. That's the same loop as
fictty, for documents rather than components.

The second group is about how an agent uses a screen it didn't write, and it's crowded: tui-use,
pilotty, two unrelated projects called agent-tui, Coder's agent-tty, Microsoft's tui-test rewritten
in Rust with Playwright-style locators. ConductorOne's
[agent-tui](https://www.c1.ai/labs/research/agent-tui-structured-terminal-access-for-ai-agents)
writeup has the line I'd pin above the whole field: "A terminal program is already a state
machine. The problem is that the state is presented as a screen." The number of clones says the
plumbing is a commodity.

**Aram:** Why are Litt and Ink & Switch on the map? They aren't building agent UI.

**Claude:** They're the most sceptical voices and, for you, the most useful. Ink & Switch's
[Malleable software](https://www.inkandswitch.com/essay/malleable-software/) argues that people
need to adapt the tools they already have, and that "AI code generation alone does not address all
the barriers to malleability." Robin Sloan's
[home-cooked app](https://www.robinsloan.com/notes/home-cooked-app/) is about audience, not
lifespan: his family's messaging app has four users and has run for six years.

Geoffrey Litt is the one who matters most here. In ["Understanding is the new
bottleneck"](https://www.geoffreylitt.com/2026/07/02/understanding-is-the-new-bottleneck) he argues
that when agents write the code, people accrue cognitive debt, and the fix is instruments rather
than copilots. His examples start with a Prolog debugger Claude built for him in about a minute,
with a timeline he could scrub: "I still haven't looked at the UI code." "I caught a couple bugs
immediately." Then a migration command centre with the old and new sites side by side, and an
explainer with a quiz he has to pass before sharing the code. And [marimo
pair](https://marimo.io/blog/marimo-pair) puts Claude Code or Codex inside a live notebook kernel
where it reads values back exactly. Outside the terminal, that's the closest loop to fictty's I
found.

## The arguments underneath

**Aram:** So where do these camps actually disagree?

**Claude:** Strip away the products and five arguments are running.

The first is code or data. The page camp says models write HTML well, so let them, and the design
ceiling is the whole web. The data camp says a catalogue is safer, smaller and consistent. Each is
borrowing from the other: the page camp adds templates, and every "safe like data" format has grown
logic (A2UI's `and`/`or`/`not`, json-render's `$cond` and `$computed`, OpenUI's loops and queries).
So the honest form of the debate is about *where the logic lives*, not whether there is any. Which
is a question for you: fictty's rule is substitution only, no expressions. That's at the strict end
of the field. Is it a principle or a habit?

**Aram:** The repo's rule is that a pushed UI is data and behaviour belongs in the runtime. Why
does that matter more than expressiveness?

**Claude:** Because everyone else gave ground on it, it's a position to argue in public, not a
default to assume. The case for it is that a data value can be validated before it draws and can't
do anything the runtime doesn't allow, which matters when the screen binds commands on someone's
machine.

The second argument is who owns the interface's lifecycle: the vendor (a durable app), the person
(malleable software), or the model for a minute (generated UI). Anthropic made that product policy
with "permanent" Artifacts and "temporary" visuals. Most of the industry hasn't said it that
clearly.

The third is where generated UI lives: inline in the chat (a2tea, json-render's `ink-chat`,
Claude's visuals), beside it in a split (claude-canvas, Laura, fictty), or in a browser tab
(Artifacts, Lavish). Inline suits a one-shot picker; a split suits a screen that outlives a
message; a tab suits a thing to keep and share.

The fourth is whether the terminal is the display or only the control surface. OpenAI and Google
are moving the display to desktop apps. Anthropic is investing in both.

The fifth is whether any of it changes understanding, or only preference. Almost nobody is having
that one out loud, and I think it's the one that matters most.

**Aram:** That's the one I want to get to. But first the bigger frame.

## A UI used to be durable

**Aram:** We need to rethink UI. This ability to create UI rapidly is novel. We've always thought
of a UI as a durable thing, a costly thing to produce.

**Claude:** And that cost decided almost everything about software: which questions got a screen
(the ones enough people asked), which got a report (the ones a few people asked), and which got
nothing (yours, today, about this particular thing). Cost forced generality. A screen that served
many people needed settings, menus, onboarding and documentation, so you had to learn the tool
before it could help you learn anything else.

**Aram:** And when it costs one model turn?

**Claude:** Then an interface can be specific: the forty rows that matter to this question, not a
configurable view of all of them. It can be legible immediately, because it does one thing and
says so. It can be shaped like the data: a migration looks like a before and after, a queue looks
like a queue, a probability looks like something you can drag. And it can be gone afterwards, or
kept if it turned out to matter. That last part is what Sloan and the malleable camp get right:
some throwaway screens become tools.

There's historical evidence for why this matters. Distill's editors
[spent 50 or more hours](https://distill.pub/2021/distill-hiatus/) on single articles, including
"more than 50 hours of help with designing diagrams", before the journal stopped in 2021. Bret
Victor's [Explorable Explanations](https://worrydream.com/ExplorableExplanations/) (2011) was right
about reactive documents, and every example in it was built by hand. Interactive explanation didn't
fail because it didn't work. It failed because nobody could afford to make it.

**Aram:** That's what the name is about. *Fictile* means moldable, made of clay; *tty* is the
terminal. Ephemeral building blocks an agent assembles and takes apart again, recombined for the
next question. The identity we chose is glass panes: overlapping, tinted, like the layers of a
terminal UI.

**Claude:** Then the question I'd hold the name to is this. If the blocks are recombinable, who
recombines them, and what stays the same from one screen to the next?

**Aram:** The agent assembles them.

**Claude:** And the runtime is what stays the same. That's consistent with how the code is built:
the UI is one value the agent writes, and scrolling, focus, selection and data all live in the
runtime. It's also the clearest answer to the page camp. A generated page re-implements behaviour
every time. A runtime gives every throwaway screen the same keys and the same focus rules, so the
person doesn't have to learn each one.

So the question changes. It used to be "is this question common enough to deserve a screen?" Now
it's "would a screen help *right now*?" For a lot of what agents do (explaining a codebase,
reviewing a change, comparing options, watching a long job, asking for a decision), I think the
answer is yes. But I'd want to separate "helps" from "people like it", and that's your teaching
question.

## Learning by seeing

**Aram:** This might be a revolution in teaching. Up till now we're mostly still in the world of
text, reading what the agent writes. We're not tapping our visual sense, or our ability to learn by
building and interacting with UIs, to understand the data and the concepts better.

**Claude:** I want to push on "revolution", because this is where it's easiest to overclaim. Here's
what the evidence actually supports.

Words plus relevant pictures beat words alone, moderately. A
[2025 meta-analysis](https://doi.org/10.1016/j.edurev.2025.100730) of Richard Mayer's multimedia
learning research found an overall effect of g = 0.37 across 181 studies. Simulations and games
averaged about g = 0.41, with larger effects on inference and transfer than on recall. That's real
and replicated, and it's moderate, not magic. Effects have declined slightly in newer studies.

Decoration hurts. Among the largest reliable effects in the same analysis is *removing* seductive
details: interesting but irrelevant pictures and facts. Generated UIs are at high risk of exactly
that. They're impressive and often beside the point.

And "visual sense" needs care, because "visual learners" is a myth. Matching instruction to a
person's preferred modality [isn't supported](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4678182/).
Everybody benefits from relevant pictures with words. Nobody should claim more than that.

**Aram:** I'm not talking about learning styles. My point is that we're all still reading text,
when we could be seeing the thing and handling it.

**Claude:** That's the defensible version, and the evidence is on your side there. Then the next
trap is preference. Google's [generative UI study](https://arxiv.org/abs/2604.09577) found raters
preferred a generated page to Markdown 82.8% of the time on LMArena prompts and 90.5% on
information-seeking ones. But the results were pre-cached, so the generation time ("a minute or
more") was hidden, and the study measured liking, not understanding.

**Aram:** What about tutors? That's where this lands for most people.

**Claude:** The tutoring results say design matters more than medium.
[Bastani et al.](https://papers.ssrn.com/abstract=4895486) (PNAS 2025) found students who practised
with plain GPT-4 did 17% worse on exams than students who never had it; a version with guardrails
largely removed the harm. [Kestin et al.](https://pmc.ncbi.nlm.nih.gov/articles/PMC12179260/) (2025)
found a tutor built not to give answers beat an active-learning physics class over two weeks. The
[two-year Khanmigo trial](https://www.nber.org/papers/w35620) (NBER, August 2026, not peer
reviewed) found 0.04 SD, about the same as practice without AI, because students didn't engage:
39.4% of their messages were bare answers.

There's one encouraging result about generated material. Google's
[Learn Your Way](https://arxiv.org/pdf/2509.18664) turned textbook chapters into switchable text,
narrated slides, mind maps and quizzes. In a 60-student trial it beat a PDF reader by 11 points on
recall three to five days later. Small, Google-funded, promising.

And then the gap: we found no study comparing a text tutor with a tutor that builds an interactive
screen. Every tutoring trial above is text chat. So I can't tell you the revolution is real. Nobody
has tested it.

**Aram:** So what would you build, given that?

**Claude:** The evidence points to design rules more than to a medium. Make the person act: predict,
choose, answer, step. A screen the person only looks at is "GPT Base with pictures", and the one
result we have about that is bad. Nicky Case's explorables work because you guess before you see.
Keep it relevant, not decorative. Put the words on the thing, as callouts on the node rather than a
paragraph underneath (Mayer's contiguity principle). Bind it to the real data, which is
NotebookLM's lesson: a generated chart with made-up numbers teaches the wrong thing confidently.
And read back what the person did, so the agent responds to their actual misunderstanding, not a
guess at it.

**Aram:** So the part that matters is building and interacting, not looking. That was always the
point: using the UI to understand the data and the concepts.

**Claude:** Then we agree on more than I expected. I came into this treating the teaching angle as
the part of the pitch most likely to embarrass you, and I've moved a bit. The cost collapse is real,
the multimedia evidence is real if modest, and Litt's instruments show a version that isn't hype:
not "the agent draws a picture" but "the agent builds a small world you can push on, watches what
you do in it, and responds." That needs an interface that's cheap to make, bound to real data, and
readable back. Text alone can't do it. A page the model can't see can't do it either.

**Aram:** But you still don't think the terminal is the place for it?

**Claude:** Not for the general case. A terminal is a weak medium for rich explanation. If you have
a browser beside you and want a beautiful, animated explanation of a concept, a generated page or
Claude's inline visuals will look better than any grid of cells, and the labs own that. Where the
terminal has a case is narrower: instruments for understanding what an agent is doing, in the place
the agent works, bound to live state. Litt's debugger, as data.

**Aram:** So not an education product?

**Claude:** I'd say no. Teaching screens as recipes and a skill, and a claim you test, not a product
line.

## Adopt or build

**Aram:** Then we need a hard look at the landscape. Should we adopt another tool, or build our
own?

**Claude:** To answer that, strip fictty down to what nobody else already does for you. Drawing
cells, layout, pixels and text editing come from ratatui, caretline and resvg. Five jobs remain: the
UI as one serializable value, pushed whole or patched by id, with selection and scroll kept across
pushes; data sources the runtime runs itself, so data never passes through the model; behaviour at
frame rate; exact read-back of the frame as text and the state as JSON; and a long-lived process any
agent can attach to, from any harness, over SSH. The question is whether anything does most of
those.

**Aram:** Start with the standards. A2UI has Google behind it.

**Claude:** It's the best fit of the formats: surfaces, components by id, a data model by JSON
Pointer. But it has no place for data sources bound to commands, key maps, layers, pixels or theme
(v1.0 removed theming on purpose), and no notion of read-back. Everything that makes fictty fictty
would ride in extension fields no other renderer can draw. It's had three spec versions in ten
months, and 1.0 is still a release candidate. So speak it as an input, which the plan already has,
and don't make it the internal model unless 1.0 goes stable and grows bound data sources.

json-render has the most adoption in the camp, and its Ink renderer already does tables, forms,
charts and Markdown in a terminal. But it's a TypeScript library embedded in someone's Ink app: no
process, no socket, no patch from outside, no data sources, no read-back. And its expression
language breaks your "data, never code" rule. Take its patch dialect (RFC 6902) and its trick of
generating the agent's instructions from the schema.

**Aram:** And the TUI frameworks? Textual, OpenTUI?

**Claude:** Textual's company wound down in 2025, and moving would mean a Python rewrite: ideas
only. OpenTUI is the one serious alternative substrate, proven by opencode, but it would be a
rewrite onto a pre-1.0 API steered by one product, and ratatui already gives you more speed than
you need. Not now. Revisit if you find yourself rebuilding its Markdown, diff and code views.

**Aram:** What about Lavish? It came up last time.

**Claude:** Lavish has the right loop, already working: the agent writes, the person points at the
exact thing, the agent waits cheaply on a long poll, then revises. But it's a browser editor for
static HTML. Its data is frozen at write time, every revision regenerates the page (its own notes
say long pages cost output tokens and drift), and adopting it means giving up the terminal, live
data and state read-back, which is all of fictty's distinct ground. Interoperate: export to HTML
and hand off. Take its `watch` shape.

**Aram:** What about building on herdr or TUIOS? They have the distribution.

**Claude:** Neither has a UI model an agent can write, patch by id or read back as state. Building
fictty as their plugin means building fictty anyway, inside one frame's contract, and losing every
other frame. Be their content instead: report "waiting for you" through OSC 7501 and the older
OSC 9 and 777 notifications, call `herdr pane report-agent` inside herdr, and ship recipes for
their popups and floating panes. And build no frame features: no sessions, no detach, no
multi-agent status.

**Aram:** And Anthropic's own pieces?

**Claude:** This is where I'd be most careful, because they're the strongest substitutes.

Claude Code mods are the real threat, not as a library but as a replacement. For a Claude Code
user, a mod plus a good skill covers much of fictty's job with nothing to install, and Claude
writes and hot-reloads mods itself. Their limits are real (Claude Code only, code-first, redraws
capped at 30 a second, no read-back of what a mod drew for the model), but most people won't hit
them. If Anthropic ships a first-party "draw this JSON" mod with a read-back tool, fictty's Claude
Code story becomes "use that."

Claude Code Artifacts is the other. Hosting, versions, sharing, comments delivered to a watching
session, connector data, a shared database. If you're at a desk with a browser and a claude.ai
login, it covers a real share of the ten-minute screens fictty targets. What it doesn't do: no
pane, no local command as a data source, no frame read-back, one vendor and one account. So cede
share-and-keep entirely, and treat its comment-to-agent loop as the minimum bar for `watch`.

**Aram:** So after all that: build or not?

**Claude:** Build the runtime, and only the runtime. Nothing in the landscape does the five jobs
together. Laura has the loop for documents. The format renderers have the values without the loop.
The frames have text read-back without state. Artifacts have the return path without the terminal
or local data. That position is real, narrow, and probably temporary.

**Aram:** Temporary?

**Claude:** The lead over json-render's Ink renderer and the A2UI terminal renderers is thin. A
socket and a read-back verb would be a weekend's work for a motivated maintainer, with far more
adoption behind it. fictty's lead is in the quality and speed of the loop, not in the idea. And
demand is untested. Before building much more, I'd put fictty and an Artifact side by side in front
of five people supervising a real agent task, and watch which one they reach for.

## Where fictty fits

**Aram:** Give me the position in one sentence, then.

**Claude:**

> **fictty is the disposable screen an agent puts beside its work in the terminal: written once as
> data, changed by id, driven by the person at frame rate, fed by your commands instead of the
> model, and read back exactly, including what the person did.**

The defence, claim by claim.

Data, not code, is the right representation for a screen that lives ten minutes. A page is
regenerated on every revision. A fictty screen is a small JSON value; the agent changes one node by
id and the rest stays put, including the person's selection and scroll. A data value can be
validated before it draws, so "valid" means "renders", which is the class of bug Google's
generative UI needs post-processors for.

Semantic read-back, not just text. fictty returns state: which row is selected, what was typed,
which callout is showing, by node id. For a teaching screen that's the difference between "the
agent can see the screen" and "the agent knows what you predicted."

Data skips the model. Sources are commands and streams bound to nodes. The agent writes the screen
once and never relays a number, so a live dashboard costs no tokens per update and the numbers are
the real ones. Almost nothing else does this: in the data camp, values flow through the agent; in
the page camp, through a vendor's proxy. OpenUI's `Query` is the lone cousin.

Behaviour at frame rate. Scrolling, selection, focus and mouse run in the runtime, so a person
navigating never waits on a model.
[Imagine with Claude](https://www.anthropic.com/news/claude-sonnet-4-5), Anthropic's five-day demo
where every click generated the next screen, is the clearest picture of why the model shouldn't be
in that loop.

Any agent, in the pane, over SSH. fictty isn't tied to one harness or one account. It runs in a
herdr or TUIOS pane on a remote box.

**Aram:** And where it isn't the answer?

**Claude:** I'd rather say it than have someone discover it.

- **Anything you want to keep or share.** Use HTML: Claude Code Artifacts, Lavish, a page in the
  repo.
- **Rich general explanation** with a browser beside you. A grid of cells, with pixels on some
  terminals, won't beat a page.
- **Apps people use every day.** Ptacek is right. Build a native or web app.
- **A one-shot question inside your harness.** If you only use Claude Code and want a picker,
  `AskUserQuestion` or a mod is already there, and inline UI in the chat will cover more of this
  over time.
- **Managing agents.** herdr, TUIOS, tmux and zellij do that. fictty runs inside their panes.
- **Accessibility, today.** Cell-based terminal UI is bad for screen readers. A UI that is data
  *could* render a linear, semantic version; fictty doesn't yet.

**Aram:** What would prove us wrong?

**Claude:** Five things, and I'd watch for all of them.

- json-render or A2UI ships an attachable terminal host with a socket and read-back.
- Anthropic ships a first-party "draw this JSON" mod with a read-back tool. For Claude Code users,
  that covers most of the job.
- herdr adds declarative plugin UI, and the content layer gets claimed from above by the tool with
  the distribution.
- Agents leave the terminal. If remote, long-running agents end up supervised from desktop apps, a
  terminal runtime for their screens goes with them.
- The learning claim doesn't hold up. If a small study (a text tutor against a text tutor plus an
  interactive screen, on understanding an unfamiliar module) shows no difference, the teaching
  story shrinks to preference. fictty's state read-back makes that study cheap to run.

## What we came away with

**Aram:** Then what changes in the plan?

**Claude:** Some of this is already there: `watch`, the command policy, the MCP server, JSON Patch
and the A2UI adapter are all in [the plan](../PLAN.md). What this pass adds are proposals, not
commitments:

- make the instrument (a step-through, a scrubber, a side-by-side, bound to live state) the
  headline use case: Litt's examples, as data;
- ship `watch` with a question or checkpoint component, so a screen can ask for a prediction and
  the agent gets the answer;
- report to the frames: OSC 7501 and OSC 9/777 when a screen waits, `herdr pane report-agent`, and
  recipes for herdr plugins, `tuios popup`, tmux modal panes and zellij floating panes;
- bridge into the harnesses with a Claude Code mod that renders a fictty value in a pane, with a
  `screen` tool for read-back, then opencode and pi;
- export to HTML, so a screen worth keeping can graduate to an Artifact or Lavish;
- measure time to first useful screen against generate-a-page, the number Google left out;
- and run the two cheap tests: five people choosing between fictty and an Artifact, and the text
  tutor against the tutor with a screen.

**Aram:** And if the tests go the wrong way?

**Claude:** Then we'll have learned it in a week instead of a year, and fictty becomes a smaller
thing: supervision and live data in a terminal, without the teaching claim. That's still a real
tool. It just isn't the bigger idea, and we should find out which one it is before we build much
more.

The full reasoning, option by option, is in [adopt-or-build](../design/adopt-or-build.md).

## Sources

The dossiers behind this conversation, with every source, are in
[docs/research/landscape](../research/landscape/). The main ones:

**Frame and harness.** [herdr](https://github.com/herdrdev/herdr) and its
[socket API](https://herdr.dev/docs/socket-api/); [TUIOS](https://github.com/Gaurav-Gosain/tuios);
[zellij plugin UI](https://zellij.dev/documentation/plugin-ui-rendering.html);
[tmux CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES);
Superlogical, [Program Status Protocol (OSC 7501)](https://www.superlogical.com/rex/docs/build/program-status);
[Vibe Kanban shutdown](https://www.vibekanban.com/blog/shutdown);
[Superset Pages](https://superset.sh/blog/review-agent-work-with-pages); [cmux](https://cmux.com/blog/zen-of-cmux);
[Claude Code mods](https://code.claude.com/docs/en/plugins/mods/overview) and
[fullscreen rendering](https://code.claude.com/docs/en/fullscreen);
[Codex CLI](https://github.com/openai/codex); [opencode TUI plugins](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/specs/tui-plugins.md);
[pi](https://github.com/earendil-works/pi);
Google, [Transitioning Gemini CLI to Antigravity CLI](https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/).

**Pages.** Thariq Shihipar, [HTML effectiveness](https://thariqs.github.io/html-effectiveness/);
[Claude Code Artifacts](https://code.claude.com/docs/en/artifacts);
Anthropic, [Claude builds visuals](https://claude.com/blog/claude-builds-visuals);
[Lavish](https://github.com/kunchenguid/lavish-axi) and [axi.md](https://axi.md);
[MCP Apps (SEP-1865)](https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp);
[Cursor Canvas](https://cursor.com/docs/agent/tools/canvas);
OpenAI, [model release notes](https://help.openai.com/en/articles/9624314-model-release-notes);
[Imagine with Claude](https://www.anthropic.com/news/claude-sonnet-4-5).

**Data.** [A2UI](https://github.com/a2ui-project/a2ui) and its
[v1.0 protocol](https://github.com/a2ui-project/a2ui/blob/main/specification/v1_0/docs/a2ui_protocol.md);
[json-render](https://github.com/vercel-labs/json-render); [OpenUI](https://github.com/thesysdev/openui);
[AG-UI](https://github.com/ag-ui-protocol/ag-ui); [ChatKit widgets](https://developers.openai.com/api/docs/guides/chatkit-widgets);
[Adaptive Cards](https://adaptivecards.io); [Tambo shutdown](https://tambo.co/blog/posts/tambo-is-shutting-down).

**Terminal runtimes and drivers.** [claude-canvas](https://github.com/dvdsgl/claude-canvas);
[Laura](https://github.com/thomaseleff/laura-tui); [a2ui crate](https://github.com/Liangdi/a2ui);
[a2ui-ink](https://github.com/kokoro-ele/a2ui-ink); [a2tea](https://github.com/charmbracelet/a2tea);
[Raxol](https://github.com/DROOdotFOO/raxol); [Melker](https://github.com/wistrand/melker);
ConductorOne, [agent-tui](https://www.c1.ai/labs/research/agent-tui-structured-terminal-access-for-ai-agents);
[tui-test](https://github.com/microsoft/tui-test); [agent-tty](https://github.com/coder/agent-tty);
[tui-use](https://github.com/onesuper/tui-use).

**Ideas.** Ink & Switch, [Malleable software](https://www.inkandswitch.com/essay/malleable-software/);
Robin Sloan, [An app can be a home-cooked meal](https://www.robinsloan.com/notes/home-cooked-app/);
Maggie Appleton, [Home-cooked software](https://maggieappleton.com/home-cooked-software);
Geoffrey Litt, [Understanding is the new bottleneck](https://www.geoffreylitt.com/2026/07/02/understanding-is-the-new-bottleneck),
[Enough AI copilots! We need AI HUDs](https://www.geoffreylitt.com/2025/07/27/enough-ai-copilots-we-need-ai-huds) and
[an AI-generated debugger](https://www.geoffreylitt.com/2024/12/22/making-programming-more-fun-with-an-ai-generated-debugger);
Bret Victor, [Explorable Explanations](https://worrydream.com/ExplorableExplanations/);
[Nicky Case](https://ncase.me); Distill, [Distill hiatus](https://distill.pub/2021/distill-hiatus/);
[marimo pair](https://marimo.io/blog/marimo-pair); [Observable Notebooks 2.0](https://observablehq.com/notebooks/2/);
Thomas Ptacek, [Stop Making TUIs](https://sockpuppet.org/blog/2026/08/20/stop-making-tuis/) and the
[HN thread](https://news.ycombinator.com/item?id=49384210);
Jane Street, [strace-ui, Bonsai_term, and the TUI renaissance](https://blog.janestreet.com/strace-ui-bonsai-term-and-the-tui-renaissance/).

**Learning evidence.** Cromley and Chen, [A meta-analysis of Mayer's multimedia learning research](https://doi.org/10.1016/j.edurev.2025.100730) (2025);
Noetel et al., [Multimedia design for learning](https://doi.org/10.3102/00346543211052329) (2022);
Carl Hendrick, [Which multimedia strategies actually improve learning?](https://carlhendrick.substack.com/p/which-multimedia-strategies-actually);
Newton, [The learning styles myth is thriving in higher education](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4678182/);
Google Research, [Generative UI](https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/) and
[the paper](https://arxiv.org/abs/2604.09577); [Learn Your Way](https://arxiv.org/pdf/2509.18664);
Bastani et al., [Generative AI without guardrails can harm learning](https://papers.ssrn.com/abstract=4895486);
Kestin et al., [AI tutoring outperforms in-class active learning](https://pmc.ncbi.nlm.nih.gov/articles/PMC12179260/);
Oreopoulos and Low, [One Click Away](https://www.nber.org/papers/w35620) (Khanmigo trial).
