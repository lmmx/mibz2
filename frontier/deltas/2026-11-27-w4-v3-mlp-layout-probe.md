---
date: 2026-11-27
claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/w4-v3-mlp-layout-probe.md
---

W4_v3 MLP & memory-layout smooth-zone probe (`examples/w4_v3_mlp_layout_probe.rs`, zero `src/` edits, 100% safe Rust, 233/233 byte+CRC32 exact on `wikidata.bz2`, `ovf=0`, `fb=0`, resident scratch `3,383–3,493 KiB <= 4.0 MiB`, 7 interleaved repeats warm + L3-cold across baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned `-C target-cpu=native`):

- **Frozen stop-test verdict — STOP:** Zero of 12 candidate arms beat `W4_v2_ctrl` (`K=4, S=16, SLAB=32 KiB`, coupled) by the frozen bars (`A6c+A7+A9 <= -5.0%` AND `total <= -3.0%` disjoint on baseline and native). Coupled `K=4, S=16, SLAB=32 KiB` wins every compiler profile uncontested (`609.70 ms` base, `646.98 ms` native, `551.46 ms` JCC-aligned L3-cold CPU).
- **Wider MLP (`K = 6, 8, 10` with `K * SLAB <= 128 KiB`, `eff_K = 5.07 -> 7.68`) regresses across all 3 profiles:** `+4.8%..+14.4%` total L3-cold CPU (`+10.8%..+45.4%` on `A6c+A7+A9`). `objdump -d` shows the 15-GPR wall: `4K = 24..40` live scalars (`j, ed, a, e`) spill all `K` edge words and slab cursors to stack every iteration (`0x3e2a8..0x3e356`), while `edges` (`456 KiB` avg) fits in L3 (~35 cycles) and `16 -> 64` segments quadruple compaction/GF(2) combines.
- **Decoupled batch-chase (`B = 8, 16`), 1-step pipelining (`PIPE`), arena hoisting (`HOIST`), `SLAB = 8 KiB`, and `Inline15Len` (`FUSED15`) all miss:** Decoupling loses `+56 ms` even at `K=4` (`+8.2%..+14.7%` at `K=8` L3-cold) by filling the 192-entry ROB and destroying free OoO chase/expand overlap plus `&mut a` store-forwarding stalls (`0x150(%rsp)`); `6.43%` `LONG` steps break `all_fast` arena hoisting on `23.3%` (`K=4`) / `41.2%` (`K=8`) of rounds (`+4.1%..+20.7%`); 1-step pipelining adds `K` live regs (`+3.8%..+15.3%`); `S=16 -> S=32` at `K=4` (`eff_K 3.15 -> 3.49`) is flat (`+0.01%..+2.71%`); `Inline15Len` routes `len == 16` rules to `LONG` (`-0.8%..+2.6%` vs `K=8`).
- **JCC-32B control bonus & next focus:** On `-C target-cpu=native`, 32B branch alignment cuts `W4_v2_ctrl`'s own L3-cold `A7+A9` from `277.31` to `227.71 ms` (`-17.9%`; total `551.46 ms`). Scoped dead-end appended to `frontier/dead-ends.tsv`; closing the remaining `~6–11 ms` gap to `#1056`'s `6 GB/s` contour requires real i9-10900K measurement of `W4_v2` (C5) and/or cuts outside unchanged-`m` `A7` lane count.
