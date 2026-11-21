#!/usr/bin/env bash
# one cachegrind run of the shipped lazy route at exactly 7 workers (the i9's plan at 10 and 20
# threads): $1 = build (port|v0.5.6), $2 = repeat, $3 = corpus path
set -u
S=/tmp/claude-0/-home-user-mibz2/e6647659-039f-564b-ba35-f13ad4a0fdc3/scratchpad/cg3/partA/raw
B=/tmp/claude-0/-home-user-mibz2/e6647659-039f-564b-ba35-f13ad4a0fdc3/scratchpad/orb/target/open-port
n=$(basename "$3" .bz2)
valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes \
  --cachegrind-out-file="$S/cg.$n.w7.$1.$2" "$B/build-$1/target/release/examples/ab" child mibz2-workers-lazy 7 "$3" \
  > "$S/out.$n.w7.$1.$2" 2>&1
