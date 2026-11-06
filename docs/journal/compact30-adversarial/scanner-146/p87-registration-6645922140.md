# P87 production registration: independent assessment

Trigger: [6645922140](https://github.com/lmmx/mibz2/pull/162#issuecomment-6645922140).
Reviewed registration d6401b55, candidate f142e38ea9ffbc37cce1047e643e8c2dbb907f9c against 08e7fc3b46c7a6adb61ce0e70b64e9973b95cbb7. Research only: no implementation, benchmark, merge or acceptance-criteria amendment. Bench results were not read. This review does not claim publication before their arrival.

## 1. Claim less: this is a regression screen, not demonstrated no harm

The registered rule rejects a row only when both runs flag it slower. Let p1 and p2 be each run's power against a particular harmful effect. Under independent runs, the repeated-flag rule's power is p1*p2, not the larger single-run power. For an illustrative normal estimator with 95% half-width h=1.96*SE and true slowdown h, each run flags with probability approximately 0.5; two such runs flag together only approximately 0.25. Host heterogeneity makes independence and a common effect additional assumptions.

The registration's half-width sentence also needs precision: exclusion depends on the interval endpoint, not half-width alone. With estimated slowdown d and interval [L,U], harms greater than U are excluded at the interval's stated coverage; a positive harm delta remains compatible when delta lies inside [L,U]. For example d=+2%, h=3% gives [-1%,+5%]: a 4% harm remains compatible although it exceeds the half-width.

Recommended claim: **“No reproducible statistically detected wall regression under this two-run screen; row-wise harmful effects up to the reported upper bounds remain compatible.”** Report every row in both runs, including single-run flags and disagreements. A single-run flag is unresolved evidence, not permission to pass because another host did not reproduce it. “Reported and weighed” leaves the acceptance decision discretionary; state that explicitly.

If genuine no-unacceptable-harm assurance is required, it needs a prospectively justified material margin delta per required control and upper bounds U<delta for all required rows/runs; otherwise INCONCLUSIVE. Do not pick delta from observed intervals. Marginal 95% intervals are not automatically simultaneous 95% bounds for every row. An all-controls intersection-union acceptance test has a different error claim from simultaneous interval coverage. No power calculation is established merely by two runs of ten rounds. This is advice, not authorization to change statistical methods or acceptance minima.

A separate conflict: [STEWARDSHIP](https://github.com/lmmx/mibz2/blob/f142e38ea9ffbc37cce1047e643e8c2dbb907f9c/docs/STEWARDSHIP.md) requires an end-to-end improvement beyond noise and reserves gate relaxation to the maintainer. A peak-RSS-only pass does not establish a faster decoder; the registration cannot itself certify that this alternative meets the merge gate. CPU-efficiency improvement likewise should be labeled as such rather than substituted silently for elapsed-time improvement. Obtain the applicable interpretation through the existing governance process; I grant no merge qualification. Not meeting the benefit screen is failure to qualify under this rule, not proof the construction has zero benefit.

## 2. No attribution to destination reuse alone

The registration explicitly denies mechanism attribution; retain that denial. The inspected src/lib.rs diff both selects in-place output over ll and avoids the rle allocation for the non-cooperative supporting kernel. The treatment is the bundle. CPU, wall or RSS differences do not partition its effect into allocation removal, destination reuse, residency, page placement or traffic mediation.

The current fresh-Scratch instruction totals in the candidate document differ by only 270–542 instructions; that scope includes first allocation/growth. They are not steady-state savings. I inspected the driver and reported table, not an independent replay of all raw profiles. Length plus CRC is not itself byte-exact parity; keep the separate byte comparisons and tests required by the registration.

## 3. Cheap discriminators: existing evidence first

- Archive both runs' runs.jsonl, env.json, result.json and comment.md by run ID before a rerun overwrites the PR comment. The existing workflow already uploads these. Record exact base/head, toolchain and actual CPU/core/affinity configuration, not just “4 threads.”
- Read the existing 1-thread, 2-thread spread and full-runner rows separately with CPU and elapsed time. The workflow already uses --spread: two threads are on separate cores on the documented two-core runner; four threads add SMT. This supplies topology sensitivity, not a two-SMT-siblings comparison or the i9 four-physical-core experiment. CPU/elapsed divergence distinguishes accounting categories, not a unique causal bottleneck. Keep each run separate.
- A small optional Callgrind diagnostic using the already archived driver's REPEATS can separate first growth from warmed scratch work without changing production code: for each arm record C1 and C2, total inclusive Ir over one and two calls with the same Scratch; C2-C1 is the second-call increment. Compare those increments between arms, separately from C1. This is instruction accounting only, not timing or isolation of destination-reuse causality. The driver's CRC uses XOR across repetitions: even repeats yield zero for identical CRCs, so that printed zero must not be used as a parity certificate. Retain the existing byte-exact checks.

Do not add a matched-allocation timing arm, sibling-affinity benchmark, PMU campaign or phase-harness rewrite to this PR under “cheap.” They alter the experimental design or measured paths. A retained but untouched allocation also does not recreate the separate destination's page touches/cache history. The existing phase harness's direct inversion path does not measure this integration's in-place intervention. The cheap additions above constrain interpretations; none identifies the mediator.

## Evidence and timing boundary

Primary reads: [registration](https://github.com/lmmx/mibz2/blob/d6401b55/docs/journal/2026-11-05-p87-production-validation.md); candidate's same document with instruction table, archived driver, src/lib.rs patch, docs/STEWARDSHIP.md and .github/workflows/bench.yml at f142e38e; PR #164 metadata. Role and MODE read at established e8e494a2/e44d87f9 pins after master retrieval failed. Existing causal assessment remains the background, not a new measurement.

The workflow posts a **pending** bench comment before measurement and edits it to final. “Before the first bench comment” is therefore ambiguous as an intended results-blind amendment boundary and, literally, may close before results. Do not silently reinterpret it as “before final results.” Record any amendment's actual time, data exposure and authority; if the registered boundary has passed, retain the rule and attach this critique rather than retroactively calling a change preregistered.
