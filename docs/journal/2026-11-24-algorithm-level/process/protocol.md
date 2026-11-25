# From a hunch to a well-posed problem: the protocol, as it was run

*The Coordinator, 25 November 2026. Louis asked that the process be recorded at the meta level, so it can be reused on later projects. This file describes what was done, in order, with the reason for each step and the records it left. It is descriptive first: each step names its example from this work. The reusable rules are at the end.*

## The shape

1. **Start from a measured surprise, not a theory.** Here: two oracles that each help a little, and help much more together (the four-arm test: D/C = 0.920 at 10 workers; journal §44).
2. **Say what problem would explain it, and what would count as solving it.** The task and its referee come first: what is measured, what is the cost model, what is a fair comparison (`../essence.md`, `../programme.md` steps 1–3). Without that, every proposal is a guess about which resource it relieves.
3. **Find the simplest model that could be wrong, and test it on the cheapest data.** A two-station queue (one serial scanner, k workers). It predicted the knee from four worker counts; the profile across 1–20 confirmed it within 0.2 workers (§57). A model that predicts before it fits earns trust; one fitted to everything earns none.
4. **Read the field properly, in full.** Three performance-analysis courses, read in full rather than skimmed by subagents. The sources are saved (`../methodology/sources/`) and the reading is written up (`../methodology/performance-courses.md`). Louis's rule: "read the full material, don't use subagents that gloss."
5. **Mine the field against the measurements, with several independent readers.** The same question went to four readers: the Coordinator, ChatGPT Pro, Neon and MONODROMY. Each wrote its own answer *before* reading the others, then compared. The independence is what caught the errors:
   - two readers found the Coordinator's "the default costs 20%" was wrong about what the default does;
   - one found engineering wins the others missed (the scanner's plumbing, §62).
6. **Record every answer verbatim, then decide.** External answers go in `../external/` or `../reviews/` as pasted, never paraphrased. The Coordinator's decisions go in separate files with the reasons (`../briefs/character-review-decisions.md`), so a reader can check the decision against its inputs.
7. **Correct in public, in place, and say so.** A wrong claim is corrected where it was made (struck or rewritten, with a pointer), in the journal (a new section, never an edit of history), and on the channel (a CORRECTION). The correction says what was wrong and who found it.
8. **Review the experiment's design before its data, with several independent reviewers, for one round.** Three design reviews (ChatGPT Pro, Neon, MONODROMY record 128). Where they agreed, adopt. Where they differed, the Coordinator decides and says why. Stage 1 then took twelve additions *before* it ran, because they could not be added after.
9. **Register each run before its data.** Each run states its decision rule (or that it is diagnostic) before any data, on the channel. This is the team's RULES C1a.
10. **Run, analyse against a pre-registered plan, and classify what controls.** Every controlling cost is classified as engineering (how the work is done) or mathematics (what work is done). The analysis plan is registered before the data (channel #39).
11. **Once a bottleneck moves, revisit what it masked.** Ideas rejected while a different bottleneck held the wall may have been masked, as the four-arm test showed can happen. Re-qualify those whose evidence fits that pattern, in one batch, not everything every time (`../briefs/experiment-programme.md`).
12. **Tell the story as it deepened.** The blog follows the work, publishing as it goes and revising in flight. It says what was believed at each stage, what changed it, and what is still open.

## The records it leaves

| what | where |
|---|---|
| the problem and its referee | `../essence.md`, `../programme.md` |
| measurements, raw and extracted | `evidence` branch bundles; `../math-targets/data/` with its scripts |
| the literature, saved and read | `../methodology/` |
| independent answers, verbatim | `../external/`, `../reviews/` |
| leads, with corrections at the top | `../leads/` |
| decisions with their reasons | `../briefs/*-decisions.md`, journal sections, LOGs on #145 |
| the running account | the journal `../../2026-11-24-algorithm-level.md` |
| the Coordinator's replies to Louis | `../notes/coordinator.md` |
| reports that point to the records | `../reports/` |

## Rules for reuse

- **Measured surprise first.** Don't start from a technique looking for a use.
- **Pose the problem and its referee before proposing solutions.**
- **Prefer a model that predicts from little data,** and check it on data it didn't see.
- **Read sources in full and save them.** A summary of a summary loses what matters.
- **Use several independent readers and reviewers.** Each answers before seeing the others; then compare with arithmetic. The disagreements are the product.
- **Verbatim records for external answers; separate files for decisions.**
- **Correct in place and in public,** naming who found the error.
- **Register before data,** including the analysis plan.
- **Price changes jointly.** A bottleneck moves when you relieve it; a fixed-count prediction that ignores that overstates the gain.
- **Masked effects:** when a bottleneck moves, re-test what it may have hidden, selectively.
- **Durable records, not chat:** every substantive reply to the person directing the work is written to disk first, and the message points to it.
