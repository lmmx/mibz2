# The open-return decoder against 0.5.6's plan on the i9: the registration

*18 November 2026. The Pathfinder's C6 registration, frozen before any data, for the Researcher's
review through the Coordinator. A diagnostic: it changes no production default.*

- **Assigned:** #145 QUEUED 6978711740, dispatched to the Pathfinder as CHECKPOINT 6989549514 on
  #314, with its control updated there to 0.5.6.
- **Recipe:** `just bench-open-return`, on `bench/open-return`.
- **The decoder under test:** Astra's open-return decoder (#565,
  `docs/journal/2026-11-18-open-return/open_return.rs`), compiled where it lives.

## The question

Does the open decoder's smaller state give a throughput gain where contention limits us?
- *Its state:* 6.29 MB of peak requested heap on the frozen block, against Cells' 10.35 MB.
- *Its own measurements,* on a 4-CPU Xeon VM with no SMT siblings: 17% slower than eager Cells
  at one thread, 9% slower at four, and 7–8% faster than LazyCells.
- *What is missing:* none of those is at the i9's contended counts, and none is against
  LazyCells with 0.5.5's wide admission (#559), which 0.5.6 ships.

## Why a diagnostic

A gain here cannot ship from this run.
- *The code:* the open decoder is research code with its own unsafe blocks. It reaches
  production only through B5: a written safety argument; Miri, differential, fuzz and mutation
  testing with runtime receipts; and an adversarial review.
- *What the run decides:* whether that integration study is worth starting. The study, if
  started, ends in its own C6 adoption registration on the integrated code, against master as
  it is then.

## The decoder measured

- **The commit:** v0.5.6's source (530ed404), with the research feature `open-return` added.
  - *What the feature adds:* the journal's `open_return.rs` as a hidden module, and
    `src/open_gate.rs`, the two wrappers below. Both are cfg-gated in `src/lib.rs`.
  - *What it leaves alone:* no line of v0.5.6's `src`, `Cargo.toml` or `Cargo.lock` changes.
    The recipe refuses to run otherwise: the changed files must be exactly `Cargo.toml`,
    `src/lib.rs` and `src/open_gate.rs`, with no line removed from the first two. It records
    the diff in the bundle.
- **The reference, `mibz2`:** the default decoder, built by `BzReader::parallel` as a user gets
  it. Unpinned on the i9 that is LazyCells with 7 workers and 28 blocks in flight at 10 threads,
  and 8/32 at 8 threads. `tests/recalibrate_arms.rs` checks those defaults deterministically.
  The recalibration (run-20261118T122744Z, LOG 6978881191) and the sustained-load run
  (run-20261118T170613Z, LOG 6986593076) changed neither, so these are the counts the checkpoint
  asks for ("compare at the new knee, not at an assumed 7 or 8 workers").
