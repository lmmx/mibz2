# R1: retain failed child process measurements

Assignment363/6821674655; independent review145/6821626797 at
7b53ded9eb139fdd69b4c29565e7373ac95e90d4. One appended targeted correction.

The fake-Popen/wait4/clock regression asserts wall150ns, user0.1s,
system0.2s, total CPU0.1+0.2s and RSS321KiB. These are fixtures, not timings.
It checks the flushed sidecar already exists at Popen context exit, before JSON
parsing or exception-handler cleanup, then checks final command/job/PID/raw
wait-status/exit-code/validation and retained stdout/stderr for successful exit,
exit7, malformed JSON and decoder-error JSON. A separate spawn test requires null
PID/status/resources and no wait4 call. Five run-level failure subcases (the three
rejections, spawn, phase mismatch) assert FAILED completed-count0, retained raw
files/sidecar and absence of runs.jsonl, analysis.json and COMPLETE. Existing full
129-job success wiring and real Just argument/exit checks still pass.

Actual checks (all untimed), 2026-11-12:

```sh
CELLS_LIVE_REQUIRE_JUST=1 python3 bench/cells-gap/live/test_driver.py -v
python3 docs/journal/2026-11-12-cells-live/r1/check_protection.py
python3 bench/cells-gap/live/driver.py plan
ruff check bench/cells-gap/live/{driver,test_driver}.py docs/journal/2026-11-12-cells-live/r1/check_protection.py
ruff format --check bench/cells-gap/live/{driver,test_driver}.py docs/journal/2026-11-12-cells-live/r1/check_protection.py
python3 -m py_compile bench/cells-gap/live/{driver,test_driver}.py docs/journal/2026-11-12-cells-live/r1/check_protection.py
bash -n bench/cells-gap/live/run.sh
git diff --check
```

Just1.38.0 was on PATH; Ruff used the existing uv cache. Python13/13, zero skips;
final test log0.115s is test execution duration only. Validation began11:14:31UTC
and ended11:16:17UTC (prepared-source hash verification immediately followed),
within the10-minute bound. Preparation first recorded time11:11:24UTC; the one
correction was completed within60minutes. No Rust/Miri/CLI replay is required for
this Python-only correction; prior evidence remains scoped to its original head.
No decoder, hooks, unsafe invariants or performance configuration changed.

`protection-final.json`:1509 prior tracked files untouched outside the four
explicitly allowed Python/docs files;27/27 historical367 files unchanged;
53/53 corrected365 source identities and97/97 archived digests match. PLAN is
byte-identical to prospective3b43af956d553905ea744f6cb6e1c785a0f8297a. Schedule,
interval and analyse ASTs are unchanged. `prepared-source.json` additionally
checks the existing generated source, logger, hash helper, generator and patch.
The initial protection report selected only7 historical harness files; the final
selector includes the20 historical evidence files too. Both reports are retained.
Initial static check found two missing docstrings in the new evidence helper;
those docstrings were added, final check/format pass. The initial failure remains
in static.log. Prior tests/evidence are not rewritten.

No live timing, workstation/i9, corpus replay, build, manual CI dispatch or rerun.
Ordinary automatic changed-head Actions and legacy statuses are recorded in the
completion REVIEW and own state branch. Prior Occupancy ledger failure, legacy
Class D and macOS ARM cancellation/Class D remain unresolved until Coordinator
reconciliation. No readiness, integration or new B5 certification is asserted.

The sidecar is flushed to the filesystem; power-loss durability/fsync is not
claimed. Spawn or wait failure has no collected rusage and is explicitly absent.
Sidecar validation covers exit/JSON/decoder errors; FAILED records later attempt
rejections. Existing evidence does not test injected panic unwind. Logical position
initialization is not physical stores; >=50MiB byte files remain outside bundles;
same-category off/on effects do not establish effect-size equivalence or causal
off-build savings. No estimator, gate,120+9 budget or unadmitted20+20 warm proposal
was changed. Coordinator now assesses the one correction and remaining
CI/integration/final recipe/PROTOCOL9 requirements; no second full review requested.
