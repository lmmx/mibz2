claim: branch-protection
by: antigravity
outcome: partial
evidence: #1018 head c108886c160cd33cd3fc5f3bd5405465bddcbbc0 (all checks + pre-commit.ci green); squash b1eeb34 on master; docs/ops/master-ruleset.json; docs/ops/branch-protection.md; .github/workflows/deploy.yml; .github/workflows/frontier.yml; .github/workflows/CI.yml; .github/workflows/no-ai-attribution.yml; frontier/deltas/2026-11-26-owner-decisions.md

**What was asked:** Cold independent review and integration of #1018 (`branch-protection`, submitted at `status = "review"`), since the ruleset enforces merge gates.
**What I found:**
- Verified `docs/ops/master-ruleset.json` is valid JSON matching GitHub's repository rulesets schema and targeting only `refs/heads/master`, leaving `hold/*`, `evidence`, `frontier-admission`, `bench-*`, `gh-pages` and `ready/*` / `hold/*` tags untouched.
- Verified check contexts and app IDs directly against GitHub check-runs/status APIs on `#1018` head: `no-ai-attribution` and `check-commit` run under GitHub Actions (`app_id = 15368`); `pre-commit.ci - pr` posts a legacy commit status (`creator = null`), so omitting `integration_id` is right.
- Verified `.github/workflows/deploy.yml` runs `just release` and pushes straight to `master` as `github-actions[bot]`, so the GitHub Actions (`actor_id = 15368`) bypass is needed unless Deploy Release is converted to a PR first; `docs/ops/branch-protection.md` documents that fallback explicitly.
- Verified `.github/workflows/frontier.yml` emits both `success` (`pull_request`) and `skipped` (`pull_request_target`) `check` runs on every PR, confirming `check` must be renamed (e.g. `frontier-check`) and verified before being required, matching the 26 November owner condition.
- Verified `.just/ship.just` and `docs/RELEASING.md` §5 references, zero AI attribution across commits/PR metadata, and 0 errors / 0 warnings on `frontier.py check`.
- Caught one post-merge bookkeeping gap: `#1018` left `branch-protection.toml` at `status = "review"`, `pr = 1018`, `needs_louis = ""`, which would create a stale PR pointer after merge and hide the admin application step from `#944`.
**What changed:** Took review (`take-review`), merged `#1018` at exact head `c108886c` as `b1eeb34` with `coord.py head-sha` before and after, and released `hold/branch-protection` via `finish-review`. Updated `frontier/claims/branch-protection.toml` to `status = "blocked"`, `pr = 0`, and populated `needs_louis` with the exact `gh api -X POST` command and recommendation.
**What's next:** Louis applies `docs/ops/master-ruleset.json` with an administrator token and confirms a direct push to `master` is refused, then marks `branch-protection` done. Separate follow-on: rename `frontier.yml`'s `check` job before requiring it.
**Caveats:** As noted in `#1018`, no agent holds an admin token to test `POST /repos/lmmx/mibz2/rulesets`; GitHub Actions bypass acceptance on a personal repo is verified at apply time.
