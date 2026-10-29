<!--
Provenance: the research chatbot's deep dive 4 (Gog, Navarro & Petri, "Improved and
Extended Locating Functionality on Compressed Suffix Arrays", Journal of Discrete
Algorithms, 2015), relayed by the user into the working session on 29 October 2026,
unedited. Some inline mathematics survived the copy as LaTeX source (for example
"\Psi(i)=SA^{-1}\[SA\[i\]+1]", with its escapes as received); the rest was dropped,
leaving gaps where a symbol stood (for example "where is the suffix array"). Both are kept
as received. Links appeared as their titles only, followed by the source's site name.
Tables arrived tab-separated and are kept so.
-->

Literature deep dive 4 — The paper that studies Ψ itself

Simon Gog, Gonzalo Navarro & Matthias Petri · Journal of Discrete Algorithms, 2015

Improved and Extended Locating Functionality on Compressed Suffix Arrays⁠

We’ve reached the paper I was particularly looking forward to.

The previous papers taught us how to compress monotone integer lists, how to exploit consecutive values, and how to construct interval structures efficiently.

This one actually applies those ideas to Ψ, the same kind of inverse-BWT permutation we’ve been studying for mibz2.

And there’s a striking result right at the centre of the paper:

The authors found that partitioned Elias–Fano was their fastest representation for randomly accessing Ψ—even though it wasn’t their smallest.

That’s exactly the kind of distinction our research has been trying to understand.

However, this paper has a broader agenda than Ψ compression alone. Its principal objective is to improve locating pattern occurrences in compressed suffix arrays (CSAs), including retrieving matches in text order or within a specified window. For mibz2, Section 4 and the experiments in Section 7.2 are the most directly relevant parts. users.dcc.uchile.cl

1. First, we’re definitely studying the same mathematical object

The authors define:

\Psi(i)=SA^{-1}\[SA\[i\]+1]

where is the suffix array.

In plain English: given a suffix’s position in the sorted suffix array, Ψ finds the position of the suffix beginning one character later.

This is the inverse of the conventional LF mapping.

And because the suffixes are sorted, Ψ is increasing within each character’s interval.

So the authors describe Ψ as the concatenation of increasing sequences, one for each character in the alphabet.

That is precisely the property that led us to treat mibz2’s Ψ as sorted occurrence-position lists. users.dcc.uchile.cl

There is a convention difference worth preserving: the paper assumes a suffix-array model of text with an endpoint, whereas bzip2 uses cyclic rotations and a primary index. The local monotonicity property transfers, but the complete traversal and termination conventions aren’t identical.

2. Their first attempt: compress the differences

Historically, compressed suffix arrays often represented Ψ by storing differences between successive values, using Elias-γ or Elias-δ codes.

For example, suppose part of Ψ is:

200,201,202,350,351,352.

Rather than storing the full values, we store the starting point and differences:

1,1,148,1,1.

Most differences are tiny. The occasional large jump is expensive, but that’s still usually much cheaper than storing every absolute position.

The practical difficulty is random access.

To recover the fifth value, a variable-length difference codec may need to decode preceding values in its block.

So these structures periodically restart the encoding, making independent blocks addressable.

The authors compare several alternatives to this traditional scheme.

3. The three representations they investigated

Representation	How it stores Ψ	Main trade-off
Elias-δ	Encodes successive differences using variable-length integer codes	Compact, but retrieving an arbitrary element can require partial block decoding
OptPFor	Packs most differences using a common bit width, with large differences stored as exceptions	Fast extraction, but exception handling is necessary
Partitioned Elias–Fano	Uses monotone-sequence structure and high/low bit decomposition	Efficient random access, with additional indexing metadata

All three were evaluated using blocks of 128 Ψ values, with block starting positions stored to permit pseudo-random access.

That number is interesting: their experiment uses chunks four times larger than our proposed 32-row groups.

Our 32-row choice isn’t driven by an information-theoretic optimum. It’s driven by the desire to fit a directly addressable record into a cache line. users.dcc.uchile.cl

