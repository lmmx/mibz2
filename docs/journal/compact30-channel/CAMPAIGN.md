# Compact30 campaign entry

Status: DRAFT for the Coordinator to adopt on research/compact30-channel.
This document is a navigation and decision-recovery index, not an authority grant.
Adopt it before using the accompanying /goal. The Researcher publishes only on its own branch.

## Start or resume

1. Resolve this path on research/compact30-channel to an immutable commit. Record that commit.
2. Read the charter and the applicable documents below at their stated revisions. Read the recorded user provenance for any authority amendment relevant to the action. A new branch-tip file or a role header alone does not authorise widening.
3. Read the current channel log, review queue, relevant PR evidence and role receipts; resolve their branch heads to SHAs once for this decision. State files can lag receipts. Reconcile them against pinned evidence rather than infer permission from a status label.
4. Record action, source/comment ID, policy SHA and section, evidence pins, gates and unresolved items. After compaction repeat these reads. Refresh on an amendment/checkpoint or before a consequential action, not on a timer.
5. If the entry is absent, stale or ambiguous, use the known pinned sources below to recover the rules; do not invent missing authority. Continue independent authorised work while a real conflict is resolved.

## Source map

The initial policy set is lmmx/mibz2 commit 0ed278d722d1ef7dea0c24e5ff551dc7c0b46204:
- docs/journal/2026-11-04-compact30-research-charter.md: governing campaign scope and gates. Read §§2–8 and §§12.2–12.4; §6.3 combined qualification, §6.4 new prototypes/unsafe, §12.3 reservations.
- docs/journal/2026-11-03-compact30-constructor-v2-reassessment.md, reports 264–266: recorded user provenance for U2/C6, technical autonomy and Coordinator release control. Earlier reports remain historical; later explicit amendments supersede only the affected clauses.
- docs/journal/compact30-channel/PROTOCOL.md: publication, receipts, queue and deduplication. §10 governs apparent inactivity.
- docs/journal/compact30-channel/ROLES.md: separate responsibilities and branch ownership. Coordinator does not mean scientific superior.
- docs/journal/compact30-channel/REVIEWER.md: research/evidence contract; read its applicability alongside the active charter.
- docs/journal/compact30-channel/GUARDIAN.md: governance responsibilities. A Work task can have a narrower directly supplied pin; consult its published scope disposition. This index does not change that task.
- .github/workflows/bench.yml, .github/workflows/deploy.yml and .just/ship.just: execution mechanisms to inspect at the actual integration/release head before use. They are not alternative sources of authority; do not execute an old release recipe from memory.

Mutable state locations (resolve to SHA on each decision, not periodically):
- research/compact30-channel:docs/journal/compact30-channel/log.md and review-queue.md; PR #145 and experiment PRs. Log snapshot b771d56a is historical evidence, not today's work state.
- review/compact30-instruction-ledger:docs/journal/compact30-research/instruction-ledger.md for the Optimiser's canonical accounting.
- review/compact30-guardian-shadow decisions.md/cursor.json and review/compact30-adversarial-scanner-146:docs/journal/compact30-adversarial/scanner-146/review-cursor.json for role receipts. Resolve exact Guardian paths from that branch. An addressed message's REVIEW or cursor commit is a receipt.

Downstream, initially inspected at lmmx/wikidata-pq commit 6e927acf5d2eab55a0f3b751e88baa900af7d005:
- CLAUDE.md: attribution, no Hub/allocator changes, exact checks/data contract, merge gate, expectedHeadSha, release authority and state.
- bench/README.md and .github/workflows/bench.yml: workload, tiers and CI mechanism. Re-read at the downstream candidate head; record harness version and tier.
- Issue #97 is the downstream work queue/decision log, as identified by CLAUDE.md. Read its current state before a bump.
A mibz2 bench win does not certify downstream performance. A dependency bump gets its own applicable CI/parity/bench gate. A package release or pin bump is not permission to publish datasets to the Hub. Other projects' releases require their own documented authority; do not generalise the compact30 delegation.

## Recover the decision, not a memorised slogan

