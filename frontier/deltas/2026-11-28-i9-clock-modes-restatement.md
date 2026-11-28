claim: i9-clock-modes
by: claude-code
outcome: done
evidence: origin/evidence bce3914d (release-scaling/run-20261127T140323Z.tar.gz, SHA-256 f7ef4216…6fb0, re-verified); `python3 bench/clock_modes.py analyze runs.jsonl --threads 1,12` re-run on it; raw per-round walls at 1t and 12t; docs/notebook/2026-11-28-i9-clock-modes-verdict.md §5; bench/scaling.py:148 (`--settle 0`)

**What was asked:** the closing steps of the frozen stop_test that must come from an agent other than 0.5.15's author (antigravity). They are: restate the 0.5.15 1t and 12t rows in CHANGELOG.md and docs/journal/release-attribution.md with within-mode figures beside the pooled ones, document the mode-robust estimator in docs/RELEASING.md §5, and close the claim. The i9 bundle had already been read (PASS, mechanism found; #1283).
**What I found:**
- I re-checked the bundle's SHA-256 and re-ran `analyze` myself. Every figure in the verdict notebook's §5 reproduces to four places:
  - 1t: pooled 1.0567 [1.0115, 1.1039]; within-fast 1.0175 (n 2/7); within-slow 1.0019 (8/3); same-mode 1.0159 [0.9905, 1.0419] (n = 3).
  - 12t: pooled 1.0950; within-fast 1.0430 (7/10); same-mode 1.0458 [1.0135, 1.0790] (n = 7).
- The raw walls bear out the 12t sub-cluster note. 0.5.15's fast rounds sit at 0.0594–0.0603 s (×7) and 0.0643–0.0651 s (×3). Within those clusters, 0.5.14/0.5.15 is about 1.02 (n 2/7) and about 1.006 (n 5/3).
- **Reading:** within a mode, the 1t step (+1.6%, CI spans 0) no longer clears the RELEASING §5b bar. The 12t step does (+4.6%, CI +1.4..+7.9), at half the pooled size. The 2t row was not in scope and is unchanged.
- **Caveat on the classification:** release scaling records no cycle counters, so its modes are classified by wall alone. It also runs back to back (`--settle 0`), where the probe saw no slow-clock runs. Its 1t slow rounds are therefore more likely the probe's full-turbo, higher-CPI layout mode than a clock drop. I say so in the CHANGELOG.
**What changed:**
- CHANGELOG 0.5.15 §Release scaling gains a within-mode restatement bullet, with the pooled figures kept.
- The ledger row `0.5.14 → 0.5.15` carries the within-mode ratios beside the pooled ones. Its candidates and its "unexplained" class are unchanged.
- RELEASING §5.4 gains a new step: read a moving step within mode as well as pooled with `clock_modes.py analyze`, and use a 2.0 s settle, never 0.2 s. The §5b bar itself is unchanged.
- `i9-clock-modes` is marked done.
**What's next:** nothing for this claim. Two candidates for a later change, neither taken here: release scaling could record cycle counters, so that its modes are classified by cycles rather than wall; and whether §5b's bar should be judged within mode is a rule question for Louis (RULES D).
