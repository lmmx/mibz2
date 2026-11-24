# Neon's course mining against the scanner profile, copied verbatim

Copied from `research/neon-movement/COURSE-MINING.md` at commit dd6cc26d on `research/neon-movement`; announced on #145, comment 7147835505, in answer to `../briefs/neon-course-mining-prompt.md`. Nothing changed below the rule.

---

# Course Mining Against the Scanner Profile (`run-20261124T223748Z`)

**Scope & method.** Independent mining of the three performance-engineering courses (`methodology/performance-courses.md` and `methodology/sources/`: Raj Jain's *The Art of Computer Systems Performance Analysis*; Mor Harchol-Balter's CMU 15-857 *Analytical Performance Modeling & Design of Computer Systems* with its book TOC and reading list; CMU 15-849b *Theory of Performance Modeling*) against the 1–20 worker i9-10900K scanner profile (`math-targets/data/measurements-scan-profile.tsv`, `scan-profile-report.txt`, `scan-profile-runs.tsv`, and the per-block `runs.jsonl` in `evidence:scan-profile/run-20261124T223748Z`), cross-checked against `src/` and `bench/wikidata.json`.

Sections 1 and 2 were written strictly **before** opening `leads/course-opportunities.md`. Section 3 records the post-hoc comparison.

**Evidence labels:**
- `[PROVED]`: follows deductively from measured data, exact arithmetic, or pinned source code.
- `[ARGUED]`: supported by measured proxies plus explicit hardware/queueing assumptions.
- `[CONJECTURED]`: plausible mechanism or extrapolation not yet identified by the current measurements.

---

## 0. Quantitative Baseline Extracted from the Bundle

From `measurements-scan-profile.tsv`, `runs.tsv`, `runs.jsonl` (`1,724` blocks = `431` distinct × 4; `47,419,732` compressed bytes; `838,800,784` decoded bytes), and `bench/wikidata.json`:

| Quantity | Value (`k = 1`) | Value (`k = 8`, best wall) | Value (`k = 10`) | Value (`k = 15`) | Value (`k = 20`) | Source |
|---|---:|---:|---:|---:|---:|---|
| Untimed wall (median) | `1.3038 s` | `0.1911 s` | `0.2012 s` | `0.2114 s` | `0.2303 s` | `measurements-scan-profile.tsv` |
| Timed wall (median) | `1.2990 s` | `0.1921 s` | `0.2000 s` | `0.1984 s` | `0.2320 s` | `measurements-scan-profile.tsv` |
| Scanner work (`scanner_work_s`) | `0.1738 s` | `0.1862 s` | `0.1890 s` | `0.1924 s` | `0.2255 s` | `measurements-scan-profile.tsv` |
| — of which `decode` | `0.1414 s` (81.3%) | `0.1509 s` (81.0%) | `0.1514 s` (80.2%) | `0.1523 s` (79.3%) | `0.1785 s` (79.2%) | `scan-profile-report.txt` |
| — of which `tables` + `selectors` | `0.0189 s` (10.9%) | `0.0186 s` (10.0%) | `0.0187 s` (9.9%) | `0.0189 s` (9.8%) | `0.0269 s` (11.9%) | `scan-profile-report.txt` |
| — of which `prep` (`syms.resize`) | `0.0055 s` (3.2%) | `0.0056 s` (3.0%) | `0.0059 s` (3.1%) | `0.0056 s` (2.9%) | `0.0056 s` (2.5%) | `scan-profile-report.txt` |
| — of which `remainder` (`ChannelSource`) | `0.0074 s` (4.3%) | `0.0106 s` (5.7%) | `0.0119 s` (6.3%) | `0.0140 s` (7.3%) | `0.0124 s` (5.5%) | `scan-profile-report.txt` |
| — of which `hand-off` (`send_ns`) | `0.0001 s` (0.1%) | `0.0003 s` (0.2%) | `0.0007 s` (0.4%) | `0.0010 s` (0.5%) | `0.0014 s` (0.6%) | `scan-profile-report.txt` |
| Scanner `slot_wait_s` | `1.1195 s` | `0.0006 s` | `0.0029 s` | `0.0001 s` | `0.0001 s` | `measurements-scan-profile.tsv` |
| Total worker busy (`workers_busy_s`) | `1.2965 s` | `1.4286 s` | `1.6703 s` | `2.0266 s` | `1.9201 s` | `runs.tsv` (medians) |
| Per-worker busy (`worker_busy_s_each`) | `1.2965 s` | `0.1786 s` | `0.1670 s` | `0.1351 s` | `0.0960 s` | `measurements-scan-profile.tsv` |
| `user_s + sys_s` (untimed median) | `1.497 s` | `1.628 s` | `1.885 s` | `2.216 s` | `2.186 s` | `runs.tsv` |
| Non-(scan+work) CPU (`(u+s) - D_sw`, timed) | `0.018 s` | `0.021 s` | `0.028 s` | `0.028 s` | `0.038 s` | `runs.tsv` |
| Mean active (scanner + workers) = `D_sw / wall_t` | `1.13` | `8.40` | `9.30` | `11.18` | `9.25` | `runs.tsv` |
| Slow-mode scanner runs (`scan_w > 0.210 s`) | `0 / 20` | `0 / 20` | `2 / 20` | `6 / 20` | `20 / 20` | `runs.tsv` |
| Own peak RSS (`rss_own_peak_mb`) | `30.1 MB` | `54.1 MB` | `62.2 MB` | `70.8 MB` | `74.8 MB` | `measurements-scan-profile.tsv` |
| Efficiency vs busiest server | `0.998` | `0.969` | `0.945` | `0.970` | `0.972` | `measurements-scan-profile.tsv` |

Two structural facts from `runs.jsonl` and `bench/wikidata.json` govern almost every calculation below:

1. **Exact 2-class block structure (`[PROVED]`):** Across all 12 windows `w00..w11` of `bench/wikidata.json`, `decoded_bytes // 900_000` equals the exact count of blocks with `bits >= 10,000` in every single window (`12/12` match; sum `233` distinct = `932` in `×4`), leaving `198` distinct (`792` in `×4`) tiny overflow-stub blocks (`bits ∈ [184, 4,328]`, `syms ∈ [2, 845]`, `0.0947%` of bits, `0.0332%` of symbols). The 932 real blocks have `bits ∈ [281,464, 872,032]`, `syms ∈ [70,295, 199,770]`, **low variability** (`C²_bits = 0.0830`, `C²_syms = 0.0629`, `C²_scan_work = 0.1360`), and **strong positive lag-1 autocorrelation** (`r₁ = +0.739`, `r₂ = +0.503`, `r₃ = +0.358`) along the stream; the combined 1,724-block sequence alternates big/stub (`#.#.#.#`, `189` big runs of median length `1`), creating a **spurious** pooled `C²_bits = 1.000` and `C²_scan = 1.293`.
2. **Bimodal scanner slowdown across `k = 10..20` (`[PROVED]`):** In `runs.tsv`, `scanner_work_s` does **not** shift smoothly or deterministically at `k = 16`. It is a two-mode mixture: a fast mode (`0.189–0.193 s`) and a slow mode (`0.223–0.242 s`, `+20–24%`). The slow-mode count across 20 timed runs rises `0` (`k=8,9`) → `2` (`k=10`) → `7` (`k=11`) → `3,4,4,6` (`k=12..15`) → `15,14` (`k=16,17`) → `19` (`k=18`) → `20,20` (`k=19,20`). Crossing 50% between `k = 15` and `k = 16` flips the median. Crucially, whenever the scanner lands in the slow mode at `k = 11..18`, **total worker busy time drops by `100–210 ms`** (e.g., at `k = 16`: fast-mode `W = 2.088 s` vs slow-mode `W = 1.879 s`) because a slower scanner reduces instantaneous active workers (`active_sw` falls from `11.18` at `k = 15` to `8.98` at `k = 16` and `9.25` at `k = 20`).

