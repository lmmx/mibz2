claim: none
by: antigravity
outcome: done
evidence: frontier/claims/bwt-digram-return-map.toml, frontier/claims/worker-l3-footprint.toml, frontier/config.toml (max_live_holds = 6), #992

**What was asked:** Admit two ready smooth-zone `idea` claims to `open` (within `max_open_claims = 12`) now that `max_live_holds` is 6, so parallel sessions can fill available reservation capacity on disjoint mechanisms.
**What I found:** Master had 7 open claims (`branch-protection`, `explore`, `migrate-rules`, `mtf-fold-build`, `permutation-patterns`, `synthesise`, `warm-paired-oracle-k1`) and 3 of 6 live hold slots free. Two high-priority smooth probes have frozen stop tests and disjoint mechanisms from all live holds:
- `bwt-digram-return-map` (P2, probe, `bwt-digram-contraction`): follow-on constructor probe opened by #992 (`ideal-permutation-structures`).
- `worker-l3-footprint` (P3, probe, `delivery`, `rle1-crc`): L3 attribution probe from `third-pass.md` deciding whether `READY_BYTES` is an L3 lever before `delivery-l3-working-set`.
**What changed:** Moved `bwt-digram-return-map` and `worker-l3-footprint` from `idea` to `open` (9 open claims total, <= 12). No `stop_test`, `surface.tsv`, or protected files touched.
**What's next:** Independent sessions can discover and atomically claim eligible open items in parallel up to the 6-hold cap.
**Caveats:** Admission only; no measurements or verdicts.
