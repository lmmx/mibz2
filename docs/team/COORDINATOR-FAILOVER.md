# Coordinator continuity — owner-authorised failover

**Owner direction, 10 November 2026:** Louis asked for a complete Codex relay setup and that the Coordinator **not be a single point of failure**. This procedure is proposed by the accompanying PR; it becomes operative after that PR is merged. A document does not create or awaken a Work task or Codex session.

## A role, not a single session

The Coordinator's job (queue, decisions, merges and releases) is a durable responsibility in GitHub, not a property of one Claude Code chat. Claude Code is its primary executor. When unavailable, Louis can activate **ChatGPT Work as the acting executor of the same role**; if that is unavailable, he can expressly appoint another capable session for a bounded period. This is a succession, **not** a second parallel Coordinator.

The acting operator inherits only the Coordinator's existing delegated authority. It cannot change the role's mandate, widen team authority, override an independent B5 safety objection, run Louis's i9 without his authorised recipe, spend, or alter the scientific programme. MONODROMY retains mathematical ownership; Codex is the scoped implementation delegate. Any reviewer of new unsafe code must be independent of the code's author.

## Activation / collision avoidance

Louis activates a named operator explicitly. Before dispatching, the acting operator reads master policies, all #145 updates since the last handoff, and the most recent active-operator LOG. It must post one #145 activation LOG identifying **ACTOR: ChatGPT Work (acting)**, the owner's authorization, time, scope and current master SHA. The existing routing tool recognises the *role* `FROM: coordinator`; include `ACTOR:` in the body of every acting LOG and assignment to distinguish which session operated.

Only **one operator** may dispatch or merge at a time. A recovered Claude Coordinator must read the acting handoff and explicitly take control back on #145 before doing work. On uncertainty or simultaneous operators: stop writes and reconcile with Louis. All prior assignments, cursor IDs, PRs and run digests survive that handoff unchanged. GitHub account names and message headers are **not** authentication.

## First session checklist

1. Read current `OWNERSHIP.md`, `RULES.md`, `PROTOCOL.md`, `ROLES.md`, `MODE.md`, `PERFORMANCE-POLICY.md`, `CLAUDE.md`, `docs/RELEASING.md` and `docs/journal/2026-11-10-coordinator-handoff.md`; compare with later master commits.
2. Read #145 from the last Coordinator LOG forward; inspect #361, #362, issue #363, any new cells-kernel PR, and the `evidence` branch. Mark **done / pending / blocked / needs Louis** separately.
3. Run `python3 .github/coordination/coord.py audit`; use `route --to <role> --type CHECKPOINT` and `verify` when delegating, observing one item in flight. After this setup merges, Codex's manual inbox #363 is routable but does **not** automatically start a Codex session.
4. Record an activation LOG, then decide/dispatch only within the delegated existing policy. Prefer Codex's off-by-default cells implementation and independent B5 review; do not conflate the i9's 18% inversion-stage win with a whole-decoder gain.

## Safety and decisions

- New unsafe code requires a written safety case, differential and forced-route fuzz tests, Miri, mutation checks and **independent adversarial B5 review**. Neither its author nor a merely administrative Work session may self-certify.
- A merge requires the exact 40-character head SHA, successful Actions and legacy commit statuses, a supported expected-head check, and post-merge verification (`CLAUDE.md`). If the available connector cannot enforce the expected-head invariant, defer to a capable CLI/operator. Do not convert a read-only reviewer into merger.
- Do not edit frozen test gates after seeing data. If review or performance is adverse, log the reason and preserve the partial PR. The cells rollout stays **two PRs**: off-by-default implementation/third benchmark arm, then a separately justified default flip.
- Only Louis can authorise extra spending, infrastructure, role/rule expansion, or a workstation run outside current standing authority.
- No actor can honestly report a Work task or webhook as enabled until the platform confirms its trigger and enabled state. Without a trigger the process is manual, not background monitoring.

## Leaving the chair

Before stopping, post a `FROM: coordinator`, `TYPE: LOG` with `ACTOR:`, exact SHAs, evidence links/digests, decisions, unprocessed queue IDs and next actor. Hand back explicitly on #145. An agent returning after a quota pause **must reconcile**, not resume from its own old chat.

**Fallback of the fallback:** Louis can start a fresh operator with [WORK-ACTING-COORDINATOR.md](WORK-ACTING-COORDINATOR.md) and the latest GitHub evidence. No model-specific memory or provider subscription is indispensable. Codex's task remains separately documented in [CODEX-PILOT.md](CODEX-PILOT.md).
