# Two forward first-return walks in one output buffer

Let `F` be the inverse LF permutation on the original `n` rows, with one forward output byte per edge. Let `S` be the retained rows, containing the primary `p`. For each retained row `r`, follow at least one edge and stop on the first subsequent visit to `S`. Write `P(r)` for that destination and `W(r)` for the bytes traversed, including the byte on the edge leaving `r`. The outgoing label is source-indexed. The primary is a retained coordinate at a word boundary in this constructor. The block can have several cycles, and a marked period need not divide the requested output length.

## The byte budget

Consider a cycle of `F` containing retained rows. In cyclic order, successive retained rows partition its outgoing edges into disjoint nonempty arcs. Each edge of that cycle appears in precisely one `W(r)`. A cycle without a retained row contributes zero. Thus

```text
sum over r in S of |W(r)| = number of original rows in cycles meeting S <= n.
```

The sum counts each **retained row's emission**. A dictionary word reused by a band of width `w` contributes its length `w` times. Different retained paths may spell identical bytes. Coverage and the byte bound depend only on their distinct original edges. The equality also shows that the induced map `P` is a permutation on `S`. The original endpoint proof provides the same arc decomposition in the decoder's backwards-LF notation. [Open-Return proof at 2b53453](https://github.com/lmmx/mibz2/blob/2b53453e124ffdc75cc4d2141b833a319e3cfd73/docs/journal/2026-11-18-open-return/proof.md)

## Two first hits determine order and membership

Take `s != p`. From each seed follow `P`, emitting the current word before advancing, and stop upon first *arrival* at either seed. If they lie on one cycle, the terminal pair is `(s,p)`. Call their emitted strings A and B. The marked period is A followed by B. If they lie on two cycles the terminals are `(p,s)`, and the marked period is A alone. These exhaust the possibilities because a permutation has unique successors and predecessors. In either case the two paths are edge-disjoint, before as well as after closure. A cursor's arrival at its own seed certifies separate cycles immediately. Speculative work can stop once A closes at p. Coincident seeds require only the original marked walk.

The period is nonempty. Repeat it on the output and stop at exactly `n` bytes. Treating the two words emitted immediately before the two hits as part of their respective paths handles one-edge cycles and both arc-length parities. A fictional mark in the middle of a word would require splitting or tracking a byte offset. The current width-16 construction explicitly retains the primary.

### Reverse suffix

Keep A's prefix at the left and the byte reversal of B's prefix at the right. To append a word to B, decrement the right cursor once per byte **in the word's ordinary forward order**. For example, adding `ab` and then `cd` gives suffix `dcba`. The storage ranges cannot overlap by the byte-budget theorem. On separate cycles discard B. On a shared cycle reverse the occupied suffix once, move it down into the gap if there is one, then repeat the marked period. The move must support overlap. This uses `n` output bytes and a constant number of scalars. Initializing an entire vector or reversing long populated suffixes has a real write cost.

For a generic labelled permutation whose total word lengths can exceed the supplied budget, the same closure rules apply, and an implementation can guard each speculative append. On an impending collision discard B, keep all A bytes already written and finish the ordinary marked walk from A's current row, including the edge that could not fit. A paused at `s` resumes from `s`. Truncate a final word when A reaches the requested length. When A closes earlier, repeat. This repair is exercised under arbitrary clipping in the checker. An over-budget state is unreachable for the decoder's first returns.

### Two forward reservations

Fix any byte split `k`. Begin A at offset 0 with capacity `k`, and B at offset `k` with capacity `n-k`. Each emits words in their ordinary forward order. Let `a,b` be their eventual first-hit lengths. In either terminal case `a+b <= n`. This gives an exact response to a reservation overflow before any part of the next word has been written.

* If A's next word would cross `k`, then `a>k` and `b<=n-k`. Pause A. B can reach a seed in its initial reservation. If B closes at s, discard its bytes and finish A. If B reaches p, its length is known. Move B to `n-b`, finish A, and close any remaining gap.
* If B would cross `n-k`, then `b>n-k` and `a<=k`. Pause B. A can reach a seed in its initial reservation. If A closes at p, discard B. If it reaches s, move B's current prefix down to the exact final start `a` and resume it there until p.
* Whenever a cursor reaches a seed before exhausting its capacity, its terminal and exact byte count are already known. If A reaches s first, its exact length suffices to place and finish B. If B reaches p first, leave its completed bytes at `k` while A fits, and right-align B only if A needs the space.

There is no second exhausted reservation before closure of at least one cursor, even with indivisible variable-length words and highly skewed schedules. The split can be 0 or `n`. In the shared-cycle, full-length case `a+b=n`, at most one contiguous range move of B is required. In a shorter marked cycle B may move twice. This proof specializes the broader plan's bounded prefix relocation. It permits a writer to drain the other cursor on first overflow and keeps all word copies forward. The choice between draining and relocating a live prefix is an engineering question. Both can reduce overlap when one side stalls.

## Available parallelism

Suppose the marked cycle has `L >= 2` retained rows in a graph of `m` retained rows. A uniformly chosen distinct seed lies on that cycle with probability `(L-1)/(m-1)`. Conditional on this event, its forward edge distance `j` is uniform in `1..L-1`. With unit-cost edges and unlimited placement, the best two-lane dependency span is `max(j,L-j)`, so the mean edge-span reduction is `floor(L²/4)/(L-1)`. A deterministic midpoint in *residual row rank* can still lie just one logical edge from p. Word lengths, speculative cycles, memory latency, array initialization, relocation and unbalanced execution all affect a real writer.

## Prior implementations

| Frozen code | What it already implements |
|---|---|
| [Ranked Open-Return at 2b53453](https://github.com/lmmx/mibz2/blob/2b53453e124ffdc75cc4d2141b833a319e3cfd73/docs/journal/2026-11-18-open-return/segments.rs) | Seeds forward walks, stages word codes, then resolves and emits segments in marked order. |
| [Four-tape writer at 994466b](https://github.com/lmmx/mibz2/blob/994466b9e00627caf25537432677e3fac622bc6e/research/astra-e/open/lanes/writer.rs) | The same general forward first-hit partition theorem and delayed fragment placement. It emits into four auxiliary byte tapes from 16–32 seeds, maintains fragment successors and copies marked fragments into output. This is the closest prior construction. |
| [Two-ended writer at 4653a4d](https://github.com/lmmx/mibz2/blob/4653a4d12cf019c6181325472c622ec97ba26442/research/astra-e/bidirectional/writer.rs) | Opposed walks both start at p. One uses a separately built inverse-edge table. It already joins a prefix and a suffix in output, compacts a periodic gap, and repeats. |
| [Anchor writer at 9a93d9d](https://github.com/lmmx/mibz2/blob/9a93d9d5e1daf8895423f4a9ebdc220215217ca7/research/astra-e/anchors/writer.rs) | Uses dictionary-word anchors, materializes literal tails and walks anchor successors to emit marked output. |
| [Sparse landmarks at 1877dbd](https://github.com/lmmx/mibz2/blob/1877dbd7082b9d41b73011da7746dce336765e47/research/astra-e/sparse-landmarks/sparse.rs) | Discovers weighted segment lengths and topology in a first pass, computes their final positions, then revisits the primary segments to write bytes directly. |

The two-seed emitter combines the four-tape writer's graph decomposition with bounded placement in the final output. In the inspected implementations, neither a reverse-oriented speculative suffix nor the two-forward provisional reservations implement the historical writer's arbitrary-seed decomposition in exactly this way. This is a limited source comparison.

## Exact checks

All permutations through six rows were crossed with every pair of seeds, every independent assignment of 1- or 3-byte words, an area-sized output budget and a deterministic clipped budget. Multi-byte labels have distinguishable order. Four schedules alternate, alternate B first, drain A first or drain B first. A second construction enumerated all permutations of original rows through six and every nonempty retained subset. It verified disjoint original-row coverage, induced the *actual* variable-length words, and compared all marked/seeded outputs to a raw uncontracted walk. For these induced graphs every possible byte split was tested.

The reverse-suffix model passed 16,150,788 schedule comparisons. It exercised 2,598,232 over-budget speculative discards among the clipped weighted fixtures and zero on genuine induced inputs. The two-forward-reservation model passed 28,371,496 schedule/split comparisons. Among distinct-seed cases A overflowed first in 3,331,810 and B in 3,006,666. There were 49,489 raw permutation/retention graphs, including 24,308 with omitted original cycles. These are model checks of the emitter and byte-area proof. They make no complete-decoder timing or peak-RSS claim. [Checker](check.cpp) · [Raw counts](results.txt)

```sh
d=research/astra-e/two-seed/proof
g++ -std=c++17 -O2 -Wall -Wextra -Werror "$d/check.cpp" -o /tmp/astra-two-forward
/tmp/astra-two-forward
```
