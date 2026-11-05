# Work task instructions

Each Astra role's Work task runs from two files here, at a pinned commit: `RUN.md`, how every run goes, and its role's file, what its work is. Its prompt in ChatGPT
Work is only a pointer to that file. Its subscription is set in the task's settings, not here.

| Role | File | Subscription |
|---|---|---|
| Optimizer | [optimizer.md](optimizer.md) | new comments on lmmx/mibz2 PR #160 |
| Guardian | [guardian.md](guardian.md) | new comments on lmmx/mibz2 PR #161 |
| Researcher | [researcher.md](researcher.md) | new comments on lmmx/mibz2 PR #162 |

Every subscription excludes bots, review submissions, pushes, commit updates, comment edits and
lifecycle events.

**The pointer prompt**, with the role and its file filled in:

> You are the compact30 ROLE for lmmx/mibz2. Fetch `docs/journal/compact30-channel/tasks/FILE`
> and `docs/journal/compact30-channel/tasks/RUN.md` from lmmx/mibz2 at commit SHA, and follow
> them. Those files, at that commit, are your complete instruction. If they can't be fetched,
> report that and stop. Keep this task enabled.

A changed file takes effect only when Louis moves a task's pin to a new commit. The Coordinator
compares each role's recorded pin with the pin recorded here at every start or resume
(CAMPAIGN.md, start step 4).
