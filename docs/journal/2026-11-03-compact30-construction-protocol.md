# Compact30's direct constructor against production's build: a construction-cost protocol (draft 1, for approval)

- **Status:** a draft for the guardian's approval, written under report 190, task 2
  ([2026-11-03-compact30-constructor-checkpoint.md](2026-11-03-compact30-constructor-checkpoint.md)).
  Nothing has been timed. Nothing below is a performance figure, and the collector it
  describes doesn't exist yet (§9).
- **Not authorised:** the collector's implementation, any timing (local or remote), and the
  i9 run. Each needs the guardian's approval. The recipe in §9 is how a run would execute
  once approved.
- **Prerequisite:** the independent code review of the constructor
  ([2026-11-03-compact30-constructor-review-handoff.md](2026-11-03-compact30-constructor-review-handoff.md)).
  If the review changes `ctor.rs`, the pins below move to the reviewed commit before any run.
- **The model:** the traversal protocol
  ([2026-11-02-compact30-traversal-protocol.md](2026-11-02-compact30-traversal-protocol.md),
  revision 5, closed VALID ADVANCE). This draft keeps its structure:
  - one binary with both arms in the same process;
  - paired, alternating rounds;
  - pinned workers, with a spinning start barrier and a completion barrier;
  - per-thread CPU clocks;
  - stages S (schedule), G (gates) and D (decision);
  - a frozen source and toolchain;
  - a dry run before any capture.

  What is new here is the arms, the boundary, the two allocation regimes, the memory
  measures and the decision ratio.

## 1. The question

On the primary corpus, does building compact30's tables directly (`Ctor::build::<false>`)
cost no more than 1.06714 × production's build CPU cost, at four cores, with the tables in a
usable state?

- **1.06714** is the conservative construction budget the traversal result left
  (constructor brief, report 186, §2): the largest b for which
  S ≈ 0.592(1 − b) + 1.847(1 − w) ≥ 0.25, at w = w4's upper end, 0.84313.
- **It is an investment screen, not a prediction for the decoder.** Passing it authorises a
  separately approved integrated-decoder comparison, nothing more (report 190). Failing it
  is an engineering decision about this constructor, not a proof that no constructor could
  meet the budget.
