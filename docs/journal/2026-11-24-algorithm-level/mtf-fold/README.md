# MTF-into-construction fold census (A5 + A6a)

Claim [`mtf-fold-census`](../../../../frontier/claims/mtf-fold-census.toml) — smooth-zone characterisation before theory. Zero changes to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero changes to production defaults.

**Harness:** [`examples/mtf_fold_census.rs`](../../../../examples/mtf_fold_census.rs), running actual production `mibz2::runs::front` (AVX2) and `mibz2::runs::front_scalar` alongside an exact safe mirror of production A6a (`src/open.rs` `maximal` + `Pass::take` + `edges` strip packing and `src/open/outq.rs` `OutQ16`), structural ablations of A6a, Pass-2 isolated MTF+RLE2 ablations, and direct folded A5+A6a prototypes verified **100% bit-for-bit A6a state-identical** to production on every admitted block.

**Machine, method & caveats:**
- **Hardware:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2, BMI1/BMI2, LZCNT, POPCNT, 32 KiB L1d, 256 KiB L2, 45 MiB L3), single-threaded, reusable `out` and scratch buffers matching production worker steady state.
- **Method:** Interleaved repeats with alternating arm traversal direction (`7` repeats on `wikidata`, `runs`, `text`, `pattern`, `records`; `5` repeats on `wikidata ×4`) after a full warm-up and exactness pass. Every folded prototype is verified bit-for-bit identical against production A5+A6a on `n`, `First` (`thick_rows[256]`, `thin[256]`, `thick[256]`, `m`, `mark`, `runs`), `ns`, `edges[..3*ns]`, `spill.cuts`, and all 256 drained `OutQ16` rank streams on every block.
- **Cross-machine i9-10900K anchors (`wikidata ×4`, SHA-256 `0b820d14...ce08e`):**
  - `character-stage1/report.txt`: at `k = 1`, worker busy `1,280.1 ms`, **A5 = `132.4 ms`** (`39.8%` of A5+A6a), **A6a = `200.3 ms`** (`60.2%` of A5+A6a), **combined `332.7 ms`** (`26.0%` of 1w worker busy). At `k = 8`, worker busy `184.7 ms/w`, **A5 = `19.3 ms/w`**, **A6a = `28.2 ms/w`**, **combined `47.5 ms/w`** (`25.7%` of 8w worker busy).
  - `ideal-parts/results/full/README.md`: M alone ideal = `0.925` (`7.5%` ≈ `98.0 ms` of `1,306 ms` 1w wall; net `8.2%` ≈ `107 ms`), giving an `M-ideal + A6a` combined anchor of **`~298.3 ms`** at 1w.
  - On this Xeon VM (`wikidata ×4`), production `prod_a5_avx2_alone` = `322.05 ms` (`37.1%`) and `prod_a6a_alone` = `547.00 ms` (`62.9%`), combined `868.60 ms` (`2.61×` i9 scale factor; A5:A6a ratio matches i9's `39.8%:60.2%` within 2.7 pp).

---

## Executive summary

**Verdict: ADMIT `mtf-fold-build` (`status = "open"`, frozen paired-oracle registration at [`registration.md`](registration.md)).**

1. **The 127-length cap splits only 1.14% of maximal runs on Wikidata, yet those splits hold 76.19% of all bytes and inflate `runs: Vec<u16>` by +7.79%.**
   On `wikidata` (`233` Open blocks, `209.70 MB`), A5 emits `16,320,934` capped `u16` records for `15,141,602` maximal runs (`+1,179,332` extra records; `4,717,328` extra on `wikidata ×4`). Just `172,682` runs (`width > 127`, mean `7.83` records/run, max `29,665` rows) produce `1,352,014` records (`8.28%` of records) and account for `159.77 MB` (`76.19%` of rows). Meanwhile, `95.01%` of maximal runs are thin (`width <= 16`, `12.67%` of rows), and `71.67%` have `width == 1`.
2. **A6a's `200 ms` (1w i9) is dominated by `OutQ16::push` (`61.3%`), not by `maximal()` merging (`5.7–7.0%` vs 4B packed maximal; `18.9–19.2%` vs 8B unpacked).**
   A6a reads back `31.13 MiB` (`124.52 MiB` on `wikidata ×4`) of intermediate `runs: Vec<u16>` (`119.4 KiB` p50, `202.8 KiB` p90, `361.8 KiB` max per block — spilling the 256 KiB L2 alongside `syms` `188–390 KiB` and bitmaps `24–49 KiB`), ignores A5's `counts[256]` + `per[256]` (`2 KiB/block`), recomputes per-byte row/strip counts, packs `755,021` thick strips (`8.64 MiB`), and executes `14.39M` `OutQ16::push` calls (`97.1%` fast path, `75.4%` `w == 1`, `8.41×` 32B-iota write amplification = `426.3 MiB` written for `50.7 MiB` of 2B ranks). Attempting a `w == 1` 2-byte fast path in `OutQ16::push` **regresses A6a by 10.4–10.9%** (`1.104–1.109×`) due to unpredictable `w == 1` vs `2..=16` branching.
3. **Folding A5 directly into A6a saves far more than `maximal()` alone because MTF's serial register chain overlaps `OutQ16::push`'s scattered 32B stores via out-of-order execution (plus eliminates the 124.5 MiB L2/L3 `runs` round-trip).**
   - Apples-to-apples direct fold (100% bit-for-bit A6a state-identical, `wikidata ×4`, 5 repeats):
     - `folded_scalar / unfolded_scalar` = **`0.767×`** `[0.762..0.773]` (**−23.3%**, saved `+481.8 ms`)
     - `folded_bitmap_u128 / unfolded_bitmap_u128` = **`0.862×`** `[0.857..0.874]` (**−13.8%**, saved `+245.7 ms`)
     - `p2_folded / p2_unfolded` (pre-built bitmaps, pre-sized slice writes, L2-hot `runs`) = **`0.816×`** `[0.782..0.827]` (**−18.4%**, saved **`+239.8 ms`** `[+222.9..+300.8 ms]` = **`27.6%`** of production `prod_a5_avx2 + prod_a6a`!; `0.814×` / `27.9%` on `wikidata ×1`).
   - **Surgical breakdown of the `+239.8 ms` Pass-2 fold gain on `wikidata ×4`:**
     - Eliminating A5's cap-127 split + `runs` write + `counts`/`per`: **`+15.6 ms`** (`1.8%` of prod A5+A6a)
     - Eliminating A6a's `maximal()` rescan + merge (vs 4B packed maximal): **`+38.4 ms`** (`4.4%` of prod A5+A6a)
     - Eliminating the `124.5 MiB` `runs` L2/L3 round-trip **+ OoO ILP overlap**: **`+185.8 ms`** (`21.4%` of prod A5+A6a). Specifically, `OutQ16::push`'s marginal cost drops by **46%** (`336.4 ms` standalone → `181.9 ms` inside `p2_folded`) as its scattered 32B stores hide behind MTF's serial register-update dependency chain.
4. **Implied i9-10900K room (with AVX2-speed caveat):**
   - Full Pass-2 fold ratio (`27.6%` of prod A5+A6a): **`~82–92 ms`** at 1w (`6.4–7.2%` of 1w wall).
   - Conservative non-ILP floor (`6.2%` of prod A5+A6a, zero extra ILP): **`~19–21 ms`** at 1w (`1.45–1.61%` of 1w wall).
   - Because `front_avx2` Pass 2 is faster than safe `MtfU128` (~2.5× fewer cycles per literal), an AVX2-folded prototype in `src/` has fewer MTF cycles available to overlap `OutQ16` — placing the realistic i9 expectation at **`~40–85 ms` at 1w (`3.1–6.6%` of 1w wall)** and **`~6–13 ms` at 8w behind ideal `S` (`3.2–7.0%` of `S`-ideal wall)**. Only a real AVX2 prototype timed on the i9 resolves where inside that `1.5–7.2%` band it lands, so `mtf-fold-build` is admitted under a frozen `S`-paired kill test judging `k = 1` and `k = 8`.

---

## 1. Structural census: cap-127 splitting, run lengths, and admission

### 1.1 Corpus routing & cap-127 split overview

`corpora/wikidata.bz2` (SHA-256 `82f65ace...9915`, matching `bench/wikidata.json`) has `431` blocks (`209,706,557` rows): `383` take the run route (`takes_runs`), of which `233` are Open-admitted (`228` `Cap::NONE`, `5` `Cap::dense`, `0` `Cap::dense` overflows; `209,697,662` rows = `99.996%`) and `150` tiny/sparse blocks fall back to `LazyCells`. `wikidata ×4` (SHA-256 `0b820d14...ce08e`) scales every count by exactly 4 (`932` Open blocks, `838,790,648` rows).

| corpus (mode) | Open blks (total) | Open rows (`n`) | `u16` records (`CAP=127`) | maximal runs | extra split recs (`+%`) | split runs (`>127`) | split byte share | thin runs (`≤16`) / rows | `w==1` run share | `w<=16` 2-bit miss |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **wikidata** (prod) | 233 (431) | 209,697,662 | 16,320,934 | 15,141,602 | +1,179,332 (+7.79%) | 172,682 (1.14%) | **76.19%** | 95.01% / 12.67% | 71.67% | 5.19% |
| **wikidata ×4** (prod) | 932 (1,724) | 838,790,648 | 65,283,736 | 60,566,408 | +4,717,328 (+7.79%) | 690,728 (1.14%) | **76.19%** | 95.01% / 12.67% | 71.67% | 5.19% |
| **runs** (`ab gen`, prod) | 1 (1) | 508,269 | 93,212 | 90,449 | +2,763 (+3.05%) | 109 (0.12%) | **70.24%** | 99.74% / 28.72% | 73.39% | 0.29% |
| **text** (`ab gen`, forced) | 26 (26) | 22,736,676 | 6,360,881 | 6,325,814 | +35,067 (+0.55%) | 8,559 (0.14%) | **21.45%** | 97.62% / 56.71% | 61.00% | 2.53% |
| **pattern** (`ab gen`, forced) | 44 (44) | 39,200,000 | 308,905 | 352 | +308,553 (+87,657%) | 308 (87.50%) | **100.00%** | 12.50% / 0.00% | 12.50% | 40.91% |
| **records** (`ab gen`, forced) | 22 (22) | 19,200,003 | 19,139,317 | 19,139,317 | 0 (+0.00%) | 0 (0.00%) | **0.00%** | 100.00% / 100.00% | 99.68% | 0.00% |

*(Under production admission, `text.bz2` has 10 blocks that enter `Cap::dense` and overflow to `LazyCells`, plus 16 refused by density; `pattern.bz2` has 44 blocks refused by sparsity `records * 64 < n`; `records.bz2` has 0 `takes_runs` blocks.)*

### 1.2 Maximal run-length distribution on `wikidata` (233 Open blocks)

| maximal run length | maximal runs | run % | rows | row % | `u16` records emitted | extra cap-127 records |
|---|---:|---:|---:|---:|---:|---:|
| `1` | 10,851,740 | 71.67% | 10,851,740 | 5.17% | 10,851,740 | 0 |
| `2` | 1,601,666 | 10.58% | 3,203,332 | 1.53% | 1,601,666 | 0 |
| `3..=4` | 873,906 | 5.77% | 3,004,268 | 1.43% | 873,906 | 0 |
| `5..=8` | 528,359 | 3.49% | 3,331,052 | 1.59% | 528,359 | 0 |
| `9..=16` | 530,916 | 3.51% | 6,177,182 | 2.95% | 530,916 | 0 |
| **thin subtotal (`≤ 16`)** | **14,386,587** | **95.01%** | **26,567,574** | **12.67%** | **14,386,587** | **0** |
| `17..=32` | 317,129 | 2.09% | 7,285,811 | 3.47% | 317,129 | 0 |
| `33..=64` | 171,853 | 1.13% | 7,721,337 | 3.68% | 171,853 | 0 |
| `65..=127` | 93,351 | 0.62% | 8,349,329 | 3.98% | 93,351 | 0 |
| `128..=255` | 56,345 | 0.37% | 10,141,459 | 4.84% | 112,872 | 56,527 |
| `256..=1024` | 78,506 | 0.52% | 42,805,129 | 20.41% | 379,010 | 300,504 |
| `> 1024` | 37,831 | 0.25% | 106,827,023 | 50.94% | 860,132 | 822,301 |
| **thick subtotal (`> 16`)** | **755,015** | **4.99%** | **183,130,088** | **87.33%** | **1,934,347** | **1,179,332** |

Percentiles of maximal run length (by run count): `p25 = 1`, `p50 = 1`, `p75 = 2`, `p90 = 6`, `p95 = 16`, `p99 = 160`, `max = 29,665` (split into `234` records!). Over half of all decoded bytes (`50.94%`) reside in just `0.25%` of maximal runs (`width > 1024`).

### 1.3 Admission sensitivity (`records.len()` vs `maximal_runs`)

Checking whether replacing capped `records.len()` with `maximal_runs` in `OpenCells::cap` changes any block's admission:
- **`wikidata` (`431` blocks):** `3` dense-band blocks (`41`, `56`, `278`: `r/n = 0.1509, 0.1549, 0.1518` → `mr/n = 0.1460, 0.1499, 0.1467`) shift from `Cap::dense` to `Cap::NONE`. All three already pass `Cap::dense`, so **0 blocks change Open vs LazyCells routing**.
- **`runs`, `text`, `pattern`, `records`:** **0 blocks differ**.
- Even better: tracking exact `cap127_records` inside a folded Pass 2 requires only `recs += total.div_ceil(127)` on the `1.14%` of runs with `total > 127` (`recs += 1` otherwise), preserving **100% identical admission** on every possible input at near-zero cost.

### 1.4 What A5 + A6a write, re-read, and recompute (`wikidata ×4`, 932 Open blocks)

| structure / work | where written | where read | per-block KiB (`p50 / p90 / max`) | `wikidata ×4` total | fate under fold |
|---|---|---|---:|---:|---|
| `syms: Vec<u16>` | Scanner (A3) | A5 Pass 1 & 2 | 188.3 / 297.5 / 390.2 | 190.51 MiB | unchanged |
| `lit` + `one` bitmaps | A5 Pass 1 | A5 Pass 2 | 23.6 / 37.2 / 48.8 | 23.8 MiB | unchanged |
| **`runs: Vec<u16>`** | **A5 Pass 2** | **A6a `maximal()`** | **119.4 / 202.8 / 361.8** | **124.52 MiB** | **ELIMINATED** |
| **`counts[256]` + `per[256]`** | **A5 Pass 2** | **never read by Open** | **2.0 / 2.0 / 2.0** | **1.82 MiB** | **ELIMINATED** |
| `First` (`thin`/`thick`/`thick_rows`) | A6a `Pass::piece` | A6b `finish()` | 3.0 / 3.0 / 3.0 | 2.73 MiB | computed directly in A5 Pass 2 |
| `edges` thick strips (`ns = 3.02M`) | A6a `Pass::piece` | A6b `finish()` | 40.2 / 45.9 / 61.7 | 34.56 MiB | written directly in A5 Pass 2 |
| `OutQ16` in `out` (`m = 106.27M`) | A6a `OutQ16::push` | A6c `wire_xor` | 254 / 389 / 516 chunks (1 KiB each; `0` ovf) | 202.69 MiB exact ranks / **1,705.27 MiB** 32B-iota written (`8.41×` amp) | written directly in A5 Pass 2 (overlaps MTF chain!) |

**L2 capacity observation:** On both the Xeon VM and the i9-10900K (256 KiB L2 per core), `syms + bitmaps + runs` totals **331 KiB at p50, 538 KiB at p90, and 801 KiB at max** — exceeding the 256 KiB L2 on nearly every block. Eliminating `runs` removes `124.52 MiB` of write+read traffic on `wikidata ×4` and shrinks the live working set of A5+A6a directly inside L2.

---

## 2. Exact-state-verified timing census

Every candidate arm was verified 100% bit-for-bit state-identical to production A5+A6a on every admitted block before timing.

### 2.1 `wikidata ×4` (`932` Open blocks, `838.79M` rows, `60.57M` maximal runs, `5` interleaved repeats)

| arm | median ms | `min..max` ms | ns/row | ns/max_run | vs `prod_a5_avx2_plus_a6a` |
|---|---:|---:|---:|---:|---:|
| `prod_a5_avx2_alone` (actual `runs::front`) | 322.05 | 321.30..330.53 | 0.384 | 5.32 | 0.371× |
| `prod_a5_scalar_alone` (actual `runs::front_scalar`) | 1412.56 | 1408.43..1433.86 | 1.684 | 23.32 | 1.626× |
| `bitmap_pass1_alone` (safe scalar Pass 1) | 477.52 | 473.52..494.97 | 0.569 | 7.88 | 0.550× |
| `bitmap_u128_a5_alone` (safe Pass 1 + Pass 2) | 1236.66 | 1228.17..1245.48 | 1.474 | 20.42 | 1.424× |
| `prod_a6a_alone` (exact mirror of `src/open.rs` A6a) | 547.00 | 538.22..560.08 | 0.652 | 9.03 | 0.630× |
| `a6a_from_maximal` (8B `(u8, u32)` maximal input) | 435.90 | 431.78..452.79 | 0.520 | 7.20 | 0.502× |
| `a6a_from_packed_max` (4B packed `u32` maximal input) | 507.39 | 502.62..512.33 | 0.605 | 8.38 | 0.584× |
| `a6a_no_outq` (A6a without `OutQ16::push`) | 210.57 | 207.15..215.06 | 0.251 | 3.48 | 0.242× |
| `a6a_maximal_no_outq` (classify + `edges` only) | 200.68 | 198.74..217.24 | 0.239 | 3.31 | 0.231× |
| `a6a_w1_fast` (`OutQ16` 2B branch for `w == 1`) | 603.12 | 593.99..606.79 | 0.719 | 9.96 | 0.694× |
| **`prod_a5_avx2_plus_a6a`** | **868.60** | **866.04..888.11** | **1.036** | **14.34** | **1.000×** |
| `unfolded_scalar` (L2-hot `runs`) | 2057.96 | 2048.00..2148.07 | 2.453 | 33.98 | 2.369× |
| `folded_scalar` (direct fold) | 1581.27 | 1566.57..1660.20 | 1.885 | 26.11 | 1.820× |
| `unfolded_bitmap_u128` (L2-hot `runs`) | 1777.61 | 1771.74..1863.20 | 2.119 | 29.35 | 2.047× |
| `folded_bitmap_u128` (direct fold) | 1534.23 | 1531.13..1605.68 | 1.829 | 25.33 | 1.766× |
| `p2_mtf_core` (Pass 2 only, no `runs` write) | 741.16 | 728.58..765.37 | 0.884 | 12.24 | 0.853× |
| `p2_mtf_with_runs` (Pass 2, pre-sized slice write) | 750.76 | 744.19..764.54 | 0.895 | 12.40 | 0.864× |
| `p2_unfolded` (`p2_mtf_with_runs` + `prod_a6a`, L2-hot) | 1304.44 | 1288.23..1380.79 | 1.555 | 21.54 | 1.502× |
| **`p2_folded`** (Pass 2 folded directly into A6a) | **1065.35** | **1050.13..1101.66** | **1.270** | **17.59** | **1.227×** |
| `p2_folded_no_outq` (Pass 2 folded without `OutQ16`) | 883.44 | 876.84..982.78 | 1.053 | 14.59 | 1.017× |

*(On `wikidata ×1`, 7 repeats: `prod_a5_avx2_alone = 80.85 ms`, `prod_a6a_alone = 136.00 ms`, `prod_a5_avx2_plus_a6a = 218.07 ms`, `p2_unfolded = 323.98 ms`, `p2_folded = 262.22 ms`, paired `p2_folded / p2_unfolded = 0.814× [0.803..0.828]`, saved `+60.54 ms [+55.94..+63.57 ms]` = `27.9%` of prod A5+A6a — matching `wikidata ×4` to within 0.3% per row.)*

### 2.2 Paired per-repeat decomposition across all corpora

| corpus (repeats) | `a6a_from_packed_max / prod_a6a` | `a6a_no_outq / prod_a6a` (`OutQ16` share) | `a6a_w1_fast / prod_a6a` | `folded_scalar / unfolded_scalar` | `p2_folded / p2_unfolded` | Pass-2 saved as % of `prod A5+A6a` |
|---|---:|---:|---:|---:|---:|---:|
| **wikidata** (7) | `0.943×` `[0.902..0.998]` | `0.384×` (**61.6%**) | `1.104×` (−10.4%) | `0.764×` `[0.705..0.789]` | **`0.814×`** `[0.803..0.828]` | **27.9%** |
| **wikidata ×4** (5) | `0.930×` `[0.897..0.948]` | `0.387×` (**61.3%**) | `1.109×` (−10.9%) | `0.767×` `[0.762..0.773]` | **`0.816×`** `[0.782..0.827]` | **27.6%** |
| **runs** (prod, 7) | `0.933×` `[0.858..0.949]` | `0.308×` (**69.2%**) | `1.203×` (−20.3%) | `0.784×` `[0.753..0.803]` | **`0.899×`** `[0.875..0.918]` | **13.7%** |
| **text** (forced, 7) | `0.976×` `[0.947..1.031]` | `0.255×` (**74.5%**) | `1.212×` (−21.2%) | `0.792×` `[0.771..0.825]` | **`0.833×`** `[0.808..0.978]` | **24.1%** |
| **pattern** (forced, 7) | `0.102×` `[0.096..0.116]` | `1.000×` (**0.0%**) | `0.976×` (+2.4%) | `0.041×` `[0.036..0.071]` | **`0.046×`** `[0.018..0.110]` | **95.8%** |
| **records** (forced, 7) | `0.968×` `[0.904..0.977]` | `0.196×` (**80.4%**) | `1.008×` (−0.8%) | `0.815×` `[0.792..0.822]` | `0.988×` `[0.974..1.047]` | **1.6%** |

### 2.3 Where the Pass-2 fold savings come from (`wikidata ×4`)

| component | VM delta (`wikidata ×4`) | % of `prod A5+A6a` (`869.1 ms`) | implied i9 1w (`stage1` 332.7 ms / `M-ideal+A6a` 298.3 ms) | mechanism |
|---|---:|---:|---:|---|
| (1) A5 emit (`p2_with_runs − p2_core`) | `+15.6 ms` | `1.8%` | `6.0 / 5.4 ms` | cap-127 split + `runs` slice store + `cnt`/`counts`/`per` |
| (2) A6a `maximal()` (vs 4B packed max) | `+38.4 ms` | `4.4%` | `14.7 / 13.2 ms` | rescanning `runs` + inner adjacent-byte merge loop (`+111.1 ms` / `12.8%` vs 8B unpacked) |
| (3) L2 round-trip + MTF/OutQ16 OoO ILP | `+185.8 ms` | `21.4%` | `71.1 / 63.8 ms` | removing `124.5 MiB` `runs` L2/L3 spill + overlapping `OutQ16` 32B stores behind MTF's serial `f0`/`f1` chain |
| **Total Pass-2 fold (`p2_unfolded − p2_folded`)** | **`+239.8 ms`** | **`27.6%`** | **`91.8 ms (7.17% 1w) / 82.3 ms (6.43% 1w)`** | **direct fold of A5 Pass 2 into A6a** |
| **Conservative floor (1 + 2, zero extra ILP)** | **`+54.0 ms`** | **`6.2%`** | **`20.7 ms (1.61% 1w) / 18.5 ms (1.45% 1w)`** | **hard lower bound if AVX2 Pass 2 hides zero extra `OutQ16` latency** |

**Proof of the OoO ILP mechanism:** In standalone `prod_a6a`, `OutQ16::push` adds `547.0 − 210.6 = 336.4 ms`. Inside `p2_folded`, the exact same `OutQ16::push` adds only `1065.4 − 883.4 = 181.9 ms` — **46% (`154.5 ms`) of `OutQ16::push` vanishes behind MTF's serial register chain.**

---

## 3. Projected 8-worker i9-10900K effect & caveats

- **1 worker (unmasked):** Anchored to `character-stage1` (`A5+A6a = 332.7 ms` of `1,280.1 ms` worker busy) and `ideal-parts` (`~298.3 ms`), the conservative floor is `~19–21 ms` (`1.5–1.6%` wall) and the full Pass-2 ratio is `~82–92 ms` (`6.4–7.2%` wall). Accounting for `front_avx2`'s faster Pass 2 (`~280 ms` vs safe `MtfU128`'s `741 ms` on `wikidata ×4`, leaving fewer MTF cycles for OoO overlap), a realistic AVX2-folded prototype lands around **`~40–85 ms` (`3.1–6.6%` of 1w wall)**.
- **8 workers (paired against `S` ideal):** At `k = 8`, shipped `S` and workers co-limit (`185.0 ms` vs `184.7 ms`, `character-stage1`), so standalone `F / none` is scanner-masked (~1–2%). Behind **`S` ideal** (zero-copy scan oracle, where `C` is real and warm), `A5 + A6a` is `19.3 + 28.2 = 47.5 ms/w` (`25.7%` of worker busy), so a `13–28%` AVX2-folded cut saves **`~6–13 ms/w` (`~3.2–7.0%` of `S`-ideal wall)**.
- **Critical caveats:**
  1. **2-vCPU KVM VM:** Single-threaded timings on a 2.30 GHz Haswell/Broadwell VM cannot measure 8-worker contention, L3 pressure, or Comet Lake's wider 5-wide OoO window directly.
  2. **AVX2 Pass-2 speed vs safe `MtfU128`:** Zero-`unsafe` `examples/` cannot invoke `front_avx2`'s SIMD MTF intrinsics inside the folded loop; the real AVX2-folded kernel (`mtf-fold-build`) must be measured on the i9 to determine how much `OutQ16` overlap persists behind AVX2 MTF.
  3. **20 MiB L3 giveback at `k = 9..10`:** Per [`2026-11-26-synthesis/README.md`](../../2026-11-26-synthesis/README.md) §§1 & 3, speeding up workers at `k = 9..10` pushes `ready_bytes` past 12–16 MiB and triggers coupled `R` (`A9`) + `D` (`copy_out`) L3 eviction. Accordingly, [`registration.md`](registration.md) judges `mtf-fold-build` at **`k = 1` and `k = 8`** (where `ready_bytes` stays moderate) and logs `A9`, `copy_out`, and `ready_bytes` hwm on every trace.

---

## 4. Decision & handoff to `mtf-fold-build`

`mtf-fold-census` shows clear, mechanistically understood room (`13–28%` of `A5+A6a`, realistic `~3.1–6.6%` 1w wall and `~3.2–7.0%` at 8w behind `S` ideal, plus `124.5 MiB` of L2/L3 traffic eliminated). `mtf-fold-build` is therefore **admitted (`status = "open"`)** with its frozen paired-oracle registration written before any prototype code at [`docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md`](registration.md).
