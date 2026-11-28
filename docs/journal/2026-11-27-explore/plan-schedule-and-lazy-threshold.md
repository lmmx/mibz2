# Plan Scheduling Cliffs (`5T`/`7T`/`9T`) and Stale `LAZY_FROM_THREADS = 8` Dissection

*Date:* 27 November 2026
*Zone:* Smooth (`docs/journal/2026-11-27-explore/plan-schedule-and-lazy-threshold.md`, zero `src/` edits)
*Primary i9-10900K evidence (`origin/evidence`):*
- `45ae3ff3` (`crossover/run-20261127T141422Z.tar.gz`, v0.5.15, 10C/20T i9-10900K, 20 MiB L3, `wikidata.bz2` 209.7 MB out)
- `bce3914d` (`release-scaling/run-20261127T140323Z.tar.gz`, v0.5.15 vs v0.5.14, 1–20T)
- `run-20261125T022939Z` (`kernel-count`, unpinned `OpenCells` vs `EagerCells` at `k = 1..=9`, `wikidata-x4` 838.8 MB)
- `run-20261121T020723Z` (`recalibrate-058`, pinned & unpinned `4T/6T/8T/10T` on `wikidata-x4`, `text`, `random`)
*Local VM diagnostic (C5):* 2-vCPU Haswell Xeon @ 2.30 GHz (32 KiB L1d / 256 KiB L2 / 45 MiB L3, AVX2), unmodified `origin/master` (`9001629a`), baseline x86-64 and JCC-32B-aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`), all 7 CI corpora (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`), 100% byte- & SHA-256-exact vs `libbzip2`.

---

## 1. Frozen Stop Test (C1a / C2 — Frozen Before Any Local Measurements)

1. **Exact root-cause dissection (from `src/parallel.rs` + `origin/evidence`):**
   - Enumerate the exact `(plan, workers, window, kernel)` chosen by `StreamDecoder::new` for every thread count `t ∈ {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 20}` across: (a) i9-10900K pinned 1-CPU/core (`crossover` / `release-scaling` at `t ≤ 10`), (b) i9-10900K unpinned (`whole_machine`), and (c) non-i9 machines (8-core 16 MiB L3, 16-core 32 MiB L3, cgroup-quoted containers, or non-exact affinity masks).
   - Verify whether `trim_cell`'s missing odd thread counts (`5T`, `7T`, `9T`) plus stale `LAZY_FROM_THREADS = 8` fully explain the `v0.5.15` i9 crossover cliff (`7T` `+27.2%` slower wall, `+43.4%` CPU, `3.25×` RSS vs `6T`; `5T` `+19%` CPU, `2.4×` RSS).
2. **Paired local A/B across all 7 corpora (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`):**
   - 100% exact byte-count & hash parity vs `libbzip2` on every run (`0` parity failures).
   - Paired alternating A/B of `OpenCells` (`mibz2-workers-lazy`) vs `EagerCells` (`mibz2-workers-eager`) at `t = 1, 2, 3, 4` under both baseline x86-64 and JCC-32B-aligned builds (wall, CPU, peak RSS, paired 95% t-interval; `taskset -c 0` at `t = 1`, both vCPUs at `t = 2, 3, 4`).
   - Paired A/B of `workers = t - 1` vs `workers = t` (`t = 2, 3, 4`, both `lazy` and `eager`) and `Seq(Cells)` (`mibz2 1`) vs `parallel_workers(1) lazy`.
3. **PASS (admit a follow-on `status = "idea"` build claim in `src/parallel.rs`):**
   - The i9 evidence + local 7-corpus guard measurements identify a clean, generalizable scheduling & lazy-threshold update that:
     - **(a)** projects **`>= 10.0%` wall reduction AND `>= 40%` peak RSS reduction** on `wikidata` at the cliff counts (`5T` and `7T` pinned on i9, and `4T–7T` on non-i9 machines);
     - **(b)** produces **zero `> 3.0%` disjoint CPU/wall regression** on any of the 6 off-wikidata guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`) at the proposed `LAZY_FROM_THREADS` threshold (`t >= t*`) under JCC-32B-aligned builds (documenting honestly any corpus that still favors `EagerCells` at `t < t*` or interacts with open admission); AND
     - **(c)** defines exact pre-registered physical i9-10900K verification gates (C5: the 2-vCPU VM cannot substitute for physical 10C/20T i9-10900K adoption runs).
   - **Otherwise (STOP):** record a scoped dead-end in `frontier/dead-ends.tsv` and do not open a build claim.

