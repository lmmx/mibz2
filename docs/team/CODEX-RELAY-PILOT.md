# Codex ↔ mibz2 relay — manual pilot

**Status: manual, explicitly not an automated integration.** The live team protocol is [PROTOCOL.md](PROTOCOL.md); this page only describes the human-invoked pilot proposed in [CODEX-PILOT.md](CODEX-PILOT.md). It does not amend the existing role registry, webhooks, routing checker, cursor branches, authority or merge rules.

## Addresses and responsibilities

| Place | Purpose | Who writes |
|---|---|---|
| [#363 — Codex manual inbox](https://github.com/lmmx/mibz2/issues/363) | Durable assignment / correction / reading list. **Codex must open it manually.** | Louis or his explicitly instructed assistant |
| Codex's new implementation PR | Patch, CI checks, test results, review findings, progress, blockers, resume point | Codex or its operator |
| [#145 — existing team channel](https://github.com/lmmx/mibz2/pull/145) | The Coordinator's authoritative decision log and existing role messages | **Only through existing PROTOCOL.md pathways** |
| [MONODROMY #346](https://github.com/lmmx/mibz2/issues/346) / [Pathfinder #314](https://github.com/lmmx/mibz2/pull/314) | Existing agents' inboxes, unchanged | Existing Coordinator under the live routing protocol |
| [PR #362](https://github.com/lmmx/mibz2/pull/362) | MONODROMY's bibliography and cells production design/handoff | Preserve; do not turn it into Codex's implementation PR |

`.github/coordination/coord.py` does **not** know a Codex role or inbox #363; this issue does not wake Codex and `coord.py route --to codex` cannot be used for this pilot. GitHub issue creation alone cannot configure an automatic Codex or Work subscription. Never describe a posted issue comment as dispatched, received or acknowledged unless a Codex session actually reads and acknowledges it.

## Beginning a Codex session

1. Open [#363](https://github.com/lmmx/mibz2/issues/363) and read the initial assignment and newer comments. Confirm the assignment is still current.
2. Fetch `docs/team/CODEX-PILOT.md` and this file from this setup PR's branch (or master **only after merged**). Also read **current master** team policy and the latest #145 gate before implementing.
3. Read MONODROMY's DESIGN.md and HANDOFF.md at PR #362's branch; inspect the actual existing `src/`, `examples/rauzy.rs`, and benchmark harness.
4. State the exact branch and head SHA you began from, and the code/testing boundaries. Create **one implementation branch** off master.
5. Work toward the slice-one PR; do not change the default or treat GitHub status as permission to merge.

A suitable **manual launch prompt** for Codex:

> Implement the first, off-by-default cells-kernel production slice for `lmmx/mibz2` as the manual Codex production delegate. Read issue #363 and both `docs/team/CODEX-*-PILOT.md` guides at their setup-PR branch, then the current master policies, #145 gate, and MONODROMY's PR #362 design and handoff. Work on a separate branch and open a reviewable implementation PR. Report tests, proof/safety gaps and exact head SHA. Do not merge, release, change the default, use the i9, or impersonate any agent.

The filename shorthand above means `docs/team/CODEX-PILOT.md` and `docs/team/CODEX-RELAY-PILOT.md`.

## Incoming: assignment or correction on #363

Use plain issue comments; this is a manual mailbox, not the registered team's `CHECKPOINT` format. One scope-changing comment at a time:

```text
MANUAL PILOT ASSIGNMENT | CORRECTION | HOLD
From: Louis / assistant at Louis's request
For: Codex implementation delegate
Regarding: <implementation PR and exact head SHA, if known>
Request: <one bounded change or question>
Must not change: <safety, default route, release, role authority>
Reference: <links to evidence/decision>
```

An assistant must not invent Louis's authorisation or silently change scope. If the Coordinator later returns, its existing rules still decide production merges.

## Outgoing: one durable checkpoint on the implementation PR

Codex should comment on its **implementation PR** at natural milestones (not every command), and link the message or PR from #363 if Louis requests a mailbox summary:

```text
CODEX PILOT CHECKPOINT
Branch / exact head SHA:
Scope completed:
Safety/correctness evidence (with commands and outcomes):
Bench: default-arm neutral? experimental-arm result? measurement limits?
Open blocker / uncertainty:
Decision needed (by whom):
Next action / how another implementer resumes:
```

No `FROM: coordinator`, `FROM: monodromy`, `FROM: guardian` or forged team review headers. A Codex self-review does not substitute for the independent B5 review. Record a genuine mathematical counterexample precisely and flag it without silently changing the proof's claimed scope.

## To get something into the Coordinator's channel

During the pause, **the implementation PR is the live work record**. Only post to #145 when Louis explicitly asks ChatGPT to relay an update. Follow the *current* `CHATGPT.md` and `PROTOCOL.md` §7 exactly: a `FROM: louis via chatgpt`, `TO: coordinator`, `TYPE: PROPOSAL` comment, giving his words verbatim, a factual interpretation, and links to the PR and evidence. Do not call it an ACK, delivery to an agent, acceptance, or approval; it awaits the Coordinator's `TYPE: LOG` after the Coordinator returns.

If an existing registered Work reviewer is independently and explicitly asked to stand in, use PROTOCOL.md §8 instead; Codex is not automatically a B5 reviewer or a replacement for any paused role.

## Resuming the original team

The first restored Coordinator session should read (1) #363, (2) Codex's implementation PR at its exact head, (3) #145 since the handoff, and (4) the saved handoff files, then decide under the existing policy whether to accept, review, reassign, defer or abandon. MONODROMY may assess any discovered mathematical counterexamples independently when it resumes. No cursor or state branch is assumed advanced by this pilot.

If a permanent Codex role or automated inbox is later desired, **that is separate work** requiring Louis's authenticated role/rule decision and a properly tested change to the role registry, `coord.py`, inbox offset/cursor and wake trigger. Do not infer any of that from this manual pilot.
