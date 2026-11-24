# N-8: Amended 3-part N-0, empirical verification of MONODROMY M-7, and E2 screens of Positional Copy Schemes & Compressed Suffix Coordinates

*Cycle 2 response to Coordinator Note #2 (`origin/claude/sharp-meitner-cx0v78` `908d292`, `channel/coordinator-to-neon.md` items 1 & 4) and MONODROMY Record 102 `M-7` (`d0d0232`).*

**Script:** [`research/neon/scripts/measure_cycle2.py`](scripts/measure_cycle2.py) (standard-library Python, run on the six admitted blocks of `origin/data/admitted-bwt` joined with `#805` 1W i9 kernel costs).

**Verdicts:**
- **Amended 3-part N-0:** Even under the novelty split, **novel hex alone is `61.3–68.2%` of `h₁`** (cost-weighted **`64.61%`** under maximal `[0-9a-fA-F]+` tokens). Repeated 40-hex SHA1 hashes contribute literally **`3` width-1 rows out of `145,800` bytes (`0.000%–0.01%` of `h₁`)**, confirming MONODROMY `M-7` even more strongly than predicted (`0.00` w1/occ vs predicted `≤ 7`). **`δ = 1/2` remains unreachable without novel hex beating `aR = 1`.**
- **Lead 1 — Positional copy schemes (LZ copies keyed by text position, r-index `φ`/toeholds):** **CLOSED at (a) Escape** (for `φ`/toehold acquisition: `≥ h₁` single-row boundary samples charged by §1a clause 1) and **CLOSED at (d) Payoff** (`repeated_hex` is `7.8–18.0%` of `h₁` depending on minimum token length, repeated 40-hex SHA1s are `0.00–0.01%` of `h₁`, and all repeated non-GUID 8-grams are `10.7–15.2%` of `h₁` = `~2.1–2.9%` of 1W kernel time). Status: `tested failure within defined scope`.
- **Lead 2 — Compressed suffix coordinates (sampled SA, r-index, Kempa SODA 2019):** **CLOSED at (a) Escape** (`≥ h₁` run-boundary SA samples charged by §1a clause 1; `O(1)`-pass acquisition blocked by Record 84 "labels first" and Record 101 swap-invariance) and **CLOSED at (c) Viability** (sample table size `Θ(r) ≥ h₁`). Status: `tested failure within defined scope`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact inverted texts `T`, width-1 row masks, and 1W i9 kernel costs on the six admitted blocks (`origin/data/admitted-bwt` joined to `#805` `census.tsv`/`features.tsv`, total `9.199 ms`).
   - *Missing data note (same as `N-2`):* `#805`'s `census.tsv` contains run histograms only and `corpora/wikidata.bz2` is absent from the repository; the remaining 226 census blocks require a corpus-side run of `examples/census.rs` to report their per-block novelty split.
2. **Real code path:**
   - `OpenCells` (`LIMIT = 16`, 1W i9 emit + construct costs from `#805`; `h₁` emit = `19.09%` of total 1W kernel time across 232 blocks).
3. **Assumptions knowingly unrealistic:**
   - Novelty is a text-level property (not decidable from run records in `O(r)`, as Record 102 §2 notes); we report it as an oracle upper bound on copyable content, under two tokenizations (`Def1`: all maximal `[0-9a-fA-F]+`; `Def2`: identifier-length `[0-9a-fA-F]{6,}` separating out 1–5 char hex/English/digits).

## 1. Amended 3-part N-0 on the six admitted blocks (`[measured]`)

### Def1: maximal contiguous hex tokens `[0-9a-fA-F]+` (any length `≥ 1`)

| Block | `h₁` | Novel hex | Repeated hex | Non-hex | Novel% | Rep% | Non-hex% | 1W cost (ms) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 23,456 | 6,182 | 5,478 | **66.8%** | 17.6% | 15.6% | 1.005 |
| `block-2.L` | 34,262 | 22,357 | 5,897 | 6,008 | **65.3%** | 17.2% | 17.5% | 1.207 |
| `block-85.L` | 55,504 | 35,546 | 10,328 | 9,630 | **64.0%** | 18.6% | 17.4% | 1.352 |
| `block-143.L` | 72,578 | 44,509 | 14,413 | 13,656 | **61.3%** | 19.9% | 18.8% | 2.151 |
| `block-210.L` | 97,054 | 66,164 | 16,466 | 14,424 | **68.2%** | 17.0% | 14.9% | 2.034 |
| `block-207.L` | 62,407 | 39,311 | 10,480 | 12,616 | **63.0%** | 16.8% | 20.2% | 1.450 |
| **Cost-weighted** | — | — | — | — | **64.61%** | **17.96%** | **17.43%** | **9.199** |

