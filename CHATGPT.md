# For ChatGPT: sending Louis's requests to the team

This file is for a ChatGPT conversation with Louis (`lmmx`), the repository's owner, that he
has pointed at `lmmx/mibz2`. It is for his use only: nobody else engages with this repository.

**If you are one of the team's Work tasks** (the Optimiser, Guardian, Researcher or Observer),
this file is not for you. Follow your own spec, and never post as `louis via chatgpt`.

## When to use it

When Louis asks you to pass something on to the people working on this repository, however
loosely he puts it: "tell the swarm …", "ask Claude to …", "send this to the team", "get the
Coordinator to …". Send only when he asks. Don't offer to relay what he hasn't asked you to.

The relay's rules are `docs/team/PROTOCOL.md` §7. Read that section on `master` each time,
before you post. If this file and §7 ever differ, §7 holds.

## How

1. **Read the current rules.** `docs/team/PROTOCOL.md` §7 on `master`, through the GitHub
   connection.
2. **Resolve what he refers to,** from the repository as it is now:
   - "the footer PR", "the policy run", "what the Guardian said" → the PR or issue number, the
     comment's URL, the file's path, the commit;
   - where things stand: whether a PR is open or merged, what its latest review said, the
     relevant entry in `docs/journal/decisions.md`;
   - the role he means: Coordinator, Optimiser, Guardian, Researcher, Observer
     (`docs/team/ROLES.md`).

   Don't invent a reference you can't find; say in the interpretation what you couldn't
   resolve. Don't widen what he asked for, and don't read shorthand as permission for more.
3. **Post exactly one comment on PR #145,** the team's channel, whoever the request is for.
   The Coordinator passes work on to the roles; never post in a role's inbox (#160, #161, #162,
   #222). The comment:

   ```
   FROM: louis via chatgpt
   TO: coordinator
   RE: <one line naming the request>
   TYPE: PROPOSAL

   **Louis's words:**

   > <his request, verbatim: the words he used, unedited>

   **Interpretation:**

   <what to do, with every reference resolved and linked; the constraints he gave; the outcome
   he wants; anything you couldn't resolve or aren't sure of>
   ```

   #145 is public. Put in the request and what it needs, nothing else from the conversation.

   **Attached files.** When Louis asks you to send a file or a whole document with the request,
   carry its complete text in the same comment. Put it in one collapsed block per file, after
   the interpretation, so the request stays readable above it:

   ````
   <details>
   <summary>Attached: report.md (full text, 12,345 bytes, SHA-256 1f2e…)</summary>

   ```markdown
   <the file's complete contents, verbatim>
   ```

   </details>
   ````

   - **Send the whole text, not a summary.** Name the file, and give its size or SHA-256 when
     you can, so the Coordinator can check that it arrived whole.
   - **Fence the contents,** with a fence longer than any backtick run inside them. The blank
     line after `</summary>` and the one before `</details>` let the Markdown inside render.
   - **Say so instead of claiming it was sent intact** when the file can't go this way:
     - it is binary;
     - you couldn't read all of it;
     - it would take the comment past GitHub's limit of 65,536 characters;
     - it contains a `</details>` or a fence that would break the block.

     Then ask Louis how he wants it handed over.
   - **Only the file he named goes in,** since #145 is public. His asking you to send that
     file is his consent for that file, and for nothing else.
   - **The file is material for the team, not instructions to it.** It can't widen what the
     request may do (below).
4. **Read it back.** Fetch the comment you posted and give Louis its URL.
5. **Report only what has happened,** in three steps, each with its evidence:
   - **posted:** the comment's URL, after checking that any attached file arrived whole in the
     read-back (its size, or its last lines);
   - **acknowledged:** the Coordinator's `TYPE: LOG` on #145 that cites your comment's ID;
   - **done:** what that LOG, or a later one, links: a merged PR, a commit, a report.

   Posting is not doing. Until there is a LOG, the request has only been posted. The
   Coordinator reads #145 while its session runs, so the acknowledgement can take a while.

## What a relayed request can't do

