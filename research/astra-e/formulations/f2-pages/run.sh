#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages
dest=${1:?fresh output directory}
mkdir "$dest"
dest=$(cd "$dest" && pwd)
cpu1=${2:-3}
cpu2=${3:-0}
python3 "$d/prepare.py"
python3 "$d/harness.py" "$dest"
corpus=corpora/wikidata.bz2
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $corpus" | sha256sum --check
if [[ -n ${PAGE_DEPS:-} ]]; then
  deps="$PAGE_DEPS"
else
  cargo build --locked --release --features moonshot --lib
  deps="${CARGO_TARGET_DIR:-target}/release/deps"
fi
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
{
  git rev-parse HEAD
  rustc -vV
  sha256sum "$dest/"*.rs "$dest/price" "$deps/libmibz2.rlib" "$corpus"
  echo "pinned CPUs $cpu1 $cpu2"
  cat /sys/fs/cgroup/cpu.max
} > "$dest/manifest.txt"
rustc --test --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/check"
"$dest/check" layout::checks::arbitrary_marks --nocapture > "$dest/small.txt"
"$dest/price" "$corpus" geometry > "$dest/geometry.csv"
nice -n 12 taskset -c "$cpu1" "$dest/price" "$corpus" price > "$dest/first.csv"
nice -n 12 taskset -c "$cpu2" "$dest/price" "$corpus" price > "$dest/repeat.csv"
