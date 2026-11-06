# External research report 6: the researcher's study session on the algebra of inverse MTF (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 5 (`2026-10-30-external-report-5.md`), and asked for it
to be recorded verbatim. It is one study session in five parts, in the order received:

1. **The reading plan:** Edwards–Vishkin 2014 §3.2.2, Weißenberger–Schmidt 2024, and the
   Mäkinen–Navarro survey. It ends with the "control against data computation" framing
   of a chunk of MTF instructions.
2. **Reading session I:** the algebra of inverse MTF, including the stable move-to-front
   characterisation L_out = D ‖ (L_in \ D).
3. **The researcher's reading of bzip2gpu's full source.** The maintainer supplied the
   source; their note "(I provided full source code)" is kept where it appeared.
4. **The maintainer's question,** whether the basis of the work had moved to GPUs.
5. **The researcher's answer:** no. The target stays ordinary CPUs (four-core laptops,
   the i9 for controlled experiments), Rust, existing `.bz2` files, and baseline mibz2
   0.2.1.

It is source material for the research portfolio. Its claims are checked against the
primary sources and against this journal's history in the portfolio's register
(`2026-10-30-research-portfolio.md` §2.3 and §3.2). They are not taken as findings. It
is not authorisation to implement or benchmark anything: the session says so itself.

The text reached us as rendered text. Its citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, the `&#x20;` entities, the
`utm_source` parameters and the inline image links.

## The session, as received

