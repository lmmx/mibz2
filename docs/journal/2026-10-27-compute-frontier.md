# 2026-10-27: The compute frontier: cycles per byte, stage by stage, against the hardware

Until now the campaign has won by removing work: fusion, overlap, fewer intermediates and
fewer bytes on the link. This entry asks a different question. How close is the
remaining computation to what the hardware can do, and is there an order of magnitude
left in the computational core? It covers SIMD, branchless code, memory-level
parallelism, cache-aware layouts, bandwidth and allocation.

It measures how many cycles each stage spends on a byte of Wikidata JSON, on the bench's
dump slice and on real dump lines. For the hot kernels it finds the bound, and it builds
prototype kernels that are byte-identical to today's output. Each opportunity is then
ranked by what it can move at a release.

The scripts, kernels and data are in `2026-10-27-compute-frontier/` (see Reproduce).
**Measured** figures come from this machine or from the bench's CI records. **Modelled**
figures scale them to a release, and each says what it assumes.

## Current State

- **Docs only.** Nothing in `src/`, `reshape/`, `bench/` or the dependencies changed.
  - No Hub call.
  - 48 MiB of byte-range reads from dumps.wikimedia.org: 12 windows of 4 MiB from
    `latest-all.json.bz2`, the 2026-10-21 dump. They were deleted after use.
- **Master is 59a692e.** The release model is bench 17.12.0's, from master 8f7720b's 6
  runs (AMD EPYC 7763, 4 cores): **6h51m at 120 MB/s, 6h54m at 60, 7h16m at 30, 11h02m
  at 7**. Within it, the split takes 3h20m, the pipeline 1h35m and finalise 2h13m.
- **End to end, the split spends 87.5 cycles of aggregate CPU on each byte of JSON**
  (the slice, this box, 2.1 GHz; elapsed: 26.6 wall-cycles a byte on 4 cores, 82%
  parallel efficiency):

  | | c/B |
  |---|--:|
  | the bz2 decode (lbzip2) | 42.7 |
  | the JSON reshape and Arrow build | 7.9 |
  | the claims' Parquet encode | 10.4 |
  | the rest of the work | 4.8 |
  | page faults (kernel time) | 13.2 |
  | thread oversubscription | 8.6 |

  Finalise's per-byte part is 71 c/B (53 at a release's ratio of group files to JSON):
  57% Parquet decode, 23% encode, 14% sort, 6% checks.
- **The release's JSON is 1.81 TB (95% CI 1.58–2.01 TB), not ~1 TB.**
  - The real dump decompresses 17.46× (12 windows, every whole block, CRC checked); the
    bench's slice 13.69×.
  - Real dump lines also cost 1.27× the slice's per byte in the split's own work: 28.2
    c/B against 22.2.
  - **So the bench probably understates the release's split by 25–60% (modelled).** It
    scales the split by bz2 bytes. A release's own split time would settle this.
- **The one large lever is the bz2 decode.** It is per-core bound by the inverse BWT's
  chain of dependent loads, not by bandwidth: a 3.6 MB T-vector per block, walked one
  load at a time.
  - **Measured:** lbzip2 scales 3.82× on 4 threads with 1% overhead. So block parallelism
    is already exploited, and what is left is per-core throughput.
  - **The prototype** (`kernels/bz2k`, Rust, byte-identical output, every CRC checked):
    - the walk split into 12 chains joined in order (memory-level parallelism within a
      block);
    - the CRC by PCLMULQDQ;
    - RLE1 and MTF in AVX2.

    It decodes at **24.3 c/B against lbzip2's 42.7 (1.76×)**: 27.9 on the real dump, and
    27.2–27.7 a thread with 4 threads at once. Its ceiling with these mechanisms is ~15
    c/B (2.8×).
  - **Modelled:** −46 min of the release at 30–120 MB/s (−69 at the ceiling; −59/−88 by
    the real-data model). Nothing at 7 MB/s, which is upload-bound.
- **SIMD, measured and honest:**
  - **Where SIMD clearly won**, the parts were already small. CRC 6.8 → 0.37 c/B
    (PCLMULQDQ, 18×), RLE1 3.6 → 0.56 (AVX2), MTF 4.1 → 2.2 (AVX2): together 10.5 →
    3.1 of the decode's ~50.
  - **The 2× came from restructuring the inverse BWT for memory-level parallelism.**
    That is scalar code, with no vector instructions.
  - **No AVX-512 is needed for any of it.** Every kernel is AVX2 or PCLMULQDQ, and each
    has a NEON/PMULL equivalent.