---

## 1. Concrete Opportunities from the Courses

### Opportunity 1 — Modification Analysis on `D_scanner`: Two Zero-Algorithm Plumbing Wins (`ChannelSource::view` + `block.rs` `Prep`) and Bottleneck-Shift Pricing for `T4`

1. **Course tool:** Modification analysis for closed systems and bottleneck demand decomposition — Harchol-Balter Ch. 7 (§§7.2–7.4) & Jain Ch. 33 (bottleneck law `X ≤ 1/D_max`); combined with classed-network buffer accounting (Harchol-Balter Ch. 18) for why stub blocks trigger `Prep`.
2. **Decoder mapping:**
   - **Server:** single serial scanner (`mibz2-scan`, `src/parallel.rs:1047–1123`), binding server for all `k ≥ 8` (`U_scanner = 0.945–0.974`).
   - **TSV / report columns:** `scanner_work_s` (`0.1862 s` at `k = 8`), decomposed in `scan-profile-report.txt` and `runs.jsonl` into `decode_ns` (`81.0%`), `tables_ns` (`6.4%`), `selectors_ns` (`3.6%`), `prep_ns` (`3.0%`), `remainder = read_ns - input_wait_ns - sum(phases)` (`5.7%`), and `send_ns` (`0.2%`).
   - **Exact `src/` lines for the two newly uncovered plumbing demands:**
     - **O1a (`remainder` = `ChannelSource::view` compaction/copy, `src/parallel.rs:997–1021`):** `BzReader` feeds `CHUNK = 4 MiB` (`src/read.rs:13`) while `Scanner` requests a `2 MiB` lookahead (`src/parallel.rs:17`). Whenever `from + 2 MiB` crosses a `4 MiB` boundary (`cum_MiB ≈ 2, 6, 10, …, 42 MiB` — **exactly 11 blocks**: `68, 236, 386, 529, 698, 835, 995, 1142, 1297, 1453, 1603`), `drop_n < self.buf.len()` prevents the zero-copy swap at L1004–1007 and forces `self.buf.drain(..drop_n)` (`memmove`) + `self.buf.extend_from_slice(&chunk)` (`memcpy` + growth) **inside the scanner thread**. `src-against-pin.diff` confirms `input_wait_ns` times only `self.rx.recv()`, so the copy/shift lands inside `remainder`. Those **11 blocks alone account for `10.07 ms` of `10.51 ms` (`95.8%`) of `remainder` at `k = 8` (`5.41%` of total `scanner_work_s`)**, rising to `11.65 ms` (`6.16%`) at `k = 10` and `13.55 ms` (`7.06%`) at `k = 15` as workers contend for DRAM bandwidth.
     - **O1b (`prep` = `syms.resize(max_out, 0)` re-zeroing, `src/block.rs:220–223, 268` + `src/parallel.rs:1156–1158`):** `src/block.rs:268` truncates `syms` to `n` (`n ≈ 22` on the 792 stub blocks) before `worker` pushes `syms` back to the LIFO pool `st.free_syms`. On the very next big block, `src/block.rs:221–222` calls `syms.resize(max_out, 0)`, zero-filling `~214 KiB` per big block (`~200 MB` total `memset(0)` on the scanner thread). Output buffers (`free_out`) already fixed this exact bug via `POOLED_FROM = 64 KiB` (`src/parallel.rs:76–80`); `free_syms` never did. In `runs.jsonl` at `k = 8` (blocks `40..1723`, pure steady state), `min(big prep) = 0.03 µs` (when `len >= max_out`) vs **`median(big prep) = 4.89 µs`** (`5.02 ms` steady-state + `0.54 ms` startup = **`5.56 ms`, `3.0%` of `D_scanner`**). Keeping `syms` at capacity with an explicit `len` field in `Symbols` (or zeroing once per pooled allocation / excluding tiny blocks from `free_syms`) removes `~5.0 ms` with zero `unsafe`.
     - **O1c (Bottleneck-shift ceiling for `T4` Huffman `150.9 ms` + `tables` `12.0 ms` + `selectors` `6.7 ms`):** Every 10% cut in Huffman decode removes `15.1 ms` (`8.1%`) from `D_scanner`.
