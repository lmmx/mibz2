# The guarded service default against the current default: registration (DRAFT)

9 November 2026. **DRAFT: not frozen, not queued, not run.** Nothing in this entry is a
measurement. `bench/service-guarded.frozen` names no commits, so `just bench-service-guarded`
refuses. A freeze needs the Researcher's assessment and the Coordinator's registration
(RULES C1, C5); the run is Louis's, on the i9, after that. **RULES C6 is amended (#306):**
bench design, merge acceptance and adoption policy are the team's, as a class, under review;
this registration's design choices below are each decided by the Coordinator under amended C6 (#306), subject to the Researcher's review. The Guardian's D069 (#145, comment
6741390621) resolved D067's technical objections at 6642faf4 and said to draft this; timing,
queueing and production admission stay gated by it.

**Revision (the Researcher's REVISE, REVIEW 6742157313 on #145; report and probes at
`state/reports/service-guarded-registration-6741912733/` on b365250e):** the judge now admits
an attempt's directory, not a bare runs file, and binds its workload, identity and preflight
(R1, R2); the row schema is strict (R2); the report keeps the earlier qualifications (R3); the
base, RSS, N1 and replacement answers below follow the review's six answers (the grid follows
a later Coordinator decision: the pipelined default's, exactly); the recipe
is exercised end to end past its refusals (answer 6); and the preflight-only entry
(`2026-11-09-svc-target-registration.md`, answer 5) comes first.

**Second revision, 9 November 2026 (the Researcher's REVISE, REVIEW 6746023579 on #145;
report at `state/reports/service-guarded-rereview-6745811380/report.md` on ceb53e4f, reviewed at
910fd5de):** see *Revision 2* below. In short, prospectively, before any freeze or run: every
row must carry the child's termination as success; the diagnostics are passive, bounded and
archived with their exits, never judged; the raw discovery facts are archived beside them; and
the freeze records the fetched master's SHA and fetch time. No grid, threshold, arm or estimand
changes.

