#!/bin/bash
# Synthetic traces for agg3 (D086's witness, mixed classes, the static image, ring wraparound,
# a true dependence), then the count-unit and completeness witnesses of REVIEW 6763607539
# (Researcher) and D092 (Guardian). Exits non-zero on any unexpected count.
cd "$(dirname "$0")"; fail=0
chk() { out=$(printf "%b" "$2" | ./agg3 f:400000:400100 | sed -n 2p); if [ "$out" != "$3" ]; then echo "FAIL $1: got '$out' want '$3'"; fail=1; else echo "ok   $1: $out"; fi; }
chk witness-a "I  400000,4\n S 1000002000,8\n S 3000,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store): H<-S=1 H<-H=1"
chk witness-b "I  400000,4\n S 1000002040,8\n S 3000,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store): H<-H=1"
chk witness-reordered "I  400000,4\n S 3000,8\n S 1000002000,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store): H<-S=1 H<-H=1"
chk static-image "I  400000,4\n S 113000,8\n S 1000002000,8\n S 3000,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store): H<-S=1 H<-D=1 H<-H=1"
chk only-1K "I  400000,4\n S 3400,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store):"
w="I  400000,4\n S 3000,8\n"; for i in $(seq 1 48); do w="$w S 1000005$(printf %03x $((i*16))),8\n"; done
chk wrap-evicted "$w""I  400004,4\n L 4000,8\n" "  mod 4K (load<-store):"
w="I  400000,4\n S 3000,8\n"; for i in $(seq 1 47); do w="$w S 1000005$(printf %03x $((i*16))),8\n"; done
chk wrap-kept "$w""I  400004,4\n L 4000,8\n" "  mod 4K (load<-store): H<-H=1"
chk true-dep "I  400000,4\n S 4000,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store):"
chk partial-overlap-real "I  400000,4\n S 3ffc,8\nI  400004,4\n L 4000,8\n" "  mod 4K (load<-store):"

# has NAME TRACE ARGS LINE...: every LINE must appear exactly in agg3's output for TRACE.
has() { n=$1; t=$2; a=$3; shift 3; out=$(printf "%b" "$t" | ./agg3 $a 2>&1); ok=1
  for l in "$@"; do if ! grep -qxF -- "$l" <<<"$out"; then echo "FAIL $n: missing '$l'"; ok=0; fail=1; fi; done
  [ $ok = 1 ] && echo "ok   $n"; }
F="mtf_avx2:1000:2000"
# Researcher's two-store witness: two stores at one PC, one load. Class 1 (loads), pair weight 2,
# pair loads 1; the named population is counted directly, never class minus a pair sum.
two="I 2000,4\n S 2000100,4\nI 2000,4\n S 2000100,4\nI 1100,4\n L 100100,16\n"
has two-store "$two" "--named=2000:1100 $F" \
  "  mod 4K (load<-store): D<-H=1" \
  "NAMED mtf_avx2 mod4K D<-H: class=1 named=1 outside=0 both=0 | weight named=2 outside=0" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=2" \
  "PAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=2 named=1"
# Researcher's six-pair witness: six loads, one distinct pair each, 48 non-matching stores before
# each. The legacy PAIRS line shows five; every pair must still be listed and counted.
six=""; for j in 0 1 2 3 4 5; do
  six="${six}I 3000,4\n"; for i in $(seq 48); do six="${six} S 3000800,4\n"; done
  six="${six}I $(printf %x $((0x2000+j))),4\n S 2000100,4\nI $(printf %x $((0x1100+j))),4\n L 100100,16\n"; done
has six-pairs "$six" "--named=2000:1100,2001:1101,2002:1102,2003:1103,2004:1104,2005:1105 $F" \
  "  mod 4K (load<-store): D<-H=6" \
  "NAMED mtf_avx2 mod4K D<-H: class=6 named=6 outside=0 both=0 | weight named=6 outside=0" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=1" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2005 load 1105 loads=1 weight=1" \
  "PAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=1 named=1" \
  "PAIR mtf_avx2 mod4K D<-H: store 2005 load 1105 loads=1 weight=1 named=1"
# The same trace with one named pair, outside the legacy top five: the outside population is
# counted directly (5 loads), and the named pair is complete.
has six-one-named "$six" "--named=2000:1100 $F" \
  "NAMED mtf_avx2 mod4K D<-H: class=6 named=1 outside=5 both=0 | weight named=1 outside=5" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=1"
# D092's overlap witness: one named store and one other store match the same load. Class 1,
# named 1, outside 1, both 1: class minus named (0) would hide the outside pair.
ov="I 2000,4\n S 2000100,4\nI 2004,4\n S 2005100,4\nI 1100,4\n L 100100,16\n"
has named-and-other "$ov" "--named=2000:1100 $F" \
  "  mod 4K (load<-store): D<-H=1" \
  "NAMED mtf_avx2 mod4K D<-H: class=1 named=1 outside=1 both=1 | weight named=1 outside=1" \
  "PAIR mtf_avx2 mod4K D<-H: store 2004 load 1100 loads=1 weight=1 named=0"
# Overlapping named PCs: two named store PCs match one load. Named counts the load once; each
# named pair counts it once (pair loads don't sum to the class). A named pair absent from the
# trace is printed with explicit zeros.
has overlapping-named "$ov" "--named=2000:1100,2004:1100,2fff:1100 $F" \
  "NAMED mtf_avx2 mod4K D<-H: class=1 named=1 outside=0 both=0 | weight named=2 outside=0" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2000 load 1100 loads=1 weight=1" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2004 load 1100 loads=1 weight=1" \
  "NAMEDPAIR mtf_avx2 mod4K D<-H: store 2fff load 1100 loads=0 weight=0"
# Completeness: the output ends with the completion line.
has complete "$two" "--named=2000:1100 $F" "END agg3 complete lines=6 pairs=2 named=1"
exit $fail
