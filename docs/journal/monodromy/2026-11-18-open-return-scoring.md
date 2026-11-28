# 2026-11-18 · MONODROMY 71 · bench-open-return scored. The adjudicator reruns byte-identically: ADOPTS stands for opend 8/32 at 8 and 10 threads and 7/28 at 10. The gain is not a gate, order or output artefact. A constant ~1–2 ms construction offset in the reference explains about 0.5 points of it. opend moves more DRAM bytes, but bandwidth is not binding; the gain fits a kernel less sensitive to contended latency (I). The test is a worker sweep with stall counters

**The request.** The Coordinator's CHECKPOINT 6992235283.
- *The bundle:* `open-return/run-20261118T210422Z`, evidence fa9707e9, SHA-256 ec8d3254… (checked).
- *The registration:* #590 at 8045ce81.

**Labels:** **C** counted from the bundle · **P** · **I** · **Conj**.

## 1. The adjudicator: VERIFIED, byte-identical (C)

**The rerun.** I ran `bench/two_arm.py` from a worktree at 8045ce81 on `unpinned.jsonl`, with the
recipe's arguments, the three `--pair` reports included. **The output is identical to `verdict.txt`,
byte for byte.**

**The rest of the verdict:**
- *Data:* 324 of 324 records, 12 rounds, the reference's workers checked (7 at 10 threads, 8 at 8),
  0 errors.
- *Holm* rejects all 5 main-corpus cells; the largest p is 4.3e−6.

| arm | threads | Wikidata ×4 | text | random | memory | verdict |
|---|---|---|---|---|---|---|
| opend 8/32 | 10 | **0.919** [0.905, 0.934] | faster | faster | ≤ 1.080 | **ADOPTS (the choice)** |
| opend 8/32 | 8 | **0.944** [0.928, 0.959] | inconclusive | inconclusive | ≤ 1.013 | **ADOPTS (the choice)** |
| opend 7/28 | 10 | **0.940** [0.928, 0.952] | | | ≤ 0.998 | **ADOPTS** |
| open 7/28 | 10 | 0.915 | **1.73× SLOWER** | | | a finding, not adopted |
| open 8/32 | 8 | 0.929 | **1.73× SLOWER** | | | a finding, not adopted |

- *The loss screen passes* for opend: no cell is SLOWER.
- *The memory gate passes:* opend 8/32's text and random peaks are 1.078–1.080 at 10 threads, under
  1.10 but close.

## 2. Gain or artefact?

**The settle gate:** the registered 2 s (console), so record 68 §4's quiet-gap penalty is excluded
(C).
- *Spread:* the largest max/min wall within any cell is 1.069 on Wikidata, 1.105 on text and 1.109
  on random, so there is no bimodality.
- *The gate never waited* (`waited_s` 0).

**Arm order is rotated and has no effect (C).**
- *Positions:* each arm sits in each position 2–3 times at 10 threads and 3 times at 8.
- *Position effects:* 0.996–1.005.
- *Candidate before or after the reference:* 0.909/0.922, 0.941/0.939 and 0.919/0.920 at 10 threads;
  0.926/0.931 and 0.946/0.941 at 8. No order effect.

**Census admission (C).** opend's density line selects exactly the blocks LazyCells admits.

| corpus | run-route blocks | LazyCells admits | opend's line selects | both |
|---|---|---|---|---|
| Wikidata ×4 | 1,532 | 912 | 912 | 912 |
| text | 49 | 0 | 0 | 0 |
| random (36 column-route blocks only) | 0 | 0 | 0 | 0 |

So opend changes only the 912 blocks 0.5.6 already inverts with LazyCells. On text and random it
runs LazyCells as shipped.

**Outputs match (C).** One output hash per corpus across all arms and rounds; every census line has
`same_output: true`; all 324 runs succeeded.

**One artefact, small and quantified (C, I).**
- *The two constructions:*
  - the reference `mibz2` is `BzReader::parallel`, whose `StreamDecoder::new` on the i9 at 10
    threads reads the machine and trims to `workers(7, 28, lazy)`;
  - every candidate arm, `oce` included, is `parallel_workers(_with)`, which builds the same kind of
    worker pool directly.
- *The symptom:* on text and random, where opend runs LazyCells as shipped, every named arm still
  beats the reference by a **constant** amount, not a ratio. At 10 threads that is
  +0.7 to +1.8 ms whether the run takes 0.067 s or 0.120 s; at 8 threads it is 0 to +0.8 ms.
- *What fits (I):* a fixed construction cost in `new`. The reference's main thread uses 0.01 s of CPU,
  against 0.00 for the other arms.
- *Its size on Wikidata:* about 1.5 ms of opend's 18.5 ms (7/28) and 26.1 ms (8/32) gains, so about
  0.5 percentage points.
- *Corrected for it,* the kernel effect is about **0.946 (7/28 at 10), 0.923 (8/32 at 10) and 0.95
  (8/32 at 8).** The adjudicated verdicts stand; every Holm interval's upper bound is ≤ 0.959.
- *The right control already exists:* `BzReader::parallel_lazy_as::<I>` (`StreamDecoder::new_lazy_as`)
  builds production's decoder with only the lazy kernel swapped. The registration did not use it.

**The 8/32 arm mixes two changes (C).** The paired 8/32-over-7/28 opend ratio is 0.978 at 10 threads.
That is the 7 → 8 worker gain bench-sustain found for LazyCells (0.974), not a kernel effect. **The
kernel's own gain at 10 threads is opend 7/28's, about 0.95 after the correction.**

