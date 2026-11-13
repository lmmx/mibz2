# Equal-unit elision: sound composition, no-build on complete cost

13 November 2026. Direct owner task, notice6854993878, Coordinator checkpoint6855132749. **Verdict: NO-BUILD under the specified feasibility gate.** The finite composition is sound, but the proposed cheap deferred walk is not a complete representation compatible with the unchanged writer. The straightforward compatible construction restores the omitted word work and adds deferred storage/traversal. Avoiding that restoration requires an unpriced representation and lifetime scheme. No strict positive conservative work bound is established. This rejects progression of this proposal, not the mathematical possibility of equal-unit elision.

The Coordinator reconciled the existing MONODROMY6850491677 queue: this executor owns this finite engineering assessment; duplicate execution there is held. MONODROMY retains mathematics and Pathfinder architecture. No native kernel, real-corpus count, performance run, new writer or dispatcher was built. The owner's conditional implementation gate, rather than a missing permission or CI check, is the stopping reason.

## Exact sources and reproduction

- Master3e7e294cc10f29495e9828181345d74233c549d4; journal §§5.6,6.4,8.8 and original examples/rauzy.rs.
- Baseline: corrected production Cellsd4f0e8fb3ac68fa649e1810dff58dc06e9c545c2. Its src/cells.rs SHA256 is42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2.
- Guard37993781bd9578eae95c599583e387180d673659180 is a separate completed experiment. Its induce_cells function is byte-identical to baseline: cd99d9830628cce21123b7e8019f2a87cb0c86a0941163fd62af8b258266d2c9. The ledger below uses the365 constructor and unchanged365 writer, not a mixture with379's omitted passes.
- Independent assessmentfa79d39ffec5cbe0de081ad04ac3fdffa24a0f47, current Coordinator6853665214/6855132749, full amended MONODROMY6850491677.
- PLAN and initial checker frozen in d3eeb4b before execution. `python3 check.py > results.json` uses only the Python standard library. No hardware timing is collected. All finite results are reproducible, not sampled corpus estimates.

The first run is retained as initial-results.json. A subsequent observational extension adds deferred-child consumer witnesses and explicit rank-descent start counts on the SAME inventory; every original count is asserted unchanged. It does not alter the gate. The Python model keeps full symbolic words to make assertions easy; it is deliberately not a compact implementation or an allocation benchmark.

## Invariant and eligibility

Let pi be stable LF and K=[0,m). Each current top interval I_a maps by translation to its bottom interval J_a under first return to K. Its label w_a is the REVERSE of the ordered forward-LF emission itinerary, with the selected original-row occurrence marked. Length lambda_a is interval width; |w_a| is return time. They are different quantities.

At the right end, A is the last top letter, B the last bottom letter. Check A!=B FIRST: A=B is a closed orbit group, not a concatenation. With ra=rb=1, I_A=J_B={m-1}. For x in I_B, the next induced visit goes through that removed singleton and then to J_A. Thus the survivor has width1 and ordered word w_A w_B, return time |w_A|+|w_B|. Other points' returns are unchanged. Both orders lose their rightmost point and B occupies A's old bottom position. m decreases by1.

A deferred descriptor may stand for that concatenation rather than materializing bytes/rules immediately. It must retain ordered children or an equivalent ordered fragment sequence, both lengths (or equivalent retrieval), and mark information. A count, a link to only one child, or the fact width=1 is insufficient. For min(ra,rb)=1 with ra!=rb, a surviving wider interval remains and the production sweep promises a fresh opposite slot. That is NOT this equal-unit shortcut.

A useful restricted fact survives: the new width-one occurrence cannot be split by later positive-integer interval induction. At its next consumption it is either one entire child of another width-one cell or part of a closed orbit. It has at most one live consumer; no fan-out arises for that deferred result. This permits a forest of owned occurrences, but not automatically a singly linked chain: two deferred children may merge. Immutable underlying word definitions can still be shared by other occurrences. Mutable splicing may only touch unique occurrence records, never those shared definitions.

**Mark and phase.** Split orig_ptr as the original code does, so at most one live letter carries its marked occurrence, and that letter has width1. Concatenation propagates that occurrence; if it is in B, its index increases by |w_A|. The final selected-cycle word has length p and mark index j; output begins at (j+1) mod p and repeats for exactly n bytes. This preserves the inverse-LF orientation. Tracking only the selected byte value is insufficient when bytes repeat.

**Orbits and termination.** An unmarked A=B group can be dropped; marked A=B terminates. Do not concatenate a closed cycle to another orbit. Each ordinary or deferred logical step decreases m, and every deferred child has an earlier creation index. Those facts prove finite model termination. They do NOT prove that an arbitrary native pointer-splicing scheme has no cycles or stale references. The selected p need not divide n for an arbitrary column; output still follows that cycle for n bytes.

## Finite results and decisive witnesses

