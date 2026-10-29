<!--
Provenance: the research chatbot's overview at the start of the literature study, relayed by
the user into the working session on 29 October 2026, unedited. The chat interface dropped
most inline mathematics when the text was copied, so some sentences have gaps where a symbol
stood (for example "If there are complete groups"); the gaps are kept as received. Links
appeared as their titles only.
-->

Where the mibz2 research programme stands

I think we’re at a particularly interesting stage. We’ve stopped asking whether clever compression techniques might help, and started asking which properties of the inverse BWT we can exploit without sacrificing the performance characteristics that already make mibz2 fast.

The programme now sits at the intersection of four subjects:

* Compressed data structures: How little information is needed to represent the navigation permutation?
* BWT combinatorics: Where do the predictable regions come from, and how are their boundaries distributed?
* Computer architecture: How quickly can the CPU recover the next pointer, particularly with 32 independent traversal lanes?
* Construction algorithms: Can we build the compact representation directly, without doing more work than our newly improved constructor?

The first two have produced substantial theoretical results. The latter two remain the main performance uncertainties.

The knowns, unknowns and current hypotheses

Topic	What we know	What remains open
Ψ structure	Ψ consists of increasing occurrence lists for each character	Which locally compressed representation is fastest
Translation groups	Reported full-block coverage is ~77.4% for aligned groups of 32	Whether coverage remains high outside our corpus
Discontinuities	Their positions can be derived exactly from BWT run information	How their distribution relates to input characteristics
Fixed 64-byte records	Corpus census strongly supports format feasibility	Actual navigation latency and construction cost
Two-tier directory	Translation groups can return the next pointer from one descriptor	Whether extra loads for other groups destroy the advantage
Construction	#81 improved cooperative build by 31.3% and build+walk by 10.2% in the reported 3×3 experiment	Whether directly constructing compact Ψ can beat that stronger baseline
Cooperative traversal	Independent lanes can overlap pointer-chasing latency	Whether compressed slow paths defeat that overlap

The crucial distinction is between representational feasibility, which is now strongly supported, and performance superiority, which remains unproven.

And the literature has become more useful precisely because we can now ask it specific questions.

1. The most interesting theoretical problem: why do the breaks cluster?

Our recent research established that runs of identical characters in the BWT become consecutive stretches in Ψ.

That part is straightforward.

What is less straightforward is the distribution of the boundaries between those stretches.

Imagine two BWTs with the same number of runs. Their Ψ permutations could have very different proportions of translation-eligible 32-row groups, depending on how those run boundaries align.

The natural mathematical object is therefore:

K_{32}

\text{number of 32-row groups containing an internal Ψ break}.

If there are complete groups, translation coverage is exactly:

1-\frac{K_{32}}G.

Known

We can calculate exactly from the character-specific BWT run lengths, their cumulative occurrence ordinals and the necessary F-boundary information.

This is a deterministic structural characterization.

Unknown

We don’t know whether typical BWTs exhibit a statistically predictable distribution of based on simpler properties of the input.

For example:

* Does the average BWT run length predict group occupancy?
* Do runs of different symbols have markedly different ordinal-boundary patterns?
* Are breaks concentrated near particular alignments modulo 32?
* Does high repetitiveness produce clustered breaks, or mainly fewer breaks?

These questions should not be conflated. Two sequences with identical run counts and lengths in aggregate can have different fixed-group occupancy.

Where literature could help

The most relevant existing theory concerns runs in compressed permutations and run-length BWT indexes.

One especially relevant work is Locally Compressed Suffix Arrays, which studies consecutive-value runs in Ψ and how they relate to compressible text. ACM Journal of Experimental Algorithmics

Rather than reading it for an encoding to copy, I’d read it for its structural results: what does the repetitiveness of the original text imply about intervals on which Ψ acts as a translation?

A useful research objective would be a statistical model for as group width varies. But I’d keep the exact run-endpoint calculation as our reference: any statistical predictor should be judged against it.

2. A newer research direction: move structures and complete permutation traversal

This is where the literature search turned up something especially significant.

NEWLY RELEVANT PAPER · SEA 2026

Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations⁠

Nathaniel K. Brown and Ben Langmead, 2026.

