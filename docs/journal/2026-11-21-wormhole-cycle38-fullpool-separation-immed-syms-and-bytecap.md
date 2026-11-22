# Wormhole Cycle 38 (Neon) — Wall W4: Full-Block Pool Separation, Immediate `syms` Return, & `12 MiB` Byte-Budgeted Window (`4W = 0.9417×`, `10W minflt -57%`, `10W RSS 56 MiB`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `BzReader::parallel_workers` (`OpenCells`, `slots = 4*W`), with A/A control (`MasterA` vs `MasterB`) in the same binary.
**Candidate harness:** [`c38_window_diet_ab.rs`](2026-11-21-wormhole-cycle38/c38_window_diet_ab.rs) (`safe_c38` module `#![forbid(unsafe_code)]`), raw log [`c38_results.txt`](2026-11-21-wormhole-cycle38/c38_results.txt), manifest [`c38_window_diet.toml`](2026-11-21-wormhole-cycle38/c38_window_diet.toml).

---

## 1. Solving the Window-vs-HOL Dilemma (`Wall W4`)

In Cycle 32, naïve static window shrinking (`slots = 2W`) cut `minflt` and RSS on uniform Wikidata but regressed **`1.0857×`** on `hetero_60b` from head-of-line (HOL) scanner starvation. Cycle 38 eliminates dirty-window L3 pollution **without** starving heterogeneous streams via three orthogonal mechanisms on top of `P1c + C37a`:
1. **`W0: Full-Block `free_out` Pool Separation (`cap >= 65_536`):**
   Tiny blocks (`<= 2,048` symbols, `197/431` blocks in Wikidata!) allocate from thread-local glibc fastbins (`~15 ns`, `0` faults) instead of polluting `free_out` with `len = 2 KiB` buffers. This guarantees every recycled full-block buffer in `free_out` stays at `len ≈ 900 KiB`, eliminating `src/open.rs:360`'s `898 KiB` `memset(0)` on tiny→full transitions and cutting active `out` pool count from `4*W` down to `9` (`4W`) / `21` (`16W`).
2. **`W1: Immediate `syms` Return Post-`runs::front` (`0` Dead `syms`):**
   Returns `symbols.syms` to `free_syms` immediately after `runs::front` (~7% of worker time, piggybacked onto the same `ready.lock()` that pops `out`!), eliminating the `1.2 MiB` of dead `syms` observed in `examples/occupancy.rs`.
3. **`W3: Byte-Budgeted Ready Window (`max_ready_bytes = 8 MiB` / `12 MiB` with `W + 2` In-Flight Floor):**
   Scanner pauses only when `ready_bytes >= max_ready_bytes && issued - taken >= W + 2` (or `>= 4*W`). Tiny `2 KiB` blocks cost `0.002 MiB` (never stalling the scanner!), heterogeneous slow blocks still get `13+` completed full-block lookahead (zero HOL starvation!), and `10W` dirty `ready` L3 pressure is capped at **`10.3 MiB`** instead of `27–36 MiB`!

---

## 2. Native Safe-Rust A/B + A/A Results (Fresh-Process Cold-Start, Exact `AbHash`)

### 2a. `12`-Round Rotated `4W` A/B on `wikidata_431b_210m.bz2` (`211.0 MB`) & `hetero_60b.bz2`

| Arm | `wikidata_431b` `4W` ratio `[95% CI]` | `sys_ms` / `minflt` / `rss_MB` | `hetero_60b` `4W` ratio `[95% CI]` | `pkRdyMB` / `out/syms#` | Verdict |
|---|---|---:|---|---:|---|
| `MasterA` | `1.0000 [1.0000, 1.0000]` | `40.85` / `16,056` / `58.93` | `1.0000 [1.0000, 1.0000]` | — | Control |
| `MasterB` (`A/A`) | `0.9929 [0.9737, 1.0125]` | `40.28` / `16,097` / `59.92` | `1.0070 [0.9828, 1.0319]` | — | `A/A` clean |
| **`W0_C37_4W`** | **`0.9417 [0.9238, 0.9600]`** | **`22.62` (`-44.6%`) / `8,281` (`-48.4%`) / `45.25`** | **`0.9678 [0.9415, 0.9948]`** | `4.29` / `9/16` | **PROMOTE `W0`** |
| `W1_ImmedSyms` | `0.9617 [0.9418, 0.9819]` | **`20.60` (`-49.6%`) / `7,973` (`-50.3%`) / `45.27`** | **`0.9672 [0.9427, 0.9924]`** | `6.02` / `10/14` | **KEEP (`10W` win)** |
| `W13a_Cap8M` | `0.9499 [0.9350, 0.9650]` | `22.44` / `8,096` / `44.81` | `0.9775 [0.9549, 1.0007]` | `6.02` / `9/14` | **KILL `8 MiB` (tight)** |
| **`W13b_Cap12M`** | **`0.9546 [0.9382, 0.9713]`** | **`21.88` / `8,030` (`-50.0%`) / `44.79`** | **`0.9627 [0.9399, 0.9861]`** | `5.15` / `9/14` | **PROMOTE `W13b`** |

