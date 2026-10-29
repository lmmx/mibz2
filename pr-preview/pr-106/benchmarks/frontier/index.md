# Throughput–memory frontier

In 2001 Seward set the inverse BWT's space–time trade-off for one block: a smaller table (`indexF`, the byte recovered from the first column) costs more work a step than the merged one (`mergedTL`, the byte kept beside each link, the layout mibz2 uses). The question here is whether that trade-off still holds when several blocks decode at once and their tables share one L3 cache. A table that is slower for one block might be faster for the machine, if its smaller footprint keeps more blocks resident.

**Where it stands:** the mechanism is measured. A shift of the frontier is not.

| finding | status |
| --- | --- |
| Shared cache capacity bounds throughput: the knee sits at about 14.4 MB of live tables, 0.7 of the i9's 20 MB L3, at two block sizes | measured ([workstation journal §9](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md>)) |
| The table is compressible: about 77% of a full block's rows are in 32-row translation groups | measured ([ψ census](<https://github.com/lmmx/mibz2/issues/99>)) |
| A compact table can be navigated as fast, built directly from the last column | not shown |
| A compact table gives more throughput from less memory than the best 32-bit configuration, on one machine and workload | not shown |

## The two charts

They will be drawn from [`frontier.json`](<https://lmmx.github.io/mibz2/pr-preview/pr-106/assets/data/frontier.json>), the page's measured observations, as series are added:

1. **Throughput against concurrent full blocks:** where a smaller table would overtake the 32-bit one, if it does, as more blocks are live.
2. **Throughput against memory:** each configuration's measured throughput and memory, with the ones nothing beats on both marked. This is the frontier.

**The rules the charts keep:** - **One machine and one workload a chart.** An L3 effect on the i9 may appear at another concurrency, or not at all, on another CPU. - **Two kinds of memory, never mixed:** the live tables' bytes (computed: live full blocks × 900,000 rows × 4 bytes for today's table) and the process's peak RSS (measured). Peak RSS also counts the scanner's slots, the output and every worker's buffers: at 4 threads on CI it is about 74 MB, against 14.4 MB of tables. - **Two levels, never mixed:** the full decoder (a file to verified output) and the research harness's inversion (MTF, build and walk). The headline is the full decoder; the inversion shows the mechanism. - **Releases are the record.** Selected pull requests and research runs appear as labelled overlays. - **Only measurements.** Footprints of designs that don't exist yet, and models, stay in the text. - **The frontier is judged against the best alternative,** not a convenient one. A compact table at four blocks beating today's at four blocks is a step; the frontier moves only if it beats today's best configuration at any block count.

## What is measured so far

Two series exist, on the i9 (Intel i9-10900K, 10 cores, 20 MB L3). They are different runs at different levels, so they are separate series, never one line.

**1\. The full decoder, today's table, by worker count** (unpinned; the bench corpus's bytes recompressed with lbzip2 -9; [workstation journal §9](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-28-workstation-scaling.md>)):

| workers | live tables | MB/s | CPU-seconds |
| ---: | ---: | ---: | ---: |
| 2 | 7.2 MB | 707 |  |
| **4** | **14.4 MB** | **1026** | 0.834 |
| 6 | 21.6 MB | 889 |  |
| 8 | 28.8 MB | 698 |  |
| 10 | 36.0 MB | 645 | 2.921 |

Past 4 workers, more tables than the L3 holds make each block's work dearer: from 4 to 10 workers the CPU-seconds rise 3.5× and throughput falls. Peak RSS was not recorded in this run.

**2\. The inversion, production against cooperative teams** (every thread pinned; the research harness's MTF, build and walk; [coop journal §17.4](<https://github.com/lmmx/mibz2/blob/research/coop-fused/docs/journal/2026-10-28-cooperative-inversion.md>)):

| configuration | live tables | cores | MB/s | against p4 (paired) |
| --- | ---: | ---: | ---: | ---: |
| p4, production | 14.4 MB | 4 | 1206 | 1 |
| y3x3, cooperative | 10.8 MB | 9 | 1657 | 1.375 ±0.022 |
| y4x2, cooperative | 14.4 MB | 8 | 1725 | 1.439 ±0.020 |

A cooperative team shares one block's table among k cores, so N teams keep N tables live on N × k cores. - y3x3 runs faster than p4 from fewer live tables. p4 is production's best worker count on this machine in every scaling run (series 1). - So at this level, in throughput against live table bytes, sharing a table already does better than every production configuration measured. - **It is not yet a frontier result:** - it is the inversion, not the decoder; - it is pinned; - its memory axis is computed table bytes, not peak RSS; - it uses more cores (CPU-seconds are not yet measured).

## What fills it in

- **The cooperative decoder's A/B** on the i9 (the integration's step 7) runs the full decoder unpinned, with production at 4–10 workers and cooperative teams beside it, and measures peak RSS and CPU-seconds. Its observations will be written in this page's format: the first series with measured memory.
- **The CI benchmark's release runs** measure wall time, CPU and peak RSS at 1, 2 and 4 threads on every release, on GitHub's 4-vCPU runners. That is below the i9's knee, so it can't show a crossover, but it keeps a frontier for every release.
- **Compact tables** join when one exists: a 24-bit link-only table (\#79) first, then others if the census and the walk's costs justify them.

The research behind this page: Seward's trade-off revisited, verbatim ([the researcher](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier/seward-frontier-research.md>), [the coordinator](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-frontier/seward-frontier-coordinator.md>)), and the [literature study](<https://github.com/lmmx/mibz2/blob/master/docs/journal/2026-10-29-literature-study.md>).
