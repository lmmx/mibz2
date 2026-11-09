# Remaining pipeline costs: checkpoint boundaries before another walker

Inbox [6715370785](https://github.com/lmmx/mibz2/pull/160#issuecomment-6715370785), resumed by [6729116363](https://github.com/lmmx/mibz2/pull/160#issuecomment-6729116363). Governing specs and current master: **a4f97714cc14366984015869d9d094d89488ac57**. This is a source audit, archived arithmetic and proposals. No implementation, compilation, new timing or production qualification. The existing Optimiser proposals make this a design assessment, not an independent safety sign-off.

## Decision

1. **Finish the already registered #258 service-2x3 comparison first.** The evidence since the assignment strengthens that question; do not create a competing registration or change its frozen arms.
2. **Best new bounded construction: selected-boundary checkpoint validation (C25).** At 3×3p the helpers consume two build ranges. The leader currently validates 55 checkpoints at a900,000-row block before dispatch, although only one interior checkpoint is consumed. Validate the selected boundaries with the same window/end obligations in a distinct internal interface. This removes exposed setup work without changing the MTF kernel, table, range partition, walk or pipeline policy.
3. **Next: chunked RLE1+CRC preserving the current assembled buffers (C27).** Avoid a full output reread while retaining CLMUL throughput. First falsify fragmentation/setup cost; bytewise CRC fusion is not the candidate.
4. **Leader job cap (C26) needs a trace discriminator before a production-shaped build.** Count is the wrong cost proxy on this corpus:198 small blocks together contain only 8,895 BWT rows, from the committed census. Recovering the full summed join delay is not a credible forecast, and cap1 largely reverses a lever that won.
5. **Combinable per-segment CRC and direct segment-to-RLE fusion (C28) stay behind those.** They move work onto helpers or retain segment scratch across the next inversion; neither cost is free. Checkpoint emission/stride changes follow C25 only if the remaining cost warrants them.

No positive numeric lower bound on a new candidate's speed is established. Below, conditional prices are explicit; zero and regression remain possible.

## What moved since the assignment

The five relevant source files (parallel, coop, mtf, rle, crc) have **identical blobs** at original assignment e01db6a2895711eb13897fdc378816075d927c1e and current master. The source baseline has not moved under this analysis. The evidence has:

- Service-core #251's first attempt was INVALID on N1 and judges nothing. The registered decision run, reported at **a97e5b2a4bdbc83245975a6ef50fb4a72b039fd7**, was VALID. One service thread regressed; two at 12 threads gained 1.058[1.035,1.081] over the default at CPU ratio reference/arm 0.901. Coordinator6721254215 declined that production trade.
- Its exploratory svc2@10 cell (2×3+2 service,8 decode threads versus9) gave speed 1.045[1.023,1.068], CPU reference/arm 1.138[1.118,1.158], RSS64 versus74MB. Wall reduction corresponding to1.045 is 4.31%, not4.5%; CPU reduction1−1/1.138=12.13%. These are reported, not qualifying results. #258 now isolates plain2×3, pipelined2×3 and svc2@10 at the same requested budget; window8 versus default12 remains part of the product comparison. Registration read at **3e4477d035e96310eb4ac3d4014e01e13ffaa33c**, frozen 21cf20a49e3a90817e718a14abcc254679de6b2a. Its review belongs to the Researcher; no substitute admission is issued here.
- The level1–2 plan regression and the boundary/bytes diagnostics are reported on #145 (6720088760,6719903152,6719949774). They make plan/workload sensitivity material. No 3×3@10 result below is generalized to small bzip2 levels or a different topology.
- #266 added test-only run-fed/Ψ² research. Its cost estimates are uncompiled models, not timed falsifications. Coordinator6728891399 has since started a controlled run-fed diagnostic after relay6728860546. That is separate work: keep its whole MTF+sort+build accounting and SIMD comparability; do not duplicate it or treat its existence as a gain. The table remains4n. Selected-boundary validation is a much smaller alternative for the checkpoint part of its proposed saving.

## Evidence quality and replay

The levers journal at master reports the small-only arm with 236.5 overlapping blocks and 26.0ms join delay. **The default's combined park+small arm instead has 229.5 overlaps,22.4ms summed join delay,100.5 late joins of 233 runs.** The brief quotes the former. Same-team overlap is a count of intervals, not overlap duration, useful work or critical-path shortening.

The text readers cannot retrieve the binary pipeline-levers bundle (blob20a74d7c9ef680cb097d45c0fe9566b455190c75,105856bytes): fetch_blob raised UnicodeDecodeError; the generic API fetch returned400, “GitHub Fetch only accepts UTF-8 text.” Thus no raw-trace replay, stage-duration reconstruction or independent validation of that run is claimed. The journal's registered outcomes are retained as reported evidence.

The release-scaling JSONL **was** independently replayed: 240 rows,8 thread counts×3 arms×10 rounds, one 209700196-byte output/hash, no error rows. Paired log wall ratios, exponentiated mean±t(9,.975)·SE, give at 10 threads **1.03730[0.98401,1.09349]** for 0.2.6 over0.2.5, with arm/reference CPU **1.09415**;0.2.6 median wall139.55ms. At 12/16/20 the intervals exclude1. This release check is distinct from the levers' registered benefit: it neither reproduces a precise7% at 10 nor rewrites the prior qualification. See release-replay.json and replay.py.

## The actual dependency chain

At 3×3p, each team has two build helpers and one leader:

- Leader: MTF/checkpoints of a future block and RLE1+CRC/publish of the previous assembled block; it may also decode queued tiny blocks in full. It then joins an unfinished walk.
- Before helper dispatch, on the leader: FusedBuild::new, all-checkpoint validation, prefix starts, reserve, and relay allocation/setup. This part is not overlapped with this run's helpers.
- Helpers: two disjoint build shares; last builder validates completion and sets up the walk. Walkers dynamically claim segments. **Last walker**, possibly the leader, unflags and assembles the current output. Assembly is not invariably leader work.
- Pool completion requires the leader closure and helpers to return. Then next inversion can begin. Published output remains ordered at the reader.

An appropriate per-run model is C=U+max(L,P(L))+D, where U is pre-dispatch setup, L the leader closure, P includes helper build/walk/assembly and depends on when the leader joins, and D is exposed completion/dispatch. For a leader reduction d with P fixed, saving is max(L,P)−max(L−d,P): zero when entirely hidden, at mostd otherwise. With dynamic joining P is not fixed. Sum of phase durations across teams is not stream makespan. Ordered output, scanner supply and the slowest team must remain in the evidence. α fitted to total speed cannot identify any of these terms.

## C25: validate the boundaries actually consumed

Source: coop::edge, check_inputs, FusedBuild::{new,share,finish}; mtf::CHECKPOINT=16384. At n=900000:

| stride | stored vectors incl zero | bytes | 2-share interior edge | longest range |
|---|---:|---:|---:|---:|
|16384 current|55|56320|442368|457632|
|32768 scenario|28|28672|458752|458752|
|65536 scenario|14|14336|458752|458752|

Each vector has 256u32s. Existing valid-input validation performs three whole-vector scans (monotonicity, totals bound, sum) for each of 55 vectors. Actual shares read only C(0),C(442368),C(n); totals supply C(n). Retaining two selected checkpoint vectors plus totals removes **53×256=13568 checkpoint entries, or40704 source-level predicate/sum element visits per full block** from validation. This is neither an instruction count nor a DRAM-traffic claim.

**Construction, minimal first arm.** Keep the 55 emitted checkpoints and exact current edges. Introduce a separately validated selected-boundary object for the pipelined build; retain the generic all-checkpoint API and its rejection contract. For sorted edges e_j, check zero/end, sum C_j=e_j, componentwise0≤C_j≤C_(j+1)≤total, valid counts and range bounds. Each byte bucket's windows [start_b+C_j[b],start_b+C_(j+1)[b]) partition its full bucket. Preserve unique-share claiming, every write's upper-bound check, every share's exact end check and all-share completion before table length. This is the proof obligation, not a safety approval. A corrupted selected count still cannot permit overlap or uninitialized reads. Unused metadata is no longer an input to this distinct object's contract; do not quietly weaken check_inputs in place.

**Price.** Let v be observed critical-worker validation time; d approximately(53/55)v minus adapter cost is a source-count hypothesis, not a time forecast. It is exposed before that run's helper dispatch, but stream speed can still be limited elsewhere. If v is 1% of whole critical path and costs scale with visits, about 0.96% wall is the optimistic scenario; if v=.1%, about .096%. Establish v before a costly prototype. CPU should decrease by the removed scans; copying/repacking every vector would defeat the proposal. Memory capacity is unchanged in this first arm.

**Cheapest CI falsifier.** Untimed selected-boundary differential/mutation checks with all split sizes, duplicate shares, invalid counts, malformed metadata and periodic/multicycle columns. Then one registered diagnostic on existing CI corpus/scale, forcing the3-thread pipeline so the code executes, counting selected versus old validation work and attributing BuildSetup cost. No workstation gain may be inferred from that machine. If setup is negligible or selected checks/adapter do not reduce it, stop before an end-to-end trial. B5 proof/Miri/differential/fuzz/mutation and independent review apply if unsafe paths change.

**Emission follow-up, not bundled.** Larger stride cuts checkpoints but moves helper edges and can lengthen the slowest build share. Worse, MIN_COOP=16×CHECKPOINT: merely doubling the constant raises cooperation threshold262144→524288 and changes the policy. Hold MIN_COOP explicitly fixed in a stride experiment. For3×3p, a sparse emitter can in principle emit only the actual interior boundary, but actual n is unknown until MTF ends; using max_len/2 silently unbalances short blocks. A symbol-length prepass or adaptive boundaries has a cost and needs its own design. Do not sell elimination of the entire0.41ns/B MTF proxy as a checkpoint saving. The AVX2 loop deliberately keeps its hot body unchanged; adding per-symbol boundary bookkeeping previously harmed it.

## C26: bound leader admission, not elapsed time

**New census reconciliation.** #266 reports 431 Wikidata blocks with 209706557 BWT rows;233 blocks≥65536 contain 209697662 rows. The pipeline reports 233 cooperative blocks and 198 below262144. With the same corpus digest and these consistent counts, the198 remaining blocks carry **8895 rows,44.92 average,0.00424% of BWT rows**. This is derived from journal counts, not a fresh per-block census. Even a conservative RLE expansion bound259/5 gives less than 0.461MB output for their 8895 encoded bytes. Their46% block-count share is emphatically not 46% payload work. Fixed job/publish costs may still dominate them.

The loop already takes at most **one next cooperative block**; extra admissions occur only after small blocks. A cap changes how far the leader searches through those small blocks to prime the next cooperative block. A long job count may be cheap; one full MTF may be expensive. Count cap1 approximates removal of the small lever and risks losing its registered3–4% benefit. Prioritize an untimed census of run-by-run admitted jobs, sizes, next cooperative frontier and missed joins, then an archived trace replay if accessible. A useful candidate keeps the previous block's finish, never blocks for a job, and stops only at completed-small-block boundaries; it must leave the queue/front/output ownership consistent and guarantee progress with finite slots and cancellation.

**Price.** In a toy live walk with two helpers at rate v, leader atv joining after d, residual work W in v·ms gives completion(W+d)/3 while d<W/2. Advancing the leader byΔ saves onlyΔ/3 until clipping; the two helpers were already working. If the leader outlasts the helpers, pool delay can fall more, but the small jobs removed from the closure still require later service. Assembly and heterogeneous rates break this toy law.

Using default summed22.4ms merely as a scale: balanced three teams give7.47ms/team; if all of it were recoverable live-walk delay with equal rates and no lost future work, saving is **22.4/9=2.49ms,1.75% of142ms**. The7.47ms/142ms=5.26% scale assumes recovering the whole closure delay per balanced team; it is not a bound or forecast. These scenarios overlap C25/C27's earlier-join gains and cannot be added. Zero or harm is plausible, especially with fewer primed team blocks.

**CPU/risk/falsifier.** Work relocates rather than disappears; charge extra joins, wakeups, queue locks and loss of overlap. Earliest falsifier is a schedule/cost replay showing no expensive extra small-job chains or a lost next-block prime. A forced-plan CI arm with fixed cap2 against unbounded may be proposed only after that discriminator; freeze the choice before timing, preserve park/spin/MIN_COOP/window. Require preserved byte/errors, ordered output, cancellation and deadlock progress. Shorter join delay without shorter whole decode rejects the speed mechanism.

## C27: chunked RLE1+CRC with existing ownership

finish_block currently unrles to final output then rereads it for CRC. RLE1 has AVX2 literal detection/copy; CRC uses four CLMUL lanes on inputs ≥ 128 bytes, scalar slicing otherwise. A bytewise CRC inside RLE would serialize and discard this throughput.

**Construction.** Keep the assembled buffer and final output contract. Carry one CRC folding state through sufficiently large output chunks, updating while bytes are hot; literal spans should eventually feed existing SIMD loads to the fold and store. Carry short tails until one final reduction, not a 64-byte fold reduction per tiny segment/run. Repeated-byte expansion must still produce the exact output, with bounded arithmetic. CRC verification, RunUnfinished and truncation-on-error must precede publication exactly as now. Streaming the CRC by rereading just-emitted chunks improves locality but does not itself eliminate the logical output load; only feeding emitted registers removes that reread. Distinguish those two arms.

**Price.** The logical full output reread is209.7MB for this corpus, not209.7MB DRAM traffic. The other-CI one-thread proxy is CRC.07ns/outputB:14.68CPU-ms, or4.89ms at ideal three-team distribution (**3.45% of142ms**). That is an intentionally generous transferred *entire CRC-cost* scale; fusion still performs polynomial work, so removable cost is only a fraction and the exposed fraction is unknown. At 50% removable cost and 50% exposure the scenario is 1.22ms/.86% wall. It may lose through smaller SIMD batches. CPU delta is saved reread/control minus stream/fold/tail bookkeeping. No positive numerical forecast is justified before listing and matched stage costs.

**Cheapest CI falsifier.** Boundary-focused differential checks at 0/1/3/4/5/31/32/35/63/64/127/128bytes, long repeated-byte expansion, arbitrary chunk splits, all kernel dispatches and malformed terminal runs. Diagnostic comparison on the same existing inputs: unfused versus chunked whole RLE+CRC, including final reduction, reserve/growth and output stores. If isolated work rises, reject before team timing. If isolated work falls but is hidden by helpers, no whole-decoder claim. Historical C17's rejected four-thread fusion verdict remains; this is a new pipeline-specific formulation, not its retrospective pass.

## C28: why per-segment CRC and assembly fusion are later

A segment holds inverse-BWT/RLE1-encoded bytes, not final bytes. CRC on those bytes is the wrong checksum. A run marker can cross a segment boundary: four equal bytes then the extra-count byte. Independent segment decoding requires an incoming RLE state or a prefix composition of finite-state summaries, exact decoded lengths and ordered CRC composition. A single cyclic BWT component may repeat to n rows; assemble follows links and truncates the final segment, rather than concatenating every buffer once. Off-cycle and truncated suffix bytes must not enter the checksum.

The CRC algebra is easy; the ownership and RLE boundary problem is not. Let U(s,x) be the unfinalized non-reflected0x04C11DB7 register, G_m(s)=U(s,zero^m), Z(x)=U(0,x). Then U(s,xy)=G_|y|(U(s,x)) xor Z(y). Start at0xffffffff and complement once at the end. Composing per-chunk summaries (length,Z) is associative in **output order**. CRC32C hardware instructions and bzip2's stream-level rotate/XOR of block CRCs are not substitutes. Length is decoded length, not BWT rows.

At 3×3p there are 1024 segment starts (512 per helper), around 879 BWT rows/segment at 900k, with skew. Extra per-segment summaries, reductions and queue/barrier work move to helpers already on the inversion path. If parallel CRC replaces serial cost c with helper increment p, benefit requires exposed c to exceed critical-helper p plus boundary/combine cost. Total CPU can increase. The generous entire-CRC pool above is not a predicted gain.

**Direct segment→RLE fusion has an additional lifetime charge.** Today assembly copies into one of two byte buffers; next run reuses the one segment scratch while the leader finishes the preceding byte buffer. Removing the assembled buffer but asking that leader to read old segments requires either a second segment scratch (capacity high-water, not simply n bytes) or delaying the next inversion. Doing RLE+CRC on the last walker instead preserves scratch ownership but extends the helper critical path and removes leader overlap. Price both against saved assembly. The transferred.04ns/B assembly proxy is only8.39CPU-ms or2.80ms/three teams before those costs. No free2n copy saving without a schedule change.

**Cheapest falsifier:** an untimed boundary/transducer and ordered traversal proof on periodic/multicycle cases, followed by a measured-cost model including all summaries and a second-scratch/lost-overlap ledger. Unless the remaining exposed finish cost dominates that ledger, stay with C27. No parallel CRC prototype is recommended yet.

## Registration sequence and opportunity cost

| Priority | next action | expected saving at 3×3@10 | CPU | principal risk | cheapest falsifier |
|---|---|---|---|---|---|
|Existing1|Finish #258 exactly as frozen|Exploratory1.045 speed,4.31% wall; not confirmed|reported−12.13%|plan/window and service trade; confirmation|existing controls and registered result|
|New1 C25|Selected-boundary validation diagnostic|≈.964×exposed validation pool minus overhead; pool unmeasured|likely lower scans|invalid disjointness/contract weakening|selected-boundary proof, mutations, setup census|
|New2 C27|Preserve SIMD in chunked RLE+CRC|fraction of transferred3.45% entire-CRC scale; example.86%, not forecast|read/control saving vs folding overhead|tiny-chunk throughput, errors|whole post-stage diagnostic and adversarial splits|
|Information C26|Leader admission census before cap|toy live-walk scale1.75%, before lost priming; may be negative|mostly relocated work|cap cuts profitable tiny-block chain|per-run sizes/frontier and trace cost replay|
|Later C25b|Checkpoint stride/emission|remaining checkpoint portion of MTF, not whole MTF|less copying/checks vs imbalance|MIN_COOP confound, slowest share|fixed-threshold edge/work ledger|
|Hold C28|Segment CRC/direct fusion|unidentified; helper/lifetime costs can dominate|likely added summaries/combine|RLE state, cycles, scratch reuse|boundary proof plus ownership/cost ledger|

A diagnostic can falsify source-cost hypotheses on CI without claiming i9 adoption. New instrumentation and accounting must preserve C6's workload/estimand/repeats/multiplicity/acceptance policy; a proposal to change those goes through the reserved process. The current CI policy may not select3×3p, so force that path for **correctness/diagnosis** while retaining the unchanged production bench. Target benefit still needs the exact frozen recipe run by Louis under C5, and adoption still needs the applicable CI gate. Nothing here authorizes a run.

C3's four questions: expected remaining serial overlap opportunity; observed combined arm has later joins but wins, current sources expose redundant setup, service evidence favors a fresh2-team comparison. Gap closed by this report: localized removable work and corrected ownership/count denominators, **zero additional measured speed**. Mechanism learned: all-checkpoint validation is pre-dispatch, assembly is last-walker work, tiny-block count exaggerates payload, and fusion changes scratch lifetime. Next highest-value work: finish #258; then falsify C25 cheaply before undertaking a new walker or a second architectural rewrite.

## Sources and scope

Pinned source and evidence blobs are in evidence.json. Primary files: src/{parallel,coop,mtf,bwt,rle,crc}.rs; the team-pipeline and pipeline-levers registration/result; knee-mechanism; #266 run-fed census; release-scaling meta/runs. Service result at a97e5b2a4bdbc83245975a6ef50fb4a72b039fd7 and #258 registration at 3e4477d035e96310eb4ac3d4014e01e13ffaa33c. Channel snapshots supplied changed-state/dispatch context; they are mutable evidence, not authority. No raw service or levers tar replay, hardware counters, assembly inspection, code execution of the decoder or safety certification. All numerical phase transfers and equal-rate scenarios are labeled, not admitted results. Historical outcomes remain alongside these proposals.
