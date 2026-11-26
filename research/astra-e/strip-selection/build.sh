#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/strip-selection
out=${1:?fresh directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/prepare.py" "$out"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/strip"
{ rustc -vV; echo "source badb2aec3624bd65606c429b2e6f79e9d9d0050f"; sha256sum "$out"/{main.rs,stock.rs,changed.rs,strip} "$deps/libmibz2.rlib"; } > "$out/manifest.txt"
