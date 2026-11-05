# Tail pricing, launch order and release scope

Brief [6659992544](https://github.com/lmmx/mibz2/pull/160#issuecomment-6659992544).
Evidence **1132857e18819ce689424dd2df7751630b29522a**; instructions **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**.
Proposal-only review. No implementation, compilation, timing or workflow dispatch.

## Findings and ranking

The late drain can have a real latency cost even when the busy walk saturates at M=16. It loses concurrency only below that knee, but the last few lanes contribute disproportionately. The tail clock remains necessary: 5.7–6.0% tail bytes does not identify their cost. My conditional random-segment estimate is approximately **0.25–0.27 c/B alone and 0.21–0.25 c/B at four copies** on the two AMD models with fitted parameters. It does not support transferring the entire 0.20–0.45 range to every CPU.

For release-oriented research, rank a measured tail opportunity ahead of an SMT-only placement build **when the configured release really uses physical cores without active sibling contention**. That is not a forecast of a winning tail construction. First validate the tail clock; then price the three existing designs. The fourth design below is conditional and low-priority until predictability is demonstrated. Existing handoff/Huffman/MTF work is not displaced or duplicated.

C15's staged C-resolve retest is now **REJECTED under its registered rule**. C21 remains held for M/P pair-matrix evidence; the C outcome closes its proposed partner-selection mechanism. Cross-chain lockstep remains closed. None earns new credit.

## Pinned audit and evidence limits

Read at the evidence pin:
- segment-cost.md and its tailsim.py;
- corun-placement.md;
- compact30-staged.md;
- tail-passes.md;
- src/bwt.rs and src/coop.rs;
- the earlier binding-census walk evidence.

Recovered prior art from the blog index, the room-walk-overlap and memory-level-parallelism posts, and the research portfolio register (blob recorded in evidence.json). Relevant precedents are P21/P23 (lanes/segment tuning), P30 (start placement), P119 (cross-block lockstep), P120 (AMAC), and P90/P122 (cooperative work and dispatch). The proposal below changes launch order of existing segments, not any of those constructions verbatim.

This is a source/model audit, not an independent rerun of the oracle suite, complete machine listings, raw timing artifacts, Miri or simulator inputs. Performance and test results below are archived observations. I previously proposed the staged compact retest; this assessment is not a blind qualification.

**Compact:** the reported 4t wall intervals are wholly slower: +4.8% (+3.2,+6.4) and +7.7% (+5.9,+9.6). Benefit fails and the same-head settling run repeats harm. Reported correctness is unchanged output, not a new safety certification here. The products 1.114×0.961=1.0706 and 1.136×0.973=1.1053 reproduce the CPU ratios. But amplification is CPU4/CPU1: this multiplication is largely an accounting identity, not independent proof that the amplification reduction is caused solely by L2 hits. The 8573C alone walk improvement supports the residency hypothesis; it is not a whole-decoder win. Layout C versus L also changes instructions and addressing. Neither outcome proves all compressed representations impossible.

**Segment cost:** 457 Ir/segment and the lean 108 Ir/segment are archived instruction observations. The cycle decomposition is inferred and explicitly solved against CI net cost; its agreement with that cost is not an independent validation. The quoted gross 380–450 minus 147 gives 233–303, rather than literally 262–338 throughout; the rounded component ranges/cases cannot be treated as one exact identity. Require paired per-case component arithmetic before using those endpoints as a precise budget. This does not rescue the lean gate: the reported 33–64 is well above 10, and the whole-walk cost model is adverse. Sweep segmentation of tiny blocks is a real mismatch with production's MIN_SEGMENTED fallback.

**Tail passes #173:** closed as not shown on wikidata, not a proof that tail instructions have zero cost. The archive reports benefits on pattern and onebyte. Busy and tail bodies differ: bwt.rs has staged tiles in the busy loop, then a compacting Vec of lanes, direct stores and capacity checks in the tail. An extra instruction term can coexist with lost MLP.

## Independent derivation

Let n_k be useful tail bytes produced with k live lanes, n total walked rows, and q=sum(n_k)/n. Under the additive fit t(k)=a+lat/min(k,M), excess over producing those bytes at 32-lane rate is

    E = sum_k (n_k/n) * lat * [1/min(k,M) - 1/min(32,M)].

The constant a cancels **by assumption**. Thus tailsim.py structurally excludes a change in instruction cost between busy and tail; its result cannot establish “latency, not instructions.” Use the more general expression sum(n_k/n)[t_tail(k)-t_busy] for the actual clock. Boundary, finish, growth and flag cleanup costs must be classified separately and consistently.

For randomly sampled segment boundaries, a useful independent approximation is exponential segment lengths with mean ell=n/S. At queue exhaustion, 32 residual lengths then also have mean ell. Expected rounds at k live lanes are ell/k, so expected bytes at each k are ell. This predicts q≈32/512=6.25%, close to but not proof of the observed 5.7–6.0%. Condition on the observed q while retaining equal byte weights q/32:

    E ≈ (q*lat/32) * (H_M - 1), for integer M≤32.

H_M is the harmonic sum. For M=16, H_16−1=2.381; for M=32 it is 3.058. Halving the knee does not halve the excess: at fixed latency it reduces this model by only 22%. Correlations, finite permutations, cycles and length bias at the real exit can violate the exponential approximation.

| CPU / occupancy | lat, M used | Conditional E, c/B of entire walked block | Fraction of archived walk cost |
|---|---|---|---|
| 9V45 alone | 47.1, 32 | 0.257–0.270 | 8.9–9.3% of 2.89 |
| 9V74 alone | 52.0, 22 | 0.249–0.262 | 5.8–6.1% of 4.27 |
| 9V45 four copies | 44.6, 22 | 0.214–0.225 | 3.3–3.5% of 6.44 |
| 9V74 four copies | 54.8, 16 | 0.232–0.245 | 2.6–2.7% of 9.08 |

Parameters are from the pinned census fit used by tailsim.py; the shape derivation is mine. Fit predictions and archived walk denominators need not coincide exactly. These are conditional model outputs, not intervals on a measured saving. Zero remains the demonstrated lower bound; implementations may lose.

**Other models:** neither 7763 nor either Xeon has a validated tail knee in this evidence. A sensitivity, not a fitted price: with M=16–32 and 7763 plain-chain latency 40.8 alone / 47.9 co-run, the same expression gives about 0.17–0.23 alone / 0.20–0.27 co-run. For 8370C, M=8–32 and lat=43.9 alone / 68.3 co-run gives about 0.13–0.25 / 0.21–0.39. These plain-chain latencies belong to different partner/cache conditions; they cannot substitute for a tail measurement. The 8573C price remains unquantified: measure its latency/knee or directly t_tail(k), rather than borrowing the 8370C. A larger cache can reduce latency and change this ranking.

**One thread per core:** alone parameters are a first approximation, not a measured 2t tail price. Two busy physical cores can change shared-cache pressure, frequency and memory latency even without SMT. Apply the same histogram formula using spread-2t tail rates.

**Whole-decoder budget:** if all modelled excess could be removed for free, multiply E/t_walk by the appropriate walk time share. On 9V45 alone, a historical walk share ~37.5% gives ~3.3–3.5% decoder wall; four-copy walk shares ~50–60% give ~1.7–2.1%, conditional on unchanged siblings/utilisation. For 9V74 four copies, ~50–60% gives ~1.3–1.6%. There is no matched 9V74 alone whole-decoder decomposition here. These are gross pools, never predicted net gains. A 0.05 c/B whole-block overhead consumes about one fifth of a 0.25 c/B pool.

## What would invalidate 0.20–0.45?

The histogram is decisive. With q=6%, most tail bytes near k=16 give little excess at M=16; most near k=1 can cost several c/B of the whole block. Identical q permits very different answers. Other falsifiers:

- Loaded latency falls as lanes and sibling demand drain; constant lat overprices the end.
- The fitted M is a large-L curve parameter, not literal outstanding misses. Extrapolating it to k=1–8 can fail.
- Execution and misses overlap rather than add. The model.json max-versus-sum sensitivity yields ~0.15–0.19 at four copies, but it is **not a second calibrated fit**.
- Tail buffers, branch work, growth and direct writes change a; ignoring them can underprice total tail overhead.
- The scalar simulator omits tile flush time, prefetch/cache history and exact mid-offset exit bookkeeping. Structural byte counts need comparison with actual counters.
- Periodic and short blocks, multiple cycles, the forced origin start, duplicate starts, and size weighting change residual lengths. A two-block sample is not a release corpus estimate.
- Simultaneous drains can relieve shared contention or create a team straggler; neither is represented by independent constant-rate lanes.

A CI falsifier should record useful bytes and clocks separately for busy and tail, alone, spread-2t and four copies, per CPU model; bin tail bytes by live count (1,2,3–4,5–8,9–16,17–32 is a minimum). Separate transition/finish/grow work, exclude assembly consistently, and report medians as well as fastest rounds. Timing every lane step would perturb exactly the effect being tested: use aggregate phase clocks and a separate counting arm, validate perturbation, and preserve production's tiny-block fallback. Report absolute cycles/byte and elapsed/CPU time; frequency changes must not masquerade as latency. Compare measured excess with histogram predictions before production builds. No such measurement is dispatched by this review.

## C22: length-aware launch order, with reusable bounded scouting

A fourth construction: keep exactly master's existing starts and segments, but try to launch the segments likely to have long remaining paths earlier. A bounded scouting phase advances existing segments up to a fixed budget, retaining every decoded prefix in its original buffer and each resume row. Completed short segments are done; unfinished segments are prioritised only if a cheap feature predicts **remaining**, not already-walked, length. Resume them with normal lanes. No extra START marks, no denser row-regular segmentation, and no second block/table.

This attacks the residual-length distribution rather than creating more concurrent chains once only one remains. It is distinct from soft starts, cheap additional segments and cross-block continuity. It is also different from C21's selection of whole blocks. Generic launch-order scheduling is not claimed as novel prior art.

**Equivalence conditions:** retain the same segment IDs, starting rows, emitted bytes and successor relation. A segment is in exactly one of unstarted, suspended, active or finished states. Scout prefixes are never decoded or appended twice. Suspended state stores offsets, not dangling pointers across buffer growth. START ownership and first-byte handling remain identical, and assembly follows the original successor relation, not launch order. Every cycle and malformed-state guard remains bounded as in the incumbent; no unchecked jump, speculative output length or shared mutable table access is proposed. This is a proof obligation, not a safety PASS.

**Price and hard early falsifier:** the exponential model is memoryless. If real segment residuals behave that way, merely surviving a scout gives no information about remaining length, so this design should be pruned. In fact, under that model the final 32 residuals still have mean ell: scouting short prefixes does not buy the harmonic tail pool. Do not build it on that assumption.

For n=900,000 and S=512, a 64-step scout can touch up to 32,768 rows (3.64% of n), yet only about 1−exp(−64/1758)=3.6% of exponential segments end in that prefix. A 512-step scout reaches up to 29.1% of n and still completes only about 25% of segments. Reusing bytes avoids duplicate pointer work, but adds suspension metadata, dispatch and changed cache/write behaviour. Counting its prefix work as a new saving would be double credit.

Its only viable gate is held-out structural evidence that a cheap available feature predicts residual lengths well enough for an oracle-versus-feasible launch-order comparison to show a positive budget. No exact-length oracle pass is free. Price net as reduction in measured-histogram tail excess minus scouting's incremental work, allocation/copy traffic and queue/cache cost. Reject if even oracle reordering saves less than those costs, if a feasible predictor fails out of sample, or if improvement depends on knowing true lengths from an extra full walk. Under current evidence C22 is **CONDITIONAL, zero credit, below the existing three rivals**, not a recommendation to implement. This negative gate is a useful fourth competitor: it tests whether reducing imbalance without buying boundaries is possible.

## Scope correction and teams

The claim is directionally useful but too absolute.

- A tail lever can pay in **1t, 2t and 4t**. The model's absolute four-copy excess remains appreciable; its fractional value is smaller. No positive release saving follows until overhead and actual release workload are included.
- The placement and window-light designs in corun-placement.md engage only when threads exceed physical cores. Their 1t/2t rows are unchanged **by dispatch policy**, not because a lighter instruction window cannot matter without SMT. Changing that policy needs its own evidence and registration. Keep their current scope.
- One decoder thread per physical core does not guarantee its SMT sibling is idle: caller stages may run there. Release topology, placement and other work determine applicability, not CI row labels alone.
- For teams, coop.rs takes segments from a shared atomic counter, uses roughly k times the starts and k×L lanes, and Pool::run waits for every helper before returning. The “Nothing uses it yet” file preamble is not a reliable description of current integration. The actual barrier means block time follows the **slowest** finishing thread, not mean per-thread tail cost. More starts reduce individual segment lengths but more residual lanes and the maximum across threads work against that; memory bandwidth and a shared table change latency too.
- Therefore teams may raise the priority of straggler reduction, but do not justify multiplying the single-core saving by team size. C22's launch ordering could be especially relevant to the maximum if prediction exists; soft starts/cheap boundaries need their shared-table ownership proofs. Cross-block continuity changes the team's barrier and table residency, so cannot simply fill a private worker's tail. Team-specific start counts, residual histograms, barrier slack and critical-path reduction must be priced separately.

No fresh team timing or six-core forecast is supplied. Historical P90/P122 and current shared-counter source already cover dynamic segment assignment; generic work stealing is not a new proposal. Frozen constructor credit, A1–A7 and D013 remain unchanged.
