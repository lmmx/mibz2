# CI incidents: classify, recover, record

What the Coordinator does when a CI check on a PR is red. It applies to every PR the team drives.
It doesn't change the merge gate: `docs/STEWARDSHIP.md` still requires CI green on the exact
head that merges. What it changes is how a red check is handled before that.

**The principle:** a red check is evidence about one job, at one step, on one head.
- Read it from the log, not from its colour.
- A failure that stopped before the project's own steps ran tested nothing. It is not a
  verdict on the change. It is an incident to resolve, and it is not a pass either.

## 1. Classify, from the log

For each failed job, open its log and find the step that failed. Then put it in one class:

| class | what the log shows | examples |
|---|---|---|
| **A. the change failed** | a project step ran and failed: build, test, parity, lint, typecheck, bench gate | a compile error, an assertion, `same bytes: no`, a bench row beyond noise |
| **B. setup failed before the project ran** | the failure is in a setup step, before any project step started, and names an external service | a tool or toolchain download (HTTP 5xx, a truncated or non-archive body), registry or cache timeouts, runner provisioning |
| **C. flaky or ambiguous** | a project step failed in a way that may not reproduce: a timeout inside a test, an intermittent assertion, a race | a test that passed on this exact commit earlier |
| **D. unexplained** | none of the above fits, or the evidence is mixed | |

**Class B needs all three of these, or it is D:**
1. **The failed step comes before any project step,** and no project step in that job ran.
2. **The error names an external acquisition or setup action,** not the project's code.
3. **The PR's diff can't plausibly affect that step,** because the step reads none of the changed
   files.
   - A diff that touches the workflow, the toolchain files, `pyproject.toml`'s build
     requirements or `Cargo.lock` can affect setup, so it is not class B.

**Read each failed job's log separately.** Two red jobs in one run don't share a cause until each
log shows it.

**Record, in the PR comment or the incident note:**
- the head SHA;
- each failed job: its name, its id, the failed step and the decisive log line;
- the class, and why;
- **which checks ran and passed, and which did not run at all.** A job that never reached its
  build tested nothing. List it as *not run*, never as passed.

## 2. Recover, in proportion

**Class A:** fix the change, and push the fix. Don't re-run a check that the change failed.

**Class B:** re-run, scoped and paced.
- **Re-run the failed job alone,** on the same head:
  `gh api -X POST repos/lmmx/mibz2/actions/jobs/<job_id>/rerun`.
  - Use the run-level "re-run failed jobs" only when several jobs of one run need it.
  - Never re-run the whole workflow for one job.
  - GitHub allows one re-run in a run at a time, so the next job's re-run waits until the
    current one has finished.
- **Pace the re-runs to the outage, not to a count.** Up to 4 re-runs of a job, spaced about 10,
  20, 40 and 60 minutes apart: a short outage clears within that, and a long one doesn't.
  - Before each re-run, look at the new log: is it still the same step and the same error?
  - If the failure moves to a project step, the job is now class A or C.
- **After the fourth failure, or once the outage has lasted 2 hours, stop re-running.** The
  incident is now *waiting on infrastructure*.
  - Note it on the PR once, and check it at the PR's usual check-ins.
  - Re-run once more when there is a sign of recovery: the same step passing on another PR or on
    master, or the service's status page clearing.
- **A repair is a separate infrastructure PR, never part of this one.** Examples: pinning a tool
  version, caching, a mirror, or a retry wrapper around a download. Open it only when the
  failure is persistent and the repair would remove it (for example, a pin, when one version's
  asset is broken). A pin doesn't help when the asset host itself is down.

**Class C:** re-run the job once to see whether it reproduces.
- If the re-run passes, record the flake on the PR and in the queue issue.
- If it fails, it is a real failure: class A, or D if its cause is still unclear.
- Making the test robust is a fix within the PR's scope. Never skip, disable or quarantine a test.

**Class D:** investigate it as a real failure. Escalate to Louis if it still can't be classified
and it blocks a merge he is waiting on.

