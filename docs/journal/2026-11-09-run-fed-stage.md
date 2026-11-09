# 2026-11-09: The run-fed stage (line A), reconciled, built opt-in, and a diagnostic registered (NOT FROZEN)

**Status: a draft registration, NOT FROZEN, NOT RUN.** The coordinator freezes it, or doesn't.
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
  blocks; the CRC and `decode`'s output; the unsafe code under Miri, the AVX2 fill included.
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
| 2 | about 2.6 instructions a row today against about 2.5 run-fed (model) | **Yes, for a win in instructions.** A win could come only from cycles at equal instructions: locality, misses, or dependency chains. The model sees none of those, in either direction | measure cycles and instructions by phase, on hardware: M1–M3 |
| 3 | the same 4-byte-a-row table, so no peak-RSS saving | **Yes, for memory.** For speed, it means both builds store the same 4n bytes, so they share one floor for those stores | measure that floor in decode order (arm D): what any build of this table can save is above it |
| 4 | the run sort is serial work on the pipelined team's leader | **No, a priori.** Against the sort's O(r) stands what the records save the leader: the column's stores, and the count checkpoints, which a run-fed share doesn't need (it finds its iterators by binary search) | measure the leader's serial MTF stage, today's (`mtf_checkpoints`) against the emitter plus the sort: Q2. The emitter alone is the bound with the sort moved to the helpers |
| 5 | a fallback to L is always needed for high-r blocks | **No, it caps the gain** to run-heavy blocks, and it must cost nothing elsewhere. It must be decided before MTF, from the symbols, since the block's length is known only after | measure the fallback on corpora where records don't pay: Q1's second condition |

**Conclusion.** Line A as #266 framed it, a cheaper build by instruction count, stays closed.
The reconciliation leaves no structural saving to predict. What it leaves are three
falsifiable claims about cycles, and the diagnostic measures exactly those:
- **M (the first new mechanism): the build's cost is set by its table's 4n-byte write, not by
  its instructions.**
  - **Falsified if** the stores alone, in decode order, cost well under the build (M2), and a
    build whose table lines are already resident costs about as much as one whose aren't (M1).
  - Then the build's cost is its own work above the floor. A run-fed fill that doesn't come
    near the floor either is bound by something else. The candidate is the sweep's iterator
    chain: each cell's end is a load and compare on the previous cell's indices, so cells
    cost about a load's latency each, whatever their instructions (M3).
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
  bounds.
- **`run_stage::fill`:** one sequential sweep:
  - a cell ends where either order's run ends, so at most 2r − 1 cells (#266, proved);
  - on a cell, `T[q + i] = T[q] + 256·i`, written as 8-lane AVX2 stores (base + 0, 256, …,
    1792; then + 2048), unconditionally;
  - a cell's last store runs into the next cell's rows, which that cell writes again, or past
    n into 8 slots of slack, cut off by the length;
  - with no AVX2, the scalar `fill_rows`, which also fills any share of rows, finding its
    iterators by binary search. So a share needs no count checkpoints.
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
  time, and nothing else, leaving the length 0 (they are no table).

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
| D, the floor | `mtf::mtf` → L | `floor`, then `bwt::build` over the buffer it left resident | M1, M2: no candidate |

**The fairness conditions:**
- **The same input:** the blocks' symbols, scanned once, held in memory (as `sweep` does).
  The scanner is identical across arms and untimed.
- **The same CPU features:** `Kernels::detect()`, AVX2 in every arm's MTF and fill.
- **The same buffer reuse:** each arm keeps its own buffers from block to block, as a worker
  does.
- **The same corpus,** the bench's Wikidata file, plus synthetic corpora.
- **The same cache conditions:** after each block's stage, the walk (`walk_segments_room`),
  RLE1 and the CRC run untimed on that arm's own table, so every stage meets the caches as a
  decode leaves them, never warm from a repeat.
- **The same order:** the arms' order rotates by round.

**The phases.** Each is summed over a round's blocks:
- `front`: MTF or emit, on the blocks B takes by records;
- `sort`;
- `build`: build or fill, on the same blocks;
- `fallback`: the whole stage on the other blocks, in every arm.

So the new MTF output cost (`front`, B against A and C), the sort, the fill and the fallback
are each attributed.

**Counters.** Where the runner lets a process count its own events, one more untimed round an
arm counts each phase's instructions, cycles, LLC misses and back-end stalls (the `pmu` module,
moved from `examples/sweep.rs` to `examples/support/pmu.rs`, unchanged).

