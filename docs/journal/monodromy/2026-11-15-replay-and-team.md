# 2026-11-15 · MONODROMY 15 · Path replay and a team sharing one block's Cells induction: both closed as priced

Assignment: QUEUED 6902723777. Louis's aim is Cells ahead of teams at high thread counts, with
memory bandwidth as the lever. Two builds were proposed:
1. inverting a block from the Rauzy path τ without the rule store;
2. splitting one block's induction across a team, through the free closure.

Both are closed on counts. Neither needs a timing prototype to decide it, because what decides
each is a count, not a clock:
1. the state a replay must hold;
2. Amdahl's law on the induction's share of Cells.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** measured (this container,
indicative) · **Conj** conjecture.

## 1. Path replay holds no less than Cells with the equal-word ring

**What a replay must hold (P).**
- *Three passes, not two.* τ with the run bytes determines the block (#440). But a block's
  bytes must be written bottom-up, because a copy's source must be complete first, and the
  positions are known only top-down, from the root, which is reverse creation order. So the
  replay is: forward (τ), backward (positions), forward (bytes).
- *Between passes 2 and 3:* one position per step, at least log₂ n bits each.
- *To undo an equal step:* the identity of B and the length of A's word. The step deleted A and
  moved B into A's bottom place. Neither is recoverable from τ and the state after, short of
  replaying forward again.
- *During the passes:* a letter must keep both orders' links in both directions. Undoing a top
  win finds B as the next letter after A in the bottom order; undoing a bottom win finds A as
  the next after B on top. Pass 3 writes the output while it still needs those links, so the
  bottom links cannot live in the dead output buffer the way #433 put Cells' there.

**Priced per letter (C).** The 196 admitted Wikidata blocks, per-block letters L, long rules R,
arena bytes A and cells S (`blocks_letters_rules_arena.txt`, here). Bytes a letter,
min / median / max:

| | words, or replay's state | orders, in the kernel's buffer | together |
|---|--:|--:|--:|
| Cells today: 16-byte rules, a position a rule, the arena | 15.94 / 17.93 / 20.08 | 20 (+8 in the output buffer) | about 37.9 |
| Cells with the equal-word ring of 8 (#446; 32% of the words) | about 12.2 | 20 | **about 32** |
| replay, 4-byte fields: a position a step, B's identity and A's word length an equal step, τ at 2 bits | 15.22 / 15.85 / 16.37 | 28 | about 43.9 |
| replay, packed to its information: 20-bit positions, 16 + 20 bits an equal step, τ at 1.2 bits a run | 8.90 / 9.27 / 9.57 | 28 | about 37.3 |

**Verdict.**
- *Memory:* even packed to its information content, replay holds more than Cells with the ring.
- *Time:* it costs about three inductions where Cells makes one.
- *A prototype:* it could only measure the time, which is already against it. **Closed.**
- *What stands is the bijection:* a decoder can be driven by τ (#440), and a future format could
  store τ. A rule store cannot be traded for replay cheaply.

## 2. A team on one block's induction: Amdahl closes it

**The simulation (C, every 8th Wikidata block, 30 blocks).** `team.rs.txt`, here, runs the free
closure in BSP rounds of disjoint free steps (`2026-11-15-closure-structure.md` §2) and splits
each round among T threads. The remainder, 0.54 r steps, stays on one thread: in BSP form it
would need about 0.36 r barriers.

| split of a round's steps | T | the closure's critical path ÷ r | plus the remainder ÷ r | ÷ the right end's 1.837 r steps | cross-thread letter touches a closure step |
|---|--:|--:|--:|--:|--:|
| fixed row ranges | 2 | 0.791 | 1.328 | 0.723 | 0.069 |
| fixed row ranges | 3 | 0.560 | 1.097 | 0.597 | 0.351 |
| fixed row ranges | 4 | 0.621 | 1.163 | 0.633 | 0.277 |
| each round cut into T equal chunks | 2 | 0.448 | 0.990 | 0.539 | 0.329 |
| each round cut into T equal chunks | 3 | 0.299 | 0.838 | 0.456 | 0.462 |
| each round cut into T equal chunks | 4 | 0.224 | 0.763 | 0.415 | 0.540 |

All medians.

**What the table says.**
- *Partition:* fixed row ranges balance badly, because free steps cluster by byte. Each round
  must be cut into equal chunks.
- *Rounds:* 20 / 42.5 / 168 a block.
- *The bound:* the steps' bound is about 0.46 of the right end's at 3 threads, or 0.29 with
  unlimited threads (the remainder alone). That is before cross-thread touches (0.46 a step at
  3 threads), the barriers, and the cost of finding free steps.

**Amdahl (M, this container, one thread; the profile of this morning's #433 work, Wikidata, 196
blocks).** A Cells block divides as: slots 21%, induction 50%, writer 29%. Splitting only the
induction at 3 threads, with free synchronisation and no worklist cost:

  0.21 + 0.50 × 0.456 + 0.29 ≈ 0.73 of one thread's Cells time.

With the free-step worklist's price, about 1.3× a step (§8.2 of the Rauzy–Veech entry), it is
about 0.80. That is a speedup of about 1.25 on 3 cores. A team of 3 on the T-vector path splits
both its build and its walk. On these numbers a team sharing one block's Cells induction cannot
beat it.

**Verdict.** Closed as priced, for the induction alone.

**The condition for reopening (Conj).** The slots stage and the writer would have to split as
well. The slots stage is a counting sort, and the writer's copy pass is independent rule by
rule once positions are known. That is a different construction, the Pathfinder's to judge: its
parallel parts are standard, and none of them is the induction's mathematics.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-replay-and-team/team.rs.txt examples/team.rs
cargo run --release --example team -- corpora/wikidata.bz2 8   # about 10 s
```

**The output.** `team_wikidata_every8.jsonl`, with, per T: `t{T}` (fixed row ranges) and `b{T}`
(balanced chunks) = [rounds, the summed busiest thread's steps, closure steps, cross-thread
touches, 0].
- The remainder's steps come from
  `2026-11-15-closure-structure/ctree_wikidata_every8.jsonl` (`remainder_paid_free_depth`).
- The replay pricing reads `blocks_letters_rules_arena.txt`: lines `BLK L R A S`, from an
  instrumented induction at master `df93e35`.
