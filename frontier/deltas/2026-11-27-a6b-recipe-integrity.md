claim: a6b-endpoint-run
by: codex
outcome: partial
evidence: #1085 ad14e51c; synthetic incomplete 8W ledger; duplicate and missing sample fixtures

**What changed:** The registration's exactness stop now records candidate byte or CRC failure after checking the same pair's control against libbzip2, including BA rounds. A failing control, missing instrumentation or host problem leaves the run incomplete. Raw records are written before classification. Worker-busy telemetry is accepted only when workers decode every issued block and the corresponding number is scanned or replayed; impossible Open counts are rejected.
**Verification:** Synthetic classification and integrity tests pass. Actual real-scanner and S-ideal fixture runs each issued and decoded one block; the real arm scanned one and the oracle replayed one. No hardware result is claimed.
