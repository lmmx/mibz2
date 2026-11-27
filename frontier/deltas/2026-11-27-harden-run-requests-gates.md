claim: declarative-ready-signal
by: antigravity
outcome: partial
evidence: #1104 review 6084603749 (at 01181d78); bench/run_requests.py; bench/test_run_requests.py (62 tests)

Hardened `bench/run_requests.py` against four security and protocol defects empirically reproduced during independent O2 review of `#1104` (`01181d78`, merged as `1d1b23df` before review findings were addressed).

1. **Whole-batch DoS via non-blob `request.toml` (`World._show`, `World.requests`):** replaced `cat-file -s` + `cat-file blob` with `git ls-tree -l` validating regular-file mode (`100644`/`100755`), `blob` type, exact path match, and size `<= limit`; narrowed fetch to validated `REQUEST_REF` refs; isolated per-ref metadata failures so one malformed ref cannot crash reconciliation for other recipes.
2. **Fail-open completed non-passing Actions suites (`check_ci`):** completed `github-actions` suites now require `conclusion in PASSED` (or `cancelled` with non-empty suite runs all excused by passed twins); `startup_failure`, `failure`, `timed_out`, `action_required`, `None`, and zero-run `cancelled` suites fail closed.
3. **Fail-open skipped/neutral required checks (`check_ci`):** `REQUIRED_CHECKS` (`check-commit`, `no-ai-attribution`) now require `conclusion == "success"` specifically (rejecting `skipped` and `neutral`, while still excusing a superseded `cancelled` twin when a `success` run exists).
4. **Frontier authority and PR gates (`evaluate_ready`, `World.open_flags`):** reject parked leases (`phase not in (None, "integration")`), check open/malformed `frontier/flags/*.md` on the default branch (matching claim, recipe, or overlapping owned/premise paths), and reject draft PRs (`draft is not False`) or non-default PR base branches (`base.ref != world.default`).

Added 9 regression test methods (22 subtests) in `bench/test_run_requests.py` (`SecurityHardening`); left open for independent O2 review before merge.
