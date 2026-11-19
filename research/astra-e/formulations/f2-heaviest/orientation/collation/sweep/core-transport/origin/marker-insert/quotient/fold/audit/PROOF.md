# Confluence of iterated whole-class contraction

For positive token spellings, every maximal sequence of legal contractions leaves the same number of ordinal rows. After separating isolated deterministic cycles and rotating their token seams, it leaves the same weighted labeled permutation, including the original marked byte. The physical order of isolated token buckets can change the number of incoming runs. A six-row example has irreducible columns with four and five runs.

## Model

Start with a finite permutation of physical vertices. Vertices are ordered, each source class is an interval, and the successor map is increasing within each source class. A class has one spelling shared by all its vertices. A source class K may absorb a different class J if every K vertex has its successor in J. For each K vertex append the successor's payload, delete that successor, and link to the following surviving vertex. Other survivors retain their order. This is the legal whole-class step proved in the source record. Incompatible adjacent choices in a single batch are excluded. Every legal independent batch is a sequence of commuting singleton steps, so singleton choices suffice to analyze all maximal histories. [One-step construction](../PROOF.md)

Temporarily attach a unique identity to every initial vertex and every byte position inside it. A surviving vertex carries the consecutive original path it represents. Every contraction concatenates two such paths. A mark on an absorbed successor moves into the predecessor's payload with the old predecessor length added to its offset. Path identity therefore preserves marks at every original byte, for arbitrary many laps on any cycle. The proof allows recurring spellings under distinct labels and repeated byte periods. Zero-byte cycles would require an independent marked-byte convention.

## Resolving all competing one-step choices

Consider two different eligible source classes K and J in the same state.

1. **Neither targets the other.** The K and J source sets are disjoint. Their successor sets are also disjoint by injectivity and contain no source of the other choice. Both sets of source/successor pairs can be concatenated in either order. Sharing a target class leaves the pairs disjoint. The resulting original paths, surviving vertex order, labels and transferred marks are identical.
2. **There is a one-way overlap K → J → T with T distinct from K and J.** First absorb T at every J, then let K absorb its now-extended J successor. This removes every original T vertex immediately after a J and every J vertex immediately after a K. In the opposite order, first absorb J at K. The extended K vertices and all remaining J vertices now target T. Absorb T in each of these classes, omitting J if it vanished. The T vertices used by the two choices partition precisely the old images of J. The deleted vertices on the two routes coincide. The surviving paths are K·J·T at each old K, J·T at every remaining J, and the old paths elsewhere. Hence the original identities, ordering, payloads and mark offsets coincide. No hypothesis about successors of T is used.
3. **There is a mutual overlap K ↔ J.** Injectivity gives f(K) ≤ f(J) ≤ f(K). Both maps are bijections of their entire classes, so the pair has no outside predecessor or successor. In the original stable ordinal system the i-th K maps to the i-th J and conversely. Each pair is its own permutation cycle. Choosing K leaves f(K) fixed points with payload K·J. Choosing J leaves f(J) fixed points with payload J·K. Corresponding rows are the same original circular paths; shifting each retained seam adds or subtracts the same byte offset modulo the positive circumference. This preserves an absolute marked byte on a primitive or a periodic cycle. The chosen class occupies its old position among the other classes, which can differ between the two outcomes.

These cases exhaust all forks up to exchanging K and J.

## Normalization, termination and row optimum

Separate a class all of whose vertices map back into itself. Its stable order-preserving self-bijection fixes every vertex. Its bucket is closed to external transitions, so the class cannot contract or be a target afterward. Record the group of its original circular paths in their within-class source order. Compare paths up to rotation, carrying along the marked absolute byte if present. The group remains separate from other isolated groups. Drop the closed bucket from the active ordered system and preserve relative order of the rest. This normalization commutes with every active contraction. An isolated deterministic component of several classes may equivalently be left active until it contracts to such a group.

