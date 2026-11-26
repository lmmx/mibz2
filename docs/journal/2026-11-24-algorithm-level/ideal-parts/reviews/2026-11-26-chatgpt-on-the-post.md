# ChatGPT's review of the post "Complementary bottlenecks", 26 November 2026 (verbatim)

*Relayed by Louis on 26 November 2026 and recorded at his request.*
- **What was reviewed:** the post's Markdown as merged in #920, before the rewrite of #923.
- **The review** follows between the rules, verbatim. Its eight figures arrived as screenshots of interactive mock-ups, and their text came through flattened in the paste. They are not committed as images: the **Figures, described for remaking** section after the review describes each one so it can be remade in the site's own style.
- **Names:** the review names the agents involved. The house rule keeps agent names out of research posts (`write-blog-post`), so the post credits "an independent review" and "the design analysis" instead.

---

I've read the [source of the blog post](https://github.com/lmmx/mibz2/blob/master/docs/site/blog/2026-11-26-complementary-bottlenecks.md?utm_source=chatgpt.com) against my original review, and I agree. The problem is not primarily that it has omitted the numerical results. It has actually preserved most of them, including several of the important caveats.

**The problem is that it has preserved the conclusions more faithfully than the reasoning that makes those conclusions interesting.**

My review was attempting to reconstruct the intellectual journey from a search for mathematical innovation, through the experimental characterisation of an entire computational system, to a substantially better-informed search for new algorithms. The blog reads more like an account of the matrix and its corrections.

It tells the reader what was found, but not sufficiently why it was worth finding, how it changes the space of possible algorithms, or what mathematical opportunities have now become visible.

I would ask the writer to make the following changes.

## **1. Restore the original intellectual motivation**

The introduction begins:

> The ideal-parts matrix made each part of mibz2's decoder free in turn, and in combination...

This starts with the experimental apparatus rather than the problem that motivated it.

The more interesting story is that you had already discovered a fundamentally different way of performing inverse BWT through interval-exchange induction, and wanted to know **where another mathematical discovery of that kind could produce a substantial improvement**.

You therefore needed to establish:

- Which computations still consume substantial resources.
- Which computations constrain the system's overall performance.
- Which constraints could be removed together.
- Whether a different mathematical representation could change the organisation of the decoder itself.

The experiment was conceived to guide discovery, not merely to improve performance attribution.

I would explicitly restore the original systems-biology inspiration, the connection to Raj Jain's experimental methodology, and the ambition to understand the decoder as an interacting network before attempting to redesign it.

The narrative ought to be:

**Mathematical ambition**

Discover a fundamentally cheaper way of reconstructing compressed information.

**System characterisation**

Understand the relationships between computation, concurrency, memory and elapsed time.

**Counterfactual experiment**

Measure what could be gained if particular computational obligations became cheap.

**Identify mathematical targets**

Locate expensive representations, dependencies and opportunities for shared work.

**Algorithmic invention**

Find a realisable computation that captures the measured opportunity.

Without this narrative, the reader cannot appreciate why a four-hour factorial experiment was worth undertaking in the first place.

## **2. Restore the hybrid dynamical systems framework**

The post reduces this to a single paragraph at the end of §1:

> between events, threads make progress at rates set by contention; at events, blocks are admitted, dispatched, finished and taken

This is a description of a hybrid system, but it omits **why representing the decoder as a hybrid system is useful**.

The interesting aspect of the framework was that it gave us a vocabulary for systematically reorganising computation. The repository's [structural levers document](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/levers.md?utm_source=chatgpt.com) contains considerably more interesting material than the blog has retained.

I would restore a version of this table:

| What can change                        | Example in mibz2                                                                              |
| -------------------------------------- | --------------------------------------------------------------------------------------------- |
| Flow rates                             | Make Huffman decoding or induction cheaper.                                                   |
| Conditions under which work progresses | Alter admission and memory constraints.                                                       |
| Conditions under which events happen   | Change when work can be dispatched.                                                           |
| Effects of events                      | Change which worker performs a computation and how it is routed.                              |
| Ownership of information               | Remove intermediate buffers and cross-thread transfers.                                       |
| The state space itself                 | Replace LF traversal with interval-exchange induction or another mathematical representation. |

The last of these is particularly important. It distinguishes making an existing computation faster from **changing what the computation consists of**.

