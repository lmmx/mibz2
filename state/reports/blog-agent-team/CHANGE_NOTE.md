# Blog revision: method, evidence and reader checks

Input: [post at 648c84e5](https://github.com/lmmx/mibz2/blob/648c84e5d4a3fb6c1cb77503ed76abff42c18e89/docs/site/blog/2026-11-04-autonomous-agent-team.md), superseding 9fa6795e.
Requests: [6633675410](https://github.com/lmmx/mibz2/pull/152#issuecomment-6633675410) and [6633777567](https://github.com/lmmx/mibz2/pull/152#issuecomment-6633777567).
Drop-in destination: `docs/site/blog/2026-11-04-autonomous-agent-team.md`.

## What changed

The title, front matter and first-person dek are preserved exactly from 648c84e5. The post now makes its target claim in the opening: routine research handoffs are demonstrated; their effect on research throughput is unmeasured. It introduces mibz2 and compact30 before the workflow.

Seven sections answer the reader's next question: who does the work; what is handed over; how it arrives; how completion is recorded; what authority it carries; how its conclusions are reviewed; and what the record establishes. The Coordinator comes first, followed by the paired Guardian/Researcher readings and the Optimiser's instruction-reduction work. The whole team shares one repository workflow.

The rewrite uses first-person British prose, with no body bullet lists and one message-type table. It removes the before/after transport narrative, repeated checklists, and claims about the arrangement's productivity. It preserves the requested title without proposing an alternative.

Both `bf-fig` wrappers remain at the points they explain. Their titles, captions and HTML comments specify the responsibility map and common handoff sequence for the Coordinator's redraw. Old SVGs are deliberately removed under the addendum; the Coordinator still needs to supply the final drawings and render the site. No usage, cost or quota figures were added.

The style references were the memory-level parallelism, constructor and BWT tradeoff posts at master commit `a5fa5984a0b5218508df2bfbc488e07a7700a9f9`: concrete introduction, numbered explanatory sections, technical examples, figures attached to claims, and a separate statement of the evidence boundary.

## Facts tightened and their sources

| Claim | Treatment and source |
| --- | --- |
| Every new execution is fresh or blind | Distinguish execution from its supplied context. [REVIEWER §5](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md), [ROLES §5.7](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md), and the [intake record](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md). |
| REVIEW replies cannot trigger tasks | Explain event delivery separately from role filtering. The Researcher accepts addressed messages independently of type; [ROLES §6](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md) records different filters. |
| Each checkpoint automatically receives both readings | Present paired responsibilities and an illustrative dual-address checkpoint; explicitly condition coverage on routing and filters. Same role record; the supplied Researcher task stops when it is not addressed. |
| Cursors guarantee idempotence and branch ownership prevents concurrent writes | Describe deduplication and single-role ownership without claiming transactional publication. The saved task orders publication before cursor saving. [REVIEWER §3](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md) supplies the review key; interruption and concurrent-execution validation are recommendations, not claimed completed tests. |
| Only the team's agents can comment | Attribute recorded interaction restrictions and named locks to [PROTOCOL §1](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md); do not equate collaborators with agents or claim to have rechecked live access settings. |
| Message headers establish authority | Name the user's decisions in the Coordinator's session and distinguish behavioural rules from tool permissions: [charter §12](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md). [PROTOCOL §§4–5, 8](https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md) provide the receipt and pending-work path. |
| An observed loop proves general autonomy or speed | State D007's setup intervention and D008's real assessment at the scope recorded, with independent review outstanding: [channel log](https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md). The snapshot is explicit; it is not updated by inference from later activity. |

The requested channel-branch snapshot did not contain ROLES.md or GUARDIAN.md. Their constructor-branch versions were used at the immutable pin above. The channel's older protocol has historical wake-up limitations; the later role record and event log supply the current recorded mechanism. No claim is made that all historic specification text has been reconciled.

## Simulated readers: before and after

These were three separate, history-free editorial simulations, not human testing or measured comprehension. Each received only the original post for project context, recorded beliefs, confusion and transferable practices section by section, then read the revision. The same reader instances were used for the second pass. [The condensed section records](reader-tests.md) retain that comparison.

| Reader | Before | After |
| --- | --- | --- |
| Agent-tooling engineer | Understood the role split but could not locate reliable recovery, the actual routing topology or the authority boundary. | Found one traceable handoff, explicit filtering and honest publication/concurrency limits. Requested the precise human channel and deployment-evidence boundary; final sentences now name both. |
| Systems programmer | Connected the roles to optimisation but found Guardian gating, message delivery and independence intertwined. | Found the workflow and evidence boundary coherent; requested the hold-delivery path and recovery status. The final pass names the Coordinator's subscribed PR and distinguishes delivery observations from unvalidated interruption/concurrency cases. |
| Newcomer | Lacked project orientation and expected performance results; repository topology and fresh-review identity were unclear. | Could explain the project, handoff and separate independent assessment. Requested the human decision channel and recovery status, addressed in the final pass. |

The final clarifications are editorial responses to the second-pass comments; no third full cold-reader trial is claimed. The title remains as requested, with its scope explained promptly in the opening.

## Validation and handoff

Front matter, title and dek checked against 648c84e5; seven numbered sections, two figure wrappers, one table and no body bullet lists; British role spelling and common -ise words checked. All research-record links use immutable commits. No site build or final SVG rendering is claimed: those remain the Coordinator's requested port, redraw and site check. No code, research rules, implementation branch or merge was changed.
