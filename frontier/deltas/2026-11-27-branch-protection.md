claim: branch-protection
by: claude-code
outcome: partial
evidence: docs/ops/master-ruleset.json; docs/ops/branch-protection.md; frontier/deltas/2026-11-26-owner-decisions.md; PR #995 head 81527e94 check runs; .github/workflows/deploy.yml; .just/ship.just

**What was asked:** Make the approved branch protection for master binding (Louis, 26 November: PRs required, existing gates, no force push or deletion, no approval bottleneck, no routine admin bypass).
**What I found:**
- master has no ruleset or protection today (`gh api repos/lmmx/mibz2/rulesets` returns `[]`).
- Deploy Release (`just release`) pushes its bump commit straight to master as github-actions[bot], so a PR-only rule needs a GitHub Actions bypass (app 15368) or the release breaks.
- The frontier job is named `check`. Each PR also gets skipped `check` runs from the workflow's pull_request_target event (PR #995's head: 2 success, 3 skipped), and a skipped run satisfies a required check. So requiring it by name would enforce nothing, quite apart from Louis's condition that its failures be fixed first.
- Requiring the heavy suites would override CLAUDE.md's "When not to wait for CI" cases, since a required check can't be waived per PR without admin bypass. So they stay conventional.
- Every master commit in the clone's history is a GitHub-made squash. `just ship` is the only local recipe that pushes master directly, and RELEASING.md no longer uses it.
**What changed:** Added the exact ruleset `docs/ops/master-ruleset.json` and its rationale and apply/verify/withdraw steps, `docs/ops/branch-protection.md`. It blocks deletion and force pushes, requires a squash PR with 0 approvals and the checks `no-ai-attribution`, `check-commit` (pinned to Actions) and `pre-commit.ci - pr`, with strict up-to-date off and no admin bypass. Claim at review.
**What's next:**
- A different agent reviews it, since it enforces gates.
- Louis applies it with one `gh api -X POST` (admin credential; no agent holds one) and confirms a direct push is refused.
- Follow-on idea: rename frontier.yml's `check` job before it can be required.
**Caveats:** Not applied and not tested against the API. Whether GitHub accepts the Actions bypass on a personal repository is unverified; the doc gives the fallback. The settings are infrastructure, so Louis applies them; this PR changes no workflow.
