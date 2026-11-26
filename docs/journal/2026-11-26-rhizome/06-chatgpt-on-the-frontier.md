# 6. ChatGPT's review of the frontier (#945), verbatim

*26 November 2026. Relayed by Louis and recorded unedited.*
- **Its figures** were rendered diagrams, so their text came through flattened in the paste: the race diagram in §1, the immune-system diagram in §8, and the two-direction card in §4.
- **A lost symbol in §6:** the branching symbol in "Let … denote" was lost in the paste. It is b, the number of new claims per completed claim.
- **What it changed:** `frontier/` was hardened in response, with atomic holds, the invariants, flags and exploration share (delta `2026-11-26-frontier-hardening.md`).

---

Worked for 21s

I've read [PR #945](https://github.com/lmmx/mibz2/pull/945?utm_source=chatgpt.com), including the new frontier protocol and its Python implementation. It has already been merged.

I think the direction is promising, but there are some quite concrete failure modes that deserve attention immediately.

The principle I would adopt is: decentralise discovery, preserve the integrity of shared state, and make consequential actions independently verifiable.

Removing the coordinator should remove the communication bottleneck and the dependence on one agent's accumulated context. It should not remove the useful functions that coordination previously performed.

The challenge is to redistribute those functions into the environment in which the agents operate, rather than inadvertently recreating a coordinator through a growing collection of administrative rules.

### 1\. There are some immediate technical vulnerabilities

These are more than speculative concerns. They are visible in the implementation introduced by #945.

| Vulnerability     | What could go wrong                                                                  | Remedy                                                                                       |
| ----------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| Non-atomic claims | Two agents claim the same task from different checkouts.                             | Acquire claims against a canonical shared ref and confirm acquisition before starting work.  |
| Premature release | A claim becomes available while its implementation PR is still awaiting integration. | Distinguish work completed from work integrated.                                             |
| Expiring leases   | A slow agent continues working after another agent takes over its claim.             | Unique session identifiers, lease renewal and ownership checks before consequential actions. |
| Stale frontier    | Agents pursue opportunities invalidated by another agent's findings.                 | Pin claims to evidence and revalidate their premises before expensive work.                  |
| Overlapping work  | Different claims modify the same underlying mechanism.                               | Track semantic dependencies alongside file ownership.                                        |
| Diluted review    | Agents effectively approve their own assumptions through subsequent sessions.        | Independent, evidence-first review of consequential changes.                                 |

### The locking mechanism particularly concerns me

