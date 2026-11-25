# Writer brief: the deepening, told from the beginning

*Prepared by the Coordinator on 25 November 2026. Sent to the writer once MONODROMY's course-mining record (channel #39) has landed, so that all four readings are in. Louis's instruction: feed the writer by pointing to files, not by passing a summary, so the post can carry real detail and colour rather than a hand-wave. The work comes first; the post runs in parallel.*

## What the post should do

**Extend "Posing the inversion problem"** (or start its sequel, if it reads better as two). The narrative is a *deepening*:
1. a simple observation (two oracles: 0.976 and 0.806 alone, 0.741 together at 10 workers; a gain invisible until another bottleneck is removed);
2. posing the problem properly: the referee, the programme's eight steps;
3. the first model and its first test (a two-station queue predicting the knee from four counts);
4. finding the performance-analysis literature and reading it in full;
5. mining it four ways, independently, and what the disagreements caught:
   - the default-count error;
   - the scanner's plumbing;
   - bottleneck migration;
6. the experiment that follows, and where mathematics may enter.

It should also say, at the meta level, *how* the work is being done (`../process/protocol.md`): independent readers, verbatim records, public corrections, registration before data.

## Takeaways first (`coordinate-blog-post` §3)

Before drafting, write the simplest correct account and the key takeaways. Our suggestion, to check against the records:
- The decoder is a serial scanner feeding parallel workers. A simple queueing model predicts where adding workers stops helping (about 7.5 on the i9) from four measurements.
- Above that point the scanner sets the speed, and a scheduling change that leaves the work unchanged can recover only a few percent. Real gains need less work at the bottleneck. When you get it, the bottleneck moves, so improvements must be priced together.
- Four independent readings of the literature, compared, found:
  - an error in the first reading (what the default worker count actually is);
  - about 8% of the scanner's work in two avoidable copies.
- The next experiment is designed to tell which mechanism controls where. Its results decide which costs are engineering and which might yield to mathematics.

## Files to read (in this order)

All are under `docs/journal/2026-11-24-algorithm-level/` on `docs/journal-algorithm-level`, unless noted.

1. `programme.md`, `essence.md`: the problem and its referee.
2. The journal `../2026-11-24-algorithm-level.md` §44 (the four-arm test), §§52–56 (the methods; the queueing question), §57 (the scanner profile), §§58–62 (the mining and its corrections).
3. `math-targets/data/measurements-scan-profile.tsv`, `measurements-four-arm.tsv`: the numbers, for figures.
4. `methodology/performance-courses.md`: what the courses contain.
5. The four readings:
   - `leads/course-opportunities.md`, with its corrections at the top;
   - `external/chatgpt-course-mining.md`;
   - `reviews/neon-course-mining.md`;
   - MONODROMY's record from channel #39 (`docs/journal/monodromy/` on `claude/sharp-meitner-cx0v78`).
6. `reports/2026-11-25-course-mining.md`: the summary that points to them.
7. The design reviews and decisions:
   - `external/chatgpt-design-review.md`;
   - `reviews/neon-design-review.md`;
   - MONODROMY record 128;
   - `briefs/character-review-decisions.md`.
8. `briefs/experiment-programme.md`: what runs next.
9. `process/protocol.md`: the meta level.

## Figures we expect

- **Wall against workers, 1–20,** with the model's two asymptotes and the knee (predicted against measured).
- **The scanner's time split** (work against waiting) across counts.
- **Bottleneck migration:** predicted wall against scanner cut at 8 and 10 workers. ChatGPT Pro's table gives the numbers.
- **The four readers:** what each found, and where they disagreed. A table or a diagram.
- **The experiment programme** as a dependency chart.

## Care points

- The corrections are part of the story; tell them plainly. The claim "the default runs 20 workers and costs 20%" was wrong: the public API runs 7 on the i9. Louis's point stands: the 7-worker cap is a temporary symptom of the scanner limit.
- The 0.109 s "machine bound" is a *conditional scenario*, not a bound. Don't print it as a ceiling.
- Every number is traced to its file; anything that can't be traced is flagged in the PR.
- Follow `write-blog-post`, then `publish-page`. Publish when fact-checked and rendered; the Researcher's two review rounds then run on the live post.

## Added 25 November (overnight): the latest records

Louis asked for the writer to be pointed at these now. MONODROMY's course-mining record is still to come and will be added when it lands. Read them in this order:
1. **Stage 1's results:** journal §66 and `math-targets/data/character-stage1/report.txt`.
   - The two-station model held exactly, and the switch from workers to scanner comes at 8.
   - Concurrent-state evidence shows the scanner limits above the knee.
   - The reader is not a co-bottleneck.
   - New finding: the public API runs the older eager kernel when asked for 3, 5 or 7 threads (61% slower at 7).
   - Caveat: the run flagged itself PERTURBED at 10 and 20 workers.
2. **The independent readings, and how they corrected each other:** §§61–62 (ChatGPT and Neon on the course mining: the default-count error; the scanner's plumbing; bottleneck migration).
3. **The design thread:** Louis's Alon question and the turn to design for a purpose: §§64–65, `leads/structure-and-function.md`, `external/chatgpt-alon-motifs.md`, `external/chatgpt-khammash.md`.
   - Alon's motifs, then Ma et al.'s exhaustive enumeration, then Araujo and Liotta's basis for the solution space, then Khammash's "function, necessary conditions, solution space, realisability, optimality".
   - This is the deepening at the level of method, and the best material for the post's middle.
4. **The dynamical layer:** §67 and `external/chatgpt-dynamics.md`: hybrid systems, "digital on analogue".
5. **The hybrid-systems book:** §§68–70, `reading/hybrid-dynamical-systems/relevance/levers.md`.
   - The five levers are the plain-language takeaway.
   - **§70 is a correction to tell plainly:** the Coordinator first called the bottleneck bound a completion-time certificate. ChatGPT showed it is only a lower bound, with a three-line counterexample (a 1 s scan, a 1 s decode and a 1 s delivery in series: the bound says 1 s, the truth is 3 s).
   - That correction is a good example of the process: independent review catching an error before it guided the experiments.
6. **The process itself:** `process/protocol.md`. Update it if the post needs it; the reviews and corrections above are instances of its steps 5–7.

**Care points, added:**
- The certificate claim is withdrawn: don't print it.
- Stage 2 is running overnight; its results are not in yet.
- The scanner fix's gain at 8 workers is uncertain: about 6% (Neon, on the scan profile) or nearly nothing (ChatGPT, on stage 1's instrumented numbers). Run 2 decides.
