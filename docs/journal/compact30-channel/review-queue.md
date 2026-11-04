# Compact30 review queue

Every checkpoint or correction awaiting an independent review (PROTOCOL.md §8). An item is
PENDING until a pinned review is published. Silence is never approval.

| item | pin | PR | review needed | status |
|---|---|---|---|---|
| prototype 2, descriptor arrays | `29dd071f` | #151 | checkpoint audit; interval; refill write trade | **REVIEWED** (Guardian D008; Optimiser 6634221840; §6.2 S62). Gate 1 PASS; gate 3 **HOLD** on §5.3(b) counters (S62-F01, Optimiser F3) |
| prototype 1, F1/F2 correction | `a1d110c7` | #146 | re-review of F1 and F2 | **REVIEWED** (D008; Optimiser 6634221840). Accepted in scope; still nonqualifying |
| prototype 1, U1/U2 trial (U2 only) | `49cb46b` | #146 | U2 safety, MU1–MU3, interval, writes | **REVIEWED** (D008; Optimiser; §6.2 S62). Gate 1 PASS; gate 3 **HOLD** on §5.3(b) counters (S62-F01, F3) |
| §6.2 pre-integration review | index `18ca5d4c` | #153 | a fresh, independent review from pinned evidence only | **PUBLISHED** (6634356048, #153 @ `e62fc15f`). Isolated child launched under D009 (`684a3379`). Findings S62-F01 to F04 accepted (ACK 6634430380). Coordinator replay and archive pending; F01 counters and F02 overlap to resolve before §6.3 |
| refill-boundary proposal (Astra) | `0ac2e67` | #148 | the coordinator's assessment against prototype 2's listing; then the user's scope decision | **PENDING** (coordinator) |
