# 2026-11-10: The chunked link: a 1.3 to 1.4 MB table with one kind of step, and four ideas bounded by arithmetic

**Status: a table checked row by row against the T-vector, its size counted, its fill and its
step counted and timed in common loops (local, indicative, RULES C5). For the Optimiser's
price (SPEC.PATHFINDER 3a). Nothing in the decoder changes and no build slot is asked.**

- **The question:** share-nothing needs a table near 1 MB a worker on text as well as on
  Wikidata. The k-step table is small only where runs are long; the line table (E) died on its
  overflow path and its flags. Is there a small table with one rule for every row?
- **Found:** the links of one bucket's rows are increasing, so their upper bits change rarely.
  Keep the low byte a row and the upper bits once a stretch.
- **Counted:** 1.46 bytes a row on Wikidata (1.31 MB a block), 1.60 on text (1.44 MB), with
  the start bit a row the room walk needs (corrected, §0). The
  T-vector is 4 (3.60 MB). Every link and byte checked on 25 blocks.
- **Timed, the same loop for both tables:** one lane 0.98 to 1.02 of the T-vector's time;
  32 prefetching lanes 1.30 to 1.50. Instructions a row: 22.2 against 8.2.
- **Its fill, built and checked:** 6.0 instructions a row on Wikidata against `build`'s 5.06,
  17.6 on text against 11.5; 1.45 and 1.93 times `build`'s time. The model said `build` + 2.
- **By the law of the knee:** 2,000 to 3,120 MB/s of inversion at ten plain workers. Three
  teams measured 2,430 to 2,584. Ahead on Wikidata, level on text as built. What decides is
  its one-block cost on the i9, which no local run gives, and a complete kernel, which this
  is not.
- **Bounded by arithmetic, no code:** inversion by induction on intervals, tunnels learned on
  the fly, staggered walks, and rank-or-select tables.

Labels: **claim** (proved here), **counted** (exact for its blocks), **measured** (local, one
container, indicative), **model**.

## 0. Correction (2026-11-10, the Optimiser's REVIEW, #145 6759415547)

- **The size was a layout's that was not the one timed.** The tightened step reads a 16-byte
  directory entry a 64 rows; the size still charged 12. And the room walk's segment starts
  need a bit a row that this table has no spare bit for: 0.125 bytes a row more. As timed the
  table is 1.33 bytes a row on Wikidata and 1.47 on text (1.20 and 1.32 MB); with the start
  bits, 1.46 and 1.60 (**1.31 and 1.44 MB**). "Under 1.3 MB" was wrong. `chunk_table` now
  charges what it allocates.
- **The kernel is not complete.** The fill (§3) is from the last column and counts, with its
  packing, counts and validation; it sets no starts. The walk's loop has its starts from an
  oracle walk, equal slices, no segment ends and no assembly. The complete kernel is the next
  falsifier, and it is not built.
- **The law recomputed at the true size** (§4): text needs g of 1.62 to 1.75 or less; it has
  1.69 in these loops.

## 1. The table (claim, proved; checked on every row)

Row y of the T-vector links to Ψ(y). Rows are in first-column order, and the rows of one byte
(a *bucket*) are its copies in the last column, in order. So within a bucket Ψ is increasing.

- **A stretch:** consecutive rows of one bucket whose links lie in one chunk of 256 rows. They
  share every bit of the link above the low byte.
- **Their number is a count of the last column:** a (bucket, chunk) pair is a byte that occurs
  in a chunk. Stretches = the distinct bytes of each aligned 256-row chunk of L, summed.
- **The table:**
  - `low[y]`: the link's low byte. One byte a row.
  - `dir`: one bit a row, set where a stretch begins, and beside each word of 64 the set
    bits before it.
  - `rec[k]`: stretch k's upper bits and its bucket's byte.
