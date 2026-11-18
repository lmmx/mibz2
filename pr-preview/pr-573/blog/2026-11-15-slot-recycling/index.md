[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-573/blog/index.md>)

# Reduce, reuse, recycle: hand-me-down slots for every fragment of text

Each new text fragment takes over the slot its finished neighbour just vacated, so mibz2 touches 30% fewer slots and needs up to 15% less memory.

15 November 2026 · 13 min read · [Research result](<#7-what-the-benchmarks-tell-us>)

One of mibz2's ways of rebuilding a block of bzip2 text works on fragments of it: pieces of the output, each standing for a stretch of the block. It keeps the bookkeeping for every fragment in a small record called a slot, 20 bytes holding the fragment's length, where its text is kept, the slot before it, and its partner slot in a second arrangement of the same fragments. When one fragment is finished, its slot is free just as the next fragment needs one, and mibz2 now hands it over instead of taking fresh memory.

## The idea in a nutshell

**Slot recycling makes mibz2 faster by recognising that, whenever its [Cells algorithm](<https://lmmx.github.io/mibz2/pr-preview/pr-573/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) needs somewhere to put a new piece of information, another piece of information has just finished using its space.**

Previously, the algorithm reserved a fresh memory slot for the new piece, leaving the old slot behind. Now, it reuses the old slot immediately.

This is a deceptively simple improvement. The algorithm still performs the same mathematical operations and reconstructs exactly the same bytes. What changes is how much memory it needs to touch while doing so.

On the Wikidata benchmark, slot recycling:

- **Eliminates 29.7% of the slots previously touched** during Cells induction.
- **Reduces peak decoder memory usage by 10–15%.**
- **Makes four-thread decoding 1.7% faster** in the initial paired CI benchmark, without a statistically established speed change at one or two threads.

The important insight is that **we aren't making the mathematics smaller; we're making its physical representation more efficient**. We're taking advantage of something already guaranteed by the mathematics: one slot's useful lifetime ends just as another begins.

And because mibz2's performance is sensitive to memory contention, eliminating unnecessary memory accesses can matter more than shaving a few arithmetic instructions.

## 1\. Why memory matters so much to mibz2

To understand why this is worthwhile, we need to understand what mibz2 spends much of its time doing.

Bzip2 compression uses the Burrows–Wheeler transform (BWT), which rearranges a block's bytes into an order that's easier to compress. Decompression must reverse that rearrangement and recover the original text.

The conventional way to reverse the BWT is to construct a permutation describing where each position leads, then follow that permutation to reconstruct the original sequence.

This is computationally awkward for modern processors.

Each step tells the processor where to look next. The address of the following memory access often cannot be known until the current access completes. If the relevant data isn't in cache, the processor may spend much of its time waiting for memory.

mibz2's original approach was to interleave several independent traversals, allowing the processor to have multiple memory operations in flight.

But the Cells algorithm takes a more radical approach.

**Instead of reconstructing the text by following the permutation one position at a time, Cells exploits the structure of the permutation itself.**

The inverse-BWT permutation has a special organisation: it acts as a translation over intervals corresponding to runs of equal bytes. That allows it to be treated as a finite interval-exchange system.

Using a form of Rauzy–Veech induction, Cells repeatedly simplifies this system. Each simplification combines pieces of the permutation and builds a compact description of the text they represent.

Eventually, the algorithm has enough information to reconstruct the original block.

This can require dramatically fewer algorithmic steps than following every position in the original permutation.

However, those steps have their own cost: Cells must maintain a changing representation of the intervals, their relationships and the words they represent.

That representation is where slot recycling operates.

## 2\. What is a slot?

Imagine that Cells is manipulating a collection of pieces, which we'll call *letters*. Each letter represents an interval together with a word: the sequence of output bytes associated with it.

The induction maintains two orderings of these letters, corresponding to the two sides of the interval exchange.

We can think of them as:

- The **top order**, describing one arrangement of the pieces.
- The **bottom order**, describing the corresponding arrangement on the other side.

These aren't two complete copies of the decompressed text. They're two linked arrangements of the same mathematical pieces.

Each letter has a slot in each order.

Here are both orders for a handful of letters, with each letter's two slots joined:

Two linked orders of the same letters

Conceptual: five slots of each order. Each letter has a slot in both orders, in a different arrangement, so the partner links cross. The induction reads each order from its front, where A and B are, and follows the links to the slots before them.

A slot is a small record in memory. In the implementation shipped before this change, it occupies **20 bytes**, containing the information needed to navigate and manipulate the induction:

| Information | Purpose |
| --- | --- |
| Previous-slot link | Finds the next piece when advancing through the order |
| Interval length | Describes how many positions the piece covers |
| Word reference and length | Identifies the text represented by the piece |
| Partner-slot reference | Connects its position in one order to the corresponding position in the other |

The slot's five fields, as the code lays them out:

A slot is 20 bytes: five 4-byte fields

The layout as `src/cells.rs` documents it, unchanged by \#408. Every field is a 4-byte offset or length. The table above lists the word’s length and offset as one row, since the induction reads them together. Below, to scale: 64 bytes hold three whole slots and part of a fourth.

The algorithm walks through these two orders, repeatedly taking pieces from their fronts.

As it goes, it transforms letters into *cells*: finished pieces whose words can be represented by literal bytes or by references to previously constructed words.

The slot records are the bookkeeping that makes this transformation possible.

And because there can be many millions of cells, even seemingly small inefficiencies in this bookkeeping accumulate.

## 3\. How an induction step works

At each step, Cells considers two pieces, A and B, one encountered through each order.

Suppose their remaining interval lengths are:

- A has length **7**.
- B has length **3**.

The algorithm takes the smaller length, 3, and constructs a cell representing the corresponding portions of A and B. Its word is constructed by concatenating their associated words.

Now B has been exhausted, while A still has 4 positions remaining.

Conceptually:

One unequal step: B ends, A goes on

Conceptual: the example in the text. Each square is one position. The green squares become the cell; A’s other four stay A’s. B ends, so its slot in the bottom order is free from this step on.

This is a simplification of the mathematical operation, but it captures the essential event.

One letter continues. The other finishes.

If A and B had equal remaining lengths, both would finish together. The interesting case for slot recycling is when their lengths differ, because the surviving letter will eventually produce another cell.

**That next cell needs a place in the other ordering.**

Previously, the algorithm reserved a fresh slot for it.

But notice what has just happened: the other letter, B, has finished. Its position at the front of that ordering is no longer needed.

This is where the opportunity appears.

## 4\. The old approach: make room for a future cell

The original implementation was designed to avoid complicated rearrangements of the linked orders.

When a letter continued after an unequal-length step, Cells *promised* it a slot for its next cell. That slot came from additional space reserved beyond the original letters' slots.

This was straightforward and efficient in an important sense: it avoided having to move existing letters or reorganise the whole structure.

But every promise of a fresh slot extended the set of memory locations the induction would eventually touch.

Across millions of steps, many of these fresh locations occupied cache lines that had not previously been used.

Touching a new cache line may require the processor to bring it into cache, obtain write ownership and eventually evict or write back dirty data. These costs become particularly significant when several workers are competing for limited cache capacity.

Meanwhile, some original slots were already dead.

The algorithm had information about their former contents in registers, yet their memory locations were being left unused.

We had **space becoming available at precisely the moment we were reserving more space elsewhere**.

## 5\. The new approach: recycle the slot that just died

Slot recycling changes how that promised location is chosen.

Returning to our example, B has just finished and A continues.

Previously, the next cell cut from A would receive a fresh slot in the bottom order.

Now it receives **B's former front slot in that same order**.

Here is the bottom order after B ends, before and after the change:

The bottom order after B ends: where A’s next cell goes

Conceptual, after the step in the last figure. Slots are in memory order; the front is the slot the induction reads next, and it moves left as the induction goes on. The promised slot is linked in as the slot before A’s cell, so A’s next cell will be read there. The cell is also written to B’s slot in the top order (not drawn). When A ends instead, the same happens in the top order, with A’s front slot.

This isn't an ordinary memory allocator discovering that a block of memory happens to be free. The algorithm knows, from the structure of its induction, that this specific slot has become available.

The implementation has three crucial facts in its favour.

**First, the finished letter cannot be encountered again through that old front slot.** It was the last remaining piece at the front of its order, and no surviving link leads back to the slot as that letter.

**Second, its corresponding position in the other order has already been replaced by the newly constructed cell.** The pairing and links have been updated to refer to the new structure.

**Third, everything the current operation still needs from the finished letter has already been read into registers.** The old slot's memory contents are no longer required.

Together, these facts establish that the slot can safely take on a new purpose.

Instead of advancing a cursor into fresh storage, the algorithm remembers the location of the slot it just consumed.

In the Rust implementation, two offsets, `fa` and `fb`, track the front slots in the respective orders. When one letter finishes and the other continues, the appropriate front offset becomes the promised location.

The surrounding induction remains essentially unchanged. It creates the same cells, constructs the same words and ultimately produces the same decompressed output.

The difference is that its slots are **reused rather than continually extended**.

That is why the name *slot recycling* is so precise.

## 6\. How much does this save?

The builder, the agent that implemented \#408, [counted](<https://github.com/lmmx/mibz2/pull/408>) what happened across the 196 Wikidata blocks admitted by Cells in the benchmark corpus.

The result was substantial:

| Quantity | Before | After |
| --- | ---: | ---: |
| Slots touched | 31,547,362 | 22,181,734 |
| Fresh slots required | 9,365,628 | 0 |
| Reduction in touched slots | — | **29.7%** |

The same counts, drawn to scale:

Slots the induction touches, over the 196 Wikidata blocks

Counted, not timed: \#408’s census of the benchmark’s Wikidata corpus, the 196 of its 431 blocks that Cells admits. The letters’ slots are two per letter, one in each order. Every unequal cell used to take one fresh slot.

Each avoided slot is 20 bytes.

Across those blocks, this corresponds to approximately **187 million bytes of slot storage no longer touched**.

That doesn't mean precisely 187 million fewer bytes travel to DRAM. Cache lines, write allocation, reuse and eviction behaviour determine actual memory traffic.

But it does mean that millions of distinct slot locations are no longer needed.

The allocation can shrink too. The previous implementation reserved room for additional slots based on its cell budget. The new one needs only enough room for the original letters' slots in each order.

And this matters beyond the amount of memory allocated.

### Why fewer slots can mean faster decoding

Modern CPUs don't fetch arbitrary individual bytes directly from main memory for ordinary cached accesses. They work with cache lines, commonly 64 bytes.

A 20-byte slot is small, but enough slots spread over millions of addresses occupy a substantial working set.

When several workers decode different blocks simultaneously, each has its own induction state competing for space in the shared cache.

If their combined working sets are too large, useful data gets displaced, and later accesses may have to retrieve it from slower levels of the memory hierarchy.

Slot recycling attacks that problem directly.

The slot being recycled has only just been read, so its cache line may already be resident. The fresh location it replaces might otherwise require a first touch of a different line.

The immediate benefit isn't fewer mathematical operations. It's **less memory to keep resident and fewer distinct places to access**.

This explains why the change is especially interesting for parallel decoding.

## 7\. What the benchmarks tell us

The initial paired CI benchmark measured the complete decoder, rather than just timing the induction function in isolation.

For Wikidata:

| Threads | Wall-time change | Peak memory change |
| --- | ---: | ---: |
| 1 | \+0.5% | −9.7% |
| 2 | −0.2% | −12.5% |
| 4 | **−1.7%** | **−14.6%** |

The same run with its intervals, and the memory in megabytes:

The decoder on Wikidata, with and without slot recycling

Measured: \#408’s paired CI bench, 10 rounds on one runner (Intel Xeon Platinum 8573C, 4 CPUs on 2 cores), the whole decoder on the Wikidata corpus. Left: the change in median wall time with its 95% interval across the rounds; green where the interval excludes no change. Right: peak resident memory, each a reduction the bench reports as beyond noise. The `records` corpus was 3.4% slower at 4 threads in the same run.

The four-thread wall-time improvement had a reported 95% confidence interval of −3.3% to −0.1%.

That is a modest but positive result on this runner. The one- and two-thread changes were indistinguishable from noise.

The memory reductions are much larger.

This pattern is consistent with the intended mechanism: reducing the working set matters more as concurrency increases.

But we should be precise about what has been established. The CI runner isn't the i9 workstation. It has a different cache hierarchy and different core-sharing conditions. The benchmark demonstrates a benefit on the tested machine, but it doesn't yet establish how much slot recycling helps the i9's known [concurrency plateau](<https://lmmx.github.io/mibz2/pr-preview/pr-573/blog/2026-11-07-why-mibz2-stops-scaling/index.md>).

[A later paired run on the i9](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6894519106>), a 10-core desktop, shows the gain growing with the number of Cells workers: no change at one or two threads, 4% less CPU at four, and 5% faster wall time at five (interval 4.7% to 6.1%). Peak memory was lower at every thread count from one to five, and nothing changed at six or more threads, where mibz2 uses its team path instead of Cells.

There's also a secondary-workload caveat: the `records` corpus showed a four-thread regression in the same CI run. Correctness was preserved, but the change is not a universal speedup.

The valuable thing is that **the memory reduction is substantial without requiring additional expensive computation**. The extra bookkeeping is minimal, with the front positions already available to the induction.

## 8\. Why this is different from field elision

The terminology initially caused some confusion because two related research ideas both used the word *elision*.

[MONODROMY](<https://github.com/lmmx/mibz2/pull/410>), the team's mathematics agent, investigated whether individual fields inside existing slots could be left unwritten or reconstructed from other information.

It found that approximately 25.5% of the induction's stored bytes were never subsequently read, and that certain mathematical identities could account for an even larger fraction.

That sounds promising, but almost every redundant field shares its 64-byte cache line with live information.

If you stop writing a redundant field while leaving the surrounding record layout unchanged, the processor may still need the same cache line.

So removing those field writes doesn't necessarily make the working set smaller.

**Slot recycling solves a different problem.**

It doesn't selectively remove fields from slots. It removes the need for whole additional slots.

That means entire ranges of memory no longer need to be touched, and the required working set can genuinely shrink.

This distinction is central:

- **Field elision:** store less information in the same locations.
- **Slot recycling:** use fewer locations to store the necessary information.
- **Equal-cell elision:** avoid constructing some cells altogether, replacing those operations with another reconstruction mechanism.

The third idea remains a separate mathematical opportunity. Slot recycling hasn't implemented it or disproved it.

## 9\. Why the change is safe

Reusing memory in an algorithm implemented with unchecked offsets and `unsafe` Rust deserves particular care.

The crucial correctness obligation is not merely that a slot *looks* unused. It must be guaranteed that no later operation still needs its old contents.

The proof comes from the induction's ordering invariant: the consumed front slot has no surviving references as the old letter, and its necessary values have already been captured.

The implementation also narrows the slot regions to the exact number of original letters. Tests check that promised slots stay inside those regions, including cases where there is no spare room beyond them.

The builder compared the new induction against the original on exhaustive small inputs, accounting for the fact that physical slot offsets legitimately differ. The resulting words, rules, state and output agree.

The broader decoder tests, CRC checks and Miri validation passed before the change was merged.

This is important because the optimisation changes where state lives, **not what that state means**.

The resulting decoder remains subject to the same correctness requirements as before.

## 10\. What this teaches us about improving mibz2

Slot recycling is a good example of why sophisticated algorithms still benefit from careful attention to mundane-looking implementation details.

The Cells algorithm is built on substantial mathematics: interval exchanges, Rauzy–Veech induction, permutation structure and compact representations of words.

That mathematics makes it possible to avoid much of the work required by conventional inverse-BWT traversal.

But the mathematical abstraction also introduces a physical representation: linked orders, slots, words and rules.

The performance of the resulting program depends on both levels.

An algorithm can have excellent asymptotic properties and still spend too much time managing its intermediate state.

Conversely, improving that state management can produce worthwhile gains without changing the underlying algorithm at all.

The particularly elegant aspect of slot recycling is that it derives the optimisation from a structural property of Cells itself.

**At every unequal induction step, the continued letter needs a future slot just as the exhausted letter releases one.**

Recognising that relationship removes a whole category of unnecessary memory use.

The successful change therefore sits at the intersection of mathematics and systems programming: a proof about object lifetimes becomes a more efficient memory layout, which becomes a measurable improvement in the complete decoder.

It also points towards the next questions.

MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could mean up to 25% fewer induction lines (an estimate made before recycling), although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier [failed 16-byte layout](<https://github.com/lmmx/mibz2/pull/398>) is a warning that compactness isn't automatically speed.

Meanwhile, measuring how memory traffic scales after slot recycling can help determine whether Cells is becoming competitive at higher thread counts.

Those are separate experiments. Slot recycling has already justified itself on its own evidence.

## The essential insight

**Slot recycling makes mibz2 better by exploiting the fact that the useful lifetime of one memory record ends exactly when another needs its space.**

It doesn't eliminate the induction steps. It doesn't change the text reconstruction. It doesn't require a more elaborate parallel algorithm.

It simply stops the program from touching memory it never needed to touch.

That's why a change that is small in code can remove nearly a third of the induction's touched slots — and why understanding the mathematical structure of an algorithm can yield practical improvements well beyond the mathematics itself.

---

*Implementation and evidence: [mibz2 PR \#408](<https://github.com/lmmx/mibz2/pull/408>), [merged on 15 November](<https://github.com/lmmx/mibz2/commit/381fbf1d5f76a7f1689e7546d254b7e2bfd664b6>). For the distinction between slot recycling and field elision, see [MONODROMY's analysis, PR \#410](<https://github.com/lmmx/mibz2/pull/410>).*
