# 2026-11-10: Ownership, the governing files amended

**Status: decision record (RULES E2).** This is the record the Guardian asked for in D096, "the
established E2 record", for the change to the team's governing files made on this date. It
applies **prospectively**: from the commit that merges it, forward.

## 1. Source

Louis, in the Coordinator's session, 10 November 2026 (RULES A4, D1). His key lines:

> "Make concrete, persistent changes to the governing files and role instructions"
>
> "Replace open-ended review loops with bounded decisions: merge, make one targeted correction,
> defer or abandon"
>
> "Keep correctness and memory safety non-negotiable"
>
> "Preserve historical experimental verdicts. Change acceptance policies prospectively."

The governing intent is [OWNERSHIP.md](../team/OWNERSHIP.md), set by him the same day. Two
inputs were applied on top of it:
- **the Coordinator's audit** of the governing files (rows 1–19 and sections a, c, d, e below);
- **the Pathfinder's ownership proposal**, PROPOSAL 6764559374 on #145
  (`docs/journal/2026-11-10-ownership-proposal.md` at `23fb769c`), adopted in part at Louis's
  direction in the Coordinator's session.

## 2. The diagnosis adopted

The Pathfinder's diagnosis is adopted:
- **The one-builder ratio.** Only the Coordinator could change the decoder; four roles produced
  documents for it. Review capacity was several times build capacity, and review grew to fill it.
- **Unbounded objection.** A HOLD cost its author nothing, had no expiry, and could rest on a
  constructed counterexample rather than an estimated risk.
- **Product decisions governed as scientific claims.** A reversible merge was handled as an
  irreversible published finding: pre-registered, reviewed twice, never relaxed.

## 3. What changed

