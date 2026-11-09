# 2026-11-09: The run-fed stage (line A), reconciled, built opt-in, and a diagnostic registered (NOT FROZEN)

**Status: a draft registration, NOT FROZEN, NOT RUN,** revised for the Researcher's REVISE
(§7). The coordinator freezes it after re-review, or doesn't.
Nothing here changes the decoder, with or without the new feature. Nothing here runs on the
workstation.

**The input.** #145 comment 6728860546, a proposal from Louis via ChatGPT ("Can we maybe
implement something for that? Can we start getting that ready if it's not already ready?").
It asks for a bounded, opt-in prototype or diagnostic of line A, the T-vector filled from MTF
run records. It asks first for #266's objections to be reconciled
(`2026-11-08-run-fed-tvector-and-psi2-census.md`), and for an honest stop if they rule it out.

**In short:**
- **As a speed change on its own motivation, line A is unjustified.** #266's model holds: the
  run-fed build does about as many instructions a row as `build_one_pass`, it writes the same
  4n-byte table, and it saves no memory. Nothing in the reconciliation finds a removed pass.
- **What is still open is cycles, not instructions.** No model here or in #266 sees where the
  build's cycles go. Three questions about that can be settled by measurement, cheaply, on CI:
  1. the whole one-worker stage (Q1, the gate);
  2. the pipelined leader's serial MTF stage, which records free of its count checkpoints (Q2);
  3. what bounds the build: its table's write traffic, or its own work (M1–M3).
- **Built:** the stage, production-shaped, behind the non-default `run-fed` feature:
  - an AVX2 run emitter, `mtf_avx2`'s loop with only its stores changed;
  - a checked counting sort into F order;
  - an AVX2 8-wide sequential fill;
  - a fallback to the column;
  - and a diagnostic binary, `examples/run_fed_stage.rs`, for CI's existing `sweep.yml`
    dispatch.
- **Correct:** the table word for word against production on every test column, on the
  stage's MTF streams (corrupt ones included, every error kind the same) and on all 431 bench
  blocks; the CRC and `decode`'s output; `sort`'s writes and the AVX2 fill under Miri. The
  AVX2 emitter is not under Miri (§3).
- **Safe by construction since §7:** `fill` and `fill_rows` take a `Sorted`, which only `sort`
  makes, so no safe caller can reach the AVX2 fill's unchecked reads on unchecked state.
- **Recommendation:** freeze and dispatch the one CI diagnostic, if the coordinator wants
  line A closed on measured costs rather than a model. The drafter's prior is that Q1 reads
  STOP. No workstation run is proposed. #258 (`bench-service-2x3`) is the experiment that is
  ready to run; this one is not a candidate for the workstation queue.

Labels as in #266:
- **proved**, a short argument given here;
- **checked**, by tests, as counts;
- **model**, an estimate with its assumptions;
- **local, indicative**, a timing on the shared machine, never evidence.

## 1. #266's objections, reconciled

Each objection, and whether it rules out a win.

| # | objection (#266) | rules out a win? | what is left |
|---|---|---|---|
| 1 | 82% of Wikidata's rows already go through `build_one_pass`'s 16-row run stores | **No, but it takes the motivating mechanism.** The hoped-for "one monotone fill instead of scattered streams" is mostly taken: 82% of rows are already stored as 64-byte lines | the 18% scattered rows, the column's n-byte write in MTF, and the build's reads of L |
| 2 | about 2.6 instructions a row today against about 2.5 run-fed (model) | **Not as a proof (§7).** It is a source-shape model, not compiled counts, with unspecified uncertainty (ratio 0.96). It makes an instruction win unlikely without ruling it out, and it says nothing about cycles: locality, misses or dependency chains, in either direction | measure cycles and instructions by phase, on hardware: M1–M3 |
| 3 | the same 4-byte-a-row table, so no peak-RSS saving | **Yes, for memory.** For speed, both builds leave the same 4n-byte table, but not necessarily the same store work: B overstores at its cells' ends (§4, M2) | measure a store-only reference in decode order (arm D): a reference for the stores, not a floor every build obeys |
| 4 | the run sort is serial work on the pipelined team's leader | **No, a priori.** Against the sort's O(r) stands what the records save the leader: the column's stores, and the count checkpoints, which a run-fed share doesn't need (it finds its iterators by binary search) | measure the leader's serial MTF stage, today's (`mtf_checkpoints`) against the emitter plus the sort: Q2. The emitter alone is the bound with the sort moved to the helpers |
| 5 | a fallback to L is always needed for high-r blocks | **No, it caps the gain** to run-heavy blocks, and it must cost nothing elsewhere. It must be decided before MTF, from the symbols, since the block's length is known only after | measure the fallback on corpora where records don't pay: Q1's second condition |

