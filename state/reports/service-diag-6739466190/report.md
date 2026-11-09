# Service-starvation diagnostic review

**REVISE #293 before a C5 trace. Port the corrected diagnostic to the guarded integration; do not spend the next diagnostic run on the old runtime solely to inform adoption.**

Inbox [6739466190](https://github.com/lmmx/mibz2/pull/162#issuecomment-6739466190), dispatching QUEUED 6736622257. Exact reviewed head **e63f33cbe93ba8d55adcbfc17d784c1b39470e3c**, based on service-lookahead 8a0fc8e0. Specs: a4f97714cc14366984015869d9d094d89488ac57. This is research and diagnostic review, not implementation, safety clearance, timing authorisation or production acceptance.

## What advances, what fails

The original discriminator is substantially built: hub ownership snapshots, window snapshots, task transitions, per-block stages and waits split across state changes. The old nine-unit entry-label counterexample and the front-availability enumeration are reproduced by the repository tests. This is useful groundwork.

The output is not yet reliable enough to choose a prep-reserve versus credit/handoff intervention. Three independent counterexamples survive with an empty Python `defects` result:

| Probe | Expected interpretation | Actual |
|---|---|---|
| Independently locked event obtains sequence before a hub event, timestamp after it | Nine-unit wait remains nine, or duration analysis refuses ambiguity | **16** attributed units for endpoint duration **9** |
| Reader releases one credit; scanner has not resumed; later result ready behind next missing block | Scanner wake/rescheduling interval, no full-window block | **5** units classified `window_credit`, with outstanding **2 < W=3** |
| Both finishes selected when no preparation was eligible; a job arrives afterwards | Finish occupancy with late job arrival, not demonstrated finish-over-prep priority | **5** units `finish_first`, **zero** `finish_over_prep` decisions |

Executed evidence: [probes.py](probes.py), [probes.json](probes.json). These are deterministic event-model probes against the exact analyser, not native decoder executions or measured workload frequencies. The simulator enforces both hub ownership identities. The probe histories are intentionally partial attribution histories, like the supplied Python witness tests; I do not claim they pass the Rust complete-log checker.

## R1 — sequence order is not timestamp order

[`diag::record`](https://github.com/lmmx/mibz2/blob/e63f33cbe93ba8d55adcbfc17d784c1b39470e3c/src/diag.rs#L240) takes `SEQ.fetch_add` and then samples `Instant`. A thread may be descheduled between them while another thread records under a different lock. The global sequence respects each lock's order; it does **not** make timestamps monotone in the merged sequence.

[`analyse`](https://github.com/lmmx/mibz2/blob/e63f33cbe93ba8d55adcbfc17d784c1b39470e3c/bench/svc_diag.py#L260) clips negative deltas with `max(0,next.ns−current.ns)`. This hides the defect and adds positive elapsed intervals twice. In the nine-unit witness, insert an independent window event assigned its sequence before the hub's t=1 preparation completion, but timestamped at t=8. The resulting time sequence includes 0→8→1→9; clipped accumulation is 16, not 9. This schedule is permitted by the recorder: the paused thread holds the window lock, while the preparation event uses the hub lock. `diag::check` does not test timestamps; its structural checks do not repair this.

Required: define one defensible event-time/linearisation contract, preserve each lock's and each thread's causal order, and test cross-lock descheduling. Reject or explicitly mark duration attribution unavailable when that contract fails; never silently clamp. Assert per-wait sum equals endpoint length and scanner partitions equal the analysed span. Counts can remain useful under a separately stated sequence-order interpretation, but “constant state between timestamps” needs its own proof. A global logging lock is not automatically free: if chosen, its perturbation becomes part of the diagnostic condition. Do not cure this by sorting one column without proving the resulting order preserves the needed causal relations.

## R2 — outstanding credit must be tested now

[`classify`](https://github.com/lmmx/mibz2/blob/e63f33cbe93ba8d55adcbfc17d784c1b39470e3c/bench/svc_diag.py#L128) trusts `scanner == slot_wait`, which lasts until the scanner records `SlotGot`. The reader can meanwhile execute `Taken`, decrement outstanding occupancy, notify the scanner, and continue. A scanner not yet scheduled is not still resource-blocked merely because its last program event was a wait.

The probe reaches a full W=3 with blocks 2,3,4 outstanding, results 2 and4 ready, and3 finishing. The reader takes2: next3 is missing and4 is ready, but only two blocks remain outstanding. For five subsequent units before `SlotGot`, the analyser reports `window_credit`.

Required: classify window-credit blockage only while `issued−taken >= slots` (and legal noncancelled state), with later-ready/earliest-missing evidence. Once credit exists, split scanner wake/handoff from resource blockage. Recompute scanner “slot waiting” occupancy with the same distinction. The current 0.78 scanner-wait share is a wait-episode residence share, not proof the window was full for 78% of the span. A positive-ready head also does not establish reader lag without current occupancy and reader-state qualifications.

## R3 — occupancy does not prove finish-priority causation

The predicate P=0,A=0,J>0,C=K identifies service workers occupied by finishes while jobs await preparation. It does not establish that those finishes displaced an eligible preparation.

The third probe lets each worker choose a finish when J=0, then admits a new job. The resulting wait satisfies `finish_first` for five units with no finish-over-prep choice at all. A reserve policy cannot be credited with eliminating this gap merely by changing a choice that never existed.

Required: name the base class descriptively (all services in finish with queued jobs), then separately reconstruct the active workers' decision provenance: which finishes began with eligible prep, what prepared stock existed then, when the last stock was consumed, and whether credits/fronts permitted the proposed extra preparation. The event stream already contains much of this evidence; connect it per wait instead of comparing an aggregate 560 decisions with three waits. Even a proven antecedent supports a counterfactual candidate, not a measured recoverable time or guaranteed gain.

Likewise:
- `prep_in_progress` means A>0, not A=K. It cannot by itself trigger the draft's “both services prepare” branch. Split mixed A/C and free-back status.
- P>0/B=0 establishes back unavailability, not that all held backs are actively finishing; inspect U,Q,C.
- `feeder_lag` covers Issued→JobIn, including scanner send, channel residence and feeder execution. Use Enqueued/FeedRecv to distinguish them before proposing a feeder optimisation.
- Keep startup/drain as phase labels alongside mechanism state. Their first-match precedence otherwise hides resource states in those phases. Report the excluded mass; do not infer that a low steady-state class bounds total opportunity.

## R4 — bind admission to the actual analysed logs

The executable compares output and calls Rust `check`, which is valuable. The offline analyser is much weaker: an incomplete Config/ReadEnd log produces no defects; pooling W8 and W12 results silently labels the aggregate with the first plan. It does not establish stage completeness or timestamp validity.

A frozen recipe must either revalidate the transported TSV fully, or bind it by digest to the executable's successful validation and exact plan/corpus/output metadata, then enforce the additional analyser invariants. Group heterogeneous arms explicitly; refuse accidental pooled configurations. Preserve malformed, truncated and cancelled logs as failed/incomplete evidence, not successfully classified exposure. The Rust checker checks aggregate exit counts, not a unique exit for each thread identity; its claim that “each thread ends once” is stronger than that check. Strengthen the checks needed by attribution, rather than relying on the name `check`.

## Does the local picture change candidate priority?

It reduces the case for immediately buying a prep-reserve change. It does **not** settle the i9's mechanism, and the newly found errors prevent treating the printed local shares as trustworthy quantitative attribution.

Expected: distinguish finish occupancy, credit exhaustion, preparation and handoff. Observed in authored local records: both named classes appear small, while handoff/Issued→JobIn residence is larger. Independent review closes the conceptual entry-label gap but exposes the three inference defects above; no fraction of target wall-time opportunity is established. Oversubscribed local schedules and the global sequence counter, timestamp calls and buffer allocation can all alter scheduling. “No scheduling rule changed” is not “schedule unperturbed.”

Retain prep reserve as conditional, behind corrected guarded-runtime observation. Larger prepared stock can mask a finite finish interval, but it cannot itself release ordered-output credits; preferential preparation can also delay the earliest finish and worsen credits/back availability. The next valuable step is correcting the classifier and replaying these negative controls, then instrumenting the runtime being adopted. No new old-runtime performance replication is required for that step. C29's separately frozen work is not held by this review.

## Runtime and C5 registration

**Port to the guarded current integration, after its shape and lifetime rules are fixed, and before its separately authorised diagnostic run.** #293 is based on8a0fc8e0, not the proposed guarded current default. A trace there answers that old runtime's schedule. Using3e201246 instead would move further away from the adoption target. Keep the old branch as historical reference; an old-runtime trace needs its own distinct decision value, not inherited priority.

No concrete guarded candidate is supplied by this assignment, so none is approved here. Reuse reviewed ownership evidence only within its actual scope. Prove the exact default/opt-in entry path reaches the intended 2×3+2, L2/W8 implementation; hidden `StreamDecoder::service` alone is not proof that a new default dispatches there. Verify feature-off code removal and exact candidate CI/B5 before queue admission.

The frozen diagnostic must state:

1. **Information objective and branching rules.** Observed state versus causal antecedent, per-wait provenance, treatment of simultaneous deficits and phase exclusions; define “recurrent” and the minimum coverage in advance. Allow multiple live explanations and an explicit unresolved outcome.
2. **Identity.** Source/registration commits, actual binary path and SHA-256, compiler/build/features, analyser digest/schema, runtime dispatch/configuration, exact input and expected output digest/bytes/block count. Record actual machine/affinity/quota and input/consumer feeding behaviour. The existing example uses a four-thread segmented reference as its byte oracle; identify it accurately rather than assuming this is the actual default entry.
3. **Bounded design.** Fixed arms, exact log count/order, timeout and maximum retained data; archive every attempted log and failed check. “Several” is not a freeze. The six-arm lookahead sweep is not automatically necessary: first establish the guarded L2/W8 baseline mechanism; add threshold/window controls only for a stated question, within existing authorisation and workload/grid constraints. This is a proposed scope, not permission to alter reserved C6 choices.
4. **Admission.** Output exactness, complete stage/ownership/task lifecycle, window occupancy and ready-set consistency, timestamp contract, partition conservation, no unexplained records, homogeneous grouping. Bind stdout/exit status/check output to the exact TSV/JSON. Test failure propagation through packaging/submission. No retry-to-pass.
5. **Estimand and limits.** Counts and explicitly defined instrumented residence exposures, not speedup or recovered CPU time. Report per-log results as well as a defined aggregate. Overlapping team waits cannot be added to wall-time saving. The global counter makes this an instrumented schedule; neither repeated agreement nor a zero defect count proves absence of perturbation.
6. **Separation and execution.** No `svc-diag` in timed arms. A diagnostic run is separately registered/frozen and Louis-executed under C5; output cannot amend #258/#271 verdicts or qualify a production default. Untimed smoke/negative controls and exact-source readiness precede it.

## Validation and scope

- Fresh migration SHA and manifest digest pass; cursor receipt committed first.
- Fresh routing audit: five resolved historical findings, zero unresolved.
- Exact #293 Python suite: **10 tests pass**.
- Independent probes: timestamp overcount, released-credit misclassification and absent-priority-choice witness reproduced; incomplete-log and mixed-plan admission weaknesses reproduced.
- Read `src/diag.rs`, the relevant hub/window/scanner/service/team hooks in `src/parallel.rs`, example, Rust tests, Cargo gating, analyser, tests and draft journal.
- Rust toolchain unavailable (`rustc: command not found`); no fresh native/Rust/Miri/CI execution claimed. Authored local logs were not supplied as raw artifacts here; their shares were not independently replayed.
- The saved analyser and test module are exact reviewed sources for reproducing the probes. No production files changed, no benchmark or workstation trace run, no tag/merge/authority changes.
