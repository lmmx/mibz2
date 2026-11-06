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
| Rules that no longer match practice | Charter §2 and §12.3 (and STEWARDSHIP's reserved list) forbid i9 or workstation timing as evidence, yet #219 and #220 rely on it. §12.3 reserves bench-method changes, yet the harness is versioned through routine PRs | The written rules and the actual practice had silently diverged |
| Status contradictions | GUARDIAN.md said "PROPOSED, not active" while SPEC.GUARDIAN.md said "ACTIVE, stage A". GUARDIAN.md §4 reserved merges to the user while the charter delegated them | Inherited text no longer agreed with itself |
| Lapsed duties | PROTOCOL §4.1 required every message to be transcribed verbatim into `log.md`. The last entry is from 2026-11-05 | A duty stopped without anyone deciding to stop it |
| Procedure for a finished stage | Charter §13 activation stages, PROTOCOL §7's end-to-end test, GUARDIAN.md's shadow pilot | Bootstrapping steps, long passed, still framed the live specs |

**This entry first said that MODE.md inherits nothing. That was wrong** (Researcher,
6685052784). MODE.md never names compact30, but it carries three inherited premises:
- "the charter governs what may be done";
- the Optimiser's "Work? Ir — not time";
- an objective narrower than STEWARDSHIP's, the inverse BWT against the whole decoder.

MODE.md is Louis's to change, so these are proposals to him (RULES.md, section M).

**The Researcher's audit found three more live premises:**
- **STEWARDSHIP.md.** It is permanent, and kept as A6, but it still named compact30 as the
  current campaign. Through its kernel-PR rule ("the whole count at or below master's") it
  re-imported the instruction gate that C4 retires. It also reserved the timing and bench
  changes that C5 and C6 adapt. It is reconciled in RULES.md section S, with the edits in this
  PR.
- **CAMPAIGN.md's further limits:**
  - whole-instruction parity before integration;
  - opportunities ranked by instructions;
  - full kernel replacement without a feature flag;
  - one fixed workload and thread regime.

  These are archived as historical constraints. None carries forward.
- **Historical framing still in circulation:**
  - the research portfolio register (P87 still "open"; the four-worker objective);
  - the agent-team blog post (instruction-led optimisation; the charter; an isolated reviewer).

  These are records of their time, not today's queue. A follow-up adds a short dated pointer
  from each to the live machinery, without rewriting their history.

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
- **Each role's state** moves to a neutral **continuation** branch, `team-state/<role>`:
  - branched from the old branch's final head, so the history, blame and links survive;
  - then one commit moves the live state under `state/` and removes the unrelated tracked
    files, so the current tree is neutral.

  `migrated_from` records the old head. A manifest in that commit lists every processed ID,
  offset, entry in progress, complete or finished, every REVIEW link and every open HOLD.
- **The Oracle's account** goes on its own orphan branch, `observatory/intention`, deployed
  against the repository rather than within it. Its first commit links its predecessor,
  `review/oracle-intention`, at that branch's final head.
  - It has no PR per update, and its history is the record of how intention changed.
  - **It also has no CI, but by the workflows' filters, not because it is an orphan.** On
    push, they run only for `master`, `main` and `py-*` tags, and `pull_request` runs need a
    PR. That was checked in `.github/workflows/*.yml` at this PR.
- **The channel #145 is the log.** Transcribing it is retired.

**Kept:**
- the PR numbers: #145 and the inboxes #160, #161, #162 and #222, so no subscription changes;
- the routing rules and `coord.py`'s checks;
- every earlier branch, at the commits recorded at cutover.

**#220's frozen registration is untouched.**

## Cutover

Only after Louis approves and re-pins (Guardian D033):

1. **Review.** The Guardian checks authority and routing. The Researcher checks inherited
   premises and this audit's completeness.
2. **Approval.** Louis approves the whole RULES.md table, and decides on MODE.md's proposals.
3. **Drain.** Hold dispatch, and wait until every role's current comment is *finished*: its
   REVIEW published, and its final cursor commit made.
4. **Copy and verify** at each old branch's exact head:
   - create `team-state/<role>` as a continuation of that head;
   - commit the state under `state/`, with a manifest. It lists every processed ID, the
     offset, every entry in progress, complete or finished, every REVIEW link and every open
     HOLD;
   - set `migrated_from` to that head's SHA and the manifest's digest;
   - create `observatory/intention` from the seed, linking `review/oracle-intention`'s final
     head.
5. **Merge,** checking the head before and after.
6. **Re-pin.** Louis loads the specs with SPEC.md's command at the merged commit: three tasks
   edited in place, and the Oracle created. All tasks stay enabled.
7. **Re-check before dispatch resumes:**
   - each old branch's head still equals `migrated_from`;
   - each task's saved instruction text carries the new pin;
   - each cursor reads back from its new path.

   Any mismatch fails closed: the role reports it and stops (SPEC.md, "Always"). Don't replay
   assignments as a test.
8. **Dispatch resumes.** The Oracle's first assignment is to rebuild its record. Its first test
   is whether its account would have caught this incident, from comments 6643481606 and
   6643744616 onwards.
