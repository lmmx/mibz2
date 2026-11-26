**Yes. Permutation-pattern research is worth bringing into this work, and there is a concrete algorithmic connection to what you want `mibz2` to do.** There are results that turn restrictions on permutation patterns into smaller representations with efficient navigation. That makes this a plausible source of better decoder structures.

My recommendation is to use the field to investigate **the permutation that remains after the decoder’s existing reductions**: whether it has a decomposition that lets us store, construct, or traverse it more cheaply. That fits the experiment in [#985](https://github.com/lmmx/mibz2/pull/985), which separates the value of a representation from the cost of discovering and constructing it.

I also think I found the conference you remember.

## The conference is probably Permutation Patterns 2021

The strongest match is the **Permutation Patterns 2021 Virtual Workshop**, hosted by Strathclyde on **15–16 June 2021**. Its programme includes **Christian Gaetz from MIT**, speaking on *Spherical Schubert varieties and pattern avoidance*. There was also a virtual meeting in 2020, so I cannot identify your attendance conclusively, but the 2021 event matches several details you mentioned. ([combinatorics.cis.strath.ac.uk](https://combinatorics.cis.strath.ac.uk/pp2021/?utm_source=chatgpt.com), [combinatorics.cis.strath.ac.uk](https://combinatorics.cis.strath.ac.uk/pp2021/PP2021-Programme.pdf?utm_source=chatgpt.com), [permutationpatterns.com](https://permutationpatterns.com/history/?utm_source=chatgpt.com))

The associated publication was a **special issue of *Enumerative Combinatorics and Applications*, volume 2, issue 4, published in 2022**. The editors describe it as a selection of papers, some directly related to the presentations. So “conference proceedings” was a reasonable way to remember it; “conference special issue” is the more precise description here. ([ecajournal.haifa.ac.il](https://ecajournal.haifa.ac.il/Volume2022/ECA2022_S4PP0.pdf?utm_source=chatgpt.com))

If that was your workshop, attending to listen fitted the published arrangements: participants were **invited** to submit work. Submission was not stated as an obligation of attendance. The [conference page](https://combinatorics.cis.strath.ac.uk/pp2021/) still links the programme, abstracts, slides, and videos. ([combinatorics.cis.strath.ac.uk](https://combinatorics.cis.strath.ac.uk/pp2021/?utm_source=chatgpt.com))

Your recollection of an NSA connection also has a factual basis: the conference series’ history lists NSA grants for some editions, including 2018 and 2022. That establishes sponsorship; it does not establish undisclosed applications of particular talks. For your performance question, the public algorithms provide plenty to work with. ([permutationpatterns.com](https://permutationpatterns.com/history/?utm_source=chatgpt.com))

## There is a surprisingly direct result

The most relevant paper I found is **László Kozma and Michal Opler, [*Compact Representations of Pattern-Avoiding Permutations*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2026.92), ESA 2026**.

For a permutation of size \(m\) avoiding a fixed pattern \(\beta\), their result gives:

- \(O(m\log s_\beta)\) **bits** of storage;
- \(O(1)\) time to evaluate the permutation or its inverse;
- \(O(m\log s_\beta)\) construction time.

Here \(s_\beta\) is the growth constant associated with the avoided pattern. For a fixed pattern, the storage and construction bounds are linear. The construction does not even require being told which pattern is avoided. ([drops.dagstuhl.de](https://drops.dagstuhl.de/storage/00lipics/lipics-vol388-esa2026/html/LIPIcs.ESA.2026.92/LIPIcs.ESA.2026.92.html?utm_source=chatgpt.com))

This is directly relevant: a restriction on relative order can yield a compact, navigable data structure. The remaining engineering question is whether its actual operations beat our current accesses. A theoretical constant-time lookup can still involve several dependent operations, tables, and coordinate conversions.

A more specific example comes from **separable permutations**, which avoid the two patterns **2413 and 3142**. Chakraborty, Jo, Kim, and Sadakane gave a succinct representation supporting constant-time forward and inverse queries in their ISAAC 2024 paper. That is another explicit connection between a permutation class and executable data structures. It remains conditional on the input having the required structure; we have not established that for `mibz2`’s retained maps. ([drops.dagstuhl.de](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ISAAC.2024.17?utm_source=chatgpt.com))

These results give a solid reason to investigate your idea. They do not yet give us a decoder speedup.

## The crucial question is which permutation we examine

There are several related mathematical objects in this project, and applying a theorem to the right one matters.

A classical permutation pattern records **relative order**. For example, the values \(7,2,5\), selected in that order, have pattern \(312\). The field studies which such configurations occur, which are forbidden, and what structural consequences follow. ([oro.open.ac.uk](https://oro.open.ac.uk/23475/1/simples-survey.pdf?utm_source=chatgpt.com))

The decoder also cares about **iteration**: starting at a row, following its successor, then that successor’s successor, while recovering the correct bytes. Relative-order structure and traversal structure can interact, but neither determines the other completely.

### Raw LF already has strong pattern structure

For the BWT column \(L\), using zero-based indexing, the LF permutation is

\[
\operatorname{LF}(i)
=
C[L[i]]
+
\#\{j<i:L[j]=L[i]\},
\]

where \(C[c]\) counts symbols smaller than \(c\). This is the stable-sort mapping already described in the project’s [interval-exchange research](https://github.com/lmmx/mibz2/blob/master/docs/journal/monodromy/2026-11-10-rauzy-veech.md).

Two useful conclusions follow directly.

First, if you look only at positions containing one particular symbol, their LF destinations increase in the same order. Therefore, for an alphabet of size \(\sigma\), LF can be partitioned into at most \(\sigma\) increasing subsequences.

In permutation-pattern language, **LF avoids a decreasing pattern of length \(\sigma+1\)**: a longer decreasing subsequence would need two occurrences of the same symbol, but those two destinations must increase.

Second, within a contiguous run of identical symbols, LF is particularly simple:

\[
\operatorname{LF}(i+d)=\operatorname{LF}(i)+d.
\]

It translates an entire interval without changing its internal order.

That is already close to the mathematics the project exploits through run intervals, interval exchanges, and first-return constructions. The new question is what additional structure becomes visible through the pattern-theory viewpoint, beyond these familiar regularities.

### The reduced map can lose the raw map’s pattern restrictions

The Open kernel removes intermediate work and retains a smaller map. Its edges can represent several original LF steps, with the corresponding output information attached.

This is a **first-return map**: follow LF until you reach another retained row.

That operation differs from taking a classical permutation pattern. Here is a small exact example, using one-based indices.

For the binary BWT column

\[
L=\texttt{bbaaa},
\]

the LF permutation is

\[
[4,5,1,2,3].
\]

It avoids \(321\), as expected for a binary alphabet.

Now retain rows \(\{1,2,3,4\}\). The edge from row 2 follows

\[
2\longrightarrow 5\longrightarrow 3,
\]

so the first-return map becomes

\[
[4,3,1,2].
\]

This **contains \(321\)**, through the values \(4,3,1\).

By comparison, selecting the original LF entries at those same four positions gives \([4,5,1,2]\), whose classical pattern is \([3,4,1,2]\). That is a different permutation.

This arithmetic example establishes a useful boundary:

**A pattern restriction proved for raw LF does not automatically hold for the permutation that Open actually traverses.**

Consequently, the structural census should examine the actual retained map and any proposed alternative map. If we consider squared maps, jumps, or further contractions, those deserve their own examination too.

### “One cycle” leaves a lot of structure undecided

The existing [Wikidata block census](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-24-wikidata-block-census.md) found one cycle through all retained rows on every admitted block it measured. It also found relatively little short-distance locality in the retained ranks.

Those findings do not settle the pattern question.

For a small illustration, both \(2341\) and \(4312\) are single four-cycles. The first avoids \(321\); the second contains it. Their cycle counts agree while their order patterns differ.

Likewise, large average jumps do not prove that a permutation lacks a useful recursive decomposition. They tell us something about displacement in the present coordinates. A different structural description might still reduce the metadata or the work needed to navigate it.

**The missing empirical link is whether useful additional structure exists in the real retained maps.** The existing census gives us suitable inputs and some controls, but it did not answer that specific question.

## The parts of the field I would prioritize

I would start with structural descriptions that suggest a concrete representation. That gives each mathematical observation an immediate performance question.

| Direction | What we would look for | Potential benefit | What must be checked |
|---|---|---|---|
| **Substitution decomposition and common intervals** | Consecutive source positions whose destinations form a consecutive set, recursively | Compact descriptions of blocks and their internal permutations | Whether navigation through the decomposition is cheap |
| **Monotone decompositions** | Increasing runs or interleaved increasing subsequences beyond those already exploited | Smaller routing information or simpler local mappings | Whether lookup machinery costs more than it saves |
| **Griddings and compact local shapes** | A permutation plot decomposable into a small collection of structured regions | Shared small tables, fewer coordinates, smaller working sets | Total metadata and coordinate-conversion cost |
| **Small-pattern profiles** | Overrepresented, rare, or absent relative-order configurations | Evidence guiding the choice of a representation family | A statistical signature alone supplies no execution shortcut |

### 1. Substitution decomposition is a good first investigation

An *interval* in this literature is a consecutive set of input positions whose image is also a consecutive set of values. Substitution decomposition recursively organizes such blocks.

The terminology contains an amusing trap: a **simple permutation** has no nontrivial interval of this kind. “Simple” means irreducible under this decomposition; it does not mean easy to execute. Brignall’s [*A Survey of Simple Permutations*](https://oro.open.ac.uk/23475/1/simples-survey.pdf) is a useful introduction. It also discusses compact strong-interval decompositions, which avoid explicitly listing the potentially quadratic number of all intervals. ([oro.open.ac.uk](https://oro.open.ac.uk/23475/1/simples-survey.pdf?utm_source=chatgpt.com))

For our purposes, I would ask:

- How much of the retained map breaks into substantial blocks?
- How large are the irreducible pieces?
- Are the same small internal shapes repeated?
- How deep is the decomposition?
- After adding exact addresses and output labels, how many bytes remain?

Suppose a region can be represented by a destination base and a small local permutation. That could replace many unrelated destination entries with a compact descriptor. If many regions share the same local shape, a shared lookup table becomes conceivable.

The benefit would depend on the full access sequence. A smaller representation might improve cache behaviour enough to justify extra arithmetic. Equally, a deep decomposition might replace one lookup with several dependent lookups and lose.

There is another subtlety: a block’s image being contiguous does not mean the whole block has a single next block under repeated traversal. Its destination interval can cross boundaries in the source partition. Therefore, a substitution decomposition can help evaluate the permutation without automatically letting us emit an entire block as one output word.

That is a useful distinction when designing the consumer.

### 2. Monotone structure is an established bridge to compression

Barbay and Navarro’s [*On Compressing Permutations and Adaptive Sorting*](https://barbay.cl/Publications/2013-TCS-OnCompressingPermutationsAndAdaptiveSorting-BarbayNavarro_withoutPreamble.pdf) develops representations that exploit sorted runs and related subsequence structure while supporting permutation and inverse operations. Its bounds connect the amount of order in the input with storage and navigation costs. ([barbay.cl](https://barbay.cl/Publications/2013-TCS-OnCompressingPermutationsAndAdaptiveSorting-BarbayNavarro_withoutPreamble.pdf?utm_source=chatgpt.com))

This is a good source of mechanisms and comparators. For `mibz2`, the informative question would be whether the retained map has a useful monotone decomposition **after** the existing run-based reductions.

It also suggests a productive way to reason about the trade-off: compression changes the cost of accessing the permutation, not just the number of bytes allocated. We should measure both.

### 3. Small-pattern counting is useful as a diagnostic

There was a particularly relevant talk at PP2021: **Chaim Even-Zohar’s *Counting Small Patterns and Testing for Independence***, based on joint work with Calvin Leng. Their paper gives efficient algorithms for small-pattern counts, including nearly linear algorithms for all patterns of lengths two and three. ([combinatorics.cis.strath.ac.uk](https://combinatorics.cis.strath.ac.uk/pp2021/abstracts/EvenZohar-Chaim.pdf?utm_source=chatgpt.com), [arxiv.org](https://arxiv.org/abs/1911.01414?utm_source=chatgpt.com))

That gives a practical way to ask whether our maps have order structure that the existing run and displacement statistics miss.

For example, we could compare fast and slow blocks with similar retained sizes and run characteristics. If their small-pattern profiles differ consistently, that could suggest where to look for a better decomposition.

But I would keep this part tightly connected to a representation hypothesis. Finding a nonrandom profile is interesting; it becomes useful for speed when we can explain which operations the corresponding structure eliminates.

Also, **sampling can find a forbidden-pattern counterexample, but failing to find one does not prove avoidance**. A representation relying on exact avoidance needs exact recognition, a proof, or an explicitly supported exception mechanism.

### 4. “Constructive” results need to be read with our cost model in mind

Your recollection that many talks concerned construction is relevant. There are several meanings of construction here, though:

| Mathematical or algorithmic task | Relevance to this project |
|---|---|
| Generate members of a permutation class | Useful for examples, adversarial cases, and exploring hypotheses |
| Derive a recursive specification or counting formula for a class | Can reveal a representation family worth testing |
| Find a decomposition of one given input permutation | Directly relevant to the online constructor |
| Traverse that representation and recover the labelled output | Directly relevant to the decoder’s consumer |

For example, the **Combinatorial Exploration** programme automatically derives structural specifications and enumeration results, with an open-source implementation and a database of permutation-class results. That is substantial constructive mathematics. Its immediate purpose differs from finding a sufficiently cheap representation of the particular block being decompressed. ([arxiv.org](https://arxiv.org/abs/2202.07715?utm_source=chatgpt.com))

I would bring that kind of machinery in once a concrete family has emerged. It could help discover or prove a decomposition rule. Starting with a broad automated search across classes would make it harder to keep the work connected to the actual decoder costs.

## How this should fit the permutation-structure experiment

The [existing experiment note](https://github.com/lmmx/mibz2/blob/6b14323dee33615d00f0421095fd29af405282a4/docs/journal/2026-11-24-algorithm-level/ideal-permutation-structures/README.md) is well suited to this idea. It asks us to measure the value of alternative exact representations before spending effort on their constructors.

I would give permutation patterns a specific role within that experiment.

### First, inspect structure on the actual maps

Take representative retained maps from the real blocks and compute a small set of structural measurements:

- Strong-interval decomposition, including the sizes of irreducible pieces.
- Monotone runs and a suitable increasing-subsequence decomposition.
- Repetition among small local permutation shapes.
- A bounded sample or inexpensive counts of short patterns.

The raw LF map can serve as a comparison, showing how much structure the reduction preserves or destroys.

For performance interpretation, compare blocks of similar size and known run characteristics. Otherwise we could discover that a pattern statistic correlates with cost simply because both correlate with the amount of retained work.

I would also preserve the experiment’s separation of training and held-out blocks. Choosing a decomposition because it looks unusually good on one block is a discovery step; it is not yet evidence that the representation is useful across the workload.

### Then construct one or two concrete representations offline

The measurements should decide what to build.

If there are large recursively structured intervals, try a representation built around them. If repeated small local shapes dominate, investigate shared local tables. If monotone subsequences provide the clearest saving, test that route.

This is where the existing “ideal structure first” approach is particularly valuable. We can allow expensive offline discovery and ask:

> Given this representation already built, can a real consumer produce the correct output more cheaply?

That isolates whether the mathematical structure has downstream value.

A large irreducible core would be useful negative evidence for a substitution-based candidate. It would not prove that every pattern-based or compact representation must fail. The verdict should stay attached to the family actually tested.

### Charge the complete labelled consumer

The permutation is only part of the decoder object.

The consumer must retain the information needed to recover exact bytes in the correct order. Depending on the design, that includes return words, lengths, offsets, routing information, the marked row, and handling of periodic or multicycle cases.

All navigation, coordinate conversions, label retrieval, output stores, and supporting metadata belong in the comparison. Otherwise a beautifully compressed permutation could hide the real cost in a second structure.

This is also why constant-time permutation queries do not automatically solve multi-step emission. The extended Kozma–Opler paper explicitly leaves efficient power queries under comparable bounds as an open question. The theorem gives an important component, while the decoder still needs its own exact consumer. ([arxiv.org](https://arxiv.org/html/2510.20382v1?utm_source=chatgpt.com))

The matched prebuilt baseline and forward-emission control already specified in #985 are useful here. A candidate should earn credit for what its representation changes, with comparable warm and cold conditions.

### Finally, derive the construction budget

Once the consumer has been measured, the constructor question becomes much sharper.

Let

\[
C_0+E_0
\]

be the baseline construction and emission cost, and let \(E_1\) be the candidate’s measured consumer cost.

To obtain a desired improvement \(\Delta\), the new constructor must satisfy

\[
C_1 \le C_0+E_0-E_1-\Delta.
\]

That gives the mathematical search an explicit engineering target.

A candidate with a very cheap consumer might tolerate a somewhat more expensive constructor. A candidate with only a tiny consumer advantage needs extremely cheap construction. If the consumer itself loses, it needs a separate, credible construction saving to compensate.

The current experiment already requires an explicit allowable construction cost and a meaningful improvement budget. Permutation-pattern theory can supply candidate structures and recognition methods within that framework; there is no need to weaken its existing tests.

## One particularly useful boundary on the search

There is an easy mathematical observation that illustrates why this separation matters.

Every single-cycle permutation can be relabelled into the shift

\[
i\mapsto i+1\pmod m.
\]

Choose a starting point \(s\), and define its new coordinate by

\[
q(\pi^t(s))=t.
\]

In those coordinates, traversal is trivial.

But discovering \(q\), translating coordinates, and retrieving the correct labels can contain essentially the work we wanted to avoid. The existence of a simple representation is therefore weaker than the existence of a useful decoder.

The promising pattern-theory results give us something more demanding to investigate: **structure visible in the given ordering, from which a compact representation can be obtained without first solving the whole traversal problem**.

That is where I would focus your idea.

## Where I would start reading

For this project, I would put these three first:

1. **[Kozma–Opler: *Compact Representations of Pattern-Avoiding Permutations*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2026.92).**  
   The clearest evidence that pattern restrictions can yield useful navigation structures.

2. **[Brignall: *A Survey of Simple Permutations*](https://oro.open.ac.uk/23475/1/simples-survey.pdf).**  
   A route into intervals, substitution decomposition, and the irreducible pieces we could measure.

3. **[Barbay–Navarro: *On Compressing Permutations and Adaptive Sorting*](https://barbay.cl/Publications/2013-TCS-OnCompressingPermutationsAndAdaptiveSorting-BarbayNavarro_withoutPreamble.pdf).**  
   A concrete connection between ordered subsequences, representation size, and access cost.

The question I would carry into the existing task is:

**Does the exact labelled permutation left after Open’s reductions admit a pattern-based decomposition that makes its real consumer cheaper, and what construction budget does that saving permit?**

That gives your conference interest a specific, testable role in making the software faster.
