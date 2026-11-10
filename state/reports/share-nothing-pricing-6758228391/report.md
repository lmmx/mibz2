# Share-nothing pricing: REVISE before freeze

Inbox6758228391, dispatching6758128730. PR326 reviewed at **aae6dc5ed4779c094f1af932a384204f1933a25b**.
Spec pin a2da49638ac54d7763de1c167bd054f67ffdc763.
2026-11-10. Diagnostic review only; no timing, decoder implementation or merge.

## Decision

**REVISE** the interpretation, replay admission and conditional intervals before freezing. The paired round statistic and all-round primary analysis are sound choices for a bounded descriptive diagnostic. They do not establish a compact kernel's speed budget or falsify share-nothing when an interval spans unity.

Read registration, entire judge/driver, tests, recipe, bounded supervisor, and ab source at the pin. GitHub check-runs show successful completed checks with release/deploy/publish intentionally skipped; this is not independent execution of those jobs. The new sysfs flag changes parent observation while the child machine-state flag remains separately controlled. No target run or end-to-end dry recipe was rerun here.

## Required corrections

### 1. §4 draws conclusions that the measurements cannot support

“R's interval including1” is **inconclusive**, not “smaller table buys nothing” and not failure of the architectural premise. A constructed60-round log-ratio series,30 atlog(.8),30 atlog(1.2), gives geometric mean .979796 and the actual block5 interval [.879383,1.084322]. It includes a sizeable benefit and a penalty. The judge correctly reports only ratio_of_ratios_below_1=false; the registration supplies the erroneous stronger interpretation.

Likewise G9's interval containing3.50 establishes compatibility, not that the whole historical ladder still describes today's decoder. No intermediate level is measured, and failure to reject a point cannot establish equivalence.

Replace the1.44× kernel “common gate” and C/D/E consequences with the conditional budget analysis already recorded in REVIEW6758204909. C is withdrawn, E falsified, D4-delta is now roughly1.089MB estimated and has its own unchanged registered filters; the draft's1.44MB D table and proposed C step are historical. Do not reopen either by implication.

The 3.76/G2 curve may remain labelled historical MODEL, with its fixed numerator and transfer assumptions explicit. G2 is CPU10/CPU4, while400MB/s is a one-thread rate: using it as a1-to10 inflation factor implicitly assumes the unmeasured1-to4 factor is one. More fundamentally, the scanner-limited Workers arm does not reproduce ten continuously supplied independent workers. Idle workers may reduce cache contention; scanner/queue overhead also contributes CPU. G2 cannot identify the finder-fed kernel's inflation. Record that limit explicitly, not only that scanner limits wall.

The -2/-9 comparison tests a **recompression-level × worker-count interaction**. It does not isolate table capacity causally. Shared decoder changes need not affect the two corpora equally. A positive interaction is useful evidence of a favourable workload regime; a null or contrary interaction closes neither compact representation nor share-nothing.

### 2. Evidence replay currently accepts missing provenance

Against the exact admission function, these synthetic incomplete records passed:

- Remove wikidata-l2's digest from both corpora.json and identity.corpora_after: None==None satisfies the equality check.
- Give every run machine_pre={} and t_spawn_unix_ns=null: the dict/type-and-key check admits it.
- The supplied valid fixture contains no actual head, frozen SHA, source digests or archived preflight files; the full judge still returns VALID.

The live wrapper normally writes more than the fixture. These are **replay/admission holes**, not evidence that a real completed attempt has corrupt data. Close them before freezing: require present well-formed hashes on both sides, valid head/frozen identities and the registered source evidence, and re-evaluate archived preflight evidence rather than accept preflight_exit=0 alone. Reconcile any absent source or telemetry with an explicit unknown/error reason; actual unavailable frequency need not invalidate the diagnostic, but an empty invented state must not masquerade as an observation. Require usable finite/ordered timestamps, with documented clock semantics, for the temporal record used to interpret waits and modes.

The result can retain deviations rather than exclude runs. Separate missing/unread telemetry from observed governor/EPP/turbo deviations; current not_as_declared combines both. Record source/control failures as INVALID rather than selecting “good” rounds.

### 3. Quota preflight fails open when the hierarchy is unavailable

A synthetic root with /proc/self/cgroup='0::/missing.scope' and no mounted hierarchy returns:
state='unlimited',files={},limits=[].
Absence of a hierarchy is not evidence of no limit. Resolve/validate the active cgroup2 mount and the process's path; distinguish a legitimate absent controller/root cpu.max from an unreadable or nonexistent hierarchy/path. Require enough readable facts to establish unlimited, otherwise unknown/refuse. Add the missing-mount/path negative controls. This is a read-only collection defect, not a request to alter cgroups.

### 4. Mode-class bootstrap compresses the time axis

analyse() selects rs of fast or slow rounds, then summary(rs) resamples neighbouring entries in that selected list. For fast rounds1,3,5,7,9, a “block5” actually spans nine original rounds; for1,2,50,51,60 it bridges long gaps. The same label therefore ceases to mean five consecutive rounds.

