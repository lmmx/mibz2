# 2026-11-10: Walk backward: in the LF direction the table is written in the order the runs arrive

**Status: one observation, checked on the corpus, and three designs it opens. Designs only: each
is handed to the Coordinator with its bound, its falsifier and its gate.**

- **The observation.** mibz2 walks Ψ = LF⁻¹, as libbzip2 does, because Ψ emits a block's bytes
  first to last. Its table is indexed by F rows, so a run of L lands wherever its byte's bucket
  is: the build scatters. The same walk in the other direction, i ← LF(i), emits the bytes last
  to first, and its table is indexed by L rows: **a run of L is a run of consecutive entries, an
  arithmetic progression, written at the next slots.** mibz2 buffers a whole block before RLE1,
  so the order the walk emits in costs nothing.
- **Checked.** `examples/lf_fill.rs`: on all 431 blocks of the corpus the backward walk's bytes,
  written last to first, are `bwt::walk`'s.
- **What it opens:**
  1. the table filled from run records in one sequential pass, with no column, no sort, no merge
     and no bucket checks: every run-fed construction the record tried, without their costs;
  2. a table that steps two or four bytes at a time, built from the same records at little more
     than the single table's cost. Construction cost is what closed the pair walk (#34);
  3. a table of runs with a rank directory, 0.8 bytes a row, whose constructor is two sequential
     writes. The constructor is what compact30's campaign fought, and what #42's move structure
     lost on.

Nothing in the decoder changes. No design here is built beyond the identity check.

Labels: **proved**, **checked** (bytes equal on the corpus), **counted** (a census of this or an
earlier entry), **local** (timed on the shared VM: indicative only, RULES C5), **model**.

## 1. The identity (proved; checked)

For a block's last column L of n rows, with C(c) the rows before byte c's bucket:

- LF(i) = C(L[i]) + rank_{L[i]}(L, i), the copies of L[i] before row i.
- The walk i₀ = `orig_ptr`, i_{t+1} = LF(i_t) emits L[i_t] as the block's byte n − 1 − t. (Row
  `orig_ptr` is the original text's rotation, whose last byte is the text's last; LF moves to
  the rotation one byte earlier.)
- **On a run.** Let L[p..p + ℓ) be a run of byte c with k copies of c before p. Then
  LF(p + j) = C(c) + k + j. So the entries P[p + j] = c | (C(c) + k + j) << 8 are consecutive
  values in consecutive slots.
- **So the table is a function of the run records in arrival order** and one counter a byte:
  for each record (c, ℓ): write ℓ entries from next[c], then next[c] += ℓ. The counters start
  at C, which needs the block's byte counts: the records' pass gives them.
- **Soundness is the tiling.** The records' lengths sum to n, so every slot is written once, in
  order. Each byte's lengths sum to its count, so its values tile its bucket. P is then a
  permutation with no bucket check and no end check per write.

`bwt::build_one_pass` writes `ll[s] | i << 8` at s = the byte's next F slot: a scatter into up to
256 streams, a read of `ll[s]` beside each, and a bound check per write. Libbzip2's small mode
builds LF sequentially too, then reverses the chain in place to stream forward. No decoder in the
record walks it backward.

**Checked:** `lf_fill` on the corpus: 431 blocks, 209,706,557 rows, 15,148,086 runs (0.0722 a
row); the bytes equal `bwt::walk`'s on every block. A fill that writes 8 entries a store gives
the same table as the plain one.

## 2. Design A: the sequential table (the record's run-fed line, without its costs)

**The design.**
- **Front:** MTF emits run records and byte counts, with no column. `examples/run_front.rs` has
  that emitter at 0.173 ns a row against `mtf::mtf`'s 0.423 (local; the Coordinator is taking it
  to production).
- **Build:** one pass over the records: `e = next[c] << 8 | c; next[c] += ℓ;` then ℓ entries
  `e, e + 256, …` written 8 at a time into slack the next run overwrites. No data-dependent
  branch but the loop over runs longer than 8.
- **In a team:** the helpers fill disjoint row ranges. A share starting at row q needs each
  byte's count before q: one prefix pass over the records (r of them), or counts the emitter
  drops every so many rows. No checkpoints in the column, and no write-window checks: a share
  writes only its own rows.
- **Walk:** today's room walk with one sign changed. A lane writes its bytes downward; a segment
  ends at a flagged row as now; the segments join in the reverse of today's order. Bit 31 stays
  the start flag: LF(i) < 2²⁰ as Ψ is.
