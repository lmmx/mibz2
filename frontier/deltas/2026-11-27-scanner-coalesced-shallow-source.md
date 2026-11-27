claim: scanner-coalesced-shallow-source
by: codex
outcome: partial
evidence: A6b archive b87790e5, #1105, #1120, #1123, immutable squash 5ed20c1d, parent 5c7fe5d2

A fresh depth-one clone of bench/scanner-coalesced from the real origin failed the old driver's first-parent lookup before compilation. The corrected driver fetches the registered squash to depth two into a private repository, validates the fetched head and its first parent against both full pins, and checks them out in separate detached worktrees. It copies the registration from the pinned head's private working tree. The live shallow check recovered both expected SHAs, baseline and head phases blob 403cfbbdd160559826793d173b673dad5bcac682, and registration blob 161604b5aaeeac4496899404325860725e243fca.

The shallow fixture regression proves both source objects can be absent from the runner, checks the detached source contents and SHA on both arms, and rejects a wrong parent. All eight scanner tests passed. The measured Rust driver, compiler configuration, feature sets, randomisation, score and registered decision thresholds are unchanged. The original request remains withdrawn; the standing authorisation applies once to the reviewed, green successor.
