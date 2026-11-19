#!/usr/bin/env bash
# Same timer and verified stage as phase.sh, stopping after exactly one reduction.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
d=research/astra-e/formulations/f4
digest=$(sha256sum corpora/wikidata.bz2)
test "${digest%% *}" = 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
python3 "$d/phase.py"
python3 - <<'PY'
from pathlib import Path
d = Path("research/astra-e/formulations/f4/target")
s = (d/"instrumented.rs").read_text()
assert s.count("for level in 0..8") == 1
(d/"instrumented_one.rs").write_text(s.replace("for level in 0..8", "for level in 0..1"))
s = (d/"phase.rs").read_text().replace("instrumented.rs", "instrumented_one.rs")
(d/"one.rs").write_text(s)
PY
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat -L "dependency=$deps" \
  --extern "mibz2=$deps/libmibz2.rlib" "$d/target/one.rs" -o "$d/target/one"
{ git rev-parse HEAD; rustc -Vv; sha256sum corpora/wikidata.bz2 "$d"/{candidate.rs,phase.py,one.sh} "$d/target/one"; } >&2
"$d/target/one" corpora/wikidata.bz2 9
