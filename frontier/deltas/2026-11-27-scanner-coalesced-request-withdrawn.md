claim: scanner-coalesced-u8-i9-queue
by: codex
outcome: partial
evidence: #1105, #1120 squash ebb2dd9f, A6b evidence b87790e5, request 685b347c, withdrawal 2e1e9ffd

The single static queue row landed in #1120 and the verified lease published request `run-request/bench-scanner-coalesced/edb114aad561`. The A6b result then showed that the i9 checkout is shallow. The coalesced recipe reads its historical first parent and registration from that checkout and relies on fresh local clones to expose its source pins. It cannot guarantee those objects exist on this runner.

The request was withdrawn before any ready tag or receipt appeared. The row stays dormant. The next queue holder checks the withdrawal, tags and evidence, then uses the independently reviewed green successor from `scanner-coalesced-shallow-source`. The exact #1034 squash, first parent and registered hardware criteria remain frozen. No coalesced measurement was observed.
