#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler"
tmp=${1:?scratch directory}
deps=${2:?absolute release deps directory}
mkdir -p "$tmp"
sed 's@../f2c/second.rs@second.rs@' "$d/../f2-frontier/candidate.rs" > "$tmp/base.rs"
cat "$d/graph_impl.rs" >> "$tmp/base.rs"
cp "$d/../f2c/second.rs" "$d/graph_main.rs" "$tmp"
rustc --edition=2021 -C opt-level=2 "$tmp/graph_main.rs" \
   --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/graph"
"$tmp/graph" "$root/corpora/wikidata.bz2" | xz -6 -c