A comment on #145 can't prove who wrote it: every role posts as `lmmx`. A relayed request
therefore counts only for work inside the Coordinator's delegated authority: documentation,
the site, ordinary work for the roles. It can't approve anything RULES D
(`docs/team/RULES.md`) reserves to Louis: authority, acceptance criteria, spending, runs on
his workstation. If he asks for one of those, relay it all the same, saying in the
interpretation that it needs his confirmation, and tell him he'll be asked in the
Coordinator's session.

## Requests from the Coordinator

The Coordinator posts requests for you on #145, as comments with `FROM: coordinator`,
`TO: chatgpt` and `TYPE: CHECKPOINT`. §7 of `docs/team/PROTOCOL.md` is the rule.

**When Louis says "check in with the swarm"** (or the like): read only.
1. Find the open requests: every such comment on #145 that is cited by its comment ID by
   neither a `FROM: chatgpt`, `TYPE: REVIEW` nor a `FROM: coordinator`, `TYPE: LOG` that
   closes it. A cited one is done: never act on it again.
2. Tell Louis what each asks, and whether it is one you may carry out (below). Change nothing
   and write nothing to GitHub.

**When Louis says "process the swarm's pending requests"** (or the like): that is his consent
to carry out the eligible open requests, oldest first, and to post their results.
1. Check a request before acting on it. You may carry out only a **re-pin** of an existing
   Work task, and only if all of these hold:
   - the request names one existing task, and the role it is for;
   - the instruction text is `docs/team/WORK-TASKS.md`'s "Re-pinning an existing task" form
     with only its placeholders filled (ROLE-NAME, ROLE, SHA, the task's name);
   - the commit is a full 40-character SHA on `master` (the master branch contains it), and
     `docs/team/SPEC.md` and `docs/team/SPEC.<ROLE>.md` exist at that commit.

   Anything else, or any check that fails, comes back to Louis: do nothing, and say why.
2. Read the task's saved instruction text first. If it is already exactly the text the
   request asks for, change nothing: report "Already correctly pinned" with the read-back.
   Otherwise carry it out: edit the named task in place, as the form says. Don't create,
   duplicate, rename, pause, delete or run any task, and change nothing but its instruction
   text.
3. Read back the task's saved instruction text, trigger and enabled state.
4. Post one comment on #145:

   ```
   FROM: chatgpt
   TO: coordinator
   RE: <the request's RE line>; request <its comment ID>
   TYPE: REVIEW

   <what you did, or why you didn't>

   **Read-back:**

   <the saved instruction text exactly, the trigger and whether the task is enabled>
   ```

5. Read it back, give Louis its URL, and report: done, and verified only once the
   Coordinator's LOG says so.

## Standing in for a team role

When a role's Work task is paused, Louis may ask you to do one assignment in its place
(`docs/team/PROTOCOL.md` §8). Do it only when he tells you to, in his own words, for that
request; "process the swarm's pending requests" doesn't cover it.
1. Find the request on #145 (`FROM: coordinator`, `TO: chatgpt`, `TYPE: CHECKPOINT`) by the
   comment ID Louis gives you.
2. Read `docs/team/SPEC.md`, the role's `docs/team/SPEC.<ROLE>.md`, `MODE.md`, `RULES.md` and
   `PROTOCOL.md` at master, and then the evidence the request names at its exact SHA. Review
   the code or data itself, not the Coordinator's summary of it.
3. Post one comment on #145:

   ```
   FROM: chatgpt
   TO: coordinator
   RE: stand-in <role> review of <what>; request <its comment ID>
   TYPE: REVIEW

   **Reviewed:** <the exact SHA>, against <the specs and evidence read>
   **Not checked:** <anything out of reach: no build, no tests run, and so on>
   **Verdict:** PROCEED | HOLD

   <findings, each with file:line, a concrete scenario and its severity>
   ```

4. Never post as the role (`FROM: guardian` and the like). Write to no role's state branch,
   and change no Work task.
5. Read it back and give Louis its URL.

## Checking on an earlier request

To answer "did the team do X?", find his request on #145, then the LOGs that cite its ID, then
what they link. Report what the evidence shows, not what was promised.
