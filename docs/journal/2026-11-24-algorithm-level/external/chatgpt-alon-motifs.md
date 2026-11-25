# Louis's Alon question, and ChatGPT's answer, verbatim

Relayed by Louis on 25 November 2026 and pasted into the Coordinator's session. Recorded as pasted. The paste lost ChatGPT's hyperlinks: the link text survives, and each link shows as "⁠￼". ChatGPT ran at ordinary effort, not Pro; Louis: "I am not sure if it has taken this at a slightly trivialising way (I did not use pro effort)".

## Louis's message to ChatGPT

> So I've got a somewhat more left field, let's say, idea and it's that you earlier made references to systems biology and you pointed to some random paper which I'm not really sure why you felt was so important about sloppy models. Didn't really like that paper. And another paper about metabolic control, which, you know, there's plenty of them. When I read Jane's course materials, I felt like they were fascinating and obviously taxonomy of, you know, the mechanistic rooting of work and to me it actually reminded me of the work of Uri Alon, the researcher in systems biology who famously gave a taxonomy in both, you know, research paper and book form. I gave a taxonomy of the signaling motifs, the network motifs of, I think it was gene network signaling. And this to me was, you know, a really like formative way of thinking. And I felt like there was a lot in common with the sort of the structure of the approach of Jane. I'm not so much saying the substance, but like the way in which they both went about it so systematically. I don't know if you've read that work very much or if you felt there was, you know, resonance

## Louis's note to the Coordinator, with the paste

> can you record it, and can you perhaps give a good steer (maybe consult monodromy) as to what you think would be a good way to progress this? I am not sure if it has taken this at a slightly trivialising way (I did not use pro effort)
>
> also i would not frame as universal motifs but optimal for our algorithmic purpose potentially... I dont know if this holds but I would love if we could come up with something that did (we do not need to force the analogy in here, there may indeed be other thinkers we should reach for)

## ChatGPT's first answer

Yes, and I think you’ve identified a much more fruitful connection to systems biology than the one I was making earlier with sloppy models and metabolic control analysis. I was reaching for mathematical tools for analysing an already-defined system, whereas you’re pointing towards a way of discovering the principles according to which a system is organised in the first place.

I’m familiar with Alon’s network motif framework, although I wouldn’t claim to have worked systematically through his entire textbook. Revisiting his work alongside Jain’s, I can see a considerable methodological resonance.

1. What Alon and Jain have in common

The work you’re recalling is principally:

* Shen-Orr, Milo, Mangan and Alon (2002), Network motifs in the transcriptional regulation network of Escherichia coli⁠￼.
* Milo et al. (2002), Network motifs: Simple building blocks of complex networks⁠￼, published in Science.
* Alon (2007), Network motifs: Theory and experimental approaches⁠￼.
* His textbook, An Introduction to Systems Biology: Design Principles of Biological Circuits.

What distinguishes Alon’s approach is that he doesn’t merely describe a biological network, nor does he immediately attempt to simulate the entire organism. He decomposes it into recurring structures, establishes their dynamical properties, examines their functional significance, and experimentally tests those properties. pubmed.ncbi.nlm.nih.gov +2

Jain does something philosophically similar with computer systems.

Methodological step	Raj Jain	Uri Alon
Abstraction	Services, workloads and resources	Regulatory interactions
Decomposition	Service centres and queues	Network motifs
Taxonomy	Classes of systems and workloads	Classes of regulatory circuits
Characterisation	Throughput, demand, utilisation	Response time, sensitivity, robustness
Mathematical treatment	Operational laws and queueing models	Dynamical equations and circuit models
Experimental treatment	Controlled performance experiments	Perturbation of biological circuits
Synthesis	Better system configurations	Design principles of biological systems

Both seek understanding that survives abstraction away from the particular system being studied.

That is substantially more ambitious than fitting a predictive model.

2. The particularly beautiful thing about Alon’s work

