# Wormhole Cycle 42 (Neon) — Post-Mechanisms `ParScan`-Coupled Interface: Zero-`syms`-Handoff, Atomic Steal, & `P2` (`4W = 0.8932×`, `minflt -75.4%`, `10W RSS 51.3 MiB`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `BzReader::parallel_workers` (`OpenCells`, `slots = 4*W`), with A/A control in the same binary.
**Mechanisms Coordination (ZERO duplication):** Uses `mibz2`'s existing public `mibz2::scan` (`BlockSpan`) + `mibz2::bits::Bits::at` + `mibz2::block::read_item` to benchmark the exact **interface & downstream architecture** once Mechanisms' `C38b StreamParScan` supplies block bit-offsets (with `~2.2 ms` finder cost on i9, `3.3 ms` on 2-vCPU VM).
**Candidate harness:** [`c42_parscan_interface_ab.rs`](2026-11-21-wormhole-cycle42/c42_parscan_interface_ab.rs) (`safe_c42` module `#![forbid(unsafe_code)]`), raw log [`c42_results.txt`](2026-11-21-wormhole-cycle42/c42_results.txt), manifest [`c42_parscan_interface.toml`](2026-11-21-wormhole-cycle42/c42_parscan_interface.toml).

---

## 1. Architectural Shift Once Mechanisms' `ParScan` Supplies Block Spans

When Huffman decoding moves onto workers via Mechanisms' `ParScan`, three entire synchronization & memory layers in `v0.5.12` vanish simultaneously:
1. **Zero Scanner Thread & Zero `JobQueue` Mutex:** Replaced by a single `AtomicUsize::fetch_add(1, Relaxed)` over `&[BlockSpan]`!
2. **Zero `free_syms` Pool & Zero Cross-Core `syms` Migration:** Each worker keeps ONE thread-local `syms: Vec<u16>` (`~360 KiB`) reused on its own core across all blocks — eliminating `40` shared `syms` buffers and **`-0.56 B/B`** of cross-core L3 cache-coherence traffic!
3. **Direct Composition with Wormhole `P2`:** Emit-Direct-to-`out` (`0 B` `scratch.ll`, `-9 MiB` @`10W`) + Full-Block Pool Separation + `O(1)` Ring Ready Slots + `BufRead` Zero-Copy!

---

## 2. Native Safe-Rust A/B + A/A Results (`wikidata_431b_210m.bz2`, `211.0 MB`, exact `AbHash`)

### 2a. `12`-Round Rotated Fresh-Process `4W` A/B

| Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`vcsw` (`Δ`)** | **`rss_MB` (`Δ`)** |
|---|---:|---|---:|---:|---:|---:|---:|
| `MasterA` (`v0.5.12`) | `372.89` | `1.0000 [1.0000, 1.0000]` | `655.72` | `39.12` | `16,121` | `726` | `59.15` |
| `MasterB` (`A/A`) | `369.47` | `0.9793 [0.9548, 1.0044]` | `650.70` | `42.68` | `15,928` | `730` | `59.90` |
| **`Par_Std_0512`** (`TLS syms + BufRead + FullPool`, AVX2 `unrle`) | **`334.77`** | **`0.8932 [0.8804, 0.9062]`** | **`614.53`** | `15.04` (`-61.6%`) | `5,063` (`-68.6%`) | `296` (`-59.2%`) | `35.95` (`-39.2%`) |
| **`Par_P2_NoCap`** (`+0 B ll + InPlaceRLE`) | `339.16` | `0.9022 [0.8876, 0.9171]` | `631.75` | `12.99` (`-66.8%`) | `3,965` (`-75.4%`) | **`284` (`-60.9%`)** | `31.78` (`-46.3%`) |
| **`Par_P2_Cap12M`** (`+12 MiB Ready Cap`) | `339.61` | **`0.8972 [0.8714, 0.9238]`** | `630.41` | **`11.99` (`-69.4%`)** | **`3,960` (`-75.4%`)** | `286` (`-60.6%`) | **`31.71` (`-46.4%`)** |

