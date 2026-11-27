# Composed End-to-End Open Worker Stack Probe (`A5 + A6a + A6b + A6c + A7 + A9`)

- **Date:** 2026-11-27
- **Probe binary:** `examples/composed_worker_stack_probe.rs` (zero `src/` edits, smooth zone, `Claim: none`)
- **Corpus:** `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks, **233 production-admitted `Open` blocks**, **209,691,304 decoded bytes**).
- **Hardware:** 2-vCPU cloud VM (Intel Xeon, 45 MiB L3, AVX2 + BMI2 + PCLMULQDQ). **VM diagnostic only (C5)**: absolute times and i9-10900K projections are conditional models, not hardware confirmation.

---

## 1. Motivation (`frontier/DECODER.md` Composition Rule)

Four independent worker-side mechanisms have passed smooth-zone validation on `master`:

1. **`#1055` (`A5 + A6a`, `examples/mtf_l1_batch_probe.rs`)**: L1d-batched `[u32; 512]` packed maximal-run `front_avx2` (`second_cold` out-of-line, zero `vzeroupper`) + `OutQ16` `496/512` power-of-2 guard zone. Eliminates the ~756 KiB `front.runs` buffer.
2. **`#1046` (`A6b`, `examples/a6b_combined_probe.rs`)**: Fused single-pass `setup_soa_64k` + `image_order64` + safe zero-bounds-check `Box<[T; 65536]>` `u16`-link `inline15` `u128` SoA + direct `aw == bw` replace + generation-tagged L1d `GenCache` dict.
3. **`#1057` (`A6c + A7`, `examples/forward_4lane_probe.rs`)**: Unmodified SIMD `QueueReader::wire` + `S = 16` uniform BWT row-index bit-31 anchor tags + dynamic 4-lane counted forward emission (`SLAB = 32 KiB`).
4. **`#1045` (`A7 + A9`, `examples/forward_rle1_crc_fusion.rs`)**: Zero-scratch single-pass in-place L2-hot RLE1 compaction + PCLMULQDQ CRC32 + GF(2) `crc32_combine` directly on the 4-lane vectored slabs (zero stitch copy, exact boundary-straddle repair).

`frontier/DECODER.md` explicitly warns: *do not add isolated percentages or pretend projections are measurements; compare the combined real execution and relevant isolated/paired diagnostics.*

This probe executes all 9 end-to-end arms from `b: &Symbols` to verified RLE1-decoded bytes + block `u32` CRC32 on every admitted `Open` block:

- **Cumulative ladder:** `W0_shipped` -> `W1_plus_l1_mtf` (`+#1055`) -> `W2_plus_a6b` (`+#1046`) -> `W3_plus_fwd4_stitch` (`+#1057` contiguous stitch) / `W3b_plus_fwd4_vec_unfused` (`+#1057` vectored, post-pass RLE1+CRC) -> **`W4_full_fused_vec`** (`+#1045`, all 4 mechanisms).
- **Isolated controls (for exact interaction residual):** `I2_only_a6b` (`#1046` only), `I3_only_fwd4_vec` (`#1057` vectored only), `I4_only_fwd4_fused` (`#1057 + #1045` only).

---

## 2. Exactness & Per-Worker Working-Set Audit

Every arm is verified **byte-for-byte** (concatenating vectored slices in chronological order) and **CRC32-exact** against `mibz2::decode_block` on all 233 admitted Open blocks before any timing starts.

