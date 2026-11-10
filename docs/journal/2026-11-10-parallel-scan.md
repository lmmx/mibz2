# 2026-11-10: The scan without the scanner: found by magic, accepted by chain (#10 reopened)

**Status: an exact prototype, a bound, and a registered diagnostic, revised after the
Researcher's REVISE (#145 comment 6755860961; §8) and its re-review (#145 comment 6756608043;
§9), and re-frozen. No workstation data yet.**
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
- **The bound (model).** One scanner caps the decoder's throughput, relative to the sequential
  decoder, at 1 ÷ its share s of one thread's decode: 9–11× on Wikidata at level 9, 4.9× on the
  same text at level 1, 4.0–4.3× on incompressible data. The workers it keeps busy are
  (1 − s) ÷ s, since their work excludes the scan. A ten-core machine is past the last two
  today, in this ideal model.
- **The test (registered below).** `just bench-par-scan`: the harness's two complete
  constructions, one reading the symbols on a scanner thread and one on the workers, at 1 to 10
  workers, on three inputs. They also differ in buffer ownership, locking and speculative work
  (§5).

Nothing in the decoder changes. `examples/par_scan.rs` uses `block::read_item` and
`decode_block` as the library does. Its own `unsafe` is at two boundaries: the AVX2 filter, and
the `getrusage` call that reads its CPU time (§8.4).

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
`find` differ in who reads the symbols, and with it in who owns the symbol buffers, in their
locking, and in `find`'s candidates and speculative decodes (§5).

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

A scanner that is a share s of one thread's decode caps the decoder's throughput at 1/s of the
sequential decoder's, if everything else scales for free. The workers it keeps busy are
(1 − s)/s, not 1/s, since their work excludes the scan. Past that the decoder gains nothing
from cores; the finder design gains all of them, to the next serial stage (the reader's
in-order copy, unmeasured). Neither figure is the real multicore ceiling: it charges nothing
for the workers' supply, the joins and copies, or memory.

| input | s (measured, local) | cap today, × one thread | on 10 cores, the finder design over today's |
|---|--:|--:|--:|
| Wikidata, level 9 | 0.091–0.112 | 9–11 | 1.0–1.1 |
| text | 0.147 | 6.8 | 1.5 |
| Wikidata, level 1 | 0.206 | 4.9 | 2.1 |
| random bytes | 0.232 | 4.3 | 2.3 |
| binary records | 0.248 | 4.0 | 2.5 |

- **Ideal cores:** the last column is min(10, anything else that binds) ÷ the cap, with nothing
  else binding. The cap is in units of one thread's sequential throughput (1/s).
