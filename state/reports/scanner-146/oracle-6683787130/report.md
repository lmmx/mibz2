# Oracle review: an intention tree can preserve a mistaken reason
Researcher, 2026-11-06. Responding to [inbox 6683787130](https://github.com/lmmx/mibz2/pull/162#issuecomment-6683787130).

## Judgment
Keep the observer role and its narrow correction route. The proposed shape is useful, but a fully sourced chain can still convey a false necessity. The live example is the conversion of **25% less aggregate footprint** into **25% less touched T required for a useful speed gain**. This is already changing the programme's direction. An Oracle should report the Coordinator's actual restriction without certifying the reason as established science.

This is independent advice, not approval of #223 or a new gate. I recommend correcting the seed's major discrepancies and distinguishing a purpose link from a scientific premise and a scheduling dependency. No implementation, benchmark, merge, registration amendment or Oracle-branch edit is made here.

## Evidence and scope
- [Oracle specification](https://github.com/lmmx/mibz2/blob/8e3f6352baad3c26c3aa1c19d3ab74c3a4752c65/docs/journal/compact30-channel/SPEC.ORACLE.md) and ROLES §7 at that pin.
- [Seed](https://github.com/lmmx/mibz2/blob/01ea79a345a342dfce29b448be3c0ffe9adf0c72/docs/journal/oracle/intention.md).
- [House-goal source](https://github.com/lmmx/anscombe-intention/blob/HEAD/.claude/skills/house-goal/SKILL.md), fetched through the repository contents API on this run. This is the supplied interpretive source, not an independent reading of the full text of Intention.
- [Team-size journal](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md) and [three-table registration](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-three-tables.md); the latter still says not yet run at this inspected head.
- [My earlier raw-row reconstruction](https://github.com/lmmx/mibz2/blob/186702b380614658aa2864908de3b3ab92e836a3/docs/journal/compact30-adversarial/scanner-146/three-tables-6682938532/report.md). Its arithmetic is reused, not represented as a new raw replay.
- [Optimiser's report](https://github.com/lmmx/mibz2/blob/b6d99258c84d285614dd2fddc7dedcd2737eea8a/docs/journal/compact30-research/reviews/inbox-6683037641/report.md) and adjacent model.json; [Coordinator's ensuing decision](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683240055); [Guardian D030 announcement](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683039047).
- Governing Researcher/common specifications at 0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0; current constructor-branch ROLES §3 at 4109dccd512b867af8e3ce28b31b4184b8ca62e4.

I previously contributed the #220 causal qualifications. This review is not blind. The new contribution is the dependency analysis and explicit counterexample below; it is not a fresh performance measurement.

## 1. Counterexamples to the design
The Oracle is an observer of reported intentions and outcomes. It can help Louis exercise practical direction; its record does not acquire the builders' first-person authority or make their empirical beliefs true. Preserve the distinction between "the Coordinator intends X for reason Y" and "Y has been demonstrated."

| Case | Can the present rules admit it? | Minimum remedy |
|---|---|---|
| Every role supplies a purpose leading to 0, but the reason for choosing this means is an unsupported necessity claim | Yes. A genuine intention is compatible with a mistaken means-end belief. Mere reachability and a source link do not test the claim | Keep the stated purpose; separately label its efficacy premise as evidence, inference, or decision. Flag a material disputed premise, without deciding the route |
| A later checkpoint quotes an inferred mechanism as a fact | Strictly, no: Known already requires evidence and inference labels; the weak-chain rule covers dependency on something unshown | Enforce those rules across rebuilds. A later summary is not new empirical evidence; retain the original support and caveat until an identified result changes them |
| "Waiting on Louis for a run" masks a discretionary decision to dispatch nothing else | Yes, if the report accurately quotes the pause but omits its scope and reason. The status rule asks who/what, not whether this is a resource choice or necessary dependency | Record "Coordinator deferred dispatch until result, decision [link]; scientific dependence unestablished." When missing, flag the decision/reason as unstated |
| A sound experiment executes correctly and rejects its speed prediction | The prescribed execution-failure/plan-fault language can misclassify useful science | Report execution against registration separately from hypothesis support and qualification. A falsified prediction need not mean a faulty experiment or execution |
| A task serves two independent fronts but is forced under one parent | A single-parent presentation can hide the second reason, especially when the displayed parent closes | Permit one primary parent plus a short additional sourced purpose/dependency link; do not invent another final objective |

The first row does not require Oracle to originate a technical review. It must distinguish what a role decided from what evidence established, and carry a supplied substantive disagreement when that affects direction. If all record authors agree on an undetected false premise, no summariser can guarantee discovery under a descriptive-only remit. This is a limit, not an excuse to appoint another gate.

The house-goal source itself permits several reasons and distinguishes achieved from handled. Those distinctions support, rather than undermine, the modest additional bookkeeping above. A negative experiment can be **performed as intended**, **not qualify a candidate**, and **advance the cause-finding front** simultaneously.

A minimal practical audit for a "waiting" edge: state which downstream action changes if the awaited result passes, fails on benefit, fails on resources, or is invalid. If no specified action differs, that does not prove the wait irrational—staff or machine scheduling may justify it—but it is not an established informational prerequisite. Cite its actual scheduling owner and reason.

## 2. Seed accuracy

### The #219 figures are substantially right; the causal compression is not
The 16 cells × 20 rounds, primary ratios 1.017 and 1.003, three-table ratios 1.097 and 1.075, and approximately 18% lower peak RSS reproduce the journal and previous independent reconstruction. Per-table rates are approximately 558–583 MB/s at two tables, 461–466 at three, 319–325 at four and 110 at eight. They are whole-run throughput divided by team count, not direct block service rates.

"Speed follows tables, not threads" should remain a descriptive lead for this host/input/grid. At fixed k, changing team count changes decoding concurrency and total window as well as tables. A high-level "Found" line without that scope lets a reader treat a mechanism as established even though the detailed caveat survives lower down.

"More helpers didn't help" also overstates: 4×3 gave 1.016955 with interval 1.007929–1.026062. It missed the registered 3% threshold; it did not demonstrate zero improvement. 4×4 gave 1.003463 (0.992881–1.014157) and failed the CPU cap. Keep "neither qualified."

### 25% is an equality of demands, not a speed threshold
The Optimiser explicitly writes: at fixed shared demand and otherwise identical behaviour, matching **aggregate effective footprint** requires 25% less per team. With F effective bytes/team and G shared bytes:

    4(1-delta)F + G = 3F + G  =>  delta = 1/4.

That identity is correct. It does not make delta >= 1/4 necessary for useful improvement, a cache-fit transition, or matching the three-team throughput. Nor is it sufficient for a speed gain once decoding overhead, access patterns and service capacity change.

A counterexample uses the Optimiser's own hypothetical uniform-capacity model, not a calibration to the i9. Set C = 20 MiB = 20,971,520 bytes, G = 0, F = 5,400,000 bytes (3.6 MB T plus 1.8 MB effective competing bytes). Three teams fit; four have h = C/(4F) = 0.9709037. Reducing F by only **157,120 bytes = 2.90963%** makes four fit at 5,242,880 bytes/team and h = 1. Both then have the same model hit fraction without equal aggregate footprints. A latency-dominated extension can therefore improve four teams, even beyond three with sufficient parallel service, without a 25% cut. This disproves necessity; it predicts no real gain.

More generally, in this simplified model the fractional cut to fit four teams is

    max(0, 1 - (C-G)/(4F)),

when usable capacity is positive. It equals 25% only when C-G = 3F. Access locality, phase overlap and non-capacity limits may invalidate the model altogether.

The Coordinator's 6683240055 then turns the equality into a condition to reopen research: at least 25% fewer touched **T** bytes. That is a second substitution: effective competing demand is not T alone. The source ledger includes written segment prefixes and other phases; T itself is at most 4n bytes, 3.6 MB at n=900000, not the complete active set.

"No current candidate does that" is supportable only as **no reviewed candidate establishes this 25% effective-footprint reduction at acceptable cost**: C23 removes an allocation outside the direct walk, C24 awaits evidence, C15 remains closed. It is not proof that every smaller reduction is uneconomic. A 30-bit versus 32-bit table saves 6.25% of T storage before extra costs, a quarter of the nominal 25% target, not an order of magnitude short. This does not rehabilitate C15's measured failure or authorize a repeat. It challenges the additional blanket exclusion, while retaining the actual candidate dispositions.

### Occupancy and the #220 rule
The seed's no-common-k caveat is correct for the **new #220 diagnostic set** 4×2, 3×3, 3×4. The amended registration explicitly defers the same-k block-time claim. Do not confuse this with the existing timing grid, which has exploratory same-k comparisons. Adding same-k alone would still leave concurrency/window and possibly mask differences; it would not identify L3 capacity.

The seed's #220 rule is a faithful short summary: mean ratio >=1.03, interval lower bound >1, all validity predicates, RSS/CPU limits, same-cell confirmation in one predeclared second run. The linked rule supplies <=1.15 RSS, <=1.5 CPU, no pooling/switching/third attempt, one experiment-wide invalid replacement, and no screen reuse. D030 supports carrying over the numerical rule; it is not a candidate PASS. The seed's later "candidate, own PR/CI/parity" paragraph correctly limits its earlier automatic-sounding cap-change sentence. No numerical gate amendment is proposed.

## 3. Weakest why link and the next consequence
Item 1.2 explicitly names parent **1**, not 1.1. The closing "none open" paragraph says its link to 0 runs through 1.1. That is internally false. Cause-finding remains useful to the scaling objective if neither new candidate qualifies. A small positive effect below 3%, resource rejection and an invalid run have different implications; none automatically removes the reason to understand scaling.

There is, however, a real Coordinator decision to dispatch nothing until #220 reports. Correctly record that decision. Do not silently unpark work or replace it with Oracle's preferred priority. The scientific claim and the sequencing decision must be independently visible so Louis can change one without rewriting the other.

Placement has independent empirical motivation: the existing 4×2 packed/spread ratio is 0.777876 (0.755056–0.801386). This changes cores available as well as sibling sharing; it is not an estimate of pinning versus the unpinned production decoder. It remains worth a separately priced proposal whether #220 succeeds or fails. The selected policy may change which baseline is worth testing, not whether the placement question exists.

Pipelining is a separate, less established opportunity, not supported merely by idle logical processors. About 96% leader decode time says little about helper wait and productive work within that interval. Recoverable serial phase time, overlap, buffering and synchronization cost need a price before recommending implementation. Do not bundle this hypothesis with the measured placement lead or assert equal priority.

**Novel consequence:** the immediate conceptual repair is to separate three relations: purpose (1.2 serves 1), empirical premise (capacity is unproved), and dispatch order (Coordinator waits for 1.1). A single tree edge cannot safely stand for all three. As a concrete test, a failed #220 run with a real 2% gain leaves cause-finding useful, invalidates no sound experiment, and may still leave the dispatch pause intact until the Coordinator interprets the result. The page should represent all three facts without inventing a failed why-chain.

## Major corrections for forwarding
ORACLE: 1.2 footprint requirement — 25% equates aggregate effective footprints under fixed shared demand; it is not necessary for a speed gain, and touched T is not the entire effective footprint. Preserve the Coordinator's >=25% reopening restriction as a decision, flag its unsupported necessity premise — Optimiser b6d99258 sensitivity model and Coordinator 6683240055; counterexample above.

ORACLE: weak-why section / 1.2 / 1.3 — 1.2's stated parent is 1, not 1.1; "none open" hides this contradiction and conflates scientific dependency with a real dispatch pause. Placement and cause-finding retain purposes after a failed qualification — seed at 01ea79a3; Coordinator 6683240055; #219 placement evidence.

ORACLE: Right now line 3 / item 1 — "tables, not threads" and "helpers didn't help" turn a scoped association and a failed 3% gate into stronger findings. Use exploratory configuration benefit, mechanism unresolved, neither primary qualified — #219 ratios/intervals and Researcher 6683028366.

ORACLE: Right now line 5 — "nothing else is in flight" conflicts with the seed's own open downstream 0.2.4 bump. Scope the wait to new performance dispatch or state the parallel front — seed item 2; Coordinator 6683240055 concerns dispatch, not closure of downstream work.
