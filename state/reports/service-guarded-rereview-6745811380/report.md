# Service-guarded registration: REVISE before freeze

Inbox **6745811380**, dispatching **6744634688**. Reviewed PR #302 at **910fd5de80a92c3fad80c3e18824d113d9470fe0**. Specification: docs/team/SPEC.md and SPEC.RESEARCHER.md at **a4f97714cc14366984015869d9d094d89488ac57**, with MODE, RULES and PROTOCOL at that pin.

**REVISE:** preserve actual child termination status; port the promised passive, bounded facts collector. The earlier workload/identity/preflight admission objections are substantially closed. The effective-quota objection from #305 does **not** transfer to this source's svc_target discovery. This is a scientific design assessment, not a freeze, run, merge or adoption authorisation.

## 1. Child exit status: a producer gap survives the judge's new test

Source: [examples/ab.rs](https://github.com/lmmx/mibz2/blob/910fd5de80a92c3fad80c3e18824d113d9470fe0/examples/ab.rs), run loop around child.wait_with_output; [bench/service_guarded.py](https://github.com/lmmx/mibz2/blob/910fd5de80a92c3fad80c3e18824d113d9470fe0/bench/service_guarded.py), row_errors; test_the_producers_failed_child and test_the_researchers_witnesses in its test file.

The parent obtains an Output, uses o.stdout, and does not persist or check o.status. An empty output becomes broken:1; a child-reported decoding error becomes err. Those two encodings are rejected. They do not exhaust process failure: valid output can be flushed before abnormal shutdown. The judge rejects an explicitly present exit:17, but the producer never supplies it. A test which manually injects that field proves the consumer only.

Research witness in probe.py: a Python child prints one synthetic, otherwise valid fixture row, then exits 0, exits 17, or terminates itself with SIGTERM. Project its stdout as ab does and embed it in the clean 420-row fixture. Results:

| Actual termination | Status omitted by projection | Actual status supplied |
|---|---|---|
| exit 0 | VALID | VALID |
| exit 17 | VALID | INVALID structural |
| SIGTERM (Python returncode -15) | VALID | INVALID structural |

This is a tested **model of the producer/consumer boundary**, not an execution of Rust ab or a claim that this decoder has exhibited the failure. It establishes that the evidence cannot distinguish successful completion from output-before-failure. With a guarded runtime's teardown/unwind obligations, that distinction is material.

**Closure:** the parent records successful exit versus nonzero exit/signal for every run, and the judge requires the successful representation. Include a producer-level stand-in which flushes valid output then exits nonzero, plus successful and empty-output controls. Catching a crashing harness at the recipe level is a different boundary. Freeze the resulting harness/checker together. No decoder redesign is requested.

## 2. Acquisition: carry over the passive, bounded collector

At this SHA, .just/service-guarded.just runs ab env, lscpu and ab sample before the target preflight; none has an outer timeout. The earlier #305 review established that ab env includes spin workers and Perf::probe. Calling the output diagnostic does not make its acquisition passive.

**Concur with the proposed common repair:** port the passive facts-only collector with a timeout and kill-after; preserve its exit, stdout/stderr and missing/error states. There is no scientific reason for #302 to retain an active, unbounded ab env. Bound the other external diagnostic calls too, with an explicit failure disposition. A diagnostic failure should be archived as such, not silently promoted to target evidence. Do not add post hoc row deletion or replacement from these diagnostics.

svc_target itself already has timeout --kill-after=10 120 and a whole-record checker. Its plan exercises are intentional preflight work, distinct from a passive facts snapshot. Preserve that distinction. Preparing the corpus before the final preflight would also shorten the gap between target observation and measurements; a point-in-time TARGET still does not certify unchanged quota/affinity throughout 420 runs. That temporal limitation is already acknowledged and is not a new whole-run telemetry requirement.

## 3. Effective quota: source closes the specific #305 objection

[src/parallel.rs](https://github.com/lmmx/mibz2/blob/910fd5de80a92c3fad80c3e18824d113d9470fe0/src/parallel.rs), cpu_quota_of, parse_cpu_max and discovery_fails_closed_quota:

- Resolve the process's v2 membership through mountinfo; refuse ambiguous/missing/unreadable mappings.
- Walk from the mapped mount point through every process-path directory. Require each directory to exist. Read each cpu.max; malformed values or read errors other than genuine absence yield Unknown.
- Take the minimum finite quota across the hierarchy. A max leaf cannot erase a restrictive parent.
- Require positive evidence that the mount top is the true root: cgroup.type must be absent, not unreadable. A visible non-root top/hidden ancestry yields Unknown.
- Keep Unlimited, Cpus(value), and Unknown separate. The target predicate requires established unlimited or a finite quota of at least ten; Unknown cannot select svc2.

The source's fake-filesystem tests explicitly include unreadable membership/parent, unlimited leaf with an eight-CPU parent, inherited ten, two finite limits, missing level, hidden ancestor and unreadable root evidence. I inspected these tests and control flow; I did **not** run native Rust discovery tests here. Under the real kernel filesystem contract and a stable snapshot, this is materially different from ab env's unknown/null collapse. Do not duplicate its weaker quota logic in the new collector and accidentally make that authoritative.

**Raw discovery archive question:** retain bounded membership/mountinfo and the resolved ancestor cpu.max facts (including absence/error status), plus the existing complete preflight record and exit, if practical in the common collector. They make later diagnosis replayable. Archive raw facts without inventing a second quota verdict or changing the registered gate. The lack of such a second raw archive does not itself refute the existing structured preflight or justify reopening its already-closed parser findings.

## 4. Registration choices: scientific assessment

**Base.** Current master by immutable SHA, merged into the candidate at freeze, answers the shipping comparison. Ancestry alone does not establish currency: record the fetched master SHA/time and both freeze identities as promised. Master moving later does not retroactively change this estimand; a later adoption decision must account for any intervening changes.

**Grid and three arms.** Seven budgets × twenty rounds × three arms = 420 runs. Ten is the switching cell; 8/9 and 11/12/16/20 are useful negative-control regimes for induced dispatch/construction costs. Keeping the twin everywhere is a defensible simple design and gives visible off-P1 diagnostics. It costs 140 extra runs versus two arms (50% more runs), not zero. The decision value is sufficient for this exact-default question; no need to add another corpus, grid or replication now.

**No-harm family.** Four one-sided head/base log-ratio bounds, each at 1 - .05/4 with df=19 and t≈2.4334, give the stated Bonferroni family control without assuming independence between budgets. Inverting the base/head speed ratio gives the correct head/base cost bound. The usual paired-log t assumptions across rounds remain; rotation reduces order imbalance but does not prove independence or eliminate drift. At twenty rounds, a three-arm six-round rotation cycle is not perfectly balanced; this is the declared harness design, not a newly discovered invalidity. The observed order should remain auditable.

**Scope of no harm.** 8/9 are reported identity flags, not passed non-inferiority gates. Do not advertise no harm over all fallback budgets. Failure to show the upper bound ≤1.02 is not proof of a regression. The four judged cells are distinct from P1.

**P1 and N1.** Preserve the two separate 95% speed/CPU intervals and their non-simultaneity qualification. CPU inconclusive permits the registered product reading but proves no CPU-cost bound. The 10-thread twin validity check is reasonable; off-P1 twin cells stay reported. N1 constrains same-binary instability at that cell; it neither certifies all seven cells nor bounds head-specific instrumentation effects. Do not delete rows or silently add a new gate after observing the other twins.

**RSS, attempts, adoption.** Report RSS only, as revised. One attempt with no replacement prevents retry-to-pass and is scientifically coherent; an invalid/refused attempt can leave no answer. Any later proposal is separately reviewed and preserves that outcome. Guarded-core evidence has higher immediate value than resurrecting old-runtime repeats. Adoption remains a separate decision after evidence; a single i9 cannot qualify every admitted machine.

The comments report amended C6 and a requested re-pin. My authenticated instruction still pins a4f97714; repository text cannot enact that re-pin. These design assessments do not certify authority newly delegated by #306. I neither modify this task nor claim its pin changed.

## 5. Verification, evidence limits and next value

- **13/13 repository Python judge tests pass locally** with the exact source and manifests from 910fd5de. The first local invocation lacked copied manifests; after restoring the exact repository layout, all thirteen pass. This was a review workspace setup omission, not a repository defect.
- The independent three-termination witness above passes its assertions. Successful output is the positive control.
- Whole-record preflight/workload/identity validation and restored report qualifications were inspected. Reported recipe stand-in CI and the prior real DRY run are useful provenance but were not rerun or claimed as fresh native coverage.
- No native build, timing benchmark, workstation capture, implementation, merge or admission occurred.
- coord.py audit on fresh channels 145/160/161/162/222: five recorded resolved misroutes, zero unresolved; REVIEW route is #145. Cursor/migration gates were checked before the receipt.

**C3:** expected closure was admissible exact-candidate evidence. Workload, frozen identity metadata, strict rows and whole-target parsing now close the previous synthetic admission failures. The remaining gap is upstream: successful-looking output loses actual termination, while diagnostic acquisition remains active and unbounded. The mechanism is producer information loss plus acquisition side effects, not a statistical threshold problem. Highest-value next work is the narrow producer/status witness and common collector port, followed by an exact-source re-review. Changing thresholds, expanding the grid or timing this draft would not resolve either gap.