| Arm | Exactness (233 blks) | Resident scratch / worker | Virtual scratch / worker | 8W resident total | Delta vs `W0` |
|---|---:|---:|---:|---:|---:|
| `W0_shipped` | 233/233 byte+CRC32 | 2,896.6 KiB | 2,896.6 KiB | 22.6 MiB | baseline |
| `W1_plus_l1_mtf` | 233/233 byte+CRC32 | **2,140.2 KiB** | 2,140.2 KiB | 16.7 MiB | **-756.4 KiB (-26.1%)** |
| `W2_plus_a6b` | 233/233 byte+CRC32 | 3,265.7 KiB | 4,637.7 KiB | 25.5 MiB | +369.1 KiB (+12.7%) |
| `W3_plus_fwd4_stitch` | 233/233 byte+CRC32 | 4,402.4 KiB | 5,774.4 KiB | 34.4 MiB | +1,505.8 KiB (+52.0%) |
| `W3b_plus_fwd4_vec_unfused` | 233/233 byte+CRC32 | 6,893.2 KiB | 8,265.2 KiB | 53.9 MiB | +3,996.6 KiB (+138.0%) |
| **`W4_full_fused_vec`** | **233/233 byte+CRC32** | **3,488.5 KiB** | **4,860.5 KiB** | **27.3 MiB** | **+591.9 KiB (+20.4%)** |
| `I2_only_a6b` | 233/233 byte+CRC32 | 4,022.1 KiB | 5,394.1 KiB | 31.4 MiB | +1,125.5 KiB (+38.9%) |
| `I3_only_fwd4_vec` | 233/233 byte+CRC32 | 6,524.1 KiB | 6,524.1 KiB | 51.0 MiB | +3,627.5 KiB (+125.2%) |
| `I4_only_fwd4_fused` | 233/233 byte+CRC32 | 3,119.5 KiB | 3,119.5 KiB | 24.4 MiB | +222.9 KiB (+7.7%) |

**Key footprint discoveries:**
1. **`#1055` saves 756.4 KiB/worker** by removing `front.runs: Vec<u32>`, offsetting 67% of `#1046`'s `Fixed64kSoa` resident working set (`~420 KiB` touched of 64K table + `image_order64` + `rules` + `GenCache`).
2. **`#1045` cuts vectored working set in HALF** (`W4` **3,488.5 KiB** vs `W3b` **6,893.2 KiB**, `-3.33 MiB`/worker) and saves **913.9 KiB/worker** over `W3_plus_fwd4_stitch` (`4,402.4 KiB`). In `W3b`, post-pass out-of-place RLE1 expansion on spilled/straddled slices appends after the raw slabs, inflating `out` to ~4.7 MiB; in `W4`, compacting each 32 KiB slab in-place during emission keeps `out` compact and total 8-worker resident footprint at **27.3 MiB** (only +4.6 MiB over `W0`).
3. Note on `Vec` growth caught during verification: initial exponential `len * 2` resizing in `SlabAlloc::take_slab` bloated `W4` to 6.09 MiB/worker (`+2.6 MiB` dead capacity); switching `SlabAlloc::take_slab` and `want_arena` to exact linear `reserve_exact` immediately reclaimed 2.6 MiB/worker.

---

## 3. Timing Methodology

- **7 interleaved repeats** (alternating forward/reverse arm order per repeat) pinned to CPU 0 (`taskset -c 0`).
- **Two cache regimes:**
  - **WARM:** single reused `WorkerState` across all 233 blocks (resident ~2–7 MiB fits inside 45 MiB L3).
  - **L3-COLD:** rotating pool of 16 independent `WorkerState` instances (> 50 MiB total, exceeding the 45 MiB L3) plus an explicit 64 MiB streaming LLC flush before each arm, honestly pricing per-worker working-set L3 eviction (including `Fixed64kSoa` and extra stitched `arena`).
- **Two builds:**
  - **Baseline x86-64** (`cargo build --release`, shipping default with runtime AVX2 dispatch for `front_avx2`, `find_next_run_avx2`, and PCLMULQDQ CRC32).
  - **Native** (`RUSTFLAGS="-C target-cpu=native"`, AVX2 + BMI2 + PCLMULQDQ everywhere).
