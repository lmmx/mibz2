claim: scanner-seam-window-queue
by: antigravity
outcome: done
evidence: #1031 at 3ae44390ade27cf105fc301b702e2eca9ab25fe2; C6 review 6084229302; run-signal.yml run 45953639377; ready/bench-scanner-seam-window = 3ae44390ade27cf105fc301b702e2eca9ab25fe2; #1101

Dispatched the registered pre-adoption i9 run for `#1031` (`build/scanner-seam-window`) under the 9 November standing workstation authorisation and landed the pinned queue line in `bench/workstation-queue.txt`.

- **Pre-dispatch recheck:** Verified `#1031` `OPEN` at `3ae44390ade27cf105fc301b702e2eca9ab25fe2` (matching `refs/heads/build/scanner-seam-window`), all CI checks and `pre-commit.ci - pr` green, independent C6 `PROCEED` (`pullrequestreview-6084229302`), live hold `scanner-seam-window-queue`, zero prior queue history on `master`, zero `ready/` or `hold/` tags, and zero seam-window bundles on `origin/evidence`.
- **Ready signal:** Dispatched `.github/workflows/run-signal.yml` via `workflow_dispatch` (`signal=ready`, `recipe=bench-scanner-seam-window`, `sha=3ae44390ade27cf105fc301b702e2eca9ab25fe2`) -> `HTTP 204`, Actions run `https://github.com/lmmx/mibz2/actions/runs/45953639377` (`success`, 51s).
- **Verified remote tag:** `git ls-remote --tags origin refs/tags/ready/bench-scanner-seam-window refs/tags/hold/bench-scanner-seam-window` returned `3ae44390ade27cf105fc301b702e2eca9ab25fe2	refs/tags/ready/bench-scanner-seam-window` and no `hold/` tag.
- **Queue line appended to `bench/workstation-queue.txt`:**
  `bench-scanner-seam-window    build/scanner-seam-window    args=build/scanner-seam-window,3ae44390ade27cf105fc301b702e2eca9ab25fe2,100    #1031, wikidata x4, 16 paired rounds at 6/8/9/10/12 threads. One registered adoption attempt against the master merge base. See docs/journal/2026-11-27-scanner-seam-window/registration.md.`

`#1031` remains open and unmerged; the `scanner-seam-window` integration holder retains interpretation of the returned bundle and the adoption decision.
