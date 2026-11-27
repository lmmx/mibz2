claim: bottlenecks-floor-correction
by: claude-code
outcome: done
evidence: #975 (aa0558d), the notes it added; floor results README and report.txt at 311923ef (evidence cc8ba137); #952 cold-store reread (2aaded3a); the outline's §6 goal questions and §7 fact ledger

**What was asked:** the post-publication review #975's delta left open: a different agent from the drafter checks the dated notes in both bottlenecks posts against the evidence and the outline's goal questions, sets `reviewed_by`, and closes the card.
**What I found:** Every figure in the five notes and the companion's §3 table matches `report.txt` ("The floor and its parts", "The prediction, scored") and the README: Residual B 5.7 [5.5, 5.8] / 5.3 [5.3, 5.4] / 5.2 [5.0, 5.5] / 5.4 [5.3, 5.5] ms at 1, 6, 8, 10; Residual A 303.9 / 54.0 / 42.4 / 40.5 ms with the stated intervals, 42.9 ms at 9; 2.7% of 194.6 ms (5.2 / 194.6 = 2.67%); 13–17 GB/s; scan alone 179.1 ms, 92%. No original sentence was altered; forecasts stay above their measurements. The outline's goal questions 1–4 and both held-out questions are answerable from the notes alone. One wording error: the companion said the cold store was "adding about 55–65 ms to its 298 ms word-spelling figure", which reads as 298 + 60 ms. The traffic is inside the 298 ms (`E_consume` = A − B = 298.2 ms; #952 attributes ~55–65 ms of it to cold oracle-store reads), as `dropping-the-coordinator` states.
**What changed:**
- `docs/site/blog/2026-11-26-from-bottlenecks-to-better-algorithms.md` §3: that clause now reads "about 55–65 ms of its 298 ms word-spelling figure is that traffic, not the decoder's work".
- `frontier/blog/bottlenecks-floor-correction.toml`: `reviewed_by = "claude-code"`, `stage = "published"`, `status = "done"`.
**What's next:** nothing for this card. The `floor-and-pooled-scan` "48 ms flat" note from the outline delta is that card's concern, not this one's.
**Caveats:** goal question 5 (the transfer question about trusting a published forecast) is answered only by implication: the notes show a forecast being checked by an oracle run, but they don't state the general check. I left it that way, since the card asked for notes, not new argument.
