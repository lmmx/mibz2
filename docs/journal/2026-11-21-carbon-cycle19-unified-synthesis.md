# Carbon Cycle 19: Temporal Arena Multiplexing (`EdgesMux`), Width-Certified Leaf-Loop Ostrowski Contraction, and 5-Cycle Shared-L3 Synthesis (`AhtEdgesMux16` / `AhtEdgesMux14`)

- **Branch:** `research/carbon-word-transport`
- **Baseline:** `origin/master` `mibz2 0.5.8` (`mibz2::bwt::OpenCells`, commit `bb98c99`; `src/` identical in `v0.5.9` `10207ff`)
- **Prior Internal Baselines:** Cycle 17 `SentOutQ16` (`e5dc8632`, `Gate 0.15`, `228/431`) and Cycle 18 `AhtSentOutQ16` (`929ee1c3`, `StructGate`, `383/431`)
- **Safety & MSRV:** `#![forbid(unsafe_code)]`, Rust `1.80` (`edition = "2021"`), single-file drop-in `bench/moonshot` candidate (`candidate_open_c19.rs`)
- **Official `moonshot check` Verdict:** **`PASS` (`failures: 0` across `300+` crafted cases, `Cells` witnesses `0..3`, `wikidata.bz2` `209,700,196 B`, and `600` mutated inputs vs `0.5.6`)**
- **Strict Gate G5 Verdict:** **`PASS (4/4)` (`G5.1=PASS, G5.2=PASS, G5.3=PASS, G5.4=PASS`)**
- **Artifacts:**
  - Drop-in `moonshot` candidate: `docs/journal/2026-11-21-carbon-cycle19/candidate_open_c19.rs` (`pub type Candidate = AhtEdgesMuxOpenCells;` + `AhtEdgesMux14OpenCells` + `SameGateEdgesMuxOpenCells`)
  - Acquisition manifest: `docs/journal/2026-11-21-carbon-cycle19/unified_edges_mux16.toml` (`docs/team/acquisition/check_manifest.py`: `PASS`)
  - Exhaustive mathematical verifier: `docs/journal/2026-11-21-carbon-cycle19/verify_cycle19.py` (`1,000` clobber-checked `EdgesMux` runs + `ra>rb`/`ra<rb` spirals + sticky `m=0xFFFFFFFF` abort: `PASS`)
  - Unified Gate G5 confrontation suite: `docs/journal/2026-11-21-carbon-cycle19/g5_unified_c19.rs` (`WIP` pre-registration commit `82f58ee4`)

---

## 1. Explore & Triage: Disjoint-Lifetime Producer Multiplexing & Zero-Spill `R4`

Cycle 18 (`AhtSentOutQ16`) admitted all `383/383` `takes_runs` blocks of `wikidata.bz2` (`100%` of full `900 KB` blocks), eliminating `Segmented`'s `+5.6 MiB`/worker fallback (`9,872 -> 1,788 KiB` 10-worker max heap), yet missed its `cg_s9` cycle sub-bar (`0.947× / 0.867×` vs `0.925× / 0.860×`) due to `+2.98M` extra instructions, and still carried separate per-worker allocations for `tops`, `bots`, and `u32_scratch`.

Auditing the live-range interval graph of every buffer in `Open` / `R4` across `try_run`'s five phases (`P1` Pass-1 `OutQ16`, `P2` thick-strip radix + linked-list build, `P3a` `R4` elimination + `xb` scan, `P3b` `make_code` band extraction, `P4` `wire`, `P5` `emit_sentinel`) revealed **producer-side temporal disjointness**:
- `u32_scratch` (`2 * ns` `u32`s) is live strictly inside `P2`;
- `bots` (`2 * slots` `u32`s, `slots = 3 * ns + 5`) is live strictly inside `P2 .. P3a`;
- `tops` (`4 * slots` `u32`s) is live strictly inside `P2 .. P3b`;
- `self.edges` (`m` `u32`s) is **100% dead throughout `P1 .. P3b`** and written for the first time in `P4` (`wire`)!

