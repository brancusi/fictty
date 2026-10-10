# Claude Code Artifacts

- **Maker:** Anthropic
- **URL:** https://code.claude.com/docs/en/artifacts
- **Status (10 October 2026):** shipping, still labelled beta in parts. The `Artifact` tool first
  appears in Claude Code 2.1.172 (dated 13 June 2026 by a third-party release tracker); the public
  beta was announced on 18 June 2026 for Team and Enterprise, and the docs now list Pro, Max, Team
  and Enterprise. Features have landed every few weeks since: `list` and update-by-URL (2.1.203,
  July), `/artifacts` (2.1.208), MCP connector calls from pages (2.1.209), comments (2.1.221),
  Claude replying to comments on its own (2.1.228), Slides/Design/Docs templates (2.1.265),
  `remote-control` sessions (2.1.281). Closed source, part of a paid product.

## What it is

A tool inside Claude Code that publishes a page the agent wrote (HTML, or Markdown rendered as a
styled page) to a private URL on claude.ai. Republishing to the same URL updates the page in place
for everyone viewing it, and each publish becomes a version. The person shares it from the page
header: inside their organisation, as viewer or editor, or publicly.

## The problem it's solving

Terminal text is the wrong medium for a lot of what a coding agent produces: annotated diffs,
comparisons of several options, dashboards, timelines of a long investigation. Pasting that into
Slack or reading it as Markdown in a scrollback loses the structure. Artifacts give the agent a
cheap way to put a *page* in front of a person, and give the person a link they can pass on.

## Its path / bet

The agent writes code (one self-contained HTML page) and Anthropic hosts it. The bet is that a
model writing HTML is now good enough, and a browser close enough to hand, that "make it a page"
should be the default for anything spatial, and that the page should keep living after the session
(versions, sharing, comments, live data at view time).

## How it works

- Claude writes the page to a file (by default a temp directory outside the project) and calls the
  `Artifact` tool. The first publish of a new artifact can need approval depending on mode;
  republishing an approved one doesn't prompt.
- The page is wrapped in a document shell and served from a sandboxed `*.claudeusercontent.com`
  origin under a strict CSP: scripts only from five CDNs, fonts only from Google Fonts, no external
  images, `fetch` only to its own origin. 16 MiB rendered limit. No backend.
- **Live data at view time:** a page can declare claude.ai MCP connectors it may call. Each viewer's
  own connector account makes the call; claude.ai proxies it; the page refreshes on load, on an
  interval or on a control. Local `.mcp.json` servers can't be called by a published page.
- **Runtime capabilities.** In our own Claude Code session (10 October 2026) the `Artifact` tool
  describes further page capabilities: shared state across viewers, a small per-artifact database
  that Claude reads and writes with an `ArtifactData` tool, knowing who is viewing, the page asking
  Claude a question, file uploads and downloads, and multi-file publishes. The public docs cover
  connectors and downloads; we saw the rest only in the tool's own description, so treat the
  details as subject to change.
- **The feedback loop:** viewers comment on the page; an editor can "Send to Claude" or `@claude` a
  thread, and a session that published the artifact watches it, so the comment arrives in the
  session and Claude can reply or edit the page without being asked (rate-limited to 60 a hour).
- **Read-back:** Claude can read its own artifact's raw HTML, an isolated summary of someone else's,
  the comment threads, and the database rows. It cannot read the rendered page as the viewer sees
  it (scroll position, what's selected, what a chart actually drew).

## Strengths

- Zero setup for anyone already in Claude Code with a claude.ai login; a URL is the best
  distribution there is.
- The design ceiling of the web, and a design skill that makes pages look deliberate by default.
- Pages outlive the session: versions, sharing, editors, an audit log, retention controls.
- It now has a real loop: publish, comment, Claude watches and replies, republish. With connector
  calls and a shared database, a page can show fresh data and hold state without the model in the
  path.
- It is improving fast and is backed by the lab whose agent fictty's users mostly run.

## Weaknesses / limits

- Tied to one vendor and one account type: not on Bedrock, Vertex or Foundry, not with an API key,
  off in `-p` headless mode, the Agent SDK and MCP-server contexts, and unavailable under ZDR,
  HIPAA or CMEK.
- Live data only through claude.ai connectors. A local command, a log tail or a process on a remote
  box can't feed a published page directly; the agent has to relay it (republish) or wrap it in a
  hosted connector.
- Every change of layout costs a model turn and output tokens; the docs warn a styled page costs
  more tokens than terminal text.
- No exact read-back of what the viewer sees. Claude reads source, comments and rows, not the frame.
- A browser away from the terminal. Over SSH, a person needs a browser on another machine.
- Interaction is whatever JavaScript the model wrote. Keyboard behaviour, focus and selection are
  re-implemented per page, with no shared runtime guaranteeing them.

## Relation to fictty

**Complement, with real overlap.** `docs/design/scope.md` already cedes "reports to keep or share"
to Artifacts. That line has moved: with comments-to-Claude, watching, connector data and a shared
database, Artifacts now covers a good share of the ten-minute screens fictty's scope lists (a
walkthrough, a review queue, a dashboard, a picker, a form), for anyone with a browser at hand and
a claude.ai account. What it does not cover: a screen bound to local commands and streams at
hundreds of updates a second, keyboard-speed navigation in the terminal the agent runs in, exact
read-back of the frame, sessions over SSH with no browser, and any agent other than Claude Code.

## Could fictty adopt it instead of building?

For the share-and-keep case, yes, and it should: fictty shouldn't grow a hosting, sharing or
commenting story. For the in-terminal case, no. Artifacts can't run in a pane, can't bind a local
command as a source, and can't be driven or read back as a frame. The honest risk is that many
users never need the in-terminal case because they always have a browser beside Claude Code. That
is a demand question fictty has to answer with users, not with architecture.

## What fictty should take from it

- **Export, don't compete:** `fictty export --html` (or an Artifact-friendly Markdown/HTML render of
  the current state) so a screen can graduate to a shared page when the person wants to keep it.
- **The comment-to-agent loop is the bar.** Watching a surface and delivering a person's response
  to the session without polling by hand is now table stakes; fictty's planned `watch` must be at
  least this good, including rate limits and a way to stop it.
- **Declared capabilities.** Pages declare connectors and downloads up front and the host enforces
  it. fictty's data sources (commands that run on the person's machine) deserve the same: a
  declared, reviewable list, not arbitrary `cmd` arrays accepted silently from any push.
- **Versions by default.** Every publish is a version. fictty's planned history should make "show
  me version 3" as cheap.

## Sources

- Claude Code docs, [Share session output as artifacts](https://code.claude.com/docs/en/artifacts) (fetched 10 October 2026)
- Claude help centre, [What are artifacts and how do I use them?](https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them)
- [2026-06-18 Artifacts come to Claude Code](https://www.dsebastien.net/2026-06-18-artifacts-come-to-claude-code/) (third-party)
- Tech Dev Notes, [Claude Code 2.1.172](https://techdevnotes.com/releases/claude-code/2.1.172) and [2.1.203](https://techdevnotes.com/releases/claude-code/2.1.203) (third-party release tracker)
- [AI Weekly on the beta](https://aiweekly.co/node/3192) (third-party)
- The `Artifact`, `ArtifactData` and `ArtifactComments` tool descriptions as exposed to Claude Code on 10 October 2026 (observed directly; not a public document)

Not verified: the exact first-release date (13 versus 18 June), and which of the runtime
capabilities seen in the tool description are generally available rather than per-account.
