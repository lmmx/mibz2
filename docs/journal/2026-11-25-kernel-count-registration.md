# bench-kernel-count: lazy against eager below 8, and the worker-count cap (registration)

*Registered before any data. A C1a diagnostic: its rule is this document, posted on #145
before the data, and it decides nothing. The finding it follows up is stage 1's (journal §66,
`docs/journal/2026-11-24-algorithm-level.md`; bundle `character/run-20261125T001135Z` on
`evidence`), which §66 sends to the programme's run 4 as a production-default question. This
run only measures; a later adoption registration (RULES C6) decides.*

## The question

Stage 1 measured, through the public API `BzReader::parallel` on the i9-10900K, unpinned:
- **Eager below 8.** Asking for 3, 5 or 7 threads builds *eager* workers (`bwt::Cells`, a slot
  for every letter), because `Plan::lazy` starts at `LAZY_FROM_THREADS = 8` and the measured i9
  cells cover only 4, 6 and 10 or more. At 7 that was 0.337 s and 140 MB, against 0.209 s and
  54 MB for 7 lazy workers in stage 1's grid. Stage 1's grid had lazy workers only, so it has
  no paired eager arm at any k.
- **The cap.** Asking for 10 or more builds 7 lazy workers with a 28-slot window (0.209–0.223 s),
  while asking for 8 or 9 built 8 or 9 lazy workers (0.192 and 0.190 s).

Two diagnostic questions, at one corpus on one machine:
1. At each k from 1 to 9, how do k lazy workers (`OpenCells`, `LazyCells` elsewhere) compare
   with k eager workers (`Cells`), the window 4k either way: wall, CPU and peak RSS, paired by
   round?
2. Among lazy workers at k = 7, 8, 9 and 10 (window 4k), how does each compare with 8?

And one record: at each t from 3 to 12 asked, what `BzReader::parallel` builds on the i9 (its
workers, window, kernel, and whether it is the service core or the pipelined teams), and its
wall, CPU and peak RSS beside the explicit cells.

## The contract

- **The decoder.** Master at `16aaee3b11ed91d8a1ff6cb3d1a39426a44718a0`, unchanged. The recipe
  refuses to run unless `src`, `Cargo.toml`, `Cargo.lock`, `build.rs` and `examples/ab.rs` are
  byte for byte that commit's. rustc 1.99.1, release, `Kernels::detect()`, the allocator as
  shipped. No research feature.
