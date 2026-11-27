claim: open-cap-prescan-cost
by: antigravity
outcome: done
evidence: src/open.rs; src/open/dense.rs; origin/evidence 5182d546; docs/notebook/2026-11-27-synthesis-seventeenth-pass.md; frontier/claims/open-cap-prescan-cost.toml; frontier/surface.tsv

**What was asked:** Diagnose whether the `within_cap` scalar pre-scan added by `#1222` (`0.5.17`) on `0.10 <= r/n < 0.15` blocks caused the `0.5.17` i9 `release-scaling` regression (`5182d546`: `0.5.16/0.5.17` wall `0.974 [0.967, 0.982]` at 8T and `0.985 [0.976, 0.995]` at 10T, `+8–9 ms` CPU), and if confirmed, eliminate it with every block's route unchanged (`0` new `unsafe`).

**Gate 0 (Diagnostic, recorded before build, VM 2-vCPU Xeon):**
- **(a) `within_cap` cost on `wikidata.bz2` (`82f65ace`, 431 blocks, 233 full):**
  - Exact block count in `0.10 <= r/n < 0.15` (`lazy_counts_admit && r/n >= 0.10`): **30 full blocks** (`3,306,468` records total), **30/30 admitted** (plus 5 dense blocks in `[0.15, 0.21)`).
  - Pinned to CPU 0 (`taskset -c 0`, 7 passes): **`23.442 ms` (min of 7)** / `31.039 ms` median (`32.169, 36.140, 31.039, 35.100, 27.520, 23.442, 23.491 ms`).
- **(b) Paired diagnostic (`v0.5.17` `01acd74d` with `Cap::NONE` restored for `lazy_counts_admit` vs `v0.5.17`, 15 alternating interleaved rounds, `ab child`, `wikidata.bz2`, 60/60 runs hash `a8830caee5c511a6`):**
  - `mibz2-workers-lazy` 1T (pinned CPU 1): CPU median `924.6 → 894.4 ms` (`−30.2 ms`), paired ratio **`0.9664 [0.9564, 0.9765]`** (`−3.36%`); wall `0.9551 [0.8769, 1.0403]`; RSS `24.11 → 24.12 MiB`.
  - `mibz2` 8T: CPU median `1004.6 → 968.4 ms` (`−36.2 ms`), paired ratio **`0.9766 [0.9611, 0.9922]`** (`−2.34%`); wall `0.9865 [0.9608, 1.0129]`; RSS `51.34 → 51.37 MiB`.
  - **Verdict:** Pre-scan hypothesis decisively confirmed (both 95% CI upper bounds `< 1.000`, matching Gate 0(a)'s `~31 ms` median scan).

**What I changed (`0` new `unsafe`, route-preserving):**
- `src/open.rs` (`OpenCellsOf::invert_runs`): skip the separate `within_cap` pre-scan when `crate::cells::lazy_counts_admit(recs.len(), n)` holds (`0.10 <= r/n < 0.15`). `Kernel::try_run` already enforces the exact same `cap = Cap::dense(n)` inside Pass 1 with sticky refusal and falls back cleanly to `Segmented` on overflow, so all 30/30 Wikidata `[0.10, 0.15)` blocks pay zero pre-scan overhead while high-`m/n` blocks (`text` `m/n = 0.42`) still refuse identically. Kept `within_cap` for `[0.15, 0.21)` where most off-Wikidata dense blocks refuse (`#1040`).
- `src/open/dense.rs`: updated `within_cap_matches_construction_and_skips_first_pass` to verify both `[0.15, 0.21)` precheck skip and `[0.10, 0.15)` Pass-1 cap refusal, and added `route_census_matches_v0517_across_corpora` asserting 100% route identity against `v0.5.17`'s `within_cap` oracle and byte-identical output.

**Gate 1 (Exactness & route identity):**
- **489/489 blocks across all 7 CI bench corpora 100% route-identical to `v0.5.17` and byte-identical** (`onebyte`: 1/0/1, `pattern`: 22/0/22, `random`: 9/0/9, `records`: 11/0/11, `runs`: 1/0/1, `text`: 14/0/14, `wikidata`: 431/233/198 open/seg).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (default, `forward-anchor-lanes`, `trace`, `mtf-fold`), `open::` unit tests, and Miri (`miri_open_kernel_small_columns`, `open_cells_gate_edges`, `dense_band_gate_edges`): all PASS, `0` new `unsafe`.

**Gate 2 (Acceptance — 15 alternating interleaved paired native rounds, head vs `v0.5.17` `01acd74d`, `wikidata.bz2`, 60/60 runs hash `a8830caee5c511a6`):**
- `mibz2-workers-lazy` 1T (pinned CPU 1): CPU median `966.0 → 925.7 ms` (`−40.3 ms`), ratio **`0.9588 [0.9438, 0.9740]`** (`−4.12%`, 95% CI upper `0.9740 < 1.000` PASS); wall `0.9366 [0.8653, 1.0137]` (`−6.34%`, no wall loss); RSS `24.06 → 24.27 MiB`.
- `mibz2` 8T: CPU median `1006.5 → 974.8 ms` (`−31.7 ms`), ratio **`0.9707 [0.9626, 0.9789]`** (`−2.93%`, 95% CI upper `0.9789 < 1.000` PASS); wall `945.2 → 916.2 ms`, ratio **`0.9692 [0.9429, 0.9963]`** (`−3.08%`, faster beyond noise); RSS `51.26 → 50.89 MiB` (`1.0042 [0.9727, 1.0367]`).
- **What's next:** Exact-head CI (`bench`, `miri`, `rust`, `fuzz-smoke`), author-merge, `frontier/ledger.tsv` row, and `0.5.18` release-scaling i9 readback (no separate workstation run per stop_test).
