#!/usr/bin/env bash
# one cachegrind run: $1 = build (port|v0.5.6), $2 = threads, $3 = repeat, $4 = corpus path
set -u
S=/tmp/claude-0/-home-user-mibz2/e6647659-039f-564b-ba35-f13ad4a0fdc3/scratchpad/cg3/partA/raw
B=/tmp/claude-0/-home-user-mibz2/e6647659-039f-564b-ba35-f13ad4a0fdc3/scratchpad/orb/target/open-port
n=$(basename "$4" .bz2)
valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes \
  --cachegrind-out-file="$S/cg.$n.t$2.$1.$3" "$B/build-$1/target/release/examples/ab" child mibz2 "$2" "$4" \
  > "$S/out.$n.t$2.$1.$3" 2>&1
