#!/usr/bin/env bash
set -eu -o pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
cd "$root"
out=${1:?supply an empty or reproducible temporary output directory}
mkdir -p "$out"
out=$(cd "$out" && pwd)
lib=${EDGE_LIB:-"$out/vendor-build/libmibz2.rlib"}
if [[ ! -f "$lib" ]]; then
    python3 research/astra-e/edge-displacements/build.py "$out/vendor-build"
fi
edge=${EDGE_TAPE:-"$out/edges.bin"}
if [[ ! -f "$edge" ]]; then
    OPENBLAS_NUM_THREADS=1 python3 research/astra-e/edge-displacements/tape.py restore research/astra-e/edge-displacements/edges.select.xz "$edge"
fi
words=${WORDS_TAPE:-"$out/words.bin"}
if [[ ! -f "$words" ]]; then
    xz -dc research/astra-e/edge-spellings/words.bin.xz > "$words"
fi
verify() {
    printf '%s  %s\n' "$1" "$2" | sha256sum --check --status
}
verify 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915 corpora/wikidata.bz2
verify 670b97cbe1f6ba513df848ba68e428e9004a80bf74807254e507fb263ebf0697 "$edge"
verify 9195dc5b870bbf620af4e74a369b3b2b52e1ff454e514e1b265b61eeaeabad1b "$words"
rustc --edition 2021 -C opt-level=3 -C codegen-units=1 "$here/thin.rs" --extern "mibz2=$lib" -o "$out/thin"
"$out/thin" corpora/wikidata.bz2 "$out/thin.bin" "$out/union.bin"
verify bd4c7941987f3b64145549aae00445931f90f65e0e990ae1eb1677a4d35be9b7 "$out/thin.bin"
verify a8878e349bc47afba60069365730da275c2eec494ff7e9bddd98b8c15ce2b921 "$out/union.bin"
for exe in census extra union; do
    g++ -std=c++17 -O2 -Wall -Wextra "$here/$exe.cpp" -o "$out/$exe"
done
"$out/census" "$edge" "$words" "$out/thin.bin" "$out" 2>"$out/check.txt"
"$out/extra" "$edge" "$words" "$out/thin.bin" "$out" 2>"$out/extra-check.txt"
"$out/union" "$edge" "$words" "$out/thin.bin" "$out/union.bin" "$out" 2>"$out/union-check.txt"
g++ -std=c++20 -O2 -Wall -Wextra "$here/cuts.cpp" -o "$out/cuts"
"$out/cuts" "$edge" "$words" "$out/thin.bin" "$out/union.bin" >"$out/cuts.csv" 2>"$out/cuts-check.txt"
python3 "$here/small.py" > "$out/small.txt"
for artifact in "$here"/*.csv "$here"/{check,extra-check,union-check,cuts-check,small}.txt; do
    cmp "$artifact" "$out/$(basename "$artifact")"
done
