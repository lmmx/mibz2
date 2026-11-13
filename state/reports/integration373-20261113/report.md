# Guarded integration of #373 and retarget of #376 — 13 November 2026

Direct owner authorisation in this session permits #373's exact-head integration and #376's retarget only. It explicitly excludes merging #376, workstation execution, manually launched measurements, queue submission, Run signal, releases, production-default changes and new optimisation work. Coordinator remains sole orchestrator. Coordinator comment6848610020 separately confirms deferral, not completion, of onecore inbox6848164049.

## Completed integration

- Authorised and merged PR head: `8329f0c702a1c5598e39e7f2413db55a12ce8884`.
- Prior master: `8138e014c53f8c61cf028cbea78d8d9ae134c865`.
- Resulting merge/master: `4df9651b84d0a01143b601a05dd7748720008fdd`.
- Parents are exactly prior master and authorised head, in that order.
- Result tree: `05e2a56c3381600ef975cbd351b19d973cb0f123`, identical to GitHub's tested synthetic merge `078f37a5ef27240ee9161cfc70c134a95110939a` and a fresh local conflict-free merge-tree.

Fresh pre-merge Actions: Test43393348451, packaging43393348433, Occupancy43393349525, Benchmark43393348524 and preparation43393348575 all SUCCESS. Hosted pre-commit SUCCESS, including its exact result URL in observations.json. Both macOS architectures passed. Release/publish jobs were intentionally skipped, not failures. PR open, non-draft, mergeable/clean; no requested or rejecting formal reviews. Independent Researcher6821626797 and Coordinator R1 acceptance6822234208 remain the governing review records. No new unsafe code or production adoption is being integrated.

Policy: current OWNERSHIP, PERFORMANCE-POLICY, STEWARDSHIP, PROTOCOL and CI-INCIDENTS read/reconciled. Measurement-only integration follows STEWARDSHIP's green-CI/no-unintended-measured-change route. Retain ordinary CI's pattern2t +6.0% [-10.9%,+26.0%] and maximum RSS point growth0.8%; every wall interval includes zero. These are ordinary EPYC head/base results, not Cells live measurement or optimisation acceptance.

Merge used the connector's explicit `expected_head_sha` argument, not an unrecognised alias. Post-merge API confirms merged=true at the same authorised head. Repository coord.py's unchanged head_sha function passed before and after on fresh connector responses; this is not a claim that native API transport/audit succeeded. API branch/ref and git ls-remote/fetch identify master directly: the PR payload's historical base.sha still named d504898 before merge, so it was not used as the actual current master.

Source correspondence: no diff from previous master for src/, root Cargo.toml/Cargo.lock, examples/ or workstation queue. Diagnostic source/evidence, frozen PLAN and both Just recipes exactly match authorised #373. #375's fixes and blog publication survive unchanged.

## Retarget and separate #376 disposition

PR #376 now targets master at `4df9651b84d0a01143b601a05dd7748720008fdd`. Head remains `5f9b4fade9e2bfaa061b625a0a55f7b3f8d9d34a`; no commit, rebase, force-push or lint correction was added. Its diff is still exactly two files: run.sh2+/1- and test_driver.py66+. Original implementation and evidence preserved.

Fresh local merge-tree is `afd43a466f4d9ee7f5dbc7acb41c310285b00bfe`. All14 existing harness tests pass/no skips when run from an archive of that combined tree with real Just and stand-in children (test376-merged.log). This is untimed validation, not hosted changed-base CI or decoding-performance evidence.

**Separate decision: do not merge #376.** Retargeting produced no changed-base Actions run or new hosted legacy result. The workflows do not subscribe to pull_request edited/base-change events; Benchmark explicitly lists opened/synchronize/reopened/ready_for_review, and Test/CI/preparation use the default pull_request event types. The displayed Test failure43396306206 is still the observed inherited MTF lint on old synthetic merge6fb3be52e3af18604bf56ea0d544b08b6e2db583. Packaging/preparation/Benchmark/legacy successes likewise predate retargeting. They must not be relabelled as new-base validation. The old synthetic merge ref also remains at the observation point.

Smallest remaining integration action for Coordinator authorisation: append an ordinary merge of verified master4df9651 into #376, preserving its patch/history, then inspect normal synchronize-triggered full-head CI and hosted legacy. This is a real dependency integration, not an empty commit, repeated lint patch or unchanged-check rerun. The present approval authorised retargeting, not this extra branch mutation; none performed. No new general review, cleanup or research assignment is needed.

## Post-merge and readiness limits

Master's automatic push workflows are recorded in observations.json. Test43398652622, Documentation43398652731 and Benchmark43398652640 were still running at that snapshot; Benchmark links43398652710 passed. Packaging43398652808 is skipped by its non-release push condition; this does not negate the exact-head PR platform passes. No manual workflow or benchmark was launched. Any final status refresh is retained beside this report rather than changing historical observations.

After #376's separate integration requirements are satisfied, next hardware evidence is the actual i9 allowed CPU/affinity/topology and absolute corpus paths with the three frozen compressed hashes. The existing read-only preflight is in state/reports/cells-measurement-resume-20261113/i9-preflight.py; no i9 executor is connected and no machine facts have been supplied. Do not infer archived bindings. Select/approve an allowed CPU, freeze the argument-free recipe binding and exact source, then Coordinator §9 admission and explicit owner-run authority. Frozen120 measured+9 untimed/oracle retained; warm20+20 unadmitted. No ready/queue/Run action or workstation measurement occurred.

This completes the authorised merge/retarget sequence, not measurement readiness or the broader performance objective. A GitHub relay comment cannot automatically restart Codex; further execution still needs a live/manual session.
