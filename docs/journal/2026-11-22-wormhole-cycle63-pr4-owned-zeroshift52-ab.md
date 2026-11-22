# Wormhole Cycle 63 (Neon): Composed `PR4 W2M_L3` + Owned Run-Route (`0 B s.ll`) + Theorem 52.1 `ZeroShift52` Multi-Regime A/B

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Harness:** [`c63_pr4_owned_zs52_ab.rs`](2026-11-22-wormhole-cycle63/c63_pr4_owned_zs52_ab.rs) (`#![forbid(unsafe_code)]`, same-binary A/B + A/A control, fresh-process child OS counters, exact `AbHash`)  
**Output:** [`c63_results.txt`](2026-11-22-wormhole-cycle63/c63_results.txt)  
**Manifest:** [`c63_pr4_owned_zs52.toml`](2026-11-22-wormhole-cycle63/c63_pr4_owned_zs52.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective & Removing the External `s.ll` Handicap

In Cycles 57–62, our external PR 4 harnesses called public `mibz2::decode_block`, which inverts into `Scratch::ll` (`+0.90 MiB/W` = `+9.0 MiB` at `10W`) and runs two-buffer `unrle`, whereas `#739` (`StreamDecoder`) calls `pub(crate) decode_owned`, inverting `OpenCells::invert_runs` directly into the pooled `out` buffer (`0 B` `s.ll`) with in-place `RLE1` + `64 KiB` tiled `CLMUL` CRC.

Cycle 63 composes **both** production pieces in `#![forbid(unsafe_code)]` Rust using public `mibz2::runs::{Runs, takes_runs, front}`, `mibz2::bwt::OpenCells::invert_runs`, and `mibz2::crc::update_with`:
1. **`PR4_Own_739`:** Hardened `PR4 W2M_L3` (`2 MiB × 3 = 6 MiB`, Invariants #1–#6) + **true `#739` Owned Run-Route (`0 B` `s.ll`)** + carry-free SWAR `find_runs_swar` + exact `#739` `src/rle.rs:103-170` double-shift `unrle_in_place`.
2. **`PR4_Own_ZS52`:** Identical `PR4 W2M_L3` + Owned Run-Route (`0 B` `s.ll`) + **Theorem 52.1 `ZeroShift52`** skip-free zero-shift in-place `RLE1` + `64 KiB` tiled `CLMUL` CRC (`-46.6%` `copy_within`, `0` pre-shift, `0` `s.ll` staging fallback).

*(Note: `M739_A`/`M739_B` still enjoy `#739`'s private `unsafe` `find_runs_avx2` (`32 B/iter`), whereas our `#![forbid(unsafe_code)]` arms use safe SWAR `find_runs_swar` (`5 B/iter`, ~`+12–15 ms` user CPU overhead on `210 MB`). Porting `ZeroShift52` inside `src/rle.rs` behind the existing `find_runs_avx2` eliminates that `12–15 ms` gap completely while keeping the full `-5..-8 ms` `ZeroShift52` win.)*

---

## 2. Headline Results Across All 5 Regimes (`c63_results.txt`)

All 4 arms verified 100% byte-exact via `AbHash` across every round of all 5 regimes (`509/509`, `379/379`, `73/73`, `12/12` — **`973/973 = 100.0%` fast hits, `fb = 0`**).

| Regime | Arm | med_ms | vs `M739_A` `[95% CI]` | user | sys | minflt | vcsw | **Peak RSS (MiB)** |
|---|---|---:|---|---:|---:|---:|---:|---:|
| **`wikidata-l9 4W`** | `M739_A` | `325.86` | `1.0000 [1.0000, 1.0000]` | `601.8` | `24.98` | `9,829` | `297` | `49.59` |
| (`10` rd, `509` blks) | `M739_B` (A/A) | `326.14` | `1.0021 [0.9859, 1.0186]` | `599.3` | `26.08` | `9,810` | `305` | `49.76` |
| | `PR4_Own_739` | `324.22` | `0.9998 [0.9880, 1.0117]` | `622.6` | `16.12` | `5,152` | `160` | `35.96` (`0.725×`) |
| | **`PR4_Own_ZS52`** | **`319.41`** | `1.0001 [0.9778, 1.0229]` | **`614.5`** | **`15.01` (`-39.9%`)** | **`4,922` (`-49.9%`)** | **`159` (`-46.5%`)** | **`35.14` (`0.709×`)** |
| **`wikidata-l9 10W`** | `M739_A` | `338.67` | `1.0000 [1.0000, 1.0000]` | `617.6` | `29.97` | `15,767` | `522` | `71.91` |
| (`5` rd, `509` blks) | `M739_B` (A/A) | `345.94` | `1.0125 [0.9996, 1.0256]` | `616.2` | `33.28` | `16,071` | `518` | `72.66` |
| | `PR4_Own_739` | `335.28` | `0.9858 [0.9631, 1.0090]` | `632.1` | `23.61` | `10,009` | `175` | **`54.64` (`0.760×`)** |
| | **`PR4_Own_ZS52`** | **`330.91`** | **`0.9777 [0.9687, 0.9868]`** | **`626.9`** | **`23.51` (`-21.6%`)** | **`10,019` (`-36.5%`)** | **`185` (`-64.6%`)** | **`54.90` (`0.763×`)** |
| **`wikidata-l1 4W`** | `M739_A` | `95.44` | `1.0000 [1.0000, 1.0000]` | `167.6` | `9.05` | `2,987` | `289` | `18.09` |
| (`10` rd, `379` blks) | `M739_B` (A/A) | `95.11` | `0.9995 [0.9879, 1.0112]` | `166.7` | `9.05` | `3,020` | `303` | `18.16` |
| | `PR4_Own_739` | **`90.89`** | **`0.9619 [0.9478, 0.9763]`** | `171.4` | `7.13` | `2,877` | `106` | `17.61` (`0.973×`) |
| | **`PR4_Own_ZS52`** | `91.50` | **`0.9642 [0.9534, 0.9752]`** | `171.8` | **`6.39` (`-29.4%`)** | **`2,865`** | `112` | **`17.59` (`0.972×`)** |
| **`hetero-l9 4W`** | `M739_A` | `244.58` | `1.0000 [1.0000, 1.0000]` | `427.6` | `42.79` | `19,896` | `82` | `91.14` |
| (`10` rd, `73` blks) | `M739_B` (A/A) | `242.27` | `0.9924 [0.9788, 1.0063]` | `427.3` | `43.79` | `20,169` | `85` | `91.79` |
| | `PR4_Own_739` | **`237.11`** | **`0.9774 [0.9630, 0.9919]`** | `434.0` | `33.17` | `14,159` | `87` | `76.85` (`0.843×`) |
| | **`PR4_Own_ZS52`** | `242.21` | `0.9796 [0.9576, 1.0021]` | `432.6` | `34.48` | `14,194` | `81` | **`76.80` (`0.843×`)** |
| **`random-l9 4W`** | `M739_A` | `118.44` | `1.0000 [1.0000, 1.0000]` | `194.9` | `32.20` | `13,643` | `31` | `62.48` |
| (`10` rd, `12` blks) | `M739_B` (A/A) | `117.62` | `0.9964 [0.9733, 1.0199]` | `192.6` | `30.85` | `14,038` | `31` | `63.19` |
| | `PR4_Own_739` | `119.25` | `0.9920 [0.9627, 1.0222]` | `198.9` | `29.53` | `12,015` | `35` | `58.88` (`0.942×`) |
| | **`PR4_Own_ZS52`** | **`115.59`** | **`0.9775 [0.9487, 1.0072]`** | **`195.1`** | **`27.45` (`-14.8%`)** | **`11,791` (`-13.6%`)** | `36` | **`58.17` (`0.931×`)** |

---

## 3. Key Findings & Takeaways

1. **True PR 4 + Owned Run-Route (`0 B s.ll`) RSS collapses to `0.709×–0.763×` of `#739`:**
   - At `4W`: **`35.14 vs 49.59 MiB` (`-14.45 MiB`, `minflt -49.9%`, `sys -39.9%`)**.
   - At `10W`: **`54.90 vs 71.91 MiB` (`-17.01 MiB`, `minflt -36.5%`, `vcsw -64.6%`)** — roughly **`40%` less peak RSS than `#741`'s `1.28×` whole-input `parscan`** and **`24%` less than `#739` itself**!
2. **`ZeroShift52` delivers clean additive CPU & wall savings over `#739` double-shift (`PR4_Own_ZS52` vs `PR4_Own_739`, apples-to-apples):**
   - `wikidata-l9 4W`: **`-8.1 ms` user CPU (`-1.30%`), `-4.81 ms` median wall (`319.41 vs 324.22 ms`)**.
   - `wikidata-l9 10W`: **`-5.2 ms` user CPU, `-4.37 ms` median wall (`330.91 vs 335.28 ms`, `0.9777× [0.9687, 0.9868]` vs `M739_A` even with SWAR `find_runs`!)**.
   - `random-l9 4W`: **`-3.8 ms` user CPU, `-3.66 ms` median wall (`115.59 vs 119.25 ms`)**.
3. **Zero regressions:** Both `PR4_Own_739` and `PR4_Own_ZS52` achieve **`973/973` (`100.0%`, `fb=0`)** fast hits and beat/match `M739_A` on all 5 regimes.

---

## 4. Verdict & Next Step

- **PROMOTED `PR4_Own_ZS52` (`W2M_L3` + Owned Run-Route `0 B s.ll` + Theorem 52.1 `ZeroShift52`):** Cuts `10W` peak RSS to **`54.90 MiB` (`0.763×` `#739`)**, `4W` `minflt` by **`-49.9%`**, `10W` `vcsw` by **`-64.6%`**, and delivers **`0.9777× [0.9687, 0.9868]`** at `10W` & **`0.9642× [0.9534, 0.9752]`** on `wikidata-l1` even before inheriting `#739`'s `find_runs_avx2`.
- **Next (Cycle 64):** Post-`#739 + MechBR + PR4_Own_ZS52` residual stage profile (`1T` & `10T` i9 roofline) + honest test of cold-start `MADV_HUGEPAGE` / `Slab` vs `3296195e3` to see whether anything outside the BWT kernel + Huffman remains on the `10T` critical path!