**Conclusion.** Line A as #266 framed it, a cheaper build by instruction count, stays closed.
The reconciliation leaves no structural saving to predict. What it leaves are three
questions about cycles, which the diagnostic measures. Since §7, Q1 alone is judged, and the
mechanism readings are exploratory:
- **M (the first new mechanism): the build's cost is set by its table's 4n-byte write, not by
  its instructions.**
  - **Evidence against it, not a falsification:** the stores alone, in decode order, cost
    well under the build (M2), and a build after a pre-touch of its table costs about as
    much as one without (M1). Neither probe isolates write misses, and the store-only
    reference is not a floor every build obeys (§4). So M1 and M2 together can weaken M, but
    cannot rule out every write-traffic explanation.
  - If the build's cost is its own work, a run-fed fill could be bound by something else. One
    candidate is the sweep's iterator chain: each cell's end is a load and compare on the
    previous cell's indices, so cells could cost about a load's latency each, whatever their
    instructions (M3). A figure near that is consistent with the chain, not proof of it.
- **Q2:** the run records shorten the pipelined leader's serial stage, because they need no
  count checkpoints.
- **Q1:** the whole one-worker stage is cheaper. This is #266's sketch, made fair and runnable.

## 2. The design

### 2.1 The opt-in stage (`run-fed` feature)

Nothing in the decoder calls any of it, with or without the feature (`src/lib.rs`: `pub mod
run_stage` under `cfg(any(test, feature = "run-fed"))`). A default build compiles none of it
but its tests.

- **`mtf::mtf_runs`** (`src/mtf.rs`, beside `mtf_avx2`):
  - **The record:** one `u32` a run, `p << 8 | c`: its first row and byte. The run's length is
    the next record's p less its own; its k is computed by the sort; a closing record `n << 8`.
    That is 4 bytes a run, against the column's 1 byte a row: 0.25 B a row at Wikidata's
    r/n of 0.062.
  - **The AVX2 kernel is `mtf_avx2`'s loop, unchanged but for its stores.** The same
    two-level list in registers, the same `second`, `STEPS` and checks, the same counts.
  - **What changes:** a literal stores a 4-byte record at index m instead of a byte at row n
    (one more register and increment); a run stores nothing (`n += run`), where `mtf_avx2`
    makes two 32-byte stores or a memset.
  - So the comparison is SIMD against SIMD: the emitter differs from the production kernel
    only in what it writes.
  - **A scalar twin** is `mtf_scalar`'s loop, likewise.
  - **Proved:** a literal's index is at least 1, so its byte isn't the list's front, the open
    run's byte. So literals start runs, and counts extend them. A block that begins with a
    count opens its first run at row 0 with the list's initial front.
  - **Capacity:** at most one record a symbol, plus the closing one (`syms.len() + 1`).
- **`run_stage::sort`:** a stable counting sort of the records by byte into F order, `[a, p]`
  a run (a = C(c) + k, k accumulated in the sort itself), closed by `[n, 0]`. It returns false
  unless both orders tile [0, n):
  - the L starts rise strictly from 0 to the closing n;
  - each byte's run lengths sum to its count;
  - the counts sum to n.
  This is the soundness check that replaces `build_one_pass`'s end check. `fill` relies on
  exactly it: q always lies in the current F interval and L run, and every index stays in
  bounds. **Since §7 the check is a type:** `sort` returns `Option<Sorted<'a>>`. A `Sorted`
  holds the records, the F order (in a `Scratch` whose field is private) and n, all borrowed
  immutably, and has private fields, so only a `sort` that passed can make one, and neither
  input can change while it lives.
- **`run_stage::fill(&Sorted, ..)`:** one sequential sweep:
  - a cell ends where either order's run ends, so at most 2r − 1 cells (#266, proved);
  - on a cell, `T[q + i] = T[q] + 256·i`, written as 8-lane AVX2 stores (base + 0, 256, …,
    1792; then + 2048), unconditionally;
  - a cell's last store runs into the next cell's rows, which that cell writes again, or past
    n into 8 slots of slack, cut off by the length;
  - with no AVX2, the scalar `fill_rows(&Sorted, q0, q1, ..)` (safe code; it panics past
    n), which also fills any share of rows, finding its iterators by binary search. So a
    share needs no count checkpoints;
  - the AVX2 sweep is a private `unsafe fn` whose contract (AVX2, and a `Sorted`) is
    documented; `fill` is its one caller.
- **`run_stage::takes_runs`:** the fallback rule, from the symbols before MTF runs: records
  when 8·|syms| ≤ the block's bound and the bound is below 2²⁴; else the column's path.
  - It is #266's sketch rule (8·|syms| ≤ n), with the bound in place of n, which is unknown
    until MTF has run.
  - For a full block the two are about equal. For a short block (a stream's last, or a small
    file) the bound overstates n, and the rule may send a costly block to the records. The
    synthetic corpora measure that.
- **`run_stage::build`:** the whole stage with its fallback. It gives `bwt::build`'s table,
  word for word, and `mtf::mtf`'s result, error kind included.
- **`run_stage::floor`:** the probe for M. It stores the table's n entries in order, 8 at a
  time, and nothing else, leaving the length 0 (they are no table). Since §7 the buffer's
  address goes through `black_box` after the stores, so an optimiser must keep them. No
  assembly has been inspected to show they survive.

### 2.2 The diagnostic: `examples/run_fed_stage.rs`

