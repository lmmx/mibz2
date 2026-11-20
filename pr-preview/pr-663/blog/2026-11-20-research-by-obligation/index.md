[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-663/blog/index.md>)

# Research by obligation: modelling the work, not the algorithms

We now track the work any decoder must pay for rather than the algorithms that pay it, and a claimed Riemann proof has sharpened how we hunt.

20 November 2026 · 11 min read · [Meta](<#5-where-this-leaves-us>)

**What changed.** mibz2's biggest speed-ups did not come from tuning code. They came from changing what the decoder computes: walking a block's rows one by one gave way to [rearranging whole intervals of rows at once](<https://lmmx.github.io/mibz2/pr-preview/pr-663/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), and that gave way to [keeping a tenth of the rows and letting the rest travel in groups](<https://lmmx.github.io/mibz2/pr-preview/pr-663/blog/2026-11-19-open-returns/index.md>). This week we set out to find the next change of that size, and spent much of it learning how *not* to search for one. The result is a written research method, the [research charter](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md>), whose central idea is simple: keep a register of the **obligations**, the kinds of work every decoder we have built must pay for, and judge every new idea by which obligation it removes and what it pays instead.

**Why obligations, not algorithms.** An algorithm is a bundle. It pays for several obligations at once, so when it loses, it is hard to say what lost. An obligation is the part that decomposes: "follow the retained rows one at a time", "build the map that says where each retained row goes next". Candidates come and go; an obligation stays open until something removes it or a proof bounds it. Six candidate improvements to the current kernel lost this week, and every one of them left the register knowing more than before.

**What happened today.** Two things sharpened the method after it was written. A measurement traced the expensive work back to the text it produces, and [found it sitting in random identifiers](<https://lmmx.github.io/mibz2/pr-preview/pr-663/blog/2026-11-20-where-the-decoding-work-goes/index.md>). And a mathematician announced a claimed proof of the Riemann Hypothesis, found with an AI system, whose account of *how* it was found gave us a better way to look for new ideas. [Section 1](<#1-the-obligation-register>) explains the register and [section 2](<#2-generate-from-structure-judge-against-obligations>) the cycle that uses it. [Section 3](<#3-what-the-register-has-taught-us>) says what the register taught us this week, and [section 4](<#4-adapting-the-method-toy-missing-effect-native-surrogate>) the adaptation we made today.

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

## 3\. What the register has taught us

Five research cycles by one model, and a sixth line now running with another, have mapped the walls fairly precisely. Representations that are expensive to build lose on construction. Representations that are compact lose on consumption, because recovering each next step takes several dependent operations instead of one load. Neither is a lower bound for the problem; both are evidence about the families tried.

Today's measurement added the most useful single fact. Tracing every expensive row back to the byte it spells showed that the repetitive JSON skeleton is already nearly free, while unique random statement IDs, about 5% of the text, hold about 62% of the rows that stand alone. The obligation "follow retained rows one by one" is, in large part, the obligation to spell random strings. That turned a vague idea into specific questions, written up in [the previous post](<https://lmmx.github.io/mibz2/pr-preview/pr-663/blog/2026-11-20-where-the-decoding-work-goes/index.md>). The idea is a zoo: rather than one universal decoder, a repertoire of exact methods, each very good on one kind of structure, with the decoder recognising which applies. Recognition has to be cheap, read from what the compressed block already gives us, or it costs more than it saves.

## 4\. Adapting the method: toy, missing effect, native surrogate

Today Levent Alpöge [announced](<https://alpo.ge/remark.pdf>) that he believes he has proved the Riemann Hypothesis, and its generalisation, working with Claude, with the argument formalised in Lean. The claim is new and has not yet been independently checked, and nothing here depends on it being right. What interested us is his account of how the argument was found, because it describes a way of searching that our method lacked.

As he tells it, he started from a *toy* setting where the theorem was already known (the Riemann Hypothesis over function fields) and re-proved it in a new way chosen so the proof might travel. He then named the ingredient that the hard setting lacks: Stepanov's classical argument uses *multiplicity*, a polynomial vanishing to high order, which over the integers would be "something like differentiating whole numbers". The decisive step was to find a different mechanism, native to the hard setting, with the same effect: chains of theta lifts, each standing in for one order of multiplicity. The human held the analogy; the AI system supplied the technical mastery to build it.

We have a toy setting of our own. When a block's last column has exactly one run per letter, the text is the coding of an especially simple interval exchange. With two letters these are the Sturmian words, and their text needs no walk at all: letter \\(k\\) is \\(\\lfloor (k+1)\\alpha \\rfloor - \\lfloor k\\alpha \\rfloor\\) for a fixed slope \\(\\alpha\\). Every output position is computed, not reached.

Toy, damaged toy, real block

Conceptual. The middle panel is a suggested experiment, not yet run.

The first version of our transfer went wrong in an instructive way. We named the missing ingredient as run width, since the toy has two enormous runs and the random IDs have none, and tested an obvious surrogate the same evening: walking many independent stretches of the path at once, as mibz2's plain decoder already does. It gave no gain on Open's walk, and more to the point, it was the wrong test. Running walks side by side still follows the same edges; it is a speed-up of the old operation, not a new one.

The correction, which came out of the discussion, is that the toy's real power is not width but **cheap global coordinates**: a compact number (\\(k\\alpha\\) modulo 1) from which any position's letter follows. In the toy the two coincide, which is why they were easy to confuse. They come apart if the toy is damaged on purpose: code a rotation by a circle cut into many pieces, and the coordinate \\(x + k\\alpha\\) survives while the runs multiply. So the question for the research line is now precise: what object, native to real blocks and cheap to read off the compressed input, gives coordinates the way \\(k\\alpha\\) does, or lets long stretches of the path be composed without walking them? Mathematics that might answer it already exists in outline (the numeration systems that generalise continued fractions to interval exchanges), and so does the discipline to test it: the register asks what it would remove and what it would cost to acquire.

## 5\. Where this leaves us

Three pieces now fit together. The obligation register tells us what our present algorithms pay for. Importing mathematical structure tells us what other operations might exist. Toy-to-hard transfer tells us which missing effects those operations should try to reproduce.

The method in one chain: model the obligations, because they decompose and algorithms don't; generate ideas from mathematical structure; judge them against the obligations, with acquisition counted and models treated as diagnostics; and when a phenomenon makes the problem easy somewhere, find what makes it easy, break it on purpose, and look for a different mechanism that recreates the effect where it is missing.

The method has not yet been shown to produce breakthroughs. Five research cycles under it found real structure and no faster decoder. What it has shown is narrower: failed approaches are cheaper to reject, their hidden costs are easier to see, and the mathematics they leave behind is harder to lose.

A new research line is running now, independently, with the geometric leads (square-tiled surfaces, train tracks, interval pairings) as starting points and the toy-transfer lead waiting at its next cycle boundary. Its results come back for independent verification before anything reaches the decoder.

---

**Research records:**

- [The research charter: method](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md>) and [frontier and obligation register](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/FRONTIER.md>).
- [Toy, missing ingredient, native surrogate](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-toy-transfer.md>), with the correction and the lanes experiment.
- [Working backwards from the text](<https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-working-backwards.md>).
- Levent Alpöge, [Remark](<https://alpo.ge/remark.pdf>) (20 November 2026), announcing a claimed proof, not yet independently verified.
- S. Ferenczi and L. Q. Zamboni, *Clustering words and interval exchanges* (2013); S. Mantaci, A. Restivo and M. Sciortino, *Burrows–Wheeler transform and Sturmian words* (2003).
