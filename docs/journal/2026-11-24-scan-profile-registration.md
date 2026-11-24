# bench-scan-profile: registration

*24 November 2026. Written and frozen before any data, on `bench/scan-profile`. The brief is
`docs/journal/2026-11-24-algorithm-level/SCANNER-BRIEF.md` (part 1) at edc32f1b, #145 QUEUED
7145028132, commissioned after the four-arm verdict (LOG 7144221012; journal §44). The recipe is
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
- **The rounds:** 12. Each round decodes the corpus twice at each of 1, 6 and 10 workers, the
  timers on and off:
  - the counts are rotated by `r mod 3`;
  - the timed decode runs first in even rounds;
  - 2 s of settle precede each decode.
- **The estimate:** about 6 minutes on the i9, the build included. That is 72 decodes, about
  2.4 minutes of settle and about 1 minute of decoding.

## The report

Each figure is the median over a count's 12 timed decodes.

- **The scanner:**
  - its work (each read less its input waits, plus its hand-off);
  - its slot waits and its input waits, each as a share of the decode's wall;
  - the work by component: the six phases, the read's remainder, the hand-off;
  - bits and symbols a second of work, and decode ns a symbol;
  - table switches and selectors a block, and the fit's ns a symbol, ns a switch and µs a block;
  - a block's read (less its input waits): p10, median, p90, max.
- **The workers:** mean idle workers (their idle time over the wall), and each worker's busy time,
  the decode time the workers alone would need.
- **The timers' overhead:** each round's paired wall ratio, timed over untimed (geometric mean,
  95% interval).

## The classification (registered before the data; it decides nothing)

At 10 workers, on the medians' shares of the decode's wall:
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

1 and 6 workers are classified the same way, and decide nothing.

**The figure for part 2:** the serial scanner's work a decode, against the wall and against each
worker's busy time. A cheaper or parallel scanner must bring the first below the last.

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
- **`bench/test_scan_profile.py`:** 14 checks. They cover each classification,
  DECODE-DOMINATED, the largest component named, PERTURBED, the fit recovering a known 2 ns a
  symbol and 50 ns a switch, and each VOID.
- **Lints:** clippy `-D warnings` with and without `--all-features`, `cargo fmt` and ruff are clean.
- **A smoke run here** (4 copies; 1, 2 and 4 workers; 2 rounds; no settle; a 4-CPU Xeon, so its
  timings mean nothing):
  - all 1,724 blocks were recorded in every timed decode;
  - the Huffman decode was 0.74–0.81 of the scanner's work, at about 2.4 ns a symbol;
  - 438 table switches in 1,163 selectors a block;
  - the report read the records and classified the scanner QUEUE-BOUND at 4 workers there.
- **The recipe's DRY preflight:** below.

## Reproduce

- `python3 bench/test_scan_profile.py`
- `cargo test --release --features scan-profile --test scan_profile --example scan_profile`
- The run: `just submit bench-scan-profile`. The DRY: `just bench-scan-profile 1`.
