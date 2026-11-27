claim: ci-bench-open-route-arm
by: antigravity
outcome: partial
evidence: frontier/claims/ci-bench-open-route-arm.toml; .github/workflows/bench.yml; docs/notebook/2026-11-27-synthesis-seventeenth-pass.md

Admitting `ci-bench-open-route-arm` (`idea` -> `open`, P2) and freezing its C1a `stop_test` strictly before implementation or data:

- **Diagnostic cell:** paired `mibz2-workers-lazy` at 1 thread on `corpora/wikidata.bz2` and `corpora/text.bz2` (head vs base in PR mode, head-only on master/dispatch) in `.github/workflows/bench.yml`.
- **Reporting:** paired ratios appear in a separate diagnostic section of the PR report (`comment.md`) and `$GITHUB_STEP_SUMMARY`.
- **Invariance:** all existing cells (`runs.jsonl`), `result.json`, and the `PERFORMANCE-POLICY.md` §2 acceptance rule are byte-for-byte unchanged.
- **Cost accounting:** added wall time is measured and recorded on the PR. Otherwise drop.
