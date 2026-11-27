# RLE1 + CRC32 (`A9`) census and single-pass prototype measurements (`single-pass-rle1-crc`)

*Smooth-zone probe (`frontier/claims/single-pass-rle1-crc.toml`, 26 November 2026). Reproducible zero-`unsafe` harness in `examples/rle1_crc_census.rs`; zero `src/` edits; frozen stop test evaluated strictly under C2.*

**Verdict: STOP.** Neither branch of the frozen stop test is cleared. On `wikidata x4` (838.80 MB, 1,724 blocks), the L1d-hot compute floor of PCLMULQDQ `crc32` (`61.8 ms`) plus AVX2 4-equal-byte scanning (`42.0 ms`) already consumes **72.3%** (`103.7 ms`) of shipped `unrle_in_place` (`143.5 ms` per-block L3-warm) before counting the unavoidable 726 MiB of in-place compaction writes (`89.4%` of `wikidata` runs shrink with `extra == 0`). The best zero-extra-buffer single-pass prototype (`sp_inplace_q_tiled16k`) cuts A9 by **−3.52%** (`138.4` vs `143.5 ms`, overlapping) L3-warm and **−6.12%** (`183.0` vs `194.9 ms`, non-overlapping) cold-stream under `-C target-cpu=native` (**−2.67% / −1.42%** baseline x86-64) — far short of the **≥ 35%** 1W gate. On the 8–10W LLC/DRAM branch, single-pass in-place cuts logical write traffic by **−20.8%** (`916.4 → 726.0 MiB` on `wikidata x4`, eliminating `190.4 MiB` of shift-up writes on the 59 `shift > 0` blocks), which bounds 8–10W worker CPU savings to **0.4–1.7%** (realistically **~0.6–1.1%** after accounting for non-LLC SMT overhead and real `none`/`S` A9 = `102.8–131.3 ms` from `third-pass.md` §1) — below the **≥ 2.0%** worker CPU gate. Out-of-place single-pass designs regress **+13.6%..+56.1%** on `wikidata` and double per-worker RLE1 L3 footprint (`0.86 → 1.72 MiB`).

---

## 1. Environment, corpora, and exactness

- **VM hardware:** 2-vCPU Intel Xeon (Haswell) @ 2.30 GHz (`avx2`, `bmi1`, `bmi2`, `pclmulqdq`, `sse4_1`, `ssse3`), 32 KiB L1d, 256 KiB L2, 45 MiB shared L3, no vPMU. Both baseline x86-64 (`cargo run --release`) and `-C target-cpu=native` (`RUSTFLAGS="-C target-cpu=native"`) are reported.
- **SIMD fidelity in `#![forbid(unsafe_code)]`:**
  - `mibz2::crc::crc32` and `mibz2::crc::update_with` are public and execute the exact shipped runtime-dispatched **PCLMULQDQ** kernel in both baseline and native builds (`Kernels::detect() = Kernels { avx2: true, clmul: true }`).
  - `mibz2::rle::unrle` is public and executes the exact shipped runtime-dispatched **AVX2** out-of-place kernel (`unrle_avx2`) in both builds.
  - Because `src/rle.rs::find_runs_avx2` is `pub(crate)`, `examples/rle1_crc_census.rs` implements a bounds-check-free `chunks_exact(32)` safe scanner (`find_next_run_autovec32`). Disassembly (`objdump -d`) confirms `-C target-cpu=native` lowers its inner loop to the exact AVX2 instruction sequence of `src/rle.rs::find_runs_avx2` (`vmovdqu` + 3× `vpcmpeqb` + 2× `vpand` + `vpmovmskb` + `test`/`je`, zero bounds checks); baseline x86-64 lowers it to 16-byte SSE2.
