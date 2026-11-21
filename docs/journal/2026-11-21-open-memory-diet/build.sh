#!/bin/bash
# build.sh TREE NAME: the tree's lib tests with the diet harness, copied to NAME.bin.
# $DIET_SCRATCH/diet holds this directory's files, and diet/ref master's kernel as the
# in-process reference: master's src/open.rs (its `mod checks;` and `mod b5;` lines removed)
# as ref/mod.rs, and its src/open/endpoint.rs as ref/endpoint.rs. SCRATCH_RS picks the
# scratch accounting (scratch_master.rs, or scratch_outq.rs from the OutQ16 commit on).
# Then: python3 drive2.py t1 12 m0.bin w1.bin oq.bin (and t4), with MIBZ2_ADMITTED_BWT's
# blocks at ../wt-data/admitted-bwt from the driver (the six of data/admitted-bwt, each
# checked against manifest.json's SHA-256 first).
set -e
S=${DIET_SCRATCH:?the directory holding diet/ and the worktrees}
T=$S/$1
cp $S/diet/diet_harness.rs $T/src/open/diet_harness.rs
cp ${SCRATCH_RS:-$S/diet/scratch_master.rs} $T/src/open/diet_scratch.rs
cp $T/src/open.rs $S/diet/open.rs.keep
rm -rf $T/src/open/diet_ref; cp -r $S/diet/ref $T/src/open/diet_ref
printf '\n#[cfg(test)]\n#[allow(dead_code)]\nmod diet_ref;\n#[cfg(test)]\nmod diet_harness;\n#[cfg(test)]\nmod diet_scratch;\n#[cfg(test)]\nuse diet_scratch::{b5_plain, diet_parts, diet_scratch};\n' >> $T/src/open.rs
cd $T
bin=$(CARGO_TARGET_DIR=$S/diet/target cargo test --release --lib --no-run 2>&1 | tee $S/diet/build.log | grep -o '/tmp[^)]*mibz2-[0-9a-f]*' | tail -1) || true
cp $S/diet/open.rs.keep $T/src/open.rs
rm -r $T/src/open/diet_harness.rs $T/src/open/diet_scratch.rs $T/src/open/diet_ref
[ -n "$bin" ] || { tail -30 $S/diet/build.log; exit 1; }
cp $bin $S/diet/$2.bin
echo built $2.bin
