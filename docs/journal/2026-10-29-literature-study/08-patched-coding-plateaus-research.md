<!--
Provenance: the research chatbot's deep dive 6 (patched integer coding: Lemire & Boytsov,
"Decoding Billions of Integers per Second Through Vectorization", Software: Practice and
Experience, 2015; Brisaboa, Ladra & Navarro, "DACs: Bringing Direct Access to
Variable-Length Codes", 2013; and its own residual-plateau analysis), relayed by the user
into the working session on 29 October 2026, unedited. Some inline mathematics survived
the copy as LaTeX source; the rest was dropped, leaving gaps where a symbol stood (for
example "define a bitmask ." and "There are: \binom{B-1}{k-1}"). Both are kept as
received. The inline figure and the code-like lines arrived as plain lines. Links appeared
as their titles only, followed by the source's site name. Tables arrived tab-separated and
are kept so.

Three entries in §10's table are slightly off, and are corrected in the index, not here:
log2 C(31, k−1) is 12.13 for k = 4 (not 12.31), 21.33 for k = 8 (not 21.47) and 28.16 for
k = 16 (not 28.54).
-->

Literature deep dive 6 — Patched integer coding and piecewise-constant Ψ

From PForDelta to FastPFOR, and a new way of looking at mibz2’s nontranslation groups

We’ve reached a particularly interesting part of the research because the question has changed.

We already know that roughly 77% of the rows in the measured full-sized blocks belong to groups where 32 Ψ values are consecutive.

We also know the remaining groups aren’t necessarily arbitrary. Within an F-symbol interval, their values are still increasing.

What if many of the supposedly difficult groups are just two or three short consecutive sequences joined together?

That’s the question I’d like to investigate through the patched-integer-compression literature.

1. The key paper

Decoding Billions of Integers per Second Through Vectorization⁠ — Daniel Lemire and Leonid Boytsov, Software: Practice & Experience, 2015.

This paper compares established integer codecs and develops FastPFOR, SimplePFOR and SIMD-FastPFOR. It is useful partly because it makes the construction-versus-decoding trade-off unusually explicit.

Its main idea is straightforward:

Encode most numbers using a small, cheap representation, then handle the unusual numbers separately.

That’s exactly the philosophy behind our proposed compact Ψ records. Wiley Online Library

2. How patched frame-of-reference works

Imagine these eight integers:

2,3,1,4,2,3,200,1

Seven of them fit comfortably in three bits.

But 200 requires eight bits.

If we use one fixed-width field for every integer, we must allocate eight bits to all eight values.

That’s wasteful.

Patched frame-of-reference (PFOR) uses three ingredients instead:

A simplified patched encoding

Ordinary values, stored using three bits each

010

011

001

100

010

011

000

001

The unusual value is represented by a placeholder in the packed data.

Exception position

6

Exception value

200

Decode the common values normally, replacing slot 6 with its exceptional value.

The large number no longer forces every value to occupy eight bits.

This is useful whenever most values are small and only a few are unusually large.

The original PFOR approach dates to work by Zukowski and colleagues; Lemire and Boytsov compare and extend several variants. Wiley Online Library

But notice something important.

PFOR was designed primarily to decode blocks of integers.

We need something more demanding: retrieve one arbitrarily selected integer, then immediately use that result as the next pointer.

That difference is going to shape our conclusions.

3. The experimental result that caught my attention

Lemire and Boytsov compare formats based on compression ratio, encoding speed and decoding speed.

Their experiments show that a more sophisticated compressor isn’t necessarily a better engineering choice.

In particular, they report that OptPFD can achieve excellent compression but has very expensive encoding: in their tests, its encoding rate was approximately 20–50 times slower than SIMD-FastPFOR.

That’s an enormous difference. Wiley Online Library

This immediately reminds me of #81.

Our recently fused T-vector construction became 31.3% faster in the reported 3×3 configuration.

A compact Ψ representation must compete against that improved builder.

So we cannot afford to treat encoding complexity as irrelevant.

If a compression scheme spends significant time trying to choose optimal exception positions or bit widths, that time is paid during every decompression.

A searchable index might amortize construction over millions of future queries.

mibz2 builds the structure, traverses it once, and discards it.

The best persistent-index codec need not be the best temporary-decompression codec.

This is one of the most important lessons from the paper.

4. The most useful mathematical transformation: remove the predictable increment

Let’s return to Ψ.

Consider a group of eight increasing navigation positions:

\Psi=500,501,502,510,511,512,530,531.