`````text
Oh, excellent. Back to the stacks. 📚

I think we should resist another broad literature survey. We've accumulated plenty of references; what we need now are papers whose actual constructions might reveal a new algorithm.

After searching around parallel decompression, prefix computation and symbolic state transformations, I've found a particularly good reading trail. And one of the older sources contains a detail I think we should examine much more carefully.

## The reading desk

[PPT - Truly Parallel Burrows-Wheeler Compression and Decompression PowerPoint Presentation - ID:2733640](https://images.openai.com/static-rsc-4/pQzDtbeHbKpA4H_2vort2zeLVNQBT71MC10TRAw3sXjoPBwNO8TCJ2urk3xt4liPdx-ieaVyAFsGHvyFAa8x09hEvKuJJruMd2oQRvsXAzCjsRKuDSk4I5mWwdrkKOok8f1SRUkBpvaqQEwzqLCDrqZabeY2NaDNd8PXowYGWcQ?purpose=inline)

READ FIRST · ALGEBRAIC FOUNDATIONS

[Parallel Algorithms for Burrows–Wheeler Compression and Decompression](https://www.umiacs.umd.edu/~vishkin/XMT/parallelBW-TCSpreprint30August2013.pdf?utm_source=chatgpt.com)

Edwards & Vishkin, 2014

In particular, §3.2.2 on inverse MTF. The authors explicitly turn MTF updates into permutations and use an associative prefix computation to recover cumulative state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.umiacs.umd.edu\&sz=32)

umiacs.umd.edu



This is the most direct mathematical precedent for our chunk-summary question.

READ SECOND · IMPLEMENTATION PRECEDENT

[Massively Parallel Inverse Block-sorting Transforms for bzip2 Decompression on GPUs](https://doi.org/10.1145/3673038.3673067?utm_source=chatgpt.com)

Weißenberger & Schmidt, ICPP 2024

This one implements parallel inverse BWT and inverse MTF for actual bzip2 archives, rather than just a theoretical transform. Its reported results include 2,400 MB/s combined inverse-BWT/inverse-MTF throughput on an A100 GPU.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://bibbase.org\&sz=32)

bibbase.org



The source repository is [bzip2gpu](https://github.com/weissenberger/bzip2gpu?utm_source=chatgpt.com). I haven't yet verified the internal CUDA kernel design, so that is precisely what we'd read for.

[Compressed Compact Suffix Arrays Veli Mkinen University of](https://images.openai.com/static-rsc-4/l8FXa9v8IIdyfy2K0Gwr1aSBU5Vu6DNJKqb5Uzt02MtkzdN92Ist6qlzFwu3alV0qDjbor_KCO5ySyKyjjZ2D8fEru-vveDo9m8lLA5aRyhjbqT3XB4jiV3eFajuppvG2gyUgcBefHJe-rL4IU8iTSKtfVYPT7UOasM26N2o8gI?purpose=inline)

READ THIRD · THE UNEXPECTED CONNECTION

[Compressed Full-Text Indexes: four-Russians and MTF state tables](https://www.cs.helsinki.fi/u/vmakinen/papers/survey.pdf?utm_source=chatgpt.com)

Mäkinen & Navarro, survey

The intriguing passage describes precomputing answers for short compressed blocks while recording the starting MTF state at block boundaries. The lookup is parameterized by the encoded block and the incoming permutation state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.helsinki.fi\&sz=32)

cs.helsinki.fi



That is not a fast bzip2 decoder as such. But it connects our chunk-summary question to tabulation and state-space reduction, rather than just parallel scan.

## The idea I'd like to read around

The Edwards–Vishkin paper says that a sequence of MTF instructions can be represented as a composition of permutations.

That's true, but rather expensive if every intermediate result is a full 256-element permutation.

Here's the question that struck me while looking at the four-Russians precedent:

What if we don't summarize the entire MTF list? What if we summarize only the information that the next chunk could possibly observe?

An MTF operation with rank \\(r\\) accesses the first \\(r+1\\) list entries. Suppose a chunk's largest rank is \\(m\\).

Then every operation in that chunk acts entirely within the first \\(m+1\\) entries.

More importantly, every output symbol produced by that chunk is one of the symbols that occupied those initial \\(m+1\\) positions.

That means the entire chunk can be simulated symbolically, using position labels rather than actual byte values.

For example, let the incoming MTF list begin:

\\[ [a,b,c,d,e,\ldots] \\]

A chunk of rank instructions

\\[ [2,0,1] \\]

produces:

\\[ [c,c,a] \\]

and leaves the prefix as:

\\[ [a,c,b,d,e,\ldots]. \\]

But we could instead perform the operation on symbolic labels:

\\[ [0,1,2,3,4,\ldots] \\]

and obtain:

\\[ \text{output positions}=[2,2,0], \\]

with an outgoing state permutation for the first three positions.

The actual decoded bytes can then be recovered from the incoming list.

That's a potentially important separation:

- Control computation: what positions does the rank sequence select?
- Data computation: which actual bytes occupy those positions?

The former depends only on the encoded rank sequence. The latter depends on the incoming MTF state.

This gives a compact mathematical way to describe a chunk:

\\[ (\text{output position sequence},\ \text{outgoing prefix permutation}). \\]

Now, the problem is obvious: storing the output position sequence may cost nearly as much as the original ranks, and obtaining the summary may require essentially all the original work.

So this isn't yet a breakthrough.

But it suggests a new family of questions that the straightforward 256-entry permutation approach obscures.

Can the output-position sequence be represented by runs, repeats or a tiny instruction program? Can multiple chunks share a precomputed transition? Can the small maximum ranks of actual Wikidata sequences keep the state space manageable?

And can the whole operation be cheaper than just performing the MTF updates directly?

That's what I want the papers to help us determine.

## One mathematical warning worth carrying into the reading

We mustn't confuse a small maximum rank with a small number of distinct symbols.

A chunk with ranks `[1,1,1,1,...]` can repeatedly alternate or cycle through a small working set, but the state transformations and output still need to be calculated.

Conversely, a chunk with a single rank 200 touches a large prefix even if almost all other ranks are zero.

Those cases have very different summary costs.

So I would pay attention to three statistics together: maximum rank, distinct symbolic positions selected and compressibility of the symbolic output sequence.

That is a more discriminating lens than a histogram of MTF ranks alone.

## Why these three, rather than another 20 papers?

They each answer a different question.

The 2014 paper asks: What algebra makes the serial operation associative?

The 2024 implementation asks: What representation of that algebra actually runs quickly on hardware?

The succinct-index survey asks: Can the state space be reduced enough to replace computation with tabulated answers?

Together they offer a route from mathematical possibility to a plausible implementation mechanism, while leaving room for a genuinely different idea to emerge.

I'd start with Edwards and Vishkin §3.2.2. Not the abstract or the performance claims—the actual permutation-composition construction. It's the most likely place to discover whether we've been thinking about reverse MTF at the wrong level of abstraction.

And then I'd follow whichever mathematical detail refuses to leave us alone.
`````

