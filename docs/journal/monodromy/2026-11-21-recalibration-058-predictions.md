# 2026-11-21 · MONODROMY 87 · Record 83 scored: 16 of 20 ratios inside, 7 of 9 best counts, and refuted at pinned 8, where my recalibrated budget term was wrong. Frozen before bench-recalibrate-058: no trim should change. At 10 threads and up, 8 beats 7 by about 2% but stays memory-held; pinned 8 stays at 6; the lazy 6-thread trims stay; two arms are near-ties (lazy 4/16 unpinned at 4, lazy 4/16 pinned at 6)

**The request.** The Coordinator's CHECKPOINT 7055160250 (RELEASING §5c.3).
- *The recipe:* `bench-recalibrate-058` (#671, ready tag at c695b930, guarding master 70b18ca4).
- *Its cells and arms:* those of #671's `2026-11-21-recalibration-058-registration.md`.
- *Frozen:* **before any of its data.**

**Labels:** **C** counted (an earlier run) · **M** modelled · **N** no basis.

**Script:** `recal058.py`, beside this record as `.txt`.

**Evidence used.** 0.5.7's run, `recalibrate-057/run-20261119T185157Z` on `evidence`. Its SHA-256,
98200b84…3805, matches the stored digest.

## 1. Record 83 scored against 0.5.7's run (C)

Each figure is the wall ratio on Wikidata ×4, candidate over reference. A hit means inside record
83's interval.

| cell | arm | predicted | measured | |
|---|---|---|---|---|
| pinned 10 (ref 7) | 6 | 1.06 [1.02, 1.10] | 1.116 [1.044, 1.193] | **miss** |
| | 8 | 0.97 [0.93, 1.01] | 0.973 [0.947, 1.000] | hit |
| | 10 | 1.05 [1.00, 1.10] | 1.097 [1.031, 1.169] | hit |
| pinned 8 (ref 6) | 5 | 1.10 [1.06, 1.14] | 1.144 [1.141, 1.148] | **miss** (just above) |
| | 7 | **0.96 [0.92, 1.00]** | **1.043 [1.037, 1.048]** | **miss** |
| | 8 | 1.01 [0.95, 1.05] | 1.047 [1.042, 1.052] | hit |
| pinned 6 (ref eager 5) | eager 4 | 1.035 [0.97, 1.10] | 1.027 | hit |
| | eager 6 | 1.07 [1.03, 1.10] | 1.065 | hit |
| pinned 4 (ref eager 3) | eager 2 | 1.35 [1.30, 1.39] | 1.346 | hit |
| | eager 4 | 1.04 [1.01, 1.08] | 1.027 | hit |
| unpinned 8 (ref 8) | 7 | 1.026 [0.99, 1.06] | 1.003 | hit |
| unpinned 10 (ref 7) | 6 / 8 / 10 | 1.05 / 0.975 / 1.05 | 1.084 / 0.981 / 1.083 | hit / hit / hit |
| unpinned 12 | 8 / 10 | 0.975 / 1.05 | 0.956 / 1.010 | hit / hit |
| unpinned 16 | 8 / 10 | 0.975 / 1.05 | **1.038** / 1.090 | **miss** / hit |
| unpinned 20 | 8 / 10 | 0.975 / 1.05 | 0.964 / 1.021 | hit / hit |

**The totals.**
- *Ratios:* 16 of 20 hit.
- *Best counts:* 7 of 9 hit. The misses are pinned 8, predicted 7 against 6 measured, and
  unpinned 16, predicted 8 against 7 (point estimate only; its interval spans 1).
- *Not predicted:* the "other kernel" arms (lazy 5/20 pinned at 6, lazy 3/12 pinned at 4) and
  the unpinned 6 and 4 cells.

**Refutation conditions (record 83 §4).**
1. *8 slower than 7 by over 1% at 10 threads and up:* triggered on the point estimate at unpinned
   16 only (1.038 [0.956, 1.127], spanning 1).
2. *Untrimmed 10 beating 7:* not triggered.
3. ***7 slower than 6 by over 1% at pinned 8:*** **triggered, at 1.043.**

**So record 83's pinned budget term is refuted.** I had refitted ν to 0.05 W from 0.5.5's
crossover point (0.983 at 8 threads). That point was measured with one block a thread, not the
untrimmed arm's 4 blocks, and it did not carry over. Refitted on 0.5.7's own pinned cells, ν is
about 0.2 W, close to record 56's original 0.18 W. The contention calls at 10 threads and up held.

## 2. The model, refitted on 0.5.7 (M)

**The same form** as records 46–56 and 83. Each placement is fitted by least squares in log ratio
over 0.5.7's lazy count arms:

| placement | β | ν | RMS log error | cells |
|---|--:|--:|--:|--:|
| pinned | 0.0060 | 0.205 W | 0.021 | 6 (at 10 and 8) |
| unpinned | 0.0124 | 0.210 W | 0.032 | 10 (at 8–20) |

**#668's change enters as a uniform per-worker speed-up g.** It is 1/0.947 at 10 threads and up,
1/0.974 at 8 and 1/0.922 at 6 (the open-vec16 run).
- *Why it matters:* the front end's share then rises, so ν becomes ν·g.
- *The effect on the count ratios:* at most 1.2%, at 10 and 10/7. So **0.5.8's predictions are
  0.5.7's measured ratios moved by the model's ν·g correction.** The knee does not move.
- *The intervals:* 0.5.7's own 95% interval, widened by ±0.01 for the correction.

**The arms the model doesn't reach:**
- *The new lazy count arms at 6 threads:* 0.5.7's eager count ratios at 6, with the same
  correction. That assumes lazy and eager share the count shape there (**N**, widened by ±0.03).
- *The kernel-switch arms:* 0.5.7's measured kernel ratio, times #668's lazy gain, which I take
  as 0.92–0.97 at 4 and 6 threads. Only the 6-thread figure (0.922) was measured, so the 4-thread
  gain is **N**.

## 3. The predictions for bench-recalibrate-058 (wall, Wikidata ×4, candidate over the shipped reference)

| cell | reference | candidate: predicted [interval] | predicted best by wall | predicted rule outcome |
|---|---|---|---|---|
| pinned 10 | lazy 7/28 | lazy 6/24: 1.12 [1.04, 1.20]; lazy 8/32: **0.98 [0.94, 1.02]**; lazy 10/40: 1.11 [1.03, 1.18] | 8 | 7 stays (8: no Holm gain likely; memory about 1.12) |
| pinned 8 | lazy 6/24 | lazy 5/20: 1.14 [1.13, 1.16]; lazy 7/28: 1.05 [1.03, 1.06]; lazy 8/32: 1.05 [1.04, 1.07] | **6** | 6 stays |
| pinned 6 | lazy 5/20 | lazy 4/16: 1.01 [0.97, 1.05] (**N**); lazy 6/24: 1.07 [1.03, 1.11] (**N**); eager 6/24: 1.21 [1.15, 1.28] | 5 (4 a near-tie) | lazy 5 stays |
| pinned 4 | eager 3/12 | eager 2/8: 1.35 [1.32, 1.38]; eager 4/16: 1.03 [1.01, 1.05]; lazy 3/12: 1.07 [1.03, 1.11] (**N**) | eager 3 | eager 3 stays |
| unpinned 4 | eager 4/16 | eager 3/12: 1.22 [1.15, 1.29]; lazy 4/16: **1.02 [0.97, 1.06]** (**N**) | eager 4 (lazy 4 a near-tie) | eager 4 stays |
| unpinned 6 | lazy 6/24 | lazy 5/20: 1.11 [0.98, 1.26] (**N**); eager 6/24: 1.24 [1.15, 1.34] | lazy 6 | lazy 6 stays |
| unpinned 8 | lazy 8/32 | lazy 7/28: 1.00 [0.92, 1.09] | tie | 8 stays |
| unpinned 10 | lazy 7/28 | lazy 6/24: 1.08 [1.03, 1.14]; lazy 8/32: **0.98 [0.93, 1.03]**; lazy 10/40: 1.09 [1.00, 1.18] | 8 | 7 stays |
| unpinned 12 / 16 / 20 | lazy 7/28 | lazy 8/32: **0.98 [0.89, 1.08]**; lazy 10/S: 1.05 [0.93, 1.18] | 8 (weakly) | 7 stays |

**On 12–20 threads:** the model has no thread-count dependence there. 0.5.7's three readings
(0.956, 1.038 and 0.964) differ by noise inside one interval, so I predict one value for all
three.

## 4. What it says

**No trim should change in 0.5.8.**
- *At 10 threads and up:* 8 should stay about 2% faster than 7 by wall, as in 0.5.7. Its memory
  cost of 8–13% keeps it from adopting by rule, as the registration itself expects.
- *Pinned 8:* 6 stays best. Both neighbours are about 5% slower: the CPU-budget term, now
  correctly fitted.
- *The lazy 6-thread trims (pinned 5, unpinned 6):* they stay. The eager arm should lose by more
  than in 0.5.7, about 21–24%, because #668 made the lazy kernel faster.

**The two near-ties to watch:**
- *Lazy 4/16 against eager 4/16, unpinned at 4,* at 1.02 [0.97, 1.06]. 0.5.7 read 1.073, and
  #668's lazy gain closes most of that.
- *Lazy 4/16 against lazy 5/20, pinned at 6,* at 1.01.

Both rest on no-basis inputs: the lazy gain at 4 threads, and the lazy count shape at 6.

## 5. How it will be scored

**Per cell:** the best-count hit by wall, and the ratio hit inside its interval.

**The model is refuted for 0.5.8's kernel if any of these holds:**
1. A count's rule outcome differs from the table, outside the two flagged near-ties.
2. At 10 threads and up, any 8/7 ratio lies outside [0.93, 1.03].
3. At pinned 8, 7/6 or 8/6 falls below 1.00. That would refute the refitted budget term in the
   other direction.

**A miss confined to an N arm** (the 6-thread lazy count arms, or the 4-thread kernel arms)
scores that input, not the model.
