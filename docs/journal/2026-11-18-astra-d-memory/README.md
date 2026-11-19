# Reproducing the Astra D ledger

The [journal](../2026-11-18-astra-d-memory.md) gives the results. The [proofs](proof.md) cover both reconstructions. Every experimental source and hook in this directory runs in a disposable checkout. The research branch changes files under `docs/journal/` only.

## Replay the published frozen references

On a little-endian host with Python 3.11+ and a C++17 compiler, the following command reproduces the frozen ledger directly. It checks SHA-256 for each ordered trace, instruction tally and address map, every model total at all three LLC sizes, and every phase/region row at 2 MiB. The archive contains data from the recorded binaries. A fresh native build can move a stack slot across a line boundary. In a separate stock build directory the frozen 2 MiB replay gave 181,262 L2 misses and 103,047 DRAM transfers, against 181,269 and 103,045 from the archive.

```sh
python3 docs/journal/2026-11-18-astra-d-memory/replay_frozen.py
```

## Rebuild the corpus and repeat the native captures

Use an x86-64 Linux machine with AVX2/BMI1 and PCLMUL, Rust 1.90.0, a C++17 compiler, Python 3.11+, Git, `nm`, Valgrind Lackey, lbzip2 and the crate's locked build dependencies. Supply a fresh scratch directory on a filesystem with room for the native traces, three builds and three decompressed corpus checks.

```sh
study=docs/journal/2026-11-18-astra-d-memory
bash "$study/reproduce.sh" "$(mktemp -d)"
```

The script uses the release profile (fat LTO, one codegen unit), links the tracer executable non-PIE to identify its phase-marker address, and keeps a separate Cargo target directory for each version. Its smaller whole-corpus and allocation drivers use `rustc -C opt-level=3` and a separate rlib per version. Set `VALGRIND_LIB` if Valgrind is locally unpacked. The recorded trace used Debian 13, glibc 2.41-12+deb13u4, Valgrind 3.24.0, GCC 14.2 and Python 3.12.14. Native text addresses, library routines, stack layout and allocator placement can vary across builds. The instruction and line counts describe the recorded binaries. The per-row permutation, edit counts, output digests and cache accounting identities are independent checks.

The corpus is assembled from the 12 archived Wikimedia byte windows. The repository's extraction checks each window against libbzip2, lbzip2 and the decoder. `extract.py` keeps original compressed block bits, CRC and primary and adds a one-block envelope. Whole-stream ordinals index all 431 blocks. Large ordinals index the 233 with at least 65,536 rows. The systematic sample uses every 24th large block from the historical data pin. Whole ordinal 270 is a separate high-density fallback control. All rates use the number of **final decoded bytes**. The encoded row count differs whenever RLE1 changes length. [Input lengths, original bit counts and SHA-256 values](inputs.csv)

## Trace boundary

One measured lap runs the shipped AVX2/PCLMUL scanner on a slice and the lazy worker's real `decode_block` body through materialization and CRC. The preceding lap supplies reusable vector capacities. The cache simulator starts empty at the second lap. This gives a cold-cache, reused-storage reference experiment. It charges explicit initialization that executes, including growing a cleared output vector's initialized length. Page first-touch, allocator zero pages on the first lap, unrelated workers, OS I/O, instruction fetches and concurrent queue locks are outside this boundary. The scanner and worker execute in production call order. At scanner/worker handoff and return to the scanner for the trailer, private lines are flushed and invalidated, and the LLC remains live. This gives each side a cold private cache without charging DRAM for every symbol handed over.

`StreamDecoder::read` can subsequently copy the verified slot to the caller. For the release harness's 4 MiB buffer, a 900,000-byte block incurs exactly 900,000 logical bytes read and 900,000 stored by that copy. The destination is reused between calls, and the slot may still be resident or may have waited in the queue. Those logical bytes have no fixed conversion to additional DRAM transfers. The counters on the desktop include the reader, scanner, other workers, channel and the rest of the process. [Production worker and public read boundary](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/src/parallel.rs) · [Release harness reader](https://github.com/lmmx/mibz2/blob/1edb29fdc3d42dfddfc3d60e46456b611323760c/examples/ab.rs)

Lackey emits dynamic guest data loads, stores, modifications and instruction events. `capture.cc` keeps the ordered data references between the marker writes and counts instruction events by phase. A modification is replayed as a read then a write. Each range is split across all touched 64-byte lines. Instructions and software prefetch hints create no demand data references in this cache model. REP iterations may generate multiple guest instruction events. CPU prices use these events as a work surrogate. There are no cycle measurements in the trace. The phase markers themselves are excluded from data references. Two diagnostic pointer stores and the remaining incidental data accesses are included.

