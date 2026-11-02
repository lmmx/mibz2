# Compact30 against the shipped walk: a traversal-comparison protocol (revision 2, for approval)

- **Status:** revised as report 174 asks, with the minimum harness and recipe written and
  checked on the VM without timing. Nothing has been timed, and nothing below is a
  performance figure.
- **Not authorised:** the capture itself. The i9 command in §9 runs only once the guardian
  approves it.
- **The candidate:** the refined compact30 walker (`examples/c30walk.rs`, `room::<false>`),
  unchanged at its reviewed 21-instruction ordinary path. Its validation is in the walker
  journal's §8. The harness adds a `grid` mode around it and changes nothing in the walker:
  the function's listing is checked against the reviewed one, instruction by instruction
  (§5, G3).
- **What changed from the draft (`38b9a81`):**
  1. 0.865 is stated as an investment screen, not a proof, and the constructor budget is
     reported across the whole interval (§6).
  2. Four-core completed-work time (e4) is measured beside the worker cost (w4), with a
     no-harm rule (§4, §6).
  3. The four-worker allocation is fixed and recorded, and the cache-state limitation is
     stated (§3, §7).
  4. The four-worker design is lockstep by block, so both arms always walk four at once.
     G2 checks the simultaneous start instead of the windows' overlap.

## 1. The question

At four cores, does compact30's whole traversal cost enough less than the shipped walk's
to justify the investment in a direct constructor, without making completed four-core work
slower?

**The traversal, on each side:**
- start selection;
- marking and restoration;
- the room walk with its exceptional paths and tail;
- assembly.

**What decides it, and what doesn't:**
- The decision is on the absolute ratios against the shipped walk at four cores: the worker
  cost w4 and the completed-work time e4.
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

**The matched boundary:** each arm's timing starts after its table is written and ends
when the block's inverse-BWT output is assembled.
- **Excluded from both arms:** P's build, C's oracle build and copy, MTF, RLE1, CRC and the
  scanner.
- **Each worker keeps its own table, arena and scratch** across blocks and rounds, as a
  decoder's worker does, so neither arm's timing includes first-use allocation after round
  −1.

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

**The output** is the same inverse-BWT bytes, by production's convention. Every walk, in
every round, both arms, is asserted equal to production's own `Segmented::invert` output
for that block, outside the timed window. A wrong walk aborts the run.

## 3. The workload and the conditions

**The corpus:** the primary `wikidata-20261026.bz2` (SHA-256 `cf13f439…`), decoded once to
each block's last column and counts, before any timing.
- 434 blocks; 207,006,515 rows.
- 230 full blocks (at least 899,000 rows); the short ones use each arm's plain walk.

**One core (W1):**
- one worker, pinned to the first listed CPU;
- every block in order, under one arm, then every block under the other.

**Four cores (W4), lockstep by block:**
- four workers, each pinned to its own physical core;
- for each step, every worker:
  1. prepares its next block's table (untimed);
  2. waits at a barrier;
  3. walks the block (timed);
  4. waits at a barrier.
- So every timed walk starts with the other three workers' walks, of the same arm, and no
  build or copy overlaps a timed walk.
- A worker with no block left in a step still waits at both barriers.

**The allocation, deterministic and recorded:**
- The blocks are sorted by rows, descending, ties by index.
- They are dealt four at a time, one to each worker, the direction reversing each step (a
  snake). Step s holds the blocks ranked 4s to 4s+3.
- So the four walks of a step are of near-equal size, and every worker gets the same mix
  of full and short blocks.
- The harness writes the four lists, each block's rows and each block's RAW-group count
  into the run's meta record.

**The allocation on the primary corpus** (from the dry run; corpus facts, not timing):

| worker | blocks | full blocks | rows | RAW groups | RAW groups a million rows |
|--:|--:|--:|--:|--:|--:|
| 0 | 109 | 57 | 51,301,938 | 12,232 | 238.4 |
| 1 | 109 | 57 | 51,301,991 | 12,316 | 240.1 |
| 2 | 108 | 58 | 52,201,043 | 12,576 | 240.9 |
| 3 | 108 | 58 | 52,201,543 | 12,509 | 239.6 |

- Blocks are indivisible, and 230 full blocks don't divide by four. So two workers carry one
  full block more: the rows' max ÷ min is 1.0175, and their difference (899,605) is less
  than one block.
- That difference falls in step 57, where two workers walk a full block and two a short
  one. In every other step, the walks are all full or all short; the last step, 108,
  holds two short blocks.
- The candidate's exceptional path (RAW groups a million rows) is within 0.6% of its mean
  on every worker.

**Rounds:** 20 paired rounds plus an untimed round −1.
- **A round** is W1 under each arm, then W4 under each arm, in the same arm order. The
  order alternates by round: even rounds are P first, odd rounds are C first.
