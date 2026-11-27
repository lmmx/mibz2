flags: frontier/tools/frontier.py frontier/config.toml
invariant: traceability
by: antigravity
status: resolved
resolved_by: claude-code

**Evidence:** `tests/test_frontier.py` (`test_s4_remote_flag_on_origin_master_must_block_unrebased_claim_and_verify`, `test_s4_check_validates_claim_premises_exist`). In `frontier/tools/frontier.py`: (1) `open_flags()` (lines 168-174) reads only the local working tree (`ROOT / "flags"`), never `origin/master:frontier/flags/`, so an open flag merged to `origin/master` fails to block `verify` or `claim` from any un-rebased checkout; (2) `premises` is never read by `check`, `brief`, `claim`, or `verify` (even nonexistent premise paths produce 0 errors and 0 warnings); (3) path-scoped flags (`flags/README.md`) are not matched against claim `premises` or `owned_paths`.
**Scope:** Do not raise `max_live_holds` above 3 in `frontier/config.toml` until resolved.
**What would resolve it:** A reviewed PR on `frontier/tools/frontier.py` fetching and checking open flags from `origin/master:frontier/flags/` (merged with local flags) in `claim` and `verify`, matching path-scoped flags against claim `premises`/`owned_paths`, validating `premises` paths in `check` (and surfacing changed premises in `brief`/`claim`), and promoting the `expectedFailure` tests in `tests/test_frontier.py` to passing tests.
**Resolution (26 November 2026):** met by #955 and #974 (3ced2be). `open_flags()` reads `origin/master:frontier/flags/` merged with local flags and matches path-scoped flags against `premises`/`owned_paths`; `check` validates `premises` paths. Both named s4 tests pass on master; no `expectedFailure` remains (36/36).
