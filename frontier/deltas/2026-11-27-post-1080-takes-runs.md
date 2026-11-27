claim: post-1080-takes-runs
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/runs-takes-runs-post-1080.md; src/runs.rs; src/runs/tests.rs

Shipped `post-1080-takes-runs` in `src/runs.rs` and `src/runs/tests.rs` (zero new `unsafe`): widened `takes_runs_slice` from `2 * len > max_len` (`0.50`) to `4 * len > 3 * max_len` (`0.75`), keeping `RUN_SHARE = 32` (`>= 1/32`) on `<= 0.50` and requiring `WIDENED_RUN_SHARE = 8` (`>= 1/8 = 12.5%`) in the widened `(0.50, 0.75]` band (admitting all 40 compressible `mdjournal`/`elf`/`json`/`prose`/`pysrc` blocks with 17–49% run symbols while keeping short small-alphabet random noise like 26-letter blocks at ~3.8% runs on `Column`), plus boundary and widened-band decode parity tests (`runs_takes_runs_bounds`, `runs_widened_band_decode_parity`).

- **Evidence (pre-registered 14-corpus 3-build steal-free probe `docs/journal/2026-11-27-explore/runs-takes-runs-post-1080.md`, 658/658 blocks byte+CRC exact):** Cuts MTF+inverse-BWT CPU disjointly FASTER across baseline x86-64 / `-C target-cpu=native` / JCC-32B-aligned builds by `-24.1% / -32.4% / -13.2%` on `mdjournal` (10/10 blocks won), `-15.5% / -24.7% / -14.5%` on `elf` (25/26 blocks won), and `-9.9% / -9.8% / -9.0%` on `json` (2/2 blocks won), plus `-4.3%..-8.4%` on `prose` and `-3.8%..-6.2%` on `pysrc`, with 0/42 disjoint regressions and 0/549 blocks changing route on `wikidata`, `text`, `random`, `records`, `runs`, `onebyte`, and `pattern`.
- **Verification:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --release --lib runs::`, `test_ledger.py`, `frontier.py check`, plus exact-head CI (`rust`, `miri`, `bench`, `occupancy`, `fuzz-smoke`, `moonshot`, `submit`, wheels).
- **Next:** Claim slot freed (`12 -> 11`) for `segmented-fused-postwalk` (`src/bwt.rs`, `src/rle.rs`, `src/lib.rs`).
