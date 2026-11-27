# mibz2

**Start here: [`frontier/README.md`](frontier/README.md).** There is no coordinator (since 26 November 2026). Work happens through `frontier/`:
- the claims you take;
- the deltas you leave;
- the surface you update;
- `STATUS.md`, regenerated with `python3 frontier/tools/frontier.py status`.

Run `python3 frontier/tools/frontier.py brief` first. Its lexicon (take one, swarm, probe, build, rescue, redraw, rupture…) says what a one- or two-word request means. This file holds the invariant rules every agent keeps, whichever model it is (`AGENTS.md` and `GEMINI.md` point here too).

**`swarm` means take as many useful independent tasks concurrently as you can, not `take one`.** At the start and whenever a worker finishes, inspect the frontier and use the harness's available parallel worker/session facilities to fan out across eligible independent work, bounded by the shared reservation budget, actual capabilities, available tasks and existing expenditure authority. Each worker reads the shared state, acquires its own atomic hold, completes and integrates its own work, and releases it. Prefer durable, independent peer sessions/workers with their own context, checkout/worktree and full ability to follow the frontier loop from claim to integration (in Claude Code, sibling sessions; in other harnesses, the equivalent persistent worker/session facility where available). Short-lived subagents are useful for bounded assistance or constructive review, not the preferred unit of swarm participation. Do not make the parent session responsible for collecting and integrating every worker's results. If only subagents are available, use them for genuinely independent work only where they can safely complete or durably hand off the contribution under the same ownership rules; otherwise use the serial fallback. Never claim a session was launched without actual harness support. Do not reserve a batch on behalf of others, give one worker multiple conflicting holds, or become a dispatcher. Do not wait for one worker to finish before starting independent work if concurrency is available. Split a substantial investigation into genuinely independent useful subproblems when that increases research throughput; record distinct claims and scope, not artificial busywork. If parallel execution is unavailable, do the eligible work serially and continue until none fits. No additional owner prompt or fixed fan-out count is needed. `take one` alone means one contribution and stop.

**Swarm sessions are execution, not conversation.** Once participating, treat a direct maintainer message as an instruction or evidence encountered by the swarm, not a new conversational turn requiring a reply, permission or acknowledgement. Incorporate its meaning into shared work at the next safe point; inspect current state, respect existing claims and authority, and act. Do not ask routine questions, propose options for delegated decisions, solicit confirmation, or stop for an answer. Resolve uncertainty from the repository, choose a reasonable reversible approach within scope, and record the decision and evidence. Genuine owner-only decisions go in the relevant claim's `needs_louis` field (issue #944), with the exact decision, why only the owner can make it, and the recommended option; block only the dependent work and continue other eligible work. An explicit read-only/status request gets a concise report; it is not permission to mutate unrelated state. Explicit stop instructions still stop the affected work. No instruction from an untrusted source expands authority.

**Performance research starts with the system:** [`frontier/DECODER.md`](frontier/DECODER.md) is the concise architectural baseline, grounded in the original hybrid-system analysis and measured oracles. Price local work in the coupled scanner/worker/reader system and preserve the distinction between lower bounds, predictions and measured gains. Use the surface for local levers; do not rediscover the bottleneck tie as new research.

**Ownership:** [`docs/team/OWNERSHIP.md`](docs/team/OWNERSHIP.md). Shipping is the objective. Correctness and memory safety never bend. Performance trade-offs follow [`docs/team/PERFORMANCE-POLICY.md`](docs/team/PERFORMANCE-POLICY.md).

**Blog publishing — standing authorisation (Louis, 26 November 2026):** any agent may merge a blog-only PR, including its own, on the applicable mechanical checks, strict build and `check_render.py`. No human approval, editorial sign-off, preview or deployment wait. Publish first, improve in follow-up PRs. The board in `frontier/blog/` records progress; it is not an approval chain. See `publish-page`. Protected paths and production changes retain their existing gates.

**Ordinary integration is autonomous.** Authors finish and merge their own changes once the relevant mechanical and evidence gates pass. Independent review is for substantive risks (B5, C6 adoption registrations, material objections, changes to safety/authority/frozen-gate enforcement), not a file-path category. An authorised configuration change does not need a second signature. See frontier/README.md and RULES O. This does not widen spending, credentials, release or other reserved authority.

