# 2026-11-08: The run-fed T-vector (line A) and a run-compressed Ψ² census (line B)

**Result: both lines stop here, before any timing.**
- **Line A,** the existing T-vector filled from run records by one monotone sweep: correct,
  word for word against `bwt::build` on every test case and on all 431 blocks of the bench
  corpus. But the cost model finds no structural saving to buy. The production build already
  stores 82% of Wikidata's rows through its 16-row run path, and the model puts both builds
  at about 2.5–2.6 instructions a row. Replacing L with run records saves no peak memory in
  either production path, and costs up to 2.2 MB a worker on one of them. A registered A/B is
  not proposed.
- **Line B,** a two-hop record table: the refinement is real and smaller than its bound
  (r₂/r = 1.22 on Wikidata, not 2). But the census puts the table at 0.96 B a row and 0.53
  fast-forwards a two-hop step. By the instruction model, the walk alone would be about
  three times the line's own gate. No walker is proposed.

**The input.** #145 comment 6726489772, a research relay from Louis via ChatGPT ("why it
hasn't been making as fast? And can we do anything that would make it faster? Can we research
this intensively"). It proposes these two lines, each a hypothesis. This entry re-derives
both, builds the reference code and runs the untimed census the relay asks for first.

**What changed:** nothing in the decoder. The reference code is `src/run_fed.rs`, compiled
only for the tests (`#[cfg(test)] mod run_fed` in `src/lib.rs`). Nothing calls it.

Labels used below:
- **proved:** a short argument, given here;
- **checked:** by the tests or the census, as counts, on the cases named;
- **model:** an instruction or footprint estimate, assumptions stated, never measured;
- **unmeasured:** no timing of any kind was taken. Local timing on the shared machine is
  not evidence, and none is reported.

## 1. The objects (proved)

Take a block's last column L, of n rows, with byte counts and C(c) the rows before byte c's
bucket in F. Production's table is `T[q] = L[q] | Ψ(q) << 8` (`build_one_pass` writes
`ll[s] | i << 8` at slot s, where s is the next slot of row i's byte, so `Ψ(s) = i`).

Each run of L is a record (c, p, ℓ, k): byte c at positions p..p+ℓ, with k copies of c in L
before p. A stable counting sort gives the occurrence of c at p + i the F row C(c) + k + i.
So on the F interval [a, a + ℓ), a = C(c) + k, **Ψ(a + i) = p + i**. Over all runs, these F
intervals tile [0, n): each bucket is tiled by its byte's runs in k order. The L runs tile
[0, n) too.

## 2. Line A: the T-vector from run records

### 2.1 The claim (proved)

- **The cell bound.** Two tilings of [0, n) by r intervals each have at most (r − 1) +
  (r − 1) interior boundaries between them. So their common refinement has **at most 2r − 1
  cells**.
- **The fill.** On a cell, q lies in one L run, so L[q] is constant. It also lies in one F
  interval, so Ψ steps by one. Then T[q + i] = T[q] + 256·i, with no carry, since Ψ < 2²⁴.
- **The sweep.** One iterator walks the runs in F order (the records counting-sorted by
  byte, stable). The other walks them in L order. At each step it writes `min(F end,
  L end) − q` entries and advances whichever iterator ended. It is O(n + r + σ), with no L
  array.

**A count identity (proved, and checked on every block).** The L runs' starts are Ψ(S),
where S is the set of the F intervals' starts. So line A's boundaries are S ∪ Ψ(S).
Line B's (section 3) are S ∪ Ψ⁻¹(S). The bijection Ψ⁻¹ maps the first set onto the second,
so **line A's cells and line B's r₂ are always equal in number**. The census agrees on all
464 blocks.

### 2.2 The reference and its checks (checked)

`src/run_fed.rs`:
- `runs_of`: a column's runs, maximal or also cut at every `split` rows, as the
  checkpointed MTF cuts a run that crosses a checkpoint;
- `runs_from_symbols`: `mtf_scalar`'s loop with the same checks and errors, emitting run
  records and writing no column. It asserts the MTF post's claim that a literal always
  starts a run;
- `f_order`: the stable counting sort, giving each interval's first row a;
- `sweep`: the two-iterator fill over any row range. Its starts are found by binary search,
  so a share needs no count checkpoints. It asserts the tiling at every cell;
- `build_from_runs`: the sort, then one sweep;
- `build_from_runs_wide`: the same table, with each cell written 8 entries at a time,
  unconditionally, into slack that the next cell overwrites. This is the branch-free fill
  that the cost model assumes.

