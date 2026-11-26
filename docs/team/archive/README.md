# Coordinator-era team machinery (archived 26 November 2026)

**Frozen historical record.** On 26 November 2026 the repository moved from the coordinator and
role-inbox workflow to [`frontier/`](../../../frontier/README.md), following the cross-readings in
`docs/journal/2026-11-26-rhizome/` (consequences C5 and C6 of
`deleuze-guattari-cross-reading.md`).

Nothing in this directory governs live work:
- **Role specs and Work task templates:** `SPEC.md`, `SPEC.*.md`, `WORK-*.md`, `CODEX-*.md`,
  `COORDINATOR-FAILOVER.md`, `MONODROMY-ORIENTATION.md`, and `codex/astra-e-brief.md`.
- **Inbox protocol and role directory:** `PROTOCOL.md` and `ROLES.md`.
- **Linguistic register:** `MODE.md` and `MODE.RATIONALE.md`.

**Related retired machinery:**
- The seven role inbox threads (`#160`, `#161`, `#162`, `#222`, `#314`, `#346`, `#363`) are
  retired in favour of `frontier/claims/` and `frontier/deltas/`.
- PR `#145` is no longer a coordination channel; it remains only as the workstation evidence
  noticeboard (`just submit`).
- `.github/coordination/coord.py`'s routing commands (`route`, `verify`, `check`, `warn`, `audit`)
  are retired; `coord.py head-sha` remains live as the exact-head merge check.

For live instructions, start at [`frontier/README.md`](../../../frontier/README.md) and
[`CLAUDE.md`](../../../CLAUDE.md). Standing policies that remain live are listed in
[`docs/team/README.md`](../README.md).
