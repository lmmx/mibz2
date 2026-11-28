claim: forward-emission-adopt
by: antigravity
outcome: blocked
evidence: origin/evidence f170c5c4 (forward-adopt/run-20261128T012417Z.tar.gz, sha256 883b87351598810f50965cae58d5e5c1f2559ec8d4dda0f87790ee7355659b30), bench/forward_adopt.py, bench/clock_modes.py

**What was asked:** read the returned physical i9 evidence bundle `f170c5c4` for `bench-forward-adopt`.
**What I found:** `f170c5c4` verified SHA-256 (`883b8735...`), compiled both distinct release binaries, passed all 34/34 SHA-256 `parity.jsonl` checks against C `libbzip2`, and then VOIDed before round 0 (zero timing samples collected) due to a schema mismatch in `check_no_quota` (introduced by `2a7ad588`): `ab env --facts/1` emits `cpu_quota` as `{"state": "unlimited", "cpus": null, "hierarchy": "v2", ...}`, not scalar `null`.
**What changed:** fixed `check_no_quota` in `bench/forward_adopt.py` to accept both `None` and `{"state": "unlimited", "cpus": None}` while still fail-closed VOIDing missing `cpu_quota`, numeric quotas, `limited`/`unknown` states, or non-null `cpus` (matching `bench/clock_modes.py:check_cpu_quota_unlimited`), with regression subtests in `bench/test_forward_adopt.py`. Zero change to the frozen C6 registration, grid, rounds, settle, arms, or decision rule.
**What's next:** merge this harness fix and move `ready/bench-forward-adopt` to the fixed master commit so the workstation queue runs `bench-forward-adopt` cleanly (the queue entry from `#1270` remains in place).
**Caveats:** zero timing data was collected in `f170c5c4`; the frozen `stop_test` and C6 registration are untouched.
