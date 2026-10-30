#!/usr/bin/env bash
# The level-aware plan's two diagnostics (docs/journal/2026-10-29-level-aware-plan.md, "The
# diagnostics"): not shipping gates, and no design is changed by them.
#
# 1. The decoder's own timeline (examples/occupancy.rs, the `trace` feature) on -3 then -9 at
#    all CPUs, under today's plan (master), revision 1 (00e9dd6: the gate before the
#    inversion) and revision 2 (this branch: the gate when a worker takes the block): where
#    each design's time goes, the reader's waits and the blocks held behind the head of
#    the line among them.
# 2. An A/A on master: master's own `ab`, `mibz2` against `mibz2-auto` on -6, the arms and
#    rounds of the A/B, so a gap between the two arms that predates #115 shows as one.
#
# Each build is made in a worktree of its own (target/diagnose-tree-*) and removed after.
# THREADS (default all CPUs), ROUNDS (the timeline's, default 6) and AA_ROUNDS (default 20)
# override the defaults. It writes diagnose-<UTC time>-*.json and .jsonl here.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
root=$PWD
threads=${THREADS:-$(nproc)}
rounds=${ROUNDS:-6}
aa_rounds=${AA_ROUNDS:-20}
stamp=$(date -u +%Y%m%dT%H%M%SZ)
bash bench/corpus.sh
for l in 3 6 9; do
  test -s corpora/wikidata-bz$l.bz2 || (bzip2 -dc corpora/wikidata.bz2 | bzip2 -$l > corpora/wikidata-bz$l.bz2)
done
cat corpora/wikidata-bz3.bz2 corpora/wikidata-bz9.bz2 > corpora/wikidata-mix39.bz2
git fetch -q origin master
bins=$root/target/diagnose
mkdir -p "$bins"

# build LABEL REF EXAMPLE [FEATURES]: the example at REF, copied to target/diagnose/LABEL-EXAMPLE
build() {
  local tree=$root/target/diagnose-tree-$1
  git worktree remove --force "$tree" 2> /dev/null || true
  git worktree add -q --detach "$tree" "$2"
  (cd "$tree" && CARGO_TARGET_DIR=$root/target/diagnose-build cargo build -q --release --example "$3" ${4:+--features "$4"})
  cp "$root/target/diagnose-build/release/examples/$3" "$bins/$1-$3"
  git worktree remove --force "$tree"
  echo "diagnose: built $1 ($(git rev-parse --short "$2")) $3"
}
build today origin/master occupancy trace
build rev1 00e9dd6 occupancy trace
build rev2 HEAD occupancy trace
build master origin/master ab

# 1. the timelines
for label in today rev1 rev2; do
  "$bins/$label-occupancy" corpora/wikidata-mix39.bz2 --threads "$threads" --rounds "$rounds" \
    --out "diagnose-$stamp-occupancy-$label.json" --meta "build=$label" \
    --meta "commit=$(git rev-parse --short "$([ $label = today ] && echo origin/master || ([ $label = rev1 ] && echo 00e9dd6 || echo HEAD))")" \
    --meta "corpus=wikidata-mix39"
done

# 2. the A/A on master: its `mibz2` (BzReader::parallel) against its `mibz2-auto`
"$bins/master-ab" run "diagnose-$stamp-aa6.jsonl" --arms mibz2-workers,mibz2-auto,mibz2 \
  --threads "4,6,8,10,$threads" --rounds "$aa_rounds" --settle 0 corpora/wikidata-bz6.bz2
python3 bench/level_aware.py --no-gates "diagnose-$stamp-aa6.jsonl" | tee "diagnose-$stamp-aa6-scores.md"
echo "diagnose: wrote diagnose-$stamp-*: upload them all, and the console"
