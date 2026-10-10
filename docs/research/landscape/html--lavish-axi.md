# Lavish (lavish-axi) and the AXI principles

- **Maker:** Kun Chen (@kunchenguid)
- **URL:** https://github.com/kunchenguid/lavish-axi ; https://axi.md (repo https://github.com/kunchenguid/axi)
- **Status (10 October 2026):** active open-source project. MIT, JavaScript (TypeScript-checked),
  about 4,050 stars and 370 forks, created 11 May 2026, last push 10 October 2026. Released
  through release-please: v0.1.85 on 9 October 2026, v0.1.84 on 7 October, v0.1.83 on 6 October,
  so roughly a release a day. GitHub's Releases tab is empty; the CHANGELOG is the record.

## What it is

"HTML is the new markdown. Lavish is the new editor for your HTML artifacts." A local CLI plus a
browser editor for HTML files that an agent writes. The agent opens a file in a review session; the
person reads it, annotates elements and text, edits Mermaid diagrams as whiteboards, attaches
images and queues notes; the agent collects the feedback with a long poll and revises the file,
which live-reloads.

## The problem it's solving

Agent-written HTML is a one-way broadcast. The person sees a page but has no precise way to point
at a part of it and say "this is wrong", and the agent has no cheap way to wait for that. Lavish
closes the loop: targeted feedback in, cheap waiting out.

## Its path / bet

Accept HTML as the agent's output format, and build the *review loop* around it, locally, as a
tool any agent can drive from a shell. It's an "AXI", Chen's term for an Agent eXperience
Interface: a CLI designed for agents first.

## How it works

- `lavish-axi <file.html>` starts (or reuses) a detached local server (port 4387 by default) and
  opens a review session keyed by file path. The artifact runs in a sandboxed iframe; Lavish
  injects a small SDK for annotations, snapshots and layout checks without changing the saved file.
- `lavish-axi poll <file>` long-polls until feedback arrives, the session ends or every reviewer
  disconnects. Feedback carries the selected text, range anchors and CSS locators (table clicks
  include row and column names); images arrive as local file paths. Per the changelog, poll output
  also carries a DOM snapshot and browser layout warnings.
- `reply` posts an agent reply (a Markdown subset) into the chat panel; `poll --agent-reply` replies
  and polls again. Delivery receipts show whether a note was seen, worked on or answered.
- `export` makes a standalone copy; `share` publishes to a third-party host (ht-ml.app), with
  password-protected private links since 0.1.60; Tailscale binding lets a phone on the tailnet open
  a session.
- "Playbooks" and a design skill ship with the CLI to steer how the agent writes pages (code
  walkthroughs, explanations of existing systems, and so on).
- Installs as an agent skill (`npx skills add kunchenguid/lavish-axi --skill lavish`) or with hooks.

**AXI's ten principles** (axi.md): token-efficient output (TOON, claimed about 40% fewer tokens than
JSON); minimal default schemas; truncation with a size hint and `--full`; pre-computed aggregates;
definitive empty states; structured errors, idempotent mutations, no interactive prompts; ambient
context through session hooks; content first (a bare command shows live data); contextual
`help[]` next steps; concise per-command `--help`. The site reports benchmarks (Claude Sonnet 4.6
as agent and judge; 490 browser runs and 425 GitHub runs) in which AXI tools matched or beat MCP
equivalents at a fraction of the cost (for example $0.050 versus $0.101 to $0.148 per GitHub task).
The author notes the limits: public sites, one model, read-only tasks, an LLM judge.

## Strengths

- The loop is right: agent writes, person points at the exact thing, agent waits cheaply, agent
  revises. It's the nearest thing in the browser to what fictty wants in the terminal.
- Precise, structured feedback (locators, ranges, row and column names) rather than "the third
  chart looks off".
- Local-first, no account, any agent that can run a shell command.
- Layout warnings reported back to the agent: a partial answer to "the agent can't see what it drew".
- Shipping daily, with a clear voice and a growing user base.

## Weaknesses / limits

- The artifact is still a static file the model writes. Data in it is frozen at write time unless
  the agent rewrites the file; there is no binding to live sources.
- Read-back is a DOM snapshot plus annotations, not the frame the person sees.
- Needs a browser and Node. Over SSH it relies on Tailscale or a share link.
- Every revision is a regeneration of HTML; long pages cost output tokens and drift.
- Single maintainer; the format and commands are young and moving.

## Relation to fictty

**Inspire, edging into compete.** It solves the same loop for the same users (people supervising a
coding agent), in the browser. If people are happy to review in a browser tab, Lavish plus Claude
Code Artifacts covers much of fictty's "review queue" and "walkthrough" cases. fictty's distinct
ground is live data bound without the model, keyboard-speed interaction, exact frame read-back, and
running in the pane over SSH.

## Could fictty adopt it instead of building?

Not for fictty's core: it's a browser HTML editor, not a runtime. But for the "this screen became a
document" path, yes: rather than build HTML review, fictty should be able to export a screen to a
self-contained HTML file and hand it to Lavish or an Artifact. Interop costs little; rebuilding
annotation and whiteboards would be a waste.

## What fictty should take from it

- **`watch` as a long poll that returns structured feedback,** with the same three exits Lavish has
  (response, session ended, everyone gone) and an "ended" state that tells the agent to stop. This
  moves `watch` to the top of the plan, as the first landscape pass already said.
- **Point-at-it feedback.** Let the person mark a node, row or cell and attach a note; deliver it
  with the node id and row key. fictty can be more exact than CSS locators because every node has an
  id.
- **Delivery receipts** (seen, working, answered) shown in the screen itself.
- **Layout warnings to the agent:** fictty knows when text is truncated, a column is squeezed or a
  node is off-screen. Report it in `get --stats` or a `lint` verb.
- **Follow AXI for the CLI:** content-first bare commands, definitive empty states, `help[]` next
  steps, compact output. Consider TOON or a compact text form for `get --state`.
- **Playbooks:** ship recipes and guidance as an agent skill, installed the same way.

## Sources

- [kunchenguid/lavish-axi](https://github.com/kunchenguid/lavish-axi), its [CHANGELOG](https://github.com/kunchenguid/lavish-axi/blob/main/CHANGELOG.md) and the [GitHub API record](https://api.github.com/repos/kunchenguid/lavish-axi) (fetched 10 October 2026)
- Kun Chen, [AXI: Agent eXperience Interface](https://axi.md) and [kunchenguid/axi](https://github.com/kunchenguid/axi)

Not verified: the AXI benchmark numbers (self-reported, single model), and the exact contents of the
DOM snapshot in poll output (inferred from changelog entries, not tested).
