# The walk's factorial sweep (#41, #45): a harness artifact, then the curve in decode order

**Result:** no setting beats the shipped one, the room walk at 32 lanes with L1 prefetch and
512 segments, beyond noise at one copy and at four. Three findings come out of the sweep:
- a measurement artifact, caught and fixed;
- the room walk's advantage over the flagged walk holds in decode order;
- four copies at once double each copy's cost, and lanes beyond 12–16 then buy nothing.

The sweep is a diagnostic. The decoder's paired A/B decides what ships.

Every finding is labelled:
- **measured:** a figure from the CI runs below;
- **interpretation:** ours, and untested.

## The artifact (measured)

**The first run** ([40569712252](https://github.com/lmmx/mibz2/actions/runs/40569712252), at
master `d797922`) built each block's T-vector once and walked it at all 59 settings in turn:
- 56 of those settings share the default's 512 segments, so each followed other walks of the
  same block in the same layout;
- the three other segment counts never did, and read 40–75% slower than the default.

**Locally**, with the room walk at 32 lanes and L1 prefetch, in the sweep's call order:

| call | segments | as the sweep ran | build before each walk | 64 MB scrub before each walk |
|---|---|---|---|---|
| 1st | 512 | 4.05 ns/B | 4.18 | 4.79 |
| 4th | 512 | 3.00 | 3.40 | 4.61 |
| then | 256 | 3.93 | 4.02 | 4.68 |
| then | 1,024 | 4.15 | 4.29 | 4.89 |
| then | 2,048 | 4.42 | 4.50 | 5.12 |

**Interpretation:**
- Rebuilding the T-vector before each walk left most of the warm-up in place. That makes
  the warm state something outside the T-vector, most likely the 1.8 MB of segment output
  buffers, reused at the same addresses with the same sizes when the layout repeats.
- A decode never walks a block twice, so every 512-segment row of the first run was
  measured in a state no decode reaches. Its rankings were withdrawn:
  - NTA 1–13% ahead of L1;
  - 24 lanes ahead of 32 with four copies;
  - prefetch saving 34% at 32 lanes.

**The fix** ([#45](https://github.com/lmmx/mibz2/pull/45)): each round decodes the file once
at each setting, block by block (MTF, build, walk, RLE1). Only the walk is timed, and each
walk follows the previous block's walk at the same setting.

**The same lesson for the stage benchmarks:** a benchmark that repeats a walk over retained
working memory answers a different question from one in decode order. Say which one it
measures. `examples/phases.rs` (the bench's stage table) is in the repeated state:
- it keeps each stage's best of `STAGE_REPEATS` (2) runs;
- it times the flagged walk and then the room walk on the same block, with the same
  scratch and the same 512 starts.

So its walk row, the room walk's, is measured warm. The decoder's paired A/B, which
shipped #37, is not affected.

## The curve in decode order (measured)

[Run 40573190665](https://github.com/lmmx/mibz2/actions/runs/40573190665), at `5431d90`:
- 5 rounds; one copy pinned to one CPU, then a copy on each of the runner's 4 CPUs at once;
- the walk alone, in c/B at the runner's reported 2.87 GHz;
- a different runner from the first run's (4.25 GHz reported). The artifacts did not name
  the CPU model then; they do now. So the two runs' absolute figures do not compare.
- no PMU was exposed, so there are no counters.

**The room walk, 512 segments** (four copies: the mean of the four CPUs):

| lanes | 8 | 12 | 16 | 24 | 32 | 48 | 64 |
|---|--:|--:|--:|--:|--:|--:|--:|
| one copy, no prefetch | 8.34 | 6.81 | 6.10 | 5.64 | 5.73 | 5.92 | 5.73 |
| one copy, L1 | 8.01 | 6.36 | 5.35 | 4.87 | **5.00** | 5.15 | 4.96 |
| one copy, L2 | 8.46 | 6.56 | 5.89 | 5.39 | 4.99 | 5.30 | 4.87 |
| one copy, NTA | 7.93 | 6.31 | 5.47 | 5.06 | 4.74 | 4.97 | 4.88 |
| one copy, L1's saving over none | 4% | 7% | 12% | 14% | 13% | 13% | 13% |
| four copies, no prefetch | 13.74 | 12.62 | 12.30 | 12.42 | 12.53 | 12.56 | 12.73 |
| four copies, L1 | 11.97 | 10.60 | 10.23 | 10.28 | **10.37** | 10.46 | 10.83 |
| four copies, L2 | 12.72 | 10.82 | 9.70 | 10.21 | 10.58 | 10.49 | 10.62 |
| four copies, NTA | 12.27 | 10.76 | 10.32 | 10.22 | 10.51 | 10.45 | 10.52 |

**The four CPUs separately**, with four copies running:
- each copy's default (32 lanes, L1) took 9.67, 10.35, 10.85 and 10.62 c/B;
- L1's saving over no prefetch was 11–16% at 8 lanes and 15–22% at 32 lanes across the four;
- the CPUs differ by up to 12% in level but not in the curve's shape. Each copy's file is
  in the run's artifact.

**The flagged walk**, at matched settings, costs about 15–20% more at one copy and 15–25%
more at four (32 lanes with L1: 5.86 against 5.00, and 12.70 against 10.37).

**Against the default, paired by round:**
- **One copy:** no setting is faster beyond noise. The closest are 32 lanes with NTA
  (−1.0%, −6.0% to +4.3%) and 24 lanes with L1 (+0.1%).
- **Four copies:** 16 lanes with L2 is faster on all four CPUs (−3.8%, −6.3%, −8.9%, −6.7%,
  each interval excluding zero). At one copy the same setting is 23% slower (+17% to +29%).
- **Segment counts:** 256, 1,024 and 2,048 sit within noise of 512 at one copy (−0.8%,
  +1.8%, +1.8%). With four copies, 2,048 is 8% slower on two of the four CPUs.

**The shortlisting rule** was a setting that clears zero both alone and with four copies.
No setting does, so nothing goes to the decoder's A/B. The shipped setting is competitive
across the 59 settings tested, on this runner.

## Interpretation

- **Prefetch survives decode order but saves less:** 13% at 16 or more lanes alone and
  15–17% with four copies, against 34% in the warm first run on another runner. Both the
  runner and the state changed, so the size of the drop isn't attributable to either.
- **In decode order, the prefetch's curve flattens about where the no-prefetch curve does,
  near 24 lanes, alone.** In the warm first run, the prefetch arm kept improving to 32 lanes
  after the no-prefetch arm had flattened. We read that as a sign of a limit set by the
  instruction window (#39). This run doesn't show that separation, so the window
  hypothesis lost its main evidence. It is not refuted, but it is not supported either.
- **The shared-offset room walk's advantage over the flagged walk is not an artifact of
  warm repeats.** It holds in decode order, alone and contended.
- **With four copies, each copy's walk costs about twice as much (5.0 → 10.4 c/B), and
  lanes stop helping at 12–16.** That points at a shared resource: the L3's capacity or the
  memory bandwidth. Four copies' working sets (each about 3.6 MB of T-vector, 1.8 MB of
  segment buffers and 0.9 MB each of MTF output and walk output) approach a 32 MB L3,
  which on a 4-vCPU VM may also be shared with other tenants. Contention, not the core's
  window, is the regime a multi-threaded decode runs in.
- **The L2 hint winning at 16 lanes only under contention** is an interaction we can't
  explain without counters. One candidate: under higher latency, the L1's miss buffers
  fill; an L2-targeted prefetch may not hold one (documented for Intel's fill buffers, not
  known to us for AMD), and fewer lanes ask for fewer buffers.

## What it leaves open

- **#42's move structure under contention.** #42's closing comment named this as the one
  condition that would reopen it: the T-vector walk moving towards memory-bound with four
  copies. The doubling above is that condition, measured. A 0.6 MB structure's working set
  is about a sixth of the T-vector's, and the question is whether its walk slows less when
  four share the L3. Pre-registered criterion, as #42's: move build + walk at least 10%
  under build + room walk per copy, with four copies at once; otherwise it closes for good.
- **#39:** the window hypothesis needs a new discriminating test before any experiment
  that adds instructions to the step. The contended regime is the more release-like one to
  test in, since a multi-threaded decode runs in it.
- **#40 (TLB):** four copies' T-vectors at 4 KiB pages are about 3,500 pages, beyond a
  per-core STLB's reach only if the walk's rows spread over all of them at once, which they
  do. Its measurement remains a counter question on a host with a PMU.

## Limits

- One CI run of each kind, on one runner model each, with no hardware counters.
- The four-copy figures are four processes in one VM. A multi-threaded decode is one
  process with four threads, and the paired whole-decoder A/B at 4 threads is its measure.
