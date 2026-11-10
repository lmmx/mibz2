# E closed; count D4 with a packed translation delta
Inbox6757808815. Measured source **84bf177bd0af9fa416964eca2c0f56ad278b119e**, PR323.
Specs **a2da49638ac54d7763de1c167bd054f67ffdc763**.
Proposal only; no implementation, compilation, new timing or safety certification.

## Evidence and disposition

Independently downloaded run42745744064 artifact13731585881; ZIP SHA256
3734059c48e66288bb0498a8f55cacbe27cd68c075691e6968358011cafda3ba verified.
Replayed all five head stage records, ordinary paired log ratios/t4, fastest-of-two passes
per registered stage. [Replay](replay.json) reproduces inversion3.66494 [3.59132,3.74007],
build1.82714 [1.77525,1.88056], walk4.34219 [4.25600,4.43013], and E32/flagged
2.95594 [2.89697,3.01610]. **FALSIFIED as built remains unchanged.**
These are stage estimands, not whole decoder or fleet results.

Registration/source were fetched at the measured pin. The result entry is absent there:
read its later PR diff at **b759304ee171ee316675f1a8cf8f3b455a977fb5**, blob
5bdbcd351275294e892981a0d0ac285aa0669211, explicitly separate from measured inputs.
Backward-table and share-nothing proposals read at **97e0067dc6defab2c77aeb1c2b5a6b1105f9ec69**,
blobs d88aed17f90d953f579f0f139dba92814fa8c3b3 and384d02082617f38b11173d627b5cc52080b26279.
Callgrind numbers below are their registered three-block summaries, not an independent
raw instruction/listing replay. Safety tests and Miri are reported, not rerun.

## 1. What bound E

**The step representation first; the skeleton second; build third.** Do not claim the counters
uniquely identify a hardware bottleneck.

- E32 has48.21 Ir/row versus room10.41: +37.80. The common step22–26 plus about22 lane
  bookkeeping accounts for the scale. Even deleting all22 leaves26.21, above15. A room-like
  skeleton costing about6 gives28–32, still1.87–2.13 times that reopen target. Skeleton work
  alone cannot rescue E.
- The matched bitmap comparison is61.63 versus34.16: +27.47 Ir/row, ratio1.804. Its measured
  time ratio1.843 corroborates a large representation/step cost even holding the broad
  skeleton fixed. E32/flagged2.956 likewise rules out blaming the entire loss on choosing
  flagged instead of room. Neither is a clean cycle allocation: E retains a separate starts
  bitmap, different dependency graph and data layout.
- Nominal median cost: walk16.87−3.87=13.00 c/B extra; build2.59−1.42=1.17. Roughly92% of
  this descriptive inversion penalty is the walk. Separately measured medians need not add
  to the reported inversion median or reproduce the paired estimator.
- The source's dependency path is line address → byte → broadcast/compare/mask → first-bit
  and slot-popcount → base selection/load; rank-popcount also feeds the target. One cache
  line is not one load instruction or one latency: multiple dependent accesses to that
  same line remain. Overflow is only1.82% of Wikidata rows, so overflow elimination alone
  cannot explain or remove a fourfold common-path cost.
- Build4.06Ir versus4.92Ir, but2.59 versus1.42 nominal c/B: time per instruction is about
  **2.21×** the scatter's. Source exposes rest→ctz→byte→eq→rest, a serial distinct-symbol
  loop, plus next[c] updates, copying/zeroing/packing and variable trip counts. This supports
  a dependency/control hypothesis, not a measured attribution to any one term. Sequential
  stores are not evidence of a cheaper build. A future diagnostic should count these sites
  separately and compare identical validation/output work, rather than assume instruction
  reduction implies speed.
- E's apparent48.21/16.87≈2.86 Ir/nominal-cycle versus10.41/3.87≈2.69 only says aggregate
  instruction throughput barely improved. It does not prove front-end/retirement saturation;
  the cycle labels use reported frequency, not PMU cycles.

**Footprint also fails:**2.039×900000=1.8351MB for table alone. Add112500 bytes of start
bitmap, lane buffers, output, column and allocator capacity. E does not meet a1.2MB full-block
goal even before its timing problem. The1.2MB budget is not an L3 residency guarantee.

## 2. Ranked constructions: D4-delta is worth a counting probe

Pathfinder REVIEW6757876056 at bb4d19e0a8a8f676d4e9a9326f55522baa33b84f withdraws C and
revises the inclusive paper estimates to C≈45, D2≈23, D4≈12, D8≈6 Ir/row.
Coordinator6757895217 limits D's next step to a cheap counting probe before timing.
These are paper extrapolations, not compiled D measurements. This revision supersedes this
report's earlier unpublished D2-first ranking; neither the initial hypothesis nor the new
ranking changes E's frozen verdict.

All sizes below use decimal MB, n≤900000, and reported piece densities as estimates rather
than guarantees. Charge actual capacity, tails, metadata and concurrent scratch.