- **A step:** k = (the count before y's word) + popcount(its bits up to y) − 1; Ψ(y) is
  rec[k]'s upper bits with low[y]; the byte is rec[k]'s. It is the first column's byte of y,
  which is the T-vector's byte one step on: the same output from the origin.
- **One rule.** No row has a second access of another kind, no line overflows, nothing is
  flagged. E's overflow list and its flagged skeleton have no counterpart.

**Counted** (`examples/line_census.rs`, stretches a row by chunk size; `examples/chunk_table.rs`
for the bytes):

| chunk, rows | 16 | 64 | 256 | 1,024 | 4,096 |
|---|--:|--:|--:|--:|--:|
| Wikidata | 0.110 | 0.046 | 0.0205 | 0.0094 | 0.0043 |
| text | 0.223 | 0.114 | 0.0548 | 0.0247 | 0.0104 |
| records | 0.977 | 0.907 | 0.628 | 0.249 | 0.063 |

| at 256 rows, 4-byte records | bytes a row, as timed | with a start bit a row | a 900,000-row block |
|---|--:|--:|--:|
| Wikidata | 1.332 | 1.457 | 1.31 MB |
| text | 1.470 | 1.595 | 1.44 MB |
| records | 3.764 | 3.889 | 3.50 MB |

- **Records keeps the T-vector.** The choice is made a block, from the stretch count.
- **The floor of this form is the low byte and the directory:** 1.24 MB a block. A 4-bit low
  part with 16-row chunks is 1.32 bytes a row on Wikidata (less) and 1.77 on text (more).

## 2. The step in a common loop (measured; instructions counted)

`chunk_table`: both tables walked by the same two loops. One lane; and 32 lanes in turn, each
prefetching its next row's lines, with starts taken from the true walk and equal slices. The
loop is simpler than the room walk (no flags, no segment ends), and the same for both, so the
difference is the step's.

| | T-vector | chunked | ratio |
|---|--:|--:|--:|
| instructions a row, one lane (callgrind, text) | 3.4 | 14.0 | |
| instructions a row, 32 lanes | 8.2 | 22.2 | |
| ns a row, one lane, Wikidata | 14.4 | 14.7 | 1.02 |
| ns a row, one lane, text | 14.5 | 14.2 | 0.98 |
| ns a row, 32 lanes, Wikidata | 1.68 | 2.19 | 1.30 |
| ns a row, 32 lanes, text | 1.52 | 2.28 | 1.50 |

- **Compiled with popcnt.** Without it the count is a routine of shifts and masks: 32.0 and
  39.3 instructions a row, and 2.3 to 2.5 times the T-vector's time. The first run was that.
- **Two cuts.** The first kept the bits and their counts in two arrays: 18.0 and 25.2
  instructions a row, 1.58 to 1.75 of the T-vector's time at 32 lanes. With one directory
  entry a word (one load, the count stored less one) it is the table above.
- **The step is 14 instructions over the T-vector's in the laned loop.** On the room walk's
  10.41 that would be about 24 a row (model). E's was 48 as built.
- **Not done:** the stretch record is not prefetched (its index is known only after the
  count); a second stage would.
- **This container's caches hold the T-vector.** The ratio here is an instruction ratio. On
  the i9 the T-vector's walk waits on L3; the chunked walk's `low` (0.9 MB) does too, and its
  directory and `rec` (0.2 to 0.4 MB) contend for a 256 KB L2. Its loads a row there
  are between 1 and 3. Unknown.

## 3. The fill (built and checked; counted and measured)

Two fills in `chunk_table`, each equal to the definition's table on every block (25 blocks:
low bytes, directory and records compared whole).

- **By rows:** for row i with byte c, y = cursor[c]++; low[y] = i's low byte; if i's chunk is
  not the last chunk c was seen in, set bit y and append a record to c's span.
- **By runs:** a run of the column is consecutive rows of its bucket whose low bytes count up,
  written 16 at a time. It begins a stretch where its chunk is new to its byte, and one more
  at each chunk it crosses.
- Both then pack the spans in bucket order and sum the directory. A bucket's span is the lesser
  of its rows and the block's chunks, so the spans total at most n records and are touched only
  where written.

| a row | production's `build` | by rows | by runs |
|---|--:|--:|--:|
| Wikidata, instructions (callgrind, 2 blocks) | 5.06 | 17.3 | **6.0** |
| text, instructions (3 blocks) | 11.5 | **17.6** | 20.6 |
| Wikidata, ns (10 blocks) | 0.48 | 2.49 | **0.70** (1.45×) |
| text, ns (9 blocks) | 1.23 | 2.60 | **2.37** (1.93×) |

- **The model was "`build` + 2 instructions".** On Wikidata it is + 0.95 by runs: held. On text
  it is + 6.1 at best: failed. In time it failed on both: 1.45 and 1.93 times `build`.
- **Which failure:** the construction. `build` is tuned (it takes runs 16 at a time where they
  are few and rows otherwise); each of these fills is one loop. By rows the cursor is read and
  written through memory every row, a chain where a byte repeats. A fill that chose by run
  density as `build` does is not written.
- **One byte written a row** where the T-vector writes four, and it is not yet cheaper.
- The room walk's start flag needs a second bit a row in the directory (+0.125 bytes a row).

## 4. What the law says (model; **withdrawn as a decoder's price**)

**Withdrawn (2026-11-10, later).** The i9's share-nothing run shows the law does not transfer to
the whole decoder (`2026-11-10-hypothesis-register.md` §1a): it predicts nearly twice the
inflation ten workers show, and a scanner-fed small table is capped at 1.86 GB/s, 1.24 times
the default plan, before its own cost. The figures below are the inversion harness's and rank
no decoder. Kept as written.

Time a row = g + a · X(L), `2026-11-10-hypothesis-register.md` §1. L = 10 × 1.31 to 1.44 MB =
13.1 to 14.4 MB, so X is 0.40 to 0.49.