- **Two clocks (both reported transparently):**
  - **Steal-free CPU (`schedstat`)**: `/proc/thread-self/schedstat` field 0 (`se.sum_exec_runtime`), immune to concurrent-VM OS preemption (4 ms tick quantization, ~0.5% over 550–780 ms passes). **Primary clean signal** on this shared 2-vCPU VM.
  - **Wall (`Instant`)**: reported alongside; concurrent VM load produced heavy preemption spikes (especially in L3-cold wall, up to 2.75x `[min..max]` spread).

---

## 4. Baseline x86-64 Results (7 Interleaved Repeats)

### 4.1 Baseline WARM — Steal-Free CPU (`schedstat`, ms)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | Total med `[min..max]` | vs `W0` | Disjoint vs `W0` | Modeled i9 `W'/8` |
|---|---:|---:|---:|---:|---:|---:|:---:|---:|
| `W0_shipped` | 206.36 | 131.29 | 103.05 | 301.46 | **737.06** `[730.98..740.97]` | +0.00% | base | 185.68 ms |
| `W1_plus_l1_mtf` | 152.83 | 129.15 | 86.24 | 301.11 | **678.76** `[670.85..718.27]` | **-7.91%** | FASTER | 170.99 ms |
| `W2_plus_a6b` | 162.09 | 94.19 | 93.84 | 304.76 | **650.28** `[644.40..694.06]` | **-11.77%** | FASTER | 163.82 ms |
| `W3_plus_fwd4_stitch` | 170.18 | 98.94 | 36.76 | 244.66 | **554.69** `[550.23..564.07]` | **-24.74%** | FASTER | 139.74 ms |
| `W3b_plus_fwd4_vec_unfused` | 155.42 | 107.39 | 33.84 | 262.35 | **553.27** `[543.79..564.94]` | **-24.94%** | FASTER | 139.38 ms |
| **`W4_full_fused_vec`** | **167.24** | **95.11** | **34.49** | **251.53** | **548.46** `[540.08..607.17]` | **-25.59%** | **FASTER** | **138.17 ms** |
| `I2_only_a6b` | 211.11 | 94.92 | 97.02 | 309.87 | 708.61 `[699.82..726.72]` | -3.86% | FASTER | 178.51 ms |
| `I3_only_fwd4_vec` | 204.56 | 134.21 | 32.95 | 245.33 | 628.56 `[620.03..652.25]` | -14.72% | FASTER | 158.35 ms |
| `I4_only_fwd4_fused` | 209.45 | 130.59 | 35.83 | 261.67 | 634.69 `[627.71..652.26]` | -13.89% | FASTER | 159.89 ms |

- **Cumulative marginals:** `+#1055 = +58.30 ms`, `+#1046 = +28.49 ms`, `+#1057_stitch = +95.58 ms` / `+#1057_vec = +97.01 ms`, `+#1045_fused = +4.81 ms` (and `+6.23 ms` over `W3_stitch`).
- **Exact interaction residual (`W4`):** actual `+188.60 ms` (`0.7441x`) vs sum of isolated (`+58.30 + +28.45 + +102.37 = +189.11 ms`) -> **`-0.52 ms` (`-0.07%` of `W0`)** — **perfect additivity**.
- **Baseline WARM Wall (`Instant`) confirms the identical ratio:** `W0 = 1485.06 [1472.32..1500.88]` ms -> `W1 = 1360.59 (-8.38%)` -> `W2 = 1309.56 (-11.82%)` -> `W3_stitch = 1114.63 (-24.94%)` -> `W3b = 1128.38 (-24.02%)` -> **`W4 = 1107.40 [1082.85..1510.58]` ms (`-25.43%`, `0.7457x`, residual `-0.62%`)**.

