# Compact30 campaign entry

**Read first, every run, and again after compaction: [MODE.md](MODE.md)**, the register for every role, set by Louis. At natural checkpoints and transitions, renew it inside the substantive message itself, above all where the tempo has drifted. There are no standalone reminders, nudges, extra runs, acknowledgement rituals or recitation (Louis, 2026-11-04). It changes how the work is done, never what is authorised. Its linguistic premise: [MODE.RATIONALE.md](MODE.RATIONALE.md); its drafting history: [2026-11-04-compact30-mode.md](../2026-11-04-compact30-mode.md). Read the premise before proposing any change to MODE.md.

Adopted by the Coordinator on research/compact30-channel from the Researcher's draft at
review/compact30-adversarial-scanner-146@8b3913a6 (REVIEW on #145, comment 6637667413), with the
objective section below added from the user's instructions of 2026-11-04.
Apart from that objective, this document is a navigation and decision-recovery index, not an
authority grant.

## Objective: a faster production decoder

The campaign exists to make production's inverse BWT faster. The whole inverse BWT is in scope:
the constructor, the walker and the representation (charter §1–§2, report 273). The agents
hill-climb round by round on whole-decoder work removed, until a candidate beats production on
CI's bench. Only then does it go into production. Technical direction is the Coordinator's and
the three agents': optimise, redesign, measure, redirect or stop, on credible whole-decoder
payoff per unit of compute.

**The measure that decides is the whole decoder, not the constructor (charter §5.6, report
272).** On production master a Wikidata block's `decode_block` is about 21% build, 60% walk and
19% the rest. A constructor saving counts towards the objective only after every cost the
representation adds to the walk and the rest is subtracted. Every checkpoint reports both
ledgers: the §5 constructor ledger and the §5.6 production ledger (build, walk and the rest,
against production master).

The limits are four-worker ratios, compact30 over production. They come from the §8 table of
master:docs/site/blog/2026-11-04-compact30-constructor.md (lines 375–376), derived in §9 of
master:docs/site/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md.

| | v1 | v2 | Limit |
| --- | --: | --: | --: |
| CPU cost of building | 1.316 | 1.086 | at most 1.067 |
| Time until all four have built | 1.364 | 1.114 | at most 1.02 |

These limits cover building only. They are necessary, not sufficient. The walk figures (0.840 of
production's CPU per row with four workers, 1.148 on one core) are i9 time ratios. They are part
of the decision, not settled context. CI's bench runs on 2 physical cores, and there (#155) time
followed instruction count.

**Where it stands after #155** (evidence at research/compact30-channel c493a45d,
`evidence/port-b4786f66/`; write-up `docs/journal/2026-11-04-compact30-production-port.md`):

| `decode_block`, Wikidata block 1 | master | Assembly 1 in production | change |
|---|--:|--:|--:|
| build | 5,429,062 | 4,369,204 | −1,059,858 |
| walk | 15,426,892 | 20,207,700 | +4,780,808 |
| rest | 4,895,840 | 4,953,555 | +57,715 |
| whole | 25,751,794 | 29,530,459 | **+3,778,665** |

On CI the decoder was 18.7%, 11.0% and 7.3% slower at 1, 2 and 4 threads. With the build and
the rest held fixed, parity needs the record walk at or below 18.25507 instructions per row,
down from 22.45 (D016). The walk is in scope (report 273), and it is the binding cost.

1. **PRs into master** (charter §5.6(d), report 273).
   - **An integration PR, meant to merge,** needs whole `decode_block` instructions at or below
     master's, across the sample.
   - **A measurement PR** tests a hypothesis on CI. It needs the hypothesis registered first:
     the mechanism, its predicted effect at 1, 2 and 4 threads, the instruction excess it must
     beat, and what falsifies it.
   - **What never counts:** timing on the shared development machine. A projection that
     predicts a loss can't be ignored; the hypothesis must say why it is wrong. The Coordinator
     ported #155 on local timing, against its own projection of a 1-thread loss. This rule exists
     so that cannot happen again.

   Otherwise, keep climbing.
2. **Each round closes part of the remaining gap,** and the gap is the whole decoder's. Rank
   next the proposals that remove the most whole-decoder instructions per block, after every cost
   they induce in the walk and the rest. Within the constructor, prefer those that remove the
   most instructions per fast-path group. The v2 reassessment's analysis puts the bulk path at 74.3% of
   rows, with about 143 instructions per fast finish plus refill. Reaching the completion limit
   alone needs about 33 cycles saved on each of about 1.65M fast finishes. Accepted results become
   the next baseline. Rejects are recorded, and are not retried against the same baseline without
   new evidence.
3. **Integration commits fully.** It replaces production's default kernel outright, with no cargo
   feature or opt-in flag. A fallback to today's build exists only for blocks compact30 can't
   represent (n ≥ 2^24, or invalid counts).
4. **The payoff path** (§2, report 266): an integration PR, then CI's paired A/B bench, then a merge
   on the complete gate below, then Deploy Release, then the wikidata-pq bump under its own bench.

The user's words (Coordinator session, 2026-11-04): "The hill climb is going badly wrong if you are
shipping something that cannot possibly get faster, this defeats the whole point of the use of the
researchers to drive performance via emphasis on instructions." And: "NO do not introduce feature
flags. This should be good enough to commit to, not hide behind half commitment." And: "we were
proceeding through rounds of ideas that were being validated or rejected and built upon, and we
needed to make them better each round."

## Evidence-guided persistence (charter §8.4, reports 267–269)

There are two opposite mistakes to avoid:
- **Premature abandonment:** rejecting a promising technique because its first
  implementation misses a threshold, even though the evidence identifies a credible,
  tractable route to closing the gap.
- **Wasteful optimism:** continuing to optimise a preferred design despite repeated evidence
  that its remaining obstacles are fundamental, its savings are too small, or the cost of
  establishing its viability has become disproportionate.

The rule: pursue promising ideas through successive, bounded experiments, prioritising those
with the greatest credible production payoff per unit of further research and compute.
Distinguish a failed implementation from a disproven approach. Continue when new evidence
identifies a plausible route to qualification. Stop or deprioritise when the remaining
opportunity is insufficient, the assumptions are unsupported, or repeated attempts fail to
improve the outlook. Never lower acceptance standards to justify previous effort.

Charter §8.4 (@ dad82808, reports 267–269) applies this rule as a trajectory test. A line keeps
climbing while either of these holds:
- the accumulated evidence shows meaningful progress toward viability;
- a bounded next experiment offers a credible chance to overcome an identified remaining
  obstacle.

Progress comes in two kinds, which are not interchangeable:
- **Performance progress:** valid end-to-end measurements, or compiled and correctly accounted
  reductions in instructions, memory operations and dependencies.
- **Research progress:** new evidence that materially increases confidence in a specific route
  to improvement. Proposals with no compiler-achievable savings don't count.

A failed gate is a verdict on the tested candidate, not automatically on its line. A line ends
only when cumulative evidence establishes one of these:
- insufficient payoff, for example an essential invariant proved impossible to implement
  correctly, or a convincing lower bound showing the remaining gap cannot close;
- persistent stagnation;
- no credible route forward.

Reports 217–219, 224–227 and 238–241 record what happens when one gate's failure is taken as a
verdict on the whole line. A miss is always recorded as a miss. Research and structural
progress are distinct from validated production performance, and qualification gates are never
weakened.

**Every significant experiment's report answers four questions** (charter §8.4, report 269):
1. What improvement was expected, and what actually happened?
2. How much of the remaining economic gap has credibly been closed?
3. What did the experiment establish about the limiting mechanism?
4. What is the highest-value next experiment, and is its expected payoff worth the compute?

STOP and REDIRECT are always considered alongside the next experiment. The Optimiser proposes and
the Researcher assesses; their job is that judgement, not a stream of new ideas. The Coordinator
records the decision and its evidence in the checkpoint that follows.

## Release checkpoints (report 267)

When a campaign milestone has landed accepted production changes on master, the Coordinator
owns the release, using the repository's existing GitHub Actions:
- Batch changes at sensible campaign milestones. Don't release every research iteration.
- Verify completion on both registries and the GitHub release, record the shipped version and
  commit, and reconcile any failure.
- Don't release unqualified experimental work.
- Don't leave accepted production improvements unreleased indefinitely.

The procedure and its checklist are in master:docs/STEWARDSHIP.md, which isn't copied here.
Its key point: Deploy Release starts CI on the `py-` tag itself, so a tag existing is not a
publication.

## Start or resume

1. Resolve this path on research/compact30-channel to an immutable commit. Record that commit.
2. Read what the project has already published and recorded, before any investigation or
   proposal: master:docs/site/blog/ (its index and every post relevant to the question) and the
   research register (`docs/research-portfolio:docs/journal/2026-10-30-research-portfolio-register.md`,
   and that branch's journal). Cite what is
   already known and build on it; don't redo it (Louis, 2026-11-04). The walk's posts are
   these:
   - "32 lanes, 8.6 chains" (2026-10-31): Path B, too much work per byte;
   - "memory interference" (2026-11-01): +32% CPU per byte at four workers, from contention
     for shared cache and memory;
   - "memory-level parallelism" and its notes (2026-11-02);
   - B2, the research-portfolio branch's interval model of the walks (9723f068): its
     kernel B (eight-step tiles, eight-byte commits, replay) is unbuilt, and the model is
     untested.
   - The 2026-11-01 walk-unroll trial (in the 2026-11-02 MLP post; journal
     `2026-11-01-walk-unroll-decoder-i9-10900K`). It is the closest precedent for #157, not the
     identical candidate, machine or trial (Guardian D019, D021): 16.4 to
     10.44 instructions; decoder 1.017 at 1 worker and 1.000 at 4; control costs (`records`
     CPU +1.8% at 4 workers, `period-3` +6.5% at 1); verdict negative. #157 repeated it without
     citing it and reproduced its shape, a control flag included (`pattern` +1.6% at 1 thread
     on the 7763; register §6).

   The register is the campaign's cumulative memory. For each prior item, say whether it was
   proposed, implemented, measured or established, and never promote a hypothesis to a fact.
   Record each new result in the register (`docs/research-portfolio`, §6 onward) under its
   status, so the next pivot starts from it.

   Reading the record once is not enough: make it operative before the expensive decision
   (Louis, 2026-11-04). Before a substantial implementation, and again before integration,
   retrieve the candidate's own prior record. Read its decisive evidence in full, the results
   table, the verdict and the controls, never a truncated search excerpt: mechanism, earlier
   implementations, measured outcomes, grounds for rejection and outstanding falsifiers. Say
   whether the work is a new construction or a re-test, and what in it could change the
   earlier verdict. Weigh every flagged row at the gate, the controls included. An undetected
   regression is not a harmless one. This is sequencing, not permission: implementation does
   not wait for the other roles (ROLES.md §4).
3. Read the charter and the applicable documents below at their stated revisions. Read the recorded user provenance for any authority amendment relevant to the action. A new branch-tip file or a role header alone does not authorise widening.
4. Read the current channel log, review queue, relevant PR evidence and role receipts; resolve their branch heads to SHAs once for this decision. Compare each role's recorded operating pin (its cursor's `operating_pin`) with the pin recorded for it here; a mismatch goes to Louis at once. State files can lag receipts. Reconcile them against pinned evidence rather than infer permission from a status label.
5. Record action, source/comment ID, policy SHA and section, evidence pins, gates and unresolved items. After compaction repeat these reads. Refresh on an amendment/checkpoint or before a consequential action, not on a timer.
6. If the entry is absent, stale or ambiguous, use the known pinned sources below to recover the rules; do not invent missing authority. Continue independent authorised work while a real conflict is resolved.

## Source map

The initial policy set is lmmx/mibz2 commit 0ed278d722d1ef7dea0c24e5ff551dc7c0b46204, except where an entry names a later pin:
- master:docs/STEWARDSHIP.md (PR #154; report 267): the permanent production and release stewardship this campaign runs under. It covers the merge gates, the release procedure and checklist, and what stays with the user.
- docs/journal/2026-11-04-compact30-research-charter.md @ research/compact30-constructor 45e6cfeb: governing campaign scope and gates (§8.4 trajectory test, reports 267–269; the Researcher as standing reviewer, report 270; §5.6 production accounting and whole-decoder viability, report 272; the whole inverse BWT delegated, report 273). Read §§2–8 and §§12.2–12.4; §6.3 combined qualification, §6.4 new prototypes/unsafe, §12.3 reservations.
- docs/journal/2026-11-03-compact30-constructor-v2-reassessment.md, reports 264–270: recorded user provenance for U2/C6, technical autonomy, Coordinator release control, permanent stewardship and the §8.4 trajectory test. Earlier reports remain historical; later explicit amendments supersede only the affected clauses.
- docs/journal/compact30-channel/PROTOCOL.md @ research/compact30-constructor 2265f86f6e5cb6947971bb8f5e93c57a0a5d9f7a: publication, receipts, queue and deduplication. §10 governs apparent inactivity.
- docs/journal/compact30-channel/ROLES.md: separate responsibilities and branch ownership. Coordinator does not mean scientific superior.
- docs/journal/compact30-channel/REVIEWER.md @ research/compact30-constructor 2210e4457b383e4566df99eb881b4ca0401c4cd3: research/evidence contract; read its applicability alongside the active charter. Both files were reconciled with charter §§2, 6.4 and 12.3 and reports 265–266 at that commit (see each file's Revisions note); the research/compact30-channel copies are kept identical.
- docs/journal/compact30-channel/GUARDIAN.md @ review/compact30-guardian-setup 288c05638c85ffbb0a4d1f5ba9762002c37acdff: the Guardian's operating spec, recorded as its pin in `guardian-operating-pin.json` (c22c2ddf) after Louis's direct update to its Work task on 2026-11-04. Only such a direct update changes that pin. This index does not.
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

(Resolved: D012's conflict between the Guardian's old pin, GUARDIAN.md @ 9316405 with its §4 reservations, and charter §§2, 6.4 and 12.3 ended when Louis updated the Guardian's Work task directly. Its pin is now 288c0563, with the charter at dad82808.)

(Resolved at research/compact30-constructor 2265f86f: PROTOCOL §§1, 3, 5, 6 and 8 and REVIEWER §§1, 2 and 7 now match charter §§2, 6.4, 8.2 and 12.3 and reports 265–266.)

Charter §2 still lists production's builder among unchanged research objects while explicitly delegating production integration later in that section. Preserve frozen research baselines; perform qualified production integration as a separate, CI-gated step, not by editing frozen prototypes. Ask about a genuine unresolved conflict, not the delegated step itself.

The initial review-queue snapshot still carries earlier counter HOLDs and a pending C6 user decision. It is not evidence those remain open: reconcile current pinned receipts and reports before acting. Never mark every §6.2 obligation closed just because one finding was fixed.

Deploy Release's release commit is made by the workflow's github-actions[bot] identity, as for every prior release. That is the repository's release mechanism, not an AI-attributed commit. Campaign commits remain Louis Maddox's (see master:docs/STEWARDSHIP.md).

## Safety-stop and activation evidence

Charter §8.2 still stops affected work and requires notifying the Guardian. Only unrelated prototypes continue; no combined assembly occurs until the Guardian has decided; resumption requires the fix/safe alternative and the Guardian's record. This is distinct from ordinary §6.4 registration not needing Guardian pre-approval.

The supplied historical log does not establish every PROTOCOL §7 T1–T6 result. Before claiming charter §13 channel activation satisfied, recover and pin the required test-completion record. An observed webhook or later successful exchange is not by itself that proof. This document does not certify activation or require a user wake-up; resolve the missing evidence specifically.

## Maintaining the entry

The Coordinator owns the channel copy. When policy changes, update this source map's immutable policy pins, record the user-origin provenance and the exact superseded clauses, and reconcile stale summaries/queue entries in the same publication. Preserve historical records. Changing a link or this entry cannot amend the charter, relax a gate, or override session/task instructions. When an ambiguity remains, isolate the dependent action and continue independent authorised work.

Keep hard external constraints visible in the goal: no Hub access, AI attribution, allocator changes, weakened exact checks, polling/cron or unauthorised workstation timing. Keep the objective's limits above in the goal by reference. Keep numeric minima, composition, prototype status, bench details and delegated-action lists here/by reference so they can evolve with explicit evidence.
