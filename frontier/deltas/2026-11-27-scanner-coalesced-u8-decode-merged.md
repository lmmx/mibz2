claim: scanner-coalesced-u8-decode
by: claude-code
outcome: done
evidence: #1034 squash 5ed20c1d (head 782f4feb); Miri workflow run 45946292022 at ffc519b9; CI bench run 45921211004

**What was asked:** integrate #1034 after the independent code and B5 review (2026-11-27-scanner-coalesced-u8-decode-review.md).
**What I found:** every gate passed on head 782f4feb. All check runs and pre-commit.ci were green. `coord.py head-sha` passed before and after the merge. The Miri workflow (dispatched, since its path filter misses the scanner files) was green at ffc519b9, the same code with only the claim file differing.
**What changed:** #1034 merged as 5ed20c1d. The reservation is released with `finish-review`. This claim is done for its build. Its workstation done_when (the i9 kill test under the frozen registration) moves to `scanner-coalesced-u8-i9-run` (P1, idea), which needs a recipe first.
**What's next:** take `scanner-coalesced-u8-i9-run`: write the recipe to registration §3.1, get it green and read, then set the ready tag and queue it. `scanner-a2-fast-build` (its dependency was #1034) can now be admitted.
**Caveats:** this is CI evidence only (PERFORMANCE-POLICY §2). Keeping or reverting the change waits for the i9 run.
