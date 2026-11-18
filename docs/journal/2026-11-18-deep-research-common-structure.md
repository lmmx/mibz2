<!-- Reader's note (added on committing, 18 November 2026). This is a ChatGPT Deep Research
report that Louis commissioned. It is preserved verbatim, including its unresolved `cite…`
citation markers, which point to sources not checked here. Treat every citation as a pointer to
verify, and every "new deduction" as unverified.
What followed:
- MONODROMY tested its central proposal in record 63 (docs/journal/monodromy/, branch
  claude/sharp-meitner-cx0v78, commit 1306f77e; REVIEW on #145).
  - The identification E₀ = 2r − 1 − C, the atoms of P ∨ TP, is VERIFIED on all 233 blocks.
  - q₁ = r − C, which is the forest's chord count.
  - q_k is the depth-k excess of record 62's O3 refinement, so the profile is not a new object.
  - Adding Σ_{k=2..8} q_k/r explains the residual Cells–forest correlation (0.393 → −0.061).
  - "Collective cylinders" adds no algorithmic route beyond O3, Rauzy–Veech or the known
    certificates (record 63 §4). -->

# Common structure in three inverse-Burrows–Wheeler reconstruction schemes

## Executive assessment

The strongest mathematical common structure is **not** an event-by-event correspondence between Cells, adjacent-rank-gap induction and forest-section elimination. It is a common finite piecewise-translation object: the LF permutation, when cut at BWT-run boundaries, is exactly a **discrete interval-exchange transformation**. On each BWT run, LF is an order-preserving translation because consecutive equal symbols have consecutive LF ranks. Thus an \(r\)-run BWT gives a permutation of \(n\) integer positions assembled from \(r\) translated intervals. This is an exact statement, not an analogy. citeturn330100search6turn382094search11turn330100search16

The supplied identity
\[
E_0=2r-1-C
\]
has an equally precise interpretation. Let \(\mathcal P\) be the partition by source runs and \(T\mathcal P\) its LF-image partition. Each has \(r-1\) internal boundaries; if \(C\) boundaries coincide, their common refinement has
\[
1+\bigl|( \partial\mathcal P\cup\partial(T\mathcal P))_{\rm int}\bigr|
 =1+(r-1)+(r-1)-C
 =2r-1-C
\]
atoms. Thus \(E_0\) is a **first-order marked-discontinuity complexity** of the LF interval exchange: the size of the overlay of its domain and range partitions. This is closely related to the standard cylinder-complexity description of interval exchanges, where length-\(k\) coding cells are obtained by refining the original partition with successive inverse images of its boundaries. citeturn709348search7turn709348search1

This interpretation gives a credible explanation for much of the observed correlation. All three algorithms begin from, or rapidly encounter, essentially the same finite arrangement of run boundaries, image boundaries and adjacency chains. Their operation counts can therefore share a strong dependence on \(r\), run density, and especially boundary coincidence or boundary excess, without processing the same subsequent objects. The reported collapse in residual correlation after controlling for initial boundary excess is exactly what one would expect if the algorithms are different elimination/induction schedules whose workloads are largely driven by the same initial refinement complexity.

Classical Rauzy–Veech and return-word theory sharpen that picture, but with an important qualification. A bzip2 LF permutation with integral interval lengths is highly nongeneric as a real interval exchange: translations are integral, the fractional part of every point is preserved, and every orbit lies over a finite permutation of integer cells. Consequently the usual Keane infinite-distinct-orbit condition, minimality, unique ergodicity and generic \((r-1)k+1\) complexity theory are normally inapplicable. citeturn213031search3turn213031search4 The relevant part of Rauzy theory is the **finite first-return and cut-and-stack algebra**, not its asymptotic ergodic consequences.

The most promising algorithmic implication is correspondingly modest but real. Mathematics does permit **collective treatment of whole LF intervals**, return blocks and reusable word rules instead of individually recognising all row-level events. Existing RLBWT work already exploits precisely this fact: LF is stored and traversed as a piecewise-affine permutation with \(O(r)\) pieces, and recent move-structure results obtain full BWT inversion in optimal \(O(n)\) time with \(O(r)\) working space while amortising navigation across a traversal. citeturn382094search7turn986397search1turn986397search4 Grammar and macro-scheme results go further and show that an \(r\)-run BWT can imply an \(O(r)\)-sized bidirectional macro description or an \(O(r\log(n/r))\)-scale grammar representation of the text. citeturn986397search5turn174859search2 These results demonstrate that many dependent reconstruction relations can genuinely be *represented* collectively. They do **not** imply that Cells, gap induction or forest elimination can avoid all equivalent discovery cost, and they do not establish an operation-count lower bound near \(1.8r\).

My overall classification is therefore:

| Relationship | Status |
|---|---|
| LF on BWT runs is a finite/discrete interval exchange | **Exact equivalence** |
| \(E_0=2r-1-C\) is the number of atoms in the source/image common refinement | **Exact equivalence** |
| Boundary coincidences are finite analogues of IET discontinuity connections | **Exact at the marked-partition level; classical infinite-IET consequences do not transfer** |
| Right-end subtraction in Cells versus Rauzy induction | **Exact only for steps satisfying the classical winner/loser first-return rule; otherwise a Rauzy-style transformation** |
| Grammar rules versus return words/substitutions | **Structurally well motivated; classical return-word theorems require hypotheses absent here** |
| Forest and gap algorithms versus boundary-chain decomposition | **Shared structural measure, not event equivalence** |
| Similar operation counts near \(1.8r\) | **Empirical association; no known theorem makes this a universal constant or lower bound** |
| Collective reconstruction of multiple dependent positions | **Mathematically possible in compressed representations; extent of savings for these three particular algorithms remains open** |

## The exact finite interval-exchange model

Write the BWT last column as \(L[0..n-1]\), and let
\[
I_j=[a_j,a_{j+1}),\qquad 0=a_0<a_1<\cdots<a_r=n
\]
be its maximal equal-symbol runs. The standard LF formula is
\[
\operatorname{LF}(i)=C[L_i]+\operatorname{rank}_{L_i}(L,i)-1
\]
in zero-based notation. citeturn382094search11turn330100search0 If \(i,i+1\) lie in the same run, then \(L_i=L_{i+1}\) and the relevant rank increases by exactly one, so
\[
\operatorname{LF}(i+1)=\operatorname{LF}(i)+1.
\]
This consecutive-image property is explicitly exploited by run-length BWT move structures and interval-based LF algorithms. citeturn330100search15turn330100search23

Consequently, for every run \(I_j\) there is an integer \(\delta_j\) such that
\[
\operatorname{LF}(i)=i+\delta_j,\qquad i\in I_j.
\]
Its image
\[
J_j=\operatorname{LF}(I_j)
\]
is therefore an integer interval of the same length. Since LF is a permutation, the \(J_j\) are disjoint and cover \([0,n)\). This is exactly the combinatorial definition of a **block permutation**, the discrete analogue of an orientation-preserving interval exchange. citeturn330100search16

There is also an exact continuous embedding. For
\(x=i+\theta\), \(0\leq\theta<1\), set
\[
T(x)=\operatorname{LF}(i)+\theta.
\]
Equivalently, on every source run,
\[
T(x)=x+\delta_j.
\]
Then \(T:[0,n)\to[0,n)\) is a genuine interval exchange with integer interval lengths, and restricting \(T\) to the integer lattice recovers LF. Standard definitions of an IET are precisely bijections obtained by partitioning an interval into finitely many pieces and translating them. citeturn213031search4turn330100search5

This embedding is useful because it makes “Rauzy-like” terminology mathematically meaningful. Classical Rauzy induction compares the two rightmost intervals in the domain and range order, removes the shorter terminal piece, and identifies the induced map on the remaining interval with a first-return IET. The associated visitation matrices record how old intervals are concatenated to form return itineraries. citeturn382094search9turn213031search3 Therefore, whenever a Cells step literally performs this terminal winner/loser subtraction on the marked LF intervals, that step is an exact finite Rauzy move and its produced word rule is the finite counterpart of a Rauzy visitation substitution.

The qualification matters. Classical Rauzy induction is normally studied under hypotheses ensuring the rightmost lengths are unequal and induction can continue indefinitely. The Keane condition is equivalent, in the irreducible case, to absence of discontinuity connections and to indefinite well-defined Rauzy induction; under that condition the natural coding has complexity
\[
p(k)=(d-1)k+1.
\]
citeturn213031search3turn709348search8 Integer LF exchanges are almost the opposite regime. Because every \(\delta_j\) is integral,
\[
T(i+\theta)=\operatorname{LF}(i)+\theta,
\]
so \(\theta\) is invariant and each horizontal fibre \(\{0+\theta,\ldots,n-1+\theta\}\) is permuted finitely. Thus all such orbits are periodic. Equal terminal lengths and finite discontinuity connections can occur routinely. Generic minimality, ergodicity, asymptotic Rauzy frequencies and “almost every length vector” results should therefore **not** be imported into bzip2.

That distinction also explains why “Rauzy–Veech-style” is safer than simply identifying Cells with the classical dynamical system globally. The mibz2 documentation itself describes Cells as a Rauzy–Veech induction that constructs reusable word rules, but its finite engineering treatment includes special termination and storage behaviour not supplied by the classical infinite-induction theory. citeturn382094search0turn880557search0

For bzip2 specifically, the relevant object is a cyclic BWT together with an origin pointer identifying the original rotation, rather than the usual theoretical convention of appending a unique endmarker. The wire format stores a 24-bit origin pointer into the sorted BWT rotations. citeturn903443search22turn903443search14 This does not alter the piecewise-translation proof above, but it matters when importing theorems whose proofs assume a unique \(\$\) and hence a distinguished single LF cycle.

**Established mathematics:** runwise LF is exactly a discrete IET; classical right-Rauzy moves are first-return transformations.

**New deduction for mibz2:** the correct continuous model is an *integer, periodic, marked* IET, not a generic real IET. Its utility is combinatorial and algebraic rather than ergodic.

## Boundary coincidences, discontinuity orbits and the meaning of \(E_0\)

Let
\[
D_s=\{a_1,\ldots,a_{r-1}\}
\]
be the internal source-run boundaries and let
\[
D_t
\]
be the \(r-1\) internal boundaries of the image intervals \(J_j\), sorted in target order. These should be regarded as **marked boundaries** rather than automatically as topological discontinuities: an algorithm may retain a boundary even in a degenerate presentation where adjacent translations could be amalgamated.

The common refinement of the two interval partitions is cut at every point in \(D_s\cup D_t\). If
\[
C=|D_s\cap D_t|,
\]
then
\[
|D_s\cup D_t|
 =(r-1)+(r-1)-C,
\]
and hence the number of refinement atoms is
\[
1+|D_s\cup D_t|
 =2r-1-C.
\]
Thus the supplied \(E_0\) identity is precisely a partition-refinement identity.

This has a standard dynamical interpretation. For an IET with natural partition \(\mathcal P\), the words of length \(k\) in its natural coding correspond to atoms of
\[
\mathcal P\vee T^{-1}\mathcal P
 \vee\cdots\vee T^{-(k-1)}\mathcal P.
\]
Equivalently, their endpoints are generated by successive preimages of the discontinuities. citeturn709348search7turn709348search1 Applying \(T\) to the length-two refinement identifies it with the overlay of \(\mathcal P\) and \(T\mathcal P\). Therefore, when all \(r\) run intervals are given distinct labels, \(E_0\) is exactly the **length-two cylinder complexity** of this marked finite exchange.

For a regular \(r\)-IET with no discontinuity connections, every generation supplies \(r-1\) new distinct cut points and one obtains
\[
p(k)=(r-1)k+1.
\]
When discontinuity orbits collide, complexity falls below this maximum because several nominal cuts represent the same point. citeturn709348search0turn213031search3 In the finite LF setting collisions are unavoidable eventually; at depth one, \(C\) is exactly the deficit from the maximal value \(2r-1\).

This gives a considerably stronger interpretation of \(C\) than “another empirical predictor”. It is a count of the **first generation of singularity-orbit coincidences**. The corresponding excess
\[
2r-1-E_0=C
\]
measures how much nominal two-layer boundary information has already coalesced before any induction schedule is chosen.

That observation provides a natural explanation for the reported regression behaviour. Run density measures how much physical text length is represented per initial interval. Boundary excess measures something orthogonal: how badly the two interval orders disagree locally and therefore how fragmented the first common refinement is. Two algorithms can consequently have highly correlated workloads merely because both inherit their candidate interactions from this refinement. Controlling for \(r/n\) alone leaves this source of variation untouched; controlling also for the refinement excess can remove much of the residual association. That explanation requires no event-level bijection.

The result should not be overextended. The quantity \(E_0\) is attached to a **chosen marked presentation**. IET theory explicitly warns that combinatorial presentations need not be unique: an interval exchange can sometimes be represented with redundant subdivisions, while its actual map remains unchanged. citeturn709348search4 Splitting a run interval artificially could increase marked boundaries and therefore alter \(E_0\) without changing the underlying permutation. Hence \(E_0\) is not an invariant of LF as an abstract permutation unless one specifies the canonical maximal-BWT-run presentation. This matches the caution in the question.

There is also a useful higher-order object:
\[
B_k=
\bigcup_{j=0}^{k-1}T^{-j}D_s.
\]
The number of atoms of the \(k\)-fold refinement is \(1+|B_k|\). For generic regular IETs this grows at the maximal rate \(r-1\) per generation; in an LF permutation it grows only when a newly pulled-back marked boundary has not appeared before. This suggests a finite **boundary-orbit profile**
\[
q_k=|B_{k+1}\setminus B_k|
\]
as a more principled structural descriptor than any single initial excess.

**New deduction:** \(C\) is the depth-one collision count of marked boundary orbits, and \(E_0\) is the corresponding length-two partition complexity. A sequence such as \(q_1,q_2,\ldots\) is a plausible schedule-independent object capable of explaining more of the three operation counts than \(E_0\) alone.

**Limitation:** there is presently no theorem identifying Cells operations, gap operations or forest arcs with individual elements of \(B_k\). Boundary-orbit complexity is a common *supply of possible interactions*, not a count of algorithmic events.

## Rauzy moves, return words and substitutions

The closest established theory to Cells’ reusable word rules is the theory of induced IETs and return words. Under Rauzy induction, the induced first-return map has intervals whose itineraries through the old partition are concatenations of the previous itineraries; algebraically, one obtains products of elementary nonnegative visitation matrices. citeturn382094search9 This is precisely the mechanism by which an interval induction becomes a word substitution: a “winner” itinerary acquires the “loser” itinerary as a prefix or suffix while the other words remain unchanged.

There is an extensive literature making that connection exact for regular interval exchanges. Dolce and Perrin show that admissible subintervals are characterised by branching Rauzy induction and that derivation by return words preserves the class of regular interval-exchange sets. citeturn382094search2 Earlier work on substitutions for rotation-class IETs likewise constructs symbolic orbits by compositions of elementary substitutions closely related to Rauzy induction. citeturn382094search3

Return-word theory supplies a second algebraic fact. If \(u\) is a prefix in a uniformly recurrent word, return words to a longer prefix decompose uniquely into return words to \(u\); derived sequences therefore form a hierarchy of exact word substitutions. citeturn213031search5 For uniformly recurrent tree sets, including natural codings of regular IETs, first-return words even form a basis of the free group, and return-word decoding preserves the tree-set structure. citeturn213031search7

These theorems explain why a Rauzy reconstruction naturally produces a grammar or word DAG rather than a flat LF walk. A first-return operation does not merely discover the next symbol: it discovers a whole itinerary whose expansion can subsequently be reused. That is genuine collective computation.

However, none of the strongest return-word theorems transfers automatically to bzip2. Uniform recurrence, minimality, regularity and the Keane condition are central hypotheses in the cited results. citeturn213031search5turn213031search7 The finite integer LF exchange fails the infinite-distinct-orbit condition. A bzip2 block may also have reducible interval combinatorics or periodic rotation structure. Thus statements such as “there are exactly \(r\) return words”, “return words form a free-group basis” or “the derived language remains a regular IET language” cannot be asserted for an arbitrary block.

A finite replacement is nonetheless valid without any ergodic assumptions. Given a subset \(J\) of the finite LF state space, every \(i\in J\) has a finite first-return time whenever its LF cycle meets \(J\) again. Its itinerary before return is a finite word. If several starting points share an identical itinerary, that word can be represented once and referenced repeatedly. This is elementary finite dynamics and does not require minimality. Classical return-word theory should therefore be viewed here as a **design pattern and algebraic model**, not as a source of generic counting formulas.

The same distinction matters for adjacent-rank-gap induction. A gap between adjacent ranks is a relation in the *ordered presentation* of the permutation. If induction repeatedly removes or coalesces gaps, this resembles derivation of a coding partition, but a gap is not automatically a return word or a Rauzy loser. Two gaps may collapse because a boundary orbit has become identified while Cells may encode the same consequence as one grammar substitution—or several. No known theorem forces equality of those counts.

**Promising conjecture:** Cells rules and maximal surviving gap chains may admit a common description as two bases for a finite category of first-return itineraries on the marked LF exchange. Such a statement would be weaker than event bijection: one Cells rule could correspond to a concatenation of several gap-chain generators. The appropriate mathematical object would probably be a finite substitution or path category rather than the raw operation traces.

A route to proving or refuting this would be to assign every operation a pair of boundary-orbit endpoints and compare the *generated interval/path it generates*, rather than operation timestamps. Equality of the resulting generated submonoids or path languages would establish a genuine transformation-level equivalence while allowing differing operation totals.

## Graph topology, adjacency chains and forest elimination

There is a second exact representation independent of real-IET language. Regard LF as a functional directed graph on \(\{0,\ldots,n-1\}\). Because LF is a permutation, every vertex has indegree and outdegree one, so the graph is a disjoint union of directed cycles. The bzip2 origin pointer selects the rotation from which the desired output traversal begins. citeturn903443search22turn330100search7

Run compression contracts long stretches on which the permutation is the translation \(i\mapsto i+\delta_j\). What remains are the places at which adjacency is interrupted by a source or target boundary. The common refinement above therefore supplies a natural path decomposition of the permutation into maximal pieces on which both source rank and image rank have uniform local behaviour.

Under the premise supplied in the question that \(E_0-V_0\) counts interrupted adjacency-gap chains, this has an immediate structural interpretation: the excess is a **path-fragmentation statistic** of the initial marked permutation. Each additional unmatched source/image boundary can terminate one otherwise continuous rank-adjacency chain. This is exactly the same combinatorial phenomenon that raises the common-refinement count.

This gives a plausible common cause for gap induction and forest elimination. Gap induction explicitly works on these interrupted rank adjacencies. A forest representation can encode the same chain fragments as paths connected through branch or section nodes. Eliminating a forest section then removes or contracts a collection of these paths. Cells sees neither the same “gap” nor necessarily the same “arc”; it sees terminal interval relations. Yet all three are different quotient presentations of the same boundary-fragmented permutation.

The schedule dependence of forest arc counts is important evidence about what is *not* invariant. In classical graph elimination, eliminating a vertex can create fill edges among its remaining neighbours, and the amount of fill generally depends on elimination order. Rose, Tarjan and Lueker developed the standard graph-theoretic formulation and algorithms for computing fill induced by a given ordering; perfect elimination orderings are precisely the special cases avoiding fill on chordal graphs. citeturn473041search0turn473041search22 The analogy with forest-section elimination should not be claimed as an exact reduction without its update rules, but it demonstrates a well-established mathematical pattern: a fixed underlying graph can admit different elimination schedules with different intermediate and total edge work.

That makes the independent observation about forest tie-breaking mathematically unsurprising. If tie-breaking changes the number of arcs processed while the underlying LF map and Cells trace remain unchanged, the forest arc count cannot itself be a topological invariant of the LF exchange. It is better understood as **presentation-plus-schedule work**.

A useful hierarchy therefore emerges:

\[
\text{LF permutation}
\longrightarrow
\text{canonical run partition}
\longrightarrow
\text{source/image overlay}
\longrightarrow
\text{chosen graph/forest presentation}
\longrightarrow
\text{elimination schedule}.
\]

Quantities become progressively less invariant as one moves to the right. Cycle structure belongs to the first level. \(r\) belongs to the canonical BWT-run presentation. \(C\) and \(E_0\) belong to the overlay. Forest arc counts may belong only to the final two levels.

This hierarchy also explains why a large raw Cells/forest correlation need not survive adjustment for \(E_0\). If
\[
N_{\rm cells}=f(\mathcal O)+\epsilon_C,\qquad
N_{\rm forest}=g(\mathcal O,S)+\epsilon_F,
\]
where \(\mathcal O\) denotes overlay complexity and \(S\) a forest schedule, the covariance of \(N_{\rm cells}\) and \(N_{\rm forest}\) can be dominated by variation of \(\mathcal O\), despite the absence of a direct mapping between the operations. Removing a good proxy for \(\mathcal O\) removes that common-cause component.

**Established mathematics:** permutation cycles, common refinements and order-dependent graph elimination are rigorous frameworks.

**New deduction:** the observed facts are consistent with a latent-variable model in which the source/image overlay is the shared structural cause and forest scheduling supplies independent extra work.

**What is not established:** that the forest is a chordal-elimination graph, that its arcs equal IET singularity connections, or that \(E_0-V_0\) is an Euler characteristic invariant. Those would require exact definitions of the forest updates and are not consequences of the present data.

## Can dependent reconstruction events genuinely be processed collectively?

The literature gives an unambiguous answer at the level of *possibility*: **yes, dependent LF navigation can be grouped so that one structural operation represents many row-level transitions**.

The simplest example is already the runwise LF identity
\[
\operatorname{LF}(i+t)=\operatorname{LF}(i)+t
\]
while \(i,\ldots,i+t\) remain inside one run. Modern RLBWT “move structures” store such piecewise translations and answer LF navigation from \(O(r)\)-sized structures rather than an \(n\)-entry table. Nishimoto and Tabei obtained constant-time LF and related queries in \(O(r)\) words. citeturn532584view4 Subsequent work explicitly decomposes LF into permutation runs and uses table lookups plus sequential scans. citeturn382094search5 The 2026 average-query analysis shows that an \(O(r)\)-time-and-space construction can support a complete LF traversal, giving optimal-time BWT inversion in \(O(r)\) working space while amortising move-query cost over the cycle. citeturn986397search1turn986397search4

A stronger form of collectivisation comes from dictionary representations. The literature shows that an RLBWT with \(r\) runs can induce a bidirectional macro scheme of \(O(r)\) phrases, and locally consistent parsing yields grammar representations on the order of \(r\log(n/r)\). citeturn986397search5turn174859search2 String-attractor theory places BWT runs, LZ parsing, grammars and macro schemes in a common framework in which a small set of text positions captures all distinct substrings. citeturn174859search4turn174859search6 Conversion from RLBWT to LZ-style descriptions can also be performed without first materialising the entire text, although known bounds vary with the precise representation. citeturn174859search0turn174859search1

These results are highly relevant to Cells. A grammar rule such as
\[
X\to YZ
\]
can stand for an arbitrarily long text fragment, so recognition cost can in principle be proportional to grammar size rather than expansion length. Expansion to the literal block still costs \(\Omega(n)\) simply because \(n\) output bytes must be written, but **structural discovery need not perform \(n\) independent LF recognitions**. The distinction between discovery and output is crucial.

The harder question is whether several *run-level induction events* can likewise be skipped. Here the literature is less decisive. Rauzy acceleration provides a conceptual precedent: rather than performing one elementary Rauzy subtraction at a time, accelerated algorithms group consecutive moves with the same winner, exactly as the Euclidean algorithm can replace repeated subtraction by a quotient step. Zorich acceleration is the standard dynamical example. citeturn382094search9 In a finite integer setting, an even simpler arithmetic acceleration is possible whenever the same terminal interval remains winner for \(q\) consecutive subtractions: one may compute \(q=\lfloor a/b\rfloor\) subject to the next combinatorial change and update the relevant visitation/substitution matrix in one arithmetic operation.

That does **not** automatically reduce total reconstruction cost, because three distinct costs must be separated:

1. determining that the same combinatorial relation persists for \(q\) steps;
2. updating the compact structural representation;
3. eventually expanding the represented word.

An acceleration wins only if cost (1) can be inferred from endpoint lengths, ranks or aggregate metadata without individually recognising the \(q\) elementary events. Classical Rauzy acceleration shows that this is sometimes possible from interval lengths alone. citeturn213031search3turn382094search9 Runwise LF move structures show the analogous fact for permutation navigation. citeturn986397search1 Grammar results show it for repeated output fragments. citeturn174859search3 There is therefore no mathematical barrier saying every elementary Cells/gap/forest event must be separately recognised.

On the other hand, none of these results proves an asymptotic shortcut for the three mibz2 event systems. A sequence of nominally dependent events may change winner, chain endpoint or forest connectivity after every step. If deciding the maximal safe batch requires examining every intermediate breakpoint, batching only moves the cost. The relevant theoretical target is consequently not “fewer event objects” but a **certificate of persistence**: a compact condition proving that \(q\) elementary transformations have a predictable aggregate effect.

The boundary-orbit formulation suggests one such certificate. If a whole interval contains no point of
\[
B_k=\bigcup_{j<k}T^{-j}D_s
\]
before a known depth, all points in that interval have the same length-\(k\) symbolic itinerary. They can therefore be processed as one cylinder until the next boundary-orbit cut. This is exactly how natural-coding cylinders of an IET arise. citeturn709348search7turn709348search1 For the finite LF permutation, the same statement remains true without any Keane assumption; collisions merely make the cylinder family smaller.

This yields the most concrete research proposition from the present investigation:

\[
\boxed{\text{Batch maximal regions with identical finite boundary-orbit itinerary, not nominal algorithm events.}}
\]

Such regions are schedule-independent objects derived from LF itself. Cells rules, gap chains and forest sections could then be tested as different ways of discovering or approximating them. If one operation can certify a cylinder containing many eventual elementary events, genuine collective recognition has occurred. If every cylinder split must itself be discovered one by one, the apparent batching offers no asymptotic recognition saving.

## Conclusions and research status

The literature supports a deeper relationship among the three algorithms, but it is a relationship at the level of **finite interval partitions and boundary dynamics**, not a hidden universal bijection between their traces.

The first exact fact is that run-compressed LF is a discrete interval exchange. That follows directly from LF’s consecutive-rank property on every BWT run and is already exploited by modern RLBWT navigation structures. citeturn330100search15turn382094search7 The second exact fact is that
\[
E_0=2r-1-C
\]
is the cardinality of the common refinement of the run partition and its LF-image partition. In IET language it is the two-step marked cylinder complexity, and \(C\) records first-generation boundary-orbit collisions. The broader IET literature identifies exactly this mechanism—preimages and collisions of discontinuities—as the source of symbolic-complexity growth and deficit. citeturn709348search0turn709348search7

Those facts provide a credible structural explanation for correlated operation counts. A block with many uncompensated source/image boundaries presents more fragments, more broken adjacency chains and more potential first-return distinctions to *any* reconstruction algorithm using the runwise LF structure. A block with many coincident boundaries presents fewer. This shared initial geometry can produce high correlation even when later schedules differ substantially.

The evidence against a simple correspondence remains decisive. Forest work is schedule-sensitive; Cells is not changed by that forest tie-breaking. Classical graph-elimination theory gives a familiar mathematical mechanism for exactly this sort of order-dependent auxiliary work. citeturn473041search0 Therefore forest arc count should not be treated as an invariant of the LF exchange, and its numerical closeness to Cells count cannot establish an event bijection.

Return-word and substitution theory provides a persuasive explanation for why Cells-like induction can build reusable word programs. Rauzy induction is fundamentally a first-return construction whose visitation matrices encode concatenated itineraries, and regular-IET return words form highly structured substitution systems. citeturn382094search2turn213031search7 But the strongest classical theorems assume regular, minimal or uniformly recurrent infinite systems; an integer bzip2 LF exchange is periodic and violates those hypotheses. The appropriate lesson is algebraic—first-return itineraries compose—not ergodic.

The operation ratios \(1.837r\), \(1.929r\) and \(1.778r\) therefore have, at present, **no established mathematical status** beyond empirical statistics for the specified corpus. Nothing in IET complexity, return-word theory, graph elimination or RLBWT indexing singles out a constant near \(1.8\). In particular, the standard maximal two-step partition count is near \(2r\), but that numerical proximity is insufficient: \(E_0\) is an initial atom count, whereas the reported algorithm totals count schedule-dependent operations of different kinds. Equating the two would be a category error.

The most promising new invariant to investigate is not another total operation count but the finite marked-boundary orbit profile
\[
q_k=
\left|
\left(\bigcup_{j=0}^{k}T^{-j}D_s\right)
\setminus
\left(\bigcup_{j=0}^{k-1}T^{-j}D_s\right)
\right|.
\]
It is canonical once maximal BWT runs are fixed, it directly generalises the coincidence statistic behind \(E_0\), and it determines how quickly distinct finite LF itineraries proliferate. IET theory establishes the relevance of exactly these discontinuity-preimage sets to coding complexity. citeturn709348search7turn709348search1 What remains conjectural is whether low-order summaries of this profile predict Cells, gap and forest work substantially better than \(r\), run density and \(E_0\) alone.

A second promising object is the set of **maximal cylinders with common LF itinerary up to their next marked boundary encounter**. These cylinders are invariant under forest tie-breaking and do not privilege Cells or gaps. If all three algorithms can be mapped to refinements or contractions of this cylinder family, one would have the sought common structure without demanding event correspondence.

Finally, the answer to the algorithmic question is qualified but positive. Existing mathematics already proves that LF transitions, text references and repeated output fragments can be represented and navigated collectively in \(O(r)\)- or \(O(r\log(n/r))\)-scale structures. citeturn986397search4turn986397search5turn174859search3 There is therefore no general lower-bound principle forcing one recognition operation per elementary reconstruction dependency. What has **not** been demonstrated is that the particular Cells, adjacent-gap and forest events contain enough long, cheaply certifiable regularity to realise such a saving beyond constant factors. The right mathematical question is whether maximal common-itinerary regions or accelerated first-return steps can be certified from aggregate endpoint data. If so, several dependent events can genuinely be collapsed. If not, their similar counts are best understood as three responses to the same fragmented boundary geometry rather than evidence of a hidden common algorithm.

The strongest defensible conclusion is thus intermediate rather than spectacular: **there is a real common mathematical substrate—the finite marked interval exchange and its boundary-orbit refinement—but no present theorem identifies the three operation streams. The initial excess is naturally explained as a first-order discontinuity-complexity statistic, which in turn gives a plausible common-cause explanation for the empirical correlations. Collective reconstruction is mathematically possible in principle and already realised elsewhere in RLBWT theory, but proving an analogous saving for these specific inductions requires a new theorem about batching finite boundary-orbit cylinders or first-return substitutions.**