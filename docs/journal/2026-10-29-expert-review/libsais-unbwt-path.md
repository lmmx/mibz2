<!-- A delegated source reading (2026-10-29), spot-checked against libsais.c at c284ba8: the size thresholds (L55-70), the skipped histogram (L11493-11496), init_parallel's recount (L11572), the decode's prefetch (L12496) and the chain-path test (L13284). Kept as written. -->

# libsais 2.12.1 `libsais_unbwt_cyclic_ctx`: source-level path (n = 900,000, freq given)

Source: `libsais-st/libsais.c` (commit c284ba8, VERSION file = 2.12.1). All `L####` are line numbers in that file.
"Inference" marks anything not read directly from the source. The disassembly figures come from the local
`libsais-st/libsais.o` (no GOMP symbols, so this is the single-threaded build, gcc -O3 -march=native on *this* Cascade Lake
Xeon). The harness binaries on the i9 and the EPYC may have different codegen.

**Headline correction to the brief.** For n = 900,000 the decode does **not** use the "fastbits lookup + bucket2 search"
scheme. It takes the **symbol-rows** path (n <= `UNBWT_SYMBOL_ROWS_MAX_SIZE` = 8,388,607, L69):
- each P entry holds `(PSI²(row) << 8) | middle symbol`;
- the other symbol comes from a 28 KB `first_symbols` table plus a 1 KB `symbol_ends` table;
- bucket2 is not read during the decode (comment at L13228-13229).

The fastbits/bucket2 decode (`decode_1..16`, `decode_chains`, `decode_chains_large`) runs only when n > 8,388,607, or as
the failure fallback (section 1c).

---

## 1. Call path

### 1a. Entry and selection (identical for the ST ctx and the OMP ctx)

| Step | Lines | What happens for n = 900,000, r = n (cyclic), freq != NULL |
|---|---|---|
| `libsais_unbwt_cyclic_ctx` | L13368-13383 | Validates. `index = i + 1` (1-based primary row). Calls `main_ctx(ctx, T, U, A, n, LIBSAIS_FLAGS_CYCLIC, freq, r = n, &index)`. |
| `libsais_unbwt_main_ctx` | L13273-13293 | n > `UNBWT_PACKED_MAX_SIZE` (65,535, L68), so the packed path is skipped. `threads = ctx->threads`. `chains_threads = chains_threads(n, threads)` (L10312-10318) = min(max(n/262144 = 3, 8), threads, 64) = **threads** (1, 2 or 3). |
| chain-path test | L13284 | `ctx->marks != NULL` holds (allocated in `create_ctx_main`, L10350). `n >= 65536` holds (`r < n` is false, so `UNBWT_CHAIN_MIN_SIZE`). `((n-1)/r + 1)/8 = 0 < threads` holds. **Chain path taken.** |
| marks geometry | L13286-13287, L10330-10336 | `chains_slots(n, T)` = 24, because n < 16 MiB (L10325-10328). `chain_marks = max(32·24·T, min(n>>8, 4096))` = max(768·T, 3515) = **3515** for T ≤ 3. `marks_shift` = **8**: 900000>>8 = 3515 ≥ 3515, but >>9 = 1757 < 3515. |
| chunks | L13289, L10320-10323 | `chunk_size(n)` = 16,320 (`UNBWT_CHUNK_SIZE`; n ≥ 262,144). `chunks_count = 900000/16320 + 24·T = 55 + 24·T`. |
| `libsais_unbwt_core` | L12991-13011 | Cyclic: `index = 0`, `lastc = 256` (none), `lastu = T[I[0]-1]`, `end = I[0]`, `start = I[0]`. Order: `touch` → `init` → `decode_split`. If `decode_split` returns non-zero, the fallback runs (1c). |
| `libsais_unbwt_touch` | L12952-12973 | Writes `P[k] = 0` for every 1024th entry (one per 4 KB page) of P[0..n]. This is an OMP region when threads > 1. |
| `libsais_unbwt_init` | L12975-12989 | `init_threads > 1 && n >= 262144` selects **`init_parallel`** (OMP ctx with T ≥ 2). Otherwise it selects **`init_single`** (ST ctx, or the OMP ctx with T = 1). |

### 1b. Build (`init_single`, L11485-11539)