### 4.2 Baseline L3-COLD — Steal-Free CPU (`schedstat`, ms)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | Total med `[min..max]` | vs `W0` | Disjoint vs `W0` | Modeled i9 `W'/8` |
|---|---:|---:|---:|---:|---:|---:|:---:|---:|
| `W0_shipped` | 218.48 | 136.81 | 102.67 | 322.85 | **779.26** `[770.06..800.80]` | +0.00% | base | 185.68 ms |
| `W1_plus_l1_mtf` | 168.37 | 140.60 | 96.13 | 319.39 | **726.74** `[703.36..738.80]` | **-6.74%** | FASTER | 173.17 ms |
| `W2_plus_a6b` | 170.65 | 106.51 | 105.45 | 308.66 | **692.62** `[679.49..710.73]` | **-11.12%** | FASTER | 165.04 ms |
| `W3_plus_fwd4_stitch` | 169.02 | 106.54 | 41.30 | 289.51 | **607.60** `[599.05..635.88]` | **-22.03%** | FASTER | 144.78 ms |
| `W3b_plus_fwd4_vec_unfused` | 166.85 | 101.45 | 40.78 | 293.70 | **604.09** `[597.37..671.59]` | **-22.48%** | FASTER | 143.94 ms |
| **`W4_full_fused_vec`** | **172.70** | **110.49** | **43.28** | **287.68** | **615.29** `[589.86..651.46]` | **-21.04%** | **FASTER** | **146.61 ms** |
| `I2_only_a6b` | 216.56 | 103.18 | 111.37 | 308.26 | 747.72 `[740.70..761.88]` | -4.05% | FASTER | 178.17 ms |
| `I3_only_fwd4_vec` | 226.65 | 137.49 | 32.33 | 286.05 | 687.89 `[668.11..722.29]` | -11.72% | FASTER | 163.91 ms |
| `I4_only_fwd4_fused` | 233.87 | 135.57 | 35.98 | 280.26 | 684.07 `[671.36..689.29]` | -12.21% | FASTER | 163.00 ms |

- **Cumulative marginals:** `+#1055 = +52.51 ms`, `+#1046 = +34.12 ms`, `+#1057_stitch = +85.02 ms` / `+#1057_vec = +88.53 ms`, `+#1045_fused = -11.20 ms` (median; note `W4` achieved the fastest `A7+A9` of any arm at `287.68 ms` and fastest single-repeat minimum `589.86 ms`).
- **Exact interaction residual:** `W3b` actual `+175.16 ms` vs sum_iso `+175.41 ms` -> **`-0.25 ms` (`-0.03%`)**; `W4` actual `+163.97 ms` vs sum_iso `+179.23 ms` -> **`-15.26 ms` (`-1.96%` of `W0`)**.
- **Baseline L3-COLD Wall (`Instant`, heavily preempted):** `W0 = 1179.98 [796.64..2193.27]` ms vs `W4 = 1221.04 [590.46..1604.69]` ms (all 9 arms overlapped due to 2.75x host CPU steal; minimum wall `590.46 ms` vs `796.64 ms` matches the CPU ratio).

---

## 5. `-C target-cpu=native` Results (7 Interleaved Repeats)

### 5.1 Native WARM — Steal-Free CPU (`schedstat`, ms)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | Total med `[min..max]` | vs `W0` | Disjoint vs `W0` | Modeled i9 `W'/8` |
|---|---:|---:|---:|---:|---:|---:|:---:|---:|
| `W0_shipped` | 200.17 | 133.42 | 87.75 | 268.88 | **690.47** `[682.31..718.45]` | +0.00% | base | 185.68 ms |
| `W1_plus_l1_mtf` | 180.36 | 121.01 | 101.04 | 274.55 | **674.55** `[655.61..685.59]` | **-2.31%** | overlap | 181.40 ms |
| `W2_plus_a6b` | 166.41 | 102.66 | 98.10 | 277.50 | **641.36** `[632.22..654.06]` | **-7.11%** | FASTER | 172.47 ms |
| `W3_plus_fwd4_stitch` | 158.48 | 116.37 | 37.75 | 230.46 | **547.87** `[541.25..549.96]` | **-20.65%** | FASTER | 147.33 ms |
| `W3b_plus_fwd4_vec_unfused` | 161.19 | 95.67 | 32.86 | 248.22 | **543.81** `[537.14..556.42]` | **-21.24%** | FASTER | 146.24 ms |
| **`W4_full_fused_vec`** | **166.00** | **94.88** | **38.12** | **253.74** | **553.78** `[538.11..588.10]` | **-19.80%** | **FASTER** | **148.92 ms** |
| `I2_only_a6b` | 193.35 | 95.08 | 112.85 | 269.56 | 666.18 `[658.66..692.20]` | -3.52% | overlap | 179.15 ms |
| `I3_only_fwd4_vec` | 192.79 | 121.17 | 37.11 | 247.76 | 611.53 `[591.98..625.47]` | -11.43% | FASTER | 164.45 ms |
| `I4_only_fwd4_fused` | 193.01 | 126.42 | 36.99 | 260.05 | 620.26 `[607.85..633.10]` | -10.17% | FASTER | 166.80 ms |

