# Service-lookahead admission: HOLD, independently of D045

Inbox **6730378782**, dispatching **6730279906**. Reviewed #271 at **e64469f614f5d635a38de042741bdd344e61ae5c**; frozen inputs **79e414197ba956d19ba6b660927f8eca6c3ed520**. Specs/MODE/RULES/PROTOCOL: **a4f97714cc14366984015869d9d094d89488ac57**.

**HOLD queue admission at this head.** D045 already holds service-core timing pending reached-path B5 evidence. Even conditional on its closure, repair the trace evidence predicate and narrow the mechanical interpretation below. The eight-cell diagnostic is worthwhile; approximately eight minutes warm is a reasonable planning estimate, not a deadline or permission to run. No changed numerical outcome threshold, N1 rule or extra timed arm is requested.

## 1. Accepted design, freeze and evidence

- The live PR head is the requested SHA. Comparing freeze to head changes only `bench/service-lookahead.frozen`; its target is correct. Comparing with #258's measured base shows the parameter wrapper, `admission_for`, forwarding API and test, without a replacement ownership/retirement implementation. Existing calls pass None, preserving L=teams. This is source-level behavioral equivalence of the default parameterization, not proof of identical binary code or B5 closure.
- `ab.rs` implements the named L=2,3,4/window=8,12 cells correctly at ten threads: two teams of three plus two service workers. The L=2 arms use the existing API. Occupancy's slots 40/60 become windows 8/12 and its explicit admissions match. Whole decoder timing includes construction and buffer costs; this is not a walk-only proxy. Do not divide results by the nominal ten-thread budget: service cells have eight decode workers, plus the disclosed feeder/scanner/consumer threads.
- The recipe checks tracked cleanliness, untracked measured inputs and the frozen measured tree, including source/build/config/toolchain/harness/recipe/report inputs. It preserves raw AB records, corpus digest and trace JSON/text; each attempt has its own directory and a failure outcome. The imported output, finite-positive numeric, corpus, exact cell/round and N1 checks reject the original #258 mutations.
- **42 repository fixtures pass.** ResourceWarnings about unclosed files were emitted but did not alter results. The evidence JSON records GitHub's exact returned check list, distinguishing successful checks from skipped publishing/deployment jobs. No CI result here is a fresh performance verdict.
- The PR describes an expected-INVALID dry run with matching bytes and P maxima 3/4/5. I read that report; I did not rerun a decoder or independently inspect a dry-run bundle. Its four-core observations are correctness/configuration evidence only.

Prior art is #258's same service core and D045's admission-threshold correction, not a new queueing algorithm. Varying L directly is a better intervention on the stated question than varying only window. #258's earlier acceptance remains intact; this review does not re-adjudicate that received result.

## 2. H1: validate the prepared counts actually used in the interpretation

`service_lookahead.py:load_trace` checks `prep_max` and `prep_median` only for finite nonnegative values. It never checks L+K−1 or median≤max. Executed counterexamples, retaining an otherwise valid 160-row fixture, all return **VALID, exit 0**:

| Mutation of trace summary | Observed |
|---|---|
| prepared max = 99 at all L | VALID |
| prepared median = 3, max = 2 | VALID |
| blocks = 0, with registered decoded size and other statistics retained | VALID |
| raw trace `runs` empty | VALID |

The repository's own `test_full_run_threshold_binds` assigns max=5 to both L=3 and L=4. Five exceeds L=3's ceiling four, yet its expected result is VALID. This is a defect in the positive test fixture as well as the predicate.

For each complete round, require the registered workload's block count, finite summary values and `1 <= median_at_push <= max_at_push <= L+K−1` when prepared events exist; validate event counts and recompute from the raw trace where available. An impossible count is conflicting evidence about the parameter's operation, not a successful admission demonstration. An observed breach requires investigation of source/trace assumptions, not silently clamping it.

**Be exact about aggregation.** Occupancy computes per-round max and push-median, then takes their medians across complete rounds (`occupancy.rs:1609–1627`). The reported `prep_max` is therefore a **median of round maxima**, not the maximum of all events across the pass. The reported median is a median of round medians, not necessarily the pooled event median. It may be half-integral with six rounds; do not impose an incorrect integer constraint on the aggregate. Retain actual per-round maxima or label the aggregates correctly, and enforce the ceiling before aggregation so one violating round cannot disappear into a median.

**Do not invent a six-complete-round gate.** The recipe requests six, while the inherited validity rule accepts at least one complete round. My #258 re-review explicitly accepted that distinction. The one-complete-round witness is an observation of that rule, not a new HOLD. Likewise changing settings.rounds to 1 is currently accepted; report requested/completed counts and the loss of precision without pretending the existing rule promised six complete traces. Raw trace inventory/status validation below is distinct from demanding six trace-complete rounds: all scheduled children must be accounted for, while some traces may remain incomplete under the existing rule.

## 3. H2: preserve child status; a trace summary alone is insufficient

The recipe correctly omits a top-level occupancy pass that exits nonzero, causing its required trace to be missing. But the occupancy **parent** has a gap: at `occupancy.rs:1525` it logs a nonzero child's exit and continues. The raw JSON entry retains round/threads/slots/placement/mode/stdout, **not the child's status**. Final parity checks stdout hash/output/err; parent exit at the end depends on parity, not the recorded OS status.

Thus a child that emits an otherwise acceptable line and then fails can retain parity, produce complete summaries and leave the parent successful. This is source analysis, not a reproduced OS crash. The reviewer cannot recover the lost status from the submitted JSON. Ordinary missing stdout/errors often fail parity; that does not close this different path.

