# One workstation runner for several repositories

*22 November 2026. Measurement infrastructure only: nothing here is a bench, and no measured
figure changes meaning.*

**Why.** Louis runs one i9 workstation. More than one repository queues work for it: mibz2 now,
upigz soon (a gzip decoder cloned from mibz2's template), and wikidata-pq, which already has its
own port of the queue. Until now each repository had its own runner, and the machine was shared
by hand: the header said "only one workstation queue runs at a time: stop wikidata-pq's before
this one". Now one runner serves every queue and shares the machine between them. The queues,
tags, evidence branches and channels stay separate per repository. Nothing becomes a monorepo.

## What changed

- **A list of repositories.** `bench/run-queue.sh` (`just workstation-queue`) serves every
  checkout given with `--repo PATH` (repeatable), or else listed in
  `${XDG_CONFIG_HOME:-~/.config}/workstation/repos`, one absolute path a line (`#` comments
  and blank lines are ignored). With neither, it serves the checkout it is run from, as before:
  same output, same log location, same exit statuses. `just workstation-repos [PATH...]` shows
  the list and adds checkouts to it.
- **Turns.** Each pass reads every repository's queue, waiter and ready tags from that
  repository's own origin (its default branch). The repositories then take turns in the order
  listed: in its turn a repository runs at most one entry, skipping past entries that are not
  ready, on hold or already run. A busy queue can't starve another. `--hours`, `--wait`, the
  polling, HOLD, the 10-pass cap and the suspend are as before, across all the repositories: the
  runner suspends only after a pass in which no repository has anything new.
- **One machine lock.** The runner holds an flock on
  `${XDG_STATE_HOME:-~/.local/state}/workstation/queue.lock` for its whole run. It refuses to
  start while anything else holds the lock, and says who holds it (the holder writes its pid
  and role into the file). `bench/await-submit.sh` (`just submit-when-ready`) and `just submit`
  take the same lock, or keep the runner's, which they inherit on fd 9 (`WORKSTATION_LOCK_FD`).
  So two runners, or a stray `just submit` while the queue runs, can't overlap on the machine.
  The manual "stop the other queue first" rule is gone from the header and from PROTOCOL.md §9.
  **wikidata-pq's own runner is kept apart only once it takes the same lock**, the same path,
  with flock, for its whole run. Until then, running it beside this one is still unsafe. (Listing
  wikidata-pq's checkout in this runner's config already works, as below.)
- **State stays per repository.** Each entry's log and its once-per-commit marker live under
  `${XDG_STATE_HOME:-~/.local/state}/<repo>/workstation/` (`logs/`, `ran/`), where `<repo>` is
  the name of the checkout's origin repository. For mibz2 that is `mibz2/workstation/`, as
  before, and wikidata-pq's port already uses `wikidata-pq/workstation/`. The run's own log is
  in the repository's `logs/` when one checkout is served, and in
  `${XDG_STATE_HOME:-~/.local/state}/workstation/logs/` when a list is.
- **Each repository runs its own flow.** An entry runs in its own checkout, through the
  `bench/await-submit.sh` on that repository's origin and its own `just submit`. So its
  evidence lands on that repository's `evidence` branch, and its LOGs on that repository's
  channel.
- **One repository's trouble stops nothing else.** A missing checkout, a dirty one, a
  duplicate, or an origin that can't be read leaves that repository out (for the run, or for the
  pass), and the summary says so. A failed recipe is logged and the turns go on. The exit status
  is 1 if any run failed, 5 if no repository could be served or the lock is held, else 0.
- **The four scripts name no repository.** `run-queue.sh`, `await-submit.sh`, `submit.sh` and
  `bundle.sh` can be copied byte for byte into a sibling. A repository's own names come from its
  checkout and its queue file:
  - the state directory's name, from the origin URL;
  - the evidence repository and the channel's issue number, from two comment lines in
    `bench/workstation-queue.txt`, `# repo: lmmx/mibz2` and `# channel: 145`. `submit.sh` checks
    that origin is that repository, as it checked against the hard-coded `lmmx/mibz2` before. A
    queue file that declares neither line makes `submit.sh` refuse (keeping the bundle), and the
    waiter skip its notices;
  - the default branch, from origin's HEAD (master if origin doesn't say).
