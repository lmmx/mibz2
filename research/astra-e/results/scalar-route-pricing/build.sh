#!/usr/bin/env bash
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
dest=${1:?new temporary workspace}
projection=${2:-3df195b10e9fb6f86f0c06ad9fad1feb992406d5}
mkdir -p "$dest"
export CARGO_BUILD_JOBS=1
for arm in edge scalar; do
  if [[ $arm == edge ]]; then
    pin=badb2aec3624bd65606c429b2e6f79e9d9d0050f
    entry=research/astra-e/edge-budget/candidate.rs
  else
    pin=d0a42980e896211906234c7679b38760ae44a46e
    entry=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
  fi
  wt=$dest/$arm
  git -C "$repo" worktree add --detach "$wt" "$pin" > "$dest/$arm-checkout.txt" 2>&1
  git -C "$wt" checkout "$projection" -- bench/moonshot .just Justfile
  python3 -B "$wt/${entry%/*}/prepare.py"
  MOONSHOT_CANDIDATE="$wt/$entry" CARGO_TARGET_DIR="$dest/$arm-build" cargo build --locked --release --manifest-path "$wt/bench/moonshot/Cargo.toml" > "$dest/$arm-build.txt" 2>&1
  "$dest/$arm-build/release/moonshot" info > "$dest/$arm-info.json"
done
