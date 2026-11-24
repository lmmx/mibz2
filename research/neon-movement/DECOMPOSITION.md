# Decoder Character Across 1–20 Workers: Decomposition, Minimal Design, Analytical Bounds, and Formalism

*Response to `briefs/decomposition.md` (`9174b294d`), journal §§44 & 47, and `channel/coordinator-to-neon.md` #34.*

**Claim tags used throughout:**
- **`[PROVED]`** — follows mathematically, directly from `mibz2` source code (`master` at `9621b7a`), or directly from measured workstation data (`four-arm/run-20261124T204154Z`, `bench/wikidata.json`, `2026-11-24-wikidata-block-census.md`, `2026-11-20-open-stage-attribution.md`).
- **`[ARGUED]`** — inferred from i9-10900K (Comet Lake-S) microarchitecture, queueing / control theory, or cross-machine ratios.
- **`[CONJECTURED]`** — falsifiable empirical hypothesis to be settled by the experiment below.

---

## 1. Audit of the Decomposition (`S0–S7`, `R1–R3`)

### Prerequisite: separate the homogeneous worker pool from `StreamDecoder::new`'s policy switches
- **`[PROVED]`** Production `StreamDecoder::new(k)` (`src/parallel.rs` L468–503, L2827–2919, L3350–3431) does **not** run a single architecture across `k = 1..20` on the i9-10900K: `k=1` in `ab` runs serial `BzReader::new` (`Cells`); `k ∈ {2,3,5,7}` run eager `Cells`; unpinned `k ∈ {4,6,8,9}` run `OpenCells`; **all unpinned `k ≥ 10` trim to `TRIMMED_WORKERS = 7` lazy workers (`28` slots)**; and `Plan::Teams` / `SERVICE_PLAN` (`2×3` teams + `2` service workers) introduce a **second** serial-vs-parallel split (team leader doing MTF/RLE1/CRC vs helpers sharing build/walk).
- **`[PROVED]`** By contrast, `bench-four-arm` and `bench-scan-profile` (`#838`, `examples/scan_profile.rs`) run the homogeneous worker pool `BzReader::parallel_workers(k, 4*k, kernels, lazy=true)` (`OpenCells` + `LazyCells` fallback, `READY_BYTES = 12 MiB`, `Cap::floor = k + 2`).
- **Rule:** Establish the full 1–20 character on the **homogeneous `parallel_workers` (`OpenCells`) topology** first. Evaluate `StreamDecoder::new`'s kernel/trim/team switches separately as model predictions.

### Wrongly merged stages and resources

