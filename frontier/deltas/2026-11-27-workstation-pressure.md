claim: workstation-pressure
by: antigravity
outcome: review
evidence: tests/test_frontier.py (53/53 pass, incl. 9 frozen TestWorkstationPressure fixtures); frontier/tools/frontier.py; live brief/status on origin/master 1a61f5b3; #1078; #1086; #1089

Implemented derived repository-local workstation replenishment pressure in `frontier/tools/frontier.py` (`brief` and `status`), reusing only existing state (`origin/master:bench/workstation-queue.txt`, `origin` `refs/tags/ready/*` & `hold/*`, `origin/evidence` bundle `commit.txt`, claim TOMLs, `dead-ends.tsv`, live `hold/*` leases, open flags, open PRs) and existing authority. No new scheduler, approval layer, runner dispatch, or spending was added; `check()` remains 5 checks (C7).

**Mechanism:**
- `read_workstation_queue_state()` checks `origin/master:bench/workstation-queue.txt` entries with active `ready/*` (unheld by `hold/*`) against archived `origin/evidence` tarballs (`commit.txt`/`commits.txt`) via a reverse watermark plus recent-tail check, failing closed to `unknown` on any fetch, git, or tarball error.
- `Physical i9 state: unknown` is always reported because `bench/run-queue.sh` keeps `queue.lock`, `ran/*`, and multi-repo runner state locally under `~/.local/state/` on Louis's workstation; an empty repo queue never infers physical i9 idleness.
- When repository-local starvation and live PR discovery are both verified, `workstation_pressure()` derives ephemeral `effective_priority = 1` (never mutating declared claim TOML priorities) for the shortest eligible unfinished workstation-validation path: review-ready PRs first, then unheld non-conflicting `run` claims, then unheld non-conflicting upstream `build` claims. Blocked, dropped, done, `idea`, flagged, `needs_louis`, dead-ended, leased, and path/mechanism-conflicting claims are never promoted or bypassed.
- Per `#1086`, `brief --can` separates runtime capability requirements (`needs`) from standing permission: missing capabilities surface as `Capability-blocked handoff (needs ...)` for capable participants while exposing the next eligible work instead of sending incapable workers to a denied action.

**Verification & follow-up:**
- All 9 frozen fixtures (`test_wp1`..`test_wp9`, committed before implementation per C2) and all 44 existing frontier stress tests pass (53/53 `OK`).
- On live `origin/master` (`1a61f5b3`), `brief` and `status` verified `Repository queue: starved (all 50 ready/* queue entries already have archived evidence)`, `Physical i9 state: unknown`, surfaced review PRs (`a6b-endpoint-run` `#1085`, `scanner-seam-window` `#1031`), respected live leases (`scanner-seam-window-queue`, `forward-emission`) and hold overlap (`a6b-endpoint-queue` blocked on `workstation-queue` / `bench/workstation-queue.txt`).
- Recorded optional physical runner telemetry in `frontier/claims/workstation-runner-telemetry.toml` (`status = "idea"`, RULES D gated).
- Stopped at `status = "review"` awaiting independent review (`frontier/tools/` protected path and `done_when`).
