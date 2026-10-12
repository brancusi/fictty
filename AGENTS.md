# AGENTS.md

Guidance for agents (and people) working on fictty. Read [README.md](README.md) for what it is,
[docs/PLAN.md](docs/PLAN.md) for what's next, and
[docs/design/architecture.md](docs/design/architecture.md) for the rules behind the code.

## How we write code here

These come first; every review checks them.

- **Elm architecture, strictly.** A serializable `State`, every input a `Msg`, a pure
  `update(&mut State, Msg) -> Vec<Effect>`, a pure `view`. Runtime handles (the terminal, the
  socket, child processes, threads) stay in `runtime.rs`, outside the state. New state goes in
  `State`, never a hidden field, cell or thread-local. Time comes from `State::now_ms`, never
  `Instant::now()` in `update` or `view`. If it isn't in the state, it can't be read back,
  replayed or tested: treat that as a bug.
- **One source of truth.** The UI value, the data and the view state each live in one place;
  everything else holds an id. Never copy state between owners.
- **Data, never code.** A pushed UI is data. Don't add anything that lets a UI value execute
  logic: no expressions, loops or conditionals beyond the template language's substitution.
- **Behaviour belongs in the runtime; arrangement belongs in data.** A new primitive is justified
  only by behaviour or speed that composition can't give. Anything that only arranges or styles
  other components is a project component or a recipe, not Rust.
- **Special cases are a smell.** A one-off guard or a branch for one UI means the model is wrong.
  Fix the model instead.
- **Agents must be able to see it.** Every feature shows up in `fictty screen` (the frame as text,
  or `--nodes`) or `fictty get --state` (JSON).

## Layout

Two crates in one workspace. `crates/fictty-core` is the value and the engine, with nothing that
draws (it never compiles ratatui or GPUI); `crates/fictty` draws it and re-exports the core, so
embedding apps depend on `fictty` alone.

- `crates/fictty-core/src/model.rs`: the UI value (nodes, kinds, sources, actions, theme, layers)
  and the template language (`{path}`, `{±x}`, maps, conditionals, `<style>…</>` markup)
- `crates/fictty-core/src/state.rs`: `State`, `Msg`, `Effect`, `update`, key resolution, focus and
  selection
- `crates/fictty-core/src/engine.rs`: what runs a screen: the socket and its `Request`s, data
  sources, the host, saving, `upgrade`; `App` (how a screen starts), `Live` (the engine for
  whatever draws, on its own thread with `spawn`), `Inbox` and `Input`, `Headless` (driven
  inline, drawn by a `Frame` if given one); `Rect`, `Hit`, `Drawn`
- `crates/fictty-core/src/readback.rs`: the screen as an agent reads it, node by node
- `crates/fictty-core/src/lists.rs`: Markdown lists as text: renumbering, styles by depth (pure,
  text to edits)
- `crates/fictty-core/src/fuzz.rs`: the fuzz driver (the `fuzz` feature): generated acts on
  `Headless`, properties, shrinking, replay
- `crates/fictty-core/src/host.rs`: `Host`, what an embedding app provides (sources, actions, kept
  store entries), and the `Notifier` it talks back with
