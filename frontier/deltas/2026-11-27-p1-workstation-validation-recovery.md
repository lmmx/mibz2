claim: none
by: codex
outcome: done
evidence: origin/master 3f98f2ae; #1031, #1034, #1053, #1084, #1105, #1106, #1116, #1118, #1120, #1126, #1127, #1131; origin/evidence c0b2341 and b87790e5

The three P1 dependencies identified in #1070 all reached the i9 queue under the 9 November standing authorisation. At 08:14 UTC master contained one row per recipe. Each ready tag matched its reviewed branch head. No hold tag was present for any of the three.

| Recipe | Exact ready SHA | Workstation disposition at verification |
|---|---|---|
| bench-scanner-seam-window | 3ae44390ade27cf105fc301b702e2eca9ab25fe2 | Completed on the i9. NOT ADOPTED. Ten threads were faster, but the 12-thread wall ratio of 0.9898 had a 95% interval of 0.9726–1.0073. Bundle c0b2341, disposition #1118. #1031 and #857 closed unmerged. The scored attempt is final. |
| bench-a6b-endpoint | bdcfeeeb2154a90dda0f24ea8069cb8710168184 | Successor queued through the existing row and verified receipt 1044c68c. It fetches candidate faa9df70 directly into its private checkout. The original 02e492a7 attempt returned INCOMPLETE before building or timing anything (b87790e5). |
| bench-scanner-coalesced | 96e574a7b3b12d3dfa86e85f02538851e5a1e6e7 | Queued for the registered comparison of 5ed20c1d to its first parent. The earlier request for 5aeb37a4 was withdrawn before activation after the shallow-checkout failure was reproduced. The independent four-thread CI objection in #1084 remains open. |

Queue activation is distinct from a hardware result. The evidence tip at this check was b87790e5. The next receipts for A6b and the coalesced scanner should be checked against their frozen registrations. Existing queue markers and tags preserve once-only execution.