- **Corpora:**
  - `corpora/wikidata.bz2` (431 blocks, 209,700,196 B decoded, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; timed and traffic-accounted at `wikidata ×4` = 1,724 blocks, 838.80 MB = 799.94 MiB).
  - All six `ab gen` corpora generated at scale 4 (`cargo run --release --example ab -- gen /tmp/ab-corpora 4`): `text.bz2` (14 blocks, 13.86 MB), `random.bz2` (9 blocks, 8.00 MB), `onebyte.bz2` (1 block, 20.00 MB), `pattern.bz2` (22 blocks, 19.60 MB), `runs.bz2` (1 block, 10.00 MB), `records.bz2` (11 blocks, 9.60 MB).
- **Exactness:** Every prototype arm (15 arms total, including GF(2) `crc32_combine`, backward in-place, fused out-of-place, and single-pass in-place across 0/4K/16K/32K/64K tiles and fresh/pooled capacities) passed synthetic edge-case tests (`src/rle.rs` `miri_edge_runs`, `RunUnfinished`, runs at offsets 0..64 and lengths 4..=40, tile-boundary straddles `±4 B`, mixed `min_grown < 0 && shift > 0`, massive `shift > n` spills, 120 random blocks) and verified **100% byte-identical output and CRC32** against `mibz2::rle::unrle` + `mibz2::crc::crc32` on **every block of all 7 corpora**.
- **Timing protocol:** 1 warmup + **7 interleaved repeats** on `wikidata ×4` (alternating arm order each repeat; 5 repeats on `ab gen` corpora), reporting median and `[min..max]` in two cache regimes:
  1. **Per-block L3-warm:** restores each block into a single hot recycled `Vec<u8>` immediately outside the per-block timer — matching 1W production right after `invert_runs` finishes writing the ~879 KiB block (hot TLB + hot L3, spilled 256 KiB L2).
  2. **Cold-stream:** sweeps all 431 distinct block buffers (200 MiB >> 45 MiB L3) unbatched — matching LLC-evicted DRAM-streaming pressure.

---

## 2. Part 1 — RLE1 block census across `wikidata` and `ab gen` corpora

### 2.1 Block classification, `min_grown`, and `reserve_exact` reallocation rates

| corpus | blocks (full / runts) | post-RLE1 (MiB) | `len/n` | `runs == 0` blks (% B) | `runs > 0, shift == 0` blks (% B) | `shift > 0` blks (% B) | `shift > 0 & min_grown >= 0` blks (% B) | `shift > n` stage-swap | `min_grown >= 0` all (% B) | realloc: fresh `cap=n` / `prod_pool` / recycled |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **`wikidata.bz2`** | **431** (233 / 198) | **199.99** | **0.999970×** | **196** (45.5% / **0.0%**) | **176** (40.8% / **74.7%**) | **59** (13.7% / **25.3%**) | **12** (2.8% / **5.1%**) | 0 (0.0%) | 208 (48.3% / 5.2%) | 59 (13.7%) / **0 (0.0%)** / **0 (0.0%)** |
| `text.bz2` | 14 (14 / 0) | 13.22 | 1.142184× | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 14 (100% / 100%) | 6 (42.9% / 40.7%) | 0 (0.0%) | 6 (42.9% / 40.7%) | 14 (100%) / **14 (100%)** / 13 (92.9%) |
| `random.bz2` | 9 (9 / 0) | 7.63 | 1.000000× | 6 (66.7% / 66.3%) | 3 (33.3% / 33.7%) | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 0 (0.0%) | 6 (66.7% / 66.3%) | 0 / 0 / 0 |
| `onebyte.bz2` | 1 (1 / 0) | 19.07 | 50.999592× | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 1 (100% / 100%) | 1 (100% / 100%) | 1 (100%) | 1 (100% / 100%) | 0 / 0 / 0 *(stage-swap)* |
| `pattern.bz2` | 22 (22 / 0) | 18.69 | 1.000000× | 22 (100% / 100%) | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 0 (0.0%) | 22 (100% / 100%) | 0 / 0 / 0 |
| `runs.bz2` | 1 (1 / 0) | 9.54 | 39.304156× | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 1 (100% / 100%) | 1 (100% / 100%) | 1 (100%) | 1 (100% / 100%) | 0 / 0 / 0 *(stage-swap)* |
| `records.bz2` | 11 (11 / 0) | 9.16 | 1.000000× | 9 (81.8% / 81.3%) | 2 (18.2% / 18.7%) | 0 (0.0% / 0.0%) | 0 (0.0% / 0.0%) | 0 (0.0%) | 9 (81.8% / 81.3%) | 0 / 0 / 0 |

