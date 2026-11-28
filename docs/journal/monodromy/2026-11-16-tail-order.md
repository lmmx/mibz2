# 2026-11-16 · MONODROMY 32 · Past the switch the streams no longer bind, so the tail can take a better order: free-first then the largest cell, from 1.2 r, makes 0.31 r tail cells instead of 0.59. The compact tail's joins halve (1.24 against 2.36 B a letter) and 0.28 r cells go

Follows records 27 (the compact tail), 29 (the order of contractions) and 31 (the cost lies in the
path).

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled.

## 0. Result

**Why the streams no longer bind (P on the record, C).**
- *Where the constraint lives:* the stream constraint (record 17) binds only on untouched original
  letters.
- *How many are left:* by the compact tail's switch nearly all are touched: 0.0001 r untouched
  at 1.5 r (record 23), and a small share at 1.2 r (record 19's curve: 0.048 r untouched at m = n/4,
  0.000 by n/8).
- *So:* the tail runs on pool letters, where any order is admissible. The few untouched ones cost
  one stream read each.

**The tail under other orders (C, every 24th Wikidata block, 10 blocks).** The right end runs until
the switch; then the order changes.

| switch | tail order | tail cells | all cells | compact tail, 4 B a join |
|---|---|--:|--:|--:|
| 1.2 r | right end (as now) | 0.590 r | 1.79 r | 2.36 B a letter |
| 1.2 r | ratio (points per word byte) | 0.381 r | 1.58 r | 1.53 |
| 1.2 r | **free-first, then the largest cell** | **0.311 r** | **1.51 r** | **1.24** |
| 1.5 r | right end | 0.290 r | 1.79 r | 1.16 |
| 1.5 r | free-first, then the largest cell | 0.192 r | 1.69 r | 0.77 |

**At the 1.2 r switch:**
- *Memory:* the joins fall by 1.1 B a letter, on top of §6's 4.9 B (record 27). Together that is
  about 6 B a letter, 39% of the word store.
- *Cells:* 0.28 r fewer.

## 1. What it costs (M; the Pathfinder's to count)

**The tail order needs a priority on the pool's coincidences.**
- *Its size:* about 0.18 r live letters at 1.2 r.
- *Free steps:* those whose loser lands on a boundary take a test at the step's ends, as in
  `schedules_geo`. Then come the largest cells.

**The trade.** The lazy right-end loop has no coincidence index, so the tail would be a second loop
over the pool. Against that, 0.28 r cells are not made, at about 38 instructions each (the
Pathfinder's figure for a tail cell). That is about 10.6 a letter to spend on the priority.

**A caveat on these counts.**
- *In the simulator:* the switch drops the free-step list that the right end did not use, so
  free-first starts from the coincidences the tail makes, plus the largest-cell heap.
- *In a kernel:* a kernel would rebuild that list, so the free-first rows here are if anything
  conservative.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-tail-order/geo.rs.txt examples/geo.rs
SWITCH=1.2 cargo run --release --example geo -- corpora/wikidata.bz2 24 65536 6,8,9   # 8: then ratio; 9: then free-first, largest cell
```

**Field:** `switch_{p}` = [steps at the switch, word bytes made by then]. `wfinal_{p}` and the
step counts are as in record 29.
