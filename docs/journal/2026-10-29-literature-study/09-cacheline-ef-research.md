<!--
Provenance: the research chatbot's deep dive 7, the last of the original reading list
(cache-line Elias–Fano and CPU-aware decoding: Groot Koerkamp's cacheline-ef and
"PtrHash: Minimal Perfect Hashing at RAM Throughput", SEA 2025; Hough & Bhatele,
"Elias–Fano Compression for Space-Efficient Rank and Select Structures", SEA 2025),
relayed by the user into the working session on 29 October 2026, unedited. Some inline
mathematics survived the copy as LaTeX source, partly mangled (for example
"H\lefti+ \\left\\lfloor..."); the rest was dropped, leaving gaps where a symbol stood
(for example "Let be increasing integers."). Both are kept as received. The inline figures
and code arrived as plain lines. Links appeared as their titles only, followed by the
source's site name. Tables arrived tab-separated and are kept so.
-->

Literature deep dive 7 — Cache-line Elias–Fano and CPU-aware decoding

Final paper family in our original literature programme · Groot Koerkamp (2025), Hough & Bhatele (2025)

This is an unusually satisfying place to finish the reading sequence, because we have found a researcher who independently reached almost exactly the same hardware-driven conclusion as us:

Sometimes you should deliberately use more bits than necessary so that answering a query requires fewer memory accesses.

And unlike our proposed 16-bit Ψ representation, the resulting data structure has actually been implemented and benchmarked.

Our central sources are:

* cacheline-ef⁠ — Ragnar Groot Koerkamp’s implementation.
* PtrHash: Minimal Perfect Hashing at RAM Throughput⁠ — Groot Koerkamp, SEA 2025.
* Elias–Fano Compression for Space-Efficient Rank and Select Structures⁠ — Hough and Bhatele, SEA 2025.

The first two are directly relevant to packing monotone integers into cache lines. The third helps us understand the actual CPU instructions used to decode their high bits.

1. The implementation that looks remarkably like our idea

cacheline-ef stores 44 sorted integers in one 64-byte cache line.

Its layout is:

CacheLineEF’s exact 64-byte record

Base

44 low bytes

One per integer

EF high bits

128 bits

4-byte base

44-byte low section

16-byte high section

Total space per value

11.64 bits

The format stores a 32-bit high-part base, eight low bits for every value, and a 128-bit unary representation of the differences between high parts.

The source implementation exposes both indexed access and prefetching.

The author explicitly says the trade-off: the format consumes more space than ordinary Elias–Fano for comparable data, but retrieves an integer using one cache line rather than the approximately three accesses needed by the conventional representation in his implementation. GitHub +1

That is remarkably close to our fixed-line Ψ hypothesis.

We independently proposed allocating 64 bytes to 32 logical rows, putting the navigation information and required metadata together, and reconstructing a requested successor from that line.

The major difference is our extra requirements: the byte to output, the segment marker, and groups that cross F-symbol boundaries.

2. How the actual lookup works

Let be increasing integers.

The record stores:

base=\left\lfloor\frac{x_0}{256}\right\rfloor

and each value’s low eight bits:

low_i=x_i\bmod256.

The remaining high-part difference is encoded in a bitvector:

H\lefti+
\\left\\lfloor\\frac{x_i}{256}\\right\\rfloor-base
\\right=1.

To retrieve :

\boxed{
x_i=256\left(base+select_1(H,i)-i\right)+low_i
}

where select₁ returns the zero-based position of the requested set bit.

The interesting part is that H is only 128 bits.

We don’t need a large, multi-level select index. We need to select a set bit from two machine words.

The implementation can first count the set bits in the lower word, decide which half contains the requested bit, then perform in-word select. DROPS

For our proposed 32-entry EF-12 record, the high-part bitvector is only 64 bits.

So the select operation could be even simpler: just one machine word.

That is a meaningful architectural advantage.

3. The CPU trick: PDEP and TZCNT

The Hough–Bhatele paper provides a useful independent explanation of how to perform in-register select.

The combination is:

* PDEP — deposit a bit at the position of a selected set bit in a mask.
* TZCNT — count trailing zeros to find that selected position.

Conceptually:

uint64_t selected =
    _pdep_u64(UINT64_C(1) << k, high_bits);
unsigned bit_position = _tzcnt_u64(selected);

The result is the zero-based position of the -th set bit of high_bits.

