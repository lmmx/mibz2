# PR318 re-review: repair the primary gap block before freeze

Inbox [6759800960](https://github.com/lmmx/mibz2/pull/162#issuecomment-6759800960), dispatching QUEUED 6759596721. Reviewed head **795d4d75d130891a143cb2ff377f70e1ab041d54**. Role/rules pin **a2da49638ac54d7763de1c167bd054f67ffdc763**.

**REVISE before freeze; retain this diagnostic.** The previous five objections are substantially repaired. The newly primary gap block, however, can manufacture a perfect gap/slow association from missing wall readings while the complete replay says VALID. Its first G observation never receives its assigned treatment. These are small instrument repairs with consequential effects, not grounds to abandon the experiment.

## Answers

1. **REVISE** the gap block's measurement/admission and the remaining bound/readback inconsistencies below. No freeze or target run approved at this head.
2. **Use 0.25 s prospectively.** It targets the stated ten-thread harness phenomenon; 0.15 s at six threads is a different question, not a superior inherited constant. Name the estimand precisely: the slow-class frequency under the registered G versus immediate-relaunch policy, at ten threads on this host. This does not replicate the earlier six-thread experiment or identify a critical idle duration. Keep the threshold, seed, order, exclusions and interpretation fixed before data. No duration sweep or retry for a preferred mix.
3. **Yes: require the complete revised in-recipe DRY before freeze/readiness.** It must include the gap block, sustained block, applicable probe passes, identity, judgement and bundle. Require an empty consistency-problems list, correct counts and successful stage exits; DRY exit 0 alone is insufficient because `judge` returns 0 with problems in DRY mode. Expected host/freeze waivers remain disclosed. A no-SMT dry host cannot demonstrate pass 2's native plumbing: record that scope, with synthetic pass-2 coverage separately. The older pass-1 recipe check and standalone gap-driver check do not compose into evidence that the new whole recipe works. An unavailable idle window postpones readiness; it does not waive the check. This review performs no workstation run.

## What is accepted from the revision

I ran the exact source's **71 tests: all pass**. Source bytes were checked against all seven fetched Git blob IDs. The tests now cover the previous witnesses, and the inspected code agrees:

- elapsed-time probe progress replaces the unweighted interval-rate median; boundary loss is exposed through coverage, with the active median retained separately;
- R-only sibling coincidence is explicitly an asynchronous proxy; clock samples use nearby running-CPU observations;
- sustained output is raw input-byte trajectory, not a corpus-ratio conversion into instantaneous output speed; the old constant-rate/chunk-phase witnesses pass;
- per-row observation support and decisive rows distinguish compatibility, partial compatibility and untestability; no SIXTH MECHANISM or ONE MODE deduction;
- provenance, stage statuses and before/after identities are checked on replay; concatenation is content-validated; passes now run under a process-group supervisor;
- randomized probe schedule, serial-contrast caveat, truthful earlier DRY history and noncausal follow-up predictions address the earlier interpretation defects.

These repairs close those particular counterexamples. They do not prove general measurement accuracy or identify an i9 mechanism. No new native evidence was supplied or measured here.

## R1. Missing timing manufactures the primary result

`bench/two_modes.py:1135–1166` checks the gap child's exit/output but never requires its wall reading. `gaps_reading` at 1050–1063 defines slow as every record not classified fast. Thus missing/invalid timing becomes positive slow evidence.

`probe.py` runs the actual CLI judge on synthetic complete attempts constructed by the source's own fixture builder:

| Mutation of an all-fast baseline | Replay | Primary result |
|---|---|---|
| none | VALID, no problems | G0/30 slow; Z0/30 |
| remove the first G child's wall only | VALID, no problems | G1/30 slow; Z0/30 |
| remove every G child's wall | VALID, no problems | G30/30 slow; Z0/30; **THE GAP GOES WITH THE SLOW MODE**, Fisher p≈1.69e-17 |
| set every Z gap to −1 s | VALID, no problems | negative gaps admitted |
| reverse gap spawn timestamps and set spans to −1 | VALID, no problems | impossible chronology admitted |

This is an admission counterexample, not an assertion that the archived child ordinarily omits wall. Replay's stated purpose is to reject missing or contradictory evidence; it fails on the newly decisive quantity.

**Repair:** require finite strictly positive wall and span, finite nonnegative CPU where the per-block comparison is required, successful output/status and ordered timestamps. Classify neither missing nor nonfinite wall as fast or slow. Enforce `0 <= gap < .05` for Z and the registered G lower bound, with a documented first-run rule. Validate recorded gap against independently retained monotonic endpoints, not just against its own scalar. Add the single-field witnesses above. In outputs, replace `NO SLOW MODE IN THE BLOCK` when 1–4 slow observations exist with `INSUFFICIENT SLOW-CLASS SUPPORT` (or equivalent); a support floor is not absence of the mode.

## R2. The first G treatment is absent; reap time is not exit time

The registered order begins **G**. `run_gaps` at 986–1033 starts with `prev_exit=None`, skips sleeping, records `gap_s=None`, and still counts this child among the 30 G observations. `gaps_problems` explicitly exempts index 0. Therefore the advertised “every gap ≥.245 s” and 30 treated G observations are false for the normal driver, not merely malformed replay.

**Repair before freezing:** choose and register the initial condition. For 30 genuine inter-decode G intervals and 30 Z intervals, use one fixed conditioning child, retain its output/status and exclude it prospectively from the 60 treatment observations. Alternatively declare and analyze a distinct initial observation with the consequent counts; do not silently exempt it while claiming 30 treated G observations. A fresh initial .25 s sleep after arbitrary preflight is a different initial history from an interval after a decode; document that if chosen.

There is a second scope issue. `prev_exit=time.monotonic()` is taken **after** `stdout.read()` and `wait4`; next `t0` is **before** `Popen`. The scalar is parent-observed reap-to-launch-call time. If the child exits, the parent is descheduled for 300 ms, then reaps and immediately relaunches, this scalar can be near zero despite a 300 ms physical inter-process idle interval. Delayed process creation adds another unobserved interval. Hence Z's `<.05` check does not bound actual exit-to-spawn or decode-to-decode idle time.

The cheapest valid design can retain the archived child and call this the **assigned relaunch policy**, with explicit timestamp uncertainty. Retain launch/reap endpoints and bound or report delays where possible; do not claim exact physical idle durations from this scalar. If actual idle-time discrimination is essential, obtain suitable child/parent-boundary evidence and freeze its admission rule. G versus Z is a useful controlled policy contrast even when its physical latency is measured imperfectly; the claim must match it.

## R3. Finish the fixed-configuration admission and bound

The recipe records `identity.deadlines` and `sustained.rest_s`, but `admission_problems` never compares them with the registration. A replay with **deadlines={}**, **rest_s=0**, or both remains VALID with no problems (probe output archived). The supplied synthetic baseline itself lacks both fields. Check each field independently against the registered values, including the DRY rest exception. Keep the successful process-group supervisor work; this is a replay consistency repair, not evidence that the normal recipe changes those values.

The whole-attempt “about 60 minutes” is not a hard bound on this script. Fetch/build each invoke `bd corpus` with 900 s, two passes add 1800 s, concat 600, gaps 600, sustained 300 and judge 300: **5400 s before fetch, preflight, rest or cleanup**. The table also omits the new gap stage. Compilation, archive extraction, some digest calls and final bundling are outside `bd`; summing only supervised stages cannot bound them. `bounded.py` additionally allows two 10 s cleanup phases beyond the stated deadline/grace, and promises bounded refusal, not that all descendants necessarily disappear.

**Repair:** register a truthful total operational ceiling or an outer supervisor covering the attempt with a preserved partial-record path; include repeated stages and cleanup. State exactly which timeout is INVALID versus a setup REFUSED, retain it, never retry automatically. No need to rework the decoder. Standalone `bd` safety remains useful and its 125 stop remains necessary.

## Interpretation that should remain modest

The gap Fisher statistic is appropriately descriptive: sequential carryover and host state prevent treating its iid p-value as a causal confidence guarantee. Preserve counts, sequence and transition summaries. The randomized schedule removes the deterministic PPN alias; it cannot make accidental alignment with every periodic nuisance impossible. Correct that absolute sentence in §1 without adding another gate.

The sustained 2C envelope is conditional on bounded buffering/refill and sample timing; observed maximum `rchar` increment is not independently a proof of the decoder's buffering bound. The collector targets 10 ms but records actual times and may miss reads. Keep the raw observations, disclose the assumed envelope, and avoid upgrading shape beyond input-arrival evidence. It enters no signature, so this need not grow into a separate calibration experiment before collecting the bounded diagnostic.

Probe whole-window≥.90 coverage does not by itself guarantee≥.90 coverage in each early/late third. `readings` uses the whole-window flag for both; similarly its clock whole-window sample floor does not establish early/late support. Report or gate each subwindow's own support before reading a rise as temporal evidence. This is a narrow support correction, not a demand to identify clock causation.

## Information value and limits

Expected: close the previous instrumentation objections and freeze a cheap gap-first discriminator. Observed: the old witnesses are repaired and 71 tests pass, but the new primary block admits fictitious timing classifications and does not execute its first assigned gap. **No empirical performance gap has closed; no mechanism established.** The next valuable step is these small prospective fixes plus the complete DRY check, then a separately frozen diagnostic. Useful outcomes would select a controlled follow-up; none qualifies a production change or reopens a frozen verdict.

Reproduction: fetch the seven paths in `source-manifest.json` at the reviewed head, retaining their layout; place `probe.py` beside `bench/`. Run `python bench/test_two_modes.py` (71 tests), then `python probe.py`. The added probe uses the existing synthetic fixture builder; it calls the actual judge and asserts each reported admission failure. It builds or times no decoder. Native overhead, physical idle gaps, archived executable rehashing and any complete new DRY bundle were not verified. Migration SHA/manifest guards passed before receipt; channel audit returned []. Only this role's state branch and its REVIEW are written; task configuration unchanged.
