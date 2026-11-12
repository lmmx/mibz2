# Deliberate integration of PR375

Owner-authorised integration in session, following Coordinator decision [6836680787](https://github.com/lmmx/mibz2/pull/145#issuecomment-6836680787). This executes that narrow integration; Codex does not assume the Coordinator role or self-dispatch a successor.

## Decision and completed action

**MERGED** PR375 using the connector's supported `expected_head_sha` argument, exactly **853b4ac9eea20195a2dbc5b5374ab5fd2a2e8117**, method merge. GitHub returned merged=true and **8138e014c53f8c61cf028cbea78d8d9ae134c865**.

Fresh premerge master/base: **1f2738c5448fc123a71fd201921e4101e664b0c6**.
Two commits: accepted packet correction ec1b84eb60c8f011da4fa41d6d7d4a839f91bf24 and MTF correction853b4ac.
Only examples/packet_census.rs and examples/mtf_two_level.rs differ,4 insertions/4 deletions total. Packet visits the same checked prefix; MTF enumerates every base in identical order and preserves N−16*K+16*k arithmetic/overflow behavior. No unsafe path, assertion, algorithm, production default, gate, recipe or dependency change.
Full implementation evidence remains at feec83892087019889a906954f3c6342cd55377f/state/reports/mtf-clippy-6835658086, prior packet evidence at d2ba8d734ea0ef66a09ef98ec4d4586b43492617/state/reports/packet-clippy-6832249633. Neither correction was reopened.

Premerge exact-head gate refreshed:
- Test43310214911: SUCCESS, including Rust, harnesses, fuzz-smoke, submission, typing and Python matrix.
- Packaging CI43310214895: SUCCESS, including macOS x86_64/aarch64, both Linux targets, Windows and sdist. Release/publish jobs skipped by design.
- Hosted legacy pre-commit: SUCCESS at https://results.pre-commit.ci/run/github/1435546475/1794518771.1rJN_h_iRSeFdtWzLgcGWQ.
- API PR mergeable=true, mergeable_state=clean.
- Coordinator explicitly accepted the two example-only refactors and required no new review round. No new unsafe implementation or production performance claim; benchmark-path filters do not schedule a benchmark for these files. No gate was waived or changed.

## Postmerge verification

Fresh PR response confirms merged=true, same full head, merge SHA8138e014.
Fetch confirms origin/master at the merge.
Merge parents, in order:1f2738c5448fc123a71fd201921e4101e664b0c6 and853b4ac9eea20195a2dbc5b5374ab5fd2a2e8117.
Merge tree **0e24049f91bc4ff1c4a6456d34651058b444fcb4** equals the reviewed head's entire tree, not merely the two changed blobs.
Changed-file assertion and git diff --check pass. No repeated implementation/test pass is needed to establish source identity; exact-head validation applies to this identical tree. Broader postmerge CI is tracked separately.

Repository coord.py's native urllib transport returns CONNECT403. As in the previous deliberate integration, its unchanged head_sha validation function was executed with the fresh connector PR response as its read transport. Before and after (merged=True) both returned ok=true; assertions were enforced. The server also enforced expected_head_sha atomically. No validation function or policy changed. Native audit also returned403, and is not claimed passed.

Postmerge normal push workflows observed:
- Test43323058105 running.
- Documentation site43323058116 running.
- Packaging CI43323058112 skipped: CI.yml check-commit intentionally only builds non-release pushes when its message condition matches; this merge is not a release or cibuildwheel request. Exact-head PR packaging already passed; skipped master packaging is not claimed as a new pass.
No manual workflow dispatch, benchmark retry, release or publication action. The normal master push triggers its existing workflows.

Read-only release inventory: latest published tags v0.2.9/py-0.2.9 resolve to b65b2aa54488f072cf005b8fe62461c32e6ef083. since-last-release.txt records subsequent work; this integration adds only example compatibility. No release is authorised here.

## PR373 recovery: evidence and next action

PR373 remains unmerged at **ed6f424f238851ff8296e7a48c2b96dfb8332e7e**. Accepted R1 and reviewed accounting integration are complete; no change to that head or frozen diagnostic.

Fresh exact-head runs:
- Test43269375554, Occupancy43269375729, Benchmark43269375558 and Cells preparation43269375801 SUCCESS.
- Packaging43269375621 FAILURE: macos(macOS15,x86_64), job/check **131361540210**, CANCELLED. Other actual build jobs passed, including arm64 macOS. No recorded steps. Job started17:26:09Z and completed17:41:10Z; elapsed time alone does not establish cause.
- This job's log retrieval returned404 BlobNotFound, request f60631fc-f01e-00d1-75f7-72f960000000 at22:47:44.6013519Z. Four check annotations exist, but the connector rejects their endpoint as unsupported; summary/text/title are null. This is **Class D**, not proven runner/setup failure.
- Legacy pre-commit remains FAILURE at https://results.pre-commit.ci/run/github/1435546475/1794504358.Y40p1JvNSESWR89bgRGktA. Previous bounded retrieval had CONNECT403/web-reader inaccessible. **Not repeated in this integration. Class D retained.**

**Single next actionable step: Coordinator arrange an operator-access evidence handoff for these two failed checks on the pinned373 head.** An operator who can access the hosted results should provide:
1. The actual failing pre-commit hook, command/exit code and complete failure output, retaining the result URL, head, attempt and timestamp.
2. All four annotations and any cancellation/runner explanation for https://github.com/lmmx/mibz2/actions/runs/43269375621/job/131361540210, including who/what cancelled it if visible.

Store that output durably and classify each check separately under CI-INCIDENTS. Actual cause then decides the smallest code correction (A), scoped and paced job recovery (B), one diagnostic repeat (C), or continued operator escalation (D). No guessed cause, full-workflow rerun, empty commit, no-op head update or waiver.375's passing jobs do not validate373's missing check. No additional Codex implementation task is justified before this evidence.

After CI recovery, the Coordinator still owns373's deliberate integration and any necessary current-master reconciliation. Final PROTOCOL9 readiness additionally needs actual-i9 topology/affinity/permitted-CPU selection and approval, verified absolute corpus directory and corpus identities, the argument-free queue/parameterised-recipe binding, final source/recipe identities and explicit queue/run authorisation. CPU and corpus bindings remain PENDING;120 measured decodes+9untimed/oracle checks preserved; warm20+20 unadmitted. No workstation measurement, queue submission, Run signal or gate/default change.

Completion is recorded on Codex's own state branch and through REVIEW145. No new inbox completion ID is invented for this direct owner-authorised integration. Acting Coordinator remains sole orchestrator; GitHub comments alone do not guarantee restarting a Codex session.


Closing snapshot: documentation43323058116 SUCCESS; Test43323058105 still running (Rust Tests step at last job inspection; submission, fuzz-smoke, typing and Python3.10–3.15 passed). No failure observed and no final Test pass claimed. Coordinator must reconcile this ordinary postmerge run. The two exact-head premerge workflows and legacy were green before the guarded merge. PR373 recovery note posted as comment6838120361.
