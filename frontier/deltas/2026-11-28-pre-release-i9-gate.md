claim: pre-release-i9-gate
by: antigravity
outcome: review
evidence: bench/release_preflight.py, bench/test_release_preflight.py (18/18 passed), .github/workflows/deploy.yml, .just/ship.just, .just/release-preflight.just, Justfile

**What was asked:** Enforce RELEASING §3.5's physical i9 pre-release regression gate before any version bump, tag, push, or publish, reusing the paired `ab` harness, `origin/evidence` archival/SHA-256 verification, workstation queue, and `docs/journal/decisions.md` owner-decision mechanism, without touching `frontier/tools/` (`#1241`).

**What changed:**
- **Frozen fixtures first (commit `888b3039`):** Committed the 18-case fixture matrix in `bench/test_release_preflight.py` before implementation, covering passing verified receipts, simulated 0.5.17-style 8T/10T `<3%` primary regressions with neutral CI, primary `text` regression, byte-parity failures, PERFORMANCE-POLICY §2 secondary wall and peak RSS bounds (including WideLazyCells-style valid trades vs >64 MB / multi-corpus / secondary RSS rejects), missing/unread/corrupt/digest-mismatched/wrong-machine/self-declared-PASS bundles, wrong base, stale inputs, dispatch master race, non-performance metadata-only releases and post-preflight metadata reuse, committed owner waivers in `docs/journal/decisions.md` vs unauthorised env/agent/uncommitted bypasses, live `needs_louis` blockers, shallow `--depth=1` clones, §5c worker-cost recalibration ordering/freshness, corrective recovery against the last known good tag (`v0.5.16` auto-detected from `bench/release-scaling/`), and pre-bump/tag/push abort in `deploy.yml` and `.just/ship.just`.
- **Gate & runner (`bench/release_preflight.py`, `.just/release-preflight.just`, `Justfile`):** Standard-library-only fail-closed verifier (`verify`), offline tarball/dir judge (`judge`), read-only status (`status`), and shallow-safe workstation recipe (`bench-release-preflight` -> `run`, bundled via `bench/bundle.sh`). Auto-detected the live `v0.5.17` regression against `v0.5.16` at `wikidata` 8T and 10T from `bench/release-scaling/`.
- **Fail-closed release wiring (`.github/workflows/deploy.yml`, `.just/ship.just`):** Added `python3 bench/release_preflight.py verify --expected-head "${{ github.sha }}"` immediately after checkout in `deploy.yml` (no bypass workflow input) and before version bumps/tags/pushes in all four `.just/ship.just` recipes.

**Verification:**
- `pytest -v bench/test_release_preflight.py`: 18/18 passed.
- `python3 bench/test_workstation_queue.py` (4/4) and `python3 bench/test_submit.py` (7/7) passed.
- `ruff check` and `ruff format --check` clean; `.github/workflows/deploy.yml` YAML valid; `frontier.py check` clean.

**Next:** Independent O2 review of the `.github/workflows/deploy.yml` gate enforcement and frozen recipe before merge (lease `de98815eb37e` retained in `review` state for handoff). Once merged, run `bench-release-preflight` on the next eligible 0.5.18 candidate (`#1246` + `#1234`) against `v0.5.17` with `v0.5.16` recovery.
