# 2026-11-18 · Decoding through the gaps between BWT rows

The half-million context refinements in the support inverse can be bypassed. Two adjacent BWT rows inside a run emit the same byte and move to adjacent rows. Follow the **gap** between them until an original run boundary breaks that relation. For a column with $r$ runs, these gaps form a partial permutation with only $r-1$ finite paths. Whole stretches of two executions can therefore be compared at once. The comparison survives repeatedly cutting the LF cycles at their local minima. [Earlier support result](2026-11-18-bispecial-factors.md) · [Towers, words and loops](../site/blog/2026-11-16-mathematical-dictionary.md)

This gives an executable construction **from bzip2's Huffman symbols to a grammar for the marked output**. On the frozen block it reads a 41,618-byte bzip2 stream, constructs the grammar in 12 rounds, emits all 899,931 inverse bytes and passes RLE1 and CRC. On the full Wikidata workload Cells still wins by a wide margin. On a primitive 875,001-byte control with five BWT runs, the direct decoder is 9.5 times as fast as shipped Cells. All 875,001 futures on that control are distinct. [Proof](2026-11-18-adjacent-rank-gaps/proof.md) · [Executable and input reproduction](2026-11-18-adjacent-rank-gaps/README.md)

## What order supplies for free

Give the BWT rows their usual ranks and let $f$ be LF. A sequence of ranks along a cycle alternately rises and falls. Keep the local minima. Between two minima the ranks rise to a maximum and fall to the next minimum. This permits a computation quite different from following each occurrence until it has a unique name.

There is a simple numerical inverse that demonstrates the principle. In an increasing sweep of the ranks, every decreasing LF edge can inherit its endpoint and distance from an earlier row. In a decreasing sweep, an increasing edge can inherit the answer already at its destination. A row now knows the next minimum it reaches and its distance from it. Give the minima output coordinates. A row $x$ that reaches minimum $q$ in $d$ steps starts at output coordinate $\operatorname{pos}(q)+d$ modulo the marked period. Its preceding byte goes one place earlier. The executable uses this identity to write the output without discovering any return spellings. It can also solve for the minima's coordinates by repeating the construction on their weighted permutation. The order halves at every level. List ranking for inverse BWT and sampling local minima in suffix order both have precedents. [Edwards and Vishkin](https://drum.lib.umd.edu/items/7b674b30-00f2-4a11-9509-ace1fc3aa3f7) · [Nong, Zhang and Chan](https://doi.org/10.1109/TC.2010.188)

On the frozen column this numerical version starts from 307,346 minima and ends with one. Its single-round version took 8.90 ms for the complete decode. It still writes an integer pair for every row. An earlier intermediate experiment chose the most common predecessor at each four-byte context and cut the exceptions and remaining cycles. It worked with 37,490 contexts and 111,675 exceptional occurrences. Both constructions made the cost of maintaining individual states visible. The gap construction below charges the work to the input discontinuities. [Numerical sweep](2026-11-18-adjacent-rank-gaps/evidence/frozen-rank-sweeps.csv) · [Context-forest measurements](2026-11-18-adjacent-rank-gaps/evidence/frozen-context-forest.csv)

## Computing excursions as intervals

Rotate the *rank order* to put the primary at zero. This changes at most two cuts in the initial translation intervals. Zero is now a local minimum on every nontrivial marked cycle, including every later induced cycle. When it becomes a fixed point, the label of that one return spells its entire period in forward order. Repeating or truncating this word gives the usual marked answer for an arbitrary column.

For example, `banana!` has last column `annb!aa` and primary 4. In the rotated order its LF cycle from zero is

```text
0 → 3 → 4 → 1 → 5 → 2 → 6 → 0
```

The minima are 0, 1 and 2. Their successive returns carry the forward words `na!`, `na` and `ba`. The next contraction retains zero. Reversing the order of these words in the backward traversal gives `ba` · `na` · `na!`, the marked text. The numerical sweep above uses unrotated order, so its measured number of first minima differs from the sparse decoder's.

