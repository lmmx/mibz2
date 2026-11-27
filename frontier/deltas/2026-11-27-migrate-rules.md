claim: migrate-rules
by: antigravity
outcome: partial
evidence: docs/team/RULES.md, frontier/claims/migrate-rules.toml, docs/journal/decisions.md, #977, #995

**What was asked:** Migrate `docs/team/RULES.md` off coordinator references by meaning for the frontier while preserving A1, A4, RULES D, and #995's O1–O2, and fixing broken archived links.
**What I found / did:**
- Acquired `migrate-rules` atomically after the expired `hold/archive-team-strata` lease cleared.
- Audited every row of `docs/team/RULES.md` against `frontier/README.md`, `CLAUDE.md`, `docs/team/OWNERSHIP.md`, `docs/journal/decisions.md` (2026-11-26–27), and the #977 archive delta.
- Rewrote all live Coordinator, Guardian, Researcher and #145-channel authority references by meaning:
  - Coordinator authorisation → frozen registration / claim `stop_test` committed before data plus independent review by a different agent where required (B5, C6, O2);
  - Guardian safety HOLD / B6 immediate stop → scoped flag in `frontier/flags/`, holding only its named scope until a different agent from the flagged work's author resolves it;
  - Coordinator decision / LOG → acting agent's append-only delta in `frontier/deltas/` (with authority/policy changes committed verbatim in `docs/journal/decisions.md`, E2);
  - C1a non-adoption pre-data rule → claim `stop_test` or committed registration file before data (#145 retained solely as workstation evidence noticeboard for `just submit`, C5).
- Preserved A1 and A4 (shared knowledge is not shared authority; direct authenticated owner session + verbatim `docs/journal/decisions.md` record replaces "the Coordinator's session"), all of RULES D (items 1–8), B1–B7, C2, and #995's O1–O2 verbatim; mapped O3's role names to scoped flags and independent reviewers.
- Repaired broken relative links (`PROTOCOL.md` → `archive/PROTOCOL.md`, `MODE.md` → `archive/MODE.md`); verified all relative links in `docs/team/RULES.md` resolve.
- Added a dated `PROPOSED` entry under `## Amendments` (awaiting Louis's approval under RULES D1) and populated `needs_louis` in `frontier/claims/migrate-rules.toml` without altering `stop_test`.
**What changed:** `docs/team/RULES.md`, `frontier/claims/migrate-rules.toml`, `frontier/deltas/2026-11-27-migrate-rules.md`.
**What's next:**
- Independent cold review by a different agent (per `done_when` and RULES O2: substantive change to authority/governance rules), then Louis's approval under RULES D1 before merge (do not merge without his approval).
- Separate follow-up (outside `migrate-rules`'s `owned_paths`): `docs/team/OWNERSHIP.md` still links to `COORDINATOR-FAILOVER.md` (now `archive/COORDINATOR-FAILOVER.md`) and is owner-only (L127).
**Caveats:** Rule change under RULES D1 — prospective only until Louis approves and merges.
