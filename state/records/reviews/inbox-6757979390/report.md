# Rescue pass: remove the copy boundary; repack the dense pair fill

Inbox **6757979390**, dispatch of QUEUED6757843578. Specs **a2da49638ac54d7763de1c167bd054f67ffdc763**.
Research/source snapshot: PR313 **0a649c696db745ce83a1629381590dc9ded061f0**.
All constructions below are proposals. No decoder implementation, compilation, candidate execution,
new timing or safety certification. Arithmetic is reproducible in [arithmetic.py](arithmetic.py).
Local timings and simulated counts remain labelled as such; no raw profile replay was performed.

## Decision and order

Service remains first for adoption, emitter first for useful front-end work. D4-delta's accepted
counting-only probe remains the compact-table priority; its 15Ir/byte,7.35Ir/row,1.2MB filters
are unchanged. Do not create a rival full implementation queue from this rescue pass.

Among the new bounded opportunities, rank:
1. **BufRead through an owned output-block handoff**, Rust first, Python one-copy adaptation
   separately. Source establishes a removable copy; a consumer-specific CI comparison must price it.
2. **Dense pair table with split4+1 layout and wide, fully initialising stores**, as a construction
   count/fill probe reusing the LF-piece work, after the compact probe. It attacks an observed
   source cost and serves text where compact D becomes too large. No target timing slot yet.
3. **Chunk-parallel MTF with translation fused into run consumption**, conditional on the
   post-emitter critical path. Census/count first; no same-core interleaved-MTF implementation now.

No new build slot for scanner-chain interleaving, generic list contraction, the current phrase
walker, bandwidth hogs, dir, PGO or global v3. Reasons and reopening information are below.
“No rescue” here means no construction worth a new implementation on the present evidence,
not mathematical impossibility and not a closure of Louis's broader research line.

## 1. BufRead: a real copy boundary, overstated attribution

### What the source establishes

At this pin, src/read.rs Seq::read copies out of Seq.out; the parallel arm calls
StreamDecoder::read, which copies from reader.cur in src/parallel.rs. A caller consuming D
decoded bytes through these paths pays D logical bytes read and D written by this copy.
For the cited corpus D=209,700,196, that is419,400,392 logical bytes of copy traffic removed
if the consumer instead reads the lent block. Consumer inspection itself remains.

This is not true of every public entry point: sequential decode_with appends final block
output directly into its result Vec. It has no BzReader copy to remove. decode_parallel,
by contrast, currently reads StreamDecoder into a4MiB temporary then extends the result:
two output transfers after block production. A shared owned-block drain can remove the
temporary transfer there too, without claiming the final concatenation is zero-copy.

### What the cited ledger does not establish

The archived sim-pipe3-20 table is **mibz2-pipe3-both9**, not a universal ten-thread run;
sim-workers10 is a different plan/cache size. In the former:
- total Dw=1,182,227,023; combined memcpy Dw=221,687,097;
- combined memcpy DLmr=2,345,438 of3,656,744 attributed data read misses;
- the header's3,696,341 last-level read misses includes more than the data-only denominator.

**Dw counts memory-write accesses, not bytes.** D/1,182,227,023=17.7377% reproduces the quoted
17.7 numerically but is not a valid byte-share calculation. REP and vector stores have different
access widths. The snippets combine assembly, reader and input copies in memcpy; they do not
isolate the reader call arc. The claim “52% of cold reads” is therefore not reproduced from
these archived summaries. Retain it as an unverified attribution pending the caller-split
profile, not as an established saving. See Valgrind's event definitions [V1].

More fundamentally, lending **does not eliminate the consumer's need to read worker-produced
cache lines**. A cold source read performed in memcpy becomes a cold source read in the parser.
It removes the intermediate destination writes and rereads, and can change retention/eviction;
it does not make the worker-to-consumer transfer disappear. Valgrind's sliced-thread cache
model supplies neither measured DRAM traffic nor a lower bound valid for every real interleaving.

The quoted1.3 of26.7Ir/byte is about4.87% of instructions, or at best1.051× instruction-count
speedup if all that term vanished. Those totals are reported, not independently call-arc
replayed here. They are not a4.87% wall forecast. The consumer-copy roof at “about5GB/s” is also
unmeasured: even a crude f×IPC/1.3 varies with both inputs and omits the parser/hash.
For a pipeline, wall is governed approximately by max(scanner,workers,ordered-consumer);
an unexposed consumer saving can reduce CPU without moving wall.

### Construction and contract

