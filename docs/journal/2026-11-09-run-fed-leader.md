# 2026-11-09: The run-fed leader for the pipelined teams, built opt-in, and registered (DRAFT, NOT FROZEN)

**Status: a draft registration, NOT FROZEN, NOT RUN** (RULES C1). The coordinator freezes it,
or doesn't. Nothing here changes the default decoder, with or without the new feature:
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
  three? The primary arm is C30 (the sort on the helpers), the metric the paired wall ratio
  against the shipped default, with CPU co-reported and capped.
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
| `Helpers`, `-hs` (C30, **primary**) | `mtf_runs` | count, prefix, scatter, fill shares | the sort parallelised |
| `Direct`, `-dir` (C31) | `mtf_runs` | count rows, prefix, fill F intervals | no F-order array |

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
- **The fill's ownership (C32).** Every vector store ends at or before the rows its thread
  owns (`cell`'s `end`): the share's last row in `fill_share`, the byte's range in
  `fill_direct`. A store past its cell writes only the same thread's next cell, which
  rewrites it. The entries stored, W, are counted (`stores`) against n (`rows`).
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
- its wait for the build after its work (`join_wait_ns`);
- the build's span (a run's start to its walk's set-up done) and the run's;
- the helpers' sort and fill, summed;
- `leader_late`: runs whose leader ended its work after the build;
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
| the share kernels' unsafe writes, scalar and AVX2, and a whole run's hand-offs between helpers and leader (the race detector) | `cargo miri test --features run-fed --lib -- run_team::tests::miri_`, with and without `-C target-feature=+avx2,+bmi1` | see the PR |
| the default suites, unchanged code paths: `cargo test` (default features), and with `run-fed` | `cargo test` | pass |
| clippy (`--all-targets`, all features and default), fmt | | clean |

A mutation check: the vector store's ownership bound moved 8 rows past its end
(`w + 8 <= end + 8`) corrupts the heap at once in the parallel tests (`malloc(): invalid
size`), so the bound is exercised.

**Not covered:** an independent adversarial review of the new unsafe code (RULES B5),
required before any production use and not before a diagnostic run, whose VALID compares
every run's output hash with the incumbent's.

## 4. C31's census and C32's count

From the local smoke (§5, indicative, a 4 MB test file of 133 blocks, 16 of them team
blocks by records), per decode:

| | value | against |
|---|---|---|
| r / n (records a row, team blocks by records) | 0.0756 | Wikidata's 0.062 (#266) |
| C31's predecessor lookups | 2,614 (≈ 163 a block) | the r scatters and 8(r + 1) bytes of F-order array it removes (≈ 45,000 a full block) |
| W / n, the fills' entries stored a row | 1.59 (`ls`, `hs`), 1.59 (`dir`) | 1: a cell averages about 6.6 rows, and its one vector stores 8 |

So C31's lookups are negligible against the work it removes; its price, if any, is the 256
interleaved write streams. And the vector fill overstores by about 59%: every overstore is
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
  of 70 runs. On that file and machine, the helpers are the run's critical path.

**Set before it:** the question, the primary arm (C30, as briefed), the metric, the
incumbent and the arms. **Set after it:** the thresholds' values below. None was
registered before; the coordinator may make any stricter at freezing.

## 6. The registration (RULES C1). DRAFT, NOT FROZEN

- **The question:** does the run-fed leader cut decode wall in the pipelined plan, three
  teams of three with park and small?
- **The incumbent:** `mibz2-default33`, the shipped default constructor with that plan
  forced, through its own entry point. Its baseline is master's code: this branch changes
  no line it runs.
- **The head:** this PR's head at freezing, named by the coordinator; the run is valid only
  if its summary's head line is that commit.
- **The run (CI):** `sweep.yml`, dispatched on this branch at the frozen commit, `tool:
  run_fed_leader`, `rounds: 15`. One dispatch: `ab run` of the seven arms
  (`bench/run_fed_leader.py`'s list), interleaved by round (`ab`'s rotation), at 9
  threads (judged) and at 3 (reported), unpinned, on the bench's Wikidata corpus. A void
  run (VALID fails, or the job fails before its report) may be re-dispatched once at the
  same commit; no other re-run.
- **The estimand:** each arm's per-round paired ratio over the incumbent's, wall and CPU,
  the geometric mean and its 95% t-interval (df 14).
- **VALID** (`bench/run_fed_leader.py`, every predicate, or INVALID with reasons and no
  reading): every arm exactly 15 runs at 9 threads; every child exited 0 with no error;
  every output's hash and length the incumbent's in the same round; every run-fed arm's
  phases present, team runs > 0, no fallback, the record arms' team blocks by records > 0,
  the controls' none; every interval present and finite. The raw paired timings are
  written to `rfl-9.json` and the whole `rfl-runs.jsonl` archived as the run's artifact.

**The reading, on the primary arm (`hs`) at 9 threads:**

| reading | condition |
|---|---|
| **LEAD** | wall ratio ≤ 0.97, its interval's high end < 1, and the CPU ratio's high end ≤ 1.05 |
| **STOP** | the wall ratio's low end > 0.99: no 1% cut |
| **INCONCLUSIVE** | anything else |

**Reported, never judged:** `ls` and `dir` (C31) the same way; `ctl` against the incumbent
and `mibz2-pipe3-both` (the worker's own cost: an interval excluding [0.98, 1.02] means
the instrumented worker differs from the shipped, and every other reading is then read
against `ctl` instead); `reg` against `ctl` (C29: the list kernel's own effect); the gross
leader release a rule block, against both controls; `leader_late` and `join_wait` (which
side ends the runs); W / n and the lookups; the 3-thread cells (one team), against
`mibz2-pipe3-both`.

**The stop criteria:**
- **Positive:** LEAD on CI permits freezing the workstation run below, unchanged but for
  its frozen commit, as the decision's evidence (CI's runners can't host the plan: §7). It
  permits no production change. A production prototype needs its own registration, CI's
  paired bench, and B5's review.
- **Negative:** STOP on CI, with `ctl`'s `leader_late` under a quarter of its runs, closes
  the run-fed leader as a wall lead for this plan: a journal entry, the queue issue's
  Rejected list, the feature kept as research or removed at the coordinator's choice. It
  reopens only if a workstation trace (the occupancy instrument, #39) shows the i9's
  leader ending most runs, which CI can't show.
- **Either way, C29 stands apart:** if `reg` is faster than `ctl` on the leader's MTF,
  that is a separate lead (the shipped `mtf_checkpoints_avx2` replaced by the register-list
  kernel), with its own registration. It is a candidate only where the leader is critical.

## 7. What CI's 4-CPU Xeons can and cannot say about the i9

- **The runner:** 4 vCPUs on 2 physical cores (SMT), an unknown Xeon a run. Three teams of
  three is 9 decoding threads plus the scanner: 2.5 threads a CPU, 5 a core. The i9-10900K
  has 10 physical cores, and the plan was chosen there because 9 threads each get a core.
- **It can say:** whether the arm is correct under heavy contention (VALID); whether its
  total work is less (CPU, which oversubscription doesn't hide); what each phase costs in
  isolation of the others' cores (roughly, since threads share cores); whether the parked
  gates hold up when helpers are descheduled; C29's and C31's per-block costs.
- **It cannot say:** the i9's critical path. With threads time-sharing cores, the wall is
  closer to total work over 4 CPUs than to the longer of the leader's and helpers' sides;
  a leader released on CI frees a CPU for a helper rather than ending a run sooner. So CI's
  wall ratio tests "less total work at equal synchronisation", not "a shorter critical
  path". `leader_late` on CI is a contention figure, not the i9's.
- **Hence** the CI dispatch is a gate: LEAD there justifies the workstation run; STOP there
  is decisive only together with the trace condition above.

## 8. The workstation recipe, drafted (NOT FROZEN: not in the Justfile, not in the queue)

The shape, following CLAUDE.md's "one tarball back" and `just submit`:

```just
# NOT FROZEN (docs/journal/2026-11-09-run-fed-leader.md §8): the run-fed leader on the i9
bench-run-fed-leader corpus="bench/corpora/wikidata.bz2" rounds="20":
    #!/usr/bin/env bash
    set -euo pipefail
    DIR="bench/runs/run-fed-leader-$(date -u +%Y%m%dT%H%M%SZ)"
    mkdir "$DIR"
    exec > >(tee -a "$DIR/console.log") 2>&1
    git rev-parse HEAD > "$DIR/commit.txt"
    git status --porcelain > "$DIR/git-status.txt"
    echo "NOT FROZEN: a draft recipe, not evidence" > "$DIR/NOT-FROZEN.txt"
    sha256sum "{{corpus}}" > "$DIR/corpus.sha256"
    cargo build --release --features run-fed --example ab
    target/release/examples/ab env > "$DIR/env.json"
    target/release/examples/ab run "$DIR/runs.jsonl" \
      --arms mibz2-default33,mibz2-pipe3-both,mibz2-pipe3-runfed-ctl,mibz2-pipe3-runfed-reg,mibz2-pipe3-runfed-ls,mibz2-pipe3-runfed-hs,mibz2-pipe3-runfed-dir \
      --threads 10 --rounds "{{rounds}}" "{{corpus}}"
    python3 bench/run_fed_leader.py report "$DIR/runs.jsonl" --rounds "{{rounds}}" \
      --threads 10 --out "$DIR/report" > "$DIR/report.txt" || true
    bash bench/bundle.sh "$DIR"
```

- At 10 threads (the i9's plan selection, as `bench-pipeline-levers` ran it), every arm is
  three teams of three; the incumbent selects it by itself there, and is forced here only
  so CI and the workstation run the same arm.
- The maintainer's command, once frozen: the branch's checkout at the frozen commit, then
  `just submit bench-run-fed-leader`. Until then, nothing to run.
- To freeze: the occupancy traces of `ctl` and `hs` (one decode each) would be added, so
  the i9's critical path is seen, not inferred.

## 9. Limitations

- **One corpus judged.** The rule's share of blocks, and of small blocks, is the corpus's.
- **The rule is #269's,** decided before MTF from the symbols and the bound, so short
  blocks may take records at a loss on the leader (`lead_small_ns` shows it).
- **The fill overstores** (W / n ≈ 1.6 on the smoke file); not cut (§4).
- **The instrumented worker** carries two clock reads a phase a block; `ctl` against the
  incumbent measures what that and the copy cost.
- **No adversarial review** of the new unsafe code yet (B5).

## Provenance and reproduction

- **Base:** `research/run-fed-stage` at `fd60fffa` (#269 with its safety fix), itself on
  master `a4f97714`.
- **Tests:** `cargo test --features run-fed --lib run_team`;
  `cargo test --features run-fed --test run_fed_leader`;
  `cargo +nightly miri test --features run-fed --lib -- run_team::tests::miri_` (plus
  `RUSTFLAGS="-C target-feature=+avx2,+bmi1"` for the AVX2 fills).
- **The A/B:** `cargo build --release --features run-fed --example ab`, then
  `ab run R.jsonl --arms <the seven> --threads 9,3 --rounds 15 wikidata.bz2` and
  `python3 bench/run_fed_leader.py report R.jsonl --rounds 15 --threads 9 --out rfl-9`.
  On CI: Actions → Walk sweep → `tool: run_fed_leader`, `rounds: 15`.
