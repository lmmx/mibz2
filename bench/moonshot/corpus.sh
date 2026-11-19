#!/usr/bin/env bash
# The Wikidata bench corpus the scorer reads, built once at OUT (default corpora/wikidata.bz2)
# and checked by its SHA-256: bench/corpus.sh's corpus step alone (the 12 dump windows from the
# bench-data branch, their whole blocks kept by examples/windows.rs, built in CARGO_TARGET_DIR,
# default target/). Needs cargo, python3, git and lbzip2 (examples/windows checks the blocks
# against it).
#   bash bench/moonshot/corpus.sh [OUT]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
for tool in cargo python3 git lbzip2; do
  command -v "$tool" > /dev/null || { echo "moonshot corpus: $tool is not installed"; exit 1; }
done
out=${1:-corpora/wikidata.bz2}
spec=bench/wikidata.json
want=$(python3 -c "import json; print(json.load(open('$spec'))['corpus_sha256'])")
have() { [ -f "$out" ] && [ "$(sha256sum "$out" | cut -d' ' -f1)" = "$want" ]; }
if ! have; then
  dir=$(python3 -c "import json; print(json.load(open('$spec'))['windows_dir'])")
  if [ "$(ls "$dir"/*.bin 2> /dev/null | wc -l)" = 0 ]; then
    git fetch -q origin bench-data:refs/remotes/origin/bench-data 2> /dev/null \
      && git archive origin/bench-data "$dir" | tar -x || true
  fi
  python3 bench/wikidata.py fetch "$spec" "$dir" > /dev/null
  cargo build -q --release --example windows
  mkdir -p "$(dirname "$out")"
  python3 bench/wikidata.py build "$spec" "$dir" "${CARGO_TARGET_DIR:-target}/release/examples/windows" "$out"
fi
have || { echo "moonshot corpus: $out is not the bench corpus ($want)"; exit 1; }
echo "$out"
