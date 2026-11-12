# External research report 37: the researcher on Gate 1's result (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after Gate 1's result (`2026-10-30-p74-gate1-result.md`).

**What it contains:** the researcher's reading of the result.
- The 33.2% Wikidata stage gain is measured, and the verdict is correctly mixed.
- **A recommendation for option 1,** the single registered selector arm, fixed before
  measurement:
  - the branch-free arm when a block uses at most 32 symbols, or when at most 10% of its
    explicit ranks are 32 or more; otherwise the shipped kernel;
  - the classification's cost inside the timing.
- **What to report with it:** per-block selections, the original paired rules, and the
  records anomalies kept.
- **The CPU correction:** the runner is an AMD EPYC 9V74, Zen 4.
- **It is a recommendation, not authorisation.**

**The coordinator's note:**
- The selector arm is not started. It waits on the maintainer's own authorisation.
- The microarchitecture is corrected in the result file. The pre-registration's "Zen 3"
  stays as written, frozen, and the result file says which CPU the run used.

The text reached us as rendered text (cards flattened) and is kept as received.

## The message, as received

````text
P74 Gate 1: a genuine breakthrough on Wikidata, with a clear limitation

This is a substantially better result than the local tests suggested.

The branch-free P74 kernel reduced the complete checkpoint-aware MTF stage’s cost by 33.2% on Wikidata. That is measured, not modeled, and comfortably exceeds the registered 5% threshold.

At the same time, random and records do not meet the negative-control requirements. The formal verdict is therefore correctly mixed, not positive.

P74 branch-free · paired CI stage-cost change versus shipped

Wikidata checkpoints

−33.2%

Lower stage cost; 95% interval ±0.2 percentage points

Random checkpoints

+8.8%

Higher stage cost; 95% interval ±0.3 percentage points

Records checkpoints: +6.6%, with a wide 95% interval of −10.2% to +23.5%. That uncertainty must remain visible.

The experimental result supports an important claim: a register-resident MTF implementation can outperform the shipped checkpoint-aware MTF kernel substantially on rank-local data.

It does not isolate register residency as the sole cause, because the arms also change run emission and high-rank fallback.

And the reported 5.7% full-decoder gain is still a projection, not an observed decoder result.

My research recommendation: test the conservative 10% selector

Of Claude’s three options, I recommend Option 1: one final, preregistered selector arm, with the rule fixed before measurement.

The reason is that the workload separation is unusually strong.

Share of explicit tokens taking the high-rank fallback

Wikidata

1.6%

Text

5.1%

Random

76%

Records

90%

Threshold

10%

Observed corpus-level shares. The selector must classify individual blocks, not entire corpora.

The fixed rule would choose P74 branch-free when a block has at most 32 used symbols, or when at most 10% of its explicit MTF tokens have rank at least 32. Otherwise, it chooses shipped MTF.

There are three reasons this is scientifically defensible.

First, the cutoff sits well between the reported corpus populations. It isn’t being tuned to a narrow performance optimum.

Second, the selection inputs are already available in Symbols.syms; the exact high-rank fraction can be counted before executing MTF.

Third, it directly tests the remaining practical hypothesis: can we retain the substantial Wikidata improvement while avoiding the negative-control regressions?

There are two limitations I would explicitly preserve.

* Corpus averages don’t guarantee every constituent block falls on the expected side of the threshold. Claude should report actual per-block selections, not just overall rank shares.
* The records baseline had severe round-to-round anomalies. The selector’s result needs to satisfy the original paired statistical rules, not merely look good in its point estimate.

I would choose the branch-free arm, not the branching arm, because the CI result decisively favors it on Wikidata. This would not establish branch misprediction as the cause; it simply uses the better measured candidate.

The CPU correction matters

Claude’s identification of the runner as Zen 4, rather than Zen 3, is important and should be corrected everywhere the result is interpreted. The measured Wikidata gain is genuine for that CI environment; it must not be presented as a measured result for the i9 or other laptop CPUs.

Suggested decision for the coordinator

My research recommendation is Option 1: the single conservative per-block selector arm permitted by Gate 1’s registered mixed-result rule.

Freeze the selector before measurement: use P74 branch-free when the block has A ≤ 32 used symbols, or the exact proportion of explicit ranks q ≥ 32 is at most 10%; otherwise use the shipped kernel. Include the classification cost inside the timed stage.

Preserve the original paired statistical criteria, the negative controls, and the existing iteration limits. Report per-block selection counts, CI ratios and intervals, correctness, and whether the selector satisfies the positive rule.

Please also correct the CI microarchitecture attribution to AMD EPYC 9V74 (Zen 4), and retain the records corpus’s wide intervals and anomalous shipped rounds in the report.

This is a research recommendation, not authorization for Phase 2, N1 Gate B, or any i9 experiment.

The bottom line: P74 has now demonstrated a substantial stage-level improvement on its target workload. The question is no longer whether the fast-path design can be competitive—it can. The remaining question is whether one inexpensive selector can make that improvement safe enough to consider for the full decoder.

I recommend giving that single selector test its registered chance. No further tuning, extra experimental arms or Phase 2 promotion yet.
````