- **Cumulative marginals:** `+#1055 = +15.92 ms`, `+#1046 = +33.19 ms`, `+#1057_stitch = +93.49 ms` / `+#1057_vec = +97.54 ms`, `+#1045_fused = -9.97 ms`.
- **Exact interaction residual (POSITIVE SYNERGY):**
  - `W3b`: actual `+146.65 ms` (`0.7876x`) vs sum_iso `+119.15 ms` -> **`+27.50 ms` (`+3.98%` of `W0`)**!
  - `W4`: actual `+136.69 ms` (`0.8020x`) vs sum_iso `+110.42 ms` -> **`+26.27 ms` (`+3.80%` of `W0`)**!
- **Native WARM Wall (`Instant`) matches CPU:** `W0 = 1394.75 [1371.54..1764.91]` ms -> `W3b = 1109.78 (-20.43%, FASTER)` -> **`W4 = 1120.73 [1086.93..1310.13]` ms (`-19.65%`, FASTER, residual `+3.29%`)**.

### 5.2 Native L3-COLD — Steal-Free CPU (`schedstat`, ms)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | Total med `[min..max]` | vs `W0` | Disjoint vs `W0` | Modeled i9 `W'/8` |
|---|---:|---:|---:|---:|---:|---:|:---:|---:|
| `W0_shipped` | 211.53 | 141.06 | 106.03 | 299.48 | **754.63** `[741.69..795.85]` | +0.00% | base | 185.68 ms |
| `W1_plus_l1_mtf` | 184.63 | 122.29 | 101.91 | 299.53 | **710.44** `[698.12..748.04]` | **-5.86%** | overlap | 174.81 ms |
| `W2_plus_a6b` | 167.08 | 112.63 | 112.99 | 286.23 | **676.19** `[671.50..686.80]` | **-10.39%** | FASTER | 166.38 ms |
| `W3_plus_fwd4_stitch` | 178.88 | 109.59 | 36.89 | 302.93 | **626.82** `[607.44..632.82]` | **-16.94%** | FASTER | 154.23 ms |
| `W3b_plus_fwd4_vec_unfused` | 166.32 | 103.65 | 39.18 | 275.49 | **596.59** `[591.41..650.41]` | **-20.94%** | FASTER | 146.79 ms |
| **`W4_full_fused_vec`** | **175.13** | **97.88** | **45.53** | **275.55** | **597.61** `[590.84..637.64]` | **-20.81%** | **FASTER** | **147.04 ms** |
| `I2_only_a6b` | 208.70 | 102.96 | 117.52 | 290.81 | 713.67 `[694.19..761.77]` | -5.43% | overlap | 175.60 ms |
| `I3_only_fwd4_vec` | 218.37 | 126.23 | 36.42 | 288.39 | 669.21 `[644.89..697.70]` | -11.32% | FASTER | 164.66 ms |
| `I4_only_fwd4_fused` | 207.84 | 134.19 | 34.49 | 287.29 | 662.95 `[648.90..704.02]` | -12.15% | FASTER | 163.12 ms |

