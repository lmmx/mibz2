[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# Halving the single-row steps: a target only a new algorithm can meet

We now search for a new way to undo bzip2's block sort by forbidding the obvious mechanism and asking whether any exact decoder can do without it.

24 November 2026 · 15 min read · [Meta](<#7-where-the-search-stands>)

For most of its life, mibz2's research has moved in small steps. Each change made the existing decoder a few per cent faster, and the bench judged it. What we are actually after is bigger: a new decoder, a different way to undo bzip2's block sort, as different from ours as Rauzy–Veech induction was from the row-by-row walk it replaced. In the past week we changed the size of step we attempt. We set a target that no edit to our decoder can meet, by theorem, and built what it takes to pursue it:

- **A target that is met or not.** It was frozen before the new measurements. An exact decoder must pay at most half the width-1 events that the current kind must, in two passes over the block and no more memory than Open, and then beat Open on the bench.
- **A proved lower bound, G1,** for every route of the current kind. Anything below it is a new algorithm, not an increment.
- **A census of our benchmark's blocks.** It predicts the cost of building Open's structures from block structure alone.
- **Fourteen candidate frameworks closed.** Each keeps its counterexample, its scope and what would get past it.
- **The transformation that connects our three decoders,** with the one thing it can never remove.

This post reads our own search. In the past week it has been reframed four times: use everything, count the cost, set a target that is met or not, and keep the method rather than the result. Each reframing changed what we count as progress (§§1–2). The last one came from an essay against AI mathematics, and it turned into a precise question about our decoders. The rest of the post works that question through. It sets out the routes we have (§3) and the transformation between them (§4). It then tests two proposals against it (§5), says what a new route would have to be (§6), and ends on where the search stands (§7).

## 1\. The goal: a new algorithm, not a faster one

The step from the walk to induction is the model. The walk follows the block's cycle one row at a time. Induction reframed the same permutation as an interval exchange, and moved whole stretches of rows at once. It was not a faster walk. It implemented different mathematics for the same specification, and its cost scaled with a different property of the block, one that is small on real data.

The same permutation, read as a loop and as an exchange of runs

Conceptual, on the BWT of `banana$`. The walk pays one dependent lookup per row; induction treats each run of equal letters as an interval and moves it with one translation (the signed numbers), so its cost follows the number of runs. On real blocks runs are long, which is why the step from the walk to induction paid.

That gives the goal its shape. A constructive proof that the transform can be inverted *is* an inversion algorithm: the walk is a proof by induction along the text, and the induction kernels are a proof by induction over the interval exchange. A new proof with a different structure is a new algorithm with a different cost. So the search is for a new proof of an old fact, with a cost element alongside correctness. Without that cost element, an algorithm can be changed and kept correct indefinitely, and whether it helps is discovered only at the end.

## 2\. Four reframings of the search

**Use everything.** A claimed proof of the Riemann hypothesis, built on [fifty older tools reaching back two centuries](<https://lmmx.github.io/mibz2/blog/2026-11-22-two-centuries-of-tools/index.md>), made mathematical distance look cheap. Our [method](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/METHOD.md>) adopted mathematical maximalism: no limit on how far a framework sits from compression, and chains of worlds, where one representation's job is to carry the problem into another. It produced many frameworks tried and closed, and a register of obstructions. It did not produce the step.

**Count the cost.** A paper analysing the simplex method "by the book" (Bach, Black, Huiberts and Kafer) puts cost theorems beside correctness theorems. We did the same. A census of our benchmark's blocks now predicts the cost of building Open's structures from block structure alone, and a lower bound, G1 (§3), says what every route of our current kind must pay.

**Set a target that is met or not.** A Super Mario 64 project asked whether a level's challenge could be done with *fewer than one* press of the A button. That is a minimisation turned into a yes-or-no question, answered by a route or by a proof that none exists. Its decisive move came from working backwards: it supplied the impossible-looking condition in a test fixture, confirmed the payoff, and only then found how the game could produce it, one supplied condition at a time. Payoff first, mechanism second.

What makes "fewer than one" force a leap is that it forbids the obvious mechanism: the tool every ordinary route relies on. mibz2 has an exact counterpart.

|  | The Mario 64 challenge | mibz2 |
| --- | --- | --- |
| **The obvious mechanism** | the A press | following the block's cycle, one row at a time |
| **The best known** | one A press | Open: h₁ width-1 steps among its m retained steps |
| **The class's floor** | ordinary routes need at least one | G1: every route of the current kind pays at least one event per width-1 row |
| **The challenge** | fewer than one | at most h₁/2 |

Here h₁ is the block's number of width-1 runs, and h₁/m is 0.30, 0.39 and 0.83 at the minimum, median and maximum across Wikidata's distinct blocks. Because no route of the current kind goes below the floor, anything below it is, by theorem, not an edit to Open. Whether a candidate is a new algorithm or an increment stops being a judgement and becomes a consequence of the bound.

Our [target](<https://github.com/lmmx/mibz2/blob/d60c4215e5afa94478fff856b018275af157abd9/docs/team/research-charter/FRONTIER.md>) now has the same form: invert every block exactly, in two passes and no more memory than Open, with at most half the single-row events that every route of the current kind must pay, and then beat our current decoder on the bench. A candidate either crosses that line or it doesn't. Each candidate starts with a supplied-capability test: hand the decoder what the new idea would compute, for free, and measure what it buys.

**Keep the method, not the result.** In October the essayist doomslide published ["Against Mathematics"](<https://doomslide.substack.com/p/against-mathematics>), a polemic about what machines are doing to the subject. It tells the history of mathematics as a creature with three heads, each paid at a different rate: a proud head that discovers, a blind head that checks, and a slow head that understands. Proof assistants have already replaced the blind head. The essay reads this year's AI results, such as a batch of 722 machine-written manuscripts released on 6 October, as the proud head's turn: results arrive with machine-checked proofs and nobody to explain them. Those reports are the essay's.

Three older ideas carry its argument, and each bears on our search:

- **Hearers and Learners.** After Pythagoras died, his followers split. The Hearers kept his sayings and dropped the reasons; the Learners kept both. A register of closed frameworks is a list of sayings unless each entry keeps its reason: the counterexample, the scope, what would get past it, and what the attempt revealed.
- **Archimedes' *Method*.** Archimedes sent the librarian at Alexandria not only his results and their proofs, but the mechanical method that found them, so that others might "discover other theorems in addition, which have not yet occurred to me". A result, its proof and a method that generates more results are three different things, and only the last moves a search forward.
- **Hilbert's lost 24th problem.** Hilbert planned a 24th problem for his 1900 list and left it in a notebook: a theory of the simplicity of proofs. Its key sentence is an instruction: "Given two routes, it is not right to take either of these two or to look for a third; it is necessary to investigate the area lying between the two routes." Using everything had been a search for a third route. This says to study the map between the routes we already have.

The essay makes the distinction between a result and a method with its own mathematics. In one retelling, the Pythagoreans are handed a wall of right triangles with whole-number sides. Checking them one by one, with knotted rope, gives results. Looking at the triangles whose short sides differ by one, a brother finds the rule that turns each into the next, and that rule is a method: it generates triangles that were never on the wall, and, run backwards, it proves that the diagonal of a square has no whole-number ratio to its side.

One rule that generates the answers, and proves the impossibility

← BackPlayNext →

1. Start from the 3-4-5 triangle: its short sides sum to s = 7 and its long side is c = 5, and 7² − 2·5² = −1.
2. One application of the rule gives (41, 29): the 20-21-29 triangle, still one seed short of the square on the diagonal.
3. Again: (239, 169), the 119-120-169 triangle. The ratio s/c is now within about 10⁻⁵ of √2.
4. Again: (1,393, 985). Each answer comes from the last by the same rule, without solving anything afresh.
5. Run the rule backwards and every pair gets smaller. A pair with s² = 2c² exactly would also get smaller forever, which whole numbers can't do: the same rule that generates the answers proves √2 has no ratio.

Use Next and Back, or Play. The essay's triangles with short sides one apart, as pairs (s, c): s is the sum of the short sides and c the long side. One matrix carries each pair to the next and preserves s² − 2c² = −1; its inverse is the descent that shows √2 is irrational. A result is one triangle; the method is the rule.

The essay's next retelling turns the rule into a proof. Lay the side of a square along its diagonal. It fits once, with a sliver left over. The sliver is the side of a smaller square in the corner, and the measuring starts again on a copy of the first figure. For whole numbers the process must stop, but for a true square it never does, so no unit, however small, measures both the side and the diagonal. The Greeks called this alternate subtraction anthyphairesis. It is the Euclidean algorithm on two lengths.

Anthyphairesis: subtract the side from the diagonal, and repeat

Exact values from the essay's footnote (Euclid, *Elements* X.2). For whole numbers the subtraction stops at 1 against 1; for a true square it never stops, which is why its side and diagonal have no common measure. On the right, conceptual: a Rauzy–Veech step on two intervals is the same subtraction, and its sequence of steps is the continued fraction of the ratio of the lengths.

It is also the smallest case of our own step. Cut a line into two intervals and swap them: that is a two-interval exchange. A Rauzy–Veech step on it subtracts the shorter interval from the longer, as the side was laid along the diagonal, and its sequence of steps is the continued fraction of the ratio of the lengths. Rauzy–Veech induction, the step that took mibz2 from the walk to Cells, is anthyphairesis on many intervals at once.

The essay ends the story with a name. Plato records it in the *Theaetetus* (147d): Theodorus showed, one case at a time, that the sides of squares of three, five and so on up to seventeen square feet have no common measure with a foot. Theaetetus stopped taking the cases one at a time and called them all *powers*. That is the move from checking frameworks to naming an operation. Contraction along compose, in §4, is our attempt at it: one name for what turns each of our decoders into the next.

Hilbert's instruction, applied to our own search, changes its shape:

Two ways to search for a new route

Conceptual. Importing distant mathematics produces candidates one at a time, each judged alone. Studying the transformation between the routes we already have produces one object, with an invariant, that says what every future candidate has to do differently (§4).

The readings and records this post draws on are kept verbatim in our [source record](<https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md>).

## 3\. Three routes back from the BWT

mibz2 decodes bzip2, and the expensive step is undoing the Burrows–Wheeler transform (BWT). The transform sorts every rotation of a block of text and keeps the last letter of each, the column L. Undoing it means following a map called LF, which takes a row of the sorted table to the row one letter earlier in the text. With C\[c\] the number of letters smaller than c,

\\\[LF(x) = C\[L\[x\]\] + \\\#\\{\\,t \< x : L\[t\] = L\[x\]\\,\\}.\\\]

We have three exact routes along LF, at very different costs:

- **the walk** applies LF one row at a time;
- **Cells**, our [Rauzy–Veech kernel](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), moves whole stretches of rows at once, wherever neighbouring rows share their letter;
- **Open** ([Open Returns](<https://lmmx.github.io/mibz2/blog/2026-11-19-open-returns/index.md>)) does the same for thick stretches and walks the thin ones.

A **width-1 row** is a run of one letter whose neighbours differ, so no stretch carries it. We have proved a lower bound, G1: in the family of methods these three routes belong to, every width-1 row costs at least one step, which we call an event.

On our Wikidata benchmark most of these rows come from [random statement IDs](<https://lmmx.github.io/mibz2/blog/2026-11-20-where-the-decoding-work-goes/index.md>). On six blocks we measured closely, about 65% of the width-1 rows lie in hexadecimal tokens that occur only once, 18% in hex tokens that repeat and 17% in everything else. Repetition is not the difficulty. Across 3,645 repeated 40-character SHA-1 hashes on those blocks, only 3 bytes fall in width-1 runs. Each novel hash has about 36 of its 40 bytes in them, close to the \\((15/16)^2 \\times 40 \\approx 35\\) that random hex predicts. So a block splits into two streams, the random rows and the rest, and the cost adds over the two.

## 4\. Between the routes we have

Put to mibz2, Hilbert's instruction asks: what transformation turns the walk into Cells, and Cells into Open? What does it keep, what does it remove, and what can it never remove? Our [record](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) answers it for these three, with proofs.

The answer is **contraction along compose**. Think of a decoder as tiling the block's cycle with steps. Two consecutive steps can be replaced by their composite, and wherever a whole stretch of rows stays inside runs of one letter, the composite covers the stretch in one step:

Three tilings of the same stretch of a block's cycle

Conceptual, not measured. The walk takes one step per row. Cells merges every stretch that stays inside runs. Open merges only the thick ones and walks the thin ones, trading steps for less construction. The width-1 rows are single steps in all three.

What the transformation keeps, removes and cannot touch:

|  | Contraction along compose |
| --- | --- |
| **It preserves** | the text the cycle spells; the order of the cycle; that every row is covered once |
| **It removes** | one LF evaluation for every interior row of a stretch: w rows paid by one translation |
| **Its invariant** | the set of width-1 rows, each a single step under every contraction |
| **On scattered rows** | it carries order (the frontier) but removes nothing: one count per row |

This is the shape Hilbert asked for: not a third route, but the map between the two we had, with its invariant. It also says where a fourth decoder would have to come from. An event disappears only where LF applied to a set of rows has a description much shorter than the set. Stretches of one letter are one such family. The open question is whether a scattered set, such as the rows of the random IDs, has another.

One limit is sharp. The IDs' letters are random, about 4 bits a row, so any description of where those rows go needs at least 4 bits per row. That bounds the *size* of the description, not the *number of events* needed to apply it. A description applied in few, wide steps isn't ruled out.

## 5\. Two proposals, and what they amount to

So, two ways the reframed search might move a decoder forward, and what each turns out to be.

### Moving scattered rows together

The walk follows one random ID letter by letter before it reaches the next. The first proposal is to move all the IDs at once: take the sorted set of rows where the IDs begin, apply LF to all of them, and group the results by the letter each row emits. Within one letter LF keeps order, and smaller letters land below larger ones, so the result is sorted again with no comparison sort.

Moving a scattered set of rows one step, and keeping it sorted

← BackPlayNext →

1. The column L = annb$na, with five scattered rows selected and listed in row order.
2. Apply LF to each: LF(x) is the first sorted row of x's letter plus the number of earlier copies of that letter. The destinations 1, 4, 3, 6, 2 are out of order.
3. Group them by the letter each row emits, keeping order inside each group. Within one letter LF keeps order, and smaller letters land lower, so the result 1, 2, 3, 4, 6 is sorted again.
4. Why: within one letter, the gap between two destinations is the number of that letter between the two source rows. One step of the frontier is a set of gap counts.

Use Next and Back, or Play. Green is a, blue n, amber b. This column's LF splits into three cycles, so it illustrates the ordering only, not a whole decode. Redrawn and corrected from the proposal's own illustration.

What the [frontier](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) amounts to:

- **A known step.** It is the step of batched backward search.
- **The same number of events, built record by record.** Each record step composes one row with the next, matching the walk's steps one for one: G fields of d letters cost G·d events either way, as G1 requires. A packed implementation of the partition, moving many rows per machine word, is a different object and would need pricing in its own right.
- **Cheap starting rows, unpriced assembly.** The rows where IDs begin lie in one contiguous range and take one scan to find. Placing each ID's letters in the output is a further job.
- **Gap counts.** Within one letter c, the gap between consecutive destinations is the number of c's between the two source rows: with \\(b\_c = LF(x\_0)\\) and \\(g\_i\\) the count of c in \\(\[x\_i, x\_{i+1})\\), \\(LF(x\_i) = b\_c + \\sum\_{j\<i} g\_j\\). One step of the frontier is a matter of counting letters in gaps. Many steps, carrying each ID's identity to its place in the output, is the hard part.

The random IDs show what the frontier changes and what it doesn't:

The same random IDs, decoded along each ID or across all of them

Conceptual, with illustrative IDs. The letters are random, but every ID has the same shape, so the frontier can step all of them together. The count of events is the same either way (G·d = 3 × 8); what changes is which memory each round touches. The three totals are different kinds of number, so they aren't drawn on one scale.

As a whole decoder, it would still have three jobs to do:

The frontier as a whole decoder: what is in hand and what is open

Conceptual. Finding where the IDs start is cheap. Moving them, placing them and joining them to the rest of the block are the obligations a decoder built on the frontier would still have to meet and price.

### The count of events against their price

The second proposal separates two things G1 runs together. G1 floors the *number* of width-1 events. Nothing floors the *price* of each one. A decoder that took the same events far more cheaply would sit outside the bound.

What our records say about the price:

- **Lanes, in a C harness.** Running up to eight independent walks side by side over Open's cycle bought 5–14% while each step's append was branchy and variable-length. Once each append was a single 16-byte store, eight lanes cut the emit to 0.55–0.78 of its time, though either change alone barely moved it ([toy-transfer](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>)). Hiding the wait for memory pays once the work per step is small.
- **Lanes in the decoder itself.** mibz2 already has that store ([Sixteen at a stroke](<https://lmmx.github.io/mibz2/blog/2026-11-21-sixteen-at-a-stroke/index.md>)); there, eight lanes were 13% faster on one thread and 5% slower once eight workers shared the processor ([Fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)).
- **Halving the dependent steps.** A [squared map](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md>) that halved the number of dependent steps emitted 1.22 times as slowly per step on ten workers, and the whole decode took about 1.3 times as long. It also changed the footprint and the preprocessing, and recorded no miss counts.

So neither the event count nor the count of memory waits sets the time on its own. The work in each step and the waits between steps interact, and only the whole decoder's time decides. The price per event is an open question in its own right.

## 6\. What a new route would have to be

**Not a short proof.** Lower bounds on proof size are classically escaped by abbreviation: name a composite once and reuse it, as extension variables shorten propositional proofs. G1 already charges shared work only once, and [survives it](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>). Nor is a short statement enough. The formula in §3 is already a one-line rule for every row of every block, and applying it row by row is the walk. Short to state, short to prove and cheap to execute are three different properties.

**An interface with its costs exposed.** A candidate would have to take the form

\\\[\\mathrm{Build}(L) \\to z, \\qquad \\mathrm{Advance}(z, h) \\to z', \\qquad \\mathrm{Emit}(z') \\to T,\\\]

with a fixed language for the description z, its size, and the cost of building, composing and reading it, every access to L counted. If Emit's first act is to enumerate rows and rank each one, the candidate has moved the walk, not removed it. A useful test is to supply z for free and look at what consumes it.

**Carrying identities, not only sets.** Compressing the set of rows is not enough. For the set of all rows, \\(LF^h\\) of the set is the set itself, a constant description for every h, and nothing is decoded. What a decoder needs is the labelled map \\(x \\mapsto LF^h(x)\\), carrying each start's identity to its place in the output.

**Beyond the swap argument.** Our lower bounds for rules that read only aggregate counts rest on swaps: two letters exchanged so that a column stays valid, every count is unchanged, and the output differs. A cheap order-sensitive summary defeats every such swap. With letters coded as numbers, the first moment

\\\[A(L) = \\sum\_i i \\cdot \\mathrm{code}(L\[i\])\\\]

changes by \\((i-j)(\\mathrm{code}(L\[j\]) - \\mathrm{code}(L\[i\])) \\ne 0\\) under any swap of unequal letters at \\(i\\) and \\(j\\), and a run of code \\(a\\) over \\(\[s, e)\\) contributes \\(a\\,(s+e-1)(e-s)/2\\), so A takes one pass over the runs. Yet it decodes nothing: `bca` and `cab` have the same letters and the same moment, A = 2, while their LF maps are \[1, 2, 0\] and \[2, 0, 1\]. Separating swaps is not decoding. A stronger lower bound has to find two valid columns that every permitted observation, taken together, fails to tell apart.

Same letters, same moment, different permutations

Exact. The first moment A = Σ i·code(L\[i\]) changes under every swap of unequal letters, so it tells apart every pair our swap-based lower bounds rely on, and it takes one pass over the runs. Yet these two columns share it and decode differently. Separating swaps is not decoding.

**Where multi-row rules need their own induction.** Extending G1 to rules that move several rows on a shared context needs a separate argument for the rows inside such a rule. In the valid column `Paaabxbccddeeff$MZ`, whose LF is one cycle of 18 rows, rows 1 and 3 both spell `abcdef` before diverging. One rule keyed on that context carries both. Their next rows, 4 and 6, are width-1 runs of b whose own six-letter contexts, `bcdef$` and `bcdefM`, are unique. A rule keyed on a shared context at its start can carry width-1 rows inside it, so the charging argument has to cover those interior rows directly.

A shared context can carry width-1 rows inside it

Exact, on a valid column whose LF is one cycle of 18 rows. Green: rows 1 and 3, which share the context abcdef. Amber: rows 4 and 6, width-1 runs inside those paths. A rule keyed on the shared context at its start carries them, so an argument that charges width-1 rows has to cover rows inside multi-row rules directly.

## 7\. Where the search stands

The frozen target is precise. On every block, an exact decoder must pay at most half of h₁ in single-row events, in at most two passes over the block and no more scratch memory than Open. It must be one fixed program with no stored answers, and it must beat our current decoder on whole decoding time.

The target is harder than its headline. Counted strictly, Open pays one event for every retained step, m of them, not only for the h₁ width-1 rows. Even if every width-1 event became free, the steps on wide runs would remain, so a decoder that cheapens only the random rows needs \\(h\_1/m \\ge 2/3\\). Across the 232 distinct blocks of our benchmark, \\(h\_1/m\\) is 0.41 overall and 0.30 to 0.83 block by block. A yes therefore has to carry all of the retained words more cheaply, not just the random IDs.

- **The count.** G1 holds for routes built by contracting stretches, with sharing. What it doesn't cover is a rule applied to a whole scattered set, with a short description that is also cheap to apply: the Build, Advance and Emit of §6.
- **The price.** A separate question. The records show the price per event moving with the work in each step and with how steps overlap, not with either count alone.
- **The next test.** Supply Open's existing word codes in output order, for free, and time the unchanged consumer against Open. If even that leaves too little headroom, this representation is closed. If it leaves room, the room is the budget for building the order natively.
- **Exactness.** mibz2 must be exact on every block, so checking a guessed decoding against bzip2's 32-bit block checksum, which would accept a wrong guess about once in four billion, is not a route.

The next test: supply the order, keep everything else

A proposed experiment, not yet run. Supplying the order for free bounds what any way of producing it could win, before anyone builds one. The 10% bar is our method's headroom gate for a native prototype.

Read together, the four reframings say what the search now values. Using everything supplies the material. The book supplies the cost. The yes-or-no target supplies the line a candidate has to cross. The last says what to keep from each attempt: not the verdict alone, but the method behind it, the transformation with its invariant, which can generate the next route. The three decoders are results. Contraction along compose is a method: it explains why each route costs what it does, and names the one thing, the width-1 rows, that a new route must find another way to carry. When results can be produced cheaply, that kind of understanding is the scarce part, and it is what we now count as progress.

---

**Sources:**

- doomslide, ["Against Mathematics: A religion of the dead, by the dead, for the dead"](<https://doomslide.substack.com/p/against-mathematics>), Substack, 24 October 2026. Quotations of Archimedes and Hilbert are from the essay.
- [The source record](<https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md>): two readings of the essay, their figures (described) and an audit of the records below (§6), all verbatim.
- [The ordered frontier and the transformation between decoders](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) and [abbreviation, batching and the block checksum](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>), our research records.
- The two streams and the totals: [the six-block measurements](<https://github.com/lmmx/mibz2/blob/8756c557205290add83d3f90a0dc0a222fb03fb5/research/neon/n8-amended-n0-copies-rindex.md>) and [the 232-block baseline](<https://github.com/lmmx/mibz2/blob/656d5e7ab8faaee34172bf8355f676224dc70b48/research/neon/n2-f-baseline-and-needs-m2.md>). The frozen target: [the main theorem](<https://github.com/lmmx/mibz2/blob/cdeef65aba3b5ebad758cf4f5d169f0534bf80ab/docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md>), §1.
- [Toy-transfer](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>), the lanes experiments with their later correction, and [the squared map](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md>).
- The first moment, the Build/Advance/Emit interface, the multi-row witness and the supplied-codes test are from the audit (source record §6); the witnesses were checked independently.
