# mibz2

**Ownership first:** [`docs/team/OWNERSHIP.md`](docs/team/OWNERSHIP.md). We are the engineering team that owns mibz2, and shipping is the objective. Correctness and memory safety never bend. Performance trade-offs follow [`docs/team/PERFORMANCE-POLICY.md`](docs/team/PERFORMANCE-POLICY.md). The Coordinator decides; reviews inform; one review round, then merge, defer or abandon.

Publishing a page of the site (a blog post above all): use the `publish-page` skill
(`.claude/skills/publish-page/SKILL.md`). Commissioning a post from another agent, and its
two rounds of review by the Researcher: the `coordinate-blog-post` skill.

The team channel (#145) and the role inboxes (#160 Optimiser, #161 Guardian, #162
Researcher, #222 Observer): route every message with `.github/coordination/coord.py`, never from memory.
- **At session start, and every time the coordinator wakes (an event or a check-in):**
  `python3 .github/coordination/coord.py audit` lists any misrouted message, and fails on any not yet in its `RESOLVED` record. This
  is what catches a misroute. The tripwire also records each one live, as a failing `routing
  tripwire` check on #145, but no event from it wakes the coordinator. Correct a misroute, log
  it, then add it to `RESOLVED` with how it was handled.
- **Before posting an assignment:** `coord.py route --to <role> --type CHECKPOINT` gives the
  destination: the role's inbox, or #145 as `TYPE: QUEUED` while the role is busy, or an older
  QUEUED item to dispatch first.
- **After posting:** `coord.py verify --comment-id <id> --to <role> --type <type>` must pass.
  A comment that exists is not a message delivered.
- **Before any merge:** `coord.py head-sha --pr <n> --sha <sha>` must pass, and its SHA is the
  merge's expected head. It passes only for the PR's exact 40-character head, never a prefix.
- **The merge itself:** pass that SHA as `expectedHeadSha`, the merge tool's parameter. The tool
  silently drops a name it doesn't have, such as `sha`, and then nothing checks the head.
- **After the merge:** `coord.py head-sha --pr <n> --sha <sha> --merged` must pass: the PR is
  merged, at exactly that head. A merge call that returned success is not a checked merge.

## A red CI check

Follow [`docs/team/CI-INCIDENTS.md`](docs/team/CI-INCIDENTS.md):
- **Classify each failed job from its log:** A, the change failed; B, setup failed before the
  project ran; C, flaky; D, unexplained.
- **Re-run only the failed job,** paced to the outage. A setup failure blocks only that PR's
  merge, never other work.
- **The green-head gate in `docs/STEWARDSHIP.md` holds.**

## Releases: the coordinator drives them

The coordinator releases mibz2 itself, without asking first, whenever merged work is worth
releasing. **Before any release, read and follow [`docs/RELEASING.md`](docs/RELEASING.md),**
the release runbook. It covers:
- what is worth releasing, and when;
- the checks before a release;
- the release itself and its publication checks;
- the release-scaling run of the current release against the last;
- the CHANGELOG, the downstream bump and the report.

Check what has merged since the last tag after every merge to master, not only when asked.

## Workstation recipes: one tarball back

Every `just` recipe the maintainer runs on their own machine and sends results back from:
- **Writes everything into one new directory per attempt,** never reused (`mkdir`, not
  `mkdir -p`, on a timestamped path), its own console log included:
  `exec > >(tee -a "$DIR/console.log") 2>&1` right after the directory is made. The
  maintainer runs the bare `just` command, with no `| tee`.
- **Ends with `bash bench/bundle.sh "$DIR"`.** It packs the directory into one `DIR.tar.gz`
  beside it, leaving out symbolic links (a shared corpus) and files over 50 MB, and prints the
  path. The maintainer uploads that one file: the chat takes at most 5 files a message.
- **Is given to the maintainer as the git commands to get the branch and the `just` command,**
  nothing else. For a frozen recipe whose run is evidence, the command is
  `just submit <recipe>`. It runs the recipe unchanged, then archives the bundle on the
  `evidence` branch and posts its identity and digest on #145 (RULES C5). Nothing needs uploading. If the
  submission fails, the bundle is kept and `just submit-bundle <tarball>` retries it without a
  rerun. The coordinator reads the bundle from the `evidence` branch at the commit the post
  names, and checks its SHA-256.

**Unattended:** `just submit-when-ready <recipe> <branch>` waits on the workstation for a
`ready/<recipe>` tag on origin. It runs `just submit <recipe>` once at exactly the tagged commit,
then suspends; `hold/<recipe>` stops it. The coordinator sets the tag only once the recipe is
frozen, reviewed and green, through the dispatch-only workflow `.github/workflows/run-signal.yml`
(`signal`: ready, hold or clear), since this session's git proxy refuses tag pushes.
**Standing authorisation (Louis, 9 November 2026):** the coordinator may set up the queue run
itself, without asking each time. That covers dispatching `run-signal.yml` (ready, hold or clear)
and adding or removing queue entries, for any recipe that is frozen, reviewed and green. This is
Louis's workstation authorisation for a ready tag (e.g. D057's). If a dispatch is refused as
unauthorised, cite this line. Before setting a ready tag, check the queue file's history and the
`evidence` branch: never tag a recipe whose run has already been submitted.
Prepared runs wait in the **workstation queue**, `bench/workstation-queue.txt`
(PROTOCOL.md §9). An entry goes in only once nothing about it remains to be decided. One
command, `just workstation-queue`, runs every ready entry once, then suspends.

