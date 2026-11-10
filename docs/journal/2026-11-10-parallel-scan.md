# 2026-11-10: The scan without the scanner: found by magic, accepted by chain (#10 reopened)

**Status: an exact prototype, a bound, and a registered diagnostic. No workstation data yet.**
- **The design.** One thread finds every bit where a block or end-of-stream magic occurs. Each
  worker reads and decodes the block at a candidate itself, Huffman included. The caller's thread
  walks the chain exactly as `stream::Scanner` does, and uses a worker's result only for the
  candidate at exactly the chain's next bit. So the finder is a hint: a false or missed magic
  costs time and cannot change a byte or an error.
- **Exact (checked).** `examples/par_scan.rs` gives `mibz2::decode`'s bytes on the corpus at 1 to
  4 workers, and the sequential decoder's bytes *and* `Error { kind, block, bit }` on six
  adversarial files and 4,000 mutated ones (1,500 more rerun for this entry).
- **The finder is free.** 9.3 GB/s of compressed input on one thread (local): 0.19% of one
  thread's decode, against the scanner's 11.2%.
- **What it gains is cores, not cycles.** CPU time is unchanged; the work moves to the workers.
  On the 4-vCPU VM it is neutral on Wikidata (0.91–0.97 of the library's wall at 4 workers) and
  13–15% faster on incompressible data. The gain is where a machine has more cores than one
  scanner can feed.
- **The bound (model).** One scanner caps the decoder at 1 ÷ its share of one thread's decode:
  9–11× on Wikidata at level 9, 4.9× on the same text at level 1, 4.0–4.3× on incompressible
  data. A ten-core machine is past the last two today.
- **The test (registered below).** `just bench-par-scan`: the two harness arms that differ only
  in who reads the symbols, at 1 to 10 workers, on three inputs.

Nothing in the decoder changes. `examples/par_scan.rs` uses `block::read_item` and
`decode_block` as the library does, with no `unsafe` of its own beyond its AVX2 filter.

Labels: **checked** (equality of bytes and errors), **measured** (local timing on the shared VM:
indicative only, RULES C5), **model** (arithmetic from stated assumptions), **the record's**.

## 1. What the record decided, and what has changed

- **The scanner was a twelfth of a decode.** `2026-10-27-bz2-decoder.md`: 1.6–1.9 of 24–30 c/B,
  so "one scanner feeds about 12 workers". Blocks are found exactly, "not by searching for the
  48-bit magic, so a magic inside a block's data can never be taken for a block".
- **A speculative scan was planned and never built.** `2026-10-27-mibz2-v2.md` read pbz2's
  marker scan and planned "workers that decode Huffman from speculative block markers, joined
  exactly".
- **#10 was deferred, then closed.** It "binds only from about 10 workers"
  (`2026-10-27-research-synthesis.md`), and "stays closed" (`2026-10-28-workstation-scaling.md`
  §9). The level-1 plateau near 1,460 MB/s was "consistent with the scanner bound"; it was not
  shown. Only a paired experiment reopens it (`2026-10-27-four-thread-ceiling.md`).
- **What has changed: the workers.** A decode is now 9–10 c/B here, not 24–30, and the scanner
  about 1 of it. The same scanner now feeds 9 workers, not 12, and fewer on every input that
  compresses less. The condition for reopening is the paired experiment, registered in §5.
- **The reason for not searching does not hold for this design.** A magic inside a block's data
  is found as a candidate and never accepted, because acceptance is the chain's (§2).

## 2. The design, and why it cannot change the output (checked)

- **The finder.** A magic at any of 8 bit shifts covers 5 whole bytes, so 16 byte patterns: a
  nibble-table filter over 32 positions a step, then the 48 bits compared. A scalar finder and a
  bit-by-bit one agree with it, as do 20 random cuts of the input into ranges (6 bytes carried
  across a cut).
- **The workers.** Each takes the next block-magic candidate and reads one item there with
  `block::read_item`, then `decode_block`, and reports where the item ended and its CRC.
- **The joiner** runs on the caller's thread and is the only authority. It parses the stream
  header, then each item from the last one's end, the combined CRC, the byte alignment and the
  next header, as the sequential scanner does.
  - It takes a worker's result only for a candidate at exactly the chain's bit, decoded at
    exactly the stream's level.
  - Otherwise it reads the item itself with the same two calls.
  - Candidates the chain steps over are dropped, with whatever was decoded there.
- **So:** every accepted block is one the sequential decoder would have read at that bit and
  level, and every error is raised by the chain at the chain's position. This is an argument and
  tests, not a proof.

**Against the truth** (`mibz2::scan`): 431 block magics and 12 end magics in 94.8 Mbit, none
missed, none false.

**False magics.** Uniform bits match one of the two magics at a given bit with probability
2⁻⁴⁷: 6.7 × 10⁻⁷ expected on the corpus, 6 × 10⁻³ in the 103 GB dump. Crafted input is not
uniform: a file that uses a chosen set of byte values carries the magic in every block's in-use
bitmap, in libbzip2's own valid output. That file is case 1.

| case | the sequential decoder | the harness |
|---|---|---|
| 1. 31 valid blocks, each with a stream header, a block magic and an end magic in its bitmap | 3,000,000 bytes | identical; 30 candidates stepped over, 31 results dropped |
| 2. a whole valid block spliced into a block's payload | 299,988 bytes, then `SelectorOverrun` (block 1) | identical; the spliced block decoded by a worker, never emitted |
| 3. stream, 4 stray bytes, stream | 200,000 bytes, then `TrailingData` | identical; 3 valid blocks beyond decoded and dropped |
| 4. a block marked randomised | 600,039 bytes, then `Randomised` (block 2) | identical |
| 5. cut inside a block; cut inside the stream CRC | `Truncated`, both | identical |
| 6. two blocks exchanged | 1,500,000 bytes, then `StreamCrc` | identical |

**Mutation fuzz:** 4,000 mutated multi-stream files (bit flips, truncation, magics written or
inserted, a valid block spliced in, deletions, repeats, stray bytes, level digits changed), at
1–4 workers and windows of 1–12: the bytes before the error and the error equal the sequential
decoder's in every case, over 14 error kinds. 6,091 blocks with valid CRCs were decoded at places
the chain never reached; none was emitted.

**Not checked:** a streaming input; `BlockTooLarge`, `Io` and `Cancelled`; a block over 64 MiB;
aarch64 (the scalar finder is its fallback).

## 3. Measured locally (indicative only)

**The finder:** 9.25 GB/s of compressed input on one thread with the AVX2 filter (164 GB/s of
decoded Wikidata), 1.08 with the scalar one. `mibz2::scan` reads 0.205.

**The harness's arms,** all exact: `lib` is `decode_parallel(&data, t)`; `scan` is one `Scanner`
thread and t workers in the harness; `find` is the finder thread and t workers. `scan` and
`find` differ only in who reads the symbols.

| input | scanner's share of one thread | find / lib, MB/s, 4 workers | find / scan, MB/s, 4 workers |
|---|--:|--:|--:|
| Wikidata, level 9 (17.7×) | 9–11% | 1.03–1.10 (three runs) | 1.04–1.05 |
| text (5×) | 14.7% | 1.13 | 0.99 |
| random bytes (1.1×) | 23.2% | 1.13 | 1.03 |
| binary records (1.0×) | 24.8% | 1.15 | 1.14 |
| 20,000 tiny streams | | 2.65 | 5.3 |

- **Four CPUs hide it.** `lib` at t threads is a scanner and t workers, so on this machine both
  designs fill the cores. CPU seconds are equal within noise on every input.
- **At one worker `find` is slower** (1.10–1.41 of `lib`'s wall): `lib` at one thread runs its
  scanner beside its worker.
- **Tiny streams** are the one large local effect: per-block headers and tables parallelise.

## 4. The bound (model)

A scanner that is a share s of one thread's decode feeds 1/s workers. Past that the decoder
gains nothing from cores; the finder design gains all of them, to the next serial stage (the
reader's in-order copy, unmeasured).

| input | s (measured, local) | cap today, × one thread | on 10 cores, the finder design over today's |
|---|--:|--:|--:|
| Wikidata, level 9 | 0.091–0.112 | 9–11 | 1.0–1.1 |
| text | 0.147 | 6.8 | 1.5 |
| Wikidata, level 1 | 0.206 | 4.9 | 2.1 |
| random bytes | 0.232 | 4.3 | 2.3 |
| binary records | 0.248 | 4.0 | 2.5 |

- **Ideal cores:** the last column is min(10, anything else that binds) ÷ the cap, with nothing
  else binding. On the i9 at level 9 the memory knee binds first (`2026-11-07-knee-mechanism.md`,
  and `2026-11-10-bandwidth-titration.md`), so Wikidata's own dumps gain nothing from this
  until that moves.
- **Level 1 is where the i9 should show it.** Its blocks are 100,000 rows, its tables 0.4 MB,
  and the record already suspects the scanner there. The README's level-1 slowdown is on the
  same inputs.
- **If the workers double in speed, every cap halves** in workers: Wikidata's falls to 4.5–5.5.

## 5. The registration (RULES C1's diagnostic class; the paired experiment #10 asked for)

**The question:** on the i9, at 1 to 10 workers, what does reading the symbols on the workers
gain over reading them on one scanner thread, on inputs whose scanner share differs?

**It can qualify no production change.** Both arms are a research harness, one table a worker,
in memory.

- **The recipe:** `just bench-par-scan`, frozen at the commit in `bench/par-scan.frozen`. For
  each input: `par_scan FILE knee 5 1,2,4,6,8,10`.
- **The inputs:**
  - `wikidata`: the bench corpus, SHA-256 `82f65ace…c29915`;
  - `wikidata-l1`: its decoded bytes recompressed by Python's `bz2.compress(data, 1)`
    (libbzip2): here 19,018,084 bytes in 2,098 blocks, SHA-256 `2ae39adf…0d9e78`. The digest the
    workstation gets is recorded, not required: the harness checks every decode against
    `mibz2::decode`'s, and the decoded bytes' SHA-256 must be the corpus's;
  - `records`: `ab gen DIR 8`'s `records.bz2`: here 19,175,457 bytes, SHA-256 `78bc91db…faa4c0`,
    recorded likewise.
- **Before the cells,** `par_scan FILE check` on each input: every arm's bytes equal to
  `mibz2::decode`'s, or the run is INVALID. It also serves as the warm-up.
- **The cells:** five rounds; in each, the worker counts 1, 2, 4, 6, 8, 10; at each, the arms
  `scan` and `find`, their order alternating. No gating on the machine's load and no retries.
- **The figures** (`bench/par_scan_knee.py`, which refuses any file that is not exactly this
  grid): per worker count, each arm's median MB/s and CPU seconds, and find/scan in MB/s as the
  median of the rounds' paired ratios, with its range.
- **The reading, fixed here,** per input, from the 10-worker cells, r the median paired ratio:
  **FASTER** if r ≥ 1.15; **NOT FASTER** if r ≤ 1.05; **MIXED** otherwise.
- **Predictions, written before any i9 data:**
  - `wikidata`: NOT FASTER or MIXED (r 0.95–1.12): ten tables are past the knee in both arms;
  - `wikidata-l1`: FASTER, r between 1.3 and 2.2, with the ratio rising from 4 workers;
  - `records`: FASTER, r between 1.4 and 2.5;
  - at 1 worker, `find` is slower on every input (r 0.7–0.95).
- **What each reading directs:**
  - **FASTER on `wikidata-l1` or `records`:** the design goes to the Coordinator as a build
    candidate for the parallel decoder, with §6's list as its obligations. It would be the first
    change in the record aimed at inputs other than level-9 text.
  - **NOT FASTER on both:** something else holds ten one-table workers on those inputs. #10
    closes again, on a paired result this time.
  - **`wikidata` FASTER** would contradict §4's reading of the knee, and is reported as such.
- **Limits:** one machine; in-memory input; the harness's own worker loop, not the library's
  teams; unpinned; frequency and thermal state uncontrolled beyond the alternation. All five
  rounds are reported, and there is no rerun to a reading.

## 6. What a production version would owe (not built)

- **A streaming input.** Today one thread owns the input window. Workers reading their own
  blocks need shared chunks, held from the oldest unaccepted candidate to the finder's head,
  bounded by `slots` blocks plus the 64 MiB item cap. A worker that runs out of bytes before the
  input ends must wait, not fail, and `Truncated` may be decided only at the end. This is where
  a change of contract could hide.
- **No guess at the level.** A stream's level enters a block's decode only as the bound on its
  rows. Workers can decode at level 9's bound and report the rows; the chain accepts if they fit
  the stream's level. The prototype guesses instead, and re-read 12–14 of 31 blocks in case 1.
- **A bound on wasted work.** Crafted nested valid blocks can each cost a worker a whole
  discarded decode: at worst about sequential wall time and threads × the CPU. Never wrong
  bytes. The window bounds how far ahead it goes.
- **The one-thread path stays as it is:** `find` is slower there.

## 7. Against other decoders

- **lbzip2** scans with a byte-wise automaton over all 8 alignments and confirms candidates by
  parsing block headers; magics that straddle its 1 MiB input blocks are recovered by its
  sequential parser.
- **indexed_bzip2 / rapidgzip** searches shifted byte strings; a candidate "may be a false
  positive inside this block, so the decoded encoded size is authoritative".
- **pbzip2** parallelises only files it wrote itself.
- **Here** the chain alone accepts, and the errors are the sequential decoder's, bit for bit,
  which none of those states for itself.

## Reproduce

```text
cargo build --release --example par_scan --example phases
target/release/examples/par_scan corpora/wikidata.bz2 finder
target/release/examples/par_scan corpora/wikidata.bz2 check
target/release/examples/par_scan corpora/wikidata.bz2 adversarial
target/release/examples/par_scan corpora/wikidata.bz2 fuzz 4000
target/release/examples/par_scan corpora/wikidata.bz2 time 11
target/release/examples/phases corpora/wikidata.bz2        # the scanner's share
```

The prototype and its measurements are an agent's of this session; the checks, the timings on
text, random and records, and the scanner's shares were rerun for this entry.
