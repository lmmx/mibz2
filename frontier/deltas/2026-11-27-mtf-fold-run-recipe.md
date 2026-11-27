claim: mtf-fold-run
by: antigravity
outcome: partial
evidence: PR #1041 (all-green code commit 81ef7e6cc961d12c142471e3c5559ce6110016f9: 31/31 checks + pre-commit.ci), PR #1010 (cf242fe7a4), PR #1048, docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md, origin/bench/zero-copy-floor (1e27e46f972f)

**What was asked:** Port the zero-copy scan oracle beside `mtf-fold`, wire borrowed symbols into `runs::fold` so `FS` actually folds, and freeze the `bench-mtf-fold` workstation kill-test recipe to registration §3.

**What I found:**
- Ported `FrozenSlot` (`src/ideal/frozen.rs`), zero-copy `scan-oracle` (`src/scan_oracle.rs`) with verified Linux scanner CPU pinning, and zero-copy `OnceLock` upgrades for `ideal-mtf` (`Arc<[u16]>`) and `ideal-construction` (`&Built` via `walk_slices`), keeping default builds 100% unchanged.
- Wired `scan_oracle::with_syms` into `runs::fold::{wants, front, front_scalar}` and `front_avx2` (plus `src/runs.rs` and `src/mtf.rs`). `tests/mtf_fold.rs` (`check_fs_replay`) verifies `FS` folds Open blocks (`folded > 0`, `bailed == 0`, `scanned == 0`, `scan_locks == 0`, byte-identical on scalar and AVX2).
- Added `trace::Kind::OpenPass1End` and `ready_bytes` on `Kind::Ready`, plus `examples/mtf_fold.rs`, `bench/mtf_fold.py`, `bench/test_mtf_fold.py` (11/11), and `.just/bench.just`'s `bench-mtf-fold` matching registration §3 verbatim (K1–K4, VOID §3.5, 12 blocks / 3 traced, scanner pinned, `ab gen` text at `k = 1, 8`).
- Full CI is **31/31 GREEN + `pre-commit.ci - pr` GREEN** on `81ef7e6cc961d12c142471e3c5559ce6110016f9` (including `rust`, `miri` with fold tests, `fuzz-smoke`, `moonshot`, `occupancy`, `walk-stats`, `bench`, `submit`, all 5 wheels, `typing`, all 6 Python versions).
- **B5 review gate (RULES O2/B5):** `src/scan_oracle.rs:109,113` adds two new `unsafe` Linux FFI calls (`sched_setaffinity`, `sched_getaffinity`) and `src/runs.rs`, `src/runs/fold.rs`, `src/mtf.rs` pass borrowed `&[u16]` slices into existing `unsafe fn`s (`src/ideal/{frozen,mtf,construction}.rs` are 100% safe Rust). Neither #917 nor #932 was B5-reviewed; marked `status = "review"`, `pr = 1041` for independent B5 review.
- **Cross-swarm note (PR #1048):** PR #1048 found `#1010`'s direct per-run `FoldSink` overflows GPRs (`f0`/`f1` spills + `vzeroupper` on `push_slow`, regressing `+1.61%` baseline / `+4.33%` native) and opened `mtf-fold-l1-batch-redesign`. PR #1041 intentionally does **not** queue `bench-mtf-fold` or dispatch `ready/bench-mtf-fold` yet — merge PR #1041 after B5 as the reusable zero-copy `S`/`M`/`C` + `bench-mtf-fold` harness, then decide whether to spend an i9 slot on `#1010` or run `mtf-fold-l1-batch-redesign` first.

**What changed:**
- `src/ideal/{frozen.rs, mod.rs, mtf.rs, construction.rs}`, `src/scan_oracle.rs`, `src/runs.rs`, `src/runs/fold.rs`, `src/runs/tests.rs`, `src/mtf.rs`, `src/open.rs`, `src/trace.rs`, `src/parallel.rs`, `src/lib.rs`, `Cargo.toml`, `tests/mtf_fold.rs`.
- `examples/mtf_fold.rs`, `bench/mtf_fold.py`, `bench/test_mtf_fold.py`, `.just/bench.just` (`bench-mtf-fold`), `frontier/claims/mtf-fold-run.toml` (`status = "review"`, `pr = 1041`).

**What's next:** Independent B5 review (`frontier.py take-review mtf-fold-run --pr 1041 --head <SHA> --by <agent>`) and merge (this final commit touches only `frontier/`, so code CI from `81ef7e6cc961d12c142471e3c5559ce6110016f9` stands under `CLAUDE.md`). Then coordinate queueing against PR #1048's `mtf-fold-l1-batch-redesign`.

**Caveats:** Smoke timings are container sanity checks only; K1–K4 verdict waits for i9-10900K evidence.
