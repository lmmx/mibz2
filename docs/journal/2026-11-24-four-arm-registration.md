# bench-four-arm: registration

*24 November 2026. Written and frozen before any data, on `bench/four-arm`. The brief is
`docs/journal/2026-11-24-algorithm-level/FOUR-ARM-BRIEF.md` at 4934a495, commissioned by Louis
(#145 QUEUED 7142566835). The recipe is `just bench-four-arm` (`.just/bench.just`), run as
`just submit bench-four-arm`. The verdict is `bench/four_arm.py`'s, tested by
`bench/test_four_arm.py`. It is built on bench-supplied-codes (`ready/bench-supplied-codes`,
5e5495c5; its registration is `docs/journal/2026-11-24-supplied-codes-registration.md`), and
keeps its workstation conventions: one new timestamped directory per attempt with its console
log, the machine's records, `bench/bundle.sh`.*

**Diagnostic only (RULES C1a).** This run decides no production default and no merge. The rule
below is posted on #145 before the run.

## The question

Does removing **two** costs together free whole-decoder time that removing either alone does
not? The two costs are the scanner's work (its Huffman and MTF/RLE2 symbol decoding, run on one
thread ahead of the workers) and Open's code-ordering in emission.

Only one cell of the square has been measured: bench-supplied-codes (journal §30). It halved the
emitter (0.53 at 1 worker, 0.55 at 10), but the whole decoder moved only to 0.838 at 1 worker and
0.988 [0.957, 1.020] at 10.

## The arms

There is one binary, `examples/four_arm.rs`, built with the `scan-oracle` research feature. That
feature (`src/scan_oracle.rs`, the hooks in the scanner of `src/parallel.rs`) includes
bench-supplied-codes' `supplied-codes` feature. Both switches are process-global, set between
decodes. Without the features nothing of this is compiled, and production defaults are untouched.

| Arm | Scanner (`scan_oracle::Mode`) | Open's emission (`supplied_codes::Mode`) |
|---|---|---|
| A | `scan`: as shipped | `open`: as shipped |
| B | `scan` | `supplied`: bench-supplied-codes' arm B, unchanged |
| C | `replay`: the recorded scanner output through the job queue | `open` |
| D | `replay` | `supplied` |

**The replay (arms C and D).** The parallel decoder's scanner thread reads the mode once, at its
start. In `replay` it reads no bit and decodes no Huffman code. Each slot the scanner would fill
is filled in the same order, under the same slot limit (the window), with that block's recorded
`block::Symbols`: its stored CRC, `orig_ptr`, the bytes in use, `eob`, the level's bound, and its
MTF/RLE2 symbols.
- *The delivery stays timed:* the symbols are copied (one `memcpy` of u16s) into the reused
  symbols buffer from the pool, as the scanner fills it. The block is sent through the production
  bounded job queue exactly as the scanner sends a block it has read.
- *The input is still fed and drained:* `BzReader`'s reader thread copies the compressed bytes
  into chunks as in arm A. The replaying scanner takes them as they come and recycles them as the
  scanner's window does, but reads none of them.
- *What the replay removes:* the bit reading, the selectors and Huffman tables, the symbol
  decoding, the stream headers and the stream's combined-CRC check.
- *What stays and is timed:* everything downstream. That is the workers' MTF and RLE2 inversion
  and runs, the BWT inversion (Open's construction and, per arm, its emission), RLE1, each block's
  CRC check, and the ordered output.
- *At 1 worker* the scanner already runs on its own thread, ahead of the one worker. So C at 1
  worker measures the scanner's contention and overlap, not its serial cost.

**The supplied codes (arms B and D)** are bench-supplied-codes' arm B, unchanged: the block's
marked-cycle codes in output order, copied into `edges`, consumed from the front and back.

**The oracle pass (outside every clock).** Before the rounds, one decode at 10 workers runs with
both sides recording:
- each block's symbols, by its index in the input (counted across streams). The corpus's four
  copies share one entry for each distinct block, found by a hash and confirmed equal;
- each open block's output-order codes, by its key (bench-supplied-codes' store).

