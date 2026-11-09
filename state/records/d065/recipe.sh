#!/usr/bin/env bash
set -euo pipefail
# each attempt in a directory of its own, never overwritten
B=$PWD/target/telemetry-check
W=$B/$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$B"
mkdir "$W" || { echo "bench-telemetry-check: $W already exists; not reusing an attempt's directory"; exit 1; }
exec > >(tee -a "$W/console.log") 2>&1
outcome="INCOMPLETE: stopped before the check finished (see console.log)"
finish() {
  status=$?
  echo "$outcome" > "$W/outcome.txt"
  echo "bench-telemetry-check: OUTCOME: $outcome"
  bash bench/bundle.sh "$W"
  [ "$status" -ne 0 ] && exit "$status"; exit 0
}
trap finish EXIT
git rev-parse HEAD > "$W/commit.txt"
git status --porcelain --untracked-files=no > "$W/git-status.txt"
cargo build -q --release --example ab
AB=$PWD/target/release/examples/ab
"$AB" env > "$W/env.json"
lscpu > "$W/lscpu.txt"
"$AB" sample --count 3 --interval 1 > "$W/sample.jsonl"
status=0
"$AB" telemetry-check > "$W/telemetry-check.jsonl" || status=$?
python3 -c 'import json, sys
for l in open(sys.argv[1]):
    r = json.loads(l)
    print(r["status"].ljust(12), r["case"].ljust(28), r["detail"])' "$W/telemetry-check.jsonl" | tee "$W/telemetry-check.txt"
if [ "$status" -eq 0 ]; then
  outcome="CHECKED: no case FAILed ($(grep -c '"status":"UNAVAILABLE"' "$W/telemetry-check.jsonl" || true) UNAVAILABLE); see telemetry-check.txt"
else
  outcome="FAILED: a case FAILed; see telemetry-check.txt"
fi

