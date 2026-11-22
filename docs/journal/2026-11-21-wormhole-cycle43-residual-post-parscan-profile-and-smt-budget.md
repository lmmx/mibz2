# Wormhole Cycle 43 (Neon) — Residual Post-`ParScan` Stage Profile (`95.1%` Pure Parallel), `M1/M2` Kills, & Comet Lake SMT Core-Budget Model

**Baseline:** `C42` `ParScan`-coupled `P2` (`BaseA` vs `BaseB` A/A control) on `wikidata_431b_210m.bz2` (`211.0 MB`, `431` blocks).
**Candidate harness:** [`c43_residual_smt_ab.rs`](2026-11-21-wormhole-cycle43/c43_residual_smt_ab.rs) (`safe_c43` module `#![forbid(unsafe_code)]`), raw log [`c43_results.txt`](2026-11-21-wormhole-cycle43/c43_results.txt), manifest [`c43_residual_smt.toml`](2026-11-21-wormhole-cycle43/c43_residual_smt.toml).

---

## 1. Exact Residual Post-`ParScan` Stage Breakdown (`431` blocks, `211.0 MB`, Xeon `8481C`)

| Stage | `1T` CPU (`ms`) | Share (`%`) | Parallelizable? | Remaining Lever |
|---|---:|---:|---|---|
| `1. Huffman (read_item)` | `93.34` | `16.8%` | `100%` (worker) | Mechanisms `C38a` (`-28%` → `~67 ms`) |
| `2. runs::front (MTF)` | `49.30` | `8.9%` | `100%` (worker) | Memory-local (`syms` stays in core L2!) |
| **`3. OpenCells (invert_runs)`** | **`385.67`** | **`69.4%`** | `100%` (worker) | Round 4 `WormholeNeonCells` (`-5.2%` beyond `#719`!) |
| `4a. unrle_avx2` (2-buf) | `18.11` | `3.3%` | `100%` (worker) | Production `unrle_in_place_avx2` (`0 B ll`!) |
| `4b. crc32 (CLMUL)` | `8.98` | `1.6%` | `100%` (worker) | `64 KiB` L2-tiled streaming `clmul_update` |
| *4c. Scalar SWAR `InPlaceRLE+CLMUL`* | *`41.92`* | *(+`14.82 ms`)* | — | Exact measurement of scalar-SWAR vs AVX2 scan delta |
| **Total `1T` Worker Work** | **`555.41`** | **`100.0%`** | **`100%` parallel!** | `sys_ms` now only `11.5 ms` (`4W`) / `19.5 ms` (`10W`)! |

**Key proof:** After Mechanisms `C38b` + Wormhole `P2` + `C42`, **`95.1%` of total decode CPU sits cleanly in `OpenCells` (`69.4%`), `Huffman` (`16.8%`), and `runs::front` (`8.9%`)** — zero serial scan wall, zero `mmap`/`munmap` storm (`minflt = 3,961` at `4W`, `9,188` at `10W`), zero reader copy wall (`8.5 ms` on i9 = `21%` of one SMT sibling)!

---

## 2. Native `12`-Round Rotated Fresh-Process `4W` & `10W` A/B (`M1_PreSyms` & `M2_SpanLPT`)

| `W` | Arm | med (`ms`) | Ratio vs `BaseA` `[95% CI]` | `sys_ms` | `minflt` | `rss_MB` | Verdict |
|---:|---|---:|---|---:|---:|---:|---|
| `4` | `BaseA` / `BaseB` (`A/A`) | `341.1 / 337.4` | `1.0000` / `0.9912 [0.9844, 0.9982]` | `13.9 / 11.5` | `3,961 / 3,962` | `31.7 / 31.7` | Control |
| `4` | `M1_PreSyms` (`384 KiB` TLS) | `337.40` | `0.9905 [0.9789, 1.0022]` | `10.76` | `4,180` (`+219`) | `32.50` | **KILL** |
| `4` | `M2_SpanLPT` (16-blk window) | `340.16` | `0.9946 [0.9857, 1.0037]` | `12.89` | `4,397` (`+436`) | `33.29` | **KILL** |
| `10` | `BaseA` / `BaseB` (`A/A`) | `335.5 / 335.2` | `1.0000` / `0.9908 [0.9630, 1.0194]` | `23.5 / 19.5` | `9,188 / 9,383` | `52.1 / 52.9` | Control |
| `10` | `M1_PreSyms` | `337.16` | `1.0009 [0.9789, 1.0234]` | `22.16` | `9,413` | `53.03` | **KILL** |
| `10` | `M2_SpanLPT` | `337.37` | `0.9979 [0.9732, 1.0233]` | `21.44` | `9,191` | `52.10` | **KILL** |

---

## 3. Comet Lake `10C/20T` SMT Core-Budget Allocation

- **10 physical cores:** `10` workers (1 per physical core, `256 KiB` unshared L2 each, `22.5 MiB` total L3 demand → `f_miss ≈ 11%`).
- **SMT sibling overhead for Reader (`T_reader = 8.5 ms / 40 ms = 21%` duty cycle):** Even when sharing 1 core with worker 0, a `21%` sequential `AbHash`/`write_all` reader steals `<= 3%` of that single core (`<= 0.3%` total throughput!), far cheaper than dedicating a whole physical core (`-10%` worker throughput, i.e., `9W` vs `10W`)!
- **Adding SMT workers `11..20`:** Splits `256 KiB` L2 (`128 KiB`/worker) and pushes L3 demand `22.5 → 45 MiB` (`f_miss 11% → 55%`), always losing to `10` physical workers (`39–41 ms` vs `46–50 ms`).

---

## 4. Kill Decisions (One Line Each)

- **KILL `M1_PreSyms` (`reserve_exact(196_608)` TLS `syms`):** Neutral vs A/A (`1.0009×` at `10W`), adds `+219` `minflt` and `+0.8 MiB` RSS on small/tail workers; standard `Vec` growth on 1 thread-local buffer already amortizes across `43` blocks/worker.
- **KILL `M2_SpanLPT` (mid-stream 16-block windowed span-LPT):** Neutral vs A/A on uniform Wikidata (`0.9979×` at `10W`) while adding `+436` `minflt` at `4W` from out-of-order `ready` residency; reserve LPT strictly for heterogeneous / final-tail blocks (`last 2*W`).