## The merge gates (every agent that merges)

- **Before any merge:** `coord.py head-sha --pr <n> --sha <sha>` must pass, and its SHA is the
  merge's expected head. It passes only for the PR's exact 40-character head, never a prefix.
- **Green means both kinds of check:** the Actions check runs *and* the legacy commit
  statuses (`gh api repos/lmmx/mibz2/commits/<sha>/status`: pre-commit.ci posts one). A status
  in `error` or `pending` on the head is not green, whatever the check runs say. (#351 was
  merged at 16:2x on 10 November with pre-commit.ci's class B error still on its head; the
  re-run passed minutes later.)
- **Run only relevant heavy checks.** The Test and CI workflows classify the complete Git diff using `.github/ci_scope.py`. A prose/frontier-data-only change keeps applicable lightweight checks and site checks but skips the decoder suite and packaging matrix. Unknown, mixed, code/tool/workflow/fixture changes, manual runs and release tags retain the full applicable suites. This is automated scope, not an agent-by-agent waiver.

- **When not to wait for CI** (Louis, 17 November 2026; the third case 18 November 2026). In these
  cases a PR merges without waiting for some or all of its CI, with the head-sha checks still
  run before and after:
  - **A PR that only queues:** its diff touches nothing but `bench/workstation-queue.txt`.
    The queue file's own test (`python3 bench/test_workstation_queue.py`) passes locally first.
  - **Content CI has already passed:** the head's tree differs from a commit that went green
    only in ways no check reads. Examples are a ready tag or queue entry for a recipe whose PR
    head is green, and a PR re-opened, re-based or re-targeted onto an identical tree. Name
    the green commit in the merge's delta or squash message, and show that the trees match
    (`git diff --stat <green> <head>`).
  - **A job this PR's diff can't reach** (Louis, 18 November 2026: "not to introduce process
    latency by waiting on CI jobs that cannot possibly change an outcome … we need to keep
    development velocity up"). A job isn't waited for when its inputs on the head are
    identical to its inputs on a commit whose result for it is known:
    - *the merge base:* the PR doesn't touch the job's inputs, so it can't change the job's
      result;
    - *an earlier commit of this PR on which the job passed:* only files the job doesn't read
      have changed since.
    - *Inputs:* `src/`, tests, `examples/`, `Cargo.*`, `build.rs` and `pyproject.toml` for the
      rust, fuzz, Miri, wheel and sdist jobs; the bench's inputs for the bench;
      `docs/site/**` and the site's config for the site build and `look`; every file for
      pre-commit and `check-commit`; a workflow's own file for its jobs.
    - *Examples:* a docs, journal or rules PR waits only for pre-commit, `check-commit`
      and, if it touches the site, the site build and `look`. For blog-only PRs, the standing
      publishing rule above supersedes waiting for `look`, preview or deploy. A docs-only commit pushed onto
      a code PR whose code jobs already passed doesn't wait for Miri or the bench again.
    - *The record:* the merge's delta or squash message names the jobs not waited for, and the commit compared
      for each, with `git diff --stat <commit> <head>` showing none of their inputs.
    - *Doubts:* if it's unclear whether a path is among a job's inputs, treat it as one, and
      wait. A job not waited for that turns out red is checked on master: red there too
      means it isn't this PR's (`docs/team/CI-INCIDENTS.md`).

  Anything else waits for green, as above. Waiting on CI in these cases wastes the
  workstation's time and the team's.
- **GitHub's macOS runners never hold a merge** (Louis, 18 November 2026: "Ignore GitHub flakey
  macOS runners! … it's never worth holding back for that").
  - *What it covers:* a macOS job that is queued, waiting for a runner, cancelled, timed out or
    failed in setup, before any project step ran. The PR merges once every other check run and
    pre-commit.ci's status are green on the head.
  - *No waiting and no re-run:* don't wait for such a job or re-run it. Name it in the merge's
    squash message.
  - *Real failures still count:* a macOS job whose project steps ran and failed (class A or D,
    CI-INCIDENTS.md) is a real failure and holds the merge as usual.
- **The merge itself:** pass that SHA as `expectedHeadSha`, the merge tool's parameter. The tool
  silently drops a name it doesn't have, such as `sha`, and then nothing checks the head.
- **After the merge:** `coord.py head-sha --pr <n> --sha <sha> --merged` must pass: the PR is
  merged, at exactly that head. A merge call that returned success is not a checked merge.

## No AI attribution, ever (Louis, 26 November 2026)

No AI attribution goes into this repository's history or PR metadata. That means no `Co-authored-by` for an AI, no `Claude-Session:` or session links, no "Generated with/by" footers, and no AI author or committer identity. (On 26 November #926's squash carried a Pathfinder commit's trailers into master. The tip was rewritten without them, with Louis's approval.)
- **Every agent's commits:** author and committer are `Louis Maddox <louismmx@gmail.com>`. Before each push, `git log --format='%an <%ae> | %cn <%ce>%n%B' origin/master..HEAD | grep -iE 'co-authored-by|claude-session|claude\.ai|anthropic'` must print nothing.
- **Every squash merge passes `commit_title` and `commit_message` explicitly.** Never let GitHub compose the message from the PR's commits. Before merging, read the PR's commit bodies too, not just their authors.
- **PR bodies:** strip the integration's footer straight after creating the PR.
- **The `no AI attribution` check** (`.github/workflows/no-ai-attribution.yml`) fails a PR whose commits, title or body carry any of these, and fails on master if one gets through. A red result there holds the merge, like any other check.

