claim: declarative-ready-signal
by: claude-code
outcome: done
evidence: #1104; bench/run_requests.py; bench/test_run_requests.py (35 tests frozen at c3e92ab, 53 after review); .github/workflows/run-signal.yml (reconcile job); docs/ops/run-requests.md

**What was asked:** let a participant that can push a branch, but not dispatch a workflow (#1086), raise a ready signal without a PR, master commit or new credential per run.
**What I found:** the deployed runner needs no change, because it only reads `ready/` and `hold/` tags. The bridge is therefore a writer.
- **Requests:** a request is a commit on `refs/heads/run-request/<recipe>/<12 hex>` holding a strict `request.toml` (`python3 bench/run_requests.py publish`).
- **Reconciler:** a scheduled job in run-signal.yml, every 15 minutes plus `signal: reconcile`. It runs only the default branch's code, treats each request as data, and sets the tags only when every gate holds at that moment:
  - the queue line;
  - a queue claim on master that names the recipe, status open or review, and no needs_louis;
  - a live `hold/<claim>` whose session matches;
  - the commit on the branch, and the PR (open or merged, same repository) with its head exactly the commit;
  - the named review at that head, and no CHANGES_REQUESTED at it;
  - green as CLAUDE.md defines it: every run passed or skipped, check-commit, no-ai-attribution and pre-commit.ci reported success, every workflow finished; only a macOS job still waiting for a runner, and a cancelled run whose same-named twin passed, are not waited for;
  - no `hold/` tag; `ready/` unset, already that commit, or the request's `supersedes`.
- **Atomicity and replay:** activation is one `--atomic` push of `ready/` (leased on its raw old value) and an annotated receipt `run-receipt/<recipe>/<sha>`. Receipts are tombstones. No request re-signals a recipe at a commit. A request older than the recipe's latest receipt is retired. A hold leaves `hold-<request commit>`, so a manual clear stays cleared.
**Independent review:** a cold review at 7f8e5c0 asked for changes, and every point is fixed with a test:
- a hold re-applied after a clear;
- malformed lease or claim TOML killing the whole pass;
- a macOS exemption wider than CLAUDE.md's;
- absent checks or status counting as green;
- the concurrency group able to cancel manual dispatches (the group is now reconcile-only);
- the claim not tied to the recipe;
- an older request reviving after a clear;
- the deny-listed claim status, fork and closed PRs, `version = true`, and leasing on a peeled tag.
- Withdrawal-after-activation is now documented: use a hold.
**Re-review at a9d21e3:** all earlier points confirmed fixed. One new defect: a ready receipt from the same pass retired a pending hold. Fixed with like-retires-like receipts and holds decided first, with tests. A nit asked to key cancelled twins by suite; it was declined, because real twins sit in different workflow runs (`mirror` on #1104). This is documented.
**CI at 7f8e5c0, two failures, both class A:**
- no-ai-attribution: the integration appended its footer to the PR body; stripped.
- test: the module's git calls had no identity on the runner; the tests now set one.
**Frozen-test corrections:** `test_commit_not_on_branch` used an ancestor commit, so it now uses a stray one. `refused()` now asserts the tag is unchanged, not absent. Both were corrected before any real run.
**What's next:** codex's queue claims (scanner-seam-window-queue, a6b-endpoint-queue) can publish requests. The first scheduled reconcile is the first real API and tag-push test: read its summary. CLAUDE.md's Unattended paragraph still names only dispatch. A pointer is left out because CLAUDE.md is protected.
**Caveats:**
- One GitHub account means requester and reviewer identity are unprovable. Authority is the frontier lease, as for manual dispatch.
- Evidence bundles don't name their commit, so the prior-attempt check beyond receipts and `ran/` stays with the requester.
- Request refs are never deleted, only aged out.
- Not yet run on GitHub or the i9.
