# Pre-port tests: Deterministic Network Calculus (Panco + Nancy) and uiCA vs llvm-mca

Thread 6 of [`../../plan.md`](../../plan.md), answering part 2 of [`../../rauzy-census/prompt.md`](../../rauzy-census/prompt.md) and the frozen pre-port gates in `research/neon-movement/SOFTWARE-SURVEY.md` §3 (`leads/research-software.md`). Offline characterisation only: zero changes to `src/` or production behaviour, zero Rust ported, zero AGPL (`uiCA`) code committed.

## Summary of verdicts (frozen gates)

| Deep Dive | Package(s) tested | Frozen gate (`SOFTWARE-SURVEY.md` §3) | Verdict | One-line reason |
|---|---|---|---|---|
| **#1: Network Calculus** | Stock **Panco** (BSD-3 + `lp_solve`) & stock **Unipi.Nancy 1.6.0** (MIT, `.NET 10.0`) on `character-stage1` at `k ∈ {1, 6, 8, 10}` | `U_NC / T_wall ≤ 1.5`, knee near `7–8`, and separates two policies (`W = 4k` vs `2k`, or `S+C` vs `C`-only at 8w) | **KILL** | Strict worst-case bounds fail at **3.21–3.47×** (`1.74–1.88×` even after stripping tiny blocks) and miss the `k = 8` crossover; adding Nancy's window closure to empirical burst curves explodes to **3.09–11.56×**; the only tight (`1.03–1.20×`) bounds are non-strict corpus-mean open-loop arithmetic where `SubAdditiveClosure` is a no-op and Panco PLP collapses identically to scalar SFA. |
| **#2: Microarchitectural cycle model** | Stock **`uiCA.py -arch CFL`** (external CLI, AGPL-3.0, zero code committed) vs **`llvm-mca -mcpu=skylake`** (LLVM 18.1.3) on 4 compiled hot loops (`target-cpu=skylake`) | `≤ 5%` on `src/huff.rs` Huffman loop, right sign and size on PR #870's 3-symbol regression, clearly better than `llvm-mca` | **KILL** | `uiCA` under-predicts `src/huff.rs` by **−28.8%** (`10.05` vs measured `14.12 cyc/lookup`; `llvm-mca` is `10.60`, `−24.9%`, closer than `uiCA`) and gets **PR #870's sign backwards** (**−8.0%/lookup**, **−19.0%/sym faster** vs measured **+0–2.4% slower** on wikidata and **+9% slower** on `records`), because an L1-hit straight-line simulator cannot see the 24→48 KiB L1d cache spill or tail-loop mispredicts. |

Combined with Part 1's **STOP** on Rauzy induction orders (`surface_dynamics`), **all three top-ranked porting candidates in `leads/research-software.md` are eliminated by their pre-port tests**. No Rust port should be started.

---

## Test 1 — Deterministic Network Calculus on `character-stage1` (Deep Dive #1)

**Data:** `docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/` (`1,724` blocks of `wikidata ×4`: `912` `open` + `20` `open_dense` full 900 KB blocks = `932` Open blocks, plus `600` `skipped` + `192` `column` tiny blocks = `792` tiny blocks at `~0.5–1 µs` each; 36–48 grid decodes per `k`).
**Scripts:** [`nc_panco.py`](nc_panco.py) (stock Panco `FifoLP` PLP / `SfaLP` / `TfaLP` + `lp_solve`, exporting [`nc_stage1_params.json`](nc_stage1_params.json)) and [`nc_nancy/Program.cs`](nc_nancy/Program.cs) (stock `Unipi.Nancy 1.6.0` exact rational UPP tandem convolution `β_fwd = β_S ⊗ β_{W,k} ⊗ β_D` and Zippo–Stea / Le Boudec–Thiran window-feedback closure `β_cl = β_fwd ⊗ SubAdditiveClosure(β_fwd + W)`, outputting [`nc_nancy_results.json`](nc_nancy_results.json)).

### 1.1 Why block heterogeneity breaks strict deterministic service curves

In `character-stage1/report.txt` (lines 360–364), per-block worker service times span **3,500×**:
- `column` (`192` blocks, median 8 rows) & `skipped` (`600` blocks, median 30 rows): `~0.7–1.0 µs/block`;
- `open` (`912` blocks, 900k rows): `~1.17 ms/block` at `k=1`, `~1.62 ms/block` at `k=10`;
- `open_dense` (`20` blocks, 900k rows): `2.43 ms/block` (`2.7 ns/row`) at `k=1`, `3.06 ms/block` (`3.4 ns/row`) at `k=10`.

