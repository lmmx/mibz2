# Codex ↔ mibz2 relay (registered inbox, manual execution)

**Status:** This document and the `coord.py` routing change take effect only when the setup PR merges. Until then, [#363](https://github.com/lmmx/mibz2/issues/363) remains a manual pilot issue. Even after merging, **routed ≠ awakened**: no Codex/Work webhook was created. Codex must be started in its own session and given the inbox link.

## Addresses

| Address | Meaning |
|---|---|
| [#363](https://github.com/lmmx/mibz2/issues/363) | Codex's registered **issue** inbox, one new assignment at a time, *manual launch* |
| `team-state/codex:state/cursor.json` | Completed inbox IDs, written only by Codex after actually finishing |
| [#145](https://github.com/lmmx/mibz2/pull/145) | Team channel: Codex's REVIEW and the Coordinator's LOG/decisions |
| Codex's own implementation PR | Source changes, independent review, CI results, exact SHA, resumable checkpoints |
| [#362](https://github.com/lmmx/mibz2/pull/362) | MONODROMY's research/handoff; preserve this branch and its mathematical claims |

## Dispatch and receipt

The current primary or [owner-activated acting Coordinator](COORDINATOR-FAILOVER.md) runs:

1. `python3 .github/coordination/coord.py audit` and resolve misroutes.
2. `python3 .github/coordination/coord.py route --to codex --type CHECKPOINT`. If busy, stage the assignment as `TYPE: QUEUED`, `TO: codex` on #145, not in the inbox. Respect queue priority and one in-flight item.
3. If free, post to #363 with `FROM: coordinator`, `TO: codex`, `RE: ... @ <full head SHA>`, `TYPE: CHECKPOINT`. The acting operator adds a visible `ACTOR: ChatGPT Work (acting)` line; this is attribution, **not** authentication.
4. `python3 .github/coordination/coord.py verify --comment-id ID --to codex --type CHECKPOINT`. **Separately start Codex** and direct it to that exact inbox comment.
5. Codex reads the latest specs, performs the scoped work, posts a `FROM: codex`, `TYPE: REVIEW` on #145 explicitly citing the assigned inbox comment ID, and appends the ID to its own `processed_comment_ids` cursor. The Coordinator verifies the result rather than treating a post as a finished implementation.

Incoming example (only active Coordinator posts):

```text
FROM: coordinator
TO: codex
RE: cells slice one @ <full commit SHA>
TYPE: CHECKPOINT

ACTOR: ChatGPT Work (acting, if applicable)
INPUTS: docs/team/CODEX-PILOT.md, PR #362, #145 gate 6775049636
TASK: Implement the off-by-default first production slice on a fresh branch.
OUTPUT: implementation PR, tests, exact head SHA, safety gaps and REVIEW on #145
LIMITS: no default flip, self-approval, merge, release or workstation use.
```

Outgoing example (Codex reports on #145, not the inbox):

```text
FROM: codex
TO: coordinator
RE: inbox <assigned comment ID>; PR <number> @ <full head SHA>
TYPE: REVIEW

Changes: ...
Evidence (commands/results): ...
Missing checks: ...
Independent B5 review requested: ...
Next step: ...
```

**No auto-delivery guarantee:** `coord.py verify` proves location/header/one-at-a-time queue state; it does not prove that Codex was running or consumed the message. Check the cursor, PR evidence and actual REVIEW. If Codex can't update its cursor, the item remains busy until reconciled; do not fake a completion ID.

## Scope

[CODEX-PILOT.md](CODEX-PILOT.md) is the current assignment. The new `InverseBwt` implementation remains **off by default**. Default route, cooperative teams, release policy and mathematical programme remain untouched. A different session must supply independent B5 safety review before unsafe code merges. New mathematical counterexamples are preserved with the smallest reproducible case and referred to MONODROMY, not silently repaired as research.

## Coordinator failover

[COORDINATOR-FAILOVER.md](COORDINATOR-FAILOVER.md) documents how Louis can replace the **operator** of the existing Coordinator role during a quota pause. Its `FROM: coordinator` messages pass current routing checks, with `ACTOR:` explicitly distinguishing the acting executor. No parallel Coordinators, no authority widening, and no claims of running a background Work task until the platform confirms it.

## Direct work before merging this setup PR

Louis may already start a Codex task by giving it #363, the current master policies, PR #362 and this branch. In that case it is a **direct manual assignment**, not a verified `coord.py` dispatch; label it so in the implementation PR. Once this setup merges, use the routed procedure for subsequent assignments.
