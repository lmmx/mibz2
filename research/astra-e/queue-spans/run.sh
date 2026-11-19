#!/usr/bin/env bash
# From the research tree, pass a fresh output directory and optionally two allowed CPU IDs.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out=${1:?new output directory required}
cpu1=${2:-$(python3 -c 'import os; a=sorted(os.sched_getaffinity(0)); print(a[min(3,len(a)-1)])')}
cpu2=${3:-$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')}
mkdir "$out"
out=$(cd "$out" && pwd)
d=research/astra-e/queue-spans
corpus=corpora/wikidata.bz2
sha=82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
echo "$sha  $corpus" | sha256sum --check
# The separate preserved arm is built from its original generated source.
python3 research/astra-e/narrow-short/prepare.py
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
opts=(--edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat -L "dependency=$deps" --extern "mibz2=$deps/libmibz2.rlib")
rustc "${opts[@]}" "$d/check.rs" -o "$out/check"
rustc "${opts[@]}" --cfg astra_queue_stats "$d/ledger.rs" -o "$out/ledger"
rustc "${opts[@]}" "$d/stage.rs" -o "$out/stage"
{
  git rev-parse HEAD
  rustc -vV
  cat /sys/fs/cgroup/cpu.max
  echo "first and repeat pinned CPUs $cpu1 $cpu2"
  sha256sum "$corpus" "$d/"*.rs "$deps/libmibz2.rlib" "$out/check" "$out/ledger" "$out/stage"
} > "$out/manifest.txt"
"$out/check" > "$out/check.txt"
"$out/ledger" "$corpus" > "$out/ledger.csv"
taskset -c "$cpu1" "$out/stage" "$corpus" 28 > "$out/first.csv"
taskset -c "$cpu2" "$out/stage" "$corpus" 28 > "$out/repeat.csv"
python3 "$d/report.py" "$out/first.csv" "$out/repeat.csv" > "$out/summary.csv"
