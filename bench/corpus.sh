#!/usr/bin/env bash
# The occupancy instrument's build and the benchmark's corpus (bench/wikidata.json), as
# bench/occupancy.sh and bench/imc.py use them: the examples built, and
# corpora/wikidata.bz2 built here once and checked by its SHA-256.
# Needs: cargo, python3, git and lbzip2 (the corpus's blocks are checked against it).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
for tool in cargo python3 git lbzip2; do
  command -v "$tool" > /dev/null || { echo "bench: needs $tool on PATH"; exit 1; }
done

cargo build --release --features trace --example occupancy --example windows

# The corpus:
# its windows from windows/, else the bench-data branch, else the dated dump itself
spec=bench/wikidata.json
corpus=corpora/wikidata.bz2
field() { python3 -c "import json, sys; print(json.load(open('$spec'))[sys.argv[1]])" "$1"; }
sha() { python3 -c "import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], 'rb').read()).hexdigest())" "$1"; }
want=$(field corpus_sha256)
if [ ! -f "$corpus" ] || [ "$(sha "$corpus")" != "$want" ]; then
  dir=$(field windows_dir)
  if [ "$(ls "$dir"/*.bin 2> /dev/null | wc -l)" = 0 ] && git fetch -q origin bench-data:refs/remotes/origin/bench-data 2> /dev/null; then
    git archive origin/bench-data "$dir" | tar -x || true
  fi
  python3 bench/wikidata.py fetch "$spec" "$dir" > /dev/null
  mkdir -p corpora
  python3 bench/wikidata.py build "$spec" "$dir" target/release/examples/windows "$corpus"
fi
