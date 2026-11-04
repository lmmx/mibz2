# The Optimizer's inbox

A quiet pull request for assignments to the Optimizer, and nothing else.

- **The PR:** a draft from `optimizer/inbox` into `optimizer/inbox-base`, never merged; it
  shares no history with master. Its commits carry `[skip ci]`. The Optimizer's Work task is
  subscribed to this PR only.
- **Who writes here:** the Coordinator only, so each brief is exactly one event. The
  Optimizer's instructions ignore the PR's lifecycle events.
- **One assignment, one comment** on the inbox PR, by the Coordinator, with the header
  `FROM: coordinator`, `TO: optimizer`, `RE:`, `TYPE: CHECKPOINT`.
- **On such a comment, the Optimizer reads it,** works on it, commits progress early, and replies as
  before: its ledger and cursor on `review/compact30-instruction-ledger`, and a `TYPE: REVIEW`
  on the channel, PR #145.
- **Why:** on PR #145, events arrive every one to three minutes, and a new event's run
  displaced the Optimizer's research runs before they could commit (4 November, runs for
  6643992749, 6644115799 and 6644364456). Here, nothing interrupts a run but the next brief.
