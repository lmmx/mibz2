claim: dense-cap-precheck
by: antigravity
outcome: partial
evidence: frontier/claims/dense-cap-precheck.toml; docs/journal/2026-11-27-explore/route-census.md; #1023; #1010 (squash cf242fe7, hold/mtf-fold-build released)

**What was asked:** Admit `dense-cap-precheck` from `idea` to `open` now that `src/open.rs` is unheld (following #1010 merge and `hold/mtf-fold-build` release) and open claims stand at 10/12.
**What changed:** Set `frontier/claims/dense-cap-precheck.toml` `status = "open"` (11/12 <= `max_open_claims = 12`). Frozen `stop_test` untouched.
**What's next:** Claim `dense-cap-precheck`, implement the dense-band record-count precheck in `src/open.rs`, and verify against the frozen stop test (exactness + wikidata 233 Open + dense-band refused blocks <= 1.05x Segmented + wikidata 1w wall <= 1.005x baseline).
**Caveats:** Admission only; no code edits in this PR.
