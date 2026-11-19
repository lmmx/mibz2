# Simultaneous seams can be necessary for the best charged refinement

Let P contain the physical F-byte boundaries. A seam t splits before zero-based row t. Write S(Q) for the sum, over physical source intervals in Q, of the largest count of a (preceding byte, physical destination interval) pair. The objective here is S(P∪T)−S(P)−λ|T|, for λ>0. A candidate T is *neutral* when the refined target leaves every old byte/destination histogram bin unsplit inside each refined source interval.

A unique optimal neutral refinement of a primitive binary column can require an arbitrarily large simultaneous addition. With a fixed charge of one, true periodic columns can require arbitrarily many together as well. The obstruction concerns monotone refinement using only seams in the proposed final partition. It does not bound reconstruction time or establish which refinements win on real blocks.

## A directed graph gives the minimum batch size

For an interior non-F seam t define I(t) = {Ψ(t−1)+1,…,Ψ(t)}, where Ψ is inverse stable LF. The two source rows Ψ(t−1) and Ψ(t) are consecutive occurrences of one byte in source order. The new target seam t splits an old histogram bin on a refined source interval precisely when that source interval contains both rows. Thus t is supported exactly when some physical source seam lies in I(t). For P containing all F boundaries, a proposed T is neutral exactly when each t in T has a supporter in P∪T. For such T the actual modal pair score can be computed from preceding-byte counts when P is exactly the F partition. [General neutral-interval proof](../README.md)

In the directed graph on T, put an edge from each t to every selected supporter. Add a terminal with a loop and an edge from t to the terminal when I(t) meets P. Neutrality says every t can reach the terminal or a directed cycle. The following refinements use only seams in T.

- If each added seam requires support already present, a singleton schedule exists exactly when every t reaches the terminal. For sufficiency add vertices in order of increasing distance from the terminal. For necessity follow the strictly earlier support at each addition.
- If a newly added seam may support itself, a singleton schedule exists exactly when every t reaches the terminal or a self-loop. Add the self-loops first.
- For any nonempty neutral T, the least possible largest batch size is the maximum of one and the shortest directed cycle length in each sink strongly connected component other than the terminal. When a schedule first enters such a sink component, every new member has a supporter in that same batch and component. Consequently the batch contains a directed cycle. Conversely insert a shortest cycle of each sink in its own batch. Every other vertex can reach the terminal or one of those cycles. Add the remaining vertices individually in increasing distance order. The empty set has width zero.

A direct dynamic program enumerated all nested neutral batch updates and recovered this last formula on 37,003 neutral extensions of 7,289 old partitions. Its arbitrary binary and ternary columns include periodic and multiple-cycle cases. [Verifier](batches.py) · [Counts](batches.json)

On a primitive column, allowing temporary seams *and subsequent deletion* changes the accessibility problem. From the F partition, successive complete old-safe rounds reach the partitions by contexts of lengths 2, 3, …, giving all seams by length n because all cyclic rotations are distinct. Each such round can be added one seam at a time because support persists. Starting from a given neutral T, the same full extension works by monotonicity. Reverse that latter order to delete the temporary seams after reaching the full partition. This construction can pay for many extra states and requires deletion.

## Smallest primitive obstructions

The following scores use exact pair histograms and a single physical LF cycle. Sequential includes self-support at addition.

| L | λ | Initial F seams | Unique optimal T | Base → refined mass | Best net, simultaneous | Best net, sequential |
|---|---|---|---|---:|---:|---:|
| 101100000 | 1/3 | 6 | 1,4,7 | 6 → 8 | 1 | 2/3 |
| 2120010000 | 1/2 | 6,8 | 3,7 | 6 → 8 | 1 | 1/2 |
| 122010011000 | 1 | 6,10 | 3,7 | 6 → 9 | 1 | 0 |
| 111000001101000 | 1 | 9 | 3,6,12 | 9 → 13 | 1 | 0 |

The first row's selected supporters form exactly 1→4→7→1. All three seams must be born together. The last row has a two-cycle 6↔12, after which 3 can be added individually. The nine-row example first separates the objectives for charges strictly between 1/4 and 1/2. At either endpoint at least one sequential solution ties. [Direct pair-histogram oracle and every tied optimum](witnesses.json) · [Source](oracle.py)

The length-nine example is globally minimal over ordered effective alphabets for any positive real λ, and the length-twelve ternary example is globally minimal for λ=1. These statements require primitive cyclic input, initial P exactly F, simultaneous histogram neutrality, and a strict advantage over the best monotone singleton schedule even with self-support. At length nine four columns have an open charge interval with a gap. All 147,512,052 length-eleven ordered primitive columns have identical unrestricted-neutral and sequential optima at λ=1. One unit-charge witness at length twelve is sufficient for the upper bound. A separate digit-column enumeration puts the minimum at length fifteen when the alphabet is restricted to binary and λ=1. [All-alphabet census](minimal.csv) · [Binary](binary.csv) · [Ternary](ternary.csv)

