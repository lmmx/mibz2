# The guarded service default against the current default: registration (DRAFT)

9 November 2026. **DRAFT: not frozen, not queued, not run.** Nothing in this entry is a
measurement. `bench/service-guarded.frozen` names no commits, so `just bench-service-guarded`
refuses. A freeze needs the Researcher's assessment and the Coordinator's registration
(RULES C1, C5); the run is Louis's, on the i9, after that. The Guardian's D069 (#145, comment
6741390621) resolved D067's technical objections at 6642faf4 and said to draft this; timing,
queueing and production admission stay gated by it.

**Revision (the Researcher's REVISE, REVIEW 6742157313 on #145; report and probes at
`state/reports/service-guarded-registration-6741912733/` on b365250e):** the judge now admits
an attempt's directory, not a bare runs file, and binds its workload, identity and preflight
(R1, R2); the row schema is strict (R2); the report keeps the earlier qualifications (R3); the
base, grid, RSS, N1 and replacement answers below follow the review's six answers; the recipe
is exercised end to end past its refusals (answer 6); and the preflight-only entry
(`2026-11-09-svc-target-registration.md`, answer 5) comes first.

**What this does not presume.** D067 §5's adoption question is open and Louis's: the change
makes svc2 an automatic default on a class of machines CI can't exercise, and how CI's
unchanged acceptance rule applies to that is not settled. This registration measures the
candidate on the one machine it can; it decides nothing about adoption, and no outcome below
is a merge or release rule.

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
  candidate as `candidate`. The recipe checks the base is an ancestor of the candidate; that
  the base is *current* master at the freeze is the freeze's own check, recorded in the
  freezing commit's message. The bundle records both commits (`commits.txt`), both
  executables' digests and the toolchain (`inputs.txt`).
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
- **The grid:** 9, 10 and 11 threads; 20 rounds, paired by round, arms' order rotated as `ab`
  does; the bench's Wikidata sample. 3 arms × 3 budgets × 20 = 180 runs. **Louis's authority
  for this grid is pending** (RULES C6 reserves the thread grid; the Researcher has asked
  him). Neither precedent is this design: service-2x3 was one budget with six arms, the
  pipelined default seven budgets with two. The freeze waits for his decision.
- **No PMU telemetry:** `--machine-state` is not passed (`perf_event_paranoid=4` on the
  workstation; `telemetry-check/20261109T125938Z` found the #298 counters UNAVAILABLE).
  `ab env`, `lscpu` and `ab sample` before and after the run go to `diagnostics/`, never judged
  and never used to select runs (D064).
- **The plans each build must run**, from every run's two thread records, as (leaders,
  plain helpers, pipelined helpers, service workers; the hub's feeder not counted):

  | threads | base and twin | head |
  |---|---|---|
  | 9 | 4 × 2 plain | 4 × 2 plain |
  | 10 | 3 × 3 pipelined | 2 × 3 plain + 2 service (svc2) |
  | 11 | 3 × 3 pipelined | 3 × 3 pipelined |

  These are the sampled thread counts. That the windows are 16, 12 and 8 follows only from the
  pinned dispatch and `window` code at both commits, not from the names. **Equal plans at 9 and
  11 are a fallback check, not fallback equivalence:** they don't show the binaries equal there
  (the candidate's discovery and layout differ), nor that the candidate doesn't regress there.

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
  booleans; `err` present and empty (ab's failed child prints its error there; a child that
  printed nothing is marked `broken`, refused); no `exit`/`status` other than 0 where a row
  carries one (ab's rows carry none: the parent records no child exit status, so these two are
  the producer's own failure encodings); finite positive wall, CPU and RSS; 20 rounds a cell,
  each once; every run's plan as in the table;
- **N1:** the twin against the base at 10 threads, both bounds of its 95% interval within
  [0.97, 1.03] (the service-core control's method, unchanged; RULES C6 reserves it to Louis).
  **What changed in its implementation:** service-2x3's twin was the same decoder under two
  arm names in one build; here it is the base executable copied byte for byte to a second
  path. Both are A/A controls of run-to-run noise under the interleaving; neither certifies
  differential overhead between the two builds or between-build layout effects. The twins at
  9 and 11 are reported, not judged (answer 4: extending validity to them would change the
  selection policy, not only the reporting).

**P1 (judged, 10 threads):** speed (base wall over head wall) and CPU (base CPU over head CPU),
each the geometric mean of the 20 rounds' paired ratios with its two-sided 95% t-interval,
co-primary, each by the outcome map: GAIN lo > 1.02 (CPU SAVING); SMALL GAIN 1 < lo ≤ 1.02;
NULL lo ≥ 0.97, hi ≤ 1.03, 1 inside; REGRESSION hi < 1 (CPU COST); INCONCLUSIVE otherwise.
**Readings:** THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT (a speed GAIN or SMALL GAIN, CPU
not a CPU COST); A TRADE (a speed gain with a CPU COST); NOT REPLICATED (NULL or REGRESSION);
INCONCLUSIVE otherwise. The head runs fewer decoding threads than the base at 10 (8 against
9): no extra-thread label applies.

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

**Reported, not judged:** the identity checks at 9 and 11 (speed with the ±2% flag), the
twins at 9 and 11, every cell's CPU, and **peak RSS**: the head's over the base's as a paired
geometric mean with its interval, and every arm's medians (answer 3). **No memory condition
is registered, and no result of this run can be advertised as passing one.** The old
64/74 MB corpus medians are not a resource bound.

**Attempts:** **one attempt; no replacement is registered** (answer 4: neither precedent
authorises one here). Every attempt is kept and reported with its kind of outcome: VALID;
INVALID (identity, workload, structural or N1, said which); REFUSED by the preflight (before
any measurement); REFUSED otherwise (nothing ran); INCOMPLETE. What follows any attempt that is
not VALID is decided then, through the Coordinator and, for a rerun, Louis: never a retry until
a pass.

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

## The model (before any run; not a prediction to be judged)

From #258's runtime on this sample (history, not transfer): svc2@10 against the default at 10,
P1 1.045 [1.017, 1.073], CPU 1.146 [1.123, 1.170], peak RSS about 64 MB against 74 MB. The
guarded runtime adds per-block `Cell` stores and `failed` checks inside existing critical
sections and one lock acquisition per hub-thread exit. At 9 and 11 both builds run the same
plan, so the identity checks should be about 1.00.

**Size:** #258's paired sd of the log-ratio at 10 threads, 0.046 (speed) and 0.038 (CPU), gives
intervals of about ±2.2% and ±1.8% at 20 rounds; N1's 0.032, about ±1.5%. **Run time:** about
2.2 s a run, so 180 runs in about 6.6 minutes, plus two release builds and the corpus: about 10
minutes from built, 15 from cold.

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

1. **The grid's authority** (Louis, RULES C6): pending; the freeze waits for it.
2. **The base's merge and SHA:** at the freeze (above).
3. **The real-build exercise** (above), then the Researcher's re-review of this revision.
4. **Adoption** (D067 §5): not this registration's to answer.
