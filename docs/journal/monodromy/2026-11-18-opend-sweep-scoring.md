# 2026-11-18 · MONODROMY 76 · Record 71's mechanism refuted by the opend sweep. The gain is fewer instructions (−24% at every W) at a lower IPC, not contention; DRAM traffic varies 0.72–1.05× while the gain stays at ~0.95. The VM is reconciled by its reference: eager Cells beats lazy by 17–21% at W = 3–4. Records 72, 74 and 75 lose their contention arguments; record 73 stands; one-thread work now decides

**The request.** The Coordinator's CHECKPOINT 6994386758.
- *The bundle:* `opend-sweep/run-20261118T225038Z`, evidence 824d0954, SHA-256 11560fda… (checked).
- *The run:* 432 records, W = 2–10, kernel-only arms.

**Labels:** **C** · **P** · **I** · **Conj**.

## 1. Record 71, scored (C)

| record 71 claim | outcome |
|---|---|
| §1 adjudication (ADOPTS) | stands (not retested) |
| §2: the reference's constant ~1–2 ms construction offset; kernel effect ≈ 0.946 (7/28) and ≈ 0.95 (8/32 at 8) | **Consistent.** Kernel-only, the sweep gives 0.934–0.953 at W 7–8 |
| §2: 8/32 mixes the worker effect | **Consistent.** opend over LazyCells at W 8 is 0.952, the kernel alone |
| **§3 M1: the ratio falls with W** | **FAILS.** W 2 over W 10 is 1.018 [0.989, 1.047]; flat |
| **§3 M2: it crosses 1 between W 2 and 6** | **NOT CONFIRMED.** It is already 0.950 at W 2 |
| **§3 M3: fewer L3-miss stalls per byte at W 8** | **FAILS:** 1.004. Elsewhere 0.73–0.98 |
| §3: "more base work, smaller contention coefficient" (a ≈ 1.3a, b ≈ 0.45b) | **REFUTED.** opend does *less* work: 24% fewer instructions |
| §3: lower power from "fewer stalled cycles" | **Wrong reason.** opend has *more* stall cycles. The lower power comes from fewer instructions retired |
| §3: bandwidth not binding at 7–8 | **Stands:** 20–25 GB/s |
| §4: B5 checklist | stands. The kernel-only construction is now used |

**The registered mechanism is REFUTED.**

## 2. What explains a flat 5–7% gain? (C, then I)

**Counted:** opend over LazyCells, per output byte, the median of 5 repetitions at every W.

| W | wall | worker CPU | cycles | **instructions** | IPC (LazyCells → opend) | branch misses | stores forwarded | total stall cycles | memory stalls | L3-miss stalls | DRAM B/B |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2 | 0.950 | 0.948 | 0.978 | **0.760** | 2.27 → 1.78 | 1.20 | 2.94 | 1.43 | 1.55 | 0.89 | **0.72** |
| 4 | 0.953 | 0.956 | 0.957 | **0.759** | | 1.20 | 2.82 | 1.20 | 1.33 | 0.73 | 0.97 |
| 6 | 0.936 | 0.938 | 0.941 | **0.760** | | 1.21 | 2.73 | 1.21 | 1.31 | 0.92 | **1.05** |
| 7 | 0.934 | 0.937 | 0.940 | **0.759** | | 1.20 | 2.69 | 1.19 | 1.26 | 0.87 | 1.03 |
| 8 | 0.952 | 0.947 | 0.953 | **0.759** | | 1.20 | 2.63 | 1.13 | 1.19 | 1.00 | 1.02 |
| 10 | 0.933 | 0.935 | 0.968 | **0.757** | | 1.21 | 2.66 | 1.05 | 1.07 | 0.93 | 0.98 |

**What the counters show:**
- *Instructions are constant:* 27.4–27.8 → 20.8–21.1 per byte, a 24% cut at every W.
- *The scanner's CPU is unchanged* (0.99–1.02). The gain is entirely in the workers.
- *Extra stall cycles* run +1.29 cycles per byte at W 2, +0.88 at W 7 and +0.59 at W 10.
- *Cycles saved outside stalls:* about 1.3–1.7 per byte.
- *Energy per GB* is 0.90–0.93 at every W.

**The reading (I).** opend's gain is a **work reduction**, on the 912 blocks it takes from LazyCells.
- *Where it comes from:* opend retires 24% fewer instructions per byte. It gives back roughly two
  thirds of that through a lower IPC: about 1.6× the memory-stall cycles at low W (L1/L2-hit
  dependencies, not L3), 2.7× the forwarded stores and 1.2× the branch misses.
