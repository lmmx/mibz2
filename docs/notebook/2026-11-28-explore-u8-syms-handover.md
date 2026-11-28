# Compact `u8` Symbol Handover (`SymBuf`: `u8` + Sparse `pos256`) Across All Three Coupled Stations

**Date:** 2026-11-28
**Claim:** `explore` (smooth zone)
**Base commit:** `6ce89cc5` (rebased cleanly onto `7be50513`)
**Hardware (local VM):** Intel Xeon Haswell-EP (2 vCPU, 32 KiB L1d, 256 KiB L2, 45 MiB L3, AVX2 + BMI1/BMI2 + LZCNT + POPCNT)

## Summary

Today `mibz2` hands decoded Huffman/MTF symbols from the serial scanner (`S`) through `Shared` (`free_syms` + `jobs`) to workers (`W`: `runs::front_avx2` and `mtf::mtf_avx2`) as `Vec<u16>` (`MAX_SYMS = 900,100` -> `1.72 MiB` capacity per slot, `2 B/sym` traffic, trailing `eob` element stored in-band).

A cold 7-corpus census, isolated station dissection (`base` portable fat-LTO and `JCC-32B` aligned fat-LTO, warm and L3-cold, `n = 11` alternating pairs), and full coupled end-to-end prototype (`default` bidirectional and `--features forward-anchor-lanes`, `1T/4T/8T` across all 7 corpora + `wikidata-x4` 839 MB, 100% SHA-256 parity) establish that replacing `Vec<u16>` with a **single unified compact buffer `SymBuf { bytes: Vec<u8>, pos256: Vec<u32> }`** (`EOB` implicit at `bytes.len()`, `255` placeholder at `pos256` indices, zero `pos256` allocation for `n_in_use <= 255`) delivers **simultaneous, disjoint improvements across all three coupled stations** (`S`, `Shared` L3/RSS, and `W`), with zero guard regressions and **superlinear synergy with `#1153` (`K = 3` `u8`-chunk A3)**:

- **Census:** `100.0%` of blocks on `wikidata` (`431/431`, `max_niu = 102`), `text` (`27/27`, `max_niu = 142`), `runs` (`1/1`, `252`), `onebyte` (`1/1`, `3`), and `pattern` (`44/44`, `7`) have `n_in_use <= 255` (`0` occurrences of `sym == 256`). Even on `n_in_use == 256` corpora (`random` `18/18`, `records` `22/22`), `sym == 256` is only `0.1563%` (`~1,389/blk`) and `0.8743%` (`~7,630/blk`), so unified `u8 + pos256` cuts handover bytes by `-48.5%..-50.0%` on **all 7 corpora** with **zero dual-pool RSS bloat**.
- **Coupled E2E `wikidata-x4` (839 MB) at `8T` (`n = 7` paired, 100% SHA-256 exact):**
  - **Default (`bidi`):** scanner `mibz2-scan` CPU **`-9.70%` (`0.9030 [0.8838, 0.9226]`)**, workers `wkr_sum` CPU **`-2.77%` (`0.9723 [0.9543, 0.9906]`)**, total CPU **`-3.33%` (`0.9667 [0.9536, 0.9800]`)**, peak RSS (`VmHWM`) **`64.22 -> 60.32 MiB` (`-3.89 MiB`, `0.9455×`)** (and `-4.43 MiB` / `0.9319×` in the 20-cell suite).
  - **`--features forward-anchor-lanes`:** scanner `mibz2-scan` CPU **`-8.47%` (`0.9153 [0.9056, 0.9251]`)**, workers `wkr_sum` CPU **`-3.64%` (`0.9636 [0.9546, 0.9727]`)**, total CPU **`-3.85%` (`0.9615 [0.9540, 0.9691]`)**, wall **`-4.86%` (`0.9514×`)**, peak RSS **`65.86 -> 62.32 MiB` (`-3.55 MiB`, `0.9474×`)**.
