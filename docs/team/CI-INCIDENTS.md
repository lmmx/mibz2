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
  - Use the run-level "re-run failed jobs" when several jobs of one run failed.
  - Never re-run the whole workflow for one job.
  - GitHub allows one re-run in a run at a time.
- **Re-run when a different outcome is plausible, not to a count.** Each re-run should have a
  reason:
  - the error is the kind that clears (a 5xx, a timeout, a truncated body);
  - time has passed;
  - or there is a sign of recovery, such as the same step passing elsewhere or the service's
    status clearing.
- **Back off as the failure repeats.** As an illustration, not a rule: minutes at first, then
  tens of minutes, then only on a sign of recovery.
- **Before each re-run, read the new log.** Is it still the same step and the same error? If
  the failure has moved to a project step, the job is class A or C now.
- **Stop when the cause is established and nothing has changed.** That is the same acquisition
  failing the same way, with no sign of recovery. Further re-runs then only spend runners.
  - The incident is *waiting on infrastructure*: note it on the PR once, and look again at
    check-ins.
  - A number of attempts, or the time elapsed, says how long the outage has lasted. It is not
    evidence about the change.
- **A repair is a separate infrastructure PR, never part of this one.** Examples: a tool pin,
  caching, a mirror, or a retry around a download. Open it when the cause is persistent and the
  repair would remove it. A pin doesn't help when the asset host itself is down.

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
| class B, recovery plausible | re-runs, as in §2, on its own |
| the class B job passes on the same head | closes the incident, and merges once the PR is otherwise eligible |
| class B, cause established and no sign of recovery | waits, looks again at check-ins, and notes it once |
| class A or C | fixes, or re-runs C once, in the PR |
| class D, or a misclassification it can't rule out | investigates; escalates if it blocks |
| class B on a PR that Louis needs merged soon | tells Louis what is blocked, and that the gate holds |

## 4. A red check that won't recover

**In force: the green-head gate.** A PR merges only when every required check is green on the
head that merges (`docs/STEWARDSHIP.md`). Until Louis approves otherwise in session (RULES A4),
a class B job that won't recover leaves its PR waiting, with the blocker stated as in §3.

### Proposed for Louis's approval, not in force: a judged infrastructure exception

**The principle.** CI exists to verify the change. A required job that never ran its
validation has verified nothing, so it is neither a pass nor a failure. When its cause is
demonstrably external, the question is whether its absence leaves a material gap in what this
change needed verified. If there is no material gap, the Coordinator may merge, and must record
its judgement. If there is a gap, or any doubt about one, it holds.

**What the Coordinator must establish, and record on the PR, before an exception:**
1. **The external cause.** For every attempt of the job: the failed step, which comes before
   the project's own steps, and the decisive log line naming an external service.
   - Every failed attempt is the same external failure.
   - Nothing in the diff can reach that step: not the workflow, the toolchain, the build
     requirements, or anything else the step reads.
2. **What the change needed verified.** From the exact diff, list its obligations:
   correctness, safety, packaging, platform, compatibility and performance.
   - A docs change needs the docs to build and render.
   - A decoder change needs the tests, parity, the bench and every platform it compiles on.
   - A packaging change needs the artefacts themselves.
3. **What ran on this exact head,** and which of those obligations it met.
4. **What the missing job would have verified,** and whether anything that ran covers it for
   this change.
   - Equivalence must be argued, not assumed. Another platform's build is not this platform's
     build. It covers only what the change can't make platform-specific.
5. **The residual risk:** what the missing job could have caught that nothing else did, and why
   that is acceptable for this change.
6. **The follow-up:** the job re-run on master, or on the next PR, once the service recovers,
   with the result linked from the record.

**The exception applies if and only if all of these hold:**
- the cause is external, as in 1;
- the missing job leaves no material gap, as in 4 and 5;
- every other required check is green on the same head;
- the merge comment names the exception and links the record.

**Hold, whatever else is true, when:**
- any project step failed: a test, parity, a safety check, the bench, lint or typing;
- the cause is unclear, mixed or possibly change-induced, or the diff could reach the failed
  step;
- the missing job is the only verification of something the change needed. Examples are the
  only build of a platform whose behaviour the change could affect, the only render check of
  a page the change edits, or the bench for a measured change;
- the PR makes or prepares a release, or touches publishing. Release artefacts are verified
  themselves, and independently;
- a test is flaky and the flake is unresolved.

**The Coordinator's responsibilities:** the record above, made before the merge; no invented or
disabled checks; no reinterpretation of a failed test as infrastructure; and the follow-up
re-run. Louis can review any exception from its record.

**Examples:**

| PR | missing job and cause | needed verification, and what covered it | verdict |
|---|---|---|---|
| #255: `bench/run-queue.sh` and PROTOCOL.md | x86_64 Linux wheel; *Install maturin* returned a 92-byte non-archive | Nothing in the diff is built into a wheel. The wheel build verifies packaging of unchanged code, and the other wheel builds, tests, typing and the bench passed. The script itself was tested in a scratch repo, recorded on the PR. | PROCEED (no material gap) |
| #256: README.md and CHANGELOG.md | x86_64 and aarch64 wheels, sdist; maturin download 504 / non-archive | Docs only. The README is the package's long description, so the sdist carries it, but its rendering is checked by nothing in those jobs. | PROCEED, once any docs check that does run is green |
| A change to `src/` (a kernel) | x86_64 Linux wheel; the same download failure | The tests and bench ran on x86_64 Linux, so the code is verified there. The manylinux wheel's packaging is what's missing: its glibc symbol versions and its auditwheel repair, which a code change can affect. | HOLD (material gap) |
| A change to `.github/workflows/` or `pyproject.toml` | any setup step | The diff can reach setup, so the cause can't be shown to be external. | HOLD |
| A release PR | any job | Release artefacts are verified themselves. | HOLD |
| Any PR | a test that timed out | That is a project step, so class C, not infrastructure. | HOLD until re-run and explained |

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
- **Under this runbook:** the job alone is re-run on the same head, backing off, while recovery
  is plausible. The PR merges once the job passes. Under §4's exception, if approved, it would
  merge with the job's absence recorded.
- **#256,** a docs-only PR, hit the same incident at the same time.
