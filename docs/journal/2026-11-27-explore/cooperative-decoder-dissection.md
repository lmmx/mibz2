# Cooperative Decoder Dissection (`src/coop.rs`)

**Date:** 2026-11-27
**Claim:** `explore` (smooth, recurring; zero `src/` edits)
**Reproducer:** [`examples/coop_dissection_probe.rs`](../../../examples/coop_dissection_probe.rs) (`--features moonshot`, `--repeats 7`, baseline x86-64 and `-C target-cpu=native`)
**Hardware (C5 diagnostic):** 2 physical cores (no SMT), Intel Xeon @ 2.30 GHz (AVX2, BMI1/BMI2, PCLMULQDQ, 45 MiB L3). `k = 1, 2` measure real 1- and 2-core hardware scaling; `k = 4` measures 2x oversubscribed thread/barrier contention.

---

## Executive Verdict

1. **`Cooperative` is NOT a single-block / few-block accelerator and is UNREACHABLE on `<= 10` cores (including CI and the i9-10900K) from `StreamDecoder::new`.** `Plan::for_threads` ignores block count completely; on `phys in 8..=10`, `plan_with` (`src/parallel.rs:3578,3585`) returns `Plan::Workers` for *every* thread count (`1..=usize::MAX`), rendering both `PIPELINED_PLAN` (`L3588`) and `SERVICE_PLAN` (`L474`) unreachable via `StreamDecoder::new`. The only default production trigger of `src/coop.rs` today is Linux hosts with `phys >= 11, threads >= 11` (or `phys in 6..=7, threads > phys`), where it dispatches `Teams { k: 2 }` (`Cooperative<2>`).
2. **On `wikidata`, `Cooperative<K>` loses badly (`0.37x..0.73x`) to 1-thread `OpenCells`** because `Cooperative<K>` never runs `Open` (it builds a 3.6 MB `TVector` via `runs::front -> cuts -> build_runs -> walk_room_back`). On 1 block (0.90 MB, native), 1-thread `OpenCells` finishes in **`3.07 ms`** vs **`8.22 ms`** (`0.37x`) for `Cooperative<2>` and **`5.78 ms`** (`0.53x`) for `Cooperative<4>`.
3. **Amdahl's Law caps full-block `Cooperative<K>` speedup even on non-`Open` corpora:** serial phases (`Huffman + MTF/front + cuts + walk_setup + assemble + RLE1+CRC`) consume **`48%..71%` of `k=1` wall** and **`61%..81%` of `k=2` wall** (`text` `63.2%`, `prose_src` `61.5%`, `records` `80.5%`, `random` `80.8%`). On high-entropy `Column` blocks (`records`, `random`), serial `Huffman (17.5%) + mtf_checkpoints_avx2 (50–51%)` alone account for **68–69% of `k=1` wall** (`78%` at `k=2`).
4. **Thread spawn and barrier latency are NOT the bottleneck:** cold `Pool::new(k)` + 1st run is `110–131 us` (`k=2`) / `282–290 us` (`k=4`); warm `Pool::run` barrier is `1.01 us` spinning (`77 us` when sleeping). Replacing the 2-barrier `Pool::run` with a single spin-yield barrier hurts oversubscribed `k=4` wall (`+11%..+47%`) without helping `k=2`.
5. **Candidate kernel levers inside `src/coop.rs` cut inversion `-13%..-33%`, but full-block wall only `-5%..-6%` (`< 10%` gate):**
   - **Parallel sequential `LF` build (`L2_lf_build`)** replacing `FusedBuild`'s 256-stream random `Ψ` scatter cuts `Column` inversion **`-24.9%..-27.8%`** (DISJOINT-FAST at `k=1,2` native & baseline), and stacked with **staged `TILE=256` 8-way unrolled walk (`L1_staged_walk`) + trusted `fill_share_avx2` + AVX2 `assemble_reversed`** (`C_all`) cuts inversion **`-30.4%..-33.0%`** on `records`/`random` (`-31%..-39%` CPU), **`-19.4%..-24.1%`** on `prose_src`, and **`-12.9%..-18.5%`** on `text`.
   - **HOWEVER**, after Amdahl weighting (`build + walk + asm` is only `19.5%` of `records`/`random` wall and `40%..48%` of `prose_src`/`text` wall at `k=2`), full-block wall improves only **`-5.2%..-6.2%`**, failing the `>= 10%` threshold. No `coop-build` claim is opened.
