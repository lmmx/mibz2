# The agent team's protocol

How the coordinator and the three roles (Optimiser, Researcher, Guardian) work together:
the register (MODE.md), the message protocol (PROTOCOL.md), the roles (ROLES.md, SPEC.*.md,
GUARDIAN.md, REVIEWER.md), the campaign procedure (CAMPAIGN.md) and the channel's log
(log.md). The channel is PR #145; the inboxes are #160 (Optimiser), #161 (Guardian) and #162
(Researcher); `.github/coordination/coord.py` routes and verifies messages.

The team began on the compact30 research, which has since been set aside; the protocol is
the team's, not compact30's. It lived only on two research branches, and is kept here
byte-for-byte as they had it:

| from | commit | files |
|---|---|---|
| `research/compact30-channel` | `e52252f51419` | MODE.md, MODE.RATIONALE.md, MODE.ORIGIN.md, PROTOCOL.md, CAMPAIGN.md, REVIEWER.md, log.md, tools/mode_doctest.py; `../2026-11-04-compact30-mode.md` |
| `research/compact30-constructor` | `4109dccd512b` | GUARDIAN.md, ROLES.md, SPEC.md, SPEC.GUARDIAN.md, SPEC.OPTIMIZER.md, SPEC.RESEARCHER.md, review-queue.md, section-6.2-brief.md, section-6.2-index.md; `../2026-11-04-compact30-research-charter.md` |

- PROTOCOL.md and REVIEWER.md were identical on both branches.
- log.md: the constructor branch's 6 lines are the start of the channel branch's 576; the
  channel branch's is kept.
- The research charter: the constructor branch's is the later (2026-11-04 21:58, 745 lines,
  against 11:03 and 497); it is kept.
- The compact30 evidence on the channel branch (`evidence/`, 1.5 MB) and everything else on
  both branches stay on those branches, at the commits above. Keep both branches; they are
  the full record.

MODE.md is changed only by Louis. The roles' own working files (cursors, decision logs)
stay on their review branches.