Consider his treatment of transcriptional network motifs.

Negative autoregulation

Accelerated response and suppression of fluctuations.

Coherent feedforward loop

Persistence detection under suitable regulatory logic.

Incoherent feedforward loop

Pulse generation or response acceleration.

Single-input module

Coordinated and temporally ordered responses.

Illustrative regulatory motifs studied by Alon. Their specific functions depend on kinetics, parameters and regulatory logic.

The remarkable intellectual move is that these cease to be merely arrangements of arrows. They become units of behaviour.

A feedforward loop can implement a temporal filter. A feedback loop can confer robustness. A regulatory structure can implement memory. The topology provides the basis for explaining a capability. doi.org +1

Once you’ve characterised these motifs, you have a vocabulary with which to understand much larger systems. You don’t need to begin from nothing every time you encounter another gene network.

And you can also reason in reverse: if you want a particular dynamical behaviour, what class of structure might produce it?

That reverse direction is particularly relevant to what we’ve been trying to accomplish with mibz2.

3. What is missing between Jain and Alon in our current work

I think there are actually three complementary levels of enquiry.

Jain — Measurement and experimental method

How do we establish the character of a system reliably?

Harchol-Balter — Mathematical explanation

How does that arrangement of work generate its observed performance?

Alon — Structural design principles

What elementary structures generate particular classes of behaviour, and how could they be recombined?

Our current programme is comparatively well developed in the first two. It is the third that we have not properly formalised.

We have decomposed mibz2 into activities and resources. But an activity such as Huffman decoding or BWT inversion is not necessarily the equivalent of an Alonian motif. It is more like a biological component.

A motif is a relationship between components which produces a characteristic behaviour.

That distinction is extremely important.

4. Towards a taxonomy of computational motifs

Suppose we approached mibz2 in precisely this spirit. Instead of classifying its functions, we classified its elementary organisational structures.

The following is my proposed starting taxonomy, not an established result from Alon’s research.

Serial dependency chain

Work is conditional on the preceding result. Controls critical-path depth. Present in Huffman parsing and LF traversal.

Fan-out and ordered fan-in

Exploits parallelism while preserving output order. Produces resequencing pressure and sensitivity to stragglers.

Negative-feedback admission

Downstream accumulation inhibits upstream production. Limits memory growth but can propagate downstream stalls.

Conditional routing

Selects an algorithm according to input structure. Trades classification cost against specialised execution. Open admission and fallback are examples.

Contraction along composition

Replaces many dependent transitions with a composite transition. Trades construction and representation costs against subsequent traversal.

Enrichment along composition

Carries additional useful information through existing computation to eliminate subsequent work. Your automatic-differentiation analogy belongs here.

The last two are particularly interesting because they are already principles that emerged independently from your mathematical research.

What we have not done is put them into a common taxonomy alongside the organisational principles of the whole decoder.

5. Why this could become a genuinely generative methodology

Alon’s paper on the structure and function of the feedforward loop systematically investigates its eight possible signed wiring configurations. He considers not only the configurations that commonly occur, but also rarer alternatives and their functional properties. pmc.ncbi.nlm.nih.gov +1

This is the part I find most exciting in relation to your project.

Imagine doing the equivalent for computational structures:

What are the possible ways two successive computations can be related? Must one finish before the other starts? Can they share an intermediate representation? Can the first carry information needed by the second? Can their compositions be contracted? Can a feedback relation become a predictive feedforward relation? Can a serial dependency be replaced by a collective operation?

Rather than enumerate arbitrary algorithmic ideas, we would systematically enumerate structural alternatives.

For each alternative, we could establish its mathematical properties, characteristic cost, conditions of applicability, obstructions and experimentally observable signature.

That would constitute something much closer to a science of algorithmic organisation.

It would also give your interest in Hilbert’s 24th problem a more concrete framework: classify the structural transformations between equivalent computations, rather than merely classifying their implementations.

