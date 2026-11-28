claim: continuation-runbook
by: claude-code
outcome: partial
evidence: PR #1356; fixtures frozen at f30e744; frontier/tools/continuation.py; tests/test_continuation.py (34 pass); frontier/CONTINUATIONS.md; live wait/ refs on origin

**What was asked:** let each dependency chain continue itself, with no coordinator, global watcher or dispatcher. There should be no duplicate waits or executions on one continuation, departed peers must be recoverable, waits should be bounded and cheap, ordering read-only, and timings honest.
**What I found:** one compare-and-swap ref per gate is enough. `wait/<claim>--<action>` holds the exact pin, the next action, a TTL'd waiter, a TTL'd acquirer and the timings. Distinct chains share nothing. A poll writes nothing. A wait writes to start, once per half-TTL and on a change. One real CAS hole turned up: git accepts a push of the value already on the remote as up to date without checking the lease. Each write now carries a nonce.
Live, on origin and GitHub, 28 Nov:
- (a) A second peer watching a watched key got `duplicate` at once (exit 11) and polled nothing.
- (b) Pushing to #1356 moved its head. The old-head watcher reported `superseded` after 2 polls (exit 12) and did not wake as ready.
- (c) Two real chains (#1356 CI; #1341 merge) waited at the same time from separate processes.
- (d) A read-only probe of v0.5.18's pins read: `ready/bench-release-preflight` at 7be50513, ready with no authoritative set time. Its evidence: no bundle yet. `recalibrate-0518` at b4d25f64: `returned-unread`, evidence commit 11:11:05Z. `v0.5.18`: not set.
- (e) The integration's PR-body footer turned `no-ai-attribution` red at 775de062. The watcher reported `failed`, not ready.
- (f) Live bug, fixed: after a re-run, GitHub still lists the earlier failed attempt at the same head, and the probe stayed `failed`. Only the latest run per check name counts now (`summarise_checks`, with a test).
- (g) Live pickup on a green head: recorded in the next commit.
**What changed:** the new module and tests, the runbook, rows in README's files table, a step-5 pointer, and a `frontier.yml` step that runs the new tests. `frontier.py` and `test_frontier.py` are untouched (#1341's hold).
**What's next:**
- An independent review of #1356; the claim requires one.
- After #1341 merges, a minimal follow-up embeds `continuation.py view` in `brief`/`status`. It is declared as the continuation `continuation-runbook--brief-after-1341` on `merged:1341@09abcd99`, so whoever takes it resumes from that record.
- Release sessions can declare their gates now. Example: `evidence:bench-release-preflight@7be50513…` with next "read and judge the preflight".
**Caveats:**
- "Unread" is relative to the continuation's own record. recalibrate-0518 was already read back in #1350, and the probe does not parse deltas.
- A ref pin has no authoritative set time.
- Independence is declared (`--by`), not authenticated, as elsewhere in the frontier.
- The real-store fixture uses a local bare remote. Live behaviour on the GitHub proxy was observed, not fixture-tested.
- `view` is not yet in `brief`.
