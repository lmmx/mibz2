[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/index.md>)

# Walking the Burrows–Wheeler cycle forward across four lanes

Walking the inverse BWT cycle forward across four safe Rust lanes lets workers outrun the serial scanner, alongside three default speedups.

28 November 2026 · 7 min read · [Research result](<#5-what-happens-next>)

Every bzip2 block ends with a pointer chase. Once the decoder builds its inverse Burrows–Wheeler table, it must follow a single cycle through the block to recover the text. The open-returns kernel already shrinks that table to a tenth of the block's rows ([Open Returns](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-19-open-returns/index.md>)). Even so, a worker still stalls on every cache miss along the chain.

Behind an experimental feature flag, mibz2 now breaks that chain open in pure safe Rust. The decoder samples sixteen anchors along the cycle and walks forward across four parallel lanes at once. Each lane writes into its own recycled 32 KiB slab, keeping four independent memory lookups in flight with zero unsafe code.

On a ten-core Intel Core i9-10900K decoding 839 MB of Wikidata, that experimental four-lane walk cuts single-worker time by 17.2%. At eight workers it pushes real throughput to 4.55 GB/s, and 4.99 GB/s when paired with an ideal scanner. More importantly, it flips the eight-worker bottleneck on Wikidata back from the workers to the serial bitstream scanner.

Alongside that experimental walk, three default-path improvements have merged onto master. They remove an extra buffer copy on Python reads, recalibrate the open kernel's admission bands and fold dense-band cap checks into the first pass.

**Takeaways**

- **Four safe forward lanes cut single-worker time by 17.2%.** Behind an experimental feature, walking forward from sixteen sampled anchors into four private 32 KiB slabs trims single-worker Wikidata decoding on a ten-core workstation from 1.353 s to 1.113 s.
- **Eight-worker Wikidata flips from worker-bound to scanner-bound.** Per-worker busy time falls by 10.6% while contended scanner time rises to 179.4 ms. Real throughput climbs by 5.7% to 4.55 GB/s, and ideal-scanner throughput rises 12.1% to 4.99 GB/s.
- **Hot cache working set matters more than peak memory.** Peak eight-worker resident memory rises by 6.3 MiB for recycled slabs that stay cold between blocks, yet multi-core throughput still climbs cleanly.
- **The walk stays opt-in pending a full-guard adoption check.** Ordinary text ran 0.4% faster at identical peak memory, but fifteen-round jitter nudged its upper confidence bound to 1.017× against a strict 1.010× gate.
- **Three default-path speedups are live now.** Direct-copy Python chunk reads, tighter admission bands and fused dense-band cap checks speed up default decoding across Wikidata, text and Python.

## 1\. Why a compact cycle still stalls the core

mibz2's open-returns kernel already shrinks each compressible block to roughly a tenth of its rows. Every retained row points to its successor and carries a short word of output bytes. To emit the block, the default decoder walks that cycle from both ends at once so two lookups stay in flight together.

Two chains are still far below what a modern out-of-order core can hide. A desktop processor can track ten or more memory misses at once, yet a two-ended walk only ever exposes two dependent loads. Every time a link misses the L2 cache, the core waits tens of nanoseconds before it even knows the next address to fetch.

Earlier experiments ran eight lanes inside the general-purpose kernel. That design hit a wall once eight threads shared the L3 cache ([fast alone, slow in a crowd](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-21-fast-alone-slow-in-a-crowd/index.md>)). In the open-returns kernel, however, the edge table is small enough to stay close to the core. The challenge was splitting one cycle into parallel forward walks without adding unsafe code, extra cache pressure or expensive stitching.

## 2\. Sixteen anchors, four lanes and disjoint slabs

The experimental four-lane forward walk solves that problem in three steps, all inside a module that forbids unsafe code.

First, before the walk begins, the kernel keeps ordinary forward links instead of XOR-paired two-way links. It samples sixteen anchor positions spread across the retained table and marks each one with a stop bit in the high bits of its entry. Those anchors slice the single cycle into up to sixteen independent forward segments, with no search required.

Second, a worker walks up to four active segments in lockstep. On every iteration, all four lanes issue their next table lookup together, exposing four independent misses to the processor at once. Whenever one lane reaches another lane's anchor, it retires and pulls the next waiting anchor from the remaining pool. Each active lane count compiles out of line so the compiler keeps every lane cursor in a register as lanes drain.

Third, each lane writes its decoded bytes into its own private, recycled 32 KiB slab rather than competing over one shared output buffer. Because the four slices are completely disjoint, safe Rust proves they never alias and keeps every write cursor in a register. Long words copy directly into the current slab whenever space remains, avoiding spillovers to a separate side buffer. Once all segments finish, a single linear pass stitches the filled slabs into the block's output buffer in cycle order.

Breaking the pointer-chase chain with four disjoint forward lanes

Top: the shipped two-ended walk keeps at most two dependent loads in flight. Bottom: the experimental four-lane walk splits the cycle at sixteen sampled anchors, overlaps four independent misses in safe Rust and writes into four disjoint 32 KiB slabs before one sequential stitch.

## 3\. Ten-core measurements and the bottleneck flip

We tested the experimental four-lane walk on a ten-core Intel Core i9-10900K against the default 0.5.17 build across fifteen interleaved paired rounds. The main workload was four concatenated copies of the Wikidata dump totalling 839 MB of decoded output, alongside a 52 MB ordinary-text guard corpus. Every single run verified byte-exact SHA-256 parity.

| workload (i9-10900K, 15 paired rounds) | default median | experimental 4-lane median | paired ratio (95% CI) | throughput |
| --- | ---: | ---: | --- | ---: |
| Wikidata, 1 worker (839 MB) | 1.353 s | 1.113 s | 0.828× (0.816 to 0.834) | 753 MB/s |
| Wikidata, 8 workers, real scanner | 195.9 ms | 184.4 ms | 0.943× (0.938 to 0.948) | 4.55 GB/s |
| Wikidata, 8 workers, ideal scanner | 190.6 ms | 168.3 ms | 0.879× (0.857 to 0.895) | 4.99 GB/s |
| Ordinary text, 1 worker (52 MB) | 135.4 ms | 137.0 ms | 0.996× (0.989 to 1.017) | 379 MB/s |

The single-worker drop of 231 ms (17.2%) comes entirely from the emission walk itself. In the traced single-worker run, walk and induction time fell by 222.7 ms. Move-to-front decoding, table setup and final run-length expansion moved by less than 6 ms combined.

The traced eight-worker cells reveal a deeper system shift. Under the default build on Wikidata, eight workers still set the pace. Each worker stayed busy for 196.5 ms against a 171.8 ms serial scanner, leaving a 24.7 ms worker-bound gap.

Turn on the experimental four-lane walk and those two stations trade places. Per-worker busy time drops 10.6% from 196.5 ms to 169.6 ms. Meanwhile, the serial scanner slows slightly from 171.8 ms to 179.4 ms under the faster eight-worker load. The pipeline flips to scanner-bound by 9.8 ms. Elapsed time settles at 184.8 ms, within 6 ms of the serial scan floor.

Eight-worker station times on 839 MB Wikidata before and after four forward lanes

Traced eight-worker medians on 839 MB Wikidata. Under the default build, workers take 24.7 ms longer than the serial scanner. With the experimental four-lane walk enabled, workers drop inside the scanner's bar and the serial scanner binds by 9.8 ms. Removing scanner cost (ideal scan) shows the full 12.1% worker speedup at 4.99 GB/s.

That flip also answers a puzzle from our previous scaling post ([why compression ratio moves the scaling ceiling](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-27-why-the-scaling-ceiling-moves/index.md>)). Until now, scanner speedups stayed partly hidden on Wikidata at eight threads because workers still bound the pipeline. Once the four-lane walk ships by default, every millisecond shaved off the serial scanner pays directly at eight cores too.

The memory numbers delivered a second lesson. At eight workers, peak resident memory grew from 106.4 MiB to 112.9 MiB. That adds about 0.8 MiB per worker. Earlier, a 4.2 MiB increase from a hot-loop endpoint redesign had slowed eight-worker decoding by 0.7%. Here, most of the extra memory lives in recycled slab pools that stay cold between blocks. Hot per-worker cache footprint governs multi-core scaling, not headline peak RSS.

Why is the experimental walk not the default yet? On the 52 MB ordinary-text guard, both builds route every block to the general-purpose fallback and use identical peak memory (1.000×). The experimental build was faster on nine of fifteen paired rounds, with a median ratio of 0.996×. Normal single-decode jitter on a 135 ms run still lifted the bootstrap upper confidence bound to 1.017×, narrowly missing the pre-registered 1.010× pass mark. Rather than relax a frozen gate, the walk stays behind an experimental feature until a full multi-corpus adoption run confirms it.

## 4\. Three default-path improvements shipped alongside

While the experimental four-lane walk ran on the workstation, three default-path changes landed on master. They build on two companion advances already covered on this blog: [half the memory on compressible files](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-27-bounded-memory-on-highly-compressible-input/index.md>) and [faster decoding for ordinary text](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-27-faster-decoding-for-ordinary-text/index.md>).

**Direct-copy Python chunk delivery.** Reading chunks from Python previously allocated a staging buffer in Rust and copied bytes twice before returning a Python bytes object. mibz2 now writes straight from the decoder's ready buffer into Python bytes. On a virtual machine decoding Wikidata, single-thread reads ran 7.0% faster with 1 MiB chunks and 3.1% faster with 64 KiB chunks. Eight-thread reads sped up by 3.8% and 4.8%.

**Recalibrated open-kernel admission bands.** Earlier changes made the general-purpose segmented fallback markedly faster on dense text blocks. mibz2's default admission rules now reflect that new balance. Sparse blocks under 10% run density enter the open kernel outright with no cap check. Intermediate blocks between 10% and 21% density are capped at 27% retained rows, keeping every full Wikidata block off the fallback's larger scratch arrays. Blocks at 21% density or above skip straight to the fallback without pre-scanning. On a virtual machine, eight-worker Wikidata used 1.9% less processor time and finished 2.6% faster. Single-worker ordinary text ran 4.4% faster at 0.99× peak memory. On continuous-integration runners, Wikidata ran 0.9% faster at one thread and within 0.3% at four threads with zero regressions.

**Fused dense-band cap checking.** For blocks sitting between 10% and 15% run density, the default decoder previously ran a separate scalar pre-scan over the run records before starting construction. That check now happens inside the open kernel's first pass itself, keeping block-by-block routing identical while eliminating a full memory pass over the records. On a virtual machine, this cut Wikidata processor time by 4.1% at one worker. At eight threads it cut processor time by 2.9% and elapsed time by 3.1%. On continuous-integration runners, four-thread Wikidata ran 0.5% faster in 4.2% less peak memory.

## 5\. What happens next

All three default-path improvements are already merged on master and ready for the next release. The four-lane forward walk remains opt-in behind an experimental feature until a full multi-corpus workstation validation clears every guard corpus and thread count.

Once the four-lane walk becomes the default, eight-worker Wikidata throughput will sit right on the serial scanner's heels. Reaching the next tier will take faster serial scanning and lighter worker setup together. Neither station alone can outrun the other by much anymore.

---

**Sources:**

- Experimental four-lane forward walk and disjoint slabs: [\#1110](<https://github.com/lmmx/mibz2/pull/1110>), [\#1217](<https://github.com/lmmx/mibz2/pull/1217>), [i9-10900K verdict](<https://github.com/lmmx/mibz2/blob/ab47049b/docs/notebook/2026-11-27-forward-emission-i9-verdict.md>), [traced station-flip analysis](<https://github.com/lmmx/mibz2/blob/ab47049b/docs/notebook/2026-11-27-synthesis-nineteenth-pass.md>), and archived bundle `origin/evidence` `e98948b6` (`forward-lanes/run-20261127T223754Z.tar.gz`).
- Earlier endpoint redesign comparison on the same workstation: [\#1053](<https://github.com/lmmx/mibz2/pull/1053>), `origin/evidence` `a6e427b1`.
- Direct-copy Python chunk delivery: [\#1033](<https://github.com/lmmx/mibz2/pull/1033>) and [delta](<https://github.com/lmmx/mibz2/blob/ab47049b/frontier/deltas/2026-11-26-python-read-single-copy.md>).
- Recalibrated open-kernel admission bands: [\#1222](<https://github.com/lmmx/mibz2/pull/1222>) and [delta](<https://github.com/lmmx/mibz2/blob/ab47049b/frontier/deltas/2026-11-27-post-1080-open-admission.md>).
- Fused dense-band cap checking: [\#1246](<https://github.com/lmmx/mibz2/pull/1246>) and [delta](<https://github.com/lmmx/mibz2/blob/ab47049b/frontier/deltas/2026-11-27-open-cap-prescan-cost.md>).
- Companion posts covering bounded memory ([\#1129](<https://github.com/lmmx/mibz2/pull/1129>)) and ordinary-text fallback speedups ([\#1040](<https://github.com/lmmx/mibz2/pull/1040>), [\#1080](<https://github.com/lmmx/mibz2/pull/1080>)): [Half the memory on bzip2's most compressible files](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-27-bounded-memory-on-highly-compressible-input/index.md>) and [Faster decoding for ordinary text](<https://lmmx.github.io/mibz2/pr-preview/pr-1295/blog/2026-11-27-faster-decoding-for-ordinary-text/index.md>).
