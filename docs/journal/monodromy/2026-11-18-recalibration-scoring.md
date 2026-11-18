# 2026-11-18 · MONODROMY 58 · Record 56 scored against the recalibration sweep: every resolvable cell holds but one, missed by 0.2%, and the 10+-thread cells are unresolved. The bimodal regime is outside the worker model: a chip-wide clock state with memory of recent load. Heavy runs after a rest are slow 2% of the time; after heavy runs, 72%

**The request.** The Coordinator's CHECKPOINT 6978889765.
- *The bundle:* `recalibrate/run-20261118T122744Z`, on `evidence` at 2b244991. Its SHA-256 is
  c9ae47ee…7e77, rechecked here.
- *Context:* LOG 6978881191.

**Labels:** **C** counted (here, from the bundle's per-run records) · **M** · **Conj**.

## 1. Record 56 scored (Wikidata ×4, wall against the shipped trim)

**Pinned** (24 rounds at 8 threads, 12 elsewhere):

| threads | predicted best | measured | the arms, predicted → measured |
|---|---|---|---|
| 8 | 6 | **6 ✓** | 7: 1.029 [0.96, 1.09] → 1.040 ✓ · 8: 1.10 [1.02, 1.18] → **1.050 ✓** · 5: 1.092 [1.05, 1.13] → 1.132 ✗ (0.002 above the top) |
| 6 | 5 | **5 ✓** | 4 eager: 1.035 [0.97, 1.10] → 1.033 ✓ · 6 eager: 1.07 [1.03, 1.10] → 1.073 ✓ |
| 4 | 3 | **3 ✓** | 4 eager: 1.04 [1.01, 1.08] → 1.045 ✓ · 2: 1.35 [1.30, 1.39] → 1.366 ✓ |
| 10 | 8 | unresolved (bimodal) | 8: 0.982 → 0.860 [0.70, 1.05] · 6: 1.054 → 1.051 · 10: 1.115 → 1.032; every interval about ±15–20% |

**Pinned 8, specifically.** 8/32 over 6/24 came out 1.050 SLOWER, which the model predicted.
- *Why:* the CPU budget (record 49), 0.80 × 8 = 6.4 CPUs for workers.
- *Inside its interval,* below its point (1.10). The model overstates the cost of the extra
  workers there.
- *The 5-worker arm:* missed by 0.002, the model's only clear miss. It understates the cost of
  removing a worker below the budget: 1.132 against 1.092.

**Unpinned:**

| threads | predicted best | measured | the arms |
|---|---|---|---|
| 4 | 4 | **4 ✓** | 3: 1.21 [1.17, 1.25] → 1.225 ✓. 5 was not run. |
| 6 | 6 (eager) | eager 6 against eager 5 ✓ | 5 eager: 1.01 [0.98, 1.04] → 1.037 ✓ (bimodal: one slow run) |
| 6 | (none for lazy) | **lazy 6/24 at 0.940 FASTER** | **a miss of the kernel model** (below) |
| 8 | 8 | unresolved | 7: 1.022 → 1.062 [0.89, 1.27] |
| 10 / 12 / 16 / 20 | 8 | unresolved | 8: 0.979 → 0.880 / 0.929 / 0.927 / 0.969 · 10: 1.043 → 1.004 / 0.980 / 0.922 / 0.943 (all INCONCLUSIVE) |

**The record's refuters can't be read.**
- *8 slower than 7 by more than 1%:* every point is below 1.
- *Untrimmed 10 beating 7:* the points at 16 and 20 lean that way, but nothing is significant.
- *Wide admission's 8-over-7 shift at 10 or more threads* stays untested at the resolution
  needed. The RSS growth would hold it under rule 4 anyway.

**The kernel miss (C).** Record 49's kernel gap was fitted on master's lazy kernel: lazy wins where
W · W_run ≥ 56, so eager is better at 6 workers. That predicted eager 6 beats lazy 6 unpinned.
Wide lazy is 6% faster there.
- *The reason:* wide admission cut the lazy kernel's per-byte cost, so its constants (Δa = 1.67,
  Δb = 0.030) are stale.
- *The rough refit:* the kernel threshold falls to about W ≥ 6 for wide lazy. That is moot for
  the default, since the memory gate holds it (20.7% RSS).

