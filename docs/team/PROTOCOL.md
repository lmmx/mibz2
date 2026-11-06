# The team's protocol

How the roles (ROLES.md) exchange work through the repository. The channel carries evidence and
messages, not authority. Authority is in RULES.md. Every role writes in MODE.md's register,
and reads MODE.md first on every run.

## 1. Where

- **The channel, PR #145.** One standing draft PR, never merged.
  - Every role publishes its reviews and findings here, whatever woke it.
  - Only the Coordinator is subscribed to it.
  - Its comments are the team's verbatim, durable record (RULES E1).
- **Inboxes, one per role:**
  - #160 Optimiser;
  - #161 Guardian;
  - #162 Researcher;
  - #222 Oracle.

  Each is a quiet draft PR, never merged. A role's Work task is woken by its inbox PR's webhook (SPEC.md, "Your trigger"); new comments on its
  own inbox alone, so the inbox is the address.
  - Only the Coordinator writes in an inbox, and after its offset it writes only assignments
    (CHECKPOINT, PROPOSAL, CORRECTION).
  - Nobody pushes to, edits or retitles an inbox PR, because lifecycle events can't be excluded
    from a subscription.
- **State branches**, one per role (`team-state/<role>`):
  - each holds its role's cursor and records under `state/`;
  - each is written only by its role;
  - none has a PR, and the workflows' push filters run no CI on them.
- **The Oracle's record: `observatory/intention`**, an orphan branch.
  - It shares no history with master, and the workflows' filters run no CI on it.
  - It is written only by the Oracle, by direct commit, never through a PR.
- **Experiment PRs** go into master, and their checkpoints go to the inboxes.

## 2. Dispatch: one assignment in flight per role

- **When a role is busy:** while any of its inbox comments after its offset is not marked
  complete in its cursor. That includes a brief just posted, which has no receipt yet.
- **Work for a busy role** is staged on #145 as `TYPE: QUEUED` with `TO: <role>`.
- **The next item goes out** when the role's REVIEW citing its current comment is on #145, and
  its cursor marks that comment complete.
- **Recovering the queue:** a QUEUED comment has been dispatched exactly when an inbox comment
  cites it.
- **A silent run:** a run that has ended without finishing gets one resume request, naming the
  comment.
- **A HOLD or STOP** takes effect through the Coordinator's own authority, without an inbox.
- **Before posting:** `coord.py route` gives the destination. **After posting:** `coord.py
  verify` must pass. **On every wake:** `coord.py audit`.

## 3. Message format

```
FROM: coordinator | optimizer | guardian | researcher | oracle | user
TO:   <role>                (assignments and QUEUED)
RE:   <subject> @ <commit sha>
TYPE: CHECKPOINT | PROPOSAL | CORRECTION | QUEUED | REVIEW | DECISION | ACK | LOG | ESCALATION
DECISION: HOLD | STOP | REVISE <what> | PROCEED      (DECISION only)
```

Long material goes in a committed file, which the comment links at a commit. A comment with a
missing or malformed header is treated as a REVIEW, never as a DECISION. What a message can
trigger is RULES A1–A4: anything narrowing is honoured; anything that would widen authority
counts only from Louis.

## 4. The Coordinator's duties on each message

1. **Act or record.** Answer with an ACK or LOG on #145 that states:
   - what it understood;
   - the rule it relies on (RULES.md);
   - the action it takes, or that it takes none.
2. **Treat ambiguity, or conflict between messages, as a HOLD,** and say so.
3. **A request that needs Louis** (RULES D) is refused in the ACK and escalated (§5).
4. **Forward an `ORACLE:` line** in a role's REVIEW to the Oracle's inbox as one CORRECTION,
   once for each item and its evidence (SPEC.ORACLE.md).

## 5. Escalation to Louis

- **When:**
  - a decision RULES D reserves;
  - a stop whose resolution would need one;
  - reviewers who disagree on a decision.
- **Not escalated:**
  - a stop under RULES B6: the work stops, the Guardian is told, and the Coordinator fixes it,
    falls back or abandons it;
  - routine technical direction (RULES A5).
- **How:** a message in the Coordinator's session saying exactly what is blocked, and what his
  options are.
- **Meanwhile,** work that doesn't depend on it continues.
- Louis is never the notification mechanism: the Coordinator never asks him to wake a role.

## 6. Runs

- **A new event can displace a run in progress.** Nothing may rely on a message waiting for a
  run to end. Protection comes from dispatch (§2), not from queueing on the platform.
- **Every assignment ends in a receipt:** the cursor entry, then the REVIEW citing the comment.
- **Commits on state branches:** `Louis Maddox <lmmx@users.noreply.github.com>` (the Work
  tasks' connector) or `<louismmx@gmail.com>`, with no AI attribution.

## History

This replaces `docs/journal/compact30-channel/PROTOCOL.md`, which is kept as history. Dropped:
- the end-to-end test and the stage gating (done: RULES E3);
- the verbatim transcription into `log.md` (RULES E2);
- the review queue;
- the canonical copies on the compact30 branches.
