# 2026-11-10: D4-delta, the counting probe: FAILS on construction (F2), passes walk and size

**Result:** FAILS THE COUNT under `2026-11-10-d4-delta-count-registration.md` §4, on F2 alone.
On Wikidata's first 3 blocks the complete construction costs **14.45 instructions a row**,
against a filter of 7.35: almost twice it. The inclusive laned walk is **7.73 instructions an
output byte** (filter 15: passes), and the admitted table is at most **857,428 bytes** a block
(filter 1.2 MB: passes). No timing registration follows from this probe.

- **Source:** branch `research/d4-delta-count`, the counts taken at `dabe28e8` and reproduced to
  the instruction at the PR's head, `561665b61ea61ba8027e4e15068322d938ea0077` (#331, closed
  unmerged: the module, example, tests and Miri cases stay on that branch and are not in master).
- **The registration** (`e90213e5`) was committed and pushed before the module was written,
  and before any count.
- **Counted only.** No timing of any kind was taken or used.

Labels: **counted** (callgrind, exact for the input), **checked** (bytes equal), **post hoc**
(added after the counts, at the Coordinator's request or for attribution; decides nothing),
**model**.

## 1. The registered counts (counted)

callgrind, `d4_delta count FILE 3`: the inclusive instructions of each `#[inline(never)]`
function, over the first 3 blocks' rows.
- **Wikidata:** 1,799,936 rows. Its first block is a 13-row block (a window's stream end),
  then two full blocks.
- **Text:** 2,699,943 rows.

| function | Wikidata, Ir a row | text, Ir a row | filter (Wikidata) | verdict |
|---|--:|--:|---|---|
| `count_t_build`: production's build (control) | 4.92 | 11.13 | | |
| `count_t_walk`: production's room walk (control) | 10.45 | 10.56 | | |
| **`count_d4_walk`: the inclusive D4 walk** | **7.73** | 7.68 | F1 ≤ 15 Ir a byte | **passes** |
| **`count_d4_build`: the complete D4 construction** | **14.45** | 71.78 | F2 ≤ 7.35 Ir a row | **FAILS** (1.97×) |
| D4 inversion (build + walk) | 22.17 | 79.46 | | |
| production's inversion | 15.38 | 21.69 | | |
| D4 over production | 1.44× | 3.66× | | |

The controls reproduce #323's registered counts (build 4.92, room walk 10.41–10.45 on
Wikidata).

**F3, the admitted table** (records for the level-4 pieces, the 12-byte directory, and the
registered split reserve of 32,784 bytes; `d4_delta census`):

| Wikidata block | rows | r4 / n | split records | bytes held (records and directory) | admitted capacity | ≤ 1.2 MB |
|---|--:|--:|--:|--:|--:|---|
| 0 | 13 | 1.0000 | 0 | 116 | 32,900 | yes |
| 1 | 899,931 | 0.0847 | 3,741 | 808,616 | 811,472 | yes |
| 2 | 899,992 | 0.0911 | 3,684 | 854,116 | 857,428 | yes |

F3 passes on each counted block.

**The verdict: FAILS THE COUNT, on F2.** F1 and F3 pass.

## 2. The table on every block (counted; descriptive)

- **Wikidata's 233 full blocks:** admitted capacity min / p10 / median / p90 / max =
  763,700 / 814,980 / 875,836 / 1,372,744 / 2,017,996 bytes. **197 of 233 (84.5%) are
  admitted** at 1.2 MB.
- **Pooled over all 431 blocks:** r1 0.0722 n, r2 0.0881 n, r4 0.1088 n (Pathfinder's census
  value), and records with splits 0.1129 n.
- **Text, records and random:** no full block is admitted. Text's table is 4.49 MB at
  r4 = 0.597 n. Records' and random's are 7.40 MB at r4 = 1.000 n.
- **Pattern:** all 21 full blocks are admitted, at 0.20 MB.
- **With the Optimiser's 20-byte directory** (a third word of start flags) every table is
  8 bytes a 64 rows larger: 112,500 bytes on a full block. Wikidata's counted blocks are then
  923,968 and 969,932 bytes, still under 1.2 MB.
- **The doubling scratch, reported apart:** level 1, level 2 and the level-2 directory, at
  their allocated capacities. That is 2.59 MB after block 1 and 4.40 MB after block 2: more
  than the table it builds. The records' allocation is the build's bound of 2 r2 + 2 S
  records. On block 2 that is 2.05 MB of address space, though only the records written are
  touched.

## 3. What dominated the counts

### The construction (post hoc attribution, by difference)

Method: `d4_delta phases FILE 3` builds each block's first 1, 2, … 5 steps (`build_partial`,
the same code ended early) and then the full build, each in a function of its own.
- **Ir a row by step,** as the differences of the inclusive counts. In that binary the full
  build counted 14.21 on Wikidata, not 14.45: its call context compiled differently. The
  registered figure is §1's.