*(Note: `wikidata ×4` multiplies block counts and bytes by 4 with identical fractions. `prod_pool` capacity is `n + n/16 + 64` for `n >= 64 KiB` from `src/parallel.rs:1210`.)*

### 2.2 Run, `shift`, `grown`, `min_grown`, and prefix distributions on `wikidata.bz2`

| metric (`wikidata.bz2`, 431 blocks) | min | p50 | p90 | p95 | p99 | max | notes |
|---|---:|---:|---:|---:|---:|---:|---|
| `run_count` per block | 0 | 7 | 74 | 116 | 241 | 1,234 | 12,029 runs total (mean 27.9/blk; 1 run per 17.4 KiB) |
| run `extra` breakdown | — | — | — | — | — | — | **`extra == 0` (shrink): 10,752 (89.4%)**, `extra == 1`: 463 (3.8%), `extra >= 2`: 814 (6.8%) |
| `shift` (B) | 0 | 0 | 13 | 48 | 133 | 193 | matches `#966` (`59/431` blocks, max `193 B`) |
| `total_grown = len - n` (B) | −1,183 | 0 | +1 | +41 | +124 | +191 | net corpus shrinks by −6,361 B (`0.999970×`) |
| `min_grown` (B) | −1,183 | −3 | 0 | — | — | 0 | p10 = −57 B; **47/59 (80%) of `shift > 0` blocks have `min_grown < 0`** |
| `len / n` | 0.97917 | 1.00000 | 1.00000 | — | 1.00014 | 1.00021 | max `(n + shift) / n = 1.00021` << `1.0625` pool headroom |
| `first_run / n` (`runs > 0`) | — | 0.051 | 0.224 | — | — | — | p10 = 0.005; `first_mut_run / n` p50 = **0.055** (**90.8%** of bytes follow first mutating run) |

Key census takeaways:
1. **Almost zero bytes take the `runs == 0` fast path on `wikidata`:** all 196 `runs == 0` blocks are tiny runts (`0.0%` of bytes). Every full block has runs.
2. **Shrinking runs dominate `wikidata` (`89.4%` have `extra == 0`)**, and the first shrinking run hits early (`first_mut_run / n` median `5.5%`), so **90.8% of all bytes** (`181.5 / 200.0 MiB` per pass, `726.0 MiB` on `×4`) MUST be moved in-place regardless of algorithm!
3. **Only 25.3% of `wikidata` bytes (`59/431` blocks) take the 4-pass `shift > 0` route**, and **80% of those (`47/59`) have `min_grown < 0`** (mixed shrink+grow). Backward in-place expansion without shift-up (`min_grown >= 0`) applies to only **12/431 blocks (5.1% of bytes)**.
4. **Zero `reserve_exact` reallocations on `wikidata`** under `src/parallel.rs:1210`'s `n + n/16 + 64` capacity (`0/431`). By contrast, `text.bz2` (source code with `len/n = 1.148×`, 22,148 runs/blk) overflows `n + n/16 + 64` on **14/14 blocks (100%)**.

---

## 3. Part 1B — Per-pass breakdown of shipped A9 (`wikidata ×4`, 838.80 MB, 7 repeats)

### 3.1 L1d-hot (16 KiB resident, run-free) kernel compute ceilings

| kernel (838.80 MB, 7 repeats) | Baseline x86-64 med `[min..max]` (GB/s) | `-C target-cpu=native` med `[min..max]` (GB/s) |
|---|---:|---:|
| **`L1d-hot crc32` (shipped PCLMULQDQ)** | **58.10 ms** `[57.00..60.37]` (14.44 GB/s) | **61.76 ms** `[59.79..63.28]` (13.58 GB/s) |
| **`L1d-hot scan_autovec32` (SSE2 / AVX2)** | **84.61 ms** `[83.92..87.39]` (9.91 GB/s) | **41.98 ms** `[40.10..49.03]` (19.98 GB/s) |
| **Sum of L1d-hot `crc32 + scan` alone (0 writes)** | **142.71 ms** | **103.74 ms (72.3% of shipped L3-warm!)** |