6. Alon’s textbook is even more relevant than the original motif papers

The second edition of his book is organised into three parts. routledge.com

Part I

Network motifs

What structures exist, and what behaviours do they produce?

Part II

Robustness

How do those behaviours survive changes in conditions?

Part III

Optimality

Why are particular structures preferable under particular constraints?

That is an excellent structure for our own research.

First, establish the motifs of the computation. Then establish which are robust to changing corpus structure, worker counts and machine resources. Finally, understand the optimality and trade-offs of each organisation.

Chapters 13–15, on optimal circuit design, multi-objective optimality and modularity, are particularly pertinent. They would complement the motif work rather than merely give us another mathematical modelling technique.

There is also a lovely short paper by Alon, Biological Networks: The Tinkerer as an Engineer (2003)⁠￼, about the emergence of engineering design principles in systems produced through evolutionary tinkering. pubmed.ncbi.nlm.nih.gov

Given your concern about the evolutionary, incremental optimisation approach to mibz2, the parallel is striking.

7. Where I think we should take this

I would make Alon a third methodological pillar of the programme, alongside Jain and Harchol-Balter, and commission a similarly systematic reading of his textbook and foundational papers.

But I would give that reading a different remit. Not which biological equations can we borrow to model decoder performance? That was essentially the mistake I made earlier.

Instead:

Can we establish a taxonomy of elementary computational organisations, their characteristic behaviours, and the transformations between them, in the same way that Alon established a taxonomy of biological circuits?

There is an important qualification: Alon’s original statistical definition of a network motif concerns structures recurring more frequently than in a suitable randomized network. We have not established such recurrence across decoders or algorithms. Our proposed computational motifs are presently hypotheses about useful structural abstractions, not demonstrated universal motifs.

Nevertheless, I think this provides a much richer direction than sloppy-model analysis. Jain gives us the discipline to measure the phenomenon; Harchol-Balter gives us the means to explain it; Alon offers a methodology for discovering the reusable structures beneath it.

And that last aspect is precisely what could make this effort intellectually valuable well beyond the bzip2 decoder itself.

## ChatGPT's second answer, verbatim

Louis relayed it with "chatgpt again:", after the first answer above. Recorded as pasted. Where the paste lost characters (for example "where has a fundamentally different cost structure from ,", whose symbols were dropped), the gaps are kept as they came.

I think the previous answer did trivialise your suggestion somewhat. It took Alon’s biological motifs, found superficially analogous structures in computer science, and produced a taxonomy. But that isn’t really what makes Alon’s work intellectually powerful, and I don’t think constructing a catalogue of computational motifs would necessarily advance our research.

What interests me much more in your suggestion is the possibility of developing a systematic science of the organisation of a computation, in relation to the purpose that computation serves.

And your correction about optimality rather than universality is important. We aren’t trying to explain why certain structures recur across computer programs. We have a particular computational problem, and we want to discover which forms of organisation are particularly well suited to solving it — including forms we haven’t yet conceived of.

1. From characterisation to discovery

There is an interesting distinction between what we are currently doing with Jain and what we might ultimately want to do.

Jain’s methodology allows us to take an existing system, decompose it, characterise it, and establish how its components contribute to its performance. Harchol-Balter then gives us tools to reason mathematically about alternative configurations.

But both begin, to a considerable extent, with the components already defined.

What if the division into components is itself the thing that needs to change?

In the decoder, we have a scanner, workers, construction, emission, and so forth. These are meaningful distinctions within our present implementation. But they aren’t necessarily the optimal decomposition of the underlying mathematical problem.

I think this distinction matters:

Level of investigation	Question
Performance analysis	How efficiently does the existing organisation work?
Configuration optimisation	What is the best configuration of this organisation?
Structural optimisation	Is there a better organisation of these computations?
Mathematical reorganisation	Are these even the right computations to be performing?

