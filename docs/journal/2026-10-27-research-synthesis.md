# External research, checked against the sources and mapped onto #6–#10

**Inputs.** Two research reports relayed by the maintainer, preserved verbatim:
[report 1](2026-10-27-external-report-1.md) (practitioner sources) and
[report 2](2026-10-27-external-report-2.md) (further detail from them). Further sources
the maintainer found during the same session are named in the sections where they were
checked.

**Method.** Each lead was followed to its primary source: the code at a pinned commit, a
changelog line, or a paper section. Each was then held against a measured mibz2 cost, and
the cheapest experiment was run where it took minutes. The chain for each is:

> source → mechanism → measured mibz2 bottleneck → experiment → result

This complements [the literature reading](2026-10-27-literature-reading.md), which read
KKP 2012, KP 2011 and libsais's `unbwt` routines for #6 and #9.

## The bottleneck the leads are measured against

From `examples/phases` on the Wikidata corpus (431 blocks, 11.9 MB → 209.7 MB), one thread,
c/B of output at a nominal 2.8 GHz, best of 3:

| Stage | c/B | Share |
|---|---|---|
| scanner (headers, Huffman) | 1.20 | 5.4% |
| MTF + RLE2 | 2.38 | 10.7% |
| T-vector build | 6.32 | 28.3% |
| walk, flagged | 11.90 | 53.4% |
| RLE1 + CRC | 0.50 | 2.3% |
| **total** | **22.31** | 126 MB/s |

The build and the walk are 81.7% of a decode. A lead aimed at Huffman decoding can move
at most 5%.

## Findings

### 1. libsais 2.12.1: the inverse BWT itself, run on our blocks