We evaluated four mathematically distinct calibrations in stock **Nancy** (`k = 1..10`, showing gate counts `1, 6, 8, 10` plus `7, 9` around the knee):
- **`M1_strict_block_worst`** *(valid strict deterministic certificate in block units for arbitrary block orderings)*: `N = 1,724`, `R_S = 1/s_open_scan_max`, `R_W = k/s_open_dense_max`, `R_D = 1/s_open_read_max`, single-block latencies `T_stage = s_max`.
- **`M3a_open_strict_worst`** *(strict deterministic certificate after separating the 792 tiny blocks as an exact additive overhead `Δ_tiny`)*: `N = 932` full blocks, worst-case `open_dense` rate.
- **`M2_trace_envelope_block`** *(trace-calibrated block-unit envelope on `wikidata ×4`)*: worst-round corpus rates `R = N / max_round(busy)` from `observations.tsv` and exact empirical latencies `T = b_max(f=1.0) / R` from `analysis/burst_frontier.tsv` (`b_max(1.0) = 32.6..93.4` issued, `39.8..107.4` ready).
- **`M3b_open_mean_hol`** *(optimistic corpus-mean full-block model — NOT a worst-case certificate because 20 `open_dense` blocks could cluster)*: `N = 932`, worst-round corpus-mean rates `932 / max_round(busy)`, latency `T_W = s_open_dense_max`, `T_D = s_open_read_max + hol_wait_p99`.

### 1.2 Nancy results: `U_NC(k)` and `U_NC / T_wall` (exact UPP convolution + window closure)

| `k` | `T_wall` (med) | `M1` strict worst (`W=4k`, `2k`) | `M3a` Open strict (`W=4k`, `2k`) | `M2` open-loop `U_fwd` | `M2` closed **`W=4k`** | `M2` closed `W=2k` | `M3b` Open mean (`U_fwd == U_4k == U_2k`) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | `1.2839 s` | `4.3322 s` (**3.37×**) | `2.3444 s` (**1.83×**) | `1.3580 s` (1.06×) | `14.8385 s` (**11.56×**) | `29.6724 s` (23.11×) | `1.3275 s` (1.03×) |
| **6** | `0.2380 s` | `0.8254 s` (**3.47×**) | `0.4478 s` (**1.88×**) | `0.2668 s` (1.12×) | `0.7359 s` (**3.09×**) | `1.4671 s` (6.16×) | `0.2600 s` (1.09×) |
| 7 | `0.2079 s` | `0.8830 s` (4.25×) | `0.4793 s` (2.31×) | `0.2881 s` (1.39×) | `0.8201 s` (3.94×) | `1.6355 s` (7.87×) | `0.2791 s` (1.34×) |
| **8** | `0.1923 s` | `0.6408 s` (**3.33×**) | `0.3481 s` (**1.81×**) | `0.2316 s` (1.20×) | `1.0337 s` (**5.38×**) | `2.0619 s` (10.72×) | `0.2160 s` (1.12×) |
| 9 | `0.1969 s` | `0.6262 s` (3.18×) | `0.3403 s` (1.73×) | `0.2385 s` (1.21×) | `1.0577 s` (5.37×) | `2.1091 s` (10.71×) | `0.2204 s` (1.12×) |
| **10** | `0.1972 s` | `0.6328 s` (**3.21×**) | `0.3441 s` (**1.74×**) | `0.2488 s` (1.26×) | `1.1194 s` (**5.68×**) | `2.2128 s` (11.22×) | `0.2276 s` (1.15×) |

*(Note on `k = 7`: one outlier round in `observations.tsv` had `W_max = 0.2749 s`, `S_max = 0.2512 s` vs medians `0.2032 / 0.1836 s`, which shows up in any worst-round bound.)*

### 1.3 Panco results: FIFO PLP (`FifoLP`), SFA (`SfaLP`), and TFA (`TfaLP`)

