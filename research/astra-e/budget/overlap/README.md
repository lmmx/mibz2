# Can early block release hide enough MTF work?

The independently timed edge decoder needs another 20.93, 25.20, 29.08 and 23.95 ms over Wikidata ×4 at W7, W8, W9 and W10. Moving the same worker's MTF/run collection ahead of the block's Huffman EOB yields only 1.26–3.07 ms in a calibrated FIFO simulation. A separate necessary capacity test allows both the serial scanner and all W workers to occupy the entire target time. Even its most favorable bound over 100 pairings of measured per-block cost shapes covers only 77%, 61%, 53% and 70% of the required saving. Both results are conditional on the per-block service shapes. The edge target receipt contains whole-process timing and RSS. It contains no per-block target release or stage timestamps.

## Actual precedence and the attainable schedule

The 0.5.6 worker gets whole blocks in receive order from one queue. A scanner waits for a free slot, then calls Scanner::next. It learns the block end and emits the vector only after reading EOB. While it scans the next block, any free worker can receive the preceding one. Each worker keeps its scratch and completes MTF/run collection, the marked inverse, RLE1 and CRC on the same thread. The ordered reader takes completed blocks by place and returns a slot on take. The window is 4W blocks in the edge run. The worker assignment is dynamic. A long earlier block can hold the ordered reader even when later blocks are ready. These queue, scanner and submit/receive implementations have identical Git blobs in release source 530ed404a014a414ef0b62adfd947fa14d4533e6 and measured candidate source badb2aec3624bd65606c429b2e6f79e9d9d0050f. The candidate's changes to runs.rs expose existing run fields to research. [Pipeline](../../../../src/parallel.rs) · [Independent edge result](../../workstation/edge-i9.md)

For block i write A for scan start, E=A+s for issue, V for the assigned worker's availability, f for its frontend duration and d for all its worker service. The atomic ready time is max(V,E)+d. Give an early-release oracle every symbol needed for f at A, free publication and zero waiting overhead, while requiring the inverse to start after E. Its earliest ready time is

```
max(E, max(V,A) + f) + d - f = max(V+d, E+d-min(s,f)).
```

Thus the cross-stage discount is at most delta_i=min(s_i,f_i). No reconstruction work is removed. For fixed services and no scanner slot feedback, release improvements enter a critical path only once. The last block's take and any later drain can move at most max(delta_i), independent of the block count. This includes startup and drain. Steady-state throughput already overlaps adjacent blocks.

With K slots, a chain can collect another discount only after a worker result crosses ordered take and returns to the scanner through its capacity check. A taken block j first permits the scanner to start j+K. Consequently successive discounted indices are at least K apart. The maximum discounted chain obeys

```
M(0) = 0
M(i+1) = max(M(i), delta_i + M(max(0,i+1-K))).
```

The same bound holds with work-conserving FIFO dispatch onto identical workers when assignments change between the two schedules. Inductively, the available-worker vector sorted by availability can advance coordinatewise by at most M(i). Replacing its earliest coordinate with the new ready time and re-sorting preserves the coordinatewise bound. The scan completion can advance by at most the largest improvement to an eligible earlier take, at most M(max(0,i+1-K)). Combining the unchanged worker predecessor with its discounted scanner predecessor gives the recurrence. Ordered reader max and a fixed copy duration preserve the bound. Worker-specific speed changes, changed cache behavior, altered scanner service, arbitrary scheduling changes and a changed reader workload fall outside this theorem.

An independent subset enumeration agrees with the spacing DP on 4,096 short schedules. Earliest-available dispatch changed worker assignments in 770 early replays. Every fixed and rerouted gain respected the bound. A one-slot witness attains a 14-unit gain with largest single discount 7, demonstrating why a bounded pipeline needs the slot term. [Executable checks](check.py) · [Receipt](check.txt)

## Capacity falsifier and FIFO replay

| Workers | Saving needed ms | Largest conditional capacity bound ms | Bound / required | Fixed worker gain ms | FIFO with reassignment gain ms |
|---:|---:|---:|---:|---:|---:|
| 7 | 20.93 | 16.20 | .774 | 1.34 | 1.26 |
| 8 | 25.20 | 15.36 | .610 | 2.68 | 2.94 |
| 9 | 29.08 | 15.29 | .526 | 2.93 | 3.07 |
| 10 | 23.95 | 16.88 | .705 | 2.08 | 2.08 |

The target-time normalization is T*=median(edge wall) × .80 / paired geometric edge-to-Lazy wall ratio. Absolute milliseconds provide a scale for the paired point objective. The archived budget has 10 complete isolated thread-CPU rotations of scan, ordered copy and worker scopes over the 431 blocks. The worker input shape is the matched edge predecessor narrow-short/ahead, across both run and column routes. Five additional large dense blocks use a different edge inverse. The measurements cover 431×4 service positions and 1,724 actual blocks across 48 streams in the target input. They are local shapes transported to the i9. [Block-cost provenance 00efe9b](../README.md) · [Measured i9 roles 824d095](../target.md) · [Measured edge archive 683c22e](../../workstation/evidence/edge-i9-683c22e.tar.gz)

