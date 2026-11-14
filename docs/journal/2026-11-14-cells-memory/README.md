# Cells memory census, 14 November 2026: where the Cells kernel touches memory

An untimed census of the addresses the Cells kernel (src/cells.rs) loads and stores, on every
block of the pinned Wikidata corpus. The Segmented kernel (the old run-records path:
`runs::fill`'s T-vector table plus the backward room walk) is the baseline, on the same blocks.
This is research. It changes nothing on master and nothing in a production build.

Why: on the i9, forced Workers + Cells stop scaling at about 6 threads. The first suspect was
Cells' ~15 MB per-worker buffer (relay #145 6879411570; LOG 6879443839). That figure is
reserved, reused high-water capacity, not the bytes a block moves. This census measures
the bytes and lines each block actually touches, so a later change can aim at real traffic.
**It makes no claim about what causes the i9's scaling.** That needs hardware counters on the
workstation.

All figures cover the 196 blocks Cells admits (n = 176,398,965 output bytes), pooled: summed
over the blocks, then divided by Σn. A "line" is a distinct 64-byte line touched in one
attempt, summed over attempts. "Heap" means the kernel's own buffers, excluding the stack
and the kernel struct ("other").

## Results

### 1. Per output byte: Cells against the old path (same 196 blocks)

| per output byte | Cells | Segmented (old path) |
|---|---:|---:|
| distinct lines touched | **0.1025** (6.56 B) | **0.0974** (6.24 B) |
| bytes written, all / heap | 17.16 / **13.79** | 14.09 / **7.71** |
| bytes read, all / heap | 18.31 / **11.27** | 14.14 / **5.25** |
| modelled misses, LRU 32 KiB | 0.440 | 1.149 |
| modelled misses, LRU 256 KiB | 0.333 | 1.063 |
| modelled misses, LRU 1 MiB | 0.208 | 0.839 |
| modelled misses, LRU 2 MiB | **0.144** | **0.510** |
| modelled misses, LRU 32 MiB (= cold lines) | 0.1025 | 0.0974 |

Both kernels touch about the same footprint per output byte: Cells' is 5% larger. Cells
moves about twice the heap bytes, but its accesses are far more local. In a fully
associative 2 MiB LRU model it misses 0.144 lines per output byte against 0.510, which is
9.2 B of line traffic against 32.7 B. 0.1025 of Cells' 0.144 are first touches in the block
(cold), and only 0.041 are re-touches after an eviction-sized gap. For the old path, 0.396 of
0.510 are re-touches: the walk's random reads of its 3.6 MB table.

Without the first admitted block (no buffer growth), the figures agree to three digits
(`cells_steady`, `segmented_steady` in summary.json).

### 2. Which phase and region dominate (Cells)

Lines touched per output byte. A line touched by several phases counts in each.

| phase | lines | heap bytes read | heap bytes written | modelled 2 MiB misses |
|---|---:|---:|---:|---:|
| `slots` | 0.049 | 0.40 | 3.02 | 0.050 (0.049 cold) |
| `induce_cells` | **0.082** | **7.70** | **8.20** | **0.069** (0.035 cold) |
| `write_cells` | 0.031 | 3.17 | 2.57 | 0.024 (0.018 cold) |

The stack also takes 3.5 B read and 2.6 B written per output byte in the induction, and
2.2 B read in the writer's expansion. These are register spills, about 5 lines in all, always
L1-resident. They count instructions, not lines.

| region | distinct lines / out byte | share | modelled 2 MiB misses | its lines' phases |
|---|---:|---:|---:|---|
| `buf.bots` (bottom order's slots) | 0.0360 | 35% | 0.054 | 36% written by `slots` and read once by the induction; 34% written and read in the induction; 29% both |
| `buf.tops` (top order's slots) | 0.0311 | 30% | 0.047 | 61% written by `slots` and the induction, then read; 24% induction only; 15% `slots` then one read |
| `out` | 0.0156 | 15% | 0.016 | 74% written by the fill and then the expansion, never read; 14% also read as copy sources |
| `buf.rules` | 0.0090 | 9% | 0.012 | written by the induction, read in the writer's two passes (`descend`, then `expand`) |
| `buf.arena` (words kept as bytes) | 0.0064 | 6% | 0.010 | written by the induction; read by it (83%) and by `expand` (57%) |
| `pos` | 0.0022 | 2% | 0.002 | NIL-filled, then written and read in `descend`, read in `expand` |
| `recs` (run records, input) | 0.0021 | 2% | 0.003 | read twice by `slots` (count pass, letters pass) |

**The two slot orders are 65% of Cells' distinct lines and 70% of its modelled 2 MiB
misses.** By phase, the induction dominates bytes (16 of Cells' 25 heap bytes per output byte)
and re-touches. `slots` dominates cold writes: 0.047 lines per output byte go to the slots
for the first time.

**Repeated passes** (re-touch with LRU distance of 2 MiB or more, lines per output byte):

| Cells | | Segmented | |
|---|---:|---|---:|
| `induce_cells`, `buf.bots` | 0.0177 | `seg.walk`, `tt` | **0.396** |
| `induce_cells`, `buf.tops` | 0.0157 | `seg.assemble`, `seg.bufs` | 0.0157 |
| `write.expand`, `buf.arena` | 0.0033 | | |
| `write.descend`, `buf.rules` | 0.0031 | | |
| `slots.letters`, `recs` (second pass) | 0.0007 | | |
| **total** | **0.041** | **total** | **0.413** |

Regions touched by more than one phase: `buf.tops` and `buf.bots` (`slots` then the
induction), `buf.rules` (the induction, `descend`, `expand`), `buf.arena` (the induction,
`expand`), `pos` (fill, `descend`, `expand`), `out` (fill, `expand`), `recs` (two `slots`
passes). In the old path, `tt` is written by `fill` and read by `walk`, and `seg.bufs` is
written by `walk` and read by `assemble`.

### 3. Capacity against touched bytes (Cells' worker, medians per admitted block)

| buffer | reserved by this block | capacity after it | touched (lines × 64) |
|---|---:|---:|---:|
| `buf` pad | 4,096 | 4,096 | 256 |
| `buf` top slots | 4,883,520 | 4,883,520 | 1,733,568 (35%) |
| `buf` bottom slots | 4,883,520 | 4,883,520 | 2,004,992 (41%) |
| `buf` byte arena | 2,304,320 | 2,304,320 | 351,360 (15%) |
| `buf` rules | 2,304,016 | 2,304,016 | 513,024 (22%) |
| `buf` past this block's layout (earlier high-water) | — | 981,024 | 0 |
| `pos` | 128,244 | 160,832 | 128,256 |
| output `Vec` (`Scratch.ll`) | 900,000 | 1,799,926 | 900,096 |
| run records (input) | 118,956 | 118,956 | 118,976 |
| fallback `Segmented` table | — | 3,600,032 | 0 |
| fallback `Segmented` segment scratch | — | 4,390,189 | 0 |

`buf` alone holds about 15.4 MB, the ~15 MB of the hypothesis. Pooled per output byte, Cells'
worker reserves 18.3 B for a block. With high-water slack, the output's doubling and the
fallback's buffers, it holds 27.3 B of capacity, and it touches 6.56 B. The old path reserves
6.16 B, holds 17.5 B and touches 6.20 B. So the large arena is mostly untouched reservation,
sized for the cell budget (0.16 n cells, with about 0.10 n used), and the bytes touched per
block are close to the old path's. Kernel page zeroing (first touch of a 4 KiB page in the
process) happens only when a buffer grows: 3,380 pages for Cells' regions over the whole run,
none recurring.

### 4. The writer's long copies: is the source near?

0.797 of each output byte comes from long copies of earlier output, in 2,122,787 copies with a
mean length of 66 B. The other 0.203 is short factors from the arena. No copy overlaps its
own destination.

- **By byte distance back (d − f):** 0.4% of copy bytes come from under 512 B back, 10% from
  under 4 KiB, 30.6% from under 32 KiB, 57% from under 256 KiB, 78% from under 512 KiB,
  and all from under 1 MiB (a block is at most 900 kB).
- **By LRU distance** (the copy source loads in `write.expand`, against the whole address
  stream): 94.6% of these line accesses have a stack distance under 32 KiB, 97.2% under
  256 KiB, 99.8% under 1 MiB and 100% under 2 MiB. None is a first touch. These are line
  accesses, so a 66 B copy's repeated, overlapping loads of the same source line count each
  time, which inflates the near share. The bound holds anyway: **no copy source is
  further than 2 MiB of distinct lines back.** In this model the writer's copies are L2-near,
  not memory-far.

### 5. Cheap changes this points at (estimates from these counts, not measured)

- **Slot size (24 → 20 or 16 bytes).** Bytes 16..24 of a slot hold the other order's slot
  offset in 8 bytes. It is always below the allocation, which is checked to be below 2³², and
  `ahead!` already reads it as a u32. A 20-byte slot would cut the slot regions' lines by
  about 1/6: roughly 0.011 lines per output byte, **≈11% of Cells' distinct lines and ≈12% of
  its modelled 2 MiB misses**. A 16-byte slot would need the lengths packed (n < 2²⁴) and
  would cut about 1/3: **≈22% of lines, ≈23% of 2 MiB misses**. This assumes the lines
  touched scale with slot bytes. That holds roughly, because the orders are walked from
  their ends and fresh slots are appended contiguously. It is the biggest lever these counts
  show.
- **The output's zero fill (`out.resize(n + 32, 0)` in `write_cells`).** The comment says only
  the slack is written. But `Scratch.ll` is shared with the other paths, which leave it shorter
  (e.g. `assemble_reversed` clears it), and 178 of the 196 admitted blocks follow a block that
  Cells did not admit. So 173 blocks zero more than half their output first: 0.88 B written per
  output byte, 6.4% of Cells' heap bytes written. Skipping it saves those bytes but **no
  distinct lines**: every output line is written by `expand` anyway. In the LRU model the
  misses only move from the fill to `expand`. This is a small, bytes-only win. The `pos` NIL
  fill adds another 0.15 B.
- **Slots written by `slots`, then read once.** 36% of the bottom order's lines and 15% of the
  top's are written by `slots` and read exactly once, by the induction. These are the 0.047
  cold lines per output byte (a third of the modelled 2 MiB misses). Removing them would mean
  the induction reading the run records directly for the original letters. That is a design
  change, not a cheap one.
- **The reservation.** Reserved but untouched bytes are 2.8× the touched bytes. Shrinking the
  reservation changes no traffic in this census: the untouched pages are never read or
  written after the first growth. It could only matter for RSS, page tables or TLB reach,
  which this census does not measure.
- **The rules' second pass.** `descend` reads the rules in reverse after the induction, and a
  third of those reads are re-touches from more than 2 MiB back (0.003 lines per output byte).
  This is small.

## Method

**The instrument: a minimal valgrind tool, `memtrace`** (memtrace/mt_main.c, built out of tree
against the system valgrind 3.22.0's headers and static libraries by memtrace/build.sh). It
instruments every guest load and store: plain, guarded, CAS, LL/SC, and the dirty helpers'
declared memory effects. While tracing is on, it writes each access as one 64-bit record
(address, size, load or store) to a FIFO. Valgrind client requests from the kernels (feature
`memtrace`, src/memtrace.rs) turn tracing on and off, and put markers in the same ordered
stream:

- BEGIN (block, n, records);
- PHASE;
- REGION (each buffer's base, bytes reserved by this block, capacity);
- END (outcome, and a stack address).

Only the kernels' attempts are traced. The Huffman/MTF front, `unrle` and the CRC are not
traced: they are the same for both paths.

- Cells phases: `slots` (`count`: the record pass and gates; `alloc`; `letters`), the
  induction, and `write_cells` (`fill`: `out.resize` and `pos`'s NIL fill; `place`; `spine`;
  `descend`; `expand`; `respine`; `repeat`).
- `buf` is split at the layout's offsets: pad, top slots, bottom slots, arena, rules, and the
  spare bytes beyond this block's layout.
- Segmented phases: `fill` (`runs::fill`), `walk` (`room_segments`, or `walk_back` for a short
  block) and `assemble` (`assemble_reversed`).

**The driver** (examples/cells_memory.rs) decodes every block on one thread with
`decode_block`, as `decode` does, with its CRC checked. It runs once with `Scratch<Cells>` (the
Cells worker, its fallback included) and once with `Scratch<Segmented>`. A global mode makes
only that kernel's attempts emit markers. In Cells mode it also writes each admitted block's
long copies by distance back, from the census record.

**The consumer** (memtrace/consumer.rs) counts, per attempt and per (phase, region, load or
store):

- bytes and accesses;
- distinct 64-byte lines (an access is split over the lines it covers);
- the exact LRU stack distance of every line access (a Fenwick tree over the attempt's
  line-access clock). A repeat of the immediately preceding line is distance 0.

Per region it also counts each line's phase pattern and 4 KiB pages first touched in the
process. The region of an access is found by its first byte. summarise.py pools the counts.
A *modelled miss* at capacity C is a cold access, or one with distance ≥ C: a fully
associative LRU cache, per attempt, starting empty.

**Cross-check** (crosscheck.py, crosscheck.txt, cachegrind.out.cells): the same Cells run
under cachegrind (D1 48 KiB 12-way, LL 2 MiB 16-way, 64 B lines), compared per function:

- `induce_cells`: memtrace's loads and stores equal cachegrind's Dr and Dw to within 1,200 of
  273 M and 243 M. The difference is the markers' own stack stores.
- `slots`: loads agree to 1,354. memtrace has 11.3 M more stores. These are the
  read-modify-write increments of the `cnt` table: cachegrind counts a modify as a read only,
  and memtrace counts a load and a store.
- `write_cells`: memtrace's counts include its memmove/memset callees, which cachegrind
  reports as separate functions.
- The set-associative LL misses agree with the LRU 2 MiB model where comparable:
  - `induce_cells`: 12.40 M (cachegrind) against 12.2 M (model);
  - `slots`: 8.57 M (cachegrind) against 8.6 M cold lines (model).

## Limits

- **Address-level counts, not hardware traffic.** The cache figures come from a model: one
  core, fully associative LRU, cold at each block's start, with no prefetchers, no
  write-back/write-allocate distinction, and no other threads sharing the LL or memory
  bandwidth. They count which lines must be fetched in principle, not what the i9 fetches. The
  set-associative cachegrind run agrees with the model on the dominant phases, but neither is
  a hardware counter.
- **Cross-block reuse is not modelled.** Each attempt starts cold, so a buffer reused from
  the previous block counts as cold even if it is still cached. A Workers worker's
  consecutive blocks share these buffers.
- **Access counts for `rep stos`/`rep movs` are per byte** (valgrind executes each iteration
  as an access). Bytes and lines are right; access counts in `write.fill` are not
  instruction counts. This is why the report uses bytes and lines, not accesses.
- **Kernel page zeroing** is seen only as a page's first touch in the process. Unmapping and
  remapping by the allocator, transparent huge pages and the TLB are not seen.
- **Stack and struct traffic** ("stack", "other") is counted but excluded from the heap
  figures. It is a few L1-resident lines.
- **One corpus.** Generated text and pattern corpora have no admitted block (2026-11-14
  census), so they have no Cells figures.
- **Timing.** These counts carry no timing and support no claim about time or scaling.
- **The traced binary.** It was built before `cargo fmt` reformatted src/memtrace.rs and
  examples/cells_memory.rs (whitespace only). Hashes are in inputs-binaries.sha256. The
  feature `memtrace` implies `census`. The 2026-11-14 census binary rebuilt from this branch
  writes byte-identical `census-wikidata.jsonl` (SHA-256 28e3a629…).

## Files

- `trace-cells.jsonl`, `trace-segmented.jsonl`: the consumer's records, one per kernel attempt
  (383 each: 196 Cells admissions, 187 refusals; Segmented on the same 383 run-record blocks).
- `blocks-cells.jsonl`, `blocks-segmented.jsonl`: the driver's records, one per block (431).
  The two runs decoded the same lengths.
- `summary.json`: `python3 summarise.py .`, all the figures above and more (per phase ×
  region, line patterns, LRU buckets).
- `run.sh`: the whole run. `memtrace/`: the tool, its build, and the consumer.
- `crosscheck.py`, `crosscheck.txt`, `cachegrind.out.cells`, `cachegrind-cells.log`: the
  cachegrind cross-check.
- `inputs-binaries.sha256`: the corpus and binaries. `SHA256SUMS`: this directory's files.

Reproduce: build the corpus as docs/journal/2026-11-14-cells-census/ does
(`bench/wikidata.py` from the `bench-data` windows), then
`bash docs/journal/2026-11-14-cells-memory/run.sh target/census/wikidata.bz2 WORKDIR`. This
takes about 4 minutes on one core, plus about 20 s for the cachegrind cross-check.
