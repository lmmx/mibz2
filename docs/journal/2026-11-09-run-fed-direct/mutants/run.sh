#!/usr/bin/env bash
# C31's mutants (docs/journal/2026-11-09-run-fed-direct.md §10, Guardian D055): each patch
# applied to src/run_stage.rs, the run-fed lib tests run (and, where named, Miri on the
# direct-placement tests with AVX2, and the D fuzz mode for 20 s), then the patch reversed;
# last, the restored tree's tests, which must pass. Logs go to logs/. Run from the
# repository's root, on a clean src/run_stage.rs:
#
#   bash docs/journal/2026-11-09-run-fed-direct/mutants/run.sh
set -u
here=docs/journal/2026-11-09-run-fed-direct/mutants
mkdir -p "$here/logs"
git diff --quiet -- src/run_stage.rs || { echo "src/run_stage.rs is not clean"; exit 2; }
echo "source: $(git rev-parse HEAD), src/run_stage.rs blob $(git rev-parse HEAD:src/run_stage.rs); $(rustc --version)" | tee "$here/logs/source.txt"

tests() { cargo test -j2 --features run-fed --lib run_stage -- --test-threads=2 2>&1; }
miri() {
  RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1 \
    MIRIFLAGS=-Zmiri-env-forward=MIBZ2_EXPECT_AVX2 \
    cargo +nightly miri test -j2 --lib -- run_stage::tests::miri_direct 2>&1
}
fuzz() {
  cargo build -j2 --profile fuzz --features run-fed --example fuzz 2>&1 | tail -1
  target/fuzz/examples/fuzz 20 1 - run-fed-direct 2>&1 | tail -6
}

for m in M1-tail-removed M2-psi-off-by-one M3-final-check-removed M4-allocation-bound; do
  log="$here/logs/$m.log"
  git apply "$here/$m.patch" || { echo "$m: does not apply"; exit 2; }
  {
    echo "== $m: $(git diff --stat -- src/run_stage.rs | tail -1)"
    echo "== native tests"
    tests | grep -E "^test |test result|panicked at|signal|malloc|realloc|free\(\)"
    case $m in
      M1-*) echo "== Miri, AVX2, the direct tests"; miri | grep -E "^test |test result|Undefined Behavior|out-of-bounds|error:" | head -20 ;;
    esac
    case $m in
      M2-* | M4-*) echo "== the D fuzz mode, 20 s, seed 1"; fuzz ;;
    esac
  } > "$log" 2>&1
  git apply -R "$here/$m.patch"
  git diff --quiet -- src/run_stage.rs || { echo "$m: not restored"; exit 2; }
  echo "$m: $(grep -c 'FAILED\|failed' "$log") failure lines; log $log"
done
{
  echo "== restored: $(git diff --quiet -- src/run_stage.rs && echo clean)"
  tests | grep -E "^test |test result"
} > "$here/logs/restored.log" 2>&1
cat "$here/logs/restored.log"
