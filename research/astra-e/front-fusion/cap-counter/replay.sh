#!/usr/bin/env bash
# From a checkout containing the harness, create a new scratch workspace.
set -euo pipefail
study=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$study" rev-parse --show-toplevel)
scratch=$(realpath -m "${1:?supply a new scratch directory}")
corpus=$(realpath "${2:?supply canonical wikidata.bz2}")
test "$(sha256sum "$corpus" | cut -d ' ' -f 1)" = 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
mkdir "$scratch"
git -C "$repo" worktree add --detach "$scratch/scorer" 50b8ecb8946a498601e3fd894ed84a2687df6cc8
cd "$scratch/scorer"
for arm in counter_edge both; do
  git fetch origin master
  git checkout origin/master -- bench/moonshot .just Justfile
  git rev-parse origin/master > "$scratch/$arm-master.txt"
  git log -1 --format=%h origin/master -- bench/moonshot > "$scratch/$arm-path.txt"
  CARGO_BUILD_JOBS=1 nice -n 15 taskset -c 4 python3 -B bench/moonshot/moonshot.py score \
    "research/astra-e/front-fusion/cap-counter/$arm.rs" --no-timing --no-proxy \
    --corpus "$corpus" --target-dir "$scratch/$arm-build" --out "$scratch/$arm-gate"
done
# The four native paths link against exactly the same refreshed control library.
deps="$scratch/counter_edge-build/release/deps"
nice -n 15 taskset -c 3 python3 -B "$study/native.py" "$scratch/native" "$deps" original meter
for mode in original meter; do
  rounds=40
  if [[ $mode == meter ]]; then rounds=32; fi
  nice -n 12 taskset -c 3 "$scratch/native/$mode/panel" "$corpus" "$rounds" "$mode" > "$scratch/$mode.csv" 2> "$scratch/$mode.stderr"
done
nice -n 15 taskset -c 4 python3 -B "$study/native.py" "$scratch/native" "$deps" adverse
nice -n 15 taskset -c 4 "$scratch/native/adverse/adverse" > "$scratch/adverse.txt"
python3 -B "$study/report.py" "$scratch" > "$scratch/contrasts.csv"
