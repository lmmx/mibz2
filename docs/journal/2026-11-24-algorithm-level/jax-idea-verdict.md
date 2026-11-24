# Louis's JAX idea, and another model's verdict on it (verbatim)

*24 November 2026. Louis put an idea to another model, then gave it the Coordinator's vetting prompt (journal §35). Both are recorded here verbatim, as he pasted them. The other model's inline links and interactive charts did not survive the paste; each `⁠￼` marks a lost link. The Coordinator's cross-check against the records is in journal §35.*

## Louis's idea

> I just had a thought. So I'm working on this software called mibz2. The repo is lmmx slash mibz2 on GitHub. I've previously worked with JAX. And when you rewrite software as JAX, you often get a big speedup, especially if it's to do with like parallelizable tasks. And I did this for some software for page dewarping (lmmx/page-dewarp). I rewrote it all with JAX instead and it got many, many times faster. Part of me is wondering whether there is any actual insight. I don't mean metaphorical insight. I mean like an actual reusable change of approach that basically I could use. The software is Rust and I don't think JAX has any sort of Rust flavored basis way of doing it. But yeah, part of me feels like that's possibly a great way to approach it

> that is really really exciting. i asked another llm i am talking about mibz2 with to give you a question that would help establish whether we are headed in a useful direction or not, please do as it says: [prompt]

## The other model's answers

Yes — and having looked at mibz2⁠￼, I think there is a very concrete insight to extract from your experience with JAX. It is potentially more interesting than simply finding a Rust equivalent of JAX.

The distinction I'd make is between optimising the execution of an algorithm and transforming the representation of an algorithm so that it exposes more parallelism in the first place.

JAX encourages the latter. When you rewrote your page dewarping software, you may have benefited from several things simultaneously:

* Vectorisation: introducing batch dimensions into previously independent computations.
* Fusion: eliminating intermediate arrays and unnecessary memory traffic.
* Functional state: making dependencies explicit and therefore easier to transform.
* Algebraic reformulation: replacing sequential operations with parallel primitives.
* Specialisation: generating efficient machine code for particular computation shapes.

The first interesting observation is that mibz2 already independently embodies some of this philosophy. Your 12 interleaved inverse-BWT chains are conceptually very similar to introducing a batch dimension into a recurrent computation using vmap and scan. You're expressing multiple independent states together rather than completing one traversal before starting another. lmmx/mibz2 · GitHub +2

But I think there is another JAX transformation that is particularly relevant to your project.

1. The particularly relevant JAX concept: associative_scan

I think this is the most promising transferable idea.

Ordinary sequential code represents a computation as a succession of state changes:

s_{i+1}=f_i(s_i)

Each step appears to depend on the preceding one. However, if the state transformations themselves have an efficient, associative composition operation, you can compute their cumulative effects using a parallel tree.

This is precisely what jax.lax.associative_scan does. JAX documentation

Conventional sequential execution

O(n) dependency depth

Associative composition

O(log n) composition depth; a full prefix scan adds a propagation pass.

This isn't just an analogy to bzip2. There is a 2014 paper by Edwards and Vishkin, Parallel algorithms for Burrows–Wheeler compression and decompression⁠￼, which reformulates decompression in terms of associative prefix sums and list ranking, obtaining logarithmic parallel depth with linear work under the paper's computational model. drum.lib.umd.edu +1

Interestingly, I also found that your own expert review journal⁠￼ already discusses that paper in the context of BWT inversion.

What I'd investigate is applying that same algebraic approach to the other stages of your decoder.

2. A concrete example: rewriting move-to-front as permutation composition

I looked at your actual src/mtf.rs⁠￼. The scalar implementation contains this:

let v = list[nn];
list.copy_within(0..nn, 1);
list[0] = v;

Your AVX2 implementation already optimises this state transition using register shuffles. But the transition is still sequential.

The JAX-style reformulation would be to recognise that every move-to-front index describes a permutation, independently of the contents of the list.

For example:

Initial state

A

B

C

D

Move index 2 to front

C

A

B

D

Move index 3 to front

D

C

A

B

Since permutations compose associatively, you can express inverse MTF as a prefix scan over permutations.

The literal JAX expression would be something like:

def compose(p, q):
    return p[q]
states = jax.lax.associative_scan(
    jax.vmap(compose),
    permutations,
)
decoded = initial_alphabet[states[:, 0]]

This assumes normalised MTF indices and excludes bzip2's run handling and validation for clarity.

