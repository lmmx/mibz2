#!/bin/bash
# segcost's heavy work, in one hold of the shared build lock: the driver's build, the
# corpus's block sizes, callgrind (Ir, branch and cache simulation) of master's room walk at
# 512 / 2048 / 16384 / 131072 segments and of the lean prototype, then the trace simulations.
S="$(cd "$(dirname "$0")" && pwd)"  # place the bench corpus (sha 82f65ace) at $S/corp/wikidata.bz2
export CARGO_TARGET_DIR=$S/target
O=$S/out
mkdir -p $O
cd $S/drv && cargo build --release > $O/build.log 2>&1 || { echo BUILD FAILED; tail -30 $O/build.log; exit 1; }
B=$CARGO_TARGET_DIR/release/segcost
W=$S/corp/wikidata.bz2
$B sizes $W > $O/sizes.txt
CG="valgrind --tool=callgrind --toggle-collect=*measured* --cache-sim=yes --branch-sim=yes --D1=32768,8,64 --I1=32768,8,64"
# the full block 3 (n 900,000) after full blocks 1 and 2 at the same setting (the sweep's state)
for segs in 512 2048 16384 131072; do
  $CG --LL=524288,8,64 --callgrind-out-file=$O/cg-m$segs-w2-512K.out $B cg $W 3 $segs 2 > $O/cg-m$segs-w2-512K.stdout 2> $O/cg-m$segs-w2-512K.console
done
# cold scratch (the first block a SegScratch walks)
for segs in 512 2048; do
  $CG --LL=524288,8,64 --callgrind-out-file=$O/cg-m$segs-w0-512K.out $B cg $W 3 $segs 0 > $O/cg-m$segs-w0-512K.stdout 2> $O/cg-m$segs-w0-512K.console
done
# a 1 MiB "L2" for the two settings that matter
for segs in 512 2048; do
  $CG --LL=1048576,16,64 --callgrind-out-file=$O/cg-m$segs-w2-1M.out $B cg $W 3 $segs 2 > $O/cg-m$segs-w2-1M.stdout 2> $O/cg-m$segs-w2-1M.console
done
# a tiny block (block 4, n 62) after block 3: what the segment count costs a block that cannot hold it
for segs in 512 2048; do
  $CG --LL=524288,8,64 --callgrind-out-file=$O/cg-m$segs-tiny.out $B cg $W 4 $segs 1 > $O/cg-m$segs-tiny.stdout 2> $O/cg-m$segs-tiny.console
done
# the lean prototype, K = 8 / 16 / 32
for bits in 2 3 4 5; do
  $CG --LL=524288,8,64 --callgrind-out-file=$O/cg-lean$bits-w2-512K.out $B lean $W 3 $bits 2 > $O/cg-lean$bits-w2-512K.stdout 2> $O/cg-lean$bits-w2-512K.console
done
# a second block for the two master settings and lean K = 8 (block 41, n 900,000)
for segs in 512 2048; do
  $CG --LL=524288,8,64 --callgrind-out-file=$O/cg-m$segs-b41-512K.out $B cg $W 41 $segs 2 > $O/cg-m$segs-b41-512K.stdout 2> $O/cg-m$segs-b41-512K.console
done
$CG --LL=524288,8,64 --callgrind-out-file=$O/cg-lean3-b41-512K.out $B lean $W 41 3 2 > $O/cg-lean3-b41-512K.stdout 2> $O/cg-lean3-b41-512K.console
echo CG DONE