The ideal-parts experiment principally investigates the first lever, pushing selected computational demands towards zero. But its results can motivate changes to any of the others.

That is a much more interesting conclusion than saying that the decoder can be described as a hybrid system.

## **3. Make the factorial design more intelligible**

Section 2 is accurate in broad terms, but it is too compressed for someone who doesn't already know experimental design.

The phrase:

> 32 forming a half fraction of resolution VI

is precisely the kind of terminology that should be unpacked visually.

The writer should explain why this is a particularly elegant design rather than merely stating how many combinations were tested.

### **How the 64 possible configurations were sampled**

0 ideal parts

1/1

1 ideal parts

6/6

2 ideal parts

15/15

3 ideal parts

0/20

4 ideal parts

15/15

5 ideal parts

6/6

6 ideal parts

1/1

Factorial fraction (32)

Added arms (12)

Omitted (20)

Configurations measured

# **44 / 64**

Pairwise interactions studied

# **15**

The reader should understand that the experiment was deliberately constructed to answer two distinct questions:

- What is the average effect of removing a part across many possible configurations?
- What is the directly observed effect of removing it from the ordinary decoder?

Those are not interchangeable quantities. This distinction explains the later difference between a stage's factorial importance and its standalone opportunity.

Also restore the qualification that higher-order interactions remain aliased. Resolution VI does not imply that all possible interactions have been independently identified.

## **4. Show the complete results, not just the headline pair**

The article is called *Reading the ideal-parts matrix*, but it doesn't actually present the complete six-part standalone-results matrix.

It presents the scanner–construction table and distributes the remaining results throughout prose.

I would add this as a central figure.

### **Elapsed-time reduction when each part is made ideal**

Positive values indicate a faster decode. Negative values indicate a slower one.

1 worker

6 workers

8 workers

10 workers

Scanner

-1.0%

2.6%

3.2%

8.1%

MTF / RLE2

7.5%

8.4%

2.5%

2.1%

**Construction**

**42.1%**

**24.6%**

5.6%

5.3%

Emission¹

**16.5%**

**14.9%**

3.5%

2.2%

RLE1 / CRC

1.7%

0.4%

-0.9%

-2.1%

Delivery

-1.1%

1.1%

0.8%

0.6%

¹ Emission oracle removes the graph walk, not all word expansion. Point estimates from the full matrix; confidence intervals and registered significance labels remain in the report.

This makes several important findings immediately apparent:

- Construction's importance declines with concurrency.
- Scanner importance increases.
- Emission has substantial hidden opportunity.
- MTF/RLE2 should not be dismissed.
- Relieving later stages alone is unlikely to be the next major breakthrough.

The article should then explain how this picture is incomplete without the pairwise interactions. That creates a natural reason to introduce the scanner–construction result, rather than simply declaring it the headline finding.

## **5. Explain how much the interaction changes the value of a discovery**

The current interaction figure is useful, but the prose doesn't develop its implications sufficiently.

The important question is not just how much faster the two oracles are together. It is how dramatically the expected value of a mathematical discovery changes depending on the rest of the system.

### **How much elapsed time can construction remove?**

Eight workers, using the measured oracle configurations

With the ordinary scanner

**5.6%**

With the scanner supplied

**33.9%**

Each percentage is relative to the corresponding starting configuration.

This should lead to a much stronger scientific observation:

**An unsuccessful whole-decoder benchmark is not necessarily evidence that an algorithmic improvement lacks value. It may be evidence that the system is unable to exploit that improvement in its current organisation.**

Conversely, the result also tells us how to unlock previously disappointing discoveries. Some mathematical approaches that produced excellent local results but little whole-system gain may deserve reconsideration after the scanner is restructured.

That opens up a research direction, rather than merely identifying two bottlenecks.

### **Correct the overstatement in the subtitle**

The subtitle currently says:

> At eight workers the scanner and construction must be relieved together

This contradicts the article's own subsequent qualification that other combinations can help.

I would change it to:

*At eight workers, relieving the scanner and construction together exposes a much larger opportunity than either alone. The experiment reveals where mathematical innovation could pay.*

The stronger universal claim is not established by the experiment.

## **6. Give the reader a model they can reason with**

The post has almost entirely removed the simple mathematical model that explains bottleneck migration.

I would restore it, ideally with an interactive figure. This would make the interaction result something the reader can understand and explore, rather than another statistic they need to remember.