| step | Wikidata | text |
|---|--:|--:|
| 1. runs of the column, a word at a time, and the tiling check | 2.72 | 8.66 |
| 2. level 2 = level 1 ∘ level 1 (cursor a byte) | 2.96 | 16.65 |
| 3. the level-2 directory | 1.21 | 4.89 |
| 4. stops, chains, singletons (sorts of about 2,000 entries a block, two bitmap clears) | 1.00 | 1.84 |
| 5. level 4 = level 2 ∘ level 2, written as records, with the splits and directory bits | 5.79 | 37.40 |
| 6. the final directory's ranks | 0.54 | 2.29 |
| total | 14.21 | 71.72 |

- **The doubling's piece-steps cost about twice the model's 30 instructions.** On Wikidata,
  step 2 is 41.8 Ir for each level-2 piece it writes, and step 5 is 65.9 for each level-4
  piece. A child piece is a dependent chain:
  - the parent's record and its successor's start;
  - the covering piece's record and its successor's start;
  - a min, the offsets, and the packing;
  - a store of 16 or 8 bytes, and a directory bit;
  - a branch on the next singleton, and the loop's exits.
- **The fixed terms alone nearly fill the filter.** Steps 1, 3, 4 and 6 are 5.46 Ir a row on
  Wikidata. That leaves 1.89 under 7.35 for both doublings together. At the Optimiser's
  30 Ir a piece-step, the doublings would cost 0.159 n × 30 = 4.8 Ir a row on these blocks,
  for a total near 10.2. **F2 is out of reach by tuning the piece loop alone.** It would also
  need the run list to arrive from the front end (the emitter) and the level-2 directory to
  be dropped.
- **Text costs 5× Wikidata a row,** because its pieces are 5–7× as many (r1 0.287 n,
  r4 0.595 n).

### The walk (counted; listing)

- **The step as compiled** (`room_fast`, popcnt and BMI2) is **23 instructions for 4
  bytes**, 5.75 a byte:
  - the row and its word index;
  - `not`, `shlx` of the boundary word, `popcnt`, and the add of the rank;
  - the record's load;
  - the bytes' shift and 4-byte store;
  - the flag test and branch;
  - the next row's add and mask, and its store;
  - the prefetch's address and the prefetch.
- **Two of the 23 are spill reloads** of the rank and record arrays' bases. With 32 lanes the
  registers run out.
- **The other 2 Ir a byte** are the skeleton:
  - the tile's flushes;
  - the hand-overs, about 3,700 a block;
  - the tail, about 0.3 Ir a byte;
  - the reversed assembly.
- **So the walk passes F1 with room:** 7.73 against 15, and 0.74× the room walk's 10.45.
  That holds on text too (7.68), where the table isn't admitted.

## 4. Data reads and writes (post hoc, descriptive; at the Coordinator's request)

callgrind with `--cache-sim=yes` on the same run. Dr and Dw are every data read and write
the functions make, stack and spills included. The L1 misses come from valgrind's simulated
default cache: descriptive only, not this machine's.

| a row | production build | production walk | D4 build | D4 walk |
|---|--:|--:|--:|--:|
| Wikidata: Ir | 4.92 | 10.45 | 14.45 | 7.73 |
| Wikidata: Dr (reads) | 0.67 | 2.56 | 3.60 | 1.89 |
| Wikidata: Dw (writes) | 0.55 | 2.32 | 2.13 | 0.76 |
| Wikidata: simulated D1 read misses | 0.03 | 1.03 | 0.15 | 0.64 |
| text: Ir | 11.13 | 10.56 | 71.78 | 7.68 |
| text: Dr | 2.09 | 2.60 | 17.73 | 1.88 |
| text: Dw | 1.29 | 2.32 | 9.20 | 0.76 |