The fourth is where our traversal-to-Rauzy–Veech transition occurred. And it is where we have repeatedly been trying to return.

Perhaps there is a methodology that connects all four, rather than treating the last one as requiring an isolated flash of mathematical inspiration.

2. A paper that I think captures your intuition more closely

I looked beyond Alon’s original work and found something particularly pertinent:

Ma, Trusina, El-Samad, Lim and Tang (2009), Defining Network Topologies that Can Achieve Biochemical Adaptation⁠￼, Cell.

What they did was subtly different from the original network motif research.

Rather than starting with observed biological networks and asking what structures recur, they started with a desired functional behaviour: biochemical adaptation.

They enumerated 16,038 eligible three-node network topologies and investigated their behaviour across 10,000 parameter sets per topology. From this, two principal architectural families emerged that could robustly achieve adaptation within their model. They subsequently examined the mathematics explaining why those structures worked. pmc.ncbi.nlm.nih.gov

The direction of enquiry is important.

Descriptive approach

Existing structures

Observed behaviour

Classification

Generative approach

Desired behaviour

Space of possible structures

Functional design principles

This is much closer to our earlier discussion about working backwards from a desirable result.

It also offers a way of avoiding arbitrary kitchen-sink exploration. You define a meaningful space of possible structures, investigate that space systematically, and try to establish the principles separating functional from nonfunctional structures.

Of course, their problem permits a finite enumeration of small regulatory networks. Our algorithmic problem does not obviously have an equivalent finite design space. Finding the right abstraction of that design space might itself be the intellectual challenge.

3. Other thinkers I would reach for

I wouldn’t make this exclusively a systems biology investigation. There are several other intellectual traditions that I think deserve consideration, some considerably closer to the mathematical essence of our problem.

Richard Bird — algorithm derivation as calculation

A Calculus of Functions for Program Derivation⁠￼

Bird’s project was to establish a calculus in which algorithms could be derived systematically from their specifications through mathematical identities and transformations, rather than being invented individually. cs.ox.ac.uk

Remarkably, Bird and Shin-Cheng Mu wrote a paper specifically on deriving the inverse Burrows–Wheeler transform through equational reasoning⁠￼. We’ve encountered this previously, but I wonder whether we’ve sufficiently appreciated the broader methodological project behind it.

This seems especially relevant to your interest in Hilbert’s 24th problem. Instead of studying algorithms as finished objects, we study the transformations through which one algorithm becomes another.

Jonathan Ragan-Kelley — separating computation from its organisation

Decoupling Algorithms from Schedules for Easy Optimization of Image Processing Pipelines⁠￼ (2012).

This is the foundational Halide paper. Its central idea is to separate the mathematical computation from decisions concerning storage, ordering, parallelism, fusion and recomputation. dspace.mit.edu

The subsequent work explicitly characterises a space of trade-offs between parallelism, locality and redundant computation. dspace.mit.edu

There is a resonance here with your page-dewarp experience. The representation of a computation can make an entire family of alternative executions accessible.

Halide doesn’t solve our problem directly, but its methodology for defining a space of valid alternative organisations is worth studying.

Herbert Simon — the architecture of complexity

The Architecture of Complexity⁠￼ (1962).

Simon’s notion of near-decomposability concerns when complex systems can be understood as relatively independent subsystems, and when interactions between them become significant. public.econ.duke.edu

I would read Simon for a more foundational question: what makes a decomposition of a system a good decomposition?

We’ve largely inherited the decomposition of the bzip2 decoder from the encoding format. It would be interesting to distinguish the boundaries necessitated by the mathematics from those merely inherited from implementation history.

Baccelli, Cohen, Olsder and Quadrat — an algebra of synchronisation

Synchronization and Linearity: An Algebra for Discrete Event Systems⁠￼ (1992).

