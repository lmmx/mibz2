#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
wt=${1:?dedicated scorer checkout}
build=${2:?dedicated scorer build directory}
out=${3:?fresh gate directory}
source_rev=aed11af
candidate=research/astra-e/residual-24/p16.rs
mkdir "$out"; out=$(cd "$out" && pwd)
if [[ ! -d $wt ]]; then git -C "$root" worktree add --detach "$wt" "$source_rev"; else git -C "$wt" checkout --detach "$source_rev"; fi
cd "$wt"
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
{
  echo "candidate $(git rev-parse HEAD)"
  echo "refreshed origin/master $(git rev-parse origin/master)"
  echo "scorer path $(git log -1 --format=%H origin/master -- bench/moonshot)"
  rustc -vV
  sha256sum "$candidate" research/astra-e/residual-24/{constructor.rs,adapter.inc.rs,narrow.rs} "$root/corpora/wikidata.bz2"
} > "$out/manifest.txt"
nice -n 12 python3 bench/moonshot/moonshot.py score "$candidate" --corpus "$root/corpora/wikidata.bz2" --no-timing --no-proxy --target-dir "$build" --out "$out/score" > "$out/stdout.txt" 2> "$out/stderr.txt"