- L11490: `shift` = 3. This is the fastbits shift, and it is unused on this path.
- L11491: `checkpoints_shift` is unused, because freq != NULL.
- L11493-11496: **freq != NULL, so the histogram is skipped.** `memcpy(count1, freq)` runs and `checkpoints_count` stays 0.
  `compute_histogram_checkpoints` (L11501) is **not** called.
- L11504-11507: `bucket1[c]` = 1-based start row of symbol c (prefix sum from 1).
- L11509-11512: the symbol range [first used, last used] is rounded out to multiples of 16.
- L11514: `memset` of bucket2 rows over that range (up to 256 KB).
- L11515: `compute_bigram_histogram` (L10627-10660). Because `checkpoints_count = 0`, the else-branch at L10652-10655 runs
  for each non-empty symbol c: `compute_histogram(&T[bucket1[c]-1], count1[c], &bucket2[c<<8])`. Then
  `transpose_bucket2` runs (L10659, L10584-10625).
- L11517: skipped (`lastc = 256`).
- L11519 `if (marks != NULL && n <= 8388607)` is **true**, so:
  - L11521 `calculate_bigram_starts` (L10766-10785; SSE `prefix_bigrams` L10666-10694)
  - L11522 `calculate_first_symbols(bucket1, n, (uint8_t*)fastbits)` (L10787-10803)
  - L11523 `calculate_biPSI_marks_symbol_rows(..., 0, n)` (L11263-11483)
- The else-branch (`calculate_fastbits` and `calculate_biPSI[_marks]`, L11525-11535) is **not taken**.

### Decode (`decode_split`, inlined into `core`; L12772-12950)

1. **Mark enumeration.** L12782-12797, serial. Loops over marks 0..n>>8 (3516 marks).
   - `mark_row(m)` (L11034-11039) is one row per 256-row window, at a Fibonacci-hashed offset. The mark at `end>>8` lands
     exactly on `end`.
   - A chain is created when the mark row is not 0, `start` or `end_lf`, and its LF is neither `start` nor a mark row.
   - This gives about 3.5k chains.
2. **Chain setup.** L12801-12812: `omp parallel for schedule(static,64)`.
   - It saves `P[row]` and `P[lf]`, and `gap = T[row-1]`.
   - It overwrites `P[row] = MARK | c<<1` and `P[lf] = MARK | c<<1 | 1`.
   - L12816: `P[start]`, `P[end]` and `P[end_lf]` get the terminal marks. `start == end` in the cyclic case.
3. **Decode dispatch.** L12824-12851. n ≥ 262,144, so the code goes to the else-branch at L12833 and opens
   `omp parallel num_threads(threads) if(threads > 1)`. Inside it, n ≤ 8,388,607 selects
   **`libsais_unbwt_decode_chains_symbol_rows(..., batch = threads >= 3 ? 8 : 1, ...)`** (L12840, L12466-12566).
   - `_small` (4,080-byte chunks) is only for n < 262,144 (L12826-12830).
   - `_large` (64 lanes) is only for n > 8,388,607 with 64 slots (L12842-12845).
4. **Linking.** L12854-12873, serial. Follows `successor` links from chain 0 and assigns each chain's output `position`.
   On failure it restores P and returns 1 (L12864-12870).
5. **Stitch.** L12875-12922. `omp parallel for schedule(static,64)` over chains. It `memcpy`s each chain's chunk-linked
   bytes into `A = (uint8_t*)P` at `position`, and appends the 1-byte `gap` when `successor & 1`.
6. **Periodic replication.** L12924-12926 (a no-op when cycle == n).
7. **Final copy.** L12928-12945. `omp parallel`, and each thread `memcpy`s its n/T slice `A → U`.

### 1c. Fallback (only if `decode_split` returns 1)

- L13001-13006: `init` runs **again**. `marks = NULL` sends `init_single` to its else-branch: `calculate_fastbits` +
  `calculate_biPSI`, where `P[pos] = row` (L10918, L10928).
- Then `decode_omp` (L12274-12302) runs:
  - `blocks = 1 + (n-1)/r = 1`, so `max_threads = 1`;
  - `decode` (L12153-12272) takes the `blocks == 1` branch and calls **`decode_1`** (L12177-12181, L11689-11701): one
    serial chain, no interleaving, no prefetch.
