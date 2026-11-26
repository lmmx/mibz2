#!/usr/bin/env bash
# Invoke from the repository, retaining stdout as laps.csv and stderr as manifest.txt.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
d=research/astra-e/stage2
mkdir -p "$d/target"
data=${1:-corpora/wikidata.bz2}
rounds=21
expected=82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
digest=$(sha256sum "$data")
test "${digest%% *}" = "$expected"
python3 "$d/prepare.py"
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat -L "dependency=$deps" \
  --extern "mibz2=$deps/libmibz2.rlib" "$d/experiment.rs" -o "$d/target/experiment"
{
  git rev-parse HEAD
  rustc -Vv
  sha256sum "$data" "$d/experiment.rs" "$d/target/experiment"
  echo "same 11 frozen ordinals as stage 1; 21 recorded passes, seven arms; one warm pass"
  echo "wall and Linux thread CPU; no worker pool, MTF front, CRC or finish in either clock"
  cat /sys/fs/cgroup/cpu.max
  taskset -pc $$
} >&2
"$d/target/experiment" "$data" "$rounds"
"$d/target/experiment" "$data" 0 probe 2> "$d/target/word-probe.txt"
python3 "$d/rules.py" "$d/target/word-probe.txt" > "$d/target/rules.csv"
