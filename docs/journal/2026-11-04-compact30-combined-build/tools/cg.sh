#!/bin/sh
# each builder alone in its own process; order v2 run 1, c run 1, v2 run 2, c run 2
set -e
B=$1; OUT=$2; C=$3
for R in 1 2; do for W in v2 c; do
  valgrind --tool=callgrind --cache-sim=yes --dump-instr=yes --callgrind-out-file=$OUT/only-$W-run$R.out \
    $B ctor2c-count --only $W $C > $OUT/only-$W-run$R.stdout 2> $OUT/only-$W-run$R.console
  gzip -9 -f $OUT/only-$W-run$R.out
  mv $OUT/only-$W-run$R.out.gz $OUT/only-$W-run$R.callgrind.gz
done; done
echo DONE