- `UNBWT_DECODE_12_MIN_SIZE` / `_16_MIN_SIZE` (L66-67) and `decode_2..16` matter only when r < n (aux/sampled
  indexes). **The cyclic call never uses them except via this one-chain fallback.**

### ST vs OMP

| ctx | build | decode |
|---|---|---|
| `create_ctx()` (ST build, or OMP build with T = 1) | `init_single`; freq used | `decode_chains_symbol_rows` with 24 lanes; `take()` without lock (L12310) |
| `create_ctx_omp(2 or 3)` | `init_parallel`: **freq ignored**, and each thread recomputes the histogram of its T slice (L11572) | 24 lanes **per thread**; chains and chunks handed out under `omp critical` (L12312) |

---

## 2. Data layout and one decode step

### Tables

| Object | Size (allocation) | Content on this path |
|---|---|---|
| **P = A** | (n+1) × 4 B = 3.6 MB | Entry `P[q]` for 1-based row q is `(i << 8) \| c`, written at L11393-11474 as `P[bucket2[w]++] = (i<<8) + c`. Here `i = PSI²(q)` (the row two steps forward) and `c = L[i] = F[PSI(q)]` (the middle symbol). **Inference from the construction:** the position `bucket2[w]++` with `w = (L[LF i] << 8) \| L[i]` is the rank of row `LF²(i)` inside its bigram bucket. Bits 8-30 hold the row (hence the 2²³-1 cap, L69). Bit 31 = `UNBWT_MARK` (L71) holds chain marks: `MARK \| chain<<1 \| parity` (L12810, L12816). |
| **bucket2** | 256 × 256 × 4 B = 256 KB (L10347) | Bigram histogram, transposed (L10659) so that the index is `(first << 8) \| second`. `calculate_bigram_starts` turns it into 1-based bigram start rows. It is used only as write cursors in `biPSI`. **Not read by the symbol-rows decode** (L13228-13229). |
| **fastbits buffer** | (1 + 2¹⁷) × 2 B = 256 KB (L10348) | Reinterpreted as follows. `first_symbols` = uint8 at bytes [0, 28126): `first_shift` = 5 (900000>>5 = 28125 < 2¹⁵, L10791), so there is one cell per 32 rows and `first_symbols[j]` = F-symbol of row 32j (memset fill, L10797-10802). `symbol_ends` = uint32[256] at byte offset 32,768 (L10789): `symbol_ends[c]` = one past the last row of c (L10794-10795). The 1<<17 fastbits **index** with shift 3 is **not used** on this path. |
| marks | ctx: (2·4096+2) × 4 B | About 3516 used. `marks[m]` = LF(mark_row(m)) (L11393-11474). |
| chains | ctx: 8194 × 40 B | One `LIBSAIS_UNBWT_CHAIN` (L130-142) per chain: row, next, lf, lf_next, chunk, offset, length, successor, position, gap. |
| slots | stack: 24 × 48 B (L12468, L144-152) | Per lane: `p`, `chain`, `chunk`, `length`, `out`, `end`. |
| U chunks / slack | 55 chunks of 16,320 B inside U, plus slack | **Slack size, from the source:** the ctx allocates `chains_slots(SAINT_MAX, T) · T · 16320` (L10351). For T ≤ 4 that is 64·T chunks. **Inference:** the run uses at most 24·T chunks. 16,320 = 255 cache lines, so consecutive chunks shift by 63 L1 sets (inference: chosen against set or 4K aliasing). |

### One lane step = 2 output bytes (L12493-12499; lockstep loop L12482-12530)

```
p = slot->p                       // load (stack); store-forwarded from the previous round
q = P[p]                          // THE dependent random load (3.6 MB)
if (q & MARK) -> chain switch
prefetcht0 P[q >> 8]              // next row of THIS lane, issued one full round ahead
c = first_symbols[p >> 5]         // load, depends on p only (off the q chain), 28 KB random index
if (symbol_ends[c] <= p) c++...   // load, 1 KB; loop only when a 32-row cell straddles a symbol boundary
out[offset] = c; out[offset+1] = (uint8)q   // one 16-bit store
slot->p = q >> 8                  // store
```