### Triage Ablations (`c19_explore.rs` + `chaingrind`)
1. **Manual 4-Wide `u64` Queue Unpacking in `wire` (`WIRE4`, `chunks_exact(8)`):** Killed (`75.09M` vs `74.72M` instructions, `4.615` vs `4.602 cyc/B` at `2,560 KiB` LLC — LLVM already auto-vectorizes `chunks_exact(2)` cleanly without scalar tail overhead).
2. **Inline `&mut` / 6-Tuple `ValRet` `#[cold]` Calls Inside `R4`'s Inner Loop:** Killed (`74.78M–75.42M` vs `73.35M–74.26M` instructions). Even when `#[cold] #[inline(never)]`, any `call` or 10-scalar live-out edge inside `R4`'s 13-variable inner loop clobbers the 9 caller-saved x86-64 SysV GPRs (`rax..r11`), spilling loop state around the inner loop. Solved cleanly by **Theorem 19.2a** (pre-loop strip-width certificate).
3. **Hot-Loop Test Step Counter (`r4_steps += 1` inside non-AHT `R4`):** Initial run `task-2102` showed that incrementing a test-only `r4_steps` counter inside the non-AHT leaf loop added `+0.25M` instructions and nudged `s9` `W=8` `cyc/B` to `0.9257×` (just `+0.0007×` over the frozen `0.9250×` bar, while `10W` `Master-0.5.8` scheduling variance gave `0.224×` vs `0.220×` sum-heap ratio). Removing those two debug-counter lines from `candidate_open_c19.rs` (with **zero** changes to `g5_unified_c19.rs`'s frozen bars) dropped `s9` instructions to `74.26M` (`0.915×`) and `s9` `cyc/B` to `0.9241× / 0.8438×`, passing **`4/4` strict Gate G5 sub-gates** (`task-2129`).

---

## 2. Mathematical & Algorithmic Constructions (`Theorems 19.1–19.2`)

### Theorem 19.1 (`EdgesMux` Disjoint-Lifetime Temporal Arena Multiplexing — `proved` + `verified`)
Let `slots = 3 * ns + 5`. Partitioning `self.edges[0 .. max(m, 6 * slots + 2 * ns + 8)]` (`max(m, 20 * ns + 38)` `u32`s) via safe Rust `split_at_mut` into `(t_buf[0 .. 4 * slots], b_buf[0 .. 2 * slots], scr[0 .. 2 * ns])` during Phases `P2 .. P3b` and overwriting `self.edges[0 .. m]` in Phase `P4`:
1. **Zero Aliasing in `#![forbid(unsafe_code)]` Rust 1.80:** `scr` dies at `P2_end`, `b_buf` dies at `P3a_end`, `t_buf` dies at `P3b_end` (after `make_code` extracts `(width, dst, code)` into `self.triples`), and `self.edges[0 .. m]` is first written in `P4`. Verified across `1,000` clobber-checked runs in `verify_cycle19.py` (poisoning `scr` after `P2`, `b_buf` after `P3a`, and all of `edges` right before `P4`).
2. **Heap & Shared-L3 Cache-Line Collapse:** Saves $\min(m,\; 20\,\text{ns} + 38) \times 4\text{ B}$ of per-worker heap. Across all `383` Wikidata run blocks, $\max(20\,\text{ns} + 38) = 121{,}098 < \max m = 183{,}067$, so `tops`, `bots`, and `u32_scratch` require **`0 B` of net heap** (`-432.1 KiB`/worker, cutting 228-block heap from `1,722.5 KiB` to **`1,290.4 KiB = 0.288×` `0.5.8`**) AND pre-warm the exact L2/L3 cache lines overwritten by `wire` — cutting `W = 10` (`2,048 KiB` LLC) DRAM traffic by a **further `-32.7%` (`0.361 -> 0.243 B/outB = 0.070×` `0.5.8`)** on `sample9` and **`-31.1%` (`2.539 -> 1.750 B/outB = 0.060×` `0.5.8`)** on the mixed high-density window! $\square$

### Theorem 19.2 (Width-Monotone Strip Certificate & Sticky `m = u32::MAX` Zero-Hot-Cost Abort — `proved` + `verified`)
1. **(a) Strip-Width Monotonicity Certificate:** Every Rauzy–Veech splitting step replaces $\max(r_a, r_b)$ with $|r_a - r_b| < \max(r_a, r_b)$ without increasing any segment's width. Hence at every step of `R4`, $\max(r_a, r_b) \le W_{\max} := \max_{s \in \text{Tops}} \text{width}(s)$, and every Ostrowski quotient satisfies $q < W_{\max}$ (`0` violations across `1,000` trials in `verify_cycle19.py`). Pre-dispatching `records.len() * 64 < n || max_w > 32_768` to `#[cold] run_r4_aht_mux` contracts all sparse blocks and all large spirals (`300,000 -> 2` steps) in $O(\log q)$, while letting all normal dense blocks run a **100% call-free, zero-AHT-branch `R4` leaf loop** (`74.26M` instructions = **`0.915×` `0.5.8`**).
2. **(b) Sticky `m = u32::MAX` Early-Abort Invariant:** Setting `self.pm.m = u32::MAX` inside `#[cold] push_slow` whenever `w > m_limit || m > m_limit - w` forces `m_lo = m & 0xFFFF == 0xFFFF > M_LO_MAX` (`0xFFEF` at `L=16`, `0xFFF3` at `L=14`) on every subsequent `push_thin` call while keeping `self.pm.max_out_chunks` loop-invariant in a GPR across Pass 1 (`0` hot-path instructions, `11.1 KiB <= 32 KiB` early-abort heap; lazy `Option<Box<PassMeta>>` gives `1.4 KiB` = `0 B` added overhead on pure column-route workers). $\square$

---

## 3. Confront: Strict Gate G5 Results (`g5_unified_c19.rs` + Official `moonshot`)

### 3.1 Per-Worker Heap Footprint (`G5.2`: `PASS`)

| Kernel Arm | `228`-Block Subset Peak Heap (`vs 0.5.8`) | `431`-Block `10`-Worker Max Per-Worker Peak (`vs 0.5.8`) | `431`-Block `10`-Worker Sum Peak (`vs 0.5.8`) |
|---|---:|---:|---:|
| `0: Master-0.5.8` (`OpenCells`, `228/431`) | `4,479.5 KiB` (`1.000×`) | `9,979.4 KiB` (`1.000×`) | `64.30 MiB` (`1.000×`) |
| `1: C19-SameGate-Mux16` (`Gate 0.15`, `228/431`) | **`1,290.4 KiB` (`0.288×`, `-71.2%`)** | `10,621.0 KiB` (`1.064×`) | `31.91 MiB` (`0.496×`) |
| **`2: C19-AhtEdgesMux16` (`Candidate`, `383/431`)** | **`1,290.4 KiB` (`0.288×`, `-71.2%`)** | **`1,346.0 KiB` (`0.135×`, `-86.5%`)** | **`12.17 MiB` (`0.189×`, `-52.13 MiB`)** |
| **`3: C19-AhtEdgesMux14` (`StructGate L=14`, `383/431`)** | **`1,329.5 KiB` (`0.297×`, `-70.3%`)** | **`1,313.6 KiB` (`0.132×`, `-86.8%`)** | **`11.89 MiB` (`0.185×`, `-52.41 MiB`)** |

### 3.2 Dual-Suite `chaingrind` Contention Proxy (`W = 8` `2,560 KiB` & `W = 10` `2,048 KiB`, `G5.3`: `PASS`)

| Suite & Per-Worker `LLC` | Kernel Arm | Instructions (`vs 0.5.8`) | `dep_past_l2` | `DRAM B/outB` (`vs 0.5.8`) | `cycles/B` (`vs 0.5.8`) |
|---|---|---:|---:|---:|---:|
| **`s9` (`sample9`), `2,560 KiB` (`W=8`)** | `Master-0.5.8` | `81.17M` (`1.000×`) | `331.5k` | `1.185` (`1.000×`) | `4.964` (`1.000×`) |
| `s9`, `2,560 KiB` (`W=8`) | `C19-SameGate-Mux16` | **`73.61M` (`0.907×`)** | `324.8k` | **`0.218` (`0.184×`, `-81.6%`)** | **`4.564` (`0.919×`)** |
| **`s9`, `2,560 KiB` (`W=8`)** | **`C19-AhtEdgesMux16`** | **`74.26M` (`0.915×`)** | `324.8k` | **`0.218` (`0.184×`, `-81.6%`)** | **`4.587` (`0.924×`)** |
| **`s9`, `2,560 KiB` (`W=8`)** | **`C19-AhtEdgesMux14`** | **`74.92M` (`0.923×`)** | **`297.8k`** | **`0.217` (`0.183×`, `-81.7%`)** | **`4.470` (`0.901×`)** |
| **`s9` (`sample9`), `2,048 KiB` (`W=10`)** | `Master-0.5.8` | `81.17M` (`1.000×`) | `331.3k` | `3.452` (`1.000×`) | `5.442` (`1.000×`) |
| `s9`, `2,048 KiB` (`W=10`) | `C19-SameGate-Mux16` | **`73.61M` (`0.907×`)** | `324.8k` | **`0.243` (`0.070×`, `-93.0%`)** | **`4.569` (`0.840×`)** |
| **`s9`, `2,048 KiB` (`W=10`)** | **`C19-AhtEdgesMux16`** | **`74.26M` (`0.915×`)** | `324.7k` | **`0.243` (`0.070×`, `-93.0%`)** | **`4.592` (`0.844×`)** |
| **`s9`, `2,048 KiB` (`W=10`)** | **`C19-AhtEdgesMux14`** | **`74.92M` (`0.923×`)** | **`297.8k`** | **`0.224` (`0.065×`, `-93.5%`)** | **`4.472` (`0.822×`)** |
| **`dense` (`blk 40..278`), `2,560 KiB` (`W=8`)** | `Master-0.5.8` | `112.61M` (`1.000×`) | `1,970.1k` | `23.272` (`1.000×`) | `21.971` (`1.000×`) |
| **`dense`, `2,560 KiB` (`W=8`)** | **`C19-AhtEdgesMux16`** | `120.83M` (`1.073×`) | **`781.6k`** | **`0.386` (`0.017×`, `-98.3%`)** | **`10.056` (`0.458×`)** |
| **`dense` (`blk 40..278`), `2,048 KiB` (`W=10`)** | `Master-0.5.8` | `112.61M` (`1.000×`) | `1,975.3k` | `29.174` (`1.000×`) | `24.590` (`1.000×`) |
| **`dense`, `2,048 KiB` (`W=10`)** | **`C19-AhtEdgesMux16`** | `120.83M` (`1.073×`) | **`781.3k`** | **`1.750` (`0.060×`, `-94.0%`)** | **`10.295` (`0.419×`)** |

### 3.3 Native `lto=fat` Multi-Worker Timing & Fresh-Process `moonshot` `VmHWM` (`G5.4`: `PASS`)

| Benchmark & Worker Count `W` | `Master-0.5.8` | `C19-SameGate-Mux16` | **`C19-AhtEdgesMux16` (`Candidate`)** | **`C19-AhtEdgesMux14` (`L=14`)** |
|---|---:|---:|---:|---:|
| `228`-blk Kernel (`W=1`, 15 rds) | `326.9 ms` (`1.000×`) | `299.0 ms` (`0.915×`) | **`299.2 ms` (`0.915×`)** | `301.4 ms` (`0.922×`) |
| `228`-blk Kernel (`W=2`, 15 rds) | `158.5 ms` (`1.000×`) | `144.5 ms` (`0.912×`) | **`144.2 ms` (`0.910×`)** | `145.8 ms` (`0.919×`) |
| **`228`-blk Kernel (`W=8`, 15 rds)** | `174.9 ms` (`1.000×`) | **`156.8 ms` (`0.897×`)** | **`157.3 ms` (`0.900×`)** | **`157.3 ms` (`0.899×`)** |
| **`228`-blk Kernel (`W=10`, 15 rds)** | `177.0 ms` (`1.000×`) | **`158.7 ms` (`0.896×`)** | **`158.5 ms` (`0.895×`)** | **`158.5 ms` (`0.895×`)** |
| `431`-blk Kernel (`W=1`, 15 rds) | `350.6 ms` (`1.000×`) | `316.7 ms` (`0.904×`) | **`315.2 ms` (`0.899×`)** | `318.3 ms` (`0.908×`) |
| `431`-blk Kernel (`W=2`, 15 rds) | `171.3 ms` (`1.000×`) | `156.0 ms` (`0.911×`) | **`152.7 ms` (`0.892×`)** | `154.9 ms` (`0.904×`) |
| **`431`-blk Kernel (`W=8`, 15 rds)** | `191.2 ms` (`1.000×`) | `168.8 ms` (`0.883×`) | **`166.2 ms` (`0.869×`)** | `167.7 ms` (`0.877×`) |
| **`431`-blk Kernel (`W=10`, 15 rds)** | `194.3 ms` (`1.000×`) | `173.3 ms` (`0.892×`) | **`169.4 ms` (`0.872×`)** | **`168.4 ms` (`0.867×`)** |
| **`BzReader` End-to-End (`W=8`, 11 rds)** | `315.6 ms` (`1.000×`) | `286.1 ms` (`0.907×`) | **`281.2 ms` (`0.891×`)** | `284.6 ms` (`0.902×`) |
| **`BzReader` End-to-End (`W=10`, 11 rds)** | `309.4 ms` (`1.000×`) | `277.1 ms` (`0.895×`) | **`280.8 ms` (`0.908×`)** | **`277.1 ms` (`0.896×`)** |
| **Official `moonshot run` (`W=8`, 5 fresh procs, 2× `wikidata.bz2`)** | `722.4 ms` / `1.39 s` CPU / `134,876 kB` RSS | — | **`606.2 ms` (`0.839×` wall) / `1.13 s` (`0.813×` CPU) / `123,348 kB` (`-11.3 MiB` RSS)** | — |
| **Official `moonshot run` (`W=10`, 5 fresh procs, 2× `wikidata.bz2`)** | `723.2 ms` / `1.38 s` CPU / `149,764 kB` RSS | — | **`603.6 ms` (`0.835×` wall) / `1.15 s` (`0.833×` CPU) / `135,664 kB` (`-13.8 MiB` RSS)** | — |

---

## 4. Five-Cycle Synthesis (`Cycles 15–19` vs `Master-0.5.8`, `#675`, and `#679`)

| Arm / Cycle | Safety | `228`-Blk Worker Heap | `431`-Blk `10W` Max Heap | `s9` `DRAM B/B` (`W=8 / W=10`) | `dense` `DRAM B/B` (`W=8 / W=10`) | `s9` `cyc/B` (`W=8 / W=10`) | `431`-Blk Kernel Wall (`W=8 / W=10`) | Gate G5 |
|---|---|---:|---:|---:|---:|---:|---:|:---:|
| **`Master-0.5.8` (`OpenCells`)** | safe | `4,479.5 KiB` (`1.000×`) | `9,979.4 KiB` (`1.000×`) | `1.185 / 3.452` (`1.00× / 1.00×`) | `23.27 / 29.17` (`1.00× / 1.00×`) | `4.964 / 5.442` (`1.000× / 1.000×`) | `1.000× / 1.000×` | baseline |
| `#675` (`8L BP4K + R4, L=24`) | `unsafe` | `4,933.2 KiB` (`1.101×`) | `> 9.9 MiB` | `2.374 / 4.669` (`2.05× / 1.36×`) | — | (`1.049× / 1.012×` i9 wall!) | lost at `8–10T` | — |
| `#679` (`1L R4, L=24`) | safe | `5,157.8 KiB` (`1.151×`) | `> 9.9 MiB` | `0.858 / 3.844` (`0.74× / 1.12×`) | — | `1.000× / 0.996×` | `~1.05×` (`228` blk) | — |
| **Cycle 15 (`ContigEpochQ16`)** | `#![forbid]` | `2,426.8 KiB` (`0.542×`) | `~7.9 MiB` | `0.332 / 1.823` (`0.29× / 0.53×`) | — | `0.927× / 0.894×` | (`0.896×` `228` blk) | **`PASS`** |
| **Cycle 16 (`SwarOutQ16`)** | `#![forbid]` | `1,722.5 KiB` (`0.385×`) | `~7.3 MiB` | `0.233 / 0.355` (`0.20× / 0.10×`) | — | `0.941× / 0.852×` (`L=16`) | (`0.915×` `228` blk) | `FAIL` (dyn `L`) |
| **Cycle 17 (`SentOutQ16`)** | `#![forbid]` | `1,722.5 KiB` (`0.385×`) | `7,341.0 KiB` (`0.744×`) | `0.233 / 0.359` (`0.20× / 0.10×`) | `22.42 / 28.31` (`0.96× / 0.97×`) | `0.924× / 0.847×` | `0.912× / 0.908×` (`228`) | **`PASS`** |
| **Cycle 18 (`AhtSentOutQ16`)** | `#![forbid]` | `1,722.5 KiB` (`0.385×`) | `1,787.5 KiB` (`0.181×`) | `0.233 / 0.361` (`0.20× / 0.11×`) | `0.660 / 2.539` (`0.03× / 0.09×`) | `0.947× / 0.867×` | `0.868× / 0.876×` | `3/4` (`s9 cyc`) |
| **Cycle 19 (`SameGate-Mux16`)** | `#![forbid]` | **`1,290.4 KiB` (`0.288×`)** | `10,621 KiB` (`228/431`) | **`0.218 / 0.243` (`0.18× / 0.070×`)** | `22.27 / 28.17` | **`0.919× / 0.840×`** | `0.883× / 0.892×` | **`PASS` (`228`)** |
| **Cycle 19 (`AhtEdgesMux16`)** | **`#![forbid]`** | **`1,290.4 KiB` (`0.288×`)** | **`1,346.0 KiB` (`0.135×`)** | **`0.218 / 0.243` (`0.18× / 0.070×`)** | **`0.386 / 1.750` (`0.017× / 0.060×`)** | **`0.924× / 0.844×`** | **`0.869× / 0.872×`** | **`PASS (4/4)`** |
| **Cycle 19 (`AhtEdgesMux14`)** | **`#![forbid]`** | **`1,329.5 KiB` (`0.297×`)** | **`1,313.6 KiB` (`0.132×`)** | **`0.217 / 0.224` (`0.18× / 0.065×`)** | **`0.426 / 1.759` (`0.018× / 0.060×`)** | **`0.901× / 0.822×`** | **`0.877× / 0.867×`** | **`PASS (4/4)`** |

### Coordinator Handoff Recommendation
`docs/journal/2026-11-21-carbon-cycle19/candidate_open_c19.rs` is ready for immediate native i9-10900K `just moonshot-score` audit:
1. **Default export (`pub type Candidate = AhtEdgesMuxOpenCells`):** Drop-in whole-corpus structural-gate replacement (`LIMIT = 16`, `#![forbid(unsafe_code)]`, Rust `1.80`).
2. **One-line A/B switches inside the same file:**
   - `SameGateEdgesMuxOpenCells` (`USE_AHT = false`): isolates `OutQ16 + EdgesMux + SentEmit` on the exact `228`-block `0.5.8` `Gate 0.15` subset (`-71.2%` heap, `-93.0%` `W=10` DRAM, `0.895–0.897×` speed).
   - `AhtEdgesMux14OpenCells` (`LIMIT = 14`): cuts `dep_past_l2_loads` by `-33.7k` (`0.901× / 0.822×` `s9` `cyc/B`, `-93.5%` `W=10` DRAM, `1,313.6 KiB = 0.132×` 10-worker max heap) if the physical i9-10900K L3 rewards the smaller `edges` array even further at `8–10` threads.
