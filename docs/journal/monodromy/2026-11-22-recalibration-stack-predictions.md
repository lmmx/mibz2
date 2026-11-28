# 2026-11-22 · MONODROMY 89 · Record 88 scored against the diet recalibration: 11 of 11 rule outcomes, 18 of 26 ratios. Condition 2 tripped only on two high-thread cells too noisy to inform. Frozen before bench-recalibrate-stack: one trim should move, pinned 4 to lazy 3/12. The contention trims at 10 threads and up stay as near-ties

**The request.** The Coordinator's CHECKPOINT 7073098344 (RELEASING §5c.3).
- *The recipe:* `bench-recalibrate-stack`, #695's design pinned to master 53f7ecf9 (#719, the
  Open construction stack, with #714).
- *Its cells and arms:* #695's table, with every window at 4 blocks a worker.
- *Frozen:* **before any of its data.**

**Labels:** **C** counted · **M** modelled · **N** no basis.

**Script:** `recalstack.py`, beside this record as `.txt`.

**Evidence** (SHA-256 checked against the stored digests):
- `recalibrate-diet/run-20261121T124719Z` (643e6a32…);
- `open-stack/run-20261122T022924Z` (50c832fd…).

## 1. Record 88 scored against the diet run (C)

Each figure is the wall ratio on Wikidata ×4, candidate over reference.

| cell | arms | predicted | measured | hits |
|---|---|---|---|---|
| pinned 10 (ref 7) | 6 / 8 / 10 | 1.07 / 1.01 [0.93, 1.09] / 1.14 | 1.163 / **0.913** / 1.096 | 1 of 3 |
| pinned 8 (ref 6) | 5 / 7 / 8 | 1.13 / 1.07 / 1.07 | 1.149 / 1.066 / 1.065 | 3 of 3 |
| pinned 6 (ref lazy 5) | lazy 4 / lazy 6 / eager 6 | 1.14 / 1.07 / 1.21 | 1.154 / 1.080 / 1.253 | 2 of 3 |
| pinned 4 (ref eager 3) | eager 2 / eager 4 / lazy 3 | 1.35 / 1.055 / 1.05 (N) | 1.334 / 1.040 / 1.020 | 3 of 3 |
| unpinned 4 (ref lazy 4) | lazy 3 / eager 4 | 1.21 (N) / 1.07 (N) | 1.300 / 1.025 | 0 of 2 |
| unpinned 6 (ref lazy 6) | lazy 5 / eager 6 | 1.16 / 1.33 | 1.221 / 1.273 | 2 of 2 |
| unpinned 8 (ref 8) | 7 | 0.995 | 1.065 | 1 of 1 |
| unpinned 10 (ref 7) | 6 / 8 / 10 | 1.05 / 1.02 [0.91, 1.14] / 1.05 | 0.977 / **0.889** / 1.007 | 2 of 3 |
| unpinned 12 / 16 / 20 | 8 | 1.015 / 1.02 / 0.956 | 0.935 / 0.986 / 1.029 | 3 of 3 |
| | 10/S | 1.04 / 1.10 / 1.03 | 0.999 / 0.925 / 1.033 | 1 of 3 |

**The totals.**
- *Ratios:* 18 of 26 hit.
- *Best counts by wall:* 6 of 11.
- ***Rule outcomes: 11 of 11.*** No trim moved, as predicted.

**Refutation conditions (record 88 §5).**
1. *An outcome other than the table's:* not triggered.
2. *8/7 at 10 threads and up outside [0.93, 1.10]:* **triggered on point estimates only,** at
   pinned 10 (0.913 [0.738, 1.129]) and unpinned 10 (0.889 [0.737, 1.073]).
3. *A neighbour beating 6 at pinned 8:* not triggered.

**Condition 2 carries little information here.** This run's high-thread cells were noisy, with
intervals of ±17–25%, about three times 0.5.8's. Both readings lie inside their own wide
intervals, and they point toward 8, the direction the diet's lower contention predicts. I count
it as a formal trip and an uninformative one.

**The precise cells all hit:** pinned 8, pinned 6 and pinned 4, whose intervals are about ±1%.

## 2. The model for the stack (M)

**The stack's adoption run:** the default decoder, unpinned, at W = 6, 7 and 8 workers (6, 10
and 8 threads):

| W | wall | CPU |
|--:|--:|--:|
| 6 | 0.952 | 0.958 |
| 7 | 0.975 | 0.949 |
| 8 | 0.987 | 0.962 |

**A uniform saving, capped.** The CPU saving is uniform, about 5% (the native Open scratch fell
19%). The wall gain shrinks as W grows. That is the serial front-end cap diagnosed in record 88: a
uniform per-worker saving shows fully only where the front end does not bind.