- **Round −1** warms the binary, the workers' tables and scratch, and checks every output.
  It is not used.
- No timing change rides on any other run, and no workstation setting is changed.

## 4. The measures

**Per arm and round** (rounds 0–19):
- **W1 cost:** Σ walk ns ÷ Σ rows.
- **W4 worker cost:** Σ over the four workers of walk ns ÷ Σ rows. This is aggregate worker
  traversal time.
- **W4 elapsed:** Σ over steps of (latest end − earliest start). This is each step's time
  from the barrier's release to its last completion, summed: the completed-work time of
  the whole corpus at four workers.

**The ratios,** paired by round, C ÷ P:
- **w1:** one-core cost (reported).
- **w4:** four-core worker cost. The construction-budget screen reads this one.
- **e4:** four-core completed-work time. Above 1 is slower.

**The estimate:** the geometric mean of the 20 round ratios, with a 95% interval from a
t interval (19 degrees of freedom) on their logs.

**Reported, deciding nothing:**
- each arm's ns a row at W1 and W4, and its W4 ÷ W1 scaling;
- each worker's summed walk time per round and arm;
- the four walks' common overlap in each all-full step;
- every walk's preparation and walk window, in the records.

**Why e4 can differ from w4:**
- e4 counts each step's slowest walk; w4 counts the mean.
- A candidate whose walk time varies more between blocks of the same size raises e4 above
  w4. RAW-heavy blocks would do that.
- So a candidate can cost less CPU and still finish four-core work no sooner.

## 5. Validity gates and measurement-path checks (read before any decision)

| gate | condition | if it fails |
|---|---|---|
| G0 output | every walk exact (asserted in the harness, so a finished run has none wrong), and every (round, mode, arm, block) present exactly once | invalid: no decision |
| G1 stability | P's per-round figure, max ÷ min ≤ 1.05, for the W1 cost, the W4 cost and the W4 elapsed time | invalid: no decision |
| G2 simultaneous start | in every all-full W4 step, the start skew (latest start − earliest start) over the step's span, row-weighted per arm and round, ≤ 0.02, every arm and round | invalid: no decision |
| G3 identity | the corpus hash matches, and `room::<false>`'s listing in the built binary is the reviewed one (`bench/asm_listing.py`: every instruction in order with its operands; branch targets relative, callees by symbol, RIP displacements masked) | no run |
| G4 allocation | every block in exactly one W4 list; per-worker rows differ by at most the largest block; full-block counts by at most one | invalid: no decision |

**The measurement-path checks** (all must hold, or the run is invalid):
- every walk's preparation ends before its walk starts;
- in every W4 step, every walk starts after all four workers' preparations end (the
  barrier);
- every walk ran on its worker's listed CPU (read after the walk);
- no W4 step holds two walks of one worker.

**G2 checks the barrier, not the walks' lengths.**
- The draft's overlap gate would have made a candidate's uneven walks an invalid run. That
  unevenness is a result, which e4 carries.
- So G2 asks only that the four walks start together.

**The binary's own hash** is recorded in the manifest. It is not a gate: the line tables
hold build paths, so two builds of one tree differ in bytes. The listing is the identity.

## 6. The construction budget and the decision rules (fixed before any run)

