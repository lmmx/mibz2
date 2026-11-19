#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/crc-phrases/followup
out=${1:?fresh scratch directory}
cpu=${2:-4}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/prepare.py" "$out"
corpus=corpora/wikidata.bz2
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $corpus" | sha256sum --check
python3 - "$corpus" "$out/oracle" <<'PY'
import bz2,sys,shutil
with bz2.open(sys.argv[1],'rb') as f,open(sys.argv[2],'wb') as g:shutil.copyfileobj(f,g)
PY
echo "3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf  $out/oracle" | sha256sum --check
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/price"
{ rustc -vV; echo "immutable combined 80d8409359214b4e5abfd4430213921e82e9cd30"; sha256sum "$out"/{main.rs,price} "$d/rounds.inc" "$d/prepare.py" "$deps/libmibz2.rlib"; echo "pin $cpu"; } > "$out/manifest.txt"
nice -n 10 taskset -c "$cpu" "$out/price" "$corpus" "$out/oracle" "$out"
