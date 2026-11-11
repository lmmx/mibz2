# Beyond elementary Rauzy–Veech cells: an inverse-BWT research investigation

Date: 11 November 2026. Scope: mathematical derivations, primary-source review,
repository evidence, executable correctness probes and structural censuses.
**No new decoder timing, real-Wikidata experiment, production implementation,
or several-fold speedup is claimed. No repository changes were made.**

## 1. Main finding

The strongest target is **constructing a compact phrase/copy program directly
from the run-level LF map**, rather than finding a better order for the same
binary cell operations. This remains an extension of first-return induction,
not an abandonment of it. A compact representation is useful only when its
construction, routing, expansion and final output are all affordable.

The repository's real-Wikidata comparison is particularly important:

| Representation / process | Objects per BWT run |
|---|---:|
| Existing right-end induction | 1.812 cells |
| Best counted alternative cell schedule | 1.279 cells |
| Cell-step floor in the specified elementary model | 1.220 |
| Distinct right-end grammar rules | 0.852 |
| Batch RePair grammar, including final sequence | 0.530 |
| LZ77 phrases, with a following literal | 0.295 |

These are different kinds of objects, not interchangeable units of runtime.
The last two were computed from the text; they are **not cheap constructions
from its BWT**. Nevertheless they establish that the present cell count is not
an information-theoretic limit on real data. At r/n = 0.0722, the last row is
about 0.0213 phrases per byte, or 47 bytes per phrase on average. The distribution
of phrase lengths, rather than that mean alone, determines copying efficiency.

A repeated Thue–Morse substitution can be described by A_(k+1)=A_k B_k and
B_(k+1)=B_k A_k: two grammar rules per level. The relevant benefit is reusable
word construction, not a low one-symbol entropy or a long repeated Rauzy winner.
There is no reason to expect those particular substitution rules on Wikidata.

## 2. What the current evidence does and does not establish

PR #365's recorded qualification at 57f51bbc0ca62ac3ee1cc148a1010842eea6222a
reports experimental sequential Wikidata time 1.000 [0.990,1.011] of baseline,
and RSS 26 to 34 MB. The PR metadata now names a newer head, so these figures
must not be attributed to that head without another check.

PR #367's diagnostic head a08277e1c36e276cfea7b3116d99ea5fd93f0e84 finds
196 admitted blocks and 84.117% of rows in the exact 431-block corpus.
Admission dilution is not the leading explanation for the missing gain.
The old `phases` instrument does not measure the real cells route. Different
hardware, port code, writer, refusal, allocation and reader context remain
unresolved explanations, not demonstrated causes.

Already tested, and not proposed here as new discoveries:

* Independent segment rounds rebuilt as arrays: 385–550 instructions per run;
  order and induction alone 1.99 times the old column-based inversion.
* A nonbranching forest/tower: 291 instructions per run against 186 for cells;
  no established advantage over cells.
* An exact-chain prepass whose own passes outweighed the induction work saved.
* A phrase-copy walk exact on all 431 blocks, but not faster in lanes; about
  150 instructions and two mispredictions per copy event.
* Compact links and k-step tables with excessive construction or access costs.
* Speculative block finding with exact chain validation: built, and not faster
  on the measured level-9 Wikidata workload. It may become necessary if workers
  become much faster; that is a different claim.

The mathematical ingredients of the prototypes below are classical induction,
interval refinement, word composition and pair compression. They are concrete
investigations here, not claims of an unprecedented general mathematical idea.

## 3. Exact construction A: simultaneous first-return pair contraction

### State and invariant

Represent a permutation f of m active rows by pieces (s,t,length,w):

    f(s+j) = t+j, 0 <= j < length.

The grammar word w records, in forward-LF order, all original emissions along
that induced edge. The source intervals and image intervals each tile the
active domain. One marked active row is retained for the requested origin.

Color each current word ID 0 or 1. Select an edge x -> f(x) only when its source
word has color 0 and its destination word has color 1. Do not select an edge
whose destination is the marked row. Let D be the selected destinations.

