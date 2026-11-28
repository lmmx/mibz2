claim: scanner-a2-fast-build
by: claude-code
outcome: blocked
evidence: PR #1117 head eb1ec40f76a20cf346b024701634d5415b418922; `frontier.py take-review scanner-a2-fast-build --pr 1117 --head eb1ec40f… --by claude-code` on 28 Nov refused ("integration handoff requires a participant different from the producer"); hold/scanner-a2-fast-build edd8a85e (released); i9 PASS origin/evidence 9dacc085 (#1252, re-verified in 2026-11-28-scanner-a2-fast-i9-run-review.md)

**What was asked:** take-review #1117 and drive it to merge now that its i9 adoption gate has passed.
**What I found:** every gate but integration ownership is in place.
- **C6:** the adoption registration (#1237) had its one independent review by a non-claude-code agent (antigravity, delta 2026-11-27-scanner-a2-fast-i9-queued.md), which also answered #1239's `trim_cell` pointer. The dependency #1239 routed is met; nothing is missing there.
- **Evidence:** the frozen §4 rule passes on all four gates (S' 0.9548 [0.9514, 0.9582], disjoint); I recomputed it from `runs.jsonl`.
- **Safety:** independent cold review of 7b7a6bd on the PR (no new `unsafe`; the BMI2 dispatch only changes `sel_buf` to `runs` under the same argument; Miri 4/4). eb1ec40f differs from 7b7a6bd only in frontier files.
- **Measured code is what merges:** the measured head 689e55f7 has zero diff from eb1ec40f on `src/block.rs`/`src/huff.rs` (checked at #1237's review). Since the measured base 2af67a67, master's code changes are `src/open/outq.rs` and #1246's `src/open` pre-scan skip, which don't touch the scanner.
- **Blocker:** `take-review` names the integrator by agent kind, and #1117's producer kind is claude-code, so no claude-code session can take it. `claim` refuses because a PR is pending. I did not pass a different `--by` or write a lease by hand, since either would be self-review in effect.
**What changed:** this delta only. Nothing on #1117, its branch or its hold.
**What's next:** a non-claude-code participant runs `take-review scanner-a2-fast-build --pr 1117 --head <head> --by <kind>`. It then merges current master into #1117, waits for exact-head CI (rust, Miri, fuzz-smoke, bench, pre-commit.ci status), and sets the card to `done`. It merges with `coord.py head-sha` before and after, `expectedHeadSha`, its own squash title/message and no AI attribution, then runs `finish-review`. CI's bench may again show the known `src/mtf.rs` layout shift; the i9 run stands in for PERFORMANCE-POLICY §2 rule 1 (registration §4), so that shift doesn't hold the merge unless the CI's outputs differ. release-0-5-18's i9 preflight isn't queued yet, so merging changes the candidate harmlessly.
**Caveats:** two ungated diagnostic cells leaned slower beyond noise (`real/records/8` 1.0076, `real/onebyte/8` 1.0288, a 12 ms decode). Watch them in the release's i9 gate.
