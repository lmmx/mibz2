# The worker counts under sustained load: the registration

*18 November 2026. The Pathfinder's C6 adoption registration, frozen before any data, for the
Researcher's review through the Coordinator.*

- **Assigned:** #145 QUEUED 6978994390, dispatched to the Pathfinder as CHECKPOINT 6984075662 on
  #314.
- **Recipe:** `just bench-sustain`, on `bench/sustain`.
- **Predictions:** MONODROMY's record 64, frozen first, cited below before any data:
  `docs/journal/monodromy/2026-11-18-sustained-predictions.md` at
  e9559bd0c34a14bef75a928cb722669fef82c3b6 (PR #497), REVIEW 6984343491 on #145.

## What it decides, and why

The recalibration (recalibrate/run-20261118T122744Z, LOG 6978881191) came out bimodal in every
cell with about ten cores busy. MONODROMY's record 58 (REVIEW 6978967731, #497 at 56758285)
explained the two modes:
- **A chip-wide clock state with memory of recent load.** In the slow mode every thread runs
  about 1.4× slower, the compute-bound scanner included, and parallelism is unchanged.
- **What triggers it:** a heavy decode after a rest is slow 1 time in 65. After 2.5 heavy
  CPU-seconds or more in the three runs before, it is slow 74 times in 103.

So every worker optimum this project has measured is the optimum of the fast mode, a few
seconds long. A decode longer than a few seconds runs mostly in the slow mode, and the
decoder's primary workload, a Wikidata dump, runs for hours in it.

Record 58 conjectured that under a power cap, throughput tracks the budget divided by the energy
per byte, so the sustained optimum may sit at 7 workers or below. Record 64 reverses that before
the data: it predicts 8 workers best when clamped, at 10 threads and up. Both conjectures stand
frozen, and this run scores them.

**The integration decision:**
- *What changes:* each trimmed cell's worker count (`trim_cell`, `trims_to_seven`), as this run
  finds it in the sustained state.
- *Why the sustained state decides:* it is the state of every decode long enough for its speed
  to matter.
- *What it costs:* the fast mode's figures stand as measured (the recalibration's pinned cells,
  and the earlier runs with the 2 s gate). A change adopted here gives up whatever fast-mode
  advantage the shipped count had on decodes under a few seconds.
- *How it ships:* any cell whose choice is not the shipped trim gets its own PR to master, with
  this run's figures, and the Coordinator routes it to the Researcher before any tag.
- *Where nothing ADOPTS:* the shipped trim stays.

## The decoder measured

- **The commit:** master's source as released, v0.5.5 at 7214f8ca. The recipe refuses to run
  where `src`, `Cargo.toml` or `Cargo.lock` differ.
- **The arms:** `ab`'s occupancy arms, unchanged from `bench/recalibrate`:
  - `mibz2-occ-N-S`: N lazy workers and S blocks in flight;
  - `mibz2-oce-N-S`: the same, eager.
- **The shipped arm in each cell** is the default decoder there on the measured i9.
  `tests/recalibrate_arms.rs` checks this deterministically.

## The cells and arms

The reference in each cell is the shipped trim.

| cell | placement | threads | shipped (reference) | candidates | rounds |
|---|---|---|---|---|---|
| u10 | unpinned | 10 (stands for 10–20) | lazy 7/28 | lazy 6/24, 8/32, 10/40 | 8 |
| p10 | pinned | 10 | lazy 7/28 | lazy 6/24, 8/32, 10/40 | 8 |
| p8 | pinned | 8 | lazy 6/24 | lazy 5/20, 7/28, 8/32 | 8 |
| p6 | pinned | 6 | eager 5/20 | eager 4/16, 6/24 | 6 |
| p4 | pinned | 4 | eager 3/12 | eager 2/8, 4/16 | 6 |

- **Why one unpinned cell:** unpinned, the occupancy arms build the same decoders whatever the
  threads asked, and run on every CPU. So u10 stands for 10, 12, 16 and 20 threads. The
  untrimmed plan's window at 12–20 (10/48 to 10/80) is not measured: the plan at 10 threads is
  10/40 itself.
