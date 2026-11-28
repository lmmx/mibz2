claim: release-0-5-18
by: antigravity
outcome: partial
evidence: tag v0.5.18 & py-0.5.18 at edf2ceafc292afba4312a8d38115b06cd676ba54; deploy.yml 46145821994; CI.yml 46145936623; run-signal.yml 46146191252 & 46146194190; PR #1360

**What was asked:** publish `v0.5.18` (`docs/RELEASING.md` §4 & §5) after all §3.2, §3.3, §3.5, and §5c gates passed on `07a1b1d0`, queue post-release i9 scaling and crossover runs at the exact tag commit, and record the measured `CHANGELOG.md` entry.

**Publication verified (`RELEASING.md` §4):**
- **Tag commit (`v0.5.18`, `py-0.5.18`):** `edf2ceafc292afba4312a8d38115b06cd676ba54`
- **Deploy Release (`deploy.yml`):** run `46145821994` green (`release_preflight.py verify --expected-head 07a1b1d0` PASS, `just release patch` pushed `edf2ceaf` and `py-0.5.18`, triggered `CI.yml`).
- **Release CI (`CI.yml` on `py-0.5.18`):** run `46145936623` all 9 jobs green (crates.io Trusted Publishing, 5 `abi3-py310` wheels, sdist, PyPI attestation & publish).
- **crates.io:** `mibz2 0.5.18` live at `https://crates.io/crates/mibz2/0.5.18` (`yanked=false`, published `2026-11-28T12:42:55Z`).
- **PyPI:** `mibz2 0.5.18` live at `https://pypi.org/project/mibz2/0.5.18/` (all 5 wheels: Linux x86_64/aarch64, macOS x86_64/arm64, Windows amd64 + `mibz2-0.5.18.tar.gz` sdist).
- **GitHub Release & docs.rs:** `https://github.com/lmmx/mibz2/releases/tag/v0.5.18` published `12:42:59Z`; `https://docs.rs/mibz2/0.5.18/mibz2/` returning HTTP 200.

**Post-release i9 runs queued (`RELEASING.md` §5 & §5a, standing authorisation `CLAUDE.md` line 146):**
- `ready/bench-release-scaling` -> `edf2ceafc292afba4312a8d38115b06cd676ba54` (`run-signal.yml` `46146191252`, verified via `git ls-remote --tags origin`).
- `ready/bench-crossover` -> `edf2ceafc292afba4312a8d38115b06cd676ba54` (`run-signal.yml` `46146194190`, verified via `git ls-remote --tags origin`).
- Queue-only PR #1360 (`8fecbabb`) updated `bench/workstation-queue.txt` (`test_workstation_queue.py` 4/4 OK, `coord.py head-sha` verified before and after).

**What changed in this PR:**
- Added measured `0.5.18` release notes to `CHANGELOG.md` covering all 8 software PRs (`#1234`, `#1246`, `#1117`, `#1268`, `#1294`, `#1274`, `#1312` feature-gated, `#1335`), §3.2 CI bench (`46131501937`, AMD EPYC 9V45), §3.5 i9 preflight VALID PASS (`c8489a45`), §5c recalibration (`f8b21e4c`), and §3.3 feature review (20 research features, none removed).
- Recorded this publication delta; `release-0-5-18` stays open until the post-release `bench-release-scaling` and `bench-crossover` bundles return on #145 and are recorded in `bench/release-scaling/` and `CHANGELOG.md`.

**What's next:** §2a candidate freeze is lifted; await `bench-release-scaling` and `bench-crossover` bundles for `v0.5.18` (`edf2ceaf`), record them on master and verify the live History page, then close `release-0-5-18`.