D and f(D) are disjoint: a removed vertex has color 1 and cannot simultaneously
be the color-0 predecessor of another removed vertex. Therefore the first-return
map on K = domain minus D is exactly

    R(x) = f(f(x)) if f(x) is in D, otherwise f(x), for x in K.

For a contracted edge replace w(x) by concat(w(x),w(f(x))). Order-compress the
remaining coordinates using q(x)=number of retained rows below x; the new map
is q R q^-1. This preserves every surviving cycle and its expanded word.
Protecting the marked row ensures that its cycle never disappears.

### Implementation and cost

`batch_induction.py` overlays image intervals and source intervals, marks selected
subintervals, subtracts D, translates retained coordinates, and coalesces adjacent
compatible pieces. It stores a word DAG and intervals, not an n-entry table in
the contraction itself. However, it performs sorting and binary searches each
round: this is NOT an O(r)-time implementation. If a round has E overlaps and
comparably many resulting fragments, its bookkeeping is O(E log E), apart from
hash-consing. More precisely, output fragments and removed-interval searches
must also be charged. Peak interval storage and accumulated grammar nodes are
reported or derivable from the results.

The fixture loader scans L to obtain runs. A run-fed production integration
could obtain these from the existing front, but that integration is not built.
The n-entry inverse permutation is used ONLY by the independent test oracle.
The expansion/checking path is intentionally slow and is not a production writer.

The implementation performs pair compression, NOT the repeated-equal-letter
block-compression phase of full recompression. It therefore has no universal
constant-factor shrinkage guarantee; the near-periodic fixture exposes this.

### Correctness tests actually run

* All binary columns of lengths 1 through 9, every original row.
* 1,000 deterministic random arbitrary columns, lengths below 120, alphabets
  of up to 14 symbols.
* Four rounds per case, checked after each round against independent inverse-LF.
* Total: 9,194 cases, all passed.
* Twelve rounds on each of seven 32,768-byte fixtures, with output checked after
  every round. All passed.

### Census after twelve rounds

| Input | Right-end reference cells | Residual active rows | Total overlap visits |
|---|---:|---:|---:|
| Thue–Morse | 211 | 856 | 586 |
| Fibonacci prefix | 126 | 2,954 | 322 |
| Nearly periodic | 16,385 | 16,380 | 51 |
| Synthetic JSON | 5,918 | 1,121 | 25,673 |
| Uniform four-letter data | 32,070 | 1,367 | 134,536 |
| Uniform bytes | 32,767 | 1,034 | 127,200 |
| GPL-3 prose | 24,573 | 1,184 | 99,948 |

The right-end reference is an unmarked mathematical induction count, not the
production kernel's marked admission count. An overlap visit and a Rauzy cell
are different operations. The data refute the simple argument that shortening
the final walk necessarily reduces construction work: JSON's residual is about
29 times shorter, but construction visits 4.34 times as many overlaps as the
reference has cells, before sorts and rank bookkeeping. This is not a runtime
comparison or a proof that every compiled batch implementation must lose.

### Useful next variant

Keep the contraction invariant, but update only affected boundaries and share
word-level descriptions before constructing intermediate fragments. Measure
all touched intervals, emitted metadata, and residual work. Another full-array
round implementation without a lower total cost is not the desired advance.

## 4. Exact construction B: phrase-level induction gives a token BWT

### Construction

Pick a delimiter byte d. Let A consist of all rows with L[i]=d, a union of whole
BWT runs. For a valid cyclic BWT these are text positions immediately after a
delimiter. Induce LF on A, recording every return word. Reverse the return words
to put them in forward text order.

Each return word is a phrase ending in d with no internal d. These phrases are
prefix-free: no one can be a proper prefix of another, because its terminal d
would then be internal to the other phrase. Lexicographic comparison of text
rotations at these boundaries therefore agrees with comparison of token
rotations, when tokens are ordered by their phrase strings.

It follows that the order-compressed induced map is the LF permutation of the
phrase-token BWT, with phrase IDs as emissions. Thus the smaller map can be
inverted or induced again, and its tokens expanded to bytes. This is a specific
realization of the class-induced exchange already flagged in the repository.

