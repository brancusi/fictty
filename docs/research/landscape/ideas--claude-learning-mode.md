# Claude learning mode and Claude Code's Learning and Explanatory styles

- **Maker:** Anthropic
- **URL:** https://code.claude.com/docs/en/output-styles
- **Status (10 October 2026):** learning mode launched in Claude for Education in April 2025 and
  was opened to all Claude users in August 2025. Claude Code gained two output styles at the same
  time, **Explanatory** and **Learning**; a third built-in style, Proactive, was added later. The
  `/output-style` command was deprecated in v2.1.73 and removed in v2.1.91 (per a third-party
  source); styles are now chosen in `/config`.

## What it is

- **Learning mode (Claude apps):** Socratic tutoring: Claude asks guiding questions and works
  through problems with the person instead of giving the answer.
- **Explanatory (Claude Code):** adds short "Insights" about why the code is the way it is while
  doing the task.
- **Learning (Claude Code):** Claude stops and asks the developer to write small, strategic pieces
  of code, marked `TODO(human)` in the source.

## The problem it's solving

Students and developers who let the model do the work learn less (see the Bastani study in the
AI-tutoring dossier). Anthropic's answer is to change the model's conversational behaviour.

## Its path / bet

Behaviour through prompting, in the same medium: text in chat, text in the terminal, and the code
itself. No new interface.

## How it works (concretely)

Output styles replace part of Claude Code's system prompt. Learning inserts `TODO(human)` markers
and waits for the developer to fill them in; Explanatory adds insight blocks in the transcript.

## Strengths

- Ships where developers already work (the terminal), with zero setup.
- `TODO(human)` is a clever device: the exercise lives in the real code, and the agent can check it.
- Grounded in a real finding (unguarded AI help reduces learning).

## Weaknesses / limits

- Text only. Insights scroll away in the transcript; there's nothing to look at or manipulate.
- No measurement published that we found of whether developers learn more with these styles.
- Many developers will switch it off when they're in a hurry, which is most of the time.

## Relation to fictty

**Complement.** This is the closest existing thing to "learning inside the coding agent," and it's
all text. A fictty screen is the visual half it's missing: the Explanatory insight as a
walkthrough with callouts on the real code, the `TODO(human)` as a screen that shows the exercise,
the tests and the result, and reads back when it passes. If Anthropic adds visuals to Claude Code
itself (they already have inline visuals in chat), this complement becomes a competitor.

## Could fictty adopt it instead of building?

Nothing to adopt: it's a prompt inside a closed product. fictty could ship an output style or skill
that tells Claude Code when to put up a teaching screen.

## What fictty should take from it

- **Ship a "teach" skill for Claude Code** that pairs with the Explanatory and Learning styles: when
  Claude explains, it pushes a walkthrough screen; when it leaves a `TODO(human)`, it pushes a
  screen with the task, the test status bound to a live `cargo test` (or similar) source, and waits
  with `watch`.
- **Keep the exercise in the real artefact.** `TODO(human)` works because it's in the code. A
  fictty teaching screen should bind to the real repo state, not a copy.

## Sources

- Claude Code docs, [Output styles](https://code.claude.com/docs/en/output-styles)
- AlternativeTo, [Claude's learning mode is now available to all users](https://alternativeto.net/news/2025/8/claude-s-learning-mode-is-now-available-to-all-users-with-new-coding-styles) (August 2025)
- Tessl, [Claude Code now lets you customize its communication style](https://tessl.io/blog/claude-code-now-lets-you-customize-its-communication-style)
- eesel, [Claude Code output styles (2026)](https://www.eesel.ai/blog/output-styles-claude-code) (source for the deprecation versions; unverified against Anthropic's changelog)