Each case below is compared **word for word** with `bwt::build`'s table. That is
`build_one_pass`, run stores included, which the cases of 16 rows and more exercise. Each
case is built five ways: the plain and wide fills, from maximal and from checkpoint-cut
runs, and in 1–5 shares. Each also asserts the cells are fewer than 2r.

| set | cases | mismatches |
|---|--:|--:|
| every column over 3 letters, lengths 1–9 | 29,523 | 0 |
| every column over 4 letters, lengths 1–7 | 21,844 | 0 |
| every column over 2 letters, lengths 1–14 | 32,766 | 0 |
| random columns, 16–70,000 rows: run-heavy, uniform and periodic, σ 1–256 | 126 | 0 |
| **the bench corpus**, `wikidata.bz2` (SHA-256 `82f65ace…`, the file the bench builds), every block | 431 | 0 |
| tests/common-style text, records and random corpora (scratch copies, SHA-256s below), every block | 33 | 0 |

Any column is a valid input: `build` takes any L, whose Ψ may have several cycles. So
arbitrary columns are a stronger test than texts' BWTs alone.

On the real blocks, the census test (`census_real_blocks`, ignored without a corpus) also
checks the following:
- the runs from the MTF symbols equal the runs of the column `mtf::mtf` writes, with the
  same length and counts;
- the literal-starts-a-run assertion holds on every block;
- the table built in three shares is the same;
- the production room walk over the table leaves it as built;
- each block's RLE1 output matches its stored CRC.

The run-fed table is identical to the production one, so the room walk's output from it is
the production output, byte for byte.

**Mutation checks.** An off-by-one in the link of cells longer than 20 rows fails
`line_a_random`. Dropping Ψ⁻¹(S) from S₂ fails `line_b_real_bwts_and_columns`. Both were
run once and reverted.

### 2.3 What production would need

- **`orig_ptr`:** nothing. The table is identical, so the walk's first row `T[orig_ptr] >> 8`
  and the caller's `orig_ptr < n` check are unchanged.
- **Segment starts:** nothing. The walks flag their starts in bit 31 of the built table and
  take the flags off again (`room_segments`, `RoomWalk`). They never read L.
- **Checkpoints:** none needed for the build.
  - A share of rows q0..q1 finds both iterators by binary search: over the F-order starts a
    and over the L-order starts p. So `mtf_checkpoints` and `FusedBuild`'s windows would
    go.
  - Their soundness argument would need a replacement: `check_inputs` and the end check
    proving every slot written once.
  - The replacement is O(r): the runs' lengths sum to n, each byte's runs sum to its count
    with k their running sum, so both tilings hold. The sweep's tiling assertion is that
    check, and it must pass before the table's length is set.
- **The MTF kernels:** `mtf_scalar`, `mtf_avx2` and both checkpointed twins would emit
  records instead of bytes, with every error unchanged. The reference does this for the
  scalar loop only.
- **A fallback to L, always** (the ledger below):
  - Records cost more than the column when 8r > n. That is 15 of the 233 full Wikidata
    blocks, and every text, records and random block.
  - Deciding up front from the symbol count (8·|syms| ≤ n) keeps 186 of the 233.
- **The run sort:** O(r + 256) more work. It runs on whichever thread holds the records. In
  the pipelined team that is the leader, whose serial stage is the team's critical path
  (`2026-11-07-knee-mechanism.md`, H1).
- **The trait:** `InverseBwt` takes `ll`, so a records entry point would be new, and so would
  `Front`'s storage. RLE1 and the CRC are untouched.

### 2.4 The memory ledger (checked counts, model bytes)

Run records cost 8 B (byte and start packed in a u32, plus a u32 order index) to 12 B (byte,
start, length, k) each. Per full Wikidata block (233 blocks, n ≥ 65,536):

| | pooled | median block | p90 | max |
|---|--:|--:|--:|--:|
| r / n (checkpoint-cut) | 0.0723 | 0.0623 | 0.1106 | 0.2000 |
| records, B/row at 8 B | 0.58 | 0.50 | 0.88 | 1.60 |
| records, B/row at 12 B | 0.87 | 0.75 | 1.33 | 2.40 |

The column costs 1 B a row. A kept buffer's capacity is its high-water mark: the worst block,
not the pooled one.

