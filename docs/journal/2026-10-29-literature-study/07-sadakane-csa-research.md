<!--
Provenance: the research chatbot's deep dive 5 (Sadakane, "New Text Indexing
Functionalities of the Compressed Suffix Arrays", Journal of Algorithms, 2003), relayed
by the user into the working session on 29 October 2026, unedited. The researcher says it
could not read the full 2003 article, only its abstract, secondary accounts and SDSL's
`csa_sada`; its claims about the original are therefore secondary. Some inline mathematics
survived the copy as LaTeX source; the rest was dropped, leaving gaps where a symbol
stood (for example "using local indices :"). Both are kept as received. The code excerpt
arrived as plain lines. Links appeared as their titles only, followed by the source's site
name. Tables arrived tab-separated and are kept so.
-->

Literature deep dive 5 — Sadakane’s compressed suffix array

Kunihiko Sadakane · New Text Indexing Functionalities of the Compressed Suffix Arrays · Journal of Algorithms, 2003

Original paper⁠

This is the historical foundation of much of the Ψ-specific compression work we’ve been reading.

And I think its most important contribution to our research is a change in how we think about the navigation table.

Ψ doesn’t have to be stored as a permutation of absolute positions. It can be stored as a set of locally increasing sequences, with enough checkpoints to reconstruct individual entries.

That sounds familiar now, but Sadakane was developing this approach more than twenty years ago.

One source qualification: I could verify the original paper’s publication and abstract, and read secondary technical accounts and the later SDSL implementation. I couldn’t inspect the full original 2003 article directly, so I’ll distinguish its documented results from later engineering details. ScienceDirect +2

1. Why this paper was a breakthrough

A suffix array is effectively a sorted directory of all suffixes in a text.

Ordinarily, it stores an integer position for every suffix. That’s expensive for large texts.

Sadakane’s approach was to replace much of that explicit information with a compressed representation of Ψ, the permutation connecting neighbouring text positions through suffix-array order.

The resulting structure could perform text-search operations without keeping the original text separately, and could also reconstruct substrings.

The paper established text extraction in time proportional to the extracted length plus a small initialization term, under its stated indexing and sampling assumptions. ScienceDirect

For mibz2, the fascinating part is that text reconstruction proceeds by repeated navigation through Ψ.

The same permutation that helps a persistent text index reconstruct substrings is the object our decoder needs to traverse efficiently.

2. How the compressed Ψ representation works

Consider a portion of a character’s Ψ list:

100,101,102,140,141,142,143.

Since its values increase, we can store the first value and the positive differences:

1,1,38,1,1,1.

Those differences are typically cheaper to encode than the full absolute values.

Sadakane’s classical representation uses compact codes for positive differences and periodic samples of absolute values to permit access without decoding from the beginning of the entire sequence.

For example, with a sample every 128 entries, the decoder can find a nearby absolute position and reconstruct the requested value by decoding subsequent differences.

The sample interval produces the usual trade-off:

* More samples cost more memory but reduce decoding work.
* Fewer samples save memory but increase the number of differences traversed.

Later comparative studies describe this sampled-difference method using γ-style codes; related treatments also discuss δ coding. Sampling intervals of 128 appear in practical comparisons. ResearchGate

The important distinction is that a compressed Ψ lookup is no longer a direct array access.

It is a reconstruction computation.

That was reasonable for a compact searchable index whose data is accessed repeatedly. For mibz2, every additional dependency matters because the recovered value becomes the next navigation address.

3. The connection to mibz2 is deeper than just compressing differences

Our current research has found that many groups of 32 Ψ values are not merely increasing—they’re consecutive.

That lets us replace a sequence of differences:

1,1,1,1,1,\\ldots

with the implicit rule:

\Psi(j)=base+(j\bmod32).

This is an important evolution of Sadakane’s approach.

Instead of encoding lots of small differences efficiently, we’re asking:

Can we avoid encoding the differences at all when they’re predictable?

