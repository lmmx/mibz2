# Compact30: the guardian's review of the constructor prototype (report 190)

Kept as relayed. It reviews the prototype at `e003a32`
([2026-11-03-compact30-constructor-prototype.md](2026-11-03-compact30-constructor-prototype.md)),
accepts its correctness checkpoint provisionally, and authorises two untimed tasks: a handoff
for an independent code review, and a draft construction-cost protocol.

## Report 190

This is a successful constructor prototype checkpoint. I recommend advancing to an independent code review and a separately registered construction-cost experiment—not starting another constructor redesign.

Claude's results address the two questions Astra left open: whether the frozen compact30 arrays could be constructed directly, and whether enough rows would use the cheap bulk path to make the design worth measuring.

| | |
|---|---:|
| Primary corpus: rows emitted through complete-record patterns (the bulk path) | 74.34% |
| Other construction paths: fragments and individual rows | 25.66% |
| Construction budget: conservative limit relative to production | 1.067× |

I inspected the constructor branch at `e003a32` and its journal. The source change is larger than the 13 lines mentioned in Claude's summary: those 13 lines wire in a new 1,963-line `examples/c30walk/ctor.rs` module. That's a reasonable research-only organization, but the new module deserves an independent review before we authorize performance measurement.

The evidence is nevertheless encouraging:

- The recorded release and synthetic suites compare both arrays byte-for-byte with the oracle.
- The untouched walker produces equivalent output from the constructed arrays.
- RAW canonicalization and promotion have dedicated edge-case coverage.
- The compiler preserves a recognizable complete-record bulk-store loop.
- The constructor avoids the oracle's full successor table.

None of those tests establishes construction speed. In particular, 74% fast-path coverage does not mean the constructor is 74% faster—or even that it is faster at all.

### What I would do next

Authorize Claude to draft the construction benchmark protocol, while the researcher independently audits the new constructor. These can happen in parallel. Neither requires another i9 run yet.

The independent review should concentrate on the parts most capable of invalidating the result: initialization of the unsafe record arena, RAW promotion after partially written groups, in-place RAW permutation, release-mode bounds assumptions, and equivalence with the oracle. Miri's successful small-case run is useful, but it did not exercise wide promotion.

The construction protocol should then compare the actual `Ctor::build::<false>` against production's `build_one_pass`, including all required initialization, allocation, RAW promotion, ordering and finalization work. It must distinguish reused-capacity operation from cold or growing allocations, and must not allow oracle construction or prepared images to distort the comparison.

We should keep the original 1.06714× conservative cost ceiling as the investment screen. A future integrated decoder comparison must still be separately approved, even if construction meets that budget.

### The instruction

I have reviewed the constructor prototype checkpoint at `e003a32`.

The recorded correctness and structural results are sufficient to advance to independent review and construction-benchmark protocol preparation. The result is promising, but 74.34% complete-record coverage is not a speed measurement.

Please proceed with two bounded tasks, neither involving timing:

1. **Prepare an independent reviewer handoff.**
   Ask the researcher to inspect the actual new `examples/c30walk/ctor.rs` module, not just its journal. Focus on unsafe record initialization and `set_len`, release-mode bounds assumptions, RAW promotion and canonicalization, reuse after changing block sizes, and exact oracle equivalence.
   The source adds a substantial 1,963-line module even though `c30walk.rs` itself changes by only 13 lines. Treat that module as the audit surface.
   Record Miri's actual coverage limit: the quick run did not exercise wide promotion. Do not claim exhaustive memory-safety verification.
   No new implementation or benchmarking is authorized by this review assignment.

2. **Draft one construction-cost comparison protocol.**
   Compare the current uncounted direct constructor against production `build_one_pass` on identical last columns and counts.
   Include all constructor work needed to produce usable tables: cursor and bitmap initialization, allocations and capacity reuse, run scanning, ordinary and RAW writes, promotion, canonicalization, padding and PARK.
   Require exact output and complete array initialization as correctness gates. Separate warm reused-capacity measurements from cold/growth costs. Report both one-core and four-core worker CPU cost, as well as relevant completion time and memory footprint. Use a frozen compiler and source identity.
   Use the established conservative constructor-cost ceiling of 1.06714× production's build as the investment screen. Make the statistical decision rules explicit before any timing.
   Account for the particularly expensive fallback and fragment paths. Preserve untimed structural counters so we can interpret their influence without adding profiling work during the measurement.
   Do not modify the walker, introduce another constructor variant, integrate into production, or run anything on the i9.

Return the independent-review handoff and protocol draft for guardian approval. The previously validated traversal result remains closed and unchanged.

### One caution about the larger objective

The researcher has identified the next major uncertainty beyond construction cost: real decoder concurrency.

Compact30's prepared walker was about 15% slower on one core but 16% cheaper in CPU time with four simultaneously active workers. Even if the constructor costs less than its allowed budget, the real decoder might not spend enough time in that four-walker regime to benefit.

So the progression remains: validated four-core traversal → byte-identical direct construction → measured construction economics → production-shaped concurrency and end-to-end evaluation. We have completed the first two stages; the third is now ready for protocol design.

The separate checkout/stash issue Claude described should stay outside this research decision. It was sensible not to commit an apparent rollback of master, and there's no reason to touch that stash or clean up further worktrees as part of the constructor experiment.

**My disposition:** constructor correctness checkpoint accepted provisionally, independent code audit next, benchmark protocol preparation authorized, no workstation measurement yet.
