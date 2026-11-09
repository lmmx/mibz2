# Run structure that can change the decode bill

Inbox **6729486092**; queued brief **6729427506**. Researcher, 2026-11-09.
Operating pin: a4f97714cc14366984015869d9d094d89488ac57. This is research and a proposed experimental ordering, not registration, implementation, timing evidence or production qualification.

## Decision

**The best chance of beating production is still a cheaper leader with the present dense walk.** First isolate the MTF list-kernel change; then compare helper-side sorted fill with sort-free direct placement. Neither a leader-with-sort loss nor a one-worker loss closes those constructions.

There are also two concrete mathematical changes beyond that scheduling idea:

1. Construct a dense two-step table by intersecting translated runs. This removes the per-row bigram histogram/scatter of the previous pair builder. Its word layout alone is old; the run-based construction is the consequential difference.
2. Exploit the 20-bit row bound of a bzip2 block. Three otherwise unused bits in a pair word can encode the low three bits of its first symbol. A 3.5 KB directory supplies the high five bits on ordinary rows. This replaces the old first-symbol correction path, with an exact fallback on at most 31 exceptional row blocks.

A three-byte one-step table can use the same symbol decomposition, at about 2.705 MB for 900k rows. That proves a space reduction, not a speedup. I rank it below retaining the current consumer. Balanced move structures, lazy materialization and output-copy schemes remain lower-value than these routes on a 900k block with 32 independent lanes.

**Expected winners, plainly:** matched MTF modernization and a bounded helper-side one-hop construction have the strongest prospect. Direct placement could beat sorted fill if predecessor starts are cheap. The run-built pair is worth a distinct candidate because it attacks the observed old build loss; I do not predict a whole-decoder win until its unchanged-walker control passes. I expect the packed three-byte and general move-walk routes to lose on ordinary CI cases unless a measured shared-cache constraint changes their price.

## Evidence that changes the ranking