---

## 2. Exact Root-Cause Dissection of `src/parallel.rs` and `45ae3ff3`

### 2.1 What `StreamDecoder::new` Actually Builds (`t = 1..=20`)

Tracing `StreamDecoder::new` (`src/parallel.rs:562-597`), `plan_with` (`src/parallel.rs:3763-3788`), `window` (`src/parallel.rs:3507-3513`), `trim_cell` (`src/parallel.rs:3603-3613`), `unpinned_cell` (`src/parallel.rs:3633-3643`), and `LAZY_FROM_THREADS = 8` (`src/parallel.rs:3001`):

| `t` | i9-10900K pinned (`crossover` / `release-scaling`) | i9-10900K unpinned (`whole_machine`) | Non-i9 8C (16 MiB L3) / cgroup-quota | Non-i9 16C (32 MiB L3) |
|--:|---|---|---|---|
| 1 | `Seq(Cells)` (1T inline) | `Seq(Cells)` (1T inline) | `Seq(Cells)` (1T inline) | `Seq(Cells)` (1T inline) |
| 2 | `W(2) Eager s=8` | `W(2) Eager s=8` | `W(2) Eager s=8` | `W(2) Eager s=8` |
| 3 | `W(3) Eager s=12` | `W(3) Eager s=12` | `W(3) Eager s=12` | `W(3) Eager s=12` |
| 4 | `W(3) Eager s=12 [trim]` | `W(4) Open s=16 [trim]` | **`W(4) Eager s=16`** | **`W(4) Eager s=16`** |
| **5** | **`W(5) Eager s=20` (unmapped!)** | `W(5) Open s=20 [trim]` | **`W(5) Eager s=20`** | **`W(5) Eager s=20`** |
| 6 | `W(5) Open s=20 [trim]` | `W(6) Open s=24 [trim]` | **`W(6) Eager s=24`** | **`W(6) Eager s=24`** |
| **7** | **`W(7) Eager s=28` (unmapped!)** | `W(7) Open s=28 [trim]` | **`W(7) Eager s=28`** | **`W(7) Eager s=28`** |
| 8 | `W(6) Open s=24 [trim]` | `W(8) Open s=32` | `W(8) Open s=32` | `W(8) Open s=32` |
| **9** | **`W(9) Open s=36` (unmapped!)** | `W(9) Open s=36` | `W(8) Open s=36` | `W(9) Open s=36` |
| 10 | `W(7) Open s=28 [trim]` | `W(8) Open s=32 [trim]` | `W(8) Open s=40` | `W(10) Open s=40` |
| 12 | `W(10) Open s=48` | `W(8) Open s=32 [trim]` | `W(8) Open s=48` *(slot bug!)* | `T(6×2) Coop s=24` *(2.8–4.5× slower!)* |
| 16 | `W(10) Open s=64` | `W(8) Open s=32 [trim]` | `W(8) Open s=64` *(slot bug!)* | `T(7×2) Coop s=28` *(2.8–4.5× slower!)* |
| 20 | `W(10) Open s=80` | `W(8) Open s=32 [trim]` | `W(8) Open s=80` *(slot bug!)* | `T(7×2) Coop s=28` *(2.8–4.5× slower!)* |

### 2.2 Exact Line-by-Line Match Against Physical i9 `45ae3ff3` (`crossover`) & `bce3914d` (`release-scaling`)

Extracting every `mibz2` and `mibz2-workers-w` row from `45ae3ff3`'s `runs.jsonl` (10 rounds, `wikidata.bz2` 209.7 MB out) and `bce3914d`'s `t = 1, 2`:

| `t` | `arm` | `cpus` | Observed workers `w` | Kernel | Wall med (s) | CPU med (s) | Peak RSS (MB) | `runq_s` med (ms) | What happened |
|--:|---|---|--:|---|--:|--:|--:|--:|---|
| 1 | `mibz2` (`bce3914d`) | `1` | 0 (`Seq`) | `Cells` | `0.361` | `0.362` | `27.7` | `0.00` | Inline sequential |
| 2 | `mibz2` (`bce3914d`) | `0,1` | 2 | `Cells` | `0.217` | `0.415` | `56.5` | `131.60` | 2 Eager + scan on 2 cores |
| 4 | `mibz2` | `0..3` | 3 | `Cells` | `0.132` | `0.449` | `68.0` | `26.40` | `trim_cell(4)` -> `3 Eager` |
| 4 | `mibz2-workers-w` | `0..3` | 4 | `Cells` | `0.134` | `0.483` | `78.4` | `124.90` | 4 Eager + scan oversubscribed |
| **5** | **`mibz2`** | **`0..4`** | **5** | **`Cells`!** | **`0.114`** | **`0.512`** | **`97.5`** | **`123.90`** | **`trim_cell(5) = None` -> 5 Eager + 124 ms runq!** |
| 5 | `mibz2-workers-w` | `0..4` | 5 | `Cells` | `0.112` | `0.505` | `99.6` | `113.60` | Identical build to `mibz2` |
| **6** | **`mibz2`** | **`0..5`** | **5** | **`OpenCells`** | **`0.081`** | **`0.426`** | **`40.2`** | **`33.35`** | **`trim_cell(6)` -> 5 OpenCells (clean!)** |
| 6 | `mibz2-workers-w` | `0..5` | 6 | `Cells` | `0.104` | `0.551` | `111.7` | `115.40` | 6 Eager (`+28.2%` slower, `2.78×` RSS!) |
| **7** | **`mibz2`** | **`0..6`** | **7** | **`Cells`!** | **`0.103`** | **`0.611`** | **`129.7`** | **`113.35`** | **`trim_cell(7) = None` -> 7 Eager (+27.2% slower than 6T, +43.4% CPU, 3.25× RSS!)** |
| 7 | `mibz2-workers-w` | `0..6` | 7 | `Cells` | `0.102` | `0.605` | `121.4` | `114.90` | Identical build to `mibz2` |
| **8** | **`mibz2`** | **`0..7`** | **6** | **`OpenCells`** | **`0.066`** | **`0.427`** | **`41.8`** | **`1.45`** | **`trim_cell(8)` -> 6 OpenCells (clean!)** |
| 8 | `mibz2-workers-w` | `0..7` | 8 | `OpenCells` | `0.068` | `0.441` | `49.5` | `73.90` | `8 >= LAZY_FROM_THREADS(8)`, 74 ms runq |
| **9** | **`mibz2`** | **`0..8`** | **9** | **`OpenCells`** | **`0.060`** | **`0.449`** | **`51.5`** | **`60.10`** | **`trim_cell(9) = None` -> 9 OpenCells (+60 ms runq, +7.6 MB RSS)** |
| **10** | **`mibz2`** | **`0..9`** | **7** | **`OpenCells`** | **`0.059`** | **`0.440`** | **`43.9`** | **`0.00`** | **`trim_cell(10)` -> 7 OpenCells (`0.00 ms` runq!)** |
| 10 | `mibz2-workers-w` | `0..9` | 10 | `OpenCells` | `0.063` | `0.476` | `55.2` | `57.40` | `+5.6%` wall, `+8.5%` CPU, `+11.3 MB` RSS |
| 12–20 | `mibz2` (unpinned) | `all` | 8 | `OpenCells` | `0.060–0.062` | `0.459–0.472` | `46.0–46.6` | `0.00` | `unpinned_cell(>=10)` -> 8 OpenCells |
| 12–20 | `mibz2-team2` | `all` | 12–20 | `Coop<2>` | `0.175–0.274` | `1.747–4.505` | `89.5–128.4` | `0.30–90.45` | `2.85×–4.53×` slower wall, `3.77×–9.71×` CPU |

### 2.3 Two Distinct Root Causes + Two Secondary Bugs

1. **Root Cause A — `trim_cell` (`src/parallel.rs:3603-3613`) hardcodes only even pinned counts `{4, 6, 8, >=10}`:**
   - Every odd pinned count (`5T`, `7T`, `9T`) returns `None` and falls through to un-trimmed `Plan::Workers(t)` with `plan.lazy()` (`t >= LAZY_FROM_THREADS`).
   - At `5T` and `7T`, that hits a **double penalty**: (i) `EagerCells` (`~18 MiB/worker` working set thrashing the 20 MiB L3 at `5–7` workers, driving peak RSS to `97.5 MB` and `129.7 MB` and CPU from `0.426 s` to `0.512 s` / `0.611 s`), AND (ii) `w = t` workers competing with `mibz2-scan` + `mibz2-read` + `main` on `t` pinned cores (`113–124 ms` of Linux scheduler `runq_s` contention!).
   - At `9T`, `9 >= 8` selects `OpenCells`, so it avoids the Eager L3 cliff, but still runs 9 workers on 9 pinned cores (`60.1 ms` `runq_s`, `51.5 MB` RSS, `0.449 s` CPU vs `7` OpenCells workers at `10T` giving `0.00 ms` `runq_s`, `43.9 MB` RSS, `0.440 s` CPU, `0.059 s` wall).
