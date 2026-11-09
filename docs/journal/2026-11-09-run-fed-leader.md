# 2026-11-09: The run-fed leader for the pipelined teams, built opt-in, and registered

**Status: CI's run VALID (JUDGED), an attributed LEAD** (`03e52206`, run 42564217298,
`2026-11-09-run-fed-leader-result.md`). **The workstation run (§8) is NOT FROZEN.**
Revised nine times: §R1, the reviews of `f94551ab`; §R2, D049; §R3, an empty block, from
CI's fuzz-smoke; §R4, the workstation run made freeze-ready; §R5, the plan read from the
decoders' own record (D059); §R6, that hook's claims corrected (D061); §R7, the occupancy admission (the Researcher's freeze review); §R8, the record's completeness by completion order (D062); §R9, its two qualifications (D063).
The coordinator froze it at `ff6c2f74` (6730600503). That freeze supersedes the draft
headings below, which are kept as history. The Guardian's D049 then held the timing
dispatch at that head (a B6 boundary defect, B5's reached fuzz evidence, and the copied
worker's M12). §R2 repairs and tests each obligation. Because `src/` changed, the
registration needs a new exact run pin (§R2.5). Its question, primary, reference and
thresholds are unchanged. The coordinator re-freezes it at the repaired head, or doesn't. Nothing here changes the default decoder, with or without the new feature:
only `StreamDecoder::pipelined_run_fed` builds the new arm. Nothing here runs on the
workstation, and no recipe is in the Justfile or the workstation queue.

**The input.** Louis's direction for line A, through the coordinator: make run-fed
construction pay, rather than abandon it. #269 (`2026-11-09-run-fed-stage.md`) found:
- the emitter `mtf::mtf_runs` costs about 25% less than `mtf_checkpoints`, the shipped
  pipelined leader's MTF (2 local rounds, indicative only);
- `run_stage::fill_rows` can fill any share of the table with no count checkpoints;
- the one-thread stage is about 1.0–1.1×.

So a win, if there is one, is on the pipelined leader's critical path. Folded in from the
reviews of #269 and of this line:
- the Researcher's: a fair incumbent (the shipped decoder through its own entry point),
  whole-entry-point timing, a strict VALID that refuses absent or non-finite intervals and
  failed children, and the raw paired timings archived; and #269's safety fix (a checked
  `Sorted`), onto which this branch is rebased, with every call into `run_stage` in one
  place;
- the Optimiser's (REVIEW 6729548266): C29, a matched control; C30, this arm, with its
  costs charged and the leader's gross release reported apart from the wall; C31, the rival
  with no F-order array, built as a sub-arm, with its census; C32, the AVX2 fill's
  overstores counted, and exact ownership at share edges.

**In short:**
- **Built:** the pipelined team at the shipped levers (park and small) with five sub-arms,
  all one worker, so they differ only in the leader's MTF and the helpers' build.
- **Correct:** whole-output equality with the default decoder on the corpora and on crafted
  run-heavy streams, every error at the same block with the same bytes before it, drops and
  cancellations without a hang, panics in every phase, the tables word for word, Miri on the
  new unsafe code.
- **The registration's question:** does the run-fed leader cut decode wall at three teams of
  three? The primary arm is `dir` (C31, no F-order array, its shares cut by C35), chosen
  prospectively but informed by the 2-round local smoke (§5, §R1). `hs` (C30) is secondary.
  The metric is the paired wall ratio against the shipped default, `mibz2-default33`, always,
  with CPU co-reported and capped.
- **The drafter's prior is STOP**, from the local smoke (§5): on that file the helpers, not
  the leader, end most runs, so a freed leader has little to release; and most of the
  emitter's gain over `mtf_checkpoints` is the list kernel (C29), not the records.

## 1. The mechanism

A pipelined run (`coop::invert_pipelined_with`): the helpers build and walk block i on the
team's table while the leader finishes block i − 1 (RLE1, CRC) and stages block i + 1 (its
MTF), then joins the walk. The decode's wall is set by whichever side ends later.