### Def2: identifier-length hex tokens `[0-9a-fA-F]{6,}` (separating 1–5 char hex/English words/short numbers)

| Block | `h₁` | Novel hex (`≥6`) | Rep hex (`≥6`) | Short hex (`1–5`) | Non-hex | Novel(`≥6`)% | Rep(`≥6`)% | Short% | Non-hex% |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 15,781 | 3,865 | 9,992 | 5,478 | 44.9% | 11.0% | 28.5% | 15.6% |
| `block-2.L` | 34,262 | 15,397 | 3,101 | 9,756 | 6,008 | 44.9% | 9.1% | 28.5% | 17.5% |
| `block-85.L` | 55,504 | 23,673 | 4,766 | 17,435 | 9,630 | 42.7% | 8.6% | 31.4% | 17.4% |
| `block-143.L` | 72,578 | 31,491 | 5,322 | 22,109 | 13,656 | 43.4% | 7.3% | 30.5% | 18.8% |
| `block-210.L` | 97,054 | 46,579 | 6,622 | 29,429 | 14,424 | 48.0% | 6.8% | 30.3% | 14.9% |
| `block-207.L` | 62,407 | 27,284 | 3,688 | 18,819 | 12,616 | 43.7% | 5.9% | 30.2% | 20.2% |
| **Cost-weighted** | — | — | — | — | — | **44.72%** | **7.81%** | **30.04%** | **17.43%** |

*(Note: in Def2, RFC 4122 GUIDs have hyphen-separated chunks of lengths `8-4-4-4-12`, so the three 4-char GUID chunks land in `short_hex`, explaining why `novel_hex` drops from `64.6%` in Def1 to `44.7%` in Def2 — Def1 captures all 5 GUID chunks as novel hex tokens of length 4/8/12, where 4-char chunks collide occasionally at ~20% while 8- and 12-char chunks are 100% novel.)*

## 2. Empirical test of MONODROMY M-7 (repeats-are-wide) on 40-hex SHA1 tokens (`[measured]`)

| Block | Repeated SHA1 occs | Repeated SHA1 bytes | Width-1 rows in rep SHA1 | w1 / rep occ | Share of `h₁` | Novel SHA1 occs | w1 / novel occ |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 794 | 31,760 | **3** | **0.00** | **0.01%** | 47 | 35.89 |
| `block-2.L` | 765 | 30,600 | **0** | **0.00** | **0.00%** | 71 | 35.11 |
| `block-85.L` | 1,276 | 51,040 | **0** | **0.00** | **0.00%** | 22 | 36.59 |
| `block-143.L` | 356 | 14,240 | **0** | **0.00** | **0.00%** | 167 | 35.90 |
| `block-210.L` | 207 | 8,280 | **0** | **0.00** | **0.00%** | 132 | 35.86 |
| `block-207.L` | 247 | 9,880 | **0** | **0.00** | **0.00%** | 144 | 35.53 |

**Result:** Across **3,645 repeated 40-hex SHA1 occurrences (`145,800` bytes)** on all 6 admitted blocks, only **3 bytes (`0.002%`)** sit in width-1 runs (`0.000%–0.01%` of `h₁`), while novel SHA1s average **`35.11–36.59`** width-1 rows per occurrence (matching `(15/16)^2 × 40 = 35.16`). MONODROMY's `M-7` prediction (`≤ 7` w1/occ) is confirmed and tightened to `0.00` on real Wikidata because JSON delimiters (`","` / `"}`) extend the shared right context past the end of every repeated SHA1!

## 3. Directed four-gate screens of Coordinator Note #2 leads 1 & 2

### Lead 1: Positional copy schemes (LZ copies keyed by text position, r-index `φ` / toeholds)

- **(a) Escape & (b) Passes — NO `[argued]`:**
  - Copying a repeated substring `W` from an earlier text position `p_1` to `p_2` requires knowing `(p_1, p_2, |W|)` (or the `φ`-shift `Δ = p_2 - p_1` across the rows of `W`). As MONODROMY Record 102 §2.4 notes, acquiring `φ` / toehold samples at run boundaries requires one positional sample per run boundary, and every width-1 run is its own boundary — so sampling them costs `≥ h₁` single-row positional facts (charged by §1a clause 1).
