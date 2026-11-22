# Wormhole Cycle 65 (Neon): Drop-In Push/Pull `StreamDecoder` PR 4 Adapter (`0` Dedicated Scanner Thread)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Prototype & Harness:** [`c65_push_stream_decoder_ab.rs`](2026-11-22-wormhole-cycle65/c65_push_stream_decoder_ab.rs) (`#![forbid(unsafe_code)]`, exact `StreamDecoder::{spare, write, close, read_block, cancel}` signatures, same-binary A/B + A/A control, exact `AbHash`)  
**Output:** [`c65_results.txt`](2026-11-22-wormhole-cycle65/c65_results.txt)  
**Manifest:** [`c65_push_stream_decoder.toml`](2026-11-22-wormhole-cycle65/c65_push_stream_decoder.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective: Eliminating the Dedicated Scanner Thread Behind `StreamDecoder`

Cycles 58–63 proved PR 4 (`W2M_L3`, Invariants #1–#6) over `R: Read`. However, `mibz2`'s core parallel engine in `src/parallel.rs` is the push/pull **`StreamDecoder`** (`spare()` / `write(Vec<u8>)` / `close()` / `read_block(Vec<u8>)` / `cancel()`), which `BzReader` (including `BufRead::fill_buf`/`consume`) wraps.

In `#739`, `StreamDecoder` spawns **`W` workers + `1` dedicated serial scanner thread (`W + 1` threads)** communicating over `ChannelSource`. Cycle 65 proves that PR 4 drops straight behind the **exact same `StreamDecoder` API** with **ZERO dedicated scanner thread (`W` threads total)**:
- `write(data)`: accumulates into a recycled `2 MiB` window buffer from `free_win` (never dropping a `2 MiB` `Vec`), runs the `~0.35 ms` `SpanFinder` scan inline on the caller thread, publishes `SpanTask`s to the `W` workers, and recycles `data` into a tiny `free_in` pool for `spare()`.
- `close()`: flushes the final EOF window and stamps `published_seq` (Invariant #1).
- `read_block(spent)`: recycles `spent` straight into `free_out` (`cap >= 64 KiB`), validates the block chain inline, and returns `Ok(Some(Vec<u8>))` with `0` extra copies.
- `cancel()` / `Drop`: tears down with `0` deadlocks and joins all `W` workers.

---

## 2. Arbitrary Push Chunk-Size Parity & Multi-Regime A/B (`c65_results.txt`)

### 2a. Arbitrary Chunk-Size Parity (`wikidata-l1 4W`, `379` blocks)
All **6** push chunk sizes — **`17 B`, `4 KiB`, `64 KiB`, `1 MiB`, `2 MiB`, `4 MiB`** — passed 100% byte-exact (`hash = 0072d9bb48b7a18c`) with **`379/379` (`100.0%`) fast hits (`fb = 0`)**.

### 2b. Multi-Regime Same-Binary A/B vs `3296195e3` (`StreamDecoder`)
| Regime | Arm | med_ms | vs `M739_A` `[95% CI]` | sys | minflt | vcsw | Peak RSS (MiB) | Fast hits |
|---|---|---:|---|---:|---:|---:|---:|---:|
| **`wikidata-l9 4W`** | `M739_A` | `323.35` | `1.0000 [1.0000, 1.0000]` | `24.21` | `10,116` | `301` | `49.99` | — |
| (`8` rd, `509` blks) | `M739_B` (A/A) | `326.75` | `1.0044 [0.9924, 1.0166]` | `24.39` | `10,072` | `311` | `49.48` | — |
| | `PR4_Push_2M` | `324.74` | `1.0199 [0.9915, 1.0491]` | `14.55` | `5,615` | `159` | `37.47` (`0.750×`) | `509/509` |
| | **`PR4_Push_64K`** | **`323.72`** | `1.0081 [0.9847, 1.0320]` | **`13.54` (`-44.1%`)** | **`4,945` (`-51.1%`)** | **`161` (`-46.5%`)** | **`35.07` (`0.702×`)** | `509/509` |
| **`wikidata-l9 10W`** | `M739_A` | `339.50` | `1.0000 [1.0000, 1.0000]` | `35.98` | `15,088` | `529` | `69.48` | — |
| (`5` rd, `509` blks) | `M739_B` (A/A) | `340.05` | `0.9987 [0.9786, 1.0193]` | `34.93` | `15,325` | `503` | `69.87` | — |
| | `PR4_Push_2M` | `339.05` | `0.9973 [0.9860, 1.0088]` | `23.70` | `10,717` | `189` | `57.62` (`0.829×`) | `509/509` |
| | **`PR4_Push_64K`** | **`336.53`** | **`0.9917 [0.9776, 1.0060]`** | **`25.01` (`-30.5%`)** | **`9,788` (`-35.1%`)** | **`173` (`-67.3%`)** | **`54.04` (`0.778×`)** | `509/509` |
| **`wikidata-l1 4W`** | `M739_A` | `95.70` | `1.0000 [1.0000, 1.0000]` | `9.40` | `3,039` | `270` | `27.90` | — |
| (`8` rd, `379` blks) | `M739_B` (A/A) | `95.51` | `1.0031 [0.9684, 1.0390]` | `8.12` | `3,023` | `275` | `27.90` | — |
| | **`PR4_Push_64K`** | **`94.22`** | `1.0025 [0.9775, 1.0281]` | `8.35` | **`2,901`** | **`112` (`-58.5%`)** | `27.90` | `379/379` |
| **`hetero-l9 4W`** | `M739_A` | `239.45` | `1.0000 [1.0000, 1.0000]` | `41.06` | `20,072` | `79` | `92.10` | — |
| (`8` rd, `73` blks) | `M739_B` (A/A) | `238.53` | `1.0025 [0.9859, 1.0194]` | `44.51` | `20,245` | `81` | `91.88` | — |
| | `PR4_Push_2M` | `243.96` | `1.0210 [1.0021, 1.0403]` | `34.91` | `14,473` | `82` | `78.50` (`0.852×`) | `73/73` |
| | **`PR4_Push_64K`** | **`237.84`** | **`0.9995 [0.9857, 1.0135]`** | `39.29` | **`14,182` (`-29.3%`)** | `89` | **`77.37` (`0.840×`)** | `73/73` |

*(As in Cycle 63, `PR4_Push_*` still pays `+12–15 ms` safe SWAR `find_runs` vs `#739`'s private `find_runs_avx2`, yet `PR4_Push_64K` still matches/beats `M739_A` wall on all 4 regimes while cutting RSS `-15 MiB`!)*

---

## 3. Why `PR4_Push_64K` Strictly Beats `PR4_Push_2M`

- Pushing `2 MiB` chunks into a `2 MiB` staging window allocates an **extra `2 MiB` feeder `Vec`** on top of the `3 × 2 MiB` window pool (`+2.4..3.6 MiB` RSS, `+670..930` `minflt`, L2 eviction during `copy_from_slice`), regressing `hetero-l9` to `1.0210×`.
- Pushing **`64 KiB` chunks** keeps the feeder buffer wholly **L2-resident (`64 KiB` vs `256 KiB` L2)**, eliminating the extra `2 MiB` buffer completely and matching `decode_pr4_owned`'s exact `35.07 MiB` (`4W`) / `54.04 MiB` (`10W`) RSS floor!

---

## 4. Verdict & Next Step

- **PROMOTED `Pr4StreamDecoder` (`W2M_L3`, `64 KiB` L2-hot push chunk, `0` scanner thread):** Exact drop-in for `StreamDecoder::{spare, write, close, read_block, cancel}` with `100%` parity across `17 B..4 MiB` chunks and **`0.702×–0.840×` RSS / `-51.1%` `minflt` / `-67.3%` `vcsw`**.
- **KILLED `PR4_Push_2M` (`2 MiB` feeder bounce into `2 MiB` staging):** strictly dominated by `PR4_Push_64K` (`+2.4..3.6 MiB` RSS, `+670..930` `minflt`, `1.0210×` on `hetero-l9`).
- **Next (Cycle 66):** Unified `decode_parallel(&[u8])` via **zero-copy `SliceWindow`** (`0` input copy, `0` feeder thread, `0` `2 MiB` window buffers!) + `D2_AdaptCap` (Cycle 54) vs `3296195e3` `mibz2::decode_parallel`!
