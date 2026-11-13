# PR381: selective Cells i9 queue preparation

Exact recipe head cf1a36ed429f361839fab6748383187104d8ab27 on
bench/cells-selective-i9, based on master3e7e294cc10f29495e9828181345d74233c549d4.
No production file changed. Source #380a8a23c1 and #365d4f0e8f are frozen.
Coordinator6862221345 reconciles the lane;6862425749 accepts the prospective
design for preparation, not final READY. Owner request is preserved byte-exactly.

Implemented one argument-free bench-cells-selective-i9 recipe, using the existing
ab runner, bounded supervisor, paired log-t estimator, Just bundle/submission and
queue infrastructure.12rounds×7families×3arms =252 measured decodes when admitted.
One ab invocation. Identical Rust1.100.0/cfg/release surroundings. A/B share original
binary; B/C differ only in src/cells.rs, mechanically checked across source files.

## Completed untimed evidence

- 22 local tests pass at cf1a36e: balanced schedule, strict complete-row/byte/pin/
  resource/machine/instrumentation validation, source pins, bad-input/build override
  refusals, topology/core separation, failed collectors, unchanged estimator.
- Real Just1.38.0→wrapper→bounded supervisor→bundle→scratch Git submission passes
  on success, preparation failure and measurement failure. Each archives once;
  resubmitting the same bundle posts no duplicate and does not rerun preparation.
- Native exact-head prepare completes: pinned text/generator archive freshly built,
  all seven hash/length checks, Wikidata reconstructed via existing verified windows,
  both native reader builds and full-byte libbzip2/Segmented/Cells parity; both route
  logs match exactly. Zero measured decodes. An earlier pre-commit preparation also
  passed and is preserved under validation/native; final evidence is exact-cf1a36e.
- RSS1/64MiB controls pass for native self getrusage, wait4 and /proc HWM; raw records
  included. No decoder memory-performance claim from these controls.
- Actual native preparation bundle successfully submitted and idempotently resubmitted
  to a scratch local bare repo using the established submission script and fake gh.
  No fake result posted to real GitHub. Bundle is below the95MiB submission ceiling.
- Read-only target preflight refuses this AMD cloud/limited affinity/missing sysfs
  configuration as expected. No target fact is invented; actual i9 facts are verified
  noninteractively when Louis runs the admitted queue. No CPU/corpus question remains.
- Connector-backed coord.py audit:6resolved historical findings,0unresolved. Direct
  API CONNECT403 is preserved, not mislabeled as a successful direct audit.
- Fresh evidence branch01f5bc23ea2242969b084ebff7bf3b178814f3a2 has no prior
  cells-selective-i9 submission; queue and new recipe tags are absent.

## Pending admission, not READY

At19:14UTC Test43489092176 succeeded, including the real new recipe test step;
hosted pre-commit succeeded. CI43489092042 still has macOS x86_64 queued; all other
packaging jobs succeed/legitimate publish skip. Benchmark43489732975 runs.
Initial43489092104 was automatically superseded when draft PR was marked reviewable,
not manually rerun to seek a result. It remains recorded as cancelled.

Need final exact-head green snapshot and Coordinator frozen-executable approval.
Then the existing authorised Run Signal executor must dispatch ready for
bench-cells-selective-i9 at cf1a36ed429f361839fab6748383187104d8ab27. Coordinator
must add `bench-cells-selective-i9 bench/cells-selective-i9` to master queue and LOG
its admission; verify queue/tag/branch/recipe/approved commit agree. No direct-tag
bypass. Neither this session nor acting monitor exposes a workflow-dispatch tool;
this capability remains an explicit admission dependency, not a fabricated signal.

Louis alone runs `just workstation-queue`. After receipt, Coordinator reconciles
all252rows and gates, removes consumed entry and clears tags. No merge/release/
default/gate change; existing380integrationDEFER stands. ATTRIBUTION.md completes
only the bounded source finding, not causal explanation or performance clearance.


## Final refreshed checkpoint

All required latest same-head checks are now green at exactcf1a36ed:
Test43489092176, CI43489092042 (including both macOS architectures),
Benchmark43489732975 and hosted pre-commit success. Release/publish skips are
appropriate; superseded Benchmark43489092104 remains recorded cancelled.
Full fresh PR/check/status payload is validation/final-checks.json. Head unchanged,
master3e7e294 unchanged, PR open/unmerged and mergeable.

Coordinator6862613562 accepted the executable diagnostic conditional on these
final checks and routed the existing authorised Run Signal handoff. Those checks
have now passed. This is not a claim that the operator has picked up or executed
that handoff. No tag/queue mutation has occurred in this Codex session.

**Single remaining dependency: an existing workflow-enabled executor must carry
out6862613562, then Coordinator records/verifies queue admission.** Our exposed
GitHub tools include read/rerun operations but no workflow_dispatch; rerun cannot
supply new signal inputs. Direct API access previously returned CONNECT403 and
was not retried indefinitely. The current cloud environment reports no configured
secrets/outbound identities, restricted policy without api.github.com. Desktop
capability is enabled/stateunknown but no desktop js/control/open tool is exposed;
there is no verified authenticated browser fallback. No bypass/new infrastructure
or borrowed credential is attempted.

Required existing workflow inputs: signal=ready,
recipe=bench-cells-selective-i9,
sha=cf1a36ed429f361839fab6748383187104d8ab27, on master.
Then expected-blob-safe master queue line from Coordinator6862613562:
`bench-cells-selective-i9 bench/cells-selective-i9 Matched Segmented/original/selective Cells;12x7x3`.
Verify successful workflow, exact ready tag, no HOLD, unchanged branch head,
matching recipe and queue, no prior submitted job; LOG admission. Until then
**NOT READY**. Louis has no individual benchmark command or input-preparation work
to do. His execution command after admission is only `just workstation-queue`.
A GitHub handoff is durable notification, not automatic activation/restart of a
manually launched Codex or transport-executor session.
