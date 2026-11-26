claim: none
by: chatgpt
outcome: done
evidence: #948, #954; frontier/claims/branch-protection.toml; frontier/claims/three-arm-experiment.toml; docs/team/RULES.md A4, D; owner conversation 2026-11-26

**What was asked:** Record the repository owner's decisions on branch protection and the proposed three-arm experiment, without manual relays to agents.
**Owner authorisation:** In the owner conversation, the two decisions were offered for direct recording; the owner replied verbatim: "Send". The following records the agreed scope, not a new delegation of authority.

**Branch protection — approved with conditions.** Require PRs to master, applicable existing CI/correctness gates, and prohibit force pushes and deletion of master. Require the frontier check only after its known failures are fixed and independently verified. No mandatory independent-account or CODEOWNERS approvals while agents share the owner's identity; no human approval bottleneck for routine merges. Disable routine administrator bypass if compatible with the approved workflow. Preserve hold/* refs and existing authority boundaries. Investigate separation of admin and agent credentials, but do not change access permissions without separate approval.
**Three-arm experiment — deferred, not rejected.** No approval for the proposed token expenditure or workstation runs, nor for a smaller pilot yet. Continue the existing bounded rollout; first harden the protocol and collect observational evidence from ordinary authorised work. Before seeking fresh approval, reconcile the 20M/26M weighted-token ambiguity and run count, benchmark premise drift, model/topology confounds and between-session variance; propose a smaller pilot with a hard resource ceiling. Preserve the original frozen registration and version any proposed revision before data.

**What changed:** branch-protection is available for implementation within the approved scope; three-arm-experiment is blocked pending a revised proposal and fresh owner approval.
**What's next:** Implement and verify approved protection via the normal striated workflow, including independent review; retain the three-live-hold canary until its existing gates pass.
**Caveats:** This is a decision record, not application of GitHub settings, a change to credentials, authorisation of additional spending or a relaxation of any existing gate.
