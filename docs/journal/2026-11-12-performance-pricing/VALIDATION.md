# Validation and handoff

2026-11-12, assignment363/6808549538. Master unchanged at
2635f04e9155dd31388b24fd3f7248f87b3ddcda. Manually resumed00:03UTC;
analysis and checks completed by00:26UTC, inside the02:03UTC bound.
New backend/falsifier execution: **zero**. No old LF constructor, census,
real-block probe, benchmark or workstation command was executed.

Local environment: Python3.12.14/Linux, standard library only. Available
Ruff was invoked with writable UV_CACHE_DIR/UV_TOOL_DIR under /tmp.

- Every INPUTS.json data/source entry was fetched by `git show COMMIT:path`
  and its original byte count and SHA256 checked. All snapshots preserve the
  pinned original bytes; existing compressed case evidence is copied unchanged.
- All #370 family aggregates recompute exactly from its saved7,021 origin cases
  and3,108 contraction cases. The saved12,429 rounds are joined by case ID,
  column, origin and family. Deleted original/packed row counts agree; intervals
  are ordered, bounded and disjoint. No schedule was regenerated.
- Raw-record, allocation, coordinate-write and merge identities pass for every
  contraction case. Packed/stable selection, output, grammar-node and final-map
  counts agree. No new candidate's semantics are asserted by these checks.
- #367's431 actual route rows reproduce its recorded route/block/output totals;
  the untimed census has no decode/inversion times to substitute for attribution.
- All artifact hashes, original git blobs and derived arithmetic identities pass.
  A second calculation compared exactly against saved pricing (including the
  source and input-manifest identities). This is arithmetic verification,
  not a second experiment or old decoder rerun.
- Scoped Ruff check and format check pass; Python compilation passes;
  `git diff --check` passes. No production tests or Miri are claimed for this
  standalone arithmetic/report-only change.

`pricing.json.gz` losslessly stores the complete per-case/family calculations:
uncompressed2,612,705 bytes, SHA256
`9cc8933c9e638c1c3580898f046a18da95db8d38182c9f3469fd3dadeaf66a6c`.
Its generator source SHA256 is embedded in that result. `SHA256.json` covers
the report, source, snapshots, plan and compressed results. No timing is offered
as performance evidence.

Disposition: prospective backend screen not cleared; no extra experiment.
Compact global rebuilding is the next maintenance comparator if this line is
reopened, not a measured faster decoder. #371 REVISE METADATA and #365's exact
head DEFER stand. Recommend the existing #367 matched port/context diagnostic
as the strongest next whole-decoder falsifier, under Coordinator routing and
the existing execution/readiness rules. #366 remains deferred/incomplete;
no claim that any workstation job is ready.

Only Codex's own completed cursor may be updated for6808549538. Coordinator
owns disposition and subsequent dispatch; no merge, release or independent
self-certification. GitHub messages can be consumed by this running session,
but cannot start/restart a stopped Codex session in the configured relay.