**Sequential:** factor refill from copying; fill_buf returns out[at..], consume advances at;
Read uses the same cursor. Empty slice means EOF, not merely an empty current block.

**Parallel:** this is not a one-line BufRead impl. StreamDecoder exposes read(&self) with
reader.cur behind a Mutex and is held through Arc. Returning a slice after dropping its
guard is invalid; retaining a shared lock across arbitrary consumer work is undesirable.
Give BzReader's private parallel state ownership of one checked, ordered output Vec and
cursor, using a private dequeue/recycle operation. Move the Vec, not its bytes; return the
spent buffer to the free list only after consumption. Keep public shared StreamDecoder::read
semantics intact, and prohibit simultaneous private handoff and shared reading of the same
reader state. The feeder only writes input. Read and BufRead on BzReader must use the same
owned cursor, including partial interleaving.

Charge one retained output block plus bounded queue occupancy; no new unbounded pool. Slow
consumers, consume(0), early drop and cancellation must not deadlock or recycle borrowed
storage. Preserve ordered error delivery, persistent decoder errors, feeder I/O/panic
precedence, concatenated streams, stream-CRC checking and zero-length Read behaviour.
A borrowed block has passed its block CRC, not necessarily the enclosing stream's final CRC.
The consumer must drain to EOF to establish full-stream verification. No API may silently
turn early stop into “verified”. [R1] defines the borrowing/consume contract.

**Rust consumer cases:**
- fill_buf/consume parser or hash: eliminates the intermediate output copy.
- direct BufRead::read_line: still copies into String; it can eliminate the outer BufReader
  copy, not the String copy. Existing code that keeps BufReader<BzReader> need not benefit.
- read_to_end: still copies to the destination Vec; reserve/extend from filled chunks may
  remove repeated intermediate reads or initialisation, but zero-fill removal must be checked
  against this toolchain's actual default implementation, not presumed.
- a record spanning block boundaries needs a small carry buffer or two-slice consumer logic;
  owned records still require ownership somewhere.

**Python:** src/python.rs Reader.read allocates a zero-filled Rust Vec, copies decoded bytes
into it, then PyBytes::new copies those bytes into Python. Use the owned/borrowed checked
chunk to create PyBytes once, preserving up-to-size semantics and the existing GIL-release
behaviour while waiting. A standard PyBytes owns its payload; it cannot just adopt a reusable
Rust Vec. CPython documents copying creation [P1]. A memoryview would need an owning exporter
and buffer lease until all views die, changes retention/API, and is not this proposal.
Do not broaden this into a Python zero-copy claim or hold the GIL while blocking for output.

### CI falsifier

First counted copy bytes and allocation/caller attribution on matched consumers: Read+fixed
buffer; BufReader+read_line; direct BufRead+read_line; fill_buf parser with identical parse/hash;
Python read(size). Baseline and arm must do equal semantic work and hash all output.
Compare small reads, cross-block lines, huge RLE expansion, malformed later blocks/CRC,
concatenation, cancellation and mixed Read/BufRead. Bounded ownership and any unsafe code
need their stated B5 evidence.

Then a prospectively registered paired end-to-end consumer benchmark on Wikidata and text,
seq and parallel, with ordinary Read non-regression. Falsify the lending optimisation if it
still copies D bytes internally, leaks/reorders/recycles borrowed data, or consumer wall/CPU
and retained capacity do not justify the added path. An API addition alone does not waive
measurement design when claiming performance. No claim that it must beat the emitter or
service on today's1.5GB/s decoder.

## 2. The seven requested rescues

### (a) Overlapped MTF chains: move parallelism across cores, fuse relabelling

**No rescue for same-core interleaving now.** The53/79Ir per move and simulated branches
suggest substantial work, but “53/4 plus0.67×15 leaves no cycles” is not a hardware proof:
instruction width, uops, clock and branch costs are assumptions; costs need not add.
Interleaving might expose independent work, but all lanes still consume one core's resources.

The useful construction is the equivariant MTF formulation on **K chunk tasks**, not K lanes
on one core. Each chunk starts with the identity alphabet, emits symbolic run records, and
returns its final permutation Pk. Prefix-compose the true start alphabets Ak:
A(k+1)[j]=Ak[Pk[j]]. Translate a local output symbol x as Ak[x] **while consuming each run
record into the downstream fill/counts**, not in a second pass over the expanded column.
This removes the extra O(n) byte relabelling pass; O(r) dependent translation remains.
Use the actual alphabet size, at most256, not assumed byte identity for unused symbols.

