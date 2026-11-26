#!/usr/bin/env bash
# The first three arguments identify an input, a unique result directory and a writable build location.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
data=${1:-corpora/wikidata.bz2}
out=${2:?new result directory required}
build=${3:-target/astra-e-narrow-stage}
mkdir "$out"
mkdir -p "$build"
d=research/astra-e/narrow-short
python3 "$d/prepare.py"
python3 "$d/stage.py"
expected=82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
observed=$(sha256sum "$data")
test "${observed%% *}" = "$expected"
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat -L "dependency=$deps" \
    --extern "mibz2=$deps/libmibz2.rlib" "$d/stage.rs" -o "$build/stage"
{
    git rev-parse HEAD
    rustc -Vv
    sha256sum "$data" "$d/stage.rs" "$d/constructor.rs" "$d/constructor_ahead.rs" "$d/narrow.rs" "$build/stage"
    cat /sys/fs/cgroup/cpu.max
    taskset -pc $$
    echo "11 pinned real blocks; prepared runs; 21 rounds; one untimed warm pass; 7 arms"
} > "$out/manifest.txt"
"$build/stage" "$data" 21 > "$out/laps.csv"
python3 research/astra-e/stage2/report.py "$out/laps.csv" > "$out/summary.csv"