For a simplified pipeline, the elapsed time for a large workload is approximately governed by the slower of two overlapping resource demands:

\\[ T\approx\max\left(S,\frac{W}{k}\right) \\]

where \\(S\\) is total scanner demand, \\(W\\) is total worker demand and \\(k\\) is the worker count.

### **Explore complementary bottlenecks**

Illustrative two-station model, not a fitted prediction of mibz2.

Scanner work reduction

**0%**

Worker work reduction

**0%**

Scanner

185.0 ms

Worker pool

185.0 ms

Model elapsed time

# **185.0 ms**

Overall reduction

# **0%**

Both resources are equally limiting.

Reset

Then introduce why this model eventually becomes inadequate: the memory system couples the two demands, meaning that reducing worker computation can also change scanner speed.

Antigravity's revised three-resource model is worth discussing as a hypothesis.

The experimental work becomes a progression of models, predictions, failures and refinements. At present, too much of that scientific development has been left in the preceding blog post.

## **7. Expand the most important section: where mathematics could help**

This is my biggest criticism of the article.

Section 5 is called *Where mathematics could help*, but it is essentially two dense paragraphs. Given that identifying mathematical opportunities was the entire purpose of the programme, this section should be one of the longest and richest parts.

The distinction between **contraction and enrichment** deserves separate subsections with concrete examples.

### **Contraction: changing the unit of computation**

The existing mathematical discovery replaces many individual row operations with an operation on an interval of rows that behave alike.

Schematic illustration of contraction, not measured row counts.

The future mathematical question is whether there is another useful equivalence relation between computations, perhaps involving nonadjacent rows, recurring transformations or structurally equivalent paths.

Explain why the rigid-interval closure result matters: it rules out a particular way of achieving compactness through composition, not the entire possibility of discovering a different representation.

**The blog omits this limitation of the negative theorem**, which was important to my review. Without it, the reader doesn't know why the research is searching outside the existing algorithmic family.

### **Enrichment: making existing work more productive**

The article should restore your original inspiration from automatic differentiation.

AD does not simply execute the same function faster. It lifts the computation into an algebra in which extra information is produced alongside the ordinary result.

For example, multiplication of dual numbers:

\\[ (a,a')(b,b')=(ab,a'b+ab'). \\]

The relevance to mibz2 is the question of whether an existing computation can cheaply produce additional information that eliminates a later expensive operation.

This is much more intellectually interesting than the generic formulation of making existing work yield information.

The writer should distinguish several possible sites for enrichment: Huffman decoding, MTF reconstruction, interval induction, and construction of the retained graph. It should also explain the known limitations on acquiring final LF information early, without mistaking those limitations for an impossibility result about deferred summaries.

The [essence document](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-24-algorithm-level/essence.md?utm_source=chatgpt.com) already has this motivation. Bring it into the article.

## **8. Develop the concrete algorithmic proposals properly**

The blog spends considerable space on the Rauzy schedule and then compresses several potentially more consequential proposals into three paragraphs.

This produces an unfortunate imbalance: the article's most developed mathematical opportunity is the one with a relatively modest estimated payoff.

I would make the concrete proposals individual subsections.

### **Proposal A: alternative induction schedules**

This section is already relatively strong. It preserves the arithmetic linking the reduction in induction steps to the potential elapsed-time saving.

However, the post should go further and quantify the relationship between the time spent in induction and the likelihood of reaching the payoff threshold.

### **The economics of a cheaper induction schedule**

Share of A6b that scales with step count

**65%**

Additional scheduling cost

**5 ms**

Estimated net saving

# **36.3 ms**

Whole-decode time saved

# **2.78%**

Progress towards the 39.2 ms research gate

93%

Does not meet the proposed payoff gate under these assumptions. Assumes the historical 29.4% step reduction transfers to Open and that reducible work scales proportionally.

This communicates why the next experiment exists much better than another explanation of the census.

### **Proposal B: eliminating intermediate representations**

The MTF/construction fusion deserves a diagram.

### **Existing computation**

### **Proposed direct fold**

Remove the intermediate capped-run representation where the fast path permits it.

This is an example of deforestation, eliminating an intermediate data structure between two transformations.

