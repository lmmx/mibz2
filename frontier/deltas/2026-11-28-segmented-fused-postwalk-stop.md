claim: segmented-fused-postwalk
by: antigravity
outcome: dropped
evidence: PR #1183 (56049aeb / 3c80d181); CI runs 46084269815 and 46085315398; docs/notebook/2026-11-28-segmented-fused-postwalk-rss-pricing.md

Closed PR `#1183` unmerged under RULES C2 and recorded a scoped `STOP` in
`frontier/dead-ends.tsv` while admitting `segmented-fused-postwalk-rescue`
(`idea -> open`).

- **Frozen verdict preserved (C2):** PR `#1183` (`56049aeb`, CI run `46084269815`)
  passed exactness, B5 review, and its timing gate (`text` `-4.2%` 1T, `-8.9%` 2T,
  `-6.7%` 4T, all faster; `runs`/`onebyte` `-7..-20%`; `wikidata` faster at every
  thread count), but exceeded the frozen `<= 1.02x` point RSS bound on `pattern`
  4T (`+4.3%`) and `wikidata` 2T (`+3.0%`). That frozen `STOP` stands permanently.
- **Root-cause pricing (`#1290`, `#1289`):** CI run `46085315398` on identical
  `src/` (`3c80d181`) measured `pattern` 4T `-0.6%` with every 1T cell flat or
  lower (`text` `-4.4%`, `pattern` `-4.8%`), and local 20-round paired + A/A runs
  confirmed `pattern` 2T/4T is bimodal queued-output jitter inside `#1273`'s null
  bands (`docs/notebook/2026-11-28-rss-bound-calibration.md` §2.2/§2.4).
- **Real residual defect & handoff:** Both CI runs showed `random` 1T slower
  (`+1.5% [+0.8, +2.2]` and `+2.2% [+1.4, +3.0]`), sitting on the `+2%` secondary
  bar. Follow-on `segmented-fused-postwalk-rescue` is admitted (`idea -> open`) to
  run the route census on `random.bz2`, remove the `random` 1T cost cleanly, and
  re-test under a freshly frozen `#1273` §5 calibrated rule (not reusing runs
  `46084269815` or `46085315398`). Head `3c80d181` remains preserved on
  `build/segmented-fused-postwalk`.
