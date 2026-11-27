# Post-`#1046` `A6b` Second-Pass Dissection & Redesign Probe (`a6b-v2-dissection`)

**Date:** 2026-11-27  
**Probe:** [`examples/a6b_v2_dissection.rs`](../../../examples/a6b_v2_dissection.rs) (`#![forbid(unsafe_code)]`, 100% safe Rust)  
**Corpus:** `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; 431 total blocks, 233 production-admitted Open blocks) + 6 genuine single-cycle synthetic stress blocks (`ns == 0`, 40k spiral `widest = 39,999`, deep/duplicate long words, non-ASCII digrams, equal-width strip cascade, 70k wide spiral `widest = 69,999 > 65,535`) + generation-counter wraparound (`content_gen`, `content_gen8`, `pair_gen`).  
**Verdict:** **STOP** against the frozen pre-data `>= 15.0%` all-3-profile stop test (RULES C2 strictly preserved). Real safe-Rust gains reach **−12.2%..−13.6%** across all three build profiles for `V2_u128` (`−11.2%..−16.1%` for `V3_i7_lazy`, clearing `−15.0%` on the 32B JCC-aligned control at **−16.05%..−16.39%** but stopping at `−11.70%` baseline / `−11.15%` native), translating to only `~2.0–2.7%` of the `#1062` composed `W4` worker stack (`~19–25 ms` 1W / `~2.7–3.5 ms/w` 8W on i9-10900K).

---

## 1. Motivation & Context

In `#1062` (`composed-worker-stack-probe`), combining the four passing worker mechanisms (`#1055` L1d-batched MTF+OutQ16 + `#1046` combined A6b + `#1057` S=16 4-lane forward emission + `#1045` vectored RLE1+CRC) leaves the remaining `W4` worker budget split as:
- `46%` `A7+A9`,
- `29%` `A5+A6a`,
- **`17%` `A6b` (`~155 ms` 1W / `~22 ms/w` 8W on i9-10900K: `~24 ms` setup, `~75 ms` steps, `~55 ms` dict)**,
- `7%` `A6c`.

In `#1046` (`docs/journal/2026-11-26-explore/a6b-combined-redesign.md`, PR `#1053`), `A6b` fell `-27.4%..-28.8%` (`setup -41.7%`, `steps -31.1..-31.8%`, `dict` only `-13.5..-14.5%`). This probe performs a complete sub-phase census of `#1046`'s winning `comb_nobc_soa_gen` and tests whether a second-pass safe-Rust redesign can unlock another `>= 15%` (`>= 23 ms` 1W / `>= 3.3 ms/w` 8W) without regressing `A7`.

---

## 2. Frozen Pre-Data Stop Test (RULES C1a/C2)

Frozen in the header of `examples/a6b_v2_dissection.rs` before running benchmarks:

- **Baseline:** exact `#1046` (`comb_nobc_soa_gen`), paired with `#1057` unmodified SIMD `Reader::wire` (`A6c`) + `S=16` sampled-anchor dynamic 4-lane forward emission (`A7`), warm reusable buffers, 233 admitted Open blocks of `wikidata.bz2`, 7 interleaved repeats pinned to CPU 0 (`taskset -c 0`).
- **Correctness:** `#![forbid(unsafe_code)]`, 100% byte-exact vs `bwt::walk` on all 431/431 wikidata blocks (forced Open) + synthetic stress suite (`spirals = false` and `true`) + generation wraparound.
- **PASS iff** a safe-Rust candidate clears **all three**:
  1. Cuts `#1046` isolated `A6b` median by **`>= 15.0%`** (disjoint `[min..max]` spreads vs `base_1046`) on **all three** builds:
     - `base`: `cargo build --release` (generic x86-64),
     - `nat`: `RUSTFLAGS="-C target-cpu=native"`,
     - `jcc`: `RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp"` (Intel JCC-erratum 32B alignment control);
  2. Cuts full `A6b + A6c + 4-lane A7` (`b67_w` and `C+E_cpu`) with **non-overlapping `[min..max]` spreads** on all three builds;
  3. Zero `A7` emission regression (`A7_delta <= +0.5%` on all three builds).
- **STOP otherwise** (record exact census, ablation ladder, and scoped row in `frontier/dead-ends.tsv`).

Note on measurement under shared VM load: isolated `A6b` (Part 1) executes 5 consecutive passes over the 233 pre-extracted blocks per repeat (~400–450 ms/sample) with a single Linux `/proc/thread-self/schedstat` on-CPU thread clock read (`A6b_cpu`) before and after, keeping `A6b_cpu` spreads within `~±1–2%` even when parallel swarm workers share the VM.