And for the nontranslation groups, we’ve discovered a useful representation:

R(j)=\Psi(j)-j.

Consider:

\Psi=100,101,102,140,141,142,143.

Then, using local indices :

R=100,100,100,137,137,137,137.

Notice what happened.

A sequence of small increments and occasional jumps becomes a piecewise-constant function.

This gives us two mathematically equivalent ways to describe the structure:

* Sadakane’s perspective: store differences between increasing Ψ values.
* Our residual perspective: store constant displacements and the positions where they change.

The second may be better suited to direct indexed retrieval when entire stretches are constant.

But it introduces a new problem: to retrieve , we must identify which constant stretch contains .

That’s why our regular 32-row groups are attractive. They allow the group address to be calculated directly, rather than discovered through a predecessor search.

The original theory establishes compressibility. Our particular contribution is exploring how much of it can survive one-load, latency-sensitive navigation.

4. What later implementations actually changed

The SDSL library includes csa_sada, a practical compressed suffix array based on Ψ.

Its default representation is a compressed vector, with separate suffix-array samples, inverse-suffix-array samples and alphabet metadata.

The library provides both Ψ access and text-indexing operations built around it. GitHub +1

A revealing piece of its implementation is the suffix-array lookup procedure:

while (!m_sa_sample.is_sampled(i)) {
    i = psi[i];
    ++off;
}
return m_sa_sample[i] - off; // conceptually, modulo n

The actual implementation handles wraparound explicitly. GitHub

This is almost a miniature version of the problem we’ve been studying: navigation along Ψ to recover information without storing a full suffix array.

But there’s a critical difference.

In a persistent index, only some operations require the suffix-array value, and samples can trade memory for the number of navigation steps.

In mibz2, every output character advances the navigation permutation.

We cannot afford to reconstruct an individual next pointer by decoding a long sequence of previous pointers. That would add serial work to every step.

A second interesting SDSL detail is that it implements select_bwt(c,i) directly through Ψ:

\operatorname{select}_c(L,i)=\Psi(Cc+i-1).

That is precisely the select formulation from which our research began. GitHub

So the connection isn’t merely conceptual: the modern implementation exposes Ψ as the fundamental operation for selecting occurrences from the BWT.

5. The historical experiments offer a warning

A later study of directly addressable variable-length codes compared a Sadakane-style Ψ representation with more general direct-access integer coding.

It reported one comparison in which the Sadakane representation used 66.72% of the original text size and took approximately 646 ns per Ψ query.

Some of the more general encodings approached its space and time, but didn’t clearly dominate it. ResearchGate

Those numbers shouldn’t be applied to mibz2.

They involve a different memory hierarchy, larger indexes and isolated query access rather than many interleaved traversal lanes.

But the experiment illustrates an enduring problem:

Making variable-length integers directly addressable can consume much of the space saved by compression, without necessarily producing cheap enough access.

This is why our fixed-size translation descriptors and cache-line records remain interesting.

They trade some compression efficiency for an address calculation that is immediately available from the current row.

6. The most important new conclusion: output reconstruction and Ψ navigation should be studied together

Here’s something Sadakane’s original objective brings into focus.

He wasn’t simply trying to encode Ψ compactly.

He wanted a compressed structure capable of reconstructing the original text without storing the original text separately.

That is particularly relevant to our 24-bit link-only proposal.

We have been considering eliminating the output byte from each navigation entry and recovering it from F.

Sadakane’s compressed suffix array uses an analogous principle: the alphabet and suffix-array intervals supply the character information, while Ψ supplies navigation.

The difference is where the computational cost appears.

For an index, recovering a character through auxiliary structures can be acceptable.

For mibz2, that information must be available at the same high rate as the pointer traversal.

Thus the complete operation we’re trying to optimize is not just:

j\mapsto\Psi(j)

but:

\boxed{
j\mapsto
\left(\Psi(j),Fj,\operatorname{marker}(j)\right)
}

