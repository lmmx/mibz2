# Compact30's direct constructor against production's build: a construction-cost protocol (draft 4)

- **Status: draft 3 accepted (report 194); draft 4 amends it before any construction
  timing** (reports 195–197,
  [2026-11-03-compact30-harness-review.md](2026-11-03-compact30-harness-review.md)).
  - Draft 2 applied the guardian's eight decisions to draft 1 (`6a097d8`), and draft 3 the
    answers to draft 2's two open questions (report 192).
  - Draft 4 makes G2 a per-step condition, requires a capture to consume a verified dry
    run's evidence (G9), records the poison's actual coverage (G7), validates the memory
    diagnostics, reports the mixed step apart (A7), and reads production's table through
    read-only accessors (A1).
  - §10 records every decision. Nothing has been timed. Nothing below is a performance
    figure.
- **Not authorised:** any construction timing, and any i9 or workstation run. The harness
  exists ([2026-11-03-compact30-construction-harness.md](2026-11-03-compact30-construction-harness.md),
  repaired in [-harness-repair.md](2026-11-03-compact30-construction-harness-repair.md)).
  The recipe in §9 is how a run would execute once approved.
- **The model:** the traversal protocol
  ([2026-11-02-compact30-traversal-protocol.md](2026-11-02-compact30-traversal-protocol.md),
  revision 5, closed VALID ADVANCE). This protocol keeps its structure:
  - one binary with both arms in the same process;
  - paired, alternating rounds;
  - pinned workers, a spinning start barrier and a completion barrier;
  - per-thread CPU clocks;
  - stages S (schedule), G (gates) and D (decision);
  - a frozen source and toolchain;
  - a dry run before any capture;
  - archiving of every run, passed or failed.

## 0. Preconditions (before the harness is written, and again before any capture)

1. **The independent code review is complete,** and the guardian has resolved its findings.
2. **If the review changes `examples/c30walk/ctor.rs`:**
   - the new source is frozen at a new commit, which becomes the pin;
   - `Ctor::build::<false>`'s assembly listing is produced from it with the pinned toolchain
     and reviewed again: whether the complete-record loop and the bounded fragment writer
     still have the form the prototype journal records, and the function's extent;
   - the prototype's correctness suites are rerun on it, all exact, and recorded:
     - `ctor-check` on the 11 hash-checked corpora (held-out included);
     - `ctor-adversarial`;
     - `ctor-synthetic` in full;
     - the Miri quick set.

   No measurement precedes this.
3. **If the review changes nothing,** the pin is `e003a32` (constructor source as at
   `446a3b2`), with its recorded suites and listing.
4. **The memory preflight passes** on the capture machine (§3) at the start of the capture,
   and is recorded in it.

## 1. The question

On the primary corpus, with every worker reusing its tables, does building compact30's
tables directly (`Ctor::build::<false>`) cost no more than 1.06714 × production's build CPU
cost at four cores, without making the synchronised four-worker construction steps more
than 2% slower to complete?

- **1.06714** is the conservative construction budget the traversal result left (report
  186, §2): the largest b for which S ≈ 0.592(1 − b) + 1.847(1 − w) ≥ 0.25, at w = w4's upper
  end, 0.84313.
- **It is an investment screen, not a prediction for the decoder.** PASS authorises only
  drafting the next separately approved step (§7).