**What this does not presume.** D067 §5's adoption question (the change makes svc2 an
automatic default on a class of machines CI can't exercise, and how CI's acceptance rule
applies to that) is, under amended C6 (#306), a team decision, **made separately, after the
evidence, by a Guardian-reviewed decision-record entry**. This registration measures the
candidate on the one machine it can; it does not presume that decision, decides nothing about
adoption, and no outcome below is a merge or release rule.

It follows the switched-default precedent (`2026-11-07-pipelined-default-registration.md`: the
exact switched default against the exact current default through the same public entry point,
two builds) and the service-core registrations (the service-2x3 registration,
`docs/journal/2026-11-08-service-2x3-registration.md` at **3e201246cf1266d4b2a984cb19eb9fdba5b9835d**,
not on this branch: the paired estimand, the outcome map and its readings, co-primary speed
and CPU, the noise control N1, its report's evidence boundary), unchanged where they apply.

## The candidate and the base

- **The candidate:** `perf/service-guarded` (#302) at its frozen commit. `StreamDecoder::new`,
  and so `BzReader::parallel`, `parallel_with` and `decode_parallel`, builds svc2 (two teams
  of three that only build and walk, two service workers; lookahead 2; window scaled to two
  teams, 8 blocks at `ab`'s 40) when, and only when, no plan is forced and
  `parallel::service_target` holds (`2026-11-09-service-guarded-scope.md` §2). Everything else
  is master's dispatch, unchanged.
- **The base: current master at the freeze, by its immutable SHA, merged into the candidate
  before the freeze** (answer 1). This asks whether the proposed default improves the default
  that would otherwise ship; a release comparison would answer a different, user-upgrade
  question and include unrelated changes. The freeze procedure: fetch master, merge it into
  `perf/service-guarded` (never a rebase), name that master commit as `base` and the merged
  candidate as `candidate`, with `base_fetched`, the UTC time that master was fetched
  (Revision 2, R4). The recipe checks the base is an ancestor of the candidate and refuses a
  freeze without the fetch time; that the base was *current* master when fetched is the
  freeze's own check, recorded in the frozen file and the freezing commit's message. The
  bundle records both commits and the fetch time (`commits.txt`), both executables' digests
  and the toolchain (`inputs.txt`).
- **The runtime is not #258's.** #258's P1, 1.045 [1.017, 1.073], and #271's reported D-a2w8 are
  history on a different runtime: neither transfers, and this run does not pool with them.

## The run

`just submit bench-service-guarded` on the i9, at the frozen commit, in CLAUDE.md's
workstation-recipe shape (one new directory an attempt, its console log in it, one tarball
whatever the outcome). The recipe is `.just/service-guarded.just`, imported by the Justfile.
Its preflight is sent TERM at 120 s and KILL 10 s later.

- **Refusals before anything runs:** the frozen file unset; the base not an ancestor of the
  candidate; uncommitted changes, or untracked files among the measured inputs; HEAD's
  measured inputs differing from the candidate's (`src`, Cargo's files and configuration,
  `examples` (the harness and the preflight), `tests/common`, the judge and the preflight's
  checker, the corpus's builder and manifest, the bundler, the Justfile, the recipe and this
  file; `NOT-FROZEN.txt` then marks the attempt); a `CARGO_*`/`RUST*`/`AB_CENSUS_MS`/
  `AB_SAMPLE_MS` override; differing toolchains; a Cargo configuration only the base build would
  read; the base worktree not clean at the base; a CORPUS without DRY.
- **The builds:** each in its own checkout, Cargo's messages naming each executable, copied
  into `bin/`: `ab-base`, `ab-base-twin` (a byte-identical copy, checked by SHA-256), `ab-head`,
  `windows` (the corpus builder) and `svc_target` (the preflight).
- **The preflight, before any measurement:** `bin/svc_target`'s record (schema
  `mibz2.svc_target/1`) and exit status, checked whole by `bench/svc_target_check.py` (the
  predicate recomputed from the record's own facts; the dispatch at 9, 10 and 11 against it).
  **Anything but TARGET refuses the attempt** (REFUSED, exit 4), its record and check archived.
  The preflight-only entry (`just bench-svc-target`) runs first, separately; it does not
  replace this one.
- **The workload manifest:** `corpus.sha256` (the corpus's digest), `arms.txt` (the exact
  three arm strings run), `inputs.txt` (every executable's and the corpus's digest, the
  toolchain, the Cargo configuration read).
- **The arms:** `mibz2` (the head's `BzReader::parallel`), `mibz2@<attempt>/bin/ab-base` (the
  base's, the same entry point) and `mibz2@<attempt>/bin/ab-base-twin` (N1). Unpinned. Each run
  is its own child process (`ab child`), so its wall, its CPU (`RUSAGE_SELF`: every thread of
  that process, the scanner, the feeders, the caller and the pool helpers included) and its
  **peak RSS (its own `ru_maxrss`, which never resets: one process per run is what makes it
  the run's)** are recorded for every run.
- **The grid: the pipelined default's, exactly** (decided by the Coordinator under amended C6 (#306), subject to the Researcher's review): 8, 9, 10, 11, 12, 16 and 20 threads,
  20 rounds, paired by round, arms' order rotated as `ab` does, on the bench's Wikidata sample,
  as registered in `2026-11-07-pipelined-default-registration.md` and authorised there (Louis's
  decision to ship after one frozen confirmation run, "Confirm first, then ship",
  `decisions.md`, 2026-11-07; that registration's grid is the one this run copies). P1 stays
  the 10-thread cell, the only budget where the plan switches; at the six others neither build
  changes plan, and they take that registration's own rules: identity checks at 8 and 9, no
  harm at 11, 12, 16 and 20 (below).
- **What is not carried over unchanged, said exactly** (each decided by the Coordinator under amended C6 (#306), subject to the Researcher's review):
  - **the twin arm.** The precedent ran two arms (head and base). This run keeps the
    service-core registrations' noise control N1 (a validity condition of the P1 rule it
    uses), which needs a third arm, the base's byte-identical twin. `ab` runs every arm at
    every budget, so the twin runs at all seven. That is a departure from the precedent's arms
    structure, made for P1's validity rule; it changes no budget, round count or threshold.
    **Decided:** the twin runs at all seven budgets; N1 is a validity condition at 10 only,
    and the twin's other six cells are reported, not judged.
  - **the replacement.** The precedent allowed one replacement of an invalid run, as decided
    for that experiment. The Researcher (REVIEW 6742157313, answer 4) found no authority that
    carries it to this one. **Decided: no replacement attempt**: one attempt, its kind of
    outcome recorded, no retry to pass.
  - **the primary's rule.** The precedent's primary (speed at least 1.03 with its lower bound
    above 1.00) is not this run's P1. **Decided:** P1 keeps the service-core outcome map and
    readings, unchanged from those registrations.
- **The cost:** 7 budgets × 3 arms × 20 rounds = **420 runs**, at about 2.2 s a run about
  **15.4 minutes** of runs (the precedent's two arms were 280 runs, about 10.3 minutes; the
  twin adds 140 runs, about 5.1 minutes), plus two release builds and the corpus: **about 20
  minutes from built, 25 from cold.**
- **No PMU telemetry:** `--machine-state` is not passed (`perf_event_paranoid=4` on the
  workstation; `telemetry-check/20261109T125938Z` found the #298 counters UNAVAILABLE).
  `ab env --facts`, `lscpu`, the raw discovery reads and `ab sample` before (and `ab sample`
  after) the run go to `diagnostics/`, each bounded and its exit archived, never judged and
  never used to select runs (D064; Revision 2, R2).
- **The plans each build must run**, from every run's two thread records, as (leaders,
  plain helpers, pipelined helpers, service workers; the hub's feeder not counted):

  | threads | base and twin | head |
  |---|---|---|
  | 8, 9 | 4 × 2 plain | 4 × 2 plain |
  | 10 | 3 × 3 pipelined | 2 × 3 plain + 2 service (svc2) |
  | 11, 12, 16, 20 | 3 × 3 pipelined | 3 × 3 pipelined |

  These are the sampled thread counts. That the windows are 16, 12 and 8 follows only from the
  pinned dispatch and `window` code at both commits, not from the names. **Equal plans at the
  other budgets are a fallback check, not fallback equivalence:** they don't show the binaries
  equal there (the candidate's discovery and layout differ); whether the candidate harms them
  is the no-harm predicate's question, at 11, 12, 16 and 20.

## The rule (`bench/service_guarded.py`, fixed with the freeze)

**Admission** (the earlier service-2x3 report's evidence boundary restored; each failure says
its kind, and none is a reason to retry):
- **identity:** `commits.txt`'s candidate and base are the frozen file's; no `DRY-RUN.txt`;
  `arms.txt` is exactly the three registered arms; `inputs.txt` gives both base paths' digests,
  equal;
- **workload:** `corpus.sha256` is the registered corpus's (`bench/wikidata.json`,
  82f65ace…); every run's corpus `wikidata`, its output exactly the registered 209,700,196
  bytes, its digest hex, one digest across every run;
- **preflight:** the record and its exit status checked whole: TARGET (a record of only
  `{"ok": true}`, or `ok` beside an unknown quota, an absent snapshot or missing observations,
  is refused);
- **structural:** every row's arm one of `arms.txt`'s; `threads` and `round` integers, never
  booleans; `termination` exactly `"success"` (Revision 2, R1: ab's parent records how it
  reaped each child; `"exit N"`, `"signal N"`, `"unknown"` or no field refuses); `err` present
  and empty (ab's failed child prints its error there; a child that printed nothing is marked
  `broken`, refused); no `exit`/`status` other than 0 where a row carries one (ab's rows carry
  `termination`, not these); finite positive wall, CPU and RSS; 20 rounds a cell, each once;
  every run's plan as in the table;
- **N1:** the twin against the base at 10 threads, both bounds of its 95% interval within
  [0.97, 1.03] (the service-core control's method, unchanged; its placement at 10 only is decided by the Coordinator under amended C6 (#306), subject to the Researcher's review).
  **What changed in its implementation:** service-2x3's twin was the same decoder under two
  arm names in one build; here it is the base executable copied byte for byte to a second
  path. Both are A/A controls of run-to-run noise under the interleaving; neither certifies
  differential overhead between the two builds or between-build layout effects. The twins at
  the other six budgets are reported, not judged (answer 4: extending validity to them would
  change the selection policy, not only the reporting).

**P1 (judged, 10 threads):** speed (base wall over head wall) and CPU (base CPU over head CPU),
each the geometric mean of the 20 rounds' paired ratios with its two-sided 95% t-interval,
co-primary, each by the outcome map: GAIN lo > 1.02 (CPU SAVING); SMALL GAIN 1 < lo ≤ 1.02;
NULL lo ≥ 0.97, hi ≤ 1.03, 1 inside; REGRESSION hi < 1 (CPU COST); INCONCLUSIVE otherwise.
**Readings:** THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT (a speed GAIN or SMALL GAIN, CPU
not a CPU COST); A TRADE (a speed gain with a CPU COST); NOT REPLICATED (NULL or REGRESSION);
INCONCLUSIVE otherwise. The head runs fewer decoding threads than the base at 10 (8 against
9): no extra-thread label applies.

**No harm (judged, 11, 12, 16 and 20 threads), the pipelined default's rule unchanged:** each
head-over-base wall ratio's one-sided upper bound at most 1.02, the bound Bonferroni over the
four (t at 1 − 0.05/4, 19 degrees of freedom, 2.4334); an interval too wide to show it counts
as not shown. It is reported beside P1 (shown at every one, or not shown at the budgets named),
and like P1 decides nothing about adoption.

**The qualifications the report states with them** (the earlier reports', restored):
- CPU INCONCLUSIVE means no CPU cost was detected, not that none exists; a speed gain with
  CPU INCONCLUSIVE keeps its reading but does not meet a demonstrated CPU non-inferiority
  condition (none is registered);
- NULL is bounded uncertainty within ±3%: a small effect is not excluded, and no effect is not
  shown;
- the two co-primary labels each have their own 95% interval: together they are not a
  simultaneous 95% statement; both intervals are reported;
- a product gain does not identify service-thread causality: svc2 against the current default
  changes plan, teams, window and thread count together.

**Reported, not judged:** the identity checks at 8 and 9 (speed with the ±2% flag, the
precedent's), the twins at every budget but 10, every cell's CPU, and **peak RSS**: the head's over the base's as a paired
geometric mean with its interval, and every arm's medians (answer 3). **No memory condition
is registered, and no result of this run can be advertised as passing one.** The old
64/74 MB corpus medians are not a resource bound.

**Attempts:** **one attempt; no replacement is registered** (decided by the Coordinator under amended C6 (#306), subject to the Researcher's review). Every attempt is kept and reported with its kind of outcome: VALID;
INVALID (identity, workload, structural or N1, said which); REFUSED by the preflight (before
any measurement); REFUSED otherwise (nothing ran); INCOMPLETE. What follows any attempt that is
not VALID is decided then, by the team under amended C6 (#306): never a retry until a pass.

## Before readiness: the recipe exercised (answer 6)

`bench/test_service_guarded_recipe.py` (CI) runs the whole recipe in a scratch repository
with a test-fixture freeze, stand-ins for Cargo, the harness, the preflight, the corpus builder,
`lscpu` and `gh`: both builds (the base's in its worktree), the copied and hashed binaries and
the twin, the preflight and its check, the corpus manifest, the run, the report and the bundle,
on every exit path (VALID; INVALID (n1); REFUSED by the preflight with no `runs.jsonl`; a DRY
check, refused by the judge as DRY; INCOMPLETE by a crashing harness; REFUSED unfrozen, dirty
and changed-after-freeze) and through `just submit`. `bench/test_service_guarded.py` (CI) holds
the judge to the Researcher's witnesses and the rest of its admission. **A DRY run still reads
clocks (one round of `ab run`); its numbers are excluded from evidence and the judge refuses
it.**

**The real path, exercised once (not evidence):** on the shared 4-core machine (not the
target), in a throwaway clone (never pushed) of 1e635647 with the recipe imported and a
pointer-only freeze (base: master 5c98d423; candidate: the clone's commit), `just
bench-service-guarded 1 <a small .bz2>`: the base's real release build in its worktree and the
head's, the copied and hashed binaries and the twin, the real preflight (NOT TARGET, exit 3,
recorded; the DRY check continued), the manifest (`corpus.sha256`, `arms.txt`, `inputs.txt`),
one round of `ab run` (9 runs), the judge's report and the bundle (25 files). Outcome INVALID
(identity): a DRY run, the corpus not the registered one, the preflight NOT TARGET, the
outputs not the registered length, as it should be. The eventual frozen inputs were not
edited for it.

## Revision 2 (9 November 2026): the Researcher's REVISE, REVIEW 6746023579

Prospective: written before any freeze, queueing or run, at a DRAFT that names no commits.
The Researcher's report (inbox 6745811380, on `ceb53e4f`, reviewing 910fd5de) asked for two
repairs and confirmed two things to keep. Each change below is to the harness, the recipe, the
judge or the freeze record; **no grid, threshold, arm or estimand changes.**

- **R1, the child's termination (report §1).** At 910fd5de the parent kept a child's stdout
  and dropped its exit status, so a valid-looking record written before a nonzero exit or a
  signal was indistinguishable from a successful run; the judge's `exit` check was reachable
  only by a hand-injected field. Now:
  - `ab run` records every child's `termination` as the parent reaped it: `"success"`,
    `"exit N"`, `"signal N"` or `"unknown"` (the shared harness commit cb2c233e from #305,
    cherry-picked; it also adds `ab env --facts`, below). The records carry `termination` and
    no `exit` field.
  - `bench/service_guarded.py` refuses any row whose `termination` is not exactly
    `"success"`, a missing field included, as INVALID (structural).
  - The producer is tested at its own boundary: `ab run` itself with stand-in children that
    write a valid-looking record and then exit 0, exit 17, are killed by SIGKILL, or write
    nothing (examples/ab.rs, `the_run_records_each_childs_termination`), and, in
    `bench/test_service_guarded.py`'s `Producer`, each record that real `ab run` writes judged
    in place of a clean attempt's row: success VALID; exit 17, signal 9 and the empty child
    INVALID (structural). CI builds `ab` for it. The judge's own tests refuse a missing
    termination, `"exit 17"`, `"signal 9"`, `"signal 15"`, `"unknown"` and non-canonical
    spellings. The recipe's end-to-end test carries a row with `"exit 17"` to INVALID
    (structural).
  - The harness and the judge are frozen together, as `examples` and the judge are both
    measured inputs.
- **R2, the diagnostics' acquisition (report §2).** At 910fd5de the recipe ran `ab env` (which
  starts spin workers and probes perf), `lscpu` and `ab sample` unbounded before the preflight.
  Now:
  - `ab env --facts` replaces `ab env`: procfs, sysfs, cgroupfs and `sched_getaffinity` only,
    no thread started, nothing spun, no perf counter opened.
  - Every external diagnostic outside the timed run (`ab env --facts`, `lscpu`, the raw
    discovery reads, `ab sample` before and after) runs through one helper, bounded by
    `timeout --kill-after=5 30` (TERM at 30 s, KILL 5 s after). Its stdout, stderr
    (`NAME.stderr`) and exit (`diagnostics/exits.txt`) are archived.
  - **The failure disposition:** a failing or timed-out diagnostic is archived as failed and
    nothing else. It never refuses the attempt, never changes the outcome, and never deletes,
    replaces or selects a timed row. (At 910fd5de, `ab sample` after the run failing would
    have stopped the recipe before the judge.) It is not target evidence.
  - **The preflight stays distinct:** `svc_target` keeps its own bound
    (`timeout --kill-after=10 120`), its plan exercises and its whole-record check by
    `bench/svc_target_check.py`, and its quota discovery (`parallel::cpu_quota_of`). The
    Researcher found that discovery sound (report §3), and `ab env --facts`'s quota is not
    made authoritative.
  - The recipe's tests use a stand-in `ab` that refuses `env` without `--facts`, check every
    diagnostic's exit is 0 in a clean run, check that failing diagnostics leave a VALID
    outcome and all 420 rows, and check statically that every diagnostic goes through the
    bounded helper and the preflight keeps its own bound.
- **R3, the raw discovery provenance (report §3's archive question).** Diagnostics only, not a
  new gate. `python3 bench/service_guarded.py discovery` (`diagnostics/discovery.json`)
  archives:
  - the process's `/proc/self/cgroup` and `/proc/self/mountinfo` as read;
  - for each cgroup2 mount, and each v1 mount with the cpu controller, its membership line and
    every directory from the mount point down the membership path, each with its existence and
    its quota files (`cpu.max`; v1 `cpu.cfs_quota_us` and `cpu.cfs_period_us`) as read, absent,
    or failed with the error;
  - for v2, the mount top's `cgroup.type`.

  It derives no quota verdict, and nothing judges it. Read by a child of the recipe's shell,
  it is in the cgroup that `ab` and its children run in.
- **R4, the base's currency at the freeze (report §4, Base).** Ancestry alone does not show the
  base was current. `bench/service-guarded.frozen` now names `base_fetched <UTC time>` beside
  `candidate` and `base`. The freeze is: fetch master, record `origin/master`'s SHA and the
  fetch time, merge it into `perf/service-guarded` (a merge, never a rebase), and name the
  merge as `candidate`.
  - The recipe refuses a freeze without a well-formed fetch time and copies it into
    `commits.txt`.
  - The judge requires `commits.txt`'s `base_fetched` to be the frozen one (identity).
  - Master moving after the freeze does not change this run's estimand; a later adoption
    decision accounts for any intervening changes.

**What the design does and does not claim** (report §4, recorded as the Researcher qualified
it):
- **Separate intervals.** P1's speed and CPU intervals are each a 95% interval. Together they
  are not a joint or simultaneous 95% statement.
- **CPU INCONCLUSIVE is not CPU non-inferiority.** It permits the registered product reading
  and proves no CPU-cost bound.
- **No harm, and its scope.** It is judged at 11, 12, 16 and 20 only (one-sided, Bonferroni
  over the four, t at 19 df 2.4334, the usual paired-log assumptions across rounds). 8 and 9
  are reported identity flags, not passed non-inferiority gates: **no result of this run is a
  claim of "no harm at all fallback budgets"**. Failing to show the upper bound at most 1.02 is
  not proof of a regression.
- **N1 at 10.** It constrains same-binary run-to-run instability at that cell. It neither
  validates every cell nor bounds head-specific instrumentation effects. The twins at the other
  six budgets stay reported, and no row is deleted, and no gate added, after seeing them.
- **Arm order.** At twenty rounds, a three-arm six-round rotation cycle is not perfectly
  balanced. This is the declared harness design, and the observed order stays auditable in
  `runs.jsonl`.
- **The temporal limit.** A point-in-time TARGET does not certify unchanged quota or affinity
  through 420 runs. The registration already states this, and it is not a new telemetry
  requirement.

**Authority.** The grid, the twin's placement, no replacement and adoption's separation are
decided by the Coordinator under RULES C6 as amended in #306 (cc84bd5a), subject to the
Researcher's review. The Researcher assessed them as scientifically sound (report §4).

That assessment is not a certification of the authority newly delegated by #306. The
Researcher's authenticated instruction still pins the specification at a4f97714, and a re-pin
request (6745788393) is pending. **The Researcher has not accepted it**, and nothing here
claims the pin changed.

**Not done in this revision.** Moving the corpus build before the preflight, which would shorten
the gap between the target observation and the measurements, was suggested in passing
(report §2). It isn't made here: the order is unchanged.

## The model (before any run; not a prediction to be judged)

From #258's runtime on this sample (history, not transfer): svc2@10 against the default at 10,
P1 1.045 [1.017, 1.073], CPU 1.146 [1.123, 1.170], peak RSS about 64 MB against 74 MB. The
guarded runtime adds per-block `Cell` stores and `failed` checks inside existing critical
sections and one lock acquisition per hub-thread exit. At the other budgets both builds run
the same plan, so the identity checks and the no-harm ratios should be about 1.00.

**Size:** #258's paired sd of the log-ratio at 10 threads, 0.046 (speed) and 0.038 (CPU), gives
intervals of about ±2.2% and ±1.8% at 20 rounds; N1's 0.032, about ±1.5%. **Run time:** about
2.2 s a run, so 420 runs in about 15.4 minutes, plus two release builds and the corpus: about
20 minutes from built, 25 from cold (above).

## Scope: what a result would and would not show

- **One i9 result does not qualify the class.** The predicate admits the 10-core Comet Lake-S
  class (family 6 model 165 with these counts); only the one i9-10900K is measured.
- **CI's off-path bench does not qualify svc2's gain.** CI's runners fail the predicate, so its
  paired bench prices only the candidate's construction and discovery cost on the existing
  plans. A null there is not a gain here, and a gain here is not a CI result.
- **Unmeasured:** every other budget, custom windows, other workloads and levels (covered for
  correctness, not speed), a busy machine, a quota or affinity change during a run.
- **Not the cause:** the run measures the product comparison, not which part carries it.

## Still open before a freeze

1. **The Researcher's re-review** of this revision (Revision 2's R1 to R4, at the exact
   source): the grid, the twin arm, no replacement and P1's rule are each decided by the
   Coordinator under amended C6 (#306), subject to the Researcher's review.
2. **The base's merge, SHA and fetch time:** at the freeze (above; Revision 2, R4).
3. **Adoption** (D067 §5): a team decision under amended C6, made separately after the
   evidence by a Guardian-reviewed decision-record entry; not this registration's to answer,
   and not presumed by it.
