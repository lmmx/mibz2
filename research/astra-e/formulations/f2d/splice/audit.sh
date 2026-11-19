#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
cat "$study/f2c/candidate.rs" "$study/f2d/splice/merge.rs" "$study/f2d/splice/audit_impl.rs" > "$dest/candidate.rs"
cp "$study/f2c/second.rs" "$study/f2d/splice/audit.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/audit.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/audit"
"$dest/audit" "$root/corpora/wikidata.bz2"
