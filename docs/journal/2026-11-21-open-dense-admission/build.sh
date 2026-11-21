#!/bin/bash
# build.sh TREE NAME: TREE's lib tests with the admission harness and master's kernel as
# admit_ref, copied to NAME.bin
set -e
A=$(cd $(dirname $0) && pwd)
T=$1
cp $A/admit_harness.rs $T/src/open/admit_harness.rs
cp $T/src/open.rs $A/open.rs.keep
rm -rf $T/src/open/admit_ref; cp -r $A/ref $T/src/open/admit_ref
printf '\n#[cfg(test)]\n#[allow(dead_code)]\nmod admit_ref;\n#[cfg(test)]\nmod admit_harness;\n' >> $T/src/open.rs
cd $T
bin=$(CARGO_TARGET_DIR=$A/target cargo test --release --lib --no-run 2>&1 | tee $A/build.log | grep -o '/tmp[^)]*mibz2-[0-9a-f]*' | tail -1) || true
cp $A/open.rs.keep $T/src/open.rs
rm -r $T/src/open/admit_harness.rs $T/src/open/admit_ref
[ -n "$bin" ] || { tail -40 $A/build.log; exit 1; }
cp $bin $A/$2.bin
echo built $2.bin
