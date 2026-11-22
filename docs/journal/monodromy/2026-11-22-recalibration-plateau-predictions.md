# 2026-11-22 · MONODROMY 91 · Record 89 scored against the stack recalibration: 22 of 26 ratios, 10 of 11 outcomes, not refuted. Frozen before bench-recalibrate-plateau: #739 alone moves nothing. With #745's faster scanner the knee moves up, and at 10–20 threads unpinned 9 and 8 should beat 7 by 6–8% and 10 by about 4%. Wormhole's "10 wins" is predicted wrong in its optimum, right in its direction

**The request.** The Coordinator's CHECKPOINT 7077232695 (RELEASING §5c.3): the contention model's
prediction for `bench-recalibrate-plateau`.
- *Its cells:* #695's design, at master with #739 (3296195e) and possibly #745.
- *Frozen:* **before any of its data.**

**Labels:** **C** counted · **M** modelled · **N** no basis (extrapolated or never measured).

**Evidence** (SHA-256 checked against the stored digests):
- `recalibrate-stack/run-20261122T030603Z` (88d062e0…);
- `wormhole-plateau/run-20261122T085156Z` (9faac67b…);
- the 058 and diet runs, used for pooling.

## 1. Record 89 scored against the stack run (C)

**The totals.**
- *Ratios:* 22 of 26 inside their intervals. The misses are pinned 10's 6/7 (1.117) and 8/7
  (0.896), pinned 8's 8/6 (1.069) and unpinned 16's 8/7 (0.922).
- *Rule outcomes:* 10 of 11. The miss is the one predicted move, pinned 4's lazy 3/12. It measured
  **0.982 [0.966, 0.998] FASTER**, inside its predicted 0.97 [0.92, 1.02], but Holm did not reject:
  p 0.0145 against a first threshold of 0.025/12 ≈ 0.0021. So eager 3 stayed.
- *Refutation conditions:* none triggered. Pinned 4 lay in [0.97, 1.03], the band record 89 charged
  to its no-basis g(3), not to the model.

## 2. What moved, and how it enters (M)

**#739, the plateau stack** (`wormhole-plateau/run-20261122T085156Z`, the default decoder, W = 6,
7 and 8 workers):

| W | wall | CPU |
|--:|--:|--:|
| 6 | 0.944 | 0.942 |
| 7 (mean at 10, 12, 16 and 20 threads) | 0.960 | 0.93 |
| 8 | 0.973 | 0.911 |

It applies to every arm, lazy and eager, because the output path, RLE1, CRC and buffers are the
whole decoder's. The wall gain falls slowly with W: the front-end cap of record 88.

**#745, the block reader.** It cuts the serial scanner's per-byte cost by about 19%.
- *Against the old base:* 0.977 at 1 thread, 0.932 at 10 threads (W = 7) and 0.884 at 8 threads
  (W = 8).
- *Its gain grows with W.* That is the signature of relieving a serial cap.
- *Above 8 workers (N):* the gain is extrapolated, bounded below by the scanner's own 0.81. I take
  g₇₄₅(9) ≈ 0.85 and g₇₄₅(10) ≈ 0.83.

**The method.** Each count ratio is a pooled base times g(W)/g(ref) for each change.
- *Cells measured precisely:* the base is the stack run.
- *At 10 threads and up:* each of 058, diet and stack had intervals near ±20%, so the base is the
  pool of all their readings. The model has no thread-count dependence from 10 to 20.
  - *8/7:* 0.975 across 12 readings.
  - *10/7:* 1.031 across 12 readings.
  - *6/7:* 1.017 across 3 readings at 10 threads.
  - *9/7:* never measured. 0.985 by interpolation (N).

**Two scenarios,** since #745 is still being re-qualified:
- **A:** #739 only.
- **B:** #739 and #745.

## 3. The predictions (wall, Wikidata ×4, candidate over the shipped reference; quotable verbatim)

