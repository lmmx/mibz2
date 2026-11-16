#!/usr/bin/env bash
# The staged kernel's store fence in the compiled library (Guardian D108, #145), and the
# staged kernel's code absent from a build without its feature. (The experiment's other
# kernel, lazy letters, is production's: src/cells_lazy.rs.)
#
# The tests model the fence (src/cells_lever.rs, PENDING): they show a fence object covers
# every exit of the staged induction, but not that the object still runs `sfence`. This
# reads the machine code:
#   1. built with `memory-lever`, the staged induction reaches an `sfence`: in its own body,
#      or in the `Fence` drop it calls;
#   2. built without the feature, the library has no `sfence` and none of the staged
#      kernel's functions.
# x86-64 only (elsewhere the staged kernel stores nothing past the cache). Exits non-zero on
# a failure. Usage: bash bench/memory_lever_fence.sh
set -euo pipefail
if [ "$(uname -m)" != x86_64 ]; then
  echo "memory_lever_fence: not x86-64, nothing to check"
  exit 0
fi
target=${CARGO_TARGET_DIR:-target}
lib=$target/release/libmibz2.rlib
# symbol <tab> count of sfence in its body, for every function that has one
fences() {
  objdump -d --no-show-raw-insn "$1" 2>/dev/null \
    | awk '/^[0-9a-f]+ <.*>:$/ {sym=$2} /[ \t]sfence/ {c[sym]++} END {for (s in c) print s "\t" c[s]}'
}
# the functions a symbol's body calls
calls() {
  objdump -d --no-show-raw-insn "$1" 2>/dev/null \
    | awk -v want="$2" '/^[0-9a-f]+ <.*>:$/ {on = index($2, want) > 0} on && /call/ {print}'
}
fail() { echo "memory_lever_fence: FAIL: $*"; exit 1; }

cargo build -q --release --features memory-lever
with=$(fences "$lib")
echo "with memory-lever, functions holding an sfence:"
echo "${with:-  (none)}"
staged=$(echo "$with" | grep -c 'induce_cells_staged' || true)
dropfn=$(echo "$with" | grep -c 'Fence' || true)
called=$(calls "$lib" induce_cells_staged | grep -c 'Fence' || true)
if [ "$staged" -ge 1 ]; then
  echo "the staged induction holds an sfence in its own body"
elif [ "$dropfn" -ge 1 ] && [ "$called" -ge 1 ]; then
  echo "the staged induction calls the Fence drop, which holds the sfence"
else
  fail "the staged induction reaches no sfence"
fi
# every exit: where the drop is not inlined it must be called, and hold the fence
if [ "$called" -ge 1 ] && [ "$dropfn" -lt 1 ]; then
  fail "the staged induction calls a Fence drop that holds no sfence"
fi

cargo build -q --release
without=$(fences "$lib")
[ -z "$without" ] || fail "a build without the feature holds an sfence: $without"
if objdump -d --no-show-raw-insn "$lib" 2>/dev/null \
  | grep -E '^[0-9a-f]+ <.*(induce_cells_staged|send_rules).*>:$' >/dev/null; then
  fail "a build without the feature holds one of the staged kernel's functions"
fi
echo "without the feature: no sfence, none of the staged kernel's functions"
echo "memory_lever_fence: ok"
