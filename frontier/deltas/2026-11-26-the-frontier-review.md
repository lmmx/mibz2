claim: the-frontier
by: claude-code
outcome: done
evidence: docs/journal/2026-11-26-rhizome/the-frontier-review.md; draft at daf0224 (#950); frontier/surface.tsv (D row after #952); frontier/deltas/2026-11-26-stress-test-frontier.md; paradigm3.org/research/swarms and tobyord.com/writing/swarm-scaling (read 26 Nov)

**What was asked:** Move blog card `the-frontier` from `draft` to `review`. That means a cold review by an agent other than the drafter: a simulated-reader pass, and every number checked against its source.
**What I found:** The draft is sound. Nearly all of its ~90 figures match their primary files, and all four external pages match what the post cites. Eight findings remain, three of them substantive:
- **F1 (must fix): the 48 ms is stale.** The draft says delivery's copy-out is "48 ms flat". #952 showed that was a cold-store artifact. The surface's D row is now 29.1 ms at 1 worker and 34.0 ms at 8, spilling L3 at 9–10 workers (43.7–77.9 ms at 10).
- **F2: protection is not yet binding.** The invariants table calls protected paths "mechanical enforcement". In fact `check` only warns, and `master` has no branch protection (stress-test scenario 8).
- **F3: two stress-test failures are understated.**
  - `renew` and `release` exit 0 when their compare-and-swap push fails (scenario 3).
  - `check` misses edits to a claim's `stop_test` (scenario 6).
  - The walkthrough also implies that `frontier.py` checks `premises`. It doesn't (scenario 4).
- **F4–F8 (small):**
  - **F4:** Paradigm 3's λ 0.6–0.9 comes from one task.
  - **F5:** the post's lexicon tables hold 18 rows, though it says 19, and "eight" should be "seven". The README's 19 has `admit` as its own row.
  - **F6:** √(SC/C) is the kill bar, not the ceiling.
  - **F7:** the Deleuze & Guattari line isn't on the linked page; it is the close of plateau 14.
  - **F8:** "eight role threads" (#145 plus seven inboxes); "dormant inboxes" (four identities with no posts); 702 lines is `MODE.RATIONALE.md`; dead-ends now has 13 rows; cite #931 for the floor result.
- **Simulated reader:** questions 1–4 and both held-out questions (6, 7) are answerable from the post alone. The transfer question (5) is only mostly answerable: the reader must assemble the sequence from §3 and §6. One sentence at the end of §3 would close it.

**What changed:**
- Added `docs/journal/2026-11-26-rhizome/the-frontier-review.md`, with the findings, their sources and the fixes.
- Updated `frontier/blog/the-frontier.toml`: `stage = "review"`, `reviewed_by = "claude-code"`, and the review added to `links`.
- Draft untouched (fixes belong to `revise`).

**What's next:**
- **`revise`:** apply F1–F8 and the question-5 sentence, rerun `check_prose.py` and `link_audit.py`, and repin the Sources links past #952.
- **`ready`:** move the post to `docs/site/blog/2026-11-26-dropping-the-coordinator.md`, wire nav, index, llmstxt and the social card, and run `publish-page`.
- **Optional:** cut §3's hybrid-system table to prose; the post runs 21 minutes.

**Caveats:**
- Taken without a hold: the live-hold budget was full (3 of 3), and the README says review needs none.
- I did not render the figures or run `check_prose.py`. The figure script draws no D value, so F1 changes no figure.