This avoids walking a general-purpose rank/select index.

Hough and Bhatele use the same technique on short Elias–Fano bitvectors inside their orzo rank/select structure. Their purpose is to reduce metadata space without substantially increasing query time. DROPS

This is important because it gives us two independent implementations of the same hardware principle:

* cacheline-ef uses short high-part bitvectors for directly indexed monotone integers.
* orzo uses short high-part bitvectors to compress auxiliary rank/select metadata.

The common insight is that Elias–Fano becomes a different engineering proposition when the entire high-part select problem fits in a register.

That is exactly what our bounded 32-entry record attempts.

A hardware qualification

The PtrHash paper explicitly warns that PDEP can be slow on some AMD CPUs, including Zen 2, despite being supported by the instruction set. DROPS

Your i9-10900K supports BMI2, so the relevant instructions are available.

But availability is not the same as low latency.

And importantly, even a fast PDEP still adds dependent operations between receiving a Ψ record and knowing the next row address.

This is where our CPU-specific research must be precise.

A record can require one unpredictable cache-line fetch while still requiring several serial instructions to decode the next pointer.

Those are separate costs.

4. Their measurements are extremely encouraging—but need careful interpretation

The PtrHash paper includes a controlled comparison of remapping representations.

Its fast configuration, with one billion keys, reports:

Remapping representation	Space (bits/key)	Query, ordinary loop	Query, streaming
Plain u32 vector	0.33	12.5 ns	8.8 ns
CacheLineEF	0.12	12.9 ns	8.8 ns
Conventional EF	0.09	14.2 ns	9.7 ns

These are complete PtrHash query measurements, not isolated EF lookup latencies. The remapping structure is accessed only when needed, so the differences between rows are diluted by the surrounding hash computation. DROPS

Still, they demonstrate an attractive engineering result.

The author was able to replace a larger direct array with a more compact cache-line EF representation while retaining almost the same observed throughput in the streaming setting.

The overall PtrHash implementation also demonstrates unusually high query throughput by explicitly processing many independent lookups concurrently. DROPS

That sounds remarkably familiar.

However, there’s a crucial difference.

PtrHash can know future queries in advance

Suppose we need to hash these keys:

apple, banana, cherry, date

The keys are already known. We can calculate their hash addresses independently, issue several prefetches and then collect the results.

But Ψ traversal works differently:

j_0\rightarrow j_1\rightarrow j_2\rightarrow j_3

where:

j_{t+1}=\Psi(j_t).

We cannot calculate until we’ve retrieved .

That is a genuine dependency chain.

mibz2 overcomes this by following 32 different chains in parallel, rather than issuing future requests along one chain.

So the correct analogy is:

PtrHash	mibz2
Many independent hash keys	Many independent Ψ traversal lanes
Prefetch future known keys	Prefetch the known next position of each lane
Compressed remap lookup	Compressed navigation lookup
Query throughput	Decode throughput
No dependence between successive keys	Serial dependence within each Ψ chain

The architectural overlap is strong, but not exact.

The PtrHash results support the feasibility of fast cache-line compression. They do not establish that a compressed successor chain can match direct-pointer navigation.

This is the main boundary between what the literature demonstrates and what we still need to measure.

5. The part of PtrHash that may matter even more than Elias–Fano

Groot Koerkamp makes an unusually explicit point in the paper’s benchmarking discussion.

A normal loop over random hash queries does not necessarily measure the latency of one isolated query.

Modern CPUs execute instructions from multiple iterations concurrently.

Therefore, the reported nanoseconds per query often measure inverse throughput, not the complete response latency of an individual request.

The author then develops explicit batching and streaming interfaces, using prefetching to control the amount of parallelism. DROPS

This is directly relevant to mibz2.

We’ve been distinguishing:

* How quickly one Ψ successor can be calculated.
* How many Ψ successors the CPU can calculate per second across 32 independent lanes.

These are not the same optimization problem.

A compressed representation might add two or three cycles to one chain, yet have little effect on total throughput when other chains can progress.

Alternatively, those extra cycles might prevent the walker from sustaining enough outstanding memory requests.

This suggests the most important performance metric for our research is not merely isolated lookup latency.

It is successor throughput as a function of the number of independent chains.

That is a much closer analogue of PtrHash’s explicit streaming experiments than conventional compressed-index random-access benchmarks.

6. A subtle but important correction to our cache-line argument

