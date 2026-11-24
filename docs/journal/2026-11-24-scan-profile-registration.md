# bench-scan-profile: registration

*24 November 2026. Written and frozen before any data, on `bench/scan-profile`. The brief is
`docs/journal/2026-11-24-algorithm-level/SCANNER-BRIEF.md` (part 1) at c74884ce, #145 QUEUED
7145028132, commissioned after the four-arm verdict (LOG 7144221012; journal §44).

**Amended before any data** to Louis's finer grid (the brief's amendment at c74884ce, which
supersedes its grid): every worker count from 1 to 12, then 14, 16 and 20. The amendment also
adds, at each count, the scanner's and the workers' busy and idle time and the measured
crossover, and a fitted pipeline model's shift point, checked against the measured grid. The
first draft's head was e1ba6833; its instrument is unchanged. The recipe is
`just bench-scan-profile`, run as `just submit bench-scan-profile`. The report is
`bench/scan_profile.py`'s, tested by `bench/test_scan_profile.py`. The workstation conventions are
bench-four-arm's.*

**Diagnostic only (RULES C1a).** It decides nothing. Its classification is posted on #145
before the run. Its purpose is to tell a cheaper or parallel scanner, part 2 of the brief among
them, exactly what it must beat.

## The question

At 10 workers the serial scanner is the wall: supplying its output saves 19% (C/A 0.806
[0.782, 0.831], §44). At 6 workers it saves about 2%, and at 1 worker nothing. What is the
scanner's time made of? How much is work and how much waiting, and how is the work divided
between the Huffman bit decode, the selectors and tables, the block headers and boundaries, and
the hand-off to the job queue? And how idle are the workers?

## The instrument

There is a research feature, `scan-profile`: `src/scan_profile.rs`, plus hooks in
`src/block.rs` and in the scanner and workers of `src/parallel.rs`. With the feature off, nothing
of it is compiled. With it on but the timers off at run time, each hook is one relaxed atomic
load. With the timers on, the only addition is a clock read at each boundary:

- **The scanner thread, per block:**
  - *the slot wait:* its wait for a free slot (the window full, the scanner waiting on the
    workers);
  - *the read:* `Scanner::next`, whole;
  - *the hand-off:* `send_job` into the bounded job queue.
- **Inside the read:**
  - *the input waits:* on the reader's channel;
  - *the phases of `block::read_item_with_scratch`:*
    - the header (magic, CRC, `orig_ptr`);
    - the bitmap of bytes in use and the group and selector counts;
    - the selectors (unary and move-to-front);
    - the code lengths and the Huffman tables' builds;
    - the symbols buffer's preparation (its reserve and zero fill);
    - the symbol decode: the Huffman decode of every 50-symbol group, the table switched per
      selector;
  - *the read's remainder:* the stream headers and ends, and the window's lookahead.
- **Counted per block:** its compressed bits, symbols, selectors, table switches (a selector
  naming another table than the one before it) and tables built.
- **Each worker:** its time blocked waiting for a job (the queue empty), its jobs, and its life.

**What "table switching" can and cannot show.** A switch costs a pointer change. Its real cost is
whatever the next group's decode pays for a colder table, and that cannot be timed separately
without perturbing a 50-symbol loop. So the report fits, across blocks, the decode phase's time
on the block's symbols and switches (least squares), and reads the switch coefficient as an
estimate, not a measurement.

## The run

- **The build:** master at d2705dad plus the feature and the harness (`examples/scan_profile.rs`).
  The recipe checks that `src` differs from the pin only in those files: `src/scan_profile.rs`,
  `src/block.rs`, `src/parallel.rs`, `src/lib.rs`. rustc 1.99.1.
- **The corpus and decoder:** 4 copies of `corpora/wikidata.bz2` (checked by its SHA-256), read
  into memory once; production's lazy workers (`BzReader::parallel_workers`, lazy: `OpenCells`, 4
  blocks a worker), unpinned, as in bench-four-arm.
