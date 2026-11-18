# The worker-count recalibration on 0.5.5's kernel: the registration

*18 November 2026. The Pathfinder's C6 adoption registration, frozen before any data.*

- **Assigned:** the Coordinator's CHECKPOINT 6977086376 on #314.
- **Runbook:** RELEASING §5c, proposed in #560.
- **Recipe:** `just bench-recalibrate`, on `bench/recalibrate`.
- **Predictions:** MONODROMY's, frozen first as record 56, `docs/journal/monodromy/2026-11-18-recalibration-predictions.md`, at #497's head 131c8a548a0506670941be2d30f6e0e6e1799e40 (REVIEW 6977179546 on #145).

## What it decides, and when

0.5.5 (#559) widened the lazy kernel's admission. On Wikidata ×4 at 8 threads unpinned, the wide
arm took 0.317 s where master's took 0.381 s: about 17% less wall
(admit/run-20261118T100717Z, LOG 6975512468). Within the wide arm, 8 workers took 0.979 of
10's, where master's took 1.026.

Every trim the default decoder makes on the measured i9 (`trim_cell`, `trims_to_seven`) was
measured on the narrow kernel. This run measures each one again on 0.5.5's.

**The integration decision:** each trimmed cell's worker count and kernel, in `trim_cell`'s
table, before the next release (§5c). Any cell whose choice is not the shipped trim gets its
own PR to master. Each such PR carries:
- the change;
- a deterministic test of the new cell;
- this run's figures.

The Coordinator routes it to the Researcher before any tag. A cell where nothing ADOPTS keeps
the shipped trim.

## The decoder measured

- **The commit:** master's source as released, v0.5.5 at 7214f8ca. The recipe refuses to run
  where `src`, `Cargo.toml` or `Cargo.lock` differ from that commit's.
- **What this branch adds:**
  - the harness's occupancy arms and its unpinned runs, from `bench/unpinned`, unchanged;
  - the readers, as below;
  - a test.

  It adds nothing to `src`.
- **The arms:**
  - `mibz2-occ-N-S` is `StreamDecoder::workers(N, S, k, true)`: N lazy workers and S blocks in
    flight, whatever the threads asked.
  - `mibz2-oce-N-S` is the same with `false`: N eager workers, a slot for every letter.

## The cells and arms

Each cell is a thread count and a placement:
- **Pinned:** one CPU on each of exactly T cores, `ab run --spread`, as every earlier trim sweep
  pinned. The spinner is held on a free core at 4, 6 and 8 threads, none at 10.
- **Unpinned:** every online CPU, `ab run --unpinned`, with no spinner.

At each cell the reference is the **shipped trim**: the decoder `StreamDecoder::new(T, 4T, k)`
builds there on the measured i9. The candidates are:
- **its neighbours:** one worker fewer and one more, in its kernel, with 4 blocks a worker;
- **the untrimmed plan:** `Plan::for_threads(T)` with 4T blocks;
- **at the eager cells, the lazy kernel at the shipped count.**

| placement | threads | shipped (reference) | −1 | +1 | untrimmed | other kernel |
|---|---|---|---|---|---|---|
| pinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| pinned | 8 | lazy 6/24 | lazy 5/20 | lazy 7/28 | lazy 8/32 | — |
| pinned | 6 | eager 5/20 | eager 4/16 | eager 6/24 (= untrimmed) | eager 6/24 | lazy 5/20 |
| pinned | 4 | eager 3/12 | eager 2/8 | eager 4/16 (= untrimmed) | eager 4/16 | lazy 3/12 |
| unpinned | 4 | eager 4/16 (untrimmed) | eager 3/12 | not run (above the threads) | — | lazy 4/16 |
| unpinned | 6 | eager 6/24 (untrimmed) | eager 5/20 | not run | — | lazy 6/24 |
| unpinned | 8 | lazy 8/32 (untrimmed) | lazy 7/28 | not run | — | — |
| unpinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| unpinned | 12 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/48 | — |
| unpinned | 16 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/64 | — |
| unpinned | 20 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/80 | — |

`tests/recalibrate_arms.rs` checks this table deterministically, under
`parallel::with_machine` with the measured i9's facts:
- **The shipped column:** each shipped arm has the default decoder's worker count, window and
  kernel. `Machine::pinned_i9(T)` stands in for the pinned cells and
  `Machine::service_target()` for the unpinned ones.
- **The untrimmed column:** each untrimmed arm is the plan's decoder.
- **The neighbours:** each differs from the shipped arm by one worker and 4 blocks.
- **The kernel arms:** each differs from the shipped arm in its kernel alone.

At run time, `two_arm.py --check-workers` VOIDs the run if any record of an arm fails to name
exactly that arm's N worker threads (`mibz2-0` to `mibz2-(N-1)`).

### Three choices of design, and why

1. **No arm runs more workers than the threads asked.**
   - *The contract:* the public contract is the thread count. `BzReader::parallel(r, threads)`
     "decode[s] on `threads` threads", and the README's "4 worker threads" says the same. A
     default that ran 5 workers when asked for 4 would break it, whatever its speed.
   - *What it removes:* the "+1" arm at unpinned 4, 6 and 8, where the shipped decoder already
     runs one worker a thread asked. Pinned at 6 and 4, "+1" is the untrimmed plan itself.
   - *The prediction there:* MONODROMY's record marks exactly those cells "no basis".
2. **Unpinned, one worker fewer is run once, at 10 threads; one worker more at all four counts.**
   - *The same decoders:* an occupancy arm builds the same decoder whatever the threads asked,
     and unpinned it runs on the same CPUs. So lazy 6/24, 7/28 and 8/32 at 12, 16 or 20
     threads are the runs at 10 again.
   - *The shipped trim is the same too:* it is 7/28 at all four counts. The window scales as
     4 blocks a worker (`window`; the test above checks 12, 16 and 20).
   - *What differs:* only the untrimmed plan's window (40, 48, 64, 80).
   - *Where the power goes:* 8 workers against 7 is the predicted change, and the Coordinator
     asked for the power on 7, 8 and 10 at 10 to 20 threads. So lazy 8/32 is judged at 10, 12, 16
     and 20: four independent paired replicates of one comparison, each its own cell. That costs
     9 runs a round, about a minute. One worker fewer (lazy 6/24, predicted 1.046) is judged at
     10 alone, and is read as the answer for 12, 16 and 20.
   - *Where the change lands:* a trim adopted here changes `TRIMMED_WORKERS` from 10 threads up,
     where `trims_to_seven` applies it. If the four counts disagree, the choice is made count by
     count, as the rule says, and a PR changing only some counts needs its own predicate.
   - *If pinned and unpinned 10 disagree:* the PR must split the constant by affinity, or keep
     it.
3. **The lazy kernel at the eager cells.**
   - *Why the kernel is a candidate:* the kernel is part of the trim (`trim_cell` returns it),
     and wide admission changed the lazy kernel alone. Eager Cells, the teams and the service
     core keep Cells' admission.
   - *The old result:* unpinned at 6, lazy 5/20 took 1.077 of eager 6/24, while eager 5/20 took
     1.009 (`bench-unpinned`, LOG 6949008682). Those were the narrow kernel's figures.
   - *Why it may flip:* a 17% gain at 8 threads could change that ordering. Its size at 4 to 6
     workers is unknown: with fewer workers there is less memory contention for the kernel's
     smaller traffic to relieve.
   - *Not predicted:* MONODROMY's record predicts the eager cells from their earlier measured
     ratios, and has no prediction for these four arms. Their verdicts are read as this rule
     reads them, with no model to check.

## Method

- **One `ab run` a cell, with that cell's arms alone.** In the shipped trim's order, rotated and
  reversed round by round as `ab run` does.
- **Rounds:** 12 paired rounds. Twelve is a multiple of 2 × arms for 2, 3 and 4 arms, so every
  arm runs in every position equally often in every cell.
- **Pinned at 8 threads, 24 rounds** (`--rounds-at 8=24`), 24 being a multiple of 2 × 4 arms.
  This is the cell most likely to contradict the model (the Coordinator's input, 18 November):
  - *0.5.5's crossover* (crossover/run-20261118T113151Z, the Coordinator's LOG on #145) put
    one block a thread, 8 lazy workers, against the shipped 6/24 at 0.983 [0.955, 1.011];
  - *0.5.4's* was 1.050, in the trim's favour;
  - *record 56* predicts no change there (8: 1.10).

  The cell is meant to tell 6 workers from 8 in either direction, if its noise allows. The
  power section below says how far it can; that claim is qualified there.
- **One build** by rustc 1.99.1, the repository's pin.
- **The corpora:**
  - Wikidata ×4, the gain family;
  - text and random, the guards, generated afresh by `ab gen` for the run;
  - every input's SHA-256 recorded.
- **The settle gate is 0.25 s** (`--settle 0.25`), not the default 2 s:
  - *Why:* the time budget is about 15 minutes, and almost all of the default run's time is
    the gate. In the admit run, 120 runs took 264 s, 240 s of them the 2 s gate and 21.5 s
    the decodes.
  - *What it keeps:* 0.25 s still samples the other processes' load before each run, the
    gate's purpose (`--max-busy` 1.5 as before). It is the i9's `share-nothing` setting.
  - *Drift:* thermal drift is balanced by the paired, rotated rounds. Balance by position
    does not rule out carryover, so the diagnostics below are declared now, before the data.
- **Time:** 111 runs a round (48 pinned and 63 unpinned). At 0.25 s of gate, about 0.02 s of
  harness and about 23 s of decoding, that is about 53 s a round:
  - about 10½ to 11 minutes for 12 rounds;
  - about 1 minute for pinned 8's 12 further rounds (12 runs and about 5 s a round);
  - about 1 minute to build and generate;
  - about ½ minute for the counters;
  - **about 13½ minutes in all.**

### Power, sized to the effects predicted (the fused-finish lesson)

**Noise.** The paired rounds' SD of the log wall ratio on Wikidata ×4 was:
- 0.010 to 0.014, between neighbouring counts at 6 to 10 workers (`bench-unpinned`);
- 0.031, eager 3/12 against 4/16 at 4 threads unpinned.

**The strictest step.** Holm's strictest step here is one-sided p < 0.025 / 12 ≈ 0.0021. With
12 rounds (t₁₁ ≈ 3.60), the gain detectable with 80% power is:

| SD | 0.010 | 0.012 | 0.014 | 0.031 |
|---|--:|--:|--:|--:|
| gain detectable at 80% | 1.3% | 1.5% | 1.8% | 3.9% |

(With 8 rounds: 1.8%, 2.1%, 2.5% and 5.4%.)

**Against MONODROMY's predictions:**
- *8 workers against 7:* predicted at 0.979 unpinned and 0.982 pinned at 10, gains of 2.1% and
  1.8%. Both are within reach.
- *A gain under about 1.3%:* not resolved.

**Pinned at 8 threads, with 24 rounds:**
- *Holm's strictest step* (t₂₃ ≈ 3.18) detects a gain of 1.0% with 80% power at an SD of
  0.012, and 1.6% at 0.020.
- *The 95% interval's half-width* is about 0.5% at an SD of 0.012, and 0.8% at 0.020.
- *A simulation* (as below, with the other eleven cells null) rejects a 1.7% gain of 8 over 6:
  with power 1.00 at an SD of 0.012, and 0.84 at 0.020.
- *Either direction:* a loss of that size is labelled SLOWER.
- **Qualified (the Researcher's REVIEW 6978182129):** those figures assume an SD of 0.012 to
  0.020. The one measured SD for this cell is higher. 0.5.5's pinned-8 crossover gave a paired
  log SD of **0.03955** (one block a thread against 6/24, Wikidata ×1). At that SD, a true 1.7%
  gain has about **18%** power at Holm's first step, against 83% at 0.020. The registered
  Wikidata ×4 input may be quieter, but that is not known before the data. So this cell
  probably resolves a gain of about 1.7% only if its noise is near 0.020. If the SD comes out
  near 0.04, an INCONCLUSIVE there is the expected outcome and is no evidence of no
  difference. The run's SD is reported for the cell, beside these figures.

**Splitting α between the two placements** costs nothing. One family of 26 cells at 0.05
would give about the same thresholds. The unpinned family's strictest step, 0.025 / 14, needs
t₁₁ ≈ 3.66 rather than 3.60, and loses under 0.05 points.

**At 10 threads and up unpinned,** 8 against 7 is judged four times, at 10, 12, 16 and 20.
Each count still gets its own verdict. A simulation (paired normal logs, 12 rounds, Holm over
14 cells at 0.025, the other ten cells null) gives the chance that Holm rejects 8 over 7:

| true gain | SD | at one count or more | at a given count |
|---|---|--:|--:|
| 2.1% | 0.012 | 1.00 | 0.99 |
| 2.1% | 0.014 | 1.00 | 0.92 |
| 1.5% | 0.012 | 1.00 | 0.74 |
| 1.0% | 0.012 | 0.75 | 0.29 |
| 1.0% | 0.014 | 0.57 | 0.19 |

## The rule

`bench/two_arm.py`, called once per placement, as #506's family, extended:

- **The family.** Pinned: 12 main-corpus cells. Unpinned: 14. Each is one Holm family at
  α 0.025 (`--alpha 0.025`). Two families at 0.025 keep the family-wise error at 0.05
  (Bonferroni over the two). Each call has a reference for each count (`--ref-at`) and
  candidates judged at their counts alone (`--judge-at`, `--sparse`).
- **A cell's step:** the paired rounds' wall ratios, candidate over reference: their geometric
  mean and its 95% interval (Student's t on the logs). FASTER, SLOWER or INCONCLUSIVE.
- **The gain:** Holm's one-sided tests on Wikidata ×4.
- **The loss screen:** no cell on any of the three corpora SLOWER. This is stricter than
  PERFORMANCE-POLICY rule 3 for random: #506's family, as the CHECKPOINT asks.
- **Memory, rule 4 as amended (#558).** At the candidate's count, the median peak RSS over the
  reference's must be at most 1.10 on every corpus. Where it is above:
  - **ADOPTS BY TRADE**, encoded (`--primary wikidata-x4,text`), when exactly one corpus is
    above 1.10 and it meets all of these:
    - it is a primary corpus (Wikidata or text);
    - its step is FASTER with a geometric mean at most 0.95 (a gain of at least 5% beyond
      noise);
    - its growth in percent is at most twice that gain;
    - its growth is at most 64 MB (62,500 KiB) of peak RSS.
  - **Two corpora above 1.10 never trade,** however each qualifies alone. #558 lets no other
    corpus grow past 10%. As first written, the reader accepted Wikidata and text both at +12%
    with 8% gains. The Researcher's REVIEW 6978182129 reproduced that, and the reader now
    holds it, with a regression check.
  - **HELD ON MEMORY** otherwise: the Coordinator weighs it, as rule 4 says.
- **The verdicts at each count:**
  - ADOPTS
  - ADOPTS BY TRADE
  - HELD ON MEMORY
  - A FINDING, NOT ADOPTED (a gain, but a cell SLOWER)
  - NOT ADOPTED
- **The choice at each count:** of the candidates that ADOPT or ADOPT BY TRADE, the lowest
  main-corpus step. If none does, the shipped trim stays.
- **VOID** (no verdict for that placement):
  - every check `two_arm.py` makes: the records whole; the outputs identical; the machine the
    registered i9 (10 cores, 20 CPUs, no quota); the pinning and the spinner as registered;
  - and now a record whose observed worker threads are not its arm's N.

  A VOID placement is not re-run on this registration.
- **One run decides.** As PERFORMANCE-POLICY says, there is no repeat for an inconclusive or
  non-ADOPTS cell, and no extension of the rounds.

**What the rule most likely says, from the predictions:**
- *A trim from 7 to 8 workers at 10 threads and up:* about 2% faster for about 12–14% more peak
  RSS. Per worker, the earlier runs measured 163 MB at 7 and 186 MB at 8, on Wikidata
  unpinned at 8 and 10.
- *Under rule 4:* a gain under 5% cannot buy growth above 10%. So the likely verdict for
  8 workers is HELD ON MEMORY, for the Coordinator to weigh, if Holm rejects and the loss screen
  clears. To pass the trade, 12–14% growth would need a gain of 6–7%. It is not an adoption by rule.
  This is said now, before the data, so that it cannot be argued after.

## MONODROMY's predictions, cited

Record 56 (#497 at 131c8a54). The predicted wall over the shipped trim, on Wikidata ×4:

| threads | placement | shipped | its best | its predicted ratios |
|---|---|---|---|---|
| 10 | pinned | lazy 7 | 8 | 8: 0.982 [0.94, 1.04] · 6: 1.054 · 10: 1.115 |
| 8 | pinned | lazy 6 | 6 | 7: 1.029 [0.96, 1.09] · 5: 1.092 · 8: 1.10 |
| 6 | pinned | eager 5 | 5 | 6: 1.07 [1.03, 1.10] (measured) · 4: 1.035 (model) |
| 4 | pinned | eager 3 | 3 | 4: 1.04 [1.01, 1.08] (measured) · 2: 1.35 (model) |
| 4 | unpinned | eager 4 | 4 | 3: 1.21 [1.17, 1.25] (measured) |
| 6 | unpinned | eager 6 | 6 | 5: 1.01 [0.98, 1.04] (measured) |
| 8 | unpinned | lazy 8 | 8 | 7: 1.022 [0.99, 1.06] |
| 10, 12, 16, 20 | unpinned | lazy 7 | 8 | 8: 0.979 [0.94, 1.01] · 6: 1.046 [1.01, 1.08] · 10: 1.043 [0.99, 1.09] |

- **Its "+1 above the threads" predictions** (5 at unpinned 4, 7 at 6, 9 at 8) are not tested:
  those arms are not run, by design choice 1.
- **Its refutation criterion:** at 10 or more threads unpinned, 8 slower than 7 by more than 1%,
  or untrimmed 10 faster than 7. This run's unpinned 10, 12, 16 and 20 cells read both, each
  count's 8/32 and 10/S against 7/28.
- **The four lazy-kernel arms at the eager cells:** the record makes no prediction for them.

## Reported, and deciding nothing

**The memory controller.** It runs only where `perf` can count it system-wide
(`kernel.perf_event_paranoid` ≤ 0), with `bench/counters.py --events imc`, 5 reps on
Wikidata ×4, the idle rate subtracted. It counts DRAM read and write bytes per output byte at
three cells:
- unpinned at 10: lazy 7/28, against 6/24, 8/32 and 10/40. This checks MONODROMY's traffic
  model, 4.42 + 0.074·W·W_run B per byte.
- unpinned at 6: eager 6/24 against lazy 6/24;
- pinned at 4: eager 3/12 against lazy 3/12.

## Diagnostics, declared before the data

Each is reported for every judged cell in the run's record. None changes a verdict. A
diagnostic that shows a problem is written up beside the verdict as a qualification.
- **Order:** the mean log wall ratio of each arm by its position in the round. A position
  effect larger than the cell's 95% half-width marks the cell as possibly carrying over.
- **Drift:** the least-squares slope of each cell's paired log ratio on the round index, with
  its 95% interval. A slope whose interval excludes zero marks a drifting cell.
- **Load:** the settle gate's busy-CPU samples before each run (`--max-busy` 1.5). Reported: the
  median and maximum by cell, and any run that waited more than once.

Holm controls the family's error across the 12 and 14 comparisons without needing the cells to
be independent, given valid paired p-values for each. The four unpinned 8-against-7 comparisons
on one host are each given a separate count verdict. The independent-batch simulation above
gives a power estimate, not a joint guarantee across counts.

## What it does not decide

- No other machine, mask, quota or SMT placement.
- No arm with more workers than the threads asked.
- No count other than these eleven cells.
- `LAZY_FROM_THREADS` and the plan off the measured i9.

A lazy kernel adopted at an eager cell changes that cell's `trim_cell` entry alone.

## The tooling, checked before the data

- **`bench/two_arm.py`:** `bench/unpinned`'s multi-arm reader, extended with:
  - several guards (`--guard text,random`);
  - `--alpha`;
  - rule 4's trade (`--primary`);
  - `--check-workers`;
  - a reference by count that is a candidate at another count (lazy 6/24, the shipped trim at
    8 pinned and one worker fewer at 10), never at its own;
  - `--rounds-at`, rounds by count.

  `bench/test_two_arm.py`: 144 checks. 28 of them are new (one is the two-corpus regression of
  the Researcher's REVIEW 6978182129): each trade condition met and
  broken, the workers check, the halved α, the reference-and-candidate cases, and rounds by
  count.
- **`bench/counters.py`:** `bench/admit`'s, with the IMC events.
- **The smoke run:** a 2-round run of the whole recipe on a 4-core CI-like machine. Every arm
  ran. Both families VOIDed only on the machine (not the i9) and, pinned, on the spinner
  sharing a decoder core. Every record's workers matched its arm.