6. **Non-obvious negative result (recorded in `dead-ends.tsv`):** porting `mtf_avx2`'s two-level register-resident `(f0, f1)` list + branchless 7-row `second()` to `mtf_checkpoints_avx2` (`L3_reg_mtf_cp`) **regresses `+42.2%..+42.6%` on `records` and `+24.7%..+42.4%` on `random`** (`64.51` vs `45.37 ms` native), because on uniform high-entropy `Column` blocks (`syms/row = 1.000`), **87.5% of MTF indices are `nn >= 32`** (hitting the 224-byte SIMD `second()` scan every literal, whereas shipped `PagedList` executes `ptr::copy(base, base + 1, nn)` over `~112` bytes on average).

---

## 1. Production Trigger Audit (`src/parallel.rs:469-504, 3573-3598`)

Verified directly at runtime via `Plan::for_threads` and `StreamDecoder::new` with `Machine::service_target()` (exact i9-10900K: `phys = 10`, `L3 = 20 MiB`):

| Topology (`phys`, `L3`) | Requested `threads` | `Plan::for_threads` | `is_service` | `is_pipelined` | Runs `src/coop.rs`? |
|---|---:|---|---:|---:|---|
| 2–5 cores (CI, VM, laptop) | any | `Workers(t)` | `false` | `false` | **No** |
| 6–7 cores | `<= phys` | `Workers(t)` | `false` | `false` | **No** |
| 6–7 cores | `> phys` | `Teams { teams, k: 2 }` | `false` | `false` | **Yes (`Cooperative<2>`)** |
| 8–10 cores (i9-10900K) | `1..=10` | `Workers(t)` (`trimmed_here` on i9) | `false` | `false` | **No** (`L3578`) |
| 8–10 cores (i9-10900K) | `11..=usize::MAX` | `Workers(phys)` | `false` | `false` | **No** (`L3585`) |
| `>= 11` cores (12–64c Linux) | `<= 10` | `Workers(t)` | `false` | `false` | **No** (`L3578`) |
| `>= 11` cores (12–64c Linux) | `>= 11` | `Teams { (cores/2).min(cap), k: 2 }` | `false` | `false` | **Yes (`Cooperative<2>`)** |

**Dead-branch finding:** In `plan_with` (`src/parallel.rs:3578-3590`), `LAZY_FROM_THREADS = 8` and `WORKERS_BELOW_THREADS = 11`:
- `phys = 10, threads <= 10` returns `Plan::Workers(threads)` at line 3578;
- `phys = 10, threads > 10` returns `Plan::Workers(10)` at line 3585;
- Therefore line 3588 (`phys == 10 && l3 == 20 MiB -> Teams { 3, 3 }` = `PIPELINED_PLAN`) and `StreamDecoder::new` line 474 (`SERVICE_THREADS == 10 && !Workers` -> `SERVICE_PLAN`) are **unreachable via `StreamDecoder::new`**.

---

## 2. Pool Spawn & Barrier Micro-Benchmark (7 Repeats, us)

| Team size `k` | Build | Cold `Pool::new` + 1st `run` (us) | Warm `Pool::run` barrier (us) | 2 barriers/blk (us) |
|---:|---|---:|---:|---:|
| `k = 2` | baseline | `109.9 [49.7..376.5]` | `1.01 [0.92..1.14]` (spinning) | `2.02` |
| `k = 2` | native | `130.8 [123.6..168.0]` | `76.61 [74.13..97.94]` (parked) | `153.22` |
| `k = 4` | baseline | `282.2 [262.6..442.5]` | `175.37 [170.31..188.84]` | `350.75` |
| `k = 4` | native | `290.4 [261.0..4030.1]` | `174.68 [166.14..215.14]` | `349.37` |

