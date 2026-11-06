# Resource-based ranking after the walk round

Inbox [6651023083](https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083).
Baseline **c94333183e01f338321c3b9735dc6a044fe762b7**, with #180's **16-slot window at four workers assumed**, not treated as already validated/merged. Window registration inspected at **c90b3625c83d50098c7cd2547cd3c7d80005727d**.
Spec pin **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**. 2026-11-05.

## Recommendation

**Next build: ordered segment → RLE1 → CRC fusion**, developing the Coordinator's already-running local prototype rather than starting a duplicate. Keep the staged walk, prefetch policy, representation and assumed window fixed. Price it on four workers sharing two physical cores. Rank compact-table memory parity second, split condition variables third, a byte-budgeted window fourth, and self-scanning workers fifth.

This changes my earlier instruction-led ranking. Instruction accounting remains useful for correctness of cost attribution and regression diagnosis; it is not a prediction of wall time. No numerical four-thread speedup is identified by the available evidence for any unbuilt candidate. Below are conditional resource prices, ceilings and explicit falsifiers, not invented confidence intervals.

## Evidence and changed verdicts

Read [walk-round](https://github.com/lmmx/mibz2/blob/c9433318/docs/journal/2026-11-05-walk-round.md), its 17 archived run summaries for #168–#173, [deferred flags](https://github.com/lmmx/mibz2/blob/c9433318/docs/journal/2026-11-05-deferred-flag-walk.md), [staging](https://github.com/lmmx/mibz2/blob/c9433318/docs/journal/2026-11-05-staged-walk.md), and the [window registration](https://github.com/lmmx/mibz2/blob/c90b3625c83d50098c7cd2547cd3c7d80005727d/docs/journal/2026-11-05-window.md).
Source inspection: c9433318 src/bwt.rs, lib.rs, rle.rs and parallel.rs; the staged spec_offset-P0.asm listing; b4786f66 src/c30.rs. Earlier portfolio and causal evidence remain linked in [the prior report](../inbox-6646251725/report.md); the current published MLP discussion explicitly warns that a lane-count plateau does not uniquely identify a resource.

| Candidate | Evidence disposition retained |
|---|---|
| #168 deferred flags | Benefit not shown, 1t harm signal unsettled at closure; subsequently included in #171 |
| #171 staging + flags | Merged; all five reported 4t wall intervals faster, about −5% to −14%; **runs 1t remains unresolved**, not proven harmless |
| Gather | Local rejection relative to staged scalar, despite −31% Ir versus older master; not a four-thread CI qualification |
| #169 build vectors, #173 tail passes | Stage/Ir improvements did not establish decoder benefit; closed as not shown |
| #170/#172 MTF | Reported production improvements, already baseline; #170 random 4t and #172 later onebyte 2t harm signals became unresolved, not erased |
| #177/#178 large tiles | Brief reports closed/failed; do not transfer indicative local gains. Those later results are not in c9433318's round archive |
| #180 window | Assumed intervention for this ranking. Brief's first-run −8.1% wall comes with +6.1% CPU and +8 MB RSS; not a free kernel gain |
| #181 staging-only | Local comparison supports no detected additional flag benefit, not equivalence; CI result was pending in the brief. No result inferred here |

The listing at c9433318, staged spec_offset-P0.asm 0x34440 onward, shows table loads, tile-byte stores, row-slot stores and prefetch. It supports the changed store destinations. It does **not** count RFO occupancy or prove the precise miss-buffer explanation. Removing 32 concurrent output streams can improve locality and request overlap while leaving nearly the same total eventual output traffic. Bandwidth, queue occupancy, cache conflicts, replacement and code-layout effects remain competing explanations.

Likewise the old occupancy shares must not be summed: 5.5+4.3+13.0+7.9 = 30.7%, exceeding 23.5% idle. They describe overlapping conditions/different attribution boundaries. The 93% busy observation at window 16 is unpaired and is not a causal measurement of what #180 recovered. Both within-block contention and between-block availability can bind at different times.

This is a scoped research assessment, not fresh independent qualification: I proposed the earlier B direction. No candidate execution, compilation, timing, Miri rerun or full raw-profile replay occurred. Run summaries are transcriptions, not reconstructed raw confidence intervals.

## Shared-resource price: units and uncertainty

Let n be inverse-BWT rows (RLE1-coded bytes), m final output bytes, and a=n/m. For text a is near one, but it is not always one; run-heavy inputs require their actual expansion. Convert all per-row traffic below to per-output-byte by multiplying by a.

Four workers on two cores mean **two active private tables per shared L2**, not four independent L2s and not four tables in each L2. Scheduling/phase overlap changes this occupancy; two is the explicit sensitivity assumption.

For random accesses, record separately:
- L1 miss requests served by L2, rate q2 per output byte;
- L2 misses served below L2, q3, with measured effective latency L3plus;
- prefetch requests, merging and usefulness; RFOs/writebacks;
- the dependent address/reconstruction delay and sustainable concurrency.

A useful queue-occupancy price is request-cycles/output-byte:
**D = q2·L2 + q3·L3plus + D_prefetch + D_RFO**.
At core output rate R, occupancy is approximately R·D. Shared capacity M requires R·D ≤ M; bandwidth and dependent-chain throughput impose separate bounds. This is a falsifiable model, not permission to add overlapping stall counters or equate instructions with cycles.

The supplied L2 capacities do not specify L2 hit latency, below-L2 latency, bandwidth or effective M. No pinned same-run latency/counter measurements establish those numbers here. I therefore do not manufacture cycle constants for Zen 3, Zen 4/5 or Emerald Rapids. Fit/measure them on each runner, separately for the two SMT siblings, before converting the model to time. A capacity-only model below is labelled illustrative.

## 1. Fusion: remove passes without changing random traversal

At c9433318, SegScratch::assemble follows first/next segments until exactly n bytes, writes ll, rle::unrle reads that slice and writes final output, then decode_block calls crc32 over the appended output. P87 changed the assembly destination; it did not eliminate these passes.

**Traffic ledger, excluding walk/staging common to both arms:**
- current: read n segment bytes + write n assembled bytes + read n RLE input + write m decoded bytes + read m CRC bytes = **3n+2m logical bytes**;
- fused: read n ordered segment bytes + write m decoded bytes = **n+m**, with CRC computed on the bytes being emitted;
- removed: **2n+m logical bytes**, or **2a+1 per final byte**.

This eliminates no random T access. The removed intermediate write/read and CRC read correspond to potential sequential line transfers, not random misses. Under a cold write-allocate/writeback model their removable transfers are at most approximately (3a+1)/64 lines per output byte (RFO + writeback + read of intermediate, CRC read), ignoring edges. The lower bound on off-core traffic saved is zero if everything hits owned/resident cache; ll may already be owned from MTF. Do not present the upper model as measured DRAM savings. Segment reads and final output stores remain.

**Construction and equivalence.** Feed a streaming RLE1 transducer from segments in exactly assemble's first/next order, capped at n total bytes. Preserve partial runs and pending count across segment/chunk boundaries. A repeated cycle must be traversed until n, not stopped at the first repeated segment. Maintain CRC across emitted decoded chunks in the existing byte order/polynomial convention; CRC arithmetic remains, only its reread is targeted. Use a bounded carry buffer for boundary lookahead so vector loads never cross initialized input. On truncated runs, CRC failure or cancellation, retain existing error semantics and out.truncate(start). No partial block becomes visible before validation.

**Whole-decoder Δ at 4t.** Sign and magnitude remain unmeasured. If the exposed post-walk fraction is f and fusion reduces that phase by fraction g, unchanged-rest wall Δ is −f·g, plus any increase from lost SIMD/bulk CRC efficiency and changed sibling contention. For illustration f=.10–.20 and g=.25–.50 gives −2.5% to −10%; **these are sensitivity inputs, not a forecast or evidence that f/g take those values**. Phase instruction shares cannot supply f.

**Prior:** fusion has not been established by the cited records; the brief says a prototype is local. P87 and #171 are related destination/locality priors, not fusion results. Prior assembly's low Ir does not bound the cost of the passes or cache interference.

**Refute:** on the same pinned window-16 baseline compare CRC-on-output alone, assembly→RLE fusion, and full fusion as attribution arms within the registered evaluation. If total 4t wall fails the prospective benefit rule, or boundary/CRC/error controls fail, reject that formulation. If line traffic falls but wall does not, the removed traffic was not the binding cost at that point. Cover every run/count split, empty/tiny blocks, repeated cycles, high expansion and reuse after error. Retain ordinary AVX2/CLMUL fast paths where possible; scalarizing everything to save a pass can lose.

## 2. Compact-table parity in lines, including construction

The prior **46.25% walk-Ir reduction is still the arithmetic condition for instruction-count parity at its old pins**. It is not a time break-even and must no longer rank this opportunity. A current Ir budget would also need current full counts; do not reuse it against c9433318.

Verified layout in [b4786f66 src/c30.rs](https://github.com/lmmx/mibz2/blob/b4786f66/src/c30.rs): Rec is aligned 64 bytes, 30 u16 fields with a header at byte 60. A normal field and header occupy the **same physical line**, so two loads do not mean two independent line fills. Labels map directly to records; this formulation has no extra global offset-directory lookup. RAW records additionally chase a 128-byte RawG (30 used entries). Saved start/successor arrays, PARK, marks and metadata add costs.

For n=900,000:
- plain main table: 3.6 MB, 56,250 lines, 16 entries/line;
- compact main table: 1.92 MB, 30,000 lines, 30 entries/line, plus PARK/RAW and auxiliary state.
- compulsory distinct main lines/row: **1/16 versus 1/30** if every relevant line is used. These are **not random steady-state miss rates**. If reuse distance exceeds residency, either format can require nearly one main-line fill per step.

Illustrative independent-uniform-reference model: equal siblings, no other cache users, all stated L2 assigned to two tables. Main-table L2 miss probability approximately max(0,1−C/(2S)):

| Assumed shared L2 | Plain | Compact main only | Reduction in below-L2 main-line demand |
|---|---:|---:|---:|
| 512 KiB (brief's Zen 3) | .9272 | .8635 | .0637 lines/row |
| 1 MiB (brief's Zen 4/5) | .8544 | .7269 | .1274 lines/row |
| 2 MiB (brief's Emerald Rapids) | .7087 | .4539 | .2549 lines/row |

Multiply by a for final-byte rates. This is optimistic about table residency, not a measured bound; real permutation reuse, RAW footprint, prefetch and other buffers change it. **Even the compact pair is 3.84 MB and does not fit 2 MiB.** A one-table-fits argument is wrong under two simultaneous sibling walks.

If L1 miss rate is approximately unchanged, the simple service-demand saving is approximately a·(p_plain−p_compact)·(L3plus−L2), **not** that difference times DRAM latency unconditionally. Subtract RAW dependent misses, any lower prefetch timeliness and reduced effective concurrency. Smaller footprint can help while additional reconstruction dependence still loses.

**Builder ledger.** Plain writes 4n table bytes. Compact writes at least 64·ceil(n/30) main bytes plus 128 per RAW group and PARK; both read ll. Under cold write-allocate plus writeback, main-output traffic is approximately 8 bytes/row plain versus 128/30≈4.267 compact. But compact also initializes its bitmap, captures snapshots, finishes portions, canonicalises RAW and may copy/resize arenas. Those read/write/revisit costs and lost overlap are **not zero**. Existing Ctor::build explicitly exposes these operations. Warm reuse changes ownership traffic. Measure builder and walk together; the ideal main-output reduction is not the net builder saving.

**Whole-decoder parity:** require total builder-time change + walk-time change + rest-time change < 0 at four workers. Resource model equivalently requires the saved main-line service demand to exceed new RAW/metadata demand, exposed reconstruction cost and builder/rest penalties. No current positive percentage follows from footprint alone. Existing #155 remains rejected; a new staged compact walker is a distinct proposal that must earn its own result.

**Refute:** a trace-driven two-table cache/reuse analysis including RAW and builder stores, followed by a properly registered paired whole decoder comparison with staging/prefetch/window matched. Reject if the net result fails even though the footprint shrinks. A same-byte layout-only control distinguishes reconstruction overhead from caching; do not reopen the old port unchanged.

## 3–5. Remaining scheduling candidates

At the assumed window, 93% busy gives only a conditional ceiling: with unchanged useful-work cost and perfect redistribution, increasing busy fraction to 98% reduces wall by 1−.93/.98=**5.10%**; to 100% by **7%**. Those are optimistic capacity illustrations from an unpaired observation, not attainable speedup bounds when SMT interference changes. All three proposals compete for that same slack; never add their ceilings.

| Rank | Candidate and resource price | Prior / whole-decoder Δ | Falsifier |
|---|---|---|---|
| 3 | Split condition variables by predicate (ready-next vs free-slot), targeted notification; broadcast all relevant wait sets on cancel/end/error | Source uses one shared Condvar with notify_all. Unbuilt in this record. Zero direct random-table misses removed; may reduce scheduler wakeups/cache-line ownership transfers per job. Unknown wall benefit, likely within the residual-slack opportunity under fixed useful-work cost | Count wrong-predicate wakes and runnable delay at window 16. Reject if waits are not spurious-wake driven or whole wall does not improve; stress lost wakeups, EOF, cancel, ordering |
| 4 | Byte-budgeted admission, retaining a hard block cap and a progress reservation for the earliest unfinished block | #180 is a positive window-size prior, not validation of byte budgeting. Primarily a memory/tail-latency proposal after window 16; speed can be neutral or worse. No direct T miss saving; reducing buffered active bytes may improve cache residency indirectly | Compare at equal peak resident bytes AND equal throughput targets. Reject if head-of-line progress stalls, high-expansion blocks exceed the declared bound, or memory benefit requires unacceptable wall regression |
| 5 | Self-scanning workers, one serialized scanner owner feeding bounded jobs before returning to decode | No demonstrated current gain. Removes/hides handoff but moves scanner CPU onto workers sharing the same two cores; can starve all workers while one owner blocks on input. Old 13% scan-wait share is not remaining recoverable time | Equal window/bytes, matched topology, measure scanner work, claim latency, off-CPU time and total CPU/wall. Reject if scan work merely displaces useful decode or tail/input starvation worsens |

Byte budgeting needs capacity-aware accounting for symbols, in-flight output and expansion, not just compressed length. Exact output size may be unknown before RLE1; use conservative reservations/reconciliation and ensure the earliest required result can finish without waiting for credit held by later results. Otherwise a memory optimization can deadlock ordered publication.

Split condvars is the smallest independent scheduling experiment if fusion fails, but its potential is unmeasured. Neither self-scanning nor a queue rewrite should be justified with stale pre-window percentages.

## Deliverables and scope

[resource-model.json](resource-model.json) records the arithmetic and assumptions. Findings above are CONDITIONAL resource models, not new verified instruction credits or a governance decision. The compact main-line model is reproducible from n, entry geometry, two active tables/core and the supplied capacities. Absolute latency and current four-thread phase exposure remain missing measurements; they prevent a credible numerical prediction, not a ranking.

Next evidence requested: the already-running fused prototype at a pinned head, ordered-boundary/error tests, matched four-thread decoder results against c9433318 plus the assumed window, CPU/RSS tradeoffs, and enough attribution to distinguish removed passes from changed code layout. No fresh timing or implementation was performed here.