Disassembly of the hot path in the local `.o` (`decode_chains_symbol_rows.constprop.0`, 0xeaa0-0xeae5):
- **21 instructions per lane step**: 5 loads (`slot->p`, `P[p]`, `first_symbols`, `symbol_ends`, `slot->out`),
  1 `prefetcht0`, 2 stores, 3 macro-fusable branches.
- The step includes `mov dh,bl` followed by a 16-bit store of `dx`. **Inference:** this is a high-8 partial-register
  write; on Intel it costs a merge uop, on AMD it does not.
- About 6 more instructions per round of 24 lanes.
- That is about **10.5 instructions per output byte** in the decode.

**Dependency structure per lane.**
- The only load on the loop-carried chain is `P[p]`. It is reached through `slot->p`: a store, then a load in the next
  round.
- `first_symbols` and `symbol_ends` hang off p and do not feed the next address.
- `P[p]` is usually prefetched by the same lane's previous step, about 24 × 21 ≈ 500 instructions earlier.
- **Exception:** the first load after each chain start is not prefetched. `start_chain_symbol_rows` (L12356-12362) sets
  `slot->p = next >> 8` without a prefetch. That happens about 3.5k times per block.

**Loads per 2 output bytes.**
- 1 dependent random load (P, 3.6 MB).
- 2 independent small-table loads: 28 KB + 1 KB, normally L1 or L2.
- 2 stack loads and 1 prefetch.
- The block needs n/2 = 450k dependent loads (versus n for a one-symbol-per-load T-vector).

**Inference on cache pressure.** The 28 KB `first_symbols` table and `symbol_ends` (at a 32 KB offset, so it maps to the
same sets as `first_symbols[0..1023]`) nearly fill a 32 KB 8-way L1D. Several more working sets compete for it:
- about 24 prefetched P lines;
- 24 output lines;
- the slot and chain records.

---

## 3. Build cost (ST, freq given)

| Pass | Lines | Reads | Writes | Pattern |
|---|---|---|---|---|
| page touch | L12952-12973 | – | `P[k·1024]` | one store per 4 KB page of P (3.6 MB) |
| freq → count1 → bucket1 | L11495, L11504-11507 | 1 KB | stack | – |
| memset bucket2 range | L11514 | – | ≤ 256 KB | sequential |
| **bigram histogram** | L10636-10657 → `compute_histogram` L10427-10498 | **T once, sequentially** (as up to 256 sub-ranges, one per F-bucket) | per call: a 4.3 KB 4-copy stack histogram (memset L10437) reduced into bucket2 row c (L10495) | 64 B uniform-run shortcut (L10453); software prefetch at +256 B (L10450) |
| transpose | L10584-10625 | bucket2 range | bucket2 range | 4×4 block swaps |
| bigram starts | L10766-10785 | bucket2 range | bucket2 range | SSE prefix sums (L10666-10694) |
| first_symbols | L10787-10803 | bucket1 | 28 KB memset + 1 KB | sequential |
| **biPSI_marks_symbol_rows** | L11377-11475 (cyclic: the first half, L11270-11373, is empty because `index = 0`) | **T sequentially** (`T[i-1]`). **T[p-1]** with `p = bucket1[c]++`: 256 interleaved sequential streams, one per F-bucket. `bucket1[c]` RMW (1 KB stack). **`bucket2[w]` RMW** (256 KB, random over active bigrams). | **`P[bucket2[w]++]`**, n × 4 B scattered over **up to 65,536 bigram streams**, each sequential. `marks[]` about 3.5k entries. | 16-row run shortcut (L11385-11397): both 16-runs uniform gives one SIMD store of 16 entries (L10874-10884). No software prefetch in this function. |
| chain setup | L12801-12812 | about 3.5k × (2 random P + 1 T) | about 3.5k × 2 random P | – |
| stitch | L12877-12921 | n bytes of chunks (U + slack) | n bytes into `(uint8_t*)P` | sequential memcpy, per chain |
| final copy | L12928-12945 | n bytes of P | n bytes of U | sequential memcpy |

**Totals over n-sized arrays (from the table; the comparison is inference):**
- **Sequential:** 2 reads of T, 1 page-touch of P, 2 extra n-byte copies (chunks → P → U).
- **Streamed or scattered:** 1 read of T in 256 streams; 1 write of P in up to 65,536 streams.
- **Decode:** n/2 dependent random reads of P, plus n bytes of output in 24·T interleaved streams.
- A plain bzip2 T-vector build with freq known is 1 sequential read and 1 scatter to 256 streams. libsais adds:
  - a full T pass for the bigram histogram;
  - a second RMW counter per row (`bucket2[w]`, 256 KB);
  - a dependent `T[p-1]` read per row;
  - 256× more write streams into P.