`run_fed_stage FILE.bz2 [ROUNDS] [GHZ]`, the `sweep`/`phases` calling convention, so CI's
existing diagnostic dispatch (`sweep.yml`, new `tool: run_fed_stage`) runs it twice:
- **alone,** one copy pinned to one CPU;
- **contended,** one copy on every CPU at once.

**The arms.** Each is a decode of every block, in decode order:

| arm | front | build | why |
|---|---|---|---|
| A, the column | `mtf::mtf` → L | `bwt::build` | the one-worker stage (`Plan::Workers`, CI's 4-core runner) |
| B, the records | `mtf::mtf_runs`, then `sort` | `fill` | the candidate; the fallback (A's stage) on blocks `takes_runs` refuses |
| C, the column with checkpoints | `mtf::mtf_checkpoints` → L | `bwt::build` | today's pipelined leader's MTF |
| D, the store reference | `mtf::mtf` → L | `floor`, then `bwt::build` over the buffer it pre-touched | M1, M2: no candidate |

B's entry point is `run_stage::build` itself: the selector `takes_runs`, then the records' path
or the fallback, with its checks.

**The fairness conditions:**
- **The same input:** the blocks' symbols, scanned once, held in memory (as `sweep` does).
  The scanner is identical across arms and untimed.
- **The same CPU features:** `Kernels::detect()`, AVX2 in every arm's MTF and fill.
- **The incumbent's buffers (since §7):** each arm keeps its own buffers from block to
  block, as a worker does, and **the walk writes over the arm's `ll`**, as production's
  `Segmented::invert_in_place` does. There is no separate walk buffer. B's real extra storage,
  its records and F order, is what it holds beyond A's.
- **The same corpus,** the bench's Wikidata file, plus synthetic corpora.
- **Decode-order cache conditions (approximate):** after each block's stage, the walk, RLE1
  and the CRC run untimed on that arm's own table. So every stage meets the caches roughly as
  a decode leaves them, never warm from a repeat of the same block. This is a controlled
  approximation, not identical cache contents: a round's first block follows another arm or
  sweep, the rounds reuse the same buffers, and a small corpus's working set can stay cached.
  Scanning is removed, and its cache effects with it. No cache flush is made.
- **The same order:** the arms' order rotates by round.

**Two sweeps a round (since §7).** Each runs over every arm in the rotated order:
1. **The entry sweep (judged figures).** One clock bracket a block around the arm's whole
   entry point. For B that is `run_stage::build`: the selector, the dispatch, the sort's check
   and the fallback are all charged. Every arm has exactly one bracket a block, so the
   clock's cost is the same in each. The bracket's times are summed by the block's path (the
   blocks `takes_runs` accepts; the fallback's), giving `R_stage`, `R_acc` and `R_fb`.
2. **The phase sweep (attribution only).** Each phase in its own bracket, summed over the
   round's blocks:
   - `front`: MTF or emit, on the blocks B takes by records;
   - `sort`;
   - `build`: build or fill, on the same blocks;
   - `fallback`: the whole stage on the other blocks, in every arm.

   B takes three brackets on a records' block and A two, so the phases' sum is not the
   entry point's cost. `R_phases` beside `R_stage` measures that difference.

**Counters.** Where the runner lets a process count its own events, one more untimed phase
sweep an arm counts each phase's instructions, cycles, LLC misses and back-end stalls (the
`pmu` module, moved from `examples/sweep.rs` to `examples/support/pmu.rs`, unchanged).

**The ratios.** Arm over arm, paired by round, as the geometric mean and its 95% t-interval
on the logs. Below 1, the first arm is cheaper. A ratio with any round's figure absent (0) or
not finite is absent, never a pass.
- From the entry sweep:
  - `R_stage`: B / A, every block (Q1);
  - `R_acc`: B / A on the records' blocks;
  - `R_fb`: B / A on the fallback's blocks, the selector charged (Q1's fallback screen).
- From the phase sweep, attribution and exploration only:
  - `R_phases`: B / A, the phases' sum;
  - `R_front`: B's emit / A's MTF;
  - `R_build`: B's sort + fill / A's build;
  - `R_lead`: B's emit + sort / C's MTF (Q2);
  - `R_emit`: B's emit / C's MTF (Q2 with the sort moved to the helpers);
  - `R_floor`: D's store reference / A's build (M2);
  - `R_warm`: D's build after its pre-touch / A's build (M1);
- **M3:** the fill's *nominal* cycles a cell (median wall time × the sampled clock, not counted
  cycles), the fill's counted cycles a cell where a cycle counter ran with full coverage, cells
  a row, and the fill's stored slots a row (its overstores included).

**Validity.** Checked on every block before any timing, each failure a panic (and then no
record is written):
- B's table is A's word for word, from the records and through `run_stage::build`;
- B's counts and length are A's, and its path is `takes_runs`'s;
- the records are exactly L's maximal runs, closed by n;
- its cells are fewer than 2r;
- the walk of B's table, then RLE1, gives the block's stored CRC;
- the first file's whole output equals `mibz2::decode`'s.

Every timed pass checks every block's CRC again, and B's path.

