<!--
Provenance: the research chatbot's deep dive 3 (Ottaviano & Venturini, "Partitioned
Elias–Fano Indexes", SIGIR 2014), relayed by the user into the working session on
29 October 2026, unedited. Some inline mathematics survived the copy as LaTeX source (for
example "S’i=Si-i", which lost its subscripts: S'_i = S_i − i); the rest was dropped,
leaving gaps where a symbol stood (for example "where is the overhead of describing a
chunk"). Both are kept as received. Links appeared as their titles only, followed by the
source's site name. Tables arrived tab-separated, and the two inline figures as plain
lines; both are kept so.
-->

Literature deep dive 3 — Partitioned Elias–Fano Indexes

Giuseppe Ottaviano & Rossano Venturini · SIGIR 2014

Read the paper⁠

This paper turns out to be an unusually good continuation of our research. It tackles a problem almost identical in shape to one we’ve encountered in mibz2:

A sequence is mathematically easy to compress, but the most obvious compression scheme fails to exploit how unevenly its simple regions are distributed.

The authors’ solution is to divide the sequence into independently encoded chunks. Their deeper contribution is an efficient algorithm for deciding where the chunks should begin and end.

And there’s a particularly interesting surprise: they explicitly recognize consecutive-value chunks as requiring no encoded values at all.

That’s the same principle behind our translation records, published twelve years before our investigation.

1. Why ordinary Elias–Fano leaves compression opportunities unused

Suppose we have an increasing list of positions:

10,11,12,13,14,5000,5001,5002,5003,5004.

It contains two extremely dense groups, separated by one enormous jump.

Ordinary Elias–Fano chooses its encoding parameters using the length of the entire list and its overall numerical range.

The large jump makes that range wide. Consequently, even values inside the perfectly consecutive groups pay for a representation designed to accommodate the wider universe.

The authors’ observation is that global statistics conceal local regularity.

Their solution is conceptually simple:

One monotone list, two locally simple chunks

Chunk A

10, 11, 12, 13, 14

Base 10 + offsets 0–4

Chunk B

5000, 5001, 5002, 5003, 5004

Base 5000 + offsets 0–4

Each chunk is trivial when considered locally. Only the chunk boundaries and bases need separate representation.

Their actual design stores a first-level index of chunk boundaries and compressed second-level chunks.

They compare fixed-size chunks with variable-length chunks, where boundaries are chosen to optimize compression cost. pages.di.unipi.it

This is a direct conceptual ancestor of our fixed-32-row Ψ groups and two-tier descriptor proposal.

2. The first result that really matters to us: three chunk formats

The paper doesn’t insist on Elias–Fano for every chunk.

Instead, it uses three alternatives depending on local density.

Chunk type	Representation	Why
Ordinary sparse chunk	Elias–Fano	Efficient representation of increasing values
Dense chunk	Characteristic bitvector	Sometimes smaller than Elias–Fano
Completely filled chunk	No encoded payload	All values follow consecutively from the boundaries

That third case is especially important.

If a chunk fills its entire local universe, its values are determined by the chunk’s beginning and its length.

The authors explicitly say that no additional information is needed to encode its individual elements. pages.di.unipi.it

For mibz2, that’s precisely what we’ve been calling a translation group.

Our reported corpus coverage of approximately 77.4% means that this isn’t a rare special case in the representation we have been examining. It’s potentially the dominant case.

The paper therefore offers strong historical support for our decision to treat translation as its own format, rather than forcing everything through general Elias–Fano.

But there is a critical distinction: the authors optimize sorted posting lists for search-engine queries. We need to recover one unpredictable Ψ successor, then immediately use it to compute another address.

That changes the cost of looking up the chunk.

3. The real invention: choosing chunk boundaries is an optimization problem

This is where the paper becomes much more interesting than a catalogue of compression formats.

Suppose you’re allowed to divide a sequence at any position.

Each possible chunk has a storage cost:

C(i,j)=F+\text{bits needed to encode }Si\\ldots j-1

where is the overhead of describing a chunk in the first-level index.

A short chunk might be extremely cheap to encode internally, but you’d need many chunks, each paying .

A long chunk amortizes that overhead but may need larger offsets.

So there is a genuine trade-off.

How do we find the partition with the smallest total number of bits?

The authors turn it into a shortest-path problem.

Create a vertex for each possible boundary in the sequence. An edge from to means “encode elements through as one chunk.”

Give that edge weight .

Partitioning as a shortest-path problem

Schematic: vertices represent chunk boundaries, and edges represent candidate chunks with encoding costs. A least-cost path chooses the minimum-space partition.

The difficulty is that every starting position could connect to every later position.

There are quadratically many candidate chunks.

An ordinary dynamic programme would therefore be too expensive.

The authors’ key algorithmic result is that they can discard almost all those possibilities while keeping a guarantee of near-optimal compression.

They obtain a partition using only linear work in the number of input integers for any fixed approximation tolerance :

\boxed{C_{\text{chosen}}\le(1+\varepsilon)C_{\text{optimal}}}

with running time for fixed , where is the number of integers. pages.di.unipi.it +1

This is where the paper connects most strongly with the Brown–Langmead construction work we just studied.

Both papers eliminate an apparently expensive construction step through a better algorithmic organization.

Brown and colleagues remove repeated searches during interval balancing.

Ottaviano and Venturini remove almost all the candidate partitions from a shortest-path optimization problem.

The common lesson is that construction complexity can sometimes be eliminated without weakening the desired final representation very much.

4. How the linear-time approximation actually works

The authors start from a useful property of their chunk cost:

Extending a chunk cannot reduce the cost of encoding that chunk.

That monotonicity means that as a chunk grows, its cost crosses successively larger thresholds.

They exploit this by considering only representative endpoints whose costs cross thresholds such as:

F,\quad F(1+\varepsilon),\quad F(1+\varepsilon)^2,\ldots

Rather than consider every possible edge in the shortest-path graph, they keep edges that approximately represent each cost scale.

They then introduce a second restriction: sufficiently expensive chunks can be replaced by several shorter chunks at bounded additional cost.

Combining both ideas gives a small collection of candidate edges per position.

Finally, they use sliding windows to generate those candidates as the dynamic programme moves through the sequence.

The implementation does not need to materialize the original quadratic graph.

The proof shows that the two approximations multiply, so their parameters can be chosen to keep the total error below . pages.di.unipi.it

This is a genuinely elegant result.

The algorithm spends a little extra space to buy an enormous reduction in the search space of possible partitions.

And importantly, its guarantee concerns compressed size, not decoding latency.

That’s exactly where our own objective differs.

5. The experiments contain a lesson that could easily be missed

The authors compared ordinary Elias–Fano with fixed-size partitioning and near-optimal variable-size partitioning on two large web corpora, Gov2 and ClueWeb09.

Their results are substantial.

Representation	Gov2 total index size	ClueWeb09 total index size
Plain Elias–Fano	7.66 GB	19.63 GB
Uniform chunks	5.17 GB	17.78 GB
Near-optimal chunks	4.65 GB	15.94 GB

From the paper’s Table 2. These are complete inverted-index sizes, not mibz2 table sizes. pages.di.unipi.it

On Gov2, near-optimal partitioning saves approximately 39% relative to plain Elias–Fano.

Even uniform chunking saves about 32%.

So simply acknowledging local clustering is already enormously valuable.

The paper also reports that its near-optimal representation is competitive with the speed of unpartitioned Elias–Fano. Sometimes it is negligibly slower or even slightly faster, which the authors attribute partly to improved memory locality from the smaller representation. pages.di.unipi.it

That observation is remarkably close to our cache-residency hypothesis.

But there is a second, subtler result.

The authors conclude that fixed-size partitioning offers a useful alternative when construction time matters. Their uniform representation loses approximately 12% in compressed size relative to near-optimal partitioning but avoids the additional optimization work. pages.di.unipi.it

For mibz2, that might be the most important experimental lesson in the entire paper.

We are building temporary navigation tables during decompression. We don’t get to amortize a sophisticated compression optimizer over thousands of later search queries.

Our tables may be traversed only once.

That radically changes the economics of near-optimal partitioning.

A representation that is 12% smaller but significantly more expensive to construct may be a poor choice for a one-shot decoder.

6. Where the analogy with mibz2 breaks

Ottaviano and Venturini use a two-level index.

To access an arbitrary element, their implementation may first consult information about a chunk’s boundaries, then retrieve an element from the chunk.

That is fine for a search-engine inverted index.

But our inverse-BWT walker follows a chain:

j\rightarrow\Psi(j)\rightarrow\Psi^2(j)\rightarrow\cdots

The next query cannot even be formulated until the previous one is decoded.

Any additional dependent memory access becomes part of that serial chain.

For this reason, we have deliberately explored globally aligned 32-row groups:

g=j\gg5.

We can calculate the group number instantly, without a predecessor search.

That sacrifices some compression opportunities.

A dense cluster might begin halfway through a group, so our fixed 32-row boundaries may divide it awkwardly.

But predictable addressing could be worth more than the compression sacrificed.

This creates a fundamental difference between the papers’ optimization objective and ours.

Their objective	Our objective
Minimize compressed list size	Minimize build-plus-navigation time
Variable chunk boundaries allowed	Prefer arithmetic group addressing
Efficient random access and search	Extremely low successor dependency latency
Index used for many queries	Temporary navigation structure
Construction amortized over later use	Construction is part of every decode

The paper shows why adaptive chunking compresses better. It does not show that adaptive chunking is the right choice for mibz2.

7. The most exciting idea for mibz2: optimize total cost, not compression ratio

Here’s the research direction I would extract from the paper.

Ottaviano and Venturini assign every possible chunk a cost measured in bits.

For mibz2, the more relevant cost would include three components:

\boxed{
C_{\text{total}}

C_{\text{construction}}
+
C_{\text{navigation}}
+
C_{\text{cache pressure}}
}

This isn’t a directly measurable additive equation yet. It’s a conceptual objective: these terms interact, and we would need a hardware-aware model to assign them comparable costs.

But it suggests a potentially fruitful research question:

Could we choose a compact encoding locally, based not on which format uses the fewest bits, but on which format is expected to give the best build-and-walk performance?

For a group of 32 Ψ values, the candidate formats are already familiar:

* Translation: a base plus slot, if all values are consecutive.
* Frame-of-reference: a base plus small packed offsets.
* Elias–Fano: a monotone sequence using high and low parts.
* Raw: direct packed values for exceptional cases.

We might choose among them using a much simpler rule than the paper’s general-purpose partition optimizer.

That’s because our group boundaries are fixed.

The complicated partitioning problem disappears.

For each 32-row group, we can determine which formats are valid and choose one according to a known cost model.

In other words, we can borrow the paper’s local-adaptation principle without borrowing its variable-boundary indexing overhead.

This is particularly appealing after #81: a cheap local decision during construction is more plausible than a global compression-optimization pass.

8. An overlooked technique: turn increasing sequences into nondecreasing residuals

There’s one more detail from Section 4 that is worth preserving.

For a strictly increasing sequence , the authors discuss transforming it into:

S’i=Si-i.

Because , the residuals satisfy:

S’i+1\ge S’i.

In other words, subtracting the index removes the mandatory increase of one.

Why is this interesting?

Suppose:

S=500,501,502,503,504.

Then:

S’=500,500,500,500,500.

A consecutive translation becomes a constant sequence.

For a nearly consecutive list:

S=500,501,502,510,511

we get:

S’=500,500,500,507,507.

Now the residual is piecewise constant, with jumps only at discontinuities.

This transformation exposes exactly the structure we’ve been studying in Ψ. pages.di.unipi.it

A new perspective on our translation records

Rather than thinking of Ψ as an increasing sequence, define:

R(j)=\Psi(j)-j

within each symbol’s F interval.

Then:

\Psi(j)=j+R(j).

Whenever Ψ is a translation over an interval, is constant there.

This gives us a particularly clean interpretation:

Our compact Ψ problem is equivalent to representing a piecewise-constant displacement function.

That doesn’t automatically create a faster codec. A fixed 32-row translation descriptor already implements essentially this idea.

But it changes how we can analyze the data.

We could study:

* How many distinct residual values occur in each 32-row group.
* How often residuals change.
* Whether nontranslation groups contain only a few residual plateaus.
* Whether those plateaus admit cheaper representations than general FOR or EF.

That last question is a genuinely new hypothesis arising from this reading.

A group that is not wholly translation-eligible might still consist of just two or three short translations.

We have been treating it as a general monotone sequence requiring packed offsets.

Perhaps some nontranslation groups can be represented more cheaply as a handful of displacement changes.

However, the decoder would need to identify the correct plateau for an arbitrary slot, so compression savings would have to compensate for the resulting comparisons or bit operations.

This is a research possibility, not evidence for a new decoder format.

9. What we’ve learned from this paper

Local density matters more than global density.

A sorted sequence can be much more compressible after partitioning into locally homogeneous chunks.

Consecutive-value chunks need no individual encoded values.

This directly anticipates our translation-eligible Ψ groups.

Near-optimal compression partitions can be found in linear time.

The paper proves this by pruning a shortest-path formulation using monotone encoding costs.

Fixed partitioning remains attractive when construction matters.

The paper’s own conclusions explicitly recognize this trade-off.

The residual function may be a useful new research object.

Its changes correspond to translation discontinuities, and its constant stretches may expose additional structure in nontranslation groups.

10. The outstanding research questions

This paper gives us three focused avenues for deeper work.

First: residual complexity. Among the approximately 22.6% of rows in nontranslation groups on the measured full-block corpus, how many distinct displacement plateaus occur per group? Could a very small plateau representation cover most of them?

Second: hardware-aware local encoding. Is there a useful model for choosing translation, FOR, EF or another simple group format that accounts for decoding operations and construction cost, rather than bits alone?

Third: fixed versus adaptive boundaries. How much additional compression could variable boundaries achieve on our actual Ψ lists, and is that benefit remotely large enough to compensate for extra indexing complexity?

That third question is particularly interesting as a theoretical comparison even if we never intend to implement variable boundaries.

It would tell us how much compression we sacrifice to preserve direct group addressing.

My verdict

I think this paper is more directly applicable to our compact-Ψ work than the recent move-structure construction paper, although the latter is more surprising theoretically.

It establishes three things we should take seriously.

One: locally clustered increasing lists benefit enormously from locally chosen encodings.

Two: perfectly consecutive chunks can be represented without storing their constituent values.

Three: the time required to choose the partition is part of the representation’s cost—and fixed chunks can be a very sensible compromise.

The new research direction I find most exciting is not general variable-length partitioning.

It’s the residual representation:

\boxed{R(j)=\Psi(j)-j}

because it turns our navigation structure into a piecewise-constant function.

We’ve already learned that entire 32-row groups frequently have constant residuals. The next question is whether the groups that fail that test are still mostly composed of a small number of constant pieces.

If so, there may be a middle ground between a translation-only descriptor and full FOR/Elias–Fano decoding.

That’s the strongest new hypothesis I’d carry forward from Ottaviano and Venturini.

And it points us naturally toward our next paper: Gog, Navarro & Petri — Improved and Extended Locating Functionality on Compressed Suffix Arrays⁠. That work addresses Ψ itself, giving us a chance to see whether this residual, piecewise-translation perspective already appears in established compressed suffix-array engineering—and how those researchers handled the practical trade-off between compact representation and fast random navigation.