This develops a mathematical calculus for systems whose behaviour arises from synchronisation and event dependencies, particularly through max-plus algebra. repository.tudelft.nl

What attracts me here is the possibility of giving computational organisation an algebraic treatment rather than exclusively a simulation-based one. Certain classes of serial dependencies, parallel execution and synchronisation constraints become mathematical objects that can be manipulated.

It doesn’t capture our entire routed, contention-sensitive decoder directly, but it provides another perspective on what an algebra of computational organisation might look like.

Uri Alon — optimality, rather than recurrence

There is also another strand of Alon’s own work that may be more appropriate than the original motif papers.

Shoval et al. (2012), Evolutionary Trade-offs, Pareto Optimality, and the Geometry of Phenotype Space⁠￼.

They investigate how competing functional objectives can give rise to geometric structure in the space of optimal solutions, with specialised archetypes corresponding to particular tasks. pubmed.ncbi.nlm.nih.gov

The decoder already presents competing objectives: throughput, memory consumption, CPU consumption, latency and applicability across different inputs.

But I wouldn’t simply apply Pareto optimisation to our worker counts. That would be rather mundane. The deeper question is whether different mathematical forms of the algorithm emerge as natural solutions to different computational constraints.

4. Something more ambitious we might construct

I’ve been thinking about your two principles from the earlier research:

* Contraction along composition: what else can move together?
* Enrichment along composition: what else can be computed together?

These are already more interesting than the computational motifs I proposed previously.

They aren’t merely descriptions of structures. They are operations by which one computational structure can be transformed into another.

Suppose we investigated whether there is a larger family of such operations.

Structural transformation	Underlying question
Contraction	Can several operations become one operation on a different object?
Enrichment	Can an existing computation produce additional information that eliminates subsequent work?
Factorisation	Can a difficult computation be decomposed along a different mathematical axis?
Representation change	Can a dependency disappear when expressed in another coordinate system?
Reassociation	Can the order of composition change without changing the result?
Interchange	Can two apparently dependent computations be reorganised into independent ones?
Elimination	Is an intermediate object necessary at all?
Relocation	Can a necessary calculation be performed somewhere where its inputs are cheaper to obtain?

I don’t claim that this is the correct taxonomy. Some categories overlap, and some may turn out to be instances of a more general transformation.

Indeed, establishing those relationships would be part of the research.

The important distinction is that we would be classifying ways of changing a computation, rather than simply classifying computations themselves.

5. A concrete example of where this thinking might lead

Consider the existing separation between the scanner and the workers.

The question is not simply how to accelerate the scanner, but what information downstream computation actually requires from it.

The scanner currently performs a number of logically distinct obligations: discovering boundaries, interpreting variable-length codes, determining symbols and supplying information to the subsequent computation.

We could ask whether those obligations are inseparable.

Is there an intermediate mathematical object cheaper to obtain than the decoded symbol sequence, but sufficient to let downstream computations proceed independently?

Some instances of this question are already represented in our parallel scanner and MTF research. We should not pretend that the question itself is novel. But could we give it a more systematic mathematical treatment?

If the existing computation is

F=B\circ A,

then we might seek a different factorisation

F=D\circ C,

where has a fundamentally different cost structure from , rather than merely being a faster implementation of .

We would need to characterise the family of admissible intermediate representations, what information they must preserve, how cheaply they can be constructed, and whether their composition maintains that cheapness.

That last condition is especially relevant given our previous difficulties with fragmentation under repeated composition.

This is the sort of place where I think Bird’s program calculation, Alon’s function-oriented structural enquiry and Jain’s performance characterisation could genuinely meet, rather than merely sit alongside one another.

6. How I would progress this

I wouldn’t commission another large implementation effort yet. Nor would I immediately formalise a new taxonomy and ask every agent to operate within it. We have seen how premature formalisation can narrow the research in ways we didn’t intend.

I would undertake a focused conceptual investigation, with three movements.