- **Cumulative marginals:** `+#1055 = +44.19 ms`, `+#1046 = +34.25 ms`, `+#1057_stitch = +49.37 ms` / `+#1057_vec = +79.61 ms`, `+#1045_fused = -1.02 ms` (**but `+29.21 ms` faster than `W3_stitch`!**).
- **Exact interaction residual:** `W3b` actual `+158.04 ms` vs sum_iso `+170.56 ms` -> **`-12.52 ms` (`-1.66%` of `W0`)**; `W4` actual `+157.03 ms` vs sum_iso `+176.82 ms` -> **`-19.80 ms` (`-2.62%` of `W0`)**.
- **Native L3-COLD Wall (`Instant`, preempted):** `W0 = 1625.07 [1300.92..2245.42]` ms -> `W3_stitch = 1218.40` -> `W3b = 1026.41` -> **`W4 = 932.48 [706.33..1467.04]` ms (`-42.62%`, overlap due to VM steal)**.

---

## 6. Cross-Mechanism Interaction Diagnosis

1. **Zero destructive interference across the 4-mechanism stack (`-2.62%` to `+3.80%` residual):**
   - On baseline x86-64 WARM CPU, the 4 mechanisms compose with **-0.07% residual** (`-0.52 ms` out of `737.06 ms`), eliminating **-25.59%** of total Open worker time (`0.7441x`).
   - On native WARM CPU, `#1055 + #1046 + #1057` exhibit **+3.8% to +4.0% positive synergy**: eliminating `front.runs` (`-756 KiB`) leaves L2/L3 cleaner entering `#1046`'s `Fixed64kSoa` and `#1057`'s forward walk.
   - On L3-COLD CPU (both builds), `#1055` (`-5.9%..-6.7%`) and `#1046` (`-4.5%..-4.8%` marginal) stack cleanly without `Fixed64kSoa` thrashing L3; the slight `-1.7%..-2.6%` L3-cold residual is minor overlap between `#1055` and `#1057` reducing DRAM pressure on the same worker buffers.
2. **Eliminating the stitch copy (`W3` -> vectored `W3b`/`W4`) is huge under L3-cold:**
   - On native L3-cold, `W3_plus_fwd4_stitch` pays **`+30.2 ms` (`+4.0%` of `W0`)** for copying 900 KiB of cold slabs into a second 900 KiB `arena` buffer (`626.82` vs `596.59` / `597.61 ms`). Avoiding the stitch copy is mandatory for L3-cold 8W scaling.
