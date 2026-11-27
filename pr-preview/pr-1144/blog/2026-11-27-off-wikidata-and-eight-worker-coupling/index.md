[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1144/blog/index.md>)

# Off Wikidata and inside the eight-worker tie: route refusals and coupled stalls

Off Wikidata, 84% of full blocks bypass our Open kernel and pay dense refusals while trace replay decomposes the eight-worker tie.

27 November 2026 · 8 min read · [Research result](<#4-what-changes-next>)

**Two measurements from primary traces sharpen where mibz2 should spend its next engineering effort.** First, moving off our Wikidata benchmark exposes a blind spot. Across seven other corpora, 84% of full blocks bypass the Open inverse-BWT kernel completely, and every block in Open's dense admission band is refused *after* paying for a failed first pass. Second, replaying per-block timestamps at eight workers on Wikidata explains the 8 to 9 millisecond gap between shipped wall time and the bottleneck lower bound. Most of that gap is either already targeted by our seam-window patch or part of an irreducible queueing floor, leaving no hidden 3% lever in pipeline coordination.

Both findings follow directly from [The decoder's true floor and the limits of a pooled scan](<https://lmmx.github.io/mibz2/pr-preview/pr-1144/blog/2026-11-26-true-floor-and-pooled-scan/index.md>). That post established that on our 10-core Intel i9-10900K machine, decoding four copies of Wikidata (838.8 MB uncompressed) at eight workers puts the serial Huffman scanner (`S`) and the eight workers (`W/8`) in a near-exact tie near 185 ms, while shipped wall time sits at 192.9 to 194.6 ms. Two natural questions remained open: how much of our Wikidata-tuned worker surface actually runs on ordinary files, and whether the 8 to 9 ms coupling gap at the eight-worker tie hides an unowned speedup.

This post answers both from primary evidence. [Section 1](<#1-what-happens-when-the-decoder-leaves-wikidata>) reports the off-Wikidata route census and the exact cost of dense-band refusals. [Section 2](<#2-why-the-eight-worker-tie-leaves-8-to-9-ms-on-the-table>) corrects a common accounting mistake about our 5.2 ms plumbing floor and validates a discrete-event replay of the parallel decoder. [Section 3](<#3-chunk-maintenance-stalls-and-runt-diluted-slots>) isolates the two mechanisms driving mid-stream stalls and prices the remaining floor at a future 150 ms tie. [Section 4](<#4-what-changes-next>) sets out the concrete changes these measurements justify.

**Takeaways**

- **Off Wikidata, Open runs on only 16% of full blocks.** Across seven local corpora (107 full blocks), 90 blocks run the Segmented fallback instead of Open, and all 25 blocks in Open's dense band (\\(r/n \\in \[0.15, 0.28\]\\)) are refused after running Open's first pass to its cap.
- **Dense-band refusals cost 13 to 48% extra inverse time on refused blocks.** On ordinary prose, that wasted pass makes production 1.09 to 1.32 times slower than Segmented alone. A single pass over the run records predicts every refusal before Open begins.
- **Do not subtract 5.2 ms of empty-pipeline plumbing from the eight-worker tie gap.** When real work runs, per-block handoffs overlap behind active stages. Unoverlapped handoff overhead is only 1.12 to 1.16 ms.
- **Chunk maintenance amplifies the eight-worker tie by about 2.15 ms.** Eleven discrete 0.53 to 1.54 ms copy-and-drain stalls in `ChannelSource::view` skew the scanner across the four corpus copies. Removing them via a reused seam buffer recovers that coupling loss on top of its 10 to 11 ms direct scan reduction.
- **No standalone 3% lever hides in the remaining coupling gap.** Post-seam at a balanced 150 ms tie, about 4.5 ms is an irreducible queueing, startup and handoff floor. The remaining unowned levers (runt-exempt slots and a smaller initial read chunk) cap near 2.5 ms combined, or 1.6% of wall time.

## 1\. What happens when the decoder leaves Wikidata

Almost every worker-side speedup we have measured so far targets the [Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-1144/blog/2026-11-19-open-returns/index.md>) inverse-BWT kernel (`OpenCells`). On our Wikidata benchmark (`corpora/wikidata.bz2`, 431 blocks), all 233 full blocks enter Open because their run-to-row ratio \\(r/n\\) sits near 0.068. To test how often Open actually runs on other data, we ran a [route census](<https://github.com/lmmx/mibz2/blob/6f8a26f507567fd8a8eac9076da42f6f5821c973/docs/journal/2026-11-27-explore/route-census.md>) across Wikidata and seven container-built `bzip2 -9` corpora spanning prose, source code, JSON, binaries, DNA and random bytes.

Each full block (\\(n \\ge 100{,}000\\)) takes one of three routes:

1. **Column route (`Segmented`):** move-to-front keeps the flat BWT column because runs are too short to compress into run records.
2. **Open route (`OpenCells`):** run records pass admission and stay within Open's retained-row and thick-strip caps.
3. **Run-route fallback (`Segmented`):** run records enter the run path, fail Open's gates or its dense-band cap, and fall back to Segmented inversion.

| Corpus | Full blocks | Column (`Segmented`) | `Open` | Run-route fallback (`Segmented`) | \\(r/n\\) p10 / p50 / p90 |
| --- | ---: | ---: | ---: | ---: | --- |
| `wikidata` | 233 | 0 | 233 | 0 | 0.063 / 0.068 / 0.115 |
| `prose` | 8 | 0 | 1 | 7 | 0.162 / 0.170 / 0.175 |
| `pysrc` | 11 | 1 | 1 | 9 | 0.226 / 0.294 / 0.314 |
| `rustsrc` | 16 | 0 | 8 | 8 | 0.053 / 0.158 / 0.229 |
| `json` | 16 | 5 | 7 | 4 | 0.053 / 0.070 / 0.217 |
| `elf` | 33 | 27 | 0 | 6 | 0.232 / 0.261 / 0.413 |
| `dna` | 14 | 13 | 0 | 1 | 0.724 |
| `random` | 9 | 9 | 0 | 0 | — |

The split is stark. Off Wikidata, Open admits only **17 of 107 full blocks (16%)**. The column route takes 55 blocks and the run-route fallback takes 35, putting **84% of non-Wikidata full blocks** onto the Segmented kernel. Where Open does admit a block off Wikidata, it wins cleanly: 0.38 times Segmented on `json` (\\(r/n \< 0.15\\)) and 0.76 times on `rustsrc`, matching its 0.48 ratio on Wikidata.

The real surprise sits in the dense band (\\(r/n \\in \[0.15, 0.28\]\\)). In [Admitting dense blocks to Open](<https://lmmx.github.io/mibz2/pr-preview/pr-1144/blog/2026-11-21-a-memory-gain-is-a-gain/index.md>), we added a bounded cap (`Cap::dense`: at most \\(\\min(256{,}000, 0.3n)\\) retained rows and 16,384 thick strips) fitted to Wikidata's five dense blocks. All five Wikidata blocks pass that cap and finish in 0.78 times Segmented's time. Off Wikidata, however, **all 25 blocks in the dense band fail the cap** — after Open has already executed its first pass up to the limit.

| Corpus and dense-band outcome | Blocks | Production inverse (ms) | `Segmented` inverse (ms) | Ratio |
| --- | ---: | ---: | ---: | ---: |
| `prose` (refused) | 7 | 21.0 | 17.1 | 1.23 |
| `rustsrc` (refused) | 8 | 28.0 | 20.6 | 1.36 |
| `json` (refused) | 4 | 17.6 | 11.8 | 1.48 |
| `elf` (refused) | 4 | 16.4 | 14.5 | 1.13 |
| `pysrc` (refused) | 2 | 7.0 | 6.9 | 1.02 |
| `wikidata` (admitted) | 5 | 11.4 | 14.6 | 0.78 |

*(Single-thread timings on a 4-vCPU Xeon VM, production vs Segmented on the same blocks; shares and ratios are diagnostic, not workstation adoption evidence.)*

Those 25 refused blocks waste about 19 ms (\~0.8 ms per block), or 13 to 48% on top of the fallback's own inverse time. On the whole `prose` file, production finishes 1.09 to 1.32 times slower than running Segmented directly. Yet the refusal is completely predictable before Open allocates or scans anything. Retained rows are simply the rows belonging to maximal runs of at most 16 bytes, plus the marked row. Summing thin runs across the run records in one cheap pass separates the two populations cleanly: every refused block exceeds 256,000 retained rows (the smallest is 257,495 on `elf`, and `prose` spans 311,318 to 380,411), while every admitted Wikidata block lies between 199,189 and 231,600. Thick strips never bind (at most 8,081 against 16,384).

This reveals a sharp corpus asymmetry. Serial Huffman scanner improvements benefit every corpus equally because every route reads the same compressed bitstream. Open-kernel worker speedups cover 100% of Wikidata's full blocks but only 16% off Wikidata. Fixing the dense-band pre-check and dissecting the unowned Segmented fallback are therefore essential for ordinary text and source code.

## 2\. Why the eight-worker tie leaves 8 to 9 ms on the table

Turning back to eight workers on Wikidata (`wikidata ×4`, 1,724 blocks, 838.8 MB output on our i9-10900K), the operational lower bound is

\\\[ L = \\max\\!\\left(S,\\;\\frac{W}{k}\\right), \\\]

where \\(S\\) is total serial scanner busy time and \\(W/k\\) is mean worker busy time across \\(k = 8\\) workers. Across three primary i9-10900K evidence bundles — [character stage 1](<https://github.com/lmmx/mibz2/blob/6f8a26f507567fd8a8eac9076da42f6f5821c973/docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/report.txt>) (48 grid runs plus 4 complete nanosecond per-block traces), the [zero-copy floor](<https://github.com/lmmx/mibz2/blob/6f8a26f507567fd8a8eac9076da42f6f5821c973/docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md>) (`cc8ba137`), and the [pooled-scan run](<https://github.com/lmmx/mibz2/blob/6f8a26f507567fd8a8eac9076da42f6f5821c973/docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md>) (`c71c4a56`) — shipped wall time exceeds \\(L\\) by **7.6 to 9.1 ms** (4.0 to 4.7% of wall):

| Primary i9-10900K bundle (`k = 8`, medians, ms) | Untimed wall | Traced wall | `scan_busy` | `wkr_busy/8` | `wall − max` | `slot_wait` (waits) | `in_wait` | `job_wait/8` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `character-stage1` grid `none` (\\(n = 48\\)) | — | 192.27 | 184.45 | 182.67 | **7.57** | 3.25 (13) | 1.39 | 8.41 |
| `character-stage1` traced `r0`–`r3` mean | — | 195.61 | 184.99 | 185.64 | **9.13** | 5.20 (19) | 1.43 | 9.57 |
| `zero-copy-floor` `none` (pinned CPU 6) | 194.57 | 194.36 | 184.35 | 186.35 | **8.01** | 6.10 (24) | 1.26 | 6.94 |
| `pooled-scan` `none` (unpinned) | 192.93 | 193.72 | 184.79 | 184.86 | **8.86** | 4.11 (14) | 1.34 | 8.50 |
| `pooled-scan` `S` (scan ideal, workers bind) | 184.88 | 185.35 | 1.61 | 184.19 | **1.16** | 180.14 (1,342) | 0.00 | 0.22 |
| `pooled-scan` `SC` (scan + construction ideal) | 135.31 | 133.30 | 1.46 | 132.18 | **1.12** | 129.15 (1,256) | 0.00 | 0.23 |

That table immediately corrects a tempting accounting error. It is easy to subtract our all-ideal `SFCMRD` plumbing floor (5.2 ms) from the 9.1 ms gap and assume only \~3.9 ms is queueing loss. **That subtraction is wrong.** In `SFCMRD`, all 5.2 ms sit exposed because no real decoding work runs alongside them. As soon as real worker stages run (`S` and `SC` above, or `SFMRD` at 0.88 ms and `SECMRD` at 0.69 ms), per-block handoffs (`issue` 0.65 ms, `pool+publish` 0.07 ms per worker, and mutex acquisitions of 0.3 ms scanner / 0.06 ms worker) overlap behind real compute. The true unoverlapped handoff residual above \\(L\\) is only **1.12 to 1.16 ms**.

To break down the rest of the 9.13 ms gap without guessing, our [coupling dissection](<https://github.com/lmmx/mibz2/blob/6f8a26f507567fd8a8eac9076da42f6f5821c973/docs/journal/2026-11-27-explore/eight-worker-coupling.md>) built a discrete-event simulator implementing the exact admission guard (`Shared::full`), FIFO job queue (`Jobs`), \\(k\\)-worker pool, and in-order `Ready` ring from `src/parallel.rs`. Fed the recorded per-block service times from all four traced rounds (`blocks-r00` to `blocks-r03`), the replay tracks real wall time to within **0.94 to 1.36 ms** (mean **\+1.15 ms**, matching `S` and `SC`'s handoff residual):

| Round (`k = 8`) | \\(S\\) (ms) | \\(W/8\\) (ms) | \\(L = \\max\\) | Real wall (`slot_wait`) | Replay wall (`slot_wait`) | Real − Replay |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `r0` | 183.99 | 188.31 | 188.31 | 198.89 (9.61) | 197.53 (8.72) | \+1.36 ms |
| `r1` | 183.24 | 182.58 | 183.24 | 192.02 (3.27) | 190.93 (2.56) | \+1.09 ms |
| `r2` | 184.13 | 181.43 | 184.13 | 192.33 (2.18) | 191.11 (1.46) | \+1.21 ms |
| `r3` | 188.59 | 190.24 | 190.24 | 199.21 (5.74) | 198.27 (5.10) | \+0.94 ms |
| **Mean** | **184.99** | **185.64** | **186.48** | **195.61 (5.20)** | **194.46 (4.46)** | **\+1.15 ms** |

Because the replay reproduces real wall time and `slot_wait` closely across all four rounds, toggling one pipeline rule at a time isolates each physical source of delay.

## 3\. Chunk maintenance stalls and runt-diluted slots

Four mechanisms explain the entire gap above \\(L\\).

### 1\. Stop-the-world chunk maintenance creates a mid-decode phase inversion (\~2.15 ms coupling bonus)

Whenever a block's lookahead crosses a 4 MiB input chunk boundary, `ChannelSource::view` (`src/parallel.rs`) drains the buffer and copies the incoming chunk on the serial scanner thread (`in_maint`: 10.1 to 11.2 ms at eight workers). In the per-block traces, that 10.1 ms is not smooth overhead. It arrives as **11 discrete stop-the-world stalls of 0.16 to 1.54 ms**, heavily front-loaded across the four identical 431-block copies as the window buffer grows and faults pages:

\\\[ \\text{in\\\_maint}(c\_0, c\_1, c\_2, c\_3) = (3.81,\\; 3.09,\\; 1.86,\\; 1.40)\\;\\text{ms}. \\\]

That slope pulls scanner time from 48.74 ms on copy 0 down to 44.61 ms on copy 3, while worker time changes far less (47.95 to 45.22 ms). On copies 0 and 1, the scanner falls behind and starves the workers. On copies 2 and 3, the scanner surges ahead and hits the admission window. In the exact-tie replay (\\(S = W/8\\), zero startup), removing `in_maint` drops the mid-stream coupling gap from **6.61 ms to 4.46 ms** — a **2.15 ms coupling bonus** that our seam-window patch captures automatically on top of its 10 to 11 ms direct scan cut.

### 2\. Runt blocks dilute the 32-slot window (1.21 ms at a 150 ms post-seam tie)

Wikidata is not clustered into bursts of full blocks and bursts of tiny runts. Its 233 full blocks (\~900 KB) and 198 runts (median 8 to 30 bytes) alternate almost everywhere (`F.F.F.F…`; max runt streak is 2). Yet `Shared::full` counts every block equally against `SLOTS_PER_THREAD × k = 32` slots. As a result, **15 to 16 of the 32 slots always hold tiny runts**, leaving only 15 to 17 full blocks of slack.

Meanwhile, `READY_BYTES` (12 MiB) **never fires at eight workers**: `slot_cap` is 0.00 ms across all 48 grid runs, and median peak ready bytes is only 6.01 MiB. Every single `slot_wait` at eight workers is a 32-slot head-of-line stall caused by one slow executing block (such as block offsets 56, 171, 278, and 389, taking 2.1 to 3.4 ms). While that single worker finishes, the scanner runs 32 places ahead (offsets 88, 203, 310, and 421) and blocks.

Importantly, `slot_wait` and worker `job_wait` are **sequential, not simultaneous**. Intersecting the exact nanosecond intervals in `blocks-r00` through `blocks-r03` shows mean worker idle time during `slot_wait` is only **0.033 to 0.042 ms** in rounds 1 to 3 (matching `pooled-scan`'s 0.04 ms). Every worker is busy while the scanner waits, and the lost scanner time reappears downstream as worker starvation once the slow block clears.

Expanding the window to 64 slots — or exempting tiny runts from the 32-slot count — captures 100% of the infinite-window wall saving. Today (with `in_maint` still present) that saves 2.10 ms (1.08%); at a future 150 ms exact tie *after* the seam window lands, it saves only **1.21 ms (0.77%)**. Trying to inline runt decoding onto the scanner thread makes wall time *worse* (+1.12 ms) because it adds worker CPU directly onto the tied serial scanner.

### 3\. Startup and the 4 MiB initial read chunk (2.62 ms total; 1.35 ms recoverable)

Startup accounts for 2.41 ms median across 48 runs (2.62 ms mean in `r0`–`r3`), split into two pieces:

- **1.22 ms** spawning `k + 2` OS threads before the scanner starts block 0 (a one-shot per-stream cost that amortises to zero on larger inputs);
- **1.40 ms** of block-0 `in_wait` (1.26 to 1.40 ms across all three bundles), because the feeder thread (`src/read.rs`) allocates a fresh 4 MiB buffer and reads all 4 MiB before pushing chunk 0, while the scanner and all eight workers sit idle waiting for a \~25 KiB first block.

Starting with a smaller first chunk (for example 256 KiB before ramping to 4 MiB) recovers about **1.35 ms** (0.68% today; 0.86% at a 150 ms tie), once per stream.

### 4\. The irreducible queueing floor and cross-talk

Even with infinite slots, zero startup, and zero `in_maint`, a balanced 150 ms tie still pays **2.19 ms (1.44%)** above \\(L\\): about 0.7 ms for one-block fill and drain, 0.6 ms from full-vs-runt size bimodality, and 0.9 ms from within-class block-time variance and eight-worker end-of-stream finish spread. Adding thread spawn (1.22 ms) and unoverlapped handoff overhead (\~1.1 ms) gives an **irreducible floor near 4.5 ms**.

Cache cross-talk between the scanner and workers is also smaller than it looks at first glance. Comparing pinned `none` against scan-alone `FCMRD` at eight workers (`zero-copy-floor`) shows scanner active time rising by +9.67 ms: +4.16 ms is `in_maint` memory contention (already removed by the seam window), +1.50 ms is the 4.95 → 4.90 GHz all-core turbo step, and +4.46 ms in Huffman decode (`A3`) is a one-worker cross-core `free_syms` coherence handoff that is already present at \\(k = 1\\) and stays completely flat (746 to 752 Gcycles) from 6 to 10 workers. Worker-side cross-talk from the scanner is only +0.67 ms (`S` vs `none`).

| Component of the 8-worker coupling gap | Today (`r0`–`r3`, \~195.6 ms) | Future 150 ms tie (post-seam, \~157.5 ms) | Status / ceiling |
| --- | ---: | ---: | --- |
| `in_maint` coupling amplification | \~2.15 ms | 0.00 ms | Covered by seam-window patch |
| 32-slot HOL window (runt dilution) | \~2.10 ms | **1.21 ms (0.77%)** | Unowned; runt-exempt 32 slots |
| Block-0 `in_wait` (4 MiB first chunk) | 1.40 ms | **1.35 ms (0.86%)** | Unowned; one-shot startup |
| OS thread spawn (`t_first_scan_start`) | 1.22 ms | 1.22 ms (0.77%) | Irreducible per decode call |
| Indivisible-block variance &amp; drain tail | \~1.11 ms | **2.19 ms (1.39%)** | Irreducible queueing floor |
| Handoff / wakeup residual (`Real − Replay`) | 1.15 ms | \~1.1 ms (0.7%) | Closed plumbing residual |
| **Total coupling gap** | **9.13 ms (4.67%)** | **\~7.5 ms (4.8%)** | **Combined unowned ceiling: \~2.5 ms (1.6%)** |

## 4\. What changes next

Together, these two measurements close a dead-end temptation and open a neglected axis of real performance work:

1. **Do not chase a standalone coupling project at eight workers.** Once the input seam window removes `in_maint` and its 2.15 ms coupling bonus, the remaining unowned coupling mechanisms cap near 2.5 ms (1.6% at a 150 ms tie). Reaching our 6 GB/s (\~140 ms) research milestone still depends on algorithmic cuts to both the scanner and the workers.
2. **Keep `READY_BYTES` distinct from `SLOTS_PER_THREAD` when tuning L3 cache pressure.** Because `READY_BYTES` never fires at eight workers while 32 slots already suffer runt-diluted head-of-line stalls, shrinking `SLOTS_PER_THREAD` below 4 would regress eight-worker throughput unless runts are excluded from the slot count. Bounding `READY_BYTES` alone protects 9-to-10-worker L3 residency without hurting eight workers.
3. **Fix dense-band refusals and map the Segmented fallback.** A one-pass thin-run pre-check removes the 13 to 48% dense-band regression on prose and source code with zero Wikidata impact, and a dedicated phase dissection of `Segmented` now targets the 84% of non-Wikidata blocks that our Wikidata-only surface never showed.
4. **Land the seam window first on the workstation queue.** It removes 10 to 11 ms of serial `in_maint` copy work plus 2.15 ms of coupling distortion, giving both the worker-side fold test and the combined scanner improvements a clean, stationary baseline.
