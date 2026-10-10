# 3Blue1Brown and manim (and agents that write manim)

- **Maker:** Grant Sanderson (3Blue1Brown); the community fork by the Manim Community
- **URL:** https://github.com/3b1b/manim and https://github.com/ManimCommunity/manim
- **Status (10 October 2026):** `3b1b/manim` has 94.8k stars (MIT, last push 9 September 2026);
  `ManimCommunity/manim` has 41.4k stars (MIT, v0.22.0 on 8 October 2026). Both active. Research
  systems that generate manim with LLMs include TheoremExplainAgent (TIGER-AI-Lab, February 2025,
  1.5k stars), Manimator (ICML 2025) and LLM2Manim (2026), plus community Claude Code
  skills.

## What it is

manim is the Python animation engine Sanderson wrote for the 3Blue1Brown videos: you describe
mathematical objects and transformations in code and it renders a video. The community fork is the
documented, stable one most people use.

## The problem it's solving

Mathematical ideas are about change and structure, which static diagrams and text show badly.
Precise, programmatic animation makes the transformation itself visible.

## Its path / bet

Code in, video out. The explanation is authored, scripted and narrated; the viewer watches, they
don't interact. With LLMs, the bet shifts: an agent writes the scene code. TheoremExplainAgent
reports a 93.8% success rate generating long theorem videos with an o3-mini agent, while noting
that most videos had minor layout problems; LLM2Manim keeps a human review step for that reason.

## How it works (concretely)

Python scenes made of mobjects (shapes, LaTeX, graphs) and animations (Transform, FadeIn), rendered
with Cairo or OpenGL to video. Agent pipelines plan a storyboard, write scene code, render, look at
the result and repair.

## Strengths

- The most successful visual-maths explanation tool there is, by audience.
- Code is a precise, checkable description, which suits agents.

## Weaknesses / limits

- Output is a video: not interactive, not bound to data, not readable back by an agent except by
  watching frames.
- Rendering is slow (seconds to minutes per scene).
- Generated videos have layout errors often enough that research pipelines keep humans in the loop.

## Relation to fictty

**Inspire.** It shows what a good visual explanation looks like and that agents can now author one.
It sits at the opposite corner from fictty: authored code, rendered offline, watched passively.

## Could fictty adopt it instead of building?

No. Different medium. A fictty screen could show a rendered manim clip through pixels on a kitty
terminal, but that's a curiosity.

## What fictty should take from it

- **Animate the data, not the pictures** (already fictty guidance) is what manim does well: a
  transform between two states. A fictty transition between two values of a bound node, shown as
  steps, is the terminal version.
- **Layout errors are the failure mode of generated visuals.** A component library with layout
  built in, as fictty has, avoids the class of bug that dominates manim generation.

## Sources

- [3b1b/manim](https://github.com/3b1b/manim) and [ManimCommunity/manim](https://github.com/ManimCommunity/manim) (GitHub API, 10 October 2026)
- Ku et al., [TheoremExplainAgent](https://arxiv.org/abs/2502.19400) (February 2025); [repo](https://github.com/TIGER-AI-Lab/TheoremExplainAgent)
- [Manimator](https://icml.cc/virtual/2025/50491) (ICML 2025)
- [LLM2Manim](https://arxiv.org/pdf/2604.05266) (2026)