---

## 4. Traversal scheduling

- **Lanes.** The symbol-rows variant always has `UNBWT_CHAINS` = **24** lanes per thread (L56, `slots[UNBWT_CHAINS]`
  L12468). This is a compile-time constant.
  - 64 lanes (`UNBWT_CHAINS_LARGE`, L57) is only for n ≥ 16 MiB with ≤ 4 threads (L10325-10328), and only in
    `decode_chains_large`.
  - **Inference:** the author raises the lane count only once P outgrows cache.
- **No decode_N.** The cyclic chain path never uses `decode_2..16`.
- **Chunks of text.** About 3.5k marks, one per 256 rows at hashed offsets (`mark_row`, L11034-11039), cut the single
  PSI cycle into chains.
  - Each mark is set at both `row` and `LF(row)` (L12810), because a PSI² walk visits only every other text position.
    Whichever parity a walk has, it stops at one of the two marks.
  - Stopping at the LF mark sets bit 0, and the stitch inserts the `gap` byte `T[row-1]` (L12808, L12920).
  - **Inference:** the mean chain length is about n/3.5k ≈ 256 bytes, and lengths are roughly geometric because the
    marks' text positions are effectively random.
- **Lockstep main loop** (L12482-12531):
  - `room` = the minimum free space across the 24 slots' current chunks (L12484-12485);
  - then `room/2` rounds over all 24 slots;
  - on a mark, the lane records `successor` and `length` (L12505), takes the next chain index (`next_chain`,
    L12326-12331; batch of 1, or 8 when threads ≥ 3) and restarts in place;
  - full chunks take a new chunk and record `links[old] = new` (L12338-12346, L12526-12530).
- **Tail.** Once chain indices run out, `active` drops below 24. A round-robin tail loop (L12533-12565) finishes the
  remaining lanes with per-step chunk checks, so lane parallelism falls as lanes finish.
- **Output order.** Each chain writes contiguously into its slot's chunk sequence. Chain order inside chunks is
  arbitrary, so the bytes are put in text order afterwards by the serial link walk (L12858-12862), the per-chain memcpy
  into P (L12877-12921) and the copy to U (L12928-12945).
- **Software prefetch.** Exactly one site on this path: `libsais_prefetchr(&P[q >> 8])` at L12496 (tail loop: L12540).
  - The macro is `__builtin_prefetch(addr, 0, 3)`, i.e. `prefetcht0` (L232-233).
  - Its distance is **one step of the same lane**, which is one full 24-lane round of about 500 instructions.
  - There is no prefetch in `biPSI_marks_symbol_rows`, the setup, or `first_symbols`.
  - Only `compute_histogram` prefetches T at +256 B (L10450).

---

## 5. OpenMP coordination (ctx from `create_ctx_omp(T)`, T = 2 or 3)

Parallel regions per call:

| # | Region | Lines | Split | Serial parts / sync |
|---|---|---|---|---|
| 1 | touch | L12955 | static n/T pages | join |
| 2 | init_parallel | L11551-11685 | T blocks of `(n/T) & -16` rows (L11565-11567) | **4 explicit barriers** (L11575, L11610, L11619, L11664). One **`omp master`** section (L11577-11608): T×256 prefix sums plus `calculate_first_symbols` (28 KB memset), while the others wait. |
| 3 | chain setup | L12802 | `parallel for schedule(static,64)` over about 3.5k chains | implicit barrier |
| 4 | decode | L12835 | dynamic: chains in batches of 1 (T = 2) or 8 (T = 3); chunks one at a time; all via `take()` under `#pragma omp critical(libsais_unbwt_take)` (L12305-12324) | join |
| 5 | stitch | L12903 | `schedule(static,64)` by **chain count**, not bytes | implicit barrier |
| 6 | final copy | L12929 | static n/T bytes | join |

Serial between regions: the mark enumeration (L12784-12797), linking (L12854-12873) and replication (L12924-12926).

