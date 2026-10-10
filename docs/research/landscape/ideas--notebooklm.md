# NotebookLM (Google)

- **Maker:** Google (Google Labs)
- **URL:** https://notebooklm.google
- **Status (10 October 2026):** shipping, free with paid tiers. Audio Overviews (2024) made it
  famous; mind maps arrived in 2025; Video Overviews launched 29 July 2025 (narrated slides with
  images, diagrams, quotes and numbers pulled from the sources), with a Studio panel holding audio,
  video, mind map and report outputs, several per notebook. One secondary source reports "Cinematic
  Video Overviews" in March 2026 (Gemini 3, Veo 3, Ultra plan only); we couldn't confirm it from
  Google.

## What it is

A research and study tool grounded in sources you upload: it answers questions with citations and
turns the sources into other formats (podcast-style audio, narrated slide videos, mind maps,
study guides, flashcards, quizzes).

## The problem it's solving

Reading and digesting a pile of documents is slow, and one format doesn't suit every learner or
every moment (commuting, reviewing, first pass).

## Its path / bet

Grounding first (answers come from your sources, with citations), then many generated
representations of the same material. The visual side is a set of fixed output types the model
fills, not free-form UI.

## How it works (concretely)

Gemini models over the uploaded sources. A mind map is a navigable tree of topics from the selected
sources; a Video Overview is a generated slide deck with narration, steerable by focus, learning
objective, audience and expertise.

## Strengths

- Grounding and citations make it trustworthy in a way free generation isn't.
- Fixed output types (map, deck, audio) are reliable and familiar.
- Very large adoption; the Audio Overview format was widely copied.

## Weaknesses / limits

- Mostly passive formats (listen, watch). The mind map is the only manipulable one, and it's
  navigation, not a model you can change.
- No evidence we found that the formats improve learning, as opposed to convenience.

## Relation to fictty

**Inspire.** Two lessons: ground the representation in real sources, and offer a few fixed,
well-made formats rather than infinite generated ones. Both match fictty's catalogue-plus-data-sources
design. Different medium and market.

## Could fictty adopt it instead of building?

No. A closed Google product (third-party MCP wrappers exist, unofficial).

## What fictty should take from it

- **Every number on a teaching screen should trace to a source.** NotebookLM's citations are the
  same principle as fictty's data sources; show the source of a bound value on demand (the command
  it came from).
- **The mind map is a good component to have:** a navigable tree with a detail pane. fictty has
  list, table and text; a tree view is worth checking for in the component set.

## Sources

- 9to5Google, [NotebookLM Video Overviews](https://9to5google.com/2025/07/29/notebooklm-video-overviews/) (29 July 2025)
- Maginative, [NotebookLM can now turn your notes into narrated videos](https://www.maginative.com/article/notebooklm-can-now-turn-your-notes-into-narrated-videos/)
- NYU Shanghai RITS, [What's new in NotebookLM](https://rits.shanghai.nyu.edu/ai/whats-new-in-notebooklm-video-overviews-studio-upgrades/)
- Cyfuture, [What is NotebookLM in 2026](https://cyfuture.cloud/kb/ai/what-is-notebooklm-features-and-how-to-use-it-in-2026) (secondary, unverified)
