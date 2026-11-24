# ChatGPT Pro on the narrowed question: closure under composition (verbatim)

*24 November 2026, pasted by Louis. Inline links did not survive the paste (`⁠￼`). ChatGPT's reproducer, `mibz2_band_census.zip`, sits in its own sandbox and is not in this repository. The Coordinator's response is in journal §42.*

---

Louis, I found a concrete place to investigate, and a concrete reason why its simplest version fails. I also found two claims in the summaries that are stronger than the underlying records support. Those claims are making the search look more boxed in than it actually is.

I read the source notes, relevant implementation and experiment records, the page-dewarp PRs, and the newer Neon critique that followed the exchange you pasted. I then ran a structural experiment on one of your admitted real blocks.

My main conclusion is:

The useful next question is whether a compact description of how to access the data can remain cheap to use after composition. Your existing mathematics already supplies such a description for one important operation. My experiment shows that its simple form fragments badly after that operation.

That is a specific missing mechanism to investigate. It is not yet a breakthrough candidate, and I would not present it as one.

1. Two conclusions need to be narrowed before they guide further research

The singleton theorem is not a proved lower bound on unpredictable memory reads

This distinction is already present in your careful source documents.

The scoped G1 theorem counts certain operations in a particular calculus of interval facts. A composition resulting in a single-row domain is charged, including when an implementation could perform that composition through a stream. It establishes a provenance bound within that calculus.

It does not establish that every implementation must perform one unpredictable memory read per singleton.

Your council's comparison says this explicitly: a theorem connecting those proof charges to the chosen machine costs is still needed. MONODROMY's later specification also warns that streamed composition can make the two currencies diverge. See the scoped G1 statement⁠￼, the council comparison⁠￼, and the machine specification and bridge discussion⁠￼.

Writing h_1 for the number of singleton rows, the distinction is:

\text{at least }h_1\text{ charged proof operations}
\quad\not\Rightarrow\quad
\text{at least }h_1\text{ unpredictable machine reads}.

This matters to your search. An implementation could preserve the same mathematical interval facts while discovering a substantially better way to acquire, arrange, or consume them. The theorem would remain true.

Keep "fewer than one unpredictable read per singleton, acquisition included" as your ambitious target. Treat the universal machine lower bound as unproved.

That does not make the target easy. It changes which arguments can legitimately rule out a route.

The 16% result is not a ceiling on every possible inverse-BWT improvement

I checked the actual supplied-codes experiment behind this conclusion.

It supplied Open's word codes in output order, with their acquisition treated as free. But it retained substantial work: Open construction, word lookup and copying, dictionary-word emission, long-word handling, and the later output processing.

The recorded results were:

* Emitter time approximately halved: ratios 0.53 at one worker and 0.55 at ten.
* Whole-decoder time became 0.838 of baseline at one worker.
* At ten workers, whole-decoder time was 0.988, with a confidence interval spanning no change.

The registration explicitly declines to draw a global conclusion about all alternative representations. See the registered experiment⁠￼ and journal §30⁠￼.

The supported conclusion is:

Supplying these particular codes saved about 16% of single-worker decode time and produced no measurable whole-decoder gain at ten workers.

The broader statement—

Even perfect inverse BWT can save at most 16%, and essentially nothing at ten workers—

does not follow from that intervention.

This experiment is strong evidence against investing heavily in that particular consumer with only its code-ordering problem removed. It does not settle an algorithm that also removes construction, changes word emission, reduces shared memory traffic, or changes the division of work between scanner and workers.

The careful records contain these qualifications. Some later summaries lose them. I would correct the summaries before passing them to another mathematical agent.

2. Your gradient observation identifies something more specific than "composition"

Looking back at page-dewarp, the most useful lesson is the sequence of algorithmic changes.

PR #130⁠￼ introduced the JAX path, associated with release 0.2.4. This combined a differentiable objective with automatic gradients and L-BFGS-B. The previous default was Powell, a derivative-free optimizer.

Then PR #135⁠￼ increased L-BFGS-B's maxcor from 10 to 100. This lets the optimizer retain more information from previous parameter and gradient changes when estimating the objective's curvature.

For the Boston A example, the PR reports:

