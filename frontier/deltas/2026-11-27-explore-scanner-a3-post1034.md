claim: explore
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/scanner-a3-post1034-dissection.md; examples/scanner_a3_post1034_dissection.rs

Completed deep post-`#1034` `A3` (`decode_runs_bmi2` / `decode_runs_scalar` + `Huff::build_into`) dissection and redesign probe across 6 corpora (`wikidata.bz2` 431 blocks / 24,979,299 symbols + 5 `ab gen` guards `text`, `records`, `random`, `runs`, `onebyte` + synthetic `alpha = 258`), 11 arms, and all 4 compiler profiles (Baseline x86-64, Baseline + JCC-32B, Native, Native + JCC-32B; 7 interleaved rotated repeats on pinned CPU 0 using vDSO `CLOCK_THREAD_CPUTIME_ID`).

**Frozen Stop-Test Verdict: PASS** across all 4 compiler builds:
- **`C6_k3hi_pipe4`** (high-2-bit `cnt` `B=10, K=3` LUT packing `SYM1/2/3` in bits `6..29` with `bits 4..5 == 0` for zero-`and` `shlx`, single 32-bit `movl` store into `chunk_u8: [u8; 1042]`, 32-entry cold `PatchBuf` for `sym == 256`, pipelined 4-step overlapped refill hiding refill under Step 3's 5-cycle L1d load, 100% in-place zero-copy step-jumping `K = 3` build gated at `K3_GATE_SEL = 1024`, compile-time `max_batch_syms = 12` for zero GPR spills, `ScanScratch` `60.3` vs `59.8 KiB` = `1.008x`):
  - **Baseline x86-64:** `wikidata` paid **`-5.64%`** disjoint (`68.545` vs `72.645 ms`; decode `-9.30%`), `text -7.88%`, `random -1.35%`, `runs -0.21%`, `records +0.93%`, `onebyte -2.33%`.
  - **Baseline + JCC-32B:** `wikidata` paid **`-6.15%`** disjoint (`66.639` vs `71.008 ms`; decode `-9.65%`), `text -11.61%` (disjoint), `random -3.54%` (disjoint), `runs -5.22%` (disjoint), `records +1.23%`, `onebyte +3.28%` (`0.004 vs 0.004 ms`, ~140 ns floor, overlapping).
  - **Native (`-C target-cpu=native`):** `wikidata` paid **`-7.65%`** disjoint (`65.992` vs `71.461 ms`; decode `-10.95%`), `text -11.33%` (disjoint), `random -0.20%`, `runs -6.28%`, `records -1.08%`, `onebyte -2.56%`.
  - **Native + JCC-32B:** `wikidata` paid **`-7.48%`** disjoint (`64.497` vs `69.709 ms`; decode `-11.33%`), `text -11.40%` (disjoint), `random -1.78%`, `runs -6.43%` (disjoint), `records -0.58%`, `onebyte +0.24%`.
- Projects **`~6.5–8.8 ms`** off physical i9-10900K post-`#1034` serial scan `S'` (`143.5 ms`) and **`-10..11%`** off scanner-bound `text` at 8W, stacking orthogonally with `scanner-a2-fast-build` (`#1069`) and `scanner-adaptive-seam-window` (`#1141`).
- Opened build claim `frontier/claims/scanner-a3-k3-pipe-u8.toml` at `status = "idea"` and recorded scoped dead-ends (`B=9 K=3`, `B=11 K=2`, split hot/cold 24 KiB LUTs, deferred `sum_e`/runtime `max_k` 15th-GPR spills) in `frontier/dead-ends.tsv`.
