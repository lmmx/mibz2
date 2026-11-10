# The guarded service default under the target-gated route: registration (DRAFT)

10 November 2026. **DRAFT: not frozen, not queued, not run.** Nothing in this entry is a
measurement of this candidate. `bench/service-tg.frozen` names no commits, so
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
  - CI's history sizes condition 2 (§2).

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

**The bench runner:** the Benchmark job runs no `cargo test`. The CI judge
(`service_tg.py ci`) therefore admits the registered run only if its recorded runner CPU
(`result.json`, `machine.cpu`, read from that runner's `/proc/cpuinfo`) is not the target's
name. The predicate is then false on that runner by that fact alone.

**What this establishes:** fallback selection only. Equal plans don't establish zero
discovery, layout or shared-code cost; condition 2 bounds that.

## 2. CI off-target noninferiority (condition 2)

**The standing grid at the pinned bench revision.** The pinned revision is bench **3.5.0**, at
the frozen candidate: `.github/workflows/bench.yml` (pull-request mode), `examples/ab.rs`, and
`bench/report.py`'s `BENCH`.

| axis | the cells |
|---|---|
| runner configuration | `ubuntu-latest`, 4 CPUs on 2 physical cores (`nproc` = 4), whatever CPU model it gets; `--spread`, settle 0.25 s |
| corpus | `onebyte`, `pattern`, `random`, `records`, `runs`, `text` (`ab gen corpora 4`), `wikidata` (bench/wikidata.json's 12 MB of blocks, SHA-256 82f65ace…) |
| threads | 1 (pinned to CPU 1), 2 (CPUs 0 and 2), 4 |
| metric | wall time, CPU time (the child's `RUSAGE_SELF`) |

7 corpora × 3 thread counts × 2 metrics = **42 cells**. Every one is required.

- **Arms:** `mibz2` (the head's `ab`) and `mibz2@base/ab` (the base's own `ab`, built from
  the merge base with master).
- **Corpus validity:** the runs of each corpus give one output digest, with no error (parity).

**The estimator and the pass rule.** For each cell:
- `R` = the n paired head-over-base ratios, paired by round;
- the estimate is their geometric mean;
- `U = exp(mean(log R) + t(0.95, n − 1) · sd(log R) / √n)`;
- the cell passes if `U ≤ 1.02`.

Condition 2 passes only if all 42 cells pass: an intersection-union test at level 5%. It is
not simultaneous coverage. `bench/service_tg.py ci` computes it from the run's artifact
(`runs.jsonl`, `result.json`), with exact t quantiles. Those quantiles are checked against
scipy in the tests.

**The registered CI run.** The run, and how it is admitted:
- **The head:** the freeze commit F. It adds `bench/service-tg.frozen` to the candidate C and
  changes nothing else. The judge checks with git that C is an ancestor of F and that the
  frozen file is the only path between them.
- **The run:** the Benchmark workflow's first completed (not cancelled) run on F, the pull
  request's own run.
- **The base:** the frozen base, which is the merge base the workflow computes. The base must
  not move: master isn't merged again after the freeze, because merging it would change the
  candidate and need a new registration.
- **The run is admitted only as registered:**
  - mode `pr`;
  - its head F, its base the frozen base;
  - bench 3.5.0;
  - `ci_rounds` rounds, threads 1, 2, 4;
  - a 4-CPU runner whose CPU is not the target's;
  - every cell complete;
  - every run's `termination` "success", with an empty `err`;
  - one digest per corpus.
- **Not admitted:** the run is invalid. The registration records it and ends there; there's no
  re-run to pass.
- **Earlier runs:** the bench runs on this pull request before F are exploratory. They are
  disclosed and never qualify. The parameters above were fixed from CI's history (below)
  before this candidate's first CI run completed.

**The repeat and confirmation policy, under the inherited harm-confirmation rule**
(`2026-11-05-p87-production-validation.md`, `2026-11-09-build-window-runs-v2.md`):
- **Condition 2 is decided by the registered run alone.**
- **A slower flag:** a wall cell whose two-sided 95% interval lies above 1 in the registered
  run is a harm signal. One confirmation run on the same head F (a re-run of the job) settles
  every signal at once.
  - A cell flagged slower again is a **confirmed harm**, and fails condition 7, even where its
    U is at most 1.02. The route accepts an untested margin, not a replicated slowdown.
  - A cell not flagged again is unresolved, and reported with both runs.
- **No more runs:** at most one confirmation run, and none if nothing is flagged.
- **The confirmation run can't rescue anything.** It never enters condition 2: it cannot
  rescue a failed cell, replace the registered run, or be pooled with it.
- `service_tg.py confirm` applies this rule.

**BLOCKER: at the standing 10 rounds, condition 2 is not attainable, whatever the candidate
does.** CI's history shows why. `bench-results` `links.jsonl` holds 47 paired PR runs from
28 October to 9 November 2026: bench 3.x, 10 rounds, 4 CPUs, every runner model seen.
- **Measured from that history:**
  - Under this rule, **none of the 47** would have passed all 42 cells.
  - They fail 9 to 26 cells each, median 15. The noisiest cells' within-run sd of the log ratio
    is about 0.10 (`pattern` at 2 threads, wall, median over runs), against a margin of
    log 1.02 = 0.0198.
- **Modelled:** the probability that a candidate with no effect at all passes every cell.
  This assumes each run's per-cell sds as its noise and normal paired logs, averaged over the
  47 runs' noise:

  | rounds | 10 | 50 | 100 | 160 | 250 | 400 | 600 |
  |---|--:|--:|--:|--:|--:|--:|--:|
  | P(all 42 pass \| null) | 0.000 | 0.03 | 0.17 | 0.34 | 0.53 | 0.72 | 0.85 |
  | one runner's Run step, at 16–28 s a round | 3–5 min | 13–23 | 27–47 | 43–75 | 1.1–1.9 h | 1.8–3.1 h | 2.7–4.7 h |

So a qualification run at 10 rounds would very probably fail, and would spend this
registration's one attempt on an uninformative result. Reaching about 80% power takes
around 500 rounds, beyond one job's 60-minute limit.
- **The rounds are not the grid,** and condition 6 makes them the registration's to fix.
- **But the workflow has no way to run more,** and adding one, or sharding the run across
  runners with a pooled estimator, is a change to the standing bench. The route makes any such
  change "a separate prospective bench-policy decision".
- **So this registration is not freezable** until that decision fixes:
  - `ci_rounds`;
  - the mechanism that runs them;
  - if sharded, how the interval accounts for runners: per-shard means or pooled pairs.

The judge already takes `ci_rounds` from the frozen file and checks the run against it. A
sharded design would need the judge extended to several artifacts, under that decision.

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
  - `rustc`: the toolchain;
  - `ci_rounds`: the CI rounds.

  The procedure:
  - fetch master and record its SHA and the time;
  - merge it into this branch (never a rebase), if it moved, and name that commit `candidate`;
  - push F.

  The recipe refuses if HEAD's measured inputs differ from the candidate, or if the base isn't
  an ancestor of the candidate. A changed candidate or base needs a new registration.
- **The public entry point:** `BzReader::parallel` (ab's `mibz2` arm, `ab child`), both builds.
  On CI, the same through the standing bench's arms.
- **The toolchain:** the workstation's `rustc -V`, frozen exactly. The recipe refuses another
  before anything is built, and both checkouts must select the same one. CI uses the
  workflow's own stable toolchain, as the standing bench does; it is recorded in `result.json`.
- **The measured inputs:**
  - `src`, Cargo's files and configuration, `examples` (the harness and the preflight),
    `tests/common`;
  - the judge (`bench/service_tg.py`, with `bench/service_guarded.py` for its shared helpers),
    `bench/svc_target_check.py`, `bench/report.py`;
  - the corpus's builder and manifest, the bundler, the Justfile, the recipe and this file.
- **Pair construction:**
  - **Workstation:** one `ab run`, 2 arms, threads 9, 10, 11, 128 rounds, each run its own
    child process, unpinned. A pair is (head, base) at one budget in one round; within a round
    the budgets run in order and the two arms back to back, their order rotated as `ab` does.
    Settle as `ab`'s default: 2 s at most 1.5 busy cores before each run.
  - **CI:** the standing bench's pairs, by round.
- **The estimand:** the geometric mean of the paired ratios. The interval is a t-interval on the
  logs, at the levels in §§2–4.
- **Sample size and stopping:**
  - workstation, 128 rounds, fixed;
  - CI, `ci_rounds` (§2, open), fixed;
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
  - **CI:** the run is not admitted (§2). It is retained, and the registration ends there.
  - **The rule for later registrations:** a later registration for this change discloses every
    attempt here and its outcome, and justifies its information value. It cannot reset the
    no-retry-to-pass rule.
- **Order:** the workstation run and the registered CI run may happen in either order after the
  freeze. Neither's result changes the other's rule.

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
- **Wall:** CI's cells by condition 2 and the harm-confirmation rule (§2); the workstation's
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

## 10. Open before a freeze

1. **Condition 2's sample size and mechanism (BLOCKER, §2):** a prospective bench-policy
   decision on how the registered CI run gets enough rounds. This registration can't make it
   within the route.
2. **The Researcher's review** of this registration, and the Guardian on the coverage
   argument and the CI decision (STEWARDSHIP condition 5: "reviewed before the qualification
   run").
3. **The i9's name, read back before the freeze:** `just bench-svc-target`, the untimed
   preflight-only entry, at a new freeze of its own. It confirms that the narrowed predicate is
   TARGET on the workstation as Louis runs it, before a timed slot is spent.
4. **The freeze's values:** `candidate`, `base`, `base_fetched`, `rustc`, `ci_rounds`.