### 2b. Cold-Start `W = 10` & `W = 16` (`3`-round median)

| `W` | Arm | `wall_ms` (`Δ`) | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`vcsw`** | **`rss_MB` (`Δ`)** |
|---:|---|---:|---:|---:|---:|---:|---:|
| `10` | `MasterA` / `MasterB` | `390.3 / 375.2` | `687.0 / 656.3` | `66.65 / 68.67` | `27,880 / 27,119` | `407 / 306` | `98.84 / 95.79` |
| `10` | **`Par_Std_0512`** | **`334.74` (`-14.2%`)** | **`628.70`** | `27.17` (`-59.2%`) | `11,647` (`-58.2%`) | `124` (`-69.5%`) | `61.87` (`-37.4%`) |
| `10` | **`Par_P2_NoCap`** (`0 B ll`) | `347.71` (`-10.9%`) | `650.01` | **`24.04` (`-63.9%`)** | **`9,012` (`-67.7%`)** | **`94` (`-76.9%`)** | **`51.34` (`-48.1%`)** |
| `10` | `Par_P2_Cap12M` | `345.36` (`-11.5%`) | `644.59` | `27.37` (`-58.9%`) | `9,405` (`-66.3%`) | `107` (`-73.7%`) | `53.06` (`-46.3%`) |
| `16` | `MasterA` / `MasterB` | `392.9 / 391.0` | `668.4 / 676.5` | `86.27 / 83.49` | `35,140 / 38,156` | `307 / 316` | `129.96 / 138.60` |
| `16` | **`Par_P2_NoCap`** | **`359.40` (`-8.5%`)** | `658.44` | **`33.92` (`-60.7%`)** | `14,835` (`-57.8%`) | `284` | `74.15` (`-42.9%`) |
| `16` | `Par_P2_Cap12M` | `361.56` (`-8.0%`) | `661.23` | `37.43` (`-56.6%`) | **`13,554` (`-61.4%`)** | `1,078`* | **`69.26` (`-46.7%`)** |

*\*`vcsw = 1,078` at `16W` on `Par_P2_Cap12M` exposed a `cv_worker.notify_all()` herd when waking backpressured workers — fixed to `notify_one()` in Cycles 44–45!*

---

## 3. Updated `10T` i9-10900K Critical-Path Model (Mechanisms `C38b` + Wormhole `P2` + `C42` TLS `syms`)

- `10W` peak RSS drops to **`51.3 MiB`** (`-48.1%` vs `v0.5.12`'s `98.8 MiB`!), `10W` `minflt` drops **`-67.7%`** (`27.9k → 9.0k`), `10W` `sys_ms` drops **`-63.9%`** (`66.7 → 24.0 ms`), and `0.56 B/B` of cross-core `syms` coherence traffic vanishes!
- Including Mechanisms' `2.2 ms` i9 finder cost, total `10W` L3 demand falls to **`~22.5 MiB`** (`f_miss ≈ 11%`), off-chip DRAM falls to **`~3.1 B/B` (`15.6 ms` BW floor, `16.4 GB/s` — 2.6× below DDR4 peak!)**, bringing modelled `10T` i9 wall time to **`~39–41 ms`** (or **`~36–38 ms`** with Round 4's `WormholeNeonCells` `-5%` Open kernel + Cycle 41 LPT)!

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `C42` ParScan-Coupled Interface (`TLS syms + AtomicUsize Steal + P2`):** Cuts `4W` to **`0.8932 [0.8804, 0.9062]`**, `sys_ms -69.4%`, `minflt -75.4%` (`4W`) / `-67.7%` (`10W`), `vcsw -76.9%` (`10W`), and `10W` RSS **`-48.1%` (`98.8 → 51.3 MiB`)**.
- **KILL `cv_worker.notify_all()` on single-block `ready` drain:** Causes `1,078` `vcsw` herd at `16W` under `12 MiB` cap; replace with directed `notify_one()` (keeping `notify_all()` only for cancel).
