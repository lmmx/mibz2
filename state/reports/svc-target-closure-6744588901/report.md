# Service-target preflight: objection closed

**PROCEED to freeze the preflight-only entry at 278f09ce38609a1d0e702bd88cac69d27a6c73cc for one workstation attempt.** This closes the strict-admission REVISE in [REVIEW 6743780326](https://github.com/lmmx/mibz2/pull/145#issuecomment-6743780326). It does not approve the timed service-guarded registration, performance qualification, production adoption, or a retry-to-pass.

Assignment: [6744588901](https://github.com/lmmx/mibz2/pull/162#issuecomment-6744588901).
Authority: docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md at a4f97714cc14366984015869d9d094d89488ac57. C1/C5 permit this bounded diagnostic assessment. The assignment's account of the C6 amendment is context, not a re-pin or a premise needed for this verdict.

## Independent replay

I fetched the checker, fixtures, tests, producer, recipe and registration at the complete reviewed SHA, and replayed the previous independent probe unchanged against the revised checker.

| Case | Revised result |
|---|---|
| Positive target control | TARGET |
| Infinite quota; quota parsed from valid JSON 1e309 | INVALID RECORD, both |
| Incomplete teams plan | INVALID RECORD |
| Float budget 9.0; float service-plan teams 2.0 | INVALID RECORD, both |
| Negative pooled count | INVALID RECORD |
| Missing-plan and contradictory unknown-quota controls | INVALID RECORD, both |
| Non-target fallback decode failure | FAILED |

All **9 repository checker tests pass locally**, including their additional malformed-field variants. The attached probe and result make the original closure replayable: place them beside the three svc_target checker/fixture/test files from the reviewed SHA and run python3 probe.py.

The producer now requires ended && finished == prepared && prepared > 0 for its target service counts. The checker additionally requires the non-pipelined service decoder, eight decoding threads and window eight. Target facts, predicate answers, actual dispatch, decoded bytes and producer exit are checked together. NOT TARGET no longer hides a failed fallback decode.

## Recipe and CI evidence

Source inspection confirms:
- raw/reads.txt records per-CPU topology/cache files and the cgroup-v2 hierarchy's cpu.max/cgroup.type, with read errors. The registration correctly describes independent adjacent reads, not an atomic snapshot or replay of discovery.
- stages.txt distinguishes completed stages; early refusals do not fabricate later evidence. A failed stage can leave partial files, so “not reached” should be read as “not completed”, not proof that the stage never started.
- timeout --kill-after=10 120 supplies TERM followed by KILL; statuses 124 and 137 map to INCOMPLETE. The tests exercise those status paths using stand-ins; they do not demonstrate actual elapsed-time enforcement or make an absolute bound against uninterruptible kernel waits.
- moving the timed recipe to .just/service-guarded.just permits a later timed-only revision without changing this preflight recipe. Source identity checks for the preflight remain in force.

GitHub's checks associated with the exact reviewed head show **25 success, 3 skipped, no failures or cancellations**. The [submit job](https://github.com/lmmx/mibz2/actions/runs/42644898491/job/129255405926) logs **9 checker tests and 6 preflight end-to-end tests passing**, including submit, refusals and both timeout outcomes. Precision: CI checked out synthetic merge 63f4522cdf8aed86c563607c66a139cccd9ed057, not the bare head; GitHub's comparison from the reviewed head to that merge reports no changed files. These are the same source tree for this assessment.

I did not run a native workstation decode or a timed benchmark. The Coordinator's real non-target smoke run is reported evidence, not my independent reproduction. Local end-to-end execution was previously blocked by this sandbox's unavailable /dev/fd process-substitution path and missing just; I rely on the inspected CI log for the recipe execution evidence, not a claimed local pass.

## Nonblocking residual

A JSON integer quota of 10**1000 reaches math.isfinite and raises OverflowError. This is outside the Rust producer's f64 range. It cannot yield TARGET: the standalone checker writes no verdict, and the recipe falls back to INCOMPLETE. Make finite_pos total over arbitrarily large JSON integers when next touching the parser; do not delay this bounded preflight or represent that case as a successful INVALID RECORD classification.

## C3: consequence

Expected: close malformed-record admission without losing real target/non-target discrimination. Observed: all six prior false admissions are rejected; the positive control survives; fallback failure is exposed. The full prior blocker closes. Mechanism learned: typed structural admission must precede cross-field agreement; the producer's ok bit cannot substitute for either.

The next useful evidence is the single real-machine target/dispatch observation, before spending a timed slot. Its information value exceeds more synthetic shape enumeration now. Preserve every outcome and the registered stopping rule. TARGET establishes this small decode and momentary facts only; NOT TARGET redirects discovery; FAILED triggers defect investigation. Freeze and queue preparation remain the Coordinator's work, with the required readiness checks at the actual runnable commit. No implementation or merge performed.

Migration gate passed before receipt: old branch head 1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest SHA-256 9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Routing audit: five resolved historical incidents, zero unresolved.