The paper studies compressed representations of permutations that consist of long translated intervals, including BWT-related permutations. It introduces length capping: splitting long intervals to obtain better average query guarantees and simpler construction than earlier balancing approaches. arXiv

This paper is unusually relevant to mibz2 because its authors explicitly distinguish the cost of isolated permutation queries from the cost of traversing a complete permutation cycle.

That’s exactly the distinction we’ve been grappling with.

Conventional compressed indexes usually optimize access to selected positions. mibz2 instead follows an enormous sequence of successor links, with many chains progressing concurrently.

The authors show that their length-capped move structures support constant amortized query time over a complete single-cycle traversal, using space proportional to the number of runs. arXiv

Why this matters

It suggests a theoretical distinction we haven’t fully exploited:

A data structure that is expensive for an arbitrary isolated lookup can still be efficient when the sequence of lookups forms a complete permutation traversal.

This doesn’t rescue the move-structure implementation that previously performed poorly in mibz2. Constant amortized time does not mean low dependency latency, and their workloads differ from our 32-lane decoder.

But the paper raises some genuinely new questions:

Could segment boundaries be chosen to make compressed interval navigation cheaper?

Can interval splitting reduce occasional long navigation paths without storing direct answers for every row?

Does a full-cycle amortized guarantee survive, or become more complicated, when we divide traversal among 512 segments and schedule them across multiple lanes?

Those questions deserve mathematical attention before we dismiss the entire family of run-based structures based on one negative experiment.

I would read this paper closely—particularly its treatment of single-cycle traversal, length capping and construction.

3. Cache-line codecs: the connection to actual hardware

The other research track is much more concrete.

The cacheline-ef implementation⁠ packs 44 sorted 40-bit integers into one 64-byte cache line, at approximately 11.6 bits per integer.

It directly supports indexed retrieval and prefetching.

The author explicitly identifies the benefit: one cache-line access rather than the multiple accesses associated with a conventional Elias–Fano representation. GitHub

That’s remarkably close to our original 32-values-per-line proposal.

But there are some fundamental differences:

cacheline-ef	mibz2
Sorted integers	Sorted Ψ occurrence lists with occasional boundaries
Index retrieval	Repeated dependent successor navigation
Integer values alone	Successor, output byte and segment marker
Fixed coding parameters	Highly variable group spans and translation eligibility
General indexing	High-throughput, one-shot decoding

What further literature study could establish

The question isn’t whether Elias–Fano works. It clearly does.

The useful questions are much more specific:

How many load instructions are needed to extract one value, even when all data resides in the same cache line?

Can low and high parts be loaded independently?

What does the compiler produce for in-register select, packed-field extraction and format dispatch?

Are there known CPU-specific alternatives to PDEP, especially when its latency or throughput is unfavorable?

This is where compression theory meets microarchitecture.

The best representation may have slightly worse compression but a shorter path from the current Ψ row to the next memory address.

4. Construction has become a first-class theoretical problem

The #81 result made this more urgent.

Our 32-bit constructor is now significantly faster, so compact representations must compete with a stronger baseline.

The mathematical question is:

Can compressed Ψ be emitted directly in a parallel pass over L, without materializing an uncompressed permutation or introducing expensive coordination?

We know a sequential scan naturally generates monotone occurrence lists.

But parallel construction changes the problem.

Each worker sees a portion of L, while the destination groups are organized by character occurrence ordinal.

That connects to a well-studied family of algorithms: parallel counting, stable distribution, prefix sums and histogram-based scatter.

The relevant literature is not only succinct data structures. It also includes high-performance radix sorting and parallel stable partitioning.

A particularly interesting possibility is to separate construction into:

* Counting and determining each worker’s occurrence ranges.
* Assembling complete compressed groups with clear ownership.
* Handling the relatively few groups crossing worker boundaries.

The challenge is to do this without introducing an extra full scan or enough synchronization to undo the benefit.

This is worth deeper algorithmic investigation, but we currently lack the detailed traffic and synchronization measurements needed to identify the winning construction strategy.

5. Traversal-order clustering remains the hardest open question

The previous research established a way to count consecutive slow-path visits.

Let contain the Ψ rows whose fixed groups require additional payload accesses.

Then:

P(\text{slow next}\mid\text{slow now})

\frac{|S\cap\Psi^{-1}(S)|}{|S|}.

