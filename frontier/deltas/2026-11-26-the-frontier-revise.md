claim: the-frontier
by: claude-code
outcome: done
evidence: docs/journal/2026-11-26-rhizome/the-frontier-draft.md; docs/journal/2026-11-26-rhizome/the-frontier-review.md; frontier/deltas/2026-11-26-owner-decisions.md (41088076f); docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md (#952)

**What was asked:** Move blog card `the-frontier` from `review` to `revise`, applying the review's findings. The work was done under hold `hold/the-frontier`.
**What I found:** Every review finding applied cleanly. Two more facts landed while I worked, and both went in:
- **The maintainer's decisions (41088076f):** branch protection is approved but not yet applied, and the three-arm experiment is deferred until a capped pilot is proposed. The draft had said the experiment "awaits spending approval", which is no longer true.
- **The rest of #952's cold reread:** about 55–65 ms of `E_consume`'s 298 ms is oracle-store DRAM traffic. The pooled scan's corrected capture at 8 workers is about 0.73 against a bar of about 0.78, so its KILL stands. §2 now says so, as an instance of the post's own argument.
**What changed:** the draft only (still in `docs/journal/`, not the site):
- **F1:** D is now 29.1 / 34.0 / 44–78 ms (1, 8 and 10 workers), and §5 no longer says "48 ms flat".
- **F2:** the invariants column is now "How the files keep it", and protection is described as a warned rule with binding protection approved.
- **F3:** the stress-test failures are named exactly, and the walkthrough says the tool doesn't inspect `premises`.
- **F4:** the λ range is scoped to the one task it comes from.
- **F5:** the lexicon is 3 + 7 + 7 + 2 = 19, with `admit` as its own row.
- **F6:** the gloss now gives √(SC/C) as the kill bar against the room SC/C.
- **F7:** the Deleuze & Guattari line is quoted and attributed to its plateau.
- **F8:** the counts, the inboxes, the 702-line rationale, 13 dead-end rows and #931.
- **Question 5:** one paragraph at the end of §3 answers it.
- **Sources:** repinned to f88024a9a, with the cold-store reread and the owner decisions added.
- `check_prose.py`: body 10% flagged (target ≤ 15%); dek PASS. `link_audit.py` §2: none.
**What's next:** `ready`, which needs `shell`:
- move the draft to `docs/site/blog/2026-11-26-dropping-the-coordinator.md`;
- re-run the figure script's `--inject`;
- wire nav, the blog index, `llmstxt` and a social card;
- run `publish-page` on the final head, and look at the rendered page.
**Caveats:**
- The reviewer also did the revise. The board requires only that review ≠ draft (`drafted_by` = antigravity), so `ready` would benefit from a third reader.
- `link_audit.py` §1 still lists six "used before explained" hits, all table-header words (State, Flows…). They predate this pass and look like false positives.
- The 21-minute read is unchanged.
