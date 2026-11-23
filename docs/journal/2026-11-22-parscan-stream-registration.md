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

*Amended 23 November 2026, before any data and before its review. On the i9 from 10 threads the
head runs 8 lazy workers where master runs 7 (`tests/service_default.rs` expects 8), and the
recalibration of 22 November (#145 LOG 7085615681, evidence `472ce552`) measured 8/32 over
7/28 at pinned 10 as 0.935 on Wikidata but 1.109 SLOWER on `text`, and 1.06-1.11 SLOWER on
text unpinned at 12-20 threads. Judged on Wikidata alone, this rule could have adopted the
text loss that rule had just refused. So: the `text` corpus joins the timed cells at every
count; a named loss guard on text (THE TEXT LOSS GUARD, below) and text's peak RSS in the 1.10
gate; base and head moved to master as it now stands (6f7b2ac4, with the 0.5.13 bump and the
moonshot lockfile fix, #773, merged into the PR); and the reason no diagnostic arm separates the
front end from the worker count (below, "No diagnostic arm"). Rounds and settle are unchanged.*

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

**What changed.** The PR's head is master's `6f7b2ac4` (merged in, as a merge commit, on 23
November; the PR was cut from `cb7121e8`, whose `src` is the same but for the 0.5.13 version
in `Cargo.toml` and `Cargo.lock`) plus the parallel front end of the
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
- **The 8th worker on text:** from 10 threads the front end runs 8 lazy workers where master
  runs 7 with 28 in flight. The recalibration measured that change alone, on master's serial
  scanner, as 1.109 SLOWER on text at pinned 10 (0.935 on Wikidata), and 1.06-1.11 SLOWER on
  text unpinned at 12-20 (#145 LOG 7085615681, evidence `472ce552`). The front end may hide
  that loss or carry it; only text at 10 shows which.

**How the port avoids #741's two failures** (its Decision 1, NOT ADOPTED: 1.111 at 6 threads,
peak RSS 1.275 at 10; LOG 7076671871):
- **The loss below 8:** below 8 threads the default decoder is master's own, byte for byte the
  same code (`parscan_here` is false). 6 threads can still differ from the base by layout alone.
- **The memory:** no whole-input read (input in bounded slabs, at most 8 MiB live plus 2 queued
  chunks), at most `workers + 4` blocks ahead, the ready bytes capped as master's are, and no
  per-thread multiple of in-flight blocks.

## What is compared

- **Head:** perf/parscan-stream at `2618c54c5d6f8f6926681655acbfb8acfb2c36ae` (the merge of
  master `6f7b2ac4` into the PR's `d32da8fd`), arm `mibz2` (`BzReader::new` at one thread,
  `BzReader::parallel` above).
- **Base:** `6f7b2ac4223f26e2252bbd30931609cb3a39a1ec`, master as it stood at the amendment and
  the head's merge base, arm `mibz2@build-base`. It is an ancestor of HEAD; the recipe checks
  it. Its `src`, `Cargo.toml` and `Cargo.lock` are master's exactly.
- **The contrast, checked by the recipe:** the builds' `src`, `examples`, `tests`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in
  `src/{block,lib,par_scan,parallel,read,runs}.rs`, `src/parallel/unwind_tests.rs`,
  `examples/ab.rs`, `tests/par_scan.rs`, `tests/parscan_stream.rs`, `tests/service_default.rs`
  and `tests/common/mod.rs`, or the run stops.
- **Should master move before the run,** the run still measures the PR against this base; the
  Coordinator weighs the later change at the merge.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run, and on any other machine the head's default decoder is the
  base's at every count (the dispatch check is `parallel::front_end_tests`).
- **The corpora:**
  - **Wikidata (judged):** `corpora/wikidata.bz2` (431 blocks, 209.7 MB decoded), checked by
    `bench/wikidata.json`'s SHA-256, four copies end to end (`wikidata-x4`, 838,800,784 bytes
    out).
  - **text (the loss guard):** `ab gen DIR 16`'s `text.bz2` (the repository's sources and
    README repeated to about 48 MB, compressed at level 9), the corpus of bench-cores-less-one,
    bench-w12 and the recalibration's text guard. Made afresh in each attempt by the head's
    harness alone (`ab gen` reads the build's own source tree, so the two builds' texts would
    differ), its SHA-256 recorded, and its decoded bytes counted by Python's `bz2` (libbzip2),
    independently of either build; the verdict checks every text output against that count.
- **The cells:** `ab run --arms mibz2@build-base,mibz2` on both corpora at 1, 2, 4, 6, 8, 10,
  12, 16 and 20 threads, the corpora, counts and arms interleaved within each round. Above one
  thread unpinned on every CPU (no `--spread`, no spinner); at one thread on ab's one-CPU pin,
  for both arms alike.
- **The preflight, untimed:** one round of every cell with no settling, before the timed run
  (`preflight.jsonl`). `bench/parscan_stream.py --preflight` checks it as the verdict checks a
  run: the machine, every record present on both corpora, successful, unpinned above one
  thread, and every output of the registered size (838,800,784 bytes on Wikidata, the counted
  bytes on text) with one hash a corpus. A failure stops the recipe before the timed run and
  is not a verdict; `just bench-parscan-stream 1` (DRY) runs the preflight alone. No step is
  taken from it.
- **Rounds:** 16, each running both arms at each count on each corpus, interleaved, the arms'
  order rotated (`arm_order`; 16 is a multiple of 2 x 2 arms, so each arm takes each position
  equally often; text adds a corpus, not an arm, so the balance holds),
  with a **2 s settle** before each run. The settle matters: at these cells, with a 0.25 s gap,
  wall time is bimodal by the run's position in the round (about 0.25 s against 0.39 s, paired
  log SD about 0.33), and with 2 s every run falls in one mode (paired log SD 0.01-0.06); REVIEW
  7084586885 on #145, and #318's idle-gap effect.
