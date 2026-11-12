# Post-R1 integration and readiness reconciliation

12 November 2026; direct owner request, read-only reconciliation. Policy/master
`2635f04e9155dd31388b24fd3f7248f87b3ddcda`. No new dispatched assignment:
all eleven #363 messages through6823213899 are completed in remote Codex cursor
`6161de241c5ff7557d0dbf77f31c57520963be55`. Latest Coordinator decision6830074647
retains the separate integration boundary. No completion ID is added by this report.

## Complete

- #373 R1 at `b5131679f51c2a0245bc03f62b7afd833b8bff23` was accepted by
  [Coordinator6822234208](https://github.com/lmmx/mibz2/pull/145#issuecomment-6822234208).
  Failed-child resource retention and its regression evidence are closed. No new
  evidence calls for reopening R1, repeating its independent review or another B5 round.
- #368 repair at `bfd5d322651974f9803f4c03942cf3ba3b5161d2` has independent D103
  review and seven successful Actions plus successful legacy pre-commit.
- #366 at `d61e0976b6faa8e8b1dfdcb0e10bffdc28235108` already contains that repair
  and completed untimed recipe preparation. Seven Actions and legacy remain green.
  Recomputed today: all41 repair files identical; both stable patch IDs identical
  to their appended counterparts. See correspondence.json. This is correspondence
  verification, not a new safety review.
- #374 preparation remains complete and independently eligible for the Coordinator's
  publication procedure; this research reconciliation adds no publication dependency.

## Integration remains a Coordinator action

All three research PRs are open/unmerged against the above master. Follow
[6820358956](https://github.com/lmmx/mibz2/pull/145#issuecomment-6820358956):
deliberate final diff/source/performance assessment, fresh full-head Actions AND
legacy observation, expectedHeadSha-safe merge, then post-merge source and check
validation. Green badges alone are not that decision. Preserve #366 random2t
+3.3% [0.5,6.1] and maximum RSS+4.8%; preserve #368 records2t+3.8% [0.2,7.5]
and maximum RSS+2.8%. These diagnostic/accounting integration judgements are not
optimisation ACCEPT and do not erase #361's adverse evidence or #365 DEFER.

#366 already includes the two reviewed #368 patches by cherry-pick. Coordinator
must choose the integration order and explicitly reconcile #368's PR disposition;
do not apply the repair twice or infer Git ancestry from byte/patch identity.
Read-only three-tree preview found no conflict between368 and366. It found one
conflict between366 and373, at the end of `.just/bench.just`: both append independent
recipe stanzas. Retain both unchanged when integrating. Preview is diagnostic,
not an actual merge or proof of runtime compatibility.

## #373 checks: distinct causes and missing evidence

At exact `b5131679f51c2a0245bc03f62b7afd833b8bff23`:

| Evidence | Current finding | Consequence |
| --- | --- | --- |
| Preparation43207829271; Test43207829676; Benchmark43207829571 | Success | Existing valid evidence, not a speedup or all-green verdict |
| Occupancy43207829422/job131147393835, step10 | Class A project assertion: `AssertionError: the ledger check fails under teams` | Real inherited accounting defect; integrate reviewed dependency, never suppress assertion |
| Packaging43207829260/jobs131147445829 and131147445942 | Both macOS jobs cancelled, no steps; each decoded log404 BlobNotFound | Both platform builds unvalidated; Class D, no external-cause inference |
| Legacy result1794482231.FWhSnWB4TjeLnaE7eIXJmg | Failure; log unavailable | Class D; diagnose before prescribing a fix/rerun |

Occupancy retained-capacity witness: `ll1799923 outside900000..=900000` and
`rle36 outside900000..=1799999`; events reconcile. Ordinary occupancy also prints
violations even though its old step passes. Root `src/trace.rs`, `src/lib.rs`,
`src/parallel.rs` and occupancy workflow in373 exactly match master, confirming
the inherited dependency. Reviewed368 adds route-history accounting and the
ordinary-run assertion. The accepted R1 touches none of these files.

Packaging sdist, Linux x86_64/aarch64 and Windows succeed; release/publish jobs
are appropriately skipped. macOS absence is not satisfied by those other builds.
Check metadata exposes annotation counts, but the connector rejects annotation
URLs as unsupported; no annotation cause was inspected. Legacy direct access
fails HTTPS CONNECT with proxy403 and web fallback is inaccessible. These are
access limitations, not evidence of why CI failed. Historical unchanged Ruff
formatting failures noted in373's handoff do not establish the legacy job's cause.
No reruns, waivers, empty commits or speculative lint changes were made.

## Recommended next action and smallest possible Codex assignment

**Next actor: the Acting Coordinator's deliberate integration run**, followed by
classification/recovery of the outstanding373 checks under CI-INCIDENTS. No new
kernel, accounting algorithm, R1 correction or experimental harness is justified.

If the Coordinator delegates the mechanical integration follow-up, propose just
one bounded assignment: **append the selected, integrated accounting/recipe base
to373 and resolve the two-recipe conflict, preserving the frozen diagnostic**.
Inputs are373/b5131679f51c2a0245bc03f62b7afd833b8bff23,
366/d61e0976b6faa8e8b1dfdcb0e10bffdc28235108 and reviewed
368/bfd5d322651974f9803f4c03942cf3ba3b5161d2. Coordinator must name the exact new
integrated master SHA in the dispatch; it does not exist yet. Append history,
no force-push. Reconcile both Just stanzas without changing their bodies; retain
the reviewed repair exactly and all original/R1 evidence. The generated experiment
must still use corrected365/d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2, not silently
substitute the newly integrated root source.

Completion criteria: real Just1.38 no-skip onecore4 and live13 tests, both recipe
dry runs (live uses explicit stub CPU/corpus arguments only), fresh-checkout
untimed live preparation/fixture/error tests, reviewed-source/PLAN/R1/archive
digest correspondence, exact new-head CI/legacy record and one REVIEW/cursor.
The normal changed-head Occupancy job must validate ordinary and teams assertions;
unresolved platform/legacy failures stay explicit. No unrelated fix, scheduling,
estimator, corpus, CPU choice, gate/default change, timing or queue mutation.
Do not dispatch this until Coordinator selects the integration base. If that run
handles the small conflict itself, no Codex coding assignment is necessary.

## Measurement readiness after integration

Final recipe commit and measured-source identities are separate. Onecore keeps
candidate d33787e138426cb441db6975f54b2eccca02e89a, baseline
b65b2aa54488f072cf005b8fe62461c32e6ef083, frozen driver digest
6167171b7889706226340adc0beba89bfbf5e933af944d7f68e49d97bc7bfa07 and ten pairs.
Cells keeps PLAN freeze3b43af956d553905ea744f6cb6e1c785a0f8297a, corrected365
source, Rust1.99.1, four off/on builds, frozen corpus digests, one explicit allowed
logical CPU, ten balanced pairs/three corpora, fresh children and0.25s settle.
Budget stays **120 measured decodes +9 untimed/oracle checks**; warm20+20 unadmitted.

CPU/topology and corpus-directory binding are still decisions before execution.
Current `bench-cells-live-path cpu corpora` requires arguments; queue/await-submit
passes only a recipe name to `just submit`. Thus freeze a concrete argument binding
and untimed wiring check before admitting an argument-free queue recipe; do not
assume the parameterised preparation is queue-ready or modify generic queue code.
This is a subsequent readiness decision, not folded into the proposed integration
assignment. Preserve all CRC/hash/oracle checks and failed-attempt evidence.

Coordinator must check final frozen recipe, exact run-commit CI, integration and
post-merge checks, then alone arrange PROTOCOL9 master queue, exact ready tag/Run
signal and LOG. Louis's execution authorises the workstation run. No run, tag,
queue entry or readiness claim is made here. Full-corpus prechecks remain part of
the eventual authorised programme, not extra preparation measurements.

## Reconciliation limits and routing

Only source comparisons, log inspection and a read-only merge preview ran today;
earlier test/Miri evidence is not represented as freshly rerun. `coord.py route`
selects145; direct audit fails proxy403. Connector inbox/cursor reconciliation
finds no unprocessed assignment; this is not a successful coord.py audit.
Completion IDs and the blog checkpoint stay unchanged. A Codex state report and
one channel REVIEW carry this proposal; stop for deliberate routing. GitHub events
can notify the Acting Coordinator's Work task but cannot restart this Codex session:
CODEX-RELAY-PILOT still requires a separate manual launch.