### 3.2 Per-pass breakdown on `wikidata ×4` (838.80 MB, 7 interleaved repeats)

| pass | Native L3-warm med `[min..max]` (GB/s, % ship) | Native Cold-stream med `[min..max]` (GB/s, % ship) | Baseline L3-warm med (% ship) | Baseline Cold-stream med (% ship) |
|---|---:|---:|---:|---:|
| `scan_scalar` | 770.24 `[747.56..842.40]` (1.09, 542.8%) | 752.00 `[739.57..797.35]` (1.12, 377.6%) | 536.93 (310.2%) | 536.63 (252.5%) |
| `scan_swar64` | 159.44 `[156.14..169.99]` (5.26, 112.4%) | 211.84 `[209.10..214.67]` (3.96, 106.4%) | 192.78 (111.4%) | 209.76 (98.7%) |
| **`scan_autovec32` (exact `find_runs_avx2` on native)** | **56.68** `[55.20..57.81]` (14.80, **40.0%**) | **103.60** `[99.62..112.68]` (8.10, **52.0%**) | **90.24** (52.1%) | **123.35** (58.0%) |
| **`pass2_shift_up_only` (`shift > 0`, 25.3% B)** | **7.51** `[7.26..8.30]` (111.73, **5.3%**) | **22.42** `[21.82..23.14]` (37.41, **11.3%**) | **7.26** (4.2%) | **22.43** (10.6%) |
| **`pass3_expand_no_crc` (90.8% B moved)** | **31.31** `[29.85..32.22]` (26.79, **22.1%**) | **83.29** `[82.62..84.31]` (10.07, **41.8%**) | **29.29** (16.9%) | **84.29** (39.7%) |
| **`pass4_crc32_only` (shipped PCLMULQDQ)** | **60.44** `[59.64..64.84]` (13.88, **42.6%**) | **101.85** `[100.66..102.85]` (8.24, **51.2%**) | **58.60** (33.9%) | **99.77** (46.9%) |
| `pass4_crc32_tiled16k` | 64.32 `[63.21..68.44]` (13.04, 45.3%) | 104.52 `[103.12..105.55]` (8.03, 52.5%) | 61.86 (35.7%) | 101.41 (47.7%) |
| `shipped_unrle_avx2_only` (real `rle::unrle`) | 101.21 `[99.33..104.14]` (8.29, 71.3%) | 131.10 `[126.43..135.14]` (6.40, 65.8%) | 90.57 (52.3%) | 125.23 (58.9%) |
| **`shipped_inplace_total` (64K tiled CRC)** | **141.89** `[137.45..145.08]` (5.91, **100.0%**) | **199.13** `[190.57..238.47]` (4.21, **100.0%**) | **173.11** (100.0%) | **212.52** (100.0%) |

Three crucial observations from Part 1B:
1. **PCLMULQDQ `crc32` is already compute-bound at L1d speed when streaming from L3:** `60.44 ms` L3-warm vs `61.76 ms` L1d-hot (identical within noise!). Hardware prefetching completely hides L3 read latency behind 13.6–14.4 GB/s PCLMULQDQ execution.
2. **Shipped `unrle_in_place` already captures L2 overlap on `expand + crc32`:** summing isolated passes gives `56.68 + 7.51 + 31.31 + 60.44 = 155.94 ms`, yet `shipped_inplace_total` runs in `141.89 ms` (`14 ms` faster) because `src/rle.rs:85` already computes CRC32 in 64 KiB tiles as compaction writes them.
3. **Shift-up is small (`7.51 ms` = `5.3%` L3-warm):** confirming `third-pass.md` §3's pre-data prediction ("under 9% of A9").

---