`````text
# Reading session I: the algebra of inverse MTF

We have the papers. No need for you to find copies yet.

I've read the relevant constructions in Edwards–Vishkin (2014), Patel's 2012 GPU thesis, and the Navarro–Mäkinen succinct-index survey. The newer 2024 bzip2 GPU paper's abstract is accessible, although I haven't obtained enough of its implementation to make claims about the kernels.

And the reading has produced something useful: we can now identify exactly where the older parallel inverse-MTF algorithms pay their overhead—and a more compact way of representing their intermediate states.

## 1. Edwards and Vishkin: a mathematically clean solution with a serious constant factor

Primary source: [Parallel Algorithms for Burrows–Wheeler Compression and Decompression](https://www.umiacs.umd.edu/~vishkin/XMT/parallelBW-TCSpreprint30August2013.pdf?utm_source=chatgpt.com), §3.2.2, page 20

Their construction is straightforward once you adopt the right abstraction.

Each MTF rank \\(r_i\\) defines a permutation \\(P_i\\) of the list positions. The state before any particular symbol can be found by composing all the preceding permutations:

\\[ Q_i=P\_{i-1}\circ\cdots\circ P_1\circ P_0. \\]

Function composition is associative. Therefore, a parallel prefix algorithm can compute the cumulative permutations.

The paper explicitly states these costs:

\\[ \begin{aligned} \text{Time}&=O(\sigma\log n),\\\ \text{Work}&=O(\sigma n), \end{aligned} \\]

where \\(\sigma\\) is the alphabet size.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.umiacs.umd.edu\&sz=32)

umiacs.umd.edu



For mibz2, \\(\sigma\\) can reach 256.

That makes the distinction between theoretical work-optimality for a constant alphabet and practical CPU efficiency substantial. An operation involving 256 entries may be considered constant-time in the paper's asymptotic model, but it is not cheap relative to a conventional inverse-MTF update.

This is the first solid conclusion:

The associative algebra is established. The missing innovation is an economically compact representation of its elements.

## 2. Patel's GPU thesis: the implementation exposes the cost directly

Primary source: [Parallel Lossless Data Compression on the GPU](https://escholarship.org/content/qt7bs1x3qn/qt7bs1x3qn_noSplash_09e55cf537ecac8b90b470830b3121dd.pdf?utm_source=chatgpt.com), Chapter 3, Algorithms 5 and 6

Patel's implementation divides the rank stream into 64-symbol chunks. Each chunk independently computes an MTF-list permutation from an identity initial state.

Then the chunks' effects are combined by composition.

What the existing parallel construction does

Schematic of the paper's per-chunk state summaries; not a measured mibz2 data path.

The decisive detail is Algorithm 6. For every composition, it performs:

\\[ \text{result}[i]=A[B[i]] \\]

for all 256 positions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://escholarship.org\&sz=32)

escholarship.org



That is a full permutation composition—even when a chunk barely modifies the MTF list.

Patel's performance measurements also provide a warning: on the tested 2012 GPU system, reverse MTF plus Huffman was slower than its CPU counterpart. The authors attributed a substantial portion of their decompression runtime to the reverse-MTF stage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://escholarship.org\&sz=32)

escholarship.org



So we're not looking at a forgotten high-performance CPU implementation.

We're looking at a mathematically valid parallel algorithm whose state representation is potentially much too large.

## 3. Navarro and Mäkinen: the precomputation alternative is severely constrained

Primary source: [Compressed Full-Text Indexes](https://www.cs.helsinki.fi/u/vmakinen/papers/survey.pdf?utm_source=chatgpt.com), pages 43–44

The survey describes a Four-Russians-style lookup table that takes the compressed block, its offset and the incoming MTF state as inputs.

It also explains why that construction cannot simply scale to large alphabets.

The lookup table's state includes a permutation of the alphabet. The theoretical space guarantee requires an alphabet small enough that the state representation fits within logarithmic space constraints.

The survey explicitly notes that the practical FM-index implementation couldn't use the full theoretical table and instead decoded and scanned the compressed blocks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.helsinki.fi\&sz=32)

cs.helsinki.fi



For our purposes, that rules out naively tabulating arbitrary 256-symbol MTF states as an attractive CPU strategy.

It does not rule out tabulation over a much smaller, demonstrably common subset of states.

That distinction is worth preserving.

## 4. A mathematical reduction the papers don't exploit explicitly

Here is the result I find most interesting.

A general permutation of 256 items has \\(256!\\) possible states. That is an enormous state space.

But an MTF chunk does not produce an arbitrary permutation. It produces a stable move-to-front permutation.

Suppose the incoming list is:

\\[ [a,b,c,d,e,f,g]. \\]

The chunk selects the symbols:

\\[ [c,e,c,e]. \\]

After all four operations, the outgoing list is:

\\[ [e,c,a,b,d,f,g]. \\]

Notice what happened.

The selected symbols moved to the front in reverse order of their last occurrence. Every other symbol retained its original relative order.

Therefore, the final list can be reconstructed from just the ordered set of distinct selected symbols.

Let \\(D\\) contain those symbols, ordered by decreasing last-occurrence time. Then:

\\[ \boxed{ L\_{\mathrm{out}} = D\\;\Vert\\; (L\_{\mathrm{in}}\setminus D) } \\]

where \\(\Vert\\) denotes concatenation and the remaining elements retain their incoming order.

This is an exact characterization, not a performance hypothesis.

The stable move-to-front property

This tells us something more specific than merely bounding the affected prefix by the maximum rank.

A chunk's final transformation can be described by the identities of the distinct items it moved, in their final recency order.

If it moves \\(d\\) distinct items, the summary needs \\(d\\) identities, rather than a general 256-entry permutation.

Those identities can be expressed as positions in the incoming list, making the representation independent of actual byte values.

This could give much smaller intermediate summaries.

But there are two substantial problems.

First, discovering those distinct symbolic identities from the rank stream has a computational cost.

Second, combining two compact summaries is not just concatenating their moved-item lists. We must translate their identities through the earlier transformation and eliminate duplicates.

The result is still another compact stable-move-to-front summary, but the composition may require additional searches or bookkeeping.

So the algebra is closed under composition, and its representation can be smaller.

What remains unknown is whether the compact representation can be built and composed more cheaply than ordinary decoding.

That is a genuinely sharp research question.

## 5. Where I think the breakthrough condition lies

The preceding result suggests measuring two quantities for each rank chunk:

\\[ m=\max_i r_i,\qquad d=\text{number of distinct symbolic items selected}. \\]

Here \\(m+1\\) bounds the affected prefix, while \\(d\\) bounds the length of the distinct-moved-item summary.

For example, a chunk could have maximum rank 200 but select only one distinct symbolic item. That gives a potentially tiny transformation summary despite the large maximum rank.

Conversely, a chunk with many different small ranks could have a larger summary than its maximum-rank histogram alone might suggest.

This is an improvement over our earlier research question.

We should investigate the actual distribution of \\(d\\), not just the distribution of \\(m\\).

And there is an important third quantity: how many decoded symbols can be emitted using that summary without reconstructing the full intermediate list for every output position?

An algorithm that computes chunk boundary states quickly but still executes every MTF operation to emit output may improve parallelism without reducing CPU work.

A true breakthrough would need to use the compact summary to eliminate some of that work too.

## 6. The 2024 GPU paper remains the next essential source

The public record establishes that Weißenberger and Schmidt implemented inverse MTF and inverse BWT for standard bzip2 and reported substantial GPU throughput.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://bibbase.org\&sz=32)

bibbase.org



But I couldn't reliably inspect the CUDA source through the available web access today. So I won't infer its algorithm from performance figures or the older GPU implementation.

That is the one source for which an accessible PDF or source archive from you could be useful later. No need to obtain it immediately, because the older papers already provide plenty of mathematical material to investigate.

## Where this leaves our reading

Our first focused session has produced three useful results:

- Edwards–Vishkin establishes permutation composition but explicitly pays alphabet-sized work.
- Patel's GPU implementation makes that overhead concrete: 256-element summaries composed with indexed array operations.
- The special structure of MTF permits exact stable-move-to-front summaries of size proportional to the number of distinct items moved.

The third is the interesting opening.

My preference for the next research session is to study how two such compressed summaries compose, derive a realistic operation-count bound, and investigate whether we can recover output symbols without expanding all intermediate states.

That is where a mathematical possibility could turn into something much more consequential.

No code, no workstation experiments, and no interruption to Claude's portfolio reconstruction.

Just the algebra—and whether it offers a fundamentally cheaper way to decode.
`````