All3,586 binary cases (n<=8, all origins),6,015 ternary cases (n<=6, all origins),20 fixed larger cases and3 budget checks pass. The alphabet inventories overlap intentionally; they are9,621 executions, not9,621 distinct columns. Every intermediate state is checked against independently constructed stable LF: return time, ordered labelled itinerary, affine destination and mark. Root reconstruction is independently compared with n inverse-LF steps. Eager, deferred-reference and materialized words agree.

| Model count, excluding the three separately recorded budget checks | Exact total |
|---|---:|
| Logical cells / eligible equal-unit / other full cells | 29,077 / 20,630 / 8,447 |
| All width-one cells (NOT eligibility) | 28,556 |
| Eligible cells carrying the mark / with word length >16 | 12,160 / 10 |
| Eligible short unmarked cells | 8,468 |
| Deferred records / child references | 20,630 / 41,260 |
| Deferred child consumed by later ordinary full cell | 1,677 |
| Equal-unit cell joining two deferred children | 858 |
| Selected-root reference expansion visits / deferred visits | 56,099 / 16,593 |
| Hypothetical lane-start descent: internal / leaf visits | 42,816 / 17,255 |
| Materialized compositions / deferred compositions restored | 29,077 / 20,630 |
| Original kept rules / small-arena bytes | 15,878 / 35,009 |
| Materializer's copy16 destination bytes / speculative rule-write bytes | 930,464 / 465,232 |
| Output bytes / oracle return visits | 61,355 / 293,740 |
| Short selected cycles / cases where p does not divide n | 8,100 / 3,707 |

These are logical/source-access counts from tiny cases. Reference expansion visits are NOT production writer operations. Selected-root deferred visits can be fewer than created records because unselected orbits are dropped. Native writer operations, actual allocation/RSS and fallback runtime were not measured.

Smallest witnesses below follow exhaustive binary n/lexicographic/origin order; long witness is from the fixed larger inventory, not a global-minimality claim:

- L=`ba`, origin0: eligible widths1/1 compose `a` with marked `b`. Removing the rule and retaining only `b` loses `a`; expected output is `ab`. Blind deletion is already wrong at n=2.
- L=`baa`, origin0: the first cell has widths2/1: unit width but ineligible. Its later equal-unit cell joins words of lengths1 and2. Unit interval width does not mean a one-byte return word.
- L=`baaa`, origin3: a deferred length2 child is consumed by an ordinary cell with widths2/1. Deferral cannot in general remain isolated until the final writer.
- L=`bbaaa`, origin3: an equal-unit cell joins TWO deferred children. Treating every deferred component as one next-pointer chain needs additional splice/end bookkeeping.
- L=`b`+`a` repeated17, origin0: an eligible step joins a length1 and length17 word. Long/marked grammar handling cannot be omitted merely because ra=rb=1.

## Complete-cost gate

The original eligible step reads its frontier state as usual, then performs TWO16-byte copies and TWO8-byte speculative rule stores, regardless of which word representation is kept. Its equal branch performs TWO4-byte width stores, TWO8-byte word stores and TWO8-byte crosslink stores:40 slot-write bytes. It allocates NO fresh slot on either side. Thus the tempting avoided total is88 destination bytes per eligible cell:48 word-construction plus40 slot-update bytes. This is source-level traffic, not measured instructions, cache misses or nanoseconds; frontier reads/prefetches and logical budget checks remain.

| Obligation | Removed versus added, with coefficient provenance |
|---|---|
| Eligibility | One ra/rb test on all S steps; comparisons/control cost cannot be charged only to E eligible steps. Source conditions are known; machine branch cost unknown. |
| Return-word retention | A simple fixed record of two full (length,word) descriptors is16E bytes, the same size as a grammar rule. An8E-byte pair of offsets instead requires retrieving the descriptors elsewhere and proving their lifetime. No per-cell heap objects proposed. |
| Word materialization for unchanged writer | Restores the same48E destination-byte operations if original composition is replayed. These are not saved construction. An actual compact-word serializer could avoid speculative writes, but that is the separately named representation-selective-store optimisation, not proven link elision. |
| Slot elision and routing | At most40E slot-write bytes are candidates for removal. The remaining live order endpoints must still locate their counterpart/next word. Routing/tag writes, reads and checks are not specified by the mathematical concatenation; their cost is unknown, not zero. If ordinary slots are retained with lazy descriptors, most or all40E remain. |
| Deferred access | Full descriptors:16E record-store+16E record-load bytes. Offsets into stable retired descriptors:8E record-store+8E record-load+16E descriptor-load bytes. Both simple representations add32E bytes before routing/control/starts; only8E of the maximum40E slot saving remains. A straightforward two-order forwarding scheme adds two4-byte forwarder stores and two4-byte resolutions per eligible cell: another16E, exceeding that8E margin. This is a counter-ledger for those explicit choices, not a lower bound for every possible encoding; machine cycles remain unknown. |
| Finding starts | The actual selected state supplies one root, not four evenly spaced deferred-chain positions. A concrete reference alternative descends known word lengths to three output-quarter positions. It costs42,816 internal and17,255 leaf visits here and can be linear in depth per start; p<4 cannot supply four nonempty output lanes. These starts are byte ranks, not guaranteed equally balanced deferred workloads. No free starts or fixed four-lane cost is assumed. |
| Traversal/output | Deferred children are ordered and unique occurrences, but may form a branching forest. Every selected deferred point still needs resolution; direct expansion needs stack/iterator/splice work and output destination assignment. Emission bytes, rotation, periodic repetition and long-factor F01 recopy are not eliminated by the model. |
| Refusal | Charge E+F logical cells against the original budget. If elision gets free budget, admission changes and worst-case link work can grow to O(n). Abort must discard deferred state and take exactly the old fallback; no refund or second attempt is priced as free. |
| Capacity/lifetime | An external8/16-byte record array adds8/16E live bytes, at most8/16B if E is budgeted. With production B=floor(4n/25), this is up to1.28n/2.56n bytes in addition to current buffers, before starts/stack. Reusing retired slots might avoid this allocation but needs an alias/lifetime proof and may conflict with speculative prefetch addresses. No such native proof or layout is claimed. |

