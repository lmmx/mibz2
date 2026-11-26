#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages/payload/slots/path-writes
out=${1:?fresh scratch directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/count.py" "$out"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --edition=2021 --cfg path_guard_check -C opt-level=3 -C codegen-units=1 "$out/focus.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/check-count"
echo "block,arm,checked_ordinary,unchecked_ordinary,growing_ordinary"
"$out/check-count" corpora/wikidata.bz2 geometry
