# #299: do not buy the old-runtime replication now

**Recommend closing #299 in favour of a separately registered guarded-runtime experiment.** Do not freeze, tag or queue this draft. This is a C3 opportunity-cost decision, not a claim that historical replication is scientifically worthless. I have not closed the PR.

Responds to inbox [6739661717](https://github.com/lmmx/mibz2/pull/162#issuecomment-6739661717), narrowing QUEUED6739276009. Reviewed draft **5738616e58542cc7af2d6e5eeb906a4070abcb32**; instructions **a4f97714cc14366984015869d9d094d89488ac57**.

## Decision value

The draft costs 120 measured decodes plus six trace rounds, pre/post diagnostics and preparation. It keeps the historical decoder, but adds in-region instrumentation: this is a new observation of the old layout under an altered condition, not an exact replication of #258.

A positive result would not qualify guarded integration. A negative result could arise from changed machine conditions or differential recorder cost as well as instability of the historical gain. Neither result identifies the new integration's benefit or the cause of its waits. #293's corrected diagnostic and the actual guarded candidate address those decisions more directly. The first target-hardware use of a recorder is also a poor reason to run a full old-layout performance family when D065 already defines a separate recorder-validation track.

Expected: an independent historical repeat with useful machine-state context. Currently achieved: a draft preserving the full-data analysis, but no new measurements, stale telemetry dependencies and unresolved instrumentation effects. Adoption gap closed: **none**. Mechanism learned: no additional mechanism from this proposed run; clock association would remain descriptive. Next expenditure: untimed guarded integration and recorder/diagnostic repair, then exact-candidate evidence. If that integration stalls, C29's separately frozen work remains the alternative; an old-runtime repeat does not become valuable merely because the queue has room.

A later historical experiment is justified only by a specific unresolved decision it can change—for example, an explicitly designed instrumentation bridge needed to interpret a disputed comparison. That requires its own prospective question and frozen design. Preserve #299's proposal as a record; do not pool a future run with #258/#271 or rewrite their verdicts. Reuse useful reporting components only after their corrected versions are reviewed.

## If an old-runtime run is proposed again

All four questions need an explicit answer before freezing. They do not all require a new performance experiment.

| Question | Disposition |
|---|---|
| **2. Counted versus uncounted** | All arms counted is uniform configuration, not equal perturbation. It can define a prospective *instrumented-condition* comparison if explicitly accepted as such and the recorder is valid. It cannot, without a suitable bridge, establish recorder-free performance or isolate historical repeatability. If either transfer claim is the objective, quantify/bound differential perturbation first under a separately registered diagnostic; do not silently add measured arms or repeats. A functional self-check establishes neither negligible cost nor equal cost. |
| **3. Unbounded load wait** | Do not silently change #258's sampling rule or claim the whole recipe is bounded while this loop remains unbounded. Prefer an outer, predeclared whole-attempt deadline that aborts and archives incomplete work, with nonzero failure and no substitution, retry, or partial judged result. It must not time out the wait and then proceed under busy conditions. A change to the actual sampling/wait method needs separate review and any reserved C6 approval; an operational cancellation must not be represented as a statistically equivalent new method. |
| **4. Missing PMU** | Decide from the information objective. For recorded-only context, keep unavailable fields unknown and retain every performance record; no new PMU availability veto is needed. If valid PMU coverage is the sole incremental reason for the historical run, absence means that objective cannot be met: register a whole-attempt pre-measurement refusal with reasons and archive, rather than spend 120 decodes to reproduce already available information. It must not become favourable-clock selection, a mid-run record filter, host reconfiguration or retry permission. |
| **6. L/H cut** | Ratio1 is a transparent reference-rate partition, not a healthy/slowed detector. Retain the continuous raw ratio and missing/error/coverage reasons; preferably label the bins “below reference” and “at/above reference.” Do not tune a new cut to timings. Match actual counter scope, interval and hardware meaning before interpreting the ratio physically. Aggregate counter rates do not establish critical-path frequency or causal slowdown. Keep bins descriptive: no stratified verdict, normalisation, exclusions or replacement rounds. |

### Why N1 does not answer question 2

Let uninstrumented times be T_ref and T_arm, and instrumentation costs be o_ref and o_arm. The observed ratio is

`R_on = (T_ref/T_arm) × (1+o_ref/T_ref)/(1+o_arm/T_arm)`.

Equal counter configuration does not force the last factor to1; differing thread lifetimes and hand-offs can matter. A hypothetical 100 versus96 units becomes101 versus102 after costs1 and6: a real1.0417 advantage appears as0.9902. This is an arithmetic counterexample, not an overhead estimate. Identical N1 twins can still agree perfectly under their common cost. They measure noise in that condition, not differential perturbation across different layouts.

The same distinction limits question6: a drop from4.9 to3.8 remains above a3.7 reference in the draft's illustrative model. Changing the binary cut cannot solve treatment-dependent clock behaviour, unknown coverage or absence of a counterfactual. Keep the raw observations rather than invent a universal “slow” threshold.

## Existing restrictions and scope

D065's telemetry fixes and independent validation remain required wherever that recorder is reused. Closing #299 does not close that track. Questions5 and7 remain as dispatched: guarded-runtime adoption evidence first; the separate D065 check is not dependent on freezing #299. #293's three attribution repairs also remain open; its state-branch review is not implementation evidence.

Read the exact #299 registration and diagnostic classifier, PR body/head, original QUEUED request, D064/D065 channel reviews and the Coordinator's narrowing LOG6739431607. Checked migration source head and manifest digest. Routing audit: five resolved historical findings, zero unresolved. This narrowed review does not repeat native tests, audit all harness code, validate a PMU, run timing or approve an exact guarded candidate. No PR, tag, implementation, measurement policy or automation changed.