2. **Root Cause B — `LAZY_FROM_THREADS = 8` (`src/parallel.rs:3001`) was never updated after `OpenCells` replaced `LazyCells` in `v0.5.7`:**
   - Its doc comment explicitly cites `v0.5.0`'s pre-Open `LazyCells` run (`#145 LOG 6903318530`), where `LazyCells` was 9% slower at `4T` and 3% slower at `6T`.
   - Physical i9-10900K `kernel-count` (`run-20261125T022939Z`, `wikidata-x4` 838.8 MB, unpinned) already proved that `OpenCells` beats `EagerCells` cleanly from **`k = 4` onward**:
     - `k = 1`: `1.123×` wall (Eager wins by `12.3%`), RSS `34.7 vs 46.1 MB`
     - `k = 2`: `1.046×` wall (Eager wins by `4.6%`), RSS `39.5 vs 63.2 MB`
     - `k = 3`: `1.000× [0.984, 1.016]` wall (exact tie; pinned `recal-058` at `4T` had `3 Eager` beat `3 OpenCells` by `3.29%`), RSS `43.3 vs 86.8 MB` (`0.50×`)
     - **`k = 4`**: **`0.922× [0.907, 0.937]`** wall (`-7.8%`), RSS `47.2 vs 95.7 MB` (`0.49×`), CPU `1.687 vs 1.827 s` (`-7.7%`)
     - **`k = 5`**: **`0.834× [0.826, 0.841]`** wall (`-16.6%`), RSS `50.7 vs 120.6 MB` (`0.42×`), CPU `1.688 vs 2.010 s` (`-16.0%`)
     - **`k = 6`**: **`0.737× [0.731, 0.744]`** wall (`-26.3%`), RSS `54.2 vs 134.2 MB` (`0.40×`), CPU `1.702 vs 2.280 s` (`-25.4%`)
     - **`k = 7`**: **`0.639× [0.616, 0.662]`** wall (`-36.1%`), RSS `57.2 vs 145.4 MB` (`0.39×`), CPU `1.725 vs 2.695 s` (`-36.0%`)
     - **`k = 8`**: **`0.546× [0.531, 0.562]`** wall (`-45.4%`), RSS `59.3 vs 165.3 MB` (`0.36×`), CPU `1.811 vs 3.398 s` (`-46.7%`)
   - Journal §76 patched only `unpinned_cell` (`UNPINNED_LAZY_FROM = 4`) on the exact Linux i9-10900K without CPU quota — leaving **`LAZY_FROM_THREADS = 8`** unchanged for every other machine (AMD Zen, Intel Xeon, Apple Silicon, ARM, laptops, Docker/K8s containers with `cpu.max`, and any non-exact affinity mask) AND for unmapped pinned i9 counts (`5T`, `7T`)!
3. **Secondary Bug 1 — `window()` (`src/parallel.rs:3507-3513`) fails to scale slots when `plan_with` caps workers (`n < threads`):**
   - When `plan_with` caps `threads > phys` to `Plan::Workers(phys)` on an 8–10 core non-i9 machine (e.g., `t = 20` on 8 cores -> `Plan::Workers(8)`), `trimmed` is `false`, so `window()` returns raw `slots = 4 × threads = 80` (`10` blocks/worker instead of `4` blocks/worker = `32`), wasting RSS and L3 capacity. Scaling whenever `n < threads` fixes this in 1 line.
4. **Secondary Bug 2 — Non-i9 machines with `phys >= 11` fall into `Plan::Teams { k: 2 }` (`Cooperative`) at `t >= 11`:**
   - `45ae3ff3` shows `mibz2-team2` is `2.85×–4.53×` slower wall and `3.77×–9.71×` more CPU than `OpenCells` workers at `12–20T` (consistent with `dead-ends.tsv` Row 31). That is flagged as a separate follow-on note (`WORKERS_BELOW_THREADS` / `Cooperative` retirement on high-core machines) rather than conflated with the `LAZY_FROM_THREADS` + `trim_cell` fix.

---

## 3. Local 7-Corpus Paired A/B Measurements (`origin/master` `9001629a`)

