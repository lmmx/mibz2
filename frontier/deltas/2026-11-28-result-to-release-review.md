claim: result-to-release
by: claude-code
outcome: done
evidence: PR #1241 (author head 9dd1b182, antigravity); this review's fix on the same PR; tests/test_frontier.py 84/84; live brief on origin/master dc36b51d; origin/evidence ccfa473f

**What was asked:** the independent review and integration #1241 asked for (author kept the hold for an agent other than the producer).

**What I found:** the change is visibility only. It adds a `## Result-to-release` section to `brief` and `status` and merges its derived P1 into `brief`'s ordering for open claims; declared priorities, holds, review, dispatch and merge gates are untouched. Reviewed against the claim's ten frozen fixture classes:
- Evidence joins fail closed: a corrupt tarball, a `.sha256` sidecar mismatch or an ambiguous recipe join gives `unknown`, never a pass. Verdicts come only from an explicit `verdict:` header in a delta, never from prose or an archive.
- `release_due` now ignores a commit only when every PR its subject names is in `frontier/ledger.tsv` at an explicitly non-default state (experimental, prototype, target). An unledgered or default commit still makes a release due. This is the claim's fixture (9) and matches RELEASING.md section 1, whose list of what is worth releasing has no opt-in experimental features; the failure direction is the safe one (unknown counts as default).
- **One defect, fixed here:** every live `release-*` claim was reported as `released ... tagged <last tag>` with P1, even before its own tag existed. On master today that showed the blocked, untagged `release-0-5-18` as "tagged v0.5.17: awaiting post-tag release-scaling", and would have looked for its scaling bundle at the previous release's commit. A release claim is now `released` only once `v<version>` exists, and its scaling evidence is looked up at that tag; otherwise it is `release-in-progress`, not P1-derived. New test `test_r2r_08b_live_untagged_release_is_not_released`; the frozen fixtures are unchanged and pass.
- Live demonstration on current master: `forward-emission-adopt` shows as `returned-unread` for origin/evidence ccfa473f (bench-forward-adopt, run-20261128T020039Z), blocked by the live `a6c-wire-redesign` hold on src/open.rs: the next reader of that bundle is whoever can take that claim. The adaptive-scanner and forward-emission bundles the author named have since been read and are no longer listed.

**What changed:** frontier/tools/frontier.py, tests/test_frontier.py (author's work plus the release-stage fix), claim `result-to-release` marked done; the hold is released by the merge.

**What's next:** releasing this hold frees `frontier/tools/` and `tests/test_frontier.py`, so the P1 `parked-hold-moved-head` (stuck parked hold on src/crc.rs blocking `crc32-avx2-ring-rescue`) becomes claimable.

**Caveats:** neither file is ruff-format clean, as on master (not gated). `_is_ordinary_ci_claim` labels any build claim that never mentions the workstation `ordinary-ci`; it is a label, with no priority effect.
