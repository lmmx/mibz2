# Carbon Cycle 56 (18 / 20) — Single-Pass `mtf_checkpoints_scalar`, Outer-Tiled `mtf_scalar`, and 8-Unrolled `runs::cuts`

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base Candidate:** Cycle 55 (`eac38fc6`)
- **Saved Drop-Ins:** `docs/journal/2026-11-22-carbon-cycle56/{mtf.rs,runs.rs}`
- **Safety:** `100%` safe Rust (`#![forbid(unsafe_code)]` harness `c56_mtf_cuts_screen.rs`; `0` new `unsafe` blocks in `src/mtf.rs` and `src/runs.rs`)

---

## 1. Algorithmic Improvements

1. **`c56_mtf_cp_singlepass` (`src/mtf.rs:mtf_checkpoints_scalar` — `0.3605x`, `2.77x` faster):**
   - Previously, `mtf_checkpoints_scalar` ran `mtf_scalar(b, ll, counts)` and then executed a **complete second scalar pass** over all `<= 900 KiB` of `ll` (`for chunk in ll.chunks_exact(CHECKPOINT) { for &c in chunk { cum[c] += 1 } }`) to construct Cooperative `16 KiB` count checkpoints.
   - Replaced with a single-pass outer-`CHECKPOINT` (`16 KiB`) tile loop (`limit = max.min(next)`) that pre-sizes `ll.resize(limit, 0)` once per `16 KiB` tile (removing per-symbol `n >= ll.len()` checks from the inner symbol loop) and snapshots `checkpoints.push(*counts)` inline at each `CHECKPOINT` boundary (splitting runs that cross `CHECKPOINT` boundaries identically to `mtf_checkpoints_avx2`).

2. **`c56_mtf_scalar_outer_tile` (`src/mtf.rs:mtf_scalar` — `0.9625x` / `0.9734x`):**
   - Applied the same `limit = max.min(next)` outer `16 KiB` tile loop to `mtf_scalar`, hoisting `ll.resize` out of the per-symbol loop (`~600,000` checks per `-9` block reduced to `<= 55` tile boundaries) while keeping exact `ll.reserve(b.max_len)` capacity (`trace::tests` `5/5`).

3. **`c56_cuts_unroll8` (`src/runs.rs:cuts` — `0.6068x–0.6421x`, `1.56x–1.65x` faster):**
   - Precomputed the `11` cut thresholds `targets[1..PIECES]` (`u32`, since `total < 9 * 2^24 < 2^28`) once outside the record loop (eliminating `div_ceil(12)` inside the loop), and unrolled `8` records per cut-distance check whenever `work + 2104 < due` (`8 * (255 + 8) = 2104` max work across 8 records).

---

## 2. Same-Binary A/B + A/A Results (`c56_mtf_cuts_screen.rs` & `c48_scalar_rle_ab`)

| Kernel & Corpus | Base (ms) | `A/A` | Candidate (ms) | Ratio (`Cand / Base`) | Verdict |
| :--- | ---: | ---: | ---: | ---: | :---: |
| `mtf_checkpoints_scalar` (`wikidata-l9`, `32` blks, `21` rds) | `49.50` | `0.9978x` | **`17.84`** | **`0.3605x` (`2.77x` faster)** | **ADOPT** |
| `mtf_checkpoints_scalar` (`records-l9`, `11` blks, `21` rds) | `107.63` | `0.9973x` | **`104.94`** | **`0.9751x`** | **ADOPT** |
| `mtf_scalar` (`wikidata-l9`, `32` blks, `21` rds) | `17.67` | `1.0051x` | **`17.01`** | **`0.9625x`** | **ADOPT** |
| `mtf_scalar` (`records-l9`, `11` blks, `21` rds) | `108.80` | `0.9946x` | **`105.90`** | **`0.9734x`** | **ADOPT** |
| `runs::cuts` (`wikidata-l9`, `30` runs blks, `31` rds) | `1.905` | `0.9946x` | **`1.223`** | **`0.6421x` (`1.56x` faster)** | **ADOPT** |
| `runs::cuts` (`wikidata-l1`, `128` runs blks, `31` rds) | `2.048` | `0.9914x` | **`1.243`** | **`0.6068x` (`1.65x` faster)** | **ADOPT** |
| Full `2T` scalar decode (`8` `wikidata-l9` streams vs `master`) | `812.30` | `1.0006x` | **`625.32`** | **`0.7698x` (`1.30x` faster)** | **ADOPT** |

All `tests/mtf_checkpoints.rs` (`3/3`), `runs::tests` (`15/15`), and `trace::tests` (`5/5`) pass with zero warnings.

Next cycle (`Cycle 57`, `19/20`): Screen streaming negative-zone tile CRC in `rle::unrle_in_place` + `u128` 13-position SWAR `find_runs_scalar` (`src/rle.rs`).
