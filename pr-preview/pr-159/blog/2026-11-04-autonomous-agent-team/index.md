[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-159/blog/index.md>)

# Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm

Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each other directly, without me acting as their message bus.

4 November 2026 · 10 min read · [Meta](<#8-where-it-stands>)

[The last post](<https://lmmx.github.io/mibz2/pr-preview/pr-159/blog/2026-11-04-compact30-constructor/index.md>) was about compact30, my effort to give the decoder's navigation table a smaller representation. Four agents did the research behind it, but until today they could only work together if I copied each one's messages to the next by hand.

This post is about how they now pass work between themselves through the repository: who does what, how a message gets from one agent to another, and what I learnt from an afternoon of running it. It covers the setup and its first full cycle. Whether it actually makes the research faster, we'll have to wait and see.

## 1\. A tale of four agents

The Coordinator is Claude Code. It writes the code for each change in the plan, compiles it, runs the tests and keeps the results. When a result is ready it posts a checkpoint: a comment saying what was built and which commit holds the evidence.

The Guardian and the Researcher both read every checkpoint, looking for different things. The Guardian checks the result against the research charter: which gates it passes, what work can follow, and what has to come to me. The Researcher looks at the reasoning: whether the argument holds, whether something else explains the numbers, whether there's a better direction, and whether the Guardian has got it right.

The Optimiser works on the instruction counts themselves. It reads the source, the compiled listings and the execution counts, works out transformations that remove instructions, and keeps the ledger of what each one saves. Its proposals are what the Coordinator builds next.

The Guardian, the Researcher and the Optimiser all run Astra, a model I use through ChatGPT, each as a ChatGPT Work task with its own reasoning-effort setting. What separates them is what they're responsible for, not the model. The roles, and the branch each one owns, are written down in [ROLES.md](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md>). To reproduce the setup, these are the settings:

| Role | Runs as | Model | Reasoning effort | Started by |
| --- | --- | --- | --- | --- |
| Coordinator | Claude Code session | Claude, as configured for the session | High | a subscription to the PRs it follows |
| Guardian | ChatGPT Work task, *Compact30 (W) Guardian* | Astra | Extra High | any PR comment; acts only on a `CHECKPOINT` |
| Optimiser | ChatGPT Work task, *Compact30 (W) Optimiser* | Astra | Ultra | any PR comment; acts only on a `CHECKPOINT` addressed to it |
| Researcher | ChatGPT Work task, *Compact30 (W) Researcher* | Astra | Extra High | any PR comment; acts only on a message addressed to it |

The team: three Astra agents and Claude Code, joined by one repository

Claude Code builds each change and posts a checkpoint. The Guardian checks it against the rules and the Researcher checks the reasoning, while the Optimiser hunts for more instructions to cut. All four work through the same repository.

## 2\. Every result is a commit

A checkpoint points at a commit, never at a branch, because a branch keeps moving. The commit holds the source and its baseline, the compiled listings, the toolchain, the test records, the execution counts and the arithmetic that ties them together. Reports and scripts go in files, and the comment links to them.

That lets a reader check a claim rather than take it on trust. If the Optimiser predicts that a change will remove a load from the inner loop, the checkpoint shows whether the compiled code actually lost it, whether setup got more expensive, whether a caller picked up a spill, and how often each path runs.

Each agent writes only to its own branch: the Coordinator to the implementation branches, the others to their reports, decision logs and ledgers. Every file has one author, and a review never edits the code it reviews. Each agent also keeps its working state on its branch, which is why a fresh run can pick up exactly where the last one stopped.

## 3\. Comments as messages, events as the clock

The agents talk in comments on pull requests. Each comment starts with a short header:

```
FROM: coordinator
TO: guardian, researcher
RE: prototype 2 @ 29dd071f8787a28560a7a081fd7ac0ef13991466
TYPE: CHECKPOINT
```

`TO:` says who should act, `RE:` pins the commit, and `TYPE:` says what kind of message it is. The [protocol](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md>) has six types:

| Type | Means |
| --- | --- |
| `CHECKPOINT` | a result at a pinned commit, ready to assess |
| `REVIEW` | an assessment, with its evidence on the sender's branch |
| `PROPOSAL` | an idea to consider; not permission to build it |
| `DECISION` | a decision the sender is allowed to make |
| `ACK` | what was received, and what happens next |
| `ESCALATION` | something only I can decide |

Most of the traffic goes through one draft PR, \#145, which serves as the team's channel. Each experiment and each agent also has a PR of its own.

Here is what that looks like on GitHub. Every comment comes from my account, `lmmx`, and the small badge on the avatar shows which app posted it: Claude's mark for the Coordinator, ChatGPT's for the Astra agents. The header is the only thing that says which agent wrote it.

Three agents, one account: a checkpoint from the Coordinator, the Guardian's review of it, and the independent reviewer's report, all on \#145.

**Read the whole exchange**

The originals: [the checkpoint](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394>), [the Guardian's review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082>) and [the independent review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6634356048>).

A new comment is also what wakes the agents up. The Coordinator subscribes to the PRs it follows, so a reply reaches it within seconds. The three Astra agents are Work tasks that start on any PR comment in the repository. The event only includes the comment's ID, so each run first fetches the comment and checks whether it's meant for it. The Guardian acts only on a `CHECKPOINT`, the Optimiser only on a `CHECKPOINT` addressed to it, and the Researcher on anything addressed to it. Anything else, including the other agents' reviews, ends the run straight away. A checkpoint that both readers should see therefore has to name them both.

One cycle: a checkpoint, two readers, the next step

Each arrow is a pull-request comment, and each comment starts the next agent's run. Nobody polls, and nothing goes through me.

## 4\. A receipt for every message

Before doing any work, a run checks its cursor: a file on its branch listing the comments it has already handled. Then it loads what it needs, does the work, posts its result, and adds the comment to the cursor. If several comments arrive at once, each is checked separately. For technical reviews the key also includes the PR, the candidate's commit and a hash of the evidence, so a repeated request can't be mistaken for a revised candidate.

The cursor answers "have I already done this?" for the next run, and it lets me trace every message to what came of it. Two cases it doesn't cover on its own: a run that posts its result but stops before saving the cursor, and two runs of the same agent at once. Recording the result's commit and comment ID makes the first recoverable. I haven't tested either case yet.

A review queue in the repository lists what is waiting on whom. Work that depends on a pending review waits for it, and everything else goes ahead.

## 5\. What a message can authorise

The [research charter](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md>) sets out what is authorised: the representation, the reference builder, the baselines, and for each prototype a registered design, acceptance criteria and a fixed number of attempts. Inside that, the Coordinator implements and tests without asking me, and the Guardian makes the decisions delegated to it. Anything that widens the work, such as a new prototype, new unsafe code or integration into production, needs me.

So the rule is: **a message in the repository can narrow what's allowed, but only I can widen it, and only in the Coordinator's session.** A finding can put a prototype on hold. A proposal can suggest something new, but nobody acts on it unless it's already authorised or I say so. Charter amendments are approved the same way, and every review cites the charter revision it applied.

This matters because every agent posts through my GitHub account, as the screenshots in section 3 show, so `FROM: guardian` names a role without proving who wrote it. The repository also limits who can comment, and the channel PRs are locked. Before the Guardian went live it was tested on exactly this: it was sent forged requests for things only I can approve, and turned both of them down.

## 6\. Reproducible, independent review

The main figure for compact30 counts the instructions executed by the whole table builder and everything it calls, weighted by how often each path runs on the corpus. Setup, fallbacks, callers, callees and extra memory operations all count, and a saving that only moves work elsewhere is charged where the work lands. Correctness needs its own evidence: byte-identical output against the reference builder, safety arguments for release builds, Miri checks for undefined behaviour, and mutation tests that break an invariant on purpose to show the tests notice.

Reviewers treat the implementer's write-up as a set of claims to check, and work the figures out again from the source, listings, counters and test records. Before prototypes are combined, the charter also asks for one review by an agent that has seen none of the discussion. The ongoing Researcher can't do that: it has followed the work, and it suggested some of the tests. So it started a fresh helper whose only input was a pinned brief, and recorded exactly what that helper was given. A fresh run on its own wouldn't have been enough: the point is that the reviewer never saw the discussion.

## 7\. What running it looks like

The first afternoon taught me what to expect. Mostly it comes down to patience, and knowing where to look.

Runs are long and quiet. A Work task can spend anything from a minute to a quarter of an hour on one message: the Researcher's revision of this post took about sixteen minutes, most of it reading the record and testing drafts on simulated readers. Between runs, a task's page just shows that it has worked. That means it is waiting for its next comment, not that it has stalled.

Messages queue behind the work in hand. A comment that arrives while an agent is busy is handled once it finishes, and several can be handled together: one of the Researcher's runs did the blog revision first, then launched the independent reviewer, picking up the Guardian's approval of that launch at the same time. Re-sending a message because a page looks idle only adds a duplicate, which the agent's cursor then ignores.

Watch the repository, not the task pages. Every message ends in a receipt: a `REVIEW` that cites it, or a cursor commit on the agent's own branch. A message without one is queued or in progress. It's only lost if the agent's branch shows it has moved on without a receipt, and then I send it again, once.

When an agent starts a helper of its own, I ask it to keep its run open until the helper has published, and to record both the launch and the result. The independent reviewer was set up this way, with its settings and inputs written to the repository before it began.

Keeping things moving is the Coordinator's job. When the record goes quiet, it checks the receipts, works out who has the next step, and posts the checkpoint that starts it. There's no timer involved, and nothing sits waiting for me.

## 8\. Where it stands

The [channel log](<https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md>) records each step. The trigger was first tested on the Guardian: a comment started its run, it published an assessment, and its review woke the Coordinator. The first real cycle followed on [a checkpoint](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394>) covering two prototypes. The Guardian's run started nine seconds after it was posted, checked both prototypes against the gates, and [asked](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082>) for the independent review.

That review came back while this post was being finished. The Researcher launched the isolated reviewer, and its [report](<https://github.com/lmmx/mibz2/pull/153>) arrived as a comment on the channel. Both prototypes pass on correctness and safety, and so does v3's finaliser. Two things need fixing before anything is combined: some changed paths still need their own execution counters, and both prototypes remove the same instruction, so their savings overlap. That is the next checkpoint.

So far, then, the messages get through and the reviews get done, without me relaying anything. Whether that makes the research faster, and whether the constructor gets fast enough, will show over the next few posts.

If you want to set up something similar: give each responsibility its own output, point every message at an exact commit, wake agents with events rather than timers, keep a receipt for every message, write down what a message is allowed to authorise, and decide what an independent reviewer gets to see. My own part is choosing the scope and making the decisions the charter keeps for me.

---

**Research records:** [Role identities](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md>) · [Research charter](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md>) · [Channel protocol](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md>) · [Review contract](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md>) · [Recorded exchanges](<https://github.com/lmmx/mibz2/blob/688d4f7c4cdd813d13ed2b06708e47cbfdce2ca4/docs/journal/compact30-channel/log.md>).
