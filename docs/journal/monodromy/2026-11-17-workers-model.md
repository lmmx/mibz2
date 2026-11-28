# 2026-11-17 · MONODROMY 43 · The simplest model that fits 10 / 9 / 8 / 7 / 6 workers: cycles a byte c(W) = a + bW², with contention growing like W², plus a residual of 3–4% a worker that cycles don't carry. Predictions, before data: 7 workers best at 10 threads; 4 (or 5) on a 6-core, 12 MB machine; and the counters that refute each

Asked by the Coordinator (07:48) after bench-workers-low (LOG 6942020517) refuted both models
(records 41 and 42) on their counter shapes:
- *model A (capacity cliff):* the LLC misses did not flatten below the cliff;
- *model B (contention):* the counters did not stay flat.

**Labels:** **C** counted (the runs' figures) · **M** modelled · **Conj** conjecture.

## 1. The data (C)

Against 8 workers, per byte. Walls are at 12 to 20 threads; the counters at 12.

| W | wall | cycles | LLC misses | IPC |
|--:|--:|--:|--:|--:|
| 10 | 1.099 (#481 × #510) | | | |
| 9 | 1.050 to 1.06 | 1.131 | 1.125 | 1.19 |
| 8 | 1 | 1 | 1 | 1.35 |
| 7 | 0.972 | 0.881 | 0.827 | 1.53 |
| 6 | 0.998 | 0.766 | 0.593 | 1.71 |

## 2. The model (M)

**Cycles a byte are quadratic in W.** Least squares on the four counted points gives

  **c(W) = 0.480 + 0.00808 W²** (relative to c(8)),

which fits within 0.6%: 0.771, 0.876, 0.997 and 1.134 against 0.766, 0.881, 1 and 1.131.
- *Reading:* a fixed cost a byte, plus a contention cost a byte that grows like W². That is
  consistent with a miss rate that rises with the number of workers sharing the L3 (LLC misses
  fall smoothly as W falls), times a miss cost that also rises with W. That second factor is Conj.

**But walls are not c(W)/W.**
- *What c(W)/W predicts:* nearly flat walls, 1.031 at 6, 1.004 at 7, 1.011 at 9, 1.033 at 10.
- *What was measured:* steeper. The residual r(W) = wall ÷ (c(W)/W) is 1.064 at 10, 1.039 at 9,
  1 at 8, 0.968 at 7 and 0.968 at 6.
- *So:* above 7, each worker removed gains another 3–4% that the cycle count does not carry.
  Below 7 the residual stops falling.
- *What it can be (Conj):* the clock (fewer active cores, higher turbo, so cycles turn faster),
  or the workers' busy fraction (a starved worker's idle cycles are not counted).

**A direct wall fit** absorbs the residual: wall(W) = p/W + qW with p = 3.080 and q = 0.0783.
- *Fit:* within 1.6% at all five points (1.091, 1.047, 1.011, 0.988, 0.983 against 1.099,
  1.050, 1, 0.972, 0.998).
- *Its least:* W* = √(p/q) = **6.3**.
- *The cycles model alone:* gives 7.7.
- *Measured:* W* = 7, between them.

## 3. Predictions, before any data (M)

### At 10 threads, within the cores

**The setting.** The same i9, with exactly 10 threads: W workers plus the scanner and the consumer on
10 cores. The worker contention terms are the same as past the cores, so the models carry over.

| W | wall against 10 workers, cycles model (c/W) | wall against 10 workers, wall fit |
|--:|--:|--:|
| 9 | 0.979 | 0.960 |
| 8 | 0.968 | 0.927 |
| 7 | 0.972 | 0.906 |
| 6 | 0.998 | 0.901 |

**The prediction:** **W\* = 7** at 10 threads, with wall(7) / wall(10) = **0.93, in [0.90, 0.98]**,
and 8 within 2% of 7.
- *A further reason 10 should do worse:* 10 workers plus the scanner and consumer oversubscribe 10
  cores, which this model doesn't charge. So the true ratio should sit nearer the interval's low
  end.

### On a 6-core machine with a 12 MB L3 (2 MB a core, as on the i9)

**The assumption.** Contention scales with W against the L3's size, as a capacity effect does:
W_eff = W · 20/12.

| W | wall, cycles model (relative units) | wall, wall fit |
|--:|--:|--:|
| 3 | 0.227 | 1.68 |
| 4 | **0.210** | **1.64** |
| 5 | **0.208** | 1.70 |
| 6 | 0.215 | 1.82 |

**The prediction:** **W\* = 4, or 5 within 2%.** Six workers is **3 to 11% slower** than 4.

## 4. What refutes it

1. **Cycles a byte off the curve.** If c(W) at a new configuration lies more than 5% off
   0.480 + 0.00808 W², with W scaled by L3 size on another machine, the quadratic form is wrong.
2. **The residual is not frequency or busy fraction.** Count `cycles` against `ref-cycles` (the
   effective clock) and each worker's busy share at 10 / 9 / 8 / 7. If neither moves by the
   residual's 3–4% a worker, the residual is something the model does not name, and the wall fit
   is only curve-fitting.
3. **The 6-core scaling.** If its best W is 5 or 6, not 4, the shared resource is not one sized
   per core (L3 or ring). It is something absolute, such as the memory controller's bandwidth,
   which a 6-core shares as a 10-core does. Then W* tracks total bandwidth, not cores.
   - *The discriminating counter:* LLC misses a byte at the 6-core's W = 4.
   - *If they match the i9's at W ≈ 6.7:* capacity scaling holds.
   - *If they match the i9's at W = 4:* it is refuted.

**An observation, not a lever:** in both arms of bench-w12-stores the store buffer was full for
13.3% of cycles (LOG 6941872163).