The correct origin is retained by following the original row to a boundary and
recording the number of original emissions traversed. If the chosen cycle never
encounters a delimiter, the candidate declines and must use a fallback. This
initial search is NOT free; in the worst case it walks the whole selected cycle.

### Executed test

`phrase_induction.py` advances interval pieces directly through run-level LF,
splitting at run boundaries, until they return to delimiter rows. It checks
both full reconstruction and the token-LF counting-sort identity, using
lexicographically sorted expanded dictionary entries as an independent check.

Exhaustive binary columns of length 1 through 8, every origin, both delimiters:
6,168 successful constructions and 1,004 proper no-anchor fallbacks. Every
successful construction passed both checks. Periodic and arbitrary multi-cycle
columns are included, but these are NOT full compressed-stream/error tests.

### Synthetic JSON: 32,768 bytes, 2,218 BWT runs, 5,918 reference cells

| Delimiter | Piece advances to construct returns | Token rows | Distinct phrase bytes |
|---|---:|---:|---:|
| Newline | 31,043 | 281 | 32,768 |
| Colon | 5,776 | 2,535 | 7,334 |
| Quote | 4,587 | 7,887 | 2,661 |

The quote case has 425 distinct phrases and 928 return-map pieces. These are
not 425 metadata operations: routing and intermediate word construction still
cost much more. Newline grouping gives very few tokens but effectively no
construction saving, because records with changing IDs remain distinct.
On GPL-3 prose, space induction requires 25,071 piece advances versus 24,573
reference cells, before the residual 5,414-token traversal.

A suitable implementation would charge E (piece advances), H (word DAG nodes),
Q (return-map pieces), the live frontier, residual decoding, and actual output.
The present Python code additionally expands and sorts dictionary strings for
verification; no fast runtime or small peak-RSS claim is made.

### What this leaves open

Adaptive local boundaries and reusable phrase/subphrase descriptions may be
better than fixed delimiters. The objective is to build a compact token/copy
program without paying independently for each occurrence's history. This must
preserve occurrence routing: equal words at different places do NOT imply
identical future dynamics.

Important bzip2 detail: inverse BWT emits the RLE1-coded stream, not necessarily
raw JSON. A count byte may equal a delimiter. That does not invalidate the
byte-level construction, but real-corpus statistics must be taken at the actual
inverse-BWT seam. The synthetic fixtures here are not complete bzip2 blocks.

## 5. A precise caveat about cyclic copy graphs

A graph cycle between whole phrases is not necessarily a cycle between byte
dependencies. Take T[0]='a' and T[1:n]=T[0:n-1]. The copy phrase overlaps itself,
so a whole-phrase graph has a self-edge. Yet every byte dependency decreases
its index and reaches the known seed. The string can be expanded by doubling
initialized ranges: n=32,768 takes 15 nonoverlapping bulk copies and 32,767
copied bytes, not 32,767 dependent byte loads.

`copy_dependency_example.py` checks 7,696 seed/length combinations. All pass.
This is a counterexample to a too-strong general impossibility inference, NOT
a refutation of the repository's measured copy-walk failure or evidence that
Wikidata's large copy-graph SCC admits an efficient solution.

A genuine candidate would derive certified affine copy ranges from LF or an
induced grammar, propagate initialized subranges, accelerate overlap periodicity,
and charge every split and every copied byte. Source discovery and output-position
recovery must be included. If fragmentation approaches a per-byte description,
the proposed advantage is gone. Bidirectional macro schemes require valid
byte-level dependencies; arbitrary cyclic equations do not define an output.

## 6. Quantitative targets, not speedup forecasts

The journal's historical, same-run table gives production inversion
2.049 ns/BWT-row, cells order 0.367, induction 0.937, writer 0.347, whole 1.677.
Using 0.347 as a **representative unchanged writer cost**, not a physical floor:

