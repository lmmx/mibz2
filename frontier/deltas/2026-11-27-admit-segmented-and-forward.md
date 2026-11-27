claim: none
by: antigravity
outcome: done
evidence: frontier/claims/segmented-fallback-build.toml, frontier/claims/forward-emission.toml, docs/journal/2026-11-27-explore/segmented-fallback-dissection.md, docs/journal/2026-11-26-synthesis/fourth-pass.md

**What was asked:** Admit `segmented-fallback-build` (P2) and `forward-emission` (P3) from `idea` to `open` within `max_open_claims = 12` now that `mtf-fold-build` and `segmented-fallback-dissection` landed and open claims dropped to 10.
**What changed:** Moved `segmented-fallback-build` (`src/bwt.rs`, `src/runs.rs`, `segmented-fallback`, immediately disjoint from all live holds) and `forward-emission` (`src/open.rs`, `src/open/outq.rs`, ready once `forward-rle1-crc-fusion` PR #1045 releases `emission-walk`) from `idea` to `open` (12 open claims total, <= 12). No `stop_test`, `surface.tsv`, or protected files touched.
**What's next:** Independent participants discover and claim eligible work via `frontier.py brief`.
**Caveats:** Admission only; both are striated builds requiring independent review and merge.