4. Their treatment of uniform blocks is exceptionally relevant

Remember our reported finding that approximately 77.4% of rows in full-sized corpus blocks belong to groups of 32 consecutive Ψ destinations?

The authors address exactly the same mathematical special case.

They call them uniform blocks.

If the encoded block contains every value in its local numerical interval, its elements can be inferred from the boundaries.

Nothing needs to be stored for the individual values.

For example, a block containing:

500,501,502,503,504

can be recovered from a base and the element’s ordinal position.

The authors apply the uniform-block optimization not only to partitioned Elias–Fano but also to their OptPFor and Elias-code representations.

They note that this apparently straightforward improvement had not been used in the original Sadakane CSA implementation or earlier versions of SDSL. users.dcc.uchile.cl

That establishes an important prior-art boundary for us.

Our translation records are a specialized instance of an established compressed-Ψ optimization.

The novelty, if we eventually have any, wouldn’t be recognizing consecutive Ψ values. It would be the particular construction, cache-line layout and cooperative traversal strategy used to exploit them.

5. The really instructive obstacle: Ψ is not one increasing list

This is an easy subtlety to miss, and I think it’s one of the paper’s most useful contributions to our research.

Ottaviano and Venturini’s partitioned Elias–Fano indexes assume a globally increasing sequence.

But Ψ is only increasing within each character interval.

When we move from one character’s interval to the next, its values can jump backwards.

That breaks several convenient assumptions:

* The sequence of block representatives need not be increasing.
* A block might cross a character boundary and cease to be monotone.
* Merely comparing neighbouring block endpoints does not reliably establish that a block is uniform.

Gog, Navarro and Petri therefore adapt the partitioned scheme.

They distinguish blocks that remain monotone from those crossing character boundaries, using additional block-format information. For boundary-crossing blocks, they describe a prefix-sum-based encoding rather than applying ordinary monotone Elias–Fano unchanged. users.dcc.uchile.cl

This connects directly to our earlier concern about globally aligned row >> 5 groups.

We want the group address to be calculated immediately from the current row. We don’t want to move group boundaries just to accommodate the characters.

But globally aligned groups can straddle F intervals.

The paper confirms that this is a genuine structural issue, not an artefact of our proposed encoding.

What differs in our case?

Their alphabet can be enormous—the Wikipedia word-parsing experiment has millions of distinct symbols.

Ours is a byte alphabet with at most 256 symbols.

That gives us a valuable bound: there are at most 255 boundaries between nonempty character intervals.

Consequently, only a relatively small number of globally aligned 32-row groups can cross those boundaries.

This is a major simplification specific to our workload.

It makes a specialized boundary fallback much more credible than it would be for a word-based compressed suffix array.

6. Their experiments: Elias–Fano wins on speed, not minimum space

The authors measured random access to Ψ on two datasets:

* english.2108MB: a byte-alphabet English text collection.
* enwiki.4646MB: a Wikipedia collection parsed into word identifiers, with an alphabet of approximately 3.9 million symbols.

They issued ten million random Ψ queries.

Their Figure 3 shows that partitioned Elias–Fano was the fastest of the three encoding methods on both datasets.

Figure 3 compares access time with representation size for the three Ψ codecs. The original graph is available on page 13 of the linked PDF.

For the byte-alphabet dataset, their partitioned Elias–Fano implementation answered a random Ψ query in approximately 200 nanoseconds.

They describe this as roughly the cost of two memory accesses on their test system. users.dcc.uchile.cl

That is an especially important number to interpret carefully.

It isn’t evidence that a modern 32-row cache-line EF record takes 200 ns to decode.

Their dataset contains over two billion characters, far larger than the relevant caches, and their encoding uses a multilevel representation.

Our proposed full-block table is at most a few megabytes, and the cooperative decoder issues many requests concurrently.

So the architectural situation is fundamentally different.

What transfers is the comparative result:

In their tests, the codec that retrieved Ψ values most efficiently wasn’t always the one using the fewest bits.

They report that partitioned Elias–Fano was approximately 5–10% larger than the most space-economical alternative, but significantly faster.

This strongly reinforces our central research principle: optimize the cost of obtaining the next address, not bits per row in isolation.

An additional finding about OptPFor

The authors also observed that OptPFor became relatively more attractive on their huge-alphabet Wikipedia dataset.

They conjecture that this came from many character-interval transitions falling inside blocks.

OptPFor handles the resulting large differences as exceptions, whereas other encodings may require more work or more bits. users.dcc.uchile.cl

This is relevant to our own exception-handling design.

It suggests that a simple patched format can be especially attractive when the data contains a small population of structurally awkward values.

But our situation is not quite theirs: we’re proposing direct absolute-value lookup within a tiny record, rather than decoding a block of differences.

So it’s an idea to investigate, not a measured win we can transfer.

7. The paper contains a second insight that may be just as valuable

We’ve concentrated on its Ψ codecs, but I don’t want to overlook another contribution.

The authors noticed that compressed suffix arrays traditionally store two related sampled structures:

* Samples for locating occurrences.
* Samples for extracting text.

These structures are essentially inverse permutations of one another.

Instead of storing both independently, they demonstrate how to store one together with a compact auxiliary structure that recovers access to the inverse.

That reduces redundancy while preserving useful access guarantees. users.dcc.uchile.cl

This is another example of a theme we’ve seen in the Brown–Langmead construction paper:

If two data structures describe opposite directions of the same relationship, we should look for opportunities to store or construct one in terms of the other.

For mibz2, we shouldn’t immediately conclude that storing both LF and Ψ would help. That would increase our memory footprint and could undermine the purpose of compression.

But there is a useful construction question:

Could transient information about LF—or the original L positions—make compressed Ψ cheaper to assemble, while leaving only Ψ in the final navigation structure?

That is an inference inspired by the paper, not one of its demonstrated algorithms.

It fits our broader interest in using temporary redundancy to simplify construction.

8. A warning hidden in their end-to-end experiments

The most instructive result may be what happens when the authors put their faster Ψ representation into a complete compressed suffix array.

The isolated Ψ query improves substantially.

But the resulting improvement in suffix-array operations is smaller.

Why?

Because each operation must also access sampling and other auxiliary data.

The authors explicitly point to additional accesses to their compressed sampling structure as a reason the isolated Ψ improvement doesn’t transfer fully to the complete operation. users.dcc.uchile.cl

This is remarkably close to our current challenge.

We might devise a beautiful compressed Ψ lookup that takes only a few instructions after loading its cache line.

But mibz2 still needs to recover the output byte and handle its segment markers.

If those require additional memory accesses, we could lose much of the theoretical benefit.

That’s why our proposed record functionality must remain:

\boxed{
j\longmapsto
\bigl(\Psi(j),Fj,\operatorname{marker}(j)\bigr)
}

rather than merely:

j\longmapsto\Psi(j).

This is perhaps the paper’s clearest experimental warning for us:

Improving the central compressed-data-structure operation does not automatically improve the larger operation that uses it.

We’ve already experienced a version of this with #81, where a large construction improvement translated into a smaller—but still substantial—combined-pipeline improvement.

9. Does the paper change our representation research?

Yes, but it changes our confidence in the approach more than it identifies a new winning design.

Our research question	What this paper contributes
Is Ψ really the right object to compress?	Yes. The paper explicitly treats it as concatenated increasing lists.
Are translation blocks a legitimate special case?	Yes. Uniform blocks are an established Ψ optimization.
Can different local integer codecs be useful?	Yes. The authors compare Elias, OptPFor and partitioned EF.
Are character-boundary blocks a real complication?	Yes. Their EF adaptation requires special handling.
Is EF necessarily slower because it uses select?	No. Their implementation is the fastest in their isolated Ψ tests.
Does the smallest Ψ representation win?	Not necessarily; their fastest scheme uses slightly more memory.
Can we infer mibz2’s performance from these results?	No. Dataset sizes, query patterns and hardware differ.
Does the paper establish a fast direct-from-L compressed builder?	No. That remains our construction-research problem.

