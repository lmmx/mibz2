# Service-guarded Revision 2: technical PROCEED, freeze remains conditional

Inbox **6746912441**; PR #302 at **bb362fa3c5e0650d730614682faeab8a43b20d19**. Governing spec pin **a4f97714cc14366984015869d9d094d89488ac57** (SPEC, SPEC.RESEARCHER, MODE, RULES, PROTOCOL).

**The technical REVISE from REVIEW 6746023579 is closed.** Required preflight acquisition also closes the failure-status gap raised in REVIEW 6746559255. No new technical revision is requested on these findings. **PROCEED technically toward freeze; this is not an unconditional authorization to freeze or run.** The authenticated C6 pin still reserves the relevant policy; the requested re-pin has not occurred in my instruction. Base merge/currency and the acknowledged BENCH/CHANGELOG reconciliation remain explicit freeze prerequisites.

## Reconstructed findings

### Child output and termination are now separate evidence

At this SHA examples/ab.rs obtains Term from the reaped child status and emits termination alongside stdout. bench/service_guarded.py requires exactly the string success. It does not depend on an exit field absent from this branch.

The producer-to-judge test does more than inject a field: it invokes real ab run with stand-in children, preserves the resulting termination, replaces only fixture metadata needed for embedding, and judges the emitted record. Exit zero with valid output is accepted; exit 17, SIGKILL after valid output and empty output are refused. I inspected this test and verified its actual execution in the exact-head rust job log:

- [rust job 129321580196](https://github.com/lmmx/mibz2/actions/runs/42663644396/job/129321580196)
- SG_AB=target/debug/examples/ab SG_REQUIRE_AB=1 python3 bench/test_service_guarded.py
- test_the_producer_records_each_ending_and_the_judge_refuses_all_but_success ... ok
- Ran 18 tests; OK, no skip.

Locally the same Python suite passes **17 tests and skips one**, because no native ab was built here. I do not label that local skip coverage; the independent CI log supplies the native execution evidence. My additional consumer probes reject exit 0 as a termination label, exit 17, signal 9, signal 15, unknown, a missing field and boolean true. Successful termination remains the positive control.

### Required acquisition gates on success; optional diagnostics do not

The recipe now captures svc_target's status, the checker's status, and the parsed verdict. It proceeds only for collector exit 0 AND checker exit 0 AND TARGET. Both raw preflight streams and the statuses are retained. The judge separately recomputes the whole target predicate from the raw record and collector status; it does not trust only the previously written verdict.

Independent probes retain a valid TARGET record while varying preflight.exit: 17, 124, 137, empty and malformed all yield INVALID/preflight; exit zero remains valid. The recipe's stand-in test establishes the stronger execution boundary: complete TARGET output followed by 17/124/137 produces REFUSED, retains output/status/stderr and creates no runs.jsonl. The exact-head submit job records that test passing, plus its successful control:

- [submit job 129321580919](https://github.com/lmmx/mibz2/actions/runs/42663644396/job/129321580919)
- test_a_complete_target_record_then_a_nonzero_exit_refuses ... ok
- Ran 11 tests; OK.

The local recipe suite could not exercise this shell path: the environment cannot open /dev/fd/63 for the recipe's process-substitution logging. All local setup attempts failed before the experiment logic; just is also absent. This is an environment limitation, not a repository test failure. I relied on inspected test logic and the exact-head CI log, without modifying the recipe to make a local pass.

Optional env/lscpu/discovery/sample calls use the common diag helper, bounded by TERM at 30 seconds and KILL five seconds later. It archives stdout, stderr and exit, and permits judging the complete run even if a post-run diagnostic failed. That distinction is correct: these records do not supply target admission. My positive fixture remains valid with optional diagnostic failures. Do not port an optional-failure policy onto #305's required collectors.

### Provenance and scientific scope

Raw hierarchy discovery is diagnostic, has no independent verdict and cannot override svc_target. Fake-tree discovery tests passed locally. Keeping svc_target's existing hardened quota source avoids replacing established hierarchy logic with the weaker former ab-env source.

base_fetched is now required and matched between freeze and bundle. This binds the recorded fetch time; it is not cryptographic proof that the chosen base was master then. The Coordinator must still fetch and record actual master at freeze and merge it into the candidate as registered. Later master movement does not change the frozen estimand.

The recorded qualifications accurately preserve the assessment: separate speed/CPU intervals are not jointly 95%; CPU inconclusive is not non-inferiority; 8/9 are reported identity flags; N1 at ten neither validates every cell nor bounds differential instrumentation effects. Other twin results cannot introduce post hoc deletion or gates. No additional grid, replication or telemetry requirement is warranted.

Leaving corpus preparation after the preflight retains the acknowledged point-in-time limitation. My earlier suggestion to shorten that interval was not a mandatory blocker. A one-machine result remains one-machine evidence, not qualification of every admitted CPU or production adoption.

The known BENCH 3.4.0 versus comments naming 3.5.0 mismatch should be reconciled with CHANGELOG before either freeze, as promised. Version text does not itself invalidate these closure tests; any merged source changes still require the corresponding exact-source checks. This review does not preapprove unseen integration.

## Verification and C3

- Local: 17 passing judge/discovery tests, one native-producer skip; **15 independent admission probes pass their assertions**.
- CI: freshly retrieved exact-head metadata has **25 success, 3 skipped**, no pending/failing checks. Native producer execution and all eleven recipe tests were verified in logs, not inferred from a green aggregate alone.
- Local recipe execution was unavailable for the shell reason above. No native build, workstation capture, performance timing, implementation or merge was performed by this review.
- Migration gate passed before receipt. Routing audit: five resolved historical findings, zero unresolved.

**Expected:** closure of termination loss and active/unbounded acquisition without converting optional diagnostics into new acceptance gates. **Observed:** producer termination retained, required preflight status enforced, optional failure archived; the earlier failure witnesses now refuse while successful controls survive. **Gap closed:** the material technical objections in scope. **Mechanism learned:** payload validity and successful acquisition must both reach admission; diagnostic availability has a different role. **Next value:** finish the authenticated authority update and exact freeze provenance/version integration, then the already bounded registered measurement. More synthetic variations on the same closed predicate have less value than that transition. No retry policy, adoption authority or task pin has been changed.
