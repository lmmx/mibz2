# Compact30 against the shipped walk: a traversal-comparison protocol (revision 4, for approval)

- **Status:**
  - Revision 3 repaired the harness as report 176 authorises, after the researcher's review
    (report 175).
  - Revision 4 is an analyser-only repair (reports 178 and 179). The calibration evidence
    G5 reads is now checked complete and well formed (S9) before any median is read. G5
    is worded as a calibration-based estimate, and two reporting limits are stated.
  - Checked on the VM without timing. Nothing has been timed, and nothing below is a
    performance figure.
- **The collector is unchanged** from `c0f8d38`: the Rust source, the walker, the
  synchronisation and the timing windows. Only `bench/c30_traversal.py`, its tests and this
  text differ at the new pin (§8).
- **Since revision 4, two preflight repairs (§8):** the recipe selects the reviewed
  toolchain (report 180), and G3 compares the function's own extent, leaving out the
  linker's alignment fill after it (report 182). Neither changes the collector or the
  analysis.
- **Not authorised:** the capture. The i9 command in §9 runs only once the guardian approves
  it.
- **The candidate:** the refined compact30 walker (`examples/c30walk.rs`, `room::<false>`),
  unchanged at its reviewed 21-instruction ordinary path. Its validation is in the walker
  journal's §8. The harness around it changes nothing in the walker: the function's listing
  is checked against the reviewed one, instruction by instruction (§5, G3).

**What revision 3 changes (report 176):**

| item | revision 2 (`b12b49b`) | revision 3 |
|---|---|---|
| the output check | inside each worker's run, before the completion barrier, so an early finisher read ~1.8 MB while its neighbours walked | after the completion barrier; records go into storage allocated before the step loop |
| a failure | an assertion panicked one worker and could strand the others at a barrier | a failure record is appended to the records file and the process exits at once; tested by injection |
| the worker cost | summed monotonic windows: elapsed time, named as CPU cost | the thread's CPU time (`CLOCK_THREAD_CPUTIME_ID`); the monotonic windows give completion time and ordering |
| the record checks | the parts checked separately; the round count taken from the records | the records checked against the whole registered schedule before any ratio (§5, S); 20 measured rounds enforced |
| the sample | one round gave a zero-width interval | anything but 20 measured rounds is refused |

The design is unchanged: the arms, the lockstep workload, the allocation, the 0.865 screen
and the 1.02 tolerance.

## 1. The question

At four cores, does compact30's whole traversal cost enough less CPU than the shipped
walk's to justify the investment in a direct constructor, without making lockstep traversal
completion more than 2% slower?

**The traversal, on each side:**
- start selection;
- marking and restoration;
- the room walk with its exceptional paths and tail;
- assembly.

**What decides it, and what doesn't:**
- The decision reads two ratios against the shipped walk at four cores: the CPU cost w4
  and the completion time e4.
- The one-core ratio w1 is reported but does not decide.
- Beating FOR-14, or scaling better from one core to four, decides nothing (report 173).

## 2. The arms (one binary, both arms in the same process)

| arm | what is timed | the table it reads | written just before (untimed) |
|---|---|---|---|
| **P** (control) | `mibz2::bwt::walk_segments_room` | the shipped 4-byte T-vector | by `mibz2::bwt::build`, production's build, into the worker's `TVector` |
| **C** (candidate) | `C30::walk::<false>` | compact30 records and RAW groups | by a copy of the block's oracle-built image into the worker's arena |

**What P times:**
- production's 511 random starts plus the first;
- START flags set and cleared in the table;
- the 32-lane room walk with L1 prefetch;
- the tail and `SegScratch::assemble`.

This is the function the decoder calls (`Segmented::invert` is `build` then
`walk_segments_room`), unchanged.

**What C times:**
- the same random starts mapped through φ, plus φ(orig_ptr);
- saving and marking the starts, and restoring them afterwards;
- the room walk with RAW parking and resumption;
- the tail and assembly.

**The matched boundary:** each arm's window opens after its table is written and closes
when the block's inverse-BWT output is assembled.
- **Excluded from both arms:**
  - P's build, and C's oracle build and copy;
  - the barrier waits;
  - the output check;
  - MTF, RLE1, CRC and the scanner.
- **Each worker keeps its own table, arena and scratch** across blocks and rounds, as a
  decoder's worker does. So after round −1 neither arm's window includes first-use
  allocation.