**The planning model** (report 165's, from the earlier phase reconstruction):

> S ≈ 0.592(1 − b) + 1.847(1 − w) CPU-s per decoded GB

- **b** is a direct constructor's cost relative to production's build.
- **w** is the traversal-plus-assembly cost relative to the shipped walk's.
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
| 0.865 | 1.00 |
| 0.90 | 0.89 |
| 1.00 | 0.58 |

**What 0.865 is, and what it isn't:**
- 0.865 is the registered **minimum-investment screen.** It is derived from the 0.25
  CPU-s/GB target with an unchanged constructor (b = 1).
- Below it, compact30 meets the target with a constructor no costlier than production's
  build.
- Above it, meeting the target needs a constructor faster than production's build. That
  is not impossible or inherently incredible, but it is a larger, less certain investment.
- **Missing the screen may justify STOP as an engineering investment decision. It is not a
  proof of architectural impossibility.**

**The constructor budget, reported in every outcome:** b_max at w4's lower end, point and
upper end. This includes STOP and INCONCLUSIVE.

**The throughput no-harm rule:** e4's upper end ≤ 1.02.
- **What it asks:** the candidate's completed four-core work is no slower than the
  control's, within 2%.
- **Why 2%:**
  - A strict 1.00 would turn an equal-throughput candidate into INCONCLUSIVE on noise
    alone.
  - 2% is well inside the 13.5% worker-cost saving the screen asks for. So a candidate
    passing both rules saves CPU and doesn't trade it for completed work.
  - It is set before any i9 timing of these arms, from the design, not fitted to a
    result.

**The decision rules,** on the 95% intervals [w4−, w4+] and [e4−, e4+]:

1. **ADVANCE:** w4+ ≤ 0.865 and e4+ ≤ 1.02.
   - Even at the interval's worst, the traversal leaves a constructor a budget of at least
     production's build cost (b_max(w4+) ≥ 1), and completed four-core work is not harmed.
   - This authorises a separately registered constructor design, with budget
     b ≤ b_max(w4+). It does not authorise integration.
2. **STOP:** w4− > 0.865, or e4− > 1.02.
   - **On cost:** even at the interval's best, meeting the target needs a constructor
     cheaper than production's build. Compact30 stops at this checkpoint as an investment
     decision. The record states the budget it would have needed, b_max(w4−) < 1, and that a
     faster constructor is not ruled out.
   - **On throughput:** completed four-core work is slower by more than 2% even at the
     interval's best, whatever the CPU saving.
   - The record and the code stay, with no further compact30 work.
3. **INCONCLUSIVE:** otherwise, when either interval straddles its threshold and neither
   STOP condition holds.
   - No automatic rerun and no extra rounds.
   - Report w4, e4, both intervals, b_max at w4's ends and point, and w1. The guardian
     decides.
4. **INVALID:** any gate or measurement-path check fails. No decision is read; the
   figures are reported as an invalid run's.

**Read in every outcome, deciding nothing:**
- w1, with its interval;
- each arm's ns a row at W1 and W4;
- each arm's W4 ÷ W1 scaling;
- the per-worker times and the overlaps.

## 7. What it cannot show

- **A production-decoder result:** MTF, RLE1, CRC, the scanner and the pipeline are absent.
- **Construction cost**, and **the cache state a direct constructor would leave** (§2).
- **The decoder's four-core mix:** lockstep puts four walks side by side. A decoder's free
  workers overlap one worker's build with another's walk, which this design excludes.
- **Peak RSS** or any memory measure.
- **Other machines or corpora.**
- **Whether a better compact30 schedule exists.** The code is fixed at the reviewed listing.
- **Why a result came out as it did.** No counters are collected. A counter capture would
  be a separate registration.

## 8. The VM checks done (no timing)

**Correctness:**
- The analysis's 14 tests pass (`bench/test_c30_traversal.py`, synthetic records). They
  cover:
  - each decision: ADVANCE; STOP on cost; STOP on throughput harm; INCONCLUSIVE straddling
    the screen; INCONCLUSIVE on throughput;
  - each gate: an unstable control, a staggered start, a missing walk, a listing mismatch,
    an allocation beyond one block, a barrier violation;
  - uneven walks being a valid result;
  - a dry run suppressing every timing figure;
  - the b_max table.
- The listing check passes on this tree's binary and fails on a reference with one operand
  changed.

**The untimed dry run** (`c30walk grid … --dry-run`, VM CPUs 0–3, rounds −1, 0 and 1):
- every one of the 5,208 walks was exact (asserted);
- G0, G3 and G4 hold;
- every measurement-path check holds: preparation before walk, the barrier, the CPUs, one
  walk per worker per step;
- the analysis suppressed every timing figure, G1, G2 and the decision, by design. None was
  read.

**The source identity:**
- the toolchain of the walker journal's §8: rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6;
- baseline x86-64 (fxsr, sse, sse2), no RUSTFLAGS;
- the release profile (codegen-units = 1, fat LTO, line tables).
- `room::<false>` in this tree's binary is identical to the reviewed listing
  (`2026-11-02-compact30-walker/room-false.asm`) under the normalisation of G3.

## 9. The records and the command

**The recipe** (`just bench-c30-traversal`) runs the frozen commit in its own worktree
(`target/c30-traversal-pin`):
1. the corpus hash;
2. the build;
3. the analysis's tests;
4. G3, and no run if it fails;
5. the CPUs: the second to fifth physical cores (the first takes more interrupts), or the
   first four where there are only four;
6. the capture;
7. the analysis.

**Records:** `target/c30-traversal-<UTC time>/`, archived as
`target/c30-traversal-<UTC time>.tar.gz` with a `MANIFEST.json` by the convention of report
142 (commit, corpus and binary hashes, the command and its exit status, every file's
checksum). It holds:
- the console;
- the toolchain;
- the listing and its diff;
- the records: the meta line with the assignment, each block's rows and RAW groups, and
  one line per walk with its preparation and walk windows and CPU;
- the analysis JSON.

**i9 time:** a rough estimate, not a measurement. About five minutes:
- each round walks the corpus's 207 M rows once per arm at W1 and once at W4;
- each walk follows its untimed build or copy;
- the oracle images are built once.

**The dry-run mode:** `just bench-c30-traversal CORPORA 20 dry-run` runs every step but
forces rounds −1, 0 and 1, marks the records a dry run, and the analysis reads no timing
from them. This is how the recipe itself was checked on the VM.

**The command for approval:**

```
just bench-c30-traversal
```
