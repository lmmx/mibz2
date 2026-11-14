# Performance opportunities after #369–371

**Decision: do not build another contraction backend from this evidence yet.**
Compact global rebuilding is the stronger *maintenance baseline* than the stopped
treap; selective chunk updates have not established an advantage over that baseline.
For the actual first-return block, compact fields alone are insufficient: ordinary
hash-table growth can exceed the metadata gate even after two obvious buffers are
removed. Reusing the already-built phrase dictionary is a more concrete expansion
opportunity than origin marking, but neither qualifies the failed configuration.

The strongest next whole-decoder falsifier is the **existing #367 matched
research/production warm-inversion plus decode-context diagnostic**, subject to
Coordinator routing and its existing execution authority. It tests whether the
observed stage advantage survives the production port before paying for a different
induction algorithm. No new benchmark, real-block attempt, backend, timing or old
census was executed here. No production gain is established by this report.

Assignment [363/6808549538](https://github.com/lmmx/mibz2/issues/363#issuecomment-6808549538),
unchanged brief6807501300. [Plan](PLAN.md) frozen at
`3b9cc40e76200123a3ef4dc696f337c1217e554a`, pre-calculation post
[6810330001](https://github.com/lmmx/mibz2/pull/145#issuecomment-6810330001).
Analysis started 2026-11-12 00:03 UTC, bounded to 02:03 UTC.

## Evidence boundary and ranking

`INPUTS.json` pins the exact #365/367/369/370/371 sources. The original #370
results, case counters and SAME frozen nonoverlapping deletion schedules are
archived losslessly in `inputs/`. `price.py` reads those records without importing
an LF implementation. It verifies all original family aggregates, joins each
schedule by case/column/origin/family, and cross-checks derived record counts
against allocation/merge identities. It also verifies #367 route totals against
its original analysis. [pricing.json.gz](pricing.json.gz) contains every derived count,
per-family/per-case account and conditional layout calculation.

**Observed** below means previously recorded, not remeasured. **Derived** means
arithmetic from those observations. **Model** names a proposed representation and
its assumptions. Unknown terms stay unknown. Counts across different cases and
origins are not a representative workload mixture, simultaneous memory, or time.

| Priority / opportunity | Evidence and removed work | Added work, risk and disposition |
| --- | --- | --- |
| 1. Production cells port/writer and context | Historical i9 inversion advantage; #367 actual admission covers 84.117% of rows. Production writer contains memcpy/memmove sites unlike the research helper. | Matched port/context time is missing. **Pursue existing diagnostic**, not a writer rewrite or default change. Preserve long-factor recopy/overlap obligations and frozen #365 gate. |
| 2. Compact first-return lifetimes/hash cons plus dictionary reuse | Actual #371 has 70,559 live grammar nodes, 6,133 pieces, 2,716 dictionary words; redundant dictionary expansion accounts for 103,233 DAG visits. | Constructor hash growth, word lookup and output obligations remain. **Revise representation/expansion proposal**; this run still fails its gate and does not authorise a compiled-cost test. |
| 3. Compact two-order global batch | Same maps can use flat records; treap spends 883,875 pointer and 893,297 size writes. Packed implementation's sorts/rank searches are concrete removable targets. | Must pay cut planning, two-order remapping, grammar-ID order and output. **Retain as comparator design; defer implementation** until target relevance and missing incidence costs are established. |
| 4. Selective batched chunk contraction | Can copy only dirty chunks when genuine edits are sparse; stable occurrence IDs can localise attachments. | Global selection remains; dirty-chunk distribution, growth, cross-index repair and output-query overhead are unrecorded. **Revise**, not justified over a compact global pass yet. |
| 5. Selection/scheduling and cheaper refusal | Selection repeatedly pays for empty rounds; endpoint research reports fewer steps under a different schedule. | Selecting the schedule is not free; changing it changes word IDs/next rounds. Only two real blocks hit late budget refusal. **Defer** until costs on a relevant critical path warrant it. |
| 6. Selective origin marking | Exact occurrence invariant from #370 survives; checks can cease after discovery. | Half the successful cases already anchor; preserving early unanchored rejection retains reachability work. #371 saves zero origin steps. **Stop as a standalone performance task** on current evidence. |

Ranks are expected information/engineering value, not a predicted wall-time order.
Reusable summaries remain deferred: identical words do not imply identical routes.
The #369 `babaaa` witness has distinct destinations for equal `ab` words; no
content-keyed summary or oracle-built path is credited as a priced constructor.

## 1. What whole-decoder headroom is actually known

At master2635f04, the run route is scanner/Huffman → `runs::front` →
`invert_runs` → RLE1 → CRC/output. Segmented is the default. The cells proposal
attempts bounded admission/construction and calls the existing Segmented body on
refusal. Neither local contraction nor phrase induction currently supplies that
production route. Reducing their maintenance does not directly reduce today's
decoder critical path.

| Component | Evidence / recoverable term | Cost that must remain charged |
| --- | --- | --- |
| Extraction/order | #371 read 56,386 real records, coalesced to 51,184 runs. Cells slots uses two counted-record passes. | Reading/coalescing, byte-prefix construction, origin split and storage/reuse. No expanded-column oracle shortcut. |
| Boundary overlay/splitting | #370 118,213 overlay records and 8,895 treap splits. | Exact source/image refinement; stable intermediate split count is not a minimal batch-cut count. |
| Global selection | 222,766 input visits, 187,765 color queries. | Still global in all compared schedules, even when a round deletes nothing. |
| Sort/remap | Packed 406,949 sort comparisons; 173,944 rank and 112,413 keep-search comparisons. | Ordered cut planning, prefix fields, cross-indices, allocation/copies and coalescing replace these operations. |
| Grammar | #371 123,401 pair queries, 70,559 unique nodes. | Construction, hash cons, growth and later expansion. Equal content never substitutes for occurrence routing. |
| Origin | #371 already anchored; #370 declines expose a real early-rejection advantage. | Reachability or exact occurrence phase, plus short/multiple-cycle semantics. |
| Residual traversal | #371 135,716 tokens and 1,723,659 residual-search comparisons. | Token lookup and emitted length n; a shorter dependency chain is not less total work automatically. |
| Expansion/writer | #371 1,664,146 residual DAG visits; actual production writer differs from research helper. | Output bytes, dictionary lookup, rotation/repetition, long-factor copies and correctly initialised reusable buffers. |
| Admission/fallback | #367 196 admitted, 185 density-refused, 2 budget-refused, 48 column blocks. | Probe overhead, any abandoned construction and unchanged fallback. No gate relaxation or relabelling floor semantics. |

#367's 431 blocks contain 209,706,557 rows. Admitted rows are176,398,965;
budget-refused rows1,799,975 (0.8583% of rows); density-refused rows31,507,152.
These are **row weights**, not critical-worker time fractions. The 198 omitted
small blocks have only8,895 rows, but their per-block overhead is unmeasured.
Cheap budget prediction may save attempted work, but a large missing gain cannot
be assigned to that 0.8583% without measuring its cost. Density already refuses
before allocating the cells arena; changing admission is not this assignment.

Historical i9 research cost on admitted rows was approximately0.247–0.250 ns/row
order,0.654–0.658 induce and0.2123–0.2126 write. They are conditioning-specific
historical observations, not times for #371 or the production port. The old
admitted+fallback counterfactual saves70–74 ms of inversion while omitting refusal
work. None establishes current worker occupancy, scanner starvation, or total
decoder time. For a matched measurement only, `decoder_ratio = 1 - f*(1-s) + e`,
where f is the baseline critical-path inversion fraction, s the *production*
inversion ratio and e added work normalised to baseline decoder wall time.
Here f, s and e are not measured together. No numerical decoder prediction follows.

#365 corrected headd4f0e8fb3ac68fa649e1810dff58dc06e9c545c2 remains **DEFER** under
[Coordinator6790163736](https://github.com/lmmx/mibz2/pull/145#issuecomment-6790163736):
default onebyte4t+2.2% [0.5,4.0], records2t+2.6% [0.8,4.4], pattern1t+0.7%
[0.0,1.4] flagged; primaries neutral, max RSS+1.5%. Its independent F01 correction
was accepted, but the specific all-default-arms-level gate failed. Neither
historical binary identity nor the generic secondary tolerance replaces that
verdict. No rerun or layout perturbation was made here.

## 2. Compact batches versus local chunk updates

### What the identical schedules tell us

| Derived schedule/account quantity | Value |
| --- | ---: |
| Cases / rounds | 3,108 / 12,429 |
| Active / empty rounds | 2,770 / 9,659 |
| Selected deletion intervals / deleted rows | 8,514 / 76,751 |
| Rows before / after active rounds | 222,168 / 145,417 |
| Surviving rows with changed packed rank | 144,668 (99.485% of active survivors) |
| Surviving rows before first deletion | 749 |
| Selection piece-rounds, one order | 111,383 |
| Raw records after cutting holes | 23,630 |
| Canonical records after active rounds, summed | 23,445 |
| Packed allocated records including setup/merges | 52,854 |
| Treap allocated edges / nodes | 37,934 / 75,868 |

Raw records are independently recovered as `(rank_queries - active_rounds)/2`
and equal saved coalescing checks. Allocation equals initial29,039 + raw23,630
+ merge185. This avoids inventing a per-round trace. Row-rank churn is **not**
dirty-record churn: a long unchanged interval may need one uniform shift. It
does refute the assumption that local deletion leaves packed suffix coordinates
unchanged. Chunk-prefix or lazy-rank metadata is necessary to avoid those writes.

77.71% of rounds are empty, but emptiness was learned by global selection; it
cannot retrospectively skip that selection. On the 372 entirely inactive cases,
selection still made6,904 input visits and3,804 overlay records. A new seed can
recolor every word, so a cached empty round does not certify the next seed empty.

The `alternating` family alone accounts for81,922 of111,383 selection piece-rounds,
16,439 raw output records,714,381 pointer writes and790,016 size writes. Its largest
input has128 rows. Periodic/aligned families have many rows but few run records.
This strongly weights the aggregate comparison toward tiny alternating maps;
it is not evidence that a real900k-row block has the same edit locality.

### Price a real compact competitor first

Let R be pre-round records, K selected overlap pieces, S distinct interior cuts,
M uncoalesced survivors and Q final records. These count records, not bytes/rows.
For the frozen nonadjacent deletion (`D ∩ F(D) = empty`), collect cuts on the
incoming source interval and outgoing source interval of each selected overlap.
At most four interior cut endpoints per selected overlap are proposed; duplicate
endpoints must be deduplicated. Each cut splits the same edge in both orders.
This is a planning bound `S <= 4K`, not the measured minimal refinement count.

A credible flat design retains source-order records and a bottom-order ID vector:

1. Charge the same global overlay and color/marker selection (2R input visits).
2. Intern the K concatenations in the **original selection order**. Reordering
   interning changes numeric word IDs and later color seeds. The8,541 prepared
   word assignments/queries paid by the treap do not disappear by renaming it.
3. Group/deduplicate cuts: sort O(K) endpoint keys, or bucket them with an O(R)
   pass and extra headers. Stable IDs require an explicit old-edge/fragment map.
4. Refine both orders, splice incoming source attachment to outgoing image
   attachment, remove the other endpoints, then compact/remap once. Assign
   source ranks and image ranks with two ordered prefix passes, and scatter the
   resulting destination ranks back through the new-ID map.
5. Coalesce iff adjacent in both orders with equal words and contiguous routing;
   merge across holes, update the bottom ID vector/forward map, and transport the
   selected occurrence. Finally traverse and expand the same selected cycle.

This replaces repeated tree rotations/size repair with array passes. It cannot
claim zero writes: two coordinate writes per raw output give47,260 fields here;
one pass over both canonical output orders gives46,890 entries. Cut sorting,
mapping, coalescing and setup remain additional. As a **payload model**, reusing
the raw buffer for coalescing allocates initial+raw=52,669 four-u32 records and
the same number of bottom IDs cumulatively:1,053,380 bytes of exact-fit fields.
That excludes transient cuts/maps, grammar, old/new capacity overlap and output;
it is neither an implementation nor peak memory. A two-order ID representation
adds an index to packed's four fields; it is not free stable identity.

Existing packed/treap cumulative record-field counts are211,416 versus644,878
slots, before hash tables and language overhead. Pointer-sized tree fields and
u32 packed fields are different byte widths. No slot-count ratio is called a
memory or runtime ratio. Their final output still writes223,735 bytes across
cases, expands14,351 selected-cycle bytes and materialises225,223 repeated bytes.

### When selective chunk rewriting could beat that pass

With at most B records per chunk, D dirty chunks across both orders, and G
directory entries, a batched local design pays:

- the same global2R selection and K word operations;
- cut grouping/refinement, O(BD+S) record copying, split/overflow allocation,
  occurrence-ID mapping and adjacency checks at chunk boundaries;
- directory prefix rebuilding O(G), or explicit rank-update/query machinery;
- retained chunk/ID metadata and slower two-level rank/lookup during output.

The global design pays O(R+S) sequential refinement/rebuilding instead, with
simpler final coordinates. Sparse K is insufficient: cuts can touch both orders,
overflow chunks or invalidate cross-indices. The local design is plausible only
when **BD is materially below R across relevant active rounds**, directory/rank
and output penalties fit the saving, and enough total time is maintenance rather
than the common selector. If D approaches G, rebuilding is the better candidate.

The saved schedules contain row deletion intervals, not source/image fragment
IDs per round, dirty-chunk occupancy or marker discovery order. Exact D, S and
cross-index movement cannot be recovered from the aggregate counters without
another construction/replay. We did not perform one or assume ideal locality.
The given sparse-update claim therefore does **not** clear the prospective screen.
Nor does the broad flat prototype: target whole-decoder relevance is absent and
the existing matched-port diagnostic has greater immediate information value.

MONODROMY's historical array-round construction already lost at1.99× the old
column build+walk before writing. That was a different schedule/implementation;
it neither proves all flat passes bad nor permits treating fewer sorts as a win.
Its proposed1.279r versus1.812r step ordering also leaves selection cost unpaid.

## 3. Compact metadata must include growth and phase lifetimes

#371's Python lower bound23,511,653 bytes still fails4n=3,599,724. For clarity,
price a specific *hypothetical* ordinary layout using its observed cardinalities:

| Constructor live/reserved payload model | Bytes |
| --- | ---: |
| 51,184 source records, four u32 fields | 818,944 |
| Original u16 run input | 112,772 |
| Redundant source-start index | 204,736 |
| Anchor ID/start/prefix arrays | 8,152 |
| 70,559 grammar nodes, children+length (three u32) | 846,708 |
| Open-addressed hash cons,131,072 slots ×12 bytes | 1,572,864 |
| Completed6,626 records ×16 | 106,016 |
| Frontier peak5,303 ×16 | 84,848 |
| Sum of maxima / exact-fit payload envelope | **3,755,040** |

This is a reservation model, not a proved simultaneous peak: the queue and
completed maxima need not coincide. Dropping the raw-input allocation after
conversion and searching the source field directly instead of keeping starts
reduces that envelope to3,437,532, only162,192 bytes below4n. In a real decoder
the caller may retain the run buffer anyway; omission must be declared, not
hidden. Model excludes output, allocator overhead, alignment and vector capacity.

**Ordinary doubling independently breaks this proposed compact design.** At
75% load, growing65,536 slots to131,072 while keeping both tables live occurs
with49,152 existing grammar nodes. Source map818,944 + anchors8,152 + grammar
589,824 + old/new hash tables2,359,296 = **3,776,216 bytes >4n**, even before
queue/completed records, raw input, redundant starts, allocator or output.
This is a conditional lower bound for that stated growth policy, not a claim
about all hash tables. Freeing hash cons after construction helps expansion
but cannot remove its earlier peak. In-place growth, capped preallocation or
another dictionary need their own work/lifetime proof; exact final H cannot be
known for free before construction. Grammar-vector growth must also be charged.

Conversely, the small Q and dictionary payload83,454 bytes do leave a plausible
compact representation question. The failed Python object layout is not a native
impossibility result. The right reopening condition is a finite allocation/lifetime
ledger including growth, bounded refusal and a concrete lookup policy—not a sum
of final field sizes fitted after seeing H. No new allocation implementation ran.

## 4. Construction and expansion before origin micro-optimisation

The #371 dictionary comprehension expands6,133 piece words but retains only2,716
distinct IDs. A binary word DAG expanded as a tree uses2L−1 visits for length L.
The saved totals therefore give:

- observed repeated expansion:267,425 visits,136,779 bytes;
- expand each distinct ID once:164,192 visits,83,454 bytes;
- potential avoided work:103,233 visits and53,325 bytes in each of the leaf,
  payload-copy and reverse-copy phases;
- added work:6,133 membership decisions/lookups and changed dictionary insertion
  behaviour, plus the existing lexical ordering and occurrence routing.

This is exact conditional arithmetic, not a measured new constructor. It saves
dictionary work, not E=124,080, the123,401 hash-cons queries or the selected
135,716 token visits. Storing actual words has already been paid in #371; it
must not be regenerated by an oracle for a future cheaper writer comparison.

The larger possible saving is to reuse those words for output instead of doing
1,664,146 residual grammar visits. Token lookup135,716 times and899,931 emitted
bytes remain. For this already-anchored, one-cycle block, copying the dictionary's
already-reversed word bytes into the output from right to left could remove up
to2,699,793 counted logical writes from the current append/copy/reverse sequence.
This optimistic calculation excludes zero-initialisation, dictionary lookup
structure, arbitrary-origin rotation and short-cycle repetition/placement.
For example, pre-zeroing an n-byte destination adds n writes. A length prepass
would add another135,716 token traversals and their lookup cost. A proposed
single-pass suffix writer instead needs a valid arbitrary-cycle completion
argument; no such implementation or correctness check is claimed here.

Unlike origin marking, this targets substantial work actually observed on the
real block. It remains secondary to resolving constructor memory and matching
port costs; the frozen real-block failure is not converted into a pass.

## 5. Selective origin handling: bound it, preserve early rejection

| Saved origin group | Cases | Separate LF steps / search comparisons | Original marking checks |
| --- | ---: | ---: | ---: |
| Already on delimiter | 3,107 | 0 /6,278 | 28,893 |
| Successful non-anchor | 3,120 | 6,956 /18,751 | 34,620 |
| Selected cycle without anchor | 458 | 530 /1,086 | 1,336 |
| Delimiter absent | 336 | 0 /0 | 0 |

Already-anchored cases require the original single point lookup/rank, not
construction marking. A fast path can hand its lookup result to the remaining
search when it misses, avoiding a duplicate lookup; this preserves the existing
algorithm rather than adding a new saving. #371 falls in this group (one lookup,
15 comparisons, zero steps), so it supports no origin-search headroom claim.

For non-anchor successes, a stop-on-discovery marker would execute between3,120
and34,620 containment checks across the saved cases. Discovery times were not
recorded; reporting an exact improvement would require unsaved work. Each
containment test adds one or two comparisons; a `marker is None` guard checked
on every interval also adds E tests unless the loop changes phase. State includes
a found flag/phase and the two-scalar `(anchor occurrence, word offset)` record,
plus a grammar-length read when the prefix is nonempty.

The constructor's existing interval-end arithmetic can be shared, but no existing
production predicate tests membership of this origin. A debug tiling assertion
is not a paid production branch available for free reuse. Preserving the
delimiter section and occurrence identity also forbids inserting origin as an
extra delimiter: #370's `bba` counterexample remains decisive.

Most importantly, the unanchored458 cases were rejected **before construction**
by the separate search. Marking constructs1,336 intervals and362 extra grammar
nodes before rejecting. To preserve that early property for arbitrary LF, one
must retain a reachability search or supply a separately priced certificate that
the selected cycle meets the delimiter. An offline all-cycle oracle is not free.
The adversarial `(ab)^64` comparison remains1 search step/8 comparisons versus
127 constructor steps/1,778 constructor-search comparisons before a decline.
Keeping the original precheck plus marking pays both. An incremental interleaving
also does extra construction before a rejected cycle; it does not preserve the
same early rejection. **No standalone origin implementation is justified.**

## Next action, stop rule and validation

**Prospective backend screen: not cleared.** No candidate currently has both a
whole-cost case against compact global rebuilding and demonstrated whole-decoder
relevance exceeding the matched production-port diagnostic. Therefore zero new
backend falsifiers ran (zero of the20-minute allowance), and no existing census,
constructor, real block or workstation job was repeated. This is the required
bounded stop at pricing, not a claim that local contraction cannot win.

The strongest single next falsifier is #367's already-described matched warm
production/research inversion with decode-context trace on the same corpus,
machine/toolchain and conditioning. Its decision is concrete: if warm production
loses the historical advantage, price the writer/port difference; if warm wins
but decode-context inversion does not, investigate conditioning; if inversion
wins while total does not, attribute the remaining stages/reader. Keep all
fallback/refusal costs and exact output checks. This report does not authorise
its execution, change the prepared #366 diagnostic, or bypass workstation readiness.

Reopen selective batching only with a named target and charged cut/dirty-chunk,
cross-index, selection and output costs that plausibly beat a global compact
pass. Reopen compact first returns only with bounded peak allocation including
growth and actual constructor/expansion costs. Reopen origin handling only where
relevant long-prefix frequency and an economical early-rejection certificate
exist. Reusable summaries still require exact routes, words and marked occurrence
phase; graph structure and alternation distance remain outside scope.

What changed in understanding: the arithmetic narrows locality and memory claims,
and identifies already-paid dictionary bytes as a concrete expansion resource.
What performance gap closed: **none measured**. The next diagnostic has higher
expected information value than another synthetic backend because it addresses
the existing real-decoder integration boundary directly.

Validation uses Python3.12.14 and standard library only:

```sh
python3 docs/journal/2026-11-12-performance-pricing/price.py /tmp/pricing.json
python3 docs/journal/2026-11-12-performance-pricing/price.py /tmp/pricing.json --check
gzip -dc docs/journal/2026-11-12-performance-pricing/pricing.json.gz > /tmp/saved-pricing.json
python3 docs/journal/2026-11-12-performance-pricing/price.py /tmp/saved-pricing.json --check
```

The first command requires a new output path; it never overwrites evidence.
These commands validate arithmetic, input identities and original aggregates;
they are not semantic tests of unimplemented candidates. Scoped Ruff check/format
and Python compilation pass. Exact source/data hashes are in INPUTS.json and
pricing.json.gz; source-file identities can be checked with `git show SHA:path`.
No independent self-review, production correctness/B5, or all-green CI claim.
Direct coordination audit is blocked by proxy tunnel403; route selects145 and
connector readback verifies published messages. Durable cursor/REVIEW will mark
only this completed assignment. #366 remains deferred and implementation incomplete.
