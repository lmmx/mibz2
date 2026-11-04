# Compact30 review queue

Every checkpoint or correction awaiting an independent review (PROTOCOL.md §8). An item is
PENDING until a pinned review is published. Silence is never approval.

| item | pin | PR | review needed | status |
|---|---|---|---|---|
| prototype 2, descriptor arrays | `29dd071f` | #151 | checkpoint audit; the width of the [54.55 M, 62.28 M] interval; the trade of extra writes in `refill` | **PENDING** |
| prototype 1, F1/F2 correction | `a1d110c7` | #146 | re-review of F1 and F2 | **PENDING** |
| prototype 1, U1/U2 trial (U2 only) | `49cb46b` | #146 | checkpoint audit: the U2 safety argument and the MU1–MU3 controls; [84.04 M, 84.06 M]; the extra writes (+3.39 M) | **PENDING** |
| §6.2 pre-integration review | the index to come | — | a fresh, independent reviewer with pinned evidence only, covering prototype 1 (U2), prototype 2 and v3's §7(a) safety evidence | **due once the coordinator publishes the evidence index** |
| refill-boundary proposal (Astra) | `0ac2e67` | #148 | the coordinator's assessment against prototype 2's listing; then the user's scope decision | **PENDING** (coordinator) |
