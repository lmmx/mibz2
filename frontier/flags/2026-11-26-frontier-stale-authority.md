flags: frontier/tools/frontier.py frontier/config.toml
invariant: exclusive ownership
by: antigravity
status: resolved
resolved_by: claude-code

**Evidence:** `tests/test_frontier.py` (`test_s3_cas_failure_on_renew_and_release_must_exit_nonzero`, `test_s3_verify_must_fail_when_origin_unreachable`, `test_s7_verify_detects_cross_id_overlapping_owned_paths_after_race`). In `frontier/tools/frontier.py`: (1) `renew` (line 469) and `release` (line 476) print `"not renewed"` / `"not released"` when `write_lease` fails CAS, yet exit 0; (2) `holds()` (line 116) ignores `git fetch` errors, so `verify` exits 0 offline against stale `refs/remotes/origin/hold/*`; (3) `verify` (lines 460-464) does not re-check overlapping `owned_paths` or `mechanisms` across live holds after a cross-id claim race.
**Scope:** Do not raise `max_live_holds` above 3 in `frontier/config.toml` until resolved.
**What would resolve it:** A reviewed PR on `frontier/tools/frontier.py` returning non-zero on `renew`/`release` CAS failure, failing closed in `holds()`/`verify` when `git fetch` fails, re-checking live-hold `owned_paths`/`mechanisms` overlap in `verify`, and promoting the three `expectedFailure` tests in `tests/test_frontier.py` to passing tests.
**Resolution (26 November 2026):** met by #955 and #974 (3ced2be). `renew`/`release` return 1 on CAS failure; `holds()`/`verify` fail closed when fetch fails; `verify` re-checks live `owned_paths`/`mechanisms` overlap. The three named tests pass on master and no `expectedFailure` remains (`tests/test_frontier.py` 36/36). Acquisition, renewal and transfer also CAS a shared admission epoch atomically (#974).
