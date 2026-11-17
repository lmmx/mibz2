# 2026-11-17 · MONODROMY 49 · The low sweep scored, and the worker-count rule frozen as a spec. Record 46's CPU budget is right at 8, 6 and 4 threads. Record 44 has the right counts but misses four intervals. Record 48's homogeneity test fails at W = 3. The rule: W\* from min(C, W_L3, 0.80 N, W_ser), with the kernel by contention, not by count. One untested combination is flagged: eager at a trimmed count

Asked by the Coordinator (10:37), after `workers-sweep-low/run-20261117T102836Z`:
- *the bundle:* on `evidence` at 479a56d2; its SHA-256 7d3af59a…fa30, rechecked here;
- *the scoring LOG:* on #145.

Two tasks:
1. score record 44 and record 48's LLC-miss test;
2. consolidate the rule as the spec for a machine-parameterised plan, frozen, with its transfer
   test still to come.

**Labels:** **C** counted · **M** modelled · **Conj**.

## 1. Scores (C)

**Record 46, primary (ι = 0.125 N): right on every frozen point.**

| threads | best W | wall, predicted / measured | worker busy share at T workers (thread split), predicted / measured |
|---|---|---|---|
| 8 | 6 ✓ | 6: 0.940 / 0.925 · 5: 1.002 / 0.997 | 0.77 / 0.742 |
| 6 | 5 ✓ | 5 against lazy 6: 0.946 / 0.925 · 4: 1.044 / 1.036 | 0.76 / 0.750 |
| 4 | 4 ✓ | 3: 1.019 / 1.014 · 2: 1.42 / 1.396 | 0.75 / 0.769 |

- *The rival (ι = 1.25 fixed):* refuted. It predicted shares of 0.56 and 0.67 at 4 and 6 threads,
  and W\* = 3 at 4.
- *Its one failure stands:* at -3 it has no serial term (record 48).

**Record 44: the right counts, four of eleven intervals missed.**

| threads | quantity | predicted | measured | |
|---|---|---|---|---|
| 8 | 5 against 8 | 0.951 [0.92, 0.99] | 0.997 | ✗ |
| 8 | 5 against 6 | 1.02 [0.99, 1.06] | 1.078 | ✗ |
| 6 | lazy 5 against lazy 6 | 0.97 [0.94, 1.00] | 0.925 | ✗, a larger gain |
| 6 | lazy 5 against production | 1.00 [0.95, 1.05] | 0.962 | ✓ |
| 6 | lazy 4 against lazy 6 | 1.01 [0.97, 1.05] | 1.036 | ✓ |
| 6 | lazy 4 against production | 1.04 [0.99, 1.09] | 1.077 | ✓ |
| 4 | lazy 3 against lazy 4 | 1.04 [1.00, 1.08] | 1.014 | ✓ |
| 4 | lazy 3 against production | 1.13 [1.07, 1.18] | 1.113 | ✓ |
| 4 | lazy 4 against production | 1.09 [1.04, 1.13] | 1.098 | ✓ |
| 4 | lazy 2 against lazy 4 | 1.26 [1.15, 1.35] | 1.396 | ✗ |
| 4 | lazy 2 against production | 1.37 [1.25, 1.47] | 1.533 | ✗ |

- *Its busy-share check:* refuted. It predicted near saturation (d ≤ 0.05) at T workers, but the
  share is 0.74–0.77 at every T. That is record 46's ι ∝ N.
- *The verdict:* its 0.65 calibration of gains was a patch on the wrong busy term.

**Record 48's homogeneity test.**
- *Counted where:* `counters-4.txt` has **no LLC misses** (only cycles, ref-cycles, instructions,
  L2 misses and task-clock). Only the 6-thread group carries them, at W = 6, 5, 4 and 3, pinned
  to 6 CPUs. So W = 2 is not counted.

| -9 at W | predicted | measured | |
|--:|---|--:|---|
| 4 | 0.0045 [0.0035, 0.0058] | 0.00368 | ✓ |
| 3 | 0.0034 [0.0027, 0.0042] | **0.00225** | ✗, 34% low |

- *The local exponent:* 1.70 between 3 and 4, and 2.69 between 4 and 5. It steepens, where
  homogeneity with a convex miss curve said it should flatten (1.0–1.4).
- **So contention is not a function of x = WF/L alone.** At equal x, -3 (more workers, smaller
  footprints) misses more than -9 (fewer workers, larger).
- *A power fit over both sizes:* misses ∝ W^1.55 F^1.38, rms 15% in log. That is poor: the
  misses have a floor (-3 at W = 4 is +35% off the fit).
- *What survives:* k ≈ 2 from the 6–9 slopes (record 48), as an empirical exponent over that
  range. It is not a scaling law.
- *Conj:* the number of workers running at once matters beyond the footprint they occupy. That is
  record 47's concurrency reading, which the counters run tests.

