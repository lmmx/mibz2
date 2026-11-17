# Worker-count calibration: the sweep and the model

*Opened 17 November 2026 at Louis's request. A standing record: how the decoder's worker count for a given thread count is found, what has been measured, and how to redo it when the algorithm or the machine changes.*

## Why this needs a standing method

The best number of decoding workers is not the number of threads. On the i9-10900K (10 cores, 20 threads, a 20 MiB L3; each core's L2 is 256 KiB), fewer workers than threads win at every count measured from 8 threads up. The reason is contention, not idle cores:
- Each worker's cost per byte falls as fewer workers share the L3.
- In #515, at 10 threads, going from 10 workers to 7 cut cycles per byte by 25% and LLC misses per byte by 48%.
- The clock (cycles over ref-cycles) stayed at 1.326 at every count.

Where the optimum lies depends on two things that change:
- **The algorithm:** each worker's working set (Cells' tables, arena, rules and slots), and its instructions per byte.
- **The machine:** cores, L3 size and sharing, memory bandwidth, and SMT.

So the optimum must be re-measured whenever either changes. The way to do that is below.

## What has been measured (i9-10900K, Wikidata ×4, rustc 1.99.1, lazy Cells)

Each value is the wall time against production's plan at that count, as the paired geometric mean of 10 rounds. Text was flat in every run, and all outputs were byte-identical.

| threads | production | arms tried (workers / blocks in flight) | best | wall vs production | peak RSS | source |
|---|---|---|---|---|---|---|
| 12, 16, 20 | 9 / 36 | 8/32 | 8 | 0.951–0.955 | 0.86–0.91 | #510, LOG 6931719814 |
| 12, 16, 20 | (against 8/32) | 7/28, 6/24 | 7 | 0.971–0.973 against 8 (≈ 0.92 against 9, chained) | 0.87–0.90 | #514, LOG 6942020517 |
| 10 | 10 / 40 | 9, 8, 7 (6 as context) | 7 | 0.885 | 0.74 | #515, LOG 6943194948 |
| 8 | 8 / 32 | 7, 6 | 6 (5 untested) | 0.929 | 0.80 | #515 |
| 4, 6, 8 | own plans | 2–6, with lazy controls | pending | | | `bench-workers-sweep-low` |