**The record (since §7).** With `RUN_FED_RECORD` set, the binary writes a JSON record once every
check has passed. It holds:
- the head as `RUN_FED_HEAD` gives it (the workflow passes the full `git rev-parse HEAD`);
- the rounds, kernels, ghz, the CPU model and the process's CPU affinity;
- for each corpus: its name, size, SHA-256 (the Wikidata one cross-checked against
  `sha256sum`), blocks, rows, path counts, records, cells and fill slots;
- each round's arm order, and its **raw** entry times (arm × path) and phase times (arm ×
  phase).

`bench/run_fed_stage.py verify` judges the record on its own (§4).

**What the harness can't see:**
- **It is one thread.** Q2 measures the leader's serial MTF stage in isolation, not a
  pipelined team: how the helpers' share fill compares with `coop::build_fused`, and what α
  becomes, are not measured.
- **The contended run** (a copy on every CPU) is a stress condition, not a team. Each copy's
  exit status is collected (`bench/copies.sh`); a failed copy marks it INCOMPLETE and fails the
  job.
- **CI's runners** differ from the i9: AVX2, a smaller L3, and an unknown CPU model a run.

## 3. Correctness (checked)

| check | where | result |
|---|---|---|
| the table from records, each kernel's fill, in 1–5 shares, against `bwt::build`: every ternary column of 0–8 rows, and random run-heavy and uniform columns of 7–70,000 rows over σ 1–256 | `run_stage::tests` | pass |
| `sort` refuses a count off by one, a moved run start, a moved end and n off by one either way, and leaves its scratch empty | `run_stage::tests` | pass |
| the unchecked states (§7): no records for n 0 and 1, the closing record alone, another block's n, records out of order, two runs at one row, a first run past row 0, an uncounted byte, a scratch re-sorted for another block: each refused, so no `Sorted` exists for it; `fill_rows` past n panics; the empty block fills an empty table by both kernels | `run_stage::tests::miri_unchecked_states_refused`, natively and under Miri (scalar and AVX2) | pass |
| no `Sorted` made by hand, no `Scratch` forged, no records changed or scratch re-sorted under a live `Sorted` | doc tests, `compile_fail` with the error codes E0451, E0616, E0502, E0499 checked on nightly | pass |
| the record writer against the verifier's fixture, byte for byte; cells and fill slots; an absent round is no interval | `cargo test --features run-fed --example run_fed_stage` | pass |
| the verifier: the fixture VALID; each defect INVALID (below, §7) | `python3 bench/test_run_fed_stage.py` (37 tests) | pass |
| `mtf_runs` (scalar and AVX2) against libbzip2's loop on every stream of `tests/mtf_kernels.rs`: result and error kind (`RunTooLong`, `BlockTooLong` at every limit around each length), counts, records = L's maximal runs; the table by records and by `run_stage::build`'s chosen path = `bwt::build`'s | `tests/mtf_kernels.rs` with `--features run-fed` (new CI step) | pass |
| all 431 blocks of the bench corpus (`wikidata.bz2`, SHA-256 `82f65ace…`), and the six synthetic corpora: table word for word, counts, records = runs of L, cells < 2r, CRC; the Wikidata file's output = `decode`'s | the diagnostic's validation pass | pass |
| `sort`'s and `floor`'s unsafe writes, and the AVX2 fill (`-C target-feature=+avx2,+bmi1`; since §7 the tests assert they reached AVX2, `MIBZ2_EXPECT_AVX2`) | `cargo miri test --lib -- run_stage::tests::miri_`, scalar and AVX2: `miri.yml`'s two steps | pass, both |
| mutation: a record's p off by one at row 40 in the AVX2 emitter | `tests/mtf_kernels.rs` | caught (3 of 4 tests) |
| mutation: the AVX2 fill's step 2304 instead of 2048 | `run_stage::tests` | caught |
| the decoder's own suites, unchanged code: `cooperative`, `pipelined` (the checkpointed multi-core decodes), `in_place`, `mtf_checkpoints`, `mtf_kernels`, `decode`, the lib's tests | `cargo test` | pass |
| clippy (`--all-targets`, all features and default), fmt, taplo, whitespace, machete | `just`'s checks | clean |

**Not covered:**
- The AVX2 emitter under Miri: its intrinsics are `mtf_avx2`'s, and only the differential
  tests check it.
- No fuzzing of the reached paths.
- No disassembly: `black_box` is there to keep `floor`'s stores in the fat-LTO build, but
  that they survive has not been observed.
- `Kernels`' fields are public, so a safe caller can claim AVX2 on a CPU without it. That is
  the crate's existing contract for every kernel (`mtf::mtf` included), not this research's.
  It is noted for the Guardian, not changed here.
- **An independent adversarial review of the new unsafe code (RULES B5).** The earlier draft
  said a diagnostic run doesn't need one, because its checks compare every table first. That
  claim is withdrawn (§7): a comparison made after construction can't repair undefined
  behaviour during it, and B5 grants no such exemption. Since §7 the unchecked fill can't be
  reached from safe code on unchecked state. The review of `sort`, the fill and the emitter
  is still owed, and the Guardian decides whether it must precede the diagnostic run.