---

## 3. Exact Structural Census of `#1046` `A6b` (233 Admitted Open Blocks)

### Setup & Working Set
- `ns = 755,021` thick strips (avg `3,240.4/blk`), `dom_gaps = 544,612`, `img_gaps = 503,341`, **`total_nodes = 1,802,974`** (avg `7,738.1/blk`, max `12,835`).
- `spirals_blks = 0 / 233` (100% of admitted wikidata blocks have `widest <= CERTIFIED_WIDTH = 32,768`).
- Live `#1046` working set on avg `7,738` nodes: `links` (8B) `61.9 KiB` + `widths` (4B) `31.0 KiB` + `words128` (16B) `123.8 KiB` + `rules` (~3,210 × 16B) `51.4 KiB` = **`268 KiB`** (spills 256 KiB L2!). Shrinking `words` to `u64` (`V3_i7`) drops live footprint to **`209 KiB`** (100% L2-resident).

### Induction Steps (`2,668,970` total)
- `x == y`: **`0` (`0.00%`)**.
- `aw > bw`: `1,049,954` (`39.34%`, adjacent `yp == x`: `0`).
- `aw < bw`: `632,144` (`23.68%`, adjacent `xp == y`: `9`).
- `aw == bw`: `986,872` (`36.98%`, adjacent `xp == y`: `1,291` = `0.13%`).

### `join()` Operand & Result Census (`2,668,970` calls)
- **Empty operands = `38.26%`:** `both0 = 447,386 (16.76%)`, `l0 = 279,706 (10.48%)`, `r0 = 294,132 (11.02%)`.
- `1 + 1` digram joins: `348,292 (13.05%)`.
- Non-empty `<= 8B` results: `1,040,440 (38.98%)`.
- Non-empty `9..=15B` results: `213,982 (8.02%)`.
- **`LONG (>= 16B)` joins:** `393,324 (14.74%)`.
- Total rules created: `747,845` (`354,521` inline promotions `[47.4%]`, `393,324` internal long nodes `[52.6%]`).

### Bands & Dictionary Census (`827,858` bands, `368,867` dict calls)
- Whole-node bands: **`97.37%`** (`split_node = 21,790` = `2.63%`).
- 1-byte literal bands (`word == 0`): `458,991 (55.44%)`; `dict` calls: `368,867 (44.56%)`.

| Spelling Length Class | Calls | Share | Hits (Hit %) | Misses | Evicts |
|---|---:|---:|---:|---:|---:|
| `len == 2` (digram, **100.00% ASCII**) | `84,072` | `22.79%` | `41,430` (`49.28%`) | `42,642` | `5,142` |
| `len 3..=4` | `65,352` | `17.72%` | `40,013` (`61.23%`) | `25,339` | `2,378` |
| `len 5..=8` | `64,670` | `17.53%` | `31,872` (`49.28%`) | `32,798` | `6,068` |
| `len 9..=16` | `47,907` | `12.99%` | `25,965` (`54.20%`) | `21,942` | `2,901` |
| `LONG (>= 17B)` | `106,866` | `28.97%` | `42,552` (`39.82%`) | `64,314` | `7,084` |
| **Total** | **`368,867`** | **`100.0%`** | **`181,832` (`49.29%`)** | **`187,035`** | **`23,573`** |

### Key Structural Surprises in `LONG` / `append_rules`
1. **`gen_cache.pairs` hits ONLY `1,134 / 106,866` (`1.06%`)!** Dropping `pairs` completely eliminates `106.9K` random reads + `105.7K` writes + `8 KiB` L1d pressure with **zero** dictionary bloat (`dictE` unchanged at `187,035`, `longE = 64,314`, `spelling = 9,888.3 KiB`).
2. `memoized` long calls are only `1,499` (`1.40%`, `1,026` hits, `42.2 KiB` wasted copy); `104,233` (`97.5%`) are unmemoized (`40,392` hits).
3. `append_rules` DFS visits (`892,380` total): `lit = 8,971 (1.0%)`, `inline = 354,521 (39.7%)`, `memo = 135,564 (15.2%)`, `expand = 393,324 (44.1%)`; **`max_stack = 16`** across all 233 blocks.

---

## 4. Incremental Ablation Ladder (Round 3 — 12 Arms, 5,340/5,340 Exact)

Before combining mechanisms, Round 3 isolated each lever independently against `base_1046` (7 repeats × 5 passes/rep, CPU 0):

