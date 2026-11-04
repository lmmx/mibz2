[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-152/blog/index.md>)

# Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm

Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each other directly, without me acting as their message bus.

4 November 2026 · 11 min read · [Method](<#9-good-practice-in-one-list>)

The research behind [the last post](<https://lmmx.github.io/mibz2/pr-preview/pr-152/blog/2026-11-04-compact30-constructor/index.md>) was done by several AI agents. One implemented and tested each change to compact30's table builder. Others searched the machine code for instructions to remove, reviewed each result, and checked that the experiments followed their registered rules. Each agent could work alone for long stretches. Together, though, they formed a team only when I copied each agent's messages to the next. I was the team's message bus.

This post describes the team: why the work is split into four roles, and how they now hand it to each other through the repository without me relaying. Nothing in the setup is exotic: a shared repository, a small message format, and one event-driven trigger for each agent. Most of the effort went into the rules that make it safe for the agents to act on each other's messages without asking first.

## 1\. Four roles, defined by what they're responsible for

The team has four members. Each role is defined by its responsibilities, not by the model behind it. Three of the roles run on the same model, and that doesn't make them interchangeable.

| Role | Runs as | Responsible for | Never does |
| --- | --- | --- | --- |
| **Coordinator** | Claude Code | implementing, compiling and testing each change; archiving the evidence; posting checkpoints | change the scientific rules, or grant itself authority |
| **Optimiser** | Codex | finding instructions to remove, proving the transformations, keeping the canonical instruction ledger | implement code, or certify its own proposals |
| **Guardian** | Codex | applying the research charter's gates to evidence, resolving delegated decisions, escalating reserved ones | originate research, or become an approval gate |
| **Researcher** | Codex | independent, adversarial challenge, including the formal review before changes are combined | agree by default, or duplicate the Optimiser |

The split follows the questions each role asks. The Optimiser asks how to remove more instructions. The Guardian asks whether the process is scientifically defensible. The Researcher asks whether the team's reasoning is sound, and whether the Guardian is making the right call.

### Why this work needs four roles

Instruction-level optimisation is easy to fool yourself about, in a few specific ways, and each role answers one of them.

- **Savings are estimates until compiled.** The Optimiser's design for a change predicts which instructions disappear. The compiler then decides what's actually emitted, and often spills a register or recomputes an address somewhere the design didn't touch. So the agent that proposes isn't the one that builds and measures. The Coordinator compiles each change and counts every instruction the whole function executes over a real corpus.
- **Targets drift unless someone holds them.** Each experiment registers its gates before it runs: correctness first, then a minimum net saving. The Guardian applies them as written, which stops a near miss from being reinterpreted after the fact.
- **Accounting errors look like results.** Here the Researcher and the Optimiser checked each other's arithmetic, and both checked the Coordinator's. Two runs of a profiler had been presented as a bound, and they weren't one. A Miri test covered one unchecked access but not the other. Both were caught and corrected before anything was combined.

Keeping those questions in separate agents means a mistake made in one has to get past the others.

These identities are written down in the repository ([`ROLES.md`](<https://github.com/lmmx/mibz2/blob/research/compact30-constructor/docs/journal/compact30-channel/ROLES.md>)), so they survive a session restarting or being replaced.

## 2\. The repository is the team's memory

None of the agents keeps anything important in its own conversation. Everything that matters is a file in the repository, on a branch that only one agent writes to:

- **The charter** says what work is authorised, which gates each experiment must pass, and which decisions are reserved for me.
- **The operating specifications** say what each role does and how it reports.
- **Each agent's own state** lives on its own branch: a cursor of what it has already processed, and its ledger or decision log.
- **The evidence** is in the checkpoint journals, pinned to commits: source, compiled listings, counters and raw profiles.

Everything an agent relies on is cited by commit. A decision says which version of the charter it applied and which evidence it checked, so anyone can replay it later. One writer per branch means two agents never edit the same file.

This design is what makes the rest possible. An agent that starts with an empty conversation can read its specification, its cursor and the evidence, and carry on exactly where its last run stopped.

## 3\. Messages are PR comments with a small header

The agents talk to each other in comments on pull requests. Each comment begins with a short header:

```text
FROM: coordinator
TO: guardian
RE: prototype 2, hot descriptor pointer arrays @ 29dd071f
TYPE: CHECKPOINT
```

The `TYPE` says what kind of message it is:

| Type | Sent by | Means |
| --- | --- | --- |
| `CHECKPOINT` | Coordinator | a result is ready at a pinned commit; the addressed roles should act on it |
| `REVIEW` | Optimiser, Guardian, Researcher | an assessment, with its evidence published on the sender's branch |
| `PROPOSAL` | Optimiser | a new idea; it isn't authorisation to implement it |
| `ACK` | Coordinator | a message was received and logged, and what happens next |

`TO:` routes the message to particular roles, and `RE:` pins the commit it's about. One draft pull request serves as the team's channel. Every checkpoint is announced there, and each experiment and each agent also has a draft PR of its own.

The header is a label, not authentication. All the agents post through my GitHub account, so a line saying `FROM: guardian` proves nothing about who wrote it. The safeguards in §6 are built around that fact.

## 4\. Who wakes whom

An agent that has finished its turn is idle until something starts it again. The two kinds of agent here wake differently.

**The Coordinator** subscribes to pull requests. A new comment, a review or a CI result on a subscribed PR starts a turn in its existing session. When another agent replies, the Coordinator reads it within seconds.

**The Codex agents** needed a different mechanism. A Codex conversation keeps its objective between turns, but nothing on GitHub can start a turn in it. Before the change, each Codex agent waited until I typed a nudge, such as "new checkpoint on \#151". Messages from Codex to the Coordinator flowed on their own. Messages in the other direction depended on me.

Before: two directions, one of them through me

The gap was in one direction only. Each agent could already do its work; what was missing was a way for the Coordinator's message to start a Codex agent.

## 5\. The doorbell

ChatGPT Work can run a task when a GitHub event happens. Each Codex role now has one such task, subscribed to new comments on the repository's pull requests. A new comment rings the doorbell: it starts a short, fresh run of that role.

GitHub can't filter on a comment's text before a run starts. So every run begins with the same guard:

1. **Fetch the triggering comment by its ID.** The event supplies only the ID.
2. **Check that it's addressed to this role.** For the Optimiser, that means a `TYPE: CHECKPOINT` with its role in the `TO:` line. If not, stop at once, without doing anything else.
3. **Check the cursor.** If this comment's ID has already been processed, stop.
4. **Rebuild the state.** Read the role's specification, its cursor and its ledger or decision log from its own branch.
5. **Do the work.** Reproduce the figures it relies on, then assess, audit or research as the message asks.
6. **Publish.** Commit the result to its own branch, and post one `TYPE: REVIEW` comment that links to it.
7. **Save the cursor,** and stop.

After: one cycle, with no one relaying

Every link in the cycle is an event. No agent polls, and I pass no messages along. Replies are typed REVIEW, so they never re-trigger a CHECKPOINT run.

A run that rings the bell for another role's message costs one comment fetch and stops. Replies are typed `REVIEW`, never `CHECKPOINT`, so an agent's answer can't start another round of work by accident.

## 6\. Authority lives in the charter, not in the messages

When agents act on each other's messages without asking, it matters what a message is allowed to make them do. The rule the team follows is simple: **the repository can narrow authority, never widen it.**

- **Narrowing messages count, whoever wrote them.** A `HOLD`, a `STOP`, a finding, or a request to tighten a gate is acted on even if its author can't be confirmed. Obeying a false one only costs time.
- **Routine work needs no permission.** Work the charter already authorises goes ahead without anyone's go-ahead. The Guardian isn't an approval gate. It steps in when evidence is disputed, a safety argument fails or a rule needs interpreting.
- **Widening needs me.** New experiments, unsafe code, timing runs, changes to frozen code, merges and new infrastructure all count only when I say so in the Coordinator's own session. That session is the one channel whose author is known.
- **Only collaborators can comment.** The repository limits interactions to collaborators, so only the team's own agents can post in the channel.

Each agent's specification spells out what it may decide and what it must pass up. The Guardian, for example, can rule on whether a checkpoint met its gates, or name an independent reviewer, straight from the repository. It can't approve a new experiment.

## 7\. Independence is designed in

An agent that proposed an idea isn't independent when it comes to judging that idea's result. The team keeps three things separate:

- **The proposer.** The Optimiser flags any audit of its own proposal as not independent.
- **The governor.** The Guardian checks that the rules were followed. It doesn't check its own rulings.
- **The independent review before changes are combined.** It goes to a fresh run that's given only an index of pinned evidence: no discussion history, no proposals, and no earlier verdicts. It must disclose any earlier involvement it had.

The checkpoint journals record the implementer's own claimed results. Reviewers are asked to treat those as claims to falsify. They re-derive each figure from the source, the compiled listings, the counters and the raw profiles.

## 8\. The details that make it work

A few practical choices turned the idea into something that runs:

- **A guard comes before anything else.** Every run fetches its triggering comment and stops unless the comment is addressed to it, before reading anything else.
- **Cursors make runs idempotent.** An event delivered twice, or two events batched into one run, don't produce duplicate decisions.
- **One writer per branch.** Each agent owns its branch. The Coordinator owns the implementation branches.
- **Pinned evidence everywhere.** Each message names the commit it refers to, and each decision names the documents it applied, by blob.
- **A commit identity on every branch.** Every commit is under my own GitHub identity, with no AI attribution. Automated runs use the account's GitHub noreply address.
- **A model and an effort level for each role.** The table below has them.
- **No polling.** Nothing runs on a timer. Every run starts because something happened.

| Role | Runs on | Model ⠶ effort |
| --- | --- | --- |
| Coordinator | Claude Code session | Claude |
| Optimiser | ChatGPT Work event-triggered task | Astra ⠶ Ultra |
| Guardian | ChatGPT Work event-triggered task | Astra ⠶ Extra High |
| Researcher | ChatGPT Work event-triggered task | Astra ⠶ Extra High |

## 9\. Good practice in one list

1. **Define roles by responsibility,** and write them down where every agent can read them.
2. **Keep all state in the repository,** on branches with a single writer, cited by commit.
3. **Use a tiny, typed message format.** Treat its headers as routing, never as authentication.
4. **Wake agents with events, not timers.** A subscription for one agent, an event-triggered task for another: whatever each platform supports.
5. **Make each run small and idempotent:** guard, deduplicate, rebuild state, act, publish, record.
6. **Let messages narrow authority but never widen it.** Keep widening decisions with a person, in a channel whose author is known.
7. **Make the governor something other than an approval gate.** Authorised work proceeds without waiting for it.
8. **Separate proposing, governing and independent review,** and give the reviewer evidence, not history.
9. **Limit who can post,** so the channel carries only the team's own messages.

## 10\. What changes

I'm no longer the message bus. A checkpoint posted by the Coordinator reaches the Guardian and the Optimiser on its own, their reviews reach the Coordinator on their own, and the next step follows. My part is what it should be: setting the rules, and making the decisions the rules reserve.

The first cycles of this team are running now, on the compact30 table builder. Once there's a record to compare, this post will gain a short note on how its throughput compares with the earlier way of working.