**The interpretation's limit (report 174's wording):**

> This compares the measured traversal after each arm's specified table preparation. It
> does not isolate the table representation under identical cache state, nor reproduce the
> cache state of a future direct compact30 constructor.

- **P's table** is left in cache as production's build leaves it: a 4-byte-a-row scatter
  over the block's T-vector.
- **C's table** is left as a sequential copy leaves it: a stream of 64-byte records, then
  the RAW groups.
- A direct constructor would leave yet another state. Nothing here measures construction
  cost, or how a constructor's writes would warm or cool the walk.

**The output** is the same inverse-BWT bytes, by production's convention.
- Every walk, in every round, both arms, is compared with production's own
  `Segmented::invert` output for that block.
- The comparison runs after the step's completion barrier (§3).
- A difference ends the run as a failure (§3).

## 3. The workload, the order of work, and failure

**The corpus:** the primary `wikidata-20261026.bz2` (SHA-256 `cf13f439…`), decoded once to
each block's last column and counts, before any timing.
- 434 blocks; 207,006,515 rows.
- 230 full blocks (at least 899,000 rows).
- The 204 short blocks hold 8,324 rows between them (0.004% of the rows; median 23 rows).
  They are 47% of the walk windows, which matters for the clock calls' cost (§4).

**One core (W1):** one worker, pinned to the first listed CPU. Every block in order, under
one arm, then every block under the other. For each block it:
1. prepares the table;
2. walks it (timed);
3. checks the output;
4. records the walk.

**Four cores (W4), lockstep by block:** four workers, each pinned to its own physical core.
Before its first step, each allocates storage for all its records in the pass. Then, for
each step, every worker:
1. prepares its next block's table (untimed);
2. waits at the start barrier;
3. walks the block (timed);
4. waits at the completion barrier;
5. checks its output and writes its record into the preallocated slot;
6. goes on to step 1 of the next step.

**What that order guarantees:**
- No check, record or next preparation runs while any walk of the step runs. An early
  finisher waits at the completion barrier.
- No walk starts until every worker has prepared.
- A worker with no block left in a step still waits at both barriers.
- The records file is written between rounds, when no walk runs.

**Uneven finishes are a result.** The four walks start together, but the faster ones end
first, and the slower ones finish with fewer neighbours walking. So a measured window is
not a window spent beside three other walks throughout. The overlap is reported, and
decides nothing.

**Failure, from any worker or the main thread:**
- **Triggers:** the explicit harness failures (an output difference, a pinning failure, a
  clock failure) and Rust panics, through a panic hook.
- **Not covered:** a process abort, a signal, an out-of-memory kill or any other
  termination from outside. Such a run leaves no failure record. Its records file is
  incomplete, so S3 rejects it, and the recipe still analyses and archives it.
- **What it does:** it appends a `failure` record, naming the round, mode, arm, worker,
  step, block and the first differing byte, to the records file. It prints it, and the
  process exits with status 3.
- **Why exit:** no worker can be left waiting at a barrier for one that has stopped. This
  needs no new concurrency machinery.
- **What remains:** the records of every finished round stay in the file.
- **What the recipe does after:** it still runs the analysis, which reports the failure
  and rejects the run. It archives everything, and exits with the run's status. A timeout
  (one hour) kills a run that hangs for any reason, and the archive still follows.

**Fault injection,** dry runs only (the harness refuses it otherwise; the analysis rejects
a capture carrying it):
- `--inject output:MODE:ARM:WORKER` flips the first byte of that worker's first output in
  round −1. It acts after the completion barrier and before the check.
- `--inject pin:MODE:WORKER` pins that worker to CPU 1023, which doesn't exist, in round
  −1.

**The allocation, deterministic and recorded:**
- The blocks are sorted by rows, descending, ties by index.
- They are dealt four at a time, one to each worker, the direction reversing each step (a
  snake). Step s holds the blocks ranked 4s to 4s+3.
- The harness writes the four lists, each block's rows and each block's RAW-group count
  into the meta record. The analysis re-derives the lists from the rows and requires them
  equal (S1).

**The allocation on the primary corpus** (corpus facts, not timing):

