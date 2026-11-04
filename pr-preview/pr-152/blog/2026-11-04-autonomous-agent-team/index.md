[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-152/blog/index.md>)

# Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm

Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each other directly, without me acting as their message bus.

4 November 2026 · 11 min read · [Meta](<#8-what-this-establishes>)

Can I let a team of agents carry research from one checkpoint to the next, while keeping the evidence inspectable and the scope of the work under my control? In mibz2, my bzip2 decoder, I am using a shared repository to do that. The working claim is quite specific: agents can hand routine research to one another when each handoff names the work, the evidence and the rules for acting on it. The recorded exchanges show that this handoff works; its effect on research throughput remains to be measured.

The research itself follows [the compact30 constructor](<https://lmmx.github.io/mibz2/pr-preview/pr-152/blog/2026-11-04-compact30-constructor/index.md>). Compact30 stores the navigation table used during decoding in a smaller form. Building that table involves several kinds of work: finding instructions to remove, expressing a change in Rust, inspecting what the compiler emits, and checking that the resulting records are still correct. A local saving also has to survive the accounting for the whole builder.

I have given those responsibilities to four agents. They share one way of working: read a pinned result, do an assigned piece of work, and publish the next result through GitHub. Their responsibilities differ; the shape of the handoff stays the same. The sections below follow it from the team, through a checkpoint and its delivery, to the evidence that lets the next step proceed.

## 1\. One workflow, four responsibilities

The Coordinator runs in Claude Code. It implements the registered changes, compiles them, runs the tests and archives the evidence. When a result is ready, it publishes a checkpoint: a short message saying what has been built and where the evidence lives.

The Guardian and the Researcher are the paired readers of that work. They read the same checkpoint with different questions in mind. The Guardian asks what follows under the agreed research rules: which gates the evidence meets, which work can continue, and which decisions are reserved for me. The Researcher asks whether the reasoning is sound, whether another explanation fits, and whether the team is pursuing the most useful direction. That includes questioning the Guardian's interpretation when the evidence warrants it.

Meanwhile, the Optimiser studies the instruction-reduction problem itself. It works from the source, compiled listings and execution counts, develops transformations and keeps the canonical ledger of their savings. Its proposals give the Coordinator concrete ideas to implement within the authorised research programme.

All four publish through the repository. A proposal, a checkpoint and a review each identify something the next reader can inspect. The roles are written down in [the identity record](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md>), along with the branches they own. The Guardian, the Researcher and the Optimiser all run Astra, a model I use through ChatGPT, each as a ChatGPT Work task at a fixed reasoning-effort setting (the badges in the figure below). Sharing a model doesn't make them interchangeable; their different responsibilities are what distinguish them.

To reproduce the setup, these are the exact settings:

| Role | Runs as | Model | Reasoning effort | Started by |
| --- | --- | --- | --- | --- |
| Coordinator | Claude Code session | Claude, as configured for the session | High | a subscription to the PRs it follows |
| Guardian | ChatGPT Work task, *Compact30 (W) Guardian* | Astra | Extra High | any PR comment; acts only on a `CHECKPOINT` |
| Optimiser | ChatGPT Work task, *Compact30 (W) Optimiser* | Astra | Ultra | any PR comment; acts only on a `CHECKPOINT` addressed to it |
| Researcher | ChatGPT Work task, *Compact30 (W) Researcher* | Astra | Extra High | any PR comment; acts only on a message addressed to it |

For another project, I would begin with those responsibilities and their outputs. Who makes the change? Who checks its evidence against the agreed criteria? Who questions the argument? Who develops the next idea? Giving each question an owner makes the handoffs concrete.

The team: three Astra agents and Claude Code, joined by one repository

The Coordinator implements and publishes checkpoints. The Guardian and the Researcher examine process and reasoning; the Optimiser develops the instruction-reduction work. Each role reads and writes through the same repository.

## 2\. Give the next reader an exact result

A checkpoint is a package of evidence with an address. For a constructor change, that package includes the source and its baseline, the compiled listings, the toolchain, test records, execution counts and the accounting that connects them.

The address is a commit. A branch is where work continues; the commit identifies the particular version being reviewed. A review cites that version and the governing documents it applied. Long reports and scripts live in committed files, with a short comment linking to them.

Suppose the Optimiser predicts that changing a descriptor will remove a load from the inner loop. The Coordinator's checkpoint lets another agent ask what actually happened after compilation. Was the load removed? Did setup acquire more work? Did a changed caller add a spill? How often does each path execute? The compiled code and counters let the reader follow the claim beyond the sentence that announces it.

Each role owns its publication branch. The Coordinator owns the implementation branches; the other agents own their reports, decisions and ledgers. That gives each result a clear writer and keeps the Coordinator's source changes separate from a review of them.

The same arrangement makes a resumed task useful. Its important working state is explicit in the repository: what it has reviewed, which evidence it used and what remains pending. Conversation history can help an investigation, while the committed record lets another reader inspect it without having taken part.

## 3\. Let the repository ring the doorbell

A comment carries the handoff; an event starts the next reader. The messages use a small header. For a checkpoint that needs both process and reasoning reviews, I can address both readers:

```
FROM: coordinator
TO: guardian, researcher
RE: prototype 2 @ 29dd071f8787a28560a7a081fd7ac0ef13991466
TYPE: CHECKPOINT
```

The body says what needs attention and links the evidence. One standing draft PR carries messages across the project, and individual experiments and reviewers have their own PRs. The shared channel is \#145: a checkpoint there points to an experiment's evidence, and a short review links back to the report on the reviewer's branch. The PRs give each piece a home; the comment and its evidence pin connect the pieces. The [protocol](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md>) gives the main message types a small, practical vocabulary:

| Type | What it carries |
| --- | --- |
| `CHECKPOINT` | A pinned result ready for assessment |
| `REVIEW` | An assessment and its evidence |
| `PROPOSAL` | An idea for consideration |
| `DECISION` | A decision within the sender's delegated remit |
| `ACK` | What was understood and what follows |
| `ESCALATION` | A matter requiring a reserved decision |

The Coordinator subscribes to the relevant PRs. The three Astra roles run as ChatGPT Work tasks triggered by PR comment events in this repository. Those are two delivery mechanisms for the same workflow: a result appears, a recipient reads it, and the recipient publishes its response.

In the events observed here, the payload supplies the comment's ID rather than its text. A task first fetches that exact comment, then applies its configured filter. The Researcher's filter looks for its role in the address line. Other tasks also filter by message type. To obtain paired reviews, the checkpoint must reach both configured readers; the role diagram alone does not establish that coverage.

A comment can wake a task without giving it any work. A review is itself another event, so a checkpoint-only task checks its type and stops. A task with an address-based filter checks whom the message names. The useful boundary is the filter at the start of the run: it decides whether this particular message belongs to the role.

One cycle: a checkpoint, two readers, the next step

A repository event starts a reader. The reader checks the message and its saved state, does the work assigned to it, then publishes a result and records the receipt. Each arrow is a pull-request comment; no agent polls, and no message passes through me.

## 4\. Save a receipt for the work

Once the message matches, the task checks its cursor: a file recording the work already processed. A comment ID identifies the notification. The review contract also keys technical work by PR, candidate commit and evidence hash, so a repeated request can be distinguished from a revised candidate.

The task then loads the state and evidence it needs, does the work, publishes its result and records the receipt. If several comments arrive together, each gets its own check. One already processed message leaves the others available for assessment.

This is a small amount of state with an important purpose. The next execution has a durable answer to “have I handled this?” I can also follow the receipt to the published result and see what was concluded.

Publication and recording the receipt are separate operations. For another team adopting this pattern, I would make recovery at that boundary explicit: if a result was published but the cursor was not saved, locate the published result before repeating the write. Keeping its commit and outgoing comment ID makes that check possible. The cursor supports deduplication; exactly-once publication also needs that interruption case to be handled.

Pending work is explicit in the same way. A review stays in the queue until a pinned assessment appears. Work that depends on it waits for that result, while other authorised work can continue. The queue gives the team a shared account of progress.

Branch ownership assigns responsibility to one role. Concurrent executions of that role also need serialisation or a way to reconcile competing writes. I treat that as part of the execution design, alongside recovery, rather than a guarantee supplied by the cursor. The recorded exchanges demonstrate delivery; interruption recovery and overlapping executions need their own validation before I can claim those guarantees.

## 5\. Decide what a message may authorise

For these handoffs to carry useful work, the recipients need to know what they may do. I put that in the [research charter](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md>). It fixes the representation, the reference builder and the baseline versions. Each prototype has a registered design, acceptance criteria and a bounded set of attempts.

Within that scope, the Coordinator can implement and test without asking me at every step. The Guardian can apply the agreed rules and make the decisions delegated to it. I keep the decisions that expand the work, such as adding a prototype, approving a new unsafe operation or integrating into production.

The rule for messages follows from that split: **the repository can narrow authority; widening comes from me in the Coordinator's session.** A supported finding can hold a component's qualification. A proposal can suggest a new direction. Carrying the proposal out depends on the authority already granted or on my explicit decision. I approve the charter and its amendments there; the Coordinator records the decision's provenance, and reviews cite the governing revision they applied.

Under the protocol, a review that holds qualification returns to the Coordinator on a subscribed PR. It logs the finding and keeps the dependent work pending until the required resolution. The checkpoint filter of another role is not the route for that decision.

Every agent uses my GitHub account. The sender label therefore names a role rather than authenticating it. Recipients check what the message requests against their standing instructions. This is a rule for their behaviour, distinct from the permissions enforced by the tools. The protocol also records time-limited interaction restrictions and locks on named PRs. Those controls limit participation; the charter defines what a participant's message can authorise.

The channel's test cases make that boundary inspectable. They include a finding, a hold, an acknowledgement and requests for actions reserved to me. Their value is in checking both halves of the exchange: the recipient understood the message, and its response stayed within the permitted next step.

## 6\. Make the review reproducible, and its independence explicit

Compact30's qualification figure counts machine work across the whole constructor and everything it calls. Compiled path lengths are weighted by execution counts on the corpus. Setup, fallback paths, callers, callees and added memory operations all belong in the account. A saving moved from one place to another must be counted where it lands.

Correctness has its own evidence: byte-exact output against the reference builder, release-mode safety arguments, targeted Miri checks for undefined behaviour, and mutation tests that deliberately break an invariant to show that a test detects it. These requirements give the Guardian and Researcher specific claims to examine.

A report states what was inspected and what was reproduced. An implementer's journal contains the implementer's conclusions; the reviewer works from the source, listings, counters and test records to derive its own. That lets another reader replay the important parts of the argument.

Before components are combined, the charter asks for an independent review from a pinned evidence index. This has a different input boundary from an ongoing role. The continuing Researcher knows the team's discussion and may have contributed ideas. The independent reviewer receives the permitted evidence and governing specifications, with earlier involvement disclosed.

A new execution and an independent context are separate properties. To make a review blind, I need to control which history is supplied to it as well as which files it reads. The required review context excludes earlier discussion and verdicts. Its findings then have to be resolved before the components can be combined.

For a team working on another problem, this is a useful way to specify independence: write down the reviewer's inputs, its earlier involvement and what it reproduced. A separate role provides a separate responsibility; a recorded evidence boundary explains what makes a particular assessment independent.

## 7\. What running it looks like

The first afternoon of running the team taught me what to expect, and most of it is about patience and where to look.

A run is long and quiet. A Work task can spend anything from a minute to a quarter of an hour on one message: the Researcher's revision of this post took about sixteen minutes, most of it reading the record and testing the draft on simulated readers. Between runs, a task's page simply shows that it has worked. That is a role at rest, waiting for its next comment, not a role that has stalled.

Messages queue behind the work in hand. A comment that arrives while a role is busy is handled once the current work is done, and several can be handled together: one of the Researcher's runs took the blog revision first, then a request to launch the independent reviewer along with the Guardian's decision approving it. Re-sending a message because the page looks idle only adds a duplicate, which the role's cursor then discards.

The repository is the place to watch, not the task pages. Every message ends in a receipt: a `REVIEW` that cites it, or a cursor commit on the role's own branch that records it. A message without a receipt is either queued or in progress. Only when the role's branch shows it has moved past that point without one is the message lost, and then I send it again, once.

When a role hands work to a helper of its own, I ask it to keep its run open until the helper has published, and to record both the launch and the result. The Researcher's independent reviewer was set up this way: an isolated child given only the pinned brief, with its launch settings and inputs written to the repository before it began.

Keeping the process moving is the Coordinator's job. When the record goes quiet, it reads the receipts, works out who holds the next step, and posts the checkpoint that starts it. Nothing waits on a timer, and nothing waits on me.

## 8\. What this establishes

The [channel log](<https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md>) records a completed event-delivery loop. A GitHub comment started the Guardian task; the Guardian fetched it, published an assessment and saved its cursor; the review reached the Coordinator. That setup test included confirmation of the commit identity. The log also records the subsequent D008 assessment of real pinned checkpoints and the Coordinator's handoff of an evidence index.

That real exchange gives the abstract handoff an address. The Guardian's review on \#145 identified the descriptor candidate at commit 29dd071f and linked its assessment at 62a313a8. The Coordinator recorded it in the channel log and supplied the independent review index at 18ca5d4c. The next step was to obtain that independent assessment; the checkpoint review alone did not complete the pre-integration requirement.

The independent assessment then arrived the same way, while this post was being finished. The Researcher launched an isolated reviewer with the pinned brief as its only input, and its [review](<https://github.com/lmmx/mibz2/pull/153>) came back as a comment on the channel. It passes both prototypes on correctness and safety, and v3's finaliser on its safety evidence. It also holds two things for the Coordinator to resolve before anything is combined: several changed paths still need dedicated execution counters, and the two prototypes both remove one shared instruction, so their savings overlap. That is the next checkpoint.

That establishes delivery and review at the scale recorded. The effect of the team's arrangement on research throughput remains to be measured, and the constructor's performance will be established by its own experiments.

The method a reader can transfer is already concrete. Give each responsibility an owned output. Make a handoff name an exact result. Use events to bring that result to the next reader, and save enough state to recognise completed work. Define the authority that travels with a message and the evidence required for a conclusion.

My part is to choose the scope and make the decisions reserved to me. The agents can carry routine work between those decisions through the repository. A reader arriving later can follow the same route from the question, through the evidence, to the result that justified the next step.

---

**Research records:** [Role identities](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md>) · [Research charter](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md>) · [Channel protocol](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md>) · [Review contract](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md>) · [Recorded exchanges](<https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md>).
