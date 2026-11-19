#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages/payload/slots/gather
out=${1:?fresh scratch directory for the edge test}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/harness.py" "$out"
cat "$d/edge-test.rs" >> "$out/slots.rs"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --test --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/focus.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/edge"
"$out/edge" mixed_full_lanes --nocapture
