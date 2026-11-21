---
title: "Besieging hard problems: routes through distant mathematics, from the Riemann proof to the inverse BWT"
image: assets/social/blog-2026-11-21-besieging-hard-problems.png
description: "The Riemann proof reached its goal through mathematics built for other problems, and four AI research lines that tried this on a decoder found its bottleneck."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Besieging hard problems: routes through distant mathematics, from the Riemann proof to the inverse BWT

The Riemann proof reached its goal through mathematics built for other problems, and four AI research lines that tried this on a decoder found its bottleneck.
{: .post-dek }

21 November 2026 · 17 min read · [Meta](#7-what-we-are-searching-for-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

On 20 November 2026, a proof of the Riemann Hypothesis was accepted by three independent proof checkers. It is about 17 million lines of Lean, found by Levent Alpöge working with Claude. Its strangest feature is not its size but its route. It does not build new theory towards the goal. It passes through a long chain of existing results, each made for some other problem, and arrives somewhere a direct route could not reach. This post is about that route, and about an experiment. We gave the same mandate, to use everything, to four AI research lines. Their problem was our own: the inverse Burrows–Wheeler transform at the heart of mibz2's bzip2 decoder.

**The takeaways:**

- **The achievement is a new mode of invention, not only a large proof.** A number theorist described it as a wormhole through "the infrastructure built for a hundred other mineral reserves". The infrastructure the proof needs, from resolution of singularities to prismatic Dieudonné theory, is proved inside it, with no axioms. Three independent kernels accepted it before anyone could say how its main ingredients combine.
- **The route can be described as twelve levers.** We read them in the proof's reported shape. The target is strengthened into a statement about a family. A forbidden event is excluded by bounds that independent theories place on one quantity. The work moves to where those bounds exist, and every foundation is built rather than assumed. This is our interpretation, not the authors'.
- **We ran the experiment.** Since 20 November, four Gemini research lines have attacked the inverse BWT. One of them, Kitchen Sink, was told not to require a field of mathematics to justify its relevance in advance. Their chains ran through graph theory, discrete Morse theory, interval-exchange dynamics, ergodic theory, queueing theory and compiler register allocation.
- **The chains broke informatively, and three of them converged.** Most produced exact mathematics and poor algorithms; each break named the next link to look for. This morning Mechanisms, Wormhole and Kitchen Sink independently identified the same bottleneck. Each worker holds about 4.5 MiB of scratch, which overflows the processor's shared cache once 7 to 10 workers run. Their candidate fixes act on different buffers. Whether they compose, and hold up natively on our i9, is still to be measured.
- **The pace is set by checking.** Exact correctness checks are cheap, so the search can be broad. Checks of speed on the real machine are not, and they decide how fast any of this can be adopted. The open question is the floor: the cost that remains once memory fits, and whether a different route from runs to text order can avoid it.

[Section 1](#1-the-problem-and-what-was-achieved) shows what was achieved, with the overview that started this. [Section 2](#2-the-wormhole-hypothesis) states the wormhole hypothesis, and [section 3](#3-the-twelve-levers) the levers that make such a route work. [Section 4](#4-we-tried-it-on-a-decoder) describes the experiment, [section 5](#5-what-happened-when-we-used-everything) what it found, [section 6](#6-the-constraint-that-sets-the-pace) the constraint that governs it, and [section 7](#7-what-we-are-searching-for-next) what comes next.

## 1. The problem, and what was achieved

The Riemann Hypothesis was posed in 1859. It says that the nontrivial zeros of one function, which governs how the primes are spread, all lie on a single line. It has resisted every attempt for more than 160 years, and most mathematicians assumed that a proof would need large bodies of theory not yet built.

The Lean development proves it, together with its generalisation to Dirichlet L-functions, which carry the same question for primes in arithmetic progressions. It does not cover the wider Selberg class. Members of the Lean community ran it through three independent kernels, nanoda, con-leche in its verified mode, and pestka, and each accepted it. Alpöge's own [remark](https://alpo.ge/remark.pdf) describes a toy proof that guided the search; that toy is not in the Lean. He also describes generated exposition running past 250,000 pages, which he chose not to present as a readable proof.

A kernel's acceptance is strong evidence that the formal theorem follows from its foundations. It is not yet the mathematical community's review of the argument, which has only begun. **We have not checked the proof,** and nothing in this post depends on it being right. What we use is its reported shape. The clearest account of that shape is the overview Chris Birkbeck posted on the Lean Zulip thread, written with Fable's help:

<figure class="bf-fig" id="birkbeck">
<a href="../assets/images/besieging/birkbeck-overview.jpg"><img src="../assets/images/besieging/birkbeck-overview.jpg" width="1303" height="808" loading="lazy" alt="Chris Birkbeck on Lean Zulip: a high-level overview in five points. One, RH is deduced from no Dirichlet L-function vanishing for real part above one half, a short standard step. Two, that nonvanishing is deduced from a growth bound on Tate–Shafarevich-like Galois cohomology groups of a family of subfields of cyclotomic fields, a covolume bound from arithmetic geometry, and statements about theta integrals on unitary groups with local Langlands for GL_n; he has not yet understood how these combine to exclude zeros. Three, the cohomology bound is proved by an automorphy-lifting argument using Galois representations attached to regular algebraic polarized cuspidal automorphic representations of GL_n over CM fields. Four, those Galois representations are not assumed but derived from a geometric construction for unitary groups and theta lifts, which is not the route in the literature. Five, the arithmetic geometry side proves resolution of singularities, arithmetic Hilbert–Samuel, Moriwaki's bigness theorem, toroidal compactifications of Siegel varieties, Kawamata–Viehweg and Nadel vanishing, Grothendieck–Messing and prismatic Dieudonné theory inside the project, with no axioms."></a>
<figcaption>Chris Birkbeck's overview, posted in the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis", 21 November 2026.</figcaption>
</figure>

Laid out as a structure, the overview describes a proof that reaches its target from far away. Each block is an existing body of mathematics, developed for some other purpose:

<figure class="bf-fig" id="proof-shape">
<p class="bf-title">The proof's shape, as Birkbeck's overview describes it</p>
<svg viewBox="0 0 720 432" role="img" aria-label="A dependency tree. At the top, the Riemann Hypothesis follows by a short standard step from: no Dirichlet L-function vanishes for real part above one half. That follows from three blocks: a growth bound on Tate–Shafarevich-like Galois cohomology over cyclotomic subfields; a covolume bound from arithmetic geometry; theta integrals on unitary groups with local Langlands for GL_n. How the three combine is not yet understood. Below them: automorphy lifting with Galois representations built, not assumed, from Eichler–Shimura for unitary groups and theta lifts; foundations proved inside with no axioms, from resolution of singularities to prismatic Dieudonné theory; and complete local classification by local Langlands, Clozel's purity and Bernstein–Zelevinsky."><defs><marker id="bs-a" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" style="fill:var(--bf-muted)"/></marker></defs>
<rect x="265" y="8" width="190" height="36" rx="6" class="bf-card-hi"/><text x="360" y="31" class="bf-ts" text-anchor="middle">Riemann Hypothesis</text>
<rect x="150" y="74" width="420" height="44" rx="6" class="bf-pill"/><text x="360" y="93" class="bf-t-strong" text-anchor="middle">No Dirichlet L-function vanishes for Re(s) &gt; 1/2</text><text x="360" y="110" class="bf-tm" text-anchor="middle">RH follows by a short, standard step</text>
<line x1="360" y1="74" x2="360" y2="47" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/>
<text x="12" y="140" class="bf-tm">How the three combine:</text><text x="12" y="155" class="bf-tm">“not yet understood”</text>
<rect x="12" y="172" width="224" height="72" rx="6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1"/><text x="124" y="194" class="bf-t" text-anchor="middle" style="font-size:11.5px">Growth bound on Galois</text><text x="124" y="210" class="bf-t" text-anchor="middle" style="font-size:11.5px">cohomology (Tate–Shafarevich-like)</text><text x="124" y="226" class="bf-t" text-anchor="middle" style="font-size:11.5px">over cyclotomic subfields</text>
<rect x="248" y="172" width="224" height="72" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="360" y="202" class="bf-t" text-anchor="middle" style="font-size:11.5px">Covolume bound from</text><text x="360" y="218" class="bf-t" text-anchor="middle" style="font-size:11.5px">arithmetic geometry</text>
<rect x="484" y="172" width="224" height="72" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="596" y="194" class="bf-t" text-anchor="middle" style="font-size:11.5px">Theta integrals on unitary</text><text x="596" y="210" class="bf-t" text-anchor="middle" style="font-size:11.5px">groups, with local Langlands</text><text x="596" y="226" class="bf-t" text-anchor="middle" style="font-size:11.5px">for GL_n</text>
<line x1="124" y1="172" x2="300" y2="121" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/><line x1="360" y1="172" x2="360" y2="121" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/><line x1="596" y1="172" x2="420" y2="121" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/>
<rect x="12" y="282" width="224" height="92" rx="6" class="bf-pill"/><text x="124" y="302" class="bf-t-strong" text-anchor="middle" style="font-size:11.5px">Automorphy lifting</text><text x="124" y="319" class="bf-t" text-anchor="middle" style="font-size:11.5px">Galois representations for GL_n</text><text x="124" y="335" class="bf-t" text-anchor="middle" style="font-size:11.5px">over CM fields, built, not assumed:</text><text x="124" y="351" class="bf-t" text-anchor="middle" style="font-size:11.5px">Eichler–Shimura for unitary</text><text x="124" y="367" class="bf-t" text-anchor="middle" style="font-size:11.5px">groups, then theta lifts</text>
<rect x="248" y="282" width="224" height="142" rx="6" class="bf-pill"/><text x="360" y="302" class="bf-t-strong" text-anchor="middle" style="font-size:11.5px">No axioms: proved inside</text><text x="360" y="319" class="bf-t" text-anchor="middle" style="font-size:11.5px">resolution of singularities,</text><text x="360" y="335" class="bf-t" text-anchor="middle" style="font-size:11.5px">arithmetic Hilbert–Samuel,</text><text x="360" y="351" class="bf-t" text-anchor="middle" style="font-size:11.5px">Moriwaki's bigness theorem,</text><text x="360" y="367" class="bf-t" text-anchor="middle" style="font-size:11.5px">toroidal compactifications,</text><text x="360" y="383" class="bf-t" text-anchor="middle" style="font-size:11.5px">Kawamata–Viehweg and Nadel,</text><text x="360" y="399" class="bf-t" text-anchor="middle" style="font-size:11.5px">Grothendieck–Messing,</text><text x="360" y="415" class="bf-t" text-anchor="middle" style="font-size:11.5px">prismatic Dieudonné theory</text>
<rect x="484" y="282" width="224" height="76" rx="6" class="bf-pill"/><text x="596" y="302" class="bf-t-strong" text-anchor="middle" style="font-size:11.5px">Complete local classification</text><text x="596" y="319" class="bf-t" text-anchor="middle" style="font-size:11.5px">local Langlands, Clozel's purity,</text><text x="596" y="335" class="bf-t" text-anchor="middle" style="font-size:11.5px">Bernstein–Zelevinsky</text>
<line x1="124" y1="282" x2="124" y2="247" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/><line x1="360" y1="282" x2="360" y2="247" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/><line x1="596" y1="282" x2="596" y2="247" style="stroke:var(--bf-muted);stroke-width:1.5" marker-end="url(#bs-a)"/>
</svg>
<figcaption>Arrows point from what is used to what it proves. The blocks follow Birkbeck's five points; which foundation feeds which bound is simplified here. Birkbeck's own words on the three-way join: "I have not yet understood how these combine to exclude zeros."</figcaption>
</figure>

## 2. The wormhole hypothesis

A number theorist, writing on social media, gave the reading that this post is built on. A sufficiently deep problem is like a mineral reserve buried miles underground, and the usual way to reach it is to build the infrastructure for it, painstakingly, over decades. This proof did something else:

> It takes an incredibly complicated tour through the infrastructure built for a hundred other mineral reserves, situated far from the destination, and somehow create[s] a wormhole to the original destination. … no human brain, on its own, is capable of creating a wormhole that requires a 25,000 page long tour through existing infrastructure with no obvious or realistic connection to the goal.

Two remarks relayed to us made the consequence plain. Alex Kontorovich is reported to have summed the proof up as "RH is proved by using *everything*!", and Igor Carron to have observed that throwing the whole kitchen sink at a problem has never been easier.

The hypothesis we took from this is about invention, not proof checking. **Some problems may be hard not because their solution needs an idea no one has had, but because the only routes to it pass through mathematics whose relevance cannot be seen from the start, and are too long for people to assemble.** If that is so, the way to attack such a problem is to search across distant fields on purpose, accept routes of any length, and check every step exactly. Whether this transfers from number theory to an algorithmic problem is an empirical question. Section 4 describes how we set out to test it.

## 3. The twelve levers

What makes a long detour useful rather than merely long? Read as strategy, Birkbeck's overview suggests a set of moves, which we call levers. Our [research note](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md) derives them. Where a lever names a classical pattern, such as Wiles's numerical criterion or Deligne's tensor-power trick, that is our reading, not something the overview states.

They are not independent. Strengthening the statement (1) is what lets bounds from different theories apply to it (4). Those bounds often exist only in another setting, which forces the move (7). The move is then paid for by building its foundations (9). The breadth of that search is affordable only because each step is cheap to check (10).

The third column gives the corresponding move in our research lines, where there is a genuine one. These are analogies and applications, not claims about how the proof works.

| | Lever | In the proof | In our research lines |
|---|---|---|---|
| **I** | **1. Strengthen the target until your tools preserve it** | Not RH alone, but nonvanishing for every Dirichlet L-function, a family closed under twisting and base change | Claims stated for all small cases and every authenticated block, not one kernel on one block |
| | **2. Prove a growth rate, not a value** | Bounds compared along a tower of cyclotomic fields | Not yet attempted: scaling of dependent loads with block size |
| | **3. Use robust inequalities in both directions** | Hilbert–Samuel, bigness and vanishing theorems are inequalities | Registered intervals on the i9, never point estimates |
| **II** | **4. Find a quantity two independent theories can each bound** | A Selmer-like cohomology group, bounded by Galois theory and by geometry (our reading) | Bytes resident per worker, bounded by Kitchen Sink's contention law and by Wormhole's cache model |
| | **5. Amplify a defect until coarse tools can see it** | A single zero pushed through a family (our reading) | Kitchen Sink's 2-adic conjugation of LF (cycle 13): no amplification, as hard as the answer itself |
| | **6. Reach for positivity** | Theta integrals and volumes, which cannot be negative | Not yet found |
| **III** | **7. Move to a setting where the tools exist, then come back** | Base change and theta lifts to unitary groups, whose Shimura varieties carry geometry | The kernel recast as a discrete Morse collapse; runs as an interval exchange |
| | **8. Push difficulty to the local steps, where classification is complete** | Local Langlands at each prime | Failed: the output depends on global order (section 7) |
| | **9. Re-optimise the route when the cost function changes** | "Not the route in the literature": a cited black box costs its full formalisation | From instructions per byte to bytes resident per worker (section 5) |
| **IV** | **10. Make checking cheap and certain, then search widely** | Kernel checks: certain, and cheap next to discovery | Exact verifiers on tens of thousands of cases make Kitchen Sink's breadth affordable |
| | **11. Use a toy to find the missing ingredient, then drop it** | The remark's toy proof is not in the Lean | Toy blocks where inversion is free, used only to name what must transfer |
| | **12. Separate trust from understanding** | Accepted before anyone understood the three-way join | Adopt on registered measurement; mechanisms stay hypotheses |

## 4. We tried it on a decoder

mibz2 is a bzip2 decoder in Rust. The core of decoding is inverting the BWT: rebuilding each 900 kB block from its sorted form. The textbook method walks the LF mapping, one dependent memory load per output byte, and that walk dominates decoding time. Since early November we had been searching for faster routes.

On 20 November, after Alpöge's announcement, we reorganised the search around the wormhole hypothesis. Each research line is a Gemini session with its own branch, brief and journal. The brief for Kitchen Sink, the line created for this purpose, reads in part:

> Decide which mathematical frameworks, theorems, analogies, constructions and combinations deserve investigation. Range as widely across mathematics as your judgement suggests, including fields with no established connection to BWT, compression or algorithms. … Do not require every mathematical framework to justify its relevance in advance. Discovering that relevance is part of your research.

That is from its [launch record](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/launches/2026-11-20-carbon-kitchen-sink.md). The four lines differ in purpose:

- **Mechanisms** builds reconstruction mechanisms and times them against the shipped kernel.
- **Kitchen Sink** is the breadth engine, trying bridges from about 25 fields per cycle.
- **Depth** priced the work and sought lower bounds. It has finished, and [our audit](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-20-carbon-depth-audit.md) found its bounds rest on asserted constants, so they are not yet floors.
- **Wormhole**, launched after the number theorist's description, looks for one long chain of existing results, each built for something else, ending in a cheap operation.

Breadth without checking would produce only noise, so the lines work inside a discipline:

<figure class="bf-fig" id="siege">
<p class="bf-title">The siege: four research lines, their own checks, and ours</p>
<svg viewBox="0 0 720 340" role="img" aria-label="Four research lines on the left: Mechanisms, which builds and times mechanisms, 16 cycles; Kitchen Sink, which uses everything, about 25 fields, 15 cycles; Depth, costs and lower bounds, finished; Wormhole, a chain of existing results to one target, 5 cycles with round 2 running. They feed their own checks: claims tagged proved, verified, measured or conjecture; exact verifiers on small cases and six authenticated blocks; kill rules set before data. Then the coordinator: native Rust port, CI parity, fuzzing and Miri, the i9 at 6 to 10 workers with a rule registered first. Then adoption and release, or a written negative result. A dashed arrow returns what did not transfer to the research charter, the fidelity ledger."><defs><marker id="bs-b" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" style="fill:var(--bf-muted)"/></marker></defs>
<text x="12" y="22" class="bf-ts">Four Gemini research lines</text>
<rect x="12" y="34" width="246" height="56" rx="6" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1"/><text x="24" y="56" class="bf-t-strong">Mechanisms</text><text x="24" y="76" class="bf-t" style="font-size:12px">builds and times mechanisms; 16 cycles</text>
<rect x="12" y="100" width="246" height="56" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="24" y="122" class="bf-t-strong">Kitchen Sink</text><text x="24" y="142" class="bf-t" style="font-size:12px">uses everything, ~25 fields; 15 cycles</text>
<rect x="12" y="166" width="246" height="56" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-green);stroke-width:1"/><text x="24" y="188" class="bf-t-strong">Depth</text><text x="24" y="208" class="bf-t" style="font-size:12px">costs and lower bounds; finished</text>
<rect x="12" y="232" width="246" height="56" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="254" class="bf-t-strong">Wormhole</text><text x="24" y="274" class="bf-t" style="font-size:12px">one chain of results; round 2 running</text>
<rect x="292" y="96" width="186" height="132" rx="6" class="bf-pill"/><text x="385" y="118" class="bf-t-strong" text-anchor="middle">Their own checks</text><text x="385" y="140" class="bf-t" text-anchor="middle" style="font-size:12px">every claim tagged: proved,</text><text x="385" y="157" class="bf-t" text-anchor="middle" style="font-size:12px">verified, measured, conjecture</text><text x="385" y="179" class="bf-t" text-anchor="middle" style="font-size:12px">exact verifiers on small cases</text><text x="385" y="196" class="bf-t" text-anchor="middle" style="font-size:12px">and 6 authenticated blocks</text><text x="385" y="216" class="bf-t" text-anchor="middle" style="font-size:12px">kill rules set before data</text>
<line x1="258" y1="62" x2="290" y2="140" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/><line x1="258" y1="128" x2="290" y2="155" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/><line x1="258" y1="194" x2="290" y2="170" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/><line x1="258" y1="260" x2="290" y2="185" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/>
<rect x="514" y="40" width="194" height="120" rx="6" style="fill:var(--bf-claude-tint);stroke:var(--bf-claude);stroke-width:1"/><text x="611" y="62" class="bf-t-strong" text-anchor="middle">Our checks</text><text x="611" y="84" class="bf-t" text-anchor="middle" style="font-size:12px">native Rust port</text><text x="611" y="102" class="bf-t" text-anchor="middle" style="font-size:12px">CI: parity, fuzzing, Miri</text><text x="611" y="120" class="bf-t" text-anchor="middle" style="font-size:12px">the i9 at 6–10 workers,</text><text x="611" y="138" class="bf-t" text-anchor="middle" style="font-size:12px">its rule registered first</text>
<line x1="478" y1="150" x2="512" y2="110" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/>
<rect x="514" y="190" width="194" height="70" rx="6" class="bf-card-hi"/><text x="611" y="214" class="bf-t-strong" text-anchor="middle">Adopted and released,</text><text x="611" y="234" class="bf-t" text-anchor="middle" style="font-size:12px">or a negative result</text><text x="611" y="251" class="bf-t" text-anchor="middle" style="font-size:12px">written up and kept</text>
<line x1="611" y1="160" x2="611" y2="187" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-b)"/>
<path d="M611,260 L611,312 L135,312 L135,292" style="fill:none;stroke:var(--bf-muted);stroke-width:1.2;stroke-dasharray:4 4" marker-end="url(#bs-b)"/><text x="373" y="330" class="bf-tm" text-anchor="middle">what did not transfer goes back to every line: the fidelity ledger</text>
</svg>
<figcaption>Each line is a Gemini session with its own branch and brief; a cycle ends in a journal with its programs. The lines claim; our checks decide.</figcaption>
</figure>

