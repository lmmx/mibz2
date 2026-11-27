[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/index.md>)

# What else moves together: the search for a decoder below the proved floor

The next decoder for bzip2's block sort will come from rows that move together in a new sense, or from inversion steps that yield more information.

24 November 2026 · 11 min read · [Meta](<#9-what-comes-next>)

[Our last post](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/index.md>) set a target that no decoder in our current family can meet: at most half an event per width-1 row. This post is about where a decoder that meets it will come from. Rauzy–Veech induction, the step that took us beyond the walk, was found by noticing that a whole stretch of rows takes the same step, so that the step can be composed once and shared. Our proved floor says only that width-1 rows have nothing to move together with *in that sense*. The search is therefore for other senses in which they do, pursued all at once, and, beside it, for ways to make the steps we already take yield more information than they do now.

From one step per row, to one step per run, to the open question

Schematic, on a column of nine rows. The walk follows the block's cycle one dependent step at a time. Rauzy–Veech induction lets each run of equal letters share one step, but a width-1 row (b, x) has no neighbour to share with, and every current decoder pays at least one step for each: the proved floor. The search is for a sense in which those rows do move together, and for steps that yield more than where their own row goes.

How a candidate is judged is a separate matter, and this post settles it too. The target as first written turned out to be decided by a bookkeeping convention rather than by the data, so we have restated it in real machine cost on a declared model, where the floor has to be transferred by a proof of its own. Those rules referee a candidate once it exists; they do not produce one. The first candidate has already been through them: it shortened the chain of memory reads that wait on one another by 50 to 75 times on our real blocks, a real find, but it did not reduce the number of unpredictable reads, which is where the floor lies.

## 1\. Where the last post left us

The last post's terms are used here without being explained again:

- **The target:** at most h₁/2 events in all, in two passes over the block, with no more memory than Open, and faster than Open on whole decoding time ([§7](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/#7-where-the-search-stands>)).
- **The floor:** G1 proves that every decoder in the current family pays at least one event per width-1 row, so anything meeting the target is new by theorem ([§3](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/#3-three-routes-back-from-the-bwt>)).
- **The way out it left:** a rule applied to a whole scattered set of rows, short to describe and cheap to apply, in the form Build, Advance, Emit ([§6](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/#6-what-a-new-route-would-have-to-be>)).
- **The method:** the [Mario](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level.md#2-the-ssl-coq-material>) move, which supplies the payoff first and looks for the mechanism second, and the supplied test that came of it ([§2](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/#2-four-reframings-of-the-search>)).

## 2\. What moves together

Rauzy–Veech induction came from an observation about the data, not from an accounting of cost: a stretch of rows holding the same byte takes the same step. Comparing the two algorithms, the walk and the induction, and asking what the two have in common, which is the question the last post took from [Hilbert's 24th problem](<https://en.wikipedia.org/wiki/Hilbert%27s_twenty-fourth_problem>), gives an answer in two parts, which together we call *contraction along composition*:

1. **a notion of "these move together",** which for Rauzy–Veech is adjacent rows holding the same byte;
2. **a rule for composing what moves together,** and then recursing on what is left.

G1 says that width-1 rows have nothing to move together with in that first sense. It does not say that they have nothing to move together with in any sense, and the distinction decides where to look.

Of the arches, and of walks in disguise

Contracting harder in the same sense is the failure to avoid, and our most recent rounds of candidates show what it looks like. The best of them, "arches" cut at the low points of the map and swept twice, reached 0.366 of an event per width-1 row, but only on synthetic random hex and only with a third pass. On our six real blocks the same construction cost 3.18 to 9.03 per width-1 row, and a single sweep we already had gave 0.412 on the same hex. Other builds that looked free walked the map row by row inside them, one read the true decoded text, and the arches themselves turned out to be a known suffix-array technique, SA-IS ([the review](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level/neon-cycle5-critique.md>)). Each squeezed the same notion of moving together further, and paid for it with an extra pass or a hidden walk. Contracting the left-over cycle again does not escape either: that recursion is list ranking by contraction, a known technique for linked lists, and each row it removes still costs that row's own event, so the depth falls but the count does not ([the record](<https://github.com/lmmx/mibz2/blob/299a0624baadacfa079d1bf4d64939b326293b1a/docs/journal/monodromy/2026-11-24-supported-rows.md>); see also [our synthesis](<https://github.com/lmmx/mibz2/blob/cdeef65aba3b5ebad758cf4f5d169f0534bf80ab/docs/journal/monodromy/2026-11-18-synthesis.md>)).

The way forward is to change what moving together means. The phrase is deliberately loaded with several meanings at once: no one of them leads, all are pursued side by side, and finding further ones is part of the work ([the principle](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md>)). Three are named so far:

- **Contract along the text, not along the rows.** A repeated stretch of text is a run in the other direction, and copy plans are the start of it.
- **Move together up to a consistent relabelling,** rather than by exact equality.
- **Find a recurrence among the induction's own steps,** as the Pell rule in the last post ([§2](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-24-halving-the-single-row-steps/#2-four-reframings-of-the-search>)) generates new triangles where an oracle could only hand them over.

## 3\. Making each step yield more

The second principle, *enrichment along composition*, comes from another of Louis's projects, [page-dewarp](<https://github.com/lmmx/page-dewarp>). Its large speedups came from rewriting its projection so that automatic differentiation could supply gradients, which let it switch from a derivative-free optimiser to a better algorithm. Forward-mode differentiation lifts a computation into a richer algebra of pairs, with (a, a′)(b, b′) = (ab, a′b + ab′), so that extra information rides along with the ordinary result ([the account](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#37-enrichment-along-composition-the-second-principle>)).

Where contraction does less work by recognising that computations are the same, enrichment makes work already done yield more, so that later work becomes unnecessary. Here it asks what one LF step, or one step over a run, reveals for free beyond where its own row goes, and whether something computed in an earlier stage, the Huffman or move-to-front decoding, could give the later ones what they need. One test keeps it honest: the extra quantity must be cheap to acquire and must not restate the inversion. Each row's position in the text is the answer itself, and a fingerprint of the column can separate every pair of inputs and still decode nothing.

## 4\. The first candidate, measured

The first worked notes already sharpen the search ([record 113](<https://github.com/lmmx/mibz2/blob/806aa0123c9a24ad7a1e9ef15da015e1408022f0/docs/journal/monodromy/2026-11-24-moving-together.md>), [record 114](<https://github.com/lmmx/mibz2/blob/bfdbc245fce475fbb7d017501a1e7423dd20eaf5/docs/journal/monodromy/2026-11-24-more-senses.md>)). Every sense of moving together puts the rows in some order, and it helps exactly as far as that order agrees with the order of the text, so the search becomes one for a cheap ordering of rows that correlates with text position.

The first candidate worked up in full, *landmark induction*, starts a chain at every row holding a chosen common byte. Those rows sit together in one known block and cost nothing to find. It advances all the chains at once to the next landmark, and repeats on what remains. It leaves the number of unpredictable reads where it was, but makes those reads independent of one another, so the chain of reads that must each wait for the last was estimated to fall from about one per byte to a few hundred on hex-dense content.

Measured on our six real blocks, that chain fell to 1.3–1.9% of the block's length, some 10,000 to 17,000 dependent steps: a cut of roughly 50 to 75 times, but far short of the estimate. On JSON the most frequent landmark strings stay frequent as they lengthen (`"`, then `,"`, then `","`), so the final walk dominates. The unpredictable reads rose by 5 to 20%, and width-1 rows turned out to be under-represented among the landmark rows ([the measurement](<https://github.com/lmmx/mibz2/blob/1799a29e7fcc2a63e06f8849bf939b318624fef4/docs/journal/2026-11-24-algorithm-level/m21/m21-results.md>)). Under the referee's ruling (§6), a breakthrough is a cut in the unpredictable reads, and a large cut in the waiting chain is a real find that is measured rather than claimed ([the ruling](<https://github.com/lmmx/mibz2/blob/3c19c9a2fb4e7831d1bf7a1d0e554c1bc6f8c6f2/channel/coordinator-to-neon.md>)). Landmark induction is therefore a real but modest find, and a reminder that estimates which assume random text can be badly wrong on real data.

## 5\. Why the target had to be restated

The rest of this post is the referee. Its first task was to repair the target, whose decisive gaps were in its accounting. Every known route that beats the floor does so with a *build pass*: a sweep over all the block's [run records](<https://lmmx.github.io/mibz2/pr-preview/pr-1061/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) that composes facts in bulk, as squaring the map or batching the frontier does. The target never said whether such a pass is charged, and [our record of the question](<https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md>) found that each reading settles it without looking at a single block:

- **If every record a pass reads is charged,** every exact decoder pays for nearly every width-1 row, since each such row's byte is recorded nowhere else. The answer is no, by definition, and it tells us nothing.
- **If a sweep is free and only row-by-row steps are charged,** the number of passes decides it. With passes unlimited, repeated doubling of the map brings the count below h₁/2 with nothing new in it. With the two passes the target allows, the best known decoder, squaring the map on Open's retained rows, still pays at least 0.60 of h₁, and misses.

The frozen target's verdict was therefore fixed by two conventions, the charge on a pass and the pass budget, and not by any fact about the column. A target that a choice of convention can satisfy or defeat cannot tell a new algorithm from a relabelled old one, which was the whole reason for setting it.

## 6\. Two yardsticks, kept apart

An [outside review](<https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md>) of the problem, one of three [compared side by side](<https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md>), made the decisive distinction. G1 is proved in bookkeeping units: it counts charges in a calculus of facts, not instructions, memory traffic or time. Two yardsticks are therefore in play, and they must not be allowed to blur into one:

|  | Bookkeeping (version 1) | Real machine cost (version 2) |
| --- | --- | --- |
| **What it counts** | events: charges per row in the calculus of rigid facts | unpredictable memory reads, the longest chain of them, passes, and peak memory |
| **What is proved in it** | G1: at least h₁ for the current family | the bridge, for most of the family (§7) |
| **The target** | at most h₁/2 events, as frozen | fewer unpredictable reads than one per width-1 row, within a fixed budget |

The original target stays exactly as frozen, as version 1. The machine-cost target is a new version and is labelled as one, because changing the yardstick without saying so can look like solving the problem. It judges a candidate by what it makes the hardware do ([the decision](<https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md>)).

The model separates the two kinds of memory access that matter. A *streamed* access moves a cursor one step along memory, which the hardware can anticipate. An *addressed* access goes to an address computed from data, which it cannot. Every run reports four figures, all of them, always ([the formal statement](<https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md>)):

- **Q**, the number of addressed accesses, in which the floor is to be beaten;
- **D**, the longest chain of addressed accesses in which each address depends on what the one before it returned, which is what makes the walk slow;
- **P**, the passes: bytes streamed, divided by the block's length;
- **S**, the peak working memory.

The budget fixes the passes at two and the number of live cursors at one per letter of the alphabet, plus a constant, which is enough to distribute rows by their letter. Holding the budget fixed is what makes relocation visible: a fall in Q bought with an extra pass, or with more memory, shows up in the other coordinates instead of disappearing from the count.

**Counting bytes** would have been the obvious yardstick, and the wrong one, for a decoder must write every byte of its output; whence a target stated in bytes written is impossible for a reason of no interest whatever: the output alone costs n, and n is never less than h₁. The review pointed out the trap, and the adopted yardstick counts accesses instead.

## 7\. The bridge

G1 alone proves no floor of h₁ in the machine yardstick. It bounds a sum of charges in the calculus, so it says nothing directly about Q. To claim that a candidate goes below the old family's floor in machine cost, we need a second theorem: that every decoder in the old family, executed on the declared machine, pays at least a fixed multiple of its bookkeeping charge in Q. Without it, "below the floor" would compare a machine count with a calculus bound, which is not a theorem at all. The yardstick also has to be fixed before any candidate is judged, or a convenient one can be chosen after the fact.

The bridge has now been attempted ([record 112](<https://github.com/lmmx/mibz2/blob/ed6a3e9668587b993ab6a7f4732b043da03937e5/docs/journal/monodromy/2026-11-24-spec-pi-and-bridge.md>)). It holds with a constant of 1 for the walk, for Open and for lanes, and with one half for a single doubling sweep. It fails as a universal constant for one family, in-place sweep composition, where the floor becomes an input-defined count rather than a fixed multiple of the bookkeeping charge. That failure is not a counterexample to G1; it names the first candidate escape. The family is built entirely from old objects, though, so it would escape by how its work is laid out rather than by a new sense of moving together.

Two more terms needed the same treatment:

- **A pass** had no definition. There was no rule for a partial scan, for two scans merged into one, or for a scan that reads values it updated earlier in the same scan. The cursor model now supplies one: P counts bytes streamed through cursors, whatever the program calls its loops.
- **New** had been defined by what an object is not ("not rows, not intervals"), which renaming can satisfy. It now means new *relative to the method being escaped*, with a measurable gap in cost. Rauzy–Veech induction shows why the relativity matters. Against the walk it was a genuinely new basis: its cost scales with the number of runs, not of rows, and its records come from one pass. Against today's floor it is no escape at all, because G1's family already contains it. A definition that asked our own historical step to beat h₁ would be testing the wrong floor.

## 8\. The question in full

With the rules fixed (the machine model, the yardstick, the inputs and the budget), the question becomes one of existence. Is there a decoder with a new kind of object, a way to build that object from the column without the old pointer-following, and a cost law, such that:

1. it is exactly correct on every input, deterministically;
2. it builds its object only from what it is given, within the budget;
3. its cost, Q above all, is low enough on the random-looking content where the width-1 rows live;
4. and under a model of time fitted to our machine, it beats the current decoder on whole decoding time, judged at one thread and at the default ten separately?

The last condition reflects the [supplied test](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>): handed a free decoding order, the whole decoder took 0.838 of its time on one thread and 0.988 on ten, two results that differ in kind and cannot be averaged into one.

A negative answer is harder than it sounds. It must rule out every allowed decoder, not the ones we have tried, and it must fix the budget, because with unlimited passes sorting alone brings Q to zero. No lower bound of that kind is known for inverting a permutation presented as a BWT; the only negative result on record is the scoped one, G1, in the bookkeeping yardstick.

**The Mario proof,** on a closer reading, is narrower than our last post allowed, and it is not where the payoff-first move came from. That move belongs to a separate effort on the Shifting Sand Land level. The [Rocq repository](<https://github.com/tra38/sm64-wmotr-abc-proof>) proves, for the Wing Mario over the Rainbow level, that a run with no A press never makes Mario fly, and even that theorem still rests on named residual hypotheses; its theorem that the level needs an A press is not yet started. What we borrow from both is the method, a large claim split into premises that are proved one by one, and not the standing of a finished impossibility proof.

## 9\. What comes next

The order of work follows from the two halves of this post. The search is for new senses of moving together and for steps that yield more, all pursued at once, and a candidate is first an idea of that kind, an object with its own rule of composition, before it is a cost. Once one exists, the referee applies: the specification of the machine, the bridge, and the four figures (Q, D, P, S) with building the object included. In the Mario manner, the supplied test then measures what the candidate's object would buy before anyone pays to build it.

Nothing is ruled out for having been tried before. An earlier idea may be taken up again when there is reason to think something in it was missed, but it may not be presented as new. Every proposal therefore starts by searching our records, and says either that it is new or which earlier attempt it revisits, and what that attempt missed.

The review that clarified the yardstick also found errors in our own records, and they are part of the work ahead. It listed 21 places where records contradict one another, have been superseded, or differ in scope. Among them, the draft negative theorem's list of excluded mechanisms omitted one that its own text calls open, and one measurement note compared about forty edges against the number of width-1 rows in an example with two runs, which cannot be right, since a block never has more width-1 rows than runs.

---

**Sources:**

- [The essence of the search](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md>), with the reasoning behind it in [journal §34](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#34-the-essence-restated-change-what-moving-together-means>) and [§§36–37](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#37-enrichment-along-composition-the-second-principle>).
- [Records 113](<https://github.com/lmmx/mibz2/blob/806aa0123c9a24ad7a1e9ef15da015e1408022f0/docs/journal/monodromy/2026-11-24-moving-together.md>) and [114](<https://github.com/lmmx/mibz2/blob/bfdbc245fce475fbb7d017501a1e7423dd20eaf5/docs/journal/monodromy/2026-11-24-more-senses.md>): the scheme, the senses of moving together and landmark induction; [the measurement](<https://github.com/lmmx/mibz2/blob/1799a29e7fcc2a63e06f8849bf939b318624fef4/docs/journal/2026-11-24-algorithm-level/m21/m21-results.md>) on the six blocks; [the ruling](<https://github.com/lmmx/mibz2/blob/3c19c9a2fb4e7831d1bf7a1d0e554c1bc6f8c6f2/channel/coordinator-to-neon.md>) on the two machine figures.
- [The review of the latest candidates](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level/neon-cycle5-critique.md>), and [record 109](<https://github.com/lmmx/mibz2/blob/299a0624baadacfa079d1bf4d64939b326293b1a/docs/journal/monodromy/2026-11-24-supported-rows.md>) on list ranking.
- [The charge convention](<https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md>): why the frozen target's answer is set by the charge on a pass and the pass budget.
- [The problem, stated formally](<https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md>): the machine model, the cost vector (Q, D, P, S) and the conditions for a new basis; [the specification and the bridge](<https://github.com/lmmx/mibz2/blob/ed6a3e9668587b993ab6a7f4732b043da03937e5/docs/journal/monodromy/2026-11-24-spec-pi-and-bridge.md>) (record 112).
- [The outside review](<https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md>), recorded verbatim, including its table of contradictions (§5.2) and its reading of the Mario repository.
- [The comparison of the three answers](<https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md>), with the agreed formulation, and [the decision on the currency](<https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md>).
- [The supplied-codes result](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>).