## 4. Part 2 — Single-write-pass & single-pass designs A/B (`wikidata ×4`, 838.80 MB, 7 repeats)

### 4.1 Native AVX2 + PCLMULQDQ (`-C target-cpu=native`) — apples-to-apples with shipped SIMD

| arm | L3-warm med `[min..max]` ms | vs shipped | non-ov | Cold-stream med `[min..max]` ms | vs shipped | non-ov |
|---|---:|---:|:---:|---:|---:|:---:|
| **`shipped_inplace`** (faithful `src/rle.rs`) | **143.49** `[139.61..152.52]` | +0.00% | — | **194.90** `[192.15..199.30]` | +0.00% | — |
| `backward_inplace_sep_crc` | 142.40 `[139.76..144.16]` | −0.76% | overlap | 195.70 `[187.85..202.97]` | +0.41% | overlap |
| `backward_inplace_tiled32k` (GF(2) combine) | 141.25 `[139.86..144.86]` | −1.56% | overlap | 194.43 `[191.31..198.35]` | −0.24% | overlap |
| `shipped_oop_avx2_sep_crc` (real `rle::unrle` + `crc32`) | 162.96 `[160.32..169.28]` | **+13.57%** | SLOWER | 240.79 `[234.82..249.81]` | **+23.55%** | SLOWER |
| `sp_oop_fused_notile` (pre-sized `&mut [u8]`) | 175.31 `[171.57..181.29]` | +22.17% | SLOWER | 304.20 `[301.26..312.85]` | +56.08% | SLOWER |
| `sp_oop_fused_tiled16k` (hot `stage`) | 181.18 `[174.46..182.60]` | +26.27% | SLOWER | 300.14 `[298.26..311.75]` | +54.00% | SLOWER |
| `sp_oop_fused_tiled32k` (hot `stage`) | 175.68 `[170.19..181.08]` | +22.43% | SLOWER | 299.25 `[297.20..303.51]` | +53.54% | SLOWER |
| `sp_oop_fused_tiled64k` (hot `stage`) | 174.39 `[170.07..178.39]` | +21.53% | SLOWER | 296.97 `[296.26..301.87]` | +52.37% | SLOWER |
| `sp_oop_fused_16k_cold_dst` (cold 2nd buf) | 189.33 `[184.30..3338.41]` | +31.94% | SLOWER | 228.54 `[225.71..232.05]` | +17.26% | SLOWER |
| **`sp_inplace_q_tiled16k`** (1-pass in-place, L1d queue) | **138.43** `[135.43..140.58]` | **−3.52%** | overlap | **182.96** `[181.95..187.47]` | **−6.12%** | **FASTER** |
| `sp_inplace_1w_notile` | 153.04 `[152.58..158.56]` | +6.66% | SLOWER | 209.35 `[205.86..213.83]` | +7.42% | SLOWER |
| `sp_inplace_1w_tiled16k` | 156.62 `[152.47..158.62]` | +9.15% | overlap | 201.29 `[199.57..205.76]` | +3.28% | SLOWER |
| `sp_inplace_1w_tiled32k` | 155.07 `[150.98..157.86]` | +8.07% | overlap | 204.90 `[196.68..219.25]` | +5.13% | overlap |
| `sp_inplace_1w_tiled64k` | 163.49 `[156.83..172.06]` | +13.94% | SLOWER | 206.33 `[201.32..208.01]` | +5.86% | SLOWER |
| **`crc32_only_floor`** (PCLMULQDQ alone) | **61.75** `[60.55..70.35]` | **−56.97%** | FASTER | **102.14** `[99.42..105.52]` | **−47.59%** | FASTER |

### 4.2 Baseline x86-64 (default release, SSE2 auto-vec vs runtime-dispatched AVX2 `rle::unrle`)