- **The claims' JSON → Arrow decoder is already schema-specialised.** It goes straight
  from the dump line into Arrow builders, with no tape or DOM (`claims_arrow.rs`).
  - It costs 7.9 c/B on the slice, 10–12 on real lines (10,000-line calls, as the split
    makes them).
  - It is bound by per-token dispatch, not scanning: 32 instructions a byte, a token every
    3.7 bytes. Replacing its `memchr2` string scans with the inline SIMD scanner cut 5.6%
    of instructions and **0% of time** (measured).
  - A SIMD parse into a general tape (simd-json) costs 2.85 c/B on the same lines, and a
    SIMD stage 1 0.6. So a rewrite is bounded at ~2.5–4× on a stage that is 9–13% of
    the split: ≤ −11 to −17 min. Not an order of magnitude.
- **Parquet encode and decode (arrow-rs, pq-glue's writer) have a flat profile.**
  - zstd is 8% of the split's encode (zstd 1), and CDC 19% of finalise's.
  - The rest is level computation, RLE, dictionaries and validity bitmaps, spread over
    dozens of functions.
  - Compiling for AVX2 or AVX-512 (`target-cpu`) moved the decode 10% and the encode 0%
    (measured).
- **Dead ends** (Dead ends):
  - assembly or AVX-512 for the bz2 walk;
  - interleaving blocks (≤ 1.27×: K blocks' T-vectors outgrow L2);
  - huge pages;
  - SIMD string scanning in the claims decoder;
  - `target-cpu=native`;
  - zstd levels in the split;
  - memory bandwidth (nothing is within 50× of it).
- **Two cheap, non-SIMD findings (measured on this VM, to be benched on CI):**
  - **Page faults:** 805k minor faults are 15% of the split's CPU here (sys 6.8 of 45.2
    s), from freshly mapped buffers on every chunk.
  - **Thread oversubscription:** lbzip2's 4 threads, the split's 2 workers and the
    writer's rayon pool of 4 run at once. `RAYON_NUM_THREADS=1` cut the split's CPU 13%,
    and `=2` its wall 9%.
- **The fastest plausible release on a 4-core AVX2 laptop** (Floors) is about **3h45m–4h
  at ≥ 30 MB/s** against 6h51m–7h16m today.
  - The split would be bound by Wikimedia's download (1h59m for 103.5 GB at 14.5 MB/s).
  - Finalise's Parquet decode/sort/encode would be about 1h30m, plus its fixed parts.
  - What stands in the way, in order:
    1. the bz2 decode's serial walk (~45% of the split's CPU);
    2. page faults and oversubscription in the split;
    3. the pipeline's per-file work (1h35m, structural: the trace's E2);
    4. finalise's arrow-rs decode and encode.
- **Verdict:**
  - **One real compute opportunity:** a bz2 decoder built for memory-level parallelism,
    worth ~2× on 45–55% of the split.
  - **Two cheap system ones:** faults and oversubscription.
  - **Everything else is ≤ 2–4% of the release each.** At 7 MB/s compute doesn't matter.

## Method

- **Cycles:**
  - This box: an Intel Xeon at 2.1 GHz with a constant TSC, 4 vCPUs. L2 is 2 MiB a core,
    and L3 260 MiB shared, but it acts like ~8–16 MB (the chase sweep below).
  - TSC cycles are wall time at 2.1 GHz. CPU seconds are converted at 2.1 GHz, as
    *aggregate* CPU over every thread. Wall is reported apart, never mixed with it.
- **Hardware counters are not available:** this is a VM, and `perf` is Ubuntu's shim
  with no kernel tools.
- **So every bound below rests on real timings and controlled experiments:**
  - working-set sweeps;
  - thread scaling 1 → 4;
  - interleaving independent chains (a latency test);
  - a bandwidth reference.
- **callgrind** counts instructions only. It is used to explain mechanisms, never as
  cycles.