At any round an interval $[x,x+w)$ maps by a translation $\delta$. On an upward interval, only its first $\min(w,\delta)$ rows can be entered from outside it. The number of visits before leaving is a ceiling division and takes at most two values across those entrances. The downward case is the reflected calculation. Cut the exits at target-interval boundaries. This gives at most three times as many pieces as there are input intervals. Each piece carries one old word or a power of it.

Start a list of origin intervals wherever a downward piece lands in an upward interval. These are exactly the minima. Join incoming lists in rank order and split at the outgoing widths. Process upward intervals left to right, then downward intervals right to left. A piece reaching the next minimum now supplies a whole interval of origins, its destination and its return word. A balanced tree lets the program split a list by width or prepend a word to the entire list. The runs of the original last column, their widths and the primary are sufficient to start.

## Why the representation stays small

Write $R$ for the initial number of translation intervals after rotating the ranks. At **every** level the program needs at most **$3R-2$ intervals**. This is a bound on the constructed object, including its words. Merely knowing that the number of remaining rows halves would give no such bound.

On a path of original gaps the ranks on the two sides differ by one throughout. All comparisons between two ranks on one side give the same result on the other. Consequently the local minima in the interior of the path are selected together. At each end at most one selected rank can lose its partner at any level. Taking minima again cannot accumulate more unpaired ranks there, because two consecutive candidates cannot both be minima. Each finite gap path can account for at most three surviving interval starts. Gap cycles agree throughout. Rank zero accounts for the remaining possible start.

There is a cheap certificate for the word equality this argument uses. An interval carries one bit saying whether its first row's excursion hit an original interval start. A clear bit means that every step of that excursion can also be taken one rank to the left. If the source ranks on either side of the boundary are adjacent in the original order, the images are consecutive and the original return lengths agree, the two intervals can merge. The certificate proves word equality in that test. The code propagates the bit during the same joins and splits. The proof gives the endpoint cases, the precise adjacency test and an example in which four intervals become five. [Gap-path proof and construction bounds](2026-11-18-adjacent-rank-gaps/proof.md)

Summing the smaller of $3R-2$ and $n/2^j$ over levels $j$ gives $O(R(1+\log(n/R)))$ interval visits. Sorting and balanced list operations give a deterministic construction time of

$$O\!\left(R(1+\log(n/R))\log(2R)\right)$$

in machine-word operations from the maximal run list. The live interval data take $O(R)$ words, with the grammar stored in addition. The direct frontend takes one pass over the Huffman symbols, treats each binary RUNA/RUNB count as a width and uses the block's MTF alphabet. The ordinary writer then pays for the actual output bytes. The companion also plugs the construction into mibz2's existing capped run-record interface and worker pool. The widths, bytes and primary supply the complete mathematical input to both constructors. [Direct frontend and sparse kernel](2026-11-18-adjacent-rank-gaps/extrema.rs)

## Where the machine benefits

Take $T_m=(\mathtt{abcd})^m\mathtt{!}$ with the origin at its first `a`. The unique `!` makes the text primitive. Sorting its rotations gives $L=\mathtt d^m\mathtt !\mathtt a^m\mathtt b^m\mathtt c^m$. There are exactly five runs and LF is subtraction by $m$ modulo $4m+1$. Its minima are ranks $0$ through $m-1$ in the rotated order. On this section LF induces addition by one modulo $m$. The second round retains only rank zero for $m\ge2$. Thus the construction reaches the complete marked word in two rounds for arbitrarily many distinct rotations.

At five tested sizes from 129 to 875,001 bytes it used **nine interval visits and ten grammar rules** each time. The 875,001-byte input was a 56-byte bzip2 stream. The table shows how the result changes after reproducible substitutions in that same text. The marker stays unique. All times are single-core medians of complete bzip2 decodes in milliseconds.

