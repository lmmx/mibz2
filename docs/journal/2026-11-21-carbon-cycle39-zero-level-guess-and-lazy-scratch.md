# Carbon Cycle 39 (`1/20`) — Zero-Level-Guess Theorem, Lazy Worker Scratch, & `slice::fill` Header Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Base Reference:** `origin/master` (`53f7ecf935d62ac7896eafaf55c5d249789614ca`)  
**Artifact:** [c39_zero_level_guess_parscan.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle39/c39_zero_level_guess_parscan.rs)

---

## 1. Mathematical & Architectural Moves (`2026-11-10-parallel-scan.md` §6 Obligation #2)

1. **`C39a` — Zero-Level-Guess Theorem (`KEEP`):**
   - **Theorem:** Across the entire `mibz2` decode pipeline (`block::read_item_with_scratch`, `runs::takes_runs`, `runs::front`, `mtf::mtf`, `OpenCells`, `rle::unrle`, `crc::crc32`), a stream's block-size level $L \in \{1, \dots, 9\}$ (`max_len = 100_000 * L`) enters correctness solely via two inequalities:
     1. `orig_ptr <= 10 + 100_000 * L` (else `ErrorKind::BadOrigPtr` before reading `in_use`), and
     2. `n_rows <= 100_000 * L` (else `ErrorKind::BlockTooLong` during RLE2/MTF expansion),
     while `2 * syms.len() <= max_len` in `takes_runs` only selects between two byte-exact, error-exact inverse-BWT routes (`runs` vs `column`).
   - **Construction:** Workers decode every candidate at `bound_level = 9` (`900_000` rows) while passing `hint_max_len = 100_000 * hint_level` to `takes_runs_with_bound`, returning `WorkerDone::Block { end_bit, crc, orig_ptr, n_rows, out }`. When the authoritative chain reaches `at` in a stream of true level $L$, it checks `orig_ptr <= 10 + 100_000 * L` (`BadOrigPtr`) and `n_rows <= 100_000 * L` (`BlockTooLong`) in $O(1)$ and accepts immediately with **zero re-decodes (`redone = 0`) on every valid block**, falling back to an inline `level = L` read only on the single terminal corrupt block of a `level < 9` stream.
   - **Measured Effect:** On the 31-block adversarial false-header stream (`magic_in_map`, `3.00 MB`, level 1 with embedded `"BZh9"` + `BLOCK_MAGIC` inside every block's `in_use` bitmap), `redone` drops from **`28 / 31` (`C37`) to `0 / 31` (`C39`)**, cutting `4W` real wall from `42.07 ms` to **`18.42 ms` (`0.4378×` / `2.28×` faster)**. Verified byte-exact and `Error { kind, block, bit }`-exact across `2,000 / 2,000` mutated multi-stream, multi-level (`1..9`) corrupt files.
2. **`C39b` — Lazy Worker Scratch Initialization (`KEEP`):**
   - Workers hold `Option<ScanScratch>` and `Option<Scratch<OpenCells>>` on stack and allocate `54 KiB + 1.27 MiB` heap scratch only after claiming their first valid candidate `idx < st.cands.len()`, eliminating idle-worker heap zeroing when $W > B$.
3. **One-Line Kill (`C39c`):**
   - **KILLED `C39c` (`block.rs` dynamic `slice::fill` `lzcnt` code-length + `8×` selector batching):** Variable-length `slice::fill` (`memset`) on `z = 1..2` zero-delta runs regressed `wikidata-l1` `1T` from `0.9017×` (`C37` header loop) to `0.9414×` because >80% of canonical code-length delta-0 runs have length $1$–$2$, where `C37`'s branchless single-bit check beats dynamic `memset`.

---

## 2. Same-Binary `1T | 2T | 4W | 8W | 10W | 16W | 20W` A/B + A/A + Explicit Serial Share

### 2.1 `wikidata-l9` (`11.85 MB` $\to$ `209.70 MB`, 431 blocks)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C39` Wall (ms) | `A/A` Wall | **`C39 / Master` Wall** | `Master` CPU (ms) | `C39` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C39` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C39` `CritW` (ms) | **`C39 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `475.63` | `475.64` | `460.17` | `1.0000×` | **`0.9675×`** | `475.54` | `460.05` | `80.58 ms` (`16.9%` / `16.9%`) | `65.56 ms` (`14.3%` / `14.3%`) | `475.54` | `460.05` | **`0.9674×`** |
| `2T` | `262.31` | `264.02` | `247.25` | `1.0065×` | **`0.9426×`** | `514.92` | `490.45` | `89.12 ms` (`17.3%` / `41.9%`) | `3.54 ms` (`0.7%` / `1.5%`) | `212.72` | `243.46` | `1.1445×` |
| `4W` | `276.47` | `270.81` | `259.32` | `0.9795×` | **`0.9380×`** | `539.50` | `510.86` | `88.93 ms` (`16.5%` / `79.1%`) | `3.19 ms` (`0.6%` / `2.5%`) | `112.45` | `126.83` | `1.1279×` |
| `8W` | `287.62` | `284.09` | `270.63` | `0.9877×` | **`0.9409×`** | `554.49` | `529.92` | `87.25 ms` (`15.7%` / **`100.0%`**) | `3.44 ms` (`0.6%` / **`5.2%`**) | `87.25` | `65.81` | **`0.7543×`** |
| `10W` | `297.39` | `294.72` | `273.97` | `0.9910×` | **`0.9212×`** | `572.87` | `535.35` | `88.83 ms` (`15.5%` / **`100.0%`**) | `2.42 ms` (`0.5%` / **`4.5%`**) | `88.83` | `53.29` | **`0.5999×`** |
| `16W` | `297.90` | `298.57` | `286.61` | `1.0022×` | **`0.9621×`** | `580.60` | `562.51` | `88.61 ms` (`15.3%` / **`100.0%`**) | `3.15 ms` (`0.6%` / **`9.0%`**) | `88.61` | `34.96` | **`0.3946×`** |
| `20W` | `308.13` | `305.18` | `299.96` | `0.9904×` | **`0.9735×`** | `607.50` | `590.61` | `91.61 ms` (`15.1%` / **`100.0%`**) | `4.39 ms` (`0.7%` / **`15.0%`**) | `91.61` | `29.31` | **`0.3199×`** |

### 2.2 `wikidata-l1` (`3.22 MB` $\to$ `36.00 MB`, 361 blocks — with `C39a + C39b`)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C39` Wall (ms) | `A/A` Wall | **`C39 / Master` Wall** | `Master` CPU (ms) | `C39` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C39` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C39` `CritW` (ms) | **`C39 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `116.33` | `114.23` | `104.90` | `0.9820×` | **`0.9017×`** | `116.30` | `104.89` | `30.70 ms` (`26.4%` / `26.4%`) | `21.22 ms` (`20.2%` / `20.2%`) | `116.30` | `104.89` | **`0.9019×`** |
| `2T` | `75.06` | `76.06` | `61.91` | `1.0134×` | **`0.8249×`** | `134.68` | `121.10` | `33.71 ms` (`25.0%` / `67.3%`) | `2.62 ms` (`2.2%` / `4.4%`) | `50.12` | `59.20` | `1.1811×` |
| `4W` | `74.09` | `73.72` | `64.33` | `0.9951×` | **`0.8683×`** | `135.23` | `122.85` | `33.20 ms` (`24.6%` / **`100.0%`**) | `1.82 ms` (`1.5%` / **`6.0%`**) | `33.20` | `30.26` | **`0.9113×`** |
| `8W` | `74.48` | `73.93` | `66.17` | `0.9926×` | **`0.8885×`** | `136.73` | `124.95` | `32.73 ms` (`23.9%` / **`100.0%`**) | `1.47 ms` (`1.2%` / **`9.5%`**) | `32.73` | `15.44` | **`0.4719×`** |
| `10W` | `73.00` | `74.51` | `65.19` | `1.0208×` | **`0.8931×`** | `136.27` | `125.50` | `32.42 ms` (`23.8%` / **`100.0%`**) | `1.39 ms` (`1.1%` / **`11.2%`**) | `32.42` | `12.41` | **`0.3828×`** |
| `16W` | `74.79` | `74.62` | `66.28` | `0.9977×` | **`0.8861×`** | `138.67` | `127.88` | `32.94 ms` (`23.8%` / **`100.0%`**) | `1.20 ms` (`0.9%` / **`15.1%`**) | `32.94` | `7.91` | **`0.2403×`** |
| `20W` | `73.06` | `72.17` | `69.45` | `0.9878×` | **`0.9506×`** | `139.65` | `131.35` | `32.92 ms` (`23.6%` / **`100.0%`**) | `1.19 ms` (`0.9%` / **`18.3%`**) | `32.92` | `6.51` | **`0.1978×`** |

### 2.3 `records-l9` (`9.59 MB` $\to$ `9.60 MB`, 11 blocks)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C39` Wall (ms) | `A/A` Wall | **`C39 / Master` Wall** | `Master` CPU (ms) | `C39` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C39` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C39` `CritW` (ms) | **`C39 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `156.79` | `156.63` | `151.28` | `0.9990×` | **`0.9649×`** | `156.78` | `151.12` | `38.49 ms` (`24.6%` / `24.6%`) | `33.52 ms` (`22.2%` / `22.2%`) | `156.78` | `151.12` | **`0.9639×`** |
| `2T` | `89.44` | `88.93` | `83.97` | `0.9943×` | **`0.9389×`** | `167.49` | `160.64` | `38.44 ms` (`23.0%` / `59.7%`) | `0.24 ms` (`0.2%` / `0.3%`) | `64.43` | `80.21` | `1.2449×` |
| `4W` | `93.49` | `89.96` | `89.83` | `0.9622×` | **`0.9609×`** | `171.57` | `170.66` | `38.49 ms` (`22.4%` / **`100.0%`**) | `0.27 ms` (`0.2%` / **`0.6%`**) | `38.49` | `42.60` | `1.1069×` |
| `8W` | `98.63` | `98.48` | `98.13` | `0.9985×` | **`0.9949×`** | `190.57` | `186.50` | `38.53 ms` (`20.2%` / **`100.0%`**) | `0.31 ms` (`0.2%` / **`1.3%`**) | `38.53` | `23.27` | **`0.6040×`** |
| `10W` | `101.19` | `100.44` | `101.24` | `0.9926×` | `1.0005×` | `194.90` | `194.54` | `38.60 ms` (`19.8%` / **`100.0%`**) | `0.32 ms` (`0.2%` / **`1.6%`**) | `38.60` | `19.42` | **`0.5031×`** |
| `16W` | `105.54` | `104.50` | `105.71` | `0.9902×` | `1.0016×` | `199.92` | `206.44` | `38.29 ms` (`19.2%` / **`100.0%`**) | `0.44 ms` (`0.2%` / **`3.4%`**) | `38.29` | `12.87` | **`0.3362×`** |
| `20W` | `102.12` | `104.47` | `105.36` | `1.0230×` | `1.0317×` | `198.14` | `206.15` | `38.17 ms` (`19.3%` / **`100.0%`**) | `0.53 ms` (`0.3%` / **`5.1%`**) | `38.17` | `10.28` | **`0.2694×`** |