| Desired inversion speedup vs that production arm | Total budget | Construction budget after writer |
|---|---:|---:|
| 2x | 1.0245 ns/row | 0.6775 ns/row |
| 3x | 0.6830 ns/row | 0.3360 ns/row |
| 5x | 0.4098 ns/row | 0.0628 ns/row |

The existing order stage alone exceeds the illustrative 3x construction budget.
This is why tuning the induction schedule is insufficient. A lower-cost writer
would change the budgets. Recalibrate all terms on the same current machine,
implementation and inclusive call path before using them as acceptance gates.

For an illustrative serial decoder with 70% of time in inversion,
whole speedup = 1 / (0.30 + 0.70 / inversion_speedup): 2x inversion gives 1.54x
whole, 3x gives 1.88x, 5x gives 2.27x, and free inversion gives 3.33x. The 70%
comes from an older column-path stage split; it is NOT a measurement of the
current port and does not directly model overlapped multicore execution.
A 2x whole-decoder goal needs 3.5x inversion under that illustrative split.
A 5x whole-decoder result must remove other work too.

Distinguish n BWT rows from N final bytes after RLE1: all-byte APIs must write N,
which can exceed n substantially. A compressed-domain consumer changes the
interface and output requirement, not this lower bound.

## 7. Ranked investigations

1. **Word-level/phrase-level first-return reconstruction.** Primary Rauzy-derived
   route. Aim for a compact reusable phrase/copy program, not the particular
   elementary cell grammar. Strongest real-data evidence is the measured size
   gap to LZ77/RePair. Main unknown: cheap construction from RLBWT. First test:
   expand an offline oracle phrase program on the real blocks to price the
   writer alone, then charge the actual RLBWT-to-program constructor separately.
   An offline parse is an upper-bound probe, never a decoder speed result.

2. **Boundary-local batch induction/recompression.** Use the exact contraction
   invariant, but eliminate global rescans, intermediate slots and duplicate
   word descriptions. This can support direction 1 rather than being an
   independent additive gain. First test: total changed-boundary work, grammar
   nodes, routing records, bytes moved and peak live metadata on real blocks;
   reject the simple global-round variant when those exceed its budget.

3. **Interval-level copy-dependency elimination.** Alternative representation
   compatible with induced words. Reopen only the strong claim that a coarse
   SCC rules out partial range expansion; do not repeat the failed row-walk
   discovery scheme. First test: a range solver on the already counted real
   copy relations; measure fragment growth and duplicate copies, with full
   source-discovery cost included before any decoder performance claim.

4. **Grammar-native output and consumer.** Highest ceiling may be outside the
   byte-returning API: compose finite-state lexical summaries, decoded lengths,
   CRC effects and selected-field extraction over the word DAG. For a finite
   transducer, F_(uv)=F_v composed with F_u under function application order.
   Summaries cost state-count times grammar size, not necessarily O(1) in any
   useful hardware sense. General JSON nesting is not a finite-state problem;
   materializing every byte still incurs output cost. First test: a fixed
   real consumer/query and exact parity, not a vague "parse compressed JSON"
   claim. Keep a byte-compatible baseline and separate application speedups.

A fifth, lower-confidence route is to discover reusable morphism templates
(modulo byte relabeling or parameters), rather than exact copies alone. String
morphism/NU-system research shows a genuine distinction in representational
power. It supplies no cheap learner or RLBWT inversion construction here.
It should be promoted only after an inexpensive census establishes large,
repeated, cheaply certifiable templates on the actual intermediate stream.

## 8. Small experiments that should precede more implementation

* Oracle expansion of the already counted LZ77/RePair descriptions on exactly
  the measured real blocks. Count small-copy overhead, output traffic, metadata
  and final RLE1/CRC; do not count offline construction as free in the result.
* Export actual inverse-BWT run records and apply these untimed probes. Report
  E, H, Q, peak frontier and output equality. The code here is ready for an
  input adapter, but no real corpus was available in this environment.
* For a surviving constructor, compare whole-decoder inclusive spans on one
  matched machine/build/context. Include refusals, allocation/reuse, fallback,
  reader copies and multicore contention. The #367 investigation remains useful
  engineering work and is not superseded by this mathematical investigation.

