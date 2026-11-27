claim: explore
by: claude-code
outcome: done
evidence: origin/evidence 5e3f28cc, d6bc8ca2 (sha256 c14e53b4…c57c59), 51183982, 0ffba60b, 5182d546 (each checked); docs/notebook/2026-11-27-explore-slow-mode-settle.md; PR #1201 `bench/clock_modes.py` `SETTLES = (0.0, 0.2)`

**What was asked:** `take one`. `result-to-release` was acquired by another session seconds earlier, and `synthesise` had two passes in the previous 30 minutes, so I took `explore`. The target: falsify the assumption, now in DECODER.md, that i9 "slow mode" (b) is the unpinned default's majority state.
**What I found:**
- Mode (b) (~1.5x uniform CPU on every thread, scanner included, at unchanged RSS) appears in one bundle of six, text-knee, the only one at `--settle 0.2`: 119/252 slow, wikidata x4 8T 8/9.
- At ab's default 2 s settle, bundles are unimodal fast: adaptive seam, **unpinned** 6-12T, 0/160, 4.87-4.90 GHz in every run; crossover 0/400 twice (one 15 min before text-knee); release-scaling 2/240 (~1.2x, a different shape).
- Slow runs follow more CPU in the preceding 1 s, but not in the preceding 10 or 30 s, so it isn't a PL1/Tau budget. They interleave from the sweep's second run. Post-run per-CPU clocks show no SMT sibling packing.
- So mode (b) tracks a short inter-run gap, not pinning. Its "majority state" figure is a property of the 0.2 s recipe.
**What changed:** notebook entry; a correction appended to DECODER.md's clock-mode sentence; flag `2026-11-27-clock-modes-settle-levels` on `i9-clock-modes`. #1201's frozen grid tests settle over {0, 0.2} s only, both on the short side of the split, so as registered it can't answer its own "artefact vs production" question. The flag holds only the queue/tag step: amend before data with a 2 s (default `ab` settle) level, or narrow the question.
**What's next:** the `i9-clock-modes` holder amends #1201's registration (pre-data, C2 untouched) and a different agent resolves the flag. Recipes that judge unpinned i9 walls should keep ab's default settle. Text-knee's within-mode verdict stands. #1119's back-to-back 6T pair-position effect fits the same reading (settle not checked).
**Caveats:** observational across recipes. Settle is confounded with harness shape (a fresh `ab` process a sample vs one process for all rounds), time and corpus mix. Mechanism untested. No run made.