## 4. The registration (RULES C1, diagnostic form). DRAFT, NOT FROZEN

Revised for §7. The coordinator freezes it, after re-review, before any new observation.

- **The information objective** (C1's diagnostic option: it can never qualify a production
  change): line A's cost on one CI runner, measured rather than modelled.
  - Q1, at one worker: judged, as a screen for registering a prototype.
  - Q2, the pipelined leader's serial stage: exploratory.
  - M1–M3, what bounds the build: exploratory.
- **The baseline:** master `a4f97714`. Arms A, C and D's build are master's code, unchanged.
- **The head:** this PR's head at freezing. The coordinator names it as a full 40-character
  SHA (a registration can't name its own commit). It is dispatched as `sweep.yml`'s
  `registered_head` input.
- **The run:** `sweep.yml`, dispatched on this branch at the frozen commit, `tool:
  run_fed_stage`, `rounds: 15`, `registered_head` the frozen SHA. One dispatch.
- **Excluded data.** The two local discovery rounds (§4's disclosure, below) are excluded from
  every estimate and reading. Only the registered CI run's record is judged.
- **The primary condition** is the one copy pinned to one CPU. The contended run is
  reported, never judged.
- **The estimand:** per-round paired ratios of the entry sweep's times, the geometric mean
  and its 95% t-interval (df 14).

**Admission: VALID or INVALID** (`bench/run_fed_stage.py verify run-fed-1.json --head
<frozen SHA> --wikidata corpora/wikidata.bz2`, run by the workflow; anyone can re-run it on
the artifact). VALID needs all of the following. Anything else is INVALID, and an INVALID
record has no reading:
- the record exists (the binary writes it only after every correctness check has passed)
  and loads as JSON with no NaN or infinity;
- its head is a full 40-character SHA, equal to the registered head;
- rounds exactly 15; AVX2 selected; correctness `passed`; the host recorded; the affinity a
  single CPU;
- exactly seven corpora: first the Wikidata file, its SHA-256 the registered
  `82f65ace…c29915` and equal to the file's own hash; then the six registered synthetic
  corpora, by name, in order, each with a SHA-256, positive size and blocks. The synthetic
  bytes are fixed by the head, since the text corpus is read from the checkout's sources. The
  workflow refuses to run if those sources differ from the head;
- the path counts consistent (records' blocks plus fallback's equal the blocks);
- every round's arm order the registered rotation;
- **every registered interval present.** For every corpus and every arm:
  - each entry path with blocks has a positive finite time in every round;
  - each path without blocks has exactly zero;
  - each required phase is positive wherever its path has blocks;
  - A's and C's sort is zero.

  An absent corpus, round, arm or interval is INVALID, never a worst of 0.

**Q1, the gate (judged).** On the Wikidata file, with two screens:

| reading | condition |
|---|---|
| **FEASIBLE:** a one-worker production prototype may be registered, on its own | `R_stage` ≤ 0.90 and its hi < 1, **and** both screens pass |
| **STOP: no saving of 5% or more shown** | `R_stage`'s lo > 0.95 |
| **STOP: a screen failed** | `R_stage` ≤ 0.90 and its hi < 1, but a screen fails |
| **STOP, INCONCLUSIVE** | anything else |

The screens:
- **the synthetic whole-stage screen:** every synthetic corpus's `R_stage` geometric
  mean ≤ 1.02;
- **the fallback screen:** every corpus with fallback blocks has `R_fb` geometric mean ≤ 1.02.
  `R_fb` is measured on the fallback's blocks at the entry point, with the selector
  charged. A corpus with no fallback blocks has no `R_fb` (N/A, legitimately). If no corpus
  has fallback blocks, the screen is vacuous, and the verifier says so.

**What the screens are not.** They are **point-estimate screens, reported, and explicitly not
a noninferiority claim.** A mean of 1.01 with interval [0.80, 1.30] passes, and the verifier's
fixture tests show exactly that. A pass does not show that no corpus or fallback regresses by
more than 2%. So this registration makes no noninferiority claim, and needs no C6 interval or
multiplicity decision from Louis. Any production prototype that follows must settle
fallback noninferiority under its own registration, and that is where C6 applies.

**What the readings are not.** Every STOP, and INCONCLUSIVE, is an **operational pause**: no
further work on this implementation without a new registration. None is proof of no effect.
- A STOP with lo > 0.95 excludes a saving of 5% or more only for the tested implementation,
  workload and runner, under the interval's assumptions.
- INCONCLUSIVE is recorded, never re-run without a new registration.
- FEASIBLE shows some saving, with a 10% point estimate. It is not a confident 10% lower
  bound.

**Q2, the leader: exploratory.** `R_lead` (B's emit + sort / C's MTF, from the phase sweep)
is reported with its interval and labelled: LEADER LEAD (≤ 0.90, hi < 1), NO LEADER LEAD (lo
> 0.95), or NO LEADER LEAD, INCONCLUSIVE. The label is a discovery-derived lead, not a judged
result. A lead can justify preparing a separately registered experiment, nothing more.
`R_emit` is reported beside it.

**M1–M3, the mechanism: exploratory.** Each is reported as an estimate and its interval, with
the labels set before any new data, and none is a proof of mechanism:
- **M1:** `R_warm` ≥ 0.95 is labelled "no pre-touch effect"; below, "a pre-touch effect".
  - It is a pre-touch contrast. It does not isolate cold write misses: A's table may already
    be resident after the previous walk, and D's own writes disturb the caches and write
    buffers.
  - A difference does not uniquely attribute its cause to write misses.
- **M2:** `R_floor` ≥ 0.80 is labelled "near the store-only reference"; below, "mostly above
  it".
  - The reference is store-only, not a universal floor. Production overlaps its stores with
    other work, in another order.
  - B overstores at cell ends: `8·Σ⌈lᵢ/8⌉` slots, between n and n + 7c (three one-row cells
    store 24 slots for a 3-word table). So equal final 4n-byte tables need not mean equal
    store work. The fill's slots a row are reported.
- **M3:** the fill's **nominal** cycles a cell (wall time × the sampled clock, not counted
  cycles), and its **counted** cycles a cell where a cycle counter ran with full coverage.
  - A figure near a load's latency is consistent with the iterators' chain bounding the fill;
    it does not prove it. That would need a dependency-changing contrast or a demonstrated
    critical path.
- The earlier draft's claim that "mechanism M is falsified by M1 ≥ 0.95 with M2 < 0.80" is
  withdrawn: M1 and M2 cannot falsify every write-traffic explanation.
- #266's 2.6 against 2.5 instructions a row is a source-shape model with unspecified
  uncertainty (ratio 0.96). It is motivation for measuring, not an admission gate, and it
  rules out neither an instruction win nor a cycle win.

**Voids and failures.**
- **A void run** may be re-dispatched once at the same commit. A run is void only when the
  infrastructure fails before the measurement means anything:
  - the job fails during setup, build or corpus fetch, before the binary starts;
  - the runner lacks AVX2 (the record says `avx2: false`);
  - the runner fails mid-run for a reason outside the project (a runner lost or cancelled).
- **A correctness assertion failure is not a void.** That covers a validity panic, a CRC
  mismatch, a path disagreement, and no record written because a check failed. It starts an
  investigation under B6, with no retry: repair it, then a new head and a new registration
  identify the new evidence.
- **An INVALID record** for any other reason (a missing interval, the wrong head or corpus)
  is investigated likewise. It is never re-run blind.
- No other re-run, no workstation, no ready tag, no queue entry. Neither local nor CI
  outcomes choose extra repeats.

**The positive and negative stop criteria.**
- **Positive:** Q1 FEASIBLE, or a useful Q2 lead. Either permits drafting a production
  prototype's own registration, with a pre-set speed threshold, CI's paired bench, B5's
  review, and its own fallback noninferiority rule (C6).
  - It may also justify preparing a separately frozen i9 diagnostic, with the incumbent's
    buffer lifetime and an explicit question, for Louis to run by C5. Q1 INCONCLUSIVE may
    justify the same.
  - No workstation run is permitted by this registration.
- **Negative:** Q1 STOP and no Q2 lead. Line A is paused as a performance lead on this
  runner. Record it as "negative on this runner", not "line A disproved on the i9": a journal
  entry, the queue issue's Rejected list, and the feature kept as research or removed, at the
  coordinator's choice.
  - If Louis wants the i9 question answered even then, an i9 measurement is still needed.
  - It reopens only if a later profile shows a build bound as M describes, on a corpus or
    host that matters.

**What was seen before this draft, disclosed.** A local 2-round run on the shared machine was
made, for validity only. Its figures are indicative, not evidence: R_stage about 1.0–1.1, the
fill about the build, the sort about a quarter of the build, the emitter about the plain MTF
and below `mtf_checkpoints`, the floor about half the build, the warm build about the build.
**Those two rounds are discovery data, and are excluded from every estimate here.** No
further local timing was run for the revision (§7).
- **Set before it:** Q1's 0.90 threshold and its interval condition, both #266's.
- **Set or changed after it:**
  - Q1's negative threshold (lo > 0.95);
  - the synthetic screen's form (the geometric mean ≤ 1.02, since the synthetic corpora
    have 1–6 blocks and wide intervals). That is why it is only a screen;
  - Q2's and M1–M3's labels. That is why they are exploratory.