Cold pool creation is `0.11–0.29 ms` (negligible even on a single 900 KB block of `8–25 ms`); warm barriers add `0.002–0.15 ms/block` at `k=2` (`< 1%` of block time).

---

## 3. End-to-End Shipped `StreamDecoder::planned` on 1-, 2-, and 4-Block Streams (Native, 2 Cores, 7 Repeats)

Median wall ms (ratio vs `W(1)`, summed process CPU ms):

| Corpus (route) | Blocks | `W(1)` (`OpenCells`) | `W(2)` | `W(4)` | `T(1x2)` (`Coop<2>`) | `T(1x4)` (`Coop<4>`) | `T(2x2)` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` (`Open`) | 1 (0.90 MB) | **`3.07`** (`3.1` CPU) | `3.10` (`0.99x`) | `5.63` (`0.55x`) | **`8.22` (`0.37x`, `8.3` CPU)** | `5.78` (`0.53x`) | `4.55` (`0.68x`) |
| `wikidata` (`Open`) | 2 (1.80 MB) | **`7.17`** (`7.1` CPU) | `7.32` (`0.98x`) | `8.44` (`0.85x`) | `10.85` (`0.66x`, `10.9` CPU) | `10.26` (`0.70x`) | `11.50` (`0.62x`) |
| `wikidata` (`Open`) | 4 (3.60 MB) | **`13.16`** (`13.1` CPU) | `15.09` (`0.87x`) | `15.75` (`0.84x`) | `19.61` (`0.67x`, `19.5` CPU) | `19.81` (`0.66x`) | `18.12` (`0.73x`, `22.0` CPU) |
| `text` (`Runs`) | 1 (1.01 MB) | **`8.02`** (`8.0` CPU) | `8.00` (`1.00x`) | `11.76` (`0.68x`) | `9.16` (`0.88x`, `9.1` CPU) | `10.85` (`0.74x`) | `9.31` (`0.86x`) |
| `text` (`Runs`) | 4 (4.21 MB) | **`30.90`** (`30.7` CPU) | `32.24` (`0.96x`) | `33.07` (`0.93x`) | `35.64` (`0.87x`, `35.3` CPU) | `35.30` (`0.88x`) | `33.64` (`0.92x`, `38.5` CPU) |
| `records` (`Col`) | 1 (0.90 MB) | **`22.28`** (`22.3` CPU) | `23.51` (`0.95x`) | `24.54` (`0.91x`) | `25.20` (`0.88x`, `24.6` CPU) | `25.09` (`0.89x`) | `24.94` (`0.89x`) |
| `records` (`Col`) | 4 (3.60 MB) | `88.67` (`87.9` CPU) | `94.33` (`0.94x`) | `84.06` (`1.05x`) | `95.92` (`0.92x`, `98.1` CPU) | `96.69` (`0.92x`) | `93.04` (`0.95x`, `101.3` CPU) |
| `random` (`Col`) | 1 (0.90 MB) | `23.44` (`23.3` CPU) | `23.62` (`0.99x`) | `30.59` (`0.77x`) | `23.48` (`1.00x`, `23.3` CPU) | `23.64` (`0.99x`) | `23.93` (`0.98x`) |
| `random` (`Col`) | 4 (3.60 MB) | `89.30` (`89.1` CPU) | `93.55` (`0.95x`) | `86.98` (`1.03x`) | `91.99` (`0.97x`, `93.5` CPU) | `92.50` (`0.97x`) | `81.19` (`1.10x`, `97.4` CPU) |
| `prose_src` (1R/3C) | 1 (0.90 MB) | `11.62` (`11.4` CPU) | `11.56` (`1.01x`) | `12.83` (`0.91x`) | `11.45` (`1.01x`, `14.3` CPU) | `13.12` (`0.89x`) | `11.82` (`0.98x`) |
| `prose_src` (1R/3C) | 4 (3.60 MB) | `37.01` (`39.7` CPU) | **`27.79` (`1.33x`)** | `27.57` (`1.34x`) | `33.93` (`1.09x`, `50.1` CPU) | `36.16` (`1.02x`) | `27.85` (`1.33x`, `51.5` CPU) |

Baseline x86-64 shows the exact same hierarchy (`wikidata 1blk`: `W(1) = 3.11 ms` vs `T(1x2) = 8.21 ms [0.38x]`; `text 1blk`: `8.26` vs `10.14 ms [0.81x]`; `prose_src 1blk`: `11.88` vs `13.94 ms [0.85x]`).

---

## 4. Exact Phase Breakdown of `Cooperative<K>` (4 Full Blocks, Native, Median Wall ms)

| Corpus (path) | `k` | Total | `huff` | `front`/`mtf` | `cuts` | `build` (par) | `wset` | `walk` (par) | `asm` | `rle+crc` | Serial % | `1t OpenCells` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` (`Runs`) | 1 | `16.98` | `1.24` (7.3%) | `0.99` (5.8%) | `0.55` (3.2%) | `2.28` (13.5%) | `0.07` | `10.37` (61.1%) | `0.61` (3.6%) | `0.86` (5.1%) | `25.4%` | **`12.99`** |
| `wikidata` (`Runs`) | 2 | `18.91` | `1.24` (6.5%) | `1.06` (5.6%) | `0.60` (3.2%) | `2.70` (14.3%) | `0.17` | `11.30` (59.7%) | `0.91` (4.8%) | `0.93` (4.9%) | `26.0%` | **`12.99`** |
| `text` (`Runs`) | 1 | `33.79` | `4.88` (14.5%) | `4.14` (12.3%) | `2.09` (6.2%) | `5.78` (17.1%) | `0.09` | `11.76` (34.8%) | `0.87` (2.6%) | `4.18` (12.4%) | `48.1%` | `28.77` |
| `text` (`Runs`) | 2 | `25.64` | `4.88` (19.0%) | `4.06` (15.8%) | `2.16` (8.4%) | `3.58` (14.0%) | `0.17` | `5.86` (22.9%) | `0.80` (3.1%) | `4.13` (16.1%) | **`63.2%`** | `28.77` |
| `records` (`Col`) | 1 | `88.24` | `15.41` (17.5%) | **`45.35` (51.4%)** | `0.00` | `12.49` (14.2%) | `0.09` | `13.09` (14.8%) | `0.75` (0.8%) | `1.07` (1.2%) | **`71.0%`** | `86.24` |
| `records` (`Col`) | 2 | `77.99` | `15.41` (19.8%) | **`45.23` (58.0%)** | `0.00` | `7.99` (10.2%) | `0.20` | `7.25` (9.3%) | `0.86` (1.1%) | `1.06` (1.4%) | **`80.5%`** | `86.24` |
| `random` (`Col`) | 1 | `87.93` | `15.36` (17.5%) | **`43.95` (50.0%)** | `0.00` | `13.22` (15.0%) | `0.09` | `13.46` (15.3%) | `0.76` (0.9%) | `1.09` (1.2%) | **`69.7%`** | `87.26` |
| `random` (`Col`) | 2 | `73.97` | `15.36` (20.8%) | **`42.42` (57.3%)** | `0.00` | `7.23` (9.8%) | `0.17` | `7.00` (9.5%) | `0.83` (1.1%) | `0.97` (1.3%) | **`80.8%`** | `87.26` |
| `prose_src` (1R/3C) | 1 | `44.12` | `6.43` (14.6%) | `16.32` (37.0%) | `0.55` (1.2%) | `7.48` (16.9%) | `0.08` | `11.04` (25.0%) | `0.66` (1.5%) | `1.56` (3.5%) | `58.0%` | `39.67` |
| `prose_src` (1R/3C) | 2 | `43.04` | `6.43` (14.9%) | `16.93` (39.3%) | `0.55` (1.3%) | `6.75` (15.7%) | `0.16` | `9.81` (22.8%) | `0.79` (1.8%) | `1.61` (3.7%) | **`61.5%`** | `39.67` |

