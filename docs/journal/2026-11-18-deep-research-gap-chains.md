<!-- Reader's note (added on committing, 18 November 2026). This is a second ChatGPT Deep
Research report that Louis commissioned, on LF gap chains. It is preserved verbatim, including
its unresolved `cite…` citation markers, which are unchecked.
- *What it worked from:* by its own account it could not read the attached synthesis or review,
  so it worked from the prompt's stated premises.
- *Status:* its theorems, including no gap cycles when LF is a single cycle and
  LCP(τ(g)) = c·LCP(g) along a chain, and its counterexamples are unverified. Independent
  verification is to follow.
- *A likely home in the literature, noted by the coordinator:* the chains are chains of
  reducible LCP values ending at *irreducible* LCP values (those at BWT run heads). Their sum is
  bounded by n lg r + O(n) (Kärkkäinen, Kempa and Piątkowski, CPM 2015), later n log δ. A test
  relating that to the q_k profile is queued for MONODROMY (QUEUED 6984248084 on #145). -->

# The hidden structure of LF gap chains: finite connection orbits, Weiner-link paths, and the limits of collective evaluation

## Scope and principal conclusion

The finite interrupted-gap chains have a substantially sharper mathematical identity than a generic resemblance to interval exchanges. For a primitive cyclic BWT, or for the usual uniquely marked BWT where LF is a single cycle, the run partition makes LF a **finite discrete interval-exchange permutation**: on each maximal BWT run, LF is an order-preserving translation. The induced map on the \(n-1\) adjacency gaps is therefore the boundary action of that discrete interval exchange wherever both neighbouring rows lie in one continuity interval. Classical interval-exchange theory studies precisely the corresponding phenomenon in continuous form: finite orbits joining top and bottom discontinuities are called connections, with Rauzy induction replacing the transformation by a first-return map. citeturn652946search7turn652946search6turn730269search0

There is a second, and algorithmically more informative, exact interpretation. If an adjacency gap is labelled by the longest common prefix of its two BWT rows, then **one application of the gap map is one left extension of that LCP**. Thus an interrupted LF-gap chain is a path of explicit left extensions—equivalently a path of Weiner links between right-special factors—and its terminal run boundary is labelled by a bispecial factor, hence by a maximal repeat except in the usual degenerate empty-factor cases. Weiner's right-to-left suffix-tree construction is the original setting for these left-extension links, while modern compressed text indexes explicitly combine BWT runs with structures organised around maximal repeats. citeturn227178search7turn845158academia2

That identification is exact for finite marked BWT blocks; it requires no limiting IET, ergodicity, recurrence or asymptotic hypothesis.

The negative part is equally important. The chains do **not** supply a hidden conservation law matching Cells, adjacent-rank induction or forest elimination operation-for-operation. They are intrinsic to a particular finite piecewise-translation presentation of LF, whereas induction replaces that presentation by first-return compositions, and even a harmless refinement of the original run partition changes the chain decomposition. A schedule-dependent forest operation count cannot, in particular, be an exact function of any quantity determined solely by the original LF gap chains.

The best exact collective operation I find is therefore narrower than a decoder: **composition of path-local transition summaries along a gap chain**. This can eliminate intermediate events only when the reconstruction state propagated through the chain belongs to a bounded-size associative transition algebra. The chain geometry itself does not imply that property. Existing move-structure results already show how far one can exploit LF's piecewise-translation nature: after an \(O(r)\)-size refinement, LF can be evaluated in constant time per step, and current implementations obtain optimal-time BWT inversion in \(O(r)\) working space. That is strong evidence that any further gain from chain aggregation must come from avoiding *structural reconstruction events*, not from merely making individual LF steps cheaper. citeturn188606search6turn188606search3

I could not access the project's mathematical synthesis or independent review through the tools available in this conversation. I have therefore treated the stated formulas, the 233-block measurements, and the forest tie-breaking counterexample as project premises, rather than inventing or extrapolating their detailed definitions.

## Exact finite identity of the gap chains

Write rows and gaps with zero-based indices. Let

\[
G=\{0,\ldots ,n-2\},
\]

where gap \(g\) lies between rows \(g\) and \(g+1\). If \(g\notin D\), the two rows lie in the same maximal run of \(L\), so

\[
L[g]=L[g+1]=c.
\]

The standard LF formula is

\[
\operatorname{LF}(i)=C[L[i]]+\operatorname{rank}_{L[i]}(L,i),
\]

up to the indexing convention. Consequently

\[
\operatorname{LF}(g+1)=\operatorname{LF}(g)+1
\qquad(g\notin D),
\]

which is exactly why

\[
\tau(g)=\operatorname{LF}(g)
\]

defines an order-preserving map of adjacency gaps. This is the same stable-order property underlying the original BWT inversion procedure and subsequent combinatorial formulations of BWT. citeturn324954search8turn531909academia49

### Newly derived theorem: the finite connection decomposition

**Theorem.** Suppose LF is an \(n\)-cycle, as it is for a primitive cyclic word when rows are distinct, or for the standard uniquely marked text. Then the partial gap permutation \(\tau:G\setminus D\to G\setminus I\) has no directed cycles. Hence every component is a finite directed path. Its initial vertex lies in \(I\), its terminal vertex lies in \(D\); the paths of positive length run from \(I\setminus D\) to \(D\setminus I\), while \(D\cap I\) gives the zero-length components.

**Proof.** If a gap \(g\) belonged to a \(\tau\)-cycle of length \(m\), then

\[
\tau^m(g)=g.
\]

Every defined \(\tau\)-step simultaneously maps the two endpoints of the gap, so

\[
\operatorname{LF}^m(g)=g,\qquad
\operatorname{LF}^m(g+1)=g+1.
\]

An \(n\)-cycle has a fixed point under its \(m\)-th power only if \(n\mid m\). But a simple cycle in \(G\) has at most \(n-1\) vertices, so \(1\le m\le n-1\), impossible. Therefore no gap cycle exists. Since \(\tau\) is injective, every component is a path. Missing predecessors are precisely \(I\); missing successors are precisely \(D\). ∎

This supplies a finite analogue of an interval-exchange connection without importing any infinite-IET theorem. Standard IETs are piecewise translations whose source and image partitions have top and bottom endpoints; in the modern connection convention one follows an orbit from a bottom endpoint to a top endpoint. That is exactly the orientation \(I\to D\) occurring here, except that the phase space is the finite ordered set of adjacency edges. citeturn652946search7turn652946search6

The distinction from Keane's infinite distinct orbit condition is crucial. Continuous regular IET theory is often interested in the *absence* of finite discontinuity connections, because no-connections imply strong minimality properties under appropriate hypotheses. The finite BWT system is almost the opposite regime: because the space has only \(n-1\) gaps, its boundary trajectories necessarily terminate, and those finite connections are the object of interest. Thus Keane/Rauzy language is structurally exact at the level of “piecewise translation + discontinuity connection”, but conclusions about minimal continuous IETs do not transfer. citeturn652946search0turn652946search4

### Newly derived theorem: LCP growth along a chain

Let \(\lambda(g)\) be the LCP of the two cyclic rows adjacent to gap \(g\). If \(g\notin D\) and the common BWT character is \(c\), then

\[
\lambda(\tau(g)) = c\,\lambda(g),
\]

as words, and therefore

\[
|\lambda(\tau(g))|=|\lambda(g)|+1.
\]

**Proof.** LF takes a row representing rotation \(x\) to the rotation obtained by prepending its preceding symbol. If the two adjacent rows are \(x<y\) and \(L[g]=L[g+1]=c\), their images are \(cx\) and \(cy\). Stability of LF makes these two rows adjacent in the \(c\)-block. Their common prefix is therefore exactly \(c\,\operatorname{lcp}(x,y)\). ∎

This is stronger than a loose suffix-array analogy. Every adjacent-row LCP is a right-special factor whenever it is nonempty: the two rows agree on that prefix and diverge immediately afterwards. Along a gap chain, each step left-extends that right-special factor while preserving right-speciality. In suffix-tree terminology these are explicit Weiner-link steps. The terminal gap lies in \(D\), so its two occurrences additionally have distinct left extensions. Hence the terminal LCP is both left-special and right-special: a bispecial factor, and in ordinary string-index terminology a maximal repeat. Weiner's original suffix-tree method is built around right-to-left extension, while modern CDAWG/RLBWT work makes the relation between BWT structure and maximal-repeat extensions explicit. citeturn227178search7turn845158academia2

This gives the strongest exact dictionary I found:

| LF-gap object | Exact finite interpretation |
|---|---|
| BWT run | continuity interval of the discrete piecewise translation LF |
| \(D\) | top/source discontinuities |
| \(I\) | bottom/image discontinuities |
| \(\tau\) | induced action on adjacency edges away from discontinuities |
| interrupted chain | finite top/bottom connection orbit |
| one chain edge | one explicit left extension / Weiner-link step of an adjacent-row LCP |
| terminal \(D\)-gap | adjacent pair whose LCP is left- and right-special |
| chain length | increase in LCP depth between its initial and terminal gap |

The connection with return words is weaker in the unrestricted finite-BWT setting. In regular interval-exchange languages, first-return maps, return words and derived languages are genuinely equivalent pieces of structure; Dolce and Perrin show that derivation by return words stays within the class of regular interval-exchange sets, and recent work connects BWT clustering of return words to regular or generalised IET codings. citeturn652946search2turn628460academia1turn628460academia0 But an arbitrary primitive finite BWT block need not be a return word in any prescribed regular IET language, so no canonical return-word substitution is attached to its gap chains. For general finite marked BWTs, “substitution path” is therefore an analogy unless an additional IET-language hypothesis is supplied.

The relation to bispecial factors is less speculative. Recent work on clustering words characterises clustering through the resolution of certain bispecial factors, reinforcing the fact that BWT run structure is naturally controlled by left/right extension behaviour. citeturn628460academia3 The present gap-chain theorem is nevertheless different: it applies to every finite marked BWT and identifies a concrete path of successive left extensions, not merely a condition on whether the whole BWT clusters.

## Information contained in the chains

There are three different levels of “knowing the chains”, and they should not be conflated.

If one knows only the number \(d\), almost no reconstruction information is retained. It records only the defect

\[
d=r-1-|D\cap I|,
\]

so radically different LF permutations and words can have the same value.

If one knows the multiset of chain lengths, considerably more geometry is visible, but the ordered embedding of the trajectories into BWT-row space is still lost. For example, the primitive binary necklaces represented by

\[
w_1=\texttt{aaaab},\qquad
w_2=\texttt{aabab}
\]

have respectively

\[
L_1=\texttt{baaaa},\qquad
L_2=\texttt{bbaaa}.
\]

Both have two maximal run labels in the order \(b,a\), and in both cases the nontrivial gap system is a single chain of three \(\tau\)-steps. Yet the run lengths are \((1,4)\) and \((2,3)\), LF is different, and the marked decoded necklaces are different. Thus “run-symbol sequence + chain-length multiset + marked-row index” is insufficient.

If, on the other hand, one knows every chain as an **ordered list of actual gap indices**, including the zero-step components, then—under the one-cycle hypothesis—the union of those paths is exactly the graph of \(\tau\). No further gap-map information is hidden: the chain decomposition and \(\tau\) contain precisely the same edge data.

That still does not make the chain representation an independent encoding of the text. If the complete original RLBWT is already known—that is, each run's character and length—then LF itself is already determined by stable character ranks, before the chains are computed. The BWT transform is reversible from \(L\) plus the marked original row; this is the basis of the original inversion algorithm and of the later combinatorial description by Crochemore, Désarménien and Perrin. citeturn324954search8turn531909search43 The full indexed chain decomposition is therefore **derived information**, not extra entropy.

What the chain representation adds is a different organisation of that information: it groups adjacency relations by how long stable LF transport can continue before a run discontinuity is encountered.

Several quantities that might otherwise be lumped together are genuinely distinct:

\[
\text{number of chains}\;d,
\]

\[
\text{total chain length}
   = \sum_j \ell_j,
\]

\[
\text{ordered location of every vertex},
\]

\[
\text{sequence of run/character labels encountered},
\]

\[
\text{intersections with boundaries created by a later partition},
\]

and

\[
\text{cost of propagating the decoder's state through those vertices}.
\]

For the primitive one-cycle case, all \(n-1\) adjacency gaps lie in exactly one chain component. Consequently a literal representation of every chain vertex has total size \(\Theta(n)\), regardless of whether \(d\ll n\). Small \(d\) alone is therefore not a compressed representation.

The LCP interpretation adds useful semantic information. If a chain

\[
g_0\to g_1\to\cdots\to g_\ell
\]

has terminal LCP \(u\), then

\[
|\lambda(g_j)|=|\lambda(g_0)|+j,
\]

and \(u\) is the result of \(\ell\) successive left extensions of \(\lambda(g_0)\). Thus the endpoints plus the sequence of extension letters determine the factor labels along the path *symbolically*. But if one needs the actual gap positions \(g_j\), or if later decisions depend on which induced boundary each \(g_j\) crosses, the symbolic word alone does not recover those events for free.

## Behaviour under refinement and induction

There is a simple exact answer for **partition refinement**, and a substantially less rigid answer for genuine first-return induction.

### Newly derived refinement lemma

Suppose a continuity block of LF is split at an interior gap \(x\notin D\). Put

\[
y=\tau(x).
\]

The refined presentation declares \(x\) a new source boundary and \(y\) a new image boundary. On the gap graph this does exactly one thing:

\[
x\longrightarrow y
\]

is deleted.

Therefore refinement does not preserve the original chains. It cuts one existing chain at the chosen edge. The two resulting pieces may be nontrivial or may collapse to zero-step components depending on whether \(x\) or \(y\) was already an endpoint.

This already shows that \(d\) is not intrinsic to the LF permutation alone; it is intrinsic to **LF together with its chosen interval presentation**. Maximal BWT runs give a canonical presentation, but a finer move-table or induction partition changes the quantity.

This point matters because modern run-length compressed indexing deliberately refines the run partition. Nishimoto and Tabei's move structure splits intervals until images overlap only a constant number of source blocks, without blowing the number of blocks up by more than a constant factor; that refinement is what enables constant-time LF movement in \(O(r)\)-scale space. citeturn188606search0turn188606search6 Any chain statistic claimed to be a fundamental invariant must therefore specify whether it is measured on the original maximal-run presentation or on the refined one.

### First-return induction can split an original chain

Rauzy induction is a first-return construction: the induced transformation on a subinterval is

\[
T_J(x)=T^{q(x)}(x),
\]

where \(q(x)\) is the first return time. The return itinerary of each induced interval is encoded by products/substitutions in the usual IET formalism. citeturn730269search0turn531909search44

For a finite discrete LF permutation the same operation can be performed literally. It does **not** merely shorten existing gap chains.

A small primitive counterexample is

\[
w=\texttt{aaabaab},
\]

whose cyclic BWT is

\[
L=\texttt{bbaaaaa}
\]

with

\[
\operatorname{LF}=[5,6,0,1,2,3,4].
\]

The original maximal-run presentation has

\[
D=\{1\},\qquad I=\{4\},
\]

and exactly one nontrivial chain,

\[
4\to2\to0\to5\to3\to1.
\]

Now induce the LF permutation on the first six rows

\[
J=\{0,1,2,3,4,5\}.
\]

The first-return permutation, in \(J\)-coordinates, is

\[
T_J=[5,4,0,1,2,3].
\]

Its gap data are

\[
D_J=\{0,1\},\qquad I_J=\{3,4\},
\]

and it has two nontrivial interrupted chains,

\[
3\to1,
\qquad
4\to2\to0.
\]

Thus one original chain has become two under an entirely standard first-return operation. There is no one-chain-in/one-chain-out ancestry law.

The opposite phenomenon also occurs. For

\[
w=\texttt{aaabab},
\qquad
L=\texttt{babaaa},
\qquad
\operatorname{LF}=[4,0,5,1,2,3],
\]

the original gap system has two nontrivial chains,

\[
3\to1,\qquad4\to2,
\]

along with one trivial component. Inducing on rows \(\{0,1,2\}\) gives

\[
T_J=[2,0,1],
\]

whose only nontrivial chain is

\[
1\to0.
\]

So first return can also merge the *effective* gap structure seen by the induced presentation.

The exact ancestry statement that survives is weaker:

**An induced branch has an exact itinerary in the original LF permutation, and every induced discontinuity can be traced to either a boundary of the inducing interval or to an original discontinuity encountered by one of those itineraries.**

That is standard first-return logic and is the finite version of the Rauzy/return-word mechanism. citeturn730269search0turn652946search2 But an induced *gap chain* need not descend from one original gap chain. Composition of several LF steps changes which rows become adjacent in the induced system, and adjacency is precisely the datum on which the gap graph is built.

Consequently, the useful ancestry object under induction is not merely an original chain identifier. It is an **itinerary word or composed partial bijection** together with the induced adjacency relation.

This is also where the project's forest tie-breaking counterexample becomes decisive. Taking that counterexample as supplied, two forest schedules can have different counted operation totals while LF—and therefore the original maximal-run gap-chain system—is unchanged. Hence no schedule-independent scalar or multiset derived solely from the original gap chains can equal the forest count. At most it can bound, predict or correlate with it.

## Collective evaluation: what is possible exactly

The chain graph does admit an exact composition operation. The real question is whether that operation summarises the **decoder state**, not merely the motion of a gap.

For a chain

\[
g_0\to g_1\to\cdots\to g_\ell,
\]

define the endpoint map

\[
\eta(g_0)=g_\ell
\]

and, more generally,

\[
\tau^{[k]}(g_j)=g_{j+k}
\]

whenever the latter exists. These powers can be composed associatively:

\[
\tau^{[a+b]}=\tau^{[b]}\circ\tau^{[a]}.
\]

At this level there is no mathematical obstacle to binary lifting, pointer jumping or path contraction.

But this alone gives no serial work saving. There are \(n-1\) gap vertices in total. Building a jump table explicitly costs at least linear work in the number of represented vertices; standard pointer jumping reduces *parallel depth* to \(O(\log n)\) but not the total amount of path information that has to be materialised.

The only promising form of collective evaluation is more general. Suppose every elementary reconstruction event on an edge \(e\) acts on some decoder state \(s\) by

\[
s\mapsto \Phi_e(s).
\]

If the \(\Phi_e\) belong to a family having a bounded-size associative representation, then a whole chain segment can be replaced by

\[
\Phi_{e_k}\circ\cdots\circ\Phi_{e_1}.
\]

Examples of such summaries in other settings are matrices, affine maps, substitutions, semigroup elements and cocycle products. Rauzy induction itself uses matrix/substitution products to summarise return itineraries, and return-word derivations are naturally described by morphism composition. citeturn730269search0turn652946search2

For the LF gap problem, three conditions would have to hold simultaneously:

1. **Closure.** The decoder's relevant state transition across an event must remain inside a summary class of \(O(1)\) or \(O(\operatorname{polylog} n)\) size under composition.
2. **Cheap recognition.** One must locate maximal segments on which that summary applies without first enumerating every event in the segment.
3. **Cheap extraction.** The marked decoded output, or the boundary information needed for it, must be recoverable without expanding the composed summary back into essentially all of its constituent events.

The gap-chain mathematics establishes none of these three automatically.

The LCP/Weiner interpretation offers one concrete summary:

\[
\lambda(g_\ell)
   =c_\ell c_{\ell-1}\cdots c_1\,\lambda(g_0).
\]

So if the only question is “what factor label occurs at the terminal boundary?”, the entire path is exactly a left-extension product. That is a true collective operation.

But most reconstruction procedures need more than the terminal factor name. If they must know which intermediate boundary is crossed, update a dynamic ordering after each crossing, or emit symbols associated with intermediate states, the product does not remove that dependency.

This distinction is visible in existing BWT data structures. The LF permutation itself already has the highly compressible form “a permutation with \(r\) contiguously translated intervals”. Nishimoto–Tabei move structures exploit precisely that fact, and later work gives \(O(r)\)-space structures supporting constant-time LF movement after suitable balancing. citeturn188606search6turn188606academia5 Recent work goes further and reports \(O(r)\)-time/space construction of such structures and optimal-time BWT inversion and suffix-array enumeration. citeturn188606search3 Thus simply observing that many LF actions can be represented by one interval translation is already an exploited optimisation.

For a decoder that must materialise a length-\(n\) output string, there is moreover an unconditional output-size obstruction: writing \(n\) symbols costs \(\Omega(n)\) operations in any standard sequential RAM model. The only remaining asymptotic opportunity is to reduce auxiliary structural work or to produce a compressed representation rather than the literal text. The project's roughly \(1.8r\)-type structural counts live in that potentially improvable overhead, not in the unavoidable output term.

A plausible new direction is therefore **chain-local transducer minimisation** rather than raw path jumping. Associate to every original gap-chain edge the smallest state transformation actually needed by the reconstruction procedure. Then test whether long products collapse to bounded-size summaries. This is stronger than representing \(\tau^k\), because it asks whether the *semantic effect* of the events composes compactly.

At present I find no theorem implying such a collapse for arbitrary finite marked BWTs.

## Counterexamples and obstructions

The finite theory supplies several sharp obstructions.

**Chain count is not chain work.** One can have \(d=1\) and a chain of length \(\Theta(n)\). The family

\[
L=b\,a^{n-1}
\]

arising, for example, from the primitive necklace \(a^{n-1}b\), has one nontrivial chain running through essentially all gaps. Thus \(d=O(1)\) does not imply \(O(1)\) gap work.

**Endpoints do not determine the intermediate embedding.** Even when a chain's length and terminal maximal repeat are known, later induced partitions can cut the chain at different interior locations. Any algorithm whose decisions depend on those locations needs more than the endpoint summary.

**The maximal-run presentation is not refinement invariant.** Splitting one translation interval deletes one edge \(x\to\tau(x)\) from the chain graph. Hence the number and lengths of chains can change without changing the underlying LF permutation at all. Move-structure balancing deliberately performs exactly this kind of refinement. citeturn188606search0

**First return does not preserve chain ancestry.** The \(\texttt{aaabaab}\) example shows one chain becoming two; the \(\texttt{aaabab}\) example shows the converse effective coalescence. Therefore a later boundary cannot in general be labelled by one immutable “original chain ID”.

**Schedule-sensitive costs cannot be intrinsic chain invariants.** From the supplied forest tie-breaking counterexample, the same LF and Cells instance can produce different forest operation counts under different legal schedules. Since the gap-chain structure is fixed by LF and the original run presentation, no exact schedule-independent identity of the form

\[
\text{forest cost}=F(\text{original chains})
\]

can hold.

**Bispecial/maximal-repeat structure is richer than chain topology.** Two blocks may have the same number of chains and the same chain-length multiset but different run lengths, different LF permutations and different maximal-repeat labels. The chain topology alone therefore does not encode the relevant extension graph.

Extension graphs make this especially clear. A bispecial factor is not determined merely by the fact that it has at least two left and two right extensions; the incidence relation between those extensions matters. Work on interval-exchange languages distinguishes tree-like, neutral and more general extension graphs precisely because this incidence carries additional combinatorial information. citeturn652946search2 An LF chain records one sequence of successive left extensions of one adjacent pair; it does not in general encode the complete extension graph of the terminal maximal repeat.

Return-word substitutions are similarly conditional. For regular IET languages, induction and return words form a coherent substitutional structure. citeturn531909search44turn730269academia32 For an arbitrary finite BWT block, there is no theorem making the interrupted chains into return-word substitution paths. Treating them as such would impose recurrence and language structure that the finite data do not supply.

The strongest model-qualified negative statement I can justify is therefore:

**Proposition.** In any reconstruction model in which each intermediate event may alter a state used to decide the next event, and in which two event sequences with the same chain endpoints can induce different subsequent states, no endpoint-only chain summary is sufficient. Any correct summarisation must retain enough information to distinguish those state transformations.

This is almost tautological as a computational statement, but it is the correct obstruction. A meaningful lower bound must be stated on the *state-transition family*, not on the combinatorial existence of a path.

## The cheapest decisive structural test

The cheapest useful test is not another large decoder benchmark. It is a **summary-collision test on small primitive BWTs**.

Choose the candidate collective summary \(S(P)\) for a chain segment \(P\). For example it might contain

\[
(\text{start gap},\text{end gap},\ell,
\text{endpoint run labels},
\text{terminal LCP/maximal repeat}),
\]

or a proposed cocycle/substitution product.

Then exhaustively enumerate small primitive marked words, run the actual reconstruction rule on every chain segment, and record the semantic transition

\[
\Phi_P:
\text{incoming decoder state}\longmapsto
\text{outgoing decoder state}.
\]

Search for

\[
S(P)=S(Q)
\quad\text{but}\quad
\Phi_P\ne \Phi_Q.
\]

A single such collision disproves the proposed summary as a collective operation. Because primitive binary necklaces of lengths below roughly ten already exhibit chain splitting and merging under first return, this test can be extremely small.

If no collision appears, the second test is **closure under concatenation**:

\[
S(PQ)\stackrel{?}{=}S(P)\star S(Q)
\]

for an associative operation \(\star\) whose computation is cheaper than expanding \(P\) and \(Q\).

Only after those two algebraic tests pass is benchmarking warranted.

The third and decisive accounting test is a **materialisation audit**. Instrument the decoder after replacing maximal chain segments by summaries. Count separately:

\[
A=\text{number of summaries constructed},
\]

\[
B=\text{number of summary compositions},
\]

\[
M=\text{number of interior chain vertices later materialised},
\]

and

\[
O=n
\]

for literal output symbols.

A genuine structural saving requires something of the form

\[
A+B+M=o(\text{old structural-event count})
\]

on the target family, with the cost of locating applicable segments included. If \(M\) asymptotically returns to the old event count, the scheme has merely replaced a long derivation by a short notation and then expanded it again.

For the specific gap-chain hypothesis, an even cheaper diagnostic is available before implementing any semantics: measure how often induced steps cut original chains internally. If a substantial fraction of later boundaries lie at previously interior vertices, then endpoint-only aggregation is structurally implausible. If, conversely, induced boundaries occur overwhelmingly at a small reusable subset of original chain vertices, there is a realistic opportunity for jump summaries or memoised chain fragments.

The literature suggests one useful benchmark for what “real” aggregation should look like. Move structures do not merely describe LF compactly; after a controlled \(O(r)\)-size refinement they provably support LF movement in constant time, with the refinement itself constructed in compressed space. citeturn188606search6turn188606search2 A proposed LF-gap collective operation should be judged by the same standard: its recognition, construction, composition and output extraction costs must all be included.

The resulting assessment is therefore fairly sharp. The chains are not a numerical curiosity: they are exact finite discontinuity connections of the discrete LF interval exchange, and simultaneously explicit Weiner-link paths terminating at maximal-repeat boundaries. That is a substantive structural identification. They support exact path composition, LCP-depth composition and, conditionally, composition of any decoder transition that forms a compact semigroup or cocycle. What they do **not** presently provide is an invariant of induction, an exact predictor of schedule-sensitive reconstruction counts, or a proof that dependent boundary updates can be skipped. The mathematical opportunity lies in discovering a small transition algebra attached to chain segments; the mathematical obstruction is that induction changes adjacency and can split or merge the original connection structure. Until such an algebra is exhibited and passes the collision/materialisation tests above, the most defensible conclusion is that LF gap chains give a powerful **organisation of dependencies**, but not yet a cheaper computation.
