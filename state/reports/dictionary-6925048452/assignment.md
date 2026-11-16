FROM: coordinator
TO: researcher
TYPE: CHECKPOINT
RE: Proof check of "A mathematical dictionary for Cells" (a Research synthesis post), before it goes out

Louis wants this proof-checked by you. It is the project's own precise account of Cells, and we take responsibility for it. It does three things:
- maps our working vocabulary (letter, word, cell, rule, induction step, marked row, writer, stop) to exact objects;
- assesses three correspondences from Ghrist, *Elementary Applied Topology* (printed edition): discrete Morse theory, monoid-valued sheaves and circular coordinates;
- proves several statements and lists the empirical counts.

A blog PR is being drafted from it now. I'll send its number and head when it opens. **Review the mathematics on the text below now; don't wait for the PR.**

**What the Coordinator has already checked:**
- `abab` gives L = `bbaa` and LF = (0 2)(1 3);
- `(ab)^k a` gives two runs, of widths k and k+1;
- n = 899,981 gives S = 449,991.

**Please give:**
1. **Each P statement:** correct as stated, or the counterexample or fix. Above all:
   - (1) the orientation;
   - (3) the conjugacy of the suspension;
   - §1.4's update table;
   - (12) the correctness theorem and its width-one argument;
   - (13) the dual transform;
   - (14) the bounds;
   - (15) and (16), the batching identities;
   - (17) the itinerary certificates;
   - the position sheaf in §5.3, including whether "sections exist iff each cycle's increment vanishes mod the modulus" is exact;
   - the grammar sheaf in §4.
