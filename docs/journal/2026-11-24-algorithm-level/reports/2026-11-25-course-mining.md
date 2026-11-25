# The performance courses, mined four ways: the report

*The Coordinator, 25 November 2026. A summary that points to the records. Nothing here is new: every claim is in one of the files below, which are the durable form.*

## What was asked

Louis, after the scanner profile (journal §57): "that is awesome and also suggests we need to mine these courses for opportunities then."

The three courses (Jain's; Harchol-Balter's 15-857 and 15-849b, with their reading lists) were read in full earlier: `../methodology/performance-courses.md`, with the sources in `../methodology/sources/`.

The same question went to four readers, each answering independently before comparing:

| reader | answer | prompt | journal |
|---|---|---|---|
| the Coordinator | `../leads/course-opportunities.md` (with its corrections at the top) | Louis's message | §58 |
| ChatGPT Pro | `../external/chatgpt-course-mining.md` (verbatim) | `../briefs/chatgpt-course-mining-prompt.md` | §61 |
| Neon (Antigravity) | `../reviews/neon-course-mining.md` (verbatim copy of `research/neon-movement/COURSE-MINING.md` at dd6cc26d) | `../briefs/neon-course-mining-prompt.md` | §62 |
| MONODROMY | pending: channel #39, #346 comment 7148095764 | channel #39 | — |

The measurements they all used: `../math-targets/data/measurements-scan-profile.tsv`, derived by `../math-targets/scripts/extract_scan_profile.py` from the bundle `scan-profile/run-20261124T223748Z`.

## What came out of it

**1. A concrete engineering win, found by Neon and verified in the code (§62).** Two plumbing costs sit on the serial scanner, the bottleneck:
- the input window's copy at each 4 MiB chunk boundary (about 10 ms);
- the symbol buffer's re-zeroing after it is truncated (about 5 ms).

Together they are about 8% of the scanner's demand. They are routed to the Optimiser (#145 QUEUED 7147862124).

**2. A correction to the Coordinator's lead, made by both ChatGPT Pro and Neon (§61, CORRECTION 7147771581).** The public API builds 7 workers on the i9, not 20. The live default question is 7 against 8 workers (−7.6% wall in the profile). Louis's caution, recorded in §61: the 7-worker cap is a temporary symptom of the scanner limit, and no idea that raises the ceiling is abandoned because of it.

**3. Bottleneck migration, priced (ChatGPT Pro).** At 8 workers the scanner is only 4% above the workers, so a scanner-only gain there caps at about 4–6%. Scanner and worker improvements must be priced together, with the count retuned after either: both cut by 20% predicts 0.162 s at 8 workers.

**4. The "machine bound" demoted to a conditional scenario (both).** The Coordinator's 0.109 s figure rests on five unmeasured assumptions. Neon's 10-core pooled figure is 0.163 s, or 0.125–0.142 s with realistic SMT gain. ChatGPT Pro's idealised splits are 0.156–0.172 s.

**5. Sharper facts about the workload (Neon):**
- the 792 tiny blocks are overflow stubs at window ends;
- the real blocks' sizes are autocorrelated along the stream;
- the scanner's high-count slowdown is a bimodal mixture across runs, not a step;
- favoured-core pinning likely buys 0–2% at 8 workers.

**6. What was ruled out, and what was only "not supported yet".**
- Ruled out, given unchanged demands: gap-removing scheduling, which can recover at most a few percent.
- Not supported yet: size-based scheduling and heavy-tail methods. Worker service per block was never measured.
- Withdrawn: the claim that the decoder is a closed system. It is a finite pipeline with feedback-controlled admission.

## How this feeds the experiment

The programme that follows from it is `../briefs/experiment-programme.md`. The process that produced it, written to be reused, is `../process/protocol.md`.
