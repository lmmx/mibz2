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
| `SPEC.<ROLE>.md` | each Work role's own spec: OPTIMIZER, GUARDIAN, RESEARCHER, ORACLE |

**Where the rest lives:**
- **Each role's state:** `team-state/<role>`, a continuation of its old branch, holding its
  cursor and records. These branches have no PR, and the workflows' filters run no CI on them.
- **The Oracle's record of what the team is doing and why:** `observatory/intention`, an orphan
  branch with its own history.
- **Routing:** `.github/coordination/coord.py`.
- **History:** the team began as the compact30 research channel. That record is frozen in
  `docs/journal/compact30-channel/` and on the compact30 branches. How it was separated:
  `docs/journal/2026-11-06-team-neutral.md`.
