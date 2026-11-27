claim: a6b-endpoint-queue
by: antigravity
outcome: done
evidence: #1053 at faa9df70f98d72ece27befebb966a9d9683b6dd9; #1085 at 02e492a7aa9980fa710ae93c1b6e761f2214d18e; recipe review 6084253348; run-signal.yml run 45955683329; ready/bench-a6b-endpoint = 02e492a7aa9980fa710ae93c1b6e761f2214d18e; #1089

Dispatched the registered i9 validation run for the A6b endpoint redesign (`bench-a6b-endpoint` on `bench/a6b-endpoint-redesign`) under the 9 November standing workstation authorisation and landed the queue entry in `bench/workstation-queue.txt`.

- **Pre-dispatch recheck:** Verified `refs/heads/bench/a6b-endpoint-redesign` resolves to exact reviewed commit `02e492a7aa9980fa710ae93c1b6e761f2214d18e` (`#1085`, building reviewed source `faa9df70f98d72ece27befebb966a9d9683b6dd9` from `#1053`), independent recipe review passed (`6084253348`), all exact-head CI checks and `pre-commit.ci - pr` green, live hold `a6b-endpoint-queue` verified, zero prior queue history on `master`, zero `ready/` or `hold/` tags, and zero `a6b` bundles on `origin/evidence`.
- **Ready signal:** Dispatched `.github/workflows/run-signal.yml` via `workflow_dispatch` (`signal=ready`, `recipe=bench-a6b-endpoint`, `sha=02e492a7aa9980fa710ae93c1b6e761f2214d18e`) -> `HTTP 204`, Actions run `https://github.com/lmmx/mibz2/actions/runs/45955683329` (`success`, 45s).
- **Verified remote tag:** `git ls-remote --tags origin refs/tags/ready/bench-a6b-endpoint refs/tags/hold/bench-a6b-endpoint` returned `02e492a7aa9980fa710ae93c1b6e761f2214d18e	refs/tags/ready/bench-a6b-endpoint` and no `hold/` tag.
- **Queue line appended to `bench/workstation-queue.txt`:**
  `bench-a6b-endpoint  bench/a6b-endpoint-redesign  Registered A6b redesign of Open endpoint construction (#1053, recipe #1085), 15 paired rounds on Wikidata x4 and text, including the eight-worker ideal-scanner comparison.`

The original build registration (`docs/journal/2026-11-27-a6b-endpoint-build/registration.md`) governs interpretation of the returned bundle and the adoption decision.