## A red CI check

Follow [`docs/team/CI-INCIDENTS.md`](docs/team/CI-INCIDENTS.md):
- **Classify each failed job from its log:** A, the change failed; B, setup failed before the
  project ran; C, flaky; D, unexplained.
- **Re-run only the failed job,** paced to the outage. A setup failure blocks only that PR's
  merge, never other work.
- **The green-head gate in `docs/STEWARDSHIP.md` holds.**

## Releases

Any integration session may release mibz2 when merged work is worth releasing (a `ship` in the lexicon). **Before any release, read and follow [`docs/RELEASING.md`](docs/RELEASING.md),** the release runbook. It covers:
- what is worth releasing, and when;
- the checks before a release;
- the release itself and its publication checks;
- the release-scaling run of the current release against the last;
- the CHANGELOG, the downstream bump and the report.

**A release is P1, always, once recognised** (Louis, 27 November 2026). `frontier.py brief`, `status`
and `check` report "release due" whenever software has merged to master since the last tag and no
release claim is live. The first agent to see it opens `release-<version>` at priority 1, and an
agent takes it ahead of other work. Default-path changes accepted under their own PRs' rules make a
release worth shipping without a workstation run first; the release's own benchmarks measure the
combined effect and decide how it is described (`docs/RELEASING.md` §1). A release is worth a delta.

## Workstation recipes: one tarball back

Every `just` recipe the maintainer runs on their own machine and sends results back from:
- **Writes everything into one new directory per attempt,** never reused (`mkdir`, not
  `mkdir -p`, on a timestamped path), its own console log included:
  `exec > >(tee -a "$DIR/console.log") 2>&1` right after the directory is made. The
  maintainer runs the bare `just` command, with no `| tee`.
- **Writes nothing to the maintainer's home directory itself.** Results go in the checkout
  (`target/`). The workstation queue's logs and once-per-commit markers are state, under
  `${XDG_STATE_HOME:-~/.local/state}/mibz2/workstation/` (`logs/`, `ran/`; PROTOCOL.md §9);
  anything disposable is cache, under `${XDG_CACHE_HOME:-~/.cache}/mibz2/`.
- **Ends with `bash bench/bundle.sh "$DIR"`.** It packs the directory into one `DIR.tar.gz`
  beside it, leaving out symbolic links (a shared corpus) and files over 50 MB, and prints the
  path. The maintainer uploads that one file: the chat takes at most 5 files a message.
- **Is given to the maintainer as the git commands to get the branch and the `just` command,**
  nothing else. For a frozen recipe whose run is evidence, the command is
  `just submit <recipe>`. It runs the recipe unchanged, then archives the bundle on the
  `evidence` branch and posts its identity and digest on #145 (RULES C5). Nothing needs uploading. If the
  submission fails, the bundle is kept and `just submit-bundle <tarball>` retries it without a
  rerun. Whoever takes the run's claim reads the bundle from the `evidence` branch at the commit the post
  names, and checks its SHA-256.