Source: [IlyaGrebnov/libsais @ c284ba8](https://github.com/IlyaGrebnov/libsais/tree/c284ba843dd16c159acb976f17bc19382793ea75), `src/libsais.c`, `CHANGES`.

**What it does.**
- 2.12.0 (2026-10-18) added "cyclic Burrows-Wheeler transform and inverse BWT … as used
  by bzip2" (`CHANGES`). `libsais_unbwt_cyclic(T, U, A, n, freq, i)` (L13351) takes a
  bzip2 last column and `orig_ptr` directly. There's no forward BWT to redo and no
  convention to translate.
- For a 900 kB block on one thread, `libsais_unbwt_core` (L12991) does two things:
  - **Init:** `libsais_unbwt_init_single` (L11485).
    - a sequential bigram histogram: `compute_bigram_histogram` (L10627), with checkpoints
      when no `freq` is given (L10500);
    - the ψ² scatter `calculate_biPSI_marks_symbol_rows` (L11263): one byte packed into
      P's low 8 bits, `(i << 8) + c`.
  - **Decode:** `decode_split`, then `decode_chains*` (L12364): 24 interleaved chains per
    thread, from hashed mark rows (`mark_row`, L11034), into 16 KB chunks with successor
    links. Each step emits 2 bytes, the first from a 256-entry `first_symbols`/
    `symbol_ends` lookup (`start_chain_symbol_rows`, L12356).
- **`RUN_LANES`** (L10807, used in every scatter): when `T[i..i+16]` is uniform, and for
  ψ² `T[LF(i)..+16]` is too, 16 rows go in with one SSE store
  (`store_symbol_rows`/`store_rows`) and one counter step.

**Experiment.** This is the reference benchmark R.
- libsais built static (gcc -O3 -march=native, no OpenMP), one thread.
- Reused context and buffers, our `counts` as `freq`, no auxiliary indices (bzip2 has
  none).
- Timed against mibz2's `Segmented` (build + flagged walk) in the same binary, best of 5
  per block, taskset.
- A scratch copy with an rdtsc pair at the init/decode boundary gives the split.

| Corpus | mibz2 build + walk | libsais cyclic inverse | libsais setup / decode | Rows via libsais's run path | Output |
|---|---|---|---|---|---|
| wikidata | 18.21 c/B | **12.10 c/B (−34%)** | 37% / 63% (≈4.6 / ≈7.7 c/B) | 78.5% | byte-identical, 431 blocks |
| text | 17.02 | 18.36 (+8%) | 57% / 43% | 32.8% | byte-identical |
| records | 17.59 | 29.10 (+65%) | 75% / 25% | 0.0% | byte-identical |

Memory: libsais needs A (n + 1 words, the same size as our T-vector), U (n bytes) and a
context with the 256 KB bigram table. That's no more than mibz2's per-slot scratch at one
thread.

**Result and consequences.**
- On Wikidata, libsais builds the *pair* table faster than mibz2 builds the single one,
  and walks it in fewer cycles. It loses where runs are rare (records), which is why its
  advantage depends on the workload.
- The construction mechanism is `RUN_LANES`. My earlier ψ² port (#6, H1) lacked it and
  measured 12.5–14 c/B for its build. Also missing there was the sequential bigram
  histogram, which took that port from 16.6 to 12.5 c/B.
- **Run lanes in our own build: #32.** Our single table needs only `L[i..i+16]` uniform.
  Locally: Wikidata build 5.70 → 2.55 c/B (81.9% of rows), text 5.63 → 4.40, runs
  5.62 → 3.29, no consistent change on records and random. CI decides.
- **#6 reopened, then closed again:** a ψ² kernel with run lanes and the 2-byte walk,
  inside our flagged segmented walk (#34). It was byte-identical, but slower on every
  corpus (Wikidata +40.6% at 1 thread). See [the pair walk](2026-10-27-pair-walk.md).
  After #32, libsais's remaining edge (10.86 against 12.92 c/B) is in its decode loop's
  per-step economy, not in the pair table.
- **libsais's decode has converged on our walk:** many interleaved chains, sampled starts,
  successor links and a prefetch one step ahead. Its remaining walk advantage is 2 bytes
  a step.

### 2. rapidgzip's changelog

Source: [mxmlnkn/rapidgzip @ 063ec89, CHANGELOG.md](https://github.com/mxmlnkn/rapidgzip/blob/063ec89/CHANGELOG.md).

| Entry (line) | What it actually was | mibz2 equivalent | Verdict |
|---|---|---|---|
| 0.14.0, L148–149: "Avoid unnecessary windows … bzip2 … 344 MB/s to 388 MB/s" | 0.13.0 put bzip2 on the gzip `ParallelGzipReader` chunk architecture (L211), which made 32 KiB seek windows. bzip2 blocks need none, so making them was overhead of that architecture | mibz2 has no windows or seek points | doesn't apply; the general lesson (buffer lifetimes) goes under #8 |
| L615: "Only create Huffman LUT up to max code length in block: 140 → 195 MB/s" | Deflate | mibz2's LUT is `LUT_BITS = 10`, two symbols per u32 (`src/block.rs:59`): 4 KB a table, ≤ 24 KB for 6 tables, L1-resident | no experiment: the whole scanner is 5.4% |
| L616: "Decrease double-cached Huffman LUT cached bits: 165 → 255 MB/s" | Deflate | as above. indexed_bzip2's own bzip2 sweep of LUT widths 11–18 bits found 12 best, all within ±10% (`bzip2.hpp` L210–218) | as above |
| L407: markers replaced in place, Silesia +11%, FASTQ +36% | Deflate's 16-bit marker buffer reinterpreted as 8-bit | the T-vector built over the MTF output (#8) | see 3 |
| L492–493: memset/memcpy for back-references | Deflate | Precopy (#9) copies known bytes with `ptr::copy` | already the #9 plan |
| L500: rpmalloc, 2.5 → 3.1 GB/s | allocator | no allocator changes (CLAUDE.md) | out of scope |
| L5–6: block finder simplified, 64 → 92 MB/s | Deflate's block finder | #10's scanner | #10 stays deferred: the scanner binds only from about 10 workers |

### 3. In-place T-vector construction: indexed_bzip2 and libbzip2

Sources:
- [mxmlnkn/indexed_bzip2 @ 5e68a96, `src/indexed_bzip2/bzip2.hpp`](https://github.com/mxmlnkn/indexed_bzip2/blob/5e68a96/src/indexed_bzip2/bzip2.hpp): `dbuf` is `std::vector<uint32_t>(900000)` (L416); MTF writes each byte into its low 8 bits; `prepare()` (L811–831) does `dbuf[byteCount[uc]] |= i << 8`; the walk follows (L839 on).
- libbzip2's `decompress.c` ([mirror](https://github.com/centricular/bzip2/blob/master/decompress.c)), found by the maintainer: `s->tt[nblock] = (UInt32)uc` during MTF (L412), then `s->tt[s->cftab[uc]] |= (i << 8)` (L562).

**What mibz2 does.**
- Same layout: `build` (`src/bwt.rs`) packs byte | link << 8, and the walk reads only
  `tt`.
- The difference: MTF writes a separate `ll: Vec<u8>` (`src/mtf.rs`), and `build` widens it
  into `tt` (`tt.extend(ll.iter().map(|&c| c as u32))`) before the link pass, which also
  reads `ll`.

**What in-place would remove:**
- the `ll` buffer, 0.9 MB per slot;
- the widening pass, inside the build's cost.

**Compatibility with the flagged walk.** The walk flags START (bit 31) on segment starts
and clears it again. It never needs `ll` once the build is done, so there's no conflict.

The cost moves: MTF writes 4 bytes per output byte instead of 1, and #32's run check would
read `tt`'s low bytes, or a byte view of them.

This is H4 (#8), already planned as "decode … straight into the T-vector's words".

**Cheapest experiment:** MTF writing into `tt` in a scratch harness, then build + walk
c/B against the current path on the Wikidata blocks, plus the RSS saving at 4 threads.
After #32, because #32 changes what the build reads.

### 4. Seward's sliding-block MTF (libbzip2, the maintainer's lead)

Source: libbzip2 `decompress.c` L361–470.
- `mtfa[MTFA_SIZE = 4096]` holds 16 blocks of `MTFL_SIZE = 16`, with `mtfbase[]`.
- An index below 16 is a short shift inside block 0 (L429).
- A larger one moves one entry across each block boundary (L447–461), and the array is
  re-packed when `mtfbase[0]` reaches 0 (L463–470).

**What mibz2 does** (`src/mtf.rs`): a 256-byte list, with an AVX2 blend for indices below
32 and a scalar shift otherwise. The journals have no sliding-block variant.

**Measured ceiling.** These are the shares of MTF moves with an index at or above each
bound, the only moves a sliding block could speed up past AVX2's below-32 path:

| Corpus | ≥ 16 | ≥ 32 | ≥ 64 |
|---|---|---|---|
| wikidata | 15.2% | 1.52% | 0.23% |
| text | 18.0% | 6.74% | 1.47% |
| records | 95.5% | 90.6% | 75.9% |

On Wikidata, 98.5% of moves already take the AVX2 path. The whole MTF + RLE2 stage is
2.4 c/B, so the sliding block can't move Wikidata measurably. It would matter only for
records-like data. **No experiment now**; noted under #8, in case a records-heavy workload
appears.

### 5. Weißenberger & Schmidt (ICPP 2024), GPU inverse BWT and MTF (the maintainer's lead)

Sources: [doi:10.1145/3673038.3673067](https://doi.org/10.1145/3673038.3673067); code read
at [weissenberger/bzip2gpu @ 66d263c](https://github.com/weissenberger/bzip2gpu/tree/66d263c). The paper's text itself was not read, only its code.

**What the code does.**
- **iBWT** (`src/ibwt.cu`): the permutation comes from a segmented stable radix sort of
  the BWT bytes with their indices (`cub::DeviceSegmentedSort::StableSortPairs`, in
  `batch_iBWT`, L203 on). That's a counting sort's job done as a GPU sort. Then
  Helman–JáJá list ranking:
  - sublists cut at every `LIST_DISTANCE`-th row (`helman_jaja_local`, L85);
  - each thread walks one sublist, writing a rank and a sublist id per element;
  - a reduce ranks the sublists, and `add_permute` scatters every byte to its place.
- **iMTF** (`src/imtf.cu`): each tile's MTF effect is a permutation of the 256-entry list
  (`private_perm`, "Algorithm 4"/"Algorithm 5", L191–254). An up/down-sweep scan composes
  them (`concat_perms_*`, `scan_perms`, L322), so tiles decode in parallel within a block.

**Against mibz2.**
- The list ranking is our flagged segmented walk (sublists from fixed splitters ↔ our
  pseudo-random flagged starts). It pays two extra 4-byte arrays per element (rank,
  sublist id) and a second scatter pass, which a CPU walk avoids by writing each segment's
  bytes straight into its buffer.
- The scanned iMTF parallelises inside a block. mibz2 already parallelises across blocks,
  which has ample parallelism at 4–16 threads. Intra-block MTF would matter only for time
  to first byte or for a file of very few blocks.
- **Nothing to transfer to the CPU decoder.** Recorded as an overlap.

### 6. Intel IPP `ippsDecodeBlock_BZ2_16u8u` (the maintainer's lead)

IPP is distributed as binaries, so its implementation can't be inspected. It could serve as
an opaque timing reference only, and it needs Intel's runtime. **Not pursued.** libsais now
gives an inspectable reference on the same input (finding 1).

### 7. KKP 2012 and the libsais Rust wrapper's advice on auxiliary indices (report 2)

- Auxiliary indices are `I[]` samples of the original text's positions, produced by the
  *forward* `libsais_bwt_aux`. A bzip2 stream carries only `orig_ptr`, so they can't exist
  for our input, and finding 1 ran without them.
- libsais 2.12's chain decode gets its parallelism without them, from hashed mark rows
  (`UNBWT_CHAIN_MIN_SIZE` against `UNBWT_CHAIN_MIN_SIZE_AUX`, L60–62).
- KKP §3's "one counting-sort pass over character pairs" is the sequential bigram
  histogram of finding 1, as the literature reading already notes.

## Mapped onto the issues

| Issue | Finding | Status |
|---|---|---|
| #6 LF² / ψ² | 1: libsais's run lanes + 2-byte walk beat our build + walk by 34% on Wikidata, byte-identical | reopened, ported (#34), slower: **closed**. The walk-loop question is queued |
| #7 build (the goal) | 1: `RUN_LANES` in our single-table build | **#32**, in CI |
| #8 fused front end, memory | 3: in-place T-vector (indexed_bzip2, libbzip2); 4: sliding MTF, ceiling measured as negligible on Wikidata; 2: buffer lifetimes | H4 as planned; 3 is its first experiment, after #32 |
| #9 Precopy | 2: memcpy for repeats is already the plan | unchanged: forward chains, 44.7% coverage |
| #10 scanner | 2: rapidgzip's block-finder simplification is Deflate's | deferred |

**No new issues.** Every finding fits #6–#10 or is out of scope.

**Order from here** (the decision log is wikidata-pq#97):
1. #32: merged, −16% decode, released as 0.1.4.
2. The ψ² port (#34): slower, closed.
3. Precopy (#9), on the single table.
4. The in-place front end (#8).
5. Per-stage figures in the CI bench, then libsais's decode-loop mechanics in our ψ walk.

## Not done, and why

- **Hardware-counter evidence** (cache misses, instructions) for libsais against mibz2:
  `perf` isn't installed for this kernel, and no PMU tooling exists in these repositories.
  hotpath was evaluated in [the compute-frontier journal](2026-10-27-compute-frontier.md)
  and not wired in.
- The rdtsc pair above is a boundary split, not a counter profile. A `perf_event_open`
  helper, a candidate for a later shared measurement crate (provisional name: tiramisu),
  would add it when a question needs it.
- **The Rapidgzip paper** (Knespel & Brunst, HPDC 2023,
  [arXiv:2308.08955](https://arxiv.org/abs/2308.08955)): its subject is gzip's chunk
  windows, which bzip2 doesn't need. Not read beyond its abstract.
