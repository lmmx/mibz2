[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/index.md>)

# Lazy letters: just-in-time memory for induction

Holding fewer letters at once leaves the cache less crowded, so eight to ten threads decode up to 11% faster despite extra instructions.

16 November 2026 · 11 min read · [Research result](<#5-what-comes-next>)

[The width of an induction](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/2026-11-15-induction-width/index.md>) counted how much of a block's rebuild must be held in memory at once, and found it was about a third of what mibz2's Cells kernel actually lays out. This post is about a kernel that keeps only that third, called *lazy letters*, and about what happened when ten threads ran it at once on a 10-core machine.

On a 10-core Intel i9, with each thread decoding a block of its own, lazy letters cut the wall time against mibz2 0.4.0 by 4.2% at 8 threads, 10.8% at 9 and 5.1% at 10. At 10 threads mibz2 0.4.0 runs teams of threads that share a block, so there lazy letters make one block per thread beat the teams. At 4 and 6 threads the extra instructions outweigh the saving, and lazy letters are 3% to 9% slower.

**The recap.** bzip2 compresses text in blocks of up to 900 kB. Each block goes through the Burrows–Wheeler transform (BWT), which reorders its bytes so that equal bytes sit together in runs. Undoing it means following a map from row to row of the sorted block. [Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), mibz2's kernel for most blocks of the benchmark's Wikidata text, does that on intervals instead of rows. It starts with one interval per run. Each step of the rebuild (the *induction*) shortens the map by letting one interval stand for a longer piece of text. The code calls each interval a *letter* and the text it stands for its *word*. When one letter is left, its word is the whole block. Cells keeps each letter's bookkeeping (its length, where its word is, its neighbours) in a record called a *slot*.

**What width is.** A letter whose word is still one byte long is an untouched piece of an original run: everything about it can be read off the list of runs. Only the letters the rebuild has already changed hold new information. The most of those alive at any one moment is the *width*. On Wikidata it is about 0.32 of the letters, under the order of steps Cells takes. Cells today lays out a slot for every letter before its first step, and keeps them all until the end.

**Why mibz2 cares.** Width is how much state the rebuild keeps in memory. When every thread decodes a block of its own, those states share the processor's cache, and the threads contend for it. Keeping less live state per block eases that.

## What was found

The takeaways, each shown in a section below:

- **Lazy letters keep a third of the slots.** An interval the rebuild hasn't reached yet is kept as a 5-byte entry, and gets a slot only when a step first reaches it. Slots that are finished with go back to a pool. The pool's peak matches the width, 0.318 of the letters, to three digits. The memory the rebuild touches besides its output falls by 38% ([section 1](<#1-what-does-a-block-keep-while-it-is-rebuilt>)).
- **It costs instructions.** The first stage and the induction together take 28% more instructions. Every letter is now visited twice, and the inner loop runs short of registers ([section 2](<#2-what-does-keeping-less-cost>)).
- **At 4 and 6 threads it loses; from 8 it wins.** On a 10-core Intel i9, lazy letters take 3% to 9% more wall time than mibz2 0.4.0 at 4 and 6 threads. From 8 to 10 threads they take 4% to 11% less ([section 3](<#3-what-happened-on-ten-cores>)).
- **At 10 threads it beats the teams.** From ten threads, mibz2 doesn't give each thread a block of its own. It runs teams of threads that share a block, because one block per thread was slower there. Lazy letters reverse that: one block per thread is 5.1% faster than the teams, at 10.6% more processor time.
- **The gain grows with the thread count, as contention for the cache predicts.** Against the current kernel, lazy letters fare better at 10 threads than at 4 in all ten paired rounds ([section 4](<#4-why-would-fewer-bytes-help-only-with-many-threads>)).
- **Since 0.5.0 it is the default from 8 threads.** This is one machine and one text. mibz2 0.5.0 uses lazy letters whenever each thread decodes a block of its own at 8 threads or more, and a confirming run on the i9 is queued ([section 5](<#5-what-comes-next>)).

How much faster mibz2 decodes with lazy letters, by thread count

Lazy letters, one block per thread, against mibz2 0.4.0 on an Intel i9-10900K with 10 cores, decoding an 11.9 MB extract of a Wikidata dump, 10 paired rounds per thread count. Below 10 threads mibz2 already gives each thread a block of its own, so only the kernel differs. From 10 threads it runs teams of threads that share a block, so at 10 threads lazy letters makes one block per thread beat the teams. 12 threads uses the cores’ second hardware threads; its interval spans zero. The crossover between 6 and 8 threads is why the production version switches to lazy letters only from 8 threads up.

## 1\. What does a block keep while it is rebuilt?

Cells tracks the letters in two orders. The first is the order of the runs in the block. The second is the same runs sorted by byte value, keeping runs with the same byte in their original order. Each step works at the right-hand end of both orders, at what the code calls the two *fronts*. It compares the two letters there, records a *cell* (one piece of a word, added to a list of rules the writer later expands into text), and moves on. The fronts work backwards from the end, so at any moment most letters are still ahead of them, untouched.

Cells in 0.4.0 gives every letter a slot before the first step. A slot is 20 bytes in the kernel's own buffer, holding the letter's length, its links and where its word is. A second slot, 8 bytes in the output buffer, holds its links in the other order. (The output buffer is free to use until the writer runs, at the very end.) Most of those slots sit unused for most of the rebuild, because most letters are still untouched pieces of runs.

Lazy letters keep an untouched letter as 5 bytes in the output buffer instead. A 4-byte entry in the second order holds its length, its byte and whether the decoder's walk starts there. One more byte, in the first order, repeats its byte. That is all a step needs until a front first reaches the letter. The step itself is unchanged: the same cells, in the same order, written as the same words at the same places.

How lazy letters keep a letter, from untouched to finished

Conceptual. A letter costs 5 bytes until a front reaches it, then holds a pair of slots while it is in play. When it is finished (merged away at a step between two letters of equal length, or closed), its pair goes back on a last-in-first-out free list for the next letter a front reaches.

Two facts, both in [the research journal](<https://github.com/lmmx/mibz2/blob/caad2bd97e236905a15a970b3a3a945877b4316e/docs/journal/2026-11-15-lazy-letters.md>), mean nothing else needs storing. First, within one byte value, the two orders list the letters in the same order. So a counter for each of the 256 byte values finds a letter's entry in the second order from its byte, with no stored map between the orders (which would cost another 4 bytes a letter). Second, a front reaches a letter first from one order only, and never reads that entry again. So the entry, once used, is overwritten with a tagged pointer to the letter's new slots, for the other front to follow when it gets there.

What the rebuild keeps, eager and lazy

Above: conceptual, ten letters at one moment. Below: counted on one thread over the 196 Wikidata blocks Cells handles (11,090,867 letters), per letter, besides the output buffer. The lazy pool's bytes follow from its peak, the median over blocks.

**The count.** Over Wikidata's 196 blocks, the pool peaks at a median of 0.318 of the letters. No block goes above 0.337. That equals the width the earlier post counted for Cells' right-end order, 0.318, to three digits: the pool holds a pair for each letter the fronts have reached that is still alive. So the memory the kernel holds now follows the width, not the letter count. Per letter:

| Bytes per letter, Wikidata | 0.4.0 | Lazy letters |
| --- | ---: | ---: |
| Slots in the kernel's buffer | 20.0 | 6.4 |
| Words (the rules and their store) | 15.6 | 15.6 |
| **Total besides the output** | **35.6** | **22.0** |
| Inside the output buffer, before the writer | 8.0 | 7.5 |

The 6.4 bytes are the 20-byte slots of the letters in play at the peak: about 20 × 0.318. Inside the output buffer, lazy letters keep the 5-byte entries. They also keep second-order slots for the letters in play, about 8 × 0.318, or 2.5 bytes a letter. The pools have room for a slot for every letter, so the memory set aside is not smaller; what falls is the memory handed out and touched. Every block decodes to the same bytes as before.

## 2\. What does keeping less cost?

The same count says what it costs. Per letter, on one thread:

| Instructions per letter | 0.4.0 | Lazy letters |
| --- | ---: | ---: |
| First stage (laying out the letters) | 81.0 | 71.2 |
| Induction | 115.5 | 181.0 |
| **Both together** | **196.4** | **252.2 (+28%)** |

The first stage gets cheaper, since it writes 5 bytes a letter instead of a slot. The induction gets much dearer, for two reasons. Every letter is now visited twice as each order is read, once untouched and once after it has a slot, and each visit tests which it is. The shipped kernel reads a slot each time with no test. And the lazy loop holds about 30 values at once, against the shipped loop's 19. That is more than the processor has registers for, so some spill to the stack and are reloaded.

These are instruction counts, not times. A first build cost 36% more; a leaner one, which takes slots only in the branches that need them, brought that to 28%. Before any timing, the kernel was set a limit of 10% more instructions, and at +28% it failed it. It was kept as a recorded result, with the open question of whether 38% less memory would outweigh the instructions with ten threads sharing one cache. Only a clock on a 10-core machine could answer that.

## 3\. What happened on ten cores?

The machine is the maintainer's workstation, an Intel i9-10900K. It has 10 cores, each with two hardware threads, and they all share one 20 MiB last-level cache. The run decoded the Wikidata extract at six thread counts from 4 to 12, in 10 paired rounds each. It compared four arms. The first is mibz2 0.4.0. The second gives each thread a block, with the current kernel, and the third does the same with lazy letters. The fourth is described below. All 240 readings produced the same output.

**The rule, set before the data.** [The decision rule](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6902022809>) compared lazy letters with one block per thread on the current kernel, so that only the kernel differed. Memory would count as the cause if lazy letters were faster at 10 threads, with the interval wholly below 1, and level or slower at 4. As a sanity check, the plain one-block-per-thread arm had to match mibz2 0.4.0 at 4 to 9 threads, where they run the same plan. It did: every one of those four intervals spans 1.

**The outcome.** Lazy letters were 8.4% slower at 4 threads, as their extra instructions predict. At 10 threads they were 8.1% faster, with an interval of 6.5% to 9.8%. The rule's case is met. Against mibz2 0.4.0, which is what a user ran then, the figures by thread count are:

| Threads | 4 | 6 | 8 | 9 | 10 | 12 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wall time, lazy ÷ 0.4.0 | 1.086 | 1.033 | **0.958** | **0.892** | **0.949** | 1.018 |
| 95% interval | 1.070–1.102 | 1.019–1.048 | 0.936–0.980 | 0.875–0.909 | 0.933–0.966 | 0.986–1.051 |
| Processor time, lazy ÷ 0.4.0 | 1.106 | 1.059 | 0.959 | 0.885 | 1.106 | 1.546 |
| What mibz2 0.4.0 runs | a block per thread | a block per thread | a block per thread | a block per thread | 3 teams of 3 | 3 teams of 3 |

At 8 and 9 threads lazy letters are faster and use less processor time as well. At 10 the comparison is with the teams. Teams were chosen there because, on this machine, one block per thread had been 4% slower than the teams at ten threads ([the earlier run](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900029992>)). With lazy letters, one block per thread is 5.1% faster than the teams instead, at 10.6% more processor time. At 12 threads, two threads share some cores. No change in wall time was detected there, and processor time rose by 55%.

**Memory, measured as the whole process's peak.** One block per thread holds ten blocks' state at once, and teams hold only three. At 10 threads the teams peaked at 73 MB. One block per thread peaked at 191 MB with the current kernel and 186 MB with lazy letters. Lazy letters cut the rebuild's working memory, but a plan that decodes ten blocks at once still uses far more memory than one that decodes three.

**The fourth arm: rules written past the cache.** The induction writes each new word's description, a *rule*, and never reads it back; the writer reads the rules afterwards. Writing them with non-temporal stores, which bypass the cache, saves cache space without changing the amount of memory. At 10 threads no difference was detected (1.011, interval 0.985 to 1.037). At 4 threads it was 2.6% slower. That idea stays closed.

## 4\. Why would fewer bytes help only with many threads?

On one thread, the cache holds a block's working state comfortably, and the processor's time goes on instructions. That is why lazy letters lose at 4 threads: they do more work, and the memory they save was not in short supply.

With a block per thread, each thread's state competes for the one shared cache. A rough calculation shows the scale. A Wikidata block that Cells handles has about 56,600 letters on average. At 35.6 bytes a letter, the rebuild touches about 2.0 MB besides its output. Ten threads together touch about 20 MB, the size of the i9's whole last-level cache. At 22.0 bytes a letter, it is about 1.2 MB a block and 12 MB for ten. This is arithmetic on averages, not a measurement of the cache. The output buffers and the decoder's other stages also use the cache, and not all of a block's state is in use at the same moment.

The measurement that supports contention is the pattern across thread counts. If the extra instructions were the only effect, lazy letters' ratio against the current kernel would be about the same at every count. Instead it falls as threads are added, from 1.084 at 4 threads to 0.919 at 10. The rule asked whether the 10-thread ratio was below the 4-thread ratio round by round. It was in all ten paired rounds (sign test, p = 0.002). At 12 threads, where hardware threads share cores, the ratio against the current kernel falls further, to 0.852. Cache misses were not counted in this run. So this is evidence from timing that memory per core costs time at high thread counts, not a direct observation of the cache. It doesn't say which part of the memory system is the limit: the cache's capacity, the misses themselves, or the bandwidth to main memory. Hardware counters, or runs designed to separate them, would.

## 5\. What comes next?

**Accepting the processor time.** At 10 threads, lazy letters use 10.6% more processor time than the teams. mibz2's release rules send any rise over 10% to the maintainer, and he accepted it. The extra processor time mattered only as a sign of contention for the cache, and the wall time fell.

**In 0.5.0.** mibz2 0.5.0 uses lazy letters for one block per thread [from 8 threads up, and the current kernel below](<https://github.com/lmmx/mibz2/pull/452>), chosen by one constant in the decoder's thread plan. The old kernel stays selectable, so the two can be compared at the same commit. A run at the release, on the i9 from 4 to 20 threads, confirms the switch point under [a rule fixed before it](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6904285481>). It also decides whether one block per thread should replace the teams at more thread counts. A move into counts above the 10 cores, where threads share a core, goes back to the maintainer with the figures.

**A better order of steps.** Lazy letters still take Cells' order, always at the right-hand end, and the pool's peak is that order's width. Other orders build the same rules while finishing letters sooner. The [order census](<https://github.com/lmmx/mibz2/pull/437>) found that taking the steps that need no new bookkeeping first, the *free* steps, brings the median width on Wikidata from 0.318 to 0.107 of the runs. Newer simulations, [in review](<https://github.com/lmmx/mibz2/pull/446>), choose each step by how much it changes the width. One that still prefers free steps peaks at 0.077 of the runs, and one that puts the change in width first at 0.058. These are counts from a simulator, not a kernel. They also have a cost the counts leave out: lazy letters read the untouched letters as two streams from the right-hand end, and any other order needs letters that neither front has reached yet. [Priced against the kernel's own bytes](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6904481722>), that cost decides it for now. The slots are under a third of what lazy letters touch, and the index any other order needs adds about 8 bytes a letter, so no order cut the kernel's bytes by the quarter the rule asked. Orders that read only near the two streams, which would need no index, are still being counted. The same run will also measure how much slower a block is at 10 threads than at 1, for each kernel, which says how much of the high-thread penalty memory still accounts for.

**What changes in how memory cuts are judged.** Lazy letters failed their own instruction limit and still won where it matters for many threads. Earlier memory cuts were rejected because they cost instructions at 1 to 4 threads. From now on a memory cut is judged by its peak live memory per block and by wall time at 8 to 10 threads.

Since 0.5.0, lazy letters are the default kernel for one block per thread from 8 threads up.

The chain, in short, runs like this. The rebuild needs only a third of the slots Cells lays out. A kernel that keeps only those cuts the working memory by 38%, at 28% more instructions. With few threads that is a loss. With ten threads sharing one cache it is a gain, enough for a block per thread to beat the teams. When threads share a cache, the bytes a block keeps can matter more than the instructions it runs.

---

**Research records:**

- The kernel's design and counts: [`2026-11-15-lazy-letters.md`](<https://github.com/lmmx/mibz2/blob/caad2bd97e236905a15a970b3a3a945877b4316e/docs/journal/2026-11-15-lazy-letters.md>).
- The experiment's arms, safety arguments and recipe: [`2026-11-15-memory-lever.md`](<https://github.com/lmmx/mibz2/blob/caad2bd97e236905a15a970b3a3a945877b4316e/docs/journal/2026-11-15-memory-lever.md>) and its [counts](<https://github.com/lmmx/mibz2/blob/caad2bd97e236905a15a970b3a3a945877b4316e/docs/journal/2026-11-15-memory-lever/counts.txt>) ([\#447](<https://github.com/lmmx/mibz2/pull/447>)).
- The decision rule, set before the run: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6902022809>).
- The run's verdict: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6903318530>). Its results bundle: [`run-20261115T233002Z.tar.gz`](<https://github.com/lmmx/mibz2/blob/ccd052c04b80ac5e2c12dcc3d76ff0042cd8f267/memory-lever/run-20261115T233002Z.tar.gz>), with `summary.txt` (against mibz2 0.4.0), `summary-workers.txt` (against one block per thread) and `lever.txt` (round by round).
- The production plan and the follow-ups: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6903395401>) and [again](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6903426694>).
- Background: [the width of an induction](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/2026-11-15-induction-width/index.md>) and [the free closure of an induction](<https://lmmx.github.io/mibz2/pr-preview/pr-531/blog/2026-11-15-free-closure/index.md>).