- **The window:** 4 blocks a worker held up. Doubling it changed nothing, and halving it cost 8% (#510).
- **Not covered:** the confirmation of 7 against 9, direct and on held-out corpora, and W12 at 7 workers. Both are being built.

### The first model: MONODROMY's records 41–44 on #497

The fit for cycles per byte is c(W) = a + b·W², with W the number of workers.
- From the 10-thread points: a ≈ 0.42, b ≈ 0.0068.
- The b·W² term is the shared-cache contention: each worker's misses grow with how many others share the L3.

Wall time ∝ c(W) / (W · busy(W)), where busy(W) is the share of each worker's time spent busy. That share is what made the residual: the busy share implied by walls and cycles matched the counted one within 0.01 at all five 10-thread points.

How the model's predictions have scored:
- **Record 41** (a capacity cliff): refuted. LLC misses fall smoothly with W, with no cliff.
- **Record 42** (ring or bandwidth contention): refuted on its counter shape.
- **Record 43:** W* = 7 at 10 threads, which was right. Its predicted 0.93 gain was too small; the measured gain was 0.885.
- **Record 44:** predictions for 4, 6 and 8 threads, frozen before the sweep.

## The procedure: how to redo the sweep

1. **Arms:** for each thread count T to set, run lazy workers W at 4 blocks a worker (`ab`'s `mibz2-occ-W-4W`), with W running from T (or production's count) down to about 0.5·T.
   - Where production's plan is not lazy at T, add a lazy arm at T workers as a control, so the kernel's effect is separated from the count's.
2. **Reference:** production's own plan at each T, paired by round.
   - Use #515's two_arm.py `--judge-at` to judge each candidate only at its counts.
   - Use one Holm family over all the judged main-corpus cells, a SLOWER screen on every corpus, memory ≤ 1.10, and a choice per count.
3. **Corpora:** Wikidata ×4 is the gain family. Text, and at least one held-out corpus, are the loss and memory guard.
4. **Counters,** reported and not judged:
   - cycles, ref-cycles, instructions, LLC-load-misses and l2_rqsts.miss per byte;
   - the busy share per worker, task-clock / (wall × W), an upper bound.

   These feed the model.
5. **Before the data:**
   - post the rule as a LOG;
   - have MONODROMY freeze its predictions as a record, so the model is calibrated, not only the plan.
6. **Run it** with `just submit <recipe>` on the workstation (RULES C5), queued from master's `bench/workstation-queue.txt`.
7. **Admit:** a production change follows only after the Researcher's C6 review.

Today's recipes are `bench-workers-window` (#510), `bench-workers-low` (#514), `bench-workers-at-cores` (#515) and `bench-workers-sweep-low`. The plan is to merge them into one recipe, `bench-workers-sweep`, that takes the thread counts and the W range as parameters and covers steps 1–4 on any Linux machine.

## Towards a plan the machine parameterises

Today's plan hard-codes the measured i9: a profile check (10 physical cores, a 20 MiB L3) gates the trim past the cores. The aim, following Louis on 17 November, is a plan whose worker count comes from a model, with the model's inputs read from the machine at runtime. That keeps today's gains, and makes it likely they carry to other hardware.

**The inputs, already read at runtime** (`physical_cores()` and `l3_bytes()` in `src/parallel.rs`, used by the teams policy's "live tables near 0.7 of the L3" rule):
- physical cores;
- L3 bytes;
- SMT;
- the block size from the stream header (`Plan::for_stream`).

**The model to fit:**
- the best count is W*(T) = min(T, cores, the W that minimises wall(W));
- wall(W) ∝ c(W) / (W · busy(W)), with c(W) = a + b·W².
- **a:** the per-byte cost of a worker alone. It depends on the kernel and roughly on the clock.
- **b:** the contention cost. The hypothesis to test is that b scales with the working set per worker over the L3 size, (footprint / L3)^k. The footprint is set by the block size and the kernel, and is about 2.1 MB for a Cells-lazy worker on a 900 kB block.
- **busy(W):** the share of time left after the reader and scanner threads. It depends on T against the cores.

**Reading the cache correctly.** Reading the cache is cheap: one sysfs read at startup, cached. Today's `l3_bytes()` reads `/sys/devices/system/cpu/cpu0/cache/index3/size`. That has two faults to fix before any model relies on it:
- It assumes `index3` is the L3. It should find the index whose `level` is 3.
- It reads one instance and ignores `shared_cpu_list`. On CPUs with several L3 domains (AMD's CCXs, or two sockets), workers compete within a domain, not across the total. The model needs the per-domain size and the number of domains the threads span.

**A first candidate rule,** proposed on 17 November and checked against the data already in:
- **The rule:** W_cache = min(cores, ⌊α·L3 / M_worker⌋), with α = 0.7 (the teams policy's budget) and M_worker ≈ 2 MB. On the i9 that gives 7.
- **Where it fits:** past the cores (7, as measured) and at 10 threads (7, as measured).
- **Where it misses:** at 8 threads it says 7, but 6 measured better (0.929 against 7's 0.975).
- **What that means:** the cache budget alone is not enough, and the thread count matters too. The busy-share term, the reader and scanner competing for the T threads, is the candidate.
- **What would fit all three:** W = min(W_cache, ≈ 0.7·T).
- **What decides it:** the low-thread sweep (4, 6 and 8 threads, with 5 workers at 8) tests it, against MONODROMY's frozen record 44.

**How the model is fitted and tested:**
1. Fit a, b and k on the i9 from the sweep above. That is one machine at two block sizes: the -9 Wikidata and a -1 or -3 file, which varies the footprint.
2. Predict W* on a second machine before measuring it. CI's GitHub runners (4 vCPUs on AMD EPYC, and the macOS runners) are free and differ in L3 size. Then measure with `bench-workers-sweep`. This is the transfer test, and its rule is posted before the data.
3. Ship the model only if it predicts the second machine's W* to within one worker, and its wall to within the sweep's noise. Otherwise keep the measured profile and widen the measurement.

**Options considered:**
- **Online self-tuning,** where the decoder varies W during a stream and keeps the fastest: deferred. Streams are often short, the noise is high, and output timing would stop being deterministic. A one-off calibration command, which runs the sweep and caches its parameters under the XDG cache directory, is the fallback if no static model transfers.
- **Hard-coding more profiles:** rejected as the long-term route. It doesn't scale, and it says nothing about unmeasured machines.

## Status

- **Owners:** the sweep recipes are with the Coordinator's subagents; the model is with MONODROMY (its lane: records on #497, scored against each run); admission is with the Researcher.
- **Open items:** the low-thread sweep; the direct confirmation; W12 at 7 workers; merging the recipes into `bench-workers-sweep`; fitting a, b and k at two block sizes; and the transfer test on CI runners.