The native map preserves the offsets, allocator alignments and aliases of actual buffers. The low-level `info rows` field is the retained `scratch.ll` length. The authoritative encoded length for alternative paths is the matching stock block's row count. When a warm-lap repair fell back, `scratch.ll` can retain a copy. `fix_ranges.py` detects a direct measured lap from its phase-12 marker and maps entries, bytes and bottom slots to their actual final-slot base. The event addresses themselves never change. Immutable MTF tables are located by symbol. The two register MTF heads generate no data references while held in registers. MTF rows, histograms and bitmaps have explicit entries. Remaining locals, fixed tables, spills, allocation metadata, instructions' stack traffic and mixed-line boundary effects are accounted in the residual by phase.

## Cache and output files

The model is strictly inclusive, with 64-byte lines and set-associative true LRU. L1D is 32 KiB eight-way. L2 is 256 KiB four-way. Effective LLC is 1, 2 or 4 MiB, 16-way in each replay, with set index given by virtual line number modulo set count. Inclusion and LRU are explicit approximations to hardware. Capacity sensitivity varies the number of sets. It does not emulate the i9's physical indexing, slice hash, replacement, hardware prefetch, coherence, fill buffers or the timing of competitors.

A demand store miss write-allocates. Lower-level replacement first collects any dirty private copy. A full-line writeback updates the resident inclusive parent without an RFO. Every writeback updates parent recency. After the trailer, all dirty lines drain to memory. A request is attributed to its accessing phase and the first byte touched in that line. A writeback is attributed to the most recent store to that line, possibly in a different phase or lifetime than its allocation. These conventions explain a structure having more dirty evictions than its own write allocations. [Cache simulator and conflict/back-invalidation tests](replay.cc)

| CSV column | Count |
|---|---|
| `loads`, `stores` | CPU-side demand **line** references, including hits and boundary crossings |
| `l1_rmiss`, `l1_wmiss` | L1 misses by reference type |
| `l2_req`, `l2_miss` | Demand lookups arriving at L2 and those forwarded to the LLC |
| `llc_req`, `llc_miss` | LLC demand lookups and last-level misses |
| `l1_wb`, `l2_wb` | Additional full-line dirty transfers L1→L2 and L2→LLC |
| `dram_read`, `dram_rfo` | Modelled line reads from DRAM on a demand load or write allocation |
| `dram_wb`, `terminal_wb` | All modelled dirty writes to DRAM, and the subset on the terminal drain |

Thus L1/L2 boundary bytes equal `64*(l2_req+l1_wb)`, L2/LLC boundary bytes equal `64*(llc_req+l2_wb)`, and DRAM bytes equal `64*(dram_read+dram_rfo+dram_wb)`. Hits never generate a fill. `summarise.py` checks the identities on each replay. There is no conversion from these counts to measured IMC bytes.

| Phase | Code |
|---|---|
| 1, 11 | Stream header, Huffman symbols and scanner trailer |
| 2 | Worker routing and MTF/RLE2, including capped run records |
| 3, 4 | Cells gates/layout and lazy induction |
| 5, 6, 7 | Writer setup/marked descent, reverse rule scan, forward commands |
| 8 | Segmented fallback, including T construction, lane walks and assembly |
| 9, 10 | Ordinary RLE1 and final CRC (the in-place arm shares the CRC) |
| 12–15 | Edit discovery, left moves, right moves and marker fills |
| 16, 17 | Affine tile construction and forward walk with final output and incremental CRC |
| 31 | Measured-lap entry |

The `*-regions.csv` files give the 2 MiB counts for each phase and structure on each of the eleven blocks. The `*-blocks.csv` files contain all three LLC capacities. `frozen-stock.csv` regroups the same frozen lines by structure. `summary.csv` weights ratios by final output bytes. The deliberately selected control is excluded from the systematic total. `compulsory.csv` gives each input and output address modulo 64, its line footprint and its ordinary-store comparator. `instructions.csv` and `trace-digests.csv` identify the native input to each replay. `dependencies.csv` counts algorithmic dependencies from the independently recorded induction tape. `segments.csv` reconstructs the shipped T-vector segment lengths from actual capped records and the production xorshift starts. [Evidence directory](evidence)

The pool census assigns block `i` to worker `i mod W` and slot `i mod 4W`, retaining symbols, worker scratch and output capacity. Its allocator counts requested live bytes above the already-loaded compressed input. Reallocation takes the net change in request size. Peak includes temporary guest allocations and excludes physical allocator overhead, kernel RSS, threads' stacks and a transient second copy internal to system realloc. This controls assignment and queue capacity so comparisons can be repeated. [Allocation census](pool.rs) · [Heap results](evidence/pool.csv)