Simplest correction: retain class counts and point estimates, omit conditional intervals. Or resample whole chronological rounds with their class labels, then calculate the class statistic within each resample and disclose empty/low-count classes. Keep the primary all-round sequence and simultaneous within-round G2/G9 pairing unchanged. Five selected rounds cannot support a dependable persistence-adjusted interval merely because MIN_CLASS=5.

## Six requested answers

1. **Sentinel:** acceptable as a descriptive, endpoint-bracketed indicator of another plan's state. Neither matching sentinels nor their <.21s classification identifies each intervening cell's mode. Keep all primary rounds. Report pre-clock as a covariate with missingness; do not add a second data-dependent exclusion or claim a low pre-clock diagnoses the in-decode mode. The scheduled sentinel itself can affect subsequent thermal/cache state, so the estimand includes that harness.

2. **60 rounds, block5:** acceptable as a fixed-budget exploratory attempt after the interpretation corrections; **not guaranteed95% coverage** under unknown persistence. Blocks1/5/10, lag1 and t sensitivity are useful but do not bound longer dependence. A run-wide latent shock can make all observed log ratios constant, yielding a zero-width bootstrap while repeated-attempt uncertainty is nonzero. Show chronological paired logs, elapsed round times and sentinel traces, and label intervals conditional on this attempt and assumed dependence treatment. Do not choose the narrowest interval or retry to obtain the desired band. A time-series diagnostic is preferable to increasing rounds by fiat.

3. **2/4/6/8/10 grid:** yes for the registered4-to10 question and knee description. No1-thread anchor means it cannot establish1-to10 inflation or directly calibrate the400MB/s numerator. Either keep the narrower estimand explicitly, or prospectively redesign with a1-thread anchor if that calibration is essential. There is no requirement to add a new arm merely to run a valid descriptive comparison.

4. **-2 proxy:** the existing caveat is necessary but §4 contradicts it by using G2 to decide C/D/E. Change the decision map as above. It is an optimistic footprint proxy, not a demonstrated upper bound on runtime or cost: more blocks can increase setup while smaller buffers reduce pressure, and neither sign dominates universally. Useful contextual counts would include blocks, rows and compressed bytes; do not present the ratio as isolated cache evidence.

5. **waited_s:** yes, retain and summarise by cell/position with timestamps and actual round duration. ab already records its wait; judge currently discards it from joined rows. Waiting is related to prior load and potentially prior treatment, so use it descriptively, not for excluding “bad” rounds or causally adjusting them away. “Five rounds≈one minute” is not reliable with variable gate waits and120s slot limits.

6. **Replacement gate formula:** no formula derived from G2 alone. Before data, record the conditional surface T<=min(Q,A/[(1-f+fg)h(g)]) and which terms this experiment leaves unobserved. If G2 is above the historical band, report the observation and revisit prospective experiments; do not change the current D4 filters or assign a new kernel budget after seeing it. Later qualification compares a concrete candidate with matched default throughput and charges construction, traversal and full live state.

## Checks, limits and repair acceptance

Executed standard-library probes against the exact downloaded Python source:
- registered732-slot schedule digest reproduces1fb24aee9592e70f764ee7dc739f2372c18796ef43a30f415eb534a2dc419ad2;
- all17 existing INVALID mutations reject at admission;
- clean synthetic fixture produces G2=1.25,G9=3.50,R=.357143 and VALID;
- four additional incomplete-evidence/quota probes demonstrate the defects above;
- correlated synthetic series demonstrates the unity-interval counterexample.

pytest is absent locally: **not a61-test suite pass**. No compressors, Rust build, process supervisor stress or full dry recipe run here. Existing bounded.py explicitly excludes descendants that create their own session; this recipe nests drive and per-slot supervisors. Ordinary forwarded TERM appears covered, but abrupt outer/nested-supervisor death was not tested here; no broader process-tree guarantee is certified. Finalisation is inspected as reuse of the earlier reviewed supervisor/finaliser, not a fresh guarantee of every failure mode.

probes.py extracts only two fixture builders from the pinned test file with AST, then calls the actual source; no decoder implementation. For reproduction, place the pinned share_nothing.py and test_share_nothing.py alongside it. probes.json preserves results.

Repair acceptance: correct §4 without retroactive gate changes; make missing provenance fail closed; repair cgroup unknown handling; remove or correctly resample conditional intervals; retain waits and chronological records. Then rerun the negative controls and an end-to-end DRY at the head intended for freeze. Existing CI being green does not answer these counterexamples.

Expected a bounded workload-scaling diagnostic; found a useful paired design with overclaimed interpretation and incomplete replay guards. No performance gain established. Fixes preserve its information value and prevent an ambiguous observation being used to close a viable architectural line. Migration verified; routing audit clean; only own state branch and one REVIEW on145 written.
