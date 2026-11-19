#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
form="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps}
mkdir -p "$dest"
for arm in direct direct8 nested nested8; do
    case "$arm" in
        direct) source="$form/f2-word"; measure="$form/f2-word";;
        direct8) source="$form/f2-frontier-direct"; measure="$form/f2-word";;
        nested) source="$form/f2c"; measure="$form/f2c";;
        nested8) source="$form/f2-frontier"; measure="$form/f2c";;
    esac
    sed 's@../f2c/second.rs@second.rs@' "$source/candidate.rs" > "$dest/$arm.rs"
    cat "$measure/price_impl.rs" >> "$dest/$arm.rs"
done
cp "$form/f2c/second.rs" "$form/f2-frontier-direct/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