## 2. The bimodal regime: outside the worker model; a clock state with memory (C)

**What the per-run fields show.** These are the cells with at least about 7.7 CPUs busy:
- *pinned:* 10 threads;
- *unpinned:* 8 to 20 threads.

Splitting each cell's runs at the midpoint of min and max wall:

| | fast | slow | slow ÷ fast |
|---|--:|--:|--:|
| wall | 0.32–0.36 s | 0.45–0.49 s | about 1.40 |
| total CPU | | | the same ratio |
| parallelism (CPU ÷ wall) | | | **unchanged**, 7.72 → 7.78 at 7/28 |
| workers' CPU | | | **1.42–1.45×** |
| scanner thread's CPU | | | **1.42–1.54×** |
| main's CPU | | | 1.2–1.4× |
| pre_busy, load1, waited_s, others_busy | | | no difference |

**A chip-wide slowdown, not a memory one.** The scanner is compute-bound and light on memory, and it
slows as much as the workers.
- *Memory contention* would slow the workers more.
- *External load* would change the parallelism or the pre-run fields.
- *What's left:* a per-core slowdown hitting every thread equally. That is a clock-frequency drop
  of about 1.4×, roughly 4.9 → 3.5 GHz.

**The state has memory: it depends on the runs before (C).** Each run is classed by the
CPU-seconds of heavy runs (≥ 7 CPUs busy) among the three runs just before it:

| run | after a rest | after at least 2.5 heavy CPU-seconds in the last three runs |
|---|--:|--:|
| heavy, unpinned | **1 / 65 slow (2%)** | **74 / 103 slow (72%)** |
| light, unpinned | 67 / 402 (17%) | 79 / 186 (42%) |
| heavy, pinned | 1 / 41 (2%) | 16 / 28 (57%) |
| light, pinned | 15 / 556 (3%) | 23 / 95 (24%) |

- *The time-ordered trace:* the first heavy Wikidata run of a round is fast, and the next heavy
  run is slow.
- *Light runs too:* text and random runs that *follow* heavy ones are slowed, and the state
  clears after a stretch of light runs.
- *Why it scatters:* the arm order rotates by round, so each arm lands in the slow position at
  random. That gives the ±15–20% intervals.
- *Below the threshold:* pinned 8 (at most 8 CPUs) and the eager 4- and 6-thread cells never
  build enough history, and stay tight.

**Reading:** a package power budget. That fits the i9-10900K's PL1/PL2 turbo limits, enforced on an
exponentially weighted power average with a time constant (Tau) of tens of seconds. Throttling
would also fit (Conj). Either way:
- *with the 0.25 s settle gate,* the average stays above PL1 through a heavy round and the chip
  clamps;
- *with the 2 s gate* (the admit run), it recovers between runs.

The coordinator's hypothesis is **supported** by the per-run fields, with one refinement: the
trigger is cumulative recent load, not the run's own load.

**Does the worker model have such a term? No.** Records 43–53 take a fixed clock (cycles ÷ ref-cycles
flat at 1.326 in #515) and deterministic walls for a given W. Nothing in them produces history
dependence. It is outside the model.

## 3. What follows

1. **For benches:** the 2 s gate until a power-aware gate exists. Better: gate on RAPL `energy-pkg`
   falling below a power threshold, and record per run:
   - the package energy;
   - the effective clock (cycles ÷ ref-cycles, or aperf/mperf).

   Then each run's clock state is *measured*, and runs can be stratified by it instead of discarded.
2. **For the plan (Conj, worth a measurement).** A long decode, such as a real dump of minutes,
   runs in the *clamped* state, but every W\* so far was measured in the turbo state.
   - *Why it could differ:* under a package power cap, throughput tracks the power budget ÷ energy
     per byte, not cores × clock. Contention stalls and active-core overheads both raise energy
     per byte.
   - *The test:* a sustained-load run (tens of seconds a cell, no gate) at 6, 7, 8 and 10 workers,
     with RAPL, for the clamped state's W\*.
   - *Its prediction (M, weak):* the clamped optimum sits at or below the turbo one, because
     each worker's slice of the budget falls as W rises.
3. **The model needs a clock term** for the clamped state: f(P_cap, W), from RAPL. It can't be
   fitted from this bundle, which has neither cycles nor energy.