Feeding the empirical 5-bucket concave arrival curve `α_S(t) = min_{f ∈ {1.0, 1.1, 1.25, 1.5, 2.0}} (b_max(f) + f·ρ·t)` from `burst_frontier.tsv` (`t_issued`) into the downstream tandem `W_k → D` (using corpus-mean service rates `R_W = N/W_med`, `R_D = N/D_med`, because strict worst-case `open_dense` service rate violates Panco's stability condition `ρ < R_W` and yields infinite delay!):

| `k` | `T_wall` | `d_PLP` (single-blk lat) | `d_SFA` | `d_TFA` | `U_PLP = max(S_max, W_max) + d_PLP` (`U/wall`) | `d_PLP` (burst lat) | `U_burst/wall` | 2-flow FIFO (`Open`/`Tiny`) `d_PLP` |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | `1.2839 s` | `30.40 ms` | `30.40 ms` | `31.77 ms` | `1.3539 s` (1.055×) | `58.41 ms` | 1.076× | `30.40 / 30.40 ms` |
| **6** | `0.2380 s` | `7.38 ms` | `7.38 ms` | `8.84 ms` | `0.2640 s` (1.109×) | `10.97 ms` | 1.124× | `7.38 / 7.38 ms` |
| **8** | `0.1923 s` | `10.75 ms` | `10.75 ms` | `14.25 ms` | `0.2233 s` (1.161×) | `17.82 ms` | 1.198× | `10.75 / 10.75 ms` |
| **10** | `0.1972 s` | `13.08 ms` | `13.08 ms` | `18.69 ms` | `0.2365 s` (1.199×) | `22.59 ms` | 1.247× | `13.08 / 13.08 ms` |

### 1.4 Policy separation at `k = 8` (Nancy & Panco)

| Policy at `k = 8` (`T_wall = 0.1923 s`) | Nancy `M1` strict | Nancy `M3a` Open strict | Nancy `M3b` Open mean | Panco empirical `U_PLP` |
|---|---:|---:|---:|---:|
| `baseline` (`W = 4k = 32`) | `0.6408 s` (3.33×) | `0.3481 s` (1.81×) | `0.2160 s` (1.12×) | `0.2233 s` (1.16×) |
| `window W = 2k = 16` | `0.6408 s` (**1.00×**) | `0.3481 s` (**1.00×**) | `0.2160 s` (**1.00×**) | *(no window in tandem LP)* |
| `C_minus_30pct` (−30% C only) | `0.5553 s` (2.89×) | `0.3017 s` (1.57×) | `0.2156 s` (1.12×) | `0.2218 s` (1.15×) |
| `SC_minus_30pct` (−30% S and −30% C) | `0.5552 s` (**0.02% sep**) | `0.3016 s` (**0.03% sep**) | `0.1768 s` (0.92×) | `0.1831 s` (0.95×) |
| `C_ideal` (C free) | `0.3950 s` (2.05×) | `0.2146 s` (1.12×) | `0.2147 s` (1.12×) | `0.2185 s` (1.14×) |
| `SC_ideal` (S+C free) | `0.3557 s` (1.85×) | `0.1933 s` (1.01×) | `0.1133 s` (0.59×) | `0.1173 s` (0.61×) |

### 1.5 Why Test 1 is **KILL**

1. **Every strict worst-case deterministic bound fails the frozen gate:** `M1` gives **3.21–3.47×** (`> 3×`, hitting the survey's explicit KILL condition), and even `M3a` (after factoring out all 792 tiny blocks) gives **1.74–1.88×** (`> 1.5×`). Both are **blind to the `k = 8` scanner crossover** (`M1`/`M3a` predict `k = 9` faster than `k = 8` and give `0.02–0.03%` separation between `C_minus_30pct` and `SC_minus_30pct`), because `open_dense` inflates worst-case worker demand `3.3×` while scanner demand inflates `1.85×`.
2. **Nancy's window-feedback closure (`SubAdditiveClosure`) either explodes or does nothing:** On trace-calibrated block curves (`M2`), `burst_max(1.0) = 33–94` blocks exceeds `W = 4k`, so sub-additive closure assumes the macro-scale heavy/light clustering repeats every `W` blocks, collapsing rate to `W / T_fwd` (**3.09–11.56×** `T_wall`, and flipping the knee to `k = 6`). On single-block-latency curves (`M1`, `M3a`, `M3b`), `W / T_fwd > R_min`, so `U_win == U_fwd` to 6 decimal places for both `W = 4k` and `W = 2k` (**1.00× separation**).
3. **Where bounds are tight (`1.03–1.20×`), neither Nancy nor Panco is doing any real work:** Those bounds rely on non-strict corpus-mean rates, Panco `FifoLP` collapses **identically** to closed-form scalar `SFA` (`d_PLP == d_SFA` to 6 decimal places; no cross-traffic multiplexing exists in `mibz2`), and **98.4%** of Nancy `M3b`'s `0.2160 s` at `k = 8` is simply `max(S_max, W_max) = 0.2125 s`. Adding three scalars takes 3 lines of Python, not a 3–5k LOC Rust port of Nancy/Panco.

---

## Test 2 — Stock `uiCA.py -arch CFL` vs `llvm-mca -mcpu=skylake` (Deep Dive #2)

**Build & extraction:** Compiled unchanged `origin/master` (`21971a4`) and authentic PR #870 (`refs/pull/870/head`, in `/tmp`) with `RUSTFLAGS="-C target-cpu=skylake" cargo rustc --lib --release -- --emit=asm,obj`. Extracted exact fast-path compiled instructions via `objdump -d -l`, reassembled with GNU `as` so both tools see the **exact same x86-64 machine-code bytes and macro-fusion pairs**. Script: [`uica_vs_mca.py`](uica_vs_mca.py); raw JSON: [`uica_vs_mca_results.json`](uica_vs_mca_results.json).
**Tooling note (not committed):** Stock `uiCA` built against Intel XED `HEAD` (2026) initially marked BMI1/BMI2 (`shlx`, `shrx`, `blsr`, `bzhi`) and `movzx r32, word ptr [mem]` as unsupported (`X`) because newer XED renamed iforms `GPR64q → VGPR64q` and `MOVZX_GPR32_MEMw → MOVZX_GPRv_MEMw` relative to `uops.info` XML. After adding the 3-line iform-name alias in the local `/tmp/uica-tool` install, **0 instructions** were unsupported across all 8 loops.

### 2.1 Predictions vs already-measured i9-10900K figures

Measured i9-10900K figures come strictly from `character-stage1` (`k = 1`, `5.27622 GHz` from `contention.tsv`; `k = 6..10`, `4.9052 GHz`) and journal §82 (`wikidata`: `24.98M` symbols, `13.49M` lookups → `wikidata ×4`: `99.92M` symbols, `53.96M` lookups; `10,675,880` A6b steps; `60.57M` Open-block runs). No workstation run was made; loops without a direct per-loop hardware counter are noted explicitly.

| Loop (exact compiled Skylake hot path) | Insns (B) | `uiCA -arch CFL` cyc/iter (`align` range) | `llvm-mca -mcpu=skylake` cyc/iter (`RThr`) | `uiCA` per unit | `llvm-mca` per unit | `uiCA` vs `mca` | Already-measured i9-10900K | `uiCA` error vs measured |
|---|---:|---:|---:|---:|---:|---:|---|---|
| **1a. `src/huff.rs` master** `decode_group_win` 5-lookup batch + refill | 108 (433) | `50.25` (`47.25..50.25`) | `53.01` (`18.2`) | **`10.05 cyc/lkp`** | **`10.60 cyc/lkp`** | `−5.2%` | **`14.12 cyc/lkp`** (`k=1` A3 `0.1445s`; `13.97–14.24` at `k=6..10`; §82 `13–15`) | **−28.8%** (**FAIL** `≤ 5%`) |
| 1b. `src/huff.rs` master single `step_win!()` | 19 (72) | `8.96` (`8.96..8.96`) | `10.01` (`3.2`) | `8.96 cyc/lkp` | `10.01 cyc/lkp` | `−10.5%` | *(sub-loop of 1a; no separate i9 counter)* | *(−36.5% vs 1a)* |
| **2a. PR #870 3-sym LUT** `decode_group_win` 5-lookup batch + refill | 100 (402) | `46.25` (`46.25..46.25`) | `44.02` (`16.8`) | **`9.25 cyc/lkp`** | **`8.80 cyc/lkp`** | `+5.1%` | **+0–2.4% slower/sym** on wikidata (**+14–16%/lkp** at −12% lkps); **+9% slower** on `records` (§82) | **WRONG SIGN**: predicts **−8.0%/lkp**, **−19.0%/sym** |
| 2b. PR #870 single `step_win!(false, acc)` | 16 (62) | `8.00` (`8.00..8.00`) | `8.02` (`2.7`) | `8.00 cyc/lkp` | `8.02 cyc/lkp` | `−0.2%` | *(sub-loop of 2a; no separate i9 counter)* | **WRONG SIGN** (−10.7%/lkp) |
| **3a. `src/open/endpoint.rs`** A6b `Narrow` rule-push step (~62% of steps) | 111 (466) | `35.40` (`35.40..35.40`) | `21.04` (`19.0`) | `35.40 cyc/step` | `21.04 cyc/step` | `+68.3%` | **No separate i9 step-loop counter.** Total A6b (`setup+steps+dict`) = `106.76 cyc/step` (`k=1` `0.2160s`). | *(Weighted L1 floor `30.7 cyc/step` vs ~55 cyc/step implied by Part 1's 51.1% share)* |
| 3b. `src/open/endpoint.rs` A6b `Narrow` zero-word step (~38% of steps) | 74 (292) | `23.11` (`22.89..23.11`) | `13.05` (`12.7`) | `23.11 cyc/step` | `13.05 cyc/step` | `+77.1%` | *(see 3a)* | — |
| **4a. `src/runs.rs`** `front_avx2` Pass 1 (32-sym AVX2 bitmap) | 34 (148) | `9.29` (`9.17..9.29`) | `7.28` (`6.0`) | `0.290 cyc/sym` | `0.228 cyc/sym` | `+27.6%` | **No per-pass i9 counter.** Total `A5` = `6.99 cyc/sym` (`~11.5 cyc/run`, `k=1` `0.1324s`). | *(Pass 1 + Pass 2 = `5.74 cyc/sym` vs `6.99` total A5; residual `1.25` for `s≥33`, `run>127`, epilogue)* |
| **4b. `src/runs.rs`** `front_avx2` Pass 2 (literal MTF/RLE2, `s<33`) | 25 (112) | `9.00` (`9.00..9.00`) | `8.70` (`5.2`) | `9.00 cyc/lit` | `8.70 cyc/lit` | `+3.4%` | *(see 4a)* | — |

### 2.2 Why Test 2 is **KILL**

Every single clause of the frozen gate fails:
1. **`≤ 5%` on the Huffman decode loop:** **FAIL** (`−28.8%` error: `10.05` predicted vs `14.12 cyc/lookup` measured on the i9-10900K). The critical L1 shift-load chain (`shr → load → and → shlx`) is 8–9 cycles plus ~1 cycle amortized refill = `10.05 cyc/lookup`; the remaining **+4.1 cyc/lookup (~29% of `A3`)** comes from per-50-symbol group switching, tail-loop branch mispredictions, 1% slow-path calls, and L1d pressure across the 6 × 4 KiB = 24 KiB LUTs — none of which an L1-hit straight-line simulator models.
2. **Right sign and magnitude on PR #870's 3-symbol regression:** **FAIL (WRONG SIGN)**. Compiler inspection of PR #870's actual Skylake binary reveals that PR #870 removed `and $0x3f` from the `shlx` chain (via batched `acc`), cutting the L1 critical chain from `9` to `8` cycles, while introducing two stack reloads (`mov 0x48(%rsp), %r14` and `mov 0x98(%rsp), %rsi`) on every lookup. Because those reloads sit off the critical `rbp` shift-load chain in an L1-hit out-of-order simulator, `uiCA` predicts PR #870 is **−8.0% faster per lookup** and **−19.0% faster per symbol** on wikidata (`9.25 × 0.88 / 10.05 = 0.810×`) — the exact opposite of the measured **+0–2.4% regression** on wikidata and **+9% regression** on `records`. The real regression came from growing the 6 root LUTs from **24 KiB → 48 KiB** (spilling out of Comet Lake's **32 KiB L1d cache**!) plus per-group clamping branch mispredicts and +30–50% table-build cost.
3. **Clearly better than `llvm-mca`:** **FAIL on the primary target.** On `src/huff.rs`, `llvm-mca` (`10.60 cyc/lookup`, `−24.9%`) is actually closer to measured `14.12` than `uiCA` (`10.05`, `−28.8%`), both get PR #870's sign backwards, and on `runs` Pass 2 they agree within `3.4%` (`9.00` vs `8.70`). `uiCA` is genuinely more faithful than `llvm-mca` on large front-end-bound loops (`a6b_narrow` +68–77%, `runs` Pass 1 +28%), which makes stock `uiCA` useful as a 1-second offline diagnostic — **not** as an automated kernel-search fitness oracle, and **not** worth a 3–4k LOC clean-room Rust port.