A conventional difference encoder sees:

1,1,8,1,1,18,1.

Most increments are one. Only two are unusual.

A patched-difference encoder could store the common increments implicitly and patch the two jumps.

But there’s a representation I find more revealing.

Subtract the local position from every value:

R(i)=\Psi(i)-i.

We get:

R=500,500,500,507,507,507,524,524.

The same Ψ group viewed as residuals

Ψ0

500

Ψ1

501

Ψ2

502

Ψ3

510

Ψ4

511

Ψ5

512

Ψ6

530

Ψ7

531

Subtract slot number

500

500

500

507

507

507

524

524

Eight pointers become three constant plateaus, with changes at slots 3 and 6.

The original sequence contains eight values.

The residual sequence contains only three distinct values, arranged in three consecutive stretches.

We could describe it as:

* Start at residual 500.
* At slot 3, add 7.
* At slot 6, add 17.

That’s all the information needed for the eight navigation positions, aside from the group’s starting index.

This is a useful change of perspective.

Instead of compressing 32 increasing positions, we can compress the changes in a piecewise-constant displacement function.

5. What a random-access residual codec would need

For a group containing 32 rows, define a bitmask .

Set bit whenever the residual changes at slot , for .

For our eight-entry example:

Slot:       0 1 2 3 4 5 6 7
Residual: 500 500 500 507 507 507 524 524
Break:      · · · 1 · · 1 ·

Now imagine separately storing the residual value at the beginning of each plateau:

V=500,507,524.

To retrieve the residual at slot :

* Count the break bits at or before .
* Use that count to select the appropriate plateau value.
* Add to recover the corresponding Ψ destination.

Mathematically, using zero-based indexing and a 32-bit break mask:

k(i)=\operatorname{popcount}
\left(M\ \&\ (2^{i+1}-1)\right)

and:

\boxed{\Psi(i)=i+Vk(i)}.

For an actual group beginning at global row , the residual values would be based on , and the reconstruction becomes:

\Psi(a+i)=a+i+Vk(i).

This is a simple and exact representation for any group, not just perfectly consecutive ones.

It is particularly compact when the number of plateaus is small.

Why this is interesting for the CPU

popcount is a hardware instruction on your Intel i9.

If the plateau values can be packed into one or two machine words, finding the appropriate value could require:

* A directly indexed record load.
* A mask operation.
* A popcount.
* A packed-field extraction.
* An addition.

No predecessor search.

No variable-length difference decoding.

No general select structure.

This is a research design derived from the residual mathematics, not a codec whose performance has been demonstrated by Lemire and Boytsov.

And it has an important limitation: the plateau index becomes available only after processing the break mask, so a separate plateau-value array may introduce a dependent memory lookup.

To preserve the benefit, the mask and values should ideally reside in the same directly addressable record.

6. The subtle difference between patches and plateaus

PFOR treats rare large values as exceptional.

Our residual representation treats rare changes as significant.

These sound similar, but the distinction matters.

Suppose a group has one enormous jump followed by a long consecutive stretch:

100,101,102,700000,700001,700002,\\ldots

A FOR representation needs offsets wide enough to accommodate the large span, unless it uses an exception.

A residual-plateau representation only needs to store the new displacement and where it begins.

It does not care how large the jump is until choosing the width of that one displacement value.

This might be a major advantage on data containing a few large discontinuities.

Conversely, suppose a group contains many small jumps:

100,102,104,106,108,\\ldots

A conventional FOR or Elias–Fano representation could encode this efficiently.

A plateau representation would have almost as many plateaus as values.

It could be terrible.

So the useful predictor isn’t merely the numerical span or the number of exceptional large differences.

It’s the number of residual plateaus per group.

This is a new statistic we have not yet seen in the reported census.

It may be more informative than simply classifying the 22–23% nontranslation population as FOR-14 or EF-12.

7. A concrete space model for plateau records

Suppose a 32-row group has distinct consecutive residual plateaus.

A straightforward representation could store:

* One 32-bit change bitmap.
* absolute residual values, using up to 20 bits each for a 900,000-row block if an appropriate unsigned base-relative form is used.

Its approximate raw cost is:

\boxed{32+20k\text{ bits}}

before metadata, markers, output symbols and alignment.

Strictly, a signed global displacement can range from to , so directly storing signed residuals in 20 bits is not always sufficient. A group-local base or absolute destination anchors avoid that problem.

For example:

Plateaus	Bitmap + 20-bit anchors
1	52 bits
2	72 bits
3	92 bits
4	112 bits
8	192 bits
16	352 bits
32	672 bits

