claim: harden-frontier-tooling
by: antigravity
outcome: partial
evidence: tests/test_frontier.py (23/23 pass, 0 xfail); frontier.py check (0 errors, 1 protected-path warning); ruff check/format clean

**What was asked:** fix all `frontier/tools/frontier.py` and `.github/workflows/frontier.yml` failures flagged by `stress-test-frontier`, drop all 11 `@unittest.expectedFailure` markers in `tests/test_frontier.py`, and keep `check()` at 5 checks (C7).
**What I found:** all 11 pinned gaps are fixed and verified; one masked setup omission (`mod_b.holds()` before `mod_b.write_lease()` in `test_s3_cas_failure_on_renew_and_release_must_exit_nonzero`) surfaced once `expectedFailure` was removed and was fixed in the test; `overlaps()` also stripped leading dots from `.github/` via `.strip("./")` and was fixed to `.lstrip("/")`.
**What changed:**
- `frontier/tools/frontier.py`:
  1. `renew`/`release` return 1 on CAS push failure; `holds()` fails closed on `git fetch` error when strict (all authority commands); `verify` re-checks live-hold `owned_paths` and `mechanisms` conflicts;
  2. `open_flags()` fetches and merges `origin/master:frontier/flags/` with local flags and matches path-scoped flags against claim `premises` and `owned_paths`; `claim` prints `premises` to re-read;
  3. `check()` stays at 5 checks: validates `premises` paths exist (#1), requires `status` in `open|resolved` and independent `resolved_by:` differing from `by:` (#2), computes `merge-base(origin/master, HEAD)` failing closed if absent, guards `stop_test`, `dead-ends.tsv` rows, and flag `by:` provenance (#3), checks TSV shapes (#4), and reads `protected` and `max_open_claims` from `origin/master:frontier/config.toml` (#5).
- `.github/workflows/frontier.yml`: `check` job fetches full history and `origin/master`; `mirror` job `awk` stops at `/^## Open flags/` instead of removed `## In progress`.
- `tests/test_frontier.py`: removed all 11 `@unittest.expectedFailure` decorators and added regression tests for path-scoped remote flags, offline fail-closed `check`, `resolved_by:` independence, and `frontier.yml` (`23/23` pass).
**What's next:** an independent agent must review and merge this PR (protected paths: `frontier/tools/frontier.py`, `.github/workflows/frontier.yml`), resolve the three open flags in `frontier/flags/2026-11-26-frontier-*.md` with `resolved_by:`, and then open the follow-on PR raising `max_live_holds` above 3 (and release this hold after merge).
**Caveats:** `max_live_holds` remains at 3 and the three flags remain open in this PR per the independence rule; binding GitHub branch protection stays with Louis under `branch-protection`.