- **No arm runs more workers than the threads asked** (the public contract, as in the
  recalibration). Unpinned below 10 there is no trim to test.
- **Both neighbours at 6 and 4 pinned.** Fewer workers draw less power, and the conjecture
  says the sustained optimum may lie lower.
- **The lazy kernel at the eager cells is not re-tested.** In the fast mode, at unpinned 6, it
  was 6.0% faster for 20.7% more peak memory, which the Coordinator declined under rule 4. A
  sustained gain would need at least 10.4% to trade, and the time is spent on the counts.

## Method (`bench/sustain.py`)

**A decode:**
- *the command:* `perf stat -a -x, -e power/energy-pkg/ -e cycles -e ref-cycles -e
  uncore_imc/data_reads/ -e uncore_imc/data_writes/ -- ab child ARM T FILE`, with the affinity
  set before the exec;
- *pinned:* one CPU on each of the first T cores, as `ab run --spread`;
- *unpinned:* every online CPU, as `ab run --unpinned`;
- *its record:*
  - from the child: its own wall, CPU, peak RSS, output and worker threads;
  - from perf: the package energy, the effective clock (cycles ÷ ref-cycles), the memory
    controller's bytes and perf's elapsed time;
  - from `/proc/stat`: the other processes' busy cores over it.

Each event is its own `-e`. They are on three PMUs, so none is grouped and none is
multiplexed.

**A cell:**
1. **The gate:** the machine is quiet for 2 s with nothing of ours running, and the other
   processes keep at most 1.5 cores busy. This is the 2 s gate the Coordinator's LOG
   6978881191 asks for. Its purpose here is a quiet start: within a cell, continuous load is
   the point.
2. **An idle baseline:** `perf stat -a sleep 2`. The cell's own DRAM rate is taken off each
   decode's bytes.
3. **A 60 s pre-warm on the reference,** so the chip and its cooler start the rounds in the
   sustained state.
4. **Rounds.** Each round runs one block per arm, back to back with no gap, in `ab run`'s order:
   rotated round by round and reversed every other cycle of rounds. So over 2 × arms rounds,
   each arm runs in each position equally often: 8 rounds for four arms, 6 for three.
5. **A block:**
   - the arm decodes the main corpus back to back for 6 s;
   - then its measured decodes: the main corpus, text, random, the main corpus.

**Corpora:**
- Wikidata ×16 (about 1.5–2.5 s a decode), the gain family;
- text ×16 and random ×8 (about 1 s each), the guards, generated afresh.

**Why no gap.**
- *The cold start:* 10 November's cold start (`docs/journal/2026-11-10-cold-start.md`) was a
  decode slowed by a 0.25 s idle gap before it. Back to back, no decode follows an idle gap.
- *The slow mode:* record 58's slow mode is the state that continuous load holds. Each block's
  6 s of warm-up puts the chip in the state of the arm's own load. Record 58 puts the onset
  within one heavy decode, under a second.

**The probe** decides nothing. It runs first:
- *what:* 10 lazy workers (10/40) unpinned, decoding Wikidata ×16 back to back for 120 s, after
  a 30 s rest;
- *what it records:* the clock and the watts of each decode against time;
- *what it reports:* the first decode's clock (the fast mode) against the last 60 s's, and the
  onset, the time from which every decode's clock stays within 2% of the last 60 s's.

**Time on the i9:**
- about 7.5 min each for u10, p10 and p8;
- about 4.5 min for p6 and 5 min for p4;
- 5 min of pre-warm (60 s a cell), 2.5 min of probe and rest, and about 2 min to build and
  generate;
- **about 43 minutes in all.**

## The rule

**`bench/sustain.py judged`** reduces each placement's blocks to `bench/two_arm.py`'s records,
one per (round, corpus, arm). Each record carries:
- the mean of its measured decodes' walls and CPU (two on the main corpus, one on each guard);
- the largest peak RSS;
- the CPUs;
- the worker threads.

It writes nothing, and the placement is VOID, where:
- a decode is malformed, missing, duplicated or failed, or has no output;
- an event on any warm or measured decode is not counted, not supported, multiplexed or not
  positive;
