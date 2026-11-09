# The mibz2 agent team

The live machinery of the agent team that works on mibz2. Its objective is STEWARDSHIP.md's: a
measurably faster, correct production decoder, properly released. MODE.md says the same.

| file | what it is |
|---|---|
| [MODE.md](MODE.md) | the register every role writes in; read first on every run. Changed only by Louis ([why](MODE.RATIONALE.md)) |
| [RULES.md](RULES.md) | who may decide what: the team's standing rules, with STEWARDSHIP.md (`docs/STEWARDSHIP.md`) |
| [PROTOCOL.md](PROTOCOL.md) | how messages move: the channel #145, the inboxes, dispatch and format |
| [ROLES.md](ROLES.md) | the five roles, and what each does and doesn't do |
| [SPEC.md](SPEC.md) | what every Work task's spec has in common, and the command that loads them |
| [WORK-TASKS.md](WORK-TASKS.md) | the only prompts for creating or re-pinning a ChatGPT Work task, each marked known to work or untested |
| `SPEC.<ROLE>.md` | each Work role's own spec: OPTIMIZER, GUARDIAN, RESEARCHER, OBSERVER |
| [SPEC.PATHFINDER.md](SPEC.PATHFINDER.md) | the Pathfinder's spec: a proposed role, not yet in force (ROLES.md) |

**Where the rest lives:**
- **Each role's state:** `team-state/<role>`, a continuation of its old branch, holding its
  cursor and records. These branches have no PR, and the workflows' filters run no CI on them.
- **The Observer's record of what the team is doing and why:** `observatory/intention`, an orphan
  branch with its own history.
- **Routing:** `.github/coordination/coord.py`.
- **Louis's requests from ChatGPT:** PROTOCOL.md §7 is the rule; `CHATGPT.md` at the
  repository's root is the procedure a ChatGPT conversation follows to send one.
- **History:** the team began as the compact30 research channel. That record is frozen in
  `docs/journal/compact30-channel/` and on the compact30 branches. How it was separated:
  `docs/journal/2026-11-06-team-neutral.md`.