Its output's size and CRC-32 are the reference. The harness stops, and the run is VOID, on any of:
- a codes key recorded twice with different codes;
- a block index recorded twice with different symbols;
- a block missing.

## The run

- **The build:** one build of this branch, master at the pin (f438f38d, bench-supplied-codes' own)
  plus the two research features and the harness. The recipe checks that `src` differs from the
  pin only in those features' files, now including `src/parallel.rs` and `src/scan_oracle.rs`.
  rustc 1.99.1.
- **The corpus:** `corpora/wikidata.bz2`, checked by its SHA-256, 4 copies end to end, read into
  memory once.
- **The decodes:** production's lazy workers (`BzReader::parallel_workers`, lazy: `OpenCells`,
  4 blocks a worker), unpinned.
- **The rounds:** 20. Each decodes the corpus once in each cell of {1, 6, 10} workers × {A, B, C, D}:
  - the worker counts are rotated every four rounds;
  - within each count, the arms run in row `r mod 4` of a Williams square (`ORDER`). Each arm
    takes each position, and each ordered pair of arms is adjacent, five times over 20 rounds at
    every count;
  - 2 s of settle precede each decode.
- **The check at the end of each round** (outside every clock): one decode at 10 workers with the
  scanner replayed and the emission in supplied-codes' `check` mode, in which every open block's
  supplied bytes are compared with the walk's. Its output's size and CRC-32 are compared with the
  reference, and it must replay every block.
- **Each decode's peak resident memory:** the process's high-water mark is reset before it
  (`/proc/self/clear_refs`) and read after (`VmHWM`). This is the whole process's, both stores
  included (precomputed symbols cost memory: the brief). `/usr/bin/time -v` adds the run's own
  where present.
- **The estimate:** about 14 minutes on the i9. That is 261 decodes after 2 s of settle each
  (about 8.7 minutes), plus their decoding, at §30's medians: 1.1–1.3 s at 1 worker, about 0.3 s
  at 6 and 0.21 s at 10, about 2.5 minutes in all. Add the build and the corpus.
  bench-supplied-codes' 180 decodes took 8 minutes.

**Changed from bench-supplied-codes, with reasons:**
- **Workers {1, 6, 10}** (the brief).
- **The conditioned decodes (cond 1) are dropped.** They were a receipt of B's emitter in
  isolation, and this run's question is the whole decoder's wall. Keeping them would double the
  decodes to about 25 minutes.
- **The arm order** is a Williams square in place of the pairwise alternation, since there are
  four arms.
- **The records:** a `run` line a decode, its open blocks' delivery and consumption summed, in
  place of a line a block.

## The rule (registered before the data)

**Each count:** each round's paired ratios of whole-decoder wall at the same worker count, B/A,
C/A and D/A, with their geometric means and 95% intervals (Student's t on the logs,
`bench/two_arm.py`'s `step`). Then:
- **X,** the better single arm, is whichever of B and C has the lower geometric mean;
- **D/X:** each round's ratio, with its interval.

**The decision, at 10 workers, on D/A's interval [lo, hi]:**
- **INTERACTION** iff all three hold:
  - hi ≤ 0.90;
  - D/A's geometric mean is below X/A's by more than 0.03;
  - D/X's interval lies wholly below 1.

  Both capabilities together are worth an algorithm that changes both interfaces.
- **ADDITIVE** iff hi ≤ 0.90 otherwise. Pursue the larger single arm first.
- **NO POTENTIAL** iff lo > 0.95. This pair of capabilities is closed as a joint target.
- **MARGINAL** otherwise, reported as such.

The same figures at 1 and 6 workers are reported and decide nothing.

**The one refinement of the brief's thresholds, made here before the data.** The brief's
D ≤ 0.90 and D > 0.95 are read on D/A's interval bounds (hi ≤ 0.90, lo > 0.95), not its point
estimate. The reasons:
- bench-supplied-codes' rule did the same: ADVANCE iff the upper bound < 0.90, STOP iff the lower
  bound > 0.95;
- at 10 workers the paired interval is about ±3% (§30: 0.988 [0.957, 1.020]), so a point rule
  would classify noise;
- this keeps the two diagnostics' verdicts on one scale.

The interaction's margin stays as the brief puts it: on the point estimates (D more than 0.03
below the better single arm), with the difference's interval excluding no-difference (D/X wholly
below 1).

