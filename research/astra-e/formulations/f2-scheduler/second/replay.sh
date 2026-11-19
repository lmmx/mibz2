#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/second"
tmp=${1:?scratch directory}
deps=${2:?absolute release deps}
mkdir -p "$tmp"
for mode in refill batch scalar; do
    python3 "$d/prepare.py" "$mode" "$tmp/$mode"
    sed -i 's/pub(super) struct Second {/pub(super) struct Second { pub(super) discovery_ns:u64,/' "$tmp/$mode/second.rs"
    sed -i 's/self.discover(d, first_words, bytes);/let t=crate::thread_cpu_ns();self.discover(d, first_words, bytes);self.discovery_ns=crate::thread_cpu_ns()-t;/' "$tmp/$mode/second.rs"
    cat "$d/replay_impl.rs" >> "$tmp/$mode/candidate.rs"
done
cp "$d/replay.rs" "$tmp/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$tmp/replay.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/replay"
cpu=$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')
echo "fixed affinity CPU $cpu" >&2
taskset -c "$cpu" "$tmp/replay" "$root/corpora/wikidata.bz2"
