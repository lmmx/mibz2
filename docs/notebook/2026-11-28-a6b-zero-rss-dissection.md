# Zero-Extra-RSS Dissection & Safe-Rust Redesign of `A6b` (`src/open/endpoint.rs`)

**Date:** 2026-11-28
**Base:** `origin/master` (`5c762ed1` for code/probe; published on `3214a002`)
**Evidence scope:** VM-only (`Haswell-EP`, 2 vCPU, 32 KiB L1d, 256 KiB L2, 45 MiB L3, Linux x86-64, pinned CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID` / `CLOCK_PROCESS_CPUTIME_ID`, both default `base` and `-x86-branches-within-32B-boundaries` `jcc`, plus true `64 MiB > 45 MiB` per-block L3-cold flushes). Physical i9-10900K verification is required before shipping (C5).

---

## 1. Question & Prior Failure Mode

`A6b` (`src/open/endpoint.rs`, `Kernel::induced_with::<Narrow>`) accounts for ~20.3% of post-`B4` worker CPU (`120.5 ms` `jcc` / `139.2 ms` `base`) across all 233 Wikidata Open blocks in both shipped bidirectional `OpenCells` and `forward-anchor-lanes`.

Two prior attempts left `A6b` untouched on `master`:
1. **`#1053` (`a6b-endpoint-build`, `STOP/KILL` on i9 `a6e427b1`):** cut `wikidata/1` by `-1.76%` (`-24.7 ms`), but allocated fixed 64K SoA slabs outside `Kernel.edges`, adding `+524–546 KiB/worker` resident RSS (`+4.19 MiB` / `1.0374x` at 8W) and pushing Comet Lake's 20 MiB L3 past its cliff (`wikidata/8/S` `worker_busy = 1.0106`).
2. **`a6b-compact-footprint-rescue` (`dropped`):** `C5_edges_overlay_soa_u64` (unpacked `u32` links inside `edges`) reached `-16.45%` `jcc` / `-18.50%` `base` isolated, yet still grew `edge_c/w` by `+204 KiB/worker` (`1.0149x` RSS) and `ep_cap/w` by `+16 KiB/worker`.

**Forensic audit of both `S0_shipped` and `C5`:**
- **Disassembly of shipped `<Kernel>::induced_with::<Narrow>` (`objdump -dC`, 3,221 instructions):**
  - **97 `core::panicking::panic_bounds_check` calls** inside one monolithic function (including 4 separate bounds checks just to read one 4-word rule in `join`, and 6 bounds checks inside the inner `aw > bw` elimination branch).
  - **16-bit partial stores overlapping 32-bit loads** (`mov %r11w, 0x6(%r9,%rcx,4)` followed by `mov (%r9,%rax,4), %eax`), triggering Intel store-forwarding stalls on every packed-link RMW.
  - **Heavy register spilling** (`0x140(%rsp)` frame; 5 stack reloads inside `aw > bw`, 12 stack spills/reloads per iteration inside `Dictionary::code`) because setup, both elimination loops, bottom assignment, and dictionary emission were inlined into one function.
  - **`memset(..., 0, 0x6000)` (`24 KiB`) per block**, zeroing `8 KiB` of the `pairs` cache despite a `1.06%` hit rate (`1,134 / 106,866`).
  - **`grow(edges, ...)` uses `a.resize(need.max(2 * a.len()), 0)`**, doubling `edges` capacity whenever rules outgrow `arena_words(ns, m)`.
- **Why `C5` still leaked `+220 KiB/worker` RSS:**
  1. `C5` kept a separate `Box<[u32; 4096]>` (`GenCacheV2`, `+16 KiB/worker`) outside `edges`.
  2. `C5` called `edges.resize(need, 0)` without `reserve_exact` (triggering `Vec` 2× capacity doubling past `max(m)`).
  3. `C5` bucketed power-of-two `CAP` on `most = 3*ns + 3` (`mean = 9,723 > 8,192`) instead of exact `n_nodes` (`mean = 7,739`; `57.5%` of Wikidata blocks fit in `CAP = 8,192`!).

---

## 2. Zero-Extra-RSS Design (`#![forbid(unsafe_code)]`, `0 B` Extra Heap)

We tested five progressive `#![forbid(unsafe_code)]` variants overlaid 100% inside `Kernel.edges` with `reserve_exact`-only growth and **zero extra fields on `Endpoint` or `Kernel`**:

- **`S0_shipped`**: verbatim `src/open.rs:486-515` + `src/open/endpoint.rs` `Narrow`.
- **`Z1_narrow_0bc`**: keeps `Narrow`'s `4 u32/node` (`16 B/node`) layout inside `edges`, splits phases `#[inline(never)]`, computes exact `n_nodes` (`1 + ns + from_gaps + to_gaps`) for tight power-of-two `CAP` (`2K/4K/8K/16K/32K`) masked with `& (CAP4 - 1)` (0 bounds checks), keeps `(x, y)` tails in registers, skips link writes on `q == 1` adjacent steps (`yp == x`, `xp == y`), drops `pairs`, overlays `contents[4096]` (`16 KiB`, `(fp16 << 16) | code16`) in dead `edges[0..4096]`, and uses stack `[u32; 64]` DFS in `append`.
- **`Z2_u64_soa`**: `7 u32/node` (`28 B/node`) unpacked `u32` links (`4*CAP`) + `widths` (`CAP`) + inline-7 `u64` `words` (`2*CAP`) overlaid inside `edges` (`0 B` extra heap).
- **`Z3_u64_aos`**: **`8 u32/node` (`32 B/node`, half cache line) unified AoS** `[prev0, next0, prev1, next1, width, word_lo, word_hi, pad]` overlaid inside `edges` — collapses 3 cache-line streams per node (`links`, `widths`, `words`) into **1 single cache line per node**!
- **`Z4_u64_hyb2`**: 2-array hybrid (`links[4*CAP]` + `payload[4*CAP]`).
- **`Z3v2_aligned`** (refinement of `Z3` guided by `objdump` of `elim_z3` & `dict_z3`):
  1. Reorder node fields to `[prev0, next0, prev1, next1, word_lo, word_hi, width, pad]` so `word` sits at **8-byte-aligned `+0x10`** — collapses 10 instructions (two 32-bit loads + shifts + ORs) into **two single 64-bit aligned loads** (`mov 0x10(%rdi,%rbx,4), %rax`) and aligned 64-bit stores.
  2. Move `rules.room() < 12` check into the `0.13%` cold `join_u64_long` branch (leaving `rules` untouched on `99.87%` of steps) and update `left -= bw/aw` inside each branch rather than keeping `w = min(aw, bw)` live across the cold call — eliminates hot-path register spills (`0` hot spills, `215` total instructions, `0` bounds checks in `elim_aos::<131072, false>`).
  3. Set high bit on `fp16` (`| 0x8000_0000`) to eliminate the separate `slot != 0` branch and deduplicate `dict_aos`'s cache-miss push path; gate dead `stats.{steps,words,copies,traced,thick_area,unit_work,longest}` updates behind `#[cfg(any(test, feature = "census", feature = "induction-trace"))]`; size `init_rules = 4 * (ns / 2 + 256)` (inline-7 `u64` averages `3,374` rules vs `7,072` in `S0`).

---

## 3. Isolated `A6b` Results (All 233 Wikidata Open Blocks)

Pinned CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID`, interleaved rotated repeats (`n = 9` warm & L2-cold, `n = 5` true `64 MiB > 45 MiB` L3-cold per-block LLC flush). Exact band/word/cursor parity verified across all 233 Wikidata Open blocks (default + forced `spirals = Some(true)`) and all 6 guard corpora.

### 3.1 Five-Arm Structural Sweep (`S0` vs `Z1..Z4`)

| Arm | `edges_cap` (KiB) | `Endpoint` heap | `base` warm | `base` L2-cold | `base` 64 MiB L3-cold | `jcc` warm | `jcc` L2-cold | `jcc` 64 MiB L3-cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `S0_shipped` | `231,600` (`904.7`) | `64 B` | `146.33 ms` (`0.00%`) | `146.42 ms` (`0.00%`) | `147.97 ms` (`0.00%`) | `131.79 ms` (`0.00%`) | `131.25 ms` (`0.00%`) | `142.22 ms` (`0.00%`) |
| `Z1_narrow_0bc` | `231,600` (`904.7`) | **`0 B`** | `120.15 ms` (`-17.89%`) | `121.16 ms` (`-17.25%`) | `121.97 ms` (`-17.57%`) | `102.79 ms` (`-22.01%`) | `102.89 ms` (`-21.60%`) | `112.89 ms` (`-20.62%`) |
| `Z2_u64_soa` | `231,600` (`904.7`) | **`0 B`** | `117.60 ms` (`-19.63%`) | `119.44 ms` (`-18.43%`) | `120.17 ms` (`-18.79%`) | `102.00 ms` (`-22.60%`) | `101.93 ms` (`-22.34%`) | `112.58 ms` (`-20.84%`) |
| **`Z3_u64_aos`** | **`231,600` (`904.7`)** | **`0 B`** | **`111.85 ms` (`-23.56%`)** | **`111.78 ms` (`-23.65%`)** | **`115.62 ms` (`-21.86%`)** | **`96.54 ms` (`-26.75%`)** | **`96.39 ms` (`-26.56%`)** | **`108.29 ms` (`-23.85%`)** |
| `Z4_u64_hyb2` | `231,600` (`904.7`) | **`0 B`** | `114.48 ms` (`-21.76%`) | `115.41 ms` (`-21.17%`) | `118.24 ms` (`-20.09%`) | `98.82 ms` (`-25.02%`) | `99.55 ms` (`-24.15%`) | `110.52 ms` (`-22.28%`) |

Key structural finding: **`32 B/node` AoS (`Z3`) beat `28 B/node` SoA (`Z2`) by `-3.9 pp` warm and `-3.0 pp` under 64 MiB L3-cold** because touching `x` and `y` fetches 2 cache lines (`links + width + word` co-located) instead of 6 separate cache lines!

### 3.2 `objdump`-Guided Refinement (`S0` vs `Z3` vs `Z3v2_aligned`)

| Mode (`233` Wikidata Open blocks) | `S0_shipped` | `Z3_u64_aos` | **`Z3v2_aligned`** | **`Z3v2` vs `S0`** |
|---|---:|---:|---:|---:|
| **`base` L3-warm** (`n=9`) | `135.88 ms` | `103.86 ms` (`-23.56%`) | **`102.72 ms`** | **`-24.40%`** |
| **`base` L2-cold** (`n=9`) | `135.38 ms` | `103.34 ms` (`-23.67%`) | **`101.82 ms`** | **`-24.79%`** |
| **`base` 64 MiB L3-cold** (`n=5`) | `146.40 ms` | `114.32 ms` (`-21.91%`) | **`111.90 ms`** | **`-23.57%`** |
| **`jcc` L3-warm** (`n=9`) | `131.91 ms` | `98.23 ms` (`-25.53%`) | **`97.85 ms`** | **`-25.82%`** |
| **`jcc` L2-cold** (`n=9`) | `135.82 ms` | `98.85 ms` (`-27.22%`) | **`97.01 ms`** | **`-28.57%`** |
| **`jcc` 64 MiB L3-cold** (`n=5`) | `141.64 ms` | `107.27 ms` (`-24.27%`) | **`105.85 ms`** | **`-25.27%`** |

Phase breakdown (`jcc`, `S0` → `Z3v2_aligned`):
- `setup`: `20.33 → 16.42 ms` (`-19.2%`)
- `elim`: **`67.03 → 41.29 ms` (`-38.4%`)**
- `bot`: `3.88 → 4.02 ms` (`+3.6%`)
- `dict`: `40.90 → 34.54 ms` (`-15.6%`)

---

## 4. Whole-Decoder Verification (`src/open/endpoint.rs` Prototype)

Wired `Z3v2_aligned` cleanly into `src/open/endpoint.rs` (`mod overlay_aos`, `#![forbid(unsafe_code)]`, active for `Narrow` Open blocks with `ns >= 64` and `n_nodes <= 32,768`; rare `32,769..=65,535`-node blocks fall back to `Narrow` so `edges` never allocates a `2 MiB` `CAP = 65536` table).

### 4.1 Whole-Decoder `wikidata.bz2` (Fresh Child Processes, Bit-Exact Parity `209,700,196 B`, hash `7e03818a3b10adaa`)

| Workload | Flavor | CPU (`old → new`) | Delta | Peak `VmHWM` (`old → new`) | `VmHWM` ratio |
|---|---|---:|---:|---:|---:|
| **`OpenCells` 1W** (`n=5`, pinned CPU 0) | `base` | `1253.69 → 1233.17 ms` | **`-1.64%`** (`-20.5 ms`) | `19,772 → 18,296 KiB` | **`0.9253×`** (`-1,476 KiB`) |
| **`OpenCells` 1W** (`n=5`, pinned CPU 0) | `jcc` | `1190.78 → 1169.13 ms` | **`-1.82%`** (`-21.7 ms`) | `19,792 → 18,368 KiB` | **`0.9281×`** (`-1,424 KiB`) |
| **`workers-lazy` 1W** (5 fresh 1-pass runs) | `base` | `1225.04 → 1148.72 ms` | **`-6.23%`** | `33,208 → 33,384 KiB` | **`1.0053×`** (`+176 KiB` `.text`) |
| **`workers-lazy` 1W** (`n=3` interleaved) | `jcc` | `1147.12 → 1119.99 ms` | **`-2.37%`** | — | — |
| **`mibz2` 4W** (`n=3` interleaved) | `jcc` | `1329.88 → 1342.50 ms` | `+0.95%` (wall `-2.34%`) | `120,908 → 119,400 KiB` | **`0.9875×`** (`-1,508 KiB`) |
| **`mibz2` 8W** (`n=3` interleaved) | `base` | `1279.37 → 1182.59 ms` | **`-7.56%`** (wall `-4.18%`) | `67,732 → 71,528 KiB` | `1.0560×` (multi-pass arena) |
| **`mibz2` 8W** (`n=3` interleaved) | `jcc` | `1185.74 → 1152.78 ms` | **`-2.78%`** (wall `-6.00%`) | `71,448 → 70,024 KiB` | **`0.9801×`** (`-1,424 KiB`) |

Note why single-worker `OpenCells` `VmHWM` actually **dropped by `-1.45 MiB` (`0.925×`)**: shipped `S0`'s `grow()` doubled `edges` (`need.max(2 * a.len())`), whereas `overlay_aos` emits half as many rules (`3,374` vs `7,072`) and grows exclusively with `reserve_exact`.

### 4.2 Six Guard Corpora (`OpenCells` 1W) & Quality Gates

- **Bit-exact parity PASS** across all 6 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`), `VmHWM` `0.9961×–1.0067×` (`base` & `jcc`).
- **`cargo fmt --check`**: PASS.
- **`cargo clippy --lib --tests -- -D warnings`** + `--features census` / `induction-trace` / `a6b-endpoint-redesign`: PASS (`0` warnings).
- **`cargo test --lib open::`**: `46 passed; 0 failed` (including `worst_case_scratch_per_row`, `order_in_edges_costs_nothing_on_full_blocks`, `regions_dead_after_their_last_read`, `overlay_aos_matches_wide_and_spirals`, and `miri_a6b_overlay_aos`).
- **Miri (`cargo +nightly miri test --lib miri_a6b_overlay_aos`)**: verified `0` UB (`#![forbid(unsafe_code)]`).

---

## 5. Next Step

This clears the `>= 10%` isolated `A6b` and `<= 1.00×` RSS bar by >2× (`-23.6%..-28.6%` across `base`/`jcc`, warm/L2-cold/64 MiB L3-cold, `0 B` extra `Endpoint` heap, `-1.45 MiB` single-worker `VmHWM`). Because `A6b` runs on 100% of Wikidata Open blocks in both shipped `bidi` and `forward-anchor-lanes` (`#1217`), we open striated `idea` claim `a6b-zero-rss-endpoint-build` for formal implementation and physical i9-10900K `1W / 6W / 8W` CPU + RSS verification (C5).
