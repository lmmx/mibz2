[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-740/blog/index.md>)

# Admitting dense blocks to the Open kernel: 29% less peak memory

A bounded admission band takes five dense blocks off a 5.6 MiB-per-worker fallback, and the decoder runs slightly faster in over a quarter less memory.

21 November 2026 · 10 min read · [Research result](<#5-what-transferred-from-the-research-line>)

Shipped in mibz2 [0.5.11](<https://lmmx.github.io/mibz2/pr-preview/pr-740/benchmarks/history/#release-0-5-11>) ([changelog](<https://github.com/lmmx/mibz2/blob/master/CHANGELOG.md#0511---2026-11-21>))

mibz2 is a bzip2 decoder in Rust. Most of its time goes on inverting the Burrows–Wheeler transform (BWT) of each block. Its fastest kernel for that, Open, used to refuse blocks with many short runs. On our Wikidata benchmark, five of the 233 full-size blocks were refused. They fell back to an older kernel that allocates about 5.6 MiB per worker, and that fallback set the decoder's peak memory. In mibz2 0.5.11, Open admits those blocks under a cap on its first pass ([\#703](<https://github.com/lmmx/mibz2/pull/703>)).

**The effect** was measured on the i9-10900K we test releases on, which has ten cores and a 20 MiB shared L3. Over ten paired rounds, 0.5.11 against 0.5.10, on the Wikidata benchmark:

| threads | peak memory (RSS) | wall time | median peak RSS |
| ---: | ---: | ---: | ---: |
| 8 | 0.738 \[0.710, 0.767\] | 0.976 \[0.965, 0.987\] | 87.0 → 63.4 MiB |
| 10 | 0.709 \[0.691, 0.728\] | 0.960 \[0.956, 0.965\] | 93.9 → 66.5 MiB |

The output is byte-identical on every block. The change, \#703, is the only decoder change between the two releases. The intervals are 95% intervals of the paired ratio, over this one machine and corpus.

**The takeaways:**

- **The cause was a routing gap, not the kernel's cost.** Open's admission test refused blocks whose density r/n, runs per byte, exceeded 0.15. Five full-size Wikidata blocks sit between 0.151 and 0.206, and they paid for a full-table fallback.
- **A bounded band closes the gap.** Blocks with r/n from 0.15 to 0.28 are admitted if Open's first pass stays under a cap on its retained rows and thick strips. The five fit under it.
- **Two short arguments make the port safe.** No elimination step ever widens a strip, so the widest initial strip bounds every later step. And three working buffers can live inside an array that is dead until a late phase.
- **The memory claim transferred; the speed predictions did not.** The research line that proposed the change predicted the dense blocks would take under half the cycles. Natively, on one thread, they take 12–43% longer than on the fallback. The speed that does appear on the i9 comes from the smaller working set, not from the kernel being faster on these blocks.

## 1\. Why did five blocks take the expensive path?

bzip2 cuts a file into blocks of up to 900 kB and applies the BWT to each. The transform reorders a block so that bytes with similar contexts sit together, and equal bytes form *runs*. mibz2 decodes several blocks at once, one per worker thread, and each worker needs its own scratch memory.

Open keeps only the rows of a block's narrow runs and moves each wide run as one *strip* ([Open returns](<https://lmmx.github.io/mibz2/pr-preview/pr-740/blog/2026-11-19-open-returns/index.md>)). Since 0.5.10 its scratch is about 1.2 MiB per worker, small enough for seven to ten workers to share the i9's 20 MiB L3. A block with many short runs leaves more rows for Open to keep. Open's admission test therefore used density, r/n, and refused blocks above 0.15. A refused block went to LazyCells and then Segmented, which builds a full table of the block at about 5.6 MiB per worker.

The benchmark is an 11.9 MB extract of a Wikidata dump: 431 blocks, of which 233 are full-size blocks on the route Open serves. Open took 228 of them. The other five have densities from 0.151 to 0.206. A worker that met one held the fallback's table, so the worst worker's memory was set by these five.

Where the 233 full-size blocks go, before and after the dense band

Measured routing on the benchmark's 431 blocks (\#703's census). The 228 blocks below r/n 0.15 are admitted by Open's count gates in both versions; their individual densities are not plotted. The five above the line are plotted at their measured r/n. The 150 tiny blocks and the 48 blocks on the column route are unchanged and not shown.

## 2\. How is a dense block admitted safely?

The band admits a block with r/n between 0.15 and 0.28 if Open's first pass stays under a cap. The cap is min(256,000, 0.3n) retained rows and 16,384 thick strips. The check sits on the queues' cold path, so the first pass's hot loop is unchanged. Once a block passes either limit, a sticky flag stops the pass, and the block goes to LazyCells as before. Blocks the old test admitted keep their route and no cap. The five dense blocks fit, at up to 231,600 retained rows.

A denser block also means more elimination work, so the port needs a bound on it. Open eliminates intervals by Rauzy–Veech steps: each step replaces the wider of two strips with the difference of their widths. No step ever makes a strip wider, so the widest strip at the start bounds every later step. A block whose strips are all at most 32,768 rows wide therefore runs the plain elimination loop unchanged. A block that fails this certificate takes a contracted loop. It handles a long *spiral*, a run of steps that peels one strip against the same partner, as one step, building its word by repeated squaring.

On Wikidata, no block fails the certificate, so the contracted loop never runs. It is a bound on the worst case, not a speed-up.

## 3\. Where do the extra buffers live?

Admitting the dense blocks must not bring back the memory it saves. The research line's second idea, *EdgesMux*, places working buffers inside the `edges` array, which the kernel does not write until a late phase. Its lifetime argument (Theorem 19.1 of the [cycle 19 journal](<https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md>)) is that each buffer's last read comes before `edges` is first written. Its verifier poisons each buffer after its last read, over 1,000 runs, and the output stays exact.

Most of this was already in 0.5.10, which put the elimination's nodes in `edges` ([\#692](<https://github.com/lmmx/mibz2/pull/692>)). The port moves the remaining buffer, the strips' image order, into the first words of `edges`, worth 27–28 KiB a worker. Its tests poison each region of `edges` after its last read, and the output stays exact. Poisoned before that point, as a control, the output breaks.

## 4\. What is the measured effect?

Two measurements bear on it: the port's own in-process A/B, in a 4-vCPU cloud container, and the release-scaling run on the i9.

The A/B runs master's kernel, unchanged, in the same binary as the candidate, and alternates the two. An A/A control, built from master's own tree, shows the noise floor. There are 12 rounds with 95% intervals.

| measure | A/A control | \#703 against master |
| --- | ---: | ---: |
| kernel heap, worst worker, 10 workers (KiB) | – | 6,804.5 → 1,268.8 (0.186×) |
| 1 thread, six blocks Open already took | 0.993–1.019 | 1.003–1.009 |
| 1 thread, the five newly admitted blocks | 0.987–1.019 | 1.120–1.427 |
| 4 workers, time in the five blocks | 1.004 \[0.994, 1.015\] | 0.925 \[0.909, 0.942\] |
| 4 workers, a whole pass | 1.007 \[1.000, 1.013\] | 1.003 \[0.991, 1.016\] |
| end to end, 8 threads: peak memory | – | 0.741 \[0.714, 0.768\] |
| end to end, 8 threads: wall time | – | 0.978 \[0.928, 1.031\] |
| end to end, 10 threads: peak memory | – | 0.783 \[0.754, 0.812\] |
| end to end, 10 threads: wall time | – | 1.002 \[0.957, 1.050\] |

Ratios are \#703's time or memory over master's. On a fresh kernel, each of the five blocks drops from 5.6 MiB to about 1 MiB.

The worst worker's kernel heap falls to under a fifth because no worker now holds the fallback's table. End to end, the process's peak memory falls by about a quarter, and in the container wall time does not move beyond noise. The five blocks are under 4% of a pass, and the container's cache is not the i9's shared L3.

On the i9, the release-scaling run compared 0.5.11 with 0.5.10 from 1 to 20 threads. At 1, 2 and 4 threads nothing changed beyond noise. From 8 threads up, 0.5.10 took 2.5% to 5.6% longer, with every interval excluding no change. Peak memory fell as in the container.

Per-worker memory against each worker's share of the i9's 20 MiB L3

Measured, two measures. The upper pair is per-worker scratch on the blocks Open admits (0.5.10's changelog, from \#692). The lower pair is the worst worker's kernel heap over the whole corpus at 10 workers (\#703's harness), which includes the fallback's table when a worker meets a refused block. The dashed lines are 20 MiB divided by 7 and by 10 workers.

The wall-time gain on the i9 is consistent with the cache account. With ten workers, each worker's share of the L3 is about 2 MiB. A worker holding a 6.8 MiB fallback evicts its neighbours' working sets, while a worker at 1.3 MiB fits in its share. The container, with a different cache, could not show this. That is our account of the i9 result, not a measurement of cache misses.

## 5\. What transferred from the research line?

The change came from Mechanisms, one of our AI research lines, whose cycle 19 built a candidate called `AhtEdgesMux16`. It measured the candidate in its own harness against mibz2 0.5.8. The harness includes a contention model, a simulator that charges each worker a share of the cache.

The research line's claims against the native measurements

Ratios, new over old; below 1 is better. Claims are the line's own (cycle 19 journal), against 0.5.8 and from its contention model. Container figures are \#703's, against 0.5.10. i9 figures are 0.5.11 against 0.5.10 from the release-scaling run's records, paired by round. Bars are 95% intervals where they exist.

**The memory claim held,** against a newer baseline. The line measured the ten-worker maximum heap falling from 9,979 KiB to 1,346 KiB against 0.5.8. By 0.5.10 the admitted blocks were already at about 1.3 MiB. The gain that remains is the dense blocks' fallback, which no longer allocates its table.

**The speed predictions did not.** The contention model predicted the dense blocks would take under half the cycles, with a cache share of about 2 MiB per worker. Natively, on one thread, Open is slower than the fallback on these blocks. With the cache to itself, the fallback's full table is fast. With four workers in the container, Open takes 7.5% less time on them. On the i9 the whole decoder is faster from 8 threads up. That gain is spread over all blocks, through the smaller working set, rather than concentrated in the five. Its other claims shrank too. EdgesMux was worth 432 KiB against 0.5.8, but most of it had already shipped in 0.5.10. And the spiral contraction never runs on Wikidata.

**Correctness.** Every block of the corpus, and six authenticated reference blocks, decode byte for byte as the plain inverse does. The `bzcat` example at 4, 8 and 10 threads gives the same SHA-256 as `bzip2 -dc`. The correctness gate passes its 338 cases, and eight hand-made mutations of the new code were each caught by the tests. CI's paired bench at 1 to 4 threads found no change in Wikidata's wall time beyond noise.

The change shipped in mibz2 0.5.11.

---

**Research records:** [Mechanisms' cycle 19 journal](<https://github.com/lmmx/mibz2/blob/7d86bb8601800414b2e02a1bd37a493d5cde52c3/docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md>); [the port's journal](<https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission.md>) and [its results](<https://github.com/lmmx/mibz2/blob/90e671966/docs/journal/2026-11-21-open-dense-admission/results.txt>) (\#703); [0.5.11's release-scaling records on the i9](<https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.11>) and [its attribution row](<https://github.com/lmmx/mibz2/blob/master/docs/journal/release-attribution.md>); [0.5.10's memory diet](<https://github.com/lmmx/mibz2/pull/692>).
