#!/bin/sh
# Miri over the union of the components' registered classes, one class per process, one
# after another (nightly 1.101.0 16d1ffac1, Stacked Borrows, no flags, unpoisoned arenas;
# 7,000 s timeout each: a timeout is partial coverage, never a PASS).
# usage: miri.sh OUT_DIR [CLASS ...]   (from the worktree root)
OUT=$1; shift
export CARGO_TARGET_DIR=${MIRI_TARGET:-target/miri}
cargo +nightly miri --version > $OUT/toolchain.txt; rustc +nightly -vV >> $OUT/toolchain.txt
CLASSES=${*:-"scanner:tiny scanner:tails scanner:align scanner:phase scanner:held scanner:endrun scanner:epoch scanner:malformed scanner:controls descriptor:tiny descriptor:slots descriptor:allbytes descriptor:crossing descriptor:runs descriptor:epoch descriptor:pending descriptor:malformed descriptor:reuse v3:small v3:tag-tiny v3:span v3:tag-epoch1"}
for c in $CLASSES; do
  f=$OUT/miri-$(echo $c | tr ':' '-').console
  t0=$(date +%s)
  timeout 7000 cargo +nightly miri run --example c30walk -- ctor2c-synthetic --miri $c > $f 2>&1
  e=$?
  echo "class $c exit $e seconds $(( $(date +%s) - t0 ))" | tee -a $OUT/summary.txt
done
echo DONE >> $OUT/summary.txt
