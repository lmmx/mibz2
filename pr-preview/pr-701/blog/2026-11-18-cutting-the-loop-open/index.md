[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/index.md>)

# Cutting the loop open: decoding a block along shared paths

Snip the decoder's one long loop open and most of each Wikidata block falls into bundled paths, where one traced letter spells about sixteen.

18 November 2026 · 13 min read · [Research result](<#7-what-comes-next>)

Much of mibz2's research is about one stage of bzip2 decoding: turning each block's Burrows–Wheeler transform back into text. [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) showed how the kernel mibz2 ships, Cells, does it by shrinking a map instead of walking it. [The mathematical dictionary](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-16-mathematical-dictionary/index.md>) named the objects that kernel works with. This post takes the same map apart another way. It cuts the map's one long loop open, solves the pieces separately, and joins them back.

**The setting, in brief.** bzip2 stores a block as the last column \\(L\\) of the block's rotations, sorted: the Burrows–Wheeler transform (BWT). It also stores the *primary row*, the row that holds the original text. A map called **LF** sends each row to the row of the rotation one byte earlier. Following LF from the primary row reads the block backwards, one byte a step, until it comes back to the primary row. That trip, the *marked loop*, is the decoder's loop. When LF is a single loop, as in the example below, that trip visits every row once. It need not. The periodic text `abab` has the column `bbaa`, and its LF makes two loops, 0 → 2 → 0 and 1 → 3 → 1. The marked loop covers only half the rows, yet the decoder still writes the whole block. One lap reads `ba` backwards, which is `ab` forwards, and the decoder repeats that spelling up to the block's known length of four bytes, giving `abab`. A **run** is a stretch of equal bytes in \\(L\\). LF moves all of a run's rows by the same amount, so the map is a list of shifted blocks of rows ([One shift per run](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-08-inverse-bwt-navigation/index.md>)).

**Why the loop is slow.** Each step needs the row that the step before it found, so a plain decoder makes up to 900,000 memory reads per block, each waiting on the last. Cells avoids most of them by working on runs, not rows. It comes in two forms. Eager Cells lays out all its records before it starts. Lazy Cells makes them as it goes, which suits many threads at once ([Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-16-lazy-letters/index.md>)).

**The idea.** Choose some rows to keep, the primary row among them, and cut the arrow leaving each one. The marked loop falls into open paths. Each path starts at an *entrance*, a row whose incoming arrow was cut, and ends at a kept row. Where each path ends, and what it reads on the way, can be found from the uncut arrows alone. What the cut loses is which kept row's arrow went to which entrance. That is rebuilt separately, from a stable sort of the kept rows' letters. Joining the two gives back the loop, and the text.

**What we found:**

- **The loop factorises.** The trip from one kept row to the next is the open-path map applied after the stable sort of the kept letters, and neither needs the other's input. This is proved, and the proof has been verified twice, independently.
- **It is a known operation.** In Markov-chain theory it is the *stochastic complement* (Meyer, 1989): the chain watched only on the kept states. Here the chain is one permutation with a letter on each arrow, so the complement's infinite sum stops.
- **The pieces are few.** The open paths travel in *bundles* of neighbouring rows. If the rows left on the open paths form \\(H\\) wide strips (stretches of long runs), there are at most \\(3H + \\sigma\\) bundles, where \\(\\sigma\\) counts the distinct bytes. Few bundles does not by itself mean little work.
- **Keep the short runs.** On Wikidata the cut keeps every run of 16 rows or fewer. On a test block those are 93% of the runs but hold 10% of the rows. Keeping them removes most of the places where neighbouring paths would split, while 90% of the rows stay on open paths.
- **One traced letter spells about sixteen.** Over the large Wikidata blocks, bundles cover a median 88.7% of each block, and tracing one path per bundle reads 5.5% of it. Two blocks of nearly equal run density differ by 66% in that cost.
- **Sixteen is the saving in finding the paths, not in decoding.** The decoder still writes every output byte, and still builds and walks a table of the kept rows' returns, one entry per kept row.
- **The decoder built on it,** in one decode of the test block, asked for about 40% less additional heap than eager Cells. Over the whole Wikidata benchmark, on one four-CPU cloud virtual machine without SMT siblings, it is 9–21% slower than eager Cells and 7–8% faster than lazy Cells. The test with many blocks competing for cache is still to come.

Each claim below says whether it is **proved**, **counted** on the blocks, **measured** on a machine, or an **interpretation** that relates it to other work.

Sections [1](<#1-what-happens-when-the-loop-is-cut>) to [3](<#3-stitching-the-loop-back-together>) cut, rebuild and join a seven-byte example by hand; section 2 is the key step. [Section 4](<#4-the-same-move-in-markov-chain-theory>) relates the result to the stochastic complement and to Cells. Sections [5](<#5-how-many-pieces>) and [6](<#6-what-do-real-blocks-look-like-after-the-cut>) count the bundles, in theory and on Wikidata, and [section 7](<#7-what-comes-next>) reports the decoder, its table and its costs. The [technical notes](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-18-cutting-the-loop-open-notes/index.md>) hold the longer proofs.

## 1\. What happens when the loop is cut?

Take the seven bytes `banana$`, where `$` sorts before every letter and appears once. (It keeps the example small; bzip2 blocks don't need one.) Write out the seven rotations and sort them. The last column, read down, is \\(L\\) = `annb$aa`, and the text itself is row 4.

The sorted rotations of `banana$`, with each row's last byte and its LF arrow:

| Row | Rotation | \\(L\\) | LF |
| ---: | :--- | :---: | ---: |
| 0 | `$banana` | `a` | 1 |
| 1 | `a$banan` | `n` | 5 |
| 2 | `ana$ban` | `n` | 6 |
| 3 | `anana$b` | `b` | 4 |
| 4 | `banana$` | `$` | 0 |
| 5 | `na$bana` | `a` | 2 |
| 6 | `nana$ba` | `a` | 3 |

LF comes from \\(L\\) alone: \\(\\mathrm{LF}(i) = C\[L\[i\]\] + \\mathrm{rank}\\). Here \\(C\[c\]\\) counts the block's bytes smaller than \\(c\\), and the rank counts the earlier rows that also end in \\(L\[i\]\\). Row 5 ends in `a`. One byte is smaller, and one earlier row ends in `a`, so \\(\\mathrm{LF}(5) = 2\\). From row 4 the arrows make one loop through all seven rows, and read `$ananab`: `banana$` backwards. Row numbers are places in the sorted table, not in the text, so each step must wait for the last.

The stepper below keeps rows 0, 4 and 5 and cuts the arrows leaving them. Watch why only paths are left. Every row has exactly one incoming arrow, and a row on a loop already uses it to stay on the loop. An entrance has lost its incoming arrow, so a path that starts there can never join a loop. It must run until the arrows stop, at a kept row. Each press of Next makes one change and says why.

Step through it: cut the arrows leaving the kept rows, then follow each open path

← BackPlayNext →

1. **The loop:** the arrows go 4 → 0 → 1 → 5 → 2 → 6 → 3 → 4 and read `$ananab`, `banana$` backwards.
2. **Keep rows 0, 4 and 5.** In row order they are exit indices 0, 1 and 2.
3. **Cut the arrows leaving them:** 4 → 0, 0 → 1 and 5 → 2. Rows 0, 1 and 2 lose their incoming arrows and become entrances 0, 1 and 2.
4. **Entrance 0** is row 0, a kept row: no steps, exit index 0, no letters.
5. **Entrance 1:** row 1 → row 5, reading `n`. Exit row 5 is exit index 2.
6. **Entrance 2:** row 2 → 6 → 3 → 4, reading `nab`. Exit row 4 is exit index 1.
7. **So** P = \[0, 2, 1\], with the words: none, `n` and `nab`. No path joined a loop: each row of a loop already uses its one incoming arrow.

Use Next and Back, or Play. Circles are rows, with their last-column letters; arrows are LF. An amber ring is a kept row, where open paths end; a green dot is an entrance, where an open path starts. Think of the loop as a circular railway with a station at every row: keeping a row makes it a terminus, the track out of it is lifted, and each lifted track's far end becomes the first station of a line. The analogy stops at order: a railway's stations have no ranking, and here the rows' sorted order is what rebuilds the lifted track (section 2).

**What exactly is \\(P\\)?** Number the kept rows in increasing order, \\(s\_0, s\_1, s\_2\\) = rows 0, 4, 5, and the entrances likewise, \\(e\_0, e\_1, e\_2\\) = rows 0, 1, 2. Then \\(P(k) = i\\) when the path from entrance \\(e\_k\\) ends at kept row \\(s\_i\\). So \\(P\\) maps entrance indices to exit indices. It is not a map of text positions, and it is not LF. It records how the open paths connect, in the entrances' and exits' own numbering. Here an entrance's row and its index happen to agree. An exit's need not: exit row 5 is exit index 2.

The three open paths of `banana$`, with rows and indices kept apart:

| Entrance row | Entrance index \\(k\\) | Path | Exit row | Exit index \\(P(k)\\) | Word \\(w\_k\\), in LF order |
| ---: | ---: | :--- | ---: | ---: | :--- |
| 0 | 0 | 0 | 0 | 0 | none |
| 1 | 1 | 1 → 5 | 5 | 2 | `n` |
| 2 | 2 | 2 → 6 → 3 → 4 | 4 | 1 | `nab` |

**Why each kept row ends exactly one path.** A kept row has no outgoing arrow, and every other row on a path has one of each. Run the stepper's argument backwards from a kept row, and it reaches exactly one entrance. Any loops left over never meet a cut, and play no part in the text read from the primary row.

## 2\. Where do the cut arrows go?

The paths needed only the arrows that were left. Putting the loop back together also needs the cut arrows: from each kept row, which entrance does its arrow reach? Looking that up row by row would bring back the walk. The answer comes instead from how LF is built.

LF is a **stable sort**. It lists the rows by their last letter, keeping equal letters in row order, and each row's arrow points to its place in that list. Now strike out every row that is not kept. What remains is still sorted by letter, ties still in row order. It is the stable sort of the kept letters alone. Let \\(v = L\[s\_0\]\\,L\[s\_1\]\\cdots\\) be the short column of kept letters. Then kept row \\(s\_j\\)'s arrow reaches entrance \\(e\_k\\) with

\\\[k = \\mathrm{LF}\_v(j),\\\]

where \\(\\mathrm{LF}\_v\\) is the LF rule applied to \\(v\\). This is the **restriction lemma**, and it is the key step. The cut arrows come from the kept letters, without reading any other row.

For `banana$`, the kept rows hold `a`, `$` and `a`, so \\(v\\) = `a$a`. Its stable sort puts `$` first, then the two `a`s in order, so \\(\\mathrm{LF}\_v = \[1, 0, 2\]\\). The table agrees. Row 0's arrow reaches row 1, which is entrance 1. Row 4's reaches entrance 0, and row 5's entrance 2. Here an entrance's rank and its row agree, by accident; the figure's lower half shows a kept set where they don't.

Restricting a stable sort to some rows gives those rows' own stable sort

Conceptual, on the worked example. Left: the column in row order, with each kept row's index *j*. Middle: the same rows sorted stably by letter; a row's arrow (LF) points to its place here. Right: the kept letters, sorted on their own. The green entries appear in the same order in both sorted lists, so the short column's LF gives each kept row's entrance. Below: with rows 0, 4 and 6 kept, the entrances are rows 0, 1 and 3. The short column is again `a$a`, so *k* is again \[1, 0, 2\], but *k* = 2 now means row 3: *k* is a rank among the entrances, and the rank is all the sort can give.

On its own, \\(\\mathrm{LF}\_v\\) has two loops here, 0 → 1 → 0 and 2 → 2. The short column is not a smaller BWT of the text; the open paths are what join its loops. Its stable sort is a counting sort, one pass over the kept letters. On the test block of section 5 it sorts 89,874 letters, against the block's 899,931 rows.

## 3\. Stitching the loop back together

Each kept row now has a whole trip to the next kept row: the cut arrow, then an open path. In indices, the cut arrow takes kept index \\(j\\) to entrance \\(\\mathrm{LF}\_v(j)\\). The path then takes that entrance to exit \\(P(\\mathrm{LF}\_v(j))\\). So the **first return** to the kept rows is

\\\[g(j) = P\\big(\\mathrm{LF}\_v(j)\\big).\\\]

On the way, the trip reads the kept row's own letter \\(v\[j\]\\), then the path's word.

The three trips of `banana$`. Each kept index \\(j\\) reaches entrance \\(k = \\mathrm{LF}\_v(j)\\) and then the next kept index \\(g(j)\\); the trip reads \\(v\[j\]\\) then the word \\(w\_k\\), in LF order, and the decoder writes it reversed:

| \\(j\\) | Kept row | \\(v\[j\]\\) | \\(k\\) | Word \\(w\_k\\) | \\(g(j)\\) | Written, \\(\\mathrm{rev}(w\_k)\\,v\[j\]\\) |
| ---: | ---: | :---: | ---: | :--- | ---: | :--- |
| 0 | 0 | `a` | 1 | `n` | 2 | `na` |
| 1 | 4 | `$` | 0 | none | 0 | `$` |
| 2 | 5 | `a` | 2 | `nab` | 1 | `bana` |

So \\(g = \[2, 0, 1\]\\): one loop through the three kept rows. LF reads the text backwards, so to write it forwards the decoder starts at the primary row's index, 1, and follows \\(g\\) backwards. Each step writes its trip in reverse, as in the table's last column: \\(\\mathrm{rev}(w\_k)\\), then \\(v\[j\]\\). Going back from 1 to 2 writes `bana`, from 2 to 0 `na`, and from 0 to 1 `$`. Together that is `banana$`. When the marked loop is shorter than the block, as for `abab`, the decoder writes this spelling and repeats it until it has written all \\(n\\) bytes.

Two jobs that never read each other's input, joined at the end

Conceptual, with the worked example's values in green. Job 1 needs only the rows that are not kept (here, the arrows left after the cut) and which rows are kept or entrances. Job 2 needs only the kept letters. After the spelling, the decoder finishes the block as usual: bzip2's final run-length decoding (RLE1) and the CRC check.

The same result, stated formally, with each line read on the example:

The open-return factorisation, stated formally and read on banana$

**1\. The cut.** Rows \\(0, \\dots, n-1\\), last column \\(L\\), and \\(f = \\mathrm{LF}\\). Choose a set \\(S\\) of kept rows containing the primary \\(p\\), and put \\(E = f(S)\\), the entrances. Number both in increasing order, \\(s\_0 \< \\dots \< s\_{m-1}\\) and \\(e\_0 \< \\dots \< e\_{m-1}\\), and let \\(v = L\[s\_0\] \\cdots L\[s\_{m-1}\]\\).

On banana$ \\(S = \\{0, 4, 5\\}\\), \\(E = \\{0, 1, 2\\}\\), \\(v\\) = `a$a`.

**2\. Two maps.** If \\(f(s\_j) = e\_k\\), then \\(k = \\mathrm{LF}\_v(j)\\) (the restriction lemma). With the arrows out of \\(S\\) deleted, each \\(e\_k\\) reaches a unique \\(s\_{P(k)}\\), reading the word \\(w\_k\\).

Sections 1 and 2 \\(\\mathrm{LF}\_v = \[1, 0, 2\]\\); \\(P = \[0, 2, 1\]\\), with words none, `n`, `nab`.

**3\. Factorisation.** The first return to \\(S\\) is \\(g(j) = P(\\mathrm{LF}\_v(j))\\), reading \\(v\[j\]\\,w\_k\\); its inverse writes \\(\\mathrm{rev}(w\_k)\\,v\[j\]\\). \\(P\\) and \\(w\\) depend only on the arrows outside \\(S\\) and on \\(S\\) and \\(E\\); \\(v\\) only on the kept letters.

Section 3 \\(g = \[2, 0, 1\]\\), writing `bana` · `na` · `$`.

Black: the statements and proof outline of the project's [proof](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/proof.md#restricting-a-stable-match>). Colour: the worked example. The proof holds for any column with a valid primary row, including columns whose LF has several loops, and for any kept set containing the primary.

**Proved, and checked twice.** The proof holds for any kept set containing the primary row, including columns whose LF has several loops. Two independent verifications rebuilt the construction from the proof alone, and compared it with a plain LF walk on thousands of small columns ([first](<https://github.com/lmmx/mibz2/blob/7747fddd11684e7959cb75f8de19a5b26138aef1/docs/journal/monodromy/2026-11-18-open-return-verification.md>), [second](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6978935386>); the [notes](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-18-cutting-the-loop-open-notes/#what-the-two-verifications-checked>) list what each checked).

## 4\. The same move in Markov-chain theory

Keeping only some states and recording the returns between them is an old operation. A Markov chain moves between states at random, and \\(M\_{xy}\\) is the probability of a step from \\(x\\) to \\(y\\). Split the states into the kept set \\(S\\) and the rest, \\(C\\), and order the matrix with \\(S\\) first:

\\\[M = \\begin{pmatrix} M\_{SS} &amp; M\_{SC} \\\\ M\_{CS} &amp; M\_{CC} \\end{pmatrix}.\\\]

Watch the chain only while it is in \\(S\\). From one kept state to the next, it either steps straight within \\(S\\), or it leaves, spends some number \\(a\\) of steps in \\(C\\), and comes back. Adding up every such excursion gives the watched chain's matrix:

\\\[M\_{SS} + M\_{SC}\\Big(\\sum\_{a \\ge 0} M\_{CC}^{\\,a}\\Big) M\_{CS}.\\\]

The term with \\(M\_{CC}^{\\,a}\\) covers returns that take \\(a\\) steps from \\(C\\) to \\(C\\) before coming back. Carl Meyer named this matrix the **stochastic complement** ([Meyer, 1989](<https://doi.org/10.1137/1031050>)); it is also called the trace chain. In general the sum is infinite. When the chain cannot stay in \\(C\\) for ever (every state of \\(C\\) is transient), it converges to \\((I - M\_{CC})^{-1}\\).

LF is a far simpler chain. It is a permutation, so each row of its matrix holds a single 1: every row has one next row, with certainty. Delete any loops lying wholly inside \\(C\\), as section 1 did. What remains of \\(M\_{CC}\\) is a set of disjoint paths, so a power of it is zero and the sum stops. Now write each arrow's letter in place of its 1, and multiply by joining words. The sum then gives each kept row its one return and that return's word. So the factorisation of section 3 is the stochastic complement of LF, with letters on the arrows (**proved**, and confirmed by both verifications).

The stochastic complement on banana$: each return is one term of the sum

Conceptual, on the worked example. Left: LF as a matrix, each 1 drawn as the letter its arrow reads, with the kept rows first. Right: the three returns, each picked out by one term of the complement. The block from C to C holds two arrows, 2 → 6 and 6 → 3. Its square is the two-step excursion 2 → 3, and its cube is zero. Railway reading: the complement is the timetable printed only for the termini, with each journey's intermediate stations folded into its word.

The formula says what the kept system is, not how to compute it fast: a dense matrix over 900,000 rows would be hopeless. The Markov view says what must result. LF's strips say how to remove rows cheaply, many at a time (section 5). The restriction lemma says how to reconnect them.

**How this relates to Cells** (an **interpretation** from the [first verification](<https://github.com/lmmx/mibz2/blob/7747fddd11684e7959cb75f8de19a5b26138aef1/docs/journal/monodromy/2026-11-18-open-return-verification.md>), not a theorem). Cells also computes first returns. It shrinks LF's interval exchange from the right-hand end (Rauzy–Veech induction), and each surviving interval stands for a longer word. One of the two ways we compute \\(P\\) (section 7) takes the same right-end steps, but only over the wide strips and the entrances and exits around them. On the frozen block of section 5 that is 7,786 intervals to begin with, where Cells begins with all 51,184 runs. The narrow runs never enter the induction, because the stable sort wires them. Read this way, cutting open splits the interval exchange by run width.

The two approaches side by side, as we read them:

|  | Cells | Cutting open |
| :--- | :--- | :--- |
| Where returns are taken | a shrinking interval, step by step | a chosen set of rows: here, the narrow runs |
| How returns are built | composed during the induction | open paths, solved apart from the kept rows' wiring |
| What is left at the end | one interval holding the text's word | \\(P\\), its words, and the short column \\(v\\) |
| What it relies on | interval lengths and joining words | stable sorting, open paths, rows moving together |

## 5\. How many pieces?

The construction is worth having only if the open paths can be solved many rows at a time. The runs make that possible. Keep the *narrow* runs, those of at most \\(t\\) rows (section 6 uses \\(t = 16\\)), and the primary row. The other rows then form \\(H\\) **wide strips**. A strip is a stretch of consecutive rows that LF shifts by one amount, each row reading the same byte. (It is a wide run, or half of one if the primary row splits it.)

**Why keep the short runs?** The project keeps one 899,931-row Wikidata block as a fixed test case, *the frozen block*. Of its 51,184 runs, 68.6% are a single row long, yet they hold only 3.9% of the rows. At the other end, runs longer than 256 rows are about 1.0% of the runs, yet hold 73.9% of the rows. These shares come from the cut at \\(t = 256\\), which leaves 496 wide strips. One of them may be half of a run that the primary row splits, and runs of exactly 256 rows fall on the kept side. So the short runs make most of the boundaries, and the long runs hold most of the text. Left on the open paths, every short run would add edges where the paths split apart. Kept, they go to the stable sort, which wires them in one pass. The open paths are left a few wide strips, which hold most of the rows.

Most runs are narrow, but most rows are in wide runs

Counted on the frozen block, at threshold 16: 89,874 kept rows (narrow runs, plus the primary row) and 3,448 wide strips out of 51,184 runs. The shares are derived from those counts; a strip split by the primary row counts twice, so the wide share of runs is at most 6.7%.

Two neighbouring entrances in the same strip are still neighbours after the step, since the strip shifts both by the same amount. They stay together, strip after strip, until a strip's edge falls between them, or they reach kept rows. A maximal set of neighbouring entrances that pass through the same strips and end at neighbouring exits is a **bundle**. The records call it a band. A bundle has a **width** \\(b\\), its number of rows, and a **height** \\(h\\), its number of steps. Tracing one of its paths costs \\(h\\) steps, and spells all \\(b \\times h\\) of its letters.

A bundle is traced once for all its rows, but a strip that moves a little makes bundles one row wide

Conceptual. Top: a bundle of width 4 and height 2 passing through two strips; in railway terms, a convoy on parallel tracks, surveyed by one train. Bottom: the proof's one-strip example, with *w* = 6 rows. When a strip moves by *d* rows with 0 \< |*d*| ≤ *w*, its paths have heights ⌊*w*/|*d*|⌋ and ⌈*w*/|*d*|⌉; when |*d*| ≥ *w*, each has height one. A shift by one row gives one tall path, a bundle one row wide with nothing to share, and a move clear of itself gives one short, wide bundle. The analogy fails at the edges: a bundle splits wherever a strip's edge or a kept row's falls between its rows.

Two totals describe a cut. The **bundled area** \\(A = \\sum\_i b\_i h\_i\\) is the number of letters the bundles spell. For a block whose LF is one loop, it is every row not kept, \\(n - m\\). The **traced letters** \\(U = \\sum\_i h\_i\\) are what tracing one path per bundle costs. The ratio \\(A/U\\) is how many rows travel together, on average, for each letter traced.

**The bound (proved).** There are at most \\(3H + \\sigma\\) bundles. The proof follows the gap between two neighbouring entrances. The gap moves with them through each strip. The two sides part only where the gap meets a strip's edge or the edge of a run of kept rows, and no two gaps meet the same edge first. The bound is attained on small columns ([notes](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-18-cutting-the-loop-open-notes/#the-bundle-bound>)). Real blocks sit far below it: the frozen block has 3,204 bundles, against a bound of 10,433.

**Few bundles is not little work.** The bound counts bundles, not their heights. In the figure's lower half, one strip shifted by one row is a single bundle, yet tracing it costs a step for every letter. So \\(U\\) can grow with the block, not with \\(H\\), and so can the endpoint steps of section 7. On real data, those are the figures to watch.

## 6\. What do real blocks look like after the cut?

**The threshold sets the balance.** Raising \\(t\\), the widest run kept, keeps more rows for the sort to wire, and leaves fewer, wider strips. On the frozen block that leaves less to trace at every threshold swept. That is not a rule. Raising \\(t\\) can cut open a loop the marked one never met, and so add paths to trace; the [notes](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-18-cutting-the-loop-open-notes/#one-strips-heights>) give a six-row case. The frozen block's counts, recomputed independently from the proof:

| \\(t\\) (rows) | Kept rows \\(m\\) | Wide strips \\(H\\) | Bundled area \\(A\\) | Traced letters \\(U\\) | Bundles (nonempty) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 35,116 | 16,068 | 864,815 | 555,357 | 12,526 (5,862) |
| 16 | 89,874 | 3,448 | 810,057 | 50,709 | 3,204 (1,326) |
| 256 | 235,281 | 496 | 664,650 | 4,658 | 738 (294) |

At \\(t = 16\\), \\(A/U\\) is 15.97: each traced letter spells about sixteen. On this block's sweep, up to \\(t = 16\\), the ratio rises roughly in step with \\(t\\). The median over many blocks at \\(t = 16\\) is 15.92. Yet section 6 shows two blocks of equal run density that differ widely.

Kept rows rise with the threshold while traced letters fall

Counted on the frozen block, at eleven thresholds. *m* is the rows the stable sort wires; *U* the letters traced, one path per bundle; *K* the endpoint constructor's steps (section 7). Their sum, dashed, is least at *t* = 16, at 151,798. The sum is our reading of the decoder's cost, not a measured time: it weights a sorted row, a traced letter and an endpoint step equally. Threshold 16 was chosen from timed trials on this block.

**All 233 large blocks.** Over the Wikidata blocks of at least 65,536 rows, at \\(t = 16\\), the median bundled area is 88.7% of the block and the median traced letters 5.5% (**counted**, and re-derived by a verification). Prose differs. Over its six blocks the wide strips cover a median of 32.3% of each block, from 27.6% to 49.0%. Most of a prose block is kept rows.

Run density says little about how much tracing a block needs

Counted, at *t* = 16, from the per-block records ([blocks.csv](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/evidence/blocks.csv>)). Each dot is one of the 233 Wikidata blocks of at least 65,536 rows; hover for its numbers. The rank correlation between the two axes is −0.36: denser blocks trace slightly less, not more. Blocks are numbered from zero in the benchmark file.

**Run density misses it.** Blocks 118 and 176 both have 900,000 rows, and run densities of 0.06348 and 0.06342. Yet block 118 needs 57,626 traced letters and block 176 needs 34,775: 66% more. Their bundled areas are nearly equal, and so are their numbers of wide strips. The difference is in how the bundles break up (**counted**). Block 118 has 53% more nonempty bundles, while their mean height is only 8% greater. One verification reads this as the narrow runs cutting across the wide strips' paths more often in block 118 (an **interpretation**). A run count can't see that. How the runs' images overlap can.

## 7\. What comes next?

**Two ways to compute the open paths.** The *tracer* follows one path per bundle and measures each bundle's width as it goes. The *endpoint constructor* never follows a path. It puts \\(m\\) terminal points in front of a copy of the rows, and joins them to the entrances and exits by arrows with empty words. Then it removes rows from the right-hand end. Removing a row joins the arrow into it and the arrow out of it into one arrow, with their words joined. The terminals are never removed, so when only they are left, each terminal's arrow is a whole path, with its word. That is \\(P\\).

The endpoint constructor removes rows from the right until only the paths' ends are left

Conceptual, on banana$, with every row not kept treated as wide. The decoder removes a common stretch of rows from the end of two strips in one step, where the figure removes one row; the joining is the same. Removing rows 5, 4, 3, 2, 1 and 0 in turn leaves terminal 2's arrow reading `nab` to terminal 1, terminal 1's reading `n` to terminal 2, and terminal 0's empty arrow to itself: *P* = \[0, 2, 1\].

The two constructors gave the same bundle counts and traced-letter counts on all 431 Wikidata blocks and the six prose blocks ([notes](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-18-cutting-the-loop-open-notes/#the-two-constructors-of-the-open-paths>)). The decoder below uses the endpoint constructor, and does the stable sort as it reads the runs, by queueing each kept row under its byte.

**The decoder so far (measured).** Its output has the same length and SHA-256 as libbz2's on the frozen block, the whole Wikidata benchmark and the prose. It rejects a damaged block CRC and a damaged stream CRC. The timings come from a four-CPU cloud virtual machine without SMT siblings. Each round decodes the Wikidata benchmark, 209,700,196 bytes of output, with eager Cells, lazy Cells (both as in mibz2 0.5.5) and the open decoder in turn. The times are medians in seconds, and the open decoder cuts at \\(t = 16\\). A ratio is the median over the rounds of that round's ratio, and no interval was computed.

| Workers (rounds) | Eager (s) | Lazy (s) | Open (s) | Open ÷ eager | Open ÷ lazy |
| :--- | ---: | ---: | ---: | ---: | ---: |
| calling thread (11) | 0.751 | 0.972 | 0.896 | 1.171 | 0.918 |
| 2 (9) | 0.459 | 0.583 | 0.543 | 1.211 | 0.931 |
| 4 (11) | 0.336 | 0.399 | 0.383 | 1.091 | 0.928 |

The open decoder beat lazy Cells at every worker count and lost to eager Cells at every one; on prose, on one thread, it took 1.455 times eager Cells' time. The figure below puts the wall times beside the two smaller measurements, each on its own scale: they have different units and cover different amounts of work.

Slower than eager Cells, faster than lazy, lighter on memory: three measurements, three scales

Measured on one four-CPU cloud virtual machine. Top: the paired wall-time ratios of the table above. Middle: the peak additional heap requested during one fresh decode of the frozen block. Bottom: the inverse transform alone, repeated on the frozen block, split into building the kept rows' table and writing the output; the switch cut is the harder cut described below. The three panels are not comparable with each other.

A cache simulation of the frozen block's decode counted 11% fewer instructions and 27% fewer first-level data-cache misses than eager Cells; those are simulated counts, not times. How much of the wall-time gap comes from building and walking the kept rows' table is not yet measured over the whole decode. Our reading, an **interpretation**, is that this table is the main cost eager Cells does not pay. The phase measurement shows its size on one block.

**A smaller kept system is not always cheaper to build.** A harder cut, the *switch cut*, keeps only the rows where a path switches between narrow and wide runs. On the frozen block its output phase took half as long as the decoder's above. But its build took 3.6 times as long, because finding the narrow side's words meant tracing 58,581 more letters. The smaller system was real, and this way of reaching it was slower.

**What comes next.** The open decoder holds less per block, and on many cores mibz2's speed has been limited by blocks competing for the shared cache ([Modelling the cache](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/index.md>)). The test that matters is therefore on a ten-core Intel desktop (an i9-10900K), at the worker counts where that competition starts, against the release that ships. A run that settles how the test machine's clock behaves under sustained load comes first. Whether the kept system could itself be cut open again is an open question.

The ten-core comparison is queued behind the sustained-load run; this post will be updated with what it finds.

**The chain, in three steps.** Cutting the arrows that leave the kept rows turns the marked loop into open paths, which the rest of the block determines alone. A stable sort of the kept letters rebuilds the cut arrows, and joining the two gives the exact return to the kept rows. On Wikidata the open paths travel in wide bundles, so tracing 5.5% of each block spells nearly 90% of it.

The general lesson is about where to cut. A chain of dependent steps can be split into paths and a reconnection when the reconnection has a cheap rule of its own, as the stable sort is here. The split is worth making when many starting points share the paths.

---

**Research records:**

- [The research journal](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-astra-investigation-2.md>), with [the proof](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/proof.md>), [the reproduction](<https://github.com/lmmx/mibz2/blob/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/README.md>) and [the measurement records](<https://github.com/lmmx/mibz2/tree/f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71/docs/journal/2026-11-18-open-return/evidence>), merged in [\#565](<https://github.com/lmmx/mibz2/pull/565>).
- The first independent verification: [its record](<https://github.com/lmmx/mibz2/blob/7747fddd11684e7959cb75f8de19a5b26138aef1/docs/journal/monodromy/2026-11-18-open-return-verification.md>) and [its review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6978817023>).
- The second independent verification: [its review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6978935386>) and [its report](<https://github.com/lmmx/mibz2/tree/432a6ce03857b21e57b03ff8d97094d0f96eff97/state/reports/open-return-6978698021>).
- C. D. Meyer, [Stochastic complementation, uncoupling Markov chains, and the theory of nearly reducible systems](<https://doi.org/10.1137/1031050>), *SIAM Review* 31 (1989), 240–272.
- Earlier posts: [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-10-rauzy-veech-reconstruction/index.md>), [the mathematical dictionary](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-16-mathematical-dictionary/index.md>), [One shift per run](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-08-inverse-bwt-navigation/index.md>) and [Lazy letters](<https://lmmx.github.io/mibz2/pr-preview/pr-701/blog/2026-11-16-lazy-letters/index.md>).
