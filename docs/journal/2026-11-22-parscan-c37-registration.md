# bench-parscan-c37: registration

*22 November 2026. Written and frozen before any data, on `bench/parscan-c37`. The recipe is
`just bench-parscan-c37` (`.just/bench.just`). The verdict is `bench/parscan_c37.py`'s, tested by
`bench/test_parscan_c37.py`. The recipe, its checks and the verdict script follow bench-open-r5's
(#735, `docs/journal/2026-11-22-open-r5-registration.md` on `bench/open-r5`) and R4's rule. What
differs: the two arms have different names, the grid is wider (nine counts, six of them reported
only), the memory gate is at 10 threads alone, the handoff's transfer ratio is reported, and
there are two decisions, each judged on its own run.*

*Amended before any data (the Coordinator's decision, 22 November 2026; C2 holds): the
default-path diagnostic is now Decision 2, BLOCK-READER, judged by its own rule (below).
Decision 1, PARSCAN, is unchanged.*

**The decisions it settles.** Two, each with its own run, rule and verdict line:
- **Decision 1, PARSCAN:** whether the new entry point, `mibz2-parscan`, beats master's
  parallel path on the i9.
- **Decision 2, BLOCK-READER:** whether #741's change to the default decoder (its new block
  reader and `takes_runs`, on every path) holds on the i9. CI's bench measured it FASTER on
  CI's runner (Wikidata -2.8% wall at 1 thread, -4.1% at 4; the scanner stage -19.4% c/B); this
  is the i9 evidence.

**Decision 1, in full:** whether #741 (`perf/parscan-c37`, "perf(parscan): Carbon's cycle 37
parallel front end, `decode_parscan`, beside the default path") is adopted: merged, as the
candidate to replace the default parallel path's serial scanner in a later handoff. It is an
adoption registration (RULES C6): no production default changes on this verdict alone, but the
PR also changes the default decoder's block reader (below), so its merge is decided here, on the
i9, where production runs.

## The hypothesis

**What changed.** #741's head is master's `96a24d3e` plus the port of Carbon's interim handoff 0
(`research/carbon-word-transport` at 6a2d5dadf,
`docs/journal/2026-11-21-carbon-handoff0-c37-dropin.md`). It changes `src/bits.rs`,
`src/block.rs`, `src/runs.rs`, `src/stream.rs` and `src/lib.rs`, adds `src/huff.rs` and
`src/par_scan.rs`, and changes `examples/ab.rs` and `examples/par_scan.rs`. All safe Rust (the
five decoder modules it touches or adds are `#![forbid(unsafe_code)]`), no allocator, routing or
admission change.
- **The new path, `mibz2-parscan`** (`par_scan::decode_parallel_slice_with`): every worker first
  scans a disjoint share of the input for block and end magics (a 4-byte anchor every 4 bytes,
  an 8 KiB bitmap), then claims candidate blocks from one atomic cursor, decodes each in full
  (header, symbols, inverse BWT on `OpenCells`) and hands it to a joiner on the calling thread,
  which keeps the blocks whose start is the chain's next bit and drops the rest. W threads in
  all, where master's parallel path runs a serial scanner beside its W workers.
- **The default path, `mibz2`:** unchanged in its plan, routing and kernels, but every block's
  header and symbols are now read by the new block reader (`block::read_item_with_scratch`:
  `huff::Huff`'s 10-bit table with an EOB sentinel, a 2 KiB second-level table for 11-20-bit
  codes, a 264-byte group window, a selector MTF in one u64), and `runs::takes_runs` stops
  counting once its answer is known. Both are claimed exact (the same symbols, errors and end
  bit as master's); #741's body has the review.

**The native evidence (the handoff's section 4, Gemini on a 2-vCPU Xeon Platinum 8481C VM).** One
binary linking verbatim `53f7ecf9` (whose src is `96a24d3e`'s) and the candidate, 5 interleaved
rounds, `wikidata-l9` (the corpus here, one copy):
- **10 workers:** 0.9333x wall (master 281.37 ms and 278.27 ms A/A, the candidate 262.60 ms);
- **8 workers:** 0.9637x; **16:** 0.9392x; **20:** 0.9713x;
- **1 thread:** 0.9694x (the block reader alone: `mibz2-parscan` at one thread is the default
  serial decoder);
- **its model of a 10-core critical path:** 0.6027x at 10 workers, because master's serial
  scanner (85 ms, 100% of the modelled critical path from 8 workers up) becomes a 2 ms merge.

These are claims from a 2-core machine, where W workers share two cores; not evidence here. The
modelled critical path is the claim this run tests: on the i9's 10 cores, master's scanner is a
real serial stage only if it bounds the wall at 6 to 10 threads.

**The risk, named.**
- **Speculation's waste.** Workers decode candidates the chain may drop (a false magic inside a
  block, a block whose start the chain passes). On Wikidata every magic is real, so waste is
  low, but each candidate is decoded from its first bit to its end in full, where master's
  scanner reads each block once.
- **The joiner's waits.** The calling thread waits on one condition variable for each block in
  order. At 10 threads on 10 cores, the joiner's wake-ups compete with the workers.
- **Memory.** The arm reads the whole input file into memory first (`std::fs::read`, 47 MiB for
  `wikidata-x4`), where master's arm streams it through `BzReader::parallel`, and it holds up to
  4 x threads decoded blocks in flight. Peak RSS may rise by tens of MiB; the 1.10 gate at
  10 threads may bind on that alone. This is the drop-in's harness arm, not a production
  integration (that is the handoff's Interim Handoff 1).

## What is compared

- **Head:** #741's head, `perf/parscan-c37` at `d57ec321f8d8d7212898219d67a303d995ef2746`, arm `mibz2-parscan`.
- **Base:** `96a24d3e4f764b3918089b0d5bcd5fe4b9bc1fea`, #741's merge-base with master (#738's
  merge, v0.5.12's src), arm `mibz2` (`BzReader::new` at one thread, `BzReader::parallel` above).
  It is an ancestor of HEAD; the recipe checks it with `git merge-base --is-ancestor`.
- **Base and master agree.** When the recipe was written, master's head *was* the base. Should
  master move before the run (the Wormhole plateau stack, #739, touches `src/lib.rs`,
  `src/parallel.rs`, `src/read.rs`, `src/crc.rs` and `src/rle.rs`), the run still measures #741
  against its own base, and the Coordinator weighs the later change at the merge.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in the nine files above, or
  the run stops. The bundle keeps the list (`src-changed.txt`) and the diff.
- **One-thread runs:** both arms run the default serial decoder (`BzReader::new`), the base's and
  the head's; so 1 thread measures the head's block reader alone. Reported, not judged.
- **What it predicts:** a whole-decoder wall gain at 10 threads, and at 6 and 8, if master's
  scanner bounds the wall there. If master's scanner is not the bound at these counts on the i9
  (10 real cores, so the scanner has its own), the step may be near 1, or the speculation and the
  joiner's waits may make it a loss.
- **Memory, reported:** the median peak RSS at each count, base and head, in MiB with the
  difference. Only the 1.10 gate at 10 threads decides anything.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (`wikidata-l9`: 431 blocks,
  209.7 MB decoded), checked by `bench/wikidata.json`'s SHA-256. Four copies end to end make the
  timed file (`wikidata-x4`), as bench-open-r5 and bench-wormhole-plateau. Neither bench-open-r5
  nor bench-wormhole-plateau runs the l1 or records corpora, so this run does not either.
- **The cells:** `ab run --arms mibz2@build-base,mibz2-parscan` at 1, 2, 4, 6, 8, 10, 12, 16 and
  20 threads. Above one thread unpinned on every CPU (no `--spread`, no spinner); at one thread
  on one CPU, ab's pin (`--pin`, default CPU 1), for both arms alike.
- **Rounds:** 16, bench-open-r5's. Each round runs both arms at each count, interleaved, with the
  arms' order rotated (`arm_order`) and a 2 s settle gate before each run.
- **Decision 2's run** (BLOCK-READER), after Decision 1's: `ab run --arms
  mibz2@build-base,mibz2`, the head's default decoder (its new block reader) against the
  base's, at 1, 6, 8 and 10 threads (1 on ab's one-CPU pin, the rest unpinned), 16 paired
  interleaved rounds, the same corpus, settle and checks. Its records are
  `block-reader.jsonl`; Decision 1's are `main.jsonl`.
- **Cost:** about 30 minutes of workstation time with the two builds.

## The rule: Decision 1, PARSCAN

R4's. A count's **step** is the 16 rounds' paired wall ratios, head over base: their geometric
mean and its 95% interval (Student's t on the logs, 15 degrees of freedom). A step is FASTER if
the interval lies below 1, SLOWER if it lies above 1, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER (its interval's upper end below 1.0);
  - neither 6 nor 8 threads is SLOWER;
  - at 10 threads, the median peak RSS over the rounds, head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at 10 threads. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: reported with their intervals, and only a loss there
  counts against adoption.
- 1, 2, 4, 12, 16 and 20 threads are **reported**: their steps, CPU and peak RSS are printed and
  decide nothing, whatever they show.
- **The transfer**, reported and deciding nothing: at 10 threads, ln(step) / ln(0.9333), the
  share of the handoff's claimed 10-worker log gain that the i9 shows (1 all of it, 0 none,
  below 0 a loss), with the same transform of the step's interval.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run above one thread is pinned, any one-thread run is not on a single CPU, or any run is
    beside a spinner;
  - any record is duplicated, unexpected or missing, at any of the nine counts;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the base
  and the head in MiB, with their difference.
- **One run, one verdict.** Decision 1's verdict reads `main.jsonl` alone. A VOID Decision 2
  leaves it as it is.

## The rule: Decision 2, BLOCK-READER

The same step (R4's: 16 paired wall ratios, head `mibz2` over base `mibz2`, geometric mean and
Student's t 95% interval on the logs), FASTER, SLOWER or INCONCLUSIVE as above.

- **ADOPTS:** all three hold:
  - the step at 1 thread is FASTER (its interval's upper end below 1.0). One thread is judged
    because the change is to the serial block reader, and one thread is where the decoder's
    whole wall is that reader's and the rest; there it shows undiluted;
  - none of 6, 8 and 10 threads is SLOWER;
  - at 10 threads, the median peak RSS over the rounds, head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at 10 threads. The Coordinator
  weighs it under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 1 thread not FASTER, or any of 6, 8 and 10 SLOWER.
- 6, 8 and 10 threads are **supporting**: only a loss there counts against adoption. No count
  is reported only, and no transfer is reported (the handoff's claim is Decision 1's).
- **VOID**, with no verdict: Decision 1's conditions, over `block-reader.jsonl` and its four
  counts (the roster `mibz2@build-base` and `mibz2`, at 1, 6, 8 and 10). A VOID Decision 2
  leaves Decision 1 as it is, and may be repeated once, with the reason logged.
- **Reported beside each step, deciding nothing:** CPU time and the median peak RSS, as above.
- **The verdicts' file:** `verdict.txt` holds both, Decision 1's first, each on its own line
  (`verdict (Decision 1, PARSCAN): ...`, `verdict (Decision 2, BLOCK-READER): ...`).

**No multiplicity correction across the two.** They are not two tests of one hypothesis, and
neither verdict can stand in for the other. Decision 1 is about a new entry point
(`decode_parscan`, the `mibz2-parscan` arm) against master's parallel path. Decision 2 is about
the default path, `mibz2` against `mibz2`. Each has its own arms, its own run and records, and
its own consequence. No outcome is read as "either one adopting" and acted on, so the family's
error rate is not the quantity at risk: each decision's own 95% rule is. Within each decision,
the rule is R4's as every earlier registration used it: one judged count, the supporting counts
able only to block.

**Limits:**
- **The corpus:** Wikidata alone; no l1, records or `text` corpus. The handoff's records-l9 table
  (0.9986x at 10 workers, 1.02x at 16 and 20) is not tested here.
- **What a result separates:**
  - a Decision 1 ADOPTS shows the new path's whole-decoder gain over master's parallel path,
    not its cause: the parallel scan, the work stealing, or the block reader (which only the
    head side carries);
  - Decision 2 isolates the block reader's effect on the default path. It cannot separate the
    reader from `takes_runs`' early exits, which ship together.
- **What each verdict means:** `decode_parscan` and the `mibz2-parscan` arm are new entry
  points. `decode_parallel` and `BzReader::parallel` keep master's team and plan, but run the new
  block reader. Neither verdict makes `decode_parscan` the default. How the two verdicts combine
  into #741's merge is the Coordinator's call; this registration fixes each verdict, not their
  combination.
- **The head's branch:** the recipe fetches HEAD from `perf/parscan-c37` if the workstation lacks
  it. A later push to #741 is not what this run measures, and needs its own pin.

## Result

*Added after the run; nothing above this section has changed since it was frozen. The bundle is
`parscan-c37/run-20261122T082138Z` on the `evidence` branch at `5fed1ffc`, sha256
`7accddd1273b18ea9923ec74b62efa27f8a06aa66943c16a8abe7cfe464df1bc` (checked). The i9-10900K ran
from 08:21:38Z to 08:39:57Z on 22 November 2026, with the recipe at `d022f2b6`: base `96a24d3e`,
head `d57ec321`.*

**Decision 2, BLOCK-READER: ADOPTS.** The head's default decoder against the base's, as steps
(wall, head/base, geometric mean [95%]):

| threads | role | wall | CPU | peak RSS (median) |
|--:|---|--:|--:|--:|
| 1 | judged | 0.9772 [0.9680, 0.9865] FASTER | 0.9773 | 0.9968 (30.3 → 30.2 MiB) |
| 6 | supporting | 0.9763 [0.9712, 0.9815] FASTER | 0.9588 | 1.0098 |
| 8 | supporting | 0.8837 [0.8623, 0.9055] FASTER | 0.9694 | 1.0196 |
| 10 | supporting | 0.9320 [0.9233, 0.9407] FASTER | 0.9635 | 1.0035 (87.8 → 88.1 MiB) |

Every count was FASTER, in wall and in CPU, with peak RSS within 2%. CI's runner agreed
(Wikidata -2.8% at 1 thread, -4.1% at 4). Master takes the block reader:
`perf/block-reader-c37`, #741's default-path changes without the parscan entry point.

**Decision 1, PARSCAN: NOT ADOPTED** (SLOWER at 6 threads). `mibz2-parscan` against the base's
`mibz2`:

| threads | role | wall | CPU | peak RSS (median) |
|--:|---|--:|--:|--:|
| 1 | reported | 0.9822 [0.9669, 0.9978] FASTER | 0.9826 | 0.9892 |
| 2 | reported | 1.1279 [1.1125, 1.1434] SLOWER | 0.9403 | 0.8873 |
| 4 | reported | 1.1165 [1.1104, 1.1227] SLOWER | 0.9298 | 1.1402 |
| 6 | supporting | 1.1107 [1.1073, 1.1141] SLOWER | 0.9213 | 1.0485 |
| 8 | supporting | 0.9352 [0.9200, 0.9507] FASTER | 0.8905 | 1.0113 |
| 10 | judged | 0.8833 [0.8546, 0.9129] FASTER | 1.0775 | 1.2753 (87.0 → 110.9 MiB) |
| 12 | reported | 0.9122 [0.8980, 0.9266] FASTER | 1.3402 | 1.4319 |
| 16 | reported | 1.0196 [1.0038, 1.0357] SLOWER | 1.7444 | 1.6804 |
| 20 | reported | 1.0720 [1.0604, 1.0837] SLOWER | 1.8956 | 1.9905 (86.0 → 171.1 MiB) |

The judged count was FASTER, with a transfer of 1.798 [1.320, 2.276] of the handoff's claimed
10-worker gain. But the verdict fails on the supporting count 6, and peak RSS at 10 threads
(1.2753) would have failed the memory gate in any case (HELD ON MEMORY at best). Its failure
modes:
- **Slower at 2 to 6 threads** (1.13, 1.12 and 1.11), on less CPU (0.92-0.94). With fewer
  workers than cores, the workers and the joiner wait on each other: wall is lost while CPU is
  not spent. (The cause is a hypothesis; the run measured no waits.)
- **Peak RSS grows with the thread count:** 1.28 at 10, 1.43 at 12, 1.68 at 16 and 1.99 at 20
  (+85 MiB). The arm reads the whole input first, and holds up to 4 x threads decoded blocks in
  flight. Master's decoder stays near 86-94 MiB from 8 threads up.
- **CPU rises from 10 threads up:** 1.08 at 10, 1.34 at 12, 1.74 at 16 and 1.90 at 20. That is
  consistent with speculation and contention (candidates decoded ahead of the chain, the
  joiner's condvar traffic, SMT siblings past 10 workers), again a hypothesis. Wall turns
  SLOWER again at 16 and 20.

These point to Mechanisms' push-streaming Interim Handoff 1 as the fix: a front end fed in
bounded chunks rather than the whole slice, with in-flight output bounded by bytes rather than
by 4 x threads blocks, and no speculative decode past the window that the chain can use. The
parscan entry point stays on #741 (not adopted), and is not on master.
