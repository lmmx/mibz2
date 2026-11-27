# Branch protection for master

This is the exact GitHub ruleset for `master`, `master-ruleset.json` beside this file. It puts
into GitHub's settings what Louis approved on 26 November 2026
(`frontier/deltas/2026-11-26-owner-decisions.md`), and nothing beyond it. Applying it needs an
administrator credential on `lmmx/mibz2`, which no agent session holds.

## What it does

| Rule | Setting | Why |
| --- | --- | --- |
| Deletion | blocked | Approved: no deletion of master. |
| Force push (`non_fast_forward`) | blocked | Approved: no force pushes to master. |
| Pull request | required, 0 approvals, no CODEOWNERS, squash only | Approved: PRs to master, with no human or second-account approval while agents share the owner's identity. Every master commit in the available history is a GitHub squash. |
| Required checks | `no-ai-attribution`, `check-commit` (both pinned to GitHub Actions, app 15368), `pre-commit.ci - pr` | The existing gates that run on every PR, finish in about a minute, and that CLAUDE.md never lets a merge skip. |
| Up to date with master (`strict`) | off | Requiring it would force a rebase and a re-run of CI on every merge: process latency the CI rules forbid. |
| Bypass | GitHub Actions only (app 15368), no administrator bypass | Deploy Release (`.github/workflows/deploy.yml`, `just release`) pushes its version-bump commit straight to master as `github-actions[bot]`; without this the release would fail. Approved: no routine administrator bypass. |

It targets `refs/heads/master` alone, so `hold/*`, `evidence`, `frontier-admission`,
`bench-results`, `bench-data`, `gh-pages` and the `ready/*` and `hold/*` tags are untouched.

## What it deliberately leaves out

- **The heavy suites** (rust, fuzz-smoke, python, typing, the wheel and sdist builds, Miri, the
  bench). CLAUDE.md's "When not to wait for CI" lets a queue-only PR, a PR whose content already
  went green, and a PR that can't reach a job merge without them. A required check can't be
  waived case by case without an administrator bypass, so requiring these would remove rules
  Louis made. They stay gates by convention, as now.
- **macOS jobs:** they never hold a merge (Louis, 18 November 2026).
- **The site build and `look`:** `site.yml` runs only on some paths, and a required check that
  never reports blocks the PR for ever; blog-only PRs also don't wait for it.
- **The frontier check.** Two reasons. Louis's condition: only after its known failures are
  fixed and independently verified. And a mechanical one: its job is named `check`, and every
  PR also gets skipped `check` runs from the `pull_request_target` event of the same workflow
  (PR #995's head `81527e94` has two successful and three skipped), and a skipped run satisfies
  a required check. Rename the job (to `frontier-check`, say) before requiring it.

## What changes for agents

- Nothing pushes to master except a merge of a PR, or Deploy Release.
- `just ship` (`.just/ship.just`) pushes a release commit straight to master. It is no longer
  the runbook's path (`docs/RELEASING.md` uses Deploy Release) and will be refused; run it, if
  ever, on a branch through a PR.
- RELEASING.md §5's "commit that to master" becomes a PR, as every other change is.
- A history rewrite of master (as on 26 November) needs the ruleset disabled for the rewrite,
  by an administrator. That is the point.

## Applying and checking it

With an administrator token:

```bash
gh api -X POST repos/lmmx/mibz2/rulesets --input docs/ops/master-ruleset.json
gh api repos/lmmx/mibz2/rules/branches/master   # lists the four rules
```

If GitHub refuses the GitHub Actions bypass on this personal repository, don't drop the release
path: change Deploy Release to open and merge a PR for its bump commit first, then apply the
ruleset without a bypass.

Then check that a PR still merges once its three checks pass, and that a direct
`git push origin HEAD:master` is refused. To withdraw it:
`gh api -X DELETE repos/lmmx/mibz2/rulesets/<id>`.
