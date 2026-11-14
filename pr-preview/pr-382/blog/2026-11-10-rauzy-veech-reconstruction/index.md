[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/index.md>)

# A new way back from the BWT

An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that form the block's Lyndon tree.

10 November 2026 · 17 min read · [Research in progress](<#6-before-production>)

mibz2 is our bzip2 decoder, written in Rust. This post follows [The mathematical structure of inverse-BWT navigation](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-08-inverse-bwt-navigation/index.md>), from 8 November. That post showed that the table mibz2 walks to decode each block has a simple structure, and left open whether anything built on it could beat the table. This post reports the first experiment that does, for one stage of the decoder on one thread.

**The walk.** bzip2 stores each block as the last column of the block's sorted rotations: the Burrows–Wheeler transform (BWT). [An earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-382/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion/index.md>) works through the transform and its inverse on `banana`. To undo it, mibz2 builds a table with one entry per row and walks it. The step from one row to the next is a map called **LF**, for last-to-first. Each entry gives one byte and the address of the next row, so the walk makes one dependent memory lookup per byte. Walking the table takes about 53–60% of a decode ([the concepts page on memory-level parallelism](<https://lmmx.github.io/mibz2/pr-preview/pr-382/concepts/couriers/index.md>)), and building it adds more.

The walk is a trail of sealed addresses

The envelope is one row lookup; its contents are the byte and the next row address. The wait lies *before* that next address is known. The analogy breaks at the postman: cache hits can be quick and other work can overlap a miss; neither every load nor every decoder thread must stall.

**One shift per run.** A **run** is a maximal stretch of equal bytes in the last column. Within a run, LF sends consecutive rows to consecutive rows, so it moves each run's rows as one block. The whole map is therefore one interval per run, each shifted to a new place, instead of one entry per row. That is the structure the 8 November post set out. On mibz2's main benchmark, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB, a run averages about 14 rows.

One instruction moves the whole tray

Each compartment is a row in one equal-byte run; the tray is that run’s interval, and sliding it is LF’s constant offset. The unchanged spacing explains why one interval describes all four mappings. The analogy breaks at physical movement: this describes a map; the kernel does not move the stored bytes in a tray, and the shift can also be negative.

**Working on intervals instead of rows.** The walk needs one lookup per byte because it only ever knows a row's next move. The trays allow a shortcut. Look at the last rows of the block, and say they belong to a tray A of 6 rows. Say also that LF sends another tray, B, of 4 rows, onto exactly the last 4 of them. Then every path through B's rows makes two moves we can predict together: B's shift, then A's. The 4 rows where B lands tell us nothing more, and nothing else lands there, so they can be deleted from the map. A shrinks to 2 rows. B now jumps straight to where A would have sent it, and carries a two-byte word for its two moves. The step read two tray lengths and two shifts, and no rows. (When the two trays are the same length, B lands on all of A, and A disappears.)

Our experimental **kernel**, a self-contained routine for this one stage of the decoder, repeats that step at the right-hand end of the map. Each step deletes a tray's worth of rows and records the new word as one rule, such as `X → AB`. When one tray is left, its word is the whole block, and expanding the rules writes out the bytes. Shrinking a map this way is a classical idea in dynamics, **Rauzy–Veech induction**. [Section 1](<#1-rebuilding-aab-in-two-steps>) states it formally, line by line, and runs it by hand on a three-byte block.

The kernel's work therefore grows with its steps, not with its bytes. Wikidata's blocks need 1.812 steps per run, and a run averages about 14 rows, so there is one step for about every 7.6 bytes (0.1309 per byte). A step costs about 7.7 ns in the faster kernel, so the steps cost about 1 ns per byte. The decoder's shipped inversion stage, building the table and walking it, takes about 2.05.

**The result.** After a bug fix ([§2](<#2-the-timings>)), the kernel took **0.82 to 0.86 times as long as the inversion stage mibz2 ships**, on the Wikidata blocks, on one thread. The first three runs gave 1.677 against 2.049 ns per byte. Nine runs 34 to 48 minutes later, on the same machine in a slower state, gave 0.84 to 0.86. Where runs are short, steps per byte rise and the kernel is slower: after the fix, it took about 4 times as long as the shipped stage on prose and 9 times on uniform random bytes. These are indicative timings from one cloud machine, not CI measurements. **The whole decoder has not been compared**, and nothing is in production.

**Two structural results.** A **Lyndon word** is one that comes before each of its other rotations in dictionary order, as `aab` comes before `aba` and `baa`. Its **Lyndon tree** splits it just before its smallest proper suffix, then splits each part the same way: `aab` splits as `a | ab`, then `ab` as `a | b`. So the tree is the record of a word cut again and again before its smallest ending, and that cut is the one that leaves both pieces Lyndon words themselves. The rules the kernel builds are exactly the Lyndon tree of the block's smallest rotation ([§4](<#4-the-lyndon-tree>)). Second, the order of steps changes how many there are: from 1.812 per run down to 1.279. No order of these steps can take fewer than 1.220. The cheaper order is not expected to be faster yet: by a count, finding each step would cost more than the fewer steps save ([§5](<#5-fewer-steps-by-a-different-order>)). The steps could also be shared among threads, but no multi-threaded run has been timed.

[Section 1](<#1-rebuilding-aab-in-two-steps>) runs the induction by hand on a three-byte block. [Section 2](<#2-the-timings>) gives the timings and the bug, and [section 3](<#3-why-the-cells-kernel-is-cheaper>) why this kernel is cheaper than our first one. [Section 4](<#4-the-lyndon-tree>) shows where the Lyndon tree comes from, and [section 5](<#5-fewer-steps-by-a-different-order>) how the order of steps changes their number. [Section 6](<#6-before-production>) lists what stands before any of this reaches the decoder. The work is in pull request [\#351](<https://github.com/lmmx/mibz2/pull/351>), with its [research journal](<https://github.com/lmmx/mibz2/blob/d022139b3ba41117aa7d0291e806a4ed39d032cc/docs/journal/monodromy/2026-11-10-rauzy-veech.md>) and [code](<https://github.com/lmmx/mibz2/blob/d022139b3ba41117aa7d0291e806a4ed39d032cc/examples/rauzy.rs>), as merged.

## 1\. Rebuilding `aab` in two steps

For a small example, sort the rotations of `aab`: `aab`, `aba`, `baa`. The last column is `baa`; the first is `aab`. Matching equal letters in occurrence order gives LF mappings 0→2, 1→0 and 2→1.

Name the one-byte run of `b` B and the two-byte run of `a` A. In the last column the runs come in the order B, A: call this the **original order**. LF sends their rows to the first column, where they come in the order A, B: the **image order**. The two orders cut the same three rows into intervals in different places. Together, the intervals and where LF sends them form an **interval exchange**: each interval is moved, whole, to a new place.

Interval exchanges are a classical object of dynamics, and shrinking one step by step is **Rauzy–Veech induction**. The panel states it formally, line by line, with what each line means in a bzip2 block beside it in colour. The hand-run after the panel is the same induction on `aab`.

Rauzy–Veech induction, stated formally and read in a bzip2 block

**1\. Interval exchange.** Cut an interval \\(I\\) into pieces \\(I\_\\alpha\\), one for each symbol \\(\\alpha\\) of an alphabet \\(\\mathcal{A}\\) of \\(d\\) symbols, with lengths \\(\\lambda\_\\alpha \> 0\\). An interval exchange map \\(f : I \\to I\\) moves each piece by a translation, so pieces keep their length and direction and are laid out again in a new order. Two bijections \\(\\pi\_0, \\pi\_1 : \\mathcal{A} \\to \\{1, \\dots, d\\}\\) give each piece's place before (\\(\\pi\_0\\)) and after (\\(\\pi\_1\\)).

In a block \\(I\\) is the block's \\(n\\) rows. A piece is a run of equal bytes in the last column, \\(\\lambda\_\\alpha\\) its length, and \\(f\\) is LF. \\(\\pi\_0\\) is the runs' order in the last column, \\(\\pi\_1\\) their images' order in the first. For `baa`: \\(\\pi\_0\\) is B, A and \\(\\pi\_1\\) is A, B.

**2\. Monodromy invariant.** The permutation

\\\[p = \\pi\_1 \\circ \\pi\_0^{-1} : \\{1, \\dots, d\\} \\to \\{1, \\dots, d\\}\\\]

sends each place in the old order to the same piece's place in the new.

The shape, without names \\(p\\) does not depend on how the pieces are named. For `baa` it swaps places 1 and 2.

**3\. One step.** Let \\(\\alpha(\\varepsilon) = \\pi\_\\varepsilon^{-1}(d)\\) be the last piece of each order. The step has type 0 if \\(\\lambda\_{\\alpha(0)} \> \\lambda\_{\\alpha(1)}\\) and type 1 if \\(\\lambda\_{\\alpha(0)} \< \\lambda\_{\\alpha(1)}\\). The longer piece is the winner, the shorter the loser. The classical theory excludes equal lengths.

The two right-hand ends For `baa`: \\(\\alpha(0)\\) is A (2 rows), \\(\\alpha(1)\\) is B (1 row), so type 0 and A wins. A finite block does reach equal lengths, and there the kernel joins the two pieces: the step `a + ab` below.

**4\. The induced map.** Remove the loser's end: \\(J = I \\setminus f(I\_{\\alpha(1)})\\) in type 0, \\(J = I \\setminus I\_{\\alpha(0)}\\) in type 1. The first-return map sends each point of \\(J\\) to where its path first comes back:

\\\[\\hat{R}(f)(x) = f^{\\,r(x)}(x),\\\]

\\\[r(x) = \\min\\{\\, n \\ge 1 : f^{\\,n}(x) \\in J \\,\\}.\\\]

In type 0, \\(r(x) = 1\\) everywhere except on the loser's piece, where \\(r(x) = 2\\).

Fewer rows, longer words A row of the loser now stands for two LF steps, and its word records both bytes: B's `ab` in the hand-run below. Each step removes the loser's length in rows from the map.

**5\. Proposition.** \\(\\hat{R}(f)\\) is again an interval exchange on the same alphabet, with a new pair \\(\\pi'\\) and the lengths

\\\[\\lambda'\_{\\text{winner}} = \\lambda\_{\\text{winner}} - \\lambda\_{\\text{loser}},\\\]

\\\[\\lambda'\_\\alpha = \\lambda\_\\alpha \\text{ otherwise.}\\\]

Why it repeats The result is the same kind of object, so the step applies again. For `baa` the lengths go from A 2, B 1 to A 1, B 1, and the equal ends then join.

**6\. Iteration.** Repeated steps give \\((\\pi^{(n)}, \\lambda^{(n)})\\) and a product \\(\\Theta^{(n)}\\) of elementary matrices, with

\\\[\\lambda^{(n)} = \\big(\\Theta^{(n)}\\big)^{-\\mathsf{T}} \\lambda, \\qquad w^{(n)} = \\Theta^{(n)} w,\\\]

where \\(w\\) is the vector of the pieces' translations. For \\(d = 2\\) the lengths follow the subtractive Euclidean algorithm.

Counts and rules An entry of \\(\\Theta^{(n)}\\) counts how often one new piece's path visits one original piece before it returns: the letters in its word. The kernel's rules keep the words themselves, not just the counts.

**7\. Inverting the BWT.** Decoding iterates LF one row at a time, \\(x \\mapsto \\mathrm{LF}(x)\\). Since LF is an interval exchange on the runs, induction can replace that iteration: each step shrinks the map and records a rule, and at the end the rules spell the block. Whether this is faster depends on what a step costs.

The rest of this post What a step costs, [§2](<#2-the-timings>) and [§3](<#3-why-the-cells-kernel-is-cheaper>). The tree the rules form, [§4](<#4-the-lyndon-tree>). How the order of steps changes their number, [§5](<#5-fewer-steps-by-a-different-order>).

Black: the standard definitions (after M. Viana, *Ergodic theory of interval exchange maps*, Rev. Mat. Complut. 19, 2006). Colour: what each line means in a bzip2 block, with the example `aab`, whose last column is `baa`. A block's map is on whole rows, so it is a discrete interval exchange, and it ends in equal lengths, which the classical theory sets aside.

Step through the same block below: first the walk, which visits every row, then the induction, which deletes the rows a path can only pass through.

Step through it: the walk visits every row; the induction deletes rows

← BackPlayNext →

1\. \*\*The walk:\*\* start at row 0 and read b; LF sends it to row 2 (read a), then to row 1 (read a), then back to row 0. It visits all three rows, one lookup each, and reads the block backwards. 2. \*\*The right-hand end:\*\* A (rows 1–2) is the last piece; B's row lands on row 2, the last row any arrow lands on. B is shorter. 3. \*\*Delete row 2:\*\* only B lands there, and the path always continues to row 1. So B's row returns straight to row 1, with the word ab. 4. \*\*Join:\*\* B now lands on all of A, at equal length. A's row goes too, and B's row returns to itself in one jump with the word aab, the whole block.

Use Next and Back, or Play. Rows are the block's three rows, with their last-column bytes. Arrows are LF. The green box is the row the path is on, and the green arrow is the one just taken. A path's *visit* is a row it passes through, and its *return* is the next row of the shrunken map it reaches. The induction deletes the rows a path can only pass through, and keeps their bytes in the word it carries.

A ticket can remember a longer journey

The number of tickets is an interval’s remaining row count; the stamp is its return word, in reconstruction order. Shortening the longer interval leaves the shorter one representing a longer journey. The lower stack is a separate, generic batch: all occurrences of the chosen pair occupy one cell and share one rule. Counted steps per byte: Wikidata 0.1309, prose 0.6761, uniform random bytes about 1; short runs leave less to batch. The analogy breaks at the stamp: the implementation records that rule once, not once per ticket; expanding it still has to emit the bytes.

Each step looks at the right-hand end of both orders. The last interval of the original order is A, two rows long; the last of the image order is B, one row long. The longer one, A, loses as many rows as B has, one. B's word grows to cover the row A lost: A's `a` followed by B's own `b`, giving `ab`. Both intervals now cover one row. When the two end intervals are the same length, the step joins them: A's `a` and B's `ab` give `aab`, and one interval is left.

After the first step, B holds a two-byte word on a one-row interval. Its one remaining row stands for two rows of the original map, and the word records both bytes. This is the idea behind Rauzy–Veech induction: keep only the rows where the path returns, and record the bytes of the path skipped in between.

Larger inputs repeat this operation and share word rules. If `X → ab` is reused in `Y → XX`, Y represents `abab` without duplicating X's stored definition. That second rule only illustrates sharing; it is not a step in the `aab` example.

The walk does one lookup per byte. The induction does one step per pair of end intervals, and writes the bytes out from its rules at the end.

## 2\. The timings

**Against the shipped decoder.** Since mibz2's 0.2.8 release, the shipped decoder builds its table from a list of the block's runs, its **run records**, instead of reading the last column ([\#324](<https://github.com/lmmx/mibz2/pull/324>)). After the bug fix below, we gave that path and the kernel the same run records. We timed both on the 233 blocks of the Wikidata file that have at least 65,536 bytes. The kernel took 1.677 ns per byte against 2.049, 0.818 times as long, and three repeats within four minutes gave 0.817, 0.820 and 0.818. Nine runs 34 to 48 minutes later read 0.836 to 0.858. Every arm was then 6 to 19% slower, and the kernel lost more than the reference. Running the old and new binaries back to back gave the same spread, so the change is in the machine, not the build; its cause on the host is not known. The result is therefore 0.82 to 0.86, not 0.818 with a spread of 0.2%. Over those twelve runs the kernel was faster on 185 to 197 of the 233 blocks. One more run, six minutes before the first three, overlapped a build that used every core: it took 89 s instead of 41 and read 0.90. It was set aside at the time as disturbed, and no figure here counts it. The 0.818 is higher than the 0.738 in the table below for two reasons of similar size. This reference is faster, 2.049 ns per byte against the column-based path's 2.172, which accounts for slightly more than half of the change. And the kernel's own time rose from 1.604 to 1.677, almost all in its first stage. That stage now reads the run records itself, taking 0.367 ns per byte against 0.29 to 0.31 when it was handed them. The bug fix changed the writer's time only from 0.344 to 0.347.

The cells kernel against the inversion stage mibz2 ships

Both sides start from the same run records, after the writer fix. Fastest of five rounds per block, pooled, on one cloud machine; not CI, and only the inversion stage, not the whole decoder. The induction is the largest stage: about 0.13 steps per byte at about 7.7 ns a step. The nine later runs found the machine in a slower state, which widened the result to 0.82 to 0.86.

**The bug.** Our review of the research record found that the kernel's writer left 126,658 bytes unwritten over 178 of the blocks. The kernel's byte check could not see this, because the output buffer already held the right bytes from another kernel. The fix, and a check that now catches the fault, are in [\#351](<https://github.com/lmmx/mibz2/pull/351>). Everything below was timed before the fix.

**The history.** Our first kernel stored the two orders as linked lists and was slower than mibz2's earlier reference: the table build that reads the last column, plus the walk. The second, the **cells kernel**, named for the cells it writes ([§3](<#3-why-the-cells-kernel-is-cheaper>)), records completed steps without moving anything through linked lists. Each **census** below is one timed pass over the 233 blocks.

Each census timed its own reference, so each row compares the kernel only with the reference from the same pass. The unit is nanoseconds per row of the BWT, which is one byte of the block, pooled over the 233 blocks.

| Census | Table build + walk | Kernel | Kernel ÷ reference |
| :--- | ---: | ---: | ---: |
| First kernel (linked lists) | 2.172 | 2.770 | 1.276 |
| Cells kernel, first census | 2.244 | 1.697 | 0.756 |
| Cells kernel, second census | 2.172 | 1.604 | 0.738 |

The cells kernel recorded lower times on 202 blocks in the first census and 204 in the second. Other work ran on other cores during the first; the second pinned the process to one core with the others idle.

The kernel has three stages: it orders the run records, runs the induction, and writes the bytes. The first kernel's 2.770 adds up each stage's fastest round separately, while the cells kernel's figures time the three together. So the 42% drop from 2.770 to 1.604 compares different passes and different timing methods; it is not a paired measurement.

The two sides of this table also start from different inputs. The reference reads the last column; the kernel receives run records. Turning the column into run records costs 0.484 ns per byte, which the table leaves out. Adding it back gives 2.088 against 2.172 for the second census, a 4% reduction where the table shows 26%. The run-records comparison above has no such gap.

On other inputs the cells kernel is slower, because short runs mean more steps per byte. After the fix, against the shipped decoder's inversion stage, it took 4.03 times as long on prose and 8.94 times on uniform random bytes. On a block of `ab` repeated, which needs one step per two bytes, it took 11.2 times as long.

## 3\. Why the cells kernel is cheaper

Three changes made the cells kernel cheaper than the first one. They did not all pay at once: an earlier version of the cells kernel was still 1.225 times as slow as the column-based reference.

A cell is the overlap of an interval in the original order with one in the image order. It holds the rows that read the same pair of run letters in succession. The kernel writes each cell once, when its step is done, and records where later work goes. This replaces most of the first kernel's shortening and relinking of list nodes.

Keep finished work still

Hanging cards are letters in the two linked orders; their strings are links that must be patched. A filed receipt is a completed cell, while the clipboard is the live remainder and promised next slot. Keeping completed work still removes the relinking work, and known future addresses permit prefetch. The analogy breaks at the filing cabinet: two slots are written for a cell, destinations can be scattered, and word/output processing still writes memory; this is not a claim that the decoder writes each byte only once.

A second change removes position tracking from the induction loop. The row where the text starts is marked in the rules, and the output stage finds the starting position from that mark afterwards. So the induction no longer updates a position at every step.

Third, the kernel prefetches the addresses of its scattered writes before it makes them. The journal times the changes in sequence on ten blocks. Replacing the linked lists with one buffer took a cell from 13.4 ns to 12.5. Marking the start row instead of following it took it to 9.8 ns, and fetching ahead to 7.2 to 7.7 ns.

The cells kernel uses more memory than the table, but it misses the cache less. These counts come from callgrind on single blocks, before the fix, against the column-based table build and walk; the misses are of a simulated 2 MiB cache, this machine's L2. The most favourable block has about 17 bytes per run. There the kernel executes 186 instructions per run against 268, with 2.1 misses against 9.1. At the file's average of about 14 bytes per run it is 180 against 217, with 2.3 misses against 7.9. On a block with about 8 bytes per run it executes more than the table, 164 against 129. It still misses less there, 2.6 against 4.0. The table's work is per byte, so its cost per run falls as runs get shorter.

## 4\. The Lyndon tree

Each rule joins two words into one, so the rules form a tree: its leaves are bytes, and each internal node joins its children's words. Shared rules are shared subtrees, and the whole set of rules is a **grammar** for the block.

A **primitive** word is not an exact repetition of a shorter word. The smallest of its rotations, in dictionary order, is a Lyndon word. This is the definition the opening used; for a Lyndon word it is equivalent to coming before each of its proper suffixes. For example, `aab` is smaller than `aba` and `baa`, and its Lyndon tree splits it as `a | ab`, then `a | b`.

The journal identifies the right-end induction's expanded derivation tree with the Lyndon tree of that smallest rotation. It first relates the construction to a **Cartesian tree**: a hierarchy formed by repeatedly splitting a sequence at its minimum. Here the sequence contains ranks of rotations beginning at the word's internal boundaries. For `aab`, the rotations at positions zero, one and two have ranks zero, one and two. The internal sequence is therefore \[1,2\]. Its minimum splits after the first `a`, giving `a | ab`; the remaining split gives `a | b`. This agrees with the two rules above. In general, the standard split of a Lyndon word cuts it just before its smallest proper suffix. That a Lyndon tree has the shape of the Cartesian tree of the word's suffix ranks is a 2003 result of Hohlweg and Reutenauer; Crochemore and Russo set it out in §4 of [*Cartesian trees and Lyndon trees*](<https://arxiv.org/abs/1712.08749>).

The rules are a book’s nested bindings

Each bound part is a word; its binding seam is the standard Lyndon split, before its smallest proper suffix. Reading the joins from the smaller book upward gives the rules ab = a·b and aab = a·ab. The analogy breaks at the bookbinder: the kernel does not first sort suffixes or build a separate book tree, and shared equal rules form a grammar rather than a physically duplicated tree. The stated identity concerns right-end induction’s expanded tree for a primitive block’s smallest rotation.

A longer word shows the rule at work. Dictionary order puts `a` before `b`, and puts a word before any longer word it begins, so `ab` comes before `aba`. A Lyndon word is cut just before its smallest proper suffix, and each part is cut the same way, until single letters are left. The cut goes there because both pieces stay Lyndon words. The right piece, the smallest proper suffix, is one because each of its own suffixes is also a suffix of the whole word, so comes after it; the left piece is one too (Crochemore and Russo, §3). Each cut is a fork in the tree: the part above it, its two pieces below. Here is that cut, by hand, on `aab` and on the eight-letter word `aababbab`.

One rule makes the tree

The whole of the Lyndon tree is this one step, repeated on each part until single letters are left. Positions count the letters of `abbabaababbabaab` from 0, as in the next two figures.

The cuts happen in an order: a part is cut, then its left half, then its right half, and the cursor returns up when a part is finished. That motion is a recursion, the order in which a program visits the parts. It is not the induction's visiting and returning of §1, which follows rows through a shrinking map. The two journeys are different, and they end in the same tree. Step through them for `aab`, then for the green word, then for the whole word.

Step through the cuts, one part at a time

aabaababbab\#abbabaababbabaab

← BackPlayNext →

The cuts of \`aab\`, in order: look at \`aab\`; its smallest proper suffix is \`ab\`, so cut \`a | ab\`. The left part is one letter. Look at \`ab\`; cut \`a | b\`. Every part is now one letter: two cuts, two joins. The figures below show the same for the 17-letter word.

Choose a word, then use Next and Back, or Play. Green box: the part being looked at. Circled rank: the lowest inside it, which is its smallest proper suffix. Amber line: the cut. When both halves of a part are finished, the cursor returns up to the part above it.

Repeating the cut on every part of a longer word gives its whole tree. Crochemore and Russo draw it for `#abbabaababbabaab`, where `#` is a letter placed before `a` and `b` so that the whole word is a Lyndon word. The word `aababbab` from the last figure is one of its parts, in green.

The Lyndon tree of \#abbabaababbabaab

Redrawn from Crochemore and Russo, [*Cartesian trees and Lyndon trees*](<https://arxiv.org/abs/1712.08749>), p. 7, and recomputed here. Each bar is a node; read down, the bars are ever finer cuts of the word. The \# is the paper's device for making any word a Lyndon word; it plays no part in a bzip2 block.

Sorting the suffixes of every part to find its cut is slow. Ranking the suffixes of the whole word once, in dictionary order, gives every cut at a glance: inside each part, skip its first letter, and the cut is at the lowest rank. The ranks are computed once, from the whole word, and never change as it is cut. The ranks belong to the whole word, not to the part, so it is not obvious that they pick the right cut inside each part. That they always do is the theorem that ties Lyndon trees to the Cartesian trees of the ranks. The paper's Lyndon table is a separate row, which answers a different question.

Ranks find the cuts; the Lyndon table reads the widths

The ranks are of suffixes of the *whole* word, not of the green part, yet the lowest one inside each bar is always where that bar is cut: all 16 cuts here, and in general (Hohlweg and Reutenauer, 2003). That is the Cartesian-tree reading of §4. The Lyndon table answers a different question, how far a Lyndon word can run from each position, and its entries are the widest bars of the tree.

For the kernel, this means that each rule `X → AB` it records is one join of such a tree: the tree of the block's smallest rotation. The kernel never sorts suffixes or computes these ranks. They are here to show what its rules are.

Lyndon trees, and their link to Cartesian trees, are long established. \#351 gives a way to reach one from the BWT's runs without rebuilding the text first.

The journal gives a proof and checks it with independent simulators, node by node, on every block. This post has checked that record, not the proof itself. The proof needs the word to be primitive: `aab` is, while an exact repetition such as `abab` needs separate treatment.

## 5\. Fewer steps by a different order

The induction can take its step at either end of the two orders, or in between. The cells kernel always takes it at the right-hand end; taking it at the left-hand end needs more steps, and other orders need fewer. Whether they save time is not yet known.

Counts here are steps divided by `r`, the number of runs, so blocks with different run counts can be compared. To see what a step removes, follow LF from row to row and write down, for each row, the letter of the run it lies in. The block becomes a cyclic word of run letters, one per row. A step then replaces every occurrence of one adjacent pair of letters, a **digram**, with a new letter, which is how the compressor RePair works too. A digram's occurrences fill one cell, so a step handles them all at once. Each cell's edges are **boundaries**. A “free” step here means that its bookkeeping creates no new boundary; it still costs processor work.

The favourable order takes available free steps first, then chooses the largest cell. Its count is 1.279 steps per run, compared with 1.812 for the right-end sweep.

The planner is on the clock too

A stop is a cell step; the route is their order, and the magnifying glasses are the search for the next eligible step. The second order takes steps that create no new boundary first, otherwise the largest cell. These “free” steps still cost CPU work. The analogy breaks at distance: routes and drawn stops are schematic, not a scale of counts, memory distance or time; 1.220 is a lower bound for this cell-step model, not every inversion algorithm. A different worklist variant was priced higher from counts and estimated costs; it was not a timed comparison of these two orders.

The journal proves that `steps = initial cells − 1 + newly introduced boundaries`, where the initial cells number r₂, one per distinct digram. In the toy example, the union of both orders' cuts gives three initial cells. Two steps leave one, with no new boundary introduced. A step can also move an interval into the middle of another, which adds a cut that a later step must remove.

So every order needs at least r₂ − 1 steps: (r₂ − 1)/r = 1.220 per run on Wikidata. The favourable order is about 29.4% below the right-end count and 4.8% above that floor.

The floor applies only to this kind of step. The journal describes another operation outside it, fusion, which joins two letters that sit next to each other in both orders. On eight blocks it brings the best orders found to 1.223 steps per run pooled, and one of the eight ends 0.16% below its floor. So the floor is not a lower bound on inverse BWT itself.

The fast cells kernel relies on sweeping the ends of its orders. In the version this post links to, an out-of-order step as built costs 152 instructions, against 64 for a step at the end. So an order with fewer steps is slower unless its steps can be found and taken cheaply. We have since counted, on ten blocks, a worklist: a queue of the steps that are ready, taking free steps wherever they occur. Its order takes 1.458 steps per run, not 1.279. To find each step it tests 2.25 ends of segments, the stretches of the orders it tracks, and two steps in three are in a different segment from the one before. Priced at the cells kernel's 64 instructions a cell, with the costs of a test and a move estimated, that comes to about 150 instructions per run against the right-end sweep's 115. This is a count and an estimate, not a timing.

The same applies to parallel work. The steps can be grouped into rounds of steps that do not depend on each other, but within a round each merge's steps form a chain, a median 3,861 steps deep per block. So sixteen workers would need between 10.6 and 16 times fewer turns than one, before counting what it costs to hand work between them. Run sequentially, the round-based implementation's order and induction alone, without writing the bytes, took about 1.99 times as long as the column-based table build and walk, and no multi-worker run has been timed.

## 6\. Before production

The next comparison is the whole decoder, with the kernel in place of the shipped inversion stage. The kernel uses Rust's `unsafe` code and still needs an independent safety review, which the journal's tests do not replace. Inputs with short runs need a fallback to the walk, or a different reduction.

The review of the research record has five parts: memory safety, citations, numbers, proofs, and a referee's reading for contradictions. The memory-safety part found the writer bug in §2. All five parts have reported and their corrections are in \#351; a last check of those corrections is still running.

Knowing that the grammar is the Lyndon tree suggests building the tree directly. Computing every rotation's rank first, though, could cost as much as the reconstruction this approach avoids.

The open question is whether the Lyndon tree can be built more cheaply than by running the induction step by step. The order with fewer steps would help only if the kernel could find each step cheaply, which it cannot yet.
