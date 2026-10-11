#!/bin/sh
# Try the fictty showcase in one line:
#
#   curl -fsSL https://fictty.com/try.sh | sh                 # the latest fictty
#   curl -fsSL https://fictty.com/try.sh | sh -s -- v0.1.0    # a tag, a branch or a full commit
#
# It fetches that version of fictty into ~/.fictty/<version> (or $FICTTY_HOME/<version>), uses
# the release binary when the version is a release tag and builds from source with cargo
# otherwise, then runs the showcase: a tour of ten chapters, then notes, a ledger, a live market,
# a gallery and an agent's log.
# Each version keeps its own folder and its own store, so an old demo never mixes with a new one.
# Nothing is installed anywhere else; delete ~/.fictty to remove everything.
#
# Blog posts link a frozen copy of this script pinned to one commit (https://fictty.com/try/),
# so the demo a post shows stays exactly that demo.
#
# Needs git. Release tags have binaries for macOS and Linux; anything else builds with cargo
# (https://rustup.rs). Without cargo, the latest (`main`) falls back to the newest release.
# python3 runs the ledger's and the market's helpers. FICTTY_NO_RUN=1 sets up without running.
set -eu

ref="${1:-${FICTTY_REF:-main}}"
repo=https://github.com/brancusi/fictty
home="${FICTTY_HOME:-$HOME/.fictty}"

say() { printf '\033[36m▸\033[0m %s\n' "$1" >&2; }
die() { printf '\033[31m✗\033[0m %s\n' "$1" >&2; exit 1; }

# The latest without Rust: main can only be built, so take the newest release, which has binaries.
if [ "$ref" = main ] && ! command -v cargo >/dev/null 2>&1 && command -v curl >/dev/null 2>&1; then
  latest=$(curl -fsSL https://api.github.com/repos/brancusi/fictty/releases/latest 2>/dev/null |
    sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
  if [ -n "$latest" ]; then
    say "Rust isn't installed, so this runs the latest release, $latest, instead of building main"
    ref=$latest
  fi
fi
src="$home/$(printf '%s' "$ref" | tr -c 'A-Za-z0-9._-' '-' | cut -c1-40)"
bin="$src/target/release/fictty"

command -v git >/dev/null 2>&1 || die "fictty needs git. Install it, then run this again."

# This version of the code: a branch moves on each run, a tag or a commit never does.
say "Fetching fictty $ref into $src"
mkdir -p "$src"
[ -d "$src/.git" ] || { git init -q "$src" && git -C "$src" remote add origin "$repo"; }
git -C "$src" fetch -q --depth 1 origin "$ref" || die "Couldn't fetch $ref from $repo."
git -C "$src" checkout -q --detach FETCH_HEAD

# A release binary, only when the version is a release tag (so it always matches the code).
target=""
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-gnu ;;
esac
got=""
case "$ref" in
  v[0-9]*)
    if [ -n "$target" ] && command -v curl >/dev/null 2>&1; then
      tmp="$src/fictty-$target.tar.gz"
      if curl -fsSL "$repo/releases/download/$ref/fictty-$target.tar.gz" -o "$tmp" 2>/dev/null; then
        mkdir -p "$(dirname "$bin")"
        tar -xzf "$tmp" -C "$(dirname "$bin")" fictty && got=yes
        rm -f "$tmp"
        [ -n "$got" ] && say "Using the $ref release binary for $target"
      fi
    fi
    ;;
esac
if [ -z "$got" ]; then
  command -v cargo >/dev/null 2>&1 ||
    die "Building fictty needs Rust. Install it from https://rustup.rs, then run this again."
  say "Building fictty (a minute or two the first time)"
  (cd "$src" && cargo build --release --quiet) || die "The build failed; the output above says why."
fi

command -v python3 >/dev/null 2>&1 ||
  say "python3 isn't installed: the ledger's summary and the live market will stay empty."

# The showcase keeps notes and ledger entries here, apart from every other version's, and
# listens on a socket of its own, so it never collides with another screen.
export FICTTY_SHOWCASE_STORE="$src.store.json"
export FICTTY_SOCKET="${TMPDIR:-/tmp}/fictty-$(printf '%s' "$src" | cksum | cut -d' ' -f1).sock"

# `fictty` and `fictty-tour` for a second terminal: this version, on this screen's socket.
mkdir -p "$home/bin"
printf '#!/bin/sh\nFICTTY_SOCKET="${FICTTY_SOCKET:-%s}" exec "%s" "$@"\n' "$FICTTY_SOCKET" "$bin" >"$home/bin/fictty"
printf '#!/bin/sh\nFICTTY_SOCKET="${FICTTY_SOCKET:-%s}" exec "%s" "$@"\n' "$FICTTY_SOCKET" "$src/examples/showcase/tour.sh" >"$home/bin/fictty-tour"
chmod +x "$home/bin/fictty" "$home/bin/fictty-tour"

if [ "${FICTTY_NO_RUN:-}" = 1 ]; then
  say "Ready. Run it with: FICTTY_SOCKET=$FICTTY_SOCKET FICTTY_SHOWCASE_STORE=$FICTTY_SHOWCASE_STORE $src/examples/showcase/play.sh"
  exit 0
fi
[ -r /dev/tty ] || die "This needs a terminal to run in."
# The showcase is drawn for about 160×46; it works smaller, best bigger.
if size=$(stty size </dev/tty 2>/dev/null); then
  rows=${size% *}; cols=${size#* }
  if [ "$cols" -lt 120 ] || [ "$rows" -lt 36 ]; then
    say "Your terminal is ${cols}×${rows}; the showcase looks best at 160×46 or more."
  fi
fi
say "Starting the tour: n for the next chapter, p to go back, t for the theme, q to quit"
say "Each chapter has a quest; a for the apps (notes, a ledger, a live market) at any time"
sleep 2
# Hand the showcase the terminal's own device. Under `curl | sh` our input is the pipe, and the
# generic /dev/tty won't do: macOS can't wait on it (kqueue), so the screen would hang on its
# first read. ps names the real one (ttys003, pts/1).
term="/dev/$(ps -o tty= -p $$ 2>/dev/null | tr -d ' ')"
[ -c "$term" ] || term=/dev/tty
exec "$src/examples/showcase/play.sh" <"$term" >"$term"