- **After the walk:** unchanged. RLE1 and the CRC read the block's bytes first to last.

**What it removes, against the record's attempts:**

| attempt | what it needed | here |
|---|---|---|
| line A (`2026-11-08`), the sweep | records sorted into F order, two iterators merged | none: L order is arrival order |
| #269, the run-fed stage | emitter, sort and fallback; lost 1.159× | emitter and one fill |
| #277, direct placement | a per-run search for its F slot | `next[c]` |
| every variant | the column kept for the build's `ll[s]`, or records beside it | no column |

**The price (local, indicative).** `lf_fill`, decode order, fastest of 5 rounds: `bwt::build`
0.458 ns a row; the 8-wide fill 0.293. Both are near the cost of writing 3.6 MB: the table's
bytes are the floor, whoever writes them. With the emitter: front plus build 0.47 ns a row
against today's 0.88.

- **Ceiling (model):** about 0.4 ns of a 3.4 ns row on one thread here: 12%. In a pipelined
  team the leader's serial MTF falls to 0.41 of itself and the build's two reads of the column
  go. On the i9 the column's cold writes go (13% of the plan's, in
  `2026-11-10-bandwidth-titration.md`'s ledger).
- **Cheapest falsifier:** CI's stage harness (`examples/phases.rs`) with the fill beside the
  build, both from one block's symbols: front plus build must be at most 0.75 of today's on
  Wikidata, and the walk's stage within its noise of today's. If the backward walk is slower
  than the forward one by more than the front saves, A is dead and so are B and C.
- **Gate before integration:** the walk's bytes equal on every corpus and on the fuzz loop; the
  tiling argument reviewed in place of `build_one_pass`'s (RULES B5); CI's paired bench.
- **Risk:** the reversed join, and any reader of segments that assumes ascending order (the
  staged tile, the fused post-walk idea). The identity itself is not at risk.

## 3. Design B: two or four bytes a step, built from the same records

**Why it closed.** #34's pair walk took half the steps, each about twice the work, and its build
cost 7.1–7.6 c/B against 2.5: "construction and per-step costs outweighed the saved lookups".
Under ten threads on the i9 the same kernel won (1.107, 1.117): fewer reads past L2 a byte.

**What changes in the LF direction.** LF² on a run is the run's F interval pushed through LF
again, so it is cut where that interval crosses L's run starts, and is a translation between
cuts. Counted (`2026-11-09-phrase-structure.md` §3, the cells of Ψ^s; LF^s has the same count):
the pieces number 1.22 r at two steps, 1.51 r at four, 1.97 r at eight.

**The design.**
- **Entry:** LF^k(i) in 20 bits and the k bytes L[i], L[LF(i)], …: 36 bits at k = 2, 52 at
  k = 4. So 5-byte entries for pairs (4.5 MB a full block) or 8-byte entries for fours (7.2 MB).
- **Build, k = 2:** for each record (c, ℓ) the F interval is [a, a + ℓ), a = next[c]. The runs
  of L that cover it are found by a cursor per byte, since a byte's successive F intervals are
  adjacent and increasing: 256 monotone cursors over the record list. Each covered piece is
  written as consecutive entries: LF² values consecutive, second byte the piece's run's. The
  work is one step a piece (1.22 r) and the fill.
- **Build, k = 4:** the same again from the pair table's pieces, or two lookups of the single
  table at each piece's first row. Pieces 1.51 r.
- **Walk:** n/k steps, each one load, one k-byte store, one shift and mask. A lane's last
  partial step ends at the block's start as #34's did.

**Bound (model).**
- **Steps:** the room walk is 10.5 instructions a byte. A four-byte step of about 16 is 4 a
  byte. Where the walk is bound by instructions (2 MiB L2: 1.78 ns a row here), that is up to
  1 ns a row back.
- **Build:** the entries' bytes are the floor: 0.3 ns a row at 4 bytes (§2), so about 0.6 at 8.
- **Net, one thread, here:** walk 1.78 → about 0.8, build 0.29 → about 0.7: 0.6 ns of 3.4.
- **Reads past L2 a byte** fall to 1/k, on a table 1.25× or 2× the size. On the i9 that trade
  is the one #34 won at 8–10 threads and lost below.

- **Cheapest falsifier:** a count before any code: the instructions of a four-byte step in the
  room walk's shape, by listing. Over 20, B is closed at k = 4. Then the stage harness: build
  plus walk at k = 2 with 5-byte entries must beat today's on Wikidata at 1 thread, which #34's
  never did.
