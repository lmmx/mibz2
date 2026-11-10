[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# A new way back from the BWT

An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that form the block's Lyndon tree.

10 November 2026 · 14 min read · [Research in progress](<#6-before-production>)

mibz2 is our bzip2 decoder, written in Rust. This post follows [The mathematical structure of inverse-BWT navigation](<https://lmmx.github.io/mibz2/blog/2026-11-08-inverse-bwt-navigation/index.md>), from 8 November. That post showed that the table mibz2 walks to decode each block has a simple structure, and left open whether anything built on it could beat the table. This post reports the first experiment that does, for one stage of the decoder on one thread.

**The walk.** bzip2 stores each block as the last column of the block's sorted rotations: the Burrows–Wheeler transform (BWT). [An earlier post](<https://lmmx.github.io/mibz2/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>) works through the transform and its inverse on `banana`. To undo it, mibz2 builds a table with one entry per row and walks it. The step from one row to the next is a map called **LF**, for last-to-first. Each entry gives one byte and the address of the next row, so the walk makes one dependent memory lookup per byte. Walking the table takes about 53–60% of a decode ([the concepts page on memory-level parallelism](<https://lmmx.github.io/mibz2/concepts/couriers/index.md>)), and building it adds more.

**One shift per run.** A **run** is a maximal stretch of equal bytes in the last column. Within a run, LF sends consecutive rows to consecutive rows, so it moves each run's rows as one block. The whole map is therefore one interval per run, each shifted to a new place, instead of one entry per row. That is the structure the 8 November post set out. On mibz2's main benchmark, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB, a run averages about 14 rows.

**Working on intervals instead of rows.** We wrote an experimental **kernel**, a self-contained routine for this one stage, that never walks the rows. It works on the intervals. Each step compares the last interval of the runs' order in the last column with the last interval of their order after LF. It takes the shorter one's length off the longer one, and gives the shorter interval a longer word to cover the rows removed. It records that word as one rule, such as `X → AB`. When one interval is left, its word is the whole block, and the rules spell it out. One step covers many rows, so the work grows with the number of steps, not the number of bytes. Wikidata needs 1.812 steps per run, about 0.13 per byte, and in the faster kernel a step costs about 7.7 ns. So the induction itself costs about 1 ns per byte, where the shipped decoder's whole inversion stage takes about 2. The step is a known technique, Rauzy–Veech induction, applied here to inverting a BWT.

**The result.** After a bug fix ([§2](<#2-the-timings>)), the kernel took **0.818 times as long as the inversion stage mibz2 ships**. That is on the Wikidata blocks, on one thread: 1.677 against 2.049 ns per byte. Where runs are short, steps per byte rise and the kernel is slower: after the fix, it took about 4 times as long as the shipped stage on prose and 9 times on uniform random bytes. These are indicative timings from one cloud machine, not CI measurements. **The whole decoder has not been compared**, and nothing is in production.

**Two structural results.** A **Lyndon word** is one that comes before each of its other rotations in dictionary order, as `aab` comes before `aba` and `baa`. Its **Lyndon tree** splits it just before its smallest proper suffix, then splits each part the same way: `aab` splits as `a | ab`, then `ab` as `a | b`. The rules the kernel builds are exactly the Lyndon tree of the block's smallest rotation ([§4](<#4-the-lyndon-tree>)). Second, the order of steps changes how many there are: from 1.812 per run down to 1.279. No order of these steps can take fewer than 1.220. The cheaper order is not expected to be faster yet: by a count, finding each step would cost more than the fewer steps save ([§5](<#5-fewer-steps-by-a-different-order>)). The steps could also be shared among threads, but no multi-threaded run has been timed.

[Section 1](<#1-rebuilding-aab-in-two-steps>) runs the induction by hand on a three-byte block. [Section 2](<#2-the-timings>) gives the timings and the bug, and [section 3](<#3-why-the-cells-kernel-is-cheaper>) why this kernel is cheaper than our first one. [Section 4](<#4-the-lyndon-tree>) shows where the Lyndon tree comes from, and [section 5](<#5-fewer-steps-by-a-different-order>) how the order of steps changes their number. [Section 6](<#6-before-production>) lists what stands before any of this reaches the decoder. The work is in pull request [\#351](<https://github.com/lmmx/mibz2/pull/351>), with its [research journal](<https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/docs/journal/monodromy/2026-11-10-rauzy-veech.md>) and [code](<https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/examples/rauzy.rs>); those two links show the versions from before the bug fix.

## 1\. Rebuilding `aab` in two steps

For a small example, sort the rotations of `aab`: `aab`, `aba`, `baa`. The last column is `baa`; the first is `aab`. Matching equal letters in occurrence order gives LF mappings 0→2, 1→0 and 2→1.

Name the one-byte run of `b` B and the two-byte run of `a` A. In the last column the runs come in the order B, A: call this the **original order**. LF sends their rows to the first column, where they come in the order A, B: the **image order**. The two orders cut the same three rows into intervals in different places. Together, the intervals and where LF sends them form an **interval exchange**: each interval is moved, whole, to a new place.

Two steps rebuild aab from its last column

Each box is one run's interval; its width is the rows it still covers, and its label is the word it holds. Green marks the interval whose word grew. After step 1, B covers one row but holds two bytes: a remaining visit stands for the visits removed from the exchange.

Each step looks at the right-hand end of both orders. The last interval of the original order is A, two rows long; the last of the image order is B, one row long. The longer one, A, loses as many rows as B has, one. B's word grows to cover the row A lost: A's `a` followed by B's own `b`, giving `ab`. Both intervals now cover one row. When the two end intervals are the same length, the step joins them: A's `a` and B's `ab` give `aab`, and one interval is left.

After the first step, B holds a two-byte word on a one-row interval. Its one remaining row stands for two rows of the original map, and the word records both bytes. This is the idea behind Rauzy–Veech induction: keep only the rows where the path returns, and record the bytes of the path skipped in between.

Larger inputs repeat this operation and share word rules. If `X → ab` is reused in `Y → XX`, Y represents `abab` without duplicating X's stored definition. That second rule only illustrates sharing; it is not a step in the `aab` example.

The walk does one lookup per byte. The induction does one step per pair of end intervals, and writes the bytes out from its rules at the end.

## 2\. The timings

**Against the shipped decoder.** Since mibz2's 0.2.8 release, the shipped decoder builds its table from a list of the block's runs, its **run records**, instead of reading the last column ([\#324](<https://github.com/lmmx/mibz2/pull/324>)). After the bug fix below, we gave that path and the kernel the same run records. We timed both on the 233 blocks of the Wikidata file that have at least 65,536 bytes. The kernel took 1.677 ns per byte against 2.049, 0.818 times as long. Three repeats gave 0.817, 0.820 and 0.818, and the kernel was faster on 192 to 197 of the 233 blocks. The ratio is higher than the 0.738 in the table below for two reasons of similar size. This reference is faster, 2.049 ns per byte against the column-based path's 2.172, which accounts for slightly more than half of the change. And the kernel's own time rose from 1.604 to 1.677, almost all in its first stage, which now reads the run records itself (0.367 ns per byte, against 0.29 to 0.31 when it was handed them); the bug fix changed the writer's time only from 0.344 to 0.347.

**The bug.** Our review of the research record found that the kernel's writer left 126,658 bytes unwritten over 178 of the blocks. The kernel's byte check could not see this, because the output buffer already held the right bytes from another kernel. The fix, and a check that now catches the fault, are in [\#351](<https://github.com/lmmx/mibz2/pull/351>). Everything below was timed before the fix.

**The history.** Our first kernel stored the two orders as linked lists and was slower than mibz2's earlier reference: the table build that reads the last column, plus the walk. The second, the **cells kernel**, named for the cells it writes ([§3](<#3-why-the-cells-kernel-is-cheaper>)), records completed steps without moving anything through linked lists. Each **census** below is one timed pass over the 233 blocks.

Each census timed its own reference, so each row compares the kernel only with the reference from the same pass. The unit is nanoseconds per row of the BWT, which is one byte of the block, pooled over the 233 blocks.

| Census | Table build + walk | Kernel | Kernel ÷ reference |
| :--- | ---: | ---: | ---: |
| First kernel (linked lists) | 2.172 | 2.770 | 1.276 |
| Cells kernel, first census | 2.244 | 1.697 | 0.756 |
| Cells kernel, second census | 2.172 | 1.604 | 0.738 |

The cells kernel against its own census's reference

Dashed: conventional table build + walk, from a column. Solid: the experimental kernel, from run records (the column-to-run scan, 0.484 ns a row, is excluded). Each kernel is compared only with the reference timed in its own census. Local single-thread fastest-of-five figures from \#351's journal, with no confidence intervals; not CI, not the whole decoder, and not against 0.2.8's records-based path.

The cells kernel recorded lower times on 202 blocks in the first census and 204 in the second. Other work ran on other cores during the first; the second pinned the process to one core with the others idle.

The kernel has three stages: it orders the run records, runs the induction, and writes the bytes. The first kernel's 2.770 adds up each stage's fastest round separately, while the cells kernel's figures time the three together. So the 42% drop from 2.770 to 1.604 compares different passes and different timing methods; it is not a paired measurement.

The two sides of this table also start from different inputs. The reference reads the last column; the kernel receives run records. Turning the column into run records costs 0.484 ns per byte, which the table leaves out. Adding it back gives 2.088 against 2.172 for the second census, a 4% reduction where the table shows 26%. The run-records comparison above has no such gap.

On other inputs the cells kernel is slower, because short runs mean more steps per byte. After the fix, against the shipped decoder's inversion stage, it took 4.03 times as long on prose and 8.94 times on uniform random bytes. On a block of `ab` repeated, which needs one step per two bytes, it took 11.2 times as long.

## 3\. Why the cells kernel is cheaper

Three changes made the cells kernel cheaper than the first one. They did not all pay at once: an earlier version of the cells kernel was still 1.225 times as slow as the column-based reference.

A cell is the overlap of an interval in the original order with one in the image order. It holds the rows that read the same pair of run letters in succession. The kernel writes each cell once, when its step is done, and records where later work goes. This replaces most of the first kernel's shortening and relinking of list nodes.

A second change removes position tracking from the induction loop. The row where the text starts is marked in the rules, and the output stage finds the starting position from that mark afterwards. So the induction no longer updates a position at every step.

Third, the kernel prefetches the addresses of its scattered writes before it makes them. The journal times the changes in sequence on ten blocks. Replacing the linked lists with one buffer took a cell from 13.4 ns to 12.5. Marking the start row instead of following it took it to 9.8 ns, and fetching ahead to 7.2 to 7.7 ns.

The cells kernel uses more memory than the table, but it misses the cache less. Measured with callgrind on one block, before the fix and against the column-based table build and walk, it has 2.1 misses per run of a simulated 2 MiB last-level cache against 8.7, and executes 184 instructions per run against 260.

## 4\. The Lyndon tree

Each rule joins two words into one, so the rules form a tree: its leaves are bytes, and each internal node joins its children's words. Shared rules are shared subtrees, and the whole set of rules is a **grammar** for the block.

A **primitive** word is not an exact repetition of a shorter word. The smallest of its rotations, in dictionary order, is a Lyndon word. This is the definition the opening used; for a Lyndon word it is equivalent to coming before each of its proper suffixes. For example, `aab` is smaller than `aba` and `baa`, and its Lyndon tree splits it as `a | ab`, then `a | b`.

The journal identifies the right-end induction's expanded derivation tree with the Lyndon tree of that smallest rotation. It first relates the construction to a **Cartesian tree**: a hierarchy formed by repeatedly splitting a sequence at its minimum. Here the sequence contains ranks of rotations beginning at the word's internal boundaries. For `aab`, the rotations at positions zero, one and two have ranks zero, one and two. The internal sequence is therefore \[1,2\]. Its minimum splits after the first `a`, giving `a | ab`; the remaining split gives `a | b`. This agrees with the two rules above. In general, the standard split of a Lyndon word cuts it just before its smallest proper suffix. That a Lyndon tree has the shape of the Cartesian tree of the word's suffix ranks is a 2003 result of Hohlweg and Reutenauer; Crochemore and Russo set it out in §4 of [*Cartesian trees and Lyndon trees*](<https://arxiv.org/abs/1712.08749>).

How the row map leads to the Lyndon tree

The identity concerns the expanded derivation tree; stored rules share subtrees, and the kernel need not build every node. It is proved in \#351's journal and checked by simulators there, under the primitive-word condition (aab is primitive; abab is not). Green: the two internal nodes, which are the two rules of §1.

Lyndon trees, and their link to Cartesian trees, are long established. \#351 gives a way to reach one from the BWT's runs without rebuilding the text first.

The journal gives a proof and checks it with independent simulators, node by node, on every block. This post has checked that record, not the proof itself. The proof needs the word to be primitive: `aab` is, while an exact repetition such as `abab` needs separate treatment.

## 5\. Fewer steps by a different order

The induction can take its step at either end of the two orders, or in between. The cells kernel always takes it at the right-hand end; taking it at the left-hand end needs more steps, and other orders need fewer. Whether they save time is not yet known.

Counts here are steps divided by `r`, the number of runs, so blocks with different run counts can be compared. To see what a step removes, follow LF from row to row and write down, for each row, the letter of the run it lies in. The block becomes a cyclic word of run letters, one per row. A step then replaces every occurrence of one adjacent pair of letters, a **digram**, with a new letter, which is how the compressor RePair works too. A digram's occurrences fill one cell, so a step handles them all at once. Each cell's edges are **boundaries**. A “free” step here means that its bookkeeping creates no new boundary; it still costs processor work.

The favourable order takes available free steps first, then chooses the largest cell. Its count is 1.279 steps per run, compared with 1.812 for the right-end sweep.

The order of steps changes how many there are

These are step counts, not times: the cells kernel cannot yet run the cheaper order cheaply. The floor holds for any order of these cell steps (r₂ − 1, the cells of both orders' cuts, minus one); fusion, an operation outside this step model, brings one block of eight 0.16% below its floor, so it is not a lower bound on inverse BWT.

The journal proves that `steps = initial cells − 1 + newly introduced boundaries`, where the initial cells number r₂, one per distinct digram. In the toy example, the union of both orders' cuts gives three initial cells. Two steps leave one, with no new boundary introduced. A step can also move an interval into the middle of another, which adds a cut that a later step must remove.

So every order needs at least r₂ − 1 steps: (r₂ − 1)/r = 1.220 per run on Wikidata. The favourable order is about 29.4% below the right-end count and 4.8% above that floor.

The floor applies only to this kind of step. The journal describes another operation outside it, fusion, which joins two letters that sit next to each other in both orders. On eight blocks it brings the best orders found to 1.223 steps per run pooled, and one of the eight ends 0.16% below its floor. So the floor is not a lower bound on inverse BWT itself.

The fast cells kernel relies on sweeping the ends of its orders. In the version this post links to, an out-of-order step as built costs 152 instructions, against 64 for a step at the end. So an order with fewer steps is slower unless its steps can be found and taken cheaply. We have since counted, on ten blocks, a worklist: a queue of the steps that are ready, taking free steps wherever they occur. Its order takes 1.458 steps per run, not 1.279. To find each step it tests 2.25 ends of segments, the stretches of the orders it tracks, and two steps in three are in a different segment from the one before. Priced at the cells kernel's 64 instructions a cell, with the costs of a test and a move estimated, that comes to about 150 instructions per run against the right-end sweep's 115. This is a count and an estimate, not a timing.

The same applies to parallel work. The steps can be grouped into rounds of steps that do not depend on each other, and a schedule model finds enough of them for up to sixteen workers, but it leaves out the cost of running them. Run sequentially, the round-based implementation's order and induction alone, without writing the bytes, took about 1.99 times as long as the column-based table build and walk, and no multi-worker run has been timed.

## 6\. Before production

The next comparison is the whole decoder, with the kernel in place of the shipped inversion stage. The kernel uses Rust's `unsafe` code and still needs an independent safety review, which the journal's tests do not replace. Inputs with short runs need a fallback to the walk, or a different reduction.

The review of the research record has five parts: memory safety, citations, numbers, proofs, and a referee's reading for contradictions. The memory-safety part found the writer bug in §2. All but the referee's reading have reported.

Knowing that the grammar is the Lyndon tree suggests building the tree directly. Computing every rotation's rank first, though, could cost as much as the reconstruction this approach avoids.

The open question is whether the Lyndon tree can be built more cheaply than by running the induction step by step. The order with fewer steps would help only if the kernel could find each step cheaply, which it cannot yet.
