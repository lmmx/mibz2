#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
tmp=${1:?temporary directory}
deps=${2:?release/deps with moonshot library}
mkdir -p "$tmp"
python3 "$d/prepare.py"
python3 - "$d" "$tmp/check.rs" <<'PY'
from pathlib import Path
import sys
d=Path(sys.argv[1])
source=(d.parents[1]/"narrow-endpoint/check.rs").read_text()
a=source.index('#[cfg(')
b=source.index('fn reference')
source=source[:a]+f'#[path="{d.parent}/constructor_ahead.rs"] mod old;\n#[path="{d}/candidate.rs"] mod candidate;\n'+source[b:]
Path(sys.argv[2]).write_text(source)
PY
rustc --edition=2021 -C opt-level=2 --extern "mibz2=$deps/libmibz2.rlib" -L "dependency=$deps" "$tmp/check.rs" -o "$tmp/check"
nice -n 14 taskset -c "${3:-3}" "$tmp/check"
