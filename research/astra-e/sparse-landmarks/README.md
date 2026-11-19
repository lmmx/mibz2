# Two walks with sparse final-output offsets

The tested two-walk writer lost to the original residual emitter on every one of ten measured Wikidata blocks at each of four lane/stride settings. The median of paired stage ratios ranged from 1.62 to 1.89. Giving the writer the final offsets for free brought these ratios to 0.92–1.00, with substantial block-to-block variation. Exact offsets can expose independent successor chains, but obtaining them by a second table walk costs more than the tested interleaving recovers. The measured edge constructor retains its original emitter. [Stage comparison](results/summary.csv) · [Blockwise ratios](results/block-ratios.csv)

## Segments and costs

For a power-of-two stride S, choose all retained ranks congruent to the residual primary modulo S. The residue includes the primary and makes both membership and landmark ID constant-time arithmetic. In the first pass each of four or eight lanes starts from a landmark and walks until it next enters the residue. Its descriptor records the successor landmark and exact length in expanded BWT-inverse bytes. A literal adds one byte, and a dictionary edge adds the difference of the existing adjacent spelling offsets. As soon as a lane completes it can take another landmark.

Following these descriptors from the primary orders its segments. Their prefix lengths give disjoint output ranges. A second batch of independent lanes traverses only these marked segments. Each lane writes its actual literal or dictionary slice into its assigned range. Reaching the end of that range finishes the segment. The output vector is reserved once, filled in-place and exposed after all ranges have been initialized. Only at this point does the emitter apply the original bounded period-doubling copy. The implementation retains two u32 fields per landmark for its outgoing link, two u32 fields per primary segment for its write job, and O(lanes) local state. The old edge table and dictionary stay live through both passes. [Source](sparse.rs) · [Frozen constructor extraction](prepare.py)

For any permutation, every cycle containing a landmark partitions into the edge paths from each landmark to its next landmark. A one-landmark cycle supplies one closed path. A cycle without a landmark supplies no first-pass work and cannot contain the primary. The coarse primary cycle orders exactly the primary's original path labels. Since the constructor's spellings are nonempty, the output byte cursor reaches its assigned segment end exactly when the path is complete. Cases with multiple disjoint cycles need neither a full-row scan nor per-edge tags. A short marked cycle repeats exactly as under the original emitter, including a clipped final repetition. The free-offset oracle traverses the marked cycle in ordinary serial order outside the timed region and independently identifies each source landmark and output start. The paid and oracle writer use the same direct store loop.

This is the Helman–JáJá sublist decomposition applied to a word-labelled residual permutation. The GPU review describes per-element rank and sublist-ID arrays. Open-Return's earlier ranked writer used precisely the primary residue modulo 128 and eight lanes but stored one word code per visited row, then expanded serially. The dictionary-anchor writer marked word edges and staged the literal tails. The present result prices two paid table walks using only sparse descriptors and direct final-slot copies. [Ranked implementation at 2b53453](https://github.com/lmmx/mibz2/blob/2b53453e124ffdc75cc4d2141b833a319e3cfd73/docs/journal/2026-11-18-open-return/segments.rs) · [Dictionary anchors at 9a93d9d](https://github.com/lmmx/mibz2/blob/9a93d9d5e1daf8895423f4a9ebdc220215217ca7/research/astra-e/anchors/anchor.rs) · [Prior list-ranking review](../../../docs/journal/2026-10-27-research-synthesis.md)

## Exact checks and paid geometry

The independent permutation test made 125,388 marked/label/layout/stride comparisons. It exhausts permutations through five retained ranks, covering every primary, packed and separate code arrays, mixed literal/dictionary labels and several sparse residues. Byte spellings of unequal lengths and periodic output with a partial final repetition are included. On the real stream all three tested strides produced the original marked job order and offsets, and every decoded inverse byte agreed with LazyCells on all 233 actual edge admissions. The complete 8-lane, stride-128 stream check matched Python libbzip2 on all 209,700,196 output bytes and validated all 431 block CRCs and the scanner's combined stream CRC checks. Other blocks followed the stock path. [Property result](results/property.txt) · [Complete stream differential](results/stream-check.txt) · [All 233 block geometries](results/all-geometry.csv)

