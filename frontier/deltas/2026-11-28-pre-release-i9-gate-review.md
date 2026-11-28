claim: pre-release-i9-gate
by: claude-code
outcome: partial
evidence: PR #1256 (author head 653048a0, reviewed and integrated by a different agent); bench/release_preflight.py; bench/test_release_preflight.py (19 passed: 18 frozen + 1 added; cold second review of the widening); .just/release-preflight.just

**What was asked:** Independently review the release gate (`deploy.yml` and `ship.just` fail-closed on `bench/release_preflight.py verify`) and the frozen recipe. Then integrate it.
**What I found:**
- **Sound, as written:**
  - The verify order holds: clean tree, exact head == origin/master == dispatched SHA, previous `vX.Y.Z` by ancestry, live `needs_louis`, metadata-only mode, committed owner waiver bound to exact SHAs, receipt, §5c freshness, and a bundle re-judged from origin/evidence with its sidecar and digest.
  - The statistics are a paired log-ratio Student-t 95% interval, and a regression is lo95 > 1 at every judged thread count.
  - The PERFORMANCE-POLICY §2 secondary-wall and RSS bounds, the i9 topology and the toolchain parity all check out.
  - The verify runs before any bump, tag or push in all four ship recipes and in Deploy Release, and Deploy Release has no bypass input.
  - On live master, `status` and `verify` refuse, as they should.
- **Blocking defect, fixed here:** only `docs/` and `frontier/` counted as non-performance. Queueing the preflight itself edits `bench/workstation-queue.txt` after the candidate is pinned, so every receipt was stale on arrival. Master also had 302 `bench/` file-changes in 72 h. The allowlist now covers paths that are not build inputs of the published crate or wheels: `bench/`, `benches/`, `examples/`, `tests/`, `research/`, `.claude/`, site and lint config, and agent docs. `src/`, `python/`, `Cargo.*`, `pyproject.toml`, `uv.lock`, `build.rs`, `rust-toolchain*`, `.cargo/`, all of `.github/`, `.just/` and the `Justfile` stay performance-relevant. A cold second reviewer caught that `just release` (in `.just/ship.just`) edits `Cargo.toml` and `Cargo.lock` before the build, so I had wrongly exempted the recipes at first. A new test, `test_01b`, checks both directions: each build input must be named as the staling path. A mutation check confirms it fails if `.just/` is exempt.
- **Also fixed:**
  - The recipe's rounds go from 10 to 12. ab balances the arms' positions over 2 × arms rounds, so 12 balances both the 3-arm run with a recovery base and the 2-arm guard run. The judged minimum stays 10.
  - The receipt fields `verify` reads are now documented in the recipe's header. They were defined only in code.
- **Not changed (noted):**
  - The recovery arm is found by "recovery" in its binary path. That works, since the recipe names it `ab-recovery_base`.
  - Scanner files count as worker-cost paths, so §5c is demanded for them. That is conservative, but RELEASING's knee clause covers it.
**What changed:** the three files above, plus this delta. Claim back to `open` (`pr = 1256` kept: the integration record), since `done_when` still needs a real run.
**What's next:** `done_when` still needs the first real run, on the 0.5.18 candidate against v0.5.17, with v0.5.16 recovery auto-detected at wikidata 8T and 10T. §5c recalibration must be completed first, since worker-cost paths changed. Then a receipt delta. That run belongs to release-0-5-18 (P1).
**Caveats:** this review did not run the recipe on hardware. The `run` path (materialize, build, `ab run`) was read, not executed.
