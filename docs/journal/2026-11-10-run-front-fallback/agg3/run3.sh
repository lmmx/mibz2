#!/bin/bash
# run3.sh BUILD PAD FILE: agg3 over one lackey trace of `ab child mibz2 1 FILE`, env padded by PAD bytes
t=$1; pad=$2; f=$3; B=0x108000
W=/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad
bin=$W/$t/target/release/examples/ab
args=$(nm -S -C $bin | grep -E ' (mibz2::mtf::mtf_avx2|mibz2::block::read_item|mibz2::bwt::build|mibz2::bwt::room_segments::<32, 1(, false)?>|mibz2::rle::unrle_avx2|mibz2::crc::clmul_update)$' | while read a s ty name; do n=$(echo "$name" | sed 's/mibz2:://; s/::<32, 1.*//; s/::/./g; s/ /_/g'); printf '%s:%x:%x ' $n $((B+0x$a)) $((B+0x$a+0x$s)); done)
env PAD="$(head -c $pad /dev/zero | tr '\0' x)" valgrind --tool=lackey --trace-mem=yes --log-fd=9 $bin child mibz2 1 $f 9>&1 >/dev/null 2>/dev/null | $W/laytool3/agg3 $args
