# bench-parscan-stream: registration

*22 November 2026. A draft for review, written before any data, on `bench/parscan-stream`. The
recipe is `just bench-parscan-stream` (`.just/bench.just`). The verdict is
`bench/parscan_stream.py`'s, tested by `bench/test_parscan_stream.py`: a wrapper around
`bench/parscan_verdict.py`, which is bench-parscan-c37's verdict script copied unchanged in its
logic (that recipe's Decision 1, PARSCAN). The recipe follows bench-block-reader-on-wormhole's
(its builds, contrast check, corpus and machine records). What differs from that recipe: the
nine-count grid and the PARSCAN rule (judged at 10, 6 and 8 supporting), the default decoder on
both sides, and an untimed preflight before the timed run. It is frozen once the Coordinator
accepts its review; until then nothing here is final and no run is queued.*

**The decision it settles.** Whether perf/parscan-stream (#770) merges: the push-streaming
parallel front end in place of the default decoder's serial scanner on the i9-10900K from 8
threads asked. It is an adoption registration (RULES C6): its verdict decides a production
default on the machine production runs on, at thread counts CI's 4-CPU runner never reaches
(CI's bench runs 1, 2 and 4 threads, where the PR leaves the default decoder unchanged).

- **ADOPTS:** the PR merges and ships in the next release; DELIVERED.md gets its line.
- **HELD ON MEMORY:** the Coordinator weighs the growth under PERFORMANCE-POLICY rule 4, from
  this run's figures alone.
- **NOT ADOPTED:** the PR does not merge. `StreamDecoder::parscan`, `BzReader::parscan` and
  `decode_parscan` are not split out on this verdict; any later attempt is its own PR with its
  own registration. A loss at 8 alone would be read as evidence for a narrower activation (from
  10 threads), which is a new PR, not a re-run.
- **VOID:** repeated once, with the reason logged.

## The hypothesis

**What changed.** The PR's head is master's `cb7121e8` plus the parallel front end of the
final handoff on `research/carbon-word-transport`
(`docs/journal/2026-11-22-carbon-final-handoff/`), its parscan items alone: `SlabRope`,
`ParScanHub`, the single-slab scanner, the helper scanners, `read_item_tight`, the prepared
records (`runs::PreparedRuns`, `decode_prepared_runs`). Not its CRC, RLE1, MTF, Huffman-table or
`runs` kernel items. All safe Rust (`src/par_scan.rs` is `#![forbid(unsafe_code)]`), no
allocator change.

- **The default path (`StreamDecoder::new`, so `BzReader::parallel` and `decode_parallel`):**
  on the measured i9's profile (`measured_i9`: the i9-10900K, its 20 CPUs on 10 cores, no CPU
  quota, AVX2 and CLMUL), from 8 threads asked, where the front end's topology has a helper
  scanner, the decoder is the front end's (`parallel::parscan_here`):
  - unpinned at 8 and 9 threads: the plan's own lazy workers (8 or 9, as master runs), with the
    lead scanner and two helpers;
  - unpinned from 10 threads: 8 lazy workers (`PARSCAN_TRIMMED_WORKERS`) where master's serial
    scanner runs 7, the window 4 blocks a worker, the lead scanner and two helpers;
  - pinned one CPU a core at 8: master's trimmed 6 workers and one helper; at 10, 8 workers and
    two helpers.
  - Everywhere else, below 8 threads included, the default decoder is master's, unchanged.
- **The front end:** input in 4 MiB recycled slabs (`SlabRope`, at most 8 MiB live, 2 chunks
  queued); a lead scanner that finds every block and end magic by a 4-byte anchor scan and
  shares the candidates' symbol decoding with demand-driven helpers; at most `workers + 4`
  blocks ahead of the reader (`max_front_ahead`); a chain that keeps only the candidate at its
  next bit, in order, into master's workers, ready ring and 12 MiB `READY_BYTES` cap.

**What it predicts.** At 10 threads, the serial scanner is the decoder's bound on the i9: the
plateau from 8 to 20 threads, and Decision 2 of bench-parscan-c37, whose faster block reader
alone gave 0.932 at 10 and 0.884 at 8 (evidence `5fed1ffc`). Moving the scan off one thread, and
running 8 workers in place of 7, should make 10 threads FASTER. The handoff's own 10-thread
figure is 0.9523 on wikidata-l9 (a 2-core, 4-vCPU host, same binary, 7 rounds): a claim, not
evidence here. Its modelled 10-core critical path is 0.72.

**The risks, named:**
- **At 8 threads,** the front end runs 8 workers and three scanner threads on 10 cores, where
  master runs 8 workers and one scanner. Helpers park when 3 jobs are queued, but the lead
  scanner's coordination may cost more than it saves where the scanner is not the bound.
- **Speculative decoding:** a candidate past an end magic is read at level 9, and read again at
  its stream's level if that read fails (one Wikidata stream: no such candidate here).
- **Memory:** 8 workers where master runs 7, plus the rope. The handoff measured peak RSS at
  0.81 of master at 10 threads; the 1.10 gate is at 10.

**How the port avoids #741's two failures** (its Decision 1, NOT ADOPTED: 1.111 at 6 threads,
peak RSS 1.275 at 10; LOG 7076671871):
- **The loss below 8:** below 8 threads the default decoder is master's own, byte for byte the
  same code (`parscan_here` is false). 6 threads can still differ from the base by layout alone.
- **The memory:** no whole-input read (input in bounded slabs, at most 8 MiB live plus 2 queued
  chunks), at most `workers + 4` blocks ahead, the ready bytes capped as master's are, and no
  per-thread multiple of in-flight blocks.

## What is compared

- **Head:** perf/parscan-stream at `d32da8fd576bbf3d2aca20122506f4ad81c712af`, arm `mibz2` (`BzReader::new` at one thread,
  `BzReader::parallel` above).
- **Base:** `cb7121e8d391dc7808f06a8acae53001874deab4`, master at the PR's cut, arm
  `mibz2@build-base`. It is an ancestor of HEAD; the recipe checks it.
- **The contrast, checked by the recipe:** the builds' `src`, `examples`, `tests`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in
  `src/{block,lib,par_scan,parallel,read,runs}.rs`, `src/parallel/unwind_tests.rs`,
  `examples/ab.rs`, `tests/par_scan.rs`, `tests/parscan_stream.rs`, `tests/service_default.rs`
  and `tests/common/mod.rs`, or the run stops.
- **Should master move before the run,** the run still measures the PR against this base; the
  Coordinator weighs the later change at the merge. (Since the cut, master has taken the
  0.5.13 version bump and a queue entry, #769: no `src` change.)

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run, and on any other machine the head's default decoder is the
  base's at every count (the dispatch check is `parallel::front_end_tests`).
- **The corpus:** `corpora/wikidata.bz2` (431 blocks, 209.7 MB decoded), checked by
  `bench/wikidata.json`'s SHA-256, four copies end to end (`wikidata-x4`, 838,800,784 bytes
  out).
- **The cells:** `ab run --arms mibz2@build-base,mibz2` at 1, 2, 4, 6, 8, 10, 12, 16 and 20
  threads. Above one thread unpinned on every CPU (no `--spread`, no spinner); at one thread on
  ab's one-CPU pin, for both arms alike.
- **The preflight, untimed:** one round of every cell with no settling, before the timed run
  (`preflight.jsonl`). `bench/parscan_stream.py --preflight` checks it as the verdict checks a
  run: the machine, every record present, successful, unpinned above one thread, and every
  output 838,800,784 bytes with one hash. A failure stops the recipe before the timed run and
  is not a verdict; `just bench-parscan-stream 1` (DRY) runs the preflight alone. No step is
  taken from it.
- **Rounds:** 16, each running both arms at each count, interleaved, the arms' order rotated
  (`arm_order`; 16 is a multiple of 2 x 2 arms, so each arm takes each position equally often),
  with a **2 s settle** before each run. The settle matters: at these cells, with a 0.25 s gap,
  wall time is bimodal by the run's position in the round (about 0.25 s against 0.39 s, paired
  log SD about 0.33), and with 2 s every run falls in one mode (paired log SD 0.01-0.06); REVIEW
  7084586885 on #145, and #318's idle-gap effect.
