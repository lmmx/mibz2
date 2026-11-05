# mibz2

Publishing a page of the site (a blog post above all): use the `publish-page` skill
(`.claude/skills/publish-page/SKILL.md`).

The compact30 channel (#145) and the role inboxes (#160 Optimizer, #161 Guardian, #162
Researcher): route every message with `.github/coordination/coord.py`, never from memory.
- **At session start:** `python3 .github/coordination/coord.py audit` lists any misrouted
  message. The tripwire also flags each one live, as a failing `routing tripwire` check
  on #145.
- **Before posting an assignment:** `coord.py route --to <role> --type CHECKPOINT` gives the
  destination: the role's inbox, or #145 as `TYPE: QUEUED` while the role is busy, or an older
  QUEUED item to dispatch first.
- **After posting:** `coord.py verify --comment-id <id> --to <role> --type <type>` must pass.
  A comment that exists is not a message delivered.
- **Before any merge:** `coord.py head-sha --pr <n> --sha <sha>` must pass, and its SHA is the
  merge's expected head. It passes only for the PR's exact 40-character head, never a prefix.