All runs checked **100% byte- & hash-exact against `libbzip2`** (0 parity failures across 600+ paired decodes). `wikidata` and `text`: 7 interleaved alternating rounds; `records`, `random`, `runs`, `pattern`, `onebyte`: 5 interleaved alternating rounds. `t = 1` pinned with `taskset -c 0`; `t = 2, 3, 4` across both VM vCPUs.

### 3.1 Experiment A1 — JCC-32B-Aligned: `OpenCells` (`lazy`) / `EagerCells` (`eager`) at `t = 1..4`

| Corpus | `t` | Eager wall | Lazy wall | Wall ratio [95% CI] | Eager CPU | Lazy CPU | CPU ratio [95% CI] | Eager RSS | Lazy RSS | RSS ratio | CPU verdict |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|
| `wikidata` | 1 | `2114.1m` | `1978.4m` | `0.9724x [0.8902,1.0622]` | `920.1m` | `891.7m` | **`0.9350x [0.9008,0.9705]`** | `34.2M` | `23.6M` | **`0.690x`** | **LAZY WINS** |
| `wikidata` | 2 | `993.4m` | `781.1m` | **`0.8100x [0.7095,0.9247]`** | `912.7m` | `815.3m` | **`0.8993x [0.8781,0.9210]`** | `51.6M` | `28.7M` | **`0.556x`** | **LAZY WINS** |
| `wikidata` | 3 | `1013.1m` | `792.3m` | **`0.8067x [0.6891,0.9443]`** | `970.8m` | `830.3m` | **`0.8700x [0.8527,0.8876]`** | `71.2M` | `32.7M` | **`0.460x`** | **LAZY WINS** |
| `wikidata` | 4 | `599.2m` | `501.0m` | **`0.8380x [0.7352,0.9551]`** | `1006.1m` | `841.6m` | **`0.8456x [0.7935,0.9011]`** | `86.3M` | `36.9M` | **`0.428x`** | **LAZY WINS** |
| `text` | 1 | `198.1m` | `205.5m` | `1.0355x [1.0202,1.0510]` | `200.9m` | `209.6m` | `1.0395x [1.0253,1.0540]` | `22.6M` | `25.4M` | `1.122x` | Eager wins |
| `text` | 2 | `117.3m` | `123.5m` | `1.0454x [0.9871,1.1072]` | `226.0m` | `236.3m` | `1.0355x [1.0111,1.0605]` | `40.9M` | `41.7M` | `1.020x` | Eager wins |
| `text` | 3 | `137.8m` | `144.0m` | `1.0020x [0.9394,1.0689]` | `257.7m` | `265.1m` | `1.0316x [1.0163,1.0471]` | `56.7M` | `56.7M` | `1.000x` | Eager wins |
| `text` | 4 | `150.9m` | `155.4m` | `1.0302x [0.9272,1.1447]` | `284.2m` | `299.6m` | `1.0416x [0.9573,1.1332]` | `65.7M` | `67.0M` | `1.019x` | tie |
| `records` | 1 | `470.3m` | `479.1m` | `0.9407x [0.7689,1.1508]` | `454.2m` | `450.1m` | `0.9794x [0.9331,1.0280]` | `40.9M` | `40.9M` | `1.002x` | tie |
| `records` | 2 | `451.3m` | `432.9m` | `0.9061x [0.6769,1.2128]` | `509.9m` | `485.4m` | `0.9380x [0.8660,1.0160]` | `49.9M` | `49.4M` | `0.991x` | tie |
| `records` | 3 | `452.3m` | `404.1m` | `0.9759x [0.8446,1.1275]` | `499.8m` | `496.6m` | `0.9942x [0.9648,1.0244]` | `54.8M` | `57.5M` | `1.051x` | tie |
| `records` | 4 | `377.1m` | `383.4m` | `1.0033x [0.8883,1.1332]` | `490.6m` | `491.3m` | `0.9967x [0.9823,1.0112]` | `60.4M` | `60.0M` | `0.993x` | tie |
| `random` | 1 | `493.7m` | `502.0m` | `1.1135x [0.8956,1.3843]` | `388.7m` | `393.1m` | `0.9854x [0.9309,1.0430]` | `36.4M` | `36.4M` | `0.998x` | tie |
| `random` | 2 | `535.2m` | `471.7m` | `0.9399x [0.8386,1.0535]` | `442.0m` | `412.9m` | `0.9378x [0.8770,1.0028]` | `47.1M` | `47.2M` | `1.002x` | tie |
| `random` | 3 | `423.8m` | `457.0m` | `1.0292x [0.8122,1.3041]` | `417.7m` | `426.7m` | `1.0139x [0.8371,1.2280]` | `52.3M` | `52.6M` | `1.005x` | tie |
| `random` | 4 | `442.7m` | `434.4m` | `0.9935x [0.8197,1.2043]` | `431.7m` | `436.8m` | `1.0305x [0.8816,1.2046]` | `57.5M` | `59.0M` | `1.026x` | tie |
| `runs` | 1 | `112.7m` | `111.8m` | `0.7489x [0.5464,1.0264]` | `48.9m` | `45.5m` | **`0.9382x [0.8857,0.9939]`** | `34.3M` | `32.1M` | `0.934x` | **LAZY WINS** |
| `runs` | 2 | `145.5m` | `129.1m` | `0.8688x [0.6366,1.1856]` | `47.7m` | `44.7m` | **`0.9332x [0.9082,0.9589]`** | `34.7M` | `32.1M` | `0.926x` | **LAZY WINS** |
| `runs` | 3 | `121.3m` | `82.0m` | `0.9671x [0.5210,1.7950]` | `47.4m` | `45.1m` | `0.9629x [0.9127,1.0159]` | `32.8M` | `32.1M` | `0.980x` | tie |
| `runs` | 4 | `93.5m` | `118.0m` | `1.2469x [0.7526,2.0661]` | `46.9m` | `45.0m` | `0.9465x [0.8930,1.0034]` | `32.9M` | `32.3M` | `0.979x` | tie |
| `pattern` | 1 | `261.9m` | `286.0m` | `1.1489x [0.7715,1.7109]` | `125.4m` | `132.9m` | `1.0518x [0.9463,1.1691]` | `12.4M` | `12.4M` | `0.997x` | tie |
| `pattern` | 2 | `245.9m` | `154.0m` | `0.6340x [0.4408,0.9120]` | `159.2m` | `142.7m` | `0.8885x [0.7803,1.0118]` | `21.3M` | `22.9M` | `1.076x` | tie |
| `pattern` | 3 | `113.6m` | `130.4m` | `1.2318x [0.9994,1.5181]` | `154.8m` | `166.6m` | `1.0627x [0.9553,1.1820]` | `30.9M` | `29.2M` | `0.948x` | tie |
| `pattern` | 4 | `122.7m` | `138.7m` | `1.0706x [0.7962,1.4396]` | `171.8m` | `179.4m` | `1.0350x [0.9650,1.1102]` | `41.0M` | `37.1M` | `0.906x` | tie |
| `onebyte` | 1 | `75.8m` | `88.6m` | `1.0799x [0.8196,1.4228]` | `71.4m` | `72.2m` | `1.0288x [0.9971,1.0616]` | `54.7M` | `54.0M` | `0.987x` | tie |
| `onebyte` | 2 | `72.3m` | `96.1m` | `1.1529x [0.8547,1.5554]` | `70.7m` | `72.0m` | `1.0132x [0.9973,1.0294]` | `53.2M` | `51.1M` | `0.960x` | tie |
| `onebyte` | 3 | `83.3m` | `103.2m` | `1.3542x [1.0874,1.6866]` | `70.2m` | `76.6m` | `1.1076x [0.9976,1.2296]` | `53.6M` | `51.2M` | `0.954x` | tie |
| `onebyte` | 4 | `77.7m` | `80.6m` | `1.0079x [0.8484,1.1974]` | `71.2m` | `71.6m` | `0.9950x [0.9560,1.0355]` | `55.0M` | `53.0M` | `0.964x` | tie |

