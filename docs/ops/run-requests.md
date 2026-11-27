# Run requests: ready signals without workflow dispatch

A workstation run starts when `ready/<recipe>` is on origin at the exact commit to run
(`bench/await-submit.sh`, `bench/run-queue.sh`). The tag has two writers, both in
[`.github/workflows/run-signal.yml`](../../.github/workflows/run-signal.yml):

- **Manual dispatch** (`signal`: ready, hold, clear): for an agent that can dispatch a
  workflow. Unchanged.
- **Run requests**, reconciled on a schedule (every 15 minutes, and `signal: reconcile` on
  demand): for an agent that can push a branch but not dispatch a workflow (#1086). This needs
  no pull request, master commit or review of its own.

Both act only under the 9 November standing authorisation: a frozen, reviewed, green recipe.
The runner is unchanged; it still reads only the tags.

## Publishing a request

Hold the recipe's queue claim (mechanism `workstation-queue`, or owner of
`bench/workstation-queue.txt`), with the recipe's queue line already merged on master. Then:

```
python3 bench/run_requests.py publish --recipe <recipe> --sha <40-hex commit> \
    --branch <recipe branch> --claim <queue claim> --session <your lease session> \
    --pr <recipe PR> --review <id of the independent review at that commit> \
    [--supersedes <commit ready/<recipe> is at now>] [--note "<why>"]
```

This pushes one new commit, holding `request.toml`, to
`refs/heads/run-request/<recipe>/<12 random hex>`. It is a plain branch push (the git proxy
refuses tag pushes, not branch pushes). Other agents publish their own refs at the same time
without contention. To stop asking: `python3 bench/run_requests.py withdraw <ref>`. This only
stops a request that hasn't been acted on yet. An activated run is stopped by a hold:
`publish --action hold --recipe <recipe>`. A hold needs no claim or review, because it only
ever stops work.

**A request is not a queued run.** Check `git ls-remote --tags origin 'ready/<recipe>'
'run-receipt/<recipe>/*'` after the next reconcile, or read its reasons in the Run signal
workflow's run summary.

## What the reconciler checks

It runs only from the default branch's copy of the workflow and of `bench/run_requests.py`. It
never checks out or runs a request or a candidate, and it never puts request text into a shell.
For each request, at that moment:

- **The request:** strict TOML with known keys, at most 4 KiB, safe identifiers, the same
  recipe as its ref, published within 72 hours, not withdrawn.
- **The queue:** master's queue lists the recipe on that branch.
- **The claim:** it is a queue claim on master whose text (title, question, why, stop test or
  done-when) names the recipe. Its status is open or review, and it needs nothing from Louis.
  `hold/<claim>` is live and unreleased, and its session is the requester's.
- **The candidate:** the commit is on the branch. The PR is open or merged, and its head is
  exactly that commit on this repository's branch, not a fork's. The named review is at that
  commit, and no review at it requests changes.
- **CI:** green as CLAUDE.md defines it:
  - every check run passed or was skipped;
  - `check-commit`, `no-ai-attribution` and pre-commit.ci's status have reported success;
  - every Actions workflow on the commit has finished.

  Two things are not waited for: a macOS job still waiting for a runner, and a cancelled run
  whose same-named twin passed (a superseded duplicate). A macOS job that started, timed out
  or was cancelled holds the request. The reconciler can't tell a setup failure from a test
  failure, so it fails closed; manual dispatch remains for that case.
- **The tags:** `hold/<recipe>` is not set. `ready/<recipe>` is unset, already this commit, or
  exactly `--supersedes`. No receipt exists for this recipe at this commit, and the request is
  not older than the recipe's latest receipt.

Anything missing, ambiguous or unreadable is refused. The reasons are printed, and the request
is looked at again next time.

## Receipts: once per recipe and commit

Activation is one atomic push. It moves `ready/<recipe>` with a lease on its old value, and it
writes the annotated tag `run-receipt/<recipe>/<commit>`, whose message records the request, its
claim, session, PR, review and the previous ready commit. The receipt is the tombstone. Once a
request has signalled a recipe at a commit, no request does so again: not the old one, a
replayed one or a new one, even after `ready/<recipe>` has been cleared. A request published
before the recipe's latest receipt is retired by it. So after someone clears a request-made
signal, an older pending request for another commit can't take its place on the next pass. A
hold request leaves `run-receipt/<recipe>/hold-<request commit>`, so clearing a hold is never
undone by the same request. Only like retires like: a ready receipt never retires a hold
request. Within a pass, holds are decided before ready requests, so a hold published alongside
a ready request always wins. A deliberate repeat at the same commit goes through manual
dispatch, with a delta saying why (CLAUDE.md, 16 November). A manual `clear` of a manually set
tag leaves no receipt, so whoever clears it should also withdraw or hold any pending request
for that recipe.

## What is not verified

- **Identity:** every agent pushes as the same GitHub account. The reconciler can't tell who
  published a request or wrote a review. Authority comes from the frontier's own protocol: a
  live lease whose session matches, on a queue claim. A review counts if it is at the exact
  head, and its independence rests on the recipe's claim and delta, as it does for a manual
  dispatch.
- **Prior attempts:** evidence bundles on `evidence` don't name their commit. Checking for a
  prior attempt beyond the receipts and the runner's own `ran/` markers stays with the requester
  (the queue claim's stop test).
- **Queue entries on master** (such as `bench-release-scaling`) have no PR head to match, so
  they always fail closed here. They stay manual dispatches.
- **Request age** comes from the publisher's own commit date, so the 72-hour limit is advisory.
  The authority lies in the lease and the receipts.
- **A workflow that never starts** (one waiting for approval, say) holds every request for
  its commit. This fails closed, so use manual dispatch.
- **Cancelled twins** are matched by job name across workflows. A superseded run's twin is in
  another workflow run (on #1104, `mirror` was cancelled in one run and skipped in another), so
  a cancelled job is excused when a job with the same name passed anywhere on the commit.
- **Cleanup:** nothing deletes request refs, since some proxies refuse deletions. A request
  older than 72 hours is ignored, and a received one is reported `done`.
