[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/index.md>)

# The width of an induction: how much it must hold at once

Taking the steps that cost no bookkeeping first would let mibz2 rebuild a block while keeping about a third as much in memory at its busiest moment.

15 November 2026 · 14 min read · [Research in progress](<#5-what-would-it-mean-for-mibz2>)

[A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) introduced Cells, mibz2's experimental way of undoing the Burrows–Wheeler transform (BWT) that bzip2 applies to every block. Two posts this week cut the memory Cells uses: [hand-me-down slots](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-15-slot-recycling/index.md>) and [one copy of each interval's details instead of two](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-15-bottom-slots/index.md>). This post asks a different question about the same memory: how much of it does the mathematics actually need at any one moment?

**The recap.** bzip2 sorts the rotations of a block and stores the last byte of each, the *last column*; each sorted rotation is a *row*. To decode, mibz2 follows a map called LF from row to row, reading one byte per step, which spells out the block backwards. A *run* is a stretch of equal bytes in the last column, and LF moves each run's rows as one shifted interval. Cells works on those intervals instead of on rows. Each step deletes some rows from the map and lets one interval jump straight over them. That interval now stands for more than one byte of text. When one interval is left, it stands for the whole block. The code calls each interval a *letter*, and the text it stands for its *word*. At the start there is one letter per run, and every word is one byte long.