We’ve often said that halving the table size might halve memory traffic.

That needs qualification.

A direct u32 lookup normally fetches a 64-byte cache line from the next memory level if the line isn’t already cached.

A compressed 64-byte record likewise fetches a full cache line on a miss.

So a single random navigation miss can transfer the same 64 bytes in either representation.

The compression advantage is not automatically that each navigation transfers half as many bytes.

It is primarily that the full navigation working set occupies fewer cache lines.

That can improve cache hit rates, reduce evictions and change how many concurrent block tables fit in the shared cache.

And, depending on access patterns, compressed records may create opportunities for line reuse.

But these benefits depend on how Ψ visits its records.

This distinction is particularly important because your earlier performance measurements identified cache-residency pressure as a significant constraint.

The correct research claim is:

Compression reduces the number of distinct cache lines required to represent the table; the reduction in actual memory traffic must be measured.

7. A more consequential result for our 16-bit design

Let’s compare the published CacheLineEF layout with our proposed 32-entry record.

	CacheLineEF	mibz2 fixed-line hypothesis
Record size	64 bytes	64 bytes
Values per record	44	32
Base	32 bits	Proposed 20 bits
Low parts	8 bits/value	Proposed 12 bits/value
High bitvector	128 bits	Proposed 64 bits
Other metadata	None in basic record	Output symbol, format, 32 markers
Primary operation	Indexed integer access	Dependent Ψ successor navigation

The smaller high bitvector in our proposal permits one-word select.

But it is also the reason the proposed format supports a more restricted range of positions.

As we derived earlier, with 32 values, 12 low bits, and a 64-bit high vector, the maximum representable high quotient is 32.

Under our specified encoding convention, that allows a relative offset of up to 135,167.

Values beyond that need an exception.

The published CacheLineEF similarly has a finite maximum span per record and must fall back when the span is too large. GitHub

This is valuable validation of the design pattern:

A cache-line-bounded EF format is naturally a conditional codec with a finite representable span, not a universal replacement for arbitrary integer arrays.

Our reported corpus census is what makes such a codec plausible for mibz2.

Without that census, we wouldn’t know whether ordinary records fit.

8. What I think is still missing from the published implementations

There is an important difference between fast random access and fast repeated successor access.

In a normal indexed lookup, we know both:

* The record to access.
* The slot within that record.

We can sometimes issue other independent operations while waiting for the record.

For Ψ navigation, the record and slot of the next lookup are determined by the value we just decoded.

That creates a serial chain:

One Ψ lane: the dependency chain

Current row

Load compressed record

Decode high + low parts

Next row

Only now can this lane address its next record.

Our fixed-line representation must make the decoding stage extraordinarily cheap.

This is why the FOR-14 and translation formats remain serious competitors to EF.

For translation:

\Psi(j)=base+slot.

For FOR-14:

\Psi(j)=base+packedOffset(slot).

For EF-12:

\Psi(j)=base+
\left((select_1(H,slot)-slot)\ll12\right)
+low(slot).

Each format adds a different amount of arithmetic and bit manipulation after the record arrives.

The question is whether reduced cache pressure can compensate for those additional operations.

The literature does not answer that specifically for Ψ traversal.

9. The most interesting secondary source: Orzo

Hough and Bhatele’s Elias–Fano Compression for Space-Efficient Rank and Select Structures⁠ develops orzo, a rank/select structure that compresses small blocks of auxiliary counters using Elias–Fano.

This is not a Ψ codec. But it offers an important independent demonstration of the principle we’ve been exploring.

Rather than applying EF to one enormous sequence, the authors apply it repeatedly to tiny sequences that can be decoded using register operations.

Their aim is to retain the cache-friendly layout of established rank/select structures while reducing the metadata footprint.

The paper reports a 26.5% reduction in rank-metadata overhead relative to the selected compact baselines, while attempting to limit the resulting performance cost. DROPS

For mibz2, the particularly interesting lesson is:

Elias–Fano need not be a global data structure with a separate, general-purpose select index. It can be a local codec inside a larger, directly indexed record.

That is exactly the conceptual justification for our fixed-line EF-12 format.

The surrounding record performs the indexing.

The short EF bitvector handles only the local monotone sequence.

And specialized CPU instructions recover the selected high part.

This is now well established as an engineering technique.

10. What the completed literature review tells us

We’ve now connected seven research topics:

Literature	Main contribution to our thinking
Seward (2001)	Inverse BWT is a genuine memory–computation trade-off
Move structures	Long translation intervals can replace explicit pointers
Optimal-time construction	Extra temporary structure can eliminate expensive construction searches
Partitioned Elias–Fano	Local regularity matters; fixed partitions can be economical
Gog–Navarro–Petri	Uniform Ψ blocks and specialized codecs are established techniques
Sadakane	The mathematical foundation of compressed Ψ navigation
Patched integer coding	Rare irregularities can be separated from a cheap common case
CacheLineEF / PtrHash / Orzo	Small EF blocks can support fast indexed access with cache-aware physical layouts

The most encouraging part is that these independent strands converge on a surprisingly consistent design philosophy.

We don’t need the absolutely smallest Ψ representation.

We need the representation that gets the next successor address onto the CPU’s load-address path as cheaply as possible, while keeping enough independent chains active to cover memory latency.

That is an unusual combination of constraints.

And the literature has now supplied strong precedent for nearly every component of our proposed solution.

The remaining uncertainty is their interaction inside mibz2.

11. Three specific things I’d take away for our research

First: CacheLineEF is a real reference implementation, not just inspiration

We should retain it as a technical baseline for local monotone encoding.

Its record layout, short-bitvector select and prefetch API are directly relevant.

But its 44-entry layout is not suitable unchanged for mibz2: we need to account for byte recovery, segment flags, global 32-row addressing and the actual Ψ span distribution.

Second: short in-register select is credible

Both PtrHash and Orzo demonstrate practical uses of short Elias–Fano high vectors, with CPU-supported bit selection.

This strengthens the credibility of our EF-12 fallback.

It doesn’t prove EF-12 should dominate FOR-14 or small-plateau encoding.

That remains a decoding-cost question.

Third: independent-query throughput is not dependent traversal throughput

This is the most important limitation of the published performance results.

We should not use PtrHash’s nanoseconds per key to predict mibz2’s nanoseconds per output byte.

The right comparison must reproduce the actual dependency structure:

j_{t+1}=\Psi(j_t)

across many independent chains.

That’s precisely what the existing cooperative walker does.

The literature tells us which techniques are plausible. The walker measurements must tell us which are profitable.

12. Final research verdict

I think this final reading strengthens our confidence in the fixed-line hybrid’s architectural legitimacy considerably.

There is now a working published example of putting Elias–Fano into a single cache line, using register-level select to recover arbitrary entries, and getting near-direct-array throughput in an appropriate streaming workload.

That’s meaningful.

But I’d separate the three leading candidates carefully.

Representation	Why it remains interesting
24-bit direct links	Preserves the simplest navigation dependency chain
16-bit fixed-line hybrid	Reduces working-set size while keeping each group directly addressable
Two-tier translation directory	Exploits abundant translation groups to make the common case both tiny and directly navigable

And the plateau idea from the previous deep dive remains a plausible way to simplify nontranslation payloads, if the corpus supports it.

I would not add more formats before measuring that distribution.

The remaining research frontier

Theoretical compression feasibility is no longer our main uncertainty.

The crucial questions are now:

* How cheaply can each format be constructed directly from L?
* How many dependent instructions lie between loading a record and issuing the next Ψ request?
* Does the smaller representation significantly improve cache residency at full-block concurrency?
* Do nontranslation payload accesses create problematic latency bursts across independent traversal lanes?
* Which representation minimizes complete build-plus-walk time, rather than isolated query time or bits per row?

These are now principally empirical systems questions.

The literature has given us the mathematical framework, design precedents, practical codec techniques and warnings about misleading performance comparisons.

My strongest conclusion from the entire review is that our research has converged on the right level of abstraction: not compression of a pointer array, but fast evaluation of a locally predictable permutation on a CPU that can pursue many independent chains simultaneously.

That is what makes mibz2 interesting.

And it is where the theory genuinely meets the hardware.

Where we go from here

We’ve completed the original literature-review sequence. The next valuable piece of work is a synthesis that compares the actual design choices side by side, identifies which ideas are established prior art and which are mibz2-specific hypotheses, and records the remaining experiments needed to distinguish them.

I’d especially preserve one final question from the PtrHash paper:

If one cache line contains everything needed to recover a successor, how close can that compressed lookup come to the cost of an ordinary pointer load when 32 independent chains are in flight?

That is, to my mind, the most compelling unresolved question left by this literature journey.