- **74.34% complete-record coverage** (the prototype's counters) is not a speed figure, and
  this protocol doesn't assume anything from it.

## 2. The arms (one binary, both arms in the same process)

| arm | what is timed | its output |
|---|---|---|
| **P** (control) | `mibz2::bwt::build(ll, counts, &mut tt)`: production's public builder, which runs `build_one_pass` (`src/bwt.rs`, 253–318 at the pin) and, only if it returns false, `build_two_pass` | the T-vector, `tt.len() == n` |
| **C** (candidate) | `Ctor::build::<false>(ll, counts, 0, &mut rec, &mut raw)` (`examples/c30walk/ctor.rs`, 704–826), natural configuration, `permille = 0` | `rec` (G + 1 records, PARK included) and `raw` |

**Why P is `bwt::build`, not `build_one_pass`:**
- `build_one_pass` is private to the library. `build` is the function the decoder calls
  (`Segmented::invert`), and the traversal protocol's P arm also used it.
- On the decoder's counts it never falls back. Its one-pass path accepts exactly when the
  counts equal L's own and n < 2²⁴.
- C accepts under the same condition, plus n ≤ 983,010. So **C's success on a block proves
  P took the one-pass path** on it. Gate G6 (§5) requires C's success on every block.

**The matched boundary:** each arm's window opens just before its call and closes when it
returns, so each includes all the work needed to produce usable tables.

- **P includes:**
  - `tt.clear()`;
  - the counts' sum check;
  - the bucket starts and ends;
  - `reserve(n)`, which allocates only if the capacity is short;
  - the 16-row scan and scatter;
  - the end check and `set_len`.
- **C includes:**
  - `rec.clear()` and `raw.clear()`;
  - validation and the encoded cursors and limits;
  - the RAW bound;
  - the conditional `reserve_exact` of `rec` and `raw` (an old allocation released first);
  - the bitmap's clear and zeroing resize;
  - the crossing groups;
  - the scan, with complete records, fragments, one-row writes and RAW writes;
  - promotion;
  - the cursor check;
  - the last group's padding and PARK;
  - the canonical RAW ordering;
  - `set_len`.
- **Excluded from both:**
  - decoding the corpus to last columns and counts (done once, before any round);
  - the oracle's build and every prepared image;
  - every correctness check;
  - the barrier waits;
  - the walk.
- **Usable means:** P's table is what `walk_segments_room` reads; C's arrays are what
  `C30::walk::<false>` reads. Neither arm defers work past its window.
- **Each arm uses its own builder's allocation policy.** Production reserves to n
  (`reserve`, which may round up). The constructor reserves exactly G + 1 records and the RAW
  bound. Both are timed as they are.

## 3. The workload, the regimes and the order of work

**The corpus:** the primary `wikidata-20261026.bz2` (SHA-256 `cf13f439…`, the traversal
protocol's), decoded once, before any round, to each block's last column and counts:
- 434 blocks, 207,006,515 rows;
- 230 full blocks;
- 204 short blocks (8,324 rows).

The inputs are shared, read-only, by both arms.

**Workers, as in the traversal protocol:**
- **W1:** one worker pinned to the first listed CPU, every block in corpus order.
- **W4:** four workers, each pinned to its own physical core. Blocks are dealt by the same
  deterministic snake: sorted by rows descending, four to a step, the direction reversing
  each step. Each step, every worker:
  1. waits at the spinning start barrier;
  2. builds its block (timed);
  3. waits at the completion barrier;
  4. then, untimed, runs its checks and records.

  A worker with no block in a step still waits at both barriers.

**Two allocation regimes, each a separate pass and reported separately:**

| regime | the arenas at the window's start | what it stands for |
|---|---|---|
| **warm** (reused capacity) | the worker's own arenas from its previous block, cleared by the builder itself. Capacity is grown only by the builds, so after round −1 each worker's arenas hold its largest block's capacity | a decoder worker's steady state; production's `TVector` and `SegScratch` are kept in the same way |
| **cold** (fresh allocation) | fresh, empty arenas: P a new `TVector`, C new `rec`, `raw` and `Ctor` (bitmap). The previous block's arenas are dropped before the start barrier, outside the window | first use, and growth: every build allocates its full capacity and first-touches it |

- **Cold measures allocation through the system allocator as it is** (glibc malloc, no
  tuning, no allocator change). Large allocations may be served by `mmap` and first-touched
  inside the window, or reuse freed heap without page faults: glibc's dynamic `mmap`
  threshold decides. The window's minor page faults are recorded (`getrusage(RUSAGE_THREAD)`
  read around the window, inside the monotonic readings like the CPU clock) to show which
  happened. They are reported, and decide nothing.
- **Growth within the warm regime** happens only in round −1, which isn't used.

**Rounds:** 20 measured paired rounds plus an untimed round −1.
- **A round** is W1 warm under each arm, W4 warm under each arm, W1 cold under each arm,
  then W4 cold under each arm, each in that round's arm order.
- **Even rounds** run P first; odd rounds (round −1 included) run C first.
- **Round −1** warms the binary and the arenas, and runs the full correctness gates (§5).
- **Clock calibration:** before round −1 and after round 19, 20,000 empty windows a CPU,
  with the same calls as a build window (G5).

**The checks between builds, and their cache effect:**
- After each step's completion barrier, each worker checks its own output (§5, gate G6).
  The check runs outside every window. It reads the arm's table and its reference image, so
  the next timed build starts with that worker's arena warm from the check.
- That is the same for both arms. It is recorded as a limit (§7), not corrected for: no
  cache flushing is added.
- The checks never run while any build of the step runs.

**Failure:**
- As in the traversal protocol: a failure record is appended and the process exits with
  status 3.
- The triggers are a check mismatch, a C rejection, a pinning or clock failure, or a panic.
- Fault injection is allowed in dry runs only.

## 4. The measures

**Clocks:**

| quantity | source | use |
|---|---|---|
| a build's CPU time | `CLOCK_THREAD_CPUTIME_ID` | b1, b4 and the decision |
| a build's start and end, every preparation and check window | `std::time::Instant` | e4, the start skew, the ordering checks |
| a build's CPU identity | `sched_getcpu` before the start barrier and after the completion barrier | placement (S4) |
| a build's minor faults (cold regime) | `getrusage(RUSAGE_THREAD)` | reported only |

The window's calls are, in order: monotonic, CPU (and rusage in the cold regime), the build,
then CPU (and rusage), monotonic.

**Per arm, regime and round:**
- **W1 cost:** Σ build CPU ns ÷ Σ rows.
- **W4 cost:** Σ over four workers of build CPU ns ÷ Σ rows.
- **W4 completion time:** Σ over steps of (latest build end − earliest build start).

**The ratios, paired by round, C ÷ P:**

| ratio | regime | role |
|---|---|---|
| **b4** | warm | **decides** (§6) |
| b1 | warm | reported |
| e4 | warm | reported |
| b4ᶜ | cold | reported, with its own interval |
| b1ᶜ | cold | reported, with its own interval |
| e4ᶜ | cold | reported, with its own interval |

**The estimate:** the geometric mean of the 20 round ratios, with a 95% interval from a t
interval (19 degrees of freedom) on their logs. No interval is computed from any other
number of rounds.

**Memory, never measured inside a timed window:**
- **Actual capacities,** per worker and block, read after the completion barrier:
  - P: `tt` capacity × 4 bytes;
  - C: `rec` capacity × 64, `raw` capacity × 128, bitmap capacity × 8;
  - each with glibc `malloc_usable_size`.

  Reported per regime: maximum, and sum over the four workers at the maximum step.
- **Peak RSS, in separate processes** (the `cbuild-rss` mode, §9):
  - each process loads only the last columns and counts: no oracle, no images, no
    production output;
  - it records `VmHWM` from `/proc/self/status` after loading (the baseline);
  - it runs one arm's W4 warm pass over the corpus, then records `VmHWM` again;
  - the figure is the difference.

  Three processes per arm, alternating P, C, P, C, P, C, each fresh. Reported as the median
  and range per arm, deciding nothing. This keeps the oracle's arrays, the images and the
  check buffers out of the candidate's memory figure entirely.

**The slow paths, interpreted, not timed:** a separate untimed pass, before round −1, runs
`build::<true>` on every block and records the structural counters per block:
- complete-record, fragment and one-row rows;
- runs and their length histogram;
- wide promotions and reconstructed fields;
- RAW groups, moved, cycles.

It runs in its own arena, and its results are written before any timing. The analysis
reports, deciding nothing:
- the per-block C ÷ P CPU ratio (warm W1, median over rounds) against each block's
  complete-record share, fragment share, one-row share and promotions, as a table by deciles
  of the one-row share;
- the full-block and short-block subsets' b1 separately.

No profiler, counter or extra instrumentation runs during a measured round.

## 5. The checks, in order

The analysis runs three stages; each runs only if the one before it passed.

### Stage 1: the schedule (S)

As in the traversal protocol, adapted to builds and two regimes:

| check | condition |
|---|---|
| S0 | no failure record and no unknown record kind |
| S1 | the meta record agrees with itself and the protocol (blocks, rows, the snake lists re-derived from the block rows, four distinct CPUs) |
| S2 | a capture has 20 measured rounds and no injection; a dry run has 2 |
| S3 | the record set is exactly the expected one: each (round, regime, mode, arm, worker, step) once, with its block and rows |
| S4 | every build's CPU, before and after, is its worker's listed CPU |
| S5 | in each build record: build ≤ check in time, and its CPU window no longer than its monotonic window (2 µs of noise allowed) |
| S6 | W1: each check ends before the next build starts |
| S7 | W4: in each step, every build ends before any check starts; every check of a step ends before the next step's start barrier releases |
| S8 | the passes run in the registered order and don't overlap |
| S9 | the calibration evidence is complete and well formed (the traversal protocol's S9) |
| S10 | the untimed counter pass and the round −1 correctness records are complete: one per block |

### Stage 2: the gates (G)

| gate | condition |
|---|---|
| G1 stability | P's per-round W1 cost, W4 cost and W4 completion time, max ÷ min ≤ 1.05, in each regime |
| G2 simultaneous start | in every all-full W4 step, the start skew over the step's span, row-weighted per arm, regime and round, ≤ 0.02 |
| G3 identity | the corpus hash; the toolchain (rustc 1.97.0, `2d8144b78`, LLVM 22.1.6); and the listings of the timed functions against reviewed listings, by the traversal protocol's normalisation (`bench/asm_listing.py`): `<c30walk::ctor::Ctor>::build::<false>` (reviewed at 0x1b5e bytes at `446a3b2`, to be re-reviewed at the pin) and the function the P call compiles to, `mibz2::bwt::build` with `build_one_pass` as inlined at the pin (its extent recorded and reviewed at the dry run). No run if G3 fails |
| G4 allocation | per-worker rows differ by at most the largest block, full-block counts by at most one |
| G5 clock calls | the calibration-based estimate of the clock (and rusage) calls' share ≤ 0.5% of each pass's summed build CPU, in every arm, regime, mode and round |
| **G6 exact output** | in every round, every build of both arms passed its check, run after the completion barrier, outside the window: **C:** the call returned true, and `rec` and `raw` equal the block's oracle image (`C30::build` with permille 0, built before round −1) in length and every byte. **P:** `tt` equals the block's reference T-vector in length and every byte. That reference is P's own round −1 table, accepted only after the walk check below |
| **G7 walker output and complete initialisation** | in round −1 only (untimed): **(a)** before C's build, its record arena's spare capacity is filled with the 0xA5A5 poison, and the arrays must still equal the oracle's; **(b)** the unchanged walker (`C30::walk::<false>`) on C's arrays, and `walk_segments_room` on P's table, each give production's `Segmented::invert` output for the block; **(c)** the arrays are compared with the oracle's again after the walk. The poison and the walks never run in a measured round |

**What G6 and G7 rest on:**
- **Equivalence and walker exactness** were established by the prototype's suites and are
  re-checked here on the measured corpus and build.
- **Complete initialisation** is shown on every block in round −1 by the poison (G7a). In
  measured rounds the warm arena holds the previous block's data, so a field left unwritten
  would almost surely differ from the oracle (G6).
- **No memory-safety tool runs during the capture.**

**The dry run** (rounds −1, 0, 1) runs S, G3, G4, G6 and G7. It suppresses every timing
figure, G1, G2, G5 and the decision.

### Stage 3: the figures and the decision (§6), on a valid run only

## 6. The decision rule (fixed before any timing)

**The ratio:** b4, the warm regime's four-core build CPU cost, C ÷ P. That is the geometric
mean of the 20 per-round ratios of Σ worker build CPU ns ÷ Σ rows, with its 95% t interval
[b4−, b4+] on the logs (19 degrees of freedom).

**The ceiling:** **1.06714** (§1), fixed.

1. **PASS:** b4+ ≤ 1.06714.
   - Even at the interval's worst, the constructor's four-core CPU cost is within the
     budget the traversal result left.
   - This authorises drafting the next, separately approved step: a production-shaped
     concurrency and end-to-end decoder evaluation. It authorises no integration and no
     claim about the decoder.
2. **FAIL:** b4− > 1.06714.
   - Even at the interval's best, the constructor costs more than the budget.
   - The constructor as reviewed stops at this checkpoint, as an investment decision. The
     record states the S the measured b4 and the traversal's w4 would give:
     S = 0.592(1 − b4) + 1.847(1 − w4), at both intervals' ends.
   - A cheaper constructor is not ruled out. Any new variant would need its own design
     review.
3. **INCONCLUSIVE:** otherwise, when [b4−, b4+] contains 1.06714.
   - No automatic rerun and no extra rounds. The guardian decides.
4. **INVALID:** any S check, gate or failure.
   - An S failure or a failure record: no figure is computed.
   - A gate failure: the figures are reported as an invalid run's, with no decision.

**Reported in every outcome, deciding nothing:**
- b1, e4;
- the cold regime's b4ᶜ, b1ᶜ, e4ᶜ with their intervals;
- each arm's CPU ns a row at W1 and W4, warm and cold;
- the running fraction (CPU ÷ elapsed);
- the minor faults (cold);
- the capacities and the peak RSS (§4);
- the slow-path tables (§4);
- S at b4's and w4's interval ends.

**Why the warm regime decides:**
- The 1.06714 budget comes from a model of a decoder's steady state, in which a worker
  reuses its tables from block to block, as production's does.
- The cold regime describes first use and growth. It is reported with its own interval so
  its cost can't hide.

Whether it should also gate is open question 1 (§10).

**Why b4, not b1:**
- The budget was derived from w4, the four-core traversal ratio.
- The model's b is the constructor's CPU cost in the same four-worker regime.
- b1 is reported, as w1 was.

## 7. What it cannot show

- **The decoder's result:** MTF, RLE1, CRC, the scanner and the pipeline are absent. So is
  the walk that follows each build.
- **The cache state each build leaves for its walk,** and whether a direct constructor's
  writes help or hurt the traversal measured with a copied image (report 186's caution).
- **The decoder's four-core mix:** lockstep puts builds side by side. A decoder overlaps one
  worker's build with another's walk.
- **A cache-neutral comparison:** each timed build follows its own arm's check of the
  previous block (§3), with the arena and the reference image warm.
- **Allocator independence:** the cold regime measures glibc as configured on the machine.
- **Other corpora or machines,** unless open question 3 adds a reported corpus.
- **Why a result came out as it did,** beyond the untimed counters' associations (§4).

## 8. The VM checks (no timing), before the i9

Before any capture, on the VM, untimed:
- the collector's unit tests, and the analysis's regression tests on synthetic record files:
  every S check and gate failing on purpose, and the decision rule at its boundaries;
- a dry run on the primary corpus;
- the failure path by injection, as the traversal protocol's revision 3 did:
  - a corrupted output byte in each arm (G6 must fail);
  - a pinning failure;
  - a forced C rejection;
- G3's listing review of both timed functions;
- the barrier test (`barrier-test`, unchanged).

## 9. The records and the command (not authorised)

**What would be built** (after approval, as its own reviewed change; this draft doesn't
write it):
- a `cbuild` mode and a `cbuild-rss` mode in the c30walk example, beside `grid`. They reuse
  `grid`'s pinning, `SpinBarrier`, clocks, calibration, failure path and record format;
- `bench/c30_construction.py`, the analysis, on the traversal analyser's pattern.

Neither the constructor nor the walker changes.

**The recipe** (`just bench-c30-construction`, modelled on `bench-c30-traversal`), at a
frozen pin `<PIN>` that the guardian approves:
1. the corpus hash;
2. the toolchain, rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6) via `rustup run 1.97.0`, stopping
   if it is missing or misidentified;
3. the build, `CARGO_ENCODED_RUSTFLAGS=`, `--locked`, the release profile (codegen-units 1,
   fat LTO, line tables), baseline x86-64, into `target/rust-1.97.0/`;
4. the analysis's regression tests and G3's;
5. G3 on both timed functions, and no run if it fails;
6. the CPUs: the second to fifth physical cores (the first four where there are only four);
7. `barrier-test` on those CPUs;
8. the untimed counter pass (`ctor-check`'s counters, one file);
9. the capture, `c30walk cbuild OUT.jsonl wikidata-20261026.bz2 --rounds 20 --cpus A,B,C,D`,
   under a two-hour timeout;
10. the six `cbuild-rss` processes;
11. the analysis, whether the capture passed or failed;
12. the archive (`target/c30-construction-<UTC time>.tar.gz`) with a `MANIFEST.json` by
    report 142's convention: the pin, the corpus and binary hashes, the commands, exit
    statuses, and every file's checksum.

**Not authorised:** steps 8–12 on the i9 or any workstation, and steps 1–7 on the i9.
On the VM, steps 1–7 and the dry run only, once the collector itself is authorised.

## 10. Open questions for the guardian

1. **Should the cold regime gate?** This draft decides on warm b4 alone and reports b4ᶜ. An
   alternative adds a second condition, such as b4ᶜ+ ≤ a cold ceiling, if first-use cost
   matters to the decoder (short-lived decoders, one block per worker).
2. **The P arm's identity.** Timing `bwt::build` (the public, decoder-called function) rather
   than a copy of the private `build_one_pass`, with G6's C-success argument proving the
   one-pass path was taken. Is that acceptable, or should `build_one_pass` be exposed for
   the experiment? Exposing it would be a `src/` change, which is not authorised.
3. **Corpora.** Only the primary corpus decides, since the budget came from it. Should
   `wikidata-heldout` be run as a reported second corpus? It would add a second capture and
   about double the run time.
4. **The check's cache effect.** Accept the asymmetric but structurally matched post-build
   check (§3, §7), or add a fixed cache-neutralising step (for example a write over a 32 MB
   buffer) before each start barrier, outside the windows?
5. **Completion time.** e4 is reported only here; the traversal protocol gated it at 1.02.
   Should construction's e4 also carry a tolerance?
6. **Short blocks.** The 204 short blocks are 0.004% of the rows but 47% of the windows, and
   their builds are dominated by fixed costs (256-entry cursor setup and the bitmap on C's
   side). They are kept in the decision ratio, as the traversal protocol kept them. Should
   b4 also be reported, or decided, on full blocks only?
7. **The review's outcome.** If the independent review requires changes to `ctor.rs`, the
   pin, G3's reviewed listing and the prototype's correctness suites must be redone first.
   Confirm that ordering.
8. **The decoded inputs' residency.** The capture process holds every block's last column,
   counts, production output, oracle image and P reference (about 8.2 bytes a row, so about
   1.7 GB on the primary corpus; an estimate, not a measurement), as the
   traversal capture held its images. Confirm this is acceptable on the i9. Peak RSS is
   measured separately (§4) for that reason.