## 3. The mechanism (I, from the records)

**What the diagnostic shows (C).**
- *Bytes, per output byte at 10 threads:*
  - opend 7/28 moves **1.035×** the reference's DRAM bytes, and 8/32 **1.152×**.
  - Most of 8/32's excess is the eighth worker: record 53's model, 4.42 + 0.074·W² bytes per output
    byte, gives 8.05 → 9.16 from 7 to 8 workers, and opend 8/32 measured 9.25.
  - The all-blocks `open` arm moves *fewer* bytes (0.975).
- *Energy:* package power and energy per GB are *lower* for every open arm (J/GB 0.875–0.925).
- *Bandwidth:* 22–24.5 GB/s at the controller, well under record 52's ceiling of about 32–33 GB/s at
  7–8 workers. **Bandwidth is not the binding resource.**

**The contention model (records 45–53).** Cells' per-step cost is c(W) = a + b·W·W_run.
- *What b is:* the cost of *latency* under contention. Records 47–53 trace it to dependent slot loads
  whose L3 and DRAM latency rises with the miss rate (L3-hit latency ≈ 44.8 + 3,418 × L3-miss demand
  reads per byte).
- *Its share:* at 7–8 workers the contention term is about 40% of per-step cost (b·W² = 0.33 against
  a = 0.42). At 4 workers it is about 20%.

**The open decoder (record 57):** Rauzy–Veech only on the wide strips, and a counting sort for the
rest. Its main access pattern is streaming passes with high memory-level parallelism, not a chain of
dependent loads.

**The reading (I).** opend does more base work and moves somewhat more bytes, but has a much smaller
latency-contention coefficient.
- *Why it wins at 8–10 workers:* there, inflated latency dominates Cells' step cost.
- *Why it loses on a 4-CPU VM:* there, latency isn't inflated, so the extra work shows. The VM result
  was also against eager Cells, on different hardware, so it is only suggestive.
- *A toy fit:* taking a 9–21% loss at W = 4 and a 6% gain at W = 7 gives a_open ≈ 1.3a and
  b_open ≈ 0.45b. Two points; it illustrates, it does not test.
- *Consistent with:* lower power per wall (fewer stalled cycles at the same clock), and peak RSS
  0.79–0.86 of the reference (a smaller hot set under a shared 20 MB L3).

**The measurement that would test it.** One cell on the i9 at the registered 2 s settle, unpinned.
- *The arms:* opend against LazyCells, both built with `parallel_lazy_as` or both with
  `workers_with`.
- *The counts:* W ∈ {2, 4, 6, 7, 8, 10} workers.
- *Per-decode counters:* cycles, instructions, `cycle_activity.stalls_l3_miss` (or
  `l1d_pend_miss.pending`, for mean miss latency) and IMC bytes.
- *Predicted under the reading:*
  1. opend's wall ratio over LazyCells *falls* with W, crossing 1 between W = 2 and 6;
  2. opend has more instructions and DRAM bytes per output byte at every W, but **fewer L3-miss stall
     cycles per output byte at W ≥ 7**;
  3. LazyCells' stalls per byte grow about as W², and opend's much more slowly.
- *Refuted if* the ratio is flat in W, or if opend's stalls per byte are no lower at W = 8.

## 4. What a B5 integration study must check that this run didn't

1. **Kernel-only construction.** Use `new_lazy_as::<OpenGated>` (or `parallel_lazy_as`), so that an
   arm differs from production in the kernel alone. Judge the worker count (8 rather than 7)
   separately, under rule 4, since its memory sits near the gate.
2. **Other thread counts and placements.** Only unpinned 8 and 10 were run. Production trims at 4
   and 6 (eager), and at 12–20 unpinned and pinned. The VM loss suggests a crossover: map the W sweep
   above, and gate by worker count if it holds.
3. **Corpora below the density line that are not Wikidata.** `ab gen` made onebyte, pattern, runs and
   records, but **no arm decoded them.** On text and random opend never engaged, so the guards say
   nothing about the open kernel off Wikidata. The all-blocks `open` arm was 1.73× slower on text,
   so the density line is doing real protective work.
   - *Run:* prose, the generated pathological corpora, and adversarial low-density columns.
4. **Correctness at production standard.** The open decoder is compiled from the journal
   (`#[path = docs/journal/.../open_return.rs]`). Before any default it needs:
   - the exhaustive small-column tests;
   - per-block equality on all 233 large blocks and the prose controls;
   - its refusal and budget paths, including the case where a block under the line exceeds the cell
     budget and the open decoder runs;
   - the Guardian's safety review of any `unsafe`.
5. **Worst-case memory.** Peak RSS was 0.79–0.86 on Wikidata. The open decoder's scratch scales with
   its band count (record 57: B ≤ 3H + σ, tight), so its worst case on adversarial blocks needs a
   bound or a cap.
6. **The other production paths.** opend was only in the `workers` plan. Production at other counts
   uses the service core or pipelined teams, and the gate would have to sit inside those too.
7. **Sustained and multi-stream load.** These were 0.3 s decodes behind 2 s gates. Run a continuous
   (bench-sustain-style) cell, because the contention reading predicts the gain *grows* under
   sustained contention, and that should be checked.

**What this decides:** nothing in production. As registered, ADOPTS opens a B5 study.
