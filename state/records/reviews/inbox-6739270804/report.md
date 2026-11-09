# Service core: buy default evidence before another tuning run

Inbox [6739270804](https://github.com/lmmx/mibz2/pull/160#issuecomment-6739270804), dispatching [QUEUED6735266897](https://github.com/lmmx/mibz2/pull/145#issuecomment-6735266897). Governing specs a4f97714cc14366984015869d9d094d89488ac57; briefing baseline a773853f05fa9e12bb65fb93bf5c07e1bc7fea11. Proposals only.

**Recommendation:** prioritise a guarded, current-baseline service-default candidate at the narrow measured budget over a third old-runtime replication. Keep C29's bounded diagnostic moving. Defer J4 qualification until its incremental question and #293's corrected diagnostic justify the extra run; C43 remains behind both.

## 1. The comparison changes the economics

[The #271 result at the briefing pin](https://github.com/lmmx/mibz2/blob/a773853f05fa9e12bb65fb93bf5c07e1bc7fea11/docs/journal/2026-11-09-service-lookahead-result.md) records VALID INCONCLUSIVE, step3. Preserve it. Ratios here are reference/arm; above1 is faster or less CPU:

| Comparison | Wall ratio, 95% interval | CPU ratio | Status |
|---|---|---|---|
| LA3: a3w8 / a2w8 | .992 [.968,1.016] | 1.006 | judged, inconclusive |
| LA4: a4w8 / a2w8 | .979 [.951,1.008] | .983 | judged, inconclusive |
| J4: a4w12 / a4w8 | 1.049 [1.017,1.082] | .992 | reported only |
| J: a4w12 / a2w8 | 1.027 [.993,1.063] | .976 | reported only, inconclusive |
| W12: a2w12 / a2w8 | 1.000 [.972,1.029] | .978 | reported only |
| D-a2w8: service / default | 1.073 [1.041,1.106] | 1.169 | reported only |
| D-a4w12: service / default | 1.102 [1.070,1.135] | 1.140 | reported only |

J4 changes the window at threshold4. It does **not** estimate the joint increment over a2w8. LA4×J4=.979×1.049=1.026971, consistent with J after rounding. This identity holds for paired geometric means over the same rows; it supplies no new independent evidence, and interval endpoints must not be multiplied.

J4's central estimate means about4.67% less wall than a4w8; J means about2.63% less than a2w8. J's interval still permits slower wall. Its CPU point estimate implies about2.46% more CPU than a2w8. Threshold4 alone was centrally slower; the wider window partly recovers that loss. Calling the combined change a demonstrated4.9% gain buys the wrong experiment.

[#258's result](https://github.com/lmmx/mibz2/blob/a773853f05fa9e12bb65fb93bf5c07e1bc7fea11/docs/journal/2026-11-09-service-2x3-result.md) remains the registered reading: P1 1.045 [1.017,1.073], CPU1.146 [1.123,1.170], overall FOR SVC2@10. It implies about4.31% less wall and12.74% less CPU centrally. #271's D-a2w8 direction is supportive reported replication, not a second judged P1, pooled estimate, or guarded-runtime qualification. Its corresponding central reductions are6.80% wall and14.46% CPU.

### Mechanism: overlap remains plausible, the claimed bottleneck move does not

The original prep-idle .138→.057 and backlog2.42/.35 cannot establish PREP STARVATION→FINISH BACKLOG. The corrected journal withdraws the wait-label inference: a predicate observed at wait entry can be false during the wait charged to it. Queue occupancy is stock, not service demand; thread life minus idle is not CPU utilisation. #293's pending diagnostic is the dependency for that story.

The candidate moves prepare/finish work off team leaders so two teams can keep building/walking. Charge the hub, wakes, feeder, buffers, and reader ordering against that exposed overlap. A useful lower-bound model is wall ≥ max(scanner demand, team critical work, service demand/2, ordered reader demand), with precedence and queue waits adding to it. Aggregate service CPU cannot identify which maximum binds. Corrected mutually exclusive wait predicates, phase CPU and ordered critical-block waits would discriminate; another queue average will not.

## 2. Smallest decisive tuning design — proposed, not registered here

**Question:** does a4w12 beat the already useful a2w8 by the fixed minimum, within its CPU/memory conditions? Make J primary. J4 can explain an effect; it cannot substitute for J.

Let A=a2w8, B=a4w8, C=a4w12, D=a2w12. At the existing20-round scale:

| Purpose | Cells | Decoder executions |
|---|---|---:|
| Bare J estimate | A,C | 40 |
| Smallest operational proposal retaining an A/A′ noise control | A,A′,C | 60 |
| Also estimate J4 | A,A′,B,C | 80 |
| Identify the threshold×window interaction | A,A′,B,C,D | 100 |
| Also compare to the actual production default | preceding cells plus default | 120 |

The60-execution proposal answers the adoption increment, not an interaction claim. With four corners, interaction on log wall is the window effect at threshold4 minus its effect at threshold2. Without D, do not claim that interaction. These counts exclude traces, builds and preflight. A registered arm policy may require more; these are design costs, not permission to delete existing cells.

Freeze the exact guarded source, workload, budget10,20 rounds, rotation, paired geometric-mean estimand, intervals, noise/CPU/RSS constraints and outcome map before any attempt. Preserve every pair. A new judged family, changed repeats/grid or acceptance belongs to C6; the table grants none of it. The Researcher assesses, Coordinator registers, Louis runs the frozen recipe under C5. One attempt, no retry until favourable.

**Value judgment:** not the next standalone workstation qualification. First read #293 and settle the production base. If its information supports widening the window, fold the bounded J question into a separately approved candidate design where possible; otherwise keep a2w8. A larger pool is not free insurance against an unidentified stall.

## 3. #299: answer open question5 with the adoption objective

Reviewed [r2 registration at 5738616e58542cc7af2d6e5eeb906a4070abcb32](https://github.com/lmmx/mibz2/blob/5738616e58542cc7af2d6e5eeb906a4070abcb32/docs/journal/2026-11-09-service-2x3-r2-registration.md). It is a DRAFT, not a frozen ready entry. The briefing master pin does not contain this registration; the explicit PR head supplies its separate identity.

It deliberately retains runtime3e201246, six cells×20=120 decoder executions and six svc2 trace rounds. Its stated estimate is about7 minutes built/11 cold, not a time measured or promised by this review. #298's recorder check is an untimed prerequisite. The inherited counters introduce work inside the timed interval; equal instrumentation across arms does not guarantee equal perturbation when thread and hand-off patterns differ.

For **old-runtime replication/recorder investigation**, r2 has a coherent bounded question, subject to Researcher's open questions. For **the next production qualification**, use the guarded current candidate under its own registration. Do not silently rebase and call it the same experiment. D057 expressly withholds old-performance transfer to the guarded integration; M12 unwind changes also belong in the source reconciliation.

[D064/Researcher6738754551](https://github.com/lmmx/mibz2/pull/145#issuecomment-6738754551): recorded telemetry only, unchanged full-data analysis. No clock filtering, pair replacement, strata verdict, frequency-normalised substitute or new veto. A/A′ controls noise under counted conditions, not differential instrumentation bias. If that bias remains material, price one separately registered counted/uncounted diagnostic with both candidate and reference crossed against instrumentation; do not solve it by selecting quiet records. No PMU correctness or overhead was tested here.

## 4. Path to a default, priced by missing evidence

**Narrow proposal:** initially expose service2×3+2 only at requested budget exactly10 on an explicitly documented supported target predicate, L2/window8. Keep other budgets on their current path. The existing topology predicate is10 physical cores/20MiB L3 with requested budget≥10; its reach is wider than one i9 and one budget. Do not inherit that entire reach as if measured.

[Master parallel.rs](https://github.com/lmmx/mibz2/blob/a773853f05fa9e12bb65fb93bf5c07e1bc7fea11/src/parallel.rs) has only Workers/Teams and selects pipelined3×3 in defaulted. It has no Service plan. Physical-core discovery already uses allowed affinity CPUs; quota, offline/unknown topology and broader CPU-model equivalence still need explicit treatment.

| Missing item | Evidence needed | Incremental performance-run price |
|---|---|---|
| Guarded source and B5 | Diff old service kernel to exact proposed integration; reconcile guard/M12 changes and live holds; independent Guardian review with reached differential/fuzz/mutation/Miri and unwind/cancel/reuse obligations | 0 timing campaigns; at least one exact-candidate correctness package plus review; failures cost repairs, not permission to rerun timing |
| Public default reach | Prove StreamDecoder::new, BzReader::parallel/parallel_with and decode_parallel reach service naturally on target; forced-plan tests separately; sequential and explicit-planned paths preserve their contracts | 0 extra timing campaigns if these reach checks precede the target comparison |
| Target adoption | Exact switched public default against exact current default on the qualifying target, including CPU and RSS, frozen paired method | ≥1 C5 campaign, not another old-runtime replication |
| Ordinary adoption/off-path | Normal CI paired bench on unchanged1,2,nproc grid/corpora; full release binary and unchanged paths | ≥1 CI performance package; no substitute for target reach |
| Other requested budgets | Narrow default10: boundary/fallback correctness at neighbouring and larger requests; no claim of service speed there | 0 extra performance campaigns while excluded; exposing each further budget adds cells in an approved grid or a separately frozen campaign |
| Other machines | Preserve fallback and establish dispatch correctness; no universal service claim from i9 | 0 extra service campaigns for excluded machines; ≥1 frozen campaign per additional exposed hardware class, which is evidence for that class, not every CPU sharing cache size |
| RSS/backpressure | High-water live and retained capacities, slow reader, high expansion, cancellation and small/mixed blocks; record target peak RSS in the main comparison | 0 extra timed campaigns if integrated; ≥1 untimed memory stress package. New performance workloads require C6 |
| J4 tuning | Incremental J plus controls, only after a useful diagnostic | Optional +1 campaign (60/80/100/120 decoder executions as above), not a prerequisite for base service |
| Recorder validity/perturbation | #298 real-PMU check before freeze; separate differential-overhead question if needed | 1 untimed check bundle; optional +1 diagnostic timing campaign, never automatic |

**Lower bound for narrow adoption: two distinct performance packages**, target C5 plus normal CI, after correctness/reach/registration. Not two guaranteed passes. If CI never selects service, a clean CI result only prices off-path effects. If its unchanged acceptance rule cannot admit the candidate, that is an unresolved adoption gate; do not reinterpret a null CI interval as a win or replace it with forced-path timing. Any rule change remains Louis's.

The three public entry points need one common dispatch proof plus each wrapper's lifecycle tests. BzReader::parallel_with calls StreamDecoder::new through BzReader::on; its feeder carries CancelOnUnwind. decode_parallel calls new directly and has its own scoped feeding/cancel path. A test that calls service directly proves neither wrapper reach nor default slot scaling. Custom slots must remain respected: the current window function scales by teams/threads. Budget10 with40 requested slots would imply8 for two teams, but changing dispatch must actually implement and test that rule.

D057 lifts D045/D053 for the reviewed3e201246/e64469f6/8a0fc8e runtime and stated input/kernel/source-panic scope. Do not restart the resolved hold as though no safety work exists. Equally, reuse is by exact scope, not by name: demonstrate service branches reached, fence TLS publication before resetting counters, and preserve wake/retire/helper completion/error-ordering witnesses on the integrated source. D057's build/walk counts are marginals with joint lower bounds, not measured intersections. A timed run cannot close these obligations.

### Memory and thread accounting

#258 reports median peak RSS about64MB svc2 versus74MB default. That is corpus evidence, not a worst-case bound or a guarded candidate result. At r2's pinned service source, fronts=T+L+K=6 and backs=2T=4 for T2/L2/K2. A threshold4 version using the same allocation rule has8 fronts, still4 backs; verify that at its own source pin before qualification. L is a scheduling threshold, not a strict prepared-queue cap: concurrent preparers matter.

Window8→12 admits four more blocks, including symbols and potentially greatly expanded outputs. Master's documentation allows approximately1.8MB symbols and46MB decoded bytes per high-expansion block; four ordinary JSON blocks substantially underprice the worst case. Retained Vec capacities, free pools, current reader output and scanner/input buffers must be accounted separately; do not turn these rough per-block numbers into an exact heap ceiling.

Eight reported decoding threads are two leaders+four helpers+two service workers. The service implementation also has scanner and hub feeder; BzReader has its input feeder, and the calling reader exists too. Compare process CPU and full live-thread count under matched APIs. “One fewer decoder thread” does not by itself mean one fewer total process thread.

## 5. Ranking and what would change it

1. **Service core, next adoption work:** strongest directly relevant target evidence (judged P1 plus supportive reported direction) and about4.3% historical central wall reduction. Spend on guarded exact-default closure, not a third unguarded P1. Its cost is architectural safety/integration and narrow hardware coverage.
2. **C29, first bounded kernel investigation:** retain the already specified repaired diagnostic and D060 transfer minimum. Run42582708226 remains VALID/UNJUDGED; .9700 is not LEAD. Its potentially smaller source change may make it cheaper if service integration stalls. Do not interrupt its accepted diagnostic to buy J4; do not add C29 and service percentage estimates as independent savings.
3. **J4:** optional incremental service tuning after #293, with J rather than J4 alone deciding value. Current evidence does not earn priority over adoption closure or C29's bounded diagnostic.
4. **C43:** conditional sort-budget lead, still behind the layout/host inquiry. Prior .318992 nominal c/outputB (6.43878/run) allowance is not a production forecast or hardware-independent cycle budget.

This refines the earlier “C29 first for investigation,” not a retrospective change to its result. If the service candidate cannot cheaply establish guarded reach/B5, C29 moves ahead on marginal cost. If J is convincingly positive at fixed CPU/RSS conditions in a new admitted comparison, widen the tuning line; today that gap is open.

C3 answers: expected useful extra overlap from larger lookahead; judged LA3/LA4 did not establish it. No new gap closed by this audit. We learned that J4's baseline and the trace labels overstate the tuning case; service2's historical whole-decoder case survives within its own scope. Highest next information value is exact guarded default reach and qualification, with #293 and C29's existing diagnostic informing the next pivot.

## Audit scope and provenance

Read the two pinned result journals, exact inbox/queued brief, D045/D053/D057/D064 and Researcher clock review, prior C29 REVIEW6736939946, PR298/299 metadata/change lists, r2 registration, master parallel/read/lib source and r2 service source. Journal blobs: #271 99a79ca669078e0a97bac9775ef12ff44974bb92; #258 9e345a92b27405e4af27fe2ae931ef5bae00d6cc. Service parallel blob f80d87caf23b66a4fc1041fab2b4807062713b5d. Numeric reductions are reciprocal arithmetic on published rounded estimates, not raw-bundle replay. The reported bundle digests were not independently verified in this review.

No implementation, compilation, timing, native tests, merge, B5 certification or admission change. Migration matched old head e6eea5889432aef28f4d9e450cee87dd5bf7bacf and manifest SHA25686d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde. Pinned coord.py audit on connector-read inbox/channel data found five resolved historical findings, zero unresolved; REVIEW routes to145. Its verify hardcodes FROM coordinator; exact optimizer author/body/destination readback is required and reported separately, not mislabelled as a verify pass.
