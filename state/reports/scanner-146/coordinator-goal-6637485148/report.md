# A persistent Coordinator goal that survives policy changes

Trigger: 6637485148. Research-only drafting and simulated-reader test; no campaign execution or authority change.

## Recommendation

Use one maintained entry document at a discoverable branch tip, then pin every decision's document and evidence revisions. A permanently hard-coded policy SHA becomes stale; an unrecorded “latest” reference makes decisions irreproducible. The proposed combination separates discovery from evidence: resolve CAMPAIGN.md once, follow its immutable policy map, validate amendment provenance, and log exact SHAs. Refresh after compaction or a relevant change, never by polling.

There is no suitable single Coordinator entry in the inspected tree. The charter governs authority but is not the state index; REVIEWER.md is another role's brief; the queue and log are historical/state records. Draft CAMPAIGN.md is navigation only, for the Coordinator to adopt on research/compact30-channel. Do not pin the goal until the entry exists there.

## Pasteable goal

Run the compact30 campaign in lmmx/mibz2 autonomously as its implementation Coordinator, working with the Optimiser, Researcher and Guardian. Aim for verified production speed-ups through qualified combined builds, integration PRs, CI bench, merge, release and a separately gated wikidata-pq bump.

At start, after compaction and before consequential decisions, resolve research/compact30-channel:docs/journal/compact30-channel/CAMPAIGN.md to a commit; read its source map and applicable documents, verify amendment provenance, and log the exact revisions used. Refresh on policy changes; do not rely on remembered policy.

Proceed with documented delegated work without renewed approval; escalate only actual reservations or unresolved authority conflicts. Honour safety HOLDs and STOPs. The entry document is navigation, not new authority, and cannot override this session's instructions or another role's task limits.

No Hub reads/writes/API calls, AI attribution, allocator changes, weakened exact checks, cron/polling, or unauthorised i9/workstation timing. Never merge an unbenched head or treat silence as approval.

Character count: 1123, excluding this heading and count.

## What stays inline

Keep the role, lookup/recovery procedure, source-of-authority boundary and externally consequential prohibitions inline: no Hub access or AI attribution, no allocator changes or weakened exact checks, no cron/polling, no unauthorised workstation timing, measured-head merge requirement and silence is not approval. These guard against acting before lookup. Keep numeric minima, status, composition, authority lists and workflow recipes by reference; duplicating them recreates the drift.

“Coordinator of the Guardian, Optimiser and Researcher” becomes “working with” them. ROLES.md gives separate scientific responsibilities; it does not make the other roles subordinates. “Never ask the user” becomes “without renewed approval for documented delegated work”: it cannot override real §12.3 reservations or a narrower directly supplied task boundary.

## Sources and maintenance

| Source | Use | Reference strategy |
|---|---|---|
| CAMPAIGN.md on research/compact30-channel | fixed entry path | resolve tip to SHA at start/resume/decision |
| charter §§2–8,12 | authority, gates, stops, reservations | immutable pin in entry; initial 0ed278d7 |
| reassessment reports264–266 | recorded user provenance | same policy pin; do not infer authority from role headers |
| PROTOCOL, ROLES, REVIEWER, GUARDIAN | receipts, responsibilities, review contract | same initial pin; respect actual role task pins |
| log, review queue, role cursors, canonical ledger, PR evidence | current work and receipts | resolve each relevant branch to SHA; pin comments and evidence |
| mibz2 bench/deploy workflows and ship recipe | actual mechanisms | inspect at proposed merge/release head, not remembered policy pin |
| wikidata-pq CLAUDE.md, bench README/workflow | downstream constraints and gate | initial inspection 6e927acf; re-read/pin downstream candidate |
| wikidata-pq issue97 | downstream work queue | current state, timestamp and relevant decision IDs before bump |

The report's b771d56a log is useful historical provenance, not a current state anchor. The proposed entry explicitly treats it that way. Future updates to the entry must change immutable policy pins with authenticated user-origin provenance and reconcile summaries; a file edit alone grants no authority.

## Conflicts found

The initial PROTOCOL §§1/3 still reserve prototype additions and production/master/release work to the user. Explicit charter amendments in §§2/6.4/12.3 and reports265–266 supersede those specific Coordinator reservations. Preserve unaffected protocol safeguards and narrower Work-task instructions. The Coordinator should repair these stale paragraphs on its own branch; the draft entry names the conflict so a compacted reader does not ask for redundant approval.

The cold reader additionally found REVIEWER §2's contradictory Guardian/user registration clauses and PROTOCOL §5's obsolete blanket safety escalation. The revised entry names both, preserving a stop for undefined behaviour and the actual unaccepted-rebuttal reservation while routing routine fixes through the amended charter.

The initial queue still shows old counter HOLDs and a pending C6 user decision; reconcile it from pinned receipts rather than use it as an authority source. The charter's frozen research-builder rule and later production-integration delegation apply to different stages: don't edit frozen baselines when integrating qualified production code.

A 95% interval excluding zero and parity alone is not the complete merge predicate. The direction must be beneficial, CI green, regressions acceptable and the exact head measured. A downstream bump has its own gate; an upstream win is not downstream evidence. Package release control does not permit Hub dataset operations.

The Deploy Release workflow was inspected, not executed. It currently configures github-actions[bot], whereas campaign text requires Louis for campaign commits. Before release, reconcile the applicable automated-release identity rule rather than silently claim that workflow produces Louis-authored commits. This is a concrete execution detail to resolve, not a request to repeat the already-recorded release delegation.

The cold-reader packet lacked the full T1–T6 completion record required by charter §13/PROTOCOL §7. The entry now instructs recovery of that pinned evidence and preserves §8.2's Guardian decision/record requirements for an actual safety stop. These are specific evidence/process conditions, not new approval requirements for ordinary delegated work.

## Simulated-reader result

One fresh, history-isolated Coordinator reader received only the goal and its referenced source packet. All six decisions were correct; an informed revision check passed all six after making stale-clause conflicts explicit. Case (c) correctly remained conditional on the full bench gate, not just the two facts in the prompt. See reader-tests.md for isolation, findings, revisions and limits. No campaign operation was performed.

## Adoption

The Coordinator should copy the proposed CAMPAIGN.md to research/compact30-channel, reconcile its stale policy summaries and state entries, publish the adoption commit, and then give goal.txt to the user for /goal. This Researcher changed only its own review branch.
