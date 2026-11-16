[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/index.md>)

# Stop and walk: cutting a text rebuild short and walking the rest

Halting each block's rebuild early and walking the small loop that remains spares mibz2 a third of the text pieces it stores, for two per cent more instructions.

16 November 2026 · 16 min read · [Research in progress](<#7-what-decides-it>)

[Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-16-lazy-letters/index.md>) made mibz2's Cells kernel keep only the bookkeeping a block's rebuild needs at each moment, and from 8 threads up that made the decoder faster. This post is about the other half of what the kernel keeps: the pieces of text it assembles along the way. *Stop and walk* is a new way to finish the rebuild that never makes a third of them. It is built and checked, its instructions are counted, and the run that decides whether it ships has not happened yet.

**The recap.** mibz2 is our bzip2 decoder, written in Rust. bzip2 compresses text in blocks of up to 900 kB, and stores each block as the last column of its sorted rotations: the Burrows–Wheeler transform (BWT). Undoing it means following a map, LF, from row to row of the sorted block, one byte per row. Within a *run*, a stretch of equal bytes in that column, LF moves every row by the same amount. So the map is one shifted interval per run, not one entry per row ([the post of 8 November](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-08-inverse-bwt-navigation/index.md>)).

**Cells, in one paragraph.** [Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) rebuilds a block from those intervals without visiting its rows. Each step takes the two intervals at the right-hand end of the map. Where one interval lands on the end of the other, the rows it lands on can only be passed through, so the step deletes them and gives the shorter interval a longer piece of text to stand for. The code calls an interval a *letter* and the text it stands for its *word*. This shrinking is a classical idea, Rauzy–Veech induction, and [section 1](<#1-how-does-cells-rebuild-a-block>) runs it by hand on a three-byte block. When one letter is left, its word is the whole block, and a final pass, the *writer*, expands it into bytes.

**What the kernel holds.** It holds two things while it works. The first is the letters' bookkeeping: their lengths, links and places. [Hand-me-down slots](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-15-slot-recycling/index.md>) and [smaller second-order slots](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-15-bottom-slots/index.md>) trimmed it, and lazy letters then cut its slots from 20 bytes a letter to 6.4. The second is the words. A word of up to 16 bytes is kept as its bytes, and a longer one as a 16-byte rule naming its two halves. Those *word bytes* average 15.6 a letter on mibz2's Wikidata benchmark, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB. That is now about half of the 29.5 bytes a letter the lazy kernel touches besides its output ([the ring's record](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-ring.md>)). Nothing so far has reduced them.

**Why memory matters now.** Since mibz2 0.5.1, every thread count on our 10-core test machine, an Intel i9, decodes with one Cells kernel per thread. From 8 threads up it is the lazy kernel. Before that, the highest thread counts ran teams of threads sharing one block, so earlier memory cuts to Cells never reached them. A cut to the lazy kernel's working set now reaches every count from 8 to 20 ([section 6](<#6-why-test-it-now>)).

**Stop and walk.** The full induction can't write a word into the output until its last steps: only then does a word's place become known. So it holds all its words to the end. Stop and walk stops the induction early, once it has made 1.5 cells (steps) for every run, about four fifths of the way through. The map left at that point is a single loop of points, about one for every 22 rows of the block. One point's place in the output is known. So a walk round the loop from it can place every other point's word just below the last one. The words the last fifth of the steps would have made are never made. It trades building and storing words for moving round a map.

## What was found

The takeaways, each shown in a section below:

- **A third of the word bytes are never made.** Stopping at 1.5 cells a run leaves 33% of the word bytes unmade. That is 5.0 bytes a letter, counted by a prototype on all 233 large Wikidata blocks. It is about a sixth of everything the lazy kernel touches. No drop in memory traffic or cache misses has been measured yet ([section 2](<#2-why-does-the-rebuild-hold-its-words-to-the-end>), [section 4](<#4-where-should-it-stop-and-what-does-it-cost>)).
- **The walk writes each word once and copies it after.** It goes once round the loop, one point at a time. A long word is expanded where the walk first meets its letter, and every later point copies it from there. The built kernel gives the same bytes as the shipped one on all 431 blocks of the Wikidata extract ([section 3](<#3-how-can-a-walk-place-the-words>)).
- **It costs 10 to 12 instructions a letter.** Counted over the whole decode, that is about 2% more instructions than the shipped decoder, inside the +20 a letter the build had to stay under. On one thread it is about 10% slower in kernel time, because the walk waits on one memory load after another. That timing is from a cloud machine, so it is a sign and not evidence ([section 4](<#4-where-should-it-stop-and-what-does-it-cost>)).
- **It was found by failing first.** A first build cost +97 instructions a letter. Its report listed the two terms the model had missed, and a new construction aimed at exactly those two came back within about fifteen minutes ([section 5](<#5-how-was-it-found>)).
- **Whether it ships is for a 10-core run to say.** It is not adopted. It would replace the lazy kernel everywhere that kernel runs, from 8 threads up, or not ship at all. It never gets a thread range of its own ([section 7](<#7-what-decides-it>)).

Run to the end, or stop at 1.5 cells a run and walk

Above and middle: conceptual bars, with counted figures. Wikidata's blocks need about 1.8 cells a run in all; stopping at 1.5 leaves 33% of the word bytes unmade (a prototype's count over 233 blocks). Below: the analogy. Each bead is a point of the map left at the stop, its colour is the letter it belongs to, and the letter's word is the bead's piece of text. The real loop has about one point for every 22 rows. The analogy stops holding for the induction itself: it does not thread beads one at a time, but deletes whole intervals of points at each step, under one rule per letter.

**Where things stand.** Stop and walk is built as a set of test variants of the lazy kernel, and the default is unchanged. The run that decides it is a timing on the i9 from 8 threads to 20, set out in [\#493](<https://github.com/lmmx/mibz2/pull/493>).

*The result of that run will appear here once it is in.*

## 1\. How does Cells rebuild a block?

The terms this post uses, each worked through on an example below:

| Term | What it is in mibz2 |
| --- | --- |
| run | a stretch of equal bytes in the BWT's last column |
| letter | an interval of rows that the map moves as one; at first, one run |
| word | the bytes a letter's interval stands for; one byte at first, longer as the rebuild goes on |
| cell | the overlap of an interval in one order with an interval in the other; one is made at each step |
| induction | replacing the map by a smaller one that skips some rows, while recording their bytes in words |
| rule | a long word kept as its two halves, such as X = AB, each an existing word |

Take the three-byte block `aab`. Its rotations, sorted, are `aab`, `aba` and `baa`, so its last column reads `baa`. That column has two runs: `b` in row 0 and `aa` in rows 1 and 2. Call them B and A. LF sends each row to the row of the rotation one byte earlier. It sends B's row to row 2, and A's two rows to rows 0 and 1, both shifted by the same amount. Followed from row 0, LF visits row 2, then row 1, then row 0 again: one cycle through every row, reading the block's bytes backwards.

The block aab as two intervals and one cycle

Worked by hand. On the left, the sorted rotations and their last bytes. On the right, the two runs as intervals, in the last column's order and in the order LF lands them. Moving the intervals around without reversing any of them is what makes this an interval exchange. Section 1 of [the Cells post](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) runs the same block.

**One step of the induction.** Look at the right-hand end of both orders. In the last column, the last interval is A, two rows long. Among the places LF lands, the last is row 2, where B lands. B is the shorter, and it lands inside A, at A's end. Row 2 is reached from nowhere else, and from row 2 the path always goes on to row 1. So row 2 can be deleted, as long as its byte is kept. B's word grows from `b` to `ab`: A's `a` followed by B's own `b`, in the order the rebuild records them. Both intervals are now one row long, and the map is 0 → 1 → 0.

One step: row 2 goes, its byte stays in B's word

Worked by hand, on the block aab. The smaller map skips row 2, and nothing is lost: its byte is now part of B's word. Each move of the smaller map stands for the run of moves of the old one that it skips, which is what a first-return map means.

On a real block each step does the same at the right-hand end, deleting a whole interval's worth of rows at once and recording the new word. A long word is recorded as a rule naming its two halves, so it costs 16 bytes whatever its length. When only one letter is left, its word is the whole block. On `aab` one more step would do it: A and B are now the same length, so they join into one letter whose word is `aab`.

**The rule that makes the walk possible.** Moving along the smaller map moves through the text by exactly the length of the word that move stands for. If \\(p(x)\\) is the place in the block of the text at point \\(x\\), \\(T\\) is the smaller map, and \\(w\_\\alpha\\) is the word of the letter \\(\\alpha\\) that holds \\(x\\), then

\\\[ p(T(x)) \\equiv p(x) - |w\_\\alpha| \\pmod n , \\\]

where \\(n\\) is the block's length. The word \\(w\_\\alpha\\) lies just below \\(p(x)\\) in the text, and \\(T(x)\\) starts where it ends. On `aab` after one step, point 0 holds `ab`, of length 2, and point 1 holds `a`. Going round the loop once moves through 2 + 1 = 3 bytes, the whole block. This is invariant I, as stated in the [first proposal's record](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>), and it holds at every stage of the induction, not only at its end.

## 2\. Why does the rebuild hold its words to the end?

A word is only useful once it is in the output, and to put it there the writer needs its place: the offset in the block where its bytes begin. Cells learns places from one row only. bzip2 records which row of the sorted block is the original text's first rotation, and Cells keeps that row marked as a letter of its own. A word's place is fixed only once the word has become part of the word around that row. In Cells' order of steps, that word grows only in the last steps, so until then almost no word's place is known.

That has a direct consequence for memory. [A census of every word](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-words.md>), over Wikidata's 233 blocks of at least 65,536 rows, followed each word from when it was made until nothing needed it. It found that 72% of the word bytes are still needed when the induction ends, because the writer reads them. The other 28% die earlier. A short word that every longer word built from it has copied is dead once its letter no longer holds it.

Those 28% can't be given back cheaply. A dead word is almost never at the top of the store where it was written: its letter kept it for a long time while other words were added above it. A store that only frees from the top, the cheapest kind, reclaims none of them on any block. Reclaiming them needs a free list or a compaction pass, which is bookkeeping for every letter, in a loop that is already short of registers. Writing words out early doesn't help either. A word can't be written before its place is known, and places become known only at the very end.

So the census closed with a lower bound. **While places come only from that one row, every word not yet inside the word around it has to be kept.** In Cells' order that is the writer's whole set of words until the last step, which is exactly what the kernel holds. The words weren't too big; their places weren't known yet. The one way out the census named was to learn places some other way: by walking.

## 3\. How can a walk place the words?

Stop the induction early. What is left is a smaller map of the same kind: a set of points, each belonging to a letter, and a rule for moving from a point to the next. Because LF passes through every row of a block in one cycle, the smaller map also forms one loop through all its points. The rule of [section 1](<#1-how-does-cells-rebuild-a-block>) says each move round the loop passes over exactly one letter's word. So once one point's place is known, every other place follows from the word lengths.

One point's place is known. The first proposal proves that the mark on bzip2's starting row always stays on an interval one row long, so at the stop it is a single point. Its place follows from the row's offset within its word, which the writer already finds. The stopped map still holds everything the rebuild needs: where each letter's intervals lie in both orders, each letter's word as bytes or rules, each word's length, and that mark. The induction has discarded nothing; it has only changed how the text is described.

**Two ways to finish.** On `aab`, after one step, ordinary Cells would take the second step and then expand `aab`. Stop and walk instead stops there and walks the two points.

Two ways to finish aab

Worked by hand. The places follow the rule of section 1: each point's word ends at its place, and the next point's place is this one's less the word's length. Here the walk is started at point 0 for the illustration; in the kernel it starts at the marked point.

**The walk in the kernel.** [The reopened design](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>) walked the loop upwards, from low places to high, following the map backwards. The built kernel, in [\#493](<https://github.com/lmmx/mibz2/pull/493>), goes the other way. It follows the map forwards from the marked point, writing each point's word so that it ends at the point's place, and moving down the output. After as many points as the loop has, it is back at the marked point, one block lower: the whole block is written. If the loop closes too early, or the bytes don't add up to the block, the block is decoded again the shipped way.

**Expand once, then copy.** A letter's interval may hold many points, and each has the same word. Expanding a long word from its rules at every point would repeat the same work, which is what the first build did. The walk expands a long word the first time it meets its letter, and notes where it wrote it. Every later point of that letter copies the bytes from there, so the output itself serves as a store of words already expanded. A short word kept as bytes is copied straight from where the induction stored it. The copy is a fixed 32 bytes, then 16 at a time for a longer word, so the walk doesn't branch on a word's length.

Writing abracadabra from a loop of four points

Conceptual: a made-up loop, not a real block's. The walk moves down the output, so the first X (places 7 to 10) is written before, and above, the copy at 0 to 3. In a real block the loop has about one point for every 22 rows.

**Why the wide stores are safe.** A 32-byte copy of a 4-byte word writes 28 bytes too many. They land below the word, at places the walk hasn't reached yet, because places only fall as it goes. Each of those bytes is written again, correctly, when the walk gets there. A copy's source is always the first occurrence, which lies wholly above, so it was finished before it is read. Where this can't hold, near place 0 where the walk wraps round the block, and in its last 32 bytes, just above where it began, each word is written exactly. The first proposal's eight-lane writer had a failure of exactly this kind. Its lanes wrote separate stretches side by side, and one lane's wide stores could overwrite bytes another lane had already written. A single ordered walk removes that case.

The only new table the walk needs is one 4-byte first place for each letter alive at the stop, about a tenth of the letters. As first built, its tables were sized to every letter and added about 20% to a worker's peak memory. They were cut to the live letters before any timing, and locally the peak is now the shipped kernel's: 37 against 37 MB on one thread, 83 against 85 on four ([the revised rule](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>)).

The first proposal's walk used eight lanes, so that the waits on memory overlapped, at the price of a measuring pass and repeated expansions ([section 5](<#5-how-was-it-found>)). The one-lane walk drops both, and pays with a single chain of dependent loads, which [section 4](<#4-where-should-it-stop-and-what-does-it-cost>) comes back to.

## 4\. Where should it stop, and what does it cost?

The stop needs a test, and the induction's inner loop has no room for one. That loop already uses more values than the processor has registers. The first build's stop, a check of the map's remaining size, cost about 10 instructions on every cell, whether the block stopped or not. The reopened design puts the stop on a count the kernel already keeps. The kernel refuses any block that would take more cells than a budget allows, so it counts every cell down already. Stopping after ⌈β r⌉ cells, where r is the block's number of letters at the start, needs only a different starting count. The design priced that at nothing. In the build, the cleanest form of it spilled the counter to memory and cost 16 to 38 instructions on every cell, so the build compares the counter with a stored bound instead. That costs about 2 instructions a cell.

The stop is set as β r cells, not as a size of the map, because r is known before the induction starts and the total number of cells is not. On Wikidata a block takes 1.29 to 2.08 cells a run in all. Early in the induction most of the map disappears cheaply. Late in it few rows are left, but each step still makes a long word. Stopping earlier saves more words and leaves more points to walk, so β = 1.5 is a compromise read off the counts, not an optimum. The prototype counted three settings over the 233 large blocks:

| β (cells a run at the stop) | blocks that stop | map left, as a share of the rows (median) | points walked a letter | word bytes never made |
| --- | ---: | ---: | ---: | ---: |
| 1.4 | 232 of 233 | 0.082 | 1.23 | 43% (6.5 B a letter) |
| 1.5 | 232 of 233 | 0.046 | 0.66 | 33% (5.0 B) |
| 1.6 | 232 of 233 | 0.022 | 0.34 | 23% (3.4 B) |

A block that takes fewer cells than β r never stops, and is rebuilt as before. The table shows the striking thing. By the stop at 1.5, the map is down to under a twentieth of the rows, yet a third of the word bytes are still to come. What the count measured is word storage, the rules and the stored bytes. It is not the process's memory, and no drop in memory traffic or cache misses has been measured.

**What it costs.** The model and the built kernel disagree, and the kernel is what counts. The prototype predicted that stopping at 1.5 would save about 20 instructions a letter against the shipped kernel. [The kernel's count](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920005870>), over the whole decode of the Wikidata extract with callgrind, an instruction counter, came out the other way. A letter here is one run of the 11,090,867 in the blocks Cells takes.

| β | whole decode, first build to pass | a letter, against the shipped lazy kernel | a letter, on the head the i9 run uses |
| --- | ---: | ---: | ---: |
| shipped | 6.076 G instructions |  |  |
| 1.4 | 6.360 G | \+25.6 | \+27.3 |
| **1.5** | **6.196 G** | **\+10.8** | **\+12.3** |
| 1.6 | 6.154 G | \+7.0 | \+8.1 |

The build had to stay under +20 a letter before any timing, and β = 1.5 does. The counter itself varies by about 10 a letter between builds of the same source. So the claim is that β = 1.5 passes the gate, not that it costs exactly 11 or 12 instructions, and the two heads' figures are the same within that noise.

The trade: word bytes never made, against instructions added

Counted and modelled, not timed. The heights, the share of word bytes never made, are a prototype's counts over the 233 large Wikidata blocks. The filled points are the built kernel's whole-decode instruction counts on the head the i9 run uses (the first head to pass counted +25.6, +10.8 and +7.0); the hollow ones are the reopened design's model, after reading the stopped map. The first build stopped at a 32nd of the rows and left 26% unmade by its proposal's count.

The count also shows where the model went wrong. At β = 1.5, on the first head to pass, three terms differ. The cells never made were the cheap ones. The tail of the induction reads no new runs, so each of its cells costs about 38 instructions, not 71. Skipping them saves 13.2 a letter, not 21.5. The walk costs 52.1 instructions a letter, against the shipped writer's 49.5, where the prototype's standalone walk cost 38.3. Each point takes about 31 instructions of fixed work: a rank lookup, a bit count and a read of the letter's record. Expanding each letter's first occurrence adds 14.6 a letter. And reading the stopped map into the walk's tables costs 18.9, not the 10 the model allowed. About 6,000 letters are alive in a block at the stop, and each is read twice.

**Time on one thread.** On a cloud machine, with the kernel's phases timed, β = 1.5 was about 10% slower than the shipped kernel. The walk runs about the writer's instructions in about twice the writer's time. Its step from one point to the next is a load, a bit count, a second load and an add. That takes about 20 cycles, and each step needs the one before it. That machine's timings are no evidence of speed, but they say what the decisive run has to show: that when many threads share the cache, the bytes saved buy back more than the walk costs. A second lane, walking the other way round the loop, would halve the chain, but needs more tables, so it waits on the result.

**What is known, and how.** The records label each statement as proved, counted or modelled:

| Status | What the records support |
| --- | --- |
| Proved | the stopped map keeps every path and its words; one loop with one known place gives every place; an ordered walk with first-place copies never overwrites a byte still needed |
| Counted or tested | a third of the word bytes never made at β = 1.5 (prototype, 233 blocks); +10.8 and +12.3 instructions a letter (kernel, callgrind); the same bytes as the shipped kernel on all 431 Wikidata blocks, the test files and 4,000 random columns |
| Modelled | how much the smaller working set helps at 8 threads and more; whether the cache's size, its misses or the bandwidth to memory limit it; what a second lane would save |
| Not done yet | the Miri and fuzz runs, the mutation pass and the review of the new unsafe code, which the first build's record lists as owed before any adoption |

## 5\. How was it found?

Two of the team's agents found the design in one day: MONODROMY, its mathematician, and the Pathfinder, its engineer. Most of what made it work came from a failure.

It began with a negative result. The words census of [section 2](<#2-why-does-the-rebuild-hold-its-words-to-the-end>) asked whether words could be freed before the induction ends, against a bar of about a third of the word bytes for no extra instructions. It found 28% freeable in principle, none of it cheaply, and proved the lower bound. Its one open door was walking.

The [first proposal](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>) walked through that door. It stopped when the map was a 32nd of the rows, and placed words from 256 ranked starts with eight lanes. The model gave 26% fewer word bytes for 0 to 9 more instructions a letter, and a prototype was byte-exact on all 233 blocks.

The [first build](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-stop-walk.md>) was also byte-exact, and cost +97 instructions a letter against a gate of +20, so it stopped before any timing. Its report did not stop at the total. It set each term of the model against what the kernel counted. The tail's saving was as modelled. But the stop test cost about 18 instructions a letter, on every cell, which cancelled that saving. And writing from the kernel's own word layout cost 104 instructions a letter, not the prototype's 78. The prototype had written from a layout built for it, whose building it did not count, and the kernel expanded each long word again at every occurrence.

[The Coordinator's log](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6916535874>) stopped the build and sent exactly those two terms back to the proposal's author as the question to reopen it on. Fourteen minutes later the [reopened design](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>) answered both. It moved the stop onto the cell budget's counter and replaced the eight-lane writer with the one-lane walk that expands each word once. The second build passed the gate at +10.8. It still missed the model by about 30 instructions a letter, and its report again named where.

The rescue, term by term: what each build added, in instructions a letter

Counted with callgrind over the whole decode of the Wikidata extract, in instructions per run of the blocks Cells takes, against the shipped lazy kernel; the rescue is its first head to pass. The two builds stop at different points, a 32nd of the rows and 1.5 cells a run, so the rows compare designs, not one stop. “The rest” is each total less its listed terms, worked out here, not counted. The first build's two largest avoidable terms, the stop test and writing the text, are the two the rescue was asked to remove.

That sequence is now one of the team's rules ([§11a of its protocol](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/team/PROTOCOL.md#11a-a-failed-gate-goes-back-to-its-author-term-by-term>)). A build that fails its gate reports each term of the model against what it measured, largest first. That list goes straight back to whoever proposed the design, who gets one round to remove those terms or to say that nothing can.

**Stopping twice doesn't help.** [A follow-up record](<https://github.com/lmmx/mibz2/blob/806979d4402945685da22a334bde4a63db83b470/docs/journal/monodromy/2026-11-16-tower.md>), in review as [\#492](<https://github.com/lmmx/mibz2/pull/492>), stopped again on the walk's own tail. It is byte-exact, but describing each extra level of the map costs about as much as the words it saves: the best two-level tower saves 1.3 bytes a letter more, for about 16 to 26 more instructions a letter. The single stop at 1.5 stays the design.

## 6\. Why test it now?

The kernel's bytes matter only where threads compete for the cache; on one thread, what costs time is instructions. That is why lazy letters lost at 4 and 6 threads and won from 8. So the question is which thread counts run the lazy kernel, and until recently the answer was few of them. [Why mibz2 stops scaling](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) showed that past about 8 threads, each block decoded at once slowed the others. mibz2 answered with teams: [three teams of three](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-07-pipelined-teams/index.md>) sharing a block each at 10 threads, later with [two service threads](<https://lmmx.github.io/mibz2/pr-preview/pr-493/blog/2026-11-09-pit-crew-service-threads/index.md>) taking each block's serial steps. Those teams ran their own kernel, so Cells' memory cuts reached only 8 threads and below.

That changed in 0.5.1. [\#462](<https://github.com/lmmx/mibz2/pull/462>) gave 10 threads workers with lazy letters, one block each. [\#468](<https://github.com/lmmx/mibz2/pull/468>) did the same for machines with 8 to 10 physical cores asked for more threads than that. They run one lazy worker a core, and leave the second hardware threads idle. The thread plan in the code, [`plan_with`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/src/parallel.rs#L2680-L2706>), now runs on the 10-core i9 as follows:

| Threads asked for | What runs |
| --- | --- |
| 1 to 7 | a worker a thread, eager Cells |
| 8 to 10 | a worker a thread, lazy letters |
| 11 to 20 | 10 workers, one a core, lazy letters |

Teams now run only on other machines. A machine with 6 or 7 physical cores gets teams of two when asked for more threads than it has cores. So does a machine with 11 cores or more, asked for 11 threads or more. Where the machine's topology can't be read, which includes every system other than Linux, mibz2 runs a worker a thread at every count.

[The release ledger](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/release-attribution.md>) shows why that matters. It lists, for each release, which merge moved which thread count on the i9 by 3% or more. From 0.2.4 on, 10 to 20 threads moved in only three ways. Two were changes to how threads were grouped: the teams, then pipelining them. One was an existing kernel ported to the teams. The third was 0.5.1's move onto Cells, which sped up 10 threads by 7.3%. Cells' memory cuts, from the 20-byte slot to lazy letters, each moved 8 threads or fewer, because teams ran above. Now a cut to the lazy kernel's working set reaches every count from 8 to 20. Stop and walk trades instructions for memory, the trade lazy letters won and [a ring of shared buffers](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-ring.md>) lost. The ring cut the lazy kernel's bytes by 15% for 13% more decoder instructions, and was slower at 8 threads and level at 9 and 10. By the prototype's count, stop and walk would cut about a sixth of those bytes, 5.0 of 29.5 a letter, for about 2% more instructions.

## 7\. What decides it?

A timing on the i9, [\#493](<https://github.com/lmmx/mibz2/pull/493>), under [a rule fixed before the data](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>). The test decoder is the shipped lazy kernel's, with the same workers, plan and window, and only the kernel swapped. It decodes four copies of the Wikidata extract at 8, 9, 10, 12 and 20 threads, in ten paired rounds, each running the shipped kernel and the three settings of β. At each count, β = 1.5 is faster if the 95% interval of its wall-time ratio lies wholly below 1, and slower if wholly above.

Stop and walk becomes *the* lazy kernel, everywhere lazy letters run now, if three things hold. β = 1.5 must be faster at one of those counts or more, slower at none, and its peak memory no more than 10% above the shipped kernel's at every count. Faster at some counts and slower at others is a finding, not an adoption. Stop and walk gets no thread range of its own, and moving the lazy threshold would be a separate decision. Because its verdict can change a default, the rule is itself under one review before the run. The run also records two figures without judging them: how much slower each setting is on one thread, and how much more processor time it takes at 10 threads than at 1, which is where a smaller working set should show.

*The verdict, with each thread count's ratio and interval, will appear here once the run is in.*

The chain, in short. The full rebuild holds its words to the end because it learns their places only in its last steps. Walking the small map left after an early stop learns the places another way, so the last fifth of the steps, and a third of the word bytes, are never made. A first build failed on two costs its model had missed, and a design aimed at those two passed at about +11 instructions a letter. Since 0.5.1 the lazy kernel runs from 8 threads to 20 on the i9, so a memory cut to it reaches the whole range where threads contend for the cache. Only a timing there can say whether a third of the words is worth a walk that is slower on one thread. Whichever way it goes, there is now a second way to finish an induction, with a proof that it is correct and a kernel that is. And when a design fails against its own model, the list of terms where it failed is the most useful thing it produces.

---

**Research records:**

- The words census and its lower bound: [`2026-11-16-words.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-words.md>).
- The first proposal, with ranked starts and invariant I: [`2026-11-16-stop-and-walk.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>).
- The first build and its divergences: [`2026-11-16-lazy-stop-walk.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-stop-walk.md>), and [the review on the team channel](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6916500604>).
- The decision to stop it and the reopening question: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6916535874>).
- The reopened design: [`2026-11-16-stop-walk-reopened.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>).
- The second build's counts and divergences: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920005870>). The rule for the i9 run, the memory fix and the current head's counts: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>). The build and the recipe: [\#493](<https://github.com/lmmx/mibz2/pull/493>).
- Stopping twice: [`2026-11-16-tower.md`](<https://github.com/lmmx/mibz2/blob/806979d4402945685da22a334bde4a63db83b470/docs/journal/monodromy/2026-11-16-tower.md>) ([\#492](<https://github.com/lmmx/mibz2/pull/492>)); the loops in the induction's path: [`2026-11-16-loops.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-loops.md>).
- Which merge moved which thread count: [`release-attribution.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/release-attribution.md>); the rule for a failed gate: [PROTOCOL §11a](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/team/PROTOCOL.md#11a-a-failed-gate-goes-back-to-its-author-term-by-term>).
- The thread plan: [`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/src/parallel.rs#L2680-L2706>).