## 2. The rule, frozen as the spec (M)

**Inputs, all read at runtime.**
- C: physical cores in the affinity.
- N: the **logical** CPUs allowed. On SMT, siblings enter through each worker's cycles, not
  through N. This corrects record 46 §3, which counted them in core-equivalents: that would put
  the CI runner at W = 2, against 4–9% measured gains from 2 to 4 there.
- The L3 domains {(C_d, L_d)}: `index3/shared_cpu_list` over the affinity.
- s: the block size, from the stream header.

**The constants,** calibrated on the i9, Wikidata, lazy Cells, rustc 1.99.1:

| constant | value | source |
|---|---|---|
| b₀, a worker's busy share with CPUs to spare | 0.93 | #515 thread samples; low sweep 0.929–0.979 |
| ι, idle CPUs when fully subscribed | 0.125 N | i9 at N = 10 (1.25), runner at N = 4 (≈ 0.5), low sweep's shares at N = 4, 6, 8 |
| ν, non-worker CPUs | ≈ 0.16 W (0.42–0.60 cores per GB/s of output) | low sweep's thread split, #515 |
| the L3 constant | W_L3 = 0.8 · L_d / F(s), with F(900 k) = 2.1 MB a lazy worker | records 45 and 48 |
| the serial ceiling R_ser | about 2.5 GB/s at -3; above 2.14 GB/s at -9 | record 48 |

**The count.**
1. Compute m = min( C, Σ_d min(C_d, W_L3,d), **0.80 · N**, W_ser ).
   - *0.80 N:* the solution of 0.93 W = N − 0.16 W − 0.125 N.
   - *W_ser:* the least W with 0.93 W × clock ÷ c(W) at R_ser.
2. Take W\* from {⌊m⌋, ⌈m⌉}, whichever has the lower predicted wall c(W) ÷ min(0.93 W, N − ν − ι).
   Within 1%, take the larger.

**The rule's results on the i9, all as measured:**

| N | m | W\* | measured |
|---|---|---|---|
| 4 | 3.2 | 4 | 4 |
| 6 | 4.8 | 5 | 5 |
| 8 | 6.4 | 6 | 6 |
| 10 | 7.9 (the L3 binds) | 7 | 7 |
| 12–20 (C = 10) | 7.9 | 7 | 7 |
| 10 at -3 | about 7 (the serial stage binds) | 7 | 7 |

**The kernel: lazy where the contention product is large.** In absolute cycles a byte at the same W:
- *Eager, at W = 4 and 6:* cheaper by 1.30 and 0.86.
- *Lazy:* wins at 9–10 (the memory lever).
- *The gap:* fits −1.67 + 0.030 · W · W_run cycles. So lazy wins where **W · W_run ≥ 56**: about
  W ≥ 8 at -9 on the i9, scaling as W_lazy ≈ 8 · (L / 20 MiB) · (F(900 k) / F(s)).
- *Today's code:* the threshold is a count, LAZY_FROM_THREADS = 8, which this matches at T workers.

**The flag, and why the plan must not ship blind.**
- *What the count rule would select at 6 threads:* W = 5, and `Plan::Workers(5)` runs **eager**
  Cells today. Eager at a trimmed count has never been measured. The measured winner is
  **lazy** 5 (0.962). The Researcher flagged the same mechanism for 7 (REVIEW 6943617731).
- *The kernel model predicts eager wins at the trimmed counts:*
  - eager 5 at 6 threads: **0.90 [0.86, 0.95]** against production, better than lazy 5's 0.962;
  - eager 3 at 4 threads: **0.99 [0.95, 1.03]**;
  - eager 7 against lazy 7 at 10 threads: **0.98 [0.95, 1.02]**.
- *Until a run decides:*
  - **The spec's safe default:** the count from the rule, and the kernel as measured. That is lazy
    at any trimmed count from 6 threads up, and eager W = T at 4.
  - **The proposed run:** eager at 5 (6 threads), 3 (4 threads) and 7 (10 threads), against the
    lazy arms already measured.
  - **If eager 5 does not beat lazy 5:** the kernel cost model (Δa = 1.67, Δb = 0.030) is wrong,
    and the kernel stays chosen by measurement.

## 3. Before it ships off the i9

**The transfer test** is the CI runner, record 45's three refuters: lazy W = 2, 3 and 4, and
production. This rule gives m = 3.2 there, so W\* = 4 with 3 within 2%. That agrees with record
45.

**Constants that may not transfer:**
1. b₀ and ι/N (scheduler properties);
2. ν per GB/s (the serial stages' speed relative to the workers');
3. R_ser.

A one-off calibration command (#522's fallback) would measure 2 and 3 on a new machine with one
pinned run and its thread split.

**The constants refit only with the kernel:** F(s), Δa and Δb.