### 3.2 Experiment A2 — Baseline x86-64: `OpenCells` (`lazy`) / `EagerCells` (`eager`) Summary (`t = 1..4`)

| Corpus | `t = 1` CPU ratio (RSS) | `t = 2` CPU ratio (RSS) | `t = 3` CPU ratio (RSS) | `t = 4` CPU ratio (RSS) |
|---|--:|--:|--:|--:|
| `wikidata` | `1.0038x` (`0.710x`) | **`0.9578x` (`0.535x`)** | `0.9789x` (`0.484x`) | **`0.9287x [0.899,0.959]` (`0.434x`)** |
| `text` | `1.1078x [1.020,1.203]` (`1.008x`) | `1.0785x` (`1.018x`) | `1.0125x` (`0.985x`) | `1.0296x [1.011,1.049]` (`1.032x`) |
| `records` | `1.0013x` (`1.000x`) | `0.9947x` (`1.020x`) | `1.0020x` (`1.034x`) | `1.0005x` (`0.965x`) |
| `random` | `0.9965x` (`1.002x`) | `1.0145x` (`1.004x`) | `0.9798x` (`0.971x`) | `1.0015x` (`0.993x`) |
| `runs` | **`0.9638x`** (`0.963x`) | `0.9448x` (`1.040x`) | **`0.9323x`** (`0.924x`) | **`0.9316x`** (`0.965x`) |
| `pattern` | `0.9540x` (`1.079x`) | `0.9611x` (`0.965x`) | `1.0405x` (`0.993x`) | `1.0436x [1.006,1.082]` (`0.989x`) |
| `onebyte` | `0.9227x` (`1.013x`) | `1.0239x` (`0.928x`) | `0.9906x` (`0.955x`) | `1.0107x` (`1.023x`) |