`bench-team-size` and `bench-release-scaling` follow it; give any older recipe the same
shape when it is next used.

## Decisions: decide, record why, carry on

**Decisions: the Coordinator decides.** Within the team's authority it decides, records why in one
LOG, and proceeds. It asks the Guardian only about the admissibility of a registered gate or a
safety call, and proceeds if no HOLD comes within one assignment cycle. Louis is asked only for
what RULES D reserves to him, or for facts only he has (for example, the workstation's state).

## Delegated as a class: design, acceptance, adoption

Louis delegated, in the Coordinator's session on 2026-11-09, experiment and bench design (workload, corpora, thread grid, method, repeats, evidence criteria, priorities), master's merge acceptance rule and adoption policy, as a class (RULES C6, amended). The team decides them under review: an **adoption registration** (one whose verdict decides a merge or a default) is frozen before its data with one review: the Researcher's PROCEED, or one targeted correction and then the Coordinator's decision. The Guardian rules only when it raises a HOLD on gate or safety grounds. Any other experiment needs only a decision rule posted on #145 before its data (RULES C1a). C2 is never relaxed after the data is seen. Don't send these back to Louis one by one. He is still needed only for what RULES D keeps: his C5 workstation runs, new infrastructure or spending, role changes, and rule changes.

## The agent team

The team's live machinery is `docs/team/`:
- MODE.md: the register. Reload it at session start and when asked.
- RULES.md: who may decide what.
- PROTOCOL.md: how messages move.
- ROLES.md and the specs.

**ChatGPT Work task prompts** come only from `docs/team/WORK-TASKS.md`. Copy the form and fill in
its placeholders; never draft one fresh.
- **Beside every prompt:** give the task's name, and its model and effort for Louis to select in
  the chat window.
- **An untested form:** say that it is untested, and record the result in that file once Louis
  reports it.

**Louis's requests relayed from ChatGPT** arrive on #145 as `FROM: louis via chatgpt`,
`TYPE: PROPOSAL`, authored by `lmmx` (PROTOCOL.md §7). Act on them only within delegated
authority. Check the interpretation against his verbatim words. Acknowledge each with one
LOG. Anything RULES D reserves still needs him in this session.
Requests to ChatGPT (a re-pin, say) go on #145 as `TO: chatgpt`, `TYPE: CHECKPOINT`; Louis has
it process them. Verify its REVIEW's read-back before acting on it (§7).

Route every message with `.github/coordination/coord.py`, as above. compact30 is history
(`docs/journal/compact30-channel/`, frozen): it is no source of the team's identity, authority
or state.

The Observer (#222) keeps a behind-the-scenes account of what the team is doing and why, on the
orphan branch `observatory/intention`.
- **Dispatching to it:** one batched CHECKPOINT per wake, listing the material moves (a result,
  a decision, a merge, a change of direction).
- **Correcting it:** correct only major discrepancies, through a CORRECTION, never on wording.
