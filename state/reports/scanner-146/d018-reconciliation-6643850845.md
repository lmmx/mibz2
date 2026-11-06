# D018 reconciliation

Trigger: [6643850845](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643850845).
Read [D018 at 285f76d4](https://github.com/lmmx/mibz2/blob/285f76d4daf16d494c05e4ec03be756b454cbee3/docs/journal/compact30-guardian-shadow/decisions.md#d018--walk-archive-reconciled-ci-causality-remains-unestablished).

D018's CI interpretation agrees with the independently published [Researcher pivot review](https://github.com/lmmx/mibz2/blob/a8fe7261507485f5d8ba836e47d6985c5e125b14/docs/journal/compact30-adversarial/scanner-146/pivot-review-6643744616/report.md): no detected multi-worker wall regression is not equivalence or causal identification; the own-baseline denominator explains amplification arithmetic without erasing the scaling signal. The Researcher independently parsed block 1 only; D018's wider 14-profile reconciliation is the Guardian's reported replay, not relabelled as this reviewer's work.

The “independent invariance review remains open” status is now stale as a request for a Researcher assessment: the earlier [schedule assessment](https://github.com/lmmx/mibz2/blob/825af2e73fac1b1ff2559a5883031766b6b53f7e/docs/journal/compact30-adversarial/scanner-146/deferred-handover-6643481606/report.md) and the pivot review supply it. They do not certify all Rust safety or close a Guardian gate.

Specifically, for an already-stepped unmarked j>k, immutable T and distinct output buffers allow its ordinary step to commute before earlier handovers. next_unset sees only seg/next, unchanged by that step. The tail's p/end pre-check is an observer, so “no observer” and literal intermediate-state equality are incorrect premises. If j's step fills its buffer, the new tail grows the already-written prefix; the old tail performs the same step then grows. If the advanced row is START, both finish at the same successor with identical bytes. Allocation/growth chronology may differ; the argument concerns resulting segment bytes and next[], with the stated bounds.

The test evidence qualification stands. Counters pooled over lane counts do not show each witness at both counts; the archive predates the new test. The previously recommended later-unmarked mutant (advance row but retain p=off) and explicit exact-full/successor-START witnesses complement the reported pending-marked mutants. No new native/Miri run was performed.

Disposition: use the completed Researcher simulation argument rather than await an absent assessment; retain the explicit safety-evidence limits and performance gates. No implementation, PASS, merge or authority change.
