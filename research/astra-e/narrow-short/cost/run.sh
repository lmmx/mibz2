#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
tmp=${1:?scratch output directory}
deps=${2:?release/deps directory with a moonshot-enabled library}
cpu=${3:-3}
mkdir -p "$tmp/rebill"
python3 "$d/prepare.py" "$tmp" "$deps"
PYTHONDONTWRITEBYTECODE=1 python3 "$d/reprice.py" "$tmp" "$deps"
input="$root/corpora/wikidata.bz2"
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $input" | sha256sum --check
for binary in phase reprice; do
  out="$tmp"
  if [[ "$binary" == reprice ]]; then out="$tmp/rebill"; fi
  nice -n 14 taskset -c "$cpu" "$tmp/$binary" "$input" "$out" native
  nice -n 14 taskset -c "$cpu" valgrind --tool=callgrind --instr-atstart=no --collect-jumps=no --cache-sim=no --branch-sim=no --callgrind-out-file="$out/ir.%p" "$tmp/$binary" "$input" "$out" callgrind
done
