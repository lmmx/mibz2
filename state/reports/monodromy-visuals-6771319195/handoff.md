# Six causal figures for the MONODROMY post

Assignment: [6771319195](https://github.com/lmmx/mibz2/pull/162#issuecomment-6771319195). Research-only draft, on `team-state/researcher`; Coordinator owns the port and publication. No decoder code or site source changed.

## Pins and delivery

- Placement baseline: master **ba52e4d6f05729b6317ac8b7b37e576d79b193d3**, `docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md`. This is the rewrite that landed during drafting, after the initial c265dfb read.
- Technical evidence: #351 **2dde542d0bcc176e68aee20058f74f5e290bb060**, `docs/journal/monodromy/2026-11-10-rauzy-veech.md`, and that PR's current description. All numeric claims below come from this record; no fresh timing experiment.
- Style/reference: master's `docs/site/assets/stylesheets/blog.css` and `docs/site/blog/2026-11-09-pit-crew-service-threads.md`.
- `figures.md` is the porting manuscript: six complete inline figure snippets, captions included. Individual `.svg` files are source fragments using house classes: **inline them inside `.bf-fig`; do not use an external `<img>`**, which cannot inherit the page's theme variables.
- `proof.html` is a standalone visual proof sheet with the house stylesheet embedded. It is a drafting aid, not a new site page. `figure-notes.json` contains each mapping, explanation and evidence anchor. `offline-qa.json` records the checks and their limits.

## Placement against the rewrite

Count prose paragraphs within each section, excluding headings and figures. Quote anchors take precedence if later editing changes numbering.

| Figure | Location | Action |
|---|---|---|
| 01 Hidden addresses | Opening, paragraph labelled **The walk**, ending “and building it adds more.” | Insert after it. |
| 02 Sliding tray | Opening, paragraph labelled **One shift per run**, ending “a run averages about 14 rows.” | Insert after it. |
| 03 Return tickets | §1, after paragraph 2 defining the original and image orders, ending “each interval is moved, whole, to a new place.” | Replace **Two steps rebuild aab from its last column**. Retain the worked-example paragraphs after it. |
| 04 Filed receipts | §3, after paragraph 2 beginning “A cell is the overlap”. | Insert. This section has no figure in the rewrite. |
| 05 Route planning | §5, after paragraph 3 giving 1.279 versus 1.812. | Replace **The order of steps changes how many there are**. |
| 06 Nested book | §4, after paragraph 3 ending with the Crochemore–Russo reference. | Replace **How the row map leads to the Lyndon tree**. |

These are six proposed figures, not six additions on top of the existing figures. The §2 timing figure is outside this six-figure delivery; Pathfinder REVIEW 6771693515 correctly calls for rebuilding it around the repaired, shipped-reference comparison. Do not read this handoff as endorsing the existing historical timing figure. Display 06 before 05 in the post because the rewrite's section order puts the Lyndon tree first. The draft IDs follow the assignment's causal checklist.

## What each analogy earns

1. **Sealed addresses.** A request cannot name its successor until the present lookup returns. An envelope is a row access; the address inside it is the next row index. This makes the load dependency visible. It does not make every lookup a miss, every miss exposed, or all workers serial.
2. **Sliding tray.** The compartments keep their spacing under one translation. A compartment is a row; a tray is a maximal equal-byte run; the displacement is LF's common offset. This explains why one start/length/shift describes a run. It is a mathematical map, not a physical byte move; shifts can have either sign. The addresses 10…13 → 14…17 are an illustrative translation, not a measured block.
3. **Return tickets.** Ticket count and printed itinerary are different quantities. A loses a row ticket while B's one ticket remembers two bytes; the generic stack beneath shows one rule representing a whole cell. The toy is checked directly: L=baa, LF=[2,0,1], A=(2,a), B=(1,b); the top-winning step gives A=(1,a), B=(1,ab), then the equal case gives aab. Generic AB→X is separate from the three-row example. Words are in reconstruction order, not the forward LF emission order. The caption carries the density consequence: S/n=0.1309 on Wikidata, 0.6761 on prose, about 1 on uniform bytes (§4). Short runs amortise less; **long runs alone do not guarantee a small S**: `(ab)^k a` has r=2 and S=n/2 (§2). Output still costs Θ(n) byte emission; only induction work is counted by S.
4. **Filed receipts.** Rehanging list cards means patching neighbours; filing a completed cell leaves old neighbours alone. The live clipboard explicitly keeps unfinished state separate. §3.1 writes 5–6 links; §3.5 writes two slots for a completed cell, keeps the winner remainder in registers, and promises the next slot. No claim that all writes are sequential, no memory-space saving, and no claim that the corrected output writer writes every byte only once. Marking the original row and prefetch are additional changes. Journal §6.2 does time successive builds on ten blocks, one run each: patched cells 13.4 ns/cell, unpatched/three-way 12.5, untracked 9.8, prefetched 7.2–7.7. These are staged changes, not a factorial isolation of every mechanism.
5. **Route planning.** Fewer stops compete with the cost of choosing them. Stops are cell steps; magnifiers are candidate searches. “Free” means no new boundary, not free CPU time. The 1.812, 1.279 and 1.220 figures are pooled steps/run for the recorded Wikidata census. The drawn routes/stops are schematic, not scaled counts or physical cache distances. The floor is r₂−1 in this step model; it is not a universal inverse-BWT bound. §8 reports a DIFFERENT worklist: free steps where ready, else the RIGHT END, 1.458 steps/run on ten blocks (right-end sweep 1.790 on that subset). It tests 2.25 segment ends/step, with two steps in three moving segment. The 150 versus 115 instructions/run is a price model using 64 instructions/cell, estimated 10/test and 25/move, not a timing result. The 1.279 order additionally needs largest-cell selection. The caption explicitly separates this estimate from the illustrated schedule count.
6. **Nested binding.** Opening a bound part reverses the joins that made it. A book is a word, a binding is a binary concatenation, and the bookmark identifies the standard smallest-proper-suffix split. For aab the proper suffixes are ab and b, so the first split is a|ab; ab then splits a|b. The seam rule prevents this being a generic parse-tree cartoon. The identity is the right-end induction's expanded tree for the primitive word's least rotation (§5.6); shared equal rules produce a grammar. No new suffix-sort pass is implied. The known Lyndon/Cartesian identity remains attributed as in the prose.

Every public caption includes the mapping and a sentence beginning “The analogy breaks…”; those qualifications are part of the figure, not optional audit footnotes. None depicts a whole-decoder gain.

## Corrections worth carrying with the port

The brief's “fewer instructions and cache misses” is supported with narrower scope than a causal comparison of list versus cells alone. **184/260 instructions and 2.1/8.7 simulated last-level misses per run are block-1 callgrind counts, before the writer fix, cells versus the column path** (§0/§6.2). They are not hardware miss counts, a paired post-fix comparison with the shipped records path, or an isolated effect of filing cells. Replace the existing §3 claim that the changes were only timed together: §6.2 has a successive-build timing table (ten blocks, one run each), which supports a staged comparison, though not a clean isolated effect for each simultaneous edit. Suggested replacement for §3's final paragraph:

> The cells kernel uses more memory than the table, but the earlier block-1 simulation recorded fewer last-level cache misses: 2.1 per run against the column path's 8.7, with 184 instructions against 260. These counts predate the writer fix; they explain that build's behaviour, not the corrected kernel's exact counts or a whole-decoder gain.

Do not restore the old 26.2% headline with a new drawing. §6.3's repaired comparison is 1.677/2.049=0.818 of the records-path inversion stage, fastest-of-five per block, local one-thread evidence. A later same-code census report, channel LOG 6771530689, widens the observed local ratio to **0.82–0.86** and says the journal/PR will be corrected; the old three tightly grouped runs were one machine state. This update is reported evidence, not a replay here. No whole-decoder comparison. The figures deliberately avoid timing bars so that their mechanisms survive another census without silently becoming claims about it.

## Late evidence incorporated before publication

Pathfinder REVIEW [6771693515](https://github.com/lmmx/mibz2/pull/145#issuecomment-6771693515) arrived after the first draft commit. Re-read the actual journal §6.2 and §8 item 1a at 2dde542d: its F1 and F2 are correct. The earlier draft inherited two false statements from the post (no staged timings; a timed failure of the cheaper-order worklist). This handoff and the public scheduling caption correct both. No timings have been run in this assignment. The first draft is superseded by this commit. The later census range is from [MONODROMY LOG 6771530689](https://github.com/lmmx/mibz2/pull/145#issuecomment-6771530689).

## Checks performed and remaining

- Six SVG documents parse as XML. Titles/descriptions and unique per-figure accessibility IDs are present. No scripts, external fonts, linked images or animation. All six viewBoxes are width 540; body labels are at least 23–24 SVG px, with large quantities at 28–38 px. Colour is redundant with labels and geometry.
- Rendered all six in light and slate palettes at **312 px** drawing width (a conservative phone content area) and **750 px**. Used the actual house palette variables, resolved for an offline PyMuPDF SVG raster, and DejaVu Sans glyph outlines for proofs so the renderer did not substitute a serif font. Inspected both six-figure phone contact sheets; corrected envelope-label/flap and arrow/text collisions. Text bounds pass with the wider DejaVu metrics. These are offline proofs, not browser screenshots.
- Tried installed Chromium with Playwright. It cannot start in this sandbox: `process_singleton_posix.cc:297 socket() failed: Operation not permitted`. No escalation attempted. Browser typography, integrated responsive layout, HTML captions, site build and publish-page remain Coordinator checks.
- Routing audit via pinned `coord.py`, fed fresh connector comments for #145/#160/#161/#162/#222 since the last completed assignment: no misroutes. REVIEW route is #145. Publication verification is recorded separately in the cursor.

Port the six snippets at the anchor paragraphs, preserve captions, then run publish-page on the integrated docs head at phone and desktop widths in both themes. This delivery does not claim the post has been updated or published.
