---
date: 2026-11-27
claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/composed-worker-v2-probe.md
---

Composed Worker Stack v2 smooth-zone probe (`examples/composed_worker_v2_probe.rs`, zero `src/` edits, 233/233 byte+CRC32 exact on `wikidata.bz2`, `ovf_chunks = 0` across all 10 arms, warm + L3-cold across baseline x86-64, `-C target-cpu=native`, and JCC-aligned `-C target-cpu=native`):

- **Upgrade (b) split — ADOPT `#[cold] #[inline(never)]` @ `32 KiB`, REJECT `64 KiB`:** Outlining `compact_and_crc_slab` at `SLAB = 32 KiB` (`4 × 32 KiB = 128 KiB` = 50% of 256 KiB L2) cuts `W4_v1` by `-11.4..-15.8 ms` native warm (`-27.44%` CPU / `W'/8 = 134.74 ms`, `-27.89%` wall / `133.89 ms` on quiet VM) and `-5.1..-11.0 ms` in L3-cold across all 3 profiles with zero working-set increase (`3,488.5 KiB`). Raising `SLAB` to `64 KiB` (`4 × 64 KiB = 256 KiB` = 100% L2) evicts slabs before compaction and regresses **`+12..+25 ms`** in `A7+A9` across every profile while adding `+325 KiB`/worker.
- **Upgrade (a) (`#1059` hoisted `OutQ16` `496/512` & `1008/1024`, with cold-outlined `flush_batch_fallback` & `grow_edges`):** Combined `W4_v2_q496_32k` / `W4_v2_q1008_32k` beats `#1062`'s `W4_v1` by **`-15.0..-17.3 ms` on native** (`-23.5%` warm, `-21.6%` cold), **`-35.2 ms` on baseline cold (`1008/1024`, `-22.81%`)**, and **`-15.4 ms` on JCC-aligned cold (`496/512`, `-19.81%`)**, while widening the L3-cold RLE1+CRC fusion edge over unfused `W3b_v2` to **`-7.2..-15.6 ms`** at half the scratch (`3,486.5 KiB` vs `6,891.2 KiB`).
- **JCC-erratum & `6 GB/s` contour verdict (`#1056` `W'/8 <= 133.9 ms` / `135.4 ms` small-ch0):** Quiet-VM native warm (`W4_b_32k_cold`) clears `135.4 ms` and touches `133.9 ms`; realistic L3-cold under matched build flags reaches `-21.6%..-24.6%` (`W'/8 = 140.0..145.5 ms`, `~6–11 ms` short of `133.9 ms`). JCC-aligned flags cut `W0_shipped` itself by `-81..-86 ms` (`-11..12%`); vs unaligned `W0`, JCC-aligned `W4_v2_q496_32k` is `-28.2%` warm / `-26.5%` cold. Real i9-10900K measurement (including JCC alignment test) is required before claiming `6 GB/s` (C5).
