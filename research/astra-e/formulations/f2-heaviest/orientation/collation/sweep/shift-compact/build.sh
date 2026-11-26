#!/usr/bin/env bash
set -euo pipefail
d=$(dirname "$0");out=${1:?absolute scratch directory};deps=${2:?absolute release deps}
python3 -B "$d/generate.py" "$out"
rustc --edition=2021 -A unexpected_cfgs -A unused_imports -C opt-level=3 -C codegen-units=1 -C debug-assertions=yes --cfg clipped_profile --cfg shift_counts "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/check"
rustc --edition=2021 -A unexpected_cfgs -A unused_imports -C opt-level=3 -C codegen-units=1 --cfg clipped_profile "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/price"
