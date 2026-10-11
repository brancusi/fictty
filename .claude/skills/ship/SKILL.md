---
name: ship
description: Ship a finished fictty working session to the public - docs, a pinned demo one-liner for the session's changes, the latest demo on fictty.com, and a blog post - then push and deploy. Use when Aram says to ship, release, publish or push live a session's work.
---

# Shipping a session

When a working session is done, everything goes live together: the code, the docs, a demo anyone
can run in one line, the landing page's demo, and a post. Asking for this skill is Aram's go-ahead
to push `main` and deploy fictty.com. Anything beyond that (a release tag, a crates.io publish,
deleting anything public) still needs its own yes.

Work through the steps in order. Don't move to the next while the current one fails.

## 0 · Ready

- `mise exec -- cargo test`, `mise exec -- cargo clippy --all-targets`: green.
- Everything is committed on `main` (merge any branch), with explicit `git add` paths.
- `scripts/preflight.sh` passes.
- Read `git log` since the last shipped post (the newest `web/public/try/*.sh` names its commit)
  and list what changed for a person using fictty. That list drives every step below.

## 1 · Update the docs

- `README.md`, `docs/guide.md`: every new or changed feature, command, flag and example is
  described, and nothing describes what's gone.
- `docs/PLAN.md`: tick what landed, add what the session found, keep the order honest.
- Design docs the session followed: set their status (built, merged on <date>), and add what
  building them taught.
- `AGENTS.md`, if the layout, rules or skills changed.
- New docs a post will link get a page on fictty.com (see the blog-post skill, "Links").

## 2 · A versioned demo for this build

One line a reader can paste that runs **exactly this build** and **shows off what changed**.

1. Make the demo show the change. If the showcase (`examples/showcase/`) doesn't exercise the new
   features, extend it first: a new screen, a new step in `tour.sh`, a new seed in the store.
   The demo is the proof; a feature it doesn't show didn't ship. Commit and push.
2. Freeze it on that commit:

   ```sh
   scripts/freeze-demo.sh <post-slug> <commit> "<post title>"
   ```

   This writes `web/public/try/<post-slug>.sh`, pinned to the commit, served at
   `https://fictty.com/try/<post-slug>.sh`. Never edit a frozen file by hand, and never move or
   delete one; to fix one, rerun the script with a dated note.
3. Test it the way a reader runs it, through a pipe, in a terminal, from an empty home, with a
   second terminal driving it:

   ```sh
   scripts/try-test.py web/public/try/<post-slug>.sh --expect Showcase --key enter --expect "New note" \
     --second "fictty get --stats >/dev/null" --second "TOUR_PAUSE=0.2 fictty-tour >/dev/null 2>&1"
   ```

   Add an `--expect`/`--key` pair that reaches each new feature on screen.
4. Commit the frozen script with the demo line in the commit message, so the commit carries it:

   ```
   Demo: curl -fsSL https://fictty.com/try/<post-slug>.sh | sh
   ```

5. Wherever the build is shown (the post, release notes), use the **demo link component**, never a
   bare code block:

   ```html
   <div class="demo-link" data-name="What it shows" data-date="YYYY-MM-DD" data-script="try/<post-slug>.sh" data-commit="<full commit>"></div>
   ```

   It renders the name, the date, the pinned commit, the command and a copy button
   (`web/src/lib/demo-link.mjs`; on pages, `web/src/components/DemoLink.astro`).

## 3 · The latest demo on the landing page

The landing page's demo link, above the fold, is the global one: `https://fictty.com/try.sh`, which
runs `main` as it is, not a pinned commit. It should show a good range of fictty, so the showcase
on `main` is the one to keep growing: new features join it (step 2.1), old ones stay.

- `web/src/pages/index.astro` renders `<DemoLink … script="try.sh" latest compact />` at the top of
  the hero; its date is `main`'s last commit at build time. Check it's there and above the fold at
  1300×900 (a screenshot).
- After pushing, test the live latest link from an empty home:

  ```sh
  scripts/try-test.py https://fictty.com/try.sh --expect Showcase --second "fictty get --stats >/dev/null"
  ```

  and check it fetched `main`'s head (`git -C <home>/main rev-parse HEAD` = `git rev-parse origin/main`).
- If `try.sh` itself changed, older frozen scripts don't change with it. Decide whether they need
  the fix (rerun `freeze-demo.sh` with a note) and test each one you touch.

## 4 · The post

Write the session's post with the **blog-post** skill. On top of what it asks:

- The versioned demo link (step 2.5) goes where the post shows the new features, with a sentence
  on what to try.
- The Tokens panel covers the session (`scripts/blog-tokens.py`).
- The post links the docs from step 1 rather than repeating them.

## 5 · Publish and check

1. `scripts/preflight.sh`, then `git push origin main`.
2. `cd web && npm run deploy` (build, private-material scan, link check, upload).
3. Check live, through a resolver that isn't cached (`dig @1.1.1.1 fictty.com`):
   - the post, `/try.sh`, `/try/<post-slug>.sh` answer 200;
   - the landing page shows the latest demo link above the fold;
   - `scripts/try-test.py https://fictty.com/try/<post-slug>.sh …` passes against the live URL.
4. Tell Aram what shipped: the post's link, the demo line, the commit, what was tested, and
   anything that wasn't.
