# Two modes: engineering value without pretending the cause is known

Inbox6756316971, dispatching QUEUED6735326795. Specifications a2da49638ac54d7763de1c167bd054f67ffdc763; baseline3e1a7ec361ee6c8ea6b540264aa69189d4823d9e. #313 inspected at ce6954e65f285953b0cb8dd18f2390b4dc2cc91b; #316 at b63f6bee2e3ceb821198129d649bed2bd2fb349e. Proposals only; no implementation or timing.

## Decision

Service first for target adoption; bounded removal of wasted wait work next for prototyping. Do not queue a third dir/default33 attempt under a new name. Preserve dir's attributed CI LEAD and both failed-fidelity workstation results. A future target-gated dir proposal needs a new, diagnostic-supported information case; not merely more rounds or a new acceptance route.

The current C6 delegates prospective design/acceptance/adoption, with Researcher review, Guardian gate rulings and frozen decisions. It supersedes my earlier report's claim that those choices always require Louis. C5 still reserves workstation execution to Louis. The target-gated route is enacted at3e1a7ec3; it is not a waiver of its seven conditions. D077 permits designing sharded CI, not qualification at an unfinished protocol.

## What the two-mode evidence actually narrows

Pathfinder's journal reports29 fast/111 slow, median walls .159/.306 and CPU1.08/2.25; the result journal's28/112 uses a slightly different descriptive cut. Neither is a qualification population. Retain all140 observations.

The endpoints and equal median gaps fail to explain the modes; they do not exclude in-run frequency, local thermal excursions, carryover or arm×state interactions. “Every arm has both modes” refutes a deterministic arm-only explanation, not treatment influence on the probability of a state. A fast successor rate15/29≈.517 versus29/140≈.207 supports dependence descriptively; without boundary/order accounting it is not a causal test or a precision estimate.

Several stronger claims need narrowing before they steer engineering:

- SMT packing is not constrained by a universal2× slowdown bound. Shared execution, cache/TLB effects and work changes make that a model assumption, not an exclusion. Last-CPU samples are not simultaneous running-state occupancy.
- A spin probe measures loop progress per elapsed TSC tick, not frequency alone. Scheduling, SMT, instruction throughput and its own power consumption affect it. The “idle” core becomes active; fencing it reduces decoder capacity. Its diagnostic is an intervention, not passive replication.
- Low package-temperature snapshots do not eliminate regulator/local thermal or power-limit mechanisms. Missing event counters do not mean zero events.
- Short-process effects need not disappear in long decodes: migration, recurrent allocation and changing demand can recur. Conversely sustained power throttling is not guaranteed merely because bursts have a slow mode.
- CPU seconds are neither instructions nor joules. Lower CPU/byte does not mechanically become the same percentage of throughput under H3/H5.

Kernel documentation independently supports the narrower measurement reading: intel_pstate's scaling_cur_freq depends on operating mode/update interval, rather than supplying the decode's full clock history; THP documentation uses process smaps/AnonHugePages to establish a mapping's backing. Global vmstat changes alone cannot attribute pages to this child. Sources: https://docs.kernel.org/admin-guide/pm/intel_pstate.html and https://www.kernel.org/doc/html/v6.12/admin-guide/mm/transhuge.html (read2026-11-10). No host setting or allocator change proposed.

## Conditional ranking of the four properties

C=less executed/wait work per byte; P=owned placement; M=smaller/reused working sets; B=work-sensitive thread budget. Rankings below are mechanism-conditional hypotheses for ten-thread-request i9 use, not measured gains. “CPU” means whole-process CPU seconds on matched work, with kernel work included where available. None has a defensible numeric forecast yet.

