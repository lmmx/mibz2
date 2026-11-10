#!/bin/bash
# geom.sh BUILD: placement of the hot kernels and mtf's STEPS (mod 64, mod 4K), and counts mod 4K (gdb, ASLR on)
t=$1; W=/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad; bin=$W/$t/target/release/examples/ab
nm -S -C $bin | grep -E ' (mibz2::mtf::mtf_avx2|mibz2::block::read_item|mibz2::bwt::build|mibz2::bwt::room_segments::<32, 1(, false)?>|mibz2::rle::unrle_avx2|mibz2::crc::clmul_update|mibz2::mtf::STEPS|mibz2::decode_block::<mibz2::bwt::Segmented>)$' | python3 -c "
import sys
for l in sys.stdin:
    a,s,ty,*n=l.split(); n=' '.join(n).replace('mibz2::',''); a=int(a,16)
    print(f'  {n[:32]:32s} mod64={a%64:2d} mod4K={a%4096:#05x} size={int(s,16):#x}')
"
off=$(objdump -d --no-show-raw-insn -C $bin | awk '/<mibz2::mtf::mtf_avx2>:/{f=1} f' | grep -m1 -A1 "mov    %rcx,%rbx" | sed -n 2p | awk '{print $1}' | tr -d ':')
PROBE_OFF=$off gdb -q -batch -x $W/stackprobe324.gdb --args $bin child mibz2 1 $W/lay/rand1.bz2 2>&1 | grep -o "counts=0x[0-9a-f]*" | head -1 | python3 -c "import sys; c=int(sys.stdin.read().split('=')[1],16); print(f'  counts mod4K={c%4096:#05x}')"