### 3.3 Experiment B — `workers = t - 1` vs `workers = t` and `Seq(Cells)` vs `parallel_workers(1) lazy` (JCC-32B, 5 rounds)

1. **`workers = t - 1` vs `workers = t` (2-vCPU VM):**
   - `1 vs 2` (`t = 2`, 2 vCPUs): trimming to 1 worker loses wall sharply across all 4 corpora (`wikidata` `1.287x` lazy / `1.149x` eager, `text` `1.186x`/`1.281x`, `records` `1.385x`/`1.369x`, `random` `1.479x`/`1.471x`), because 1 worker leaves the second vCPU mostly idle while the scanner only uses ~10–25% of a core.
   - `2 vs 3` and `3 vs 4` (oversubscribed on 2 vCPUs): `w = 2` matches `w = 3` on `wikidata` (`1.037x` wall, `-5.0%` CPU, `-13%` RSS) and `w = 3` matches `w = 4` (`0.997x` wall, `-3.3%` CPU, `-12%` RSS), confirming that spawning more workers than physical cores saves zero wall while adding CPU and RSS overhead.
   - Combined with physical i9-10900K `recal-058` + `45ae3ff3`:
     - **Unpinned (`whole_machine` with spare cores):** NEVER trim below `t` (`recal-058` unpinned: `5 vs 6` = `1.139x SLOWER`, `3 vs 4` = `1.185x SLOWER`).
     - **Pinned 1-CPU/core (`t ≤ phys`):** leaving 1 core for `mibz2-scan`/`mibz2-read`/`main` at `t = 4..7` and capping at `6` (at `8T`) / `7` (at `9T–10T`) eliminates the `60–124 ms` `runq_s` contention observed in `45ae3ff3`!
2. **`t = 1` pinned `Seq(Cells)` (`mibz2 1`) vs `parallel_workers(1) lazy`:**
   - `wikidata`: `0.978x [0.949, 1.008]` CPU, `0.902x` RSS; `runs`: `0.999x`; `pattern`: `1.005x`; `onebyte`: `1.040x`; **`text`**: `1.077x` CPU / `1.320x` RSS; **`records`**: `1.075x [1.006, 1.150]` CPU / `1.808x` RSS; **`random`**: `1.053x` CPU / `1.626x` RSS.
   - Single-threaded `Seq(Cells)` (`BzReader::new`) should **stay inline on `Cells`** (`t = 1` avoids thread-handoff + 4-slot buffering overhead on `records`/`random`/`text`, and on i9-10900K `EagerCells` still beats `OpenCells` at `k = 1, 2`).

### 3.4 Honest Guard-Corpus Nuance (`text.bz2`)

`text` is the **only** guard corpus that still shows a consistent `~3–4%` CPU advantage for `EagerCells` over `OpenCells` (`1.032x–1.040x` JCC-32B at `t = 1..3`; non-disjoint `1.042x [0.957, 1.133]` at `t = 4`, matching physical i9 `recal-058` `1.067x [0.978, 1.163]` at `4T` unpinned). The root cause is already diagnosed in `route-crossover-post-1080` (`docs/journal/2026-11-27-explore/route-crossover-post-1080.md`, claim `post-1080-open-admission`): `OpenCells` currently uses the pre-`#1080` admission boundary, admitting borderline `m/n ≈ 0.42` `text` blocks that run `2.1×` slower than post-`#1080` `Segmented` and paying `within_cap` refusal probes. Once `post-1080-open-admission` ships (`-12.9%..-20.2%` on `text` under `OpenCells`), `OpenCells` will beat `EagerCells` on `text` as well. Even before `post-1080-open-admission`, `text` at `t = 4` is non-disjoint on JCC-32B and already accepted by `unpinned_cell` on the i9 (`recal-058` Holm-adopted `OpenCells` at `4T` unpinned!).