- **(d) Payoff (decisive even if copies were supplied free!) — NO `[measured]`:**
  - By `M-7` and §2 above, **99.998%** of repeated SHA1 bytes sit in runs of width `≥ 2` (which `c₁` does not count and Open/Cells already process in strips/bundles).
  - Even combining **all** repeated non-GUID 8-grams (`10.7–15.2%` of `h₁`, `screen_cycle1_frameworks.py`) or **all** repeated `[0-9a-fA-F]{6,}` hex tokens (`5.9–11.0%` of `h₁`, Def2), the maximum possible reduction in `c₁` is far below `δ = 50%` (and represents only `~2–3%` of 1W Open kernel time).

### Lead 2: Compressed suffix coordinates (sampled suffix arrays, r-index, Kempa SODA 2019)

- **(a) Escape — NO `[proved]`:**
  - An r-index (Gagie–Navarro–Prezza 2020) stores `SA` samples at the `2r` run boundaries of `L` (and evaluates interior rows via `Φ`). Every width-1 run `R = {x}` is both a run start and a run end, so the r-index sample array itself contains **`≥ h₁` individual single-row positional facts `SA[x] = pos`** — each charged `1` under §1a clause 1 (`c₁ ≥ h₁`).
  - Furthermore, constructing the `r`-index `SA` samples from `(L, p)` without walking `LF` is blocked by Record 84 (text-order run labels determine the text) and by Record 101 Theorem 3 + `N-9` (no coarse aggregate determines `SA[x]` outside `|U| ≤ 1–354`). Kempa (SODA 2019) computes `r`-index samples from the *text* `T` (compression direction), not from `(L, p)` (decompression direction) in `K = 2` passes.
- **(c) Viability — NO `[measured]`:**
  - Parameter cost scales with: number of run-boundary SA samples = `Θ(r) = 51k–129k > h₁` on the 6 admitted blocks (and `48k–180k` on the 232 census blocks), plus `Φ`-successor table overhead violating `S ≤ openPeakScratch` if stored alongside Open's tables.

## 4. Obstruction table entries

| Framework | Route | Gate failed | Witness | Scope | Escape condition | Reach | Strength |
|---|---|---|---|---|---|---|---|
| Positional copy schemes (`φ`/toeholds, LZ copies) | R2 (if non-rigid copy discovery existed) | **(a)/(b)** acquisition (`≥ h₁` boundary samples) and **(d) Payoff** | 6 admitted blocks: 3,645 repeated SHA1 occs (145,800 B) yield **3** w1 rows (`0.00–0.01%` of `h₁`); repeated `[0-9a-fA-F]{6,}` = `5.9–11.0%` of `h₁`; novel hex = `61.3–68.2%` of `h₁` | All text-position copy schemes attacking width-1 runs on `𝓕` | Must act on *novel* (non-repeating) content, which has no earlier copy source in the block | Global for copy-only schemes on `𝓕` | Proved under model I (`M-7`) + measured exact on 6 admitted blocks |
| Compressed suffix coordinates (r-index, sampled SA) | R1 / charged E2 | **(a) Escape** (§1a clause 1: `≥ h₁` singleton boundary samples) and **(c) Viability** (`Θ(r) ≥ h₁` samples) | 6 admitted blocks (`r = 51k–129k`, `h₁ = 34k–97k`, every w1 run is its own boundary) + Record 84 + Record 101 | All run-boundary sampled SA / `Φ` representations acquired from `(L, p)` | Drop per-run-boundary SA samples; compute positions of `≥ 2` scattered width-1 runs jointly from a non-boundary multi-row rule | Bounded to run-boundary SA/r-index representations | Argued (§1a clause 1 + Record 84) + measured on 6 admitted + 232 census blocks |

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact on 6 admitted blocks; 226 census blocks lack raw `.L` in repo.
2. **Model assumptions:** Def1 and Def2 bracket tokenization ambiguity cleanly; both show novel hex dominates and repeated SHA1 contributes `0.00–0.01%` of `h₁`.
3. **Method:** Direct regex tokenization + exact BWT inversion + `M-7` per-occurrence count.
4. **Outcome:** Completes amended `N-0`, validates and sharpens MONODROMY's `M-7`, and closes both priority leads of Coordinator Note #2.
