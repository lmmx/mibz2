# Coordinator's replay and archive of the §6.2 review (charter §6.2(3))

**Source:** the independent reviewer's publication, `review/compact30-s62` @ `e62fc15f91311823ab0c48a6f6e9c7dd35a28d10` (draft PR #153, REVIEW comment 6634356048). The files in this directory are byte-identical copies of that commit's `docs/journal/compact30-s62-review/`, archived here so the review survives independently of its branch.

**Replay, run by the Coordinator on 2026-11-04 from a checkout holding the pinned commits:**

```sh
python3 fetch_evidence.py --repo /path/to/mibz2   # Verified 132 pinned blobs.
python3 replay.py --output replay-results.json
python3 audit_figures.py --replay replay-results.json --output figures-replayed.json
```

**Result:** the replayed figures are identical to the archived `figures.json`.

| item | interval replayed (instructions removed) | profile-only terms (S62-F01) | own writes |
|---|---|---|---|
| scanner U2 @ `49cb46b6` | [84,043,218, 84,060,421] | +4,359,203 / −17,203 | +3,391,432 |
| descriptors @ `29dd071f` | [54,550,306, 62,277,795] | +1,173,383 / −3,279,857 | +5,004,156 |

The demonstrated overlap (S62-F02) is 2,393,313. The caller adds 868 instructions per harness (S62-F04).

**Disposition:** findings S62-F01 to F04 are accepted (ACK 6634430380). Gate 1 is PASS for items 1 and 2, and item 3 (v3) passes the §7(a) safety review. Gate 3 for items 1 and 2 stays on HOLD until the §5.3(b) counters land and the ledger is replayed. The overlap enters the §6.3 combined floor. Nothing is integrated.
