claim: none
by: claude-code
outcome: done
evidence: #995 head 81527e94 (green: every check run and pre-commit.ci); squash 97bd4416 on master; coord.py head-sha before and --merged after

**What was asked:** `swarm`: finish eligible integrations before starting new work.
**What I found:**
- **The integrations queue at 00:2x on 27 November.** #1016, #1012, #1010 and #979 all have live producers. Each pushed within minutes of this session: #1016 to 6ad3c5d, #1012 to 480febe, and #1010 and #979 to new heads. None was handed over with `status = "review"`, so I left them to their holders rather than race them.
- **#995** (unclaimed; relevant-CI selection plus author-owned integration) had been still for 20 minutes. Its body asked for an independent review under the old policy. I did that review cold from the diff:
  - *`ci_scope.py` fails safe:* an unknown or mixed path, an empty diff, a missing or zero base, a failed fetch, a manual event or a tag all select the full suite. The self-tests pass.
  - *No gated job reads a lightweight path:* the `src/` hits are comments; `tests/test_frontier.py` uses its own fixtures and runs in `frontier.yml`, which is ungated; pytest's `testpaths` is `python/`.
  - *The jobs' conditions are sound:* GitHub implies `success() &&` on each new `if:`.
  - *The trial merge with master* (22 commits ahead of its base, touching frontier.py, README and CLAUDE.md) is clean. `frontier.py check` gives 0 errors and the one expected protected-paths warning; `tests/test_frontier.py` passes 41/41; master's `limits.toml` interlock is kept.
  - *Authority:* the rule change rests on the owner's words, recorded verbatim in `decisions.md` (26 and 27 November). It narrows review; it widens no spending, credentials or release authority, and RULES D is unchanged.
**What changed:** merged #995 as 97bd4416. From now on, ordinary integration is the author's own in either zone (RULES O1/O2, frontier/README.md "Merging").
**What's next:**
- **#979** edits the same entrypoints (CLAUDE.md, frontier/README.md) and will now conflict. Its author reconciles it against 97bd4416, keeping both policies.
- **Under the new rule,** #1012, #1016 and #1010's producers can merge their own PRs once their heads are green.
- **Small follow-up:** class `frontier/limits.toml` as lightweight in `.github/ci_scope.py`. Today it selects the full suite, which is conservative but slow.
**Caveats:**
- **Not yet exercised live:** the scope selector has not run on a prose-only PR. This delta's own PR is the first such run; check that its `rust` job shows as skipped.
- **The green run predates master's last 22 commits.** It was taken on #995's head, not on the squash.
