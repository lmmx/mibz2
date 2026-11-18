# The opend worker sweep: the rule, before any data

*18 November 2026. The Pathfinder. A diagnostic, posted on #145 before the data (RULES C1a). It
decides nothing in production.*

- **Assigned:** #145 QUEUED 6992388066, step 1 of the opend B5 study.
- **The prediction it tests:** MONODROMY's record 71 §3 (REVIEW 6992362897, `f94a1119`), frozen
  before this run.
- **Recipe:** `just bench-opend-sweep`, on `bench/opend-sweep`.
- **The scorer:** `bench/opend_sweep.py`, tested in `bench/test_opend_sweep.py`.

## The question

`bench-open-return` (open-return/run-20261118T210422Z, LOG 6992226855) found opend faster than
0.5.6 at 8 and 10 workers, with more DRAM bytes and less package power. Record 71 reads that as a
smaller latency-contention term: Cells' dependent slot loads cost more as workers contend, and the
open decoder's streaming passes cost less. If so, opend's advantage grows with the worker count:
- it loses at few workers;
- it wins at many;
- and it stalls less on L3 misses a byte where it wins.

## The arms, built the same way

| arm | built as | kernel |
|---|---|---|
| `mibz2-occ-W-4W` | `StreamDecoder::workers(W, 4W, k, true)` = `workers_with::<LazyCells>` | LazyCells as shipped |
| `mibz2-opend-W-4W` | `workers_with::<OpenGated>` | the open kernel under LazyCells' density line, LazyCells elsewhere |

- **W workers exactly, 4W blocks in flight,** for each W from 2 to 10. The two arms differ in
  the kernel alone.
- **Why not `parallel_lazy_as`** (`StreamDecoder::new_lazy_as`):
  - it builds production's plan for the threads asked, so on the i9 it trims 10 threads to 7
    workers;
  - at production's eager cells (6 and 4 threads) it is `new` itself, with no kernel swap.
  - So it cannot give an exact W from 2 to 10.
- **The `BzReader::parallel` constant is avoided:** record 71 measured it at 1–2 ms in
  `bench-open-return`. Neither arm here is built through `parallel`.
- **Text is an A/A check:** no text block is under the density line, so opend runs LazyCells as
  shipped there.

## Method

- **The machine:** the i9-10900K, unpinned on every online CPU, with no spinner.
- **The decoder:** v0.5.6's source with the research feature `open-return`. The recipe checks
  that nothing else changed.
- **Corpora:** Wikidata ×4, the main corpus, and text, generated afresh.
- **Timing:** one `ab run` a W, LazyCells first in the arm list, 12 paired rounds, the 2 s
  settle gate. That is 432 decodes, about 25 minutes.
- **The census** of each corpus's routes, before any timing. It decides nothing.
- **The counters, per W, 5 reps** on Wikidata ×4. One file a W and an event set, each reported
  pairwise:
  - `counters.py --events default`, per process: cycles, instructions, branches,
    `cycle_activity.stalls_total`, `stalls_mem_any` and `stalls_l3_miss`;
  - `--events imcpower`, system-wide: the memory controller's bytes and the package's energy.
  - *Time:* about 8 minutes.
- **Needs `kernel.perf_event_paranoid` ≤ 0.** The recipe refuses to start otherwise, except as
  a dry run.

## The rule

It is **VOID**, and nothing is read, where:
- a timed record is missing, duplicated, malformed or failed;
- a corpus's outputs differ;
- an arm's record does not observe exactly its W workers;
- the `lscpu` does not name an i9-10900K;
- a stall count at any W is missing, failed or not counted;
- the run is a dry run.

**The ratios:** at each W and corpus, the 12 rounds' paired log ratios of wall time, opend over
LazyCells. The report gives their geometric mean and a two-sided 95% t-interval.

**The predictions scored** (record 71 §3):

| | prediction | test |
|---|---|---|
| **M1** | the ratio falls with W | W = 2's log ratio exceeds W = 10's, the 95% interval of the difference (Welch, 11 degrees of freedom) excluding 0 |
| **M2** (secondary) | it crosses 1 between W = 2 and 6 | W = 2's interval wholly above 1, and some W from 3 to 6 wholly below 1 |
| **M3** | fewer L3-miss stalls a byte at W ≥ 7 | at W = 8, `stalls_l3_miss` per output byte, opend over LazyCells: the median of the 5 reps' paired ratios below 1, and every rep below 1. W = 7, 9 and 10 reported beside it |

**The verdict:**
- **REFUTED** if M1 fails or M3's median at W = 8 is 1 or above. These are record 71's own
  refutation conditions: "flat in W, or opend's stalls a byte no lower at W = 8".
- **SUPPORTED** if M1 and M3 hold.
- **INCONCLUSIVE** if M1 holds and M3's median is below 1 at W = 8, but not in every rep.

M2 is reported as CONFIRMED or NOT CONFIRMED, and it does not change the verdict. A text interval
excluding 1 is flagged as A/A drift, and is not judged.

## What it decides

Nothing in production. It informs the B5 study's later steps:
- **SUPPORTED:** the adoption registration may gate the open kernel by worker count, at the
  crossover M2 reports.
- **REFUTED:** the 8- and 10-worker gain stands as measured, but without the mechanism. The
  adoption registration covers every count it would change, with no extrapolation from 8 and 10.

C2 holds: none of this is relaxed after the data.
