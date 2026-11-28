# 2026-11-18 · MONODROMY 56 · Frozen before the 0.5.5 recalibration sweep: with wide admission, the best count moves from 7 to 8 at 10 or more threads, pinned and unpinned. It stays at 6 pinned at 8, at the shipped eager plans at 4 and 6, and at 8 unpinned at 8. Where the model has no basis is marked

**The request.** The Coordinator's CHECKPOINT 6977093320: the Pathfinder's recalibration sweep on
0.5.5 (#559, WideLazyCells). Frozen **before any of its data.**
- *The sweep:* pinned and unpinned at 4, 6, 8 and 10 threads, and unpinned at 12, 16 and 20.
- *The arms:* the shipped trim, one worker fewer, one more, and the untrimmed plan.

**Labels:** **C** counted (earlier runs) · **M** modelled · **N** no basis (model only, never measured
in that regime).

## 1. The model and its inputs

**The wall time,** as records 46, 49, 52 and 53 set it out: wall ∝ c(W) / (worker CPUs), with
c(W) = 1 + β · W · W_run, normalised to the floor.
- *Unpinned:* worker CPUs = W · u, with u = 0.941 − 0.0103 (W − 7). That is the measured unpinned
  per-worker utilisation: 0.941 at 7 workers, 0.910 at 10 (imc-ten).
- *Pinned:* worker CPUs = min(0.93 W, N − ν − 0.125 N), the CPU budget, with ν ≈ 0.18 W cores of
  non-worker load. That is record 49's 0.16 W, raised for 0.5.5's higher throughput.
- *DRAM:* traffic per byte ≈ 4.42 + 0.074 · W · W_run. This is the admit run's 8.05 and 9.16 B at
  7 and 8 workers, against record 52's 6.66 + 0.088x on master. A queueing penalty applies only
  above 80% of B_max(W) = 33.4 − 0.25 (W − 4) GB/s (record 53). **No cell reaches it:** the shipped
  plans run at 52–75%.

**β for the wide kernel: 0.0116 [0.0088, 0.0150].** It is fitted so this form gives the admit run's
Wide8 / Wide10 = 0.979 [0.963, 0.995] unpinned. Master's 1.026 gives 0.0244. **Wide admission halves
the contention-associated overhead per worker pair.**
- *β for the eager kernel:* 0.0244 [0.020, 0.030], master's value. No separate eager fit exists (N).

**Excluded:** the pinned-confounded cells, the low sweep's pinned 6 and 8 as contention evidence.
- *Where they still enter:* only through the pinned CPU-budget term, which *is* the
  front-end scheduling effect bench-unpinned identified. Its constants were fitted on those
  same cells, so the pinned predictions below are calibrated, not independent.

**Unchanged cells.** The eager plans (pinned 4 and 6, unpinned 4 and 6) don't use the lazy kernel,
so 0.5.5 doesn't change them. Where they were measured before, the measured ratio is the
prediction.

**The shipped trims (0.5.5, `trimmed_workers`):**
- *Pinned:* lazy 7/28 at 10, lazy 6/24 at 8, eager 5/20 at 6, eager 3/12 at 4.
- *Unpinned:* lazy 7/28 from 10 threads, otherwise untrimmed (lazy 8 at 8, eager 6 at 6, eager 4
  at 4).
- *Assumed:* the ±1 arms keep the shipped arm's kernel.

## 2. The predictions (wall against the shipped trim, Wikidata ×4)

| threads, placement | shipped | **predicted best** | fewer | more | untrimmed |
|---|---|---|---|---|---|
| 10, pinned | lazy 7 | **8** | 6: 1.054 [1.02, 1.09] | 8: **0.982 [0.94, 1.04]** | 10: 1.115 [1.03, 1.20] |
| 8, pinned | lazy 6 | **6** | 5: 1.092 [1.05, 1.13] | 7: 1.029 [0.96, 1.09] | 8: 1.10 [1.02, 1.18] |
| 6, pinned | eager 5 | **5** | 4: 1.035 [0.97, 1.10] (M) | — | 6: **1.07 [1.03, 1.10]** (C: 1/0.938) |
| 4, pinned | eager 3 | **3** | 2: 1.35 [1.30, 1.39] (M) | — | 4: **1.04 [1.01, 1.08]** (C: 1/0.959) |
| 4, unpinned | eager 4 | **4** (C) or 5 (N) | 3: **1.21 [1.17, 1.25]** (C: 1.214) | 5: 0.94 [0.90, 0.98] (**N**) | = shipped |
| 6, unpinned | eager 6 | **6** | 5: **1.01 [0.98, 1.04]** (C: 1.009) | 7: 1.01 [0.97, 1.04] (**N**) | = shipped |
| 8, unpinned | lazy 8 | **8** | 7: 1.022 [0.99, 1.06] | 9: 1.016 [0.99, 1.04] (**N**) | = shipped |
| 10, unpinned | lazy 7 | **8** | 6: 1.046 [1.01, 1.08] | 8: **0.979 [0.94, 1.01]** | 10: 1.043 [0.99, 1.09] |
| 12 / 16 / 20, unpinned | lazy 7 | **8** | 6: 1.046 [1.01, 1.08] | 8: **0.979 [0.94, 1.01]** | 10: 1.043 [0.99, 1.09] |

**How the intervals were set.**
- *Model cells:* the spread over β and ν (0.14–0.22 W), widened by ±0.02, which is about record
  49's worst model miss.
- *Measured cells (C):* the earlier run's interval, carried over.

## 3. What it says, and where it has no basis

**The trim at 10 or more threads should move from 7 to 8,** with 8 against 7 at about 0.98 both
pinned and unpinned. That is the admit run's knee shift, carried across the thread counts. Both
intervals reach 1.0, so a tie is inside them.
- *Pinned at 10:* 8 sits at the CPU budget's edge, 0.80 N. A miss there points to the budget term,
  not to contention.

**Untrimmed lazy 10 (unpinned) should lose to 7, at about 1.04,** but by much less than on master
(1/0.873 = 1.146).

**Nothing should move at 8 pinned (6), 8 unpinned (8), or the eager cells.**

**The model claims no thread-count dependence from 10 to 20 unpinned.** It sees only N = 20 CPUs
and the same plan. The measured plateau (0.370 / 0.371 / 0.370) supports that.

**No basis (N):**
1. **More workers than threads asked, unpinned below 10:** eager 5 at 4, eager 7 at 6, lazy 9 at 8.
   This was never measured; the model gives contention alone. Its 0.94 for 5 workers at 4 threads
   is the boldest number here. The same model under-predicted 3-against-4 unpinned (1.154 against
   1.214 measured), so it overvalues small worker counts' contention relief. Treat it as weak.
2. **The eager kernel's contention ratio:** master's lazy β is used. The ±1 eager arms at pinned
   6 and 4 (M) carry it.
3. **SMT siblings:** no arm puts a worker on a sibling.
   - *Unpinned at 10–20:* at most 10 workers. Non-workers can share cores.
   - *The busy-share line* that represents this was fitted only from 7 to 10.

## 4. How it will be scored

The per-cell scores are:
- best-count hits;
- ratio hits, inside the interval.

The model is **refuted for the wide kernel if:**
1. at 10 or more threads, unpinned, 8 is slower than 7 by more than 1% (the interval's top); or
2. untrimmed 10 beats 7 there.

Either means β for the wide kernel is not what the admit run implied.
