# Mark, selector and grammar reachability checks

The exact gate constructs the ordinary stable permutation by sorting the original physical row indices by their L bytes and recovers Ψ and F. For every tested mark it independently emits n forward bytes, measures the true marked lap, computes each of the three selection predicates on the original physical rows, and compares the complete decoded output, saved lap and every real or synthetic selection decision. The small gate enumerates all marked binary columns through length eight and ternary columns through six. It adds long capped runs, more than sixteen distinct F buckets with tied frequencies, intervals on the four-byte threshold, the floor(n/22) cutoff, near-primary cuts, repeated blocks and multiple-cycle cases.

All 30,603 arm/mark checks passed on 1,702 physical columns. Of the 10,201 tested marks, 8,640 had short laps and 3,873 had lap length not dividing n. All thirty real arm/block calls matched stable F/Ψ and the independently hashed original capped/marked outputs. All ten real physical marked laps traversed every row. [Exact finite receipts](small.txt) · [Every real geometry, mark and capacity](real.csv)

| Across 8,999,915 physical rows | Absolute prefix | L runs ≤4 | Least frequent 16 F bytes |
|---|---:|---:|---:|
| Selected rows including primary | 409,094 | 784,776 | 6,745 |
| Spans including complements | 38 | 97,682 | 170 |
| Weighted affine return records | 80,819 | 617,326 | 4,110 |
| Binary joins constructed | 1,029,865 | 445,729 | 1,227,593 |
| Binary joins reached by marked output | 1,029,865 | 445,729 | 1,227,593 |

**Reachability on primitive input.** At every surgery boundary, the current permutation is the original first return on the remaining positive-width physical region, after stable conjugation. A single original LF cycle induces a single cycle on every nonempty subset. While the remaining elimination budget is positive this permutation has at least two rows because the final section contains the mark. A coincident tail record would translate each rank in its positive-width common tail interval to itself, contradicting that single cycle. Thus no coincident self deletion occurs on primitive inputs. At a distinct equal or unequal reduction, any word taken off a disappearing edge is a child of the fresh concatenation on a positive-width edge. The final partial step preserves all prior positive words directly or as children. Inductively every created join remains present in at least one surviving edge's subgrammar. The marked induced orbit visits every final source row, so every such rule is reached. This explains the real equality across three very different retention densities. On multiple cycles an unmarked component can use grammar absent from the marked spelling.

If the selected marked orbit has e return edges and its grammar reaches V distinct joins, memoized emission processes exactly e+2V emitted handles plus V completion frames. A first visit schedules two children, while a previously completed join copies a contiguous output occurrence. Copies and literal stores in the first lap together write exactly its byte length, and repetition writes each remaining output byte once. The emitter uses O(e+V+n) work at fixed word size, in addition to initializing the memo for all G constructed joins and building the compact forward successor table. The corpus counts make clear why reducing e alone can increase the paid spelling and induction budget.

The per-block check reuses a Worker for the three selectors in order within a block. Capacities in the gate reflect that reuse. The timed donor uses private per-arm workers, and fresh peak memory is measured separately. [Complete worker](WORKER.md) · [Registered benchmark](bench/PLAN.md)

```sh
d=$(dirname "$(rg --files | rg '/return/collation/induce/native/worker.hpp$')")
g++ -std=c++20 -O3 -Wall -Wextra -Werror "$d/gate.cpp" -o /tmp/astra-induce-caps-gate
/tmp/astra-induce-caps-gate | diff - "$d/small.txt"
python3 -B "$d/gate.py" /tmp/astra-induce-caps-gate /tmp/astra-cut-caps | diff - "$d/real.csv"
```