- **74.34% complete-record coverage** (the prototype's counters) is not a speed figure; this
  protocol assumes nothing from it.

## 2. The arms (one binary, both arms in the same process)

| arm | what is timed | its output |
|---|---|---|
| **P** (control) | `mibz2::bwt::build(ll, counts, &mut tt)`, production's public builder. It runs `build_one_pass` (`src/bwt.rs` 253–318 at the pin) and only if that returns false `build_two_pass` | the T-vector, `tt.len() == n` |
| **C** (candidate) | `Ctor::build::<false>(ll, counts, 0, &mut rec, &mut raw)` (`examples/c30walk/ctor.rs` 704–826 at `e003a32`), natural configuration (`permille = 0`) | `rec` (G + 1 records, PARK included) and `raw` |

**P takes `build_one_pass` on every block, checked without a `src/` change (gate G8):**
- At the pin, `build_one_pass` returns false in three cases, and only those:
  - n ≥ 2²⁴;
  - the counts don't sum to n;
  - a bucket overflows or ends short (lines 256–258, the two in-scan returns, and the final
    `ftab != lim`).

  With the counts summing to n, an overflow or shortfall happens exactly when the counts
  differ from L's byte histogram. So **`build_one_pass` succeeds if and only if n < 2²⁴ and
  `counts` equals the histogram of `ll`**.
- **The harness checks that condition on every block before round −1** (untimed): it
  recomputes the histogram from `ll` and compares it with `counts`, and checks n. The result,
  one per block, goes into the records. A block failing it ends the run before any timing.
- **A second, independent confirmation:** C's own acceptance condition is the same (counts
  equal to L's histogram, n < 2²⁴) plus n ≤ 983,010. So C returning true on a block, which G6
  requires in every round, also implies P's one-pass path on that block.
- **Reading production's table** (A1, report 195): through two read-only accessors added to
  `src/bwt.rs`, `TVector::as_slice` and `TVector::capacity`. They change observation only,
  not the builder: G3 checks that `bwt::build`'s machine code is unchanged. There is no
  mutable access. An injected P fault corrupts a copy of the expected values, never
  production's table.
- **Why output equality can't show it:** `build_two_pass` would produce the same table, so
  the path is shown by its precondition, not by the output. The analysis also confirms from
  G3's reviewed listing that `bwt::build` calls the one-pass code first and the two-pass
  code only on its false result.

**The timed boundary: every operation needed for a usable table is inside the call.**
- **P's call includes:**
  - `tt.clear()`;
  - the counts' sum check;
  - the bucket starts and ends;
  - `reserve(n)`, which allocates only when the capacity is short;
  - the 16-row scan and scatter;
  - the end check and `set_len`.
- **C's call includes:**
  - `rec.clear()` and `raw.clear()`;
  - validation and the encoded cursors and limits;
  - the RAW bound;
  - capacity growth when applicable (`reserve_exact` of `rec` and `raw`, the insufficient
    allocation released first);
  - the bitmap's clear and zeroing resize;
  - the crossing groups;
  - the scan, with complete records, fragments, one-row writes and RAW writes;
  - wide promotion;
  - the cursor check;
  - the last group's padding and PARK;
  - the canonical RAW ordering;
  - `set_len`.
- **Nothing migrates into untimed preparation.** Neither arm is given a pre-sized, pre-zeroed
  or pre-touched arena, a precomputed bound, cursor table, bitmap or pattern beyond the
  constructor's own static `PATTERNS`, or any work that its builder does itself. The
  preparation before a window is limited to choosing the block and, in the cold regime,
  dropping the previous arenas (§3).
- **Excluded from both windows:**
  - decoding the corpus to last columns and counts (once, before any round);
  - the oracle's build and every reference image;
  - every correctness check;
  - the barrier waits;
  - the walk.
- **Usable means:** P's table is what `walk_segments_room` reads, and C's arrays are what
  `C30::walk::<false>` reads. Neither arm defers work past its window.
- **Each builder keeps its own allocation policy:** production reserves n entries; the
  constructor reserves exactly G + 1 records and its RAW bound. Both are timed as they are.

## 3. The workload, the regimes, the order of work and the memory preflight

**The corpus:** the primary `wikidata-20261026.bz2` only (SHA-256 `cf13f439…`), decoded once,
before any round, to each block's last column and counts:
- 434 blocks, 207,006,515 rows;
- 230 full blocks, 204 short blocks (8,324 rows).

The inputs are shared and read-only. The held-out and fitted Wikidata corpora, and the
other eight, contribute correctness evidence only (§0); they are not part of the capture.

**Workers, as in the traversal protocol:**
- **W1:** one worker pinned to the first listed CPU, every block in corpus order.
- **W4:** four workers, each pinned to its own physical core, dealt the blocks by the
  traversal protocol's **deterministic snake**: sorted by rows descending, ties by index,
  four to a step, the direction reversing each step. The harness writes the four lists and
  the block rows into the meta record; the analysis re-derives the lists and requires them
  equal (S1). Each step, every worker:
  1. waits at the spinning start barrier;
  2. builds its block (timed);
  3. waits at the completion barrier;
  4. then, untimed, runs its checks and writes its record into storage allocated before the
     pass.

  A worker with no block in a step still waits at both barriers.

**Two allocation regimes, each a separate pass:**

| regime | the arenas at the window's start | role |
|---|---|---|
| **warm** (reused capacity) | the worker's own arenas from its previous block, cleared by the builder itself; after round −1 each holds its largest block's capacity | **primary**: a decoder worker's steady state. The decision reads it alone (§7) |
| **cold** (fresh allocation) | fresh empty arenas: P a new `TVector`, C new `rec`, `raw` and `Ctor`. The previous block's arenas are dropped before the start barrier, outside the window | **reported separately, never a gate**: first-use and growth cost, which every cold build pays inside its window |

- **The cold regime uses the system allocator as configured** (glibc malloc, no tuning, no
  allocator change). Each cold window's minor page faults (`getrusage(RUSAGE_THREAD)`, read
  inside the monotonic readings like the CPU clock) are reported, deciding nothing.
