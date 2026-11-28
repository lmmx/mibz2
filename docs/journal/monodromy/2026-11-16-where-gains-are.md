# 2026-11-16 · MONODROMY 38 · Where I expect results at 10–20 threads on the i9: in per-worker speed through the simple layout line, and probably small. Not in new mathematics, on the evidence counted so far

The Coordinator's question, at Louis's direction (19:01): given everything counted and measured,
where do I expect real results for decoding at 10–20 threads? A short ranked list, each with its
mechanism, its expected gain at the calibrated cost (record 36, §4), and what would show it wrong
cheaply.

**Labels:** **C** counted (a measured figure from the record) · **M** modelled.

## The evidence that ranks it

- **The workers bind past the cores** (LOG 6906620039). In the capped plan at 20 threads they are
  91–92% busy, and the scanner 61–69%. So the next gain is per-worker speed.
- **Per-worker speed at 10 threads is stall-bound** (LOG 6925144719). Lazy's IPC is 1.16 at ten
  threads against 2.19 at one. Its LLC misses barely move when 5 B a letter of words go (1.000).
  So the stalls are hits in L2 or L3, contention and latency, not DRAM.
- **The SMT route is far off** (LOG 6906620039). Uncapped lazy, two workers a core, loses 22% to the
  capped plan at 20 threads (0.776). stop15's 0.965 there was against uncapped lazy, so a footprint
  cut on the scale of W12 cannot close that gap.
- **Instruction-heavy reorganisations lose at one worker a core.** That is the ring, stop-and-walk
  and the compact tail (record 36).

## The ranked list

| rank | mechanism | expected gain at 10–20 threads (M, calibrated) | what shows it wrong cheaply |
|--:|---|---|---|
| 1 | **12-byte rules (W12)**: 2.29 B a letter fewer rule bytes, writer at parity (#503). If the stalls are partly L2 traffic from rules, fewer bytes a rule cut them. | **0 to 2%**, only if storing 12-byte rules at the cell costs ≤ 5 instructions a letter. Its induction side is a hot-loop change: a kernel count is needed. | Its Gate 1 count. Then **one counters run with L2 misses** (`l2_rqsts.miss`, which the last run lacked) on W12 against lazy at 10 threads: if L2 misses a byte don't fall, no layout cut will show at 10 threads. |
| 2 | **10-byte rules (W10)**, record 37: 3.43 B a letter, the floor without rebuilding lengths. The same mechanism as W12, a third larger. | **0 to 3%**, if the tag test per child costs the writer under W12's margin. | #503's replay of W10's writer against W12's (HANDOFF 6925942307). If it is slower than W12 by more than its extra bytes are worth, drop it. |
| 3 | **Common-child grouping**: 1.2 to 2.5 B a letter (#503). | **≤ 1%**, and only on top of 1 or 2. A group header and a seek index add a dependent load in the writer. | The replay harness, as for W10. |
| 4 | **8-byte rules (W8)**: 4.57 B a letter, at +17% writer time (about +2.5% of the kernel). | **Negative at 10 threads** unless L2 relief is real and large. Only worth testing if rank 1's L2 counters show relief scaling with bytes. | The same L2 counters. |
| 5 | **The serial scanner** (parallel scan; `examples/par_scan.rs`). | **0 now.** It is 61–69% busy; it binds only once the workers are about 30% faster. | Occupancy, re-read after any worker gain. |
| 6 | **New mathematics**: orders, stops, towers, the surface. | **Not expected near term.** Every route I found either needs random access the streams don't give (free-first at the fronts is the best stream order, −15% of words, but costs a second induction, the ring's class), or adds state to the hot loop (record 36). | — |

## The answer, plainly

**My honest answer is the simple layout line,** W12 first, and I expect its gain at 10–20 threads to
be small. The counted evidence says the stalls at ten threads are not LLC misses, and byte cuts have
not yet moved them.

**The one cheap measurement that would tell us** whether any byte cut can pay is L2 misses a byte at
ten threads: W12 against lazy, or even stop15's bundle against lazy if the recipe can add the event.
That measurement decides whether ranks 1 to 4 are worth their builds.

**New mathematics ranks last for now.** The decoder's remaining cost at 10–20 threads is stalls per
worker, and none of my results changes what the workers touch per byte at an instruction cost the
calibration allows. I'll branch out only after the layout line is read, and only with a LOG first.

**Added after #481 (the Coordinator, 20:48).** Nine lazy workers past the cores is adopted:
- *the gain:* 0.956 to 0.960 wall at 12 to 20 threads, CPU 0.85 to 0.89, peak memory 0.82 to 0.88;
- *the mechanism:* fewer workers in flight, and a core left for the scanner.

It relieved contention without cutting bytes a letter. The ranking is unchanged, but it sharpens the
falsifier: if W12's L2 diagnostic shows its stalls falling less than #481's did, contention, not
footprint a letter, is the lever, and ranks 1 to 4 drop below further occupancy changes.
