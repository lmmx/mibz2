# Rauzy kernel: static cost and profiling handoff

Brief [6767027944](https://github.com/lmmx/mibz2/pull/160#issuecomment-6767027944), PR351 head **149307d6c20868a18edf9d835c30ca213a54ec6e** as fetched. Source examples/rauzy.rs blob 672b965c9d6c2736045b3e55a868280ac235464c; journal docs/journal/monodromy/2026-11-10-rauzy-veech.md blob ec57be774b68abe5ce0b8de1c7be5df90acd6fda. Governing spec a2da49638ac54d7763de1c167bd054f67ffdc763.

**Partial result: no fresh Callgrind/Cachegrind counts.** Executable lookup found no cargo, rustc, valgrind or callgrind_annotate. No Rauzy profiles/input were found in the workspace; the pinned journal's adjacent directory contains two Python mathematical checks, not stage profiles. More fundamentally, the example's main invokes Instant/elapsed for production, reference and kernel loops and has no untimed corpus mode. ROUNDS=0 is not a census mode: it skips construction and then checks empty outputs against the reference. Under the explicit no-source-edits/no-timing request, this is not an executable profiling recipe as supplied. No tool installation or altered driver would resolve the latter constraint by itself.

The requested instructions/run, instructions/row, L1/LL simulated misses/run and simulated branch mispredictions/step are **unmeasured**, not zero. No CPU latency has been inferred from a synthetic miss count. This report supplies source-level work and conditional ceilings, distinctly labelled; the requested dynamic profiling remains outstanding.

## Consequential source findings

### Kernel step, not instrumented reference

The reference induce includes touch/born/seated/members, depth/round/streak census and a grammar rule on every step. Kernel::induce is a different routine: 32-byte nodes, short-word arena and only retained long rules. Profiling the whole executable or the reference function cannot answer where the kernel spends its work. Similarly, order is Kernel::sort plus Kernel::link (JOIN disabled); runs scans the column separately. Assertions, grammar_crc, admissible, coincidences and the production comparison must not leak into kernel stage counts.

In the ordinary nonterminal Kernel::induce path, at source level:
- nine scalar u32 stores: three payload updates, four list-reseat writes and two equal-case writes (the latter target the shared dummy node when not equal);
- two unconditional copy16 operations: 32 addressed store bytes and 32 addressed load bytes, even for a long word, when both inputs become BYTES and neither short copy is retained;
- one unconditional16-byte Rule store, even for a short word, when nr does not advance;
- additional scalar loads, two software prefetches, capacity tests and possible arena/rule growth.

Thus **84 addressed store bytes per ordinary step** (36+32+16) before growth, return/drop paths, allocator effects or compiler removal/combining. At journal S/n=27,439,723/209,697,662=0.130854 this is10.992 source-addressed store bytes/row. This is NOT retired store instructions, unique cache lines, DRAM traffic or measured Ir. Losing-word construction is not free because only one representation is retained: the source writes both.

Cheapest count targets: distinguish long/short cases; quantify discarded arena copies and discarded Rule writes; attribute dummy-node stores; count capacity-growth frequency. Removing one side by a branch can reintroduce misprediction cost, so compare matched counts and eventual wall effects instead of assuming removal wins.

### Dependent chain

On ordinary steps, tl/bl select nodes; lam/prev loads feed comparison and selection of next tl/bl; that index addresses the next node's lam. This is a loop-carried load→compare/select→address→load recurrence. Same-node length updates can also feed the next step through memory. Independently, at/ab advance two linked prefetch cursors; prefetch does not remove the architectural links needed to advance those cursors. Word construction, list stores and growth checks share execution resources but are not all on the next-index recurrence.

A machine-cycle critical-path length needs the exact optimized listing, target CPU instruction latencies, forwarding and hit/miss assumptions. Source establishes the recurrence, not a29-cycle critical path. The reported14ns/step (about29 nominal cycles at2.1GHz) is average elapsed work per step of a serial implementation. It does not distinguish recurrence latency from stores, branch effects, instruction throughput, allocation or cache waits. Likewise “11ns at r=20k” does not prove absence of cache misses: nodes alone are640kB, before rules/arena and competing state.

### Rounds are ordered, segments within a round may be independent

The93 figure is median dependency depth in whole segments, not93 independent rounds. Rounds depend on earlier rounds. The journal separately reports median step depth3,861 and widest segment1,562; a segment may contain a substantial serial merge. There is no justified93× or S/93 speedup prediction.

A task implementation must resolve:
- work per ready segment and imbalance, not just median606 segments/round;
- discovery of ready segments, active-list compaction and next-round scatter;
- word/rule identifiers and arena allocation (currently shared top/nr);
- writes to neighbors and dummy nodes.

“Disjoint consumed letters” alone is not a proof that the current in-place store sets are disjoint: each step also writes neighbors and a shared sentinel. Reusing the current branch-free kernel concurrently introduces shared writes, including dummy stores. A round implementation needs explicit disjoint destinations or staged next-state output, thread-private dummy/storage or omitted inactive stores, and an allocation/ID plan. This is a prospective design obligation, not a finding of a race in the current serial example.

### Array/SoA is not automatically smaller or cheaper

Current Node is32 bytes: lam,len,word,pad and four32-bit links. Changing only the four links to16 bits leaves20 bytes of useful fields (24 including pad), and retaining align(32) leaves sizeof32. It does **not** halve nodes to16 bytes without further representation changes. State this before budgeting a cache win.

SoA can reduce bytes loaded for an individual field but can split lam,len,word and links across more lines when all are needed at an irregular endpoint. Arrays remove the linked recurrence only if the algorithm changes access order to streaming merges/scatters; merely replacing AoS with SoA leaves data-dependent indices.

## Conditional economic ceiling

These use the journal's existing local figures; no timing performed and no portability claim:
- production build+walk2.172ns/row;
- order0.525, induction1.826, write0.419;
- fixed order+write0.944.

Parity requires induction+new coordination <=1.228ns/row: at least **1.487× induction speedup** if all else stays fixed. A2× induction gives1.857ns/row total, only1.170× versus production;4× gives1.4005ns/row,1.551×. Halving production's2.172 needs induction+new costs <=0.142: **12.859× induction speedup with zero extra coordination**. Infinite induction speed gives2.301× at most under this fixed-stage assumption. These are inversion-stage bounds, not whole-decoder gains. Whole-decoder reduction is bounded by that stage's measured share and binding-worker overlap.

The original claim “32 cycles/run all told” is a target, not a ceiling derived from counts. Scheduler and representation cost must fit the same budget. If a proposed implementation scans all r letters in each of93 rounds, work is93r≈6.715n at r/n=.0722. Even35Ir per such visited cell is≈235Ir/row; the35Ir/cell threshold is meaningless without defining cells and counting all visits. A sparse active frontier may avoid this multiplication, but needs the actual sum of active letters/segments across rounds. Do not multiply pooled totals by a median depth as if it were an observed aggregate.

Use work/span/resource bounds: time >= max(total executed work / sustainable throughput, weighted dependency span, required traffic / sustainable bandwidth), plus scheduling where not overlapped. Counts alone supply neither sustainable throughput nor exposed latency; they cannot give a numerical reachable ns/step. Missing active-round census and compiled kernel profiles prevent a defensible SoA speed prediction.

## Exact untimed profiling handoff

Provide either archived profiles from this exact pin or a separately authorized timer-free counting harness with toolchain/corpus identity; the current request prohibits editing this example. Keep original source immutable and name any driver separately. Do not reuse instrumented elapsed values as performance evidence.

Required record per block and summed:
- n rows, r runs, S kernel steps, repetition count; JOIN absent (the source enables it on any RAUZY_JOIN value, including “0”);
- separate order(sort+link), kernel induce, kernel write; separate runs, oracle/census and setup;
- Ir, Dr/Dw, D1 read/write misses, LL read/write misses, conditional/indirect branches and mispredictions. Archive raw profiles, cache geometry/simulator version, binary/debug-info hashes and compiler flags.
- report pooled Ir/Σr, Ir/Σn and mispredictions/ΣS, not averages of per-block ratios. Also per invocation so “run” cannot mean either corpus repetition or BWT run without notice.
- inclusive stage costs including allocation/reset, matched first-use/retained-capacity conditions. Repetition counts matter; branch/cache warm-up from reference/oracle execution must be excluded or explicitly retained as the condition.
- optimized assembly and per-case histograms for recurrence, dummy writes, discarded representations; no fixed latency derived from Cachegrind misses.

Lowest-cost next change to price: avoid redundant representation work and dummy writes before building a full93-round scheduler. Segment-parallel induction remains promising, but its global passes and allocation can erase the saving; the failed chain prepass is evidence about those costs, not a rejection of independent segments.

## Delivery and disposition

The governing SPEC only permits state-branch writes plus REVIEWs on145. The inbox instruction to post directly to346 and substitute a LOG cannot widen this. This REVIEW asks the Coordinator to forward the report to MONODROMY through the authorized route. No promise to accept future346 assignments outside the pinned queue is made.

Static analysis complete; requested dynamic counts outstanding. No source edits, compilation, decoder execution, timing, PR or merge. Migration verified and routing audit run; old unrelated162 LOG routing issue remains unchanged.
