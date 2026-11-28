# Post-`#1328` `A6b` (`Endpoint + Dict`) Residual Dissection & Zero-RSS Fast Build

**Date:** 2026-11-28
**Code base dissected:** `#1328` (`37a2a37a`, `overlay_aos` in `src/open/endpoint.rs`), paired with `#1312` (`--features forward-anchor-lanes`)
**Evidence scope:** VM-only (`Haswell-EP`, 2 vCPU, 32 KiB L1d, 256 KiB L2, 45 MiB L3, Linux x86-64, pinned CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID` and per-child `wait4` process CPU, both portable `base` and `JCC-32B` `-x86-align-branch-boundary=32 -x86-align-branch=fused+jcc+jmp`, warm and true `64 MiB > 45 MiB` per-block L3-cold flushes). Physical i9-10900K verification is required before shipping (C5).

---

## 1. Question & Context

In `#1362`'s composed `M5` worker stack, `#1328` (`overlay_aos`) cut `A6b` by ~22% (`125 → 98 ms` `base`, `112 → 87 ms` `jcc`), yet `A6b` remains `18.0–18.6%` of total worker CPU (`#3` overall after `A7` and `A5+A6a`). This note answers three questions on all 233 Wikidata Open blocks paired with `--features forward-anchor-lanes`:

1. Where do the remaining `~98–118 ms` inside post-`#1328` `A6b` actually go (`prepare_header`, `setup_aos`, `elim_aos`, `assign_bottom_aos`, `dict_aos`)?
2. Does `--features forward-anchor-lanes` waste any work inside `A6b` (e.g. building `inline_back` or bidirectional tables)?
3. What `#![forbid(unsafe_code)]`, `0 B` extra-RSS improvements cut residual `A6b` by `>= 10%` across both `base` and `JCC-32B` (warm and 64 MiB L3-cold) without guard or RSS regressions?

---

## 2. Sub-Stage Profile, Micro-Counters & Disassembly Audit of `37a2a37a`

### 2.1 Forward-Anchor-Lanes Audit

- **No wasted bidirectional dictionary work:** `src/open/endpoint.rs` (`overlay_aos` and `Dictionary`) builds only forward `inline16`, `lens`, and `spelling`; there is no `inline_back` or reverse table. `src/open.rs` (`wire_fwd_edges` vs `wire_xor_edges`) already skips XOR wiring whenever `lanes::fits(...)` holds.
- **Code-budget headroom:** Across all 233 Wikidata Open blocks, `mean codes = 1,058.6` (`max = 1,712`), leaving `>= 6,480` unused codes below `forward-anchor-lanes`'s bit-31 anchor-tag ceiling (`((u32::MAX >> 1) >> bits)`).

### 2.2 Exact Sub-Stage Split (`37a2a37a`, 233 Wikidata Open Blocks, `CLOCK_THREAD_CPUTIME_ID`)

| Sub-stage | `base` warm (`n=11`) | Share | `base` 64 MiB L3-cold (`n=5`) | `JCC-32B` warm (`n=11`) | Share | `JCC-32B` 64 MiB L3-cold (`n=5`) |
|---|---:|---:|---:|---:|---:|---:|
| **`dict_aos`** | **`47.00 ms`** | **40.0%** | **`49.70 ms`** | **`47.70 ms`** | **41.4%** | **`47.79 ms`** |
| **`elim_aos`** | **`46.42 ms`** | **39.5%** | **`48.23 ms`** | **`43.53 ms`** | **37.7%** | **`44.06 ms`** |
| `setup_aos` | `14.84 ms` | 12.6% | `19.23 ms` | `13.25 ms` | 11.5% | `17.73 ms` |
| `prepare_header` | `4.51 ms` | 3.8% | `7.76 ms` | `4.68 ms` | 4.1% | `9.02 ms` |
| `assign_bottom` | `3.39 ms` | 2.9% | `3.25 ms` | `3.92 ms` | 3.4% | `2.93 ms` |
| **Total `A6b`** | **`117.56 ms`** | **100%** | **`129.42 ms`** | **`115.32 ms`** | **100%** | **`122.97 ms`** |

Surprise #1: **`dict_aos` is the #1 largest sub-stage in post-`#1328` `A6b` (`40.0–41.4%`, `47.0–47.7 ms`)**, ahead of `elim_aos` (`37.7–39.5%`, `43.5–46.4 ms`).

### 2.3 Exact Micro-Counters (233 Wikidata Open Blocks)

- **Geometry:** `CAP` histogram `[2K, 4K, 8K, 16K, 32K, >32K] = [1, 11, 122, 99, 0, 0]`; `mean ns = 3,240.4`, `mean nodes = 7,738.1`, `mean m = 114,024.0`, `mean rules = 3,374.0`, `mean bands = 3,553.0`, `mean spelling = 43,457 B`, `spirals blocks = 0`, `elim_grows = 5`. `prepare_header_exact_nodes` runs its 2nd pass (`to_gaps`) on `178 / 233` (`76.4%`) blocks.
- **`elim_aos` (`2,668,975` total steps, `11,454.8/blk`):**
  - `aw > bw`: `39.34%` (`1,049,954`), `aw < bw`: `23.68%` (`632,144`), `aw == bw`: `36.98%` (`986,872`), `x == y`: `0`.
  - Endpoint reuse across consecutive steps: `x` reused `39.39%`, `y` reused `23.69%` (`63.08%` total).
  - Joins (`2,668,970`): `zero = 38.26%` (`1,021,224`), `inline < 8 = 37.03%` (`988,409`), **`inline 8..15 = 9.97%` (`266,013`), `long >= 16 = 14.74%` (`393,324`)**. Notably, `join_u64_long` runs on **`24.71%`** (`659,337`) of all elimination steps (not `0.13%`).
- **`dict_aos` (`827,858` total bands, `816,102` nodes visited = `1.014` bands/node):**
  - `nwd == 0`: `458,991` (`55.44%`).
  - `tag 1..7`: `214,094` (`25.86%`; `tag == 1` alone is **`84,072` = `39.3%`** of all `tag 1..7`!), `52.9%` hit rate (`113,312` hits, `100,782` misses, `13,681` evictions, `4` false `fp16`, `17` true duplicates missed).
  - `tag 8..15`: `47,907` (`5.79%`, `54.2%` hit rate).
  - `tag >= 16` (long rules): `106,866` (`12.91%`, `39.8%` hit rate = `42,585` hits, `64,281` misses, `2,633` already memoized). **`append_fast` expands `10,244,526 B` (`95.9 B/call`, `max_len = 8,693`) into `spelling`, and `spelling_hash` hashes all `10.24 MB` byte-by-byte with serial FNV-1a (`imul` 3-cycle latency × 10.24M bytes ≈ 31M cycles ≈ `10–13 ms`)!**
  - Consecutive identical `nwd != 0` bands: `35,818` (`4.33%` of all bands).

### 2.4 Disassembly Audit (`objdump -dC`, `37a2a37a` `base`)

| Function | Insns | Panic / BC | `(%rsp)` refs | `vzeroupper` | Key findings |
|---|---:|---:|---:|---:|---|
| `prepare_header_exact_nodes` | `174` | `7` | `17` | `0` | 5 bounds checks per strip (`strips_raw[3*i..+2]`, `img_u32[2*pos..+1]`) |
| `setup_aos::<65536>` | `345` | `4` | `31` | `0` | 2 bounds checks per strip (`strips_raw[3*i]`, `img_u32[2*pos]`), repeated `base(tail)` recomputation |
| `elim_aos::<65536, false>` | `214` | `4` | `8` | `0` | `bw` spilled to `0x20(%rsp)` on 100% of steps because `left_ref: &mut u32` occupies `%rdx`; `join_u64_long` spills `%rdi/%rsi/%rdx` on 24.71% of steps |
| `assign_bottom_aos::<65536>` | `67` | `2` | `7` | `0` | Clean 2-ended walk |
| **`dict_aos::<65536>`** | **`721`** | **`9`** | **`132`** | `0` | Monolithic 10-arg function inlining `tag<8`, `tag<16`, `tag>=16` (`append_fast` + FNV-1a), `inline16`/`lens` bounds checks on every cache hit (`181K` hits), `132` stack spill/reload refs |
| `join_u64_long` | `245` | `0` | `0` | `0` | Marked `#[cold]` despite 24.71% call rate |
| `EdgesRules::append_fast` | `238` | `2` | `43` | `0` | No upfront `dest.reserve(total_len + 1)`; `rec[0..4]` indexed separately |

---

## 3. Structural Optimizations & Negative Results (All `#![forbid(unsafe_code)]`, `0 B` Extra RSS)

### 3.1 Winning Mechanisms (`S1_setup` + `D2_dict_ctx`, `0 B` Extra Heap)

1. **Per-`ch` disjointness in `dict_aos` (`D2_dict_ctx`):** Because `for ch in 0..256` visits each trailing byte strictly once, words emitted in distinct `ch` buckets can **never** match. Partitioning the *existing* overlaid `16 KiB` `edges[0..4096]` scratch into three per-`ch` generation-tagged L1d tables (`13 KiB` total, `0 B` extra heap/RSS):
   - `two_byte[256]` (`1 KiB`, `3072..3328`): direct `(ch_tag16 | code)` lookup for `tag == 1` (`84,072` bands = `39.3%` of `tag 1..7`) — **0 hashing, 0 `lens`/`inline16` verify loads, 0 bounds checks**.
   - `short_tab[512 × 4 u32]` (`8 KiB`, `0..2048`): key-in-slot `[nwd_lo, nwd_hi, ch_tag16 | code, 0]` for `tag 2..7` (`130,022` bands) — 1-multiply hash on `nwd` directly (no `ch` shift on hit), **0 `lens[old]` / `inline16[old]` verify loads, 0 bounds checks**.
   - `long_tab[1024]` (`4 KiB`, `2048..3072`): `((ch + 1) << 23) | fp7 | code16` for `tag >= 8` (100% L1d resident).
   - 1-entry `(last_nwd, last_code)` register cache per `ch` (`4.33%` consecutive identical `nwd` hits in 1 cycle).
2. **O(1) 4-word sampled hash for `long` words (`bytes.len() >= 17`):** Replaces 1-byte serial FNV-1a over `10.24 MB` with 4-word sampled mixing (`len`, `bytes[0..8]`, `bytes[len/2-4..len/2+4]`, `bytes[len-8..len]`). Cache hits still verify full byte equality via `long_word == bytes`, preserving 100% soundness (`max_codes` `1,712 → 1,721`, well inside the `8,191` `forward-anchor-lanes` limit).
3. **Single-pointer `DictCtx` + outlined `push_short` / `resolve_ge8` + `append_fast` reserve:** Packs `(rules, long_tab, inline16, lens, spelling, stack)` behind one pointer, shrinking `dict_aos::<65536>` from `721 → 379` instructions (`-47%`, `9 → 2` panics/BC), and calls `dest.reserve(total_len + 1)` once in `append_fast`.
4. **Zero-BC `chunks_exact` setup (`S1_setup`):** Replaces indexed `3*i` / `2*pos` loops in `prepare_header_exact_nodes` (`7 → 1` BC, `17 → 8` `rsp_refs`) and `setup_aos` (`4 → 0` BC) and caches `ord0_tail_b` / `ord1_tail_b` / `term*_tail_b`.

### 3.2 Negative Structural Findings (Killed — Do Not Retry)

1. **`16 B/node` inline-5 AoS (`V2_16b_in5`, `CAP4`, 100% 256 KiB L2-resident):** Halving node size cut L3-cold `setup` by `-3.0..-4.0 ms` (`19.23 → 15.20 ms` `base`, `17.73 → 14.88 ms` `jcc`), **confirming** L2 residency helps setup — **but `elim` regressed by `+8.3..+10.5 ms`** (`46.42 → 56.53 ms` `base`, `43.53 → 51.81 ms` `jcc`, `44.06 → 53.61 ms` `jcc` L3-cold) due to 32-bit packed-`u16` link RMW + `width` bit-shift overhead on the serial pointer-chase critical path and spilling `tag 6..7` into `rules`.
2. **`32 B/node` packed-`u16`-link inline-15 `u128` AoS (`C2_u128_all`):** Eliminates all `266K` `inline 8..15` rules and `47.9K` `get_inline` loads, yet regressed `elim` by `+4–6 ms` vs `32 B` `u64` AoS (`-9.54%` `base` / `-7.21%` `jcc` total vs `-13.80%` for `D1` alone) due to `u128` register pressure and packed link RMW.
3. **Register-caching `(wx, wy, aw, bw)` in `elim_aos` (`E1_elim_u64`):** Exceeded x86-64's 16 GPRs (`214 → 299` insns, `8 → 15` `rsp_refs`), regressing `+3.84%` on `JCC-32B` warm. Even lean `E1b` (`if wx == 0 ... else if wy == 0 ...` + non-cold `join_u64_long`) was mixed (`-2.1%..-3.2%` `JCC-32B` but `+4.0%` `base` L3-cold), so `#1328`'s original branchless `elim_aos` was kept intact.
4. **Over-reserving `bands` (`bands.reserve(nodes_used + 256)`):** Bloated `bands` capacity from `~48 KiB` to `256 KiB/worker` (`+2.05 MiB` at `8W`), regressing `8W` `VmHWM` (`1.0334×`) and L2 residency. Leaving `bands` naturally sized restored `8W` `VmHWM` to `1.0019×`.
5. **Per-call `dest.resize(start + total_len + 16, 0)` in `append_fast`:** Added `10.24 MB` of zeroing per decode and hurt E2E CPU; plain `dest.reserve(total_len + 1)` won cleanly.

---

## 4. Benchmark Results

### 4.1 Isolated `A6b` (All 233 Wikidata Open Blocks, `CLOCK_THREAD_CPUTIME_ID`, Exact Parity, `edges_cap = 904.7 KiB`)

| Arm (`0 B` extra heap) | `base` warm (`n=11`) | `base` 64 MiB L3-cold (`n=5`) | `JCC-32B` warm (`n=11`) | `JCC-32B` 64 MiB L3-cold (`n=5`) |
|---|---:|---:|---:|---:|
| `B0_post1328` (`37a2a37a`) | `117.56 ms` (`+0.00%`) | `129.42 ms` (`+0.00%`) | `115.32 ms` (`+0.00%`) | `122.97 ms` (`+0.00%`) |
| `E1b_elim_lean` | `117.61 ms` (`+0.04%`) | `134.65 ms` (`+4.04%`) | `111.64 ms` (`-3.19%`) | `120.44 ms` (`-2.06%`) |
| **`D2_dict_ctx`** | **`100.61 ms` (`-14.42%`)** | **`116.24 ms` (`-10.19%`)** | **`97.64 ms` (`-15.33%`)** | **`107.79 ms` (`-12.35%`)** |
| **`V1_32b_combo` (`S1+D2+E1b`)** | **`100.63 ms` (`-14.40%`)** | **`116.06 ms` (`-10.32%`)** | **`97.20 ms` (`-15.72%`)** | **`108.39 ms` (`-11.86%`)** |
| `V2_16b_in5` (`16 B/node`) | `111.81 ms` (`-4.89%`) | `123.44 ms` (`-4.62%`) | `105.67 ms` (`-8.37%`) | `115.75 ms` (`-5.88%`) |

Uninstrumented clean L3-warm medians (`n=11`): `B0` `116.96 → 100.68 ms` (`-13.92%` `D2`) / `99.85 ms` (`-14.63%` `V1`) on `base`, and `109.89 → 96.71 ms` (`-12.00%` `D2`) / `95.77 ms` (`-12.85%` `V1`) on `JCC-32B`.

Sub-stage shift (`B0 → D2`, `JCC-32B` warm): `dict` dropped **`47.70 → 32.18 ms` (`-15.52 ms` = `-32.5%` of `dict`)**; `S1` cut `prep + setup` **`17.93 → 16.42 ms` (`-1.51 ms` `JCC`, `-2.66 ms` `base`)**.

### 4.2 Whole-Decoder E2E (`a6b_endpoint_run`, `--features forward-anchor-lanes`, Paired Fresh Processes `n=5`, `37a2a37a` → `S1 + D2`)

| Workload | Flavor | `2×`-decode Process CPU (`B0 → C2`) | Paired median | Peak `VmHWM` (`B0 → C2`) | `VmHWM` ratio | SHA-256 |
|---|---|---:|---:|---:|---:|---|
| `wikidata` 1W (pinned CPU 0) | `base` | `2612.9 → 2585.4 ms` | `-0.25%` (`3/5` wins) | `35,272 → 35,412 KiB` | `1.0040×` | `3f075e30f4f8` PASS |
| `wikidata` 1W (pinned CPU 0) | `JCC-32B` | `2480.4 → 2480.2 ms` | `+0.60%` (`2/5` wins) | `35,416 → 35,488 KiB` | `1.0020×` | `3f075e30f4f8` PASS |
| **`wikidata` 8W** | `base` | `2650.6 → 2621.2 ms` | **`-1.34%`** (`3/5` wins) | `62,164 → 62,816 KiB` | `1.0105×` | `3f075e30f4f8` PASS |
| **`wikidata` 8W** | `JCC-32B` | `2593.4 → 2579.2 ms` | **`-2.56%`** (`4/5` wins) | `62,216 → 62,336 KiB` | **`1.0019×`** | `3f075e30f4f8` PASS |

- **All 6 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`), `1W` `base` & `JCC-32B`:** `12/12` exact SHA-256 and byte-count parity PASS; `VmHWM` `0.8789×–1.0076×` across all corpora (one noisy `onebyte base` `1.0321×` vs `1.0001×` `jcc`, non-Open path).
- **Quality gates on local prototype (`build/a6b-post1328-residual`):** `cargo fmt --check` PASS; `cargo test --lib open::` **46/46 PASS** (`0` failed, including `worst_case_scratch_per_row`, `overlay_aos_matches_wide_and_spirals`, `miri_a6b_overlay_aos`); `cargo clippy --lib --tests -- -D warnings` PASS across `default`, `forward-anchor-lanes`, `census`, and `induction-trace` (`0` warnings).

---

## 5. Next Step

`S1_setup + D2_dict_ctx` clears the `>= 10%` residual `A6b` bar across all four quadrants (`-14.4%` `base` warm, `-10.2%` `base` 64 MiB L3-cold, `-15.3%` `JCC-32B` warm, `-12.4%` `JCC-32B` 64 MiB L3-cold; `-1.34%..-2.56%` `8W` E2E CPU) at `0 B` extra heap/RSS (`1.0019×` `8W jcc` `VmHWM`) and `#![forbid(unsafe_code)]`. Filed P2 `idea` claim `a6b-post1328-fast-build` (`frontier/claims/a6b-post1328-fast-build.toml`), queued behind `#1328` (`a6b-zero-rss-endpoint-build`), and recorded the killed `16 B` inline-5 / `32 B` packed-`u128` `elim_aos` layouts in `frontier/dead-ends.tsv`.