2. **Each C figure** checked against its record in the repo (the census; records 21, 23, 27 and 28 in docs/journal/monodromy on the branch of #497). Any figure you can't trace is a finding.
3. **The Ghrist claims:** whether each verdict is fair: Morse exact as an auxiliary model, the monoid sheaf "not established", circular coordinates exact mod d. Quote the book at most a formula or a sentence (fair use).
4. **Is a sheaf practical?** The text says a genuine sheaf of positions or of grammar evaluations can be built, and that doing so is natural. Does building one plausibly buy anything for the decoder (H), or is it only a formal encoding? A short, reasoned view is enough.
5. **A verdict:** PROCEED, REVISE (with the findings) or STOP.

No decoder timing and no merge. The text follows, verbatim.

---

Cells is best described as a finite interval-exchange induction that maintains labelled return towers, compiles their return words into a straight-line program, and evaluates that program from a distinguished occurrence.

The three Ghrist correspondences have different statuses:

| Claimed correspondence | Assessment |
|---|---|
| Discrete Morse theory | An exact model can be constructed on the LF orbit graph. An implementation "cell" batches several vertex–edge eliminations. Word labels are additional structure beyond ordinary Morse homology. |
| Monoid-valued sheaves | Not established by the description in the records. Words compose in a free monoid, but that does not specify a sheaf's stalks and restriction maps. A path-category functor is the immediate exact formulation. |
| Circular coordinates | Exact after specifying the coefficients, orientation, and cycle length. The invariant modulo the block length $n$ needs a single-cycle assumption, or a distinction between rows and repeated output occurrences. |

I use P for mathematical statements proved under explicit hypotheses, C/M for counts or measurements reported in the project, and H for hypotheses about algorithmic benefit. I have not rerun the benchmarks.

Ghrist page references below use the printed pages of your supplied edition: EAT.pdf. For the repository, I inspected the construction and companion at commit 4599b763, and the later records 22–29 at the cited head of research PR #497, 25481a03.

## 1. The mathematical algorithm

### 1.1 The initial object: a permutation presented as an interval exchange

Let

$$X=\{0,\ldots,n-1\},\qquad L\in\Sigma^n,$$

where $\Sigma$ is the byte alphabet, $L$ is the BWT last column, and $r_*\in X$ is orig_ptr.

Define

$$C(c)=\#\{i:L[i]<c\},$$

and the LF permutation

$$\pi(x)=C(L[x])+\#\{i<x:L[i]=L[x]\}.$$

The output required by the reference inversion convention is

$$O[k]=L\!\left[\pi^{-(k+1)}(r_*)\right],\qquad 0\le k<n.$$

These are the inverse-BWT bytes, before the subsequent RLE1 decoding.

On a maximal constant-byte run $I_\alpha$, LF is a translation:

$$\pi(x)=x+\delta_\alpha,\qquad x\in I_\alpha.$$

Its image $J_\alpha=\pi(I_\alpha)$ is another integer interval of the same width. Thus LF has a discrete interval-exchange presentation:

* the domain intervals in their order in $L$;
* their image intervals in first-column order;
* their widths.

The same formula extends to a real interval exchange on $[0,n)$, but all lengths and translations here are integers. Cells uses an unnormalized, finite version of Rauzy induction, including equality and component-closing cases. Generic results about minimal irrational interval exchanges do not automatically apply. (Construction, §§1–2)

### 1.2 The maintained object: a labelled first-return map

At an intermediate stage, the retained section is

$$D_m=\{0,\ldots,m-1\}.$$

The active permutation is the first-return map

$$T_m(x)=\pi^{h_m(x)}(x),\qquad h_m(x)=\min\{j>0:\pi^j(x)\in D_m\}.$$

The section is partitioned into intervals $I_\alpha$ on which the return itinerary is constant. Each interval has:

$$\lambda_\alpha=|I_\alpha|,\qquad J_\alpha=T_m(I_\alpha),\qquad h_\alpha=h_m|_{I_\alpha},$$

and a byte word $w_\alpha$.

The crucial orientation convention is

$$\boxed{L[x]\,L[\pi x]\cdots L[\pi^{h_\alpha-1}x]=\operatorname{rev}(w_\alpha)\quad(x\in I_\alpha).}\tag{1}$$

Thus the stored word is the reverse of the LF byte itinerary, so that it is already in forward text order.

Two different quantities must remain separate:

$$\lambda_\alpha=\text{number of base rows},\qquad h_\alpha=|w_\alpha|=\text{number of original LF steps per return}.$$

A one-row interval can carry a very long word. Two intervals can carry identical words while having different destinations. This is precisely the construction's invariant I. (Technical companion, §§2.2–2.3)

### 1.3 The tower interpretation is exact

For each interval, consider its levels

$$I_\alpha,\ \pi(I_\alpha),\ \ldots,\ \pi^{h_\alpha-1}(I_\alpha).$$

These form a finite return tower, also called a Kakutani–Rokhlin tower. Its base width is $\lambda_\alpha$; its height is $h_\alpha$.

**P.** All these tower levels partition the saturation of the retained section:

$$X_m=\bigcup_{j\in\mathbb Z}\pi^j(D_m).$$

Every point in $X_m$ has a unique most recent visit to $D_m$ before reaching that point. That visit identifies its base point and its tower level. Consequently,

$$\boxed{\sum_\alpha \lambda_\alpha h_\alpha=|X_m|.}\tag{2}$$

For one original $n$-cycle, this is always $n$. If entire unmarked components have been discarded, the right-hand side is smaller.

This gives the precise sense in which the original dynamics are retained. Define the discrete suspension

$$\widehat X_m=\{(x,j):x\in D_m,\ 0\le j<h_m(x)\},$$

with dynamics

$$\widehat T_m(x,j)=\begin{cases}(x,j+1),&j+1<h_m(x),\\(T_m(x),0),&j+1=h_m(x).\end{cases}$$

Then

$$\eta(x,j)=\pi^j(x)$$

is a bijection onto $X_m$, and

$$\eta\circ\widehat T_m=\pi\circ\eta.\tag{3}$$

The expanded tower system is conjugate to the original permutation on its saturation. The smaller base map $T_m$ itself is an induced system: it has fewer states and uses variable return times.

This corrects record 28's description of induction as a conjugacy to the larger map. Its intended statement becomes exact after adding the tower levels. (Record 28, §3)

### 1.4 One elementary step

Let $A$ be the last domain interval and $B$ the last image interval, with $A\ne B$. Put

$$s=\min(\lambda_A,\lambda_B).$$

Their terminal overlap is

$$Q=I_A\cap J_B=[m-s,m).$$

The new section is $D_{m-s}$. For

$$K=T_m^{-1}(Q)\subseteq I_B,$$

the old trajectory has the form

$$x\in K\ \xrightarrow{B}\ T_m(x)\in Q\ \xrightarrow{A}\ T_m^2(x).$$

Therefore the new return branch is

$$T_{m-s}|_K=T_m^2|_K,$$

with width $s$, height $h_B+h_A$, and stored word

$$\boxed{w_{\mathrm{new}}=w_Aw_B.}\tag{4}$$

The LF itinerary is $B,A$; the stored forward-text word is $w_Aw_B$.

The combinatorial updates are:

| Case | Width update | Word update | Order update |
|---|---|---|---|
| $\lambda_A>\lambda_B$ | Shorten $A$ by $\lambda_B$ | $w_B\leftarrow w_Aw_B$ | Move $B$ immediately after $A$ in image order |
| $\lambda_A<\lambda_B$ | Shorten $B$ by $\lambda_A$ | $w_A\leftarrow w_Aw_B$ | Move $A$ immediately after $B$ in domain order |
| $\lambda_A=\lambda_B$ | Delete $A$ | $w_B\leftarrow w_Aw_B$ | $B$ takes $A$'s image-order position |

These follow directly from the two old returns crossing the removed suffix. When $A=B$, a different event occurs: $I_A=J_A$, so this branch is the identity and its return tower closes. (Construction, §2)

## 2. A precise dictionary

| Informal term | Mathematical object | Ambiguity or correction |
|---|---|---|
| Letter | A label $\alpha$ indexing a base interval $I_\alpha$, its image $J_\alpha$, and associated return data | Initially a run interval, possibly refined to isolate the marked row. It is neither a byte nor a word value. |
| Word | An element $w_\alpha\in\Sigma^*$, the forward-text label of a return excursion | Distinguish the string value, its grammar node, and an occurrence of that node. These are three different objects. |
| Cell | Depending on context: the terminal overlap $Q$, the new return branch $K\to T_m(Q)$, or the immutable record representing that branch | It is not a CW cell. Calling an induction step "a cell" is implementation terminology. |
| Rule | A production $N\to UV$ in a binary straight-line program, or equivalently a node of a concatenation DAG | A pair of child IDs remains a rule even if the implementation calls it a compact "join." |
| Induction step | Replacement of $T_D$ by its first return to a smaller section $D'$, with composed return labels | The unequal cases are Rauzy moves. Equal-length reductions and closed-component handling complete the finite algorithm. |
| Marked row | A distinguished original state $r_*$, transported as a tower coordinate or a distinguished terminal occurrence in the grammar | Its integer row index is not its output position. It need not remain the base point of its current tower. |
| Writer | Evaluation of an SLP, together with occurrence placement, materialization/copy scheduling, and cyclic phase selection | The grammar's correctness and the writer's correctness are separate obligations. |
| Stop | A stopping index in the induction history, selecting an intermediate section and its labelled return map | An early budget stop differs from natural cycle closure and from budget refusal followed by fallback. It is not a critical cell. |

The implementation uses "cell" because a completed return piece is written into records for the two orders, while the surviving remainder stays at the fronts. This is a representation of the induction above. (Technical companion, §4)

### Substitutions, cylinders, and grammar nodes

There are three useful additional descriptions.

**Finite S-adic presentation.** Each stage defines a substitution from the new interval alphabet into the preceding alphabet. Their composition gives a finite directive sequence of return-word substitutions. The final stored object is most directly called an SLP. Versioning matters: an assignment such as $w_B\leftarrow w_Aw_B$ creates a new node referring to the old values, so the grammar is acyclic.

**Cylinder sets.** A base interval lies within a cylinder of constant return itinerary. It need not be the maximal cylinder for its byte word. Marker isolation deliberately refines intervals, and equal byte words need not identify equal state itineraries.

**Occurrence coordinates.** A grammar node can occur many times in the output. The writer's "first place" is the minimum position of one of its occurrences. It is not a position intrinsically attached to the string value.

For an unrotated root expansion, the first-place calculation is an acyclic min-plus computation. If $N\to UV$,

$$P(U)\leftarrow\min(P(U),P(N)),\qquad P(V)\leftarrow\min(P(V),P(N)+|U|).\tag{5}$$

This explains the reverse rule-order pass. The subsequent pass materializes factors and copies repeated occurrences. The marked-path handling adds the required output phase. (Construction, §§3.2, 3.5)

## 3. Discrete Morse theory: an exact auxiliary model

### What Ghrist states

In EAT §8.8, pp. 205–206, a discrete vector field pairs cells $(\sigma,\tau)$, with $\sigma$ a codimension-one face of $\tau$. Unpaired cells are critical. The field is a gradient when it has no periodic alternating flowline.

The discrete flow is

$$\Phi=\operatorname{Id}+\partial V+V\partial,$$

and Theorem 8.24, p. 206, identifies the resulting Morse homology with the original cellular homology.

These are constructions on chain modules, with signed or mod-two counts of paths.

### The exact construction for LF

Construct the orbit graph $G_\pi$:

* a vertex for every row $x$;
* an oriented edge $e_x:x\to\pi(x)$.

Every component is a circle. This complex is constructed from the dynamics; it is not the coordinate interval $[0,n)$. Subdivide one-point loops when necessary to satisfy Ghrist's regular-cell-complex hypothesis.

Choose a section $D$ meeting every component under consideration. Define

$$\boxed{V=\{(y,e_y):y\notin D\}.}\tag{6}$$

That pairs each removed vertex with its outgoing edge.

**P: acyclicity.** An alternating path follows LF through removed vertices. It must eventually meet $D$, where it stops. Hence it cannot be periodic.

The correspondence is now explicit:

| Discrete Morse object | LF interpretation |
|---|---|
| Original 0-cell | A row |
| Original 1-cell | One LF transition |
| Matched pair | A removed row paired with its outgoing transition |
| Critical 0-cell | A retained row $x\in D$ |
| Critical 1-cell | The original outgoing edge $e_x$ of a retained row |
| Gradient continuation of $e_x$ | The excursion from $x$ to its first return $T_D(x)$ |
| Morse boundary | $\widetilde\partial e_x=T_D(x)-x$ |

The resulting Morse boundary is exactly the incidence boundary of the first-return orbit graph.

An elementary Cells step of width $s$ performs $s$ such row-level eliminations together. This batching is crucial: explicitly constructing the full matching would throw away the interval representation's advantage.

### What corresponds to a recorded word?

A word is the byte coding of the associated return path, with the orientation reversal in (1).

That is weaker than recording the path itself. Different paths can emit the same string. A shared SLP node can represent multiple path occurrences. The interval geometry and marker retain distinctions that the string alone does not.

It is therefore legitimate to speak of Morse return paths decorated with word labels. The phrase "Morse theory with free-monoid coefficients" is not justified by the cited theorem. Its chain maps use addition and incidence coefficients; the free monoid of strings is not a chain module. Ordered labels are an enrichment that requires its own preservation argument.

There are two further qualifications:

* Suppressing degree-two vertices of a circle is a graph contraction/Morse reduction. It is not literally a succession of free-face collapses inside the original circle, whose vertices have two incident edges.
* Discarding a closed unmarked LF component removes an entire circle. That does not preserve the homology of the full original orbit graph. The Morse statement applies componentwise to the components still represented.

### What does the theorem add?

It gives a valid homology-preservation statement and forces at least one critical vertex and one critical edge per circle. A circle cannot be reduced to a point while preserving its homology.

It does not establish byte correctness, output phase, grammar size, or an optimal contraction schedule. Those depend on the labelled return invariant.

Thus the mathematician's formal match can be repaired into an exact construction, but the correspondence is between row-level return-path contraction and a Morse matching, not between the implementation's records and topological cells. The stronger wording in LOG 6924300674 omits these distinctions.

## 4. Monoid-valued sheaves: the proposed identification is incomplete

### What a sheaf here would require

In EAT §11.9, p. 286, a cellular sheaf is a functor

$$\mathcal F:\operatorname{Face}(X)\longrightarrow\mathcal C.$$

For $\mathcal C=\mathbf{Mon}$, this requires:

1. a monoid at every cell;
2. a monoid homomorphism for every face incidence;
3. compatibility of those homomorphisms.

Equation (11.3), p. 286, describes global sections as the limit of that diagram.

A stored word is an element of a monoid. It is not itself a stalk. Neither LOG 6924126823 nor the follow-up supplies the required stalks and incidence maps for the claimed word sheaf.

Simply assigning $\Sigma^*$ everywhere with identity restriction maps would force equal words on adjacent cells. It would not concatenate them.

Nor can multiplication automatically serve as a restriction map. For noncommutative strings,

$$\mu:\Sigma^*\times\Sigma^*\to\Sigma^*,\qquad\mu(u,v)=uv$$

is not a homomorphism for the product monoid:

$$\mu((a,b)(c,d))=acbd,\qquad\mu(a,b)\mu(c,d)=abcd.\tag{7}$$

The distinction matters precisely because order matters.

### The immediate exact categorical object

Use the inverse-LF orientation. Label the induced edge

$$T_D(x)\longrightarrow x$$

by $w_{\alpha(x)}$. Along paths, these labels concatenate in forward output order.

This is a functor

$$\boxed{\mathsf{Path}(G_{T_D}^{-1})\longrightarrow B\Sigma^*,}\tag{8}$$

where $B\Sigma^*$ is the one-object category whose morphisms are words and whose composition is concatenation.

This uses a monoid as a one-object category. A sheaf valued in the category of monoids is a different construction.

The SLP stores compressed expressions for these path labels. Length is then a monoid homomorphism

$$|\cdot|:\Sigma^*\longrightarrow(\mathbb N,+).$$

That accounts exactly for ordered word composition and additive return times.

### A genuine sheaf can be added

For the grammar itself, one can construct a sheaf of sets on a bipartite incidence graph:

* a symbol vertex has stalk $\Sigma^*$;
* a production vertex for $N\to UV$ has stalk $\{(u,v,w):w=uv\}$;
* incidence edges have stalk $\Sigma^*$;
* the production-to-edge maps are the appropriate projections;
* the symbol-to-edge maps are identities;
* terminal values are fixed.

Its global sections are consistent evaluations of the grammar. This is an exact encoding, but it is additional formalization, and it does not turn concatenation into a monoid homomorphism.

There is also a particularly natural sheaf of positions, described below.

**Verdict:** the monoid observation is sound and useful. "Words are free-monoid sections" does not follow without specifying a sheaf. Ghrist's section licenses such definitions; it does not supply them or a faster evaluation algorithm.

## 5. Circular coordinates: exact, with an essential modulus correction

### 5.1 The cocycle and its period

Restrict attention to the selected original LF cycle $C$, of length $d$. The induced cycle on $D\cap C$ has return lengths $h(x)$.

**P.** Its return excursions partition $C$, so

$$\sum_{x\in D\cap C}h(x)=d.\tag{9}$$

On the induced graph, orient edges in the LF direction and define

$$a(e_x)=-h(x),\qquad e_x:x\to T_D(x).$$

This is an integer 1-cocycle. On a graph, every 1-cochain is a cocycle because there are no 2-cells. The substantive information is its period:

$$\langle a,[C_D]\rangle=-d.\tag{10}$$

Consequently:

* $a$ is not the coboundary of an integer- or real-valued position function on the closed cycle;
* modulo $d$, it is exact:
  $$\boxed{p(T_D(x))-p(x)\equiv-h(x)\pmod d;}\tag{11}$$
* $a/d$ is a real cocycle with integral period $-1$, so it integrates to a coordinate in $\mathbb R/\mathbb Z$;
* a marked occurrence fixes the additive phase.

This is the exact connection to EAT §7.13, pp. 185–186, especially the correspondence between integral first cohomology and circle-valued maps on p. 186.

It is important to retain the edge increments as well as vertex coordinates. At final closure the induced graph may have one vertex and one loop: its vertex coordinate alone says nothing about the loop's winding.

### 5.2 Why modulo $n$ is not always correct

The invariant quoted in your question,

$$p(Tx)\equiv p(x)-|w(x)|\pmod n,$$

is correct for a single $n$-cycle.

For a selected cycle of length $d<n$, summing it around the cycle would give

$$0\equiv-d\pmod n,$$

which is impossible.

A concrete valid periodic example is

$$\text{text}=\texttt{abab},\qquad L=\texttt{bbaa}.$$

Its LF permutation is

$$\pi=(0\ 2)(1\ 3).$$

There is no row-valued $p:X\to\mathbb Z/4\mathbb Z$ satisfying $p(\pi x)=p(x)-1$: either two-cycle would require $0\equiv-2\pmod4$.

The precise options are:

1. positions modulo the selected period $d$;
2. positions on an occurrence cover distinguishing successive visits to the same physical row;
3. an explicit single-$n$-cycle hypothesis.

For valid periodic BWT input, the selected cycle is repeated to produce $n$ bytes. For arbitrary columns, the reference semantics similarly repeat and truncate the selected cycle.

The original construction handles this. The earlier stop-and-walk proof explicitly assumes one $n$-cycle; its later shorthand should retain that hypothesis. (Construction, §2; Stop-and-walk, §2)

### 5.3 The exact position sheaf

Here is a concrete realization of "positions are sections of length data."

For each induced edge $e:x\to y$, put $\mathbb Z/d\mathbb Z$ at both endpoint vertices and at the edge. Define the endpoint maps

$$\rho_{x,e}(u)=u-h_e,\qquad\rho_{y,e}(v)=v.$$

A global section satisfies

$$\rho_{x,e}(p(x))=\rho_{y,e}(p(y)),$$

which is exactly (11).

These are maps of sets, or affine maps of torsors. They are generally not group or monoid homomorphisms, since translation by a nonzero constant does not preserve the identity.

Sections exist precisely when the total increment around each cycle vanishes modulo the chosen modulus. One anchor fixes the remaining additive freedom on each component. This makes the compatibility claim exact using the stalks and restriction maps that were missing.

### 5.4 What Example 7.37 contributes

Example 7.37, p. 185, occurs immediately before the §7.13 heading. It uses the discrete Hodge decomposition from Theorem 7.34, p. 184, to separate pairwise comparison data into a gradient ranking and cyclic inconsistencies.

For Cells, its relevant lesson is the obstruction to a real-valued ranking on a closed cycle with nonzero circulation.

The harmonic smoothing in §7.13 has no evident role in exact decoding. Replacing the length cocycle by a cohomologous harmonic representative preserves winding, but generally changes the positions assigned to intermediate vertices. Decoding needs the actual return lengths, not merely their cohomology class.

Finally, the row-coordinate function $p$ and the writer's first-occurrence function $P$ in (5) are different objects. The former integrates lengths along an orbit; the latter chooses canonical occurrences in a grammar DAG.

## 6. What the precise formulation usefully proves

### 6.1 A compact correctness theorem

The following gives a clean proof obligation for Cells.

**P.** Suppose an implementation maintains:

1. the first-return map on the retained section;
2. the exact return-word invariant (1);
3. a distinguished occurrence of $r_*$ in the return towers;
4. an SLP whose evaluation equals each stored return word.

Then closure of the marked tower determines the reference inverse-BWT output.

*Proof.* Isolate $r_*$ initially as a singleton base interval. The marked base remains width one: a width-one interval cannot strictly win against another positive integer width, and equality preserves width one.

When its branch closes, its return word $E$ describes exactly the selected LF cycle. Let $d=|E|$, and let $j$ be the position of the marked terminal in $E$. Since the reference walker steps through inverse LF before emitting, its output is

$$\boxed{O[k]=E[(j+1+k)\bmod d],\qquad 0\le k<n.}\tag{12}$$

Equation (1) gives the bytes; the marker gives the phase; repetition gives the requested length. ∎

This is stronger than a homology argument because it preserves the observable and the distinguished occurrence.

The writer still needs a separate proof that its scheduled copies actually realize this string. The recorded marked-spine bug illustrates the distinction: the grammar could be correct while some output positions remained unwritten. (Technical companion, §3)

### 6.2 Widths and heights transform dually

For a top win, let $M=I+E_{AB}$, with rows indexed by old intervals and columns by new intervals. Then

$$\lambda_{\mathrm{old}}=M\lambda_{\mathrm{new}},\qquad h_{\mathrm{new}}=M^{\mathsf T}h_{\mathrm{old}}.\tag{13}$$

The first equation undoes the subtraction of widths. The second adds return times. Therefore

$$\lambda_{\mathrm{new}}^{\mathsf T}h_{\mathrm{new}}=\lambda_{\mathrm{old}}^{\mathsf T}h_{\mathrm{old}}.$$

This is a useful algebraic check on every update. Equal-length deletion uses the corresponding rectangular incidence matrix. It expresses tower-volume conservation without following individual rows.

### 6.3 Bounds on the present elementary schedule

Let $r_0$ be the number of initial intervals after isolating the marked row; $r_0\le r+2$. Let $S$ count ordinary concatenating induction steps.

For one $n$-cycle:

$$\boxed{r_0-1\le S\le n-1.}\tag{14}$$

**P.** Each step removes at least one retained row, giving the upper bound. Only equality steps delete an interval, and complete reduction ends with one interval, giving exactly $r_0-1$ equality steps.

The lower bound needs modification when unmarked components are discarded or the selected component closes before other components.

There is no general $O(r\,\mathrm{polylog}\,n)$ bound for the present subtraction schedule. For the unrefined primitive text

$$T=(ab)^k a,\qquad n=2k+1,$$

the BWT has two runs, of widths $k$ and $k+1$, but the algorithm takes exactly

$$S=k+1=\frac{n+1}{2}.$$

After the first subtraction, the widths are $k$ and $1$; repeated unit subtractions finish the reduction. The journal's $449{,}991$ steps at $n=899{,}981$ match this formula. (Construction, §2 and §4)

This family also explains why long words must remain compressed: successive return words grow linearly, so storing all of them literally can require quadratic total bytes.

### 6.4 Some batching identities are proved; their speed benefit is not

For $q$ consecutive strict top wins by the same pair,

$$\lambda_A'=\lambda_A-q\lambda_B,\qquad w_B'=w_A^q w_B.\tag{15}$$

For consecutive strict bottom wins,

$$\lambda_B'=\lambda_B-q\lambda_A,\qquad w_A'=w_Aw_B^q.\tag{16}$$

These are exact identities. Binary powering can represent $w^q$ with $O(\log q)$ additional SLP rules. Order changes, equality boundaries, and marker transport must also be composed correctly.

**H:** whether such acceleration improves the actual decoder depends on how often suitable streaks occur and what their detection costs. The identities already show that topology does not force one operation per elementary subtraction.

### 6.5 Exact itinerary certificates

Suppose a piecewise translation $F$ follows intervals

$$I_{i_0},\ldots,I_{i_{k-1}},$$

with cumulative translations $s_0=0,s_1,\ldots,s_k$. Then

$$\boxed{J=\bigcap_{j=0}^{k-1}(I_{i_j}-s_j)}\tag{17}$$

is exactly the set of starts following that itinerary.

**P.** For $x\in J$, the $j$-th state is $x+s_j\in I_{i_j}$, by induction. Every such start emits the same sequence of words and finishes at $x+s_k$.

This is a deterministic certificate for reusing an excursion. It needs no hashing.

A later induced interval is contained in the appropriate certificate. It need not equal the maximal certificate. Record 28's displayed lemma states containment; its stronger language about intervals being "exactly" the columns needs that qualification. Also, a composite map being a translation does not alone prove a constant itinerary—different itineraries can have the same net translation. The maintained return partition supplies the missing premise. (Records 26–28)

### 6.6 What lower bounds are actually available

Explicitly producing $n$ bytes requires $\Omega(n)$ byte traffic. In a byte-cost model, that gives an $\Omega(n)$ output bound.

It does not require $n$ dependent LF loads, $n$ grammar rules, or a serial ranking walk. Similarly, explicitly producing a position for every retained row requires space and work for those values; an algorithm that avoids that representation is a different problem.

The records' memory-retention arguments say that unplaced occurrences need recoverable expansion information. They do not prove that this information must occupy the existing rule and arena layout. Re-encoding, recomputation, alternative grammars, and different placement strategies remain possible. Record 27's compact tail is a concrete example: it preserves the same grammar while changing its representation. (Record 18, §4; Record 27, §1)

## 7. What the records establish empirically

These are reported observations, separate from the proofs above:

| Status | Recorded result | Scope |
|---|---|---|
| C | $27{,}439{,}723$ elementary steps, $S/r=1.812$, $S/n=0.1309$ | Original right-end census on 233 Wikidata blocks; not an asymptotic law |
| C | At a $1.5r$-cell stop, about $33\%$, or $5.0$ bytes per original run, of word storage avoided | Reopened stop design; prototype emitter counts are separate from complete decoder timing |
| C | Compact tail at $1.2r$: $4.89$ bytes per original run saved, about $32\%$ of word storage | All 233 Wikidata blocks, with wider IDs where required; same mathematical grammar |
| C | At $\beta=1.5$, about one third of starts still have another start sharing their certificate after 256 emitted bytes | Every eighth Wikidata block; does not establish that all this reuse is profitably realizable |

Sources: original census, record 21, record 27, record 28.

One vocabulary correction materially affects those figures: "per letter" usually means per original run $r$. It does not mean per live interval. Record 23 gives approximately

$$m=0.672r,\qquad q_{\mathrm{live}}=0.1212r,$$

so there are about $5.54$ retained points per live interval, not $0.672$. (Record 23)

Likewise, "the stop is free" means the existing budget check can be reused. Snapshot construction and emission still cost work. "One lane is needed" is an engineering judgment, not a mathematical consequence of the point count. The early prototype also leaves the original-row phase shift unexercised or uncounted, so its byte comparisons should not be promoted into a proof of every origin and periodic-input case. (Stop-and-walk, §3; reopened design, §3)

The strongest warranted conclusion is therefore that labelled first-return towers and SLP evaluation give Cells its correctness proof; discrete Morse theory provides an exact auxiliary reduction model; and circular cohomology precisely describes phase compatibility once the cycle length is correct. The monoid-sheaf claim needs additional construction.

None of the cited Ghrist sections establishes decoder optimality. A claim that continuing induction is the only cheap way to recover positions would require a specified representation and cost model, followed by a lower-bound argument. The records provide useful design comparisons and counts, but no such impossibility theorem.


---
_Generated by [Claude Code](https://claude.ai/code)_