A representation that compresses Ψ brilliantly but requires two extra memory accesses for the symbol and marker may be a poor decoder representation.

This is one place where our problem meaningfully diverges from classical compressed suffix-array design.

7. What this paper teaches us about construction

This is also relevant after #81’s positive construction result.

Sadakane’s compressed index is intended to persist and answer many future queries.

Consequently, the cost of constructing the index can be amortized over a large amount of later work.

mibz2’s navigation table is temporary. It is constructed, traversed, and discarded.

That changes the cost calculation fundamentally.

An encoding method that saves another 15% in stored bits but needs an extra complete pass over Ψ might be excellent for a text index and terrible for a decompressor.

Petri’s later thesis explicitly identifies the need for complete suffix-array construction as an important memory bottleneck in traditional compressed-index building. That’s a useful historical reminder that compact final storage and compact construction are different problems. jsc.github.io

Our favourable circumstance is that the BWT has already been produced by the compressor.

We aren’t building a suffix array or BWT from raw text. We’re constructing only the structure needed to invert an existing BWT.

That gives us a simpler starting point than the full text-index construction problem.

And the sorted occurrence lists of Ψ can be assembled from a sequential scan of L.

The major open question is how to parallelize that assembly cheaply enough to compete with our fused #81 constructor.

8. What this changes about the research agenda

I think Sadakane’s contribution is best understood as the theoretical foundation of the entire line of work we’ve followed.

Research stage	Central idea
Sadakane	Compress Ψ by exploiting increasing character-specific sequences
Practical sampled codecs	Balance space against the work needed to reconstruct arbitrary entries
Gog, Navarro & Petri	Use specialized block codecs, including uniform blocks, for faster Ψ access
Ottaviano & Venturini	Exploit local density and choose encoding partitions efficiently
Our mibz2 research	Preserve direct addressing and memory-level parallelism during complete Ψ traversal

The mathematical foundation is well established.

What remains distinctive about mibz2 is the combination of constraints:

* The representation is built anew for each compressed block.
* Every row is navigated during inversion.
* Each navigation determines the next address.
* Many independent traversals run concurrently.
* Multiple blocks compete for shared cache.
* The output byte and segment markers are needed alongside the link.

Those constraints change which space–time trade-offs are attractive.

9. The strongest question Sadakane leaves us with

A sampled-difference representation stores the occasional large change and encodes all intervening increments.

Our fixed-group translation representation does almost the reverse: it stores the common case as one directly indexed base and pushes complicated groups into external payloads.

Both exploit the same underlying structure.

So the most interesting research question isn’t whether the sequence is compressible.

We know it is.

It’s this:

Can we expose the piecewise-constant displacement structure of Ψ in a way that makes each individual successor nearly as cheap to obtain as a direct array lookup?

This is a much more hardware-specific question than classical compressed suffix-array theory was designed to answer.

And Sadakane’s work makes the historical progression clear: we’ve inherited a well-understood mathematical representation problem, but we’re optimizing it under a radically different execution model.

My verdict

The Sadakane paper establishes the intellectual origin of our Ψ-compression programme.

It demonstrates why a large explicit array of permutation positions isn’t fundamentally necessary, and why sorted character-specific occurrence lists are a powerful basis for compressed navigation.

The subsequent literature has steadily improved how that information is encoded and accessed.

Our research now differs mainly in what we’re willing to pay for:

A persistent compressed index can afford occasional additional work to save memory.

A high-performance one-shot decoder may prefer a larger representation if it lets the CPU issue the next unpredictable memory request sooner.

The most promising bridge between those two worlds remains the same: directly indexed translation groups, with carefully controlled exceptional decoding.

The remaining historical question is how much more cheaply the nontranslation regions can be represented.

That leads directly to our next literature topic: piecewise-constant residuals and patched integer coding.

There, we’ll investigate whether the difficult 22–23% of Ψ rows really need general Elias–Fano at all—or whether many of them are just a handful of constant-displacement pieces waiting for a cheaper representation.
