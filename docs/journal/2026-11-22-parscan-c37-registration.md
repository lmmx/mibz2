# bench-parscan-c37: registration

*22 November 2026. Written and frozen before any data, on `bench/parscan-c37`. The recipe is
`just bench-parscan-c37` (`.just/bench.just`). The verdict is `bench/parscan_c37.py`'s, tested by
`bench/test_parscan_c37.py`. The recipe, its checks and the verdict script follow bench-open-r5's
(#735, `docs/journal/2026-11-22-open-r5-registration.md` on `bench/open-r5`) and R4's rule. What
differs: the two arms have different names, the grid is wider (nine counts, six of them reported
only), the memory gate is at the judged count alone, and the handoff's transfer ratio is
reported.*

**The decision it settles:** whether #741 (`perf/parscan-c37`, "perf(parscan): Carbon's cycle 37
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
- **The default-path diagnostic** (decides nothing): `ab run --arms mibz2@build-base,mibz2`, the
  head's default decoder (its new block reader) against the base's, at 1, 6, 8 and 10 threads,
  16 rounds, reported as steps only, after the judged run.
- **Cost:** about 30 minutes of workstation time with the two builds.

## The rule

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
- **The diagnostic decides nothing.** The verdict is the judged run's alone (`verdict.txt`). A
  VOID diagnostic leaves the verdict as it is.

**Limits:**
- **The corpus:** Wikidata alone; no l1, records or `text` corpus. The handoff's records-l9 table
  (0.9986x at 10 workers, 1.02x at 16 and 20) is not tested here.
- **What a result separates:** an ADOPTS shows the new path's whole-decoder gain over master's
  parallel path, not its cause (the parallel scan, the work stealing, or the block reader). The
  diagnostic shows the block reader's share on the default path, and decides nothing.
- **What adopting means:** `decode_parscan` and the `mibz2-parscan` arm are new entry points;
  `decode_parallel` and `BzReader::parallel` keep master's team and plan. Adopting merges #741,
  and with it the new block reader on the default path; it does not make `decode_parscan` the
  default.
- **The head's branch:** the recipe fetches HEAD from `perf/parscan-c37` if the workstation lacks
  it. A later push to #741 is not what this run measures, and needs its own pin.
