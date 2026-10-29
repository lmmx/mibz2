#!/usr/bin/env bash
# The compact T-vector pilot (examples/compact.rs, research only;
# docs/journal/2026-10-28-compact-tvector.md): the inverse BWT with a 20-bit ψ table (2.5 B
# a row, packed or libbzip2's split layout) against production's 32-bit T-vector and a
# 32-bit control under the same walk, N blocks in flight on one physical core each. Runs
# the correctness selftest first. Writes compact-<UTC time>.md and its rounds, raw, in
# compact-<UTC time>.jsonl: the files to share.
#
#   bash bench/compact.sh [--rounds R] [--arms p4,c6,...]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

rounds="" arms=""
while [ $# -gt 0 ]; do
  case "$1" in
    --rounds) rounds="$2"; shift 2 ;;
    --arms) arms="$2"; shift 2 ;;
    *) echo "bench/compact.sh: unknown argument $1"; exit 1 ;;
  esac
done

# One CPU of each physical core this process may use (the first of each sibling list)
cpus=$(for c in $(taskset -pc $$ | sed 's/.*: //' | tr ',' '\n' | while IFS=- read a b; do seq "$a" "${b:-$a}"; done); do
  cut -d, -f1 /sys/devices/system/cpu/cpu$c/topology/thread_siblings_list | cut -d- -f1
done | sort -nu | paste -sd,)
cores=$(echo "$cpus" | tr ',' '\n' | wc -l)

cargo build --release --example compact
bin=target/release/examples/compact

# p: production; t: the same walk as c and b on a 32-bit ψ (the control); c: packed 20-bit
# ψ; b: libbzip2's split 16 + 4-bit ψ. p4 first: the paired base, today's best.
if [ -z "$arms" ]; then
  arms="p4,p1,p2,p6,p8,p10,t1,t4,t6,t8,t10,c1,c2,c4,c6,c8,c10,b1,b4,b6,b8,b10"
fi
fit=""
for a in $(echo "$arms" | tr ',' ' '); do
  [ "${a:1}" -le "$cores" ] && fit="$fit,$a"
done
fit=${fit#,}
n_arms=$(echo "$fit" | tr ',' '\n' | wc -l)
rounds=${rounds:-$((2 * n_arms))}

out=compact-$(date -u +%Y%m%dT%H%M%SZ).md
export COMPACT_JSONL=${out%.md}.jsonl  # (every round of the grid, raw)
export COMPACT_CPUS=$cpus  # (each worker pinned to its own CPU of these, and checked)
{
  echo "# The compact T-vector pilot ($(git rev-parse --short HEAD), $(date -u +%FT%TZ))"
  echo
  echo "- **Machine:** $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs); $(nproc --all) CPUs on $cores physical cores used ($cpus); L3 $(cat /sys/devices/system/cpu/cpu0/cache/index3/size 2> /dev/null) shared by CPUs $(cat /sys/devices/system/cpu/cpu0/cache/index3/shared_cpu_list 2> /dev/null)"
  echo "- **Load:** $(cut -d' ' -f1-3 /proc/loadavg)"
  echo
  echo "## Correctness (selftest)"
  echo
  "$bin" selftest | tail -1
  echo
  echo "## N blocks in flight, one core each (grid, $rounds rounds)"
  echo
  "$bin" grid corpora/wikidata.bz2 "$rounds" "$fit"
} 2> >(grep -v '^round' >&2) | tee "$out"
echo "wrote $out and $COMPACT_JSONL"