| Mechanism | Expected wall priority | Expected CPU priority | Engineering consequence and cost |
|---|---|---|---|
| H1 governor ramp | B or locality-preserving reuse first for genuinely short jobs; C next; P last unless migration restarts ramp | C first; B/M next; P uncertain | Avoid launching idle helpers for insufficient ready work; amortise real worker/buffer setup. Fewer threads can also ramp less aggressively and lose parallelism. No synthetic warming or governor setting change. |
| H2 sibling packing | P first if physical spread is actually missing; B next when runnable work exceeds useful cores; C/M next | C first; P/M can reduce contention CPU; B conditional | Respect allowed mask and quota; map critical roles, not just helpers. Charge discovery/binding and interference with other applications. Affinity is a diagnostic candidate until placement benefit exceeds these costs. |
| H3 stop-clock | C/M have useful-work benefits; no property guarantees escaping externally imposed duty. B/P weak unless decoder demand triggers throttling | C first, M second, B conditional, P last | Remove real work and traffic. If duty is externally fixed, fewer threads can make wall worse. Do not promise to repair board/firmware throttling in decoder code. |
| H4 page backing/faults | M first; B next for short allocation-heavy jobs; C then P | M first if faults dominate; C next; B conditional | Reuse bounded existing allocations and avoid redundant touches. Compact representation earns credit only after decode/reconstruction costs. Do not change allocator, force THP or equate bytes saved with proportional speed. |
| H5 package power | C first; B/M compete second; P uncertain | C first; M next; B conditional | Reduce instruction/traffic/wait demand; balance concurrency against energy/byte. Spread may raise package power; fewer active workers may lower power but lengthen the run. Need sustained end-to-end evidence, not burst CPU alone. |

Most robust direction is **remove work without removing useful parallelism**. Next is bounded reuse where an actual allocation/touch census identifies redundant work. Blanket affinity and a hard-coded small-input threshold are not robust across all five.

A minimal model makes the uncertainty explicit. Let per-byte work be W, effective useful rate q(f,duty,contention,parallelism), and serial/setup time S. Then T=S+W/q, not T proportional to CPU. Reducing W by fraction r gives at most r of the exposed work component if q stays fixed; the whole-decoder gain is diluted by S and other bottlenecks. Under an energy-limited model throughput may scale as package power/energy-per-byte, but CPU time does not determine energy-per-byte. Pathfinder's14.7% CPU reduction is evidence worth pursuing, not14.7% guaranteed wall recovery. None of the four properties is a guaranteed win under every mechanism.

## Buildable brief A: separate pool waiting cost from the refuted bimodality hypothesis

**Worth prototyping now**, opt-in and correctness-only before the diagnostic reports. Scope small enough not to disturb #316's frozen candidate when it exists.

Files: src/coop.rs (Pool caller join and helper idle wait policy), src/parallel.rs (research selection only), examples/wait_audit.rs and examples/ab.rs (arm identity/counters), new bench reporter/registration. Pin the starting tree and keep the production default unchanged.

Arms: A=current pool budget2000 and current walk budget; B=pool budget0; C=pool budget64. Keep the walk barrier policy, task partition, front/back allocation, kernels and source identical. First compare on the service plan; a separately specified default3x3 context tests transfer, not an unannounced extra primary. Expose a construction-time wait policy, not an environment-dependent hot-loop decision or post-outcome adaptive threshold.

The source has separate pool SPINS and walk BARRIER_SPINS. The existing spin2k lever shortens walk waits and is not this experiment. Preserve condition-recheck ordering, unpark tokens, completion publication, cancellation and join-before-reuse; zero spins must fall through the same park protocol. Guardian B5 where touched obligations require it, plus lost-wakeup/mutation/Miri coverage; a hang fails the candidate.

Prediction: less useless wait work/CPU; wall neutral or improved if most waits park anyway. Falsifier: reduced spin counts but extra park/wake latency exposes the critical worker, yielding wall harm, or no meaningful CPU reduction at target. The VM's98–100% wasted spins are motivation, not an i9 distribution. #311 refuted spins as the cause of the2× mode, not all marginal value in wait tuning.

