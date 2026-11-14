#!/usr/bin/env bash
# The Cells memory census: both kernels' attempts on the pinned Wikidata corpus, traced by
# the memtrace valgrind tool and counted by the consumer. Untimed.
#
#   bash docs/journal/2026-11-14-cells-memory/run.sh CORPUS.bz2 WORKDIR
#
# Needs: valgrind 3.22 with its tool libraries (/usr/lib/x86_64-linux-gnu/valgrind), gcc,
# rustc/cargo. Writes into this directory: trace-{cells,segmented}.jsonl (the consumer's,
# one line per kernel attempt), blocks-{cells,segmented}.jsonl (the driver's, one line per
# block), and summary.json (summarise.py).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
corpus=$1
work=$2
mkdir -p "$work"
cd "$(git -C "$here" rev-parse --show-toplevel)"
cargo build --release --features memtrace --example cells_memory
exe=target/release/examples/cells_memory
bash "$here/memtrace/build.sh" "$work" > /dev/null
rustc -O --edition 2021 -o "$work/consumer" "$here/memtrace/consumer.rs"
for mode in cells segmented; do
  fifo="$work/trace-$mode.fifo"
  rm -f "$fifo"
  mkfifo "$fifo"
  "$work/consumer" "$fifo" "$here/trace-$mode.jsonl" &
  consumer=$!
  VALGRIND_LIB="$work/lib" valgrind -q --tool=memtrace --out="$fifo" \
    "$exe" wikidata "$corpus" "$mode" > "$here/blocks-$mode.jsonl"
  wait "$consumer"
  rm -f "$fifo"
done
python3 "$here/summarise.py" "$here" > "$here/summary.json"
