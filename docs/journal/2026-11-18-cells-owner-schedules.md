# Which occurrence should the Cells writer expand?

18 November 2026. Independent follow-up to Study 5. The baseline is its verified full-period last-use schedule for the W16 grammar produced by eager and lazy Cells. The inputs here are finished grammars. The frozen block was checked first, followed by the ten archived Wikidata traces spaced every 24 large blocks. A grammar-only count covers all 233 large blocks. [Study 5](2026-11-18-cells-live-reuse.md) · [MONODROMY verification](https://github.com/lmmx/mibz2/blob/633fc596f5a41c6858b70df384680dc3fa910a24/docs/journal/monodromy/2026-11-18-live-reuse-verification.md) · [Researcher verification](https://github.com/lmmx/mibz2/blob/05770d721d9d294c35496baa75fe716d9c7529c1/state/reports/live-reuse-6986296845/report.md)

**Every ownership tree does exactly the same number of rule copies and copies exactly the same bytes. What can disappear is the queue.** Selecting the first occurrence in cyclic output order allows a writer to expand there, remember the address in the spent rule itself, and execute every subsequent copy immediately. On the frozen block this makes 9,628 copies covering 735,582 bytes, with zero deferred commands and at most 23 pending right operands. The last-use schedule makes the same copies after its reverse scan. Merely changing which edges it selects has little effect on its measured spatial locality.

The first-occurrence walk is the one alternative here that passes a several-percent *writer* relevance screen. Its principal cost is a sequence of data-dependent visits to rules and short words. The evidence below includes that cost. It gives a structural candidate for independent review and leaves native performance open.

## The lower bounds

Take the reachable grammar of the marked root. A vertex is a stored rule identity, including a marked short rule. Edges distinguish the parent's two operand positions. Let $R>0$ be the number of vertices, $E$ the number of edges to stored children and $d_v$ the incoming degree of each nonroot rule $v$. Let $|v|$ be its byte length. An expansion-once tree selects one incoming edge per nonroot. Every chain of selected parents reaches the root because birth indices increase. Within each selected parent occurrence, its two children occupy the two prescribed subintervals.

| Quantity | Value for every choice |
|---|---:|
| Selected rule edges | $R-1$ |
| Unselected rule references to supply by copying | $K=E-R+1=\sum_{v\ne root}(d_v-1)$ |
| Bytes copied within the output | $\sum_{v\ne root}(d_v-1)\,|v|$ |
| Short or singleton writes | $2R-E=R+1-K$ |
| Total emitted factors, including copies | $R+1$ |

These are equalities in the model with one expansion per identity and an atomic copy from its representative at each other reference. A physical copy call could join adjacent spans. The equalities alone provide no lower bound of $K$ on calls or queue entries. A terminal root needs one short write and zero copies.

Last-use already attains two additional optima in birth order. Each reused rule's parents form one consecutive run on the same side. All but its last parent defer that child, and a cut can keep only one child alive. The commands from each source therefore form a single group, the fewest possible. In the reverse scan the owner also arrives *before* every deferred reference to that child. Every source **coordinate** is already available at command creation. Its **bytes** still await the later reverse visit to the child. Frozen reaches the minimum of 1,309 source groups.

The in-place capacity bound extends beyond last-use. In any suffix of $t$ scanned parents, exactly $t-1$ selected edges join nodes of the suffix. There is at least one further selected edge to an outside child or one short terminal edge, since the selected expansion tree is finite and leads to a terminal. Of $2t$ operand slots, at most $t$ can require commands. A different known tree can therefore generate two commands at one parent and still fit the whole suffix in spent rule slots. Discovering the tree has its own cost.

## The first occurrence in output order

Read the root spelling from its grammatical left end, starting the physical output cursor at the rotation obtained from the usual marked-child walk. The cursor wraps once after the marked singleton. An unmarked repeated factor is contiguous on both source and destination sides.

1. On first entering a rule, read its two operands. Store the current output offset in the high half of its first 64-bit operand field and zero the low half. Every genuine operand has a positive packed length, including marked operands. The two cases are distinguishable even at output offset zero.
2. Expand left before right. A short left operand can be emitted immediately and followed by the right. If the left is a rule, save just the right descriptor on a stack. The output cursor supplies its destination when it is resumed.
3. On a later visit, the tagged field gives the source and the incoming descriptor gives the length. Copy immediately and advance the cursor.

**Proof of readiness.** A tagged rule whose first expansion has started and is still incomplete is an ancestor of the current visit. Re-entering it would give a cycle in a grammar with strictly decreasing child birth indices. Thus every tag hit has a complete source in the already-emitted cyclic prefix. Different occurrences of the same rule are disjoint. This proves correctness with just one tag state, even across the physical wrap. The marked-path read precedes destructive tagging. The caller can discard the grammar exactly as in Study 5.

**Why this owner matters.** Under a single cyclic output cursor, with sources restricted to the completed prefix and one expansion per identity, the first occurrence is forced. If the earliest occurrence of a rule lay inside an ancestor supplied by a copy, the earlier source of that ancestor would contain a still earlier occurrence of the same rule at the same relative offset. The first occurrence must therefore be reached explicitly and expanded. In contrast, 4,765 frozen last-use commands have a source later in the root's *logical* spelling. They cannot all be executed in that order with those owners. Output-backed first-visit expansion is established grammar-decompression practice. The question here is what its exact W16 realization costs relative to Study 5. [Prior-art discussion and references](https://github.com/lmmx/mibz2/blob/05770d721d9d294c35496baa75fe716d9c7529c1/state/reports/live-reuse-6986296845/report.md)

## What the actual grammars pay

Let $P$ count stored parents whose left operand is a rule. The walk reads a first field $R+K$ times, reads a second field and stores a tag $R$ times each, and pushes and pops $P$ right descriptors. Its metadata count is $3R+K+2P$. Study 5's count is $4R+5K$. The difference $R+4K-2P$ is decidable from the finished grammar without computing positions, discovering the owners or emitting bytes. The recognition test is charged on every first visit and every repeat in the control ledger. No position array, visited table, prepass or restoration is needed.

| | Frozen | Ten archived blocks | All 233 large blocks |
|---|---:|---:|---:|
| Rules | 28,710 | 359,764 | 8,176,313 |
| Logical copies, either schedule | 9,628 | 106,902 | 2,528,876 |
| Left-rule operands $P$ | 18,046 | 220,940 | 5,074,013 |
| Last-use metadata field accesses | 162,980 | 1,973,566 | 45,349,632 |
| First-occurrence metadata field accesses | 131,850 | 1,628,074 | 37,205,841 |
| Reduction | 19.10% | 17.51% | 17.96% |

Every large block has positive field balance, from 8.05% to 20.94%. The ten actual walks used at most 20–30 pending descriptors per block, each eight bytes. No input-independent small-stack bound follows. For $L=1^{n-1}0$ and zero-based primary $n-2$, $n\ge4$, the singleton zero first joins the marked one at equality. The resulting width-one marked left operand repeatedly absorbs a one from the remaining right operand. The final marked rules form a left comb of $R=n-1$ vertices, $K=0$ and $P=n-2$. Its stack peaks at $n-2$ and its field balance is $3-n$. The four-row column already favors last-use, 12 fields against 13. [All per-block counts](2026-11-18-cells-owner-schedules/counts.csv) · [Executable small cases](2026-11-18-cells-owner-schedules/small.py)

**Locality and costs.** A cold, fully associative 32 KiB line model includes the rules, short-word arena, output, stack or position table, and in-place command queue. Across the ten blocks, changing to first-occurrence owners while retaining Study 5's two phases changed output-only line fills from 426,530 to 426,408. Walking those *same* owners in output order gave **180,974**. In the shared cache the rule fills rose from 143,262 to 315,205 and short-arena fills rose from 67,493 to 155,149. Net shared-cache fills fell **5.70%**, 716,908 to 676,064. Fills plus dirty writebacks fell **8.72%**. The model counts geometry and access order. It does not measure prefetch, miss latency or bandwidth.

The scalar ledger charges tag tests, tag encoding, cursor arithmetic, capacity checks, stack traffic and cold growth. Across ten blocks it favors the first-occurrence walk by **0.66%** at one unit per field or scalar operation and **5.23%** at two units per field. Equally, an extra cost of only 0.157 fill units on each net additional rule or arena fill erases the fill-only advantage. This is a real exchange of sequential rule reads for sequential output. Native writer time remains unresolved. [Access traces and per-block results](2026-11-18-cells-owner-schedules/first-visit.csv) · [Ledger, model and reproduction](2026-11-18-cells-owner-schedules/README.md)

## What changing the tree alone can accomplish

**Exact minimum dependency depth.** Charge weight $w_v$ to a copy of child $v$, zero to a selected edge. For atomic whole-source copies with arbitrarily many workers, the critical path is the largest total charge along any root-to-leaf path. A copy waits for precisely the deferred holes inside its chosen source. Following selected edges inside that source reaches the next such holes. Thus a chain of waiting copies is exactly a grammar path with its zero-cost edges contracted. The small checker also calculates this span independently by intersecting physical source intervals with all destination holes.

Give the root distance zero. Process children in descending birth order. If $a,b$ are the largest and second-largest distances at the incoming parent edges of $v$, select a parent attaining $a$. The smallest possible maximum distance at $v$ is $\max(a,b+w_v)$, with the second term absent for a unique incoming edge. Any other choice leaves a largest parent charged. This recurrence is monotone in every parent distance, so induction proves it simultaneously minimizes every vertex distance. Unit weights give exact copy rounds. Weights $w_v=|v|$ give exact byte-weighted span. These are separate objectives.

Frozen admits **four rounds against five** for last-use, or **4,444 against 4,786 bytes** on a longest weighted path. Across the ten blocks, last-use needs four to six rounds and the optima are three or four. The genuine W16 column $1^4 0^{67}$ at primary 3 needs two copies totaling 35 bytes. Changing owner halves the rounds from two to one, with exactly the same work. Tracking parent distances and choosing winners costs work on long edges. Neither optimum eliminates a copy in the serial writer.

**Spatial choices.** For fixed selected parent positions, the median offered coordinate minimizes the sum of start-address distances for one child. Repositioning a parent can move the offers of its children. On $1^5 0^{83}$ at primary 4 the local median with last-use tie-breaking gives distance 88, while exhaustive enumeration gives 70. On the ten blocks the median reduces total distance by 34.0%, yet birth-order output-only fills rise from 426,530 to 426,621. Even an oracle choosing parents independently for each incoming reference can put at most **13 of 106,902** source/destination starts on the same 64-byte line, against eight for last-use. A separate diagnostic sorted commands by dependency round and destination. Much of its apparent locality gain was available with the original owners, and it needs both readiness information and positional ordering. [Proof of the locality bound and ordering controls](2026-11-18-cells-owner-schedules/README.md) · [Measurements for five policies on every archived block](2026-11-18-cells-owner-schedules/wikidata.csv)

**Disposition.** The lower bounds settle the copy-count question. The first-occurrence walk passes the structural screen because its representative choice permits immediate copying in cyclic output order. The remaining uncertainty is the cost of chasing and dirtying scattered recipes. The deliverable stops at the mathematical result, executable span audits and per-block evidence for independent review.