| 875,001-byte input | BWT runs | Rules built | Direct sparse decoder | Shipped Cells | Forced Cells |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original | 5 | 10 | 0.161 | 1.526 | 5.542 |
| 16 substitutions | 49 | 260 | 0.206 | 1.832 | 4.076 |
| 64 substitutions | 174 | 1,163 | 0.343 | 1.990 | 4.038 |
| 256 substitutions | 580 | 5,220 | 1.045 | 2.436 | 5.069 |
| 1,024 substitutions | 2,114 | 20,432 | 2.911 | 2.358 | 4.197 |
| 16,384 substitutions | 32,711 | 199,626 | 42.881 | 4.973 | 8.155 |

Production's existing sparse-record gate sends the five-run case to Segmented. Forced Cells is included to price induction with that admission restriction lifted. The crossing between 256 and 1,024 edits belongs to this family and machine. [Construction counts and input hashes](2026-11-18-adjacent-rank-gaps/evidence/family.csv) · [Every timing observation](2026-11-18-adjacent-rank-gaps/evidence/timing.csv)

## What the frozen and full workloads cost

The frozen block starts with 51,184 maximal BWT runs and 51,185 intervals after rotation. In the first round 298,922 minima give 27,327 finished intervals and 23,935 intervals after certified merges. Across 12 rounds the constructor visited **102,233 intervals**, processed 128,800 ports and built 103,131 rules, of which 99,368 were reachable from the mark. It processed zero factor contexts. The earlier support route processed 504,258 splitting contexts and ended with 899,931 terminal futures. These are different units of work. In a separate paired full-decode measurement the published support encoder and decoder took 1.048 s against 37.4 ms for the run-record contraction. The support encoder includes its published checks and weighted bookkeeping. This prices those two implementations. [Frozen rounds](2026-11-18-adjacent-rank-gaps/evidence/frozen-rounds.csv) · [Paired support comparison](2026-11-18-adjacent-rank-gaps/evidence/support-comparison.csv)

| Complete decode, median | Shipped Cells | Run-record contraction | Direct maximal runs |
| --- | ---: | ---: | ---: |
| Frozen block, one calling thread | 2.55 ms | 27.56 ms | 28.50 ms |
| Wikidata, one calling thread | 0.729 s | 8.370 s | 8.721 s |
| Wikidata, two workers | 0.350 s | 4.298 s | |
| Wikidata, four workers | 0.345 s | 2.682 s | |
| Prose, one calling thread | 0.057 s | 0.363 s | 1.607 s |

The run-record arm uses the ordinary Segmented column route when mibz2 declines run records. The direct arm processes every block through maximal runs. The Wikidata census forced the new construction on all 431 blocks and compared every pre-RLE byte with Plain. It used 29,092,269 interval visits. Among the 233 large blocks the median was 1.93 visits per initial interval, across 10 to 13 rounds. Independent bzip2 decompression matched all 209,700,196 final Wikidata bytes and all 4,947,388 prose bytes. [Wikidata per block](2026-11-18-adjacent-rank-gaps/evidence/wikidata.csv) · [Prose per block](2026-11-18-adjacent-rank-gaps/evidence/prose.csv) · [Checks](2026-11-18-adjacent-rank-gaps/evidence/checks.txt)

The frozen single-block run-record contraction peaked at 12.51 MB of additional requested heap against 10.35 MB for Cells. Cachegrind counted 73.34 million data references against 6.38 million. On the five-run primitive control the same arm used 3.56 MB against 9.12 MB and the direct decoder issued 2.18 million simulated data references against 4.85 million for Cells. These figures include the surrounding decoder. The VM exposed one thread per core and a four-CPU quota. The project's ten-core i9 measurements concern real sibling threads and different cache sharing. [Heap ledger](2026-11-18-adjacent-rank-gaps/evidence/heap.csv) · [Cache simulation](2026-11-18-adjacent-rank-gaps/evidence/cachegrind.csv) · [i9 worker calibration](2026-11-17-worker-count-calibration.md)

The result separates two claims about computation. The program can derive the marked word with work controlled by original BWT run boundaries, however deep the factor language needed to distinguish individual futures. That claim has a construction and a bound. Making this way of doing the work cheaper than Cells on the project's principal workload remains open. Here the cost of routing intervals and building their words outweighs the distinctions it eliminates.
