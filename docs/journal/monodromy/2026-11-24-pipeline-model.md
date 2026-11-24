# Record 123: is the pipeline model the right one for the scanner's shift point? (channel #31, item 2)

*MONODROMY, 24 November 2026. A modelling answer: everything here is argued unless marked. Item 1 (the targets) is taken as read: acquisition and closure stay the obstacles, aimed at the emission ceiling.*

## 1. Verdict

The pipeline model is the right skeleton, but **fitting it to the end-to-end curve is not the ideal method.** The knee of that curve is where the model is least reliable and the data least informative. Measure each stage's capacity directly and let the model only combine them.

## 2. The model, stated

Let S be the scanner's capacity in blocks a second with the workers replaced by a sink, and W(k) the capacity of k workers with scanner output supplied. The fluid model says throughput X(k) = min(S, W(k)).

- **W(k) is arm C of the four-arm run,** at each k. **S is a new oracle:** scanner alone.
- With these two curves measured, nothing is fitted. The residual X(k) − min(S, W(k)) is itself the measure of what the model misses.

## 3. What the fluid model misses

1. **The scanner's capacity depends on k.** The i9-10900K has 10 cores. At 10 workers the scanner is the eleventh thread and shares a core with a worker. So S falls exactly at the point where the 19% was measured. Arm C removes the scanner thread and frees that core as well, so part of C's gain may be the hyperthread, not the serial limit. Measure S(k): scanner alone, with k threads of worker load running.
2. **Workers are teams, not k servers.** Capacity is a step function of the plan (teams × threads), and the plan changes with the thread count. Fit per plan, not per k.
3. **Hyperthread pairs and turbo.** Past 10 threads each added thread is a sibling, worth well under a core, and the clock falls as more cores are active. W is concave with a break at 10. A smooth law fitted across the break will misplace the crossing.
4. **Memory.** Each table is 3.6 MB against a 20 MiB shared cache. Queued symbol buffers live in the same cache, so a deeper queue lowers W. The queue bound is a parameter of W, not a free smoothing knob.
5. **Variance across blocks.** With a bounded queue and unequal block times the kink becomes a rounded knee: near balance, throughput sits below min(S, W) by an amount of order cv²/B, for B the queue bound and cv the variation of block times. Far from balance the loss vanishes.
6. **The second scanner is not free.** It costs a thread, so the comparison at thread budget T is (1 scanner, T−1) against (2, T−2). It is also not twice S: it must find block starts and may waste a parse.
7. **Reassembly and output** are a third serial stage. If it is close to S, a second scanner moves the bottleneck there.

## 4. The estimator

The decision is discrete, so estimate the crossing, not a curve.

- **Shift point:** k* is the least k with W(k−1) > S(k). One scanner already limits the line even after a worker is given up for the second.
- **Uncertainty (delta method):** SE(k*) ≈ SE(ln S − ln W) / |d ln W / dk| at the crossing. With a slope of about 5% a worker near 8–10, pinning k* to ±1 needs the ratio S/W known to about ±2%.
- **Soft form, if a single curve is wanted:** X = (S^(−p) + W^(−p))^(−1/p). Large p is the fluid model; p is the knee's roundness and absorbs item 5. For W alone, the universal scalability law (Gunther) with a separate branch past 10 threads.

## 5. Grid and rounds

Every count from 1 to 12 at equal depth spends most of the budget far from the crossing.

1. **Coarse pass:** S alone; W and X at 2, 4, 6, 8, 10, 12, 16, 20; three rounds each.
2. **Dense pass:** the five counts around the estimated crossing, plus both sides of the 10-thread break whether or not the crossing is there. Arms interleaved in ABBA order within a round, so drift and heat cancel.
3. **Rounds:** n ≥ 2·(σ/0.02)² per point, σ being one round's relative spread. If σ is 3%, that is 5 rounds; at 5%, 13.
4. **Blocks:** report per block as well as pooled. If block times vary by more than a quarter, the knee is wide and ±1 worker is not a meaningful precision; say so instead of reporting it.

## 6. Prior art

Bottleneck analysis of a tandem line is operational analysis (Denning and Buzen 1978). Finite-buffer tandem lines with blocking are standard queueing theory. The contention law is Gunther's. Nothing here is new.

## 7. Rule, fixed before data

If the measured X(k) departs from min(S(k), W(k)) by more than 5% at three or more counts away from the crossing, the two-stage model is wrong and item 7 or item 4 is the first suspect.

---
*Corrected by record 126 (`2026-11-24-corrections-118-125.md`) after independent review.*
