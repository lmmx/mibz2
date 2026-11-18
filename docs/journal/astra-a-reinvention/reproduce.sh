#!/usr/bin/env bash
# Run in the research checkout. Supply an empty scratch directory or omit the argument.
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
study="$repo/docs/journal/astra-a-reinvention"
scratch=${1:-$(mktemp -d)}
mkdir -p "$scratch"
scratch=$(cd "$scratch" && pwd)
out="$scratch/results"
columns="$scratch/columns"
plain="$scratch/plain"
records="$scratch/records"
traces="$scratch/lines"
target="$scratch/target"
mkdir -p "$out" "$records" "$traces" "$scratch/team"
cd "$repo"

pinned=1edb29fdc3d42dfddfc3d60e46456b611323760c
archive=2e4b34e086ac95e8218f1ee822a4652c8db5bb19
windows=a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a
for object in "$pinned" "$archive" "$windows"; do
    git cat-file -e "$object^{commit}" 2>/dev/null || git fetch --no-tags origin "$object"
done
git diff --quiet "$pinned" -- src examples/induction_trace.rs bench/irec.py
git archive "$windows" windows/wikidata-20261019 | tar -x -C "$scratch"
git archive "$archive" wikidata | tar -x -C "$records"
python3 "$study/input.py" "$scratch/windows/wikidata-20261019" "$scratch"
CARGO_TARGET_DIR="$target" cargo build --release --locked --lib --features induction-trace --example induction_trace
rustc --edition 2021 -O "$study/extract.rs" --extern "mibz2=$target/release/libmibz2.rlib" \
    -L "dependency=$target/release/deps" -o "$scratch/extract"
"$scratch/extract" "$scratch/wikidata.bz2" "$columns" 1 "$plain" > "$out/input.csv"
awk -F, 'NR==1 || $1%24==0' "$out/input.csv" > "$scratch/sample.csv"
awk -F, 'NR==1 || $1%24==0 || $1==1 || $1==2' "$out/input.csv" > "$scratch/pricing.csv"

python3 "$study/early.py" "$columns" "$scratch/sample.csv" "$out/screens.csv" "$out/levels.csv"
head -2 "$out/screens.csv" > "$out/frozen.csv"
awk -F, 'NR==1 || $1==0' "$out/levels.csv" > "$out/frozen-levels.csv"
python3 "$study/pairs.py" "$columns" "$out/screens.csv" > "$out/pairs.csv"
python3 "$study/pairs.py" small
python3 "$study/controls.py" "$columns/0000.L" 264045 > "$out/controls.json"
python3 "$study/check.py" > "$out/small.json"
for arm in shortcut whole; do
    flags=()
    if [[ $arm == whole ]]; then flags=(--whole); fi
    python3 "$study/measure.py" "$columns" "$out/input.csv" "$scratch/checksums.csv" \
        "$out/$arm.csv" --plain "$plain" "${flags[@]}"
    head -2 "$out/$arm.csv" > "$out/frozen-$arm.csv"
    python3 "$study/measure.py" "$columns" "$scratch/pricing.csv" "$scratch/checksums.csv" \
        "$scratch/priced-$arm.csv" --plain "$plain" --traces "$traces" "${flags[@]}"
done
"$target/release/examples/induction_trace" "$scratch/first-four.bz2" "$scratch/first-records" \
    > "$scratch/recorder.jsonl"
cp "$scratch/first-records/0001.irec" "$scratch/first-records/0002.irec" "$scratch/team"
python3 "$study/baseline.py" "$records/wikidata" "$traces"
python3 "$study/baseline.py" "$scratch/team" "$traces"
python3 "$study/ledger.py" "$columns" "$records/wikidata" "$out/operands.csv"
c++ -std=c++17 -O2 -Wall -Wextra -pedantic "$study/cache.cpp" -o "$scratch/cache"
python3 "$study/price.py" "$scratch/cache" "$traces" "$out"
python3 "$study/costs.py" --data "$out"
python3 "$study/verify.py" "$out" --cache "$scratch/cache" --historical-frozen "$records/wikidata/0000.irec.xz"
for receipt in "$study"/evidence/*; do
    cmp "$receipt" "$out/$(basename "$receipt")"
done
echo "All committed receipts reproduced in $out"
