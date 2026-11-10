---
title: "A new way back from the BWT"
image: assets/social/blog-2026-11-10-rauzy-veech-reconstruction.png
description: "An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that form the block's Lyndon tree."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# A new way back from the BWT

An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that form the block's Lyndon tree.
{: .post-dek }

10 November 2026 · 14 min read · [Research in progress](#6-before-production)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 is our bzip2 decoder, written in Rust. This post follows [The mathematical structure of inverse-BWT navigation](2026-11-08-inverse-bwt-navigation.md), from 8 November. That post showed that the table mibz2 walks to decode each block has a simple structure, and left open whether anything built on it could beat the table. This post reports the first experiment that does, for one stage of the decoder on one thread.

**The walk.** bzip2 stores each block as the last column of the block's sorted rotations: the Burrows–Wheeler transform (BWT). [An earlier post](2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md) works through the transform and its inverse on `banana`. To undo it, mibz2 builds a table with one entry per row and walks it. The step from one row to the next is a map called **LF**, for last-to-first. Each entry gives one byte and the address of the next row, so the walk makes one dependent memory lookup per byte. Walking the table takes about 53–60% of a decode ([the concepts page on memory-level parallelism](../concepts/couriers.md)), and building it adds more.

**One shift per run.** A **run** is a maximal stretch of equal bytes in the last column. Within a run, LF sends consecutive rows to consecutive rows, so it moves each run's rows as one block. The whole map is therefore one interval per run, each shifted to a new place, instead of one entry per row. That is the structure the 8 November post set out. On mibz2's main benchmark, an 11.9 MB extract of a Wikidata dump that decompresses to 210 MB, a run averages about 14 rows.

**Working on intervals instead of rows.** We wrote an experimental **kernel**, a self-contained routine for this one stage, that never walks the rows. It works on the intervals. Each step compares the last interval of the runs' order in the last column with the last interval of their order after LF. It takes the shorter one's length off the longer one, and gives the shorter interval a longer word to cover the rows removed. It records that word as one rule, such as `X → AB`. When one interval is left, its word is the whole block, and the rules spell it out. One step covers many rows, so the work grows with the number of steps, not the number of bytes. Wikidata needs 1.812 steps per run, about 0.13 per byte. The step is a known technique, Rauzy–Veech induction, applied here to inverting a BWT.

**The result.** After a bug fix ([§2](#2-the-timings)), the kernel took **0.818 times as long as the inversion stage mibz2 ships**. That is on the Wikidata blocks, on one thread: 1.677 against 2.049 ns per byte. Where runs are short, steps per byte rise and the kernel is slower. Before the fix, against the older column-based reference, it took about 4 times as long on prose and 10 times on uniform random bytes. These are indicative timings from one cloud machine, not CI measurements. **The whole decoder has not been compared**, and nothing is in production.

**Two structural results.** A **Lyndon word** is one that comes before each of its other rotations in dictionary order, as `aab` comes before `aba` and `baa`. Its **Lyndon tree** splits it just before its smallest proper suffix, then splits each part the same way: `aab` splits as `a | ab`, then `ab` as `a | b`. The rules the kernel builds are exactly the Lyndon tree of the block's smallest rotation ([§4](#4-the-lyndon-tree)). Second, the order of steps changes how many there are: from 1.812 per run down to 1.279. No order of these steps can take fewer than 1.220. The cheaper order is not yet faster, because finding each step costs more ([§5](#5-fewer-steps-by-a-different-order)). The steps could also be shared among threads, but no multi-threaded run has been timed.

[Section 1](#1-rebuilding-aab-in-two-steps) runs the induction by hand on a three-byte block. [Section 2](#2-the-timings) gives the timings and the bug, and [section 3](#3-why-the-cells-kernel-is-cheaper) why this kernel is cheaper than our first one. [Section 4](#4-the-lyndon-tree) shows where the Lyndon tree comes from, and [section 5](#5-fewer-steps-by-a-different-order) how the order of steps changes their number. [Section 6](#6-before-production) lists what stands before any of this reaches the decoder. The work is in pull request [#351](https://github.com/lmmx/mibz2/pull/351), with its [research journal](https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/docs/journal/monodromy/2026-11-10-rauzy-veech.md) and [code](https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/examples/rauzy.rs); those two links show the versions from before the bug fix.

## 1. Rebuilding `aab` in two steps

For a small example, sort the rotations of `aab`: `aab`, `aba`, `baa`. The last column is `baa`; the first is `aab`. Matching equal letters in occurrence order gives LF mappings 0→2, 1→0 and 2→1.

Name the one-byte run of `b` B and the two-byte run of `a` A. In the last column the runs come in the order B, A: call this the **original order**. LF sends their rows to the first column, where they come in the order A, B: the **image order**. The two orders cut the same three rows into intervals in different places. Together, the intervals and where LF sends them form an **interval exchange**: each interval is moved, whole, to a new place.

<figure class="bf-fig">
<p class="bf-title">Two steps rebuild aab from its last column</p>
<svg viewBox="0 0 540 718" role="img" aria-label="For aab the last column is baa: run B of one row, run A of two. The original order is B then A, the image order A then B. Step one shortens A to one row and gives B the word ab. Step two joins a and ab into aab."><text x="0" y="28" class="bf-ts" style="font-size: 25px">Start: the runs of baa</text><text x="0" y="82" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="48" width="119" height="50" rx="6" class="bf-pill"/><text x="212.5" y="81" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · b</text><rect x="278" y="48" width="244" height="50" rx="6" class="bf-pill"/><text x="400.0" y="81" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><text x="0" y="140" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="106" width="244" height="50" rx="6" class="bf-pill"/><text x="275.0" y="139" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><rect x="403" y="106" width="119" height="50" rx="6" class="bf-pill"/><text x="462.5" y="139" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · b</text><text x="0" y="192" class="bf-t" style="font-size: 22px">At the right end, A (2 rows) is longer than B (1).</text><path d="M0 222H540" class="bf-rule"/><text x="0" y="272" class="bf-ts" style="font-size: 25px">Step 1: A loses a row; B takes a + b</text><text x="0" y="326" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="292" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="325" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · ab</text><rect x="278" y="292" width="119" height="50" rx="6" class="bf-pill"/><text x="337.5" y="325" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><text x="0" y="384" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="350" width="119" height="50" rx="6" class="bf-pill"/><text x="212.5" y="383" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><rect x="278" y="350" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="337.5" y="383" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · ab</text><text x="0" y="436" class="bf-t" style="font-size: 22px">Both now cover one row; equal lengths join.</text><path d="M0 466H540" class="bf-rule"/><text x="0" y="516" class="bf-ts" style="font-size: 25px">Step 2: join a + ab</text><text x="0" y="570" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="536" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="569" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · aab</text><text x="0" y="628" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="594" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="627" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · aab</text><text x="0" y="680" class="bf-t" style="font-size: 22px">One interval is left, holding the whole block.</text></svg>
<figcaption>Each box is one run's interval; its width is the rows it still covers, and its label is the word it holds. Green marks the interval whose word grew. After step 1, B covers one row but holds two bytes: a remaining visit stands for the visits removed from the exchange.</figcaption>
</figure>

Each step looks at the right-hand end of both orders. The last interval of the original order is A, two rows long; the last of the image order is B, one row long. The longer one, A, loses as many rows as B has, one. B's word grows to cover the row A lost: A's `a` followed by B's own `b`, giving `ab`. Both intervals now cover one row. When the two end intervals are the same length, the step joins them: A's `a` and B's `ab` give `aab`, and one interval is left.

After the first step, B holds a two-byte word on a one-row interval. Its one remaining row stands for two rows of the original map, and the word records both bytes. This is the idea behind Rauzy–Veech induction: keep only the rows where the path returns, and record the bytes of the path skipped in between.

Larger inputs repeat this operation and share word rules. If `X → ab` is reused in `Y → XX`, Y represents `abab` without duplicating X's stored definition. That second rule only illustrates sharing; it is not a step in the `aab` example.

The walk does one lookup per byte. The induction does one step per pair of end intervals, and writes the bytes out from its rules at the end.

## 2. The timings

**Against the shipped decoder.** Since mibz2's 0.2.8 release, the shipped decoder builds its table from a list of the block's runs, its **run records**, instead of reading the last column ([#324](https://github.com/lmmx/mibz2/pull/324)). After the bug fix below, we gave that path and the kernel the same run records. We timed both on the 233 blocks of the Wikidata file that have at least 65,536 bytes. The kernel took 1.677 ns per byte against 2.049, 0.818 times as long. Three repeats gave 0.817, 0.820 and 0.818, and the kernel was faster on 192 to 197 of the 233 blocks. The ratio is higher than the 0.738 in the table below mainly because this reference is faster: 2.049 ns per byte, against the column-based path's 2.172. The kernel's own time moved from 1.604 to 1.677, partly from the fix.

**The bug.** Our review of the research record found that the kernel's writer left 126,658 bytes unwritten over 178 of the blocks. The kernel's byte check could not see this, because the output buffer already held the right bytes from another kernel. The fix, and a check that now catches the fault, are in [#351](https://github.com/lmmx/mibz2/pull/351). Everything below was timed before the fix.

**The history.** Our first kernel stored the two orders as linked lists and was slower than mibz2's earlier reference: the table build that reads the last column, plus the walk. The second, the **cells kernel**, named for the cells it writes ([§3](#3-why-the-cells-kernel-is-cheaper)), records completed steps without moving anything through linked lists. Each **census** below is one timed pass over the 233 blocks.

Each census timed its own reference, so each row compares the kernel only with the reference from the same pass. The unit is nanoseconds per row of the BWT, which is one byte of the block, pooled over the 233 blocks.

| Census | Table build + walk | Kernel | Kernel ÷ reference |
|:--|--:|--:|--:|
| First kernel (linked lists) | 2.172 | 2.770 | 1.276 |
| Cells kernel, first census | 2.244 | 1.697 | 0.756 |
| Cells kernel, second census | 2.172 | 1.604 | 0.738 |

<figure class="bf-fig">
<p class="bf-title">The cells kernel against its own census's reference</p>
<svg viewBox="0 0 540 524" role="img" aria-label="Three censuses. Original linked orders: reference 2.172, kernel 2.770 ns per row, 1.276 times. Cells first census: reference 2.244, kernel 1.697, 0.756 times. Cells second census: reference 2.172, kernel 1.604, 0.738 times."><text x="0" y="26" class="bf-tm" style="font-size: 21px">ns per BWT row, 233 Wikidata blocks</text><text x="0" y="74" class="bf-ts" style="font-size: 25px">Original linked orders</text><text x="0" y="118" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="92" width="156.4" height="36" rx="3" class="bf-bar-est"/><text x="278.384" y="119" class="bf-t-strong" style="font-size: 23px">2.172</text><text x="0" y="164" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="138" width="199.4" height="36" rx="3" class="bf-bar"/><text x="321.44" y="165" class="bf-t-strong" style="font-size: 23px">2.770 (1.276×)</text><text x="0" y="232" class="bf-ts" style="font-size: 25px">Cells, first census</text><text x="0" y="276" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="250" width="161.6" height="36" rx="3" class="bf-bar-est"/><text x="283.568" y="277" class="bf-t-strong" style="font-size: 23px">2.244</text><text x="0" y="322" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="296" width="122.2" height="36" rx="3" class="bf-bar"/><text x="244.184" y="323" class="bf-t-strong" style="font-size: 23px">1.697 (0.756×)</text><text x="0" y="390" class="bf-ts" style="font-size: 25px">Cells, second census (idle core)</text><text x="0" y="434" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="408" width="156.4" height="36" rx="3" class="bf-bar-est"/><text x="278.384" y="435" class="bf-t-strong" style="font-size: 23px">2.172</text><text x="0" y="480" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="454" width="115.5" height="36" rx="3" class="bf-bar"/><text x="237.488" y="481" class="bf-t-strong" style="font-size: 23px">1.604 (0.738×)</text></svg>
<figcaption>Dashed: conventional table build + walk, from a column. Solid: the experimental kernel, from run records (the column-to-run scan, 0.484 ns a row, is excluded). Each kernel is compared only with the reference timed in its own census. Local single-thread fastest-of-five figures from #351's journal, with no confidence intervals; not CI, not the whole decoder, and not against 0.2.8's records-based path.</figcaption>
</figure>

The cells kernel recorded lower times on 202 blocks in the first census and 204 in the second. Other work ran on other cores during the first; the second pinned the process to one core with the others idle.

The kernel has three stages: it orders the run records, runs the induction, and writes the bytes. The first kernel's 2.770 adds up each stage's fastest round separately, while the cells kernel's figures time the three together. So the 42% drop from 2.770 to 1.604 compares different passes and different timing methods; it is not a paired measurement.

The two sides of this table also start from different inputs. The reference reads the last column; the kernel receives run records. Turning the column into run records costs 0.484 ns per byte, which the table leaves out. Adding it back gives 2.088 against 2.172 for the second census, a 4% reduction where the table shows 26%. The run-records comparison above has no such gap.

On other inputs the cells kernel is slower, because short runs mean more steps per byte. It took 3.93 times as long as its reference on prose and 9.78 times on uniform random bytes. On a block of `ab` repeated, which needs one step per two bytes, it took 8.28 times as long. These were also timed before the fix.

## 3. Why the cells kernel is cheaper

Three changes made the cells kernel cheaper than the first one. They did not all pay at once: an earlier version of the cells kernel was still 1.225 times as slow as the column-based reference.

A cell is the overlap of an interval in the original order with one in the image order. It holds the rows that read the same pair of run letters in succession. The kernel writes each cell once, when its step is done, and records where later work goes. This replaces most of the first kernel's shortening and relinking of list nodes.

A second change removes position tracking from the induction loop. The row where the text starts is marked in the rules, and the output stage finds the starting position from that mark afterwards. So the induction no longer updates a position at every step.

Third, the kernel prefetches the addresses of its scattered writes before it makes them. The journal's code and instruction counts support these three explanations, but the three changes were made and timed together, so their separate effects are not measured.

The cells kernel uses more memory than the table, but it misses the cache less. Per run of the block it has 2.1 simulated last-level cache misses against the table build and walk's 8.7, and executes 184 instructions against 260.

## 4. The Lyndon tree

Each rule joins two words into one, so the rules form a tree: its leaves are bytes, and each internal node joins its children's words. Shared rules are shared subtrees, and the whole set of rules is a **grammar** for the block.

A **primitive** word is not an exact repetition of a shorter word. The smallest of its rotations, in dictionary order, is a Lyndon word. This is the definition the opening used; for a Lyndon word it is equivalent to coming before each of its proper suffixes. For example, `aab` is smaller than `aba` and `baa`, and its Lyndon tree splits it as `a | ab`, then `a | b`.

The journal identifies the right-end induction's expanded derivation tree with the Lyndon tree of that smallest rotation. It first relates the construction to a **Cartesian tree**: a hierarchy formed by repeatedly splitting a sequence at its minimum. Here the sequence contains ranks of rotations beginning at the word's internal boundaries. For `aab`, the rotations at positions zero, one and two have ranks zero, one and two. The internal sequence is therefore [1,2]. Its minimum splits after the first `a`, giving `a | ab`; the remaining split gives `a | b`. This agrees with the two rules above. In general, the standard split of a Lyndon word cuts it just before its smallest proper suffix. That a Lyndon tree has the shape of the Cartesian tree of the word's suffix ranks is a 2003 result of Hohlweg and Reutenauer; Crochemore and Russo set it out in §4 of [*Cartesian trees and Lyndon trees*](https://arxiv.org/abs/1712.08749).

<figure class="bf-fig">
<p class="bf-title">How the row map leads to the Lyndon tree</p>
<svg viewBox="0 0 540 820" role="img" aria-label="LF, a constant shift within each run, feeds right-end induction, which keeps word rules. Expanding the rules gives a Cartesian tree of rotation ranks, identified with the Lyndon tree of the smallest rotation for a primitive word. For aab the ranks at the inner cuts are 1 and 2; the tree splits a | ab, then a | b."><rect x="0" y="0" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="34" class="bf-ts" style="font-size: 25px">LF on the block's rows</text><text x="18" y="66" class="bf-tm" style="font-size: 21px">a constant shift within each run</text><path d="M60 86V120" class="bf-arrow"/><path d="M53 111L60 120L67 111" class="bf-arrow"/><rect x="0" y="126" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="160" class="bf-ts" style="font-size: 25px">Right-end induction</text><text x="18" y="192" class="bf-tm" style="font-size: 21px">keeps word rules: a grammar</text><path d="M60 212V246" class="bf-arrow"/><path d="M53 237L60 246L67 237" class="bf-arrow"/><text x="80" y="236" class="bf-tm" style="font-size: 21px">expand the rules</text><rect x="0" y="252" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="286" class="bf-ts" style="font-size: 25px">Cartesian tree of rotation ranks</text><text x="18" y="318" class="bf-tm" style="font-size: 21px">split at the minimum, recursively</text><path d="M60 338V372" class="bf-arrow"/><path d="M53 363L60 372L67 363" class="bf-arrow"/><text x="80" y="362" class="bf-tm" style="font-size: 21px">identified with</text><rect x="0" y="378" width="540" height="82" rx="8" class="bf-card-hi" style="stroke: var(--bf-frame)"/><text x="18" y="412" class="bf-ts" style="font-size: 25px">Lyndon tree of the smallest rotation</text><text x="18" y="444" class="bf-tm" style="font-size: 21px">for a primitive word (journal §5.6)</text><path d="M0 476H540" class="bf-rule"/><text x="0" y="514" class="bf-ts" style="font-size: 23px">For aab: ranks at the inner cuts are [1, 2]</text><text x="0" y="544" class="bf-tm" style="font-size: 21px">split at the minimum: a | ab, then a | b</text><path d="M270 614L150 666" class="bf-link"/><path d="M270 614L390 666" class="bf-link"/><path d="M390 714L320 766" class="bf-link"/><path d="M390 714L460 766" class="bf-link"/><rect x="226.0" y="566" width="88" height="48" rx="6" class="bf-pill-hi"/><text x="270" y="599" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">aab</text><rect x="122.0" y="666" width="56" height="48" rx="6" class="bf-pill"/><text x="150" y="699" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">a</text><rect x="354.0" y="666" width="72" height="48" rx="6" class="bf-pill-hi"/><text x="390" y="699" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">ab</text><rect x="292.0" y="766" width="56" height="48" rx="6" class="bf-pill"/><text x="320" y="799" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">a</text><rect x="432.0" y="766" width="56" height="48" rx="6" class="bf-pill"/><text x="460" y="799" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">b</text></svg>
<figcaption>The identity concerns the expanded derivation tree; stored rules share subtrees, and the kernel need not build every node. It is proved in #351's journal and checked by simulators there, under the primitive-word condition (aab is primitive; abab is not). Green: the two internal nodes, which are the two rules of §1.</figcaption>
</figure>

Lyndon trees, and their link to Cartesian trees, are long established. #351 gives a way to reach one from the BWT's runs without rebuilding the text first.

The journal gives a proof and checks it with independent simulators, node by node, on every block. This post has checked that record, not the proof itself. The proof needs the word to be primitive: `aab` is, while an exact repetition such as `abab` needs separate treatment.

## 5. Fewer steps by a different order

The induction can take its step at either end of the two orders, or in between. The cells kernel always takes it at the right-hand end; taking it at the left-hand end needs more steps, and other orders need fewer. Whether they save time is not yet known.

Counts here are steps divided by `r`, the number of runs, so blocks with different run counts can be compared. To see what a step removes, follow LF from row to row and write down, for each row, the letter of the run it lies in. The block becomes a cyclic word of run letters, one per row. A step then replaces every occurrence of one adjacent pair of letters, a **digram**, with a new letter, which is how the compressor RePair works too. A digram's occurrences fill one cell, so a step handles them all at once. Each cell's edges are **boundaries**. A “free” step here means that its bookkeeping creates no new boundary; it still costs processor work.

The favourable order takes available free steps first, then chooses the largest cell. Its count is 1.279 steps per run, compared with 1.812 for the right-end sweep.

<figure class="bf-fig">
<p class="bf-title">The order of steps changes how many there are</p>
<svg viewBox="0 0 540 414" role="img" aria-label="Steps per run: left end 2.137, right end 1.812, free steps first then largest cell 1.279, against a floor of 1.220 for this step model."><text x="0" y="26" class="bf-tm" style="font-size: 21px">induction steps per BWT run, 233 Wikidata blocks</text><text x="0" y="72" class="bf-ts" style="font-size: 25px">Left end</text><rect x="0" y="86" width="416.7" height="38" rx="3" class="bf-bar"/><text x="428.715" y="114" class="bf-t-strong" style="font-size: 24px">2.137</text><text x="0" y="180" class="bf-ts" style="font-size: 25px">Right end (as built)</text><rect x="0" y="194" width="353.3" height="38" rx="3" class="bf-bar"/><text x="365.34000000000003" y="222" class="bf-t-strong" style="font-size: 24px">1.812</text><text x="0" y="288" class="bf-ts" style="font-size: 25px">Free first, then largest cell</text><rect x="0" y="302" width="249.4" height="38" rx="3" class="bf-bar"/><text x="261.405" y="330" class="bf-t-strong" style="font-size: 24px">1.279</text><path d="M237.9 40V364" class="bf-grid" style="stroke: var(--bf-green); stroke-width: 2.5"/><text x="0" y="396" class="bf-t-green" style="font-size: 23px">dotted: floor r₂ − 1 = 1.220 for these steps</text></svg>
<figcaption>These are step counts, not times: the cells kernel cannot yet run the cheaper order cheaply. The floor holds for any order of these cell steps (r₂ − 1, the cells of both orders' cuts, minus one); another operation outside this step model can go below it, so it is not a lower bound on inverse BWT.</figcaption>
</figure>

The journal proves that `steps = initial cells − 1 + newly introduced boundaries`, where the initial cells number r₂, one per distinct digram. In the toy example, the union of both orders' cuts gives three initial cells. Two steps leave one, with no new boundary introduced. A step can also move an interval into the middle of another, which adds a cut that a later step must remove.

So every order needs at least r₂ − 1 steps: (r₂ − 1)/r = 1.220 per run on Wikidata. The favourable order is about 29.4% below the right-end count and 4.8% above that floor.

The floor applies only to this kind of step. The journal describes another operation, outside it, that can go below 1.220, so the floor is not a lower bound on inverse BWT itself.

The fast cells kernel relies on sweeping the ends of its orders. In the version this post links to, an out-of-order step as built costs 152 instructions, against 64 for a step at the end. So an order with fewer steps is slower unless its steps can be found and taken cheaply. We have since counted a worklist, a queue of the steps that are ready, that takes them in the cheaper order. On one thread it is slower than the right-end sweep. To find each step it tests 2.25 ends of segments per step on average, the stretches of the orders it tracks, and two steps in three are in a different segment from the one before.

The same applies to parallel work. The steps can be grouped into rounds of steps that do not depend on each other, and a schedule model finds enough of them for up to sixteen workers, but it leaves out the cost of running them. Run sequentially, the round-based implementation took about 1.99 times as long as the column-based table build and walk, and no multi-worker run has been timed.

## 6. Before production

The next comparison is the whole decoder, with the kernel in place of the shipped inversion stage. The kernel uses Rust's `unsafe` code and still needs an independent safety review, which the journal's tests do not replace. Inputs with short runs need a fallback to the walk, or a different reduction.

The review of the research record has five parts: memory safety, citations, numbers, proofs, and a referee's reading for contradictions. The memory-safety part found the writer bug in §2. All but the referee's reading have reported.

Knowing that the grammar is the Lyndon tree suggests building the tree directly. Computing every rotation's rank first, though, could cost as much as the reconstruction this approach avoids.

The open question is whether the Lyndon tree can be built more cheaply than by running the induction step by step. The order with fewer steps would help only if the kernel could find each step cheaply, which it cannot yet.
