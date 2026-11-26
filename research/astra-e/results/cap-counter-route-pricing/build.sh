#!/usr/bin/env bash
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
scratch=${1:?new scratch directory}
projection=${2:?refreshed origin/master commit recorded in the research worktree}
old="$repo/research/astra-e/results/scalar-route-pricing"
mkdir -p "$scratch"
git -C "$repo" worktree add --detach "$scratch/source" 50b8ecb
git -C "$scratch/source" checkout "$projection" -- bench/moonshot .just Justfile
tar -tzf "$old/workers.tar.gz" > "$scratch/archive-contents.txt"
mkdir -p "$scratch/archive"
tar -xzf "$old/workers.tar.gz" -C "$scratch/archive"
if [[ "${3:-build}" == archive ]]; then
    tar -xJf "$repo/research/astra-e/results/cap-counter-route-pricing/counter-workers.tar.xz" -C "$scratch"
    exit 0
fi
export CARGO_BUILD_JOBS=1
for arm in counter both; do
    entry=both.rs
    if [[ "$arm" == counter ]]; then entry=counter_edge.rs; fi
    entry="$scratch/source/research/astra-e/front-fusion/cap-counter/$entry"
    MOONSHOT_CANDIDATE="$entry" CARGO_TARGET_DIR="$scratch/$arm-build" cargo build --locked --release --manifest-path "$scratch/source/bench/moonshot/Cargo.toml" > "$scratch/$arm-build.log" 2>&1
done
