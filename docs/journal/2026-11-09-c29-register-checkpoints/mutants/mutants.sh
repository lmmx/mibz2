#!/usr/bin/env bash
# C29's mutants (docs/journal/2026-11-09-c29-register-checkpoints.md, B5): each patch here
# applied to a clean tree, the tests that should kill it run, the log kept, the patch
# reversed and the tree checked clean; then the restored tree's tests, which must pass.
#
#   bash docs/journal/2026-11-09-c29-register-checkpoints/mutants/mutants.sh [MUTANT..]
#
# From the repository's root, on a tree with no changes to src/. One test binary at a
# time, two test threads, a 6 GB address-space cap (the shared machine's rules). The Miri
# run (C29c) needs the nightly toolchain with miri. A mutant is KILLED when any of its
# commands fails (a test failure, an abort, Miri's UB report), SURVIVED when all pass.
set -u
here=$(cd "$(dirname "$0")" && pwd)
logs="$here/logs"
mkdir -p "$logs"
ulimit -v 6000000
git diff --quiet -- src || { echo "src/ has changes: refusing"; exit 1; }

lib_c29='cargo test -j2 --lib mtf::tests -- --test-threads=2'
miri_c29='env RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1 MIRIFLAGS=-Zmiri-env-forward=MIBZ2_EXPECT_AVX2 cargo +nightly miri test -j2 --lib -- --exact mtf::tests::miri_register_checkpoints'
pipelined_reach='cargo test -j2 --test pipelined an_mtf_error_after_a_small_block -- --test-threads=2'
switch='cargo test -j2 --lib parallel::c29_tests -- --test-threads=2'

# each mutant's killing commands, in order
declare -A cmds=(
  [C29a-checkpoint-before-part]="$lib_c29"
  [C29b-final-checkpoint]="$lib_c29"
  [C29c-vector-store-bound]="$lib_c29;;$miri_c29"
  [C29d-lever-dropped]="$pipelined_reach"
  [C29e-front-stage-ignores-switch]="$switch"
)
names=("$@")
[ ${#names[@]} -gt 0 ] || names=(C29a-checkpoint-before-part C29b-final-checkpoint C29c-vector-store-bound C29d-lever-dropped C29e-front-stage-ignores-switch)

summary="$logs/summary.txt"
: > "$summary"
for m in "${names[@]}"; do
  git apply "$here/$m.patch" || { echo "$m: does not apply" | tee -a "$summary"; exit 1; }
  log="$logs/$m.log"
  { echo "mutant $m on $(git rev-parse HEAD) + mutants/$m.patch; $(date -u +%FT%TZ)"; } > "$log"
  killed=no
  IFS=';;' read -ra parts <<< "${cmds[$m]}"
  for c in "${parts[@]}"; do
    [ -n "$c" ] || continue
    echo "cmd: $c" >> "$log"
    RUST_BACKTRACE=0 bash -c "$c" >> "$log" 2>&1
    rc=$?
    echo "rc=$rc" >> "$log"
    [ $rc -ne 0 ] && killed=yes
  done
  git apply -R "$here/$m.patch"
  git diff --quiet -- src || { echo "$m: the tree did not restore" | tee -a "$summary"; exit 1; }
  echo "$m: $([ $killed = yes ] && echo KILLED || echo SURVIVED)" | tee -a "$summary"
done

# the restored tree: every killing command passes
log="$logs/restored.log"
echo "restored tree $(git rev-parse HEAD); $(date -u +%FT%TZ)" > "$log"
ok=yes
for c in "$lib_c29" "$miri_c29" "$pipelined_reach" "$switch"; do
  echo "cmd: $c" >> "$log"
  RUST_BACKTRACE=0 bash -c "$c" >> "$log" 2>&1
  rc=$?
  echo "rc=$rc" >> "$log"
  [ $rc -ne 0 ] && ok=no
done
echo "restored: $([ $ok = yes ] && echo PASS || echo FAIL)" | tee -a "$summary"
