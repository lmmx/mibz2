# Compact30 research channel: log

Each entry gives the message verbatim, its source (GitHub comment id and time, or the user in the
coordinator's session), the charter section it relies on, and the action taken. See
[PROTOCOL.md](PROTOCOL.md).

Opened for the end-to-end test in PROTOCOL.md §7.

## T1: coordinator CHECKPOINT (dummy)

- **Source:** comment 6628984134 on #145, 2026-11-04T11:01Z, posted by the coordinator.
- **Charter section:** §13, stage 2; protocol §7.
- **Action:** none; it's a dummy checkpoint.
- **Observed:** the coordinator's own comment produced no wake event in its session, as
  expected for its own posts. The subscription itself (`subscription.created`) and CI events
  did reach the session. Waking on a comment from another party is still to be shown, at T2.

Verbatim:

> FROM: coordinator
> RE: channel test dummy checkpoint @ e0fd65b
> TYPE: CHECKPOINT
>
> **T1 of the end-to-end test (PROTOCOL.md §7).** This is a dummy checkpoint: no research action depends on it.
>
> Steps T2–T6 are posted by the user or a reviewer, one comment each:
>
> - **T2:** `TYPE: REVIEW` with one finding.
> - **T3:** `TYPE: DECISION`, `DECISION: HOLD` on this checkpoint.
> - **T4:** `FROM: guardian`, `TYPE: DECISION`, `DECISION: PROCEED`, asking for an i9 capture. Expected: refused under charter §12.3.
> - **T5:** `TYPE: DECISION`, `DECISION: PROCEED` on this checkpoint. Expected: recorded, and the T3 HOLD lifted.
> - **T6:** a comment with no header. Expected: treated as a REVIEW.
