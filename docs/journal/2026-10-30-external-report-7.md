# External research report 7: the researcher's reading session II, on recency, order statistics and inverse MTF (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 6 (`2026-10-30-external-report-6.md`), and asked for
it to be recorded verbatim. It has two parts, in the order received:

1. **The reading plan:** MTF as a recency-ordering problem. Its three readings are
   Sleator–Tarjan 1985, Niemi 2020 §2.4 (Move-From-Front), and SwiftShader's
   timestamp-based MTF.
2. **Reading session II:**
   - timestamps and order-statistic trees as alternative implementations, not less work;
   - the exact recency formulation;
   - Move-From-Front, which needs information bzip2 does not store;
   - the symbolic (O, P) decomposition of a chunk;
   - the bit-parallel slot universe, and the hot-prefix hybrid (which the researcher
     declines to reopen without a new mechanism, given #25);
   - a sharpened breakthrough condition: a representation S(R) yielding both the state
     change T_R and the emitted symbols O_R at less work than sequential
     interpretation.

It is source material for the research portfolio. Its claims are checked against the
primary sources and against this journal's history in the portfolio's register
(`2026-10-30-research-portfolio.md` §3.2). They are not taken as findings. It is not
authorisation to implement or benchmark anything.

The text reached us as rendered text. Its citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, the `&#x20;` entities, the
`utm_source` parameters and the inline image links.

## The session, as received

````text
I’d take us somewhere slightly unexpected next: MTF as a recency-ordering problem, rather than as a permutation-composition problem.

The literature search turned up a promising way to change our perspective. MTF is fundamentally a list ordered by the most recent access to each symbol. That means we may be able to learn more from order-statistic and timestamp-based data structures than from another parallel prefix paper.

## Our next three readings

[(PDF) Performance of the Move-To-Front Algorithm with Markov-Modulated Request Sequences](https://images.openai.com/static-rsc-4/ON1rPj1VBb6mn2yQmCNQL5SumXD7ArL3TuN9qL24hjzOOLA7nUxN5An4hUDNj0apx_bgBsRp99hETR8TtBY2JM28WBOmE0bd6T9BGwisNvdz3qNnuPygQYp9LWfndEbSxam3Ah_RR3MzNY6s9MMetMSR3BEJU8eJ5woUm0aC2Zg?purpose=inline)

01 · THE MATHEMATICAL FOUNDATION

[Amortized Efficiency of List Update and Paging Rules](https://www.cs.cmu.edu/~sleator/papers/Amortized-Efficiency.htm?utm_source=chatgpt.com)

Sleator & Tarjan, 1985.

The classic analysis of move-to-front and recency ordering. It explains why the list-update rule is so effective, although its online competitiveness results do not directly optimize decoding.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.cmu.edu\&sz=32)

cs.cmu.edu



[Burrows-Wheeler Transform Overview | PDF | Computer Science | Algorithms](https://images.openai.com/static-rsc-4/p8HvDWQKuIDM_QZQaYNgdiFnk9FPR1aWo5aEbVooCk-tI9IuLHfLYIdoOX-zfSwOLIJ-VVH1kMGeI3fZMlHNjl0U5YTyhgWnREfutDJdKd-zcjNVjufSlrnflgvOyYbZF__1exTYeeSuRU0mF-NmFKBeWMVybNLBTEAoQNtKZ2s?purpose=inline)

02 · AN UNUSUAL INVERSE TRANSFORM

[Burrows–Wheeler post-transformation with effective clustering and interpolative coding](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.2873?utm_source=chatgpt.com)

Niemi, 2020.

Section 2.4 describes a Move-From-Front transform that reconstructs MTF states backwards. It solves a different encoding problem, so it isn't directly applicable to bzip2, but the reverse-state perspective could be illuminating.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://onlinelibrary.wiley.com\&sz=32)

Wiley Online Library



[PPT - Amortized Analysis PowerPoint Presentation, free download - ID:775309](https://images.openai.com/static-rsc-4/9p75AMzz1YAnxxB2yESGgdWXPkZ-3w6mBz5CSSwFvnjqQzccWsB0Z-SoKn6JwBebhN4zw8Yyx8dsis_TEuOdHzBbXC1BbU93HytmTocs-fFwWAhdFnb6qITTtX79Drwmbjd40Rc9RpBS8PP3eRPT0dl4Muyx8895F0j6nh15u7Q?purpose=inline)

03 · PRACTICAL DATA-STRUCTURE PRECEDENT

[SwiftShader's timestamp-based move-to-front data structure](https://swiftshader.googlesource.com/SwiftShader.git/+/cc5697f9685479191c012cb919c97f5eedd1e041/source/comp/move_to_front.h?utm_source=chatgpt.com)

An implementation using an AVL order-statistic tree, ordered by recency timestamps. It provides selection by rank and moves the selected item to the front.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://swiftshader.googlesource.com\&sz=32)

Git at Google



## Why the recency perspective matters

A conventional inverse-MTF decoder maintains:

\\[ L=[s_0,s_1,\ldots,s\_{\sigma-1}] \\]

and, given rank \\(r\\), emits \\(L[r]\\) and moves that symbol to the front.

But we could instead assign each symbol a timestamp indicating its latest appearance. The MTF list is simply the symbols ordered by decreasing timestamp, with never-seen symbols retaining their original order.

So the decoder becomes:

\\[ \text{select the symbol with recency rank }r, \\]

then update that symbol's timestamp to the current position.

This is exact. It represents the same algorithm, but through a different data structure.

A tree may be far too costly for mibz2's 256-symbol alphabet. We're not proposing an AVL tree as the next decoder.

The intriguing possibility is to exploit the alphabet's small size with bitsets or a compact hierarchical select structure.

For example, if ranks can be maintained through a handful of machine words, the update might consist of a few bit operations rather than moving a prefix of a 256-byte array.

The central obstacle is determining whether maintaining that representation costs less than the highly optimized array updates mibz2 already performs.

And there may be a more profound opportunity: recency timestamps could allow selected chunk operations to be summarized or combined without explicitly materializing their intermediate permutations.

That's the mathematical connection I'd like to investigate.

## What I'd read first

I'd begin with the SwiftShader code, unusually enough.

The key question is whether its timestamp representation exposes an operation we can simplify drastically when the alphabet is limited to 256 symbols.

Then I'd read Niemi's Section 2.4 for the reverse-state viewpoint, and Sleator–Tarjan for the underlying recency structure.

A useful reading session should leave us able to answer:

Can inverse MTF be represented as a succession of order-statistic queries on a small, evolving set—and does that representation admit a cheaper CPU algorithm than moving list prefixes?

If the answer is merely “use a different tree,” we can reject it quickly.

If it suggests a constant-sized, bit-parallel recency representation or a work-reducing chunk algorithm, we've found something worth studying seriously.

That's where I'd go next.
````

````text
# Reading session II: recency, order statistics, and the possibility of a cheaper inverse MTF

I've followed the trail through the original list-update literature, SwiftShader's timestamp-based MTF implementation, and Niemi and Teuhola's 2020 reverse-MTF construction.

The reading has changed my view in one important way:

A timestamp or order-statistic representation is unlikely to be the breakthrough by itself. But it exposes a useful distinction between the information needed to recover the next symbol and the information needed to maintain the complete MTF state.

That distinction deserves our attention.

## 1. SwiftShader: timestamps are an alternative implementation, not less work

The [SwiftShader move-to-front implementation](https://swiftshader.googlesource.com/SwiftShader.git/+/cc5697f9685479191c012cb919c97f5eedd1e041/source/comp/move_to_front.h?utm_source=chatgpt.com) is explicit about its design.

It maintains an AVL order-statistic tree whose nodes are ordered by recency timestamps. The tree stores subtree sizes, making selection by rank possible in logarithmic time. Accessing an element changes its timestamp and moves it to the front.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://swiftshader.googlesource.com\&sz=32)

Git at Google



That's elegant for a dynamic collection of arbitrary size.

But for mibz2's alphabet of at most 256 symbols, it introduces tree traversal, subtree-size maintenance, rotations, and less predictable memory accesses.

The existing MTF list occupies at most 256 bytes. A conventional decoder can select a symbol by direct indexing and update a contiguous prefix.

On a modern CPU, that is an exceptionally difficult baseline for a tree to beat.

The classical [Sleator and Tarjan (1985)](https://www.cs.cmu.edu/~sleator/papers/Amortized-Efficiency.htm?utm_source=chatgpt.com) result establishes the competitive properties of move-to-front as a list-maintenance strategy. It doesn't establish that a timestamp representation can implement inverse MTF more cheaply.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.cmu.edu\&sz=32)

cs.cmu.edu



Conclusion: timestamp trees are useful conceptual prior art, but not a promising direct CPU implementation.

## 2. An exact recency formulation

Suppose the initial MTF list is

\\[ L_0=[a,b,c,d,\ldots]. \\]

After processing a sequence of decoded symbols, the current list is ordered by each symbol's most recent occurrence, followed by symbols never encountered in their original order.

Consequently, inverse MTF at step \\(t\\) can be expressed as:

\\[ x_t=\operatorname{select}\_{r_t} \bigl(\text{symbols ordered by recency before }t\bigr). \\]

Once \\(x_t\\) is identified, it becomes the most recent symbol.

This separates two operations:

- Selection of the \\(r_t\\)-th most recent symbol.
- Updating that symbol's recency.

A tree performs both in logarithmic time. A flat 256-byte list performs selection in constant time and update in time proportional to the selected rank.

So the potential win would be a small-alphabet representation offering both operations at very low constant cost.

The difficulty is that the two operations are dependent: the chosen symbol determines the next ordering.

A packed bitset can accelerate selecting a position from a fixed set, but the MTF order itself changes after every nonzero rank. Maintaining that order is the expensive part.

There is no obvious way to replace the entire update with one `popcnt` or bit-extraction instruction.

## 3. Niemi and Teuhola: reverse MTF really does work—but it requires information bzip2 doesn't store

I found the full open-access article:

[Burrows–Wheeler post-transformation with effective clustering and interpolative coding (2020)](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.2873?utm_source=chatgpt.com)

Section 2.4 introduces the Move-From-Front (MFF) transform.

Instead of reading an MTF rank and moving the selected symbol to the front, the decoder starts with the final MTF list and reverses the operations.

At each step, it takes the front symbol and moves that symbol down to its previous rank.

The paper uses this property to recover a sequence that has been stably sorted by character during compression.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://onlinelibrary.wiley.com\&sz=32)

Wiley Online Library



The construction is mathematically sound, but its format stores additional information, notably the final MTF list and character frequencies.

Standard bzip2 doesn't supply that final list.

We could compute it from the rank stream, but that would introduce another transformation pass before reversing the sequence.

So this is not directly applicable to existing `.bz2` files.

Nevertheless, it demonstrates that the direction in which a sequence of MTF operations is evaluated can change the structure of the reconstruction problem.

That observation is worth remembering, even though this particular decoder uses a different compressed format.

## 4. The more interesting result: an exact symbolic decomposition

Let's examine a sequence of MTF ranks independently of the actual input alphabet.

Take:

\\[ R=[2,0,1,2]. \\]

Start with symbolic identities:

\\[ L=[0,1,2,3,4,\ldots]. \\]

The inverse-MTF operations are:

| Rank | Symbol emitted | New front of list |
| ---- | -------------- | ----------------- |
| 2    | 2              | 2                 |
| 0    | 2              | 2                 |
| 1    | 0              | 0                 |
| 2    | 1              | 1                 |

So the symbolic output is:

\\[ O=[2,2,0,1], \\]

and the final symbolic state begins:

\\[ L\_{\mathrm{out}}=[1,0,2,3,4,\ldots]. \\]

This gives a chunk two distinct mathematical products:

\\[ \boxed{\text{chunk}=(O,P)} \\]

where \\(O\\) specifies which symbols from the incoming list are emitted, and \\(P\\) specifies how the list is permuted.

There's a useful relationship:

Every symbol in the chunk's final moved-to-front prefix must have appeared in its output.

Furthermore, the order of those distinct moved symbols is exactly the reverse order of their final occurrences in \\(O\\).

This means we needn't store \\(P\\) as an independent general permutation if we already have the complete symbolic output sequence.

We can derive the final state from that sequence.

But this exposes a deeper problem: the symbolic output sequence itself has one entry per MTF instruction.

If we have to construct and store all of it, then apply the incoming-state mapping to emit actual bytes, we're adding work and intermediate memory rather than eliminating them.

A compact state summary is therefore insufficient for a work-reducing decoder.

### What would make it economical?

A chunk would need to admit a compact description of both:

1. Its outgoing list transformation.
2. Its emitted symbol sequence.

For example, long stretches of rank zero have an extremely compact description: repeat the current front symbol without changing the state.

But standard bzip2 already exploits this through RLE2, and mibz2's existing decoder processes those runs without performing ordinary list updates for every repeated symbol.

So that familiar opportunity is largely taken.

The remaining question is whether nonzero-rank sequences exhibit similarly useful algebraic structure.

That is the precise point at which further structural analysis could be productive.

## 5. A bit-parallel alternative—and its limitation

There's another representation worth considering, though not implementing yet.

Assign every symbol an integer slot in a much larger ordered universe. Maintain an occupancy bitmap of those slots.

To process rank \\(r\\):

1. Find the \\(r\\)-th occupied slot.
2. Read its symbol.
3. Clear that slot.
4. Insert the symbol into a new slot ahead of all existing symbols.

A hierarchical bitmap with population counts supports selection and updates in logarithmic time relative to the slot universe.

This trades list-prefix movement for bit operations and rank/select queries.

However, the slots must be recycled or renumbered eventually, and there are two occupancy updates for every nonzero rank.

A 256-byte array is extraordinarily compact compared with a large dynamic slot universe.

So this approach may have lower asymptotic update complexity, but almost certainly higher overhead on common small MTF ranks.

Its most plausible use would be as a slow path for exceptionally large ranks, not as the main algorithm.

That still wouldn't constitute a fundamental breakthrough unless the large-rank workload were unexpectedly costly.

## 6. A more promising hybrid emerges

I think we can derive a useful candidate from the preceding results.

Rather than replacing the whole MTF list, split it into two logical regions:

Hot-prefix / cold-suffix representation

If most nonzero ranks fall within the first 16 entries, the hot prefix could live in one SIMD register. Common updates would be a shuffle, while exceptional accesses would use a conventional array.

The difficulty is maintaining the exact invariant when a symbol is promoted from the cold suffix: one item must cross the hot/cold boundary, and the cold state must remain correctly ordered.

This is an implementation hypothesis rather than a mathematical breakthrough.

More importantly, mibz2's #25 experiment already investigated a register-resident MTF head and rejected it under its measured conditions.

So we must not reopen that design merely because its description now uses the words hot and cold.

It would need some genuinely different mechanism, such as processing several rank instructions together for less total work.

Otherwise this remains closed.

## 7. The actual breakthrough condition revealed by this reading

I think we can now state the reverse-MTF question much more sharply.

For a rank sequence \\(R\\), let \\(T_R\\) denote its final-state transformation and \\(O_R\\) its emitted-symbol sequence.

Can we find a representation \\(S(R)\\) such that:

\\[ S(R)\longrightarrow (T_R,O_R) \\]

and both constructing and applying \\(S(R)\\) require substantially less CPU work than interpreting the rank instructions sequentially?

For the ordinary MTF state, the answer is not obvious.

The final transformation \\(T_R\\) can be highly compressible.

But \\(O_R\\) still carries a potentially complicated sequence of symbol identities.

A useful breakthrough might exploit repeated patterns in the rank instructions to compute both products together, perhaps through small tabulated programs or composition of frequently occurring rank subsequences.

This is where I would move the reading next: CPU-oriented tabulation, small-state transducers and batched interpretation of short instruction sequences.

Not because those techniques necessarily win, but because they directly address the problem that defeated the earlier parallel schemes: they processed the instructions once to derive the state and then again to produce the output.

If we cannot avoid that duplicated work, the algebra has limited value for our four-core objective.

## What we've established today

The SwiftShader implementation shows that recency-based order statistics are practical, but with machinery inappropriate for a tiny alphabet.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://swiftshader.googlesource.com\&sz=32)

Git at Google



The Niemi–Teuhola paper shows how MTF operations can be reversed, but its compressed format contains information unavailable in ordinary bzip2.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://onlinelibrary.wiley.com\&sz=32)

Wiley Online Library



And the algebra confirms that a chunk's outgoing permutation and its symbolic output are related, but that relationship does not automatically save enough work.

No breakthrough demonstrated yet.

What we have accomplished is narrower and useful: we've eliminated another tempting category of superficial optimization, while identifying the kind of representation that would actually change the computational cost.

The next truly interesting reading shelf is therefore the theory of small-state transducers, Four-Russians tabulation, and composable sequence transformations—particularly methods that produce outputs and update states in a single inexpensive operation.

That's where I'd take the search next, while Claude completes the broader portfolio audit.
````