| arm | L3-warm med `[min..max]` ms | vs shipped | Cold-stream med `[min..max]` ms | vs shipped |
|---|---:|---:|---:|---:|
| **`shipped_inplace`** (SSE2 `find_runs` + PCLMULQDQ) | **175.65** `[169.67..179.04]` | +0.00% | **216.57** `[206.21..225.96]` | +0.00% |
| `backward_inplace_sep_crc` | 173.11 `[170.09..186.41]` | −1.45% | 212.30 `[207.18..236.52]` | −1.97% |
| `backward_inplace_tiled32k` | 178.40 `[167.32..181.44]` | +1.56% | 218.63 `[213.11..242.34]` | +0.95% |
| `shipped_oop_avx2_sep_crc` *(runtime AVX2 advantage on baseline)* | 150.17 `[147.40..158.62]` | −14.51% | 244.04 `[239.56..353.36]` | +12.69% |
| `sp_oop_fused_tiled16k` | 182.63 `[177.91..194.80]` | +3.97% | 313.96 `[301.98..330.66]` | +44.97% |
| **`sp_inplace_q_tiled16k`** | **170.96** `[163.52..175.02]` | **−2.67%** | **213.50** `[206.09..220.55]` | **−1.42%** |
| **`crc32_only_floor`** | **58.70** `[57.13..61.10]` | −66.58% | **100.20** `[97.98..148.99]` | −53.73% |

*(Note: on baseline x86-64, `shipped_oop_avx2_sep_crc` appears −14.5% faster L3-warm only because `mibz2::rle::unrle` runs runtime-dispatched AVX2 while our safe `#![forbid(unsafe_code)]` mirror of `find_runs` compiles to SSE2; once both run AVX2 under `-C target-cpu=native`, `shipped_oop_avx2_sep_crc` is **+13.57% slower** L3-warm and **+23.55% slower** cold-stream!)*

### 4.3 Category breakdown (`wikidata ×4`, Per-block L3-warm, native AVX2) and `ab gen` summary

- **`wikidata ×4` by block class (native L3-warm):**
  - `runs == 0` (196 runts, 0.0 MiB): `0.07 ms` across all arms.
  - `runs > 0, shift == 0` (176 blks, 74.7% B): `shipped = 105.58 ms`, `sp_inplace 16K = 97.30 ms` (**−7.8%**; and **−10.9%..−11.5%** when batched across 16 MiB), `oop_fused16k = 115.48 ms` (**+9.4%**).
  - `shift > 0` (59 blks, 25.3% B): `shipped = 38.64 ms`, `bwd_tiled = 38.41 ms` (**−0.6%**), `oop_fused16k = 40.24 ms` (**+4.2%**).
