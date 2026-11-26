#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2-rootstream"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
for arm in leaf frames refill; do
    case "$arm" in
        leaf) candidate="$study/leaf/candidate.rs" ;;
        frames) candidate="$study/frames/candidate.rs" ;;
        refill) candidate="$study/../f2-scheduler/candidate.rs" ;;
    esac
    sed -E 's@"(\.\./)+f2c/second.rs"@"second.rs"@' "$candidate" > "$dest/$arm.rs"
    cat "$study/../f2c/price_impl.rs" >> "$dest/$arm.rs"
done
cp "$study/../f2c/second.rs" "$study/frames/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