Cuts must align complete RUNA/RUNB token groups, or carry their exact state; chunks also need
row offsets, output bounds and ordered errors. Pk construction still executes every move,
so this is critical-path redistribution, not reduced total MTF work.
With K4, initial alphabet storage is1024B and three prefix compositions visit768 entries,
plus task/scratch overhead and one mapping per run. Text r/n≈.300 means .300 mappings/row;
records≈.997, essentially one/row. Prefix, counts, queueing and memory traffic are not free.

Ideal span before overhead: M/K, versus M serial. From the local text stage5.47/20.30, K4
could at most remove20.2% of that old whole-decoder model. After the emitter's model reduces
MTF to1.80c/B, only1.35c/B remains removable at K4; it cannot claim the old ceiling as extra.
Idle helpers are required: taking cores from the critical walk can make the decoder slower.

**CI falsifier, conditional census first:** after the emitter, show scanner supply and team
waits exposing MTF, and spare helper capacity. Count O(r)+O(256K) induced work and compare
the same symbols with scalar-reference bytes/counts/errors. If no exposed MTF interval,
**no build**. Otherwise one fixed K4 cooperative arm against post-emitter baseline, full
front+build/whole decoder, text/Wikidata/records, not a kernel-only win. Rank behind BufRead
and the already commissioned D4 count.

### (b) Overlapped scanner chains: no new rescue slot

The source already has a two-symbol Huffman LUT. “Decode multiple symbols per lookup” is
not a new rescue. K independent bit streams duplicate accumulator, selectors, LUT state and
output cursors, risking spills and increasing useful work on the scanner core. The validated
finder/worker distribution is the better existing route; do not count its gain twice.

The claimed bound “under1.5× scanner therefore under5% text” is only an estimate. At the
reported3.17/20.30 share, a hypothetical1.5× scanner gives1.055× whole decode, about5.2% wall
reduction, not a universal less-than5% theorem. Macro-instruction IPC alone supplies no
hard bound on a different implementation or CPU. Still, no concrete eliminated work is
identified by adding lanes, and scanner distribution now has direct scoped evidence.
**No rescue as proposed.** Reopen only for an actual source/listing reduction in refill,
selector or decode work not already in the paired LUT, with scalar error equivalence and
scanner+whole-decoder CI falsifier. Not another timing of interleaved state alone.

### (c) Dense pair table: replace packed-overlap fill with split4+1

Source fill_pairs zeros5n+3 bytes and then stores8 bytes at stride5 once per row.
Its logical store volume is approximately **13n bytes**, although overlapping hot stores
are not13n DRAM bytes. It also pays one loop iteration per row. The local3.8×/5.4×/5.1×
build ratios describe this formulation, not a lower bound.

**Construction:** keep LF² pieces, but split each dense record into a u32 lane containing
target20 + first byte8 + metadata4, and a byte lane containing the second byte. Final size
stays5n=4.5MB at900k, not the compact-table goal. Fill target/first-byte words as an affine
vector progression, and fill the second-byte span as a repeated byte, once per piece.
Fully initialise reserved output before setting its length; do not zero then overwrite.
This reduces the logical initialisation/store volume from≈11.7MB to4.5MB/full block and
replaces n scalar overlapping stores by vector spans plus piece tails. A safe initial version
can retain initialisation if needed, but must price it; uninitialised storage needs proof.

The walk loads both lanes at the current row; those two addresses are available together.
It emits two bytes and advances to the masked target. Extra load/address and stop handling
can erase the fill saving. A final single byte and intermediate segment marker require a
one-step escape/record policy; the simple whole-block reference has no laned correctness proof.
Use the already reviewed D4 stop obligations rather than assume pair lanes can cross markers.

**Price:** exact break-even is Bpair−Bplain < Wplain−Wpair, measured consistently.
Using the local stage table and the unattained ideal Wpair=.5Wplain gives build-ratio
ceilings2.869 Wikidata,2.536 text,1.782 records. With text build2× and walk.5×,
whole-stage sum falls only6.60%; no advertised half-walk gain survives uncharged construction.
At text r2/n=.449, average piece length≈2.23, so wide-span utilisation may be poor:
histogram tails and instructions per piece first. At Wikidata average≈11.36, but compact
D4 already has the stronger capacity proposition. Five-byte dense tables can worsen the
many-worker knee; no extrapolated share-nothing win.

**CI falsifier:** reuse the piece constructor census; count fill stores, tails and full build.
Compare this split layout against the current pair fill and an equally wide/no-redundant-zero
ordinary fill so a general constructor fix is not misattributed to pairing. Then only if
inclusive build+laned walk has headroom, one registered stage/whole-decoder comparison.
Stop if piece overhead or the second load consumes the bound. Second new opportunity,
not a parallel target implementation.

