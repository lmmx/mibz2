#!/usr/bin/env bash
set -euo pipefail
d=$(dirname "$0")
out=${1:?absolute scratch output}
deps=${2:?release dependencies}
mkdir -p "$out"
python3 -B "$d/../../../clipped/native/generate.py" "$out"
python3 - "$out" "$d" <<'PY'
import hashlib,sys
from pathlib import Path
out,src=map(Path,sys.argv[1:])
for n,want in (("shared.rs","b0377bd9d5fdfaa9c8a8e3c17a84fae24dab9b8f45ddb02bbf922e12c1e2a1b2"),
               ("second.rs","60c828228d27397676d822cede82d2e1033e0af6507184572782a0dc10edc80b")):
    assert hashlib.sha256((out/n).read_bytes()).hexdigest()==want
with (out/"shared.rs").open("a") as f:f.write((src/"bridge.rs").read_text())
(out/"libcore.rs").write_text("pub use mibz2::{bwt,runs,Kernels};\n#[allow(dead_code,unused_imports)] mod shared;\n")
PY
rustc --edition=2021 --crate-type staticlib -A unexpected_cfgs -C opt-level=3 -C codegen-units=1 -C panic=abort "$out/libcore.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/libcore.a"
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic "$d/observed.cpp" "$out/libcore.a" -ldl -lpthread -lm -o "$out/observed"
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic "$d/memory.cpp" "$out/libcore.a" -ldl -lpthread -lm -o "$out/memory"
