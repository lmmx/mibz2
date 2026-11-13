# Guarded small-spine writer: candidate and qualification

Frozen before candidate benchmark data, 13 November 2026. Coordinator queued
6851728088, following completed screen6850979995 / PR378. One candidate only.

Question: can an all-small guard in the existing marked-origin walk remove the
pattern writer's unnecessary metadata work while retaining the Wikidata gain?
Decision: apply the standing paired CI estimator and PERFORMANCE-POLICY.md to the
complete reader; do not call saved visits or a kernel result a decoder speedup.
Integration: one ordinary CI run decides this candidate; preserve failures and
return the disposition to the Coordinator. No secondary rerun to obtain a pass.

## Source and scope

Base master3e7e294cc10f29495e9828181345d74233c549d4, incorporating corrected
Cells d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2 only on this isolated branch.
The merge commit before the candidate preserves both histories. PR365 and master
are unchanged. The production delta is one scalar guard in `write_cells`.
Long-factor blocks retain all original operations; all-small blocks omit only
position initialization, two rule scans and the final long-only marked walk.
No extra classification traversal, decoder defaults, API, team/cap/threshold,
admission, induction, original-row phase, RLE1, CRC or refusal change.

Screen evidence: pattern22/22 admitted blocks all-small, Wikidata196/196 admitted
blocks long-bearing. This suggests eligibility, not a measured bottleneck or proof
that the guard cost is free. Count and charge the extra comparisons/ANDs in every
iteration of the existing origin walk; no per-copy clocks or production tracing.

## Ordinary CI comparison

Keep the existing default-head versus master paired benchmark and its gates,
datasets, thread counts, round schedule and estimators unchanged. Those results
must not be confused with the separately reported experimental Cells comparison.

The existing experimental arm's ten paired rounds compare `mibz2@bin/ab` with
`mibz2-cells` at one thread, hooks off. Retain its schedule, settling and complete
reader/drain semantics. Its seven inputs are the Optimizer's pinned identities
from historical generator/text tree57f51bbc0ca62ac3ee1cc148a1010842eea6222a,
scale4, libbzip2 1.0.8 level9. The extra generator runs outside the timed region;
the text corpus is never taken from moving master. Exact compressed hashes and
the ordered text manifest are checked before timing. No extra measured arm,
custom estimator, i9 run or140+21+2 programme is admitted here.

Report each family's wall, CPU and peak RSS with the existing paired 95% log-t
intervals, named runner/CPU/compiler/source and archived raw rows. Keep missing or
invalid memory evidence explicit; screen RSS includes diagnostic scratch and is
not a substitute. A primary gain beyond noise, no primary loss, policy-permitted
secondary costs and credible memory are necessary for a performance recommendation.
The original #365 DEFER/slice-one gate is an additional integration dependency,
not silently replaced by this candidate plan. No default flip or merge follows
from an experimental-arm result alone.

The seven input oracles/identities were established in PR378. Before the experimental
timing step, one explicit ignored test compares both candidate-build readers against
libbzip2 byte-for-byte on those seven inputs (21 untimed decodes, 300 second deadline).
This is correctness validation, not a new measurement schedule. Candidate-specific
unit/differential tests include both guard outcomes, 16/17 boundary, poisoned
output, short cycles and mixed buffer reuse, malformed/CRC/concatenated streams,
forced-route fuzzing, Miri and byte-correctness mutation kills. Independent
Researcher B5 review of the changed argument remains required. MONODROMY's
existing mathematical proposal is reused; no theorem approval is claimed.
