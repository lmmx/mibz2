# ChatGPT Pro on "Against Mathematics" (doomslide, 24 October 2026), relayed by Louis, 24 November 2026

Relayed verbatim. The interactive figures in the original came through as text only: their captions are kept, and their contents are lost.

---

Against Mathematics — from the death of proof to a new method of discovery

I think this piece contains something considerably more valuable for mibz2 than its explicit argument against AI-generated mathematics.

It is, underneath its mythology, an essay about the distinction between possessing a mathematical result, possessing its proof, and possessing the generative principle from which further mathematics can be discovered.

That third thing is almost exactly what we have been trying to capture in the major-shot formulation. And there is a passage in the essay—Hilbert's unpublished twenty-fourth problem—which bears extraordinarily directly on what MONODROMY and Neon are doing.

But I want to follow the narrative before extracting the implications, because the construction of the essay is itself part of its argument.

## I. Charting the narrative

The essay operates through three interwoven narratives:
- The historical narrative: the progressive externalisation of mathematical cognition, from oral teaching to writing, formalisation, proof assistants and language models.
- The mythological narrative: Hippasus, Pythagoras, Hermes, the sea, death and the recovery of knowledge from the underworld.
- The contemporary narrative: AI-generated mathematics and the institutional, intellectual and economic crisis that the author believes it produces.

The first explains the mechanism. The second supplies its symbolic meaning. The third supplies its urgency.

### The nine movements

**I. Rooster — The fragmentation of mathematical cognition.** Mathematics continues functioning even as its constituent intellectual activities become detached from the mathematician. The proud head discovers; the blind head verifies; the slow head understands. Their different speeds and incentives create the conditions for intellectual death.

**II. Tombstone — The formalist victory.** Hilbert relocates mathematical authority from the living intellect to formal inscriptions. Brouwer and Poincaré resist the identification of mathematical activity with its written representation. Gödel undermines the dream of completeness, but formalisation survives institutionally.

**III. Sponge — When a mathematical theory becomes a religion.** Pythagorean arithmetic begins with an astonishing explanatory discovery: numerical ratios govern musical harmony. It then overextends its success into a totalising theory, inventing phenomena to preserve its commitments. Sextus introduces the destructive and potentially liberating power of scepticism.

**IV. Curtain — The separation of knowledge from reasons.** The Pythagoreans divide into Hearers and Learners. One group inherits statements; the other inherits demonstrations. The Hippasus legend becomes a founding myth about authorship, secrecy and the price of understanding.

**V. Turducken — The automation of verification.** Proofs become too large for individual mathematicians to assimilate. Collective trust and eventually machine verification replace comprehensive human checking. The blind head is automated.

**VI. Signature — The ownership of discovery.** Archimedes, Cardano, Tartaglia and Copernicus establish that discovering something, explaining it, proving it and publishing it are distinct activities. Authorship conceals this plurality.

**VII. Trench — The oracle is constructed.** The history of computational reasoning culminates in the language model, represented as an oracle animated by the accumulated writings of the dead. It speaks without necessarily possessing the understanding traditionally associated with its speech.

**VIII. Bag/Boat — Two possible futures.** The contemporary AI announcements alternate with fictional reconstructions of Hippasus. The Bag represents the accumulation and commercial appropriation of results. The Boat represents the journey undertaken to recover their reasons.

**IX. Epilogue — The religion of the dead.** The author concludes that mathematics has become a system for manufacturing certified results while progressively dispensing with the activity that makes those results intellectually meaningful.

I am treating the essay's contemporary reporting as part of its narrative here, not independently verifying its allegations or historical reconstructions. Its philosophical and mathematical implications can be examined independently of whether every anecdote is accurate.

### The deeper architecture

The entire essay can be understood as a series of separations.

| Originally united | Subsequently separated |
|---|---|
| Thought and speech | Writing |
| Knowledge and memory | External records |
| Understanding and verification | Refereeing |
| Proof and its author | Formalisation |
| Discovery and understanding | Automated theorem production |
| Mathematical result and generative method | The oracle |

