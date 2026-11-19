#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/second"
tmp=${1:?scratch directory}
deps=${2:?absolute release/deps}
graph=${3:?output raw csv file}
python3 "$d/profile.py" "$tmp"
cp "$d/profile_main.rs" "$tmp/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$tmp/profile_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/profile"
"$tmp/profile" "$root/corpora/wikidata.bz2" "$graph"
