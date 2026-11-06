# ChatGPT Work task prompts

The only prompts the Coordinator gives Louis for ChatGPT Work tasks. Each is copied from here,
with only its placeholders filled in. None is drafted fresh (Louis, 2026-11-06). Each form is
marked:
- **known to work:** Work has carried it out;
- **untested:** never run yet. It is offered as untested, and is updated here once Louis
  reports the result.

## What Work can and can't do (learned)

- **Model and effort can't be set by a prompt.** The task tool doesn't expose them. Louis
  selects them in the model setting of the Work task chat window. So every prompt is given with
  its model, its effort and the task's name **beside** it, never inside it.
- **The UI needs a signed-in browser.** A prompt that asks the chat agent to change settings
  only reachable through the interface fails when the browser is signed out (2026-11-06).
- **One task per creation prompt.** A prompt that creates one task and edits others confuses
  it (2026-11-06).

## Creating a task — known to work

This is the form that created the first three tasks (2026-11-04). It was confirmed again for
the Oracle on 2026-11-06.

Beside the prompt, tell Louis:
- the **name**: `mibz2⠶<Role>⠶<effort code>`;
- the **model ⠶ effort** to select in the chat window (ROLES.md lists each role's).

```
Create an event-triggered task, not a scheduled one and not a one-off, that runs automatically whenever a new comment is posted on pull request #N in `lmmx/mibz2`. Subscribe to newly created comments only, excluding bots, reviews, commit updates and edits where supported. Do not filter by comment content. Before enabling it, show me the trigger, the condition, the GitHub permissions it will use, and whether it runs on my plan or costs anything extra. The task's instructions are:

You are the ROLE-NAME of the mibz2 agent team (lmmx/mibz2). Fetch docs/team/SPEC.md and docs/team/SPEC.ROLE.md from lmmx/mibz2 at commit SHA, and follow them. Those files, at that commit, are your complete instruction. If they can't be fetched, report that and stop. Keep this task enabled.
```

Fill in N (the role's inbox PR), ROLE-NAME (Optimiser, Guardian, Researcher, Oracle), ROLE
(OPTIMIZER, GUARDIAN, RESEARCHER, ORACLE) and SHA (the pin). Nothing else changes.

- **Why the instructions are only a pointer:** the rules (commit identity, no polling, no
  widening) live once, in the pinned spec. A copy in the task would drift from it.
- **Why "do not filter by comment content":** the inbox is the address. A content filter, such
  as the first tasks' `TYPE: CHECKPOINT`, drops other assignments, for example `CORRECTION`.
- **Why there is no commit identity:** the connector commits as
  `lmmx@users.noreply.github.com`, which SPEC.md allows. Forcing another address could block
  every commit.
- **Why the subscription line is there:** edits, bot comments and reviews would start runs that
  displace real work (PROTOCOL.md §6).

Then ask Louis to send back what Work shows before enabling: the trigger, the condition, the
permissions and the instruction text as saved. The Coordinator checks these against the pin
before the first assignment.

## Re-pinning an existing task — untested

This is for one existing task at a time, so that it loads the specs at a new pin. Beside the
prompt, tell Louis the task's name, and to leave its model and effort as set.

```
Edit the existing event-triggered task named NAME in place. Don't create, duplicate, rename, pause or delete any task, and don't run it. Replace its whole instruction text with exactly this, and change nothing else:

You are the ROLE-NAME of the mibz2 agent team (lmmx/mibz2). Fetch docs/team/SPEC.md and docs/team/SPEC.ROLE.md from lmmx/mibz2 at commit SHA, and follow them. Those files, at that commit, are your complete instruction. If they can't be fetched, report that and stop. Keep this task enabled.

Then show me the instruction text exactly as saved, the trigger and subscription, and whether the task is enabled.
```

When Louis reports the result, mark this form known to work, or record what failed.

## Revisions

- 2026-11-06: created, after a combined create-and-edit command and a prompt that tried to set
  the model both failed.