| Brief element | Code/hardware reality | Required split | Status |
|---|---|---|---|
| **`S5` (construction)** | `src/open.rs` L350–596 & `src/open/outq.rs` L250–323 combine three physically distinct kernels: (a) pass-1 run scan + 256-bucket `outq` writes into `out` + `out.resize(n+16, 0)` (zero-fills cold/shorter `out`, zero-cost on recycled full blocks since `free_out` is not cleared at `src/parallel.rs` L916–933); (b) `endpoint` interval-exchange induction on `ns = 568–5,264` thick strips (median `3,439`, working set `~100–350 KiB`, L2-resident, compute/branch-bound); (c) `wire_xor` (`edges[u] ^= pack \| v; edges[v] ^= u`, run on 232/233 distinct Open blocks where `m ≥ 65,536`), whose `edges[v]` target is a **random read-modify-write** over `4m = 274–926 KiB` (> 256 KiB L2), generating **dirty cache-line writebacks** in the same random-access class as `S6`. | **`S5a`** (pass-1 + `outq` + zero-fill), **`S5b`** (endpoint induction + dictionary), **`S5c`** (`wire_xor` scatter-RMW). Tag every block by kernel route (`Open-bidir`, `Open-fwd`, `LazyCells`, `Column`). | `[PROVED]` (`src/open.rs`, `outq.rs`) / `[ARGUED]` (L2/L3 class) |
| **`S3` (hand-off & queue)** | `src/parallel.rs` L188–249, L280–409, L927–967, L1071–1199 combine: (a) lock/condvar serialization (`Mutex<State>` acquired **5 times per block** across scanner, worker, and reader + `Jobs` mutex); (b) **two distinct backpressure gates** in `Shared::full` (L402–408): slot window `ahead ≥ 4k` AND byte cap `ready.bytes ≥ 12 MiB && ahead ≥ k + 2`; (c) cross-core L3 cache-line migration of `syms` (scanner → worker) and `out` (worker → reader). | **`S3a`** (mutex/condvar wait + lock time), **`S3b`** (backpressure stall, split into `slot_full` vs `ready_bytes_cap`), **`S3c`** (cross-core buffer transfer, captured via L3/cache-to-cache counters). | `[PROVED]` (`src/parallel.rs`) |
| **`S7` (RLE1, CRC, output)** | `rle::unrle_in_place` + 64 KiB L2-tiled CRC (`src/lib.rs` L563–583, `src/rle.rs` L85, L103–120) runs **parallel in workers**. In-order block retirement, `Ready` removal, `free_out` recycling, and any output copy/write (`StreamDecoder::advance`/`read_block`/`read`, `src/parallel.rs` L840–967) run **serially on the caller/reader thread**. | **`S7a`** (worker-parallel `unrle_in_place` + tiled CRC) and **missing serial stage `S8`** (reader reordering, drain, buffer recycling, caller sink). | `[PROVED]` (`src/lib.rs`, `src/parallel.rs`) |
| **`R1` (bandwidth & LLC)** | Conflates a **capacity threshold**, a **rate ceiling**, and an **address-translation limit**. On Comet Lake-S: 20 MiB shared L3 (`R1a`) determines miss rate; dual-channel DDR4 (`~35–42 GB/s` practical, `R1b`) caps bytes/s once traffic misses L3; 64-entry L1 dTLB / 1,536-entry L2 STLB (`6 MiB` 4 KiB-page reach, halved to `3 MiB`/thread under SMT, `R1c`) penalizes scattered `edges` + `out` + 256 `outq` buckets. | **`R1a`** (L3 capacity & slice contention), **`R1b`** (DRAM bandwidth), **`R1c`** (DTLB/STLB translation). | `[ARGUED]` (microarchitecture) |
| **`R3` (SMT pairing, `k > 10`)** | **Wrong onset threshold.** `BzReader::parallel_workers(k)` (`src/read.rs` L245, `src/parallel.rs` L689) spawns `k` workers + `1` scanner + `1` `mibz2-read` feeder + `1` caller/reader (+ `1` `#838` RSS sampler) = **`k + 3` (`k + 4`) threads**. With scanner and reader continuously active, physical-core oversubscription on 10 cores begins at **`k = 8–9`**, not `k = 11`, and acts **asymmetrically** when the serial scanner shares a core with a worker. This directly explains why `TRIMMED_WORKERS = 7` beat `10` unpinned by `12.7%` (`0.873 [0.866, 0.880]`, `src/parallel.rs` L3414) and why `four-arm` paired CI width explodes **7×** at `k = 10` (`4.15%` vs `0.54–0.65%` at `k = 1, 6`). | **`R3`** (hardware SMT execution-port/L1/L2/STLB sharing, onset at `k = 8–9` for active threads, split by **scanner-on-SMT** vs **worker-on-SMT**) separated from **`R5`** (CFS placement/migration). | `[PROVED]` (thread count & `four-arm` variance) / `[ARGUED]` (CFS asymmetry) |

### Missing elements, dropped elements, and covariates