- **Superlinear synergy with `#1153` (`K = 3` `u8`-chunk A3):** `K = 3` already decodes into a `[u8; 1042]` staging chunk and pays a widening loop (`flush_u8_patched`) to copy into `Vec<u16>`. Flushing directly to `SymBuf::bytes` cuts `K = 3` A3 by an **extra `-6.75%..-9.19%` on `wikidata`** (`-15.80%..-17.14%` cold32 vs shipped `K = 2` `u16`), **`-6.37%..-9.11%` on `text`**, and **eliminates `K = 3`'s `+4.4%..+9.1%` regression on `random`/`records`**, flipping them to `-2.1%..-6.9%` faster than shipped `K = 2` `u16`.
- **Worker `A5` (`runs::front_avx2` + `mtf_avx2`):**
  - `runs::front_avx2` Pass 1 drops from 4×32B loads + 6 `vpacksswb` + 6 `vpermq` + `vptest` to **2×32B loads + `vpandn` + `vpcmpeqb` + `vpmovmskb` with `0 vpermq`, `0 vpack`, `0 vptest`**, running **`-50.3%..-51.4%` (`2×` faster)** across both `base` and `JCC-32B` (`-7.70%` full `front_avx2` on `JCC-32B` `wikidata`, `-8.92%` on `text`).
  - `mtf_avx2` segmented `u8 + pos256` (zero per-symbol `eob` and zero per-symbol `sym == 256` checks inside the hot loop) cuts isolated `mtf_avx2` **`-4.61%..-8.72%`** on `random` and **`-4.88%..-6.40%`** on `records`, translating to **`-4.5%..-6.2%` E2E CPU** and **`-3.0..-4.6 MiB` RSS** on `random`/`records` at `8T` (`forward-anchor-lanes`).

Filed as P2 `idea` claim **`u8-syms-handover-build`** (with a zero-breaking-API internal `CompactSymbols` implementation plan so `pub struct Symbols` stays semver-compatible and requires zero owner API sign-off).

---

## 1. Seven-Corpus & Mechanism Census

Measured across all 7 corpora (`/tmp/l6a_corpora/{wikidata,text,runs,onebyte,pattern,random,records}.bz2`, exact `Scanner::new` block walk):

| Corpus | Blocks | `n_in_use <= 255` | `n_in_use == 256` | Max `n_in_use` | Non-EOB Syms | `sym == 256` Count (%) | `u16` Handover (MiB) | `u8` Handover (MiB) | Max Blk Syms | `takes_runs` Blks |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | **431 (100%)** | 0 | 102 | 24,978,868 | 0 (0.0000%) | 47.64 | **23.82** | 199,770 | 383 |
| `text` | 27 | **27 (100%)** | 0 | 142 | 9,455,229 | 0 (0.0000%) | 18.03 | **9.02** | 424,495 | 27 |
| `runs` | 1 | **1 (100%)** | 0 | 252 | 124,189 | 0 (0.0000%) | 0.24 | **0.12** | 124,190 | 1 |
| `onebyte` | 1 | **1 (100%)** | 0 | 3 | 39 | 0 (0.0000%) | 0.00 | **0.00** | 40 | 1 |
| `pattern` | 44 | **44 (100%)** | 0 | 7 | 5,280 | 0 (0.0000%) | 0.01 | **0.01** | 121 | 44 |
| `random` | 18 | 0 (0%) | 18 | 256 | 15,999,059 | 25,010 (0.1563%) | 30.52 | **15.26** (+0.10) | 899,946 | 0 |
| `records` | 22 | 0 (0%) | 22 | 256 | 19,199,838 | 167,865 (0.8743%) | 36.62 | **18.31** (+0.64) | 899,979 | 0 |

### Why Unified `SymBuf { bytes: Vec<u8>, pos256: Vec<u32> }` Beats Dual-Pool `enum Syms { U8, U16 }`

1. **Zero dual-pool RSS bloat:** A single `free_syms: Vec<SymBuf>` serves every block regardless of `n_in_use`. Every block reserves at most `MAX_SYMS = 900,100` bytes (`879 KiB`) instead of `1.72 MiB`. On `n_in_use <= 255` (`100%` of `wikidata`, `text`, `runs`, `onebyte`, `pattern`), `pos256` never allocates (`capacity == 0`).
2. **Even `n_in_use == 256` wins by `-48.5%` memory and `-5%..-9%` MTF CPU:** Storing `255u8` as the placeholder byte at every `pos256` index preserves `s <= 1` (run symbol) verbatim in `runs::takes_runs` and `runs::front_avx2` Pass 1. In `mtf_avx2`, iterating contiguous slices `&bytes[cur..pi]` between `pos256` positions (`avg 648 syms/segment` on `random`, `118 syms/segment` on `records`) eliminates both per-symbol `s == b.eob` and per-symbol `s == 256` branches from the hot loop.
3. **Implicit `EOB`:** `block::read_item` only returns `Ok(Item::Block(..))` after decoding `EOB`. Omitting `EOB` from `SymBuf::bytes` makes `end = bytes.len()` exact even when `n_in_use == 255` (`eob = 256`), eliminating all `EOB` compares and `vptest` from `runs::front_avx2` Pass 1 and `s == b.eob` from `mtf_avx2`.
4. **Zero-breaking-API implementation path:** Because `pub struct Symbols { pub syms: Vec<u16>, .. }`, `block::read_item`, and `stream::Scanner::next` are public Rust APIs, the production build claim (`u8-syms-handover-build`) should introduce `pub(crate) struct SymBuf` + `pub(crate) struct CompactSymbols` inside the internal `Scanner::next_compact` / `StreamDecoder` / `BzReader` / `decode` / `decode_parallel` pipeline while preserving all existing public `Vec<u16>` APIs untouched (zero semver break, zero `needs_louis`).

