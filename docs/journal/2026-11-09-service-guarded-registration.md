# The guarded service default against the current default: registration (DRAFT)

9 November 2026. **DRAFT: not frozen, not queued, not run.** Nothing in this entry is a
measurement. `bench/service-guarded.frozen` names no commits, so `just bench-service-guarded`
refuses. A freeze needs the Researcher's assessment and the Coordinator's registration
(RULES C1, C5); the run is Louis's, on the i9, after that. The Guardian's D069 (#145, comment
6741390621) resolved D067's technical objections at 6642faf4 and said to draft this; timing,
queueing and production admission stay gated by it.

**What this does not presume.** D067 §5's adoption question is open and Louis's: the change
makes svc2 an automatic default on a class of machines CI can't exercise, and how CI's
unchanged acceptance rule applies to that is not settled. This registration measures the
candidate on the one machine it can; it decides nothing about adoption, and no outcome below
is a merge or release rule.

It follows the switched-default precedent (`2026-11-07-pipelined-default-registration.md`:
the exact switched default against the exact current default through the same public entry
point, two builds, identity checks at neighbouring budgets) and the service-core
registrations (`2026-11-08-service-2x3-registration.md`: the paired estimand, the outcome map,
co-primary speed and CPU, the twin noise control N1), unchanged where they apply.

## The candidate and the base

