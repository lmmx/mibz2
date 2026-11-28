[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/blog/index.md>)

# When the virtual machine lies: why mibz2 now gates releases on real hardware

Cloud benchmarks invert small-block cache behavior and misprice parallel gains, so mibz2 now gates every release on physical hardware.

28 November 2026 · 7 min read · [Research result](<#5-what-changes-when-hardware-has-the-final-word>)

Performance work usually starts on a cloud virtual machine. Cloud hosts spin up fast, automate cleanly and run experiments in parallel. It is tempting to treat a cloud timing as a slower, noisier copy of real silicon.

Ten matched experiments in mibz2 proved that assumption wrong in two opposite ways. On a physical ten-core desktop chip, serial scanning gains transfer almost one-to-one. Parallel worker compute savings shrink roughly in half, while memory-latency wins grow substantially larger. Worse still, a larger cloud L2 cache can flip the sign of a routing choice completely.

Version 0.5.18 is now out. It is our first release gated by an automated physical-hardware regression check before publication. The gate confirmed full recovery from a slowdown in version 0.5.17, delivered roughly 4% faster multi-thread Wikidata decoding and kept peak memory flat.

**Takeaways**

- **Cloud-to-hardware transfer splits sharply by mechanism.** Serial Huffman scanning transfers at a median of 1.05× from our cloud virtual machine to an Intel Core i9-10900K. Parallel worker arithmetic cuts compress to 0.45×. Memory-latency hiding amplifies to 1.37× against baseline cloud builds and 1.85× against branch-aligned cloud builds.
- **A larger cloud L2 cache inverted a small-block routing choice.** On a cloud machine with a 2 MiB L2 cache, small blocks at bzip2 levels 1 and 2 looked slower on our open decoder than on the segmented fallback. On our desktop chip with a 256 KiB L2 cache, the result flipped. The open decoder was already 12% faster at level 2 and 15% faster at level 3.
- **Two cloud wins actually slowed down physical hardware.** A compiler branch-alignment flag cut 15.6% off cloud workers yet made desktop workers 3.1% slower. Adding 4.2 MiB of worker scratch flipped from a 1.8% single-core gain to a 1.1% slowdown at eight workers past the 20 MiB L3 cliff.
- **Version 0.5.18 ships behind an automated physical-hardware preflight gate.** Before publishing to crates.io and PyPI, twelve paired desktop rounds checked every primary and guard corpus. Version 0.5.18 beat version 0.5.17 by 3.3% at eight threads and 3.1% at ten threads in preflight. Post-release scaling reached 3.61 GB/s at ten threads on Wikidata.

## 1\. One transfer factor does not fit a two-stage decoder

Parallel bzip2 decompression in mibz2 splits across two coupled stages. One serial scanner thread reads the compressed bitstream and decodes Huffman symbols. A pool of worker threads then inverts the Burrows–Wheeler transform, expands run-length codes and checks block checksums.

For weeks we prototyped ideas on a 2.3 GHz cloud Xeon virtual machine. We then measured final candidates on a physical 5.3 GHz Intel Core i9-10900K. Early on, we wondered whether cloud timings simply needed one discount factor to predict desktop results. Putting ten matched same-corpus pairs side by side ruled that out.

The ratio of desktop saving to cloud saving splits into three clear regimes, plus two traps that reverse sign completely:

- **Serial front-end scanning transfers nearly one-to-one (median 1.05×).** Four algorithmic scanner changes landed between 0.99× and 1.13× of their cloud percentage on physical silicon. They covered coalesced run decoding, direct selector table builds and two input-window copy cuts. The scanner runs inside a 24 KiB L1-resident table and sequential copies, so neither processor stalls heavily on DRAM.
- **Parallel worker compute cuts shrink roughly in half (median 0.45×).** Removing arithmetic work, tightening SIMD front-end passes or replacing CRC loops delivered only 0.20× to 0.47× of their cloud gain on desktop silicon. On a 5 GHz desktop core, random L3 and DRAM pointer chases dominate worker time during inverse-BWT emission. On a 2.3 GHz cloud core with a much larger L3 cache, compute fills a bigger share of the total. Cutting instructions therefore looks twice as large in the cloud.
- **Worker memory-latency hiding amplifies past one-to-one (1.37× to 1.85×).** Our experimental four-lane forward walk ([walking forward across four lanes](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/blog/2026-11-28-four-lanes-forward/index.md>)) overlaps four independent pointer chains inside the reorder buffer. Because pointer-chase miss latency dominates desktop workers far more than cloud workers, the same gap that shrinks compute wins magnifies latency-hiding wins. Once we bounded its arena to 80 KiB ([shrinking worker scratch](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/blog/2026-11-28-shrinking-worker-scratch-speeds-the-scanner/index.md>)), the four-lane walk jumped from an 11.4% cloud gain to a 17.1% desktop gain on single-worker Wikidata.

Ratio of physical Intel i9-10900K speedup to cloud VM speedup across mechanisms (Wikidata)

Speedups do not scale by one constant between a cloud VM and a desktop processor. Serial scanning transfers roughly one-to-one, worker arithmetic cuts compress by half, and memory-latency hiding amplifies. Two classes reverse sign entirely on real silicon.

The two negative bars at the bottom explain why we no longer trust cloud wins on their own:

1. **Microarchitecture errata do not travel.** Older cloud Xeon chips pay a steep penalty when jump instructions cross 32-byte boundaries. Enabling branch alignment in the compiler sped up our cloud workers by 15.6%. Our physical desktop chip fixes that erratum in silicon. There, the same flag added 2.9% code bloat and made workers 3.1% slower.
2. **Last-level cache cliffs hide behind server L3.** One worker redesign kept 524 KiB of fixed scratch per worker. It looked 3.7% faster on the cloud host's 45 MiB L3 cache. At eight workers on a 20 MiB desktop L3 cache, those extra 4.2 MiB overflowed the cache and slowed decoding down by 1.1%.

We did find one useful middle ground. Our continuous-integration runners now include a single-thread open-decoder check on a modern AMD EPYC processor. That host matches desktop memory latency far better than our older virtual machine. Its 3.6% gain on version 0.5.18 tracked the desktop's 3.3% to 4.0% multi-thread gain almost one-to-one.

## 2\. The L2-cache inversion on small bzip2 blocks

A transfer table only warns you about scale. Our starkest proof that a virtual machine can invert the *direction* of a design choice came from small bzip2 blocks.

Standard bzip2 compresses data in blocks from 100 kB at level 1 up to 900 kB at level 9. When mibz2 decodes a block, it picks between two inverse-BWT kernels. Our open decoder contracts runs into a compact graph to avoid walking a 4-byte-per-symbol navigation array. The segmented fallback builds that flat array directly and walks it in parallel lanes.

On our cloud virtual machine, bzip2 levels 1 through 9 showed a dramatic split. On level 9 blocks of 900,000 rows, the open decoder finished in half the time of the segmented fallback. On level 1 blocks of 100,000 rows, however, it looked 17% to 20% *slower*. At level 2 (200,000 rows), it still looked 6% to 9% slower. That looked like a clear routing bug: small blocks seemed to belong on the segmented fallback.

Before editing any routing code, we asked *why* the crossover sat between 300,000 and 500,000 rows. A 4-byte navigation vector for 300,000 rows takes 1.2 MB. Our cloud virtual machine exposes a **2 MiB per-core L2 cache**, so every level 1 through level 3 fallback vector fit inside L2. A physical Intel Core i9-10900K has only a **256 KiB per-core L2 cache**. On desktop silicon, even a 400 KiB level 1 vector spills straight out of L2 into L3.

We froze a hardware calibration probe before touching the workstation. To justify any routing change, at least one sub-400,000-row bin had to run at least 5% slower on the open decoder in both measurement orders. Otherwise we would stop and leave production routing untouched.

On the physical desktop processor, the cloud result inverted completely.

| Level | Rows | Cloud VM (2 MiB L2) | Physical i9 (256 KiB L2) | i9 Open wins |
| --- | ---: | ---: | ---: | ---: |
| Wikidata level 1 | 100k | 1.17–1.20× | **1.015×** | 42.9% |
| Wikidata level 2 | 200k | 1.06–1.09× | **0.880×** | **83.0%** |
| Wikidata level 3 | 300k | 0.98–1.02× | **0.846×** | **86.2%** |
| Wikidata level 5 | 500k | 0.71–0.76× | **0.869×** | **82.9%** |
| Wikidata level 9 | 900k | 0.48–0.51× | **0.828×** | **85.1%** |

Values below 1.00× mean the open decoder is faster than the segmented fallback. On the 2 MiB L2 cloud machine, level 2 looked 8% slower on the open decoder, and level 3 looked tied. On the 256 KiB L2 desktop chip, the sign flipped. The open decoder was already **12.0% faster at level 2** and **15.4% faster at level 3**. It won more than five in six blocks at both levels.

Grouping the desktop blocks by run-to-length ratio exposed the deeper truth. There was zero block-size effect at all. Within any fixed run-density band, the ratio stayed flat from 100,000 rows to 900,000 rows. Parity sat near a run density of 0.10 at every block size. On text, level 1 (1.032×) matched level 9 (1.036×) within four thousandths. The mild rise from level 9 to level 1 on Wikidata came purely from shorter blocks having slightly higher average run density.

Had we trusted the cloud benchmark, we would have shipped a double-digit regression on desktop hardware. The frozen probe stopped that change before we edited a single line of routing code.

Left: L2 cache size flips small-block routing. Right: version 0.5.18 desktop preflight recovery.

Left: a 2 MiB cloud L2 cache makes level 1 and level 2 blocks look slower on the open decoder, whereas a 256 KiB desktop L2 cache shows it already wins level 2 by 12% and level 3 by 15%. Right: version 0.5.18's automated hardware preflight recovers the version 0.5.17 regression and finishes net faster than version 0.5.16.

## 3\. How a 2.6% regression slipped into 0.5.17

Just before version 0.5.18, we saw the same gap hit a live release. Version 0.5.17 added a quick dense-band check so ordinary text blocks could exit early without wasted setup work. On cloud benchmarks, the check looked completely harmless. Only *after* publishing version 0.5.17 did the desktop scaling sweep catch a 2.6% slowdown at eight threads and a 1.5% slowdown at ten threads on Wikidata.

The culprit was a redundant pre-scan over 3.31 million run records on medium-density blocks. Those blocks were already admitted by the open decoder anyway. Because the desktop sweep ran after tagging, users received the regression before our workstation flagged it.

That post-mortem changed how mibz2 ships.

## 4\. Automated hardware gates and release 0.5.18

Every release of mibz2 now clears an automated physical-hardware preflight gate *before* deployment. Until that gate passes, the release workflow refuses to tag, build wheels or publish to crates.io and PyPI.

The gate checks four things mechanically:

1. **Exact candidate identity.** The preflight bundle must match the exact commit being released, with no modified decoder sources since the run.
2. **Unrestricted physical silicon.** The runner verifies CPU model, full online core mask, unlimited CPU quota and clock-mode stability. Every arm must also match C libbzip2 byte for byte.
3. **Zero regressions across primary and guard corpora.** Twelve paired alternating rounds test the candidate against the prior release across Wikidata, text and five guard corpora. Peak memory is checked on every cell as well.
4. **Explicit recovery against the last known good tag.** When the prior release carried a known regression, the gate also pairs the candidate directly against the last clean tag. During audit we caught a parser bug that silently ignored malformed recovery cell names. We hardened it to fail closed before the first release shipped.

Version 0.5.18 is the first release to clear this gate. It ships five improvements together:

- **Skipping the redundant pre-scan** on medium-density blocks, erasing version 0.5.17's overhead with zero routing changes.
- **Faster Huffman table construction** in the serial scanner, cutting single-thread desktop scan time by 4.5%.
- **Tighter AVX2 run-front parsing** in workers, replacing cross-lane shuffles with 64-symbol vector tests and hoisted register steps.
- **Safe-Rust open-decoder wiring**, removing redundant bounds checks and using overlapping vector stores with zero added memory.
- **Direct Rust Python line and streaming delivery** inside one universal Stable ABI wheel ([twenty-six times faster Python line reads](<https://lmmx.github.io/mibz2/pr-preview/pr-1282/blog/2026-11-28-python-line-delivery-stable-abi/index.md>)).

The physical desktop preflight ran twelve paired rounds on 839 MB of Wikidata, text and five guard corpora. Against version 0.5.17, version 0.5.18 ran **3.3% faster at eight threads** and **3.1% faster at ten threads**. From twelve to twenty threads, it gained **4.2% to 5.9%**. Ten-thread text decoding also sped up by **1.5%**.

Directly paired against version 0.5.16, version 0.5.18 finished **0.9% faster at eight threads** and **1.3% faster at ten threads**. Both intervals stayed strictly below parity. Peak memory stayed within 1.0 MB of baseline across every cell.

Post-release scaling on 209.7 MB of Wikidata confirmed the same gains across ten fast-mode rounds. At eight threads, throughput climbed from **3.12 GB/s to 3.24 GB/s** (+3.9%). At ten threads, it rose from **3.46 GB/s to 3.61 GB/s** (+4.1%). On the same ten cores, lbzip2 ran at 0.42 GB/s.

## 5\. What changes when hardware has the final word

Putting the transfer ledger, the L2-cache inversion and the hardware gate together gives us three clear rules for every change ahead:

1. **Price cloud prototypes by mechanism, never by raw percentage.** Expect serial scanner cuts to hold near 1.0× on desktop silicon. Cut worker arithmetic gains roughly in half. Expect memory-latency hiding to grow stronger, as long as eight-worker peak memory stays flat.
2. **Check cache-size assumptions before touching routing heuristics.** Any decision sensitive to a working set between 256 KiB and 2 MiB can flip sign between a server L2 cache and a desktop L2 cache.
3. **Gate releases on real silicon before tagging.** Catching a 2.6% slowdown before deployment is far cheaper than shipping a corrective release afterward.

Version 0.5.18 is live on crates.io and PyPI today. Behind an opt-in experimental flag, our bounded 80 KiB four-lane forward walk already reaches nearly 5 GB/s with a pinned scanner. Its full unpinned desktop adoption run is next in the hardware queue.

---

**Sources:**

- Transfer ledger (10 matched levers and failure modes): [\#1368](<https://github.com/lmmx/mibz2/pull/1368>), [transfer ledger notebook](<https://github.com/lmmx/mibz2/blob/80222230/docs/notebook/2026-11-28-vm-i9-transfer-ledger.md>).
- L2-cache inversion proof (`block-size-route-i9-probe`, STOP): [\#1369](<https://github.com/lmmx/mibz2/pull/1369>), [\#1374](<https://github.com/lmmx/mibz2/pull/1374>), [verdict notebook](<https://github.com/lmmx/mibz2/blob/80222230/docs/notebook/2026-11-28-block-size-route-i9-verdict.md>), and `origin/evidence` `7f03816b`.
- Automated hardware preflight gate and audit: [\#1256](<https://github.com/lmmx/mibz2/pull/1256>), [\#1357](<https://github.com/lmmx/mibz2/pull/1357>), [\#1359](<https://github.com/lmmx/mibz2/pull/1359>), and `origin/evidence` `c8489a45`.
- Version 0.5.18 changes and scaling: [\#1117](<https://github.com/lmmx/mibz2/pull/1117>), [\#1246](<https://github.com/lmmx/mibz2/pull/1246>), [\#1274](<https://github.com/lmmx/mibz2/pull/1274>), [\#1294](<https://github.com/lmmx/mibz2/pull/1294>), [\#1335](<https://github.com/lmmx/mibz2/pull/1335>), [release attribution](<https://github.com/lmmx/mibz2/blob/80222230/docs/journal/release-attribution.md>), and `bench/release-scaling/2026-11-28-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.18/`.