- *Why it's flat in W:* the work is fixed per byte, and contention adds stall cycles to **both**
  kernels roughly alike. L3-miss stalls per byte stay within 0.73–1.0.
- *DRAM traffic plays no part in it:* the DRAM ratio moves from 0.72 to 1.05 while the wall ratio
  stays at about 0.95.

**The measurement that would separate the candidates (instructions, branches, front end).** This
is one machine run, a single worker, Wikidata ×4, both arms. Flatness in W makes W = 1 sufficient.
1. **Top-down slots.** `perf stat --topdown` (or `uops_retired.retire_slots`,
   `idq_uops_not_delivered.core`, `int_misc.recovery_cycles`, `uops_issued.any`) splits each arm's
   cycles into retiring, front-end bound, bad speculation and back-end bound. That says whether
   opend's IPC loss is back-end (stall dependencies), bad speculation (the extra branch misses) or
   front end (code layout, DSB).
2. **Instructions by symbol.** `perf record -e instructions:u`, or `callgrind` on the frozen block,
   locates the 24% reduction. My prediction (Conj) is that it falls in induction rather than the
   writer or the front end.

## 3. Reconciling the VM (C)

**The VM compared different things.** It ran `open` (every block), not `opend`, against **eager**
Cells, on 4 vCPUs.

**On the i9, eager beats lazy at low W.** From the recalibration bundle's Wikidata medians:

| cell | lazy over eager |
|---|---|
| pinned 3/12 (spinner, tight) | **1.205** |
| unpinned 4/16 | **1.166** |
| pinned 5/20 | 1.03 |
| 7 workers (open-return, Wikidata) | the reverse: eager is 1.27–1.30× *slower* |

**So:** opend at 0.95 of lazy is about **1.11–1.15× eager at W 3–4** on the i9. That matches the
VM's "open 9–21% slower than eager".
- *That is most of the story.* The reference flips between W ≈ 5 and 7, so opend beats lazy
  everywhere but beats eager only at higher W.
- *Something else may add to it:* the all-blocks `open` arm loses 1.73× on dense, text-like blocks,
  which opend's gate avoids. Microarchitecture is possible but not needed: opend's lower IPC (more
  memory stalls and forwarded stores) could cost more on another core.
- *For B5:* opend should replace LazyCells where production uses lazy (6–7+ workers, the trims).
  At W ≤ 4 it must be compared with **eager**, not lazy.

## 4. Records 72–75: which calls change (I)

**The lesson from the sweep.** Kernel ratios were **flat in W**, and DRAM traffic changes did not
move wall time. So contention-based extrapolations ("the ratio rises or falls with W") and
traffic-based credits are both unsupported. What decides is **per-byte work** (instructions ÷ IPC),
which a single-thread run measures.

| record | old call | new call |
|---|---|---|
| **72 (Astra C)** | "unlikely to cut time against lazy at 7–10, contention-sensitive dependent chain" | **CHANGES.** The traffic argument stands, and is *strengthened*: traffic savings bought nothing here. The contention argument falls. Undetermined, decided by the one-thread prototype (ns/row against lazy), whose result should now carry to 7–10 |
| **73 (Astra B)** | staged arrays cost 2.5–2.9× the credit | **Unchanged.** It was a work argument |
| **74 (Astra D): tiles** | "no" | **Unchanged:** 82 against 22 events per byte, a work argument |
| **74 (Astra D): final slot** | "the likelier of the two, ≤ 2–3%, via less fill-buffer occupancy" | **CHANGES** to *likely neutral (about −1% to +3% wall)*. It adds 2.6% instructions and saves traffic, and the sweep shows traffic doesn't buy time. The paired run still decides; a one-thread run on instructions and wall would show the direction more cheaply |
| **75 (Astra A)** | "roughly neutral at 7–10, the serial L3-hit chain offsets the traffic saving" | **CHANGES** to *undetermined*. Its ~1 B/B traffic saving buys nothing by itself; its −7.8% resident-work model is the relevant quantity, against its IPC (4.21× spans, short copies, 2–3-deep chains). The one-thread prototype decides, and should carry across W |

**A caveat on generalising (I).** Flatness held for one pair of kernels whose L3-miss stall shares
were similar. A kernel that cut L3 misses sharply might still scale differently. None of the Astra
candidates does that: their savings are mostly writes.

**What this decides:** nothing in production. It simplifies B5: opend's case is a uniform 5–7% work
gain over lazy. It is not yet established against eager at low W.
