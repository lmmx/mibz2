# T5 (Cycle 4) — Cross-Stage Enrichment: What the Serial Huffman Scan Already Sees vs What Workers Recompute

- **Headline (referee's terms — total reads per lone row, acquisition + emission included, like for like):**
  - **$\Delta Q = 0$ (no reduction in `Q_total / h₁`).** Every quantity visible to the serial Huffman scan (`src/block.rs`) is purely streamed (`Q = 0` addressed reads), so cross-stage riders leave total inversion reads unchanged (`2.105–3.271 h₁` Open walk, `0.799–1.289 h₁` M-17, `0.690–1.173 h₁` L1-cursor `R⁴`). By **Theorems T5.1–T5.2** (proved & witnessed on 8/8 blocks), nothing in the Huffman scan before `EOB` or before MTF determines exact `LF(x)` targets or output-order codes; hence the four-arm `26%` ceiling at 10 workers (`D/A = 0.741`) is **pipeline-masking superposition of T4 (`0.806`) × T1 (`0.920`)**, not a single shortcut from Huffman to emission.
  - **What the scan *does* already see (streamed / T3–T2 leverage):** RLE2 alone (`s ≤ 1` vs `s ≥ 2`, zero MTF) recovers **100%** of `(n, r, h₁, m₁₆, n_thick, mark_rank)` on 8/8 blocks, strips **`30.5–46.5%`** `RUNA`/`RUNB` symbols (cutting scanner→worker queue bytes by **`30.3–46.2%`**, `175–362 KB → 101–252 KB`), exposes **`3,990–5,284`** spurious `CAP=127` splits (`3.5–10.2%` of `r`) that `runs.rs` creates and `open.rs::maximal()` re-coalesces, and its already-decoded 50-symbol Huffman selectors (`1.8–3.7 KB`) separate novel-hex lone-row regimes (`83–85%` lone rows, `0.03–0.25%` `nn ≥ 32`) from thick template regimes (`91–98%` thick rows) with normalized mutual information **`NMI = 0.190–0.338`** (on 4/6 exact bit-identical + 2/6 boundary-recompressed libbzip2 blocks).
- **Prior art:** MONODROMY Records 121–122 (Proposition M placeholder MTF chunking; scan counters for `r`, `h₁`, width histogram, and 6-way selectors); `src/runs.rs` (direct MTF/RLE2 → run records without `L`); 22 Nov fused Huffman+MTF kill (`1.185×` whole decoder, `tracks/cross-stage.md`); Patel et al. 2012 & pugz/rapidgzip placeholder chunking.
- **Script & TSVs:** [`t5_cross_stage_huffman_enrichment.py`](t5_cross_stage_huffman_enrichment.py) derives [`data/t5_huffman_stage_accounting.tsv`](data/t5_huffman_stage_accounting.tsv) (8 rows) and [`data/t5_selector_regime_map.tsv`](data/t5_selector_regime_map.tsv) (42 rows). All 8/8 blocks verified exact on MTF+RLE2 round-trip (`decode_runs_rs_mirror == blk.L`) and RLE2-only geometry (`rle2_only_geometry == common.build_retained_system_genuine`).

---

## 1. Exact Inventory: What the Serial Scan Sees vs What Workers Recompute (6 Wikidata Blocks)

| Block (`n ≈ 900k`) | `syms` (`2B` queue KB) | `RUNA/B` % | Compact `(lit,varint)` KB (saved) | `runs.rs` `CAP=127` extra splits (`% r`) | `pair (L,L)` % (lone in 1 LUT) | `nn < 16` / `< 32` / `≥ 32` | `RLE2`-only geom exact? | `sel_NMI` (`bz2_exact_L`) |
|---|---:|---:|---:|---:|---:|---|---:|---:|
| `block-0.L` | `89,730` (`175.3`) | `43.0%` | `100.7` (`-42.6%`) | `+5,202` (`10.2%`) | `39.1%` | `85.1% / 98.5% / 1.5%` | **YES** | `0.317` (`YES`) |
| `block-2.L` | `101,931` (`199.1`) | `46.5%` | `107.2` (`-46.2%`) | `+4,946` (`9.1%`) | `33.6%` | `84.0% / 98.2% / 1.8%` | **YES** | `0.213` (`NO*`) |
| `block-85.L` | `126,486` (`247.0`) | `37.2%` | `155.7` (`-37.0%`) | `+5,284` (`6.7%`) | `43.7%` | `84.2% / 98.2% / 1.8%` | **YES** | `0.268` (`YES`) |
| `block-143.L` | `180,206` (`352.0`) | `39.9%` | `212.4` (`-39.7%`) | `+3,990` (`3.7%`) | `40.3%` | `83.1% / 97.9% / 2.1%` | **YES** | `0.190` (`YES`) |
| `block-210.L` | `185,183` (`361.7`) | `30.5%` | `252.2` (`-30.3%`) | `+4,501` (`3.5%`) | `52.3%` | `85.2% / 98.5% / 1.5%` | **YES** | `0.338` (`YES`) |
| `block-207.L` | `137,584` (`268.7`) | `36.7%` | `171.1` (`-36.3%`) | `+4,305` (`4.9%`) | `45.4%` | `83.0% / 97.8% / 2.2%` | **YES** | `0.240` (`NO*`) |

*\*`block-2.L` and `block-207.L` differ at the 900k RLE1 block cut when recompressed standalone; `block-0, 85, 143, 210` reproduce `blk.L` bit-for-bit identically through `libbzip2`.*

### Four Concrete Redundancies Across the Pipeline
1. **RLE2 run geometry is 100% MTF-independent (recomputed twice today):**
   `src/runs.rs` first samples 4,096 symbols (`takes_runs`), then scans all of `syms` to build `lit`/`one` bitmaps, then decodes RLE2, then caps runs at `CAP=127` (`+3,990–5,284` extra records); `src/open.rs::maximal()` then scans all of `runs` again just to re-coalesce maximal runs and classify `w ≤ 16` vs `w ≥ 17`! Yet `s ≤ 1` vs `s ≥ 2` + `orig_ptr` already determines 100% of `(n, r, h₁, m₁₆, n_thick, M₁₆ intervals, mark_rank)` before MTF runs.
2. **10-worker T4 warning (where to fuse vs NOT fuse):**
   At 10 workers the serial scanner (`T4`, `C/A = 0.806`) is the bottleneck. Adding branching RLE2 or MTF logic into `src/block.rs` risks slowing T4 (the 22 Nov kill went from `74.5 → 107.9 ms`, `1.185×` whole decoder). By contrast, **fusing `runs.rs::front_avx2` directly into `open.rs` Pass 1 inside the worker** (eliminating `runs: Vec<u16>`, `CAP=127` splits, and `maximal()`) adds **0 cycles to the serial scanner** while saving 1 full streaming pass + `113–267 KB` intermediate traffic per block in the workers!
3. **Genuine 50-symbol Huffman selectors (`scratch.selectors`) already segment the column into hex vs template regimes:**
   On bit-exact `block-0.L` (`6` tables, `1,795` groups, `675` switches; full per-table breakdown in [`data/t5_selector_regime_map.tsv`](data/t5_selector_regime_map.tsv)):
   - **Table 1 (hex / lone-row regime, `18.1%` of groups):** `93.2%` of runs have `w = 1`, `82.6%` of rows are lone, only `4.4%` thick rows, `92.6%` have `nn < 16`, and **only `0.03%` hit `nn ≥ 32`** (essentially pure single-XMM `f0` MTF!).
   - **Table 4 (template / thick-run regime, `25.2%` of groups):** covers **`660,722` rows (`73.4%` of the entire block)**, **`97.6%` thick rows**, only `0.23%` lone rows, `72.7%` `RUNA`/`RUNB`.
   - **Table 3 (2-byte swap regime, `7.9%` of groups):** **`49.9%` of literals are `nn = 1`** (reaches `65–72%` on `block-85.L` and `block-207.L`).
   Passing `scratch.selectors` (`1.8–3.7 KB`, already decoded!) costs zero scanner work and gives workers a free per-50-symbol regime hint (e.g., skipping `second()` / specialising MTF on `nn < 16` hex groups, or exact zero-sample admission).
4. **2-symbol LUT pair structure (`src/huff.rs`):** `33.6–52.3%` of adjacent symbol pairs are `(LIT, LIT)` (immediate lone row `w = 1` in 1 lookup) and `13.3–26.6%` are `(RUN, RUN)` (2-bit bijective base-2 digit pair).

---

## 2. Structural Limits of T5 (Proved & Witnessed on 8/8 Blocks)

- **Theorem T5.1 (Pre-EOB Causality Obstruction — proved & witnessed):**
  For any prefix `syms[0..t]` strictly before `EOB`, changing even the final single byte of the block alters `counts[b]` and shifts `C[b'] = Σ_{a < b'} counts[a]` for all bytes `b'` between the old and new final byte. On the 6 Wikidata blocks, flipping only `L[n-1]` (keeping `99.9999%` of `syms` identical) shifts exact `LF(x)` targets on **`15.4%–50.0%` of all rows** (`t51_shifted_frac`). Hence **no exact `LF` coordinate, thick-strip image `strip.to`, retained-map edge `R[j]`, or output-order code is determined before `EOB`**. $\square$
- **Theorem T5.2 (Pre-MTF Equivariance Obstruction — proved & witnessed):**
  Permuting `seq_to_unseq` (e.g., reversing it) while leaving **100% of `syms` bit-identical** preserves `(n, r, h₁, m₁₆, n_thick)` identically while shifting `C[L[x]]` on **`100.0%` of rows** (`t52_shifted_frac = 1.0000` on 8/8 blocks). Without running MTF on the `r` literals, the Huffman scan determines 100% of domain geometry and 0% of byte identities, F-bucket offsets, or cycle order. $\square$

### Consequence for the Four-Arm 26% (`D/A = 0.741` at 10 Workers)
Because T5.1 and T5.2 rule out acquiring output-order codes from the Huffman scan, the 26% four-arm payoff at 10 workers decomposes cleanly into:
1. **T4 (`C/A = 0.806`, `-19.4%`):** breaking the serial Huffman scan wall across blocks (parallel scanner, `briefs/scanner.md`).
2. **T1/T2 (`D/C = 0.920`, `-8.0%` of `C`):** cutting worker inversion work once the scanner no longer masks workers.
3. **T5 riders (worker-side `runs.rs`+`open.rs` Pass 1 fusion; passing `selectors_used` / exact RLE2 stats once T4 is parallelized):** capture T3/T2 streamed-work and queue-traffic savings with `ΔQ = 0`, never risking T4 regression on the serial thread.