| worker | blocks | full blocks | rows | RAW groups | RAW groups a million rows |
|--:|--:|--:|--:|--:|--:|
| 0 | 109 | 57 | 51,301,938 | 12,232 | 238.4 |
| 1 | 109 | 57 | 51,301,991 | 12,316 | 240.1 |
| 2 | 108 | 58 | 52,201,043 | 12,576 | 240.9 |
| 3 | 108 | 58 | 52,201,543 | 12,509 | 239.6 |

- Blocks are indivisible, and 230 full blocks don't divide by four. So two workers carry
  one full block more: the rows' max ÷ min is 1.0175, and their difference (899,605) is
  less than one block.
- Step 57 holds two full blocks and two short ones; every other step is all full or all
  short. The last step, 108, holds two short blocks (workers 0 and 1).
- The RAW groups a million rows are within 0.6% of their mean on every worker.

**Rounds:** 20 measured paired rounds plus an untimed round −1.
- **A round** is W1 under each arm, then W4 under each arm, in the same arm order.
- **The order alternates:** even rounds are P first, odd rounds (round −1 included) C
  first.
- **Round −1** warms the binary and the workers' tables and scratch, and checks every
  output. It is not used.
- **The clock calibration** runs before round −1 and after round 19 (§4).
- No timing change rides on any other run, and no workstation setting is changed.

## 4. The measures

**Three clocks, three uses:**

| quantity | clock | use |
|---|---|---|
| a walk's CPU time | `CLOCK_THREAD_CPUTIME_ID`, the calling thread's consumed CPU | w1 and w4, the CPU-cost ratios, and the constructor budget |
| a walk's start and end, and every preparation and check window | `std::time::Instant`, monotonic | e4, the start skew, the overlap and every ordering check |
| a walk's CPU identity | `sched_getcpu`, before the start barrier and after the completion barrier | placement (S4) |

**The window's clock calls, in order:**
1. the monotonic clock;
2. the thread CPU clock;
3. the walk;
4. the thread CPU clock;
5. the monotonic clock.

The CPU window lies inside the monotonic one. Each holds the part of its two clock calls
between the readings. The barrier waits, the preparation and the check are outside both.

**The clock calls' cost (G5), a calibration-based estimate:**
- **The calibration:** before round −1 and after round 19, each of the four CPUs runs
  20,000 empty windows: the same calls in the same order, around nothing. Their quantiles
  are recorded.
- **What is kept:** the raw walk windows. Nothing is subtracted from the figures.
- **The evidence, checked first (S9):**
  - exactly one `before` and one `after` batch;
  - in each, exactly the four CPUs of the run, once each;
  - 20,000 windows a CPU;
  - for both the CPU and the monotonic clock, five finite, nonnegative, non-decreasing
    quantiles.

  Missing, duplicated, misidentified or malformed calibration makes the run invalid
  before any median is read. It can never yield a smaller estimate.
- **The estimate:** the largest median on any CPU, in either batch. That is a typical cost
  of the calls, not a bound on every call or on the perturbation of any one walk.
- **The screen:** that estimate, times the windows in a pass, must be at most 0.5% of the
  pass's summed walk CPU time, in every arm, mode and round.
- **Why 0.5%:**
  - Both arms have the same number of windows, so the calls add about the same CPU time
    to each.
  - That pulls the ratio toward 1 by about that share: at most 0.005, against the 0.135
    the screen asks for.
- **Reported, deciding nothing:**
  - the same share over the short blocks alone. The calls may be a large part of a
    23-row walk, but those walks hold 0.004% of the rows;
  - w4 with the median subtracted from every window, labelled a sensitivity figure.

**Per arm and round** (rounds 0–19):
- **W1 cost:** Σ walk CPU ns ÷ Σ rows.
- **W4 cost:** Σ over the four workers of walk CPU ns ÷ Σ rows. This is the aggregate worker
  traversal CPU time.
- **W4 completion time:** Σ over steps of (latest traversal end − earliest traversal start),
  on the monotonic clock.
  - It is the summed lockstep traversal completion time.
  - It starts at each step's earliest recorded traversal start, not at the barrier's
    release, which isn't recorded.
  - It leaves out the preparations, checks and barrier waits between steps. So it is not
    the capture's wall time, nor a decoder's throughput.

**The ratios,** paired by round, C ÷ P:
- **w1:** one-core CPU cost (reported).
- **w4:** four-core CPU cost. The construction-budget screen reads this one.
- **e4:** four-core lockstep completion time. Above 1 is slower.

