# Ordering complete complementary components

Fix the original physical atoms and ensure every component endpoint is a cut on both LF axes. Keep V in physical order. Within every complementary component keep its row orientation fixed. Let C be its set of components and W be the union of V with a subset M⊆C. For U∈M, let g(M,U) be the cost in ordinary endpoint steps of moving U to the tail in the current induced exchange on W and eliminating all of U.

**Claim.** The value g(M,U) depends only on M, U and the fixed orientation, independently of the order in which prior components were discarded and the relative placement of all the other components still in M.

First returns compose: the first positive return to W after eliminating an intermediate set is the direct first return from the original permutation to W. The initial-atom itinerary theorem applies equally when W is retained. It identifies the current source bands and their mapping to whole contiguous target bands. Each current band stays inside one of the original component chunks on each axis, since its first source atom and last target atom had those cuts. Reordering the surviving chunks preserves inside each chunk the widths, labels and correspondence of incident atoms. On both axes the start of U remains an explicit cut.

When U is last, its last source and target atoms and their widths determine the first step. The step only splits a surviving incident atom and updates the top/bottom positions of the affected pieces. Moving an external whole chunk cannot change any of those relative positions inside U. Equivalently, after removing a suffix of U, the current permutation is first return to W minus that suffix. The ordinary removal cut is a boundary of at least one current atom. Every pair inside U following the same current source/target path remains paired in either external placement. This gives the same next tip identities, widths and events inside U by induction over steps. The surviving cut at U's start prevents crossing it on the last subtraction. The resulting state on W minus U is again the canonical initial-atom itinerary partition with those complete components kept. [Itinerary and boundary invariants](THEOREMS.md)

This yields the exact recurrence D(∅)=0 and

```text
D(M) = min over U in M of [ g(M,U) + D(M minus U) ].
```

Each optimal choice is a suffix removal, so reverse its sequence to recover the coordinate order. There are |C|·2^(|C|−1) possible subset edges. Computing a single edge still requires obtaining the relevant current first-return atoms and their widths. The finite reference deliberately recomputes them by direct original physical LF. It separately runs every complete order from its initial atoms, checks each individual component cost against its subset edge, and compares exhaustive best cost with D(C). Across 3,671 selected small sections it matched all 246,562 physical component removals in 49,220 complete orders and all DP minima. A 12-step stable witness has an 11-step optimum. The recorded samples include many orders with identical cost. These results establish the scope of the optimization variable rather than an assumption that ordering must matter. [Independent DP and exhaustive replay](order_dp.py) · [Evidence and witness](order-dp.json)

The exact recurrence can serve as an oracle on a small number of components. It does not supply an efficient optimizer for the thousands of real event-section components. Deliberately splitting a component to create more ordering freedom also changes the initial label inventory, the final cylinder partition and the cost of subsequent publication. Its arithmetic belongs in a new comparison.
