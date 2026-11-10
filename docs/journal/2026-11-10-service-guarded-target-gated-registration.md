# The guarded service default under the target-gated route: registration (DRAFT)

10 November 2026. Amended the same day:
- under the Guardian's D077 (§2 replaced, §3.1 added);
- under D078 (tooling via #319);
- under the Researcher's REVISE, 6756808726 (§§2.3, 2.4, 2.8, 3.1);
- under the Researcher's REVIEW of the pilot, 6758898817 (§2.9 added: the final design; §§2.2–2.6,
  6, 7, 9 and 10 aligned to it).
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
unchanged. Master c47fbe50 (#317, #319, #322, #327, #329, #330: tooling, diagnostics and docs) was
merged in the same way on 10 November with §2.9; the freeze's base is whatever master is fetched
then (§6).

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
| toolchain | the dispatch's `toolchain` input; the freeze's `ni_rustc` is its `rustc -V`, checked on every shard; `cargo build --locked` for both builds (the committed `Cargo.lock`s enforced); `rustc -vV`, `cargo -V`, the C compiler, linker, libc and `uname -a` recorded (`build-env.txt`) |
| actions | every external action pinned to a full commit SHA, resolved from its ref on 2026-11-10 (the Guardian's D078 G3): `actions/checkout` d23441a48e516b6c34aea4fa41551a30e30af803 (tag `v6` = `v6.1.0`), `actions/upload-artifact` ea165f8d65b6e75b540449e92b4886f43607fa02 (tag `v4` = `v4.6.2`), `dtolnay/rust-toolchain` 683b588c487a69d01f6c6ddf40f036ff682cc2c8 (branch `master`: the action has no release tags) |
| harness and estimator | `examples/ab.rs` and `bench/service_tg.py` at the frozen candidate; the workflow file at the dispatched head, which must be the head itself |
| head and base | the freeze commit F (the candidate C plus the frozen file alone, checked with git) and the frozen base |

**The tooling and the candidate are separate.** The workflow, the judge, the verdict gate and
the read-back's checker reach master through the tooling-only PR #319 (D078). That PR changes
nothing the decoder builds from, and its branch is merged into this one, with the tooling
identical in both. The workflow is dispatched at this branch's freeze commit F, never at
master. GitHub requires the file on the default branch to dispatch it, but the run executes
F's copy, and the plan job requires `GITHUB_SHA` = F. F carries the candidate's
`examples/svc_target.rs` and bench 3.5.0's `ab`, and a shard checks for both. The decoder code
itself never lands on master through #319.

**What is not pinned, declared as an admitted environment assumption** (D078):
- the hosted runner image (recorded per shard: `ImageOS`, `ImageVersion`);
- its kernel (recorded: `uname -a`);
- its system tools: `gh`, `python3`, coreutils, `lscpu`, the C toolchain and libc used for
  linking (recorded);
- `lbzip2`, installed by apt on the `wikidata` shards for `examples/windows.rs`'s check (its
  version as apt resolves it that day);
- the CPU model the runner draws (recorded, and it must not be the target's).

These are recorded, not controlled. Both arms of a cell share them, on one runner, in one run.
No custom runner infrastructure is used.

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
- **The timeout (at R = 600, superseded by §2.9):** the slowest shard (`records`) was then about
  54 minutes at the slowest pace seen, under a 90-minute timeout. **At the final R = 2880 the
  job timeout is 240 minutes** (§2.9).
- **Accounting (D078 §3), at R = 600, superseded by §2.9:** one dispatch was about **293–412
  shard runner-minutes**. **At R = 2880: about 1098** including setup, 1483 at 1.4× pace (§2.9).
  - **On top of that:** the plan and aggregate jobs' few minutes, and, if a wall cell is flagged,
    one harm-confirmation dispatch of the same size (§2.6).
  - **The timeout** (240 minutes, §2.9) is a per-job ceiling, not the expected cost.
  - **What it uses:** standard GitHub-hosted compute only. It is free for this public
    repository. No larger runners, no paid storage, no new credentials.
  - **Artifacts:** a shard's is a few MB, the binaries excluded. Evidence is preserved durably
    (the evidence branch) before GitHub's retention expires, and is never deleted to meet a
    storage budget.
- **Cost against the alternatives (RULES C3):** repeating 10-round PR runs can't answer
  condition 2 (§2.4), and a single 60-minute job can't hold 600 rounds.
- **The partition is fixed now.** It can't be changed after a timeout, a wide interval or a
  failing cell.

**Within a shard,** one `ab run` measures its cells for R rounds:
- in each round, every thread count in order;
- at each, the two arms back to back, their order as `ab`'s `arm_order` sets it: head first,
  base, base, head, repeated from round 0 (a four-round cycle; §2.9);
- every run its own child process.

A cell's pairs are (head, base) at one thread count in one round, on that shard's runner. **No
cell mixes runners. Nothing is pooled across shards.** Each cell's interval comes from its own
runner alone.

**The scope of the claim** (the Researcher's §4):
- The intersection-union test gives a **conjunction of 42 conditional claims, each on the
  runner model that shard drew**, under this harness.
- It is not a fleet mean, not one hardware response surface, and it covers no unobserved model.
- **The model is recorded per shard, and models are not balanced across corpora.** Balancing
  would still confound corpus with model.
- A model-general claim would need a prospectively crossed cell × model design, or a reviewed
  mechanistic extrapolation. It must never come from re-running shards until a convenient model
  is drawn.

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

### 2.3 The sampling model, and the estimator: a candidate, not yet calibrated

**Amended after the Researcher's REVISE** (#145 6756808726; report
`state/reports/target-gated-ni-6756614616/report.md` on `team-state/researcher` at e1552925).

**The setup:**
- **For each cell:** X_i = log(head time_i / base time_i), i = 0 … R − 1, in round order.
- **The estimate:** exp(mean X).
- **Pairing doesn't make the X_i independent.** It removes common-mode noise, but drift,
  thermal evolution, a runner's noisy episode or order effects can still correlate them (D077).

**The candidate estimator is batch means:**
- R rounds form B consecutive batches of m = R/B rounds;
- se = sd(batch means)/√B;
- U = exp(mean X + t(0.95, B − 1) · se), and the cell passes if U ≤ 1.02.

It is a candidate, not a certified design.

**It is not calibrated, and the draft's assumption was wrong:**
- **Thirty rounds are about 35–67 s of a shard at the pilot pace** (about 61–117 s at 1.75×),
  not 5–8 minutes. Dependence lasting longer than that is not ruled out by anything measured.
- **A correct t quantile is not a calibrated test when the batch means are dependent.** The
  Researcher's falsifier, simulated: a stationary Gaussian AR(1) at the null boundary, R 600,
  B 20, m 30, exact batch-mean covariance, 100,000 draws.

  | ρ | nominal 5% test passed | E[estimated se²] / true Var(mean) |
  |--:|--:|--:|
  | 0.5 | 5.4% | 0.96 |
  | 0.9 | **8.5%** | 0.69 |
  | 0.99 | **28.7%** | 0.12 |

  That is not evidence that CI has ρ = 0.9. It refutes the draft's "size 0.05 whatever σ, R
  and B".

**What is a pilot output, not an assumption:** the batch length m, the number of batches B, R,
and the dependence model the bound rests on. The pilot (§2.8) estimates these, on the
workflow's own runners, under the qualification's own conditions:
- dependence at several lags;
- the batch means' dependence;
- the long-run variance's sensitivity to m;
- order contrasts;
- trends and change points;
- each batch length's physical duration.

The final m, B, R and estimator are then frozen against fresh qualification data, never
against the pilot's.

**Any trend or adequacy veto** is frozen numerically before qualification. A veto:
- names its statistic, threshold and scope;
- with the veto triggered, marks the cell's qualification INVALID: rows kept, no retry, no
  switch to another estimator;
- can only narrow an already valid test's passes. It cannot calibrate an invalid bound.

A trend test that finds nothing, or a low lag-1 correlation, is not proof of independence. No
reported 95% coverage is conditioned on passing a screen unless the combined procedure is
justified.

**Arm order:** `ab`'s schedule is the four-round cycle head, base, base, head. Every two rounds
from an even round balance the count in each position; a whole cycle also cancels an order
coefficient varying linearly within it. **The final batch length is a multiple of four,** each
batch whole cycles from a cycle boundary (§2.9: m 120, 30 cycles). R900/B20 (m 45) and R600/B20
(m 30) are not whole cycles.

### 2.4 Power, size and R: what the pilot must settle

- **Size:** at a cell's null boundary (true ratio 1.02), the test passes that cell with
  probability 0.05 only if the batch means are iid normal. Under dependence it can be
  anticonservative (§2.3). Correct uncertainty is kept apart from apparent power.
- **Power:** at a true ratio of 1.00, which is inside the alternative. Low power at R = 10 is
  **conditionally low power at R = 10** under every history-based variance input. It does not
  make passing impossible.
- **The intersection-union test needs no independence across cells.** The all-pass power does.
  Two summaries are given:
  - the dependence-free lower bound, max(0, 1 − Σ misses);
  - the product, which assumes independent cells and is not a bound.

**The power target is defined now.**
- **The relevant slowdown:** a true ratio of **1.005**, a quarter of the margin, in every cell.
- **The target:** all-cell power of at least **0.80** there, by the dependence-free lower
  bound.
- **The variance it is planned against:** the pilot's **long-run** variance per cell, not the
  marginal ten-round sd.
  - Normalised as LRV_m = m · var(batch means) = n_m · SE_m², over the fixed menu (§2.8).
  - The largest across that menu is a **sensitivity choice**: conservative relative to those
    point estimates, but not a confidence upper bound on the long-run variance.
  - Using its square root as the power model's σ assumes Var(mean) ≈ LRV/R under the chosen
    dependence model. One R600 pilot cannot establish that, nor guarantee 80% power. The final
    review assesses the uncertainty and any residual low-frequency dependence explicitly.

**The history is a scenario, not an envelope.** The 47-run history (bench-results
`links.jsonl` at f99e2f35) gives ten-round marginal sds:
- each is a 9-df estimate, whose own 95% interval is about [0.69 s, 1.83 s];
- its p90 over runs mixes estimation error, runner and candidate heterogeneity, and a
  selection of merged PRs' last runs;
- it says nothing about dependence on the scale of minutes.

**The calculation:** `bench/service_tg_power.py`, integrating over the batch-variance
chi-square. That replaces the noncentral-t call, which returned NaN at R900/median. A
non-finite power is now an error. True ratios are used (log 1.005, not 0.005), and batch
lengths must be even. Its output, reproducing the Researcher's independent integral:

| R / B / m | sd scenario | true ratio | all-42 lower bound | all-42 if independent |
|---|---|--:|--:|--:|
| 600 / 20 / 30 | median | 1.000 | 0.9979 | 0.9979 |
| 600 / 20 / 30 | median | 1.005 | 0.9578 | 0.9578 |
| 600 / 20 / 30 | p90 | 1.000 | 0.8069 | 0.8193 |
| 600 / 20 / 30 | p90 | 1.005 | **0.0015** | 0.3344 |
| 960 / 20 / 48 | median | 1.005 | 0.9956 | 0.9956 |
| 960 / 20 / 48 | p90 | 1.000 | 0.9747 | 0.9749 |
| 960 / 20 / 48 | p90 | 1.005 | **0.7192** | 0.7465 |

- **The verdict on R:** R = 600 is not frozen. R960/B20/m48 is a candidate to evaluate. It
  doesn't meet 0.80 at 1.005 even in the p90 scenario, before any dependence.
- **Budget:** at 960 rounds the slowest shard is about 15 + 38.9 × 1.6 ≈ 77 minutes, which
  leaves 13 minutes of the 90. A different prospective partition may be needed. The pilot's
  setup and runtime tails decide it.
- **If no feasible design meets the target,** the registration says so and stops. It does not
  lower the target after seeing data, or proceed at low power without saying so.
- **After the pilot (§2.9):** R960 and R1200 (0.79124 at the pilot's max LRV) fall short; the
  final design is **R2880/m120/B24**, 0.98548 at the pilot's max LRV and 0.85610 at twice it,
  conditional on iid normal batch means.

### 2.5 Fixed R, timeouts and invalid runs

- **R and B are fixed at the freeze, after the pilot (§2.8), and then never change:** R = 2880,
  B = 24 (§2.9). Once frozen, there is no interim look, no early stop and no extra rounds.
  - If the realised variance runs high, the result is the registered bound: PASS, or NOT
    SHOWN, which fails qualification.
  - Never more rounds, a re-run, a new shard, another estimator or an easier threshold.
- **One qualification dispatch**, labelled `qualification`, with `shards: all`, the frozen head
  and base, the frozen `rounds`, and the frozen toolchain.
- **The only recovery allowed is strictly before measurement, and the judge enforces it
  fail-closed** (`history_problems`, after the Researcher's probes).
  - **The history it reads:** every job of every attempt, with its steps, from the API
    (`filter=all`), and every artifact GitHub lists, downloaded or not.
  - **Counted as measured:** an attempt whose artifact holds rows, OR a start marker, OR whose
    measurement step the API shows begun.
  - **Admitted:** exactly one measured attempt, the shard's last, whose job and measurement step
    succeeded, and whose artifact is listed and downloaded.
  - **Recoverable:** an earlier attempt, but only on positive evidence that it never measured:
    its job failed, its measurement step was skipped, and its artifact (if any) holds no rows
    and no marker. Such a job may be re-run as the same run's next attempt.
  - **Refused:**
    - a cancelled job, even with a complete-looking artifact;
    - an earlier measured attempt that lacks markers or artifacts;
    - a job of another head or run;
    - shard metadata inconsistent with it (workflow SHA, attempt, repository);
    - unknown steps;
    - a listed artifact not downloaded, or a downloaded one GitHub didn't list.
  - **Never recoveries:** a missing download and a terminal failure stay failures.
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
- **The verdict is enforced** (D078 §2): the aggregate job's last step, `bench/tg_ni_gate.py`,
  runs after the evidence is uploaded (`if: always()`).
  - **On a `qualification` dispatch,** it fails the job unless the judge's verdict is a valid
    pass. NOT SHOWN (the judge's exit 1) and INVALID (exit 2) both fail it, and nothing masks
    the judge's exit.
  - **On a `smoke` dispatch,** the job succeeds only if the judge refused the run as smoke, and
    it says "smoke: not evidence".
  - So a green qualification run means condition 2 passed. The registered verdict is still the
    archived `ni.json`.

### 2.6 The harm-confirmation rule, reconciled

The inherited rule stays (D077 keeps it unless a separate forward-only decision changes it). It
is **retained unchanged with the final design** (§2.9: NI power is not adoption power). It
cannot touch condition 2, and the trend veto (§2.9) cannot touch it:
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
- **What a confirmation tests:** the confirmation dispatch draws its own runners, possibly other
  CPU models. So confirmation tests **recurrence across runner draws**, not a literal
  replication of the first runner's conditional estimand. Condition 7 uses it in that sense.
- `service_tg.py confirm` applies the rule. It requires two valid, **distinct** dispatches (two
  run ids) of the same head, records both run ids, and refuses the same verdict given twice.

### 2.7 Data before the freeze, disclosed

None of the following is qualification data, and none can be pooled with the fresh sample:
- **Run 42732367790:** the standing PR Benchmark, attempt 1, on b63f6bee, 10 rounds.
  - It completed successfully, before any freeze, with this registration then marking CI rounds
    OPEN and excluding pre-freeze runs.
  - It is an exploratory pilot. Read under the draft's rule, 18 of 42 cells were NOT SHOWN, on
    width, and none was flagged slower.
  - Its per-run cost is the §2.2 budget's input. Its outcome informed nothing else here.
- **Every later standing Benchmark run on this PR before the freeze** is exploratory too.
- **Run 42736742147 was a push run, not the dispatch path.** It ran on commit 7e325dd0 of the
  throwaway branch `smoke/target-gated-ni` (never merged). That branch's copy of the workflow
  added a `push` trigger with the smoke inputs as defaults, because a dispatch-only workflow
  can't be dispatched before it is on the default branch.
  - It exercised the shards, the read-back, the manifest and the judge, at R = 2, labelled
    `smoke`. It ran before the verdict gate, the action pins and `--locked` existed.
  - It establishes neither that the dispatch path works nor that the qualification run would
    be valid (D078). It is not evidence.
- **A dispatch-path smoke check** becomes possible once #319 is on master. Its inputs are
  prepared and **not dispatched**:
  - `--ref perf/service-guarded-target-gated`;
  - `head` = this branch's commit at the time, which must be the ref's head;
  - `base` = the base its merge of master names (3e1a7ec361ee6c8ea6b540264aa69189d4823d9e at
    this writing);
  - `rounds` = 2, `toolchain` = 1.99.1, `shards` = all, `label` = smoke.

  It must end green with "smoke: not evidence". It is never evidence, and it consumes no
  qualification attempt.
- **#302's outcomes are preserved unchanged** (§0).

- **The design pilot (§2.8)** is pre-freeze data too, disclosed with every earlier run.

Only fresh observations from the qualification dispatch after the freeze count.

### 2.8 The design pilot: one fixed, nonqualifying dispatch

**What it is for:** the design's missing information, which no ten-round history supplies. That
is the long-run variance and dependence of a 20–40 minute serial process on a single-corpus
shard, the order and trend structure, and the real setup and runtime tails. It is not for
finding whether the candidate passes. The Researcher's REVISE asks for it, and C6 permits it as
prospective design work.

**What it is, fixed now:**
- **One dispatch** of `.github/workflows/target-gated-ni.yml`, once the tooling is on master
  (#319) and its admission fixes are in, **and only after a dispatch-path R2 smoke check has
  passed the structural admission** (below).
  - **Inputs:** `label: pilot`, `shards: all` (all nine, the plan job refuses a subset for a
    pilot), `rounds: 600`.
  - **The candidate:** this branch's head at dispatch, with the base it names.
  - **The conditions:** the toolchain and settings registered in §§2.1–2.2: placement, settle
    0.25 s, `ab`'s two-round order rotation, the read-back.
- **Every row is timestamped** as it lands (`bench/tg_ni_stamp.py`, passive, beside both arms
  alike; the same in a qualification dispatch).
  - **The clock:** the runner's CLOCK_REALTIME (Python's `time.time()`), written to the
    millisecond, with a `# start` line when the stamper begins.
  - **What a time means:** when a poller running every 0.2 s saw the row. That is an
    observation time; 0.2 s is not a hard bound on scheduling latency.
  - **The stamper's exit is recorded** (`stamp.exit`) and never masked: a failed stamper fails
    the measurement step, after its record is kept.
  - **The judge and the pilot refuse incomplete or impossible timing**
    (`service_tg.timing_problems`):
    - a missing file, or a stamper exit other than 0;
    - no start time;
    - any row without exactly one finite time;
    - times that go backwards (equal times are allowed, under polling);
    - times outside the measurement's `date -u` markers (to the second) by more than 2 s.

    Missing timing is INVALID, never a printed zero. The same holds for a job start the API
    didn't record: setup is unknown, not 0 s.
- **The analysis menu, frozen before it runs:** `bench/tg_ni_pilot.py`, on #319.
  - **Admission:** the judge's own structural admission (`service_tg.structural`): the
    manifest's schema, no unlisted artifact, the history reconciled, every file rehashed
    against the manifest, every shard's files, runner, read-back, corpus, rows and timing.
    Only the label (`pilot`) and the frozen values (from the command line) differ.
  - **For each of the 42 cells, wall and CPU alike:**
  - the raw order and timestamps;
  - the marginal sd;
  - autocorrelation at lags 1, 2, 5, 10, 30 and 60;
  - for each batch length in the menu below, LRV_m = m · var(batch means) = n_m · SE_m²,
    against the marginal variance, and the batch means' lag-1 correlation;
  - each batch partition's **actual elapsed span** (median, p95, max). It runs from the previous
    batch's last round's end (the first batch's from the stamper's start) to its own last
    round's end, settle and polling included. It is not m times a median round;
  - the head-first minus base-first contrast and its standard error;
  - the OLS trend over the run, with its t;
  - a CUSUM change-point screen.

  Per shard: setup and measurement durations, the round-duration tail, and the runner model.
  One setup observation per shard describes the jobs sampled, not a well-estimated future
  setup tail. **It reports no cell's ratio or bound.** Every series is read centred.
- **The batch-length menu at R = 600, chosen now, before any dispatch:**
  - **m = 10, 20, 30 and 60 over all 600 rounds,** since each divides 600 into an even-length
    partition.
  - **m = 48 over the first 576 rounds only:** twelve complete 48-round batches. m = 48 is
    the R960 candidate's batch length, and 600 = 12 × 48 + 24.
  - **The last 24 rounds** are left out of that one diagnostic, disclosed, and kept in the
    archive and in every other analysis. Its LRV uses its own n = 576.
  - **This is fixed now:** no rounds are added, and the tail's treatment isn't chosen after the
    data. The code refuses a batch length that doesn't divide its rounds, rather than silently
    dropping it, as the earlier draft's filter did.
- **Its rules:**
  - one dispatch, R = 600 whatever it shows;
  - no extra rounds or shards for a favourable or unfavourable estimate;
  - no re-run except the enforced setup-only recovery (§2.5);
  - never pooled into, or substituted for, qualification data;
  - disclosed as pre-freeze data, with run 42732367790 and the smoke run.

  If it is INVALID by the judge's admission, that is recorded. Whether to pilot again is a
  separate, disclosed decision, never automatic.
- **What follows:** from its outputs, and before any qualification data, the registration
  freezes:
  - one R;
  - one batch length and B;
  - the dependence model and its justification;
  - any numerical veto (§2.3);
  - the shard partition and timeout, re-checked against its tails;
  - the power at 1.005 under its long-run variances (§2.4).

  It then goes to the Researcher's review and is frozen against **fresh** qualification
  observations. A single pilot cannot prove mixing or cover all hosted runners. The residual
  assumption is stated with the final design, not claimed away.
- **What it costs:** about 293–412 shard runner-minutes, as a qualification dispatch (§2.2), on
  standard hosted compute.

**The smoke prerequisite, defined now.** Before the pilot, one dispatch-path smoke check, the
same command with `-f rounds=2 -f label=smoke`.
- **What passes:** the aggregate job's gate (`bench/tg_ni_gate.py`) passes it only if two things
  hold:
  - the judge refused it as smoke;
  - it passes the **structural admission** (`service_tg.py structure`, label smoke, R = 2, the
    dispatched toolchain): all nine shards present, their artifacts listed and downloaded, the
    manifest valid, the history reconciled, every file rehashed, and every shard's runner,
    read-back, corpus, rows and timing admitted.
- **The text corpus's digest** is recorded, not checked, before the freeze.
- **The label refusal alone** (the negative qualification-label test) with nothing behind it
  fails.
- **Green on that structure** is the pilot's prerequisite. The pilot's own green means only the
  same structural admission, plus "pilot: not evidence". Its diagnostic admission is
  `tg_ni_pilot.py`'s, offline.

**The pilot's dispatch, prepared and not dispatched:**
```
gh workflow run target-gated-ni.yml --repo lmmx/mibz2 --ref perf/service-guarded-target-gated \
  -f head=<this branch's head at dispatch, which must equal the ref's head> \
  -f base=3e1a7ec361ee6c8ea6b540264aa69189d4823d9e \
  -f rounds=600 -f toolchain=1.99.1 -f shards=all -f label=pilot
```
- **The base:** the master this branch last merged, 3e1a7ec3 at this writing. If master is
  merged again before the pilot, the base is that commit, and the head is the new merge.
- **The analysis afterwards:**
  ```
  python3 bench/tg_ni_pilot.py shards --manifest manifest.json --head <head> --base <base> \
    --rustc "rustc 1.99.1 (58cf8f9c3 2026-10-13)" --corpus-text <the text shard's corpus.sha256> --out pilot
  ```
  It runs on the aggregate job's archived artifacts. The aggregate job itself ends green only
  with the structural admission and "pilot: not evidence".

### 2.9 The final design, from the pilot (DRAFT: for the Researcher's review, not frozen)

**Amended after the Researcher's REVIEW** (#145 6758898817; report
`state/reports/pilot-final-design-6758725018/report.md` at e4f1d8be, with `menu.json`,
`design-results.json` and `analyse_design.py`). Pilot run 42748681243, its menu archived as
#316 comment 6758585874. No candidate mean, ratio or qualification bound was read for this
design: its inputs are the pilot's mean-free menu (variances, correlations, spans, durations).
This section supersedes the open values of §§2.2–2.5; where they differ, it governs.

**The design:**

| item | value |
|---|---|
| R | **2880** rounds a cell, fixed |
| batches | **B = 24** consecutive batches of **m = 120** rounds; no incomplete tail |
| schedule | `ab`'s arm order (`arm_order`, two arms): head first, base, base, head, repeated from round 0. Each batch starts on a cycle boundary and holds **30 complete cycles**, 60 rounds in each order |
| estimator | the mean of all 2880 paired logs, which is the equal-weight mean of the 24 batch means; se = sd(batch means)/√24; U = exp(mean + t(0.95, 23) · se) ≤ 1.02 at every one of the 42 cells |
| order | **no covariate** (below) |
| trend veto | frozen numerically (below) |
| rows | **every row kept**: no trimming, no robust estimator (below) |
| shards | **the same nine** (§2.2); nothing pooled across runners |
| job timeout | **240 minutes**; a timeout consumes the attempt (below) |
| condition 7 | **retained unchanged** (below) |

**The tooling enforces it** (`bench/service_tg.py`, `bench/tg_ni_gate.py`, the workflow; tested in
`bench/test_service_tg.py` and `bench/test_tg_ni_gate.py`):
- `NI_ROUNDS = 2880`, `NI_BATCHES = 24`; asserted at import: 2880 = 24 × 120 and 120 = 30 × 4.
- `cycle_problems` refuses any R and B whose batches are not whole four-round cycles (R600/B20,
  m 30, is refused); `batch_stats` raises on one; the judge reports it INVALID.
- The structural admission (qualification, pilot and smoke alike) refuses a shard whose rows are
  off the schedule: in each round and thread count the two arms back to back, the head first
  exactly in rounds ≡ 0, 3 (mod 4). The tests check `ORDER_CYCLE` against `ab`'s `arm_order`,
  restated, over all 2880 rounds.
- The workflow's plan job refuses a `qualification` dispatch whose `rounds` isn't 2880, before
  any runner starts. Its default is 2880. The pilot's analysis keeps its own R 600
  (`tg_ni_pilot.PILOT_ROUNDS`), so it stays replayable.

**Order: cancelled by the schedule, not modelled.**
- With signs s = (+1, −1, −1, +1) per cycle, Σ s = 0 and Σ i·s = 0. A constant order coefficient
  and one varying linearly within the cycle both cancel exactly in every batch mean.
- **No order covariate.** Subtracting an estimated constant β changes no batch mean (Σ s = 0 over
  each batch): it buys no variance reduction for this estimator. Subtracting β² from the batch
  LRV, already order-balanced, would remove the same term twice.
- **The pilot's evidence:** in pattern/1, the order term β² is 0.24169 (wall) and 0.48471 (CPU)
  of the marginal variance. It predicts the observed raw ACF: lag 2 −0.20085 / −0.46900, lag 30
  −0.21749 / −0.45574, lag 60 +0.23210 / +0.44369, lags 1 and 5 near zero. The conspicuous
  negative correlations are the schedule, not residual dependence.
- **Not cancelled:** nonlinear carryover, common phase effects, rapid changes. They are part of
  this harness's estimand. Balanced order does not establish an order-free production effect,
  nor identify the physical cause of the order contrast.

**The trend veto, frozen:**
1. For each of the 42 (cell × wall, CPU), regress its 24 batch means on batch index 0 … 23 with
   an intercept.
2. The slope's usual t, residual df 22.
3. **|t| > t(1 − 0.01/(2 × 42), 22) = 4.3806308666** invalidates that cell's qualification, and
   so condition 2. A missing or non-finite input invalidates. A zero residual variance gives
   t = 0 for a zero slope, |t| = ∞ otherwise.
4. Every row and the original NI estimate are kept and reported. No detrending, removal,
   estimator switch, replacement shard or retry. CUSUM and the raw order and trend diagnostics
   stay descriptive.

- **Implemented twice:** the judge (`trend_t`; the verdict's `vetoed` and `condition2`; exit 2,
  INVALID) and the verdict gate, which recomputes the veto from the verdict's batch means
  (`tg_ni_gate.veto`, its own regression) and fails the job on any vetoed cell, any missing or
  short batch means, or any disagreement with the judge.
- **Scope: condition 2 only.** The admission (`valid`), the wall flags, peak RSS and the harm
  confirmation (§2.6, `confirm`) are untouched: a vetoed run still flags and confirms harm.
- **Its cost, under its model:** with constant-mean iid Gaussian batch means, the 42-cell family
  probability is at most 1% by Bonferroni, with no independence across cells: a power loss of
  at most 0.01. It is not a 1% bound under drift or dependence, not a goodness-of-fit
  certificate, and not a calibrated test conditional on passing. It certifies no independence.
- **Not tuned to the pilot:** pattern/1 wall's descriptive round-level trend t was 3.03, a change
  of about 1.46% over the run: a relevant warning, not a reason to fit it away.

**The tails are kept.**
- Across all shards, the rounds whose observed span exceeds 10 s are exactly the rounds holding a
  row with `waited_s` > 10 (onebyte 26, records 33, runs 25, text 30, wikidata/1 2, wikidata/2 18;
  none in the other three). `ab` checks the load before each child and sleeps 20 s while busy.
  These spans are settle waiting before the measured child, not decoder latency.
- **`waited_s` is not pure waiting.** `ab` writes it after the child, as the time since before the
  settle check less one settle interval, rounded to whole seconds (`examples/ab.rs`, the record's
  `waited_s:{:.0}`). It includes the child's execution and the parent's overhead: wikidata/1
  reports 1 on 1198 rows. It is never summed as idle waiting, nor subtracted from wall. Its
  semantics are unchanged here; a correction would be a reviewed measurement delta of its own.
- **Every paired log ratio is kept**, noisy child measurements included. No trimming, no robust
  replacement estimator: either would change the estimand and need its own power calculation.
  The estimand stays the geometric-mean head/base time, on the drawn runner, under this
  idle-gated harness. It is not request latency including the harness's waiting.

**A timeout consumes the attempt.** A shard that reaches 240 minutes after its measurement
started is a data-bearing failure (§2.5): the run is INVALID and the attempt is spent. No rescue
by dropping or dividing cells, choosing another model, adding rounds or repartitioning. The
partition stays the nine shards: repartitioning would alter cache, thermal and placement
histories, lose the pilot's variance transfer and redraw the CPU models.

**Power at 1.005 (condition 2 only), conditional:**
- **Planning σ per cell:** √ of the largest LRV_m over **all five** registered menu lengths
  (§2.8). The limiting cell is pattern/2/wall: its largest LRV is at m 20 (ratio 1.233071),
  σ = **0.20139894**. The four columns displayed in the archived table alone would give
  0.19145862: m 20 was computed and archived, not displayed.
- **The model:** λ = log(1.02/1.005) · √(R/LRV), integrated over χ²(B − 1) (`service_tg_power`);
  all-cell lower bound max(0, 1 − Σ misses).

| R / m / B | pilot max-LRV | twice pilot max-LRV | with the veto's ≤ 0.01 |
|---|--:|--:|--:|
| 1200 / 60 / 20 | 0.79124 | 0.52213 | 0.78124 / 0.51213 |
| **2880 / 120 / 24** | **0.98548** | **0.85610** | **0.97548 / 0.84610** |

- The target (≥ 0.80 by the lower bound, §2.4) is met in both scenarios, under iid normal batch
  means. R1200 fails it; R2400/m60/B40 (0.97098 / 0.80471) is nearly marginal at twice the LRV.
- **Twice the LRV is a sensitivity reserve, not a confidence upper bound.** The maximum of five
  correlated estimates (each m60 one on 9 variance df) is no envelope for a new runner. No power
  or coverage is guaranteed.
- **Reproduced for this revision,** by an independent quadrature over `menu.json`: every figure
  above in the table to five decimals, the cutoff 4.380630866588199, and pattern/1's LRV growth (below).
  `bench/test_service_tg.py` pins the limiting cell's two powers in `service_tg_power`.

**The residual batch-memory assumption, and its defence.**

*The assumption, stated:* after aggregation over complete order cycles, the residual process has
a stable mean, a finite long-run variance, and memory short enough that its 24 batch means of
120 rounds are approximately identically distributed, normal and independent. Under exactly iid
normal batch means the t bound is exact; under the short-memory approximation it is only
approximate. No AR(1) structure, reversibility or fleet generalisation is assumed from the raw
ACF. **Independence of the batch means is not certified;** this registration does not claim it.

*For it:*
- **Mechanism, not fit:** the dominant raw dependence, the negative lag-2/30 and positive lag-60
  correlation, is the deterministic schedule (above), and it cancels inside every batch.
- **The long spans are settle waits** before the child (above): idle gating, not decoder state
  carried across rounds.
- **No state crosses rounds in the harness:** every run is its own child process. What can carry
  is the runner's (caches, thermal state, neighbours), on minute scales.
- **m 120 spans about 125–397 s** a batch at the pilot's pace, by shard: double the pilot's
  longest window, and it keeps 24 variance units rather than buying a longer window with ten.

*Against it, stated, not claimed away:*
- **No plateau was observed.** Pattern/1's LRV grows **2.51× (wall) and 2.37× (CPU)** from m 10 to
  m 60, and m 60 has only ten means. **m 120 was not in the pilot menu: its adequacy is an
  extrapolation from m ≤ 60**, not an observed plateau.
- **Residual persistence is plausible.** Pattern/1's batch lag-1 correlation is about 0.21–0.24
  at m 20–30; onebyte/4 at m 60 has **0.417 (wall) and 0.435 (CPU)**, on ten means. Noise and
  persistence both remain possible.
- **The marginal variance includes the periodic order term,** so a low LRV/marginal ratio can
  conceal residual dependence.

*What a violation costs:* Gaussian batch means, B 24, AR(1) correlation ρ across batches, centred
at the NI null boundary; seed 6758725018, 200,000 draws a row, exact covariance and t quantile
(the Researcher's; reproduced here exactly with the same seed and generator):

| batch correlation | nominal 5% NI test passes | with the trend veto |
|--:|--:|--:|
| 0 | 5.0195% | 5.0180% |
| **0.1** | **6.8230%** | 6.8195% |
| **0.2** | **8.8760%** | 8.8660% |
| 0.4 | 14.1200% | 14.0190% |

Monte Carlo SE about 0.05–0.08 points. These are mathematical scenarios, not fitted CI
correlations. More rounds, the veto or a doubled planning variance repair none of it: none is an
inferential correction to an invalid variance model.

*What follows:* the bound is offered as an approximate batch-means bound under the stated
assumption, with its sensitivity, not as certified 95% coverage. The final review judges the
assumption explicitly against the full menu and the mechanism; the chosen m is not its proof. If
it can't be defended, this candidate is not freeze-ready, and the inference is revised
prospectively, never by running and selecting the analysis that passes.

**NI power is not adoption power: condition 7 retained, unchanged.**
- The figures above are condition 2's. **Condition 7's CI criterion is retained explicitly and
  unchanged** (§2.6): a wall cell whose two-sided 95% batch-means interval lies above **1.00** is
  flagged; flagged again on the one confirmation dispatch, another runner draw, it is a confirmed
  harm and fails condition 7, even with U ≤ 1.02.
- **The conflict, stated:** more precision makes a true 0.5% slowdown easier to confirm. At a
  hypothetical true 1.005, wikidata/1/wall's planning σ 0.01212046 gives R2880/B24 a slower-flag
  noncentrality of **22.08**: essentially certain to flag under the same model. A second runner
  with the same response confirms harm, and adoption fails, though both NI tests pass. This is a
  conditional example: confirmation draws other hardware.
- So "0.98548 all-pass at 1.005" is not an adoption probability. **The rule is kept on purpose:**
  any reproducible wall slowdown vetoes adoption. Tolerating up to 2% would need a separate,
  prospective, Guardian-reviewed C6 decision before data. None is sought here, and no threshold
  changes.

**Cost, at the pilot's pace** (each shard's measured R600 time × 4.8; setup budgeted at 15
minutes a shard, though it took 56–98 s in the pilot: one draw per shard doesn't justify cutting
it):

| shard | pilot R600 min | R2880 measurement | + 15 setup | at 1.4× pace, + 15 |
|---|--:|--:|--:|--:|
| `onebyte` | 26.1 | 125.4 | 140.4 | 190.6 |
| `pattern` | 16.5 | 79.3 | 94.3 | 126.0 |
| `random` | 20.3 | 97.4 | 112.4 | 151.3 |
| `records` | 33.1 | 159.0 | **174.0** | **237.5** |
| `runs` | 24.9 | 119.7 | 134.7 | 182.6 |
| `text` | 30.0 | 143.8 | 158.8 | 216.4 |
| `wikidata-1` | 21.2 | 101.9 | 116.9 | 157.7 |
| `wikidata-2` | 18.0 | 86.6 | 101.6 | 136.2 |
| `wikidata-4` | 10.4 | 50.0 | 65.0 | 85.0 |
| all | 200.6 | 963.0 | **1098.0** | 1483.3 |

- **About 1098 runner-minutes per attempt** including the setup allowances, the plan and
  aggregate jobs excluded; 1483 at 1.4× pace. Job ceilings total 2160 shard-minutes a dispatch.
  A harm-confirmation dispatch roughly doubles it.
- **`records` binds:** 174 minutes, 238 at 1.4× pace, within 240. A pace above about 1.415×
  the pilot's on `records` (225/159) times out, and that is INVALID (above). This is a
  transparent sensitivity, not a future-tail quantile.
- Standard GitHub-hosted compute, free for this public repository; no new infrastructure, larger
  runners or spending.

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
  - Arm-order rotation balances position, not plan. It need not remove the bias or the serial
    dependence a switch brings.
- **What the earlier attempts showed:** `service-guarded/20261109T195345Z`'s 60 runs at 10
  threads were all fast (0.126–0.158 s), and `run-fed-leader/20261109T201013Z`'s, 17 minutes
  later on the same machine, mostly slow. So the mode can change between attempts as well as
  within one.
**The causal rules, fixed now** (after the Researcher's §6):
- **No mode exclusion.** A mode the candidate induces is a mediator of its deployment effect,
  part of what the change does. Excluding it, or adjusting it away, answers a different
  question and can erase a real cost or benefit.
- **No outcome-derived mode.** A fast/slow label inferred from the decoded wall time under test
  can't be used to exclude, weight or stratify: conditioning on it can manufacture an effect.
- **Pre-treatment state only.** Only state measured independently *before* a run's treatment
  may serve for blocking or as a covariate, such as the diagnostic's probe before the child
  starts. Its use is frozen before any qualification data:
  - its timing;
  - its classifier;
  - its missingness handling;
  - the arm × state interactions;
  - the deployment weights;
  - the estimand.

  The unadjusted and any model-standardised estimates are reported apart. Switch pairs are
  kept.
- **An external state isn't irrelevant just because the arm didn't cause it.** Whether its
  distribution belongs in the estimand is decided by the target usage regime, stated
  explicitly: fresh processes, as `ab child` measures, or sustained decoding.
- **128 pairs aren't automatically 128 independent units.** If the state persists across runs,
  as the 52%-after-fast persistence shows, the on-target interval needs a valid uncertainty at
  the right unit, frozen before data. The old iid variance is not imported because the mode
  now has a name.
- **The design option to weigh:** randomised arm order within fixed balanced blocks, with the
  seed and schedule archived and carryover addressed, if compatible with the experiment. It
  protects against deterministic time/order aliasing, not against interference.

These, with the diagnostic's result, decide the on-target design. Until then §§3–4's numbers
are a candidate, not frozen.

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
  - CI, R = 2880 rounds a cell in B = 24 batches of 120, fixed (§2.9);
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
- **Wall:** CI's cells by condition 2 and the harm-confirmation rule (§2.6, retained unchanged
  with the final design, §2.9: a confirmed interval above 1.00 fails); the workstation's by
  conditions 3 and 4.
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
- **CI (§2.9):** nine parallel hosted runners at R = 2880, the slowest shard (`records`) about
  174 minutes at the pilot's pace, 238 at 1.4×, under a 240-minute ceiling. About **1098
  runner-minutes** an attempt, setup included (1483 at 1.4× pace); a harm confirmation roughly
  doubles it.

## 10. Open before a freeze

1. **The final design (§2.9):** from the pilot (run 42748681243), R2880/m120/B24, the schedule's
   cycles enforced, the trend veto, all rows kept, the same nine shards, a 240-minute timeout,
   condition 7 retained. **It needs the Researcher's review,** above all of the residual
   batch-memory assumption; qualification-freeze PROCEED is withheld until then.
   - **The tooling delta is on this branch only.** `bench/service_tg.py`, `bench/tg_ni_gate.py`,
     `bench/tg_ni_pilot.py` and `target-gated-ni.yml` now differ from master's #319 copies. The
     run executes the head's own copies (§2.1), but master's workflow file is what GitHub
     dispatches from, and §2.1 records the tooling as identical in both. Whether it reaches
     master by a tooling-only PR before the freeze, as #319 did, is open.
2. **The Researcher's REVISE** (#145 6756808726). Addressed in this revision:
   - the admission bugs, in #319: history reconciled fail-closed, `confirm` bound to distinct
     dispatches, non-finite power rejected;
   - §2.3: batch means not calibrated;
   - §2.4: the power target and R;
   - §2.8: the pilot;
   - §2.2: the scope;
   - §3.1: the modes.

   It needs re-review.
3. **The two-modes diagnostic** (§3.1), before the on-target half is frozen.
4. **The Guardian on coverage** (STEWARDSHIP condition 5), and the bench-policy record D077 asks
   for.
5. **The i9's name, read back** before the freeze: `just bench-svc-target`, at a new freeze of
   its own.
6. **The freeze's values:** `candidate`, `base`, `base_fetched`, `rustc`, `ni_rustc`,
   `corpus_text`. R and B are no longer open: 2880 and 24, in the judge (§2.9).
