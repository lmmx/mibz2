# Dissecting the 8-worker coupling gap

*Recurring [`explore`](../../../frontier/claims/explore.toml) investigation (27 November 2026), cold from primary i9-10900K traces and `src/parallel.rs` / `src/read.rs`. Smooth zone only: zero `src/` edits, no frozen gate reopened (C2).*

## Primary evidence verified

All three `origin/evidence` bundles were fetched and checked against their recorded `.sha256` digests:

1. `character/run-20261125T001135Z.tar.gz` (`bb72107b…2204`, checked): 48 grid observations at `k = 8` (`observations.tsv`), `features.tsv` (431-block `wikidata` corpus), and 4 complete nanosecond per-block traces (`blocks-r00.tsv`–`blocks-r03.tsv`, 1,724 blocks each).
2. `zero-copy-floor/run-20261126T112751Z.tar.gz` (`10df0d89…5e34`, `cc8ba137`, checked): `runs.jsonl` (8 blocks, 4 traced per cell, scanner pinned to CPU 6).
3. `pooled-scan/run-20261126T133034Z.tar.gz` (`97e397ad…8867`, `c71c4a56`, checked): `runs.jsonl` (12 blocks, 3 traced per cell, unpinned).

## Bottom line

At `k = 8` on `wikidata ×4` (1,724 blocks, ~839 MB output), the coupling gap `wall − max(scan_busy, worker_busy/8)` is **7.57 ms [p10 6.35, p90 10.08] (3.95%)** across the 48 `character-stage1` grid runs, **9.13 ms (4.67%)** across the 4 per-block traced rounds (`r0`–`r3`), and **8.0–8.9 ms** in `pooled-scan` (`none − max(C, S) = 8.0 ms` untimed; `193.72 − 184.86 = 8.86 ms` traced).

**Two corrections to prior accounting:**

1. **Do not subtract `SFCMRD`'s `5.2 ms` plumbing from the `~8–9 ms` gap.** When real work runs, per-block handoffs (`issue` 0.65–0.70 ms, `pool+publish` 0.07 ms/worker, `lock_state+lock_jobs` 0.27–0.33 ms scanner / 0.05–0.06 ms worker) are overlapped behind real work or already inside `scan_busy`/`wkr_busy`. In `pooled-scan` `S` and `SC` (scan free), `wall − wkr/8` is only **1.16 ms** and **1.12 ms** (matching `zero-copy-floor` `SFMRD` 0.88 ms and `SECMRD` 0.69 ms), not 5.2 ms. The unoverlapped wakeup/handoff residual above `max(S, W/8)` is **~1.15 ms**, not 5.2 ms.
2. **Removing `ChannelSource::view`'s `in_maint` stalls (`scanner-seam-window`, #857) eliminates ~2.15 ms of the coupling gap *in addition* to its ~10–11 ms direct scan cut.** Today's `in_maint` is not steady drag; it is 11 discrete 0.53–1.54 ms stop-the-world stalls, front-loaded in copies 0–1 (`3.81 / 3.09 / 1.86 / 1.40 ms` across `c0..c3`), which induce a mid-decode phase inversion (workers starve in `c0–c1`; scanner slams into the 32-slot window in `c2–c3`).

**No standalone >= 2.5–3% unowned lever hides in the coupling gap.** Once `scanner-seam-window` removes `in_maint`, a future exact tie at `S = W/8 = 150 ms` has a total gap of **~7.5 ms (4.8% of 157.5 ms)**, of which **~4.5 ms** is an irreducible queueing/startup/handoff floor (`1.22 ms` thread launch + `2.19 ms` indivisible-block variance & drain tail + `~1.0–1.15 ms` handoff residual). The two remaining unowned mechanisms together cap at **~2.5 ms (1.60% at 150 ms; ~1.8% today)**:
- **Runt-exempt slots (or `slots = 64`):** **−1.18 to −1.21 ms (−0.76%)** at a 150 ms post-`in_maint` tie (−2.10 ms / −1.08% today with `in_maint`);
- **Smaller initial feeder chunk (`src/read.rs:13, 256–261`):** **−1.30 to −1.35 ms (−0.85%)** at a 150 ms tie (−0.68% today), one-shot per stream.