**Scaling observations (`1t -> 2t` on 2 physical Xeon cores):**
- Inside `build` and `walk` on `text`, `records`, and `random`, parallel scaling is healthy: `build` scales **`1.56x–1.83x`** and `walk` scales **`1.81x–2.01x`** at `k=2`.
- Yet **whole-block `1t -> 2t` scaling is only `1.13x–1.32x`** because `48%–71%` of `k=1` time is strictly serial (`Huffman`, `mtf_checkpoints` / `runs::front` + `cuts`, `assemble`, `RLE1+CRC`).
- On `text` (`4` blks), `runs::cuts` alone takes `2.09–2.16 ms` (`8.4%` of `k=2` wall — more than half of `build`!).

---

## 5. Candidate Kernel Lever Benchmarks (100% Byte-Exact, 7 Interleaved Repeats)

### 5.1 Inversion Phase (`cuts + build + walk + assemble`, `k = 1, 2`)

| Corpus | `k` | Build | `shipped_coop` (wall / CPU ms) | `L1_staged_walk` | `L2_lf_build` | `C_all_combined` | Est. full-block wall delta |
|---|---:|---|---:|---:|---:|---:|---:|
| `records` (`Col`) | 1 | nat | `21.19` / `21.2` | `-2.0%` (`-1.8%` CPU) | **`-24.9%` DISJOINT** (`-25.0%` CPU) | **`-31.5%` DISJOINT** (`-31.4%` CPU) | `-7.6%` |
| `records` (`Col`) | 2 | nat | `13.79` / `25.5` | `+1.1%` (`-1.1%` CPU) | **`-27.6%` DISJOINT** (`-28.8%` CPU) | **`-33.0%`** (`-32.9%` CPU) | **`-5.8%`** |
| `records` (`Col`) | 2 | base | `39.76` / `26.0` | `-3.1%` (`-11.2%` CPU) | `-23.8%` (`-27.6%` CPU) | **`-48.5%`** (`-35.5%` CPU) | **`-6.1%`** |
| `random` (`Col`) | 1 | nat | `22.22` / `22.0` | `-5.1%` (`-4.1%` CPU) | **`-26.4%` DISJOINT** (`-25.8%` CPU) | **`-31.0%` DISJOINT** (`-30.3%` CPU) | `-7.8%` |
| `random` (`Col`) | 2 | nat | `13.14` / `24.6` | `-3.8%` (`-3.8%` CPU) | **`-27.8%` DISJOINT** (`-29.0%` CPU) | **`-30.4%`** (`-31.1%` CPU) | **`-5.5%`** |
| `random` (`Col`) | 2 | base | `22.74` / `22.7` | `-7.1%` (`-7.2%` CPU) | **`-24.9%` DISJOINT** (`-25.2%` CPU) | **`-31.2%` DISJOINT** (`-31.2%` CPU) | **`-5.5%`** |
| `text` (`Runs`) | 1 | nat | `18.28` / `17.9` | `-3.3%` (`-3.1%` CPU) | n/a | **`-14.3%` DISJOINT** (`-13.5%` CPU) | `-7.7%` |
| `text` (`Runs`) | 2 | nat | `12.36` / `19.9` | `-4.6%` (`-2.3%` CPU) | n/a | **`-12.9%`** (`-8.8%` CPU) | **`-6.2%`** |
| `text` (`Runs`) | 1 | base | `39.97` / `20.3` | `-10.3%` (`-10.6%` CPU) | n/a | **`-18.5%` DISJOINT** (`-19.5%` CPU) | `-8.5%` |
| `text` (`Runs`) | 2 | base | `35.43` / `21.6` | `-9.4%` (`-11.5%` CPU) | n/a | `+1.5%` wall (**`-19.4%` CPU**) | **`-6.0%`** |
| `prose_src` (mix) | 2 | nat | `11.40` / `20.6` | `-2.6%` (`-5.9%` CPU) | `-9.7%` (`-17.8%` CPU) | **`-19.6%`** (`-19.6%` CPU) | **`-5.2%`** |
| `wikidata` (`Runs`) | 1 | nat | `15.77` / `13.6` | `-16.3%` (`-3.1%` CPU) | n/a | `-18.6%` (`-6.5%` CPU) | Still `>1.25x` slower than `1t OpenCells` |