- **Reopens #6/#34 on their named gap,** construction cost, and only that.

## 4. Design C: a table of runs with a rank directory (for plans the T-vector does not fit)

**The gap it meets.** Past the knee the i9 gives each table a fraction of the L3. The record's
compact tables either kept every step (the 24-bit table), or paid for their constructor
(compact30's campaign; #42's three O(r) passes at 30–40 cycles a run each, 7.8–9.2 c/B), and
#42's step carried a fast-forward taken on 27–29% of steps.

**The design, in the LF direction.**
- **`runs`:** one u64 a run of L, in L order: its first row (20 bits), its LF base C(c) + k
  (20 bits), its byte (8). Written sequentially from the records: it *is* the record list with
  two running sums.
- **`dir`:** one 16-byte entry per 64 rows: a u64 with a bit at each run start, the count of
  runs before the word (u32), and the segment-start flags' word or a pointer to it. Written
  sequentially in the same pass.
- **A step at row i,** with no branch:
  `d = dir[i >> 6]; run = d.rank + popcnt(d.bits & (2 << (i & 63)) − 1) − 1;`
  `r = runs[run]; emit r.byte; i = r.base + (i − r.first);`
- **State is the row,** as in the T-vector's walk, so the room walk's lanes, flags and join are
  unchanged. There is no fast-forward: the next row's run is looked up, not carried.
- **Size (counted):** 8 r + n/4 bytes: 0.83 a row pooled (0.75 MB a full block), 1.1 at the
  ninth decile of blocks, 1.85 at the worst. The T-vector and the column are 5.
- **Constructor:** two sequential streams of 0.83 bytes a row. No sort, no merge, no caps.
- **Admission:** by the record count, known before the build: blocks over a share of n keep the
  T-vector.

**Bound (model).**
- **A step:** about 17 instructions and two dependent loads (the directory word, then the run),
  against the T-vector's 10.5 and one. Where the T-vector's load is an L3 hit anyway (the i9,
  256 KiB L2), the lanes hide the second load, and the step is bound by its instructions: about
  1.6× the room walk's.
- **Alone on the i9:** build 2.4 → about 0.2 c/B; walk 5.5 → about 7–9. No gain, perhaps a
  loss: C is not for one thread.
- **Ten workers on the i9:** ten of these are 7.5 MB, inside the L3 with the output streams.
  Production's ten tables are 36 MB, and its per-thread inversion costs 3–4× its cost alone
  (the knee entry's E1). A walk that stays at 1.6× would be about twice production's rate there
  before anything else binds, and it writes 0.83 cold bytes a row where the build writes 4.
- **What binds next:** the scanner, at 9–11× one thread (`2026-11-10-parallel-scan.md`).

- **Cheapest falsifier:** by count, then by one run. The step's instructions by callgrind on
  three blocks: over 20 a row, stop. Then the inversion harness the knee used (`examples/sweep`
  or the coop grid) at 1 to 10 one-table workers on the i9, as a registered diagnostic: C's
  rate at 10 workers must be at least 1.3× production's best plan's inversion rate, or it is
  closed for that machine.
- **Against compact30:** the same family (a popcount in a per-tile word). Compact30 indexed F
  rows in fixed tiles with plateau values, and its constructor was the campaign. Here the tile
  word marks exact runs of L and the values are the records.
- **Against #42:** no interval state, no fast-forward, no F-order sort, no merge.

## 5. Ranked

| | ceiling | confidence | first step, and whose |
|---|---|---|---|
| A. the sequential table | 12% of one thread's decode here; the column and its cold writes on the i9 | high: identity checked, fill timed | the Coordinator's, with the emitter it is already building: the fill replaces the F-order sort and sweep |
| C. runs with a rank directory | about 2× at ten workers on the i9 (model); nothing at one thread | medium: step cost unmeasured | a listing count, then a registered inversion diagnostic |
| B. two or four bytes a step | about 0.6 ns of a 3.4 ns row at one thread on a 2 MiB L2; fewer reads past L2 everywhere | medium-low: #34 lost on per-step cost too | a listing count of the four-byte step |

A is a prerequisite of B and C as written: all three walk backward.

## Reproduce

```text
cargo run --release --example lf_fill -- corpora/wikidata.bz2 5
```

Base: this branch at master `02c2bbfb`.
