# 2026-11-18 · What the BWT needs to know at a branch

Given the block length and a marked future, a cyclic block can be recovered from the **support of
its factor language**. Here support means which finite strings occur, with their frequencies forgotten. There is a finite, executable way to
supply this information. For a context that admits several bytes on both the left and the right,
record which left and right extensions actually occur together. The decoder discovers where to ask
each question from its previous answers. It receives no factor names, phrase spellings, interval
widths or occurrence counts.

On the frozen block, a **67,024-byte file** is sufficient to recover both the 899,931 inverse-BWT
bytes and the entire original last column. The companion includes the file and a standalone decoder.
On the 431-block Wikidata corpus the same construction produced the reference output through
ordinary RLE1 and the block CRCs. The earlier timing result still applies to the two phrase
constructors. This experiment measures what suffices to identify the output. Its encoder starts
from the byte BWT and pays to discover the extensions.
[Standalone decoder and frozen input](2026-11-18-bispecial-factors/README.md) · [Earlier phrase experiment](2026-11-18-phrase-bwt-rle1.md)

## The spelling–order split depends on the section

Consider the indexed words `#|x₁|x₂|…xₘ|`, with each x either 0 or 1 and exactly h ones.
The unique `#` fixes the origin. There are exactly $\binom{m}{h}$ possible texts.

* Cut after `#`. There is one phrase, its token BWT is constant, and its spelling specifies the
  entire choice.
* Cut after `|`. The dictionary is `#|`, `0|`, `1|`, with fixed multiplicities when
  $0<h<m$. The order after `#|` specifies the entire choice.

Both markers satisfy the earlier RLE1 condition. This moves an arbitrarily large amount of
information between the two accounts while keeping the same family, byte histogram and origin.
The verifier checks all 70 texts for $m=8,h=4$. The frozen block also illustrates the dependence
on the cut. Byte 59 occurs once and makes an 899,931-byte dictionary. Byte 34 makes 2,716 phrases
with 83,454 distinct spelling bytes. A spelling cost in a particular constructor is real. There
is no corresponding invariant amount of spelling information that every inverse must discover.
[All 89 marker choices](2026-11-18-bispecial-factors/evidence/frozen-markers.csv)

## The object that survives a change of section

