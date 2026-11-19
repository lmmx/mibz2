[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/index.md>)

# What decoding pays for: how long run boundaries persist

Three ways of undoing bzip2's block sort spend about two operations per run, and how long run boundaries persist explains much of why Cells and the forest vary.

18 November 2026 · 16 min read · [Research result](<#6-where-things-stand>)

Over the past two days we built three new ways to undo the Burrows–Wheeler transform, and studied a fourth that describes the text by copies. The aim was a method that does less work than the kernel mibz2 ships. [A new way back from the BWT](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) introduced that kernel, Cells, and [Cutting the loop open](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-18-cutting-the-loop-open/index.md>) reported one of the new ones, open return, which cuts LF's loop at chosen rows and decodes along the open paths left. This post puts all of them side by side. It asks what work each one cannot avoid, and why that work is so similar from one construction to the next.

**The setting, in brief.** bzip2 stores each block of text as the last column \\(L\\) of the block's sorted rotations: the Burrows–Wheeler transform (BWT). A map called **LF** sends each row of the sorted table to the row of the rotation one byte earlier, so following LF reads the text backwards, one byte a step. A **run** is a stretch of equal bytes in \\(L\\). LF moves all the rows of a run by the same amount, so the whole map is a list of shifted runs ([One shift per run](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-08-inverse-bwt-navigation/index.md>)). We write \\(n\\) for a block's rows and \\(r\\) for its runs. A Wikidata block has about 900,000 rows and 50,000 runs. Cells works on runs, not rows. It shrinks the map one comparison at a time until a single row is left, recording how to spell the text as it goes. [The mathematical dictionary](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-16-mathematical-dictionary/index.md>) names the objects it keeps.

**The structural work** of a construction is the number of basic operations it performs on the runs before it writes any output. Every construction must also write the 900,000 output bytes. The structural work is the part where they differ by design.

**What we found.** We counted the structural work on all 233 large blocks of mibz2's Wikidata benchmark.

- **Cells and two of the new constructions, adjacent-rank induction and forest-section elimination ("the forest"), do about the same work:** 1.78 to 1.93 operations per run at the median. Their counts rise and fall together from block to block. Yet they do not perform the same operations, as a small example shows.
- **One property of the block explains much of the shared part.** Call the place between two neighbouring rows of a run a *gap*. Follow a gap's two rows through LF, and they read the same bytes until one of them crosses a run boundary. How long these pairs stay together, counted over the whole block, is the block's *boundary-orbit profile*. The persistence of boundary-orbit chains explains much of the variation in the structural work done by Cells and the forest. It explains adjacent-rank induction's count poorly.
- **The obvious ways to lower that work did not lower it where we tried them.** Refining the description of the block raised the forest's count on one block. Wikidata has almost no repeated steps to accelerate. Splitting one block across cores was only modelled, and the model does far more total work.
- **A description by copies of earlier text is far more compact,** about 11,000 copies for a 900,000-byte block, but it is written in text positions. The copy routes we tested had to get those positions by walking LF, the walk the other constructions avoid.
- **So Cells is the best-performing formulation tested.** Nobody has shown that it is optimal. The open question is whether a gap's separation depth, the number of bytes its two rows read together, can be found without walking its path.

Each figure is labelled as measured or proved, and each reading of it as interpretation. Sections 1 and 2 count the work and compare it. Section 3 builds the profile on a seven-row example. Sections 4 and 5 cover the routes that failed to do less, and section 6 says where things stand.

## 1\. How much work does each construction do?

Each construction counts a different kind of operation, so the fair comparison divides each count by the block's runs.

- **Cells** counts its *steps*. Each step compares the last stretch of rows in the runs' order with the last stretch in the order after LF, and trims the longer by the shorter. [The earlier post](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-10-rauzy-veech-reconstruction/#1-rebuilding-aab-in-two-steps>) works two steps by hand. Our *test block* is one block of the benchmark, with 899,931 rows and 51,184 runs; Cells takes 92,570 steps on it.
- **Adjacent-rank induction** cuts LF's loop at the rows that are smaller than both of their neighbours along the loop, joins the paths between them, and repeats on the shorter loop. Stretches of neighbouring rows that move together are its *atoms*. It counts its *atom visits*: 102,233 on the test block, over 12 rounds ([the first investigation](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-1.md#7-astras-final-result-adjacent-rank-gap-induction-draft-pr-553-head-79688e06>)).
- **Forest-section elimination** draws each run as a site. Each slice of a run that LF sends into a single run is an *arc*, and the test block has 62,138. It keeps a forest of arcs whose inside is fixed by their boundary, and computes the returns through the arcs left over. Then it repeats on that smaller system until one row is left. It counts the *arcs summed over its levels*: 90,945 ([the third investigation](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-3.md>)).

Over all 233 blocks, the three counts per run look like this:

Three constructions, about two operations per run

Measured: the structural counts of 233 large Wikidata blocks, each divided by the block's runs. The box is the middle half of the blocks, the whiskers the full range, and the triangle the test block. Population coefficients of variation: 4.27% (Cells), 2.66% (adjacent-rank induction) and 3.45% (forest). The units differ (steps, visits, arcs), so the near-equality is a finding, not a definition.

The medians are 1.837, 1.929 and 1.778 per run, and each count varies by only 3–4% across blocks ([the synthesis](<https://github.com/lmmx/mibz2/blob/d256502657463014f64820865093fb419e89b7a3/docs/journal/monodromy/2026-11-18-synthesis.md#2-the-shared-count>)). An [independent verification](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6983166783>) reproduced every figure from the three census files.

**Why the alternatives are slower anyway.** Equal counts do not mean equal time. Dividing each construction's time on the test block by its count gives about 17–28 ns per Cells step, 270 ns per adjacent-rank visit and 430 ns per forest arc. Cells is a tight loop over flat arrays; the other two use balanced trees, matching and stored recipes. Decoding the whole Wikidata stream on the calling thread of a four-CPU cloud virtual machine, against eager Cells (the form that lays out all its records before it starts):

| Construction | Full Wikidata stream, calling thread, against eager Cells |
| --- | ---: |
| Open return ([Cutting the loop open](<https://lmmx.github.io/mibz2/pr-preview/pr-617/blog/2026-11-18-cutting-the-loop-open/index.md>)) | 17.1% slower |
| Adjacent-rank induction ([its study](<https://github.com/lmmx/mibz2/blob/79688e06/docs/journal/2026-11-18-adjacent-rank-gaps.md#what-the-frozen-and-full-workloads-cost>)) | 11.5× slower (8.370 s against 0.729 s) |
| Forest-section elimination | 15.23× slower (median of paired rounds) |

With a pool of worker threads the gaps change. Open return is 21.1% slower with two workers and 9.1% slower with four. Adjacent-rank induction is 7.8× slower with four (2.682 s against 0.345 s). The per-operation figures are quotients of times by counts in different units. They say where the time went, not that cost per operation is the only difference. The forest, for one, also makes 48,647 slab matches on its first level that its arc count omits.

## 2\. Is it the same work?

The three medians are close. The stronger finding is that the counts move together from block to block: a block that is expensive for Cells is expensive for the forest too.

Cells and the forest rise and fall together, and the profile explains much of what they share

Measured, on 233 large Wikidata blocks. Top: Cells' steps per run against the forest's arcs per run. Bottom: the correlation between the two counts' fitted residuals, after removing, by linear regression, the quantities named on each bar. The second bar's term is the historical proxy for the excess (one less than the first historical depth count, per run), and the third bar's is the historical profile's tail, counted with row 0; both are explained in section 3. The per-block points were joined from the Cells census and the forest census; the third and fourth bars come from [the profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md#2-does-the-profile-explain-the-three-counts-for-cells-and-the-forest-yes-c>), whose per-block profile values are not published.

The Cells–forest Pearson correlation is 0.906. Part of it could be run density, \\(r/n\\), which differs from block to block. Removing \\(r/n\\) by regression leaves 0.859, so most of the agreement is about something else in the block. The agreement is weaker for adjacent-rank induction: 0.74 with the forest and 0.68 with Cells.

**Not the same events.** A tempting reading is that the constructions perform the same operations in different clothes. An independent verification refuted that on a 13-byte column, `1111000011100`. The forest has a free choice when two arcs have equal width. Reversing only those ties leaves the input, the starting network and the first level's output the same, yet the forest's total goes from 12 arcs to 15. Cells has no such choice, and its count is unchanged. So the shared count is not an event-by-event match. It is shared dependence on something in the block, and the next section names it.

## 3\. What the profile measures

Take the seven rotations of `banana$`, sorted, with their last bytes. Its last column `annb$aa` has five runs: `a`, `nn`, `b`, `$` and `aa`.

Two neighbouring rows read the same bytes until a run boundary separates them

Conceptual, on the worked example. Rows 1 and 2 are neighbours inside the run `nn`. LF keeps neighbours in one run as neighbours, so the pair moves together until it straddles a run boundary. Read the chain the other way, from the boundary between rows 2 and 3: it pulls back to the gap between rows 5 and 6, then to the gap between rows 1 and 2, and stops there, because the rows LF sends to 1 and 2 (rows 0 and 5) are not neighbours. The profile counts four gaps at depth 0 and one each at depths 1 and 2.

Why do the two rows stay together? Inside a run, LF sends neighbouring rows to neighbouring rows: both end in the same byte, and LF ranks rows with equal last bytes in their original order. So the pair reads the same byte, moves one step, and if it is still inside a run, does so again. It parts only when it lands on a **run boundary**. The number of bytes the pair reads together is its gap's **separation depth**. In `banana$`, the gap between rows 1 and 2 has depth 2 and the gap between rows 5 and 6 has depth 1. The four gaps that are run boundaries have depth 0.

**The boundary-orbit profile** counts the gaps by depth. Let \\(q\_k\\) be the number of gaps between neighbouring rows whose rows read exactly \\(k\\) bytes together. For `banana$`, \\(q\_0 = 4\\), \\(q\_1 = 1\\) and \\(q\_2 = 1\\).

**Why it counts chains.** Read backwards, as in the figure, each run boundary starts a chain of gaps. In `banana$`, the boundary between rows 2 and 3 leads to the gap between rows 5 and 6, and then to the gap between rows 1 and 2. A chain that lasts \\(\\ell\\) steps has exactly one gap at each depth \\(0, 1, \\dots, \\ell\\). So \\(q\_k\\) is also the number of chains that last at least \\(k\\) steps. In `banana$` one chain reaches depth 2 and the other three stop at once, which is why \\(q\_1 = q\_2 = 1\\). The profile records how long the run boundaries persist when pulled back through LF. This is **proved**, and checked on all 233 blocks ([the chain record](<https://github.com/lmmx/mibz2/blob/2e31341f46e5c44d7398ebfa2dd7babb3bc35af7/docs/journal/monodromy/2026-11-18-q-lcp.md#1-the-exact-correspondence>)).

**Why the short tail is the part that matters.** Added over every depth from 1 up, the profile is the total length of the chains, and that is always \\(n - r\\): in `banana$`, \\(q\_1 + q\_2 = 2 = 7 - 5\\). This holds for any block that is not one word repeated, which covers all 233. (In a repeated word such as `abab`, some gaps lie on loops and belong to no chain.) So the total alone cannot tell apart two blocks with the same rows and runs, though the whole profile can. What separates them is how early their chains die. The **short tail**, \\(\\sum\_{k=2}^{8} q\_k\\), counts one gap for each chain still alive at each depth from 2 to 8. In `banana$` it is 1: the single chain that outlives its first step.

**How the profile meets the forest.** The start of the profile is already part of the forest's network. Overlay two ways of cutting the rows into pieces: at the run boundaries, and at the boundaries of the runs' images under LF. Each has \\(r - 1\\) internal cuts. If \\(C\\) of the cuts coincide, the overlay has

\\\[E\_0 = 2r - 1 - C\\\]

pieces, and those pieces are exactly the forest's arcs. This is **proved**, and it holds on all 233 blocks. The forest's *excess* is \\(E\_0 - V\_0\\), its arcs minus its sites (\\(V\_0 = r\\)). A gap has depth 1 when LF carries it onto a run boundary, and the run boundaries reached that way are the \\(r - 1 - C\\) that are not also image cuts. So

\\\[q\_1 = r - 1 - C = E\_0 - r,\\\]

the excess. The forest's leftover arcs, its *chords*, number one more, \\(E\_0 - r + 1\\). In `banana$`, \\(C = 3\\) and \\(E\_0 = 6\\), so \\(q\_1 = 1\\) and the forest has 2 chords.

**The profile test's convention.** The profile test that the fits below come from counted one point more. Row 0 starts a run but sits at no gap, and the test added it at the depth \\(d\_0\\) where row 0's own path first lands on a run boundary. We write that *historical* profile as \\(\\tilde q\_k = q\_k + \[k = d\_0\]\\). On 229 blocks \\(d\_0 = 1\\), so \\(\\tilde q\_1\\) equals the chord count: 10,955 on the test block, against \\(q\_1 = 10{,}954\\). On the other four, \\(d\_0 = 2\\) and the extra point falls in the tail. The three-byte column `100` shows the same case: there \\(\\tilde q\_1 = q\_1 = 1\\), while the forest has 2 chords. So the historical \\(\\tilde q\_1\\) matches the chords only when \\(d\_0 = 1\\), not by a rule ([the check](<https://github.com/lmmx/mibz2/blob/2e31341f46e5c44d7398ebfa2dd7babb3bc35af7/docs/journal/monodromy/2026-11-18-q-lcp.md#1-the-exact-correspondence>)). On the test block, \\(\\tilde q\_2\\) to \\(\\tilde q\_5\\) equal exactly the excesses of the refined networks in section 4.

On the real blocks the profile decays slowly. The median of \\(\\tilde q\_k/r\\) over the 233 blocks falls only from 0.223 at \\(k = 1\\) to 0.101 at \\(k = 8\\) ([the profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md#1-the-identification-verified-p-c>)). After eight steps, there is still about one chain running for every ten runs.

**The test.** It compared the short-tail sum with the depths fitted one by one. Each count per run was fitted by least squares, with leave-one-out error, using the historical profile:

| Model | Cells \\(R^2\\) | Forest \\(R^2\\) | Adj.-rank \\(R^2\\) | Residual correlation |
| --- | ---: | ---: | ---: | ---: |
| \\(r/n\\) and the excess proxy \\((\\tilde q\_1 - 1)/r\\) | 0.825 | 0.887 | 0.529 | 0.393 |
| the same, plus \\(\\sum\_{k=2}^{8} \\tilde q\_k/r\\) | 0.925 | 0.923 | 0.557 | −0.061 |
| \\(r/n\\) and \\(\\tilde q\_1, \\dots, \\tilde q\_8\\) separately | 0.946 | 0.959 | 0.570 | 0.307 |

**From one chain to the fit.** The first model already holds run density and the historical proxy for the excess, \\((\\tilde q\_1 - 1)/r\\). The exact excess \\(q\_1\\) counts the chains reaching at least one step. On the 229 blocks with \\(d\_0 = 1\\) the proxy equals the exact excess per run, \\(q\_1/r\\), and on the other four it is lower by \\(1/r\\). The first-depth count includes longer chains, but it cannot tell how long they last. What it cannot see is chains like the one in `banana$` that outlive that step, and that is where Cells and the forest still agree. The last column is the correlation between the two counts' fitted residuals, the part of each that the model leaves unexplained. It is 0.393 under the first model. Adding the tail, the chains that persist, takes it to −0.061: about zero. As a separate check, the leave-one-out error falls by 34% for Cells and 17% for the forest. Fitting all eight depths separately fits each count more closely, but leaves 0.307 in common, so the single tail sum accounts for more of what the two share. Adjacent-rank induction stays poorly explained under every model, at \\(R^2 \\le 0.570\\). Its rounds are chosen by local minima of the rows' order rather than by run boundaries, which may be why some other property drives it.

These fits use the historical profile as recorded. On the four blocks with \\(d\_0 = 2\\) its tail holds row 0's extra point, and the fit's excess term, taken as \\((\\tilde q\_1 - 1)/r\\), is one gap low. A fit on the internal-gap profile alone has not been run.

So, **measured:** the persistence of boundary-orbit chains explains much of the variation in the structural work done by Cells and the forest. **Our interpretation**, not a proof: both pay roughly once for each point on a boundary chain until the chains collide. Cells meets those points in order, and the forest by levels. The profile is fixed by the column \\(L\\) before any construction runs. It is not an exact accounting of either count, and it gives no lower bound on the work of other constructions.

## 4\. Why not do less of it?

If the work follows the boundary chains, there are three obvious ways to try to reduce it. None lowered it where we tried it, on evidence of three kinds: a count on the test block, a measurement on Wikidata, and a model.

**Refine the presentation.** The forest pays for its excess, so describe the block with smaller pieces that have less excess. At depth \\(k\\), a piece is a stretch of neighbouring rows that pass through the same runs in their first \\(k\\) steps of LF; at depth 1 the pieces are the runs themselves. The excess does fall, from \\(0.21\\,r\\) to \\(0.07\\,r\\). The total work rises:

Smaller pieces lower the excess, and raise the work

Measured on the test block, from the first refinement record: the forest's arcs summed over all its levels (amber), and its first-level excess (steel, dashed), both in units of the block's runs, at depths 1, 2, 3, 4, 5, 7, 10, 16 and 32. The horizontal axis is logarithmic.

The arcs summed over levels go from 90,945 at depth 1 to 208,766 at depth 32. Part of the reason is **proved**. Refining by one more step turns every arc of the old network into a site of the new one, \\(V\_0(k+1) = E\_0(k)\\). So the sites grow by the old excess, and the first level's arcs by the new one. That the later levels shrink by less is **counted** at the nine depths tried, not proved ([record of the refinement test](<https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-o3-o5.md#1-o3-a-smaller-excess-presentation-decisive-and-negative>)). Coarser pieces fail the other way, with an excess of about \\(r\\). So among the presentations tried, runs are best. That is a finding about this block, not a theorem: on the three-byte column `100`, refining lowers the total from 5 arcs to 3.

**Accelerate repeated steps.** When a stretch of the map repeats, one division can stand in for many identical steps. Adjacent-rank induction does this. On an artificial text, \\(abcd\\) repeated with an end marker, whose column has five runs, it decodes 875,001 bytes about 9.5 times faster than mibz2's shipped route: 0.161 ms against 1.526 ms, over five runs. That route sends so sparse a column to its Segmented decoder, not to Cells. Wikidata has almost nothing to accelerate. Division compresses only repeated passes through one atom, and Cells' census on the test block finds no repeated steps. On the full benchmark adjacent-rank induction is the slower of the two (section 1).

**Split a block across cores.** Cells is one sequential chain of 92,570 steps on the test block: each step reads the state the previous one left. Adjacent-rank induction is more parallel, in a model. A row-by-row model reproduces its 12 rounds exactly, and their combined longest path is 87 row steps: the classical parallel route of contracting a list at an independent set of its points. But the model executes 1,337,701 row steps, \\(1.49\\,n\\), against Cells' 92,570 steps, about \\(0.1\\,n\\). The real decoder routes atoms, not rows, and how long that routing runs in sequence was not measured. No threaded version was built. Meanwhile, one block per thread already fills a ten-core Intel desktop (an i9-10900K), which is limited by memory contention and power at seven to ten busy cores. So the model trades much more work for a shorter path, with no speed-up shown on the test machine. Our reading is that giving another core another block remains the better use of it.

## 5\. Why not copy earlier text?

The fourth route leaves the run network behind. Much of a block repeats earlier text, so describe the text as literal bytes plus *copies*, each a pointer back to an earlier stretch of output and a length. This is the idea behind LZ77-style compressors, and decoding a copy is a plain memory copy.

**It is compact.** Seed it with the 51,184 run-head bytes in text order, and a greedy parse covers the rest of the test block with 11,145 copies and 33 single literal bytes. Replaying them is estimated, not timed, at 0.1–0.3 ms a block. The inverse BWT it is set against, the control in that assessment, took about 2.4 ms on the same block in the original investigation; it was not timed again.

**It needs text positions.** The short parse needs its seed bytes in *text* order, and the BWT gives its rows in *sorted* order. With the same seed bytes in sorted order, the parse needs 18,132 copies. The BWT does carry a copy description of its own: neighbouring rows of a run read the same bytes (section 3), so each one can copy from its neighbour. Grouped into phrases, this is a known scheme (Navarro, Ochoa and Prezza, Theorem 7). On the test block it has 51,184 literal bytes and 10,954 copies. But 5,611 of its copies have sources that overlap them or come later in the text. An evaluator that sweeps front to back needs 12,406 sweeps. One that follows the dependencies could order them differently, and was not tried. Putting the copies in text order needs each row's place in the text. The routes we tested got those places by walking LF. Whether list ranking or another method could recover them more cheaply is open. The obstacle here is a different one from the run network's ([the copy-route assessment](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982429070>); [its independent reproduction](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982850361>)).

All four routes, side by side:

What each route avoids, and what it still pays

A summary of sections 1, 4 and 5. Counts are medians per run over 233 blocks; times are full-stream slowdowns against eager Cells on a four-CPU cloud virtual machine, on the calling thread unless a worker count is named; the copy route's replay is an estimate. Open return, reported in its own post, avoids induction on the narrow runs and is 17.1% slower than eager Cells on the calling thread, 9–21% across the calling thread and two or four workers.

## 6\. Where things stand

**Measured and proved.** Three unlike constructions do about 1.8 operations per run on Wikidata, their counts correlate strongly, and they do not process the same events. The identity \\(E\_0 = 2r - 1 - C\\) is proved and holds on every block, and the depth-one count \\(q\_1 = E\_0 - r\\) is the forest's excess, one less than its chords. The short tail of the profile, as historically counted, explains much of the variation Cells and the forest share. Refinement raised the work on the test block, acceleration has nothing to act on in Wikidata, and in a model, splitting a block costs far more total work; no threaded version was tested. The copy route is compact, but the routes tested had to walk LF for its text positions.

**Not shown.** No one has shown that Cells is optimal. The shared count is not a lower bound, and the only proved bounds are the trivial ones: reading the \\(r\\) runs and writing the \\(n\\) bytes. Cells is the best-performing formulation tested.

**The open question.** On our reading, Cells and the forest pay for each boundary chain step by step. Some shortcuts exist. Division compresses repeated passes through one atom, and a one-bit test tells when two neighbouring paths spell the same word. But among the methods we measured or costed, learning how long an *aperiodic* chain runs meant walking it, or pointer jumping after preprocessing of order \\(n \\log n\\). Can a gap's separation depth be found without walking its path? A method that could would attack the work Cells and the forest share.

**Next.** Two studies are under way: a bounded study of whether steps of different kinds in Cells can be fused into one, and a literature study of these gap chains.

In short: three constructions count about the same work, and much of what they share is how long the run boundaries persist. The ways tried to shorten that work raised it, found nothing to act on, or, in a model, multiplied it. When unlike methods agree on a count, the reason is worth looking for in the input before tuning the methods.

---

**Research records:** [the synthesis of the four investigations](<https://github.com/lmmx/mibz2/blob/d256502657463014f64820865093fb419e89b7a3/docs/journal/monodromy/2026-11-18-synthesis.md>) · [refinement and parallel depth](<https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-o3-o5.md>) · [the boundary-orbit profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md>) · [the profile's chains and conventions](<https://github.com/lmmx/mibz2/blob/2e31341f46e5c44d7398ebfa2dd7babb3bc35af7/docs/journal/monodromy/2026-11-18-q-lcp.md>) · [the adjacent-rank study](<https://github.com/lmmx/mibz2/blob/79688e06/docs/journal/2026-11-18-adjacent-rank-gaps.md>) · [the forest's verification](<https://github.com/lmmx/mibz2/blob/eda3167badcbd82be5ebe47607ae4b4528ed030d/docs/journal/monodromy/2026-11-18-forest-verification.md>) · [the independent synthesis review](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6983166783>) and [its report](<https://github.com/lmmx/mibz2/blob/57b4a9403b1e26f602258248c1f088783156ec06/state/reports/synthesis-6982863130/report.md>) · [the forest review](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982103916>) · investigations [one](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-1.md>), [two](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-2.md>) and [three](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-3.md>) · [the copy route, recovered](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-pro-investigation-recovered.md>) · [the Cells census](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-10-cells-gap/evidence/i9-census-1.jsonl>) · [the forest census](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-switch-elimination/evidence/forest-wikidata.csv>)