The starting sources are [#266's census](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md), [#269's draft](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/docs/journal/2026-11-09-run-fed-stage.md), and the two 8 November posts on [MTF runs](https://github.com/lmmx/mibz2/blob/31e6a0c5e1e68db64555ee9c630dc679c3ca8d18/docs/site/blog/2026-11-08-bwt-runs-from-mtf-codes.md) and [inverse-BWT navigation](https://github.com/lmmx/mibz2/blob/31e6a0c5e1e68db64555ee9c630dc679c3ca8d18/docs/site/blog/2026-11-08-inverse-bwt-navigation.md). The [Optimiser's report](https://github.com/lmmx/mibz2/blob/e071cb7c981589c02f305f1de5c28ebfe2c7d6fc/state/records/reviews/inbox-6729426658/report.md), received during this review, supplies C29–C34. Its C31 sort-free derivation is credited below.

- **C29 matters first.** Run/plain MTF uses a two-register list kernel; checkpointed MTF uses a different memory-list kernel. The indicative 25% comparison does not isolate checkpoint removal. A modern-list checkpointed control can recover benefit without new table construction.
- **Pair inversion is not new here.** [The 27 October pair journal](https://github.com/lmmx/mibz2/blob/31e6a0c5e1e68db64555ee9c630dc679c3ca8d18/docs/journal/2026-10-27-pair-walk.md) already tried a dense symbol-rows pair layout. Recorded Wikidata regressions were 40.6% at one thread and 24.3% at four. Its local phase indications put build near 7.1–7.6 versus 2.5 cycles/byte, while walking changed little. Those local figures are indicative, not qualified speed evidence. A new builder must remove that cost, and a new consumer must prove its own gain.
- **Copying is not new either.** [The Precopy journal](https://github.com/lmmx/mibz2/blob/31e6a0c5e1e68db64555ee9c630dc679c3ca8d18/docs/journal/2026-10-27-precopy-forward-chains.md) records 31.5% copied Wikidata output but regressions of 18%/6% at one/four threads. Pattern/runs copied nothing despite admission. Raw run count is not a usable-copy predictor.
- #266 reports line-A cells/rows about .0881 and 91.7% short cells, while production already fills many rows with run stores. These are inherited census results, not rerun here. A rough 2.6 versus 2.5 instruction model is neither emitted assembly nor a universal impossibility result.
- #269's unsafe safe-interface objection and admission qualifications remain scoped to that candidate pending its separate corrected review. D042 is not resolved by this research. #258 remediation and C25 remain competing uses of engineering effort.

## Primary-source map: what transfers, and what does not

Page references below are printed paper pages unless explicitly called PDF pages. Source findings are separate from my proposals.

| Source | Relevant result and location | Transfer to this decoder |
|---|---|---|
| Nishimoto & Tabei, [Optimal-Time Queries on BWT-Runs Compressed Indexes](https://drops.dagstuhl.de/storage/00lipics/lipics-vol198-icalp2021/LIPIcs.ICALP.2021.101/LIPIcs.ICALP.2021.101.pdf), 2021 | pp. 101:7–101:9, especially Lemma 6: balance a translated-interval bijection with at most twice the intervals; carry row and interval state to support constant-time moves. Construction/extraction: pp. 101:12–13. | Establishes bounded navigation, not a small native instruction count. At 32 lanes, an extra dependent interval lookup can cost more than saved footprint. |
| Brown & Langmead, [Bounding the Average Move Structure Query…](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.pdf), SEA 2026 | pp. 9:4–9:8; Lemma 6, p. 9:7, gives linear-in-r LF/FL construction; Theorem 9, p. 9:8, gives O(n) inversion in O(r) space using the average bound. | Useful construction bound. Single-cycle amortization cannot silently price repeated visits in a periodic cyclic bzip2 transform. Genomic-scale experiments do not predict 900k-block speed. |
| Brown et al., [Optimal-Time Move Structure Construction, arXiv:2603.22147v2](https://arxiv.org/pdf/2603.22147v2), 24 April 2026 | Printed pp. 5–9 (PDF pp. 6–10); Theorem 26 and Corollary 27, printed p. 9. Given interval starts, images and image order, balance arbitrary permutation moves in O(r) time/space. | Removes superlinear balancing from the candidate bill. It supplies neither a parallel span bound nor a replacement for the measured 41-instruction consumer. This version numbers the theorem differently from earlier accounts. |
| Nishimoto, Kanda & Tabei, [An Optimal-Time RLBWT Construction in BWT-Runs Bounded Space](https://drops.dagstuhl.de/storage/00lipics/lipics-vol229-icalp2022/LIPIcs.ICALP.2022.99/LIPIcs.ICALP.2022.99.pdf), 2022 | pp. 99:1–2: r-comp constructs the RLBWT **from text**, in O(n+r log r) time and run-bounded space. | Different input problem. We already receive MTF/RUNA/RUNB information; importing text-to-RLBWT machinery would redo work. |
| Bertram, Fischer & Nalbach, [Move-r: Optimizing the r-index](https://drops.dagstuhl.de/storage/00lipics/lipics-vol301-sea2024/LIPIcs.SEA.2024.1/LIPIcs.SEA.2024.1.pdf), 2024 | §4.1, p. 1:5 onward: practical move-structure construction and balancing. | Relevant implementation comparison, not evidence that the reviewed construction has a work-efficient parallel span. I found no applicable parallel guarantee in these reviewed move papers. |
| Kärkkäinen & Puglisi, [Cache Friendly Burrows-Wheeler Inversion](https://www.cs.helsinki.fi/juha.karkkainen/publications/ccp2011.pdf), 2011 | PDF pp. 3–4, §IV: exploit repeated path/output information; alternatives recover symbols from F buckets. | Copying completed output is stronger than observing a long L run. Metadata and bucket lookup must be charged. Large single-chain cache results do not establish a 32-lane improvement. |
| Kärkkäinen, Kempa & Puglisi, [Slashing the Time for BWT Inversion](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf), 2012 | PDF pp. 4–5, §3: two-symbol steps and eight interleaved chains; pp. 5–7, §4: s-runs, reusable output, Lemma 8 and Theorem 9's cache-miss bound O(n/s+r_s+n/B). | Pair steps and equal-offset repeated trajectories are directly relevant. Supplied chain starts in the paper are not free in bzip2; discover/mark/assemble them. Its larger table and copy bookkeeping are not free either. |
| Ferragina, Gagie & Manzini, [Lightweight Data Indexing and Compression in External Memory](https://arxiv.org/pdf/0909.4341), v1 | PDF pp. 10–12, §5, Theorem 6: inversion by substring records, sorting and list ranking with bounded internal space and multiple disk passes. | Transfers destination grouping and sequential-I/O principles, not a competitive implementation for a 3.6 MB table. The [bwt-disk site](https://people.unipmn.it/manzini/bwtdisk/) principally distributes BWT **construction**; do not call it a drop-in fast inverse. |
| Intel, [Optimization Reference Manual vol. 1, rev. 050](https://cdrdv2-public.intel.com/821612/248966-Optimization-Reference-Manual-V1-050.pdf) | §9.6.1, p. 9-25: non-temporal stores suit appropriate streaming/write-once uses. The indexed primary excerpt was available; the full PDF retrieval failed. | A table read immediately by the walk is a producer-consumer case. Avoided write allocation competes with lost cache residency and fence/reader cost. No architecture-independent “4n floor.” |

### libsais: source, not reputation

Inspected [libsais.c at c284ba843dd16c159acb976f17bc19382793ea75](https://github.com/IlyaGrebnov/libsais/blob/c284ba843dd16c159acb976f17bc19382793ea75/src/libsais.c); code has line/function references rather than pages.

- Lines 11263–11483 build biPSI/symbol information; **11543–11686** split input into contiguous thread ranges, form per-thread symbol/bigram counts, prefix them, and scatter into disjoint ranges. This is verified parallel construction practice, not a theorem imported from the O(r) papers.
- **12466–12566**, symbol-rows decoding: 24 interleaved lanes; one pair-table load, a marker test, next-row prefetch, first-symbol directory plus ends correction, then two output bytes.
- **12772–12950** covers chain setup, OpenMP distribution and stitching, including cyclic/periodic handling. **12975–13011** gates parallel initialization by size/thread count.
- The fully packed two-byte/16-bit-pointer path is limited to n≤65535. It does not fit 900k. For the relevant symbol-rows path, **13273–13293**, a shift of five makes the used first-symbol directory about 28,126 bytes at 900k, plus 256 ends and a 4n pair table.

The repository's [earlier libsais review](https://github.com/lmmx/mibz2/blob/31e6a0c5e1e68db64555ee9c630dc679c3ca8d18/docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md) already recognizes this family. I claim no novelty for “use libsais's pair layout.”

## Ranked proposals and falsifiable predictions

Every row applies to at most 900k rows and must retain or explicitly replace the 32-lane room walk. None authorizes a run. Coordinator registration must freeze the controls and information/performance objective first; production adoption retains the unchanged CI grid and paired whole-decoder gate.

| Rank | Proposed mechanism / source basis | Expected application | Falsifier on the existing CI path |
|---|---|---|---|
| 1 | Matched MTF-list control, then helper one-hop fill; C29/C30, libsais parallel-prefix pattern | Preserve current table and 32-lane consumer. Greatest prospect where the leader binds; only two helpers in a three-thread team. | Matched checkpoint control explains the emitter gap, or total setup/helper/barrier cost erases leader release. Separate attribution from whole-decoder benefit. |
| 2 | C31 direct placement without sorted F-record materialization; Optimiser derivation, independently checked below | Same consumer; remove 8(r+1) live sorted-record bytes, scatter/read and count histogram. | Predecessor-start and bucket-stream cost exceeds removed sort/materialization; compare same emitter and bounded fill. |
| 3 | Run-intersection dense pair builder; §3 of Slashing plus new derivation below | Same 4n table; two bytes per step. First keep the old pair walker unchanged to isolate build. | It fails to reduce old pair-build cost enough to beat full one-hop construction+walk. A faster pair build alone is insufficient. |
| 4 | Spare-bit first-symbol path for the pair table; original decomposition below | 20-bit pointers, three symbol tag bits, about 3.5 KB high-symbol directory; potentially shorter pair loop. | Native common path/critical dependency fails to improve; exception frequency or marker/assembly work erases it across the existing corpus. |
| 5 | Exact bounded vector/tile fill before non-temporal stores; C32/C33, Intel | Preserve table/consumer. Reduce repeated stores on short cells and enforce independent ownership. | Exact W census/listing shows little removable store work, or masks/packing/tails cost more. Measure construction **and following walk**, including fence. |
| 6 | Three-byte direct-index one-hop table; original decomposition below | About 25% smaller navigation data; no move-descriptor chain. Likely only compelling under cache pressure. | Packed addressing/load/extraction and build cost exceed measured cache benefit at the existing thread grid. |
| 7 | Dense/affine block hybrid; move literature plus original fixed-block specialization | Direct block index; exceptional blocks stay dense. | Actual uniform-block/visit coverage is too low, or dependent directory-to-dense lookup outweighs omitted writes. Census before walker construction. |
| 8 | Run-interior jumps, repeated-trajectory batching/Precopy; 2011/2012 papers | Specialize profitable trajectories, not all rows. Retain cheap dense fallback. | Consecutive valid arithmetic steps/copy reuse are rare, or checking and regrouping 32 lanes exceed saved steps. Prior copied-byte fraction alone is not sufficient. |
| 9 | General balanced move inversion; O(r) construction papers | Smaller table, but interval state and boundary navigation remain. | Reconstructed emitted loop still near the failed consumer cost; O(r) preprocessing alone cannot pay it back. |
| 10 | External-memory inversion/list ranking | Useful only if the problem becomes genuinely external-memory or heavily batched. | Multiple record/sort passes cannot repay their cost for 3.6 MB per block. No present reason to prototype this route. |

## 1. Parallel construction: weighted prefixes, then destination ownership

Let run i be (c_i,p_i,ell_i) in L order. Stable symbol order gives its F start

a_i = C(c_i) + sum of lengths of earlier runs of symbol c_i.

On I_i=[a_i,a_i+ell_i), Ψ(q)=p_i+q-a_i. Thus maximal runs give translated intervals without a rowwise LF construction. The blog's m≤r(1+log₂(n/r))+1 bounds encoded symbols, not the number of instructions in their decoding.

Partition original records into h ordered contiguous shards. For symbol c, each shard t records row sum W_t[c] and record count H_t[c]. Its a-prefix is C(c)+Σ_{u<t}W_u[c]. A stable compact scatter slot instead needs the prefix of H across symbols and earlier shards. **One histogram cannot supply both.** Work is O(r+hσ), σ=256; span includes shard work, histogram reduction/prefix and barriers. With a serial small prefix it is O(max shard work+hσ), not automatically O(r/h). A long run destroys row balance despite equal record counts.

For C31, omit the record-count histogram and sorted-record array entirely. Process each original run with its weighted a. Its F interval has exclusive destination ownership. Find the L run containing a, then merge its boundaries while writing

T[q] = L[q] | ((p_i+q-a_i)<<8).

The low byte is **L[q], not c_i**. The intersections across all runs are at most 2r−1 regardless of processing order. What replaces sorting is r predecessor starts and up to 256 bucket streams. A row directory can bound the search range; it needs construction, space and actual comparison/cache-line counts. Equal record shards still need row-weight balancing or virtual splits of very long runs. Unconditional overstores cannot cross another helper's range.

I independently checked the weighted-prefix identities; C31's proposal belongs to the Optimiser. It is a stronger rival than assuming sort must merely be parallelized. At roughly tens of thousands of runs in a 900k block, also retain **serial sort on a helper** as a control: a theoretically parallel sort can lose to its barriers.

For economics, use C30's arrival-dependent form:

T = U + max(L, P(L)) + D.

P includes helper build, walk and the leader's eventual joining. If E=.75M were established by matched controls, gross leader release is .25M minus new leader work, not 25% of decode wall. Helpers now pay their new construction; a leader arriving earlier cannot be counted twice. A four-core oversubscribed 3×3 run cannot establish the i9 ten-core claim.

## 2. Dense two-step construction in O(r+n), without per-row bigram counts

This is the strongest new construction route I found. It applies the translated intervals directly to the already-known dense pair idea.

Let J_i=[p_i,p_i+ell_i), the image of I_i. Intersect J_i with every F-domain interval I_j it meets. For an intersection [x,y):

q = a_i+x-p_i,
Ψ²(q+t) = p_j+x-a_j+t,  0≤t<y-x.

Also F(q+t)=c_i and F(Ψ(q+t))=c_j throughout this cell. Therefore emit

D[q+t] = ((p_j+x-a_j+t)<<8) | c_j.

No rowwise bigram histogram, random second-row symbol fetch, or per-row bucket increment is required. The word sequence advances by 256.

**Construction accounting.** J intervals are already in L order; I intervals require the stable symbol/weighted-prefix representation. Merge the two interval partitions in x order, O(r) boundary work. Emit directly into the corresponding q ranges, which are disjoint but not necessarily monotone in emission order. This avoids an additional cell array; it trades output locality against a reordered-cell version. If monotone q writes are desired, charge grouping/materialization rather than calling them free. Total word emission remains O(n); helper splits of x need start-location metadata and bounded destination ownership.

If S is the set of I starts, the refinement is S ∪ Ψ⁻¹(S), of size at most 2r−1. It equals the line-A count |S ∪ Ψ(S)| by applying the bijection Ψ⁻¹. These are uncoalesced construction cells; adjacent equal translations can coalesce, so do not equate them with the minimal number of Ψ² breaks.

A walker starting at q=orig_ptr emits F(q), then D[q]&255, and advances to D[q]>>8. That explicitly chooses F coordinates. Porting the current L-coordinate room protocol requires rephasing its segment starts, marker tests for **both** traversed rows, odd tails, periodic orbits and assembly. It is not a drop-in table substitution.

**Prediction:** eliminate the old pair builder's dominant per-row histogram/scatter work. First compare the old pair walker on old versus run-built identical pair words; only then test a changed first-symbol path. At 32 lanes, pair stepping may have little remaining latency prize, so build improvement must survive against the new run-fed one-hop baseline too.

## 3. Spare bits buy a smaller first-symbol path

This is an original proposal here, not a cited literature performance result.

For bzip2 n≤900000<2²⁰. In D[q], bits 8–27 hold the next row; bits 28–30 are unused if bit 31 remains the marker. Store

D3[q] = (Ψ²(q)<<8) | F(Ψ(q)) | ((F(q)&7)<<28).

Partition F rows into fixed blocks of B=256. Directory H[b] holds F(256b)&248 and a flag saying whether the block's first and last symbols have different high five bits. F is sorted, so an unflagged block has constant high five bits. Then its first symbol is

(H[q>>8]&248) | ((D3[q]>>28)&7).

Its successor is (D3[q]>>8)&0xfffff. Test bit 31 separately. A flagged block uses an exact predecessor search in the 256 cumulative symbol ends. Directory construction uses sorted boundaries; no per-row symbol search is needed. Within each pair cell, the two symbols and tag are constant, so vector-fill arithmetic remains affine.

There are only 32 high-symbol groups and at most 31 transitions. At most 31 blocks are flagged. A conservative whole-flagged-block fallback covers at most **31×256=7,936 distinct rows**, 0.882% of 900k. H uses **3,516 bytes**, versus about 28,126 used bytes for the old shift-five first-symbol directory; retain roughly 1 KB of cumulative ends.

**Do not turn that row bound into a dynamic rate.** Ψ² may visit only one parity class; periodic cyclic transforms can repeatedly visit a small exceptional set. Count actual fallback executions. Small blocks can have a much larger exceptional fraction. The progress note's 31×255 bound covered only post-boundary rows; whole-block fallback needs 31×256, as used here.

This replaces the common ends load/correction with a tag extraction, OR and flag branch, plus a pointer mask. Whether native instructions and critical latency improve is unresolved. The directory access is indexed by the already-known row, so can be independent of the pair-word load; a compiler need not exploit that. Archive the emitted loop. Marker restoration must preserve the tag. The new table cannot inherit old room safety merely because it still occupies four bytes.

## 4. A 3-byte one-hop table without move descriptors

Use the same directory but encode

E[q] = Ψ(q) | ((F(q)&7)<<20),

with bit 23 reserved for a marker. Three bytes hold the word; get the next row by masking 20 bits and the symbol by the high-directory/low-tag rule. Starting at orig_ptr and emitting F(q) gives a direct-index one-hop formulation. There is no interval-ID search or next-pointer reconstruction through a move record.

Navigation payload at 900k is

3n + ceil(n/256) + 256×4 = **2,704,540 bytes**,

against 3,600,000 for the current four-byte words: 895,460 bytes, about 24.87%, less, before alignment, tails, marker/segment auxiliaries and allocator capacity. This comparison does not claim a 25% reduction of total decoder RSS.

The price: 3q addressing, packed loads/extraction, flag handling and less natural vector stores. A four-byte read of a three-byte entry can cross an ownership/cache-line/page boundary; padding and bounds must be proved, including concurrent marker changes. A literal three-byte load may cost more instructions. The 32-lane consumer may already hide enough latency that all of this loses. This is a direct fixed-stride alternative to compact30's dependent descriptors, not an assertion that packing is now free.

## 5. Store policy and hybrids: count the work that actually disappears

For cell lengths ell_i, the current eight-row overstore shape writes

W = Σ_i 8 ceil(ell_i/8),  n≤W≤n+7c.

At illustrative c/n=.0881, W/n≤1.6167; that is an upper bound, **not the measured W**. Short-cell percentage alone does not determine W. Use a census before choosing masks, scalar tails or assembled boundary vectors. Fewer stores can still cost more instructions. Cache-line-aligned destination tiles let helpers own exact full lines, but tile setup and boundary reconstruction need pricing.

Non-temporal stores should be a conditional long-full-line arm, with ordinary bounded tails, a justified producer-consumer fence and subsequent walk included. A “store-only floor” cannot decide the cost of making a table warm for its immediate reader. A global pointer pass followed by low-byte read/modify/write means roughly 12n logical table bytes rather than one 4n write; only genuinely cache-local tiling or a sufficiently shorter computation can repay that.

A fixed-row hybrid is a different way to avoid writes. Use one tagged four-byte directory entry per B-row block. If an entire block lies in a common T affine cell, its entry supplies a base and T[q]=base+256(q mod B); otherwise it points to dense words. Ignoring final partial-block rounding, footprint is approximately

4n/B + 4(1-f)n,

where f is the fraction of rows in wholly implicit blocks. Saving requires f>1/B; at B=32, over 3.125%. This is only a space threshold. Dense exceptions add a dependent directory-to-table load, and markers require forced-dense blocks or a separately proved side structure. Census uniform blocks and **actual lane visits** before proposing a walker.

Lazy materialization is weaker when the walk ultimately touches every table line: it saves no final writes and adds first-touch control. Periodic traversal may differ; measure distinct touched lines, not run count. Do not overwrite pre-existing markers on first fill.

## 6. What a run can and cannot skip

Inside a common affine cell [A,E), suppose Ψ(q)=q+d and the output symbol is constant. For d>0, at most

1+floor((E-1-q)/d)

successive source rows remain in that cell; for d<0 use 1+floor((q-A)/(-d)). Cap by the actual output/segment limit and any marker. For d=0, finite requested output still bounds repetition. This gives a candidate repeated-byte emission and arithmetic jump, without consulting a move record per byte.

A long MTF zero run does **not** establish this condition: its image may jump far away immediately. Census valid consecutive trajectory lengths, not ell alone. Equal-offset lanes preserve their offset while they follow the same translations, but finding, grouping and splitting those lanes at boundaries can cost more than 32 ordinary independent loads.

The s-run/Precopy literature offers stronger repeated-trajectory certificates. A descriptor that skips several checked rows could avoid the old per-row neighbour test; it still needs completed source output and exact resume/marker/cycle handling. With only the crude r_s≤rs bound, optimizing n/s+rs suggests s≈sqrt(n/r), around four at r/n near .06–.07. That pessimistic bound is a warning about metadata cost, not a refutation: actual r_s may be better. Require reusable-trajectory and saved-native-work counts before repeating the failed copy experiment.

## Validation, scope and next decision

The accompanying [probes.py](probes.py) checks formulas, not a candidate decoder. It exhausts ternary columns of lengths 1–7 and adds 80 deterministic random columns up to 9000 rows: **3,359 columns, 108,244 rows**, no assertion failures. Checks cover translated two-step cells, cell-count equality/bound, weighted shard prefixes, spare-bit symbol/pointer identities, whole exceptional-block bounds, and representative 20-bit pointer endpoints. Results are in [probes.json](probes.json).

The exceptional path deliberately uses the exact F oracle; it does not test a proposed search implementation. Arbitrary L columns test permutation identities; they are not a bzip2 corpus. No native decoder, emitted listing, parallel implementation, malformed-input path, marker/scheduler proof, Miri/fuzz/mutation suite or timing ran. None is implied by the algebra.

Next value ordering:

1. Preserve #269 corrections/HOLD scope; freeze a matched list-kernel control.
2. Census C31 predecessor starts and exact W, then choose helper-sorted/direct placement controls.
3. Register a run-intersection pair-builder comparison **with unchanged pair consumer**. Keep the old failure and new question separate.
4. If that construction is competitive, isolate D3's first-symbol common path and dynamic exceptions. Only then contemplate packed one-hop/hybrid navigation.

C3 accounting: expected run structure to remove real work; found concrete removable materialization, bigram construction and symbol-lookup paths, plus a confounded emitter comparison. **Zero measured whole-decoder gap closed.** Learned that O(r) construction, dense consumption and small symbol metadata can be combined without committing to a move walker. Next experiments have value because each removes a named old cost and has a matched falsifier; general move/lazy/external-memory work does not currently outrank them.

Migration and routing audit were checked; only the five recorded resolved historical misroutes appeared. Own-state commits only, no implementation, timing, CI dispatch, merge or automation change.

