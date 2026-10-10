# Melker

- **Maker:** Erik Wistrand (GitHub `wistrand`)
- **URL:** https://github.com/wistrand/melker, https://melker.sh
- **Status (2026-10-10):** 24 stars, MIT, TypeScript on Deno (Node experimental). Created
  December 2025, last push 8 July 2026, CalVer releases. Small, single maintainer.

## What it is

"A TUI framework for apps you want to share safely." A Melker app is a `.melker` document: HTML-like
markup with CSS-like styles, a declared permission `<policy>`, and inline handlers. You can run it
from a URL; it runs sandboxed with only the permissions it declares; F12 opens dev tools showing
source, policy and the document tree. An optional AI assistant (F7/F8) sends the serialised UI
tree and focus to a model to help the person navigate.

## The problem it's solving

Running someone else's terminal app is running opaque code. Melker makes the app a document you
can read and approve before it runs.

## Its path / bet

Document-first terminal UIs: readable markup, explicit permissions, run from a URL. Not built for
agents, but it lands close to where agents need to be: a UI an agent can write as a file, which a
person can inspect before running.

## How it works

Deno launcher parses and bundles the document, spawns the app in a subprocess with Deno permission
flags from the policy, hash-based approval for remote apps. Flexbox layout, sortable tables,
dialogs, mouse, sextant-character graphics with fallbacks.

## Strengths

- Takes trust seriously: permissions are visible data, remote apps need approval.
- The UI tree is serialisable, and the project already sends it to a model.

## Weaknesses / limits

- Handlers are code (`onClick="$melker.exit()"`, scripts), so a model-written file can do
  anything its policy allows.
- No socket for an outside agent to push, patch, drive or read back a running app (that we could
  find).
- Tiny adoption.

## Relation to fictty

Inspire. Melker is the nearest thing to fictty's "command policy" item: what a pushed UI is allowed
to run, shown before it runs.

## Could fictty adopt it instead of building?

No. Different runtime, code in the document, no agent loop.

## What fictty should take from it

- Make a screen's permissions visible data: list every command a UI value can run (its data
  sources and key actions) in `get --state`, and let a person approve them once, by hash, the way
  Melker approves remote apps. That turns fictty's planned allow/ask/deny policy into something a
  person can read.
- The "linear rendering for an assistant" idea: Melker already serialises the tree for a model to
  read aloud or explain, which is the accessibility rendering the landscape post asked for.

## Sources

- https://github.com/wistrand/melker (README, FAQ.md)
- https://melker.sh
