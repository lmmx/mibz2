# 0.5.7 six-thread trim adoption

**PROCEED at `05f4174c7fbdcc43f499bbc35174ba542dced05e`, subject to the exact-head CI and paired bench gate.** PR #642 implements both registered six-thread winners on the measured i9. The other default dispatch cases retain their prior semantics.

## Replay of the registered adoption

I fetched the workstation bundle at evidence commit `f6c32d79466b5660f45a9a8cc198a69a2c446cfd` and verified SHA-256 `98200b84513f4a4d1ace03af1f3874f1dedf37afc32f9836b462d9b584013805`. It records recipe head `9ba4416acc9911f2dbd668e6d9895be52c485a76` and decoder pin `1740582674ac8e6dcd90329a62a6c46a4b92fd29`. Between that decoder pin and #642's base, the only changed file is the workstation queue. I replayed both placements with the frozen `two_arm.py` and the recipe's arguments. Outputs match the archive byte for byte over 720 pinned and 756 unpinned records. There are exactly two adoptions over the 26 comparisons, one at each placement of six threads.

| Placement | Winner and shipped comparator | Wikidata ×4 wall ratio [95% interval] | Median peak RSS ratio |
| --- | --- | --- | --- |
| Pinned, one CPU per core | OpenCells 5/20 over Cells 5/20 | 0.9340 [0.9264, 0.9417] | 0.844 |
| Whole machine | OpenCells 6/24 over Cells 6/24 | 0.8531 [0.8024, 0.9071] | 0.869 |

The one-sided Wikidata gain tests reject in each placement's Holm family at 0.025. Every RSS ratio in both winners is at or below 1.10. The unpinned text guard is 1.0846 [0.8799, 1.3370]. It clears the registered SLOWER screen as INCONCLUSIVE, with a wide interval that leaves a text slowdown unresolved. The CHANGELOG states that interval and the scope of the adoption accurately. No new timings were collected.

## Dispatch proof and test scope

Let P be `pinned_one_a_core`, W be `whole_machine`, and U be the existing `threads >= 10` condition. On the accepted i9 profile W's target has 10 physical cores and 20 logical CPUs, so P and W are disjoint. The old choice applied `P || (W && U)` to the old `trim_cell` roster. The new choice selects the same roster for P except for the lazy flag at `6 / Workers(6)`. For W and U it calls that same roster. For W at 6 it adds `Some((6, true))` only with `Workers(6)`. Other whole-machine counts below 10 return None. All other masks and all rejected profiles return None under both predicates. The machine, kernels and unlimited-quota predicates have the same conditions. The new `trimmed_read` prefilter covers the same thread/plan pairs, since `unpinned_cell` is defined only where `trim_cell` already returns a cell. Forced plans and `trims_to_seven` have the same paths and `LAZY_FROM_THREADS` is still 8.

With the normal four slots per requested thread, `StreamDecoder::new` constructs `workers(5, 20, k, true)` or `workers(6, 24, k, true)`. Both use OpenCells. A small arithmetic edge lies outside the measured default: in the new unpinned cell, directly supplying more than `usize::MAX / 6` slots can now saturate in the generic scaling calculation, even though worker count equals thread count. Realistic caller-specified windows are preserved at that placement, including the previous worker-count minimum.

The new deterministic test checks the constructor's concrete OpenCells type and measured shape at both placements, byte-exact results and shape through `BzReader::parallel` and `decode_parallel`, and eager fallback for quota, unsupported affinity, scalar and another machine. The wrapper observations record count and window, and both wrappers delegate to the constructor whose concrete kernel is asserted. I inspected these assertions and the constructor paths. The full Rust suite is covered by the exact-head CI gate.

## Correct the model score beside the verdict

The Coordinator's LOG 7019274828 calls the pinned-eight prediction untested. The candidate was measured: `occ-7-28` over `occ-6-24` gave **1.0426 [1.0369, 1.0484]** on Wikidata ×4. This crosses record 83's explicit +1% refutation threshold for the recalibrated pinned-eight budget term. At unpinned 16, the 8/7 ratio was **1.0376 [0.9556, 1.1267]**. The point crosses the model's +1% criterion, with an inconclusive interval. At pinned 10 and unpinned 10, 12 and 20, the 8/7 point estimates were below one and failed adoption. The prediction's fit and the evidence-qualified adoption are separate outcomes. The only qualified move is the six-thread kernel, at the same worker counts.

[PR #642](https://github.com/lmmx/mibz2/pull/642) · [Workstation submission](https://github.com/lmmx/mibz2/pull/145#issuecomment-7019212049) · [Coordinator's outcome log](https://github.com/lmmx/mibz2/pull/145#issuecomment-7019274828) · [Frozen prediction](https://github.com/lmmx/mibz2/blob/4040b3a1483f226ba3bfabdb2f675f91abc96c0b/docs/journal/monodromy/2026-11-19-recalibration-057-predictions.md)