(I provided full source code)

`````text
This is exactly the source we needed. I've now inspected the inverse-MTF and inverse-BWT implementations, including the actual CUDA kernels rather than inferring their algorithms from the paper's performance claims.

The central finding: the 2024 decoder does implement parallel inverse MTF using full 256-element permutations and prefix scans. It does not use the compact stable-move-to-front summaries we've been investigating.

That makes the latter a genuine representation-level difference from this particular implementation—although not yet evidence of a faster CPU decoder.

## 1. What the GPU inverse-MTF algorithm actually does

The important implementation is `src/imtf.cu`.

It divides decoding into three phases:

1. Compute independent chunk transformations. `private_perm` begins with the identity MTF list, processes the rank instructions in a chunk, and writes a 256-byte permutation.
2. Compute chunk-entry states. `scan_perms` composes the transformations using a Blelloch-style parallel prefix scan; `concat_stacks` applies the resulting permutations to each bzip2 block's actual initial symbol list.
3. Decode the chunks. `private_perm_write` replays the rank instructions, now starting with the correct incoming MTF state, and replaces ranks with decoded bytes.

The source sets:

```
#define BLOCK_SIZE 64
#define PRIVATE_PERM_SIZE 10000
```

So the configured independent chunk size is 10,000 ranks, not 64. The 64 is the CUDA kernel's thread-block dimension.

That corrects a detail from our earlier reading of the GPU thesis.

The permutation composition itself is explicit:

```
aux[block_b + i] = perms[block_a + perms[block_b + i]];
```

for all 256 entries.

Thus the newer implementation still pays for full-sized permutation composition.

More importantly, it processes the MTF instructions twice: once to calculate each chunk transformation and again to emit the decoded symbols.

That is a significant clue about what this approach optimizes. It exposes parallelism across chunks, but does not eliminate the ordinary per-symbol MTF work.

For a four-core CPU targeting lower CPU-seconds per GB, that is a serious obstacle.

## 2. The inverse-BWT algorithm is equally revealing

In `src/ibwt.cu`, the implementation uses a different architecture from mibz2's cooperative walker.

The GPU inverse-BWT pipeline

Stable sort BWT rows

`cub::DeviceSegmentedSort::StableSortPairs` constructs the permutation from the last column and row indices.

Rank independent sublists

`helman_jaja_local` follows links from regularly spaced starting rows, recording local ranks and sublist identities.

Order the sublists

`helman_jaja_reduce` traverses a reduced list of sublist successors.

Scatter the output

`add_permute` combines local and sublist ranks to place characters at their final positions.

The source sets `LIST_DISTANCE = 32`, creating approximately 28,125 candidate sublists for a 900,000-row block. The reduced-list processing remains serial within each block, although different blocks can run independently.

This is practical Helman–JáJá list ranking. It confirms the approach can be implemented for standard bzip2 blocks.

But it also identifies its cost: constructing the permutation through a full segmented sort, storing ranks and sublist identifiers, and performing a final scattered output pass.

Mibz2 already obtains traversal parallelism through segmented walks without this general sorting and ranking machinery.

So I don't see an immediate CPU breakthrough in copying the GPU inverse-BWT algorithm.

The interesting precedent is its explicit separation of discovering traversal positions from placing output bytes. That is useful when thinking about alternative reconstruction algorithms.

## 3. The strongest new question is narrower than I initially expected

The GPU code reinforces a distinction we have been developing.

A chunk of MTF instructions induces a permutation. But the source materializes that permutation as 256 bytes, and it replays the instructions to recover output.

Our compact-summary idea would represent the final transformation as the distinct moved items, expressed relative to the incoming list.

That suggests two questions:

Can we construct the compact state summary more cheaply?

A 10,000-rank chunk can touch all 256 symbols, in which case the summary may offer little compression. We need the actual distinct-item distribution, not just evidence that the encoding favours small ranks.

Can we recover the output more cheaply?

Even a tiny chunk summary doesn't automatically let us skip decoding individual MTF ranks. The output sequence contains information about intermediate states, not merely the final state.

This second question is the harder one.

A CPU algorithm that calculates chunk states cheaply but then replays every symbol may use more CPU than mibz2's current serial MTF implementation, exactly as the GPU approach does.

For that reason, I'd now distinguish compact boundary-state composition from work-reducing symbol reconstruction as separate hypotheses.

The first is mathematically interesting and potentially useful for parallelization. The second is what could produce the breakthrough we're actually after.

## 4. A source-level observation about the front end

The Huffman decoder in `src/decoder.cu` launches:

```
decode_block<<<num_blocks, 1, ...>>>(...);
```

That is one GPU thread per compressed bzip2 block for the Huffman/RLE2 stage.

So the implementation is not uniformly parallel within every decompression stage.

It uses coarse block parallelism for Huffman decoding, chunk-level parallelism for inverse MTF, and sublist parallelism for inverse BWT.

This matters when interpreting its reported throughput. The implementation demonstrates an effective heterogeneous GPU pipeline, not necessarily an efficient parallel algorithm for every stage within an individual block.

Nor can its GPU performance figures be used as evidence that its inverse-MTF algebra saves CPU work.

## 5. What this source establishes for our research portfolio

| Question                                            | What the code establishes                                |
| --------------------------------------------------- | -------------------------------------------------------- |
| Does the GPU decoder use full permutations for MTF? | Yes, 256-byte transformations                            |
| Does it use associative prefix composition?         | Yes, Blelloch-style scan                                 |
| Does it avoid replaying the rank instructions?      | No, it processes them again for output                   |
| Does it use compact moved-item summaries?           | Not in the supplied implementation                       |
| Does its inverse BWT use fragment/list ranking?     | Yes, Helman–JáJá with 32-row splitter spacing            |
| Does it eliminate general permutation construction? | No, it builds the mapping using segmented stable sorting |
| Is there demonstrated lower CPU work for mibz2?     | No; no corresponding CPU experiment is provided          |

## My updated research judgment

This is a very useful negative precedent.

It tells us that a successful GPU implementation can afford algorithmic overhead that would probably be unacceptable for a four-core laptop CPU.

But it also leaves open a deeper question:

Can we determine the output of a sequence of MTF operations without materializing the intermediate full list after every operation—or performing a second complete replay?

The compact state-summary result is promising for representing what changed. What we still lack is an equivalently compact method for representing what was emitted.

That's where I'd concentrate our mathematical reading next: algorithms for dynamic selection, move-to-front transformations, persistent permutations and batched rank-to-symbol queries.

The supplied code gives us enough evidence to stop speculating about what the 2024 GPU implementation does. And it makes the next challenge considerably more precise.

No additional source files needed for this part. Thanks for providing the full repository rather than just the paper; it resolved several important uncertainties.
`````

> ok, note that we are not going to use gpu, I assume you didn't change the basis of our work to that

`````text
Correct. We are not changing the basis of the research to GPU computing.

Our target remains:

- Hardware: ordinary CPUs, particularly four-core laptops, with the i9 workstation for controlled experiments.
- Software: Rust, decoding existing standard `.bz2` files.
- Objective: a breakthrough in full-decoder throughput, preferably with equal or lower CPU-seconds per GB and no material increase in memory.
- Baseline: mibz2 0.2.1, including its shipped cooperative decoder.

I examined the GPU implementation strictly as algorithmic prior art, not as a candidate architecture.

In particular, we wanted to discover whether its authors had found a way to eliminate dependent operations, compress MTF state transformations or reconstruct output more efficiently.

What we learned is that much of its performance comes from exposing massive parallelism, often at the cost of additional work. That's interesting scientifically, but generally the opposite of what we're pursuing on a four-core CPU.

Our guiding question remains: Can we make the CPU do fundamentally less work per decoded byte?

We'll continue reading GPU literature where it contains transferable mathematics, but we won't pursue GPU-specific implementations, CUDA optimizations or solutions that depend on thousands of concurrent threads.

We're looking for a better algorithm, not a bigger machine.
`````