Now, materialising a 256-element permutation for every symbol would almost certainly be a bad implementation for your CPU decoder. But chunking changes the economics.

You could summarise each chunk with its net permutation, perform a prefix scan over those summaries, then decode each chunk independently from its computed initial state.

That is a genuinely different algorithmic formulation from optimising the existing serial MTF loop.

3. Where else this thinking applies to mibz2

Looking across the decoder, almost every stage has a potential formulation in terms of parallel array primitives.

Decoder stage	JAX-style reformulation
Huffman decoding	State-transition composition, speculative decoding
Move-to-front	Associative permutation scan
T-vector construction	Histogram, prefix sum, stable scatter
Inverse BWT	Gather, pointer jumping, list ranking
Run-length expansion	Prefix sum, segmented expansion
CRC	Associative polynomial composition

The Huffman scanner particularly interests me because your README identifies it as a serial bottleneck on poorly compressible binary records. lmmx/mibz2 · GitHub

The research question there is whether you can represent chunks of the compressed bitstream as composable state transitions, rather than requiring the exact decoder state before beginning work on each chunk. Bzip2's Huffman table switching makes that substantially harder than ordinary fixed-table Huffman decoding, however.

4. Is there actually a Rust equivalent of JAX?

There are some surprisingly relevant projects.

CubeCL⁠￼

A JIT-compiled Rust compute language supporting CPU and GPU backends, compile-time specialisation and vectorisation. Probably the closest fit for experimenting with custom integer-heavy kernels.

Closest to the kernel compilation aspect of JAX

Massively⁠￼

Built on CubeCL, offering composable parallel scans, gathers, reductions, segmented operations and lazy transformations.

Closest to the array programming approach

Futhark⁠￼

A functional, compiled array programming language designed around parallel primitives and fusion. Not Rust, but potentially a very interesting language for prototyping alternative algorithms.

Closest to the mathematical philosophy

docs.rs +2

I wouldn't introduce any of these as a production dependency yet, especially given your existing standard-library-only architecture.

5. What I would actually do

I'd create a separate research track in mibz2 that deliberately approaches decompression as though it were being written in JAX, even if the eventual implementation is entirely in Rust.

The methodology would be:

* Express each decoder stage as a pure transformation.
* Make independent dimensions of computation explicit.
* Identify which sequential state changes admit compact associative summaries.
* Reformulate them using maps, scans, gathers and scatters.
* Fuse transformations to avoid intermediate materialisation.
* Compare work, dependency depth, memory traffic and actual execution time against the existing kernels.

You could even use JAX itself as an executable mathematical prototyping environment, then translate successful formulations back into Rust.

The important caveat is that page dewarping is much more naturally amenable to dense array computation than decompression. Your Rust decoder is already highly optimised, so I wouldn't assume another order-of-magnitude improvement. I have examined the architecture, MTF implementation and some research journals, but haven't benchmarked these proposals or established that every one is unexplored in your project.

Nevertheless, I think the reusable insight you've identified is real. It has a name: algebraic data-parallel programming, combined with work–span analysis.

Rather than asking how to make each instruction execute faster, you ask whether the dependencies between instructions are mathematically necessary at all.

That is quite a powerful research direction for mibz2, and one that goes considerably beyond adopting JAX or finding a Rust equivalent.

Verdict: the JAX insight under the mibz2 breakthrough test

I've now read considerably more of your repository, including today's research discussion⁠￼, the research frontier⁠￼, and some of the previous contraction experiments.

That changes my assessment in an important way. Your research programme is substantially further along than I appreciated when I first suggested associative scans.

My preliminary verdict is:

SOLID BUT INCREMENTAL for the conventional associative-scan approach. NEEDS ONE MISSING PIECE for its generalisation into a genuinely different basis of inversion.

There is something worth pursuing here, but I would distinguish the JAX insight itself from the particular algorithm I initially proposed. The former is more powerful than the latter.

1. Steelmanning the idea

The strongest interpretation of what you discovered through your page dewarping work is not vectorisation, parallelism or even associative scans.

It is this:

Replace the state of a computation with a representation of what the computation does to any possible state.

That sounds like a subtle distinction, but it is mathematically significant.

Consider three levels of representation:

Level 1 — The walk

Calculate where one row goes, then repeat.

Level 2 — Rauzy–Veech induction

Compose one transformation shared by neighbouring rows.

Level 3 — Transformation equivalence

Find non-neighbouring states whose transformations can be represented by a common rule.

The third level is what I would extract from JAX and apply to your programme.

