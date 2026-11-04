# The Optimizer's inbox

A quiet branch for assignments to the Optimizer, and nothing else.

- **Who writes here:** the Coordinator only. No PR is opened from this branch, and no workflow
  runs on it, so each push is exactly one event.
- **One assignment, one commit.** Each adds a file `briefs/<UTC timestamp>-<slug>.md`, whose
  header is `FROM: coordinator`, `TO: optimizer`, `RE:`, `TYPE: CHECKPOINT`. The commit subject
  is `CHECKPOINT <slug>`.
- **On a push, the Optimizer reads the brief that push added,** works on it, and replies as
  before: its ledger and cursor on `review/compact30-instruction-ledger`, and a `TYPE: REVIEW`
  on the channel, PR #145.
- **Why:** on PR #145, events arrive every one to three minutes, and a new event's run
  displaced the Optimizer's research runs before they could commit (4 November, runs for
  6643992749, 6644115799 and 6644364456). Here, nothing interrupts a run but the next brief.
