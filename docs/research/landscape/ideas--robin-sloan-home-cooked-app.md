# "An app can be a home-cooked meal" (Robin Sloan)

- **Maker:** Robin Sloan (novelist and programmer)
- **URL:** https://www.robinsloan.com/notes/home-cooked-app/
- **Status (10 October 2026):** an essay from February 2020, updated yearly. The app it describes,
  BoopSnoop, was still in daily use in February 2026 (TestFlight build 30 in 2025, "no new
  features" in 2026).

## What it is

A short essay about BoopSnoop, a private iOS messaging app Sloan built for his family (his mother,
father, sister and a group of all three) to share photos and videos after the app they used,
Tapstack, shut down. "I am the programming equivalent of a home cook."

## The problem it's solving

Software is assumed to be a product for strangers at scale. Sloan argues it can also be made for a
handful of people you know, the way a meal is: no growth, no market, four daily active users and
zero churn.

## Its path / bet

Write the small thing yourself, for the people in front of you, and keep it running. Value comes
from fit and care, not from scale. The essay predates LLM coding by two years; later writers
(Maggie Appleton, Geoffrey Litt, Simon Willison) used it as the founding text for "home-cooked
software" once models made it cheaper.

## How it works (concretely)

A native iOS app distributed through TestFlight, with AWS S3 and Lambda behind it. He doesn't give
a cost.

## Strengths

- The cleanest framing of "software for an audience of a few," and widely cited.
- The yearly updates are a rare long-run data point: a home-cooked app lasted at least six years.

## Weaknesses / limits

- One person's experience, by a skilled programmer. It says nothing about non-programmers.
- Its example is durable, not disposable. A home-cooked meal is eaten once; this app has been kept
  for six years. The metaphor is about audience size, not lifespan.

## Relation to fictty

**Inspire**, at a distance. It separates two things the "cheap UI" debate often merges: software
for few people, and software that lives a short time. fictty is about the second; Sloan is about
the first.

## Could fictty adopt it instead of building?

Not applicable. It's an essay.

## What fictty should take from it

- Be precise in our own writing: "throwaway" is about lifespan, "home-cooked" is about audience.
  A fictty screen is usually both (one person, ten minutes), but the arguments are different.
- Some agent-made screens will be kept (BoopSnoop is the proof). The path from a screen an agent
  pushed once to a screen a person reopens every morning should exist: save, name, reopen.

## Sources

- Robin Sloan, [An app can be a home-cooked meal](https://www.robinsloan.com/notes/home-cooked-app/) (February 2020, with updates to February 2026)
- Simon Willison, [home-cooked software](https://www.simonwillison.net/2024/Jul/6/home-cooked-software) (on the lineage to Appleton)