These are illustrative information fields, not complete physical record sizes.

Notice what this implies.

A group with three or four translation pieces might be representable using around 100 bits of position information, far less than the 448 bits allocated to 32 FOR-14 offsets.

That is potentially a large saving.

But if we give every group a fixed 64-byte record anyway, those saved bits don’t reduce storage.

To exploit the saving, we would need either:

* Densely packed payloads behind the small directory.
* More than one logical group per cache line.
* Or a different way to spend the unused record space.

This brings us back to the distinction between the fixed-line and two-tier architectures.

Plateau coding looks particularly interesting for the two-tier design, whose exceptional payloads are allowed to have different sizes.

For direct navigation speed, however, the fixed-line design may still be preferable.

8. A second paper provides an important warning about random access

DACs: Bringing Direct Access to Variable-Length Codes⁠ — Nieves Brisaboa, Susana Ladra and Gonzalo Navarro, 2013.

This is worth bringing into the discussion because the patched-coding literature has largely optimized throughput when decoding many integers together.

DACs tackles a more relevant problem: how can we retrieve one integer from a compressed variable-length sequence without decoding all preceding integers?

The authors divide integer codewords into chunks and organize those chunks into levels. Auxiliary bitvectors let a query follow the necessary levels for one chosen integer.

That provides direct access without periodic sampling, although accessing larger values can require traversing additional levels. ScienceDirect +1

This gives us a useful conceptual distinction.

Technique	What it optimizes
SIMD-FastPFOR	High-throughput bulk integer decoding
DACs	Access to an individual variable-length integer
Our fixed-line Ψ	Recovering one successor with a short memory dependency chain
Our two-tier Ψ	Very small common-case descriptors with occasional payload access

A codec can decode billions of integers per second in a tight sequential loop and still be unsuitable for pointer chasing.

Why?

Because bulk decoding amortizes setup costs, exploits SIMD, and already knows where the next compressed block lives.

In mibz2, the next row is unknown until the current one has been decoded.

That’s a fundamentally different execution pattern.

So I wouldn’t conclude from Lemire and Boytsov’s extraordinary bulk-decoding numbers that SIMD-FastPFOR should be used inside mibz2’s hot walker.

The transferable lessons are exception separation, per-block bit widths, fast bit packing and cheap encoding—not necessarily the entire bulk codec.

9. A third useful lesson: exceptions can be organized globally or locally

Lemire and Boytsov describe several strategies for storing exceptions.

Some patched schemes encode exceptions alongside each compressed block.

Others collect them into separate arrays, which can make bulk compression and decoding more efficient.

FastPFOR combines per-block bit-width decisions with page-level exception storage. GitHub

For mibz2, the trade-off is different.

An exception stored elsewhere may introduce a second unpredictable memory access.

That is exactly the risk we’ve identified in our two-tier Ψ proposal.

So there is a tension:

Putting all values together helps one-shot random access; separating rare values can improve space and bulk processing efficiency.

The best choice depends on how often exceptions occur and whether their accesses hit cache.

Our current data strongly supports keeping ordinary translation answers in the direct descriptor.

For nontranslation groups, the correct choice remains open.

If plateau coding fits several small values into a single auxiliary record, it might reduce storage without adding another exception lookup beyond the payload access already required.

That is an important constraint on how we should evaluate the idea.

10. How far can we push the mathematics?

The plateau representation admits a neat combinatorial interpretation.

A group of rows with exactly plateaus has internal break positions.

There are:

\binom{B-1}{k-1}

possible ways to place those breaks.

The minimum information required just to specify their locations is:

\boxed{
\log_2\binom{B-1}{k-1}\text{ bits}
}

For :

Number of plateaus	Minimum break-position bits
1	0
2	4.95
3	8.86
4	12.31
8	21.47
16	28.54

This shows that a full 32-bit break bitmap contains plenty of redundancy when there are only two or three plateaus.

For example, two plateaus require just one break location among 31 possible boundaries. Five bits suffice.

But we need to ask whether the resulting compression helps navigation.

A break bitmap has an appealing property: the plateau number can be obtained with popcount.

A packed list of break positions uses fewer bits but might require several comparisons or a specialized rank calculation.

So, once again, the information-theoretic optimum isn’t necessarily the fastest representation.

The mathematical opportunity is real. The architectural choice remains undecided.

11. A particularly simple candidate: the two-plateau group

Let me isolate one case because it may be the easiest way to understand what plateau compression buys us.

