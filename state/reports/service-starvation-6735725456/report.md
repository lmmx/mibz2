# Service starvation: distinguish supply, dispatch and retirement before buying another worker

Assignment [6735725456](https://github.com/lmmx/mibz2/pull/162#issuecomment-6735725456), dispatching 6735267366. Researcher; 2026-11-09. Spec pin: a4f97714cc14366984015869d9d094d89488ac57.

**Disposition: PROCEED with a bounded, untimed eligibility/credit diagnostic. The strongest adjacent scheduling candidate is an in-flight-aware prep reserve within the existing two-worker pool, conditional on finding finish-induced prep gaps. Do not select a third service thread from 66–75% busy, or treat the reported prep-idle share as a causal partition.** #271 remains VALID / INCONCLUSIVE. This review proposes research; it neither implements nor authorises a workstation run.

## 1. What the evidence actually establishes

Read independently:
- [Result, merged c0a3b0d34210873b6389f8991dea2a4e94e99389](https://github.com/lmmx/mibz2/blob/c0a3b0d34210873b6389f8991dea2a4e94e99389/docs/journal/2026-11-09-service-lookahead-result.md).
- Exact runnable source **8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d**: [parallel.rs](https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/src/parallel.rs), [occupancy.rs](https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/examples/occupancy.rs), [registration](https://github.com/lmmx/mibz2/blob/8a0fc8e099f932c14cd746c4c5d81aaef8dcc75d/docs/journal/2026-11-09-service-lookahead-registration.md).
- Bundle identity from the result and evidence tree: service-lookahead/20261109T075939Z.tar.gz at 705319fd507d79ad545a593b5affa329275d1e00; recorded SHA-256 4df66705a99a1ff7dda4fcea76de655f306a81ddae8261dca077f9f39b84a6c0. The connector could not decode the binary blob (UTF-8 error). **I did not independently recompute that digest or reanalyse the raw trace rounds.** Numerical observations below are the published result; source findings and abstract witnesses are independent.

LA3 .992 [.968,1.016] and LA4 .979 [.951,1.008] establish no gain from raising L at W=8. Reported prep-idle shares .158/.133/.138 and push-sampled P medians 1/1.5/1.5 do not identify why teams wait. The a4,w12 .057 share and FINISH BACKLOG label are a lead, not proof that the physical bottleneck moved. Its mean prep/finish backlogs 2.42/.35 are queue populations, not stage costs. J4 remains post hoc/unjudged.

Three limits in the existing observables matter:

1. **Prep/back attribution is fixed at wait entry.** parallel.rs:1512–1545 sets TeamIdleStart.len from whether P is nonempty, once, before the condvar loop. occupancy.rs:637–650 assigns the entire paired interval to that label. If P=B=0 at entry, preparation arrives, and a back is unavailable until later, the later back wait is still charged to prep. The converse can occur after another team consumes a prepared block. This is a limitation of causal interpretation, not a retrospective change to the registered metric or verdict.
2. **Busy is not CPU utilisation.** occupancy.rs:486–488,576–587 computes (last mark − first mark − marked idle)/whole trace span. Lock acquisition, descheduling and unmarked handoff can be in its complement; startup/tail are in the denominator. Neither “70% CPU demand” nor “30% spare capacity when a team is hungry” follows.
3. **P median samples pushes.** It says nothing directly about the fraction of time P=0. Equal worker busy marginals also leave their joint overlap with team waits unidentified.

The untimed witness script demonstrates all relevant logical distinctions. In a constructed nine-unit wait, the current metric records nine prep units although P is absent for only one and P>0,B=0 for eight. Two workers each busy 70% can overlap a selected starvation interval either fully or not at all. These are counterexamples to inference, not reconstructions of #271.

## 2. Source rules out two proposed explanations as stated

**Preparation is already shared across both service workers.** Each executes service_worker → prepare → front_stage, outside the hub lock (parallel.rs:1371–1458). front_stage runs mtf_checkpoints and the primary-index check (:713–719). There is no single prep-owner thread to split. Parallelising one block's MTF/checkpoint computation would be a different algorithm, with dependency and safety obligations, not this scheduling change.

**A missing front cannot prevent an otherwise eligible prep decision in the intended ownership model.** Let T teams, K services, threshold L, P queued prepared fronts, A other preparations, U walking teams. Allocated fronts F=T+L+K. At a free service worker's decision, A≤K−1, U≤T. If P<L,

    free = F − P − A − U ≥ 2.

At P=0 the bound is stronger. Walked/finishing blocks retain backs, not fronts; teams return fronts when walking ends (:1616 onward). The accompanying enumeration checks 5,040 such states. The algebra generalises; the enumeration is not a Rust safety proof and assumes no lost/duplicated ownership.

This excludes **front availability** on that path, not front-buffer cost. Capacity growth, cache migration and repeated first-touch can still lengthen preparation. Log front identity, last owner, capacities, reallocations, rows and input-symbol count. A nonzero “eligible prep blocked by no front” count is an invariant/accounting defect to investigate, not an excuse to allocate more fronts. Raising L also changes F, so the prior threshold experiment changes working-set capacity as well as admission.

## 3. Two mechanisms survive without saturated average workers

**Nonpreemptive finish inversion.** Source priority is: prepare if P=0 and job+front exist; otherwise finish earliest walked block; otherwise prepare if P<L; otherwise wait. At P=1,Q=2,J>0, two successive service decisions can both select finish. A team takes the last prepared block; the other now waits while both services finish. “Never leave P empty if preparation is possible” is true only at a scheduling decision, not across the ensuing nonpreemptive tasks. L=3 or 4 does not change the finish-first decision while Q>0. The witness reproduces these choices exactly.

**Closed-window supply starvation.** Scanner admission waits on issued−taken<W (:479–487). Publishing inserts into ready but does not release the slot (:741–758); the reader increments taken only when it removes its next ordered result (:359–370). Thus jobs can run dry because window credits are held by later ready results behind an unfinished earlier block, or by a slow reader. The relevant predecessor may be preparing, walking or finishing. Finishing min(walked.place) is earliest *available* walked, not necessarily the reader's missing place. A third service worker cannot prepare a not-yet-issued block.

Distinguish these loops before attributing prep starvation to preparation cost. Wider W can break a credit-induced gap and can mask jitter; it does not establish that MTF itself became faster.

For a stationary idealised workload with fixed mean occupation demands p (prep), f (finish), w (one team's inversion), and r (issue-to-take residence), necessary throughput bounds are

    x ≤ min(K/(p+f), T/w, W/r, scanner capacity, reader capacity).

They are not a speed forecast: shared memory contention changes demands, r depends on scheduling, and finite startup/drain break a stationary reading. A rigid one-prep/one-finish partition has ideal service capacity min(1/p,1/f) ≤ 2/(p+f), equal only when p=f. Permanent specialisation may buy locality, but cannot claim a capacity gain from splitting alone.

## 4. Smallest useful next diagnostic

**Register an information objective: classify why an idle team cannot obtain P and B, and identify the next transition that releases it.** Start with untimed counters and deterministic schedules. Validate the collector without workstation captures. A later frozen diagnostic recipe, if needed, is run only through RULES C5 by Louis; no agent timing is proposed.

Use a hub-local sequence number and block/buffer IDs. At state-changing operations, record J,P,Q, free fronts/backs, active preparations A, active finishes C, and active teams U; increment decision counts for empty-P prepare, reserve prepare if tested, finish, threshold wait and no-job wait. Maintain the ownership identities

    F = free_front + A + P + U
    2T = free_back + U + Q + C

on normal paths, with explicit terminal/error accounting. Diagnostic fields must not create nested hub/shared locks or change production scheduling.

| Question | Evidence to collect | Consequence |
|---|---|---|
| Actual team deficit? | Four states P=0/B=0, P=0/B>0, P>0/B=0, P>0/B>0; transitions throughout the wait, not entry label only | Separate missing prep, missing back, both, and delayed pickup |
| Eligible work but no service decision? | At P=0,B>0,J>0: ownership of every service worker, task ID, preparation/finish state; ready-to-take versus actual take | Both finishing supports reserve; both preparing supports prep latency/demand; a runnable idle worker supports handoff investigation |
| Why J=0? | Scanner slot-wait state, issue/take sequence, feed enqueue/dequeue, input availability, oldest missing place and its stage; ready IDs | Separate window credit, scan/input supply and feeder transfer |
| Is prep expensive? | Counts of MTF symbols/rows/checkpoints, capacity growth and buffer migrations; in an authorised diagnostic, per-block phase/ownership intervals and thread CPU separately | Attribute compute versus allocation/migration/lock/deschedule; counters alone cannot measure latency |
| Finish blocks teams/retirement? | Q IDs/ages, C IDs, B=0, reader-next ID, contiguous ready prefix released after each take | Establish which finish releases useful back/window credit |
| Handoff exposed? | Prepared/Walked publication sequence and take sequence, notify/wake/recheck counts; authorised trace for elapsed gap | Do not call a positive-ready interval compute shortage |

Queue snapshots under the hub lock are coherent there. Shared issued/taken/ready belong to another lock: record their own event sequence and block IDs; reconstruct happens-before, retaining ambiguous overlaps. An unsynchronised composite tuple is not one real state.

Deterministic untimed checks: a P=0 wait changing to B=0; both services choosing finish with P=1; a full window with later ready results and earliest missing finish; feeder lag; cancellation/errors/panics returning ownership; startup and drain reported separately. Validate every issued block's terminal disposition and reject dropped/duplicate events. For time-bearing data, integrate these states rather than relabelling an entire wait. Keep the old metrics beside corrected diagnostic attribution.

**Pre-register the branching decision**, not a desired outcome: recurrent finish-induced gaps with eligible jobs → reserve policy; gaps with both services preparing → inspect prep work/locality and service capacity; gaps with J=0/full window → retirement/credit path; P>0,B=0 → finish/back pressure; P>0,B>0 → handoff. Counts establish occurrence, not recovered wall time. Rank using exposed critical-path intervals only once a permitted trace exists; do not add overlapping waits or subtract .158 from whole runtime.

## 5. Adjacent construction, ranked

**First candidate, if the finish witness occurs materially: retain two shared workers, add one in-flight-aware prep reserve.** A concrete bounded proposal for Coordinator/Optimiser:

- Preserve T=2,K=2,W=8,L=2, the existing front/back allocations and kernels.
- On a service decision, finish first if Q>0 and B=0.
- Otherwise, if a job/front is available, A=0 and P+A<min(T,L), allow a prep before finishing.
- Otherwise retain earliest-walked finish, ordinary P<L prep, wait/end.
- A is reserved/released under the hub lock, including failure/cancellation; it affects the priority override, **not a new hard admission cap**. With no finishing work, both workers can still prepare under the existing fallback.

At P=1,Q=2 this prevents both decisions choosing finish: the first starts a prep, the second sees A=1 and finishes. It does not guarantee zero starvation, a permanent finisher reservation, or improvement. Both workers may already be in prep when a finish arrives; the oldest missing output can still be delayed. The B=0 override protects a concrete back-pressure case, not all reader-credit cases. Price locality, added shared state and errors. Test progress under bounded work and adversarial arrivals before any measured comparison. If credit waits dominate, this policy is the wrong first candidate.

**Third worker (2×3+3): conditional second choice.** Buy it only when an additional runnable prep/finish would reduce the exposed path while both services are occupied. Service demand alone has an ideal 3/2 capacity ratio; whole-decoder improvement can be zero or negative. Six team threads plus three service threads is nine decode threads, with scanner, service feeder and caller/input/output activity additional. It is not “one unused physical core” merely because the request says @10. Account for memory traffic, front pool (K changes F), OS scheduling and total active demand. Hold buffers fixed if isolating worker-count cost, or register that confound explicitly. Existing workload/grid and evidence rules remain binding.

**Move finishing:** a permanently dedicated finisher is the rigid-split bound above and risks underusing one role. Moving finish back onto a walking team extends its occupation from w to w+f and reduces inversion overlap; it needs a demonstrated locality win to pay. Moving finish to the reader serialises it and can tighten the very ordered-credit loop under investigation. A separate third finisher is a worker-count experiment. None has a stronger unconditional prior than diagnosing the shared pool.

## 6. Prior art and transfer limits

- Welsh, Culler and Brewer, **SEDA**, SOSP 2001 ([original paper](https://people.eecs.berkeley.edu/~brewer/papers/SEDA-sosp.pdf)): explicit stage queues, resource control, pool sizing and batching make overload and scheduling visible. Transfer the stage accounting and feedback distinction. Its request shedding/Internet-load objectives do not transfer to a lossless decoder: blocks cannot be dropped to make throughput look better. Our two logical service stages sharing workers are deliberate, not a reason to create one fixed pool per stage.
- oneTBB, [Throughput of pipeline](https://uxlfoundation.github.io/oneTBB/main/tbb_userguide/Throughput_of_pipeline.html) and [token-based systems](https://uxlfoundation.github.io/oneTBB/main/tbb_userguide/create_token_based_system.html): bounded tokens constrain concurrency and memory; too few limit parallelism, too many increase resource pressure. The useful analogy is issue-to-retire credit and explicit release point, not a recommendation to replace this implementation with TBB. These docs provide no numeric optimum for mibz2.
- “Work-conserving” must name eligibility. A worker cannot consume a future job or absent back buffer. The present pool takes eligible work at its decision points, yet can cause downstream starvation through nonpreemptive priorities. Work conservation alone does not establish throughput optimality or eliminate ordered head-of-line blocking.

## 7. Research value and limits

Expected: larger L would raise prepared supply and modestly reduce waits. Observed: push counts rose slightly, registered wall comparisons remain inconclusive; the wait labels do not establish a causal shift. **No measured performance gap closed by this review.**

New mechanism information: prep is already multiworker; eligible front exhaustion is excluded under stated invariants; entry-labelled waits can mix blockers; finish priority and ordered credits admit distinct starvation witnesses. The next diagnostic has high information value because it chooses between a priority change, prep-cost work, credit-path work and extra capacity. Another threshold sweep before that discrimination has weaker value.

Artifacts: witnesses.py and witnesses.json, five abstract checks including 5,040 front states and 400 rigid-split cost pairs. No decoder benchmark, native capture, production implementation or safety qualification was run. C29 is untouched. Audit of the fetched channel/inboxes found only the five already-resolved historical routing exceptions; route selects #145 for this REVIEW.
