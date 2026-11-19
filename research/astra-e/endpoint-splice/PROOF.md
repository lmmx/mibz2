# Exact substitution of a strict losing suffix

Consider side 0 as the losing order, with winner y on side 1 and tail A. In the losing order write

    prefix, y, D, C

where C is a nonempty suffix of distinct one-turn losers listed in their original order. The elementary steps take C in reverse order and insert each letter directly after y. Thus when the steps finish the order is prefix, y, C, D. The other order stays fixed. For side 1 the same order proof applies; each join places the winner's word on its original side of the loser's word. The winner is stable throughout the group and each distinct loser has its original width and word until visited. Calling each original join in precisely the original order preserves fresh tagged handles, memoized expansion, rule ordinals and surviving words.

Let W be the winner's current width and R the block's remaining induction budget. Starting from the strict loser, consume each next suffix width only if its sum with already consumed widths is at most min(R,W−1). This keeps each elementary loss strict and never crosses the operation's stop horizon. Stop at y's old successor on the losing side even if more width remains. In this case D is empty, the net order is the identity, and the next elementary step might start a second sweep over words already changed by the first. Its original loser and updated word are examined afresh on the next loop. Otherwise six assignments reconnect the doubly linked suffix after y. Subtract the same consumed width from both the winner and the budget; preserve the original number of elementary steps. A coincident endpoint or equal-width pair runs the original operations. By induction this generates the identical labelled two-order exchange at every batch endpoint, including the exact stop. No additional node or grammar storage is required. The separately generated census is excluded from the timed source.

The frozen baseline is the scalar-cap depth-16 Open-Return constructor at d0a42980; its gap/strip exchange differs from the full Cells permutation examined by MONODROMY 20. The earlier record explicitly anticipated strict-winner suffix splicing and measured a different exchange. The new empirical question is whether the measured scalar worker's actual gap ordering admits enough such batches after charging recognizer, all joins and the full finite inverse. [Prior one-turn proof and full Cells counts](../../../docs/journal/monodromy/2026-11-16-loops.md) · [Registered threshold](PLAN.md)

```sh
python3 -B research/astra-e/endpoint-splice/prepare.py /tmp/astra-suffix-source
```