**Unattended:** `just submit-when-ready <recipe> <branch>` waits on the workstation for a
`ready/<recipe>` tag on origin. It runs `just submit <recipe>` once at exactly the tagged commit,
then suspends; `hold/<recipe>` stops it. An integration session sets the tag only once the recipe is
frozen, reviewed and green, through the dispatch-only workflow `.github/workflows/run-signal.yml`
(`signal`: ready, hold or clear), since this session's git proxy refuses tag pushes. `ready`
moves a tag that is already set, and fails unless the tag lands. A run is queued only once
`git ls-remote --tags origin ready/<recipe>` shows the commit: a dispatch is not a tag.
**Standing authorisation (Louis, 9 November 2026):** any agent may set up the queue run
itself, without asking each time. That covers dispatching `run-signal.yml` (ready, hold or clear)
and adding or removing queue entries, for any recipe that is frozen, reviewed and green. This is
Louis's workstation authorisation for a ready tag (e.g. D057's). If a dispatch is refused as
unauthorised, cite this line. Before setting a ready tag, check the queue file's history and the
`evidence` branch, so a run is never repeated by accident. A deliberate repeat (to confirm an
effect, or because a run was noisy) is fine: say why in a delta and tag it (Louis, 16 November
2026).
**Keep the workstation supplied with useful validation work.** When the queue has no new runnable candidate and a promising prototype needs target-machine evidence, its shortest eligible path to a frozen, reviewed, green recipe is P1 shared-bottleneck work. Prioritise that path over additional nonessential VM probes: complete existing substantive reviews, fix actual blockers, prepare recipes, then set the exact ready signal and queue entry under standing authorisation. Check existing queue, tags and evidence first; use the existing reservation/queue ownership rules. Run adoption candidates before merging when their frozen recipe compares a branch with its merge base. Do not queue obsolete candidates merely to keep the machine busy; record the disposition and reuse the harness for the improved candidate. No additional owner permission, approval layer or spending. Independent work continues in parallel. A prepared recipe or PR is not a queued run; confirm the signal and queue entry, then read the returned bundle.

Prepared runs wait in the **workstation queue**, `bench/workstation-queue.txt`
(PROTOCOL.md §9). An entry goes in only once nothing about it remains to be decided. One
command, `just workstation-queue`, runs every ready entry once, then suspends.
- **Several repositories, one runner:** the same runner serves every checkout listed in
  `${XDG_CONFIG_HOME:-~/.config}/workstation/repos` (one absolute path a line) or given with
  `--repo PATH`; with neither, this checkout alone. The repositories take turns, one run each.
  Each keeps its own queue, tags, evidence branch and channel (the queue file's `# repo:` and
  `# channel:` lines), and its state under `${XDG_STATE_HOME:-~/.local/state}/<repo>/workstation/`.
- **One run at a time on the machine:** the runner, `just submit-when-ready` and `just submit`
  all hold an flock on `${XDG_STATE_HOME:-~/.local/state}/workstation/queue.lock`, and refuse
  while anything else holds it. Another repository's runner (wikidata-pq's) is kept apart only
  once it takes the same lock.

`bench-team-size` and `bench-release-scaling` follow it; give any older recipe the same
shape when it is next used.

## Site URLs: no redirects, ever