Work inside `init_parallel`:
- Each thread computes its own `count1_local` histogram over its T slice. **This happens even though freq is supplied**
  (L11571-11573).
- Each thread builds a full per-thread bucket2 (256 KB each, `buckets` L10349) over its share of T (L11614-11617).
  bucket2 row-slices are then merged and prefix-summed per thread (L11621-11662).
- Each thread scatters its block's rows into P (L11666-11683).

**Straggler candidates (from the source; costs are inference):**

1. **Fallback path.** If `decode_split` returns 1 (L12864-12870), the whole build is redone (L13005). Then
   `decode_omp` uses `max_threads = min(blocks = 1, T) = 1` (L12278-12281) and `decode_1`. One thread then walks all
   n/2 = 450k dependent loads serially, without interleaving or prefetch. **Inference:** at roughly 15-100 ns per load
   that is about 7-45 ms, the same order as the 13 ms seen. The source does not say when this triggers for valid input:
   the linking fails only if the successor walk from chain 0 does not reach the end marker within n bytes. Note also
   that chunk exhaustion (`return` at L12476, L12529, L12561) would cause the fallback.
   **Inference:** chunks are sufficient, since `chunks_count = floor(n/16320) + 24·T` bounds the chunks used.
2. **Lock-holder preemption.** `take()` is a named critical section, taken about 3.5k + 100 times per block at T = 2
   (batch 1), or about 440 + 100 at T = 3. If the OS preempts the holder, every team member blocks.
3. **Fork/join and barrier count.** 6 regions + 4 explicit barriers + 2 implicit `for` barriers per block. With libgomp's
   spin-then-sleep waits on an oversubscribed machine (e.g. several workers × 3 threads on 4 vCPUs), any preempted
   member delays the whole team. **libgomp behaviour is not in the libsais source (inference).**
4. **Serial `omp master` section** in `init_parallel`, and the serial mark and link loops in `decode_split`.
5. **Static, data-dependent splits.**
   - `biPSI` is split by T rows; its run shortcut (L11385) makes speed depend on the data.
   - The stitch is split by chain count in blocks of 64 (L12903).
   - In the decode tail, each thread drains its last ≤ 24 chains with falling lane counts.
   - **Inference:** these are small, on the order of the max chain length (about 1-2k bytes).

---

## 6. Changelog (verbatim, from `CHANGES` at c284ba8; `CHANGES.md` 404s; `VERSION` = `2.12.1`)

```
Changes in 2.12.1 (October 21, 2026)
- Fixed compilation errors and warnings with MSVC for ARM64 and ARM64EC targets.
- Fixed incorrect results on big-endian AArch64 targets, and of libsais64 and libsais16x64 on all big-endian targets.
- Fixed byte-order detection on big-endian BSD targets.
- Big-endian targets now always use the portable (non-SIMD) code.

Changes in 2.12.0 (October 18, 2026)
- Changed license to Apache License 2.0 with LLVM Exceptions.
- Support for cyclic Burrows-Wheeler transform and inverse BWT construction (e.g. as used by bzip2).
- Significantly improved performance of inverse BWT, both single- and multi-threaded.
- Improved performance of suffix array and Burrows-Wheeler transform construction.
- Fixed crashes and incorrect results when OpenMP provides fewer threads than requested (e.g. under OMP_THREAD_LIMIT).
- Fixed out-of-bounds memory access in inverse BWT for inputs that are not a valid BWT.
- Fixed handling of invalid alphabet sizes, of alphabets larger than 2^31 symbols (libsais64, libsais16x64) and of very large thread counts in 32-bit builds.
- Resolved compiler warnings with GCC 16.
```

2.12.1 has no x86 or unbwt performance change. The cyclic unbwt and the rewritten unbwt arrived in 2.12.0.

---

## 7. Why it might win on the i9 and lose on the EPYC — HYPOTHESES ONLY