- **The grid:** every worker count from 1 to 12, then 14, 16 and 20 (15 counts). All unpinned:
  production's lazy workers at exactly k, with the scanner on its own thread beside them, as
  bench-four-arm and bench-supplied-codes ran them.
- **The rounds:** 20. Each round decodes the corpus twice at each count, the timers on and off:
  - the counts are rotated by `r mod 15`;
  - the timed decode runs first in even rounds;
  - 2 s of settle precede each decode.
- **The records:** every timed decode's block totals (summed over its blocks), with the harness's
  check that its blocks are one each of 0..N−1, and every worker's record. Every block's own
  record is kept only for the first two rounds (30 decodes), which feed the per-block fit and
  quantiles. Kept for all 300, the file would pass the bundle's 50 MB limit.
- **The estimate:** about 28 minutes on the i9, the build included. That is 600 decodes: 20
  minutes of settle and about 3.5 minutes of decoding (§30's and §44's walls: about 1.3 s at 1
  worker, falling as 1/k to about 0.2 s from 7), plus the build and the corpus.
- **The expected precision:** the report gives the 95% interval's half-width of each count's mean
  scanner work and mean worker busy time. From the earlier 20-round runs on this decoder, I
  expect it about ±1% at low counts and ±2–3% at 8 or more workers. On the paired timed/untimed
  ratio, §30's 20 rounds gave ±0.6% at 1 worker and ±3% at 10.

## The report

Each figure is the median over a count's 20 timed decodes, at **every count of the grid**.

- **The scanner:**
  - its work (each read less its input waits, plus its hand-off);
  - its slot waits and its input waits, each as a share of the decode's wall;
  - the work by component: the six phases, the read's remainder, the hand-off;
  - bits and symbols a second of work, and decode ns a symbol;
  - table switches and selectors a block, and the fit's ns a symbol, ns a switch and µs a block;
  - a block's read (less its input waits): p10, median, p90, max.
- **The workers:** busy and idle time a worker, and mean idle workers (their idle time over the
  wall). A worker's busy time is the decode time the workers alone would need.
