claim: ci-bench-open-route-arm
by: antigravity
outcome: done
evidence: PR #1245 (d03b8565); .github/workflows/bench.yml; bench/test_report.py (25/25); live CI Benchmark (+43 s, wikidata 1T 0.996 [0.990, 1.002], text 1T 1.023 [0.988, 1.060], same bytes: yes)

**What was asked:** Add a diagnostic paired forced-open-route cell (`mibz2-workers-lazy`, 1 thread, `wikidata` and `text`) to `.github/workflows/bench.yml` so PRs touching the open kernel receive a fixed-runner paired head-vs-base measurement outside `PERFORMANCE-POLICY.md` §2's acceptance rule, leaving existing cells and the acceptance rule byte-for-byte unchanged.
**What changed:**
- Added an `Open-route diagnostic` step right after `Stages` in `.github/workflows/bench.yml` running `bin/ab run open-route.jsonl --arms "mibz2-workers-lazy,mibz2-workers-lazy@base/ab" --threads 1 --spread --rounds "$ROUNDS" --settle "$SETTLE" corpora/wikidata.bz2 corpora/text.bz2` in PR mode (degrading to head-only if an older base lacks `mibz2-workers-lazy`, and head-only on `master`/`dispatch`), timing its wall duration into `OPEN_ROUTE_SECS`.
- In `Figures`, formatted `open-route-summary.md` via `bin/ab summary open-route.jsonl --ref "$OPEN_ROUTE_REF"` under a dedicated `### Open-route diagnostic (forced open route, 1 thread)` heading explicitly marked diagnostic only (outside `PERFORMANCE-POLICY.md` §2's acceptance rule, with added wall seconds), appended to `comment.md` (appearing in both the PR comment and `$GITHUB_STEP_SUMMARY`) and printed to the step log.
- Added `open-route.jsonl` and `open-route-summary.md` to the `Results` artifact upload.
- Left `Run` (`runs.jsonl`), `bench/report.py`, `result.json`, `Record`, all primary cells, and the `PERFORMANCE-POLICY.md` §2 acceptance rule 100% byte-for-byte unchanged.
**Verification:** YAML parse (`ruby -ryaml`) + `bash -n` on all 23 steps PASS; `python3 bench/test_report.py` 25/25 PASS; live CI `Benchmark` run on PR #1245 (`d03b8565`) passed end-to-end as its first measured PR: added wall time `43 s`, `wikidata 1T` `0.996 [0.990, 1.002]` (`±0.6%` half-width, `same bytes: yes`), `text 1T` `1.023 [0.988, 1.060]` (`same bytes: yes`); all CI checks + `pre-commit.ci - pr` green; `frontier.py check` 0 errors.
**Integration:** Non-gating diagnostic cell with zero changes to workflow permissions, triggers, safety, authority, or acceptance-rule enforcement — integrated autonomously under CLAUDE.md ordinary-integration rules (RULES O1/O2). Every subsequent open-kernel PR now automatically receives a paired CI open-route cell.
