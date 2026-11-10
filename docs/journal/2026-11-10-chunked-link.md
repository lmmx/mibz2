# 2026-11-10: The chunked link: a 1.2 MB table with one kind of step, and four ideas closed by arithmetic

**Status: a table checked row by row against the T-vector, its size counted, its step timed in a
common loop (local, indicative, RULES C5). For the Optimiser's price (SPEC.PATHFINDER 3a).
Nothing in the decoder changes and no build slot is asked.**

- **The question:** share-nothing needs a table near 1 MB a worker on text as well as on
  Wikidata. The k-step table is small only where runs are long; the line table (E) died on its
  overflow path and its flags. Is there a small table with one rule for every row?
- **Found:** the links of one bucket's rows are increasing, so their upper bits change rarely.
  Keep the low byte a row and the upper bits once a stretch.
- **Counted:** 1.27 bytes a row on Wikidata (1.14 MB a block), 1.41 on text (1.27 MB). The
  T-vector is 4 (3.60 MB). Every link and byte checked on 25 blocks.
- **Timed, the same loop for both tables:** one lane 0.96 to 1.04 of the T-vector's time;
  32 prefetching lanes 1.58 to 1.75. Instructions a row: 25.2 against 8.2.
- **By the law of the knee:** 2,200 to 3,300 MB/s of inversion at ten plain workers. Three
  teams measured 2,430 to 2,584. It brackets them. What decides is its one-block cost on the
  i9, which no local run gives.
- **Closed by arithmetic, no code:** inversion by induction on intervals, tunnels learned on
  the fly, staggered walks, and rank-or-select tables.

Labels: **claim** (proved here), **counted** (exact for its blocks), **measured** (local, one
container, indicative), **model**.

## 1. The table (claim, proved; checked on every row)

Row y of the T-vector links to Ψ(y). Rows are in first-column order, and the rows of one byte
(a *bucket*) are its copies in the last column, in order. So within a bucket Ψ is increasing.

- **A stretch:** consecutive rows of one bucket whose links lie in one chunk of 256 rows. They
  share every bit of the link above the low byte.
- **Their number is a count of the last column:** a (bucket, chunk) pair is a byte that occurs
  in a chunk. Stretches = the distinct bytes of each aligned 256-row chunk of L, summed.
- **The table:**
  - `low[y]`: the link's low byte. One byte a row.
  - `bits`: one bit a row, set where a stretch begins; `base`: the set bits before each word.
  - `rec[k]`: stretch k's upper bits and its bucket's byte.
- **A step:** k = base[y / 64] + popcount(bits[y / 64] up to y) − 1; Ψ(y) = rec[k]'s upper
  bits with low[y]; the byte is rec[k]'s. It is the first column's byte of y, which is the
  T-vector's byte one step on: the same output from the origin.
- **One rule.** No row has a second access of another kind, no line overflows, nothing is
  flagged. E's overflow list and its flagged skeleton have no counterpart.

**Counted** (`examples/line_census.rs`, stretches a row by chunk size; `examples/chunk_table.rs`
for the bytes):

| chunk, rows | 16 | 64 | 256 | 1,024 | 4,096 |
|---|--:|--:|--:|--:|--:|
| Wikidata | 0.110 | 0.046 | 0.0205 | 0.0094 | 0.0043 |
| text | 0.223 | 0.114 | 0.0548 | 0.0247 | 0.0104 |
| records | 0.977 | 0.907 | 0.628 | 0.249 | 0.063 |

| at 256 rows, 4-byte records | bytes a row | a 900,000-row block |
|---|--:|--:|
| Wikidata | 1.270 | 1.14 MB |
| text | 1.408 | 1.27 MB |
| records | 3.701 | 3.33 MB |

- **Records keeps the T-vector.** The choice is made a block, from the stretch count.
- **The floor of this form is the low byte:** 0.9 MB a block. A 4-bit low part with 16-row
  chunks is 1.13 bytes a row on Wikidata and 1.58 on text: no better.

## 2. The step in a common loop (measured; instructions counted)

`chunk_table`: both tables walked by the same two loops. One lane; and 32 lanes in turn, each
prefetching its next row's lines, with starts taken from the true walk and equal slices. The
loop is simpler than the room walk (no flags, no segment ends), and the same for both, so the
difference is the step's.

