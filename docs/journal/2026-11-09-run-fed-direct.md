# 2026-11-09: C31, the direct placement (arm D): production's one-hop T-vector from the run records, with no sort. A diagnostic registered (DRAFT, NOT FROZEN)

**Status: a draft registration, DRAFT, NOT FROZEN, NOT RUN,** revised for the
Researcher's REVISE (§9). The coordinator freezes it after review, or doesn't. Nothing here changes the decoder, with or without the `run-fed`
feature. Nothing here runs on the workstation.

**The input.** PROTOCOL §11 on #269's STOP (`2026-11-09-run-fed-stage-result.md`): the
run-fed stage lost 16% at the entry point on one worker, while its table fill (1.332 c/B)
was cheaper than production's build (1.432). The Optimiser's ranked handoff (#160 comment
6731728422, item 1) put C31 first:
> From emitted records, remove pair L reconstruction, first-symbol directory, fi/js
> composition metadata and pair-start/LF work; retain checked counts, prefixes, resumed
> cursors, F/L intersections, stores and ownership tails. Emit production
> T[q]=L[q]|Psi(q)<<8 and keep its room consumer. … Evaluate against #269's separate budget
> .218+delta_direct−e−f; do not transplant different-runner phase costs.

The Researcher's angle 1 (#162 comment 6731060416) gave the mechanism: direct F with
resumed cursors, one L predecessor cursor per symbol bucket, at most r − 1 advances and
O(256) state.

**In short:**
- **Built:** arm D, `run_stage::place` and `run_stage::build_direct`, behind `run-fed`. It
  takes #269's emitted records and places production's exact T-vector entries run by run,
  in L order, with no sort and no F-order array. Production's room walk consumes the table
  unchanged. The selector, the emitter and the fallback are #269's.
- **Correct:** the table word for word against `bwt::build` on every test column (9,841
  ternary columns, 100 random ones, the named cases below), on every stream of
  `tests/mtf_kernels.rs` (corrupt ones included, every error kind the same), and on all
  431 blocks of the bench corpus and the six synthetic corpora, with the CRC after the
  room walk. Under Miri, scalar and AVX2.
- **The diagnostic:** #269's harness, now with arms A (production), B (#269's records and
  sort) and D, paired by round, in one `sweep.yml` run (`tool: run_fed_stage`).
- **The reading set before any timed run:** Q1, R_direct = D / A at the entry point on
  Wikidata. FEASIBLE at ≤ 0.95 with hi < 1 and both screens; STOP at lo > 0.95.
- **The drafter's prior is STOP.** The budget says D must place 16% cheaper than B's
  sequential fill only to break even on the phase scale (§3).
- **What Q1 times:** the MTF stage and the table's construction, at the entry point. The
  room walk, RLE1 and the CRC that follow are untimed, so a change in what the walk costs
  after D's table is not in Q1 (§3).

Labels as in #266 and #269: **proved**, a short argument given here; **checked**, by
tests, as counts; **model**, an estimate with its assumptions; **local, indicative**, never
evidence.

## 1. The construction

### 1.1 What D is

The records are #269's: one `u32` a run of L, `p << 8 | c`, closed by `n << 8`, as
`mtf::mtf_runs` emits them. Production's table is `T[q] = L[q] | Ψ(q) << 8`, for q an F
row.

- **Proved: each L run maps to a contiguous F interval, and Ψ on it is a translation.**
  Row i of L with byte c goes to F row `C(c) + rank_c(i)` (the stable LF order that
  `build_one_pass` writes). So run (c, p, ℓ), the k-th c in L at its start, owns F rows
  `[a, a + ℓ)` with `a = C(c) + k`, and `Ψ(a + t) = p + t`.
- **The placement:** the runs in L order. Each bucket keeps its next row `a[c]`, so k needs
  no sort: it is the running sum of the bucket's earlier runs. For each run, its F
  interval is cut where an L run ends, and on each cell, L is constant and
  `T[q + i] = T[q] + 256·i`.
- **L[q]** is the byte of the L run that holds row q. Each bucket keeps one cursor into
  the L records: seeded once, at the run holding the bucket's first row (one binary search
  a nonempty bucket, narrowed from the last), and resumed run after run.
  - **Proved: a bucket's cursor only moves forward, and stays inside its bucket.** Its
    runs own rising rows, all in `[C(c), C(c+1))`. So the cursors cross disjoint row
    ranges: at most r − 1 advances in all after seeding, and 3 × 256 words of state. The
    seeds are at most σ ≤ 256 binary searches, O(σ log r) probes, so the whole cursor work
    is O(σ log r + r), not O(r + 256).
