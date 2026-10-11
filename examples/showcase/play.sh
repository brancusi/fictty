#!/usr/bin/env bash
# The fictty showcase: a tour of nine chapters, then a menu of apps (notes, a ledger, a live
# market, a gallery, an agent's log), every one of them JSON, sharing one store and three themes.
# FICTTY_START=home.json starts at the apps instead.
#
#   examples/showcase/play.sh            run it (builds fictty first if needed)
#   examples/showcase/tour.sh            in another terminal: an agent drives it
#
# The store (your notes, the ledger) is kept in $FICTTY_SHOWCASE_STORE, by default
# /tmp/fictty-showcase.json, so it's there next time. Delete that file to start over.
# Needs python3 for the two helper processes (books.py, ticks.py).
set -euo pipefail
cd "$(dirname "$0")"
root="$(cd ../.. && pwd)"
bin="$root/target/release/fictty"
if [[ ! -x $bin ]]; then
  (cd "$root" && cargo build --release)
fi
store="${FICTTY_SHOWCASE_STORE:-/tmp/fictty-showcase.json}"
exec "$bin" run "${FICTTY_START:-tour-1-hello.json}" --store "$store" --themes themes.json "$@"