### 5.2 Column MTF Lever 3 (`mtf_checkpoints_reg_avx2` vs shipped `PagedList`) — STOP

| Corpus | Build | Shipped `mtf_checkpoints_avx2` (`PagedList`) | Candidate `mtf_checkpoints_reg_avx2` (`f0,f1` + `second`) | Change |
|---|---|---:|---:|---:|
| `records` (`Column`) | native | `45.37 ms [44.78..46.32]` | **`64.51 ms [63.80..65.62]`** | **`+42.2%` (DISJOINT-SLOW)** |
| `records` (`Column`) | baseline | `69.20 ms [59.71..103.55]` | **`98.67 ms [81.43..127.65]`** | **`+42.6%`** |
| `random` (`Column`) | native | `42.40 ms [42.12..43.89]` | **`60.38 ms [59.85..61.26]`** | **`+42.4%` (DISJOINT-SLOW)** |
| `random` (`Column`) | baseline | `84.01 ms [62.88..92.09]` | **`104.76 ms [56.77..118.05]`** | **`+24.7%`** |
| `prose_src` (mixed) | native | `18.92 ms [18.54..19.85]` | `17.27 ms [16.70..19.35]` | `-8.7%` |

**Why:** On uniform high-entropy blocks (`records`, `random` — the actual `Column` path where `!takes_runs`), 87.5% of MTF indices satisfy `nn >= 32`. `mtf_avx2`'s `second()` executes a full 7-row (224-byte) branchless AVX2 compare/pack/blend loop on every `nn >= 32`, whereas shipped `mtf_checkpoints_avx2` runs `ptr::copy(base, base + 1, nn)` over `~112` bytes on average.

