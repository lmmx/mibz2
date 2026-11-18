[mibz2 · Research blog](<https://lmmx.github.io/mibz2/blog/index.md>)

# What decoding pays for: how long run boundaries persist

Three very different ways of undoing bzip2's block sort spend about two operations per run, and one property of each block explains why they rise and fall together.

18 November 2026 · 13 min read · [Research result](<#6-where-things-stand>)

Over the past two days we built three new ways to undo the Burrows–Wheeler transform, and studied a fourth that describes the text by copies. The aim was a method that does less work than the kernel mibz2 ships. [A new way back from the BWT](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/index.md>) introduced that kernel, Cells, and [Cutting the loop open](<https://lmmx.github.io/mibz2/blog/2026-11-18-cutting-the-loop-open/index.md>) reported one of the new ones, open return, which cuts LF's loop at chosen rows and decodes along the open paths left. This post puts all of them side by side. It asks what work each one cannot avoid, and why that work is so similar from one construction to the next.

**The setting, in brief.** bzip2 stores each block of text as the last column \\(L\\) of the block's sorted rotations: the Burrows–Wheeler transform (BWT). A map called **LF** sends each row of the sorted table to the row of the rotation one byte earlier, so following LF reads the text backwards, one byte a step. A **run** is a stretch of equal bytes in \\(L\\). LF moves all the rows of a run by the same amount, so the whole map is a list of shifted runs ([One shift per run](<https://lmmx.github.io/mibz2/blog/2026-11-08-inverse-bwt-navigation/index.md>)). We write \\(n\\) for a block's rows and \\(r\\) for its runs. A Wikidata block has about 900,000 rows and 50,000 runs. Cells works on runs, not rows. It shrinks the map one comparison at a time until a single row is left, recording how to spell the text as it goes. [The mathematical dictionary](<https://lmmx.github.io/mibz2/blog/2026-11-16-mathematical-dictionary/index.md>) names the objects it keeps.

**The structural work** of a construction is the number of basic operations it performs on the runs before it writes any output. Every construction must also write the 900,000 output bytes. The structural work is the part where they differ by design.

**What we found.** We counted the structural work on all 233 large blocks of mibz2's Wikidata benchmark.

- **Cells and two of the new constructions, adjacent-rank induction and forest-section elimination ("the forest"), do about the same work:** 1.78 to 1.93 operations per run at the median. Their counts rise and fall together from block to block. Yet they do not perform the same operations, as a small example shows.
- **One property of the block explains much of the shared part.** Call the place between two neighbouring rows of a run a *gap*. Follow a gap's two rows through LF, and they read the same bytes until one of them crosses a run boundary. How long these pairs stay together, counted over the whole block, is the block's *boundary-orbit profile*. The persistence of boundary-orbit chains explains much of the variation in the structural work done by Cells and the forest.
- **The obvious ways to lower that work have each failed:** refining the description of the block, accelerating repeated steps, and splitting one block across cores. Wikidata has almost no repeats to accelerate.
- **A description by copies of earlier text is far more compact,** about 11,000 copies for a 900,000-byte block, but it is written in text positions, and the BWT gives text positions only by walking LF.
- **So Cells is the best-performing formulation tested.** Nobody has shown that it is optimal. The open question is whether a gap's separation depth, the number of bytes its two rows read together, can be found without walking its path.

Each figure is labelled as measured or proved, and each reading of it as interpretation. Sections 1 and 2 count the work and compare it. Section 3 builds the profile on a seven-row example. Sections 4 and 5 cover the routes that failed to do less, and section 6 says where things stand.

## 1\. How much work does each construction do?

Each construction counts a different kind of operation, so the fair comparison divides each count by the block's runs.

- **Cells** counts its *steps*. Each step compares the last stretch of rows in the runs' order with the last stretch in the order after LF, and trims the longer by the shorter. [The earlier post](<https://lmmx.github.io/mibz2/blog/2026-11-10-rauzy-veech-reconstruction/#1-rebuilding-aab-in-two-steps>) works two steps by hand. Our *test block* is one block of the benchmark, with 899,931 rows and 51,184 runs; Cells takes 92,570 steps on it.
- **Adjacent-rank induction** cuts LF's loop at the rows that are smaller than both of their neighbours along the loop, joins the paths between them, and repeats on the shorter loop. Stretches of neighbouring rows that move together are its *atoms*. It counts its *atom visits*: 102,233 on the test block, over 12 rounds ([the first investigation](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-1.md#7-astras-final-result-adjacent-rank-gap-induction-draft-pr-553-head-79688e06>)).
- **Forest-section elimination** draws each run as a site. Each slice of a run that LF sends into a single run is an *arc*, and the test block has 62,138. It keeps a forest of arcs whose inside is fixed by their boundary, and computes the returns through the arcs left over. Then it repeats on that smaller system until one row is left. It counts the *arcs summed over its levels*: 90,945 ([the third investigation](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-3.md>)).

Over all 233 blocks, the three counts per run look like this:

Three constructions, about two operations per run

Measured: the structural counts of 233 large Wikidata blocks, each divided by the block's runs. The box is the middle half of the blocks, the whiskers the full range, and the triangle the test block. Population coefficients of variation: 4.27% (Cells), 2.66% (adjacent-rank induction) and 3.45% (forest). The units differ (steps, visits, arcs), so the near-equality is a finding, not a definition.

The medians are 1.837, 1.929 and 1.778 per run, and each count varies by only 3–4% across blocks ([the synthesis](<https://github.com/lmmx/mibz2/blob/d256502657463014f64820865093fb419e89b7a3/docs/journal/monodromy/2026-11-18-synthesis.md#2-the-shared-count>)). An [independent verification](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6983166783>) reproduced every figure from the three census files.

**Why the alternatives are slower anyway.** Equal counts do not mean equal time. Cells is a tight loop over flat arrays, at about 17–28 ns a step on the test block. Adjacent-rank induction spends about 270 ns a visit and the forest about 430 ns an arc, on balanced trees, matching and stored recipes. Measured on one thread of a four-CPU cloud virtual machine, decoding the whole Wikidata stream, against eager Cells (the form that lays out all its records before it starts):

| Construction | Full Wikidata stream, against eager Cells |
| --- | ---: |
| Open return ([Cutting the loop open](<https://lmmx.github.io/mibz2/blog/2026-11-18-cutting-the-loop-open/index.md>)) | 9–21% slower |
| Adjacent-rank induction | 11× slower (8.37 s against 0.729 s) |
| Forest-section elimination | 15.23× slower (median of paired rounds) |

These ratios divide times by different units, so they describe where the time went; they do not prove that cost per operation is the only difference. The forest, for one, also makes 48,647 slab matches on its first level that its arc count omits.

## 2\. Is it the same work?

The three medians are close. The stronger finding is that the counts move together from block to block: a block that is expensive for Cells is expensive for the forest too.

Cells and the forest rise and fall together, and the profile accounts for what they share

Measured, on 233 large Wikidata blocks. Top: Cells' steps per run against the forest's arcs per run. Bottom: the Pearson correlation between the two that remains after removing, by linear regression, the quantities named on each bar. The second bar's excess and the third bar's profile are explained in section 3. The per-block points were joined from the Cells census and the forest census; the third and fourth bars come from [the profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md#2-does-the-profile-explain-the-three-counts-for-cells-and-the-forest-yes-c>), whose per-block profile values are not published.

The Cells–forest Pearson correlation is 0.906. Part of it could be run density, \\(r/n\\), which differs from block to block. Removing \\(r/n\\) by regression leaves 0.859, so most of the agreement is about something else in the block. The agreement is weaker for adjacent-rank induction: 0.74 with the forest and 0.68 with Cells.

**Not the same events.** A tempting reading is that the constructions perform the same operations in different clothes. An independent verification refuted that on a 13-byte column, `1111000011100`. The forest has a free choice when two arcs have equal width. Reversing only those ties leaves the input, the starting network and the first level's output the same, yet the forest's total goes from 12 arcs to 15. Cells has no such choice, and its count is unchanged. So the shared count is not an event-by-event match. It is shared dependence on something in the block, and the next section names it.

## 3\. What the profile measures

Take the seven rotations of `banana$`, sorted, with their last bytes. Its last column `annb$aa` has five runs: `a`, `nn`, `b`, `$` and `aa`.

Two neighbouring rows read the same bytes until a run boundary separates them

Conceptual, on the worked example. Rows 1 and 2 are neighbours inside the run `nn`. LF keeps neighbours in one run as neighbours, so the pair moves together until it straddles a run boundary. Read the chain the other way, from the boundary between rows 2 and 3: it pulls back to the gap between rows 5 and 6, then to the gap between rows 1 and 2, and stops there, because the rows LF sends to 1 and 2 (rows 0 and 5) are not neighbours.

Why do the two rows stay together? Inside a run, LF sends neighbouring rows to neighbouring rows: both end in the same byte, and LF ranks rows with equal last bytes in their original order. So the pair reads the same byte, moves one step, and if it is still inside a run, does so again. It parts only when it lands on a **run boundary**. The number of bytes the pair reads together is its gap's **separation depth**. In `banana$`, the gap between rows 1 and 2 has depth 2 and the gap between rows 5 and 6 has depth 1. The four gaps that are run boundaries have depth 0.

**The boundary-orbit profile** counts the gaps by depth. Let \\(q\_k\\) be the number of gaps whose rows read exactly \\(k\\) bytes together. For `banana$`, \\(q\_1 = 1\\) and \\(q\_2 = 1\\). Read backwards, as in the figure, each run boundary starts a chain of gaps. So \\(q\_k\\) also counts the chains still going \\(k\\) steps back: how long the run boundaries persist when pulled back through LF.

On the real blocks the profile decays slowly. The median of \\(q\_k/r\\) over the 233 blocks falls only from 0.223 at \\(k = 1\\) to 0.101 at \\(k = 8\\) ([the profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md#1-the-identification-verified-p-c>)). After eight steps, there is still about one chain running for every ten runs.

**How the profile meets the forest.** The first term of the profile is already part of the forest's network. Overlay two ways of cutting the rows into pieces: at the run boundaries, and at the boundaries of the runs' images under LF. Each has \\(r - 1\\) internal cuts. If \\(C\\) of the cuts coincide, the overlay has

\\\[E\_0 = 2r - 1 - C\\\]

pieces, and those pieces are exactly the forest's arcs. This is **proved**, and it holds on all 233 blocks. The forest's *excess* is \\(E\_0 - V\_0\\), its arcs minus its sites (\\(V\_0 = r\\)). One more identity, verified on every block, ties the profile to it:

\\\[q\_1 = r - C = E\_0 - V\_0 + 1,\\\]

which is the forest's count of leftover arcs, its *chords*: 10,955 on the test block. The \\(+1\\) is row 0, a run start that is not an internal cut. On the test block, \\(q\_2\\) to \\(q\_5\\) also equal exactly the excesses of the refined networks in section 4.

**The test.** The test added a single summary quantity, rather than searching over combinations: \\(\\sum\_{k=2}^{8} q\_k / r\\), the tail of the profile. Each count per run was fitted by least squares, with leave-one-out error:

| Model | Cells \\(R^2\\) | Forest \\(R^2\\) | Adj.-rank \\(R^2\\) | Shared residual |
| --- | ---: | ---: | ---: | ---: |
| \\(r/n\\) and the excess \\((E\_0 - V\_0)/r\\) | 0.825 | 0.887 | 0.529 | 0.393 |
| the same, plus \\(\\sum\_{k=2}^{8} q\_k/r\\) | 0.925 | 0.923 | 0.557 | −0.061 |
| \\(r/n\\) and \\(q\_1, \\dots, q\_8\\) separately | 0.946 | 0.959 | 0.570 | 0.307 |

With the tail added, the leave-one-out error falls by 34% for Cells and 17% for the forest. The correlation left between them, the table's last column, goes from 0.393 to −0.061: about zero. Fitting all eight terms separately fits each count more closely, but leaves 0.307 in common. So the simple summary fixed in advance accounts for more of what the two share. Adjacent-rank induction stays poorly explained under every model, at \\(R^2 \\le 0.57\\). Its rounds are chosen by local minima of the rows' order rather than by run boundaries, which may be why some other property drives it.

So, **measured:** the persistence of boundary-orbit chains explains much of the variation in the structural work done by Cells and the forest. **Our interpretation**, not a proof: both pay roughly once for each point on a boundary chain until the chains collide. Cells meets those points in order, and the forest by levels. The profile is fixed by the column \\(L\\) before any construction runs. It is not an exact accounting of either count, and it gives no lower bound on the work of other constructions.

## 4\. Why not do less of it?

If the work follows the boundary chains, there are three obvious ways to try to reduce it. Each has been tested on the test block, and each failed.

**Refine the presentation.** The forest pays for its excess, so describe the block with smaller pieces that have less excess. At depth \\(k\\), a piece is a stretch of neighbouring rows that pass through the same runs in their first \\(k\\) steps of LF; at depth 1 the pieces are the runs themselves. The excess does fall, from \\(0.21\\,r\\) to \\(0.07\\,r\\). The total work rises:

Smaller pieces lower the excess, and raise the work

Measured on the test block, from the first refinement record: the forest's arcs summed over all its levels (amber), and its first-level excess (steel, dashed), both in units of the block's runs, at depths 1, 2, 3, 4, 5, 7, 10, 16 and 32. The horizontal axis is logarithmic.

The arcs summed over levels go from 90,945 at depth 1 to 208,766 at depth 32. The reason is **proved**. Refining by one more step turns every arc of the old network into a site of the new one, \\(V\_0(k+1) = E\_0(k)\\). So the first level grows by the old excess each time, faster than the later levels shrink ([record of the refinement test](<https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-o3-o5.md#1-o3-a-smaller-excess-presentation-decisive-and-negative>)). Coarser pieces fail the other way, with an excess of about \\(r\\). So among the presentations tried, runs are best. That is a finding about this block, not a theorem: on the three-byte column `100`, refining lowers the total from 5 arcs to 3.

**Accelerate repeated steps.** When a stretch of the map repeats, one division can stand in for many identical steps. Adjacent-rank induction does this. On an artificial text, \\(abcd\\) repeated with an end marker, whose column has five runs, it decodes 875,001 bytes about 9.5 times faster than Cells. Wikidata has almost nothing to accelerate. Cells' census on the test block finds no repeated steps, and on the full benchmark adjacent-rank induction is the slower of the two (section 1).

**Split a block across cores.** Cells is one sequential chain of 92,570 steps on the test block: each step reads the state the previous one left. Adjacent-rank induction is more parallel. Its 12 rounds have a combined longest path of only 87 steps, the classical parallel route of contracting a list at an independent set of its points. But followed row by row, it does about 1.34 million row-steps, \\(1.49\\,n\\), against Cells' structural work of about \\(0.1\\,n\\). Meanwhile, one block per thread already fills a ten-core Intel desktop (an i9-10900K), which is limited by memory contention and power at seven to ten busy cores. Our reading is that neither form of splitting beats giving another core another block.

## 5\. Why not copy earlier text?

The fourth route leaves the run network behind. Much of a block repeats earlier text, so describe the text as literal bytes plus *copies*, each a pointer back to an earlier stretch of output and a length. This is the idea behind LZ77-style compressors, and decoding a copy is a plain memory copy.

**It is compact.** Seed it with the 51,184 run-head bytes in text order, and a greedy parse covers the rest of the test block with 11,145 copies. Replaying them is estimated, not timed, at 0.1–0.3 ms a block. The inverse BWT it is set against, the control in that assessment, takes about 2.4 ms on the same block.

**It needs text positions.** The short parse needs its seed bytes in *text* order, and the BWT gives its rows in *sorted* order. With the same seed bytes in sorted order, the parse needs 18,132 copies. The BWT does carry a copy description of its own: neighbouring rows of a run read the same bytes (section 3), so each one can copy from its neighbour. Grouped into phrases, this is a known scheme (Navarro, Ochoa and Prezza, Theorem 7). On the test block it has 51,184 literal bytes and 10,954 copies. But 5,611 of its copies have sources that overlap them or come later in the text. Written front to back, a plain evaluation takes 12,406 sweeps. Putting the copies in text order means knowing each row's place in the text, which is what walking LF computes. The obstacle here is a different one from the run network's ([the copy-route assessment](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982429070>); [its independent reproduction](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982850361>)).

All four routes, side by side:

What each route avoids, and what it still pays

A summary of sections 1, 4 and 5. Counts are medians per run over 233 blocks; times are on a four-CPU cloud virtual machine, one thread, except the copy route's replay, which is an estimate. Open return, reported in its own post, avoids induction on the narrow runs and is 9–21% slower than eager Cells.

## 6\. Where things stand

**Measured and proved.** Three unlike constructions do about 1.8 operations per run on Wikidata, their counts correlate strongly, and they do not process the same events. The identities \\(E\_0 = 2r - 1 - C\\) and \\(q\_1 = E\_0 - V\_0 + 1\\) are proved and hold on every block. The tail of the boundary-orbit profile accounts for the variation Cells and the forest share. Refinement raised the work on the test block, acceleration has nothing to act on in Wikidata, and splitting a block costs far more total work. The copy route is compact but needs text positions.

**Not shown.** No one has shown that Cells is optimal. The shared count is not a lower bound, and the only proved bounds are the trivial ones: reading the \\(r\\) runs and writing the \\(n\\) bytes. Cells is the best-performing formulation tested.

**The open question.** On our reading, Cells and the forest pay for each boundary chain step by step. Some shortcuts exist. Division jumps over a chain that repeats, and a one-bit test tells when two neighbouring paths spell the same word. But every known way to learn how long an *aperiodic* chain runs walks it, or needs preprocessing of order \\(n \\log n\\). Can a gap's separation depth be found without walking its path? A method that could would attack the work Cells and the forest share.

**Next.** Two studies are under way: a bounded study of whether steps of different kinds in Cells can be fused into one, and a literature study of these gap chains.

In short: three constructions count about the same work, and much of what they share is how long the run boundaries persist. The ways tried to shorten that work raised it, found nothing to act on, or multiplied it. When unlike methods agree on a count, the reason is worth looking for in the input before tuning the methods.

---

**Research records:** [the synthesis of the four investigations](<https://github.com/lmmx/mibz2/blob/d256502657463014f64820865093fb419e89b7a3/docs/journal/monodromy/2026-11-18-synthesis.md>) · [refinement and parallel depth](<https://github.com/lmmx/mibz2/blob/049464756c76addb72cbfaed42d421d006126eab/docs/journal/monodromy/2026-11-18-o3-o5.md>) · [the boundary-orbit profile test](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-18-q-profile.md>) · [the forest's verification](<https://github.com/lmmx/mibz2/blob/eda3167badcbd82be5ebe47607ae4b4528ed030d/docs/journal/monodromy/2026-11-18-forest-verification.md>) · [the independent synthesis review](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6983166783>) and [its report](<https://github.com/lmmx/mibz2/blob/57b4a9403b1e26f602258248c1f088783156ec06/state/reports/synthesis-6982863130/report.md>) · [the forest review](<https://github.com/lmmx/mibz2/issues/145#issuecomment-6982103916>) · investigations [one](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-1.md>), [two](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-2.md>) and [three](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-astra-investigation-3.md>) · [the copy route, recovered](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-pro-investigation-recovered.md>) · [the Cells census](<https://github.com/lmmx/mibz2/blob/2157acad18598e9aa555b4f5e5576c0f875779de/docs/journal/monodromy/2026-11-10-cells-gap/evidence/i9-census-1.jsonl>) · [the forest census](<https://github.com/lmmx/mibz2/blob/1eb99cf48d5b5d8b62f70262ddf8cded98c38ca7/docs/journal/2026-11-18-switch-elimination/evidence/forest-wikidata.csv>)