**What width is.** At any moment of the rebuild, some letters still have a one-byte word. Each of those is an untouched piece of an original run. Its length, place and byte can be read off the list of runs, so in principle it needs no record of its own. Only the letters whose word has grown longer than one byte are new information. Call the most of those alive at once the **width** of the induction (the rebuild's sequence of steps), W. It is counted, like everything here, as a share of r, the number of runs in the block. Cells today writes a slot for every one of the r letters before its first step, and keeps r slots throughout.

**What was found.** Width depends on the order in which the steps are taken, by about three times.

- **Cells' order** always takes its step at the right-hand end of the map. At its busiest it holds about **0.32 r** letters with long words. That is the median over the 233 large blocks of the benchmark's Wikidata corpus, an extract of a Wikidata dump.
- **Taking the free steps first** brings that peak down to about **0.08 to 0.12 r**, depending on what is done when no free step is ready. It is **0.11 r** if the rest follows Cells' own order. A *free* step is one whose bookkeeping creates no new boundary between intervals ([section 3](<#3-does-the-order-of-steps-matter>) shows one). The same order also takes about 20% fewer steps: 1.46 r instead of 1.84 r.
- **The free steps have a fixed end point.** Taken in any order until none is left, they always reach the same state, in the same number of steps. That is now proved ([\#438](<https://github.com/lmmx/mibz2/pull/438>)). It follows that a kernel could take free steps in whichever order suits it, or share independent ones among threads. On Wikidata the end state has about 0.19 r letters, and no free-first order's peak exceeded it on any block.

How many letters the rebuild holds, per 100 runs of a Wikidata block

Counted, not timed: medians over the 233 blocks of at least 65,536 rows in the Wikidata corpus. Amber and green are the peaks of letters whose word is longer than one byte, under Cells’ right-end order and under free steps first. Grey is the number of letters, long words or not, left once the free steps run out.

**What is classical and what is new.** The steps are Rauzy–Veech induction, a 1979 construction in the theory of dynamical systems, and the words are the return words that theory has studied since. Counting the most words alive at once is not part of it. "Width" in this sense is not a standard term of that theory. It is a new measure, and its closest relatives are graph measures such as *cutwidth*: line a graph's vertices up, cut the line between any two, and count the edges that cross. [Section 2](<#2-why-do-only-the-long-words-count>) says where the line between old and new falls.

**What this is not.** These are counts, not timings. They come from one family of text, Wikidata's JSON, with four blocks of prose beside it. Prose gains less: the right end peaks at 0.37 r and free-first orders at 0.21 r. The link to cutwidth is a conjecture. Above all, no decoder that takes free steps first has been built or timed. Finding the free steps needs a worklist, which an earlier count priced at about 150 instructions a run against the right-end sweep's 115. Whether the smaller state pays for that is the next question, not an answer here.

**Why mibz2 cares.** Width is how much state the rebuild keeps in memory. When every thread decodes a block of its own, those states share the processor's cache, and the threads contend for it. Less state per block leaves less for the cache to hold. At ten threads on a 10-core Intel desktop, a block per thread takes 1.040 times the wall time of threads working in teams (95% interval 1.019 to 1.061; [the run's report](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900029992>)), so the margin is small. [Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-16-lazy-letters/index.md>) tested it: holding only the letters still in play, one block per thread beat the teams by 5% at ten threads, and mibz2 0.5.0 uses it from 8 threads up.

[Section 1](<#1-what-does-the-rebuild-hold-on-a-nine-byte-word>) works a nine-byte word by hand and draws its width. [Section 2](<#2-why-do-only-the-long-words-count>) says why only the long words count, and what is classical. [Section 3](<#3-does-the-order-of-steps-matter>) compares twelve orders of steps, and [section 4](<#4-how-narrow-can-a-free-first-order-be>) gives the bound and the theorem. [Section 5](<#5-what-would-it-mean-for-mibz2>) says what is open, and [section 6](<#6-where-to-read-more>) lists the reading.

## 1\. What does the rebuild hold, on a nine-byte word?

Suppose you want to compress the word `possesses`. Its nine rotations, sorted, are its nine rows, and the last byte of each gives the last column `sspssseeo`. LF sends each row to the row of the rotation one byte earlier in the text:

| Row | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Last column | s | s | p | s | s | s | e | e | o |
| LF | 4 | 5 | 3 | 6 | 7 | 8 | 0 | 1 | 2 |

The last column has five runs: `ss`, `p`, `sss`, `ee` and `o`. So the rebuild starts with five letters, each with a one-byte word.

Cells' order removes rows from the right-hand end, so the rows it still keeps are always a prefix, rows 0 to m − 1. Call m the *cut*. Each kept row's letter now jumps from that row along LF, through the removed rows, until it lands on a kept row again. Mathematicians call this the *first return* to the kept rows. The bytes read on the way, taken in text order, are the letter's word.

After four of its seven steps, Cells' order has cut the word at m = 4. The upper panel below draws that moment. Row 0, for example, reads `s` and goes to row 4. Row 4 is removed, so the path goes on, and reads `s` there. It goes on to row 7, also removed, and reads `e`. Then it returns to row 1, which is kept. Read in text order, its word is `ess`. Rows 1 and 3 likewise have the words `oss` and `es`. Row 2 goes straight to row 3, so its word is still the one byte `p`. Three letters out of four have long words, and that is the most this order ever holds on this word: its width is 3.

The same word at the busiest moment of two orders of steps

Worked by hand and checked by a script. Each box is a row with its byte of the last column; dashed boxes are removed rows. Each arc is one LF link, with its arrow at the row it goes to. Under each kept row is its letter’s word: green where it is longer than one byte. Green arcs are the paths that build those words. The removed rows matter only as places a path passes through.

The lower panel takes the free steps first, and section 3 explains what makes a step free. It removes rows 0–2 first, not the right-hand end. It never holds more than two long words at once, so its width is 2. It also finishes in six steps, not seven. The counts after each step, from the start to the single letter at the end, are:

| Order | After each step | Width | Steps |
| --- | --- | ---: | ---: |
| Right end | 1, 2, 2, **3**, **3**, 2, 1 | 3 | 7 |
| Free steps first | 1, **2**, 2, 2, 2, 1 | 2 | 6 |

Each long word can be checked in `possesses`: `ess`, `oss` and `es` occur in it, and so do `se` and `po`. The steps themselves are animated, on the three-byte word `aab`, in [the post that introduced Cells](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-10-rauzy-veech-reconstruction/#1-rebuilding-aab-in-two-steps>).

## 2\. Why do only the long words count?

A letter whose word is one byte is a row, or a few adjacent rows, that LF sends straight to another kept row. It is a piece of an original run, cut shorter. Its first row, its length and the row it goes to all follow from the run it lies in. A decoder could leave it implicit and compute it when needed. A letter with a long word is different: where it jumps, and what it reads on the way, are the result of the steps taken so far.

The long words can be read from the arcs. A kept row has a long word exactly when its LF link crosses into the removed rows. Under Cells' order, the research record proves where each long-word letter begins ([the census entry](<https://github.com/lmmx/mibz2/blob/f03b0fdca71abf45133d0720802ff3d3255a567a/docs/journal/monodromy/2026-11-15-width.md>)). It begins either at a run's first row whose link crosses the cut, or at the kept row whose path through the removed rows passes a run's first row. So the width counts crossings of the cut, bundled by runs.

Width, stated formally and read on `possesses`

**1\. First return.** Let \\(T\\) be a bijection of a finite set \\(X\\) and \\(D \\subseteq X\\). The first return of \\(T\\) to \\(D\\) is \\(T\_D(x) = T^{\\tau(x)}(x)\\), where \\(\\tau(x) \\ge 1\\) is the least power that lands back in \\(D\\).

Here \\(T\\) is LF on the rows \\(\\{0, \\dots, n-1\\}\\), and \\(D\\) is the set of kept rows: \\(\\{0, 1, 2, 3\\}\\) at the right end’s busiest moment.

**2\. Letters and words.** \\(T\_D\\) is a translation on each of finitely many intervals of \\(D\\). These are the letters \\(\\mathcal{A}\_t\\) after step \\(t\\). The word \\(w\_t(a)\\) of a letter is the sequence of bytes read in its \\(\\tau\\) moves, in text order, and \\(|w\_t(a)| = \\tau\\).

Here Row 0 takes \\(\\tau = 3\\) moves, through rows 4 and 7, and reads `ess`. Row 2 takes one move, to row 3, and reads `p`.

**3\. Width.** Over the steps of one order, \\(W = \\max\_t \\bigl|\\{a \\in \\mathcal{A}\_t : |w\_t(a)| \> 1\\}\\bigr|\\). The number of steps is \\(S\\).

Here Right end: \\(W = 3\\), \\(S = 7\\). Free steps first: \\(W = 2\\), \\(S = 6\\). On Wikidata: \\(W \\approx 0.32\\,r\\) against \\(0.11\\,r\\), and \\(S \\approx 1.84\\,r\\) against \\(1.46\\,r\\).

Definitions 1 and 2 are the classical first-return construction behind Rauzy–Veech induction, for which Viana’s survey (2006) is an introduction, in a finite, discrete form: rows instead of a real interval. Definition 3 is the research record’s, not the literature’s.

**What is classical.** Inducing a map on a smaller set by its first return, and shrinking the set step by step, is Rauzy's induction (1979). Veech (1982) developed its dynamical and geometric meaning. Writing the orbit segments as words, so that each step becomes a substitution such as \\(a \\mapsto ab\\), was made explicit by Lopez and Narbel (1999). Choosing a different path of steps, instead of always cutting at one end, is the *branching* Rauzy induction of Dolce and Perrin (2017).

**What is new.** That theory studies orbits, return times, invariant measures and coding. It does not ask how many words a computation must hold at once along a path of steps. Its usual setting also keeps the number of intervals fixed. In mibz2's discrete setting, a step between two letters of equal length deletes one, so the count falls as the rebuild goes on. The closest established ideas come from graph layout. Line up a graph's vertices in some order and cut the line after each prefix. The *cutwidth* of that order is the most edges any cut crosses, and *vertex separation* counts boundary vertices instead of edges. The least vertex separation over all orders equals the graph's *pathwidth* ([Kinnersley 1992](<https://doi.org/10.1016/0020-0190(92)90234-M>)). Width here is the same kind of quantity, with the block's cycle of rows as the graph, its edges bundled into runs, and the kept rows as the prefix. Whether W is exactly a cutwidth is a conjecture. A stand-in for it, the runs crossing the cut upwards, peaks at 0.28 r over the 233 blocks. The exact count on 29 of them peaks at 0.31 r.

**A trap in the terms.** The geometric side of the theory draws each interval as a rectangle, in *zippered rectangles*, and calls the interval's length its rectangle's width. That is λ, a length, not a count of live words, and has nothing to do with W.

## 3\. Does the order of steps matter?

Every step of the rebuild happens where a letter's interval in the map and another letter's landing place end on the same row. The research record calls such a row a *coincidence*. Cells' order always takes the coincidence at the right-hand end. Any other coincidence can be taken instead. Every order rebuilds the block, and the research record proves that the orders counted here build the same rules, the block's Lyndon tree ([the Cells post's §5](<https://lmmx.github.io/mibz2/pr-preview/pr-585/blog/2026-11-10-rauzy-veech-reconstruction/#5-fewer-steps-by-a-different-order>)). They differ in when each rule is made, and so in how long each letter lives.

A step is *free* when it creates no new boundary: either the two letters have the same length, so one is deleted, or the shorter one's rows land where a boundary already is. In `possesses` at the start, the first free step is at the boundary after row 2. The letter on row 2, `p`, ends there, and so does the place where the letter on row 8, `o`, lands. Both are one row long. The step deletes row 2, and the letter on row 8 now jumps straight to row 3, where `p` went, reading `po`. A step that is not free, a *paid* step, splits an interval somewhere new, and a later step must remove that new boundary. Each free step removes one boundary and adds none.

The census ran twelve orders on every Wikidata block of at least 65,536 rows, and on the four prose blocks. Every final word was checked to be a rotation of its block ([the orders entry](<https://github.com/lmmx/mibz2/blob/f03b0fdca71abf45133d0720802ff3d3255a567a/docs/journal/monodromy/2026-11-15-width-orders.md>); its [summary](<https://github.com/lmmx/mibz2/blob/f03b0fdca71abf45133d0720802ff3d3255a567a/docs/journal/monodromy/2026-11-15-width-orders/summary.txt>)). When no free step is ready, a free-first order needs a rule for the paid step: the right end, the largest cell, the smallest, or the leftmost. A *cell* is the stretch of rows a step removes. The orders "told from links" decide whether a step is free from the neighbouring letters alone, as a kernel would have to.

Width under twelve orders of steps, Wikidata and prose

Counted, not timed, from the census’s summary. Each dot is the median width over 233 Wikidata blocks, the line runs from the narrowest block to the widest, and the diamond is the median over 4 prose blocks. Steps are the Wikidata median, per run. Hover a row for its exact figures.

The orders fall into two groups. Every order that takes free steps first has a median width between 0.080 and 0.124 r. Every other order lies between 0.27 and 0.35 r, apart from one that takes only the equal-length free steps first, at 0.171 r. The step count does not decide it. "Free first, else the smallest cell" takes 2.26 r steps, the most of any order, and still has a width of 0.108 r. What sets the width is how soon a letter is closed once it can be.

Per block, the right end's width is 2.04 to 4.84 times that of "free first (told from links), else the right end". The median is 2.98. The right end's exact width here, 0.318 r, is a little above an earlier count of 0.311 r. That count sampled 31 cuts per block on 29 blocks, so it could only miss the peak.

## 4\. How narrow can a free-first order be?

Run free steps until none is left, and call the state reached the *free closure*. On Wikidata it leaves 0.185 r letters at the median (0.075 to 0.281 r), after 0.896 r steps.

**The closure is the same whatever the order (proved).** Free steps taken in any order until none is left always reach the same letters, the same words and the same number of steps. The one exception is a cycle of the map that closes completely within the free steps. Such a cycle takes no further part, and the orders can differ only in which rotation of its word it holds. No real block closes its cycle that early. The proof shows two things. Free steps always stop, since each removes a boundary. And any two free steps available together can be taken in either order and reach the same state again, which it checks case by case. By Newman's lemma, that is enough for a unique end point. A machine check of 10.7 million pairs of steps on short columns found no exception ([\#438](<https://github.com/lmmx/mibz2/pull/438>), [its entry](<https://github.com/lmmx/mibz2/blob/e8c5705082a5908916bd8804d8e11c6bb9cbdbc0/docs/journal/monodromy/2026-11-15-free-closure-confluence.md>) and [log](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900899853>)). So the closure is a property of the block, like a normal form, and its size is a number the block determines.

**The closure bounds the width (proved after it, counted before it).** After the closure no step adds a letter, so the long words alive can never exceed the closure's letters again. Before the closure, the census found the same on every one of the 233 blocks, under three free-first orders. The width was at most 0.74 of the closure's letters, and 0.45 to 0.57 of them at the median. On a nine-byte word the bound can fail. Every step of `beekeeper` is free, so its closure is one letter, but its width under free steps first is at least 2. On real blocks the bound held without exception.

**What follows for a kernel (a consequence, not a measurement).** A kernel may take free steps in whichever order is cheapest for it, and still reach the same state. Free steps that touch different letters can be given to different threads, and they still reach that state. And a kernel that takes free steps first would need live state no larger than the closure, about 0.19 r letters. On these blocks its peak is nearer 0.11 r.

## 5\. What would it mean for mibz2?

None of this is in the decoder. A kernel that used it would keep letters with one-byte words implicit, reading them from the run list, and write a letter only when its word first grows. Under Cells' order it would hold about 0.32 r letters at most; with free steps first, about 0.11 r. Cells today holds r.

The cost is in finding the steps. Cells sweeps the right-hand end, where the next step is always known, and that sweep is also why its cache misses are only first touches ([\#397](<https://github.com/lmmx/mibz2/pull/397>)). A free-first kernel needs a worklist of ready steps instead. An earlier count priced such a worklist at about 150 instructions a run against the sweep's 115, on one thread. Two things have changed in its favour since. The steps fall from 1.84 r to 1.47 r, and the state to hold falls by about three times. Whether that pays at ten threads sharing one last-level cache is a timing question, and it is the next one for the kernel work.

Three questions remain open:

- **Cutwidth.** Is W exactly a cutwidth of the block's cycle, and can it be bounded from the run lengths alone? This is a conjecture.
- **Speed.** Is any kernel that takes free steps first faster than Cells? Nothing has been timed.
- **The closure without the steps.** An equal-length step says that every row of one letter goes next to the other: a forced adjacency. Genome assemblers join every forced adjacency at once when they compact a graph's unbranched paths, called *unitigs*. That compaction does not depend on any order. The conjecture is that the free closure is exactly that compaction. If so, it could be computed without simulating any step.

The width census and the order census are merged ([\#435](<https://github.com/lmmx/mibz2/pull/435>), [\#437](<https://github.com/lmmx/mibz2/pull/437>)). The confluence proof, [\#438](<https://github.com/lmmx/mibz2/pull/438>), is in review.

## 6\. Where to read more

In order of use for this question:

1. Marcelo Viana, [Ergodic theory of interval exchange maps](<https://doi.org/10.5209/rev_REMA.2006.v19.n1.16621>), *Revista Matemática Complutense* 19 (2006), 7–100. A survey, and the gentlest way in to first returns and Rauzy–Veech induction.
2. Luis-Miguel Lopez and Philippe Narbel, Substitutions from Rauzy induction, in *Developments in Language Theory* (DLT 1999, World Scientific), 200–209. Rauzy induction read as substitutions on words.
3. Francesco Dolce and Dominique Perrin, [Interval exchanges, admissibility and branching Rauzy induction](<https://doi.org/10.1051/ita/2017004>), *RAIRO – Theoretical Informatics and Applications* 51 (2017), 135–139 ([arXiv](<https://arxiv.org/abs/1305.0120>)). Choosing among many paths of steps, the freedom section 3 uses.
4. Jon Fickenscher, [Decoding Rauzy induction: an answer to Bufetov's general question](<https://doi.org/10.24033/bsmf.2748>), *Bulletin de la Société Mathématique de France* 145 (2017), 603–621 ([arXiv](<https://arxiv.org/abs/1507.07481>)). Despite the title, not about decompression: it shows how an interval exchange is recovered from the products of its induction's steps.
5. Gérard Rauzy, [Échanges d'intervalles et transformations induites](<https://doi.org/10.4064/aa-34-4-315-328>), *Acta Arithmetica* 34 (1979), 315–328, and William A. Veech, [Gauss measures for transformations on the space of interval exchange maps](<https://doi.org/10.2307/1971391>), *Annals of Mathematics* 115 (1982), 201–242. The original papers; not the place to start for width.
6. Josep Díaz, Jordi Petit and Maria Serna, [A survey of graph layout problems](<https://doi.org/10.1145/568522.568523>), *ACM Computing Surveys* 34 (2002), 313–356. Cutwidth, vertex separation and their relatives.

---

**Research records:**

- The width census and its programs: [`2026-11-15-width.md`](<https://github.com/lmmx/mibz2/blob/f03b0fdca71abf45133d0720802ff3d3255a567a/docs/journal/monodromy/2026-11-15-width.md>) (\#435).
- Width by order and the free closure: [`2026-11-15-width-orders.md`](<https://github.com/lmmx/mibz2/blob/f03b0fdca71abf45133d0720802ff3d3255a567a/docs/journal/monodromy/2026-11-15-width-orders.md>) (\#437).
- The confluence proof: [`2026-11-15-free-closure-confluence.md`](<https://github.com/lmmx/mibz2/blob/e8c5705082a5908916bd8804d8e11c6bb9cbdbc0/docs/journal/monodromy/2026-11-15-free-closure-confluence.md>) (\#438, in review).
- The thought experiment that defined width: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900062843>).
- The run of a block per thread against teams at ten threads: [the team channel, 15 November](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6900029992>).