**Table loads by construction:**
- Production's walk makes 1 load a row, a 4-byte entry of a 3.6 MB table.
- The D4 walk makes 0.25 steps a row, each 3 dependent loads:
  - the boundary word;
  - the rank;
  - then the record.

  That is 0.75 loads a row, from a table of about 0.86 MB on Wikidata.
- The walk's other reads are its lanes' rows, the tile and the segments.

## 5. Tunnel heights along the walk (post hoc, descriptive; at the Coordinator's request)

**The definition** (`d4_delta census`, from the plain walk's path from z): a stretch is a run
of consecutive LF⁴ steps over which the walk's row and a neighbouring row (the next row, or
the one before, the same side throughout) stay in one record's piece. Each such step carries
the pair to adjacent rows, so the pair's paths run parallel: a tunnel of width at least 2.
- Splits at singletons end stretches.
- **This is my definition, not Pathfinder's** (`2026-11-10-tunnels.md` on #313). It
  undercounts wider tunnels whose neighbour changes side.

| | Wikidata, all 431 blocks | Wikidata, first 3 | text, all 13 |
|---|--:|--:|--:|
| steps a row (every step a lookup) | 0.2502 | 0.2502 | 0.2502 |
| share of steps in stretches of height 1 / 2–8 / 9–32 / 33+ | 0.091 / 0.046 / 0.169 / 0.695 | 0.071 / 0.028 / 0.150 / 0.751 | 0.557 / 0.226 / 0.084 / 0.132 |
| **bytes in stretches of 33+ steps** (132+ bytes) | **69.5%** | 75.1% | 13.3% |
| table accesses a row, one for each such stretch | 0.0785 | 0.0643 | 0.2173 |
| bytes in stretches of 9+ steps (33+ bytes) | 86.4% | 90.2% | 21.7% |
| table accesses a row, one for each such stretch | 0.0385 | 0.0287 | 0.1978 |

- **Consistent with Pathfinder's "at least 48% of bytes in tunnels 33+ high"** on Wikidata,
  whichever unit "high" is taken in.
- **The access counts are an upper bound on the gain.** Following a stretch without a lookup
  needs its height and its neighbour's bytes. Neither is in this table, and their cost isn't
  counted here.
- **On text the stretches are short:** 56% of steps are of height 1.

## 6. Against the prior record: N1 (descriptive consistency check)

The 30 October research portfolio's N1, "Composed Ψ^k table, rank-indexed"
(`docs/journal/2026-10-30-research-portfolio.md` §3.1(h), on branch `docs/research-portfolio`,
not master). Its Gate A census covered all 233 full Wikidata blocks, in the Ψ direction.

| | N1 (Ψ, 233 blocks) | here (LF, 233 full blocks) |
|---|---|---|
| m₁ = r, median | 56,032 | r1 median 56,032 |
| m₂, median | 68,029 | r2 median 68,029 |
| m₄, min / median / max | 70,271 / 84,287 / 227,058 | r4 70,270 / 84,287 / 227,057 |
| P⁴ resident, median | 0.90 MB | 0.873 MB held, 0.876 MB admitted capacity |
| parents passing 1 → 2 whole | 0.906 (median of 40 sampled blocks) | 0.929 (pooled, all 431 blocks) |
| parents passing 2 → 4 whole | 0.939 (median of 40) | 0.953 (pooled; a split at a singleton counts) |

- **The piece counts agree exactly at the median.** They differ by one at the extremes, a
  counting convention.
- **The sizes agree** within the directory's difference.
- **The unsplit rates are of the same order,** under different pooling. N1's brute-force check
  of capped lane-end records (10,188 segments) is the counterpart of this probe's byte
  identity on every block (§7).
- **N1 was archived on 30 October** on the work-count rule: a fused P^{2|4} construction at
  about equal instructions to the build (external reports 62 and 63). This probe's F2 failure
  is the same finding, counted inclusively and compiled.

## 7. Correctness (checked)