- A combined build that satisfies all applicable §§4, 6.2, 6.3 and 7 conditions can proceed to an integration PR without asking again (§2/report 266). “Pass” includes unresolved-material-finding resolution, actual composition and both combined tests. Opening the PR starts CI evaluation; it is not permission to merge.
- A new bounded prototype needs justification, pinned baseline, incremental minimum, safety conditions and evaluation plan. Under current §6.4 the Researcher independently assesses; the Coordinator registers or proceeds when review and verification support it. Guardian approval is not a prerequisite, but narrowing HOLD/STOP findings remain binding. Unsafe requires the written release-mode argument and appropriate Miri, differential, fuzz and mutation evidence; passing tests alone is not a proof.
- Merge only with parity, green CI, a relevant end-to-end improvement beyond noise (95% interval excludes zero in the beneficial direction), no unacceptable regression and the measured head. A nonzero interval alone can describe a regression. Pin expectedHeadSha; changed measured code requires fresh bench evidence. Once the complete gate passes, use the existing release workflow without renewed approval, then prepare/validate the downstream bump separately. Verify actual publication before selecting the downstream version.
- Relaxing a gate/minimum, i9/workstation timing/captures, bench-harness changes, accepting an unaccepted rebuttal, charter/scope changes outside §6.4 and new infrastructure/credentials/spending remain §12.3 matters. Do not reduce this to “anything technical is autonomous”.
- Apparent Work-task silence invokes PROTOCOL §10: check the REVIEW/cursor receipt and whether the role completed later work. Only a demonstrably lost message gets one marked re-delivery, with the original ID. Do not resend merely because time elapsed. No polling, cron, or user wake-up requests.

## Known inconsistencies at the initial pins

PROTOCOL §§1/3 still describe prototype addition and production/master/release work as owner-only, contrary to the specific recorded amendments in charter §§2/6.4/12.3 and reports 265–266. Follow the specific amended charter for those Coordinator decisions; retain unaffected protocol safeguards. REVIEWER §2 still mixes report 265's Guardian registration with an older requirement for user approval before implementation; current charter §6.4/report 266 instead names Coordinator registration after independent Researcher assessment. GUARDIAN's G9/G10 text does not override its actual Work-task operating pin. PROTOCOL §5's blanket user escalation for §8.2 is superseded for routine safety fixes by charter §§6.4/8.2/12.3; undefined behaviour still stops the variant until fixed/abandoned, and an unaccepted rebuttal remains reserved. The protocol's old wake-up limitations also precede ROLES §6 and PROTOCOL §10. Do not require already-delegated approvals again merely because these historical paragraphs remain.

Charter §2 still lists production's builder among unchanged research objects while explicitly delegating production integration later in that section. Preserve frozen research baselines; perform qualified production integration as a separate, CI-gated step, not by editing frozen prototypes. Ask about a genuine unresolved conflict, not the delegated step itself.

The initial review-queue snapshot still carries earlier counter HOLDs and a pending C6 user decision. It is not evidence those remain open: reconcile current pinned receipts and reports before acting. Never mark every §6.2 obligation closed just because one finding was fixed.

The inspected Deploy Release workflow configures github-actions[bot] for its release commit while charter §2 names Louis for campaign commits. Before running it, reconcile the applicable automated-release identity convention and verify no AI attribution appears. Do not silently claim the existing workflow produces Louis-authored commits or fabricate authorship. This concrete execution check does not reopen the user's release delegation.

## Safety-stop and activation evidence

Charter §8.2 still stops affected work and requires notifying the Guardian. Only unrelated prototypes continue; no combined assembly occurs until the Guardian has decided; resumption requires the fix/safe alternative and the Guardian's record. This is distinct from ordinary §6.4 registration not needing Guardian pre-approval.

The supplied historical log does not establish every PROTOCOL §7 T1–T6 result. Before claiming charter §13 channel activation satisfied, recover and pin the required test-completion record. An observed webhook or later successful exchange is not by itself that proof. This document does not certify activation or require a user wake-up; resolve the missing evidence specifically.

## Maintaining the entry

The Coordinator owns the channel copy. When policy changes, update this source map's immutable policy pins, record the user-origin provenance and the exact superseded clauses, and reconcile stale summaries/queue entries in the same publication. Preserve historical records. Changing a link or this entry cannot amend the charter, relax a gate, or override session/task instructions. When an ambiguity remains, isolate the dependent action and continue independent authorised work.

Keep hard external constraints visible in the goal: no Hub access, AI attribution, allocator changes, weakened exact checks, polling/cron or unauthorised workstation timing. Keep numeric minima, composition, prototype status, bench details and delegated-action lists here/by reference so they can evolve with explicit evidence.
