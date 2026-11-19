#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages/payload/slots/singleton
out=${1:?fresh output directory}
mkdir "$out"
python3 "$d/census.py" "$out"
if [[ -n ${PAGE_DEPS:-} ]]; then deps="$PAGE_DEPS"; else
  cargo build --locked --release --features moonshot --lib
  deps="${CARGO_TARGET_DIR:-target}/release/deps"
fi
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/census.rs" \
 --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/census"
echo '82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  corpora/wikidata.bz2' | sha256sum --check
"$out/census" corpora/wikidata.bz2 > "$out/census.csv"
