#!/bin/bash
# (the runs of docs/journal/2026-11-21-open-lanes.md; the corpus at $S/out/wikidata.bz2)
# stages.sh ROUNDS BUILD... : interleaved attrib runs, one thread, rotating order
S=${SCRATCH:?set SCRATCH to a scratch directory}
O=$S/out; R=$1; shift; B=("$@"); k=${#B[@]}
for r in $(seq 1 $R); do
  for i in $(seq 0 $((k-1))); do
    b=${B[$(( (i + r) % k ))]}
    taskset -c 2 $O/bin/attrib-$b $O/wikidata.bz2 5 blk > $O/blk-$b-$r.tsv || echo FAIL $b $r
  done
  echo round $r done
done
