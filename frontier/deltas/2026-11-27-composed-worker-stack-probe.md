claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/composed-worker-stack-probe.md, examples/composed_worker_stack_probe.rs

**What was asked:** Answer `frontier/DECODER.md`'s composition rule by composing all four independently validated worker-side mechanisms (`#1055` L1-batched MTF + `OutQ16` `496/512`, `#1046` `Fixed64kSoa` + `GenCache`, `#1057` forward SIMD wire + `S = 16` 4-lane emission, `#1045` in-place L2-hot RLE1 + PCLMULQDQ CRC32 + GF(2) combine) into one real end-to-end Open block decoder (`A5 + A6a + A6b + A6c + A7 + A9`, zero `src/` edits) across all 233 admitted Open blocks of `wikidata.bz2`, measuring real speedup, cumulative marginals, exact interaction residual, and per-worker working set.
**What I found:**
- **Exactness & working set (233/233 byte+CRC32 exact across all 9 arms):** `W0_shipped` = `2,896.6 KiB`; `W1` (`+#1055`) = `2,140.2 KiB` (**`-756.4 KiB`** by removing `front.runs`); `W2` (`+#1046`) = `3,265.7 KiB` resident (`4,637.7` virtual); `W3` stitched = `4,402.4 KiB`; `W3b` vectored unfused = `6,893.2 KiB`; **`W4_full_fused_vec` = `3,488.5 KiB` resident** (`4,860.5` virtual, **half of `W3b`**, only `+591.9 KiB` over `W0`; `27.3 MiB` across 8W).
- **Composed end-to-end Open worker speedups (7 interleaved repeats, steal-free `schedstat` CPU, DISJOINT FASTER vs `W0` in all 4 regimes):**
  - Baseline x86-64 WARM: `W0 737.06 -> W1 678.76 (-7.91%) -> W2 650.28 (-11.77%) -> W3b 553.27 (-24.94%) -> W4 548.46 ms` (**`-25.59%`, `0.7441x`**; interaction residual **`-0.07%`**, perfect additivity; warm wall `-25.43%`).
  - Baseline x86-64 L3-COLD (16W pool >50 MiB + 64 MiB flush): `W0 779.26 -> W1 726.74 (-6.74%) -> W2 692.62 (-11.12%) -> W3b 604.09 (-22.48%, -0.03% res.) -> W4 615.29 ms` (**`-21.04%`, `0.7896x`**, min `589.86 ms`; residual `-1.96%`).
  - Native WARM: `W0 690.47 -> W1 674.55 (-2.31%) -> W2 641.36 (-7.11%) -> W3b 543.81 (-21.24%) -> W4 553.78 ms` (**`-19.80%`, `0.8020x`**; **+3.80% positive synergy**, `W3b` +3.98%).
  - Native L3-COLD: `W0 754.63 -> W1 710.44 (-5.86%) -> W2 676.19 (-10.39%) -> W3_stitch 626.82 (-16.94%) -> W3b 596.59 (-20.94%) -> W4 597.61 ms` (**`-20.81%`, `0.7919x`**, `+29.2 ms` faster than stitched `W3`; residual `-2.62%`).
- **Two actionable integration findings:** (1) Eliminating the stitch copy (`W3` -> `W3b`/`W4`) saves `+29–30 ms` under native L3-cold; (2) `W4`'s 32 KiB slab invokes `compact_and_crc_slab` 15–30x/blk, costing ~5–10 ms warm vs `W3b` while halving working set (`3.49` vs `6.89 MiB`); raising the tile to 64 KiB + cold-outlining compaction recovers that warm tax.
- **Conditional i9-10900K `6 GB/s` mapping (VM diagnostic only, C5):** `W4` closes 70%–92% of the `51.8 ms` worker gap (`W/8 = 185.68 -> 138.17–148.92 ms` vs `#1056` `133.9 ms` threshold; 1W save `+252–325 ms`). Remaining `W4` time is still `46%` `A7+A9`, `29%` `A5+A6a`, `17%` `A6b`, `7%` `A6c`.
**What changed:** Added `examples/composed_worker_stack_probe.rs`, `docs/journal/2026-11-27-explore/composed-worker-stack-probe.md`, and this delta. Zero `src/` edits.
**What's next:** Proceed with integrating `#1055`, `#1046`, `#1057`, and `#1045` (with 64 KiB cold-outlined slab compaction) in order; benchmark real 1W/8W i9-10900K hardware and target remaining `A7` (46%) + `A5+A6a` (29%) if a `~4–15 ms` `W'/8` gap remains.
**Caveats:** VM 2-vCPU diagnostic (C5); `schedstat` CPU timer is clean while concurrent VM steal inflated wall `Instant` spreads (reported transparently in the journal).