**Reported, deciding nothing:**
- **The super-additivity contrast:** each round's (D × A) / (B × C), with its interval. Below 1,
  D gains more than B's and C's gains multiplied. The rule's INTERACTION means a joint gain beyond
  the better single arm; this contrast says whether that gain is more than independent effects
  would give.
- **Each arm's peak resident memory:** the median of its decodes, as a ratio to A's.
- **The stores' sizes.**

**The harness check.** B/A at 10 workers must reproduce §30's 0.988 [0.957, 1.020], meaning its
interval overlaps that one. If not, the verdict is still reported, labelled UNVALIDATED.

**VOID** (no verdict), `bench/four_arm.py`'s:
- the file missing or malformed;
- the machine not an i9-10900K;
- the `meta` line missing or duplicated, or the stores empty, conflicting, colliding or missing a
  block;
- any (round, workers, arm) decode missing, duplicated or unexpected;
- any decode's output not 4 × 209,700,196 bytes, or its wall not a finite positive number;
- a scanning arm (A, B) with any block replayed, or a replaying arm (C, D) with other than all of
  the store's blocks;
- any round's check missing or duplicated, with a mismatch, with another output (size or CRC-32)
  than the reference's, or with other than all the blocks replayed.

## Correctness, before the run (container only, not evidence)

- **`tests/four_arm.rs`:** on the crate's test corpora at levels 1 and 9, and on three streams end
  to end:
  - the oracle pass records every block with no conflict, and the copies share their entries;
  - all four arms decode to the original bytes at 1 and 3 workers, the replaying arms delivering
    every block and the scanning arms none;
  - the check decode finds no mismatch;
  - a replay from an emptied store delivers nothing;
  - a replay cancelled part way does not hang.
- **`examples/four_arm.rs`'s test:** the order is balanced.
- **`bench/test_four_arm.py`:** 22 checks. They cover each verdict, the margin and the D/X interval
  each binding, a D whose upper bound passes 0.90, the UNVALIDATED label, the 1- and 6-worker
  counts deciding nothing, peaks unmeasured, and each VOID.
- **The other checks:** bench-supplied-codes' own tests still pass; clippy is clean with
  `-D warnings`, with and without `--all-features`.
- **A smoke run of the harness** here (one copy of the corpus, 1, 2 and 4 workers, 4 rounds, no
  settle; a 4-CPU Xeon, so its timings mean nothing):
  - every decode's output was 209,700,196 bytes, and every check's CRC-32 was the reference's
    with no mismatch;
  - the replaying arms delivered all 431 blocks (431 distinct; 50.1 MB of symbols); the codes
    store held 233 keys (106.3 MB);
  - per-decode peaks were 200–255 MB;
  - the scorer read the records, and voided them on the machine alone.
- **The recipe's DRY preflight** (`just bench-four-arm 1`, two rounds, no settle, at bc8b0d98) is
  archived as `2026-11-24-four-arm-registration/run-20261124T200338Z.tar.gz`. It ran the whole
  recipe here:
  - the build check against the pin passed, and the corpus was checked by its SHA-256;
  - the oracle pass on the four copies: 1,724 blocks (431 distinct, 50.1 MB of symbols), 233 code
    keys (106.3 MB), with no collision, conflict or missing block;
  - all 24 timed decodes and both checks: 838,800,784 bytes each, CRC-32 d7626c17, 1,724 blocks
    replayed, 932 supplied blocks checked, no mismatch;
  - peaks were about 360–375 MB;
  - the scorer found the records whole and voided them on the machine alone (a 4-CPU Xeon).

## Reproduce

- `python3 bench/test_four_arm.py`
- `cargo test --release --features scan-oracle --test four_arm`
- `cargo test --release --features scan-oracle --example four_arm`
- The run: `just submit bench-four-arm`. The DRY: `just bench-four-arm 1`.