### (d) List contraction: one-third is not the impossibility proof; no separate rescue

An independent set in a cycle can contain floor(n/2) vertices. One-third is a property of
some local selection rules (and often a lower guarantee for maximal sets), not the maximum.
But obtaining an alternating phase on an unknown permutation cycle can itself require the
traversal/ranking one hoped to avoid; invalid/multiple cycles and output expansion remain.

Let a contraction round cost b per remaining row, remove fraction f, and leave a walk costing
w per survivor. Before expansion/selection storage, saving requires b < f w. Across rounds
of constant f, contraction cost is b[1−(1−f)^k]/f per original row; traversal saving is
w[1−(1−f)^k]. Expansion and certificates tighten the inequality.
Thus b≈w/3 makes f=1/3 neutral before overhead, while f=1/2 could be positive—but no cheap
phase construction is established here.

**No rescue for generic sequential contraction.** The useful restricted construction is
already k-step LF composition on affine pieces (D4): perform composition on r_k rather than
every row, store emitted bytes, and charge the resulting piece count/build/marker work.
It is funded by the existing counting probe. No separate contraction build, no free
half-size table, and no double credit for D4.

### (e) Phrase-knee: no rescue of the withdrawn experiment or current kernel

The corrected certificate is same-cell, step by step, not constant displacement; keep aab
as a counterexample. The revised nearer-source census is.0972 events/row,15.3Ir/row for the
laned prototype, with4-byte state per output position plus bytes, memo and lane/frontier work.
The code still transforms/stores a state for each copied byte. Copying93% of bytes does
not eliminate93% of per-byte work.

A concrete simplification worth **paper pricing**, not another build: first-source only,
dropping the nearest/latest selection and latest memo update. The corrected global census
rises from.0972 to.1050 events/row (+8.0%). At150Ir/event that is15.75Ir/row; even100 gives10.5
before other accounting. It needs roughly40Ir/event to meet the entry's stated reopening
condition, and the deleted selection alone does not establish that reduction.
Transferring the global census to lane-local or frontier-restricted sources is invalid.

Range-min room certificates or phrase descriptors could avoid scanning/storing every state,
but arbitrary source endpoints then need state reconstruction and copying can refer to earlier
copies. Without a bounded dependency-depth/endpoint scheme this merely moves the cost.
No counted construction here meets40Ir/event; **no rescue/build on present evidence**.
Keep the corrected histogram and high-repetition alternative (n/r≥80) as an input-specific
opening, with an actually relevant workload and same-cell safety evidence first.
The withdrawn workstation recipe stays withdrawn. Local flat lane timing and simulated
branches do not prove memory irrelevant on the i9; neither does that uncertainty justify
spending a target run on this kernel now.

### (f) Bandwidth ceiling: rescue the estimand, prefer direct copy intervention

Researcher6755766766's counterexample stands: a spin-hog baseline can become CPU-limited
after reserving a core, hiding a bandwidth ceiling of the original full machine.
Pathfinder6755915645's revision controls an **isolated nine-core decoder**, not the shipped
ten-core system. Reserving one core in all cells makes baseline capacity comparable within
that experiment; it does not guarantee the bottleneck cannot change as dose changes.

NT stores and spinning do not impose equal package-power, fill-buffer, scheduling or
frequency costs. Even a compute-only multicore control cannot calibrate away all decoder-
specific effects. A slowdown is interference sensitivity, not proof of DRAM saturation;
a flat curve does not exonerate the unperturbed ten-core plan.

**Rescue construction:** a paired direct write-removal intervention, first BufRead on the
same consumer or the column-free emitter, with unchanged plan/input/output work. Count the
specific transfer removed; retain old whole-decoder controls. That tests the performance
value of this work without requiring a global memory diagnosis. It still does not uniquely
identify bandwidth rather than instruction/cache/coherence effects.

No new hog slot now, matching Coordinator6755932529. If a direct candidate passes CI but
its target effect stays unexplained, a prospective descriptive sensitivity design may use
no-hog/full-machine plus isolated no-hog/spin/doses, explicit plans and verified masks,
shutdown guards/deadlines, independent byte count, readiness and completion records.
Report both throughputs, power uncertainty and per-round response. Its falsifier is the
specified interference model under stated nuisance assumptions; no BOUND/NOT BOUND
production ranking or line closure. CI can falsify harness/admission/liveness, not the
i9's memory ceiling. C5 and fresh Researcher/Guardian review remain.

