# The guarded service default under the target-gated route: registration (DRAFT)

10 November 2026; amended the same day under the Guardian's D077 (§2 replaced, §3.1 added).
**DRAFT: not frozen, not queued, not run.** Nothing in this entry is a measurement of this
candidate. `bench/service-tg.frozen` names no commits, so
`just bench-service-tg` refuses and `bench/service_tg.py` admits nothing. A freeze needs the
Researcher's review (RULES C1) and the Coordinator's registration; the workstation run is
Louis's, on the i9, after that (RULES C5).

**The route.** This registration seeks adoption of #302's guarded service default under
`docs/STEWARDSHIP.md`'s "Target-gated changes", conditions 1 to 7, as enacted by #309 at master
**3e1a7ec361ee6c8ea6b540264aa69189d4823d9e** (decision record
`docs/journal/2026-11-09-target-gated-route-decision.md`; the Guardian's D071 to D074 on
`team-state/guardian`, `state/decisions.md`). It is to be frozen after that commit, cites it,
and is frozen before any of its qualification observations, CI's included. Old data informs
the design below and qualifies nothing.

**The branch.** `perf/service-guarded-target-gated`, from #302's head
**2bab9aa544372a2dab7880157d192a4c99fc0acd**, with master 3e1a7ec3 (fetched
2026-11-10T00:36:11Z) merged in by a merge commit, never a rebase of #302. #302 itself is
unchanged.

**What this does not presume.** Nothing here authorises a run, a merge or a release. Every
condition below is required; missing or invalid evidence fails.

## 0. The earlier attempt, disclosed

