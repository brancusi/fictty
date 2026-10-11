#!/usr/bin/env bash
# freeze-demo.sh SLUG VERSION "Post title" [note]
#
# Writes web/public/try/SLUG.sh: a frozen copy of web/public/try.sh whose default version is
# VERSION, for the post SLUG: a release tag (v0.1.0, which has binaries, so readers don't need
# Rust) or a commit. Either must be on GitHub first. Its URL, https://fictty.com/try/SLUG.sh,
# runs exactly that commit's demo forever. Don't edit a frozen file by hand; if one must change
# (a fix in try.sh that old demos need), run this again with a note saying why and when.
set -euo pipefail
cd "$(dirname "$0")/.."
slug=$1; title=$3; note=${4:-}
if [[ $2 == v[0-9]* ]]; then
  git ls-remote --exit-code --tags origin "refs/tags/$2" >/dev/null || { echo "push the tag $2 first" >&2; exit 1; }
  commit=$2; at="release $2 ($(git rev-parse --short "$2^{commit}"))"
else
  commit=$(git rev-parse "$2^{commit}"); at="commit ${commit:0:12}"
  git branch -r --contains "$commit" | grep -q 'origin/' || { echo "push $commit first: readers fetch it from GitHub" >&2; exit 1; }
fi
out="web/public/try/$slug.sh"
mkdir -p "$(dirname "$out")"
{
  echo '#!/bin/sh'
  echo "# FROZEN for the post \"$title\" (https://fictty.com/blog/$slug/):"
  echo '#'
  echo "#   curl -fsSL https://fictty.com/try/$slug.sh | sh"
  echo '#'
  echo "# It runs fictty as it was at $at. Don't edit this file by hand: it's the"
  echo '# post'"'"'s demo. Regenerate it with scripts/freeze-demo.sh, with a note, if it must change.'
  [[ -n $note ]] && echo "# Changed: $note"
  echo '#'
  tail -n +2 web/public/try.sh | sed "s|^ref=\"\${1:-\${FICTTY_REF:-main}}\"|ref=\"\${1:-\${FICTTY_REF:-$commit}}\"|"
} >"$out"
grep -q "^ref=\"\${1:-\${FICTTY_REF:-$commit}}\"" "$out" || { echo "couldn't pin the version in $out" >&2; exit 1; }
sh -n "$out"
echo "$out pins $commit"