Antigravity estimates avoiding approximately 360 MB of intermediate read/write traffic over the repeated workload. Its proposed time saving is around 110–145 ms of one-worker work. Those are design estimates, not observed gains, but omitting them leaves the reader unable to compare this proposal against Rauzy scheduling.

### **Proposal C: joint construction and emission redesign**

This is perhaps the most interesting concrete proposal in Antigravity's report, yet the blog gives it only a few sentences.

The proposed change is from XOR-linked graph edges to forward-only links.

Conceptually:

\\[ \operatorname{link}(i) =\operatorname{prev}(i)\oplus\operatorname{next}(i) \\]

would become

\\[ \operatorname{link}(i)=\operatorname{next}(i). \\]

### **Existing: two-ended traversal**

### **Proposed: multiple forward traversals**

Conceptual cycle segments. The proposed implementation interleaves forward traversals from sampled anchors and must reconstruct their correct output order.

The attraction is that this changes several costs at once:

- Simplifies construction's graph wiring.
- Exposes additional independent memory lookups.
- Eliminates expensive backwards word-spelling operations.
- Potentially allows a more efficient final run-length consumer.

There is also actual mathematics here: the use of order statistics to determine how many anchor points to choose, since uniformly sampled starting points produce unequal arc lengths.

For a cycle of length \\(m\\) split at \\(K\\) uniformly sampled anchors, the continuous-spacing model gives approximately

\\[ \mathbb E[L\_{\max}]\approx m\frac{H_K}{K}. \\]

Here \\(H_K\\) is the \\(K\\)-th harmonic number.

That motivates the proposal to use more anchors than active traversal lanes, reducing the likelihood that one long segment determines completion.

The predicted benefit of the combined wiring and emission changes is approximately **280–340 ms of one-worker work**, subject to implementation costs, additional scratch memory, and the cost of identifying and assembling the correct cycle segments.

