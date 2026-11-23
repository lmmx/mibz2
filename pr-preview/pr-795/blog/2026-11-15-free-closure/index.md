[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-795/blog/index.md>)

# The free closure of an induction: where free steps come to rest

The steps that cost mibz2 no bookkeeping can be taken in any order and still meet at one end point, now proved to belong to the block itself.

15 November 2026 · 13 min read · [Research result](<#5-what-is-proved-and-what-is-counted>)

[The width of an induction](<https://lmmx.github.io/mibz2/pr-preview/pr-795/blog/2026-11-15-induction-width/index.md>), the previous post, looked at mibz2's experimental decoder for bzip2. It found that the decoder can hold about a third as much at its busiest moment if it takes its *free steps* first. A free step is a step of the rebuild that erases one boundary between pieces of the block and creates none. That post also reported that the free steps always reach the same end point, whatever order they run in. This post is about that end point: why the order doesn't matter, how it was proved and checked, and what it allows a decoder to do.

**The recap.** bzip2 sorts the rotations of each block and stores the last byte of each, the *last column*; each sorted rotation is a *row*. To decode, mibz2 follows a map called LF from row to row, reading one byte per move, which spells the block out backwards. A *run* is a stretch of equal bytes in the last column, and LF moves each run's rows together, as one interval. [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-795/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) introduced Cells, the decoder that works on those intervals instead of on rows. Each of its *steps* cuts some rows out of the map, and lets the interval whose rows used to land there jump straight over them. The code calls each interval a *letter*, and the bytes its jump reads, in text order, its *word*. A step happens at a *coincidence*: a boundary where one letter's rows end and another letter's landing place ends too. Every letter starts with a one-byte word, and when the last step is done, a word spells the whole block.

**The width post in brief.** The *width* of a rebuild is the most letters with a word longer than one byte that it holds at once. Cells always steps at the right-hand end of the map. Its width is about 0.32 r at the median on Wikidata, where r is the number of runs in the block. Taking free steps first cuts that to about 0.08–0.11 r. On every block, the peak stayed within the number of letters left once the free steps run out.

**The theorem.** Take free steps in any order until none is left. You always reach the same letters, with the same words, after the same number of steps. The one exception is a cycle of the map that closes completely during the free steps. It takes no further part, and two orders can leave it in different places, with its word rotated. No real block closes its cycle that early.

**Why it matters.** Call the state where the free steps run out the *free closure*. Because the order doesn't matter, it is a property of the block, not of the algorithm that found it. On the 233 large blocks of the Wikidata benchmark it leaves **0.185 r letters** at the median, and takes **0.896 r steps** to reach. Those are counts, and they are now facts about each block: no cleverer order of free steps can change them. The proof is by hand, and a program checked its every case on 10.7 million pairs of steps, with no exception.

**What it allows, and what it doesn't yet.** A decoder may take free steps in whichever order is cheapest to find. Free steps on different letters may go to different threads and still meet at the same state. So one block's rebuild can be split between threads, and it is proved to stay correct. No such decoder has been built or timed.

[Section 1](<#1-what-does-one-step-do>) works one free step on a four-byte block. [Section 2](<#2-does-the-order-matter>) takes its two free steps in both orders. [Section 3](<#3-why-does-it-hold-for-every-block>) gives the proof's shape, and [section 4](<#4-how-was-it-checked>) the machine check. [Section 5](<#5-what-is-proved-and-what-is-counted>) separates what is proved from what is counted. [Section 6](<#6-what-could-a-decoder-do-with-it>) says what a decoder could do with it, and [section 7](<#7-what-is-still-open>) what is open.

## 1\. What does one step do?

Suppose you want to compress the four bytes `abac`. Their four rotations, sorted, are `abac`, `acab`, `baca` and `caba`, so the last column is `cbaa`. It has three runs, `c`, `b` and `aa`, so the rebuild starts with three letters. Call them Y, Z and X:

| Letter | Its rows (top) | LF sends them to (bottom) | Word |
| --- | --- | --- | --- |
| Y, the run `c` | row 0 | row 3 | `c` |
| Z, the run `b` | row 1 | row 2 | `b` |
| X, the run `aa` | rows 2–3 | rows 0–1 | `a` |

The figure draws each letter twice. The top strip shows where its rows sit in the last column. The bottom strip shows where LF sends them, and the shaded band joins the two. The numbers above the strips count boundaries between rows, so boundary 4 is the end of row 3.

One free step on the column of `abac`

Computed by a program that steps the column exactly as the checker in section 4 does; not drawn by hand. Top strip: each letter’s rows. Bottom strip: where LF sends them. Each box shows the letter and its word. Grey boxes are rows the step has cut.

Two things end at boundary 4. In the top strip, X's rows 2–3 end there. In the bottom strip, so does Y's landing place, row 3. That makes boundary 4 a coincidence, and a step can be taken there. Y's one row lands on row 3. Row 3 belongs to X, which sends it on to row 1. So row 3 is only a stopover on Y's way. The step cuts row 3 out, and Y now jumps from row 0 straight to row 1. Its jump reads `c` and then `a`. LF runs backwards through the text, so in text order its word grows at the front: `c` becomes `ac`, which is indeed in `abac`. X loses the row, and keeps row 2, which still goes to row 0.

The step is free because of where Y now lands. Its new landing place starts a boundary at 1 in the bottom strip. The top strip already had a boundary at 1, between Y's row and Z's. So the step erased the boundary at 4 and created none that wasn't already there. A step that is not free, a *paid* step, would put a boundary at a new place, and some later step would have to remove it. A step between two letters of the same length is always free, since one letter simply takes over the other's place.

## 2\. Does the order matter?

The column has a second free step available at the start. At boundary 2, Z's row 1 ends in the top strip. X's landing place, rows 0–1, ends there in the bottom strip. Here Z is the shorter piece, so row 1 is cut. X's row 3 used to land on row 1 and go on with Z to row 2. Now it jumps straight to row 2. That one row splits off from X and becomes Z, with the word `ba`.

Both steps are free, and they share a letter: X. Each one changes X's length, so neither leaves the other's starting point untouched. The figure takes them in both orders.

Two free steps, two orders, one state

Computed, as in the figure above. Green lines mark the free steps available in each state. Taking the step at 4 first and then the one at 2, or the other way round, gives the same letters, places and words: X at row 2 with the word `a`, and Y at row 0, landing on row 2, with the word `bac`.

The left path makes Y's word `ac` first, and then the step at 2 puts `b` in front of it. The right path makes Z's word `ba` first, and then the step at 4 hands Y the whole of `ba` followed by its own `c`. Either way, Y ends with `bac` and X with `a`. In the language of rewriting systems this is a *diamond*, or a *square*. Two steps leave one state, and one step on each side brings them back together.

The tiny block goes on to its end. After the diamond, X and Y form a cycle of two letters, each landing on the other's rows, and one more free step closes it whole. A program tried every one of the twelve orders in which the free steps of `cbaa` can be taken. All twelve take three steps. Each ends with one letter whose word is `abac`, `acab`, `baca` or `caba`: the same cycle, read from different starting points. That is the exception in the theorem. On a four-byte block the whole text closes during the free steps. On the Wikidata blocks, where the free steps stop at about a fifth of the runs, none did. bzip2 stores with every block the row of the original rotation, which says where to start reading in any case.

## 3\. Why does it hold for every block?

Two diamonds on one example prove nothing in general, so the proof goes through a classical result. Newman's lemma, from 1942, says that a system of rewriting steps has a single end point from every start if two things hold.

1. **Termination.** No sequence of steps goes on forever. Each free step erases one boundary and creates none, and a block has only finitely many boundaries, so this holds.
2. **Local confluence.** Whenever two steps are available in the same state, there is some way to continue from each of them so that both reach the same state again.

Termination is what lets the second, local condition decide the global answer. Without it, two paths could keep diverging forever, each step of the way looking reconcilable.

So the proof takes any two free steps available together, at boundaries c₁ and c₂, and shows that they can be brought back together. It rests on two facts. First, a free step touches only its own two letters, and removes no boundary but its own. Second, the place where a free step's shorter letter lands always lies strictly inside the longer letter's interval, so it can never be the other step's coincidence. Then two steps relate in exactly one of four ways:

1. **No letter in common.** Neither step changes anything the other reads, so each stays free after the other, and the two orders give the same state.
2. **One shared letter X**, whose rows end at c₁ and whose landing place ends at c₂. This is the case of `abac`, and the hard one. Each step can be one of three kinds: the letter above wins (it is longer), the letter below wins, or the two are equal. That gives nine combinations of kinds for the pair. The table below shows which can happen.
3. **One shared letter, the other way round.** This is case 2 with the steps' names swapped.
4. **Both letters shared.** The two letters form a cycle of two, and either step closes it. The two results differ only in where the closed letter lies and in which rotation of its word it holds. That is where the theorem's exception comes from.

In case 2, X is the upper letter at c₁, over Y, and the lower letter at c₂, under Z. Rows compare X with Y at c₁, and columns compare X with Z at c₂:

| At c₁ ↓ / at c₂ → | Z longer | X longer | equal |
| --- | --- | --- | --- |
| **X longer than Y** | impossible | square; Y and Z equal | impossible |
| **Y longer than X** | square | impossible | square |
| **equal** | square | impossible | square |

The four impossible combinations fail for one reason. For both steps to be free, one of them would have to land strictly inside a letter's interval in the other strip, where no boundary can be. The five squares are each written out in both orders in the proof, and each reaches the same letters, places and words. `abac` is the square in the top row. X has two rows, so it is longer than Y and longer than Z. Y and Z have one row each, as forced.

A last detail settles the step count. Every square closes with one step on each side, and the closing cycles of case 4 need none. So any two complete paths of free steps have the same length, not only the same end point.

## 4\. How was it checked?

The proof was checked against a program that steps every short column exhaustively. It covers every column over 2 letters up to length 14, and likewise over 3, 4 and 5 letters up to lengths 9, 7 and 6. On each column it follows three random complete paths of free steps. At every state along every path, it takes every ordered pair of free steps available, classifies it by case and kinds, and runs both orders. It checks that the two orders reach the same state. In case 4 it checks instead that one step closes a cycle, and that the two agree up to its rotation.

The results, over 10.7 million ordered pairs. *Same state* counts the pairs whose two orders reach the same state, and *closed cycle* the case 4 pairs that agree up to rotation:

| Alphabet, longest | Pairs | Same state | Closed cycle (case 4) | Exceptions |
| --- | ---: | ---: | ---: | ---: |
| 2 letters, 14 | 4,322,196 | 4,042,698 | 279,498 | 0 |
| 3 letters, 9 | 3,434,430 | 3,129,050 | 305,380 | 0 |
| 4 letters, 7 | 1,741,700 | 1,531,570 | 210,130 | 0 |
| 5 letters, 6 | 1,164,480 | 991,358 | 173,122 | 0 |

In every alphabet, the combinations of kinds that occur are exactly the ones the proof allows. Case 2 shows its five squares, and case 3 their mirror images. Case 4 shows only the three combinations that close a cycle. None of the four impossible combinations was ever seen. Rerun for this post, the program gives the same counts.

Before the proof, a separate count had run the free steps to the end under different orders and compared the results directly. No two orders ever disagreed, on 404,677 short columns under ten orders each, or on the 233 Wikidata blocks and four blocks of prose. On every real block the results were equal exactly, with no closed cycle to allow for.

## 5\. What is proved and what is counted

The research record labels each statement as proved by hand, or counted (exact for the data it ran on, but no more). For this result:

- **Proved:** free steps in any order reach one end point, up to closed cycles. A program checked every case on 10.7 million pairs of steps.
- **Proved:** every complete path of free steps has the same number of steps.
- **Proved:** after the free closure, no step adds a letter.
- **Counted, on all 233 Wikidata blocks under three orders:** before reaching the closure, free-first orders never held more long words than the closure has letters.
- **Counted, on all 233 Wikidata blocks:** the closure leaves 0.185 r letters and takes 0.896 r steps, at the median.

The two counted figures are now properties of each block, so their spread across blocks is a spread across text, not across algorithms. The Wikidata corpus is an extract of a Wikidata dump. Its blocks here are the 233 with at least 65,536 rows, and each figure is divided by that block's own r.

The free closure of each of the 233 Wikidata blocks

Counted, not timed. Each dot is one block, stacked where values coincide; hover a dot for its block and value. Upper: letters left when no free step remains, as a share of the block’s runs, from 0.075 r to 0.281 r. Lower: the free steps taken to get there, from 0.79 r to 0.99 r. Each block gave the same two numbers under every order tried, as the theorem says it must.

## 6\. What could a decoder do with it?

Everything in this section is a consequence of the theorem, not a measurement.

**Any order is correct.** The cost of a free-first decoder is in finding its free steps. It needs a worklist of ready steps. An earlier count priced that worklist at about 150 instructions a run on one thread, against 115 for Cells' sweep from the right-hand end. The theorem says a decoder may take its free steps in whatever order its worklist yields them: a stack, a sweep, or batches. The end point, the step count and the bound on width hold for all of them. So a worklist can be designed for speed alone.

**Splitting one block between threads.** Two free steps on different letters commute. So free steps can be handed to different threads, and they reach the same state as on one thread. The condition is that each thread's step writes only its own two letters' records. That is a way to split one block's rebuild that is proved correct, which matters for how mibz2 uses many cores. It decodes blocks in parallel either with each thread on a block of its own, or with teams of threads sharing a block. On a 10-core Intel desktop at ten threads, a block per thread took 1.040 times the teams' wall time in the latest run. Its 95% interval is 1.019 to 1.061 ([the run's report](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900029992>)). The working explanation, not yet measured directly, is that each thread's live state competes for the shared cache. A smaller live state, and a proved way to share one block's free steps, both bear on that.

**A bound fixed by the block.** After the closure, no step adds a letter. So from the closure on, a decoder that takes free steps first never needs more live letters than the closure has. That is about 0.19 r on Wikidata, against the r slots Cells reserves today. Before the closure, the bound held on every block counted.

No free-first decoder has been built. A first decoder that keeps the untouched one-byte letters implicit, and writes a letter only when its word first grows, has been built with Cells' right-end order and is being measured. None of it has been timed yet.

## 7\. What is still open?

**Which steps does the closure leave?** Only equal steps remove letters, and a block's rebuild makes exactly r − 1 of them. So the closure's letter count is 1 plus the number of equal steps that can only become ready after a paid step. Saying which equal steps those are, from the block's runs, would describe the closure without running it. That is the next question.

**One block on many cores, by rounds.** A different route also reaches the end of a rebuild: repeatedly cut the map down to the rows where runs branch, all at once. It collapses a whole Wikidata block to between 2 and 560 letters in 5 to 18 such rounds, and the work within a round is independent. Cells' right-end order, by contrast, has a chain of 93 dependent rounds at the median. As counted, though, each round walks every row it removes, so its work grows with the rows rather than the runs, which is the cost Cells exists to avoid. Whether a round can be computed on letters and their words alone has not been tested.

**Is width a cutwidth?** The width post links width to the *cutwidth* of a graph: line up its vertices, cut between any two, and count the edges that cross. Whether the width of a rebuild is exactly a cutwidth of the block's cycle of rows, bundled by runs, is a conjecture.

The record of the rounds count is [\#440](<https://github.com/lmmx/mibz2/pull/440>).

---

**Research records:**

- The confluence proof, its checker and the checker's output: [`2026-11-15-free-closure-confluence.md`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-free-closure-confluence.md>), [`square.rs.txt`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-free-closure-confluence/square.rs.txt>), [`square_output.txt`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-free-closure-confluence/square_output.txt>) (\#438).
- Width by order, the free closure's size and the confluence count: [`2026-11-15-width-orders.md`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-width-orders.md>), with the per-block closures in [`confluence_wikidata.jsonl`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-width-orders/confluence_wikidata.jsonl>) (\#437).
- The equal steps behind the closure's size, and the rounds count: [`2026-11-15-closure-intrinsic.md`](<https://github.com/lmmx/mibz2/blob/master/docs/journal/monodromy/2026-11-15-closure-intrinsic.md>) (\#440).
- The width census: [`2026-11-15-width.md`](<https://github.com/lmmx/mibz2/blob/c33d327cafedcdaf027cf73a953a7ea3647f50f2/docs/journal/monodromy/2026-11-15-width.md>) (\#435).
- The run of a block per thread against teams at ten threads: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900029992>).
- Newman's lemma: M. H. A. Newman, [On theories with a combinatorial definition of "equivalence"](<https://doi.org/10.2307/1968867>), *Annals of Mathematics* 43 (1942), 223–243.