- **Cost:** about 50 minutes of workstation time with the two builds: Wikidata as before (9
  counts x 2 arms x 17 rounds, about 6 s a run with its settle, about 31 minutes), text about
  12 minutes more (the same 306 runs, about 2.3 s each with its settle: text decodes in well
  under a second at every count), `ab gen` about a minute, and two release builds.

## The rule

R4's, as bench-parscan-c37's Decision 1. A count's **step** is the 16 rounds' paired wall
ratios, head over base: their geometric mean and its 95% interval (Student's t on the logs, 15
degrees of freedom). FASTER if the interval lies below 1, SLOWER if above, INCONCLUSIVE
otherwise.

The steps are taken per corpus.

- **ADOPTS:** all four hold:
  - on Wikidata, the step at 10 threads is FASTER;
  - on Wikidata, neither 6 nor 8 threads is SLOWER;
  - **THE TEXT LOSS GUARD:** on `text`, none of 6, 8 or 10 threads is SLOWER (its 95% interval
    wholly above 1);
  - at 10 threads, the median peak RSS over the rounds, head over base, is at most 1.10 on
    Wikidata and on text.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at 10 threads on either corpus.
- **NOT ADOPTED:** any other outcome: 10 not FASTER on Wikidata, or 6 or 8 SLOWER on
  Wikidata, or text SLOWER at 6, 8 or 10. A loss outranks the memory gate.
- 6 and 8 are **supporting** on Wikidata: only a loss there counts against adoption.
- **The text loss guard's reason:** from 10 threads the head runs 8 lazy workers where master
  runs 7, the change the recalibration measured as 1.109 SLOWER on text at pinned 10 (#145 LOG
  7085615681, evidence `472ce552`), the result on which our own rule refused it. A rule on
  Wikidata alone could adopt that loss inside the front end. At 6 the head's default decoder is
  master's code (a loss there is layout, and still refuses); at 8 the front end runs with
  master's own 8 workers. Text only removes adoption: a text gain decides nothing, and text at
  1, 2, 4, 12, 16 and 20 is reported.
- 1, 2, 4, 12, 16 and 20 are **reported** on both corpora: their steps, CPU and peak RSS are
  printed and decide nothing. A SLOWER step at 12, 16 or 20 (where the head runs the front end
  with 8 workers; the recalibration's text loss was 1.06-1.11 there) would be weighed by the
  Coordinator under PERFORMANCE-POLICY rule 2 at the merge, from this run's figures.
- **The transfer**, reported and deciding nothing: at 10 threads, ln(step) / ln(0.9523), the
  share of the handoff's claimed 10-thread log gain that the i9 shows.
- **VOID,** with no verdict, if any of these holds: the machine is not the i9's profile; any run
  is unsuccessful or has a non-finite or non-positive wall, CPU or peak RSS; any run above one
  thread is pinned, any one-thread run is not on a single CPU, or any run is beside a spinner;
  any record is duplicated, unexpected or missing at any of the nine counts on either corpus;
  any record's corpus is neither `wikidata-x4` nor `text`; any Wikidata output is not
  838,800,784 bytes, or any text output not the recipe's count of text's decoded bytes; or a
  corpus's outputs' hashes are not all one.
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
- **text (the loss guard, at 6, 8 and 10; memory at 10):** the corpus on which the 8th worker
  lost in the recalibration; without it the rule could not see that loss.
- **Why cheaper evidence won't do:** CI's runner has 4 CPUs and its bench runs 1, 2 and 4
  threads, where the PR is inactive; the gain claimed is the 10-core i9's serial-scanner bound,
  which only the i9 has. The handoff's figures are from a 2-core host, where 10 workers share
  two cores. The static path (`parscan_here`) and the dispatch tests settle where the code runs,
  not what it costs there.

## No diagnostic arm

An arm that held the head's front end at 7 workers at 10 threads would separate the front end
from the worker-count change. The head has no switch for it without a code change, so none is
added (the amendment adds no decoder or harness change):
- `ab`'s arms on the head reach the front end only through `mibz2` (the default) and
  `mibz2-parscan` (`BzReader::parscan`), and both choose 8 workers from 10 threads
  (`parscan_workers`, `PARSCAN_TRIMMED_WORKERS`);
- there is no `mibz2-occ-7-28` arm in the head's or master's `ab` (it lived on the
  recalibration's branches), no environment variable and no flag;
- the one forcing hook, `parallel::with_plan` (`#[doc(hidden)]`, for tests), is called by no
  arm, and under it the front end would take `Plan::Workers(7)`'s eager kernel and the full
  window, not 7 lazy workers with 28 in flight; it would not be the decomposition wanted.

What is known instead: the recalibration's 8/32 over 7/28 on master's serial scanner (0.935 on
Wikidata, 1.109 on text at pinned 10) is the worker-count change alone, on the same machine; this
run's text guard says whether the head carries that loss.

## Limits

- **Wikidata and text alone:** no records or random corpus on the i9. CI's bench covers every
  corpus at 1, 2 and 4 threads, where the head's default decoder is the base's.
- **What an ADOPTS shows:** the whole decoder's gain at 10 threads on the i9, and no loss on
  text at 6, 8 or 10, not its cause (the parallel scan, the 8th worker, or the prepared
  records); they ship together, and no arm separates them (above).
- **What it does not cover:** another 10-core machine (the activation is the measured i9's
  profile alone), and multi-stream input (one stream here; the tests cover its correctness).

## Result

*Added after the run; nothing above this section changes once it is frozen.*
