# Wormhole Cycle 64 (Neon): Post-`#739 + MechBR + PR4_Own_ZS52` Residual Stage Profile & `10T` i9 Floor Roadmap

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Profiler:** [`c64_residual_profile.rs`](2026-11-22-wormhole-cycle64/c64_residual_profile.rs) (`#![forbid(unsafe_code)]`, 5–10 round average per corpus, exact block CRC verification on all `973/973` blocks)  
**Output:** [`c64_results.txt`](2026-11-22-wormhole-cycle64/c64_results.txt)  
**Manifest:** [`c64_residual_profile.toml`](2026-11-22-wormhole-cycle64/c64_residual_profile.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective

With all six Round 5 plateau walls (`W0` `trimmed_here`, `W1` serial scan, `W2` allocator/page-faults, `W3` `10.8 B/B` DRAM, `W4` ready window L3 spill, `W5` broadcast locks) now dismantled by `#739` + Mechanisms' block reader (`#745`) + Cycle 56 i9 grid + `PR4_Own_ZS52`, Cycle 64 measures the **exact remaining per-stage CPU shares** across all 4 corpora and projects the anchored `10T` i9-10900K roadmap all the way down toward the DDR4 bandwidth floor.

---

## 2. Measured `1T` Residual Stage Breakdown (`c64_results.txt`)

| Stage | Parallel? | `wikidata-l9` (`509` blks, `211 MB`) | `wikidata-l1` (`379` blks, `37.8 MB`) | `hetero-l9` (`73` blks, `53.8 MB`) | `random-l9` (`12` blks, `10.7 MB`) |
|---|---|---:|---:|---:|---:|
| **`S0 AnchorScan`** (`FastAnchorFinder`) | Serial | `4.66 ms` (`0.8%`) | `1.30 ms` (`0.8%`) | `7.02 ms` (`1.8%`) | `3.49 ms` (`2.0%`) |
| **`S1 read_item`** (`block::read_item`) | **Parallel** | **`89.37 ms` (`16.2%`)** | **`34.42 ms` (`21.9%`)** | **`86.46 ms` (`21.7%`)** | **`41.42 ms` (`23.9%`)** |
| **`S2 runs::front`** (`MTF + RLE2`) | **Parallel** | `47.31 ms` (`8.6%`) | `13.29 ms` (`8.5%`) | `8.94 ms` (`2.2%`) | `0.00 ms` (`0.0%`) |
| **`S3 OpenCells`** (owned `out`, `0 B s.ll`) | **Parallel** | **`343.17 ms` (`62.2%`)** | **`96.40 ms` (`61.3%`)** | `67.71 ms` (`17.0%`) | `0.00 ms` (`0.0%`) |
| **`S4 ZeroShift52 + CRC`** (`SWAR + CLMUL`) | **Parallel** | `67.54 ms` (`12.2%`)* | `11.86 ms` (`7.5%`) | `11.70 ms` (`2.9%`) | `0.00 ms` (`0.0%`) |
| **`S3+4 ColRoute`** (`LazyCells` + `unrle`) | **Parallel** | `0.00 ms` (`0.0%`) | `0.00 ms` (`0.0%`) | **`215.86 ms` (`54.3%`)** | **`128.19 ms` (`74.1%`)** |
| **Total `1T` CPU** | — | **`552.05 ms`** | **`157.27 ms`** | **`397.70 ms`** | **`173.10 ms`** |
| **Amdahl Parallelizable Share** | — | **`99.16%`** | **`99.17%`** | **`98.23%`** | **`97.98%`** |

*\*Note: `S4` includes ~`15 ms` for pure-safe-Rust `find_runs_swar`; behind `#739`'s existing `find_runs_avx2` in `src/rle.rs`, `S4` drops to **`~52 ms` (`~9.7%`)**. Also verified across all `973/973` blocks: **`0` blocks exceeded `n + n/32 + 64`** (`grew >n/32 = 0, >n/16 = 0`) — `#739`'s `n + n/16 + 64` pre-sizing never reallocates during `ZeroShift52`.*

---

## 3. Anchored `10T` i9-10900K Progression Toward the DDR4 Floor (`210 MB` Wikidata)

| Milestone Stack | Active Topology (`10C/20T` i9) | Pred / Meas `10T` Wall | `10W` Peak RSS | Binding Stage |
|---|---|---:|---:|---|
| `v0.5.12` (`6a36b0e10`) | `7W/28` + `1` serial scan | `~69.5 ms` *(real i9)* | `~110 MiB` | `W1` serial scan + `W3` `10.8 B/B` DDR4 |
| `#739` (`3296195e3`, PR 1–3) | `7W/28` + `1` serial scan | **`66.9 ms`** *(real i9: `0.963×`)* | `~72 MiB` (`0.65×`) | `W1` serial scan + `W0` `7W` cap |
| `+ Mechanisms #745 Block Reader` | `7W/28` + `1` serial scan | **`62.4 ms`** *(real i9: `0.932×`)* | `~72 MiB` | **`W0` `7W` cap** (`8T = 59.1 ms`) |
| `+ Cycle 56 i9 Grid` (`9W/27`) | `9W` + `1` dedicated `#745` core | **`~53.2 ms`** *(pred)* | `~60 MiB` | `9W` parallel compute (`0%` SMT clash) |
| **`+ PR4_Own_ZS52` (`W2M_L3` + `C38b` + `#745` + `ZeroShift52`)** | **`10W/30` (`99.2%` parallel)** | **`~49.5–51.0 ms`** *(pred)* | **`~55 MiB` (`0.76×` `#739`)** | **`S3 OpenCells` (`62.2%`) + `S1 read_item` (`13%`)** |
| `+ Wormhole R4 WormholeNeonCells` (`0.870×` Open) | `10W/30` (`17.9 MiB` L3) | **`~44.5–46.0 ms`** *(pred)* | `~55 MiB` | `S3` emit (`dep_past_l2`) + `S1` |
| `+ Teamwork BSAC + Eta` (`emit` + `runs::front`) | `10W/30` | **`~39–41 ms`** *(pred)* | `~56 MiB` | Within **`~1.7×`** of `20.1–23.1 ms` DDR4 floor |

---

## 4. Key Conclusion & Next Step

- **Transport & Runtime Saturation Reached:** Once `#739 + MechBR + Cycle 56 Grid + PR4_Own_ZS52` land, **99.2% of total CPU is parallel** and **`S3 OpenCells` (`62.2%`) + `S1 read_item` (`16.2%`) + `S2 runs::front` (`8.6%`)** account for **`87.0%`** of all remaining runtime! No remaining serial transport bottleneck exists above `0.8%`.
- **Next (Cycle 65):** Prove that PR 4 (`W2M_L3`, Invariants #1–#6) works behind the **EXACT existing push/pull `StreamDecoder` API (`write(Vec<u8>)`, `spare()`, `close()`, `read_block(Vec<u8>)`, `cancel()`)** — not only `R: Read` — with a compile-verified safe-Rust adapter and same-binary A/B against `3296195e3`!
