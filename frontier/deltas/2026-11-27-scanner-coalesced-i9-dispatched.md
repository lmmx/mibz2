claim: scanner-coalesced-u8-i9-queue
by: antigravity
outcome: partial
evidence: #1126 at 96e574a7b3b12d3dfa86e85f02538851e5a1e6e7 (squash c3db2e39); review 6084937446; queue row #1120 (line 68); withdrawn request 2e1e9ffd; run-signal.yml run 45963860210; refs/tags/ready/bench-scanner-coalesced = 96e574a7b3b12d3dfa86e85f02538851e5a1e6e7; #1084

Dispatched the registered i9-10900K keep/revert run for `bench-scanner-coalesced` (`#1034` squash `5ed20c1d` vs first parent `5c7fe5d2`) under the 9 November standing workstation authorisation against the existing static queue row on `master`.

- **Pre-dispatch recheck:** Verified atomic live hold `scanner-coalesced-u8-i9-queue` (`session = 888b8fb44679`, `frontier.py verify` clean) after `hold/a6b-endpoint-successor-queue` released; `refs/heads/bench/scanner-coalesced == 96e574a7b3b12d3dfa86e85f02538851e5a1e6e7` (PR `#1126` merged at `c3db2e39`, independent review `6084937446`, all CI checks + `pre-commit.ci - pr` green); single static queue row already on `master` at line 68 of `bench/workstation-queue.txt` (`#1120`, untouched); original `#1105` request `refs/heads/run-request/bench-scanner-coalesced/edb114aad561` (`2e1e9ffd`) confirmed `action = "withdraw"`; zero prior `ready/`, `hold/`, or `run-receipt/` tags and zero coalesced bundles on `origin/evidence`.
- **Ready signal:** Dispatched `.github/workflows/run-signal.yml` via `workflow_dispatch` (`signal=ready`, `recipe=bench-scanner-coalesced`, `sha=96e574a7b3b12d3dfa86e85f02538851e5a1e6e7`) -> Actions run `https://github.com/lmmx/mibz2/actions/runs/45963860210` (`success`, `signal` job 42s).
- **Verified remote tags:** `git ls-remote --tags origin refs/tags/ready/bench-scanner-coalesced refs/tags/hold/bench-scanner-coalesced` returned `96e574a7b3b12d3dfa86e85f02538851e5a1e6e7	refs/tags/ready/bench-scanner-coalesced` with no `hold/` tag.
- **Queue row on `master` (line 68, `#1120`):**
  `bench-scanner-coalesced      bench/scanner-coalesced    Frozen #1034 keep/revert comparison against its first parent, seven corpora, ten paired rounds, pinned scanner, serial phases and construction replay at eight and ten workers (#1105).`

`scanner-coalesced-u8-i9-queue` remains `open` until the immutable bundle lands on `origin/evidence`, its SHA-256 is verified, and registration §3.2–3.3 is applied verbatim (preserving `#1084`'s open four-thread CI objection). Releasing the shared queue hold on merge so `bench/workstation-queue.txt` is not blocked while waiting for hardware.
