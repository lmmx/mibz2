# Wormhole Cycle 54 (Neon) — `decode_parallel` `SliceSource` Zero-Copy Feeder & Online Adaptive-Ratio `out.reserve` on `3296195e3`

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`, [`src/lib.rs:667-703`](../../src/lib.rs))  
**Artifacts:** [`c54_decode_parallel_ab.rs`](2026-11-22-wormhole-cycle54/c54_decode_parallel_ab.rs), [`c54_results.txt`](2026-11-22-wormhole-cycle54/c54_results.txt), [`c54_decode_parallel.toml`](2026-11-22-wormhole-cycle54/c54_decode_parallel.toml)

---

## 1. Residual Cost in `#739`'s `decode_parallel` (`src/lib.rs:667-703`)

`#739` upgraded `decode_parallel` to use `dec.spare()` + `dec.read_block(spent)`, eliminating the `4 MiB` reader bounce buffer. Two avoidable costs remain for whole-slice `&[u8]` decoding:
1. **Input Feeder Copy (`src/lib.rs:676-684`):** Spawns a feeder thread to copy `data: &[u8]` into `4 MiB` `Vec<u8>` chunks for `ChannelSource` even though `data` is already a contiguous slice (`SliceSource(data)`).
2. **Fixed `4×` Output Pre-Size (`src/lib.rs:673`):** `Vec::with_capacity(data.len() * 4)` causes `2` full `Vec` reallocations/copies (`62 → 125 → 250 MiB`) on `13.5×` Wikidata while over-reserving `4×` on `1.0×` incompressible files.
   - **`D2_AdaptCap` Fix:** Once block `1`'s start bit `bit` arrives (after block `0` finishes), `comp_bytes = (bit - prev_bit) / 8` gives the exact empirical expansion ratio `out.len() / comp_bytes`; a single `reserve_exact` sizes `out` to `1.06×` of the estimated final length — zero guesswork and zero random-file bloat!

---

## 2. Native Safe-Rust Results (`12`-Round Fresh-Process `4W` A/B on `3296195e3`)

| Corpus | Arm | `med_ms` | `vs D739_A [95% CI]` | `user` | `sys` | `minflt` | `rss_MB` | `grow` | Verdict |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| **`wikidata-l9`** (`15.6→209.8 MB`, `13.5×`) | `D739_A` | `388.45` | `1.0000 [1.0000, 1.0000]` | `653.5` | `101.30` | `61,794` | `245.25` | `2` | Baseline |
| | `D739_B` (A/A) | `386.86` | `0.9924 [0.9753, 1.0097]` | `652.3` | `101.95` | `61,563` | `245.39` | `2` | Clean A/A |
| | `D1_SliceSrc` | `382.56` | `0.9948 [0.9758, 1.0142]` | `658.9` | `92.43` | `58,186` | `241.50` | `2` | **KILL alone** |
| | **`D2_AdaptCap`** | **`382.16`** | **`0.9786 [0.9633, 0.9942]`** | `659.6` | **`88.27` (`-12.9%`)** | **`58,215` (`-5.8%`)** | `240.64` | **`1`** | **PROMOTE** |
| | **`D3_Chunks`** | **`369.87`** | **`0.9540 [0.9464, 0.9617]`** | **`619.5`** | `101.50` | **`56,600` (`-8.4%`)** | **`236.84`** | **`0`** | **PROMOTE (additive)** |
| **`random-l9`** (`10.7→10.7 MB`, `1.0×`) | `D739_A` | `115.14` | `1.0000 [1.0000, 1.0000]` | `184.4` | `32.87` | `16,459` | `68.52` | `0` | Baseline |
| | `D739_B` (A/A) | `115.26` | `1.0577 [0.9827, 1.1384]` | `188.3` | `33.05` | `16,257` | `68.57` | `0` | Clean A/A |
| | `D1_SliceSrc` | `111.35` | `0.9880 [0.9582, 1.0188]` | `184.2` | `27.66` | `14,320` | `62.02` | `0` | — |
| | **`D2_AdaptCap`** | `114.08` | `0.9914 [0.9693, 1.0139]` | `185.8` | **`28.37` (`-13.7%`)** | **`14,310` (`-13.1%`)** | **`62.15` (`-9.3%`)** | `1` | **0 bloat verified** |
| | **`D3_Chunks`** | `111.71` | `0.9791 [0.9442, 1.0152]` | `183.3` | **`26.90` (`-18.2%`)** | **`13,011` (`-21.0%`)** | **`58.71` (`-14.3%`)** | `0` | **PROMOTE (additive)** |

---

## 3. Kill / Keep Decisions

- **KILLED `D1_SliceSrc` alone (`0.9948× [0.9758, 1.0142]`):** Removing the `4 MiB` slice copy alone does not clear `95%` CI on `wikidata-l9` (only `-3.6k` `minflt`).
- **PROMOTED `D2_AdaptCap` (`0.9786× [0.9633, 0.9942]`, `sys -12.9%`, `grow 2→1` on Wikidata; `minflt -13.1%`, `RSS -9.3%` on `random-l9`):** Best API-identical `decode_parallel -> Result<Vec<u8>>` refinement, naturally unified with PR 4's `SliceWindow` mode in Cycle 66.
- **PROMOTED `D3_Chunks` (`0.9540× [0.9464, 0.9617]`, `-34 ms` user, `-8.4 MiB` RSS on Wikidata; `-21%` `minflt` / `-14.3%` RSS on `random-l9`):** Best additive zero-copy `decode_parallel_chunks` API.