On the real admissions the marked cycle visits all 26,567,581 retained ranks, including 4,894,668 dictionary edges. Every one of the 26,567,581 hops is also paid by discovery. Under stride 128, the coarse pass traverses 207,562 landmarks across the corpus. The largest logically populated descriptor storage was 28,960 bytes for one block. Actual high-water capacity across the complete stream with plan reuse was 38,848 bytes for one sequential worker. The frozen block has 702 landmarks over 89,874 retained ranks and spells 899,931 bytes in one period. Its descriptors occupy 11,232 populated bytes. These measurements isolate new scratch; input, edge table, spelling, base output vector and allocator overhead remain additional. Whole-process peak RSS has not been measured for this losing writer. [Exact visits and dictionary counts](results/work.csv)

## Stage measurement

The native test constructed immutable edge tables with the pinned i9-leading inverse `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. The generated predecessor constructor's SHA-256 is `c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f`. The ten fixed stream ordinals are 1, 41, 85, 134, 176, 225, 269, 313, 359 and 397. The eight new arms and original emitter rotated forward and backward at each block, with 24 timed rounds after one discarded warmup. Every call's output was checked against the independent inverse outside the timed interval. The measured unit is emitter thread CPU time on logical CPU 3 of the four-CPU-quota Xeon. Input parsing, inverse construction and final RLE1/CRC are outside that interval, and the corpus differential accounts for those separately. [All 2,160 timed calls and 90 warmup calls](results/stage.csv) · [Reproduction driver](run.sh)

| Lanes | Stride | Median paired free-offset / stock | Median paired paid / stock | Blocks with paid median below stock |
|---:|---:|---:|---:|---:|
| 4 | 128 | 0.937 | 1.889 | 0 of 10 |
| 8 | 64 | 0.996 | 1.790 | 0 of 10 |
| 8 | 128 | 0.923 | 1.724 | 0 of 10 |
| 8 | 256 | 0.996 | 1.621 | 0 of 10 |

The aggregate of per-block median times gives lower ratios on some arms, including 0.765 for 4-lane free offsets and 1.386 for the 8-lane paid stride-64 arm. All raw samples are retained. Across the 2,160 timed samples, 113 reached 20 ms and 53 exceeded 100 ms, well above typical block times around 1–2 ms. These local data do not resolve close differences among strides or the oracle's best attainable benefit. On frozen, all paid paired medians are between 1.84 and 1.97. No whole-worker or i9 throughput gain is claimed.

An independent Valgrind 3.24.0 Callgrind replay counted the frozen emitter after its buffers were sized in a separate invocation outside collection. Under a simulated 32 KiB eight-way L1D and 2 MiB sixteen-way last cache, stride 128 and eight lanes gave the counts below. The last-cache read misses stay close despite large differences in instructions, stores and L1 read misses. The simulator does not measure native IPC, DRAM cycles or concurrent-worker pressure. [All counters](results/frozen-counters.csv) · [Original Callgrind dumps and logs](results/callgrind.tar.xz)

| Emitter | Instructions | Data reads | Data writes | L1D read misses | Last-cache read misses |
|---|---:|---:|---:|---:|---:|
| Stock | 2,713,554 | 716,539 | 229,197 | 71,367 | 6,452 |
| Free offsets | 3,966,036 | 708,672 | 351,740 | 85,440 | 6,543 |
| Paid construction and writer | 6,752,369 | 1,287,614 | 537,126 | 167,598 | 6,459 |

## Reproduce

Use Rust 1.99.1, the canonical scorer input (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`) and a local CPU permitted by affinity. The driver regenerates the frozen constructor, compares the entire stream to libbzip2, builds the census, runs the paired stage and writes summaries. For counters, install Valgrind 3.24 with Callgrind available and run on the rewrapped frozen block.

```sh
study=research/astra-e/sparse-landmarks
wk=/tmp/astra-sparse
wiki=$(realpath corpora/wikidata.bz2)
bash "$study/run.sh" "$wiki" "$wk" 3
python3 docs/journal/2026-11-18-open-return/frozen.py "$wiki" "$wk/frozen.bz2"
for arm in stock oracle8s128 paid8s128; do
  valgrind --tool=callgrind --instr-atstart=no --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 --LL=2097152,16,64 --branch-sim=yes --callgrind-out-file="$wk/$arm.out" "$wk/count" "$wk/frozen.bz2" "$arm"
done
```

The native implementation and timed results are frozen at `1877dbd7082b9d41b73011da7746dce336765e47`. A future improvement would need a cheaper independent source of exact primary offsets or a materially faster constructive first pass. Any such change faces the complete emitter and whole-worker comparisons again.