| cell | reference | candidate | A: #739 only | B: #739 + #745 |
|---|---|---|---|---|
| unpinned 10, 12, 16, 20 | lazy 7/28 | lazy 6/24 (at 10 threads only) | 1.00 [0.95, 1.05] | 1.01 [0.96, 1.06] |
| | | **lazy 8/32** | **0.99 [0.95, 1.03]** | **0.94 [0.89, 0.99]** |
| | | **lazy 9/36** (N) | 1.01 [0.95, 1.07] | **0.92 [0.86, 0.98]** |
| | | **lazy 10/S** (N) | 1.07 [1.01, 1.13] | 0.96 [0.90, 1.02] |
| pinned 10 | lazy 7/28 | lazy 6/24 | 1.09 [1.04, 1.14] | 1.10 [1.05, 1.15] |
| | | lazy 8/32 | 0.97 [0.92, 1.02] | **0.92 [0.86, 0.98]** |
| | | lazy 10/40 (N) | 1.15 [1.08, 1.22] | 1.02 [0.95, 1.09] |
| pinned 8 | lazy 6/24 | lazy 5/20 | 1.15 [1.13, 1.17] | 1.15 [1.13, 1.17] |
| | | lazy 7/28 | 1.09 [1.07, 1.11] | 1.08 [1.06, 1.10] |
| | | lazy 8/32 | 1.10 [1.08, 1.12] | 1.04 [1.01, 1.07] |
| pinned 6 | lazy 5/20 | lazy 4/16 | 1.12 [1.10, 1.15] | 1.13 [1.10, 1.15] |
| | | lazy 6/24 | 1.08 [1.06, 1.10] | 1.08 [1.06, 1.10] |
| | | eager 6/24 | 1.29 [1.25, 1.33] | 1.29 [1.25, 1.33] |
| pinned 4 | eager 3/12 | eager 2/8 | 1.35 [1.32, 1.38] | 1.35 [1.32, 1.38] |
| | | eager 4/16 | 1.05 [1.03, 1.07] | 1.05 [1.03, 1.07] |
| | | **lazy 3/12** | **0.98 [0.96, 1.00]** | **0.98 [0.96, 1.00]** |
| unpinned 4 | lazy 4/16 | lazy 3/12 | 1.30 [1.26, 1.34] | 1.30 [1.26, 1.34] |
| | | eager 4/16 | 1.11 [1.07, 1.15] | 1.11 [1.07, 1.15] |
| unpinned 6 | lazy 6/24 | lazy 5/20 | 1.22 [1.10, 1.34] | 1.21 [1.09, 1.33] |
| | | eager 6/24 | 1.34 [1.20, 1.48] | 1.34 [1.20, 1.48] |
| unpinned 8 | lazy 8/32 | lazy 7/28 | 1.01 [0.95, 1.07] | 1.07 [1.01, 1.13] |

**The best count at each thread count:**

| threads, placement | A: #739 only | B: #739 + #745 |
|---|---|---|
| 10–20 unpinned | 7 or 8 (a tie) | **9**, with 8 close behind |
| 10 pinned | 8 (weakly) | **8** |
| 8 pinned | 6 | 6 |
| 6 pinned | lazy 5 | lazy 5 |
| 4 pinned | lazy 3 | lazy 3 |
| 4 unpinned | lazy 4 | lazy 4 |
| 6 unpinned | lazy 6 | lazy 6 |
| 8 unpinned | 8 | 8 |

## 4. Which shipped trims I expect to change

**Under A (#739 only): none.** At 10–20 threads 8/7 is 0.99, a tie.

**Under B (#739 + #745), the contention trims at 10 threads and up (lazy 7/28) are the only
candidates.**
- *By wall:* 9/36 and 8/32 lead, 6–8% ahead of 7.
- *What decides adoption:* memory and Holm, not wall.
  - **8/32:** read 1.09–1.11 of 7/28's memory in the last three runs, so it sits on the 1.10 gate.
    #739's fixed 12 MiB ready cap should lower that ratio, but by how much is N. If it passes, and
    the noise at 10 threads and up is as low as the plateau run's (±0.6–3.5%, against ±20% before),
    **8/32 should adopt at 10–20 unpinned and at pinned 10.**
  - **9/36 and 10/S:** expected to be held on memory. 10/S read 1.25–1.32 before.

**Pinned 4's lazy 3/12** stays about 2% faster, as in the stack run. It adopts only if Holm can
reject at this noise. I expect it not to (p about 0.01), unless the plateau changes tighten the
4-thread cells.

**Wormhole's hypothesis** is that 10 workers with the 12 MiB cap win from 10 to 20 threads.
- *Under B:* 10/S ≈ 0.96, faster than 7 but slower than 9 and 8. Its memory fails the gate.
- *Under A:* 10 loses, at 1.07.

So the model predicts the direction right, toward more workers, but the optimum at 9, not 10.

## 5. How it will be scored

**Score the scenario the build actually is.**

**The model is refuted if any of these holds:**
1. **Under B,** at 10–20 threads unpinned, 10/S beats both 8/32 and 9/36 by more than 2% on the
   pooled cells. That refutes the cap model's interior optimum.
2. **Under A,** any pooled 8/7 at 10 threads and up falls below 0.94. That would mean #739 alone
   moved the knee.
3. **At pinned 8,** a neighbour beats 6.
4. **Under either scenario,** pinned 6's or unpinned 4's or 6's best count differs from the table.

**A miss confined to an N arm** (9/36, or 10/S's extrapolated gain) scores the extrapolation of
g₇₄₅ above W = 8, not the model.