- **The candidate:** `perf/service-guarded` (#302) at its frozen commit. `StreamDecoder::new`,
  and so `BzReader::parallel`, `parallel_with` and `decode_parallel`, builds svc2 (two teams
  of three that only build and walk, two service workers; lookahead 2; window scaled to two
  teams, 8 blocks at `ab`'s 40) when, and only when, no plan is forced and
  `parallel::service_target` holds: exactly 10 threads, AVX2 and CLMUL kernels, x86-64 Linux,
  a coherent topology snapshot of 20 allowed online CPUs on 10 cores, all `GenuineIntel`
  family 6 model 165 with a 20 MiB L3, and an established CPU quota of none or at least 10
  (`docs/journal/2026-11-09-service-guarded-scope.md` §2). Everything else is master's
  dispatch, unchanged.
- **The base:** the current default: master at the commit the candidate's branch was last
  merged from, named by SHA in the frozen file, an ancestor of the candidate (the recipe
  checks both). There the default at 10 threads on the i9 is three pipelined teams of three
  with park and small (`PIPELINED_PLAN`, `PIPELINED_LEVERS`), window 12.
- **The runtime is not #258's.** The measured 3e201246 is a different runtime (the guard, the
  terminal state, the completion witness, discovery and dispatch are new). #258's P1, 1.045
  [1.017, 1.073], and #271's reported D-a2w8 are history: neither transfers to this candidate,
  and this run does not pool with them.

## The run

`just submit bench-service-guarded` on the i9, at the frozen commit (CLAUDE.md's workstation
recipe shape: one new directory an attempt, its console log in it, one tarball whatever the
outcome).

- **Refusals before anything runs**, as the precedent's: the frozen file unset; the base not
  an ancestor of the candidate; uncommitted changes, or untracked files among the measured
  inputs; HEAD's measured inputs differing from the candidate's (`src`, Cargo's files and
  configuration, `examples` (the harness and the preflight), `tests/common` (the harness's
  `#[path]` module), `bench/service_guarded.py`, the corpus's builder and manifest, the
  bundler, the recipe and this file); a `CARGO_*`/`RUST*`/`AB_CENSUS_MS`/`AB_SAMPLE_MS`
  override; differing toolchains; a Cargo configuration only the base build would read; the
  base worktree not clean at the base; a CORPUS without DRY.
- **The builds:** each in its own checkout, Cargo's messages naming each executable, copied
  into `bin/`: `ab-base` (the base's harness), `ab-base-twin` (a byte-identical copy, checked
  by SHA-256), `ab-head`, `windows` (the corpus builder) and `svc_target` (the preflight),
  with their digests and the toolchain in `inputs.txt`.
- **The preflight (actual target), before any measurement:** `bin/svc_target`
  (`examples/svc_target.rs`) prints `Machine::here()` as read on the i9 then (the plan
  policy's facts, the predicate's topology snapshot and quota), the kernels, the predicate
  and `Plan::for_threads` at 9, 10 and 11, and what `BzReader::parallel(.., t)` builds at each
  by its own dispatch (`observe_builds`), after a small decode checked byte for byte and a
  join. It exits 0 only if the predicate holds at 10 and not at 9 or 11, the snapshot is read
  and the quota established, svc2 is built at 10 (8 decoding threads, window 8, every hub
  thread ended) and not at 9 or 11. **Otherwise the attempt is REFUSED, before any timed
  run**, its `preflight.json` archived in the bundle. These observations are consistent at
  one moment, not an atomic snapshot: affinity, quota or hotplug changing during the run is
  outside them (the runs' own thread census below is the per-run check).
- **The arms:** `mibz2` (the head's `BzReader::parallel`), `mibz2@bin/ab-base` (the base's,
  the same entry point) and `mibz2@bin/ab-base-twin` (N1). Unpinned, as a caller gets them.
  Each run is its own child process (`ab child`), so its wall, CPU (`RUSAGE_SELF`: every thread
  of that process: the scanner, the feeders, the caller's reader and the pool helpers
  included) and **peak RSS (its own `ru_maxrss`, which never resets, so one process per run is
  what makes it the run's)** are recorded for every run, beside each other.
- **The grid:** 9, 10 and 11 threads; 20 rounds, paired by round, arms' order rotated as `ab`
  does; the bench's Wikidata sample. 3 arms × 3 budgets × 20 = 180 runs.
- **No PMU telemetry:** `--machine-state` is not passed. The workstation runs
  `perf_event_paranoid=4`, so the #298 counters are all UNAVAILABLE there
  (`telemetry-check/20261109T125938Z`); nothing here depends on them. `ab env`, `lscpu` and
  `ab sample --count 5` before and after the run are recorded under `diagnostics/`, as
  diagnostics only, never judged and never used to select or drop runs (D064).
- **The plans each build must run**, read from every run's threads twice (`thread_cpu`, the
  child's busy threads; `observed_threads`, every thread the parent saw), as
  (leaders `mibz2-N`, plain helpers `mibz2-coop-N`, pipelined helpers `mibz2-coop-pN`, service
  workers `mibz2-svc-N`; the hub's feeder `mibz2-svc-feed` not counted):

  | threads | base and twin | head |
  |---|---|---|
  | 9 | 4 × 2 plain | 4 × 2 plain |
  | 10 | 3 × 3 pipelined | 2 × 3 plain + 2 service (svc2) |
  | 11 | 3 × 3 pipelined | 3 × 3 pipelined |

  The windows follow from the plans (`window`, unchanged at both commits): 16 at 9 (4 × 2),
  12 for 3 × 3, 8 for svc2.

## The rule (`bench/service_guarded.py`, fixed with the freeze)

Speed is the base's wall over the head's, CPU the base's CPU over the head's (above 1: the
head is faster, or spends less); each is the geometric mean of the 20 rounds' paired ratios,
with its two-sided 95% t-interval (19 degrees of freedom). Unchanged from the service-core
registrations.

- **Validity:**
  - the preflight passed (`preflight.json`'s `ok`);
  - only the registered rows: the Wikidata sample, 9/10/11 threads, the head's `mibz2` and one
    base and one twin binary; every run with a hash, a positive length, finite positive wall,
    CPU and RSS; no failed run; 20 rounds a cell, each once;
  - one output hash and length across every run;
  - every run's plan as in the table, in both thread records;
  - **N1:** the twin against the base at 10 threads, both bounds of its 95% interval within
    [0.97, 1.03]: the service-core registrations' control and method, unchanged (RULES C6
    reserves them to Louis).
- **P1 (judged, 10 threads):** speed and CPU, co-primary, each by the outcome map:

  | outcome | on the 95% interval [lo, hi] | CPU's wording |
  |---|---|---|
  | GAIN | lo > 1.02 | CPU SAVING |
  | SMALL GAIN | 1 < lo ≤ 1.02 | SMALL CPU SAVING |
  | NULL | lo ≥ 0.97, hi ≤ 1.03, 1 inside | NULL |
  | REGRESSION | hi < 1 | CPU COST |
  | INCONCLUSIVE | anything else | INCONCLUSIVE |

  **Readings:** THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT (a speed GAIN or SMALL GAIN, CPU
  not a CPU COST); A TRADE (a speed gain with a CPU COST); NOT REPLICATED (NULL or
  REGRESSION); INCONCLUSIVE otherwise. The head runs fewer decoding threads than the base at
  10 (8 against 9), so no extra-thread label applies.
- **Identity checks (9 and 11, reported, not judged):** both builds run the same plan; their
  speed is reported with the ±2% flag (an interval wholly beyond ±2%). Their plans count
  toward validity: they are the fallback's boundary, svc2 not built at 9 or 11.
- **Peak RSS and the medians (reported, not judged):** at every cell, the head's peak RSS over
  the base's, as a paired geometric mean with its interval, and every arm's median wall, CPU
  and RSS.
- **Attempts:** one slot. At most one replacement, only of an attempt that is not valid
  (`valid` false) or was refused by the preflight or a refusal check; a valid run is never
  replaced. Every attempt is kept and reported. No pooling across attempts or with #258/#271.

## The model (before any run; not a prediction to be judged)

From #258's runtime on this sample (history, not transfer): svc2@10 against the default at 10,
P1 1.045 [1.017, 1.073], CPU 1.146 [1.123, 1.170], peak RSS about 64 MB against 74 MB. The
guarded runtime adds per-block `Cell` stores and `failed` checks inside existing critical
sections and one lock acquisition per hub-thread exit; the model expects no measurable change
from them, and that expectation is exactly what is not known. At 9 and 11 both builds run the
same plan, so the identity checks should be about 1.00.

**Size:** #258's paired sd of the log-ratio at 10 threads, 0.046 (speed) and 0.038 (CPU), gives
intervals of about ±2.2% and ±1.8% at 20 rounds; N1's 0.032, about ±1.5%. **Run time:** about
2.2 s a run, so 180 runs in about 6.6 minutes, plus builds (two release builds, the base's in
its own worktree) and the corpus: about 10 minutes from built, 15 from cold.

## Scope: what a result would and would not show

- **One i9 result does not qualify the class.** The predicate admits the 10-core Comet Lake-S
  class (family 6 model 165 with these counts: by published specifications several i9 and
  Xeon W parts); only the one i9-10900K is measured. Its other members are unmeasured, and
  stay so after this run.
- **CI's off-path bench does not qualify svc2's gain.** CI's runners fail the predicate, so
  its paired bench prices only the candidate's construction and discovery cost, on the
  existing plans. A null there is not a gain here, and a gain here is not a CI result.
- **Unmeasured:** every other budget (svc2 is not built there, by the predicate), custom
  windows (8 is measured; others scale by `window`), other workloads and levels (covered for
  correctness, not speed), a busy machine, a quota or affinity change during a run.
- **Not the cause:** svc2 against the current default changes plan, teams, window and thread
  count together; this run measures the product comparison, not which part carries it
  (#258's C1 asked that of the old runtime).

## Open questions for the Researcher's review

1. **The base.** Master's commit at freeze (an ancestor of the candidate), or a release tag's
   commit by SHA, as the precedent used v0.2.5's? Master's is the exact current default; a
   release is what users have.
2. **The grid.** 9/10/11 with identity checks at 9 and 11, or the precedent's 8–20 grid with a
   no-harm rule at 11+? Above 10 neither build changes plan, so a no-harm rule would test only
   drift; 9 and 11 are the predicate's boundary.
3. **RSS.** Reported, not judged, as in the service-core registrations. Should P1 carry a
   memory condition (for example, the RSS ratio's upper bound below a registered ceiling), as
   the Optimiser's "within its CPU/memory conditions" suggests? If so it must be registered
   here, before the freeze.
4. **N1 at 10 only.** The twin runs at every budget (it costs 60 runs, about 2.2 minutes); only
   10's is a validity condition. Is that the right placement, or should N1 hold at 9 and 11
   too?
5. **The preflight's strictness.** It refuses on `Unknown` quota. If the workstation's cgroup
   layout turns out to be unreadable or namespaced, the run cannot happen at all without a
   change to discovery, which is a source change and a new freeze. Should a preflight-only dry
   run (`svc_target` alone, untimed, reported) be asked of Louis first?
6. **The recipe's dry run.** Its refusal path is checked (the draft refuses at the frozen
   file). A DRY run past the freeze check, on a frozen commit, has not been run: the
   precedent's found a recipe bug that way. It should precede the ready tag.
7. **Adoption.** Not this registration's to answer (D067 §5); listed so the freeze does not
   imply one.
