# Compact30: a direct-constructor design brief (for Astra; report 186)

**The assignment:** recommend one algorithm that builds compact30's frozen records directly
from a block's last column. It must not build the oracle's full temporary successor table.
Give its allocations, memory traffic, RAW handling and CPU-cost risks.

**Not in scope:**
- a new layout or walker variant;
- tuning the validated walk;
- integration into the decoder;
- a benchmark design beyond what the later construction-cost comparison needs (that is
  registered separately);
- any implementation, which follows only after the guardian has reviewed this design.

**The source pins:**
- the walker and its oracle builder: `examples/c30walk.rs` at `f1dafa3` (the frozen
  capture's source);
- production's builder: `src/bwt.rs` at the same commit.

## 1. What the traversal experiment established

The comparison is protocol revision 5 (`2026-11-02-compact30-traversal-protocol.md`). Its
fourth i9 capture is VALID, with the decision ADVANCE. The guardian and the researcher each
verified it from the raw records.

| C / P | point | 95% interval |
|---|--:|---|
| w1, CPU cost per row, one core | 1.14845 | 1.14743–1.14948 |
| w4, CPU cost per row, four cores in lockstep | 0.84047 | 0.83781–0.84313 |
| e4, four-core lockstep completion time | 0.85601 | 0.85239–0.85965 |

**Two cautions, both stated by the guardian:**
- **Compact30 loses 14.85% on one core.** The four-worker lockstep gain can't be assumed to
  carry over unchanged to the full decoder. A later integration must test the decoder's
  real mix of concurrency, the cache state construction leaves, and end-to-end throughput.
- **The gain's cause is not isolated.** The smaller table, 2.164 logical bytes a row against
  4, is a credible explanation, not a measured attribution.
  - In the capture, production built its table immediately before each walk.
  - Compact30 copied a prebuilt oracle image instead.
  - Both preparations were untimed, but they leave different caches warm.

## 2. The construction budget

The registered planning model:

    S ≈ 0.592 (1 − b) + 1.847 (1 − w)    [CPU-s saved per decoded GB; target S ≥ 0.25]

- b is the constructor's CPU cost relative to production's build, and w is the traversal
  cost ratio.
- With w at w4's upper bound (0.84313), the largest allowed b is **1.06714**: a direct
  constructor may cost up to about 6.7% more than production's build.
- At w4's point estimate, the bound is 1.07544.
- This is a planning screen, not a prediction for the decoder. Meeting it is necessary,
  not sufficient.

## 3. The frozen representation (the constructor's output)

`C30::build`, `examples/c30walk.rs` lines 226–330 at `f1dafa3`, is the reference: a direct
constructor must produce its arrays exactly.

**Rows, groups and labels:**
- A block of n rows, in F order (sorted).
- Group g holds rows 30g to min(30g + 30, n) − 1. There are ⌈n/30⌉ groups.
- Row r's label is φ(r) = 32⌊r/30⌋ + r mod 30. Labels 30 and 31 of each 32 are never used.
- **Successor:** the row whose L-position is i reaches slot s = the next free slot of byte
  L[i]'s bucket. That is the same counting scatter as production's builder. Slot s's
  successor label is φ(i).
- F[s] is the byte of the bucket that holds s.

**`Rec`:** 64 bytes, 64-aligned. Bytes 0–59 hold `f: [u16; 30]`, and bytes 60–63 hold the
header `h: u32`. There is one record per group, plus a final PARK record.

**An ordinary group:** all its rows are in one bucket, and its successor labels span at most
65,533 (`MAX_OFF`).
- `h = base << 8 | F`, where base is the first row's successor label, at most 2²⁰ − 1.
- `f[j] = label(row 30g + j) − base`.
- Unused fields of a partial last group stay `0xFFFF`.

**A RAW group:** a bucket boundary inside the group ("crossing") or a span over 65,533
("wide").
- `h = RAW_BIT | idx`, where `RAW_BIT = 1 << 28` and idx is the RAW group's index.
- Every field is `0xFFFF`.
- Its `RawG` is 32 `u32` entries, `label << 8 | F` for each row, then zeros. It is
  128 bytes and 64-aligned.
- The oracle stores RAW groups in increasing g.

**PARK:** all fields `0xFFFF`, with `h = RAW_BIT | (2²⁰ − 1)`. Its label is 32·⌈n/30⌉.

**The reserved field values:**
- `0xFFFE` marks a segment start. The walk writes these at run time; the constructor never
  does.
- `0xFFFF` marks RAW or parking.

**The logical size:** 64·(⌈n/30⌉ + 1) + 128·(RAW groups) bytes. On the capture's corpus
that is 2.1642 bytes a row, with about 240 RAW groups per million rows.

**Facts in the code that bear on direct construction:**
- **Within a bucket, successor labels increase strictly with slot.** The scatter visits i in
  increasing order, and φ is increasing in i. So an ordinary group's offsets are ≥ 0 and
  its span is last − first.
- **Crossing groups are known from the counts alone.** The bucket boundaries are the
  counts' prefix sums.
- **Wide groups are known only once a group's last row is placed.** Its span isn't known
  before then.

## 4. Production's builder (the cost baseline)

`build_one_pass`, `src/bwt.rs` lines 253–318 at `f1dafa3`:
- The bucket starts and ends come from the counts.
- **One pass over L, 16 rows at a time:**
  - **16 equal bytes** (one comparison of two `u64` and a rotation) are 16 consecutive slots
    of that byte's bucket. They are written with one bound check, entry
    `ll[s + j] | (i + j) << 8`.
  - **Otherwise,** each row writes slot `s = cursor[L[i]]++` with entry `ll[s] | i << 8`.
- **Traffic:** one 4-byte store per row into an n-entry `u32` table (3.6 MB for a 900,000-row
  block), through 256 bucket cursors. Each cursor writes sequentially within its bucket.
  L is read sequentially at i, and at s within each bucket.
- **Checks:** every store is bounds-checked against its bucket's end. Afterwards every
  cursor must equal its end, which proves each slot was written once. Otherwise it falls
  back to the two-pass build.
- After the Burrows–Wheeler transform, most of Wikidata's last column is in runs of equal
  bytes, so the 16-row path carries much of the work.

## 5. The oracle builder today (what not to keep)

`C30::build` first builds a full successor-label array (4n bytes) and an F array (n bytes),
plus a class byte per group: `oracle_temp_bytes` = 5n + ⌈n/30⌉. It then classifies every
group and writes the records and RAW groups. It was never timed: in the capture it was
built once per block, outside every window, and copied in.

## 6. What to return

**1. One construction algorithm,** from L and the counts to `rec` and `raw`, without an
n-entry successor table. Give pseudocode, including how it handles:
- the 16-equal-byte runs;
- a run that crosses a group boundary;
- the label sequence φ(i) as i advances, without a division per row.

**2. Its allocations:**
- sizes and when they are made;
- how the RAW area is sized: crossing groups are countable up front, wide groups are not;
- peak temporary memory, against production's.

**3. Its memory traffic:**
- bytes read and written per row, and the write pattern (cursors, partial lines, headers);
- against production's 4 bytes a row through 256 cursors.

**4. RAW promotion:**
- how a group that turns out to be wide becomes RAW after some of its fields are written,
  and what that costs;
- how crossing groups are handled;
- whether the RAW groups' order, increasing g, is kept. Keeping it makes equivalence a byte
  comparison.

**5. Its CPU-cost risks against the 1.06714× budget,** with the risks you expect to dominate.
For example:
- the per-group base and header;
- 2-byte stores;
- bucket-boundary and wide checks;
- the fast path's group-boundary handling;
- RAW handling.

**6. A correctness plan:**
- `rec` and `raw` byte-identical to `C30::build`'s on the 11 hash-checked corpora and the
  adversarial set;
- edge cases:
  - n not a multiple of 30;
  - a bucket boundary at every offset within a group;
  - spans of exactly 65,533 and 65,534;
  - one-symbol blocks;
  - 900,000-row blocks;
- the walker's output unchanged.

**7. What the later construction-cost comparison should measure** against `build_one_pass`
on the same inputs, so that it can be registered separately.

Keep the record format, the walker and every traversal result as they are. If the
format can't be built affordably as it stands, say so and why, rather than proposing a new
format.