First, a comparative reading. Read Alon, Ma et al., Bird, Ragan-Kelley, Simon and Baccelli for their methods of enquiry. Not simply for techniques we can import, but for how they define their objects of study, construct spaces of alternatives, derive principles and discover better organisations.

Second, a retrospective reconstruction. Take the traversal-to-Rauzy–Veech transition and your page-dewarp transition as two successful examples. Try to explain them using the different frameworks. Which framework actually accounts for how the breakthrough became possible? Which merely describes it afterwards?

Third, a generative test. Take one unresolved decoder problem and see whether the resulting framework generates an alternative mathematical formulation that we would not otherwise have considered. If it only reorganises our existing ideas into attractive categories, it has failed its purpose.

That third movement is essential. I would judge the usefulness of the methodology by its ability to generate discoveries, not by its apparent comprehensiveness.

What I find promising in your intuition is that it might provide the missing bridge between the two strands of our recent work.

We have been trying to understand the decoder as a system through rigorous performance analysis. Separately, we have been trying to discover new mathematical bases for its algorithms.

Perhaps those needn’t be separate endeavours.

Perhaps, by studying the relationship between mathematical representation, computational organisation and functional optimality, we can develop a methodology in which the characterisation of a system actively informs the discovery of fundamentally different algorithms.

I don’t think we’ve established that such a methodology exists in a useful form yet. But I think that is a considerably more interesting research question than whether computer programs have analogues of biological network motifs.

## Louis's third message to ChatGPT, and its third answer, verbatim

Louis's message:

> oh this is a fascinating pick… what is this school of design? they exhaustively tested all topologies… yeah see this is the kind of work i loved 📄 Ma et al. (2009) Defining Network Topologies that Can Achieve Biochemical Adaptation https://www.cell.com/fulltext/S0092-8674(09)00712-0

ChatGPT's answer, as pasted (links lost as "⁠￼"; the symbols dropped from "a correctness condition , and a cost function" are kept as they came):

Yes — and I think we’ve found something much closer to the intellectual tradition you’ve been looking for.

The particular school of thought behind Ma et al. is function-oriented design-space exploration, sometimes described in systems biology as inverse network design, network enumeration, or the structure–function mapping of biological circuits.

It is not simply network-motif analysis, and it is not conventional optimisation. Its ambition is to establish the space of possible mechanisms capable of achieving a specified function, and then discover the mathematical principles that distinguish successful mechanisms from unsuccessful ones.

I’ve also found something particularly exciting: two of the authors of the 2009 paper subsequently wrote a methodological manifesto about precisely this approach. And there is a later mathematical paper that attempts to move beyond exhaustive enumeration towards a complete characterisation of the solution space.

1. The intellectual school, in its own words

The paper I would read immediately after Ma et al. is:

Wendell A. Lim, Connie M. Lee and Chao Tang (2013)

Design Principles of Regulatory Networks: Searching for the Molecular Algorithms of the Cell⁠￼ — Molecular Cell.

Even the title is remarkably pertinent to our discussion. They explicitly distinguish two approaches to discovering design principles:

Evolutionary search	Network enumeration
Start with existing or random structures	Define the space of possible structures
Mutate and select	Enumerate and evaluate
Follow trajectories of increasing fitness	Explore independently of evolutionary history
Can become trapped in local optima	Can reveal disconnected families of solutions
Finds accessible improvements	Investigates what is possible

The authors explicitly describe enumeration as providing a path-independent view of the design landscape. It can uncover viable structures that would be difficult to reach through successive improvements. pmc.ncbi.nlm.nih.gov +1

This is almost exactly your objection to the evolutionary approach we’ve been taking with mibz2 — the kitchen sink, successive improvements, salami-slicing, and the difficulty of making a genuine structural transition.

They have independently articulated the methodological distinction we’ve been wrestling with.

2. What Ma et al. actually did