- **The prediction for level 9:** on the i9 at level 9 the memory knee is predicted to bind
  first (`2026-11-07-knee-mechanism.md`, and `2026-11-10-bandwidth-titration.md`, which left
  the memory cause unresolved), so Wikidata's own dumps would gain nothing from this until that
  moves. This is a prediction the diagnostic tests (§5), not an established consequence.
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
- **What it compares:** the two complete constructions, not Huffman work moved alone. Besides
  who reads the symbols, the arms differ in symbol-buffer ownership, candidate storage, locking
  and fallback work, and `find` pays for speculative decodes the chain drops. Its candidate
  metadata grows with the input: `slots` bounds the speculative jobs issued, not the candidates
  found (the finder's list is never trimmed). Each input's peak resident memory is recorded.
- **Placement, recorded, not controlled:** nothing is pinned. A cell at t workers is t + 2
  threads (the workers, the finder or scanner, and the caller's thread, which is the joiner),
  so 10 workers are 12 threads sharing the machine's CPUs. The run records the CPUs the process
  may use (its mask, `nproc`, the cgroup's quota, the topology), and the check samples every
  thread's CPU mask during a 10-worker decode of each arm.
- **The inputs:**
  - `wikidata`: the bench corpus, SHA-256 `82f65ace…c29915`;
  - `wikidata-l1`: its decoded bytes recompressed by Python's `bz2.compress(data, 1)`
    (libbzip2): here 19,018,084 bytes in 2,098 blocks, SHA-256 `2ae39adf…0d9e78`. The file's
    digest the workstation gets is recorded, not required. The file is read back and decoded
    by libbzip2, and its bytes must have the corpus's decoded SHA-256, or the run is REFUSED;
  - `records`: `ab gen DIR 8`'s `records.bz2`: here 19,175,457 bytes, SHA-256 `78bc91db…faa4c0`,
    recorded likewise.
- **Each input's decoded size, anchored apart from the run** (`bench/par_scan_inputs.py`): the
  manifest's `decoded_bytes` (bench/wikidata.json) for `wikidata` and `wikidata-l1`; the
  generator's stated size for `records`. libbzip2's decode of each must have it. The reporter
  is given it and refuses any cell of another size. The compressor's identity (Python's and
  libbzip2's versions) and every input's digest are recorded; the inputs themselves are not
  kept in the bundle.
- **Liveness, before the cells:** `par_scan - faults` injects a panic at each role of each arm
  (a worker after claiming a job, the finder before publishing a chunk, the scanner after
  issuing a block, the joiner after accepting one), at 1 and 4 workers. Each decode must end,
  with an error or the joiner's own panic, or the run is INVALID (§8.1, §9). Every step that
  runs a program of the run's (the corpus's preparation, the build, the fault tests, the
  inputs' generation and anchoring, the checks, the cells, the reports and the bundle) runs
  under a deadline in a process group of its own (`bench/bounded.py`). Past it the group is
  stopped and must be empty, and the run is INVALID. The small queries (git, lscpu, taskset,
  reading /proc) are not bounded.
- **Before the cells,** `par_scan FILE check` on each input: `mibz2::decode`'s bytes equal to
  libbzip2's, and every arm's bytes equal to `mibz2::decode`'s, or the run is INVALID. It also
  serves as the warm-up.
- **The cells:** five rounds; in each, the worker counts 1, 2, 4, 6, 8, 10; at each, the arms
  `scan` and `find`, their order alternating. No gating on the machine's load and no retries.
- **The figures** (`bench/par_scan_knee.py`, which refuses any file that is not exactly this
  grid): per worker count, each arm's median MB/s and CPU seconds, and find/scan in MB/s as the
  median of the rounds' paired ratios, with its range.
- **The reading, fixed here,** per input, from the 10-worker cells, r the median paired ratio:
  **FASTER** if r ≥ 1.15; **NOT FASTER** if r ≤ 1.05; **MIXED** otherwise.
  - FASTER is a descriptive threshold for selecting a construction to build next. It is not a
    statistical test of a population gain, and it qualifies nothing for production.
  - NOT FASTER covers small gains and small losses alike. Five rounds' extremes are not
    confidence intervals.
  - All rounds and all three inputs' outcomes are reported, with no retry and no selection.
- **Predictions, written before any i9 data:**
  - `wikidata`: NOT FASTER or MIXED (r 0.95–1.12): ten tables are past the knee in both arms;
  - `wikidata-l1`: FASTER, r between 1.3 and 2.2, with the ratio rising from 4 workers;
  - `records`: FASTER, r between 1.4 and 2.5;
  - at 1 worker, `find` is slower on every input (r 0.7–0.95).
- **What each reading directs:**
  - **FASTER on `wikidata-l1` or `records`:** the design goes to the Coordinator as a build
    candidate for the parallel decoder, with §6's list as its obligations. It would be the first
    change in the record aimed at inputs other than level-9 text.
  - **NOT FASTER on both:** this arm's claim closes: this construction, as built, gains
    nothing measurable over the scanner here. It does not show that something else holds the
    workers, nor close every speculative-scan construction: the arm's added parsing,
    scheduling, allocation and speculative work may hide a scanner saving. Its excess work is
    inspected before the idea is closed.
  - **`wikidata` FASTER** would challenge §4's prediction from the knee, and is reported as
    such. It would not show that memory pressure is absent.
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

## 8. Revised after review (REVISE, #145 comment 6755860961)

The Researcher's report
([par-scan-6755779354](https://github.com/lmmx/mibz2/blob/0fb3293015aa0c9676bcbd6ca8c14d37bf0d7dcc/state/reports/par-scan-6755779354/report.md))
supported the chain's acceptance and asked for bounded failure, recorded placement, an
independently anchored reporter, B5 evidence and narrower wording before queueing. What
changed, re-frozen in `bench/par-scan.frozen`:

### 8.1 Liveness (checked)

- **Failure reaches every waiter.** The finder, the scanner and each worker hold a drop guard
  that, if the thread unwinds, marks the shared state failed and stopped and wakes every
  waiter. `take`, the scan arm's joiner and every helper's wait check the mark. The joiner holds
  a guard that stops and wakes the helpers however it leaves, its own panic included.
- **Every helper is joined explicitly.** One that panicked makes the decode an error, even
  where the chain had finished. A decode returns or its panic propagates: it never waits on a
  thread that has gone.
- **The fault tests** (`par_scan - faults`, and the example's test `faults_end_never_hang`,
  run in CI): a panic injected at a worker after it claims a job, at the finder before it
  publishes its first chunk, at the scanner after it issues a block, and at the joiner after it
  accepts a block, in each arm at 1 and 4 workers, on a 30-block input made in memory. All 12
  cases end: the helpers' with `Internal("… panicked")`, the joiner's with its panic. Untimed:
  a 120 s deadline only bounds the wait.
- **Each guard is needed (checked):** with the helpers' guards removed, the worker case hangs;
  with only the finder's, the finder case; with only the scanner's, the scanner case; with only
  the joiner's, the joiner case (each seen as a hang at a 5 s deadline).
- **The recipe bounds its children** (300 s for the fault tests, 600–900 s for the inputs and
  checks, 1,800 s for each input's cells). A child past its deadline makes the run INVALID;
  what it wrote is bundled. Superseded in §9: the first construction, `setsid --wait timeout
  --kill-after`, did not stop a grandchild that ignores TERM.
- **Not covered:** an allocation failure (an abort, not an unwind) and a thread that stalls
  without panicking; the recipe's deadline is the bound on both.

### 8.2 Placement (recorded)

As §5 says: nothing pinned; a cell is t + 2 threads; the process's CPU mask, `nproc`, the
cgroup's quota and the topology recorded (`placement.txt`, and `knee: placement:` in each
harness log); every thread's mask sampled during a 10-worker decode of each arm (the check).
The comparison stays unpinned by design: the Researcher did not require the titration's core
isolation here.

### 8.3 The reporter (checked)

- `bench/par_scan_knee.py` takes each input's expected decoded size, recorded apart from the
  run (§5), and refuses a file whose cells decoded any other size. Its tests now include grids
  of uniformly wrong sizes (1, one byte short, one byte over, double), which used to read
  FASTER and are now refused.
- `bench/par_scan_inputs.py` anchors the sizes and checks the level-1 file's decoded digest
  against the corpus's; its tests refuse a manifest size the corpus doesn't decode to, a
  generator size the records don't decode to, and a level-1 file that decodes to other bytes.
- A failed `getrusage` is a panic in the cell, so the run is INVALID: no CPU figure is made
  from a failed call. Its timevals are checked to be well formed.

### 8.4 B5: the example's `unsafe`

The scope is this research example only; it ships in no library code.

- **The AVX2 filter (`Finder::find_avx2`).** Called only when `is_x86_feature_detected!("avx2")`
  said so, or under a build whose target has AVX2 (where detection is then asserted). Its loads
  are three unaligned 32-byte loads from `data[k+1..k+35]`, guarded by `k + 40 <= data.len()`,
  and its candidates are verified by `verify`, safe code that reads 8 bytes with `get` and pads
  a short tail. The 16-byte table loads read `[u8; 16]` arrays. No writes, no pointers kept.
  - **Miri (checked):** `miri_finder_matches_naive`, with AVX2 enabled for the target
    (`RUSTFLAGS="-C target-feature=+avx2" cargo +nightly miri test --example par_scan --
    miri_`): the filter and the scalar finder against the bit-by-bit reference on lengths 0 to
    96 bytes, the boundary lengths among them, with magics planted at random bits, at the last
    bit one fits and one past it, and with each input cut into two ranges. Under Miri a subset
    of lengths and cuts, chosen for the loop's edges; natively every length and cut. It passed
    on nightly 2026-10-26 (Miri 0.1.0 16d1ffac17). Miri interprets the intrinsics: this
    checks bounds and initialisation of every load, not the CPU's instructions.
  - **Differential (checked):** the same test natively; and the finder against `mibz2::scan`
    on the corpus and its 20 random cuts (§2), unchanged.
- **The `getrusage` call (`cpu_micros`).** A foreign call Miri does not run. The bound
  argument: `who` is RUSAGE_SELF (0); the pointer is to a live, exclusive, zero-initialised
  `Rusage` of 144 bytes (asserted at compile time on 64-bit Linux), which is `struct rusage`'s
  size and layout on x86_64 and aarch64 Linux: two `struct timeval` of two `i64` each, then 14
  `long`s. The kernel writes within it and keeps nothing; the return is checked, and the times
  are read as plain integers, every bit pattern of which is valid. Tested natively
  (`getrusage_times_grow`).
- **Fuzz and mutation testing aimed at the `unsafe` blocks, and the adversarial review by an
  agent other than the implementer:** done in #320, merged into this branch with its commits;
  recorded in §8.6. The Guardian may still HOLD (B5). The harness's thread code is safe Rust
  and was not run under Miri (it needs libbzip2, a foreign library, to make its inputs).

### 8.6 B5: the independent review, fuzzing and mutants (#320)

An agent other than the implementer reviewed the example's `unsafe` adversarially, fuzzed the
finder and ran hand-written mutants against the tests
([#320](https://github.com/lmmx/mibz2/pull/320), head `e10d1814`). Its changes are tests only,
inside `#[cfg(test)]`, and are merged into this branch with their commits. It found no
reachable unsoundness, and two latent hazards, both now fixed (below).

**Verdicts, block by block:**

| block | invariant | verdict |
|---|---|---|
| the call `unsafe { self.find_avx2(..) }` in `find` | the CPU has AVX2, from `self.avx2` | sound as used; latent hazard R1: `Finder::new(true)` is safe, so the invariant was not encapsulated |
| `find_avx2`'s pointer arithmetic and three 32-byte loads | the loads read `data[k+1..k+35]`; `k + 40 <= len`, or `k + 32 <= to <= len − 5`, each suffices, given `from <= len` | sound for every caller; latent hazard R2: a `from` near `usize::MAX` wraps `k + 32` and `k + 40` (overflow checks off) |
| `find_avx2`'s two 16-byte table loads | `[u8; 16]`, unaligned load | sound |
| `find_avx2`'s filter logic | a superset filter; `verify` (safe) is exact | sound and correct (M7–M18, fuzzing) |
| `cpu_micros`'s `getrusage` call | RUSAGE_SELF; a live, exclusive, zeroed 144-byte `#[repr(C)]` buffer, at least `struct rusage`'s size on LP64 Linux and macOS; any `i64` valid; nothing retained; the return checked | sound |
| the shared state: `Mutex` + `Condvar`, `thread::scope`, the sampler's `AtomicBool` | no `unsafe impl Send/Sync`; the stop flag only stops, the data comes back through `join`; poisoned locks recovered, read by no `unsafe` code; guards declared so the state's lock drops first | sound (safe Rust) |

**Fuzzing** (`fuzz_finder_differential`, a seeded randomised differential test: cargo-fuzz was
unavailable offline). The AVX2 filter and the scalar finder are compared with the bit-by-bit
reference on:
- each whole input, a random sub-range, and a random partition into 1–8 ranges;
- natively, placements against a `PROT_NONE` guard page after the end and before the start.

The inputs:
- *lengths* biased to the AVX2 loop's edges (32i + 30..45), and spread over 0–5,000;
- *backgrounds*: random, zero, 0xFF, or the magics' own bytes;
- *plants*: at random bits, at the tail, across step edges, back to back, and one-bit near
  misses.

The other tests it added:
- `finder_every_bit_offset`: every bit of lengths 40–104, with cuts and guard pages;
- `finder_find_chunk_edges`: a magic at each offset 0–48 bits before each chunk edge;
- `getrusage_counts_every_thread`: a 1.1 s burn on another thread must show.

The bounded run, 2 × 300 s, found no failures:

| seed | inputs | bytes | `find` calls | reference candidates | plants | guarded placements |
|---|--:|--:|--:|--:|--:|--:|
| 0x9e3779b9 | 21,788,324 | 8.32 GB | 762,572,188 | 298,806,222 | 394,731,030 | 87,153,296 |
| 0x13c6ef372 | 21,822,399 | 8.32 GB | 763,784,766 | 299,483,367 | 395,531,502 | 87,289,596 |

In CI the fuzz test runs 3,000 inputs. The Miri subset of `miri_finder_matches_naive` now
includes the AVX2 loop's tightest lengths (32i + 34..=40).

**Mutants** (hand-written; cargo-mutants was unavailable offline). "Old" is this entry's tests
before #320, "new" with it:

| # | mutant | old | new |
|---|---|---|---|
| M1, M2 | AVX2 bound `k+40` → `k+34`, `k+35` | pass | pass: equivalent (the `to` bound suffices) |
| M3 | `k+32<=to` → `k+31<=to` | killed | killed |
| M4 | `k+32<=to` → `k+33<=to` | pass | pass: equivalent (the scalar tail covers it) |
| M5, M6 | step 31, 33 | killed | killed |
| M7, M8 | third load `p+2` → `p+1`; base `k+1` → `k` | killed | killed |
| M9, M10 | low nibble unmasked; high-nibble shift 3 | killed | killed |
| M11 | third fingerprint byte dropped | pass | pass: equivalent (a weaker filter; `verify` is exact) |
| M12–M14 | mask inverted; `leading_zeros`; two bits cleared per loop | killed | killed |
| M15 | scalar tail dropped | killed | killed |
| M16–M18 | tables swapped or built from the wrong byte | killed | killed |
| M19 | clamp `len-5` → `len-6` | **pass** | killed |
| M20 | clamp `len-5` → `len-4` | pass | pass: equivalent (`verify` rejects) |
| M21 | clamp `len-5` → `len-1` | killed (panic) | killed |
| M22 | fit check `bit+48` → `bit+47` | killed | killed |
| M23 | AVX2 dispatch unconditional | pass | equivalent on AVX2 hardware; killed by Miri without `+avx2` (R1) |
| M24, M25 | `getrusage` `who` = THREAD, CHILDREN | **pass** | killed |
| M26 | `Rusage` 8 bytes short | fails to build (the compile-time assert) | fails to build |
| M27 | timeval seconds misread (× 1e3) | **pass** | killed (the test now burns past 1 s) |
| M28 | `getrusage`'s return unchecked | pass | **survivor** |
| M29 | both load bounds loosened (the last load 1 byte past, output unchanged) | **pass**, Miri too | killed: SIGSEGV at the guard page; and by Miri on the new length subset |

- **M28, a documented survivor:** a valid `getrusage` call never fails, so its check can't be
  exercised without fault injection. The defensive check is kept: a failure panics, and the
  run is INVALID.
- **Two gaps in this entry's own tests, found by it:**
  - M29: the old Miri length subset (0, 5–7, 39–41, 45, 70, 96) never reached the lengths
    where a loosened bound reads past the end, so a 1-byte overread passed under Miri.
  - M27: the old `getrusage_times_grow` burned about 150 ms, too little to notice seconds
    misread as milliseconds.

  Both are closed by #320's tests.

**The two hazards, fixed in the measured file** (no caller reached either; both changes are
equivalent for every caller, which already gated AVX2 and passed `from <= len`):
- **R1:** `Finder::new` now sets AVX2 only if asked for *and* `is_x86_feature_detected!("avx2")`,
  so the safe constructor can't reach `find_avx2` on a CPU without it.
  - *Test:* `miri_r1_avx2_asked_for_without_avx2`.
  - *Before the fix,* under Miri without `+avx2`: "Undefined Behavior: calling a function that
    requires unavailable target features: avx2".
  - *After:* it passes, under Miri with and without `+avx2`, and natively.
- **R2:** after the clamp, `find` returns when `from >= to`, so a large `from` never reaches
  the bounds that could wrap.
  - *Test:* `miri_r2_from_past_the_end`: `from` at `usize::MAX − 1` and `usize::MAX`, at the
    end, past it, and above `to`, for both finders.
  - *Before the fix,* under Miri with `+avx2` and `-C overflow-checks=off`: "Undefined
    Behavior: overflowing pointer arithmetic". Natively with overflow checks on: an overflow
    panic.
  - *After:* it passes under Miri (both configurations) and natively.

Miri: nightly 2026-10-26 (Miri 0.1.0 16d1ffac17).
- *Without `+avx2`, and with `+avx2`:* `cargo +nightly miri test --example par_scan -- miri_`,
  3 tests, all pass.
- *With `+avx2 -C overflow-checks=off`:* the R1 and R2 tests pass.

### 8.5 The dry run, the freeze and CI

The re-frozen commit, the untimed dry run (`just bench-par-scan 1`) and CI at the ready head
are recorded on the PR that carries this revision.

## 9. Revised after the re-review (REVISE, #145 comment 6756608043)

The Researcher accepted placement, the input anchoring and the interpretation, and found two
residual liveness holes
([par-scan-readiness-6756472047](https://github.com/lmmx/mibz2/blob/25f4cd29f3dc152dd124cfba6551397664428df8/state/reports/par-scan-readiness-6756472047/report.md)).

### 9.1 The sampler's unwind (checked)

- **The hole:** `sampled`, the check's wrapper that samples the threads' CPU masks, stopped its
  sampler only after the decode returned. A joiner panic propagating out of the decode left the
  scope waiting on the sampler forever.
- **The repair:** a drop guard stops the sampler on every exit, an unwinding decode included.
- **The test:** `faults` now also injects the joiner's panic through `sampled`, for each arm at
  1 and 4 workers. All 16 cases end.
- **The negative control:** with the guard removed, the first sampled case hangs (at a 5 s
  deadline).

### 9.2 The process group's cleanup (checked)

- **The hole:** `setsid --wait timeout --kill-after` returns once its direct child has gone. A
  grandchild that ignores TERM, under a parent that exits on TERM, survives the grace.
  Reproduced here with GNU coreutils 9.4, at a 1 s deadline and 1 s grace: exit 124, and the
  grandchild still running (in state S), until killed by hand.
- **The repair:** `bench/bounded.py` supervises the group itself. It starts the step in a new
  session, so the group's id is the step's pid. At the deadline it sends TERM to the whole
  group, waits the grace, then sends KILL to the whole group whether or not the step itself has
  exited. It returns only once no live process is left in the group (a killed one not yet
  reaped counts as gone), and fails with 125 if one is. A step that exits normally but leaves
  processes in its group has them stopped the same way. A signal to the supervisor stops the
  group too.
- **Out of scope:** a descendant that starts a session of its own leaves the group. The
  diagnostic's steps don't do that; the decode's workers are threads.
- **The tests** (`bench/test_bounded.py`, in CI):
  - a step's own status passes through;
  - a grandchild that answers TERM is stopped (exit 124);
  - a grandchild that ignores TERM under a parent that exits on it is stopped (exit 124, gone
    within the grace);
  - a step that exits 0 and leaves a TERM-ignoring descendant has it stopped.
- **The dry run's self-check** runs the supervisor itself, at 2 s and a 2 s grace, on both
  grandchild cases, and refuses the run unless both are stopped.

### 9.3 Classification and coverage

- **Classification:** a step past its deadline is the registered INVALID whatever the step,
  the inputs' generation and anchoring included. A failure that isn't a timeout keeps its
  earlier classification: REFUSED for inputs that are not as recorded, INVALID for the
  harness, INCOMPLETE for the corpus's preparation or the build. None of them is VALID, and
  none licenses a retry.
- **Coverage:** every step that runs a program of the run's is bounded, as §5 now lists. The
  claim no longer says "every child".

### 9.4 Wording

- The summaries that said the arms differ "only in who reads the symbols" (the opening and §3)
  now name the other differences, as §5 does.
- The memory knee at level 9 is stated as a prediction (§4).
- **Placement:** `cpu.max` is the file at the mounted path, not necessarily the effective
  quota of a nested cgroup. The recipe now records `/proc/self/cgroup` beside it.
