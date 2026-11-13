# Cells live-path: remove unnecessary writer bookkeeping before widening the search

Inbox [6850318070](https://github.com/lmmx/mibz2/pull/160#issuecomment-6850318070), 2026-11-13. Saved-evidence review; proposals only. Policy: docs/team SPEC and SPEC.OPTIMIZER at a2da49638ac54d7763de1c167bd054f67ffdc763. Source d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2; recipe cb73bf909397e2fff8c90f0a00437061ad06478e. Evidence cells-live-path/20261113T101816Z.tar.gz at 01f5bc23ea2242969b084ebff7bf3b178814f3a2, SHA-256 23c8712e1d1f2a64bbbb2f10232db7f21a8a8499826ac8f9ac42a2bd27374414.

**Decision: one bounded preparation for a structurally guarded all-small-spine writer.** First falsify eligibility with untimed reach counts; then, only if supported, prepare that one candidate for a separately admitted comparison. Do not implement in this assignment. Neither a general writer rewrite nor a conditioning rerun is justified by the present evidence. The line remains open; #365 stays DEFER.

## 1. Independent replay and its limit

Downloaded bundle digest matches. Replayed the pinned driver's pure schedule/reconcile/analyse functions over saved records: 120 measured jobs and nine checks; all per-child JSON/resource records match the schedule and runs.jsonl, accepted exits, recorded hashes/lengths, CRC, clock and budget reconciliation. Four archived executable hashes, three compressed-input hashes, driver/generator/trace hashes and the inspected cells.rs hash match the build manifest. analysis.json reproduces byte-for-byte. No binary was executed or rebuilt.

The six retained pattern/onebyte check byte files match their recorded SHA-256/lengths and oracle records. The three Wikidata byte files are absent. This review checks the recorded cross-arm/oracle identities, not missing decoder output bytes. Unlike the Coordinator's reported oracle reconstruction, this review performs **no decompression**. No fresh independent safety certification follows. Reproducer and machine-readable results accompany this report.

Cells/baseline wall ratios, complete decodes, ten pairs, frozen geometric log-t interval (df9):

| Corpus | Hooks off, ratio [95% interval] | Hooks on | Disposition |
|---|---|---|---|
| Wikidata | 0.881643 [0.856982, 0.907014] | 1.338008 [1.298813, 1.378385] | faster → slower; STOP quantitative phase attribution |
| pattern | 1.530338 [1.154570, 2.028403] | 2.913851 [2.767592, 3.067840] | slower in both; instrumented phase accounting permitted |
| onebyte | 0.535508 [0.331856, 0.864135] | 0.994670 [0.488371, 2.025853] | faster → unresolved; STOP quantitative phase attribution |

Both forbidden cost tables remain null. No phase subtraction, per-block causal estimate or overhead correction rescues them. The faster off-arm results remain their frozen whole-decode outcomes; they do not qualify a default, CPU family or thread grid.

Expected from the previous review: resolve live route, allocation and writer costs that the old stage experiment could not identify. Achieved: actual live route census and a permitted pattern instrumented cost decomposition. Gap closed is explanatory, not a production gain. No multiworker critical-path measurement was added.

## 2. What the writer actually does

Corrected source first sizes output, initializes every position slot, walks the marked chain to find rotation, walks it again to write short factors and seed long ones, scans all rules backwards to propagate earliest positions, scans all rules forwards to expand reachable rules, walks the marked chain a third time to recopy long occurrences, then repeats a short cycle to n. copy_long uses ptr::copy; a byte-at-a-time long-copy diagnosis is inapplicable to this pin.

Saved pattern census, identical across all ten traced decodes:

| Quantity | Count per complete decode |
|---|---:|
| Blocks / admitted / fallback | 22 / 22 / 0 |
| Attempted induction cells | 2,700,141 |
| Total rules from pos_initialized_bytes / 4 | 2,700,009 |
| Rows / sum of emitted cycle periods | 19,600,000 / 18,899,608 |
| Logical small-copy bytes | 18,899,586 |
| Logical long-copy bytes | 0 |
| Logical repetition-copy bytes | 700,392 |
| Position initialization bytes | 10,800,036 |
| Position growths / sum of requested growth sizes | 3 / 1,542,876 bytes |
| Cells-buffer growths / requested initialization | 1 / 11,864,624 bytes |

Small bytes + one marked byte per block equal the period sum; period sum + repetition bytes equal rows. These are logical counts, **not physical memory traffic**: copy_small may load/store a wider unit; allocator size and capacities are not DRAM bytes. Do not add capacities over blocks and call the sum peak RSS. Most full-block periods are near the block size; the input's seven-character alphabet pattern does not establish a seven-byte BWT cycle.

**Critical missing discriminator:** zero long-copy bytes does not prove zero long seeds or zero reachable expansion. A long seed can expand into short children at its first occurrence without invoking copy_long. The archive has no writer_reach.seeds/descending/expanded/spine-depth counts: those fields are cfg(test) in this source, not live hooks. The candidate below is conditional on this missing discriminator. No all-small eligibility claim is smuggled in from the corpus name.

Wikidata descriptive census: 431 blocks; 196 admitted, 187 refusals and 48 column-path blocks; 235 fallback calls. Its 148,399,232 cumulative requested Cells-buffer initialization bytes match the previous static prediction. onebyte: one budget refusal at 62,745 attempted cells, one fallback, no writer. These observations exclude “onebyte's writer dominated” for this saved path but assign **no time** to either corpus.

## 3. Separate hooks, variation and causal hypotheses

Pattern exclusive instrumented means: writer 177.427977 ms; induction 25.013724; order 0.351239; attempt 0.030833. The inversion residual falls by 51.812392 ms. Summed exclusive buckets, including the process remainder, give baseline 75.058501 ms and Cells 219.108887 ms. Inclusive inversion is not an additional summand.

The writer bucket includes diagnostic work. prepare.py inserts add() at **every copy_small and copy_long**, not just one event per block. trace.rs add() takes a thread-local RefCell mutable borrow and updates both global and current-block BTreeMaps. This is a demonstrated mechanism for differential instrumentation cost; its duration is not isolated. The small-copy byte count bounds the number of positive-length calls only loosely: at least ceil(18,899,586/16)=1,181,225, at most 18,899,586 if each copies at least one byte. Exact call counts and length histogram are missing. A zero-length case would invalidate the latter bound; no reliance on it is needed for the decision. Do not call the entire writer bucket avoidable native work.

On/off inflation for Cells is 2.269679 [2.052584, 2.509736] on pattern; baseline 1.192022 [0.922897, 1.539627]. This is not an overhead estimator that may be subtracted from individual phases.

Timing variation is also real. Pattern baseline off has two ~31 ms observations among eight ~74–78 ms observations; Cells off has one ~69 ms observation among nine ~98–101 ms observations. onebyte off spans baseline 20.17–55.35 ms and Cells 17.21–56.20 ms. Retain every observation and the frozen intervals. Frequency, cache residency, launch-order conditioning, placement and scheduling remain hypotheses: this sample does not identify which caused the clusters. No trimmed subset, mixture refit, warm20+20, extra repeats or retrospective estimator change.

The most generous bucket ceiling for **any** pattern writer-only removal is 177.428 ms in the instrumented run, and even that leaves required writing to replace. If every other instrumented bucket stayed fixed, zero-cost writing would leave 41.680911 ms; whole-decode parity permits at most 33.377590 ms for a replacement writer. Ignoring favorable changes outside inversion gives the stricter accounting allowance 26.416596 ms (=51.813988 minus induction/order/attempt/residual). These are conditional instrumented arithmetic, not predicted off-arm times, confidence bounds or attainable savings.

## 4. Ranked constructions and cheapest falsifiers

Three constructions are priced, including the Researcher's newly published output-order alternative. Do not combine their ceilings.

### 1 — Guarded all-small marked-spine path: prepare first

During the existing first marked-chain walk, inspect the unmarked sibling length as well as the marked successor. If every sibling is <=SMALL, there are no long seeds. The second walk can emit the existing small copies and marked byte directly, after which only the existing repetition step remains. Position initialization, both all-rule scans and the third long-only chain walk are unnecessary in this branch.

Proof obligation: the marked chain partitions the period into its unmarked siblings plus the marked base byte; all siblings are available as arena bytes. No position is read on this branch. Preserve rotation, over-copy/slack assumptions and repetition semantics exactly; do not silently replace them with a new wrap algorithm. If any sibling is long, use the old general writer. The first-pass predicate adds one sibling-length extraction/comparison per marked rule and one per-block branch; it adds no allocation. Charge that predicate also on ineligible blocks. Keep only scalar state, no per-rule bitmap or intermediate output.

Let R be allocated rule slots, M marked-chain depth, E eligible blocks. Static removal ceiling is:
- 4 * sum_E R bytes of mandatory NIL initialization;
- 2 * sum_E R position-slot loop visits/reads;
- sum_E M visits to the third marked-chain traversal, including its address/modulo work.
For pattern, **if all 22 qualify**, these become 10,800,036 initialization bytes and 5,400,018 scan visits, plus an unknown third-walk depth bounded by sum R=2,700,009 (strictly older-rule links). If none qualify, removal is zero and the predicate is pure overhead. These are source operations, not emitted instructions or cache misses. Retained: induction, output growth, small-factor copies and repeated bytes. No claim that all 177 ms disappears; copy hooks remain in a traced build.

**Cheapest CI falsifier, before timed admission:** an untimed reach-only check at the exact source, accumulating per-block M, long seeds and reachable rules in scalars, emitted once per block. If pattern has no eligible blocks, park this candidate immediately. Even if eligible, differential output must cover stale/complemented output reuse, primary-row extremes, short cycles, SMALL boundary, and the existing F01/general-long-factor mutation witnesses; the long case must still exercise the old path. A candidate that removes a required seed or bypasses the existing malformed-input contract fails. New unsafe reach/control obligations remain B5 work for an independent reviewer, not discharged by this proposal.

### 2 — Output-order grammar emitter: mathematical alternative already queued

Fresh [Researcher report9c920c64](https://github.com/lmmx/mibz2/blob/9c920c64a5cc369b4b2934606ea7236f07fda6bf/state/reports/cells-adjacent-6850324757/report.md) proposes traversing W[start:p], then W[0:start], followed by existing repetition. Coordinator6850491677 has already queued its bounded mathematical falsifier for MONODROMY; no concurrent dispatch or ownership transfer is proposed here.

It can remove position initialization and both full-rule scans without all-small eligibility. On pattern, its demonstrated logical long-copy removal ceiling is **zero**: none was recorded. Byte-once output is a useful invariant, but the archived logical copy total is already n when marked bytes and repetition are included; do not sell n writes as a measured reduction of the 177 ms bucket. Native wide-copy/overlap traffic could differ and remains unmeasured.

Charge explicit traversal and pending-stack accesses. For positive-length binary expansions, a coarse bound for two full traversals is 4p−2 node visits per block; summed pattern ceiling 75,598,388, plus at most 2,700,009 rule steps for the initial marked descent. Interval pruning can sharply improve that bound, but the archive has no grammar/reach inventory to price the actual visits. This is a deliberately loose work ceiling, not a forecast. Up to R pending u32 parent indices could reuse pos's 4R allocation only after the exact simultaneous stack bound/recovery discipline is proved; no extra descriptor vector is free. General long-word sharing can make scalar expansion much worse than existing bulk copies.

Cheapest falsifier is the already queued safe tiny-reference inventory: at most128 grammars, roots<=4096, rules<=256, exhaustive rotations only for roots<=64 and fixed boundaries otherwise, plus F01 n550/period55. Check bytes, visits, stack high-water, refusal invariants and exact output-write count; no corpus reconstruction or clocks. MONODROMY owns that proof and inventory, Pathfinder any later storage/API integration. Prefer the narrower candidate1 guard if its premise holds; this general alternative remains viable if it does not. No B5 clearance follows.

### 3 — Lazy position initialization and live-rule expansion: defer

If long seeds exist but reach only R_live << R rules, avoid unconditional NIL filling and full-array scans. An epoch-tagged position table with a touched list can initialize positions on demand; processing reachable rules in reverse/forward topological order must preserve the original earliest-position and dependency semantics. Old rule index order alone does not make an arbitrary DFS visitation order valid. Sorting/ordering and duplicate suppression are new costs, not free.

Ceiling: replace the same 4R initialization and 2R scan visits by work proportional to reached vertices and edges; at most two child edges per reached binary rule. A straightforward design adds 4R epoch-tag capacity and up to 4R_live touched indices alongside the existing positions, tests/updates a tag on access, and may sort R_live indices. At pattern's summed R, that is up to another 10,800,036 logical epoch bytes plus equally large touched-index traffic if reach is dense; these sums are not peak storage. Epoch wrap requires a bounded reset. This loses its economic case as R_live approaches R and changes much more correctness machinery than candidate 1.

**Cheapest CI falsifier:** use the same untimed reach census. Dense reach, sorting cost exceeding eliminated scans, or any earliest-position/topological counterexample parks it. Zero observed long-copy bytes alone is not a favorable reach census. Do not build this alongside candidate 1; it is a deferred alternative, not another recommendation.

## 5. Whole-decoder alternatives and one selected next step

Deferred #366 compares frozen #361 candidate d33787e138426cb441db6975f54b2eccca02e89a against v0.2.9 b65b2aa54488f072cf005b8fe62461c32e6ef083, ten one-thread Wikidata pairs. #361 proposes restoring the earlier inlining/frame/code-generation shape; its own description explicitly does not establish frame growth as the cause. The recorded release regression target is ~0.9%, with adverse CI and tradeoffs preserved. #366 preparation is integrated, but the current channel reports no new workstation result; this review has no saved #366 measurement to rank numerically.

For production value, a general one-core regression is broader than this synthetic-pattern writer arm. For the **next bounded information step**, candidate 1's reach census comes first: it can cheaply eliminate or expose several million specific source operations without timing or changing the measuring stick. That is not evidence that Cells is globally superior to onecore. Unknowns: pattern eligibility, off-arm saving, real-corpus eligible fraction, worker/thread scaling, onecore's i9 effect. Candidate 3 ranks below the already-prepared onecore diagnostic until sparse reach is shown. Candidate 2's already-queued mathematical falsifier may resolve a more general writer route; it is not a second production implementation recommendation. Conditioning-only repetition ranks below a concrete reach falsifier and remains unadmitted.

**Exactly one recommendation:** Coordinator preparation of candidate 1, gated by untimed eligibility; no implementation or execution here. Fold its all-small predicate and cost questions into the existing mathematical-owner handoff rather than dispatch a parallel writer project. Current preparation uses the bounded tiny witnesses/proof work; no full-corpus decode is authorized to fill the missing reach census. Actual corpus eligibility is checked only within the nine checks of a separately admitted future candidate comparison below. If zero pattern eligibility or correctness failure, park candidate 1. Reopen only on a new structural discriminator/proof or an independent reach census identifying a qualifying writer path, not another timing sample. If eligibility survives but performance is inconclusive/adverse under the prospective rule, park its performance claim and return to the deferred alternatives.

For later admission only, a proposed fixed diagnostic:
- Two hook-free arms: unchanged corrected Cells versus only candidate 1; no sparse writer, copy primitive, induction, cap, default, spinner or conditioning change.
- Same three compressed inputs/digests; one allowed logical CPU, its topology and affinity recorded. One frozen toolchain/build/manifest and whole-process live-reader boundary.
- Nine untimed checks: old/new/oracle once per corpus, reach counters only in the check build, byte-exact output including retained files. Separate existing B5 regression/mutation coverage is a prerequisite, not counted as benchmark repeats.
- If prerequisites pass, exactly 3 corpora x 10 pairs x 2 arms = **60 measured fresh processes**, 0.25 s fixed settle before each, no warmups beyond the nine checks. Corpus order Wikidata/pattern/onebyte; old/new order on rounds 0,3,4,7,8, new/old on 1,2,5,6,9. Preserve all records; no retries/extensions.
- Ratio new/old; same nominal two-sided 95% log-t method, df9. Information-success requires pattern geometric ratio <=0.95 and interval upper <1, with Wikidata and onebyte upper bounds <=1.05. Any output/resource-validation failure is STOP; any unmet performance bound is park/inconclusive, not permission to add samples. No power or familywise production guarantee is claimed.
- This measures improvement over old Cells, **not parity with production**. Even success merely supports a subsequent separately registered production comparison and applicable CI/thread-grid adoption gates. Researcher assessment/Coordinator registration and Guardian disposition must precede that future run. No queue admission follows from this report.

#365 DEFER, B5 independence, MONODROMY mathematical ownership and Pathfinder architecture stand. No constructors or decoder code written, binaries run, compilations, new timings, warm20+20 admission, workstation launch, cap/threshold/default changes, merges or releases. Persistent task remains enabled.