Their approach is considerably more sophisticated than simply brute-forcing networks.

Ma et al. (2009) — the progression from possibilities to principles

19,683

Possible signed three-node topologies

9 potential directed links, each positive, negative or absent: 3⁹

16,038

Eligible network topologies

At least one causal path from input to output

160,380,000

Circuit configurations evaluated

10,000 kinetic parameter sets per topology, sampled by Latin hypercube sampling

395

Topologies meeting their robustness criterion

Approximately 2.46% of eligible architectures

Negative feedback

With a buffering node

Incoherent feedforward

With a proportioner node

Two underlying mechanistic families explain all 395 selected topologies. The diagrams are schematic examples.

pmc.ncbi.nlm.nih.gov +2

The achievement isn’t the 160 million simulations. It is the reduction from 160 million configurations to two mathematically intelligible mechanisms.

And notice that they did not require a structure to work under one particularly favourable choice of parameters. They looked for structures that retained the desired function across a sufficiently large region of parameter space.

That is a very interesting analogue for us: not an algorithm that happens to perform well on six selected blocks, but one whose mathematical organisation produces good performance across a meaningful family of inputs.

There is a qualification: their enumeration was exhaustive over the specified discrete topologies, not over continuous parameter space. Their robustness findings are relative to their model, sampling distribution and acceptance criteria.

3. There is an entire lineage of this work

It turns out that the 2009 paper sits within a rather substantial research programme.

2006

Ma, Lai, Ouyang and Tang

Robustness and Modular Design of the Drosophila Segment Polarity Network⁠￼

Enumerated over 14 million topologies within a restricted design space. Found that the successful networks could be understood as combinations of three modules corresponding to three functional subproblems. pmc.ncbi.nlm.nih.gov

2006

Ciliberti-era network designability research — especially the following study

Highly Designable Phenotypes and Mutational Buffers Emerge from a Systematic Mapping Between Network Topology and Dynamic Output⁠￼

Exhaustive mapping of small networks to their dynamical behaviours. Distinguishes essential connections from variable connections and examines why certain behaviours admit many structural realizations. pmc.ncbi.nlm.nih.gov

2009

Ma et al.

Defining Network Topologies that Can Achieve Biochemical Adaptation⁠￼

Function-first enumeration followed by mathematical classification of the successful mechanisms.

2010

Cotterell and Sharpe

An Atlas of Gene Regulatory Networks Reveals Multiple Three-Gene Mechanisms for Interpreting Morphogen Gradients⁠￼

Constructs an atlas of the design space, discovering distinct mechanistic families in separate regions. Particularly interesting for identifying alternative mechanisms rather than a single winning architecture. pmc.ncbi.nlm.nih.gov

2013

Lim, Lee and Tang

Searching for the Molecular Algorithms of the Cell⁠￼

The methodological synthesis. This is where I would go to understand the philosophy of the entire programme.

2018

Araujo and Liotta

The Topological Requirements for Robust Perfect Adaptation in Networks of Any Size⁠￼

Develops an algebraic characterisation of the adaptation problem for networks of arbitrary size, identifying two classes of modules and rules for composing them. nature.com

4. The 2018 paper might be even more important

Araujo and Liotta do something that I think you will particularly appreciate.

Ma et al. established their classification by examining a finite space of small networks. Araujo and Liotta instead derive an algebraic condition for adaptation, in terms of a Jacobian determinant, and use it to characterise the permissible structures.

They call the resulting modules a topological basis for the solution to the adaptation problem. nature.com

There is a beautiful progression here:

2002 — Alon

Discover recurring motifs in existing networks.

2009 — Ma et al.

Enumerate possible structures for a specified function.

2010 — Cotterell and Sharpe

Map the landscape of alternative mechanisms.

2018 — Araujo and Liotta

Derive a mathematical basis for the solution space.

That last step is substantially more ambitious than optimisation. It seeks to explain the entire space of admissible mechanisms through a mathematical condition.