- **wikidata-pq's `args=` column.** wikidata-pq's queue lines may carry `args=A,B` as their
  third column, which its waiter takes as `--args A,B`. The runner now passes that through, and
  mibz2's waiter accepts `--args` too (`just submit <recipe> A B`). mibz2's own entries don't use
  it.
- **Tests.** `bench/test_run_queue_multi.py` (new, in CI's `submit` job beside the queue test):
  two scratch repositories with bare file origins and stand-ins for just and gh. It covers the
  turns, an entry that is not ready not spending a turn, once per ready commit per repository
  (the same recipe name in both), the lock refusing a second runner and a lone waiter (and being
  held while a recipe runs), each repository's own waiter and the `args=` column, a failed recipe
  and a dirty checkout or lost origin not stopping the other, and single-repository mode
  unchanged, with `--repo` overriding the config file. `bench/test_workstation_queue.py` (single
  repository, unchanged in substance) now names its scratch origin `lmmx/mibz2.git`, since the
  state directory follows the origin's name. `bench/test_submit.py` checks the declared channel
  and the refusal without one.

## Setting it up on the workstation

Once this is merged, in the mibz2 checkout:

    git fetch origin
    git checkout master
    git merge --ff-only origin/master
    just workstation-repos . <path to the upigz checkout>

and from then on, from any of the listed checkouts:

    just workstation-queue

`just workstation-repos` with no paths shows the list. Without a list, `just workstation-queue`
behaves as it always has, for the checkout it is run from.

To try it before the merge, use `git checkout bench/runner-multi-repo` in place of the master
lines above, and run `just workstation-queue --no-suspend`. Until the merge, each repository's
waiter comes from its origin's master, so mibz2's entries still go through the old waiter. It
posts to #145 as before, and holds the lock only because it inherits the runner's.

## How a sibling repository joins

1. Copy `bench/run-queue.sh`, `bench/await-submit.sh`, `bench/submit.sh` and `bench/bundle.sh`
   from mibz2 byte for byte. Copy the `submit`, `submit-bundle`, `submit-when-ready`,
   `workstation-queue` and `workstation-repos` recipes (`.just/submit.just`): the `submit`
   recipe takes the machine lock.
2. Give its `bench/workstation-queue.txt` the two lines `# repo: <owner>/<name>` and
   `# channel: <issue number>`, and put it on the default branch.
3. Add its checkout to the workstation's list: `just workstation-repos <path>`.

A repository whose own port keeps the same waiter interface (`<entry> <branch> --no-suspend
--hours N [--args A,B]`, exit 0 passed, 1 failed, 3 hold, 4 not ready, 5 refused) and names its
state directory after its origin can be listed without copying anything. wikidata-pq's
`bench/await-submit.sh` does both, so it can be listed today. Its own `run-queue.sh` still has to
adopt the lock before it can safely be run on its own beside this one.

## What resisted

- **`submit.sh` is not the same in wikidata-pq.** wikidata-pq's port reads more from the bundle
  (its PR number, the mibz2 versions compared) and posts to the PR or to #97. That is a real
  difference in what the evidence says, not a name. So "copy byte for byte" holds for mibz2's
  template siblings (upigz), and wikidata-pq keeps its own `submit.sh` and waiter. It joins
  through the shared interface instead.
- **The channel and the repository have to be declared.** Neither can be derived safely: the
  origin URL names the repository, but the check that a bundle goes to the right repository
  needs an independent statement of which one that is. So the queue file declares it.
- **`bundle.sh` records the Rust toolchain.** That is harmless where there is none (it is
  recorded as missing), but it is a template assumption.
- **The posted text** still says "(RULES C5)", a rule of the team template's that a sibling
  shares, and the lock only covers runs that go through these scripts. A bare `just <recipe>`
  typed by hand takes no lock.