### First: D4-delta — four bytes and the translation in one u64

On a piece starting at p with target base b, LF^k(i)=b+(i−p).
Store delta=(b−p) mod 2^20 instead of storing both p and b.
Then next=(i+delta) & (2^20−1). For a validated piece whose targets lie in [0,n),
this is the same target: b+i−p is in [0,2^20), and modular reduction preserves it.
The directory already identifies the piece, and current row i is already available;
the walk does not need p separately.

A record can hold delta20 + four bytes32 + length3 + flag1 =56 bits in a u64,
leaving eight spare bits. Length encodes one through four emitted bytes. This is an
algebraic proposal, not an implemented packing or a safety certificate. Construction may
still need explicit boundaries, interval ends and wider intermediate records. Do not
mistake the smaller final table for smaller peak construction memory.

Use separate arrays for64-bit boundary words,32-bit prefix ranks and64-bit segment flags:
20 bytes/64 rows, .3125n. The original directory's starts64+rank32+flags64 cannot fit16 bytes;
a pointer still requires charging its pointee. An aligned24-byte AoS is larger than this SoA.
Keeping the full segment bitmap makes the estimate conservative about that particular field.

At r4/n=.1088:
- final pieces plus directory: (8×.1088+.3125)n =1.1829n;
- at n900000: **1,064,610 bytes**, versus1,456,290 for12-byte records;
- for S512 markers, conservatively reserve2(k−1)S=3072 extra records for interior
  singleton splits:24,576 bytes, reaching **1,089,186 bytes**, before other capacity/metadata.

The bound is on additional piece boundaries for a valid permutation: for each marker and
j=1..k−1, its unique LF^{-j} predecessor can need a singleton, which introduces at most two
boundaries. Duplicates reduce the count. Malformed cycles, missing-marker termination and
bounded output still need the ordinary decoder invariants; no claim that markers alone
prove termination. For a shortened j-step record store LF^j's delta and its j bytes, not
LF^4's target with a partial store. Initial marker handling, reversed output order, lane
refill, final partial pieces and safe byte stores all count in the implementation obligation.

**What this removes:** one record field, the first-row subtraction, and the extra load/packing
work of9–12-byte records. It retains directory rank and the dependent record access. It does
not inherit E's instruction count or prove D's12-Ir paper forecast.

**Construction price:** the reported .197n doubling piece-steps at≈30Ir each cost≈5.91Ir/row.
That is just a model; full run input, directory production, singleton discovery/splits,
validation, allocation and retained scratch must be counted too. Against the proposed
1.5×4.9=7.35Ir/row construction ceiling, only1.44Ir/row remains for any terms omitted from
5.91. The estimated walk12 plus doubling5.91 is≈17.91 against production15.3, already
about17% more instructions before omissions. The bet is lower whole-decoder contention at
many workers, not a one-thread instruction win.

**Cheapest CI falsifier, proposed to the Coordinator:** count the full doubling construction
and one laned k4 walker with the stop fix in the common room-style skeleton, on the registered
three blocks before timing. Also archive corpus-wide piece counts, allocated footprint and
fallback frequency, plus adversarial marker/tail correctness. Reject this candidate if the
inclusive walk exceeds15Ir/output byte, construction exceeds the separately registered
7.35 ceiling, or final table capacity exceeds1.2MB on the admitted blocks. Charge setup and
short steps per byte; ≤60Ir/full four-byte step is not sufficient when short steps are common.
A passing count earns only consideration of a prospectively registered build+walk and
whole-decoder comparison. No timing or production build slot is authorised by this report.

### Second, conditional only: D2-delta

The same encoding fits delta20+bytes16+length/flags in a u64, with the same directory.
At r2/n=.088, size≈.91485MB before exceptions/capacity. One doubling pass costs less than D4,
but the revised≈23Ir/row paper walk misses15. **Do not build it in parallel.** Retain only if
the D4 probe shows a shared room-loop simplification can bring the whole pair loop below
30Ir, including tails/setup per emitted byte. That is its CI falsifier; smaller size alone
is not a reason to reopen. At text r2/n≈.449, even this table is≈3.51MB, so fallback/admission
is essential. Report corpus-specific densities rather than transfer Wikidata's census.

### Withdrawn or behind the probe

C remains withdrawn as a walk table; no C build is requested. Its .80109MB estimated table
does not rescue a one-byte dependent lookup. C's direct construction is useful only as an
accounting reference for initial runs, not another experiment.

D8 stores delta20+bytes64 plus metadata, exceeding one u64;16-byte records at r8/n=.1422
plus this directory are≈2.329MB. It misses the budget before exceptions and retains more
doubling work. E with room bookkeeping still misses both count and size. Dense B and
24-bit per-row tables likewise miss1.2MB for a full block.

