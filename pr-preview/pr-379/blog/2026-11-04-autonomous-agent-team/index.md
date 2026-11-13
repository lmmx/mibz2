[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/index.md>)

# Call on mibz2: cutting instructions from bzip2 decoding with an autonomous swarm

Each of four agents does one part of the research. A GitHub doorbell lets them hand work to each other directly, without me acting as their message bus.

4 November 2026, revised 5 November · 20 min read · [Meta](<#10-where-it-stands>)

[The last post](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-11-04-compact30-constructor/index.md>) was about compact30, my effort to give the decoder's navigation table a smaller representation. Four agents did the research behind it, but until today they could only work together if I copied each one's messages to the next by hand.

This post is about how they now pass work between themselves through the repository: who does what, how a message gets from one agent to another, and what I learnt from running it: an afternoon, a first full cycle, and then a first night with nobody watching. That night produced real gains, and one ordinary mistake that taught me the most: a message can be sent, and recorded as sent, without ever being delivered.

## 1\. A tale of four agents

The Coordinator is Claude Code. It writes the code for each change in the plan, compiles it, runs the tests and keeps the results. When a result is ready it posts a checkpoint: a comment saying what was built and which commit holds the evidence.

The Guardian and the Researcher both read every checkpoint, looking for different things. The Guardian checks the result against the research charter: which gates it passes, what work can follow, and what has to come to me. The Researcher looks at the reasoning: whether the argument holds, whether something else explains the numbers, whether there's a better direction, and whether the Guardian has got it right.

The Optimiser works on the instruction counts themselves. It reads the source, the compiled listings and the execution counts, works out transformations that remove instructions, and keeps the ledger of what each one saves. Its proposals are what the Coordinator builds next.

The Guardian, the Researcher and the Optimiser all run Astra, a model I use through ChatGPT, each as a ChatGPT Work task with its own reasoning-effort setting. What separates them is what they're responsible for, not the model. The roles, and the branch each one owns, are written down in [ROLES.md](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/ROLES.md>). To reproduce the setup, these are the settings:

| Role | Runs as | Model | Reasoning effort | Started by |
| --- | --- | --- | --- | --- |
| Coordinator | Claude Code session | Claude, as configured for the session | High | a subscription to the shared channel, \#145, and the PRs it follows |
| Guardian | ChatGPT Work task, *Compact30 (W) Guardian* | Astra | Extra High | a new comment in its inbox, \#161 |
| Optimiser | ChatGPT Work task, *Compact30 (W) Optimiser* | Astra | Ultra | a new comment in its inbox, \#160 |
| Researcher | ChatGPT Work task, *Compact30 (W) Researcher* | Astra | Extra High | a new comment in its inbox, \#162 |

Each Astra agent's instructions live in the repository as its **agent spec**, beside ROLES.md. [SPEC.md](<https://github.com/lmmx/mibz2/blob/5fbdd2c9c09f753282d9c5608ef236657837efa9/docs/journal/compact30-channel/SPEC.md>) holds what the three have in common: how a run goes, from the inbox comment to the receipt, the work and the review, and the reasoning behind that design. `SPEC.OPTIMIZER.md`, `SPEC.GUARDIAN.md` and `SPEC.RESEARCHER.md` each say only what that agent's work is. A task's prompt in ChatGPT is a single sentence pointing at its two files at one pinned commit. So an agent's behaviour changes only by a commit to its spec and a new pin, both of which stay in the record. SPEC.md also holds the one command that loads a new pin into all three tasks.

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
TO: guardian
RE: prototype 2 @ 29dd071f8787a28560a7a081fd7ac0ef13991466
TYPE: CHECKPOINT
```

`FROM:` names the sender, since every agent posts from the same account. `TO:` records who it's for, `RE:` pins the commit, and `TYPE:` says what kind of message it is. The [protocol](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/PROTOCOL.md>) has six types:

| Type | Means |
| --- | --- |
| `CHECKPOINT` | a result at a pinned commit, ready to assess |
| `REVIEW` | an assessment, with its evidence on the sender's branch |
| `PROPOSAL` | an idea to consider; not permission to build it |
| `DECISION` | a decision the sender is allowed to make |
| `ACK` | what was received, and what happens next |
| `ESCALATION` | something only I can decide |

The messages travel through draft PRs that are never merged. One, \#145, is the team's shared channel, where every agent publishes its reviews and findings. Each Astra agent also has a quiet inbox of its own, where it receives work: [\#160](<https://github.com/lmmx/mibz2/pull/160>) for the Optimiser, [\#161](<https://github.com/lmmx/mibz2/pull/161>) for the Guardian and [\#162](<https://github.com/lmmx/mibz2/pull/162>) for the Researcher. Each experiment has a PR of its own too.

Here is what that looks like on GitHub. Every comment comes from my account, `lmmx`, and the small badge on the avatar shows which app posted it: Claude's mark for the Coordinator, ChatGPT's for the Astra agents. The header is the only thing that says which agent wrote it.

Three agents, one account: a checkpoint from the Coordinator, the Guardian's review of it, and the independent reviewer's report, all on \#145.

**Read the whole exchange**

The originals: [the checkpoint](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394>), [the Guardian's review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082>) and [the independent review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6634356048>).

A new comment is also what wakes the agents up. Each Astra agent is a Work task subscribed to new comments on its own inbox and nothing else, so the filtering happens before a run starts. The inbox is the address: every comment in it is for its agent. The Coordinator posts each checkpoint to the inbox of every agent that should act on it, so a checkpoint for the Guardian and the Researcher is two comments, one in each inbox. Each run fetches its comment, reads from the header who sent it and what kind of message it is, and does the work. It commits a receipt to its own branch first, then its progress as it goes, and publishes its `REVIEW` on \#145. The Coordinator subscribes to \#145, so a reply reaches it within seconds.

This keeps each agent's attention on its own work. An inbox carries only messages meant for its agent, so the other agents' reviews, CI results and pull requests never start a run there. An investigation that takes a quarter of an hour runs to the end undisturbed, and everything it finds still comes back to one place.

One cycle: a checkpoint to two inboxes, two reviews, the next step

Each arrow is a pull-request comment. A checkpoint goes to the inbox of each agent that should act on it, and each review comes back on the shared channel, \#145, where it wakes the Coordinator. No Astra agent polls, and nothing goes through me.

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

Each run has room to finish. Its inbox receives nothing else while it works, and the Coordinator sends an agent its next brief only after the last one has been published. A run commits a receipt first and its progress as it goes, so if one ever stops part-way, a short resume in its inbox picks it up from its last commit.

Watch the repository, not the task pages. Every message ends in a receipt: a cursor commit on the agent's own branch, then a `REVIEW` on \#145 that cites it. A message with a receipt but no review is in progress. One with neither, while the agent's branch is still, gets a resume, once.

When an agent starts a helper of its own, I ask it to keep its run open until the helper has published, and to record both the launch and the result. The independent reviewer was set up this way, with its settings and inputs written to the repository before it began.

Keeping things moving is the Coordinator's job. When the record goes quiet, it checks the receipts, works out who has the next step, and posts the checkpoint that starts it. No Astra agent runs on a timer. On the first night the Coordinator's session had one, waking it every quarter of an hour to re-read its mode note; otherwise it too wakes on events. Nothing sits waiting for me.

## 8\. What a shared message board got wrong

The design above is the second one. The first put every message on one PR, \#145, and let every agent read the whole board. It looked simple, and over the evening of 4 November it went wrong in five ways.

**Every event woke every agent.** Each Work task started on any comment in the repository, fetched it, and decided for itself whether it was addressed. A review from one agent, a CI result, a preview bot's comment or a merged PR started a run in all three, and nearly all of those runs ended straight away with "not for me".

**A new run displaced the one in progress, and nothing caught the loss.** The automation keeps one current run per task. When an event arrived while a long run was still working, the new run took its place, and the earlier one stopped without an error, a partial result or any record of what it had read. There was no dead-letter queue to catch it. That evening six briefs to the Optimiser went this way. Each run started and began reading, and each was overtaken within two or three minutes: by the other agents' reviews of the same checkpoint, by a bot, or by a merge. None of them committed anything. From outside, it simply looked as if the Optimiser had gone quiet.

**Routing by header was brittle.** Each agent decided from `TO:` and `TYPE:` lines whether a comment was meant for it, sometimes by exact string. A checkpoint naming two agents had to be worded exactly right for both, and a message that failed the test was dropped without trace.

**Noise spent the wakes.** Every one of those runs counts against a task's use of the automation, and on a busy board most of them went on comments the agent then ignored.

**The connector added friction.** The Astra agents reach GitHub through ChatGPT's GitHub connector, not a shell with `git` as the Coordinator has. Their runs hit errors such as a call missing its repository argument, or "no usable execution environment". Each was recoverable, but each one lengthened a run, and a longer run was more likely to be overtaken. The task settings also can't show their subscriptions back, so the only way to confirm where an agent listens is to watch where it wakes.

The inboxes fix all five. An agent's subscription now covers its own inbox and nothing else, so the only events it sees are its own work, and there's nothing to filter by header. With nothing else arriving, a long run finishes. A receipt is committed first, so a stopped run leaves a trace that a one-line resume can pick up. The wake budget goes on work. And each agent's instructions are an agent spec at a pinned commit, so what each one does can be read in the repository, not inferred from a task page.

They also bring a failure of their own: when the address is the mechanism, a message sent to the wrong address fails without a sound.

## 9\. Trust, but verify

In the inbox design a comment in \#160 wakes the Optimiser, and the same comment on \#145 wakes nobody but the Coordinator. That is what keeps each agent's attention on its own work.

**A checkpoint that looked dispatched.** On the night of 4–5 November the Coordinator ran the research with nobody watching. At 06:03 UTC it sent the Optimiser a brief in [\#160](<https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083>), and the review came back at 06:12. At 07:38 it posted the next brief, reporting that the fused post-walk pass (\#183) had been rejected, to [\#145](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6652503445>) instead, and at 08:21 an addendum to the [same place](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6653242752>). Both had the right header and the right `TO:`, the first pinned its commit, and GitHub accepted both. Then the Coordinator waited for a review that couldn't come.

I found it at my morning check, by looking at what had happened rather than at what had been reported:

- the two comments were on \#145;
- the Optimiser's inbox had received nothing since 06:03;
- its task's run history showed no run since 06:12.

The Coordinator had intended to dispatch an assignment, and GitHub had confirmed that a comment was posted. Neither fact established that the Optimiser had received the work. Correctness of intent and correctness of record both held. Correctness of effect didn't.

**The recovery didn't need me as a message bus.** I pointed out the discrepancy. The Coordinator re-sent both messages as one checkpoint in the [right inbox](<https://github.com/lmmx/mibz2/pull/160#issuecomment-6653416078>) at 08:31, said so in that message, and [logged the mistake](<https://github.com/lmmx/mibz2/commit/10a8af6e>) in the channel log. The Optimiser's run started from that comment, its cursor marked the brief finished, and its [review](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6653724458>) came back at 08:48, proposing the next build. The work had sat at the wrong address for 53 minutes. Once the error was visible, the system recovered by its own means: a corrected checkpoint in the right inbox, and a review back on \#145.

**A guard that guarded nothing.** At 08:26 the Coordinator squash-merged [\#185](<https://github.com/lmmx/mibz2/pull/185>), naming the commit it expected as the PR's head, so that the merge would be refused if anything had been pushed after CI tested it. It passed `c32ba5db3b0f…`: the first eight characters of the real head, `c32ba5db9df8…`, padded out to forty with characters it made up. The merge went through anyway. The right commit merged, since nothing had been pushed after it. But the check the Coordinator relied on to merge only the commit CI had tested had checked nothing. A short prefix had been refused on an earlier merge, which was why the check seemed to work. The Coordinator noticed when it read the head back, fourteen seconds after merging.

Both mistakes have the same shape: an agent's report of its own action is a claim, like any other claim in this project, and it needs an observable effect behind it. The research already works that way, since every result pins a commit that a reviewer can check. The coordination didn't. The answer isn't for me to watch more closely. It's small, deterministic checks on the few actions whose effects matter, so that a route is computed rather than remembered, and a posted message or a merge is read back rather than assumed. The Coordinator still runs them on itself, so they narrow this failure rather than remove it.

| Check | What it compares | Status |
| --- | --- | --- |
| `route` | an assignment's destination: the role's inbox, or, while it is busy, staged on \#145 as `TYPE: QUEUED` | required before each assignment; none posted since it landed |
| `verify` | the posted comment, read back by ID: its PR, its header, the only assignment in flight | required after each; replayed, it fails the 07:38 misroute and passes the 08:31 dispatch |
| `head-sha` | a merge's expected head against the PR's exact 40-character head, before the merge and, with `--merged`, after it | before: since \#186; after: since \#190; rejects \#185's SHA |
| `audit` | every message on \#145 and the inboxes since the inbox cutover | in use at each session start and wake-up; in its tests, the 120 comments up to 08:49 give exactly the two misroutes |
| the tripwire | each new comment on \#145 or an inbox, checked by a GitHub Actions workflow that runs independently of the Coordinator | in use as a record: a failing check on \#145's head until the next commit there; no event for the Coordinator |

The first four are one stdlib script, [`coord.py`](<https://github.com/lmmx/mibz2/blob/b773431366def44a1214a0c42a6b0fe0b606d870/.github/coordination/coord.py>), added in [\#186](<https://github.com/lmmx/mibz2/pull/186>), and the repository's `CLAUDE.md` says when the Coordinator runs each one. (`TYPE: QUEUED` is a staging label added with the inboxes: work for a busy role waits on \#145 until the Coordinator dispatches it.) The script only reads: it never posts, edits or deletes. The tripwire can't stop a misroute, only report it. It never edits, moves or re-posts a comment, and never dispatches anything itself, so it can't create work of its own.

The tripwire needed the same treatment as everything else. A workflow started by a comment runs only from master, so each version could be tested only after it merged. The Coordinator tested each one with a deliberate, labelled misroute on \#145, and each test found something that the code and its unit tests hadn't. (A check run is one line in a PR's list of checks. GitHub groups a workflow's check runs into a check suite, and the Coordinator's subscription hears only when a suite finishes.)

- **The first** was classified correctly, and then its warning couldn't be posted: \#145 is locked, and the Actions bot can't comment on a locked conversation.
- **The second** raised a failing check run on \#145 instead. The check showed red, but nothing woke the Coordinator.
- **The third** made its check run start in progress and then fail, to reopen the suite. The suite stayed green and sent no event: a GitHub Actions suite follows its workflow run, not check runs added to it.

One of these tests refuted a claim that had already merged: \#188 went in saying its check run would wake the Coordinator, and its test showed within a minute that it didn't.

So the tripwire is a record, not an alarm. It leaves a failing check on \#145's head commit, independently of the Coordinator, and the check shows on \#145 until the next commit to the channel branch. The Coordinator finds the misroute itself, by running `audit` at the start of each session and every time it wakes.

The merge guard needed the same treatment, and got it only while this post was being checked. A reviewing agent pointed out that GitHub refuses a mismatched head, and it does: a merge of [\#139](<https://github.com/lmmx/mibz2/pull/139>) was refused with a 409 on 2 November. The guess had never reached GitHub. The merge tool takes the expected head as `expectedHeadSha`, and the Coordinator had been passing it as `sha`, a name the tool doesn't have and drops without a word. The prefix refused earlier had met the tool's own format check, under the right name. Read back, eight merges between 02:12 and 09:36 had passed the head that way, among them an earlier guess on [\#167](<https://github.com/lmmx/mibz2/pull/167>) and four merges made after `head-sha` existed. Each merged its PR's final head, so nothing wrong landed, but none was checked. `head-sha` had checked the value; nothing had checked that the value reached the merge. [\#190](<https://github.com/lmmx/mibz2/pull/190>) adds the missing step: the SHA goes in `expectedHeadSha`, and `head-sha --merged` reads the merge back and passes only if the PR merged at exactly that head.

Three gaps remain. The checks are run by the Coordinator on itself: `CLAUDE.md` says when, but nothing outside its session makes it run them, and the tripwire, the one check outside it, can only leave a mark. The Coordinator can't see the Astra tasks' run history; it infers a run from receipts in the cursors, which `route` reads. And a brief that was delivered but never started, one with no receipt half an hour after it was posted, is still caught only by the Coordinator noticing a quiet record (section 7). A mechanical check for it would close the loop from sent to started. It isn't built.

## 10\. Where it stands

The [channel log](<https://github.com/lmmx/mibz2/blob/abc14cc284bb971142cf5347bf8072014bc5a37a/docs/journal/compact30-channel/log.md>) records each step. The trigger was first tested on the Guardian: a comment started its run, it published an assessment, and its review woke the Coordinator. The first real cycle followed on [a checkpoint](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6632963394>) covering two prototypes. The Guardian's run started nine seconds after it was posted, checked both prototypes against the gates, and [asked](<https://github.com/lmmx/mibz2/pull/145#issuecomment-6633274082>) for the independent review.

That review came back while this post was being finished. The Researcher launched the isolated reviewer, and its [report](<https://github.com/lmmx/mibz2/pull/153>) arrived as a comment on the channel. Both prototypes pass on correctness and safety, and so does v3's finaliser. Two things need fixing before anything is combined: some changed paths still need their own execution counters, and both prototypes remove the same instruction, so their savings overlap. That is the next checkpoint.

On 5 November the redesigned setup ran its first full round:

- **The Optimiser** ranked the whole decoder's candidates from its inbox. It put P87, a production-shaped change that writes the walk's output over a buffer it has already consumed, first.
- **The Coordinator** built P87 and registered a decision rule before measuring it.
- **The Guardian and the Researcher** reviewed that rule from their inboxes before the benchmark finished. The Guardian's review changed it.
- **The change** made the four-worker Wikidata decode 2.2% faster in one run, and favourable but inconclusive in a second, with 6.8% less memory and no slower row. It [merged](<https://github.com/lmmx/mibz2/pull/164>).

Then all three agents took their next assignment under the new specs:

- each woke only on its own inbox;
- each committed a receipt within a minute or two, and published its review on \#145;
- the Guardian also moved itself onto its current operating spec.

The first reviews under the new design corrected the Coordinator's write-up before it was published.

The rest of that night was the first long test. At 02:52 UTC, at my instruction, the Coordinator's [mode note](<https://github.com/lmmx/mibz2/blob/e8b136fed714b54d79b7fa9914b70f815d2a4d8a/docs/journal/compact30-channel/MODE.md>) was amended to tell it to fan out hard on the hard research, and to do tests, CI and documentation itself rather than spend agents on them. From then until my morning check it ran with nobody watching. Only the Optimiser was in the loop: the Guardian and the Researcher received no work after 00:36.

The Coordinator's Claude Code workflows put several agents on one question at once. In one, three agents mined an old CI sweep, modelled each runner's per-core miss resources and built a walk keyed on SMT co-residency, and three sceptics each refuted part of that work. Another reviewed the fused post-walk change through three lenses, one of them a differential of about 14,600 valid and corrupt inputs against master, which [found no divergence](<https://github.com/lmmx/mibz2/pull/160#issuecomment-6653416078>). The workflows' own records stay in the Coordinator's session; the journal keeps what they found. Every decoder change that reached a pull request still went through the CI benchmark, under a decision rule registered before its first run, without review by the Guardian or the Researcher.

Five decoder changes merged. Two trimmed the move-to-front stage ([\#170](<https://github.com/lmmx/mibz2/pull/170>), [\#172](<https://github.com/lmmx/mibz2/pull/172>)). [\#171](<https://github.com/lmmx/mibz2/pull/171>) staged the walk's output through one tile, for a four-thread wall 5% to 14% faster over five runs. [\#180](<https://github.com/lmmx/mibz2/pull/180>) widened the window of blocks in flight, for 8% to 10% in three runs, at a cost of 4% to 6% more CPU and 8 MB more memory. The Coordinator weighed that in the journal and accepted it, noting that the extra CPU still has to be measured in wikidata-pq's pipeline, where the decoder shares its cores. [\#181](<https://github.com/lmmx/mibz2/pull/181>) removed the deferred flags that \#171 had carried with it: the decoder ran about 4% more instructions and still got faster. That fits the reading that the walk is bound by memory requests in flight rather than by instructions, though no hardware counter has shown it. The [benchmark history](<https://lmmx.github.io/mibz2/pr-preview/pr-379/benchmarks/history/index.md>), which chains each merge's own paired run, now puts the four-thread Wikidata wall at 0.75× its value at \#69 (95% band 0.69–0.82), against 0.85× before \#180. The chain assumes that a change measured on one runner holds on another.

As much again was tested and rejected. The 32 KiB tile, 1024 offsets a lane, filled the whole of Zen 4's L1D and ran 23.5% slower even at one thread (\#177, \#178). The fused post-walk pass ([\#183](<https://github.com/lmmx/mibz2/pull/183>)) halved the simulated cache misses after the walk, but the four-thread wall wasn't shown to follow: neither PR run was faster there beyond noise, though the fusion alone was, by 1.5%. October's four-copy win for a walk keyed on SMT co-residency reversed on today's staged walk. Split condition variables were rejected on a count of wasted wakes before anything was built.

Each is written up in the journal with its runs. The walk round had already redirected the work: since the 06:03 checkpoint the Optimiser has ranked candidates by the memory traffic of four workers on two SMT cores, not by instruction counts, and the fused pass and split condition variables were its first and third picks. Their failures narrowed the search to the walk's random reads and its table's footprint. The next build the Optimiser proposed is a retest of compact30's smaller table under today's staged walk, though its own prior on it is weak.

So the work moves without me relaying it. When a message went to the wrong address, the record showed it, and the Coordinator corrected it without my carrying anything across, though it took me to notice. Now the Coordinator reads each message and each merge back rather than assume they landed, and a misroute leaves a failing check on \#145 that the Coordinator didn't write. Whether all of this makes the research faster will keep showing in the benchmark history.

If you want to set up something similar:

- give each agent its own inbox, and one shared place to publish;
- have each run commit a receipt before it works, and its progress as it goes;
- point every message at an exact commit;
- wake agents with events rather than timers, and keep those events few;
- keep every agent's instructions in the repository at a pinned commit;
- write down what a message is allowed to authorise;
- decide what an independent reviewer gets to see;
- check effects, not reports: read every message back where it should have landed, and every merge back after it lands.

My own part is choosing the scope and making the decisions the charter keeps for me.

---

**Research records:** [Role identities](<https://github.com/lmmx/mibz2/blob/8304583b/docs/journal/compact30-channel/ROLES.md>) · [Research charter](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/2026-11-04-compact30-research-charter.md>) · [Channel protocol](<https://github.com/lmmx/mibz2/blob/8304583b/docs/journal/compact30-channel/PROTOCOL.md>) · [Review contract](<https://github.com/lmmx/mibz2/blob/09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47/docs/journal/compact30-channel/REVIEWER.md>) · [Recorded exchanges](<https://github.com/lmmx/mibz2/blob/abc14cc284bb971142cf5347bf8072014bc5a37a/docs/journal/compact30-channel/log.md>) · [Routing checks](<https://github.com/lmmx/mibz2/blob/b773431366def44a1214a0c42a6b0fe0b606d870/.github/coordination/coord.py>).
