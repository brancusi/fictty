# Releasing

How a version of fictty goes out. The first was `v0.1.0`, on 11 October 2026. The ship skill
(`.claude/skills/ship/SKILL.md`) is the whole end-of-session routine; this is the release part.

## The rule for now: CI is a signal, not a gate

We're moving fast. Tag, push, freeze the demo and publish **without waiting for CI**. The release
workflow runs in the background; check it later, and fix what it finds in the next commit. Turn
it into a gate when the project slows down and more people depend on releases.

Until a tag's binaries arrive (a few minutes), the one-liners build from source, so a reader with
Rust is fine and one without it waits. That's the trade we accept.

## Cutting a release

1. Pick the version. While it's 0.x: a minor bump (`0.2.0`) for a session that changes the
   format or adds features, a patch (`0.1.1`) for fixes. Set `version` in `Cargo.toml`, run
   `cargo build` so `Cargo.lock` follows, and commit.
2. Tag and push, with what's in it and the line to try it:

   ```sh
   git tag -a v0.2.0 -m "fictty 0.2.0: <what's new, in a line>

   Try it: curl -fsSL https://fictty.com/try.sh | sh -s -- v0.2.0"
   git push origin v0.2.0
   ```

   Ask Aram before the first tag of a session: a tag is a public release.
3. Carry on: freeze the post's demo on the tag (`scripts/freeze-demo.sh <slug> v0.2.0 "<title>"`),
   write the post, deploy. Don't wait for step 4.
4. Later, look at the run (`gh run list --workflow release.yml`). If a job failed, fix the
   workflow or the code, push, and rebuild that tag's binaries by hand:

   ```sh
   gh workflow run release.yml -f tag=v0.2.0
   ```

   That rebuilds every target and replaces the release's files; it never moves the tag.

## What the workflow builds

`.github/workflows/release.yml`, on any `v*` tag: `fictty-<target>.tar.gz` (the binary alone)
for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu`, attached to the GitHub release (created if it doesn't exist).

- Intel Macs are cross-built on Apple silicon (`macos-14`): GitHub's Intel Mac runners
  (`macos-13`) never start any more. A job stuck in "queued" for minutes means a runner that's
  gone; change the matrix rather than waiting.
- Linux binaries are built on Ubuntu 22.04, so they need glibc 2.35 or newer.
- Builds use `--locked`: a tag whose `Cargo.lock` is stale fails. Build before tagging.

## How readers get it

- `curl -fsSL https://fictty.com/try.sh | sh` runs `main` when cargo is there, and the newest
  release when it isn't (`releases/latest`), with that release's binary.
- `… | sh -s -- v0.2.0` runs that release; a post's frozen demo (`web/public/try/<post>.sh`)
  usually pins one. Release tags download the binary; commits and branches build from source.
- A tag is fetched as it is on GitHub. Never move or delete a pushed tag: frozen demos and readers
  depend on it. A broken release gets a new patch version instead.

## Checks after a release

- `gh release view v0.2.0` lists the four `.tar.gz` files (once CI is done).
- Without Rust, the pinned demo runs the binary (`release binary` in its output): put a PATH with
  no cargo in front of `scripts/try-test.py <script> --expect "release binary" --expect Showcase`.
- `crates.io` isn't published yet (see the plan).