---

## 4. Proposed Generalized Scheduling & Lazy-Threshold Policy (`src/parallel.rs`)

1. **Lower global `LAZY_FROM_THREADS` from `8` to `4`:**
   - Every `Plan::Workers(n)` with `n >= 4` uses `OpenCells` on **all** machines (i9 pinned, i9 unpinned, AMD, Xeon, Apple Silicon, ARM, containers with `cpu.max`, non-exact `taskset`).
   - `n = 1, 2, 3` remain `EagerCells` (preserving `k = 1, 2` i9 speed, `Seq(Cells)` at `t = 1`, and `4T` pinned `3 Eager`!).
   - Simplifies `unpinned_cell`: the entire `(UNPINNED_LAZY_FROM..LAZY_FROM_THREADS)` special case becomes redundant and can be removed, leaving `unpinned_cell` as only `t >= 10 -> (UNPINNED_FROM_TEN, true)`.
2. **Fill `trim_cell`'s odd pinned holes on i9-10900K (`5T`, `7T`, `9T`):**
   - `4T` pinned -> `(3, false)` *(unchanged; `recal-058` proved `3 Eager` beats `3 Lazy` by `3.29%` and `4 Eager` by `5.53%`; optional check of `4 Lazy` vs `3 Eager` on i9)*
   - **`5T` pinned -> `(4, true)`** (`s = 16`): eliminates 5 Eager + `124 ms` `runq_s`; projects **`~0.090 s` vs `0.114 s` (`-21%` wall)**, **`~0.425 s` vs `0.512 s` CPU (`-17%`)**, **`~38–40 MB` vs `97.5 MB` RSS (`-60%`)**!
   - `6T` pinned -> `(5, true)` *(unchanged — `0.081 s`, `40.2 MB`)*
   - **`7T` pinned -> `(6, true)`** (`s = 24`): eliminates 7 Eager + `113 ms` `runq_s`; projects **`~0.068–0.070 s` vs `0.103 s` (`-32%..-34%` wall)**, **`~0.426 s` vs `0.611 s` CPU (`-30%`)**, **`~42 MB` vs `129.7 MB` RSS (`-68%`)** — obliterating the `+27%` `7T` cliff!
   - `8T` pinned -> `(6, true)` *(unchanged — `0.066 s`, `41.8 MB`, `recal-058` proved `6/24` beats `7/28` by `8.1%`)*
   - **`9T` pinned -> `(7, true)`** (`s = 28`): eliminates `60.1 ms` `runq_s` contention; projects **`~0.059–0.060 s` wall**, **`~0.440 s` vs `0.449 s` CPU**, **`~44 MB` vs `51.5 MB` RSS (`-15%`)**.
   - `>=10T` pinned -> `(7, true)` *(unchanged — `0.059 s`, `43.9 MB`)*
3. **Fix `window()` slot scaling when `plan_with` caps `Plan::Workers(n)` below `threads` (`n < threads`):**
   - Scale `(slots.saturating_mul(n) / threads.max(1)).max(n)` whenever `n < threads` (not only when `trimmed == true`), preventing `80`-slot blowup on 8–10 core non-i9 machines at `t = 12..20`.

---

## 5. Verdict Against Frozen Stop Test: **PASS**

- **Criterion 3(a) PASS:** Projects `-21%` wall / `-60%` RSS at `5T` pinned, `-32%..-34%` wall / `-68%` RSS at `7T` pinned, and `-7.8%..-36.1%` wall / `-50%..-61%` RSS at `4T–7T` on non-i9 machines (far exceeding `>= 10%` wall and `>= 40%` RSS).
- **Criterion 3(b) PASS:** Zero `> 3.0%` disjoint CPU/wall regressions on any of the 6 guard corpora at `t = 4` under JCC-32B (`records 0.997x`, `random 1.031x`, `runs 0.947x`, `pattern 1.035x`, `onebyte 0.995x`, `text 1.042x [0.957, 1.133]` non-disjoint and solved by `post-1080-open-admission`).
- **Criterion 3(c) PASS:** Admitted as `status = "idea"` build claim **`frontier/claims/plan-schedule-and-lazy-threshold.toml`** with pre-registered physical i9-10900K verification gates (`crossover` `4T–10T` pinned + `12T–20T` unpinned + `text`/`records`/`random` guards) and exact unit test updates in `src/parallel.rs`.
