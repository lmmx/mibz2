# D110 — D108/D109 closure: PROCEED for opt-in memory-lever

2026-11-15. Incoming [6903067177](https://github.com/lmmx/mibz2/pull/161#issuecomment-6903067177). Head **8de6ce01bcbf8aa79fc9d4392c46bda14d77e6a7**, PR #447. Operating pin a2da49638ac54d7763de1c167bd054f67ffdc763 (SPEC, SPEC.GUARDIAN, MODE, RULES, PROTOCOL).

**G1 PROCEED on the scoped B5 evidence question. G2 lift D108/D109's evidence HOLD; record B6 resumption for the resolved fence mutation target.** The source argument accepted in D109 now has completed, source-matched native/Miri/fuzz validation and diagnostic-specific mutation evidence. Historical decisions and failed/surviving mutation results remain unchanged.

## Source identity

Independently compared Git trees at D109's 81aa4b1dfa3333754c77baca975948c3cd0121b8, this head, and CI checkout f244e6aaa3615dc21f024e61d9392b73e8560cc4 (merge into 9d0fb49cee9510a2d37028ccf946cc24ae66893b). All three have identical src, tests, examples and bench trees and Cargo.toml/Cargo.lock blobs. In particular src is 2d731a278d30d998d016fd4479e0edadbb6fad23. The new commits change documentation and the mutation reporting harness, not the reviewed unsafe implementation. D109's bounds, pool ownership, stream tagging, aliasing and guard placement review therefore carries forward.

## Completed execution, independently read

- [Rust job 132988035476](https://github.com/lmmx/mibz2/actions/runs/43771446462/job/132988035476): shipped library 113 passed, one ignored; feature cells:: selection 21 passed, including the four feature safety tests; feature harness 27 passed. Compiled check reports sfence in the staged induction and, without the feature, no sfence or experimental functions, then memory_lever_fence: ok.
- The mutation baseline's 23 tests is consistent with its broader filter cells, rather than CI's cells::. Do not conflate the two test selections.
- [Fuzz job 132988035573](https://github.com/lmmx/mibz2/actions/runs/43771446462/job/132988035573): feature run completes 13,817 cases, seed 42; production admission 363, forced admission 7,287, forced cells 116,952,719. The earlier 27,375-case run is the feature-off run, not the feature coverage claim.
- [Miri job 132988104549](https://github.com/lmmx/mibz2/actions/runs/43771446532/job/132988104549): 44 shipped miri_ tests passed; all four feature tests were selected and completed successfully by 23:15:41 UTC: lazy closed-letter recycling, lever boundary reuse, lever exact-fit/reuse, and staged fence exit coverage. This is complete coverage of the stated selection, not a timeout or merely a green listing step.

Miri still replaces streaming intrinsics with copies/model state. Native compiled inspection and the source/RAII ordering argument complement it; no claim that Miri executes sfence is made.

## Mutation target repaired

The committed table now begins with an unmutated passing baseline: 23 tests and fence-check success. Its 38 mutant rows contain 35 kills and the three previously explained redundant-check survivors. Independently parsed and counted the table.

The four fence rows provide the requested distinctions:
1. Missing guard: writer's pending-store assertion fails; compiled check reports missing sfence.
2. Early guard drop: pending-store assertion fails; compiled check passes. Instruction presence alone is deliberately insufficient.
3. Missing intrinsic: 23 tests pass, but the compiled check specifically reports that the staged induction reaches no sfence.
4. Pending marker never set: the dedicated exit test fails at its cleared-count assertion.

The revised fence classifier distinguishes its own FAIL diagnostic from a build/tool exit. I extracted that classifier and exercised four stubbed results: compiler error and missing-tool exits classify as did not run; the targeted missing-sfence diagnostic classifies as fail; baseline success classifies as pass. This validates the reporting distinction, not a local re-execution of the Rust mutants. The immutable targeted diagnostics plus independently observed native baseline close D109's specific ambiguity.

The harness is not universally fault-proof: its restoration is only on normal completion and not every arbitrary test-process error is precisely classified. Those limitations do not negate the named assertion failures and explicit fence diagnostic supplied for this closure. No new requirement for perfect mutation infrastructure is imposed.

## Isolation and boundaries

The feature-off claim is now correctly limited to source isolation, unchanged instruction counts and a reported matched normalized disassembly comparison (435 functions). The supplied journal abbreviates the digest and does not contain the full disassembly pair; I have not independently reproduced that comparison. It is not used to assert whole-file binary identity. The source gating accepted in D109 and the successful native exclusion check establish the relevant opt-in separation.

This clears the safety/evidence objection to the two experimental arms at the reviewed source. It is not a production default, threshold change, performance qualification, readiness-tag action or workstation-run authorization. #444's shipping rejection remains recorded. The new head's bench check is successful, but I have not re-evaluated its numeric paired outcomes here; a later level result does not retroactively erase an earlier flagged regression. The Coordinator retains ordinary merge/readiness obligations, the frozen registration, and Louis's execution requirement under C5.

I did not implement these kernels and did not run native Rust/Miri locally. Work in this review: source identity checks, direct completed CI log inspection, mutation-table reconciliation and classifier reproduction. No code, other branch, infrastructure or automation change.

Migration manifest and prior branch head match. Inbox reconciliation found only this unfinished checkpoint. Routing audit clean; REVIEW destination #145. Pinned coord.py's coordinator-only FROM limitation is handled by checking the sole expected verifier failure and independently verifying the Guardian header, exact body and destination.