- `crates/fictty/src/view.rs`: the pure view (the terminal's grid): draws every component, returns
  hit regions, rectangles and pictures
- `crates/fictty/src/gfx.rs`: pixels: the kitty graphics protocol, rasterising, SVG (resvg),
  caching
- `crates/fictty/src/runtime.rs`: the terminal: `App` (the core's, plus `run` and a grid-drawn
  `headless`), the draw loop on the engine's rings, terminal input as the engine's, `Grid`
- `crates/fictty/src/window.rs`: the native window (`fictty run --window`, macOS), GPUI; the
  spike, growing to parity in phase 1 of [docs/design/native.md](docs/design/native.md)
- `crates/fictty/src/sim.rs`: a screen in a pseudo-terminal with a vt100 emulator (the `sim`
  feature): keys as a terminal's bytes, the mouse, pastes, resizes
- `crates/fictty/src/main.rs`: the CLI, one subcommand per socket operation
- `crates/fictty/tests/`: the library from outside: `components.rs` (each component, driven and
  read back), `editing.rs` (the editing-keys contract), `host.rs` (an embedding app), `fuzz.rs`
  (random input over editors and every chapter of the tour; `FUZZ_RUNS=2000` in a release build
  to go deeper), `sim.rs` (the binary in a pty)
- `examples/`: UIs and their feeds. `showcase/` is the demo the one-liners run (the tour's chapters
  are written by `tour.py`; `autopilot.sh` is its agent); `stress/` and
  `finance/` are standalone; `thc/` and `pixels/` need Thought Control installed (they move to
  that repo, see the plan)
- `docs/`: the guide, the plan, design proposals, the identity explorations, the blog
- `web/`: fictty.com, an Astro site that renders the docs and the blog. `web/public/try.sh` is the
  one-liner; `web/public/try/<post>.sh` are frozen per-post copies, never edited by hand
- `scripts/`: `preflight.sh` (before commits and pushes), `freeze-demo.sh` and `try-test.py`
  (pinned demos and testing them like a reader), `blog-tokens.py` (a post's Tokens panel)
- `.claude/skills/`: `blog-post` and `ship`, how milestones are written and published
- `.github/workflows/release.yml`: binaries for a version tag; how to release is
  [docs/releasing.md](docs/releasing.md) (CI is a background signal, not a gate, for now)

## Build, test, run

```sh
mise exec -- cargo build --release      # or plain cargo; the toolchain is stable (mise.toml)
mise exec -- cargo test
mise exec -- cargo clippy --all-targets
```

- Tests live next to the code (`#[cfg(test)]` in `state.rs`, `model.rs`, `view.rs`) and in
  `crates/fictty/tests/`. `update` and `view` are pure, so test them directly: build a `State`, apply messages,
  assert on state or on the rendered text; or drive `App::new(ui).headless(w, h)` with keys and
  read back `screen()` and `state()`.
- **See your change without a terminal:** `fictty render ui.json --size 120x40 --keys j,j,tab`
  prints the frame; `--ansi` keeps colours. `fictty screen --nodes` reads a running screen back
  node by node, whatever draws it.
- **Drive a running screen** the way an agent would: `fictty run ui.json` in one terminal, then
  `push`, `patch`, `key`, `mouse`, `layers`, `screen`, `get --state`, `get --stats` from another.
  `FICTTY_SOCKET=/tmp/x.sock` runs a second screen beside the first (keep socket paths short).
  `fictty run --window ui.json` opens the native window instead: a GUI app, so as with terminals,
  ask the person to open it, then drive it over the socket.
- Performance claims need a release build and numbers: `examples/stress/play.sh` against a running
  screen prints fps, draw time and messages a second. Measure before and after.
- Don't launch terminal applications (Ghostty, iTerm, WezTerm) from an agent session to test;
  use `render`, or ask the person to run the screen and drive it over the socket.

## Dependencies

- **ratatui** draws; use its widgets where they fit and write our own only for behaviour or speed.
- **caretline** (crates.io) is the text engine and `caretline-layers` places callouts. Engine bugs
  are fixed in caretline first, released, then pulled in. Never patch the engine here.
- Keep the dependency list short; justify each new crate in the commit message.

## Public repo hygiene

This repo is public. Before every commit and push:

- Stage explicit paths (`git add <files>`), never `git add -A` or `git add .`.
- Build output stays in the gitignored `/target` (or a `CARGO_TARGET_DIR` outside the checkout).
- Run `scripts/preflight.sh --staged` before committing and `scripts/preflight.sh` before pushing.
  It fails on build output, large or binary files outside `docs/`, home and scratch paths, the
  local user name, private URLs and credential-shaped strings. `scripts/install-hooks.sh` makes
  every push run it. Never bypass it with `--no-verify`; fix the cause, or add a commented line to
  `.preflight-allow` for a real exception.
- No personal data in examples, screenshots or fixtures: demo data is made up or generated.

## Commits

Small, focused commits with a message that says what changed and why, in plain sentences. Keep
`README.md`, `docs/guide.md` and `docs/PLAN.md` true: tick plan items as they land and update the
guide when a feature changes.

## Blog posts

Posts on fictty.com/blog are milestones written as a conversation between Aram (@brancusi) and
Tilde, the name Claude goes by on this project. Follow the `blog-post` skill
([.claude/skills/blog-post/SKILL.md](.claude/skills/blog-post/SKILL.md)): keep the conversation
true, use graphics, and ask before publishing.

## Shipping

When a session's work goes public, follow the `ship` skill
([.claude/skills/ship/SKILL.md](.claude/skills/ship/SKILL.md)): docs, a demo one-liner pinned to
the build (`scripts/freeze-demo.sh`, tested with `scripts/try-test.py`), the latest demo on the
landing page, and a post. Frozen demos in `web/public/try/` are never edited by hand.