Configuration	Optimization time	Objective evaluations
JAX L-BFGS-B, maxcor=10	2.01 s	871
JAX L-BFGS-B, maxcor=100	0.84 s	147

The same PR reports 12.18 seconds for the earlier Powell configuration, giving the cumulative 14.5× comparison for that example. These are the project's reported example measurements, rather than a guarantee across all documents. Source: PR #135⁠￼.

There are two related improvements here:

1. Automatic differentiation made useful derivative information accurate and cheap enough to use.
2. Retaining more curvature information let the optimizer take far fewer evaluations to reach a comparable result.

The second change did not improve the numerical accuracy of the gradient itself. It improved how much the optimizer could infer from the gradients it had already obtained. SciPy's definition of maxcor is precisely about the stored corrections used in its limited-memory approximation. 

There is also a technical qualification to the "pairs of value and derivative" story. Those pairs illustrate forward-mode differentiation. For a scalar objective with hundreds of parameters, the particularly useful operation is reverse-mode differentiation, which shares the evaluation's intermediate computation and propagates derivative information backwards. Computing the complete gradient by separately carrying one forward derivative for every parameter would lose much of the advantage. 

So the reusable pattern is quite concrete:

Obtain information that makes a more capable algorithm possible; then retain enough of that information that the algorithm can avoid expensive work.

That suggests a sharper question for mibz2:

Which later operation is necessary because an earlier stage has failed to retain—or cannot cheaply exploit—information it already encountered?

"Find a richer algebra" can generate ideas. This question helps identify what the extra information would actually enable.

3. There is already more cross-stage work than the summaries suggest

This is worth establishing so another agent does not spend its first round rediscovering it.

In the implementation I inspected, the front end already converts inverse-MTF/RLE2 information directly into run records and byte counts for the Open path. It does not first materialize every conventional intermediate representation, including a complete byte column, and then hand those arrays through independent textbook stages. Source: ⁠￼src/runs.rs⁠￼.

The November 22 work also tested a concrete combined representation: Huffman lookup results carrying composed run actions and MTF information. That went beyond merely placing two loops next to each other. It lost: the recorded scanner time increased from 74.5 to 107.9 ms, and whole-decoder ratios were around 1.185. Source: the Neon teamwork journal⁠￼.

Likewise, composing RLE1 and CRC summaries through the word grammar was already explored in the November 10 mathematics. That is a genuine example of useful information travelling through composition. Its existence does not establish a large whole-decoder payoff. Source: the Rauzy–Veech record, particularly the RLE1/CRC sections⁠￼.

These experiments do not settle whether one representation could serve all the stages effectively. They mean that "a shared representation" needs to name a specific capability that the existing representations lack.

The newer Neon movement critique⁠￼ also supports the cautious reading of that round: sorting-based pointer composition and list contraction can exchange irregular access for substantial global traffic. "Zero unpredictable reads" in that model is not a measured zero-cost implementation.

I also accept the correction to my earlier selective-hex suggestion. C-2 tested essentially the proposed escape and charged its acquisition. Its negative result closes that version of the idea; it should not be presented again under different terminology. Source: the Carbon critique⁠￼.

4. A concrete structural lead: the first composition has a cheap address pattern

Here is the part I investigated directly.

The existing factorization

After Open has removed the long-run interiors, there is a smaller set of retained rows. Let:

* v be those rows' byte labels, in row order;
* LF_v be the ordinary LF map on those labels;
* R be the map taking each retained row to the next retained row along the original walk.

An existing MONODROMY record gives the factorization

R=P\circ LF_v,

where P is a piecewise translation: within each of its intervals, it adds a fixed offset. This factorization is already in the project; I am not claiming it as a new discovery. Source: Open first-return verification⁠￼.

For successive occurrences of a particular byte c,

LF_v(x)=C[c]+\operatorname{occ}_c(x)

advances by exactly one. While these values remain within one translation interval of P, their targets under R also advance by one.

Consequently, when a composition sweep needs to read a record at R[x], it need not necessarily treat every such target as an unrelated address. For each byte, it can advance through consecutive target addresses, restarting when it crosses a translation boundary.

The grouping is:

Rows with the same byte whose occurrence ranks lie within the same translation interval.

Those source rows can be far apart. A singleton can participate.

