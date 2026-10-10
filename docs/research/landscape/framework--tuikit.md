# TUIKit (GitHub)

- **Maker:** GitHub (repo `github/TUIKit`; titled "TUIkit Specs"). Experimental.
- **URL:** https://github.com/github/TUIKit
- **Status, 10 October 2026:** 23 stars, MIT. Created 16 April 2026; last commit 17 September 2026
  (pinning CI actions). No releases. Needs Bun and, for `compile build`, a GitHub Copilot
  subscription.

## What it is

Not a framework but a spec system: each terminal UI component is a Markdown file with YAML
frontmatter (props, tokens, accessibility role and announcements, behavioural tests, previews). An LLM
acts as the compiler and generates idiomatic code per target: Go/Bubble Tea, TypeScript/Ink,
Bun/OpenTUI, Rust/ratatui. Lock files record which spec versions were compiled.

## The problem it's solving

The same components (and the same accessibility behaviour) across many TUI frameworks, without
hand-porting each one. GitHub ships CLIs in more than one stack (gh in Go, Copilot CLI on Ink).

## Its path / bet

Headless UI primitives (like Radix or Base UI) for terminals, with the specs as an intermediate
representation and an LLM as the code generator. It bets that code is cheap to regenerate and the
durable asset is the behavioural contract.

## How it works

`compile.ts` diffs specs against lock files, writes a self-contained prompt per target, an agent
generates code into `dist/<target>/`, a person verifies it, then `compile lock` records hashes.
Semantic design tokens (colours, icons, breakpoints) are shared across targets.

## Strengths

- Accessibility is in the spec (roles, announcements), not bolted on.
- Clean separation of behaviour contract from implementation.
- A credible signal that GitHub thinks of terminal components as something LLMs generate.

## Weaknesses / limits

- Experimental, 23 stars, sparse activity since June. Could be abandoned.
- Output is code that someone still has to build, ship and maintain per framework.
- Nothing at runtime: no live screen, no read-back, no data binding.

## Relation to fictty

**Inspire.** It sits at the other end of the code-versus-data axis: specs compile to code, whereas
fictty interprets data at runtime.

## Could fictty adopt it instead of building?

No; it isn't a runtime. fictty could consume its specs, though: a TUIKit component spec is close to
a fictty project component plus behaviour, and its accessibility fields are what fictty's planned
linear rendering needs.

## What fictty should take from it

- **Accessibility fields on every primitive** (`role`, `announce`), taken from TUIKit's schema rather
  than invented.
- **Behavioural tests in the component spec,** runnable with `fictty render --keys`.
- **Semantic tokens** as the theme vocabulary.

## Sources

- https://github.com/github/TUIKit (README; GitHub API 10 Oct 2026)
- Not verified: whether any GitHub product uses TUIKit output.
