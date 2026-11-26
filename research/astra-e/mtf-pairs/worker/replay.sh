#!/bin/bash
# Build in a clean detached research worktree. Keep each target in a separate directory.
set -euo pipefail
tree=$(realpath "${1:?detached research worktree at 01e44cbe011c1f965b0e54fa724a738083512976}")
work=$(realpath -m "${2:?fresh scratch directory}")
corpus=$(realpath "${3:?wikidata.bz2}")
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$work/bin" "$work/gates" "$work/direct"
cd "$tree"
python3 research/astra-e/mtf-pairs/worker/prepare.py
python3 research/astra-e/front-fusion/cap-batch/worker/prepare.py
for arm in candidate copied scalar; do
    git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
    git log -1 --format=%H origin/master -- bench/moonshot > "$work/gates/$arm-version.txt"
    git ls-tree origin/master bench/moonshot >> "$work/gates/$arm-version.txt"
    if [ "$arm" = scalar ]; then candidate=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
    else candidate="research/astra-e/mtf-pairs/worker/$arm.rs"; fi
    python3 bench/moonshot/moonshot.py score "$candidate" --no-proxy --no-timing --target-dir "$work/build" --out "$work/gates/$arm" --corpus "$corpus" >"$work/gates/$arm.log" 2>&1
    cp "$work/build/release/moonshot" "$work/bin/$arm"
done
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%H origin/master -- bench/moonshot > "$work/direct/scorer.txt"
git ls-tree origin/master bench/moonshot >> "$work/direct/scorer.txt"
for arm in candidate copied scalar; do cmp "$work/gates/$arm-version.txt" "$work/direct/scorer.txt"; done
python3 "$here/panel.py" run "$work/bin" "$corpus" "$work/direct"
python3 "$here/panel.py" report "$work/direct" > "$work/direct/summary.json"
