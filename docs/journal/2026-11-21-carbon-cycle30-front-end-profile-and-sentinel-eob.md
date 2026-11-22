# Carbon Cycle 30 — 1T & 10T Stage Attribution on `origin/master` (`53f7ecf9`) and Safe-Rust `EOB_FAST` Sentinel Scanner

**Baseline:** `origin/master` at `53f7ecf935d62ac7896eafaf55c5d249789614ca` (post-`v0.5.11`, including `#692`, `#703`, `#714`, `#719`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle30/c30_sentinel_eob_scan.rs` (`#![forbid(unsafe_code)]`).

## 1. Stage Attribution on `origin/master` (`53f7ecf9`) at `1` and `10` Threads

Measured on `wikidata.bz2` (`431` blocks, `383` run-path / `48` column-path, `11.85 MB` compressed $\to$ `209.70 MB` decoded):

### 1a. `1`-Thread Stage Shares (`mibz2::decode` = `Cells`, and `OpenCells`, best of 5 full-file passes in decode order)

| Stage | `1T` (`Cells`) ms | `1T` (`Cells`) Share | `1T` (`OpenCells`) ms | `1T` (`OpenCells`) Share |
|---|---:|---:|---:|---:|
| **1. `scan` (headers + Huffman)** | **79.39** | **16.40%** | **79.18** | **17.03%** |
| **2a. `takes_runs` (sample)** | 0.42 | 0.09% | 0.41 | 0.09% |
| **2b. `runs::front` (MTF + RLE2 runs)** | **41.42** | **8.56%** | **41.05** | **8.83%** |
| **2c. `mtf::mtf` (MTF + RLE2 column)** | 0.02 | 0.00% | 0.02 | 0.00% |
| **[Front-End Subtotal (`1..2c`)]** | **121.25** | **25.04%** | **120.66** | **25.95%** |
| 3. `InverseBwt` (`build + walk`) | 335.75 | 69.35% | 318.06 | 68.41% |
| 4. `RLE1` (`unrle`) | 17.98 | 3.71% | 17.04 | 3.67% |
| 5. `CRC32` (`clmul`) | 9.19 | 1.90% | 9.17 | 1.97% |
| **Total** | **484.16** | **100.00%** | **464.93** | **100.00%** |

### 1b. `10`-Thread Stage & Critical-Path Attribution (`1` Scanner Thread + `10` `OpenCells` Workers, `slots = 40`, per-thread `CLOCK_THREAD_CPUTIME_ID`)

| Stage (`10T`) | Total CPU (ms) | Share of Total `10T` CPU | `10`-Core Pipeline Critical-Path Contribution |
|---|---:|---:|---:|
| **1. `scan` (`1` serial scanner thread)** | **82.27** | **15.27%** | **82.27 ms (`100.0%` of 10-core bound; `1.79×` worker/core)** |
| 2a. `takes_runs` (`10` workers) | 1.71 | 0.32% | 0.17 ms |
| **2b. `runs::front` (`10` workers)** | **47.20** | **8.76%** | **4.72 ms** |
| 2c. `mtf::mtf` (`10` workers) | 0.03 | 0.01% | 0.00 ms |
| **[Front-End Subtotal]** | **131.22** | **24.36%** | — |
| 3. `OpenCells` invert (`10` workers) | 356.54 | 66.19% | 35.65 ms |
| 4. `RLE1` (`10` workers) | 37.26 | 6.92% | 3.73 ms |
| 5. `CRC32` (`10` workers) | 13.68 | 2.54% | 1.37 ms |
| **Sum of 10 Workers (`÷ 10`)** | **460.68** | **84.73%** | **46.07 ms/worker** |

**Key Diagnostic Finding:** Following the `0.5.11` + `#714` + `#719` `OpenCells` improvements, 10-worker decode work is down to **`46.07 ms` per worker**, whereas `scan` (Huffman decode) remains **`82.27 ms` serial on a single thread** (`1.79×` the per-worker budget at 10 workers, and `1.25×` even at 7 workers). **`scan` (Huffman decode) is both the largest front-end stage at `1` thread (`16.40%`) and the `#1` overall critical-path bottleneck at `10` threads.**

### 1c. Sub-Phase Census of `scan` (`block::read_item`) on `wikidata.bz2`
- **Huffman symbols:** `24,979,299` (`39.4%` `RUNA/RUNB`, `60.6%` literals + `EOB`), decoded in `13.71M` steps (`1.822` syms/step: `16.52%` 1-sym, `82.23%` 2-sym pair, `1.243%` `>10`-bit slow).
- **Selectors & Groups:** `499,822` selectors (`1,159.7`/block); **`1,796` Huffman groups declared in headers vs only `1,606` referenced by selectors (`190` groups = `10.6%` never used)**.
- **Internal time split of `scan`:**
  1. `symbol decode loop (GROUP=50)`: `71.26 ms` (`85.7%`)
  2. `Huff::new (LUT + pair build)`: `6.08 ms` (`7.3%`)
  3. `selectors (MTF unary)`: `5.04 ms` (`6.1%`)
  4. `code lengths (delta bits)`: `0.67 ms` (`0.8%`)
  5. `hdr + in_use bitmap`: `0.12 ms` (`0.1%`)

## 2. Cycle 30 Candidate (`ScannerC30`, `#![forbid(unsafe_code)]`)

1. **`EOB_FAST` Sentinel (`COUNT = 0`) in `Huff::build_into`:** Marks `s == eob` LUT entries with `EOB_FAST | zn` (`COUNT = 0`) and excludes `s2 == eob` from 2-symbol pairs, routing the single `EOB` per block (`0.0015%` of steps) to `#[cold] decode_entry_slow` and deleting `(e >> (SYM1 + 9*(c-1)) & 0x1ff) == eob` (variable shift + mask + compare + exit branch) from 100% of hot fast-path lookups.
2. **`STEPS = 5` Lookups per Refill:** Consumes up to `5 × 10 = 50 <= 56` live bits per `Reg::refill`, cutting refill calls by `~20%`.
3. **Zero-Alloc `ScanScratch` + Lazy `used_groups` Build + Canonical `pair_end`:** Eliminates per-block `selectors` and `tables` `Vec` allocations, skips `Huff::build_into` for unreferenced groups (`10.6%` of tables), uses `O(alpha)` bucket scatter for `perm`, and restricts 2-symbol pair construction to canonical prefixes `0..pair_end` where `l1 + min_len <= 10`.
4. **Branchless `lzcnt` Unary Selectors + `u64` MTF Register:** Replaces the bit-by-bit `while br.bit()` + `mtf_g.copy_within` loop with `(!r.buf).leading_zeros()` and single-`u64` MTF word shifts.
5. **Safe Fixed-Window `&mut [u16; 64]` Group Slice:** Sizes `syms` to `sel_cap * 50 + 64` (avoiding `1.8 MB` `MAX_SYMS` zero-fills) and indexes `g_out[k & 63]` in `#![forbid(unsafe_code)]`.

## 3. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, Same Binary)

| Benchmark | `MasterA` (ms) | `MasterB` (ms) | `A/A` (`B/A`) | `Cand` (`C30`) (ms) | `A/B` (`Cand/MasterA`) | Verdict |
|---|---:|---:|---:|---:|---:|---|
| `ScanOnly` (stage 1, 15r) | 76.46 | 76.27 | `0.9975×` | **66.99** | **`0.8761×` (`-12.39%`)** | **KEEP** |
| `1T E2E Full` (`Cells`, 9r) | 493.60 | 494.27 | `1.0013×` | **484.84** | **`0.9823×` (`-1.77%`)** | **KEEP** |
| `1T E2E Full` (`OpenCells`, 9r) | 469.60 | 468.89 | `0.9985×` | **458.47** | **`0.9763×` (`-2.37%`)** | **KEEP** |
| `10T ScannerCPU` (serial, 11r) | 82.10 | 81.88 | `0.9973×` | **73.47** | **`0.8948×` (`-10.52%`)** | **KEEP** |
| `10T CritPath max(S, W/10)` (11r) | 82.10 | 81.88 | `0.9973×` | **73.47** | **`0.8948×` (`-10.52%`)** | **KEEP** |
| `10T VM Wall` (2-vCPU, 11r) | 269.74 | 272.12 | `1.0089×` | **265.61** | **`0.9847×` (`-1.53%`)** | **KEEP** |

**Parity:** `431/431` `wikidata.bz2` blocks symbol-exact + `1,390/1,390` truncated/mutated/header adversarial checks error-exact.

## 4. Kill Decisions

- **Killed — Unmasked VRP `g_out[k]` (`0.8893×` `ScanOnly` vs `0.8761×` masked):** Killed because LLVM fails to propagate `k < 64` across the 5 `if/else` slow-path merge diamonds and re-inserts slice bounds checks (`+1.60 ms`).