---

## 2. Disassembly Audit (`objdump -dC`, Portable Release Fat-LTO)

### Isolated Probe Kernels (`/tmp/u8p_base`)

| Kernel | Insns | `vzeroupper` | `shlx`/`shrx` | `vpermq` | `vpack*` | `vptest` |
|---|---:|---:|---:|---:|---:|---:|
| `u8p_s0_u16` (shipped K=2 `u16`) | 1,390 | 0 | 15 | 0 | 0 | 0 |
| `u8p_s1_u8_scalar` (K=2 `u8` 2 stores) | 1,232 | 0 | 14 | 0 | 0 | 0 |
| `u8p_s2_u8_pack16` (K=2 `u8` 1×`u16` store) | 1,232 | 0 | 14 | 0 | 0 | 0 |
| `u8p_s3_k3_u16` (K=3 `#1153` `u8->u16` widen) | 1,238 | 0 | 10 | 0 | 0 | 0 |
| `u8p_s4_k3_u8` (K=3 direct `u8` flush) | 1,241 | 0 | 10 | 0 | 0 | 0 |
| `u8p_r0_p1_u16` (shipped `runs` Pass 1) | 208 | 1 | 2 | **6** | **6** | **1** |
| `u8p_r1_p1_u8_noeob` (`u8` Pass 1, implicit EOB) | 212 | 1 | 2 | **0** | **0** | **0** |
| `u8p_r2_p1_u8_eob` (`u8` Pass 1 + EOB scan) | 200 | 1 | 2 | **0** | **0** | 1 |

### Production `ab` Binary (`BASE_DEF` vs `CAND_DEF`, Portable Fat-LTO)

| Symbol | Insns (`base -> cand`) | `vzeroupper` | `shlx`/`shrx` | `vpermq` | `vpack*` | `vptest` |
|---|---:|---:|---:|---:|---:|---:|
| `block::decode_runs_bmi2` | `2,534 -> 2,521` | `0 -> 0` | `30 -> 22` | `0 -> 0` | `0 -> 0` | `0 -> 0` |
| `runs::front_avx2` | `901 -> 893` | `4 -> 4` | `4 -> 4` | **`6 -> 0`** | **`6 -> 0`** | **`1 -> 0`** |

Note: LLVM automatically coalesced `u8p_s1_u8_scalar`'s two adjacent byte writes into the exact same single 16-bit store as `u8p_s2_u8_pack16` (`1,232` vs `1,232` insns), and `decode_runs_bmi2` dropped 8 shift instructions (`30 -> 22`).

---

## 3. Isolated Station Paired Benchmarks (`n = 11`, CPU 0, `CLOCK_THREAD_CPUTIME_ID`)

All arms passed 100% symbol-for-symbol, bit-position, `Runs` record/count, and `ll`/count exactness across all blocks before timing. `cold32` rotates across 32 distinct `MAX_SYMS` pools (matching 8W parallel `free_syms` RFO misses) with a 48 MiB LLC flush between runs; `warm1` reuses 1 buffer in L2.

### 3a. Station `S` (Scanner A3 `decode_runs_bmi2`)

