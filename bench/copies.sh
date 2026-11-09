#!/usr/bin/env bash
# One copy of a command on each CPU at once, each pinned to its CPU, and each copy's exit
# status collected (a bare `wait` returns success whatever the copies did). sweep.yml's
# contended condition.
#
#   bench/copies.sh PREFIX HEADER -- CMD [ARGS...]
#
# Copy c writes "$HEADER; this copy on CPU $c", then CMD's output, to PREFIX-cpu$c.txt, and
# runs with COPY_CPU=c in its environment (for a per-copy file name). Exits 0 only if
# every copy did; otherwise names each copy that failed and its status, and writes
# PREFIX-INCOMPLETE.txt with the same, so a partial run can't pass as a complete one.
set -u
prefix=$1
header=$2
shift 2
[ "${1:-}" = "--" ] && shift
cpus=$(nproc)
pids=()
for c in $(seq 0 $((cpus - 1))); do
  { echo "$header; this copy on CPU $c"; COPY_CPU=$c taskset -c "$c" "$@"; } > "$prefix-cpu$c.txt" 2>&1 &
  pids+=($!)
done
failed=()
for c in "${!pids[@]}"; do
  if wait "${pids[$c]}"; then :; else failed+=("CPU $c: exit $?"); fi
done
if [ ${#failed[@]} -gt 0 ]; then
  printf 'INCOMPLETE: %d of %d copies failed: %s\n' "${#failed[@]}" "$cpus" "$(IFS=';'; echo "${failed[*]}")" | tee "$prefix-INCOMPLETE.txt"
  exit 1
fi
rm -f "$prefix-INCOMPLETE.txt"