- **Unchanged by the revision:** 0.90 and 0.95 are kept as they were, neither tightened nor
  relaxed because the discovery rounds looked pessimistic. Changing them now would be one
  more data-dependent choice.
- **Changed by the revision, from the review's analysis and not from any new data:** the
  fallback screen now reads `R_fb` at the entry point, not R_stage; the screens' status;
  admission; and the void rule.

**Cost:** one `sweep.yml` job. That is about 5 minutes of build, and runs that are about
twice the earlier model (two sweeps a round, about 6 s a round alone).

**What CI can answer.** CI answers the one-worker question on its **recorded runner**:
`ubuntu-latest` names no CPU model, and the record does. It does not close the question for
the i9 or for a pipelined team:
- the contended run is a stress condition, not a team;
- Q2 times the leader's serial front, not the team's critical path.

## 5. Limitations

- **The emitter is fair only at the loop level.** It shares `mtf_avx2`'s list kernel, but an
  integration might schedule it differently: the pipelined team's leader runs
  `mtf_checkpoints_avx2`, a differently structured list kernel. That is arm C, measured as
  it is.
- **The fill is one design:** two iterators, one cell at a time. A design that breaks the
  iterators' chain, for instance by filling the F intervals independently and then the L
  bytes, would write the table twice. It isn't built.