- **The timed harness.** Every decode is `ab child ARM N FILE` (examples/ab.rs, master's), in a
  process of its own, which reports its wall, its CPU (user + system, less its sampler's) and
  its peak RSS (`ru_maxrss`), the output's length and a 64-bit hash of every byte. It reads the
  file through the page cache (the corpus is resident after the first decode) and hashes the
  output on the caller's thread. Its walls are therefore **not** stage 1's walls: stage 1 read
  the input into memory first and discarded the output. Comparisons are within this run only.
  - `mibz2-workers-lazy K` and `mibz2-workers-eager K`:
    `BzReader::parallel_workers(r, K, 4K, Kernels::detect(), lazy)`, exactly K workers, a
    window of 4K blocks.
  - `mibz2 T`: `BzReader::parallel(r, T)` as shipped.
- **The probe** (examples/kernel_count.rs, new, untimed): the same constructors on an empty
  input, printing what was built (`threads()`, `window()`, `is_lazy()`, `is_service()`,
  `is_pipelined()`), then dropped. Each explicit cell is probed once before the rounds and must
  build as asked (k workers, window 4k, the kernel asked, neither service nor pipelined): a cell
  that does not makes the run INVALID. The public API is probed once before the rounds and after
  every one of its observations, in the same placement (unpinned) as its decodes. What
  `StreamDecoder::new` builds depends on the threads asked and the machine, not the input.
- **Placement.** Unpinned on every online CPU, as stage 1 and as the API is used. No spinner.
- **The corpus.** 4 copies of `corpora/wikidata.bz2`, checked against bench/wikidata.json's
  SHA-256 (1,724 blocks, 838,800,784 bytes out).
- **Children's environment.** Fixed and minimal (`PATH`, `LANG`, `LC_ALL`), one argument layout
  (the count two digits wide).

## The cells (29 a round)

| family | cells | constructor |
|---|---|---|
| lazy | k = 1, 2, …, 10 | `parallel_workers(r, k, 4k, detect, true)` |
| eager | k = 1, 2, …, 9 | `parallel_workers(r, k, 4k, detect, false)` |
| api | t = 3, 4, …, 12 asked | `parallel(r, t)` |

The cap's cells are lazy 7, 8, 9 and 10, inside the lazy family (one cell each, not run twice).
There is no eager 10: no question here needs it.

## The method

- **Pre-warm:** 50 s of decodes of lazy 8 before round 0, discarded.
- **Rounds:** 12 randomised complete rounds. In each, the 29 cells once each, in an order drawn
  from seed 20261125 (Python's `random.Random(seed).shuffle`, a fresh permutation each round).
- **An observation:** 1 s of sleep (settle), then 3 decodes of its cell back to back, each a
  fresh process. Its wall, CPU and peak RSS are the means of the three.
- **Recorded with every decode:** its round, its position in the round, the cell before it, the
  exit status, the driver's own span, the load average, and the machine's other busy cores over
  it (from `/proc/stat`, less the child's CPU).
- **Recorded once:** `lscpu`, `lscpu -e`, the affinity, the online CPUs, the frequency policy
  (governor, driver, EPP, `no_turbo`, THP), the microcode, `ab env`, the toolchain and the
  binaries' and inputs' SHA-256.

## The report (bench/kernel_count.py)

No verdict is printed, and none is drawn from it in this registration.
- **A cell's figure:** the median over its 12 observations, for wall, CPU and peak RSS.
- **A paired ratio** of cell A to cell B: A's observation over B's in the same round, over the
  rounds both have. Its point is the geometric mean, with a 95% t-interval on the logs (n − 1
  degrees of freedom); beside it the median of the rounds' ratios and the rounds A was lower.
- **Table 1, lazy against eager at k = 1..9:** each arm's median wall, CPU and peak RSS; lazy
  over eager for wall, CPU and peak RSS.
- **Table 2, the cap:** lazy 7, 9 and 10 over lazy 8 (wall, CPU, peak RSS), and lazy 10 over
  lazy 7 (wall).
- **Table 3, the public API at t = 3..12:** what it built (each configuration the probes saw,
  with its count), its median wall, CPU and peak RSS; its wall over the explicit cell of the
  same build (where it built workers, neither service nor pipelined, with a window of 4 a
  worker: lazy-kN or eager-kN), and over lazy 8.
- **Table 4:** every cell's rounds, median, minimum and maximum wall, peak RSS and CPU.
- **TSVs:** `tsv/ratios.tsv` (every ratio) and `tsv/observations.tsv` (every observation).

**Validity:**
- **INVALID:** any decode failed (a nonzero exit, no record, or an error), the decodes'
  outputs differ in length or hash, or an explicit cell did not build as asked.
- **INCOMPLETE:** any cell short of 12 rounds of 3 decodes, or the API built more than one
  configuration at one t (a surprise on one machine, worth knowing before any reading).
- **Otherwise COMPLETE.** The count of observations during which the machine's other work
  exceeded one core is printed; such observations are kept, not excluded.

## What this run decides

Nothing (RULES C1a). In particular, it does not:
- change `LAZY_FROM_THREADS`, `Plan::lazy` or any trimmed cell (`trimmed`, `trims_to_seven`);
- admit any cell to production;
- recalibrate any measured-i9 cell.

Its tables are inputs to the programme's run 4 and to a later registration.

## What a later adoption registration would need

A change to the production default (lazy below 8 off the measured cells, lazy at 3, 5 and 7
asked on the i9, or the 7-worker cap from 10 asked) is an adoption registration under RULES C6,
frozen before its data with the Researcher's review. From this run's design, it would at least
need:
1. **A primary per decision, fixed in advance:** for the kernel below 8, lazy over eager wall at
   each k in a named set, with a margin; for the cap, the shipped 7/28 against the candidate
   count's 4k window at each asked t in a named set. A Holm family over the cells it judges.
2. **More than one corpus.** This run is Wikidata alone. The lazy kernel's admission depends on
   the blocks' run structure; text and random (`ab gen`) as guards at least, as the
   recalibrations ran, and ideally a second real corpus.
3. **Both placements.** This run is unpinned only. The trimmed cells are keyed by placement
   (pinned one CPU a core, or the whole machine); a change touching them needs the pinned cells
   too, with the spinner as the recalibrations held it.
4. **The machine question.** Off the measured i9 every count below 8 is eager; a default change
   for other machines needs either evidence there (CI's runner, another workstation) or a
   decision to rest on this machine's, said as such.
5. **Memory as a figure with a rule,** not only wall: peak RSS is part of the trade the eager
   kernel makes, and a laptop's budget weighs it.
6. **The recalibration's harness and power:** paired rounds enough for the margin (this run's
   interval widths are its input), the shipped build as the reference, `bench/two_arm.py`'s
   reading, and the release-scaling check after.

## Expected cost

About 17 minutes on the i9: about 13 minutes of rounds (29 observations a round of 3 decodes,
about 11 s of decoding a round at 3 decodes plus 29 s of settle and the processes' start), 50 s
of pre-warm, the build and the corpus. The DRY (`just bench-kernel-count 1`) checks the wiring:
two rounds, one decode an observation, no settle, 2 s of pre-warm; it is not evidence.

## The recipe

`just submit bench-kernel-count`, on branch `bench/kernel-count`. One directory per attempt
(`target/kernel-count/run-<UTC stamp>`, made with `mkdir`), its console log inside it, every
phase's status in `phases.tsv` (preflight, build, corpus, opt-machine, run, report), and one
bundle, `target/kernel-count/run-<stamp>.tar.gz`, made by the EXIT trap whatever happens.
