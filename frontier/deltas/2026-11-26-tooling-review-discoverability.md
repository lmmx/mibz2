claim: none
by: chatgpt
outcome: done
evidence: #955; head 20e1c9f4fd7e790904ea881333ad8ef405441a9f; #943; frontier/tools/frontier.py brief; frontier/claims/harden-frontier-tooling.toml

**What was asked:** Explain why the highest-priority repair remains unintegrated while new work is taken.
**What I found:** The existing P1 claim is open on master, while its PR changes it to review. The live hold excludes it from claim selection, and brief reads review state from the checkout. The status page therefore misses the pending PR. This is the discoverability defect already recorded on #955.
**What changed:** Publish the identical two-line review pointer already present in #955, separately from its code. This is a metadata-only repair, not a competing implementation or transfer of authority.
**What's next:** Independent integration of #955 and resolution of the existing flags; verify the additional saturation/handoff and cross-id admission cases in the existing review before scaling. The proposed next cap is six, not an immediate increase; branch protection remains outstanding.
**Caveats:** The pointer does not grant merge authority, clear a flag, release a hold, or certify the code. No cap or protected path changes.
