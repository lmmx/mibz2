# Helper-side run-fed construction: ranking and counter corrections

Brief [6729932062](https://github.com/lmmx/mibz2/pull/160#issuecomment-6729932062).
Source **f94551ab8c359ec28dca02956ddca150595cc7cb** (#270).
Specs **a4f97714cc14366984015869d9d094d89488ac57**.
Source/algebra review only: no implementation, compilation, tests, timing or benchmark dispatch.

## Decision

**C31 first among helper constructions; C29 remains the cheapest independent production-shaped lead.** Next correct ownership-preserving imbalance, then compare exact tails. Do not remove the whole-table build barrier to start the current walk early. Leader build assistance is plausible only with measured arrival slack and claimable bounded work, not a permanently assigned byte class.

Three corrections materially change the premise:
1. C31 now needs at most one predecessor search per active byte per helper, not per run. My preceding report's r-search budget is superseded for this implementation.
2. FusedBuild is already a bucket-stream builder with a 16-equal-byte fast path. “45k scatters removed” means F-order metadata scatters against hs, not 45k production table writes eliminated.
3. W/n=1.59 means repeated stored words, not 59% extra store instructions or DRAM traffic. Masked tails can retain almost the same vector-store instruction count.

No measured whole-decoder gain is established by the 4 MB/two-round smoke. Its helper-readiness delay is a host/schedule observation, not an i9 bottleneck result.

## 1. Source audit and price of C31

In src/parallel/run_team.rs, count_rows checks increasing L starts and accumulates row totals only. merge_rows supplies each helper's byte window. fill_direct_at processes L-order runs with hist[1][c] as the next F row and lcur[c] as the saved L-run cursor.

For helper h, byte c:
a_h,c = C(c) + sum(g<h) rows_g,c.
Successive runs of c advance a_h,c consecutively, even though runs of other bytes intervene. The required low-byte position is q in this same increasing F window. Therefore lcur[c] can resume monotonically: at most active(h,c) predecessor searches, bounded by 256h, then sequential L-run advancement within each window. Windows tile [0,n); total interior L-boundary crossings are O(r+256h), not O(256hr). The same table formula holds:
T[q] = L[q] | ((p+q-a)<<8).

This vindicates direct placement more strongly than my earlier coarse-directory proposal. Do not add that directory unless an actual lookup profile warrants it. The reported 2,614/16≈163 searches per selected block is plausible under this bound but remains the journal's census, not independently reproduced here.

**Against hs:** remove its run-count histogram, F-order scatter, 8(r+1)-byte F array and scatter gate; add per-byte cursor state and bucket-stream fill. Against production FusedBuild: remove L materialization/checkpoints upstream and row-by-row source processing, but add run validation/count/prefix, cell handling and overstores. FusedBuild::share already advances one output stream per byte and recognizes 16 identical input bytes. Its row windows come from checkpoints; direct placement obtains analogous disjoint byte windows from record-range prefixes. C31's store topology is not uniquely worse than FusedBuild's, although its temporal interleaving and burst lengths differ.

Per-helper cost model:
D_h = count_h + prefix/wait_h + lookup_h + cell_h + stores_h + checks_h.
Build readiness is setup + max_h D_h + finish/RoomWalk setup, not the sum of helper phase timers. Compare that with FusedBuild setup + max_h fused_h + finish. Measure r_h, n_h, m_h, W_h, active bytes and distinct touched lines alongside the elapsed spans. Total CPU and readiness are separate objectives.

Ownership: each helper's byte window is disjoint. cell may overwrite future cells only within that helper's same byte window; later c-runs advance monotonically and repair them. Its end bound is the ownership endpoint, not merely the current run endpoint. Different helper windows may share a cache line without sharing words: race freedom does not eliminate coherence cost. No independent B5 certification is claimed here.

**Hidden load imbalance:** share(r,h,j) divides record count, whereas row output can be very uneven. Equal r_h can coexist with n_0≈n and n_1≈0. Ideal two-helper row work can then approach twice the balanced span. C31 can lose despite fewer total instructions. Cheapest falsifier: a per-helper untimed census and readiness trace distinguishing useful work from waits. Correct this before declaring bucket streams the cause.

Propose a weighted contiguous-record cut using row and record/cell estimates, retaining the byte-prefix ownership proof. Virtual run splitting can bound a long-run tail but changes counts/ownership metadata. Initially compare the existing equal-record cut with a row-aware cut; do not immediately add a work-stealing scheduler.

## 2. Ranked next moves

| Priority | Construction / mechanism | Conditional saving on the binding path | Risk and cheapest CI falsifier |
|---|---|---|---|
| 1 | C31 direct helper construction, with per-helper census | Removes hs metadata work/gate; may beat FusedBuild by processing runs rather than all source rows | No magnitude established. Compare direct versus hs AND reg/FusedBuild; census imbalance and physical store footprint first. A matched construction+walk diagnostic must include checks and barriers |
| 2 | C35 row-aware C31 partitioning | Removes straggler excess max(D_h)−balanced span, with little change in total row work | More prefix metadata or splits; use census to bound recoverable skew, then exact-table/ownership tests before a separately registered contrast |
| 3 | C32 exact tails / multi-cell batching | Removes .59n repeated word writes on the smoke shape: 37.1% of old written words | May not remove vector-store uops; scalar tails can add several stores/branches. Census tail lengths and inspect existing listings, then compare masked, scalar-tail and bounded-overstore variants under one predeclared diagnostic |
| 4 | C36 opportunistic leader build assistance | Converts part of genuine build wait into useful construction; model below | Late leader can delay publication; atomic claims, extra small tasks and prefix work can erase gain. Census available tasks at actual leader arrival before implementation |
| Independent | C29 register-list checkpointed MTF | Removes old list-kernel work without a new table or relay; helps CPU and exposed front work | Must preserve all checkpoints/errors/fallbacks. Matched control covers every block, not only rule-selected ones. Differential/listing evidence first; whole-entry paired evidence for adoption |
| Last | Early walk over partially built table | At best overlaps some walking with remaining build | Arbitrary Ψ jumps and preinstalled markers break readiness locality. Reachability/marker proof and useful-ready-prefix census first; no barrier removal from current RoomWalk |

### C32: 1.59 is not a cycle multiplier

Let cell length ell=8a+t, 0<=t<8. A bounded vector body plus a masked residual still uses a+(t>0) vector store instructions, normally the same count as rounding the cell upward. It reduces active lanes/bytes, not necessarily issue slots, addresses, or store-buffer entries. A scalar residual uses t stores instead of one vector store. Conversely, packing adjacent cells into complete vectors can reduce vector-store count toward ceil(n/8), but adds lane assembly and dependencies.

If W=1.59n, exact words save (1.59−1)/1.59=37.1% of written words. A best-case store-byte-limited fill could approach a 1.59 speed factor; this is not a measured prediction and excludes added masking cost. If store-uop issue or cell dependencies bind, that argument disappears. Use both words and store instructions/uops in the ledger. Cross-line stores and false sharing also mean “only L1 bandwidth” is too strong without line/coherence evidence.

Keep the existing ownership bounds, especially at byte/helper boundaries. The low-risk first variant is exact tails within the same partitions, not changing stores and partitioning and scheduling together.

### C36: use slack without making the leader mandatory

For an idealized two-helper build with total 2B units, helpers starting at zero and leader arriving at L<B, all at unit rate, completion with divisible residual work is:

T_help = (2B+L)/3; gain = B−T_help = (B−L)/3.

Even with perfect scheduling, converting a build wait does not save the entire wait: it saves a third in this model. Setup, nonparallel prefix, claims and tail imbalance reduce the gain. The disclosed 1.2–1.5 ms wait would imply 0.4–0.5 ms only under those assumptions on that same host, not on the i9.

Prefix is currently performed by the last counting helper. Reserving it for the leader creates a dependence on leader arrival and can worsen readiness. A whole byte class is unbounded work and its runs are dispersed in the L record list. Better: post-validation, bounded disjoint interval tasks which helpers can finish without the leader; leader claims residual work only after its mandatory previous-output/next-front work. For initial falsification, record residual task weights at arrival; if no sizable claimable tail exists, stop this variant before a scheduler rewrite. Every task requires unique ownership and completion accounting, and borrowed fronts remain live until all consumers finish.

### Early walking: exact ranges are insufficient

A lane's next address is Ψ(q), which can lie in an unfinished range. Current end_build first finishes/validates the WHOLE T, sets its length, then constructs RoomWalk; markers are installed into that initialized table. Publishing one region does not authorize readers to use the whole Vec, and later overstores could destroy markers or race with readers. A correct redesign needs per-region readiness, no subsequent writes to published words, marker ownership, and a scheduler that can continue useful build work when lanes block. It also needs panic/cancellation termination independent of waiting walkers. That is a new concurrent representation, not a fused call site.

The maximum recoverable overlap is limited by useful ready-region walk work before full readiness; absent locality it can be almost zero. Count that first. Retain the global barrier in the present construction.

## 3. Counter interpretation: fix before diagnosing the machine

In run_team::invert, t0 starts BEFORE FusedBuild::new/RunFill::new and pool launch. build_ns runs from t0 through end_build and RoomWalk setup. lead_ns measures only lead() after pool entry. Yet leader_late compares lead_ns > build_ns. These durations have different origins. The real completion comparison is (leader_start−t0)+lead_ns > build_ns. Current leader_late can undercount late completion; it is not a reliable per-run ordering statistic.

join_wait_ns does measure a real built-gate wait after lead(), but includes descheduling/wake latency, not just CPU construction. helper_sort_ns explicitly includes waits and is summed; build_ns includes setup and walk initialization. None alone establishes a useful-compute bottleneck.

Propose timestamps on one origin (dispatch, leader start/end, each count/fill end, walk-ready); separately retain thread CPU or matched instruction counts. Nine decoding threads plus scanner on four vCPUs measure wall and CPU under contention. Wall is not “total work” and CPU is not invariant instruction work: cache, spinning, scheduling and frequency intervene. CI can falsify correctness and gross work regressions; it cannot certify the i9 path from this smoke.

## 4. C29 and whole-decoder economics

The journal's local selected-block MTF numbers imply .096 ms saved by register-list control (.224−.128), and only .014–.032 ms additionally by records (.128−.114 to .128−.096). These are arithmetic on indicative local measurements, not evidence of a decoded-file gain. When the leader waits before walking, a shorter front mostly increases waiting; it can still reduce CPU, startup/drain time or a late-block tail. At levels 1–2 or other plans, first establish exposed front time; do not assume criticality from level alone. Plain-worker mtf already uses the register kernel, so replacing only mtf_checkpoints cannot improve that path.

Fresh #145 now reports a service-2x3 workstation submission 6729877078 and Coordinator result 6729896575: speed1.045 [1.017,1.073], CPU ref/arm1.146. These are attributed reports, not a replay here. The correct implied CPU reduction is 1−1/1.146≈12.7%, not14.6%. My prior “#258 admission HOLD” description is historical; do not treat it as current after the reported run. Service safety/adoption work remains separate. Resolving that already measured lead and isolated C29 remains economically preferable to an early-walk architecture; C31 is the best bounded run-fed build rival.

Guardian D044 (6729932417) HOLDS #269's CI timing diagnostic pending B5 closure; Researcher6729947501's freeze recommendation does not lift it. #270's new parallel unsafe paths need their own physical-write-footprint and B5 review. This report neither dispatches a diagnostic nor lifts a HOLD. The pinned PROTOCOL has no §11; the brief's technical direction is within scope, without importing an unpinned governance change.

## 5. C3 read-back and provenance

Expected: helper construction might convert cheaper fronts into wall savings. Found: C31 amortizes lookup more effectively than previously priced, but production already uses bucket runs; record-balanced work and mismatched timestamp origins can obscure the actual bottleneck. Gap closed: cost/ownership model and a concrete counter correction; zero measured speed credit. Next value: C31 partition census and matched construction comparisons, then exact tails. Leader participation only if residual work exists; early walking deferred.

Primary source files at the reviewed commit: src/parallel/run_team.rs (count_rows, merge_rows, fill_direct_at, cell, invert, Relay::join/end_build), src/coop.rs (FusedBuild::new/share/finish), src/run_stage.rs, docs/journal/2026-11-09-run-fed-leader.md. Tests/results in the journal were read, not rerun. Migration head and exact manifest digest matched before receipt. Pinned routing audit found five resolved historical incidents, no new misroute. Historical outcomes preserved; no source or automation changes.