Rauzy–Veech induction discovered that many apparently separate computations are actually instances of the same operation.

The generalisation would be to discover a richer notion of the same operation.

But a useful equivalence must be discoverable cheaply, preserve enough information to reconstruct the exact output, and remain compact when composed. That last requirement is where the straightforward JAX idea encounters trouble.

2. My strongest concrete candidate: change the alphabet of the walk

Rather than composing individual pointers, suppose we compose symbols.

Your current LF mapping can be expressed as:

LF(i)=CL_i+\operatorname{rank}_{L_i}(i)

In plain language, the destination of a row depends on its letter and how many occurrences of that letter precede it.

The interesting thing is that this is true whether or not those occurrences are adjacent.

Take this simplified scrambled column:

Rows of the BWT column

Three isolated A rows become consecutive in the destination's ordering.

The grouping here is not adjacency. It is occurrence rank within a symbol class.

The first extension is to group rows not by the single letter they produce, but by the next two, three or four letters they produce.

For example, all the rows producing a7f belong to one class, irrespective of their original position in the BWT.

For hexadecimal content:

Grouping	Possible classes
Single hexadecimal digit	16
Two digits	256
Three digits	4,096
Four digits	65,536
Five digits	1,048,576

There is a potentially useful middle ground here. Three-digit words are short enough to occur repeatedly even in essentially random data, but long enough that decoding one at a time could substantially reduce the number of unpredictable accesses.

The question is whether you can promote these words to the status of individual symbols without first walking the rows to discover them.

That would be a concrete expression of contraction along composition with a changed meaning of what moves together.

3. Is there an exact construction?

There is an interesting construction, although it has serious practical limitations.

Define the two-letter word of row as:

W_2(i)=(Li,LLF(i))

The critical observation is that LF is monotonically increasing within each letter class.

Therefore, while scanning the input column, you can obtain the second letter of each word using a set of monotonically advancing cursors, one per first letter.

You don't need to follow an unpredictable pointer chain to discover the words.

The choice of cursor is data-dependent, but each cursor moves sequentially through its own region.

You can then express the squared LF mapping in terms of the occurrence ranks of those words:

LF^2(i)=C_2W_2(i)+\operatorname{rank}_{W_2(i)}(i)

Here the class offsets must be ordered by the reversed two-letter words, reflecting the direction of LF.

Why is that correct? Because composing LF twice preserves the relative order of rows sharing the same emitted word. Those rows arrive in the contiguous sorted interval corresponding to that word's reversed prefix.

This extends to longer words.

Consequently, a three-symbol transition could potentially replace three individual LF steps with one transition, yielding three bytes per traversal step.

This is exact, not a statistical prediction. There is no requirement that the input text be repetitive, although periodic blocks, the primary row and the final partial word all need correct handling.

The important distinction from your previous T² experiment

I found your 23 November squared retained-map experiment⁠￼.

That experiment tried walking two retained transitions at once. Your retrospective⁠￼ records that it failed on real workloads, particularly with multiple workers.

I would not suggest repeating that.

The distinction I'm proposing is constructing the composed mapping through the ordered occurrence streams of the original alphabet, rather than gathering pairs of pointers from an already constructed map.

The question is whether the changed construction mechanism removes the cost that doomed the previous approach.

4. Now try to break it

A. Does it actually give lone rows company?

Yes, in a mathematically meaningful sense.

Two lone rows need not be adjacent to share a three-letter word. Their relationship is expressed through occurrence rank rather than physical proximity.

On uniformly random hexadecimal content, a 900,000-byte block would have approximately 220 occurrences of each possible three-digit word, on average.

But there is an important distinction between sharing a word and sharing a destination. Each occurrence still has its own destination. Sharing the word does not automatically eliminate the cost of finding that destination.

This is the first major limitation.

B. Where does all the cost go?

Consider the following particularly favourable implementation, restricted for now to hexadecimal content.

Each row gets a packed 32-bit record:

Proposed three-symbol record

20 bits address any row in a bzip2 block. Twelve bits represent three hexadecimal digits.

In principle, this represents three emitted characters in the same space that a conventional 32-bit T-vector uses for a single transition. A small auxiliary table supplies the destination offset of each word class.

Here is the full accounting for this particular construction.

Cost	Assessment
(a) Unpredictable reads	Ideally large-table reads during emission, but additionally class-offset lookups and data-dependent cursor and counter accesses during construction.
(b) Dependent chain	Approximately macro-transitions, each with an additional offset lookup.
(c) Passes	Two outer construction scans are possible, assuming byte counts are already available. However, the second scan consumes three logical input streams, so the total input-reading volume is approximately four full-column equivalents before emission.
(d) Memory	4 bytes per row, plus counters, cursors and class offsets. Approximately 3.6 MB for the records alone on a maximum-sized block.