**Ranking against other work:** service remains first for adoption closure; emitter reduces
useful work on the serial front and remains ahead of a full D implementation. Within the
compact-table investigation, one bounded D4-delta counting probe is economically preferable
to another E timing or a parallel C/D2/D8 implementation.

## 3. Share-nothing survives as a hypothesis, not the2.7–3.4GB/s forecast

E falsifies one representation. It does not falsify eliminating cooperative work with a compact
table; D4-delta has distinct costs and footprint. Keep the existing small-block pricing recipe
as a bounded diagnostic after the applicable two-mode review/C5 admission. Service remains
first for adoption closure, emitter next useful-work opportunity. D4-delta is the next small-table
construction worth counting; do not displace those with another E timing.

The historical1.44=CPU10/CPU4 at level3 is **a contention-growth observation, not a derived
allowance for a new walk's one-thread slowdown**. It compares worker counts on one representation;
it does not compare the cooperative default with a new compact kernel. Do not turn1.44×10.4≈15
into a performance theorem. Honour the Coordinator's15-Ir reopen requirement as an additional
prospective filter; keep its rationale distinct from measured speed.

For a first-order CPU model, write:
C_new(P)=F_new(P)+B_new(P)+W_new(1)g_new(P)+queue/assembly_new(P).
Compare against the measured cooperative C_base(P), with matching bytes/workload.
For wall, compare scanner/input supply, slowest worker completion and ordered-output stalls;
aggregate CPU alone cannot determine the max. The archived38.0−26.8=11.2Ir/byte opportunity
is not11.2 cycles saved or a guaranteed42% speed gain. Valgrind serialises threads and can
change polling. The parallel scanner is another candidate, not a free already-qualified term.

The proposed -2 versus -9 sweep changes block count, compression ratio, symbols/row, runs,
per-block setup, scheduling and scan demand as well as table size. Same decompressed bytes
are necessary but do not isolate cache. Require same-build input/output digests, row/block/run
counts, whole-process CPU and wall, observed worker/plan/mask and pre-treatment state,
plus within-level C(P)/C(2) or another frozen baseline. Report the level×worker interaction;
do not equate a .8MB level2 table with a .8MB level9 compact representation's total live set.
Count retained column, run buffers, output queues and peak concurrent scratch. CPU can still
grow from scanner/queue/spin behaviour; no mode exclusion from outcome times.

A positive -2 scaling result supports the information objective: smaller working sets make
independent workers plausible. It neither qualifies C/D nor proves the parallel scanner will
supply ten workers. A negative result should identify supply, fixed per-block cost or capacity
before closing the compact-table line. The run is valuable because it separates the architectural
premise from the new-table engineering, at bounded cost; it does not authorise another run.

Expected E≤2× inversion; observed3.665×, both build and walk worse. No production gap closed.
Learned: per-row line decoding overwhelms locality, while the purported compact table remains
too large. Next information value: count D4-delta's full four-byte loop, singleton handling, construction
and exact footprint before further target expenditure.

### New par-scan evidence: premise strengthened, attribution still limited

Coordinator LOG6757860786 reports par-scan/20261110T030608Z at
efc1bd88c73b07d917617eddfdf008dffc691e3d, archived at
727674e185744b91077dd4a862f2cd9e1485f0d0 (digest prefix140a0e5b).
This report read that result, **not its raw bundle**, so it does not independently certify
VALID or replay its paired estimates.

On the i9 mask0–19, reported Wikidata level9 throughput falls from about1145/1127MB/s
(scan/find) at4 workers to631/619 at10; CPU grows .58→3.69s. Level1 reaches1598/2563
at10 versus1524/1255 at4, with CPU .74→.84s. The reported paired finder/scan ratios are
.981 on level9,1.615 on level1 and1.780 on records at10.

This strengthens the case for **one cheap compact-table counting probe**: supply can scale
with a parallel finder when the per-block workload is favourable. It does not establish
that footprint is the sole cause, that D4 reproduces level1's cost, or that the default
cooperative decoder would lose to D4. Five rounds, uncorrected cross-input comparisons and
the unresolved two-mode regime remain. Retain the reported NOT FASTER/FASTER readings in
their registered descriptive scope; no production qualification or new target run follows.

The proposed -2 pricing diagnostic retains information value only for what the existing
level1/level9 observation leaves unresolved. Prefer its predeclared near-knee question,
matching CPU/work and whole-decoder references; do not repeat a broad sweep merely to
re-observe that small blocks scale. Pathfinder's new CPU10/4≤1.3 suggestion is prospective,
not a threshold retroactively applied to par-scan.

## Limits and protocol
No raw callgrind replay, new test execution or independent B5 approval. The raw stage archive
is the only timing evidence replayed. Pinned routing audit still flags LOG6757065549; Coordinator
6757075751 and6757250382 already record its correction. No re-dispatch. Migration head and
manifest matched before receipt. Pinned verify hardcodes coordinator FROM; route/check and
exact readback are used without claiming an optimizer verify-pass. Persistent task unchanged.
