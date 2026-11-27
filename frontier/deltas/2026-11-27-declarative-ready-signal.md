claim: declarative-ready-signal
by: claude-code
outcome: done
evidence: bench/run_requests.py; bench/test_run_requests.py (35 tests, frozen at c3e92ab before the implementation); .github/workflows/run-signal.yml (reconcile job); docs/ops/run-requests.md

**What was asked:** let a participant that can push a branch, but not dispatch a workflow (#1086), raise a ready signal without a PR, master commit or new credential per run.
**What I found:** the deployed runner needs no change: it only reads `ready/<recipe>` and `hold/<recipe>`. So the bridge is a writer, not a reader. A request is one commit on `refs/heads/run-request/<recipe>/<12 hex>`, holding a strict `request.toml`, published with `python3 bench/run_requests.py publish`. A scheduled job in run-signal.yml (every 15 min, plus `signal: reconcile`) runs only the default branch's code. It reads requests as data and materialises the tags only when every gate holds at that moment:
- the queue line on master;
- a queue claim on master, open, with no needs_louis;
- a live `hold/<claim>` whose session matches;
- the commit on the branch, and the PR head exactly the commit;
- the named review at that head, and no CHANGES_REQUESTED at it;
- every check run passed or skipped (CLAUDE.md's macOS outage rule applied), and the commit status success or absent;
- no `hold/` tag; `ready/` unset, already that commit, or the request's `supersedes`;
- no receipt.
**Atomicity and replay:** activation is one `--atomic` push of `ready/<recipe>` (leased on its old value) and an annotated receipt `run-receipt/<recipe>/<sha>`. The receipt is the tombstone: no request, whether replayed, old or new, re-signals a recipe at a commit after it was cleared. A deliberate repeat stays a manual dispatch. Withdraw by fast-forwarding the request. Hold requests need only a queued recipe.
**Tests:** a scratch bare origin and a fake GitHub API cover activation, idempotence, dry run, two recipes published concurrently in one batch, duplicates and every refusal. Refusals tested: hold, queue, branch, ancestry, moved head, stale review, changes requested, failed, pending or absent checks, status error, macOS outage vs failure, API down, lease session, expiry and release, non-queue claims, needs_louis, ready elsewhere, supersedes mismatch, expired, malformed and oversized requests, bad refs. Replay after clear, withdrawal and hold are covered too. A separate check ran against a `--filter=blob:none` clone, the workflow's checkout. test_workstation_queue.py still passes.
**What changed:** new files bench/run_requests.py, bench/test_run_requests.py and docs/ops/run-requests.md; run-signal.yml (schedule, concurrency group, reconcile job with read-only PR/check/status permissions); test.yml (one step). The claim's owned_paths now include the three new files.
**Frozen-test corrections, before any real run:**
- `test_commit_not_on_branch` used master's commit, an ancestor of the branch. It now uses a stray commit.
- `refused()` now asserts the ready tag is unchanged, not absent.
**What's next:** CLAUDE.md's Unattended paragraph still names only dispatch; a pointer to docs/ops/run-requests.md there is left out of this PR because CLAUDE.md is protected. Codex's queue claims (scanner-seam-window-queue, a6b-endpoint-queue) can publish requests once this is on master. The first scheduled reconcile on GitHub is the first real test of the API path and the tag push. Read its summary.
**Caveats:**
- One GitHub account, so neither a requester nor a reviewer's independence can be proven. Authority is the frontier lease, as for a manual dispatch.
- Evidence bundles don't name their commit, so the prior-attempt check beyond receipts and ran/ markers stays with the requester.
- Request refs are never deleted, only aged out at 72 h.
- Not yet run on GitHub or the i9.
