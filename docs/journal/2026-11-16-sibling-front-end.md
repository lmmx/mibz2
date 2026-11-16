# 2026-11-16 · The sibling front end: a design and its bound; not built

**The question.** Past the physical cores, mibz2 0.5.1 runs one lazy-letters worker a core
on the i9-10900K and leaves each core's second hardware thread idle (`plan_with`, #468).
Could that sibling run the front end of the worker's *next* block while the worker inverts
the current one? The pairing would be compute-bound work beside memory-bound work, not two
walks on one core.

**The answer, before any build.** The bound does not clearly clear the cost of SMT sharing,
so nothing is built. That is the stopping condition set for this task (the Coordinator's
message relaying the Pathfinder's constraints, 16 November):
- the front end's share of a worker's time at 10 workers is 9-16%;
- the worker-bound gain after SMT sharing is +5% to +10%;
- the decoder at 8-10 workers sits on a memory plateau that this design does nothing to
  relieve.

The design is written down here, with a gate that would reopen it (§6). The draft code
(`2026-11-16-sibling-front-end/sibling.rs.txt`) was **never compiled or run**.

Labels: **record** (a figure from a bundle or a CI run on the record), **model** (arithmetic
from stated assumptions). No timing was taken for this note.

## 1. What the front end is, in today's decoder

The brief described a worker's pipeline as bit reading and Huffman decoding, then MTF and
RLE2, then inversion. **In mibz2 the bit reading and Huffman decoding are not the worker's.**
The scanner thread does them (`parallel.rs`, `scanner` → `Scanner::next`), because a block's
end is known only once its symbols are read. A worker's block (`decode_block`) is:

1. **The front:** `mtf::mtf` into `ll` and `counts`. For a block of few symbols, it is
   `runs::front` into the run records instead. Then the primary index is checked.
2. **The inversion:** Cells (`slots`, `induce`, `write_cells`) or lazy letters, or the
   kernel's fallback. This is memory-bound.
3. **The finish:** RLE1 and the block's CRC (`finish`).

So a sibling could take step 1, and perhaps step 3 of the block before. It cannot take the
Huffman stage. That stays serial on the scanner.

## 2. The front end's share (record, then model)

**The i9, one worker** (record: `cells-phase/run-20261114T235407Z`, phase-counter scopes,
Cells with a slot for every letter, Wikidata's 431 blocks, the median of 600 decodes). Each
scope's time across the run, in ms:

| scope | 1 worker | 4 workers, on 4 cores | ×4/1 |
|---|--:|--:|--:|
| `decode_block`, whole | 312 | 487 | 1.56 |
| induction | 103 | 156 | 1.52 |
| writer | 46 | 73 | 1.60 |
| rest: MTF, `slots`, RLE1, CRC, the run-record and fallback blocks | 164 | 258 | 1.57 |

**MTF has no scope of its own there.** Its price comes from CI's stage table on #468's head,
on a 3.69 GHz Xeon: MTF and RLE2 take 1.58 c/B, RLE1 0.19 and CRC 0.26 (record).

For the i9, MTF is taken at the same cycles per byte, at 4.3-4.9 GHz (model):
- MTF is 66-76 ms of the 312 ms, so **f₁ ≈ 0.21-0.24** at one worker;
- with lazy letters' larger inversion (+12% instructions in the whole decoder), about
  **0.19-0.22**;
- RLE1 and CRC are another 19-22 ms, about 0.06.

**At 10 workers** (record: `lazy-occupancy/run-20261116T032432Z`, eight copies), the workers'
CPU per copy is:
- 0.743 s at 10 threads, pinned one CPU a core by `--spread`;
- 0.931 s at 20 threads asked, unpinned.

What MTF takes there depends on how much contention inflates it (model):

| MTF under contention | MTF a copy | f₁₀, pinned | f₁₀, at 20 asked |
|---|--:|--:|--:|
| not inflated (a 256-byte list and streamed symbols) | 66-76 ms | 0.09-0.10 | 0.07-0.08 |
| inflated as the "rest" scope was at 4 workers (×1.57) | 104-119 ms | 0.14-0.16 | 0.11-0.13 |

**f₁₀ ≈ 0.09-0.16.** The Pathfinder's estimate (about 10% of the workers' CPU) is the low
end of this range. A direct measurement would need an MTF scope in `phase-counters`, or the
Cells stages in `examples/phases.rs`. Neither exists today.

## 3. The bound (model)

**The ceiling before any cost.** If inversion binds and the front is wholly hidden, a
worker's time a block falls to (1 − f), so the ceiling is 1/(1 − f):
- MTF alone: **1.10-1.19** at 10 workers;
- with RLE1 and CRC moved too (+0.03): **1.13-1.23**.

**What SMT sharing takes back.** While the front works, the inversion runs beside a busy
sibling:
- The inversion shares the core for a fraction d = f/(1 − f) of its time.
- On Intel, a walk beside a busy sibling ran ×1.46 at four copies (record: phases
  42008210159, Xeon 8370C, 2 cores × 2 threads). That was beside another stage of the
  decoder, not a pure compute loop. Taking 0.46 as the cost while shared, the inversion
  slows by d × 0.46.
- The worker's time is then (1 − f)(1 + 0.46·f/(1 − f)):
  - f = 0.09: 0.952, **+5.0%**;
  - f = 0.16: 0.913, **+9.5%**.

**The reference is the pinned arm.** The sibling layout pins each worker to its core, so its
honest reference is `mibz2-workers-lazy-cores-pin` (#477): ten lazy workers, one CPU a core.
It is not the unpinned shipped plan, which #477 expects to lose 1-5% to pinning on its own.
The +5% to +10% above is already measured against pinned workers.

**The ceiling the record puts above all of this: the decoder is not worker-bound at 8-10
workers.**
- **Lazy workers on eight copies run flat from 8 to 10 workers** (record, lazy-occupancy):
  0.781, 0.772 and 0.810 s. More worker capacity bought nothing past 8. The sibling front
  end is more worker capacity, about 10-16%.
- **One copy with a slot for every letter runs flat from 4 workers to 10** (record:
  `cells-imc/run-20261114T180428Z`), while its DRAM traffic rises:

  | workers | 4 | 6 | 8 | 10 |
  |---|--:|--:|--:|--:|
  | DRAM bytes per output byte | 15.5 | 20.4 | 22.7 | 24.2 |
  | wall, s | 0.162 | 0.162 | 0.172 | 0.180 |
  | DRAM traffic, GB/s (model) | 20 | 26 | 28 | 28 |

  That is about 26-28 GB/s of mixed random reads and writes, a plausible practical limit for
  this platform's two DDR4 channels.
- **Lazy letters won at 8-10 workers by holding 38% less memory.** That is a memory-bound
  regime's signature.
- **The sibling front end does not relieve memory, and adds to it.** Each pair holds a second
  block's `ll` (up to 0.9 MB) one block ahead: up to 9 MB more live data across ten pairs, in
  a 20 MiB L3 that is already oversubscribed.
- **The Pathfinder reads the step past the cores as placement and oversubscription, not
  memory** (#145 LOG 6912912161). If #477's rule 4 confirms it, part of what is left for a
  sibling design is what pinning alone already takes.

**The bound, then:**
- **+5% to +10% only if the workers bind.** The record says they don't at 8-10 workers.
- **About 0 or below if memory binds,** which the plateau and lazy letters' win both suggest.
- **No point inside that range clearly clears the SMT cost and the pinned reference together.**

## 4. The design, for the record

- **The pairing.** As the shipped plan, `min(threads, phys)` workers. The first
  `min(workers, threads − workers, pairs)` of them each get a front thread on the same core.
  At 12, 16 and 20 threads on the i9, that is 2, 6 and 10 pairs, a dose-response by
  construction. At 10 threads there are no fronts, and the arm is pinned workers, so it is
  its own pinning control.
- **The topology (Linux).**
  - The usable CPUs are `Cpus_allowed_list` intersected with `/sys/devices/system/cpu/online`.
  - A pair is a core whose `cpu*/topology/thread_siblings_list` names exactly two CPUs, both
    usable.
  - The i9's pairs are (c, c + 10).
- **Pinning.**
  - Each worker and its front are pinned to **the core's two CPUs**, not one CPU each, with
    `sched_setaffinity`.
  - This is the core-mask lesson of `2026-11-05-corun-placement.md`: the scanner, reader and
    feeder float, and can use a sibling while the front is idle.
  - Workers without a front are pinned to their own core, one a core.
- **The fallback.** If the topology can't be read, there are no fronts and no pins: the
  shipped plan's workers exactly. If `sched_setaffinity` is refused, the threads run
  unpinned and the output is the same.
- **The hand-off.**
  - Each pair circulates two front sets (`ll`, `counts`, the run records). The front fills
    one while the back inverts the other, so the front runs one block ahead and no further.
  - The back swaps a set's buffers into its own `Scratch` with `mem::swap`, so nothing is
    copied. It runs the second half of `decode_block`, then swaps them back.
  - There are two channels a pair: front to back (bounded) and the free sets back.
  - Nothing deadlocks. A front waits only for a free set (its back always returns one) or a
    job (the scanner). A back waits only for its front. Neither holds a block that the reader
    needs while it waits on the scanner.
- **Output and errors.** `front_half` and `back_half` together are `decode_block` for a
  kernel without checkpoints, with the same errors in the same order. The back publishes
  each block through `publish`.
- **Memory a pair.** One extra `ll` (at most 0.9 MB), the run records and 1 KB of counts:
  about +1 MB a pair, so +10 MB at ten pairs.
- **Shared machinery: one affinity module.** #477's rule 4, if it passes, has the decoder
  place its own threads one CPU a core (new unsafe under B5). Any sibling design needs the
  same mechanism.
  - So both should use one library module, say `src/affinity.rs`: the usable CPUs grouped by
    core from sysfs, and `pin(cpus)` through `sched_setaffinity`. It would be the crate's
    only `extern "C"` and carry one SAFETY argument.
  - `examples/ab.rs` already has its own copy for the harness (`cores()`, `one_cpu_a_core`).
    The library's would replace the sketch's `pin` and sit beside the existing `Fs` and
    `cpu_list` discovery in `parallel.rs`.
  - It should be written once, for rule 4 first.

## 5. Prior work, so it is not repeated

- **`2026-11-05-corun-placement.md`, C1 "stage-specialised pairs".**
  - It is this design's ancestor: a walker and a compute thread a core.
  - It was held for a "pair matrix" (walk beside MTF, beside an ALU loop, beside a walk, on
    Intel and AMD) that was never run.
  - Its scope was CI's 4-thread row. It pruned single-CPU pinning in favour of core masks,
    and engaging on Intel without a measured Intel row.
  - Not refuted, not tested.
- **The pipelined team** (`2026-11-07-team-pipeline-result.md`) overlapped a leader's MTF,
  RLE1 and CRC with its helpers' build and walk, on physical cores.
  - It gained +4.2% at 3 × 3 and +3.2% at 3 × 4 on the i9, for 8-11% more CPU.
  - The overlap fired on about a third of the blocks.
  - That is the nearest measured analogue. It fits the low end of the bound in §3.
- **The co-resident walk** (`2026-11-05-coresident-walk.md`): a narrower walk for SMT
  co-residency, not reproduced on the current kernel and closed.
- **0.5.1's cap** (#468): twenty lazy workers (two a core) were 22% (one copy) and 29% (eight copies) slower than ten
  (`lazy-occupancy`), so
  walk beside walk loses on the i9.

## 6. What would reopen it: a cheap gate, before any decoder code

1. **Measure f₁₀ on the i9.** Add an MTF scope to `phase-counters`, or the Cells and lazy
   stages to `examples/phases.rs`. Read MTF's share of a worker's time at 10 pinned workers.
   Reopen only if it is ≥ 0.15.
2. **One row of the pair matrix on the i9.**
   - The lazy induction on CPU c, beside an MTF loop on its sibling c + 10, against an idle
     sibling. Also MTF beside the induction.
   - Reopen only if the induction's factor beside MTF is ≤ 1.15, so that the SMT cost in §3
     falls from about 0.46 to about 0.15.
3. **#477's rule 4.** If pinning is a lever, the affinity module gets built anyway, and a
   sibling arm's marginal cost falls to the hand-off alone.

**If 1 and 2 both pass,** build the arm `mibz2-sibling` (research feature, never the
default) from the draft here. Run it at 10, 12, 16 and 20 threads, on Wikidata and on eight
copies, against both `mibz2` and `mibz2-workers-lazy-cores-pin`, with the rule posted before
its data. The rule as it would be registered:
- **Void** on any error or differing output.
- **Sanity:** at 10 threads, the arm against `-cores-pin` spans 1.
- **Gain:** the arm against `-cores-pin` on eight copies is wholly below 1 at 16 and 20
  threads, with a point estimate ≤ 0.95 at 20, and the same direction on one copy.
- **Dose-response:** the 12-thread estimate lies between 1 and the 20-thread one.
- **Reported, not gated:** CPU, peak RSS (expected +10 MB) and the scanner's busy share.