Regard each row as the infinite periodic future beginning at that rotation. For any finite factor
$w$, write $I(w)$ for the set of rows whose future begins with $w$. These are intervals in rotation
order. A **bispecial factor** has at least two possible preceding symbols and two possible following
symbols. Its **extension graph** is bipartite. It has an edge $(a,b)$ exactly when $awb$ occurs.
This is the usual extension graph of combinatorics on words. At fixed context length it describes
how successive edges of the Rauzy graph, also called the order-$k$ de Bruijn graph of the text,
may be joined. The earlier separator selects one family of stopping contexts in this same language.
[Berthé et al., *Acyclic, connected and tree sets*](https://arxiv.org/abs/1308.4260) · [Belazzougui and Cunial on BWT intervals and de Bruijn graphs](https://doi.org/10.4230/LIPIcs.CPM.2019.10)

Start with the set of occurring bytes in alphabetic order. It gives the first split of all futures.
Maintain a name for each class of futures, its right subclasses in lexicographic order, and for each
possible predecessor $a$ a pointer to the class $I(aw)$. A name here is an allocated index. The
decoder never constructs the string $w$.

1. When $I(w)$ splits into $I(wb_1),\ldots,I(wb_t)$, consider each predecessor $a$ in byte order.
   With just one predecessor, all $t$ continuations occur after it. With more than one, read the
   corresponding row of the binary extension graph.
2. If $a$ occurs with only $b_j$, then $I(aw)=I(awb_j)$. Reuse that class. If it occurs with
   several $b_j$, split $I(aw)$ into fresh classes in their order and queue that split.
3. Attach each resulting class as the predecessor target of its matching $I(wb_j)$. Process splits
   breadth first. Rows of bits in the file have no keys, lengths or addresses for factors.

The invariant follows directly from stable LF. Prefixing $a$ retains the relative order of exactly
those occurrences preceded by $a$. A context with a unique right extension has the same occurrence
set as that extension, so its class and all its predecessor targets can be reused. The weighted
construction did these operations on intervals with numerical endpoints. Equality, subdivision and
the order of the children suffice for the unweighted construction.

For a small example, the cyclic word `001` supplies the pairs `00`, `01`, `10` at the empty
context. The class beginning with `0` splits. At that context the permitted triples are `001`
and `100`: a preceding `0` can meet a following `1`, and a preceding `1` can meet a following
`0`. The future beginning with `1` has only one continuation. The three terminal classes in order
are `00`, `01`, and `1`, with predecessor bytes `1`, `0`, `0`. The same answers describe
`001001`. Its length makes every class twice as wide, giving the last column `110000`.

For termination, let $p(k)$ be the number of distinct length-$k$ futures, with $p(0)=1$. The
partitions refine one another. A split with $t$ right children adds $t-1$ classes. There are only
$D$ distinct infinite periodic futures. Any two unequal futures differ at a finite position, so
$p(k)$ reaches $D$. At stabilization the number of right extensions summed over all contexts equals
the number of contexts. The same holds for left extensions. Every class then has exactly one
predecessor and one successor. The terminal graph is a disjoint union of cycles. The decoder emits
the marked cycle in the required direction, repeating or truncating to the block length. The split
tree has at most $2D-1$ nodes even when some distinguishing contexts are very long.

## Precisely which multiplicities are recoverable

Every terminal cycle is a distinct primitive necklace. Its vertices are the different rotations of
that necklace. Two cycles with the same future somewhere would share its terminal class. Conversely
the set of necklaces determines the language, hence every answer read by the decoder. The unmarked
support description is therefore a complete invariant of the **set of distinct primitive
necklaces** of the byte column. This builds on the extended-BWT correspondence already in the
project journal. It gives an unweighted representation and an inverse for that invariant.
[Earlier necklace correspondence](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/docs/journal/monodromy/2026-11-10-lyndon-literature.md)

Suppose necklace $j$ has period $d_j$. LF is a bijection between consecutive terminal classes,
so all the rotations on this cycle have the same multiplicity $m_j$. For any factor $w$ its
frequency is the sum of its cyclic occurrence counts on the distinct necklaces, weighted by these
$m_j$. The full set of columns of length $n$ with this support is in bijection with

$$
\left\{(m_1,\dots,m_s)\in\mathbb Z_{>0}^s:\quad\sum_j d_jm_j=n\right\}.
$$

To construct the column for any such vector, visit the terminal classes in lexical order and write
each predecessor byte $m_j$ times. Stable LF sends the same offset within a class to that offset in
its predecessor. This proves sufficiency and also shows that distinct vectors give distinct columns.
In dynamical terms these are the positive integer masses of the periodic orbits. On a single orbit,
shift invariance forces the uniform probability measure. Knowing its support and $n$ fixes every
count in the block.

An ordinary cyclic BWT of a text has one distinct necklace, even if the text is a power. Writing
$d$ for its primitive period gives $m=n/d$. Its primary row $p$ selects distinct-future rank
$\lfloor p/m\rfloor$. For an arbitrary last column, the marked **output** requires only the
selected distinct future and $n$. A numerical primary in the original row space can depend on the
discarded multiplicities. For example, columns `aab` and `abb` both describe the fixed points
`a` and `b`. Row 1 selects `a` in the first and `b` in the second. The experimental file
records the distinct-future rank. The full-column and marked-output claims therefore apply to
periodic inputs and to columns with several LF cycles with their respective mark conventions.

## A separate test of how much context fixes a real block

There is a useful way to ask about necessity at a finite depth. Give the decoder the **multiset** of
all cyclic length-$K$ substrings. Count the primitive necklaces consistent with it. A compatible
necklace is an Euler circuit using the prescribed multiplicity of each edge in the order-$(K-1)$
graph. The directed tree count in the BEST theorem enforces the requirement that the output be
one cycle. The underlying reconstruction question is familiar from sequence assembly.
[Counting proof and contraction of deterministic paths](2026-11-18-bispecial-factors/counting.md) · [Bresler, Bresler and Tse on sequence assembly](https://arxiv.org/abs/1301.0068)

| Frozen block, cyclic substring length $K$ | Compatible last columns, any cycle structure | Compatible primitive necklaces |
| ---: | ---: | ---: |
| 2,412 | 48 | 4 |
| 4,744 | 6 | 2 |
| 4,745 | 2 | 1 |
| 4,822 | 2 | 1 |
| 4,823 | 1 | 1 |

Thus **4,745 is the first substring length whose counted spectrum identifies this block up to
rotation**. Longer spectra determine shorter ones by summing extensions. For the last ambiguous
case, the two alternatives are bona fide primitive texts. We constructed both. An independent
comparison using integer IDs for pairs of substrings checks their spectra directly from their
bytes. All 899,853 distinct substrings have identical counts at length 4,744. Four counts differ
at length 4,745. At that next length a two-row BWT interval still has two locally valid orders. Swapping
those two distinct predecessor bytes swaps the outgoing LF images. That splits the primitive cycle
into two, so the single-cycle condition decides the choice. The mark still fixes the rotation.
This threshold is for the counted spectrum on this block. The unweighted decoder above uses
extension support through all distinguishing depths.
[Exact counts](2026-11-18-bispecial-factors/evidence/frozen-assembly.csv) · [Witness columns and cycle lengths](2026-11-18-bispecial-factors/evidence/frozen-witnesses.csv) · [Independent comparison](2026-11-18-bispecial-factors/evidence/witness-spectra.csv)

## What it cost to exhibit the information

The frozen support file has a 52-byte header and 535,773 binary incidence tests. There were
22,261 bispecial contexts. The decoder processed **504,258** contexts with more than one right
extension and ended with **899,931** distinct futures. Most of that construction is forced by
earlier answers. The small file and the much larger constructed object measure different things.
The original independently wrapped bzip2 block occupies 41,618 bytes. The support stream is a
second representation produced from the byte BWT by the research encoder.

Two independent programs tested the distinction. The Rust program compared both inverses on
7,373 arbitrary columns and all 62,286 primary choices. The Python program enumerated 1,787
support classes and recovered exactly the multiplicity-vector count above in every class.
The corpus driver fed only the serialized support to the marked-output inverse, compared its
pre-RLE bytes with Plain, and ran production RLE1 and CRC. The 431 Wikidata blocks yielded
209,700,196 verified final bytes and the six prose blocks yielded 4,947,388. On Wikidata,
6,576,394 of 115,321,698 split contexts required a support table. Decoder throughput and SMT
scaling for this representation remain unmeasured.
[Per-block Wikidata measurements](2026-11-18-bispecial-factors/evidence/wikidata.csv) · [Prose control](2026-11-18-bispecial-factors/evidence/prose.csv)

What the phrase experiment exposed was the difference between identifying an orbit and paying
to present it. A delimiter dictionary is one presentation. The extension graphs give another in
which even the frequencies follow from the support on an ordinary block. The remaining computational
question is how to pass directly from the run-encoded column to the marked orbit while realizing
few of the forced distinctions. The counts above give that question a concrete scale.