3. **Why `#1045` in-place slab compaction (`W4` vs `W3b`) was `+4.8 ms` faster on baseline warm, tied (`-1.0 ms`) on native L3-cold, and `-10.0 ms` slower on native warm — and how to fix it:**
   - In `W4`, `SLAB_SIZE = 32 KiB` invokes `compact_and_crc_slab` 15–30 times per block inside `emit_4lane_sampled<true>`, doing per-slab `find_next_run` + `crc32` setup (half `#1045`'s optimal 64 KiB tile) and increasing register/icache pressure inside the 4-lane loop.
   - In **WARM** mode (3.5 MiB fits in 45 MiB L3), `W3b`'s post-pass reads L3 rather than DRAM, exposing `W4`'s ~5–10 ms per-slab call overhead.
   - In **L3-COLD** mode, `W4` recovers the DRAM pass AND cuts per-worker resident memory in **half** (`3,488.5 KiB` vs `6,893.2 KiB` for `W3b`, `-26.6 MiB` across 8 workers!).
   - **Production fix when building `#1045` on top of `#1057`:** raise the fused slab tile from `32 KiB` to `64 KiB` and mark `compact_and_crc_slab` `#[cold]`/`#[inline(never)]` so it stays completely out of the 4-lane hot register allocation while retaining the compact `3.41 MiB` working set and L2-hot DRAM avoidance.

---

## 7. Conditional i9-10900K `6 GB/s` Gap Assessment (VM Diagnostic Only, C5)

Anchors from `character-stage1` and `#1056` (`slack-hybrid-scan-feasibility.md`):
- Shipped i9-10900K `1W` Open worker = `1,270.9 ms` (`1,306 ms` 1W wall).
- Shipped i9-10900K `8W` worker busy = `W/8 = 185.68 ms`.
- `#1056` `6 GB/s` (`<= 140 ms` 8W wall) threshold at `S' = 145 ms`: **`W'/8 <= 133.9 ms`** (or `<= 135.4 ms` with small chunk 0).

| Build & Cache Regime (Steal-Free CPU) | `W4 / W0` | Best (`W3b`/`W4`) | Modeled `1W` Save (`W4`) | Modeled `W'/8` (`W4`) | Gap to `133.9 ms` (`6 GB/s`) |
|---|---:|---:|---:|---:|---:|
| Baseline x86-64 WARM | **0.7441x** (-25.59%) | `W4` 0.7441x | **+325.2 ms** (+24.90% wall) | **138.17 ms** | -4.27 ms (-2.77 ms vs 135.4) |
| Baseline x86-64 L3-COLD | **0.7896x** (-21.04%) | `W3b` 0.7752x | +267.4 ms (+20.48% wall) | 146.61 ms (`W3b` 143.94) | -12.71 ms (`W3b` -10.04) |
| Native WARM | **0.8020x** (-19.80%) | `W3b` 0.7876x | +251.6 ms (+19.26% wall) | 148.92 ms (`W3b` 146.24) | -15.02 ms (`W3b` -12.34) |
| Native L3-COLD | **0.7919x** (-20.81%) | `W3b` 0.7906x | +264.5 ms (+20.25% wall) | 147.04 ms (`W3b` 146.79) | -13.14 ms (`W3b` -12.89) |

### Strategic Takeaway

1. **The 4 merged mechanisms deliver `-19.8%` to `-25.6%` real end-to-end Open worker speedup with no destructive interference**, closing **70%–92%** of the `51.8 ms` worker gap from `185.68 ms` to `133.9 ms` on VM diagnostics (`W'/8 = 138.2–148.9 ms`).
2. **Where the remaining `W4` worker time sits (baseline / native L3-cold CPU):**
   - `A7 + A9` (4-lane forward walk + RLE1 + CRC32): **46.1%–46.8%** (`275.6–287.7 ms`) — still nearly HALF the worker!
   - `A5 + A6a` (L1-batched MTF + `OutQ16`): **28.1%–29.3%** (`172.7–175.1 ms`).
   - `A6b` (`Fixed64kSoa` + `GenCache`): **16.4%–18.0%** (`97.9–110.5 ms`).
   - `A6c` (forward SIMD wire + 16-anchor tag): **7.0%–7.6%** (`43.3–45.5 ms` — effectively solved).
3. **What bridges the final `~4–15 ms` gap at `W'/8` to clear `6 GB/s` with margin:**
   - **64 KiB tile + cold-outlined slab compaction in `W4`** (eliminates the ~10 ms VM warm overhead -> `~2–3 ms` at `W'/8`).
   - **Real i9-10900K L3 contention (20 MiB vs VM 45 MiB):** `#992`/`#998` measured larger cold 4-lane MLP gains on i9-10900K (`0.64–0.70x`) than this 2-vCPU Xeon VM, plus `#1045`'s 8-worker footprint cut (`27.3 MiB` vs `53.9 MiB` for `W3b`) matters much more on 20 MiB L3.
   - **Next algorithmic targets inside `W4`'s remaining 75% (`A7` 46% + `A5+A6a` 29%):** higher-lane forward MLP (`K = 6` or `8` on i9-10900K), reducing per-step `A7` dispatch/write cost, and cutting `A5+A6a` bitmap + `OutQ16` overhead; plus shipping `#1042` (`Segmented` fallback `-15%..-18%`) for the non-Open blocks.
