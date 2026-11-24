---
title: "Results without reasons: checking two AI eurekas against our own records"
image: assets/social/blog-2026-11-24-results-without-reasons.png
description: "An essay says AI gives mathematics results without reasons. Two models read it and proposed breakthroughs for our decoder; our records undid both."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Results without reasons: checking two AI eurekas against our own records

An essay says AI gives mathematics results without reasons. Two models read it and proposed breakthroughs for our decoder; our records undid both.
{: .post-dek }

24 November 2026 · 11 min read · [Meta](#5-where-things-stand)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

In October the essayist doomslide published ["Against Mathematics"](https://doomslide.substack.com/p/against-mathematics), a long polemic about what
machines are doing to the subject. Its charge is that mathematics is splitting into results and
reasons, and that the machines now produce the first without the second. I gave the essay to two
models and asked each what it meant for mibz2's research. Within an afternoon each came back with
a breakthrough for our decoder. Neither survived our own records:

- **The first** proposed measuring the decoder's hard core in a new currency, cache misses. Our
  journal already held three experiments that told against it.
- **The second** proposed moving many scattered rows of the block through the decoder together.
  Its lemma is true, and already known; as a decoder, it costs exactly as many steps as the walk
  it would replace.

What did survive is the essay's best idea, put as a question about our decoders, and that
question now has an answer. This post sets out the essay's argument (§1), the two eurekas (§2),
how they failed (§3), the question that survived (§4) and where things stand (§5).

## 1. The essay: results, proofs and reasons

The essay tells the history of mathematics as a creature with three heads, each paid at a
different rate:

- **the proud head** discovers, and is paid in credit, so it must be quick;
- **the blind head** checks, slowly and unpaid;
- **the slow head** understands, and is paid only once a result is old enough to teach.

Whenever one head outruns the others, something dies. Proof assistants have already replaced the
blind head: a machine now checks every line of a proof without needing to understand it. The essay
reports this year's results from AI labs as the proud head's turn: a batch of 722 machine-written
manuscripts released on 6 October, many with machine-checked proofs, and nobody to explain them or
answer for them. I take those reports as the essay's, not ours.

Its sharpest image is ancient. After Pythagoras died, his followers split in two. The **Hearers**
kept his sayings and threw away the reasons; the **Learners** kept both. The essay's claim is that a
language model is the perfect Hearer: it hands back what is true without why. Two older documents
in the essay put the alternative plainly:

- **Archimedes' *Method*.** In a letter to the librarian at Alexandria, Archimedes sent not only
  his results and their proofs, but the mechanical method that found them, so that others might
  "discover other theorems in addition, which have not yet occurred to me". A result, its proof and
  the method that produced it are three different things.
- **Hilbert's lost 24th problem.** Hilbert planned a 24th problem for his 1900 list and left it in
  a notebook: a theory of the simplicity of proofs. Its key sentence is a research instruction:
  "Given two routes, it is not right to take either of these two or to look for a third; it is
  necessary to investigate the area lying between the two routes."

That sentence matters to us because mibz2 has several routes to the same answer.

## 2. Two readings, two eurekas

mibz2 decodes bzip2, and the expensive step in bzip2 decoding is undoing the Burrows–Wheeler
transform. The transform sorts every rotation of a block of text and keeps the last letter of each.
Undoing it means following a map called LF, which takes a row of that sorted table to the row one
letter earlier in the text, one row at a time. We have three exact ways of doing it, and they cost
very different amounts:

- **the walk** follows LF one row at a time;
- **Cells** (the [Rauzy–Veech kernel](2026-11-10-rauzy-veech-reconstruction.md)) moves whole
  stretches of rows at once, wherever neighbouring rows agree on what comes next;
- **Open** ([Open Returns](2026-11-19-open-returns.md)) does the same for the thick stretches and
  walks the thin ones.

What none of them can shortcut is a **width-1 row**: a row whose neighbours disagree with it about
what comes next, so no stretch carries it. On our Wikidata benchmark most of these rows come from
[random statement IDs](2026-11-20-where-the-decoding-work-goes.md), hexadecimal strings whose
letters follow no pattern. Our mathematician agent has proved a lower bound, which we call G1: in
the family of methods these three belong to, every width-1 row costs at least one step.

Both models were given the essay and asked to chart it, read it loosely against our work, and end
on a eureka. Their full responses are in our [source record](https://github.com/lmmx/mibz2/blob/b643d28ed5f3e631260fbd4f086e64cbbc4df0bc/docs/journal/2026-11-24-against-mathematics-source.md), verbatim, kept apart from
each other so that each idea can be traced to its author.

**ChatGPT's eureka: move the random fields together.** The walk follows one ID letter by letter
before it reaches the next. ChatGPT pointed out that all the IDs share a format even though their
letters are random, and proposed transporting them collectively: take the sorted set of rows where
the IDs begin, apply LF to all of them, and sort the result by the letter each row emits. Its lemma
is that the result is sorted again, for free, so the set can be stepped forward phase by phase, one
letter of every ID at a time. Each ID would then become a single jump in the final assembly.

The lemma is easy to check by hand. Take the column L = `annb$na`. LF sends each row to its letter's
first row in the sorted column, plus the number of earlier copies of that letter:

| row | 0 | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| letter L | a | n | n | b | $ | n | a |
| LF(row) | 1 | 4 | 5 | 3 | 0 | 6 | 2 |

Pick the scattered rows 0, 1, 3, 5 and 6. Their destinations are 1, 4, 3, 6 and 2. Grouped by
letter, keeping order inside each group, they are the a's (1, 2), the b (3) and the n's (4, 6):
the frontier 1, 2, 3, 4, 6, sorted without a comparison. (ChatGPT's own illustration of this got
its numbers wrong; the lemma doesn't depend on them.)

**Claude's eureka: separate the count from the price.** The second reading started from a
correction. Earlier that day the swarm had been sent another model's headline proposal: measure
the cost of the width-1 rows in dependent cache misses, the waits for memory that each step must
finish before the next can start. Reading the essay's story of the Hearers, Claude saw that this
proposal had been accepted on the oracle's word, and set it beside our own measurements. From the
wreck it drew a split: G1 floors the *number* of width-1 steps, but nothing floors the *price* of
each one. A decoder that took the same steps far more cheaply would sit outside the bound.

## 3. Checked against the book

The essay's warning is about exactly this: an answer that arrives without its reason, confidently
stated. We held both eurekas to our records.

**The cache-miss currency had already been tested.** Three records bear on it:

- **Lanes, 20 November.** Our [toy-transfer experiment](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md) ran up to eight independent walks
  side by side over Open's cycle. If waiting on memory were the cost, independent lanes would hide
  it. At first they bought only 5–14%. The same entry later records a correction: once each step's
  append was a single branchless 16-byte store, eight lanes cut the emit to 0.55–0.78 of its time,
  in a C harness, though either change alone barely moved it. Hiding the wait pays only once the
  work per step is small.
- **Lanes in the decoder itself.** mibz2 already has that branchless store
  ([Sixteen at a stroke](2026-11-21-sixteen-at-a-stroke.md)). There, eight lanes made the loop 13%
  faster on one thread and 5% slower once eight workers shared the processor
  ([Fast alone, slow in a crowd](2026-11-21-fast-alone-slow-in-a-crowd.md)).
- **The squared map, 23 November.** A design that halved the number of dependent steps was built
  and measured. On ten workers its emit was 1.22 times as slow per step, and the whole decode about
  1.3 times as slow. Our [retrospective](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-24-retrospective-proposal.md) put it down to the wrong assumption about where
  the cost lies. It also changed the footprint and the preprocessing, and no miss counts were
  recorded, so it can't say what the saved waits alone were worth.

None of these makes misses a currency. Waiting and the work done at each step interact, and only
the whole decoder's time decides. That was enough to stop treating a miss count as the target, and
the correction went to the swarm within the hour.

**The ordered frontier is the walk, scheduled differently.** Our mathematician agent took
ChatGPT's construction apart in an [afternoon's record](https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md):

- **The lemma is standard.** It is the step of batched backward search, a known technique.
- **The count doesn't move.** Built record by record, each step of the frontier composes one row
  with the next, and those steps match the walk's steps one for one. For G fields of d letters
  there are G·d steps either way. G1 says there must be, since every width-1 row is still visited.
  That classifies this implementation; a packed version of the partition would have to be priced
  on its own.
- **The price is predicted not to fall.** The partition moves at least 16 bytes for every step.
  This is a prediction, not a measurement, and the record names what would falsify it. Only a
  matched implementation would settle it.
- **Finding the starting rows takes one scan** of a contiguous range. Placing what they produce in
  the output is a separate job, not yet priced.

The record's most useful line is what the frontier *is*. Within one letter, the gap between two
rows' destinations is the number of that letter between them. So one step for a scattered set of rows
is a matter of counting letters in the gaps between them. That ties the frontier to an earlier
result about such counts, though not as an identity: gap counts give one step's destinations,
while the earlier result concerns positions in the output.

Neither eureka was worthless. Both were results without their reasons, and the reasons were already
in our journal.

The checking ran the other way too. A later ChatGPT audit of the mathematician agent's records
narrowed several of their claims: the frontier verdict covers only the record-by-record
implementation, the lanes negative had been corrected in its own entry, and a rule's short proof
says nothing about its cost. It is in the [source record](https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md), §6, and this post follows it.

## 4. The question that survived: what lies between the routes

Hilbert's instruction survives the wreck intact, and both models had reached for it. Applied to
mibz2 it asks: what transformation turns the walk into Cells, and Cells into Open? What does it
keep, what does it remove, and what can it never remove? The same record answers it for our three
decoders, with proofs.

The answer is **contraction along compose**. Think of a decoder as tiling the block's cycle with
steps. Two consecutive steps can be replaced by their composite, and wherever a whole stretch of
rows stays inside runs of equal letters, the composite covers the stretch in one step:

<figure class="bf-fig">
<p class="bf-title">Three tilings of the same stretch of a block's cycle</p>
<svg viewBox="0 0 720 194" role="img" aria-label="Three tilings of the same stretch of a block's cycle: the walk takes one step per row, Cells merges every wide stretch into one step, Open merges only the thick stretches; the width-1 rows stay single steps in all three."><text class="bf-tm" x="150" y="18">a stretch of one block's cycle, in output order →</text><text class="bf-t" x="140" y="52" text-anchor="end">The walk</text><rect class="bf-card" x="152" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="186" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="220" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="254" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="322" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="356" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="390" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="424" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="52" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="526" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="560" y="34" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="594" y="34" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="52" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="34" width="30" height="28" rx="4"/><rect class="bf-card" x="662" y="34" width="30" height="28" rx="4"/><text class="bf-t" x="140" y="100" text-anchor="end">Cells</text><rect class="bf-card" x="152" y="82" width="98" height="28" rx="4"/><text class="bf-ttiny" x="201.0" y="100" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="254" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="82" width="132" height="28" rx="4"/><text class="bf-ttiny" x="354.0" y="100" text-anchor="middle">4 rows, 1 step</text><rect class="bf-cell-hi" x="424" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="100" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="82" width="98" height="28" rx="4"/><text class="bf-ttiny" x="541.0" y="100" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="594" y="82" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="100" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="82" width="64" height="28" rx="4"/><text class="bf-ttiny" x="660.0" y="100" text-anchor="middle">2 → 1</text><text class="bf-t" x="140" y="148" text-anchor="end">Open</text><rect class="bf-card" x="152" y="130" width="98" height="28" rx="4"/><text class="bf-ttiny" x="201.0" y="148" text-anchor="middle">3 rows, 1 step</text><rect class="bf-cell-hi" x="254" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="269.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="288" y="130" width="132" height="28" rx="4"/><text class="bf-ttiny" x="354.0" y="148" text-anchor="middle">4 rows, 1 step</text><rect class="bf-cell-hi" x="424" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="439.0" y="148" text-anchor="middle">w1</text><rect class="bf-cell-hi" x="458" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="473.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="492" y="130" width="30" height="28" rx="4"/><rect class="bf-card" x="526" y="130" width="30" height="28" rx="4"/><rect class="bf-card" x="560" y="130" width="30" height="28" rx="4"/><rect class="bf-cell-hi" x="594" y="130" width="30" height="28" rx="4"/><text class="bf-ttiny" x="609.0" y="148" text-anchor="middle">w1</text><rect class="bf-card" x="628" y="130" width="64" height="28" rx="4"/><text class="bf-ttiny" x="660.0" y="148" text-anchor="middle">2 → 1</text><text class="bf-tm" x="150" y="184">w1: a width-1 row, a single tile under every contraction</text></svg>
<figcaption>Conceptual, not measured. The walk takes one step per row. Cells merges every
stretch that stays inside runs. Open merges only the thick ones and walks the thin ones, trading
steps for less construction. The width-1 rows are single steps in all three.</figcaption>
</figure>

What the transformation keeps, removes and can't touch:

| | Contraction along compose |
|---|---|
| **It preserves** | the text the cycle spells; the order of the cycle; that every row is covered once |
| **It removes** | one LF evaluation for every interior row of a stretch: w rows paid for by one translation |
| **Its invariant** | the set of width-1 rows, each a single step under every contraction |
| **On scattered rows** | it acts (the frontier carries order) but removes nothing: one count per row |

This is what Hilbert asked for: not a third route, but the map between the two we had, with its
invariant. It also says where to look for a fourth decoder. An obligation disappears only where
LF on a set of rows has a description much shorter than the set. Runs of equal letters are one
such family. The open question is whether a scattered set, such as the rows of the random IDs, has
another.

There is one sharp limit. The IDs' letters are random, about 4 bits a row, so any description of
where those rows go needs at least 4 bits per row. That bounds the *size* of the description, not
the *number of steps* needed to apply it. A description with few, wide steps isn't ruled out. That
gap is where the next decoder would have to live.

Claude's analysis raised the standard escape from lower bounds of this kind: naming a composite
once and reusing it, as extension variables shorten proofs in propositional logic. Checked, G1
already charges shared work only once, and [survives](https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md). What could break it is a
rule for a whole scattered set that is cheap to *execute*, not merely short to state: LF's own
formula is already a one-line rule for every row, and applying it row by row is the walk. That is
the same open
question in another form.

## 5. Where things stand

The essay's charge holds up as a warning, and it applied to us twice in one afternoon. Two
confident breakthroughs arrived without reasons. Checked against what we had already measured and
proved, one was refuted and the other was a known lemma that leaves the cost unchanged. What
survived was a question about reasons: what lies between our decoders, and what no contraction
between them can remove.

The split between count and price stays open as its own question. G1 bounds the number of width-1
steps, not what each one costs on a real processor, and our records already show that the
count doesn't fix the time: the same steps ran faster or slower depending on lanes and on the
work in each step. One shortcut is ruled out: checking a guessed decoding
against bzip2's 32-bit block checksum would accept a wrong guess about once in four billion, and
mibz2 must be exact on every block.

What changes is the record each failed idea leaves. Our obstruction write-ups already keep, for
each closed approach, its counterexample, its scope and what would get past it. Since this
afternoon they also keep what the approach taught: the capability it revealed even where it couldn't pay. That is the
difference between a Hearer's saying and a Learner's reason, and it is what let us take both
eurekas apart in an afternoon.

---

**Sources:**

- doomslide, ["Against Mathematics: A religion of the dead, by the dead, for the dead"](https://doomslide.substack.com/p/against-mathematics), Substack,
  24 October 2026. Quotations of Archimedes and Hilbert are from the essay.
- [The source record](https://github.com/lmmx/mibz2/blob/b643d28ed5f3e631260fbd4f086e64cbbc4df0bc/docs/journal/2026-11-24-against-mathematics-source.md): the prompt, ChatGPT's and Claude's analyses verbatim, and
  descriptions of ChatGPT's figures.
- [The ordered frontier and the transformation between decoders](https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md) and
  [abbreviation, batching and the block checksum](https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md), the mathematician agent's records.
- ChatGPT's [audit of those records](https://github.com/lmmx/mibz2/blob/f438f38dbc2d25d3aa964633abf48ee9c453b35f/docs/journal/2026-11-24-against-mathematics-source.md) (§6 of the source record).
- [Toy-transfer](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md), the lanes experiment, and [the retrospective](https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-24-retrospective-proposal.md) on the squared map.
