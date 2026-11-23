[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/index.md>)

# Research by obligation: modelling the work, not the algorithms

We now track the work any decoder must pay for rather than the algorithms that pay it, and a claimed Riemann proof has sharpened how we hunt.

20 November 2026 · 15 min read · [Meta](<#7-where-this-leaves-us>)

**What changed.** mibz2's biggest speed-ups did not come from tuning code. They came from changing what the decoder computes: walking a block's rows one by one gave way to [rearranging whole intervals of rows at once](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), and that gave way to [keeping a tenth of the rows and letting the rest travel in groups](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-19-open-returns/index.md>). This week we set out to find the next change of that size, and spent much of it learning how *not* to search for one. The result is a written research method, the [research charter](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md>), whose central idea is simple: keep a register of the **obligations**, the kinds of work every decoder we have built must pay for, and judge every new idea by which obligation it removes and what it pays instead.

**Why obligations, not algorithms.** An algorithm is a bundle. It pays for several obligations at once, so when it loses, it is hard to say what lost. An obligation is the part that decomposes: "follow the retained rows one at a time", "build the map that says where each retained row goes next". Candidates come and go; an obligation stays open until something removes it or a proof bounds it. Six candidate improvements to the current kernel lost this week, and every one of them left the register knowing more than before.

**What happened today.** Two things sharpened the method after it was written. A measurement traced the expensive work back to the text it produces, and [found it sitting in random identifiers](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-20-where-the-decoding-work-goes/index.md>). And a mathematician announced a claimed proof of the Riemann Hypothesis, found with an AI system, whose account of *how* it was found gave us a better way to look for new ideas. [Section 1](<#1-the-obligation-register>) explains the register and [section 2](<#2-generate-from-structure-judge-against-obligations>) the cycle that uses it. [Section 3](<#3-five-research-cycles-read-through-the-register>) reads this week's five research cycles through the register, [section 4](<#4-where-ideas-come-from-the-leads>) describes where new ideas come from, and [section 5](<#5-working-backwards-from-the-text>) what working backwards from the text showed. [Section 6](<#6-adapting-the-method-toy-missing-effect-native-surrogate>) is the adaptation we made today, with a worked toy and a measurement of how to break it.

## 1\. The obligation register

A short recap of the setting. bzip2 stores each block of text as the last column of a sorted table of the block's rotations. Rebuilding the text means following a path through the table's rows, one byte per step. Equal bytes on neighbouring rows form a **run**, and rows in a wide run move together. mibz2's current kernel, Open, treats wide runs as single units, keeps the rows of narrow runs (about a tenth of all rows), and walks those one at a time.

Profiling Open splits its time into a handful of jobs, and each job is an obligation: something this family of decoders cannot avoid paying for. The register lists them with their measured cost, and beside each, the candidates that tried to remove it:

The obligations, and the candidates they outlived

The obligations are from the [register](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/FRONTIER.md>); the shares of Open's time are measured (\#653). Each grey box is a candidate rejected on native timing or by a decisive count, except for overlaps, where it is the known, costly route.

Reading across a row says what was learned about that obligation. Three candidates attacked the walk over retained rows: a software-pipelined walk (the processor already issues the next load early), a compact rank structure in place of the stored edges (two to three times slower, because one load became four dependent ones), and several walks at once (no gain in our test harness). Three attacked the build: fusing passes, compressing queues, and building the map backwards. The backward build was the most instructive. It removed the queues it targeted, but its second pass over the run records cost about as much as the queues had.

The register also keeps an object apart from the route to it. One cycle found that the structure of where runs overlap is small: about 1.3 transitions per run on our blocks. The only known way to obtain it costs about 1.8 steps per run, which loses. Those are two findings, and rejecting the route must not erase the object: a cheaper way to the same small structure would still be worth having.

When a failure exposes a real obstruction, the register records three things: the concrete **witness**, the exact **scope** (which algorithms and cost model it binds), and the **escape condition**, the assumption a different formulation would have to drop. An obstruction without a stated scope does not count. That rule matters, because the temptation after six losses is to conclude that the walk is a law of nature. It isn't. It is a law of the representations tried so far.

## 2\. Generate from structure, judge against obligations

The register is for judging, not for generating. Early in the week it was used for both, and the ideas it produced were rephrasings of the costs in its own rows: "make the queue smaller", "make the walk shorter". So the method now separates the two, inside a cycle of five phases:

The research cycle

Conceptual. Ideas come from mathematics; the register decides which survive.

**Explore** generates from mathematical structure. The charter names four routes, as tools rather than quotas. *Representation import* asks what richer object this is an instance of: both earlier breakthroughs came this way, from seeing the row map as an interval exchange. *Obstruction inversion* takes a failure's narrowest witness and asks which assumption another primitive could escape. *Primitive ablation* forbids a dominant operation and checks the cost doesn't reappear elsewhere. *Small-instance discovery* finds the best exact construction on tiny inputs and looks for the law behind it.

**Triage, Deepen, Confront and Learn** judge. Every candidate must say which paid work disappears, what replaces it, and how the information it needs is obtained: we call this *acquisition first*, because several elegant ideas died on the cost of getting their inputs rather than using them. Every cost model is treated as a diagnostic. This is not caution for its own sake. A cache model ranked two of Open's variants the wrong way round, predicting a win for the one that runs 10–16% slower natively. None of the six candidates that passed a model passed native timing.

The cycle asks two separate questions of every candidate. Has the primitive changed, or only how the same operations run? We call the first *reorientation* and the second *traction*: Rauzy–Veech induction was reorientation, replacing single steps with operations on whole intervals, while the backward build was traction on Open's existing return map. And has the complete cost fallen? A new representation is not automatically a cheaper computation. The compact rank structure did change the primitive, removing the stored edges entirely, and lost because each step became several dependent lookups.

Two smaller rules hold the process together. A research session pushes its work at every handoff and at least every half hour. One session was cut off mid-cycle by a usage limit, and its earlier cycles survived only because they were already published. And new mathematical leads go into an inbox that the researcher reads at a cycle boundary, so a good idea from outside never derails an experiment halfway through.

## 3\. Five research cycles, read through the register

The register earns its place when a cycle fails, because a failure read against it says which obligation resisted and why. A research model worked through five cycles this week under the charter, after a run of earlier candidates that prompted it. The table lists them all, with the obligation each attacked and what the attempt left behind.

| Attempt | Obligation it attacked | Outcome | What it left in the register |
| --- | --- | --- | --- |
| Precomputed jumps per run record (\#652) | following retained rows | lost | building the jumps costs about one whole Open |
| Fused passes; compressed queues (\#654) | building the map | lost | the expansion to one entry per retained row is needed, and cheapest done in one sequential fill |
| Pipelined emit (\#656) | following retained rows | not adopted | the processor already issues the next load early |
| Tuning the thin/thick limit (\#657) | both | not adopted | 16 rows is best; choosing per block gains at most 1%, even with hindsight |
| Cycle 1: contracting the exchange from both ends | metadata; overlaps | lost on cost | the overlap structure is small (1.25–1.37 transitions per run); grammar machines pay 40–100 bytes per run |
| Cycle 2: building Open's map backwards (\#659) | building the map | 1.10–1.15× slower natively | the second pass over run records costs about what the queues did |
| Cycle 3: bucketed streaming exchange | building the map | lost (reported) | a claimed limit on keeping the exchange in the last-level cache; not yet audited |
| Cycle 4: contracting an unsplit forest | both | lost (reported) | claimed structure theorems about the run graph; not yet audited |
| Cycle 5: implicit rank in place of stored edges | following retained rows | 1.7–2.8× slower | one dependent load became four; memory fell to 0.36–0.49× |

*Measured where a PR is cited or a ratio is given; cycles 3 and 4 are as the research model reported them, and are still to be checked.*

Two patterns run through the table. Representations that are expensive to build lose on construction; representations that are compact lose on consumption, because recovering each next step takes several dependent operations instead of one load. The fifth cycle shows both sides at once: it cut peak memory to under half and modelled memory traffic by a quarter or more, and still ran two to three times slower, because a smaller description is not a cheaper computation. Neither pattern is a lower bound for the problem. Both are evidence about the families tried, recorded with their scope so the next idea can aim at the assumption that bound them.

The table also shows what the cost models were worth. Every candidate that reached native timing had first passed a model, and every one lost. The register now records each model's track record beside the candidate it scored, in a *fidelity ledger*, so a model that has mis-ranked before is read with that history in view.

## 4\. Where ideas come from: the leads

If the register judges, something else has to generate. The charter keeps an inbox of mathematical leads: directions from outside the cycle, which the researcher reads at a cycle boundary and is free to follow or set aside. Three came out of this week, each from asking what richer object a block's runs might be.

**A square-tiled surface.** Pair the row map with the plain step to the next row, and the block becomes a surface glued from unit squares, a branched cover of a torus. Its singular points sit exactly where runs begin and end, and on our six blocks there are only one to eight of them. That is a striking amount of structure, but it forgets the runs' lengths, so it cannot decode a block by itself. The open question is whether directions on the surface, and how things move along them, recover what it forgets ([journal](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-origami-lift.md>)).

**A train track.** Draw the runs as a graph, with an arc wherever LF carries one run's rows into another. The arcs number exactly \\(2r - 1 - C\\), and 88–89% of runs simply pass their rows on to one other run, leaving a branching core of about a tenth of the runs. With the order of strands kept at each junction, this is a measured train track, the object Thurston used to carry tangles of curves on a few weighted branches, and whose basic move is the same as the induction behind our earlier kernel ([the train-tracks post](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-20-train-tracks/index.md>)). A census found no small separators in the core, and as a set of retained rows it does worse than Open's.

**Interval pairings.** Agol, Hass and Thurston gave an algorithm that works on huge implicit families of strands through intervals paired by translation, without walking the strands. A block's runs have exactly that shape. Their algorithm counts components, not order; the lead asks whether a version that carries words could recover the text more cheaply.

None of these is a decoder yet, and the charter does not ask them to be. They are where a change of primitive might come from. The register is what will say whether one does.

## 5\. Working backwards from the text

The register describes costs in the decoder's terms: rows, runs, passes. Today we asked the same question in the text's terms, prompted by a contrast with the Hutter Prize, which rewards the smallest self-extracting description of Wikipedia and lets decompression take tens of hours. mibz2 sits at the other end: its description is fixed by bzip2, and the goal is the least computation to recover the text. One Hutter winner, STARLIT, won by reordering Wikipedia's articles so similar ones sat together. bzip2 has already done a reordering of its own, sorting every byte by what follows it, and we asked what that exposes.

Tracing every expensive row back to the byte it spells answered it. The repetitive JSON skeleton is already nearly free: keys are about a third of the text and about 1% of the rows that stand alone. Unique random statement IDs, about 5% of the text, hold about 62% of them. Hashes that are reused form small runs, one row per copy. So the obligation "follow retained rows one by one" is, in large part, the obligation to spell random strings ([the post](<https://lmmx.github.io/mibz2/pr-preview/pr-778/blog/2026-11-20-where-the-decoding-work-goes/index.md>)).

That reshaped an idea we had been circling: a zoo. Rather than one universal decoder, a repertoire of exact methods, each very good on one kind of structure, with the decoder recognising which applies. Repeated random strings might be copied once one copy is decoded; unique ones might be a true floor for any method that relies on repetition. Recognition has to be cheap, read from what the compressed block already gives us, and the first number to know is how much a perfect choice of method could save. An earlier attempt to tune Open per block found under 1% to gain, which is why that number comes first.

## 6\. Adapting the method: toy, missing effect, native surrogate

Today Levent Alpöge [announced](<https://alpo.ge/remark.pdf>) that he believes he has proved the Riemann Hypothesis, and its generalisation, working with Claude, with the argument formalised in Lean. Nothing here depends on it being right. (*Update, 21 November:* the same day, members of the Lean community reported that three independent proof checkers, nanoda, con-leche in its verified mode and pestka, each accepted the Lean development, about 17 million lines, as proving the Riemann Hypothesis and the Generalised Riemann Hypothesis for Dirichlet L-functions. We have not checked it ourselves, and mathematicians are still digesting the argument.) What interested us is his account of how the argument was found, because it describes a way of searching that our method lacked.

As he tells it, he started from a *toy* setting where the theorem was already known (the Riemann Hypothesis over function fields) and re-proved it in a new way chosen so the proof might travel. He then named the ingredient that the hard setting lacks: Stepanov's classical argument uses *multiplicity*, a polynomial vanishing to high order, which over the integers would be "something like differentiating whole numbers". The decisive step was to find a different mechanism, native to the hard setting, with the same effect: chains of theta lifts, each standing in for one order of multiplicity. The human held the analogy; the AI system supplied the technical mastery to build it.

We have a toy setting of our own. When a block's last column has exactly one run per letter, its text is the coding of an especially simple interval exchange; Ferenczi and Zamboni proved that these "perfectly clustering" words are exactly the codings of symmetric interval exchanges, the structure our earlier kernels rest on. With two letters they are the Christoffel and Sturmian words, and their text needs no walk at all.

Take the slope \\(3/8\\). Letter \\(k\\) of the word is \\(b\\) when \\(\\lfloor 3(k+1)/8 \\rfloor - \\lfloor 3k/8 \\rfloor = 1\\) and \\(a\\) otherwise, which for \\(k = 0, \\dots, 7\\) gives `aabaabab`. Sort its eight rotations and read the last column: `bbbaaaaa`, two runs, the smallest a column can have. A decoder that knew the slope would not need the column at all. It could write any letter of the text directly from its position, in any order, with no step depending on the one before. For an irrational slope \\(\\alpha\\) the same holds with \\(\\lfloor (k+1)\\alpha \\rfloor - \\lfloor k\\alpha \\rfloor\\): every output position is computed, not reached.

Toy, damaged toy, real block

Conceptual. The middle panel's measurements are in the table below.

The first version of our transfer went wrong in an instructive way. We named the missing ingredient as run width, since the toy has two enormous runs and the random IDs have none, and tested an obvious surrogate the same evening: walking many independent stretches of the path at once, as mibz2's plain decoder already does. It gave no gain on Open's walk, and more to the point, it was the wrong test. Running walks side by side still follows the same edges; it is a speed-up of the old operation, not a new one.

The correction, which came out of the discussion, is that the toy's real power is not width but **cheap global coordinates**: a compact number (\\(k\\alpha\\) modulo 1) from which any position's letter follows. In the toy the two coincide, which is why they were easy to confuse.

They come apart if the toy is damaged on purpose. Keep the rotation by \\(\\alpha\\) and the two letters, but cut the circle into \\(p\\) pieces and give each piece the letter \\(a\\) or \\(b\\) at random. The coordinate survives untouched: letter \\(k\\) is the letter of the piece holding \\(x + k\\alpha\\) modulo 1, found by one binary search over the cut points, for any \\(k\\), in any order. What changes is the column. We measured it on texts of 5,000 letters with the golden-ratio slope:

| Pieces \\(p\\) | Runs in the last column | Mean run width (rows) |
| ---: | ---: | ---: |
| 2 | 26 | 192 |
| 8 | 44 | 114 |
| 64 | 218 | 23 |
| 256 | 802 | 6.2 |
| 1,024 | 2,018 | 2.5 |

*Measured ([`toy2.py`](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-20-toy-transfer/toy2.py>), two letters throughout).*

*Correction, 20 November 2026:* the first version of this table gave each piece its own letter, so at 1,024 pieces it had a 1,024-letter alphabet and its run count mostly counted letters. The research line caught it. With two letters the effect is stronger, not weaker.

With a few pieces the column stays wide, so a small cut barely damages the toy. With a thousand pieces the runs average two and a half rows, narrower than the random strings in our real blocks, and the coordinate still answers any position in one search. So a block can be almost all narrow runs and still have a global coordinate that makes the walk unnecessary. Width-based methods, ours included, would find almost nothing to exploit in it.

There is a limit, which the research line also proved. Every text coded from a rotation has zero entropy: a handful of numbers describe it completely. Real blocks are not like that. A block with about 760 random statement IDs holds about 90,000 random bits, and no global coordinate can be smaller than the information it determines. So on real data the coordinate we are looking for cannot be a single number like \\(k\\alpha\\). At best it describes the structured part, and the random strings must be paid for some other way.

That is the precise form of the question for the research line: what object, native to real blocks and cheap to read off the compressed input, gives coordinates the way \\(k\\alpha\\) does, or lets long stretches of the path be composed without walking them? Part of the answer may already exist in outline. For a rotation, the digits of a position in the Ostrowski numeration, built from the continued fraction of \\(\\alpha\\), say where it sits at every scale. For a general interval exchange, the induction that our earlier kernel performs builds a comparable hierarchy, and a position's path down that hierarchy is its coordinate. Our kernels pay to build the whole hierarchy and then write the text from it. Whether a position's coordinate can be read off without paying for the whole build is open, and it is exactly the kind of question the register exists to price.

## 7\. Where this leaves us

Three pieces now fit together. The obligation register tells us what our present algorithms pay for. Importing mathematical structure tells us what other operations might exist. Toy-to-hard transfer tells us which missing effects those operations should try to reproduce.

The method in one chain: model the obligations, because they decompose and algorithms don't; generate ideas from mathematical structure; judge them against the obligations, with acquisition counted and models treated as diagnostics; and when a phenomenon makes the problem easy somewhere, find what makes it easy, break it on purpose, and look for a different mechanism that recreates the effect where it is missing.

The method has not yet been shown to produce breakthroughs. Five research cycles under it found real structure and no faster decoder. What it has shown is narrower: failed approaches are cheaper to reject, their hidden costs are easier to see, and the mathematics they leave behind is harder to lose.

A new research line is running now, independently, with the geometric leads (square-tiled surfaces, train tracks, interval pairings) as starting points and the toy-transfer lead waiting at its next cycle boundary. Its results come back for independent verification before anything reaches the decoder.

---

**Research records:**

- [The research charter: method](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md>) and [frontier and obligation register](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/FRONTIER.md>).
- [Toy, missing ingredient, native surrogate](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-toy-transfer.md>), with the correction and the lanes experiment.
- [Working backwards from the text](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-working-backwards.md>).
- Levent Alpöge, [Remark](<https://alpo.ge/remark.pdf>) (20 November 2026), announcing the proof. On 20 November its Lean formalisation was reported accepted by three independent kernels (nanoda, con-leche `--verified`, pestka) in the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis".
- S. Ferenczi and L. Q. Zamboni, *Clustering words and interval exchanges* (2013); S. Mantaci, A. Restivo and M. Sciortino, *Burrows–Wheeler transform and Sturmian words* (2003).
