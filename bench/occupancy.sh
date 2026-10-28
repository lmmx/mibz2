#!/usr/bin/env bash
# The occupancy instrument (#39, examples/occupancy.rs) on the pinned Wikidata corpus, as
# `just bench-occupancy [ARGS]` and CI's occupancy workflow both run it: one instrument, one
# set of defaults, whatever the machine. ARGS go to the instrument unchanged:
#   --threads 1,2,4,8   the thread counts (default 1, 2, 4, ... up to the CPUs allowed)
#   --rounds N          rounds (default 6: each mode first twice)
#   --slots N           the window (default 2 x threads, BzReader::parallel's)
#   --placements A/B    arms pinned to CPUs, paired by round: packed (SMT siblings first) or
#                       spread (one CPU a core first), from this machine's sibling map, or
#                       CPU lists; the whole process is pinned, scanner and reader included
#   --gap SECONDS       idle before each run (default none: a release decodes without
#                       pauses; on the i9-10900K a 0.15 s gap brings on slow runs, #39)
#   --out PATH          the result file (default occupancy-<UTC time>.json here)
# Needs: cargo, python3, git and lbzip2 (the corpus's blocks are checked against it).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
bash bench/corpus.sh
spec=bench/wikidata.json
corpus=corpora/wikidata.bz2
want=$(python3 -c "import json; print(json.load(open('$spec'))['corpus_sha256'])")

target/release/examples/occupancy "$corpus" \
  --out "occupancy-$(date -u +%Y%m%dT%H%M%SZ).json" \
  --meta "commit=$(git rev-parse HEAD)" \
  --meta "dirty=$(git status --porcelain --untracked-files=no | grep -q . && echo yes || echo no)" \
  --meta "rustc=$(rustc --version)" \
  --meta "corpus_sha256=$want" \
  "$@"
