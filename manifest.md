# Settled-at manifest

Channel covered through [6698812800](https://github.com/lmmx/mibz2/pull/145#issuecomment-6698812800); evidence inspected 2026-11-06 23:42–23:44 UTC. This is a reconciled version, not an atomic snapshot or live status feed.

## Instruction and continuity

- Authenticated instruction pin: [aa5ce6f52796405feed4dba0a9f88c592f14a931](https://github.com/lmmx/mibz2/tree/aa5ce6f52796405feed4dba0a9f88c592f14a931/docs/team): SPEC, SPEC.OBSERVER, MODE, RULES and PROTOCOL.
- Covers inbox [6698816282](https://github.com/lmmx/mibz2/pull/222#issuecomment-6698816282). Receipt committed on team-state/oracle at a77d6fcd542ce1d0f611b8e533dd8786319d97ba.
- Migration checked before receipt: review/oracle-intention still 382518724f59c1c4e3755a57649c318fb443923d; state/MANIFEST.md SHA-256 aaa3e698e4dccb035318805eee24f8b41e037afe89c0729f49b441a008689b50 matches migrated_from.
- Previous record: b41f5c25951bf1a335643e73a79a7f6fb654e276. Earlier evidence pins retained in linked levels and history.
- Pinned coord.py audit on fetched channel/inbox snapshots: zero unresolved routing violations; REVIEW route #145. Its verify command hard-codes FROM coordinator; this known limitation requires direct Observer header, route, body and assignment-citation checks after posting. No protocol/script change made.

## Current evidence

| Evidence | Exact revision or observed state |
|---|---|
| Release | [v0.2.5](https://github.com/lmmx/mibz2/releases/tag/v0.2.5), published 2026-11-06 23:27:32 UTC, GitHub metadata inspected |
| Publication elsewhere | Coordinator LOG 6698812800 reports crates.io, PyPI five wheels plus sdist, docs.rs verified; those services were not independently inspected |
| Pre-release comparison | [42310037840](https://github.com/lmmx/mibz2/actions/runs/42310037840) completed/success at 16646a9bb6de6c9af34689bc78788b1e79883864; no detected change/parity from LOG and draft notes, not fresh raw-data analysis |
| Current master | 02d43b3c870ff9b2dd03d3064480f7b9880283b5 |
| Confirmation journal #243 | Merged; head 83cf36d304ebf02adbb452cd011a334210191881; merge 8e0f6e37c027e393b37b140e1397853b6a3712c8 |
| Release runbook | [docs/RELEASING.md](https://github.com/lmmx/mibz2/blob/8823f617717aa8bf02e91e95f9b4b24284e388e9/docs/RELEASING.md) read at 8823f617717aa8bf02e91e95f9b4b24284e388e9 |
| Release-scaling defaults #244 | Merged; head 7f8add6309f4e795e15cb42ff8ce65741f08e70e; merge 02d43b3c870ff9b2dd03d3064480f7b9880283b5 |
| Expanded changelog #245 | Open; head b7fddd7aaa23d7906c13c9df5d305f5a93130b67; CHANGELOG.md inspected at that pin |
| Release-scaling run | Pending Louis per release LOG; no new result observed |
| Downstream wikidata-pq #219 | Open; head 93240f94d13bd15cd1c261525725596f2a45cc11; now targets 0.2.5 |
| Downstream current benchmark | [42313607695](https://github.com/lmmx/wikidata-pq/actions/runs/42313607695) in_progress at current head; documentation succeeded; no new verdict |
| Downstream old benchmark | [6672648270](https://github.com/lmmx/wikidata-pq/pull/219#issuecomment-6672648270), scoped to 66544d5498a52f5f81256e0f350380b079c39166 and the former 0.2.4 target |

## Reconciliation and checks

- Publication is complete. Informative release scaling serves the History page and does not hold release or reopen policy admission.
- #244 merged defaults supersede the old fixed release pair. Runbook is evidence of current team procedure, not a change to this task's authenticated pin.
- #243 resolves the prior journal lag. #245 remains open documentation work.
- The downstream old no-gain/memory finding is preserved as old-head evidence; it is not carried forward as a verdict on the newly running 0.2.5 benchmark.
- Account self-checks passed: re-entry, answer first, purpose, direction, ordinary language, fidelity to levels. Independent fronts, uncertainty and dispatch pauses remain explicit.
- Only changed levels, account, weak-chain findings, change log and this manifest are updated together. No code review, new gate judgment, merge, benchmark execution or authority change.
- Automation settings were not inspected or changed; the persistent webhook task is left enabled.
