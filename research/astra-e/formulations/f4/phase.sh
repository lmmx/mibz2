#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
d=research/astra-e/formulations/f4
digest=$(sha256sum corpora/wikidata.bz2)
test "${digest%% *}" = 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
python3 "$d/phase.py"
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat -L "dependency=$deps" \
  --extern "mibz2=$deps/libmibz2.rlib" "$d/target/phase.rs" -o "$d/target/phase"
{ git rev-parse HEAD; rustc -Vv; sha256sum corpora/wikidata.bz2 "$d"/{candidate.rs,phase.py} "$d/target/phase"; } >&2
"$d/target/phase" corpora/wikidata.bz2 9