| g, the inversion at one block | a | MB/s at ten plain workers |
|--:|--:|--:|
| 1.34 (Wikidata here: fill by runs and the 32-lane walk, over `build` and T's walk) | 1 | 3,120 |
| 1.69 (text here, the same) | 1 | 2,500 |
| 2.0 (the i9's L2 misses bind) | 1.5 | 2,000 to 2,100 |

- **Against:** three teams of three 2,430 to 2,584 (measured, that harness); compact30, 1.95 MB
  at g = 1.18, 2,350 at seven (model); N1 at k = 4, 3,470 to 5,260 (model, unbuilt, long runs
  only).
- **It brackets the teams.** On text it beats them if g on the i9 is 1.62 to 1.75 or less; on
  Wikidata, 1.70 to 1.84. As built here it does on Wikidata, where D4-delta's 3,570 is better
  still, and on text, the class it is for, it is level with them.
- **What it has that the others lack:** one worker a block with no cooperation, on text and on
  Wikidata alike. Its fill is dearer than `build` as written (§3).
- **Next falsifier:** the complete kernel, counted: the fill with its starts, and a walker
  with segment ends, refills and assembly. Then, and only then, the i9.

## 5. Bounded by arithmetic

### 5a. Inversion by induction on intervals

- **True:** LF is an interval exchange: a translation on each of r runs. The first return to a
  sub-interval is again one, on r + 1 pieces at most. Induce down, and the text is a grammar's
  expansion: copies, no table loads. (For words whose BWT is perfectly clustered this is
  Ferenczi and Zamboni, 2013; a run-length grammar of size O(r log(n/r)) from the runs is
  Gagie, Navarro and Prezza, 2020. Nothing claimed but the use.)
- **Bound:** every level of the induction has at least one tower level a piece, so the work is
  at least r interval steps a level, and each step finds its run by position. With pieces of
  equal length it is 2r a halving, over log₂(n/r) halvings: 0.55 n on Wikidata, more than n on
  text. An interval step is no cheaper than the line table's row step.
- **Pays only below r/n of about 0.03.** Wikidata is 0.076, text 0.30. Not this corpus.

### 5b. Tunnels learned on the fly, with no cut structure

- **True:** a lane can carry the interval of rows that has stayed with it. Within a bucket Ψ is
  increasing, so rows lo and hi are still together exactly when their links differ by hi − lo:
  one more load a step certifies it.
- **Bound:** every step not copied is then a learning step of about twice the room walk's
  instructions, and every lane must test for a record at each landing. At c = 0.6 that is the
  room walk's instruction count for 0.4 of its loads. E's flagged skeleton says the lanes
  would not hold that. Not handed.

### 5c. Staggering the workers' walks

- **The idea:** L3 holds four or five tables. Let only that many workers walk at once and the
  rest scan and decode symbols ahead.
- **Bound:** the walk and build are 73% of a Wikidata decode. Four or five walkers at 8.77
  cycles a byte are 1.4 to 1.7 GB/s before the knee's penalty and under 1 GB/s with it,
  whatever the others do. The teams reach 1.3 to 1.5 by sharing a table three ways.

### 5d. Rank or select in the step

- **Wavelet matrix over L:** 1.1 MB, eight dependent ranks a row. g near 5.
- **Elias–Fano on each bucket's links:** 0.75 MB at its best, a select a row and a second path
  for rare bytes. E's step and E's flags again.
- **Low 16 bits of each row's rank with a 14 KB directory:** exact and one rule, but 3 bytes a
  row with L: compact30's size at best.

## 6. What this changes

| need | candidate | size, MB a block | state |
|---|---|--:|---|
| share-nothing on any data | the chunked link | 1.31 to 1.44 | table, fill and a bare walk checked; g 1.34 to 1.69 here; no complete kernel |
| share-nothing on any data | compact30 | 1.95 | built once; the Optimiser pricing a plain-worker arm |
| share-nothing on long runs | the k-step table at k = 4 | 0.79 to 0.90 | unbuilt; its cuts cost 0.11 LF evaluations a row (tunnels §0c) |
| fewer loads on long runs | tall-tunnel copies | +0.6 | certified; deferred on its economics, the family open (tunnels §0c) |

**Limits.** One container, an EPYC whose caches hold the T-vector: its ratios are instruction
ratios. `text` and `records` are the bench's synthetic corpora. The walk's loop is this
example's, not the room walk. The law is one run of 29 October on the inversion alone, and
its X holds building as well as walking (the Optimiser's objection, #145 6759278054).

## Reproduce

```text
cargo build --release --examples
target/release/examples/line_census FILE.bz2 [EVERY]             # §1: stretches a row
target/release/examples/chunk_table FILE.bz2 [EVERY] [ROUNDS]    # §1's bytes, §2's times
target/release/examples/chunk_table FILE.bz2 [EVERY] [ROUNDS] fill   # §3: checks and times
valgrind --tool=callgrind target/release/examples/chunk_table FILE.bz2 EVERY 1 fill
callgrind_annotate --inclusive=yes callgrind.out.* | grep chunk_table::   # §3's instructions;
                                                   # `build` is called twice a block there
valgrind --tool=callgrind --toggle-collect='chunk_table::walk_g_lanes' \
  target/release/examples/chunk_table text.bz2 12 1 g            # §2's instructions (t for T)
```