- **The denominators:**

  | name | what | slice |
  |---|---|--:|
  | bz2 B | compressed input | 79,247,269 |
  | JSON B | decoded dump text (every stage's common denominator) | 1,084,929,419 (33,220 lines, 32.7 KB a line) |
  | Arrow B | the claims' built arrays (64-bit offsets, `nbytes`) | 1,270,658,400 (1.17 per JSON B) |
  | Parquet B | files out | split 97.5 MB (claims 71.2 MB) |

  Real dump lines: 864,519,007 B (68,661 lines from 12 windows, 12.6 KB a line).
- **Tooling decision (hotpath):**
  - Usable experimentally for instruction counts at kernel or batch boundaries in a Rust
    harness. It was not wired in here; raw callgrind served.
  - Nothing goes into production, the bench or CI.

## 1. Cycles per byte, stage by stage

### The split (measured on the slice, this box)

`split_stages.py` runs each chunk's `dump.write_chunk` one after another. It decodes the
slice first, and wraps every function of interest to take its own process CPU time (all
threads) and wall time.

`tools/split_once.sh` runs the real split (lbzip2 `-n4`, 2 worker threads, the writer's
rayon pool) for the system-level figures.

| stage | denominator: in → out | CPU-s (slice) | c / own byte | **c / JSON B** (slice) | c / JSON B (real lines) | bound and evidence |
|---|---|--:|--:|--:|--:|---|
| bz2 decode (lbzip2 `-n1`) | 79.2 MB bz2 → 1,085 MB JSON | 22.06 | 585 / bz2 B | **42.7** | ~42 (bz2k scalar: 48.9 real against 49.5 slice) | Per-core latency of the inverse BWT's dependent loads (Section 2). Not bandwidth: the read reference is 0.2 c/B. |
| line splitting (Python readline, rstrip) | JSON → 33,220 lines | 0.45 | | **0.9** | | Python per line, on 32.7 KB lines. Negligible. |
| JSON parse + Arrow build (`entity_rows_built`, Rust, 1 thread) | JSON → claims Arrow 1.27 GB + text columns | 4.07 | 6.7 / Arrow B | **7.9** | **12.2** (10–12 a 10k-line call) | Instructions: 32 a JSON byte, IPC ≈ 4.3 (callgrind count over native cycles), a token boundary every 3.7 B. A SIMD scan patch: −5.6% instructions, 0% time (Section 3). |
| claims flatten, refs, coordinates, ids (pyarrow, Python) | Arrow → statements | 0.85 | | **1.7** | 2.0 | Arrow compute, small. |
| claims Parquet encode (pq-glue, zstd 1, 4 threads) | 1.27 GB Arrow → 71.2 MB Parquet | 5.35 (1.69 wall) | 8.8 / Arrow B, 158 / Parquet B | **10.4** | **11.4** | arrow-rs instructions: levels, RLE, dictionaries. zstd is 8% (Section 4). |
| terms and links (Rust build + Parquet), entities, glue | text columns → 27.5 MB Parquet | 1.19 | | **2.3** | 2.4 | Small. |
| **sequential sum of the work** | | 33.97 | | **65.8** | ~71 (decode + 28.2) | |
| **the real split** (lbzip2 `-n4`, 2 workers, rayon) | | **45.2** (13.7 wall) | | **87.5** aggregate; 26.6 wall-c/B | | 82% parallel efficiency on 4 cores; peak RSS 2.8 GB |
| of which sys (kernel) | | 6.8 | | **13.2** | | 804,748 minor faults (~8.5 µs each here, zeroing included): freshly mapped buffers every chunk |
| of which user beyond the sum | | 4.4 | | **8.6** | | Oversubscription. `RAYON_NUM_THREADS=1`: CPU 42.3 s (−13% against one run, −6% against the other). `=2`: wall 12.5 s against 13.7–14.7 s. |

**CI's view, as the cross-check** (bench 17.12.0, master 8f7720b, 6 runs, EPYC 7763):

- split 36.73 CPU-s, 10.37 wall (89% efficiency);
- the bz2 decode 19.65 CPU-s, **53.5%** (this box: 51% of the real split);
- this box runs the same split 1.23× slower and lbzip2 1.17× slower.

The release model: the split's ETA is 12,000 s on 4 cores.

### The pipeline and finalise (measured on the bench's run, this box)

`results/local_bench_run.json` is one `bench/pipeline.py` run, quick tier, this commit.

- **The pipeline:**
  - 5.8 CPU-s on 3.74 s of wall, 11.2 c/JSON B, of which `merge_group` is 1.62 s.
  - At a release its 1h35m is modelled from per-file and per-worker costs, not from
    bytes. The trace's E2 removes it structurally.
  - It is not a kernel problem.
- **Finalise:**
  - 114.4 CPU-s on 47.3 s of wall; ×3 rows: 187.4 CPU-s. Peak RSS 3.6 GB (8.4 GB ×3).
  - The part that grows with bytes is **0.372 CPU-s per MB of group files**:
    - 781 c per group-file byte;
    - **70.6 c/JSON B** at the slice's 0.0905 group-file bytes per JSON byte;
    - **53 c/JSON B** at a release's 0.068 (122.9 GB of group files / 1.81 TB, modelled).
  - Its exclusive CPU by function (phases, ×3 run):

    | | CPU-s | share |
    |---|--:|--:|
    | Parquet decode (`_read_bucket` 28.2, `_check_written` 15.5, `_bucket_source` 10.4, `_read_key` 4.0, IPC 2.1) | 60.2 | 57% |
    | encode (`_write_file_parallel` 22.2, `write_table` 1.5) | 23.7 | 23% |
    | sort (`_sorted` 13.6, `sort_in_memory` 1.3) | 14.9 | 14% |
    | checks and hashes | ~6 | 6% |

### References on this box (roofline)

| reference | c/B | evidence |
|---|--:|---|
| sequential read, 1 core | 0.20 | 10.3 GB/s; 4 cores 33.5 GB/s (`bz2k bandwidth`) |
| copy a line | 0.42 | `jsonk` |
| UTF-8 validation (simdutf8) | 0.22 | `jsonk` |
| SIMD stage 1 (structurals, in-string mask by CLMUL) | 0.61 | `jsonk` |
| simd-json `to_tape` (general SIMD parse, no Arrow) | 2.85 | `jsonk` (3.27 less the copy) |
| zstd 1 / 3 compress (JSON) | 4.3 / 7.5 | the CLI, 300 MB |
| zstd decompress | 1.7 (of output) | |
| igzip decode (the gz slice) | 1.57 (of JSON) | 0.84 CPU-s |
| bzip2 (libbz2 CLI) / lbzip2 `-n1` | 50.7 / 42.7 | |
| a dependent load at a 3.6 MB working set | 52 per load | `bz2k chase` |

## 2. The bz2 decode: phases, scaling and the prototype

### Where the decode's time goes

`kernels/bz2k` is a standalone bzip2 decoder in Rust (no randomised blocks; errors
panic). Each phase is its own pass, timed by TSC:

1. Huffman;
2. RUNA/RUNB + MTF;
3. the T-vector build;
4. the inverse BWT's walk;
5. RLE1 + the CRC.

Its output is **byte-identical to `lbzip2 -dc`**: `cmp` and sha256 over the slice, and
every block CRC and stream CRC is checked.

| phase (c / JSON B) | slice, scalar (CRC by byte) | slice, scalar (CRC by 8) | real dump, scalar | **prototype** (slice) | prototype (real) |
|---|--:|--:|--:|--:|--:|
| Huffman | 1.9 | 1.9 | 1.6 | 1.9 | 1.6 |
| RUNA/RUNB + MTF | 4.1 | 4.1 | 3.3 | **2.2** (AVX2) | 3.7 |
| T-vector build (a scatter) | 6.1 | 5.9 | 6.2 | 5.8 | 7.7 |
| **inverse BWT walk** | **31.4** | **30.9** | **31.3** | **14.1** (12 chains) | 14.4 |
| RLE1 + CRC | 10.1 | 4.5 | 6.5 | **0.4** (AVX2 + PCLMULQDQ) | 0.4 |
| **total** | 53.7 | 47.4 | 48.9 | **24.3** | 27.9 |

lbzip2 `-n1` is 42.7 c/B on the slice. It fuses the walk, RLE1 and the byte-at-a-time
CRC in one loop, so its CRC hides under the walk's load latency. **The walk is ~60–65% of
the decode, the rest is ~16 c/B, and memory bandwidth is nowhere near.**

### Is the walk latency-bound? (controlled experiments)

1. **The working-set sweep** (`bz2k chase`: dependent loads through a random cycle):

   | working set | 1 chain | 2 | 4 | 8 |
   |---|--:|--:|--:|--:|
   | 256 KiB | 14.6 | 7.2 | 3.6 | 2.0 |
   | 2 MiB | 37.7 | 19.9 | 7.7 | 4.3 |
   | 3.6 MB | 52.1 | 24.5 | 12.9 | 6.8 |
   | 8 MiB | 80.6 | 35.3 | 17.5 | 9.5 |
   | 16 MiB | 260 | 74 | 25 | 12.6 |
   | 1 GiB | 516 | 239 | 116 | 64 |

   - A bzip2 block's T-vector is 3.6 MB: past L2, and served at ~50 cycles a load.
   - The jump from 8 to 16 MiB is not the TLB: with huge pages (`AnonHugePages` checked)
     the 16 MiB row is 262/81/22/11.
   - The machine has ample memory-level parallelism: 8 chains are ~7–20× one.
2. **Interleaving within a block** (`bz2k walkm`: K chains through one block just built):

   | chains | 1 | 2 | 4 | 8 | 16 |
   |---|--:|--:|--:|--:|--:|
   | c/B | 28.5 | 17.6 | 10.9 | 6.8 | **4.4** |

   **So the walk is latency-bound: the hypothesis holds, by direct test.**
3. **Interleaving across blocks** (`bz2k walk`: K whole blocks in lockstep), the
   brief's suggestion:
   - K = 2–8 gives **1.03–1.27×** (huge pages or not).
   - The cause: K blocks' T-vectors are K × 3.6 MB, so each load's latency rises to the
     16 MiB+ row as fast as K hides it.
   - **A dead end.** Parallelism has to come from within one block's 3.6 MB.
4. **Thread scaling:**

   | threads | lbzip2 wall (s) | lbzip2 CPU (s) |
   |---|--:|--:|
   | 1 | 22.9 | 22.7 |
   | 2 | 11.0 | 21.9 |
   | 3 | 7.6 | 22.6 |
   | 4 | 5.9 | 23.0 |

   - That is **3.82× on 4 threads, 1% overhead**.
   - `bz2k threads` (each thread decoding the whole slice at once) keeps its per-thread
     cost at 4 threads: scalar 48.7–49.5, prototype 27.1–27.7 c/B.
   - No shared resource (L3, DRAM bandwidth) binds at 4 cores.
   - **Block parallelism is fully exploited already. What is left is per-core throughput,
     and the walk's serial chain is most of it.**

### The prototype (`kernels/bz2k`, measured, byte-identical)

- **`walk_segments`:** M = 256 pseudo-random rows are taken as segment starts, the
  walk's own first row among them.
  - A random row is a random text position, so the segments come out about equal. Evenly
    spaced rows clustered in the text and stalled the lanes.
  - 12 lanes walk segments in lockstep, each taking the next segment when its own
    reaches another start (a 112 KB bitmap).
  - The segments are then joined in the order the starts follow one another.
  - A block that is a repeated string has several cycles. It falls back to the plain
    walk (none in the slice).
  - **Results:**
    - the walk 31 → 10.0–14 c/B (2.2–3.1×) on the same blocks, equal to the plain walk;
    - in the full decode, 14.1;
    - the unjoined ceiling at 16 chains is 4.4.
  - The overhead left is pushes, the bitmap test and lane changes.
- **CRC by PCLMULQDQ:** 4 × 128-bit lanes folded with x^576 and x^512 mod P, the last 64
  bytes through the table. **6.8 → 0.37 c/B**, equal CRCs on 200 blocks. SSE4.1 +
  PCLMULQDQ, so every x86-64 laptop since 2010; PMULL on ARMv8.
- **RLE1 in AVX2:** the test for 4 equal bytes at 32 positions a vector (3 shifted
  compares, a movemask), with the stretches between runs copied whole. **3.58 → 0.56
  c/B.**
- **MTF in AVX2:** for an index below 32 (nearly all), the list's first 32 bytes are
  blended with the list shifted by a byte (one unaligned load). **4.1 → 2.2 c/B.**
- **Whole decode: 24.3 c/B, 1.76× lbzip2, 333 c per bz2 byte against 585.** Real dump
  blocks: 27.9.
- **Not tried:**
  - multi-symbol Huffman tables (≤ 1 c/B of 1.9);
  - the T-vector build (5.8 c/B: a scatter with 256 write streams);
  - a tighter segment walk. The arena-of-chunks variant was not faster.
- **Ceiling with these mechanisms:** Huffman 1.9 + MTF 2.2 + build ~5 + walk ~5 + tail
  0.4 ≈ **15 c/B (2.8×)**.
- **Memory:** about +2 MB a decoding thread (the segment buffers, the bitmap), against
  lbzip2's 11–36 MB. Nothing for a 16 GB laptop.
- **AVX2-only and ARM laptops:**
  - **Nothing here uses AVX-512.** The SIMD parts are < 1 c/B already, so ZMM would gain
    nothing.
  - **The walk's gain is from memory-level parallelism in scalar code,** so it is
    portable. Its size depends on the core's L2 and miss parallelism: Apple and recent
    x86 cores have more, likely more gain. Not measured off this box.
  - `target-cpu=x86-64-v3` or `native` did not help (25.4 and 28.2 c/B against 24.4).
- **How it could ship (not decided here):**
  1. A Rust decoder in `reshape`, replacing the lbzip2 subprocess. It decodes 900 kB
     blocks in parallel, which the sweep reader's block-aligned segments already find.
  2. Or the same walk as a patch to lbzip2's `emit`. Then the fused byte-at-a-time CRC
     must move out of the walk and be vectorised, or it becomes the new bound.

  Correctness is bzip2's own: block and stream CRCs, plus today's split checks and
  parity.

## 3. JSON → Arrow: the claims decoder

**The brief asked for a schema-specialised decoder that decodes straight into the final
Arrow buffers. It exists, and it is today's split:**

- `reshape/src/claims_arrow.rs` (`raw_statement`, `raw_snak`, `raw_reference`) reads each
  dump line once, in the same scan as the text columns (`lib.rs`).
- It builds the claims' nested lists, structs, offsets and validity bitmaps directly,
  genson's strict decode exactly.
- There is no tape, no DOM and no intermediate JSON text.

So the measured full-work figures are today's:

- **parse + build: 7.9 c/JSON B** (slice), **10–12** on real lines, one thread;
- **its Parquet encode: 10.4 / 11.4 c/JSON B** (Section 4).

A second decoder was not written. These measurements bound what one could gain:

- **Instructions, not scanning:**
  - callgrind on 300 lines (14.6 MB): 480M instructions, 32.8 a byte. Natively that is
    7.4 c/B, so IPC ≈ 4.3.
  - The cost is spread over the plan interpreter: `raw_snak` 9.4%, `string_into` 6.5%,
    `record` 5.9%, `put` 5.7%, `transcode` 5.5%, `memchr2` 5.4% + 3.6% + 3.3%,
    `Line::key` 5.0%, `Col::settle` 4.9%, `Bits::push` 3.3%, …
  - The lines hold a structural or quote every 3.7 bytes (13.4M in 49.7 MB), so it is
    ~120 instructions a token.
- **SIMD scanning, tried at full work** (`patches/claims-find-end.patch`):
  - The two `memchr2` calls in `claims_arrow.rs` were replaced with the inline SIMD
    first-block scanner `lib.rs` already uses (`find_end`).
  - −5.6% instructions (480M → 453M), **0% CPU** (7.11–7.41 against 7.18–7.39 c/B, 3 ×
    best of 5).
  - The scans were not on the critical path.
- **Bounds for a rewrite** (parse only, not results):
  - simd-json's tape costs 2.85 c/B on the same lines (2.87 on real lines), with no
    schema and no Arrow output. A SIMD stage 1 costs 0.6 and UTF-8 validation 0.22.
  - A schema-directed walk over a stage-1 index still dispatches 13.4M tokens per 49.7
    MB. At ~5 cycles a token (a key match and a builder append) that is 1.35 c/B, plus
    ~0.5 c/B of output writes (1.17 Arrow bytes a JSON byte).
  - **A floor of ~3 c/B.** The ceiling is ~2.5× on the slice and ~4× on real lines.
- **Where the lever is:** the plan is interpreted per value (`put` matching on `Kind`). A
  decoder compiled for the claims schema (generated straight-line code per record) would
  cut dispatch. That is code generation, not SIMD. Its equality with genson's strict
  decode is the cost and the risk.
- **Amdahl:** 9% of the split (slice) or ~13% (real lines) at ≤ 2.5–4× is **≤ −11 to
  −17 min** at a release. Not an order of magnitude.

## 4. Parquet encode and decode (arrow-rs through pq-glue)

`kernels/pqk` reads a claims file the split wrote and writes it again with
`pq_glue::write::write_batches`, with the split's options:

- `spec_names`;
- 122,880-row row groups;
- zstd 1;
- `statement_id` as DELTA_BYTE_ARRAY.

Or with finalise's options: zstd 3 and content-defined chunking. It reads the output
back and compares the rows: equal in every run. The file is the main set's `chunk_0`:
619,789 statements, 618.5 MB of Arrow, 43.46 MB of Parquet.

| | CPU-s (1 thread) | bytes out |
|---|--:|--:|
| decode (ParquetRecordBatchReader) | 1.37–1.48 | |
| encode, zstd 1 (the split) | 2.28 | 43.42 MB (the split's file: 43.46 MB) |
| encode, uncompressed | 2.09 | 85.2 MB |
| encode, zstd 3 (finalise, no CDC) | 2.65 | 42.85 MB |
| encode, zstd 3 + CDC (finalise) | 3.28 | 43.39 MB |
| encode, 4 threads (the split) | 2.47 CPU, 0.79 wall | |

- **Compression is not the cost:** zstd 1 is 8% of the split's encode. zstd 3 adds 0.37
  s, and CDC 0.63 s (19% of finalise's encode, kept for Xet dedup by decision).
- **Instructions** (callgrind, the scholarly `chunk_1`):
  - **Encode:** levels (`compute_leaves`, `LevelInfoBuilder`) 21%, the RLE encoder ~13%,
    dictionary hashing (`Interner`) ~13%, byte-array and primitive encoders ~25%, close
    and statistics ~13%, zstd 9%.
  - **Decode:** definition and repetition levels and validity bitmaps are the largest
    part.
  - It is a flat profile across arrow-rs: no one kernel to vectorise.
- **What compiling for the CPU gives:**
  - `target-cpu=x86-64-v3` / `native`: decode −10% (1.47 → 1.29 CPU-s, mean of 2),
    encode 0%.
  - Rows equal, bytes equal.
- **Ceilings (bounded, not measured):**
  - **A writer given levels by the decoder,** in place of recomputing them per leaf
    (`compute_leaves` is O(leaves × rows), with 93 leaves sharing a few list paths):
    ≤ 1.3× on the encode.
  - **SIMD level decode and bitmaps:** ≤ 1.5× on the decode.
  - Both are arrow-rs internals: upstream work.

## 5. Opportunities, ranked by expected effect at a release

- **The base:** bench 17.12.0's model.
  - **At 30–120 MB/s,** the release is CPU-bound in two phases: the split with the
    pipelines (compute ends at 4h06m), then finalise.
  - **At 7 MB/s,** the uploads bind: 123 GB of group files take 4h53m, and the
    publishing commits are as long. Compute savings there hide under them: ≈ 0.
- **Each saving** is the stage's ETA × its share × (1 − 1/speed-up) (Amdahl):
  - split ETA 12,000 s, finalise ETA 7,976 s;
  - split shares are CI's (decode 53.5%) or this box's real split;
  - finalise shares are from phases.

| # | approach | mechanism | kernel speed-up | stage share | saving at 30–120 MB/s | at 7 MB/s | effort | correctness and portability risk | memory |
|--:|---|---|---|---|--:|--:|---|---|---|
| 1 | **bz2 decoder built for memory-level parallelism** | the walk in 12 joined chains; PCLMULQDQ CRC; AVX2 RLE1 and MTF | **1.76× measured** (42.7 → 24.3 c/B, byte-identical); ceiling ~2.8× | 53.5% of the split | **−46 min** measured (6h51m → 6h05m, −11%); −69 min ceiling; −59/−88 by the real-data model | ≈ 0 | high: a decoder (Rust in `reshape`, or an lbzip2 patch) | low: bzip2's block and stream CRCs, plus the split's checks and parity. Portable: no AVX-512; SSE4.1/PCLMULQDQ/AVX2 dispatched at run time, a scalar fallback, NEON/PMULL for ARM | +2 MB a thread |
| 2 | **No fresh pages per chunk** | reuse the chunk's buffers (lines, Arrow builders, the writer's pages) across chunks; application-level, not an allocator change | n/a | sys is 15% of the split's CPU here (805k faults) | up to −30 min on this VM; a laptop's faults are cheaper (perhaps −5 to −10) | ≈ 0 | medium | none to the data; the gain on a laptop is unmeasured | lower churn |
| 3 | **No oversubscription in the split** | bound the writer's pool while lbzip2 and the workers run (pq-glue `max_workers`, or the pool's size) | n/a | 7–13% of the split's CPU here | −10 to −20 min (single runs: CI must measure) | ≈ 0 | low: a setting | none | none |
| 4 | finalise's Parquet decode compiled for AVX2 | `target-cpu=x86-64-v3` with function multiversioning (or upstream SIMD level decoding) | 1.11× measured (v3); ≤ 1.5× bounded | 57% of finalise's per-byte part | −8 min (v3); ≤ −25 min | ≈ 0–5 min | medium (v3 needs dispatch: a laptop without AVX2 must still run) / high (upstream) | low | none |
| 5 | Parquet encode with levels from the decoder | skip `compute_leaves`' per-leaf recomputation; RLE in SIMD | ≤ 1.3× bounded | 12% of the split; 23% of finalise's per-byte part | ≤ −5 (split) and −7 (finalise) min | ≈ 0 | high (arrow-rs internals, page-for-page equality) | medium | none |
| 6 | the claims decoder compiled for its schema | generated straight-line decode per record, not an interpreted plan | ≤ 2.5–4× bounded; SIMD scanning 0% measured | 9% (slice) to 13% (real) of the split | ≤ −11 to −17 min | ≈ 0 | very high (genson's strict-decode equality) | medium | none |
| — | gz input (the user's decision, not authorised as default) | igzip: 0.66 CPU-s against 19.65 | 30× | the decode | the split becomes download-bound at 3h00m (156.6 GB at 14.5 MB/s) against today's compute-bound 3h20m. **With #1 the bz2 split is ~2h34m, faster than gz at Wikimedia's rate.** | | | | |

**Order-of-magnitude candidates: none.**

- **The split** is at most ~2× compute-reducible in its largest stage (#1). Even then it
  meets Wikimedia's download floor (1h59m).
- **Every other stage** is a few percent of the release, with ≤ 4× kernel ceilings.

## Dead ends (measured unless marked)

- **Assembly or AVX-512 for the inverse BWT:**
  - It is a chain of dependent loads. Vector instructions don't apply, and LLVM's loop
    (`shr`, a load, a store) is already minimal.
  - The fix was to restructure it for memory-level parallelism.
- **Interleaving whole blocks per core:** 1.03–1.27×. K × 3.6 MB of T-vectors outgrow L2
  (the 16 MiB row: 260 cycles a load).
- **Huge pages for the T-vector:** no change.
- **AVX-512 anywhere in the bz2 decode:** the vectorisable parts are < 1 c/B with AVX2
  and PCLMULQDQ.
- **`target-cpu=native` / `x86-64-v3` as a free win:**
  - bz2k: 0% (slightly slower);
  - the Parquet encode: 0%;
  - the Parquet decode: −10% (item 4).
- **SIMD string scanning in the claims decoder:** −5.6% instructions, 0% time. It is
  dispatch-bound.
- **zstd settings in the split:** zstd is 8% of the encode. The writer's work is in
  arrow-rs.
- **Memory bandwidth as the bound anywhere:** the split moves ~4–5 bytes of memory traffic
  per JSON byte at most, ≤ 0.2 GB/s a core against 10 GB/s.
- **Python line splitting:** 0.9 c/B, 1%.

## Floors: the fastest plausible release on a laptop, and what stands in the way

**Assumptions:** 4 cores at ~3.0 GHz sustained, AVX2, 16 GB. This box is 2.1 GHz.

| stage | floor | limiting resource | today (model) | gap, and what causes it |
|---|---|---|---|---|
| download | **1h59m** (103.5 GB at 14.5 MB/s, 3 connections) | Wikimedia's rate | 1h59m | none: the split follows it |
| bz2 decode | 15 c/B × 1.81 TB = 2.5 core-h → **38 min** on 4 cores | the walk's latency at full memory-level parallelism; the T-vector build | 6.3–9.1 core-h (1h35m–2h16m on 4 cores) | the serial walk (#1) |
| the split's own work | reshape ~3 c/B, encode ~8, rest ~2, no faults → ~13 c/B → **33 min** | instructions in arrow-rs and the decoder | 28 c/B + faults 13 + oversubscription 9 | faults (#2), oversubscription (#3), dispatch (#6), arrow-rs (#5) |
| **split** | **= the download, 1h59m** (CPU ~1h10m under it) | the network | 3h20m (bench), 4h10m–5h20m (real-data model) | compute |
| pipeline | ~0 (group files written in the split: the trace's E2) | | 1h35m | per-file and per-worker work (structural) |
| finalise | the per-byte part at ~35 c/B → **~1h30m**, plus its fixed parts | arrow-rs decode and encode; the sort | 2h13m | #4, #5 |
| uploads | 250 GB: ~2h20m at 30 MB/s, under the compute | the link | | at 7 MB/s, ~10 h: the link |

**At ≥ 30 MB/s the floor is about 3h45m–4h00m** (the download, then finalise and its
commits), against 6h51m–7h16m modelled today. What prevents it:

1. the bz2 decode's serial walk;
2. page faults and oversubscription in the split;
3. the pipeline's per-file stage;
4. finalise's arrow-rs decode and encode.

Only (1), (2) and (4) are compute. At 7 MB/s the release is the link's: ~10 h whatever
the compute.

## Uncertainty

- **One box, a VM, single runs** for the system-level figures: page faults, the rayon
  settings and split walls. The kernel figures are best-of-3 TSC, but still this box.
  - The fault cost (~8.5 µs) is high for bare metal.
  - The laptop's clock, L2 and miss parallelism are unknown. The walk's gain depends on
    the last two.
- **The real-data model rests on 12 windows (48 MiB).** The ratio's bootstrap 95% CI is
  15.3–19.4×. The per-byte cost of real lines comes from the same 68,661 lines.
- **Finalise's shares** are the bench's phases on the sample. A release's claims are a
  larger share.
- **The prototype decoder** handles the format as lbzip2 writes it: no randomised blocks
  (bzip2 0.9.5+ never writes them). It was tested on the slice and on 1,732 real-dump
  blocks.

## Reproduce

Everything is under `2026-10-27-compute-frontier/`. Build each Rust kernel with `cargo
build --release` in its folder; the harnesses need no repository code. The slice is
`bench/dumpslice.py` on the bench data's `c0`, `r50`, `s30` and `s90` (bench-data commit
e8c97e1).

| file | what |
|---|---|
| `split_stages.py SRC SLICE.json.bz2\|LINES.jsonl OUT.json` | the split's functions one chunk at a time, CPU and wall each, bytes out (`results/split_stages_*.json`) |
| `reshape_once.py`, `reshape_chunks.py` | `reshape_built` alone, native or under callgrind (`--toggle-collect='*__pyfunction_entity_rows_built*'`) |
| `tools/split_once.sh`, `tools/rt.py` | the real split timed: wall, user, sys, faults, peak RSS |
| `kernels/bz2k` | `phases`, `cat` (with `cmp` against lbzip2), `walk`, `walkm`, `seg`, `tail`, `threads`, `chase [huge]`, `bandwidth`, `probe`, `probecat`; `BZ2K_WALK=seg`, `simd` / `slice8` |
| `kernels/pqk FILE MODE THREADS statement_id=DELTA_BYTE_ARRAY` | the Parquet decode and encode with pq-glue's writer: `split`, `none`, `final`, `final_nocdc` |
| `kernels/jsonk LINES.jsonl` | UTF-8, `memchr2`, SIMD stage 1, copy, simd-json tape |
| `patches/claims-find-end.patch` | the SIMD scanning patch to `claims_arrow.rs` (0% gain) |
| `results/` | the measurements quoted here (`kernels.jsonl`, the callgrind summaries, the local bench run, the real dump's windows) |

**Real dump windows:**

- `curl -r` 12 windows of 4 MiB at (i + 0.5)/12 of `latest-all.json.bz2`.
- `bz2k probe` decodes every whole block in them, with its CRC checked.
- `bz2k probecat` writes their text, and whole lines are taken from it.