- **The attempt:** `service-guarded/20261109T195345Z` (evidence branch commit **cd0f4240**,
  bundle SHA-256 bbe248fc…ad4), under `2026-11-09-service-guarded-registration.md` (#302),
  candidate d136df7e, base a2da4963. It was registered before this route existed and **cannot
  qualify** under it. Its verdicts stand unchanged.
- **What it found** (#145 LOG 6750273151, corrected in 6750401317): VALID; P1 met at 10
  threads, speed 1.068 [1.043, 1.095], CPU 1.173 [1.153, 1.193]; no harm shown at 11 and 20,
  **NOT SHOWN at 12** (upper bound 1.025) **and 16** (1.029); an identity flag at 8 (speed
  1.015 [1.004, 1.027], the head faster).
- **Its CI bench** on #302's head 2bab9aa5 (run 42671718574, 10 rounds, exploratory): `runs` at
  2 and 4 threads +4.0% (+2.4, +5.6) and +3.8% (+1.7, +6.0) slower, `onebyte` at 4 threads
  +2.4% (+0.4, +4.3) slower. Read under condition 2's rule, 20 of its 42 cells fail.
- **How the old data is used, and only so:**
  - its paired variances size the workstation rounds (§4);
  - its CI result exposed an off-target discovery cost, removed in this candidate (§8);
  - CI's history sizes condition 2 (§2.4);
  - the pilot run 42732367790 on this PR, before any freeze, prices the shards (§2.7).

  None of it is pooled with, or substitutes for, this registration's observations.
- **Information value (RULES C3).** A new sample is needed in any case: the route admits no
  old data. This one differs from the earlier attempt in what it can establish:
  - the off-target cost is removed at its source;
  - the predicate is narrowed to what was measured;
  - the fallback contrasts are sized to be decidable.

  The earlier attempt's NOT SHOWN cells are not re-run to turn them into PASS. Their budgets'
  decoders are covered by the transition contrasts, for the mechanism-based reason in §4, not
  because of their results. The no-retry-to-pass rule carries over from that attempt to this
  registration and every later one.

## 1. The predicate (condition 1)

**Frozen text:** `parallel::service_target(threads, k, m)` in `src/parallel.rs`, as at the
frozen candidate. It holds only if every one of these holds:

| fact read | discovered from | required | why it is needed |
|---|---|---|---|
| the requested threads | the caller | exactly 10 | the one budget measured; svc2 is a 10-thread plan (8 decoding threads + 2 service) |
| the kernels | `Kernels::detect` (CPUID) | AVX2 and CLMUL | the kernels the measured runs used |
| OS and architecture | compile time | Linux, x86-64 | the only platform whose topology discovery reads |
| vendor, family, model | `/proc/cpuinfo`, every allowed CPU | GenuineIntel, 6, 165 | the measured microarchitecture (Comet Lake-S) |
| processor name | `/proc/cpuinfo` `model name`, every allowed CPU | `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz` | the measured SKU: narrows the class to it (§5) |
| logical CPUs | `Cpus_allowed_list`, sysfs `online` and `present` | 20 | the measured SMT topology; also requires the whole machine in the affinity |
| physical cores | sysfs `core_id`, `physical_package_id` | 10 | the core count svc2's teams are laid out for |
| L3 | sysfs `cache/index3` | 20 MiB, every CPU's the same | the cache whose contention svc2's plan relieves |
| CPU quota | cgroup v2 `cpu.max`, from the process's cgroup up to the hierarchy's true root | unlimited | as measured: any limit, however large, keeps the existing plan |

- **Unknown facts select the existing plan:** a read error, an unparsable file, a missing
  `model name`, CPUs that disagree, cgroup v1 alone, a hidden ancestor (a namespace) or no v2
  mount each leave the snapshot unread (`Machine::target` None) or the quota `Unknown`.
  Either makes the predicate false, and the decoder is master's.
- **The predicate doesn't identify CI** by runner identity, and wasn't chosen after results to
  exclude losing cells. The narrowing (§5) is chosen for coverage, before any observation of
  this candidate.
- **Discovery runs only at 10 threads.** `StreamDecoder::new` evaluates the predicate only
  when no plan is forced and exactly 10 threads are asked for. `Plan::for_threads` now reads
  only master's two facts, physical cores and L3 (§8). No other budget pays for the snapshot
  or the quota.

**The CI test:** `parallel::plan_tests::off_target_here_builds_masters_plan` (`cargo test`, the
Test workflow's `rust` job). It reads `Machine::here()` for real, prints the facts into the
job's log, and evaluates the predicate at budgets 1 to 24, 32, 48 and 64. On a CI runner
(`GITHUB_ACTIONS=true`) it asserts the predicate false at every one. Wherever the predicate is
false, it asserts that `StreamDecoder::new`, `BzReader::parallel` and `decode_parallel` build
master's decoder: `Plan::for_threads`'s plan, pipelined exactly at `PIPELINED_PLAN`, its
threads and its window. It also asserts the plan equals the policy over master's two facts.
Unit tests cover:
- the target, each fact off by one, the other 10-core Comet Lake-S names (10900KF, 10900,
  10850K, W-1290P), a padded name, no name;
- every quota, any established limit included;
- discovery's failures, a CPU of another name or with no name among them.

**Every measurement runner:** the test above runs on the Test job's machine, which is not the
machine that measures (D077). So every shard of condition 2's workflow reads the predicate
back on its own runner, before measuring: `examples/svc_target.rs`, checked whole by
`bench/svc_target_check.py`, must be NOT TARGET. That means the predicate false at 9, 10 and
11, and the public dispatch building master's decoder at each, every decode correct. The shard
also records the runner's lscpu model, which must not be the target's (§2.2).

**What this establishes:** fallback selection only. Equal plans don't establish zero
discovery, layout or shared-code cost; condition 2 bounds that.

## 2. CI off-target noninferiority (condition 2)

**Amended 10 November 2026, under the Guardian's D077** (#145 comment 6756184275;
`state/decisions.md` at 6c63ce5e, D077). This section **replaces** the draft's admission path
for condition 2. That path was the first completed pull-request Benchmark run on the freeze
commit, mode `pr`, bench 3.5.0, a single artifact. In its place is a dispatch-only, sharded,
within-runner workflow, `.github/workflows/target-gated-ni.yml`, judged by
`bench/service_tg.py ni`. Everything below is fixed before any qualification observation
under the frozen registration.
- **Authority:** amended RULES C6. D077 rules that no RULES amendment is needed and that this
  replacement is a prospective bench-policy decision, recorded here and in its reviewed record.
- **The standing PR bench is unchanged.** It keeps running on every push to this PR; those runs
  are exploratory (§2.7).

### 2.1 The grid and its pinned inputs

The standing CI end-to-end grid at the pinned bench revision (bench 3.5.0 at the frozen
candidate) is 7 corpora × 3 thread counts × 2 metrics = **42 cells**. Every one is required.

| axis | the cells, pinned |
|---|---|
| runner | GitHub-hosted `ubuntu-latest`, admitted only if its observed topology is 4 logical CPUs on 2 physical cores, two a core (`ab env --facts`: `cpus` 4, `cores` 2, each `core_cpus` entry two CPUs), and its lscpu model is not the target's. The label `ubuntu-latest` and `nproc` alone prove nothing. The CPU model is recorded, not chosen. |
| corpus | `onebyte` 22924f5b…4a9e, `pattern` 0f07e2a2…1407, `random` 10d444e6…3f18, `records` ff8b5f6e…1f8c, `runs` af3be4d1…2fb8 (`ab gen`, scale 4, libbzip2 -9), `wikidata` 82f65ace…9915 (bench/wikidata.json). Full digests are in `bench/service_tg.py` `CORPUS_SHA256`. `text` is built by `ab gen` from the candidate's own sources, so its digest is the freeze's `corpus_text`. |
| threads and placement | 1 (pinned to one CPU), 2 (`--spread`: one CPU on each core), 4 (all); settle 0.25 s, as the standing bench |
| metric | wall time; CPU time (the child's `RUSAGE_SELF`) |
| arms | `mibz2` (the head's `ab`) and `mibz2@base/ab` (the base's own `ab`, built from the frozen base in a worktree) |
| toolchain | the dispatch's `toolchain` input; the freeze's `ni_rustc` is its `rustc -V`, checked on every shard |
| harness and estimator | `examples/ab.rs` and `bench/service_tg.py` at the frozen candidate; the workflow file at the dispatched head, which must be the head itself |
| head and base | the freeze commit F (the candidate C plus the frozen file alone, checked with git) and the frozen base |

### 2.2 The shards, and their budget

Nine shards, fixed now. Each is one runner and one job:

| shard | cells | measured s a round (pilot pace) | minutes at R = 600 | at the slowest pace seen (×1.75) |
|---|---|--:|--:|--:|
| `onebyte` | onebyte × 1, 2, 4 × wall, CPU | 1.70 | 17.0 | 29.8 |
| `pattern` | pattern × 1, 2, 4 | 1.74 | 17.4 | 30.5 |
| `random` | random × 1, 2, 4 | 2.17 | 21.7 | 38.0 |
| `records` | records × 1, 2, 4 | 2.22 | 22.2 | **38.9** |
| `runs` | runs × 1, 2, 4 | 1.66 | 16.6 | 29.1 |
| `text` | text × 1, 2, 4 | 2.00 | 20.0 | 35.0 |
| `wikidata-1` | wikidata × 1 | 1.87 | 18.7 | 32.7 |
| `wikidata-2` | wikidata × 2 | 1.26 | 12.6 | 22.1 |
| `wikidata-4` | wikidata × 4 | 1.17 | 11.7 | 20.5 |
| all | 42 cells | 15.8 | 158 runner-minutes | 277 |

- **Where the costs come from:** the pilot's Run step (run 42732367790: 158 s, 10 rounds, 420
  runs). Each run is its decode's wall time plus 0.261 s, which is the 0.25 s settle and the
  child's start. The table is that, per shard.
- **The slowest pace seen:** of the last 30 successful PR Benchmark runs, the same 420-run
  shape took 135 to 277 s. 277/158 = 1.75.
- **Why `wikidata` is split by thread count:** whole, it would cost 43 minutes at the pilot's
  pace and 75 at the slowest. Every other corpus fits whole.
- **Setup is budgeted at 15 minutes a shard** (cold release builds of the head and the base,
  fat LTO, no cache; the corpus; the read-back).
- **The timeout:** the slowest shard (`records`) is then about 54 minutes at the slowest pace
  seen. The job timeout is fixed at **90 minutes**, a 36-minute margin.
- **Cost against the alternatives (RULES C3):** about 160–280 runner-minutes of measurement and
  about 2 hours of setup, on hosted runners at no charge. Repeating 10-round PR runs can't
  answer condition 2 (§2.4), and a single 60-minute job can't hold 600 rounds.
- **The partition is fixed now.** It can't be changed after a timeout, a wide interval or a
  failing cell.

**Within a shard,** one `ab run` measures its cells for R rounds:
- in each round, every thread count in order;
- at each, the two arms back to back, their order rotated as `ab` does (balanced over each 2
  rounds from an even round);
- every run its own child process.

A cell's pairs are (head, base) at one thread count in one round, on that shard's runner. **No
cell mixes runners. Nothing is pooled across shards.** Each cell's interval comes from its own
runner alone.

**What sharding changes, and the resulting estimand:**
- Each corpus's cells now run on their own runner, so they don't share the round with the other
  corpora, as the standing bench's do. Their cache and thermal history is their own corpus's,
  and the nine runners may be different CPU models.
- **The estimand:** for each cell, the geometric-mean head/base time ratio on the one runner
  that shard drew, under this harness.
- **What it isn't:** a fleet average. Nor does a predicate-false read-back on nine runners show
  coverage of every non-target CPU (D077).

**Each shard records** (`shard.json` and beside it):
- the run id, its attempt, the job, the head, base, workflow ref and SHA;
- the runner's name and image;
- `rustc -V`;
- the binaries' digests;
- lscpu;
- `ab env --facts`;
- the predicate's read-back on that runner: `svc_target`'s record and exit, and the checker's
  verdict, which must be **NOT TARGET**, meaning the predicate false and master's decoder built
  by the public dispatch at 9, 10 and 11, every decode correct. A CI test on another job's
  machine is not enough (D077).
- the corpus's digest;
- when the measurement started and ended;
- the runs;
- every file's SHA-256.

**The `aggregate` job** gathers every shard artifact of every attempt into a manifest:
- each file's SHA-256;
- GitHub's artifact id and digest;
- the jobs' conclusions.

It then runs the judge. Every attempted shard is archived, failures included.

### 2.3 The sampling model, and the estimator

- **For each cell:** X_i = log(head time_i / base time_i), i = 0 … R − 1, in round order.
- **The estimate:** exp(mean X).
- **Pairing removes common-mode noise. It doesn't make the X_i independent:** drift, thermal
  evolution, a runner's noisy episode or order effects can correlate neighbouring rounds, and
  then s/√R understates the standard error (D077).
- **So the registered standard error is the batch-means one, chosen now and not after any
  data.**
  - The R = 600 rounds form B = **20** consecutive batches of 30 rounds.
  - se = sd(batch means)/√B.
  - The upper bound is U = exp(mean X + t(0.95, 19) · se), with t(0.95, 19) = 1.7291.
  - The cell passes if U ≤ 1.02.
  - The two-sided 95% interval for flags and RSS uses t(0.975, 19).
- **What this assumes, stated:**
  - **Within the run:** the cell's paired logs have a constant mean.
  - **Dependence:** short-range only, decaying well within a 30-round batch (about 5 to 8
    minutes of wall clock in a shard). The batch means are then approximately independent and,
    at 30 rounds each, approximately normal.
  - **What it tolerates:** autocorrelation within a batch, and heavy tails within a batch.
  - **What it doesn't:** a trend across the whole run, or dependence lasting longer than a
    batch. Under positive dependence of that kind the interval is too narrow.
  - **Arm order:** balanced within every batch.
- **What the interval is not:** a randomisation inference over runners. Its conclusion is about
  the run on each shard's runner.
- **Reported, never judged:**
  - each cell's lag-1 autocorrelation of X in round order;
  - its two halves' geometric means.

  They are for the Researcher's assessment of the model. They don't select an estimator, and no
  threshold on them changes a verdict.
- **The Researcher's review of these assumptions comes before the freeze** (§10). If the review
  finds the model inadequate, the design changes before any qualification data, never after.
- **No validity check is applied to the qualification data beyond §2.5's admission.** Interval
  width is not invalidity.

### 2.4 Power, and size

- **Size:** at a cell's null boundary (true ratio 1.02), the test passes that cell with
  probability 0.05, whatever σ, R and B. That is its level.
- **Power:** at a true ratio of 1.00, which is inside the alternative, not on its boundary. Low
  power at R = 10 means the test is unlikely to show noninferiority there. It doesn't mean
  success is impossible.
- **The intersection-union test needs no independence across cells or shards.** If any cell is
  truly at or beyond the margin, passing all 42 has probability at most 0.05.
- **The all-pass power does depend on dependence across cells,** so two summaries are given:
  - the dependence-free lower bound, max(0, 1 − Σ misses) (D077);
  - the product, which assumes independent cells and is not a bound.

**The inputs:**
- **History:** the `bench-results` branch's `links.jsonl`, which keeps each merged PR's last
  paired Benchmark run. Every run there at bench 3.x, 10 rounds and 4 CPUs is taken: 47 runs,
  2026-10-28 to 2026-11-09, PRs 69 to 311.
- **Per cell:** the within-run sd of the 10 paired logs on one runner, which is the noise one
  shard sees. Its median and its 90th percentile over the 47 runs are the planning sds.
  Examples:
  - `pattern` 2t wall: median 0.104, p90 0.159;
  - `wikidata` 1t wall: 0.014 and 0.030;
  - the whole table is printed by the script.
- **What these sds are not:** bounds. Each is a 9-df estimate, and the history's runs were real
  changes, not nulls (their sds, not their means, are used).

**The model:** the batch means are iid normal with sd σ/√(R/B), so a cell passes with
probability P(T ≥ t(0.95, B − 1)), where T is noncentral t with B − 1 df and noncentrality
(log 1.02 − μ)·√R/σ. It is computed by `python3 bench/service_tg_power.py links.jsonl`,
published with this registration for the Researcher to reproduce.

| R | B | planning sd, every cell | true ratio | weakest cell | Σ misses | all 42, lower bound | all 42, if independent |
|--:|--:|---|--:|--:|--:|--:|--:|
| 300 | 20 | median | 1.000 | 0.938 | 0.062 | 0.938 | 0.938 |
| 480 | 20 | median | 1.000 | 0.991 | 0.009 | 0.991 | 0.991 |
| **600** | **20** | **median** | **1.000** | **0.998** | **0.002** | **0.998** | **0.998** |
| 600 | 20 | median | 1.005 | 0.958 | 0.042 | 0.958 | 0.958 |
| 600 | 20 | median | 1.010 | 0.721 | 0.317 | 0.683 | 0.694 |
| 600 | 20 | p90 | 1.000 | 0.902 | 0.193 | **0.807** | 0.819 |
| 600 | 20 | p90 | 1.005 | 0.710 | 1.002 | 0.000 | 0.333 |
| 900 | 20 | p90 | 1.000 | 0.975 | 0.035 | 0.965 | 0.965 |
| 10 (standing) | – | – | 1.000 | – | – | ≈ 0 | ≈ 0 |

- **R = 600 is a design candidate, not a guaranteed power.**
  - At the median sds and no true effect, the all-cell lower bound is 0.998.
  - With every cell at its 90th-percentile sd at once (pessimistic), it is 0.81.
  - Power falls sharply as the true ratio nears 1.02: at a true +0.5% everywhere, it is between
    0.96 and well below 0.5, depending on the sds.
- **These numbers assume the model:** positive dependence within or across batches lowers them.
- **The draft's earlier figure is superseded.** The draft's "0.85 at 600" came from a different
  calculation: each historical run's own sds, an iid t interval, and the product across cells.
- **The standing 10 rounds** give a conditionally low power, about 0 for the all-cell claim
  under every history-based variance input. That is low power at R = 10, not a proof that
  passing is impossible.

### 2.5 Fixed R, timeouts and invalid runs

- **R = 600 and B = 20 are fixed.** No interim look, no early stop, no extra rounds.
  - If the realised variance runs high, the result is the registered bound: PASS, or NOT
    SHOWN, which fails qualification.
  - Never more rounds, a re-run, a new shard, another estimator or an easier threshold.
- **One qualification dispatch**, labelled `qualification`, with `shards: all`, the frozen head
  and base, `rounds: 600`, and the frozen toolchain.
- **The only recovery allowed is strictly before measurement.**
  - A shard job that failed before its measurement step began holds no observations: no
    `measurement-started.txt` and no `runs.jsonl`. Such a job may be re-run, as the same run's
    next attempt ("re-run failed jobs"). Its earlier attempt is archived.
  - Once a shard's measurement has started, that attempt is the shard's attempt. A later
    attempt of it makes the run INVALID.
- **INVALID fails condition 2, and so the route.** Nothing is re-run to pass. Any of these makes
  the run INVALID:
  - a shard missing;
  - a shard that timed out, was cancelled or failed after its measurement started;
  - an artifact not matching the manifest;
  - a wrong label, head, base, run, rounds, corpus, threads or toolchain;
  - a runner of another topology or the target's model;
  - a read-back that isn't NOT TARGET;
  - a corpus digest that isn't the registered one;
  - an incomplete or duplicated cell, a failed child, a misplaced run, or two output digests in
    a shard.
- **Every attempt is retained** with the manifest. A data-bearing failure is never called an
  infrastructure retry (D077).

### 2.6 The harm-confirmation rule, reconciled

The inherited rule stays (D077 keeps it unless a separate forward-only decision changes it). It
cannot touch condition 2:
- **Condition 2 is decided by the one qualification dispatch alone.**
- **A slower flag:** a wall cell whose two-sided 95% batch-means interval lies above 1 in that
  dispatch is a harm signal.
- **The confirmation dispatch:** one more full dispatch of the same workflow on the same head,
  base and settings, labelled `qualification`, settles every signal at once. It is made only if
  something is flagged.
  - A cell flagged slower again is a **confirmed harm**, and fails condition 7, even where its
    U ≤ 1.02.
  - A cell not flagged again is unresolved, reported with both dispatches.
- **The confirmation dispatch never enters condition 2.** It can't rescue a failed cell,
  replace or pool with the first dispatch, or change its verdict. If it is INVALID, the flags
  stay unresolved, and condition 7 counts them as not confirmed.
- `service_tg.py confirm` applies the rule.

### 2.7 Data before the freeze, disclosed

None of the following is qualification data, and none can be pooled with the fresh sample:
- **Run 42732367790:** the standing PR Benchmark, attempt 1, on b63f6bee, 10 rounds.
  - It completed successfully, before any freeze, with this registration then marking CI rounds
    OPEN and excluding pre-freeze runs.
  - It is an exploratory pilot. Read under the draft's rule, 18 of 42 cells were NOT SHOWN, on
    width, and none was flagged slower.
  - Its per-run cost is the §2.2 budget's input. Its outcome informed nothing else here.
- **Every later standing Benchmark run on this PR before the freeze** is exploratory too.
- **The smoke check of the new workflow** is labelled `smoke`, R = 2, a plumbing check. The
  judge refuses `smoke` as evidence.
- **#302's outcomes are preserved unchanged** (§0).

Only fresh observations from the qualification dispatch after the freeze count.

## 3. On-target superiority (condition 3)

- **The matrix:** N = 1 cell:
  - machine: Louis's i9-10900K workstation, with the predicate TARGET at the run's own
    preflight;
  - workload: the bench's Wikidata sample (bench/wikidata.json, 209,700,196 bytes decoded);
  - enabled budget: 10 threads.

  K = 2N = 2 intervals.
- **The entry point:** `BzReader::parallel` through `ab child` (the `mibz2` arm), in both
  builds.
- **Wall:** speed = base wall / head wall. Its two-sided interval at confidence 1 − 0.05/K =
  97.5% (t at 0.9875, 127 df: 2.2683) must have **lower bound > 1.02**.
- **CPU:** head CPU / base CPU. Its one-sided upper bound at confidence 1 − 0.05/K = 97.5% (t at
  0.975, 127 df: 1.9788) must be **≤ 1.02**.
- **Both must pass.** The two intervals are one Bonferroni family with coverage at least 95%.
  No cell may be added or removed after the data.
- **Expected,** from the earlier attempt's sd (0.052 wall, 0.037 CPU) at 128 rounds: a true
  6.8% gain gives a lower bound near 1.057. The probability of passing is about 1.00 at a true
  1.068, 0.97 at 1.04, and 0.42 at 1.03. CPU passes with near certainty at the earlier
  attempt's 0.85.

### 3.1 The i9's two modes: the on-target freeze waits

The Pathfinder found that at 10 threads the i9 has two persistent modes
(`docs/journal/2026-11-10-two-modes.md`, PR #313):
- **The modes:** a decode takes about 0.15 s or about 0.30 s, nothing between, with every
  thread's CPU time grown alike.
- **Persistence:** a fast run follows a fast run 52% of the time, against 21% by chance.
- **The mechanism is not yet named.** Its candidates are a core-clock mechanism (the governor's
  ramp, stop-clock throttling, the package power limit), sibling packing, and huge pages.

A separate diagnostic recipe to name it is being built. **The on-target half of this
registration (§§3, 4 and the workstation's §7 criteria) is not frozen until that diagnostic's
result is in and reviewed.** If it changes the design, the change is made before any
workstation data.

**How the paired, interleaved design behaves under the modes (model, not observed):**
- **A mode shared within a pair** (both arms of a round in the same mode, as persistence over
  runs suggests):
  - It scales both times alike, and the paired log ratio cancels it.
  - What it adds is the modes' interaction with the plan: if svc2 and the pipelined default
    slow by different factors in the slow mode, the ratio itself is bimodal. The estimate is
    then a mixture weighted by the run's mode frequencies, and its interval is wide.
- **A switch within a pair** (one arm fast, the other slow):
  - It adds about ±log 2 ≈ ±0.69 to that pair's log ratio, against a planning sd of 0.05.
  - A few such pairs dominate the variance. If switches favour one arm's position or plan,
    they also bias the mean.
  - Arm-order rotation balances position, not plan: a mode that the plan itself provokes
    (more threads, more siblings packed) is a real effect of the plan and belongs in the
    estimate, while a mode from the machine's state at the moment does not.
- **What the earlier attempts showed:** `service-guarded/20261109T195345Z`'s 60 runs at 10
  threads were all fast (0.126–0.158 s), and `run-fed-leader/20261109T201013Z`'s, 17 minutes
  later on the same machine, mostly slow. So the mode can change between attempts as well as
  within one.
- **What can't be decided until the mechanism is named:** whether a one-attempt design of 128
  rounds can estimate the plan's effect without mode exclusion, or a mode covariate,
  registered beforehand.

## 4. Fallback no harm (condition 4)

- **The family:** M = 2 wall-time contrasts on the same machine and workload, in the same run:
  **9 threads** and **11 threads**, the predicate's two transitions.
- **The test:** head over base, one-sided upper bound at confidence 1 − 0.05/M = 97.5% (t at
  0.975, 127 df: 1.9788), **≤ 1.02 at each**. NOT SHOWN fails; it is not demonstrated harm.
- **Why these two cover the fallback budgets:** the i9's plan policy, at both commits, makes
  four classes of decoder.

  | budgets | plan, base and candidate | window at `SLOTS_PER_THREAD` | covered by |
  |---|---|---|---|
  | 1–5 | `Workers(t)` (fewer than 6 cores) | 4t | CI's cells: the same `Workers` runtime, at 1, 2 and 4 threads (condition 2) |
  | 6–7 | 3 teams of 2, plain | 12 | 9: the same plain-team runtime, one team fewer |
  | 8–9 | 4 teams of 2, plain | 16 | **9** (8 builds the identical decoder: plan, teams and window) |
  | 10 off target, 11 and more | 3 teams of 3, pipelined | 12 | **11** (12, 16, 20 and more build the identical decoder) |

  At 8 and at 12 to 20, both builds construct exactly the decoder they construct at 9 or 11:
  the same plan, teams and window, from the same code. The requested count changes nothing
  past the plan and window. Contrasts there would replicate the 9 or 11 estimand, not cover a
  different one. With the discovery fix (§8), no budget but 10 reads the predicate's facts.
- **Why not all six of the earlier grid:** M = 6 at practical rounds is unlikely to show every
  bound even under no effect. With the earlier attempt's sds at 64 rounds, the probability is
  0.71, and it falls to 0.25 at a true +0.5% (§4.1). The earlier NOT SHOWN at 12 and 16 is
  disclosed above and is not the reason for this choice: 12 and 16 are the 11 class's decoder.
- **Untested budgets:** 1–7, 8, 12 and more, on the grounds in the table. Budgets past 20 on
  machines of this class are the 11 class.

### 4.1 The rounds, sized from the earlier attempt

The earlier attempt's paired sd of log(head wall / base wall), 20 rounds:

| threads | 8 | 9 | 10 | 11 | 12 | 16 | 20 |
|---|--:|--:|--:|--:|--:|--:|--:|
| head/base | 0.024 | 0.020 | 0.052 | 0.045 | 0.031 | 0.045 | 0.034 |
| twin/base (A/A) | 0.023 | 0.024 | 0.042 | 0.031 | 0.046 | 0.038 | 0.026 |

- **Planning sd:** σ = 0.046, the largest fallback-class sd seen, head/base or A/A. Shown also
  under σ = 0.055 and 0.065, since a 20-round sd is itself uncertain by about ±30%.
- **The model:** a contrast is shown with probability P(T ≥ t(0.975, R − 1)), where T is
  noncentral t with R − 1 df and noncentrality (log 1.02 − μ)·√R/σ, for a true log-ratio μ.

| rounds R | runs (2 arms × 3 budgets × R) | both shown, μ = 0, σ observed | σ = 0.046 | σ = 0.055 | σ = 0.065 | σ observed, μ = +0.5% | μ = +1% |
|--:|--:|--:|--:|--:|--:|--:|--:|
| 96 | 576 | 0.989 | 0.973 | 0.878 | 0.706 | 0.887 | 0.554 |
| 120 | 720 | 0.997 | 0.993 | 0.950 | 0.831 | 0.944 | 0.652 |
| **128** | **768** | **0.998** | **0.996** | **0.963** | **0.861** | **0.956** | **0.681** |
| 144 | 864 | 0.999 | 0.998 | 0.980 | 0.908 | 0.973 | 0.732 |

- **Decided: R = 128.** At 128 rounds, the one-sided bound sits about 0.8% above the estimate
  at σ = 0.046.
- **Arm order:** `ab`'s rotation balances the two arms' order every 4 rounds; 128 is a multiple
  of 4.
- **A true small slowdown is less likely to pass,** as it should be.

## 5. Coverage and adoption reach (condition 5)

**The deployment class**, as narrowed: a process that asks the default entry points
(`StreamDecoder::new`, `BzReader::parallel`, `BzReader::parallel_with`, `decode_parallel`) for
exactly 10 threads, with no plan forced, on x86-64 Linux, where:
- every CPU in its affinity is online and named `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`;
- the affinity holds all 20 logical CPUs of the one package (SMT on);
- the L3 reads 20 MiB;
- AVX2 and CLMUL are present;
- the cgroup v2 hierarchy is readable to its true root with no CPU limit.

**In short: i9-10900K Linux machines with SMT on, the whole CPU available, no quota, at a
10-thread request.** Any input reaches it: the predicate can't see the input.

**The required matrix:** machine (the i9-10900K) × workload (the bench's Wikidata sample) ×
enabled budget (10). Fallback: the same machine and workload at 9 and 11 (§4). The measured
facts selection depends on are the preflight's record: `svc_target`, schema 2, checked whole
by `bench/svc_target_check.py`, recomputing the narrowed predicate.

**The narrowing, and why.** At #302's head the predicate admitted the whole 10-core Comet
Lake-S class (family 6, model 165: the i9-10900K, 10900KF, 10900, 10900F, 10900T, 10850K, 10910,
the Xeon W-1290, W-1290P, W-1290T), and quotas of at least 10 CPUs. One i9-10900K was
measured.
- **The excluded parts differ materially in what svc2's gain plausibly depends on:**
  - base and turbo clocks (the 10900T's 1.9 GHz base at 35 W; the 10850K's lower bins);
  - power limits;
  - for the W-1290 parts, platform and memory (ECC).

  No mechanism-based argument carries a 6.8% wall and 15% CPU result across those.
- **The quota:** a quota between 10 and 20 CPUs can throttle a 13-thread process (8 decoding,
  2 service, scanner, reader, feeder), and none was measured.
- **The predicate is therefore narrowed** to the measured SKU's name and an unlimited quota. A
  narrowed predicate qualifies only to the extent its class is covered. The extrapolation
  below is what covers it.

**Extrapolation to other i9-10900K machines (unmeasured), on mechanism.**
- **What is the same:** the silicon, the stepping (family 6, model 165, stepping 5 on the measured part), the
  cores, the L3 and its sharing, the SMT topology, the SKU's rated clocks, the kernels, and the
  thread layout svc2 creates. The predicate fixes all of these.
- **The mechanism the earlier attempts point to:**
  - svc2 runs 8 decoding threads and 2 light service threads against the default's 9 pipelined
    decoding threads;
  - it spends about 15% less CPU for the same bytes;
  - it relieves contention for the shared 20 MiB L3 and the cores.

  That mechanism follows the parts that are fixed.
- **Material differences not covered, and the residual risk:**
  - **Memory:** DDR4 speed, timings, channels and ranks. The inverse BWT's walk is
    memory-latency-bound, so a slower memory system can shift the balance between the plans.
    The direction is not established.
  - **Board power limits and cooling** (PL1/PL2/Tau, "multi-core enhancement", thermal
    throttling). A power- or thermal-limited part may favour the plan that spends less CPU,
    which is svc2's direction, but this is not measured.
  - **BIOS and microcode, the kernel and its scheduler, the CPU governor and EPP.**
  - **Background load** during a decode: not covered. A busy machine was never measured, and
    ab's settle gate excludes one.
  - **Quota, affinity or hotplug changing during a decode:** the facts are read once, at
    construction.
  - **Other inputs at 10 threads:** other corpora, other block sizes, small inputs, other
    levels. The predicate selects svc2 for them all, and only the Wikidata sample is measured.
    For full-size blocks the per-block mechanism is the same. For very small inputs, svc2's
    fixed cost (its hub and two service threads) against the default's is unmeasured.
  - **The residual risk accepted by an adoption:** on an i9-10900K whose memory, power or
    scheduler differs, or on an input unlike the sample, a 10-thread request may gain less, or
    lose. The loss is bounded to that one budget: the predicate keeps every other budget, and
    every other machine, on master's decoder.
- **Release notes** state the measured scope (one i9-10900K, the Wikidata sample, 10 threads,
  Linux) separately from this extrapolation.
- **A reviewer may still find this extrapolation insufficient** for some part of the class,
  say other memory configurations. The route's answers are to narrow deployment further or to
  obtain the evidence. A second i9-10900K is not available to this team.

## 6. The protocol (condition 6)

- **Candidate and base.** The freeze commit F contains only `bench/service-tg.frozen`, which
  sets:
  - `candidate`: the branch's commit after master is merged at the freeze;
  - `base`: master as fetched;
  - `base_fetched`: the UTC fetch time;
  - `rustc`: the workstation's toolchain;
  - `ni_rustc`: the CI shards' toolchain (`rustc -V`);
  - `corpus_text`: the `text` corpus's digest at the candidate (§2.1).

  The procedure:
  - fetch master and record its SHA and the time;
  - merge it into this branch (never a rebase), if it moved, and name that commit `candidate`;
  - push F.

  The recipe refuses if HEAD's measured inputs differ from the candidate, or if the base isn't
  an ancestor of the candidate. A changed candidate or base needs a new registration.
- **The public entry point:** `BzReader::parallel` (ab's `mibz2` arm, `ab child`), both builds.
  On CI, the same through `ab`'s two arms in each shard.
- **The toolchain:** frozen exactly, `rustc -V`.
  - **Workstation:** the recipe refuses another before anything is built, and both checkouts
    must select the same one.
  - **CI:** the dispatch's `toolchain` input selects it, and every shard's `rustc -V` must be
    the frozen `ni_rustc`.
- **The measured inputs:**
  - `src`, Cargo's files and configuration, `examples` (the harness and the preflight),
    `tests/common`;
  - the judge (`bench/service_tg.py`, with `bench/service_guarded.py` for its shared helpers),
    `bench/svc_target_check.py`, `bench/report.py`;
  - the corpus's builder and manifest, the bundler, the Justfile, the recipe and this file;
  - for CI, `.github/workflows/target-gated-ni.yml` at the head.
- **Pair construction:**
  - **Workstation:** one `ab run`, 2 arms, threads 9, 10, 11, 128 rounds, each run its own
    child process, unpinned. A pair is (head, base) at one budget in one round; within a round
    the budgets run in order and the two arms back to back, their order rotated as `ab` does.
    Settle as `ab`'s default: 2 s at most 1.5 busy cores before each run.
  - **CI:** each shard's one `ab run`, by round, on its own runner (§2.2).
- **The estimand:** the geometric mean of the paired ratios.
- **The intervals:**
  - **Workstation:** a t-interval on the logs, at the levels in §§3–4.
  - **CI:** the batch-means bound of §2.3.
- **Sample size and stopping:**
  - workstation, 128 rounds, fixed;
  - CI, 600 rounds a cell in batches of 30, fixed (§§2.4–2.5);
  - no interim look, no early stop, **one attempt**.
- **Exclusions:** none. Every row is required, and none is deleted, replaced or selected.
  Diagnostics never gate or select.
- **What makes a run invalid, and what follows:**
  - **Workstation:** identity, workload, preflight or structural failure (the judge says which).
    A preflight that is not TARGET refuses before any timed row.
  - **Disposition:**
    - **REFUSED, no timed row exists:** a refusal of the frozen file, the inputs, the
      toolchain, the base or the preflight. It is retained, its cause fixed and stated, and the
      recipe may be run again at the same freeze. Nothing was observed.
    - **Timed rows exist:** INCOMPLETE (the harness stopped), INVALID or VALID. That is this
      registration's one attempt. It is retained as it is, and nothing is re-run under this
      registration.
  - **CI:** INVALID as §2.5 defines it. It is retained, condition 2 fails, and the route fails.
    The one recovery is §2.5's, strictly before measurement.
  - **The rule for later registrations:** a later registration for this change discloses every
    attempt here and its outcome, and justifies its information value. It cannot reset the
    no-retry-to-pass rule.
- **Order:** the workstation run and the CI qualification dispatch may happen in either order
  after the freeze. Neither's result changes the other's rule. The on-target half's freeze
  waits for the two-modes diagnostic (§3.1). It may be frozen separately, later, if the
  diagnostic changes nothing in §§2 and 7's CI parts.

## 7. Other regressions (condition 7), fixed before data

- **Tests, parity and green CI on the head that merges:**
  - `cargo test` (the CI test of §1 included), the harness's and the bench's tests, fuzz-smoke
    and Miri;
  - same bytes on every corpus in the registered CI run;
  - every decode correct at the workstation, through its single digest.
- **CPU:**
  - **CI:** in condition 2's grid. CPU flags are reported; they aren't harm signals for
    confirmation (the inherited rule names wall rows).
  - **Workstation on target:** condition 3.
  - **Workstation at 9 and 11:** head over base, two-sided 95% interval. A **lower bound above
    1.02** is a demonstrated CPU regression and fails. Otherwise it is reported. Condition 4's
    family stays wall-only, as D072 has it.
- **Peak RSS** (each run its own process, `ru_maxrss`), head over base, two-sided 95% interval:
  at every CI cell and every workstation cell, a **lower bound above 1.10** (more than 10%
  more peak memory, shown) fails. Otherwise it is reported.
  - **What was weighed:** the decoder's peak is about 16–80 MB on these workloads, so 10% is a
    few MB on an ordinary 16 GB laptop. The earlier attempt measured svc2 at 0.863 of the
    base's peak at 10 threads.
  - No memory improvement is claimed.
- **Wall:** CI's cells by condition 2 and the harm-confirmation rule (§2.6); the workstation's
  by conditions 3 and 4.
- **Not judged:** CI's stage table and resources assessment (diagnostics), and the
  workstation's medians.

## 8. What changed in the candidate since the earlier attempt

- **Master merged** (3e1a7ec3): #311 (the wait-stats feature, off by default), #312 (the
  build's windows cut into runs), the route itself, and docs.
- **Off-target discovery removed.** At #302's head, `Plan::for_threads` called
  `Machine::here()`. Every decoder, at every budget, therefore also read `/proc/cpuinfo`,
  sysfs per CPU and the cgroup files for the predicate's snapshot and quota. That is plausibly
  the +4% on `runs` and +2.4% on `onebyte` (decodes of about 16 and 23 ms) in #302's CI run,
  though that run doesn't establish the cause. The plan now reads only physical cores and L3,
  as master does, and the snapshot and quota are read at 10 threads alone.
- **The predicate narrowed** (§5): the processor name and an unlimited quota. The preflight
  record carries the name (schema `mibz2.svc_target/2`), and the checker refuses schema 1.
- **The workstation design changed** from the earlier attempt:
  - two arms; no twin and no N1;
  - 9, 10 and 11 threads;
  - 128 rounds;
  - the route's interval levels and rules, and a frozen toolchain.

  **Why no twin:** the route's conditions are paired head-over-base intervals, which already
  carry the run-to-run noise. N1 was a validity condition of the old P1 rule, not of this
  route. It would cost a third of the runs and add an invalidation path with no inferential
  role here. Its A/A sds informed §4.1.
- **Kept from the earlier attempt:**
  - the recipe's refusals;
  - the base's worktree build;
  - the passive, bounded diagnostics, each exit archived, never judged;
  - the child termination checks;
  - the preflight gated on its own successful completion and checked whole;
  - the bundle and `just submit`.

## 9. The cost

- **The timed runs:** 768 at about 2.3 s each, about **29.5 minutes**. Each run is the 2 s
  settle, the child's start and a decode of about 0.15 s. The earlier attempt's 420 runs, with
  its builds, took about 16 minutes end to end: its directory is stamped 19:53:45Z and its
  evidence commit 20:09:58Z.
- **Before them:** two release builds (fat LTO), the corpus and the preflight, about 5 minutes
  built or 10 from cold.
- **In all, about 35–40 minutes on the i9,** within 45. The settle gate's waits, if the machine
  is busy, add 20 s each.
- **CI (§2.2):** nine parallel hosted runners, the slowest shard about 54 minutes at the
  slowest pace seen. About 160–280 runner-minutes of measurement, plus setup.

## 10. Open before a freeze

1. **The Researcher's review** of the amended §2:
   - the shard plan;
   - the sampling model and batch-means estimator;
   - the power calculation (reproduced from `bench/service_tg_power.py`);
   - the timeout and invalid-run rules;
   - the workflow, `.github/workflows/target-gated-ni.yml`.

   Also the Guardian on the coverage argument (STEWARDSHIP condition 5: "reviewed before the
   qualification run"), and the bench-policy record D077 asks for.
2. **The two-modes diagnostic** (§3.1): its result, before the on-target half is frozen.
3. **The i9's name, read back before the freeze:** `just bench-svc-target`, the untimed
   preflight-only entry, at a new freeze of its own.
4. **The freeze's values:** `candidate`, `base`, `base_fetched`, `rustc`, `ni_rustc`,
   `corpus_text`.
