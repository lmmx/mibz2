# Compact30's constructor v2 against production's build: a construction-cost protocol (v2)

- **Status: registered before any construction timing of constructor v2** (report 211, in
  [2026-11-03-compact30-constructor-v2-audit.md](2026-11-03-compact30-constructor-v2-audit.md)).
  - It is derived from draft 4
    ([2026-11-03-compact30-construction-protocol.md](2026-11-03-compact30-construction-protocol.md),
    `e4ffc8d`), which governed constructor v1's capture. Draft 4 stays as it is.
  - **Everything in draft 4 holds here unless §A lists it as changed.** The sections below
    restate draft 4 with the changes in place, so that this document can be followed on its
    own. Each change cites the part of report 211 that requires it.
  - Nothing has been timed. Nothing below is a performance figure.
- **Not authorised:** any construction timing of either constructor, any benchmark, and any
  i9 or workstation run. The harness exists, with untimed VM validation
  ([2026-11-03-compact30-construction-v2-harness.md](2026-11-03-compact30-construction-v2-harness.md)).
  §9's recipe is how a run would execute once approved.
- **Constructor v1's record is unchanged.** Its STOP and its formally INVALID capture
  ([2026-11-03-compact30-construction-capture.md](2026-11-03-compact30-construction-capture.md))
  stand. **The revised G1 and G2 below are prospective rules for v2 only.** They don't
  reinterpret or reclassify v1's capture, and no figure of v1's capture under them is
  computed or reported, here or in the harness (report 211: "not permission to reinterpret or
  reclassify v1's capture").

## A. What changes from draft 4, and why

| # | where | draft 4 | v2 | justification |
|---|---|---|---|---|
| A1 | §1, §2 | C is `Ctor::build::<false>` (`ctor.rs` at `e003a32`) | C is `Ctor2::build::<false>` (`ctor2.rs` at `8beffe2`), natural configuration | report 211's comparison: "C: `Ctor2::build::<false>` at the frozen v2 source, natural configuration" |
| A2 | §2 | C's timed boundary, as v1's builder does its work | C's timed boundary lists v2's work: descriptor and snapshot initialisation, temporary writes, deferred finishing and normalisation, RAW work, canonical ordering, allocation, padding and publication | report 211: "Include descriptor and snapshot initialization, temporary writes, deferred finalization, normalization, RAW work, allocation, padding and publication in C's timed boundary" |
| A3 | §6 G1 | P's per-round W1 cost, W4 cost and W4 completion time, max ÷ min ≤ 1.05, in **each regime** | P's **warm W4** cost and **warm W4** completion time, max ÷ min ≤ 1.05. W1 (both regimes) and cold W4 stability, for both arms, are reported and decide nothing | report 211: "G1 should gate the production control's warm W4 CPU cost and warm W4 completion stability at the existing max/min ≤ 1.05. Report W1 and cold stability without making those nondeciding measurements invalidate the capture" |
| A4 | §6 G2 | **every** all-full W4 step's skew ≤ 2%, in every arm, regime and round | for **each arm's warm** W4 all-full steps over the 20 measured rounds: (a) row-weighted aggregate ≤ 2%; (b) at most ⌊N/1000⌋ steps over 2%; (c) no step over 10%. Cold steps reported | report 211: "Row-weighted aggregate start skew ≤ 2%. No more than 0.1% of all-full steps may individually exceed 2%. No individual step may exceed 10% skew." The regime follows "each arm's warm W4 full-block steps" |
| A5 | §6 S1, G9; §5 | no protocol identifier | the meta and RSS records carry `protocol: "c30-construction-v2"` and `candidate: "ctor2"`; a capture accepts only a v2 dry run's evidence; the analysis refuses records without the identifier | the same binary holds v1's and v2's modes, so the binary's checksum can't tell a v1-mode dry run from a v2 one. The brief: "a protocol identifier recorded in the evidence" |
| A6 | §6 G3 | v1's listings | v2's timed path against frozen references recorded from the harness's own build, and compared with the audited prototype listings; P against v1's reviewed listings, unchanged | report 211: "Ensure the compiled production builder is identical to its reviewed listing and inspect the v2 constructor … Prefer … frozen reference listings" |
| A7 | §5 | v1's structural counters | v2's (`Stats2`), and a v2 slow-path table | v2 has different slow paths (deferred finishing classes, normalisation, epochs) |
| A8 | §3 | memory estimate | adds v2's 16 KiB of epoch snapshots to the candidate's arenas in each worker's term | v2's own scratch, so the preflight counts it |
| A9 | §9 | `just bench-c30-construction` | `just bench-c30-construction-v2`, its own pin and worktree; v1's recipe unchanged | the brief: "add a v2 recipe if needed" and keep v1's mode as reviewed |

**Unchanged** (report 211: "Retain the remaining correctness, memory, clock-calibration,
binary-identity, evidence-provenance and archiving safeguards"):
- the primary corpus, P, the 20 paired rounds, round −1, the four physical workers, the snake,
  the barriers, the regimes and the pass order;
- the frozen compiler recipe and the matched full-construction boundaries;
- the deciding measures, warm b4 and e4, and the economic limits: **PASS needs b4+ ≤ 1.06714
  and e4+ ≤ 1.02**; FAIL and INCONCLUSIVE as draft 4;
- the statistics (geometric mean of 20 round ratios, t interval with 19 degrees of freedom);
- S0–S10, G4–G10 (G9's verified dry-run evidence and G10's major faults included), the memory
  preflight, the clock calibration, binary identity, evidence provenance, fault injection in
  dry runs only, and archiving of every run;
- the cache-state asymmetry, disclosed and not corrected (§4);
- what the experiment cannot show (§8).

### A.1 Why G1 and G2 change: v1's validity experience

Report 211 asks for the changes to be "documented and justified transparently". What v1's
capture recorded (the archived analysis of `c30-construction-20261103T172906Z`, as its
journal reports it; nothing here is recomputed):

- **G1 failed only in the one-worker control.** P's per-round W1 CPU cost varied by max ÷ min
  1.2274 (warm) and 1.2232 (cold) against 1.05, bimodally across rounds (about 0.521–0.544
  ns/row in thirteen rounds, 0.443–0.497 in seven). Both arms' W4 cost and completion time
  varied by 1.022–1.028, within the limit. W1 decides nothing under draft 4 (§7); b1 is a
  reported figure. Its cause wasn't established.
- **G2 failed on one isolated step.** One all-full W4 step out of 4,560 had a start skew of
  4.125% (round 7, warm, P, step 1). The next largest in that pass was 0.11%, and the largest
  row-weighted aggregate of any arm, regime and round was 0.09%. Report 197 had accepted that
  draft 4's per-step rule "may reject a run with one isolated scheduling interruption".

**What v2 keeps and what it relaxes:**
- **G1 gates what decides.** The decision reads warm W4 cost (b4) and warm W4 completion
  (e4). G1 gates P's stability in exactly those two, at the unchanged 1.05. W1 and cold
  instability are still measured and reported, for both arms, beside the figures they would
  qualify (b1, b4ᶜ, e4ᶜ, b1ᶜ).
- **G2 keeps the 2% target, tolerates very few isolated interruptions, and bounds them.**
  - (a) The aggregate is the row-weighted mean, over an arm's N warm all-full steps in the 20
    rounds, of draft 4's per-step skew. It is still 2%. A widespread stagger can't pass it.
  - (b) At most ⌊N/1000⌋ steps may individually exceed 2%. On the primary corpus N = 57 × 20 =
    1,140 per arm, so **the allowance is 1 step per arm**.
  - (c) No step may exceed 10%. A 33.3% step (one worker starting half a build late) is
    INVALID however isolated.
  - Cold steps decide nothing, as the cold regime decides nothing.
- **This is a looser gate than draft 4's, not a stricter one,** and that is deliberate. Under
  v2's rules one interruption like v1's 4.125% step would not invalidate a capture whose other
  steps meet 2%. The rules are registered before any v2 timing, and **they don't apply to
  v1's capture**, whose INVALID status stands as registered.
- **The limits that protect the comparison are unchanged:** 2% for ordinary steps, the
  economic limits, and the deciding regime.

## 0. Preconditions (before the harness is written, and again before any capture)

1. **Constructor v2's correctness checkpoint has passed:** the independent source audit,
   PASS (report 210), accepted by the guardian (report 211). Its evidence is archived in
   [2026-11-03-compact30-constructor-v2-audit/](2026-11-03-compact30-constructor-v2-audit/).
2. **Constructor v2 is frozen at `8beffe2`** (`examples/c30walk/ctor2.rs`), with its recorded
   suites ([the prototype journal](2026-11-03-compact30-constructor-v2-prototype.md)):
   - `ctor2-check` on the 11 hash-checked corpora, `ctor2-adversarial`, `ctor2-synthetic`
     in full and quick, all exact;
   - the partial Miri run (473 cases in five classes), accepted as partial supporting
     evidence (report 211: "No additional Miri marathon is required for this gate").

   **If the source changes,** the new source is frozen at a new commit, its listings are
   produced and reviewed again, and the suites are rerun and recorded, before any measurement.
3. **The harness is frozen at its pin** (§9), with the v2 reference listings in its tree.
4. **The memory preflight passes** on the capture machine (§3) at the start of the capture,
   and is recorded in it.

## 1. The question

On the primary corpus, with every worker reusing its tables, does building compact30's
tables with constructor v2 (`Ctor2::build::<false>`) cost no more than 1.06714 × production's
build CPU cost at four cores, without making the synchronised four-worker construction steps
more than 2% slower to complete?

- **1.06714** is the conservative construction budget the traversal result left (report
  186, §2): the largest b for which S ≈ 0.592(1 − b) + 1.847(1 − w) ≥ 0.25, at w = w4's upper
  end, 0.84313.
- **It is an investment screen, not a prediction for the decoder.** PASS authorises only
  drafting the next separately approved step (§7).
- **v2's static path counts are not speed figures.** Its 12-instruction scalar append against
  v1's 49, and its per-row field traffic (about 0.509 field stores and 0.274 field loads a
  row, against v1's 0.252 and 0.002), are static and untimed observations. This protocol
  infers nothing from them (report 211: "Do not infer runtime performance from static
  instructions").

## 2. The arms (one binary, both arms in the same process)

| arm | what is timed | its output |
|---|---|---|
| **P** (control) | `mibz2::bwt::build(ll, counts, &mut tt)`, production's public builder, through `p_build`. It runs `build_one_pass` (`src/bwt.rs` 253–318) and only if that returns false `build_two_pass` | the T-vector, `tt.len() == n` |
| **C** (candidate) | `Ctor2::build::<false>(ll, counts, 0, &mut rec, &mut raw)` (`examples/c30walk/ctor2.rs` 684–789 at `8beffe2`, with `scatter` 791–871 inlined), through `c2_build`, natural configuration (`permille = 0`) | `rec` (G + 1 records, PARK included) and `raw` |

**P takes `build_one_pass` on every block, checked without a `src/` change (gate G8),** as
in draft 4:
- `build_one_pass` succeeds if and only if n < 2²⁴ and `counts` equals the histogram of `ll`.
- The harness checks that condition on every block before round −1 (untimed), and records
  it per block. A block failing it ends the run before any timing.
- **The independent confirmation holds for v2:** `Ctor2::build` returns true only when the
  counts sum to n, n < 2²⁴, n ≤ 983,010, and no bucket overflows or ends short, which with
  the counts summing to n means the counts equal L's histogram. So C accepting a block, which
  G6 requires in every round, also implies P's one-pass path on that block.
- **Reading production's table** through the read-only `TVector::as_slice` and
  `TVector::capacity`, unchanged. Nothing is added to `src/`.

**The timed boundary: every operation needed for a usable table is inside the call.**
- **P's call** is draft 4's: `tt.clear()`; the counts' sum check; the bucket starts and ends;
  `reserve(n)`, which allocates only when the capacity is short; the 16-row scan and scatter;
  the end check and `set_len`.
- **C's call (A2)** holds all of constructor v2's work:
  - `rec.clear()` and `raw.clear()`; validation (n, the counts' sum);
  - the RAW bound; capacity growth when applicable (`reserve_exact` of `rec` and `raw`, the
    insufficient allocation released first);
  - the RAW bitmap's clear and zeroing resize;
  - **the epoch snapshots' clear and reservation** (16 × 1 KiB, allocated on first use);
  - **the 256 descriptors' initialisation** from the counts (destination ranks and limits);
  - the crossing groups' preset RAW blocks (and, in a forced configuration, which the
    natural configuration doesn't use, the forced ones);
  - the scan, epoch by epoch: **a snapshot at each epoch's start**; the 16-byte uniformity
    test and run coalescing, with runs split at epoch boundaries; the complete-record emitter
    for whole groups; **the temporary low-16-bit labels written into the final fields**
    through the descriptors, per 16-byte tile; refills;
  - **the deferred finishing of every portion,** at its next arrival or at the end: the fast
    case; **ordinary normalisation** (the header from the full label, the fields' vector
    subtraction); **wide groups promoted to RAW** at completion; **crossing portions** written
    into their RAW block and their fields restored to 0xFFFF;
  - the final rank check of every character and the closing of every open portion;
  - the last group's padding and PARK;
  - the canonical RAW ordering (`canonicalise`, v1's, shared);
  - `set_len`.
- **Nothing migrates into untimed preparation.** Neither arm is given a pre-sized,
  pre-zeroed or pre-touched arena, a precomputed bound, cursor or descriptor table, bitmap,
  snapshot, or pattern beyond the constructor's own static `PATTERNS`, or any work that its
  builder does itself. The preparation before a window is limited to choosing the block and,
  in the cold regime, dropping the previous arenas (§3). In the cold regime C's `Ctor2` is
  fresh: its bitmap and snapshots are allocated inside the window.
- **Excluded from both windows,** as draft 4: decoding the corpus to last columns and counts
  (once, before any round); the oracle's build and every reference image; every correctness
  check; the barrier waits; the walk.
- **Usable means:** P's table is what `walk_segments_room` reads, and C's arrays are what
  `C30::walk::<false>` reads. Neither arm defers work past its window: v2 finishes every
  portion before it returns.
- **Each builder keeps its own allocation policy:** production reserves n entries; v2
  reserves exactly G + 1 records and its RAW bound (the same bound as v1's), plus its bitmap
  and snapshots. Both are timed as they are.
- **The wrappers** (`p_build`, one `jmp`; `c2_build`, eight instructions setting the
  natural configuration's argument) are each one symbol, `#[inline(never)]`, and checked by
  G3. Their overhead difference is documented, as draft 4's A2 was.

## 3. The workload, the regimes, the order of work and the memory preflight

As draft 4, with C's arenas holding a `Ctor2`:
- **The corpus:** the primary `wikidata-20261026.bz2` only (SHA-256 `cf13f439…`), decoded once,
  before any round: 434 blocks, 207,006,515 rows; 230 full blocks, 204 short blocks
  (8,324 rows). The other corpora contribute correctness evidence only (§0).
- **Workers:** W1, one worker pinned to the first listed CPU, every block in corpus order;
  W4, four workers each pinned to its own physical core, dealt the blocks by the snake, with
  the spinning start barrier and the completion barrier, the timed build between them, and
  the untimed check and record after. A worker with no block in a step still waits at both
  barriers.
- **Two allocation regimes,** each a separate pass: **warm** (reused capacity; primary; the
  decision reads it alone) and **cold** (fresh arenas: P a new `TVector`, C new `rec`, `raw`
  and `Ctor2`, the previous ones dropped before the start barrier; reported, never a gate).
  Each cold window's minor faults are reported. Growth within the warm regime happens only in
  round −1.
- **Rounds:** 20 measured paired rounds plus an untimed round −1. A round runs W1 warm, W4
  warm, W1 cold, W4 cold, each under both arms in the round's arm order; even rounds run P
  first, odd rounds (round −1 included) C first. Round −1 warms the binary and the arenas and
  runs the full correctness gates (G6–G7).
- **Clock calibration:** before round −1 and after round 19, 20,000 empty windows a CPU, with
  the same calls in the same order as a build window.
- **The memory preflight** is draft 4's, with one addition (A8): the estimate counts v2's
  epoch snapshots, 16 KiB, with each worker's warm candidate arena and with its cold arena
  where the candidate's is the larger (the descriptors are on the stack). On the primary
  corpus that adds 64 KiB (four workers' warm arenas; production's cold table is the larger)
  to about 1.86 GB. The preflight's conditions are unchanged:
  - required = max(estimate, the verified dry run's peak); `MemAvailable` ≥ required + max(2
    GiB, 25% of required);
  - no swap in use; no memory pressure where PSI exists; every source read or the preflight
    fails;
  - `pswpin`/`pswpout` recorded before and after, any increase INVALID (G9); swap or pressure
    after the last round INVALID (G9);
  - any major fault in a measured build INVALID (G10); minor faults reported only;
  - a failed preflight means no run, a failure record, and the run archived. The corpus, the
    regimes and either arm's allocation policy are never changed to fit the machine.
- **The capture consumes a verified dry run** (G9), as draft 4, and **it must be v2's (A5):**
  besides draft 4's conditions (exit status 0, clean source, this binary's and corpus's
  sha256, a dry run's analysis with every schedule check and G3, G4, G6–G10 passed, a
  completed uninjected dry run of the same corpus, rows, binary, CPU selection and machine
  identity, and a plausible peak), its records and its analysis must carry the identifier
  `c30-construction-v2`. A v1-mode dry run of the same binary is refused.
- **Failure,** as draft 4: a failure record, printed, exit status 3, the recipe's analysis and
  archive whatever the outcome; fault injection in dry runs only, rejected in a capture.

## 4. The cache-state asymmetry (disclosed, not corrected)

As draft 4. No cache flushing or neutralising step is added. After each step each worker
checks its own output (G6): P reads its 4n-byte T-vector and its 4n-byte reference; C reads its
records and RAW groups (about 2.16n bytes) and the oracle image of the same size. So the next
timed build starts with that arm's arena and reference partly in cache, by construction of
the arms. The ratios compare each builder in its own natural reuse cycle (build, check,
build); they don't isolate the builders under identical cache state, nor reproduce a decoder.

**Disclosed for v2 in particular:** v2 finishes a portion at its next arrival or at the end,
possibly long after the portion's rows were written, so its finishing reads fields that may
have left the cache. That is part of v2's cost, inside its window, and this protocol measures
it only as part of b4 and e4.

## 5. The measures

As draft 4:
- **Clocks:** `CLOCK_THREAD_CPUTIME_ID` for build CPU time (b1, b4); `Instant` for starts,
  ends and every check window (e4, the start skew, every ordering check); `sched_getcpu`
  before the start barrier and after the completion barrier (S4); `getrusage(RUSAGE_THREAD)`
  for major (G10) and minor (reported) faults. The window's calls, in order: monotonic, CPU,
  rusage, the build, then rusage, CPU, monotonic.
- **The clock calls' overhead (G5):** from each calibration batch, the median cost per CPU per
  clock; the largest median times the windows in a pass, as a share of the pass's build CPU
  time; at most 0.5% in every arm, regime, mode and round.
- **Per arm, regime and round:** W1 cost Σ build CPU ns ÷ Σ rows; W4 cost Σ over the four
  workers of build CPU ns ÷ Σ rows; W4 completion time Σ over steps of (latest build end −
  earliest build start). It is not decoder throughput, and not the capture's wall time.
- **The ratios, paired by round, C ÷ P:** **b4** (warm, decides), **e4** (warm, decides), b1
  (warm, reported), b4ᶜ, e4ᶜ, b1ᶜ (cold, reported, never a gate). The estimate is the
  geometric mean of the 20 round ratios with a 95% t interval (19 degrees of freedom) on their
  logs; no interval from any other number of rounds.
- **Full-block and short-block economics** (reported): b4, b1 and e4 over the 230 full blocks
  and the 204 short blocks; the mixed step (step 57) reported on its own.
- **The start skew,** draft 4's per-step definition, unchanged: for one W4 step whose four
  builds are all of full blocks (an **all-full step**), (latest start − earliest start) ÷
  (latest end − earliest start). v2's G2 reads it (§6).
- **Stability (reported, A3):** for both arms, in both regimes, the per-round W1 cost, W4 cost
  and W4 completion time's max ÷ min, with the rounds of the extremes. Only P's warm W4 cost
  and completion gate (G1).
- **Memory (never inside a timed window):** actual capacities per worker and block (P: `tt`'s
  capacity × 4; C: `rec` × 64 + `raw` × 128, labelled rec + raw), each also with
  `malloc_usable_size`. v2's bitmap and snapshots are private to the frozen `ctor2.rs` and are
  not observed: the bitmap's requested size, 8·⌈G/64⌉ bytes, is recorded per build as for v1,
  and the snapshots' 16 KiB in the meta record (`candidate_scratch_bytes`). **Peak RSS** in six
  separate `cbuild-rss-v2` processes (P, C, P, C, P, C), each loading only the last columns and
  counts, validated before any summary as draft 4, and each carrying the v2 identifier.
- **The slow paths, interpreted, not timed (A7):** a separate untimed pass before round −1 runs
  `Ctor2::build::<true>` on every block, in its own arena, and records v2's counters per
  block: bulk, fragment and one-row rows; runs; complete groups; finishes by class (fast,
  ordinary, wide, crossing, forced); normalisation (full and partial groups, fields); RAW
  fields read and entries written; fields restored; epoch searches, epochs, runs split at an
  epoch, multi-epoch portions; RAW groups, moved, cycles; the largest ordinary and smallest
  wide span. The analysis reports, deciding nothing, the per-block warm W1 ratio (the median
  over rounds) by deciles of the one-row share, with each decile's fragment share, non-fast
  ordinary, wide and crossing finishes, and normalised fields a row. (Draft 4's table is also
  reported, reading v2's wide finishes as its promotions.) No profiler, hardware counter or
  extra instrumentation runs during a measured round.

## 6. The checks, in order

The analysis runs three stages; each runs only if the one before it passed.

### Stage 1: the schedule (S), before any figure

S0–S10 as draft 4, with one addition:

| check | condition |
|---|---|
| **S1_protocol (A5)** | the meta record carries `protocol: "c30-construction-v2"` and `candidate: "ctor2"`. Otherwise the run is INVALID and no figure is computed, whatever else it holds. (The analysis's default, draft 4's mode, refuses records carrying v2's identifier.) |

### Stage 2: the gates (G), on a schedule that passed

| gate | kind | condition |
|---|---|---|
| **G1 (A3)** | stability | **P's** per-round **warm W4 cost** and **warm W4 completion time**, each max ÷ min ≤ 1.05 over the 20 measured rounds. W1 in both regimes, cold W4, and every C stability ratio are reported and decide nothing |
| **G2 (A4)** | simultaneous start | for **each arm** separately, over its **warm** W4 all-full steps in the 20 measured rounds (N steps; on the primary corpus N = 57 × 20 = 1,140): **(a)** the row-weighted aggregate, Σ R·k ÷ Σ R over those steps, with k draft 4's per-step skew and R the step's rows, is ≤ 0.02; **(b)** the number of steps with k > 0.02 is ≤ ⌊N / 1000⌋ (1 on the primary corpus); **(c)** every step has k ≤ 0.10. The analysis reports, per arm, N, the aggregate, the maximum, the outlier count, the allowance and every outlier's location (round, step, arm, skew); and the same for cold steps, which decide nothing; and the largest per-round aggregate per arm and regime (draft 3's measure) |
| G3 (A6) | identity | the corpus hash; the toolchain (rustc 1.97.0, `2d8144b78`, LLVM 22.1.6); the pin with no tracked changes; and the listings of the timed path, compared by `bench/asm_listing.py`'s normalisation (`bench/c30_build_listing.py --protocol v2`) with the frozen references in `2026-11-03-compact30-construction-v2-harness/listings/`: for C, `<c30walk::ctor2::Ctor2>::build::<false>` (0x18e5 bytes), `W<false>::finish` (0x5fa), `W<false>::refill` (0xec), `c30walk::ctor::canonicalise` (0x305, v1's reviewed listing) and `c2_build` (0x15); for P, `p_build` and `mibz2::bwt::build` (0xe97), **v1's reviewed listings, unchanged**. Also checked: `p_build` only transfers to `bwt::build`; `c2_build` only calls `Ctor2::build::<false>`; `bwt::build` inlines both passes; every `c30walk` function the constructor's checked functions call is itself checked. The binary's sha256 is recorded but isn't an identity. No run if G3 fails |
| G4 | allocation | as draft 4 |
| G5 | calibration | as draft 4 |
| G6 | exact output | as draft 4, with C's arrays from `Ctor2::build::<false>`: in every round and regime every build of both arms passed its check, after the completion barrier, outside the window. **C:** the call returned true, and `rec` and `raw` equal the block's oracle image (`C30::build`, permille 0) in length and every byte. **P:** `tt` equals the block's reference T-vector |
| G7 | walker output and complete initialisation | as draft 4, for v2: (a) in round −1's W1 warm pass C's record arena's spare capacity is poisoned before its build, the coverage recorded per block; (a′) the untimed poison pass builds every block with `Ctor2::build::<false>` into one preallocated, poisoned arena, and every block's poison must survive and its arrays equal the oracle's; (b) the unchanged walker on C's arrays and `walk_segments_room` on P's table each give production's output; (c) C's arrays compared again after the walk |
| G8 | production's one-pass path | as draft 4 |
| G9 (A5) | memory | as draft 4, and a capture's dry-run evidence must be v2's (its records and analysis carry the identifier) |
| G10 | major faults | as draft 4 |

**What G6 and G7 rest on:** v2's equivalence and walker exactness were established by its
prototype suites and audit (§0), and are re-checked here on the measured corpus and build.
Complete initialisation is shown on every block by the poison pass (G7a′), and in round −1
wherever the poisoned allocation survived. No memory-safety tool runs during the capture.

**The dry run** (rounds −1, 0 and 1) runs the preflight, S (with S1_protocol), G3, G4 and
G6–G10. It suppresses every timing figure, G1, G2, G5 and the decision. Its archive is the
evidence a capture on the same machine consumes (G9).

### Stage 3: the figures and the decision (§7), on a valid run only

## 7. The decision (fixed before any timing)

Unchanged from draft 4. **The deciding ratios,** both from the **warm four-core** regime over
the **complete primary corpus**: **b4**, the build CPU-cost ratio C ÷ P, with 95% interval
[b4−, b4+]; **e4**, the construction-step completion-time ratio C ÷ P, with [e4−, e4+].

| outcome | registered condition |
|---|---|
| **PASS** | b4+ ≤ 1.06714 **and** e4+ ≤ 1.02 |
| **FAIL** | b4− > 1.06714 **or** e4− > 1.02 |
| **INCONCLUSIVE** | neither PASS nor FAIL. No automatic extra rounds or rerun; the guardian decides |
| **INVALID** | any schedule (S, S1_protocol included), correctness (G6, G7, G8), identity (G3), stability (G1 as revised), calibration (G5), start (G2 as revised), allocation (G4), memory (G9) or major-fault (G10) check fails, or a failure record exists. On an S failure or a failure record no figure is computed; on a gate failure the figures are reported as an invalid run's, with no decision |

- **1.06714** is the construction CPU ceiling (§1), unchanged. **1.02** is the completion-time
  no-harm allowance, a policy choice adopted before any construction timing (report 191),
  unchanged. e4 is construction-step completion time, not decoder throughput.
- **What each outcome authorises,** as draft 4: PASS, drafting a separately approved
  production-shaped concurrency and end-to-end evaluation, with no integration and no claim
  about the decoder; FAIL, constructor v2 stops at this checkpoint as an investment decision,
  with the implied S = 0.592(1 − b4) + 1.847(1 − w4) at the intervals' ends recorded, and
  whether e4 decided; INCONCLUSIVE, b4, e4, both intervals and the implied S reported for the
  guardian.
- **Reported in every outcome, deciding nothing:** b1; the cold regime's b4ᶜ, e4ᶜ, b1ᶜ with
  their intervals and minor faults; the full- and short-block economics; each arm's CPU ns a
  row at W1 and W4, warm and cold; the running fraction; the clock-call shares; **the
  stability ratios (A3)**; **the start skew, warm and cold, with every outlier's location
  (A4)**; the capacities and peak RSS; the slow-path tables; the preflight and postflight
  memory records.

## 8. What it cannot show

As draft 4: the decoder's result (MTF, RLE1, CRC, the scanner, the pipeline and the walk after
each build are absent); the cache state each build leaves for its walk; a cache-neutral
comparison (§4); the decoder's four-core mix; throughput; allocator independence; other
corpora or machines; why a result came out as it did beyond the untimed counters'
associations. Also: whether v2's late finishing reads (§4) would cost the same inside a
decoder, where other work runs between a portion's rows.

## 9. The VM checks, the records and the command (not authorised)

**What is built** ([the harness journal](2026-11-03-compact30-construction-v2-harness.md)), at
pin `fa21f33`:
- `c30walk cbuild-v2` and `c30walk cbuild-rss-v2`: the collector's v2 mode. The collector is
  generic over the candidate; `cbuild` and `cbuild-rss` remain draft 4's.
- `bench/c30_construction.py --protocol v2` (`analyse_v2`) and `bench/test_c30_construction_v2.py`.
- `bench/c30_build_listing.py --protocol v2` and the frozen references.
- `bench/test_c30_construction_evidence.py ... cbuild-v2`.
- Neither constructor, the walker nor `src/` changes.

**VM checks before any capture (untimed):**
- the analysis's regression tests: draft 4's suite (in its own mode, and again under v2 with
  its five G2 tests replaced) and v2's G1, G2 and identifier tests, including isolated
  moderate interruptions (VALID), widespread staggering and outliers above the allowance
  (INVALID), a single 33.3% step (INVALID), a step just above 10% (INVALID), the allowance at
  its limit and one over, W1 or cold instability alone (reported, VALID), and P's warm W4
  instability (INVALID);
- the collector's evidence rejections, run directly on the binary, with v2's protocol cases;
- a dry run on the primary corpus, and a `verify-evidence` run consuming its archive;
- the failure path by injection: a corrupted output in C; a corrupted expected copy for P,
  in G6 and in G7; a pinning failure; a forced C rejection; a forced preflight failure;
- G3's listing review;
- `barrier-test`, unchanged;
- **draft 4's mode unchanged:** v1's archived analyses replayed with the new analyser,
  byte for byte.

**The recipe** (`just bench-c30-construction-v2`) is draft 4's §9 with v2's collector,
analysis, listings and tests:
1. the corpus hash;
2. the toolchain, rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), selected with `rustup run 1.97.0`;
   stop if it is missing or misidentified;
3. the build, `CARGO_ENCODED_RUSTFLAGS=`, `--locked`, the release profile (codegen-units 1,
   fat LTO, line tables), baseline x86-64, into `target/rust-1.97.0/` of the pin's worktree
   (`target/c30-construction-v2-pin`);
4. the analysis's regression tests (draft 4's and v2's) and G3's;
5. G3 on the timed path; no run if it fails;
6. the CPUs: the second to fifth physical cores (the first four where there are only four);
7. for a capture, a v2 dry run's archive: kept, with its checksum, in the capture's
   directory, then unpacked for the collector;
8. `barrier-test` on those CPUs;
9. the collector, under a two-hour timeout;
10. the six `cbuild-rss-v2` processes;
11. the analysis (`--protocol v2`, with the evidence for a capture), whether the run passed or
    failed;
12. the archive, whether the run passed or failed: `target/c30-construction-v2-<UTC
    time>[-mode].tar.gz`, with a `MANIFEST.json`.

**The sequence on a workstation,** once authorised:
1. `just bench-c30-construction-v2 <corpora> 20 dry-run`;
2. `just bench-c30-construction-v2 <corpora> 20 capture <that dry run's .tar.gz>`.

**Not authorised now:** any construction timing, and every step on the i9 or any workstation.

## 10. The guardian's decisions

Draft 4's §10 (reports 191, 192, 195 and 197) holds, except where report 211 decides
otherwise:

| report 211 | decision | where applied |
|---|---|---|
| The audit's PASS | accepted; no further Miri marathon required | §0 |
| The comparison | P production's `bwt::build`; C `Ctor2::build::<false>` at `8beffe2`, natural configuration; same corpus, 20 paired rounds, four physical workers, frozen compiler recipe, matched full-construction boundaries | §1, §2, §3 |
| C's boundary | descriptor and snapshot initialisation, temporary writes, deferred finishing, normalisation, RAW work, allocation, padding and publication inside the window | §2 |
| The deciding measures and limits | warm four-core CPU cost and step completion; b4+ ≤ 1.06714 and e4+ ≤ 1.02 for PASS | §7 |
| G1 | P's warm W4 cost and completion stability at max ÷ min ≤ 1.05; W1 and cold reported | §6 |
| G2 | each arm's warm W4 all-full steps: aggregate ≤ 2%; ≤ 0.1% over 2%; none over 10%; maximum, aggregate, outliers and locations reported | §6 |
| v1's capture | not reinterpreted or reclassified; its INVALID stands | header, A.1 |
| The safeguards | correctness, memory, clock calibration, binary identity, evidence provenance and archiving retained | §3, §5, §6 |
| The harness | reuse the collector and analysis with a distinguished v2 mode and frozen reference listings; the safe `TVector` accessors; production's builder identical to its reviewed listing | §2, §6, §9 |
| Authorisation | untimed VM validation only; no measured benchmark, i9 capture, constructor tuning, walker change or production integration | header, §9 |

**Open for the guardian's review:** the protocol itself, the pin, the harness, its tests and
the VM dry run, and the proposed workstation command in the harness journal.