- **The one-worker kernels** (`Segmented::invert_in_place`: `Plan::Workers`, the plan
  below 6 physical cores, as on CI's 4-core runner):
  - **Today:** L is written over by the walk's output, so one n-byte buffer serves both.
  - **Records in a buffer of their own:** an n-byte output buffer is needed as well. That
    costs **+0.58 to +0.87 B a row pooled**, and the capacity follows the worst block:
    **+1.4 to +2.2 MB a worker**.
  - **Records stored inside the output buffer** (dead before the walk writes it): **no
    change**, with the fallback to L whenever 8r > n.
  - **No saving either way.**
- **The pipelined teams** (3 × 3, the release plan on 6 or more physical cores):
  - **Today:** each team keeps two front sets, each an n-byte `ll` and about 56 KB of
    checkpoints. The output buffers are separate, so `ll` is not reused there.
  - **With records:** a front holding records or, as the fallback, L in one n-byte arena
    keeps its capacity at n. It saves only the checkpoints: about 0.11 MB a team, 0.34 MB
    for three teams, against a peak RSS of about 67 MB at three teams (E5 in the knee
    entry).
  - **Records in buffers of their own:** the capacity grows (+0.5 to +1.3 MB a front at the
    worst block).
  - **What does fall:** the bytes a block touches in its front, 0.58n against n, and the
    build's n reads of `ll[s]`.
- **The table** stays 4n bytes. Line A is not a compact structure.

**Verdict on memory:** no peak-RSS saving worth having on either path. It costs more unless
the records share an existing buffer. The relay's warning holds.

### 2.5 The cost model (model; unmeasured)

The model assumes x86-64 with AVX2, counts instructions from the code's shape, not from
compiled output, and ignores frequency, cache misses and contention. It describes the
shapes of the two builds, not their speeds.

| per row, Wikidata | today: `build_one_pass` | run-fed: sort + sweep |
|---|--:|--:|
| rows taken by 16-row run stores | **82%** (checked: 16-row windows of one byte) | (all rows by cells) |
| cells / n | — | 0.088 (checked); mean cell 11.3 rows; 91.7% of cells ≤ 8 rows |
| loads | ll[i] sequential; ll[s] in up to 256 bucket streams | 2 record streams, sequential; 0.072 records a row |
| stores | 82% of rows in 64 B contiguous lines; 18% as single 4 B stores into up to 256 advancing bucket streams | one sequential 4 B/row stream; plus 0.072 order-index stores a row, scattered over the byte streams (the sort) |
| instructions | ≈0.44 (window test) + ≈0.61 (run windows) + ≈1.5 (mixed rows at ≈8–9 each) ≈ **2.6** | ≈0.87 (sort, ≈12 a run) + ≈1.23 (≈14 a cell) + ≈0.43 (8-wide fill) ≈ **2.5** |
| data-dependent branches | one a 16-row window (run or not) | the fill's loop exit on the 8.3% of cells over 8 rows (≈0.007 a row); the iterator advances branch-free |
| written upstream by MTF | n bytes (L; long runs as a memset) | 8–12 B a run (0.58–0.87 B a row) |

- **The two builds come out about equal in instructions a row.** The saving the relay
  hoped for, a "single monotone fill instead of many scattered streams", is already mostly
  taken. 82% of Wikidata's rows are written today as 64-byte lines by `RUN_LANES` (#32).
- **What remains:**
  - the 18% scattered rows' store pattern;
  - the column's write in MTF;
  - the build's reads of L.
- **The CI stage figures bound the prize:** build 0.48 ns a byte, of about 2.14 ns a byte
  of one-thread work (`2026-11-07-knee-mechanism.md`, E4). The model suggests a fraction of
  that, of either sign. In the pipelined team, the sort's serial work falls on the leader.

### 2.6 Verdict on line A

**No registered A/B is proposed.**
- **For it:** the construction is sound and exact, and it removes the checkpoints.
- **Against it:**
  - the model shows no structural saving: no pass, copy or materialisation is removed
    beyond the column's write;
  - the memory ledger shows no peak saving;
  - production would need a dual-format front, new MTF kernels and a replacement soundness
    argument.

The lead stays a closed hypothesis, to reopen if either of these happens:
- a profile of the current pipelined team shows the build's scattered stores, not
  instructions, on the critical path;
- the run windows' share falls well below 82% on a corpus that matters.

**If the coordinator wants the information anyway,** this is a diagnostic sketch only.
**NOT FROZEN, NOT RUN, NOT RECOMMENDED.**
- **The question:** is the run-fed front plus build cheaper than today's, in decode order?
- **The arms:**
  - B0: master's `mtf::mtf` and `bwt::build`, pinned at the merge base;
  - A1: `runs_from_symbols`, `f_order` and the wide sweep, with the L fallback when
    8·|syms| > n.
- **The harness:** CI's runner, one thread, in decode order as `examples/phases.rs` runs
  (each stage timed as the decode meets it), arms alternating by round, at least 3 rounds;
  the full tier, not the PR tier.
- **The decision rule:** a production prototype only if A1's MTF + build is at most 0.90 ×
  B0's on Wikidata, with the 95% interval of the paired ratio below 1. Also, no corpus may be
  more than 2% worse through the fallback.
- **Validity, every block:**
  - T identical word for word;
  - the walk's output identical byte for byte;
  - the CRC;
  - the runs from the symbols equal the runs of L;
  - fewer than 2r cells.
  A failure voids the round.
- **Reported apart:** symbol processing, sort and sweep, plus the fallback blocks' count.

## 3. Line B: a run-compressed Ψ², census only

### 3.1 The claim (proved; checked)

The F formulation: a step at row x emits F[x] and moves to Ψ(x). The production walk from
`orig_ptr` emits exactly these bytes, since its first byte is L[Ψ(orig_ptr)] = F[orig_ptr].
S is the set of the F intervals' starts, one a run, so |S| = r.
- **Ψ² translates off S₂ = S ∪ Ψ⁻¹(S \ {0}).** Inside an interval of S, Ψ translates. It
  maps x and x + 1 into one interval of S unless Ψ(x + 1) is a start, that is, unless
  x + 1 ∈ Ψ⁻¹(S). So off S₂, Ψ²(x + 1) = Ψ²(x) + 1.
- **The bound:** |S₂| ≤ r + (r − 1) = 2r − 1.
- **Both bytes are constant on a cell:** F[x] by the bucket, and F[Ψ(x)] since Ψ(x) stays in
  one interval.

Checked:
- **every permutation of 1–8 elements** (46,233): S₂ holds every breakpoint of π², and
  |S₂| ≤ 2|S| − 1;
- **real BWTs of every ternary text of length 1–9** (29,523, periodic and cyclic texts
  included, built by sorting rotations), and the 126 random, periodic and run-heavy columns
  at three primary indices (several cycles where the column is no text's):
  - the dense Ψ, Ψ² and F confirm that no break falls off S₂;
  - a one-chain walk of the capped, marked two-hop records (below) reproduces the
    production walk's output byte for byte, odd n included;
- **all 464 corpus blocks:** the same, against the room walk's output.

### 3.2 The census (checked counts)

Per block, untimed. These are the definitions:
- **Cap:** 2n/r, from each cell's own start (anchored).
- **Markers:** per segment start s, as the production walk reads it, the F row x = Ψ⁻¹(s)
  and the row Ψ⁻¹(x) whose intermediate row it is are each made a one-row record. A lane
  stops on the first, and after one byte on the second.
  - The one-core room walk uses 512 starts.
  - The pipelined team with two helpers uses 1,024.
- **Records:** at most 2²⁰ rows (bzip2's blocks are at most 900,000), so 8 B a record when
  its fields fit 64 bits.
- **Fields:** the image's record index, the offset into it, the length, two bytes and two
  flags.

| | Wikidata (233 full blocks) | text (13) | records (11) | random (9) |
|---|--:|--:|--:|--:|
| r / n | 0.0722 | 0.166 | 0.997 | 0.992 |
| r₂ / n (= line A's cells / n) | **0.0881** | 0.245 | 1.000 | 1.000 |
| r₂ / r | 1.22 | 1.48 | 1.003 | 1.008 |
| r₂ / (2r − 1), max block | 0.65 | 0.74 | 0.50 | 0.50 |
| fewest cells / r₂ (S₂ exact) | 1.000 | 1.000 | 1.000 | 1.000 |
| records / n, capped at 2n/r | 0.1165 | 0.272 | 1.000 | 1.000 |
| records / n, capped at 2n/r₂ | 0.1236 | 0.292 | 1.000 | 1.000 |
| + 512 starts' markers | 0.1181 | 0.274 | 1.000 | 1.000 |
| + 1,024 starts' markers | 0.1197 | 0.275 | 1.000 | 1.000 |
| records added by markers, a block (512 / 1,024) | 1,417 / 2,823 | 1,318 / 2,630 | 0 / 0 | 0 / 0 |
| bits a record needed (min–max) | 44–47 | 42–44 | 41 | 41 |
| **table, B/row at 8 B** (pooled; median, p90, max block) | **0.96** (0.83, 1.44, 2.44) | 2.20 | 8.0 | 8.0 |
| table, a mean block / the largest | 862 kB / 2,196 kB | 1,902 kB / 1,915 kB | 6,982 kB | 7,111 kB |
| fast-forwards a two-hop step, one chain (max) | **0.53** (36) | 0.49 (11) | 0.0 (1) | 0.0 (1) |

Wikidata's per-block spread at 1,024 starts' markers: records/n min 0.089, median 0.104,
p90 0.180, max 0.305.

- The relay's pre-segmentation upper bound was r₂/n ≤ 0.144 and capped records ≤ 0.217n
  (≤ 1.74 B a row). The census is well under it: 0.088, 0.120 and 0.96 B a row.
- On text, records and random, the table is 2–8 B a row, so a fallback to the T-vector
  would be required.
- **Against #42's move structure** at c = 2: that had 0.103 intervals/n with 512 starts,
  0.82 B a row and 0.35 fast-forwards a one-byte step. The two-hop table holds 15% more
  records and takes half the steps, but makes half again as many fast-forwards a step. Per
  output byte, it makes 0.27 fast-forwards against 0.35.

### 3.3 Against the line's gate (model; unmeasured)

The relay's gate: with a build of about 8 c/B, the walk must come under about 4 c/B to
match #42's local T-vector build + room walk of about 12 c/B.
- **The T room walk** is about 16 instructions a step and a byte, and ran at 9.3 c/B in
  #42's local setting: about 0.58 cycles an instruction, instruction-bound.
- **#42's move room walk** (select) was about 41 instructions a step.
- **A two-hop step** keeps all of that (the state load and store, two adjacent record
  loads, the length compare, the selects, the flag test, the state update, the prefetch)
  and adds a second byte's extract and store. That is about 43–45 instructions for 2 bytes,
  **about 22 a byte**.
- **At the T walk's measured rate,** that is about 13 c/B, three times the gate. Even with
  a free build it would exceed the 12 c/B total.
- **To meet 4 c/B at that rate,** a two-hop step would need about 14 instructions or fewer.
  That is fewer than today's one-byte step, while doing a fast-forward select on 0.53 of
  steps, a record decode and two byte stores.
- **The build would not be cheaper than #42's:** it covers 1.2× as many cells, with an extra
  Ψ⁻¹ sweep and the markers, so the walk's budget shrinks below 4 c/B.
- **#34's dense pair walk** already showed the same arithmetic: half the steps, each about
  twice the work.

**Verdict on line B: stop at the census, as the relay's preflight rule says.** The
refinement is compact on Wikidata: 1.22r, a 0.96 B/row table, 862 kB a mean block. That is
the line's one real attraction, against the cache knee (E2 in the knee entry). But the
compact-Ψ evidence there already shows that a smaller table alone did not beat cooperative
production (u6/l4×2 0.687). And the walk's instruction budget is not plausible. No walker
prototype or registration is proposed.

## 4. What this closes, and what it doesn't

- **Closed, on this evidence:**
  - line A as a performance lead: correct, but no structural saving, and no memory saving;
  - line B before a walker, since it fails its own gate by the instruction model.
- **Not closed:**
  - the counts here are untimed, and the instruction figures are a model, never compiled
    or counted on hardware;
  - a host or schedule where the walk is memory-bound rather than instruction-bound would
    change line B's arithmetic. #42's reopening condition (four copies at once) covers it.
- **Kept:** the reference code and its tests. They pin down a fact that may matter to any
  later run-based work: the run-fed table *is* the production table, and the symbols give
  the runs exactly.

## Provenance and reproduction

- **The corpus:** `wikidata.bz2`, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`
  (bench/wikidata.json), a local copy built earlier by `bench/corpus.sh`. Its 431 blocks hold
  209,706,557 rows; the tables use the 233 blocks of ≥ 65,536 rows (209,697,662).
- **The other corpora:** scratch copies of tests/common-style corpora, provenance not
  re-established, so they are indicative only:
  - text `ce0e2c5f…99de9`;
  - records `ff8b5f6e…1f8c`;
  - random `10d444e6…3f18`.
- **Reproduce:**
  `MIBZ2_RUNFED_CORPORA=a.bz2:b.bz2 MIBZ2_RUNFED_OUT=census.tsv cargo test --release --lib run_fed -- --ignored --nocapture`.
  That gives one TSV line a block; the tests without the variable run in `cargo test`.
- **Base:** master `8743487d` (0.2.6). No timing of any kind was taken.