### (g) dir: no new rescue or third attempt

Preserve attributed CI LEAD and both VALID/INCONCLUSIVE target attempts with failed fidelity.
The last review at f416149e53ea5ff96fc8011de8e6ea2c2859a707 already prices a future opening:
new diagnostic-supported observation design, explicit reconciliation of the closed sequence,
exposed work beyond service/emitter, and fresh target/current-default plus off-target evidence.
This pass finds no new dir-specific eliminated cost that changes that ranking.
Using LF run-order fill removes sort/placement work through the emitter line; it is not a
renamed dir retry and cannot inherit dir's CI or target percentages. **No rescue now.**
Service+dir additivity remains unestablished.

## 3. PGO and whole-crate v3

**Neither merits a fresh CI timing slot now; neither local result proves universal null.**
The local PGO table even includes1.042 on records4t; v3 includes.920 on text4t. Nine rounds
of short processes without the raw paired uncertainty cannot certify equivalence or dismiss
the latter by assertion. Training and evaluation use the same three corpora, further limiting
transfer. LTO/one codegen unit reduce some opportunities but do not logically make PGO redundant.

There is stronger scoped evidence for v3: #201 at74ae634564aec76a57e4f6416507ea66d7cfb88c,
run42048643495, EPYC7763, registered rule closed. Its archived result shows scanner−2.4%,
build−4.5%, MTF+19.3%; no stage met its5% criterion. Preserve that verdict. Do not promote
its central estimates to permanent upper bounds or rerun the closed rule just because a local
cell looks promising. Global v3 also raises the baseline ISA requirement; baseline wheels
cannot rely on a run-time guard around code already compiled globally for v3.

A **conditional future PGO falsifier** is a fixed representative training profile with held-out
corpora/inputs, exact release flags/toolchain, profile-mismatch diagnostics, stage counts/listing
and paired whole-decoder CI including small inputs/MTF regressions. Charge profile collection,
two builds, release reproducibility and artifact maintenance. It becomes worth doing if a
new front end or consumer path leaves demonstrated branch/layout work beyond today's kernels.
PGO uses profile data for layout/inlining/allocation [R2]; it is not a constructor mechanism.

A future ISA probe requires a changed source-level opportunity, e.g. a measured hot operation
that the baseline listing expands and the supported feature simplifies. Count the reachable
whole-decoder share first; only then propose a separately registered guarded clone with scalar
fallback, not a whole-crate portability change. No such new opportunity is established here.

## 4. Evidence, limits and next action

Expected: recover worthwhile formulations from ideation-stage rejection. Found: a concrete
output-copy boundary, an actual dense-pair fill defect in formulation, and a narrower possible
MTF span reduction; several negative premises overclaimed hardware inference or causal
identification. New measured production gap closed: **zero**.

Next action: Coordinator design note for owned output lending and equal-work consumer
falsifier; keep D4's cheap count in flight; reuse its composition census before deciding
whether dense pair deserves a fill probe. Do not displace emitter/service, spawn seven
implementations, revive dir timing, or interpret the candidate list as integration approval.

Sources at the PR313 snapshot are recorded by blob in evidence.json. Source paths inspected:
src/read.rs, src/parallel.rs, src/python.rs, src/lib.rs, src/mtf.rs, src/block.rs,
examples/lf_fill.rs, examples/phrase_lanes.rs; by-data-class, borrowed-tricks, phrase-structure,
bandwidth-titration summaries and ISA-probe journals. The reader blob also matched master
when fetched. The simulation README pins its older binaries; these are not profiles of
the current source snapshot. Repository observations and reported archived counts are
separated from this review's algebra.

Primary API/event references, read2026-11-10:
- [R1: Rust BufRead](https://doc.rust-lang.org/std/io/trait.BufRead.html).
- [R2: rustc PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html).
- [R3: rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html).
- [P1: CPython bytes](https://docs.python.org/3/c-api/bytes.html).
- [V1: Valgrind event definitions](https://valgrind.org/docs/manual/cg-manual.html#manual-cg.cache-sim);
  [caller attribution](https://valgrind.org/docs/manual/cl-manual.html).

Migration old head and manifest digest matched before receipt. Pinned coord audit reports
the historical LOG6757065549 only, already corrected by6757075751/6757250382; no new dispatch.
Pinned verify assumes FROM coordinator; route/check and exact optimizer readback are recorded
without claiming verify passes for an optimizer. Persistent task unchanged.
