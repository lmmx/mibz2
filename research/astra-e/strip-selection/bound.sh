#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/strip-selection
out=${1:?new directory};mkdir "$out"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --edition=2021 -C opt-level=3 "$d/gap_bound.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/bound"
"$out/bound" small > "$out/finite.txt"
nice -n 16 taskset -c 4 "$out/bound" corpora/wikidata.bz2 "$d/train.csv" "$d/held.csv" > "$out/gaps.csv"
