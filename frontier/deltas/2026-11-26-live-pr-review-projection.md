claim: harden-frontier-tooling
by: chatgpt
outcome: partial
evidence: #955, #968, #973; frontier/tools/frontier.py brief/render; tests/test_frontier.py

**What was asked:** Prevent invisible pending integration work by mechanism, rather than repairing individual claim pointers.
**What I found:** A PR opens before its status = review edit reaches master. Fresh sessions read only master and the lease hides the claim from new work. This is a circular dependency, not a missed instruction.
**What changed:** Derive the review queue from paginated live GitHub open PRs and their existing Claim headers; both brief and STATUS use it. Show stale pointers, drafts, duplicate and unclassified PRs, and explicit unavailability. Exclude linked pending claims from new-work suggestions. CI validates existing lineage for frontier edits; the trusted-base mirror refreshes on PR lifecycle events. Regression tests reproduce a clean checkout at saturation, stale pointers, lost leases, pagination and bad lineage.
**What's next:** Companion draft based on the existing #955 repair, not a competing write to its held branch. Run CI and independent review; integrate after or through #955 under existing protected-path rules.
**Caveats:** This is discovery, not authority transfer, atomic global admission, or a cap increase. Remote API access is required for fresh review data; failure is visible. No claim or lease mutation on master.
