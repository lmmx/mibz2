#!/usr/bin/env bash
# The occupancy instrument (#39, examples/occupancy.rs) on the pinned Wikidata corpus, as
# `just bench-occupancy [ARGS]` and CI's occupancy workflow both run it: one instrument, one
# set of defaults, whatever the machine. ARGS go to the instrument unchanged:
#   --threads 1,2,4,8   the thread counts (default 1, 2, 4, ... up to the CPUs allowed)
#   --rounds N          rounds (default 6: each mode first twice)
#   --slots N           the window (default 2 x threads, BzReader::parallel's)
#   --out PATH          the result file (default occupancy-<UTC time>.json here)
# Needs: cargo, python3, git and lbzip2 (the corpus's blocks are checked against it).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
for tool in cargo python3 git lbzip2; do
  command -v "$tool" > /dev/null || { echo "bench-occupancy: needs $tool on PATH"; exit 1; }
done

cargo build --release --features trace --example occupancy --example windows

# The benchmark's corpus (bench/wikidata.json), built here once and checked by its SHA-256:
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

target/release/examples/occupancy "$corpus" \
  --out "occupancy-$(date -u +%Y%m%dT%H%M%SZ).json" \
  --meta "commit=$(git rev-parse HEAD)" \
  --meta "dirty=$(git status --porcelain --untracked-files=no | grep -q . && echo yes || echo no)" \
  --meta "rustc=$(rustc --version)" \
  --meta "corpus_sha256=$want" \
  "$@"
