claim: post-1080-open-admission
by: antigravity
outcome: done
evidence: src/open.rs; src/open/b5.rs; src/open/dense.rs; docs/journal/2026-11-27-explore/route-crossover-post-1080.md; docs/journal/2026-11-27-explore/composed-post-0516-stack.md; docs/journal/2026-11-27-explore/eager-cells-and-seq-open-dissection.md; frontier/claims/post-1080-open-admission.toml; frontier/surface.tsv

**What was asked:** Recalibrate `OpenCellsOf::cap` in `src/open.rs` post-`#1080` (`Segmented` -15.4%..-18.5%) to eliminate misrouted high-`m/n` `OpenLow` blocks (`text` `m/n = 0.42`) and wasted `within_cap` scans on the losing upper dense tail, while preserving all 233/233 full `wikidata` blocks in `OpenCells` so 8-worker `OpenCells` workers never allocate `Segmented`'s fallback scratch (`#1194` & `#1210` memory caution).
**What I changed (`0` new `unsafe`):**
- `src/open.rs`:
  - Replaced `OpenCellsOf::fallback` from `LazyCells` to `Segmented` directly.
  - Recalibrated `OpenCellsOf::cap` to the three-band rule (`OPEN_OUTRIGHT = 1/10`, `OPEN_BAND = 21/100`, `DENSE_M_RATIO = 27/100`):
    1. `1/64 <= r/n < 0.10`: admitted outright (`Cap::NONE`, 0 `within_cap` scan overhead on 194/233 full `wikidata` blocks);
    2. `0.10 <= r/n < 0.21`: gated by `within_cap` against `m <= min(256_000, 0.27 * n)` (`243,000` at `n = 900,000`) and `strips <= 16_384`, retaining all 233/233 full `wikidata` blocks (including its 5 dense blocks at `r/n = 0.151..0.206`, `m/n = 0.221..0.257`, max `m = 231,600 < 243,000`) inside `OpenCells` while catching high-`m/n` `text` `OpenLow` blocks (`r/n = 0.142..0.146`, `m/n = 0.419..0.423`) and routing them straight to `Segmented`;
    3. `r/n >= 0.21` (and `r/n < 1/64`): straight to `Segmented` without running `within_cap` (dropping the dead `[0.21, 0.28)` dense tail).
- `src/open/b5.rs` & `src/open/dense.rs`: updated gate-edge, three-band routing, `Cap::dense` (`0.27 * n`), and `within_cap` precheck refusal tests.
**Why `[0.10, 0.21)` with `m <= 0.27 * n` instead of `R2_2tier_m22` (`< 0.15`, `m <= 0.22 * n`):**
- Both `#1194` (`eager-cells-and-seq-open-dissection.md` §3.4) and `#1210` (`composed-post-0516-stack.md`) showed that refusing any of `wikidata`'s 5 dense blocks (`r/n = 0.151..0.206`, `m/n = 0.221..0.257`) or 4 borderline `0.10..0.15` blocks forces every `OpenCells` worker to permanently allocate `Segmented`'s `~6.5–8.4 MiB/worker` scratch (`+35..40 MiB` / `1.69–1.80x` 8W RSS under `m22`; `+27.7 MiB` / `1.56x` 8W RSS under `[0.15, 0.28)` removal alone).
- Widening the capped band to `[0.10, 0.21)` with `m <= 0.27 * n` keeps 233/233 full `wikidata` blocks in `OpenCells` (`0` `Segmented` spill), bringing `wikidata` 1W RSS to `0.9947x` (`24.15 -> 24.02 MiB`), 8W `OpenCells` CPU to `-1.88%` (`971.0 -> 952.7 ms`, wall `-2.56%`, RSS `50.30 -> 52.12 MiB` vs `84.9–90.7 MiB` under `m22`), and `wikidata` 4T RSS to `0.9920x` (`82.55 -> 81.89 MiB`), while cutting `text` 1W whole-decoder CPU by `-4.44%` (`225.2 -> 215.2 ms`).
**Verification:**
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (default, `forward-anchor-lanes`, `trace`), `cargo clippy --lib --tests --features mtf-fold -- -D warnings`: 0 warnings, 0 new `unsafe`.
- `cargo test --release --lib -- open::` (40/40), `decode` (14/14), `pipelined` (12/12), `service_default` (4/4), and Miri (`miri_open_kernel_small_columns`, `open_cells_gate_edges`, `dense_band_gate_edges`): all PASS.
- 14/14 byte-and-hash exact across all 7 CI corpora (`t = 1 OpenCells`, `t = 4 mibz2`).
**What's next:** Exact-head CI paired `bench`, `miri`, `rust`, `fuzz-smoke`, and `ledger.tsv` row before author-merge.
