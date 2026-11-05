# 2026-11-05: Co-run placement on SMT: three rivals, a collision, and the kernel it found

**The question.** The binding census (`2026-11-05-binding-census.md`) found the walk's AMD
co-run excess, ×2.1–2.2 at four copies against ×1.2–1.4 for the compute passes. It is worth up
to about 16–18% of 4-thread wall. The census could not say what causes it. Placement asks whether
the decoder can choose what runs beside the walk on its SMT sibling, so that the walk avoids its
costly partners.

**How it was tested.** Design only: three rival constructions, then a collision between them.
- Each rival's cost model is calibrated to the observed four-copy stage factors of the phases
  runs: 7763 42011789918, 9V74 42020095581, 8370C 42008210159 (`phases/factors.py`).
- Each is priced under the three outcomes the pair matrix will decide between:
  - **C:** any busy sibling costs the walk about 2×;
  - **M:** only memory-heavy siblings (build, walk, streaming read) cost;
  - **P:** the walk beside compute is materially cheaper than the walk beside a walk.
- Nothing was timed:
  - C1 and C3 are designs;
  - C2's patch (`c2-token/token-probe.patch`) was not compiled.

**Every figure below is modelled (INFERRED)** as 4-thread wall against master, unless marked
otherwise. The models, the collision's probes and the full record (`results.json`) are in
`2026-11-05-corun-placement/`, and each runs from there.

## The rivals

- **C1, stage-specialised pairs.** Each core pins a walker and a compute thread. Only the walker
  walks, and both share every other stage. A core never runs two walks at once.
- **C2, the walk token.** One token a core: a worker may walk only while holding it. Otherwise
  it computes or hands the table to a sleeping sibling. The table pool is two a core.
- **C3, software interleaving.** One thread a core, with the siblings left idle. MTF and scanner
  work is interleaved into the walk's rounds, to fill its stall time in-thread.

| construction | C | M | P |
|---|---|---|---|
| C1 | −4.5% to +7.8%, central +1.5% | AMD −10% to −19% (central −14%); Intel −3% to −12% | AMD −1% to −12% (central −5%); Intel +2% to −9% |
| C2 | AMD −2.5% to +5.8% (central +1%); Intel up to +8.5% (central +3%) | walk token AMD −11% to +2% (central −3.5%); memory token −7% to −13% | AMD −15% to 0% (central −5%); Intel −6.5% to +1% |
| C3 | 7763 +4% to +15%; 9V74 −6% to +6% | dominated by pairing | dominated by pairing |

The C1 and C3 rows are as corrected in the collision, which follows.

## What the collision found

**Each model's own failure.**
- **C1** pins each thread to a single CPU.
  - Floating threads then cost walk throughput the walker cannot recover by migrating. These
    are the scanner (9.7–10.7% of 4-thread CPU), the reader and the bench sampler. The cost is
    0 to +3%.
  - Its sim locks the two threads' walks together (walk beside walk for 0.84–0.91 of walk
    time). That makes its M gain the top of the M range, not the middle.
  - Its gate, step A, runs a thread layout that would not ship.
- **C2's** calibration is the mild M, so its M central is the bottom of the range.
  - It credits short idles at the alone rate. In a Hyper-V guest a sibling may poll before it
    halts, and that is unmeasured.
  - It loses on Intel under C, yet it would engage on any SMT topology.
- **C3's filler does not fit the slack.**
  - Zen 3 leaves 48 ROB entries beside a 208-macro-op round, against about 140 macro-ops of MTF
    and scanner filler (`c3-interleave/*.asm`).
  - Absorption e is then about 0.35–0.55, against its break-even of 0.63–0.69.
  - Its scanner is one serial chain, priced as if it were split across both cores.
  - The idle siblings it needs belong to the caller: the reader, the sampler and, in a release,
    wikidata-pq's other stages.

**The shared assumption: the walk's own footprint is a constant.**
- C1 and C2 make the walk's co-run cost a function of what the sibling runs. C3 makes it a
  function of whether the sibling runs at all.
- Yet the largest 4-thread lever measured was a change in the walk's own claim on the core:
  #171's staged stores cut 4-thread CPU by 16–18% with 1 thread flat (OBSERVED).
- The four-copy factor follows each kernel's own work: on the 7763, the plain chain ×1.18
  against the room walk ×2.12 (OBSERVED, phases).
- On Zen 3 and 4 the ROB and the store queue are statically partitioned once both threads
  run.
- If the walk's SMT loss is a halved window, the matrix reads C by mechanism, and no placement
  recovers it.

**The third possibility: choose the kernel by co-run, not the placement.** Select a walk that
needs less window per miss in flight, only where SMT siblings co-run (threads > physical cores).
- **(a) The AVX2 gather walk** (`research/gather-walk`, aed6a213). It runs 31.3% fewer
  instructions and keeps no rows round trip. It was only ever timed locally on a VM without SMT,
  never on CI and never at four copies.
- **(b) A one-store-a-step room walk.** The next row is staged as a u32 in the tile and read
  back, halving the step's store-queue entries.
- **Bounds:** AMD 0 to −20% (central −3% to −8%), Intel −3% to +2%. The 1-thread and 2-thread
  rows are unchanged by construction.
- **Against it:** the per-core miss buffers cap the line rate whatever the kernel, and Zen 5
  loses as much at four copies despite its larger partitions.
- **Relation to C1 and C2:** under C it is the first lever. Under M or P it stacks with them.

## Decisions

**Pruned now.**
- **C3 as a decoder build, in every variant.** Its absorption probe is kept only as a
  contingent measurement, if the window-light walk fails.
- **C1's single-CPU pinning,** and its arm that keeps the scanner on the walker CPUs. Any
  pairing design uses C2's core-mask pinning.
- **C1's step A as specified.**
- **Engaging either pairing design on Intel** without a measured Intel row.

**Held for the pair matrix, with no PR before it.**
- **C1** is the first choice under M; **C2** under P.
- Both are rejected, with a line in the Rejected list, the moment the matrix reads C: every
  light sibling within 0.2 of walk beside walk, on both Zen models.

**Built first, whatever the outcome: the window-light walk.**
- Both variants are being built on `research/window-gather` and `research/window-onestore`.
- They are judged first as rows of the four-copy walk-only sweep.
- **Gate 0:** variant over room ≤ 0.95 at four copies, 95% interval excluding 1, on a Zen 3 or
  Zen 4 runner, with no model above 1.03.

**Added to the pair matrix by the collision.** It now needs:
- siblings: a PAUSE spin, an L1 store stream, an L2-resident 32-lane chase, the same block
  started together, and a duty-cycled ALU loop for short idles;
- the other core run both idle and loaded;
- the kernel axis;
- the median of rounds alongside the fastest.

**Outside the matrix,** and cheap:
- each runner's guest idle path: the cpuidle driver and governor, and `/proc/cmdline`;
- the production decoder's sibling-resolved walk-beside-walk share, which says whether today's
  walks lock together.

**Scope.** Every construction here engages only where threads exceed physical cores, which is CI's
4-thread row. wikidata-pq decodes with one thread a physical core. So none of them, the
window-light walk included, moves the release ETA as configured. That is weighed before any of
them merges.
