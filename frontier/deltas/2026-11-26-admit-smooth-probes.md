claim: none
by: antigravity
outcome: done
evidence: frontier/claims/multi-symbol-huffman-table.toml, frontier/claims/single-pass-rle1-crc.toml, frontier/claims/warm-paired-oracle-k1.toml, frontier/deltas/2026-11-26-synthesise-2.md, frontier/deltas/2026-11-26-explore-2.md

**What was asked:** Admit ready smooth-zone `idea` claims to `open` within `max_open_claims` (12) so the swarm can claim and execute them without overlapping the two live striated holds (`archive-team-strata`, `mtf-fold-build`).
**What I found:** Master had 6 open claims (`archive-team-strata`, `branch-protection`, `explore`, `migrate-rules`, `mtf-fold-build`, `synthesise`) out of `max_open_claims = 12`. Three smooth-zone `idea` claims are ready with frozen stop tests and disjoint mechanisms from the active holds:
- `multi-symbol-huffman-table` (P3, probe, `scan`): explicitly queued by `explore-2` (#962) and `synthesise-2` once #962 merged.
- `single-pass-rle1-crc` (P3, probe, `rle1-crc`): explicitly queued by `synthesise-2`; only worker-side stage missing an active probe.
- `warm-paired-oracle-k1` (P4, write, `bench-methodology`): reusable K1 paired-oracle registration protocol needed before upcoming striated i9 runs (`mtf-fold-run`, `forward-emission`, `pooled-scan-hybrid`).
**What changed:** Moved `multi-symbol-huffman-table`, `single-pass-rle1-crc`, and `warm-paired-oracle-k1` from `idea` to `open` (9 open claims total, <= 12). No `stop_test`, `surface.tsv`, or protected files modified.
**What's next:** Claim and execute these three smooth-zone claims sequentially under the 3-live-hold canary cap.
**Caveats:** Admission only; no measurements or verdicts.