Price the bound before timing: N removed spin iterations times target CPU cost per iteration is an upper estimate of saved CPU before extra park cost. Target spin cost and exposed fraction are presently missing. The journal's3.3M bound covers all waits, not this pool-only treatment; never credit the entire bound to B. Do not transplant VM ns or an assumed PAUSE latency as a measured saving.

CI falsifier: same-output/cancellation tests, instrumented wait counts for mechanism, and a separately frozen paired whole-decoder comparison of A/B/C on the standing corpora. If only oversubscribed CI benefits, no i9 promotion. All attempted policies retained; no search for the best lucky budget. A prospective three-arm screen at20 rounds would cost60 decodes per workload/budget, plus any independently required fidelity control; sample size must be justified before execution. No workstation screen before the mode diagnostic's result and review.

## Buildable brief B: readiness-bounded worker activation, not “compressed bytes imply small work”

**Prepare design now; defer implementation behind A and the existing diagnostic.** Files src/parallel.rs (scanner-to-job handoff and worker creation), src/coop.rs (pool lifecycle), src/read.rs/src/lib.rs public-wrapper tests, examples/ab.rs research arms.

A=current eager team/service worker activation; B=start a bounded subset, activate remaining workers once scanned eligible jobs exceed current active capacity. Freeze a small deterministic rule based on already decoded block metadata and queue readiness; do not inspect elapsed time, clock mode or benchmark corpus name. Keep a bounded queue, byte order, errors and cancellation. No full-file prescan. Requested threads remains an upper bound, not a promise to use all of them.

For an explicit diagnostic screen, compare fixed active caps1/2/full before committing an adaptive transition rule. These are competing hypotheses, not automatically retained production arms. A .bz2 compressed length or level header is not decoded size; run-rich blocks can expand greatly. The existing MIN_COOP/lazy cooperative pool paths must be counted before claiming new work removal. Benefit must come from thread/stack/hub creation or active teams currently unnecessary, not relabelling existing small-block fallback.

Prediction: tiny/few-block inputs avoid setup and idle helper work; large inputs reach full throughput. Falsifier: scanner ramp/first-block latency or missed overlap dominates saved setup, or large-stream wall regresses. CI on onebyte/runs/random plus large Wikidata and adversarial mixed block sizes; whole-process CPU, first-byte latency and peak retained capacity. Ordinary parity alone cannot falsify the performance claim.

Bounded buffer reuse is the alternative if the census finds repeated allocation instead. Existing front/back/symbol/output pools already reuse within a decoder; “reuse buffers” is not a new algorithm. Across-decoder reuse requires an explicit reusable-context API, bounded retained capacities, clean error reset and ownership proofs. That API/lifetime cost makes it inferior to A before evidence. The speculative0.8-byte/row representation is a separate construction, not a free reuse patch.

## Placement: useful H2 discriminator, not an automatic production fix

For a later opt-in arm, src/parallel.rs and src/coop.rs would receive a deterministic role→allowed-CPU map, examples/ab.rs would record it, and wrapper tests would verify mask restoration/fallback. Compare inherited scheduler placement to physical-core-first placement on identical work and total budget. Map scanner, leaders, service workers and helpers, accounting for input feeder/caller; there are more thread roles than the eight headline decoding workers. “One thread per core” is not implementable by pinning every role uniquely on ten cores without changing concurrency.

Failure to bind must be reported and handled by frozen fallback, not silently treated as successful placement. Do not widen affinity or alter host governor/SMT. Correctness/CI cost controls can run before target evidence, but useful wall/CPU benefit requires a true target H2 signature and a matched placement intervention. Spreading could hurt H5; thus lower priority before the diagnostic.

## Service versus dir: the route changes admissibility, not evidence

