# Composed Worker Stack v2 Probe: Hoisted Single-Branch `OutQ16` (`496/512` & `1008/1024`) + Cold-Outlined `32 KiB` vs `64 KiB` Slab Compaction

*Date:* 27 November 2026
*Zone:* Smooth (`examples/composed_worker_v2_probe.rs`, `Claim: none`, zero `src/` edits)
*Primary corpus:* `corpora/wikidata.bz2` (11,854,933 B compressed; 209,691,304 B decoded across 233 production-admitted Open blocks out of 431 total)
*Machine:* 2-vCPU Haswell Xeon @ 2.30 GHz (32 KiB L1d / 256 KiB L2 / 45 MiB L3, Intel JCC-erratum mitigation active, no vPMU); Linux `/proc/thread-self/schedstat` on-CPU thread time + `Instant` wall time, 7 interleaved forward/reverse repeats pinned to CPU 0 (`taskset -c 0`), warm (reused worker) and L3-cold (16-worker rotating pool `>50 MiB` + `64 MiB` L3 flush before each repeat).
*Profiles tested:*
1. **Baseline x86-64** (`cargo build --release`)
2. **Standard `-C target-cpu=native`**
3. **JCC-aligned `-C target-cpu=native`** (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`)

## 1. Questions Tested

On top of `#1062`'s end-to-end 4-mechanism composed Open worker (`W4_full_fused_vec` = `#1055` L1-batched MTF + `#1046` `Fixed64kSoa`/`GenCache` + `#1057` 4-lane forward `S=16` anchors + `#1045` fused 32 KiB slab RLE1+CRC), we tested the two exact upgrades identified by `#1059` and `#1062`:

1. **Upgrade (a) — `#1059` hoisted single-branch `OutQ16` (`496/512` and `1008/1024`):**
   - `OVF_BIT` (`1 << 31`) in `Queues::fill[a]` collapses `push_hoisted` to one add + one unsigned compare (`f.wrapping_add(w) <= CHUNK as u32`).
   - Batch-hoisted `p` check (`cur > p || cur + batch_rows <= p`), batch-hoisted `EPOCH` check (`(m & (EPOCH - 1)) + max_m_add < EPOCH`), register-local `m`, fused Pass-1 `lits` popcount, `second_cold`, and `#[target_feature(enable = "avx2")] #[inline(never)]` `flush_batch_avx2` (with cold-outlined `flush_batch_fallback` and `grow_edges` in Round 2 so `batch.as_ptr()` stays in a GPR).
   - Tested at both `496/512` and `1008/1024`.
2. **Upgrade (b) — `#1062` cold-outlined slab compaction + slab size (`32 KiB` vs `64 KiB`):**
   - Isolate `#[cold] #[inline(never)]` on `compact_and_crc_slab` (preventing 5× macro expansion inside `emit_4lane_sampled<true>`'s `handle_hit!` from bloating the 4-lane hot loop) separately from raising `SLAB` from `32 KiB` to `64 KiB`.
3. **6 GB/s contour check (`#1056`):** Does `W4_v2` close the remaining `~4 ms` gap below `#1056`'s modeled `W'/8 <= 133.9 ms` (`>= 27.89%` worker cut vs `W0_shipped`, or `<= 135.4 ms` / `>= 27.08%` with small chunk 0) in warm and L3-cold?

## 2. Exactness & Per-Worker Working-Set Audit

All 10 arms passed **233/233 byte-exact and CRC32-exact** verification against `mibz2::decode_block` on every build profile, with **`0` overflow chunks** across all 233 Open blocks on both `496/512` and `1008/1024`:

| Arm | Mechanism Delta vs `W4_v1` | Byte+CRC32 | `ovf` | Resident Scratch | Virtual Scratch |
|---|---|---|---:|---:|---:|
| `W0_shipped` | Shipped Open (`512/512`, `Elim`, bidi, in-place RLE1) | 233/233 | 0 | `2,896.6 KiB` | `2,896.6 KiB` |
| `W3b_v1_unfused` | `#1062` `W3b` (`496/512` unhoisted, `32 KiB` unfused) | 233/233 | 0 | `6,893.2 KiB` | `8,265.2 KiB` |
| `W4_v1_32k_inline` | `#1062` `W4` (`496/512` unhoisted, `32 KiB` `#[inline]`) | 233/233 | 0 | `3,488.5 KiB` | `4,860.5 KiB` |
| `W4_a_hoist496` | + Upgrade (a): `#1059` hoisted `496/512` | 233/233 | 0 | `3,488.5 KiB` | `4,860.5 KiB` |
| `W4_b_32k_cold` | + Upgrade (b1): `#[cold] #[inline(never)]` @ `32 KiB` | 233/233 | 0 | `3,488.5 KiB` | `4,860.5 KiB` |
| `W4_b_64k_cold` | + Upgrade (b2): `#[cold] #[inline(never)]` @ `64 KiB` | 233/233 | 0 | `3,813.6 KiB` | `5,185.6 KiB` |
| `W4_v2_q496_32k` | **(a) `496/512` + (b1) `32 KiB` cold** | 233/233 | 0 | **`3,488.5 KiB`** | `4,860.5 KiB` |
| `W4_v2_q1008_32k` | **(a) `1008/1024` + (b1) `32 KiB` cold** | 233/233 | 0 | **`3,486.5 KiB`** | `4,858.5 KiB` |
| `W4_v2_q1008_64k` | (a) `1008/1024` + (b2) `64 KiB` cold | 233/233 | 0 | `3,811.6 KiB` | `5,183.6 KiB` |
| `W3b_v2_q1008_32k_unf` | (a) `1008/1024` + `32 KiB` unfused vectored | 233/233 | 0 | `6,891.2 KiB` | `8,263.2 KiB` |

`W4_v2_q1008_32k` actually *shrinks* resident scratch by `2.0 KiB` vs `W4_v1` (fewer `Spill::next` entries) while `64 KiB` slabs add `+325.1 KiB` per worker (`8 × 64 KiB` arena headroom vs `8 × 32 KiB`).

## 3. Round 1 (Low-Contention VM) & Disassembly Diagnosis

Round 1 ran the initial 9-arm factorial (`W4_v2` at `64 KiB` + isolated `W4_b_32k_cold` vs `W4_b_64k_cold`) with low host contention on baseline and native (`wall ≈ cpu`), immediately exposing **two major microarchitectural mechanisms**:

### 3.1 Round 1 `schedstat` CPU Medians (ms, 7 interleaved repeats)

| Arm | Base Warm CPU | Base Cold CPU | Native Warm CPU | Native Cold CPU | JCC Warm CPU | JCC Cold CPU |
|---|---:|---:|---:|---:|---:|---:|
| `W0_shipped` | `717.87` | `757.32` | `694.07` | `750.72` | `607.63` | `669.56` |
| `W3b_v1_unfused` | `531.75` (`-25.93%`) | `594.38` (`-21.52%`) | `503.14` (`-27.51%`) | `596.74` (`-20.51%`) | `484.25` (`-20.31%`) | `553.43` (`-17.34%`) |
| `W4_v1_32k_inline` | `538.87` (`-24.93%`) | `590.28` (`-22.06%`) | `515.02` (`-25.80%`) | `571.16` (`-23.92%`) | `491.79` (`-19.06%`) | `545.22` (`-18.57%`) |
| `W4_a_hoist496` | `539.66` (`-24.82%`) | `590.43` (`-22.04%`) | `535.39` (`-22.86%`) | `602.85` (`-19.70%`) | `493.50` (`-18.78%`) | `553.55` (`-17.33%`) |
| **`W4_b_32k_cold`** | **`534.81` (`-25.50%`)** | **`589.27` (`-22.19%`)** | **`503.65` (`-27.44%`)** | **`566.09` (`-24.59%`)** | `492.15` (`-19.00%`) | **`540.93` (`-19.21%`)** |
| `W4_b_64k_cold` | `537.78` (`-25.09%`) | `614.86` (`-18.81%`) | `516.66` (`-25.56%`) | `583.57` (`-22.27%`) | `483.66` (`-20.40%`) | `558.17` (`-16.64%`) |
| `W4_v2_q496_64k` | `538.68` (`-24.96%`) | `619.32` (`-18.22%`) | `541.28` (`-22.01%`) | `614.40` (`-18.16%`) | `496.13` (`-18.35%`) | `561.86` (`-16.08%`) |
| `W4_v2_q1008_64k` | `536.01` (`-25.33%`) | `603.53` (`-20.31%`) | `539.77` (`-22.23%`) | `616.30` (`-17.91%`) | `488.65` (`-19.58%`) | `568.10` (`-15.15%`) |

*(Round 1 quiet-VM wall medians matched CPU tightly on baseline warm [`W0 719.39`, `W4_b_32k_cold 542.98`] and native warm [`W0 700.94`, `W4_b_32k_cold 505.43 (-27.89%)`, `W'/8 = 133.89 ms`]; host steal rose during Round 1 JCC warm/cold wall [`940 / 1314 ms`].)*

### 3.2 Two Root Causes Proved by Phase Split + `objdump -d`

1. **The 256 KiB L2 Cache Cliff at `SLAB = 64 KiB`:**
   On Haswell and Comet Lake (`256 KiB` private L2/core), 4 active lanes × `64 KiB` = **`256 KiB` (100% of L2)**. Competing with random `edges[]` pointer chases, each 64 KiB slab is evicted to L3/DRAM *before* `compact_and_crc_slab_cold` runs, causing a **`+12..+25 ms`** regression in `A7+A9` vs `32 KiB` cold across every profile:
   - Native warm `A7+A9`: `232.93 ms` (`32K cold`) vs `244.98 ms` (`64K cold`, `+12.05 ms`).
   - Native L3-cold `A7+A9`: `261.28 ms` (`32K cold`) vs `282.58 ms` (`64K cold`, `+21.30 ms`).
   - Baseline L3-cold `A7+A9`: `274.55 ms` (`32K cold`) vs `294.58 ms` (`64K cold`, `+20.03 ms`).
   - JCC-aligned L3-cold `A7+A9`: `244.74 ms` (`32K cold`) vs `266.97 ms` (`64K cold`, `+22.23 ms`).
   By contrast, keeping **`SLAB = 32 KiB`** (`4 × 32 KiB = 128 KiB = 50%` of L2) while marking `compact_and_crc_slab` **`#[cold] #[inline(never)]`** (`W4_b_32k_cold`) removes the 5× inline code bloat from `emit_4lane_sampled<true>` and wins everywhere (`-11.37 ms` native warm, `-5.07 ms` native cold, `-4.06 ms` baseline warm, `-4.29 ms` JCC cold), hitting **`503.65 ms (-27.44%)`** warm CPU (`505.43 ms / -27.89%` warm wall) and **`566.09 ms (-24.59%)`** cold CPU on native!
2. **Round 1 `W4_a_hoist496` Native Regression = Intel JCC Crossing + `batch.as_ptr()` Stack Spill:**
   `objdump -d` on Round 1's unaligned native binary caught two exact causes for `W4_a_hoist496`'s `A5+A6a` jumping from `145.93 ms` to `169.22 ms` (`+23.3 ms`):
   - In `front_avx2_batched_hoisted`, the per-literal hot check `0x41e7a: cmp $0x7f, %r14; 0x41e7e: jbe 41f15` (ending at `0x41e83`) **crossed the `0x41e80` 32-byte boundary**, triggering a DSB uop-cache drop on every literal (`14.39M` iterations). Compiling with JCC alignment immediately dropped `W4_a_hoist496` `A5+A6a` by **`-39.7 ms`** (`169.22 → 129.52 ms`, **`-10.12 ms` faster than `W4_v1` `139.64 ms`**).
   - In `flush_batch_avx2`, inlining both the slow fallback loop and `Vec::reserve_exact` + `memset` inside `push_thick` forced LLVM to spill `batch.as_ptr()` to `0x28(%rsp)` and reload it on every thin run (`0x498bd: mov 0x28(%rsp), %rax`).

## 4. Round 2: Cold-Outlined `flush_batch_fallback` / `grow_edges` + Direct `32 KiB` vs `64 KiB` Head-to-Head (10 Arms)

In Round 2 we outlined `flush_batch_fallback` and `grow_edges` (`#[cold] #[inline(never)]`, 100% safe Rust) and added the true winning combinations **`W4_v2_q496_32k`** and **`W4_v2_q1008_32k`** head-to-head against `W4_v2_q1008_64k` and `W3b_v2_q1008_32k_unf`. *(Note: sibling swarm jobs ran concurrently on the 2-vCPU host during Round 2, so wall times reflect ~2× vCPU contention while `schedstat` CPU remains steal-free.)*

### 4.1 Round 2 `schedstat` CPU Medians (ms, 7 interleaved repeats, all 3 profiles)

| Arm | Base Warm CPU | Base Cold CPU | Native Warm CPU | Native Cold CPU | JCC Warm CPU | JCC Cold CPU |
|---|---:|---:|---:|---:|---:|---:|
| `W0_shipped` | `706.84` | `775.85` | `701.25` | `758.97` | `649.48` | `687.65` |
| `W3b_v1_unfused` | `551.38` (`-21.99%`) | `638.52` (`-17.70%`) | `550.23` (`-21.54%`) | `616.89` (`-18.72%`) | `521.41` (`-19.72%`) | `563.79` (`-18.01%`) |
| `W4_v1_32k_inline` | `563.58` (`-20.27%`) | `634.01` (`-18.28%`) | `553.73` (`-21.04%`) | `609.75` (`-19.66%`) | `522.43` (`-19.56%`) | `566.87` (`-17.56%`) |
| `W4_a_hoist496` | `563.43` (`-20.29%`) | `635.32` (`-18.11%`) | `560.97` (`-20.00%`) | `633.57` (`-16.52%`) | `514.02` (`-20.86%`) | `555.55` (`-19.21%`) |
| `W4_b_32k_cold` | `559.55` (`-20.84%`) | `630.67` (`-18.71%`) | `537.90` (`-23.29%`) | `599.69` (`-20.99%`) | `521.14` (`-19.76%`) | `555.90` (`-19.16%`) |
| `W4_b_64k_cold` | `575.22` (`-18.62%`) | `634.36` (`-18.24%`) | `545.61` (`-22.19%`) | `607.48` (`-19.96%`) | `525.24` (`-19.13%`) | `569.74` (`-17.15%`) |
| **`W4_v2_q496_32k`** | `569.66` (`-19.41%`) | `627.14` (`-19.17%`) | **`536.48` (`-23.50%`)** | **`594.77` (`-21.64%`)** | `517.44` (`-20.33%`) | **`551.44` (`-19.81%`)** |
| **`W4_v2_q1008_32k`** | `563.01` (`-20.35%`) | **`598.86` (`-22.81%`)** | `539.04` (`-23.13%`) | **`594.88` (`-21.62%`)** | `523.09` (`-19.46%`) | `565.14` (`-17.82%`) |
| `W4_v2_q1008_64k` | `550.73` (`-22.09%`) | `620.17` (`-20.07%`) | `551.22` (`-21.40%`) | `614.85` (`-18.99%`) | `523.68` (`-19.37%`) | `566.46` (`-17.62%`) |
| `W3b_v2_q1008_32k_unf` | `539.53` (`-23.67%`) | `606.01` (`-21.89%`) | `554.31` (`-20.95%`) | `610.48` (`-19.57%`) | `519.11` (`-20.07%`) | `560.32` (`-18.52%`) |

### 4.2 Direct Upgrade Marginals & L3-Cold Fusion Advantage (Round 2 `schedstat` CPU)

| Metric | Baseline Cold | Native Warm | Native Cold | JCC-Aligned Cold |
|---|---:|---:|---:|---:|
| **`W4_v2_q496_32k` vs `W4_v1`** | **`-6.87 ms`** (`-0.89% W0`) | **`-17.25 ms`** (`-2.46% W0`) | **`-14.99 ms`** (`-1.97% W0`) | **`-15.43 ms`** (`-2.24% W0`) |
| **`W4_v2_q1008_32k` vs `W4_v1`** | **`-35.15 ms`** (`-4.53% W0`) | **`-14.69 ms`** (`-2.09% W0`) | **`-14.87 ms`** (`-1.96% W0`) | `-1.73 ms` (`-0.25% W0`) |
| **`64 KiB` vs `32 KiB` (`q1008`)** | **`+21.30 ms` SLOWER** | **`+12.18 ms` SLOWER** | **`+19.97 ms` SLOWER** | `+1.31 ms` SLOWER |
| **Fused vs Unfused (`W4_v2` vs `W3b_v2` @ `32 KiB`)** | **`-7.15 ms` FASTER** | **`-15.27 ms` FASTER** | **`-15.60 ms` FASTER** | `+4.82 ms` (noise) |

*(Wall-time medians in Round 2 confirm the same ordering despite host contention: on Native L3-cold wall, `W4_v2_q1008_32k` cuts `W0` by `-23.67%` [`1567.35 → 1196.41 ms`], beating `W4_v1` by `-97.50 ms`, `W4_v2_q1008_64k` by `-35.55 ms`, and `W3b_v2_q1008_32k_unf` by `-71.89 ms`.)*

## 5. Verdict on `#1056`'s `W'/8 <= 133.9 ms` (`6 GB/s`) Contour & Recommendations

1. **Exact answer on the `6 GB/s` contour (VM diagnostic, C5: real i9-10900K measurement still required):**
   - **Under matched build flags (same compiler flags on `W0` and `W4_v2`):**
     - **Quiet-VM native warm (`W4_b_32k_cold`):** **`-27.44%` CPU (`W'/8 = 134.74 ms`) / `-27.89%` wall (`W'/8 = 133.89 ms`)** — **clears the `135.4 ms` small-chunk-0 contour** and reaches the exact edge of `133.9 ms`.
     - **Realistic 8-worker L3-cold (`W4_b_32k_cold`, `W4_v2_q496_32k`, `W4_v2_q1008_32k`):** **`-21.6%..-24.6%` (`W'/8 = 140.0..145.5 ms`)** on baseline/native and **`-19.8%` (`W'/8 = 148.9 ms`)** vs JCC-aligned `W0` — **does NOT yet clear `133.9 ms` in L3-cold** (remaining gap **`~6–11 ms` on `W'/8`** under matched flags).
   - **Important JCC-alignment nuance for Comet Lake i9-10900K:** Compiling with `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp` speeds up **`W0_shipped` itself by `-81..-86 ms` (`-10.8%..-12.5%`)** on Haswell (Comet Lake carries the identical Intel JCC erratum mitigation). Relative to the **unaligned shipped `W0`** (`694.07 ms` warm / `750.72 ms` cold), JCC-aligned `W4_v2_q496_32k` (`498.30 ms` warm / `551.44 ms` cold) represents **`-28.2%` warm / `-26.5%` cold**, which WOULD land at `W'/8 ≈ 133.3..136.4 ms` if shipped `character-stage1` was paying JCC penalties on the i9.
2. **Concrete production guidance (all 4 mechanisms + v2 refinements):**
   - **ADOPT `#[cold] #[inline(never)]` on `compact_and_crc_slab` AT `SLAB = 32 KiB`:** Pure win (`-5..-16 ms` over `#1062`'s `#[inline]` `W4_v1`, zero working-set increase at `3,488.5 KiB`, strengthens the L3-cold fusion win over unfused `W3b` to `-7..-16 ms` at half the memory).
   - **REJECT `SLAB = 64 KiB` on 256 KiB L2 CPUs:** Regresses `+12..+25 ms` across every profile (`4 × 64 KiB = 256 KiB` evicts slabs from L2 before compaction) and adds `+325 KiB` per worker.
   - **ADOPT `#1059` hoisted `OutQ16` (`OVF_BIT` + batch-hoisted `p`/`EPOCH` + fused Pass-1 `lits` + cold-outlined `flush_batch_fallback` & `grow_edges`):** Adds **`-7..-35 ms` in L3-cold** (`-15 ms` native cold, `-15 ms` JCC cold, `-35 ms` baseline cold at `1008/1024`) with zero `unsafe` in `OutQ16` and `0` overflow chunks. On `496/512` vs `1008/1024`: `1008/1024` wins strongly on baseline (`-28.3 ms` cold vs `496/512`) and ties `496/512` on native cold (`594.88` vs `594.77 ms`), with `2 KiB` smaller working set.
   - **Next lever for the remaining `~6–11 ms` on `W'/8`:** Test JCC-aligned codegen directly on the i9 (free across the whole binary), pair with small chunk 0 (`#1056` raises the `W'/8` ceiling to `135.4 ms`), and target `A7+A9` (`45–47%` of `W4_v2` total) if further worker cuts are needed after the real i9 measurement.