| # | file | old gist | new gist |
|---|---|---|---|
| 1 | RULES C6 | every choice pre-registered, Researcher's PROCEED, Guardian rules on any gate, safety or admissibility question | an **adoption registration** is frozen with one review (PROCEED, or one correction and the Coordinator's decision); the Guardian rules only on a HOLD on gate or safety grounds; other experiments need a pre-data decision rule (C1a) |
| 2 | RULES C1 | full registration for every experiment | **C1a:** diagnostics, probes, counts and exploratory timings need a 3-line pre-data post (question, decision rule, integration decision, date); they never qualify a change |
| 3 | RULES D | an unresolved B5 objection goes to Louis (G6) | after one targeted correction the Coordinator defers or abandons; never merged with the objection open, not escalated |
| 4 | RULES A2 | ambiguity or conflict counts as a HOLD | an ambiguous **narrowing** message holds only the work it names; other ambiguity the Coordinator resolves and logs |
| 5 | PROTOCOL §4.2 | treat ambiguity or conflict as a HOLD | ambiguity in a HOLD/STOP holds the named work; other ambiguity the Coordinator resolves and logs |
| 6 | PROTOCOL §5 | reviewers who disagree are escalated to Louis | not escalated: the Coordinator decides, logging both positions |
| 7 | PROTOCOL §4.1 | an ACK or LOG for every message | log decisions, not receipts: one LOG per wake; routine REVIEWs need no ACK |
| 8 | PROTOCOL §11 | build the best construction once a review allows; the line stays open until Louis closes it | build it if its priced ceiling beats the next-best line; one sharpening round per arm; otherwise the Coordinator parks it with a reopening condition and tells Louis, who may reopen it |
| 9 | PROTOCOL §9 | a queued workstation run needs a PROCEED | a PROCEED for an adoption registration; the Coordinator's own check of the frozen recipe for a diagnostic |
| 10 | STEWARDSHIP | the target-gated route, conditions 1–7 (D071, D072) | PERFORMANCE-POLICY §2: target-machine gain (C5), a deterministic CI route check, a neutral CI bench; the old route retired, kept as history |
| 11 | STEWARDSHIP | count the instructions before a kernel PR; excess needs a mechanism written first | the PR states mechanism, predicted effect and falsifier; CI's end-to-end bench decides; counts optional, and a count result gets a CI wall-clock bench within one working cycle or is dropped |
| 12 | PERFORMANCE-POLICY §3 | a test must be one the Researcher and the Guardian agree is sound | the standing CI paired bench and sign test are pre-approved; a new test needs one review (Researcher); the Guardian only on a HOLD |
| 13 | CLAUDE.md "Decisions" | ask the Guardian and proceed on its ruling | the Coordinator decides, records why, carries on; asks the Guardian only on a registered gate's admissibility or a safety call, and proceeds if no HOLD comes within one assignment cycle |
| 14 | CLAUDE.md "Delegated as a class" | pre-registered, Researcher's PROCEED, Guardian on gates | mirrors row 1 |
| 15 | ROLES Guardian | reproduces any figure a decision depends on; unreproducible is a HOLD | recomputes from the archived bundle the figures an **adoption** decision depends on; a HOLD on that decision only |
| 16 | SPEC.PATHFINDER 3a | the design waits for the Optimiser's price | waits one assignment cycle; then the Coordinator picks the cheapest falsifier and proceeds |
| 17 | SPEC.PATHFINDER | queued once the Researcher has said PROCEED | queued under PROTOCOL §9; a diagnostic needs no PROCEED |
| 18 | CLAUDE.md, Observer | a CHECKPOINT per material move, one at a time | one batched CHECKPOINT per wake |
| 19 | PERFORMANCE-POLICY §2 | more than 10% secondary loss needs Louis's sign-off | kept; the Coordinator may also defer or abandon such a change without asking |
| a | ROLES "How the roles relate" | (none) | **Success is delivered speed**, with a measure for each role |
| c | PROTOCOL §2a; RULES C3 | (none) | **Bounded review:** one round, one targeted correction, then merge, defer or abandon; enlarging findings become follow-ups; an open B5 objection can only defer or abandon |
| e | PROTOCOL §2, §3; `coord.py` | QUEUED items dispatch by age | a `PRIORITY: 1 \| 2 \| 3` header (QUEUED only; default 2); dispatch by priority, then age; tested |
| P1 | ROLES; SPEC.OPTIMIZER, SPEC.PATHFINDER | the Optimiser "does not implement, time anything"; the Pathfinder "does not change `src/`" | deleted. Any role may open a PR against anything; the author owns it to merge. The Pathfinder is the **Architect** (owns the ranked performance backlog, builds its top item); the Optimiser a **second implementer**; the Coordinator the **Lead**, deciding merges. Names, headers and inboxes unchanged |
| P1 | ROLES; SPEC.GUARDIAN, SPEC.RESEARCHER | HOLD where material proof is absent; counterexample before consent | HOLD and STOP bind only under B1, B5, B6, B7; on performance evidence they advise once; nobody reviews their own PR |
| P2 | RULES O (new) | (none) | O1 any role may open a PR; O2 one reviewer, one round, four outcomes; O3 binding objections only under B1, B5, B6, B7 |
| P3 | RULES A2 | (none) | a HOLD on performance evidence states its harm (who, how likely, how large), its cost in days and the smallest change that lifts it; it lapses after one round unless the Coordinator concurs |
| P4 | RULES C1, C2 | registration for every experiment, never relaxed | full registration for published findings and for adoption decisions outside PERFORMANCE-POLICY; a merge under PERFORMANCE-POLICY needs none. Kept explicitly: **a decision rule written before the data is never moved after it** |
| P5 | RULES C5 | each workstation recipe registered and frozen | standing recipe templates (A/B of any two commits on the release-scaling harness, the gap block, share-nothing): filling in commits needs no review or new freeze; only new harness code is reviewed |
| P6 | STEWARDSHIP | measured code merges only on an end-to-end improvement beyond noise | merges under PERFORMANCE-POLICY.md; a **maintenance route**: neutral on the bench, better code, merges on green CI and one review; target-gated route retired as history |
| P7 | MODE.md | (Louis's register) | the "Own it." stanza first; the Coordinator, Pathfinder, Guardian and Researcher stanzas amended as the proposal's §3 wrote them; footer amended |
| P8 | OWNERSHIP §5 | (none) | expiry by default (24 h for a ready PR, 48 h for an investigation without an integration decision); a 30-day sunset for process rules outside RULES A and B; a three-line retro at each release; the reversion signal |
| P9 | CLAUDE.md | "ask the Guardian, not Louis" | "decide, record why, carry on" (with row 13) |

Consequential edits, to keep the files consistent: PROTOCOL §4.3 refuses a Louis-reserved
request "in the LOG" (it said "in the ACK", which row 7 removed); SPEC.PATHFINDER's "a change
worth a few percent is not its subject" now reads "its first subject is the largest shippable
gain", following the adopted MODE stanza; RULES and MODE carry amendment lines.

## 4. What is retained

- **Safety, unchanged:** B1 (byte-exact output), B5 (unsafe-code obligations and their
  independent review), B6 (immediate stops), B7 (partial Miri coverage is never a PASS). An open
  B5 objection blocks merge; it can only defer or abandon.
- **C2 and the honesty rules:** a registered gate is never relaxed after its data; a decision
  rule written before the data is never moved after it; every attempt is retained, no retry to
  pass.
- **Past verdicts stay as recorded** (RULES B4). Registrations frozen under the earlier routes
  (D071, D072, #324's no-harm rule) keep their verdicts. A change is judged under the new
  policy only on fresh evidence, as #324 was.
- **Unchanged:** B5's evidence list, E4 and the attribution rules, the merge-head checks
  (`coord.py head-sha`, `expectedHeadSha`), the shared-machine limits, RELEASING §2–3, and what
  RULES D reserves to Louis.

## 5. Considered and deferred

From the Pathfinder's proposal, challenged by the Coordinator and not adopted now:
- **Merging the Researcher and the Guardian into one Reviewer.** A role change mid-flight
  (RULES D8); bounded review gets most of the benefit. Left to Louis.
- **Deleting PROTOCOL §2's QUEUED and one-assignment machinery** in favour of the PR as the unit
  of work. Kept, with priority ordering, for now; a later simplification.
- **Backlog, as separate code PRs:**
  - **B1.** A generated `docs/team/SCOREBOARD.md`: released MB/s on `wikidata` and `text` at 1
    and 10 threads per release, days since a release carried a primary gain, `src/` PRs merged
    this week with median days from first measurement to merge, open HOLDs with their age.
  - **B2.** A CI job that reads the bench and prints the PERFORMANCE-POLICY verdict (ACCEPT,
    REJECT or LOUIS).

## 6. Applies prospectively

Every change here governs work from the merge of this record forward. Nothing in it reopens,
relabels or re-judges a recorded verdict.
