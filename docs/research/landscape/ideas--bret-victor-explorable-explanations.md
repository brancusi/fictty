# Explorable explanations and Dynamicland (Bret Victor)

- **Maker:** Bret Victor; Dynamicland (with the Dynamic Medium group)
- **URL:** https://worrydream.com/ExplorableExplanations/ and https://dynamicland.org
- **Status (10 October 2026):** "Explorable Explanations" is an essay of 10 March 2011 with a
  postscript dated February 2024. Dynamicland published a new website in 2024; Victor's homepage
  says "I am (now and forever) making Dynamicland" and "currently, we are building a dynamic
  library." We found no 2025 or 2026 announcement beyond that.

## What it is

The essay that named the genre. Victor argues reading tools present the author's argument and
nothing else, so the reader's questions stay private. Text should be "an environment to think in"
rather than "information to be consumed." He describes three devices:

- **Reactive documents:** the reader changes an author's assumptions and watches conclusions update.
- **Explorable examples:** an interactive model woven into the argument (not sealed in a sandbox)
  that lets the reader "develop an intuition for how a system works."
- **Contextual information:** related material pulled up in place, just in time.

Dynamicland is his later, larger bet: a building where the computer is the room, with programs on
paper that people pick up, arrange and change together.

## The problem it's solving

Passive reading. Readers can't test an author's claims, so they either trust or ignore them.
Underneath: the medium of thought is fixed by what's cheap to author, and interactive media were
expensive.

## Its path / bet

Authors build models, not just prose, and readers manipulate them. The bet is on a new authoring
medium; the essay is honest that the tools (Tangle, his small JS library) are a sketch.

## How it works (concretely)

Hand-built web pages. A number in a sentence is draggable; dragging it reruns the model and updates
every dependent number and chart. Every example in the essay was authored by hand.

## Strengths

- The canonical argument for interactive explanation, still cited fifteen years on.
- Precise about what makes interaction useful (manipulating assumptions, seeing consequences), which
  is a better test than "is it visual."
- Insists the exploration live inside the explanation, guided by the author.

## Weaknesses / limits

- Authoring cost. Every explorable is a handmade artefact; the genre stayed small for that reason
  (see Distill and Nicky Case).
- Dynamicland requires a physical space and has stayed a research site.
- No evidence base of its own; the case is made by demonstration.

## Relation to fictty

**Inspire.** The cost that held this genre back is the cost models now remove. A reactive document is
a set of values with dependents, which is close to a fictty UI value bound to a data source with a
parameter the reader drags. fictty's terminal medium is a poor fit for his richer examples
(continuous dragging, smooth charts), better for tables, step-throughs and parameters typed or
nudged with keys.

## Could fictty adopt it instead of building?

Nothing to adopt. Tangle is a 2011 JS library for the browser.

## What fictty should take from it

- **Parameters are a component.** A value the person nudges (keys or mouse) that re-runs a bound
  source and updates every dependent node is the reactive document in fictty terms. Today fictty
  has data sources and actions; the missing piece is a named parameter that sources read.
- **Guided, not sandboxed.** A teaching screen is a walkthrough with steps, callouts and spotlights
  (fictty's layers), not a free dashboard.
- **Test of value:** does the screen let the person change an assumption and see the consequence?
  If not, it's a picture, and text might do.

## Sources

- Bret Victor, [Explorable Explanations](https://worrydream.com/ExplorableExplanations/) (2011, postscript 2024)
- [Bret Victor's homepage](https://worrydream.com/); [Dynamicland](https://dynamicland.org/about-us/)
- Wikipedia, [Bret Victor](https://en.wikipedia.org/wiki/Bret_Victor)
