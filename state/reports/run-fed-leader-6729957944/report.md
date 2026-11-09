# Run-fed leader registration: REVISE before freeze

Assignment [6729957944](https://github.com/lmmx/mibz2/pull/162#issuecomment-6729957944), dispatching [6729932826](https://github.com/lmmx/mibz2/pull/145#issuecomment-6729932826).
Reviewed PR #270 at **f94551ab8c359ec28dca02956ddca150595cc7cb**.
Policy pin: a4f97714cc14366984015869d9d094d89488ac57.
Scope: registration, admission, comparison fidelity, checked-construction architecture and interpretation. No native execution, timing, implementation or complete B5 certification.

## Decision

**REVISE.** The whole-decode entry point and the 9-thread window are substantially fair. The detached admission has false VALID paths; the documented control substitution is not implemented; the leader-lateness statistic compares unlike origins; and the draft reinstates the diagnostic B5 exemption and correctness-based retry rejected for #269.

I recommend **dir as the single prospective primary**, with hs a fixed secondary, if the question is whether this run-fed construction is worth pursuing. Disclose that this choice follows the indicative smoke. Do not select whichever arm wins the judged run. Changing primary alone fixes none of the findings below.

## 1. Admission: executable counterexamples

Ran the seven existing tests unchanged: all pass. Then extended their clean synthetic fixture to fifteen rounds and called the actual report function. Control returns VALID; each following mutation also returns VALID and a primary reading (probe.py, probe.json):

| Mutation | Why admission must refuse it |
|---|---|
| Remove hash and out from every record | Equality of absent values is not output evidence. |
| Negate every CPU value | Ratios of two negatives are positive, so interval validation does not validate raw measurements. |
| Replace each run-fed phase string with runs=-1, team_records=-1 for record arms (0 for controls), fallbacks=0 | Negative counts pass the !=0 tests; required timing phases are absent and printed as zero. |
| Change corpus label by round, retaining matched output values | Pairing uses only arm/round; no common corpus identity is enforced. |

These reproduce at the registered fifteen-round count, not just the three-round test setting. The fixture itself lacks head, binary, environment and corpus-digest provenance, yet passes; the reporter has no arguments to bind those identities. Workflow summary text prints the current head/digest, but does not compare them with an expected frozen identity. Its registered_head input is used for run_fed_stage, not this branch of the workflow. A default/custom ROUNDS value is accepted as a judged run although the draft requires fifteen.

Required repair: one explicit judged-admission interface binding the frozen full head, binary/build identity, corpus digest, fifteen rounds, 9-thread plan/window and actual environment. Keep flexible rounds as explicitly UNJUDGED diagnostics. Require present, correctly typed hash/output length and error/exit evidence; positive finite raw wall and CPU; nonnegative integral phase counts; required phase keys and consistent counts; observed constructor/plan evidence. Validate before division and raw-output construction, so absent cpu/wall fields produce INVALID rather than an exception. Retain all raw records and actual arm order. This is admission of the promised experiment, not a demand that a JSON schema authenticate arbitrary forged data.

The ab harness does preserve each child's exit code and error, pairs arm order, and archives raw timings. Those mechanisms are useful; a strict consumer is still required.

## 2. Incumbent and control: preserve the estimand

At 9 requested threads:
- default33 calls BzReader::parallel under forced PIPELINED_PLAN = 3 teams x 3;
- its 36 requested slots become window = 36*3/9 = 12;
- pipe3-both and all five run-fed sub-arms also use 3 teams, k=3, 12 slots.

At 10 requested threads the same calculation gives 40*3/10 = 12. At 3, default33 is still forced to THREE teams, whereas the research arms use ONE; the workflow correctly labels pipe3-both as the reported 3-thread reference. The dedicated report's default33 comparisons must therefore not be relabelled fair at 3.

Whole-entry wall brackets decode/open/drain on each arm. CPU is the harness's process accounting with sampler subtraction, not identical in scope to that wall bracket; retain its definition and apply the cap to that estimand.

The journal says a control interval outside [0.98,1.02] changes other readings to a ctl reference. The reporter never makes that substitution: ratios() always divides by REF. A fixture with ctl about 0.816 of REF and hs about 0.945 of REF yields LEAD, while hs/ctl is about 1.158. Thus the claimed alternative reading is materially different.

**Do not cure this by silently switching baselines after seeing the control.** Keep the production-reference primary fixed. If control fidelity fails a frozen, explicit rule, mark attribution inconclusive or require revision; report hs/ctl, dir/ctl and reg/ctl separately as mechanistic contrasts. A noisy interval overlapping [0.98,1.02] does not establish equivalence. Also ctl measures this worker/counter bundle's cost on the column path; it does not subtract arm-specific record sorting, barrier or atomic-counter overhead from the record arms.

## 3. leader_late is not an aligned completion comparison

In run_team.rs::invert, t0 is captured before construction of the stage/relay and pool dispatch. build_ns = time from that t0 through build/setup completion. But lead_ns is only time from the later start of the leader closure through lead(). The code tests lead_ns > build_ns.

Let d be leader-start minus t0, L its measured duration and B the measured build completion offset. Actual leader-late is **d+L>B**; the current predicate is **L>B**. With d=20, L=95, B=100 (arbitrary time units), the leader finishes at 115, after the build at 100, but is counted early. Dispatch delay and oversubscription can change d across arms. The statistic systematically misses this interval of late completions.

Record both completion offsets from a common t0 (and, preferably, start offsets separately), then compare completion times. A deterministic staggered-start test should expose the present misclassification. The draft's STOP condition uses ctl leader_late/runs < 1/4, so this is a registration blocker, not a cosmetic phase report.

Even corrected, build completion is not the end of the helpers' walk. join_wait is a wait-for-build measurement, includes gate cost, and is not a causal measure of recoverable whole-decoder wall. Aggregating run counts also weights short and long blocks equally. An occupancy trace plus matched interventions is needed for the critical-path claim.

## 4. The checked F order: architecture acceptable, proof still owed

A public Sorted token is not the only valid safety architecture. This module owns its buffers privately and its fills do not bypass run_stage's public checked interface. The alternative checked construction can be accepted without forcing a parallel Sorted constructor into run_stage solely for naming consistency.

The local proof obligations are concrete:
1. For each contiguous L-record share j and byte c, positive run lengths sum to R[j,c], with starts tiling [0,n) and per-byte totals matching counts.
2. Prefixing byte then share allocates disjoint F-row ranges [C(c)+sum(i<j)R[i,c], C(c)+sum(i<=j)R[i,c]). Run-count prefixes similarly allocate disjoint scatter slots.
3. Every scatter range is filled exactly; the sentinel is initialized; no F slice is exposed before all scatter writes are published.
4. The counted/scattered gates publish writes through AcqRel arrival chains and release/acquire opening. All helpers reach every required gate even after caught errors. A failed check cannot turn into a partial table with initialized length.
5. Each store ends inside the writer's owned row range. filled publication and the built gate precede setting length and walking.

The inspected count/merge/scatter/fill path embodies this design. Direct placement uses the same F-row prefix ownership without the F-order scatter, which is the constructive reason C31 is plausible. The presorted shortcut is more fragile: RunFill::new trusts presorted plus length; its invariant comes from private Path::Sorted set after sort_one_share, not length alone. Keep that call graph and buffer immutability explicit. A typed private token is worthwhile hardening if reuse grows, not a prerequisite by itself.

This is a source-level architecture assessment. It does not close the concurrent raw-pointer, emitter, panic-path or mutation-coverage audit. Output equality/fallback cannot repair undefined behaviour that already happened. Remove §3's assertion that B5 is required only before production. The standing requirement and any scoped Guardian HOLD survive diagnostic labelling.

## 5. What CI can and cannot conclude

Nine decoding threads plus scanner on four available logical CPUs is an oversubscribed condition. Record actual cpuset/quota/topology/SMT/host information; do not infer two physical cores merely from four vCPUs.

CI can report wall and CPU ratios, successful checked outputs, observed waits and progress on that environment. It cannot establish universal correctness, absence of deadlock, or the i9's critical path. CPU time is **scheduled service**, not invariant instruction work: spinning, SMT throughput, cache contention, frequency and scheduling all change it. Phase wall clocks include descheduling and interference; they are not costs in isolation. The claim that oversubscribed wall tests “less total work at equal synchronisation” is especially unsupported: hs and dir deliberately change synchronisation.

A simple counterexample to CI STOP as a target impossibility rule: baseline target run has L=10 and two helpers of service 6 each; a candidate changes L to 8 and each helper to 7. Under an ideal unsaturated max-stage model its span improves 10 to 8, while aggregate service stays at 22 (or increases to 23 with a small synchronization charge). An oversubscribed service-limited run may show no saving or a regression. These are illustrative scheduling models, not predictions of this decoder; they show why the proposed implication fails.

Retain CI as a bounded environment-specific screen. A CI STOP can justify a discretionary pause against alternatives under C3; it cannot permanently close the i9 lead or require only one specified future trace to reopen all constructions. Likewise C29 can reduce contention even when its leader is not locally last; “candidate only where leader critical” is too strong.

## 6. Primary-arm choice and stopping/retry plan

Prefer dir prospectively: it removes the r-entry scatter, the 8(r+1)-byte F-order array and one gate, targeting helper work while preserving the emitter. But count removal is not proven wall gain. Up to 256 predecessor lookups per share are dependent searches; interleaved byte write streams and cache-line sharing at ownership boundaries may matter. “Lookups negligible” does not follow from their count alone. Disjoint writes prove ownership, not freedom from false sharing.

Choose one primary at freeze, with exact thresholds and one production reference. hs remains a secondary construction comparison; reg/ctl tests the matched-list-kernel contribution; ls versus hs compares placement **and parallelisation/barriers/cache distribution**, not placement alone. Disclose all smoke observations and exclude them from the judged sample. If the original question remains specifically C30, keep hs primary and treat dir only as a separate future question; do not rename a dir success a C30 success.

Remove the “VALID fails” blanket rerun. Missing infrastructure artifacts may justify a prospectively defined limited retry; output mismatch, fallback, violated invariant or safety failure must be preserved and investigated, not rerun to erase the failure. Any rerun after a code repair needs its own frozen identity.

The workstation snippet remains a draft, not admitted: its twenty-round/ten-thread design, occupancy captures and executable validation need a separate freeze and C5 authorization. CI success cannot authorize an agent workstation run.

## Evidence, limits and next action

Source: exact-head journal, run_team.rs, parallel.rs, read.rs, ab.rs, leader reporter/tests, team_size.py and sweep workflow. PR API confirmed the requested head. At observation rust, bench and submit checks reported success; miri and preview were still unfinished. No claim of a completely green head.

Local checks: seven repository tests pass; fifteen-round clean control and five adversarial fixtures recorded in probe.json. No Rust build, Miri/fuzz rerun or native decoder timing performed. Routing audit reports only its five resolved historical entries; REVIEW routes to #145.

Expected: determine freeze readiness and separate C30's mechanism from its representation. Observed: fair primary plan/window and a plausible privately checked construction, but false VALID cases and a biased critical-path statistic. No performance gap closed. Next value: repair admission/common-origin trace/control policy and freeze one disclosed primary, then finish applicable safety evidence. Those repairs make a single bounded run interpretable; another smoke timing would not.
