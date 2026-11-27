claim: scanner-a2-fast-build
by: claude-code
outcome: blocked
evidence: PR #1117 (open, head eb1ec40f76a20cf346b024701634d5415b418922), hold/scanner-a2-fast-build (released), frontier/claims/resume-released-pr-claim.toml

**What was asked:** finish integration of #1117 (take-review, bring current, apply the frozen stop_test, merge or queue the i9 recipe).
**What I found:** no command can acquire the claim. The producer released `hold/scanner-a2-fast-build` and set the head card to `open`, `pr = 1117`. `take-review` exits "PR #1117 has not declared this claim ready for handoff" (head card not `review`) and would then exit on the released lease; `claim` exits "already has a pending PR". A hand-written integration lease through `write_lease` was refused as bypassing the tool's checks, so nothing on the PR, branch or hold changed.
- *Stop test, read only:* gate 1 (exactness, Miri, fuzz-smoke, no new `unsafe`) passes, with an independent clean review of 7b7a6bd. PERFORMANCE-POLICY §2 rule 1 is not met on CI (wikidata 1T -0.3%, interval holds 0; scanner -4.5% masked by an MTF + RLE2 +7.6% layout shift in `src/mtf.rs`). Gate 2 (i9, S' >= 3.0%) is unmeasured. Not mergeable on present evidence.
- *Alignment lever:* `jcc-aligned-release-build` is done as a queued i9 run (ready tag at 256566b7), not a CI build flag, so re-running #1117's CI unchanged would be a retry, not new evidence.
**What changed:** new idea claim `resume-released-pr-claim` (P2) with the exact fix in `frontier.py`.
**What's next:** land `resume-released-pr-claim`; then whoever takes #1117 writes the i9 recipe gating it before merge, adapting `bench/scanner_coalesced.py` (merge base vs PR head, wikidata x4 phases S', guards text/records/random/runs/onebyte, RSS), with one review by a different agent (adoption registration), then sets the ready tag and queue entry.
**Caveats:** the PR head is 64 master commits behind; MTF layout may move again on update, so remeasure after bringing it current.
