# Validation and handoff — 2026-11-11

## Immutable stages

- Base master:2635f04e9155dd31388b24fd3f7248f87b3ddcda.
- Pre-data plan and exact supplied sources:
  8d4e7eb1a95342b160a9e010b6afb7b40c9d1be1; C1a6799878946.
- First candidate implementation, with already-frozen schedules:
  9321097665bd5c99ad50b1641d0de7c5646199ef.
- First passing evidence preserved:
  45c6301756c537c52c5af8b22de317b71f5be661.
- Final formatted-source provenance is embedded in evidence/results.json;
  SHA256SUMS also covers the note, plan, supplied manifest, audit and raw data.

## Executed checks

Python3.12.14/Linux, standard library only, assertions enabled.

1. Extracted five relay members with final LF and verified every byte count
   and SHA-256. Vendor sources byte-match git blobs at
   af3327d8aee9fffe26d2d6de76818b6ac69cc145. No ZIP-container verification claimed.
2. Both supplied scripts: exit0; exact saved-output comparison passed.
   One repeat of each also matched byte-for-byte. No source or input changed.
3. Baseline-only freeze:3,108 cases and12,429 rounds. Eligible deletion sets,
   retained identities, origins and color seeds are saved, not regenerated
   differently for each candidate. Frozen schedules SHA-256:
   00ad44509c9c10519dc83d6e218153752bcf3acd7f26135171cfe17eb35f398f.
4. First full candidate comparison: exit0, empty stderr, no semantic failure.
   All7,021 origin cases and12,429 contraction rounds pass against the pinned
   original constructors and independent row LF.
5. Ruff formatted only the five new Python files; byte-preserved vendor and
   supplied snapshots are excluded by directory-local configuration. Ruff
   check/format check pass. AST comparison against9321097 passes for all five.
6. One full comparison reproduction: exit0, empty stderr. Every result field
   except the intentionally changed formatted-source hashes is identical.
   The complete compressed per-case evidence is byte-identical. No candidate
   algorithm or input was revised to obtain this result; no failure was erased.
7. Independent saved-results audit: all7,021 anchor/no-anchor verdicts checked
   by stable sorting individual rows, all family aggregates recomputed from
   raw per-case counters, and every embedded source/schedule digest checked.
8. `git diff --check`, artifact SHA checks, and explicit rejection of
   `python3 -O run.py ...` pass. Reproduction destination must be new, so failed
   runs cannot silently overwrite an earlier attempt.
9. A separate repository-wide `ruff check .` failed with264 violations in46
   files outside the research directory. Every finding file was compared
   byte-for-byte with master2635f04 and is unchanged. New-file lint still passes.
   Counts, rules and source digests are saved in evidence/repository-lint.json;
   this failure is retained and no production cleanup was added.

The two candidate runs finished by15:41:30UTC, less than six elapsed minutes
after the first implementation commit at15:35:47UTC. Thus their combined
execution is below the30-minute budget. These timestamps enforce the execution
bound; they are not Python performance measurements. Implementation/analysis
began15:21UTC, with a17:21UTC deadline. No vendor real-fixture census, real-block
probe, timing comparison, workstation command, Miri or production decoder test
was executed for this research-only change.

Evidence SHA-256:

- results.json:1ba3a243ad2a04568bb0bd18ded8d72c39683a797cc066d91f8f1378a6579f76
- case-costs.jsonl.gz:15562edf13d17c757bb6e4b1ac304fe586e24beda7b557958b4be7c07ef7b013

## Limits and next action

Correctness evidence is finite and the accompanying elementary arguments are
the implementer's, not an independent review. The thesis pages remain directly
unverified. Operation/record counts cover the named construction, maintenance,
word and output categories; no physical heap or runtime advantage is claimed.
Conditional component summaries were deferred under the frozen rule.

This session started under Louis's direct instruction after relay6799687151;
manual-start receipt6799750846 records it. No subsequent #363 dispatch had
arrived at the final channel read. Do not fabricate an inbox completion or mark
production6791014104/real-block6793754589 complete. The Coordinator should
reconcile QUEUED6798260004 with this completed direct research delivery and
choose the next existing assignment, without dispatching duplicate research.
The completion review and PR provide the durable handoff; restarting a future
Codex session still requires an actual manual launch.

Direct `coord.py audit`/verification HTTP remains blocked by proxy403; connector
readback and local route/check are used without calling that a successful
direct audit. No acceptance gate or independent review has been bypassed.
