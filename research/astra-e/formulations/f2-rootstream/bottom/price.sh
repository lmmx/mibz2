#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2-rootstream/bottom"
form="$study/../.."
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
sed 's@../../f2c/second.rs@second.rs@' "$study/kernel.rs" > "$dest/kernel.rs"
for arm in leaf maximal both; do
    file=$arm
    if [[ $arm == both ]]; then file=candidate; fi
    cat "$study/$file.rs" "$form/f2c/price_impl.rs" > "$dest/$arm.rs"
done
sed 's@../f2c/second.rs@second.rs@' "$form/f2-scheduler/candidate.rs" > "$dest/refill.rs"
cat "$form/f2c/price_impl.rs" >> "$dest/refill.rs"
cp "$form/f2c/second.rs" "$study/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