- **The arms,** each the workers' kernel inside 0.5.6's worker pipeline
  (`parallel_workers_with`), with N workers and S blocks in flight:
  - `mibz2-open-N-S`: the open kernel, `Open<16, true, true>` (width-16 retention, the queued
    endpoint), on **every run-route block** (Astra's admission). Its own column fallback is
    `Segmented`;
  - `mibz2-opend-N-S`: the open kernel on the run-route blocks **under LazyCells' density line**
    (r/n below 0.15, `LAZY_DENSITY`), and LazyCells as shipped on every other block (its
    refusals by its own fallback, as in production);
  - `mibz2-oce-7-28`: eager Cells at 7/28, the checkpoint's diagnostic arm. It is context:
    reported over the reference, never judged.

### The admission, and why two arms

The checkpoint asks how the open kernel admits blocks. The census below shows that the choice
matters, so both answers are measured:
- **Astra's (`open`):** every block on the run route. Where LazyCells refuses a block over its
  density line, the open kernel decodes it all the same.
- **Production's line (`opend`):** the open kernel on the blocks under LazyCells' density line,
  production elsewhere, block for block.
  - *The line is LazyCells' first test, not all of its admission.* LazyCells also refuses a block
    under the line whose induction would exceed its budget of 0.32 n cells. The wrapper cannot
    see that refusal without running the induction, so it takes the open kernel on such a block.
    The unit test's mixed stream has 5 blocks under the line, of which LazyCells admits 1.
  - *On Wikidata the two coincide:* LazyCells admits all 228 blocks under the line, so `opend`
    competes kernel against kernel there. The census's "both" column shows whether that still
    holds on the run's own inputs.

**The census** (`ab census-open`, single-threaded, each file's output checked against the
default's). These are the inputs as the recalibration built them; the run recomputes them on its
own inputs, before any timing, into `census.jsonl`:

| corpus | blocks | run route | LazyCells admits | under the density line | both | column route |
|---|--:|--:|--:|--:|--:|--:|
| Wikidata sample (82f65ace…) | 431 | 383 | 228 | 228 | 228 | 48 |
| text | 48 | 48 | 0 | 0 | 0 | 0 |
| random | 36 | 0 | 0 | 0 | 0 | 36 |

What follows from it:
- **On Wikidata,** the density line selects exactly LazyCells' 228 admitted blocks (no block under
  it is refused on budget). `opend`
  replaces LazyCells' induction with the open kernel on those 228 and is production on the other
  203. `open` also takes the 155 run-route blocks LazyCells refuses.
- **On text,** every block is on the run route and all 48 are over the line. `open` decodes all
  48 where production falls back. `opend` is production, block for block.
- **On random,** no block is on the run route. Both arms are production, block for block.
- **So `opend`'s guards are an A/A check** of the wrapper and the pipeline: any guard loss there
  is noise or a fault in the arm, not the kernel.

## The cells

Unpinned, every online CPU, no spinner. Unpinned, the occupancy arms build the same decoder
whatever the threads asked; the threads set only the reference's plan.

| threads | reference (0.5.6) | judged | context | why |
|--:|---|---|---|---|
| 10 | `mibz2`: lazy 7/28 | `open-7-28`, `opend-7-28`, `opend-8-32` | `oce-7-28` | the shipped knee at 10–20 threads |
| 8 | `mibz2`: lazy 8/32 | `open-8-32`, `opend-8-32` | `oce-7-28` | the shipped knee at 8 threads |

- **`opend-8-32` at 10 threads:** the recalibration measured lazy 8/32 there at 0.963–0.974 of
  7/28's wall, for 13–14% more peak RSS, which rule 4 held. The open kernel's smaller state is
  the one lever that could bring that count within the memory gate. That is the checkpoint's
  question at the contended count, so it is judged.
- **Not run:** `open-8-32` at 10 threads. The census predicts that `open` loses on text (below),
  and the time goes to the arms that can answer.
- **The context arm at both counts:** with `--sparse`, a context arm must run at every count.

## Method

- **Corpora:**
  - Wikidata ×4: four copies of the pinned sample, the gain family;
  - text and random: the guards, generated afresh by `ab gen` in this run.
- **Timing:** one `ab run` a count, the reference first in the arm list, 12 paired rounds. `ab
  run` rotates the order round by round.
- **The settle gate:** 2 s (the checkpoint's; LOG 6978881191 item 2). The recalibration's
  0.25 s gate left its ten-core cells bimodal. The admit run with the 2 s gate held ±1.4%
  intervals at 10 threads.
- **The clock state:** turbo, the state production decodes are in on the i9.
  - *What showed it:* the sustained-load run's probe held 4.9 GHz for 120 s of back-to-back
    decoding at 120–146 W. Continuous load does not clamp this machine.
  - *The 2 s gate* keeps the cold-start state of the 10 November record
    (`docs/journal/2026-11-10-cold-start.md`) out of the timed decodes.
- **Records:** each decode's wall, CPU, peak RSS, output size and digest, and worker threads,
  as `ab run` writes them.
- **Time on the i9:** 324 decodes, each after the 2 s gate, about 13 minutes. With the build,
  the inputs, the census and the diagnostic, about 17 minutes.

## The rule

**`bench/two_arm.py`,** as merged in #562, one Holm family at α 0.05 over the 5 main-corpus
cells. The recipe's call is the registered one:
- `--ref-at mibz2=10,8 --sparse --unpinned --hold-max-threads 0`;
- `--judge-at` as in the table;
- `--context mibz2-oce-7-28`;
- `--guard text,random`;
- `--primary wikidata-x4,text`, the PERFORMANCE-POLICY rule 4 trade as amended (#558);
- `--check-workers`. Every record of an open arm must name exactly its N workers
  (`mibz2-0` … `mibz2-{N-1}`). The regex now accepts `mibz2-open-N-S` and `mibz2-opend-N-S`,
  tested in `bench/test_two_arm.py`.
- `--ref-workers 10=7 --ref-workers 8=8 --machine-model i9-10900K --lscpu lscpu.txt`: reference
  admission, below.

It is VOID, and nothing is read from it, where:
- a record is missing, malformed or duplicated;
- a corpus's outputs differ between any two arms;
- an arm names other workers than its own;
- the machine is not the registered one: its topology is not 10 cores and 20 CPUs, or its
  `lscpu` output does not name an i9-10900K (`--machine-model i9-10900K --lscpu`);
- **the reference is not the registered decoder:** a `mibz2` record at 10 threads that does not
  observe exactly 7 worker threads (`mibz2-0` … `mibz2-6`), or one at 8 threads that does not
  observe exactly 8 (`--ref-workers 10=7 --ref-workers 8=8`). A record with no thread census is
  missing evidence and VOIDs too.

**Reference admission** (the Researcher's targeted correction, REVIEW 6990885206, accepted in the
Coordinator's LOG 6990984237):
- *Why:* `--check-workers` reads an arm's workers from its name, and `mibz2` names none. Before
  this correction, the exact scoring call gave ADOPTS on 324 synthetic records whose 72 reference
  records each observed one worker.
- *The evidence for 7 and 8:* `tests/recalibrate_arms.rs` checks the unpinned defaults
  deterministically with the i9's facts. On the i9 itself, `mibz2` observed 7 workers at 10
  threads unpinned (the unpinned run, run-20261117T130353Z) and pinned (0.5.6's release scaling,
  run-20261118T201717Z). No i9 bundle runs the default unpinned at 8 threads: there the 8 rests
  on the test, and an i9 that builds anything else VOIDs the run.
- *The test:* `bench/test_two_arm.py` replays the Researcher's case, the recipe's own arguments
  on 324 records. With the reference's workers checked it is VOID at both counts, and without the
  check it still scores ADOPTS, which is the fault the correction closes. The other negative
  cases: 8 workers at 10, one record with no thread census, another CPU, no `lscpu`, an `lscpu`
  with no model name, and malformed options.

Otherwise each judged cell has the policy's verdict, from `two_arm.py`'s own lines:
- ADOPTS: a gain (Holm), no SLOWER cell, memory within the gate;
- ADOPTS BY TRADE;
- A FINDING, NOT ADOPTED;
- HELD ON MEMORY;
- NOT ADOPTED.

**What the verdicts decide:**
1. **No production default changes,** whatever the verdicts. Production stays 0.5.6's.
2. **An integration study follows** only if some cell ADOPTS or ADOPTS BY TRADE. It is the open
   kernel made production-grade under B5, with the admission of the arm that adopted:
   - if both `open` and `opend` adopt at a count, the one THE CHOICE names, by the lowest
     geometric mean of its main-corpus step;
   - if `opend-8-32` adopts at 10 threads, the study also registers that count.

   The study ends in its own adoption registration against master, and it is the Coordinator's
   to dispatch.
3. **A FINDING or HELD ON MEMORY:** reported with the census, and nothing follows from this run.
   Where `open` gains on Wikidata but is SLOWER on text, the census says why: those are the
   blocks over the density line.
4. **Otherwise:** stop. The open decoder stays research code in the journal, and this run's
   write-up says so.
5. **VOID:** no reading. The bundle is kept, and any rerun is the Coordinator's decision.

C2 holds: none of this is relaxed after the data is seen.

## Diagnostics: decide nothing

- **The census,** above, recomputed on the run's inputs.
- **Peak RSS** per arm and corpus, in `two_arm.py`'s memory lines.
- **The memory controller and the package's energy:** `bench/counters.py --events imcpower`.
  - *What it counts:* `uncore_imc/data_reads/`, `uncore_imc/data_writes/` and
    `power/energy-pkg/`, system-wide, each rep after an idle baseline that is taken off.
  - *What it reports:* DRAM bytes per output byte, joules per GB, and the package's mean watts,
    for each arm and for the idle machine.
  - *How:* one counters file a pair, each open arm and the context against `mibz2` at its count
    (CHECKPOINT 6984075662's fix of the recalibration's VOID). 5 reps on Wikidata ×4.
  - *When:* only where `kernel.perf_event_paranoid` is 0 or less. Otherwise it is skipped, and
    the recipe says so.

## Predictions, frozen before the data

These are the Pathfinder's, from the census and from single runs in the cloud container (4
cores, not the i9). They are not evidence, and nothing is scored against them.

- **`open` is SLOWER on text,** at both counts, so it is not adopted.
  - *Why:* it decodes text's 48 over-line blocks itself, where production falls back.
  - *The container:* 0.31–0.34 s against production's 0.125–0.15 s at 7/28.
- **`opend` gains on Wikidata ×4,** about 0.90–0.95 of production's wall at 7/28.
  - *The container:* 0.24 s against 0.29 s.
  - *The VM:* 0.92 of LazyCells.
  - *Contention:* the smaller state should help more at 10 threads than at 4.
- **`opend`'s guards are within noise:** it is the same code there.
- **Peak RSS falls** in every open arm on Wikidata. The container gave 86 MB against 150 MB, so
  the memory gate is not in play for `opend-7-28`.
- **`opend-8-32` at 10 threads:** gain and memory both uncertain. It is the cell to watch.
- **DRAM bytes per output byte fall** with the open kernel on Wikidata. The package's watts are
  not predicted.