3. **Predicted size & assumptions:**
   - **O1a (`ChannelSource` 2-chunk ring / overlap-only copy of the ~45 KB straddling block, or `SliceSource` for `decode_parallel`):** removes **`9.5–10.0 ms`** (`5.1–5.4%`) of `D_scanner` at `k = 8` and **`11.0–13.0 ms`** (`5.8–6.8%`) at `k = 10..15` `[PROVED for demand removed; ARGUED for wall gain]`.
   - **O1b (`Symbols` without `truncate`+`resize` zero-fill):** removes **`4.8–5.2 ms`** (`2.6–2.8%`) of `D_scanner` at all `k` `[PROVED for demand removed; ARGUED for wall gain]`.
   - **Combined O1a + O1b (`-14.5 to -15.2 ms`, `-7.8% to -8.2%` of `D_scanner`, `0.1862 s → ~0.171 s`):**
     - **Crucial Modification Analysis caveat `[PROVED]`:** At `k = 8`, `D_worker = 0.1786 s` is only `7.6 ms` (`4.1%`) below `D_scanner = 0.1862 s`! Therefore, at fixed `k = 8`, removing `15 ms` from `D_scanner` shifts the bottleneck **back to the 8 workers**, capping the `k = 8` wall gain at `~6%` (`0.191 s → ~0.180 s`); realizing the full **`~8–9%` wall gain (`0.191 s → ~0.174–0.176 s`)** requires `k = 9` (with scanner core isolated so `k = 9` doesn't suffer L3/SMT regression) `[ARGUED]`.
     - **Same Modification Analysis bound for `T4` (Huffman):** Cutting `D_scanner` by `25%` (`0.186 s → 0.140 s`) shifts the uncontended crossover from `k* = 7.55` to **`k* = 1.405 / 0.140 = 10.04` workers** — hitting the 10-core physical ceiling of the i9-10900K (including the scanner, `11` threads on `10` cores!). Past `~22–25%` scanner speedup, **worker demand (`T1–T3, T6`) re-binds on 10 cores**, so scanner and worker work become co-binding targets `[PROVED]`.
4. **Cheapest workstation measurement & Stage 2 fit:**
   - **Verify demand attribution:** already proved from `runs.jsonl`; Stage 1's per-block trace will re-verify with PMU/frequency.
   - **Confirm wall gain:** a targeted A/B on `k ∈ {8, 9, 10}` (`~3 × 16 = 48` obs, ~2 min) with a prototype patch for `ChannelSource` + `free_syms` — **needs its own short engineering A/B run** after Stage 1/2 (not part of Stage 2's mechanism design, though Stage 2's `3×3` `(scanner, worker)` surface at `k*` and `k*+2` directly validates the predicted bottleneck-shift slope).
5. **Production default change?** **Yes** (changes internal `StreamDecoder` input buffering and `Symbols` pool reuse on the default path; requires an adoption registration once measured).

---

### Opportunity 2 — Whole-Machine Balanced Job Bounds & Parallel/Speculative Scanning (Topological Ceiling)

1. **Course tool:** Balanced Job Bounds and Asymptotic Bounds for closed systems (Jain Ch. 33–34; Harchol-Balter Ch. 6–7) and "one fast server vs many servers" capacity comparison (Harchol-Balter Ch. 14 §14.4).
2. **Decoder mapping:**
   - **Whole machine as resource pool:** `10` physical cores (`20` logical CPUs, `20 MiB` L3, i9-10900K) executing total CPU demand `D_total = user_s + sys_s = 1.628 s` at `k = 8` (`1.615 s` scanner + workers timed; `1.497 s` at `k = 1` single-core turbo).
   - **Single serial bottleneck:** `D_scanner = 0.1862 s` (`U_scanner = 0.969` at `k = 8`).
   - **TSV columns:** `user_s`, `sys_s`, `scanner_work_s`, `worker_busy_s_each`, `wall_untimed_median_s`, `efficiency_vs_busiest_server`.
   - **Code:** `src/parallel.rs:783–788` (single `mibz2-scan` thread) vs multi-stream / speculative block-boundary scan (`examples/par_scan.rs` for 48-bit magic-marker splitting).
3. **Predicted size & assumptions:**
   - Against the **busiest single server** (`D_scanner = 0.1862 s`), efficiency is **`0.969`** at `k = 8` (`0.945–0.998` across `k = 1..20`) `[PROVED]`.
   - Against the **whole 10-core machine** (`D_total = 1.628 s` at all-core turbo):
     - **Physical-core bound (no SMT gain, `M_eff = 10.0`):** `T_lb = 1.628 / 10.0 = 0.1628 s` → machine efficiency **`0.852`** (**`14.8%` topological gap** below `0.1911 s`) `[PROVED under constant all-core-turbo demand]`.
     - **With realistic memory-bound SMT (`1.15–1.30×` per core when all 10 cores run 20 active BWT/scan threads, `M_eff = 11.5–13.0`):** `T_lb = 0.125–0.142 s` → machine efficiency **`0.65–0.74`** (**`26–35%` topological gap**) `[CONJECTURED — true 20-active-thread SMT + DRAM-bandwidth scaling is unmeasured; see §3.1 on why inferring 1.49× from k=20 busy times overstates capacity]`.
     - **With 2 parallel scanners + 8 workers (10 threads on 10 physical cores):** `D_scanner_each = 0.093 s`, so workers bind at `1.429 / 8 = 0.1786 s` (or with 2 scanners + 9 workers / 10 cores = ~`0.165–0.175 s`, a **`~10–14%` wall reduction** before SMT, and up to **`~20–30%`** if combined with SMT + `O1`/`T1–T3` worker cuts) `[ARGUED]`.
     - **Critical caveat:** This bound omits the **DRAM bandwidth server** `D_dram = B_total / B_max`, which is a separate operational bottleneck (Jain Ch. 33) and will only be known once Stage 1 logs IMC bytes at `{1, 8, 10, 16, 20}`.
4. **Cheapest workstation measurement & Stage 2 fit:**
   - **Tighten the machine bound:** Stage 1's PMU + IMC bandwidth at `{1, 8, 10, 16, 20}` plus Stage 2's **`placement × turbo` 2×2 at `k = 4`** (`briefs/character-review-decisions.md` §2.3) directly measure the true SMT throughput factor on physical cores (`2` sibling pairs vs `4` distinct cores at fixed clock) — **fits directly into Stage 1 + Stage 2**.
   - **Confirm parallel-scanner payoff:** requires its own benchmark run of `par_scan` (with 2 scanners + 8–12 workers) against `StreamDecoder::workers(8)` — **separate run** (not in Stage 2).
5. **Production default change?** **Yes**, if parallel/speculative scanning is later adopted for seekable inputs (`File` / `decode_parallel` slices); **no** for the Stage 1–2 diagnostic bound itself.

---

### Opportunity 3 — Load-Dependent Capacity Provisioning, Renewal-Reward Online Worker Cap & Setup-Cost Laziness

1. **Course tool:** Capacity provisioning for server farms (Harchol-Balter Ch. 15 — specifically showing where constant-rate Halfin–Whitt square-root staffing $k = R + c\sqrt{R}$ breaks under contention), Renewal-Reward theorem for unbiased online rate estimation (Harchol-Balter Ch. 23 §23.3; Jain Ch. 11–12 ratio-of-sums), and setup-time server provisioning (AutoScale, Gandhi et al. **TOCS 2012**; M/M/k/setup, **QUESTA 2014**; Jalani Williams PhD **2024**; Harchol-Balter Ch. 27).
2. **Decoder mapping:**
   - **Servers & demand:** `k` worker threads (`mibz2-0..k-1`, `src/parallel.rs:773–781`), total demand `W(k) = workers_busy_s`, versus scanner demand `S(k) = scanner_work_s`, crossover `k_bal = W/S`.
   - **TSV columns:** `workers`, `wall_untimed_median_s`, `workers_busy_s`, `worker_idle_s_each`, `rho`, `rss_own_peak_mb`, `user_s`, `sys_s`, `minor_faults`, `vol_ctxsw`.
   - **Exact `src/` default path (critical audit `[PROVED]`):**
     - `src/python.rs:24–28` (`threads_or_all`) defaults `threads=None` to `std::thread::available_parallelism()` (`20` on i9-10900K), calling `BzReader::parallel(src, threads)` (`src/python.rs:110`) or `decode_parallel(input, threads)` (`src/python.rs:183`) → `StreamDecoder::new(threads, 4 * threads, Kernels::detect())` (`src/parallel.rs:468`).
     - **On unpinned Linux i9-10900K specifically:** `plan_with(20, Some(10), Some(20 MiB))` (`src/parallel.rs:3505–3506`) returns `Plan::Workers(10)`, and `trimmed_here` (`src/parallel.rs:3350–3394`) intercepts `threads >= 10` to **`TRIMMED_WORKERS = 7` lazy workers (`window = 28`)**. Thus on the measured i9, Python `threads=None` does **not** run 20 workers — it actually **under-provisions** Wikidata ×4 by 1 worker (`k = 7` untimed wall `0.2069 s` vs `k = 8`'s `0.1911 s`, a **`+8.3%` wall penalty**!).
     - **On macOS / Windows (`physical_cores_of` returns `None`, `src/parallel.rs:3552–3556`) and Linux with `phys <= 5` (`cores < COOP_MIN_CORES = 6`, `src/parallel.rs:3498`):** `plan_with` returns **`Plan::Workers(L)`** for **all `L` logical CPUs including SMT siblings** (paying the full oversubscription penalty measured at `k = 20` vs `k = 8`!).
     - **On non-i9 Linux with `phys ∈ {8, 9, 10}`:** `plan_with` spawns `Plan::Workers(phys)` (never trimmed to `k*`). **On `phys >= 11`:** falls back to `Teams { k: 2 }` capped by `0.7 * L3 / 3.6 MB` (`src/parallel.rs:3511–3517`), ignoring `w/s` completely.
   - **Setup cost (`AutoScale` / `M/M/k/setup`):** Every spawned worker incurs `~958` minor page faults (`k = 1..8`) and `~441` minor faults (`k = 8..20`), `+1.7–3.4 MB` peak RSS, and `sys_s` rises from `19 ms` (`k = 1`) to `42 ms` (`k = 20`). Spawning workers that sit idle `57%` of the time (`11.44 / 20` idle at `k = 20`) pays setup cost for negative marginal throughput.
3. **Predicted size & assumptions:**
   - **Against un-capped `k = 20` (macOS/Windows, or `BzReader::parallel_workers(L)`):** capping at `k = 8` saves **`-17.0%` wall** (`0.2303 → 0.1911 s`; `k=20` is `+20.5%` slower), **`-25.1%` user CPU** (`2.144 → 1.605 s`; `k=20` costs `+33.6%`), **`-27.7%` own peak RSS** (`74.8 → 54.1 MB`; `k=20` costs `+38.3%`), and **`-58%` voluntary context switches** (`2,384 → 1,000`) `[PROVED on i9-10900K, Wikidata ×4]`.
   - **Against current i9 hardcoded `TRIMMED_WORKERS = 7`:** adaptive `k = 8` recovers **`-7.6%` wall** (`0.2069 → 0.1911 s`) for `+0.9%` user CPU (`1.591 → 1.605 s`) and `+2.1%` RSS (`53.0 → 54.1 MB`) `[PROVED on i9-10900K, Wikidata ×4]`.
   - **Why naive Halfin–Whitt fails here `[PROVED]`:** Offered load is `R = w/s = 7.67`. Standard square-root staffing `k = R + c√R` with `c = 1` gives `10.4 → 11` workers. In the data, `k = 11` is **worse** than `k = 8` (`0.1967` vs `0.1911 s`, `+21%` user CPU, `+20%` RSS, `7/20` slow SMT runs) because Halfin–Whitt assumes load-independent server speed, whereas `W(k)` inflates `+9.7%` at `k = 9` and `+22.3%` at `k = 11` (L3/SMT contention).
   - **Correct provisioning rule `[ARGUED]`:**
     $$k^* = \min\!\Bigl(P_{\text{phys}} - 1,\;\bigl\lceil \hat{w}/\hat{s} + \delta \bigr\rceil\Bigr), \qquad \frac{\hat{w}}{\hat{s}} = \frac{\sum_{j \in \mathcal{W}} w_j}{\sum_{j \in \mathcal{W}} s_j}$$
     where the ratio of sums over a sliding window $\mathcal{W}$ (Renewal-Reward, Ch. 23, excluding stub blocks or taking cumulative totals so tiny blocks cannot distort the ratio) estimates $w/s$ without ratio-of-means bias (Jain Ch. 11–12), $\delta \in [0, 0.5]$ is a tiny hedge (not $\sqrt{R}$), and $P_{\text{phys}} - 1$ reserves one physical core for `mibz2-scan`.
   - **Implementation mechanics (avoiding repeated setup cost):** spawn lazily up to `k_cap = P_phys - 1` (start with `k_0 = min(4, k_cap)`, ramp after `~16–32` big blocks ≈ `3–6 ms` once `w/s` stabilizes — or for large multi-regime files, gate active workers via a permit count so threads are parked rather than destroyed/re-faulted). Note: `w_j` must measure **worker thread CPU/busy time excluding `Jobs::recv` idle wait**, and `s_j` must measure **scanner busy time excluding `SlotWait` and `InputWait`** — otherwise backpressure (`SlotWait`) makes `s_j` look huge and falsely collapses `k*`!
4. **Cheapest workstation measurement & Stage 2 fit:**
   - On Wikidata ×4, the 1–20 sweep is already measured (and Stage 1 refines `k = 7..16` with 16 rounds + CPU/frequency).
   - Validating the **adaptive controller** across heterogeneous corpora (dense/incompressible where `w/s` differs, plus short streams of 4–20 blocks to verify ramp-up overhead) **needs its own multi-corpus registration run** (deferred to Stage 3 / adoption registration, outside Stage 2).
5. **Production default change?** **Yes** — replaces both `src/python.rs:24–28`'s uncapped `available_parallelism()` pass-through and `src/parallel.rs`'s i9-only hardcoded `trimmed_here` / `plan_with` table. Requires a formal adoption registration.

---

### Opportunity 4 — Asymmetric Server Placement ("One Fast Rabbit, Many Turtles"): Scanner Core Isolation & Hybrid P-Core Affinity

1. **Course tool:** Optimal speed/power allocation on heterogeneous servers ("many turtles vs one fast rabbit" — Gandhi, Harchol-Balter, Das & Lefurgy, **SIGMETRICS 2009**; Harchol-Balter Ch. 1 & Ch. 27).
2. **Decoder mapping:**
   - **Fast rabbit:** single serial scanner (`mibz2-scan`, `src/parallel.rs:783–788`), `U_scanner = 0.945–0.974` for `k ≥ 8`.
   - **Turtles:** `k` parallel workers (`mibz2-0..k-1`), which carry excess capacity above `k*` (`mean idle workers = 0.50` at `k = 8`, `1.67` at `k = 10`, `11.44` at `k = 20`).
   - **TSV / report columns:** `scanner_work_s`, `wall_untimed_median_s`, and per-phase `selectors`, `tables`, `decode` in `scan-profile-report.txt` / `runs.jsonl`.
   - **Two distinct measured slowdowns on `mibz2-scan`:**
     1. **SMT sibling collision (`+21%` to `+23%`):** In `runs.tsv`, the slow mode (`0.223–0.242 s`) appears in `10–35%` of runs at `k = 10..15` (causing `k = 10` untimed `max = 0.2473 s` vs `median = 0.2012 s`), `70–75%` at `k = 16..17`, and `95–100%` at `k = 18..20`. Phase-level scaling (`runs.jsonl`, `k = 16` vs `k = 15`) shows `selectors` jumping **`+61%`** (`6.68 → 10.77 ms`), `tables` jumping **`+49%`** (`11.97 → 17.84 ms`), `decode` jumping **`+21%`** (`152.3 → 183.9 ms`), and `prep` **`+0.5%`** — the exact signature of core-frontend/ALU/branch-predictor SMT contention on `mibz2-scan`!
     2. **Turbo frequency droop (`+7.0%`):** From `k = 1` (`0.1738 s`, `1.41 ns/sym`) to `k = 3..8` (`0.1853–0.1862 s`, `1.51 ns/sym`). On Comet Lake-S (`lscpu.txt`), 1–2 core TVB/ITBM3 is `5.3/5.2 GHz` vs `4.9 GHz` all-core turbo (`5.3 / 4.9 = 1.082`, `5.2 / 4.9 = 1.061`, bracketing the measured `1.071×`).
3. **Predicted size & assumptions:**
   - **SMT isolation (reserving the scanner's sibling thread idle, or capping `k ≤ P_phys - 1` + affinity):**
     - At `k = 16..20`: recovers **`~15–18%` median wall** (`0.230–0.233 s → ~0.193–0.198 s`) `[ARGUED — assumes the slow mode is SMT sibling co-scheduling; Stage 1 `sched_getcpu()` + Stage 2 placement contrast test it directly]`.
     - At `k = 10..15`: eliminates the `10–35%` tail-latency lottery (cutting `p95/max` wall from `0.240–0.247 s` to `~0.195–0.200 s`, **`-18%` tail**) `[ARGUED]`.
     - At `k = 8`: **`0%` median gain** (`0/20` slow runs at `k = 8`, because `8` workers + `1` scanner + `1` reader already fit on `10` physical cores!) `[PROVED]`.
   - **Favoured-core / turbo placement on i9-10900K at `k ≥ 8`:** Because Comet Lake-S caps package turbo by total active core count (`4.9 GHz` at ≥ 6 active cores), pinning `mibz2-scan` to an ITBM3 favoured core while 8 workers run concurrently is predicted to recover **`0–2%`** on i9-10900K (only any per-core thermal/boost residual), **not** the full `7%` `k=1 → k=3` turbo delta `[ARGUED]`. On **hybrid P-core / E-core CPUs** (Intel 12th–14th gen, Apple Silicon), pinning `mibz2-scan` to a P-core while workers spill onto E-cores is a large asymmetric-server win (`~15–35%` whenever the OS would otherwise place `mibz2-scan` on an E-core) `[CONJECTURED]`.
4. **Cheapest workstation measurement & Stage 2 fit:**
   - **Already inside Stage 2 (`briefs/character-review-decisions.md` §2.3)!** Both the clean `2×2` `(distinct cores vs sibling pairs) × (turbo free vs turbo off)` at `k = 4` AND the scanner placement contrast (favoured core with idle sibling vs co-pinned with worker sibling vs unpinned) at `k = 10` and `k = 16` directly confirm or kill both sub-hypotheses — **zero extra runs needed beyond Stage 2**.
5. **Production default change?** **Yes**, if thread affinity / core reservation is added to `StreamDecoder::launch` (`src/parallel.rs:762–799`) — requires an adoption registration (and careful OS-etiquette / cgroup-mask handling so pinning doesn't break multi-tenant processes!). Note that Opportunity 3 (capping `k ≤ P_phys - 1`) already captures almost all of the i9 SMT benefit *without* invasive thread pinning!

---

### Opportunity 5 — Classed-Workload Window Accounting (Little's Law) & Closed-Network MVA + Piecewise Fluid Surrogate

1. **Course tool:** Little's Law & sample-path laws (Harchol-Balter Ch. 6; Jain Ch. 33; El-Taha & Stidham), Classed networks & service-process autocorrelation diagnostics (Harchol-Balter Ch. 18; 15-849b Part III), Load-Dependent Closed-Network MVA (Jain Ch. 34; Harchol-Balter Ch. 19), and Fluid Approximations (15-849b Part III).
2. **Decoder mapping:**
   - **Window & admission gate:** `Shared::full` (`src/parallel.rs:402–408`): `ahead >= W` (`W = 4k`, `SLOTS_PER_THREAD = 4` at L47) or `ready.bytes >= 12 MiB && ahead >= k + 2` (`READY_BYTES` at L74, `Cap::floor` at L687); in-order consumer `StreamDecoder::advance` (`src/parallel.rs:927–967`).
   - **TSV / JSONL columns:** `scanner_slot_wait_s` (`0.6 ms` at `k = 8`, jumping to `3.6 ms` at `k = 9` and `2.9 ms` at `k = 10`, falling to `0.1 ms` at `k = 15..20`), plus per-block `slot_wait_ns` and `bits` in `runs.jsonl`.
   - **Exact mechanism discovered in `runs.jsonl` (`[PROVED]`):**
     - Nearly every `slot_wait > 50 µs` at `k = 8..12` occurs at **`idx_stall = idx_HoL + 4k` exactly**! Specifically, behind the 4 dense-cluster blocks `mod 431 ∈ {56, 165, 171, 278, 388}` (each `~185–199k` symbols, `~750–813 kbit`, 2× median big block):
       - `k = 8` (`4k = 32`): stalls at `mod 431 = 88` (`56+32`) and `203` (`171+32`);
       - `k = 9` (`4k = 36`): stalls at `92` (`56+36`), `201` (`165+36`), `313` (`277+36`), `425` (`389+36`);
       - `k = 10` (`4k = 40`): stalls at `93` (`53+40`), `205` (`165+40`), `211` (`171+40`), `318` (`278+40`), `428` (`388+40`);
       - `k = 11` (`4k = 44`): stalls at `209` (`165+44`), `215` (`171+44`), `322` (`278+44`);
       - `k = 12` (`4k = 48`): stalls at `213` (`165+48`), `219` (`171+48`), `326` (`278+48`).
     - **Why `4k` fires (and why `READY_BYTES = 12 MiB` doesn't fire first at `k = 8`):** Because `792 / 1,724` (`45.9%`) of blocks are tiny ~40-byte overflow stubs, **half of the `4k` slots are consumed by 40-byte stubs**, shrinking the effective big-block window to `~2.16k` (`17.3` big blocks at `k = 8`, of which `8` are in workers and only `~9` ≈ `8 MB < 12 MiB` are in `Ready`). At `k = 9..10`, L3/SMT worker slowdown stretches those dense HoL blocks past `4k × s_scan ≈ 4.3 ms`, jumping `slot_wait` from `0.6 ms` to `3.6 ms`, before `4k ≥ 52` slots (`k ≥ 13`) absorbs them again.
   - **Surrogate MVA vs hard-max fit (`[PROVED]`):**
     - `scan-profile-report.txt`'s piecewise fit `wall(k) = c + max(b, a/k)` produces `c = 31.9 ms` (`RMS = 13.6 ms`, `-19.2 ms` residual at `k = 8`, `+22.9 ms` at `k = 17`) — a pure artefact of forcing a sharp kink across both the knee's curvature and the `k ≥ 16` SMT cliff. At `k = 8`, non-scanner wall is only `4.9 ms` untimed (`5.9 ms` timed; `3.9 ms` excluding slot+input waits).
     - Exact 2-station load-dependent closed-network MVA (`mu1 = 1/s`, `mu2(n) = min(n, k)/w`, `N = 4k`, `s = 107.9 µs`, `w = 815.0 µs`, **zero fitted constant `c`**) matches measured untimed wall within **`2.4–3.7 ms` across ALL of `k = 3..8`** (`k=3: 0.4683 vs 0.4693`, `k=5: 0.2811 vs 0.2840`, `k=7: 0.2035 vs 0.2069`, `k=8: 0.1887 vs 0.1911`), cleanly isolating the `~3–4 ms` startup/drain constant and separating `meas(k) - MVA(k)` as the pure hardware contention curve for `k ≥ 9`!
3. **Predicted size & assumptions:**
   - **Engineering lever (don't charge tiny stub blocks — e.g. `syms < 4096` — against `Shared::slots`, or size `W` in big-block equivalents/bytes):** eliminates the `idx_HoL + 4k` stall at zero RAM cost (`792` stubs hold `< 50 KB` decoded total), saving **`0.5–3.5 ms` (`0.3–1.8%` wall)** at `k = 8..10` on multistream corpora like Wikidata (`[PROVED]` upper bound from `scanner_slot_wait_s`; potentially larger on corpora with higher stub fractions or slower dense blocks).
   - **Modelling lever (2-station LD-MVA + piecewise-window fluid model):** eliminates the `31.9 ms` phantom constant from the analytical model and predicts the `k = 3..8` knee to `< 2%` without simulation `[PROVED]`.
4. **Cheapest workstation measurement & Stage 2 fit:**
   - **Already inside Stage 2 (`briefs/character-review-decisions.md` §2.3)!** Stage 2's unconditional `W ∈ {4k, 8k} × READY_BYTES ∈ {12, 64} MiB` contrast at `k*` (plus Stage 1's split of `SlotWait` by cause + HoL `st.taken`) directly measures whether doubling `W` kills those exact `idx_HoL + 4k` waits (and whether `READY_BYTES = 12 MiB` then takes over as the secondary cap!).
5. **Production default change?** **No** for the MVA/fluid surrogate; **Yes** if `Shared::full`'s slot accounting is changed to exempt tiny blocks (adoption registration required, low priority given the `≤ 1.8%` ceiling on Wikidata).

---

### Opportunity 6 — Mixture-Aware Statistical Inference & Log-Scale Variance Allocation for Stage 1–2

1. **Course tool:** Summarising ratios & ratio games (Jain Ch. 11–12), confidence intervals & mixture/non-normal pitfalls (Jain Ch. 12–13; 15-849b "why confidence intervals don't work"), multiplicative (log) models and allocation of variation for `2×2` and general factorials (Jain Ch. 18, 21–23), and trace-driven DES verification/validation + transient removal (Jain Ch. 24–25).
2. **Decoder mapping:**
   - **Empirical trap proved in `scan-profile-report.txt`:** At `k = 16`, the report gives `timed / untimed = 1.0512 [1.0099, 1.0942]` — looking like a statistically significant `+5.1%` timer overhead at `k = 16` alone (vs `0.996–1.005` at `k = 1..8, 17..20`). Inspecting `runs.tsv` proves this is a **bimodal mixture flip**: `15/20` timed runs vs `11/20` untimed runs landed in the slow SMT mode (`> 0.215 s`), shifting the median across the `35 ms` bimodal gap! Likewise, the `±2.3%` to `±4.6%` 95% mean half-widths at `k = 10..17` are mixture-variance artefacts, not Gaussian noise.
   - **Exact application:** In Stage 1–2, stratify every observation at `k ≥ 9` by the logged `sched_getcpu()` SMT-collision indicator (splitting the mixture cleanly into its two modes before computing CIs), and analyse Stage 2's `2×2` contrasts (`placement × turbo`, `W × READY_BYTES`) and `3×3` `(scanner, worker)` surface on log-wall with Jain's allocation of variation (`SST = SSA + SSB + SSAB + SSE`).
3. **Predicted size & assumptions:** Cuts effective residual CV in the `k = 10..17` band from `~5–8%` (unstratified mixture) back to `~0.5–1.5%` (within-mode CV, matching `k = 1..8`'s `0.1–0.9%` half-widths in `scan-profile-report.txt`), giving **`> 4×` sharper statistical power** for Stage 1/2 contrasts at zero extra machine time `[PROVED from within-mode spread in runs.tsv]`.
4. **Cheapest workstation measurement & Stage 2 fit:** **Zero extra runs** — pure analysis discipline riding directly on Stage 1's `sched_getcpu()` logging (§1.5) and Stage 2's contrasts (§2.2–2.3).
5. **Production default change?** **No** (analysis methodology only).

---

## 2. Course Tools Ruled Out by the Measurements (with Numbers)

| Tool & Course Reference | Why the Measurements Rule It Out | Key Measured Numbers |
|---|---|---|
| **Size-based & preemptive single-queue scheduling: SJF, PSJF, SRPT, FB/LAS, SOAP, Gittins/Whittle index, Priority queueing** (Harchol-Balter Ch. 28–33; Scully SOAP paper; 15-849b Part III) | Four independent kills (`[PROVED]`): (1) **Efficiency ceiling:** `efficiency_vs_busiest_server ≥ 0.945` everywhere (`0.969` at `k = 8`, `0.972` at `k = 20`), so no scheduling discipline can recover more than `3.1%` at `k = 8` (`5.9 ms`) or `5.5%` at `k = 10` (and most of that gap is startup/drain/SMT, not queueing). (2) **Closed batch network:** Schroeder, Wierman & Harchol-Balter (**NSDI 2006**, on 15-857's list) prove scheduling gains collapse in closed systems with bounded concurrency. (3) **Low real-block variability (not heavy-tailed / Pareto):** the 932 real blocks have `C²_bits = 0.083`, `C²_syms = 0.063`, `max/median = 2.07×` — strongly sub-exponential (`E_15`-like), the opposite of Pareto. (4) **Ordered retirement (`st.taken`, `src/parallel.rs:941–958`) + makespan metric:** prioritizing tiny stub blocks ahead of a big HoL block *delays* retirement and hurts makespan; FCFS (`Jobs::pop_front`) is already order-optimal for HoL latency. | `eff ≥ 0.945` (`0.969` at `k=8`); `C²_syms = 0.063`, `C²_bits = 0.083`; `max/med = 2.07×`; `slot_wait = 0.6 ms` (`0.3%`) at `k = 8`. |
| **Decentralized task assignment / size-interval routing: SITA-E, SITA-V, JSQ, Round-Robin, Random, LWL** (Harchol-Balter Ch. 24; **SIGMETRICS 2009** "Surprising results on task assignment") | `mibz2` already uses a **central shared queue** (`Jobs`, `src/parallel.rs:188–249`) feeding homogeneous workers (`[PROVED]`). Central-queue FCFS strictly dominates decentralized routing whenever `C² ≤ 1` (here `0.06–0.08`). SITA only beats central-queue FCFS under extreme heavy tails (`C² ≫ 10`), and partitioning workers by size here would leave size-class workers idle while big HoL blocks wait, worsening makespan. `Jobs` hand-off contention is negligible at optimal `k` (`send_ns = 0.29 ms` total at `k = 8`, `0.15%` of wall). | Central queue `Jobs`; `C²_syms = 0.063`; `send_ns = 0.29 ms` at `k = 8`; `worker_idle ≤ 3.6 ms` for `k ≤ 7`. |
| **Redundancy-d / speculative duplicate execution** (Gardner et al., **Operations Research 2017**, 15-857 reading list) | Requires independent, high-variance stochastic server slowdowns and cheap idle capacity. Here worker service times are deterministic (`C² ≈ 0.08`) and workers contend fiercely on shared 20 MiB L3, DRAM bandwidth, and SMT cores: adding active workers past `k = 8` inflates total worker CPU by `+10%` to `+42%` and triggers `+23%` scanner SMT slowdown (`[PROVED]`). Duplicate jobs would strictly worsen wall, CPU, and RSS. | `W_tot`: `1.429 s` (`k=8`) → `1.670 s` (`k=10`, `+17%`) → `2.027 s` (`k=15`, `+42%`); `scan_w`: `+23%` under SMT. |
| **Active process migration for heavy-tailed jobs** (Harchol-Balter & Downey, **SIGMETRICS 1996**; Harchol-Balter Ch. 20 §20.6) | Requires decreasing hazard rate (Pareto heavy tails) so old jobs have more remaining work than new ones. Real blocks are hard-bounded (`≤ 900 KB`, `max/median = 2.07×`, increasing hazard rate), and migrating an in-flight worker blows up its `~2.2 MB` L2/L3 cache state (`[PROVED]`). | `max_syms = 199,770` vs `median = 96,415` (`2.07×`); `C² = 0.063`. |
| **Exact product-form Jackson / BCMP / open M/M/k networks** (Harchol-Balter Ch. 14, 16–18, 22; Jain Ch. 31–32) | Violated on five counts (`[PROVED]`): (1) closed window `W` + byte cap `READY_BYTES`, (2) HoL in-order retirement, (3) load-dependent contention `μ(k)` for `k > 8`, (4) sub-exponential (`C² = 0.06–0.14`) and positively autocorrelated (`r₁ = +0.74`) big-block service times, (5) deterministic single-trace workload. Valid only as a documented surrogate (§60). | `r₁(big bits) = +0.739`; `C²_bits = 0.083`; `W(15)/W(8) = 1.42`. |
| **Matrix-geometric / QBD & Laplace/z-transform busy-period machinery** (Harchol-Balter Ch. 21, 25–27; 15-849b Part II–III) | Built for open-queue tail distributions and heavy-tailed `M/G/1` busy periods. With `eff ≥ 0.945` and a deterministic 1,724-block trace, the entire non-asymptotic queueing curvature at `k = 8` is `≤ 3.1%` (`5.9 ms`) and already captured to `2.4–3.7 ms` by a 10-line 2-station LD-MVA surrogate + Stage 1–2 trace-driven DES (`[ARGUED]`). | `wall_u(8) - scan_w(8) = 4.9 ms` (`2.6%`); LD-MVA error `2.4–3.7 ms` across `k = 3..8`. |
| **Naive Halfin–Whitt square-root staffing (`k = R + c√R`)** (Harchol-Balter Ch. 15 §15.3) | Assumes constant per-server speed with zero contention cost. With `R = 7.67`, even `c = 1` staffs `k = 11` workers — overshooting straight into the L3/SMT contention regime (`[PROVED]`). Must be capped by `P_phys - 1` and near-zero hedge `δ` (Opportunity 3). | `k=11` vs `k=8`: wall `+2.9%` (`0.1967` vs `0.1911 s`), CPU `+21%`, RSS `+20%`, `7/20` slow SMT runs. |
| **Dynamic power ON/OFF sleep-state cycling** (Harchol-Balter Ch. 27; AutoScale **TOCS 2012** energy mode) | A decode finishes in `0.19 s`; hardware sleep-state power cycling does not apply within one decode (only its **setup-cost** analogue — lazy worker spawn/gating to avoid cold-heap minor faults on short/scanner-bound streams — carries over, covered in Opportunity 3) (`[ARGUED]`). | `wall(8) = 0.191 s`; `minflt` `+958/worker` (`k=1..8`), `+441/worker` (`k=8..20`). |
| **`2^(k−p)` fractional factorials across the bottleneck kink** (Jain Ch. 19) | Already ruled out in §60: `T = max(a, b)` generates strong multi-factor interactions that alias onto main effects in Resolution III/IV designs; replaced by OFAT screening + unaliased `3×3` `(scanner, worker)` surface (`[PROVED]`). | See `DESIGN-REVIEW.md` & `briefs/character-review-decisions.md` §3. |
| **Cooperative worker teams (`Plan::Teams`, parallel jobs with speedup functions, 15-857 datacenter unit) on the scanner-bound plateau** | Above `k* = 7.67` on this corpus, reducing individual block latency `w` via a multi-thread team cannot speed up the serial scanner (`U_scanner = 0.969`), and can at most trim the `0.6–3.6 ms` (`0.3–1.8%`) HoL stall on the 4 dense blocks (`[PROVED]`). Teams remain useful only on tiny/latency-critical 1–3 block files or if scanner parallelism shifts the bottleneck back to workers under L3 pressure. | `slot_wait = 0.6 ms` (`k=8`), `3.6 ms` (`k=9`), `2.9 ms` (`k=10`). |

---

## 3. Comparison Against `leads/course-opportunities.md` (Opened After §§0–2)

### 3.1 Where We Agree

- **Core diagnosis:** Against the busiest server (`D_scanner`), efficiency is `0.945–0.998`, so queueing disciplines (SJF, SRPT, SITA, Redundancy-$d$, heavy-tail machinery) are dead ends on this corpus — consistent with the closed-batch result of Schroeder, Wierman & Harchol-Balter (NSDI 2006).
- **Top three architectural/systems themes:** (i) whole-machine topological ceiling motivates parallel/cheaper scanning; (ii) adaptive online worker sizing via a Renewal-Reward estimate of `w/s` (tested across 3–5 corpora) beats fixed over-provisioning; (iii) scanner core/SMT isolation tests the `k ≥ 16` cliff (already folded into Stage 2's placement × turbo contrasts).
- **Modelling:** `wall(k) = c + max(b, a/k)`'s `32 ms` constant `c` is a knee-curvature/cliff artefact; trace-driven DES is primary, with MVA on a documented surrogate (our exact 2-station LD-MVA in §1 Opportunity 5 confirms this, hitting `k = 3..8` within `2.4–3.7 ms` with `c = 0`).
- **Window & classes:** Tiny stub blocks halve the nominal `4k` slot window, and real blocks have low marginal variability (`C² ≈ 0.06–0.08`).

### 3.2 Where We Disagree (with Exact Arithmetic)

#### Check 1 — The machine-bound estimate (`0.109 s` / `43%` gap from an inferred `1.49×` SMT factor): **DISAGREE**

`leads/course-opportunities.md` §1 argues:
> `workers_busy_s` rises `1.43 s` (`k = 8`) → `1.92 s` (`k = 20`); assuming all of that is 2-per-core SMT sharing gives `1.43 / 1.92 = 0.745` per sibling → `1.49` lone-thread equivalents per core → `14.9` for 10 cores → `1.62 / 14.9 = 0.109 s` (`0.57` machine efficiency, `43%` gap).

Three arithmetic/physical flaws make `0.109 s` (`43%`) substantially **too aggressive** (`[PROVED]` from `runs.tsv`):

1. **The occupancy fallacy (`active_sw = 9.25`, NOT `20`):** At `k = 20`, `wall_t = 0.2320 s`, `workers_busy_s = 1.9201 s`, and `scanner_work_s = 0.2255 s`. The mean number of simultaneously active threads is only `(1.9201 + 0.2255) / 0.2320 = 9.25` (`8.28` busy workers + `0.97` scanner; `11.44` of `20` workers are **idle** on average!). Treating `1.43 / 1.92 = 0.745` as the per-thread speed of **20 simultaneously busy threads** confuses a 9.25-active-thread state (partial SMT overlap + cold-cache worker switching) with full 20-thread saturation!
2. **Direct counter-example in the same table (`k = 15` and `k = 16` fast mode):** At `k = 15`, `active_sw = 11.18` (only ~1–2 cores SMT-paired on average, `14/20` runs still solo-scanner), yet `workers_busy_s` **already** inflates to `2.0266 s` (`1.4286 / 2.0266 = 0.705`, worse than `k = 20`'s `0.745`!). In the `5/20` fast-scanner runs at `k = 16`, `workers_busy_s = 2.088 s` (`0.684`). Moving from `k = 15` to `k = 20` *lowers* `workers_busy_s` (`2.027 → 1.920 s`) only because the scanner's `+23%` SMT slowdown drops active concurrency from `11.18` down to `9.25`! Consequently, `leads/course-opportunities.md`'s caveat (*"The 0.745 factor also absorbs memory-bandwidth and cache contention"*) points the wrong way: at **20 genuinely simultaneous BWT/scan threads**, full 10-core SMT + 20-way L3 thrashing + DRAM contention will push per-thread speed **below** `0.68`, not above `0.745`!
3. **Omitted DRAM bandwidth server & reader/sys CPU:** Total CPU at `k = 8` is `user_s + sys_s = 1.628 s` (not `1.62 s`), and Jain Ch. 33–34's Balanced Job Bounds require the memory-controller/DRAM server `D_dram = B_total / B_max` alongside CPU.
4. **Corrected bound (`[PROVED]` physical / `[CONJECTURED]` SMT):**
   - **10 physical cores, no SMT (`M_eff = 10.0`):** `1.628 / 10.0 = 0.1628 s` → machine efficiency **`0.852`** (**`14.8%` gap**).
   - **10 cores with realistic memory-bound SMT (`1.15–1.30×`, `M_eff = 11.5–13.0`):** `0.125–0.142 s` → machine efficiency **`0.65–0.74`** (**`26–35%` gap**), subject to Stage 1's IMC DRAM bound and Stage 2's `k = 4` `2×2` SMT measurement.

#### Check 2 — Claim that the scanner, not the in-order reader, limits above the knee: **AGREE, WITH TWO SHARPENINGS**

- **Agree (`[PROVED]`):** `leads/course-opportunities.md` §1's aggregate check (`k = 10`: scanner busy `0.945`, `slot_wait = 0.014`, `input_wait = 0.007`, `1.67` idle workers; `k = 15..20`: `slot_wait = 0.0001 s`) is sound, and we add an independent CPU proof from `runs.tsv`: total non-(scanner+worker) CPU `(user_s + sys_s) - (scanner_work_s + workers_busy_s)` — upper-bounding the reader, `mibz2-read` feeder, and thread spawn/join combined — is only **`13–21 ms` at `k = 8`** and **`26–28 ms` at `k = 10`** (`≤ 10–14%` of wall). The reader's CPU is nowhere near a bottleneck.
- **Sharpening (`[PROVED]` from `runs.jsonl`):** While the reader's *work* never limits, its **in-order HoL resequencing constraint** (`st.taken`, `src/parallel.rs:941–958`) IS the exact cause of the `0.6–3.6 ms` (`0.3–1.8%`) `slot_wait` at `k = 8..12` — proved by `idx_stall = idx_HoL + 4k` behind blocks `mod 431 ∈ {56, 165, 171, 278, 388}` (§1 Opportunity 5). Because that ripple is `≤ 1.8%` and disappears by `k = 13..20` (`0.1 ms`), it does not alter the verdict that the scanner's own `94.5–97.4%` work limits above `k*`.

#### Check 3 — Reading that the 792 non-Open blocks are the 792 blocks `< 10 kbit`: **STRONGLY CORROBORATED, WITH TWO PRECISION CORRECTIONS**

- **Corroborated (`[PROVED]` window structure / `[ARGUED]` set identity):** `bench/wikidata.json` confirms **12/12 windows** satisfy `decoded_bytes // 900_000 == count(bits >= 10,000)` (`233` distinct full 900 kB blocks = `932` in `×4`; `198` distinct tiny overflow stubs = `792` in `×4`, `bits ∈ [184, 4,328]`, `0.0947%` of bits, separated by a `65×` bit gap and `83×` symbol gap from the smallest big block at `281,464` bits / `70,295` symbols).
- **Correction 1 (epistemic label):** Matching totals across two separate runs (`792` non-Open in `four-arm-runs.tsv` == `792` `< 10 kbit` in `runs.jsonl`) is very strong `[ARGUED]`, not deductive `[PROVED]` set equality until Stage 1 logs per-block `open: bool`: if even 1–2 dense big blocks (`blk 56, 165, 171, 278, 387` at `182–199k` syms) hit `Cap::dense` (`src/open.rs:673–674`) while 1–2 larger stubs (e.g. `blk 167`, `845` syms) passed `OpenCells`, the counts would still tie (`792 == 792`).
- **Correction 2 (cost of the 792 stubs to the scanner):** `leads/course-opportunities.md` §6 says *"The tiny ones are almost free to the scanner (p10 read 1 µs)"*. Their own Huffman work is indeed tiny (`1.23 ms`, `0.66%`), **but** indirectly they are NOT free: (a) they truncate recycled `free_syms` buffers to `~22` symbols, causing `~5.0 ms` (`2.7%` of `D_scanner`) of steady-state `syms.resize(max_out, 0)` zero-fill on subsequent big blocks (`src/block.rs:221–222, 268`); and (b) they occupy `46%` of the `4k` slot window, halving the effective big-block window and triggering the `idx_HoL + 4k` waits at `k = 8..12`.

#### Check 4 — Python binding's default (`src/python.rs`) and what an adaptive worker cap needs: **CODE CORRECTION + FOUR DESIGN REQUIREMENTS**

- **Code correction (`[PROVED]` from `src/python.rs` & `src/parallel.rs`):** `leads/course-opportunities.md` states that on this i9-10900K, Python's default (`threads=None` → `available_parallelism() = 20` in `src/python.rs:24–28`) runs 20 workers and pays `+20%` wall / `+34%` CPU / `+38%` RSS. **In current `src/` that is false on the unpinned Linux i9-10900K:** `src/python.rs:110, 183` enters `StreamDecoder::new(20, 80, k)`, where `plan_with(20, Some(10), Some(20 MiB))` (`src/parallel.rs:3505–3506`) returns `Plan::Workers(10)` and `trimmed_here` (`src/parallel.rs:3350–3394`) trims `threads >= 10` to **`TRIMMED_WORKERS = 7` lazy workers (`window = 28`)**! Thus on the measured i9, Python's default currently **under-provisions** Wikidata ×4 by 1 worker (`k = 7`: `0.2069 s` vs `k = 8`: `0.1911 s`, `+8.3%` wall!). Where `src/python.rs` *does* spawn all `L` logical CPUs (paying the full SMT penalty) is on **macOS/Windows** (`physical_cores_of -> None`, `src/parallel.rs:3552–3556`) and **Linux with `phys ≤ 5`** (`cores < 6`, `src/parallel.rs:3498`); on non-i9 Linux `phys ∈ {8, 9, 10}` it spawns `phys` workers, and on `phys ≥ 11` it falls back to `Teams { k: 2 }` (`src/parallel.rs:3511–3517`).
- **Four requirements an adaptive worker cap needs (beyond `course-opportunities.md` §2):**
  1. **Physical-core ceiling `k ≤ P_phys - 1` (reserving a core for `mibz2-scan`), NOT uncapped `R + β√R`:** Naive Halfin–Whitt overshoots into L3/SMT contention (`k = 11` is slower and `+21%` dearer in CPU than `k = 8`). Use `k* = min(P_phys - 1, ceil(R̂ + δ))` with `δ ∈ [0, 0.5]`.
  2. **Wait-excluded Renewal-Reward ratio of sums:** `R̂ = (Σ w_busy) / (Σ s_busy)` over a sliding window of completed blocks, strictly **excluding** `Jobs::recv` idle wait from `w_busy` and `SlotWait`/`InputWait` from `s_busy` (otherwise closed-loop backpressure biases `R̂` downward). Using ratio of sums (not mean of per-block ratios) prevents the 792 stub blocks from blowing up `R̂`.
  3. **Autocorrelation-safe window + hysteresis:** Because big-block sizes have `r₁ = +0.739` and a `2.5×` density spread across `w00..w11`, the window must span `≥ 16–32` big blocks (`~3–6 ms`) with hysteresis (`±0.5` worker deadband) to avoid thrashing on 4-block dense bursts.
  4. **Lazy ramp + permit-gated parking (`AutoScale` / `M/M/k/setup`):** Start at `k₀ = min(4, P_phys - 1)` so short files never pay `~440–960` minor faults and `~2–3 MB` RSS per idle worker; spawn lazily up to `k*` and gate active permits (park rather than destroy/re-fault threads) when `R̂` drops mid-stream.

#### Additional Disagreement — Favoured-core turbo claim in `course-opportunities.md` §3

`leads/course-opportunities.md` §3 hypothesizes that pinning `mibz2-scan` to an ITBM3 favoured core with its sibling idle recovers *"about 7% on the plateau and the whole 20% step at k ≥ 16"*. We agree on the SMT step (`~15–18%` at `k ≥ 16` and the `k = 10..15` tail lottery), **disagree on recovering `7%` at `k = 8` on i9-10900K:** Comet Lake-S sets all-core turbo package-wide (`4.9 GHz` at ≥ 6 active cores) regardless of core ID, so favoured-core pinning at `k = 8` will likely recover `0–2%`, not `7%` (Stage 2's `placement × turbo` contrast will decide it cleanly). Conversely, on **hybrid P/E-core CPUs** (not mentioned in `course-opportunities.md`), pinning `mibz2-scan` to a P-core is a genuine large asymmetric-server win.

### 3.3 What `leads/course-opportunities.md` Missed

1. **Two immediate, zero-algorithm scanner-bottleneck plumbing wins (`15.1–18.6 ms` = `8.1–9.7%` of `D_scanner`, `[PROVED]`):**
   - `ChannelSource::view` (`src/parallel.rs:997–1021`) copying/shifting `4 MiB` chunks on 11 exact chunk boundaries = **`10.07 ms` (`5.41%` of `D_scanner` at `k = 8`, `13.55 ms` / `7.06%` at `k = 15`)**;
   - `Phase::Prep` (`src/block.rs:220–223, 268`) re-zeroing recycled `free_syms` buffers shortened by stub blocks = **`5.02–5.56 ms` (`2.7–3.0%` of `D_scanner`)**.
   Combined, these are larger than *every* non-decode phase in the scanner (`tables` or `selectors`), require zero algorithmic risk, and come straight out of Modification Analysis (Harchol-Balter Ch. 7) on `runs.jsonl`.
2. **The Modification Analysis bottleneck-shift cliff at `k = 8` (`[PROVED]`):** Because `D_scanner = 0.1862 s` is only `4.1%` (`7.6 ms`) above `D_worker = 0.1786 s` at `k = 8`, any scanner speedup `> 4.1%` (even just fixing `ChannelSource` + `Prep`!) immediately re-binds workers at `k = 8` and requires `k = 9`; and any scanner speedup `≥ 25%` pushes the uncontended crossover past `10` physical cores into the SMT/L3 wall.
3. **Bimodal SMT mixture in `runs.tsv` (`k = 10..18`) and its statistical consequences (`[PROVED]`):** `runs.tsv` is bimodal from `k = 10` onward (not a deterministic cliff at `k = 16`), which explains the spurious `1.0512 [1.0099, 1.0942]` timer ratio at `k = 16`, explains why `W_tot` *drops* at `k = 16`, and unlocks `> 4×` sharper variance reduction in Stage 1–2 via `sched_getcpu()` mode stratification (Jain Ch. 12–13 / 15-849b).
4. **Big-block lag-1 autocorrelation (`r₁ = +0.739`) and the exact `idx_stall = idx_HoL + 4k` proof:** Shows that even with low marginal `C² = 0.06–0.08`, dense blocks arrive in bursts (`{56, 165, 171, 278, 388}`), directly linking burst correlation (15-849b Part III) to Stage 2's window-permutation and `W × READY_BYTES` contrasts.
5. **Additional ruled-out tools from the reading list:** Active process migration (`SIGMETRICS 1996`), naive Halfin–Whitt overshooting under load-dependent contention (Ch. 15), cooperative teams on the scanner-bound plateau (15-857 datacenter unit), and open/transform/busy-period machinery (Ch. 25–27).