This does not share away all their computation. It supplies an orderly access schedule for one part of it. It therefore illustrates exactly why a charge for each singleton composition is not automatically a charge for each unpredictable address.

The important limitation

A small number of address segments does not mean a small number of total reads.

The sweep still reads the target records. It must also read source records, handle word payloads, and write its results. Cursor state costs something. Many interleaved streams need not behave ideally on real hardware.

Furthermore, once the sweep replaces R with a composed map, the new targets may no longer have the original simple pattern.

That last question was cheap enough to test immediately.

I ran the test on your admitted block 0

The input has:

* 899,931 column bytes;
* 35,116 singleton rows;
* 89,874 retained rows under the width-16 rule, including the primary row.

For each byte, I counted a new target-address segment whenever its next target was not exactly one greater than its previous target.

Then I performed one increasing-row-order, in-place composition sweep, using the same basic composition pattern as the earlier M-17 work. I tested limits of 8 bytes, 16 bytes, and no limit on newly accepted word concatenations.

Here is the result:

Map being examined	Consecutive target-address pieces	Pieces per singleton	Links followed during emission
Original retained map	2,148	0.061	89,874
After one sweep, 8-byte limit	44,814	1.276	42,325
After one sweep, 16-byte limit	46,671	1.329	40,009
After one sweep, unlimited	52,319	1.490	32,833

The limits apply to new concatenations; existing longer base words remain present.

All three composed versions reproduced the reference marked LF byte stream exactly on this block. That is a check of this inverse-BWT traversal, not a complete bzip2/RLE1/CRC validation or a correctness proof for every possible input.

The outcome is clear:

The initial access pattern is compact. One composition makes it roughly 21–24 times more fragmented under the same byte grouping.

The uncapped sweep also increased the total bytes in the diagnostic's flat word storage from 899,931 to 2,507,122. A production representation could share word contents, but the cost of consuming that shared representation would then need to be included.

I saved the complete reproducer, exact input, output and scope notes here:

mibz2_band_census.zip⁠￼

What this does—and does not—establish

This was an offline structural diagnostic. I deliberately acquired the retained map through LF walking. That acquisition was an oracle for examining the resulting structure, not a proposed cheap construction.

Accordingly:

* Unpredictable accesses: the segment count concerns one successor-address schedule. It is not the complete decoder's Q.
* Dependent depth: the emission-link count is informative, but the experiment does not measure the full dependency chain through construction and word handling.
* Passes: this diagnostic is not an implementation satisfying the two-pass budget.
* Memory: it uses explicit maps and flat words; it establishes no compact production-memory bound.

There is no native speed claim.

What it does give us is an exact failure point. The obvious recursive proposal—"use the same byte cursors again after composing"—fails on the first real block.

That is more useful than either celebrating the attractive initial 2,148 figure or dismissing the whole idea with "every retained row necessarily costs an unpredictable read."

5. The missing property is that the representation must remain cheap after composition

Mathematicians call this kind of requirement closure: after applying an operation, the result still belongs to a representation you know how to handle efficiently.

Automatic differentiation succeeds because every supported primitive has a local derivative rule that composes with the others. The derivative machinery has a controlled relationship to the original computation.

For the cursor idea, the situation is different:

1. The original map has a small, useful description of its access pattern.
2. Composition shortens the eventual walk.
3. That composition destroys much of the simple access pattern.

The important question is whether a different representation can avoid that third consequence without moving its cost into construction or emission.

A concrete research assignment would be:

Starting from R=P\circ LF_v, find a representation of composed retained-row transitions that keeps both successor acquisition and emitted-word handling cheap. Its compactness must survive composition on the admitted blocks, and its construction must not require following the transitions it is supposed to replace.

That is much more specific than "try semirings" or "find another ordering".

The representation needs to support three things together:

* determining the next retained row;
* recovering the intervening bytes;
* obtaining the data needed for those operations through a small, efficiently acquired access schedule.

Simply retaining the expression P\circ LF_v\circ P\circ LF_v is not enough. If evaluating that expression reintroduces the original dependent lookups, the work has merely been deferred.

Similarly, choosing a new grouping after every sweep may restore some order, but sorting or walking to discover that grouping must be charged.

This is one concrete research question, not a claim that every possible breakthrough must have this form. I would not replace one overly narrow universal rule with another.

