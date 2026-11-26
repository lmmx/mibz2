#!/usr/bin/env bash
# Historical replay. Use an empty directory, the original corpus and the pinned Rust/Valgrind environment.
set -euo pipefail
study=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$study" rev-parse --show-toplevel)
scratch=$(realpath -m "${1:?supply a new replay directory}")
corpus=$(realpath "${2:?supply the canonical compressed Wikidata corpus}")
mkdir "$scratch"
git -C "$repo" worktree add --detach "$scratch/scorer" e0a84bfd9acda7b21175e2efc1546c51f44a2214
git -C "$scratch/scorer" checkout 3df195b10e9fb6f86f0c06ad9fad1feb992406d5 -- bench/moonshot .just Justfile
cd "$repo"
nice -n 15 taskset -c 4 bash "$study/build.sh" "$scratch/build"
nice -n 15 taskset -c 4 bash "$scratch/scorer/bench/moonshot/chaingrind/build.sh" "$scratch/build/chaingrind"
nice -n 15 taskset -c 4 python3 -B "$study/run.py" "$scratch/scorer" "$scratch/build" "$scratch/build/chaingrind" "$corpus" "$scratch/results"
python3 -B "$study/report.py" "$scratch/scorer" "$scratch/results"