- **Growth within the warm regime** happens only in round −1, which isn't used.

**Rounds:** 20 measured paired rounds plus an untimed round −1.
- **A round** runs these passes in order: W1 warm, W4 warm, W1 cold, W4 cold. Each pass runs
  under both arms, in the round's arm order.
- **The arm order alternates:** even rounds run P first; odd rounds (round −1 included) run
  C first.
- **Round −1** warms the binary and the arenas and runs the full correctness gates (G6–G7).
- **Clock calibration:** before round −1 and after round 19, 20,000 empty windows a CPU, with
  the same calls in the same order as a build window.

**The memory preflight** (recorded in the capture's meta record; precondition 4):
- **Required memory:** the harness computes it from the corpus before allocating anything:
  - the inputs, n bytes a row plus counts;
  - production's output, n bytes a row (G7);
  - the oracle images, 64(G + 1) + 128·R bytes a block;
  - P's reference T-vectors, 4n bytes a row;
  - eight workers' arenas at the largest block (four per arm);
  - the oracle's own build scratch.

  On the primary corpus that is estimated at about 8.2 bytes a row, about 1.7 GB; the
  harness's own sum is the one recorded. **The estimate must be verified before capture:**
  the dry run (§9) records the capture process's actual peak RSS beside the computed sum,
  and the capture's preflight uses whichever is larger.
- **The capture consumes a verified dry run (draft 4, report 197).** A capture takes the
  directory of a completed dry run on the same machine (the recipe unpacks it from the dry
  run's archive), not a number. Before the sizing pass, the collector verifies that:
  - its manifest shows exit status 0, a clean source, **this binary's** sha256 (so the
    same pin and toolchain), and **this corpus's** sha256;
  - its records file and analysis match the manifest's checksums;
  - its analysis is a dry run's, with every schedule check and G3, G4 and G6–G10 passed,
    and no failure;
  - its records are a completed, uninjected dry run of the same corpus, rows, binary,
    **CPU selection and machine identity** (hostname, machine-id, kernel, CPU model, online
    CPUs, installed memory);
  - its peak RSS (the postflight VmHWM) is present, equals the analysis's, and is at least
    2 bytes a row (the decoded inputs alone), so a fabricated tiny value is refused.

  Anything missing, stale, mismatched or implausible stops the capture with a failure
  record, before any decode or build, whether the collector is run by the recipe or
  directly. The capture's required memory is max(estimate, verified peak) plus the
  registered margin. The analyser re-verifies the evidence from its files, and the recipe
  keeps the dry run's archive and its checksum in the capture's own.
- **Free memory:** `MemAvailable` (`/proc/meminfo`) must be at least the required total
  plus a margin of max(2 GiB, 25% of the required total) (report 192). The required total,
  the margin and `MemAvailable` are all recorded.
- **No swap in use:** `SwapTotal − SwapFree` must be 0, or there is no swap.
- **No memory pressure:** where `/proc/pressure/memory` exists, its `some avg10` and
  `full avg10` must be 0.00. If it doesn't exist, that is recorded as absent.
- **Every memory source is read or the preflight fails** (draft 4). A `/proc/pressure/memory`
  that exists but can't be read or parsed, and a missing `MemAvailable`, `SwapTotal`,
  `SwapFree`, `pswpin` or `pswpout`, are recorded as unreadable (null). They never count as
  zero or as absent.
- **No swap activity:** `pswpin` and `pswpout` (`/proc/vmstat`) are recorded at the
  preflight and after the last round; any increase between them makes the run INVALID (G9).
- **Repeated after the last round** and recorded. Swap in use or nonzero memory pressure at
  that point makes the run INVALID (G9).
- **Major faults inside the windows:** every measured build records its own major and minor
  faults (§5). Any major fault in a measured build, in either regime, makes the run INVALID
  (G10, report 192): anonymous allocation normally causes only minor faults, and a major
  fault could put disk-I/O latency into the comparison. Minor faults are reported
  separately and decide nothing.
- **If the preflight fails, the run doesn't proceed.** A failure record is written and the
  recipe archives it. The corpus, the regimes and either arm's allocation policy are never
  changed to fit the machine; any such change would be a new registration.

**Failure:**
- As in the traversal protocol: a failure record (round, regime, mode, arm, worker, step,
  block, what failed) is appended to the records file, it is printed, and the process exits
  with status 3, so no worker is left at a barrier.
- **The triggers:** a check mismatch, a C rejection, a one-pass precondition failure, a
  preflight failure, a pinning or clock failure, or a panic, through the panic hook.
- **Not covered:** a process abort, a signal or an out-of-memory kill leaves no failure
  record. The records file is then incomplete, so S3 rejects the run.
- **Afterwards:** the recipe runs the analysis and **archives every run, passed or failed**
  (§9).
- **Fault injection** is allowed in dry runs only, and the analysis rejects a capture
  carrying it.

## 4. The cache-state asymmetry (disclosed, not corrected)

No cache flushing or neutralising step is added (report 191). Each arm keeps its natural
preparation and validation environment, which leaves the caches in different states before
its builds:

- **The check before each build.** After each step, each worker checks its own output
  against its reference (G6): P reads its 4n-byte T-vector and its 4n-byte reference; C reads
  its records and RAW groups (about 2.16n bytes) and the oracle image of the same size. The
  next timed build of that worker therefore starts with that arm's arena, and its reference,
  partly in cache. How much survives depends on the arm's table size: production's is about
  1.85 times compact30's. So the two arms' builds start from different cache states, by
  construction of the arms, not by accident of the harness.
- **The previous block's table.** In the warm regime each build writes over the previous
  block's table in the same arena; in the cold regime it writes fresh memory.
- **What this means for the result:**
  - The ratios compare each builder in its own natural reuse cycle: build, then check, then
    build again.
  - They don't isolate the builders' cost under identical cache state.
  - Nor do they reproduce a decoder, where a block's walk, not a check, follows its build,
    and other workers' phases overlap.

## 5. The measures

**Clocks:**

| quantity | source | use |
|---|---|---|
| a build's CPU time | `CLOCK_THREAD_CPUTIME_ID`, the calling thread's consumed CPU | b1, b4 (§7) |
| a build's start and end; every check window | `std::time::Instant`, monotonic | e4, the start skew, every ordering check |
| a build's CPU identity | `sched_getcpu` before the start barrier and after the completion barrier | placement (S4) |
| a build's major and minor faults | `getrusage(RUSAGE_THREAD)`, `ru_majflt` and `ru_minflt` | major: G10 (any is INVALID); minor: reported only |

- **The window's calls,** in order: monotonic, CPU, rusage, the build, then rusage, CPU,
  monotonic, in both regimes. The rusage calls' cost is part of the measured clock overhead
  (G5).
- **CPU time and completion time are separate measurements.** The CPU window lies inside the
  monotonic one, and each is recorded and reported on its own.

**The clock calls' overhead (G5), measured and reported:**
- From each calibration batch: the median cost of the calls on each CPU, per clock.
- The estimate: the largest median. Times the windows in a pass, it is expressed as a share
  of the pass's summed build CPU time.
- Reported for every arm, regime, mode and round, and separately for the short blocks
  (whose builds are short).
- G5 requires the share to be at most 0.5% in every arm, regime, mode and round, over the
  whole corpus.

**Per arm, regime and round:**
- **W1 cost:** Σ build CPU ns ÷ Σ rows.
- **W4 cost:** Σ over the four workers of build CPU ns ÷ Σ rows.
- **W4 completion time:** Σ over steps of (latest build end − earliest build start), on the
  monotonic clock. It is the summed synchronised construction-step completion time. It
  leaves out the checks and barrier waits between steps. **It is not decoder throughput, and
  not the capture's wall time.**

**The ratios, paired by round, C ÷ P:**

| ratio | regime | role |
|---|---|---|
| **b4** (four-core build CPU cost) | warm | **decides** (§7) |
| **e4** (four-core construction-step completion time) | warm | **decides** (§7) |
| b1 (one-core build CPU cost) | warm | reported |
| b4ᶜ, e4ᶜ, b1ᶜ | cold | reported, never a gate |

**The estimate:** the geometric mean of the 20 round ratios, with a 95% interval from a t
interval (19 degrees of freedom) on their logs. No interval is computed from any other number
of rounds.

**Full-block and short-block economics (reported, deciding nothing):**
- b4, b1 and e4 are recomputed over the 230 full blocks and over the 204 short blocks alone.
  - For b4 and b1 the sums run over each subset's builds.
  - For e4 they run over each subset's steps. A step mixing the two (step 57 on the
    primary corpus) belongs to neither subset, and its e4 is reported on its own (A7).