| Arm | Mechanism | `base` `A6b_cpu` (`Δ%`, C) | `nat` `A6b_cpu` (`Δ%`, C) | Finding |
|---|---|---:|---:|---|
| `base_1046` | Exact `#1046` (`comb_nobc_soa_gen`) | `91.24 [90.55..93.00]` | `90.22 [89.28..92.25]` | Reference |
| `U1_setup_memset` | Bulk `words[term1..nodes].fill(0)` + sequential `term1`/`term0` links | `88.39` (`-3.13%`, D) | `90.54` (`+0.35%`, o) | Modest setup cut, alignment-sensitive alone |
| `S5_brless_base` | Single-branch `join_branchless_u128` (no `if left==0`/`if right==0`) | `86.29` (`-5.43%`, D) | `86.57` (`-4.05%`, D) | **Strong clean win** across both builds |
| `S6_brless_lazy` | `S5` + register `(x, y, aw, bw)` lazy sentinel close | `87.89` (`-3.67%`, D) | `85.17` (`-5.60%`, D) | Helps `nat` more than `base` |
| `S7_brless_spl4` | `S6` + split 4B `links0`/`links1` | `84.24` (`-7.67%`, D) | `86.44` (`-4.19%`, o) | Inconsistent across builds |
| `D2_flat_nopairs` | Pre-sized zero-realloc `spelling`, `[u32; 64]` stack, left-spine DFS, `u128` rules, **no `pairs`** | **`82.58` (`-9.49%`, D)** | **`81.91` (`-9.22%`, D)** | **Largest single-phase win (`~-9.3%`)** |
| `D3_s8_flat` | `D2` + separate 32 KiB `Short8` (`len <= 8`) table | `82.71` (`-9.35%`, D) | `82.28` (`-8.80%`, D) | **Loses to `D2`** (32 KiB L1d pressure) |
| `D4_a16k_s8_flat` | `D3` + 64 KiB direct ASCII digram `[u32; 16384]` table | `83.47` (`-8.52%`, D) | `83.36` (`-7.61%`, D) | **Loses harder to `D2`** (64 KiB L1d/L2 thrash!) |
| `V2_lazy_s8` | `U1 + S6 + D3` | `79.96` (`-12.37%`, D) | `81.22` (`-9.98%`, D) | Better without `D3`'s extra table |

**Key lesson from Round 3:** Even though `len == 2` digrams are `22.79%` of dict calls and `100.00%` ASCII, adding a direct 64 KiB digram table (`D4`) or a 32 KiB `Short8` table (`D3`) **loses** to a single unified 16 KiB `contents_4k` table (`D2`) because the extra tables evict `links`, `widths`, `rules`, and `inline16` from 32 KiB L1d / 256 KiB L2.

---

## 5. Second-Pass Architecture Exploration (Rounds 4–7)

Building on Round 3's findings, Rounds 4–7 added five clean safe-Rust mechanisms while keeping total cache footprint minimal:

1. **Zero-call empty-operand branchless `join`:**
   `if (ll + rl < W) | (ll.min(rl) == 0) { return (left + (rl << SHIFT)) | (right << (8 * (ll & MASK))); }`
   Handles `empty + LONG` and `LONG + empty` in the single-branch inline path, eliminating `~200K` out-of-line calls in `u128` (`W=16`) and `~400K` in `u64` (`W=8`).
2. **8-bit generation + 8-bit fingerprint (`gen8 + fp8`) inside the SAME 16 KiB `contents_4k` (`Box<[u32; 4096]>`):**
   Packs `(gen8 << 24) | (fp8 << 16) | code16` (zero extra memory; wraps every 255 blocks), rejecting **99.6% (255/256) of hash collisions** on one compare before ever reading `lens[old]`, `inline16[old]`, or `spelling_buf`.
3. **Zero-bounds-check 64K tables (`inline16_64k`, `lens_64k`, `bands_64k`, `RuleStore`):**
   Eliminates all `Vec::push` capacity branches and bounds checks in `dict` and `steps` (with safe overflow fallbacks) while passing `&inline16_64k[..dict_len]` and `&lens_64k[..dict_len]` directly to `A7`.
4. **64-bit `tag < 8` fast path in `dict_code`:**
   For `58.0%` of dict calls (`len <= 8`), uses a single 64-bit key, 1-multiply hash, and 8-byte compare.
