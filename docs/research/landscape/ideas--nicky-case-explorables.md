# Nicky Case's explorables

- **Maker:** Nicky Case
- **URL:** https://ncase.me
- **Status (10 October 2026):** an independent creator funded by Patreon. Best-known works are
  *Parable of the Polygons* (2014, with Vi Hart), *The Evolution of Trust* (2017; `ncase/trust` has
  6.3k GitHub stars, CC0) and *To Build a Better Ballot*. Recent work has moved to AI-safety writing
  ("AI Safety for Fleshy Humans," 2024 to 2025) and comics; the most recently pushed public repo is
  `civic-ai-comics` (July 2026). They also made Nutshell, a tool for expandable explanations. We
  couldn't confirm a new explorable in 2025 or 2026.

## What it is

Playable explanations of systems: segregation from small biases, trust as an iterated game,
anxiety, voting methods, epidemics. The reader runs a simulation, changes the rules, and sees the
result, with a narrative guiding them.

## The problem it's solving

Systems behaviour (emergence, feedback, equilibria) is hard to grasp from text or a static
diagram. Letting people play the system builds intuition that prose can't.

## Its path / bet

Craft: one person writing, drawing and programming each piece by hand, released free, often
public-domain. Case also curated the explorable explanations hub that collected the genre.

## How it works (concretely)

Hand-built HTML, JS and canvas pages with simulations and step-by-step narration. Each takes
months.

## Strengths

- Proof that interactive explanation can reach millions of people.
- A pattern worth copying: narrate, then hand over a control, then ask the reader to predict.

## Weaknesses / limits

- Months per piece, by a rare multi-skilled author. The output rate is a handful a year at best,
  and has slowed as the author moved to other forms.
- No controlled evidence that the pieces teach better than good text; their reach is the evidence.

## Relation to fictty

**Inspire.** The pattern (narrated steps, a simulation, a control, a prediction) is what a good
agent-made teaching screen should follow. The cost per piece is what agents change.

## Could fictty adopt it instead of building?

No. These are finished artefacts, not a toolkit. Nutshell is a small browser tool for expandable
text.

## What fictty should take from it

- **Predict, then reveal.** Ask the person to guess before showing the result; read the guess back.
  It's cheap in fictty (a prompt node, `watch`) and it is the part of explorables most supported by
  learning research.
- **Narration plus one control** beats a dashboard of controls. Keep teaching screens to one thing
  the person changes at a time.

## Sources

- [ncase.me](https://ncase.me); [The Evolution of Trust repo](https://github.com/ncase/trust) (GitHub API, 10 October 2026)
- [Nicky Case on Patreon](https://www.patreon.com/ncase) (recent activity; titles only)
- Data Stories, [episode 103 with Nicky Case](https://datastori.es/103-explorable-explanations-with-nicky-case/)
