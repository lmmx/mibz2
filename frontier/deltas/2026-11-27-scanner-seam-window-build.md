claim: scanner-seam-window
by: antigravity
outcome: partial
evidence: #1028, #1031, #857, docs/journal/2026-11-27-scanner-seam-window/registration.md, bench/scanner_seam_window.py, bench/test_scanner_seam_window.py

**What was asked:** Admit and claim `scanner-seam-window`, port PR #857's input-chunk seam window cleanly onto `master` without touching `src/block.rs`, add thorough seam/parity tests, freeze the C6 adoption registration before any workstation data, and prepare the workstation recipe for independent review.
**What I found:**
- Admitted `scanner-seam-window` (`idea` -> `open`) in #1028 (10 of 12 open claims) and acquired atomic hold (session `6513527b965e`).
- PR #857 (`ChannelSource::view` reused seam buffer + O(1) `next` swap) ported cleanly onto `origin/master` (`cf242fe7`) with zero new `unsafe` and zero edits to `src/block.rs` (disjoint from `scanner-coalesced-u8-decode`).
- Local tests passed: `cargo test --lib` (200/200, including 6 `channel_source_tests`), `cargo test --features phase-counters --test phase --test phase_off` (2/2), `cargo test --test service_default` (4/4), `cargo test --test decode --test pipelined` (26/26), `cargo clippy --lib --tests -- -D warnings`, `cargo fmt --check --all`, release builds (`ab`, `occupancy --features trace`), and `python3 bench/test_scanner_seam_window.py` (8/8).
- Local VM diagnostic only (C5, synthetic 48 MiB = 12 x 4 MiB chunks, 2 MiB lookahead): peak window capacity dropped 4x (16.0 MiB -> 4.0 MiB) and pure maintenance time dropped 46.4% (17.35 ms -> 9.30 ms); `occupancy --features trace` on a 20.74 MiB synthetic stream reconciled cleanly (`window live <= 4.2 MB, reserved <= 4.2 MB`).
**What changed:**
- Code & docs: `src/parallel.rs` (seam window + 3 new edge/parity tests: mid-seam EOF, multi-chunk < lookahead + cancellation, end-to-end byte/error parity), `src/trace.rs` (ledger notes), `docs/ledger.md`.
- Frozen C6 registration & workstation tooling: `docs/journal/2026-11-27-scanner-seam-window/registration.md` (matching `scanner-seam-window.toml`'s `stop_test` verbatim), `.just/bench.just` (`bench-scanner-seam-window`), `bench/scanner_seam_window.py`, `bench/test_scanner_seam_window.py`.
- Frontier: `frontier/claims/scanner-seam-window.toml` marked `status = "review"`, `pr = 1031`; this delta added. `bench/workstation-queue.txt` untouched pending C6 review.
**What's next:** One independent agent reviews PR #1031 (code diff, frozen C6 adoption registration, and recipe) once CI is green; then queue `bench-scanner-seam-window` on the i9-10900K under the standing workstation authorisation.
**Caveats:** Local VM diagnostic is C5 non-decision wiring only; adoption rests exclusively on the i9-10900K `wikidata x4` paired run under the frozen rule.