The one-way and nonoverlapping forks join exactly. The mutual fork produces the same normalized state. Every active contraction strictly decreases the number of vertices, and normalizing closed buckets only removes more active vertices. The terminating relation is therefore locally confluent on normalized states. Induction on the maximum remaining number of steps (the terminating case of Newman's lemma) joins two maximal histories by resolving their first fork and then their shorter tails. Their terminal active paths and the collection of isolated circular groups coincide. Stable ordinal indexing supplies the same pairing of physical orbits even for equal literal payloads in different orbits.

Every reachable intermediate system can be reduced to a terminal one, since each step strictly decreases row count. It follows that terminal row count is the minimum over **all** legal whole-class histories from that start. The number of residual classes and the sum of spelling lengths over those classes are invariant as well, including isolated classes whose spelling rotates. For a terminal with f frozen fixed-point vertices and a active classes, the residual row count is at least f + 2a. Each active class must have at least two distinct successor types and therefore at least two occurrences. If any active class remains, there must be at least two. This does not give the fastest partial schedule or a bound for arbitrary row-specific merging.

## The physical incoming RLE can differ

Write R_A for the RLE length after deleting all isolated class buckets, k for their number, and m_A for the remaining active rows. Each isolated class contributes exactly one solid incoming run in its physical source bucket. Removing them gives the exact identity

    R_full = R_A + k + s,

where s counts gaps between two consecutive active rows with the same incoming label that contain at least one isolated class bucket. Thus 0 ≤ s ≤ min(k, m_A − R_A), also when the active system is empty. A fixed-point class can land at either of two different old source positions under the mutual fork. Different s can affect the next run-based induction even after row and alphabet counts agree.

Take F=(0,1,1,2,3,3) and incoming L=(2,1,3,0,3,1), with any positive literal word for each class. The singleton classes 0 and 2 form an isolated 2-cycle. Both classes 1 and 3 have two successor types. Retaining 0 leaves incoming (0,1,3,3,1), with four runs. Retaining 2 leaves (1,3,2,3,1), with five runs. Both are irreducible, with identical active rows and the same isolated circular word. Six is the smallest possible initial number of vertices for unequal terminal RLE counts. Unequal s requires at least one frozen class and nonempty active rows. Those active rows include at least two classes with at least two rows each. Thus a terminal needs at least five rows, and an initial divergent state must have at least one extra row to consume.

The two-row input F=(0,1), L=(1,0) already has two raw terminal seam choices. The single orbit is closed and its two length-two spellings are circular shifts. Neither example gives a difference between the normalized marked systems.

**Single-orbit corollary.** If the initial permutation is one cycle, no terminal can contain both a closed fixed-point bucket and active rows. A contraction preserves the original physical cycles, so either the literal active representation agrees, or a single circular row remains. The same conclusion about RLE length holds if all orbits read the same circular type-ID pattern up to rotation, including repeated periods. A whole-class substitution is a local rewrite of that common pattern on every orbit. That property persists through the sequence; a closed fixed-type orbit in a terminal forces all orbits to use that same type. The six-row discrepancy requires heterogeneous token cycles. A primitive ordinary BWT block with a visited named delimiter gives one orbit at that delimiter; ordinary periodic copies with one type per literal return satisfy the common-pattern condition. Systems assigning distinct IDs to aliases require a separate check of the type pattern.

## Exact finite check

The independent enumerator constructs the stable permutation directly from every surjective incoming column and ordered alphabet of up to four labels on at most seven vertices. It explores every legal singleton transition from every such column. Vertex identities remain attached to payloads at every step. It checks stable source ordering, equality of all spellings within a class, bijective transfer of every marked original byte and compatibility with the next-byte permutation. These checks suffice for every marked repetition length. Terminal comparisons preserve the full ordered active identity paths and each frozen group's ordered constituent original orbits. The program also evaluates the RLE identity in every terminal representation.

```bash
d=$(dirname "$(rg --files | rg 'quotient/fold/audit/exhaustive.py$')")
python3 -B "$d/exhaustive.py" | diff - "$d/small.json"
```

Across 13,009 initial columns, the enumeration visited 166,539 states, took 270,914 legal edges, compared 37,528 terminal states and checked 2,782,663 marked byte successors. All normalized terminal systems agreed. In 9,901 inputs the literal identity paths depended on the order. Forty inputs had different terminal RLE lengths. The earliest literal witnesses are machine-readable. The general confluence and six-row minimality statements above use the fork and cardinality arguments. [Independent enumerator](exhaustive.py) · [Finite results](small.json)
