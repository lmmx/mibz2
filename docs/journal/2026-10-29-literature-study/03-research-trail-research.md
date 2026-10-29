<!--
Provenance: the research chatbot's answer on the thread connecting the papers still to be
read (RLBWT Tricks, Partitioned Elias–Fano, Gog–Navarro–Petri), given after paper 2 and
relayed by the user into the working session on 29 October 2026, unedited. The same
message continued with deep dive 2, kept as 04-rlbwt-tricks-research.md. Links appeared as
their titles only, followed by the source's site name. The trail dates RLBWT Tricks
"(2024)"; deep dive 2 gives SEA 2022, the conference version.
-->

Yes—and there’s a particularly exciting connection. I think the Langmead paper gives us a thread that runs through three of the papers we already wanted to study, culminating in one that’s probably even more directly applicable to mibz2.

The thread is:

How can we build a compressed representation without first doing expensive work that the structure was supposed to help us avoid?

That sounds simple, but different papers attack different parts of the problem.

The research trail

1 · The construction breakthrough we’ve just read

Optimal-Time Move Structure Construction⁠ — Brown et al. (2026)

The authors maintain forward and inverse interval relationships together, eliminating repeated predecessor searches during balancing.

Lesson: maintaining additional temporary information can reduce construction work.

2 · The practical predecessor

RLBWT Tricks⁠ — Brown, Gagie & Rossi (2024)

This earlier work explores practical representations of the same interval mappings, including relative positions and interval splitting.

Lesson: how we organize intermediate and final interval data can matter as much as the asymptotic algorithm. The later length-capping paper builds directly on these ideas. PubMed Central (PMC)

3 · A different construction breakthrough

Partitioned Elias–Fano Indexes⁠ — Ottaviano & Venturini (2014)

The authors divide sorted integer lists into locally encoded chunks. Crucially, they derive a linear-time algorithm for finding a near-minimum-space variable-length partition.

Lesson: choosing how to divide data into compressed records needn’t require expensive exhaustive optimization. pages.di.unipi.it

4 · The closest match to our actual Ψ problem

Improved and Extended Locating Functionality on Compressed Suffix Arrays⁠ — Gog, Navarro & Petri (2015)

This paper considers compressed representations and navigation involving Ψ, making it especially relevant to our monotone occurrence lists. users.dcc.uchile.cl

Lesson to investigate: how their Ψ representation is assembled, and whether uniform blocks, patched frame-of-reference or partitioned Elias–Fano can be adapted for directly constructing mibz2’s compact navigation groups.

What connects them?

There are really two complementary research ideas emerging.

Idea A: Make construction monotone

The Langmead paper maintains a frontier behind which interval relationships have been settled. The algorithm advances without repeatedly searching through already processed data.

For mibz2, we already know that scanning L produces Ψ destinations in increasing order within each character class.

That suggests a related ambition: construct completed Ψ groups directly, with temporary per-character state, rather than build a full table and reorganize it afterward.

The analogy is promising, though the specific algorithms differ. Langmead’s proof doesn’t automatically transfer to our builder.

Idea B: Make compression decisions locally

The Partitioned Elias–Fano paper gives us another angle.

Suppose we have a sequence of Ψ destinations. Some groups are consecutive, others fit a compact set of offsets, and others need more general encoding.

Rather than compress everything with one scheme, we can choose encodings based on each group’s local structure.

We have already independently arrived at this idea through the 32-row census.

But Ottaviano and Venturini offer something we haven’t examined deeply yet: a formal treatment of how to choose partitions efficiently, including variable-size chunks.

Their problem is not ours exactly. They optimize compressed posting lists, where decoding and locating a chunk involve different costs from permutation pointer chasing.

Nevertheless, it’s a very close algorithmic connection.

And we have a particularly interesting constraint they don’t share: our preference for fixed, directly calculable group addresses.

That could make the optimal partition for mibz2 quite different from the space-optimal partition in their paper.

The most promising connection: Gog, Navarro & Petri

This is the paper I’m most eager to reach.

Why?

Because we’ve spent a great deal of time developing increasingly specialized representations of Ψ, and this paper comes from researchers who have already explored Ψ-related compressed suffix-array engineering.

It gives us a chance to answer a crucial question:

Which parts of our proposed translation/FOR/Elias–Fano architecture are already understood, and which parts become genuinely different when we require extremely fast inverse-BWT traversal?

That includes examining how their structures encode locally consecutive values, retrieve arbitrary Ψ entries, and handle the transitions between symbol intervals.

It also gives us an opportunity to connect the construction theory we’ve just read to an actual Ψ-specific representation.

If the paper provides a concrete construction procedure, we can compare it against our proposed direct-from-L construction. If it doesn’t, that gap itself is worth recording rather than filling it with assumptions.

The route I’d take

I’d continue one paper at a time, in this order:

* RLBWT Tricks — finish understanding the practical move-structure lineage and what Langmead improved.
* Partitioned Elias–Fano Indexes — study the linear-time partition optimization and local encoding choices.
* Improved and Extended Locating Functionality on Compressed Suffix Arrays — connect those ideas to Ψ itself.

The progression is deliberate: first understand how interval structures are organized, then how compressed chunks are chosen, then how Ψ can actually be represented and navigated.

And I think there’s a potentially very fruitful hypothesis running through all three:

The fastest compressed Ψ builder may depend less on inventing a new integer encoding than on arranging the construction so that every occurrence is assigned, grouped and encoded with minimal revisiting.

That is very much in the spirit of both the Langmead construction result and our #81 fusion win.

I’d go to RLBWT Tricks next, with a specific aim: understand the practical decisions behind interval layout and construction, including which later improvements came from changing the algorithm versus changing the representation. Then we can carry those findings into Partitioned Elias–Fano and ultimately the Ψ-specific paper.

That should give us a coherent research lineage rather than a collection of clever but disconnected techniques.

