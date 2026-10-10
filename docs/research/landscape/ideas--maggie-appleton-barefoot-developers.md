# Home-cooked software and barefoot developers (Maggie Appleton)

- **Maker:** Maggie Appleton
- **URL:** https://maggieappleton.com/home-cooked-software
- **Status (10 October 2026):** a talk at Local-First Conf, Berlin, May 2024, with a companion essay
  (dated 2 June 2024 by one source). Widely cited; no product.

## What it is

A talk and essay arguing that language models could bring "a golden age of local, home-cooked
software" built by a new kind of maker, the **barefoot developer**, named after China's barefoot
doctors of the 1960s: technically capable people between end users and professional programmers
(the teacher with the elaborate spreadsheet) who are stopped today by the "command line wall."

## The problem it's solving

Most communities' software needs are simple (CRUD apps, basic auth, a few API calls) but nobody
local can build them, and industrial software doesn't serve them. LLMs might let the barefoot
developer get over the wall.

## Its path / bet

Models give you "a bunch of disconnected lego pieces" but not the glue: deployment, persistence,
sync, multiplayer. Her bet is that the glue comes from agents plus local-first infrastructure, and
she asks the local-first community to make that the default for this kind of software.

## How it works (concretely)

It doesn't; it's a thesis. She cites v0, tldraw's Make Real and Copilot studies as early signs.

## Strengths

- Names the missing layer honestly: generation is the easy part, the glue is the hard part.
- Grounds "home-cooked" in a real population (spreadsheet power users) rather than in programmers.
- Self-aware: she calls the theory "slightly bold" and acknowledges the "more crap code and bugs"
  objection.

## Weaknesses / limits

- 2024 predictions, mostly unverified two years on. Barefoot developers using agents exist, but we
  found no evidence that local-first glue became the default for them.
- Like Sloan, the target is durable community software, not ephemeral screens.

## Relation to fictty

**Inspire.** Her "glue" argument maps directly onto fictty's choice: a runtime that already does
drawing, input, data binding and read-back is glue, so that the agent only writes the arrangement.

## Could fictty adopt it instead of building?

Not applicable.

## What fictty should take from it

- The glue is the product. A screen format is cheap; a runtime that persists, binds data and runs
  behaviour without the model is what makes generated UIs usable.
- The user for "patch the screen yourself" is the barefoot developer, not the programmer. The
  format should be something such a person can read and edit, which is an argument for keeping it
  plain data with no expressions.

## Sources

- Maggie Appleton, [Home-cooked software and barefoot developers](https://maggieappleton.com/home-cooked-software) (May 2024)
- Simon Willison, [link post](https://www.simonwillison.net/2024/Jul/6/home-cooked-software)
- Web Directions, [The home-cooked computer club](https://webdirections.org/blog/the-home-cooked-computer-club/)