1. **`S8` — Serial output retirement & sink (MISSING, `[PROVED]`):** Already visible in `measurements-four-arm.tsv`: with the scanner removed (Arm `C` vs `D`), `k·(C − D)` — the parallel work saved by the emission oracle — is `0.225 s` at `k = 1` and `0.220 s` at `k = 6`, but **collapses by half to `0.115 s` at `k = 10`** (`D/C = 0.920`, while Arm `D` achieves only `6.48×` speedup at `k = 10`, `k·T_D(10)/T_D(1) = 1.543`), even though the block census proved per-row emit time grows `1.12×` rather than shrinking. Something serial or globally saturating (`S8` reader/head-of-line retirement, Arm `C`'s serial `Symbols` replay feeder, `S9` straggler tail, or `R1`/`R4`) is already masking half of emission's gain at `k = 10`.
2. **`S9` — Pipeline ramp-up, makespan straggler tail, and fresh-process cold start (MISSING, `[PROVED]` / `[ARGUED]`):** On Wikidata ×4 (`1,724` blocks across `k = 20` workers = `86` blocks/worker), ramp-up (`k` blocks scanned before worker `k` starts: `~2.5 ms`) plus the final makespan tail (a single dense `m = 232k` block takes `~2.5 ms`, `2.3×` median) plus `#838`'s fresh-process (`--one`) thread spawn and first-touch faults create an additive floor `c_0(k)` that grows with `k`.
3. **`R4` — Core clock frequency scaling (MISSING, `[ARGUED]` / `[CONJECTURED]`):** On an i9-10900K, 1-core turbo (`~5.2–5.3 GHz`) drops to 10-core all-core turbo (`~4.8–4.9 GHz`, an `~8–10%` clock loss before any AVX2/PL1 throttling). In `2026-11-24-wikidata-block-census.md` (L148–154), per-block `emit` and `construct` both slowed by an **identical, block-feature-independent `1.12×`** from `k = 1` to `k = 10` (`p10–p90: 1.11–1.14` emit, `1.11–1.18` construct; held-out `R² ≤ 0.023`). A uniform `~1.12×` inflation across both L2-compute-heavy `construct` and random-read-heavy `emit`, invariant to `m` and locality `d`, is the signature of **`R4` (~10% turbo drop)** plus a `~2%` cache/SMT term — not DRAM saturation. Without `cycles / ref-cycles`, `R4` is misattributed to memory contention.
4. **`R5` — OS CFS thread placement and migration (MISSING, `[ARGUED]`):** Distinct from hardware `R3`. Unpinned CFS stochastically co-schedules the scanner onto a worker's SMT sibling on some rounds and onto an idle core on others, inflating `k = 10` variance `7×`.
5. **Drop NUMA (`[PROVED]`):** i9-10900K is a single-socket monolithic 10-core ring-bus die (1 NUMA node).
6. **Reclassify block heterogeneity as a per-block covariate vector `x_b = (n, m, runs, strips, syms, comp_bytes, route)` (`[PROVED]`):** Not a stage or resource. It enters twice: (i) within-block service times (`construct` already hits `R² = 0.964` from `(m, n/runs, strips/m)`, and `155` refused-run + `48` column blocks are `47%` of Wikidata's 431 blocks while holding `2.1%` of rows, driving per-block lock/header rate); (ii) service-time dispersion `CV²` (`m` spans `68k–232k`, `3.4×`), which widens the crossover knee and triggers `READY_BYTES` head-of-line stalls (`S3b`) and tail makespan (`S9`).
7. **Factor cold vs warm cleanly into three layers (`[PROVED]`):**
   - *OS page cache:* `#838` pre-reads the `47.4 MB` input into RAM before timing, so `S0` is memory-to-memory `ChannelSource` copy, not disk I/O (state explicitly).
   - *Heap / first-touch minor faults (`R2`):* `#838`'s fresh process (`--one`) measures cold heap (`~80–94 MB` faulted at `k = 20` inside the clock); a 2nd in-process decode isolates warm-heap steady state.
   - *CPU caches:* A `838.8 MB` decode (`42×` the `20 MiB` L3) is inherently decode-order streaming; single-block hot-L2 (`calib.rs`) is a separate regime.

---

## 2. Minimal Experimental Design Across `k = 1–20` (Linear, Not Exponential)

### Why small perturbations (`δ = 5–10%`) fail on the i9-10900K `[PROVED]`
From `measurements-four-arm.tsv` (`20` paired rounds on the i9-10900K), the implied single-round paired log-ratio SD $\sigma_{\text{pair}}$ is:
- `k = 1` (`B/A = 0.830 [0.825, 0.834]`): rel. half-width `0.54%` $\Rightarrow \sigma_{\text{pair}} \approx 1.16\%$.
- `k = 6` (`B/A = 0.850 [0.844, 0.855]`): rel. half-width `0.65%` $\Rightarrow \sigma_{\text{pair}} \approx 1.38\%$.
- `k = 10` unpinned (`B/A = 0.976 [0.936, 1.017]`; `C/A = 0.806 [0.782, 0.831]`; `D/C = 0.920 [0.901, 0.939]`): rel. half-width `2.07–4.15%` $\Rightarrow \sigma_{\text{pair}} \approx \mathbf{4.4\%\text{–}8.9\%}$.

For a proportional stage slowdown $\delta$ and $R$ paired rounds, the 95% CI half-width on $\hat{C}_i = -\ln(T_{\text{pert}}/T_{\text{base}})/\ln(1+\delta)$ is
$$\Delta C_i \approx \frac{t_{R-1, 0.975}\,\sigma_{\text{pair}}}{\sqrt{R}\,\ln(1+\delta)}.$$
To obtain $\Delta C_i \le \pm 0.05$:

| Regime ($\sigma_{\text{pair}}$) | $\delta = 0.05$ | $\delta = 0.10$ | $\delta = 0.20$ | $\delta = 0.25$ |
|---|--:|--:|--:|--:|
| Quiet (`k = 1, 6`, `1.3%`) | `110` rounds | `29` rounds | `8` rounds | **`6` rounds** |
| Unpinned `k = 10` (`4.4–8.9%`) | `1,250–5,110` rounds | `328–1,340` rounds | `90–367` rounds | `60–245` rounds |
| Pinned `k = 10` (`~1.5–2.5%` `[CONJECTURED]`) | `146–404` rounds | `39–106` rounds | `11–29` rounds | **`7–20` rounds** |

*(Asymptotic $z = 1.96$; small-$R$ $t_{R-1,0.975}$ adds `+2–3` rounds below $R = 15$.)* **Design consequence:** `δ = 5–10%` unpinned is unusable near `k = 10`. Pinning (removing `R5`) plus **`δ = +0.25`** cuts the sample requirement by **`~100×`**.

### The 4-Tier Minimal Design (~90 conditions total, ~35–45 min workstation time)

| Tier | Purpose | Counts (`k`) | Perturbations / Knobs | Rounds & Setup | Precision Bought |
|---|---|---|---|---|---|
| **1. Boundary + PMU Grid** *(Primary)* | Full stage shares, wait breakdown, `f(k)`, LLC/TLB/DRAM curves, block-level `x_b` & `CV²`, exact `k*` | **1–20 pinned** (physical-first: scanner core 0, workers cores `1..9` then siblings `10..19`) + **9 unpinned anchors** `{1,4,6,8,9,10,12,16,20}` (or `#838`'s 1–20 unpinned) | None (timers on vs untimed check, as `#838`); 12–14 `CLOCK_MONOTONIC_RAW` / `rdtsc` marks/block (`~250 ns` / `1 ms` = `<0.03%` overhead) covering `S0, S1, S2, S3a/b, S4, S5a, S5b, S5c, S6, S7a, S8` + route tag; per-block dump for 2 rounds | `R = 10` rounds × 2 zero-mux PMU passes (fresh process `--one`, matching `#838`) | Stage shares **`±0.3–0.8%`** of wall; `f(k)` **`±0.2%`**; DRAM GB/s **`±1%`**; `k*` **`±0.3` workers**; `R5` penalty **`±1%`** |
| **2. Orthogonal Resource Knobs** *(Identifies `R1–R4, S3b, S8`)* | Separates shared hardware resources without crossing them (`O(#R)`) | **6 anchors:** `{1, 4, 7, 10, 14, 20}` (serial, worker-bound, trim knee, crossover/core-full, mid-SMT, full-SMT) | **6 single-knob paired arms** (36 conditions): (1) **Freq lock** (`R4`, fixed all-core turbo or turbo-off); (2) **THP / `MADV_HUGEPAGE`** (`R1c`); (3) **2nd back-to-back in-process decode** (`R2` warm vs cold heap); (4) **Fixed-`k=4` on 4 cores vs 2 SMT pairs** (`R3` pure per-stage SMT factor $\phi_{\text{SMT},i}$ at fixed L3/BW); (5) **`READY_BYTES ∈ {1, 12, 64} MiB`** (`R1a` + `S3b` head-of-line); (6) **`memcpy` output sink** (`S8` real caller copy) | `R = 8` paired AB/BA rounds, pinned | Each resource effect to **`±0.8–1.5%`** (95% CI) |
| **3. Stage Slowdowns** *(Validation only)* | Tests whether model-predicted control coefficients match real derivatives, especially at `k*` | **5 counts:** `{1, 6, k*, 14, 20}` (`k*` fixed from Tier 1 before running) | **6 active stages** (`S2, S4, S5b, S5c, S6, S7a`; add `S8` only if Tier 1 `S8 > 3%`), **`δ = +0.25`** proportional per-block **cycle-calibrated pure-ALU spin** (never `sleep()`, which yields cores and alters `R3/R4`); plus **1 curvature pair** `δ ∈ {+0.15, +0.35}` on `S2, S6` at `k = k*` (34 conditions) | `R = 10` paired AB/BA rounds, pinned | **`±0.05`** per $C_i$ at `σ = 1.5%` (`±0.08` at `2.5%`), **`±0.12`** on $\sum C_i$ |
| **4. Held-Out Validation** | Tests completeness out-of-sample (0 extra runs) | Train on **7** (`{1,3,6,9,12,16,20}`), score **13** held-out counts + Tier 3 + four-arm (`B, C, D`) | None | Uses Tier 1–3 data | Detects any missing mechanism causing `>3%` wall residual or `>0.12` control error |

**Counters (zero multiplexing, 2 core passes + uncore, reusing `src/phase.rs`'s `perf_event_open` infrastructure):**
- *Core Pass 1 (4 GP/fixed counters):* `cycles`, `ref-cycles` (yields exact $f_{\text{eff}}(k) = f_{\text{base}} \cdot \text{cycles}/\text{ref-cycles}$), `instructions`, `LLC-load-misses`.
- *Core Pass 2 (4 counters):* `L2_RQSTS.MISS`, `dTLB-load-misses` (`DTLB_LOAD_MISSES.WALK_COMPLETED`), `CYCLE_ACTIVITY.STALLS_MEM_ANY`, `sw:page-faults` (`minflt`).
- *Uncore (system-wide, 0 mux, 0 overhead):* `uncore_imc/cas_count_read/` + `cas_count_write/` $\Rightarrow$ exact DRAM bytes $= 64\,(\text{read} + \text{write})$.
- *Process (from `#838`):* `VmHWM`, sampled mean RSS, `getrusage` (`utime`, `stime`, `minflt`, `nvcsw`, `nivcsw`).

**Wall-time budget:** Tier 1 (~15 min) already establishes ~85% of the character and pins `k*`; Tiers 2 and 3 add ~12 min and ~10 min respectively (total ~35–40 min vs `2^8 × 20 = 5,120` conditions).

---

## 3. What to Reason About Rather Than Measure

1. **Two proved traps in the MCA summation theorem $\sum_i C_i = 1$ `[PROVED]`:**
   - *Trap A (slowdown super-additivity at the crossover $k^*$):* By Euler's theorem on degree-1 homogeneous $J(v_1,\dots,v_m)$, $\sum_i \partial \ln J / \partial \ln v_i = 1$ holds wherever $J$ is differentiable. For a pipeline bottleneck $J \approx \min(v_s, k v_w)$, $J$ is **not differentiable at $k^*$**: the one-sided *slowdown* derivatives (the only direction injectable slowdowns measure) satisfy $C_s^- = 1$ **and** $C_w^- = 1$, so **$\sum_i \hat{C}_i^- \to 2 > 1$**! Even with block-size stochasticity smoothing the corner over a band $\Delta k \sim O(\sqrt{k^*}\,c_s)$, finite `+25%` slowdowns across $k^*$ strictly produce $1 < \sum \hat{C}_i^- \le 2$. Seeing $\sum \hat{C}_i^- \approx 1.4\text{–}1.8$ near $k^*$ is a mathematical consequence of a `min`-kink, not a broken measurement.
   - *Trap B (blindness/ambiguity on shared resources):* The summation theorem checks completeness of **wall time**, not of **resources**. If a non-CPU resource (e.g. DRAM bandwidth $B$) strictly caps throughput ($J = B/b$), slowing stages by pure-CPU spin yields $\sum C_i^{\text{CPU}} = 0 \ll 1$ (a shortfall *because* the perturbation did not scale DRAM bytes). Conversely, if a shared resource enters multiplicatively across all stages (`R4` turbo drop), proportional time slowdowns still sum to `1` while hiding `R4` completely.

2. **Exact DRAM bandwidth bounds and an L3 capacity cliff prediction `[PROVED]` / `[ARGUED]`:**
   - *L3-hit streamed floor (`src/open.rs`, `src/rle.rs`, `bench/wikidata.json`):* Per decoded byte, input is `47.42 / 838.80 = 0.057 B/B`; `syms`/`records` are `~0.3–0.6 B/B`; `out` requires `1.0 B/B` write (`+1.0 B/B` RFO write-allocate when cold in L3) and `unrle_in_place` re-reads `out` from L3 (`900 KiB > 256 KiB` L2, L3-hit if `out` stays in L3). Total L3-resident DRAM traffic is **`~2.0–3.5 B/B`**. On dual-channel DDR4 (`~35–42 GB/s` practical), DRAM cannot saturate below **`J ≈ 40 / 3.5 = 11.4 GB/s`** — `2.3–3.1×` above observed `J(10) = 3.71 GB/s` (Arm `A`) and `5.01 GB/s` (Arm `D`).
   - *L3-spilled random-access ceiling:* Each `4 B` random miss in `S6` (`emit_bidirectional`, `m/n ≈ 0.12` avg, `0.26` max) pulls a `64 B` line (`64 × 0.12 = 7.7 B/B` read), and `S5c` (`wire_xor` RMW on `edges[v]`) moves `2 × 64 × 0.12 = 15.4 B/B` (read + dirty writeback) $\Rightarrow$ **`~25 B/B` DRAM** $\Rightarrow$ DRAM saturates at **`J ≈ 40 / 25 = 1.6 GB/s`**.
   - *Deduction (`[PROVED]` from `J(10) = 5.01 GB/s`, `[CONJECTURED]` cliff):* Since `5.01 GB/s` requires average DRAM traffic `≤ 40 / 5.01 = 8.0 B/B`, **`edges` is still mostly L3-resident at `k = 10`** (`10 × 0.45 MB = 4.5 MB < 20 MiB`). However, live L3 demand is $M_{\text{live}}(k) \approx k \cdot (W_{\text{edges}} + W_{\text{out}} + W_{\text{syms}} + W_{\text{scratch}}) + \min(\text{READY\_BYTES}, \dots) \approx k \cdot 1.8\text{ MB} + 12\text{ MiB}$, crossing `20 MiB` around `k ≈ 5–10` and reaching **`~47 MiB` by `k = 20`**. We therefore predict a sharp L3-spill transition (`LLC-load-misses` and DRAM `B/B` jumping steeply) between `k = 10` and `k = 16`, directly movable by Tier 2's `READY_BYTES` knob.

3. **Closed-form crossover $k^*$, analytical $C_i$ away from $k^*$, and unmeasured worker ceilings `[PROVED]` / `[ARGUED]`:**
   - With mean per-block scanner time $t_s(k) = t_s(1)\,g_s(k)$, worker time $t_w(k) = t_w(1)\,g_w(k)$, and serial reader time $t_r(k)$,
     $$T_{\text{wall}}(k) \ge N_{\text{blocks}} \cdot \max\!\left(t_s(k),\; t_r(k),\; \frac{t_w(k)}{k}\right) + T_{\text{pipe}}(k), \qquad k^* = \frac{t_w(1)}{t_s(1)} \cdot \frac{g_w(k^*)}{g_s(k^*)}.$$
     Note the sign: `k*` is **not** fixed at $t_w(1)/t_s(1)$. Pinned scanner (`g_s ≈ 1.10`, turbo only) vs contending workers (`g_w ≈ 1.25`) shifts `k*` **up**; unpinned scanner landing on a worker's SMT sibling (`g_s ≈ 1.35`) shifts `k*` **down**.
   - Away from $k^*$, $C_i$ follows directly from Tier-1 stage shares without perturbations: for $k \ll k^*$, $C_{\text{scan}} = 0$ and $C_j = t_j(k)/t_w(k)$; for $k \gg k^*$, $C_{\text{scan}} = 1$ and $C_j = 0$ (except cross-stage resource theft, see §4).
   - **Pre-Tier-1 analytical estimate of `T2`/`T3`/`T6` shares at `k = 1` (`[ARGUED]`):** Combining `2026-11-20-open-stage-attribution.md` (Xeon 1-thread ratios: Open `75%`, `front` `S4` `6.2%`, `RLE1+CRC` `S7a` `5.7%`, `scan` `12.8%`) with the i9 census (`construct` `5.90` vs `emit` `4.83 ns/row`, ratio `1.22×`) and `four-arm` `k = 1` emit ceiling (`17.0%`), **`S5` (`T2` construction) is predictably `~21–24%` of 1-worker time** (larger than `T1` emission), while `S4` (`T3`) and `S7a` (`T6`) are `~6–8%` each. Tier 1 verifies these to `±0.5%` in one pass.

4. **Head-of-line `READY_BYTES` throttling at `k ≥ 10` `[PROVED]` (mechanism) / `[CONJECTURED]` (magnitude):**
   - `Shared::full` (`src/parallel.rs` L402–408) blocks the scanner whenever `ready.bytes ≥ 12 MiB` (`≈ 13.3` full blocks) **and** `ahead ≥ k + 2`. At `k ≥ 10`, once `14` completed blocks queue behind a single slow block (`m = 232k`, `2.3×` median), only **`2` slack blocks** remain above `k` workers before the scanner stalls (`SlotWait`), even when `slots = 4k = 40..80` has ample room. This is purely architectural and directly quantified by Tier 2 Knob #5 (`READY_BYTES = 64 MiB`).

5. **Double Amdahl `[PROVED]`:** Any cooperative-team plan (`ServicePlan`, `Teams`) or future parallel scanner obeys nested Amdahl bounds — once across scanner vs workers, and again across leader-only stages (`MTF, RLE1, CRC`) vs helper-shared stages (`build, walk`), or block-boundary search vs parallel Huffman decode.

---

## 4. Formalism: Control Coefficients vs Elasticities, Coz, and Roofline

### Verdict on MCA Flux Control Coefficients ($C_i^J = \partial \ln J / \partial \ln v_i$)
**Right question; wrong primary instrument — and incomplete without MCA's other half (elasticities).** `[PROVED]` / `[ARGUED]`
1. **Tropical (`min`) degeneracy:** Metabolic pathways are smooth Michaelis–Menten networks with distributed control; a block-parallel pipeline is near a piecewise-linear $\max$/$\min$ system. Away from $k^*$, $C_i$ is trivially `0` or the Tier-1 time share $t_i/t_w$. Near $k^*$, slowdown-only differences hit the non-differentiable corner ($\sum \hat{C}_i^- \in (1, 2]$).
2. **Missing half of MCA — local elasticities $\varepsilon_r^i = \partial \ln v_i / \partial \ln x_r$:** In Kacser–Burns / Reder (1988) systems biology, $C_i^J$ is a *derived output*; the *primitives* are the local elasticities $\varepsilon_r^i$ (how stage $i$'s rate responds to shared resource $x_r$: L3 occupancy, DRAM BW, frequency, SMT load) and the resource-load matrix $N$ (cycles, `Q`, `S` bytes per block). **Concrete failure mode without elasticities:** if `k = 10` is scanner-bound ($C_{\text{scan}} \approx 1$) and `S6` (`emit`) contends with the scanner on L3 or an SMT sibling, slowing `S6` with a pure-CPU delay *reduces* `S6`'s L3/port load on the scanner and can make whole-decoder throughput **faster** — a **negative** control coefficient $C_{S6}^J < 0$. Only elasticity + resource accounting distinguishes critical-path membership from cross-stage resource theft.
3. **Noise cost:** As proved in §2, small `5–10%` perturbations need `100–4,800` rounds. Keep $C_i$ strictly as **Tier-3 validation** at `δ = +0.25`.

### Verdict on Coz Causal Profiling (Curtsinger & Berger, SOSP 2015)
- **Strength `[PROVED]`:** By pausing *peer* threads whenever a sample hits code point $i$, Coz estimates **virtual speedup** (the *right-hand* derivative $C_i^+$ and full speedup curve `0–100%`) in a single run, avoiding the slowdown-only $\sum C_i^- \to 2$ kink artifact and matching the optimization direction of the four-arm oracle.
- **Two structural blinds `[PROVED]` / `[ARGUED]`:** (1) Pausing peer threads artificially **relieves** DRAM bandwidth, L3 pressure, and SMT contention during the virtual speedup, so Coz systematically *overestimates* real speedup whenever `R1/R3/R4` bind; (2) `0.18–0.23 s` decodes yield only `~200` samples at `1 ms` `perf` periods, requiring a **30–60 s looped warm-heap decode** (dropping `R2` cold-heap effects).
- **Role:** Worth a cheap 10-minute diagnostic run at `k ∈ {6, 10, 14}` (progress point = block completed) to cross-check critical-path masking; **not** a substitute for resource-aware modeling.

### Recommended Formalism: Coupled Stage-Roofline + Trace-Driven Pipeline + MCA Validation
The true systems-biology equivalent of *(stoichiometry + enzyme kinetics + conservation laws)* is a **3-layer mechanistic model**:

1. **Layer 1 — Per-Stage Microarchitectural Roofline (= kinetics & local elasticities $\varepsilon_r^i$):**
   For each stage $i \in \{S0..S8\}$ and block covariate vector $x_b$,
   $$t_{i,b}(k) = \frac{\phi_{\text{SMT}, i}(k)}{f_{\text{eff}}(k)} \left[\frac{W_{\text{cpu}, i}(x_b)}{\text{IPC}_i} + Q_i(x_b)\,L_{\text{eff}, i}\!\big(M_{\text{live}}(k)\big)\right] \;\oplus\; \frac{B_{\text{DRAM}, i}\!\big(x_b, M_{\text{live}}(k)\big)}{B_{\text{avail}}(k)},$$
   where $\oplus$ is additive for dependent latency + streamed tail or soft-max where overlapped; $f_{\text{eff}}(k)$ is measured directly (`R4`, 0 free parameters); $\phi_{\text{SMT}, i}(k)$ comes from Tier-2 Knob #4 (`R3`); and $M_{\text{live}}(k)$ drives L3 miss probability $p_{\text{miss}}(M_{\text{live}})$ (`R1a`), which fixes both dependent miss latency $L_{\text{eff}}$ and DRAM bytes $B_{\text{DRAM}}$ (`R1b`, tied directly to the referee's `(Q, S)` vector).
2. **Layer 2 — Exact Trace-Driven Discrete-Event Pipeline Simulator (= pathway topology):**
   Rather than approximating `Shared::full` (`ahead < 4k` AND `!(ready.bytes ≥ 12 MiB && ahead ≥ k + 2)`) and Wikidata's 12-window block heterogeneity with a closed-form M/G/k/B formula, feed the 1,724 predicted triples $(t_{s,b}(k), t_{w,b}(k), t_{r,b}(k))$ into a ~50-line discrete-event simulator of `src/parallel.rs`'s exact state machine (`< 1 ms` runtime in Python) plus additive cold-start/fault term $T_{\text{init}}(k)$ (from Tier-2 Knob #3). This has **zero queueing-approximation error**, reproduces `READY_BYTES` head-of-line stalls and makespan tails exactly, and generates both exact numerical $C_i^{\text{model}}(k)$ and large-perturbation predictions (`four-arm` `B, C, D`, parallel scanner, `READY_BYTES` tuning) directly.
3. **Layer 3 — MCA Control Coefficients & Pre-Registered Falsification Rule:**
   Fit Layer 1+2 on the **7 training counts** `{1, 3, 6, 9, 12, 16, 20}` (Tier 1+2). Declare the decomposition **INCOMPLETE** if any of three criteria trips:
   - *(a) Held-out curve:* wall-time residual exceeds **`3%`** at **`≥ 3` of the 13 held-out counts** (pinned);
   - *(b) Perturbation validation:* any Tier-3 measured `δ = +0.25` control coefficient deviates from the model's predicted `δ = +0.25` finite difference by **`> 0.12`** outside its 95% CI;
   - *(c) Accounting closure:* Tier-1 stage timers + measured wait/init terms leave **`> 5%`** of wall time unattributed at any `k ∈ 1..20`.
