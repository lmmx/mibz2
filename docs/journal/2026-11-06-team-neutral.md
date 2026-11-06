# Making the team independent of compact30

2026-11-06, at Louis's direction. He pointed out that the Oracle had been named "the compact30
Oracle", although compact30 had been set aside as a research direction. The swarm was still
holding on to it in its built-in structures, while letting go of it in its deliberate ones.

## What was inherited

Searching for the name finds only the surface. This audit asks of each structure where its
identity, its authority, its instructions and its state came from.

| layer | what it was | effect |
|---|---|---|
| Names | `docs/journal/compact30-channel/`; tasks "Compact30 (W) …"; instruction texts "You are the compact30 AGENT"; the channel PR #145 titled "compact30 research channel"; `coord.py`'s "the compact30 channel" | New work copied the frame. The Oracle got the name because the template had it |
| Instructions | The three Work tasks load their specs from `research/compact30-constructor` at `0479b380`, not from master. PROTOCOL.md named that branch as canonical | The copies on master (#221) were a record that no agent ran from |
| State | Cursors and decision logs on `review/compact30-instruction-ledger`, `review/compact30-guardian-shadow` and `review/compact30-adversarial-scanner-146` | The live team's place in its own queues was kept on an abandoned project's branches |
| **Authority** | PROTOCOL.md and GUARDIAN.md took every rule from the compact30 research charter: §2, §6.4, §8.2, §12 and §13. The Guardian was "ACTIVE, stage A" under it | The team's governance was an abandoned campaign's. The Guardian could check compliance with the charter without anyone asking whether the charter should still govern |
| Role framing | The Optimiser's mission was "reduce dynamically executed instructions", with the instruction ledger as the canonical gate (charter §5) | The binding cost today is scaling and contention on the i9, not instruction count. MODE.md had already corrected this; ROLES.md had not |
| Rules that no longer match practice | Charter §2 and §12.3 forbid i9 or workstation timing as evidence, yet #219 and #220 rely on it. §12.3 reserves bench-method changes, yet the harness is versioned through routine PRs | The written rules and the actual practice had silently diverged |
| Status contradictions | GUARDIAN.md said "PROPOSED, not active" while SPEC.GUARDIAN.md said "ACTIVE, stage A". GUARDIAN.md §4 reserved merges to the user while the charter delegated them | Inherited text no longer agreed with itself |
| Lapsed duties | PROTOCOL §4.1 required every message to be transcribed verbatim into `log.md`. The last entry is from 2026-11-05 | A duty stopped without anyone deciding to stop it |
| Procedure for a finished stage | Charter §13 activation stages, PROTOCOL §7's end-to-end test, GUARDIAN.md's shadow pilot | Bootstrapping steps, long passed, still framed the live specs |

MODE.md itself inherits nothing: it never mentions compact30, and its objective is the
decoder's.

## What changes

This is three separations, following Louis's distinction:

1. **The machinery is live, on master, in `docs/team/`:**
   - `MODE.md` (byte-identical) and `MODE.RATIONALE.md`;
   - `RULES.md`;
   - `PROTOCOL.md`, `ROLES.md`, and `SPEC.md` with each role's `SPEC.<ROLE>.md`.

   It is written from the current objective. Every Work task loads from here.
2. **The history is archived:** `docs/journal/compact30-channel/` is restored to exactly what
   #221 copied, and frozen. The compact30 branches stay as they are.
3. **The governance is restated deliberately, not inherited.** `RULES.md` lists each charter
   rule with:
   - its purpose;
   - whether it was specific to compact30;
   - a proposal: retain, adapt or retire.

   Rows that change substance are marked ⚑. **Louis approves them.** Until he does, the
   charter's text applies, read as covering the whole team.

The evolving records are separated as well:
- **Each role's state** moves to a neutral branch, `team-state/<role>`. It is created as an
  orphan with its cursor and records copied under `state/`, and `migrated_from` records the
  previous branch's commit.
- **The Oracle's account** goes on its own orphan branch, `observatory/intention`, deployed
  against the repository rather than within it. It has no CI and no PR per update, and its
  history is the record of how intention changed.
- **The channel #145 is the log.** Transcribing it is retired.

**Kept:**
- the PR numbers: #145 and the inboxes #160, #161, #162 and #222, so no subscription changes;
- the routing rules and `coord.py`'s checks;
- every earlier branch, at the commits recorded at cutover.

**#220's frozen registration is untouched.**

## Cutover

1. Review: the Guardian (authority and routing) and the Researcher (inherited premises, and
   this audit's completeness).
2. Louis approves the ⚑ rows of `RULES.md`, or amends them.
3. With no assignment in flight, create the four `team-state/<role>` branches and
   `observatory/intention` from the old branches' heads, recording each head.
4. Merge, checking the head before and after.
5. Louis loads the specs with SPEC.md's command at the merged commit: three renames and re-pins,
   and the Oracle's creation.
6. Dispatch resumes. The Oracle's first assignment is to rebuild its record. Its first test is
   whether its account would have caught this incident.