- a corpus's outputs differ, or an arm's decodes name different worker threads;
- **the sustained state is not held:** in more than 10% of a cell's blocks, the measured
  main-corpus decodes' mean clock differs by more than 3% from the block's last warm decode's;
- **turbo in a cell registered clamped** (u10, p10; record 64's note 1): a measured main-corpus
  decode at a clock of 1.2 or above;
- **the pre-warm missing or failed** (the Researcher's targeted correction, REVIEW 6985154079):
  - no pre-warm decode in a cell;
  - one failed or not fully counted;
  - pre-warm decodes of more than one arm, or of an arm the cell does not measure;
  - the last ending before the registered 60 s (`judged --prewarm 60`).

**`bench/two_arm.py`** then reads each placement, as #506's family with the recalibration's
extensions (#562, with the Researcher's correction of REVIEW 6978182129):
- **The family:** one Holm family per placement, at α 0.025 each, 0.05 together.
  - *Unpinned:* 3 cells (u10).
  - *Pinned:* 10 cells. p6 and p4 run 6 rounds (`--rounds-at`).
- **A cell's step:** the paired rounds' wall ratios, candidate over reference, as a geometric
  mean with its 95% interval.
- **The gain:** Holm's one-sided tests on Wikidata ×16.
- **The loss screen:** no cell SLOWER on any corpus.
- **The workers:** each record's worker threads are its arm's N (`--check-workers`).
- **Memory, rule 4 as amended (#558):** peak RSS at most 1.10 of the reference's on every
  corpus. Otherwise:
  - ADOPTS BY TRADE where exactly one corpus is above the line, and it is primary, its gain
    is at least 5% beyond noise, its growth is at most twice that gain, and its growth is at
    most 64 MB;
  - HELD ON MEMORY otherwise, the Coordinator's to weigh.
- **The choice:** at each count, the lowest step among the candidates that ADOPT. If none does,
  the shipped trim stays.
- **One run decides:** no repeat and no extension.

**What the rule says before the data:**
- *One worker more costs memory.* It costs about 12–14% more peak RSS (8 over 7, 7 over 6).
  So it can only ADOPT BY TRADE, with a gain of at least 6–7%, or else be HELD ON MEMORY.
- *Ten workers over seven* costs about 39% more peak RSS on Wikidata and about 23% on text (226
  against 163 MB, and 123 against 100 MB, in `bench-unpinned`). Two corpora above the line never
  trade, so 10/40 can at most be HELD ON MEMORY.
- *One worker fewer* uses less memory, so it ADOPTS on a gain alone.
- *Which way this leans:* the rule leans towards the shipped count or fewer, as the
  conjecture does.

## The Researcher's review (REVIEW 6985154079), as recorded

**Its one targeted correction is applied above:**
- the pre-warm is locked at 60 s in this journal and the recipe (`PREWARM="60"`, passed to the
  judge);
- a missing, failed, uncounted or short pre-warm VOIDs its placement, with regression checks in
  `bench/test_sustain.py`.

The cells, rounds and thresholds are unchanged.

**Its other points stand as written, and are reported, not changed:**
- *The thresholds' edges:* in u10 and p10, a measured Wikidata decode at a clock of exactly 1.2
  VOIDs the placement, even where the block's pair mean would pass. The 3% steady-state test
  compares a pair mean, so opposite clock excursions within a block can cancel.
- *Pinned 8's mix:* p8 is not registered clamped. The report gives its state (clamped, turbo or
  mixed) and its clamped share per arm.
- *Pinned 6 and 4:* if their measured main-corpus decodes never clamp, the report calls the cell
  **turbo**, and its verdicts are turbo-state verdicts.
- *Predecessors:* rotation balances position, not predecessor pairs. The raw `prev_arm` and
  `prev_block` fields record the distinction, and the carry-over diagnostic reports it.
- *Power at a paired log SD of 0.020:* the exact first-step power for 8/32 at the predicted
  0.970 in u10 is 84%, and at 0.975 in p10 42%. **p10 may well come out INCONCLUSIVE.** Six
  rounds give 60% power for a 2% gain at SD 0.010.
- *What isn't yet known:* neither the sustained SD nor the i9's counters have been measured in
  this run.

## Power

The sustained state's paired SD is unknown. Each round's main-corpus value is the mean of two
decodes of about two seconds each. The gain detectable with 80% power at Holm's strictest
step:

| family | rounds | t | SD 0.005 | SD 0.01 | SD 0.02 | SD 0.04 |
|---|--:|--:|--:|--:|--:|--:|
| unpinned, 3 cells | 8 | 3.13 | 0.7% | 1.4% | 2.8% | 5.5% |
| pinned 10 and 8, of 10 cells | 8 | 4.03 | 0.9% | 1.7% | 3.4% | 6.7% |
| pinned 6 and 4, of 10 cells | 6 | 4.77 | 1.2% | 2.3% | 4.5% | 8.9% |

- **What it can resolve:** a 2% gain only if the sustained state is as quiet as the
  recalibration's cells with few cores busy (SD about 0.01).
- **Where it can't:** at the recalibration's pinned-8 SD of 0.04, it resolves only gains above
  5.5–9%.
- **Holm across the cells** needs valid marginal p-values, not independence (the Researcher's
  REVIEW 6978182129).

## MONODROMY's predictions, cited

Record 64 (e9559bd0 on PR #497; REVIEW 6984343491), frozen before any data. Wall over the
shipped reference, in the sustained state, on Wikidata:

| cell | reference | its best | its predicted ratios [interval] |
|---|---|---|---|
| unpinned (10–20) | lazy 7/28 | **8** | 8/32 0.97 [0.94, 1.00] · 10/40 0.985 [0.95, 1.02] · 6/24 1.06 [1.03, 1.09] |
| pinned 10 | lazy 7/28 | **8** | 8/32 0.975 [0.95, 1.00] · 10/40 1.01 [0.98, 1.05] · 6/24 1.06 [1.03, 1.09] |
| pinned 8 | lazy 6/24 | 6 | 7/28 1.03 [1.00, 1.06] · 8/32 1.04 [1.01, 1.07] · 5/20 1.12 [1.10, 1.15] |
| pinned 6 | eager 5/20 | 5 | 6/24 1.07 [1.05, 1.09] · 4/16 1.03 [1.02, 1.05] |
| pinned 4 | eager 3/12 | 3 | 4/16 1.045 [1.03, 1.06] · 2/8 1.37 [1.33, 1.40] |

- **The basis:** the recalibration's clamped runs, taken apart by state. In them 8/32 beat 7/28
  by 2–4% in every clamped cell, and 10 workers moved from losing in turbo (1.02–1.07) to about
  level. The record gives two mechanisms:
  - each miss costs about 1.4× fewer cycles at the clamped clock;
  - under a power cap, throughput rises roughly as N^(2/3).
- **The clock** (cycles ÷ ref-cycles; turbo is 1.326):
  - clamped cells (unpinned, pinned 10): 0.93–0.97, with fewer-worker arms 2–5% faster;
  - pinned 8: 1.0–1.33, a partial clamp (conjecture);
  - pinned 6 and 4: 1.32–1.38, in turbo.
- **Package power:** 115–135 W in the clamped cells if PL1 is 125 W; 150–200 W, drifting, if the
  clamp is thermal.
- **The probe:** onset within 0.5–5 s, then steady at about 1.40× below turbo, within ±3% if
  the limit is PL1-type, or drifting down if it is thermal.
- **Refuted if:**
  - at 10 or more threads unpinned, 7 beats 8 by more than 1%; or
  - the clamped cells stay at a clock of 1.2 or more.

**Its four design notes, as folded in:**
1. **The clock screen.** u10 and p10 are registered clamped (`judged --clamped`). Any measured
   main-corpus decode there at cycles ÷ ref-cycles ≥ 1.2 VOIDs its placement, so no arm mixes
   states.
   - *Why the main corpus only:* the guards are lighter loads, and under a power cap they may
     legitimately clock higher.
   - *Reported per arm:* the clamped share of every cell's main-corpus decodes (warm and
     measured) and the guards' median clock.
   - *Pinned 8 is not registered clamped,* since record 64 predicts only a partial clamp
     there. Its state is reported.
2. **Pinned 6 and 4 may never clamp.** Their clock state is reported. If they stay in turbo
   (≥ 1.2), their verdicts are turbo-state verdicts, a repeat of the recalibration's at those
   cells. They are not claimed for the sustained state.
3. **Carry-over.** Every decode records the arm before it (`prev_arm`) and the block before it
   (`prev_block`), whatever the probe shows. The report gives each arm's measured wall by the
   block before it. Rotation balances the carry-over; this shows its size.
4. **One unpinned cell for 10–20 threads,** as designed.

**Rule 4 and the most likely outcome.** Record 64's headline is 8/32 at about 0.97 of 7/28
(a 3% gain). 8/32 has grown peak RSS by about 8–16% over 7/28 before.
- **Within 1.10 on every corpus,** and Holm rejects: it ADOPTS. Unpinned, at 3% and an SD up
  to about 0.02, that is within reach.
- **Above 1.10 on Wikidata alone:** the trade needs a gain of at least 5% and growth of at most
  twice it. At 3% it is **HELD ON MEMORY**, the Coordinator's to weigh, as in the
  recalibration.
- **Above 1.10 on two corpora:** HELD ON MEMORY, whatever the gain.

So the likeliest results are an adoption of 8 at 10 threads and up, if its memory stays within
the line, or a HELD ON MEMORY that leaves 7 shipped unless the Coordinator weighs otherwise.

## Reported, and deciding nothing (declared before the data)

`bench/sustain.py report`:
- **By arm:**
  - the measured main-corpus wall (median);
  - package watts;
  - the effective clock (cycles ÷ ref-cycles, and in GHz at the i9's 3.7 GHz TSC);
  - joules per GB of output;
  - DRAM bytes per output byte, net of the cell's idle rate.

  These fit record 58's clock term: f(P_cap, W).
- **Order:** by position in the round, each arm's wall over its own mean.
- **Drift:** the reference's wall and clock round by round, and their slope.
- **Load:** the other processes' busy cores, the largest, and the decodes above 1.0.
- **Steady state:** the share of blocks within 3%.
- **Clock state:** per arm, the clamped share of the main-corpus decodes and the guards' median
  clock (record 64's notes 1 and 2).
- **Carry-over:** each arm's measured wall by the block before it (note 3).
- **The probe,** as above.

This also fixes the recalibration's DRAM fault (REVIEW 6984061702). Every decode is counted
in place, against the cell's own idle baseline, so there is no second roster to match.

## What it does not decide

- The fast-mode (turbo) optimum, which stands as measured.
- Any machine other than the i9, and any mask, quota or SMT placement.
- `LAZY_FROM_THREADS`, or the lazy kernel at the eager cells.

**The order after it** (CHECKPOINT 6984075662): the open-return comparison (QUEUED 6978711740)
follows, at the knee this finds, in whichever state this shows production decodes run in.

## The tooling, checked before the data

- **`bench/sustain.py`:** the driver, `judged` and the report.
- **`bench/test_sustain.py`:** 36 checks, run for real with a stub `perf` and a stub
  `ab child`:
  - a complete run, judged into records that `two_arm.py` reads;
  - the phases, blocks and rotation;
  - each decode's counts, the per-controller IMC events summed, and the parser's units;
  - every VOID: multiplexed, not counted, the clock jumping (the sustained state not held), a
    failed decode, short workers, a missing or duplicated decode, outputs differing, a line
    not JSON;
  - the clamp screen, only in the cells registered clamped;
  - each decode's predecessor;
  - the report's clock state and carry-over;
  - the pre-warm: missing, failed with no counters, uncounted, short, or on an arm the cell
    does not measure, each VOID; whole and long enough, judged.
- **The smoke run:** the driver with the real `ab` (and the stub `perf`) on a 4-core machine,
  unpinned and pinned at 4 threads. Each placement judged clean, with every record's CPUs and
  worker threads right.