| | T-vector | chunked | ratio |
|---|--:|--:|--:|
| instructions a row, one lane (callgrind, text) | 3.4 | 18.0 | |
| instructions a row, 32 lanes | 8.2 | 25.2 | |
| ns a row, one lane, Wikidata | 13.2 | 13.8 | 1.04 |
| ns a row, one lane, text | 14.4 | 13.7 | 0.96 |
| ns a row, 32 lanes, Wikidata | 1.61 | 2.54 | 1.58 |
| ns a row, 32 lanes, text | 1.53 | 2.68 | 1.75 |

- **Compiled with popcnt.** Without it the count is a routine of shifts and masks: 32.0 and
  39.3 instructions a row, and 2.3 to 2.5 times the T-vector's time. The first run was that.
- **The step is 17 instructions over the T-vector's in the laned loop.** On the room walk's
  10.41 that would be about 27 a row (model). E's was 48 as built.
- **A first cut.** Three separate arrays, the stretch record not prefetched (its index is
  known only after the count). Interleaving `bits` with `base` and a second stage would trim
  it; not done.
- **This container's caches hold the T-vector.** The ratio here is an instruction ratio. On
  the i9 the T-vector's walk waits on L3; the chunked walk's `low` (0.9 MB) does too, and its
  `bits`, `base` and `rec` (0.2 to 0.4 MB) contend for a 256 KB L2. Its loads a row there
  are between 1 and 3. Unknown.

## 3. The fill (derived; not built)

One pass over the last column, as the T-vector's: for row i with byte c, y = cursor[c]++;
low[y] = i's low byte; if i's chunk is not the last chunk c was seen in, set bit y and append a
record to c's list. Then the lists are packed in bucket order and `base` is summed.
- one byte written a row where the T-vector writes four;
- about two instructions a row over `build`'s 4.92 (model);
- the room walk's start flag needs a second bit a row beside `bits` (+0.125 bytes a row).

## 4. What the law says (model)

Time a row = g + a · X(L), `2026-11-10-hypothesis-register.md` §1. L = 10 × 1.14 to 1.27 MB =
11.4 to 12.7 MB, so X is 0.29 to 0.38.

| g, the inversion at one block | a | MB/s at ten plain workers |
|--:|--:|--:|
| 1.3 (tuned, loads do not bind) | 1 | 3,300 |
| 1.5 (this container's ratio, with `build`'s cost unchanged) | 1 | 3,000 |
| 2.0 (the i9's L2 misses bind) | 1.5 | 2,200 |

- **Against:** three teams of three 2,430 to 2,584 (measured, that harness); compact30, 1.95 MB
  at g = 1.18, 2,350 at seven (model); N1 at k = 4, 3,470 to 5,260 (model, unbuilt, long runs
  only).
- **It brackets the teams.** It beats them if g on the i9 is about 1.8 or less.
- **What it has that the others lack:** one worker a block with no cooperation, on text and on
  Wikidata alike, from a fill no dearer than `build`.
- **Cheapest decider:** `chunk_table` on the i9, one block and ten at once. A C5 run; the
  Optimiser's price first.

## 5. Closed by arithmetic

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
| share-nothing on any data | the chunked link | 1.14 to 1.27 | counted and checked; g on the i9 unknown |
| share-nothing on any data | compact30 | 1.95 | built once; the Optimiser pricing a plain-worker arm |
| share-nothing on long runs | the k-step table at k = 4 | 0.79 to 0.90 | unbuilt; its cuts cost 0.11 LF evaluations a row (tunnels §0c) |
| fewer loads on long runs | tall-tunnel copies | +0.6 | certified, and closed as a build (tunnels §0c) |

**Limits.** One container, an EPYC whose caches hold the T-vector: its ratios are instruction
ratios. `text` and `records` are the bench's synthetic corpora. The fill is not built. The law
is one run of 29 October on the inversion alone.

## Reproduce

```text
cargo build --release --examples
target/release/examples/line_census FILE.bz2 [EVERY]             # §1: stretches a row
target/release/examples/chunk_table FILE.bz2 [EVERY] [ROUNDS]    # §1's bytes, §2's times
valgrind --tool=callgrind --toggle-collect='chunk_table::walk_g_lanes' \
  target/release/examples/chunk_table text.bz2 12 1 g            # §2's instructions (t for T)
```
