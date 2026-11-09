# Integration review: #302 and #305 before freeze

Inbox assignment: [6747581238](https://github.com/lmmx/mibz2/pull/162#issuecomment-6747581238).
Operating specification: `a4f97714cc14366984015869d9d094d89488ac57`.

**PROCEED on the integration changes at both exact heads:**
- #302: `d136df7e31903bb7c18c3f90d7fa499985adb544`.
- #305: `2fab12cacca8c5235d30ca832ce0df2d53b8ceaf`.

No technical REVISE from this delta. The prior scientific review remains applicable because the changed executable content is only the BENCH version string. This is not permission to bypass the pending authenticated re-pin or final freeze records.

## #302: independently checked scope

Compared the entire tree delta from previously reviewed `bb362fa3c5e0650d730614682faeab8a43b20d19`. The six changed paths are root CHANGELOG.md, CLAUDE.md, bench/workstation-queue.txt, docs/team/RULES.md, bench/CHANGELOG.md and bench/report.py.

The first-parent chain is exactly two commits: merge `0cc4bd7d24311762e27214e856b02e34665898b7`, whose parents are the previously reviewed head and master `a2da49638ac54d7763de1c167bd054f67ffdc763`; then the version commit. GitHub compare reports six reachable commits, which includes imported master ancestry and is not a contradiction of that two-commit first-parent description.

The merge alone changes precisely the four advertised paths. No decoder, producer, Python checker, recipe, dependency or build-input change is hidden in the merge. bench/report.py changes only BENCH 3.4.0 → 3.5.0. The new bench changelog attributes passive facts and reaped child termination correctly; it does not change the observed metric, admission rule or test implementation.

CLAUDE.md and RULES.md contain a broader authority claim. I read this as proposed/current repository policy evidence, not as a replacement for this task's authenticated pin. The assignment itself retains the re-pin prerequisite, so no conflict requires a broader HOLD on technical integration.

## #305: independently checked scope

The complete delta from reviewed `14f5924e4af3c66f98254ba73bcedeac3a905eb3` is one commit and four paths: bench/CHANGELOG.md, bench/report.py, the registration journal and examples/ab.rs.

The version split is coherent: 3.5.0 names the shared facts/termination additions; 3.6.0 names the additional sysfs-state, ran_threads, exit-field and arm records. report.py changes only BENCH 3.5.0 → 3.6.0. ab.rs changes one doc-comment version reference, no executable Rust. The journal retains the original writing-time version while naming its reconciled 3.6.0 version. No collector, timeout, preflight, full-data judge or stopping logic changes. The R12 status-admission finding remains closed.

Version labels identify schema provenance; they are not evidence that a new timing experiment ran, or authority for a changed method.

## CI verification, 9 November 2026

Fresh check-run reads:
- #302: **25 success, 3 skipped**, all completed.
- #305: **24 success, 3 skipped**, all completed. The macOS aarch64 job is now successful, so the reported cancellation is no longer an outstanding check.

Read the submit-job logs, rather than accepting the assignment's local-test count:
- #302 job 129332868795: svc_target checker 9, recipe 6, service_guarded 18 (one skipped), and service_guarded recipe 11. The skipped test is not claimed as covered by that suite.
- #305 job 129336184039: leader record 25, admission 34 and recipe 11, all passing. The claimed 95 local tests were not independently reproduced; these CI results are the fresh verification used here.

CI checkout identity was independently reconciled:
- #302 merge `bb583f682140422ffcecad6d2fe8814a0654545c` and reviewed head share tree `7e44336a5783506e54364e2f78b944c8df91408d`.
- #305 merge `7bf6d3df770590ba9b45db13f781cecc9020d45e` and reviewed head share tree `3d10162572670b2e69871c9e9868d4c15cd22526`.

Thus the inspected CI recipe executions apply to the exact reviewed source trees. See evidence.json for check names, links, excerpts and diff patches. No local rerun was needed for these version/documentation changes after that verification. No native build, timing or workstation experiment was performed by this review.

## Consequence and remaining boundary

The version reconciliation prerequisite is technically satisfied at these heads. #302's reported master merge is verified. Preserve the promised fetched-master SHA/time, final frozen head and carried bench version in each freeze record. A later changed head needs its own required CI/provenance; this review is not a blanket approval of unseen changes.

The authenticated re-pin remains outstanding under the instructions actually supplied to this run. In particular, repository C6 wording alone cannot widen this task's repeat-policy authority. Keep the stated freeze hold until the re-pin read-back is verified. This review does not approve execution, change any historical verdict or infer production adoption.

Migration verification passed against old head `1283b969e9e295e268257cd1b6be3ad7da29e6dd` and manifest SHA-256 `9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0`. Receipt: `811cab5dd62c5b85a31d5d216a7e4eca31e41017`. Fresh routing audit: five resolved historical misroutes, zero unresolved. State writes are only to team-state/researcher. Automation unchanged.
