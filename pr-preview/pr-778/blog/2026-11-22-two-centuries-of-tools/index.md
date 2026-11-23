[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/index.md>)

# Two centuries of tools: a claimed Riemann proof's toolkit, and our search

A claimed Riemann proof built one new idea on fifty older tools reaching back two centuries, so our research teams now look back as hard as forward.

22 November 2026 · 10 min read · [Meta](<#6-what-happens-next>)

The claimed proof of the Riemann Hypothesis announced on 20 November draws on a wide toolkit. An independent map of it lists about fifty established techniques, from Parseval's identity in 1799 to the Gan–Gross–Prasad conjectures in 2012. They come from nine branches of mathematics, and twenty-one of them predate 1950. Exactly one entry is marked as the claimed new mechanism. The argument's weight is not where its code is either. Analytic number theory takes 1.4% of the formal proof's lines but an estimated 11% of the argument.

mibz2 is a bzip2 decoder in Rust, and AI research teams search mathematics for faster ways to undo the Burrows–Wheeler transform (BWT) at bzip2's core. What we take from the proof is the shape of its toolkit: a search that ranges across two centuries and many fields, aimed at one new mechanism. That shape works against *recency bias*, the pull toward fashionable modern theory that crowds out classical results. AI research agents feel that pull, and so do we. For inverse BWT, the classical results at risk include nineteenth-century permutation theory, combinatorics on words, continued fractions and the classical theory of interval exchanges.

**Where things stand.** The proof is an unverified mathematical claim. The Lean and Mathlib communities have [certified the formal artifact](<https://leanprover-community.github.io/blog/posts/riemann-hypothesis/>), which Lean's kernel and several independent checkers accepted. Human understanding of the argument is still in progress ([Tao](<https://terrytao.wordpress.com/2026/11/20/unforced-euler-singularity-unforced-navier-stokes-singularity-and-the-generalized-riemann-hypothesis/>)). The figures below come from readings of the proof, not verifications of it. Our own change of method has no results yet.

**The takeaways:**

- **One new mechanism, about fifty old ones.** The proof composes one new idea with established results from two centuries of mathematics. Two fifths of those results predate 1950.
- **Where the code is, the argument often isn't.** Most of the formal proof's lines build infrastructure. Some of the branches that carry the argument hold a small share of the code.
- **Recency bias is a risk for a search like ours.** Our teams now search classical mathematics as deliberately as modern, and record the era of every tool a chain uses.
- **A broad toolkit needs a way to be used.** The proof's route transplants a solved case, Stepanov's proof for curves over finite fields, step by step. Each step that fails over the integers says which tool to fetch. We use the same mechanism to say where each old tool enters our problem.

[Our earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-21-besieging-hard-problems/index.md>) and our [research journal](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) set out the proof's shape as twelve levers. This post is about its toolkit.

## 1\. What is in the proof's toolkit?

Eyal Weiss's [anatomy of the claim](<https://eyal-weiss.github.io/rh-proof-anatomy/rh_anatomy.html>) maps which branches of mathematics the proof draws on and when each technique appeared. It is a text scan of the 175,187-file Lean archive, not a build or a verification. Its importance weights are judgement calls, re-estimated by an AI model from the announcement's outline and the Lean findings.

Its timeline lists 50 established techniques and one entry marked as the claimed new mechanism: multiplicity from iterated theta lifts, in 2026.

Fifty established techniques and one new mechanism, by the year each idea appeared

Data from Weiss's anatomy (timeline table), redrawn; years mark an idea's first appearance, and the form used is usually later. Highlighted: the solved case, RH for curves over finite fields (1948); the method transplanted from it, Stepanov's (1969); and the claimed new mechanism (2026). The Riemann Hypothesis itself (posed 1859) and Lean 4 (2021) are not counted as techniques.

The oldest are Parseval's identity (1799) and Gauss sums (1801). Theta functions date from 1829 and complex multiplication from 1857. The Siegel mass formula is from 1935. These are not decorations: Weiss's outline assigns each to a step of the argument. The newest established result is from 2012, so the proof's single new idea sits on top of a stack that took two centuries to build.

## 2\. Where is the argument, and where is the code?

Weiss also compares each branch's share of the Lean code's lines with its estimated importance to the argument.

Share of the formal proof's code against estimated importance to the argument, by branch

Data from Weiss's anatomy, redrawn. Code shares are measured by scanning names in the Lean files as text; treat anything under about 7% as rough. Importance is an AI-assisted judgement with a range of several points either way.

Abelian varieties and schemes take 24% of the code but an estimated 6% of the importance. Analytic number theory takes 1.4% of the code and an estimated 11%; Iwasawa theory takes 5% and an estimated 12%. The code is mostly infrastructure: foundations that Mathlib lacks, built so that a small core can carry the argument.

For a search, this says how to judge a chain of results: by the step it carries, not by how much scaffolding it needs. A classical theorem that settles one step in a page can matter more than a modern framework that needs thousands of lines to set up.

## 3\. Why would a search drift toward the recent?

A research agent proposes what it finds salient, and what is salient skews recent. Recent work is more heavily cited and more often summarised, and it is closer to the vocabulary of the problem as stated today. The pull applies to the language models our research teams run on and to the people who direct them. We have not measured it in our own teams; it is a risk we now design against, not a finding.

Our first attempt at breadth did not address it. We gave the breadth-oriented team a quota over the Mathematics Subject Classification (MSC2020, 63 top-level areas), asking each investigator to open untouched, far-apart areas every cycle. A quota over subject areas says nothing about era. A team can satisfy it entirely with post-2000 frameworks, and the subject a classical result now sits in may not even look relevant.

The proof's toolkit suggests a different guard. Our team instructions now ask each research chain to:

- mine classical mathematics as hard as modern, from the nineteenth century on;
- not invent where a classical result supplies the step;
- record each tool's field and year, and which tool is new.

That last record makes the distribution of a chain's tools visible, so a chain built only from recent work can be seen to be one.

## 4\. Which classical tools does inverse BWT have?

The BWT's inverse is a permutation, LF, and much of its classical mathematics is older than computing. Our teams have already drawn on some of it:

Tools our research teams have drawn on, by the year each idea appeared

Each tool appears in our research journals or FRONTIER.md as part of a team's work. Years are first appearances. The list is not complete, and it shows what we have drawn on, not what we should.

Some of these are classical in the sense this post means:

- **Combinatorics on words.** Christoffel words (1875) and Sturmian sequences (Morse and Hedlund, 1940) give the two-letter blocks whose inversion is pure arithmetic. The Chen–Fox–Lyndon factorisation (1958) underlies the bijective BWT.
- **Continued fractions and the Euclidean algorithm.** On a two-letter Sturmian block, Rauzy induction *is* the Euclidean algorithm, and the block has a grammar of size O(log n).
- **Interval exchanges and rotations.** LF is an interval exchange. Keane's work (1975), Rauzy's induction (1979) and Veech's (1982) gave our [Rauzy–Veech reconstruction](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-10-rauzy-veech-reconstruction/index.md>).
- **Nineteenth-century permutation theory.** LF is one permutation of a block's rows, and its cycle structure is the classical object. This is the area our teams have used least.

The later tools in the figure, recompression (Jeż, 2015) and string attractors (Kempa and Prezza, 2018), are useful, but a search that reaches only for them is the bias in action.

## 5\. How does a broad toolkit get used well?

Breadth alone does not say where a tool enters. In the proof, the mechanism is a transplant. Stepanov's 1969 proof of the Riemann Hypothesis for curves over finite fields works with an auxiliary function that vanishes to high order. An [illustrated guide](<https://nasqret.github.io/riemann-rev/guide/proof-guide.pdf>) to the proof's architecture calls its shape "Stepanov over ℤ", and gives the dictionary row by row:

The solved case, transplanted: each row of Stepanov's proof and its substitute over the integers

The four rows are the guide's dictionary, quoted. The fields on the right are where each substitute's tools come from, read from the guide's flow diagram. A reading of the proof's architecture, not a verified dependency graph.

Each distant theory in the toolkit entered because one row needed a substitute. Iwasawa theory and the Ferrero–Washington theorem (1979) entered because Stepanov passes to all finite extensions of a field, and the proof needed a tower of fields instead.

We use the same mechanism. Each research chain starts from a *solved case*, a kind of block whose inversion is cheap:

- a perfectly clustering block, one run per letter;
- a two-letter Sturmian block;
- a periodic text;
- one of Open's thick strips, which collapses in one jump.

The chain writes the cheap inversion as explicit steps, runs it on real blocks, and finds which steps fail. A failing step names the ingredient to search for, in mathematics of any era. Our [cost map](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-20-where-the-decoding-work-goes/index.md>) of the Wikidata benchmark already says where the first steps fail:

Our first dictionary: the cheap case's steps, where they fail on real blocks, and the ingredient each failure asks for

Failure rates measured on six Wikidata blocks (working-backwards journal, cost map); the classes are regex approximations and "thin" is a run-width proxy. The right-hand column is what the search is for: open questions, not results.

The economics are unchanged. Our charter's rule of [mathematical maximalism](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/METHOD.md>) holds: unlimited ambition in the mathematics that discovers a decoder, strict economy in the decoder that runs. A chain whose substitutes hold but don't yet pay goes to the [Foundational list](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/FRONTIER.md#foundational>). Native timing on our test machine, an i9-10900K, is the final word.

## 6\. What happens next?

The breadth-oriented team, Kitchen Sink, adopts the toolkit rule and the transplant at its next cycle boundary. A separate team is working on contracting a block directly in the space of interval exchanges; [its first round](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-22-neon-teamwork-paradigm.md>) has been audited. Neither has a result from the new method yet.

The first thing to report will be each chain's own anatomy: its tools by field and year, which one is new, and which steps carry the argument. If our chains turn out to draw only on the last thirty years, that will be a finding too.

---

**Sources:** [Weiss's anatomy of the claimed proof](<https://eyal-weiss.github.io/rh-proof-anatomy/rh_anatomy.html>) (a text scan, with AI-assisted importance estimates); [the architecture guide](<https://nasqret.github.io/riemann-rev/guide/proof-guide.pdf>); [the proof explained for the zeta function](<https://www.math.uwaterloo.ca/~mrubinst/alpoge_claude_grh/zeta-paper.pdf>), by Claude Fable 5.2, Codex Astra 6.2 and Gemini Carbon at the behest of Michael O. Rubinstein (working draft, 21 November 2026); [the Lean community's statement](<https://leanprover-community.github.io/blog/posts/riemann-hypothesis/>); [Terence Tao on the claim](<https://terrytao.wordpress.com/2026/11/20/unforced-euler-singularity-unforced-navier-stokes-singularity-and-the-generalized-riemann-hypothesis/>).

**Research records:** [the twelve levers](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>); [working backwards and the cost map](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-20-working-backwards.md>); [the Rauzy–Veech literature](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/monodromy/2026-11-10-rauzy-veech.md>) and [the Lyndon literature](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/monodromy/2026-11-10-lyndon-literature.md>); [the research method](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/METHOD.md>); [the interval-exchange team's first round](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-22-neon-teamwork-paradigm.md>).