- **Cost:** about 35 minutes of workstation time with the two builds (9 counts x 2 arms x 17
  rounds, about 6 s a run with its settle, plus two release builds).

## The rule

R4's, as bench-parscan-c37's Decision 1. A count's **step** is the 16 rounds' paired wall
ratios, head over base: their geometric mean and its 95% interval (Student's t on the logs, 15
degrees of freedom). FASTER if the interval lies below 1, SLOWER if above, INCONCLUSIVE
otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER;
  - neither 6 nor 8 threads is SLOWER;
  - at 10 threads, the median peak RSS over the rounds, head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at 10 threads.
- **NOT ADOPTED:** any other outcome: 10 not FASTER, or 6 or 8 SLOWER.
- 6 and 8 are **supporting**: only a loss there counts against adoption.
- 1, 2, 4, 12, 16 and 20 are **reported**: their steps, CPU and peak RSS are printed and decide
  nothing. A SLOWER step at 12, 16 or 20 (where the head runs the front end) would be weighed by
  the Coordinator under PERFORMANCE-POLICY rule 2 at the merge, from this run's figures.
- **The transfer**, reported and deciding nothing: at 10 threads, ln(step) / ln(0.9523), the
  share of the handoff's claimed 10-thread log gain that the i9 shows.
- **VOID,** with no verdict, if any of these holds: the machine is not the i9's profile; any run
  is unsuccessful or has a non-finite or non-positive wall, CPU or peak RSS; any run above one
  thread is pinned, any one-thread run is not on a single CPU, or any run is beside a spinner;
  any record is duplicated, unexpected or missing at any of the nine counts; the corpus is not
  `wikidata-x4`; any output is not 838,800,784 bytes, or the outputs' hashes are not all one.
