[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/index.md>)

# Halving the single-row steps: a target only a new algorithm can meet

We now search for a new way to undo bzip2's block sort by forbidding the obvious mechanism and asking whether any exact decoder can do without it.

24 November 2026 · 15 min read · [Meta](<#7-where-the-search-stands>)

For most of its life this project has improved its decoders a few per cent at a time. bzip2 sorts the rotations of each block of text and keeps the last letter of each, the Burrows–Wheeler transform (BWT). Undoing it means following a map, LF, from each row of the sorted table to the row one letter earlier in the text ([the structure of that map](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-08-inverse-bwt-navigation/index.md>)). Our first decoder, the walk, applies LF one row at a time. Two later ones, [Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) and [Open](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-19-open-returns/index.md>), move whole stretches of rows together wherever neighbouring rows share their letter, and Open walks the thin stretches. The figure below shows that history at 20 threads, release by release. Six releases, from 0.2.9 to 0.5.0, left throughput near 1.6 GB/s: Cells shipped in 0.3.0, but only for fewer than six threads. Once it reached 20 threads, in 0.5.1, throughput rose release by release, to 3.26 GB/s by 0.5.13. The new algorithm did more than add a step of its own: it opened a run of gains that the releases before it had not found.

Throughput release by release: flat, then climbing once Cells took over

Measured, as on the [benchmark history](<https://lmmx.github.io/mibz2/pr-preview/pr-927/benchmarks/history/index.md>) page (CI, merge by merge; wikidata; 20 threads; Absolute). CI's runners have 4 CPUs, so at 20 threads every point is a release on one i9-10900K, its throughput modelled from paired release-against-release runs with lbzip2 as the fixed anchor, and placed at the last merge to master that its tag contains; the dotted line only joins releases. Below, each release against the one before it, with 95% intervals. The rise after 0.5.1 is many releases' work on top of Cells and then Open, not Cells alone; 0.5.13's +13% came from work outside the BWT step: a faster CRC, a new block reader and leaner parallel output.

This post is about attempting a step of that kind again. What we are after is a decoder as different from ours as Rauzy–Veech induction, the mathematics behind Cells, was from the walk, and the past week went into making that search well posed. There is no new decoder to report. What there is, instead, is a sharp boundary. Call an *event* a step that settles one row on its own ([what decoding pays for](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-18-what-decoding-pays-for/index.md>) counts them), and a *width-1 row* one whose letter forms a run of length one, so that no stretch can carry it; a block has h₁ of them, and on our benchmark [most come from random IDs](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-20-where-the-decoding-work-goes/index.md>). Our three decoders belong to one family, the **current family**: decoders that move intervals of rows by fixed translations and compose those moves. We have now proved that every member of that family pays at least one event per width-1 row. A decoder that settles a block in h₁/2 events or fewer therefore cannot be a refinement of anything we have; it must be a new algorithm. That recasts an open-ended search as a binary target, which a candidate either meets or provably misses. The first test of a candidate representation has also returned: given the decoding order for free, the whole decoder ran 16% faster on one thread, a gain that all but vanished at ten.

**Rauzy–Veech induction** studies a line cut into intervals and shuffled, an interval exchange, by cutting off the shorter end again and again until the pieces repeat. A block's runs of equal letters form such an exchange, so the same steps move whole runs at a time. The kernel built on it is told in [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), and the dictionary between the two worlds in [Towers, words and loops](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-16-mathematical-dictionary/index.md>).

Four pieces of work make the target usable. The first is the [target itself, stated exactly](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/FRONTIER.md>): an exact decoder must settle every block with at most h₁/2 events in all, in two passes over the block and with no more memory than Open, and must beat Open's whole decoding time. Open's own events are its m retained steps, one on each width-1 row and the rest on rows of wider runs, each writing a word, the letters between it and the next. The second is the [lower bound, G1](<https://github.com/lmmx/mibz2/blob/620eacc724988238fe6341c1102b9b2320847279/docs/journal/monodromy/2026-11-24-g1-scoped.md>), which proves the h₁ floor for the current family and so guarantees that anything meeting the target is new. The third is a [census of our benchmark's blocks](<https://github.com/lmmx/mibz2/blob/6793043730e41410487ff0b5dcd0f572049ae304/docs/journal/2026-11-24-wikidata-block-census.md>), in which three structural features of a block predict the time Open spends building its structures, with a held-out R² of 0.96. The fourth is a record of [fourteen candidate frameworks](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/FRONTIER.md>) already closed, each kept with its counterexample, its scope and what would get past it. Beneath all four lies the transformation that connects our three decoders, together with the one invariant it can never remove (§4).

On the way, the search was reframed four times: to use everything, to count the cost, to fix a binary target, and to keep the method rather than the result (§§1–2). The last of these came from [an essay against AI mathematics](<https://doomslide.substack.com/p/against-mathematics>), and it sharpened into a precise question about our decoders, which the rest of the post works through. It sets out the routes we have (§3) and the transformation between them (§4), tests two proposals against it (§5), says what a new route would have to be (§6), and closes with where the search stands (§7).

## 1\. The goal: a new algorithm, not a faster one

The model for what we want is the step from the walk to induction. The walk follows the block's cycle one row at a time; induction recast the same permutation as an interval exchange and moved whole stretches of rows at once. It was not a faster walk but different mathematics for the same specification, and its cost scaled with a different property of the block, one that happens to be small on real data.

The same permutation, read as a loop and as an exchange of runs

Conceptual, on the BWT of `banana$`. The walk pays one dependent lookup per row; induction treats each run of equal letters as an interval and moves it with one translation (the signed numbers), so its cost follows the number of runs. On real blocks runs are long, which is why the step from the walk to induction paid.

This gives the goal its shape. A constructive proof that the transform can be inverted *is* an inversion algorithm: the walk is a proof by induction along the text, and the induction kernels are a proof by induction over the interval exchange. A new proof with a different structure is therefore a new algorithm with a different cost, and the search becomes one for a new proof of an old fact, judged on its cost as well as its correctness. Without that second criterion an algorithm can be altered indefinitely while remaining correct, and whether any alteration helps is discovered only at the end.

## 2\. Four reframings of the search

**Use everything.** A [claimed proof of the Riemann hypothesis](<https://alpo.ge/remark.pdf>), still awaiting independent checking, built on [fifty older tools reaching back two centuries](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-22-two-centuries-of-tools/index.md>), made mathematical distance look cheap, and our [method](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/METHOD.md>) accordingly adopted a mathematical maximalism: no limit on how far a framework might sit from compression, and chains of worlds in which one representation's job is to carry the problem into another. The approach produced fourteen frameworks, each tried and closed, and a register of obstructions; it did not produce the step.

**Count the cost.** A paper analysing the simplex method "by the book" ([Bach, Black, Huiberts and Kafer](<https://arxiv.org/abs/2510.21613>)) sets cost theorems beside correctness theorems, and we followed its lead. A census of our benchmark's blocks now predicts the time Open spends building its structures from three features of a block (held-out R² 0.96), and a lower bound, G1 (§3), states what every decoder in the current family must pay.

**[The census](<https://github.com/lmmx/mibz2/blob/6793043730e41410487ff0b5dcd0f572049ae304/docs/journal/2026-11-24-wikidata-block-census.md>)** timed Open on each of the 232 distinct blocks of our Wikidata benchmark, one worker at a time, and asked whether a few features of a block's structure predict where the time goes. For building Open's structures they do: three features explain 96% of the variation on blocks held out of the fit. For writing the output they don't, which is one reason the price of an event (§5) stays an open question.

**Fix a binary target.** Super Mario 64 players keep an A-button challenge, and one effort on the Shifting Sand Land level asked whether it could be done with fewer than one press of the A button. That recasts a minimisation as a decision problem, settled either by a route or by a proof that none exists. Its decisive move, as [the transcript of that work](<https://github.com/lmmx/mibz2/blob/aff4172ec770b24fecb934d8eee7b18f30cbb047/docs/journal/2026-11-24-algorithm-level.md#2-the-ssl-coq-material>) records it, was to work backwards: it supplied the impossible-looking condition by hand in a test fixture, confirmed the payoff, and only then sought how the real game could produce it, one supplied condition at a time. Payoff first and mechanism second is also the logic of the supplied-codes test in §7.

**The A-button challenge** is a long-running game among Super Mario 64 players: to finish the game pressing the jump button as seldom as may be. Its formal side has begun elsewhere: [a Rocq proof](<https://github.com/tra38/sm64-wmotr-abc-proof>) about another level, Wing Mario over the Rainbow, shows that a run with no A press never makes Mario fly, though it still rests on named residual hypotheses, and its theorem that the level needs an A press at all is not yet started.

"Fewer than one" forces a leap because it forbids the obvious mechanism, the tool on which every ordinary route relies, and mibz2 has an exact counterpart:

|  | The Mario 64 challenge | mibz2 |
| --- | --- | --- |
| **The obvious mechanism** | the A press | following the block's cycle, one row at a time |
| **The best known** | one A press | Open: m events, h₁ of them on width-1 rows |
| **The class's floor** | ordinary routes need at least one | G1: every decoder in the current family pays at least h₁ |
| **The challenge** | fewer than one | at most h₁/2 events in all |

On our benchmark, an extract of Wikidata, h₁/m is 0.30, 0.39 and 0.83 at the minimum, median and maximum across its 232 distinct blocks. Because no decoder in the current family goes below the floor, anything below it is, by theorem, not an edit to Open. Whether a candidate is a new algorithm or an increment stops being a judgement and becomes a consequence of the bound.

Our [target](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/FRONTIER.md>) now has the same form: invert every block exactly, in two passes and no more memory than Open, with at most h₁/2 events in all, and then beat Open's whole decoding time. The criterion is binary, and each candidate begins with a supplied-capability test: the decoder is handed, for free, whatever the new idea would compute, and we measure what that buys.

**Keep the method, not the result.** In October the essayist doomslide published ["Against Mathematics"](<https://doomslide.substack.com/p/against-mathematics>), a polemic about what machines are doing to the subject. It tells the history of mathematics as a creature with three heads, each paid at a different rate: a proud head that discovers, a blind head that checks, and a slow head that understands. Proof assistants, it argues, have already replaced the blind head, and it reads this year's AI results, such as a batch of 722 machine-written manuscripts released on 6 October, as the proud head's turn: results that arrive with machine-checked proofs and nobody to explain them. (Those reports are the essay's own.)

Three older ideas carry its argument, and each bears on our search:

- **Hearers and Learners.** After Pythagoras died his followers divided: the [Hearers](<https://en.wikipedia.org/wiki/Akousmatikoi>) kept his sayings and let the reasons go, while the Learners kept both. A register of closed frameworks is a list of sayings unless each entry keeps its reason: the counterexample, the scope, what would get past it, and what the attempt revealed.
- **Archimedes' *Method*.** Archimedes sent the librarian at Alexandria not only his results and their proofs, but the [mechanical method](<https://en.wikipedia.org/wiki/The_Method_of_Mechanical_Theorems>) that found them, so that others might "discover other theorems in addition, which have not yet occurred to me". A result, its proof and a method that generates more results are three different things, and only the last moves a search forward.
- **Hilbert's lost 24th problem.** Hilbert planned a [24th problem](<https://en.wikipedia.org/wiki/Hilbert%27s_twenty-fourth_problem>) for his 1900 list and left it in a notebook: a theory of the simplicity of proofs. Its key sentence is an instruction: "Given two routes, it is not right to take either of these two or to look for a third; it is necessary to investigate the area lying between the two routes." Using everything had been a search for a third route; Hilbert directs us instead to the map between the routes we already have.

**The *Method*** came down to us in a single copy, the [Archimedes Palimpsest](<https://en.wikipedia.org/wiki/Archimedes_Palimpsest>), made in the tenth century and scraped from its parchment in the thirteenth, that the leaves might carry a prayer book. Heiberg read it beneath the prayers in 1906, and much of it was recovered only by imaging, between 1999 and 2008: a method nearly lost under a book of results of another kind.

The essay draws the distinction between a result and a method with mathematics of its own. In one retelling the Pythagoreans are confronted with a wall of right triangles with whole-number sides. Checking them one by one with knotted rope yields results; but a brother who looks at the triangles whose short sides differ by one finds the rule that turns each into the next, and that rule is a method: it generates triangles that were never on the wall, and, run backwards, it proves that the diagonal of a square has no whole-number ratio to its side.

One rule that generates the answers, and proves the impossibility

← BackPlayNext →

1. Start from the 3-4-5 triangle: its short sides sum to s = 7 and its long side is c = 5, and 7² − 2·5² = −1.
2. One application of the rule gives (41, 29): the 20-21-29 triangle, still one seed short of the square on the diagonal.
3. Again: (239, 169), the 119-120-169 triangle. The ratio s/c is now within about 10⁻⁵ of √2.
4. Again: (1,393, 985). Each answer comes from the last by the same rule, without solving anything afresh.
5. Run the rule backwards and every pair gets smaller. A pair with s² = 2c² exactly would also get smaller forever, which whole numbers can't do: the same rule that generates the answers proves √2 has no ratio.

Use Next and Back, or Play. The essay's triangles with short sides one apart, as pairs (s, c): s is the sum of the short sides and c the long side. One matrix carries each pair to the next and preserves s² − 2c² = −1; its inverse is the descent that shows √2 is irrational. A result is one triangle; the method is the rule.

The next retelling turns the rule into a proof. Lay the side of a square along its diagonal: it fits once, with a sliver left over, and that sliver is the side of a smaller square in the corner, on which the measuring begins again as on a copy of the first figure. For whole numbers the process must stop, but for a true square it never does, so no unit, however small, measures both the side and the diagonal. The Greeks called this alternate subtraction anthyphairesis. It is the Euclidean algorithm on two lengths ([*Elements* X.2](<https://mathcs.clarku.edu/~djoyce/elements/bookX/propX2.html>)).

Anthyphairesis: subtract the side from the diagonal, and repeat

Exact values from the essay's footnote (Euclid, *Elements* X.2). For whole numbers the subtraction stops at 1 against 1; for a true square it never stops, which is why its side and diagonal have no common measure. On the right, conceptual: a Rauzy–Veech step on two intervals is the same subtraction, and its sequence of steps is the continued fraction of the ratio of the lengths.

It is also the smallest case of our own step. Cut a line into two intervals and swap them: that is a two-interval exchange. A Rauzy–Veech step on it subtracts the shorter interval from the longer, as the side was laid along the diagonal, and its sequence of steps is the continued fraction of the ratio of the lengths. Rauzy–Veech induction, the step that took mibz2 from the walk to Cells, is anthyphairesis on many intervals at once.

The essay ends the story with a name. Plato records it in the [*Theaetetus*](<https://en.wikipedia.org/wiki/Theaetetus_(dialogue)>) ([147d](<https://www.perseus.tufts.edu/hopper/text?doc=Plat. Theaet. 147d>)): [Theodorus](<https://en.wikipedia.org/wiki/Theodorus_of_Cyrene>) showed, one case at a time, that the sides of squares of three, five and so on up to seventeen square feet have no common measure with a foot. Theaetetus ceased to take the cases one at a time and named them all together as powers. It is the move from checking cases to naming an operation, the move our search must make with its frameworks. Contraction along compose, in §4, is our attempt at it: one name for what turns each of our decoders into the next.

Hilbert's instruction, applied to our own search, changes its shape:

Two ways to search for a new route

Conceptual. Importing distant mathematics produces candidates one at a time, each judged alone. Studying the transformation between the routes we already have produces one object, with an invariant, that says what every future candidate has to do differently (§4).

The readings and records this post draws on are kept verbatim in our [source record](<https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md>).

## 3\. Three routes back from the BWT

The expensive step in decoding bzip2 is undoing the Burrows–Wheeler transform. The transform sorts every rotation of a block of text and keeps the last letter of each, the column L, and undoing it means following LF from each row of the sorted table to the row one letter earlier in the text. With C\[c\] the number of letters smaller than c,

\\\[LF(x) = C\[L\[x\]\] + \\\#\\{\\,t \< x : L\[t\] = L\[x\]\\,\\}.\\\]

We have three exact routes along it, at very different costs:

- **the walk** applies LF one row at a time;
- **Cells**, our [Rauzy–Veech kernel](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), moves whole stretches of rows at once, wherever neighbouring rows share their letter;
- **Open** ([Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-19-open-returns/index.md>)) does the same for thick stretches and walks the thin ones.

A **width-1 row** is a run of one letter whose neighbours differ, so no stretch carries it, and our lower bound, G1, proves that every decoder in the current family pays at least one event on each. Its scope is exactly that family: rules that move several rows on a shared context need an argument of their own (§6).

On our Wikidata benchmark most of these rows come from [random statement IDs](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-20-where-the-decoding-work-goes/index.md>). On six blocks we measured closely, about 65% of the width-1 rows lie in hexadecimal tokens that occur only once, 18% in hex tokens that repeat and 17% in everything else. Repetition is not the difficulty: across 3,645 repeated 40-character SHA-1 hashes on those blocks, only 3 bytes fall in width-1 runs, whereas each novel hash has about 36 of its 40 bytes in them, close to the \\((15/16)^2 \\times 40 \\approx 35\\) that random hex predicts. A block therefore splits into two streams, the random rows and the rest, and its cost is the sum of the two.

## 4\. Between the routes we have

Put to mibz2, Hilbert's instruction asks: what transformation turns the walk into Cells, and Cells into Open? What does it keep, what does it remove, and what can it never remove? Our [record](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) answers it for these three; the proofs are there.

The answer is **contraction along compose**. A decoder can be pictured as tiling the block's cycle with steps. Two consecutive steps can be replaced by their composite, and wherever a whole stretch of rows stays inside runs of one letter, the composite covers the stretch in one step:

Three tilings of the same stretch of a block's cycle

Conceptual, not measured. The walk takes one step per row. Cells merges every stretch that stays inside runs. Open merges only the thick ones and walks the thin ones, trading steps for less construction. The width-1 rows are single steps in all three.

What the transformation keeps, removes and cannot touch:

|  | Contraction along compose |
| --- | --- |
| **It preserves** | the text the cycle spells; the order of the cycle; that every row is covered once |
| **It removes** | one LF evaluation for every interior row of a stretch: w rows paid by one translation |
| **Its invariant** | the set of width-1 rows, each a single step under every contraction |
| **On scattered rows** | it carries order (the frontier) but removes nothing: one count per row |

This is the shape Hilbert asked for: not a third route, but the map between the two we had, with its invariant. It also says where a fourth decoder would have to come from, since an event disappears only where LF applied to a set of rows has a description much shorter than the set. Stretches of one letter form one such family; the open question is whether a scattered set, such as the rows of the random IDs, admits another.

One limit is sharp. Because the IDs' letters are random, about 4 bits a row, any description of where those rows go needs at least 4 bits per row. That bounds the *size* of the description, however, not the *number of events* needed to apply it, so a description applied in a few wide steps is not ruled out.

## 5\. Two proposals, and what they amount to

Two outside readings of the essay each proposed a way forward; this section weighs what each amounts to.

### Moving scattered rows together

The walk follows one random ID letter by letter before it reaches the next; the first proposal is to move all the IDs at once: take the sorted set of rows where the IDs begin, apply LF to all of them, and group the results by the letter each row emits. Within one letter LF keeps order, and smaller letters land below larger ones, so the result is sorted again with no comparison sort.

Moving a scattered set of rows one step, and keeping it sorted

← BackPlayNext →

1. The column L = annb$na, with five scattered rows selected and listed in row order.
2. Apply LF to each: LF(x) is the first sorted row of x's letter plus the number of earlier copies of that letter. The destinations 1, 4, 3, 6, 2 are out of order.
3. Group them by the letter each row emits, keeping order inside each group. Within one letter LF keeps order, and smaller letters land lower, so the result 1, 2, 3, 4, 6 is sorted again.
4. Why: within one letter, the gap between two destinations is the number of that letter between the two source rows. One step of the frontier is a set of gap counts.

Use Next and Back, or Play. Green is a, blue n, amber b. This column's LF splits into three cycles, so it illustrates the ordering only, not a whole decode. Redrawn and corrected from the proposal's own illustration.

What the [frontier](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) amounts to:

- **A known step.** Search tools for compressed text already use it when they look for many patterns at once (batched backward search).
- **The same number of events, built entry by entry.** Each frontier entry is a row together with the ID it belongs to, and each step moves one entry one row, matching the walk's steps one for one: G IDs of d letters cost G·d events either way, as G1 requires. A packed implementation of the partition, moving many rows per machine word, is a different object and would need pricing in its own right.
- **Cheap starting rows, unpriced assembly.** The rows where IDs begin lie in one contiguous range and take one scan to find. Placing each ID's letters in the output is a further job.
- **Gap counts.** Within one letter c, the gap between consecutive destinations is the number of c's between the two source rows: with \\(b\_c = LF(x\_0)\\) and \\(g\_i\\) the count of c in \\(\[x\_i, x\_{i+1})\\), \\(LF(x\_i) = b\_c + \\sum\_{j\<i} g\_j\\). One step of the frontier is a matter of counting letters in gaps. The hard part is many steps: carrying each ID's identity to its place in the output.

**Backward search** is this same step in an older trade. Herein a compressed full-text index seeks a pattern from its last letter to its first, each letter narrowing an interval of sorted rows by one LF step; and many patterns sought at once move all their intervals together: the frontier's step, put to another use.

The random IDs show what the frontier changes and what it doesn't:

The same random IDs, decoded along each ID or across all of them

Conceptual, with illustrative IDs. The letters are random, but every ID has the same shape, so the frontier can step all of them together. The count of events is the same either way (G·d = 3 × 8); what changes is which memory each round touches. The three totals are different kinds of number, so they aren't drawn on one scale.

As a whole decoder, it would still have three jobs to do:

The frontier as a whole decoder: what is in hand and what is open

Conceptual. Finding where the IDs start is cheap. Moving them, placing them and joining them to the rest of the block are the obligations a decoder built on the frontier would still have to meet and price.

### The count of events against their price

The second proposal separates two things G1 runs together: G1 floors the *number* of width-1 events, but nothing floors the *price* of each, so a decoder that took the same events far more cheaply would sit outside the bound.

What our records say about the price:

- **Lanes, in a test program.** Running up to eight independent walks side by side (lanes) over Open's cycle, in a small C program, bought 5–14% while each step's append, the write of its letters to the output, was branchy and variable-length. Once each append was a single 16-byte store, eight lanes cut the time spent writing the output to 0.55–0.78 of what it was, though either change alone barely moved it ([toy-transfer](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>)). Hiding the wait for memory, in other words, pays only once the work per step is small.
- **Lanes in the decoder itself.** mibz2 already has that store ([Sixteen at a stroke](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-21-sixteen-at-a-stroke/index.md>)); there, eight lanes were 13% faster on one thread and 5% slower once eight decoding threads shared the processor ([Fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/pr-preview/pr-927/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)).
- **Halving the dependent steps.** A version of Open that applied LF twice per step, a [squared map](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md>) halving the number of steps that wait on memory, took 1.22 times as long per step to write the output on ten threads, and the whole decode about 1.3 times as long. It also changed the footprint and the preprocessing, and recorded no miss counts.

Neither the event count nor the count of memory waits, then, sets the time on its own: the work in each step and the waits between steps interact, and only the whole decoder's time decides. The price per event is an open question in its own right.

## 6\. What a new route would have to be

**Not a short proof.** The classical escape from lower bounds on proof size is abbreviation, naming a composite once and reusing it; but G1 already charges shared work only once, and [survives it](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>). Nor is a short statement enough: the formula in §3 is already a one-line rule for every row of every block, and applying it row by row is simply the walk. To be short to state, short to prove and cheap to execute are three distinct properties.

**An interface with its costs exposed.** A candidate would have to take the form

\\\[\\mathrm{Build}(L) \\to z, \\qquad \\mathrm{Advance}(z, h) \\to z', \\qquad \\mathrm{Emit}(z') \\to T,\\\]

with a fixed language for the description z, its size, and the cost of building, composing and reading it, every access to L counted. If Emit's first act is to enumerate rows and count the earlier copies of each one's letter, the candidate has moved the walk, not removed it. A useful test is to supply z for free and look at what consumes it.

**Carrying identities, not only sets.** Compressing the set of rows is not enough: for the set of all rows, \\(LF^h\\) of the set is the set itself, a constant description for every h, from which nothing is decoded. What a decoder needs is the labelled map \\(x \\mapsto LF^h(x)\\), carrying each start's identity to its place in the output.

**Beyond the swap argument.** We also have lower bounds for rules that read only aggregate counts, such as how many of each letter lie in an interval. They rest on swaps: two letters exchanged so that the column is still the BWT of some text (a valid column), every count is unchanged, and the output differs. Every such swap, however, is defeated by a cheap order-sensitive summary. With letters coded as numbers, the first moment

\\\[A(L) = \\sum\_i i \\cdot \\mathrm{code}(L\[i\])\\\]

changes by \\((i-j)(\\mathrm{code}(L\[j\]) - \\mathrm{code}(L\[i\])) \\ne 0\\) under any swap of unequal letters at \\(i\\) and \\(j\\), and a run of code \\(a\\) over \\(\[s, e)\\) contributes \\(a\\,(s+e-1)(e-s)/2\\), so A takes one pass over the runs. Yet it decodes nothing: `bca` and `cab` have the same letters and the same moment, A = 2, while their LF maps are \[1, 2, 0\] and \[2, 0, 1\]. Separating swaps is not the same as decoding, so a stronger lower bound must find two valid columns that every permitted observation, taken together, fails to tell apart.

**A witness** that tells every swapped pair apart, a perfect witness even, may still know naught of the order: `bca` and `cab` share A = 2 and decode differently. A fact about one summary, not a verdict on all.

Same letters, same moment, different permutations

Exact. The first moment A = Σ i·code(L\[i\]) changes under every swap of unequal letters, so it tells apart every pair our swap-based lower bounds rely on, and it takes one pass over the runs. Yet these two columns share it and decode differently. Separating swaps is not decoding.

**Where multi-row rules need their own induction.** Extending G1 to rules that move several rows on a shared context needs a separate argument for the rows inside such a rule. In the valid column `Paaabxbccddeeff$MZ`, whose LF is one cycle of 18 rows, rows 1 and 3 both spell `abcdef` before diverging. One rule keyed on that context carries both. Their next rows, 4 and 6, are width-1 runs of b whose own six-letter contexts, `bcdef$` and `bcdefM`, are unique. A rule keyed on a shared context at its start can carry width-1 rows inside it, so the charging argument has to cover those interior rows directly.

A shared context can carry width-1 rows inside it

Exact, on a valid column whose LF is one cycle of 18 rows. Green: rows 1 and 3, which share the context abcdef. Amber: rows 4 and 6, width-1 runs inside those paths. A rule keyed on the shared context at its start carries them, so an argument that charges width-1 rows has to cover rows inside multi-row rules directly.

**The scoped no, and what lies just outside it.** These pieces combine into one theorem over a declared class of decoders. Suppose every byte is written by a rigid fact, an interval moved by one translation, and positions come only from labels on the start rows of those facts. Then every decoder of that kind pays \\(c\_1 \\ge h\_1 - |B|\\) events, where \\(B\\) is the set of width-1 rows placed by a label read off an uncharged observation. The theorem's whole content therefore lies in how large \\(B\\) can be.

**An uncharged observation** is something a decoder may read for free under our counting rules: a summary that folds the whole block into a few numbers, such as how many of each letter it holds, or the first moment below. Only the reading of a single row's own record, to settle that row, is charged as an event; whence the question of how much such free summaries can place.

A row can be placed from an observation only if every valid column with the same observation puts it in the same place; otherwise one fixed rule is wrong on one of them. That turns the question into counting. Across all 5,880 valid columns of ten letters over three symbols, the histogram plus the first moment leaves 96.4% of them sharing their observation with a column that decodes differently, and pins down only 22% of the width-1 rows. On our six measured blocks, observations made of interval histograms and declared run records place half the width-1 rows only after 6,903 to 14,120 separate observations. That is reading the block row by row under another name.

Of three decoders that broke the bound, and came nowhere near half

Just outside the class, the bound does break, and the way it breaks is informative. Three exact decoders beat it on small witnesses. One fills the last unwritten slots from the histogram, one copies a stretch of output already written, and one moves a shared context, as in the 18-row column above. None comes near half on real blocks:

- **Filling from the histogram** can skip at most the commonest letter's width-1 rows, about 7% of them. On every block that letter is the digit 4, which every version-4 ID carries in a fixed place.
- **Copying, even planned for free,** still leaves more than half of the width-1 rows as literals on five blocks of six.
- **Shared contexts** break into single rows on random fields beyond about five letters.

A negative result in this shape is one theorem over a stated class, with a section saying what it does not cover, and the nearest decoders outside it measured.

## 7\. Where the search stands

The target is precise, and it was fixed before any candidate was tested: On every block, an exact decoder must pay at most h₁/2 events in all, in at most two passes over the block and no more scratch memory than Open; it must be one fixed program with no stored answers; and it must beat our current decoder on whole decoding time.

The target is also harder than its headline suggests. Counted strictly, Open pays one event for every retained step, m of them, not only for the h₁ width-1 rows. Even if every width-1 event became free, the steps on wide runs would remain, so a decoder that cheapens only the random rows needs \\(h\_1/m \\ge 2/3\\). Across the 232 distinct blocks of our benchmark, \\(h\_1/m\\) is 0.41 overall and 0.30 to 0.83 block by block. A decoder that meets the target must therefore write the word of every retained step more cheaply, not merely those of the random IDs.

The target is the conclusion of a conditional theorem. Its premises are the agenda: each piece of work proves one, closes one, or measures what one would need.

The main theorem, as premises

Status as of 24 November 2026. The theorem is conditional: if every premise holds on every block, a decoder meeting the target exists. Random rows are well over half of h₁ on the blocks measured, so while each still costs one event (a = 1) the margin cannot hold; meeting it means beating the floor on random content itself.

- **The count.** G1 holds for the current family, with shared work counted once. What it doesn't cover is a rule applied to a whole scattered set, with a short description that is also cheap to apply: the Build, Advance and Emit of §6.
- **The price** is a separate question: the records show the price per event moving with the work in each step and with how steps overlap, not with either count alone.
- **The supplied test has run.** Handed its word codes already in output order, for free, Open spent half as long writing the output (0.55 of its time). The whole decoder took 0.988 of its time on ten threads, and 0.838 on one ([the result](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>)). On ten threads that is far below the 10% our method asks of a perfect version before building a native prototype, so this representation stops here; even a free order buys little once ten threads share the machine, though on one thread there is 16% to win.
- **Exactness.** mibz2 must be exact on every block, so checking a guessed decoding against bzip2's 32-bit block checksum, which would accept a wrong guess about once in four billion, is not a route.

The next test: supply the order, keep everything else

Run on 24 November: the free order cut the output writing to 0.55 of its time, and the whole decoder to 0.988 on ten threads and 0.838 on one. Supplying the order for free bounds what any way of producing it could win, before anyone builds one. Our method builds a native prototype only if a perfect version beats Open by at least 10% end to end.

Read together, the four reframings say what the search now values: using everything supplies the material, counting the cost the measure, and the binary target the line a candidate has to cross. The fourth says what to keep from each attempt: not the verdict alone, but the method behind it, the transformation with its invariant, which can generate the next route. The three decoders are results. Contraction along compose is a method: it explains why each route costs what it does, and names the one thing, the width-1 rows, that a new route must find another way to carry. When results can be produced cheaply, that kind of understanding is the scarce part, and it is what we now count as progress.

---

**Sources:**

- doomslide, ["Against Mathematics: A religion of the dead, by the dead, for the dead"](<https://doomslide.substack.com/p/against-mathematics>), Substack, 24 October 2026. Quotations of Archimedes and Hilbert are from the essay.
- [The source record](<https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md>): two readings of the essay, their figures (described) and an audit of the records below (§6), all verbatim.
- [The ordered frontier and the transformation between decoders](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) and [abbreviation, batching and the block checksum](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>), our research records.
- The two streams and the totals: [the six-block measurements](<https://github.com/lmmx/mibz2/blob/8756c557205290add83d3f90a0dc0a222fb03fb5/research/neon/n8-amended-n0-copies-rindex.md>) and [the 232-block baseline](<https://github.com/lmmx/mibz2/blob/656d5e7ab8faaee34172bf8355f676224dc70b48/research/neon/n2-f-baseline-and-needs-m2.md>). The target: [the main theorem](<https://github.com/lmmx/mibz2/blob/cdeef65aba3b5ebad758cf4f5d169f0534bf80ab/docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md>), §1.
- The scoped no: [Theorem N′](<https://github.com/lmmx/mibz2/blob/bc1429ed92baab9b3e468fd5b5b8718ccd555741/docs/journal/monodromy/2026-11-24-mixed-calculus-repair.md>), [the observation fibres](<https://github.com/lmmx/mibz2/blob/bf8bab3d6e658f81cb156841296e8fd79753388f/research/neon/n18-complete-observation-fibres.md>) and [the three decoders outside the class](<https://github.com/lmmx/mibz2/blob/bf8bab3d6e658f81cb156841296e8fd79753388f/research/neon/n19-theorem-nprime-output-model.md>).
- Bach, Black, Huiberts and Kafer, ["Beyond Smoothed Analysis: Analyzing the Simplex Method by the Book"](<https://arxiv.org/abs/2510.21613>).
- [The census of the benchmark's blocks](<https://github.com/lmmx/mibz2/blob/6793043730e41410487ff0b5dcd0f572049ae304/docs/journal/2026-11-24-wikidata-block-census.md>) and [the supplied-codes result](<https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060>).
- [Toy-transfer](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>), the lanes experiments with their later correction, and [the squared map](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md>).
- The first moment, the Build/Advance/Emit interface, the multi-row witness and the supplied-codes test are from the audit (source record §6); the witnesses were checked independently.
