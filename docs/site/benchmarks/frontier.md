---
description: "Whether mibz2 can decode faster with less memory when blocks share the cache: the question, the method, and what has been measured so far."
---

# Throughput–memory frontier

In 2001 Seward set the inverse BWT's space–time trade-off for one block: a smaller table
(`indexF`, the byte recovered from the first column) costs more work a step than the
merged one (`mergedTL`, the byte kept beside each link, the layout mibz2 uses). The
question here is whether that trade-off still holds when several blocks decode at once
and their tables share one L3 cache. A table that is slower for one block might be faster
for the machine, if its smaller footprint keeps more blocks resident.

**Where it stands:** the mechanism is measured, and so is a first shift. On the i9 and the
Wikidata corpus, the full decoder with cooperative teams dominates every worker count
measured (4 to 20) on both throughput and peak RSS (series 3). Whether it also does at 1
to 3 workers, the low-memory end, is not yet measured.

| finding | status |
|---|---|
| Shared cache capacity bounds throughput: the knee sits at about 14.4 MB of live tables, 0.7 of the i9's 20 MB L3, at two block sizes | measured ([workstation journal §9](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md)) |
| The table is compressible: about 77% of a full block's rows are in 32-row translation groups | measured ([ψ census](https://github.com/lmmx/mibz2/issues/99)) |
| Sharing a block's table between threads moves the full decoder's throughput–RSS frontier: every worker count from 4 to 20 is dominated by a team configuration | measured on one machine and workload, at 4 threads and above ([decoder journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-cooperative-decoder.md), series 3) |
| A compact table can be navigated as fast, built directly from the last column | not shown |
| A compact table gives more throughput from less memory than the best 32-bit configuration, on one machine and workload | not shown |

## The two charts

