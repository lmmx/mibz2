# SPEC: event-triggered Acting Coordinator — GitHub channel monitor

**Status:** Owner-authorised failover design, untested until a real persistent Work task is created and verified. Valid only after setup PR #364 is merged to master and Louis has explicitly activated the acting operator. **This is a monitoring/dispatch companion to the existing Coordinator role, not a second Coordinator.**

**Event source:** existing team channel, [PR #145](https://github.com/lmmx/mibz2/pull/145), via a GitHub `pull_request` Work webhook subscription. The subscription may include reviews, commits, edits and lifecycle events; treat all as a cue to **read the channel**, not as automatically authoritative assignments. You cannot require a comments-only subscription. **No cron or polling.**

**Durable event cursor:** `team-state/coordinator-acting:state/cursor.json`. Record reviewed comment IDs and decisions here, never in a role's original state branch. This branch exists for the monitor's event deduplication only; the Coordinator's actual decisions still live on #145.

## When awakened — fail closed on authority

1. Fetch the exact pinned version of this file and **current** master `docs/team/OWNERSHIP.md`, `RULES.md`, `PROTOCOL.md`, `ROLES.md`, `COORDINATOR-FAILOVER.md`, `CLAUDE.md`, `PERFORMANCE-POLICY.md`, the latest Coordinator handoff, and #145.
2. Check #364 **merged**, and a current #145 `FROM: coordinator`, `TYPE: LOG`, `ACTOR: ChatGPT Work (acting)` activation backed by Louis. Check for a subsequent deactivation/return of control. If not activated, if Claude has resumed, or if another acting operator claims control: **no coordination writes**. Stop. Do not silently activate yourself on a generic GitHub event.
3. Read `state/cursor.json` and find new substantive #145 messages since `last_examined_comment_id`. Ignore routine lifecycle events and comments emitted by this monitor itself (`ACTOR: ChatGPT Work (acting monitor)`) after marking their IDs observed. Comments authored by `lmmx` are not proof of identity; apply the live protocol. If no new actionable external messages, stop without a post.
4. Audit the existing role routing using `coord.py audit` from the current checkout. Reconcile already queued/finished items, and use `coord.py route` plus `verify` for any actual assignments. If you cannot run the router, cannot read the cursor, or lack a supported safe method to write state, **do not dispatch**; report a specific blocker once, not a repeated failure loop.
5. For an actionable new message, record an **in-progress receipt** in your cursor and check whether an equivalent action is already recorded on #145 or an inbox. Then perform only the authorised, idempotent action. Record final disposition/cited links in the cursor **before** publishing a single grouped #145 LOG if substantive action was taken. Never post receipt-only status updates that would trigger your own task needlessly.
6. On any retry, use the saved receipt and links to avoid duplicate PR comments, dispatches, approvals or releases. If concurrent executions overlap, no speculative write: stop and reconcile. Use full commit SHAs, verified CI status and pre-registered measurement rules; new unsafe code needs independent B5 review.

## Delegation and safety scope

The active Acting Coordinator can dispatch one bounded **Codex** task through #363, but `coord.py verify` is *not* evidence that Codex was started: manual launch remains necessary. MONODROMY owns mathematics, Codex implements, and an independent reviewer owns B5 challenge. Do not assign the reviewer role to the same author of new unsafe code.

**Automated monitor action is limited to read, triage, audit, non-destructive routing and evidence-linked LOGs.** A GitHub event by itself does not justify merging, publishing a release, running the i9, changing a default, raising a thread cap, changing policy, or creating spending. Such actions require a separate deliberate Coordinator run with all current gates satisfied and any owner approvals as required. If no operator is available, leave a clear proposed decision for Louis or the returning Coordinator; the monitor must not simulate execution.

## Return of control, loop protection and quota

When Claude takes over, the owner/acting operator posts a return-of-control LOG on #145 and **disables the persistent Acting Coordinator Work task** in Work. Until disabled, the task must remain silent on all subsequent events because the activation guard fails. Do not assume disabling happened without a verified read-back.

Self-generated LOGs on #145 are delivered as further GitHub events. These must be **no-op wakes**, because their source IDs have already been handled or are recognised as the monitor's own. Never create a comment simply to acknowledge that a comment arrived.

Work usage is finite. At most one monitor task should subscribe as the acting executor, and no duplicate monitor tasks should be created. If the task platform cannot guarantee cursor writes or appropriate execution scope, report the limitation and fall back to an owner-invoked Work session, rather than a false claim of autonomous monitoring.

## References

- [Acting Coordinator failover procedure](COORDINATOR-FAILOVER.md)
- [Manual Acting Coordinator launch](WORK-ACTING-COORDINATOR.md)
- [Work-task creation templates](WORK-TASKS.md)
- [Current team dispatch protocol](PROTOCOL.md)

This specific event task has **not** been tested. A new persistent Work task must be created and its saved trigger, instruction text and enabled status read back before describing it as active.
