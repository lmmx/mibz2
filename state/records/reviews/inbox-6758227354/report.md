# Price: stage the cooperative traverse, without staging the team

Inbox [6758227354](https://github.com/lmmx/mibz2/pull/160#issuecomment-6758227354).
Specs: a2da49638ac54d7763de1c167bd054f67ffdc763.
Claim: Pathfinder REVIEW 6758209736.
Source: 2c70fd13ebc8565cbf1eab82536d72cfb2c84cfe; coop.rs and bwt.rs also match
master ae44497e99765f3f1570c749ef5b381d7d8799a6 exactly.

## Decision

**Recommend a bounded counting prototype of a private per-walker staged traverse**, after
the transition proof and adversarial correctness cases below. No timed build or integration
approval here. Falsifier 1 is now read; falsifier 2 is the right next empirical step **with
inclusive counts, matched segmentation and transition counters**, not just a function-self
count against 17.6.

This moves ahead of a new dense-pair implementation in the kernel-probe ranking: it changes
no navigation representation, attacks the currently used cooperative walk, and has an
existing working single-worker analogue and historical CI mechanism evidence. Lending's
caller-split count is still cheap and independent. Service adoption, emitter work and the
already running D4 count remain intact.

The bulk staged loop is reachable inside a team. Mid-pass acquisition is not a structural
obstruction and needs no global chunk barrier. **10–11 inclusive Ir per row is a credible
hypothesis, not a demonstrated price.** The dominant uncertainty is the distribution between
the full-lane loop, flushes and draining tails under real segment ownership/late leader joins,
then the effect of burst stores on the critical worker. A handful of segment atomics is not
an explanation of a per-byte 7.5-instruction gap by itself.

## 1. What the record actually establishes

### Source

coop::traverse<32> still uses Lane state in the inner loop and writes directly to
base[k]+off in a segment buffer. Current bwt::room_segments<32,1> uses separate rows,
a 32-by-256 tile, eight expanded lane steps at a time, and a marked-lane mask processed
after an offset. It still tests START in each lane step. Calling it a branch-free loop,
or saying staging removes prefetch, would be wrong.

Current source is not literally #171's old speculative two-row-array implementation:
#171's archived spec_offset-P0.asm and call arcs describe that historical version.
Port the present safe transition structure first, not an unexamined mix of old speculative
code and today's queue.

### #171: transferable mechanism, not team evidence

At #171 head 23011895c32523c3967f5ba652d762e0aae91ff5, the benchmark's mibz2 arm calls
BzReader::parallel, and Plan::for_threads uses plain workers below six available cores.
The four-vCPU/two-core runs therefore **did use independent workers**.

The archived EPYC 9V45 summaries report four-thread CPU reductions of 17.6% and 16.3%,
with one-thread wall effectively unchanged. Preserve the original result and its reported
unresolved harm signals on runs; do not rejudge its old rule. These observations motivate
staging under contention, but do not measure a shared-table team, current default, or i9 gain.

I replayed the archived block-1 Ir call arcs, without executing a decoder:

| Historical scope | Base | #171 | Difference |
|---|---:|---:|---:|
| measured, inclusive | 20,184,311 | 19,051,749 | −1,132,562 |
| room_segments, inclusive | 9,859,793 | 8,727,231 | −1,132,562 |
| room_segments, self | 9,588,849 | 2,218,141 | not a like-for-like loop saving |
| both spec_offset callees | absent | 6,104,616 | part of the staged walk |
| direct room_segments → memcpy arc | absent | 125,321 | flush cost |
| whole-profile memcpy self | 223,168 | 348,489 | all callers, not just flush |

The historical journal calls roughly 0.35M memcpy the flush. The direct flush arc is
0.125M; 0.348M is aggregate memcpy. This is a caller-attribution correction beside the old
record, not a change to its verdict. The replay exactly reproduces its measured total.

### The 17.6 / 10.1 / 38.0 claim needs a matched denominator

The checked-in bandwidth files contain DLmr, DLmw and Dw summaries, **no Ir event column**.
They do not let this review reproduce Pathfinder's new 17.6 and 10.1. The share-nothing entry
at the source pin instead reports 17.8/10.1 per decoded byte. It is a reported count, not a
raw replay here. Ask the counting probe to carry the actual profiles, command, binary/source
pin, event list and caller-split inclusive attribution.

In particular, sim-pipe3-20 is the explicit mibz2-pipe3-both 9 arm with an older decoder
snapshot. It is not a fresh measurement of every present default invocation. Its single
worker comparator has a different last-level simulation model. Cache model differences do
not themselves create instruction differences, but threaded scheduling/waits, plans and
binary identity still need controlling.

Let N be table rows, M be segment bytes actually walked, and D be decoded output bytes after
RLE1. They are not interchangeable. On periodic/multicycle inputs M can be below N; assembly
repeats the selected cycle to N, and RLE1 changes N to D. Different start sets can also visit
different off-output cycles. Both the source and count evidence must identify which denominator
is used, rather than subtract Ir/row from Ir/output-byte.

If all three reported numbers use exactly the same D and complete walk attribution, then
38.0−17.6+10.1=30.5 is correct arithmetic. It would remove 19.7% of total instructions and
66.4% of the reported 11.3 Ir/B excess over 26.7, leaving 14.2% excess. That is the proposed
outcome, not evidence that staging achieves it, nor a 19.7% wall forecast.

## 2. Concrete construction and its proof obligations

Keep RoomWalk's table, starts, segment buffers, successor array and atomic taken counter.
Each invocation of share gets its own tile, rows and per-lane staged-start offsets.
Nothing in this working state is shared between walkers.

### Mid-pass take

For lane k, p denotes the first uncommitted destination byte. st[k] denotes the first
uncommitted byte in its tile row. At offset j:

1. A non-marker step writes tile[k][j] and advances rows[k].
2. A marker leaves rows[k] at the marker and records k in the local mask.
3. Process marked lanes in increasing local lane order. Copy tile[k][st[k]..j] into p,
   advance p, then finish the old segment (length and successor).
4. Claim the next segment with the existing Relaxed fetch_add. On success, take writes its
   first byte directly to the new segment buffer, as today; set rows[k] from the new Lane
   and st[k]=j+1. Do not copy a nonexistent tile byte at j into the new buffer.
5. At chunk end, copy each current lane's tile[st[k]..w] and advance its p.

Multiple mid-chunk takes simply flush multiple disjoint fragments. They do not need a shared
tile, another atomic, or a team-wide flush. Keep the first-byte exception; removing it would
need a different ownership/room proof for little expected gain.

The room bound remains valid: room is the minimum current spare, capped at cap−1 for a
newly acquired segment after take's first byte. After a take at done+j, fewer than room
offsets remain, hence no more than cap−1 bytes are staged for that new buffer in the pass.
Flush before any grow/reserve and refresh p/end afterwards. Do not retain a destination
pointer across reallocation.

### Queue exhaustion: the important changed edge

Once a Relaxed claim returns s>=ns, no unclaimed segment remains, though other workers may
still own many. Finish and flush the current marked lane, then drain this worker's remaining
lanes. **Do not copy the old unstaged “indices before k stepped, indices after k did not” rule.**
In the staged construction, all unmarked lanes have already stepped at j.

For every other lane, the flush endpoint is:
- j for a still-pending marked lane;
- j+1 for an unmarked lane;
- j+1 for a previously handled marked lane that took a new segment, whose st is already
  j+1, so that interval is empty and its first byte remains the direct take write.

Copy rows back into Lane state before tail entry. Flush all committed bytes before publishing
a segment length. A lane ending exactly full needs the existing grow-before-tail-write rule.
The ordinary direct-writing tail can remain unchanged for the first arm.

### There is no unfinished-segment handoff between team members

The counter transfers **unclaimed segment ownership**. A claimed segment stays with its lane;
other members do not read its partial bytes or take its unfinished suffix. Deferring local
marker handling can change which worker claims the next segment, so it can change balance,
but not the output contract.

Relay::walk_and_leave calls share before its AcqRel walkers decrement. Make share return
only after all local tile data is committed and all owned segments are finished. The last
decrement can then unflag and assemble exactly as before. Pool completion provides the
corresponding visibility for the non-pipelined path. No per-chunk Release, fence, ready flag,
or new barrier is needed. A late leader still joins using the existing nonzero-walkers CAS;
it never reads another worker's tile. Preserve panic/failure handling: partially committed
work after a panic is not valid output.

## 3. Whole-construction price

### Storage and live state (64-bit sizing; compiler frame still to be checked)

| Item | One 32-lane walker | Three active walkers in one team |
|---|---:|---:|
| Tile, 32×256 u8 | 8,192 B | 24,576 B |
| Separate rows, 32 u32 | 128 B | 384 B |
| st, 32 usize | 256 B | 768 B |
| Existing Lane array (row, seg, p, end; ordinarily 32 B each) | 1,024 B | 3,072 B |

The existing 256-byte base-pointer array is displaced by the staged addressing. Thus roughly
8,320 net new array bytes per walker is a useful source-level estimate, excluding alignment,
spills, scalar counters and compiler lifetime overlap. The eight-lane unroll is not a second
8 KiB tile per group. Three teams of three walkers have up to **72 KiB of tiles**, not 8 KiB.
Two SMT siblings walking simultaneously expose 16 KiB of tiles to one core's data cache,
plus everything else; “fits in L1” is not a proof of no pressure.

No additional O(N) table or output buffer is needed. Existing segment buffers and final
assembly remain. At N=900,000 and exactly S=1,024 starts, cap=floor(2N/S)+64=1,821 and
S×cap=1,864,704 bytes of requested initial segment capacity, before allocator rounding,
retained capacities and grows. This existing footprint is not removed by the tile.

The segmentation differs:
- plain cooperative K=3 requests 512K=1,536 starts;
- pipelined K=3 requests 512(K−1)=1,024 starts, because two helpers start while the leader
  performs other work and may join later;
- one ordinary worker requests 512.
Use actual deduplicated S, not requested S. Keep these policies unchanged in the first
production-shaped comparison.

### Work and traffic ledger

For one block, let B be full-phase staged bytes, T tail bytes, S successful takes, W share
invocations, C chunks, P room passes and F nonempty copied fragments. Then M=S+B+T<=N.
Every successful take contributes its direct first byte.

- **Claims:** S+W atomic fetch_add operations on a normal completed run: S successful and
  one exhausting claim per share invocation. Same count form as today, no per-byte atomic.
  At N=900,000, S=1,024, W=3, that is 0.001141 claims/row. It is sparse but the single cache
  line's ownership transfers have latency not represented by the count alone.
- **Marker completion:** S successor searches/length publications, using binary search over
  starts, O(S log S). Shared start density, extra cycles visited and tail policy make this
  a genuine part of cooperative work.
- **Chunk overhead:** rows/st bookkeeping and up to L chunk-end copies per chunk, plus a
  fragment at each mid-chunk completion; F<=LC+H, where H is full-phase segment completions.
  Zero-length fragments should contribute no bytes; do not call them full-size copies.
- **Traffic:** every staged byte still reaches its final segment buffer once. Compared with
  direct writing it additionally writes and reads the tile: **2B logical bytes**, plus tile
  initialisation (up to 8,192 bytes per active full-phase invocation if retained by codegen).
  These are logical accesses, not a prediction of extra DRAM bytes. L1-hit tile stores can
  defer the final-buffer miss stream to a burst and release miss resources for table loads.
  It is a contention/locality trade, not elimination of final writes.
- **Setup/build/assembly:** table construction, marker selection/sort/flagging, existing
  allocations, unflagging, assembly, RLE1 and CRC are unchanged in algorithm. Timing can
  still change through contention and different thread completion order. Charge them all.

A useful counted model is:

I_walk = i_B B + i_T T + i_S S + i_P LP + i_C LC
         + i_F F + I_copy(B) + I_atomic(S+W) + I_setup + I_finish.

Coefficients must come from the exact listing/profile; caller-inclusive measurement must
not add memcpy twice. Record critical-worker B/T/C/F and CPU separately from team sums.

### Branches, tails, and the achievable gap

The current staged body retains per-lane START tests; it moves finish/take out of that body
and uses a mask and trailing-zero loop for marked lanes. It does not automatically remove
32 branch tests per offset. Eight-way expansion changes addressing and loop control; spills
and branch layout require inspecting the built listing.

Markers get denser as S/N rises. A rough single-cycle, independent-position model at
S/N=1,024/900,000 predicts a marked event in about 3.6% of 32-lane offsets, versus about
1.8% at S=512. This is an illustrative distribution, not a branch-miss prediction or bound.
Periodic cycles and selected starts can be very different.

After the queue empties, each worker drains only its own remaining lanes; slow chains
cannot be stolen. The leader may arrive too late to fill 32 lanes and then receives **no
full-phase staging benefit**. Tail bytes, long chains, false sharing of Vec/successor
metadata and fragmented flushes can keep the inclusive price above the single-worker 10.1.
These are countable residuals. Extra segment topology is part of the plan; it is not proof
that its cost is irreducible.

For any actual busy cost b, tail cost t, residual h and tail fraction f, the simplified
weighted cost is b+(t−b)f+h. Reaching a target q requires
f <= (q−b−h)/(t−b), when t>b. Do not insert 10.1 as b or 17.6 as t: both quoted figures are
whole-function averages, not those components. The missing distribution is exactly what
the counting probe must supply.

The present data justify **“a substantial avoidable loop cost exists”**, not “two-thirds of
cooperation's extra work is proven removable.” Hardware sharing costs can remain even if
Ir converges. Conversely, failure to reach 10–11 Ir/row does not disprove staging's possible
contention benefit; #171 itself demonstrates why instruction arithmetic and wall can diverge.

## 4. Cheapest discriminating next step

### Before executing the prototype

Use the source transformation above, same 32 lanes/256 tile/current prefetch and same segment
policy. No batching of segment claims or new scheduler in the first arm. State the safety
argument before unchecked accesses; an independent B5 reviewer is required for the resulting
unsafe change, and partial Miri is partial coverage.

Craft transitions, not just random byte equality:
- multiple marker lanes in one offset, with only one shared claim left;
- another worker exhausting the queue between two local marked-lane claims;
- exhaustion at chunk start/end/pass end, including an unmarked lane after the exhausting
  index that already stepped;
- several takes in the same lane/chunk; a take at j=w−1;
- grow after a flush, tail entering full, S<L, periodic multicycle tables;
- leader joining after most/all claims, and panic before/after a partial flush.

Require each transition to be observed, compare **every segment and successor** with the
existing oracle as well as assembled output, and mutate st=j+1, waiting-mask selection,
flush-before-length, stale growth pointers and flush-before-last-decrement. Buffer-only
mutation detection is insufficient for the last concurrency obligation.

### Revise falsifier 2

A bounded correctness/counting prototype is warranted. Three ordinary full blocks alone
cannot establish the tail or race contract. Use three preselected full blocks (e.g. historical
1, 41, 233), plus tiny/crafted/periodic correctness cases; freeze identities before counts.

Compare old and staged **cooperative** paths with matched starts, table and segment policy.
A single-caller control with the same S isolates loop construction from multiworker queue
traffic; actual K=2/3 and late-leader runs establish which transitions/schedules occur.
Callgrind serialises threads: it can assign most work to one helper or leave a late joiner
empty, so aggregate low Ir is not proof of useful three-worker balance.

Archive, per variant and block:
- N, M, D; successful/failed claims; actual S and active walkers;
- B/T per worker, chunk lengths, F/copy bytes, grows and marker events;
- inclusive walk, setup/finish/flush/allocator calls, whole inversion and decoder Ir;
- branch events if collected, exact event definitions, source/binary/toolchain and listing;
- same bytes, segment successors and admission/safety coverage.

The fastest rejection is discovering that the counted “team” never engaged or one walker
did everything, or that omitted callees erase the saving. Those reject the measurement,
not the advantage. A matched complete count can falsify 10–11 or the 30.5 extrapolation;
it does not alone falsify contention relief.

If the private full-phase construction saves work but tails dominate, a second, separately
priced construction is small survivor groups with the same per-owner tile until drain;
it adds mask/packing/control and cannot remove the final single dependent chain. If flush
fragments dominate, first determine whether short segments or queue timing cause them.
Do not leap to cross-worker stealing, another shared buffer, or a tile-size sweep without
that finding.

### Subsequent evidence and whole-decoder economics

Only after the source/count result supports a mechanism, register a paired CI team arm
(e.g. forced coop2 at four threads) against the identical unstaged team, with ordinary
worker controls. A default four-thread mibz2 run selects workers and would miss this change.
Also exercise the pipelined team; its leader overlap and finish ordering are not covered
by plain coop2 alone. The i9 follows only through a separately frozen C5 recipe; none is
authorised here.

Price the critical worker/lead overlap: approximately max(lead serial work, helper inversion)
plus finalisation, further constrained by scanner and ordered sink. A reduction in aggregate
walk Ir need not move that maximum. A faster cooperative default raises the performance
a small table must beat; do not preserve an obsolete competitor. Under the existing
illustrative A=3, f=.7 model, a default speed D' implies zero-gain
g_max=(3/D'−.3)/.7. This is a prospective comparison, not permission to change the running
D4 filters or a validated speed budget.

## 5. Expected, learned, next

Expected: find whether a missed single-worker optimisation can be carried into the team.
Found: a concrete private-tile construction; atomic ownership does not force direct stores
or a chunk barrier. The hard edges are local flush/exhaustion bookkeeping and schedule-driven
tail distribution. #171 supports investigating contention relief; the archived bandwidth
snippets cannot support the asserted exact Ir subtraction.

Measured production gap closed: **zero**. Best next action: Coordinator's bounded staged-team
counting prototype with the above proof and matched attribution, ahead of a dense-pair
implementation. Do not close the advantage if one staging layout or scalar target fails.

No decoder implementation, compilation, candidate execution, new timing or safety certification
was performed. Only source reads and replay of previously committed counts. The replay helper
and source identities accompany this report.

## Channel update before publication

Coordinator LOG 6758381275 reports #324 closed after its registered no-harm failed; no fourth
run, and a possible boxed-Runs successor awaits the Guardian's ruling. This review preserves
that closure and claims no production adoption for the emitter. “Emitter work” above means
the surviving research line, within that restriction. Queued tunnel pricing 6758313507 is
a separate future inbox item, not work undertaken in this report.
