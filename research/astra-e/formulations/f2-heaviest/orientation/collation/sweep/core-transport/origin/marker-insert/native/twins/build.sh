#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=${1:?scratch directory}; deps=${2:?release library dependencies}
mkdir -p "$out"
python3 -B "$d/../generate.py" "$out"
python3 - "$out" "$d" <<'PY'
from pathlib import Path
import sys
out,d=map(Path,sys.argv[1:])
p=out/'main.rs';s=p.read_text();assert s.count('fn main(){')==1
p.write_text(s[:s.index('fn verify_sparse(')]+(d/'main.rs').read_text())
PY
rustc --edition=2021 -A dead_code -A unexpected_cfgs -A unused_imports -C opt-level=3 -C codegen-units=1 --cfg clipped_profile "$out/main.rs" --extern "mibz2=$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/twins"