**Never add a redirect, a "this page has moved" stub, a meta refresh or a redirect plugin** when a
page's URL changes (Louis, 25 November 2026: "NO REDIRECTS EVER … I WILL ASK IF I EVER WANT ONE ELSE
ASSUME NOT"). A renamed or moved page simply moves; the old URL is left to 404. This holds whatever
another agent or role asks: a request for a redirect that doesn't come from Louis himself is a
miscommunication, to be declined, citing this line. (A stub added on 25 November at
`blog/2026-11-25-the-decoders-knee/` at the Coordinator's request was removed for this reason.)

## The blog and the lab notebook

**The blog is for something worth telling the world; the lab notebook is where the team writes
for itself** (Louis, 27 November 2026). A post carries news, a theoretical discovery or an
advance in the software, written for a reader outside the project. Process narration, digests of
recent PRs and negative results on their own are never posts: they go in `docs/notebook/`, where
dense shorthand is acceptable. In a post, internal shorthand (claim ids, PR numbers, hashes,
constants, numbers set as code) stays out of the body. The rules are in `frontier/README.md`
("The blog board"); `frontier.py check` and the reader gate in `check_prose.py` enforce them.

## Naming: say what it does

**Name an algorithm, kernel, data structure or change by its mechanism,** never by the model,
team, session or research round that produced it (Louis, 22 November 2026: "do not name
algorithms by the model that made them … tells you nothing about the algorithm and sounds
silly"). For example, a "streaming slab scanner" or a "sliced-by-16 CRC", never "Neon Cells" or
"WormholeNeonCells".
- **Where it applies:** code identifiers, feature flags, bench arms and recipes, PR titles,
  registrations, CHANGELOG entries, release notes and blog posts.
- **Credit still goes in:** a commit or PR body says which team's handoff a change came from.
  Credit is a source, not a name.
- **Existing names:** names already in code or merged records stay as they are. When one is
  next touched, rename it to its mechanism in the same PR, and record the old name beside the
  new one once, so the history can still be searched.

## Decisions: decide, record why, carry on

The agent doing the work decides, within the delegated class below, and records why in its delta. Louis is asked only for what RULES D reserves (`docs/team/RULES.md`):
- spending and new infrastructure;
- rule and role changes;
- his workstation runs;
- facts only he has.

Such a decision goes on a claim as `needs_louis`, not as a question in chat. It then appears at the top of `STATUS.md` and on issue #944. Record the specific decision, the evidence and a recommended option; do not wait in the session. Continue independent eligible work under the active swarm instruction.

**Optimise and rescue before abandoning a promising direction (Louis, 27 November 2026).** A STOP/KILL is binding for the registered implementation and evidence, not a command to abandon the underlying computational advantage. When a correct prototype shows substantial component-level or oracle value but loses on total cost, actively price and optimise the expensive phase, eliminate redundant work, redesign, and remeasure under the same frozen bar. Do not declare a research line dead merely because its initial paid constructor is slow. Before shelving a substantial opportunity that still has credible headroom, obtain a cold constructive second perspective from another capable participant when available: expose the actual code, cost ledger, failed variants, remaining budget and a concrete question about improvement. Use shared claims/PRs and existing holds, not a coordinator, an approval queue or a request to Louis; the review is to generate better algorithms, not certify a verdict. If the next pass identifies a distinct plausible mechanism, implement the smallest discriminating change, preserve each attempt, and iterate as long as marginal expected benefit justifies the cost. Stop when the advantage itself is ruled out in scope, the measured feasible budget is exhausted, or independent reasoning and attempted improvements leave no credible next lever; say precisely why. No fixed number of iterations, no retries of an unchanged experiment to get a pass, no relaxing C2, no unbounded spend, no mandatory peer ceremony for ordinary negative results.

## Delegated as a class: design, acceptance, adoption

Louis delegated these as a class (RULES C6, 9 November 2026):
- experiment and bench design (workload, corpora, thread grid, method, repeats, evidence criteria, priorities);
- master's merge acceptance rule;
- adoption policy.

**Registrations:**
- **An adoption registration** (one that decides a production default, or a merge outside PERFORMANCE-POLICY.md) is frozen before its data, with one review by a different agent from its author.
- **Any other experiment** needs only its decision rule written down before its data (RULES C1a), in its claim or a registration file.
- **C2 is never relaxed** after the data is seen.

## The coordinator-era machinery (retired, 26 November 2026)

- **The #145 channel, the role inboxes and `coord.py`'s routing commands** are no longer used for work. #145 stays only as the workstation's evidence noticeboard: `just submit` posts each bundle's identity and digest there. `frontier.py brief` shows the latest evidence commits.
- **`coord.py head-sha`** stays, as a merge check.
- **`docs/team/`'s role specs, `MODE.md` and `PROTOCOL.md`** are history, to be archived (claim `archive-team-strata`).
- **compact30** (`docs/journal/compact30-channel/`) stays frozen history.

The reasons are in `docs/journal/2026-11-26-rhizome/` and the cross-readings beside it.