**The estimate:** the geometric mean of the 20 round ratios, with a 95% interval from a
t interval (19 degrees of freedom) on their logs. The analysis computes no interval from any
other number of rounds.

**Reported, deciding nothing:**
- each arm's CPU ns a row at W1 and W4, and its W4 ÷ W1 scaling;
- the elapsed-window versions of w1 and w4 (`w1_wall`, `w4_wall`);
- the running fraction, CPU ÷ elapsed, per arm, mode and round. Well below 1 means a
  worker lost the CPU during its walks;
- each worker's summed CPU time;
- the start skew and the common overlap of the four walks in the all-full steps.

**Why e4 can differ from w4:**
- e4 counts each step's slowest walk; w4 counts all four.
- A candidate whose walk time varies more between blocks of the same size raises e4 above
  w4. RAW-heavy blocks would do that.
- So a candidate can cost less CPU and still complete lockstep traversal no sooner.

## 5. The checks, in order

The analysis runs three stages, and a stage runs only if the one before it passed.

### Stage 1: the schedule (S), before any figure

The expected record set is derived from the protocol and the block rows the meta record
holds, not from the walk records.
- **Its limit:** `block_rows` is the collector's own reading of the corpus. It is not
  re-derived by decompressing the corpus again.
- **What it rests on:** the pinned collector and the corpus hash, which the recipe checks
  before the run.

