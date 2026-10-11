---
name: blog-post
description: Write a new post for the fictty blog (fictty.com/blog), a conversation between Aram and Tilde about a milestone. Use when asked to write up, blog, or post about a stretch of work on fictty.
---

# Writing a fictty blog post

A post is a milestone written as a conversation between **Aram** ([@brancusi](https://github.com/brancusi)),
who builds fictty, and **Tilde**, the name Claude goes by on this project (a tilde is home in a
terminal). Each post does three jobs at once:

- **For readers:** a fun, curious account of exploring and building something useful. It should
  never feel like a wall of text.
- **For Aram:** a knowledge store to look back on, a review of the milestone: what we tried, what
  we decided and why, what we learned.
- **For future agents:** a record of where the project went and why, so they don't relitigate
  settled questions or repeat dead ends.

## Voice

- **Snippets, not a transcript.** Readers want the moments that changed something: the doubt, the
  finding, the insight, the decision, the surprise. Skip the play-by-play (how a demo was launched,
  which window opened, each request in order, every reply). A few quoted lines per section, with
  short narration between them, carry the story; a typical post has a dozen or so turns, not
  fifty. If a turn doesn't change what the reader knows or feels, cut it.
- **Conversational, positive, curious.** The tone is exploration: questions, tries, surprises,
  small wins, honest dead ends. Enjoy the problem.
- **Keep the conversation true.** Aram's lines are Aram's own words from the session. Fix spelling,
  grammar and punctuation, cut filler ("so, um, like"), and trim a quote to the part that
  matters, but never change what was asked or meant, and never invent a line for Aram. Tilde's lines are condensed from what was actually
  answered; don't add claims that weren't made, and correct anything later found wrong in the
  narration rather than rewriting history.
- **Short turns.** One idea per turn. If an answer runs past two paragraphs, turn the rest into a
  graphic, a table or a short list.
- **Show, then tell.** Lead with what happened (a demo, a number, a screenshot), then what it means.
- **Plain words.** Follow the house style: specific nouns, no hype, no jargon that a curious
  developer wouldn't know without a gloss.
- **Not overwhelming.** Aim for a 5–8 minute read (about 1,200–1,600 words). Leave detail to the design docs and link them.

## Where the material comes from

1. The session transcript (`~/.claude/projects/<project>/<session>.jsonl`). Pull Aram's messages
   verbatim and Tilde's longer answers; a short script that prints `type == "user"` string contents
   and assistant `text` blocks is enough.
2. The design docs the work produced (`docs/design/*.md`, `docs/PLAN.md`), for decisions and
   numbers. Quote numbers exactly as measured, with when and how.
3. The running code: screenshots come from `fictty render`, never mockups.

## The file

- `docs/blog/YYYY-MM-slug.md`. The site drops the date prefix from the URL (`/blog/slug/`).
- Front matter:

  ```yaml
  ---
  title: "Short and specific: a subtitle if it helps"
  description: One or two sentences, used as the lede and the social card.
  date: 2026-10-11T09:00:00Z
  kicker: milestone        # or research, design, release
  ---
  ```

- Open with an italic paragraph: who's talking (Aram, and Tilde as the name Claude goes by here),
  where the conversation comes from (which sessions, which dates), that Aram's lines are lightly
  edited, and links to the earlier posts it follows.
- `## ` headings for each stop on the journey; the site builds the table of contents from them.
- Speaker turns are paragraphs that start with `**Aram:**` or `**Tilde:**`. A speaker keeps the
  floor until the next speaker, a heading or a rule, so narration placed after a turn belongs to
  that speaker. Put figures inside the turn that talks about them, and put narration either right
  after a heading or inside a turn, phrased as that speaker's.
- End with what's next, linking [the plan](../PLAN.md), and a short closing exchange.

## Graphics

Use them generously: one every screen or two of reading. All are plain HTML in the Markdown, styled
by `web/src/styles/prose.css`, so the `.md` stays the source of truth.

- **Screenshots of fictty** go in `docs/blog/img/<slug>/` as PNG. Make them with
  `fictty render ui.json --size 120x36 --ansi --themes … --theme …` turned into HTML, captured in a
  browser and cropped to the frame. Use made-up data only; strip anything personal from a store
  before rendering it. Inside a figure, write the image as Markdown (with blank lines around it) so
  the site processes it:

  ```html
  <figure class="shot">

  ![Alt text that describes the screen](img/slug/notes.png)

  <figcaption><b>What it is.</b> What to notice.</figcaption>
  </figure>
  ```

- **Three side by side:** `<figure class="trio"><div class="row">` with three Markdown images,
  each its own paragraph, then `</div>` and a caption.
- **Bar charts:** `--v` is the bar's length, 0 to 100, scaled to the largest value; `--c` a glass
  tint (`--glass-cyan`, `--glass-violet`, `--glass-amber`, `--glass-rose`, `--glass-smoke`).

  ```html
  <figure class="bars"><div class="rows">
  <div class="group">Optional group label</div>
  <div class="bar" style="--v: 100; --c: var(--glass-cyan)"><span>label</span><i></i><b>22.9k</b></div>
  </div><figcaption><b>Title.</b> Source and date.</figcaption></figure>
  ```

- **A journey:** `<ol class="route">` with one `<li style="--c: var(--glass-…)"><b>Stop.</b> What happened.</li>` each.
- **Layers** (a stack, owners, a pipeline): `<figure class="stack"><div class="layers">` with
  `<div class="layer" style="--c: …"><b>Name</b><span>What it does</span></div>` each.
- **Tables** in Markdown for comparisons; the site wraps them to scroll on phones.
- Links inside raw HTML aren't rewritten to site routes, so put links in Markdown text, not in
  captions.

## The Tokens panel

Every post carries a **Tokens** panel under its date: one line (tokens through the model, tokens
written, the models used) that opens into a table by phase. It's the `Tokens` component
(`web/src/components/Tokens.astro`), fed by a `tokens:` block in the front matter. Make the block
with the script, from the session transcript(s) the post covers:

```sh
scripts/blog-tokens.py ~/.claude/projects/<project>/<session>.jsonl \
  --phase "first words of the message that started phase 1=Phase one" \
  --phase "first words of the message that started phase 2=Phase two" \
  --end "first words of the first message after the post's work"
```

- Phases follow the post's own sections, named the way the post names them; 5–9 is plenty.
- A phase starts at Aram's message (typed, or queued mid-turn) that begins with the given words.
  Subagents and workflows in the session's folder that ran inside the window count too.
- Paste the printed YAML into the front matter. Add `note:` for anything the count leaves out
  (context compactions aren't in the transcript; work in another session needs its file too, so
  run the script on each and add the phases).
- The panel works out the totals, the share bars and model names (`claude-opus-5-5` shows as
  Claude Opus 5.5). Don't add a cost: prices change, and the panel is about scope.

## Let readers try it

If the post shows something runnable, give readers one line to run it, the way the stack post
does:

```sh
curl -fsSL https://fictty.com/try.sh | sh
```

`web/public/try.sh` fetches fictty into `~/.fictty`, uses a release binary or builds from source,
and runs the showcase. When a post is about a different demo, extend `try.sh` (a demo name as an
argument, defaulting to the showcase) rather than writing a second script, test it from an empty
`FICTTY_HOME` with `FICTTY_NO_RUN=1`, and say in the post what it needs (Rust, until releases
exist; python3 for helper processes).

## Links

Link docs relatively (`../design/state-and-theme.md`, `2026-10-why-the-terminal.md`); the site
rewrites them to its own pages. A doc a post links to must be a designed page on fictty.com, not
raw Markdown on GitHub: if it isn't routed yet, add it like `scope` (a page in `web/src/pages/`,
the `pages` collection in `content.config.ts`, `route()` in `rehype-links.mjs`, `DesignDoc.astro`,
the sitemap and `scripts/links.mjs`).

## Check, then publish

1. `cd web && npm run build && npm run links`: the build passes and every link resolves. Open the
   Tokens panel and check its numbers against the script's output.
2. Read it as a stranger would: is every section a moment that matters? Cut the rest.
3. Look at it: serve `web/dist` and screenshot the post at desktop and phone widths. Check that
   every speaker turn holds the right paragraphs, figures load, and charts read in light and dark.
4. Fact-check every number and quote against the transcript and the docs.
5. `scripts/preflight.sh --staged` before committing (stage explicit paths).
6. **Ask Aram before publishing.** Pushing to `main` puts the Markdown on GitHub; `npm run deploy`
   in `web/` puts it on fictty.com. Both are public, so neither happens without Aram's yes.