Suppose the group contains two consecutive stretches:

\Psi(a+i)=
\begin{cases}
p+i,&i<t\\
q+(i-t),&i\ge t.
\end{cases}

We need only:

* First destination .
* Second stretch’s first destination .
* Break position .

For a maximum-size block, two 20-bit absolute destinations and a five-bit break position give:

20+20+5=45\text{ bits}

of navigation information.

This is much smaller than 32 packed 14-bit offsets.

And accessing one value needs only a comparison and an addition after the record is loaded.

Conceptually:

next = slot < split
     ? first_base + slot
     : second_base + (slot - split);

No select.

No general-purpose patch decoder.

No Elias–Fano.

This is the kind of special case I find intriguing.

It would fit comfortably in a 64-byte cache-line record, but that alone would not shrink a fixed-line design’s allocated footprint.

More importantly, it does not fit into our proposed 64-bit directory descriptor once the 32 marker bits and symbol metadata are included.

So a two-plateau group would still need an auxiliary payload—or a change to metadata organization.

That’s why I don’t consider it an automatic replacement for our translation fast path.

Its potential role is making the existing nontranslation payload path cheaper.

12. What we’ve actually learned from this literature

The three source families reinforce different principles.

Patched coding: optimize the common case.

Use cheap, fixed-width representations for ordinary values and reserve additional machinery for exceptional ones.

FastPFOR: encoding cost matters.

Elaborate exception optimization can produce attractive compression while being far too expensive during construction.

DACs: individual random access is a separate problem.

A format that’s fast for streaming decompression isn’t necessarily fast for successor navigation.

New hypothesis: compress residual plateaus, not just absolute positions.

The number of distinct constant stretches within a 32-row group may predict a simpler format than FOR or EF.

The last point is our deduction from Ψ’s structure, not a result claimed by the cited authors.

13. Where I think this research leads

The critical missing information is now very precise.

We know that approximately 77% of rows are in entirely consecutive 32-row groups.

But we don’t know how the other 23% are distributed by number of translation pieces.

Here’s the census I’d most like to see conceptually:

Group complexity	Meaning	Possible encoding
1 plateau	All 32 links consecutive	Translation descriptor
2 plateaus	One internal discontinuity	Two bases + split
3–4 plateaus	A few internal discontinuities	Break mask + anchors
5–8 plateaus	Moderately irregular	Plateau coding or FOR
9+ plateaus	Highly irregular	FOR, EF or raw
Mixed F-symbol groups	May not be monotone throughout	Explicit boundary-aware fallback

The value of this information isn’t that it forces us to introduce another encoding.

It’s that it tells us whether general-purpose monotone codecs are solving a problem more complicated than most of our data actually presents.

There’s also a caution: plateau counts alone won’t tell us how fast a group can be decoded. Absolute anchor widths, branch costs, payload locations and marker access still matter.

Three research conclusions

First, we should preserve the existing translation fast path. Nothing in this literature suggests replacing its extremely simple base-plus-slot arithmetic.

Second, a small-plateau format is now a credible alternative for nontranslation payloads. In groups with only two or three constant stretches, it could avoid packed-offset decoding and in-register select.

Third, we should resist the temptation to adopt FastPFOR wholesale. Its extraordinary reported decoding speeds concern bulk integer arrays, whereas our workload is dependent random successor access.

The literature also strengthens the case for cheap, local encoding decisions. A temporary navigation table should not pay the construction cost of sophisticated compression optimization unless the resulting walk makes that cost worthwhile.

My verdict

This literature family gives us a potentially useful refinement to our earlier architecture.

We began with:

\text{Translation}\rightarrow\text{FOR}\rightarrow\text{EF}\rightarrow\text{raw}.

The new hypothesis is:

\boxed{
\text{Translation}\rightarrow
\text{few-plateau encoding}\rightarrow
\text{FOR/EF}\rightarrow
\text{raw}
}

Not because more formats are inherently better, but because the difficult rows may still contain simple structure that our current census hasn’t fully characterized.

If the nontranslation population is dominated by two- and three-plateau groups, that could be significant.

If most of those groups have many breaks, general FOR or EF remains the more plausible approach.

That’s the research boundary for this topic. Without the plateau distribution and actual single-entry decoding costs, inventing further variants would be premature.

The next and final item on our original literature-review list is cache-line Elias–Fano and CPU-aware decoding. That will bring the theory back to its most important practical requirement: how quickly an i9 can obtain the next unpredictable Ψ address from a compact representation.
