claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-synthesis/eighth-pass.md

Frontier bookkeeping per eighth-pass synthesis (`562fb844`):
- Appended run-encoded `OutQ32` scoped STOP row from `#1059` (`63e94c19`: `-20.7%`
  `A5+A6a` but `+94..+118 ms` `A6c`, net `+34.6%..+40.6%` slower) to `frontier/dead-ends.tsv`.
- Bumped `frontier/claims/forward-emission.toml` from `priority = 3` to `priority = 2`
  (single largest worker lever at `+208..+235 ms` 1W / `+28.4..+32.8 ms/w` 8W paid in
  `#1057`, sequencing behind `mtf-fold-l1-batch-redesign` on `src/open.rs`; `stop_test`
  untouched).
