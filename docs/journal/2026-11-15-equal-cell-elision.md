# 2026-11-15: Equal one-row cell elision: slower, rejected

**Status: built, measured on CI, closed unmerged.** PR #418 (head 93e6f056, against master
9146e287), decision #145 LOG 6895882037. With #410's line census, this closes equal one-row
cell elision as a performance route.

Labels: **measured** (CI's paired bench, run 43722454277); **counted** (temporary counters on
the bench corpus, not committed, no timing); **hypothesis** (what was expected before the data).

## 1. What we believed

MONODROMY's open list (`monodromy/2026-11-10-rauzy-veech.md` §6.4, §8 item 8) ranked "the equal
cells of one row, linked in the sweep and walked at the end" with a modelled ceiling of about
15% of the kernel. An equal one-row cell (ra = rb = 1; **z**) is one point. A one-point letter
ends in its first cell, so the cell's word is a factor of exactly one later word. Copying it
into the arena, only for that one reader to copy it again, looked like a copy that could go.

#410 (MONODROMY's elision bounds) had already narrowed this. Every equal one-row cell's slot
stores are still needed: which side is read first is not known at the store, and the partner
field carries the pairing. Only the cell's **word** could go. #418 built that, as one design,
for CI's paired bench to judge.

**Hypothesis** (#418's body): about 5 M fewer 16-byte copies in the induction, and 0.08–0.16 B
less arena write per output byte, so a small fall in Cells' CPU and wall, mostly at 4 threads.
The named risks were a dependent load at each reader, an extra 8-byte store into a top-slot
line, and an extra compare on every cell. If the bench showed no gain or a loss, it would close.

## 2. The census (counted)

On the Wikidata corpus, all 12 windows, master 9146e287:

| | count | share of cells |
|---|--:|--:|
| cells | 20,744,295 | 1 |
| **z: equal one-row cells** | **9,345,169** | **0.450** |
| z with a small word (≤ 16 bytes) | 7,292,450 | 0.352 (0.780 of z) |
| z words read by a later z cell (chains) | 5,716,243 | 0.612 of z |
| bytes the small z cells copy into the arena | 33,995,543 | 0.162 per output byte |
| **linked by #418** (small z, neither factor linked) | **4,293,912** | **0.207** |
| small cells that read a linked factor (one extra `copy16`) | 3,495,026 | 0.168 |

- z is large: 0.450 of cells. #410's tracer, on its own cohort, counted 9,218,588, 0.451 of
  cells.
- But z cells chain: 61% of z words are read by another z cell. A z cell whose factor is
  linked must materialise as before, so in a chain only every other cell can be linked.
  One-level linking reaches 0.207 of cells, under half of z.

## 3. The design

- **When:** at the cell step, if ra = rb = 1, the word is small, and neither factor is itself
  linked.
- **The node:** the top slot the cell let go. The cell ends letter A, and an equal cell
  promises no slot, so nothing names that slot afterwards or writes it again. It already holds
  A's (length, word) at bytes 8..16; one 8-byte store puts B's at bytes 0..8, which are dead.
  The cell's word becomes (|A| + |B|, the node). A linked word's offset is below the bottom
  region, so no flag bit is needed.
- **What the cell no longer does:** two 16-byte copies out of the arena, and `len` bytes of
  arena growth.
- **Who pays instead:** the word's one reader copies the two factors' bytes directly. That is
  a small cell in the induction's sweep (`copy16_word`), or a rule's small factor in the
  writer's forward pass or marked spine (`copy_small_word`). A read follows at most one node,
  never a chain.
- **The count:** about 2 × 4.29 M = 8.6 M copies saved, against 3.50 M extra in the
  induction's readers, plus the writer's.
- Long z words (22% of z) kept their rule. Handling them needed a position entry outside `pos`.

Correctness held: the exhaustive comparison against the original induction, a new Miri-listed
test reaching all three readers of a linked word, and the full Wikidata corpus with its CRC.

## 4. The result (measured)

CI's paired bench, run 43722454277, head 93e6f05 against base 9146e28, 10 paired rounds, AMD
EPYC 7763, 4 vCPU on 2 cores. `wikidata`, head over base:

| threads | wall | CPU | peak RSS |
|--:|---|---|---|
| 1 | **+11.9% (+9.3, +14.6)** | **+11.9% (+9.2, +14.5)** | 28 → 28 MB, −0.4% |
| 2 | **+8.7% (+6.7, +10.8)** | **+9.4% (+7.7, +11.1)** | 58 → 60 MB, +0.8% |
| 4 | **+8.2% (+7.0, +9.5)** | **+9.5% (+8.7, +10.2)** | 98 → 94 MB, −5.4% |

- All six wall and CPU rows are slower, beyond noise. The assessment: worse (speed, CPU cost),
  and no measure better.
- On the other corpora, `random` and `records` were 1.4–3.5% faster in wall or CPU on some
  rows, and the rest showed no change beyond noise.
- Every corpus gave the same bytes in every run.

## 5. Why it lost

- **What it saved was cheap.** The word copy it removed was a 16-byte copy whose source
  `ahead!` already prefetched, and a sequential arena write.
- **What it added sits on the hot loop:**
  - a compare and a branch on every cell, linked or not;
  - a branch on every factor that small cells and the writer read;
  - at each link's reader, two dependent loads: the node, then the factors' bytes, which are
    not prefetched.
- **Where it hurts most:** the loss is largest at 1 thread (+11.9%), where nothing else
  competes for memory. So the cost is compute and latency on the critical path, not
  bandwidth. The arena traffic it saved could only have paid at 4 threads, and there it
  bought −5.4% peak RSS, not time.

This is the cost #410 predicted: an equal one-row cell's slot stores remain, so only the word
copy could go, and that copy was cheap.

## 6. Conclusion

- **Closed:** equal one-row cell elision as a performance route. #410's line census showed that
  eliding stores within the slot layout removes stores, not lines (24 of 12.57 M induction
  lines hold only elidable bytes). #418 shows that eliding the one thing left, the word copy,
  costs more than it saves.
- **Not worth trying next:** deeper links, to reach the chained z cells. They would turn the
  deferred work into a pointer chase, which is the failure that sank #407.
- **Reopen only on** a design whose readers add no dependent load and whose cells add no
  branch. None is in hand.
- The route that did pay was a layout change: 8-byte bottom slots (#414, released in 0.3.3).
  The Cells induction campaign stops there; the next built candidate goes to the walk and the
  scanner (#145 LOG 6895882037).