This is precisely the sort of mathematics-to-algorithm connection the blog ought to explore. See [Antigravity's second pass, proposals 1–3](https://github.com/lmmx/mibz2/blob/7fab624e/research/neon-movement/IDEAL-PARTS-2.md?utm_source=chatgpt.com).

### **Proposal D: reorganising the scanner**

Similarly, the article should make the distinction between deleting computation and redistributing it more prominent.

The scanner oracle deletes the Huffman work from the timed decode. A pooled scanner still performs it but changes its place in the computational network.

That distinction is a central example of the hybrid-system framework actually being useful.

It also explains why the project is not simply looking for a faster Huffman lookup routine. It is asking whether Huffman decoding needs to constitute a serial station at all.

## **9. Restore the distinction between three kinds of mathematical evidence**

The post uses words like *floor*, *budget*, *room*, and *forecast* without fully developing their relationships.

This is particularly important given the ongoing investigation into mathematical bounds.

| Quantity                 | What it establishes                                                                     |
| ------------------------ | --------------------------------------------------------------------------------------- |
| Mathematical lower bound | Under specified assumptions, a computation cannot cost less than this.                  |
| Oracle intervention      | An experimentally observed counterfactual when particular work is supplied or bypassed. |
| Performance predictor    | An estimate of what a concrete proposed algorithm would cost.                           |
| Proved upper bound       | Under specified assumptions, completion will take no longer than this.                  |

These are four quite different objects.

For example:

- \\(1.220r\\) induction steps concerns a mathematical class of operations.
- 125 ms concerns a measured experimental configuration.
- 130–145 ms concerns a proposed decoder architecture.
- 14–19 ms concerns an estimated residual in an artificial experimental pipeline.

They do not constitute competing estimates of the same quantity.

A section explaining how lower bounds prune the design space, oracle measurements guide research priorities, predictors rank candidates, and upper bounds establish guarantees would reconnect the experiment to the larger mathematical programme.

## **10. Specific technical and editorial corrections**

There are some narrower issues I would explicitly ask the writer to address.

| Location                       | Problem                                                                            | Correction                                                                                                                                                                                             |
| ------------------------------ | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Introduction                   | Says every part was made free.                                                     | Say the selected computations were replaced with recorded results. Some work and handover costs remained.                                                                                              |
| §2, factorial design           | Omits higher-order aliasing qualification.                                         | Explain which effects the fraction separates and which remain confounded.                                                                                                                              |
| §3, interaction figure caption | Says parallel lines mean the parts act independently.                              | On a linear elapsed-time plot, parallel lines indicate an absence of additive interaction. The registered factorial analysis is on log time, where the corresponding interpretation is multiplicative. |
| §3, interaction figure         | The four panels use different vertical scales without numerical axes.              | Label the scales explicitly, or normalise each panel to its all-real baseline.                                                                                                                         |
| §4, residual estimate          | Gives 14–19 ms without clearly identifying the relevant worker count.              | State that this is the registered one-worker Residual B prediction being tested.                                                                                                                       |
| §5, mathematical costs         | Describes the four currencies without naming them.                                 | Restore \\(Q,D,P,S\\), including the requirement to charge acquisition costs.                                                                                                                          |
| §6, Rauzy figure               | Calls 1.812 today's production count.                                              | Identify it as the historical right-end count for the full run-letter exchange.                                                                                                                        |
| §6, Rauzy formula              | Defines \\(f\_{\text{steps}}\\) as time spent on steps that the new order removes. | It should be the share of A6b whose cost scales with induction step count; otherwise multiplying by 29.4% double-counts the reduction.                                                                 |
| §6, proposal forecasts         | Gives insufficient quantitative comparison between proposals.                      | Restore estimated cost reductions, implementation costs, memory trade-offs and rejection criteria.                                                                                                     |
| Throughout                     | Attributes the discovery of the emission accounting problem to ChatGPT's review.   | Credit Antigravity's second pass for identifying the accounting issue; the review independently verified and clarified its consequences.                                                               |

I'd also remove much of the internal project-management vocabulary from the main narrative. PRs, numbered activities, stop tests and session names are useful for reproducibility, but they should support the scientific explanation rather than dominate it.

## **11. A better structure for the whole article**

I would restructure it rather than simply inserting more material into the existing sections.

| Section                                                  | Purpose                                                                                        |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| 1. Where could another mathematical breakthrough matter? | Establish the original research motivation.                                                    |
| 2. From an algorithm to a dynamical system               | Explain the systems-biology analogy, queueing model and structural levers.                     |
| 3. Measuring a counterfactual                            | Explain oracles, factorial design and why ordinary profiling was insufficient.                 |
| 4. A map of the remaining opportunity                    | Present the complete matrix and its interpretation.                                            |
| 5. Complementary bottlenecks                             | Develop the central scanner–construction result and bottleneck migration.                      |
| 6. What the experiment could not remove                  | Explain emission accounting, memory interference and the residual measurement.                 |
| 7. From measured opportunity to mathematical invention   | Develop contraction, enrichment, closure and the cost currencies.                              |
| 8. Four possible algorithms                              | Give the concrete proposals, diagrams, forecasts and uncertainties.                            |
| 9. From characterisation to construction                 | Explain why the evidence is sufficient to progress and what the remaining measurements decide. |

A more suitable title might also reflect the broader intellectual ambition.

My preferred title would be:

**Where mathematics can make decompression faster**

With the subtitle:

*Characterising the decoder as a computational system, measuring its counterfactual limits, and using the results to search for new algorithms.*

Other possibilities are *The design space of a decompressor*, *From bottlenecks to mathematical invention*, or *Characterising the limits of decompression*.

## **12. The overall instruction I would give the writer**

Revise the complementary-bottlenecks post substantially, rather than merely adding omitted facts.

The post currently preserves most of the review's conclusions but compresses the reasoning that made them valuable. Its subject should be the transition from mathematical invention, through system characterisation, to a more informed search for fundamentally cheaper algorithms.

Restore the systems-biology motivation, the hybrid dynamical systems framework and its structural levers, and the distinction between improving the rate of an existing computation and changing the representation or organisation of computation itself.

Explain the experimental design pedagogically, with a visual explanation of the 44 configurations, and show the complete six-part results matrix. Develop the scanner–construction interaction as evidence of how the value of an algorithmic discovery depends on the surrounding system.

Expand the mathematical opportunities substantially. Explain contraction, enrichment, closure, the scope of the existing negative results, and the Q/D/P/S cost model. Restore the automatic-differentiation inspiration for enrichment.

Give each major algorithmic proposal its own substantive subsection, with diagrams of the existing and proposed computation, the mathematical or structural mechanism, the estimated opportunity, the cost of obtaining it, and the evidence that would support or reject it. In particular, do justice to direct MTF-to-construction folding, joint forward wiring and multi-lane emission, and pooled cross-block scanning. Do not let the narrower Rauzy schedule question dominate.

Keep measured interventions, mathematical bounds, conditional estimates and untested design forecasts rigorously separate. Retain the corrections concerning emission's handover, memory contention and the transfer of historical induction counts to Open.

The article should be accessible without requiring the reader to have read every preceding blog or journal entry. Do not confuse accessibility with compression: retain the reasoning, unpack the terminology, and use concrete examples and diagrams to communicate the mathematical ideas.

The ending should explain what we can now do that we could not do before the experiment: pursue mathematical innovation against experimentally established opportunities, with a clear understanding of which other changes may be needed to realise the gain.

I reviewed the repository's Markdown source because the deployed GitHub Pages URL could not be retrieved directly. I therefore haven't assessed the actual browser rendering or mobile layout, and I have not modified the repository. The figures above are suggested editorial additions.

The biggest change I'd make is one of emphasis: **the matrix is the means, not the subject.** The subject is how to use a scientific understanding of a computational system to discover better algorithms.

That is what makes this work considerably more interesting than a conventional performance optimisation exercise.

---

## Figures, described for remaking (the Coordinator's, not part of the review)

The review came with eight figures, as screenshots of mock-ups. This section describes each one so it can be remade. Treat them as sketches of the content, not as finished designs: Louis found the post's earlier figures "rather basic", and these should be built with more care.
- **How to build them:** use the `dataviz` skill's procedure (form, then colour by job, the palette validator, mark specs, light and dark).
- **Format:** inline SVG in the site's style, with no committed screenshots.
- **The two interactive mock-ups (4 and 6) become static figures,** which show the whole response surface at once rather than one point a slider at a time, and which print, render on phones and survive without JavaScript. Louis questioned whether sliders were a good idea.

1. **How the 64 configurations were sampled** (§3 of the review). One row per number of ideal parts, 0 to 6. Each row has one square per configuration with that many parts ideal (1, 6, 15, 20, 15, 6, 1: the binomial row C(6, j)) and a count measured over possible at its right.
   - **Colours:** the 32 half-fraction configurations in one colour (rows 0, 2, 4 and 6 in full); the 12 added arms in a second (rows 1 and 5); the 20 omitted three-part configurations as empty outlines (row 3).
   - **Legend:** factorial fraction (32), added arms (12), omitted (20).
   - **Two stat tiles below:** "Configurations measured 44 / 64" and "Pairwise interactions studied 15".
   - **What it should teach:** the defining relation I = +SECMRD keeps the configurations with an even number of ideal parts. That is why rows 1, 3 and 5 are absent from the fraction, and why the single-ideal and all-but-one arms had to be added. The caption should say what the fraction separates (main effects and two-factor interactions, from each other) and what stays aliased (three-factor interactions in pairs).

2. **Elapsed-time reduction when each part is made ideal** (§4). A 6 × 4 table-heatmap: rows Scanner, MTF/RLE2, Construction, Emission¹, RLE1/CRC and Delivery; columns 1, 6, 8 and 10 workers. Cells are the percentage reduction (positive is faster), on a diverging scale: one hue for faster, another for slower, neutral at zero.
   - **Values:**

     | part | 1 worker | 6 workers | 8 workers | 10 workers |
     |---|---|---|---|---|
     | Scanner | −1.0 | 2.6 | 3.2 | 8.1 |
     | MTF/RLE2 | 7.5 | 8.4 | 2.5 | 2.1 |
     | Construction | **42.1** | **24.6** | 5.6 | 5.3 |
     | Emission¹ | **16.5** | **14.9** | 3.5 | 2.2 |
     | RLE1/CRC | 1.7 | 0.4 | −0.9 | −2.1 |
     | Delivery | −1.1 | 1.1 | 0.8 | 0.6 |

   - **Check before use:** every cell against `../results/full/report.txt` (1 − the "room" ratio).
   - **Footnote:** ¹ the emission oracle removes the graph walk, not the expansion of words into bytes. Point estimates; intervals and labels are in the report.
   - **Improvement:** mark each cell's registered label (ROOM, UNRESOLVED, NO ROOM) or interval width subtly, so a reader can see which values are resolved.

3. **How much elapsed time can construction remove?** (§5). Two horizontal bars at 8 workers: "With the ordinary scanner" 5.6%, and "With the scanner supplied" 33.9%. Caption: each percentage is relative to its own starting configuration.
   - **Improvement:** show the absolute walls too (195 → 184 ms; 189 → 125 ms), so the change of denominator is visible rather than only stated. Optionally show the same pair at 1, 6 and 10 workers as small multiples, so the reader sees the interaction appear and fade with the worker count.

4. **Complementary bottlenecks: the two-station model** (§6; the mock-up had two sliders).
   - **Make it static:** a 2-D heatmap or contour plot of the model T = max(S(1 − a), W(1 − b)/k).
     - **Axes:** scanner work reduction a, 0–100% on x; worker work reduction b, 0–100% on y.
     - **Starting point:** S = W/k = 185 ms, the mock-up's illustrative balance.
     - **Colour:** elapsed time, or the overall reduction.
   - **What it shows:** the contour lines are L-shaped, which shows the complementarity at a glance. Moving along either axis alone gains nothing, while moving diagonally gains everything. Mark the diagonal a = b, and the measured 8-worker points (all real, S supplied, C supplied, both) as reference markers, labelled as measurements against an illustrative model.
   - **Caption:** "illustrative two-station model, not a fitted prediction". The text then says why it breaks: memory couples the stations, the three-resource hypothesis.

5. **Contraction, schematically** (§7).
   - **Top row,** "Individual LF operations": ten small boxes, each with its own downward arrow.
   - **Middle row,** "Shared operations on intervals": three wide boxes covering the same span, each with one arrow.
   - **Below,** a dashed box: "The next research question: what else can be treated as one computation?"
   - **Caption:** schematic, not measured row counts.
   - **Improvement:** make the rows correspond, with the wide boxes visibly grouping the small ones above them, and show the retained rows of Open as the boundaries between intervals.

6. **The economics of a cheaper induction schedule** (§8, proposal A; the mock-up had sliders).
   - **Make it static:** a line chart.
     - **x:** the share of A6b whose cost scales with the step count, 0–100%;
     - **y:** the net saving in ms at 1 worker, equal to 216 × share × 0.294 − overhead;
     - **lines:** scheduling overhead of 0, 5 and 10 ms;
     - **a horizontal line** at the 39.2 ms gate (3% of 1,306 ms).
   - **The crossings:** about 62% of A6b (no overhead), 70% (5 ms) and 77% (10 ms).
   - **Caption assumptions:** the historical 29.4% step cut transfers to Open's object, and the reducible work scales in proportion.
   - **The mock-up's example point** (65%, 5 ms → 36.3 ms, 2.78%, 93% of the gate) can be marked on the 5 ms line.

7. **Proposal B, the direct fold** (§8).
   - **"Existing computation":** three boxes in a row, MTF/RLE2 (decode) → Capped runs (materialise) → Construction (recombine).
   - **"Proposed direct fold":** two boxes, MTF/RLE2 (maximal runs) → Construction (direct accumulation).
   - **Caption:** remove the intermediate capped-run representation where the fast path permits it.
   - **Improvement:** annotate the removed box with what it costs (the materialised records and the second pass), and state the design analysis's estimate of the traffic avoided as a forecast.

8. **Proposal C, two-ended against multi-lane forward traversal** (§8).
   - **"Existing: two-ended traversal":** a row of twelve boxes (one cycle), with a forward arrow from the left over the first half and a backward arrow from the right over the second.
   - **"Proposed: multiple forward traversals":** the same twelve boxes split into four coloured segments of three, each with an anchor dot at its start and a forward arrow.
   - **Caption:** conceptual cycle segments. The implementation interleaves forward traversals from sampled anchors and must restore their output order.
   - **Improvement:** draw the cycle as a cycle (a ring), with K anchors at uneven, uniformly sampled positions. The uneven arcs then show why E[L_max] ≈ m·H_K/K, and why K is taken larger than the number of lanes.

**Also wanted, from the review's text, without mock-ups:**
- the structural-levers table as a figure or a well-set table (§2);
- the four kinds of evidence (lower bound, oracle, predictor, upper bound) placed on one axis or in one table, with the four example figures (1.220 r, 125 ms, 130–145 ms, 14–19 ms) shown as different kinds of object (§9);
- a diagram for proposal D: the serial scanner station against scanning folded into each worker, the same work redistributed rather than deleted.