For the capacity bound, a successful run necessarily has sum(scan)≤T* and sum(worker)≤W×T* under the same fixed service model. Scale each of the ten observed scanner sequences up to T*, and each whole-worker sequence up to W×T* while preserving its block-specific fraction spent in front. Apply the spacing bound for every pairing, using four copies in corpus order. This grants any scheduling policy satisfying the recurrence the largest services consistent with those observed shapes and target occupancy. No role fractions, calibration from the older opend arm or comparison of scanner CPU *ratios* are needed. The table reports the maximal bound from all 100 crossings at each W. Source-order block costs within any one archived rotation differ from the median-smoothed profile. A scanner-only bound exceeds the required gain in some rotations at every W. Retaining the separate MTF cap matters. [All 400 crossings](capacity.csv) · [Counts and ranges](capacity-summary.txt)

For the explicit FIFO example, transport old matched i9 opend scanner and reader CPU totals by the ratio of the two sweeps' common Lazy wall medians. Estimate worker demand as the new edge process CPU times the old opend worker share of process CPU. Apply those totals to the local per-block medians, keeping the predecessor's block front/total fraction. Scan start waits on the previous scan and ordered take i−4W. A worker finishes its job, the reader takes it in order and copies its known bytes, overlapping the next services. The simulated atomic wall is 92.4%, 92.2%, 90.2% and 91.8% of observed edge wall. It omits real shared-core interference, feeder stalls and lock delays. Its transported scanner already consumes approximately 90%, 97%, 96% and 98% of T*. Long-run scanner headroom, frontend service and worker idleness are distinct quantities. Varying scanner or whole-worker demand separately by ±20% is included in the raw table. With the transported profile and the slot constraint removed, even perfect early release saved 0.18–0.41 ms. [Input demands](rates.csv) · [All replay results](scenarios.csv)

## Trace on the actual edge worker

A private build uses stock trace level 1 and adds one thread-local frontend-finished mark per decode. It places the existing Ready timestamp under the publication lock, so the reader's Taken observation cannot predate it. The scorer's cursor-backed reader, four-copies input, 4 MiB output buffer, fixed 4W slots, CRC checks and block-admission kernel are retained. The experiment used two workers limited to CPUs 0 and 1 at nice +12 on the shared cloud machine. The 1,724 successful blocks summed to 838,800,784 bytes. No timing comparison or RSS inference is made between this run and the i9. [Private instrumentation script](trace_build.py) · [Trace harness](trace.rs)

The event replay reproduces every baseline issue, ready and take timestamp exactly, including observed additional slot, receive and ordered-reader delays. It keeps those residual delays and actual worker assignment in its counterfactual and grants the same ideal delta on each release. Only 96 original releases found the assigned worker available before EOB. Their separately summed exposed intervals were 13.81 ms. The final ordered take actually advanced 4.91 ms in the replay, 0.21% of the 2.367 s trace interval. Gains at each successive copy end were 1.32, 1.93, 3.24 and 4.91 ms. Repeated slot feedback permits some accumulation. Local CPU quota, placement and two-worker occupancy restrict the numerical interpretation. [Raw thread events](edge-w2.csv.xz) · [Replayed milestones](trace-summary.json)

The trace logged 99,917,196 u16 symbols, or 199,834,392 payload bytes across four copies. At 4,096 symbols per independently published chunk the same input requires 25,640 publications, including 23,916 beyond the existing one per block; at 16,384 symbols those counts are 7,328 and 5,604. The larger chunks leave only 932 blocks with any advance publication. Stock takes_runs chooses the route using the final symbol count and windows at positions derived from that count. A practical incremental frontend needs a shared representation or a paid way to resolve that decision while preserving MTF/RLE2 state, partial counts, length/mark validation, EOB and CRC/error ordering. The bounds charge none of that work. The existing scanner already writes each symbol and the worker reads it. Synchronous per-chunk handoff, duplicated payload traffic, an added core or a fragment allocation are implementation choices with different prices.

The decisive missing observation is an untimed same-build trace on the target with scan start/end, worker/ordinal assignment, post-MTF time, ordered take, and scanner wait boundaries. A trace-free matched timing remains necessary for any implementation. On the available evidence, changing whole-block admission alone does not warrant an independent target run. A hypothesis that removes front or inverse work, changes sustained memory contention, or reduces scanner service requires separate pricing.

## Reproduction

From this directory, Python 3 can recompute all tables from the tracked measurements. The Rust build extracts the frozen source into its argument directory. It uses Rust 1.99.1 and the indicated feature instrumentation.

```sh
python3 -B check.py | diff - check.txt
python3 -B schedule.py
python3 -B capacity.py | diff - capacity-summary.txt
python3 -B trace_replay.py edge-w2.csv.xz 2 | diff - trace-summary.json
python3 trace_build.py /tmp/astra-overlap
nice -n 12 taskset -c 0,1 /tmp/astra-overlap/trace ../../../../corpora/wikidata.bz2 2 > /tmp/astra-overlap/trace.csv
python3 -B trace_replay.py /tmp/astra-overlap/trace.csv 2
```
