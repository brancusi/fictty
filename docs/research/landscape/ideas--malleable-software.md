# Malleable software (Ink & Switch)

- **Maker:** Ink & Switch. The essay is by Geoffrey Litt, Josh Horowitz, Peter van Hardenberg and
  Todd Matthews.
- **URL:** https://www.inkandswitch.com/essay/malleable-software/
- **Status (10 October 2026):** an essay published June 2025, plus a research programme. The
  prototypes it cites (Patchwork, Potluck, Embark) are research software, not products. Ink &
  Switch lists Patchwork as active (2024 to 2026). Litt, the lead author, now works at Notion
  according to his own site; we couldn't find a Notion announcement.

## What it is

A long essay, "Malleable Software: Restoring User Agency in a World of Locked-Down Apps," that
defines malleable software as "a software ecosystem where anyone can adapt their tools to their
needs with minimal friction," and argues that mass-produced apps took that ability away. It sits
on a lineage of HyperCard, Smalltalk, spreadsheets, browser userscripts and Dynamicland.

## The problem it's solving

Software is built by distant teams, so people bend their work to fit their tools. The essay's
example: a team that tracked work on index cards lost the ability to change the process once it
moved to a web tracker ("computerizing work led to a loss of agency"). Plugins don't fix it,
because they only allow changes the vendor anticipated and every app's plugin system is different.

## Its path / bet

Three design patterns, not one product:

- **A gentle slope.** Each step up in customising power should cost only a little more skill
  (after MacLean et al., 1990). Start from direct manipulation and add programmability gradually.
- **Tools, not apps.** Apps are avocado slicers; tools are knives. Tools should share data the way
  files do and compose in one interface (compound documents, Dynamicland).
- **Communal creation.** Groups build and keep situated software for local needs, with local
  helpers rather than everyone programming.

On AI it is explicit and sceptical: "AI code generation alone does not address all the barriers to
malleability." A model can write you a new app, but that doesn't help you change the one you have,
compose two tools, or make a small precise edit without a conversation. The authors' image is a
talented sous chef in a food court. AI gets much more useful inside a malleable environment.

## How it works (concretely)

- **Patchwork:** tools and documents stored in Automerge (a local-first CRDT), with version
  control, branches and AI-assisted tool building. The lab uses it for its own work.
- **Potluck:** plaintext notes (recipes) gain behaviour from user-written patterns, such as scaling
  ingredient quantities. Parsing free text turned out to be cumbersome.
- **Embark:** a travel-planning outline whose embedded maps and calendars share the outline's
  context.

## Strengths

- The clearest statement of why "the model writes you a new app every time" is not the end state:
  durable data, composition and small edits still matter.
- Takes the data model seriously. Shared documents that many tools read is the same move as
  "one source of truth, everything else holds an id."
- Honest about open problems: security of shared modifications, business models, culture.

## Weaknesses / limits

- An essay and prototypes. None of it is something you can install and depend on.
- Aimed at durable personal and team tools, not at screens that live for ten minutes.
- The gentle slope is easier to describe than to build; Potluck's own write-up shows the cost.

## Relation to fictty

**Inspire.** fictty is close to the essay's ideal in one specific way: a UI that is a value can be
changed a piece at a time (patch by id) by an agent or, in principle, by a person, and the data it
shows comes from shared sources rather than being copied into the UI. It is far from the ideal in
another: fictty screens are mostly made and thrown away by agents, not adapted by their users.

## Could fictty adopt it instead of building?

No. There is nothing to adopt: Patchwork is a research environment for documents in a browser, not
a runtime for terminal screens. The ideas are free to take.

## What fictty should take from it

- **The gentle slope applies to the format.** A person should be able to tweak a pushed screen
  (change a column, add a key binding) without asking the agent to regenerate it. Patch by id is
  the start; a "save this screen as a recipe" step would be the next rung.
- **Tools share data.** Keep data sources separate from the screens that show them, so two screens
  (or a screen and a CLI) read the same source. Never copy the data into the UI value.
- **Use its scepticism.** "The model can make a new one" doesn't remove the need for a stable,
  editable representation. That is fictty's case against code-per-screen.

## Sources

- Ink & Switch, [Malleable Software](https://www.inkandswitch.com/essay/malleable-software/) (June 2025)
- Geoffrey Litt, [newsletter post on the essay](https://buttondown.com/geoffreylitt/archive/new-essay-malleable-software)
- Simon Willison, [link post](https://simonwillison.net/2025/Jun/11/malleable-software)
- Geoffrey Litt, [homepage](https://www.geoffreylitt.com/) (affiliation)
