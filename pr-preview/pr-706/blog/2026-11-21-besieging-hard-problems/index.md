[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-706/blog/index.md>)

# Besieging hard problems: cheap checks first, after the Riemann proof

A machine-checked Riemann proof shows how far a search can range once checking is cheap, so ours now begins by making speed cheap to check.

21 November 2026 · 16 min read · [Meta](<#6-what-comes-next>)

Shipped in mibz2 [0.5.10](<https://lmmx.github.io/mibz2/pr-preview/pr-706/benchmarks/history/#release-0-5-10>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0510---2026-11-21>))

mibz2 is a Rust decoder for bzip2, and for months we have searched for faster ways to undo the reordering at its core. This post describes how we are rebuilding that search, and why. A proof of the Riemann Hypothesis, built at a scale no person could write and accepted by three independent checkers, has shown what a search can attempt once checking is cheap. Our checks of correctness are already cheap. Our checks of speed are not, so that is where we start.

On 20 November 2026, Levent Alpöge [announced](<https://alpo.ge/remark.pdf>) a proof of the Riemann Hypothesis, found working with Claude and written out in full in Lean, a language whose proofs a small program called a kernel can check line by line. By the end of that day, members of the Lean community had reported that three independent kernels, nanoda, con-leche in its verified mode and pestka, each accepted the development. It is about 17 million lines of Lean v4.33.1 in 175,190 files. It proves the Riemann Hypothesis, and its generalisation to Dirichlet L-functions, a family of functions that carry the same question for primes in arithmetic progressions. It does not cover the wider Selberg class.

The Riemann Hypothesis was posed in 1859. It says that the zeros of one function, which governs how the primes are spread out, all lie on a single line. It has been the most famous open problem in mathematics for more than a century, and generations of the strongest mathematicians have failed to settle it. A proof of it, of any length, would be an event. This one is checked by machines, and built at a scale no person could write or read. That raises a second question beside the mathematics. What kind of method makes such a thing possible, and what can those of us working on smaller hard problems take from it?

**We have not checked the proof.** We are not number theorists, and nothing below depends on the argument being right. What we use is its reported shape, chiefly the overview that Chris Birkbeck posted on the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis". A formal check is also not the same as human understanding. Birkbeck himself wrote: "I have not yet understood how these combine to exclude zeros." Where this post reads a mechanism into the proof, it says so.

**The takeaways come first.**

- **The economics of difficulty have changed.** For as long as mathematics has existed, a proof had to be short enough for people to find, check and explain. A route that was valid but needed tens of thousands of pages was out of reach. When machines can build and check such routes, length stops being the dominant constraint.
- **The method is a sequence of moves, not one insight.** We read twelve of them in the proof's shape, in four stages. The target is strengthened into a statement about a whole family. The forbidden event is excluded by bounds from different fields on a quantity they can each constrain. The work moves to a richer setting where those bounds exist, and the foundations the route needs are built, not assumed.
- **Search can be wide only where checking is cheap.** A kernel's verdict is certain and costs little next to discovery, so the search can afford strange routes. Discovery, verification and understanding become three jobs, done at different times.
- **For mibz2, a fast decoder for bzip2, the binding constraint is performance checking.** Our correctness check is cheap and certain; our performance check is scarce and has misled us. Faithful, cheap measurement under realistic load comes before any wider search for new algorithms.
- **The research question sharpens.** We want a defined cost model in which lower and upper bounds on decoding can meet. Our latest negative result points at global structure, and the right primitive may not be the one we have spent months tuning.

[Section 1](<#1-what-was-achieved>) sets out what was achieved and why its scale matters. [Section 2](<#2-how-is-a-problem-changed-before-it-is-solved>) describes how the problem was changed before it was solved, and [section 3](<#3-why-does-the-route-look-so-strange>) why the route looks strange to specialists. [Section 4](<#4-why-must-checking-be-cheap-before-search-can-be-wide>) covers the role of cheap checking, and the three moves that decide the rest. [Section 5](<#5-what-does-this-mean-for-mibz2>) applies all of it to mibz2, and [section 6](<#6-what-comes-next>) says what comes next.

## 1\. What was achieved

Seventeen million lines is hard to picture. Mathlib, the community library on which most Lean mathematics is built, took years of work by hundreds of contributors to reach a fraction of that. The development uses no axioms beyond Lean's own, so every theorem it relies on is proved inside it. That includes a large part of modern arithmetic geometry, which most papers in the field cite rather than reprove.

Birkbeck's overview gives the structure in five steps, paraphrased here:

1. The Riemann Hypothesis follows from the statement that no Dirichlet L-function vanishes where the real part of \\(s\\) exceeds \\(\\tfrac12\\). This step is short and standard.
2. That non-vanishing follows from three things. One is a growth bound on certain Galois cohomology groups (objects that measure how far a number-theoretic equation fails to have solutions), resembling Tate–Shafarevich groups, over a family of subfields of cyclotomic fields. The second is a covolume bound from arithmetic geometry. The third is a set of statements about theta integrals on unitary groups, with the local Langlands correspondence for \\(\\mathrm{GL}\_n\\).
3. The cohomology bound is an automorphy-lifting argument. It uses Galois representations attached to regular algebraic, polarized, cuspidal automorphic representations of \\(\\mathrm{GL}\_n\\) over CM fields. Base change, Jacquet–Shalika, Clozel's purity theorem, Bernstein–Zelevinsky, the theory of complex multiplication and Deligne–Ribet are all used along the way.
4. Those Galois representations are not assumed. The development builds them by a geometric construction for unitary groups, with theta lifts between groups. As far as Birkbeck can tell, this is not the route in the literature.
5. Because nothing is assumed, the infrastructure is proved inside too: resolution of singularities in characteristic zero, the arithmetic Hilbert–Samuel theorem, Moriwaki's bigness theorem, toroidal compactifications of Siegel varieties, Kawamata–Viehweg and Nadel vanishing, Grothendieck–Messing theory and prismatic Dieudonné theory.

Each item in step 5 is a major theorem. Several took their original authors years, and some have never been formalised before in any system. Here they are scaffolding, built because the route needed them.

The announcement also describes more than 250,000 pages of generated exposition, which Alpöge chose not to present as a readable proof, and he expects major simplifications. One number theorist, writing on social media, put the strangeness this way: the proof "takes an incredibly complicated tour through the infrastructure built for a hundred other mineral reserves, situated far from the destination, and somehow create\[s\] a wormhole to the original destination." A problem many had assumed would need new theory in arithmetic geometry may instead yield to a long chain of existing, deep results.

A reading project has since published [a source-linked atlas](<https://nasqret.github.io/riemann-rev/atlas/>) of the development, which maps its components to pinned lines of the Lean source and marks which cuts of the argument have been checked separately. Its own summary describes a one-sided bound on sums over primes, supplied by the arithmetic, which analysis turns into a region free of zeros. That is the atlas authors' reading, and we have not checked it either.

**Why the scale matters beyond mathematics.** The usual picture of a great proof is a single insight that makes a hard problem easy. This development may contain such an insight; no one can yet say. What it certainly contains is an enormous amount of machinery assembled around the problem until the conclusion could not be avoided. That suggests some problems were hard not because the necessary idea was out of reach, but because every known route was far too long for people to build and check. If so, the limits on what can be solved were partly limits of human logistics. They were set by how many concepts one mind can hold, how long an argument a community can review, and how much foundational work its attention can pay for.

## 2\. How is a problem changed before it is solved?

Read as strategy, the overview suggests twelve moves, which [our research journal](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) sets out in four stages. They choose the statement, then the quantity the proof turns on, then the route, and finally run the search. They are our interpretation of the proof's shape, not established descriptions of its mechanism. Where a move names a classical pattern (Deligne's amplification, a Bloch–Kato-style link between cohomology and L-values), that goes beyond what Birkbeck's overview says. As principles for organising a hard search, they stand on their own.

Twelve moves in four stages: our reading of the proof's shape

Conceptual, and our interpretation: not a dependency diagram of the proof, and not a description its authors have confirmed. The highlighted moves, 4, 7 and 10, decide the others (section 4).

### Choosing the statement

**The statement is strengthened until the tools preserve it.** The development does not attack the Riemann Hypothesis alone. It proves non-vanishing for every Dirichlet L-function, across families of cyclotomic fields. A stronger statement can be easier to prove when the family is closed under the operations the tools need: twisting by characters, base change up a tower of fields, lifts between groups. A single object gives an induction nothing to work with. Pólya called this the inventor's paradox. The original problem is the destination, not necessarily the place where the work is done.

**A growth rate is proved, not a value.** Across a tower of fields, a hypothetical zero would make an obstruction grow at a measurable rate. Comparing exponents of growth is robust, since exponents survive the constants and error terms that swamp a comparison at any single level.

**Inequalities replace formulas.** The infrastructure in step 5 (Hilbert–Samuel, bigness, Kawamata–Viehweg and Nadel vanishing) consists of robust inequalities. No exact formula is needed. A chain of inequalities is long but local, since each link can be checked alone. That is part of why seventeen million lines can be checked although no one holds the whole argument.

### Choosing the quantity

**Two unrelated theories must be able to measure the same thing.** This is the most important move. Constructing a mechanism that rules out zeros directly is hard. The alternative is to find a quantity that a hypothetical zero would change, and to constrain it from independent directions. Birkbeck's step 2 names three bounds of different kinds, from algebra, from geometry and from harmonic analysis. Our reading is that they meet like a vice: one bounds a quantity from above, another from below, and a zero would need a value between them that does not exist. One theory gripping the quantity gives a bound; two give a squeeze.

**A defect is amplified before it is measured.** A single zero may be invisible to coarse tools. Pushed through a family, a tower or a repeated construction, it can produce enough obstructions to violate a bound. Deligne's tensor-power argument is the classical model: bound \\(|\\alpha|^{2k}\\) for every \\(k\\), and \\(|\\alpha|\\) itself is controlled.

**Quantities that cannot be negative are sought out.** Theta integrals, norms and volumes are manifestly non-negative. If a counterexample would force such a quantity below zero, or below a volume, the counterexample is impossible.

The difference between these ways of working and a direct construction matters. A construction gives an example. Bounds that meet give finality: if every zero, or every decoder, must satisfy constraints that leave no room, the question is closed.

## 3\. Why does the route look so strange?

The third stage chooses the route, and here the development departs furthest from the literature.

**The route is planned backwards from where the tools work.** Dirichlet L-functions have no geometry of their own. Unitary groups do: their Shimura varieties are spaces with Frobenius maps, cohomology and notions of positivity, where powerful bounds exist. The route runs from that setting back to a chain of transfers (base change, theta lifts) that reaches the original problem. The transfers are the cost, and the destination is the point. In other fields the same move has familiar forms: solving an optimisation problem through its dual, replacing a data structure with one that exposes the needed order, or giving a compiler a new intermediate representation. The richer setting may be far more complicated than the original. Complexity that does no work is the problem, not complexity as such.

**Difficulty is pushed to local problems that are completely classified.** Local Langlands, Clozel's purity theorem and Bernstein–Zelevinsky classify everything at a single prime. The global argument then combines local contributions. This works only where a complete local classification exists, and section 5 describes a case of ours where it does not.

**The route is re-optimised when the cost of proof changes.** Birkbeck's remark that the construction of the Galois representations is "not the route in the literature" follows from the economics. The literature's routes are optimised for human cost. They are short, and they cite large results proved elsewhere, such as Harris and Taylor's construction of Galois representations, as single steps. In a development with no axioms, every cited result costs its full formalisation. The cheapest route then changes: it may have more pieces, each simpler to build, and pass through constructions a specialist would never choose.

So a route that looks alien may reflect a different cost function, not a worse one. When the cost of a step changes by orders of magnitude, the best architecture of the whole argument changes with it. An established architecture should never be assumed optimal after the costs of implementation, memory, parallelism or verification have moved. For us that rule is concrete: the algorithm at the heart of every bzip2 decoder was designed for one core with cheap random access to memory, and that is not the machine we run on.

## 4\. Why must checking be cheap before search can be wide?

The fourth stage runs the search. A kernel's verdict on a proof is certain, and it costs little next to the work of finding the proof. The thread records the costs. Building the development took just over an hour on eight 192-core machines. nanoda then checked the exported proof, 61.57 GB of it, in 27 minutes on 80 threads; con-leche took under four minutes on 64 cores, and pestka about nine on 96. A proof that took a long campaign to find was checked, three times over, in an afternoon. Accepting a step does not depend on anyone's judgement. That asymmetry is what lets the search take risks: unintuitive routes, long compositions of results, steps no one has yet understood. A search of that kind pays only where checking is much cheaper than generating, so verification has to be built before the search is widened.

Two further moves follow. A simplified problem is used to find the missing ingredient and is then left behind. The toy proof in Alpöge's remark guided the discovery, but it is not part of the Lean development. His remark describes the transfer: an operation that makes the easy setting work, multiplicity in a Stepanov-style argument, has no obvious counterpart over the integers, and iterated theta lifts were proposed to reproduce its effect. What transfers is the effect, not the operation. And correctness is separated from understanding, so that three jobs once done together can be done apart.

Three jobs, once bundled, now done apart

Conceptual, not a dependency diagram of the proof. Above: when people do all three jobs, a route is viable only if it is short enough to find, check and explain at once. Below: a machine search can try long routes, independent kernels check the result, and understanding becomes later work. In the Riemann development, three kernels accepted it before anyone could say how its three bounds combine.

**Discovery** finds a route that works, however strange. **Verification** checks every step independently of whoever found it. **Understanding** compresses the route into ideas a person can hold, and it can now come afterwards, as a research project of its own. The construction may even be the raw material from which a short proof is later extracted. Rigour is not relaxed: every step is checked. What is relaxed is the demand that a person grasp the whole before it is accepted. We may increasingly have correct solutions before we know why they are the right ones.

**Three of the twelve moves decide the rest.** The fourth says what must be constrained: a quantity that two independent theories can each bound. The seventh says where those constraints can be obtained: a setting reached by planning backwards. The tenth makes searching the resulting space of routes affordable: checking that is cheap and certain. Together they describe a different kind of enterprise from thinking harder about a fixed problem. It engineers the conditions under which the problem becomes solvable, and then searches them.

## 5\. What does this mean for mibz2?

mibz2 is a Rust decoder for bzip2. Most of its time goes on undoing the Burrows–Wheeler transform (BWT), a reordering of each block of up to 900 kB that groups similar bytes together so they compress well. Undoing it is classically done by following a map called LF from one row of a table to the next. Each step's address depends on the result of the previous step, so the walk is a chain of dependent memory accesses, and every miss in the cache stalls it. Our recent posts describe the decoder's two ways of running that walk and how they behave on many cores ([Fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/pr-preview/pr-706/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)). The project's [earlier note on research by obligation](<https://lmmx.github.io/mibz2/pr-preview/pr-706/blog/2026-11-20-research-by-obligation/index.md>) described how the announcement changed the way we search.

We are not proving theorems, and we should not transplant mechanisms we cannot yet explain. We can adopt the method. The [research journal's note on the proof's shape](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) asks of each of the twelve moves what it would mean for us. Four consequences follow.

### The binding constraint is performance checking

Our correctness check is already like a kernel's. A candidate decoder must reproduce every byte of the original (parity), pass Miri, Rust's checker for undefined behaviour, and survive fuzzing. All of that runs on CI in minutes, and its verdict is close to certain.

Our performance check is the opposite. The authoritative measurement is a run on one ten-core desktop machine, an Intel i9-10900K, which a person has to start, and which can run one experiment at a time. The cheaper proxies have misled us repeatedly. Our [fidelity ledger](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>) records, for each candidate that reached a native test, what the cheap prediction said and what happened. A selection from it:

| Candidate | Cheap prediction | Native outcome |
| --- | --- | --- |
| Packed queues (\#654) | 8–15% faster | 4.6–9.3% slower |
| A backward pass through the table | 26–28% less work | 10–15% slower on all six blocks |
| Eight interleaved lanes (\#675) | 13% faster on one thread, natively | 5% slower at 8 threads on the i9 |
| A cheaper elimination step alone (\#679) | about 5% faster on one thread | faster at 6 and 8 threads, inconclusive at 10 |

The eight-lane candidate is the clearest case. It made the kernel 13% faster on one thread because it keeps up to eight memory lookups waiting at once. On the i9 with eight workers sharing one cache and one path to memory, that capacity was already in use, and the gain reversed.

Where our search is limited: the cost of each check

Conceptual. Generating candidates and checking their correctness are both cheap. The check that decides adoption, the i9 run under realistic load, is scarce, and the cheap proxies before it have often ranked candidates wrongly (the fidelity ledger above). Widening the search at the top only lengthens the queue at the bottom.

One distinction from the proof helps here. The complexity of discovering a principle is different from the complexity of executing it. A very long argument can establish that a simple decoding operation is valid, and a short theorem can justify a computationally terrible algorithm. Our checks must judge the second, whatever the first cost.

If we can generate a hundred candidate algorithms but measure only a handful faithfully, our limit is not creativity but experimental throughput. **So the first investment is the speed at which bad ideas are rejected,** not the speed at which ideas are produced. That means a harness on CI that runs the real Rust code with several workers decoding at once, under the shared-cache contention production has. A candidate's behaviour in a crowd is then screened before it reaches the i9. Research lines also hand over patches against the real code rather than re-implementations, so that every stage checks the same artefact, as the three kernels did.

### Bounds that meet need a defined cost model

Today we race algorithms against each other. The proof's second move suggests asking a different question: how close are we to the best that any decoder could do?

Let \\(D(x, A)\\) be the number of serially dependent memory accesses that decoder \\(A\\) makes on input \\(x\\). A meeting of bounds would show two things. First, a lower bound \\(L(x) \\le D(x, A)\\) that holds for every decoder \\(A\\) in a precisely defined model of computation. Second, a construction \\(A^\\ast\\) with \\(D(x, A^\\ast) \\le U(x)\\). Where \\(L\\) and \\(U\\) are close, we would have an explanation of near-optimality, not another benchmark winner. Where they are far apart, the gap says what remains unexplained.

Bounds that meet: what any decoder must pay, against what our best decoder pays

Conceptual; no bound has been proved. The lower bound would hold for every decoder in a stated model of computation, and the upper bound comes from a specific decoder. Each resource needs its own pair of bounds, and a bound in a model that ignores the hardware's real bottleneck explains nothing.

None of this is meaningful until the model is defined. Decoding spends several different resources, and each has its own bounds:

- the number of random accesses to memory;
- the depth of the serial chain, the accesses that must wait for one another;
- misses in each level of cache, and the transfers between them under contention;
- the work and memory spent preparing a structure before decoding;
- the memory footprint while decoding;
- the total work across all parallel workers.

There is an obvious candidate for a quantity both sides can grip. String compressibility has measures on two sides. On the BWT's side is \\(r\\), its number of runs. On the text's side are measures such as \\(z\\), the number of phrases in its LZ77 parse, and \\(\\gamma\\), the size of its smallest string attractor. They are tied by inequalities such as Kempa and Prezza's \\(r = O(z \\log^2 n)\\). But an inequality between measures of compressibility is not a lower bound on the cost of computation. The bridge from one to the other has to be built, and building it is the research.

### The latest negative result points at global structure

The shape of the proof suggested one concrete lead, which we tested the same day. The local Langlands correspondence classifies the local pieces of the problem completely, so the global argument only has to glue them. Our analogue would be a catalogue: for each local arrangement of runs around a row of the table, a precomputed fragment of output, so that decoding steps from fragment to fragment.

Before the data, [the note registered a rule](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md#c-localglobal-a-catalogue-of-local-factors>): drop the lead unless at least 50% of the rows the decoder walks fall in arrangements that determine their output, with a catalogue small enough for a core's own cache. On six real blocks of Wikidata, [the census](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-local-factor-census.md>) (\#687) found at best 10.5%. Arrangements of at least two rows that determine their output covered at most 3.3%. The lead was dropped.

The reason is more useful than the number. The bytes a row emits next are its own sorted prefix, fixed by its position in the global order of the block, not by the shape of the runs around it. Local classification fails because the information the decoder needs is global. That moves attention to constructions that recover the global order directly. One is \\(\\Phi\\), the map at the core of the r-index, a compressed text index that steps through text order from \\(r\\) stored samples. The cost of building those samples, and of reading them under contention, decides whether it can compete.

### LF may be the wrong primitive

The walk along LF was the best route under the cost model of its time: one core, and memory access that was cheap relative to computing. Under six to ten workers sharing 20 MiB of cache, the best route may be a different one altogether. Every kernel we have tuned so far descends from that walk. The next large gain may not come from a faster LF kernel; it may come from finding that LF traversal is the wrong primitive to optimise.

That sharpens the research question to one sentence. **Can we preprocess the run structure of a BWT block into a compact representation that provably reduces the serial dependency depth of decoding, without paying more in preprocessing, memory traffic or contention than it saves?**

## 6\. What comes next

The order follows from the argument. First, the crowd harness on CI, so that candidates are ranked under realistic contention before an i9 run. Second, the cost model, with each resource listed above defined well enough that a lower bound can be stated in it. Third, the constructions of global order that avoid LF, beginning with how cheaply \\(\\Phi\\)'s samples can be obtained from the run structure. Each enters with a falsifiable claim and a rule for rejecting it, registered before its data, as the census was.

The chain of the argument is short. A proof of the Riemann Hypothesis has been checked by three independent kernels at a scale no person could write or review. It was reached by strengthening the statement, constraining quantities from several sides, moving to a richer setting and building every foundation it needed. That search could be wide only because checking was cheap and certain. In mibz2, checking correctness is cheap, but checking performance is not, so that is where the work must go first.

The lesson we take is not to imitate a seventeen-million-line proof. It is to find out what becomes possible when the length of a solution is no longer the dominant constraint.

---

**Sources:** [Levent Alpöge's remark](<https://alpo.ge/remark.pdf>), whose toy proof is not part of the Lean development; the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis" (20 November 2026), for the kernel checks, their costs and Chris Birkbeck's overview; [the source-linked atlas](<https://nasqret.github.io/riemann-rev/atlas/>) of the development.

**Research records:** [the proof's shape, as twelve moves applied to mibz2](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md>) (\#686); [the local-factor census](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/journal/2026-11-21-local-factor-census.md>) (\#687); [the fidelity ledger](<https://github.com/lmmx/mibz2/blob/f5bd7a9/docs/team/research-charter/CONTEXT.md#5a-the-fidelity-ledger>).