| Corpus & Mode | Comparison | `BASE` med (ms) | `BASE` gm `[95% CI]` | `JCC-32B` med (ms) | `JCC-32B` gm `[95% CI]` |
|---|---|---:|---|---:|---|
| `wikidata` (`431b`) `warm1` | `S2_u8_pack16 / S0_u16` | `68.39 -> 67.41` | `0.9867 [0.9790, 0.9945]` (`-1.33%`) | `71.70 -> 69.79` | `0.9731 [0.9680, 0.9783]` (`-2.69%`) |
| `wikidata` (`431b`) `cold32` | `S2_u8_pack16 / S0_u16` | `72.83 -> 68.26` | **`0.9222 [0.9044, 0.9404]` (`-7.78%`)** | `76.42 -> 71.54` | **`0.9255 [0.9059, 0.9456]` (`-7.45%`)** |
| `wikidata` (`431b`) `warm1` | `S4_k3_u8 / S3_k3_u16` | `65.32 -> 59.90` | **`0.9153 [0.9044, 0.9263]` (`-8.47%`)** | `67.93 -> 62.82` | **`0.9325 [0.9016, 0.9644]` (`-6.75%`)** |
| `wikidata` (`431b`) `cold32` | `S4_k3_u8 / S3_k3_u16` | `67.43 -> 61.19` | **`0.9081 [0.9013, 0.9150]` (`-9.19%`)** | `71.59 -> 64.84` | **`0.9103 [0.9016, 0.9191]` (`-8.97%`)** |
| `wikidata` (`431b`) `cold32` | `S4_k3_u8 / S0_u16` | `72.83 -> 61.19` | **`0.8286 [0.8138, 0.8436]` (`-17.14%`)** | `76.42 -> 64.84` | **`0.8420 [0.8223, 0.8623]` (`-15.80%`)** |
| `text` (`27b`) `warm1` | `S2_u8_pack16 / S0_u16` | `28.34 -> 27.33` | `0.9480 [0.9058, 0.9923]` (`-5.20%`) | `29.66 -> 28.19` | `0.9478 [0.9293, 0.9667]` (`-5.22%`) |
| `text` (`27b`) `cold32` | `S2_u8_pack16 / S0_u16` | `29.45 -> 27.69` | **`0.9082 [0.8406, 0.9812]` (`-9.18%`)** | `30.46 -> 28.46` | **`0.8860 [0.8129, 0.9657]` (`-11.40%`)** |
| `text` (`27b`) `cold32` | `S4_k3_u8 / S3_k3_u16` | `26.54 -> 24.19` | **`0.9089 [0.9045, 0.9133]` (`-9.11%`)** | `27.65 -> 25.30` | `0.9363 [0.8764, 1.0002]` (`-6.37%`) |
| `text` (`27b`) `cold32` | `S4_k3_u8 / S0_u16` | `29.45 -> 24.19` | **`0.7929 [0.7346, 0.8558]` (`-20.71%`)** | `30.46 -> 25.30` | **`0.8085 [0.7213, 0.9063]` (`-19.15%`)** |
| `random` (`18b`) `warm1` | `S3_k3_u16 / S0_u16` | `67.95 -> 72.56` | `1.0655 [1.0432, 1.0882]` (`+6.55%`) | `70.54 -> 74.04` | `1.0441 [1.0170, 1.0720]` (`+4.41%`) |
| `random` (`18b`) `warm1` | `S4_k3_u8 / S3_k3_u16` | `72.56 -> 65.16` | **`0.9020 [0.8814, 0.9231]` (`-9.80%`)** | `74.04 -> 67.73` | **`0.9125 [0.8960, 0.9294]` (`-8.75%`)** |
| `random` (`18b`) `cold32` | `S4_k3_u8 / S0_u16` | `67.54 -> 64.93` | **`0.9390 [0.8830, 0.9985]` (`-6.10%`)** | `70.80 -> 68.16` | **`0.9420 [0.8906, 0.9963]` (`-5.80%`)** |
| `records` (`22b`) `warm1` | `S3_k3_u16 / S0_u16` | `84.73 -> 91.51` | `1.0905 [1.0748, 1.1064]` (`+9.05%`) | `88.83 -> 95.51` | `1.0725 [1.0648, 1.0802]` (`+7.25%`) |
| `records` (`22b`) `warm1` | `S4_k3_u8 / S3_k3_u16` | `91.51 -> 82.77` | **`0.8981 [0.8867, 0.9096]` (`-10.19%`)** | `95.51 -> 86.95` | **`0.9034 [0.8842, 0.9229]` (`-9.66%`)** |
| `records` (`22b`) `cold32` | `S4_k3_u8 / S0_u16` | `87.73 -> 86.33` | `0.9563 [0.8992, 1.0170]` (`-4.37%`) | `89.53 -> 86.93` | **`0.9313 [0.8714, 0.9954]` (`-6.87%`)** |

**Mechanism:** In `warm1`, `S2_u8` saves `-1.3%..-5.2%` on `K = 2` (fewer shifts + half `memcpy`). In `cold32` (matching 8W parallel pool rotation), halving output RFO cache-line fills jumps the `K = 2` gain to **`-7.5%..-11.4%`**. On top of `K = 3` (`#1153`), removing `flush_u8_patched` (`u8 -> u16` widening) adds a massive **`-6.8%..-10.2%` even in `warm1`** and completely rescues `K = 3` on `random` and `records`.

