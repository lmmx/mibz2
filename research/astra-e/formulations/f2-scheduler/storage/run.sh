#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/storage"
tmp=${1:?scratch}
deps=${2:?release deps}
python3 "$d/account.py" "$tmp"
cp "$d/probe.rs" "$tmp/"
rustc --edition=2021 -C opt-level=2 "$tmp/probe.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/probe"
"$tmp/probe" "$root/corpora/wikidata.bz2"
