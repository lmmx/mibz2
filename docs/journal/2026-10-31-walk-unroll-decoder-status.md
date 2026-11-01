# The unrolled room walk: where it stands (2026-10-31)

**Closed as completed:** the second exploratory CI run 41243689201. Its records
(`2026-10-31-walk-unroll-decoder-ci-41243689201/`) and the first run's
(`…-ci-41235849514/`) are preserved unchanged.

**The maintainer's reading, accepted:**
- The harness-lifecycle validation, and the cross-architecture results.
- **The candidate is a successful stage-level optimisation with mixed full-decoder
  evidence:**
  - strong Wikidata gains on Zen 5 (+18%) and Zen 3 (+6% to +8%);
  - limited gains on Zen 4 (+2%);
  - neutral on Intel Wikidata;
  - significant `records` regressions on three hosts: Zen 3 −5%, Zen 4 −8%, Intel −7%.
    Their intervals exclude the 3% tolerance.

**Status:**
- **Research only.** It is off by default, unmerged, and not combined with P74-C.
- Not retuned, no gates changed, no CPU dispatch.
- **The registered i9 walker trial is not run.** The protocol, amendments 1–2, the freeze
  and the pin (`df90ba8`, head `f5c492d`) stand ready if it is later authorised.
- The exploratory results don't replace its verdict. If its primary passes, the control
  regressions above stay visible beside it.
- **Open question, not investigated:** what explains the `records` slowdown. The
  candidates are:
  - segment-boundary behaviour;
  - prefetch effectiveness;
  - instruction-cache pressure from the larger unrolled function;
  - the accepted decoder inlining difference (W4 (c)).

**Next decision, after the P74-C i9 result:** run the registered i9 walker trial, or define
a new, explicit research hypothesis.

## 2026-11-01: closed under its registered NEGATIVE verdict

- **The registered i9-10900K run** is NEGATIVE: the primary, wikidata-20261026 at w = 4,
  is B/A 1.000 [0.987, 1.012], and the CPU gate fails on `records` (1.018 [1.010, 1.026]).
  Correctness, the structural audit, affinity and A/A all pass.
  - Records: `2026-11-01-walk-unroll-decoder-i9-10900K/`, with the results, the
    read-only analysis, the exact binary and the raw data.
- **The arm is archived at decoder level.** No rerun, tuning, threshold change, default
  change, merge or combination with P74-C.
- **Kept as research evidence:**
  - the implementation (`research/walk-unroll`, `research/walk-unroll-decoder`);
  - Astra's pack;
  - the fidelity comparison (`2026-11-01-walk-unroll-astra-fidelity/`);
  - both exploratory CI runs;
  - the i9 archive.
