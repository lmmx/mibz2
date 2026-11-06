# mibz2

Publishing a page of the site (a blog post above all): use the `publish-page` skill
(`.claude/skills/publish-page/SKILL.md`).

The team channel (#145) and the role inboxes (#160 Optimiser, #161 Guardian, #162
Researcher, #222 Oracle): route every message with `.github/coordination/coord.py`, never from memory.
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

## Releases: the coordinator drives them

The maintainer's standing instruction is that the coordinator releases mibz2 itself,
without asking first, whenever merged work is worth releasing. Nobody needs to prompt it.

- **Worth releasing:**
  - A performance gain the CI bench has shown under its PR's registered rule.
  - Or a change that is definitely committed to: a fix users need, or an API or packaging
    change.

  Docs, journal, bench-harness or blog changes alone are not. Look at what has merged since
  the last tag (`git log <last tag>..master -- src Cargo.toml`) after every merge to
  master, not only when asked.
- **When:** while the maintainer is around, not overnight or when they are away from the
  keyboard. A broken release needs someone to hear of it. If work becomes releasable while
  they are away, release it when they are next around, and say so.
- **Before:**
  - Master is green.
  - One paired bench run of the last tag against master: a `workflow_dispatch` of the
    Benchmark workflow on master with `base=<the tag's commit SHA>` (the SHA, never the
    tag name). It is a check that nothing got worse, not a gate to re-run until the
    result looks good. Name its CPU.
- **The release:**
  - Dispatch Deploy Release with `bump`: patch, or minor for a public API change. It
    bumps the crate and the Python package together, tags `py-X.Y.Z` and starts CI.yml,
    which publishes the crate (release-plz, Trusted Publishing) and the PyPI wheels.
  - Then a CHANGELOG PR with the release's notes: what changed, the paired run's table
    and its CPU, the stages, the cost, and the caveats. #210 is the model.
- **After:**
  - Verify crates.io and PyPI at the new version, with every wheel and the sdist.
  - Verify docs.rs.
  - Fix any publishing failure (as #209 did) with a follow-up patch release.
  - Then bump mibz2 in lmmx/wikidata-pq's `reshape/Cargo.toml` in a PR. It merges under
    that repository's bench rules.
- **Report** the release to the maintainer in one short message, and in the channel log.

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
  nothing else.

`bench-team-size` and `bench-release-scaling` follow it; give any older recipe the same
shape when it is next used.

## The agent team

The team's live machinery is `docs/team/`:
- MODE.md: the register. Reload it at session start and when asked.
- RULES.md: who may decide what.
- PROTOCOL.md: how messages move.
- ROLES.md and the specs.

**Work task commands** (creating or editing a ChatGPT Work task) always name each task's model
and effort, to set or to leave as set (`docs/team/ROLES.md` lists them). A command without them
is incomplete. They go beside the prompt, for Louis to select in the Work task chat window's
model setting; never inside the prompt.
- **Creating a task:** one task per creation prompt, in `docs/team/SPEC.md`'s form.
- **Editing existing tasks:** goes in a separate prompt.

Route every message with `.github/coordination/coord.py`, as above. compact30 is history
(`docs/journal/compact30-channel/`, frozen): it is no source of the team's identity, authority
or state.

The Oracle (#222) keeps a behind-the-scenes account of what the team is doing and why, on the
orphan branch `observatory/intention`.
- **Dispatching to it:** send it a CHECKPOINT whenever the record moves materially (a result,
  a decision, a merge, a change of direction), one at a time.
- **Correcting it:** correct only major discrepancies, through a CORRECTION, never on wording.
