claim: none
by: chatgpt
outcome: partial
evidence: frontier/tools/frontier.py; tests/test_frontier.py; owner relay 26 November

**What was asked:** The active Claude harness reports refusing to run the frontier client because it combines embedded commit identity with HTTP request code.
**What changed:** Remove the embedded personal identity; use the checkout's explicitly configured repository-local Git identity only when creating lease commits. Missing configuration refuses the mutation. Read-only check does not need identity or PR API calls. Use synthetic identities in isolated tests and check both paths.
**What's next:** Relevant tests and a normal retry within the harness's permitted tools after integration. Respect any continuing privacy restriction; do not ask the owner to run the blocked command in its place.
**Caveats:** The reported harness hook is not inspectable here; its exact policy and whether this refactor resolves the refusal are unverified. No credentials, network destinations, HTTP safeguards, permission, cap or existing gates changed. Existing owner commit-identity policy still applies to checkout setup.
