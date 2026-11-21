[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-690/blog/index.md>)

# Besieging hard problems: routes through distant mathematics, from the Riemann proof to the inverse BWT

The Riemann proof reached its goal through mathematics built for other problems, and four AI research lines that tried this on a decoder found its bottleneck.

21 November 2026 · 17 min read · [Meta](<#7-what-we-are-searching-for-next>)

Shipped in mibz2 [0.5.10](<https://lmmx.github.io/mibz2/pr-preview/pr-690/benchmarks/history/#release-0-5-10>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0510---2026-11-21>))

On 20 November 2026, a proof of the Riemann Hypothesis was accepted by three independent proof checkers. It is about 17 million lines of Lean, found by Levent Alpöge working with Claude. Its strangest feature is not its size but its route. It does not build new theory towards the goal. It passes through a long chain of existing results, each made for some other problem, and arrives somewhere a direct route could not reach. This post is about that route, and about an experiment. We gave the same mandate, to use everything, to four AI research lines. Their problem was our own: the inverse Burrows–Wheeler transform at the heart of mibz2's bzip2 decoder.

**The takeaways:**

- **The achievement is a new mode of invention, not only a large proof.** A number theorist described it as a wormhole through "the infrastructure built for a hundred other mineral reserves". The infrastructure the proof needs, from resolution of singularities to prismatic Dieudonné theory, is proved inside it, with no axioms. Three independent kernels accepted it before anyone could say how its main ingredients combine.
- **The route can be described as twelve levers.** We read them in the proof's reported shape. The target is strengthened into a statement about a family. A forbidden event is excluded by bounds that independent theories place on one quantity. The work moves to where those bounds exist, and every foundation is built rather than assumed. This is our interpretation, not the authors'.
- **We ran the experiment.** Since 20 November, four Gemini research lines have attacked the inverse BWT. One of them, Kitchen Sink, was told not to require a field of mathematics to justify its relevance in advance. Their chains ran through graph theory, discrete Morse theory, interval-exchange dynamics, ergodic theory, queueing theory and compiler register allocation.
- **The chains broke informatively, and three of them converged.** Most produced exact mathematics and poor algorithms; each break named the next link to look for. This morning Mechanisms, Wormhole and Kitchen Sink independently identified the same bottleneck. Each worker holds about 4.5 MiB of scratch, which overflows the processor's shared cache once 7 to 10 workers run. Their candidate fixes act on different buffers, and two of them combined: ported natively and judged on our i9, they made mibz2 0.5.10 9% faster at ten threads.
- **The pace is set by checking.** Exact correctness checks are cheap, so the search can be broad. Checks of speed on the real machine are not, and they decide how fast any of this can be adopted. The open question is the floor: the cost that remains once memory fits, and whether a different route from runs to text order can avoid it.

[Section 1](<#1-the-problem-and-what-was-achieved>) shows what was achieved, with the overview that started this. [Section 2](<#2-the-wormhole-hypothesis>) states the wormhole hypothesis, and [section 3](<#3-the-twelve-levers>) the levers that make such a route work. [Section 4](<#4-we-tried-it-on-a-decoder>) describes the experiment, [section 5](<#5-what-happened-when-we-used-everything>) what it found, [section 6](<#6-the-constraint-that-sets-the-pace>) the constraint that governs it, and [section 7](<#7-what-we-are-searching-for-next>) what comes next.

## 1\. The problem, and what was achieved

The Riemann Hypothesis was posed in 1859. It says that the nontrivial zeros of one function, which governs how the primes are spread, all lie on a single line. It has resisted every attempt for more than 160 years, and most mathematicians assumed that a proof would need large bodies of theory not yet built.

The Lean development proves it, together with its generalisation to Dirichlet L-functions, which carry the same question for primes in arithmetic progressions. It does not cover the wider Selberg class. Members of the Lean community ran it through three independent kernels, nanoda, con-leche in its verified mode, and pestka, and each accepted it. Alpöge's own [remark](<https://alpo.ge/remark.pdf>) describes a toy proof that guided the search; that toy is not in the Lean. He also describes generated exposition running past 250,000 pages, which he chose not to present as a readable proof.

A kernel's acceptance is strong evidence that the formal theorem follows from its foundations. It is not yet the mathematical community's review of the argument, which has only begun. **We have not checked the proof,** and nothing in this post depends on it being right. What we use is its reported shape. The clearest account of that shape is the overview Chris Birkbeck posted on the Lean Zulip thread, written with Fable's help:

Chris Birkbeck's overview, posted in the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis", 21 November 2026.

Laid out as a structure, the overview describes a proof that reaches its target from far away. Each block is an existing body of mathematics, developed for some other purpose:

The proof's shape, as Birkbeck's overview describes it

Arrows point from what is used to what it proves. The blocks follow Birkbeck's five points; which foundation feeds which bound is simplified here. Birkbeck's own words on the three-way join: "I have not yet understood how these combine to exclude zeros."

## 2\. The wormhole hypothesis

A number theorist, writing on social media, gave the reading that this post is built on. A sufficiently deep problem is like a mineral reserve buried miles underground, and the usual way to reach it is to build the infrastructure for it, painstakingly, over decades. This proof did something else:

> It takes an incredibly complicated tour through the infrastructure built for a hundred other mineral reserves, situated far from the destination, and somehow create\[s\] a wormhole to the original destination. … no human brain, on its own, is capable of creating a wormhole that requires a 25,000 page long tour through existing infrastructure with no obvious or realistic connection to the goal.

Two remarks relayed to us made the consequence plain. Alex Kontorovich is reported to have summed the proof up as "RH is proved by using *everything*!", and Igor Carron to have observed that throwing the whole kitchen sink at a problem has never been easier.

The hypothesis we took from this is about invention, not proof checking. **Some problems may be hard not because their solution needs an idea no one has had, but because the only routes to it pass through mathematics whose relevance cannot be seen from the start, and are too long for people to assemble.** If that is so, the way to attack such a problem is to search across distant fields on purpose, accept routes of any length, and check every step exactly. Whether this transfers from number theory to an algorithmic problem is an empirical question. Section 4 describes how we set out to test it.

## 3\. The twelve levers

What makes a long detour useful rather than merely long? Read as strategy, Birkbeck's overview suggests a set of moves, which we call levers. Our [research note](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) derives them. Where a lever names a classical pattern, such as Wiles's numerical criterion or Deligne's tensor-power trick, that is our reading, not something the overview states.

They are not independent. Strengthening the statement (1) is what lets bounds from different theories apply to it (4). Those bounds often exist only in another setting, which forces the move (7). The move is then paid for by building its foundations (9). The breadth of that search is affordable only because each step is cheap to check (10).

The third column gives the corresponding move in our research lines, where there is a genuine one. These are analogies and applications, not claims about how the proof works.

|  | Lever | In the proof | In our research lines |
| --- | --- | --- | --- |
| **I** | **1\. Strengthen the target until your tools preserve it** | Not RH alone, but nonvanishing for every Dirichlet L-function, a family closed under twisting and base change | Claims stated for all small cases and every authenticated block, not one kernel on one block |
|  | **2\. Prove a growth rate, not a value** | Bounds compared along a tower of cyclotomic fields | Not yet attempted: scaling of dependent loads with block size |
|  | **3\. Use robust inequalities in both directions** | Hilbert–Samuel, bigness and vanishing theorems are inequalities | Registered intervals on the i9, never point estimates |
| **II** | **4\. Find a quantity two independent theories can each bound** | A Selmer-like cohomology group, bounded by Galois theory and by geometry (our reading) | Bytes resident per worker, bounded by Kitchen Sink's contention law and by Wormhole's cache model |
|  | **5\. Amplify a defect until coarse tools can see it** | A single zero pushed through a family (our reading) | Kitchen Sink's 2-adic conjugation of LF (cycle 13): no amplification, as hard as the answer itself |
|  | **6\. Reach for positivity** | Theta integrals and volumes, which cannot be negative | Not yet found |
| **III** | **7\. Move to a setting where the tools exist, then come back** | Base change and theta lifts to unitary groups, whose Shimura varieties carry geometry | The kernel recast as a discrete Morse collapse; runs as an interval exchange |
|  | **8\. Push difficulty to the local steps, where classification is complete** | Local Langlands at each prime | Failed: the output depends on global order (section 7) |
|  | **9\. Re-optimise the route when the cost function changes** | "Not the route in the literature": a cited black box costs its full formalisation | From instructions per byte to bytes resident per worker (section 5) |
| **IV** | **10\. Make checking cheap and certain, then search widely** | Kernel checks: certain, and cheap next to discovery | Exact verifiers on tens of thousands of cases make Kitchen Sink's breadth affordable |
|  | **11\. Use a toy to find the missing ingredient, then drop it** | The remark's toy proof is not in the Lean | Toy blocks where inversion is free, used only to name what must transfer |
|  | **12\. Separate trust from understanding** | Accepted before anyone understood the three-way join | Adopt on registered measurement; mechanisms stay hypotheses |

## 4\. We tried it on a decoder

mibz2 is a bzip2 decoder in Rust. The core of decoding is inverting the BWT: rebuilding each 900 kB block from its sorted form. The textbook method walks the LF mapping, one dependent memory load per output byte, and that walk dominates decoding time. Since early November we had been searching for faster routes.

On 20 November, after Alpöge's announcement, we reorganised the search around the wormhole hypothesis. Each research line is a Gemini session with its own branch, brief and journal. The brief for Kitchen Sink, the line created for this purpose, reads in part:

> Decide which mathematical frameworks, theorems, analogies, constructions and combinations deserve investigation. Range as widely across mathematics as your judgement suggests, including fields with no established connection to BWT, compression or algorithms. … Do not require every mathematical framework to justify its relevance in advance. Discovering that relevance is part of your research.

That is from its [launch record](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/launches/2026-11-20-carbon-kitchen-sink.md>). The four lines differ in purpose:

- **Mechanisms** builds reconstruction mechanisms and times them against the shipped kernel.
- **Kitchen Sink** is the breadth engine, trying bridges from about 25 fields per cycle.
- **Depth** priced the work and sought lower bounds. It has finished, and [our audit](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-20-carbon-depth-audit.md>) found its bounds rest on asserted constants, so they are not yet floors.
- **Wormhole**, launched after the number theorist's description, looks for one long chain of existing results, each built for something else, ending in a cheap operation.

Breadth without checking would produce only noise, so the lines work inside a discipline:

The siege: four research lines, their own checks, and ours

Each line is a Gemini session with its own branch and brief; a cycle ends in a journal with its programs. The lines claim; our checks decide.

The lines' theorems are exploratory proofs: written out, then checked by exhaustive enumeration of small cases and byte-exact decoding of six authenticated Wikidata blocks. That makes them machine-tested claims. None has been independently reviewed as mathematics, and none is adopted on a line's own timings.

## 5\. What happened when we used everything

**Most chains produced exact mathematics and poor algorithms, and the breaks were the useful part.** Wormhole's five cycles show the pattern best. Each cycle took a result from a different field and made it work on the decoder exactly. Each then broke on a cost, and the break named the next link to look for:

Wormhole's chain: five fields, each break naming the next link

Each cycle's construction is exact: checked on 26,495 small cases and the six authenticated blocks. The citations are those the line gives. The final figures come from its own C harness. Natively, W1 alone was time-neutral; combined with Mechanisms' OutQ16 it shipped in 0.5.10 (section 5).

Kitchen Sink's feedback-set result shows the same arc within one idea. It found that breaking only 0.3% of runs makes the run-overlap graph acyclic, which suggested an exact schedule with no walk at all. In cycle 11 it built that schedule and proved it exact. Then it measured the cost. Even with every cut run filled for free, 64 to 76% of the remaining fragments still deadlock in copy cycles, because rank inversions braid them together inside a graph with no cycles. Resolving them took 8 to 30 times the shipped kernel's interval splits and 3 to 5 times its memory. A striking simplification, followed by a precise account of why it does not become a fast algorithm: that is what the search mostly yields, and the record of it is what keeps the search from repeating itself.

**Then, on the morning of 21 November, three lines arrived at the same place.**

- **Kitchen Sink** ([cycles 11 to 15](<https://github.com/lmmx/mibz2/commit/b69d74af87969168e9298551d5e08e2e1196ec15>)) audited the shipped kernel and found its 256 rank queues hold 55 to 59% of its memory. It then combined queueing theory with a model of shared-cache spill into a contention law. That law retrodicts our past i9 results, including an eight-lane walk that ran 13% faster alone and [5% slower among eight workers](<https://lmmx.github.io/mibz2/pr-preview/pr-690/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>). It also predicts that changes which shrink the working set do not reverse under load.
- **Wormhole** ([five cycles](<https://github.com/lmmx/mibz2/commit/dea7ca38a8f006bdd3931b3962245bb77592c7bd>)) was led there by its third break. Each worker holds 4.66 MiB of scratch, so seven workers need 37.9 MiB against the i9's 20 MiB shared L3.
- **Mechanisms** ([cycle 16](<https://github.com/lmmx/mibz2/commit/d42b7fb93c57cc82ec6bfe938d48e209e39e265c>)) modelled 3.4 bytes of memory traffic per output byte at ten workers. Each worker's buffers no longer fit its share of the cache.

None of the three was asked to look at memory. Each reached it from different mathematics, and that is lever 4 appearing on its own: a quantity, bytes resident per worker, that independent lines of reasoning bound. It is also lever 9. Our kernels had been tuned for instructions per byte, the cost that matters on one core. With ten workers sharing one cache, the cost that matters is bytes resident, and the best route changes with it.

Their candidate fixes differ, and act on different buffers:

Where each line's cut lands in the Open kernel's per-worker memory

Sizes are the lines' own measurements on the six authenticated blocks. The tricks are of one kind: a buffer is dead for part of each block, and something else lives in it then. Mechanisms' ablation showed that the gain comes from reusing the output buffer's cache lines, not from the chunking: the same chunks in a separate buffer lost it.

What decides the outcome on the i9 is whether a worker's buffers fit its share of the 20 MiB L3: 2.86 MiB with seven workers, 2.0 MiB with ten.

Per-worker memory against each worker's share of the i9's L3

Scratch figures are each line's own harness measurements: Mechanisms' count of the shipped kernel (Wormhole measured 4,663 KiB). Measured natively, the shipped kernel held 4,236 KiB, and W1 with OutQ16 held 1,200 KiB. A worker does not touch every byte in each block, so the working set is smaller than these totals; Mechanisms measures 1,846 KiB on average for SwarOutQ16, inside even the ten-worker share.

**Three things are established, and one is not.** The lines independently identified the same bottleneck. They proposed reductions that act on different parts of it, each exact on its checks. Their own harnesses show gains that widen under contention: Wormhole's C harness ran 0.93 to 0.97 of the shipped kernel's time with four workers against 0.95 to 0.98 on one thread. That is the reverse of the eight-lane failure. **The fourth, that the reductions compose natively, was then tested.** We ported Wormhole's W1 and Mechanisms' OutQ16 to native Rust (\#692). Kept scratch per worker fell from 4,236 to 1,200 KiB. Some C claims did not transfer: W1 alone was time-neutral, and the SWAR iota gained nothing over what the compiler already does. On the i9, under a rule registered before the run, the combination ran at 0.952, 0.917 and 0.931 of the shipped decoder's time at 6, 8 and 10 workers, with about 10% less peak memory. It shipped as mibz2 0.5.10, whose release scaling against 0.5.9 is +6.5% at 8 threads, +9.0% at 10 and +5 to 7% at 12 to 20, with nothing changed below 8. A recalibration run found that no worker-count trim needed to move, as MONODROMY's frozen prediction said. The first production gain of the experiment came from two lines' ideas combined, at exactly the counts where the cache is shared.

## 6\. The constraint that sets the pace

The experiment's breadth rests on lever 10. A line can try twenty fields in a cycle because each attempt is cheap to check exactly. Speed is the expensive check. A line's timings come from its own harness, often in C, and [those have misled us](<https://lmmx.github.io/mibz2/pr-preview/pr-690/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>) before. So nothing is adopted on them. We port to native Rust, check on CI, and judge on the i9 under a rule registered before the data. The [fidelity ledger](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>) records each result that did not transfer, and every line reads it.

That makes the i9 the bottleneck of the whole search. It is one machine, its runs are noisy, and it needs its owner at home. Mathematics can now propose candidates faster than we can judge them. The next piece of infrastructure is therefore a contention harness on CI, which ranks candidates under shared-cache load before any reaches the i9. It is a condition for pursuing the wormhole, not a substitute for it.

## 7\. What we are searching for next

**Lever 8 failed, and showed where the difficulty lives.** If the output near a row were fixed by the runs near it, a catalogue of precomputed fragments could replace the walk, as local Langlands lets the proof work one prime at a time. [We registered a rule](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-local-factor-census.md>) before counting: at least 50% of the rows the walk visits must have output fixed by their local shape, with a catalogue of at most 256 KiB. The best figure over the six blocks was 10.5%. The bytes a row emits next are its own sorted prefix, fixed by its place in the global order, not by the runs around it. The difficulty is global, so any better route has to carry global order more cheaply than the walk does.

**The open front is the floor under the decoder.** Once each worker's buffers fit the cache, what remains? Mechanisms measures it: serial loads that hit L3 after missing L2, about 40% of all cycles. That cost is the walk itself, one dependent load after another, and no reduction in memory touches it. Lever 4 says what to do. Bound it from below, for every exact decoder in a stated machine model, and from above, by what the reduced kernel pays in the same model. If the bounds meet, the walk is optimal. If they do not, the gap says what a better route must do. Wormhole's second round is working on exactly this.

The candidates for that better route are representations of global order that are not the LF walk: Φ with its toehold samples from compressed indexes, or several independent walk chains in flight at once. Finding one would be the wormhole proper: not a faster version of the primitive we have tuned for months, but a different route from the runs to the text, assembled from mathematics built for something else.

---

**Sources:** [Levent Alpöge's remark](<https://alpo.ge/remark.pdf>); the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis" (20–21 November 2026), for the kernel checks and Chris Birkbeck's overview; [a source-linked atlas](<https://nasqret.github.io/riemann-rev/atlas/>) of the development, by an independent reading project.

**Research records:** [the twelve levers](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) (\#686); [the local-factor census](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-11-21-local-factor-census.md>) (\#687); [Kitchen Sink's launch record](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/launches/2026-11-20-carbon-kitchen-sink.md>); handoffs from [Mechanisms, cycle 16](<https://github.com/lmmx/mibz2/commit/d42b7fb93c57cc82ec6bfe938d48e209e39e265c>), [Wormhole, cycles 1–5](<https://github.com/lmmx/mibz2/commit/dea7ca38a8f006bdd3931b3962245bb77592c7bd>) and [Kitchen Sink, cycles 11–15](<https://github.com/lmmx/mibz2/commit/b69d74af87969168e9298551d5e08e2e1196ec15>); [the fidelity ledger](<https://github.com/lmmx/mibz2/blob/master/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>).
