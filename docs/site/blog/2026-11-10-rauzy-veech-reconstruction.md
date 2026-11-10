---
title: "A new way back from the BWT"
image: assets/social/blog-2026-11-10-rauzy-veech-reconstruction.png
description: "An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that turn out to be the block's Lyndon tree."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# A new way back from the BWT

An experimental mibz2 kernel rebuilds bzip2 text without a row-by-row walk, from reusable word rules that turn out to be the block's Lyndon tree.
{: .post-dek }

10 November 2026 · 14 min read · [Research in progress](#6-before-production)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

bzip2 stores each block as the last column of its sorted rotations: the Burrows–Wheeler transform (BWT). To decode it, mibz2 walks a next-row map called **LF**. Each row yields one byte and the address of the next row, so the walk makes one dependent memory lookup per byte.

A **run** is a maximal stretch of equal bytes in the last column. Within a run, LF sends consecutive rows to consecutive rows, so the whole map is fixed by one interval per run, each shifted as a block. The walk does not use this. A Wikidata block has about 14 rows per run.

MONODROMY's experimental kernel does not walk the rows. Read along LF, the block is a cyclic word of run letters, one per row. Each step picks one pair of adjacent letters, a **digram**, and replaces every occurrence with a new letter, recording one rule such as `X → AB`. Because each run moves as a block, a digram's occurrences fill one interval of rows, called a **cell**. So a step costs the same however many occurrences there are. When one letter is left, its rules spell the whole block. The step is Rauzy–Veech induction, and it works like RePair compression on the run letters.

The work therefore grows with the number of steps, not the number of bytes. Wikidata needs 1.812 steps per run, about 0.13 per byte. The first kernel still lost, at 1.276 times the walk's time, because a step cost 14 ns. The **cells kernel** writes each cell once and moves nothing, cutting a step to 7.7 ns. Per run it executes 184 instructions against production's 260, with 2.1 simulated last-level cache misses against 8.7.

That kernel recorded less time than its conventional reference, the table build and walk, in two local Wikidata censuses. The second, on an idle core, recorded **1.604 against 2.172 nanoseconds per row**, a derived **26.2% reduction**, and was faster on 204 of 233 blocks. Where runs are short the steps per byte rise and it loses: 3.9 times on prose, 9.8 times on uniform bytes.

On the same day, MONODROMY's verification pass found that the cells kernel's writer left some bytes unwritten: 126,658 over 178 of the 233 blocks. Its byte check could not see this, because the output buffer already held the right bytes from another kernel. The fix, and a check that now catches it, are in [#351](https://github.com/lmmx/mibz2/pull/351). The timings above were measured before the fix. MONODROMY's re-run after it compared the kernel with the inversion path mibz2 has shipped since 0.2.8, which builds from run records, giving both the same records: 1.677 against 2.049 ns per row. That is 0.818 times the shipped path's time over three runs (0.817 to 0.820), faster on 192 to 197 of the 233 blocks.

These are indicative single-thread results from the research container, using the fastest of five rounds per block. They are not CI performance evidence, and no confidence intervals accompany them. Against the column path, the new kernel starts from a run list, and the scan that would produce that list is excluded. The gain over the shipped decoder is for its inversion stage only: **the whole decoder has not been compared**, and the experiment is not in production.

The rules the induction builds form a Lyndon tree. A **Lyndon word** is strictly smaller, alphabetically, than each of its proper suffixes; `aab` is one. Its **Lyndon tree** splits it just before its smallest proper suffix, then splits each part the same way: `aab` splits as `a | ab`, then `ab` as `a | b`. Expanded, the induction's rules are exactly the Lyndon tree of the block's smallest rotation. The journal proves this for any block that is not an exact repetition of a shorter word. Crochemore and Russo's [*Cartesian trees and Lyndon trees*](https://arxiv.org/abs/1712.08749) is a short introduction to these trees.

The number of steps also depends on which digram goes next. Every order needs at least r₂ − 1 steps, where r₂ counts the distinct digrams: 1.220 per run on Wikidata. The right-end order takes 1.812; taking steps that create no new boundary first takes 1.279. No kernel yet runs that order cheaply, so it is a counted saving, not a measured speedup.

The [research journal](https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/docs/journal/monodromy/2026-11-10-rauzy-veech.md) and [implementation](https://github.com/lmmx/mibz2/blob/ae51b11a46014e5fec928b0e25235f532a473a9f/examples/rauzy.rs) linked here are the versions in [#351](https://github.com/lmmx/mibz2/pull/351) from before the writer fix. MONODROMY is the research lane developing this experiment.

## 1. Rebuilding `aab` in two steps

This section runs the mechanism on a three-byte block. LF is short for last-to-first: it connects positions in the last and first columns of the sorted rotations.

For a small example, sort the rotations of `aab`: `aab`, `aba`, `baa`. The last column is `baa`; the first is `aab`. Matching equal letters in occurrence order gives LF mappings 0→2, 1→0 and 2→1.

Name the one-byte run of `b` B and the two-byte run of `a` A. Their original order is B,A; their image order is A,B. The two orders describe the same three row positions, cut into differently placed intervals.

<figure class="bf-fig">
<p class="bf-title">Two steps rebuild aab from its last column</p>
<svg viewBox="0 0 540 718" role="img" aria-label="For aab the last column is baa: run B of one row, run A of two. The original order is B then A, the image order A then B. Step one shortens A to one row and gives B the word ab. Step two joins a and ab into aab."><text x="0" y="28" class="bf-ts" style="font-size: 25px">Start: the runs of baa</text><text x="0" y="82" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="48" width="119" height="50" rx="6" class="bf-pill"/><text x="212.5" y="81" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · b</text><rect x="278" y="48" width="244" height="50" rx="6" class="bf-pill"/><text x="400.0" y="81" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><text x="0" y="140" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="106" width="244" height="50" rx="6" class="bf-pill"/><text x="275.0" y="139" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><rect x="403" y="106" width="119" height="50" rx="6" class="bf-pill"/><text x="462.5" y="139" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · b</text><text x="0" y="192" class="bf-t" style="font-size: 22px">At the right end, A (2 rows) is longer than B (1).</text><path d="M0 222H540" class="bf-rule"/><text x="0" y="272" class="bf-ts" style="font-size: 25px">Step 1: A loses a row; B takes a + b</text><text x="0" y="326" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="292" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="325" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · ab</text><rect x="278" y="292" width="119" height="50" rx="6" class="bf-pill"/><text x="337.5" y="325" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><text x="0" y="384" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="350" width="119" height="50" rx="6" class="bf-pill"/><text x="212.5" y="383" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · a</text><rect x="278" y="350" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="337.5" y="383" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">B · ab</text><text x="0" y="436" class="bf-t" style="font-size: 22px">Both now cover one row; equal lengths join.</text><path d="M0 466H540" class="bf-rule"/><text x="0" y="516" class="bf-ts" style="font-size: 25px">Step 2: join a + ab</text><text x="0" y="570" class="bf-tm" style="font-size: 22px">original</text><rect x="153" y="536" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="569" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · aab</text><text x="0" y="628" class="bf-tm" style="font-size: 22px">image</text><rect x="153" y="594" width="119" height="50" rx="6" class="bf-pill-hi"/><text x="212.5" y="627" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">A · aab</text><text x="0" y="680" class="bf-t" style="font-size: 22px">One interval is left, holding the whole block.</text></svg>
<figcaption>Each box is one run's interval; its width is the rows it still covers, and its label is the word it holds. Green marks the interval whose word grew. After step 1, B covers one row but holds two bytes: a remaining visit stands for the visits removed from the exchange.</figcaption>
</figure>

Compare the intervals at the right end. A has length two and B length one. Remove one row from the exchange, shorten A to one, and give B the word `ab`. Both intervals now have length one. Their equal-length step joins A's `a` to B's `ab`, leaving the word `aab`.

B now holds a two-byte word on a one-row interval because its remaining visit stands for two visits in the original map, and the word records both bytes. This is the first-return idea behind Rauzy–Veech induction: keep the points where the path returns, with the bytes of the path skipped between them.

Larger inputs repeat this operation and share word rules. If `X → ab` is reused in `Y → XX`, Y represents `abab` without duplicating X's stored definition. That second rule only illustrates sharing; it is not a step in the `aab` example.

The walk asks where to go next once for every byte. Induction instead rewrites the description of the remaining problem, keeping enough to write the text out afterwards.

## 2. The timings

The first kernel stored the two orders as linked lists and was slower than the reference. The cells kernel instead records completed steps without moving letters through linked lists.

The journal reports three censuses below. Each census timed its own reference, so each row compares the kernel only with the reference from the same run. The unit is nanoseconds per BWT row, pooled over 233 Wikidata blocks.

| Census | Conventional table build + walk | Experimental kernel | Candidate/reference |
|:--|--:|--:|--:|
| Original linked orders | 2.172 | 2.770 | 1.276 |
| Cells, first run | 2.244 | 1.697 | 0.756 |
| Cells, second run | 2.172 | 1.604 | 0.738 |

<figure class="bf-fig">
<p class="bf-title">The cells kernel against its own census's reference</p>
<svg viewBox="0 0 540 524" role="img" aria-label="Three censuses. Original linked orders: reference 2.172, kernel 2.770 ns per row, 1.276 times. Cells first census: reference 2.244, kernel 1.697, 0.756 times. Cells second census: reference 2.172, kernel 1.604, 0.738 times."><text x="0" y="26" class="bf-tm" style="font-size: 21px">ns per BWT row, 233 Wikidata blocks</text><text x="0" y="74" class="bf-ts" style="font-size: 25px">Original linked orders</text><text x="0" y="118" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="92" width="156.4" height="36" rx="3" class="bf-bar-est"/><text x="278.384" y="119" class="bf-t-strong" style="font-size: 23px">2.172</text><text x="0" y="164" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="138" width="199.4" height="36" rx="3" class="bf-bar"/><text x="321.44" y="165" class="bf-t-strong" style="font-size: 23px">2.770 (1.276×)</text><text x="0" y="232" class="bf-ts" style="font-size: 25px">Cells, first census</text><text x="0" y="276" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="250" width="161.6" height="36" rx="3" class="bf-bar-est"/><text x="283.568" y="277" class="bf-t-strong" style="font-size: 23px">2.244</text><text x="0" y="322" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="296" width="122.2" height="36" rx="3" class="bf-bar"/><text x="244.184" y="323" class="bf-t-strong" style="font-size: 23px">1.697 (0.756×)</text><text x="0" y="390" class="bf-ts" style="font-size: 25px">Cells, second census (idle core)</text><text x="0" y="434" class="bf-tm" style="font-size: 20px">reference</text><rect x="112" y="408" width="156.4" height="36" rx="3" class="bf-bar-est"/><text x="278.384" y="435" class="bf-t-strong" style="font-size: 23px">2.172</text><text x="0" y="480" class="bf-tm" style="font-size: 20px">kernel</text><rect x="112" y="454" width="115.5" height="36" rx="3" class="bf-bar"/><text x="237.488" y="481" class="bf-t-strong" style="font-size: 23px">1.604 (0.738×)</text></svg>
<figcaption>Dashed: conventional table build + walk, from a column. Solid: the experimental kernel, from run records (the column-to-run scan, 0.484 ns a row, is excluded). Each kernel is compared only with the reference timed in its own census. Local single-thread fastest-of-five figures from #351's journal, with no confidence intervals; not CI, not the whole decoder, and not against 0.2.8's records-based path.</figcaption>
</figure>

The cells kernel recorded lower times on 202 blocks in the first census and 204 in the second. Other work ran on other cores during the first; the second pinned the process to one core with the others idle.

The original 2.770 figure sums the stages' separate fastest rounds. The cells figures time order, induction and output together. So the 42% drop between the first and latest totals compares different runs and different timing methods; it is not a paired measurement.

The two arms also start from different inputs. The conventional arm builds from a column; the experimental kernel receives runs. The separately measured column-to-run scan costs 0.484 ns/row in the journal. A front end that emits runs directly could avoid that scan, but the published table does not measure that integrated path.

This table also leaves out the records-based construction that shipped in 0.2.8 ([#324](https://github.com/lmmx/mibz2/pull/324)), so its 26% figure is not an improvement over a released version. The re-run after the writer fix makes that comparison: 1.677 against 2.049 ns per row, 0.818 times.

On other inputs the cells kernel is slower: 3.93 times its reference on prose, 9.78 times on uniform bytes, and 8.28 times on a near-periodic adversarial input. Those are reported local ratios from the same journal's other-input table, timed before the writer fix; MONODROMY has withdrawn its periodic, Fibonacci and Thue–Morse figures until they are re-measured.

## 3. Why the cells kernel is cheaper

A word stored as a rule needs its bytes and its destination recorded once; its earlier descriptions need not be updated.

The first cells version was still slower: the journal records 1.225 times the reference in its development sequence.

A cell is the overlap of an interval in the original order with one in the image order. It holds the rows that read the same pair of run letters in succession. The kernel writes each cell once, when its step is done, and records where later work goes. This replaces most of the linked lists' shortening and relinking.

A second change removes position tracking from the induction loop. The original row is marked in the grammar, and the output stage finds the starting position from that mark afterwards. So the induction no longer updates a position at every step.

Third, the kernel prefetches the addresses of its scattered writes before it makes them. The journal's code and instruction counts support these three explanations, but the three changes were made and timed together, so their separate effects are not measured.

The cells kernel also uses more memory than the conventional table in the journal's representative block. The saving comes from the order in which memory is accessed; total memory is not reduced.

## 4. The Lyndon tree

Joining words builds a tree: its leaves are bytes, and each internal node joins its children's words. Sharing identical subtrees turns that tree into a reusable grammar.

A **primitive** word is not an exact repetition of a shorter word. Its lexicographically smallest rotation is a **Lyndon word**, smaller than each of its other rotations. For example, `aab` is smaller than `aba` and `baa`; its standard split is `a | ab`, followed by `a | b`.

The journal identifies the right-end induction's expanded derivation tree with the Lyndon tree of that least rotation. It first relates the construction to a **Cartesian tree**: a hierarchy formed by repeatedly splitting a sequence at its minimum. Here the sequence contains ranks of rotations beginning at the word's internal boundaries. For `aab`, the rotations at positions zero, one and two have ranks zero, one and two. The internal sequence is therefore [1,2]. Its minimum splits after the first `a`, giving `a | ab`; the remaining split gives `a | b`. This agrees with the two rules above. In general, the standard split of a Lyndon word cuts it just before its smallest proper suffix. That a Lyndon tree has the shape of the Cartesian tree of the word's suffix ranks is Hohlweg and Reutenauer's result; Crochemore and Russo set it out in §4 of [*Cartesian trees and Lyndon trees*](https://arxiv.org/abs/1712.08749).

<figure class="bf-fig">
<p class="bf-title">How the row map leads to the Lyndon tree</p>
<svg viewBox="0 0 540 820" role="img" aria-label="LF, a constant shift within each run, feeds right-end induction, which keeps word rules. Expanding the rules gives a Cartesian tree of rotation ranks, identified with the Lyndon tree of the least rotation for a primitive word. For aab the ranks at the inner cuts are 1 and 2; the tree splits a | ab, then a | b."><rect x="0" y="0" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="34" class="bf-ts" style="font-size: 25px">LF on the block's rows</text><text x="18" y="66" class="bf-tm" style="font-size: 21px">a constant shift within each run</text><path d="M60 86V120" class="bf-arrow"/><path d="M53 111L60 120L67 111" class="bf-arrow"/><rect x="0" y="126" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="160" class="bf-ts" style="font-size: 25px">Right-end induction</text><text x="18" y="192" class="bf-tm" style="font-size: 21px">keeps word rules: a grammar</text><path d="M60 212V246" class="bf-arrow"/><path d="M53 237L60 246L67 237" class="bf-arrow"/><text x="80" y="236" class="bf-tm" style="font-size: 21px">expand the rules</text><rect x="0" y="252" width="540" height="82" rx="8" class="bf-card" style="stroke: var(--bf-frame)"/><text x="18" y="286" class="bf-ts" style="font-size: 25px">Cartesian tree of rotation ranks</text><text x="18" y="318" class="bf-tm" style="font-size: 21px">split at the minimum, recursively</text><path d="M60 338V372" class="bf-arrow"/><path d="M53 363L60 372L67 363" class="bf-arrow"/><text x="80" y="362" class="bf-tm" style="font-size: 21px">identified with</text><rect x="0" y="378" width="540" height="82" rx="8" class="bf-card-hi" style="stroke: var(--bf-frame)"/><text x="18" y="412" class="bf-ts" style="font-size: 25px">Lyndon tree of the least rotation</text><text x="18" y="444" class="bf-tm" style="font-size: 21px">for a primitive word (journal §5.6)</text><path d="M0 476H540" class="bf-rule"/><text x="0" y="514" class="bf-ts" style="font-size: 23px">For aab: ranks at the inner cuts are [1, 2]</text><text x="0" y="544" class="bf-tm" style="font-size: 21px">split at the minimum: a | ab, then a | b</text><path d="M270 614L150 666" class="bf-link"/><path d="M270 614L390 666" class="bf-link"/><path d="M390 714L320 766" class="bf-link"/><path d="M390 714L460 766" class="bf-link"/><rect x="226.0" y="566" width="88" height="48" rx="6" class="bf-pill-hi"/><text x="270" y="599" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">aab</text><rect x="122.0" y="666" width="56" height="48" rx="6" class="bf-pill"/><text x="150" y="699" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">a</text><rect x="354.0" y="666" width="72" height="48" rx="6" class="bf-pill-hi"/><text x="390" y="699" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">ab</text><rect x="292.0" y="766" width="56" height="48" rx="6" class="bf-pill"/><text x="320" y="799" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">a</text><rect x="432.0" y="766" width="56" height="48" rx="6" class="bf-pill"/><text x="460" y="799" class="bf-t-strong" text-anchor="middle" style="font-size: 25px">b</text></svg>
<figcaption>The identity concerns the expanded derivation tree; stored rules share subtrees, and the kernel need not build every node. It is proved in #351's journal and checked by simulators there, under the primitive-word condition (aab is primitive; abab is not). This post checked the record; it does not independently certify the theorem. Green: the two internal nodes, which are the two rules of §1.</figcaption>
</figure>

Lyndon trees are long established. The new result is that this interval construction reaches the Lyndon tree without first rebuilding the text by a conventional walk.

The journal includes a proof and reports independent simulators and node comparisons. This post has checked that record, not independently certified the theorem. The proof needs the word to be primitive: `aab` is, while an exact repetition such as `abab` needs separate treatment.

## 5. Fewer steps by a different order

It does in the step counts. Whether it saves time is not yet known.

Let `r` be the number of BWT runs. The journal compares induction steps divided by `r`, so blocks with different run counts can be compared. A “free” step here means that its bookkeeping creates no new boundary; it still costs processor work.

The favourable order takes available free steps first, then chooses the largest cell. Its count is 1.279 steps per run, compared with 1.812 for the right-end sweep.

<figure class="bf-fig">
<p class="bf-title">The order of steps changes how many there are</p>
<svg viewBox="0 0 540 414" role="img" aria-label="Steps per run: left end 2.137, right end 1.812, free steps first then largest cell 1.279, against a floor of 1.220 for this step model."><text x="0" y="26" class="bf-tm" style="font-size: 21px">induction steps per BWT run, 233 Wikidata blocks</text><text x="0" y="72" class="bf-ts" style="font-size: 25px">Left end</text><rect x="0" y="86" width="416.7" height="38" rx="3" class="bf-bar"/><text x="428.715" y="114" class="bf-t-strong" style="font-size: 24px">2.137</text><text x="0" y="180" class="bf-ts" style="font-size: 25px">Right end (as built)</text><rect x="0" y="194" width="353.3" height="38" rx="3" class="bf-bar"/><text x="365.34000000000003" y="222" class="bf-t-strong" style="font-size: 24px">1.812</text><text x="0" y="288" class="bf-ts" style="font-size: 25px">Free first, then largest cell</text><rect x="0" y="302" width="249.4" height="38" rx="3" class="bf-bar"/><text x="261.405" y="330" class="bf-t-strong" style="font-size: 24px">1.279</text><path d="M237.9 40V364" class="bf-grid" style="stroke: var(--bf-green); stroke-width: 2.5"/><text x="0" y="396" class="bf-t-green" style="font-size: 23px">dotted: floor r₂ − 1 = 1.220 for these steps</text></svg>
<figcaption>These are step counts, not times: the cells kernel cannot yet run the cheaper order cheaply. The floor holds for any order of these cell steps (r₂ − 1, the cells of both orders' cuts, minus one); a fusion outside this step model can go below it, so it is not a lower bound on inverse BWT.</figcaption>
</figure>

The journal's identity is `steps = initial cells − 1 + newly introduced boundaries`. In the toy example, the union of both orders' cuts gives three initial cells. Two steps leave one, with no new boundary introduced. A re-seating into an interval's interior can instead add a cut that a later step must remove.

The identity gives a floor of 1.220 steps per run for these cell-step schedules. The favourable order is about 29.4% below the right-end count and 4.8% above that floor.

The floor applies only to this step model. The journal describes a fusion operation outside it that can go below 1.220, so it is not a lower bound on inverse BWT itself.

The fast cells kernel relies on sweeping the ends of its orders. In the version this post links to, the journal counts an out-of-order step, as built, at 152 instructions, against 64 for a swept cell. Fewer steps can therefore lose if each step becomes expensive enough. MONODROMY has since counted a worklist that takes steps in the cheaper order. It does not pay on one thread: it tests 2.25 segment ends per step, and two steps in three move to another segment.

The same applies to parallel rounds. A schedule model finds enough independent work for up to sixteen workers, but it leaves out the cost of running them. Run sequentially, the round-based implementation took about 1.99 times its reference, and no multi-worker run has been timed.

## 6. Before production

The timings justify continuing the experiment. They do not show a general-purpose replacement or a result for a release.

The next comparison needs matched input boundaries and the relevant records-based reference. The unsafe kernel still needs independent safety review. The journal's tests do not replace that review, and adverse inputs need a bounded fallback or a different reduction.

MONODROMY's five-part verification is under way. Its memory-safety check found the writer defect above, now fixed; the new census and the checks of numbers, proofs and the referee reading are still to come. The code this post links to predates the fix.

MONODROMY's earlier conjecture, that the grammar follows the Cartesian tree of the rotations' ranks, is now a recorded identity: the grammar is the Lyndon tree. Computing every rotation's rank first, however, could cost as much as the reconstruction this approach seeks to avoid.

The next question is whether the Lyndon tree can be built more cheaply than by running the induction step by step. Within the induction, the order with fewer steps only pays once the machine can execute it cheaply too.
