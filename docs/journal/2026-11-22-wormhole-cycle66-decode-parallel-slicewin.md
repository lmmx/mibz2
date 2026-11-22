# Wormhole Cycle 66 (Neon): Unified `decode_parallel(&[u8])` via Zero-Copy PR 4 `SliceWindow`

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739` `mibz2::decode_parallel`)  
**Harness:** [`c66_decode_parallel_slicewin_ab.rs`](2026-11-22-wormhole-cycle66/c66_decode_parallel_slicewin_ab.rs) (`#![forbid(unsafe_code)]`, generic `decode_slice_pr4<I: InverseBwt>`, same-binary A/B + A/A control, fresh-process child OS counters, exact `CRC32` parity)  
**Outputs:** [`c66_results.txt`](2026-11-22-wormhole-cycle66/c66_results.txt) (exact `Cells`/`OpenCells` policy) & [`c66_opencells_4w_results.txt`](2026-11-22-wormhole-cycle66/c66_opencells_4w_results.txt) (`OpenCells` at `4W`)  
**Manifest:** [`c66_decode_parallel_slicewin.toml`](2026-11-22-wormhole-cycle66/c66_decode_parallel_slicewin.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective

On `3296195e3` (`src/lib.rs:667`), `mibz2::decode_parallel(data: &[u8], threads)` still spawns a feeder thread that copies `4 MiB` chunks of the already-in-memory `data: &[u8]` into `StreamDecoder::write` plus a dedicated serial scanner thread.

Cycle 66 implements **PR 4 `SliceWindow<I: InverseBwt>`** directly over `&[u8]`:
- **`0 B` input copies, `0` window buffers, `0` `Arc<WindowBuf>`, `0` feeder thread, `0` dedicated scanner thread.**
- The calling thread lazily scans `data` in **`2 MiB` slice strides** (`pump_scan`, waking all `W` workers after `~0.35 ms` instead of waiting for a full-file scan!), while `W` scoped workers read straight from `Bits::at(data, abs_bit)`, run Owned Run-Route (`0 B s.ll`) + `ZeroShift52`, and deposit into the bounded `12 MiB` `ReadyRing`.
- Compares drop-in `DP_SliceWin` (`Result<Vec<u8>>`), `DP_Adapt` (`SliceWindow` + Cycle 54 `D2_AdaptCap`), and `DP_Chunks` (zero-copy `Vec<Vec<u8>>`) against `M739_DP_A`/`M739_DP_B`.

---

## 2. Headline Results: Exact Kernel-Policy Parity (`c66_results.txt`: `<8W => Cells`, `>=8W => OpenCells`)

All arms passed 100% exact `CRC32` & byte-length parity (`509/509`, `379/379`, `12/12` — `100.0%` fast hits, `fb = 0`).

| Regime | Arm | med_ms | vs `M739_DP` `[95% CI]` | sys | minflt | vcsw | Peak RSS (MiB) |
|---|---|---:|---|---:|---:|---:|---:|
| **`wikidata-l9 4W`** (`Cells`) | `M739_DP_A` | `378.93` | `1.0000 [1.0000, 1.0000]` | `134.72` | `70,265` | `331` | `273.89` |
| (`8` rd, `211 MB`, `15.3×`) | `M739_DP_B` (A/A) | `377.95` | `0.9959 [0.9815, 1.0105]` | `135.70` | `70,158` | `335` | `274.90` |
| | `DP_SliceWin` | `383.74` | `1.0185 [0.9950, 1.0425]` | **`109.60` (`-18.6%`)** | **`64,756` (`-7.8%`)** | **`172` (`-48.0%`)** | **`260.23` (`-13.7 MiB`)** |
| | `DP_Adapt` | `384.29` | `1.0231 [0.9956, 1.0513]` | `119.06` | `64,677` | `169` | **`258.48` (`-15.4 MiB`)** |
| | `DP_Chunks` | `381.56` | `1.0008 [0.9844, 1.0174]` | `125.08` | **`62,832` (`-10.6%`)** | **`163` (`-50.8%`)** | **`256.64` (`-17.3 MiB`)** |
| *(Bonus: `OpenCells` `4W`)* | **`DP_SliceWin<Open>`** | **`354.13`** | **`0.9202 [0.8963, 0.9447]`** | **`102.15` (`-25.1%`)** | **`55,304` (`-21.5%`)** | **`139` (`-58.0%`)** | **`226.61` (`-46.0 MiB`)** |
| *(Bonus: `OpenCells` `4W`)* | **`DP_Chunks<Open>`** | **`350.59`** | **`0.9185 [0.9079, 0.9292]`** | `124.71` | **`53,365` (`-24.2%`)** | `166` | **`224.10` (`-48.5 MiB`)** |
| **`wikidata-l9 10W`** (`Open`) | `M739_DP_A` (`7W` trim) | `367.70` | `1.0000 [1.0000, 1.0000]` | `139.47` | `67,596` | `559` | `265.28` |
| (`5` rd, `211 MB`, `15.3×`) | `M739_DP_B` (A/A) | `367.50` | `1.0082 [0.9840, 1.0331]` | `138.15` | `66,704` | `572` | `264.98` |
| | `DP_SliceWin` (`10W`) | `371.71` | `1.0244 [0.9968, 1.0527]` | **`113.33` (`-18.7%`)** | **`60,845` (`-10.0%`)** | `416` | **`240.76` (`-24.5 MiB`)** |
| | **`DP_Chunks`** (`10W`) | **`356.30`** | **`0.9792 [0.9526, 1.0066]`** | `125.54` | **`55,701` (`-17.6%`)** | **`145` (`-74.1%`)** | **`231.64` (`-33.6 MiB`)** |
| **`wikidata-l1 4W`** (`Cells`) | `M739_DP_A` | `86.84` | `1.0000 [1.0000, 1.0000]` | `27.18` | `13,509` | `334` | `56.10` |
| (`8` rd, `38 MB`, `9.6×`) | `M739_DP_B` (A/A) | `86.73` | `1.0040 [0.9809, 1.0278]` | `25.18` | `13,580` | `344` | `55.87` |
| | `DP_SliceWin` | `85.81` | `0.9898 [0.9759, 1.0038]` | `22.96` | `12,397` | `141` | `51.33` (`-4.8 MiB`) |
| | `DP_Adapt` | `86.47` | `0.9910 [0.9689, 1.0137]` | `21.77` | `12,499` | `142` | `51.92` |
| | **`DP_Chunks`** | **`82.10`** | **`0.9464 [0.9258, 0.9675]`** | **`21.01` (`-22.7%`)** | **`12,112` (`-10.3%`)** | **`102` (`-69.5%`)** | **`50.96` (`-5.1 MiB`)** |
| **`random-l9 4W`** (`Cells`) | `M739_DP_A` | `114.43` | `1.0000 [1.0000, 1.0000]` | `30.89` | `16,252` | `36` | `70.40` |
| (`8` rd, `11 MB`, `1.0×`) | `M739_DP_B` (A/A) | `115.66` | `1.0194 [0.9765, 1.0642]` | `31.88` | `16,263` | `39` | `68.80` |
| | **`DP_SliceWin`** | **`105.90`** | **`0.9354 [0.9166, 0.9546]`** | `26.51` | `12,776` | **`19` (`-47.2%`)** | **`58.50` (`-11.9 MiB`)** |
| | **`DP_Adapt`** | **`106.98`** | **`0.9488 [0.9217, 0.9768]`** | `24.68` | `12,818` | `20` | `59.14` |
| | **`DP_Chunks`** | **`103.59`** | **`0.9172 [0.8988, 0.9359]`** | **`21.43` (`-30.6%`)** | **`11,740` (`-27.8%`)** | `19` | **`56.01` (`-14.4 MiB`)** |

---

## 3. Verdict & Next Step

- **PROMOTED `DP_SliceWin<I>` & `DP_Chunks<I>` (Zero-Copy PR 4 `SliceWindow`):** Eliminates input copy, window buffers, feeder thread, and scanner thread in `decode_parallel(&[u8])`, cutting peak RSS **`-12..-34 MiB`** (`-48.5 MiB` with `OpenCells` `4W`), `vcsw` **`-48%..-74%`**, and winning `random-l9` **`0.9354×` / `0.9172×`**, `wikidata-l1` **`0.9464×`**, and `wikidata-l9 10W` **`0.9792×`** (even while paying safe SWAR `find_runs` vs `#739` AVX2!).
- **Honest Note on `D2_AdaptCap` inside `SliceWindow`:** Once `SliceWindow` removes the feeder/scanner copies, `DP_Adapt` vs `DP_SliceWin` is neutral on `wikidata-l9` (`mremap` absorbs `Vec` growth cheaply on Linux) and only saves `~1.8 ms` `sys` on `random-l9` — keep `D2_AdaptCap` strictly for non-Linux / RSS-guard parity rather than expecting wall speedup on Linux.
- **Next (Cycle 67):** Test **within-window LPT straggler scheduling** (`cands[i+1].rel_bit - cands[i].rel_bit` compressed-span proxy inside each `2 MiB` / `4 MiB` PR 4 window!) vs FIFO on `wikidata-l9` & `hetero-l9` at `4W` and `10W` — keep or kill honestly!
