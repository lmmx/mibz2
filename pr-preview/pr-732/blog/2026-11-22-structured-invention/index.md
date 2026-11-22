[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-732/blog/index.md>)

# Structured invention: borrowing a solved case, one step at a time

A claimed Riemann proof drew on fifty classical tools because each step of an older proof needed one, and our research teams now invent the same way.

22 November 2026 · 11 min read · [Meta](<#5-what-happens-next>)

mibz2 is a bzip2 decoder in Rust. Much of its research is done by AI research teams, looking for faster ways to undo the Burrows–Wheeler transform at bzip2's core. [Besieging hard problems](<https://lmmx.github.io/mibz2/pr-preview/pr-732/blog/2026-11-21-besieging-hard-problems/index.md>) described what we took from the claimed proof of the Riemann Hypothesis announced on 20 November. Its route ran through mathematics built for other problems, and our [research journal](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) set out twelve "levers" read from its shape. This post is about the next step: how those levers become a procedure our teams can follow.

**Where things stand.** The proof is still an unverified mathematical claim. The Lean and Mathlib communities have [certified the formal artifact](<https://leanprover-community.github.io/blog/posts/riemann-hypothesis/>): Lean's kernel and several independent checkers accepted it. Human understanding of the argument is still in progress ([Tao](<https://terrytao.wordpress.com/2026/11/20/unforced-euler-singularity-unforced-navier-stokes-singularity-and-the-generalized-riemann-hypothesis/>)). What we use is its architecture, as others have now mapped it. Our own method change has no results yet; this post is about the method and why we changed it.

**The takeaways:**

- **Breadth by quota has the wrong shape.** The proof's breadth came from transplanting a solved case, Stepanov's proof for curves over finite fields, step by step. Distant mathematics entered only where a step needed a substitute.
- **One new mechanism, about fifty old ones.** A map of the proof lists 50 established techniques and exactly one new mechanism. Twenty-one of the fifty predate 1950.
- **The code is not where the argument is.** Most of the formal proof's lines build infrastructure. The branches that carry the argument hold a small share of the code.
- **Our version** starts from easy cases of our problem, finds on real data exactly which steps of their cheap solution fail, and searches all of mathematics for a substitute for each failing step. The final judge is still native timing.

## 1\. What did breadth by quota look like?

Our breadth-oriented team, Kitchen Sink, runs the Reconstruction Programme, modelled on the Langlands programme. It proposes *correspondences* between the structure of a bzip2 block and objects in distant fields, each with a precise dictionary and testable conjectures. Its goal is *functoriality*: a cheap construction on the far side that transfers to exact reconstruction on ours. Block-specific kernels with a selector, a lead from our [working-backwards journal](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-20-working-backwards.md>), are its specialisations.

To keep the search wide, we first gave the team an atlas, the Mathematics Subject Classification (MSC2020, 63 top-level areas), and a quota. Each investigator was to open at least two untouched, far-apart areas a cycle, and the team kept a coverage table. That forces breadth for its own sake, and it invites box-ticking. Counting an area only when it produced a testable conjecture helped, but the procedure still had the wrong shape. A closer reading of how the proof was built showed why.

## 2\. How was the proof's breadth actually built?

Two maps of the proof appeared within a day of the announcement. Neither is a verification; both are readings of the formal proof and the announcement.

**The architecture.** An [illustrated guide](<https://nasqret.github.io/riemann-rev/guide/proof-guide.pdf>) "reads the proof backwards from the theorem". Its summary: "One hard estimate and two classical transformations of it." Arithmetic geometry bounds a smoothed sum over primes from above, by a square root, on sparse but long intervals. A localized form of Landau's theorem turns that one-sided, sparse bound into a region free of zeros. Functional equations then carry that region to the critical line.

**The shape: Stepanov over the integers.** The Riemann Hypothesis has a solved analogue. For curves over a finite field it was proved in 1948. In 1969 Stepanov gave an elementary proof, using an auxiliary function that vanishes to high order at the curve's rational points. Its degree then bounds the number of points. The announcement calls its own argument "a descendant of Stepanov's", with each theta lift "mimicking multiplicity". The guide gives the dictionary, row by row:

The solved case, transplanted: each row of Stepanov's proof and its substitute over the integers

The four rows are the guide's dictionary, quoted. The fields on the right are where each substitute's tools come from, read from the guide's flow diagram. A reading of the proof's architecture, not a verified dependency graph.

Each distant theory is there because one row needed a substitute. Stepanov's auxiliary function vanishes to order m. Over the integers no such vanishing exists, so the proof iterates theta lifts on unitary groups and uses the divisibility of the result. Stepanov counts degrees; Bost's slope inequality, from Arakelov geometry, counts instead. Stepanov passes to all finite extensions of the field. The proof climbs the layers of a cyclotomic tower instead, which brings in Iwasawa theory and the Ferrero–Washington theorem. The breadth came from the steps that needed ingredients, not from sampling fields.

**The anatomy.** Eyal Weiss's [anatomy of the claim](<https://eyal-weiss.github.io/rh-proof-anatomy/rh_anatomy.html>) maps which branches of mathematics the proof draws on and when each technique appeared. It is a text scan of the 175,187-file Lean archive, not a build or a verification. Its importance weights are judgement calls, re-estimated by an AI model from the announcement's outline and the Lean findings. Two of its findings changed our method.

First, its timeline lists 50 established techniques, from Parseval's identity in 1799 to the Gan–Gross–Prasad conjectures in 2012. One more entry is marked as the claimed new mechanism: multiplicity from iterated theta lifts. The solved case and the transplanted method are both on it. Twenty-one of the fifty predate 1950.

Fifty established techniques and one new mechanism, by the year each idea appeared

Data from Weiss's anatomy (timeline table), redrawn; years mark an idea's first appearance, and the form used is usually later. Highlighted: the solved case (1948), the transplanted method (1969) and the claimed new mechanism (2026). The Riemann Hypothesis itself (posed 1859) and Lean 4 (2021) are not counted as techniques.

Second, where the code is and where the argument is diverge. Weiss compares each branch's share of the Lean code's lines with its estimated importance to the argument:

Share of the formal proof's code against estimated importance to the argument, by branch

Data from Weiss's anatomy, redrawn. Code shares are measured by scanning names in the Lean files as text; treat anything under about 7% as rough. Importance is an AI-assisted judgement with a range of several points either way. Abelian varieties and lattices take almost half the code; Iwasawa theory and analytic number theory, a sixth of it at most, carry almost a quarter of the estimated importance.

Abelian varieties and schemes take 24% of the code but an estimated 6% of the importance. Analytic number theory takes 1.4% of the code and an estimated 11%. The code is mostly infrastructure: foundations that Mathlib lacks, built so that a small core can carry the argument. A chain should be judged by the step it carries, not by how much scaffolding it needs.

## 3\. What do we do now?

The method, as now written into our research teams' instructions, has seven parts. We call it *structured invention*.

**Start from solved cases.** These are worlds where inverting the transform is free or cheap.

- a *perfectly clustering* block, one run per letter, whose text can be computed rather than walked;
- a two-letter *Sturmian* block, where the inversion is the Euclidean algorithm and the block has a grammar of size O(log n);
- a periodic text;
- the thick strips in our Open kernel, where a whole strip of rows collapses in one jump.

**Write the cheap inversion as explicit steps,** its own "Stepanov proof". **Run it on real blocks** and find exactly which steps fail, where, and on which rows. **Treat each failing step as a missing ingredient:** state the property it needs, then search all of mathematics for a substitute. MSC2020 stays, but as the catalogue to search, not a quota. Each investigator records every area tried for an ingredient, and why it did or did not supply one.

The output is a dictionary per solved case: step, substitute, source field, test on the blocks, cost. A Referee checks each substitute for exactness, and checks that the assembled chain still inverts exactly.

We know where the first steps will fail, because our [cost map](<https://lmmx.github.io/mibz2/pr-preview/pr-732/blog/2026-11-20-where-the-decoding-work-goes/index.md>) of the Wikidata benchmark shows where the expensive rows come from. The step every cheap case relies on, a run of one letter moving as a block, fails on rows that sit in runs of one. Statement IDs are random GUIDs, a few percent of the bytes but over half of the one-row runs. Each is unique, with no earlier copy to borrow. Repeated sha1 hashes are 6–25% of the thin-run rows, and they are copies of something already decoded. The JSON skeleton is already free. Its keys are a third of the bytes but a few percent of the thin-run rows.

Our first dictionary: the cheap case's steps, where they fail on real blocks, and the ingredient each failure asks for

Failure rates measured on six Wikidata blocks (working-backwards journal, cost map); the classes are regex approximations and "thin" is a run-width proxy. The right-hand column is what the search is for: open questions, not results.

**Calibrated by the anatomy,** each chain aims at one new mechanism composed with established results. It does not invent where a classical result supplies the step. It mines classical mathematics as hard as modern, from combinatorics on words and continued fractions to nineteenth-century permutation theory. Each surviving chain gets its own anatomy: its techniques by field and year, which one is new, and which steps carry the argument rather than the infrastructure.

**The economics do not change.** Our charter's rule of [mathematical maximalism](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/METHOD.md>) stands: unlimited ambition in the mathematics that discovers a decoder, strict economy in the decoder that executes. A chain whose substitutes hold but don't yet pay goes to the [Foundational list](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/FRONTIER.md#foundational>), kept for its mathematics. Native timing on the i9-10900K, the machine we test on, is the final word.

## 4\. What changed, side by side?

Breadth by quota, and breadth by transplant

Conceptual: the two procedures as our team instructions describe them. Under the quota, an area is opened because it is untouched and far away. Under the transplant, an area is searched because a named step needs an ingredient, and every attempt is recorded against that step.

The quota asked investigators to go somewhere new. The transplant asks them to go somewhere for a reason. Both reach distant mathematics, but only the second produces a chain whose links are each tied to a step that failed on real data, and whose failures say which link to look for next.

## 5\. What happens next?

Two teams are working this way. Teamwork Project Sentinel is running the Reconstruction Programme now, in Kitchen Sink's cycles 46 to 51, and adopts structured invention at its next cycle boundary. The Neon Breakthrough Team runs a separate foundational track, on contracting a block directly in the space of interval exchanges. [Its first round](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-22-neon-teamwork-paradigm.md>) has been audited by MONODROMY, our verification role. Neither has a result from the new method yet.

The bar is the one the proof suggests: one new mechanism, composed with established mathematics, aimed at a step that fails on our blocks, and judged by native timing. We will report what the first dictionaries find, including the steps for which no substitute exists.

---

**Sources:** [the architecture guide](<https://nasqret.github.io/riemann-rev/guide/proof-guide.pdf>); [Weiss's anatomy of the claimed proof](<https://eyal-weiss.github.io/rh-proof-anatomy/rh_anatomy.html>) (a text scan, with AI-assisted importance estimates); [the proof explained for the zeta function](<https://www.math.uwaterloo.ca/~mrubinst/alpoge_claude_grh/zeta-paper.pdf>), by Claude Fable 5.2, Codex Astra 6.2 and Gemini Carbon at the behest of Michael O. Rubinstein (working draft, 21 November 2026); [the Lean community's statement](<https://leanprover-community.github.io/blog/posts/riemann-hypothesis/>); [Terence Tao on the claim](<https://terrytao.wordpress.com/2026/11/20/unforced-euler-singularity-unforced-navier-stokes-singularity-and-the-generalized-riemann-hypothesis/>).

**Research records:** [the twelve levers](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>); [working backwards, the zoo and the cost map](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-20-working-backwards.md>); [the research method](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/team/research-charter/METHOD.md>); [the Neon Breakthrough Team's first round](<https://github.com/lmmx/mibz2/blob/426b4d724/docs/journal/2026-11-22-neon-teamwork-paradigm.md>).