- **One run, one verdict.** The verdict reads `main.jsonl` alone. A verdict is never re-run to
  change it.

## Why this run (PERFORMANCE-POLICY 4a)

- **The decision on each outcome:** above.
- **The causal relevance of each condition:**
  - **10 (judged):** the head runs the front end with 8 workers; master's serial scanner is the
    measured bound there (Decision 2's 0.932 at 10 from a faster reader alone).
  - **8 (supporting):** the head runs the front end with 8 workers and two helpers against
    master's 8 workers and one scanner: the topology most likely to lose.
  - **6 (supporting):** the head's default decoder is master's code (`parscan_here` false);
    included because it is where #741 failed, to show that the activation rule holds there.
  - **1, 2, 4 (reported):** the same code on both sides; they bound layout effects.
  - **12, 16, 20 (reported):** the front end, past the cores.
- **Why cheaper evidence won't do:** CI's runner has 4 CPUs and its bench runs 1, 2 and 4
  threads, where the PR is inactive; the gain claimed is the 10-core i9's serial-scanner bound,
  which only the i9 has. The handoff's figures are from a 2-core host, where 10 workers share
  two cores. The static path (`parscan_here`) and the dispatch tests settle where the code runs,
  not what it costs there.

## Limits

- **Wikidata alone:** no text, records or random corpus on the i9. CI's bench covers every
  corpus at 1, 2 and 4 threads, where the head's default decoder is the base's.
- **What an ADOPTS shows:** the whole decoder's gain at 10 threads on the i9, not its cause (the
  parallel scan, the 8th worker, or the prepared records); they ship together.
- **What it does not cover:** another 10-core machine (the activation is the measured i9's
  profile alone), and multi-stream input (one stream here; the tests cover its correctness).

## Result

*Added after the run; nothing above this section changes once it is frozen.*