The lines' theorems are exploratory proofs: written out, then checked by exhaustive enumeration of small cases and byte-exact decoding of six authenticated Wikidata blocks. That makes them machine-tested claims. None has been independently reviewed as mathematics, and none is adopted on a line's own timings.

## 5. What happened when we used everything

**Most chains produced exact mathematics and poor algorithms, and the breaks were the useful part.** Wormhole's five cycles show the pattern best. Each cycle took a result from a different field and made it work on the decoder exactly. Each then broke on a cost, and the break named the next link to look for:

<figure class="bf-fig" id="chain">
<p class="bf-title">Wormhole's chain: five fields, each break naming the next link</p>
<svg viewBox="0 0 720 470" role="img" aria-label="Five cycles of the Wormhole line. Cycle 1, graph theory: a directed feedback set and a metric-DAG pushforward, citing Kahn, Levy and Low, and the Φ function of Kärkkäinen, Kempa and Puglisi; it breaks on a cycle-packing conservation law, 80 to 102 thousand seed rows and 2.8 to 6.2 MiB of bookkeeping. Cycle 2, topology: a width-filtered discrete Morse collapse, citing Forman, Sköldberg, Gelfand and Retakh, Mischaikow and Nanda; it breaks on a flattening cliff, the word dictionary growing to 0.63 to 0.82 MiB. Cycle 3, dynamics and grammars: periodic quotienting and an output-memoised grammar, citing Zorich, Agol, Hass and Thurston, Rytter and Gąsieniec; it runs 1.04 to 1.08 times slower but isolates the real cost, 4.66 MiB per worker, which overflows the L3 with seven workers. Cycle 4, ergodic theory: a meet-in-the-middle walk and a conjugate 16-bit stream, citing Birkhoff, Pohl and Kakutani; it breaks because Endpoint state still holds about 1.3 MiB. Cycle 5, compilers and counting: a pigeonhole envelope and Chaitin's lifetime colouring from register allocation, giving CPC-Micro at 0.32 of the shipped kernel's scratch and 0.93 to 0.97 of its time with four workers in its C harness."><defs><marker id="bs-d" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" style="fill:var(--bf-muted)"/></marker></defs>
<rect x="12" y="8" width="430" height="70" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="28" class="bf-t-strong">Cycle 1 · graph theory</text><text x="24" y="47" class="bf-t" style="font-size:12px">Directed feedback set, then a metric-DAG pushforward</text><text x="24" y="65" class="bf-tm">Kahn; Levy–Low; Kärkkäinen–Kempa–Puglisi's Φ</text>
<line x1="442" y1="43" x2="458" y2="43" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="462" y="14" width="246" height="58" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="474" y="32" class="bf-t-strong" style="font-size:11.5px">Breaks</text><text x="474" y="48" class="bf-t" style="font-size:11px">Cycle-packing conservation law:</text><text x="474" y="63" class="bf-t" style="font-size:11px">80–102k seed rows, 2.8–6.2 MiB of bookkeeping</text>
<path d="M585,72 L585,88 L227,88 L227,98" style="fill:none;stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="12" y="100" width="430" height="70" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="120" class="bf-t-strong">Cycle 2 · topology</text><text x="24" y="139" class="bf-t" style="font-size:12px">Width-filtered discrete Morse collapse of the kernel</text><text x="24" y="157" class="bf-tm">Forman; Sköldberg; Gel'fand–Retakh; Mischaikow–Nanda</text>
<line x1="442" y1="135" x2="458" y2="135" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="462" y="106" width="246" height="58" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="474" y="124" class="bf-t-strong" style="font-size:11.5px">Breaks</text><text x="474" y="140" class="bf-t" style="font-size:11px">Flattening cliff: the word dictionary</text><text x="474" y="155" class="bf-t" style="font-size:11px">balloons to 0.63–0.82 MiB</text>
<path d="M585,164 L585,180 L227,180 L227,190" style="fill:none;stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="12" y="192" width="430" height="70" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="212" class="bf-t-strong">Cycle 3 · dynamics and grammars</text><text x="24" y="231" class="bf-t" style="font-size:12px">Periodic quotienting, then an output-memoised grammar</text><text x="24" y="249" class="bf-tm">Zorich; Agol–Hass–Thurston; Rytter; Gąsieniec et al.</text>
<line x1="442" y1="227" x2="458" y2="227" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="462" y="198" width="246" height="58" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="474" y="216" class="bf-t-strong" style="font-size:11.5px">Breaks</text><text x="474" y="232" class="bf-t" style="font-size:11px">1.04–1.08× slower, but isolates the real cost:</text><text x="474" y="247" class="bf-t" style="font-size:11px">4.66 MiB per worker × 7 > the 20 MiB L3</text>
<path d="M585,256 L585,272 L227,272 L227,282" style="fill:none;stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="12" y="284" width="430" height="70" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="304" class="bf-t-strong">Cycle 4 · ergodic theory</text><text x="24" y="323" class="bf-t" style="font-size:12px">Meet-in-the-middle walk, then a conjugate 16-bit stream</text><text x="24" y="341" class="bf-tm">Birkhoff; Pohl; Kakutani</text>
<line x1="442" y1="319" x2="458" y2="319" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="462" y="290" width="246" height="58" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="474" y="308" class="bf-t-strong" style="font-size:11.5px">Breaks</text><text x="474" y="324" class="bf-t" style="font-size:11px">Endpoint state still holds</text><text x="474" y="339" class="bf-t" style="font-size:11px">about 1.3 MiB of separate buffers</text>
<path d="M585,348 L585,364 L227,364 L227,374" style="fill:none;stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="12" y="376" width="430" height="70" rx="6" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="24" y="396" class="bf-t-strong">Cycle 5 · compilers and counting</text><text x="24" y="415" class="bf-t" style="font-size:12px">Pigeonhole envelope, then lifetime colouring of buffers</text><text x="24" y="433" class="bf-tm">Chaitin's register allocation (1982)</text>
<line x1="442" y1="411" x2="458" y2="411" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-d)"/>
<rect x="462" y="382" width="246" height="58" rx="6" class="bf-card-hi"/><text x="474" y="400" class="bf-t-strong" style="font-size:11.5px">CPC-Micro</text><text x="474" y="416" class="bf-t" style="font-size:11px">0.32× the shipped kernel's scratch;</text><text x="474" y="431" class="bf-t" style="font-size:11px">0.93–0.97× its time, 4 workers (C)</text>
</svg>
<figcaption>Each cycle's construction is exact: checked on 26,495 small cases and the six authenticated blocks. The citations are those the line gives. The final figures come from its own C harness and are not yet natively validated; the native port is under way (section 5).</figcaption>
</figure>