5. **Three node-representation architectures tested head-to-head:**
   - **`V2_u128_{base,lazy}`**: `#1046` 16B `words128` SoA + all 4 mechanisms above.
   - **`V3_i7_{lazy,spl4}`**: **SoA-inline7 `u64` `words64`** (halves `words` from `124 KiB` to `62 KiB`, fitting the entire `steps` working set in 256 KiB L2: `209 KiB < 256 KiB`; spills `8..=15B` inline words to `RuleStore`).
   - **`V4_aos16`**: 16-byte cache-aligned `Node16` AoS packing all 4 links + `width: u16` + 48-bit inline-5 word into one 16B struct (`124 KiB` total, 1 cache line/node, automatic fallback to `V3_i7` when `widest > 65,535`).

### Negative Ablations Confirmed in Rounds 5–6
- **`V4_aos16` (16B AoS inline-5):** Only `−1.48%` (`base`), `−4.93%` (`nat`), `−8.19%` (`jcc`). Shrinking inline width from 7B to 5B pushes 6–7B words into `RuleStore` spills and doubles linked-list pointer stride from 8B to 16B.
- **Register-resident `(wx, wy)` in `eliminate_unified` (Round 6):** Regressed `V3_i7` by `+4.24 ms` (`78.66 -> 82.90 ms`) and `V2_u128` by `+1.84 ms` (`78.85 -> 80.69 ms`) on `base` due to x86-64 GPR exhaustion and 3× `join` loop-body bloat. Reverted in Round 7.

---

## 6. Final Three-Profile Results (Round 7 — 2,682/2,682 Exact Checks PASS)

All 6 arms passed 100% byte-exact verification vs `bwt::walk`: `431` wikidata blocks (forced Open) + `6 × 2` synthetic stress blocks (`spirals = false/true`, including 40k spiral and 70k wide-fallback spiral) + `4` generation-wraparound checks (`content_gen`, `content_gen8`, `pair_gen`) = **2,682/2,682 exact**.

### Part 1: Isolated `A6b` (7 interleaved repeats × 5 passes/rep, 233 admitted Open blocks, CPU 0)

| Arm | `base` `A6b_cpu` `[min..max]` (`Δ%`, C) | `nat` `A6b_cpu` `[min..max]` (`Δ%`, C) | `jcc` `A6b_cpu` `[min..max]` (`Δ%`, C) | i9-10900K proj (`jcc` 1W / 8W) |
|---|---:|---:|---:|---:|
| `base_1046` | `86.66 [86.29..89.42]` (+0.00%) | `88.66 [88.12..90.19]` (+0.00%) | `86.59 [84.65..91.57]` (+0.00%) | `+0.0 / +0.0 ms` |
| `V2_u128_base` | **`75.94 [75.07..82.75]` (−12.37%, D)** | **`76.97 [76.52..79.19]` (−13.19%, D)** | `76.01 [74.37..83.88]` (−12.22%, D) | `+18.9 / +2.7 ms` |
| `V2_u128_lazy` | `78.93 [75.88..83.73]` (−8.91%, D) | `77.21 [76.37..83.96]` (−12.92%, D) | `74.81 [73.54..75.06]` (−13.61%, D) | `+21.1 / +3.0 ms` |
| **`V3_i7_lazy`** | **`76.52 [76.37..80.22]` (−11.70%, D)** | `78.77 [77.57..85.39]` (−11.15%, D) | **`72.70 [71.16..79.04]` (−16.05%, D)** | **`+24.9 / +3.5 ms`** |
| `V3_i7_spl4` | `79.16 [77.52..84.85]` (−8.65%, D) | `81.73 [80.67..86.70]` (−7.82%, D) | `74.35 [73.41..75.20]` (−14.13%, D) | `+21.9 / +3.1 ms` |
| `V4_aos16` | `85.37 [84.05..89.32]` (−1.48%, o) | `84.29 [83.43..91.67]` (−4.93%, o) | `79.50 [78.30..85.16]` (−8.19%, o) | `+12.7 / +1.8 ms` |

*(Round 5 `jcc` confirmation: `V3_i7_opt` achieved `72.10 [71.36..75.10]` vs `base_1046 = 86.23 [85.62..87.65]`, **−16.39%**, disjoint, with `dict` cut **−34.6%** from `91.83 -> 60.03 ms` wall and `setup` cut **−18.1%**.)*

### Sub-Phase Breakdown on `jcc` (Round 7 Part 1 wall medians, ms)
- `base_1046`: `setup 19.42`, `steps 63.57`, `bot 1.42`, `dict 44.13`
- `V2_u128_lazy`: `setup 16.00 (-17.6%)`, `steps 45.79 (-28.0%)`, `bot 1.35`, **`dict 29.49 (-33.2%)`**
- `V3_i7_lazy`: **`setup 13.05 (-32.8%)`**, **`steps 43.27 (-31.9%)`**, `bot 1.69`, `dict 31.26 (-29.2%)`