The essay calls this death. I would call it the progressive factorisation of mathematical cognition.

That distinction matters enormously. Factorisation does not necessarily entail destruction. Separated components can potentially be recombined into an arrangement that was impossible before.

And recombining separated components into a new computational structure is precisely the intellectual operation we have been pursuing with BWT inversion.

## II. A porous reading — allowing the ideas to cross between worlds

### 1. The three heads are three different computational processes

The distinction between the proud, blind and slow heads gives us an unexpectedly useful model for agentic research.

[Figure: proud head = discovery, hypothesis generation; blind head = verification, proof checking; slow head = understanding, generative synthesis. Living mathematical research is discovery, verification and understanding feeding one another.]

There is a danger that the mibz2 research programme has reproduced the imbalance described by the essay.

We have mechanisms for producing candidates. We have mechanisms for proving or refuting them. We have mechanisms for benchmarking them. But the slow head—the activity responsible for transforming accumulated results into a new understanding of the underlying mathematical object—is not necessarily being given an equally explicit computational role.

An obstruction register is a step towards that role, but it is not sufficient.

An agent can accumulate fifty correct obstructions without acquiring the understanding that would allow it to generate the fifty-first approach.

What would it mean to operationalise understanding? Not merely that an agent can explain a theorem in prose, but that it can:
- derive the theorem from an alternative representation;
- identify which assumptions actually generate its conclusion;
- construct a nearby object where the conclusion fails;
- transfer the underlying mechanism into a different mathematical setting;
- generate a new construction that was not supplied to it.

That is understanding as generative capability. It is a much stronger research deliverable than explanation alone.

### 2. Hilbert's twenty-fourth problem is almost exactly our problem

This is, for me, the most important passage in the entire essay:

> Given two routes, it is not right to take either of these two or to look for a third; it is necessary to investigate the area lying between the two routes.

Consider what we have been doing. We have LF traversal. We have Rauzy–Veech induction. We have Open. And we have been looking for the next mathematical route, importing topology, dynamical systems, graph theory, algebra and combinatorics in the hope that one will supply it.

Hilbert's instruction is subtly but profoundly different. Do not look for a third route. Investigate the space between the first two.

