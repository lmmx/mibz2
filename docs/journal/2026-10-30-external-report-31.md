# External research report 31: the researcher's close of the burn-down, and round 39's acceptance (verbatim)

A research input, not a finding of this repository. The maintainer relayed both
messages on 2026-10-30, after external report 30 (`2026-10-30-external-report-30.md`).

**What it contains:**
- **The close of the analytical burn-down:** all 84 questions have a disposition, and
  N1's novelty is corrected to an engineering contribution.
- **The recommendation:**
  - P74's CI benchmark first, ahead of N1's Gate B. Both are the only immediate gates.
  - Both dispatch variants, since the branch model can't choose between them in advance.
  - The full-decoder CPU-s/GB and four-worker throughput as the acceptance criteria.
  - No i9 run.
  - The scratch corpus and literature PDFs kept until the gates are reconciled.
- **Round 39** accepted as answered.
- **No new implementation or benchmark is authorised** by these messages.

**The coordinator's notes:** `2026-10-30-research-portfolio.md` §8, which now carries
the two gates as asks.

This is source material, not a decision of the maintainer's.

The text reached us as rendered text (a table flattened) and is kept as received.

## The close of the burn-down, as received

````text
That’s a significant milestone. All 84 original research questions now have a disposition. The analytical burn-down is complete in the sense we originally wanted: no question has been left vaguely open, and every unresolved claim has a named piece of evidence that would settle it.

There are three particularly important outcomes.

First, P74 has survived serious falsification. It is no longer just a plausible SIMD optimization. Its complete state machine has passed instruction-level emulation and real-block tests, and the shipped binary exposes the precise dependency it is designed to remove. The branch-prediction and library-call findings also make its performance model substantially more credible than the initial shuffle-only estimate.

Second, N1’s novelty claim has been corrected. The s-run partition and its bound were already described by Kärkkäinen, Kempa and Puglisi in 2012. That’s an important scholarly correction. The potentially distinctive contribution is narrower: using those partitions for a multi-byte-per-lookup, whole-block inversion walk, with payload-bearing records constructed by doubling directly from L-runs. That should be presented as a candidate engineering contribution, not a newly discovered partition theorem.

Third, the remaining uncertainties are genuinely empirical. More mathematical rounds are unlikely to establish whether either surviving design improves the real decoder.

Where we stand

Area	Research status	Remaining decisive evidence
Original 84 questions	All dispositioned	None for inventory completeness
N1 correctness and space model	Established	Builder and walker CPU cost
P74 correctness	Established	Compiled-loop performance
P74 instruction mechanism	Supported by shipped disassembly and instruction tables	Whether candidate code realizes the expected gain
Literature audit	Complete	No outstanding citation review
Full-decoder improvement	Not yet demonstrated	Matched performance measurements

My recommendation

I would prioritize P74’s CI benchmark over N1 Gate B, while keeping both as the only immediate research gates.

P74 has a clearer, smaller engineering hypothesis: replace the recurring store-forwarding failure with a register-held state and preserve the advantage across RLE2 calls. The expected gain is modest—modeled at 4.6–5.8% one-core—but the mechanism is now unusually well characterized.

N1 could deliver a larger improvement, but its modeled builder traffic is much less encouraging, and a fast walk alone cannot establish a full-decoder win.

For P74, I agree with Claude’s proposed comparison: branching versus branch-free, both with short-run inline emission, against shipped on Wikidata, random and records, with a code-layout control. I would make the existing full-decoder CPU-seconds/GB and four-worker throughput requirements the eventual acceptance criteria, rather than promoting it on an MTF microbenchmark alone.

There is also one analytical subtlety worth preserving: the simple branch-predictor model does not establish the actual misprediction rate on the target processors. It is strong enough to justify testing both variants, not enough to select one in advance.

I would not authorize an i9 run at this stage. CI is the appropriate first performance gate, with production unchanged until the evidence warrants integration.

One housekeeping point: the 213 MB scratch corpus and 21 MB literature PDFs are part of the research’s reproducibility trail. I’d retain them until the gate results and the final portfolio are reconciled, rather than delete them immediately.

The outcome is better than merely having 84 answered questions: we have eliminated weak hypotheses, corrected a novelty claim, and isolated two measurable engineering decisions. That’s a successful research burn-down—even if neither survivor ultimately beats the shipped decoder.
````

## Round 39's acceptance, as received

````text
Excellent. That closes round 39’s short-alphabet correctness question with independent verification.

P74 now has checked behavior across all alphabet sizes from 1 to 256, including the edge cases where unused positions occupy part of the 32-byte register head.

I’d record round 39 as answered, not conditional.

With all 84 original questions dispositioned, P74’s remaining uncertainty is now entirely about performance: whether the compiled register-held loop beats the shipped implementation without regressions on random and records inputs.

The research position remains unchanged: correctness established, performance unproven. The next decisive evidence is the controlled CI comparison Claude outlined, rather than another round of MTF correctness proofs.

No new implementation or benchmark is authorized by this update.
````