- **The stores** are #269's 8-lane AVX2 stores (base + 0, 256, …, 1792; then + 2048), or one
  slot at a time on the scalar kernel.
  - A vector store past a cell's end lands on rows of the same bucket above the cursor.
    Those are this run's later cells or the bucket's later runs, which are written after.
    So the overstore is repaired. **Proved**, for a table that passes the checks.
  - **The ownership tail:** within 8 rows of a bucket's end, the rest of a cell is stored
    one slot at a time. A vector store there could reach the next bucket's rows, which an
    earlier run may already have written (L = c¹⁰b¹⁰ writes c's bucket first), or pass n.
    So no store leaves its run's bucket, and no slack past n is needed.
- **The checks are `sort`'s, made on the way, before any store a run makes:**
  - n below 2²⁴, the first start 0, the closing record n;
  - the counts summing to n;
  - each start above the last;
  - each run's interval within its bucket.

  At the end every bucket's next row must equal its end. That is implied by the checks
  before it (the runs' lengths sum to n, each bucket's stay within it, and the buckets'
  sizes sum to n), and it is kept as the tiling stated once (§4, mutation 3). `place`
  returns false and an empty table on any failure. The cursors read ahead of the checks,
  in safe, bounds-checked code: on records that will fail, they can find wrong bytes, and
  that table is discarded.
- **`build_direct`** is `build` with `place` in place of `sort` then `fill`: `takes_runs`,
  then `mtf_runs` and `place`, or the column's path. The fallback is #269's, unchanged.

### 1.2 What D removes, against B

| B (#269) | D |
|---|---|
| `sort`'s first pass over the records, counting runs a byte | gone |
| `sort`'s scatter into an F-order array, `[a, p]` a run: 8 bytes a run written, about 0.50 B a row at Wikidata's r/n of 0.062 | gone: no F-order array, and no `Scratch` |
| the fill's F iterator: two loads of the F order a cell | gone: the run's own p and its bucket's `a[c]` |
| one sequential sweep of the rows | up to 256 interleaved destinations, one a nonempty bucket, in L order (as `build_one_pass` scatters); the record counts the nonempty buckets, not measured hardware write streams |

**What D keeps, as the handoff specifies:** the checked counts, the prefixes, the resumed
cursors, the F/L intersections (the cells: the same cuts as B's, so the same cells, at most
2r − 1; the validation checks the counts equal on every block), the stores and the
ownership tails.

**The same cells are not the same stores.** B overstores across bucket boundaries, since
its sweep rises through every row and repairs them. D can't, so near a bucket's end it
trades vector overstores for one-slot stores. For a D cell of ℓ rows from q in a bucket
ending at b:
- `v = min(⌈ℓ/8⌉, ⌊(b − q)/8⌋)` vector stores;
- `max(0, ℓ − 8v)` one-slot stores, at most 7 rows a bucket.

So D may write fewer slots than B while issuing more table stores, and a narrow bucket can
be all one-slot stores. `Tally.slots` stays B's fill's count. The record now holds D's own
census too (`direct_census`, independent of `place`, the same rule), summed over the
records' blocks:
- its **predicted table stores**, vector and one-slot: the source's store loops counted,
  not machine code. Until a disassembly, they are not all of D's machine stores: spills,
  scratch state and the compiler's lowering are not counted;
- its **nonempty destination buckets**: where its stores go, not a measurement of
  concurrently active hardware write streams or of write-buffer pressure.

The binary and the verifier report D's predicted table stores a row, its slots written a
row against B's, and its nonempty buckets a block.

**What D adds:**
- per-run bucket state, `a[c]`, `end[c]` and `cur[c]`: loads and stores that chain through
  memory when a byte's runs follow closely;
- the cursor's reads of the L records at the bucket's position, 256 read streams over the
  records instead of one;
- the seeds: at most σ ≤ 256 binary searches a block;
- the one-slot tails within 8 rows of each bucket's end, measured by the census.

Whether that costs less than what it removes is the question. No model here answers it.

### 1.3 Safety

`place` is a safe function. Its one `unsafe` region is `place_rows`'s stores, through a raw
pointer into the table's reserved capacity, and `set_len` after a pass.
- Every store is below its run's bucket's end, checked before the run's first store, so
  below n. A vector store is made only when `w + 8` is at most the bucket's end.
- Each cell ends past its start, so the loop ends: the cursor stops at a record whose
  start is past q, and the closing record is n, past every q.
- The AVX2 instance runs only under `Kernels`' detection, as `fill`'s does.
- `Kernels`' public fields let safe code claim AVX2 on a CPU without it. That is the crate's
  existing contract (#269 §3), not changed here.

An independent adversarial review of this `unsafe` (RULES B5) is owed, as #269's is. The
Guardian decides whether it must come before the diagnostic run.

## 2. The diagnostic

`examples/run_fed_stage.rs`, #269's harness, extended, at schema `run_fed_stage/2`:
- **The arms:**
  - **A, the column:** `mtf::mtf`, then `bwt::build`.
  - **B, the records (#269):** `run_stage::build`.
  - **D, the direct placement:** `run_stage::build_direct`.

  #269's arm C (the checkpointed MTF, its Q2) and its store reference (M1, M2) are
  dropped. Their readings stand in #269's record, at `fd60fffa`. Three arms rotate by
  round.
- **Everything else is #269's:**
  - the same input, scanned once;
  - the same kernels;
  - each arm's own buffers, the walk written over its `ll`;
  - decode-order caches, with the room walk, RLE1 and the CRC untimed after every block;
  - the entry sweep, one bracket a block, for the judged figures;
  - the phase sweep, for attribution;
  - the record, raw times arm by arm.
- **D's phases** are `front` (the emitter, #269's), `build` (the placement) and `fallback`
  (the column's stage on the blocks `takes_runs` refuses). Its `sort` is exactly 0, and the
  verifier requires it to be.
- **The record** gains each corpus's decoded `output` size, the c/B figures' denominator,
  so the verifier can compute the budget's terms (§3) from the raw times itself, and D's
  store census (§1.2): `d_vector`, `d_scalar`, `d_streams`.
- **The timed paths (since §9):** on every timed pass, entry and phase sweeps alike, B's
  and D's actual path must be `takes_runs`'s, block by block. So a D fallback in any
  round, a failed placement for instance, stops the run instead of being billed to the
  records' bin. Every timed pass also checks every block's CRC.
- **What is timed:** each bracket holds the arm's MTF stage and its table construction,
  at the entry point (the selector, the checks and the fallback included). The room walk,
  RLE1 and the CRC after each block are outside every bracket. They keep the caches in
  decode order; they are not measured.
- **Validity, before any timing:** #269's checks, plus, on every block:
  - D's table, from the records by `place`, is A's word for word;
  - D's stage, through `build_direct`, has A's length, its path is `takes_runs`'s, and its
    table is A's;
  - the room walk of D's table, then RLE1, gives the block's CRC.

  Every timed pass checks every block's CRC again.

**The ratios** are #269's form: arm over arm, paired by round, the geometric mean and its
95% t-interval on the logs (df 14). A ratio with any round absent is absent, never a pass.
- From the entry sweep:
  - `R_direct`, D / A, every block (Q1);
  - `R_stage`, B / A, #269's Q1, repeated on this run;
  - `R_DB`, D / B. It compares two whole constructions: B's sort and sequential fill
    against D's placement, with its checks, seeds, cursor schedule, stores and tails. It
    is not an isolated treatment of removing the sort or the F array;
  - `R_acc_D` and `R_fb_D`, D / A on the records' and the fallback's blocks.
- From the phase sweep, attribution only:
  - `R_phases_D`, D / A, the phases' sum;
  - `R_place`, D's placement / A's build;
  - `R_pf`, D's placement / B's fill;
  - `R_psf`, D's placement / B's sort and fill;
  - `R_front`, D's emitter / A's MTF.
- **The budget's terms,** in nominal c/B of output, each round's own, reconstructed a
  round at a time, then summarised (§3).
- **D's store census** (§1.2), reported.

## 3. The budget

Set against #269's own measured phases, on its runner (Wikidata, the pinned copy, nominal
c/B of decoded output: wall time × the sampled clock):

| | front | sort | build / fill | fallback | phase sum |
|---|---|---|---|---|---|
| A | 1.290 | — | 1.432 | 0.971 | 3.693 |
| B | 1.508 | 0.317 | 1.332 | 1.071 | 4.228 |

D replaces B's sort and fill with its placement. So, on the phase sweep's scale:

**Δ_D ≡ D − A ≈ .218 + δ_direct − e − f**, where:
- **.218** is #269's B − A phase-sum gap (.535) less its sort (.317). It is what B would
  lose with the sort removed for free and the fill unchanged.
- **δ_direct** is D's placement less B's sorted fill (1.332), on the same blocks. It carries
  everything the sort did that D still does: the prefixes and checks, now inline; the
  cursors and their seeds; up to 256 destination buckets; and the ownership tails. Its sign
  is open: negative if the removed F order and its iterator cost more than what D adds.
- **e** is the emitter's recovery, B's front less D's.
- **f** is the fallback's recovery, B's fallback less D's.

**e and f are measured controls, not invariants.** D runs #269's emitter and fallback code,
but identical code does not make their times identical: a different table write before
them, other retained buffers, code layout, the caches and the arms' history can all move
the next emitter's or fallback's time. Zero is the reference model's assumption, and the
run measures both.

**Each round's identity, before any summary.** With D's sort exactly 0, each round i of the
phase sweep satisfies, exactly:

    (D − A)_i = K_i + δ_i − e_i − f_i,   K_i = (B − A)_i − B's sort_i,

where δ_i, e_i and f_i are that round's own terms. The binary and the verifier reconstruct
the right side a round at a time, check it against the left (the binary asserts it, the
verifier reports the largest residual), and only then summarise: the median of each term,
the median of (D − A)_i, and the median of .218 + δ_i − e_i − f_i. The medians of the terms
need not sum to the median of the total, and a gap between them is no missing phase.

**What that requires of D, on the phase scale (a sensitivity, not a prediction):**
- **To break even** (Δ_D ≤ 0): δ_direct ≤ −.218, a placement at or below 1.114 c/B. That is
  16% below B's sequential fill, and 22% below production's build.
- **To meet Q1's gate** (D ≤ 0.95 of A): Δ_D ≤ −0.185 (5% of 3.693), so δ_direct ≤ −.403, a
  placement at or below 0.929 c/B. That is 30% below B's fill, and 35% below production's
  build.

**What the budget is not:**
- It is not the judged figure. #269's entry gap (.615) was larger than its phase-sum gap
  (.535): the phase sum is not the entry point's cost, and separate medians don't make a
  paired ratio. The judged figure is R_direct, at the entry point.
- It doesn't transplant #272's phase costs, which came from a different runner, a different
  consumer and different admission.
- This run's own figures replace #269's where they differ. The binary and the verifier
  print δ, e, f and D − A from this run's raw times, beside .218 + δ_i − e_i − f_i, and this
  run's own K, for comparison with .218.
- **It is not the walk.** Q1 and the budget time MTF and the table's construction. The
  room walk, RLE1 and the CRC are untimed. If D's table leaves the caches in a different
  state for the walk that follows, that cost or saving is not in Q1. A production
  comparison must charge it.

## 4. Correctness (checked)

| check | where | result |
|---|---|---|
| D's table against `bwt::build` (`run_fed::production`), each kernel, into a dirty table: every ternary column of 0–8 rows (9,841 columns) and the random run-heavy and uniform columns of 7–70,000 rows over σ 1–256 (100 columns) | `run_stage::tests::check`, through `direct` (`tables_from_records_small`, `tables_from_records_random`) | pass |
| on each of those columns, each broken tiling refused, the table left empty: n ± 1, a count one less and one more, a count moved to another byte, the closing record moved, a run's start moved, two records swapped | the same | pass |
| the named cases: the empty block and the tiny ones (a, ab, ba, aa, abba, bba, aab); one symbol at every length 1–40 and at 70,000; all 256 symbols, once each (r = n) forward and reversed, and as runs of 1–20; r = n by alternation and by a 3-cycle; runs crossing bucket boundaries; the ownership tail, `c^len b^len` for len 1–24, with and without later runs of b | `run_stage::tests::direct_cases` | pass |
| the states D must never accept: no records for n 0 and 1, the closing record alone, the empty block's records for one row, n at 2²⁴, another block's n (0, 1, 3, 5), records out of order, two runs at one row, a first run past row 0, an uncounted byte, counts that wrap a u32, a run past its bucket's end; each refused with an empty table, by both kernels; the empty block accepted with an empty table | `run_stage::tests::miri_direct_unchecked_states_refused` | pass, natively and under Miri |
| `place`'s stores under Miri: `miri_sort_and_floor` (ternary columns to 4 rows, 121, now through `direct` too) and the test above, scalar, then with `-C target-feature=+avx2,+bmi1` and `MIBZ2_EXPECT_AVX2` asserting the AVX2 kernel was reached | `cargo +nightly miri test --lib -- run_stage::tests::miri_`, both of `miri.yml`'s steps | pass, both (3 tests each) |
| `place` and `build_direct` on every stream of `tests/mtf_kernels.rs`: the table on success; on a corrupt stream, `build_direct`'s error kind is `mtf::mtf`'s; its path is `takes_runs`'s | `cargo test --features run-fed --test mtf_kernels` (4 tests) | pass |
| all 431 blocks of the bench corpus (`wikidata.bz2`, SHA-256 `82f65ace…`) and the six synthetic corpora: D's table by `place` and through `build_direct`, path, length, the CRC after the room walk; D's census cells equal to B's, its stores between its rows and B's slots, its one-slot stores at most 7 a stream; the Wikidata file's output = `decode`'s; then, over 2 timed rounds, B's and D's path on every timed block and each round's budget identity | the diagnostic's validation pass, run twice here (§7) | pass |
| the record writer against the verifier's fixture, byte for byte (rewritten for schema 2, with real corpus identities since §9); D's census on two hand-counted cases (`direct_census_counts_the_tails`) | `cargo test --features run-fed --example run_fed_stage` (5 tests) | pass |
| the verifier: the fixture VALID against its evidence; each defect INVALID, now including D's placement untimed, a sort time for D, the old schema, the old arms and no output size; since §9 and §9.1, the 31 counterexamples of `bench/fixtures/run_fed_stage_counterexamples.json` (the Researcher's six and the closure review's four among them) each INVALID, and the first six by the CLI with no reading printed; the Wikidata file's own digest and blocks against the spec; no corpus evidence; the regenerated synthetic corpora equal to the pinned five; the CLI VALID only once the text corpus is this checkout's; the readings: Q1 follows D and not B, the 0.95 gate, INCONCLUSIVE on a wide 4% saving, the budget's terms | `python3 bench/test_run_fed_stage.py` (50 tests) | pass |
| the real record of the local validation run, against the verifier's own evidence (the Wikidata file, the synthetic corpora regenerated from this checkout) and every count relation | `bench/run_fed_stage.py verify` | INVALID only for its 2 rounds and its 4-CPU affinity, as expected |
| the existing run-fed tests, unchanged: `sort`, `fill`, the shares, the refusals | `cargo test --features run-fed --lib run_stage` (6 tests, 2 of them new) | pass |
| clippy (`--all-targets --features run-fed`, `-D warnings`), rustfmt, ruff | locally | clean |

**Mutation (checked):**
1. The ownership tail removed (vector stores up to n instead of the bucket's end): caught by
   5 of 6 `run_stage` tests.
2. Ψ off by one on the run whose interval starts at F row 40: caught by `direct_cases` and
   `tables_from_records_random`.
3. The final tiling check removed: survives. This is an equivalent mutant: the check is
   implied by the ones before it (§1.1), and is kept as the tiling stated once.

**Not covered:**
- **No fuzzing.** `examples/fuzz.rs` on this base has no `run-fed` mode. #270's branch adds
  one, and #270 is held under D049; it is not ported here.
- **No disassembly.** Whether `place_avx2`'s `#[inline(always)]` core has its intrinsics
  inlined, as `memchr`'s pattern does, has not been checked in the fat-LTO build. If it had
  not, D would be slower than its design, not wrong.
- **The B5 review of `place`** (§1.3).

## 5. The registration (RULES C1, diagnostic form). DRAFT, NOT FROZEN

- **The information objective** (C1's diagnostic option: it can never qualify a production
  change): whether placing the records directly, with no sort, makes the run-fed stage pay
  at one worker on one CI runner. Q1 judged, the rest exploratory.
- **The baseline identities (full):**
  - master `6b36fd8c43c1b834e69ee62cd6b5054dc99c571c`, merged into this branch. Arm A,
    `mtf::mtf` then `bwt::build`, is its code, unchanged;
  - #269's frozen head `fd60fffada25d475bbcc2b54165b96cf5fd7a92f`. Arm B, `run_stage::build`
    with `mtf::mtf_runs`, `sort` and `fill`, is its code, unchanged: `src/mtf.rs` and
    `src/bwt.rs` are byte-identical to fd60fffa's on this branch, and `run_stage.rs` only
    adds D;
  - #269's judged reading stays its own: the 0.90 gate and the STOP it read at fd60fffa.
    The 0.95 gate below is this registration's, for D, and is not applied to B.
- **The head:** this PR's head at freezing, which the coordinator names in full (a
  registration can't name its own commit) and dispatches as `sweep.yml`'s
  `registered_head` input.
- **The synthetic input is the frozen head's.** The text corpus is made from the head's
  `src/`, `tests/`, `examples/` and `README.md` (tests/common's `corpora(1)`), so any change
  to those changes it. At this revision's sources (§9.1) it is 3,326,302 bytes, compressed
  at level 9 to 612,142 bytes, SHA-256
  `5996a32dcec1cb7899cd2dd2e2cb122a3de7eeead4c2096729575e3aec79e6ff`, 4 blocks. (At §9's
  sources, `7f88388b`, it was 3,324,678 bytes, `b4c2767e…`.) The other
  five don't depend on the head (§4's pinned values). A docs-only commit changes none of
  them. The workflow refuses to run if the checkout's sources differ from the head, and the
  verifier regenerates every synthetic corpus from that checkout, so the judged record
  must carry the frozen head's own.
- **The run:** `sweep.yml`, dispatched on this branch at the frozen commit: `tool:
  run_fed_stage`, `rounds: 15`, `registered_head` the frozen SHA. One dispatch.
- **The primary condition** is the one copy pinned to one CPU. The contended run is
  reported, never judged.
- **The estimand:** per-round paired ratios of the entry sweep's times, the geometric mean
  and its 95% t-interval (df 14).

**Admission: VALID or INVALID** (`bench/run_fed_stage.py verify run-fed-1.json --head
<frozen SHA> --wikidata corpora/wikidata.bz2`, run by the workflow). #269's list, with
schema 2:
- the schema `run_fed_stage/2`; the arms exactly `A column`, `B records`, `D direct`; the
  rotation of three;
- every corpus's decoded output size recorded and positive;
- D's front and placement positive wherever its corpus has records' blocks, and D's sort
  exactly 0, as A's is;
- **the clock (since §9):** `ghz` a finite number, not a boolean, in [0.1, 10];
- **the corpus evidence (since §9), the verifier's own, never the record's:**
  - Wikidata's SHA-256, size, decoded size and blocks equal to its pinned spec,
    `bench/wikidata.json` (corpus_bytes 11,854,933, decoded_bytes 209,700,196, 431
    blocks, its SHA-256 the registered one); and the file given by `--wikidata` (its
    SHA-256, size, and blocks counted from their 48-bit headers) equal to the spec;
  - each synthetic corpus's SHA-256, size, decoded size and blocks equal to its
    regeneration from the checkout (`--sources`; tests/common's `corpora(1)` ported to
    Python, compressed at level 9 with libbzip2, as `common::compress` does);
- **the count relations (since §9 and §9.1),** each a sum of per-block bounds the
  representation fixes (`count_relations`): 1 to 900,000 rows a block, on each path; on
  the records' blocks, records from their blocks to their rows; cells from r to 2r −
  blocks (at most 2r − 1 a block); B's slots a multiple of 8, from its rows to its rows + 7
  a cell, and at least 8 a cell; D's written slots from its rows to B's slots, with at
  least one predicted table store a cell and at most 7 one-slot rows a bucket; 1 to 256
  nonempty buckets a block, and no more than its runs; RLE1's 4–259 bytes for 5 rows,
  with each block's last 4 rows unpaired at most;
- D's census (`d_vector`, `d_scalar`, `d_streams`) recorded;
- otherwise #269's: the record exists; the full head, equal to the registered one; 15
  rounds; AVX2; correctness passed; the host; one CPU; the seven corpora by name and
  SHA-256, Wikidata's the registered `82f65ace…c29915` and the file's own; consistent path
  counts; every registered interval present.

An INVALID record has no reading: the verifier decides admission, every check above
included, before it computes or prints any reading.

**Q1, the gate (judged).** R_direct = D / A at the entry point, every block, on the
Wikidata file:

| reading | condition |
|---|---|
| **FEASIBLE:** a one-worker production prototype of D may be registered, on its own | R_direct ≤ 0.95 and its hi < 1, **and** both screens pass |
| **STOP: no saving of 5% or more by D shown** | R_direct's lo > 0.95 |
| **STOP: a screen failed** | R_direct ≤ 0.95 and its hi < 1, but a screen fails |
| **STOP, INCONCLUSIVE** | anything else |

The screens are #269's, read on D:
- **the synthetic whole-stage screen:** every synthetic corpus's R_direct geometric mean
  ≤ 1.02;
- **the fallback screen:** every corpus with fallback blocks has R_fb_D (D / A on the
  fallback's blocks, at the entry point, the selector charged) geometric mean ≤ 1.02.

They are point-estimate screens, reported, and explicitly not a noninferiority claim
(#269 §4). Any production prototype settles fallback noninferiority under its own
registration (C6).

**The thresholds, and why they differ from #269's.**
- **The stop, lo > 0.95, is #269's,** unchanged.
- **The gate is 0.95, where #269's was 0.90.** C31 is a construction on a line Louis is
  pursuing (PROTOCOL §11). Its question is whether removing the sort makes the stage pay
  at all. A 5% one-worker stage saving with its interval below 1 is enough to justify
  drafting a production prototype's own registration, which sets its own end-to-end
  threshold. FEASIBLE licenses nothing more.
- **Set before any timed run of D.** No timing of D has been seen (§7).
- **What FEASIBLE means.** Some saving at the stage's entry point, with a 5% point estimate
  and an interval below 1; not a confident 5% lower bound, not a whole-decoder claim, and
  not production acceptance. It only motivates another registration. A real 4% saving may
  properly read STOP for this 5% objective: "no saving of 5% shown" is not "no speedup".

**The exploratory readings,** each reported with its interval, and none judged:
- **R_DB, D / B at the entry point:** "D CHEAPER THAN B" if hi < 1; "D NOT CHEAPER THAN B" if
  lo ≥ 1; else "D AGAINST B, INCONCLUSIVE". It compares D's whole placement with B's sort
  and fill on this runner, not the sort's removal in isolation (§2).
- **The budget's terms** (§3), each round's own and their medians: δ, e and f (e and f
  measured controls), the identity's largest residual, D − A's phase sum beside the median
  of .218 + δ_i − e_i − f_i, and this run's own K beside #269's .218.
- **D's store census** (§1.2): predicted table stores a row, slots written a row against
  B's, and nonempty destination buckets a block.
- `R_place`, `R_pf`, `R_psf`, `R_phases_D`, `R_front` and `R_stage` on this run.

**Voids and failures** are #269's:
- **A void run** may be re-dispatched once at the same commit: a failure in setup, build or
  corpus fetch before the binary starts; a runner without AVX2; or a runner lost or
  cancelled mid-run.
- **A correctness assertion failure is not a void.** A validity panic, a CRC mismatch, a
  path disagreement or no record written starts an investigation under B6, with no retry.
- **An INVALID record** for any other reason is investigated likewise, never re-run blind.
- No other re-run, no workstation, no ready tag, no queue entry.

**The positive and negative stop criteria.**
- **Positive:** Q1 FEASIBLE. It permits drafting a one-worker production prototype's own
  registration, with a pre-set end-to-end speed threshold, CI's paired bench, B5's review
  and its own fallback noninferiority rule (C6). It may also justify preparing a separately
  frozen i9 diagnostic for Louis to run by C5. No workstation run is permitted by this
  registration.
- **Negative:** Q1 STOP or INCONCLUSIVE. That pauses this implementation of C31 on this
  runner: a journal entry and the queue issue's Rejected list, at the coordinator's
  choice. Under PROTOCOL §11 it does not close the run-fed line. The measured phases, and
  δ_direct above all, go to the Optimiser and the Researcher, with the question "which
  construction removes the cost that bound this arm?"

**Cost:** one `sweep.yml` job: about 5 minutes of build, and runs about three-quarters of
#269's (three arms where it had four).

**What CI can answer.** The one-worker question on its recorded runner, which the record
names. Not the i9's, and not a pipelined team's: D's placement is serial here, and how it
would split across helpers (each bucket's cursor is independent, so shares by bucket are
natural) is not measured.

## 6. The reading: the drafter's prior

**The prior is STOP: R_direct above 1, likely between about 1.03 and 1.12.**
- **For a gain:** D drops a pass over the records, an 8-byte-a-run array, and the fill's
  F-order loads. The fill already beat production's build.
- **Against:**
  - The budget needs D's placement 16% below B's sequential fill only to break even, and
    30% below it for the gate. Most of B's fill cost is its cells and stores. D keeps the
    same cells and the same 8-lane store kernel, but not the same stores: near each bucket's
    end it trades overstores for one-slot stores (§1.2).
  - D gives up the fill's one sequential write order for up to 256 interleaved destination
    buckets. Wikidata's
    runs average about 16 rows (r/n 0.062), so a run is about one 64-byte line: roughly
    `build_one_pass`'s own 16-row scatter, which costs 1.432.
  - D adds per-run state chained through memory (`a[c]`, `cur[c]`), and its cursor reads
    the records at 256 positions instead of one.
  - D runs the emitter and fallback code whose losses #269 measured (+0.218 against plain
    MTF; +0.100), unchanged. Their times in this run are measured controls (e and f, §3),
    not assumed equal to B's.
- **Rough odds, a judgement and not a model:** STOP (lo > 0.95) about 85%; INCONCLUSIVE
  about 10%; FEASIBLE about 5%. R_DB below 1 is more likely than not, about 60%. D is
  likelier to beat B than A, because it removes the sort's 0.317 outright and has to give
  back less than that.

## 7. Disclosures

- **Local timings are not evidence.** None is reported here, and none chose anything.
- **One local run, for validity only.** The diagnostic was run once on the shared machine,
  at 2 rounds on the bench corpus, to check that validation passes on every block and that
  the record's shape admits. Its output was filtered to the validation and path-count lines
  before the drafter read it: 431 blocks, 384 by records (79.8% of rows, r/n 0.0621), 47
  fallback. Its record was checked against the verifier only for its problems (2 rounds;
  4 CPUs), which print no figures, then deleted. **No timing figure of D, B or A from
  that run was read.** It is excluded from every estimate.
- **Miri, the tests and the mutations** are correctness runs. Their times are not
  measurements.
- **The thresholds** (§5) were written before any timed run of D.
- **A second local run, for §9's validity only,** the same way: 2 rounds, its output
  filtered to the validation and path-count lines (the same 431 blocks, 384 by records, 47
  fallback; the six synthetic corpora's SHA-256s, each equal to the verifier's
  regeneration), its record checked by the verifier for problems only (its 2 rounds and 4
  CPUs, nothing else), then deleted. No timing figure was read.
- **A third and a fourth, for §9.1's validity only,** the same way, before and after its
  wording change to `examples/run_fed_stage.rs`: each record checked by the verifier, with
  its own evidence and the four new bounds, for problems only (its 2 rounds and 4 CPUs,
  nothing else), then deleted. No timing figure was read.

## 8. Not done, and why

- **The freeze and the dispatch** are the coordinator's, after review.
- **The B5 adversarial review** of `place` (§1.3) is the Guardian's or an independent
  reviewer's to make, and the Guardian decides its timing.
- **Fuzzing:** there is no `run-fed` mode in `examples/fuzz.rs` on this base (§4).
- **Disassembly** of the inlined AVX2 core (§4).
- **A pipelined or helper-side D** (#270's question, held under D049): not built, not
  measured.
- **The emitter and the fallback,** the Optimiser's items 2 and 3, are unchanged here by
  design. That does not make D an isolated test of the sort's removal: D/B compares D's
  whole placement with B's sort and fill (§2).

## 9. Revision, 2026-11-09: the Researcher's REVISE (#162 comment 6732062626)

The review of head `705323290b5470dea4a31910c22522c342e9d064` (report:
`state/reports/direct-f-registration-6731990737/report.md` at `6bc26d28`). It accepted the
construction, the budget's arithmetic, the 0.95 gate and dropping arm C and the store
reference. Each finding, its fix and the test that holds it. No timing was run for the
revision.

| # | finding | fix | test |
|---|---|---|---|
| 1a | ghz −1, 0 or `true` admitted, and read FEASIBLE with wrong budget terms; a missing ghz passed `check()` and raised a KeyError in the readings | `ghz` must be a finite, non-boolean number in [0.1, 10] GHz; checked with the rest, before any reading | counterexamples 1–4, 7, 8 (`bench/fixtures/run_fed_stage_counterexamples.json`), each INVALID; 1–6 also through the CLI, with no reading printed |
| 1b | output = 1 admitted, scaling δ to about −60,032,000 c/B | every corpus's SHA-256, size, decoded size and blocks checked against the verifier's own evidence: Wikidata's pinned spec and file, the synthetic corpora regenerated from the checkout (§5) | counterexamples 5, 10–13; `test_the_wikidata_files_own_digest`, `test_the_wikidata_files_blocks`, `test_no_corpus_evidence`, `test_the_regeneration_matches`, `test_the_cli` |
| 1c | rows_runs = 1 with 999,999,999 records and cells admitted | the count relations, per-block bounds summed (§5, `count_relations`) | counterexamples 6, 9, 14–25 |
| 1d | INVALID decided before any reading | it is: `main` prints problems and returns before `readings`, and every field `readings` reads is now checked | `test_the_counterexamples_print_no_reading` |
| 2 | the timed sweep reconciled only B's path | B's and D's actual path asserted on every block of every timed pass (§2) | the validation run: 2 timed rounds, every block |
| 3a | Q1 times MTF and construction, not the walk | said so (In short, §2, §3) | — |
| 3b | D/B is no isolated sort-removal treatment | said so (§2, §5) | — |
| 3c | identical code doesn't force e = f = 0 in time | e and f are measured controls (§3) | — |
| 3d | a sum of medians is not the median of the total | each round's identity reconstructed and checked before any summary, then each term's median and the total's (§3); the binary asserts the identity a round, the verifier reports its largest residual | `test_the_budget_terms` |
| 4 | the same cells are not the same stores; `Tally.slots` is B's | D's census: vector stores, one-slot stores and write streams, counted independently of `place` and recorded; checked against B's cells and slots on every block; reported a row and a block (§1.2) | `direct_census_counts_the_tails`; the validation pass; counterexamples 19–22 |
| 5 | the freeze identities | the full baseline identities, #269's own rule kept for B, and the frozen head's synthetic input named (§5) | — |
| — | the seeds' cost | O(σ log r) probes for the seeds, then at most r − 1 advances (§1.1) | — |

**Still open:** the B5 review of `place` (the Guardian's, queued), and fuzzing (§4, Not
covered).

### 9.1 The closure review's narrow REVISE (#162 comment 6732362367)

The review of head `7f88388b938528f22088df79387f309eb112fe1c` (report:
`state/reports/direct-f-closure-6732258318/report.md` at `409907ca`). It kept §9's closures.
Four necessary count bounds were missing, and the unmutated text fixture was itself
infeasible.

| # | finding | fix | test |
|---|---|---|---|
| 1 | four necessary bounds missing: `blocks_fallback ≤ rows − rows_runs ≤ 900,000 · blocks_fallback`, `d_streams ≤ records`, `8 · cells ≤ slots`, `cells ≤ d_vector + d_scalar` | all four added to `count_relations`, checked before any reading (§5) | counterexamples 26–29, the review's four witnesses (zero fallback blocks but a fallback row; 42.4 M fallback rows in 47 blocks; one run, 256 nonempty buckets; too few B slots and D stores for the cells), and 30–31, each bound alone: each INVALID |
| 2 | the passing fixture's text corpus declared 1.5 M cells with 5 M B slots and 562 k D stores (it needs at least 12 M and 1.5 M); the runs corpus likewise needed 192 k slots against 160 k | the record writer's fixture table and the JSON fixture fixed together: text 12.8 M slots, 1,502,000 D stores; runs 200 k slots, 24,500 D stores. Every corpus of the fixture keeps every bound, so one feasible record still passes | `test_the_fixture_is_valid`; `the_verifiers_fixture`, byte for byte |
| 2b | the real binary's record against the new bounds | two more 2-round local validity runs, before and after the wording change: the verifier, with its own evidence, finds only the 2 rounds and the 4 CPUs (§7). No timing read | — |
| 3 | the census called machine stores and hardware streams; §6 and §8 contradicted §§1–3 | the census is D's predicted table stores (the source's store loops, until disassembly) and its nonempty destination buckets, not measured hardware streams (§1.2, the binary's and the verifier's labels); §6 and §8 corrected | — |

The wording changed `examples/run_fed_stage.rs`, so the synthetic text corpus changed with
it; §5 names the new one.

## Provenance and reproduction

- **Branch:** `research/run-fed-direct`, from `research/run-fed-stage` at `fd60fffa`, with
  master merged in (`miri.yml`'s path filters and comment were reconciled by hand), and
  master `6b36fd8c` merged again for §9 (a journal-only change).
- **Corpus:** `wikidata.bz2`, SHA-256
  `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, as #266's and #269's.
- **The code:** `src/run_stage.rs` (`place`, `place_avx2`, `place_rows`, `build_direct`, and
  their tests); `tests/mtf_kernels.rs` (`runs_agree`); `examples/run_fed_stage.rs` (arms A,
  B, D; schema 2); `bench/run_fed_stage.py` and `bench/test_run_fed_stage.py`;
  `bench/fixtures/run_fed_stage.json` and `bench/fixtures/run_fed_stage_counterexamples.json`.
- **Tests:**
  - `cargo test --features run-fed --lib run_stage`;
  - `cargo test --features run-fed --test mtf_kernels`;
  - `cargo test --features run-fed --example run_fed_stage`;
  - `python3 bench/test_run_fed_stage.py`;
  - `cargo +nightly miri test --lib -- run_stage::tests::miri_`, then again with
    `RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1
    MIRIFLAGS=-Zmiri-env-forward=MIBZ2_EXPECT_AVX2`.
- **The diagnostic:** on CI, Actions → Walk sweep → `tool: run_fed_stage`, `rounds: 15`,
  `registered_head` the frozen SHA. Locally, `cargo build --release --features run-fed
  --example run_fed_stage`, then `RUN_FED_RECORD=run-fed-1.json RUN_FED_HEAD=$(git rev-parse
  HEAD) target/release/examples/run_fed_stage wikidata.bz2 15`, then `python3
  bench/run_fed_stage.py verify run-fed-1.json --head <SHA> --wikidata wikidata.bz2` from
  the same checkout (its `--sources` default).
- **#269's record** is schema 1, and this verifier refuses it. #269 is judged by the
  verifier at `fd60fffa`.
