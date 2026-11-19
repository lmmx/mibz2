#!/usr/bin/env bash
set -euo pipefail
study=$(dirname "$0")
deps=${1:?absolute release/deps with libmibz2}
dest=${2:?scratch executable}
python3 "$study/prepare.py"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$study/probe.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest"
"$dest" corpora/wikidata.bz2
