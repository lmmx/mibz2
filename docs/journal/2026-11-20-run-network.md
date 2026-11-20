# The run network: a network-science reading of LF

*20 November 2026. The Coordinator, at Louis's request (network science and graph algorithms as a
second language for the inverse). A census and a map, not a decoder.*

## The graph

- **Vertices:** the r maximal runs.
- **Arcs:** an arc k → v, weighted by its width, wherever the LF image of run k overlaps run v.
- **How it is built:** by merging the domain partition with the image partition, in O(r), with no
  row-level work.

**Prior art in mibz2:** this is the graph of Astra investigation 3, which counted 62,138 arcs on
block 0, the same number as here. Its spanning forest and chords drove the "forest and boundary
parentheses" decoder, measured at 15× slower than Cells. The new part is the census and the
reading below.

## The census, six blocks (exact; [`runnet.py`](2026-11-20-run-network/runnet.py))

| Block | r | Arcs | Cycle rank | Runs with in = out = 1 | Core after series reduction | Core arcs | Rows in core runs | Open's thin rows |
|---|---|---|---|---|---|---|---|---|
| 0 | 51,184 | 1.214r | 0.214r | 0.89r | 0.113r | 0.327r | 0.519n | 0.100n |
| 2 | 54,532 | 1.242r | 0.242r | 0.88r | 0.120r | 0.363r | 0.532n | 0.125n |
| 85 | 79,375 | 1.216r | 0.216r | 0.89r | 0.107r | 0.323r | 0.518n | 0.174n |
| 143 | 108,335 | 1.249r | 0.249r | 0.88r | 0.122r | 0.371r | 0.553n | 0.219n |
| 207 | 87,111 | 1.239r | 0.239r | 0.88r | 0.117r | 0.356r | 0.556n | 0.151n |
| 210 | 128,732 | 1.208r | 0.208r | 0.89r | 0.109r | 0.317r | 0.508n | 0.227n |

**Observations (measured):**
1. **The cycle rank is a known identity.** Arcs are the atoms of the common refinement of the
   domain partition and the image partition. So E = 2r − 1 − C, where C is the number of interior
   boundaries the two partitions share. This is MONODROMY's record 63, verified on all 233 blocks:
   "E₀ = 2r − 1 − C, the atoms of P ∨ TP; q₁ = r − C, the forest's chord count".
   - It follows that E − V = r − 1 − C. That is the copy count k = r − |D̄∩C̄| from Neon round 5,
     the chord count of Astra's forest, and the cycle rank minus one. On block 0: 10,954 against
     10,955.
   - The off-by-one is a convention. D̄ counts row 0, and the graph is connected (c = 1), so
     β₁ = E − V + 1.
   - This is a re-identification, not a new finding.
2. **The network is nearly a permutation of runs.** 88–89% of runs are pure pass-throughs
   (in-degree = out-degree = 1). Contracting those series vertices, an exact order-preserving
   operation (composing translations), leaves a core of 11–12% of the runs with 0.32–0.37r arcs.
   - Chains between core vertices average 2.4–2.8 runs.
   - The longest chain is 41 runs on five blocks and 37 on block 2 (an earlier version said 41 on
     all six). The census explains it (below).
3. **It is not scale-free.** Core degrees are tiny: mostly (2,2), (2,1) or (1,2), then (3,1). There
   are no heavy-tailed hubs.
4. **The core runs are the wide ones.** They hold 51–56% of the rows, while Open's retained thin
   rows are 10–23%. In network terms Open keeps chain rows and eliminates the branching wide
   strips. Taking the core runs as the retained set would be worse than Open.

## The reading: a measured train track

**What the arc-level graph omits:** the order in which rows pass through each vertex. LF is
order-preserving inside a run. So at every vertex the incoming rows, in order, become the outgoing
rows, in order.

**What that makes the structure:** a **measured train track** with order-preserving switches. The
LF cycle is the unique circuit compatible with those switches and widths.

**Prior art:** this lands in Thurston's world again. Train tracks carry measured foliations.
Splitting a train track is Rauzy–Veech induction, and Open's elimination is a form of folding.
That is consistent with the origami lift
([2026-11-20-origami-lift.md](2026-11-20-origami-lift.md)): the train track sits on that surface.

## Open questions in network language

1. **Separators of the core.**
   - *The object:* the series-reduced core has about 0.11r vertices and cycle rank 0.21–0.25r.
   - *The question:* does it have small separators or a shallow hierarchical decomposition, so that
     the marked circuit is composed from port maps per piece (an Euler-tour stitch) instead of
     being walked?
   - *How to decide it:* measure a balanced separator hierarchy on the core of the six blocks.
2. **Where the ordering information lives.**
   - *What to measure:* how many order-relevant events the marked circuit makes at switches. Each
     pass through a branching vertex is one, with bundles splitting at out-degree 2.
   - *Why it decides:* if that count is near Open's retained rows, the graph adds no leverage. If
     it is far smaller, there is a collective primitive to find.
3. **The longest chain:** answered by the census below.
4. **Hubs versus chains.** Open keeps the chains and eliminates the wide strips. Is there an exact
   elimination that removes the chains instead and keeps the small core, in a way whose
   acquisition and consumption are cheaper?

## The census results (branch `research/run-network-census` at bd781368)

*Its write-up: `docs/journal/2026-11-20-run-network-census.md`, with `census.py` and `baseline.py`.*

**Q1, separators of the core:** "promising" under its rule on 6 of 6 blocks, but the rule was close
to vacuous.
- **The figures:** METIS recursive bisection down to pieces of at most 64 vertices cuts about half
  the core arcs. That is 0.008–0.022n ports.
- **Why the rule was vacuous:** the whole core has only 0.019–0.045n arcs, so even cutting every arc
  passes 0.05n.
- **No small separators:** the cut stays at about 0.15 arcs per vertex at every depth, so separators
  grow in proportion to piece size.
- **But not random:** the core bisects 4–8× better than a degree-preserving random rewiring of it.

**Q2, where the ordering information lives:** "leverage" under its rule on 6 of 6 blocks, but the
rule was mis-specified, and I wrote it.
- **The measurement:** the first-return pieces to the rows of branching runs. Their count equals the
  series-reduced core's arc count exactly on every block, 0.32–0.37r, because adjacent rows stay
  adjacent through pass-through runs.
- **The error:** the rule compared that *piece* count with Open's retained *rows* m, which is not
  like with like.
- **Like with like:**
  - The section holds every row of the branching runs, 0.51–0.56n. A walk over it would make far
    more dependent steps than Open's m (0.10–0.23n).
  - Its exchange, about 0.33r pieces, is larger than Open's endpoint exchange, which has at most
    3H + 2 arrows over a few thousand thick strips.
- **Verdict:** as a retained set for first returns, the core is worse than Open's thin set on both
  counts.

**The longest chain is structural.** On block 0 it is the opening quote plus the 40 hex digits of a
SHA-1 that occurs twice, giving 41 runs of width 2. Blocks 85, 143, 207 and 210 show the same kind
of hash chain. Block 2's 37 is a GUID (36 characters) plus `$`. These chains are repeated
identifiers in the JSON.

**What survives:**
- the network is highly structured: nearly a permutation of runs, with a core that bisects far
  better than random;
- a series reduction that is exact and preserves order.

None of it is a cheaper section or decomposition for reconstruction. Any leverage would have to
come from an operation on the core that is not a first-return walk.

## Routing

- **Neon:** one lead for the foundational track, via FRONTIER.md.
- **No charter change** follows from this note.
