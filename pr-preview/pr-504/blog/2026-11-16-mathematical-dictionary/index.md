[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/index.md>)

# Towers, words and loops: a mathematical dictionary for BWT inversion

Our BWT inversion is not the sheaf an earlier analogy suggested, yet the places of its words form one naturally.

16 November 2026 · 17 min read · [Research synthesis](<#8-where-this-leaves-us>)

mibz2, our bzip2 decoder, inverts each block's Burrows–Wheeler transform with a kernel we call **Cells**. Over the past week our notes have described it in a working vocabulary of letters, words, cells, rules, steps, a marked row, a writer and a stop. They have also borrowed three pictures from Robert Ghrist's *Elementary Applied Topology*: discrete Morse theory, sheaves of monoids, and circular coordinates. This post says exactly which mathematical object each of our words names, and how far each borrowed picture holds.

**The setting, in brief.** bzip2 stores a block as the last column \\(L\\) of its sorted rotations, the Burrows–Wheeler transform (BWT), together with the row that holds the original text. A map called **LF** sends each row to the row of the rotation one byte earlier. Following LF from that row reads the block backwards, one byte per step. A **run** is a stretch of equal bytes in \\(L\\), and LF shifts all of a run's rows by the same amount. So LF is an **interval exchange**: a handful of intervals, each moved whole to a new place. [The mathematical structure of inverse-BWT navigation](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-08-inverse-bwt-navigation/index.md>) sets out that structure. [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) shows Cells shrinking the exchange step by step on a three-byte block, and the [technical companion](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-12-rauzy-technical-companion/index.md>) describes its memory layout.

**The simplest correct picture.** Cells shrinks LF's interval exchange from its right-hand end. At each stage it keeps a smaller set of rows. For each interval of kept rows it holds a **return tower**: the rows the LF path passes through before it comes back to a kept row. A tower has a **width**, the number of kept rows at its base, and a **height**, the number of LF steps from its base back to a kept row. It also has a **word**, the bytes those steps read, stored in forward text order. The towers' volumes, width times height, always add up to the block's rows. When the tower that holds bzip2's starting row closes on itself, its word is the block's text, and the starting row says where the text begins in it.

**What we found.**

- **Each of our working words names one exact object.** The [dictionary in section 2](<#2-what-do-our-words-name>) gives them. Two need care: a *letter* labels an interval, and is neither a byte nor a word, and "per letter" in our figures means per original run, not per interval still alive.
- **Discrete Morse theory is an exact auxiliary model.** It applies to the graph of LF's orbits, not to our memory records, and one step of Cells performs many of its row-level moves at once ([section 3](<#3-is-the-induction-discrete-morse-theory>)).
- **Cells is not a sheaf in the sense an earlier note claimed, but it is natural to make one.** Our notes said the words are sections of a sheaf of monoids, without giving the sheaf. The exact object is simpler: a functor from LF's paths to words. Two genuine sheaves can then be built on top: one whose sections are the grammar's consistent evaluations ([section 4](<#4-are-words-sections-of-a-sheaf>)), and one whose sections are exactly the places of the words in the text ([section 5](<#5-are-word-places-circular-coordinates>)). Whether either buys anything in the decoder is an open question, not a claim.
- **Circular coordinates are exact, with one correction.** Word places are consistent modulo the length \\(d\\) of the cycle being decoded. Modulo the block length \\(n\\) they are consistent only when LF is a single cycle through all \\(n\\) rows.
- **The formal account proves useful things** ([section 6](<#6-what-does-the-formal-account-prove>)): a correctness theorem for Cells, an algebraic check on every step, bounds on the number of steps, exact identities for batching steps, and exact certificates for reusing a path.

We label statements as the research records do. **P** marks a statement proved here under the hypotheses given. **C** marks a count reported in the project's records. **H** marks a hypothesis about what would help the decoder. We did not rerun the benchmarks for this post. We did check every formula in it on small blocks against a direct inverse BWT, in a short script.

[Section 1](<#1-what-does-cells-maintain>) builds the towers and the step that shrinks them. [Section 2](<#2-what-do-our-words-name>) is the dictionary. Sections [3](<#3-is-the-induction-discrete-morse-theory>), [4](<#4-are-words-sections-of-a-sheaf>) and [5](<#5-are-word-places-circular-coordinates>) take Ghrist's three pictures in turn. [Section 6](<#6-what-does-the-formal-account-prove>) gives what the formulation proves, [section 7](<#7-what-do-the-records-count>) what the records count, and [section 8](<#8-where-this-leaves-us>) where it leaves us.

## 1\. What does Cells maintain?

### 1.1 The starting object: a permutation presented as an interval exchange

Let

\\\[X=\\{0,\\ldots,n-1\\},\\qquad L\\in\\Sigma^n,\\\]

where \\(\\Sigma\\) is the byte alphabet, \\(L\\) is the BWT's last column, and \\(r\_\*\\in X\\) is bzip2's starting row (`orig_ptr` in its source). Define

\\\[C(c)=\\\#\\{i:L\[i\]\<c\\},\\\]

and the LF permutation

\\\[\\pi(x)=C(L\[x\])+\\\#\\{i\<x:L\[i\]=L\[x\]\\}.\\\]

The output the reference inversion produces is

\\\[O\[k\]=L\\!\\left\[\\pi^{-(k+1)}(r\_\*)\\right\],\\qquad 0\\le k\<n.\\\]

These are the inverse-BWT bytes, before bzip2's later run-length decoding.

On a maximal run of one byte, \\(I\_\\alpha\\), LF is a translation:

\\\[\\pi(x)=x+\\delta\_\\alpha,\\qquad x\\in I\_\\alpha.\\\]

Its image \\(J\_\\alpha=\\pi(I\_\\alpha)\\) is another integer interval of the same width. So LF has a discrete interval-exchange presentation: the domain intervals in their order in \\(L\\), their image intervals in first-column order, and their widths.

The same formula extends to a real interval exchange on \\(\[0,n)\\), but here every length and translation is an integer. Cells uses an unnormalised, finite version of Rauzy induction, with cases for equal lengths and for closing a component. General results about minimal interval exchanges with irrational lengths do not automatically apply. The construction and its proof are in [the construction's research journal](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-10-rauzy-veech.md>), §§1–2.

### 1.2 What is kept: a labelled first-return map

At an intermediate stage the kept rows are

\\\[D\_m=\\{0,\\ldots,m-1\\}.\\\]

The active permutation is the first-return map

\\\[T\_m(x)=\\pi^{h\_m(x)}(x),\\qquad h\_m(x)=\\min\\{j\>0:\\pi^j(x)\\in D\_m\\}.\\\]

The kept rows are cut into intervals \\(I\_\\alpha\\) on which the return path is the same. Each interval has

\\\[\\lambda\_\\alpha=|I\_\\alpha|,\\qquad J\_\\alpha=T\_m(I\_\\alpha),\\qquad h\_\\alpha=h\_m|\_{I\_\\alpha},\\\]

and a byte word \\(w\_\\alpha\\). The orientation convention is the crucial one:

\\\[\\boxed{L\[x\]\\,L\[\\pi x\]\\cdots L\[\\pi^{h\_\\alpha-1}x\]=\\operatorname{rev}(w\_\\alpha)\\quad(x\\in I\_\\alpha).}\\tag{1}\\\]

The stored word is the reverse of the bytes LF reads, so that it is already in forward text order. This is the construction's invariant I, and the code's [comment on the stored words](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/src/cells.rs#L748-L760>) follows the same convention.

Two quantities must stay separate:

\\\[\\lambda\_\\alpha=\\text{number of base rows},\\qquad h\_\\alpha=|w\_\\alpha|=\\text{number of LF steps per return}.\\\]

A one-row interval can hold a very long word. Two intervals can hold identical words and still have different destinations.

### 1.3 The towers are exact

For each interval, take its levels

\\\[I\_\\alpha,\\ \\pi(I\_\\alpha),\\ \\ldots,\\ \\pi^{h\_\\alpha-1}(I\_\\alpha).\\\]

They form a finite return tower, also called a Kakutani–Rokhlin tower. Its base width is \\(\\lambda\_\\alpha\\) and its height is \\(h\_\\alpha\\).

**P.** The levels of all the towers partition the saturation of the kept rows, the rows on LF cycles that meet them:

\\\[X\_m=\\bigcup\_{j\\in\\mathbb Z}\\pi^j(D\_m).\\\]

Every point of \\(X\_m\\) has a unique most recent visit to \\(D\_m\\) before it, and that visit gives its base point and its level. So

\\\[\\boxed{\\sum\_\\alpha \\lambda\_\\alpha h\_\\alpha=|X\_m|.}\\tag{2}\\\]

When LF is one cycle through all \\(n\\) rows, the sum is always \\(n\\). If whole components without the starting row have been discarded, the sum is smaller.

This is the precise sense in which Cells keeps the original dynamics. Define the discrete suspension

\\\[\\widehat X\_m=\\{(x,j):x\\in D\_m,\\ 0\\le j\<h\_m(x)\\},\\\]

with the dynamics

\\\[\\widehat T\_m(x,j)=\\begin{cases}(x,j+1),&amp;j+1\<h\_m(x),\\\\(T\_m(x),0),&amp;j+1=h\_m(x).\\end{cases}\\\]

Then

\\\[\\eta(x,j)=\\pi^j(x)\\\]

is a bijection onto \\(X\_m\\), and

\\\[\\eta\\circ\\widehat T\_m=\\pi\\circ\\eta.\\tag{3}\\\]

The towers, expanded, are conjugate to the original permutation on its saturation. The smaller map \\(T\_m\\) on the bases is an induced system: it has fewer states and variable return times. Our [notes on itinerary certificates](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-certificates.md>) (§3) described each step as a conjugacy of the smaller map to the larger one. That overstated it: the conjugacy holds once the tower levels are added, as (3) states.

### 1.4 One step

Let \\(A\\) be the last domain interval and \\(B\\) the last image interval, with \\(A\\ne B\\). Put

\\\[s=\\min(\\lambda\_A,\\lambda\_B).\\\]

Their overlap at the end is

\\\[Q=I\_A\\cap J\_B=\[m-s,m).\\\]

The new kept set is \\(D\_{m-s}\\). For

\\\[K=T\_m^{-1}(Q)\\subseteq I\_B,\\\]

the old path has the form

\\\[x\\in K\\ \\xrightarrow{B}\\ T\_m(x)\\in Q\\ \\xrightarrow{A}\\ T\_m^2(x).\\\]

So the new return branch is

\\\[T\_{m-s}|\_K=T\_m^2|\_K,\\\]

with width \\(s\\), height \\(h\_B+h\_A\\), and stored word

\\\[\\boxed{w\_{\\mathrm{new}}=w\_Aw\_B.}\\tag{4}\\\]

LF passes through \\(B\\) and then \\(A\\); the stored word, in forward text order, is \\(w\_Aw\_B\\). On the block `banana`, one step has \\(A\\) and \\(B\\) of equal width one, \\(w\_A=\\) `a` and \\(w\_B=\\) `b`, and it made the word `ab`.

The updates to widths, words and the two orders follow directly from the two old returns crossing the removed end:

| Case | Width update | Word update | Order update |
| --- | --- | --- | --- |
| \\(\\lambda\_A\>\\lambda\_B\\) | Shorten \\(A\\) by \\(\\lambda\_B\\) | \\(w\_B\\leftarrow w\_Aw\_B\\) | Move \\(B\\) just after \\(A\\) in image order |
| \\(\\lambda\_A\<\\lambda\_B\\) | Shorten \\(B\\) by \\(\\lambda\_A\\) | \\(w\_A\\leftarrow w\_Aw\_B\\) | Move \\(A\\) just after \\(B\\) in domain order |
| \\(\\lambda\_A=\\lambda\_B\\) | Delete \\(A\\) | \\(w\_B\\leftarrow w\_Aw\_B\\) | \\(B\\) takes \\(A\\)'s place in image order |

This is the table of the construction's journal (§2), which proves each case. When \\(A=B\\), something else happens: \\(I\_A=J\_A\\), so this branch is the identity and its tower closes.

## 2\. What do our words name?

Each term of our working vocabulary names one of the objects above. Some name more than one, and the table says which.

| Our term | The mathematical object | What to keep apart |
| --- | --- | --- |
| Letter | A label \\(\\alpha\\) for a base interval \\(I\_\\alpha\\), its image \\(J\_\\alpha\\), and their return data | At first a run, possibly cut to isolate the starting row. It is neither a byte nor a word value. |
| Word | An element \\(w\_\\alpha\\in\\Sigma^\*\\), the forward-text label of one return path | The string value, its grammar node and one occurrence of that node are three different objects. |
| Cell | By context: the overlap \\(Q\\), the new return branch \\(K\\to T\_m(Q)\\), or the fixed record that holds that branch | It is not a cell of a CW complex. Calling a step "a cell" is implementation language. |
| Rule | A production \\(N\\to UV\\) in a binary straight-line program (SLP), or a node of a concatenation DAG | A pair of child IDs is still a rule, even where the code calls it a compact "join". |
| Induction step | Replacing \\(T\_D\\) by its first return to a smaller section \\(D'\\), with return labels composed | The unequal cases are Rauzy moves. Equal-width reductions and closing components complete the finite algorithm. |
| Marked row | A distinguished original state \\(r\_\*\\), carried as a tower coordinate or as a distinguished terminal occurrence in the grammar | Its row index is not its output position, and it need not stay the base point of its tower. |
| Writer | Evaluating the SLP, together with placing occurrences, scheduling copies, and choosing the cyclic phase | The grammar's correctness and the writer's are separate obligations. |
| Stop | A stopping index in the induction's history, which selects an intermediate section and its labelled return map | An early stop on a budget differs from a cycle closing naturally, and from refusing on a budget and falling back. It is not a critical cell. |

The code says "cell" because a finished return piece is written into the records of both orders, while what remains of each interval stays at the fronts. That is a representation of the induction above, which the [technical companion](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-12-rauzy-technical-companion/#4-how-the-cells-algorithm-implements-that-mathematics>) describes in detail.

### Substitutions, cylinders and grammar nodes

Three further descriptions are useful.

**A finite S-adic presentation.** Each stage defines a substitution from the new alphabet of intervals into the previous one. Composed, they give a finite directive sequence of return-word substitutions. The stored object is most directly called an SLP. Versions matter: an assignment such as \\(w\_B\\leftarrow w\_Aw\_B\\) creates a new node that refers to the old values, so the grammar has no cycles.

**Cylinder sets.** A base interval lies inside a cylinder of constant return path. It need not be the largest cylinder for its byte word. Isolating the starting row deliberately cuts intervals finer, and equal byte words need not mean equal paths through the rows.

**Occurrence coordinates.** A grammar node can occur many times in the output. The writer's "first place" for a node is the smallest position of one of its occurrences. It is not a position that belongs to the string value itself.

For an unrotated expansion of the root, the first places come from a min-plus computation over the DAG. If \\(N\\to UV\\),

\\\[P(U)\\leftarrow\\min(P(U),P(N)),\\qquad P(V)\\leftarrow\\min(P(V),P(N)+|U|).\\tag{5}\\\]

That is why the writer passes over the rules in reverse order. Its next pass writes out factors and copies repeated occurrences, and its handling of the marked path adds the output phase (construction journal, §§3.2 and 3.5).

## 3\. Is the induction discrete Morse theory?

### What Ghrist states

In §8.8 (pp. 205–206), a discrete vector field pairs cells \\((\\sigma,\\tau)\\), with \\(\\sigma\\) a face of \\(\\tau\\) of one dimension lower. Unpaired cells are critical. The field is a gradient when it has no periodic alternating flowline. The discrete flow is

\\\[\\Phi=\\operatorname{Id}+\\partial V+V\\partial,\\\]

and Theorem 8.24 (p. 206) identifies the resulting Morse homology with the original cellular homology. These are constructions on chain modules, with signed or mod-two counts of paths.

### The exact construction for LF

Build the orbit graph \\(G\_\\pi\\), with a vertex for every row \\(x\\) and an oriented edge \\(e\_x:x\\to\\pi(x)\\). Every component is a circle. The complex is built from the dynamics; it is not the coordinate interval \\(\[0,n)\\). Subdivide one-point loops where needed, to meet Ghrist's hypothesis of a regular cell complex.

Choose a section \\(D\\) that meets every component under consideration, and define

\\\[\\boxed{V=\\{(y,e\_y):y\\notin D\\}.}\\tag{6}\\\]

That pairs each removed vertex with its outgoing edge.

**P: no cycles.** An alternating path follows LF through removed vertices. It must eventually reach \\(D\\), where it stops, so it cannot be periodic.

The correspondence, object by object:

| Discrete Morse object | In LF |
| --- | --- |
| Original 0-cell | A row |
| Original 1-cell | One LF transition |
| Matched pair | A removed row, paired with its outgoing transition |
| Critical 0-cell | A kept row \\(x\\in D\\) |
| Critical 1-cell | The original outgoing edge \\(e\_x\\) of a kept row |
| Gradient continuation of \\(e\_x\\) | The path from \\(x\\) to its first return \\(T\_D(x)\\) |
| Morse boundary | \\(\\widetilde\\partial e\_x=T\_D(x)-x\\) |

The resulting Morse boundary is exactly the incidence boundary of the first-return orbit graph. One Cells step of width \\(s\\) performs \\(s\\) of these row-level eliminations together. That batching is the point: building the full matching row by row would throw away what the interval representation gains.

### What does a stored word correspond to?

A word is the byte coding of the return path, reversed as in (1). That is weaker than recording the path itself. Different paths can emit the same string, and one shared SLP node can stand for several path occurrences. The intervals and the marker keep distinctions that the string alone loses.

So it is legitimate to speak of Morse return paths decorated with word labels. "Morse theory with free-monoid coefficients" is not justified by the cited theorem. Its chain maps use addition and incidence coefficients, and the free monoid of strings is not a chain module. Ordered labels are an enrichment that needs its own argument for being preserved.

Two further qualifications. Suppressing vertices of degree two on a circle is a graph contraction, a Morse reduction. It is not literally a sequence of collapses through free faces inside the original circle, whose vertices each have two incident edges. And discarding a closed LF component without the starting row removes a whole circle, which does not preserve the homology of the full orbit graph. The Morse statement applies component by component to the components still represented.

### What the theorem adds

It gives a valid statement that homology is preserved, and it forces at least one critical vertex and one critical edge on every circle: a circle cannot be reduced to a point with its homology intact. It does not establish that the bytes are right, the output phase, the grammar's size, or the best order of contraction. Those depend on the labelled return invariant (1).

So the correspondence holds exactly, but between row-level contraction of return paths and a Morse matching, not between the implementation's records and topological cells. Our [earlier note on Ghrist's book](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-ghrist.md>) marked as proved that the induction is a Morse contraction with "flowlines labelled by words in the free monoid". That wording drops these distinctions.

## 4\. Are words sections of a sheaf?

### What a sheaf here would require

In §11.9 (p. 286), a cellular sheaf is a functor

\\\[\\mathcal F:\\operatorname{Face}(X)\\longrightarrow\\mathcal C.\\\]

For \\(\\mathcal C=\\mathbf{Mon}\\), the category of monoids, that requires a monoid at every cell, a monoid homomorphism for every incidence of faces, and compatibility between those homomorphisms. Equation (11.3) on the same page describes the global sections as the limit of that diagram.

A stored word is an element of a monoid. It is not itself a stalk. Our notes said that words are "free-monoid sections" of such a sheaf ([the note on Ghrist's book](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-ghrist.md>), §0), but they never gave the stalks and incidence maps that claim needs.

The obvious choices do not work. Putting \\(\\Sigma^\*\\) on every cell, with identity restriction maps, would force adjacent cells to hold equal words, not to concatenate them. Nor can multiplication serve as a restriction map. For strings, which do not commute,

\\\[\\mu:\\Sigma^\*\\times\\Sigma^\*\\to\\Sigma^\*,\\qquad\\mu(u,v)=uv\\\]

is not a homomorphism of the product monoid:

\\\[\\mu((a,b)(c,d))=acbd,\\qquad\\mu(a,b)\\mu(c,d)=abcd.\\tag{7}\\\]

The difference matters precisely because order matters.

### The exact object: a functor on paths

Use the inverse-LF orientation, and label the induced edge

\\\[T\_D(x)\\longrightarrow x\\\]

by \\(w\_{\\alpha(x)}\\). Along paths, these labels concatenate in forward output order. That is a functor

\\\[\\boxed{\\mathsf{Path}(G\_{T\_D}^{-1})\\longrightarrow B\\Sigma^\*,}\\tag{8}\\\]

where \\(B\\Sigma^\*\\) is the category with one object, whose morphisms are words and whose composition is concatenation **in traversal order**: a path that takes \\(e\\) and then \\(f\\) is sent to \\(w\_e w\_f\\). (Under the usual convention that \\(g\\circ f\\) means "\\(f\\) first", the target is the opposite monoid \\((\\Sigma^\*)^{\\mathrm{op}}\\) instead.) It uses a monoid as a one-object category, which is a different construction from a sheaf valued in the category of monoids.

The SLP stores compressed expressions for these path labels. Length is then a monoid homomorphism,

\\\[|\\cdot|:\\Sigma^\*\\longrightarrow(\\mathbb N,+).\\\]

Together they account exactly for ordered word composition and for return times that add.

### A genuine sheaf can be built

For the grammar itself, we can build a sheaf of sets on a bipartite incidence graph. Each production has three distinct incidence edges, for its result, its left child and its right child, even when the two children are the same symbol, as in \\(N\\to UU\\):

- a symbol vertex has stalk \\(\\Sigma^\*\\);
- a production vertex for \\(N\\to UV\\) has stalk \\(\\{(u,v,w):w=uv\\}\\);
- incidence edges have stalk \\(\\Sigma^\*\\);
- the maps from a production to its edges are the matching projections;
- the maps from a symbol to its edges are identities;
- the terminals' values are fixed.

Its global sections are the consistent evaluations of the grammar. It is an exact encoding, but it is extra formal structure, and it does not turn concatenation into a monoid homomorphism. There is also a particularly natural sheaf of positions, built in [section 5.3](<#53-the-position-sheaf>).

**The verdict.** The monoid observation is sound and useful. "Words are free-monoid sections" does not follow until a sheaf is specified. Ghrist's section licenses such definitions; it supplies neither them nor a faster way to evaluate.

## 5\. Are word places circular coordinates?

### 5.1 The cocycle and its period

Restrict attention to the selected LF cycle \\(C\\), the one through the starting row, of length \\(d\\). The induced cycle on \\(D\\cap C\\) has return lengths \\(h(x)\\).

**P.** Its return paths partition \\(C\\), so

\\\[\\sum\_{x\\in D\\cap C}h(x)=d.\\tag{9}\\\]

On the induced graph, orient edges in the LF direction and define

\\\[a(e\_x)=-h(x),\\qquad e\_x:x\\to T\_D(x).\\\]

This is an integer 1-cocycle. On a graph every 1-cochain is a cocycle, because there are no 2-cells, so what carries the information is its period:

\\\[\\langle a,\[C\_D\]\\rangle=-d.\\tag{10}\\\]

It follows that \\(a\\) is not the coboundary of an integer-valued or real-valued position function on the closed cycle. Modulo \\(d\\), though, it is exact:

\\\[\\boxed{p(T\_D(x))-p(x)\\equiv-h(x)\\pmod d.}\\tag{11}\\\]

Here \\(p(x)\\) is the place in the text just after the word of \\(x\\). The scaled cocycle \\(a/d\\) is a real cocycle with integral period \\(-1\\), so it integrates to a coordinate in \\(\\mathbb R/\\mathbb Z\\), and one marked occurrence fixes the additive phase. This is the exact link to §7.13 (pp. 185–186), and in particular to the correspondence between integral first cohomology and maps to the circle on p. 186.

Keep the edge increments as well as the vertex coordinates. At final closure the induced graph may have one vertex and one loop, and the vertex's coordinate alone says nothing about how far the loop winds.

### 5.2 Why modulo \\(n\\) is not always correct

[Stop and walk](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-16-stop-and-walk/#1-how-does-cells-rebuild-a-block>) writes the invariant as

\\\[p(Tx)\\equiv p(x)-|w(x)|\\pmod n.\\\]

That is correct when LF is a single \\(n\\)-cycle. For a selected cycle of length \\(d\<n\\), summing it once round the cycle would give

\\\[0\\equiv-d\\pmod n,\\\]

which is impossible. A concrete valid example is a periodic block:

\\\[\\text{text}=\\texttt{abab},\\qquad L=\\texttt{bbaa}.\\\]

Its LF permutation is

\\\[\\pi=(0\\ 2)(1\\ 3).\\\]

No row-valued \\(p:X\\to\\mathbb Z/4\\mathbb Z\\) satisfies \\(p(\\pi x)=p(x)-1\\): either two-cycle would need \\(0\\equiv-2\\pmod4\\).

There are three precise ways out: take positions modulo the selected period \\(d\\); take positions on an occurrence cover that tells successive visits to the same physical row apart; or state the hypothesis that LF is a single \\(n\\)-cycle. For a valid periodic BWT input, the selected cycle is repeated to make \\(n\\) bytes. For an arbitrary column, the reference semantics likewise repeat the selected cycle and cut it to length.

The construction handles this (journal, §2). The [stop-and-walk record](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-stop-and-walk.md>) assumes one \\(n\\)-cycle explicitly, in its §2. The post states that hypothesis two paragraphs after the formula, as "LF passes through every row of a block in one cycle", which holds for every Wikidata block we decode but not for a periodic one. Later shorthand should keep the hypothesis next to the formula.

### 5.3 The position sheaf

Here is a concrete realisation of the phrase "positions are sections of length data". For each induced edge \\(e:x\\to y\\), put \\(\\mathbb Z/d\\mathbb Z\\) at both end vertices and at the edge. Give every edge two endpoint incidences, even when it is a loop (\\(x=y\\)), or subdivide loops first, so that a loop's two maps stay distinct. Then define the maps from the ends

\\\[\\rho\_{x,e}(u)=u-h\_e,\\qquad\\rho\_{y,e}(v)=v.\\\]

A global section satisfies

\\\[\\rho\_{x,e}(p(x))=\\rho\_{y,e}(p(y)),\\\]

which is exactly (11). These are maps of sets, or affine maps of torsors. In general they are not group or monoid homomorphisms, since translation by a nonzero constant does not preserve the identity.

Sections exist precisely when the total increment round each cycle vanishes modulo the chosen modulus. One anchor fixes the remaining additive freedom on each component. With the stalks and restriction maps that were missing, the compatibility claim becomes exact.

### 5.4 What Example 7.37 contributes

Example 7.37 (p. 185) comes just before the §7.13 heading. It uses the discrete Hodge decomposition of Theorem 7.34 (p. 184) to split pairwise comparison data into a gradient ranking and cyclic inconsistencies. For Cells, the relevant lesson is the obstruction to a real-valued ranking on a closed cycle with nonzero circulation.

The harmonic smoothing in §7.13 has no evident role in exact decoding. Replacing the length cocycle by a cohomologous harmonic representative keeps the winding, but in general it moves the positions given to intermediate vertices. Decoding needs the actual return lengths, not only their cohomology class.

Finally, the row coordinate \\(p\\) and the writer's first-occurrence function \\(P\\) in (5) are different objects. The first integrates lengths along an orbit; the second chooses canonical occurrences in a grammar DAG.

## 6\. What does the formal account prove?

### 6.1 A compact correctness theorem

**P.** Suppose an implementation maintains:

1. the first-return map on the kept section;
2. the return-word invariant (1), exactly;
3. a distinguished occurrence of \\(r\_\*\\) in the return towers;
4. an SLP whose evaluation equals each stored return word.

Then the closure of the marked tower determines the reference inverse-BWT output.

*Proof.* Isolate \\(r\_\*\\) at the start as a base interval of its own, one row wide. [The code](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/src/cells.rs#L524-L528>) does exactly this, cutting the starting row's run into up to three intervals. The marked base stays one row wide: an interval of width one can never be strictly wider than another positive integer width, so it is never the one shortened, and an equal step leaves the survivor one row wide.

When its branch closes, its return word \\(E\\) describes exactly the selected LF cycle. Let \\(d=|E|\\), and let \\(j\\) be the position of the marked terminal in \\(E\\). The reference walker steps through inverse LF before it emits, so its output is

\\\[\\boxed{O\[k\]=E\[(j+1+k)\\bmod d\],\\qquad 0\\le k\<n.}\\tag{12}\\\]

Equation (1) gives the bytes, the marker gives the phase, and repetition gives the requested length. ∎

This is stronger than a homology argument, because it keeps the observable and the distinguished occurrence. The writer still needs its own proof that the copies it schedules actually produce this string. One recorded bug shows the difference: an earlier writer left some bytes on the marked rules' path unwritten while the grammar was correct ([technical companion](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-12-rauzy-technical-companion/#3-the-reconstruction-program-produced-by-induction>), §3).

### 6.2 Widths and heights transform dually

For a top win (\\(\\lambda\_A\>\\lambda\_B\\)), let \\(M=I+E\_{AB}\\), with rows indexed by old intervals and columns by new ones. Then

\\\[\\lambda\_{\\mathrm{old}}=M\\lambda\_{\\mathrm{new}},\\qquad h\_{\\mathrm{new}}=M^{\\mathsf T}h\_{\\mathrm{old}}.\\tag{13}\\\]

The first equation undoes the subtraction of widths; the second adds return times. Therefore

\\\[\\lambda\_{\\mathrm{new}}^{\\mathsf T}h\_{\\mathrm{new}}=\\lambda\_{\\mathrm{old}}^{\\mathsf T}h\_{\\mathrm{old}}.\\\]

That is a cheap algebraic check on every update. An equal step uses the corresponding rectangular incidence matrix. Either way, it states that the towers' volume is conserved, without following individual rows.

### 6.3 Bounds on the present schedule of steps

Let \\(r\_0\\) be the number of starting intervals after the starting row is isolated, so \\(r\_0\\le r+2\\) for \\(r\\) runs. Let \\(S\\) count the ordinary concatenating steps. When LF is one \\(n\\)-cycle,

\\\[\\boxed{r\_0-1\\le S\\le n-1.}\\tag{14}\\\]

**P.** Each step removes at least one kept row, which gives the upper bound. Only equal steps delete an interval, and a complete reduction ends with one interval, so there are exactly \\(r\_0-1\\) equal steps. The lower bound needs changing when components without the starting row are discarded, or when the selected component closes before others.

No general \\(O(r\\,\\mathrm{polylog}\\,n)\\) bound holds for the present subtraction schedule. For the primitive text, with the starting row not isolated,

\\\[T=(ab)^k a,\\qquad n=2k+1,\\\]

the BWT has two runs, of widths \\(k\\) and \\(k+1\\), but the algorithm takes exactly

\\\[S=k+1=\\frac{n+1}{2}\\\]

steps. After the first subtraction the widths are \\(k\\) and \\(1\\), and repeated subtractions of one finish the reduction. The construction journal's count of 449,991 steps at \\(n=899{,}981\\) (§§2 and 4) matches this formula.

The same family shows why long words must stay compressed. Successive return words grow linearly, so storing all of them literally can take quadratic total bytes.

### 6.4 Some batching identities are proved; their speed benefit is not

For \\(q\\) consecutive strict top wins by the same pair,

\\\[\\lambda\_A'=\\lambda\_A-q\\lambda\_B,\\qquad w\_B'=w\_A^q w\_B.\\tag{15}\\\]

For \\(q\\) consecutive strict bottom wins,

\\\[\\lambda\_B'=\\lambda\_B-q\\lambda\_A,\\qquad w\_A'=w\_Aw\_B^q.\\tag{16}\\\]

These identities are exact. Binary powering can represent \\(w^q\\) with \\(O(\\log q)\\) extra SLP rules. Changes of order, the boundaries of equal steps, and the transport of the marker must also be composed correctly.

**H.** Whether such acceleration speeds up the actual decoder depends on how often suitable streaks occur and what finding them costs. The identities do show that the topology does not force one operation per elementary subtraction.

### 6.5 Exact itinerary certificates

Suppose a piecewise translation \\(F\\) follows the intervals

\\\[I\_{i\_0},\\ldots,I\_{i\_{k-1}},\\\]

with cumulative translations \\(s\_0=0,s\_1,\\ldots,s\_k\\). Then

\\\[\\boxed{J=\\bigcap\_{j=0}^{k-1}(I\_{i\_j}-s\_j)}\\tag{17}\\\]

is exactly the set of starts that follow that itinerary.

**P.** For \\(x\\in J\\), the \\(j\\)-th state is \\(x+s\_j\\in I\_{i\_j}\\), by induction on \\(j\\). Every such start emits the same sequence of words and finishes at \\(x+s\_k\\). This is a deterministic certificate for reusing a path, and it needs no hashing.

A later induced interval lies inside the matching certificate, but it need not equal the largest one. The [certificates record](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-certificates.md>)'s lemma states containment, correctly; its title's stronger wording, that the coarser exchange's intervals are "exactly" those columns, needs that qualification. A composite map being a translation does not by itself prove a constant itinerary, either: different itineraries can have the same net translation. The return partition that Cells maintains supplies the missing premise.

### 6.6 What lower bounds are actually available

Producing \\(n\\) bytes explicitly needs \\(\\Omega(n)\\) byte traffic, so in a model that charges per byte the output costs \\(\\Omega(n)\\). It does not need \\(n\\) dependent LF loads, \\(n\\) grammar rules or a serial ranking walk. Similarly, producing an explicit position for every kept row needs space and work for those values; an algorithm that avoids that representation solves a different problem.

Our records' arguments about what memory must be kept say that occurrences not yet placed need recoverable expansion information ([the census of word lifetimes](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-words.md>), §4). They do not prove that this information must sit in the present layout of rules and arena. Re-encoding, recomputation, other grammars and other placement strategies remain possible. The [compact tail grammar](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-compact-tail.md>) is a concrete example: it keeps the same grammar and changes how it is stored.

## 7\. What do the records count?

These are reported counts, separate from the proofs above, each with its scope:

| Status | Recorded result | Scope |
| --- | --- | --- |
| C | 27,439,723 elementary steps, \\(S/r=1.812\\), \\(S/n=0.1309\\) ([construction journal](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-10-rauzy-veech.md>)) | The original right-end census on 233 Wikidata blocks; not an asymptotic law |
| C | At a stop after \\(1.5r\\) cells, about 33%, or 5.0 bytes per original run, of word storage avoided ([stop and walk, reopened](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>)) | A prototype's counts; separate from timing the whole decoder |
| C | Compact tail from \\(1.2r\\): 4.89 bytes per original run saved, about 32% of word storage ([compact tail](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-compact-tail.md>)) | All 233 Wikidata blocks, with wider IDs where needed; the same grammar |
| C | At \\(\\beta=1.5\\), **at least** about one third of sampled starts share a whole-word itinerary covering 256 bytes with another start, certified within a 64-step search that does not wrap past the walk's end. This is a lower bound: starts the search doesn't certify are not shown to fail (on `b a^999` the true share is 0.744 and this search reports 0) ([certificates](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-certificates.md>)) | Every eighth Wikidata block; does not show that all this reuse can be realised profitably |

Here \\(\\beta\\) is the stop's budget of cells per original run. One vocabulary correction changes how these figures read: "per letter" in our records usually means per original run \\(r\\), not per interval still alive. The [record on splitting the walk](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-two-chains.md>) gives, at \\(\\beta=1.5\\), approximately

\\\[m=0.672r,\\qquad q\_{\\mathrm{live}}=0.1212r,\\\]

so there are about 5.54 kept points per live interval, not 0.672.

Likewise, "the stop is free" means that the kernel's existing budget check can be reused. Building the snapshot and emitting from it still cost work. "One lane is needed" is an engineering judgement, not a mathematical consequence of the number of points. The early prototype also leaves the shift of phase for the original row unexercised or uncounted, so its byte comparisons should not be promoted into a proof for every starting row and for periodic input ([stop-and-walk post](<https://lmmx.github.io/mibz2/pr-preview/pr-504/blog/2026-11-16-stop-and-walk/index.md>), §3; [reopened design](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>), §3).

## 8\. Where this leaves us

The strongest conclusion the evidence warrants has four parts. Labelled first-return towers and SLP evaluation give Cells its correctness proof. Discrete Morse theory gives an exact auxiliary model of the reduction. Circular cohomology describes the phase compatibility precisely, once the modulus is the selected cycle's length. And the monoid-sheaf claim needed a construction, which sections 4 and 5.3 now supply.

None of the cited sections of Ghrist's book establishes that the decoder is optimal. A claim that continuing the induction is the only cheap way to recover positions would need a specified representation and cost model, and then a lower-bound argument. The records give useful design comparisons and counts, but no such impossibility theorem.

What comes next follows from the status of each part. The two sheaves of sections 4 and 5.3 are exact, and whether either suggests a cheaper way to place words (**H**) is untested. The batching identities (15) and (16) are proved; whether streaks are common and cheap enough to find for them to pay (**H**) is a question for a count. The check (13) costs a multiply-add per step and could be added to the debug checks as it stands.

---

**Sources:**

- R. Ghrist, *Elementary Applied Topology*, edition 1.0, Createspace, 2014. Page numbers are those of the printed edition: §7.13 (pp. 185–186), Theorem 7.34 (p. 184), Example 7.37 (p. 185), §8.8 and Theorem 8.24 (pp. 205–206), §11.9 and equation (11.3) (p. 286). §8.8 (p. 205), §11.9 (p. 286) and §7.13 (p. 185) have been checked against the book; the other page numbers and Theorem 7.34 have not yet.
- The construction and its proof: [the construction's research journal](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-10-rauzy-veech.md>), and the [code](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/src/cells.rs>).
- Stop and walk: [the design record](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-stop-and-walk.md>) and [its reopening](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>); [the census of word lifetimes](<https://github.com/lmmx/mibz2/blob/4599b7633eb6b38b4994b678d8cd0e2095d1a387/docs/journal/monodromy/2026-11-16-words.md>).
- In review as research pull request [\#497](<https://github.com/lmmx/mibz2/pull/497>): [splitting the walk](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-two-chains.md>), [the compact tail](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-compact-tail.md>), [itinerary certificates](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-certificates.md>) and [the note on Ghrist's book](<https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-ghrist.md>).