- **No pipelined team, no share fill timed** (§2.2). Q2 is the leader's stage alone.
- **The fallback rule can't see n,** so short blocks may go to the records at a loss. The
  synthetic corpora measure the cost. A better rule (a cheap literal count) isn't built.
- **CI's runners vary by model,** and the summary names the one used. One run is judged as
  it lands.

## 6. Recommendation

- **Line A as a speed change:** not predicted by the reconciliation, which finds no removed
  pass. The drafter's prior for Q1 is STOP. The model is motivation for measuring, not a
  closed result (§7).
- **The diagnostic** is cheap: one CI job, no workstation, no Louis time. It turns #266's
  model into measured costs on one runner, and explores (not tests) a mechanism, M, that
  bears on every build. Freeze it, after the re-review and the Guardian's call on B5's
  timing (§3), if that information is wanted.
- **Ready to run now:** #258 (`bench-service-2x3`), frozen and awaiting admission.
  **Under development:** this diagnostic, not a workstation candidate.

## 7. Revision, 2026-11-09: the Researcher's REVISE (#145 comment 6729473179)

The review of head `187c0ba5` (full report: `team-state/researcher` at `f84cb1fd`,
`state/reports/run-fed-stage-6729371816/`). Each finding, its fix, and the test that holds
it. No timing was run for the revision; the tests are correctness tests only.

| # | finding | fix | test |
|---|---|---|---|
| 1a | safe `fill` reaches the AVX2 fill's raw reads on unverified state (`fill(&[], &Scratch::default(), 1, ..)` reads empty F-order storage) | `sort` returns `Option<Sorted<'a>>`. `Sorted` has private fields and borrows the records and scratch immutably; `Scratch.fo` is private; `fill(&Sorted, tt, k)` and `fill_rows(&Sorted, q0, q1, out)` take one. The AVX2 sweep is a private `unsafe fn` with a `# Safety` contract, and `fill` is its one caller | `miri_unchecked_states_refused`: the review's witness state, empty records for n 0 and 1, refused. Doc tests: `compile_fail` for a hand-made `Sorted`, a forged `Scratch`, records changed under a `Sorted` (E0502), the scratch re-sorted under one (E0499) |
| 1b | the scalar route could make a `TVector` with out-of-range links from a forged scratch | the same: `fill_rows` reads only a `Sorted`'s state, and panics for rows past n | the same test: `fill_rows` past n panics. Every `sort` refusal leaves the scratch empty |
| 1c | negative-state coverage | another block's n (0, 1, 3, 5, 2²⁴), records out of order, two runs at a row, a first run past row 0, an uncounted byte, n ± 1 on every test column, a scratch re-sorted for another block | `miri_unchecked_states_refused`; `check` on every ternary and random column |
| 1d | Miri on these, and AVX2 coverage not certified | `miri.yml` runs `run_stage::tests::miri_` again with `-C target-feature=+avx2,+bmi1`, and the tests assert they reached AVX2 (`MIBZ2_EXPECT_AVX2`, forwarded by `-Zmiri-env-forward`) | run here: both tests pass under Miri, scalar and AVX2. Without the flags and with the variable set, the assertion fails, as it must |
| 1e | the draft's blanket "a diagnostic needs no B5 review" | withdrawn (§3, Not covered); the B5 review is owed and the Guardian decides its timing | — |
| 2a | arm A walks into a separate `walked` buffer; production walks over `ll` | every arm walks over its own `ll` (`Segmented::invert_in_place`'s reuse). `walked` is gone | the diagnostic's CRC checks after every walk |
| 2b | R_stage summed clocked phases; `takes_runs`, the selector, the asserts and the dispatch were outside; the brackets were asymmetric | an entry sweep: one bracket a block around each arm's whole entry point (`run_stage::build` for B). The phase sweep is kept for attribution, and `R_phases` against `R_stage` measures the bracket effect | the record's raw entry times; B's path checked against `takes_runs` on every timed block |
| 2c | the fallback decision used whole synthetic R_stage | the fallback screen reads `R_fb`, from the entry sweep on the fallback's blocks, with the selector charged. The synthetic whole-stage screen is kept and named as such | `test_the_fallback_screen_reads_r_fb_not_r_stage` (the review's counterexample: the fallback screen passes, the whole-stage screen fails, each named); `test_a_costly_fallback_fails_its_screen` |
| 3a | `filter_map` dropped absent intervals; all absent gave worst = 0 | the binary prints INVALID for any absent required interval. The verifier requires all seven corpora and every path's times | `test_a_missing_synthetic_corpus`, `test_every_synthetic_corpus_absent`, `test_an_absent_interval`, `test_a_renamed_corpus`, `absent_rounds_are_no_interval` (Rust) |
| 3b | positive finite totals; exact rounds, kernels, full head, corpus identity | the verifier's admission list (§4); a JSON record with the full head, the SHA-256s (Rust, cross-checked against `sha256sum` and the verifier's own hash), the host and the affinity; the workflow refuses sources that differ from the head | `test_a_zero_total`, `test_a_negative_time`, `test_a_nonfinite_time_does_not_load`, `test_the_wrong_rounds`, `test_rounds_as_a_bool`, `test_no_avx2`, `test_a_short_head`, `test_a_short_registered_head`, `test_another_head`, `test_no_head`, `test_another_wikidata_file`, `test_the_wikidata_files_own_digest`, `test_a_synthetic_digest_missing`, `test_an_unpinned_primary`, `test_no_host`, `test_the_arms_order_not_rotated`, `test_blocks_that_dont_add_up`, `test_a_missing_round`, `test_missing_phase_times`, `test_a_time_for_an_absent_path`; SHA-256 against FIPS 180-4's vectors |
| 3c | raw paired phase timings not archived | the record holds every round's arm order, entry and phase times; the workflow uploads `run-fed-*.json` | `the_verifiers_fixture`: the fixture is `record`'s own output, byte for byte, so the verifier's tests read the writer's format |
| 3d | a bare `wait` masked a contended child's failure | `bench/copies.sh` waits on each PID and names each failure. A failure writes `…-INCOMPLETE.txt`, marks the summary INCOMPLETE and fails the job. `set -o pipefail` covers the primary run | `test_one_failed_copy_fails_the_run`, `test_all_copies_succeed`, `test_a_bare_wait_would_have_masked_it` |
| 3e | a correctness assertion was an automatic void-retry | removed: investigation under B6, no retry (§4, Voids and failures) | — |
| 4 | thresholds after the discovery rounds | 0.90 and 0.95 unchanged; the two local rounds excluded and said so; the synthetic rule demoted to a reported point-estimate screen, explicitly not noninferiority, so no C6 decision is needed; Q2 and M1–M3 exploratory; STOP and INCONCLUSIVE worded as operational pauses | `test_a_wide_interval_passes_the_screen_as_a_screen_only` (mean 1.01, interval about [0.8, 1.3], passes as a screen only); the verifier prints the notes with every reading |
| 5a | the pre-touch probe does not isolate cold write misses | M1 relabelled a pre-touch contrast (§4); D's phase is `pre-t bld` | — |
| 5b | the store-only probe is no universal floor; overstores change store work | M2 relabelled against a store-only reference. The fill's stored slots are counted (`cells` returns them) and reported a row. `floor` passes its buffer through `black_box`, so its stores must stay | `cells_and_slots`: three one-row cells, 24 slots for a 3-word table |
| 5c | M3's "cycles a cell" was wall time × GHz | labelled nominal everywhere; counted cycles a cell printed beside it where the cycle counter ran with full coverage | — |
| 5d | further qualifications: decode order is approximate; the instruction model is no gate; CI's runner is not the i9 | §2.2's fairness list, §4's M section and "what CI can answer" | — |

