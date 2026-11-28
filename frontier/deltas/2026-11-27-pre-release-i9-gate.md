claim: none
by: chatgpt
outcome: partial
evidence: #1228, evidence 5182d546, #1246, docs/RELEASING.md, docs/team/PERFORMANCE-POLICY.md

**What was asked:** the maintainer directs that workstation testing precede release; the 0.5.17 regression should not have reached users.

**What I found:** the old §1 explicitly allowed the combined release measurement after publication. §5c was also waived for 0.5.17 on route-count evidence although the cost inside an unchanged route increased. The i9 release comparison subsequently found 2.6% and 1.5% primary wall losses at 8 and 10 threads (#1228); the 3% attribution threshold is not a non-regression threshold.

**What changed:** §3.5 now requires a valid, read, exact-input i9 preflight before publishing any performance-affecting release. Updated §1, §3.4, §4, §5 and §5c and the agent release instructions to agree. Existing per-PR acceptance remains unchanged. Registered P1 pre-release-i9-gate to make the release command fail closed and reuse the workstation harness and evidence.

**What's next:** apply the runbook to 0.5.18 even if workflow enforcement is still pending. #1246 is the corrective PR; test the combined candidate against v0.5.17 and demonstrate recovery in the regressed cells against v0.5.16 before publication. Coordinate preflight visibility with #1241 rather than duplicate it. Preserve previous measurements and registrations.

**Caveats:** no hardware run, publication or workflow implementation was performed in this contribution. The new guard is prospective and does not relabel the v0.5.17 result.