To exhaust all ordered alphabets, enumerate every n-cycle Ψ by fixing the first orbit member to zero. Every descent of Ψ in physical order must contain a byte boundary. Freely choosing the remaining ascents on which to start new letters enumerates exactly the compatible first columns F up to order-preserving renaming. Assigning L[Ψ(x)]=F[x] produces that column and recovers the original stable inverse. Primitive cyclic words give precisely single-cycle inverses. To prune safely, remove seams with old support or self-support, then iteratively remove vertices with no remaining outgoing edge. If the remaining graph is empty, a selected neutral subset without a singleton schedule would require a rootless directed cycle within this empty core. For each surviving column enumerate every subset. Let A(q),W(q) be the maximum mass for q chosen seams over all neutral sets and over sequential sets. A strict gap exists exactly if A(q)−λq beats every line W(t)−λt at some λ>0. Intersect the corresponding open rational half-lines by cross multiplication. This covers all positive real charges, including interval endpoints where different maximizing cardinalities can tie. [Enumeration and comparison](minimal.cpp) · [Separate word-order enumeration at five charges](search.cpp)

## A primitive binary family forces a batch of any desired size

Take k≥3, n=3k, L equal to 1011 followed by n−4 zeros, B=n−3 and P={B}. Here Ψ maps 0→1. It maps t→t+3 for 1≤t<B, and maps B,B+1,B+2 to 0,2,3 respectively. Following the residues 1,2,0 modulo 3 in turn gives one cycle, hence a valid primitive BWT. The support intervals are

    I(1)   = {2,3,4}
    I(t)   = {t+3}  for 2 ≤ t < B
    I(B+1) = {1,2}
    I(B+2) = {3}.

Choose T={1,4,…,B+1}. With P∪T as supporters, each t in T sees exactly the next member of a directed k-cycle. Its least possible batch width is k. It produces mass n−1, up from n−3. Take λ=1/k for net gain one.

Every solution with mass at least n−1 must have a seam at 2 or 4. In their absence rows (1,2) and (3,4) give two disjoint opposite-byte pairs each lying within one source interval. Even when all four lie in one interval there are at least two errors. Selecting 2 forces the entire 2-modulo-3 chain through B+2, then the 0-modulo-3 chain from 3 up to the old B, for 2k−2 new seams. Selecting 4 forces k−1 seams along its chain through B+1, which then requires 1 or 2. Thus the unique minimum for mass n−1 is T with k seams. Perfect mass forces cuts at all original L-byte changes 1,2,4. Their forced chains fill all 3k−2 optional seams, for net gain 2/k. Any nonempty solution gaining at most one raw hit has net gain strictly below one as well. This proves unique global optimality over every neutral set. Direct pair histograms enumerate all alternatives for k=3…6. Structural checks include k=10,000. [Verifier](family.py) · [Results](family.json)

## A true periodic family at fixed charge

The stable thick cover suggested by Dual replaces each old physical row by g=k identical copies. Sorting the cyclic rotations of the g-th power of the primitive text makes the preceding column exactly the g-fold copy of each old L entry, with inverse Ψ'(gz+j)=gΨ(z)+j. The old F seam is gB. A nonaligned seam gz+j with 0<j<g has exactly one possible source supporter, gΨ(z)+j. Since old Ψ has one cycle, selecting any such seam forces all 3k seams at that offset. The initial physical error mass is 3g=3k. At unit charge every solution with a nonaligned seam therefore has net gain at most zero. For aligned-only sets the support rule projects exactly to the old one, and all hit masses scale by g. The unique optimal refinement is consequently gT, with net gain k, still requiring one batch of k. For any fixed positive integer charge c, use g=ck and the same proof. This family is the BWT of one periodic cyclic text with g physical LF lanes. The primitive fixed-unit-charge unbounded case remains open here. The fixed-unit primitive 12-row example above needs a batch of two. [Direct rotation and support checks](cover.py) · [Results](cover.json)

## Reproduce

Run from this directory. The C++ programs generate candidate optima and complete census rows. The Python oracles separately compute physical target pair counts, minimum-batch dynamic programs and cyclic power rotations.

```sh
c++ -O3 -std=c++20 minimal.cpp -o /tmp/astra-unlock-minimal
/tmp/astra-unlock-minimal 1 11 | diff - minimal.csv
c++ -O3 -std=c++20 search.cpp -o /tmp/astra-unlock-search
/tmp/astra-unlock-search 2 2 16 | diff - binary.csv
/tmp/astra-unlock-search 3 3 12 | diff - ternary.csv
python3 -B oracle.py | diff - witnesses.json
python3 -B batches.py | diff - batches.json
python3 -B family.py | diff - family.json
python3 -B cover.py | diff - cover.json
```