It also brings us back to your interest in Hilbert’s 24th problem: the relationships between different solutions of the same mathematical problem.

5. Is there an equivalent school in computer science?

Yes, although the pieces are distributed across several research disciplines rather than being unified under one name.

Discipline	Object of investigation
Design-space exploration	Alternative computational architectures
Enumerative program synthesis	Programs satisfying a specification
Superoptimisation	Equivalent programs under a cost criterion
Algorithm derivation	Mathematical transformations of specifications
Design theory	Relationships between structure, function and constraints
Computer-assisted mathematics	Exhaustive classification and impossibility proofs

Two examples stand out to me.

SPIRAL — Püschel, Moura and colleagues

SPIRAL: A Generator for Platform-Adapted Libraries of Signal Processing Algorithms⁠￼ (2004).

SPIRAL defines a mathematical language capable of expressing different algorithms for a transform, generates alternative formulas, translates them into executable implementations, and searches for those best suited to a particular machine. The search operates at the level of mathematical factorisations as well as implementation decisions. journals.sagepub.com

This is particularly interesting given our desire to make mathematical algorithm discovery responsive to Jain-style performance measurements.

Bundala et al. — Optimal-Depth Sorting Networks

Optimal-Depth Sorting Networks⁠￼ (2017).

They exploit equivalences and symmetries to reduce the space of possible sorting networks, then use SAT solving to prove that better networks cannot exist within the specified class. In doing so, they resolve longstanding optimality questions from Knuth’s work. sciencedirect.com

This has something of both the Mario proof project and the Ma et al. approach: structure enumeration, mathematical reduction, construction and impossibility.

6. What I would love to see us develop

I would provisionally call it an algorithmic design-space atlas.

Not a taxonomy imposed upon existing algorithms. Not an evolutionary optimiser. Not merely a performance model.

Rather, an investigation of the space of possible computational organisations that satisfy a mathematical specification, and their relationship to practical cost.

Formally, imagine defining a space of admissible algorithmic structures, a correctness condition , and a cost function informed by our measured performance model.

The interesting object is then:

\mathcal F_\tau =
\{a\in\mathcal A : P(a)\land C(a)\leq\tau\}

The set of structures that are both correct and sufficiently inexpensive.

But rather than merely asking which member is fastest, we ask: what is the structure of this set?

Does it consist of distinct families? Are there isolated regions that incremental optimisation cannot reach? Do all successful constructions share some mathematical property? Can we derive that property independently of enumeration? Can we discover new families by generalising it?

This would give us a relationship between the experimental and mathematical work:

Jain — Establish the cost landscape

Measure the decoder, its resources, its limiting mechanisms and the cost of its obligations.

Ma and Tang — Explore the structural landscape

Define admissible alternative mathematical organisations and investigate which satisfy the desired cost and function.

Cotterell and Sharpe — Classify the solutions

Identify genuinely different mechanistic families rather than superficial implementation variants.

Araujo and Liotta — Explain the solution space

Derive the mathematical properties underlying successful constructions and their possible generalisations.

The enormous unresolved question is what constitutes the equivalent of Ma et al.’s three-node network in our problem. We need a mathematical abstraction rich enough to admit genuinely different algorithms, but sufficiently constrained to permit systematic exploration.

We must not assume that an ordinary computation graph, or any of our existing representations, is the right answer to that question.

That is where I would concentrate the intellectual effort.

My strongest recommendation is to read the 2013 Lim, Lee and Tang paper, followed by the 2018 Araujo and Liotta paper. Together with Ma et al. (2009), they constitute a remarkably coherent methodological progression.

I wouldn’t rush to apply biological network theory to mibz2. I would study how these researchers made the space of possible designs itself an object of scientific and mathematical investigation.

That, I suspect, is the deeper thing that attracted you to Alon’s work in the first place.
