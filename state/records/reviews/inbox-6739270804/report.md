# Service core: buy default evidence before another tuning run

Inbox [6739270804](https://github.com/lmmx/mibz2/pull/160#issuecomment-6739270804), dispatching QUEUED6735266897. Governing specs a4f97714cc14366984015869d9d094d89488ac57; briefing baseline a773853f05fa9e12bb65fb93bf5c07e1bc7fea11. Proposal and evidence audit only.

## Findings committed while review is in progress

J4 is a4w12 versus a4w8: the window effect conditional on threshold4. The joint change versus the established a2w8 baseline is J, reported 1.027 [0.993,1.063], not J4's 1.049 [1.017,1.082]. LA4×J4 point estimates give .979×1.049=1.026971, consistent with J after rounding; interval endpoints cannot be multiplied. #271's judged LA3/LA4 result remains VALID INCONCLUSIVE.

The original PREP STARVATION → FINISH BACKLOG explanation is not a demonstrated critical-path transition. The corrected journal withdraws the idle-label inference: labels sampled at wait start can charge time to the wrong predicate. Queue occupancy and thread-life-minus-idle do not establish exposed service CPU or a finishing bottleneck. #293 is the appropriate diagnostic dependency.

#299's draft at 5738616e58542cc7af2d6e5eeb906a4070abcb32 preserves old runtime3e201246 and six arms×20 rounds=120 decodes. It adds a changed measurement condition (in-region inherited perf counters), not the guarded production candidate. A third old-runtime reading is not the shortest default-adoption path. Prefer a separately registered guarded, current-baseline candidate comparison; preserve the old registration and both historical results.

Master's public default at a773853f selects only Workers or Teams; its defaulted branch selects pipelined3x3, not service. A service Plan is proposed, not present. The source's physical-core detector already reads allowed affinity CPUs: test that implementation and quota/topology edge cases, rather than claiming it counts the whole host unconditionally.

Remaining work: explicit run budget, RSS/entry-point/safety closure and ranking against C29/C43.
