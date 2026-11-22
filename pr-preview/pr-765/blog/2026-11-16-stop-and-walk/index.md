[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/index.md>)

# Stop and walk: cutting a BWT inversion short and walking the rest

Halting each block's rebuild early spares mibz2 a third of the text pieces it stores, but on ten cores the slower walk outweighs the memory saved.

16 November 2026 · 17 min read · [Research result](<#6-what-did-the-timing-show>)

*Stop and walk* is a new way for mibz2 to finish rebuilding a block of text. It stops the rebuild partway, walks once round the small loop that is left, and writes the text out as it goes. The rebuild never makes the pieces of text its last steps would have built: a third of them, for about 2% more instructions. It did not ship. On our 10-core test machine it ran slightly slower, because the walk waits on one memory load after another, and those waits cost more time than the smaller memory saved.

**How it works, step by step.** mibz2's Cells kernel rebuilds a block by shrinking a map of it, one step at a time. Each step builds a longer piece of the text, a *word*. The full rebuild can't put any word in the output until near its end. So it keeps every word until then. Stop and walk stops partway instead, after 1.5 steps for every run of equal bytes in the block. That leaves one small loop of points. One point on it has a known place in the output. Walking round the loop from that point puts each point's word next to the one before. So the words the last steps would have built never get made.

**Why that could make mibz2 faster.** Fewer words mean less memory per block. That matters where many threads share one cache. Our test machine is an Intel i9 with 10 cores. Since mibz2 0.5.1, it decodes one block per thread at every thread count. From 8 threads up, mibz2 switches to the [lazy-letters](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-16-lazy-letters/index.md>) version of Cells. Before 0.5.1, the highest thread counts ran [teams](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) of threads sharing one block, and memory cuts to Cells never reached them. Now a cut to the lazy kernel reaches every count from 8 to 20 ([section 5](<#5-why-test-it-now>)).

**Where it stands.** We built stop and walk, and it gives the same bytes as the shipped kernel. We counted its instructions and timed it on the i9. It did not meet the rule we set before the timing, so mibz2 keeps the shipped kernel ([section 6](<#6-what-did-the-timing-show>)).

The rest of the opening gives the background the idea rests on.

**The BWT, in brief.** mibz2 is our bzip2 decoder, written in Rust. bzip2 compresses text in blocks of up to 900 kB. It stores each block as the last column of the block's sorted rotations: the Burrows–Wheeler transform (BWT). To undo it, a decoder follows a map, LF, from row to row of the sorted block, one byte per row. Within a *run*, a stretch of equal bytes in that column, LF moves every row by the same amount. So the map is one shifted interval per run, not one entry per row ([the post of 8 November](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-08-inverse-bwt-navigation/index.md>)).

**Cells.** [Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) shrinks that map step by step, and records each step as a word. Each step deletes rows that the path through the block can only pass through, and gives an interval a longer word to stand for the bytes it skipped. The code calls an interval a *letter* and each step a *cell*. When one letter is left, its word is the whole block, and a final pass, the *writer*, expands it into bytes. The technique is Rauzy–Veech induction ([the technical companion](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-12-rauzy-technical-companion/index.md>)), and [section 1](<#1-how-does-cells-rebuild-a-block>) runs it by hand on a three-byte block.

**What the kernel keeps.** It keeps two things while it works. The first is the letters' bookkeeping: their lengths, links and places. [Hand-me-down slots](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-15-slot-recycling/index.md>) and [smaller second-order slots](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-15-bottom-slots/index.md>) trimmed it, and lazy letters then cut its slots from 20 bytes a letter to 6.4. The second is the words. The kernel stores a word of up to 16 bytes as its bytes, and a longer one as a 16-byte rule naming its two halves. Those *word bytes* average 15.6 a letter on mibz2's Wikidata benchmark, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB. The lazy kernel touches 29.5 bytes a letter besides its output, so the words make up about half ([the ring's record](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-ring.md>)). No change so far has cut them.

**The numbers.** On Wikidata a block's rebuild takes about 1.8 steps a run in all, so stopping at 1.5 leaves out about the last fifth. At the stop, the loop has about one point for every 22 rows of the block.

## What was found

The takeaways, each shown in a section below:

- **The kernel never makes a third of the word bytes.** Stopping at 1.5 cells a run leaves 33% of the word bytes unmade. That is 5.0 bytes a letter, counted by a prototype on all 233 large Wikidata blocks. The saving comes to about a sixth of everything the lazy kernel touches. The timing showed the relief indirectly, as less extra processor time at 10 threads ([section 2](<#2-why-does-the-rebuild-hold-its-words-to-the-end>), [section 4](<#4-where-should-it-stop-and-what-does-it-cost>), [section 6](<#6-what-did-the-timing-show>)).
- **The walk writes each word once and copies it after.** It goes once round the loop, one point at a time. The walk expands a long word where it first meets its letter, and every later point copies it from there. The built kernel gives the same bytes as the shipped one on all 431 blocks of the Wikidata extract ([section 3](<#3-how-can-a-walk-place-the-words>)).
- **It costs 10 to 12 instructions a letter.** Over the whole decode, that adds about 2% to the shipped decoder's instructions. The build had to stay under +20 a letter, and it does. On one thread its kernel runs about 10% slower, because the walk waits on one memory load after another. A cloud machine took that timing, so it hints at speed and proves nothing ([section 4](<#4-where-should-it-stop-and-what-does-it-cost>)).
- **It did not ship.** On the 10-core i9 it ran 1% to 1.5% slower at 8, 9 and 20 threads, and we detected no difference at 10 and 12. The memory relief was real, but with one worker a core the walk's waits outweighed it. It could only have replaced the lazy kernel everywhere that kernel runs, and never had a thread range of its own ([section 6](<#6-what-did-the-timing-show>)).

Run to the end, or stop at 1.5 cells a run and walk

Above and middle: conceptual bars, with counted figures. Wikidata's blocks need about 1.8 cells a run in all; stopping at 1.5 leaves 33% of the word bytes unmade (a prototype's count over 233 blocks). Below: the analogy. Each bead is a point of the map left at the stop, its colour is the letter it belongs to, and the letter's word is the bead's piece of text. The real loop has about one point for every 22 rows. The analogy stops holding for the induction itself: it does not thread beads one at a time, but deletes whole intervals of points at each step, under one rule per letter.

**Where things stand.** We built stop and walk as test variants of the lazy kernel and timed them on the i9 from 8 threads to 20 ([\#493](<https://github.com/lmmx/mibz2/pull/493>)). Under the rule we set before the run, stop and walk is [not adopted](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6922810916>), and mibz2's default stays as it was. [Section 6](<#6-what-did-the-timing-show>) has the figures. The test variants stay on their branch.

## 1\. How does Cells rebuild a block?

This post uses these terms, each worked through on an example below:

| Term | What it is in mibz2 |
| --- | --- |
| run | a stretch of equal bytes in the BWT's last column |
| letter | an interval of rows that the map moves as one; at first, one run |
| word | the bytes a letter's interval stands for; one byte at first, longer as the rebuild goes on |
| cell | the overlap of an interval in one order with an interval in the other; each step makes one |
| induction | replacing the map by a smaller one that skips some rows, while recording their bytes in words |
| rule | a long word kept as its two halves, such as X = AB, each an existing word |

Take the three-byte block `aab`. Its rotations, sorted, are `aab`, `aba` and `baa`, so its last column reads `baa`. That column has two runs: `b` in row 0 and `aa` in rows 1 and 2. Call them B and A. LF sends each row to the row of the rotation one byte earlier. It sends B's row to row 2. It sends A's two rows to rows 0 and 1, shifting both by the same amount. Followed from row 0, LF visits row 2, then row 1, then row 0 again. That is one cycle through every row, and it reads the block's bytes backwards.

The block aab as two intervals and one cycle

Worked by hand. On the left, the sorted rotations and their last bytes. On the right, the two runs as intervals, in the last column's order and in the order LF lands them. Moving the intervals around without reversing any of them is what makes this an interval exchange. Section 1 of [the Cells post](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) runs the same block.

**One step of the induction.** Look at the right-hand end of both orders. In the last column, the last interval is A, two rows long. Among the places LF lands, the last is row 2, where B lands. B is the shorter, and it lands inside A, at A's end. Nothing else reaches row 2, and from row 2 the path always goes on to row 1. So the step can delete row 2, as long as it keeps its byte. B's word grows from `b` to `ab`: A's `a` followed by B's own `b`, in the order the rebuild records them. Both intervals are now one row long, and the map is 0 → 1 → 0.

One step: row 2 goes, its byte stays in B's word

Worked by hand, on the block aab. The smaller map skips row 2, and loses nothing: B's word now holds its byte. Each move of the smaller map stands for the run of moves of the old one that it skips, which is what a first-return map means.

On a real block each step does the same at the right-hand end. It deletes a whole interval's worth of rows at once and records the new word. The kernel records a long word as a rule naming its two halves, so it costs 16 bytes whatever its length. When only one letter is left, its word is the whole block. On `aab` one more step would finish it. A and B now have the same length, so they join into one letter whose word is `aab`.

**The rule that makes the walk possible.** Moving along the smaller map moves through the text by exactly the length of the word that move stands for. If \\(p(x)\\) is the place in the block of the text at point \\(x\\), \\(T\\) is the smaller map, and \\(w\_\\alpha\\) is the word of the letter \\(\\alpha\\) that holds \\(x\\), then

\\\[ p(T(x)) \\equiv p(x) - |w\_\\alpha| \\pmod n , \\\]

where \\(n\\) is the block's length. The word \\(w\_\\alpha\\) lies just below \\(p(x)\\) in the text, and \\(T(x)\\) starts where it ends. On `aab` after one step, point 0 holds the two bytes `ab` and point 1 holds `a`. Going round the loop once moves through 2 + 1 = 3 bytes, the whole block. The design's [record](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>) proves this as its invariant I. It holds at every stage of the induction, not only at its end.

## 2\. Why does the rebuild hold its words to the end?

To put a word in the output, the writer needs its place: the offset in the block where its bytes begin. Cells learns places from one row only. bzip2 records which row of the sorted block holds the original text's first rotation. Cells keeps that row marked, as a letter of its own. The kernel can fix a word's place only once the word has become part of the word around that row. In Cells' order of steps, that word grows only in the last steps, so until then it knows almost no word's place.

That has a direct consequence for memory. [A census of every word](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-words.md>) followed each word from when the kernel made it until nothing needed it. It covered Wikidata's 233 blocks of at least 65,536 rows. It found that the writer still needs 72% of the word bytes when the induction ends. The other 28% die earlier. A short word that every longer word built from it has copied is dead once its letter no longer holds it.

The kernel can't give those 28% back cheaply. A dead word almost never sits at the top of the store. Its letter kept it for a long time, while the kernel added other words above it. A store that only frees from the top, the cheapest kind, reclaims none of them on any block. Reclaiming them needs a free list or a compaction pass, which is bookkeeping for every letter, in a loop that is already short of registers. Writing words out early doesn't help either. The writer can't write a word before it knows the word's place, and it learns places only at the very end.

So the census closed with a lower bound. **While places come only from that one row, the kernel has to keep every word not yet inside the word around it.** In Cells' order that is the writer's whole set of words until the last step, which is exactly what the kernel holds. The words weren't too big; the kernel just didn't know their places yet. The one way out the census named was to learn places some other way: by walking.

## 3\. How can a walk place the words?

Stop the induction early, and it leaves a smaller map of the same kind. It has a set of points, each belonging to a letter, and a rule for moving from a point to the next. Because LF passes through every row of a block in one cycle, the smaller map also forms one loop through all its points. The rule of [section 1](<#1-how-does-cells-rebuild-a-block>) says each move round the loop passes over exactly one letter's word. So once the walk knows one point's place, the word lengths give every other place.

The walk does know one point's place. [The design's record](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>) proves that the mark on bzip2's starting row always stays on an interval one row long. So at the stop the mark sits on a single point. Its place follows from the row's offset within its word, which the writer already finds. The stopped map still holds everything the rebuild needs. It records where each letter's intervals lie in both orders, each letter's word and its length, and the mark. The induction has discarded nothing; it has only changed how it describes the text.

**Two ways to finish.** On `aab`, after one step, ordinary Cells would take the second step and then expand `aab`. Stop and walk instead stops there and walks the two points.

Two ways to finish aab

Worked by hand. The places follow the rule of section 1: each point's word ends at its place, and the next point's place is this one's less the word's length. Here the walk starts at point 0 for the illustration; in the kernel it starts at the marked point.

**The walk in the kernel.** The built kernel, in [\#493](<https://github.com/lmmx/mibz2/pull/493>), walks the loop once. It follows the map forwards from the marked point and moves down the output. It writes each point's word so that the word ends at the point's place. After as many points as the loop has, it is back at the marked point, one block lower, and it has written the whole block. If the loop closes too early, or the bytes don't add up to the block, the kernel decodes the block again the shipped way.

**Expand once, then copy.** A letter's interval may hold many points, and each has the same word. Expanding a long word from its rules at every point would repeat the same work. An earlier version did just that, and its writing pass cost about twice the shipped writer's instructions. The walk expands a long word the first time it meets its letter, and notes where it wrote it. Every later point of that letter copies the bytes from there, so the output itself serves as a store of words already expanded. The walk copies a short word, kept as bytes, straight from where the induction stored it. The copy is a fixed 32 bytes, then 16 at a time for a longer word, so the walk doesn't branch on a word's length.

Writing abracadabra from a loop of four points

Conceptual: a made-up loop, not a real block's. The walk moves down the output, so it writes the first X (places 7 to 10) before, and above, the copy at 0 to 3. In a real block the loop has about one point for every 22 rows.

**Why the wide stores are safe.** A 32-byte copy of a 4-byte word writes 28 bytes too many. They land below the word, at places the walk hasn't reached yet, because places only fall as it goes. The walk writes each of those bytes again, correctly, when it gets there. A copy's source is always the first occurrence, which lies wholly above, so the walk finished it before reading it. Two places break that rule: place 0, where the walk wraps round the block, and the walk's last 32 bytes, just above where it began. There the walk writes each word exactly.

The only new table the walk needs is one 4-byte first place for each letter alive at the stop, about a tenth of the letters. As we first built it, we sized its tables to every letter, and they added about 20% to the peak memory of a [worker](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-01-block-dispatch/index.md>), the thread that decodes a block. Before any timing, we cut them to the live letters. Locally the peak now matches the shipped kernel's: 37 against 37 MB on one thread, 83 against 85 on four ([the revised rule](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>)).

**Why one lane.** An earlier design split the walk into eight lanes, so that their waits on memory overlapped. That needed an extra pass to measure each lane's stretch of the output first. And one lane's wide stores could overwrite bytes that a neighbouring lane had already written. A single ordered walk needs neither the extra pass nor any guard at the boundaries. It pays with a single chain of dependent loads, which [section 4](<#4-where-should-it-stop-and-what-does-it-cost>) comes back to.

## 4\. Where should it stop, and what does it cost?

The stop needs a test, and the induction's inner loop has no room for one. That loop already uses more values than the processor has registers. An earlier version tested the map's remaining size on every cell. That test alone cost about 10 instructions a cell, whether the block stopped or not, and cancelled what the stop saved. So this design puts the stop on a count the kernel already keeps. The kernel refuses any block that would take more cells than a budget allows, so it counts every cell down already. Stopping after ⌈β r⌉ cells, where r is the block's number of letters at the start, needs only a different starting count. The design priced that at nothing. In the build, the cleanest form of it spilled the counter to memory and cost 16 to 38 instructions on every cell. So the build compares the counter with a stored bound instead. That costs about 2 instructions a cell.

We set the stop as β r cells, not as a size of the map. The kernel knows r before the induction starts, but not the total number of cells. On Wikidata a block takes 1.29 to 2.08 cells a run in all. Early in the induction most of the map disappears cheaply. Late in it few rows are left, but each step still makes a long word. Stopping earlier saves more words and leaves more points to walk, so β = 1.5 is a compromise read off the counts, not an optimum. Our prototype counted three settings over the 233 large blocks:

| β (cells a run at the stop) | blocks that stop | map left, as a share of the rows (median) | points walked a letter | word bytes never made |
| --- | ---: | ---: | ---: | ---: |
| 1.4 | 232 of 233 | 0.082 | 1.23 | 43% (6.5 B a letter) |
| 1.5 | 232 of 233 | 0.046 | 0.66 | 33% (5.0 B) |
| 1.6 | 232 of 233 | 0.022 | 0.34 | 23% (3.4 B) |

A block that takes fewer cells than β r never stops, and the kernel rebuilds it as before. The table shows the striking thing. By the stop at 1.5, the map has shrunk to under a twentieth of the rows. Yet a third of the word bytes are still to come. The count measured word storage, the rules and the stored bytes. It is not the process's memory, and we have not measured any drop in memory traffic or cache misses.

**What it costs.** The prototype predicted that stopping at 1.5 would save about 20 instructions a letter against the shipped kernel. [The kernel's count](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920005870>), over the whole decode of the Wikidata extract with callgrind, an instruction counter, came out the other way. A letter here is one run of the 11,090,867 in the blocks Cells takes, and the whole decode costs about 550 instructions a letter. Before any timing, the team set a limit of +20 instructions a letter, under 4% of the decode.

| β | whole decode, first passing build | a letter, against the shipped lazy kernel | a letter, on the build the i9 run uses |
| --- | ---: | ---: | ---: |
| shipped | 6.076 G instructions |  |  |
| 1.4 | 6.360 G | \+25.6 | \+27.3 |
| **1.5** | **6.196 G** | **\+10.8** | **\+12.3** |
| 1.6 | 6.154 G | \+7.0 | \+8.1 |

β = 1.5 stays under the limit. The counter itself varies by about 10 a letter between builds of the same source. So the count shows that β = 1.5 stays under the limit, not that it costs exactly 11 or 12 instructions. The two builds' figures agree within that noise.

The trade: word bytes never made, against instructions added

Counted and modelled, not timed. The heights, the share of word bytes never made, are a prototype's counts over the 233 large Wikidata blocks. The filled points are the built kernel's whole-decode instruction counts on the build the i9 run uses (the first passing build counted +25.6, +10.8 and +7.0); the hollow ones are what the design's model predicted, including the cost of reading the stopped map.

The count also shows where the prediction went wrong. At β = 1.5, on the first passing build, three costs differ. The cells never made were the cheap ones. The tail of the induction reads no new runs, so each of its cells costs about 38 instructions, not 71. Skipping them saves 13.2 a letter, not 21.5. The walk costs 52.1 instructions a letter, against the shipped writer's 49.5. The prototype's standalone walk cost 38.3. Each point takes about 31 instructions of fixed work: a rank lookup, a bit count and a read of the letter's record. Expanding each letter's first occurrence adds 14.6 a letter. And reading the stopped map into the walk's tables costs 18.9, not the 10 the model allowed. About 6,000 letters are alive in a block at the stop, and the kernel reads each twice.

**Time on one thread.** On a cloud machine, with the kernel's phases timed, β = 1.5 was about 10% slower than the shipped kernel. The walk runs about the writer's instructions in about twice the writer's time. Its step from one point to the next is a load, a bit count, a second load and an add. That takes about 20 cycles, and each step needs the one before it. That machine's timings prove nothing about speed; the i9 later measured 6.4% ([section 6](<#6-what-did-the-timing-show>)). They did say what the decisive run had to show: that when many threads share the cache, the bytes saved buy back more than the walk costs. A second lane, walking the other way round the loop, would halve the chain, but needs more tables, so it waits on the result.

**Why the design is this way.** An earlier version of stop and walk made the two mistakes this design avoids. It tested the map's size on every cell, which cost about as much as the stop saved. And it expanded each long word from its rules every time it appeared. Together they made it about 97 instructions a letter dearer than the shipped kernel, far over the limit. The figure sets its costs against this design's, item by item. The team now asks every failed build for that item-by-item list and sends it back to the design's author ([§11a of its protocol](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/team/PROTOCOL.md#11a-a-failed-gate-goes-back-to-its-author-term-by-term>)).

Why the design is this way: an earlier version against this one, in instructions a letter

Counted with callgrind over the whole decode of the Wikidata extract, in instructions per run of the blocks Cells takes, against the shipped lazy kernel; this design's figures are from its first passing build. The two builds stop at different points, a 32nd of the rows and 1.5 cells a run, so the rows compare designs, not one stop. “The rest” is each total less its listed terms, worked out here, not counted. The earlier version's two largest avoidable costs, the stop test and writing the text, are the two this design removes.

**Why stop only once.** [A follow-up record](<https://github.com/lmmx/mibz2/blob/806979d4402945685da22a334bde4a63db83b470/docs/journal/monodromy/2026-11-16-tower.md>), in review as [\#492](<https://github.com/lmmx/mibz2/pull/492>), tried stopping again, on the walk's own tail. It gives the right bytes, but describing each extra level of the map costs about as much as the words it saves. The best two-stop version saves 1.3 bytes a letter more, for about 16 to 26 more instructions a letter. The single stop at 1.5 stays the design.

**What is known, and how.** The records label each statement as proved, counted or modelled:

| Status | What the records support |
| --- | --- |
| Proved | the stopped map keeps every path and its words; one loop with one known place gives every place; an ordered walk with first-place copies never overwrites a byte still needed |
| Counted or tested | a third of the word bytes never made at β = 1.5 (prototype, 233 blocks); +10.8 and +12.3 instructions a letter (kernel, callgrind); the same bytes as the shipped kernel on all 431 Wikidata blocks, the test files and 4,000 random columns |
| Measured on the i9 | slower at 8, 9 and 20 threads, level at 10 and 12; 5% less extra processor time from 1 thread to 10; 6.4% slower on one thread; faster with two workers forced onto each core |
| Modelled | whether the cache's size, its misses or the bandwidth to memory limit it; what a second lane would save |
| Not done yet | the Miri and fuzz runs, the mutation pass and the review of the new unsafe code, which the records list as owed before any adoption |

## 5\. Why test it now?

The kernel's bytes matter only where threads compete for the cache. On one thread, instructions decide the time. That is why lazy letters lost at 4 and 6 threads and won from 8. So the question is which thread counts run the lazy kernel, and until recently the answer was few of them. [Why mibz2 stops scaling](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-07-why-mibz2-stops-scaling/index.md>) found the *knee*: past about 8 threads, each block decoded at once slowed the others. mibz2 answered with [teams](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-07-why-mibz2-stops-scaling/#1-how-does-mibz2-decode-with-several-threads>) of threads that share one block: three teams of three at 10 threads. Those teams were [later pipelined](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-07-pipelined-teams/index.md>), and later still given a [pit crew of two service threads](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-09-pit-crew-service-threads/index.md>) for each block's serial steps. Those teams ran their own kernel, so Cells' memory cuts reached only 8 threads and below.

That changed in 0.5.1. [\#462](<https://github.com/lmmx/mibz2/pull/462>) gave each of 10 threads a block of its own, decoded with lazy letters. [\#468](<https://github.com/lmmx/mibz2/pull/468>) covered machines with 8 to 10 physical cores asked for more threads than that. They run one lazy worker a core, and leave the second hardware threads idle. The thread plan in the code, [`plan_with`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/src/parallel.rs#L2680-L2706>), now runs on the 10-core i9 as follows:

| Threads asked for | What runs |
| --- | --- |
| 1 to 7 | a worker a thread, eager Cells |
| 8 to 10 | a worker a thread, lazy letters |
| 11 to 20 | 10 workers, one a core, lazy letters |

Teams now run only on other machines. A machine with 6 or 7 physical cores gets teams of two when asked for more threads than it has cores. So does a machine with 11 cores or more, asked for 11 threads or more. Where the machine's topology can't be read, which includes every system other than Linux, mibz2 runs a worker a thread at every count.

[The release ledger](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/release-attribution.md>) shows why that matters. It lists, for each release, which merge moved which thread count on the i9 by 3% or more. From 0.2.4 on, 10 to 20 threads moved in only three ways. Two changed how mibz2 grouped its threads: the teams, then pipelining them. One was an existing kernel ported to the teams. The third was 0.5.1's move onto Cells, which sped up 10 threads by 7.3%. Each of Cells' memory cuts, from the 20-byte slot to lazy letters, sped up only 8 threads or fewer, because teams ran above that. Now a cut to the lazy kernel's working set reaches every count from 8 to 20. Stop and walk trades instructions for memory, the trade lazy letters won and [a ring of shared buffers](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-ring.md>) lost. The ring cut the lazy kernel's bytes by 15% for 13% more decoder instructions. It ran slower at 8 threads and level at 9 and 10. By the prototype's count, stop and walk would cut 5.0 of those 29.5 bytes a letter, about a sixth, for about 2% more instructions.

## 6\. What did the timing show?

We timed it on the i9, [\#493](<https://github.com/lmmx/mibz2/pull/493>), under [a rule we fixed before the data](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>). The test decoder kept the shipped decoder's workers, thread plan and [window](<https://lmmx.github.io/mibz2/pr-preview/pr-765/blog/2026-11-01-block-dispatch/index.md>) of blocks in flight, and swapped only the kernel. It decoded four copies of the Wikidata extract at 8, 9, 10, 12 and 20 threads. Each of its ten paired rounds ran the shipped kernel and the three settings of β. At each count, β = 1.5 counts as faster if the 95% interval of its wall-time ratio lies wholly below 1. It counts as slower if the interval lies wholly above.

Stop and walk would replace the lazy kernel, everywhere lazy letters run now, if three things held. β = 1.5 had to be faster at one of those counts or more. It had to be slower at none. And its peak memory had to stay within 10% of the shipped kernel's at every count. Faster at some counts and slower at others is a finding, not an adoption. Stop and walk gets no thread range of its own, and moving the lazy threshold would be a separate decision. Because its verdict could change a default, the rule had one review before the run. The run also recorded two figures without judging them. One is how much slower each setting runs on one thread. The other is how much more processor time it takes at 10 threads than at 1, where a smaller working set should show.

**The verdict: not adopted.** [Our reading of the run](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6922810916>) applied the rule's checker, frozen before the data, to the run's results. β = 1.5 ran slower at 8, 9 and 20 threads. At 10 and 12 we could not tell it from the shipped kernel. Its peak memory stayed within 3% of the shipped kernel's at every count, and the run's guard on a text corpus was inconclusive at every count. Wall time, stop and walk at β = 1.5 over the shipped lazy kernel, on production's plan of one worker a core:

| Threads | Wall time ratio | 95% interval | Reading |
| ---: | ---: | ---: | --- |
| 8 | 1.012 | 1.004 to 1.019 | slower |
| 9 | 1.010 | 1.004 to 1.016 | slower |
| 10 | 0.991 | 0.980 to 1.003 | no difference detected |
| 12 | 0.997 | 0.985 to 1.010 | no difference detected |
| 20 | 1.015 | 1.003 to 1.027 | slower |

**What we learned.** The memory relief is real. Going from 1 thread to 10, the shipped kernel's processor time grows, because the threads compete for the cache. Stop and walk's grows 5% less: its ratio of the two is 0.947 (0.913 to 0.982) of the shipped kernel's. But with one worker a core, that relief did not repay the walk. On one thread stop and walk ran 6.4% slower (1.064, 1.032 to 1.096), because each step of the walk waits for the one before. When we forced two workers onto each core, 12 or 20 of them, stop and walk ran faster: 0.986 of the shipped kernel's time at 12 workers and 0.965 at 20. mibz2's plan never puts two workers on a core of the i9, so that gain is out of reach.

**What comes next.** The cost to remove is the walk's serial wait. The open question is whether the walk can split into independent chains that overlap their waits, or whether the kernel can read the stopped map more cheaply. Until one of those exists, the test variants stay on their branch.

The chain, in short. The full rebuild holds its words to the end because it learns their places only in its last steps. Walking the small map left after an early stop learns the places another way. So the last fifth of the steps never run, and a third of the word bytes never get made. Putting the stop on an existing counter, and copying each word after its first expansion, keep the cost to about 11 or 12 instructions a letter. Since 0.5.1 the lazy kernel runs from 8 threads to 20 on the i9. So a memory cut to it reaches the whole range where threads contend for the cache. The timing said no: with one worker a core, the walk's waits cost more than the memory it saves. With two workers a core the relief showed, so a walk that overlaps its waits could still change the answer. Either way, mibz2 now has a second way to finish an induction, with a proof that it is correct and a kernel that gives the right bytes.

---

**Research records:**

- The words census and its lower bound: [`2026-11-16-words.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-words.md>).
- The first proposal, with ranked starts and invariant I: [`2026-11-16-stop-and-walk.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-and-walk.md>).
- The first build and its divergences: [`2026-11-16-lazy-stop-walk.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/2026-11-16-lazy-stop-walk.md>), and [the review on the team channel](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6916500604>).
- The decision to stop it and the reopening question: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6916535874>).
- The reopened design: [`2026-11-16-stop-walk-reopened.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-stop-walk-reopened.md>).
- The verdict on the i9 run: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6922810916>), from the run's results bundle `stop-walk/run-20261116T162328Z` on the `evidence` branch. The second build's counts and divergences: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920005870>). The rule for the i9 run, the memory fix and the current head's counts: [the team channel, 16 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6920355683>). The build and the recipe: [\#493](<https://github.com/lmmx/mibz2/pull/493>).
- Stopping twice: [`2026-11-16-tower.md`](<https://github.com/lmmx/mibz2/blob/806979d4402945685da22a334bde4a63db83b470/docs/journal/monodromy/2026-11-16-tower.md>) ([\#492](<https://github.com/lmmx/mibz2/pull/492>)); the loops in the induction's path: [`2026-11-16-loops.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/monodromy/2026-11-16-loops.md>).
- Which merge moved which thread count: [`release-attribution.md`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/journal/release-attribution.md>); the rule for a failed gate: [PROTOCOL §11a](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/docs/team/PROTOCOL.md#11a-a-failed-gate-goes-back-to-its-author-term-by-term>).
- The thread plan: [`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/ea393f96dcfaed63581db6548e63795869c47a12/src/parallel.rs#L2680-L2706>).
