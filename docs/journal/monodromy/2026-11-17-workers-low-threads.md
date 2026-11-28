# 2026-11-17 · MONODROMY 44 · The residual is the workers' busy share: non-worker threads take about 1.7 cores at 10 threads. Wall ∝ c(W) / (W · busy) fits every count. Predictions, frozen before the 8 / 6 / 4-thread sweep: the best W is 6, 5 and 4 there, not the 0.7 × T rule's 6, 4 and 3

Asked by the Coordinator (08:46), after the at-the-cores run (LOG 6943194948):
- *the clock:* cycles over ref-cycles is flat at 1.326, so it is not the residual;
- *my last interval:* record 43's 0.93 [0.90, 0.98] at 10 threads missed (0.885).

**Labels:** **C** counted · **M** modelled · **Conj** conjecture.

## 1. The residual named (C)

**The identity checked.** At 10 threads the walls should satisfy wall ∝ c(W) / (W · u(W)), where c is
cycles a byte and u the worker's busy share. The u implied by the counted walls and cycles,
normalised to u(10) = 0.90, sits beside the counted busy share:

| W | implied u = c / (W · wall) | counted busy share (an upper bound) |
|--:|--:|--:|
| 10 | 0.90 | 0.90 |
| 9 | 0.98 | 0.99 |
| 8 | 1.07 | 1.06 |
| 7 | 1.09 | 1.08 |
| 6 | 1.09 | 1.10 |

**They agree within 0.01 everywhere,** so the residual is the busy share. Ten workers at 10 threads
stand idle for part of the run while the scanner, the reader and the main thread hold cores. As
cores are freed, busy rises and saturates by W ≈ T − 3.

**The deficit d = 1 − u / u_max** against the free cores h = T − W:

| h | 0 | 1 | 2 | 3 | 4 |
|--:|--:|--:|--:|--:|--:|
| d | 0.18 | 0.10 | 0.036 | 0.018 | 0 |

It fits d ≈ 0.18 · e^(−1.3 h/k), with k ≈ 1.7 cores of non-worker load at 10 threads. That load
scales with throughput, because the scanner's work is per byte.

**Cycles a byte depend on W alone (C).** The past-the-cores and at-the-cores ratios agree (9/8: 1.131
against 1.123; 7/8: 0.881 against 0.874). On W = 6–9 they fit c(W) = 0.423 + 0.00676 W² (relative
to production's 10 at 10 threads), within 0.008.

## 2. The model, and its calibration (M)

**The form.** Wall ∝ c(W) / (W (1 − d(T − W, k))), with k proportional to throughput. Normalised to
each thread count's own T workers.

**Raw, it overstates the gains by about a third.**

| threads | arm | model | counted |
|---|---|--:|--:|
| 10 | 7 workers | 0.823 | 0.885 |
| 8 | 6 workers | 0.889 | 0.929 |

The measured gains are 0.65 of the modelled ones at both counts. I apply that factor to every
predicted gain below. It is the model's calibration on its own two points, stated before the data.

**Against production.** At 8 threads production's plan is lazy workers, so its own arm is the
control. At 6 and 4 threads it is the eager or team plan.
- *The step to production:* lazy at T workers was 1.033 (6 threads) and 1.086 (4 threads) of the
  shipped plan in the memory-lever run (LOG 6903318530).
- *Assumed to hold (M):* the shipped plan may have moved since.

## 3. The predictions, frozen

| threads | best W | arm | against the lazy T-worker control | against production |
|---|---|---|---|---|
| **8** | **6** | 5 workers | 0.951 [0.92, 0.99]; so 5 against 6 is 1.02 [0.99, 1.06] | the same (production is lazy here) |
| **6** | **5** | 5 workers | 0.97 [0.94, 1.00] | 1.00 [0.95, 1.05] |
| | | 4 workers | 1.01 [0.97, 1.05] | 1.04 [0.99, 1.09] |
| **4** | **4** (no trim) | 3 workers | 1.04 [1.00, 1.08] | 1.13 [1.07, 1.18] |
| | | 2 workers | 1.26 [1.15, 1.35] | 1.37 [1.25, 1.47] |
| | | lazy 4-worker control | 1 | **1.09 [1.04, 1.13]**: production stays better |

## 4. What refutes it, and what tells it from the 0.7 × T rule

**The empirical rule so far** (7 of 10, 6 of 8, 7 past the cores) predicts **4 at 6 threads and 3 at 4.**
This model predicts **5 and 4**.

Its reasoning: at lower T fewer workers contend, so c(W)/W keeps falling with W. The non-worker load,
which scales with throughput, takes a smaller share of the cores.

**Refuted if:**
1. at 6 threads, 4 workers beats 5 by more than 2%; or
2. at 4 threads, 3 workers beats 4 by more than 2%.

Either means contention below 6 workers is stronger than c(W)'s quadratic, fitted on 6–9,
extrapolates. The counter that would show it is cycles a byte at W = 3–5, against
0.423 + 0.00676 W².

**Busy-share check.** At 6 and 4 threads with T workers, the counted busy share should be near
saturation (d ≤ 0.05), because the non-worker load is smaller there. If busy is as low as at 10
threads (0.90), the non-worker load does not scale with throughput as assumed.
