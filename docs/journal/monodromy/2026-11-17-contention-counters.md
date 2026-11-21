# 2026-11-17 · MONODROMY 47 · Predictions frozen before the contention-counters run: each LLC miss avoided between 10 and 7 workers saves 440–860 cycles, at least a whole DRAM access, so miss count is at most about half the contention. The rest should be the cost of the 0.69 L3 accesses a byte rising with W, which W12 never touched. DRAM bandwidth should sit near 10–20% of peak

Asked by the Coordinator (10:02), for the counters run at 10 threads: W = 10, 9, 8, 7 on Wikidata,
reported and not judged. It reads:
- miss latency;
- memory bandwidth;
- stall cycles;
- and, if cheap, a transparent-huge-pages arm.

It queues after the low sweep and the -3 run. Frozen here, before its data:
- how miss cost and bandwidth move with W, under two readings;
- the share of the 10 → 7 LLC-miss cut on the walk's dependent chain;
- what refutes each reading.

**Labels:** **C** counted · **M** modelled · **Conj**.

## 1. What the at-the-cores bundle already says (C)

From `workers-at-cores/run-20261117T083701Z`, `counters.txt` (per byte of output, medians of 10 reps):

| W | cycles | LLC load misses | L2 misses (L3 accesses) | IPC | GB/s out |
|--:|--:|--:|--:|--:|--:|
| 10 | 22.98 | 0.01964 | 0.689 | 1.20 | 1.91 |
| 9 | 22.27 | 0.01805 | 0.691 | 1.23 | 1.96 |
| 8 | 19.75 | 0.01285 | 0.699 | 1.38 | 2.10 |
| 7 | 17.32 | 0.01002 | 0.712 | 1.56 | 2.14 |
| 6 | 15.52 | 0.00792 | 0.718 | 1.73 | 2.08 |

**Cycles saved per LLC miss avoided** (Δcycles ÷ Δmisses):

| step | 10 → 9 | 9 → 8 | 8 → 7 | 7 → 6 | 10 → 7 |
|---|--:|--:|--:|--:|--:|
| cycles | 442 | 485 | 861 | 852 | **589** |

**The comparison.** The clock is 4.91 GHz (1.326 × 3.7). A DRAM access is about 390 cycles at
80 ns, or 490 at 100 ns. A miss fully exposed, with no overlap of misses, costs at most that.

**The consequence.** The miss count can explain at most 390–490 ÷ 589, so **two thirds to four
fifths of the 10 → 7 cycle cut**, and only with no overlap at all. Under the overlap the walk is
built for, it explains far less.
- *The remainder:* each access must cost more as W rises.
- *What it can be:* the L3 accesses, 0.69 a byte, flat in W. They are 35 times the misses. If each
  costs Δ more cycles exposed, they add 0.69Δ cycles a byte. Δ ≈ 4–8 cycles a hit carries the rest.

**W12, re-read (Conj).** W12 cut LLC misses by 7.4% at 7 workers, which is 0.0007 a byte. At
600 cycles each, that is about 0.45 cycles a byte, or 2.6% of the cycles, before its +1.6%
instructions.
- *If most avoided misses were overlapped:* the saving falls to about 1% and is lost to the
  instructions. That is the measured null.
- *What W12 doesn't change:* the L3 accesses a byte, which no footprint cut that stays above
  the L2 changes.

## 2. The two readings and their predictions (M)

**The interconnect reading (Q).** Each worker's L3 accesses queue in the ring and the L3 slices, so
every access costs more as more workers run. Misses also rise with sharing, but they are the
smaller part.

**The DRAM reading (D).** Misses rise with sharing, and DRAM latency rises because the memory
controller nears its bandwidth.