| # | Hypothesis | Source fact it rests on | Test |
|---|---|---|---|
| H1 | **The prefetch lead time is too short for EPYC memory latency.** The lane count is fixed at 24 and the prefetch runs one round ahead, so the lead is about 24 × 21 ≈ 500 instructions (about 30-50 ns). On the i9, P (3.6 MB) sits in the 20 MB L3 and the lead covers L3 latency. On a cloud EPYC, if P is served from DRAM or a contended L3, latency exceeds the lead. Throughput is then capped near 24 lanes × 2 B per memory latency. Consistent with the i9 gap shrinking at 4 workers, when 4 × about 6 MB of working sets exceed 20 MB L3 (inference). | L56, L12468, L12496. The author switches to 64 lanes only for n ≥ 16 MiB (L57-58, L10325-10328). | On the EPYC: `perf stat` with Zen fill-source events (`ls_sw_pf_dc_fills.*`, `ls_dmnd_fills_from_sys.*`, or IBS `perf mem`) on the P loads, and cycles per lane step. Micro-variant: rebuild with `UNBWT_CHAINS` = 32, 48 or 64 (L56; the `#define` is unguarded, so edit it). If the EPYC speeds up and the i9 does not, H1 holds. |
| H2 | **The instruction budget pays for itself only at high clock.** About 10.5 instructions per byte in the decode, plus about 2 extra T passes, a 65,536-stream scatter and 2 n-byte memcpys. If libsais is throughput-bound and mibz2 is latency-bound, the i9's higher clock favours libsais. A lower-clocked EPYC (or one with a busy SMT sibling) favours mibz2. | Disassembly (21 instructions per 2 B); section 3 pass table. | `perf stat -e instructions,cycles,task-clock` for both kernels on both CPUs. Compare instructions per byte, IPC and effective GHz. If the time ratio tracks instructions/IPC, the code is compute-bound. Check `lscpu` thread siblings on the runner. |
| H3 | **Load-port pressure.** Each step issues 5 loads + 1 prefetch + 2 stores, with lane state round-tripping through the stack `slots[]` (`slot->p` store, then load next round). Port mix and store-forwarding latency differ by microarchitecture (inference). | L12493-12499; disassembly 0xeae5 / 0xeace. | Port and dispatch counters (`ls_dispatch.*` on AMD, `uops_dispatched.port_*` on Intel). Micro-variant with lanes kept in registers (e.g. 8 lanes in locals, ×3) to see whether the EPYC gains. |
| H4 | **L1D contention between `first_symbols` and the P prefetches.** The 28 KB random-index table plus `symbol_ends` (same sets, 32 KB offset) nearly fill the 32 KB L1D, and evictions can drop prefetched P lines before their use about 500 instructions later. L2 size and latency differ (i9 256 KB; Zen2/3 512 KB; Zen4 1 MB), so the cost differs (inference). | L10787-10803, L12498, allocation L10348. | IBS / `perf mem` data-address sampling: split L1 misses into P, `first_symbols` and output. Micro-variant: `UNBWT_FIRST_SYMBOLS` = 13 or 14 (L70), which shrinks the table and lengthens the correction loop. |
| H5 | **Build-phase cost differs by microarchitecture.** Each row does two indexed memory RMW counters (`bucket1[c]++`, `bucket2[w]++`), loop-carried through store-forwarding when symbols repeat, plus a dependent `T[p-1]` load and a scatter over up to 65,536 streams. | L11380-11475 | `perf record`: share of `init_single` vs `decode_chains_symbol_rows` vs `memcpy` vs `compute_histogram` on each CPU. Cycles per row in `init_single`. |
| H6 | **Extra copy and touch passes.** The stitch and final copy move 2n bytes (L12877-12945), and the page touch writes once per page (L12952). memcpy strategy and page-fault cost under a VM differ (inference). | L12875-12945, L12952-12973 | `perf` share of memcpy, `page-faults`; check whether the harness reuses A and U across calls. |
| H7 | **TLB.** Random P accesses span about 880 × 4 KB pages, and every step likely misses the L1 DTLB; nested paging in a VM makes STLB misses and walks costlier (inference). | Random `P[p]` at L12493 | `dTLB-load-misses` / `ls_l1_d_tlb_miss.*`; rerun with THP-backed A (`madvise(MADV_HUGEPAGE)`) on both CPUs. |
| H8 | **The fallback may trigger on some blocks** (see 5.1). It is a serial `decode_1` with no lanes or prefetch, and it would hurt the higher-latency machine most. | L12864-12870, L13001-13008, L12278-12281 | Look for samples in `libsais_unbwt_decode` or `calculate_fastbits` in `perf record`, or set a gdb breakpoint, over a full bench round on the EPYC. |
