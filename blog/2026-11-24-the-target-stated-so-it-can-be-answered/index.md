[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Halving the single-row steps, stated so it can be answered

The breakthrough we seek comes from changing what moving together means, and the cost rules we fix here referee a candidate once it exists.

24 November 2026 · 10 min read · [Meta](<#7-what-comes-next>)

[Our last post](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/index.md>) set a binary target: decode every block with at most half an event per width-1 row, which no decoder in our current family can do, so that only a new algorithm could meet it. This post is about how such an algorithm is to be found and how it will be judged once it is, two questions that must not be confused. The finding comes from the step that took us from the walk to Rauzy–Veech induction, contraction along composition: rows that move together have their steps composed once and shared. Our proved floor says only that width-1 rows have nothing to move together with *in that sense*, so the way forward, in Louis's words, is to "find a different sense for something that can occur": to change what moving together means, in every sense we can find at once, and beside it to make the work already done yield more. The judging is the other half. The target as first written turned out to be decided by a bookkeeping convention, so we have restated it in real machine cost on a declared model, with the old floor to be carried across by a proof, the *bridge*. Those rules are the referee. They decide whether a find is real once a candidate exists; they do not produce one.

## 1\. Where the last post left us

The last post's terms are used here without being explained again:

- **The target:** at most h₁/2 events in all, in two passes over the block, with no more memory than Open, and faster than Open on whole decoding time ([§7](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/#7-where-the-search-stands>)).
- **The floor:** G1 proves that every decoder in the current family pays at least one event per width-1 row, so anything meeting the target is new by theorem ([§3](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/#3-three-routes-back-from-the-bwt>)).
- **The way out it left:** a rule applied to a whole scattered set of rows, short to describe and cheap to apply, in the form Build, Advance, Emit ([§6](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/#6-what-a-new-route-would-have-to-be>)).
- **The method:** the [Mario](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level.md#2-the-ssl-coq-material>) move, which supplies the payoff first and looks for the mechanism second, and the supplied test that came of it ([§2](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/#2-four-reframings-of-the-search>)).

## 2\. Where the breakthrough comes from

Rauzy–Veech induction was not found by counting machine cost. It came from seeing that a whole stretch of rows takes the same step, so that their steps can be composed once and shared. Comparing the two algorithms, the walk and the induction, and asking what carries over between them, which is the question the last post took from [Hilbert's 24th problem](<https://en.wikipedia.org/wiki/Hilbert%27s_twenty-fourth_problem>), gives an answer in two parts:

1. **a notion of "these move together",** which for Rauzy–Veech is adjacent rows holding the same byte;
2. **a rule for composing what moves together,** and then recursing on what is left.

G1 says that width-1 rows have nothing to move together with in that first sense. It does not say that they have nothing to move together with in any sense. Contracting harder in the same sense is the failure to avoid, and our most recent rounds of candidates show what it looks like. The best of them, "arches" cut at the low points of the map and swept twice, reached 0.366 of an event per width-1 row, but only on synthetic random hex and only with a third pass; on our six real blocks the same construction cost 3.18 to 9.03 per width-1 row, and a single sweep we already had gave 0.412 on the same hex. Other builds that looked free walked the map row by row inside them, and one read the true decoded text. The arches themselves turned out to be a known suffix-array technique, SA-IS ([the review](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level/neon-cycle5-critique.md>)). Each candidate squeezed the same notion of moving together further, and paid for it with an extra pass or a hidden walk. Nor does contracting the left-over cycle again escape: that recursion is list ranking by contraction, a known technique for linked lists, and each row it removes still costs that row's own event, so the depth falls but the count does not ([the record](<https://github.com/lmmx/mibz2/blob/299a0624baadacfa079d1bf4d64939b326293b1a/docs/journal/monodromy/2026-11-24-supported-rows.md>); see also [our synthesis](<https://github.com/lmmx/mibz2/blob/cdeef65aba3b5ebad758cf4f5d169f0534bf80ab/docs/journal/monodromy/2026-11-18-synthesis.md>)). The way forward is to change what moving together means. In Louis's words: "what I see is a kind of a looseness of language, a generalization, work smarter, not harder, rather. Find a different sense for something that can occur. It requires a kind of ability to think flexibly, laterally."

No one of these senses leads. Asked which should come first, Louis chose all of them, and named the move itself: "I don't want to frame it as one way only, because, we need this kind of, it's not ambiguity is the wrong word, but like overloaded term. So the term being like generalization, perhaps you might say, is the term, the Hilbert's 24th problem movement. And however you call it, we overload it, right, with the multiple ways in which it might occur. The recurrence, the moving together, the contracting along the text. Like, we want to look at all possible ones. In fact, we might want to look at even more." The senses named so far are pursued side by side, and finding further ones is part of the work ([the record of this principle](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md>)):

- **Contract along the text, not along the rows.** A repeated stretch of text is a run in the other direction, and copy plans are the start of it.
- **Move together up to a consistent relabelling,** rather than by exact equality.
- **Find a recurrence among the induction's own steps,** as the Pell rule in the last post ([§2](<https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/#2-four-reframings-of-the-search>)) generates new triangles where an oracle could only hand them over.

Beside contraction there is now a second principle, *enrichment along composition*. It comes from another of Louis's projects, [page-dewarp](<https://github.com/lmmx/page-dewarp>), whose large speedups came not from accelerating its optimiser but from rewriting its projection so that automatic differentiation could supply gradients, which let it switch to a better algorithm. Forward-mode differentiation lifts a computation into a richer algebra of pairs, with (a, a′)(b, b′) = (ab, a′b + ab′), so that extra information rides along with the ordinary result ([the account](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#37-enrichment-along-composition-the-second-principle>)). Contraction does less work by recognising that computations are the same; enrichment makes work already done yield more, so that later work becomes unnecessary. Here it asks what one LF step, or one step over a run, reveals for free beyond where its own row goes, and whether something carried through an earlier stage, the Huffman or move-to-front decoding, could give the later ones what they need. One test keeps it honest: the rider must be cheap to acquire and must not restate the inversion. Each row's position in the text is the answer itself, and a fingerprint of the column can separate every pair of inputs and still decode nothing ([the principle](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md>)).

The first worked notes already sharpen the hunt ([record 113](<https://github.com/lmmx/mibz2/blob/806aa0123c9a24ad7a1e9ef15da015e1408022f0/docs/journal/monodromy/2026-11-24-moving-together.md>), [record 114](<https://github.com/lmmx/mibz2/blob/bfdbc245fce475fbb7d017501a1e7423dd20eaf5/docs/journal/monodromy/2026-11-24-more-senses.md>)). Every sense of moving together puts the rows in some order, and it helps exactly as far as that order agrees with the order of the text, so the search becomes one for a cheap ordering of rows that correlates with text position. The first candidate worked up in full, *landmark induction*, also shows why the referee reports more than one figure. It starts a chain at every row holding a chosen common byte, rows that sit together in one known block and cost nothing to find, advances all the chains at once to the next landmark, and repeats on what remains. It leaves the number of unpredictable reads where it was, but makes those reads independent of one another, so the chain of reads that must each wait for the last falls from about one per byte to a few hundred on hex-dense content. It separates the two figures cleanly, and the ruling follows that line: a breakthrough means a cut in the unpredictable reads, the currency of the proved floor, while a large cut in the waiting chain is a real find that gets measured ([the ruling](<https://github.com/lmmx/mibz2/blob/3c19c9a2fb4e7831d1bf7a1d0e554c1bc6f8c6f2/channel/coordinator-to-neon.md>)). That measurement, on our six real blocks, is being run.

## 3\. The snag: a convention decided the answer

The decisive gaps in the target were in its accounting. Every known route that beats the floor does so with a *build pass*: a sweep over all the block's [run records](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) that composes facts in bulk, as squaring the map or batching the frontier does. The target never said whether such a pass is charged, and [our record of the question](<https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md>) found that each reading settles it without looking at a single block:

- **If every record a pass reads is charged,** every exact decoder pays for nearly every width-1 row, since each such row's byte is recorded nowhere else. The answer is no, by definition, and it tells us nothing.
- **If a sweep is free and only row-by-row steps are charged,** the number of passes decides it. With passes unlimited, repeated doubling of the map brings the count below h₁/2 with nothing new in it. With the two passes the target allows, the best known decoder, squaring the map on Open's retained rows, still pays at least 0.60 of h₁, and misses.

So the frozen target's verdict was fixed by two conventions, the charge on a pass and the pass budget, and not by any fact about the column. A target that a choice of convention can satisfy or defeat cannot tell a new algorithm from a relabelled old one, which was the whole reason for setting it.

## 4\. Two yardsticks, kept apart

An [outside review](<https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md>) of the problem, one of three [compared side by side](<https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md>), made the decisive distinction. G1 is proved in bookkeeping units: it counts charges in a calculus of facts, not instructions, memory traffic or time. Two yardsticks are therefore in play, and they must not be allowed to blur into one:

|  | Bookkeeping (version 1) | Real machine cost (version 2) |
| --- | --- | --- |
| **What it counts** | events: charges per row in the calculus of rigid facts | unpredictable memory reads, the longest chain of them, passes, and peak memory |
| **What is proved in it** | G1: at least h₁ for the current family | nothing yet: the bridge would carry G1 across |
| **The target** | at most h₁/2 events, as frozen | fewer unpredictable reads and scans than one per width-1 row, within a fixed budget |

The original target stays exactly as frozen, as version 1. The machine-cost target is a new version and is labelled as one, because changing the yardstick without saying so can look like solving the problem. Louis chose the machine-cost meaning of "lower the floor" ([the decision](<https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md>)), on a declared machine model, so that a candidate is judged by what it makes the hardware do.

The model separates the two kinds of memory access that matter. A *streamed* access moves a cursor one step along memory, which the hardware can anticipate. An *addressed* access goes to an address computed from data, which it cannot. Every run reports four figures, all of them, always ([the formal statement](<https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md>)):

- **Q**, the number of addressed accesses;
- **D**, the longest chain of addressed accesses in which each address depends on what the one before it returned, which is what makes the walk slow;
- **P**, the passes: bytes streamed, divided by the block's length;
- **S**, the peak working memory.

The budget fixes the passes at two and the number of live cursors at one per letter of the alphabet, plus a constant, which is enough to distribute rows by their letter. Holding the budget fixed is what makes relocation visible: a fall in Q bought with an extra pass, or with more memory, shows up in the other coordinates instead of disappearing from the count.

**Counting bytes** would have been the obvious yardstick, and the wrong one. A decoder must write every byte of its output, so a target stated in bytes written is impossible for a trivial reason: the output alone costs n, and n is never less than h₁. The review pointed out that trap, and the adopted yardstick counts accesses instead.

## 5\. The bridge: why the floor has to be carried across

In the machine yardstick, no floor of h₁ is yet proved for anything. G1 bounds a sum of charges in the calculus, so it says nothing directly about Q. To claim that a candidate goes below the old family's floor in machine cost, we first need a second theorem: that every decoder in the old family, executed on the declared machine, pays at least a fixed multiple of its bookkeeping charge in Q. Without it, "below the floor" would compare a machine count with a calculus bound, which is not a theorem at all.

The yardstick also has to be fixed before any candidate is judged; otherwise a convenient one can be chosen after the fact. If the bridge fails for some way of implementing the old family, that implementation is not a counterexample to G1. It is a candidate escape, and it gets named.

Two more terms needed the same treatment:

- **A pass** had no definition. There was no rule for a partial scan, for two scans merged into one, or for a scan that reads values it updated earlier in the same scan. The cursor model now supplies one: P counts bytes streamed through cursors, whatever the program calls its loops.
- **New** had been defined by what an object is not ("not rows, not intervals"), which renaming can satisfy. It now means new *relative to the method being escaped*, with a measurable gap in cost. Rauzy–Veech induction shows why the relativity matters. Against the walk it was a genuinely new basis: its cost scales with the number of runs, not of rows, and its records come from one pass. Against today's floor it is no escape at all, because G1's family already contains it. A definition that asked our own historical step to beat h₁ would be testing the wrong floor.

## 6\. The question, stated so it can be answered

With the rules fixed first (the machine model, the yardstick, the inputs and the budget), the question becomes one of existence. Is there a decoder with a new kind of object, a way to build that object from the column without the old pointer-following, and a cost law, such that:

1. it is exactly correct on every input, deterministically;
2. it builds its object only from what it is given, within the budget;
3. its cost, Q above all, is low enough on the random-looking content where the width-1 rows live;
4. and under a model of time fitted to our machine, it beats the current decoder on whole decoding time, judged at one thread and at the default ten separately?

The last condition carries the lesson of the [supplied test](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>): handed a free decoding order, the whole decoder took 0.838 of its time on one thread and 0.988 on ten, two results that differ in kind and cannot be averaged into one.

A negative answer is harder than it sounds. It must rule out every allowed decoder, not the ones we have tried, and it must fix the budget, because with unlimited passes sorting alone brings Q to zero. No lower bound of that kind is known for inverting a permutation presented as a BWT; the only negative result on record is the scoped one, G1, in the bookkeeping yardstick.

**The Mario proof,** on a closer reading, is narrower than our last post allowed, and it is not where the payoff-first move came from. That move belongs to a separate effort on the Shifting Sand Land level. The [Rocq repository](<https://github.com/tra38/sm64-wmotr-abc-proof>) proves, for the Wing Mario over the Rainbow level, that a run with no A press never makes Mario fly, and even that theorem still rests on named residual hypotheses; its theorem that the level needs an A press is not yet started. What we borrow from both is the method, a large claim split into premises that are proved one by one, and not the standing of a finished impossibility proof.

## 7\. What comes next

The same review found errors in our own records. It listed 21 places where records contradict one another, have been superseded, or differ in scope. Among them, the draft negative theorem's list of excluded mechanisms omitted one that its own text calls open. One measurement note compared about forty edges against the number of width-1 rows in an example with two runs, which cannot be right, since a block never has more width-1 rows than runs.

The order of work follows from the two halves of this post. The search is for new senses of moving together, all of them at once, and a candidate is first an idea of that kind, an object with its own rule of composition, before it is a cost. Once one exists, the referee applies: the specification of the machine, the bridge for the current family, and the four figures (Q, D, P, S) with building the object included. In the Mario manner, the supplied test then measures what the candidate's object would buy before anyone pays to build it.

Nor is anything ruled out for having been tried before. Louis again: "it's not necessarily the case that an idea we tried has to be disallowed because it's already been tried, but rather that we can perhaps have, think again and redouble our efforts if we think there might be something in it. But yeah, we shouldn't, feign surprise at novelty if the thing turns out to already exist." Every proposal therefore starts by searching our records, and says either that it is new or which earlier attempt it revisits, and what that attempt missed.

---

**Sources:**

- [Enrichment along composition](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#37-enrichment-along-composition-the-second-principle>) (journal §§36–37) and [the ruling on the two machine figures](<https://github.com/lmmx/mibz2/blob/3c19c9a2fb4e7831d1bf7a1d0e554c1bc6f8c6f2/channel/coordinator-to-neon.md>).
- [The essence of the search](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md>), and [Louis's words on it](<https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level.md#34-the-essence-restated-change-what-moving-together-means>) (journal §34), verbatim.
- [The charge convention](<https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md>): why the frozen target's answer is set by the charge on a pass and the pass budget.
- [The problem, stated formally](<https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md>): the machine model, the cost vector (Q, D, P, S) and the conditions for a new basis.
- [The outside review](<https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md>), recorded verbatim, including its table of contradictions (§5.2) and its reading of the Mario repository.
- [The comparison of the three answers](<https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md>), with the agreed formulation, and [the decision on the currency and the next steps](<https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md>).
- [The supplied-codes result](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>).
