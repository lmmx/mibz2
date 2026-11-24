---
title: "Between the routes: Hilbert's lost problem and three ways to undo the BWT"
image: assets/social/blog-2026-11-24-between-the-routes.png
description: "An essay against AI mathematics revives Hilbert's lost 24th problem, and for our three decoders it shows which steps each saves and which none can."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Between the routes: Hilbert's lost problem and three ways to undo the BWT

An essay against AI mathematics revives Hilbert's lost 24th problem, and for our three decoders it shows which steps each saves and which none can.
{: .post-dek }

24 November 2026 · 13 min read · [Meta](#6-where-the-question-stands)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

In October the essayist doomslide published ["Against Mathematics"](https://doomslide.substack.com/p/against-mathematics), a long polemic about
what machines are doing to the subject. Its charge is that mathematics is coming apart into results
and reasons, and that machines now produce the first without the second. Underneath the polemic
the essay draws three distinctions, and the third of them, a problem Hilbert wrote down and never
published, turns out to be a precise question about mibz2's decoders.

This post sets out the essay's distinctions (§1), our three ways of undoing bzip2's block sort
(§2), two proposals that follow from the essay and what each amounts to (§3), the
transformation between our decoders that Hilbert's problem asks for (§4), what a faster fourth
decoder would have to be (§5), and where the question stands (§6). The readings and records it
draws on are kept verbatim in our
[source record](https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md); the post draws on all of them.

## 1. The essay: result, proof and method

The essay tells the history of mathematics as a creature with three heads, each paid at a
different rate:

- **the proud head** discovers, and is paid in credit, so it must be quick;
- **the blind head** checks, slowly and unpaid;
- **the slow head** understands, and is paid only once a result is old enough to teach.

Proof assistants have already replaced the blind head: a machine checks every line without
understanding it. The essay reads this year's AI results as the proud head's turn: a batch of 722
machine-written manuscripts released on 6 October, many with machine-checked proofs and nobody to
explain them. Those reports are the essay's.

Three older ideas carry its argument, and they are what this post uses:

- **Hearers and Learners.** After Pythagoras died, his followers split. The Hearers kept his
  sayings and dropped the reasons; the Learners kept both. Knowing *that* and knowing *why* became
  two traditions.
- **Archimedes' *Method*.** Archimedes sent the librarian at Alexandria not only his results and
  their proofs, but the mechanical method that found them, so that others might "discover other
  theorems in addition, which have not yet occurred to me". A result, its proof and the method
  that generates more results are three different things.
- **Hilbert's lost 24th problem.** Hilbert planned a 24th problem for his 1900 list and left it in a
  notebook: a theory of the simplicity of proofs. Its key sentence is an instruction: "Given two
  routes, it is not right to take either of these two or to look for a third; it is necessary to
  investigate the area lying between the two routes."

## 2. Three routes back from the BWT

mibz2 decodes bzip2, and the expensive step is undoing the Burrows–Wheeler transform (BWT). The
transform sorts every rotation of a block of text and keeps the last letter of each, the column L.
Undoing it means following a map called LF, which takes a row of the sorted table to the row one
letter earlier in the text. With C[c] the number of letters smaller than c,

$$LF(x) = C[L[x]] + \#\{\,t < x : L[t] = L[x]\,\}.$$

We have three exact routes along LF, at very different costs:

- **the walk** applies LF one row at a time;
- **Cells**, our [Rauzy–Veech kernel](2026-11-10-rauzy-veech-reconstruction.md), moves whole
  stretches of rows at once, wherever neighbouring rows share their letter;
- **Open** ([Open Returns](2026-11-19-open-returns.md)) does the same for thick stretches and walks
  the thin ones.

No route can shortcut a **width-1 row**, a run of one letter whose neighbours differ, so no stretch
carries it. On our Wikidata benchmark most of these rows come from
[random statement IDs](2026-11-20-where-the-decoding-work-goes.md). We have proved a lower
bound, G1: in the family of methods these routes belong to, every width-1 row costs
at least one step, which we call an event.

## 3. Two proposals, and what they amount to

So, two ways the essay's ideas might speed up our decoder.

### Moving scattered rows together

The walk follows one random ID letter by letter before it reaches the next. The first proposal is
to move all the IDs at once: take the sorted set of rows where the IDs begin, apply LF to all of
them, and group the results by the letter each row emits. Within one letter LF keeps order, and
smaller letters land below larger ones, so the result is sorted again with no comparison sort.

A small case, with L = `annb$na`:

| row | 0 | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| letter L | a | n | n | b | $ | n | a |
| LF(row) | 1 | 4 | 5 | 3 | 0 | 6 | 2 |

The scattered rows 0, 1, 3, 5 and 6 go to 1, 4, 3, 6 and 2. Grouped by letter, keeping order
within each group, they are the a's (1, 2), the b (3) and the n's (4, 6): the sorted frontier 1,
2, 3, 4, 6. (This column's LF splits into three cycles, so it illustrates the ordering only, not a
whole decode.)

What the [frontier](https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md) amounts to:

- **A known step.** It is the step of batched backward search.
- **The same number of events, built record by record.** Each record step composes one row with
  the next, matching the walk's steps one for one: G fields of d letters cost G·d events either
  way, as G1 requires. A packed implementation of the partition, moving many rows per machine
  word, is a different object and would need pricing in its own right.
- **Cheap starting rows, unpriced assembly.** The rows where IDs begin lie in one contiguous range
  and take one scan to find. Placing each ID's letters in the output is a further job.
- **Gap counts.** Within one letter c, the gap between consecutive destinations is the number of c's
  between the two source rows: with $b_c = LF(x_0)$ and $g_i$ the count of c in $[x_i, x_{i+1})$,
  $LF(x_i) = b_c + \sum_{j<i} g_j$. One step of the frontier is a matter of counting letters in
  gaps. Many steps, carrying each ID's identity to its place in the output, is the hard part.

### The count of events against their price

The second proposal separates two things G1 runs together. G1 floors the *number* of
width-1 events. Nothing floors the *price* of each one. A decoder that took the same events far more
cheaply would sit outside the bound.

What our records say about the price:

- **Lanes, in a C harness.** Running up to eight independent walks side by side over Open's cycle
  bought 5–14% while each step's append was branchy and variable-length. Once each append was a
  single 16-byte store, eight lanes cut the emit to 0.55–0.78 of its time, though either change
  alone barely moved it ([toy-transfer](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md)). Hiding the wait for memory pays once the work per
  step is small.
- **Lanes in the decoder itself.** mibz2 already has that store
  ([Sixteen at a stroke](2026-11-21-sixteen-at-a-stroke.md)); there, eight lanes were 13% faster on
  one thread and 5% slower once eight workers shared the processor
  ([Fast alone, slow in a crowd](2026-11-21-fast-alone-slow-in-a-crowd.md)).
- **Halving the dependent steps.** A [squared map](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md) that halved the number of dependent steps
  emitted 1.22 times as slowly per step on ten workers, and the whole decode took about 1.3 times as
  long. It also changed the footprint and the preprocessing, and recorded no miss counts.

So neither the event count nor the count of memory waits sets the time on its own. The work in
each step and the waits between steps interact, and only the whole decoder's time decides. The
price per event is an open question in its own right.

## 4. Between the routes

Hilbert's instruction, put to mibz2, asks: what transformation turns the walk into Cells, and Cells
into Open? What does it keep, what does it remove, and what can it never remove? Our
[record](https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md) answers it for these three, with proofs.

The answer is **contraction along compose**. Think of a decoder as tiling the block's cycle with
steps. Two consecutive steps can be replaced by their composite, and wherever a whole stretch of
rows stays inside runs of one letter, the composite covers the stretch in one step:

<figure class="bf-fig">
<p class="bf-title">Three tilings of the same stretch of a block's cycle</p>
<svg viewBox="0 0 720 194" role="img" aria-label="Three tilings of the same stretch of a block's cycle: the walk takes one step per row, Cells merges every wide stretch into one step, Open merges only the thick stretches; the width-1 rows stay single steps in all three."><text class="bf-tm" x="150" y="18">a stretch of one block's cycle, in output order →</text><text class="bf-t" x="140" y="52" text-anchor="end">The walk</text><rect class="bf-card" x="152" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="186" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="220" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="254" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="322" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="356" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="390" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="424" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="52" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="526" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="560" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="594" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="662" y="34" width="30" height="28" rx="4"/><text class="bf-t" x="140" y="100" text-anchor="end">Cells</text><rect class="bf-card" x="152" y="82" width="98" height="28" rx="4"/><text class="bf-ttiny" x="201.0" y="100" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="254" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="82" width="132" height="28" rx="4"/><text class="bf-ttiny" x="354.0" y="100" text-anchor="middle">4 rows, 1 step</text><rect class="bf-cell-hi" x="424" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="100" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="82" width="98" height="28" rx="4"/><text class="bf-ttiny" x="541.0" y="100" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="594" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="82" width="64" height="28" rx="4"/><text class="bf-ttiny" x="660.0" y="100" text-anchor="middle">2 → 1</text><text class="bf-t" x="140" y="148" text-anchor="end">Open</text><rect class="bf-card" x="152" y="130" width="98" height="28" rx="4"/><text class="bf-ttiny" x="201.0" y="148" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="254" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="130" width="132" height="28" rx="4"/><text class="bf-ttiny" x="354.0" y="148" text-anchor="middle">4 rows, 1 step</text><rect class="bf-cell-hi" x="424" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="148" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="130" width="30" height="28" rx="4"/><rect class="bf-card" x="526" y="130" width="30" height="28" rx="4"/><rect class="bf-card" x="560" y="130" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="594" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="130" width="64" height="28" rx="4"/><text class="bf-ttiny" x="660.0" y="148" text-anchor="middle">2 → 1</text><text class="bf-tm" x="150" y="184">w1: a width-1 row, a single tile under every contraction</text></svg>
<figcaption>Conceptual, not measured. The walk takes one step per row. Cells merges every
stretch that stays inside runs. Open merges only the thick ones and walks the thin ones, trading
steps for less construction. The width-1 rows are single steps in all three.</figcaption>
</figure>

What the transformation keeps, removes and cannot touch:

| | Contraction along compose |
|---|---|
| **It preserves** | the text the cycle spells; the order of the cycle; that every row is covered once |
| **It removes** | one LF evaluation for every interior row of a stretch: w rows paid by one translation |
| **Its invariant** | the set of width-1 rows, each a single step under every contraction |
| **On scattered rows** | it carries order (the frontier) but removes nothing: one count per row |

This is the shape Hilbert asked for: not a third route, but the map between the two we had, with
its invariant. It also says where a fourth decoder would have to come from. An event disappears
only where LF applied to a set of rows has a description much shorter than the set. Stretches of
one letter are one such family. The open question is whether a scattered set, such as the rows of
the random IDs, has another.

One limit is sharp. The IDs' letters are random, about 4 bits a row, so any description of where
those rows go needs at least 4 bits per row. That bounds the *size* of the description, not the
*number of events* needed to apply it. A description applied in few, wide steps isn't ruled out.

## 5. What a faster fourth route would have to be

**Not a short proof.** Lower bounds on proof size are classically escaped by abbreviation: name a
composite once and reuse it, as extension variables shorten propositional proofs. G1 already charges
shared work only once, and [survives it](https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md). Nor is a short statement enough. The formula in
§2 is already a one-line rule for every row of every block, and applying it row by row is the walk.
Short to state, short to prove and cheap to execute are three different properties.

**An interface with its costs exposed.** A candidate would have to take the form

$$\mathrm{Build}(L) \to z, \qquad \mathrm{Advance}(z, h) \to z', \qquad \mathrm{Emit}(z') \to T,$$

with a fixed language for the description z, its size, and the cost of building, composing and
reading it, every access to L counted. If Emit's first act is to enumerate rows and rank each one,
the candidate has moved the walk, not removed it. A useful test is to supply z for free and look
at what consumes it.

**Carrying identities, not only sets.** Compressing the set of rows is not enough. For the set of
all rows, $LF^h$ of the set is the set itself, a constant description for every h, and nothing is
decoded. What a decoder needs is the labelled map $x \mapsto LF^h(x)$, carrying each start's
identity to its place in the output.

**Beyond the swap argument.** Our lower bounds for rules that read only aggregate counts rest on
swaps: two letters exchanged so that a column stays valid, every count is unchanged, and the output
differs. A cheap order-sensitive summary defeats every such swap. With letters coded as numbers,
the first moment

$$A(L) = \sum_i i \cdot \mathrm{code}(L[i])$$

changes by $(i-j)(\mathrm{code}(L[j]) - \mathrm{code}(L[i])) \ne 0$ under any swap of unequal letters
at $i$ and $j$, and a run of code $a$ over $[s, e)$ contributes $a\,(s+e-1)(e-s)/2$, so A takes one
pass over the runs. Yet it decodes nothing: `bca` and `cab` have the same letters and the same
moment, A = 2, while their LF maps are [1, 2, 0] and [2, 0, 1]. Separating swaps is not
decoding. A stronger lower bound has to find two valid columns that every permitted observation,
taken together, fails to tell apart.

**Where multi-row rules need their own induction.** Extending G1 to rules that move several rows
on a shared context needs a separate argument for the rows inside such a rule. In the valid column
`Paaabxbccddeeff$MZ`, whose LF is one cycle of 18 rows, rows 1 and 3 both spell `abcdef` before
diverging. One rule keyed on that context carries both. Their next rows, 4 and 6, are width-1 runs
of b whose own six-letter contexts, `bcdef$` and `bcdefM`, are unique. A rule keyed on a shared
context at its start can carry width-1 rows inside it, so the charging argument has to cover those
interior rows directly.

## 6. Where the question stands

The frozen target asks whether some exact decoder can take substantially fewer events than Open on
real blocks. Open pays at least one event per retained step, not only per width-1 row, so a yes
would have to carry all of the retained words, not just the random IDs.

- **The count.** G1 holds for routes built by contracting stretches, with sharing. What it doesn't
  cover is a rule applied to a whole scattered set, with a short description that is also cheap to
  apply: the Build, Advance and Emit of §5.
- **The price.** A separate question. The records show the price per event moving with the work in
  each step and with how steps overlap, not with either count alone.
- **The next test.** Supply Open's existing word codes in output order, for free, and time the
  unchanged consumer against Open. If even that leaves too little headroom, this representation is
  closed. If it leaves room, the room is the budget for building the order natively.
- **Exactness.** mibz2 must be exact on every block, so checking a guessed decoding against bzip2's
  32-bit block checksum, which would accept a wrong guess about once in four billion, is not a route.

The essay's distinction between a result and its method comes out here as a concrete object. The
three routes are results. Contraction along compose is a method: it explains why each route costs
what it does, and names the one thing, the width-1 rows, that any new method must find another way
to carry.

---

**Sources:**

- doomslide, ["Against Mathematics: A religion of the dead, by the dead, for the dead"](https://doomslide.substack.com/p/against-mathematics),
  Substack, 24 October 2026. Quotations of Archimedes and Hilbert are from the essay.
- [The source record](https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md): two readings of the essay, their figures
  (described) and an audit of the records below (§6), all verbatim.
- [The ordered frontier and the transformation between decoders](https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md) and
  [abbreviation, batching and the block checksum](https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md), our research records.
- [Toy-transfer](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md), the lanes experiments with their later correction, and
  [the squared map](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-23-emit-cost-sq.md).
- The first moment, the Build/Advance/Emit interface, the multi-row witness and the supplied-codes
  test are from the audit (source record §6); the witnesses were checked independently.
