#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages/payload
out=${1:?fresh scratch directory}
mkdir "$out"
out=$(cd "$out" && pwd)
cpu1=${2:-4}
cpu2=${3:-2}
python3 "$d/prepare.py"
python3 "$d/harness.py" "$out"
if [[ -n ${PAGE_DEPS:-} ]]; then deps="$PAGE_DEPS"; else
  cargo build --locked --release --features moonshot --lib
  deps="${CARGO_TARGET_DIR:-target}/release/deps"
fi
for bin in price check; do
  args=()
  [[ $bin != check ]] || args=(--test)
  rustc "${args[@]}" --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/focus.rs" \
    --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/$bin"
done
corpus=corpora/wikidata.bz2
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $corpus" | sha256sum --check
{
  git rev-parse HEAD
  rustc -vV
  sha256sum "$out"/{price,focus.rs,control.rs,starts.rs,ends.rs} "$deps/libmibz2.rlib"
  echo "pinned CPUs $cpu1 $cpu2"
  cat /sys/fs/cgroup/cpu.max
} > "$out/manifest.txt"
"$out/check" --nocapture > "$out/small.txt"
"$out/price" "$corpus" geometry > "$out/geometry.csv"
nice -n 8 taskset -c "$cpu1" "$out/price" "$corpus" price > "$out/first.csv"
nice -n 8 taskset -c "$cpu2" "$out/price" "$corpus" price > "$out/repeat.csv"
