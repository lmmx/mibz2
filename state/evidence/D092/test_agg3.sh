#!/bin/bash
# Synthetic traces for agg3 (D086's witness, mixed classes, the static image, ring wraparound,
# a true dependence). Exits non-zero on any unexpected count.
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
exit $fail