### 3b. Station `W1` (`runs::front_avx2` Full & Pass-1 Split) and Station `W2` (`mtf_avx2`)

| Station & Corpus | Comparison | `BASE` med (ms) | `BASE` gm `[95% CI]` | `JCC-32B` med (ms) | `JCC-32B` gm `[95% CI]` |
|---|---|---:|---|---:|---|
| `W1 wikidata (383b)` `warm` | `P1_u8_noeob / P1_u16` | `8.00 -> 3.99` | **`0.4973 [0.4852, 0.5098]` (`-50.27%`)** | `8.63 -> 4.25` | **`0.4869 [0.4729, 0.5013]` (`-51.31%`)** |
| `W1 wikidata (383b)` `L3cold` | `P1_u8_noeob / P1_u16` | `8.08 -> 3.87` | **`0.4860 [0.4757, 0.4965]` (`-51.40%`)** | `8.60 -> 4.15` | **`0.4923 [0.4769, 0.5083]` (`-50.77%`)** |
| `W1 wikidata (383b)` `L3cold` | `P1_u8_eob / P1_u16` | `8.08 -> 3.75` | `0.4687 [0.4605, 0.4771]` (`-53.13%`) | `8.60 -> 4.38` | `0.5153 [0.4989, 0.5323]` (`-48.47%`) |
| `W1 wikidata (383b)` `L3cold` | `R1_full_u8 / R0_u16` | `65.40 -> 72.45` | `1.1010 [1.0913, 1.1108]` *(JCC artifact)* | `67.83 -> 62.45` | **`0.9230 [0.9173, 0.9287]` (`-7.70%`)** |
| `W1 text (27b)` `warm` | `P1_u8_noeob / P1_u16` | `2.90 -> 1.41` | **`0.4895 [0.4711, 0.5085]` (`-51.05%`)** | `3.18 -> 1.55` | **`0.4823 [0.4385, 0.5305]` (`-51.77%`)** |
| `W1 text (27b)` `warm` | `R1_full_u8 / R0_u16` | `24.21 -> 26.09` | `1.0786 [1.0630, 1.0945]` *(JCC artifact)* | `25.49 -> 22.94` | **`0.9108 [0.8979, 0.9238]` (`-8.92%`)** |
| `W1 runs (1b)` `warm` | `P1_u8_noeob / P1_u16` | `0.04 -> 0.02` | `0.3935 [0.3402, 0.4552]` (`-60.65%`) | `0.04 -> 0.02` | `0.3448 [0.2489, 0.4777]` (`-65.52%`) |
| `W2 random (18b)` `warm` | `M1_u8_seg / M0_u16` | `201.55 -> 191.61` | **`0.9539 [0.9403, 0.9676]` (`-4.61%`)** | `205.49 -> 188.91` | **`0.9128 [0.8922, 0.9339]` (`-8.72%`)** |
| `W2 random (18b)` `L3cold` | `M1_u8_seg / M0_u16` | `202.28 -> 191.61` | **`0.9430 [0.9354, 0.9506]` (`-5.70%`)** | `206.55 -> 189.45` | **`0.9171 [0.9115, 0.9226]` (`-8.29%`)** |
| `W2 records (22b)` `warm` | `M1_u8_seg / M0_u16` | `232.55 -> 221.43` | **`0.9472 [0.9392, 0.9552]` (`-5.28%`)** | `231.26 -> 215.79` | **`0.9360 [0.9306, 0.9414]` (`-6.40%`)** |

**JCC diagnostic:** In `u8p_r1_u8_noeob`, the `BASE` binary hit a `+10.0 ms` Haswell-EP 32B JCC-erratum boundary crossing inside Pass 2 (`72.45 ms` BASE vs `62.45 ms` JCC-32B); removing the erratum with `JCC-32B` reveals the true `-7.70%` (`-5.38 ms` = `-4.45 ms` Pass 1 + `-0.93 ms` Pass 2 `movzbl` cache savings) win. Note that in the full production `ab` binary (below), `CAND_DEF` did NOT hit that Pass 2 JCC crossing and won **`-4.40%` CPU on `wikidata/mibz2-workers-lazy/1T`** even on unaligned `BASE`. Notice also that `P1_u8_noeob` vs `P1_u8_eob` shows `~95%` of Pass 1's `2×` speedup comes from halving vector loads + removing `vpacksswb`/`vpermq`, with EOB elision adding another `~2.3%` on `JCC-32B`.