- **`ab gen` corpora (native AVX2, L3-warm / Cold-stream vs `shipped_inplace`):**
  - `onebyte.bz2` (51×): `sp_inplace_q_tiled16k` **−29.7% / −28.3%** (FASTER; eliminates redundant `find_runs` before `shift > n` stage-swap + tiles CRC).
  - `runs.bz2` (39.3×): `sp_inplace_q_tiled16k` **−18.2% / −19.2%** (FASTER).
  - `pattern.bz2` (100% `runs == 0`): `sp_inplace_1w_tiled32k` **−7.9%** warm, `sp_inplace_q_tiled16k` **−6.9%** cold; out-of-place regresses **+16%..+46%** by writing 19.6 MB needlessly.
  - `random.bz2` / `records.bz2` (67–82% `runs == 0`): `sp_inplace_1w_tiled16k` **−0%..−12%**; out-of-place regresses **+18%..+55%**.
  - `text.bz2` (100% `shift > 0`, 22,148 runs/blk): `shipped_oop_avx2_sep_crc` wins **−44.0% / −47.3%** due to BMI1 `tzcnt` per run hit (irrelevant on `wikidata`'s 27.9 runs/blk).

---

## 5. Memory-traffic accounting and 8–10W i9-10900K LLC evaluation

### 5.1 Exact logical read/write byte accounting (`wikidata ×4`, 1,724 blocks, 799.94 MiB decoded)

| design | logical reads (MiB) | logical writes (MiB) | total R+W (MiB) | write cut vs shipped | per-worker live RLE1 buffers |
|---|---:|---:|---:|---:|---|
| `shipped_inplace` (`src/rle.rs`) | 2,516.3 | **916.4** | 3,432.7 | 0.0% | 1 × 0.86 MiB (`out`) |
| `backward_inplace` (`min_grown >= 0`) | 2,477.2 | 877.3 | 3,354.4 | **−4.3%** (−39.1 MiB) | 1 × 0.86 MiB (`out`) |
| `single_pass_oop_tiled` | 800.0 | 799.9 | 1,599.9 | −12.7% (−116.5 MiB) | **2 × 0.86 MiB = 1.72 MiB** (+8.6 MiB @ 10W!) |
| **`single_pass_inplace`** | **800.0** | **726.0** | **1,526.0** | **−20.8% (−190.4 MiB)** | **1 × 0.86 MiB + 16 KiB L1d queue** |

*(In `shipped_inplace`, `916.4 MiB` of writes = `726.0 MiB` unavoidable compaction writes + `190.4 MiB` shift-up writes on the 59 `shift > 0` blocks.)*

### 5.2 Implication for the i9-10900K (20 MiB L3) at 8–10W and frozen stop-test verdict

Re-reading `third-pass.md` §1 (re-derived from `c71c4a56` `runs.jsonl` on real code — separating real `none`/`S` from the cold-store oracle `PC`/`SC` arms):
- Real `A9` (summed over workers) in `none` is **74.4 ms** (1W), **88.4 ms** (6W), **102.8 ms** (8W = **7.1%** of ~1,443 ms active worker CPU), and **131.3 ms** (10W = **8.3%** of ~1,577 ms active worker CPU; `S` = `127.7 ms`).
- Total real `A9` inflation from 6W → 8W is **+14.4 ms** (**1.00%** of 8W worker CPU), and from 6W → 10W is **+42.9 ms** (**2.72%** of 10W worker CPU), of which ~7% (`~8.6 ms`) is SMT/oversubscription (11–12 threads on 10 cores, seen equally in the scanner's few-KiB A2+A3 table).
- **Applying single-pass in-place's savings:**
  - Measured cold-stream cut (`−6.12%` native, `−8.25%` v2): **0.43–0.68% of worker CPU** at 8–10W.
  - Generous upper bound applying the entire **−20.8%** write-traffic reduction to all of `A9`: `0.208 × 7.1% = 1.48%` at 8W and `0.208 × 8.3% = 1.73%` at 10W (realistically `0.208 × ~34 ms` LLC write-miss inflation + `~10 ms` L3 read savings ≈ **~17 ms ≈ 1.1% of 10W worker CPU**).
  - Out-of-place designs (`sp_oop_*`) would **worsen** 8–10W LLC misses by adding `+0.86 MiB/worker` (`+6.9..+8.6 MiB` of a 20 MiB L3) while only cutting writes by −12.7%.

**Frozen `stop_test` check (C2 — never relaxed):**
- **1W gate (`≥ 35%` of A9, ~25 ms / ≥ 1.8% wall):** **FAIL.** L1d-hot `crc32 + scan` alone is 72.3% of shipped A9 (ceiling −27.7% even with zero writes); best prototype is **−3.52%** L3-warm / **−6.12%** cold-stream.
- **8–10W gate (`≥ 2.0%` of worker CPU via LLC store misses / DRAM):** **FAIL.** Write traffic drops at most −20.8% (`−190.4 MiB`), bounding 8–10W worker CPU savings to **~0.6–1.7%** (strictly `< 2.0%`).

**Why R cannot reach 35% standalone — and where the real lever lives:** Standalone RLE1+CRC must scan 800 MiB for 4-equal-byte runs, move 726 MiB in place, and run PCLMULQDQ CRC32 over 800 MiB. The only way to eliminate the post-BWT scan and 726 MiB compaction pass is to **fuse RLE1 expansion + tiled CRC32 directly into forward BWT emission** (`forward-emission`), expanding runs and updating CRC32 in L1d/registers as bytes are emitted strictly forward (impossible under current bidirectional `invert_runs`, naturally unlocked if `forward-emission` lands).
