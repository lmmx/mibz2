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

## Full recursive solver

The next experiment recursively applies strict-ascent substitution. It pre-expands the current token dictionary into original bytes. When an old mark lies inside a new token, walking it backward to the boundary accumulates the lengths in *original bytes* of the crossed tokens. That sum adds to the previous carried offset. Thus the starting original-byte offset in the new marked token is known at every level and remains inside its spelling. At the residual it walks the marked cycle backward using a packed LF/token table, reverses the visited token list, and expands the dictionary. It rotates at the carried offset and repeats from the resulting complete marked byte period if necessary. Every step, ending at at most eight reductions or a dense endpoint of at most 32,768 rows when reached, is specified for arbitrary marks and multiple cycles. A non-shrinking reduction terminates immediately.

The Python solver checks 8,805 marked inputs with 21,049 reductions forced and verifies each archived output hash. The first oracle had a depth limit in its recursive trie publication. A reviewer provided a valid bzip2 whose second-layer alphabet and monotone token phrase both have size 1,001, causing a Python RecursionError. Publication now uses an explicit stack. The historical failure, repaired result and an additional 2,002-byte witness with five positions (final carried offsets through 2,001 bytes) are preserved. The Rust solver has iterative stacks throughout. [Independent pre-fix witness](../review/deep.json) · [Repaired independent receipt](review-deep-fixed.json) · [Second counterexample](deep.py) · [Marked offsets](deep.json)

The safe Rust implementation uses one cursor per input symbol for each merge pass. During prefix naming, all requests to append the same byte arrive in the same F bucket. The most recent child of a prefix is therefore the exact interned word if its last digit matches. Otherwise it creates a child; this removes hashing and all per-fragment binary searches. The mark lookup still searches an ordered interval array. The direct native checks force the full hierarchy on small columns, on the deep witness, on ten real blocks, and on three unrelated 900,000-row run-heavy columns, with independent stable sorting of row indices. The scorer entry takes the candidate route on width-16 density-eligible, run-fed blocks of at least 65,536 rows, inheriting LazyCells elsewhere.

On frozen, seven complete levels yield 25,977 residual token rows, a 103,908-byte dense table and 25,977 marked dependent links, down from the 89,874 residual edges in width-16 open return. The construction also processes 147,687 L/F intervals, 154,953 prefix pieces and 100,987 token fragments across those layers, creates 58,497 trie nodes and writes 633,500 dictionary bytes before final expansion. Each real input's level ledger and marked digest are in [hierarchy.csv](hierarchy.csv). These are logical structural counts. The native implementation retains arena capacities between blocks and the Python object sizes are substantially different. Neither is an independent peak-RSS measurement.

```sh
python3 "$d/deep.py" "$d/target/deep.column"
python3 "$d/hierarchy.py" small
python3 "$d/hierarchy.py" "$d/target/wikidata"
bash "$d/stage.sh" > trial.csv 2> trial-manifest.txt
```

## Native stage price

Nine balanced passes over the fixed eleven large eligible Wikidata blocks compare the complete F4 inversion of ready MTF records with opend16 and LazyCells. All 33 warm outputs and 297 measured outputs matched the separately prepared LazyCells result. F4 took 62.17 ms for the median eleven-block pass, compared with 19.65 ms for opend16 and 20.41 ms for LazyCells. F4/opend16 was **3.109** wall ratio (95% log-t interval 2.962–3.264) and **3.107** thread CPU ratio (2.960–3.262). F4/LazyCells was **3.046** wall ratio (2.919–3.179). These are stage timings on this container. The complete recursive transform has a material construction penalty on this workload. No target run is warranted by these results. [Raw stage calls](stage-laps.csv) · [Summary](stage-summary.csv) · [Build and input hashes](stage-manifest.txt)

Across ten systematic structural archives the construction needs 2,077,271 prefix pieces, 1,363,357 output pieces, 758,171 trie nodes and 8,183,049 dictionary bytes written before final expansion. This changes different dependencies from the frozen boundary-orbit depth-one count of 10,955 and the 16,230 forest routing bands. Neither count describes the work of producing or lexically naming F4 prefixes. No reduction in a boundary-profile count predicts the price of this new unit of computation. [Prior source-pinned measurements](../../prior.md)

The first native build used the same candidate digest as the immutable source commit. The independent source-property suite can be reproduced with `bash "$d/native.sh" "$d/target/wikidata"`. To regenerate the phase ratio table run `python3 "$d/price.py" "$d/stage-laps.csv"`.
