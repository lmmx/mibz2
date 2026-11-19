#!/bin/bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=$(realpath -m "${1:?new scratch directory}")
lib=$(realpath "${2:?moonshot-enabled release libmibz2.rlib}")
corpus=$(realpath "${3:?Wikidata input}")
gold=$(realpath "${4:?libbzip2 plaintext}")
mkdir -p "$out/native" "$out/split"
python3 "$d/generate.py" "$out/native"
cp "$d/driver.rs" "$out/native/main.rs"
python3 "$d/attribution.py" "$out/split"
for sub in native split; do
 rustc -A unused --edition=2021 -C opt-level=3 -C codegen-units=1 --extern "mibz2=$lib" -L "dependency=$(dirname "$lib")" "$out/$sub/main.rs" -o "$out/$sub/test"
 "$out/$sub/test" exact "$corpus" "$gold" > "$out/$sub/exact.csv"
done
"$out/native/test" adverse "$corpus"
"$out/native/test" time "$corpus" 24 > "$out/panel.csv"
"$out/split/test" time "$corpus" 18 > "$out/split.csv"
"$out/split/test" memory "$corpus" > "$out/memory.csv"
