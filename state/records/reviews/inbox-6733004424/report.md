# Pair-step budgets and the limited tandem-repeat ceiling

Brief [6733004424](https://github.com/lmmx/mibz2/pull/160#issuecomment-6733004424). Census pin **292d65b9128b8f8d3d941b755885ab2349ebf8a3**; governing specs **a4f97714cc14366984015869d9d094d89488ac57**.

**Recommendation:** finish C29's production prototype and its evidence first. Preserve the existing matched leader-sort/direct-F investigation next. C37 is a secondary bounded consumer diagnostic; C38 follows it if the remaining instruction/dependency budget supports it. Do not queue a tandem-only Precopy implementation. General nonlocal Precopy is not bounded by the census's 10%, but the two historical failures leave it below these leads unless a materially cheaper detection mechanism is proposed.

## Evidence and scope

Read the pinned census journal, full tables and `examples/packet_census.rs`, especially `tandem_cover`. Journal blob **4bde57478df1819edde8542f111357baced3c98e**. The reported self-validation checks packets, trajectory and CRC; I did not rerun it or replay the gzipped per-block rows. This is a source-and-count audit and prospective economic analysis, not an independent certification.

For the 233 full Wikidata blocks, pair k≤8 repeated packets cover 738 of209,697,662 BWT rows. Certification saves314 loads under the deliberately generous cached-descriptor model: **0.0000014974 loads/row**, about0.0002994% of ordinary pair navigation loads. That cannot furnish the proposed34.4% work reduction under a uniform load-cost account. Pairing itself, not certified repetition, accounts for essentially all the load-count reduction. Reject this bounded packet model as the next Wikidata optimisation, without turning counts into a universal cycle impossibility or excluding other representations.

These are **pre-RLE1 BWT-output bytes/rows**, not decoded file bytes. The198 short blocks add8,895 rows; decoded Wikidata is209,700,196 bytes. The numerical difference is tiny here but is enormous on some synthetic RLE-heavy inputs. Cooperative markers and scheduling were not counted. Different marker sets are not necessarily nested: recompute their actual caps rather than assuming every cooperative packet is shorter.

The residual baseline comes from [the previous raw replay](https://github.com/lmmx/mibz2/blob/8382072cecf5d2932e3bb4c5548b05a9512d1162/state/records/reviews/inbox-6731656628/report.md), run42556998342 at1c4d0f0714183f83798a0b746463ae34535fbc50. Historical Precopy evidence is [issue9](https://github.com/lmmx/mibz2/issues/9) and [the forward-chain writeup at the census pin](https://github.com/lmmx/mibz2/blob/292d65b9128b8f8d3d941b755885ab2349ebf8a3/docs/journal/2026-10-27-precopy-forward-chains.md), blob ec5e9c68cc6b76f58ced375299e9984cc7eeb96e. No new implementation, compilation, tests, timing or safety certification.

## 1. What C37 and C38 must remove

Let R=6.072128323 and P=3.983783724 nominal cycle-equivalents per decoded output byte. They are **cross-sweep residual proxies**, not consumer measurements. Different block admission, scratch/cache history, setup and assembly boundaries remain in the subtraction. Their earlier repeat intervals do not become consumer-only confidence intervals.

| Proxy quantity | Per output byte | Per ordinary two-byte pair step |
|---|---:|---:|
| Current pair residual | 6.07213 | 12.14426 |
| Room residual target | 3.98378 | 7.96757 |
| Required net saving for parity | 2.08834 | 4.17669 |

To beat the room proxy, net saving must exceed4.17669 per ordinary pair step, or34.3923% of the residual. This two-byte conversion is a planning approximation, not a relabelling of BWT rows as decoded bytes. A real diagnostic uses emitted bytes and actual ordinary/odd/tail counts. With S ordinary steps, B output bytes and changed nonordinary overhead H, the exact account is `S*saving_per_step - H > B*(R-P)`.

**C37 alone has to clear the whole4.17669 budget; C38 alone has to clear the same budget.** Do not assign half to each by assumption. If a matched C37 diagnostic yields net saving x per step, adding C38 must remove more than4.17669−x from that new loop, with interaction and fallback costs included. Independent isolated improvements cannot simply be summed.

### C37: shared-offset staged pair rooms

The current pair path already has32 lanes, flags, next-row prefetch and two-byte stores. The removable work is per-lane pointer/end-capacity updates and control; a shared offset can amortise it across active lanes. A256-byte lane tile represents128 ordinary pair rounds.

Define g37 as gross common-step saving, h37 as added per-step staging/control, F as per-lane tile flush cost, and T as tails, compaction and odd LF handoffs amortised over steps. Its budget is:

`g37 - h37 - F/128 - T > 4.17669`.

The128 divisor applies only to full tiles; depleted lanes and short segments require their measured occupancy. Charge any extra copy, buffer residency or spill caused by the chosen layout. Flags and tail obligations do not vanish. A change in branch count is useful only if the resource it saves binds.

First falsifier, before any timing proposal: ordinary-loop instruction/dependency and dynamic path counts show that removable pointer/control work is too small after flush costs. If it cannot plausibly meet the budget, retain an inexpensive negative result rather than build a broader pair system.

### C38: two-bit symbol tags

Keep START bit31 and PAIR_LF bit30; bits28–29 can carry two symbol bits only with correct20-bit link masking. A high-symbol directory indexed by the256-row block retains about3,516 bytes of lookup state. Exceptional blocks need the exact fallback. At most63 exceptional blocks is a static row-space bound; it does not bound the frequency with which the segmented pair trajectory visits them.

The prize is removing the current first[] lookup followed by dependent f[c+1] checking on the common path, replacing it with tag extraction plus a high-directory lookup. **It does not remove all symbol loads.** The reported.002 correction-loop iterations do not bound the first mandatory bucket-end check, nor establish its elapsed cost.

With e the actual exception visit fraction, g the common-path saving and z the exceptional-path regression relative to the old loop, require:

`(1-e)*g - e*z - mask/tag/marker overhead > 4.17669`

for C38 alone, or the remaining matched C37 budget for the combined arm. Avoid double-counting mask/extraction if already included in g. At32 active lanes the dependency may already overlap navigation; deleting a load is not automatically a full load-latency saving. C38 is therefore the second diagnostic, not an asserted solution.

Both proposals need source/listing inspection that separates issue demand, load/store ports, spills and dependent latency/effective memory-level parallelism. Two bytes per step do not produce64 independent misses from32 lanes. Instruction counts cannot be converted to cycles by an invented IPC.

### A bounded experiment, after C29

Prospective arms: unchanged room, unchanged pair, C37, C38, C37+C38, all on the same eligible blocks and controlled initial state. Build tables outside the consumer boundary; reset flags/outputs identically between arms; account for preparation separately. Include odd segments, depleted lanes and periodic/multi-cycle inputs in correctness coverage. Record actual dynamic exceptions and flush occupancy. Keep warm reuse separate from decode-like cache conditions.

Freeze the comparison and statistics before collecting data; preserve C6's canonical grid/repeats/multiplicity and acceptance ownership. C5 leaves workstation execution with Louis. This is a proposal, not authorisation to run or alter a registration. New entry/marker ownership needs its own B5 evidence. The old repaired pair path does not certify a new consumer.

Even measured consumer parity would not repay the previous builder premium, about3.4574 nominal c/paired row. Whole entry requires `delta_build + delta_walk + delta_setup/assembly < 0`, and whole-decoder priority uses the slowest worker, readiness and overlap, not the sum of phase clocks. C29 changes production front work without replacing the fast room consumer; its narrower, already-supported direction outranks these unmeasured pair costs.

## 2. The tandem ceiling is not a general Precopy ceiling

The census marks the union of positions inside tandem repeats W^m, m≥2, with period≤8 or≤64, within its segments. It includes **the initial seed period**, not merely the bytes that can be copied after a seed is available. Overlapping repeats are unioned; they are not additive savings. Source readiness, minimum profitable length, copy scheduling and a certified resume row further reduce usable coverage.

The table gives f=.0610 and.1001 (rounded6.1% and10.0% in the journal). These bound a **bounded-period tandem mechanism**, not arbitrary repeated substrings at nonlocal distances. Forward Precopy can reuse an earlier equal substring without the intervening output being repetitions of it. Its historical31.5% copied share therefore does not contradict10.0%.

Content equality also does not provide the Psi state after a skipped region. A mechanism needs length and resume-state certification without first performing all the navigation it claims to eliminate. Re-running the full walk to discover content is not free detection.

### Break-even detector cost

Let s≤f be the fraction actually skipped, c_s the marginal critical-path cost of those skipped rows, c_c their copy cost, and D all added detection, metadata, certification, setup and control cost amortised over all BWT rows. Then:

`D < s*(c_s-c_c)`.

For detector cost d paid on walked rows, K chains with extra cost h each and setup A:

`(1-s)*d + K*h/N + A/N < s*(c_s-c_c)`.

These costs must use the same units and denominator. If copying removes already-cached cheap steps, c_s can be far below the average walk cost. The copy's source load and output store still cost work. Savings outside the critical worker do not necessarily reduce wall time.

For scale only, pretend s=f, copy/setup are free, every covered row costs the global residual average, and Wikidata's near-equal row/output denominator is exact:

| Period bound | Coverage | Maximum added cost/all rows using room proxy | Using pair proxy |
|---|---:|---:|---:|
| ≤8 | .0610 | .2430 nominal c/row | .3704 |
| ≤64 | .1001 | .3988 nominal c/row | .6078 |

If charged only on unskipped rows, corresponding ideal limits are.2588/.4431 for room and.3945/.6754 for pair. Actual seed, copy, metadata and readiness costs must fit **inside** these limits, not be added afterwards. These are optimistic sensitivity calculations, not measured detector budgets or universal timing bounds.

Even free skipping of10.01% at uniform pair cost leaves5.4643, above the3.9838 room proxy. It cannot close34.4% under that model. Nonuniform expensive-row targeting could change this; the present census does not show such targeting. Nor does its raw coverage license a10% whole-decoder speed claim.

## 3. What issue9 already ruled out

The offline estimates and implemented outcomes must stay separate:
- [6445611569](https://github.com/lmmx/mibz2/issues/9#issuecomment-6445611569):65.1% backward estimate, with per-row mapping overhead unresolved.
- [6446318226](https://github.com/lmmx/mibz2/issues/9#issuecomment-6446318226): backward prototype#30 rejected at+118% Wikidata.
- [6447489342](https://github.com/lmmx/mibz2/issues/9#issuecomment-6447489342): forward scheduling estimate44.7%, minimum32-byte chains and ready source/marker caps.
- [6449279556](https://github.com/lmmx/mibz2/issues/9#issuecomment-6449279556): implemented#36 copied31.5%, yet CI wall regressed18% at1thread and6% at4threads against0.1.4.

The writeup describes a neighbour-entry read, add and compare on every walked step, chain records and bit30 marks. Its inline local walk was30% slower with31.5% copied; zero-copy cases still incurred overhead. These are archived observations, not current-target measurements or independently replayed results here. The proposed explanation—copies skip adjacent cached steps while detection taxes every walked step—is consistent with the result, not proof that all repeat consumers must lose.

Simply recovering the old whole-entry regression requires eliminating18% of the old baseline's time, or15.25% of that candidate's time at1thread; it does not identify a detector cost in cycles. A new design cannot inherit the old source-availability estimate,31.5% coverage or timing on a different consumer. Its bit30 mark also conflicts with the pair path's PAIR_LF.

## Ranked handoff

1. **C29 production prototype and evidence first.** Preserve its checkpoint/short-run-store B5 checks and changed-path qualification. Standard low-thread CI neutrality is not proof of benefit on the pipelined branch.
2. **Existing matched leader-sort/direct-F work**, retaining the frozen transfer and its primary/secondary choices. D058 conditionally supersedes D055's#277 objection subject to frozen registration, greenCI and completed scalar/AVX2 Miri for the new empty case; no blanket timing clearance is inferred.
3. **C37 bounded prebuilt consumer diagnostic**, only after the removable-work budget is credible. New consumer result remains separate from whole-entry admission.
4. **C38 incremental/combined diagnostic**, conditional on C37 results and actual tag/exception counts; no unpriced34.4% recovery claim.
5. **Precopy remains outside the implementation queue.** A future general mechanism must first exhibit cheaper amortised detection/certification and useful ready sources, preferably targeting expensive steps. A small evidence-only cost/availability census could rank later if that new mechanism exists. Repeating the old all-step neighbour detector or implementing a tandem-only copier does not currently outrank C37/C38.

OBSERVER correction for forwarding: limit the6.1–10.0% statement to segment-capped tandem-repeat coverage at bounded periods, including seed bytes; it is not the general Precopy copied-byte ceiling. No new measured gain or cumulative instruction credit. All previous registered negative results remain.

Migration head and manifest digest matched; receipt committed before work. Routing audit found only five previously resolved historical items. This report does not adopt instructions from an unpinned protocol§11 or modify automation, acceptance or execution authority.