**The ratios.** Arm over arm, paired by round, as the geometric mean and its 95% t-interval
on the logs. Below 1, the first arm is cheaper.
- `R_stage`: B / A, the whole stage (Q1);
- `R_front`: B's emit / A's MTF;
- `R_build`: B's sort + fill / A's build;
- `R_lead`: B's emit + sort / C's MTF (Q2);
- `R_emit`: B's emit / C's MTF (Q2 with the sort moved to the helpers);
- `R_fb`: B / A on the fallback's blocks (should be 1);
- `R_floor`: D's floor / A's build (M2);
- `R_warm`: D's warm build / A's build (M1);
- **M3:** the fill's cycles a cell, and cells a row.

**Validity.** Checked on every block before any timing, each failure a panic:
- B's table is A's word for word, from the records and through `run_stage::build`;
- B's counts and length are A's;
- the records are exactly L's maximal runs, closed by n;
- its cells are fewer than 2r;
- the walk of B's table, then RLE1, gives the block's stored CRC;
- the first file's whole output equals `mibz2::decode`'s.

Every timed pass checks every block's CRC again.

**What the harness can't see:**
- **It is one thread.** Q2 measures the leader's serial MTF stage in isolation, not a
  pipelined team: how the helpers' share fill compares with `coop::build_fused`, and what α
  becomes, are not measured.
- **The contended run** (a copy on every CPU) is a stress condition, not a team.
- **CI's runners** differ from the i9: AVX2, a smaller L3, and an unknown CPU model a run.

## 3. Correctness (checked)

| check | where | result |
|---|---|---|
| the table from records, each kernel's fill, in 1–5 shares, against `bwt::build`: every ternary column of 0–8 rows, and random run-heavy and uniform columns of 7–70,000 rows over σ 1–256 | `run_stage::tests` | pass |
| `sort` refuses a count off by one, a moved run start and a moved end | `run_stage::tests` | pass |
| `mtf_runs` (scalar and AVX2) against libbzip2's loop on every stream of `tests/mtf_kernels.rs`: result and error kind (`RunTooLong`, `BlockTooLong` at every limit around each length), counts, records = L's maximal runs; the table by records and by `run_stage::build`'s chosen path = `bwt::build`'s | `tests/mtf_kernels.rs` with `--features run-fed` (new CI step) | pass |
| all 431 blocks of the bench corpus (`wikidata.bz2`, SHA-256 `82f65ace…`), and the six synthetic corpora: table word for word, counts, records = runs of L, cells < 2r, CRC; the Wikidata file's output = `decode`'s | the diagnostic's validation pass | pass |
| `sort`'s and `floor`'s unsafe writes, and the AVX2 fill (`-C target-feature=+avx2,+bmi1`, detection confirmed reporting AVX2 under Miri) | `cargo miri test --lib -- miri_sort_and_floor` (added to `miri.yml`'s paths) | pass, both |
| mutation: a record's p off by one at row 40 in the AVX2 emitter | `tests/mtf_kernels.rs` | caught (3 of 4 tests) |
| mutation: the AVX2 fill's step 2304 instead of 2048 | `run_stage::tests` | caught |
| the decoder's own suites, unchanged code: `cooperative`, `pipelined` (the checkpointed multi-core decodes), `in_place`, `mtf_checkpoints`, `mtf_kernels`, `decode`, the lib's tests | `cargo test` | pass |
| clippy (`--all-targets`, all features and default), fmt, taplo, whitespace, machete | `just`'s checks | clean |

**Not covered:**
- The AVX2 emitter under Miri: its intrinsics are `mtf_avx2`'s, and only the differential
  tests check it.
- An independent adversarial review of the new unsafe code (RULES B5). This is required
  before any production use. A diagnostic run doesn't need it: the run's validity checks
  compare every table word for word first.

## 4. The registration (RULES C1, diagnostic form). DRAFT, NOT FROZEN

- **The information objective** (C1's diagnostic option: it can never qualify a production
  change): settle line A's real cost.
  - Q1, at one worker;
  - Q2, on the pipelined leader's serial stage;
  - M1–M3, which mechanism bounds the build.
  Each is read from measured cycles, not the model.
- **The baseline:** master `a4f97714`. Arms A, C and D's build are master's code, unchanged.
- **The head:** this PR's head at freezing. The coordinator names it (a registration can't
  name its own commit), and the run is valid only if the summary's `head` line is that commit.
- **The run:** `sweep.yml`, dispatched on this branch at the frozen commit, `tool:
  run_fed_stage`, `rounds: 15`. One dispatch.
  - A void run may be re-dispatched once at the same commit. A run is void if a validity
    assertion fails, the job fails before its summary, the kernels line shows `avx2: false`,
    or the head differs.
  - No other re-run. No workstation, no ready tag, no queue entry.
- **The primary condition** is the one copy pinned to one CPU. The contended run is
  reported, never judged.
