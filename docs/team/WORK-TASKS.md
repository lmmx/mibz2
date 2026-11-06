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
- **The trigger can't be restricted to comments only.** Asking it to be made Work stop and ask
  (2026-11-06). So the prompt doesn't ask. If Work raises it anyway, accept the broader
  subscription and add no guard: the specs ignore events that bring no new inbox comment.
- **Its permission and cost answers are generic.** It runs on the existing GitHub connection
  ("Allow low-risk actions") and on Work usage. It can't certify the scopes or billing, and
  doesn't need to.

## Creating a task — untested

This keeps the opening sentence of the prompt that created the first three tasks (2026-11-04).
It then answers, in advance, every question Work raised while creating the Oracle on
2026-11-06:
- the lifecycle events;
- the spec's exclusion wording;
- permissions and cost.

Its aim is that the task is created without a single question.

Beside the prompt, tell Louis:
- the **name**: `mibz2⠶<Role>⠶<effort code>`;
- the **model ⠶ effort** to select in the chat window (ROLES.md lists each role's);
- to start it in a **new** Work chat.

```
Create an event-triggered task, not a scheduled one and not a one-off, that runs automatically whenever a new comment is posted on pull request #N in `lmmx/mibz2`. Use the GitHub webhook for that PR as available, including its PR lifecycle events: those are expected and accepted, and the pinned spec ignores any event without a new comment, so add no condition and change nothing in the instructions. Where the spec mentions excluding bots, reviews, commit updates or edits, that means "where the settings allow it", so the available webhook satisfies it. The cost and permission terms are accepted; if you can't verify them exactly, say so in your report, but don't let that stop the creation. Create and enable the task now, without running it, and don't ask me anything further. Then report the saved trigger, the instruction text exactly as saved, and whether it is enabled. The task's instructions are:

You are the ROLE-NAME of the mibz2 agent team (lmmx/mibz2). Fetch docs/team/SPEC.md and docs/team/SPEC.ROLE.md from lmmx/mibz2 at commit SHA, and follow them. Those files, at that commit, are your complete instruction. If they can't be fetched, report that and stop. Keep this task enabled.
```

Fill in N (the role's inbox PR), ROLE-NAME (Optimiser, Guardian, Researcher, Oracle), ROLE
(OPTIMIZER, GUARDIAN, RESEARCHER, ORACLE) and SHA (the pin). **Change nothing else.**

**If Work asks anything before creating the task, the form has failed.** Record what it asked
and why, and fix the form or the spec so that it cannot ask again. A question is not a step to
be expected.

- **Why the instructions are only a pointer:** the rules (commit identity, no polling, no
  widening) live once, in the pinned spec. A copy in the task would drift from it.
- **Why the condition names no comment content:** the inbox is the address. A content filter,
  such as the first tasks' `TYPE: CHECKPOINT`, would drop other assignments, for example
  `CORRECTION`.
- **Why the prompt asks for no event exclusions:** Work's trigger can't be limited to comments,
  and asking for that made it stop instead of creating the task. PROTOCOL.md §1 already
  accommodates the extra events: nobody touches an inbox PR, and the spec ignores any event that
  brings no new inbox comment. The scripted reply answers that question if Work raises it.
- **Why unverifiable checks don't block:** Work can't inspect exact GitHub scopes or billing.
  The scripted reply says that's fine.

The Coordinator then checks the saved instruction text against the pin before the first
assignment.

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

- 2026-11-06: the unchanged original form failed too. Work fetched the pinned SPEC.md, read its
  "exclude bots, reviews, commit updates and edits" as a hard requirement it couldn't meet, and
  asked whether to add a condition. The form now answers that in advance, and SPEC.md (from the
  next pin) states that lifecycle events are expected and need no condition. Rule: any question
  from Work during creation means the form has failed.

- 2026-11-06: settled with ChatGPT on the original form unchanged, plus one scripted reply for
  its confirmation step (accept the broader events; unverifiable checks are fine).
- 2026-11-06: the creation form restored to the exact wording that worked. The added subscription
  and content-filter sentences, untested yet labelled as known to work, made Work stop and ask.
  Rule: in a known-good form, change only the placeholders.

- 2026-11-06: created, after a combined create-and-edit command and a prompt that tried to set
  the model both failed.
