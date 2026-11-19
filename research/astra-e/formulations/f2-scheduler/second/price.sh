#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/second"
tmp=${1:?scratch directory}
deps=${2:?absolute release deps}
mkdir -p "$tmp"
for mode in scalar batch refill; do
    python3 "$d/prepare.py" "$mode" "$tmp/$mode"
    cat "$d/../../f2c/price_impl.rs" >> "$tmp/$mode/candidate.rs"
done
cp "$d/price_main.rs" "$tmp/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$tmp/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/price"
"$tmp/price" "$root/corpora/wikidata.bz2"
