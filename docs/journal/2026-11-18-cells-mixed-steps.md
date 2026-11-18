# Two Cells cuts can make one return word

18 November 2026. A focused study following Astra Investigations 1–3.

**Result.** A top win followed by equality or a bottom win can sometimes be read as two cuts of one interval against the next two bottom intervals. A single substitution gives the final boundaries. When the bottom intervals carry the same literal, both new bands have the same return word. The shipped Cells constructor makes it twice. One ordinary short-word descriptor suffices for both bands. The input stream and live fronts certify this before the first boundary write.

On the frozen block this removes **2,159 of 92,570 word constructions**. Across all 233 large Wikidata blocks it removes **650,507 of 27,439,756** (2.371%). The reduction survives the cost of recognizing successes and failures in the structural model below. There is also a geometric obstruction. A surviving original top-order site separates the two bands on *every* such hit. Their word can be shared even though the bands cannot be glued into a single translation interval. [Block counts](2026-11-18-cells-mixed-steps/blocks.csv) · [Executables and reproduction](2026-11-18-cells-mixed-steps/README.md)

## What the shipped trace suggested

Record 61's roughly 1.8 events per run compares three different counters. The independent review's forest tie witness changes 12 arcs to 15 on a fixed input. Controlling for both run density and normalized excess reduces the Cells–forest correlation from 0.906 to 0.393. Record 62 found increasing forest work at nine tested context depths from 1 to 32 on the frozen block. The review gives a primitive counterexample to general monotonicity. These observations warrant looking inside an event. They give no lower bound on how many boundary writes or word products the marked output requires. The separately verified pruning of Open-Return removes empty transport paths on an auxiliary section. It supplies no identification of a pair of shipped Cells steps here. [Record 61](https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-synthesis.md) · [Record 62](https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-o3-o5.md) · [Independent review](https://github.com/lmmx/mibz2/blob/bc549b599424bafcb8e1aa6e8c77a6b26efd9683/state/reports/synthesis-6982863130/report.md) · [Pruning review](https://github.com/lmmx/mibz2/blob/8b15964c8b7eede00a16cb80448a53ed1cfddecb/state/reports/recovered-pro-6982461166/report.md)

Write E for equal remaining widths, T for a top win and B for a bottom win. Adjacent pairs on frozen large-block ordinal 0 (stream index 1, 899,931 rows, 51,184 runs) had these exact counts. A closure breaks adjacency.

| First step | Next E | Next T | Next B |
|---|---:|---:|---:|
| E | 42,734 | 5,185 | 3,263 |
| T | 5,205 | 19,433 | 323 |
| B | 3,244 | 342 | 12,840 |

Direct changes of winner occur only 665 times. A strict win followed by equality occurs 8,449 times. One tempting composition is to inline the first new cell into the second. Across **all** mixed pairs the second cell actually uses the first only eight times, and that child has a single use only twice. The productive pattern is two **sibling products**. A top winner keeps its word while the bottom front moves among distinct intervals in stable byte order. This allows the same ordered product to recur across different kinds of cuts. [Trace census](2026-11-18-cells-mixed-steps/census.py)

## The boundary and word identities

Let $A$ be the last top interval, $B$ the last bottom interval and $D$ its immediate predecessor on the bottom. Assume $A,B,D$ are distinct. Their remaining widths are $a,b,d$, their words $u,v,w$, respectively. Suppose

$$
b<a\le b+d,\qquad s=a-b.
$$

**Two-cut identity (proved).** Give the new cells distinct names $X,Y$. Their widths are $b,s$ and their words are $uv,uw$. Apply the following substitutions at the *old* positions. Empty brackets delete a position. Entries not listed retain their order and data.

| Old position | Final top order | Final bottom order |
|---|---|---|
| $A$ | $[\ ]$ | $[Y,X]$ |
| $B$ | $[X]$ | $[\ ]$ |
| $D$, when $s=d$ | $[Y]$ | $[\ ]$ |
| $D$, when $s<d$ | $[D,Y]$ | $[D]$ |

The surviving $D$ has width $d-s$. For the proof, the first cut consumes $B$, replacing it on top by $X$ and inserting $X$ immediately after $A$ on the bottom. The residual $A$ stays last on top. Since $D$ was the predecessor of $B$ and $D\ne A$, it is now last on the bottom. The second cut consumes the rest of $A$, inserting $Y$ immediately before $X$ on the bottom and after the residual of $D$ on top. Equality exhausts $D$ too. The first cut is strict, so there is no intervening closure. This proves the simultaneous substitution, including adjacent old sites and both possible second steps. The formula retains *both* width decisions. Computing $s$ is still necessary.

**Word identity (proved).** In the free monoid on bytes with the distinguished occurrence marked, left cancellation gives $uv=uw$ exactly when $v=w$. A fresh bottom operand is a literal. The packed descriptor of a plain literal is canonical per byte, and contains the mark flag when marked. Comparing two already available descriptors therefore certifies equality here. No word comparison, hash, table of phrases or preliminary pass is required. For a shared unmarked product of at most 16 bytes, both cells can receive the very same arena descriptor. In the frozen trace, at step 1 the widths $(a,b,d)$ are $(21,18,3)$. At step 84 they are $(993,835,880)$. Each pair makes the two bytes `"}` twice. The first pair ends in E, the second in B.

**Why the intervals stay separate (proved).** Give every top position its original site number. When Cells inserts a new cell after the bottom front's position on top, give the cell that site's number. Site numbers remain nondecreasing. An exhausted bottom front leaves a new cell in its site. Only removal of the top tail, including a closure, can empty a site, and it can empty only the rightmost one. Thus the nonempty site numbers always form a prefix of the initial top sites.

On a shared hit $B,D$ are distinct unmarked original letters. No composite can be a one-byte literal. Surviving original letters keep their relative order. Equal bytes are sorted stably on the bottom, so $D$'s site precedes $B$'s on top. There is an initial site strictly between them. Distinct maximal runs of the same byte have an intervening run, and two unmarked pieces of the same split run have the marked singleton between them. After the joint cut $X$ occupies $B$'s site and $Y$ occupies $D$'s. The prefix invariant preserves an intervening site. The bottom order has $Y,X$ adjacent. The top order has them separated. A single translation interval cannot represent them. The independent two-order replay found all 650,507 shared pairs separated. [Order checks and primitive witnesses](2026-11-18-cells-mixed-steps/geometry.py)

## Recognizing the pair before changing the exchange

The implementation question is whether $D$ is available in time. In the lazy kernel a strict top win *must* read the next bottom front. Its source is either a live slot, a tagged stream entry pointing to a slot, or an untouched entry in the stable bottom stream. Move the **source dispatch and stream-header read** ahead of the boundary writes. If it names a slot, defer reading that slot's payload until after the ordinary T update. The update can write it. An untouched entry supplies the original width, byte and mark immediately. The current fronts have already taken their slots, and the first T neither takes nor returns a slot. The untouched entry is therefore distinct from both current fronts, and settling it commutes with this T. This is the operational certificate for the predecessor in the theorem. [Shipped lazy loop and stream reads](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/src/cells_lazy_induce.rs)

On an untouched entry compare $s$ with $d$ immediately. For $s>d$, install ordinary T and enter the next T with its winner already known. On $s\le d$, install the table above directly. The two cells get their normal recipe descriptors on a mismatch or a long product. The pairs do not overlap because the second step of a joint update is E or B. A long string of T's can still end in a recognized TE. Each width decision, aged slot read, header, tag and cursor advance belongs to one ordinary front visit. No completed T is needed as a probe. A finite cell budget still costs one debit per logical cell. If the second debit fails, the ordinary one-cut refusal path suffices. The executable models completed inductions. [Executable slot transducer](2026-11-18-cells-mixed-steps/verify.py)

One link operation disappears even on a word mismatch. A newly settled bottom letter would initialize the previous link of its top slot to zero. In TB, the second scalar cut reads that zero into the old top front's slot and immediately overwrites the source with the old front's slot number. The joint update writes the two final fields, using zero directly. This saves one slot initialization and one slot read per fresh TB. The two fields of each completed cell still have to identify its own position and word. The scalar loop already keeps the residual top width in a register.

## The price of recognizing, constructing and restoring

Let $F$ count T's with an untouched next bottom entry, $M$ the mixed pairs among them, $H$ the equal short products shared, and $G$ the fresh TB pairs. All four are counted on complete traces. The cost comparison uses the shipped scalar lazy constructor with its two `copy16` calls for each short word. It applies to either rule-width arm.

| Part | Difference for the entire trace |
|---|---|
| Preparation and recognition | Zero extra stream passes, headers, incoming slot loads, pool takes/gives or width decisions. Reuse the next-front dispatch, and carry its result on TT. Test the two word handles at most $M$ times and the short-product flag at most $M$ times. |
| Interval boundaries | Apply $M$ direct substitutions. Eliminate $G$ previous-link reads and $G$ initializations. Charge the usual final fields individually, including both outgoing word references. |
| Word construction | $H$ fewer packed-length sums, short/rule classifications and arena-cursor advances. Exactly $2H$ fewer `copy16` calls, each one 16-byte read and one 16-byte write in the source. The logical arena advance falls by the sum of the $H$ product lengths. |
| Marked output | Zero additional rule visits, copies, ranks, offsets to restore, or searches. The short arena is immutable over each advertised length. Long rule numbers, child lengths, mark bits, first output positions and byte contents persist. The writer copies each short occurrence from its descriptor. Reading padding during later short construction does not change any advertised byte. |

**A priced comparison.** In a register/field operation model, charge one for a memory read or write up to 16 bytes, an arithmetic operation, or a predicate with its conditional successor. Renaming a live register on a control edge is free. To make bookkeeping explicit, also allow **one extra dispatch per untouched lookahead** ($F$), **both predicates per candidate**, and **one descriptor selection/move per candidate** ($3M$ including the predicates). The ordinary width branch is reused on every path. Fresh, tagged and linked reads have separate continuations, so a slot fallback uses its original dispatch. The extra live information is one next entry/front and a bounded number of registers, independent of the number of cells. No slot records or tags are added. The central routines and the direct field assignments are transcribed in the executable. Python object construction and trace assertions are verification machinery.

For a lower bound in this model, credit only the four 16-byte memory operations, the length sum, classification and arena advance per shared word. Ignore the saved slot operations entirely. The net saving is then at least $7H-F-3M$. This is **2,270** on frozen and positive on each of the 233 blocks (minimum **1,505**, total **721,413**). Equivalently, the operation vector above allows different prices for predicates, memory and control. The model gives no bound on compiled instructions, cache-line traffic or elapsed time. Arena bytes here mean bytes actually constructed, independent of the budget-sized reserved allocation. [Shipped short-word constructor](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/src/cells_lazy_induce.rs) · [Ordinary writer](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/src/cells.rs) · [12-byte rule writer](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/src/cells_w12.rs)

## What survived the corpus test

| Population | Logical Cells steps | Untouched lookaheads $F$ | Mixed candidates $M$ | Shared short words $H$ | Logical arena bytes saved | Fresh TB $G$ |
|---|---:|---:|---:|---:|---:|---:|
| Frozen Wikidata | 92,570 | 6,354 | 2,163 | 2,159 | 5,385 | 179 |
| Ten published Wikidata traces | 1,236,053 | 84,509 | 29,544 | 29,427 | 73,748 | 2,692 |
| All 233 large Wikidata blocks | 27,439,756 | 1,872,191 | 653,315 | 650,507 | 1,649,050 | 60,126 |
| 228 admitted by the study revision | 26,292,727 | 1,799,275 | 625,298 | 622,523 | 1,580,922 | 57,551 |
| Six published prose controls, forced | 3,369,224 | 445,858 | 166,500 | 166,432 | 429,067 | 30,612 |

Frozen shares 1,980 TE products and 179 TB products. Two further TE products are equal but have lengths 21 and 35. The two equality failures have bottom operands `JP` and `J`, or `778` and `7`. The frozen constructor therefore avoids **4,318 constructor copies of 16 bytes**, equivalent to 4,318 reads and 4,318 writes in the constructor. Across Wikidata, 590,785 shared products come from TE and 59,722 from TB. The per-block share of all word constructions ranges from 1.945% to 2.756%, with median 2.364%. The 842 mismatches and 1,966 equal long products are charged. The ten trace arrays published on the data branch match regeneration field for field. One admission flag changed between source revisions. [Per-block census](2026-11-18-cells-mixed-steps/blocks.csv) · [Group reduction](2026-11-18-cells-mixed-steps/summary.py)

**Checks.** The joint machine chooses its transitions from the two initial orders, widths, labels, marked site and its own slot state. The independent trace is an assertion sink. On every large Wikidata block and the six prose controls it matched the shipped front identities, both widths, read modes, closure sequence, final marked position and exact cycle word. It also matched ordinary slot/header visit counts and pool usage. The real-block check starts with the existing compressed-input recorder. The frozen cycle has mark offset 289,568. Rotating one past it gives the independently retained pre-RLE1 bytes with SHA-256 `f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6`. The existing recorder checks its own marked output against the kernel. An additional 14,209 small marked columns agreed with direct LF inversion of the marked cycle, including inputs with several cycles. Direct order substitution agreed with two scalar cuts in 10,188 local permutation/width cases. [Reproduction and witnesses](2026-11-18-cells-mixed-steps/README.md)

**Stopping point.** Two width decisions can give one boundary substitution and one ordinary short-word product. The asymmetry between stable byte order and the surviving top sites explains both the saving and its limit. It accounts for a measurable fraction of constructor work while leaving the large majority of the sequential induction intact. The effect on decoder throughput or concurrency remains open.