The construction obtains two-letter class sizes in the first scan. In the second, it uses the single-letter and two-letter occurrence cursors to discover triples, recording their occurrence ranks and accumulating their class sizes. The final offsets can then be calculated from those sizes.

If we wanted to replace the occurrence ranks with absolute destination pointers, that would require another pass to patch the records. Keeping the offsets separate avoids that pass but introduces another lookup during emission.

So this is not a free construction, and the additional lookups must not disappear from the accounting simply because they might hit L1 cache.

C. What is the theoretical payoff?

For independent uniformly random hexadecimal digits, the expected proportion of singleton runs is approximately:

\left(\frac{15}{16}\right)^2=87.9\%

Under the optimistic assumption that we could construct and use the macro-transitions without additional unpredictable reads:

Large-table traversal reads per lone row

Analytical estimate for uniformly random hex, excluding construction and auxiliary lookups

Current singleton floor

1.000

Two-symbol transitions

0.569

Three-symbol transitions

0.379

Four-symbol transitions

0.284

Your target: 0.500 or less, with construction included.

Three-symbol transitions have enough theoretical headroom to justify examining their acquisition cost. Two-symbol transitions do not even reach your half-per-singleton target on this favourable model.

But the chart is emphatically not a performance result. It describes the payoff if the missing capability were supplied cheaply.

5. The biggest problem: real Wikidata blocks

This is where I expect the straightforward construction to fail.

Your investigation into where decoding work goes⁠￼ found something extremely important:

* JSON keys constitute approximately a third of the text but only around 1% of lone rows in the reported example.
* Unique statement identifiers constitute approximately 5% of the text but around 62% of lone rows.
* Repeated hashes present a separate opportunity because their contents can potentially be copied.

The distribution of computational difficulty is radically different from the distribution of bytes.

A macro-symbol representation of the whole block would therefore do unnecessary work on the very regions that your existing decoder handles exceptionally well.

It would be like replacing a sophisticated decoder with a less sophisticated one just because the latter handles one particular class of content better.

Moreover, an unrestricted byte alphabet has up to 16.7 million possible three-byte words. A dense 32-bit counter table for those alone would occupy 64 MiB. The elegant hexadecimal representation does not generalise directly.

The missing piece is selective macro-symbolisation.

We would need a construction that promotes the difficult, singleton-heavy portions to multi-character symbols while leaving the long runs in their existing contracted representation.

And crucially, it must identify those portions directly from the scrambled column, not after decoding them.

This is the exact research obligation I would investigate.

6. Checking each of your traps explicitly

Trap	Verdict
Free construction	Not free. Requires multiple input streams, counters, cursors and record construction.
Hidden pointer walk	Avoided in the full-alphabet-class construction. Not yet established for selective construction.
Synthetic random data only	The mathematical identity is general, but the attractive cost estimate is restricted to random hex.
Reading the answer	No decoded text or suffix array is required for the construction described above.
Displaced cost	A serious risk. Traversal work has partly moved into occurrence-rank construction and auxiliary lookups.
Ignoring runs and gluing	The complete hybrid with the existing run representation is missing. This must be solved before claiming a gain.
Memory budget	Not established against Cells or Open. The packed full-row representation is especially unattractive with many workers.

It would be incorrect to declare this a breakthrough on the strength of the macro-transition count alone.

7. Prior art: has someone already done this?

I found several highly relevant precedents.

Edwards and Vishkin, 2014 — Parallel algorithms for Burrows–Wheeler compression and decompression⁠￼.

This is remarkably close to the general JAX insight. They explicitly reformulate the stages of BWT compression and decompression in terms of associative operators, parallel prefix sums and list ranking. They obtain linear work and logarithmic parallel depth for decompression under their parallel computational model. sciencedirect.com +1

That means the broad insight of my original answer is established prior art, including its application to the stages before inverse BWT.

The k-step FM-index. Researchers have developed generalised BWT representations that advance searches by multiple characters at once. This is closely related to the macro-alphabet construction above. fileserver-az.core.ac.uk +1

I cannot claim that the mathematical idea of grouping by multi-character words is novel.

Kärkkäinen and Puglisi, 2011 — Cache Friendly Burrows-Wheeler Inversion⁠￼.

