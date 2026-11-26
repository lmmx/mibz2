claim: harden-frontier-tooling
by: chatgpt
outcome: partial
evidence: #974; #980; #955; tests/test_frontier.py

**What was asked:** unblock the transition from three to six live reservations.
**What changed:** resolve stacked branch conflict while preserving the newer participation rule; repair closed-unmerged automatic reconciliation and expired-transfer capacity bypass; serialize acquisition/renewal/transfer across distinct claim IDs by atomic two-ref push. Add tests for last-slot contention, expired transfer at cap and closed-unmerged recovery.
**What's next:** independent review and exact-head tests; independently resolve original flags; updated-client rollout (old clients bypass epoch); separate cap PR.
**Caveats:** API-only editing; new tests not run locally. Legacy direct writers and older clients bypass client-side CAS; legacy lease migration and branch protection remain separate. No cap or flag change.
