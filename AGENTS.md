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
- **Agents must be able to see it.** Every feature shows up in `fictty screen` (the frame as text)
  or `fictty get --state` (JSON).

## Layout

- `src/model.rs`: the UI value (nodes, kinds, sources, actions, theme, layers) and the template
  language (`{path}`, `{±x}`, maps, conditionals, `<style>…</>` markup)
- `src/state.rs`: `State`, `Msg`, `Effect`, `update`, key resolution, focus and selection
- `src/view.rs`: the pure view: draws every component, returns hit regions and pictures
- `src/gfx.rs`: pixels: the kitty graphics protocol, rasterising, SVG (resvg), caching
- `src/runtime.rs`: the terminal, the event queue, data sources, the socket, `upgrade`
- `src/main.rs`: the CLI, one subcommand per socket operation
- `examples/`: UIs and their feeds. `stress/` and `finance/` are standalone; `thc/` and
  `pixels/` need Thought Control installed (they move to that repo, see the plan)
- `docs/`: the guide, the plan, design proposals, the identity explorations

## Build, test, run

```sh
mise exec -- cargo build --release      # or plain cargo; the toolchain is stable (mise.toml)
mise exec -- cargo test
mise exec -- cargo clippy --all-targets
```

- Tests live next to the code (`#[cfg(test)]` in `state.rs`, `model.rs`, `view.rs`). `update` and
  `view` are pure, so test them directly: build a `State`, apply messages, assert on state or on
  the rendered text.
- **See your change without a terminal:** `fictty render ui.json --size 120x40 --keys j,j,tab`
  prints the frame; `--ansi` keeps colours.
- **Drive a running screen** the way an agent would: `fictty run ui.json` in one terminal, then
  `push`, `patch`, `key`, `mouse`, `layers`, `screen`, `get --state`, `get --stats` from another.
  `FICTTY_SOCKET=/tmp/x.sock` runs a second screen beside the first (keep socket paths short).
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