---

## 6. Conclusions & Recommendations

1. **Do NOT open a `coop-build` claim:** Although `L2_lf_build + L1_staged_walk + trusted fill_share + AVX2 assemble_reversed` cuts `Cooperative`'s inversion kernel `-13%..-33%` (disjoint on `Column` and `text` `k=1`), Amdahl's Law (`61%..81%` serial wall at `k=2`) caps full-block wall gain to **`-5.2%..-6.2%` (`< 10%`)**, `Cooperative` loses `0.37x..0.73x` to 1-thread `OpenCells` on Wikidata regardless, and `StreamDecoder::new` never invokes `src/coop.rs` on `<= 10` cores (including CI and the i9-10900K).
2. **Free inheritance from `segmented-fallback-build`:** When `segmented-fallback-build` vectorizes `SegScratch::assemble_reversed` in `src/bwt.rs`, `Cooperative::invert_runs` automatically inherits that gain at zero extra cost.
3. **Future cleanup note (not urgent for the 6 GB/s i9 milestone):** On `>= 12`-core Linux machines (`threads >= 11`), `plan_with` (`src/parallel.rs:3594`) currently forces `Teams { k: 2 }` (`Cooperative<2>`, bypassing `OpenCells`), and lines `3588` (`PIPELINED_PLAN`) and `474` (`SERVICE_PLAN`) are unreachable through `StreamDecoder::new`. Any future `>= 12`-core scaling pass should benchmark `Workers(phys)` (`OpenCells`) against `Teams { k: 2 }` before optimizing `src/coop.rs`.
