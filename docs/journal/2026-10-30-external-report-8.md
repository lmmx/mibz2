# External research report 8: small-state sublanguages of inverse MTF, decoded in batches (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 7 (`2026-10-30-external-report-7.md`), and it is
recorded verbatim as the earlier reports were.

**The proposal:**
- **The binary case:** for ranks restricted to {0, 1}, the decoder state is one parity
  bit, and the output follows from a prefix XOR.
- **The general case:** for ranks restricted to 0–3 in batches of four, a 256-entry
  table gives both the emitted bytes' selectors and the first four positions' new order.
  It would be applied with `pshufb`.
- **The researcher's correctness check:** the algebra was checked exhaustively for
  four-instruction batches with prefix sizes two, three and four. They call it a check
  on the algebra, not a benchmark.
- **Complications it names:** bzip2's RLE2 and a novelty check against #25.

It is source material for the research portfolio. Its claims are checked against
mibz2's `src/mtf.rs` and against the register in `2026-10-30-research-portfolio.md`
§3.2 (candidate N2b). They are not taken as findings. It is not authorisation to
implement or benchmark anything.

The text reached us as rendered text. Its citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, and the `&#x20;` entities.

## The report, as received

````text
I think we've found something worth pursuing: inverse MTF has small-state sublanguages that can be decoded in batches, without constructing full permutation summaries or replaying the instructions.

The key is to restrict a batch to a small set of ranks. Unlike the general Four-Russians approach, we would not tabulate all 256-symbol MTF states. We'd tabulate the effect of short rank sequences on the first few positions of the current list.

That's a much smaller problem.

### The simplest case is surprisingly elegant: ranks 0 and 1

Suppose the current MTF list starts with two bytes:

\\[ [a,b,\ldots]. \\]

A rank-zero instruction emits the front byte and changes nothing. A rank-one instruction emits the second byte and swaps the first two positions.

Thus, as long as the input ranks are only 0 or 1, the entire decoder state is one bit: whether those two bytes have been swapped an even or odd number of times.

If \\(r_i\in\\{0,1\\}\\), define

\\[ p_i=r_0\oplus r_1\oplus\cdots\oplus r_i. \\]

Then the \\(i\\)-th emitted byte is:

\\[ x_i= \begin{cases} a,&p_i=0,\\\ b,&p_i=1. \end{cases} \\]

The final list orientation is determined by the last parity bit.

This means a run of binary MTF ranks can be decoded using a prefix XOR computation, rather than a sequence of dependent list updates.

And unlike the general chunk-summary approach, this produces the actual output bytes.

This is not yet a performance result. We still need to establish whether the compressed stream contains long enough eligible stretches, and how cheaply we can recognize and batch them.

But it is a concrete mechanism that could reduce the work per symbol.

## The larger idea: a tiny table that emits several bytes

We can generalize the binary case.

Suppose we restrict a batch to ranks 0, 1, 2 or 3. Every such operation acts entirely within the first four MTF positions. The remaining 252 positions are unchanged.

For a batch of four ranks, there are only:

\\[ 4^4=256 \\]

possible instruction sequences.

We can precompute, for each sequence:

- Which four incoming positions will be emitted.
- How the first four positions will be rearranged afterward.

That is an exact transformation, independent of the actual four byte values.

A four-instruction MTF lookup

Example using the four symbolic identities initially occupying the front positions. Both results follow from a single four-rank table entry.

That table needs only about 2 KB if each entry uses four bytes for output selectors and four for the final prefix permutation. It could potentially be made even smaller.

On an x86 CPU with SSSE3, a SIMD byte-shuffle instruction (`pshufb`) could apply the selectors to a register holding the front of the current MTF list.

The resulting implementation might perform one table lookup, a handful of shifts and shuffles, and one packed output store for four MTF instructions.

I checked the mathematical identity exhaustively for four-instruction batches with prefix sizes two, three and four. Every generated symbolic output and final prefix permutation matched ordinary inverse MTF.

That's a correctness check on the algebra, not a benchmark or a demonstration that it beats mibz2.

## Why this is different from the GPU approach

The GPU implementation computes a full 256-entry transformation for a large chunk, composes it with other transformations, and then replays the ranks to emit bytes.

Our proposed small-state lookup instead gives both the emitted bytes' selectors and the updated prefix ordering in one operation.

That could eliminate several dependent MTF updates in common cases.

It is a different cost model.

And it connects with a broader observation from practical Four-Russians algorithms: small, directly usable lookup tables can be preferable to enormous tables over the full state space. The limitation is always the size of the eligible local state and the cost of recognizing it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.sciencedirect.com\&sz=32)

ScienceDirect+1



## The major complication: bzip2's RLE2

There is a catch we cannot ignore.

Bzip2's Huffman symbols do not directly form an ordinary byte-per-rank MTF stream. The special RUNA/RUNB symbols encode repetitions of MTF rank zero, which the decoder reconstructs separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



This means the most attractive binary-rank sequences may already be handled very efficiently, especially when dominated by zeros.

For a useful implementation, we'd need to distinguish contiguous sequences of explicit nonzero MTF ranks from zero-rank repetitions already processed by RLE2.

We also need to know whether batches of low ranks occur frequently enough.

If the Huffman/RLE2 decoder delivers only short runs of eligible ranks, the overhead of testing and assembling each batch may overwhelm the savings.

That's an important falsification condition.

## The next precise research question

I would investigate a small family of tables, rather than commit to the four-rank example.

| Allowed ranks | Batch length | Table entries | Potential                                 |
| ------------- | ------------ | ------------- | ----------------------------------------- |
| 0–1           | 8            | 256           | One-bit parity-state method may be better |
| 0–3           | 4            | 256           | Tiny symbolic-output table                |
| 0–3           | 8            | 65,536        | Larger table, amortizes more operations   |
| 0–7           | 4            | 4,096         | Covers more ranks, but needs wider state  |

These are alternative conceptual designs, not proposed benchmarks. Table sizes exclude encoding choices and alignment.

The decisive census is straightforward: what proportion of real Wikidata nonzero MTF instructions can be covered by contiguous batches of length four or eight under each rank limit, without expanding RLE2 or making the existing Huffman path more expensive?

And then we need to audit what mibz2 already accomplishes with its fused Huffman/RLE2/MTF code before declaring the table a new optimization.

### The finding I'd preserve

I wouldn't call this a breakthrough yet.

But we've progressed from a general question about composing 256-symbol permutations to something much more specific:

Some inverse-MTF instruction sequences form tiny finite-state machines whose outputs and state updates can both be tabulated.

That gives us a plausible CPU mechanism for producing several bytes with less sequential work.

It may be irrelevant if qualifying batches are rare. It may lose to the current decoder's simple updates. It may turn out that SIMD shuffle overhead consumes the savings.

Those are empirical questions—and importantly, they're cheap questions to answer.

I'd keep this as a read-only structural hypothesis until Claude's portfolio audit is complete, with a clear novelty check against #25 and the existing Huffman/MTF fusion.

This is the first thread in our MTF reading that I think offers a concrete alternative to interpreting every rank instruction individually.
````
