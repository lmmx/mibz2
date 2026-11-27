claim: ci-bench-open-route-arm
by: antigravity
outcome: partial
evidence: PR #1245; .github/workflows/bench.yml; bench/test_report.py (25/25); ruby -ryaml + bash -n (23/23 steps)

**What was asked:** Add a diagnostic paired forced-open-route cell (`mibz2-workers-lazy`, 1 thread, `wikidata` and `text`) to `.github/workflows/bench.yml` so PRs touching the open kernel receive a fixed-runner paired head-vs-base measurement outside `PERFORMANCE-POLICY.md` §2's acceptance rule, leaving existing cells and the acceptance rule byte-for-byte unchanged.
**What changed:**
- Added an `Open-route diagnostic` step right after `Stages` in `.github/workflows/bench.yml` running `bin/ab run open-route.jsonl --arms "mibz2-workers-lazy,mibz2-workers-lazy@base/ab" --threads 1 --spread --rounds "$ROUNDS" --settle "$SETTLE" corpora/wikidata.bz2 corpora/text.bz2` in PR mode (degrading to head-only if an older base lacks `mibz2-workers-lazy`, and head-only on `master`/`dispatch`), timing its wall duration into `OPEN_ROUTE_SECS`.
- In `Figures`, formatted `open-route-summary.md` via `bin/ab summary open-route.jsonl --ref "$OPEN_ROUTE_REF"` under a dedicated `### Open-route diagnostic (forced open route, 1 thread)` heading explicitly marked diagnostic only (outside `PERFORMANCE-POLICY.md` §2's acceptance rule, with added wall seconds), appended to `comment.md` (appearing in both the PR comment and `$GITHUB_STEP_SUMMARY`) and printed to the step log.
- Added `open-route.jsonl` and `open-route-summary.md` to the `Results` artifact upload.
- Left `Run` (`runs.jsonl`), `bench/report.py`, `result.json`, `Record`, all primary cells, and the `PERFORMANCE-POLICY.md` §2 acceptance rule 100% byte-for-byte unchanged.
**Verification:** YAML parse (`ruby -ryaml`) + `bash -n` on all 23 steps PASS; `python3 bench/test_report.py` 25/25 PASS; local `ab summary` formatter dry-run confirmed clean GFM tables with paired wall/CPU ratios, 95% CIs, peak RSS, and byte parity; `frontier.py check` 0 errors. Live CI `Benchmark` run on PR #1245 verifies end-to-end execution and records exact runner added wall time on the PR.
**What's next:** Protected `.github/workflows/bench.yml` change (RULES O2) — PR #1245 is at `status = "review"` with active lease held awaiting independent review and integration.
