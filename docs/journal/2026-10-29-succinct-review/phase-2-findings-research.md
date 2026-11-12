<!--
Provenance: the research chatbot's "Phase II findings and technical handoff", 29 October
2026, the end of its multi-turn research programme after the ψ census, relayed by the user
into the working session to be recorded in full ("please record it all as much as
possible"). The text below is unedited, including its formatting, escapes and citation
links. It is that chatbot's view; the journal entry 2026-10-29-succinct-review.md §14
records which of its claims were checked against their sources and against census v2.
-->

# mibz2 research programme — Phase II findings and technical handoff

29 October 2026 · Succinct permutation navigation · Research synthesis

## Executive finding

The new corpus census suggests a potentially better architecture than the fixed 16-bit Elias–Fano hybrid: a directly indexed, two-tier Ψ representation in which the common translation case requires no secondary lookup at all.

This changes our research agenda meaningfully.

Previously, we considered choosing between fixed 64-byte records, which preserve predictable addressing, and densely packed variable-size records, which introduce an extra lookup.

But the 77.6% translation result suggests a third possibility:

Store an immediately usable translation descriptor in a small fixed-size directory, and access a larger record only for nontranslation groups.

That could substantially reduce memory footprint without charging most navigation steps for an additional dependent lookup.

I now see three candidate architectures worth comparing:

| Architecture                                          | Nominal storage          | Common-case navigation                                    |
| ----------------------------------------------------- | ------------------------ | --------------------------------------------------------- |
| Current combined T-vector                             | 32 bits/row              | One indexed 32-bit load                                   |
| Fixed-line FOR/EF hybrid                              | \~16 bits/row            | One directly addressed record, followed by decoding       |
| Direct translation descriptors + exceptional payloads | Potentially much smaller | One indexed 64-bit descriptor load for translation groups |

The third is a new design hypothesis arising from the census, not a published result or a tested implementation.

We should investigate it alongside the fixed-line candidate, not assume it is superior.

# Finding 1 — A two-tier design can make translation the genuinely cheap case

Imagine partitioning Ψ into groups of 32 rows.

We already know the group index directly:

\\[ g=j\gg5,\qquad s=j\ \&\ 31 \\]

Instead of assigning 64 bytes to every group, give every group one 64-bit directory entry.

Proposed 64-bit descriptor per 32 Ψ rows

Segment marker bitmap

32 bits

Base / payload index

20 bits

Symbol

8 bits

Mode

4 bits

One possible bit allocation, not a finalized encoding. The symbol field is valid directly for symbol-uniform groups; mixed groups require specialized handling.

Translation

20-bit base represents Ψ for all 32 slots

Next = base + slot

No payload lookup

Other formats

Descriptor identifies external FOR/EF record

Next = decode(payload, slot)

Second dependent load

The critical difference from our previous variable-size proposal is that the translation descriptor itself contains the answer.

There is no separate physical-record lookup for this case.

A conceptual implementation is:

```
uint64_t d = directory[row >> 5];
unsigned slot = row & 31;

if (mode(d) == TRANSLATE) {
    next = base(d) + slot;
} else {
    next = decode_payload(d, slot);
}
```

The marker bitmap can also be extracted from the directory entry.

For the translation case, the unpredictable memory dependency is exactly one descriptor load, followed by simple integer operations.

This is substantially more attractive than a variable-length design requiring a descriptor lookup followed by a second lookup on every step.

## The modeled space is surprisingly small

For a full 900,000-row block:

- 28,125 directory entries × 8 bytes = 225 KB.
- If 22.4% of groups need 64-byte payload records, approximately 403 KB of payload.
- Combined illustrative storage: approximately 628 KB, or 5.6 bits per row.

This assumes the reported 77.6% translates into approximately the same fraction of complete 32-row groups, that ordinary nontranslation records use one 64-byte payload, and that exceptional storage is excluded.

Illustrative full-block storage comparison

0 MB0.95 MB1.9 MB2.85 MB3.8 MBCurrent direct24-bit directFixed 16-bit hybridTwo-tier sketch

Only the first three bars have specified fixed-size base layouts. The two-tier figure is a conditional estimate, before exception side storage, alignment and other auxiliary data.

This would be a very substantial reduction.

But it introduces a dangerous new trade-off: approximately 22.4% of navigations might require a second dependent memory access.

We cannot infer whether that trade is profitable from the footprint alone.

This is now one of our most interesting research questions: is a very small direct directory plus occasional dependent payload lookups faster than always loading a larger compressed record?

# Finding 2 — The literature contains two strikingly close precedents

The deeper literature search produced two unusually relevant discoveries.

## A. Gog, Navarro and Petri already describe uniform Ψ blocks

In [Improved and Extended Locating Functionality on Compressed Suffix Arrays](https://users.dcc.uchile.cl/~gnavarro/ps/jda15.2.pdf?utm_source=chatgpt.com) (2015), the authors compare several Ψ representations:

- Elias-based difference codes.
- Patched frame-of-reference coding (OptPFor).
- Partitioned Elias–Fano.
- Uniform blocks requiring no individually stored values.

Their uniform-block case is the same mathematical phenomenon as our translation records: when a group contains all consecutive positions in an interval, the values can be inferred from a base and their ordinal positions.

They also identify exactly the complication we encountered: a block crossing character-class boundaries may not remain monotone and requires special treatment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://users.dcc.uchile.cl\&sz=32)

users.dcc.uchile.cl



This is important for our novelty assessment.

The mathematical insight and the basic compression mechanism are established prior art.

What remains distinctive about our proposal is:

- Making the common translation answer live directly in a fixed descriptor.
- Separating ordinary and exceptional lookup dependency depth.
- Evaluating it for 32-lane, one-shot inverse-BWT traversal.
- Optimizing peak multicore throughput rather than persistent suffix-array indexing.

The 2015 paper reports roughly 200 ns for some isolated Ψ lookups using partitioned Elias–Fano on its tested platform. Its measurements concern different datasets, hardware and implementations, so they aren't predictions for mibz2. But they reinforce that compact Ψ access is a real performance problem, not merely a bit-budget exercise.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://users.dcc.uchile.cl\&sz=32)

users.dcc.uchile.cl



## B. Cache-line Elias–Fano has already been implemented

[cacheline-ef — Ragnar Groot Koerkamp](https://github.com/RagnarGrootKoerkamp/cacheline-ef?utm_source=chatgpt.com)

A particularly relevant existing implementation encodes 44 sorted 40-bit integers in one 64-byte cache line, approximately 11.6 bits per value.

Its fixed record includes a base, packed low parts and unary high parts. It supports indexed access and prefetching and is associated with the PtrHash work.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



This is extremely useful because it establishes that cache-line-granular Elias–Fano is not merely an attractive diagram: there is an existing implementation of the general approach.

However, its parameters differ significantly from ours. We require permutation navigation, segment markers, output symbols, handling of F-symbol boundaries and cheap construction from bzip2 blocks.

The immediate intellectual contribution is recognizing that our proposed format belongs to an established family of cache-line integer codecs, which can provide decoding techniques and performance baselines.

The three strongest prior-art references are now:

| Reference                      | What it contributes                                   |
| ------------------------------ | ----------------------------------------------------- |
| Gog–Navarro–Petri (2015)       | Ψ-specific uniform blocks, EF and patched FOR         |
| Ottaviano–Venturini (2014)     | Locally adaptive partitioned monotone sequences       |
| Groot Koerkamp, `cacheline-ef` | Concrete fixed-cache-line EF indexing and prefetching |

This is enough literature coverage to stop treating the basic encoding concepts as unexplored territory.

# Finding 3 — We need a more exact distinction between one load and one cache-line fetch

The architectural claims can now be made more precise.

## Fixed 64-byte record

For a given row \\(j\\), the record address is known immediately:

\\[ address=base+64(j\gg5) \\]

But decoding a FOR or EF entry may still involve multiple load instructions within that line.

Those load instructions are not necessarily independent of one another, although their memory locations are close.

The advantage is that after the line is fetched into cache, subsequent reads should usually be cache-resident.

## Two-tier directory

For a translation group:

\\[ \text{one indexed descriptor load}\rightarrow\text{arithmetic}\rightarrow\Psi(j) \\]

For a nontranslation group:

\\[ \text{descriptor load}\rightarrow\text{payload address}\rightarrow \text{payload load}\rightarrow\text{decode}\rightarrow\Psi(j) \\]

Navigation dependency comparison

Blue indicates a principal indexed load, amber an additional dependent payload load. Real execution may overlap other independent work.

The core trade-off is now unusually clear.

The two-tier structure might turn most operations into cheap accesses to a small directory while making a minority considerably more expensive.

The fixed-line structure gives every group approximately the same addressing pattern, but uses much more cache capacity.

Neither strategy dominates in theory.

We can formulate a simple mixture model:

\\[ E[T\_{\Psi}] \approx pT\_{\mathrm{translation}} +(1-p)T\_{\mathrm{other}} \\]

with reported \\(p\approx0.776\\).

But this isn't sufficient to predict the 32-lane walk. Unequal latencies, outstanding request limits and exceptional steps can change its throughput nonlinearly.

That's one of the main questions for the experimental handoff.

# Finding 4 — Exact mathematical and physical constraints

I've taken the representation analysis as far as we can reasonably go without the raw Ψ census records and target-machine measurements.

Several details are worth fixing before the research is transferred.

## A. The two-tier directory is a complete format hypothesis, not yet a complete codec

A 64-bit descriptor can plausibly store:

| Field                               | Bits |
| ----------------------------------- | ---- |
| Per-row segment flags               | 32   |
| Base position or payload identifier | 20   |
| Symbol or symbol hint               | 8    |
| Format discriminator                | 4    |
| Total                               | 64   |

For translation groups, the base gives the next row directly.

For nontranslation groups, the 20-bit field can identify a payload. At the maximum bzip2 block size, there are fewer than 28,125 groups, so a payload index easily fits.

However, the payload itself still needs enough metadata to recover its absolute positions.

The proposed FOR-14 and EF-12 record layouts remain candidates, subject to their original span restrictions.

A mixed-symbol record cannot rely on the descriptor's single symbol byte. Such records require a correct alternative symbol-recovery method.

This is the key feasibility condition: every row must recover its link, output byte and marker without assuming that its group contains only one F-symbol interval.

## B. The Elias–Fano span bound needs a small correction

For the proposed EF-12 format, 32 high values are represented as set bits in a 64-bit word.

Using zero-based indexing:

\\[ position_i=h_i+i \\]

The final set-bit position must not exceed 63:

\\[ h\_{31}+31\le63 \\]

Therefore:

\\[ h\_{31}\le32 \\]

With 12 low bits, the representable maximum offset is:

\\[ 32\cdot4096+4095=135167 \\]

So the precise bound for that convention is 135,167, rather than the approximately 131,071 previously quoted.

This doesn't materially change our research outlook, but it's important for an exact codec specification.

## C. The 77.6% figure is a corpus-specific property, not an entropy bound

Our two-tier model predicts only around 5.6 bits per row before exceptions.

That isn't inconsistent with the information-theoretic argument: the data may contain considerable redundancy, and the translation records exploit it.

But it would be incorrect to infer that arbitrary BWT permutations can be represented at 5.6 bits per row.

For a near-uniform byte distribution, the combinatorial information content approaches eight bits per row.

The measured translation coverage is the essential reason such a small representation might work on this particular corpus.

## D. A fixed 64-byte record does not physically shrink when its contents are simpler

This is the most important difference between our two candidate architectures.

In the fixed-line representation, a translation group still occupies 64 bytes.

Its simplicity reduces decoding work, but not memory footprint.

In the two-tier representation, a translation group occupies just its descriptor.

Its simplicity reduces both decoding work and storage.

The price is that nontranslation groups require another navigation stage.

The question has therefore evolved from choosing an integer codec into choosing where to place the boundary between immediate answers and compressed answers.

# Finding 5 — Construction might be simpler than the compressed representation initially suggests

There is another promising deduction from the algebra.

Since Ψ is composed of sorted occurrence lists, the decoder can construct those lists by scanning the BWT last column in increasing row order.

Given character counts and cumulative boundaries, the occurrence number of a symbol determines the destination within its F interval.

Conceptually:

```
for each L position i, in ascending order:
    c = L[i]
    k = next occurrence number of c
    append i to the Ψ list for c at ordinal k
```

The appended positions are increasing by construction.

Therefore, the monotone groups need not be discovered through an expensive sorting pass.

A streaming encoder could maintain one partial group per symbol, flush a completed group into its representation, and continue.

The raw accumulation workspace for 32 positions across 256 symbols would require about 32 KB using 32-bit integers, ignoring counters and metadata.

That doesn't prove construction would be cheap. There are scattered destination writes, per-symbol state, format selection, and segment-marker integration.

But it gives us a plausible linear, streaming construction algorithm without sorting the permutation.

This is an important advantage over a generic permutation-compression algorithm.

It is directly attributable to the BWT's stable ordering property.

The research question becomes whether that streaming codec can outperform or approach the existing parallel scatter builder while constructing fewer bytes.

# Finding 6 — Three architecture candidates, no obvious dominant choice

The main design space is now sufficiently defined.

A. Direct packed Ψ

24 bits/row

One ordinary indexed load per navigation step. Minimal decoding, relatively straightforward construction. Its storage reduction is limited to 25% versus the current T-vector.

B. Fixed-line hybrid Ψ

\~16.12 bits/row modeled

Every group is directly addressed. Translation, FOR and EF records share a predictable 64-byte physical stride. All ordinary data is contained in one record.

C. Translation directory + payloads

\~5.6 bits/row illustrative

Translation groups return their next row directly from a small descriptor. Other groups require a second dependent payload lookup. Much greater potential compaction, but considerably less uniform latency.

The two-tier design is now my most interesting new research result, but it is not my predicted speed winner.

The direct 24-bit representation minimizes work per step.

The fixed-line representation minimizes variability in memory addressing.

The two-tier representation minimizes redundant storage while preserving a cheap common case.

We have identified three quite different points on a plausible space–time–MLP frontier.

# Research questions that are now decisive

I would narrow the remaining research to six questions, rather than continuing to accumulate possible encodings.

| Question                                                     | Why it matters                      | What settles it                        |
| ------------------------------------------------------------ | ----------------------------------- | -------------------------------------- |
| Is translation coverage stable per block?                    | Determines common-case reliability  | Per-full-block coverage distributions  |
| Can mixed-symbol groups be handled cheaply?                  | Correctness and output-byte latency | Exact format and lookup accounting     |
| How expensive are nontranslation payload loads?              | Determines two-tier viability       | Dependent-load microbenchmarks         |
| What is the true physical footprint?                         | Establishes cache-residency benefit | Exact record accounting and allocation |
| Can Ψ groups be encoded cheaply in one streaming pass?       | Construction is a major cost        | Builder algorithm and measured traffic |
| Does any representation improve best-concurrency throughput? | Determines practical benefit        | Full-system measurements               |

The first four are representational questions.

The last two require implementation evidence.

Beyond those, further speculation has diminishing research value until we see actual data.

# Final literature assessment

The literature search is now sufficient to establish the main prior-art boundaries.

| Research                           | What it establishes                                        |
| ---------------------------------- | ---------------------------------------------------------- |
| Seward (2001)                      | Space–time trade-offs for inverse BWT, including `indexF`  |
| Barbay–Navarro (2009/2013)         | Compressed permutations exploiting increasing subsequences |
| Ottaviano–Venturini (2014)         | Partitioned Elias–Fano for clustered monotone lists        |
| Gog–Navarro–Petri (2015)           | Ψ-specific patched FOR, Elias–Fano and uniform blocks      |
| Barbay's permutation survey (2014) | Strict ascending runs and representations of their heads   |
| Groot Koerkamp's `cacheline-ef`    | Concrete cache-line-bounded EF implementation              |

Barbay's survey explicitly discusses compressing strict ascending runs and then compressing the sequence of run heads. That is especially close to the mathematical logic behind our translation descriptors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.barbay.cl\&sz=32)

barbay.cl



More recent Elias–Fano work also continues to explore compact random-access representations; for example, a 2025 SEA paper investigates using EF to compress rank/select support. Its emphasis on the cost of select operations reinforces the importance of our register-local versus dependent-memory distinction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://drops.dagstuhl.de\&sz=32)

DROPS



I would not currently claim novelty for uniform blocks, local FOR, cache-line EF, or compressed increasing subsequences individually.

The interesting potential research contribution is their use in a one-shot cooperative inverse-BWT architecture with deliberate separation of one-load and two-load navigation cases.

# Coordinator handoff — research programme limit reached

We have reached the point where further progress depends primarily on corpus measurements, exact representation accounting and target-CPU microbenchmarks. I can continue theorizing, but doing so would not improve the evidence nearly as much as those measurements.

Here is the concise handoff I would preserve.

# mibz2 succinct Ψ research handoff

**29 October 2026**

## Principal conclusion

The corpus census materially validates the feasibility of compressing the Ψ navigation table using monotone 32-row groups.

Reported results:

- 98.11% of rows fit FOR-14.
- 1.40% fit EF-12.
- 0.49% require exceptions.
- 77.6% are translation-eligible.
- Fixed-line representation estimated at 16.12 bits/row, including modeled exceptions.

These are representation measurements or estimates, not performance results.

## New candidate discovered during further research

**64-bit translation directory with external nontranslation payloads.**

Each group of 32 Ψ rows receives one directly indexed 64-bit descriptor, conceptually containing:

- 32 segment-marker bits.
- 20-bit translation base or payload identifier.
- 8-bit symbol metadata.
- 4-bit format discriminator.

Translation groups recover their next row through `base + slot`, requiring no secondary payload access.

Other groups decode a FOR/EF/exception payload through an additional dependent lookup.

Using the reported 77.6% translation coverage, an illustrative space estimate is approximately **5.6 bits/row before exceptions and auxiliary overhead**. This is conditional, not a validated footprint.

## Architectural comparison

**A. 24-bit direct links:** one primary random load, minimal decoding.

**B. Fixed 64-byte hybrid:** approximately 16 bits/row, directly addressed records, format-dependent register decoding.

**C. Two-tier translation directory:** potentially much smaller, one-load common case and two-load nontranslation case.

No candidate currently has a demonstrated speed advantage over the existing 32-bit table.

## Required research measurements

1. Translation coverage and run-length distributions for each full-sized block, including less-compressible inputs.
2. Exact record counts by translation, FOR, EF, boundary exception and span exception.
3. Physical footprint including alignment, payloads, markers, symbol recovery and other metadata.
4. Dependency-chain latency and instruction counts for each record format.
5. Streaming construction cost from the BWT last column.
6. End-to-end throughput across concurrency configurations, not just isolated query latency.

## Correctness requirements

Each row must recover the appropriate Ψ successor, output byte and segment marker.

Mixed F-symbol groups need exact symbol handling. Traversal orientation, primary-index semantics and periodic inputs must be preserved.

For the proposed 64-bit EF high-vector with 32 entries and 12 low bits, the maximum representable relative offset is 135,167 under the specified zero-based encoding.

## Relevant prior art

- Seward, *Space-Time Tradeoffs in the Inverse B-W Transform* (2001).
- Barbay and Navarro, *Compressed Representations of Permutations, and Applications* (2009).
- Ottaviano and Venturini, *Partitioned Elias-Fano Indexes* (2014).
- Gog, Navarro and Petri, *Improved and Extended Locating Functionality on Compressed Suffix Arrays* (2015).
- Ragnar Groot Koerkamp, `cacheline-ef`.

Uniform blocks, partitioned EF, patched FOR and strict-run compression have established prior art. The research contribution, if any, would concern their specialization and hardware behaviour under cooperative inverse BWT.

## Decision boundary

The representation research has reached its analytical limit with the information presently available.

The decisive next evidence is actual corpus-level format accounting and the CPU cost of obtaining the next Ψ address.

The fixed-line hybrid and the new two-tier directory should both remain candidate architectures until those measurements distinguish them.

**Research hypothesis:** A compressed Ψ representation that keeps translation answers in directly indexed descriptors may improve whole-decoder throughput by greatly increasing cache residency, provided the remaining dependent payload lookups and construction work do not erase the benefit.

## My final research verdict

We started with the question of whether succinct structures could help mibz2. We now have a much sharper understanding.

The strongest result is not Elias–Fano itself. It's the realization that a large fraction of the actual Ψ permutation may be represented as directly navigable translations, with the rest handled by more general monotone encodings.

That has led to two viable classes of compact representation and a third, more aggressive two-tier hypothesis.

The mathematics, literature review, bit-budget analysis and initial corpus evidence are now sufficiently developed to justify an implementation study.

This is the appropriate handoff point. I do not think another round of ungrounded representation brainstorming would add comparable value. The next major research advance will come from measuring the physical encoding and its navigation dependency costs.
