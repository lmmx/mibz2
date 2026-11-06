# Team-neutral audit: moving the files does not remove the inherited premises
Researcher, 2026-11-06. [Assignment 6684938042](https://github.com/lmmx/mibz2/pull/162#issuecomment-6684938042). Reviewed #225 at **40709781a39758d14e037d70a9bb20a65fc0edf9**. Governing spec remains **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**; this review does not adopt the draft or migrate my state.

## Finding
The three-way separation is sound. The audit is incomplete at the **live references and substantive predicates**, not merely the names. C4's instruction-gate retirement conflicts with the very permanent stewardship document A6 retains. MODE is not premise-free. C3 needs an explicit opportunity-cost comparison and a distinction between target-machine research and CI adoption. State migration is safe only as a verified handover of outstanding obligations, not a copy of processed IDs.

These are recommendations to the Coordinator and Louis, not new HOLDs or authority grants. No implementation, timing, migration or edits outside the Researcher branch.

## Sources
Unless otherwise indicated, paths below are at the reviewed #225 pin:
- [audit](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/journal/2026-11-06-team-neutral.md), [RULES](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/team/RULES.md), [MODE](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/team/MODE.md), ROLES, PROTOCOL and SPEC files in that directory.
- [STEWARDSHIP](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/STEWARDSHIP.md), CLAUDE.md, bench/CHANGELOG.md, bench/report.py and .github/workflows/bench.yml.
- [archived CAMPAIGN](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/journal/compact30-channel/CAMPAIGN.md) and 2026-11-04-compact30-research-charter.md.
- [public swarm post](https://github.com/lmmx/mibz2/blob/40709781a39758d14e037d70a9bb20a65fc0edf9/docs/site/blog/2026-11-04-autonomous-agent-team.md).
- [portfolio register](https://github.com/lmmx/mibz2/blob/d25f0a4f1b33754d980a0c6b4c5b8eaf7b530563/docs/journal/2026-10-30-research-portfolio-register.md), fetched from its branch, whose head resolved to that pin.
- Coordinator pivot comments [6643481606](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643481606) and [6643744616](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643744616).

This is a targeted source audit, not an exhaustive repository, historical authorization or automation-settings audit.

## 1. Premises missed or incompletely removed
**STEWARDSHIP reimports C4.** It explicitly requires whole decode_block instructions at or below master's before an integration PR, with a separate measurement-PR route for mechanisms instruction counts miss. It also lists compact30 as the current campaign and requires campaign-document reads. A6 makes this document a live authority source. Retiring the campaign ledger in RULES alone leaves the count ceiling operative through another path. Reconcile the exact clauses under Louis's approval; until then acknowledge the conflict rather than claiming counts are merely diagnostic. Stewardship also still reserves workload/statistical-method changes and workstation captures, so C5/C6 need corresponding explicit precedence or amendments there.

**MODE inherits substantive choices.** Its first sentence says the charter governs authority. Its Optimiser stanza says “Work? Ir — not time”; later sentences correctly price the binding path. Its top objective remains “faster production inverse BWT,” whereas the permanent stewardship objective is a faster correct production decoder and the 4 November pivot explicitly included scanner, MTF/RLE, CRC and assembly. The audit's categorical “MODE.md itself inherits nothing” is false even without a compact30 token. Name-neutral language is not neutral scope. Preserve Louis's chosen style; propose only the necessary authority reference and objective/metric clarification for his decision. Do not interpret “inverse BWT” as silently authorizing all downstream work or as silently excluding other decoder stages.

**CAMPAIGN has more than two economic limits.** Besides 1.067 build CPU and 1.02 completion, it imposes whole-instruction integration parity, instruction-ranked opportunities, full kernel replacement without a feature flag, and a particular workload/thread regime. Archive these as historical constraints with explicit applicability. Do not silently carry them forward through a source map. Conversely, matched baselines, charged induced costs, preserved rejects and meaningful whole-decoder benefit remain useful without those constants.

**Researcher lineage is not its scientific remit.** The old ROLES §3 already says the scanner assessment does not define or limit the wider role. Keeping the old path during a pinned transition is provenance, not proof of a stale intention. Retirement of the specific §6.2 sequence is reasonable, but retain the general independent-review contract: pinned evidence, independent verification, disclosure of involvement, material-objection resolution. The proposed role still describes counterexamples “before consenting”; consent must not become a new permission gate. B5 supplies safety-review work, not the entirety of this role's broader scientific remit.

**The portfolio is historical evidence, not today's queue.** At d25f0a4f its P87 row still says open/not run; section 6 retains the four-worker campaign objective and the instruction-parity framing. These are time-scoped records, not an authority to replay work or reject a new regime. This Researcher's later cursor already records the P87 results review (6646250444 → 6646303348). A neutral live index should link superseding results and say which snapshot it covers. Preserve the register's useful kernel/stage/decoder/application distinctions and its failure classes, especially machine-dependent results. Do not erase old verdicts or promote “rejected in one regime” into mathematical impossibility.

**The blog can recirculate old rules.** The swarm post's early sections describe instruction-led optimization, the compact30 charter and an isolated reviewer; later material describes changed arrangements. Keep it as an account of those events, but distinguish historical process from current instructions with a short dated link to the live machinery. Do not rewrite the history to pretend the new rules always applied. This is particularly relevant because the old start procedure sends agents to blog/prior-art material.

**Bench versioning is not authorization or experimental adequacy.** CHANGELOG classifies corpora, scale, rounds, thread counts, timing and stage methods as major changes. A major version records changed meaning; it does not certify that the new workload represents the objective or authorize the change. The CI workflow selects 1,2,nproc when nproc>2. It therefore does not necessarily exercise the i9's 8–12-thread question. A target-host result can answer its registration while failing to establish improvement on CI; neither should be promoted into the other. Preserve the distinction between research evidence, candidate qualification and adoption.

**Lapsed transcription and stages:** audit them separately from authority. A missing log entry proves a record gap, not revoked technical delegation; a completed old pilot does not prove new state paths/routing work. Retire the bootstrap labels, retain a bounded cutover readback. GitHub comments are editable/deletable: the channel is convenient but not an immutable substitute for a committed verbatim snapshot. Preserve compact decision provenance (ID, captured words or content digest, observation time and applicable policy) without requiring transcription of every routine message.

## 2. Disposition review
“Retain” below means its purpose survives; it does not certify that its current wording or authority provenance is complete.

| Row | Assessment and minimal correction |
|---|---|
| A1 | Retain the authority boundary. Distinguish reversible focus choice from an explicit scoped HOLD/restriction; calling something focus must not let an agent undo a binding narrowing |
| A2 | Retain cautious handling of real conflicts. “False HOLD costs only time” understates lost opportunity and denial of service. Record scope/reason and resolve through existing channels; do not automatically revoke it |
| A3 | Retain external comments as evidence rather than authority. Do not discard useful outsider technical findings |
| A4 | Retain authenticated widening and captured words; name the approved policy revision and affected clauses |
| A5 | Retain technical autonomy within the mandate; it is compatible with abandoning a productive but lower-value route |
| A6 | Retain permanent stewardship, but reconcile its live instruction gate, reservations and current-campaign reference with C4–C6 |
| A7 / D | Adapt only through explicit approval of the actual changed reservations, including independent-review dispute handling |
| B1 | Retain byte-exact correctness; retain applicable malformed-input, safety and API obligations too |
| B2 | Retire compact30 representation predicates as gates for unrelated work. Preserve them as historical evidence and when assessing code that still relies on that representation |
| B3 | Retire ongoing campaign freezes, not the immutability/reproducibility of the evidence snapshots themselves |
| B4 | Retain original registered outcome, but permit explicit corrected calculations and later interpretations beside it; “never reinterpreted” must not fossilize a known error |
| B5 | Retain safety arguments and relevant rigorous verification plus independent review. Preserve proof scope and reviewer involvement disclosures; the package is not mere test-name compliance |
| B6 | Retain scoped immediate stops. A pinned file is not itself an invariant; say which registered invariant or frozen artifact a change violates and which work shares it |
| B7 | Retain partial-coverage honesty |
| C1 | Adapt registration location. Preserve prospective quantitative tests, but allow bounded diagnostic/research questions with an information objective rather than pretending every experiment promises speed |
| C2 | Retain prospective gates and original verdicts. A later stricter rule is a separately identified admission constraint, not a retrospective rewrite of the original experimental result |
| C3 | Retain evidence-guided persistence, with opportunity cost and explicit PAUSE/REDIRECT as well as STOP. Separate target-regime viability from mandatory CI adoption |
| C4 | Retire the compact30 attempt cap, composition sequence, V3-specific rules and instruction-only gate. Retain whole-cost accounting and independent safety review; reconcile STEWARDSHIP before claiming the count gate is gone |
| C5 | Adapt with explicit approval. Louis executing one exact frozen recipe can authorize that run; it is not blanket authorization for future captures, changed methods or agent timing. Hardware/workload validity still limits the conclusion |
| C6 | Adapt only with a clear reservation boundary. Define workload/corpora, statistical estimand, multiplicity/repeat policy and acceptance criteria, not merely “paired method.” Versioning alone cannot authorize changes to these |
| D retired rebuttal reservation | Retire the campaign-specific §6.2 route if approved; do not make the challenger permanently sovereign. Record an evidence-based resolution/escalation route for unresolved material safety disputes |
| E1 | Retain source/rule/action logging, including policy SHA and supersession |
| E2 | Retirement of full transcription is reasonable if accepted; do not call mutable comments immutable. Retain durable decision/authorization evidence |
| E3 | Retire stage labels and completed bootstrap obligations; perform a migration-specific smoke/readback check without resurrecting perpetual stage gates |
| E4 | Retain authorship, no force pushes and shared-machine protection; clarify identity exceptions already used by release workflows |

C3's omitted comparison matters: a route can keep making tiny real progress forever while another available route offers much more expected payoff. “Some credible chance” is not a sufficient priority rule. Keep exploring when its marginal expected performance **or information** value warrants the compute against alternatives; de-prioritizing does not require proving impossibility. This is already closer to CAMPAIGN's evidence-guided-persistence prose than C3's shortened retention.

C4 should demote counts from sufficient/necessary performance qualification, not from scientific importance. More instructions can improve overlap, locality or contention; fewer can fail to improve the critical path. Still account for induced work, memory, synchronization and control regressions. CI measured-head/parity/regression rules remain; this review does not waive them.

The audit says only ⚑ rows change substance. B2/B3/C3/C4/E3 retire enforceable provisions without that marker; A1's new narrowing qualification can also change behavior. Approval should cover the full exact disposition table, not only the marked rows. Its interim “charter applies, read as covering the whole team” is itself not permission to generalize a campaign's scope. Separate the already evidenced standing delegation from the proposed replacement.

## 3. Orphan state: separation is not provenance
An orphan removes Git ancestry, not the existence of old objects while their branches remain. Immutable commit URLs can still work; relative links after relocation, git log/blame traversal, ancestry checks and discovery do not automatically survive. A migrated_from SHA is a pointer, not a proof that the complete state was copied.

Prefer **continuation branches for role state**: branch from the final old head, then relocate live state under state/ and remove unrelated tracked files in a normal commit if desired. The current tree can be neutral while its audit ancestry remains intact. Historical ancestry grants no authority.

An orphan observatory is reasonable for a deliberately separate publication history. Explicitly link its predecessor and preserve a migration manifest. If orphan role state is preferred, it is viable with verified manifests: source repo/ref/SHA and paths; destination paths and blob/content equivalence; all processed IDs, offsets, in-progress/complete/finished entries, review IDs, evidence links, unresolved HOLDs and decisions. Preserve old branches; fix relative links or make them immutable URLs.

Cutover must wait for **finished**, not merely complete: the common protocol publishes a REVIEW after complete and then writes the REVIEW ID/finished cursor. “No assignment in flight” can be misread as the dispatch-ready moment before that final write. Capture final heads after writers have ended, validate copies, load the new pins, read back exact task/state paths, then resume dispatch. Do not replay assignments to test migration.

The new SPEC step 3 explicitly resumes in-progress but omits complete. Preserve the old resume behavior: a complete entry may only need deduplication/publication/finish, not recomputation. Queue recovery and the coordinator's cursor readers must consume the new paths. A cosmetic rename is not proof those readers were migrated.

Oracle publication and its cursor now live on two refs. They cannot advance atomically: commit the record, store its SHA in cursor progress, deduplicate REVIEW, then finish; resume against that SHA after interruption. An orphan branch itself also does not disable push-triggered CI—verify workflow/ref filtering rather than infer “no CI” from topology.

## 4. Would the Oracle have caught the incident?
It could flag applicability early, but not infer an authoritative campaign retirement from a failed candidate alone.

At **2026-11-04 22:14:04 UTC**, 6643481606 calls the record walk a detour and freezes the constructor line as evidence. That suffices to question why current purpose still points to that representation; it does not rescind general safety or stewardship rules. At **22:27:46 UTC**, 6643744616 explicitly redirects the search to the whole decoder and questions instruction binding. That supplies a concrete mismatch with an exclusively instruction-minimizing remit, irrespective of the old name. The stated bottleneck inference itself remained uncertain; Oracle need not certify it to flag the mismatch.

The new check requires “intention the record shows abandoned.” That is too late if the failure is **applicability drift before a formal abandonment declaration**. Minimal extension: flag a live premise, authority source or fixed scope whose applicability is unestablished after an explicit objective/regime change. Record which clauses still serve a standing objective and which need renewed disposition. Do not infer authority expiry from naming, ancestry or a negative result.

At the latest, this #225 brief explicitly establishes abandonment as the premise for an audit. The record supports an earlier mismatch flag; it does not support my inventing an earlier exact date when every compact30-derived rule lost authority.

## Minimal correction and consequence
1. Reconcile RULES with the still-live STEWARDSHIP and MODE clauses; put substantive dispositions, including retirements, before Louis at an exact pin.
2. Restore C3's marginal-value comparison and separate research success on the target regime from CI admission.
3. Prefer continuation state, or verify the full manifest and finished-state handover; explicitly resume complete entries.
4. Make Oracle check applicability after pivots, not only explicit abandonment.

The decisive lesson is **authority and evidence have different lifetimes**. A failed campaign can leave valid permanent stewardship, reusable evidence and useful scientific rules. A neutral directory can still carry an obsolete gate. Audit live uses of premises and reservations, not origin alone. This avoids both inherited lock-in and unjustified deletion.

ORACLE: team-neutral audit / MODE — “MODE inherits nothing” conflicts with its charter-authority preamble, Ir framing and inverse-BWT objective; A6 also reimports the instruction gate through STEWARDSHIP — #225 at 40709781, MODE and STEWARDSHIP.

No policy has been adopted or changed by this review. The original inbox and state remain operative until Louis changes the pinned instructions.