**Never:** an empty commit, a close and reopen, or a re-run of a check the change failed, to
turn a check green.

## 3. Keep the rest of the work moving

**A class B incident blocks only the merge of the PR it is on.** It never blocks:
- reading the PR's other results, such as the bench and parity, which are valid where their jobs
  ran;
- other PRs, workstation runs, write-ups, or the next experiment;
- deciding the PR on its merits. The PR stays decided, waiting on its gate.

**State the blocker exactly:** "merge waits on `<job>` (class B, `<step>`, since `<time>`); all
other checks green on `<sha>`." Put it in the PR comment, and in #97's state if it lasts.

| case | the Coordinator |
|---|---|
| class B, re-runs in progress | re-runs, as in §2, on its own |
| the class B job passes on the same head | closes the incident, and merges once the PR is otherwise eligible |
| class B past its re-runs | waits, checks at check-ins, and notes it once |
| class A or C | fixes, or re-runs C once, in the PR |
| class D, or a misclassification it can't rule out | investigates; escalates if it blocks |
| class B on a PR that Louis needs merged soon | tells Louis what is blocked, and that the gate holds |

## 4. A red check that won't recover

Until Louis decides otherwise, **the green-head gate holds**: a PR merges only when every
required check is green on its head. An infrastructure exception would change that gate. It is
a change to the acceptance rules, which RULES reserves to Louis. It is proposed below and is not
in force.

**Proposed for Louis's decision (not in force):** a *pre-build infrastructure exception*. A PR
could merge with one job red if every condition below holds.
1. **That job is class B,** on every attempt, with its log lines recorded.
2. **It was re-run at least 4 times over at least 2 hours,** without recovery.
3. **The PR's diff touches none of these:** `src/`, `Cargo.*`, `pyproject.toml`, `build.rs`,
   `.cargo/`, the toolchain files or `.github/workflows/`. So it is docs, bench scripts or
   coordination files.
4. **Every other required check is green on the same head,** including the CI bench where it
   applies, with nothing changed that wasn't meant to change.
5. **The job's own purpose is covered elsewhere** on the same head. For example, a wheel build
   is covered when the other platforms' wheel builds pass and no build input changed.
6. **The merge comment names the exception,** the job and the incident record, and the job is
   re-run on master once the service recovers.

**Residual risk:** a change that would have failed only that job on that platform. Condition 3
makes this negligible, because nothing in such a diff reaches the build.

**Excluded:** any test, parity, safety or bench failure; any flaky or unexplained failure; any
release PR; any PR whose diff reaches the build.

Louis's decision is recorded in `docs/journal/decisions.md`. Until then, a class B job that
won't recover leaves its PR waiting, with the blocker stated.

## 5. Check yourself before acting on a red check

- [ ] Did I open each failed job's log and find its failed step?
- [ ] Did any project step run in that job? If one did, it isn't class B.
- [ ] Could the diff affect that step?
- [ ] Did I list the checks that did not run as *not run*?
- [ ] Am I re-running only the failed job, paced to the outage?
- [ ] Is anything other than this PR's merge waiting on it? It shouldn't be.

**Worked example, #255 at 9f17436b, 8 November 2026:**
- **The failure:** `linux (ubuntu-22.04, x86_64, auto)` failed in maturin-action's
  *Install maturin*, inside its container. The download returned a 92-byte body, so
  `gzip: stdin: not in gzip format`, and no build step ran.
- **The same run's first `sdist` attempt** failed at the same acquisition with HTTP 504. Its
  re-run passed.
- **The diff:** `bench/run-queue.sh` and `docs/team/PROTOCOL.md`, which no build step reads.
- **The rest:** every other build, the tests, typing and the bench were green.
- **So it is class B.**
- **What happened first:** the job was re-run once, failed the same way, and was left with a
  note. Two things were wrong there: a fixed count of one, and no paced recovery.
- **Under this runbook:** the job alone is re-run on the same head at about 10, 20, 40 and 60
  minutes, and the PR merges once it passes. #256, a docs-only PR, hit the same incident at the
  same time.