**Not closed here, and why:**
- **The B5 adversarial review** of `sort`, the fill and the emitter is someone else's to
  make: the Guardian's, or an independent reviewer's. This revision makes the unchecked path
  unreachable from safe code; it does not certify it.
- **The emitter under Miri, fuzzing, and disassembly** of `floor`'s stores in the fat-LTO
  build: none was run. Each is listed in §3.
- **`Kernels`' public fields** let safe code claim AVX2 on a CPU without it. That is a
  crate-wide contract, `mtf::mtf`'s included, and is left for the Guardian.
- **The freeze:** the coordinator's, after re-review.

## Provenance and reproduction

- **Base:** master `a4f97714` (#266's merge). The corpus is as #266's: `wikidata.bz2`,
  SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.
- **Tests:**
  - `cargo test --features run-fed --test mtf_kernels`;
  - `cargo test --features run-fed --lib run_stage`;
  - `cargo test --features run-fed --doc run_stage` (on nightly, the error codes are checked);
  - `cargo test --features run-fed --example run_fed_stage`;
  - `python3 bench/test_run_fed_stage.py`;
  - `cargo +nightly miri test --lib -- run_stage::tests::miri_`, then again with
    `RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1
    MIRIFLAGS=-Zmiri-env-forward=MIBZ2_EXPECT_AVX2` for the AVX2 fill.
- **The diagnostic:**
  `cargo build --release --features run-fed --example run_fed_stage && target/release/examples/run_fed_stage wikidata.bz2 15`.
  With `RUN_FED_RECORD=run-fed-1.json RUN_FED_HEAD=$(git rev-parse HEAD)`, the record; then
  `python3 bench/run_fed_stage.py verify run-fed-1.json --head <SHA> --wikidata wikidata.bz2`.
  On CI: Actions → Walk sweep → `tool: run_fed_stage`, `rounds: 15`, `registered_head` the
  frozen SHA.
