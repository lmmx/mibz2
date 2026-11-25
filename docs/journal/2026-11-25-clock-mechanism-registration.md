# bench-clock-mechanism: what sets the low clock (registration)

*A C1a diagnostic, registered before its data: it decides nothing. Assigned by the Coordinator on 25 November after the Pathfinder's LOG 7152489390 on #145 (the clock and the character study's noise). Recipe `just bench-clock-mechanism`, on #843's branch with the character study's binary and pin (master 9621b7ad with the `character` feature). If an arm controls the clock, later recipes adopt it as their standard warm-up (the Coordinator's decision).*

## What is known

**Stage 2b's timed decodes at 10 workers** (evidence a4f0dcef) ran at one of two clocks:
- the two levels are 4.91 and 4.09 GHz, as cycles over ref-cycles × 3.7 GHz from each decode's PMU group 0;
- 26% of decodes ran at the low one, scattered at random through the run (306 runs against 306 expected);
- the low clock was not persistent from one decode to the next, and not tied to placement;
- the throttle counters were 0, and the package ran at 43–56 °C.

**The clock explains R² 0.96 of 2b's paired noise at 10.** The paired SD falls from 6.0% to 1.5% at a common clock.

**Journal 2026-10-28 §4** (`bench-regimes`, 6 threads, 12 interleaved rounds) found:
- **with an idle gap of 0.15 s before each run:** 5 of 12 plain runs were slow, and under `sudo perf` the clock was about 7% lower (cycles/ref 1.255–1.317, against 1.326);
- **with no gap:** no slow runs, and the full clock every time;
- **its mechanism was left open:** "a lower clock after idle and more stall cycles aren't separated".

Every decode here carries its clock, so the two are separated.

## The design

**The decode** is 2b's pinned base at 10:
- the scanner is pinned (`scanner_cpu`) to the first CPU of the core whose pinned calibration runs fastest (the `cores` phase, as in 2b);
- every other thread is masked off that core;
- the ledgers are off;
- the corpus is four copies of the Wikidata corpus.

**Five arms**, one decode each in each of 48 randomised blocks:

| arm | before the decode |
|---|---|
| S0 | nothing: back to back with the decode before it |
| S0.2 | 0.2 s idle (about the 0.15 s gap of 2026-10-28 §4) |
| **S1** | **1 s idle: 2b's settle, the reference** |
| S3 | 3 s idle |
| SPIN | 1 s idle, then about 100 ms of the pacing loop on every CPU at once, then the decode at once |

The SPIN arm's processes are the binary's own `--calibrate`, one a CPU, each pinned, sized from the scanner core's calibration.

**The clock:** each decode's G is its PMU group 0, cycles over ref-cycles × 3.7 GHz. A decode is **LOW** iff G < **4.50 GHz**, fixed before the data at the midpoint of 2b's two levels at 10.

**The run:** 240 decodes after 50 s of pre-warm, about 7 minutes.

**The size:** 48 blocks give a 95% Wilson interval of about ±0.12 on a share near 0.26. A difference of 0.26 against S1 (an arm that removes the low clock) is resolved with its interval wholly below 0.

## THE C1a RULE (it decides nothing)

**For each arm** the report gives:
- its LOW share, with a 95% Wilson interval;
- its difference from S1's share, with Newcombe's hybrid score 95% interval (method 10);
- the median G;
- the median wall and the SD of log wall;
- the paired SD of log(T/T_S1) over blocks;
- its wall over S1's, paired by block, with a t-interval.

**The labels, on the difference from S1:**
- **CONTROL** iff its interval lies wholly below 0: the arm lowers the low clock's share.
- **WORSE** iff wholly above 0.
- **NO-EFFECT** iff wholly within ±0.10.
- **UNRESOLVED** otherwise.

**The reading** (registered, not a decision):
- **If S0 is a CONTROL and S0.2 and S3 are not below S1,** idle time before a decode sets the low clock. That's 2026-10-28 §4's gap effect at 10 workers, now shown to be the clock.
- **If SPIN is a CONTROL,** a 100 ms all-CPU warm-up is the candidate standard for later recipes (the Coordinator decides).
- **If no arm controls it,** the clock's level is not set by the idle before a decode or by a cold start, and "record and decompose" (§81) stays the only remedy.

**VOID** if any of these:
- the machine is not the i9-10900K;
- an arm's decode in a block is missing, duplicated or failed;
- an output is not 4 × 209,700,196 bytes;
- any decode has a major fault;
- over 10% of decodes lack the clock (PMU group 0 unread).

## Checks

`bench/test_clock_mechanism.py`, 9/9, covering:
- the plan: the fastest core's first CPU pinned, its core masked off, the spin's size, the cut;
- an arm never at the low clock is a CONTROL;
- an arm at it more is WORSE;
- like arms are neither;
- each VOID;
- Newcombe's interval is symmetric for a nil difference.

The recipe follows the fail-soft pattern: phases in subshells, `phases.tsv`, an EXIT trap that always bundles.

## The preflight

**The DRY** (`just bench-clock-mechanism dry` at 35558756: two blocks, every settle 0, a 2 s pre-warm; a 4-CPU container with no PMU, not evidence). It is archived as `2026-11-25-clock-mechanism-registration/run-20261125T040707Z.tar.gz` (sha256 b3857923…d03398cc).
- Every phase from checks to run passed. The cores phase pinned the scanner to CPU 3 here.
- 10 decodes, all byte-exact. SPIN's all-CPU spin took 0.11–0.12 s.
- The report was VOID on the machine, and on the clock being unread in every decode (the container has no PMU), as designed.

**Since the DRY:** the paired wall figures are printed for every arm whether or not the clock was read (the walls need no clock).