No new `idea` claim is opened.

---

## 1. Cross-bundle primary baseline at `k = 8`

| Bundle & arm (k = 8, medians, ms) | Untimed wall | Traced wall | `scan_busy` | `wkr_busy/8` | `wall − max` | `slot_wait` (waits w/c) | `in_wait` | `job_wait/8` (`sa / in / adm`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `character-stage1` grid `none` (n=48) | — | 192.27 | 184.45 | 182.67 | **7.57** | 3.25 (13/0) | 1.39 | 8.41 (—) |
| `character-stage1` `r0`–`r3` mean | — | 195.61 | 184.99 | 185.64 | **9.13** | 5.20 (19/0) | 1.43 | 9.57 (—) |
| `zero-copy-floor` `none` (pinned CPU 6) | 194.57 | 194.36 | 184.35 | 186.35 | **8.01** | 6.10 (24/0) | 1.26 | 6.94 (5.59 / 1.23 / 0.03) |
| `zero-copy-floor` `FCMRD` (scan alone) | 179.10 | 176.65 | 174.56 | 0.64 | 2.09 | 0.68 (0/0) | 1.25 | 175.93 |
| `zero-copy-floor` `SFCMRD` (all free) | 5.28 | 5.80 | 2.01 | 0.86 | 3.79 | 0.81 (0/0) | 0.00 | 4.55 |
| `pooled-scan` `none` (unpinned) | 192.93 | 193.72 | 184.79 | 184.86 | **8.86** | 4.11 (14/0) | 1.34 | 8.50 (7.14 / 1.29 / 0.04) |
| `pooled-scan` `C` (scan-bound) | 183.18 | 186.60 | 182.58 | 135.66 | 4.02 | 1.55 (3/0) | 1.32 | 50.56 (48.51 / 1.22 / 0.24) |
| `pooled-scan` `S` (worker-bound) | 184.88 | 185.35 | 1.61 | 184.19 | **1.16** | 180.14 (1342/0) | 0.00 | 0.22 (0.02 / 0.00 / 0.17) |
| `pooled-scan` `SC` (both ideal) | 135.31 | 133.30 | 1.46 | 132.18 | **1.12** | 129.15 (1256/0) | 0.00 | 0.23 (0.05 / 0.00 / 0.18) |

In `character-stage1`, `S >= W/8` in 40/48 grid runs (`W/8 > S` in 8/48); across the 4 traced rounds, `r1` and `r2` have `S > W/8` (−0.67, −2.70 ms) while `r0` and `r3` have `W/8 > S` (+4.32, +1.65 ms).

---

## 2. Exact physical components of the gap

### (a) Block-service structure, runt slot dilution, and `in_maint` phase inversion

From `features.tsv` (431 blocks per copy, repeated ×4 = 1,724 blocks):
- **233 full blocks** (228 `open` + 5 `open_dense`, `n` = 899,809–900,000) and **198 runts** (150 `skipped` median `n = 30`, 48 `column` median `n = 8`).
- **No runt bursts:** runt streak lengths have **max = 2** (median 1); full-block streaks have max = 7 (median 1). The stream alternates `F.F.F.F…` almost everywhere.
- **Per-class work in `r1` (`k = 8`):** full blocks account for `169.57 ms` of `A2+A3` and `182.26 ms` of `wkr/8` (`196 µs/blk` per worker); the 792 runts account for only **`1.39 ms`** of `A2+A3` (`1.75 µs/runt`) and **`0.32 ms`** of `wkr/8` (`0.4 µs/runt`).
- **Runt slot dilution in `Shared::full` (`src/parallel.rs:47, 74, 402–408, 686–690`):**
  - `SLOTS_PER_THREAD = 4` gives `slots = 32` at `k = 8`. Every block — 900 KB full or 9-byte runt — consumes one slot (`ahead = st.issued - st.taken >= self.slots`).
  - `READY_BYTES = 12 MiB` **never fires at `k = 8`**: `slot_cap = 0.00 ms` across all 48 `character-stage1` runs (`waits w/c/b = 13/0/0`; median `hwm_ready_bytes = 6.01 MiB`, p90 `10.30 MiB`), and `0` cap waits in both `zero-copy-floor` (`24/0`) and `pooled-scan` (`14/0`).
  - Because half the stream is runts, **32 slots holds only 15–17 full blocks** (`15–16` slots are always occupied by tiny runts).
  - Inspecting every `s_slot_wait > 10 µs` in `r1` reveals an exact deterministic mechanism: a single heavy block (`mod 56` `open_dense` `w = 2.8–3.4 ms`, `mod 171` `w = 2.5–2.7 ms`, `mod 278/279` `w = 2.1–2.5 ms`, `mod 389` `w = 2.5 ms`) stays executing at head-of-line (`state = X`), pinning 10–19 ready blocks behind it while the scanner scans 32 places ahead (`88 − 56 = 32`, `203 − 171 = 32`, `310 − 278 = 32`, `311 − 279 = 32`, `421 − 389 = 32`) and hits `ahead == 32` in every copy of the corpus.
- **`in_maint` stop-the-world stalls (`src/parallel.rs:49, 1039–1063`) amplify the coupling gap:**
  - `ChannelSource::view` drains and re-copies 4 MiB chunks (`COMPACT = 4 MiB`, `CHUNK = 4 MiB` at `src/read.rs:13`) on the serial scanner thread. In `r1`, `10.07 ms` of `in_maint` arrives as **11 discrete stalls of 0.16–1.54 ms** (9 of them > 0.66 ms).
  - Across the 4 identical 431-block copies (`c0..c3`, 4-round mean at `k = 8`), `in_maint` drops sharply as `ChannelSource.buf` finishes growing/faulting: **`3.81 → 3.09 → 1.86 → 1.40 ms`**, pulling total `S_busy` from **`48.74 → 46.75 → 44.89 → 44.61 ms`** while `W_busy/8` shifts less (`47.95 → 46.63 → 45.80 → 45.22 ms`).
  - This non-stationary slope puts the scanner behind workers in `c0–c1` (generating worker starvation) and ahead of workers in `c2–c3` (generating `slot_wait`).

### (b) Simultaneity of `slot_wait` and `job_wait`

Interval-intersection over the exact nanosecond timestamps in `blocks-r00..03.tsv` at `k = 8` shows the two waits are **sequential, not simultaneous**:

| Round (`k = 8`) | Total `slot_wait` | Interval with >= 1 idle worker | Mean worker-idle/8 during `slot_wait` |
|---|---:|---:|---:|
| `r0` | 9.61 ms | 1.31 ms | 0.276 ms |
| `r1` | 3.27 ms | 0.17 ms | 0.042 ms |
| `r2` | 2.18 ms | 0.13 ms | 0.033 ms |
| `r3` | 5.74 ms | 0.18 ms | 0.035 ms |

This matches `pooled-scan`'s `idle_admission_ns / 8 = 0.04 ms` and `zero-copy-floor`'s `0.03 ms`. During `slot_wait`, all 8 workers are busy (`X = 8`, `Q = 6..14`, `R = 10..19`). Worker mid-stream starvation (`mid_idle = 5.9–7.5 ms/worker`; `idle_scanner_active = 5.6–7.1 ms/worker`) happens later while the scanner is busy — and because `S ≈ W/8`, every millisecond the scanner lost to `slot_wait` earlier converts into downstream worker starvation (or vice versa).

### (c) Startup (`in_wait`), end-of-stream tail, and cache-contention cross-talk

- **Startup (`2.41 ms` median in 48 runs; `2.62 ms` mean in `r0`–`r3`):** splits cleanly into:
  1. **~1.22 ms** (`1.09–1.73 ms`) pre-scan thread launch (`t_first_scan_start`) for spawning `k + 2` OS threads (`mibz2-0..7`, `mibz2-scan`, `mibz2-read`);
  2. **1.39 ms** (`1.26–1.40 ms` across all three bundles) block-0 `in_wait`: `BzReader::on` (`src/read.rs:13, 256–261`) allocates `vec![0u8; 4 << 20]` (1,024 fresh 4 KiB pages) and `read_full`s all **4 MiB** (`feeder_A0(startup) = 1.36–1.39 ms`) before pushing chunk 0, while the scanner and all 8 workers sit 100% idle waiting for a ~25 KiB first block.
- **End-of-stream drain:** `t_last_consumed − t_last_enqueued` is **2.50 ms** median across 48 runs (`2.15–2.63 ms` worker tail + `0.03–0.08 ms` final reader copy; last-block finish spread across the 8 workers is `1.31–1.56 ms`, contributing `0.63–0.70 ms` mean per-worker drain idle; `pooled-scan`'s `idle_drain_ns = 0.00 ms` only counts workers still blocked in `Jobs::recv`, not workers that already exited on `None`).
- **Scanner/worker cache cross-talk (`zero-copy-floor` `none` vs `FCMRD`, both pinned to CPU 6):**
  - Scanner active time rises **+9.67 ms** from `FCMRD` (`174.24 ms`) to `none` (`183.91 ms`) at `k = 8`:
    - **+4.16 ms** is `in_maint` DRAM/L3 contention (`7.00 → 11.16 ms`, climbing monotonically `7.79 → 9.66 → 10.28 → 11.16 → 11.86 → 13.56 ms` from `k = 1` to `10` while `FCMRD` stays flat at `6.93–7.20 ms`) — already owned by `scanner-seam-window`.
    - **+1.50 ms** is all-core turbo drop (`4.946 → 4.900 GHz`) — hardware floor.
    - **+4.46 ms** in `A3` (`147.19 → 153.15 ms`, `+3.0%` cycles) is a **1-worker cross-core `free_syms` coherence handoff**, *not* 8-worker contention: `A3 × f_cpu6` is already `755.9` Gcycles in `none(1)` vs `723.3` in `FCMRD(1)` (where ideal `M` never reads `syms`), and stays completely flat at **`746–752` Gcycles across `k = 6..10`**.
  - Worker-side cross-talk from the active scanner is negligible: **`+0.67 ms` (+0.36%)** (`wkr/8 = 184.86 ms` in `none` vs `184.19 ms` in `S`, `pooled-scan` `k = 8`).

### (d) `src/parallel.rs` state-machine accounting

- `Shared::state` + `Jobs` mutex/condvar acquisition (`lock_state + lock_jobs`): **0.27–0.33 ms** on scanner, **0.05–0.06 ms** per worker (`<= 0.2%` of wall).
- `Jobs::push` / `recv` (`src/parallel.rs:188–249`) wakes one worker only when `waiting > 0`; `Ready` ring (`src/parallel.rs:99–183`) never overflows (`over` empty); `advance()` (`src/parallel.rs:985–994`) increments `taken` and wakes `scan_cv` *before* copying out.
- `OpenCells` dispatch (`src/parallel.rs:572–579`) routes 912 `open`, 20 `open_dense`, 600 `skipped`, 192 `column`, 0 `refused` with zero dispatch stall.

---

## 3. Discrete-event trace replay and counterfactual quantification

A discrete-event simulator implementing `src/parallel.rs`'s exact admission, FIFO `Jobs`, `k`-worker, and in-order `Ready` state machine was fed the observed per-block `(s_busy, w_exec, r_copy, out_bytes)` from `blocks-r00..03.tsv` (`k = 8`).

**Validation against real `r0`–`r3` traces (`slots = 32`, real `t_scan_start[0]` and `s_in_wait[0]`):**

| Round | `S` (ms) | `W/8` (ms) | `L = max` | Real `wall` (`sw`) | Sim `wall` (`sw`) | `Real − Sim` |
|---|---:|---:|---:|---:|---:|---:|
| `r0` | 183.99 | 188.31 | 188.31 | 198.89 (9.61) | 197.53 (8.72) | +1.36 ms |
| `r1` | 183.24 | 182.58 | 183.24 | 192.02 (3.27) | 190.93 (2.56) | +1.09 ms |
| `r2` | 184.13 | 181.43 | 184.13 | 192.33 (2.18) | 191.11 (1.46) | +1.21 ms |
| `r3` | 188.59 | 190.24 | 190.24 | 199.21 (5.74) | 198.27 (5.10) | +0.94 ms |
| **Mean** | **184.99** | **185.64** | **186.48** | **195.61 (5.20)** | **194.46 (4.46)** | **+1.15 ms** |

The simulator reproduces real wall within **0.94–1.36 ms** (mean **1.15 ms**, matching `S`/`SC`'s unoverlapped `1.12–1.16 ms` handoff residual) and `slot_wait` within 0.7 ms across all four rounds.

**Counterfactual sweep (4-round means, `k = 8`, excluding the ~1.0–1.15 ms handoff residual):**

| Regime & policy | Sim wall (ms) | `gap = wall − L` | Delta vs baseline | `slot_wait` |
|---|---:|---:|---:|---:|
| **Shipped `r0`–`r3` (`L = 186.48 ms`, with `in_maint`)** | | | | |
| Baseline (`slots = 32`, real startup + `in_wait`) | 194.46 | 7.98 ms (4.10%) | — | 4.46 ms |
| `slots = 48` | 192.57 | 6.09 ms | −1.89 ms (−0.97%) | 1.60 ms |
| `slots = 64` **or** `runt-exempt 32` **or** `slots = ∞` | 192.36 | 5.88 ms | **−2.10 ms (−1.08%)** | 0.87 / 0.90 / 0.00 ms |
| Zero startup + `in_wait`, `slots = 32` | 191.84 | 5.36 ms | −2.62 ms (−1.35%) | 4.46 ms |
| Zero startup + `in_wait`, `slots = ∞` | 189.74 | 3.26 ms | −4.72 ms (−2.43%) | 0.00 ms |
| Inline runt decode on scanner (`slots = 32`) | 195.58 | 7.72 ms | **+1.12 ms (+0.58%)** | 3.12 ms |
| **Exact `1.00` tie (`S = W/8`, zero startup): isolating `in_maint` & variance** | | | | |
| Real blocks *with* `in_maint` (`slots = 32` → `∞`) | — | 6.61 → 3.99 ms | −2.61 ms (window) | 4.25 → 0.00 ms |
| Real blocks *without* `in_maint` (`slots = 32` → `64` / `runt-ex 32` → `∞`) | — | **4.46 → 3.11 / 3.07 → 2.55 ms** | **−2.15 ms** (`in_maint` bonus) | 2.58 → 0.57 / 0.54 → 0.00 ms |
| Static content only (`place % 431` means, `slots = 32` → `48`/`∞`) | — | 3.80 → 2.45 ms | −1.35 ms | 1.52 → 0.00 ms |
| 2-class constant (`F` / runt means, `slots = 32` or `∞`) | — | 1.61 ms | — | 0.00 ms |
| 1-class constant (all blocks equal, `slots = ∞`) | — | 0.88 ms | — | 0.00 ms |
| **Future `150 ms` exact tie (POST `scanner-seam-window`, NO `in_maint`)** | | | | |
| `slots = 32`, real startup + `in_wait` (2.62 ms) | 156.47 | 6.47 ms (4.13%) | — | 2.23 ms |
| `slots = 32`, small first chunk (`in_wait = 0.05 ms`) | 155.12 | 5.12 ms (3.30%) | **−1.35 ms (−0.86%)** | 2.23 ms |
| `slots = 64` or `runt-exempt 32`, real startup | 155.26 | 5.26 ms (3.39%) | **−1.21 ms (−0.77%)** | 0.46 ms |
| `runt-exempt 32` + small first chunk | 153.91 | 3.91 ms (2.54%) | **−2.56 ms (−1.64%)** | 0.46 ms |
| `slots = ∞`, zero startup + `in_wait` (irreducible floor) | 152.19 | **2.19 ms (1.44%)** | −4.28 ms (−2.74%) | 0.00 ms |

*(Resonance check: scaling `(W/8)/S` across `[0.85, 1.15]` shows the coupling gap peaks sharply at `ratio = 1.00`: `4.82 → 5.98 → 9.23 → 6.73 → 5.19 ms` for `slots = 32`, and `4.05 → 4.54 → 6.61 → 4.76 → 4.35 ms` for `slots = ∞`.)*

---

## 4. Complete ledger and verdict

| Component of the 8-worker coupling gap | Today (`r0`–`r3`, ~195.6 ms) | Future 150 ms tie (post `#857`, ~157.5 ms) | Status / ceiling |
|---|---:|---:|---|
| 1. `in_maint` coupling amplification (`slot_wait` + `c0–c3` drift) | ~2.15 ms | 0.00 ms | **Owned** by `scanner-seam-window` (#857) |
| 2. 32-slot HOL window (half-filled by runts) | ~2.10 ms *(partly `in_maint`)* | **1.21 ms (0.77%)** | Unowned; recoverable via runt-exempt slots without growing `ready_bytes` |
| 3. Block-0 `in_wait` (`4 MiB` initial `CHUNK` in `src/read.rs:13`) | 1.40 ms | **1.35 ms (0.86%)** | Unowned; one-shot per-stream startup, recoverable via smaller first read |
| 4. OS thread spawn (`t_first_scan_start`) | 1.22 ms | 1.22 ms (0.77%) | Irreducible per `StreamDecoder::new` (amortizes on larger streams) |
| 5. Indivisible-block variance & drain tail (`slots = ∞, 0-start`) | ~1.11 ms *(+2.15 from `in_maint`)* | **2.19 ms (1.39%)** | Irreducible queueing floor (`0.7` fill/drain + `0.6` bimodal + `0.9` variance) |
| 6. Handoff / wakeup / lock / trace residual (`Real − Sim`) | 1.15 ms | ~1.0–1.15 ms (0.7%) | Closed (`dead-ends.tsv` row 9; matches `S`/`SC` 1.12–1.16 ms) |
| **Total coupling gap** | **9.13 ms (4.67%)** | **~7.5 ms (4.8%)** | **Combined unowned ceiling: ~2.5 ms (1.6%) < 2.5–3% threshold** |

**Decision:** Do not open a new `idea` claim. The 8-worker coupling gap is NOT a hidden >= 3% lever: `scanner-seam-window` (#857) already captures the ~2.15 ms `in_maint` coupling amplification on top of its ~10–11 ms direct scan win, leaving ~4.5 ms of irreducible tandem-queue/startup/handoff floor and only two small hygiene tweaks (~0.8% each, ~1.6% combined at 150 ms) that can be folded into `delivery-l3-working-set` (if runt-exempt slots help decouple slot count from `READY_BYTES`) or a minor feeder cleanup if ever warranted. Note also a direct interaction with `delivery-l3-working-set`: tightening `SLOTS_PER_THREAD` below 4 on `wikidata` would worsen HOL `slot_wait` at `k = 8` unless runts are excluded from the slot count, whereas tightening `READY_BYTES` (currently never hit at `k = 8`, `hwm = 6.0 MiB`) can bound `k = 9–10` L3 residency without hurting `k = 8`.
