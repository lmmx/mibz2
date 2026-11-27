claim: mtf-fold-run
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md, PR #1010 (cf242fe7a4), origin/bench/zero-copy-floor (1e27e46f972f), examples/mtf_fold.rs, bench/mtf_fold.py, bench/test_mtf_fold.py, .just/bench.just

**What was asked:** Port the zero-copy scan oracle beside `mtf-fold`, wire borrowed symbols into `runs::fold` so `FS` actually folds, and freeze the `bench-mtf-fold` workstation kill-test recipe to registration §3.

**What I found:**
- Ported `FrozenSlot` (`src/ideal/frozen.rs`), zero-copy `scan-oracle` (`src/scan_oracle.rs`) with verified scanner CPU pinning, and zero-copy `OnceLock` upgrades for `ideal-mtf` (`Arc<[u16]>`) and `ideal-construction` (`&Built` via `walk_slices`) so `S`, `FS`, `SM`, and `SC` all run zero-copy in one binary with default builds unchanged.
- Wired `scan_oracle::with_syms` into `runs::fold::{wants, front, front_scalar}` and `front_avx2`, plus `runs.rs` and `mtf.rs`. Added `check_fs_replay` in `tests/mtf_fold.rs` asserting `fold::take_counts().folded > 0`, `bailed == 0`, `scanned == 0`, `scan_locks == 0`, and byte-identity on scalar and AVX2.
- Added `trace::Kind::OpenPass1End` (emitted after Pass 1 in `construct_capped` and `fold`) and `ready_bytes` on `Kind::Ready` so traced blocks report `A5`, `A6a`, fused `A5+A6a`, `A6bc+A7`, `A9`, `copy_out`, `ready_bytes` hwm, and `S` scan residual (`0.12%–0.13%` in smoke vs `5%` VOID limit).
- Built `examples/mtf_fold.rs`, `bench/mtf_fold.py`, `bench/test_mtf_fold.py` (11/11 pass), and `.just/bench.just`'s `bench-mtf-fold` implementing registration §3 verbatim (12 randomised blocks, traces in first 3, `none/F/S/FS` at `k = 1, 6, 8, 9, 10`, `SM/SC` at `k = 8`, `ab gen` text at `k = 1, 8`, frozen K1–K4 and VOID §3.5).
- Verified `tests/mtf_fold.rs` (`mtf-fold` and `mtf-fold scan-oracle`), `ideal::`, `runs::tests::fold_tests`, `tests/ideal_oracles.rs`, release builds (with and without `trace`), `clippy --all-targets --all-features -D warnings`, and a full 66+8-decode end-to-end smoke run (`F` and `FS` both folded all Open blocks with `bailed = 0`, 100% byte-identical).

**What changed:**
- `src/ideal/{frozen.rs, mod.rs, mtf.rs, construction.rs}`, `src/scan_oracle.rs`, `src/runs.rs`, `src/runs/fold.rs`, `src/runs/tests.rs`, `src/mtf.rs`, `src/open.rs`, `src/trace.rs`, `src/parallel.rs`, `src/lib.rs`, `Cargo.toml`, `tests/mtf_fold.rs`.
- `examples/mtf_fold.rs`, `bench/mtf_fold.py`, `bench/test_mtf_fold.py`, `.just/bench.just` (`bench-mtf-fold`).

**What's next:** Merge this PR once CI is green, then queue `bench-mtf-fold` in `bench/workstation-queue.txt` and dispatch `ready/bench-mtf-fold` via `run-signal.yml` (standing authorisation, `CLAUDE.md`) and read the i9 bundle against frozen K1–K4.

**Caveats:** Smoke timings are container sanity checks only (not i9 evidence); K1–K4 verdict waits for the i9-10900K bundle.
