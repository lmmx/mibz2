# Standing policies and research rules (`docs/team/`)

Live work in `mibz2` runs through [`frontier/`](../../frontier/README.md), with the hard
invariants in [`CLAUDE.md`](../../CLAUDE.md) (and [`AGENTS.md`](../../AGENTS.md) /
[`GEMINI.md`](../../GEMINI.md)).

This directory holds the standing policies and research guidelines that remain in force:

| file | what it covers |
|---|---|
| [RULES.md](RULES.md) | standing authority, evidence, safety and owner-reserved rules (RULES D) |
| [OWNERSHIP.md](OWNERSHIP.md) | engineering ownership: shipping, non-negotiable safety and correctness |
| [PERFORMANCE-POLICY.md](PERFORMANCE-POLICY.md) | performance and memory acceptance rules, statistical tests and decision quality |
| [CI-INCIDENTS.md](CI-INCIDENTS.md) | classifying, recovering from and recording red CI checks |
| [EXTERNAL-DIAGRAMS.md](EXTERNAL-DIAGRAMS.md) | describing outside-model diagrams without committing screenshots |
| [ACQUISITION-FIRST.md](ACQUISITION-FIRST.md) | costed derivation from decoder entry state (with [`acquisition/`](acquisition/primitives.toml)) |
| [DELIVERED.md](DELIVERED.md) | summary table of shipped performance changes |
| [`research-charter/`](research-charter/METHOD.md) | research method, obligation register (`FRONTIER.md`) and proof programme |

**Related live documents:**
- **Release runbook:** [`docs/RELEASING.md`](../RELEASING.md)
- **Stewardship:** [`docs/STEWARDSHIP.md`](../STEWARDSHIP.md)
- **Exact-head merge check:** `.github/coordination/coord.py head-sha` (its coordinator-era routing
  subcommands are retired)

**History:** The coordinator-era role specs, `MODE.md`, `PROTOCOL.md`, `ROLES.md`, failover docs
and role inboxes were retired on 26 November 2026 (`docs/journal/2026-11-26-rhizome/`); their
frozen copies are preserved in the historical archive beside this directory. The earlier compact30
channel record remains frozen in `docs/journal/compact30-channel/`.