There is a related qualification to the "answer in disguise" rule. Assuming text positions for free is invalid. Discovering a cheap algorithm that computes text positions from the available representation is a legitimate solution route. The issue is unpaid acquisition, not the name of the resulting information.

What would change my verdict on this lead?

A new representation would need to show, first on this same block, that it can avoid the observed fragmentation while still producing the word payloads cheaply.

Then it needs:

* the same result across all six admitted blocks;
* exact semantics, including the periodic/multiple-cycle cases that ordinary bzip2 can contain;
* a complete cost account for construction, access scheduling, word handling and emission.

A successful second composition with a cheap, explicitly constructed representation would be a meaningful new result. Merely reducing the first sweep's address-segment count further would not resolve the demonstrated problem.

6. The whole-decoder question needs a combined experiment

The ten-worker result remains important. A beautiful inverse-BWT construction that does not change total decode time would not satisfy your practical ambition.

But testing improvements one at a time can miss a real interaction.

Suppose, purely as an illustration, a pipeline takes 100 ms in its front end and 100 ms in its worker stage. Making either stage faster by itself can leave the total near 100 ms. Improving both can reduce it substantially. A null result for each isolated change does not establish a null result for the combination.

That is why I would distinguish the existing supplied-codes experiment from a two-by-two supplied-capability experiment:

Arm	Scanner output	Open emission
A	Normal scanner	Normal emission
B	Normal scanner	Existing supplied-code intervention
C	Precomputed ordinary Symbols output	Normal emission
D	Precomputed ordinary Symbols output	Supplied-code intervention

The precomputation would deliberately sit outside the timer: this is a diagnostic of potential benefit, not a proposed implementation.

Delivery of the symbols and codes must remain timed. Use the production bounded queue and preserve the ordinary remaining construction, word handling, RLE and CRC work. The point is to ask whether removing both specific costs exposes a benefit that neither removal exposes alone.

I have not run this experiment.

It would answer a useful outstanding question:

Is there enough combined benefit at these two existing interfaces to justify searching for an algorithm that changes them together?

If D barely improves on A, that is strong negative evidence for that particular pair of capabilities. If D improves substantially while B and C do little individually, the isolated ten-worker result has been masking an interaction.

Even a positive result would leave the hard algorithmic question—how to acquire those capabilities cheaply—but it would establish that the acquisition problem is worth solving.

This should not become another rediscovery of parallel scanning

I checked the newer history here too.

The November 22 Open-plus-parscan experiment already existed. Its ten-worker wall ratio was about 0.883, but it regressed at six workers and increased memory substantially; the comparison also included a faster block reader. Source: C-37 registration and result⁠￼.

The later bounded streaming version lost badly: about 1.54× baseline wall time at ten workers on Wikidata. The coordinator's postmortem discusses possible starvation and scheduling causes, which remain hypotheses rather than an established diagnosis. Source: the recorded postmortem⁠￼.

So "try parallel Huffman scanning now that Open uses less memory" is already covered by the history. The proposed combined oracle has a different purpose: it isolates whether the two capabilities together have useful potential before another implementation is attempted.

My verdict and the direction I would take

On the concrete cursor idea: SOLID BUT INCREMENTAL for the first operation. Its immediate recursive version fails the real-block test. I would not call it a breakthrough candidate.

On the larger search: it is not established to be impossible by either the singleton theorem or the 16% experiment. Those are narrower results than the summaries sometimes make them sound.

I would make three changes to the active research brief:

1. State the proven bound at its actual scope. Keep your machine-read target, and identify the missing bridge instead of treating it as established.
2. Replace the universal 16% ceiling with the exact supplied-codes result. Use the combined experiment to investigate the remaining whole-program question.
3. Give one agent the concrete composition problem above. Supply the existing factorization and the new fragmentation result, so it starts at the point where the obvious construction fails.

Your page-dewarp observation remains useful here. The strongest part is that better information changed what the consuming algorithm could accomplish with each expensive evaluation. In mibz2, we can now identify one precise kind of useful information: a compact description of where successor records will be found.

The experiment shows why that information is difficult to keep useful as the computation is contracted. Finding a representation that survives that transformation—and still delivers the bytes cheaply—would be a substantive algorithmic advance. That is the specific gap I would now investigate.