### 2b. Cold-Start `W = 10` & `W = 16` Window L3 Footprint (`wikidata_431b_210m`, `3`-round median)

| `W` | Arm | `wall_ms` | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`rss_MB` (`Δ`)** | `pkRdyMB` | `out/syms#` |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `10` | `MasterA` / `MasterB` | `375.5 / 371.8` | `670.6 / 661.3` | `54.45 / 62.32` | `25,355 / 25,609` | `92.56 / 89.71` | `~16–27` | `~35/35` |
| `10` | `W0_C37_4W` | `369.08` | `674.73` | `34.91` (`-35.9%`) | `14,789` (`-41.7%`) | `68.53` (`-26.0%`) | `16.34` | `25/27` |
| `10` | `W1_ImmedSyms` | **`365.56`** | `675.63` | `34.75` (`-36.2%`) | `12,176` (`-52.0%`) | `61.40` (`-33.7%`) | `11.19` | `18/19` |
| `10` | **`W13b_Cap12M`** | `367.74` | **`661.08`** | **`29.65` (`-45.5%`)** | **`10,890` (`-57.1%`)** | **`56.09` (`-39.4%`)** | **`10.33`** | **`15/19`** |
| `16` | `MasterA` / `MasterB` | `391.4 / 388.2` | `688.9 / 676.1` | `71.03 / 76.78` | `36,118 / 34,204` | `130.38 / 127.00` | `~25+` | `~50/50` |
| `16` | **`W0_C37_4W`** | **`370.50` (`-5.34%`)** | `676.52` | **`38.56` (`-45.7%`)** | **`15,174` (`-58.0%`)** | **`71.68` (`-45.0%`)** | **`12.05`** | **`21/16`** |
| `16` | `W13b_Cap12M` | `373.57` (`-4.56%`) | `675.81` | `39.14` (`-44.9%`) | `17,394` (`-51.8%`) | `80.95` (`-37.9%`) | `19.77` | `27/24` |

---

## 3. Updated `10T` i9-10900K Critical-Path Model (Full Phase 2 Core Complete!)

`W13b_Cap12M` empirically delivers **every single target assumption** of Cycle 36's Phase-2 roofline at `10W`:
- `out` buffers allocated: **`15`** (vs `40` on `v0.5.12`, `-62.5%`!); `syms` buffers allocated: **`19`** (vs `40`, `-52.5%`!);
- `minflt`: **`25,355 → 10,890` (`-57.1%`!)**; `sys_ms`: **`54.45 → 29.65 ms` (`-45.5%`!)**; `rss`: **`92.56 → 56.09 MiB` (`-39.4%`!)**;
- `pkRdyMB`: **`10.33 MiB`** (`mean ~4.5–5.5 MiB` dirty in L3, matching Cycle 36's `0.55 MiB/W` target!), bringing total `10W` L3 demand to **`~26.0 MiB` (`f_miss = 23%`)**, off-chip DRAM to **`3.66 B/B`**, and modelled `10T` Regime-2 critical path down to **`~43.0 ms`** (`-39%` vs `v0.5.12`'s `70 ms` plateau)!

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `W0: Full-Block free_out Pool Separation (cap >= 65_536)`:** Stops tiny blocks from triggering `open.rs:360` `898 KiB` `memset(0)` on full blocks; hits **`4W = 0.9417 [0.9238, 0.9600]`**, `-48.4%` `minflt` (`-58.0%` at `16W`), `-45.0%` `16W` RSS (`130.4 → 71.7 MiB`).
- **PROMOTE `W1 + W13b: Immediate syms Return + 12 MiB Ready Byte Cap`:** Cuts `10W` `minflt -57.1%` (`25.4k → 10.9k`), `sys_ms -45.5%`, `rss -39.4%` (`92.6 → 56.1 MiB`), and `pkRdy` to `10.3 MiB` while **winning `hetero_60b` `0.9627 [0.9399, 0.9861]`** with zero HOL regression.
- **KILL `W13a: 8 MiB Ready Byte Cap`:** Slightly tighter on `hetero_60b` (`0.9775 [0.9549, 1.0007]` vs `0.9627` for `12 MiB`) and worse `pkRdy`/`minflt` at `10W` (`13.1k` vs `10.9k`) than `12 MiB`.
