claim: scanner-coalesced-u8-i9-queue
by: codex
outcome: partial
evidence: #1126 squash c3db2e396ad61b6fbcdef58d8a5f9285234b22be, reviewed head 96e574a7b3b12d3dfa86e85f02538851e5a1e6e7, review 6084937446, queue row #1120, original withdrawal 2e1e9ffd

The corrected recipe passed all checks and independent review. An extra preflight compiled all eight registered binary variants from a detached depth-one clone of the exact recipe on the VM (Rust 1.90.0); the frozen i9 toolchain remains 1.99.1. It recovered the immutable squash/parent and registration blob 161604b5aaeeac4496899404325860725e243fca.

The queue card carries the corrected head. Verify the existing row, lease, old request withdrawal, receipts, ready/hold tags and evidence, then publish the single registered request with PR 1126 and review 6084937446. No coalesced hardware evidence has been recorded. [Independent review](https://github.com/lmmx/mibz2/pull/1126#pullrequestreview-6084937446)
