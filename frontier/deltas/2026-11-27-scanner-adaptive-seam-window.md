claim: scanner-adaptive-seam-window
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-27-scanner-adaptive-seam-window/registration.md; src/stream.rs; src/parallel.rs; src/trace.rs; #1141 (d5f8575b)

**What was asked:** Implement the safe-Rust adaptive per-block lookahead (`src/stream.rs`) and one-shot seam window (`src/parallel.rs`, `src/trace.rs`) rescue for `in_maint`, plus a frozen shallow-safe i9-10900K C6 adoption registration, preserving `#1031`'s frozen `NOT ADOPTED` verdict and `#1129`'s ready-cap worker gate.
**What I changed:**
- `src/stream.rs` (`#![forbid(unsafe_code)]`): replaced sticky `LOOKAHEAD = 2 MiB` with a non-sticky 16-block sliding-window peak `(max16 + (max16 >> 2) + 4 KiB).clamp(64 KiB, 1 MiB)` (first block `128 KiB`), retaining exact `br.overran()` retry doubling up to `MAX_ITEM = 64 MiB`. Added `adaptive_lookahead_and_retry_growth` and `miri_adaptive_lookahead_clamps_and_decays_after_16_blocks`.
- `src/parallel.rs` (zero new `unsafe`): replaced unconditional chunk append/drain in `ChannelSource` with in-place chunk scanning, parked `next` (`next_base`), one-shot seam fill (`SEAM_ONESHOT_EXTRA = 128 KiB`) into a reused `seam` buffer, direct swap back to `next` once `released >= next_base`, lock-free cancellation via `self.shared.stopped.load(Ordering::Acquire)`, `INPUT_QUEUE + 2` spare chunk recycling outside the lock, and native `WindowResume` / `WindowWorkEnd` trace marks. Added `channel_source_tests` (`views_are_the_input`, `stream_decoder_seam_and_error_parity`, `traced_window_maintenance_spans_balance_and_stay_bounded`). Preserved `#1129` (`ready-cap-worker-gate`) untouched.
- `src/trace.rs`: added `Kind::WindowResume` and `Kind::WindowWorkEnd`, updated `LOOKAHEAD` to `1 MiB`, and updated `chunks` and `window` ledger notes.
- `docs/journal/2026-11-27-scanner-adaptive-seam-window/registration.md`: froze the shallow-safe i9-10900K C6 adoption registration verbatim to the claim's frozen `stop_test` (`>= 85%` `in_maint` drop at 8W, both `10` and `12` threads `FASTER`, no `SLOWER` at `6`/`8`/`9`, median peak RSS `<= 1.05x` at every count, untimed `libbz2` SHA-256 parity).
**Local verification:**
- `cargo fmt --check` clean; `cargo clippy --all-targets` (`default`, `--features trace`, `--features scan-oracle`) with `-D warnings`: 0 warnings.
- Release unit & integration tests: `stream::` + `parallel::channel_source_tests` + `trace::` (9/9 default, 8/8 `--features trace`), `parallel::plateau_tests` (9/9), `decode` (14/14), `pipelined` (12/12), `service_default` (4/4) all pass.
- Miri: `miri_adaptive_lookahead_clamps_and_decays_after_16_blocks` passes cleanly (`0.15 s`).
- Traced `occupancy` on `wikidata.bz2` (4T): parity ok, ledger check ok, events reconciled ok, `window live <= 4.2 MB, reserved <= 4.4 MB` (down from `16.0 MiB`, `-72.6%`).
**What's next:** Independent C6 review of code diff and frozen registration (`status = "review"`), exact-head green CI (incl. paired `bench`, `miri`, `fuzz-smoke`, `occupancy`), then shallow-safe i9 workstation validation under C6 before any adoption merge.
**Caveats:** Do not self-merge or queue before independent C6 review and green exact-head CI; real i9 wall/RSS/`in_maint` adoption awaits the registered i9 run.