- Also reported per subset: each arm's CPU ns a row and ns a block. Short builds are
  dominated by fixed costs (256-entry cursor setup; C's bitmap and RAW bound).
- **The decision reads the complete primary corpus** (§7).

**Memory (never inside a timed window):**
- **Actual capacities,** per worker and block, read after the completion barrier:
  - P: `tt`'s capacity × 4 bytes (`TVector::capacity`);
  - C: `rec` × 64 + `raw` × 128, labelled **rec + raw**. The constructor's bitmap is private
    to the accepted `ctor.rs`, so it is not observed; its requested size, 8·⌈G/64⌉ bytes, is
    recorded beside it;
  - each also with glibc `malloc_usable_size`.

  Reported per regime: the maximum, and the sum over four workers at the largest step.
- **Peak RSS, in separate processes** (the `cbuild-rss` mode):
  - each process loads only the last columns and counts: no oracle, no references, no
    production output;
  - it records `VmHWM` after loading (the baseline);
  - it runs one arm's W4 warm pass, then records `VmHWM` again;
  - the figure is the difference.

  Three processes per arm, alternating P, C, P, C, P, C, each fresh: the median and range,
  deciding nothing. The oracle's arrays and every reference stay out of the candidate's
  memory figure.

  **The records are validated before any summary (draft 4).** There must be:
  - six records, three distinct processes (pid and start time) per arm;
  - each of the capture's corpus (name and sha256), block count and CPUs;
  - the high-water mark reset accepted;
  - nonnegative readings, a positive baseline and capacity, and the peak after the pass at
    least the reset value.

  Otherwise the diagnostic is reported as unavailable, with the reasons.

**The slow paths, interpreted, not timed:**
- A separate untimed pass before round −1 runs `build::<true>` on every block, in its own
  arena, and records the structural counters per block:
  - complete-record, fragment and one-row rows;
  - runs and their length histogram;
  - wide promotions and reconstructed fields;
  - RAW groups, moved, cycles.
- The analysis reports, deciding nothing, the per-block warm W1 ratio (the median over
  rounds) by deciles of the one-row share, with each decile's fragment share and promotions.
- No profiler, hardware counter or extra instrumentation runs during a measured round.

## 6. The checks, in order

The analysis runs three stages; each runs only if the one before it passed.

### Stage 1: the schedule (S), before any figure

| check | condition |
|---|---|
| S0 | no failure record and no unknown record kind |
| S1 | the meta record agrees with itself and the protocol: the block count, rows and full blocks from the block rows; the four lists equal to the snake re-derived from them; the per-worker rows; four distinct CPUs; the preflight and postflight records present; the corpus's and the binary's sha256 and the machine identity recorded (draft 4) |
| S2 | a capture has 20 measured rounds and no injection; a dry run has 2 |
| S3 | the record set is exactly the expected one: each (round, regime, mode, arm, worker, step) once, with its registered block and rows; nothing missing, extra or duplicated; each W4 step holds exactly its declared workers |
| S4 | every build's CPU, before and after, is its worker's listed CPU (W1: the first) |
| S5 | in each record: build before check, and the CPU window no longer than the monotonic window (2 µs of clock noise allowed) |
| S6 | W1: each check ends before the next build starts |
| S7 | W4: in each step, every build ends before any check starts, and every check ends before the next step's builds start |
| S8 | the passes run in the registered order (rounds in turn; in each, W1 warm, W4 warm, W1 cold, W4 cold, each in the round's arm order) and don't overlap |
| S9 | the calibration evidence is complete and well formed (the traversal protocol's S9) |
| S10 | the one-pass precondition records, the untimed counter and poison passes, and the round −1 correctness records are complete: one per block (one per block and arm for round −1's) |

### Stage 2: the gates (G), on a schedule that passed

| gate | kind | condition |
|---|---|---|
| G1 | stability | P's per-round W1 cost, W4 cost and W4 completion time, max ÷ min ≤ 1.05, in each regime |
| G2 | simultaneous start | **every** all-full W4 step's own start skew (latest start − earliest start, over the step's span) ≤ 0.02, in every arm, regime and measured round (draft 4, report 197). The row-weighted aggregate per arm, regime and round, draft 3's measure, is reported beside the per-step maximum and decides nothing |
| G3 | identity | the corpus hash; the toolchain (rustc 1.97.0, `2d8144b78`, LLVM 22.1.6); the pin with no tracked changes; and the listings of both timed functions, compared with the reviewed listings by the traversal protocol's normalisation (`bench/asm_listing.py`). For C: `<c30walk::ctor::Ctor>::build::<false>`, reviewed at 0x1b5e bytes at `446a3b2` (re-reviewed if §0.2 applies). For P: the function the `bwt::build` call compiles to at the pin, with `build_one_pass` inlined, its extent recorded and reviewed at the dry run. The binary's sha256 is recorded but is not an identity (its line tables hold build paths). No run if G3 fails |
| G4 | allocation | per-worker rows differ by at most the largest block, and full-block counts by at most one |
| G5 | calibration | the clock calls' estimated share ≤ 0.5% in every arm, regime, mode and round (§5) |
| G6 | exact output | in every round and regime, every build of both arms passed its check, run after the completion barrier, outside the window. **C:** the call returned true, and `rec` and `raw` equal the block's oracle image (`C30::build`, permille 0, built before round −1) in length and every byte. **P:** `tt` equals the block's reference T-vector in length and every byte (P's own round −1 table, accepted only after G7b) |
| G7 | walker output and complete initialisation | round −1 and the untimed preamble only. **(a)** In round −1's W1 warm pass, C's record arena's spare capacity is filled with the 0xA5A5 poison before its build, and the arrays must still equal the oracle's. The record states the capacity poisoned, the records needed, and whether the poisoned allocation survived the build: it doesn't for the first block or when the constructor grows, and then no coverage is claimed (draft 4). **(a′)** For universal coverage, an untimed poison pass before round −1 builds every block with `build::<false>` into one arena preallocated for the largest block and poisoned. Every block's poison must survive and its arrays equal the oracle's. The measured arms are never preallocated or altered. **(b)** The unchanged walker (`C30::walk::<false>`) on C's arrays, and `walk_segments_room` on P's table, each give production's `Segmented::invert` output for the block. **(c)** C's arrays are compared with the oracle's again after the walk |
| G8 | production's one-pass path | every block satisfies the one-pass condition (n < 2²⁴ and `counts` equal to `ll`'s histogram), checked before round −1 (§2); and C accepted every block in every round (G6) |
| G9 | memory | the preflight passed (§3), every source read, with required = max(estimate, verified peak) and the max(2 GiB, 25%) margin. A capture must carry the verified dry run's evidence, which the analyser re-verifies from its files; a dry run carries none. After the last round no swap is in use and no memory pressure is recorded, every source read, and `pswpin`/`pswpout` did not increase over the run |
| G10 | major faults | no measured build, in either arm, regime or mode, recorded a major fault inside its window (§3, §5) |

**What G6 and G7 rest on:**
- Equivalence and walker exactness were established by the prototype's suites (re-run
  under §0 if the source changes), and are re-checked here on the measured corpus and build.
- Complete initialisation is shown on every block by the poison pass (G7a′). It is also
  shown in round −1's warm pass wherever the poisoned allocation survived (G7a, recorded per
  block). In measured rounds the warm arena holds the previous block's data, so a field left
  unwritten would almost surely differ from the oracle (G6).
- No memory-safety tool runs during the capture.

**The dry run** (rounds −1, 0 and 1) runs the preflight, S, G3, G4 and G6–G10 (A4). It
suppresses every timing figure, G1, G2, G5 and the decision. Its archive is the evidence a
capture on the same machine consumes (G9).

### Stage 3: the figures and the decision (§7), on a valid run only

## 7. The decision (fixed before any timing)

**The deciding ratios,** both from the **warm four-core** regime over the **complete primary
corpus**:
- **b4:** the build CPU-cost ratio C ÷ P, with 95% interval [b4−, b4+];
- **e4:** the construction-step completion-time ratio C ÷ P, with 95% interval [e4−, e4+].

| outcome | registered condition |
|---|---|
| **PASS** | b4+ ≤ 1.06714 **and** e4+ ≤ 1.02 |
| **FAIL** | b4− > 1.06714 **or** e4− > 1.02 |
| **INCONCLUSIVE** | neither PASS nor FAIL. No automatic extra rounds or rerun; the guardian decides |
| **INVALID** | any schedule (S), correctness (G6, G7, G8), identity (G3), stability (G1), calibration (G5), start (G2), allocation (G4), memory (G9) or major-fault (G10) check fails, or a failure record exists. On an S failure or a failure record no figure is computed; on a gate failure the figures are reported as an invalid run's, with no decision |

**The two thresholds:**
- **1.06714** is the construction CPU ceiling (§1), unchanged.
- **1.02** is a completion-time no-harm allowance. It is a **policy choice adopted before any
  construction timing** (report 191). It allows up to 2% more synchronised construction-step
  completion time. It is not an assertion that completion is identical.
- **e4 is construction-step completion time, not decoder throughput** (§5).

**What each outcome authorises:**
- **PASS:** drafting the next step, a production-shaped concurrency and end-to-end decoder
  evaluation, which must be separately approved. No integration, and no claim about the
  decoder.
- **FAIL:** the constructor as reviewed stops at this checkpoint, as an investment decision.
  - The record states the S that b4 and the traversal's w4 imply,
    S = 0.592(1 − b4) + 1.847(1 − w4), at the intervals' ends.
  - If e4 decided, it says so.
  - A cheaper constructor is not ruled out. A new variant would need its own design review.
- **INCONCLUSIVE:** b4, e4, both intervals and the implied S are reported for the guardian.

**Reported in every outcome, deciding nothing:**
- b1;
- the cold regime's b4ᶜ, e4ᶜ, b1ᶜ with their intervals, and its minor faults;
- the full-block and short-block economics (§5);
- each arm's CPU ns a row at W1 and W4, warm and cold;
- the running fraction (CPU ÷ elapsed);
- the clock-call overhead shares;
- the start skew: the per-step maximum and the row-weighted aggregate;
- the capacities and peak RSS;
- the slow-path tables;
- the preflight and postflight memory records.

## 8. What it cannot show

- **The decoder's result:** MTF, RLE1, CRC, the scanner and the pipeline are absent, and so is
  the walk after each build.
- **The cache state each build leaves for its walk,** and so whether direct construction
  helps or hurts the traversal measured with a copied image (report 186).
- **A cache-neutral comparison** (§4).
- **The decoder's four-core mix:** lockstep puts builds side by side, where a decoder overlaps
  one worker's build with another's walk.
- **Throughput:** e4 is synchronised construction-step completion time only.
- **Allocator independence:** the cold regime measures glibc as configured on the machine.
- **Other corpora or machines.**
- **Why a result came out as it did,** beyond the untimed counters' associations (§5).

## 9. The VM checks, the records and the command (not authorised)

**What is built** (the harness journals): `c30walk cbuild` and `c30walk cbuild-rss`;
`bench/c30_construction.py` and its tests; `bench/c30_build_listing.py` (G3);
`bench/test_c30_construction_evidence.py`, the collector's own evidence rejections. Neither
the constructor nor the walker changes. The only `src/` change is the two read-only accessors
(A1).

**VM checks before any capture (untimed):**
- the analysis's regression tests on synthetic record files: every S check and gate failing
  on purpose, the decision table at each boundary, and the single staggered step (G2);
- the collector's evidence rejections, run directly on the binary;
- a dry run on the primary corpus, and a `verify-evidence` run consuming its archive;
- the failure path by injection:
  - a corrupted output in C;
  - a corrupted expected copy for P, in G6 and in G7;
  - a pinning failure;
  - a forced C rejection;
  - a forced preflight failure;
- G3's listing review of every timed-path function;
- `barrier-test`, unchanged.

**The recipe** (`just bench-c30-construction`), at a pin the guardian approves (§0):
1. the corpus hash;
2. the toolchain, rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), selected with `rustup run 1.97.0`;
   stop if it is missing or misidentified;