Service leads. #316 discloses #302's prior guarded result: speed1.068 [1.043,1.095], CPU reference/arm1.173 [1.153,1.193] at10. Central reductions are6.37% wall and14.75% CPU, on that attempt only. Earlier fallback12/16 were NOT SHOWN; old CI harmed small cells. #316 narrows SKU/quota and removes off-target Machine::here discovery outside budget10. Its causal explanation for small-cell cost remains plausible, not proven by the old run.

The draft's on-target package is2 arms×3 budgets×128 rounds=768 decodes; it is not frozen. D077's sharded CI candidate R600 costs roughly160–280 aggregate measurement minutes under the supplied estimates, not a guaranteed60-minute shard or85% pass probability. It still needs exact protocol/review. #316's on-target freeze waits for the mode result as LOG6756120169 states.

Dir has a real but different result: attributed CI LEAD at03e52206, head/base wall .9669 [.9594,.9744], CPU .9640, nine requested threads on four vCPUs. Its target attempts were independently VALID/INCONCLUSIVE with failed control fidelity; the second primary head/base1.0545 [.9290,1.1969] is not supportive target superiority. Do not invert conventions across these tables by accident. Post-run dir/reg≈.9921 is a configuration contrast, not isolated direct-fill credit.

**Do not drop the scientific idea solely because the timing sequence closed. Do drop it from the immediate target-adoption queue.** A future registration is worth review only if:
1. the diagnostic yields a materially changed, credible observation protocol, with dependence/invalid handling fixed;
2. the Coordinator explicitly reconciles the closed sequence and any live restriction—no silent third attempt;
3. source/count evidence identifies exposed target work beyond C29/service, rather than claiming the entire CI3.3% belongs to direct fill;
4. a fresh frozen candidate/current default, narrowly justified target coverage and off-target noninferiority meet all route conditions.

If the diagnostic shows persistent external H3 or a measurement problem without a workable estimator, do not spend another dir slot. If it establishes a reproducible relevant short-process or sustained regime, one new information-driven design may be valuable; preserve both old results and never condition the verdict on favourable post-run modes. Service+dir composition also needs fresh interaction evidence: front work moved to service workers changes the scheduling premise. Their percentages cannot be added.

### Rounds: the old question has no honest fixed answer yet

The paired log-difference variance, not marginal wall CV, determines precision. With independent pairs and true log-speed gap δ above the threshold, approximate n=((z_critical+z_power)σ/δ)^2; dependence inflates it by the applicable design effect. For illustration only, aiming80% power, one-sided5%, true speed1.05 versus minimum1.02 gives δ=log(1.05/1.02)=.02899. σ=.05 suggests19 independent pairs; σ=.25 suggests460. Route multiplicity and t quantiles increase these, and neither σ is an estimate of the next run. Persistent-state pairs cannot be made independent by calling each a round. Freeze the dependence-aware sampling model and sample size after diagnostic design review, before qualification data. No sequential top-up.

## C3 and provenance

Expected: a robust engineering direction despite unexplained mode changes. Found: service retains the best target case; wasted pool waits are a distinct bounded optimisation lead; none of the four properties repairs every possible cause. New measured gap closed: zero. New engineering consequence: test pool-only waiting separately, avoid categorical causal exclusions, and do not fund a third dir attempt from a changed gate alone.

Sources: exact pinned specs, inbox/queued comments, #313 two-mode journal, #316 registration/source, enacted STEWARDSHIP, spin audit, CI leader result and second-attempt closeout, D077 and sequencing LOG. Repository sources are available under the pins above. Raw archives were not replayed and reported test outcomes were not rerun. Primary kernel documentation supports only the measurement caveats stated above. No implementation, compilation, timing, safety certification, merge or acceptance change.

Migration verified old head e6eea5889432aef28f4d9e450cee87dd5bf7bacf and manifest SHA25686d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde. coord.py at the new spec pin audited the five channel/inboxes: five resolved historical findings, zero unresolved. Its verify still requires FROM coordinator; exact optimizer readback is recorded separately.
