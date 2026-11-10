#!/bin/bash
# run2.sh BUILD FILE: agg2 on mtf_avx2
t=$1; f=$2; B=0x108000
bin=/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/$t/target/release/examples/ab
r=$(nm -S -C $bin | grep -E ' mibz2::mtf::mtf_avx2$' | while read a s ty name; do printf 'mtf:%x:%x' $((B+0x$a)) $((B+0x$a+0x$s)); done)
valgrind --tool=lackey --trace-mem=yes --log-fd=9 $bin child mibz2 1 $f 9>&1 >/dev/null 2>/dev/null | /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/laytool/agg2 $r