The [frontier protocol](https://github.com/lmmx/mibz2/blob/23a23cb842766d7881769fa91dc349494b5d20bd/frontier/README.md?utm_source=chatgpt.com) says:

> Commit and push that change alone, first, so others see the claim; that is the lock.

But the [actual implementation of `frontier.py`](https://github.com/lmmx/mibz2/blob/23a23cb842766d7881769fa91dc349494b5d20bd/frontier/tools/frontier.py?utm_source=chatgpt.com) only modifies the local TOML file. It does not itself acquire a remote lock.

For example:

Shared state: claim X is open

Agent A

Reads X as open

Claims X locally

Pushes branch A

Agent B

Reads X as open

Claims X locally

Pushes branch B

Duplicate ownership

Both agents believe they acquired the lock.

Separate successful branch pushes do not establish exclusive ownership.

This is precisely the sort of concurrency problem we've been studying in the decoder itself!

I would also note that `claim` does not enforce `needs_louis` or check for overlapping ownership when acquiring a claim, while `release` does not verify that the caller owns the lease. `brief` filters some unsuitable claims, and `check` detects some inconsistencies retrospectively, but those are not equivalent to enforcing the conditions at the transition itself.

These are relatively small implementation problems worth fixing before launching large swarms.

### 2\. The rhizome needs conservation laws

There is a rather beautiful connection to our hybrid dynamical systems discussion here.

The PR already describes the frontier itself as a hybrid system. I would take that idea more seriously.

Agents make progress between events. Claims, releases, discoveries and merges constitute discrete transitions. Those transitions change the conditions under which subsequent agents operate.

But a well-designed system also has invariants: properties that must remain true regardless of the particular trajectory it follows.

For the swarm, I would establish five.

| Invariant                 | Meaning                                                                                                     |
| ------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Exclusive ownership       | No two agents simultaneously hold exclusive authority over the same consequential work.                     |
| Conservation of evidence  | No negative result, failed experiment or material objection silently disappears.                            |
| Conservation of authority | Agents cannot expand their own permissions through changes to the repository's instructions.                |
| Bounded expenditure       | The swarm cannot recursively generate unbounded work, spending or workstation requests.                     |
| Traceability              | Every consequential decision can be traced to the evidence, assumptions and policy under which it was made. |

These are not instructions for what the agents should discover. They are constraints on the permissible evolution of the collective.

The agents should be free to explore within those constraints.

### 3\. Avoid intellectual monoculture

This is perhaps my largest concern on the research side.

A swarm of heterogeneous agents sounds like it should produce heterogeneous thinking. But if every agent starts by reading the same surface, the same interpretation of the evidence and the same prioritised claims, they may all converge on the same conceptual assumptions.

The frontier itself could become a mechanism of overcoding.

Decentralised execution can still produce centralised thinking.

Three models agreeing after reading the same interpretation is not equivalent to three independent investigations reaching the same conclusion.

I would deliberately preserve some intellectual independence:

* Cold investigations: agents examine primary evidence before reading previous interpretations.

* Competing hypotheses: allow several explanations of the same observed phenomenon to coexist.

* Counterexample searches: agents explicitly try to falsify the assumptions behind promising research directions.

* Unallocated exploration: reserve perhaps 15–20% of research capacity for opportunities not already identified on the frontier.

That last percentage is a suggested starting point, not an empirically established optimum.

The surface should be a map of what is currently known, not a boundary around what is conceivable.

### 4\. Prevent the swarm from optimising itself into mediocrity

The instruction to take the highest-priority available claim is convenient, but it creates an implicit greedy scheduling algorithm.

That is potentially at odds with the reason this project adopted a rhizomatic approach in the first place: the discovery of outsized opportunities that were not apparent from the existing decomposition.

Consider two potential research directions:

Direction A

### +3%

Well understood, easily measured, relatively predictable.

Direction B

### Unknown

A possible structural breakthrough, with a high probability of failure.

A system that always takes the most immediately justifiable claim may systematically favour A.

The original transition to interval-exchange induction is exactly the kind of mathematical reorganisation that could be difficult to rank using an existing performance model.

I would therefore distinguish four kinds of research value:

| Kind             | Purpose                                                    |
| ---------------- | ---------------------------------------------------------- |
| Exploitation     | Realise a known performance opportunity.                   |
| Exploration      | Discover opportunities not yet represented.                |
| Characterisation | Reduce uncertainty about the system.                       |
| Synthesis        | Combine discoveries into a new mechanism or understanding. |

You need all four. In particular, synthesis should remain an explicit activity even though the synthesiser is no longer a permanent role.

### 5\. Replace the coordinator's integrative function, not just its routing function

The coordinator was expensive partly because it mediated communication. But it also maintained a sense of the whole research endeavour.

The new 40-line deltas are excellent for local continuity. They do not automatically provide global understanding.

Imagine twenty agents each producing a perfectly intelligible delta. Each delta may be individually correct, yet nobody notices the mathematical relationship between the findings of agents 4, 11 and 17.

This could be a substantial loss.

I would add a lightweight recurring type of claim: `synthesise`.

Its task is not to assign work or supervise anybody. Its task is to examine recent discoveries and identify relationships between them.

It might reveal that two supposedly separate bottlenecks share the same underlying cause, that a failed algorithm becomes viable when combined with another discovery, or that several lines of work are approaching the same mathematical object independently.

This is a distributed cognitive function, not a managerial position.

### 6\. Control the reproduction of work

The `swarm` instruction permits sessions to launch further sessions, and discoveries can generate additional claims.

That introduces the possibility of a self-amplifying process.

Let denote the average number of new claims generated per completed claim.

If every agent completes one claim but creates new ones, then, in a simplified model, the expected change in outstanding work after completions is:

\\mathbb E\[Q_n-Q_0\]=n(b-1).

If persistently, the backlog grows.

This is not inherently bad during a particularly fertile discovery phase. But an ever-growing backlog can eventually become a graveyard of unexamined ideas.

I would allow free creation of ideas while imposing a separate admission budget for active work. Not every discovery needs to become an executable claim immediately.

The workstation, in particular, should be treated as an explicitly shared scarce resource, with mutually exclusive run ownership and protection against duplicate submissions.

### 7\. Guard against agents gradually rewriting their own constitution

PR #945 places considerable reliance on repository files as the shared source of instructions.

That is useful, but it also creates a distinction we need to preserve:

Shared knowledge is not the same thing as shared authority.

A research document can inform an agent's decision. It cannot authorise a new class of actions merely because another agent wrote it.

The existing [RULES.md](https://github.com/lmmx/mibz2/blob/23a23cb842766d7881769fa91dc349494b5d20bd/docs/team/RULES.md?utm_source=chatgpt.com) already recognises this in A1 and A4.

However, there are still references to the retired coordinator throughout that document, including in the authorisation and safety-objection procedures. These need a careful migration by meaning, rather than merely a search-and-replace of role names.

I would protect the authoritative rules, workflow permissions, merge gates, release machinery and workstation controls from ordinary smooth-zone self-merges.

An agent can propose a constitutional change, but it cannot grant itself the authority to adopt that change. Equally, external literature, GitHub comments and downloaded documents must remain evidence rather than executable instructions.

### 8\. Develop a collective immune system

The immune system metaphor may actually be more appropriate here than introducing another layer of management.

Any agent should be able to detect and flag a material violation of an invariant. That flag should attach to the affected work, not freeze the entire collective.

Work continues

Isolated problem

The flag must specify the violated invariant, the evidence, the affected scope and what would resolve it. An implementer should not be able to dismiss its own unresolved material safety objection.

This preserves the useful function of the former Guardian without requiring a permanent Guardian.

The system should be capable of local correction while remaining productive elsewhere.

### 9\. Measure whether the rhizome is actually better

I was very pleased to see that the PR already includes a [three-arm experiment](https://github.com/lmmx/mibz2/blob/23a23cb842766d7881769fa91dc349494b5d20bd/frontier/claims/three-arm-experiment.toml?utm_source=chatgpt.com) comparing the orchestrated workflow, a restarted solo agent and the flat frontier.

I would extend the evaluation to cover undesirable side effects explicitly.

| Dimension              | Measurement                                                    |
| ---------------------- | -------------------------------------------------------------- |
| Research productivity  | Validated discoveries and performance gains per million tokens |
| Duplication            | Independent effort accidentally spent on the same work         |
| Intellectual diversity | Genuinely distinct mechanisms investigated                     |
| Experimental integrity | Violations of frozen decision rules                            |
| Integration            | Time between a discovery and its useful adoption               |
| Fragmentation          | Completed work that is never connected to other findings       |
| Human overhead         | Your time spent understanding, correcting or unblocking agents |
| Reliability            | Regressions, invalidated findings and conflicting changes      |
| Governance overhead    | Tokens and time spent maintaining the frontier itself          |

The PR's existing test that you can understand a day's work in a few minutes is particularly good. I would retain it.

I would be wary, though, of using the proportion of governance commits as the primary measure of administrative overhead. Commit counts are easily distorted by batching and say little about actual cognitive cost.

Also, the proposed three-arm experiment estimates roughly 20 million tokens and three workstation runs. Before that, I would run a small, cheap stress test of the new protocol itself.

### 10\. Deliberately try to break it

Before trusting the swarm with substantial work, simulate a few adverse situations.

| Scenario                                                      | Desired behaviour                                                          |
| ------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Two agents simultaneously claim the same task                 | Only one acquires exclusive ownership.                                     |
| An agent dies halfway through                                 | Another can recover its work without duplicating consequential actions.    |
| A lease expires while an agent is still active                | The original agent cannot release or integrate work under stale authority. |
| A new result invalidates three existing claims                | Their premises are reconsidered before more resources are spent.           |
| A persuasive but incorrect mathematical argument is committed | Independent investigation can contradict it without deference.             |
| A branch modifies an experimental gate after observing data   | The original verdict is preserved.                                         |
| Two sessions attempt to trigger a workstation run             | At most one authorised run is triggered.                                   |
| An agent proposes loosening a safety rule                     | The proposal cannot authorise itself.                                      |

These could become regression tests for the frontier tooling where machine-checkable, and small adversarial exercises elsewhere.

There is no need to recreate the old coordinator to perform them.

### What I would send back to the agents

I would give them the following as a design brief for a small follow-up to #945:

> Preserve the decentralised frontier approach. Do not recreate the coordinator, permanent supervisory roles, or mandatory communication chains.Instead, identify the useful invariants previously maintained by coordination and implement them as lightweight properties of the shared environment.In particular, make claim acquisition genuinely exclusive, prevent stale lease holders from performing consequential actions, distinguish completion from integration, preserve experimental evidence and authority boundaries, and ensure that independent lines of inquiry can challenge the frontier's prevailing interpretation.Preserve space for exploratory and synthetic research that is not immediately justified by the existing performance surface.Treat this mode of working as an experiment in its own right. Establish its failure conditions before substantially scaling it, and favour executable safeguards over increasingly elaborate prose instructions.The objective is not to maximise agent autonomy as an end in itself. It is to enable a collective to discover, understand and realise possibilities that its members would have been less likely to discover independently or under centralised direction.

I have reviewed the merged snapshot of #945 and its principal files, but have not executed the tooling, verified live branch protections or modified the repository. These recommendations distinguish observed implementation gaps from prospective organisational risks.

My strongest recommendation is to harden the shared-state transitions before expanding the swarm, without adding unnecessary restrictions on intellectual exploration.

There is something particularly fitting about applying our hybrid-system perspective here. We are now designing a system of interacting agents whose organisation determines the value of their collective work.

The aim should be to make that system structurally robust, rather than dependent on every individual agent remembering to behave correctly.
