# 2026-11-19 · MONODROMY 83 · Frozen before the 0.5.7 worker-count recalibration (OpenCells as the lazy route's kernel, master 17405826): the best count is 8 at 10 or more threads, pinned and unpinned, and 7 pinned at 8. It stays 8 unpinned at 8, and at the shipped eager plans at 6 and 4

**The request.** The Coordinator's CHECKPOINT 7018307521 (RELEASING §5c.3). The registered
`bench-recalibrate` design (2026-11-18), re-pinned to 17405826. Frozen **before any of its data.**
- *Cells:* pinned 4, 6, 8 and 10; unpinned 8, 10, 12, 16 and 20.
- *Arms in each cell:* the shipped trim, −1, +1, and untrimmed.

**Labels:** **C** counted (earlier runs) · **M** modelled · **N** no basis (model only, never measured
in that regime).

The script is `recal057.py`, beside this record as `.txt`.

## 1. What changed since record 56, and the model's inputs

**The kernel.** #630 makes `OpenCells` the lazy route's kernel. It runs the open kernel on
LazyCells-admitted blocks and LazyCells elsewhere. The eager route (`Cells`) is unchanged, so the
eager cells (pinned 4 and 6) are unchanged.

**The shipped trims (master, `trim_cell`), unchanged since 0.5.4:**
- *Pinned:* lazy 7 at 10, lazy 6 at 8, eager 5 at 6, eager 3 at 4.
- *Unpinned:* lazy 7 from 10 threads; untrimmed below (lazy 8 at 8).
- *The untrimmed plan:* 10 lazy workers at 10–20 threads.
- *The ±1 arms* keep the shipped arm's kernel.

**What the open kernel does to the knee: little.**
- *Record 76:* opend's gain is 24% fewer instructions at lower IPC, **flat in the worker count**,
  and DRAM is not involved. A uniform per-worker speed-up doesn't move the knee.
- *Open-port-3:* about 6% less wall at 8, 10 and 20 threads.
- *How it enters:* the front end's share of the CPU rises by about 6%, so the non-worker load ν
  is scaled by 1.06.
- *β, the wide kernel's contention per worker pair (record 56):* kept, 0.0116 [0.0088, 0.0150].

**The model (records 46–56).**
- *Wall:* wall ∝ c(W) / CPU(W), with c = 1 + β · W · CPU(W).
- *Pinned:* CPU = min(0.93 W, N − 0.125 N − ν W).
- *Unpinned:* CPU = min(W · u(W), 10 − ν W), with u = 0.941 − 0.0103 (W − 7).

**ν is recalibrated on the two wide-kernel worker-count points measured since record 56.** Both
come from 0.5.5's crossover run (LOG 6977656986), whose sanity cells show about ±2.5% harness
noise:
- *Pinned 8:* 8 workers / shipped 6 = **0.983 [0.955, 1.011]**. Record 56 predicted 1.10 here: a
  **miss**. Its pinned budget term (ν = 0.18 W) was too heavy for the wide kernel. This point
  alone gives ν ≈ 0.05 W.
- *10 threads:* 10 workers / shipped 7 = **1.047**. That fits ν = 0.105 W pinned or 0.219 W
  unpinned.
- *Below:* each cell is predicted at its own placement's ν (× 1.06). The interval spans both fits
  and the β range, ±0.02 (about record 49's worst model miss).

## 2. The predictions (wall against the shipped trim, Wikidata)

| threads, placement | shipped | **predicted best** | fewer (−1) | more (+1) | untrimmed |
|---|---|---|---|---|---|
| 10, pinned | lazy 7 | **8** | 6: 1.06 [1.02, 1.10] | 8: **0.97 [0.93, 1.01]** | 10: 1.05 [1.00, 1.10] |
| 8, pinned | lazy 6 | **7** | 5: 1.10 [1.06, 1.14] | 7: **0.96 [0.92, 1.00]** | 8: 1.01 [0.95, 1.05] (C: 0.983) |
| 6, pinned | eager 5 | **5** | 4: 1.035 [0.97, 1.10] (M) | 6 = untrimmed | 6: **1.07 [1.03, 1.10]** (C: 1/0.938) |
| 4, pinned | eager 3 | **3** | 2: 1.35 [1.30, 1.39] (M) | 4 = untrimmed | 4: **1.04 [1.01, 1.08]** (C: 1/0.959) |
| 8, unpinned | lazy 8 | **8** | 7: 1.026 [0.99, 1.06] | 9: 1.016 [0.98, 1.05] (**N**) | = shipped |
| 10, unpinned | lazy 7 | **8** | 6: 1.05 [1.01, 1.09] | 8: **0.975 [0.94, 1.01]** | 10: 1.05 [1.00, 1.10] (C: 1.047) |
| 12 / 16 / 20, unpinned | lazy 7 | **8** | 6: 1.05 [1.01, 1.09] | 8: **0.975 [0.94, 1.01]** | 10: 1.05 [1.00, 1.10] |

## 3. What it says, and where it has no basis

**At 10 or more threads, the trim should move from 7 to 8** in both placements, by about 2.5–3%.
This is record 56's call, unchanged: the open kernel is uniform in W and doesn't move it. Both
intervals reach 1.0, so a tie is inside them.

**Pinned 8 should move from 6 to 7.** This is new since record 56: 0.5.5's crossover showed the
wide kernel's 8-thread budget is looser than record 56 assumed. Untrimmed 8 should still be
about level with 6.

**Unpinned 8 stays at 8.** 7 and 9 are both predicted about 1–3% slower, so near a tie.

**The eager cells (pinned 6 and 4) stay at 5 and 3.** Nothing in 0.5.7 touches them.

**No thread-count dependence from 10 to 20 unpinned:** the model sees 10 cores and the same plan.

**No basis (N):**
1. *More workers than threads asked, unpinned 9 at 8.* This was never measured.
2. *The unpinned ν:* it rests on one crossover point (1.047), which was measured with one block a
   thread, not the untrimmed plan's window. The pinned and unpinned fits differ by a factor of 2.
3. *The eager kernel's ±1 arms (M):* carried from record 56, with master's lazy β.

## 4. How it will be scored

**Per cell:**
- the best-count hit;
- the ratio hit, inside its interval.

**The model is refuted for the open kernel if any of these holds:**
1. At 10 or more threads, in either placement, 8 is slower than 7 by more than 1% (the
   interval's top).
2. At 10 or more threads unpinned, untrimmed 10 beats 7 by more than 1%.
3. At pinned 8, 7 is slower than 6 by more than 1%. This refutes the recalibrated budget term,
   not the contention term.

**A miss confined to unpinned 8's 9-worker arm** is in the no-basis set and does not refute.