[Figure: Hilbert's proposed investigation, applied to the two inversion algorithms.]

What distinguishes LF traversal from Rauzy–Veech is not merely that one is slower and the other faster. There is a mathematical transformation between their computational objects.

The LF walk acts on individual points. Rauzy–Veech acts on induced transformations of intervals. Open restricts the induced dynamics to a retained set.

The question becomes: What is the general mathematical operation of which these transformations are particular instances?

And then:
- What other objects could that operation act upon?
- What other equivalences does it preserve?
- What information is retained under the transition, and what information is discarded?
- What determines the cost of moving between these representations?

This is no longer an indiscriminate search for imported mathematical machinery. It is a study of the transformations between algorithms. It gives us something akin to a geometry of proofs, enriched with computational cost.

### 3. Proof irrelevance versus algorithm relevance

The essay mentions Lean's definitional proof irrelevance: proofs of the same proposition are treated as equal for the relevant type-theoretic purposes.

There is a delicious contradiction here with our earlier invocation of Curry–Howard. We want to explore different proofs because their computational interpretations may have radically different costs. Yet ordinary logical proof equality has no reason to retain that distinction.

For mibz2, these are logically equivalent outcomes: decode_LF(L,p) = T and decode_RV(L,p) = T. But they are not computationally equivalent. What we want is something closer to π₁ : L → T with D ∼ n, and π₂ : L → T with D ∼ m, where the annotations describe the intended dependent-traversal model, not an unconditional machine-time theorem.

The mathematical object of interest is not merely the proposition that inversion is possible. It is the space of computational witnesses to inversion, carrying different resource bounds.

In Lean, we would therefore want to distinguish executable constructions and their cost semantics from proofs that those constructions are correct. We need not abandon Lean's proof irrelevance; we need to retain the algorithmically relevant information in the appropriate objects.

This suggests an important refinement of the major shot: **We are not searching for another proof that the BWT is invertible. We are searching for a transformation of computational witnesses that changes the structural parameter governing their cost.**

That is a significantly more precise interpretation of the original Curry–Howard intuition.

### 4. Pythagorean numerology and the danger of [the governing quantity]

The Pythagoreans discover something genuinely astonishing: musical consonance is governed by numerical ratios. They then extrapolate. The heavens must obey their numerical theory. There must be ten heavenly bodies. When observation supplies nine, the theory supplies the missing planet.

This has an uncomfortable parallel to the previous review. MONODROMY identifies a real mathematical phenomenon: c₁ ≥ h₁ within a particular rigid calculus, subject to the necessary formal qualifications. Then the project begins organising the entirety of possible BWT inversion around that phenomenon. A mathematical observation gradually becomes the ontology through which all candidate algorithms are interpreted.

The danger is that we start modifying the classification of algorithms to preserve the significance of the bound, rather than asking whether the bound correctly characterises the computational reality.

Record 98 initially places sort-based list ranking outside the calculus. Record 99 brings it back inside. That correction may be defensible for particular implementations, but it illustrates the danger: if every alternative computation can be expanded into singleton derivations and charged for the expansion, the resulting calculus may become incapable of recognising the very transformation we seek.

The Pythagorean lesson is not to abandon numbers. It is to avoid making a successful explanatory quantity into a metaphysical commitment.

**The width-one run is a feature of one representation. It has not been established as an irreducible atom of computation.** That distinction is potentially central to the breakthrough.

### 5. Brouwer's time and Enriques's space

The essay sets up a particularly productive opposition between Brouwer and the Italian geometers. For Brouwer, mathematics is constituted through acts occurring in time. For Enriques, mathematical objects are spatially apprehended: he sees what others must derive.

This maps surprisingly closely onto our foundational question.
- Time, the itinerary: LF traversal constructs the text through succession.
- Space, the configuration: positional inversion would recover the configuration through global relations.

The invention of Rauzy–Veech inversion was already a partial transition from time to space. Instead of asking what happens next to a point, it asks what happens to an interval under an induced transformation. But Open ultimately returns to temporal succession in its retained walk.

Perhaps the next step is not another improvement to the induction process. Perhaps it is the discovery of a representation in which the remaining succession is expressed as a simultaneous spatial relation: a coordinate system, a collection of fibres, a global conservation law, a system of relative positions, an ordered partition.

This is not yet an algorithm, but it tells us which kind of mathematical transformation to look for.

### 6. The Archimedes passage: the most important distinction for agentic discovery

Archimedes sends the librarian not just his results and their geometric proofs, but the mechanical method that originally disclosed them. Why? Because the method might lead someone else to results that Archimedes himself had never contemplated.

| Object | What it provides |
|---|---|
| Result | Something that is true |
| Proof | Certification that it is true |
| Method | A mechanism capable of generating further truths |

These are not interchangeable. A method has generative value that neither the result nor its verification necessarily possesses.

Consider two possible MONODROMY outputs. The first is a formal proof that a particular class of algorithms cannot cross a cost threshold. The second is a mathematical transformation that explains the bound, identifies exactly what makes it necessary, and systematically generates objects to which the bound does not apply. The first is a result. The second is research machinery. It may generate an entire family of approaches. It may even lead somewhere unrelated to the original obstruction.

This changes what we might want the agents to preserve. Not private internal reasoning transcripts, nor enormous dumps of unsuccessful prose, but reproducible mathematical discovery artefacts: equations tried, transformations applied, counterexamples, competing representations, failed implications, executable witnesses, and the reasons a particular construction was selected.

**The Method should become a first-class product of the research.**

### 7. Hearers and Learners — two forms of agent memory

The Hearers inherit propositions stripped of their demonstrations. The Learners inherit the relations that make the propositions intelligible. This is directly relevant to the way we maintain FRONTIER.md.

Imagine that the obstruction register grows to several hundred entries: local coordinates blocked; context classes blocked; copy replay blocked; constant-depth formulas blocked; contraction blocked.

An agent that reads these as orders becomes a Hearer. It can be highly efficient at obeying the accumulated research doctrine while losing the ability to reconstruct the reasons that originally justified it. The result is an agent that becomes progressively less capable of discovery as the project becomes progressively more knowledgeable. This is a particularly serious possibility in long-running agentic research.

A living obstruction must preserve its witness, scope, escape condition and **transferable insight**. The project already has the first three. I would add the fourth.
- What mathematical capability did the failed approach reveal, even though it could not use that capability economically?
- What survives if we remove one of its assumptions?
- Can its counterexample become the starting object of an entirely different construction?

A negative result should not merely close a door. It should change the mathematical landscape in which the next search occurs.

### 8. Sextus and the sponge: constructive destruction

Sextus's arguments are not offered as a new dogma. Their function is to cancel an existing dogma and leave the reader free of it. The sponge accidentally produces the very visual effect that the painter could not produce intentionally.

There is a research technique hiding here. Instead of asking what new mathematics satisfies the current formulation, take the formulation apart until some previously inseparable concepts become distinct. For mibz2, several concepts have been travelling together:

| Previously associated concepts | Potential separation |
|---|---|
| Random content and random memory access | Random values can be processed in ordered batches |
| A singleton row and a singleton computation | A wide operation can act on scattered singleton rows |
| Output order and computational order | The text can be assembled from independently acquired pieces |
| A coordinate and an individually computed integer | A coordinate can be encoded by relations or a shared hierarchy |
| A mathematical operation and a machine cost | The same operation can have different physical realisations |
| An obstruction and an instruction to stop | An obstruction can generate new permissible constructions |

The Mario breakthrough had this character too. A single apparent object, Mario's position, decomposed into several state representations that could be manipulated separately.

The essay repeatedly describes these acts of separation as the death of something. Computationally, they might instead be the birth of a new degree of freedom.

### 9. The economics of the slow head

The author's institutional criticism is that discovery, verification and understanding have radically different reward structures. We should take that seriously without necessarily accepting the conclusion that mathematics is dead.

Agentic research introduces a different imbalance. Candidate generation can be extremely fast. Verification can sometimes be automated. But conceptual integration has no equally straightforward reward signal. The result can be a literature of machine-generated mathematics that grows faster than either humans or machines can assimilate it.

Our project has its own miniature version: journals, theorem drafts, benchmark receipts, mathematical leads, research branches, obstruction tables. The risk is not just producing incorrect research. It is producing correct research faster than it can be transformed into a new mathematical principle.

That is why I think the next development in methodology should not simply be more exploration agents or more formalisation. It should be a deliberate process of synthesis, reconstruction and transformation between existing proofs. Hilbert's twenty-fourth problem gives this process a mathematical purpose.

## III. The Eureka movement

### 1. What actually happens in the temple yard?

The essay's fictional Hippasus narrative has three iterations. They are not merely different versions of a myth. They enact three different modes of mathematical investigation.
- **Boat I:** Hippasus consults the oracle, receives a result, cannot recover its reasons, and descends into the underworld in search of them. He returns with a geometric proof.
- **Boat II:** The brothers remain in the temple yard. Instead of seeking another answer, they inspect the numerical examples they already possess. They discover that the examples can generate further examples.
- **Boat III:** Hippasus discovers a transformation whose iteration reveals why the original problem cannot be solved within the existing conception of number.

The second and third movements are particularly important. Consider Pythagorean triangles whose two shorter sides differ by one:

| Short sides | Hypotenuse | Sum of short sides | s² − 2c² |
|---|---|---|---|
| 3, 4 | 5 | 7 | −1 |
| 20, 21 | 29 | 41 | −1 |
| 119, 120 | 169 | 239 | −1 |
| 696, 697 | 985 | 1,393 | −1 |

These apparently separate numerical results are all connected by a transformation (s′, c′) = [[3, 4], [2, 3]] · (s, c). The transformation preserves the quantity s² − 2c². It therefore generates further solutions without independently solving the original problem again.

[Figure: "From isolated answers to a generative law." Iterating the transformation, the integer ratios approach √2 while the defect stays −1. Shown: (41, 29), ratio 1.413793103448, distance from √2 4.205e-4, invariant defect −1.]

The conceptual breakthrough is not finding a better approximation to the square root of two. It is discovering the operation connecting the approximations.

And the operation also has an inverse. In the hypothetical case of an exact positive integral solution to s² = 2c², an appropriate descent would yield a smaller positive integral solution, contradicting minimality.

The generative mechanism and the obstruction are two sides of the same mathematical structure. This is very close to what Hilbert asks for. Do not give me another answer. Give me the transformation that connects the answers.

### 2. What is our equivalent of the temple wall?

Our wall is the entirety of the mibz2 research archive: LF traversal, Cells, Open, contraction, tunnelling, move structures, coordinate sampling, Φ replay, repeated cuts, grammars, the Sturmian solver, the GUID census.

We have been looking at each construction and asking whether it succeeds. What if, instead, we considered these constructions to be numerical examples of an underlying mathematical transformation that we have not yet articulated?
- What is preserved when we move from one to another?
- What is lost?
- What can be transformed in the opposite direction?
- What is the analogue of the Pell invariant?

I think there is at least one candidate worth working through immediately.

### 3. A separation we may have been missing: randomness of content versus randomness of transport

The major-shot formulation places much of its difficulty in random GUID content. The reasoning is understandable. Random digits cause singleton runs. Context keys shatter. Rigid interval compression becomes ineffective.

But these observations establish something about the values and local geometry of the rows. Do they establish something about the collective transport of those rows? Not necessarily.

Consider a thousand GUIDs of identical length embedded in a block. Their payloads may be independent random values. Yet they all possess a common temporal structure:
- They have beginnings and endings.
- Their lengths are fixed.
- Their format has a small number of phases.
- Their digits occupy corresponding positions within each field.

The entropy of the values does not destroy this structure.

[Figure: "Two different kinds of structure": illustrative identifiers with independent-looking payloads but synchronised format.]

The ordinary LF walk processes this structure horizontally, following one identifier's digits before encountering another identifier. But there is another way to organise the problem: process the entire collection of identifiers by corresponding position. Instead of following a point, lift the transformation to an ordered collection of points.

There is a useful exact mathematical property here.

### 4. The ordered-frontier lemma

Recall that LF(x) = C[L[x]] + rank_{L[x]}(L, x), using zero-based rank.

Now take an arbitrary sorted set of rows S = (x₁, x₂, …, x_k), x₁ < x₂ < ⋯ < x_k. These need not be adjacent. They need not belong to the same run. Their characters may be completely random.

Partition the rows by their emitted character, preserving their relative order within each partition. For rows having the same emitted character, L[x_i] = L[x_j] ⟹ LF(x_i) < LF(x_j) (for i < j). Moreover, rows belonging to different character partitions map into ordered, disjoint alphabet ranges.

It follows that: **Applying LF to an ordered set and stably partitioning the resulting records by emitted character produces another ordered set of rows. No comparison sort is necessary to establish that order.**

This is a direct consequence of LF's definition. But consider what it gives us when used as the primitive of a collective decoder.

[Figure: "Lift LF from points to an ordered frontier": illustrative last column L = annb$na with a selected, scattered set of rows, zero-indexed; steps Input, Map, Partition.]

The distinction from a collection of independent LF lanes is that the frontier is continually restored to global row order. Its locality is a mathematical consequence of the representation, not merely a software scheduling technique.

The earlier class-fact objection was that sufficiently long context keys shatter into singleton classes. But this construction does not require two identifiers to share their entire context. It only requires collective order to be recoverable at each successive transformation.

Randomness does not invalidate the ordered-frontier lemma.

### 5. Turn the random fields into a collective object

Suppose we can acquire the rows immediately following a collection of fixed-length fields. Call these rows anchors. Let there be G anchors, with fields of length d. Construct an ordered frontier containing a record for each anchor, initially carrying an empty payload.

At each collective step:
- Acquire the emitted symbols and LF destinations of the frontier.
- Append the symbols to the associated payloads.
- Stably partition the records by character.
- Continue with the resulting ordered frontier.

After d steps, we have simultaneously computed (a_i, LF^d(a_i), w_i), i = 1, …, G, where w_i is the emitted field word in traversal orientation.

These are word-carrying jumps across entire fields. But they were acquired by synchronised transport of a scattered collection, rather than by following each field's orbit separately.

[Figure: "Two orientations of the same inversion problem": the columns represent logical phases; within each phase, the actual records are reordered into BWT-row order by stable partition.]

This potentially changes which quantity matters. The earlier formulation treats random field characters as individually irreducible. The alternative considers:
- G fields;
- d synchronous phases;
- the memory footprint of a frontier;
- the cache lines required to process its ordered records;
- the cost of acquiring anchor rows.

There is still G·d-scale character work. We cannot make the information in the random fields disappear. But we may be able to change the structure of the memory accesses necessary to transport that information.

And importantly, once the field jumps have been constructed, there need only be one jump per field when subsequently assembling those fields into the full text.

### 6. Why this may be more than simply batched LF

We need not acquire the absolute output position of every GUID before constructing its contents. Suppose the field paths are disjoint along the marked orbit. Each can be replaced by a macro-edge carrying its complete word: a_i →(w_i)→ LF^d(a_i). All internal field rows can then be removed from the residual traversal.

This gives a possible two-level inversion architecture:
1. BWT run records.
2. Acquire field anchors (native recognition is an open obligation).
3. Ordered-frontier transport: construct all field payloads and macro-edges collectively.
4. Residual exchange: contracted field paths plus remaining non-field content.
5. Full text assembly: field-sized jumps rather than digit-sized traversal.

The direction of discovery is almost the reverse of the previous major-shot assumptions.
- Previously: random content is the presumptively irreducible part; seek global positional structure on non-random content.
- Here: random content may possess the most regular transport domain, precisely because its formatting creates many synchronised, similarly shaped fields.

It is the distinction between regularity of values and regularity of the operations performed on those values. This is the potential Eureka.

The width-one wall does not preclude collective ordered transport. Context shattering does not preclude it either. The absence of repeated payloads does not preclude it. And absolute text coordinates are not a necessary prerequisite for the proposed construction.

### 7. Does the cost element have teeth?

This is where we must be adversarial towards our own insight.

Record 96 reports between 627 and 1,903 GUIDs across the six admitted blocks. With 30 fully random nibbles each, that gives approximately 18,800–57,100 random-nibble rows. If one frontier record occupies 24–32 bytes, the active frontier has an illustrative footprint of roughly 15–61 KiB. Two frontier buffers would occupy roughly 30–122 KiB, before rank support and other state. That is at least suggestive of a different working-set scale.

[Figure: "Change in the unit of traversal": an illustration using 1,000 GUIDs, each with 30 random nibbles. Individual random-nibble rows: 30,000. Active field-frontier records: 1,000. Synchronous random-nibble phases: 30. These are different structural quantities, not equivalent work counts. All 30,000 nibble values still need to be recovered.]

However, there are serious unresolved costs. Computing rank for every row might still incur expensive memory accesses. Scanning all run records for every phase would probably violate the proposed three-pass budget. Moving field identifiers and accumulated payloads through successive bucket partitions may introduce considerable traffic. Integrating the resulting macro-edges into Open may itself be expensive.

And obtaining the anchors natively is not automatic. FM-index backward search may locate intervals associated with recognised following contexts, but distinguishing true field occurrences from other matches and acquiring an economically useful anchor set requires proof and measurement.

So this is a candidate mathematical capability, not an established competitive algorithm. Batched LF techniques already exist, and I have not established novelty for this particular combination. Nor have I benchmarked it.

Its importance is that it supplies a new question which can be attacked using the Mario method.

### 8. The supplied-state ladder

The first experiment should not attempt to solve anchor discovery. Supply the anchors. Then progressively discharge what has been supplied.
1. **Supplied anchor set.** Give the decoder the exact rows immediately following the fixed-length random fields, without supplying their payloads. Question: can it acquire the fields collectively?
2. **Native ordered transport.** Compute and stable-partition the frontiers using the actual run representation. Question: does the acquisition cost less than the displaced traversal?
3. **Contracted assembly.** Integrate the field-sized macro-edges into a residual inversion algorithm. Question: does the whole decoder pay, including glue?
4. **Native anchor acquisition.** Replace the supplied anchors with an exact recognition procedure. Question: can all prerequisites be discharged within the agreed budget?

If the supplied-anchor version cannot pay, we should not spend time constructing a sophisticated anchor recognition theorem. If it can pay, we have identified the next mathematical question.

That is the Archimedean Method combined with the Mario backward search.

## IV. The larger Eureka: research the transformations, not the results

I would not want the particular ordered-frontier idea to become the only takeaway from the essay. There is a much larger methodological proposition here.

We have been conducting research approximately like this: generate approaches from different mathematical worlds, then judge their results.

Hilbert's twenty-fourth problem suggests another research process: derive a generative theory from the relationships between already established algorithms.

In the second arrangement, we do not discard the kitchen sink. We give it a more precise job. Topology, algebra, geometry and combinatorics become languages in which to express transformations between computational witnesses.

The research subject is no longer just the BWT. It becomes the mathematics of changing the mathematics of BWT inversion.

### A possible twenty-fourth problem for mibz2

I would formulate a supplementary foundational challenge:

> Given two exact inversion algorithms with different structural cost laws, characterise the mathematical transformations that connect their computational witnesses. Identify which invariants are preserved, which assumptions change, and which computational obligations disappear. Find a generative transformation that produces a further cost law governed by a parameter substantially smaller on real inputs.

This is not a replacement for the yes/no major shot. It is a method for discovering its witness. It asks the agents to produce something richer than a collection of alternative algorithms. It asks them to establish a theory that can generate alternative algorithms. And that is, I think, the most powerful response to the essay.

## V. Against dead mathematics, but not against formal mathematics

I disagree with the author's ultimate conclusion. A mathematical proof can be machine-checked and deeply illuminating. An AI-generated result can expose a genuinely fertile mathematical mechanism. A human-authored paper can be completely devoid of transferable understanding.

The distinction is not fundamentally human versus machine. It is terminal mathematics versus generative mathematics. Terminal mathematics produces an answer and stops. Generative mathematics produces a principle from which further answers become possible.

What we are trying to achieve with mibz2 is not the accumulation of more certified facts about the inverse Burrows–Wheeler transform. Nor is it merely a faster implementation. We are trying to understand the permutation sufficiently deeply that a different computational mechanism becomes a consequence of that understanding.

The essay has supplied us with a remarkable historical articulation of how to approach that ambition: Hilbert's twenty-fourth problem, Archimedes' mechanical method, Thurston's account of mathematical understanding, and Hippasus's discovery of the transformation connecting the triangles. They all point in the same direction.

Stop asking the oracle for the seven hundred and twenty-third triangle. Discover the operation that produces the triangles.
