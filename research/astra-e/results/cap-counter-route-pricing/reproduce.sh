#!/usr/bin/env bash
# Historical reproduction with the original Rust 1.99.1 toolchain and Valgrind 3.24.0 on PATH.
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
study="$repo/research/astra-e/results/cap-counter-route-pricing"
scratch=$(realpath -m "${1:?new scratch directory}")
corpus=$(realpath "${2:?original compressed Wikidata corpus}")
projection=3df195b10e9fb6f86f0c06ad9fad1feb992406d5
mkdir "$scratch"
git worktree add --detach "$scratch/scorer" d05577e22526c5880fc6f683f4189b5b48377dc5
git -C "$scratch/scorer" checkout "$projection" -- bench/moonshot .just Justfile
nice -n 15 taskset -c 4 bash "$study/build.sh" "$scratch/build" "$projection"
nice -n 15 taskset -c 4 bash "$scratch/scorer/bench/moonshot/chaingrind/build.sh" "$scratch/chaingrind"
nice -n 15 taskset -c 4 python3 -B "$study/run.py" "$scratch/scorer" "$scratch/build" "$scratch/chaingrind" "$corpus" "$scratch/results" --historical-master "$projection"
python3 -B "$study/report.py" "$scratch/scorer" "$scratch/results"
