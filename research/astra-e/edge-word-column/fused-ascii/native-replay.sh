#!/usr/bin/env bash
set -eu -o pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
cd "$root"
out=${1:?supply a temporary output directory}
mkdir -p "$out"
out=$(cd "$out" && pwd)
mode=${2:-check}
oracle=${BZIP2_ORACLE:-"$out/wikidata.raw"}
if [[ ! -f "$oracle" ]]; then
    bzip2 -dc corpora/wikidata.bz2 > "$oracle"
fi
printf '%s  %s\n' 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf "$oracle" | sha256sum -c --status
case "$mode" in
    check)
        python3 -B "$here/build.py" "$out"
        rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$here/property.rs" -o "$out/property"
        "$out/property" > "$out/property.txt"
        "$out/price" adversarial > "$out/adversarial.txt"
        "$out/price" check corpora/wikidata.bz2 "$oracle" > "$out/checked.csv" 2>"$out/checked.txt"
        ;;
    price)
        "$out/price" price corpora/wikidata.bz2 "$oracle" 24 > "$out/paid.csv"
        ;;
    memory)
        python3 research/astra-e/edge-displacements/memory.py "$out/price" corpora/wikidata.bz2 > "$out/memory.csv"
        ;;
    *)
        exit 2
        ;;
esac
