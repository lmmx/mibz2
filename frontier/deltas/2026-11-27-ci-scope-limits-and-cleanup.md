claim: none
by: antigravity
outcome: done
evidence: #995 (97bd4416); #974 (3ced2be); #977 (db83e59); frontier/deltas/2026-11-26-tooling-flags-resolved.md; .github/ci_scope.py --self-test

**What was asked:** Follow up on #995's post-merge checklist and clear stale pointers surfaced by `frontier.py brief`.
**What changed:**
- Added `frontier/limits.toml` to `lightweight()` and `self_test()` in `.github/ci_scope.py` (noted in #995 and #1017; matches `frontier/README.md`'s lightweight operational ceiling rule so cap updates skip heavy decoder/packaging suites).
- Dropped the `AGENTS.md`/`GEMINI.md` wording sync after merging `origin/master` because #1019 already synced their protected-path merge wording on `master`.
- Marked `frontier/claims/harden-frontier-tooling.toml` `status = "done"` (#974 merged at `3ced2be`, all three flags resolved, 0 `expectedFailure` left, lease already released) to clear the stale `#974` pointer in `brief`.
- Updated the archived pointer `.claude/skills/write-blog-post/SKILL.md:407` from `docs/team/SPEC.OBSERVER.md` to `docs/team/archive/SPEC.OBSERVER.md` (noted in `2026-11-26-swarm-integrate-977-996.md`).
**What's next:** Fix stranded non-integration lease reconciliation in `frontier/tools/frontier.py` (separate PR) so `hold/archive-team-strata` releases cleanly and unblocks `migrate-rules`.
**Caveats:** Touching `.github/ci_scope.py` intentionally triggers the full CI suite on this PR to validate the updated classifier live.
