# 2026-11-10: D4-delta, a counting probe: registration before any count

**Status: registered before the first instruction count. Counting only: no timing of any kind
is taken or used. Research only: the decoder never calls the table, and nothing in production
changes.**

- **What is probed:** design D at k = 4 (Pathfinder, `2026-11-10-backward-table.md` §4a on
  #313), in the Optimiser's reduction (REVIEW 6757974415 on #145, report at `d4c4b41c`):
  within a piece of LF⁴, LF⁴(i) = base + i − first, so a record holds one
  delta = (base − first) mod 2²⁰, and the next row is (i + delta) & (2²⁰ − 1).
- **Why a count first:** design E (#323) was falsified at 3.67× production's inversion. Its
  bare step had been estimated at 16 instructions; with its lane bookkeeping it was 48 a row.
  This probe counts the **inclusive** loop: lanes, setup, tails, directory lookups,
  validation, splits and the output's assembly.
- **What a pass earns:** consideration of a later, separately registered timing. It
  qualifies nothing, and authorises no timing and no production slot.

Labels: **checked** (bytes equal), **counted** (callgrind instructions, exact for the input),
**model** (derived, not observed).

## 1. What is built (before counting)

`src/d4_delta.rs`, a `#[doc(hidden)]` research module the decoder never calls, and
`examples/d4_delta.rs` (check, census, count).

**The record,** one `u64` a piece:

| bits | field |
|---|---|
| 0–19 | delta, (LFʲ(first) − first) mod 2²⁰, j the record's length |
| 20–51 | the four bytes L[i], L[LF i], L[LF² i], L[LF³ i], in the order the walk emits them |
| 52–54 | length j, the bytes the record emits (1 to 4) |
| 55 | short: j < 4 (the row's path reaches a stop in fewer than four steps) |
| 56 | start: the row is a segment's start (a lane arriving here stops before emitting) |

Since the delta is the low 20 bits, next = (i + record) & (2²⁰ − 1) with no field extraction.

**The directory,** 12 bytes a 64 rows (structure of arrays): a `u64` with a bit at each
record's first row, and a `u32` of the records before the word, minus one. A step's record is
`rank[i >> 6] + popcnt(bits[i >> 6] << (63 − (i & 63)))`.
- **A disclosed choice against the Optimiser's 20 bytes:** the segment-start flags ride in the
  records (bit 56), not as a third directory word. A start row is then a singleton record of
  its own (at most two extra records a start), and a step makes one flag test, of a word it
  loads anyway, instead of a third load. The 20-byte directory's size is reported beside it.

**Stops and singletons.** The starts must be known when the table is built, since the
records carry them.
- Y = 511 pseudo-random rows (`bwt::random_rows`, as the room walk draws them) and `orig_ptr`.
- The proper stops are M = {LF³(y) : y ∈ Y}. Rows y, LF(y), LF²(y) become singletons: each
  stops at the first stop its path reaches in one to three steps, with that LFʲ's delta and
  its first j bytes (the "shortened j-step record").
- The segment starts are M ∪ {orig_ptr}. Each is a singleton flagged start.
- **Why `orig_ptr` need not be a proper stop:** a segment is the walk from its start to the
  first flagged row it lands on. A lane that steps over an unproper start lands later on a
  proper one, so every segment is still a correct fragment of the walk, and joining fragments
  end to start from `orig_ptr`'s, truncated at n bytes, is the walk. The proper stop
  LF³(`orig_ptr`) puts a stop on `orig_ptr`'s own cycle, so every walked segment ends.
- Choosing Y and taking M as its images (rather than choosing M and finding LF⁻ʲ(m)) needs
  only forward LF, which the level-2 pieces give through their directory. LF is a
  permutation, so M is as random as Y.
- **The split reserve:** at most 2 records for each of the 3·|Y| singletons and each of the
  |M ∪ {orig_ptr}| starts: 2·(3·512 + 513) = 4,098 records, 32,784 bytes.

**The build,** from the column `ll` and its counts:
1. **Runs** of `ll`, a word at a time: the level-1 pieces (first row, LF base, byte). The
   bases come from one counter a byte. **Validation:** the counts sum to n < 2²⁰, and after
   the pass every byte's counter stands at its bucket's end (the tiling argument of
   `2026-11-10-backward-table.md` §1). Otherwise the build refuses and leaves no table.
2. **Level 2 = level 1 ∘ level 1:** each run's image [base, base + len) is cut by the runs it
   covers, found by one monotone cursor a byte. Each level-2 piece keeps its first row, its
   LF² base, its LF base and its 2 bytes.
3. **A directory over the level-2 pieces,** as the final one, and the chains y → LF³(y)
   through it.
4. **Level 4 = level 2 ∘ level 2:** each level-2 piece's LF² image is cut by the level-2
   pieces it covers, the first found through the directory, the rest in order. Each level-4
   piece goes straight to its record, merged with the sorted singletons (a split where one
   falls inside it), and its bit is set in the final directory.
5. **The final directory's ranks,** one pass over its words.

**The walk,** in the room walk's skeleton (`bwt::room_segments`): 32 lanes, 512 segments,
passes of `room` offsets, the bytes staged in a tile (64 offsets of 4 bytes a lane, 8 KiB),
a 4-byte store a lane an offset, and one combined test of the short and start bits a step.
Each lane prefetches the directory word of its next row. A marked lane is handed over in lane
order. When the queue is empty the remaining lanes finish one at a time (the tail). The
segments hold the bytes in the order the walk emits them, last to first, and the assembly
writes them reversed from the output's end, joined from `orig_ptr`'s segment, truncated at n.
Blocks under 2¹⁶ rows take a plain walk, as production's room walk does.

## 2. Correctness gates (before counting is reported)

- Every block of the bench corpora (`ab gen corpora 4`) and of the pinned Wikidata corpus
  (SHA-256 `82f65ace…`): the D4 walk's output equals `bwt::walk`'s, the segmented walk forced
  on every block and the plain walk too, and RLE1's output matches the block's CRC.
- `tests/d4_delta.rs` in CI: `corpora(1)` at levels 1 and 9, and a fuzz smoke as #323's
  (samples, crafted random columns with any `orig_ptr`, 1,000 mutated streams).
- Unit tests and Miri (`cargo +nightly miri test --lib -- miri_d4`) for every `unsafe`,
  on small columns with several cycles, short cycles and a buffer grown mid-segment.

## 3. The counts (method fixed here)

callgrind (`valgrind --tool=callgrind`, instructions only), `d4_delta count FILE 3`: the
first 3 blocks of each file, #323's method. Each measured routine is in an
`#[inline(never)]` function, and its **inclusive** instructions are divided by the blocks'
rows (one row is one output byte).

| function | what it holds |
|---|---|
| `count_t_build` | control: `bwt::build`, production's build |
| `count_t_walk` | control: `bwt::walk_segments_room`, production's room walk with its assembly |
| `count_d4_build` | the complete D4 construction: runs, validation, both doublings, the level-2 directory, the stops and chains, the singleton splits, records and final directory |
| `count_d4_walk` | the complete laned walk: setup, lanes, passes, tile, hand-overs, tail and assembly |

- **Files:** `corpora/wikidata.bz2` (pinned) and `corpora/text.bz2` (`ab gen corpora 4`).
- **Table bytes** (`d4_delta census`), per block: records × 8, the directory, and the split
  reserve, as the admitted capacity; the records actually used; and separately the doubling
  scratch (level-1 and level-2 arrays, the level-2 directory), at their allocated capacities.
  Reported for the counted blocks and for every block of each corpus.

## 4. The prospective filters (the Optimiser's, fixed before counting)

| filter | threshold |
|---|---|
| F1: inclusive walk | ≤ 15 instructions an output byte |
| F2: complete construction | ≤ 7.35 instructions a row |
| F3: admitted table capacity | ≤ 1.2 MB (1,200,000 bytes) a 900k block: records, directory and split reserve |

- **Decided on Wikidata's first 3 blocks,** the class the design is for and the Optimiser's
  registered blocks. Each filter passes or fails on its own. F3 must hold on each of the 3
  blocks.
- **Text is reported, and decides nothing.** A text block whose table exceeds 1.2 MB is not
  admitted, and its counts are descriptive. The fraction of each corpus's blocks admitted is
  reported.
- **The verdict:** PASSES THE COUNT if F1, F2 and F3 all pass; otherwise FAILS, naming each
  failed filter. A pass earns only a later timing registration.
- **No count is re-run to pass.** A correctness fix found after the first count is disclosed
  with both counts. A change made to lower a count after it has been seen is reported as post
  hoc and doesn't change the verdict.

## 5. Protocol

- No timing, local or CI, is taken or used as evidence.
- The result goes in `2026-11-10-d4-delta-count-result.md`, with the counts, the verdict per
  filter, what dominated each count, and the negative findings.

Base: master `9c6a6b07`.
