# E closed; try two-byte pieces, not a smaller version of E
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

## 2. Ranked constructions

All sizes use decimal MB and n≤900000. All instruction targets include step, lanes, segment
handling and assembly amortised per emitted byte, not an isolated lookup. None is measured.

### First: D2 — two-byte pieces in one u64

Compose LF twice over runs; each piece has constant two emitted bytes and an affine next-row
map. Record first20 + target-base20 + bytes16 =56 bits, leaving8 bits for a length/stop tag
and representation metadata. One64-bit record load replaces E's byte compare, slot selection
and two popcounts with directory rank plus affine translation. Unlike D4, it needs only one
doubling stage, and it amortises the lookup and lane update over two bytes.

Use separate arrays for64-bit piece-start words,32-bit prefix ranks,64-bit segment-start words:
**20 bytes/64 rows**, not an unexplained16-byte directory. The advertised C directory's
64-bit starts +32-bit rank +64-bit segment flags cannot fit16 bytes. A pointer fits but its
pointee still costs space. Packed SoA gives.3125n; an aligned24-byte AoS costs.375n.

At reported r2/n≈.088, D2 uses8r2+.3125n≈**1.0165n=.91485MB**.
Allow split exceptions, final partial word, metadata and actual Vec capacities. This is a
corpus-density estimate, not a bound for every block. Admit only when the computed footprint
meets the frozen cap; otherwise retain ordinary decoding.

A k-step must stop before crossing a segment boundary. For k2, a row whose first LF step
lands on a start needs a one-byte transition with its own target and length tag. Split its
piece as necessary; each exceptional singleton adds at most two boundaries, so reserve up to
2S extra records for S starts (≤8192 record bytes for S512), before other metadata. Handle
short tails and arbitrary permutation cycles, not only valid corpus cycles. Reversed pair
stores must preserve forward output order at assembly. These are new proof/test obligations.

**Count target:** ≤30 inclusive instructions per full pair corresponds to≤15/byte only if
almost all pairs are full. Actual bound is I_total/output_bytes; partial pairs, boundaries
and setup reduce the allowance. Construction, retained emitter records and peak doubling
scratch are charged independently. No speculative claim that the listing already achieves30.

**CI falsifier:** first a counted implementation/listing, common room-style skeleton, all
corpora and adversarial boundaries byte-exact. Fail the candidate if table/capacity or inclusive
count target is missed. Then separately registered stage comparison on Wikidata AND text,
build+walk against production, with predeclared threshold and no selected fastest lane count.
If extra build work exceeds measured walk saving, redirect; no automatic i9 run.
Text's reported r2/n≈.449 makes this about3.51MB: D2 is not a compact text solution.

### Second: C — one-byte run directory as the low-build-cost control

8r+.3125n at r/n=.0722 gives **.8901n=.80109MB**. Build directly from emitted runs and counts,
no per-line distinct-byte discovery and no sort. This most directly removes E's serial build
formulation and leaves headroom for flags/metadata.

Cost: directory → popcount → dependent run-record load → translation each byte. The source
proposal's≈17-instruction step is not proof of a≤15 inclusive loop. Retain C as the construction
control for D2: D2 must repay its extra composition and records through fewer walk operations.
A possible packed modulo-2^20 translation replaces base−first arithmetic, but is only safe
after proving each selected interval's LF target is in[0,n); masking cannot substitute for
validation. It saves no dependent directory lookup.

**CI falsifier:** count whole loop including directory/flags; if it misses the stated reopen
target, do not call it accepted because the table is small. A forward, separately reviewed
information experiment can compare its build+walk economics if justified; it does not reopen E
or quietly relax C's own registered bound. Record high-density fallback and its share.

### Third: D4 only if packing and stop metadata actually fit

The published12-byte records at r4/n=.1088 cost12×.1088+.3125=**1.6181B/row=1.45629MB**,
already over1.2MB. Even the original .25n directory gives1.40004MB.
Packing first20+base20+fourbytes32 into9 bytes gives1.2917B/row=**1.16253MB**,
but leaves no tag bits. Stop lengths, singleton splits (up to2(k−1)S boundaries),
alignment, overreads and capacities can exhaust the≈37.5KB headroom. Nine-byte records also
require multiple loads, some crossing cache lines. Do not inherit the “one record load”
model or claim it is below the knee without counting these terms.

**CI falsifier:** exact per-block maximum footprint and doubling peak first, inclusive
instructions per emitted byte second, total build+walk third. If either footprint or total
cost fails, prefer D2; no k8 detour (its known footprint is larger).

**Not ranked for build:** E plus room skeleton. It neither meets the footprint nor the
count target. Further budget spent shaving its bookkeeping is economically inferior to
amortising the lookup with D2. Dense B and24-bit per-row tables likewise miss1.2MB on full
blocks. Smaller source blocks are a diagnostic/reference, not a decoder representation
available for arbitrary existing level9 input.

## 3. Share-nothing survives as a hypothesis, not the2.7–3.4GB/s forecast

E falsifies one representation. It does not falsify eliminating cooperative work with a compact
table; D2/C have distinct costs and footprints. Keep the existing small-block pricing recipe
as a bounded diagnostic after the applicable two-mode review/C5 admission. Service remains
first for adoption closure, emitter next useful-work opportunity. D2 is the next small-table
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
too large. Next information value: count D2's two-byte lookup and exact footprint, with C as
the low-construction-cost control, before any further target expenditure.

## Limits and protocol
No raw callgrind replay, new test execution or independent B5 approval. The raw stage archive
is the only timing evidence replayed. Pinned routing audit still flags LOG6757065549; Coordinator
6757075751 and6757250382 already record its correction. No re-dispatch. Migration head and
manifest matched before receipt. Pinned verify hardcodes coordinator FROM; route/check and
exact readback are used without claiming an optimizer verify-pass. Persistent task unchanged.