The run-fed leader changes the two stages that meet at the table:
- **The leader** runs `mtf_runs`: a 4-byte record a literal, nothing a run, no n-byte
  column and no count checkpoints, on every block `run_stage::takes_runs` (#269's rule)
  takes. The other blocks take today's path (`mtf_checkpoints`, then `coop::FusedBuild`).
- **The helpers** build the table from the records instead of the column, each its own
  share, then walk as today. The walk, its join, RLE1 and the CRC are unchanged.

What it can save is leader time: M − E − (the leader's new work), where M is the leader's
MTF with checkpoints and E the emitter. That shortens the run only where the leader is its
critical path. What it costs is helper time: the sort (if on the helpers), its barriers,
and the fill against `FusedBuild`'s share. That lengthens the run only where the helpers
are.

## 2. The design (`src/parallel/run_team.rs`)

### 2.1 The sub-arms

| sub-arm (`RunFedSort`, `ab` arm) | leader | helpers | question |
|---|---|---|---|
| `Control`, `mibz2-pipe3-runfed-ctl` | `mtf_checkpoints` (shipped) | `FusedBuild` | is this worker the shipped one? (against `mibz2-pipe3-both` and the incumbent) |
| `RegControl`, `-reg` (C29) | `mtf::mtf_checkpoints_reg`: checkpoints on `mtf_avx2`'s register-list kernel | `FusedBuild` | how much of E < M is the list kernel? |
| `Leader`, `-ls` | `mtf_runs`, then the stable counting sort, one share | fill shares | the sort where it is serial |
| `Helpers`, `-hs` (C30, secondary) | `mtf_runs` | count, prefix, scatter, fill shares | the sort parallelised |
| `Direct`, `-dir` (C31 + C35, **primary**) | `mtf_runs` | count rows, prefix, fill F intervals; shares cut by estimated work | no F-order array |

`mibz2-default33` is the incumbent: `BzReader::parallel` with `PIPELINED_PLAN` forced
(`parallel::with_plan`), the shipped default constructor on any machine, timed by `ab` from
opening the file to the last byte read, one process a run.

### 2.2 The choices

- **One worker for every sub-arm.** `run_team::worker` is `pipelined_worker::<true, true,
  BARRIER_SPINS>` (the shipped levers) with its front stage and build swapped by a
  constant. A child module of `parallel`, so it uses the shipped `Lead`, `Staged`,
  `finish_and_publish`, `publish` and `try_job` unchanged. `Control` measures what the copy
  and its timers cost, so no arm's difference is the worker's.
- **The sort's kernel is the same in `Leader` and `Helpers`.** `count_share`,
  `merge_shares`, `scatter_share`: a stable counting sort by records, with both
  histograms a share and byte (runs, for the slots; rows, for each run's a = C(c) + the
  rows before it), and `run_stage::sort`'s checks split by share. `Leader` runs it as one
  share on the leader. So `ls` against `hs` is placement alone. (It is also why `ls` doesn't
  call `run_stage::sort`: since #269's fix its checked `Sorted` borrows the scratch, and a
  leader's sort must outlive its stage, into the next run. The one-core path for small
  blocks does use `run_stage::sort` and `fill`, on a `Sorted`, through `api`.)
- **No `Sorted` is made outside `run_stage`.** #269's fix makes a checked `Sorted` the only
  way into `run_stage::fill` and `fill_rows`. The team's share fills never call them, and
  never make one: they are this module's own (`fill_share`, `fill_direct`), reading an F
  order that this module's checked path made (`sort_one_share`, or the helpers' prefix and
  scatter), and only once its flag says every check passed (the prefix's, then every
  scatter's exact fill). That flag is set nowhere else. So the share sort is a second
  checked constructor of a checked F order, kept beside its fills, not a `Sorted` forged
  from outside. Folding it into `run_stage` as a checked parallel constructor of `Sorted`
  is the natural step if this line goes further. It isn't taken here, as this PR leaves
  `run_stage.rs` to #269.
- **`Helpers`' synchronisation.** Two gates among the helpers (counted, then scattered),
  each a one-shot barrier whose last arrival runs the step (the prefix; the scatter's
  check) and then opens it, the others spinning `BARRIER_SPINS` and then parking. This is
  lever A's wait, with its no-lost-wake-up argument. Every helper always arrives at every
  gate, a panic caught and counted first, so no helper waits on one that won't come.
- **`Direct` (C31).** Byte c's F rows in helper j's record share are consecutive: the rows
  [C(c) + the rows of c before share j, + its own). So each helper fills them directly, in
  L order. Each run's a is the cursor of its byte, its L bytes found by one predecessor
  lookup a byte a share, then walked on, since consecutive F rows of a byte read
  consecutive L rows. No F-order array, no scatter, no run-count histogram, one gate. It
  pays up to 256 lookups a share, and 256 interleaved write streams a helper, against the
  fill's one stream a share.
- **C35, `Direct`'s partition (§R1).** Its shares are contiguous record ranges, as before,
  so `merge_rows`' byte-then-share prefix and its ownership argument are unchanged (they
  hold for any contiguous partition). But the cut is no longer equal records:
  `row_cuts` puts cut j at the first record i whose key, its L start plus
  `RECORD_WEIGHT` (8) · i, reaches j/h of n + 8r. The key rises with i, so each cut is one
  binary search on the thread setting the run up (O(h log r)), and the cuts rise and span
  [0, r) whatever the records hold. Weight 8 estimates the entries stored (W ≈ n + 7.6 r on
  the corpus): rows alone (weight 0) balance n_h but skew cells and stores (§4).
- **The fill's ownership and physical write footprint (C32, Guardian D044).** Every word a
  fill writes is in [q, `end`): a vector store covers [w, w + 8) and is issued only while
  w + 8 ≤ `end`, a scalar one writes w < stop ≤ `end`, and w only rises from q. `end` is the
  end of the rows the thread owns: the share's q1 in `fill_share`; byte c's F range in the
  share (`merge_rows`' hist[2][c]) in `fill_direct`, checked to contain the run before any
  store. So the tail at every ownership edge is exact (scalar), not an 8-word overstore: a
  range shorter than 8 rows, a one-row share among them, gets no vector store, and no store
  reaches row n (none relies on n + 8 slack, unlike `run_stage::fill`'s whole-table
  sweep). Those ranges are disjoint across threads by the prefix's construction, so no
  store from one share lands in another's rows. Inside its own range a vector store's words
  past `stop` are rows the same thread rewrites, as it fills the range in rising order to
  its end (checked; a fill that stops short returns false and the table is never exposed).
  The entries stored, W, are counted (`stores`) against the distinct words written, which
  are exactly the rows (`rows`, and each share's `share_rows_max`).
- **The checks before any fill.** The prefix makes `run_stage::sort`'s checks: the first
  run at row 0, the starts rising strictly (each share checks its records and the next
  share's first), the closing record n, each byte's rows summing to its count, and the
  counts to n. The slot and row ranges are disjoint by construction. The scatter writes
  only inside its ranges and must fill them exactly; the fill must write every row of its
  share. Only then is the table's length set. A refused check is a bug in the emitter or
  here, and the block is rebuilt by the column's path from its symbols, so the output is
  right whatever. The rebuilds are counted (`fallbacks`), and VALID requires 0.
- **The fallback rule** is #269's `takes_runs`, unchanged. Small blocks (below `MIN_COOP`,
  decoded whole by the leader) take records too where the rule says, and are built on one
  core by `run_stage::sort` and `fill`. On the Wikidata sample, 198 of 431 blocks are
  small, so this is on the leader's path, and is timed (`lead_small_ns`).

### 2.3 The phases (`parallel::run_fed_phases`)

Per-process totals, two `Instant` reads a phase a block, reported on each `ab` line of a
run-fed arm (`phases`):
- the leader's work in runs, split into MTF/emitter, sort, RLE1 and CRC, and small blocks;
- its wait for the build after its work (`join_wait_ns`: descheduling and wake-up
  included, not construction alone);
- one timeline a run, every moment from the same origin t0 (before the build is set up and
  the pool dispatched): the leader's start (`lead_start_ns`, d), its work's end
  (`lead_end_ns`, d + L), the build's end (`build_end_ns`, B), and the run's span;
- the helpers' sort and fill, summed;
- `leader_late`: runs whose leader ended its work after the build, d + L > B (§R1: it
  compared L with B before);
- the shares' census (C35), each run's largest share's records, rows, cells and entries
  stored (`share_*_max`), and the readiness: t0 to the last share's fill end
  (`ready_last_ns`) and its skew over the first's (`ready_skew_ns`);
- like-for-like blocks (`rule_*`): the leader's MTF or emitter and sort on the blocks the
  rule takes, counted in every sub-arm, the controls' included, so the gross release
  M − E − sort is computed on the same blocks;
- C31's lookups, C32's W, and the records r.

## 3. Correctness (checked)

| check | where | result |
|---|---|---|
| the helpers' sort in 1–5 shares = `run_stage::sort`'s F order; the fill in 1–5 shares, each kernel = the production table; C31 in 1–5 shares = the production table; W within [n, n + 7·2r]; C31's lookups ≤ 256 a share; a wrong count, a moved start, an emptied run and a moved end refused by every share count | `run_team::tests::share_kernels` (columns of 1–70,001 rows, σ 1–256, runs and uniform) | pass |
| a whole run (front stage, then `invert` on pools of 2–5) of every sub-arm: output and table word for word (a random column is no BWT, so its walk alone visits one cycle), leader early and late | `runs_are_walk` | pass |
| a tiling broken after the emitter: `Helpers`' and `Direct`'s checks refuse it, the column's path rebuilds it, output and table right | `a_refused_tiling_rebuilds_from_the_column` | pass |
| a panic injected in each helper phase (count, prefix, scatter, fill), `Helpers` and `Direct`, and in the leader: the run's Err (or the leader's panic resumed), no hang, the pool's next run right | `panics_in_each_phase` | pass |
| C29's control: `mtf_checkpoints_reg` = `mtf_checkpoints` (result, column, counts, checkpoints, `RunTooLong`, `BlockTooLong` at bounds around each length), each kernel | `register_checkpoints_are_checkpoints` | pass |
| every sub-arm through `StreamDecoder::pipelined_run_fed` at 1×2, 1×3, 2×3, 3×3, 1×4: byte for byte the default decoder's output and error (kind, block, bit) on the corpora, run-heavy full blocks (records taken: counted), small blocks among large across levels and streams, one block and none, a block CRC, a bad primary index, truncations, `RunTooLong` on the prefetched block and after a small one, `BlockTooLong`; no fallback | `tests/run_fed_leader.rs` | pass |
| a reader dropped mid-stream, and the push form cancelled while writing, every sub-arm and plan: no hang (a one-minute watchdog) | `dropped_and_cancelled_mid_stream` | pass |
| the share kernels' unsafe writes, scalar and AVX2, and a whole run's hand-offs between helpers and leader (the race detector) | `cargo +nightly miri test --lib -- run_team::tests::miri_`, with and without `-C target-feature=+avx2,+bmi1` | pass, both (2 tests each, about 140 s); CI's AVX2 step also asserts AVX2 was detected (`MIBZ2_EXPECT_AVX2`) |
| the default suites, unchanged code paths: `cargo test` (default features), and with `run-fed` | `cargo test` | pass |
| clippy (`--all-targets`, all features and default), fmt | | clean |
| the physical write footprint: each share of each fill (`hs`'s by rows; `dir`'s by equal records, by C35, one record a share) alone into a table padded with sentinels, both kernels: exactly its own rows changed, its distinct words its rows, the shares tiling [0, n); 1-row, 7-, 8-, 9-, 15-, 16-, 17-row shares | `fills_write_their_own_rows_only` | pass; the ownership bound moved 8 rows (`w + 8 <= end + 8`) fails it at once (n = 1, a word past the share) |
| every share at once, a thread each, one table: one-row shares (h = n), one-record shares (h = r), shares at the vector store's edges, `hs` and `dir` | `miri_parallel_footprint` (natively, and under Miri's race detector and Stacked Borrows, scalar and `+avx2,+bmi1`) | pass, both; under Miri with AVX2 the same mutation is UB at once (a write outside the share's borrow) |
| C35's cuts: rising, 0 and r at the ends, on any records (unrisen ones too); each share's work, rows + w a record, the h-th of the total give or take one record either side, w 0 and 8 | `row_cuts_partition_by_rows` | pass |
| `dir`'s table at equal-record and C35 cuts the same, 1-5 shares; the rows written n | `share_kernels` (`direct_by_shares`) | pass |
| the timeline: d = 20, L = 95, B = 100 is late (the duration's predicate called it early); a leader started 200 ms after t0 with no work is late, its duration under B | `leader_late_from_one_origin`, `a_late_started_leader_is_late` (`hs`, `dir`) | pass |
| the reporter's phase list is this module's, in order | `phases_are_the_reporters` | pass |
| the reporter's admission and reading: every probe of the Researcher's REVISE (all hash and output fields absent; every CPU negative; negative counts with the timing phases absent; the corpus label varying by round) refused at 15 rounds; absent and mistyped walls, CPUs, exits, rounds, outputs, NaN; malformed, duplicated, unregistered and negative phases; inconsistent counts; the plan not 3×3 seen; the arms out of ab's rotation; an unregistered arm or thread count; a missing round, failed child, different output, fallback, control with records, zero wall; the identity's head, tree, binary, corpus, features, profile, rustc, arms, rounds; a judged run without its head, identity or at other than 15 rounds. Each INVALID with its reason, none a crash. The Researcher's ctl ≈ 0.816, hs ≈ 0.945 fixture (dir too) reads LEAD with ATTRIBUTION FAILED, dir/ctl and hs/ctl ≈ 1.158 reported apart; an attributed LEAD; a STOP marked environment-specific; the primary dir, hs's reading only secondary | `bench/test_run_fed_leader.py` (24 tests, CI's submit job) | pass |

A mutation check: the vector store's ownership bound moved 8 rows past its end
(`w + 8 <= end + 8`) corrupts the heap at once in the parallel tests (`malloc(): invalid
size`), fails the footprint test at its first case, and is UB under Miri at once, so the
bound is exercised three ways.

**Not covered: B5.** An independent adversarial review of the new unsafe code (RULES B5) is
required before **any** timing dispatch of this registration, CI's included (Guardian D044:
B5 precedes a diagnostic, not only production). There is no diagnostic exemption: output
equality and the fallback cannot repair undefined behaviour that already happened. Still
owed for B5 (D044's list, beyond this PR's reach): source-pinned scalar and AVX2 Miri of
the emitter's record stores with reachability and exit-path coverage, and reproducible
emit/sort/fill/fallback fuzz and differential evidence. The share kernels' and the
parallel fills' Miri runs above do not reach the emitter.

## 4. C31's census and C32's count

From the local smoke (§5, indicative, a 4 MB test file of 133 blocks, 16 of them team
blocks by records), per decode:

| | value | against |
|---|---|---|
| r / n (records a row, team blocks by records) | 0.0756 | Wikidata's 0.062 (#266) |
| C31's predecessor lookups | 2,614 (≈ 163 a block) | the r scatters and 8(r + 1) bytes of F-order array it removes (≈ 45,000 a full block) |
| W / n, the fills' entries stored a row | 1.59 (`ls`, `hs`), 1.59 (`dir`) | 1: a cell averages about 6.6 rows, and its one vector stores 8 |

So C31's lookups are few against the metadata it removes (at most one a byte a share:
`lcur` resumes, §R1), though each is a dependent search and their cost is measured, not
assumed; its other prices are the 256 interleaved write streams and coherence traffic
where two helpers' ranges share a line (disjoint words are race-free, not free).

**C35's census (§R1),** untimed (counts, not times), the bench corpus (`corpora/
wikidata.bz2`, SHA-256 `82f65ace…`), its 186 team blocks by records of 431 (r/n 0.0620,
mean n 899,995), each run's largest share over the mean share, summed over blocks
(`run_team::tests::census`):

| h (helpers) | cut | r_h | n_h | m_h (cells) | W_h | W / n | lookups a block |
|---|---|---|---|---|---|---|---|
| 2 (3×3's) | `dir`, equal records (as first built) | 1.000 | **1.356** | 1.010 | 1.234 | 1.474 | 164 |
| 2 | `dir`, equal rows | 1.789 | 1.008 | 1.730 | 1.257 | 1.475 | 162 |
| 2 | `dir`, **C35** (rows + 8 r) | 1.387 | 1.182 | 1.356 | **1.007** | 1.474 | 163 |
| 2 | `hs`'s fill (rows by share) | 1.000 | 1.000 | 1.745 | 1.256 | 1.478 | 0 |
| 3 | `dir`, equal records | 1.000 | **1.877** | 1.062 | 1.597 | 1.473 | 203 |
| 3 | `dir`, equal rows | 1.627 | 1.002 | 1.618 | 1.210 | 1.474 | 229 |
| 3 | `dir`, **C35** | 1.490 | 1.358 | 1.374 | **1.019** | 1.473 | 232 |
| 3 | `hs`'s fill | 1.000 | 1.000 | 1.581 | 1.196 | 1.478 | 0 |

So the Optimiser's straggler is real: at the plan's two helpers, equal records leave the
larger share 36% more rows than the mean (88% at three). Equal rows fix n_h but skew cells
and stores the other way, as the corpus's runs are longer at one end of L than the other.
C35's weight balances the entries stored to within 1–2%. Which of rows, cells or stores
binds a helper's time is a cost the census can't give; the run's readiness skew
(`ready_skew_ns`, phases) is its measured check. `hs`'s fill, cut by rows, carries the same
cell and store skew (1.75, 1.26), unchanged here. W, the entries stored with repeats, is
1.47 n; the distinct words written are exactly n (the footprint tests). And the vector fill overstores by about 59%: every overstore is
into a line the same store or the next touches, so it is store bandwidth to L1, not
misses, but it is not free. A masked tail (`vpmaskmovd`) would make W = n; it is not
built, as its cost differs by microarchitecture (slow on AMD's), and the registration
measures the fill as #269 built it.

## 5. What was seen before this draft, disclosed (local, indicative, never evidence)

One smoke run, 2 rounds at 9 threads on the shared 4-core machine, of a 4 MB test file
(not the bench corpus), for validity only. VALID passed (exit statuses, hashes, phases).

- **The wall and CPU ratios** were all within noise of 1 (intervals spanning 0.2–9 with 2
  rounds): nothing.
- **The leader, a block the rule takes:** M (`mtf_checkpoints`, shipped) 0.224 ms, M
  (`mtf_checkpoints_reg`, C29) 0.128 ms, E (`mtf_runs`) 0.096–0.114 ms, the leader's sort
  0.085 ms. So against the matched control the records release about 0.015–0.03 ms a
  block with the sort off the leader, and nothing with it on (−0.05 ms). Most of E < M is
  the list kernel.
- **The critical path:** the leader waited for the build after its work in most runs (its
  work about 0.6–0.9 ms a run, its wait about 1.2–1.5 ms), and ended after the build in 10–18
  of 70 runs. That count used the old `leader_late`, its duration against the build's end
  from t0 (§R1), so it undercounts late runs. On that file and machine, the helpers looked
  like the run's critical path; a host and schedule observation, not an i9 result.

**Set before it:** the question, the metric, the incumbent and the arms (the primary was
then C30, as briefed). **Set after it:** the thresholds' values below, and (§R1) the
primary `dir` with `hs` secondary, and C35's cut. The primary was changed after seeing the
smoke, which showed `hs`'s sort on the helpers' side of a run the helpers already end, and
the Researcher and the Optimiser both recommend C31 on that mechanism; it is disclosed
here as smoke-informed, and the smoke is excluded from any judged sample. No judged data
exists for any arm. The coordinator may make any threshold stricter at freezing.

## 6. The registration (RULES C1). DRAFT, NOT FROZEN

- **The question:** does the run-fed leader, with its helpers building the table directly
  from the records (C31, no F-order array, shares cut by C35), cut decode wall in the
  pipelined plan, three teams of three with park and small?
- **The primary arm:** `mibz2-pipe3-runfed-dir`, chosen prospectively, before any judged
  data, but informed by the 2-round local smoke (§5): disclosed as such. `hs` (C30) is a
  fixed secondary, reported the same way and never judged. A win of `hs` or `ls` is not
  this registration's result, and a `dir` result is a C31 result, not C30's.
- **The reference, fixed:** `mibz2-default33`, the shipped default constructor
  (`BzReader::parallel` with `PIPELINED_PLAN` forced) through its own entry point. Every
  judged ratio is over it, whatever the controls show; no other denominator is ever
  substituted. Its baseline is master's code: this branch changes no line it runs.
- **One threshold set, on the primary at 9 threads** (unchanged from the draft):

| reading | condition |
|---|---|
| **LEAD** | wall ratio ≤ 0.97, its interval's high end < 1, and the CPU ratio's high end ≤ 1.05 |
| **STOP** | the wall ratio's low end > 0.99: no 1% cut |
| **INCONCLUSIVE** | anything else |

- **Attribution, stated with every reading, never a change of reference.** The control
  contrasts are reported apart, paired by round: `dir`/`ctl`, `hs`/`ctl`, `ls`/`ctl` (the
  records' and the construction's effect against the instrumented column path), `reg`/`ctl`
  (C29, the list kernel's), and `ctl`/`pipe3-both` and `ctl`/`default33` (the instrumented
  worker's fidelity). A LEAD is *attributed* to the run-fed construction only if `ctl`'s
  wall interval over the incumbent lies wholly inside [0.98, 1.02] (an interval that merely
  overlaps it is not equivalence) and `dir`/`ctl`'s wall interval's high end is below 1.
  Otherwise the reading is printed as LEAD with **ATTRIBUTION FAILED** and its reasons.
  `ctl` measures the worker's and its timers' cost on the column path; it does not subtract
  the record arms' own sorting, barrier or counter overheads.
- **The head, the build, the corpus:** this PR's full head at freezing, named by the
  coordinator and passed to the dispatch as `registered_head`. The workflow writes
  `rfl-identity.json` around the run (the head, a clean source tree, the build's features
  and profile, rustc, and the `ab` binary's and the corpus's SHA-256 before and after), and
  the reporter refuses the run unless the head is the registered one, the binary is one
  build throughout, and the corpus is `82f65ace…c29915` before and after. A dispatch
  without `registered_head` is reported UNJUDGED.
- **The run (CI):** `sweep.yml`, dispatched on this branch at the frozen commit, `tool:
  run_fed_leader`, `rounds: 15`, `registered_head: <the head>`. One dispatch: `ab run` of
  the seven arms in the registered order, rotated by round (ab's `arm_order`), at 9 threads
  (judged) and at 3 (reported, against `pipe3-both`: at 3 the incumbent is still three
  teams, so it is no fair reference there), unpinned, on the bench's Wikidata corpus.
- **VALID** (`bench/run_fed_leader.py report --head --identity`, every predicate, or INVALID
  with its reasons and no reading; every measurement checked for presence and type before
  any arithmetic):
  - exactly 15 rounds, each arm once a round at 9 and at 3 threads, no other arm or thread
    count, the arms in ab's rotation;
  - each child exited 0, with an empty error and an unbroken line;
  - each run's `hash` a non-empty string and `out` a positive integer, both equal to the
    incumbent's in its round, the incumbent's one value throughout;
  - each `wall` and `cpu` a finite positive number;
  - each run's `corpus` `wikidata` (one corpus, every round), and the identity's digests;
  - each 9-thread run seen running three leaders and six pipelined helpers;
  - each run-fed arm's phases every registered name once, each a non-negative integer:
    team runs > 0, team blocks by records and by the column summing to them, no fallback,
    `leader_late` ≤ runs, the leader's, the build's and the run's timings present, the
    leader's end after its start; the record arms' records, rows, shares, cells and
    readiness present, W ≥ n; the controls' records none; `dir`'s lookups present;
  - every judged interval present and finite.
  The whole runs file, every record in file order (the arms' actual order), the identity and
  the readings are written to `rfl-9.json`, and `rfl-*` archived as the run's artifact.
- **No re-run to erase a failure.** An INVALID (a mismatch, a fallback, a violated
  invariant, missing evidence) is kept and investigated, never re-dispatched to pass. Only
  an infrastructure failure before `ab run` wrote any record (a class B setup failure,
  docs/team/CI-INCIDENTS.md) may be re-run, once, at the same commit. A run after any code
  repair is a new head, needing its own freeze.
- **Reported, never judged:** `hs` and `ls` against the incumbent; the contrasts above; the
  gross leader release a rule block, against both controls; the timeline (`lead_start`,
  `lead_end`, `build_end`, `leader_late`) and `join_wait`; the census (each count's largest
  share over the mean, readiness last and skew); W / n and the lookups; the 3-thread cells.

**The stop criteria:**
- **Positive:** an attributed LEAD on CI permits freezing the workstation run below,
  unchanged but for its frozen commit, as the decision's evidence (CI's runners can't host
  the plan: §7). It permits no production change. A production prototype needs its own
  registration, CI's paired bench, and B5's review. A LEAD with ATTRIBUTION FAILED permits
  no freeze: it goes back for revision (what in the instrumented worker differs).
- **Negative, environment-specific:** a STOP on CI is this runner's reading, under 9
  threads on 4 vCPUs. It pauses this arm (C3's discretion against alternatives); it does
  not close the run-fed line or the target machine's question (PROTOCOL §11, on master
  since `6923ce43`: a STOP closes an arm, not a line Louis is pursuing). The Coordinator
  sends the measured phases to the Optimiser and the idea to the Researcher, as §11 asks.
  The draft's earlier rule (STOP with `ctl`'s `leader_late` under a quarter of its runs
  closes the line, reopened only by one workstation trace) is withdrawn.
- **Either way, C29 stands apart:** if `reg` is faster than `ctl` on the leader's MTF,
  that is a separate lead (the shipped `mtf_checkpoints_avx2` replaced by the register-list
  kernel), with its own registration; whether the leader's front is exposed is measured
  there, not assumed from the level.

## 7. What CI's 4-CPU Xeons can and cannot say about the i9

- **The runner:** 4 vCPUs, an unknown Xeon a run; its cpuset, quota and topology are what
  `ab env` records (`rfl-env.json`), not inferred from the count. Three teams of
  three is 9 decoding threads plus the scanner: 2.5 threads a CPU, 5 a core. The i9-10900K
  has 10 physical cores, and the plan was chosen there because 9 threads each get a core.
- **It can say:** whether the arm's output is right on this corpus under heavy contention
  (VALID: not universal correctness or the absence of deadlock); the wall and CPU ratios on
  this runner; the waits and the shares' readiness observed here.
- **It cannot say:** the i9's critical path, nor invariant work. CPU is scheduled service:
  spinning, SMT sharing, frequency and cache interference all change it. Every phase clock
  includes descheduling. With threads time-sharing cores, a leader released on CI frees a
  CPU for a helper rather than ending a run sooner, and `hs` and `dir` change the
  synchronisation itself, so the wall is no clean "less total work" test either. A
  construction can shorten the target's span while its total service is unchanged or
  larger (the Researcher's example: L 10 → 8, helpers 6 → 7 each), and read as no gain or
  a loss here. `leader_late` on CI is a contention figure, not the i9's.
- **Hence** the CI dispatch is a bounded, environment-specific screen: an attributed LEAD
  there justifies the workstation run; a STOP there pauses the arm, never closes the line.

## 8. The workstation run: its registration, recipe and admission (§R4; NOT FROZEN)

CI's run at `03e52206` was VALID (JUDGED), an attributed LEAD: `dir`/`default33` wall
0.9669 [0.9594, 0.9744] at 9 threads (run 42564217298;
`2026-11-09-run-fed-leader-result.md`). Under §6's positive criterion, that permits
freezing this run, unchanged but for its frozen commit. The Researcher's review
(6732761085) asked for this section to be revised first, and it is (§R4).

This section is **NOT FROZEN** until `bench/run-fed-leader.frozen` names its commit. Until
then the recipe refuses every run but a DRY one. CI's success authorizes no workstation
run; Louis's own run of the frozen recipe, submitted by digest, remains necessary under C5.
Nothing here authorizes a C6 choice.

**The arms, unchanged:** the seven, in this order, with the reference fixed:

| # | arm | role |
|---|---|---|
| 0 | `mibz2-default33` | the incumbent, the fixed reference of every judged ratio |
| 1 | `mibz2-pipe3-both` | the shipped worker through the research constructor |
| 2 | `mibz2-pipe3-runfed-ctl` | the instrumented control (attribution) |
| 3 | `mibz2-pipe3-runfed-reg` | the register-list control (C29) |
| 4 | `mibz2-pipe3-runfed-ls` | secondary, never judged |
| 5 | `mibz2-pipe3-runfed-hs` | secondary, never judged |
| 6 | `mibz2-pipe3-runfed-dir` | **the primary** |

**The run:**
- exactly **20 paired rounds at a requested 10 threads**: 140 timed records, the arms
  rotated by round (ab's `arm_order`);
- every run observed running three leaders and six pipelined helpers;
- `--settle 0.25`, ab's default busy gate, unpinned;
- the bench's Wikidata sample, built by `bench/wikidata.py` from `bench/wikidata.json`, its
  SHA-256 `82f65ace…c29915` (CI's corpus) checked when it is built and before and after
  the run.

At 10 threads every arm is three teams of three: the incumbent selects that plan by itself
there, and is forced here only so that CI and the workstation run the same arm. There are
no 3-thread cells. The Researcher's recomputation found that CI's one-team cells don't
rank the arms as the 9-thread cells do, which supports measuring the target, not
predicting it.

**The reading, prospective, on the primary at 10 threads.** These are §6's tables
unchanged. Each ratio is the geometric mean of the 20 paired ratios over the incumbent,
with its 95% t-interval on the logs (df = 19):

| reading | condition |
|---|---|
| **LEAD** | wall ratio ≤ 0.97, its interval's high end < 1, and the CPU ratio's high end ≤ 1.05 |
| **STOP** | the wall ratio's low end > 0.99: no 1% cut |
| **INCONCLUSIVE** | anything else |

**Attribution, stated with every reading, never a change of reference:**
- A LEAD is *attributed* only if `ctl`/`default33`'s wall interval lies wholly inside
  [0.98, 1.02] and `dir`/`ctl`'s wall interval's high end is below 1. Otherwise it is LEAD
  with **ATTRIBUTION FAILED** and its reasons.
- Reported apart, never judged: `ls` and `hs` over the incumbent (with their readings),
  and the control contrasts `ls`/`ctl`, `hs`/`ctl`, `dir`/`ctl`, `reg`/`ctl` and
  `ctl`/`pipe3-both`.
- The post-run descriptive contrasts the Researcher noted (`dir`/`reg`, `ls`/`reg`) stay
  descriptive: they are no gate. Nor is "direct fill adds 0.8%" a causal reading: `reg`
  uses C29 on every block, while the record arms keep the old kernel on blocks the rule
  doesn't take.

**VALID** (`bench/run_fed_leader_ws.py admit DIR`, every predicate, or INVALID with its
reasons and no reading; fail-closed: a missing, mistyped or extra field is a reason, and
an error of the admission itself is INVALID):
- **the identity** (`identity.json`, `run_fed_leader/workstation-identity/1`):
  - the run's head;
  - the frozen commit, with HEAD's measured inputs identical to it;
  - a clean tree before and after;
  - the recipe's, the admission's, the reporter's and this registration's SHA-256 at the
    head, each equal to the file's at the frozen commit;
  - features `run-fed` (and `trace,run-fed` for the captures), profile `release`, rustc;
  - the `ab` and `occupancy` binaries' SHA-256 before and after the run, each one build;
  - the corpus's SHA-256 before and after, equal to `82f65ace…`;
  - the arms in this order, threads `[10]`, 20 rounds, and `ab run` exiting 0;
- **the records** (`runs.jsonl`): CI's typed checks of each record, reused
  (`record_problems`):
  - exit 0, empty error, an unbroken line;
  - a hash and positive output, each equal to the incumbent's in its round, and the
    incumbent's one value throughout;
  - finite positive wall and CPU, and the corpus `wikidata`;
  - the plan (3, 0, 6), read from the decoders' own record of the threads they ran
    (`ran_threads`, §R5), not from the census's polling, which is kept, required present,
    and reported, never judged. Its scope (§R8): ab's fresh child turns recording on before
    the decode spawns any thread, and reads it once it has read the decode's whole output.
    By the order of completion, not by any join, that output comes after the record of
    every worker that published a block and of every helper of every team run that built
    one. So the record includes every thread that contributed, in every arm (§R9: it need
    not contain only contributors, as a worker records its start whether or not it
    publishes). Its entries are worker starts and helper first task attempts (a caught
    panic included), not useful work, nor that the threads ran at the same time. A
    required count missing from it is INVALID. It says nothing about teardown, which may
    still be running when it is read;
  - for the run-fed arms, every registered phase once, as non-negative integers, with
    consistent counts, no fallback, and the controls with no records;
  - then threads 10 only, rounds 0–19, each arm once a round, in ab's rotation, and
    exactly 140 records;
- every judged interval present and finite.

**What each run's wall and CPU include** (§R6, stated in the admission's output, never
subtracted): both include the thread record's pushes during the decode, one a thread under
one process-wide lock, each allocating its name. The record's read, sort and join come
after the child's wall, CPU and RSS readings, so none of them includes these. Every arm's
child runs the same code. That is not equal cost: a helper records before it publishes its
first task's completion, and a worker before its first job, so a push under the shared lock
can extend a run, and when threads start, and so what they contend with, differs by arm.
The instrumentation can be differential, and its cost is not measured. No claim that it is
small, uncontended or equal is made (§R6).

**The occupancy captures, admitted apart** (`admit-occupancy DIR ctl|dir`). They are made
after the timing and never enter its judgement. Each is one capture of
`examples/occupancy.rs --threads 10 --pipe 3 --run-fed SUB --rounds 6`: its three modes,
off, timeline and full, six observations per mode, with ab's run-fed arm's window. Each is admitted only
if:
- it is the instrument's format;
- it shows the sub-arm at 10 threads in three teams of three, with 6 rounds;
- every round is complete in both traced modes, with blocks and team runs traced;
- each team's buffers are recorded (3);
- its raw grid is whole (§R7): each of the 6 rounds × 3 modes observed exactly once, at 10
  threads, with the sub-arm's plan, no error, a finite positive clock, and the timing's
  incumbent's bytes and hash;
- its commit and corpus are the identity's;
- both traced modes' perturbation comparisons (timeline and full, each over off, round by
  round) are present and finite, each equal to its recomputation from the grid, and each
  verdict the one its interval gives.

The perturbation is reported, not judged against the timing. A capture is
**interpretable** only if both comparisons say "no perturbation shown". A capture whose
traced modes perturb is admitted, marked "its timeline not to be interpreted". A missing,
unknown or insufficient comparison ("too few rounds"), or a grid missing an observation,
is INVALID, never interpretable. A capture's failure never touches the timing's
validity. The captures' purpose is the i9's
critical path, seen rather than inferred: who ends each run, the leader or the helpers.

**The bundle** (one directory per attempt, `target/run-fed-leader/<UTC>`, packed by
`bench/bundle.sh` however the attempt ends):
- `console.log`, `outcome.txt`, `commits.txt`;
- `identity.json`, `inputs.txt` (every digest and the toolchain), `env.json`, `lscpu.txt`;
- `runs.jsonl` (partial if the run stopped);
- the two captures with their text summaries;
- the admissions (`admission.*`, `occupancy-*-admission.*`);
- the binaries that ran (`bin/`).

Partial and failing evidence is kept, never cleaned up. The outcome is one line: the
timing's VALID or INVALID, then each capture's. The exit status is 0 for VALID, 1 for
INVALID or REFUSED, and anything else for an attempt stopped before its verdict.

**No re-run to erase a failure**, as §6 says. An INVALID is kept and investigated, never
re-run to pass. Only a failure before `ab run` wrote any record may be re-run once, at the
same commit. A run after any code change is a new head, needing its own freeze.

**The maintainer's commands, once frozen:**
`git fetch origin research/run-fed-leader && git checkout <the frozen head>`, then
`just submit bench-run-fed-leader`. This section's commit is not frozen: there is nothing
to run until the coordinator names the frozen commit in `bench/run-fed-leader.frozen` and
queues it. It is not in `bench/workstation-queue.txt`, and no tag is set.

**Stop criteria:**
- an attributed LEAD on the workstation is the decision evidence §6 named. It permits no
  production change by itself: a production prototype needs its own registration, CI's
  paired bench and B5's review;
- a STOP closes this arm on the target machine (PROTOCOL §11: not the line);
- INCONCLUSIVE, or a LEAD with ATTRIBUTION FAILED, goes back for revision.

## 9. Limitations

- **One corpus judged.** The rule's share of blocks, and of small blocks, is the corpus's.
- **The rule is #269's,** decided before MTF from the symbols and the bound, so short
  blocks may take records at a loss on the leader (`lead_small_ns` shows it).
- **The fill overstores** inside its own rows (W / n ≈ 1.47 on the corpus, 1.59 on the
  smoke file); not cut (§4). C32's exact tails or masked residuals need their own matched
  fill-and-walk contrast: fewer words stored is not fewer store instructions.
- **C35's weight is an estimate** from an untimed census (§4), not a measured cost; the
  readiness skew is its check. No equal-record `dir` arm is timed beside it.
- **The instrumented worker** carries two clock reads a phase a block; `ctl` against the
  incumbent measures what that and the copy cost.
- **No adversarial review** of the new unsafe code yet (B5): it precedes any timing
  dispatch (§3).

## R1. Revision, 2026-11-09: the reviews of `f94551ab`

Three reviews of the draft at `f94551ab8c359ec28dca02956ddca150595cc7cb`: the Researcher's
REVISE (6730026367; report at `team-state/researcher` `245b375c`), the Optimiser's REVIEW
(6729991838; `team-state/optimizer` `08abbe68`), and the Guardian's D044 (6729932417). Each
finding, its fix and its test:

| # | finding | fix | test |
|---|---|---|---|
| R.1 | VALID admits missing evidence: absent hash and output, negative CPU, negative counts with timing phases absent, varying corpus labels, all VALID at 15 rounds; the head, build and corpus not bound | `bench/run_fed_leader.py` rewritten: typed raw evidence per run before any arithmetic (§6's VALID); `report --head --identity` binds the frozen head, one binary, the registered corpus digest, 15 rounds, the arms and threads; any other run `--unjudged`; a reporter error is INVALID, never a crash or a VALID. The workflow writes the identity and passes `registered_head` | every probe as a fixture, refused with its reason, plus the identity's and the types' (`test_run_fed_leader.py`, 24 tests) |
| R.2 | the journal substitutes `ctl` as the reference after a fidelity failure; the reporter never did; ctl ≈ 0.816, hs ≈ 0.945 read LEAD | the reference is `default33` always; the control contrasts reported apart; attribution stated with every reading, ATTRIBUTION FAILED where `ctl` isn't inside the band or the primary doesn't beat it (§6). The substitution sentence is withdrawn | `test_researcher_fixture_is_no_clean_lead` (dir and hs at 0.945, ctl at 0.816: LEAD, ATTRIBUTION FAILED, dir/ctl and hs/ctl ≈ 1.158), `test_attributed_lead` |
| R.3 / O.counter | `leader_late` compared L (from the leader's start) with B (from t0): late is d + L > B | one timeline from t0 (`Timeline`: `lead_start`, `lead_end`, `build_end`), `leader_late` = `lead_end > build_end`; the phases `lead_start_ns`, `lead_end_ns`, `build_end_ns` replace `build_ns`. `join_wait` is reported as including scheduling (§2.3, the reporter's note) | `leader_late_from_one_origin` (d = 20, L = 95, B = 100: late; the old predicate: early), `a_late_started_leader_is_late` (a 200 ms delayed start through `invert`) |
| R.4 | the diagnostic B5 exemption and the blanket re-dispatch after a VALID failure | both removed: B5 precedes any timing dispatch (§3); only a setup failure before any record may be re-run once (§6) | `test_no_retry_or_exemption_in_the_reporter` (the reporter has neither) |
| R.5 | CI STOP cannot close the target line; CPU is scheduled service; phase clocks include descheduling | §6's negative criterion environment-specific (PROTOCOL §11), the old closing rule withdrawn; §7 rewritten; the reporter prints both notes | `test_stop_is_environment_specific` |
| R.6 | choose `dir` prospectively, disclose the smoke, keep `hs` secondary | `PRIMARY` is `dir`, `hs` and `ls` secondary (never judged); §5 and §6 disclose the smoke-informed choice; one primary, one threshold set, one reference | `test_primary_is_dir_secondary_hs` |
| O.C35 | equal records, not equal row work: a straggler can erase the saved sort | C35: `dir`'s shares cut by `row_cuts` (rows + 8 a record), contiguous, the byte-prefix ownership kept; the census by share in the phases (`share_*_max`, `ready_last_ns`, `ready_skew_ns`), and untimed on the corpus (§4: n_h 1.36 → 1.18, W_h 1.23 → 1.01 at two helpers) | `row_cuts_partition_by_rows`, `share_kernels` (`dir` the same table at both cuts), `census` |
| O.C31 | lookups one a byte a share (`lcur`), not a run; the 45k scatters are F-order metadata, not table stores | §4's wording | (source) |
| G.D044 | no unmasked overstore may cross into another share's rows; one-row shares; W the distinct words | the footprint argument (§2.2, `cell`'s doc): exact scalar tails at every ownership edge, no n + 8 slack; W (with repeats) and the distinct words (= rows) both counted | `fills_write_their_own_rows_only` (sentinels, each share alone, 1-row to 17-row shares, both kernels), `miri_parallel_footprint` (every share at once, one-row and one-record shares, Miri scalar and AVX2), the `end + 8` mutation failing all three |

**Not closed here:**
- **B5** (D044's list): the emitter's source-pinned scalar and AVX2 Miri with reachability
  and exit-path coverage, and reproducible emit/sort/fill/fallback fuzz and differential
  evidence. The HOLD on timing stands.
- **The critical path:** an occupancy trace and matched interventions (the Researcher's §3);
  the timeline gives completion order, not the recoverable wall.
- **C32's tails, C36's leader help and early walking:** not built (the Optimiser's ranks 3, 4
  and last).
- **C35's weight** is an untimed estimate; no equal-record `dir` arm is timed beside it.
- **The workstation recipe** needs its own admission and freeze (§8).

## R2. Revision, 2026-11-09: the Guardian's D049 on `ff6c2f74`

D049 (#145 6730740928; `team-state/guardian` `d5d6f292`, `state/decisions.md` §D049, its
records in `records/d049/`) accepted the physical ownership argument for valid share indices
(C35 included). It held the dispatch on three obligations, and asked for a new exact run pin
after any repair. Each obligation, its fix and its test follows. Every result below was
**checked** on this machine (the shared 4-core box, `rustc 1.97.0`, nightly
`1.101.0-nightly (16d1ffac1 2026-10-26)` for Miri, `MIRIFLAGS` unset apart from CI's AVX2
forwarding), with each run's memory capped at 6 GB of address space. The logs are in
`2026-11-09-run-fed-leader/`. Nothing here was timed, and nothing needed to be: these are
correctness checks.

This branch also merges master twice:
- `66edecbe`: #273's pipeline fuzz candidates and the fuzz-smoke job, on which the run-fed
  candidates build. That merge changes no line of `src/` or Cargo.
- `c5f0f030`: #275 (`8be686b5`), the shipped worker's M12 hook and tests. The run-fed
  worker's twin now uses that hook itself (R2.4). The only conflict was `miri.yml`, resolved
  as the union of both path filters and both descriptions. Its selection check now also
  requires the run-fed worker's recovery tests (correction, D054: it requires at least one
  unwind test of each worker, by name prefix, not all six).

| # | D049's obligation | fix | test (all pass at the repaired head) | mutant (killed) |
|---|---|---|---|---|
| R2.1 | **B6:** the safe constructor takes k = 66. The pool runs 65 helpers, but `RunFill` holds 64 share entries, and helper 64 forms `&mut *self.hist.add(64)` (`count`) before any check | `RunFill::owns(j)` (`j < shares`), asked first by `count`, `scatter` and `fill`, before any index into `hist`, `cuts` or `census`, or any bit `1 << j`. This is how `FusedBuild::share` returns for `j >= shares`. A surplus helper still calls every phase and passes both sort gates and the built gate (`Relay::build_share`), so the gates' counts, of all the helpers, stay whole. `records_of` debug-asserts `owns`, and `RunFill::new` debug-asserts 1 ≤ shares ≤ 64. Rejecting k at construction was the alternative; the guard keeps `pipelined_run_fed` taking any k, as the shipped `pipelined` does | `helpers_past_the_shares_share_nothing`: every phase by hand at 1, 2, 63 and 64 shares, with 0, 1 and 2 surplus helpers (64 shares with 65 and 66 helpers is k = 66 and 67), in LS, HS and DIR, on each kernel. Each run checks: the table; the scatter, claim and fill bits exactly the shares'; no census from a surplus helper; and a sentinel past the shares untouched. It also runs a fresh exact 64-entry buffer, D049's witness. `miri_helpers_past_the_shares`: two shares with 2–4 helpers, the exact buffer under Miri (D054: the surplus under Miri is two shares with 2–4 helpers; the large-share cases, 63 and 64 shares and the 64–80-thread pools, are native only). `teams_past_the_shares_are_walk`: whole runs on pools of 64, 65, 66, 67 and 80 threads, every mode, each kernel. `the_constructor_past_the_shares`: `pipelined_run_fed(1, k, ..)` at k = 64–67, every sort, each kernel, two run-heavy full blocks byte for byte, `threads() == k`, `team_records` reached | B6a (`owns` always true): all four fail (an index panic; a shift overflow; a heap-corruption abort on the pools). B6b (count's guard deleted): all four fail, at the `owns` debug assertion. B6c (count's guard **and** that assertion deleted): all four fail: a slice-bound panic, and heap-corruption aborts (`free(): invalid pointer`, `sysmalloc`): D049's invalid access, made real natively. **Under Miri, B6c is UB at the witness** ("constructing invalid value of type `&mut [[u32; 256]; 3]`: encountered a dangling reference", at count's `self.hist.add(j)`) |
| R2.2 | **B5:** bounded, pinned fuzz/property evidence reaching the new constructor, the private sort and fills, and C29, against production; CI's fuzz job builds without `run-fed` | `examples/fuzz.rs` MODE `run-fed` (with `--features run-fed`): eleven `pipelined_run_fed` candidates (`tests/common`'s `run_fed_candidates`). Each case is held to the plain parallel decoder (bytes before any error, the error's kind, block and bit) and to libbzip2, on `run_fed_corpus` (the pipeline corpus, plus crafted run-heavy full and small blocks, and run-heavy and random full blocks alternating). Each case's reach is counted from the run-fed phases, and any rebuild after a refused check fails the run. CI's fuzz-smoke builds it and runs it 30 s at seed 1. For tilings valid emission can't make, an internal harness (`tampered_tilings_are_refused_or_tile`) checks them against an independent model | the fuzz runs below; the tilings harness and C29's (R2.3) | — (the harnesses' sensitivity: R2.3's mutants; the fuzz's is the pipeline harness's, #273) |
| R2.3 | **C29:** targeted checks of `mtf_checkpoints_reg`'s checkpoints and raw stores, its errors included, against `mtf_checkpoints` | `register_checkpoints_reach_every_class`: 2,811 seeded streams. They are columns whose runs and bytes land on and about the checkpoints, each at bounds −65 to +65 about its length, plus 2,000 malformed streams (random symbols, alphabets and bounds; some run-heavy; some with no end-of-block symbol), on each kernel. It compares the result always, and the column and counts on errors too (both are `mtf`'s on every input), the checkpoints where Ok. Every class of an independent input model is asserted reached. `miri_register_checkpoints`: one stream a class, AVX2 only (without AVX2 the control *is* `mtf_checkpoints`' scalar twin) | classes reached (native, both kernels): a run split at a checkpoint 1,575; a run ending at one 211; a byte at one 861; a block ending at one 172; runs by the vector stores 761,427; runs ≤ 64 by bytes at the bound 627; runs > 64 12,621; Ok 1,439; BlockTooLong in a run 901, at a byte 317; RunTooLong 129; no end-of-block 25 | C29a (a split run's checkpoint taken before its part is counted): all three C29 tests fail. C29b (the final checkpoint at a block ending on one dropped): all three fail. C29c (the 64-byte vector run store taken while n < max, not n + 64 ≤ max): a heap-corruption abort natively; **under Miri's AVX2 selection, UB**: a 32-byte store 31 bytes from the allocation's end (`mtf.rs`'s `run_store!`) |
| R2.4 | **M12 on the copied worker:** `panics_in_each_phase` calls `invert` directly. Reach the real outer `Err` with deterministic injection; current, current + next, and current + held prior; once-only publication at the right place and bit as `Internal`; a bounded reader, drop and join; kill the omission | **The shipped worker's own hook** (#275, master `8be686b5`), not a copy. `parallel::Fault` (`LeadStart`, `LeadEnd`) and `parallel::inject` sit at the same two points of the copied worker's leader part. `Shared::faults` (`parallel::Faults`) holds the fault, what the team held, every publication (`parallel::publish` records them) and the outer recoveries, which the copied worker's `Err` counts. `parallel::min_coop` gives the threshold. The tests and checks are the shipped twin's, and they share its `TINY` fixture: `tiny_chunk` and `TINY` became `pub(super)`, which is test code only. Before #275 landed, this revision kept the same design in a registry of its own (`8db31b0e`), read from that branch's worktree; the merge replaced the registry. No shipped code changes beyond #275's. Outside `cfg(test)`, the copied worker's one change is `stage`'s threshold, now `min_coop(shared)`, which returns `MIN_COOP` | `parallel::run_team::unwind_tests`, every sub-arm's worker: the three states on four full run-heavy blocks taken by the records' rule (asserted), on teams of 2 and 3 (`a_run_panic_retires_*`); on four tiny blocks, natively every arm and under Miri the three record arms (`miri_run_fed_panic_*`); and `pipelined_run_fed` at 3×3 end to end from the scanner, every sort, three faults. Each checks the reader's prefix and its `Internal` error at the first held block's place and bit; that the fault fired in the asked state and the outer branch recovered exactly once; each block published exactly once, every held one `Internal` at its own place and bit and no other an error; the worker ending by itself; the blocks after the reset decoded right; and the join. Every wait is under a 60 s watchdog | M12r-current-next (the loop over current and next `.take(0)`: M12's own shape): 7 of 9 fail, every fault test, at the watchdog (correction, D054: the logs kept, rerun after the #275 merge, hold 8 tests: 1 passes and 7 fail; for M12r-held-prior, 5 pass and 3 fail): "the reader is stuck at place 1, with places [3] ready" (D047's stranding case), "3 of 4 blocks published, places [0, 1, 3]". M12r-held-prior (the held prior's publication filtered out): the three held-prior tests fail at the watchdog ("stuck at place 1, with places [2, 3] ready"), and current and current + next pass, as they should |

**R2.2's fuzz runs** (`2026-11-09-run-fed-leader/fuzz/`; the harness at `7325d1e6`, `src/`
identical to the repaired head's; the fuzz profile, debug assertions and overflow checks on):

| run | cases | failures | valid, same bytes | error kinds, each as the plain decoder | cases reaching the pipeline (a job taken overlapped) | with an overlapped job's error | team blocks by records | small blocks by records | rebuilds after a refused check |
|---|---|---|---|---|---|---|---|---|---|
| 30 s, seed 1 (CI's) | 875 | 0 | 112 | 16 | 418 | 9 | 519 | 336 | 0 |
| 240 s, seed 2 | 6,615 | 0 | 885 | 19 | 3,135 | 67 | 3,928 | 2,983 | 0 |
| 240 s, seed 3 | 6,721 | 0 | 861 | 18 | 3,208 | 58 | 3,993 | 3,312 | 0 |

**Correction (D054):** the team-blocks-by-records column above is overstated. Its sums also
counted each candidate's "cases with team records" figure. The totals are **348, 2,660 and
2,719**. The other columns, and the per-candidate figures below, are right. These runs
predate the #275 merge, which changed `pipeline_corpus` (`tests/common`), and with it every
case's input: they did not meet §R3's empty block.

The figures in the last six columns are summed over the candidates. Per candidate, in each
240 s run:
- every record sort (Leader, Helpers, Direct, at each plan and on both kernel sets) reaches
  team blocks by records (301–382), small blocks by records, and the column's team and small
  blocks;
- the controls reach none by records, as registered;
- every candidate but one reaches overlapped jobs (182–364 cases) and overlapped errors
  (1–13 cases).

The exception is Helpers 1×2 in one slot, which never overlaps (one block in flight). It is
there for the one-helper team on the scalar kernels, and it reaches records too.

**R2.2's tampered tilings** (6,000 seeded cases, every sixth also as whole runs on a pool of
three, by the helpers' sort and the direct fill; Miri: 20). Each tiling is checked by
`run_stage::sort`, the leader's sort, and the helpers' sort and the direct fill in 1, 2, 3
and 5 shares on each kernel. Each must take it iff the model says it tiles, and then make
exactly that column's table; otherwise it must refuse it, and the whole run then rebuilds
the untampered block from its symbols. All agreed:

| tamper | refused | taken |
|---|---|---|
| a start moved; a record dropped; a record doubled; the closing record moved; the first start moved; a count moved | 600 each | 0 |
| a byte changed | 599 | 1 |
| two neighbours' bytes swapped | 535 | 65 |
| two equal runs' bytes swapped (another column) | 0 | 600 |
| a run split in two (the same column) | 0 | 600 |

**Credited, not relabelled:** the fixed-seed share differential tests (`share_kernels`,
`miri_share_kernels`), `runs_are_walk`, `panics_in_each_phase` and `tests/run_fed_leader.rs`
(9 tests, five plans, every sort against the default decoder and libbzip2) remain
component and differential evidence. They are not a composed campaign. The composed evidence
is the fuzz runs and harnesses above. D048's emitter and `run_stage` evidence transfers as
component evidence, as D049 §4 says.

**Miri** (`mutants/logs/miri-restored.log`, at `b82a7abc`), both of CI's selections:
- scalar/default, `run_team::tests::miri_` and `run_team::unwind_tests::miri_`: 9 passed,
  293 s wall, 440 MB peak;
- AVX2 (`+avx2,+bmi1`, `MIBZ2_EXPECT_AVX2=1`), `run_stage::tests::miri_` and
  `run_team::tests::miri_`: 8 passed, 242 s wall, 478 MB peak.

After the #275 merge, both workers' unwind tests, `unwind_tests::miri_`, pass under Miri:
6 tests, 178 s wall, 600 MB peak (`mutants/logs/miri-unwind-merged.log`, at `5a350c84`).
CI's selection check lists all six; it requires at least one of each worker's (D054). `miri.yml` triggers on the union of both branches'
paths: `src/run_stage.rs`, `src/parallel.rs`, `src/parallel/**` (which `run_team.rs` and
`run_team/**` also name), `src/mtf.rs` and `src/read.rs`, beside the walks', the pool's and
the workflow's.

**Native suites at the merge head `5a350c84`** (`mutants/logs/restored.log`):
- `cargo test`: the library 89 passed, 2 ignored, the shipped worker's 8 unwind tests
  included; every integration and doc suite passed;
- every run-fed CI step: `run_team` 28 passed and the one ignored census; `run_fed_leader` 9;
  and the rest;
- `run_team` with `parallel::unwind_tests` (36 passed), 5 more times;
- `cargo test --example ab`. clippy `--all-targets -D warnings`, default and all
features: clean. fmt: clean. Peak memory: under 0.6 GB for any run.

**Mutants, restored:** every mutant's patch is in `mutants/`, and each was applied with
`mutants/mutate.py` (an exact text replacement, applied once) and its log written. Each was then
restored with `git checkout`, and `src/` checked identical to the commit. The suites above
ran on the restored tree. The M12 twin's two mutant logs are at `5a350c84`, on the unified
hook. The B6 and C29 native logs are at `7325d1e6`. The two Miri mutant logs are at
`8db31b0e` (B6c) and `b82a7abc` (C29c). The lines each touches are unchanged since.

### R2.5 The run pin

- **Unchanged:** the question, the primary (`dir`), `hs` and `ls` secondary, the reference
  (`mibz2-default33`), the threshold set, attribution, VALID, the seven arms, 15 rounds, the
  corpus and its digest. `bench/` is unchanged since `ff6c2f74` (correction, D054:
  `bench/run-queue.sh`, the workstation queue's runner, changed through the master merges.
  Neither the A/B, its reporter `bench/run_fed_leader.py` nor `sweep.yml` changed).
- **The implementation pin** (superseded by §R3's): `5a350c8469b9d19c96b4e8e4facbcbfc6274b7ad`, the merge of master
  `c5f0f030` (#275) and the last commit to change `src/`, `tests/`, `examples/`, `bench/` or
  Cargo (the run's identity checks a clean tree in exactly those). It supersedes
  `7325d1e6`, the pin before #275 landed. The arms the dispatch times run code at that commit. Compared with
  `ff6c2f74`, their decoding paths differ only in the surplus-helper guard, which no plan of
  three or fewer helpers reaches, and in `stage`'s threshold, now `min_coop(shared)`, which
  returns `MIN_COOP` outside tests. The incumbent `mibz2-default33` runs master's code at
  `c5f0f030`, #275 included, whose only change outside `cfg(test)` is the same
  `min_coop(shared)`.
- **The frozen head:** this PR's head carrying this section. Every commit after the
  implementation pin changes only `docs/`. The dispatch's `registered_head` is that full
  40-character head, as the coordinator names it at the re-freeze: a run is judged only
  at it (§6). `ff6c2f74` is no longer a valid pin: a run there would be at code D049 held.

**Not closed here:**
- **The unsupported-CPU forgeable `Kernels`** issue that D049 names is neither repaired nor
  waived (unchanged).
- **A panic outside the run** (the worker loop's own code between runs) is not injected,
  as on the shipped worker's twin. Nor is one in `invert`'s set-up before the pool's run.
- **Miri with the M12 mutants** was not run. Their native kills are deterministic, by a
  bounded watchdog. Under Miri a stranded reader waits on a Miri clock of a day, and the
  shipped twin's Miri M12 run was killed for memory on this machine.
- **No fuzz of k > 4 teams:** the 64–67-thread boundary is covered by R2.1's tests, not by
  the fuzz loop.

## R3. Revision, 2026-11-09: CI's fuzz-smoke at `a08be2e9` (an empty block refused by the leader's sort)

**The failure.** CI's fuzz-smoke failed on this PR at `a08be2e9`, the head the Guardian's
D054 (6732094558) gave PROCEED for. It was job 128973594715 (run 42559832653), step
"Run-fed candidates": `case 596: mibz2 run-fed Leader 1x3 s4 w100003 r65536 detected: 1
blocks rebuilt after a refused check`, with 1,189 cases and 1 failure. The bytes were right:
the rebuild made the block from its symbols. But the harness counts any rebuild as a bug,
and no input was uploaded: the harness saved inputs only for its other failures.

**Reproduced** (`fuzz/r3-case596-replay.txt`, 20 of 20 before the fix). A case's input is its
seed's and its index's alone: every random draw is made while the input is made, before any
decode, however long the run lasts. The loop now replays one case
(`MIBZ2_FUZZ_CASE=596 MIBZ2_FUZZ_REPEAT=20`), and saves the input of any case with a rebuild,
so CI uploads it. Case 596 is 1,352 bytes
(`tests/regressions/run-fed-seed1-case596.bz2`, SHA-256 `3c69e8a3…789f8632`). It is "abab
level 6" with its end mutated:
- block 0: 599,981 rows, two runs, taken by records. It sorted and filled correctly, alone
  and in 1–3 shares.
- block 1: **an empty block**, its symbols the end-of-block alone (n = 0). The records' rule
  takes it (8 × 1 ≤ 600,000), and the emitter makes the empty tiling `[0]`, rightly.
- block 2: broken (`BadBlockMagic`).

§R2's local runs did not meet it. They predate the #275 merge, which changed
`pipeline_corpus`, and so every case's input.

**The cause: a false refusal by the check, not a bad tiling.** The leader's sort
(`sort_one_share`) is the helpers' prefix (`merge_shares`) in one share. That prefix refused
any tiling of fewer than two records, so it refused the empty block's `[0]`, which
`run_stage::sort` takes (n = 0, every count 0). The refusal counted a rebuild. The block went
to `mtf_checkpoints`, then failed `BadOrigPtr`, as the default decoder fails it. So the
error, its block and bit, and the bytes were all right. The other sorts never sort an
empty block: its front stage fails `BadOrigPtr` before any sort. `merge_rows` (the direct
fill's prefix) had the same guard.

**The fix** (`f6708567`): `merge_shares` and `merge_rows` take exactly that tiling.
- The guard is now `recs.is_empty()`. With `[0]`, the existing checks then require the
  first start 0, the closing record at n = 0, every count 0, and slots and rows summing to 0.
- No other tiling's treatment changes. The scatter and the fills write nothing for it.
- No team block is ever empty (`MIN_COOP`), so the registered plan's runs are unchanged.

**The tests:**
- `an_empty_block_by_records` (`tests/run_fed_leader.rs`): the input itself, through every
  sort at every plan, against the default decoder (bytes, error kind, block and bit), with no
  fallback. Before the fix it failed with 5 fallbacks, one for each of the leader's sort's
  five plans.
- `the_empty_block_tiles`: every sort takes `[0]` as `run_stage::sort` does, in 1–5 shares
  and each kernel, and fills its empty table. A count or an end off refuses it. Every mode's
  front stage fails it `BadOrigPtr`, and the leader's path stays `Sorted`, so there is no
  rebuild.
- The tampered-tilings model (`tiled`) now takes the empty tiling as `run_stage::sort` does.

**The mutant:** `R3a-empty-tiling-refused` (the old guard restored) fails both regression
tests (`mutants/logs/R3a-*.log`). The tampered-tilings harness does not catch it: it makes no
empty column.

**The fuzz after the fix** (`fuzz/r3-*`, at `f6708567`):

| run | cases | failures | valid, same bytes | error kinds | reached the pipeline | overlapped-error cases | team blocks by records | small blocks by records | rebuilds |
|---|---|---|---|---|---|---|---|---|---|
| replay of case 596, 20 times | 20 | 0 | — | — | 20 | 20 | 20 | 0 | 0 |
| 30 s, seed 1 (CI's) | 727 | 0 | 93 | 13 | 349 | 9 | 239 | 295 | 0 |
| 240 s, seed 2 | 6,386 | 0 | 862 | 18 | 3,037 | 65 | 2,397 | 2,741 | 0 |

**The suites at `f6708567`** (`mutants/logs/restored.log`), every run under a 6 GB cap:
- `cargo test`: the library 90 passed, 2 ignored; every integration and doc suite passed;
- every run-fed CI step: `run_team` 29 passed and the census ignored; `run_fed_leader` 10;
- `run_team`, `parallel::unwind_tests` and the C29 tests: 37 passed, then 5 more times;
- clippy (default and all features) and fmt: clean.

No unsafe code changed, so Miri was not rerun for R3. §R2's Miri results stand at their pins.

### R3.1 The run pin

- **The implementation pin:** `f67085670ee696340337aa37377afab886dded3a`, the last commit to
  change `src/`, `tests/`, `examples/`, `bench/` or Cargo. It supersedes `5a350c84`.
  Compared with `5a350c84`, the arms' decoding differs only in the two prefixes' guard,
  which only an empty block reaches.
- **The frozen head:** this PR's head carrying this section; every later commit changes only
  `docs/`. `registered_head` is that full head, as the coordinator names it at the re-freeze.
  D054's PROCEED was for `a08be2e9`: this source fix needs its review.
- **Unchanged:** the question, the primary, the reference, the thresholds, VALID, the arms,
  the rounds and the corpus.

## R4. Revision, 2026-11-09: the workstation run made freeze-ready (the Researcher's 6732761085)

CI's registered run at `03e52206` was VALID (JUDGED), an attributed LEAD. The Researcher
reproduced it from the artifact, and asked for a REVISE of §8 before it is frozen as
workstation decision evidence. Each ask, and where it is met:

| ask | met by | checked |
|---|---|---|
| keep the seven arms, their order, `default33` the reference, `dir` primary, `ls`/`hs` secondary | §8's table; the recipe's `--arms`; the admission's `ARMS` (the CI reporter's `REGISTERED_ARMS`, imported) | the identity's arms and every round's rotation are checked |
| exactly 20 paired rounds at a requested 10 threads, 140 timed records, three leaders and six pipelined helpers seen | the recipe (`--threads 10 --rounds 20`); the admission's `ROUNDS`, `THREADS` and its count of 140; CI's plan check at 10 threads | fixtures refuse 19 and 21 rounds, threads 9, a (3, 0, 6) plan not seen, a 139-record bundle |
| a separate, fail-closed admission, not the CI reporter's 15 rounds at [9, 3] nor its UNJUDGED mode | `bench/run_fed_leader_ws.py` (`admit`, `admit-occupancy`): its own predicates and constants; it imports only the CI reporter's typed record checks and interval code, and leaves the CI reporter itself unchanged | `bench/test_run_fed_leader_ws.py`: 14 tests (below), in CI's submit job |
| the thresholds and attribution stated prospectively; §6's tables at df = 19 | §8: the reading's table, the attribution rule, the contrasts reported apart, the post-run contrasts kept descriptive | the fixtures read LEAD (attributed), LEAD with ATTRIBUTION FAILED (ctl far from the incumbent), and STOP; `df` is 19 |
| frozen identities: the source, recipe, admission, and binary and corpus digests, recorded and checked | `bench/run-fed-leader.frozen` (`frozen none` until the coordinator names it); the recipe's measured-inputs check against it; `identity.json`'s sources (the recipe, admission, reporter and registration at the head and at the frozen commit), binaries and corpus before and after | fixtures refuse each mismatched digest, each source changed since the freeze, an unfrozen run, inputs off the frozen commit, a dirty tree; the recipe refuses an unfrozen registered run and a HEAD off the frozen commit (both seen, below) |
| define the ctl and dir occupancy captures and admit them apart | §8's captures; `examples/occupancy.rs --run-fed SUB` | fixtures refuse each capture defect (12 kinds), admit a perturbing capture marked as such; the dry run's captures admitted |
| keep partial and failing evidence | the recipe's `finish` trap packs every attempt, its outcome stated | the first dry run's failure (below) was packed with its partial files |
| the house shape: one new directory an attempt (`mkdir`, not `-p`), the console teed, `bench/bundle.sh`, `just submit bench-run-fed-leader` | `.just/bench.just`'s `bench-run-fed-leader` | the bundles land at `target/run-fed-leader/<UTC>.tar.gz`, where `just submit` finds exactly one |

**The captures needed two small changes in `src/` and an instrument option:**
- **Trace marks in the copied worker and relay.** The copied worker and relay emitted none
  of the shipped pipelined team's trace marks: no MTF, job, run, lead, build, build-wait,
  walk-set-up, walk or assemble marks. Now they emit each at the same point as
  `coop::invert_pipelined_with` and `pipelined_worker`. They also report their buffers at
  the end of the worker (`Team::buffers`, with the records' buffers). All of this is with
  the `trace` feature only, as the shipped team's marks are.
- **What the timed build changes.** The timed build has no `trace`, so the marks compile
  out. Its code differs from the CI-judged commit's in three trivial ways:
  - `invert` takes the block's place;
  - `Relay` holds it;
  - the helpers' built gate is `Gate::arrive`, then `set` or `wait`, which is `pass` split
    in two so that the wait can be marked. The behaviour is unchanged.

  The workstation run measures the frozen commit's code, not `03e52206`'s. These changes
  are not separately benched: they add no work on any decode path.
- **`occupancy --pipe 3 --run-fed SUB`** (`--features trace,run-fed`) builds the run-fed
  teams exactly as ab's `mibz2-pipeK-runfed-SUB` arm does. A local capture of the shipped
  team (`--pipe 3 --levers both`) and of `dir` on the same file read alike, with the team
  workers' buffers recorded in both. Both also say "events reconciled: NO" on decoded
  bytes: that is the instrument's existing approximation for pipelined teams, not a gap of
  the copy.

**The admission's fixture tests** (`bench/test_run_fed_leader_ws.py`, 14, all pass):
- a clean 140-record bundle is VALID (JUDGED), df 19, LEAD attributed;
- LEAD with ATTRIBUTION FAILED; STOP;
- a tiny DRY bundle is VALID only as DRY: refused as evidence, and a registered bundle
  refused as DRY;
- wrong rounds (19, 21, an identity saying 15) and wrong threads (9, `[9, 3]`, a plan not
  seen) are refused;
- a missing arm (records or identity) is refused;
- partial records are refused: 139 records, a truncated line, a broken record, `ab run`
  exiting 101, a missing identity;
- mismatched digests are refused: ab, occupancy, the corpus within a run, the corpus not
  the registered one, each of the four sources at the frozen commit, an output differing
  from the incumbent's;
- an unfrozen run, inputs off the frozen commit, a dirty tree and wrong features are
  refused;
- bad evidence inside a record (negative wall, a fallback, the corpus label) and a broken
  rotation are refused;
- the captures: both admitted; a perturbing one admitted and marked; 12 capture defects
  each refused; an unregistered sub-arm and a missing capture refused.

The CI reporter's 24 tests still pass.

**The local dry runs** (untimed, `just bench-run-fed-leader 1 CORPUS`, on this shared
4-core machine, under a 6 GB address-space cap; records in
`2026-11-09-run-fed-leader/dry-run/`):
1. **Stopped, and packed.** `rustc -vV | head -1` failed the recipe under `pipefail`
   (SIGPIPE). The attempt was still packed with its partial files and an INCOMPLETE
   outcome, which is the partial-evidence path working. Fixed (`rustc -V`, `53901b3f`).
2. **INVALID, rightly, on a corpus of mixed text** (`dry-run/refused-census/`):
   - no team block by records: the corpus's blocks fail the records' rule;
   - one record's thread census caught a pool helper before it named itself. ab saw
     `mibz2-1` twice and one `mibz2-coop-p2` fewer, so the plan read (4, 0, 5) (§R5 repairs
     this: the plan is now read from the decoders' own record). A thread
     carries its parent's name until it names itself. On a 20 ms decode the census can land
     in that window. On the registered corpus's multi-second decodes it is far less likely,
     and CI's 210 records never showed it. But if the workstation run shows it, the run is
     INVALID, by design: §8 admits no exception.
3. **VALID (DRY), on a run-heavy corpus** (`dry-run/valid-dry/`):
   - 14 records, every arm with team blocks by records;
   - both captures admitted, every capture run's output the timing's incumbent's;
   - the same bundle offered as evidence is refused, with every reason listed (DRY, not
     frozen, the sources not at a frozen commit, the corpus not the registered one, 2
     rounds not 20, 14 records not 140).
4. **The recipe's refusals, seen:** an unfrozen registered run ("names no frozen commit
   yet"), a CORPUS without DRY, a dirty checkout, and, through a throwaway local commit
   naming `f6708567` as frozen, "HEAD's measured inputs differ from the frozen commit".

5. **VALID (DRY) again after merging master** (`e16caf3b`, `dry-run/valid-dry-merged/`):
   both captures admitted, neither perturbing. The merge (`e16caf3b`, the head of that dry run) brings master's
   `#278` (`src/read.rs`: a reader whose source panics cancels the decode), `#279` and the
   run's result journals. Every arm decodes through that reader, the incumbent included,
   so the frozen run's reference is master's code, as §6 says.

**The suites at `e16caf3b`:**
- `cargo test`: the library 90 passed, 2 ignored; every integration and doc suite passed,
  `reader_panic` included;
- the run-fed CI steps: `run_team` 29 with the census ignored, `run_fed_leader` 10,
  `mtf_kernels` 4, `ab` 4;
- `test_run_fed_leader.py` and `test_run_fed_leader_ws.py`: OK;
- native: `run_team` with both workers' unwind tests, 37 passed;
- clippy (default, all features, and `occupancy` with `trace` alone) and fmt: clean;
- ruff check and format on the new scripts: clean.

**To freeze** (the coordinator's, not done here):
1. A commit of its own sets `bench/run-fed-leader.frozen` to the PR head carrying this
   section. The `.frozen` file is not among the measured inputs, so that commit changes
   nothing measured.
2. Review, and then the queue entry.

Not done here: no queue entry, no tag, no dispatch.

## R5. Revision, 2026-11-09: the plan read from the decoders' own record (the Guardian's D059)

D059 (6733239555; `state/decisions.md` at `57ea33e3`) accepts §R4's source changes (the
`arrive`/`wait` split, `invert`'s place) with no new B5 HOLD. CI's reached fuzz at
`e1765d09` (job 129000380243) gave 724 cases, 0 failures and 0 rebuilds. D059 reproduced
§R4's census counterexample and recommended repairing it before the freeze. A false
INVALID would cost Louis's one C5 run.

**The defect.** ab's census polls `/proc/<pid>/task/*/comm` from outside every 5 ms, and
the latest name wins. A Linux thread starts with its spawner's `comm` and names itself
only once it runs. So a pool helper polled only in that instant reads under its leader's
name. A pool that lives and dies between two polls is not seen at all.

**Why neither suggested repair would be complete by construction:**
- *Naming at spawn:* the only way to give a thread its name at birth is for the spawner,
  the leader, to rename itself around the clone. That moves the race onto the leader.
- *A readiness barrier:* a barrier holding the spawner until each helper has named itself
  doesn't stop a poll from landing in the birth window, with the helper gone before the
  next poll.

No unprivileged external observer sees a thread's birth.

**The repair: the decoders record themselves** (`src/threads.rs`, `mibz2::threads`):
- each worker, as it starts, and each team helper, when it first runs a task, records its
  own name (`std::thread::Builder`'s, never the inherited one);
- a helper that exits before any task, or never exists, is not recorded, so it is truly
  absent;
- recording is off unless a harness turns it on. Off, it costs each decoder thread one
  relaxed load, once. On, it costs one lock and push a thread, once: at a worker's start,
  and at the end of a helper's first task;
- ab's child turns it on before its decode, and reads it after its timing, as
  `ran_threads` (bench 3.3.0, minor: a new field beside the census's `observed_threads`,
  which is unchanged);
- the workstation admission requires the plan (3, 0, 6) from `ran_threads`. The census
  stays required present and its misreads are reported, never judged. Nothing is inferred
  or waived: a plan the decoders didn't run still reads INVALID. The CI reporter is
  unchanged.

**What changed in `src/`, for the Guardian:**
- `src/threads.rs` (new): the record, `record(on)`, `take()` and the crate-private `ran()`;
- `src/lib.rs`: `pub mod threads`, doc-hidden;
- `src/coop.rs`:
  - `helper` (the pool's thread loop, the shipped teams' and the run-fed teams'): a local
    `ran` flag, and `crate::threads::ran()` once, after its first task, before its `done`
    count;
  - `Pool::named` made `pub(crate)`, for the record's test;
- `src/parallel.rs` (`launch`): each worker calls `threads::ran()` once before `work`.

These touch the timed paths of every arm: the incumbent's, the shipped worker's and the
run-fed teams'. Off, the cost is one relaxed load a thread, once. In ab's children, where
recording is on, it is one lock and push a thread per decode (3 workers and 6 helpers).
It is a change to code the arms run, inside their timing, so it is stated here. (Correction, D061: the lock is one process-wide mutex, which threads starting
together can contend for, and each push allocates its name, and now and then the vector's
growth. "Uncontended" was wrong, and equal counts across arms do not make the cost equal.
No equality is claimed, §R6.)

**The fixtures** (`bench/test_run_fed_leader_ws.py`, now 16, all pass):
- a census misread (D059's (4, 0, 5), three records) with the record (3, 0, 6) is VALID,
  the misreads reported;
- a delayed or inherited name in the record (a helper under `mibz2-1`), read (4, 0, 5):
  INVALID;
- an early exit, a team's helpers gone before their first task, read (3, 0, 4): INVALID;
- an absent helper, read (3, 0, 5): INVALID;
- no `ran_threads` (an older harness): INVALID, never taken from the census;
- the census itself absent: INVALID;
- the earlier plan fixture, a record of one team (1, 0, 2): INVALID.

The Rust test `threads::tests::helpers_record_their_own_names_once_they_ran`: nothing is
recorded while off; a pool dropped before any task records none of its helpers; a pool
that ran twice records each helper once, under its own name.

**The repeat run** (`census/`; the 20 ms corpus of §R4's census counterexample,
regenerated by the same seeded generator, SHA-256 `05079dc8…`): `ab run` of the seven arms
at 10 threads, 50 rounds, 350 records, wall median 24.7 ms. Results:
- the census read (3, 0, 6) 343 times, (3, 0, 4) 6 times (a pool's helpers never polled at
  all) and (5, 0, 4) once;
- the decoders' record read (3, 0, 6) in **all 350**;
- every exit 0 and every error empty.

The old race no longer gives a false plan. (Scope, D061: this is the record's completeness for
ab's fresh children, read once their whole output is read (§R8: by completion order, not
by a join, as first claimed here). The admission's
fixtures are synthetic records: they show what the admission refuses, not that a
physically delayed or early-exiting Rust thread behaves as they model. The Rust test
`threads::tests` covers the pool's idle, ran, panicked and reused cases, §R6.)

**The checks at `a5a9f048`:**
- `cargo test`: the library 91 passed, 2 ignored, every integration and doc suite passed;
- the run-fed CI steps: `run_team` 29 with the census ignored, `run_fed_leader` 10, `ab` 4;
- `run_team`, `parallel::unwind_tests`, `coop` and `threads`: 58 passed;
- both Python suites (16 and 24): OK;
- clippy (default and all features) and fmt: clean.

**A DRY bundle** (`census/dry/`): VALID (DRY), both captures admitted, each record's
`ran_threads` (3, 0, 6). It started at a load of 3.2, above this machine's limit of 3 for
heavy steps; it was untimed, so its figures mean nothing either way.

**The pin.** `bench/run-fed-leader.frozen` stays `frozen none`. The frozen head is this
PR's head carrying this section, once the coordinator names it. Since `src/` changed, the
Guardian's PROCEED for `a08be2e9` does not cover it.

## R6. Revision, 2026-11-09: the census hook's claims corrected (the Guardian's D061)

D061 (6733631922; `state/decisions.md` at `ffff8b86`, L2667) gives **PROCEED toward the
freeze**, with no new B5 HOLD, and asks for three claims to be corrected, plus the hook's
premise stated. These are docs and comments, not new mechanisms. One change to ab's child
is made symmetrically, as D061 allowed.

| D061 | corrected in |
|---|---|
| **cost:** the single mutex can contend; the name's copy and the vector's growth allocate; equal counts across arms don't make equal cost | `src/threads.rs`'s module doc; bench/CHANGELOG 3.3.0; §R5 (the correction beside the old "uncontended" claim); §8 |
| **what CPU includes:** the record's take, sort, join and replace sat outside the wall but inside the reported CPU | changed, symmetrically: ab's child now reads the record after its CPU and RSS readings as well as its wall, in every arm's child alike. Each run's CPU includes the pushes during the decode, and none of the reading. Stated in §8, in the admission's output (`cpu_scope`, and a line with every VALID), and in ab's comment. Nothing is subtracted |
| **completeness, scoped:** a fresh child, recording on before any spawn, and joined before the take; worker starts and helper task *attempts*, caught panics included, not simultaneous occupancy or successful work; the synthetic fixtures don't prove physically delayed or early-exit Rust behaviour | `src/threads.rs`'s module doc ("What it records, and its scope"); ab's comment (turned on before the decode spawns any thread, read once the decode has returned and its decoder's threads are joined: the join claim withdrawn in §R8, replaced by the completion invariant); §8; §R5 |
| **the hook's non-unwinding premise** | `src/threads.rs` ("It never unwinds": `ran` sits between a task and the helper's `done` count; a poisoned lock is taken as it is; an allocation failure aborts, it doesn't unwind); `ran`'s doc; the helper loop's comment in `src/coop.rs` |

**The join before the take,** as first stated here, and **withdrawn in §R8** (D062: false
as stated). ab's child reads the record once `decode` has returned and dropped its reader.
`BzReader`'s drop cancels the decoder. Its last reference then drops the decoder, whose
drop joins every worker (`StreamDecoder::join`), and each worker's pool joins its helpers
on drop. If the feeder thread still held the decoder, that join would come later. On a
file the feeder has read to its end before the reader's last byte. That last sentence is
the false step: a regular file's EOF doesn't prove the feeder has exited, so the join can
come after the take. §R8 replaces this with the completion invariant.

**The optional hardening, done:** `threads::tests::miri_helpers_record_their_own_names_
once_they_ran` now also has a helper whose task panics. The run resumes the panic, the
helper is recorded once (an attempt), and the pool, reused, records nothing more. It runs
under Miri (passed locally, nightly `16d1ffac1`), and `miri.yml` now triggers on
`src/threads.rs`.

**Master merged** (`#280`'s `examples/packet_census.rs` and `bench/packet_census.py`, and
the journals since). `examples/` is a measured input, so the frozen head includes them.

**The checks** at the merged head are §R7's (`84ccdd73`).

## R7. Revision, 2026-11-09: the occupancy admission (the Researcher's freeze review, 6733680620)

The Researcher's freeze review (report at `aa966a90`, `state/reports/workstation-freeze-
6733569628/`) closes the timing registration: seven arms, `default33`, `dir` primary,
20 × 10 threads, 140 records, df = 19. It is a narrow REVISE of the occupancy admission.

**The defect.** Five synthetic captures each exited 0, printed VALID with no perturbation
shown, and wrote `interpretable=true`:
- a missing perturbation;
- an unknown verdict;
- the instrument's own "too few rounds" with a null interval;
- eighteen copies of round 0's off run;
- every capture wall clock NaN.

The cause: the raw runs' length and the summaries' completeness stood in for the 6 × 3
grid, and a capture was interpretable unless some verdict began "perturbs". Missing
evidence became permission.

**The fix** (`bench/run_fed_leader_ws.py`, `occupancy_problems`):
- **The raw grid is validated:** every (round, mode) of 6 × {off, timeline, full} exactly
  once, at 10 threads, with the sub-arm's plan, no error, a finite positive clock, and the
  timing's incumbent's bytes and hash. An observation missing, repeated or out of the grid
  is a reason.
- **Both perturbation comparisons are required:** each present, its interval finite, and
  each recomputed from the grid. The recomputation follows the instrument's own `ratio_ci`:
  the geometric mean of mode over off throughput (bytes over the line's wall, which the
  instrument also reads from its lines), round by round, with its 95% t-interval. Each
  reported interval must equal its recomputation (within 1e-5, since the instrument writes
  six decimals), and its verdict must be the one its interval gives: "no perturbation
  shown" if it contains 1, else "perturbs". Anything else is a reason.
- **Interpretable only when** both comparisons pass and both say "no perturbation shown".
  A missing, unknown or insufficient comparison is INVALID, never interpretable. The
  capture's admission stays apart from the timing's, as before.

**The fixtures** (`bench/test_run_fed_leader_ws.py`, now 19, all pass). The capture builder
now writes the instrument's real schema: the raw grid of 18 runs, each with its own wall,
and both intervals computed from it in the test, independently of the admission and the
instrument.
- **Positive:**
  - a real-schema capture with numeric intervals, interpretable;
  - a genuinely perturbing one (full 30% slower), admitted and marked not interpretable.
- **The Researcher's five, each INVALID and never interpretable:** no perturbation; an
  unknown verdict; "too few rounds" with a null interval; one run eighteen times (also "the
  grid lacks 17 of 18"); every clock NaN.
- **Also INVALID:** an interval misreported against the grid; a verdict misclassified
  against its interval; one run missing ("the grid lacks 1 of 18").

**The wording:** §8's "each twice" now reads "six observations per mode".

**The recording hook's cost, beyond D061's three corrections** (`src/threads.rs`, the helper
loop's comment in `src/coop.rs`, `launch`'s in `src/parallel.rs`, bench/CHANGELOG 3.3.0,
§8, §R5):
- the hooks are inside every run's wall timing;
- a worker records before its first job;
- a helper records after its first task and before it publishes that task's completion
  (`done`), so a contended push can extend that run's completion;
- the "uncontended", "small beside a decode" and equal-cost claims are removed;
- the instrumentation is disclosed as potentially differential, since when threads start,
  and so what they contend with, differs by arm, and its cost is not measured;
- D059's recommendation of observation outside the timed region was replaced by a record
  taken inside it, not met.

**Checked at `84ccdd73`:**
- **A DRY bundle:** VALID (DRY), and both real captures (ctl, dir) admitted by the new
  admission, interpretable. Their reported intervals equal their recomputation from their
  raw grids, so the recomputation reads the instrument's real output as the instrument
  does.
- **Rust:** `cargo test` (the library 91, 2 ignored; every integration and doc suite); the
  run-fed CI steps (`run_team` 29, `run_fed_leader` 10, `ab` 4); `run_team`, both workers'
  unwind tests, `threads` and `coop` together (58).
- **Python:** `test_run_fed_leader_ws.py` (19) and `test_run_fed_leader.py` (24).
- **Lint:** clippy (default, all features) and fmt clean.
- **Miri:** the record's test, `threads::tests::miri_…`, passes (§R6).

`bench/run-fed-leader.frozen` stays `frozen none`.

## R8. Revision, 2026-11-09: the record's completeness, by completion order (the Guardian's D062)

D062 (6733983929; `state/decisions.md` at `72a7dbc6`, L2717) closes D061's cost, CPU-scope
and premise corrections, with no new B5 HOLD. It has one narrow REVISE: §R6's
join-before-take claim is false as stated. The feeder can close the input and be
descheduled while still holding its `Arc`. The decode reaches EOF, `BzReader::drop` sees
the feeder unfinished and detaches it, and the decoder's last drop and its join then come
after ab has turned recording off, read CPU and RSS, and taken the record.

**The fix chosen: D062's (b), the invariant from completion and counts**, not a wait. It
needed no change to code: the invariant already held. Only its statement changes. The
`BzReader::drop` path is unchanged, as D062 asked.
- **Helpers:** a helper records before it counts its task's completion
  (`coop::helper`: `threads::ran()` before `done.fetch_add`). `Pool::run` returns only
  once every helper of the run has counted (its `Wait`, an acquire load of `done`). So a
  team run that ended was preceded by every one of its helpers' records.
- **Workers:** a worker records before its first job (`parallel::launch`), so every block
  it publishes comes after its record.
- **To the reader:** a block's bytes reach the reader only through `publish` and the
  reader's lock. So every byte the child reads comes after the records of the worker that
  published it and of every helper of every team run that built it, and ab's child takes
  the record only after reading the whole output.
- **Every arm:**
  - `mibz2-default33` and `mibz2-pipe3-both` run the shipped pipelined worker, and the five
    run-fed arms run `run_team::worker`;
  - every one is spawned by `parallel::launch`;
  - every team runs on `Pool::run` (`coop::invert_pipelined_with`, `run_team::invert`).

**So the record includes every thread that contributed to the output.** (Corrected in §R9,
D063: as first written here it also said "and none that didn't run", and that the plan is
read only from contributors. Neither holds. A worker records its start whether or not it
publishes a block, so the record includes every contributor but need not contain only
contributors.) Its entries are worker starts and helper task attempts, not per-thread
useful work. A thread that contributed nothing may or may not be in it: a worker that
published no block may be, and a pool that ran no task isn't. The record can fall short of
a plan that was set up but not run, and a required count missing from it is INVALID, as it
should be.

**What it doesn't say:** nothing about teardown. The decoder's threads may still be ending
when the record is read, its join may come later, and a thread recording after the take is
not in it.

**The resource snapshots:** the child's CPU and RSS (`getrusage`) are taken at that moment:
cumulative CPU and peak-so-far RSS. (Corrected in §R9, D063.) Teardown already done by
then can be in them. Only work and new peaks after the snapshot are left out, a detached
feeder's last decoder reference among them. Every arm's child takes them the same way, but
that doesn't make what is left out equal across arms. It is stated in ab's comment.

**The wording**, now the completion invariant, in:
- `src/threads.rs` ("It includes every thread that contributed to a complete output, by its
  completion order, not by any join", as §R9 qualifies it);
- ab's two comments;
- §8;
- the corrections beside §R5's and §R6's join claims;
- bench/CHANGELOG 3.3.0.

**The test** (`threads::tests::the_record_is_whole_once_the_output_is_read_before_any_join`):
a pipelined team of three decodes three full blocks, its output is read to the end, and the
record is taken **while the decoder still lives**, nothing joined. It holds `mibz2-0`,
`mibz2-coop-p1` and `mibz2-coop-p2`. The record's two tests take turns on the process-wide
switch.

**Miri's evidence for the freeze** (`miri.yml`, for the coordinator to read in CI's log; no
pass is claimed here):
- **identity:** an "Identity" step prints the checkout (the pull request's merge commit, its
  parents and its tree), fails on a dirty tree, and prints `rustc -vV` and
  `cargo miri --version`;
- **the default selection:** "The Miri selection" lists `--lib -- miri_` and fails unless
  these are listed: the recorder's panic and reuse fixture
  (`threads::tests::miri_helpers_record_their_own_names_once_they_ran`), the generic pool's
  six lifetime, panic and wake-up tests (`coop::tests::miri_pool_*`), and all three of each
  worker-recovery family (`parallel::unwind_tests::miri_run_panic*`,
  `parallel::run_team::unwind_tests::miri_run_fed_panic*`). The guard was checked locally
  against nightly `16d1ffac1`'s listing. "Miri" then runs that selection;
- **the AVX2 step:** it prints `RUSTFLAGS` (`-C target-feature=+avx2,+bmi1`),
  `MIBZ2_EXPECT_AVX2=1` and the `MIRIFLAGS` forwarding, lists its selection and fails unless
  `miri_register_checkpoints` and `miri_share_kernels` are in it, then runs it.

`bench/run-fed-leader.frozen` stays `frozen none`.

## R9. Revision, 2026-11-09: the completion invariant's two qualifications (the Guardian's D063)

D063 (#145, comment 6734295081) gives PROCEED on §R8's (b). It asks for two wording
qualifications, not a new gate, applied wherever the invariant is stated: `src/threads.rs`,
ab's two comments, §8, the bench 3.3.0 changelog and §R8, each corrected beside its old
wording.

1. **The record includes every contributor; it need not contain only contributors.** A
   worker records its start whether or not it publishes a block. The record is defined as
   worker starts and helper task attempts, not per-thread useful work. A required count
   missing from it stays INVALID. "Never a thread that didn't run", "none that didn't run"
   and "read only from contributors" are withdrawn.
2. **CPU and RSS are cumulative and peak-so-far snapshots.** Teardown already done by the
   snapshot can be in them. Only work and new peaks after the snapshot are left out. The
   same snapshot rule in every arm's child is not claimed to leave out an equal cost across
   arms.

**Provenance, the optional step:** test.yml's rust job gains one step. It runs
`threads::tests::the_record_is_whole_once_the_output_is_read_before_any_join` alone, in a
process of its own (`--exact`), so that no other decoder test's threads, whose names are
the same, are in its record.

No code path, predicate or threshold changes. `bench/run-fed-leader.frozen` stays
`frozen none`.

## Provenance and reproduction

- **Base:** `research/run-fed-stage` at `fd60fffa` (#269 with its safety fix), itself on
  master `a4f97714`.
- **Tests:** `cargo test --features run-fed --lib run_team`; `python3
  bench/test_run_fed_leader.py`; the census, `MIBZ2_RUN_FED_CENSUS=corpora/wikidata.bz2
  cargo test --release --features run-fed --lib census -- --ignored --nocapture`;
  `cargo test --features run-fed --test run_fed_leader`;
  `cargo +nightly miri test --features run-fed --lib -- run_team::tests::miri_
  run_team::unwind_tests::miri_` (plus `RUSTFLAGS="-C target-feature=+avx2,+bmi1"` for the
  AVX2 fills and C29's stores).
- **D049's (§R2):** the fuzz, `cargo build --profile fuzz --example fuzz --features
  run-fed`, then `target/fuzz/examples/fuzz SECONDS SEED - run-fed`; the mutants,
  `2026-11-09-run-fed-leader/mutants/*.patch` (`git apply`, then the named tests).
- **The A/B:** `cargo build --release --features run-fed --example ab`, then
  `ab run R.jsonl --arms <the seven> --threads 9,3 --rounds 15 wikidata.bz2` and
  `python3 bench/run_fed_leader.py report R.jsonl --unjudged --rounds 15 --out rfl-9`
  (UNJUDGED: a judged report needs `--head` and the workflow's `--identity`). On CI:
  Actions → Walk sweep → `tool: run_fed_leader`, `rounds: 15`, `registered_head`.
