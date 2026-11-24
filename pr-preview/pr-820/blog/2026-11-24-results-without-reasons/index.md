[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-820/blog/index.md>)

# Results without reasons: checking two AI eurekas against our own records

An essay says AI gives mathematics results without reasons. Two models read it and proposed breakthroughs for our decoder; our records undid both.

24 November 2026 · 11 min read · [Meta](<#5-where-things-stand>)

In October the essayist doomslide published "Against Mathematics", a long polemic about what machines are doing to the subject. Its charge is that mathematics is splitting into results and reasons, and that the machines now produce the first without the second. I gave the essay to two models and asked each what it meant for mibz2's research. Within an afternoon each came back with a breakthrough for our decoder. Neither survived our own records:

- **The first** proposed measuring the decoder's hard core in a new currency, cache misses. Our journal had already refuted that on this machine, three days earlier.
- **The second** proposed moving many scattered rows of the block through the decoder together. Its lemma is true, and already known; as a decoder, it costs exactly as many steps as the walk it would replace.

What did survive is the essay's best idea, put as a question about our decoders, and that question now has an answer. This post sets out the essay's argument (§1), the two eurekas (§2), how they failed (§3), the question that survived (§4) and where things stand (§5).

## 1\. The essay: results, proofs and reasons

The essay tells the history of mathematics as a creature with three heads, each paid at a different rate:

- **the proud head** discovers, and is paid in credit, so it must be quick;
- **the blind head** checks, slowly and unpaid;
- **the slow head** understands, and is paid only once a result is old enough to teach.

Whenever one head outruns the others, something dies. Proof assistants have already replaced the blind head: a machine now checks every line of a proof without needing to understand it. The essay reports this year's results from AI labs as the proud head's turn: a batch of 722 machine-written manuscripts released on 6 October, many with machine-checked proofs, and nobody to explain them or answer for them. I take those reports as the essay's, not ours.

Its sharpest image is ancient. After Pythagoras died, his followers split in two. The **Hearers** kept his sayings and threw away the reasons; the **Learners** kept both. The essay's claim is that a language model is the perfect Hearer: it hands back what is true without why. Two older documents in the essay put the alternative plainly:

- **Archimedes' *Method*.** In a letter to the librarian at Alexandria, Archimedes sent not only his results and their proofs, but the mechanical method that found them, so that others might "discover other theorems in addition, which have not yet occurred to me". A result, its proof and the method that produced it are three different things.
- **Hilbert's lost 24th problem.** Hilbert planned a 24th problem for his 1900 list and left it in a notebook: a theory of the simplicity of proofs. Its key sentence is a research instruction: "Given two routes, it is not right to take either of these two or to look for a third; it is necessary to investigate the area lying between the two routes."

That sentence matters to us because mibz2 has several routes to the same answer.

## 2\. Two readings, two eurekas

mibz2 decodes bzip2, and the expensive step in bzip2 decoding is undoing the Burrows–Wheeler transform. The transform sorts every rotation of a block of text and keeps the last letter of each. Undoing it means following a map called LF, which takes a row of that sorted table to the row one letter earlier in the text, one row at a time. We have three exact ways of doing it, and they cost very different amounts:

- **the walk** follows LF one row at a time;
- **Cells** (the [Rauzy–Veech kernel](<https://lmmx.github.io/mibz2/pr-preview/pr-820/blog/2026-11-10-rauzy-veech-reconstruction/index.md>)) moves whole stretches of rows at once, wherever neighbouring rows agree on what comes next;
- **Open** ([Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-820/blog/2026-11-19-open-returns/index.md>)) does the same for the thick stretches and walks the thin ones.

What none of them can shortcut is a **width-1 row**: a row whose neighbours disagree with it about what comes next, so no stretch carries it. On our Wikidata benchmark most of these rows come from [random statement IDs](<https://lmmx.github.io/mibz2/pr-preview/pr-820/blog/2026-11-20-where-the-decoding-work-goes/index.md>), hexadecimal strings whose letters follow no pattern. Our mathematician agent has proved a lower bound, which we call G1: in the family of methods these three belong to, every width-1 row costs at least one step.

Both models were given the essay and asked to chart it, read it loosely against our work, and end on a eureka. Their full responses are in our [source record](<https://github.com/lmmx/mibz2/blob/b643d28ed5f3e631260fbd4f086e64cbbc4df0bc/docs/journal/2026-11-24-against-mathematics-source.md>), verbatim, kept apart from each other so that each idea can be traced to its author.

**ChatGPT's eureka: move the random fields together.** The walk follows one ID letter by letter before it reaches the next. ChatGPT pointed out that all the IDs share a format even though their letters are random, and proposed transporting them collectively: take the sorted set of rows where the IDs begin, apply LF to all of them, and sort the result by the letter each row emits. Its lemma is that the result is sorted again, for free, so the set can be stepped forward phase by phase, one letter of every ID at a time. Each ID would then become a single jump in the final assembly.

The lemma is easy to check by hand. Take the column L = `annb$na`. LF sends each row to its letter's first row in the sorted column, plus the number of earlier copies of that letter:

| row | 0 | 1 | 2 | 3 | 4 | 5 | 6 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| letter L | a | n | n | b | $ | n | a |
| LF(row) | 1 | 4 | 5 | 3 | 0 | 6 | 2 |

Pick the scattered rows 0, 1, 3, 5 and 6. Their destinations are 1, 4, 3, 6 and 2. Grouped by letter, keeping order inside each group, they are the a's (1, 2), the b (3) and the n's (4, 6): the frontier 1, 2, 3, 4, 6, sorted without a comparison. (ChatGPT's own illustration of this got its numbers wrong; the lemma doesn't depend on them.)

**Claude's eureka: separate the count from the price.** The second reading started from a correction. Earlier that day the swarm had been sent another model's headline proposal: measure the cost of the width-1 rows in dependent cache misses, the waits for memory that each step must finish before the next can start. Reading the essay's story of the Hearers, Claude saw that this proposal had been accepted on the oracle's word, and set it beside our own measurements. From the wreck it drew a split: G1 floors the *number* of width-1 steps, but nothing floors the *price* of each one. A decoder that took the same steps far more cheaply would sit outside the bound.

## 3\. Checked against the book

The essay's warning is about exactly this: an answer that arrives without its reason, confidently stated. We held both eurekas to our records.

**The cache-miss currency was already refuted.** Two entries said so:

- **Lanes, 20 November.** Our [toy-transfer experiment](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>) ran up to eight independent walks side by side over Open's cycle. If waiting on memory were the cost, independent lanes would hide it. They bought 5–14%. The entry's conclusion: "the dependent load is not the main cost: about ten bytes are appended per retained step".
- **The squared map, 23 November.** A design that halved the number of dependent steps was built and measured. On ten workers its emit was 1.22 times as slow per step, and the whole decode about 1.3 times as slow. Our [retrospective](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-24-retrospective-proposal.md>) put it in one line: it was "built on the opposite assumption and failed for exactly that reason".

On these two experiments, cutting misses or waiting didn't buy time; the cost was the work done at each step. (Neither count is a reliable proxy for time on its own: the same number of steps ran 5–14% faster in lanes, and fewer misses ran slower.) The correction went to the swarm within the hour.

**The ordered frontier is the walk, scheduled differently.** Our mathematician agent took ChatGPT's construction apart in an [afternoon's record](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>):

- **The lemma is standard.** It is the step of batched backward search, a known technique.
- **The count doesn't move.** Each step of a frontier record composes one row with the next, and those steps match the walk's steps one for one. For G fields of d letters there are G·d steps either way. G1 says there must be, since every width-1 row is still visited.
- **The price doesn't fall.** Against independent lanes, which already bought only 5–14%, the frontier adds a partition pass moving at least 16 bytes for every step. This is a prediction, not a measurement, and the record names what would falsify it.
- **Finding the starting rows is easy.** It takes one scan of a contiguous range, so that was never the obstacle.

The record's most useful line is what the frontier *is*. Within one letter, the gap between two rows' destinations is the number of that letter between them. So moving a scattered set of rows amounts to counting letters in the gaps between them: an aggregate count, which an earlier result showed can't place a width-1 row without as many separate counts as there are rows.

Neither eureka was worthless. Both were results without their reasons, and the reasons were already in our journal.

## 4\. The question that survived: what lies between the routes

Hilbert's instruction survives the wreck intact, and both models had reached for it. Applied to mibz2 it asks: what transformation turns the walk into Cells, and Cells into Open? What does it keep, what does it remove, and what can it never remove? The same record answers it for our three decoders, with proofs.

The answer is **contraction along compose**. Think of a decoder as tiling the block's cycle with steps. Two consecutive steps can be replaced by their composite, and wherever a whole stretch of rows stays inside runs of equal letters, the composite covers the stretch in one step:

Three tilings of the same stretch of a block's cycle

Conceptual, not measured. The walk takes one step per row. Cells merges every stretch that stays inside runs. Open merges only the thick ones and walks the thin ones, trading steps for less construction. The width-1 rows are single steps in all three.

What the transformation keeps, removes and can't touch:

|  | Contraction along compose |
| --- | --- |
| **It preserves** | the text the cycle spells; the order of the cycle; that every row is covered once |
| **It removes** | one LF evaluation for every interior row of a stretch: w rows paid for by one translation |
| **Its invariant** | the set of width-1 rows, each a single step under every contraction |
| **On scattered rows** | it acts (the frontier carries order) but removes nothing: one count per row |

This is what Hilbert asked for: not a third route, but the map between the two we had, with its invariant. It also says where to look for a fourth decoder. An obligation disappears only where LF on a set of rows has a description much shorter than the set. Runs of equal letters are one such family. The open question is whether a scattered set, such as the rows of the random IDs, has another.

There is one sharp limit. The IDs' letters are random, about 4 bits a row, so any description of where those rows go needs at least 4 bits per row. That bounds the *size* of the description, not the *number of steps* needed to apply it. A description with few, wide steps isn't ruled out. That gap is where the next decoder would have to live.

Claude's analysis raised the standard escape from lower bounds of this kind: naming a composite once and reusing it, as extension variables shorten proofs in propositional logic. Checked, G1 already charges shared work only once, and [survives](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>). What would break it is a schematic lemma, one proof with a parameter that ranges over a scattered set. That is the same open question in another form.

## 5\. Where things stand

The essay's charge holds up as a warning, and it applied to us twice in one afternoon. Two confident breakthroughs arrived without reasons. Checked against what we had already measured and proved, one was refuted and the other was a known lemma that leaves the cost unchanged. What survived was a question about reasons: what lies between our decoders, and what no contraction between them can remove.

The split between count and price stays open as its own question. G1 bounds the number of width-1 steps, not what each one costs on a real processor, and our records already hold examples of the two pulling apart in each direction. One shortcut is ruled out: checking a guessed decoding against bzip2's 32-bit block checksum would accept a wrong guess about once in four billion, and mibz2 must be exact on every block.

What changes is the record each failed idea leaves. Our obstruction write-ups already keep, for each closed approach, its counterexample, its scope and what would get past it. Since this afternoon they also keep what the approach taught: the capability it revealed even where it couldn't pay. That is the difference between a Hearer's saying and a Learner's reason, and it is what let us take both eurekas apart in an afternoon.

---

**Sources:**

- doomslide, "Against Mathematics: A religion of the dead, by the dead, for the dead", Substack, 24 October 2026. Quotations of Archimedes and Hilbert are from the essay.
- [The source record](<https://github.com/lmmx/mibz2/blob/b643d28ed5f3e631260fbd4f086e64cbbc4df0bc/docs/journal/2026-11-24-against-mathematics-source.md>): the prompt, ChatGPT's and Claude's analyses verbatim, and descriptions of ChatGPT's figures.
- [The ordered frontier and the transformation between decoders](<https://github.com/lmmx/mibz2/blob/c47a2a15e13de24b9cc7564a46e94be1fdc5d461/docs/journal/monodromy/2026-11-24-ordered-frontier.md>) and [abbreviation, batching and the block checksum](<https://github.com/lmmx/mibz2/blob/c03c7ab277dd23fc12f9361142a68c4e785f9faa/docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md>), the mathematician agent's records.
- [Toy-transfer](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-20-toy-transfer.md>), the lanes experiment, and [the retrospective](<https://github.com/lmmx/mibz2/blob/f7c0f03d6613cd11a5350ad480d4691157af4e95/docs/journal/2026-11-24-retrospective-proposal.md>) on the squared map.