Persist every occupancy child's exit code/signal and exact expected round×mode inventory. Propagate any nonzero status into pass validity and require it in the report. Verify the summaries against those raw records; the empty-runs witness shows they are currently ignored. Keep failing evidence in the bundle. Separately, the shell currently trusts a JSON `verdict=VALID` even if the Python report returned nonzero; require status zero as well before writing a VALID outcome. These are evidence-contract repairs, not numerical gate changes.

## 4. H3: distinguish a policy effect from a queue-supply mechanism

The intervention changes L and its derived front-pool size. A gain identifies the effect of that implemented policy on this workload/machine, not uniquely that the prepared queue was the binding resource. The registration acknowledges the two effects at the end; the bold mechanical “THE ADMISSION THRESHOLD BINDS ...” should explicitly mean **the implemented higher-L policy improves measured performance**, with queue-supply mediation left unproved. CPU INCONCLUSIVE remains no detected CPU cost, not absence of cost. The two judged contrasts' nominal 95% intervals do not provide a familywise 95% discovery claim for “either”; preserve the registered method and avoid that stronger claim. N1 is unchanged and reserved under C6.

**The trace association mixes arms.** `mechanism()` looks for a gain anywhere and starvation relief at either higher L. Executed fixture: LA3 gains (wall 0.95 vs 1.0), LA3 remains starved; LA4 is NULL but is the sole arm with starvation relieved. The report still prints “PREP STARVATION ... RELIEVED AT A HIGHER THRESHOLD” beside the gaining reading. Both facts can be separately true, but their conjunction is not a concordant mechanism observation. Name the gaining and relieved arms and require their intersection before describing aligned evidence. Even aligned observations remain consistency, not mediation proof.

**A sharper bound on the front-pool confound.** Let T be teams, K service workers, P prepared blocks, A other workers currently holding a preparation front, and U fronts held by teams. The pool is F=T+L+K. At an idle worker's decision with P<L, A≤K−1 and U≤T. Fronts reside only in those categories or the free pool; walked/finishing blocks retain back sets, while the team's front is returned at walk completion. Therefore:

`free_fronts = F - P - A - U >= T+L+K - (L-1) - (K-1) - T = 2`.

Under those normal conservation premises, a front cannot be unavailable when rule 3 otherwise permits preparation; rule 1 has P=0 and the same result. A 5,040-state arithmetic enumeration checks this inequality, not Rust ownership or scheduler correctness. Increasing F with L is consequently not a second *independent admission bottleneck* in this model. It can still change buffer reuse, allocation history and resident memory/cache traffic. This makes the policy contrast useful without requiring an additional fixed-F arm now, but does not identify which machine cost produced a gain.

**Repair the model, not the performance thresholds.** The observed 0.165 idle-prep share is summed leader idle divided by T×trace span, not wall time proven removable from the critical path. `1/(1−0.165)≈1.20` is a balanced-utilization model, not a hard upper speed bound. Critical-worker placement, overlap and changed costs can invalidate it. A 0.70 maximum service utilization does not mean the remaining 30% was simultaneously supplied with jobs/fronts and blocked solely by L. A median job wait of 0.158 ms also does not prove jobs were available during the teams' starvation intervals. Preserve the useful predictions but label these as hypotheses; correlate eligible-but-threshold-blocked intervals with subsequent team starvation if that mechanism becomes the next question.

The push-count median samples arrivals, not time-weighted queue occupancy. Reaching L+1—even in every dry-run arm—demonstrates reachability, not frequent binding or recoverable throughput. Trace overhead can alter scheduling; use the occupancy off/timeline/full comparisons when interpreting these trace mechanisms, not as another invented production gate.

## 5. Budget, disposition and continuation

The arithmetic budget is reasonable: 8×20×2.17 s = **347.2 s (5.79 min)** for AB children at the previous empirical cadence. Occupancy adds five passes×six rounds×three modes = **90 child decodes**, not 30, plus per-pass setup/self-test. Eight minutes once built leaves about 133 s for that and corpus/report overhead. It is a plausible estimate, not a proven upper bound; cold builds and load checks can extend it. Same-plan comparisons do **not** guarantee smaller variance than #258's different-plan comparison; treat the interval-width forecast as expectation only. Keep the bounded one-attempt recipe and existing preregistered rerun rules. Louis's own submission is still required; I ran no timing and queued nothing.

Fix H1–H3 and record the correction at a newly identified source/freeze before admission. Keep the existing numerical thresholds and N1, explicitly record any additional evidence checks as stricter admission constraints under C2, and do not rewrite an outcome from the earlier frozen-but-unrun version. D045 must separately close for the changed service configurations; neither green CI nor this C1 review substitutes for it. D042 remains separately scoped.

Expected: a direct threshold intervention with rigorous existing record checks. Found: the parameterization/freeze is sound in scope, but impossible P evidence and lost child statuses survive, and the trace reading can join effects from different arms. Gap closed: no measured decoder gap; a concrete front-pool bound and exact evidence failures are now established. Next value: repair those narrow points, then one registered run after D045 closure. Additional exploratory redesign is not required to make this diagnostic informative.

Evidence: `witnesses.py/json`, `proofs.py/json`, pinned report/tests/helpers, `evidence.json`, `audit.json`. Audit contains only five previously resolved historical misroutes. No decoder implementation, compilation, native timing, Miri/fuzz execution, dispatch, queue/tag, merge, release or automation change. Research commits only to `team-state/researcher`.
