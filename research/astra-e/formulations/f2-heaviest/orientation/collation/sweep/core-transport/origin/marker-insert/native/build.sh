#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=${1:?absolute scratch output}
deps=${2:?absolute release library dependencies}
mkdir -p "$out"
python3 -B "$d/generate.py" "$out"
rustc --edition=2021 -A unexpected_cfgs -A unused_imports -C opt-level=3 -C codegen-units=1 --cfg clipped_profile "$out/main.rs" --extern "mibz2=$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/marker"