### Part 2: Full End-to-End `A6b + A6c + 4-lane Forward A7` (7 repeats, 233 admitted Open blocks, CPU 0)

| Arm | `base` `A6b_w` (`Δ%`) / `A7_Δ%` / `b67_w` / `C+E_cpu` (`Δ%`, C) | `nat` `C+E_cpu` (`Δ%`, C) | `jcc` `C+E_cpu` (`Δ%`, C) | `dictE` / `longE` / `spellK` |
|---|---|---|---|---|
| `base_1046` | `89.32` / `+0.00%` / `318.69` / `455.63 [451.40..461.74]` | `444.38 [435.85..464.18]` | `387.47 [385.97..419.23]` | `187,035` / `64,314` / `9,888.3` |
| `V2_u128_base` | `78.44 (-12.18%)` / `+0.04%` / `307.67 (-3.46%)` / `442.90 (-2.79%, o)` | **`427.32 (-3.84%, D)`** | `379.73 (-2.00%, o)` | `187,216` / `64,298` / `9,888.2` |
| `V2_u128_lazy` | `78.59 (-12.02%)` / `-1.06%` / `306.38 (-3.86%)` / **`439.77 (-3.48%, D)`** | `435.12 (-2.08%, o)` | **`378.87 (-2.22%, D)`** | `187,216` / `64,298` / `9,888.2` |
| `V3_i7_lazy` | `78.92 (-11.65%)` / `-1.13%` / `306.04 (-3.97%)` / `439.70 (-3.50%, o)` | `430.02 (-3.23%, o)` | `381.58 (-1.52%, o)` | `187,216` / `64,298` / `9,888.2` |
| `V3_i7_spl4` | `81.33 (-8.95%)` / `+0.53%` / `312.26 (-2.02%)` / **`444.11 (-2.53%, D)`** | `434.98 (-2.11%, o)` | `379.79 (-1.98%, o)` | `187,216` / `64,298` / `9,888.2` |
| `V4_aos16` | `86.88 (-2.74%)` / `+2.85%` / `322.14 (+1.08%)` / `455.49 (-0.03%, o)` | `438.37 (-1.35%, o)` | `387.92 (+0.11%, o)` | `187,216` / `64,298` / `9,888.2` |

---

## 7. Verdict & Engineering Implications

1. **Frozen Stop Test Verdict: `STOP` (RULES C2 strictly preserved).**
   - Criterion 1 (`A6b_cpu <= -15.0%` disjoint across all 3 builds): **`V3_i7_lazy` passes `jcc` (`-16.05%` R7 / `-16.39%` R5, disjoint) but stops at `-11.70%` on `base` and `-11.15%` on `nat`.** `V2_u128_base` / `V2_u128_lazy` are `-12.22%..-13.61%` across all 3 builds (disjoint on all 3, ~`1.5–2.8%` below the `-15.0%` threshold).
   - Criterion 2 (simultaneous disjoint `A6b+A6c+A7` on both wall & CPU across all 3 builds): not cleared simultaneously across all 3 builds (`C+E_cpu` improves `-2.0%..-3.8%` = `-10..-17 ms`, matching the isolated A6b cut with zero A7 regression on idle runs, but sits at the noise floor of 7-repeat wall-clock spreads when A6b is only 17% of `W4`).
2. **Physical ceiling of post-`#1046` `A6b`:**
   After cutting `dict` by `-29%..-35%` and `setup` by `-18%..-33%`, the remaining `~73–77 ms` VM CPU (`~130–136 ms` 1W / `~18.5–19.3 ms/w` 8W on i9-10900K) is dominated by irreducible doubly-linked list pointer updates (`2.67M` random steps) and `append_rules` copying `9.89 MiB` of long spellings into the flat arena.
3. **Priority & optional opportunistic harvest:**
   A standalone second-pass `src/open/endpoint.rs` rewrite is **not** justified ahead of `A7+A9` (`46%` of `W4`) and `A5+A6a` (`29%` of `W4`). However, the zero-complexity `V2_u128` subset (removing dead 1.06%-hit `gen_cache.pairs`, zero-call empty-operand `join`, `gen8+fp8` in the same 16 KiB `contents_4k`, `tag < 8` 64-bit dict fast path, and zero-bounds-check `inline16_64k`/`lens_64k`/`bands_64k`/`RuleStore`) is ~80 lines of 100% safe Rust worth a clean `-12.2%..-13.6%` A6b (`~+19–21 ms` 1W / `~+2.7–3.0 ms/w` 8W on i9-10900K) if folded opportunistically in a future endpoint cleanup.
