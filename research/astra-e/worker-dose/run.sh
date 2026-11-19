#!/usr/bin/env bash
# From the repository's designated detached research checkout containing the frozen dose.
set -euo pipefail
checker=$(cd "$(dirname "$0")" && pwd)
corpus=$(realpath "$1"); out=$(realpath -m "$2"); build=$(realpath -m "$3")
mkdir "$out"; mkdir -p "$build"
git diff --quiet HEAD -- bench/moonshot .just Justfile
git fetch origin master
git checkout origin/master -- bench/moonshot .just Justfile
{
    git rev-parse HEAD origin/master
    git log -1 --format=%h origin/master -- bench/moonshot
    rustc -Vv
    cat /sys/fs/cgroup/cpu.max
    sha256sum "$corpus"
} > "$out/manifest.txt"
export CARGO_TARGET_DIR="$build/target"
for arm in dose observer edge; do
    case $arm in
        dose) entry=research/astra-e/worker-dose/candidate.rs;;
        observer) entry=research/astra-e/worker-dose/observe.rs;;
        edge) entry=research/astra-e/edge-budget/candidate.rs;;
    esac
    bash research/astra-e/prepare.sh "$entry"
    export MOONSHOT_CANDIDATE="$(realpath "$entry")"
    cargo build --quiet --release --locked --manifest-path bench/moonshot/Cargo.toml
    cp "$CARGO_TARGET_DIR/release/moonshot" "$build/$arm"
    sha256sum "$entry" "$build/$arm" >> "$out/manifest.txt"
done
for w in 1 2 4 7 8 9 10; do
    "$build/observer" run --arm candidate --file "$corpus" --workers "$w" > "$out/w$w.json" 2> "$out/w$w.log"
done
python3 "$checker/check_roster.py" "$out" > "$out/roster.csv"
for arm in lazy cells candidate; do
    "$build/edge" check --arm "$arm" --quick --wikidata "$corpus" > "$out/gate-$arm.jsonl"
done
python3 "$checker/panel.py" "$build/edge" "$build/dose" "$corpus" "$out"
