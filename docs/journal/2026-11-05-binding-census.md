# 2026-11-05: What binds the decoder today, on CI's runners

MODE (f6581f76) asks the Coordinator first what binds today's production decoder, on which
machine, with which workers and under which contention. This entry answers it for CI's runners.

**How it was answered.** Five rival routes were each cross-examined by a falsifier taking a
different route:
- whole-decoder time accounting;
- worker supply;
- the walk on two SMT cores;
- the serial stages;
- the no-go archaeology.

Every front, its falsifier's verdicts and the run ids they cite are in
[`2026-11-05-binding-census/`](2026-11-05-binding-census/): one file per front, and
`results.json` with the full structured results. Where a falsifier corrected a front, its
correction is what stands here.

**The decoder.** Master at d1fd4405 (#181). Every later merge changed docs or CI only.

**The machine.** GitHub-hosted runners: 4 vCPUs on 2 physical cores × 2 SMT siblings, with the
CPU model varying by run:
- AMD EPYC 7763 (Zen 3), 9V74 (Zen 4) and 9V45 (Zen 5);
- Intel Xeon 8370C and 8573C.

There is no PMU on the AMD runners.

**New CI measurements.** Three phases dispatches gave per-stage co-run factors, four pinned
copies against one alone:
- 42008210159 (8370C);
- 42011789918 (7763);
- 42020095581 (9V74).

## The frame

At 4 threads the decoder is **CPU-capacity-bound** (OBSERVED: utilisation 0.91–0.97 on every
runner, runs.jsonl medians). The 4-thread wall therefore tracks total co-run CPU divided by
4. What binds is the sum over stages of each stage's alone cost times its co-run inflation.

That is why levers that cut co-run cost have been the large ones. #171 cut 4-thread CPU
16–18% on the 9V45 with 1-thread time flat (`2026-11-05-walk-round.md`). A 1-thread gain
reaches 4-thread wall at a ratio that depends on the CPU model, anywhere from 0 to above 1. No
candidate's 4-thread figure should be derived from 1-thread or stage gains without a 4-thread
CI run.

**Co-run inflation**, four pinned copies over one alone (OBSERVED):

| stage | 7763 | 9V74 | 8370C |
|---|--:|--:|--:|
| walk | 2.12 | 2.22 | 1.46 |
| build | 1.43 | 1.76 | 2.25 |
| MTF | 1.33 | 1.23 | 1.54 |
| scanner | 1.43 | 1.32 | 1.56 |
| RLE1 | 1.53 | 1.86 | 2.08 |
| assembly | 2.97 | 3.53 | 1.77 |

At 2 threads, one worker per core, whole-decoder amplification is only 1.04–1.11, and SMT
buys only 1.13–1.24× from 2 to 4 threads (OBSERVED).

## What binds, ranked by recoverable 4-thread wall

1. **The walk's co-run excess on AMD.** On AMD the walk inflates ×2.1–2.2 against ×1.2–1.4 for
   the compute passes.
   - Lo 0; hi about 16–18% of 4-thread wall on the 7763 and 9V74.
   - No lever has been measured. Lanes, prefetch hints, tile and segment counts are all flat
     at four copies (`2026-11-05-coresident-walk.md`).
   - The mechanism is **contested**:
     - The walk front's falsifier finds it core-side. Kernels 3.3× apart in line rate all
       lose ×2.12–2.16 at four copies on the 7763, and the fit's lane-independent term
       rises ×2.7–3.1.
     - The accounting front's falsifier finds it lines in flight. Per-thread walk MLP falls
       from 9.3–9.8 alone to about 5.5 at four copies, while a 1-line probe rises only
       18–28%.
2. **Co-run-aware placement.** Which stage runs beside which on a sibling has never been
   examined. The census bounds idle-only ideas at about 4%; placement is not bounded by that.
   #180 moved the co-run CPU term (about 38% of 4-thread CPU).
3. **The job handoff.** A worker holds the jobs `Mutex<Receiver>` while blocked in `recv`
   (`src/parallel.rs`, about 418–426).
   - At 4 threads, 3.0–3.5% of worker time goes to the lock and 1.7–2.4% is idle with a job
     queued (occupancy 42009406264, 41976148952).
   - Ceiling about 2–3% of 4-thread wall (INFERRED).
   - This is not the condvar that split condition variables measured.
4. **The Huffman bit buffer is kept in memory.** Each lookup carries a store-to-load forward.
   - 0.5–4.1% of 4-thread wall, plus up to about 0.9% of scanner-starvation idle (INFERRED).
5. **MTF's store-forward chain.** Point about 3–4%, range 1.3–9.4%. The MTF workflow owns
   it. P74-C, the list head held in a ymm register, is its candidate, but there is no 4-thread
   CI evidence and PR #25's 4-thread result was null.
6. **The fused post-walk pass (#183).** Pooled about −1.4% of 4-thread wall over three CI
   rows, range 0 to −2%. That is a statistical-power case, not a failed mechanism. #183 is
   closed and is not reopened without the maintainer.
7. **The build's co-run inflation** (×1.76 on the 9V74, ×2.25 on the 8370C).
   - Its excess tracks alone IPC together with RLE1's, not the number of store streams.
   - Instruction cuts in the build and RLE1 should pay more at 4 threads than at 1 thread.
   - A write-combining build is disfavoured.
8. **Supply** is exhausted at window 16:
   - slot waits under 0.5%;
   - start-up and tail fixed at about 1.9–3.4 ms each, a bench-only cost that is about 0 at
     release scale;
   - steady idle about 1.2–1.8% of 4-thread wall.

   Idle converts to wall at about 0.59–0.72, from #180's own A/B.

**Closed, and why** (the archaeology's verdicts, falsifier-checked):
- At four copies: lanes 16–64, prefetch hints, tile 512/1024, the co-resident 16-lane shape
  and AMAC.
- Teams below 6 cores: 0.71× the workers at 4 threads.
- Compact ψ tables, Precopy and the pair walk: their build costs, or slower on CI.

**compact30's staged retest** is predicted slower at 4 threads by three independent routes:
- the archaeology: +2% to +7%;
- the walk falsifier: −2% to +11%, central +3% to +5%;
- the accounting falsifier: its step work lands on the binding term.

Its CI A/B still runs, because it discriminates the regime. A win on a Zen 4/5 runner would
make the L2-footprint route real. Its full verification is pruned until it wins.

## The decisive experiment, and the fronts it opened

Both explanations of item 1, and placement (item 2), are decided by one instrument: a
**sibling-pair matrix**. A measured kernel (walk, MTF or build) runs on one CPU beside a chosen
load on its SMT sibling:
- idle;
- a walk;
- a plain chain;
- an L1-resident ALU loop;
- MTF;
- the build;
- a streaming read.

A cross-core control runs the same load on the other core. The two explanations predict
different signatures:
- **Core-side:** the ALU-loop sibling costs as much as a walking one.
- **Memory-side:** only memory-heavy siblings cost.
- **Pairing pays:** walk beside compute is cheaper than walk beside walk.

`sweep.yml` today runs one copy, then N identical copies, so it cannot answer this.

Opened as separate workflows:
- **The pair mode**, built and then reviewed as an instrument.
- **Three rival placement constructions**, design-only until the matrix answers, then a
  collision between them:
  - stage-specialised threads, one walker and one compute thread per core;
  - a per-core walk token;
  - software interleaving on one thread per core, in place of SMT.
- **Two isolated prototypes for items 3 and 4.** Each will be registered before its PR
  bench.
- **The partitioned walk**, already running. It adds per-step queue work, which lands on
  what the census says binds.

## What the census does not settle

- **The 8573C's split.** The master bench runner has no co-run factors.
- **No Intel lane or prefetch sweep has run.**
- **The bench's sampler.** It is pinned on the decoder's CPUs at 4 threads, where it costs
  +1.2% to +2.4% of wall with ±4% intervals.
- **Unattributed CPU.** About 6% of 4-thread CPU is unattributed; that is the sampler's
  resolution.
- **First-touch faults.** Every bench run is a fresh process, so about 0.5–2% of 4-thread CPU
  is plausibly first-touch faults: bench-only.
- **The release ETA.** At 4 threads, `wall_n` is the right reading. `wall_n_fixed` under-reads
  by about 21%.
- **Transparent huge pages.** Their setting on the runners is unrecorded. It can be read from
  `/sys/kernel/mm/transparent_hugepage/enabled` in any CI job without a PMU. The change itself
  stays gated on the maintainer.
