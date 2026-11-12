# Completed targeted R1 correction — PR373

Dispatch363/6821674655, independent Researcher review145/6821626797 at
7b53ded9eb139fdd69b4c29565e7373ac95e90d4. Current master/policy remains
2635f04e9155dd31388b24fd3f7248f87b3ddcda. One correction appended on existing branch:
**b5131679f51c2a0245bc03f62b7afd833b8bff23** (`diagnostic/cells-live-path`).
No force push, production/hooks/unsafe changes or new experiment.

R1: checked_child flushes command/job/PID/wait-status/exit-code and collected
wall/user/system/total CPU/RSS before rejection; stdout/stderr survive. Spawn
failure explicitly retains null child/status/resources. Successful return tuple
unchanged. Phase reconciliation precedes successful-row append. FAILED retains
attempt rejection; failed observations do not enter successful paired estimates.
Sidecar validation labels only child exit/JSON/decoder-error checks, not downstream
identity or phase checks. Sidecar flush does not claim power-loss/fsync durability.

13/13 Python tests with real Just1.38.0, zero skips. Exact fake150ns/.1s/.2s/321KiB
assertions for success, exit7, malformed JSON and decoder-error JSON; flushed
resources inspected already at Popen context exit. Spawn test requires no wait4
and null resources. Five run-level failure subcases retain FAILED/raw files and
exclude runs/analysis/COMPLETE. Existing129-job successful path still passes.
No real decoder clocks, corpus replay or performance measurements. Focused local
validation11:14:31–11:16:17UTC plus immediate prepared-source hashes, under10min;
first recorded intake11:11:24UTC, completion within60min.

Ruff/check/format, Python/shell syntax and diff whitespace pass. Initial evidence
helper lint failed two missing docstrings; added docstrings, retained failed log.
Initial historical-file selector checked7 harness files; final includes20 evidence
files too,27 total. All prior evidence remains unchanged. No new Rust/Miri/build
programme: Python-only correction preserves the prior scoped evidence, not a new
B5 certification. Existing Result::Err tests do not prove injected panic recovery;
pos_initialized_bytes is logical extent, not repeated stores/physical traffic;
>=50MiB precheck byte files remain local outside bundles. Same-category off.9/on.45
is not effect-size equivalence or causal off-build savings. No new equivalence gate.

Evidence:
https://github.com/lmmx/mibz2/tree/b5131679f51c2a0245bc03f62b7afd833b8bff23/docs/journal/2026-11-12-cells-live/r1
R1 manifest SHA2560649a64596ce05ac522ac0a90136c3eb626d11474d618480985e2a4a58c6aeca.
Driver SHA2563972761a4a02eadb1ff0de00891358b8ecd89a996bf23b8239897708fd3df14c.
Tests SHA25609d14fdcbf0dd0b27fd7320046ca9f268323c3983bb4036107e26fee3885e166.
PLAN SHA2562c285d842413602379a2c3ceb3de4d8e5e50954f06a60a5d23270e3ad9f0b2a0,
byte-identical to prospective3b43af956d553905ea744f6cb6e1c785a0f8297a.
Source manifest SHA2569cfaf73721c945a429d5b884ceefb798c9f0900890c1489ea1eb17add6210b22.
1509 prior tracked files unchanged outside four allowed Python/docs files;
53 original hashes,97 archived digests,27 historical367 files verified. Schedule,
interval/analyse ASTs unchanged. Actual prepared source/logger/hash/generator/
patch hashes match. Ten pairs,120+9, source d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2,
inputs and unadmitted warm20+20 proposal unchanged.

## CI observation (single snapshot, no polling or reruns)

At2026-11-12 11:17:32UTC, exact changed head b513167:

- CI43207829260 queued (check-commit succeeded, publish skipped, platform jobs
  queued/running; macOS ARM job131147445942 queued, not validated).
- Benchmark43207829571/job131147395333 in progress.
- Cells preparation43207829271/job131147393808 in progress.
- Occupancy43207829422/job131147393835 in progress.
- Test43207829676 queued in workflow snapshot; individual jobs starting/running.
- Legacy pre-commit FAIL, Class D. Result
  https://results.pre-commit.ci/run/github/1435546475/1794482231.FWhSnWB4TjeLnaE7eIXJmg
  could not be retrieved: URLError, tunnel403 Forbidden. Cause not inferred from
  badge. No workflow/packaging repair or unrelated formatting folded into R1.

Full Actions/job/status data in ci-snapshot.json. Pending is not pass. Prior
Occupancy D100 ledger failure remains a separate reviewed368/366 integration
dependency. Prior macOS ARM cancellation/Class D remains unverified until current
platform checks reconcile. No waiver. Prior preparation/Test/Benchmark success is
at reviewed7b53, not claimed on b513167. No benchmark acceptance or readiness claim.

## Next action

Coordinator reconcile the delivered one-pass R1 correction, then assess remaining
exact-head Actions/legacy, separate reviewed368/366 integration dependency and final
CPU/recipe/PROTOCOL9 readiness. No second full review requested. No queue/tag/Run
signal, workstation/i9, merge/release/default/team/cap/gate change.365 DEFER,
371 REVISE and separate366 disposition stand. No new performance gap measured.

Direct coord.py audit remains blocked by proxy403; route REVIEW->145 passes.
Connector readback verifies delivered REVIEW; direct verify result recorded with
receipt, never claimed pass when unavailable. After REVIEW delivery, own completed
cursor processes6821674655 only; all earlier assignments preserved. GitHub comments
cannot restart a stopped Codex session; manual launch remains necessary.
