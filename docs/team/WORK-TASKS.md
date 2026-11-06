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
- **A new task's webhook isn't live at once.** The Oracle's first assignment, posted about a
  minute after its task was created, never triggered a run. A resume request posted about 15
  minutes later started one within 8 seconds (2026-11-06). So wait a few minutes after creating
  a task before its first assignment. If the task's run list shows no run, post one resume
  request (PROTOCOL.md §2).
- **Renaming is done by hand** in the Work interface. No prompt for it is recorded.
- **One task per creation prompt.** A prompt that creates one task and edits others confuses
  it (2026-11-06).
- **The trigger can't be restricted to comments only.** Asking it to be made Work stop and ask
  (2026-11-06). So the prompt doesn't ask. If Work raises it anyway, accept the broader
  subscription and add no guard: the specs ignore events that bring no new inbox comment.
- **Its permission and cost answers are generic.** It runs on the existing GitHub connection
  ("Allow low-risk actions") and on Work usage. It can't certify the scopes or billing, and
  doesn't need to.

## Creating a task — known to work

It created the Oracle task first time, with no question, on 2026-11-06. Its saved trigger was the
`pull_request` webhook on #222 (comments, reviews, bot reviews and commit updates enabled), and
its instruction text matched the template byte for byte. This is ChatGPT's form of 2026-11-06,
agreed with the Coordinator. It removes the four traps
that made Work investigate and ask instead of creating the task (see below). It also worked with the older pin `282fcbb0`, whose SPEC.md still mentions
exclusions, so the prompt alone is enough.

Beside the prompt, tell Louis:
- the **name**: `mibz2⠶<Role>⠶<effort code>`;
- the **model ⠶ effort** to select in the chat window (ROLES.md lists each role's);
- to start it in a **new** Work chat.

```
Create and enable a persistent GitHub event-triggered Work task for pull request #N in `lmmx/mibz2`. It must not be a scheduled task or a one-off.

Use the GitHub pull-request webhook subscription supported by the platform. I explicitly accept its full event coverage, including incidental pull-request lifecycle events and review events. No narrower event filter is required. The task's pinned specifications handle checking its inbox for new assignments.

This is my authorisation to create and enable the task with that event coverage. Do not ask me to approve the broader subscription again.

Use my existing GitHub connection and normal Work usage settings. Do not change permissions, billing settings or purchase credits. No preliminary feasibility, permissions or pricing investigation is required.

Create and enable the task now. Do not run its work immediately. After creation, report the saved trigger, exact instruction text and enabled status. Report settings you cannot inspect as unknown rather than treating them as prerequisites.

The task's instructions are exactly:

You are the ROLE-NAME of the mibz2 agent team (lmmx/mibz2). Fetch docs/team/SPEC.md and docs/team/SPEC.ROLE.md from lmmx/mibz2 at commit SHA, and follow them. Those files, at that commit, are your complete instruction. If they can't be fetched, report that and stop. Keep this task enabled.
```

Fill in N (the role's inbox PR), ROLE-NAME (Optimiser, Guardian, Researcher, Observer), ROLE
(OPTIMIZER, GUARDIAN, RESEARCHER, OBSERVER) and SHA (the pin). Change nothing else.

**If Work asks anything before creating the task, the form has failed.** Record what it asked
and why, and fix the form or the spec. A question is not a step.

**The traps it removes** (2026-11-06):

| earlier wording | what Work did |
|---|---|
| "whenever a new comment is posted" | read it as requiring a comments-only webhook, which doesn't exist |
| "before enabling it, show me…" | turned creation into a preflight approval |
| "whether it runs on my plan or costs anything extra" | started a billing investigation it couldn't complete |
| "exclude bots, reviews, commit updates and edits", in the prompt or in the pinned SPEC.md it fetched | demanded controls the connector doesn't expose, then asked whether to add a condition to the instructions |

- **Why the instructions are only a pointer:** the rules live once, in the pinned spec. A copy
  in the task would drift from it.
- **Why the pin matters here too:** Work fetches the spec during creation. A spec that demands
  what the trigger can't do makes it stop, whatever the prompt says.

The Coordinator then checks the saved instruction text against the pin before the first
assignment.

## Re-pinning an existing task — known to work

It re-pinned the Optimiser, Guardian and Researcher tasks to `282fcbb0` on 2026-11-06. Their saved
texts were read back character for character, and nothing else changed. Several prompts can go
in one chat, as long as each is sent as its own message. Tell Work the task's current name: if a
prompt says NAME, it asks.

This is for one existing task at a time, so that it loads the specs at a new pin. Beside the
prompt, tell Louis the task's name, and to leave its model and effort as set.

```
Edit the existing event-triggered task named NAME in place. Don't create, duplicate, rename, pause or delete any task, and don't run it. Replace its whole instruction text with exactly this, and change nothing else:

You are the ROLE-NAME of the mibz2 agent team (lmmx/mibz2). Fetch docs/team/SPEC.md and docs/team/SPEC.ROLE.md from lmmx/mibz2 at commit SHA, and follow them. Those files, at that commit, are your complete instruction. If they can't be fetched, report that and stop. Keep this task enabled.

Then show me the instruction text exactly as saved, the trigger and subscription, and whether the task is enabled.
```



## Revisions

- 2026-11-06: the webhook warm-up recorded (the Oracle's first assignment was missed; a resume
  request started its run).

- 2026-11-06: the re-pin form worked for all three existing tasks. Marked known to work.

- 2026-11-06: ChatGPT's form created the Oracle task first time. Marked known to work.
- 2026-11-06: ChatGPT's form adopted, with its four traps recorded. It is pinned to a
  commit whose SPEC.md no longer demands exclusions. Untested until it creates a task with no
  question asked.

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