That is an exact statistic if all Ψ rows are visited once.

But performance depends on more than adjacent steps within one chain.

Consider two situations.

In the first, each of 32 lanes encounters a slow record occasionally, at different times.

In the second, many lanes reach slow records in the same execution window.

Both may have the same proportion of slow steps and even similar adjacent-step correlations.

Yet their performance could differ because the CPU has a finite capacity for outstanding memory operations.

What is known

The permutation determines the logical order of visits, and the segment starts determine how that order is split across lanes.

The one-step slow-to-slow statistic is computable from Ψ’s structure.

What isn’t known

There is no universal rule forcing slow accesses to be independent across lanes.

Their timing also depends on memory latency, format-dependent decoding costs and scheduling.

Where literature could help

This suggests looking beyond compressed text indexing to:

* Memory-level parallelism in irregular graph traversal.
* Parallel pointer chasing and latency-hiding techniques.
* Variable-latency execution under interleaved scheduling.
* Cache-aware and cache-oblivious permutation traversal.

The closest performance analogy may be graph navigation rather than ordinary compressed indexing.

In both settings, the next address depends on the result of the current access, and parallelism comes from having independent chains in flight.

We should be careful about transferring results, though. A generic graph traversal often has irregular degrees and branching, while Ψ is a permutation with one successor per row.

That extra structure could matter.

The literature I’d prioritize next

I would organize the reading around specific unanswered questions rather than around data-structure families.

Highest priority · Recent theoretical development

Brown & Langmead — Bounding the Average Move Structure Query (2026)⁠

Read for full-cycle amortization, interval splitting, construction complexity, and the relationship between run structure and repeated navigation.

Our question: Can any of these results be adapted to a segmented, MLP-oriented inverse-BWT walker?

High priority · Ψ-specific compression

Locally Compressed Suffix Arrays

Study its treatment of consecutive-value runs in Ψ and their relationship to text repetitiveness. ACM Journal of Experimental Algorithmics

Our question: Can the distribution of those runs explain the observed occupancy of fixed 32-row groups?

High priority · Engineering baseline

Groot Koerkamp — cacheline-ef⁠

Study the actual decoding operations, record layout and prefetch interface.

Our question: How close can a compressed pointer lookup get to the dependency cost of one direct table load?

Important complementary work

RLBWT Tricks⁠

Investigates practical move structures, compressed permutation navigation and space–time trade-offs. PubMed Central (PMC)

Our question: What practical factors make interval-based navigation expensive, and can direct fixed-group addressing avoid them?

A useful principle for the next literature phase is to distinguish three different kinds of evidence.

Theorems can tell us which representations are possible and what their worst-case or amortized complexity must be.

Algorithm descriptions can tell us how to construct and navigate those representations.

Performance experiments can tell us whether the resulting memory accesses and instructions suit modern processors.

None of those substitutes for the others.

My current research position

The most credible practical candidates remain the 24-bit direct table, the fixed-line 16-bit hybrid, and the much smaller two-tier translation directory.

But I wouldn’t broaden that list yet.

The greatest opportunities for deeper theoretical research are now:

* Run-boundary occupancy: Can we develop a useful model for translation-group coverage from the statistical properties of BWT runs?
* Full-traversal compressed navigation: Can recent move-structure results tell us something about segmented Ψ traversal that isolated-query complexity misses?
* Construction complexity: Is there a parallel algorithm that builds the compact representation directly without sacrificing the #81 fusion improvement?
* Execution behaviour: What structural properties predict when extra dependent memory accesses become expensive under 32-lane interleaving?

I would give the second question more attention than we previously did, especially in light of the 2026 Brown–Langmead paper. It’s unusual to find a recent result explicitly about amortized navigation over complete BWT-related permutation cycles when that is so close to the operation mibz2 performs.

But we should remember that the theoretical promise of move structures and their measured performance in your decoder have already diverged once.

The fundamental research question remains:

How much of Ψ can we reconstruct from compact structural information while still letting dozens of independent traversals progress almost as quickly as they do through a flat pointer table?

We now have a sound mathematical foundation, encouraging corpus evidence, a stronger construction baseline, and some genuinely relevant modern literature.

What we don’t yet have is a proof—or a measurement—that a compact representation can preserve the advantage of mibz2’s cooperative execution.

That is the frontier.
