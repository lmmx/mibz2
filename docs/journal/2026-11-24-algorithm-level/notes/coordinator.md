# The Coordinator's messages to Louis, kept as a running record

*From 24 November 2026, at Louis's request ("i want you to be keeping notes of your last message in the journal please"). Each substantive reply to Louis is summarised here, newest last. The full plain explanations are in `plain-explanations.md`.*

## 24 Nov, 21:20 — back to the research: three moves
- At 10 workers there is up to 26% of whole-decoder time available: 19% in the serial Huffman scanner, plus about 8% in emission that shows once the scanner is out of the way. At 1–6 workers, emission alone is worth 15–17%.
- Move 1: a parallel scanner switched on only at high worker counts, building on #741 (0.883 at 10 workers, regressed at 6) and #770.
- Move 2: the mathematics, aimed at two things. What does the Huffman scan see that the workers recompute? And cheap output-order codes, through acquisition and closure.
- Move 3: profile the scanner first.

## 24 Nov, 21:31 — both commissioned (Louis: "be autonomous")
- `SCANNER-BRIEF.md` went to the Pathfinder (#145 comment 7145028132): part 1 profile, part 2 a gated parallel scan as an adoption registration, reviewed by the Researcher.
- The Coordinator handles review, routing, merging, tags and verdicts end to end. A check-in is set for 22:22 UTC.

## 24 Nov, 21:45 — Louis's four points, actioned
1. **Finer worker grid.** Every count from 1 to 20 (as corrected), with the shift point derived from a measured crossover and a fitted pipeline model (SCANNER-BRIEF amendment). MONODROMY and Neon are asked to check the method is the ideal one (channel #31).
2. **Notes.** This file, and journal §45.
3. **What the mathematics is for.** `MATH-TARGETS.md`. The induction runs in the workers. The mathematical target is the inversion's emission, worth 15–17% at 1–6 workers and about 8% behind the scanner at 10, with acquisition and closure as the obstacles. Construction's share is unmeasured, and an oracle to split worker time is queued.
4. **Everything durable, replies as summaries.**

## 24 Nov, 21:50 — scanner profile PR #838 sent back for the grid
- The Pathfinder's part 1 (#838, head e1ba6833) has a sound instrument: a cfg `scan-profile` feature that times the scanner's phases and both sides' waits.
- It was built on the 1, 6 and 10 grid before Louis's amendment arrived. It goes back for the full grid (1–20 (as corrected)), a per-count crossover, a fitted pipeline model and an amended C1a rule. It will be tagged on the new head.

## 24 Nov, 21:55 — the grid is 1–20
- Louis: "don't do 1-12 then 14,16,20 do 1-20". The brief, these notes and §45 now say every count from 1 to 20.
- The Pathfinder is told: rounds set to fit about 40 minutes, with the expected per-count precision stated.

## 24 Nov, 21:52 — #838 (a7e251ef) returned once more for the 1–20 grid
- The Pathfinder's update added the per-count report, rho, the measured crossover, the fit wall(k) = c + max(b, a/k), k* and a prediction from counts 1–4.
- It still used 15 counts, because it crossed with Louis's 1–20 correction. It is returned for all 20 counts within about 40 minutes.

## 24 Nov, 22:05 — the record restructured: lowercase names, subdirectories, TSV
- Louis: "i dont want this all caps md file approach … if you want a math_targets subproject with a readme that is better … tsv more appropriate than md … ask monodromy what would be worth metricising … measure things like avg rss peak rss etc".
- Restructured into subdirectories with lowercase names; the path map is in journal §46.
- `math-targets/` is now a small project: a README, `targets.tsv`, `metrics.tsv`, the four-arm data and an extraction script.
- MONODROMY is asked what to measure and what to reason about (channel #33).
- The Pathfinder is asked to add memory and scheduling metrics to the scanner profile before it is tagged.

## 24 Nov, 22:12 — #838 (51f6c16f) held: unrelated files swept in
- The update adds memory and scheduling metrics per decode: peak and time-averaged RSS, user and system CPU, page faults and context switches.
- Each decode now runs in a fresh process. In one long-lived process, glibc's per-thread arenas carried earlier decodes' heap forward (1 worker peaked at 80 MB first, 236 MB after 20 workers). A fresh process avoids that without an allocator change.
- Its last commit also modified 35 unrelated bench files (+3,154/−908), apparently an accidental add of working-tree changes. Returned for a revert commit; no tag until the diff is scoped.

## 24 Nov, 22:20 — the full character: a finer decomposition, put to every member
- `briefs/decomposition.md`: eight stages and three shared resources.
- Three linear methods, built around metabolic control analysis (flux control coefficients from small injected slowdowns; the summation theorem checks the decomposition is complete).
- Sent to MONODROMY (channel #34). Paste prompts were given to Louis for ChatGPT Pro and Neon. The answers become a recipe brief for the Pathfinder after #838 runs.

## 24 Nov, 22:35 — Louis's copy_if post, assessed
- Hand-vectorised stream compaction: a compare-to-mask, compress, store and popcount advance. The workstation's i9 has no AVX-512, so the AVX2 form (a movemask-indexed shuffle table plus vpermd) is the one that applies.
- It fits three filter steps in the workers: run-record extraction from the bitmaps `runs.rs` already builds; the thin/thick partition in Open's construction; and lanes' appends.
- An engineering lead that lowers the price of each filter step, not mathematics. It is held until the decomposition measurement shows which worker stage controls throughput. Recorded as `leads/vectorised-compaction.md`.

## 24 Nov, 22:30 — the decomposition answers arriving
- **MONODROMY:** records 123–125 (the pipeline model, what to measure, the decomposition), plus 118–122. Under independent review.
- **Neon:** cycle 4 (T1/T2 acquisition, T5 enrichment) is under independent review. `DECOMPOSITION.md` (00c08750) answers channel #34:
  - it splits S3, S5, S7 and R1;
  - it adds a missing serial S8 (reader retirement and copy), S9 (ramp, tail, cold start), R4 (turbo frequency drop) and R5 (CFS placement);
  - it shows SMT oversubscription starts at k = 8–9, because the decoder runs k+3 threads;
  - it finds 5–10% slowdowns infeasible unpinned (σ_pair 4–9% at k = 10), and proposes δ = +0.25 with pinning;
  - it proposes a four-tier design of about 90 conditions in about 40 minutes;
  - it warns that the summation theorem over-sums near a min-crossover and checks time, not resources.
- **ChatGPT Pro:** pending, by paste from Louis.
- Once all three are in, the Coordinator writes the recipe brief for the Pathfinder.

## 24 Nov, 22:48 — copy_if post read in full; MONODROMY asked to generalise
- Louis supplied the post's source. The full text is verified: about 10–40× from register compress, store and popcount advance. The AVX2 alternatives are aqrit's tiny LUT, Cordes's BMI2 pext, and switch plus shuffle. The benchmark-hygiene appendix applies to the decomposition recipe. `leads/vectorised-compaction.md` is updated.
- Louis asked whether anything besides copy_if could use the idea. That went to MONODROMY (channel #37).