**So the lazy arms move by g(W):**
- *Where measured:* g = 0.952, 0.975 and 0.987 at W = 6, 7 and 8.
- *At W ≤ 5:* the CPU saving itself, **0.95 [0.94, 0.96] (N)**. The cap binds less there; this
  is not measured.
- *At W = 10:* **1.00 (N)**, fully capped.
- *Eager arms:* unchanged.

**The base for each cell:**
- *Below 10 threads:* the diet run's measured ratio.
- *At 10 threads and up:* the inverse-variance mean of record 88's prediction and the diet run's
  noisy reading.

**The intervals:** the base's, widened by ±0.01, and by ±0.02 more for N arms.

## 3. The predictions for bench-recalibrate-stack (wall, Wikidata ×4, candidate over the shipped reference)

| cell | reference | candidates: predicted [interval] | best by wall | rule outcome |
|---|---|---|---|---|
| pinned 10 | lazy 7/28 | 6/24 1.05 [1.01, 1.09]; 8/32 **1.01 [0.92, 1.09]**; 10/40 1.16 [1.02, 1.29] (N) | 7 (8 a near-tie) | 7 stays |
| pinned 8 | lazy 6/24 | 5/20 1.15 [1.11, 1.18] (N); 7/28 1.09 [1.07, 1.11]; 8/32 1.10 [1.09, 1.12] | 6 | 6 stays |
| pinned 6 | lazy 5/20 | lazy 4/16 1.15 [1.12, 1.19] (N); lazy 6/24 1.08 [1.04, 1.13] (N); eager 6/24 1.32 [1.28, 1.36] (N) | lazy 5 | lazy 5 stays |
| **pinned 4** | **eager 3/12** | eager 2/8 1.33 [1.30, 1.37]; eager 4/16 1.04 [1.01, 1.07]; **lazy 3/12 0.97 [0.92, 1.02] (N)** | **lazy 3** | **lazy 3/12 ADOPTS** (memory about 0.89) |
| unpinned 4 | lazy 4/16 | lazy 3/12 1.30 [1.23, 1.37] (N); eager 4/16 1.08 [1.02, 1.14] (N) | lazy 4 | lazy 4 stays |
| unpinned 6 | lazy 6/24 | lazy 5/20 1.22 [0.99, 1.45] (N); eager 6/24 1.34 [1.17, 1.50] | lazy 6 | lazy 6 stays |
| unpinned 8 | lazy 8/32 | lazy 7/28 0.99 [0.88, 1.10] | tie | 8 stays |
| unpinned 10 | lazy 7/28 | 6/24 1.01 [0.92, 1.10]; 8/32 **0.99 [0.89, 1.10]**; 10/40 1.07 [0.94, 1.19] (N) | tie | 7 stays |
| unpinned 12 | lazy 7/28 | 8/32 1.01 [0.93, 1.10]; 10/48 1.07 [1.00, 1.13] (N) | 7 | 7 stays |
| unpinned 16 | lazy 7/28 | 8/32 1.03 [0.98, 1.08]; 10/64 1.08 [0.96, 1.21] (N) | 7 | 7 stays |
| unpinned 20 | lazy 7/28 | 8/32 0.97 [0.89, 1.06]; 10/80 1.06 [0.93, 1.19] (N) | 8 (weakly) | 7 stays (no Holm gain likely at this noise) |

## 4. Which trims I expect to move

**One: pinned 4, from eager 3/12 to lazy 3/12.**
- *The base:* the diet run measured lazy 3/12 at 1.020 [1.004, 1.037]. That is a tight interval:
  3 workers, pinned, with little noise.
- *The stack's effect:* it is a lazy-kernel saving of about 5% CPU. At 3 workers the front-end cap
  should not bind, so the full saving shows and gives 0.97.
- *The rule:* at that precision Holm would reject, and memory (0.89) passes the gate.
- *What it rests on:* g(3) is no-basis. If the cap does bind at 3 workers, g(3) is nearer W = 7's
  0.975 and the arm lands at about 0.995: a tie, with eager 3 staying.

**Every other trim should stay.**
- *Pinned 8:* its neighbours lose by more than before (1.09–1.10), because the stack's capped gain
  favours fewer workers.
- *At 10 threads and up:* 8/7 sits at 0.97–1.03, within noise of a tie.

## 5. How it will be scored

**Per cell:** the best count by wall, the rule outcome, and the ratio inside its interval.

**The model is refuted for the stack if any of these holds:**
1. A rule outcome differs from the table at any cell other than pinned 4 or unpinned 20.
2. Pinned 4's lazy 3/12 reads above 1.03. That would refute "the saving shows fully at small W".
3. At pinned 8, a neighbour beats 6.

**A miss at pinned 4 within [0.97, 1.03]** scores g(3), an N input, not the model.
