#!/usr/bin/env bash
# Create isolated revision-pinned builds. Run once in a fresh output directory.
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
out=$(realpath -m "$1")
rev=01b15841c09bef243e0123f0bd0ebcabcbc5d2d5
mkdir "$out"
for arm in control candidate; do
    src="$out/$arm"
    git -C "$root" worktree add --quiet --detach "$src" "$rev"
    (cd "$src" && git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile)
    git -C "$src" log -1 --format=%h origin/master -- bench/moonshot
    (cd "$src" && python3 research/astra-e/edge-budget/prepare.py)
    if [[ $arm == candidate ]]; then python3 "$d/generate.py" "$src"; fi
    MOONSHOT_CANDIDATE="$src/research/astra-e/edge-budget/candidate.rs" CARGO_TARGET_DIR="$out/$arm-build" \
        cargo build --locked --release --manifest-path "$src/bench/moonshot/Cargo.toml"
done
for name in check scanner; do
    rustc -O --edition=2021 "$d/$name.rs" --extern mibz2="$out/candidate-build/release/deps/libmibz2.rlib" \
        -L "dependency=$out/candidate-build/release/deps" -o "$out/$name"
done