## 9. Reproduction

Python 3.10+ (3.11+ recommended), standard library only:

    python batch_induction.py --out batch_results.json
    python phrase_induction.py
    python copy_dependency_example.py

The first script prints incidental execution time of correctness checks; that
number is NOT a performance comparison. Fixture generation uses plaintext to
create a cyclic BWT; plaintext is not used by either candidate construction.
Validation intentionally uses slow independent oracles. The prose fixture is
the GNU GPL version 3 license text, with the complete license bundled; census
uses its first 32,768 bytes. Fixture SHA-256 values are in batch_results.json.
Other fixtures are deterministic and generated by code. The JSON is synthetic,
not a Wikidata sample. No result here uses unavailable real-Wikidata data.

## 10. Sources and pins

Repository sources reviewed through the GitHub connection:

* https://github.com/lmmx/mibz2/pull/365 (measured qualification at 57f51bbc0ca62ac3ee1cc148a1010842eea6222a)
* https://github.com/lmmx/mibz2/pull/367 (head a08277e1c36e276cfea7b3116d99ea5fd93f0e84)
* https://github.com/lmmx/mibz2/blob/master/docs/journal/monodromy/2026-11-10-rauzy-veech.md
  (read blob cc8af4590d7e88f52b05c8008d3a3df226c4d644, especially sections 5.4–5.6, 6.3–6.4 and 8)
* https://github.com/lmmx/mibz2/blob/master/docs/team/MONODROMY-ORIENTATION.md
  (read blob 29c5577cc57dd78c8854d675566f10cb721d9179)
* https://github.com/lmmx/mibz2/pull/313
* https://github.com/lmmx/mibz2/blob/f1fe26f425f39bd07f1df6e8e9471ff88cbb3fc8/docs/journal/2026-11-10-interval-exchange.md
* https://github.com/lmmx/mibz2/blob/f1fe26f425f39bd07f1df6e8e9471ff88cbb3fc8/docs/journal/2026-11-10-by-data-class.md
* https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-10-parallel-scan.md

Primary literature reviewed or checked:

* Jeż, Approximation of grammar-based compression via recompression:
  https://arxiv.org/abs/1301.5842 . This works from an input string; using its
  ideas does not make BWT-to-grammar construction automatically cheap.
* Kempa, Optimal Construction of Compressed Indexes for Highly Repetitive Texts:
  https://arxiv.org/html/1712.04886v8 . Compressed construction support is real,
  but theoretical polylogarithmic terms and supporting indexes must be charged.
* Kärkkäinen, Kempa and Puglisi, Slashing the Time for BWT Inversion, 2012:
  https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf . Multiple chains,
  two-symbol steps and precopying are prior art; headline gains against older
  baselines are not gains against mibz2. Their parallel segments assume supplied
  starting positions in the compressed representation.
* Brown and Langmead, Bounding the Average Move Structure Query for Faster and
  Smaller RLBWT Permutations, SEA 2026:
  https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html .
  Smaller/faster LF queries are not the same as eliminating the n-query walk.
* Russo, Correia, Navarro and Francisco, Approximating Optimal Bidirectional
  Macro Schemes, 2020:
  https://users.dcc.uchile.cl/~gnavarro/ps/dcc20.1.pdf . Validity concerns byte-level
  dependencies; their construction uses suffix-array information, not a free
  run-level inverse-BWT primitive.
* Navarro and Urbina, Repetitiveness Measures Based on String Morphisms, 2025:
  https://users.dcc.uchile.cl/~gnavarro/abstracts/tcs25.html . Morphism and copy
  mechanisms can capture different regularity; no practical BWT conversion is
  inferred from that representational fact.

## Bottom line

The next potentially multiplicative advance is not "the same cells, in a better
order". It is a cheaper route from run-level dynamics to a much smaller reusable
reconstruction program. First-return induction provides exact transformations
with which to search for that route. The prototypes establish correctness of
two concrete transformations and expose their present cost obstacles; they do
not yet establish a faster real-data decoder.
