# Cutting the loop open · Researcher round 2

**REVISE in flight.** The new figures and measured values check out, and the revision now answers the five main goal questions. One decoding step remains missing. A periodic block can close its marked LF lap before it has spelled the whole block. Correct the opening and carry the same stopping rule into §3. The live controls work. The stable-sort figure fits at phone width, but its smallest labels are hard to read.

I reviewed [the published article](https://lmmx.github.io/mibz2/blog/2026-11-18-cutting-the-loop-open/) and [technical notes](https://lmmx.github.io/mibz2/blog/2026-11-18-cutting-the-loop-open-notes/) after [#568](https://github.com/lmmx/mibz2/pull/568) and [#570](https://github.com/lmmx/mibz2/pull/570), for [inbox comment 6983357476](https://github.com/lmmx/mibz2/pull/162#issuecomment-6983357476). Both deployed pages loaded assets stamped 642be1bc3efc, matching 642be1bc3efc7c702773e85e8a3a696b97a1821d. I committed the [primary-record baseline](https://github.com/lmmx/mibz2/blob/da6f1ad0a9046b5b13a7b3666aeccbf833bfe8ab/state/reports/cutloop-blog-r2-6983357476/baseline.md) before opening the revision or reading #570’s goal-question answers. The first exact retrieval of the second technical review’s archived report succeeded later. The baseline reconstruction uses the proof, primary records and data.

## Required correction

The opening says the marked walk “reads the block backwards, one byte a step, until it comes back to the primary row.” Its next sentences show that abab visits two rows out of four. A reader following that stop rule would write only two bytes. The proof says that one lap gives the marked period and the writer repeats it to the known block length. For abab, LF visits 0 → 2 → 0 and reads ba backwards. Reversing gives ab, and two laps’ worth give abab.

Suggested wording in the opening. “Following LF from the primary row spells the text backwards. If the walk returns to the primary row early, the decoder repeats that spelling until it has the block’s known length.” In §3, give the inverse of the kept return the same stopping rule. The worked banana$ example emits the full block in one lap. This is the remaining chain-level gap. [Article source](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/docs/site/blog/2026-11-18-cutting-the-loop-open.md) · [Proof and spelling rule](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/proof.md)

## Where the baseline lands

| Takeaway | First landing and follow-through |
|---|---|
| Cut outgoing arrows, solve the open paths, recover the erased wiring by stable sort | “The idea” gives both jobs and says what the cut loses. The first findings bullet gives the composition. §§1–3 supply the path, rank and spelling steps. The marked-lap stopping rule needs the correction above. |
| A cycle already uses every incoming arrow | Before the first press in §1, then again in the final step and fallback. The reverse argument accounts for each kept row ending one path. |
| Stable sorting yields entrance ranks | The opening says stable sort. §2 first distinguishes the coordinates, then shows row 3 at rank 2. §3 follows the coordinates into the trip table. |
| Short runs supply many boundaries and few rows, leaving wide strips to bundle | The opening gives 93% of runs and 10% of rows with the causal explanation. §5 moves the run split next to strips and separates width from height. §6 scopes its threshold trend and links the example where another loop opens. |
| Sixteen to one describes path work, with table construction and output still to pay | All three points land in the opening. §7 shows a splice and separates whole-stream wall time, one frozen decode’s heap and inverse-only phases. The wall attribution is labelled interpretation and the i9 test remains open. |
| Proof, experiment and prior theory | The opening gives the proved factorisation and known Markov-chain result. §4 states when the matrix inverse applies and labels the Cells reading as interpretation. §7 scopes the measurements to the machine and workloads. |

## Simulated reader

| Goal | Answer from the rendered article and notes |
|---|---|
| 1 · What is cut and what are the two jobs? | Remove each arrow leaving a kept row. The remaining paths give entrance-to-exit map P and the words. A stable sort of kept letters yields LFᵥ, the target ranks of the erased arrows. Compose g = P ∘ LFᵥ. Follow the marked g cycle backwards and emit each reversed trip. The article demonstrates this through banana$, with the general length rule still missing. |
| 2 · Why can no open path enter a cycle? | Each row originally has one incoming arrow. Every row on a cycle already uses that arrow within the cycle. An entrance has lost its incoming arrow. Its path must end at a row whose outgoing arrow was cut, possibly at the same row. |
| 3 · What is k in k = LFᵥ(j)? | k is the position of that target in the sorted entrances. The relative order survives deleting the other rows from the full stable sort. With kept rows 0, 4, 6, the entrances are 0, 1, 3. The third kept source points to original row 3 at entrance rank 2. |
| 4 · Why keep the short runs? | At threshold 16 on the frozen block they account for about 93% of runs and 10% of rows. Wiring them in the stable-sort pass takes many potential split boundaries out of the open paths. About 90% of rows stay on those paths. Width enables sharing and height still sets the cost of a representative walk. |
| 5 · What does sixteen save, and how did the decoder do? | It describes how many path-letter occurrences one representative trace covers. The decoder still builds and walks the kept table and writes every output byte. It asked for about 40% less additional heap in one frozen decode. Whole-Wikidata wall time on the four-CPU VM was 9–21% slower than eager and 7–8% faster than lazy. The many-core result is open. |
| H1 · What if every thousandth row is kept? | Add the primary if needed. The factorisation and stable sort still recover the marked return. A cut inside a wide strip can fragment bundles. Leaving short runs on the open side also leaves their boundaries in play. The article supports these mechanisms. The expected answer’s “everywhere” and a guaranteed rise in U require a specified block and evidence. The new cut can change both the bundles and the amount of complementary work. |
| H2 · How can a higher threshold add tracing? | Additional kept rows can cut open a separate former loop. In the six-row notes example, raising the threshold from 1 to 2 takes U from 2 to 3. The frozen block’s falling U describes its own sweep. |

The held-out questions were revealed in #570 after the baseline was frozen. The answers above are grounded in the rendered revision. This was a simulated reading with the questions in view.

## Stuck points by level

- **Symbol.** The caption and §2 now give the rank and original row explicitly. At 340 CSS px, the 720-unit SVG is 267 CSS px wide. Lower-panel rank and explanatory glyph boxes are about 6 CSS px high. A stacked phone panel or a compact rank inset would let the visual carry its explanation. The visible prose and caption already carry it.
- **Statement.** The frozen sweep and prose range are now scoped. In §4, “every state of C is transient” would be clearer as exit from C almost surely under the dynamics stopped on first exit. The preceding wording already conveys that condition. The first-return sum can terminate in the permutation example even when an unused complement cycle makes the full matrix inverse singular.
- **Step.** The stepper now introduces the occupied incoming edge before interaction. The trip table and flow box correctly give the reversed word. The splice panel uses LF reading order for na and nab. The implementation stores reverse words for emission, as the notes explain. “Words in LF order” beside the splice would make that convention easier to spot.
- **Chain.** The marked-path explanation needs the repeat-to-length step in the opening and §3. The shortened formal frame now bridges rank and spelling without the prior lengthy repeat.
- **Beyond.** The transfer to other kept sets works with the H1 qualification. The gap-bound sketch is still the densest link. A two-gap visual could show one gap forwarded and one reaching its first boundary. The linked proof carries that derivation, and the sketch’s density does not impede the main decoder explanation. §7 now depicts the three measurement scopes directly. I found no further fragmentation of the core story or collapse across measurement scopes.

## Figure and number checks

| Item | Independent check |
|---|---|
| abab | Stable sorting bbaa by byte then row gives LF = [2,3,0,1], two cycles, and the claimed two-row marked lap. The lap alone supplies the period. |
| Six-row 110000 witness | LF = [4,5,0,1,2,3]. Primary 2 at threshold 1 gives path 0→4→2, length 2. At threshold 2, kept rows 0,1,2 give the empty path at 0 and 4→2 and 5→3→1. They contribute lengths 1 and 2 in separate bundles. |
| Stable-sort panel with row 6 | The off-diagonal panel uses seven banana rows numbered 0–6. LF = [1,5,6,4,0,2,3]. Kept rows 0,4,6 have images 1,0,3 in source order, entrance order [0,1,3], v=a$a, and LFᵥ=[1,0,2]. The displayed rank links agree. |
| Endpoint splice | Replay gives T2→2→6→3→4→T1, spelling nab in LF order. Removing rightmost row 6 replaces 2→6→3 with 2→3 spelling na. Removing 5,4,3,2,1,0 in turn gives T0→T0 empty, T1→T2 n and T2→T1 nab. The caption identifies the example as individual-row splicing and describes the actual common-suffix batches. |
| Three scales | Paired Wikidata ratio medians against eager/lazy, warm-up excluded, are 1.171195/0.917786, 1.210720/0.931476, 1.091331/0.927659 (11/9/11 measured rounds). Heap is 6,287,363 vs 10,353,399 additional requested bytes, down 39.27% in one frozen decode. Separate build/write phase medians over 51 measured rounds are 1.856518/1.018372 ms for the queued endpoint and 6.771081/0.511080 ms for the switch cut. All plotted numbers round correctly. The phase bars add the separate medians. |
| A/U and prose | Median of the 233 large Wikidata blocks’ individual A/U values is 15.9236025. The frozen ratio is 810,057/50,709 = 15.9746. Six prose A/n measurements give median 32.3374% and range 27.5938–48.9791%. The printed 15.92 and 32.3% agree. |
| Earlier scope fixes | The upper bin now says longer than 256 and describes the split-primary convention. The one-strip caption gives the range for the two heights and the height-one case. §7 gives the two constructors’ bundle and traced-letter count comparison on 431 Wikidata and six prose blocks. Output parity is scoped to length and SHA-256. The cache measurements stay simulated and wall attribution stays interpretive. |

[Proof](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/proof.md) · [Endpoint implementation](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/endpoint.rs) · [Block counts](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/evidence/blocks.csv) · [Frozen sweep](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/evidence/frozen.csv) · [Wall timing](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/evidence/timings.csv) · [Heap](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/evidence/heap.csv) · [Phase timings](https://github.com/lmmx/mibz2/blob/e6df6aee5055840b8b6ed780e29727e23da6d42f/docs/journal/2026-11-18-open-return/evidence/phases-frozen.csv)

## Live controls and responsive layout

- Next traversed all ten frames and disabled on the last. Back returned from frame 9 to 8. Home on the slider returned to 0. Play changed to Pause and advanced the slider and narration. The exact notes heading and the 110000 example loaded on their own route. The article rendered 138 KaTeX formula elements with zero KaTeX errors.
- Checked 454 and 340 CSS px in Chromium using ordinary page zoom. Those widths bracket 390 px and trigger the mobile layout. Every figure fit inside its card. Wide tables kept horizontal overflow within scrollable table wrappers. At 340 CSS px the stepper buttons were 44 CSS px high, its slider wrapped below them and Next updated the narration. I checked the stable-sort figure in light mode and the three scales in dark mode, then restored zoom and theme. This exercised the responsive layout in a desktop browser. Physical-phone touch gestures remain untested.
- I replayed the constructions and recomputed summaries from the archived data. The benchmark measurements came from the recorded runs.

The output-length correction is ready for a focused recheck of the opening and §3. The phone figure and gap illustration can be improved independently.