- **`d4_delta check`:** every block of the bench corpora (`ab gen corpora 4` at this head)
  and of the pinned Wikidata corpus (SHA-256 `82f65ace…`). Each block runs four walks: the
  production-shaped walk, the plain walk, the room walk forced at 512 segments, and the plain
  walk on that table. All equal `bwt::walk`'s, and RLE1's output matches each block's CRC.

  | corpus | blocks | rows |
  |---|--:|--:|
  | wikidata | 431 | 209,706,557 |
  | text | 13 | 11,075,143 |
  | records | 11 | 9,600,002 |
  | random | 9 | 8,000,003 |
  | pattern | 22 | 19,600,000 |
  | onebyte | 1 | 392,160 |
  | runs | 1 | 254,432 |

- **`tests/d4_delta.rs`** (CI): `corpora(1)` at levels 1 and 9, and #323's fuzz smoke: the
  samples, crafted random columns with any `orig_ptr`, and 1,000 mutated streams.
- **Unit tests:**
  - random columns of 1 to 70,000 rows over alphabets of 1 to 256;
  - repeated strings (several cycles);
  - refused counts;
  - 1 to 512 segments at 3, 8 and 32 lanes.
- **Miri** (`cargo +nightly miri test --lib -- miri_d4`, and CI's Miri workflow, now triggered
  by `src/d4_delta.rs`):
  - `miri_d4_walks`: builds and every walk on small columns, short cycles, and cycles that
    don't divide n;
  - `miri_d4_grows`: asserts a lane's buffer grown mid-segment.

  Both pass locally (nightly, 2026-11-10). Miri runs the portable code: the popcnt and BMI2
  path differs only in the instructions `count_ones` and the shift compile to.

## 8. Construction problems found, and negative findings

1. **A design correction, found by the tests before any count.** The registration (§1) made
   `orig_ptr` a segment start and let lanes step over it. That is correct when `orig_ptr`'s
   cycle has length dividing n, but then nothing gives the cycle's length. It failed on
   crafted columns whose cycle doesn't divide n, where `bwt::walk` goes round the cycle and
   ends mid-way.
   - **As built,** `orig_ptr` is not a start. The output is `orig_ptr`'s first three bytes,
     then the segments from z = LF³(`orig_ptr`), a proper stop, round its cycle. The
     cycle's length m is the sum of those segments' lengths. The output starts at offset
     (m − n mod m) mod m, which is 0 on every block a text gives.
   - **What it changes:** the starts are M only, one fewer than registered. The registered
     reserve (2·(3·512 + 513) records) is kept, a 16-byte overstatement.
2. **An implementation bug, found by the tests:** the next row (i + record) & (2²⁰ − 1)
   overflowed `u32` as written. It's now a wrapping add, the same instruction in release.
3. **Admission can't be decided before the build.** r4 is known only after the second
   doubling. The build reserves the bound 2 r2 + 2 S records, and the doubling scratch
   (2.6–4.4 MB allocated) exceeds the table. A production version would need a bound from r
   (N1's storage rule bounds m_k by k r), a partial build, or a fallback after it.
4. **The level-2 directory** (1.21 Ir a row) exists only to find a level-2 image's first
   piece. Per-byte cursors don't serve the second doubling, since level-2 images with one
   first byte aren't monotone across second bytes.
5. **The stops cost 1.0 Ir a row on Wikidata,** mostly fixed per block: 512 chains, sorts of
   about 2,000 singletons, and two clears of a 14,000-word bitmap.
6. **The walk is not bound by the step's arithmetic alone.** 2 of its 23 instructions are
   spills at 32 lanes. The 3 dependent loads a step stand against the T-vector's 1, for 4
   bytes against 1.

## Reproduce

```text
cargo build --release --example d4_delta --example ab
target/release/examples/ab gen corpora 4
target/release/examples/d4_delta check corpora/*.bz2                                   # §7
target/release/examples/d4_delta census corpora/wikidata.bz2 corpora/text.bz2          # §2, §5, §6
valgrind --tool=callgrind target/release/examples/d4_delta count corpora/wikidata.bz2 3   # §1
valgrind --tool=callgrind --cache-sim=yes target/release/examples/d4_delta count corpora/text.bz2 3   # §4
valgrind --tool=callgrind target/release/examples/d4_delta phases corpora/wikidata.bz2 3  # §3
cargo test --lib d4_delta && cargo test --test d4_delta
cargo +nightly miri test --lib -- miri_d4
```

Base: master `9c6a6b07`.