| check | condition |
|---|---|
| S0 | no `failure` record, and no unknown record kind |
| S1 | the meta record agrees with itself and the protocol: the block count, rows and full blocks from `block_rows`; the four lists equal to the snake re-derived from `block_rows`; the per-worker rows; four distinct CPUs |
| S2 | the labels: a capture has 20 measured rounds and no injection; a dry run has 2 |
| S3 | the record set is exactly the expected one. Each (round, mode, arm, worker, step) appears once, with its registered block and that block's rows; nothing is missing, extra or duplicated; each W4 step holds exactly its declared workers (only step 108 is partial, with workers 0 and 1) |
| S4 | every walk's CPU, before and after, is its worker's listed CPU (W1: the first) |
| S5 | in each walk: preparation ≤ walk ≤ check, in that order, and its CPU window is no longer than its monotonic window (2 µs of clock noise allowed) |
| S6 | W1: each check ends before the next preparation starts |
| S7 | W4: in each step, every preparation ends before any walk starts (the start barrier); every walk ends before any check starts (the completion barrier); every walk of a step ends before any preparation of the next starts; each worker's check ends before its next preparation |
| S8 | the passes run in the registered order (rounds in turn; in each, W1 then W4, each in the round's arm order) and don't overlap |
| S9 | the calibration evidence is complete and well formed (§4): one `before` and one `after` batch; in each, the run's four CPUs once each, 20,000 windows a CPU, and five finite, nonnegative, non-decreasing quantiles for each clock |

If any S check fails, the run is invalid and no ratio is computed. An invalid record set
never reaches an ADVANCE, STOP or INCONCLUSIVE decision.

### Stage 2: the validity gates (G), on a schedule that passed

| gate | condition |
|---|---|
| G1 stability | P's per-round figure, max ÷ min ≤ 1.05, for the W1 CPU cost, the W4 CPU cost and the W4 completion time |
| G2 simultaneous start | in every all-full W4 step, the start skew (latest start − earliest start) over the step's span, row-weighted per arm and round, ≤ 0.02, in every arm and round |
| G3 identity | the corpus hash matches, and `room::<false>`'s listing in the built binary is the reviewed one (`bench/asm_listing.py`). The function is its symbol's extent, start to start + size (the symbol table's size, which must be the reviewed 0x20aa). Every instruction in it is compared in order with its operands, the function's own alignment NOPs included; branch targets are taken relative to the function, callees by symbol, and RIP displacements are masked. The bytes from the function's end to the next symbol are the linker's alignment fill: they are not compared, and each must be `int3`, a `nop` or `xchg ax,ax`, or G3 fails (report 182). If G3 fails, there is no run |
| G4 allocation | per-worker rows differ by at most the largest block, and full-block counts by at most one |
| G5 clock calls | §4, on calibration S9 has checked: the calibration-based estimate of the calls' share ≤ 0.5% in every arm, mode and round |

**G2 checks the barrier, not the walks' lengths.** Uneven walks are a result, which e4
carries.

**The binary's own hash** is recorded in the manifest. It is not a gate: the line tables
hold build paths, so two builds of one tree differ in bytes. The listing is the identity.

### Stage 3: the figures and the decision (§6), on a valid run only

**A dry run** (rounds −1, 0 and 1) runs stage 1, G3 and G4. It suppresses every timing
figure, G1, G2, G5 and the decision.

## 6. The construction budget and the decision rules (fixed before any run)

**The planning model** (report 165's, from the earlier phase reconstruction):

> S ≈ 0.592(1 − b) + 1.847(1 − w) CPU-s per decoded GB

- **b** is a direct constructor's cost relative to production's build.
- **w** is the traversal-plus-assembly CPU cost relative to the shipped walk's.
- **Its assumptions:**
  - the coefficients are approximate phase costs, not constants;
  - the rest of the decoder is unchanged;
  - traversal costs measured on this boundary carry into the decoder.

  The last could fail if construction changes the traversal's cache state (§2), or phases
  overlap differently.

**The planning target:** S ≥ 0.25 CPU-s/GB. That is about 7% of the decoder's present CPU
cost, and smaller than the 1.5–2× aim.

**The construction budget a measured w leaves:**

> b_max(w) = 1 + (1.847(1 − w) − 0.25) / 0.592

| w | b_max: a constructor may cost up to this × production's build |
|--:|--:|
| 0.70 | 1.51 |
| 0.75 | 1.36 |
| 0.80 | 1.20 |
| 0.8646 (exact) | 1.000 |
| 0.865 (the screen) | 0.999 |
| 0.90 | 0.89 |
| 1.00 | 0.58 |

**What 0.865 is, and what it isn't:**
- 0.865 is the registered **minimum-investment screen**, rounded. Exactly, an unchanged
  constructor (b = 1) meets the target at w = 1 − 0.25/1.847 = 0.86465. At 0.865 itself
  the budget is 0.9989.
- Below the screen, compact30 meets the target with a constructor about as costly as
  production's build. The budget at the interval's ends is reported, not assumed to be 1.
- Above it, meeting the target needs a constructor faster than production's build. That
  is not impossible or inherently incredible, but it is a larger, less certain investment.
- **Missing the screen may justify STOP as an engineering investment decision. It is not a
  proof of architectural impossibility.**

**The constructor budget, reported in every outcome** (STOP and INCONCLUSIVE included): b_max
at w4's lower end, point and upper end.

**The completion-time tolerance:** e4's upper end ≤ 1.02.
- **What it permits:** up to 2% more lockstep traversal completion time than the control. It
  is a predeclared regression allowance, not a proof that the candidate trades no speed
  for CPU.
- **Why 2%:**
  - A strict 1.00 would turn an equal-time candidate into INCONCLUSIVE on noise alone.
  - 2% is well inside the 13.5% CPU saving the screen asks for.
  - It was set from the design, before any i9 timing of these arms.

**The decision rules,** on the 95% intervals [w4−, w4+] and [e4−, e4+]:

1. **ADVANCE:** w4+ ≤ 0.865 and e4+ ≤ 1.02.
   - Even at the interval's worst, the traversal's CPU saving leaves a constructor a budget
     of b_max(w4+), at least 0.9989 of production's build cost.
   - Lockstep completion time is no more than 2% worse.
   - This authorises a separately registered constructor design, with budget
     b ≤ b_max(w4+). It does not authorise integration.
2. **STOP:** w4− > 0.865, or e4− > 1.02.
   - **On cost:** even at the interval's best, meeting the target needs a constructor
     cheaper than production's build. Compact30 stops at this checkpoint as an investment
     decision. The record states the budget it would have needed, b_max(w4−), and that a
     faster constructor is not ruled out.
   - **On completion time:** lockstep completion is slower by more than 2% even at the
     interval's best, whatever the CPU saving.
   - The record and the code stay, with no further compact30 work.
3. **INCONCLUSIVE:** otherwise, when either interval straddles its threshold and neither
   STOP condition holds.
   - No automatic rerun and no extra rounds.
   - Report w4, e4, both intervals, b_max at w4's ends and point, and w1. The guardian
     decides.
4. **INVALID:** any S check, gate or failure.
   - S failures or a failure record: no figure is computed.
   - A gate failure: the figures are reported as an invalid run's, with no decision.

**Read in every outcome, deciding nothing:** the figures listed in §4.

## 7. What it cannot show

- **A production-decoder result:** MTF, RLE1, CRC, the scanner and the pipeline are absent.
- **Construction cost**, and **the cache state a direct constructor would leave** (§2).
- **The decoder's four-core mix:** lockstep puts walks side by side. A decoder's free
  workers overlap one worker's build with another's walk, which this design excludes.
- **Sustained four-way contention:** within a step the faster walks end first (§3).
- **Peak RSS** or any memory measure.
- **Other machines or corpora.**
- **Whether a better compact30 schedule exists.** The code is fixed at the reviewed listing.
- **Why a result came out as it did.** No counters are collected. A counter capture would
  be a separate registration.

## 8. The VM checks (no timing)

All at the pin `c0f8d38`, through the recipe itself, on the VM's four CPUs. No timing figure
was read or is reported.

**Correctness and the regression suite:**
- The analysis's 34 regression tests pass, on synthetic records. They cover:
  - each decision, on a valid run: ADVANCE; STOP on cost; STOP on completion time;
    INCONCLUSIVE on either;
  - each gate: an unstable control (G1), a staggered start (G2), a listing mismatch (G3),
    the clock calls' share and a missing calibration (G5);
  - report 175's five cases, each now rejected before any ratio: one measured pair;
    workers shifted off their lists; the candidate's rows doubled (that set's unaltered
    version is a STOP); a full step split; a next preparation inside a prior walk;
  - the measurement boundary: a check inside a peer's walk; a preparation after a peer's
    start; a W1 check overlapping the next preparation; a CPU window longer than its walk;
  - the record set: a missing, duplicated or unexpected walk; lists other than the snake; a
    wrong CPU; the arm order swapped; a capture's or dry run's wrong labels; an injected
    capture;
  - failures: a failure record, and a failed run's leftover records;
  - an uneven schedule (one worker finishing far earlier) being valid, with its checks
    after the step's last walk;
  - the cost being CPU time: a descheduled candidate's stretched elapsed windows leave w4
    unchanged and show in `w4_wall` and the running fraction;
  - the interval refusing every sample size but 20.
- **Each schedule check is load-bearing.** With each S check forced to pass in turn, the
  tests aimed at it fail:
  - S3 is caught by seven tests;
  - S2 by two;
  - every other S check by one.
- **The listing check** passes on the pin's binary and fails on a reference with one
  operand changed (revision 2). G3 held in every run below.

**The three runs** (`docs/journal/2026-11-02-compact30-traversal-protocol/runs-c0f8d38/`):

| mode | grid exit | what the analysis found | archive |
|---|--:|---|---|
| `dry-run` | 0 | every S check passes (all 5,208 walks are the expected records, the barriers' and passes' order holds, placement holds); G3 and G4 hold | yes |
| `fail-output` | 3 | the failure: round −1, W4, arm C, worker 2, step 0, block 7, differing at byte 0. INVALID, no figures | yes |
| `fail-pin` | 3 | the failure: W4 worker 1 could not pin to CPU 1023. INVALID, no figures | yes |

- **Each failed run ended at once.** The process exited while the other three workers
  waited at a barrier. The failure was the records file's last line, and the recipe still
  analysed and archived it.
- **What is kept for each run:**
  - the console;
  - the analysis JSON (no timing in any of them);
  - the listing diff (empty);
  - the manifest, naming the pin, the corpus and binary hashes, the command, the exit
    status and every file's checksum.
- **Not kept:** the records files. They hold VM timestamps and calibration figures.

**The source identity:**
- the pin `c0f8d38`, with no tracked changes at any run;
- rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6;
- baseline x86-64 (fxsr, sse, sse2), no RUSTFLAGS;
- the release profile (codegen-units = 1, fat LTO, line tables).
- `room::<false>` in the pin's binary is identical to the reviewed listing
  (`2026-11-02-compact30-walker/room-false.asm`) under G3's normalisation.

### Revision 4, at the pin `9267433`

The collector is the same source as at `c0f8d38` (no change under `examples/`, `src/` or
the manifests); the analyser, its tests and this text changed.

- **The regression suite:** 37 tests pass. Beyond revision 3's 34:
  - report 178's case is rejected: one CPU's calibration is far above the others', and
    deleting it from both batches made the pinned analyser pass G5 and give ADVANCE;
  - 24 malformed-calibration cases are rejected, none raising:
    - a missing, extra, doubled or mislabelled batch;
    - an empty batch, or `cpus` not a list;
    - a missing, doubled, extra or wrong CPU;
    - a wrong `n`, or `n` as a bool;
    - quantiles too few, negative, NaN, infinite, unordered, a string or a bool;
    - a missing key, or an entry or batch not an object;
  - any CPU order is accepted;
  - a dry run checks calibration too.
- **S9 is load-bearing:** with it forced to pass, the three calibration tests fail.
- **Unchanged on complete calibration:** in 50 fixture variants the results equal the
  `c0f8d38` analyser's, apart from the added S9 entry. The variants cover ADVANCE, STOP,
  INCONCLUSIVE, gate failures and a dry run (`revision-4/`).
- **The recipe's three runs** (`runs-9267433/`):

| mode | grid exit | what the analysis found | archive |
|---|--:|---|---|
| `dry-run` | 0 | every S check passes, S9 included, on the real calibration; G3 and G4 hold | yes |
| `fail-output` | 3 | the failure recorded (round −1, W4, C, worker 2, block 7); S0, S3 and S9 fail (only the `before` calibration exists); INVALID, no figures | yes |
| `fail-pin` | 3 | the failure recorded (W4 worker 1, CPU 1023); S0, S3 and S9 fail; INVALID, no figures | yes |

- **`room::<false>`** is the reviewed listing in every run.

### The first i9 attempt, and the toolchain repair (report 180)

- **The attempt:** the approved capture was run on the i9 with the recipe at `515ab5c`.
  - The corpus check, the source pin and the 37 tests passed.
  - The workstation's default toolchain, rustc 1.98.0 with LLVM 22.1.8, built a different
    `room::<false>`, with changed register choices and branch layout.
  - G3 stopped the run before any timing.
- **What it is:** a G3 preflight failure, not a compact30 result. Its archive
  (`c30-traversal-20261102T170319Z.tar.gz`, sha256 `81545030…`) is kept by the maintainer.
- **The cause:** the recipe recorded the reviewed toolchain but didn't select it.
- **The repair:** the recipe now selects rustc 1.97.0 explicitly (§9, step 2). Nothing else
  changes:
  - the source pin stays `9267433`;
  - G3, the walker, the collector, the design, the thresholds and the corpus are as they
    were.
- **Checked on the VM, no timing read** (`runs-9267433-toolchain/`):
  - **The reviewed toolchain is used even when it isn't the default.** The dry run was made
    with `RUSTUP_TOOLCHAIN=1.98.1` set, imitating a machine whose default is newer. It still
    built with 1.97.0 / LLVM 22.1.6, G3 passed, and every schedule check passed.
  - **A missing toolchain is refused before building,** with the install command.
  - **A toolchain named 1.97.0 that is really 1.98.1 is refused before building,** with the
    reported identity.
  - Both refusals are archived.

### The second i9 attempt, and the G3 boundary repair (report 182)

- **The attempt:** the approved capture was run on the i9 with the recipe at `0d85566`.
  - The corpus check, the source pin, the toolchain identity (rustc 1.97.0, `2d8144b78`,
    LLVM 22.1.6) and the 37 tests passed.
  - G3 stopped the run before any timing.
- **What G3 found:** the same function, followed by different linker fill. Its files are
  kept in `i9-attempt-2/`.
  - The first 1,552 normalised instructions match the reference exactly, the 21-instruction
    ordinary path included. Both end with the call to `_Unwind_Resume`.
  - After that call, the reference has six `int3` and the i9's binary one 6-byte `nop`.
    Those are the bytes from the function's end to the next symbol's 16-byte alignment,
    filled differently by the two machines' linkers.
  - G3 took the function to end at the next symbol, so it compared the fill:
    `1552: reference 'int3' / this binary 'nop WORD PTR [rax+rax*1+0x0]'` and lengths
    1,558 against 1,553.
- **What it is:** a G3 preflight failure, not a compact30 result. Its archive
  (`c30-traversal-20261102T184101Z.tar.gz`, sha256 `fe6ad4e5…`) is kept.
- **The evidence for the boundary:** the symbol table gives `room::<false>` a size of 0x20aa
  in the VM's 1.97.0 build of the pin. That build is byte-identical to the one in the
  toolchain repair's dry run (binary sha256 `82c3a685…`), and its listing is the
  reference's. 0x20aa from each function's start lands exactly on the first fill byte:
  - the reference: 0x14ed0 + 0x20aa = 0x16f7a, the first `int3`;
  - the i9: 0xcbd0 + 0x20aa = 0xec7a, the `nop`;
  - the VM: 0x18b40 + 0x20aa = 0x1abea, the first of its six `int3`.
- **The repair** (`bench/asm_listing.py`), G3 only:
  - the function is the symbol's extent, start to start + size, with the size from the
    binary's own symbol table (`nm -S`). That size must equal the reviewed 0x20aa;
  - every instruction in the extent is compared as before, with the same normalisation.
    On the reference's 1,552 instructions it gives the same text as the old one;
  - the bytes after the extent, up to the next symbol, are not compared. Each must be
    `int3`, a `nop` of any width or prefix, or `xchg ax,ax`. Anything else there fails G3,
    as does an elided run (`...`) or the lack of an instruction boundary at the function's
    end. The reference's own fill is held to the same rule;
  - the function's internal alignment NOPs are part of the function and stay compared;
  - each run also keeps `room-false.bounds`: the extent and the fill on both sides.
- **Its tests** (`bench/test_asm_listing.py`, 15, run by the recipe before G3):
  - the regression: the archived i9 listing passes, with its one `nop` reported as fill;
  - the old boundary reproduces the archived `room-false.diff` line for line;
  - the i9's 21 ordinary-path instructions normalise to the reference's;
  - G3 still fails when:
    - any one of the 21 ordinary-path instructions is replaced, by a `nop` or by a real
      instruction;
    - the ordinary step's closing branch targets another offset;
    - the last call is changed, or replaced by a `nop`;
    - one of the function's own alignment NOPs is changed, swapped or dropped;
    - a real instruction (`ret`, `jmp`, `push`, `ud2`, `int 0x3`, `mov`) is in the fill;
    - the fill has an elided run, or an unrecognised line is inside the function;
    - the size is anything but 0x20aa, or ends inside an instruction;
    - the reference's size would leave its last call in the fill;
  - the fill pattern accepts objdump's alignment forms and rejects other instructions.
- **Unchanged:** the walker, the collector, the analysis and its 37 tests, the design, the
  thresholds, the corpus and the pinned toolchain.

## 9. The records and the command

**The recipe** (`just bench-c30-traversal`) runs the frozen commit in its own worktree
(`target/c30-traversal-pin`):
1. the corpus hash;
2. the toolchain: rustc 1.97.0 (commit `2d8144b78`, LLVM 22.1.6), selected with
   `rustup run 1.97.0`, so the machine's default and any `rust-toolchain` file are left
   alone. The recipe stops before building if rustup or that toolchain is missing (with the
   install command), or if the toolchain it finds doesn't report that commit and LLVM;
3. the build, with that toolchain, rustflags forced empty (`CARGO_ENCODED_RUSTFLAGS=`, so no
   `target-cpu` or other flag from a cargo config applies), `--locked`, into
   `target/rust-1.97.0/`;
4. the analysis's regression tests, and G3's (`bench/test_asm_listing.py`);
5. G3, and no run if it fails;
6. the CPUs: the second to fifth physical cores (the first takes more interrupts), or the
   first four where there are only four;
7. the capture, under a one-hour timeout;
8. the analysis, whether the capture passed or failed;
9. the archive, whether the run passed or failed.

**Records:** `target/c30-traversal-<UTC time>[-mode]/`, archived as
`target/c30-traversal-<UTC time>[-mode].tar.gz` with a `MANIFEST.json` by the convention of
report 142: the commit, the corpus and binary hashes, the command and its exit status, and
every file's checksum. It holds:
- the console;
- the toolchain;
- the listing, its diff, and its bounds (the function's extent and the fill after it);
- the records:
  - the meta line, with the assignment, each block's rows and RAW groups, and any
    injection;
  - both calibrations;
  - one line per walk, with its preparation, walk (monotonic and CPU) and check windows
    and its CPUs;
  - a failure line, if any;
- the analysis JSON.

**i9 time:** a rough estimate, not a measurement. About five minutes:
- each round walks the corpus's 207 M rows once per arm at W1 and once at W4;
- each walk follows its untimed build or copy and precedes its untimed check;
- the oracle images are built once.

**The modes:** `capture`, `dry-run`, and the injected failures `fail-output` and
`fail-pin` (§3).

**The command for approval:**

```
just bench-c30-traversal
```
