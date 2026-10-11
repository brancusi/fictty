#!/bin/sh
# Try the fictty showcase in one line:
#
#   curl -fsSL https://fictty.com/try.sh | sh
#
# It fetches fictty into ~/.fictty (or $FICTTY_HOME), uses a prebuilt binary for this machine
# when a release has one and builds from source with cargo otherwise, then runs the showcase:
# notes, a ledger, a live market, a gallery and an agent's log. Run it again to update.
# Nothing is installed outside that folder; delete it to remove everything.
#
# Needs git, and cargo (https://rustup.rs) when there's no prebuilt binary for this machine.
# python3 runs the ledger's and the market's helpers. FICTTY_NO_RUN=1 sets up without running.
set -eu

repo=https://github.com/brancusi/fictty
home="${FICTTY_HOME:-$HOME/.fictty}"
src="$home/src"
bin="$src/target/release/fictty"

say() { printf '\033[36m▸\033[0m %s\n' "$1" >&2; }
die() { printf '\033[31m✗\033[0m %s\n' "$1" >&2; exit 1; }

command -v git >/dev/null 2>&1 || die "fictty needs git. Install it, then run this again."

mkdir -p "$home"
if [ -d "$src/.git" ]; then
  say "Updating fictty in $src"
  git -C "$src" pull --quiet --ff-only || die "Couldn't update $src. Delete it and run this again."
else
  say "Fetching fictty into $src"
  git clone --quiet --depth 1 "$repo" "$src"
fi

# A prebuilt binary, if the latest release has one for this machine.
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-gnu ;;
  *) target="" ;;
esac
got=""
if [ -n "$target" ] && command -v curl >/dev/null 2>&1; then
  tmp="$home/fictty-$target.tar.gz"
  if curl -fsSL "$repo/releases/latest/download/fictty-$target.tar.gz" -o "$tmp" 2>/dev/null; then
    mkdir -p "$(dirname "$bin")"
    tar -xzf "$tmp" -C "$(dirname "$bin")" fictty && got=yes
    rm -f "$tmp"
    [ -n "$got" ] && say "Using the prebuilt fictty for $target"
  fi
fi
if [ -z "$got" ]; then
  command -v cargo >/dev/null 2>&1 ||
    die "No prebuilt fictty for this machine yet, and cargo isn't installed. Install Rust from https://rustup.rs, then run this again."
  say "Building fictty (a minute or two the first time)"
  (cd "$src" && cargo build --release --quiet) || die "The build failed; the output above says why."
fi

command -v python3 >/dev/null 2>&1 ||
  say "python3 isn't installed: the ledger's summary and the live market will stay empty."

if [ "${FICTTY_NO_RUN:-}" = 1 ]; then
  say "Ready. Run it with: $src/examples/showcase/play.sh"
  exit 0
fi
# The showcase is drawn for about 160×46; it works smaller, best bigger.
if size=$(stty size </dev/tty 2>/dev/null); then
  rows=${size% *}; cols=${size#* }
  if [ "$cols" -lt 120 ] || [ "$rows" -lt 36 ]; then
    say "Your terminal is ${cols}×${rows}; the showcase looks best at 160×46 or more."
  fi
fi
say "Starting the showcase: arrows and Enter to open a screen, t for the theme, Esc back, q to quit"
say "Watch an agent drive it: run $src/examples/showcase/tour.sh in another terminal"
sleep 2
exec "$src/examples/showcase/play.sh" </dev/tty >/dev/tty