3. the build, `CARGO_ENCODED_RUSTFLAGS=`, `--locked`, the release profile (codegen-units 1,
   fat LTO, line tables), baseline x86-64, into `target/rust-1.97.0/`;
4. the analysis's regression tests and G3's;
5. G3 on every timed-path function; no run if it fails;
6. the CPUs: the second to fifth physical cores (the first four where there are only four);
7. for a capture, the dry run's archive: kept, with its checksum, in the capture's directory,
   then unpacked for the collector;
8. `barrier-test` on those CPUs;
9. the collector, under a two-hour timeout. For a capture: the evidence verification, then
   the sizing pass and memory preflight, the decode, the one-pass precondition, the oracle
   images, the counter and poison passes, rounds −1 to 19, and the postflight;
10. the six `cbuild-rss` processes;
11. the analysis (with the evidence, for a capture), **whether the run passed or failed**;
12. the archive, **whether the run passed or failed**: `target/c30-construction-<UTC
    time>[-mode].tar.gz`, with a `MANIFEST.json` by report 142's convention.

**The sequence on a workstation,** once authorised:
1. `just bench-c30-construction <corpora> 20 dry-run`;
2. `just bench-c30-construction <corpora> 20 capture <that dry run's .tar.gz>`.

**Not authorised now:** any construction timing, and every step on the i9 or any
workstation.

