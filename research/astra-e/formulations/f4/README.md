# F4. Induce the BWT of maximal monotone text phrases

**Object and phase.** Starting from the maximal L runs and a marked row, construct a shorter BWT column of phrase identifiers. This changes the unit of reconstruction. Each token spells an entire maximal strictly increasing phrase in forward text. A second arm uses maximal strictly decreasing phrases. The information needed is present on arbitrary columns, including columns with many LF cycles. The previous Astra A ordered-pair substitution assigned two colours to the alphabet and joined only isolated adjacent pairs. The construction here permits phrases up to the full alphabet size. [Previous substitution](https://github.com/lmmx/mibz2/tree/4ee40ab341b4fdb1681f8a585cde7cee21e076db/docs/journal/astra-a-reinvention)

## Exact identity

Let Ψ be the inverse of the stable LF permutation, with the emitted forward symbol at row r equal to F[r]. The preceding symbol is L[r] = F[LF(r)]. Select the boundary rows S for which L[r] ≥ F[r]. Write one token for the forward phrase beginning at each boundary and ending just before the next one. All following pairs inside the phrase strictly increase. A nonempty cycle always meets S, since it cannot increase strictly around a circle. The backward distance from any primary to its preceding boundary is therefore at most σ−1 for an alphabet of σ symbols.

Order token identifiers by ordinary lexicographic order, with a strict prefix before its extensions. For a surviving suffix row r, let T[rank_S(r)] be the token immediately preceding r. Stable LF of T takes rank_S(r) to the preceding boundary rank.

**Proof.** In the first-column group for a byte, Ψ is strictly increasing because LF was stable. To compare two surviving starts, advance both through their common phrase prefix. Their order is preserved at every common character. If the phrases first differ at a byte, the F buckets of the two resulting rows give their original order. If one phrase ends at the common prefix ending in byte c, the shorter phrase's next byte is ≤ c while the longer phrase's next byte is > c. The shorter one comes first. If both phrases are equal, advancing through them preserves the starting rows' order at their respective *next* boundaries. The preceding-boundary map is a permutation of S, since every cycle meets S. Its targets therefore group in the specified token order and preserve source order within each identifier. This is the defining stable count-sort property of LF. The argument uses row order and LF stability even when equal cyclic rotations have distinct row labels.

For descents select L[r] ≤ F[r]. At a shared prefix ending in c, the shorter phrase's next byte is ≥ c and an extending byte is < c. Order sibling trie edges by the original byte, but publish each terminal *after* its extensions. For the cyclic text 221 the two phrases 21 and 2 must have that order. Ordinary shorter-first lexicographic token naming reverses those target classes.

Let b be the first boundary reached by walking backward from primary p, allowing zero steps, and d that number of steps. A forward walk starting at b returns to p after d bytes. Rank b in the compressed order, decode its marked token cycle, and expand one full token period. Rotate that byte period left by d and repeat or truncate to exactly n bytes. This handles marked periods that do not divide n. One need never insert the original mark as a new phrase boundary. At several hierarchy levels the same operation applies in the alphabet of each level. Truncating a partial marked period and then repeating it at the next level would be invalid in general.

## Direct construction from runs

The constructor counts bytes, obtains F bucket boundaries, and intersects the L runs with them in one row-order sweep. A resulting record contains a constant pair (a,c) = (L,F), its LF translation and the boundary predicate. There are at most r+σ−1 such records, for r original runs. Locate the marked predecessor boundary by at most σ−1 LF queries in this representation.

For a row r in the F bucket of c, compute the identifier P[r] of the phrase prefix ending at F[r]. It is the one-byte identifier c at a boundary. Otherwise it is P[LF(r)] followed by c. In the latter case F[LF(r)]=a<c, so process F buckets in increasing order. For descents process them in decreasing order. Store consecutive equal P values as intervals. Each translated query intersects one L/F record image with a previously produced interval sequence. An intern table keyed by (prefix-node,byte) and a trie give exact word equality. No decoded bytes or row-sized LF array enter this construction.

Finally scan the boundary records in original row order. For each, copy the identifiers P[LF(r)] over its translated interval, coalescing adjacent equal identifiers. This emits T in surviving row order. Traverse the trie to assign ordered identifiers to terminal words that occur in T. Each reduction has distinct costs for interval searches, prefix fragments computed, dictionary probes, trie ordering and residual output writes. The simple program uses binary search for the first fragment of each query. Monotone cursors could reduce search cost in the ascending sweep, leaving the actual intersected fragments to process.

The residual can be inverted by stable counting sort and a permutation walk that emits one phrase at a time. This costs O(m+σ') table construction, one dependent walk per marked residual row, dictionary storage and final byte copying, for m reduced rows and σ' distinct tokens. Applying Cells after the transform instead needs a generalized alphabet wider than eight bits, marked position recovery, token ordering and eventual phrase expansion. The diagnostic computes its binary cut geometry independently. All in-place aliasing, allocator behaviour, worker scaling and hardware throughput remain untested in this mathematical prototype.

## Independent checks

The small checker uses stable sorting of original row numbers, tests the first-return destinations and actual word of every boundary arc, and compares all n marked output symbols. Its cases include both parse orientations and every primary on arbitrary binary, ternary and quaternary columns within the documented bounds. There are 17,610 marked comparisons, 6,132 short marked periods not dividing n, 4,608 swallowed marks and 13,002 audited return arcs. The column count in the output counts both parse orientations. [Checker](small.py) · [Receipt](small.json)

```sh
d=research/astra-e/formulations/f4
python3 "$d/small.py"
data=2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git fetch --no-tags origin "$data"
mkdir -p "$d/target"
git archive "$data" wikidata | tar -x -C "$d/target"
OPENBLAS_NUM_THREADS=1 python3 "$d/real.py" "$d/target/wikidata"
```

## First complete pass over Wikidata

Both orientations matched the archived pre-RLE1 SHA-256 on all ten systematic blocks. Frozen shrinks from 899,931 rows, 51,184 input runs and 92,570 original cuts to the following objects. Astra A's ordered disjoint pairs left 49,310 token runs and 89,528 cuts. Astra B's aligned two-row contraction retained 439,965 rows and had 93,746 cuts on its resulting geometry. Similar support counts need not imply similar binary recurrences.

| Frozen | Selected token rows | Token runs | Stable trie names in token alphabet | Prefix pieces built | Token pieces emitted | Residual cuts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Strict ascents | 466,617 | 31,476 | 3,085 | 52,441 | 32,052 | 62,760 |
| Strict descents | 449,070 | 31,990 | 3,204 | 53,075 | 32,562 | 60,047 |

The decrease in cuts is substantially larger than Astra A's pair substitution on this block. Across the ten large blocks the original recurrence needed 1,236,053 cuts. The pair-substituted geometry needed 1,176,998, the ascent token geometry 823,828 and the descent token geometry 800,785. Charging one unit per processed prefix or token piece would add 1,142,950 pieces for ascents and 1,157,508 for descents. Those units are cheaper than some cuts and still require dictionary naming and output. A dense token residual also leaves 4,630,169 of 8,999,915 rows for ascents and 4,527,400 for descents. Repeating coarsening before that residual is the next separate cost question. [Raw structural counts and hashes](wikidata.csv) · [Historical aligned residuals](../../../../docs/journal/2026-11-18-astra-b-collective/frozen-residual.csv)

Weak ascents would cut only at strict descents. Rows with L=F occur 15,756 times on frozen and on 1.60–1.97% of rows per block across these ten inputs, the entire possible additional reduction in one pass. Equal symbols make the prefix recurrence depend on its own symbol bucket. An all-constant orbit is necessarily a single fixed point: on rows with L=c the map LF strictly preserves order, and a strictly order-preserving permutation on a finite ordered set is the identity. A marked fixed point can be recognized from its original affine interval and emits the constant byte n times. Other marked cycles always contain a strict descent. These observations establish the small opportunity and handle the exceptional marked output; a weak-phrase run compiler has not been implemented.
