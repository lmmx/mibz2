claim: stress-test-frontier
by: antigravity
outcome: done
evidence: tests/test_frontier.py (21 tests: 10 pass, 11 xfail); bench/test_run_queue_multi.py (8/8); bench/test_workstation_queue.py (4/4); CI run 45843167366; issue #944; gh api master/protection (404)

**What was asked:** attack the frontier protocol across all eight scenarios before raising `max_live_holds` above 3, automate machine-checkable checks, and flag any FAIL.
**What I found:** **4 PASS, 4 FAIL** — `max_live_holds` stays at 3 until the tool and workflow fixes land:
1. **Same-id claim race:** PASS — `--force-with-lease` on `hold/<id>` yields exactly one winner under concurrency; budget/state/`needs_louis` guards hold.
2. **Lapsed takeover by CAS:** PASS — expired leases chain cleanly on `hold/<id>` and concurrent takeovers yield exactly one winner.
3. **Lapsed holder verify/renew/release:** FAIL — sequential lapsed `verify`/`renew` and post-takeover `release` are refused, but `renew` (`frontier.py:469`) and `release` (`frontier.py:476`) exit 0 on CAS push failure, and `holds()` (`:116`) ignores `git fetch` errors so `verify` exits 0 offline against stale refs.
4. **Invalidating result (premises + flags):** FAIL — local flags hold named claims, but `open_flags()` (`:168`) reads only local disk, never `origin/master:frontier/flags/`, so flags merged to `origin/master` do not block un-rebased `verify` or `claim`; `premises` are never inspected anywhere in `frontier.py`.
5. **Cold review contradicts wrong argument:** PASS — append-only deltas and flags surface contradictions in `STATUS.md`/`brief` (caveat: `check` does not verify `resolved_by:` on resolved flags).
6. **Gate changed after data:** FAIL — `check` misses edits to `stop_test`, deletions in `dead-ends.tsv`, and `bench/*.frozen`; `.github/workflows/frontier.yml:26` uses default `fetch-depth: 1` (no `origin/master`), silently disabling check #3 in CI; locally, two-dot `git diff origin/master` false-fails un-rebased parallel branches whenever master adds a delta.
7. **Two workstation runs (`owned_paths` + `flock`):** PASS (with caveat) — sequential `owned_paths` (segment-aware) and `mechanisms` overlap refuse the second claim, and `workstation/queue.lock` `flock` serialises runners (`bench/test_run_queue_multi.py` 8/8, `test_s7_workstation_flock_serializes_concurrent_runners`). Caveat: concurrent claims on distinct ids have a cross-ref TOCTOU race unchecked by `verify`.
8. **Loosening a safety rule (protected paths + no self-merge):** FAIL — local `check` warns, but (a) clearing `protected = []` in `frontier/config.toml` silences `check` in the same PR (reads local config, not `origin/master`), (b) CI shallow checkout drops the warning (PR #946 run `45843167366` touched 7 protected files: `0 errors, 0 warnings`), and (c) `master` has no GitHub branch protection (`404`, `rulesets: []`).
- Bonus CI bug: `.github/workflows/frontier.yml:49` `awk` stops at removed `## In progress`, dumping all of `STATUS.md` onto issue #944.
**What changed:**
- added hermetic test suite `tests/test_frontier.py` (uses temporary bare remotes; zero debris refs on `origin`);
- opened 3 flags: `2026-11-26-frontier-stale-authority.md`, `2026-11-26-frontier-remote-flags-and-premises.md`, `2026-11-26-frontier-check-and-ci-gaps.md`;
- closed `stress-test-frontier` (`done`) and opened P1 striated claim `harden-frontier-tooling` (`open`, keeping open count at 12).
**What's next:** take `harden-frontier-tooling`, fix `frontier/tools/frontier.py` and `.github/workflows/frontier.yml`, drop all 11 `@unittest.expectedFailure` markers in `tests/test_frontier.py`, resolve the 3 flags via an independent reviewer, then open the reviewed PR raising `max_live_holds`. Binding non-self-merge remains on `branch-protection` (needs Louis).
**Caveats:** `tests/test_frontier.py` pins the 11 failing assertions with `@unittest.expectedFailure` so CI stays green while every gap stays executable and falsifiable.