The most interesting new hypothesis

I think the paper makes a strong case for investigating format specialization based on why a group is difficult, rather than simply how wide its numerical span is.

Consider three reasons a 32-row Ψ group might fail the translation test:

* It contains several increasing stretches separated by jumps.
* Its values are increasing but spread over a wide range.
* It crosses an F-symbol boundary and may cease to be increasing.

Those are mathematically different problems.

A patched representation could suit the first, Elias–Fano the second, and a raw or symbol-aware fallback the third.

This is closely related to what the authors did, but our direct-addressing and one-shot-construction constraints may justify a different format mix.

There is also the residual formulation we derived from the previous paper:

R(j)=\Psi(j)-j.

Within each character interval, this is a nondecreasing, piecewise-constant sequence.

A translation group has constant residual.

A nontranslation group may still have just a few residual plateaus.

That raises a particularly interesting, still unanswered question:

Among the groups that are not wholly translations, how many consist of only two or three translation pieces?

The paper does not answer that for our corpus.

But its examination of uniform blocks and patched differences gives us a solid reason to investigate.

10. What I’d investigate next

I now see three questions that would extract the remaining practical value from this line of literature.

First: the residual plateau distribution. We’ve measured how often an entire group is a translation. We haven’t established how complicated the remaining groups are. Counting their residual changes could tell us whether they’re nearly as simple.

Second: the complete lookup dependency graph. We should compare a fixed-line FOR entry, a small EF entry, and a patched residual entry based on the number of dependent memory operations and register instructions needed to obtain the next address, output symbol and marker.

Third: direct construction. Gog, Navarro and Petri establish that these encodings work for Ψ, but they don’t solve our distinctive problem of building a temporary compressed table as cheaply as possible during decompression. That’s still where #81’s success makes the research especially interesting.

The paper also makes it clear that we should retain the 24-bit direct table as a serious comparison point. A slightly larger representation with a much cheaper lookup can win.

My verdict on Gog, Navarro and Petri

This is the closest historical precedent we’ve encountered for our compact-Ψ research.

The mathematical structure is the same. The treatment of consecutive-value blocks is the same. The difficulty with character boundaries is the same. And the authors’ experiments reinforce the exact space-versus-access-time trade-off we’ve been exploring.

But mibz2 introduces three requirements their study doesn’t directly address:

* Building the structure as part of every decompression.
* Navigating it continuously rather than making isolated random queries.
* Sustaining many independent traversal chains across concurrent blocks.

That gives us a clear research boundary.

The compressed-Ψ representation techniques are established. The open problem for us is whether they can be made cheap enough to construct and navigate under cooperative inverse-BWT execution.

The most promising immediate theoretical direction is understanding the nontranslation groups, not inventing more elaborate codecs.

We already know that the common case is extremely simple. The next question is whether the difficult 22–23% is genuinely complicated—or just a small number of similarly simple pieces joined together.

That could determine whether we need Elias–Fano at all for many of those groups.

Where the literature trail now leads

At this point, we’ve connected the major papers in the original sequence:

* Brown and colleagues: practical move structures and their construction.
* Ottaviano and Venturini: locally adaptive compression of monotone lists.
* Gog, Navarro and Petri: applying those codecs directly to Ψ.

The next worthwhile deep dive would be into Sadakane’s original compressed suffix-array representation and its subsequent practical implementations. That would give us the foundations of Ψ difference coding and help distinguish historical implementation choices from inherent limitations.

Alternatively, we could follow the more immediately relevant residual question into literature on piecewise-constant and patched integer representations.

My preference is the latter: the most promising new information now concerns the structure of the nontranslation minority, because that’s where the existing literature suggests simpler encodings may still be hiding.