- **rho:** the scanner's rate (blocks a second of its work) over the workers' combined rate
  (k blocks a worker's busy second), that is a worker's busy time over the scanner's work. Above
  1, the workers bind; below 1, the scanner.
- **The measured crossover:** the count where rho first falls from at least 1 to below it,
  interpolated linearly in k between the two counts ("none in the grid" if it never does).
- **The pipeline model:** a serial stage feeding k workers through a bounded queue, in its
  simplest deterministic form, wall(k) = c + max(b, a/k):
  - b is the serial stage's work a decode, a the workers' work in all, c a fixed cost;
  - it is fitted by least squares to the counts' median untimed walls, by a search over the shift
    point k* = a/b in hundredths with (c, a) solved in closed form;
  - k*, a, b and c are reported, with every count's residual and their RMS;
  - beside it, the shift point the measured service times predict, w/s. Here s is the scanner's
    work a block and w a worker's busy time a block, both medians over the uncontended counts 1
    to 4.

  The model has no queueing delay and no contention, so its residuals show where the bounded
  queue and the shared cores matter. MONODROMY and Neon are separately reviewing whether this is
  the right model (channel #31).
- **The grid table:** at each count, the untimed wall, the scanner's busy and idle shares, a
  worker's busy and idle time, rho and the classification.
- **The timers' overhead:** each round's paired wall ratio, timed over untimed (geometric mean,
  95% interval).

## The classification (registered before the data; it decides nothing)

At every count, on the medians' shares of the decode's wall. It is **judged at 10 workers**, and
reported at every other count, deciding nothing there:
- **SCANNER-BOUND** iff the scanner's work is at least 0.85 of the wall and its slot waits at most
  0.05;
- **QUEUE-BOUND** iff its slot waits exceed 0.10 (the scanner waits on the workers);
- **INPUT-BOUND** iff its input waits exceed 0.10;
- **MIXED** otherwise.

Beside it:
- the work's largest component is named;
- **DECODE-DOMINATED** is added iff the Huffman decode is at least 0.60 of the work;
- the classification is labelled **PERTURBED** if the timers' overhead interval at 10 workers
  lies wholly above 1.03.

**The figures for part 2:**
- the serial scanner's work a decode, against the wall and against each worker's busy time, at
  every count;
- the measured crossover, and the model's shift point. Part 2's gate threshold is derived from
  these in its registration, before its data (the brief's amendment).

**VOID** (no classification), `bench/scan_profile.py`'s:
- the file missing or malformed;
- the machine not an i9-10900K;
- any (round, workers, timed) decode missing, duplicated or unexpected;
- any output not 4 × 209,700,196 bytes;
- a timed decode without its records, its blocks not one each of 0..N−1 (N the same throughout),
  or its worker records not one a worker.

## Correctness, before the run (container only, not evidence)

- **`tests/scan_profile.rs`:** on the test corpora at levels 1 and 9 and on three streams end to
  end, at 1 and 3 workers:
  - the decode's bytes with the timers on are the original's;
  - every block is recorded once, with its counts nonzero and its phases within its read;
  - every worker records its end, the jobs summing to the blocks;
  - nothing is recorded with the timers off.
- **The harness's order test** passes.
- **`bench/test_scan_profile.py`:** 17 checks. They cover:
  - each classification, DECODE-DOMINATED, the largest component named, PERTURBED;
  - the per-block fit recovering a known 2 ns a symbol and 50 ns a switch;
  - on a 15-count grid built from a known pipeline (a 0.05 s serial stage, 0.40 s of work in all),
    the measured crossover at k = 8.00 and the model's k* within 0.1 of 8;
  - no crossover in a grid where the scanner never binds;
  - each VOID, among them the harness's block check failing and a per-block record missing.
- **Lints:** clippy `-D warnings` with and without `--all-features`, `cargo fmt` and ruff are clean.
- **A smoke run here** (4 copies; 1, 2 and 4 workers; 2 rounds; no settle; a 4-CPU Xeon, so its
  timings mean nothing):
  - all 1,724 blocks were recorded in every timed decode;
  - the Huffman decode was 0.74–0.81 of the scanner's work, at about 2.4 ns a symbol;
  - 438 table switches in 1,163 selectors a block;
  - the report read the records and classified the scanner QUEUE-BOUND at 4 workers there.
- **A smoke run of the full grid here** (2 rounds, no settle; a 4-CPU Xeon, so its timings mean
  nothing for the i9): every count was recorded and classified. The measured crossover was
  k = 5.89 and the predicted shift point 6.11. The wall fit is poor there (RMS 0.052 s), because
  4 CPUs cannot run more than 4 workers at once.
- **The first draft's DRY preflight** (`just bench-scan-profile 1`, two rounds, no settle, at
  3d22242c, on the old grid) ran the whole recipe here, and is archived as
  `2026-11-24-scan-profile-registration/run-20261124T213843Z.tar.gz`:
  - the build check against the pin passed, and the corpus was checked by its SHA-256;
  - every decode gave 838,800,784 bytes, and every timed decode recorded all 1,724 blocks and
    one record a worker;
  - the report found the records whole and voided them on the machine alone (a 4-CPU Xeon).

  Read anyway, at "10 workers" on 4 CPUs (oversubscribed, meaning nothing for the i9), the
  report showed what it can see:
  - the scanner's work 0.82 of the wall, the Huffman decode 0.55 of that;
  - the hand-off into the job queue 0.28 of the work (its lock and wake-up, contended);
  - 4.9 of 10 workers idle on average.

## Reproduce

- `python3 bench/test_scan_profile.py`
- `cargo test --release --features scan-profile --test scan_profile --example scan_profile`
- The run: `just submit bench-scan-profile`. The DRY: `just bench-scan-profile 1`.
