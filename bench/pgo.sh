#!/usr/bin/env bash
# Profile-guided optimisation of this checkout's examples: a research probe
# (docs/journal/2026-11-05-pgo-probe.md), not the published build. The rustc book's three
# steps: an instrumented build of the A/B harness, training decodes with it, and the build
# its profile guides.
#
#   bench/pgo.sh WORK OUT EXAMPLE...
#
# WORK holds the raw profiles, the merged profile, the training corpora and a target dir of
# its own; the EXAMPLEs built with the profile are copied into OUT.
#
# - **Training data:** the crate's generated corpora only (`ab gen`, tests/common's recipe),
#   at PGO_SCALE (default 2, not the bench's 4: other files from the same recipe), never
#   the Wikidata blocks, so the bench's Wikidata rows are out of sample.
# - **Training decodes,** each corpus through every path a release build takes on a
#   machine like the runner's, or a bigger one: the reader on one thread
#   (`BzReader::new`), the workers on PGO_THREADS (default nproc), the cooperative teams of
#   2 on as many (`Plan::Teams`, the policy's plan from 6 physical cores), the portable
#   kernels on one thread (CPUs without AVX2), and the one-shot `decode`. A path no
#   training decode takes is optimised for size (LLVM's profile-guided size
#   optimisation), hence the teams and the portable kernels although the runner takes
#   neither. Each decode's bytes are checked: every decode of a corpus must hash the same
#   and report no error.
# - **PGO_WARN=1** (a diagnostic) adds LLVM's warning for each function the profile has no
#   record of, to the optimised build: names that do not match the instrumented build's.
# - **No leak:** the flags go to cargo's target builds only (`--target HOST`, so build
#   scripts and proc macros are built without them), in WORK's target dir, and are
#   exported nowhere: nothing built after this script sees them.
set -euo pipefail

work=$(realpath -m "$1")
out=$2
shift 2
host=$(rustc -vV | sed -n 's/^host: //p')
profdata="$(rustc --print sysroot)/lib/rustlib/$host/bin/llvm-profdata"
[ -x "$profdata" ] || rustup component add llvm-tools
target="$work/target"
threads=${PGO_THREADS:-$(nproc)}
rm -rf "$work/raw" "$work/corpora" "$work/merged.profdata"
mkdir -p "$work/raw" "$out"

# 1. The instrumented harness
start=$SECONDS
RUSTFLAGS="-Cprofile-generate=$work/raw" CARGO_TARGET_DIR="$target" \
  cargo build --release --target "$host" --example ab
ab="$target/$host/release/examples/ab"
echo "pgo: instrumented build $((SECONDS - start)) s"

# 2. Training (the corpora's generation profiled apart, and dropped)
start=$SECONDS
LLVM_PROFILE_FILE="$work/gen.profraw" "$ab" gen "$work/corpora" "${PGO_SCALE:-2}" 2> /dev/null
rm -f "$work/gen.profraw"
for f in "$work"/corpora/*.bz2; do
  first=
  for run in "mibz2 1" "mibz2 $threads" "mibz2-coop2 $threads" "mibz2-scalar 1" "mibz2-oneshot 1"; do
    # shellcheck disable=SC2086 # (the arm and its threads, two words)
    line=$("$ab" child $run "$f")
    hash=$(sed -n 's/.*"hash":"\([^"]*\)".*/\1/p' <<< "$line")
    err=$(sed -n 's/.*"err":"\([^"]*\)".*/\1/p' <<< "$line")
    if [ -z "$hash" ] || [ -n "$err" ] || { [ -n "$first" ] && [ "$hash" != "$first" ]; }; then
      echo "::error::pgo training: $run on $f: $line"
      exit 1
    fi
    first=$hash
  done
done
"$profdata" merge -o "$work/merged.profdata" "$work"/raw/*.profraw
echo "pgo: training $((SECONDS - start)) s, $(ls "$work"/raw/*.profraw | wc -l) raw profiles"
"$profdata" show "$work/merged.profdata" | tail -n 4

# 3. The build the profile guides
start=$SECONDS
examples=()
for e in "$@"; do examples+=(--example "$e"); done
use="-Cprofile-use=$work/merged.profdata"
[ -n "${PGO_WARN:-}" ] && use="$use -Cllvm-args=-pgo-warn-missing-function"
RUSTFLAGS="$use" CARGO_TARGET_DIR="$target" cargo build --release --target "$host" "${examples[@]}"
for e in "$@"; do cp "$target/$host/release/examples/$e" "$out/$e"; done
echo "pgo: optimised build $((SECONDS - start)) s"