- **The estimand:** per-round paired ratios, the geometric mean and its 95% t-interval
  (df 14), as `sweep.rs` computes them.

**Q1, the gate (judged).** On the given file (Wikidata):

| reading | condition |
|---|---|
| **FEASIBLE:** a one-worker production prototype may be registered, on its own | `R_stage` ≤ 0.90 and its hi < 1, **and** every synthetic corpus's `R_stage` geometric mean ≤ 1.02 |
| **STOP:** no stage saving of 5% or more | `R_stage`'s lo > 0.95 |
| **STOP:** the fallback costs too much | the gate not met for that reason alone: a synthetic corpus's `R_stage` above 1.02 |
| **STOP, INCONCLUSIVE** | anything else; recorded, never re-run without a new registration |

**Q2, the leader (judged as information).**

| reading | condition |
|---|---|
| **LEADER LEAD:** the next question is a pipelined prototype (the sort and the fill on the helpers), registered on its own | `R_lead` ≤ 0.90 and its hi < 1 |
| **NO LEADER LEAD** | `R_lead`'s lo > 0.95 |
| **NO LEADER LEAD, INCONCLUSIVE** | anything else |

`R_emit` is reported: the bound if the sort moves off the leader.

**M1–M3, the mechanism (reported, with their readings set now):**
- **M1:** `R_warm` ≥ 0.95 reads "the build is not bound by its table's write misses";
  below, "the table's write misses cost the build".
- **M2:** `R_floor` ≥ 0.80 reads "the build is near its stores' floor"; below, "the build's
  cost is mostly above its stores' floor".
- **M3:** the fill's cycles a cell. If it is near a load's latency plus a few cycles, the
  iterators' chain bounds the fill. The counters' IPC by phase is reported beside it.
- **Mechanism M** (§1) is falsified by M1 ≥ 0.95 together with M2 < 0.80.

**The positive and negative stop criteria.**
- **Positive:** Q1 FEASIBLE or Q2 LEADER LEAD. That permits drafting a production
  prototype's own registration, with a pre-set speed threshold, CI's paired bench, and B5's
  review. It permits no workstation run. Any workstation run is Louis's, by the frozen-recipe
  mechanism (C5).
- **Negative:** Q1 STOP and Q2 NO LEADER LEAD. Line A is then closed as a performance lead on
  measured costs: a journal entry, the queue issue's Rejected list, and the feature kept as
  research or removed at the coordinator's choice. It reopens only if a later profile shows a
  build bound as M describes, on a corpus or host that matters.

**What was seen before this draft, disclosed.** A local 2-round run on the shared machine was
made, for validity only. Its figures are indicative, not evidence. (So far: R_stage about
1.0–1.1, the fill about the build, the sort about a quarter of the build, the emitter about
the plain MTF and below `mtf_checkpoints`, the floor about half the build, the warm build
about the build.)
- **Set before it:** Q1's 0.90 threshold and its interval condition, both #266's.
- **Set or changed after it:**
  - the fallback condition's form: #266's "no corpus more than 2% worse" is read here as the
    geometric mean ≤ 1.02, not the interval's hi, since the synthetic corpora have 1–6
    blocks and wide intervals;
  - Q2's and M1–M3's readings, added once arm D was;
  - Q1's negative threshold (lo > 0.95).
- None is a relaxation of a registered rule: none was registered before. The coordinator may
  make any of them stricter at freezing.

**Cost:** one `sweep.yml` job, about 5 minutes of build and under 5 minutes of runs (model,
from the local run: about 3 s a round alone).

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

- **Line A as a speed change:** closed by the reconciliation, as #266 closed it. The prior
  for Q1 is STOP.
- **The diagnostic** is cheap (one CI job, no workstation, no Louis time) and turns #266's
  model into measured costs. It also tests a new mechanism, M, that bears on every build, not
  only line A's. Freeze it if that information is wanted; otherwise close line A on the model.
- **Ready to run now:** #258 (`bench-service-2x3`), frozen and awaiting admission.
  **Under development:** this diagnostic, not a workstation candidate.

## Provenance and reproduction

- **Base:** master `a4f97714` (#266's merge). The corpus is as #266's: `wikidata.bz2`,
  SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`.
- **Tests:**
  - `cargo test --features run-fed --test mtf_kernels`;
  - `cargo test --features run-fed --lib run_stage`;
  - `cargo +nightly miri test --lib -- miri_sort_and_floor`, plus
    `RUSTFLAGS="-C target-feature=+avx2,+bmi1"` for the AVX2 fill.
- **The diagnostic:**
  `cargo build --release --features run-fed --example run_fed_stage && target/release/examples/run_fed_stage wikidata.bz2 15`.
  On CI: Actions → Walk sweep → `tool: run_fed_stage`, `rounds: 15`.