## 10. The guardian's decisions (reports 191, 192, 195 and 197)

**Decided** (report 191, on draft 1's open questions):

| draft 1 question | decision | where applied |
|---|---|---|
| 1. Should the cold regime gate? | No: warm reused capacity is primary; cold and growth costs are reported separately | §3, §5, §7 |
| 2. Is `bwt::build` acceptable for P? | Yes, with an explicit check that every block takes `build_one_pass` | §2, G8 |
| 3. Add the held-out corpus? | No: the primary corpus only; held-out data stays in the correctness evidence | §0, §3 |
| 4. Flush caches between builds? | No: keep each arm's natural environment and disclose the asymmetry | §4 |
| 5. Gate completion time? | Yes: warm e4 upper ≤ 1.02 for PASS, lower > 1.02 for FAIL | §7 |
| 6. Decide on full blocks only? | No: decide on the complete corpus; report full and short blocks separately | §5, §7 |
| 7. Re-review a changed constructor? | Yes: freeze, re-review the listing and redo the suites first | §0 |
| 8. Is the input memory acceptable? | Conditionally: record a preflight (free memory, no swap, no pressure); never change the corpus or allocation policy to fit | §3, G9 |

**Decided** (report 192, on draft 2's open questions):

| draft 2 question | decision | where applied |
|---|---|---|
| 1. The preflight's margin | `MemAvailable` ≥ the estimated peak plus max(2 GiB, 25% of the estimate); record the actual values and check for swap activity; verify the 1.7 GB estimate before capture | §3, §9, G9 |
| 2. Major faults during the measured rounds | Any major fault in a measured build makes the run INVALID; minor faults are reported separately | §3, §5, G10 |

Report 192, as relayed: *"Memory preflight: Require estimated peak memory plus the larger
of 2 GiB or 25% of that estimate, using Linux `MemAvailable`. Record the actual values and
check for swap activity. The 1.7 GB estimate must be verified before capture. Major page
faults: Since these are recorded inside the timed construction windows, I would make any
major fault in a measured build an INVALID condition. Anonymous allocation normally causes
minor faults; a major fault could introduce disk-I/O latency into the comparison. Report
minor faults separately. These are protocol decisions, not permission to implement or run
the benchmark."*

**Decided** (reports 195 and 197, on the harness at `e264eed`):

| finding | decision | where applied |
|---|---|---|
| A1: production's table read by reinterpreting `&TVector` | Repair: two read-only accessors in `src/bwt.rs` (`as_slice`, `capacity`), no mutable access; a P fault injected into a copy of the expected values; `bwt::build`'s machine code checked unchanged | §2, §5, G3, G6 |
| A2: thin timing wrappers | Accept, their differing overhead documented | §2 |
| A3: frozen machine-code references | Accept; recheck after the accessor change | G3 |
| A4: G9 and G10 in dry runs | Accept | §6 |
| A5: the P reference checked against the counting-scatter definition | Accept | G7 |
| A6: the dry run's peak required for capture | Accept, and strengthened: the capture consumes the dry run's verified evidence (G9 below) | §3, §9 |
| A7: the mixed step | Report it separately | §5 |
| A8: failure wording | Accept | — |
| G2: one badly staggered step can pass the aggregate | Every all-full W4 step must meet the unchanged 2%; report the maximum and the aggregate. This is stricter than the traversal experiment's gate, and may reject a run with one isolated scheduling interruption | G2 |
| G9: the dry-run peak unverified | The capture consumes verified evidence of a successful dry run on the capture machine; the collector and analyser reject missing or inconsistent evidence, including by direct invocation | §3, G9 |
| G7: poison coverage overstated | Record the actual coverage; universal coverage only by a separate untimed pass; the measured arms unaltered | G7 |
| RSS records | Validate before summarising; otherwise unavailable | §5 |
| Memory sources | An unreadable source never becomes zero or absent | §3, G9 |

**What stays unchanged:**
- the constructor at `e003a32`, and the walker;
- the timed boundaries, the paired design and the 20 rounds;
- the statistical rules and both thresholds;
- no cache flushing.

No question remains open.
