# Why the stage gain doesn't reach the i9 decoder: a bounded read-only analysis

**What this is:**
- It reads only this archive, the two exploratory CI runs and master's journal.
- Nothing was run.
- The registered verdict, NEGATIVE, stands, and nothing here re-decides it.
- **Labels:** *measured* means read from the archive; *deduced* means it follows from
  measured figures; *model* means it rests on stated assumptions.

## 1. Most of the gap is per-core, present at one worker

At w = 1 there is one worker thread, so contention between workers can't account for
anything there.

| w = 1, wikidata-20261026 | CPU-s/GB B/A | Throughput B/A |
|---|--:|--:|
| EPYC 9V45 (Zen 5), CI | 0.868 | 1.179 |
| EPYC 7763 (Zen 3), CI | 0.932 | 1.081 |
| EPYC 9V74 (Zen 4), CI | 0.983 | 1.017 |
| Xeon 6975P-C, CI | 0.999 | 1.002 |
| **i9-10900K** | **0.985 [0.976, 0.994]** | **1.017 [1.006, 1.028]** |

*(Measured.)*

**On the i9, at one worker (deduced):**
- The arm saves 1.5% of the decoder's CPU, with an interval of 0.6% to 2.4%.
- A's CPU is 2.994 s/GB. At about 4.9 GHz that is about 14.7 cycles per output byte, so
  the saving is about 0.2 c/B.
- The stage trial saved about 0.55 c/B of walk on Zen 3, on the 4.49 c/B basis.

**Model:** if the walk is 35–45% of the decoder's CPU here, as it is on Zen 3 (4.49 of
10.78 c/B), the walk itself sped up by about 3–4% on the i9, against 12.3% on Zen 3.
- This share has never been measured on the i9. No i9 stage measurement of either walk
  exists.

**So:**
- The finding is not mainly that a per-core gain gets lost in the four-core pipeline.
  The unrolled round is only slightly faster per core on this Intel core to begin with.
- The ordering of hosts matches the static floor model in
  `../2026-11-01-walk-unroll-astra-fidelity/README.md`:
  - on Zen, the dispatch saving is large;
  - on Skylake-family cores, the two load-address ports bound the unrolled step at about
    2.5 c/step.
- This is consistent, not demonstrated. No counters were collected.

## 2. The pipeline then removes the remainder

**On the primary, as w rises (measured):**

| | w1 | w2 | w3 | w4 |
|---|--:|--:|--:|--:|
| A's throughput, MB/s | 369 | 684 | 775 | 808 |
| A's CPU-s/GB | 2.99 | 3.21 | 3.55 | 3.90 |
| Cores busy (MB/s × CPU-s/GB) | 1.11 | 2.20 | 2.75 | 3.15 |
| CPU-s/GB B/A | 0.985 | 0.989 | 0.994 | 0.994 |
| Throughput B/A | 1.017 | 1.012 | 1.005 | 1.000 |

*(Measured; cores busy is derived from the measured columns.)*

**Reading (deduced):**
- **Per-byte cost rises 30% from w = 1 to w = 4,** in both arms.
- **At w = 4 the decoder isn't CPU-saturated:** only 3.15 of the 4 allocated cores are
  busy, and the fourth worker adds 4% throughput.
- So at w = 4 throughput is set by something other than the workers' CPU time:
  - the serial scanner or feeder;
  - waits between them and the workers;
  - or shared L3 and memory: four 3.6 MB T-vectors plus buffers, against a 20 MB L3.
- A saving in worker CPU then shows up only partly in wall time.
- **The arm's own CPU saving also halves** (1.5% → 0.6%). That fits a walk that becomes
  more memory-bound under four-way sharing, where bookkeeping matters less.
- The archive doesn't record per-thread CPU, so it can't separate these causes.

## 3. `records`: a per-core cost, at every w

- **Measured:** CPU-s/GB B/A is 1.013–1.018 at every w, w = 1 included, with all four
  intervals above 1.00. Throughput is −1% to −2%.
- **Measured, the contrast:** `random`, where every T access misses, is neutral at every w.
- **Deduced:** the arm costs extra per core on this input, independent of contention.
- **What isn't known:** what in `records`' access pattern causes it. Its blocks are full
  size, so boundaries are rare, and the boundary path is unlikely (fidelity note). It is
  smaller on the i9 than on three CI hosts (−5% to −8%).

## 4. What the archive cannot answer

- **Instruction-cache and front-end effects** (the 29.7 KB function, and the W4 (c)
  difference in how the per-block driver is inlined) are inside the net 1.5% at w = 1.
  They can't be separated from it.
- **The walk's share of the decoder on the i9** is unmeasured.

## 5. On redirecting to less traversal work per output byte

**An i9 result already exists** for the obvious mechanism: the pair walk (LF², two bytes
per dependent load), E1 in `2026-10-29-expert-review.md` (measured).

| Threads | 1 | 4 | 6 | 8 | 10 |
|---|--:|--:|--:|--:|--:|
| Pair walk / v0.1.4 | 0.691 | 0.816 | 0.991 | 1.107 | 1.117 |

- **Reading:** it trades extra per-core work for less memory pressure. It wins only where
  memory pressure dominates, at 8–10 threads.
- In production's configuration it is much slower: wikidata-pq runs `Workers(≤ 4)`.
- **The bar for any new hypothesis of this kind:** beat that result at w ≤ 4. It should
  also say which resource it relieves. From §2, at w = 4 that resource is not the
  workers' CPU.

## 6. What would separate the causes

None of these is authorised; each would need authorisation.

1. **The existing stage harness on the i9:** `walk_arm` (the stage trial's paired one-copy
   walk), pinned to one core.
   - It gives the per-core walk B/A on Comet Lake directly. That settles §1's model
     figure.
2. **`perf stat` on arm A only, at w = 1 and w = 4:**
   - top-down level 1–2;
   - `l1d_pend_miss.pending` and `.fb_full` (misses in flight, and how often the fill
     buffers are full);
   - `cycle_activity.stalls_l3_miss`;
   - per-thread CPU for the scanner, the feeder and the workers.
   - It separates "the workers wait on memory" from "the serial stages bound w = 4".