Both are drawn from [`frontier.json`](../assets/data/frontier.json), the page's
measured observations. Point at a mark, or Tab to it, for its values. Every value is also
in the tables [below](#what-is-measured-so-far).

1. **Throughput against memory:** each configuration's measured throughput and peak RSS,
   with the ones nothing beats on both ringed and joined by their step. This is the
   frontier. It draws series 3, the only one with measured RSS: the workers (arm
   `mibz2`), fixed teams of 2 and of 3, and the policy (v0.2.0's default). Where one plan
   ran at several thread counts, its marks are noise around one configuration. No worker
   counts below 4 were measured, so it is not yet the whole frontier.
2. **Throughput against concurrent full blocks,** as live table memory (blocks live ×
   3.6 MB, computed): where a smaller table would overtake the 32-bit one, if it does, as
   more blocks are live. No compact table exists yet, so it shows today's table and the
   knee measured in the [workstation journal §9](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md):
   series 1's worker counts and series 3's workers and teams, a panel each, since they
   are different workloads. Series 2, the inversion, is a different level: it has its own
   small panel and axes, never the decoder's.

<div id="frontier-charts" data-src="../../assets/data/frontier.json" markdown>
The charts are drawn here by JavaScript. The tables
[below](#what-is-measured-so-far) give every value they show.
</div>

**The rules the charts keep:**
- **One machine and one workload a chart.** An L3 effect on the i9 may appear at another
  concurrency, or not at all, on another CPU.
- **Two kinds of memory, never mixed:** the live tables' bytes (computed: live full
  blocks × 900,000 rows × 4 bytes for today's table) and the process's peak RSS
  (measured). Peak RSS also counts the scanner's slots, the output and every worker's
  buffers: at 4 threads on CI it is about 74 MB, against 14.4 MB of tables.
- **Two levels, never mixed:** the full decoder (a file to verified output) and the
  research harness's inversion (MTF, build and walk). The headline is the full decoder;
  the inversion shows the mechanism.
- **Releases are the record.** Selected pull requests and research runs appear as
  labelled overlays.
- **Only measurements.** Footprints of designs that don't exist yet, and models, stay in
  the text.
- **The frontier is judged against the best alternative,** not a convenient one. A
  compact table at four blocks beating today's at four blocks is a step; the frontier
  moves only if it beats today's best configuration at any block count.

## What is measured so far

Three series exist, on the i9 (Intel i9-10900K, 10 cores, 20 MB L3). They are different
runs at different levels, so they are separate series, never one line.

**1. The full decoder, today's table, by worker count** (unpinned; the bench corpus's
bytes recompressed with lbzip2 -9; [workstation journal §9](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md)):

| workers | live tables | MB/s | CPU-seconds |
|--:|--:|--:|--:|
| 2 | 7.2 MB | 707 | |
| **4** | **14.4 MB** | **1026** | 0.834 |
| 6 | 21.6 MB | 889 | |
| 8 | 28.8 MB | 698 | |
| 10 | 36.0 MB | 645 | 2.921 |

Past 4 workers, more tables than the L3 holds make each block's work dearer: from 4 to 10
workers the CPU-seconds rise 3.5× and throughput falls. Peak RSS was not recorded in
this run.

**2. The inversion, production against cooperative teams** (every thread pinned; the
research harness's MTF, build and walk; [coop journal §17.4](https://github.com/lmmx/mibz2/blob/research/coop-fused/docs/journal/2026-10-28-cooperative-inversion.md)):

| configuration | live tables | cores | MB/s | against p4 (paired) |
|---|--:|--:|--:|--:|
| p4, production | 14.4 MB | 4 | 1206 | 1 |
| y3x3, cooperative | 10.8 MB | 9 | 1657 | 1.375 ±0.022 |
| y4x2, cooperative | 14.4 MB | 8 | 1725 | 1.439 ±0.020 |

A cooperative team shares one block's table among k cores, so N teams keep N tables
live on N × k cores.
- y3x3 runs faster than p4 from fewer live tables. p4 is production's best worker count
  on this machine in every scaling run (series 1).
- So at this level, in throughput against live table bytes, sharing a table already does
  better than every production configuration measured.
- **It is not yet a frontier result:**
  - it is the inversion, not the decoder;
  - it is pinned;
  - its memory axis is computed table bytes, not peak RSS;
  - it uses more cores (CPU-seconds are not yet measured).

**3. The full decoder, workers against cooperative teams, with measured memory**
(unpinned; the bench corpus, 209.7 MB decoded; 20 interleaved rounds; the
[decoder journal](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-cooperative-decoder.md)
and its raw files). One point per distinct plan: where two arms ran the same plan (the
policy at 8, 10 and 20 threads is 4 teams of 2 on this machine; 2 teams of 3 ran at both 6
and 8), the table gives the range of their medians, which is noise.

| plan | live tables | threads | MB/s | against p4 (paired, 95%) | peak RSS | CPU-s | nondominated |
|---|--:|--:|--:|--:|--:|--:|:-:|
| 4 workers (p4) | 14.4 MB | 4 | 1007 | 1 | 76 MB | 0.84 | no |
| 6 workers | 21.6 MB | 6 | 858 | 0.861 (0.842–0.881) | 102 MB | 1.45 | no |
| 8 workers | 28.8 MB | 8 | 674 | 0.693 (0.673–0.714) | 128 MB | 2.41 | no |
| 10 workers | 36.0 MB | 10 | 632 | 0.644 (0.621–0.668) | 152 MB | 2.97 | no |
| 20 workers | 72.0 MB | 20 | 471 | 0.479 (0.462–0.497) | 267 MB | 7.31 | no |
| 1 team of 3 | 3.6 MB | 3 | 530 | 0.540 (0.524–0.556) | 35 MB | 0.77 | **yes** |
| 2 teams of 2 | 7.2 MB | 4 | 891 | 0.904 (0.885–0.923) | 50 MB | 0.73 | no |
| 2 teams of 3 | 7.2 MB | 6 | 987–988 | 1.004–1.006 | 50 MB | 0.82 | **yes** |
| 3 teams of 2 (the policy at 6) | 10.8 MB | 6 | 1155–1189 | 1.166 (1.123–1.210) to 1.200 | 64 MB | 0.81–0.84 | no |
| 3 teams of 3 | 10.8 MB | 9 | 1234 | 1.264 (1.211–1.318) | 64 MB | 0.96 | **yes** |
| 4 teams of 2 (the policy at 8–20) | 14.4 MB | 8 | 1223–1256 | 1.243–1.274; at 20, 1.257 (1.197–1.320) | 77 MB | 1.02–1.04 | **yes**, by its best median |
| 5 teams of 2 | 18.0 MB | 10 | 1147 | 1.162 (1.121–1.204) | 91 MB | 1.36 | no |
| 6 teams of 3 | 21.6 MB | 18 | 1084 | 1.095 (1.060–1.132) | 105 MB | 2.16 | no |
| 10 teams of 2 | 36.0 MB | 20 | 825 | 0.838 (0.813–0.864) | 155 MB | 3.76 | no |

Peak RSS is measured (the median run's, in decimal MB). Live tables are computed. The
nondominated column compares medians on throughput (higher) and peak RSS (lower) within
this series.

What it shows, and what it doesn't:
- **Every worker count measured is dominated.** p4, the decoder's best worker count here,
  is beaten on both axes by 3 teams of 2 (the policy at 6 threads: 1.166× with its lower
  bound 1.123, at 64 MB against 76) and by 3 teams of 3.
- **The nondominated set is all teams:** 1×3, 2×3, 3×3 and 4×2. Measured by medians,
  4×2 stays on the set by its best repeat. Its throughput is tied with 3×3's within noise
  (their intervals overlap almost entirely), at 13 MB more RSS and about 6% more CPU. 3×3
  is a lead for refining the policy, not yet a result.
- **Not a whole-frontier result.** No worker counts below 4 were measured in this run. At
  1 to 3 workers, today's decoder may still own the low-memory end: series 1 has 2
  workers at 707 MB/s, but without RSS. Filling that in needs the same A/B at 1, 2 and 3
  threads.
- **One machine, one workload, one level (the full decoder).** Threads above 10 share
  physical cores with SMT siblings.

## What fills it in

- **The low-memory end on the i9:** the same A/B at 1, 2 and 3 threads, where today's
  workers and the smallest teams meet.
- **The CI benchmark's release runs** measure wall time, CPU and peak RSS at 1, 2 and 4
  threads on every release, on GitHub's 4-vCPU runners. That is below the i9's knee, so it
  can't show a crossover, but it keeps a frontier for every release.
- **Compact tables** join when one exists: a 24-bit link-only table (#79) first, then
  others if the census and the walk's costs justify them. They are judged against the
  cooperative decoder now, not against the workers.

The research behind this page: Seward's trade-off revisited, verbatim
([the researcher](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier/seward-frontier-research.md),
[the coordinator](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier/seward-frontier-coordinator.md)),
and the [literature study](https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-literature-study.md).