The compatible illustrative construction materializes every deferred binary composition in creation order: zero word compositions eliminated, all20,630 restored, plus links. Its correctness does not establish cheapness. A concrete tag/forwarding layout which removes all40 slot bytes has not been supplied. Even granting complete elimination of40E slot-store bytes, the simple offset-record/two-forwarder choice adds48E byte accesses, before control, starts and lifetime checks. Keeping ordinary slots instead avoids those forwarding changes but forfeits the40E saving. Neither supplies the required conservative net-work advantage. Fusing records into existing links might change the ledger, but no correct layout or price is supplied; this is not a theorem that every possible layout must lose.

**Writer compatibility is a real constraint.** In365, a short unmarked descriptor points at literal arena bytes; a long/marked descriptor points into the contiguous16-byte rule region. Descending propagation followed by ascending expansion assumes every child rule precedes its parent. A deferred offset is neither kind. In the `baaa` witness, storing an ordinary parent before later appending its unresolved child violates that ordering. Options are: resolve eagerly before the parent (restore word work plus traversal); reserve normal rule positions (keep storage/scan work and fill holes); or rebuild/renumber reachable rules before writing (new global pass). Direct deferred-tree output is a different writer, explicitly outside this task. None has a demonstrated cheap complete implementation here.

Historical journal coefficients are retained with their limits:0.842r eligible,0.970r other cells,1.4ns/deferred point with four GIVEN evenly spaced starts,5.2ns with one, and no one-lane model benefit. They are not current365 full-operation prices. No coefficient was fitted to these finite results; no unavailable stage fraction is invented. The four-lane price cannot be imported while start discovery, useful lane balance and compatible output remain unpriced.

## Allocation, safety and seven-family limits

Baseline region capacity is sized from records and B before induction; equal cells already allocate no fresh slots. Reducing actual cell writes therefore does NOT reduce the original reservation. For a genuinely independent deferred array, retained high-water capacities must include the maximum prior-block link array together with current kernel/fallback/output/pos buffers, not only today's E. Release-before-grow and a logical E<=B bound would avoid simultaneous old/new link arrays; they do not erase retained capacity. Borrowing old slots requires proving each stored source descriptor survives every overwrite/prefetch/reuse path. A topological Python DAG supplies no such unsafe-memory proof.

The three budget witnesses preserve exact step refusal at0/1 and success at2 for `baa`. They are not a native admission or malformed-stream test. Production density/floor gates, CRC/RLE1/error behaviour and buffer-reuse safety are unchanged because no production code was changed; this phase does not claim fresh coverage of those contracts. Miri, fuzz, libbzip2, seven-real-corpus counts and native RSS belong after a passed build gate and were NOT run for this proposal.

The seven frozen families remain wikidata, onebyte, pattern, random, records, runs, text, with text generated from57f51bbc0ca62ac3ee1cc148a1010842eea6222a. None was re-created or decoded here. Historical233-block geometry, current431-block input, #378 route counts and #379 timings are not substituted for current equal-unit coverage. No speed or new allocation measurement is claimed for any family.

## Disposition

Expected: eligible full-cell work might be replaced by cheaper links and a short final walk. Found: semantic contraction works, but words can be long/marked, deferred subtrees merge, ordinary cells consume deferred children, and unchanged-writer normalization restores work or needs a new pass. The only claimed numerical closure is finite correctness; none of the runtime gap is demonstrated closed.

**Stop before Phase2.** The next useful input is one concrete bounded forwarding/fragment layout from the existing mathematical/architecture owners that preserves the writer contract and supplies a complete cost ledger, including starts and lifetime. Another census confirming0.842r, or another unpriced 4-lane model, would not change this decision. Coordinator should retain NO-BUILD unless that missing construction changes the gate. No automatic implementation, measurement admission, merge, release or default change follows this report.

Local validation: initial scoped Ruff check found a missing main docstring; fixed without changing model output. Ruff format/check, Python compilation, full frozen inventory and byte-identical result reproduction pass. The original and extended result files are retained; no numerical failure was hidden.
