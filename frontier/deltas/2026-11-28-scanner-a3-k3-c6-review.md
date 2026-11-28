claim: scanner-a3-k3-pipe-u8
by: antigravity
outcome: partial
evidence: PR #1343 at 3d2afb8a81d7 (measured head ad99d407e720d413f5795de263ec96656e08bcb2 = build/scanner-a3-k3-head, sole parent 07ab2e7e4cb0b037933ba5e1a1177b8fa353dd37); comment #1343#issuecomment-7238118890; docs/notebook/2026-11-28-rss-bound-calibration.md

**What was asked:** independent C6 review of PR #1343's adoption registration, frozen i9 recipe, and Rust implementation before setting `ready/bench-scanner-a3-k3`. No hold taken; PR branch untouched.

**Verdict: CHANGES REQUESTED** on three small pre-tag recipe/registration items. The measured Rust commit `ad99d407` itself **passes** (no changes needed to `src/` or `build/scanner-a3-k3-head`).

**Requested pre-tag fixes:**
1. **Calibrate peak-RSS gate (`bench/scanner_a3_k3.py:52,185-187`, `docs/journal/2026-11-28-scanner-a3-k3/registration.md:44`):** `score()` gates all 10 real cells on an uncalibrated ratio of arm medians (`statistics.median(rss[1]) / statistics.median(rss[0]) > 1.02`) with no CI. On this same i9-10900K and `scanner_coalesced_run` binary, zero-worker-heap baselines exceed `1.02` at `k >= 8` (`#1119` `wikidata/8` `medR = 1.0227` vs paired GM `1.0089`, `random/8` `1.0226`; `#1034` `random/8` GM `1.0233`, `medR = 1.0344`). Switch to `paired_ratio_ci(rss[1], rss[0])` and apply `docs/notebook/2026-11-28-rss-bound-calibration.md` §5 bands (`k = 1 <= 1.005` or `<= +256 KiB`; `k >= 4` primary `<= 1.030`, high-variance guards `<= 1.050`, or paired GM + `rss_lo > 1.0`) while keeping `stop_test` text unchanged.
2. **Guard ambient build env & distinct binaries (`bench/scanner_a3_k3.py:106-120,232`):** line 110 passes `{**os.environ, ...}` to `cargo build` without rejecting non-empty `RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, or `CARGO_PROFILE_RELEASE_*`; lines 117-120 omit a `distinct_binaries` check (`head-real != base-real`, `head-phases != base-phases`); line 232 omits `allow_nan=False` on `parity.jsonl`.
3. **Formatting (`bench/test_scanner_a3_k3.py:17-18,126-129`):** `ruff format --check` fails on `bench/test_scanner_a3_k3.py`.

**Verified PASS:**
- **Rust (`ad99d407`, `src/bits.rs`, `src/block.rs`, `src/huff.rs`):** zero new `unsafe` (`#![forbid(unsafe_code)]` in `huff.rs`, `#![deny(unsafe_op_in_unsafe_fn)]` in `block.rs`, existing `decode_runs_bmi2` unchanged in safety scope); LUT bit-packing, refill `const` asserts (`46 >= 40`, `56 >= 50`), `fill_triples` `c23 == 3` length handling and alignment, `Patches` capacity (`1042` vs `CHUNK_STOP = 1000`), and `eob == 256` precedence verified.
- **Local checks on `3d2afb8a81d7`:** `cargo test --lib -- block:: huff:: bits::` 11/11 passed; `cargo +nightly miri test --lib -- block::tests::miri_` 2/2 clean; `pytest -v bench/test_scanner_a3_k3.py` 6/6 passed; `ruff check` clean; `frontier.py check` 0 errors/0 warnings.
- **Provenance & i9 harness:** `build/scanner-a3-k3-head == ad99d407` with sole parent `07ab2e7e` and matching `phases.rs` blob `403cfbbd...`; `tmp/base` worktree stays alive past `ab gen`; plain `ab env` emits `"cpu_quota": null` (no `--facts` dict VOID); parity checks streamed stdout SHA-256 against `bz2.open`; `.just/bench.just` finishes via `bench/bundle.sh`. Minor doc note: `registration.md:10` says recipe/registration "live on master" (update to reviewed branch / ready tag until merged).

**What changed:** added this independent review delta only (non-performance path; zero staling of `7be50513`).

**What's next:** PR #1343's author updates the three recipe/registration items and re-checks formatting/tests before dispatching `ready/bench-scanner-a3-k3` and adding the queue entry.
