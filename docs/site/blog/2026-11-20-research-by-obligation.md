---
title: "Research by obligation: modelling the work, not the algorithms"
image: assets/social/blog-2026-11-20-research-by-obligation.png
description: "We now track the work any decoder must pay for rather than the algorithms that pay it, and a claimed Riemann proof has sharpened how we hunt."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Research by obligation: modelling the work, not the algorithms

We now track the work any decoder must pay for rather than the algorithms that pay it, and a claimed Riemann proof has sharpened how we hunt.
{: .post-dek }

20 November 2026 · 10 min read · [Meta](#5-where-this-leaves-us)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

**What changed.** mibz2's biggest speed-ups did not come from tuning code. They came from changing what the decoder computes: walking a block's rows one by one gave way to [rearranging whole intervals of rows at once](2026-11-10-rauzy-veech-reconstruction.md), and that gave way to [keeping a tenth of the rows and letting the rest travel in groups](2026-11-19-open-returns.md). This week we set out to find the next change of that size, and spent much of it learning how *not* to search for one. The result is a written research method, the [research charter](https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md), whose central idea is simple: keep a register of the **obligations**, the kinds of work every decoder we have built must pay for, and judge every new idea by which obligation it removes and what it pays instead.

**Why obligations, not algorithms.** An algorithm is a bundle. It pays for several obligations at once, so when it loses, it is hard to say what lost. An obligation is the part that decomposes: "follow the retained rows one at a time", "build the map that says where each retained row goes next". Candidates come and go; an obligation stays open until something removes it or a proof bounds it. Six candidate improvements to the current kernel lost this week, and every one of them left the register knowing more than before.

**What happened today.** Two things sharpened the method after it was written. A measurement traced the expensive work back to the text it produces, and [found it sitting in random identifiers](2026-11-20-where-the-decoding-work-goes.md). And a mathematician announced a claimed proof of the Riemann Hypothesis, found with an AI system, whose account of *how* it was found gave us a better way to look for new ideas. [Section 1](#1-the-obligation-register) explains the register and [section 2](#2-generate-from-structure-judge-against-obligations) the cycle that uses it. [Section 3](#3-what-the-register-has-taught-us) says what the register taught us this week, and [section 4](#4-adapting-the-method-toy-missing-effect-native-surrogate) the adaptation we made today.

## 1. The obligation register

A short recap of the setting. bzip2 stores each block of text as the last column of a sorted table of the block's rotations. Rebuilding the text means following a path through the table's rows, one byte per step. Equal bytes on neighbouring rows form a **run**, and rows in a wide run move together. mibz2's current kernel, Open, treats wide runs as single units, keeps the rows of narrow runs (about a tenth of all rows), and walks those one at a time.

Profiling Open splits its time into a handful of jobs, and each job is an obligation: something this family of decoders cannot avoid paying for. The register lists them with their measured cost, and beside each, the candidates that tried to remove it:

<figure class="bf-fig">
<p class="bf-title">The obligations, and the candidates they outlived</p>
<svg viewBox="0 0 720 372" role="img" aria-label="Six obligations on the left, each with the rejected candidates that tried to remove it on the right: the obligations stayed while the candidates were killed."><rect class="bf-card-hi" x="10" y="20" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="40">Following retained rows one by one</text><text class="bf-tm" x="22" y="57">36–48% of Open's time</text><line class="bf-rule" x1="310" y1="43" x2="340" y2="43"/><rect class="bf-slot" x="340" y="24" width="120" height="38" rx="5"/><text class="bf-ttiny" x="346" y="39">Pipelined emit</text><text class="bf-ttiny" x="346" y="54">#656</text><line class="bf-rule" x1="310" y1="43" x2="466" y2="43"/><rect class="bf-slot" x="466" y="24" width="120" height="38" rx="5"/><text class="bf-ttiny" x="472" y="39">Bit-sliced rank</text><text class="bf-ttiny" x="472" y="54">2–3× slower</text><line class="bf-rule" x1="310" y1="43" x2="592" y2="43"/><rect class="bf-slot" x="592" y="24" width="120" height="38" rx="5"/><text class="bf-ttiny" x="598" y="39">Lanes</text><text class="bf-ttiny" x="598" y="54">no gain</text><rect class="bf-card-hi" x="10" y="78" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="98">Building the map of retained rows</text><text class="bf-tm" x="22" y="115">half or more of Open's time</text><line class="bf-rule" x1="310" y1="101" x2="340" y2="101"/><rect class="bf-slot" x="340" y="82" width="120" height="38" rx="5"/><text class="bf-ttiny" x="346" y="97">Fused passes, compressed queues</text><text class="bf-ttiny" x="346" y="112">#654</text><line class="bf-rule" x1="310" y1="101" x2="466" y2="101"/><rect class="bf-slot" x="466" y="82" width="120" height="38" rx="5"/><text class="bf-ttiny" x="472" y="97">Queue-free backward build</text><text class="bf-ttiny" x="472" y="112">#659</text><line class="bf-rule" x1="310" y1="101" x2="592" y2="101"/><rect class="bf-slot" x="592" y="82" width="120" height="38" rx="5"/><text class="bf-ttiny" x="598" y="97">Precomputed jumps</text><text class="bf-ttiny" x="598" y="112">#652</text><rect class="bf-card-hi" x="10" y="136" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="156">Finding where runs overlap</text><text class="bf-tm" x="22" y="173">about 1.3 transitions per run</text><line class="bf-rule" x1="310" y1="159" x2="340" y2="159"/><rect class="bf-slot" x="340" y="140" width="120" height="38" rx="5"/><text class="bf-ttiny" x="346" y="155">Via Cells' induction</text><text class="bf-ttiny" x="346" y="170">≈1.8 steps per run</text><rect class="bf-card-hi" x="10" y="194" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="214">Metadata held for every run</text><text class="bf-tm" x="22" y="231">40–100 bytes per run</text><line class="bf-rule" x1="310" y1="217" x2="340" y2="217"/><rect class="bf-slot" x="340" y="198" width="120" height="38" rx="5"/><text class="bf-ttiny" x="346" y="213">Grammar machines</text><text class="bf-ttiny" x="346" y="228">per-run state</text><rect class="bf-card" x="10" y="252" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="272">Starting from the decoded column</text><text class="bf-tm" x="22" y="289">unexplored</text><text class="bf-tm" x="340" y="280">open: no candidate yet</text><rect class="bf-card" x="10" y="310" width="300" height="46" rx="6"/><text class="bf-t-strong" x="22" y="330">Writing every output byte</text><text class="bf-tm" x="22" y="347">the floor</text><text class="bf-tm" x="340" y="338">unavoidable</text></svg>
<figcaption>The obligations are from the <a href="https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/FRONTIER.md">register</a>; the shares of Open's time are measured (#653). Each grey box is a candidate rejected on native timing or by a decisive count, except for overlaps, where it is the known, costly route.</figcaption>
</figure>

Reading across a row says what was learned about that obligation. Three candidates attacked the walk over retained rows: a software-pipelined walk (the processor already issues the next load early), a compact rank structure in place of the stored edges (two to three times slower, because one load became four dependent ones), and several walks at once (no gain in our test harness). Three attacked the build: fusing passes, compressing queues, and building the map backwards. The backward build was the most instructive. It removed the queues it targeted, but its second pass over the run records cost about as much as the queues had.

When a failure exposes a real obstruction, the register records three things: the concrete **witness**, the exact **scope** (which algorithms and cost model it binds), and the **escape condition**, the assumption a different formulation would have to drop. An obstruction without a stated scope does not count. That rule matters, because the temptation after six losses is to conclude that the walk is a law of nature. It isn't. It is a law of the representations tried so far.

## 2. Generate from structure, judge against obligations

The register is for judging, not for generating. Early in the week it was used for both, and the ideas it produced were rephrasings of the costs in its own rows: "make the queue smaller", "make the walk shorter". So the method now separates the two, inside a cycle of five phases:

<figure class="bf-fig">
<p class="bf-title">The research cycle</p>
<svg viewBox="0 0 720 206" role="img" aria-label="The research cycle: Explore generates from mathematical structure; Triage, Deepen, Confront and Learn judge each idea against the obligation register."><rect class="bf-card-hi" x="10" y="30" width="122" height="44" rx="8"/><text class="bf-ts" x="71" y="57" text-anchor="middle">Explore</text><line class="bf-arrow" x1="134" y1="52" x2="150" y2="52"/><rect class="bf-card" x="152" y="30" width="122" height="44" rx="8"/><text class="bf-ts" x="213" y="57" text-anchor="middle">Triage</text><line class="bf-arrow" x1="276" y1="52" x2="292" y2="52"/><rect class="bf-card" x="294" y="30" width="122" height="44" rx="8"/><text class="bf-ts" x="355" y="57" text-anchor="middle">Deepen</text><line class="bf-arrow" x1="418" y1="52" x2="434" y2="52"/><rect class="bf-card" x="436" y="30" width="122" height="44" rx="8"/><text class="bf-ts" x="497" y="57" text-anchor="middle">Confront</text><line class="bf-arrow" x1="560" y1="52" x2="576" y2="52"/><rect class="bf-card" x="578" y="30" width="122" height="44" rx="8"/><text class="bf-ts" x="639" y="57" text-anchor="middle">Learn</text><text class="bf-tm" x="71" y="96" text-anchor="middle">generate from</text><text class="bf-tm" x="71" y="111" text-anchor="middle">mathematical structure</text><rect class="bf-slot" x="152" y="128" width="558" height="44" rx="8"/><text class="bf-t" x="431" y="155" text-anchor="middle">judge against the obligation register: what paid work disappears, and what replaces it?</text><line class="bf-rule" x1="213" y1="76" x2="213" y2="126"/><line class="bf-rule" x1="355" y1="76" x2="355" y2="126"/><line class="bf-rule" x1="497" y1="76" x2="497" y2="126"/><line class="bf-rule" x1="639" y1="76" x2="639" y2="126"/><text class="bf-tm" x="360" y="196" text-anchor="middle">Learn writes back the obligation, its witness, scope and escape condition</text></svg>
<figcaption>Conceptual. Ideas come from mathematics; the register decides which survive.</figcaption>
</figure>

**Explore** generates from mathematical structure. The charter names four routes, as tools rather than quotas. *Representation import* asks what richer object this is an instance of: both earlier breakthroughs came this way, from seeing the row map as an interval exchange. *Obstruction inversion* takes a failure's narrowest witness and asks which assumption another primitive could escape. *Primitive ablation* forbids a dominant operation and checks the cost doesn't reappear elsewhere. *Small-instance discovery* finds the best exact construction on tiny inputs and looks for the law behind it.

**Triage, Deepen, Confront and Learn** judge. Every candidate must say which paid work disappears, what replaces it, and how the information it needs is obtained: we call this *acquisition first*, because several elegant ideas died on the cost of getting their inputs rather than using them. Every cost model is treated as a diagnostic. This is not caution for its own sake. A cache model ranked two of Open's variants the wrong way round, predicting a win for the one that runs 10–16% slower natively. None of the six candidates that passed a model passed native timing.

Two smaller rules hold the process together. A research session pushes its work at every handoff and at least every half hour. One session was cut off mid-cycle by a usage limit, and its earlier cycles survived only because they were already published. And new mathematical leads go into an inbox that the researcher reads at a cycle boundary, so a good idea from outside never derails an experiment halfway through.

## 3. What the register has taught us

Five research cycles by one model, and a sixth line now running with another, have mapped the walls fairly precisely. Representations that are expensive to build lose on construction. Representations that are compact lose on consumption, because recovering each next step takes several dependent operations instead of one load. Neither is a lower bound for the problem; both are evidence about the families tried.

Today's measurement added the most useful single fact. Tracing every expensive row back to the byte it spells showed that the repetitive JSON skeleton is already nearly free, while unique random statement IDs, about 5% of the text, hold about 62% of the rows that stand alone. The obligation "follow retained rows one by one" is, in large part, the obligation to spell random strings. That turned a vague idea, a zoo of methods each suited to a different kind of text, into specific questions, written up in [the previous post](2026-11-20-where-the-decoding-work-goes.md).

## 4. Adapting the method: toy, missing effect, native surrogate

Today Levent Alpöge [announced](https://alpo.ge/remark.pdf) that he believes he has proved the Riemann Hypothesis, and its generalisation, working with Claude, with the argument formalised in Lean. The claim is new and has not yet been independently checked, and nothing here depends on it being right. What interested us is his account of how the argument was found, because it describes a way of searching that our method lacked.

As he tells it, he started from a *toy* setting where the theorem was already known (the Riemann Hypothesis over function fields) and re-proved it in a new way chosen so the proof might travel. He then named the ingredient that the hard setting lacks: Stepanov's classical argument uses *multiplicity*, a polynomial vanishing to high order, which over the integers would be "something like differentiating whole numbers". The decisive step was to find a different mechanism, native to the hard setting, with the same effect: chains of theta lifts, each standing in for one order of multiplicity. The human held the analogy; the AI system supplied the technical mastery to build it.

We have a toy setting of our own. When a block's last column has exactly one run per letter, the text is the coding of an especially simple interval exchange. With two letters these are the Sturmian words, and their text needs no walk at all: letter $k$ is $\lfloor (k+1)\alpha \rfloor - \lfloor k\alpha \rfloor$ for a fixed slope $\alpha$. Every output position is computed, not reached.

<figure class="bf-fig">
<p class="bf-title">Toy, damaged toy, real block</p>
<svg viewBox="0 0 720 200" role="img" aria-label="Three panels: in the two-letter toy every position is computed from kα; in a rotation cut into many pieces the coordinate survives but runs multiply; in a real block there is no known coordinate."><rect class="bf-card-hi" x="10" y="14" width="224" height="150" rx="8"/><text class="bf-ts" x="24" y="40">The toy</text><text class="bf-tm" x="24" y="60">two-letter Sturmian words</text><text class="bf-t" x="24" y="96">letter k = ⌊(k+1)α⌋ − ⌊kα⌋</text><text class="bf-t" x="24" y="124">every position computed:</text><text class="bf-t" x="24" y="144">no walk at all</text><line class="bf-arrow" x1="236" y1="90" x2="254" y2="90"/><rect class="bf-slot" x="246" y="14" width="224" height="150" rx="8"/><text class="bf-ts" x="260" y="40">Damaged on purpose</text><text class="bf-tm" x="260" y="60">a rotation cut in many pieces</text><text class="bf-t" x="260" y="96">position still x + kα</text><text class="bf-t" x="260" y="124">but the runs multiply:</text><text class="bf-t" x="260" y="144">width lost, coordinate kept</text><line class="bf-arrow" x1="472" y1="90" x2="490" y2="90"/><rect class="bf-card" x="482" y="14" width="224" height="150" rx="8"/><text class="bf-ts" x="496" y="40">A real block</text><text class="bf-tm" x="496" y="60">Wikidata JSON</text><text class="bf-t" x="496" y="96">no known coordinate</text><text class="bf-t" x="496" y="124">random IDs: one-row runs,</text><text class="bf-t" x="496" y="144">walked step by step</text><text class="bf-tm" x="360" y="190" text-anchor="middle">The question: what object, native to real blocks, gives cheap global coordinates the way kα does?</text></svg>
<figcaption>Conceptual. The middle panel is a suggested experiment, not yet run.</figcaption>
</figure>

The first version of our transfer went wrong in an instructive way. We named the missing ingredient as run width, since the toy has two enormous runs and the random IDs have none, and tested an obvious surrogate the same evening: walking many independent stretches of the path at once, as mibz2's plain decoder already does. It gave no gain on Open's walk, and more to the point, it was the wrong test. Running walks side by side still follows the same edges; it is a speed-up of the old operation, not a new one.

The correction, which came out of the discussion, is that the toy's real power is not width but **cheap global coordinates**: a compact number ($k\alpha$ modulo 1) from which any position's letter follows. In the toy the two coincide, which is why they were easy to confuse. They come apart if the toy is damaged on purpose: code a rotation by a circle cut into many pieces, and the coordinate $x + k\alpha$ survives while the runs multiply. So the question for the research line is now precise: what object, native to real blocks and cheap to read off the compressed input, gives coordinates the way $k\alpha$ does, or lets long stretches of the path be composed without walking them? Mathematics that might answer it already exists in outline (the numeration systems that generalise continued fractions to interval exchanges), and so does the discipline to test it: the register asks what it would remove and what it would cost to acquire.

## 5. Where this leaves us

The method in one chain: model the obligations, because they decompose and algorithms don't; generate ideas from mathematical structure; judge them against the obligations, with acquisition counted and models treated as diagnostics; and when a phenomenon makes the problem easy somewhere, find what makes it easy, break it on purpose, and look for a different mechanism that recreates the effect where it is missing.

A new research line is running now, independently, with the geometric leads (square-tiled surfaces, train tracks, interval pairings) as starting points and the toy-transfer lead waiting at its next cycle boundary. Its results come back for independent verification before anything reaches the decoder.

---

**Research records:**

- [The research charter: method](https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/METHOD.md) and [frontier and obligation register](https://github.com/lmmx/mibz2/blob/932ff82c5/docs/team/research-charter/FRONTIER.md).
- [Toy, missing ingredient, native surrogate](https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-toy-transfer.md), with the correction and the lanes experiment.
- [Working backwards from the text](https://github.com/lmmx/mibz2/blob/932ff82c5/docs/journal/2026-11-20-working-backwards.md).
- Levent Alpöge, [Remark](https://alpo.ge/remark.pdf) (20 November 2026), announcing a claimed proof, not yet independently verified.
- S. Ferenczi and L. Q. Zamboni, *Clustering words and interval exchanges* (2013); S. Mantaci, A. Restivo and M. Sciortino, *Burrows–Wheeler transform and Sturmian words* (2003).