| counter, per byte or per access, 10 → 7 workers | Q predicts | D predicts |
|---|---|---|
| L3-hit load latency (load-latency sampling, or `l3_hit` with `offcore_requests_outstanding`) | **falls 10–25%** (about 5–10 cycles of 40–55) | flat within 5% |
| L3-miss load latency | falls 5–15% (the same queues, then DRAM) | **falls 25% or more** |
| DRAM traffic (`uncore_imc/data_reads` + `data_writes`) at W = 10 | **4–9 GB/s: 10–20% of the dual-channel peak** | 60% of peak or more |
| DRAM traffic, 10 → 7 | 0.8–1.0 (fewer misses, more throughput) | falls with the misses |
| `stalls_l3_miss` share of the 10 → 7 cycle cut | **at most 45%** | at least 60% |
| `stalls_l2_miss` share of the cut (L3 hits included) | at least 70% | about the same as `stalls_l3_miss` |
| `mem_load_retired.l3_hit` per byte | flat within 5% | flat within 5% |
| `mem_load_retired.l3_miss` per byte, 7 against 10 | about 0.52 | about 0.52 |

**The traffic estimate behind the bandwidth line.**
- *Demand misses:* 0.0196 a byte × 1.91 GB/s × 64 B is 2.4 GB/s of reads.
- *The output:* 1.9 GB/s of writes, plus up to 1.9 GB/s of reads for ownership.
- *The input:* about 0.25 GB/s.
- *Hardware prefetch:* unknown, and the one way D could survive.

**My expectation: Q.**

## 3. The dependent chain's share of the miss cut (M, Conj)

**The estimate.** Demand LLC misses fall on what the prefetchers can't predict: the random-access
structures. By record 39's census these are the words (15.6 B a letter), the top slots (6.4) and
the bottom slots (2.5). That is 83% of a lazy worker's 29.5 B a letter. The streams and the bytes,
5 B, are read in order.

**The prediction.** Of the 10 → 7 cut in demand LLC misses, **70–85% sits on the walk's and the
expansion's loads** (by IP in load-latency samples: the rank and record loads, and the word reads).
**Under 20% sits in the streams and the copies.**

**The caveat for the levers.** A miss on these loads is not always on the dependent chain: the
lanes overlap them.
- *My figure:* about half the walk's misses are exposed.
- *What would show it:* `cycle_activity.stalls_l3_miss` over the walk's samples, against its
  misses × 400.

## 4. Transparent huge pages on 7/28 (M)

**What THP removes.** A lazy worker's random set is about 2 MB, about 500 small pages, beside its
output and input. Ten workers' 5,000 pages exceed the second-level TLB's 1,536 entries. A
page-walk load goes through the same L2 and L3, so it is also a contended access.

**The prediction:**
- *Wall:* 0.97 [0.93, 1.00] against 4 KiB pages.
- *Page walks:* `dtlb_load_misses.walk_active` falls by 80% or more.
- *If the page walks fall but the wall doesn't move:* the page walks were overlapped too.

## 5. What refutes each reading, and what follows

**Q is refuted if:**
1. L3-hit latency is flat within 5% from 10 to 7; and
2. `stalls_l2_miss` minus `stalls_l3_miss` carries under 30% of the cut.

**D is refuted if:**
1. DRAM traffic at W = 10 is under 40% of the peak (the machine's own DIMM speed, from
   `dmidecode` if the run can read it, or Louis's); and
2. L3-miss latency falls under 15% from 10 to 7.

**If both are refuted:** the cycles are something neither names: the store buffer (13% full in
bench-w12-stores), or the core's own resources. Then the model's b · W · W_run is a fit and
names no mechanism.

**What each outcome means for the levers (thread 1, staggered phases):**
- *Q:* the cost is concurrent L3 traffic. Staggering the workers' access-heavy phases, or cutting
  L3 accesses a byte (a hot set that fits the 256 KiB L2), are the levers. Cutting the footprint
  above the L2 is not, which matches W12.
- *D:* bandwidth is the cost. Cutting DRAM traffic (the output's ownership reads, by streaming
  stores) is the lever, and staggering helps only by smoothing peaks.
