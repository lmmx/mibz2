# 2026-11-21 · MONODROMY 88 · Record 87 scored against 0.5.8's run: 17 of 26 ratios, 6 of 11 best counts, 10 of 11 rule outcomes. It is refuted at 10 threads and up, where #668 moved the knee toward fewer workers. Frozen before bench-recalibrate-diet: the diet cuts contention to 0.73×, which moves the knee back. No trim should move. Unpinned 20's 8/32 is the cell most likely to

**The request.** The Coordinator's CHECKPOINT 7062657163 (RELEASING §5c.3).
- *The recipe:* `bench-recalibrate-diet` (#695 at d62890fb, master 23e40f77).
- *Its arms:* those of #695's `2026-11-21-recalibration-diet-registration.md`. Every window is
  4 blocks a worker, as registered.
- *Frozen:* **before any of its data.**

**Labels:** **C** counted · **M** modelled · **N** no basis.

**Script:** `recaldiet.py`, beside this record as `.txt`.

**Evidence** (SHA-256 checked against the stored digests):
- `recalibrate-058/run-20261121T020723Z` (bed46d52…);
- `open-diet/run-20261121T121714Z` (a0675fb1…).

## 1. Record 87 scored against 0.5.8's run (C)

Each figure is the wall ratio on Wikidata ×4, candidate over reference.

| cell | arm | predicted | measured | |
|---|---|---|---|---|
| pinned 10 (ref 7) | 6 / 8 / 10 | 1.12 / **0.98 [0.94, 1.02]** / 1.11 | 1.057 / **1.024** / 1.164 | hit / **miss** / hit |
| pinned 8 (ref 6) | 5 / 7 / 8 | 1.14 / 1.05 / 1.05 | 1.120 / 1.081 / 1.079 | miss / miss / miss |
| pinned 6 (ref lazy 5) | lazy 4 / lazy 6 / eager 6 | 1.01 (N) / 1.07 / 1.21 | 1.127 / 1.074 / 1.211 | **miss** / hit / hit |
| pinned 4 (ref eager 3) | eager 2 / eager 4 / lazy 3 | 1.35 / 1.03 / 1.07 | 1.350 / 1.055 / 1.033 | hit / miss / hit |
| unpinned 4 (ref eager 4) | eager 3 / lazy 4 | 1.22 / **1.02 (N)** | 1.185 / **0.945** | hit / **miss: ADOPTED** |
| unpinned 6 (ref lazy 6) | lazy 5 / eager 6 | 1.11 / 1.24 | 1.139 / 1.263 | hit / hit |
| unpinned 8 (ref 8) | 7 | 1.00 | 0.977 | hit |
| unpinned 10 (ref 7) | 6 / 8 / 10 | 1.08 / **0.98 [0.93, 1.03]** / 1.09 | 1.029 / **1.036** / 1.088 | miss / **miss** / hit |
| unpinned 12 / 16 / 20 | 8 | 0.98 [0.89, 1.08] | 1.034 / 1.037 / 0.973 | hit / hit / hit |
| | 10/S | 1.05 [0.93, 1.18] | 1.078 / 1.140 / 1.069 | hit / hit / hit |

**The totals.**
- *Ratios:* 17 of 26 hit.
- *Best counts by wall:* 6 of 11. The misses are pinned 10, unpinned 10, 12 and 16 (8 predicted,
  7 measured) and unpinned 4.
- *Rule outcomes:* 10 of 11. The miss is unpinned 4's lazy 4/16 ADOPT (#678), a near-tie record 87
  flagged, resting on a no-basis input.

**Refutation conditions (record 87 §5).**
1. *An outcome outside the flagged near-ties:* not triggered.
2. *8/7 outside [0.93, 1.03] at 10 threads and up:* **triggered,** at unpinned 10, 12 and 16
   (1.036, 1.034 and 1.037). Each is a point estimate whose interval spans the bound.
3. *Pinned 8:* not triggered.

**The diagnosis.** #668's faster workers moved the knee toward fewer workers: 8/7 went from
0.97–0.98 in 0.5.7 to 1.02–1.04. Pinned 8's neighbours lost by about 8%, not 5%. My ν·g
correction gave this the right sign but about a fifth of its size. A front end that is a fixed
serial cap, not a per-worker load ν·W, would bind sooner as workers get faster; that is the
likely form. **Not refitted here.**

## 2. The model for the diet (M)

**The diet's adoption run:** the default decoder, unpinned, at 6, 10 and 8 threads, so W = 6, 7
and 8 workers:

| W | wall | CPU |
|--:|--:|--:|
| 6 | 0.952 | 0.950 |
| 7 | 0.931 | 0.922 |
| 8 | 0.917 | 0.870 |

**The gain grows with W.** That is the signature of reduced contention: each worker's scratch
fell from 4,236 to 1,200 KiB (0.283×).

**The fit.** g(W) = a · (1 + β′ W C(W)) / (1 + β W C(W)), on the unpinned form, with β = 0.0124
from record 87:
- **a = 1.033** and **β′ = 0.0091 (0.73 β)**;
- the residual RMS is 0.0014 on the three points.

**The predictions.** Each is 0.5.8's measured ratio times g(W)/g(ref). For pinned cells, the same
proportion is applied to the pinned β.
- *Kernel arms:* eager is unchanged, and lazy moves by g.
- *No basis (N):* g at W ≤ 5. The diet was measured only at W = 6–8, and below that the fit
  extrapolates (g(3) = 1.006, g(4) = 0.989, g(5) = 0.970).
- *Intervals:* 0.5.8's 95% interval, re-centred, widened by ±0.01. N arms by ±0.03 more.

## 3. The predictions for bench-recalibrate-diet (wall, Wikidata ×4, candidate over the shipped reference)

| cell | reference | candidates: predicted [interval] | best by wall | rule outcome |
|---|---|---|---|---|
| pinned 10 | lazy 7/28 | lazy 6/24 1.07 [1.04, 1.10]; lazy 8/32 **1.01 [0.93, 1.09]**; lazy 10/40 1.14 [1.02, 1.27] | 7 (8 a near-tie) | 7 stays |
| pinned 8 | lazy 6/24 | lazy 5/20 1.13 [1.12, 1.15]; lazy 7/28 1.07 [1.06, 1.09]; lazy 8/32 1.07 [1.05, 1.08] | 6 | 6 stays |
| pinned 6 | lazy 5/20 | lazy 4/16 1.14 [1.11, 1.17]; lazy 6/24 1.07 [1.05, 1.09]; eager 6/24 1.21 [1.18, 1.24] | 5 | lazy 5 stays |
| pinned 4 | eager 3/12 | eager 2/8 1.35 [1.32, 1.38]; eager 4/16 1.055 [1.03, 1.08]; lazy 3/12 **1.05 [1.00, 1.10]** (N) | eager 3 | eager 3 stays |
| unpinned 4 | lazy 4/16 | lazy 3/12 1.21 [1.13, 1.28] (N); eager 4/16 1.07 [1.03, 1.11] (N) | lazy 4 | lazy 4 stays |
| unpinned 6 | lazy 6/24 | lazy 5/20 1.16 [1.08, 1.25]; eager 6/24 1.33 [1.26, 1.40] | lazy 6 | lazy 6 stays |
| unpinned 8 | lazy 8/32 | lazy 7/28 0.995 [0.89, 1.11] | tie | 8 stays |
| unpinned 10 | lazy 7/28 | lazy 6/24 1.05 [0.96, 1.14]; lazy 8/32 **1.02 [0.91, 1.14]**; lazy 10/40 1.05 [0.94, 1.17] | 7 (near-tie with 8) | 7 stays |
| unpinned 12 | lazy 7/28 | lazy 8/32 1.015 [0.93, 1.10]; lazy 10/48 1.04 [1.00, 1.08] | 7 | 7 stays |
| unpinned 16 | lazy 7/28 | lazy 8/32 1.02 [0.98, 1.06]; lazy 10/64 1.10 [0.99, 1.21] | 7 | 7 stays |
| unpinned 20 | lazy 7/28 | lazy 8/32 **0.956 [0.88, 1.03]**; lazy 10/80 1.03 [0.92, 1.15] | **8** | 7 stays: Holm unlikely to reject at this width; memory about 1.06, within the gate |

## 4. Which trims I expect to move

**None, but the contention trims sit on the edge.**
- **At 10 threads and up (lazy 7/28):** the diet's lower contention takes back about 1.8% of
  #668's shift. 8/7 lands at 1.01–1.02, a near-tie, and 10/7 at 1.03–1.14.
- **Unpinned 20 is the only cell where 8 is predicted faster,** at 0.956. 0.5.8 read 0.973 there
  too. Its memory ratio (1.06) passes the gate, so an ADOPT there would be a hit for the
  direction, not a miss.
- **Pinned 4's lazy 3/12:** I predict 1.05, so eager 3 stays. The diet's gain at 3 workers is
  extrapolated (N). If the uniform part a is really below 1, which only W ≥ 6 constrains, lazy 3
  could reach a tie.
- **Unpinned 4 (now lazy 4/16), pinned 8 and the 6-thread trims:** they should not move.

## 5. How it will be scored

**Per cell:** the best count by wall, the rule outcome, and the ratio inside its interval.

**The model is refuted for the diet if any of these holds:**
1. A rule outcome other than the table's, except an ADOPT of 8/32 at unpinned 20.
2. Any 8/7 at 10 threads and up below 0.93 or above 1.10.
3. At pinned 8, a neighbour beats 6.

**A miss confined to an N arm** (pinned 4's lazy 3/12, unpinned 4's arms) scores the
extrapolation of g below W = 6, not the model.