Kitchen Sink's feedback-set result shows the same arc within one idea. It found that breaking only 0.3% of runs makes the run-overlap graph acyclic, which suggested an exact schedule with no walk at all. In cycle 11 it built that schedule and proved it exact. Then it measured the cost. Even with every cut run filled for free, 64 to 76% of the remaining fragments still deadlock in copy cycles, because rank inversions braid them together inside a graph with no cycles. Resolving them took 8 to 30 times the shipped kernel's interval splits and 3 to 5 times its memory. A striking simplification, followed by a precise account of why it does not become a fast algorithm: that is what the search mostly yields, and the record of it is what keeps the search from repeating itself.

**Then, on the morning of 21 November, three lines arrived at the same place.**

- **Kitchen Sink** ([cycles 11 to 15](https://github.com/lmmx/mibz2/commit/b69d74af87969168e9298551d5e08e2e1196ec15)) audited the shipped kernel and found its 256 rank queues hold 55 to 59% of its memory. It then combined queueing theory with a model of shared-cache spill into a contention law. That law retrodicts our past i9 results, including an eight-lane walk that ran 13% faster alone and [5% slower among eight workers](2026-11-21-fast-alone-slow-in-a-crowd.md). It also predicts that changes which shrink the working set do not reverse under load.
- **Wormhole** ([five cycles](https://github.com/lmmx/mibz2/commit/dea7ca38a8f006bdd3931b3962245bb77592c7bd)) was led there by its third break. Each worker holds 4.66 MiB of scratch, so seven workers need 37.9 MiB against the i9's 20 MiB shared L3.
- **Mechanisms** ([cycle 16](https://github.com/lmmx/mibz2/commit/d42b7fb93c57cc82ec6bfe938d48e209e39e265c)) modelled 3.4 bytes of memory traffic per output byte at ten workers. Each worker's buffers no longer fit its share of the cache.

None of the three was asked to look at memory. Each reached it from different mathematics, and that is lever 4 appearing on its own: a quantity, bytes resident per worker, that independent lines of reasoning bound. It is also lever 9. Our kernels had been tuned for instructions per byte, the cost that matters on one core. With ten workers sharing one cache, the cost that matters is bytes resident, and the best route changes with it.

Their candidate fixes differ, and act on different buffers:

<figure class="bf-fig" id="cuts">
<p class="bf-title">Where each line's cut lands in the Open kernel's per-worker memory</p>
<svg viewBox="0 0 720 318" role="img" aria-label="Rows for the shipped Open kernel's buffers, each with the line that cuts it. Rank queues, 256 lists, about 1.5 MiB and 55 to 59 percent of the kernel: Mechanisms stores them inside the output buffer, dead until emit, at zero bytes; Wormhole replaces them with a 16-bit stream of 79 to 147 KiB; Kitchen Sink pools them into pages, saving 792 KiB. Endpoint state, 0.17 to 0.55 MiB, live only before edges is filled: Wormhole stores it inside the dead edges array, adding zero bytes. Strips, bands and image order, metadata with disjoint lifetimes: Mechanisms shares two buffers instead of four. Band dictionary, 25 to 65 KiB, spilling the 32 KiB L1 on four of six blocks: Kitchen Sink memoises repeated words, fitting L1 on all six. The edges array and the output buffer stay, and now host the others."><defs><marker id="bs-c" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" style="fill:var(--bf-muted)"/></marker></defs>
<text x="12" y="20" class="bf-ts">Open 0.5.8, one worker</text><text x="372" y="20" class="bf-ts">The cut, and the line that found it</text>
<rect x="12" y="34" width="300" height="70" rx="6" class="bf-pill"/><text x="24" y="58" class="bf-t-strong">Rank queues (256 lists)</text><text x="24" y="78" class="bf-tm">≈ 1.5 MiB: 55–59% of the kernel</text>
<line x1="312" y1="69" x2="366" y2="69" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-c)"/>
<rect x="372" y="34" width="96" height="20" rx="4" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1"/><text x="420" y="48" class="bf-t" text-anchor="middle" style="font-size:11px">Mechanisms</text><text x="476" y="48" class="bf-t" style="font-size:12px">inside the dead output buffer: 0 B</text>
<rect x="372" y="59" width="96" height="20" rx="4" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="420" y="73" class="bf-t" text-anchor="middle" style="font-size:11px">Wormhole</text><text x="476" y="73" class="bf-t" style="font-size:12px">a 16-bit stream: 79–147 KiB</text>
<rect x="372" y="84" width="96" height="20" rx="4" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="420" y="98" class="bf-t" text-anchor="middle" style="font-size:11px">Kitchen Sink</text><text x="476" y="98" class="bf-t" style="font-size:12px">pooled pages: −792 KiB</text>
<rect x="12" y="120" width="300" height="50" rx="6" class="bf-pill"/><text x="24" y="141" class="bf-t-strong">Endpoint state</text><text x="24" y="160" class="bf-tm">0.17–0.55 MiB, live only before edges fills</text>
<line x1="312" y1="145" x2="366" y2="145" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-c)"/>
<rect x="372" y="135" width="96" height="20" rx="4" style="fill:var(--bf-ultra-tint);stroke:var(--bf-ultra);stroke-width:1"/><text x="420" y="149" class="bf-t" text-anchor="middle" style="font-size:11px">Wormhole</text><text x="476" y="149" class="bf-t" style="font-size:12px">inside the dead edges array: +0 B</text>
<rect x="12" y="184" width="300" height="50" rx="6" class="bf-pill"/><text x="24" y="205" class="bf-t-strong">Strips, bands, image order</text><text x="24" y="224" class="bf-tm">metadata with disjoint lifetimes</text>
<line x1="312" y1="209" x2="366" y2="209" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-c)"/>
<rect x="372" y="199" width="96" height="20" rx="4" style="fill:var(--bf-steel-tint);stroke:var(--bf-steel);stroke-width:1"/><text x="420" y="213" class="bf-t" text-anchor="middle" style="font-size:11px">Mechanisms</text><text x="476" y="213" class="bf-t" style="font-size:12px">two shared buffers instead of four</text>
<rect x="12" y="248" width="300" height="50" rx="6" class="bf-pill"/><text x="24" y="269" class="bf-t-strong">Band dictionary (inline16)</text><text x="24" y="288" class="bf-tm">25–65 KiB; spills the 32 KiB L1 on 4 of 6 blocks</text>
<line x1="312" y1="273" x2="366" y2="273" style="stroke:var(--bf-muted);stroke-width:1.2" marker-end="url(#bs-c)"/>
<rect x="372" y="263" width="96" height="20" rx="4" style="fill:var(--bf-amber-tint);stroke:var(--bf-amber);stroke-width:1"/><text x="420" y="277" class="bf-t" text-anchor="middle" style="font-size:11px">Kitchen Sink</text><text x="476" y="277" class="bf-t" style="font-size:12px">memo of repeated words: L1 on 6 of 6</text>
<text x="12" y="314" class="bf-tm">The edges array (0.35–0.8 MiB) and the output buffer (879 KiB) stay, and now host the others.</text>
</svg>
<figcaption>Sizes are the lines' own measurements on the six authenticated blocks. The tricks are of one kind: a buffer is dead for part of each block, and something else lives in it then. Mechanisms' ablation showed that the gain comes from reusing the output buffer's cache lines, not from the chunking: the same chunks in a separate buffer lost it.</figcaption>
</figure>

What decides the outcome on the i9 is whether a worker's buffers fit its share of the 20 MiB L3: 2.86 MiB with seven workers, 2.0 MiB with ten.

<figure class="bf-fig" id="footprint">
<p class="bf-title">Per-worker memory against each worker's share of the i9's L3</p>
<svg viewBox="0 0 720 262" role="img" aria-label="Horizontal bars of per-worker memory including the 879 KiB output buffer. Open 0.5.8: 4,480 KiB of scratch plus output, 5,359 KiB in all. Wormhole's W1: 2,220 plus output, 3,099 KiB. Mechanisms' SwarOutQ16: 1,722 plus output, 2,601 KiB. Wormhole's CPC-Micro: 1,475 plus output, 2,354 KiB. Dashed lines mark each worker's share of 20 MiB of L3: 2,926 KiB with seven workers and 2,048 KiB with ten. Only the shipped kernel exceeds the seven-worker share by far; the two smallest fit it."><text x="192" y="58" class="bf-t" text-anchor="end">Open 0.5.8 (shipped)</text><rect x="200" y="42" width="381" height="24" style="fill:var(--bf-claude)"/><rect x="581" y="42" width="75" height="24" style="fill:var(--bf-track)"/><text x="662" y="59" class="bf-t-strong">5,359</text>
<text x="192" y="102" class="bf-t" text-anchor="end">W1 (Wormhole)</text><rect x="200" y="86" width="189" height="24" style="fill:var(--bf-ultra)"/><rect x="389" y="86" width="75" height="24" style="fill:var(--bf-track)"/><text x="470" y="103" class="bf-t-strong">3,099</text>
<text x="192" y="146" class="bf-t" text-anchor="end">SwarOutQ16 (Mechanisms)</text><rect x="200" y="130" width="146" height="24" style="fill:var(--bf-steel)"/><rect x="346" y="130" width="75" height="24" style="fill:var(--bf-track)"/><text x="427" y="147" class="bf-t-strong">2,601</text>
<text x="192" y="190" class="bf-t" text-anchor="end">CPC-Micro (Wormhole)</text><rect x="200" y="174" width="125" height="24" style="fill:var(--bf-ultra)"/><rect x="325" y="174" width="75" height="24" style="fill:var(--bf-track)"/><text x="406" y="191" class="bf-t-strong">2,354</text>
<line x1="449" y1="30" x2="449" y2="210" style="stroke:var(--bf-ink);stroke-width:1.2;stroke-dasharray:5 4"/><line x1="374" y1="30" x2="374" y2="210" style="stroke:var(--bf-muted);stroke-width:1.2;stroke-dasharray:5 4"/>
<text x="449" y="228" class="bf-tm" text-anchor="middle">7 workers: 2,926 KiB each</text><text x="374" y="246" class="bf-tm" text-anchor="middle">10 workers: 2,048 KiB each</text>
<text x="200" y="22" class="bf-tm">KiB per worker: scratch (coloured) + output buffer (grey)</text>
</svg>
<figcaption>Scratch figures are each line's own harness measurements, not yet natively validated: Mechanisms' count of the shipped kernel (Wormhole measured 4,663 KiB). A worker does not touch every byte in each block, so the working set is smaller than these totals; Mechanisms measures 1,846 KiB on average for SwarOutQ16, inside even the ten-worker share.</figcaption>
</figure>

**Three things are established, and one is not.** The lines independently identified the same bottleneck. They proposed reductions that act on different parts of it, each exact on its checks. Their own harnesses show gains that widen under contention: Wormhole's C harness ran 0.93 to 0.97 of the shipped kernel's time with four workers against 0.95 to 0.98 on one thread. That is the reverse of the eight-lane failure. **Not yet established:** that the reductions compose, and that the gains survive natively on the i9. The combined candidate, Endpoint state in edges, queues in the output buffer and shared metadata, is being ported to native Rust now, one commit and one native timing per step. It then goes to the i9 at 6, 8 and 10 workers, under an adoption rule fixed before the run.

## 6. The constraint that sets the pace

The experiment's breadth rests on lever 10. A line can try twenty fields in a cycle because each attempt is cheap to check exactly. Speed is the expensive check. A line's timings come from its own harness, often in C, and [those have misled us](2026-11-21-fast-alone-slow-in-a-crowd.md) before. So nothing is adopted on them. We port to native Rust, check on CI, and judge on the i9 under a rule registered before the data. The [fidelity ledger](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger) records each result that did not transfer, and every line reads it.

That makes the i9 the bottleneck of the whole search. It is one machine, its runs are noisy, and it needs its owner at home. Mathematics can now propose candidates faster than we can judge them. The next piece of infrastructure is therefore a contention harness on CI, which ranks candidates under shared-cache load before any reaches the i9. It is a condition for pursuing the wormhole, not a substitute for it.

## 7. What we are searching for next

**Lever 8 failed, and showed where the difficulty lives.** If the output near a row were fixed by the runs near it, a catalogue of precomputed fragments could replace the walk, as local Langlands lets the proof work one prime at a time. [We registered a rule](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-local-factor-census.md) before counting: at least 50% of the rows the walk visits must have output fixed by their local shape, with a catalogue of at most 256 KiB. The best figure over the six blocks was 10.5%. The bytes a row emits next are its own sorted prefix, fixed by its place in the global order, not by the runs around it. The difficulty is global, so any better route has to carry global order more cheaply than the walk does.

**The open front is the floor under the decoder.** Once each worker's buffers fit the cache, what remains? Mechanisms measures it: serial loads that hit L3 after missing L2, about 40% of all cycles. That cost is the walk itself, one dependent load after another, and no reduction in memory touches it. Lever 4 says what to do. Bound it from below, for every exact decoder in a stated machine model, and from above, by what the reduced kernel pays in the same model. If the bounds meet, the walk is optimal. If they do not, the gap says what a better route must do. Wormhole's second round is working on exactly this.

The candidates for that better route are representations of global order that are not the LF walk: Φ with its toehold samples from compressed indexes, or several independent walk chains in flight at once. Finding one would be the wormhole proper: not a faster version of the primitive we have tuned for months, but a different route from the runs to the text, assembled from mathematics built for something else.

---

**Sources:**
[Levent Alpöge's remark](https://alpo.ge/remark.pdf);
the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis" (20–21 November 2026), for the kernel checks and Chris Birkbeck's overview;
[a source-linked atlas](https://nasqret.github.io/riemann-rev/atlas/) of the development, by an independent reading project.

**Research records:**
[the twelve levers](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md) (#686);
[the local-factor census](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-local-factor-census.md) (#687);
[Kitchen Sink's launch record](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/launches/2026-11-20-carbon-kitchen-sink.md);
handoffs from [Mechanisms, cycle 16](https://github.com/lmmx/mibz2/commit/d42b7fb93c57cc82ec6bfe938d48e209e39e265c),
[Wormhole, cycles 1–5](https://github.com/lmmx/mibz2/commit/dea7ca38a8f006bdd3931b3962245bb77592c7bd) and
[Kitchen Sink, cycles 11–15](https://github.com/lmmx/mibz2/commit/b69d74af87969168e9298551d5e08e2e1196ec15);
[the fidelity ledger](https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger).
