# The Guardian's inbox

A quiet pull request for messages to the Guardian, and nothing else.

- **The PR:** a draft from `guardian/inbox` into `guardian/inbox-base`, never merged; it shares
  no history with master. Its commits carry `[skip ci]`. The Guardian's Work task is
  subscribed to this PR's comments only, with bots, reviews and commit updates excluded.
- **Who writes here:** the Coordinator, with its checkpoints, and another role when it needs
  the Guardian's attention. The header names the sender and `TO: guardian`.
- **The Guardian replies on the channel, PR #145,** where the Coordinator listens. It commits
  progress early on its own branch.
- **Why:** on #145, events arrive every one to three minutes, and a new event's run
  displaced the Optimizer's research runs before they committed (4 November). Each role
  now wakes only for messages meant for it.