---

## 4. Coupled End-to-End Prototype (`ab child`, Portable Fat-LTO, 100% SHA-256 Parity)

Implemented unified `SymBuf` end-to-end across `src/{block,huff,runs,mtf,parallel,stream,read,lib}.rs` (preserving `mtf.rs`'s exact existing `Step` with zero confounding) and ran fresh-process alternating pairs (`n = 9`, `n = 7` on `wikidata-x4`, `n = 5` on tiny guards) with `AB_SHA256=1` verified identical on every cell.

### 4a. Exact `8T` Station Split on `wikidata-x4` (839 MB, `n = 7` Paired)

| Build | Scanner `mibz2-scan` CPU | Workers `wkr_sum` CPU | Total Process CPU | Peak RSS (`VmHWM`) |
|---|---:|---:|---:|---:|
| **Default (`bidi`)** | `0.340 -> 0.310 s` **`0.9030 [0.8838, 0.9226]` (`-9.70%`)** | `3.180 -> 3.080 s` **`0.9723 [0.9543, 0.9906]` (`-2.77%`)** | `3.807 -> 3.681 s` **`0.9667 [0.9536, 0.9800]` (`-3.33%`)** | `64.22 -> 60.32 MiB` (**`-3.89 MiB`**, **`0.9455×`**) |
| **`--features forward-anchor-lanes`** | `0.340 -> 0.310 s` **`0.9153 [0.9056, 0.9251]` (`-8.47%`)** | `2.750 -> 2.640 s` **`0.9636 [0.9546, 0.9727]` (`-3.64%`)** | `3.370 -> 3.226 s` **`0.9615 [0.9540, 0.9691]` (`-3.85%`)** | `65.86 -> 62.32 MiB` (**`-3.55 MiB`**, **`0.9474×`**) |

### 4b. Full Default (`bidi`) E2E Matrix (20 Cells, All 7 Corpora + `wikidata-x4`)

| Cell | `n` | CPU `base -> cand` (s) | CPU gm `[95% CI]` | Wall gm | Peak RSS `base -> cand` (MiB) | RSS Delta | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---:|
| `wikidata/mibz2/1T` | 9 | `0.919 -> 0.910` | `0.9924 [0.9784, 1.0066]` | `1.0100` | `26.86 -> 26.25` | `-0.60` | `0.9776×` |
| `wikidata/mibz2-workers-lazy/1T` | 9 | `0.914 -> 0.877` | **`0.9560 [0.9410, 0.9712]`** | `0.8731` | `24.18 -> 23.50` | `-0.68` | `0.9717×` |
| `wikidata/mibz2/4T` | 9 | `1.031 -> 1.029` | `0.9951 [0.9811, 1.0093]` | `0.9921` | `82.30 -> 82.25` | `-0.05` | `0.9994×` |
| `wikidata/mibz2/8T` | 9 | `0.978 -> 0.955` | **`0.9775 [0.9663, 0.9888]`** | `0.9821` | `50.23 -> 48.48` | **`-1.75`** | **`0.9651×`** |
| `wikidata/mibz2-workers-lazy/8T` | 9 | `0.973 -> 0.950` | **`0.9786 [0.9733, 0.9839]`** | `0.9997` | `50.54 -> 48.30` | **`-2.23`** | **`0.9558×`** |
| `wikidata_x4/mibz2/8T` | 7 | `3.763 -> 3.694` | **`0.9782 [0.9719, 0.9846]`** | `0.9833` | `65.03 -> 60.61` | **`-4.43`** | **`0.9319×`** |
| `text/mibz2/1T` | 9 | `0.236 -> 0.234` | `0.9939 [0.9763, 1.0118]` | `1.0009` | `23.25 -> 22.51` | `-0.74` | `0.9681×` |
| `text/mibz2/4T` | 9 | `0.300 -> 0.287` | **`0.9572 [0.9216, 0.9942]`** | `0.9399` | `65.43 -> 62.09` | **`-3.34`** | **`0.9490×`** |
| `text/mibz2/8T` | 9 | `0.340 -> 0.334` | **`0.9816 [0.9651, 0.9983]`** | `0.9688` | `101.00 -> 95.79` | **`-5.21`** | **`0.9484×`** |
| `random/mibz2/1T` | 9 | `0.406 -> 0.407` | `1.0123 [1.0008, 1.0239]` | `1.0801` | `22.41 -> 21.51` | `-0.90` | `0.9599×` |
| `random/mibz2/4T` | 9 | `0.444 -> 0.441` | **`0.9922 [0.9852, 0.9993]`** | `0.9985` | `56.81 -> 52.71` | **`-4.10`** | **`0.9279×`** |
| `random/mibz2/8T` | 9 | `0.464 -> 0.467` | `1.0066 [0.9987, 1.0147]` | `0.9923` | `82.68 -> 78.13` | **`-4.55`** | **`0.9449×`** |
| `records/mibz2/1T` | 9 | `0.464 -> 0.454` | **`0.9674 [0.9413, 0.9942]`** | `0.9456` | `22.62 -> 21.68` | `-0.95` | `0.9582×` |
| `records/mibz2/4T` | 9 | `0.503 -> 0.500` | **`0.9913 [0.9846, 0.9981]`** | `0.9539` | `61.03 -> 56.89` | **`-4.13`** | **`0.9323×`** |
| `records/mibz2/8T` | 9 | `0.533 -> 0.527` | **`0.9919 [0.9868, 0.9971]`** | `0.9814` | `84.40 -> 81.88` | **`-2.53`** | **`0.9701×`** |
| `runs/mibz2/1T` | 5 | `0.042 -> 0.042` | `1.0170 [0.9919, 1.0428]` | `1.0737` | `33.55 -> 33.32` | `-0.24` | `0.9929×` |
| `runs/mibz2/4T` | 5 | `0.049 -> 0.051` | `1.0368 [0.9511, 1.1303]` | `0.9470` | `31.96 -> 30.71` | `-1.25` | `0.9609×` |
| `onebyte/mibz2/1T` | 5 | `0.069 -> 0.068` | `0.9848 [0.9663, 1.0036]` | `1.0122` | `54.00 -> 54.05` | `+0.05` | `1.0010×` |
| `pattern/mibz2/1T` | 5 | `0.188 -> 0.184` | **`0.9811 [0.9668, 0.9955]`** | `0.9076` | `14.98 -> 15.04` | `+0.06` | `1.0042×` |
| `pattern/mibz2/4T` | 5 | `0.211 -> 0.204` | `0.9846 [0.9094, 1.0660]` | `0.9343` | `37.87 -> 36.70` | `-1.18` | `0.9690×` |

### 4c. `--features forward-anchor-lanes` E2E Matrix (12 Cells)

| Cell | `n` | CPU `base -> cand` (s) | CPU gm `[95% CI]` | Wall gm | Peak RSS `base -> cand` (MiB) | RSS Delta | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---:|
| `wikidata/mibz2/1T` | 9 | `0.976 -> 0.975` | `1.0059 [0.9873, 1.0250]` | `1.0590` | `26.80 -> 26.46` | `-0.34` | `0.9873×` |
| `wikidata/mibz2-workers-lazy/1T` | 9 | `0.828 -> 0.808` | `0.9783 [0.9545, 1.0026]` | `0.9889` | `24.25 -> 23.68` | `-0.57` | `0.9763×` |
| `wikidata/mibz2/4T` | 9 | `1.078 -> 1.086` | `1.0040 [0.9924, 1.0157]` | `1.0703` | `79.34 -> 79.90` | `+0.56` | `1.0070×` |
| `wikidata/mibz2/8T` | 9 | `0.861 -> 0.833` | **`0.9646 [0.9572, 0.9721]`** | **`0.9416`** | `52.68 -> 49.17` | **`-3.51`** | **`0.9333×`** |
| `wikidata/mibz2-workers-lazy/8T` | 9 | `0.863 -> 0.835` | **`0.9700 [0.9536, 0.9866]`** | `0.9715` | `52.88 -> 49.48` | **`-3.40`** | **`0.9357×`** |
| `wikidata_x4/mibz2/8T` | 7 | `3.349 -> 3.190` | **`0.9563 [0.9477, 0.9650]`** | **`0.9514`** | `65.56 -> 62.28` | **`-3.29`** | **`0.9499×`** |
| `text/mibz2/1T` | 9 | `0.263 -> 0.244` | `0.9173 [0.8198, 1.0264]` | `0.9365` | `23.14 -> 22.82` | `-0.32` | `0.9863×` |
| `text/mibz2/8T` | 9 | `0.339 -> 0.329` | `0.9105 [0.7911, 1.0479]` | `0.9399` | `101.27 -> 96.07` | **`-5.20`** | **`0.9487×`** |
| `random/mibz2/1T` | 9 | `0.403 -> 0.379` | **`0.9403 [0.9304, 0.9504]`** | **`0.8932`** | `22.39 -> 21.63` | `-0.76` | `0.9660×` |
| `random/mibz2/8T` | 9 | `0.465 -> 0.435` | **`0.9385 [0.9290, 0.9480]`** | **`0.9070`** | `83.01 -> 78.70` | **`-4.31`** | **`0.9481×`** |
| `records/mibz2/1T` | 9 | `0.476 -> 0.450` | **`0.9532 [0.9468, 0.9596]`** | **`0.9049`** | `22.62 -> 21.95` | `-0.66` | `0.9706×` |
| `records/mibz2/8T` | 9 | `0.534 -> 0.511` | **`0.9554 [0.9375, 0.9736]`** | **`0.9335`** | `83.74 -> 80.75` | **`-3.00`** | **`0.9642×`** |

---

## 5. Coupled Physical i9-10900K Pricing *(Derived from Measured VM + i9 PMU Evidence)*

All physical i9 numbers below are **derived** (not yet measured on i9), combining our measured VM deltas with established i9-10900K measurements (`frontier/DECODER.md`, `a7ac01d8`, `1950f77f`, `95edea2a`):

1. **8W L3 working-set relief on Comet Lake 20 MiB L3:**
   - `a7ac01d8` measured `+4.17%` extra 8W scanner cycles from `+6.53 MiB` of `Arena::pool` (`~0.64%`/MiB), and `1950f77f` showed that even after `#1312` bounded slabs dropped 8W wall to `0.1691 s` (`4.961 GB/s`), `99.4%` (`+17.67M` cyc / `+2.35%`) of the remaining 8W scanner contention residual is **still L3-miss stalls**.
   - `SymBuf` removes **`-3.55 to -4.43 MiB`** of *actively ping-ponged* scanner→worker handover memory (far hotter in L3 than cold slab tails!) right out of that 20 MiB L3 — cutting handover traffic on `wikidata-x4` from `190.56 MiB` to `95.28 MiB` (`-95.28 MiB` written by `S` and read by `W`!).
2. **Simultaneous relief of both `S` and `W/8`:**
   - **Scanner `S`:** Measured `-8.47%..-9.70%` in coupled 8T VM (`-7.45%..-7.78%` isolated `cold32` `K = 2`). Conservatively discounting by `0.5×–0.7×` for Comet Lake vs Haswell-EP while accounting for L3-miss relief projects **`-4.5%..-7.5%` off `S` (`-7.5..-13 ms`)** standalone on `K = 2`, and **`-7%..-10%` on top of `#1153` (`K = 3`)**.
   - **Workers `W`:** Measured `-2.77%..-3.64%` coupled 8T (`runs::front_avx2` `-7.70%` JCC-32B, `Pass 1` `-51%`). Using `#1274`'s empirical VM→i9 calibration (`~0.5×` of VM worker delta on `95edea2a`), `SymBuf` projects **`-1.5%..-2.5%` off `W`** (`-2.5..-4.5 ms/w` at 8W; `-18..-30 ms` at 1W).
   - **Combined 8W wall (`wikidata-x4`, post-`#1312` bounded-slab `forward-anchor-lanes` at `169.1 ms`):** Because post-lanes 8W is scanner-bound and both stations drop simultaneously, `SymBuf` projects **`-3.5%..-6.0%` (`-6..-10 ms`, `~5.15–5.27 GB/s`)** standalone, and even more when stacked with `#1153` (`K = 3` A3) where `SymBuf` removes `K = 3`'s `u8 -> u16` widening tax completely.
   - **Off-Wikidata (`text`, `random`, `records`):** `text` is scanner-bound at `k >= 5` on i9 (`5e3f28cc`), so `-5%..-9%` scanner + `-5.2 MiB` 8W RSS pays directly; `random` and `records` gain both `-3.0..-4.6 MiB` 8W RSS and `-4.5%..-6.2%` CPU from segmented `u8 + pos256` `mtf_avx2`.

## 6. Recommendation

Admit **`u8-syms-handover-build`** (P2 `idea`, `zone = "striated"`, zero `needs_louis` via internal `pub(crate) SymBuf` / `CompactSymbols` keeping public `Symbols` semver-compatible). Coordinate sequencing with `#1153` (`scanner-a3-k3-pipe-u8`) since `SymBuf` unlocks an additional `-6.8%..-10.2%` on top of `K = 3` and eliminates `K = 3`'s `random`/`records` widening regression.