They exploit previously decoded repeated substrings to eliminate subsequent unpredictable reads. Their work is closely related to your copy-plan research. Their subsequent work with Kempa reported substantial practical improvements to conventional BWT inversion. cs.helsinki.fi +1

Your own repository. This is arguably the most important prior art for our purposes.

The research frontier⁠￼ already identifies the extension from interval facts to class facts, and records the problem that longer context keys cease to provide useful shared classes on random content. It also documents substantial work on recompression, list contraction and alternative acquisition mechanisms.

The specific thing I would check for novelty is whether selective, bounded-width macro-symbolisation constructed directly through monotone occurrence streams has already been attempted.

I have not established that it is new. It is a narrower candidate than the general idea of composing LF transitions, which is certainly not new.

8. Whole-program reality

The 16% figure you supplied is sobering.

Even if we made the pointer-chasing stage infinitely fast, with everything else unchanged, Amdahl's law limits us to a 19% increase in whole-program throughput.

Whole-decoder speedup calculator

Fraction of time spent in target stage

16%

Speedup of target stage

2×3×10×Infinite

Total throughput

1.119×

Time saved

10.7%

Unaffected Remaining target work Saved

This is why I would preserve the broader JAX-inspired line of investigation beyond inverse BWT.

The more consequential breakthrough might be finding a representation that is simultaneously useful for undoing MTF, constructing the BWT mapping and emitting the original bytes.

Ordinarily, each stage has its own representation:

Compressed bitstream

Huffman symbols

MTF / run-length decoding

BWT column

Inverse BWT structures

Original text

JAX's fusion philosophy invites us to question whether these boundaries are mathematically necessary, rather than treating them as fixed.

Your repository already records an unsuccessful Huffman–MTF–construction fusion experiment. So ordinary loop fusion is not enough. A common algebraic representation capable of replacing stages is a different, substantially harder proposition.

9. The single cheapest experiment

I would not implement a new production kernel, introduce CubeCL, use a GPU or even start with a Rust implementation.

I would write a small charged macro-symbol census against the six existing Wikidata blocks.

Its purpose is to establish whether enough of the difficult rows can be replaced by useful macro-transitions to make a native implementation worth attempting.

It should construct the occurrence classes using only the raw BWT column and its byte counts. It should compare the current retained representation against two-, three- and four-character macro-transitions. Any reference decoding should be confined to correctness verification and evaluating the resulting coverage, never constructing the candidate.

It should count all cursor accesses, table lookups, memory allocations, input streams, remaining transitions and the cost of joining the macro-transitions to the long-run machinery.

Pre-registered decision rule

Promote to breakthrough candidate

Construction and emission together achieve at most 0.5 unpredictable reads per lone row on real blocks, within the agreed pass and memory budgets.

Retain as incremental

Dependent depth falls substantially, but total unpredictable reads do not.

Reject this construction

The advantage disappears on mixed real content, or acquisition and gluing consume the theoretical savings.

I would fix the definition of an unpredictable read before running the experiment, and report cache-line misses separately. Your own records correctly distinguish the proved bookkeeping floor from a universal machine-cost lower bound.

Final verdict

NEEDS ONE MISSING PIECE

Cheap selective composition in a non-adjacent occurrence-rank representation.

The original associative-scan proposal alone is incremental and substantially anticipated by existing literature.

The stronger proposal genuinely changes the grouping from adjacent equal rows to non-adjacent occurrences of multi-character words. It is mathematically exact as a full-column construction, but I have not demonstrated that the selective version meets your budgets, nor benchmarked it on your real blocks. I would not yet call it a breakthrough candidate.

However, I think your experience with JAX has yielded something more durable than this particular candidate.

The generalisation I would retain

JAX's useful abstraction is that a computation can be represented by a composable object instead of a succession of executions.

There are then two distinct questions:

What is the smallest representation of a transformation?

Under what notion of equivalence can many apparently different transformations be represented by the same object?

Rauzy–Veech induction provides one answer: translations on intervals.

Permutation scans provide another: composable changes of state.

Occurrence-rank mappings provide another: ordered transformations on non-contiguous symbol classes.

There may be substantially more powerful answers.

In mathematical language, what you are searching for is a representation of the composition operation that preserves the information necessary for exact decoding, but admits a cheaper way of computing it.

Or, in plain language:

Don't just look for rows that move together. Look for transformations that are secretly the same computation, expressed in different coordinates.

That is the part of the JAX experience I would carry forward into your research programme, regardless of whether this particular macro-symbol construction survives.
