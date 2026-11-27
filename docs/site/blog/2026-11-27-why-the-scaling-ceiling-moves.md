---
title: "Why compression ratio moves the scaling ceiling"
image: assets/social/blog-2026-11-27-why-the-scaling-ceiling-moves.png
description: "On a ten-core processor, the same parallel bzip2 decoder scales to nine threads on structured dumps yet peaks at five on ordinary source code and text."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Why compression ratio moves the scaling ceiling

On a ten-core processor, the same parallel bzip2 decoder scales to nine threads on structured dumps yet peaks at five on ordinary source code and text.
{: .post-dek }

27 November 2026 · 8 min read · [Research result](#5-what-this-means-for-decoding-in-practice)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

Benchmark a parallel bzip2 decoder on a ten-core processor with a 210 MB Wikidata dump. Adding
threads keeps paying off all the way to nine workers. Now hand that same binary a compressed
archive of Rust source code or JSON. It stops speeding up at four or five workers. Give it eight or
ten workers and it actually runs slower.

The reason has nothing to do with thread scheduling or the tool that compressed the file. It is
built into bzip2 itself. Every block passes through two stations in sequence
([the decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md)). One serial
front stage decodes Huffman symbols. A pool of parallel workers then inverts the Burrows–Wheeler
transform (BWT) to recover the original bytes. The serial stage pays per Huffman symbol. The
parallel workers pay per output byte. Compression ratio sets the exchange rate between those two
currencies.

Alongside these hardware measurements, three default-path improvements have shipped in mibz2 0.5.15
and 0.5.16. They bring a faster Huffman scanner, wider branchless run decoding and chunked
whole-file reads in Python. Together they show why the same speed-up can stay hidden on one
benchmark yet pay off cleanly on another.

**Takeaways**

- **The scaling knee moves with the input.** On a ten-core Intel Core i9-10900K, Wikidata keeps
  scaling to nine workers. Ordinary source code and JSON peak at four or five workers.
- **Symbol density drives the shift.** Scanner time per Huffman symbol barely varies across text
  corpora (1.41 to 1.48 ns). Ordinary text simply packs three to five times more symbols into each
  output megabyte than a repetitive dump does.
- **Extra threads hurt past the knee.** Once the serial scanner saturates on Rust source or JSON,
  adding workers past six slows decoding down. Moving from five workers to ten costs roughly twice
  as much total processor time.
- **Faster scanning hides until the scanner binds.** The new Huffman scanner in 0.5.15 cut serial
  scan time on Wikidata by 10.5% and ten-thread time by 4.0%. Four-thread and eight-thread Wikidata
  timings barely moved because workers still set the pace there.
- **Two more default improvements landed in 0.5.16.** Raising the run-decoding ceiling from 50% to
  75% speeds up compressible Markdown, ELF binaries and JSON. Whole-file Python reads now pull
  1 MiB chunks directly without changing wheel compatibility.

## 1. Two stations, two currencies

In a block-parallel bzip2 decoder, one thread must scan the compressed bit stream in order. bzip2
blocks are not byte-aligned and carry no index of their boundaries. That thread reads each block's
headers, selector list and Huffman-coded symbols. It then hands the decoded symbols to whichever
worker is free. The workers run move-to-front decoding, BWT inversion, run-length expansion and CRC
verification in parallel.

Ignore startup and teardown for a moment. Suppose scanning a block takes time $S$ and worker
processing takes time $W$. Then $k$ workers keep pace with the scanner when $W / k = S$. That gives
a natural balance point:

$$
k^* = \frac{W}{S} = \frac{w_{\text{byte}}}{s_{\text{sym}} \cdot \rho}
$$

Here $s_{\text{sym}}$ is the scanner's cost per Huffman symbol and $w_{\text{byte}}$ is
the worker's cost per output byte. The factor $\rho$ is the number of Huffman symbols per output
byte. Repetitive data compresses tightly because move-to-front and zero-run encoding collapse long
runs into very few symbols. That makes $\rho$ small. Higher-entropy text leaves far more non-zero
symbols behind, pushing $\rho$ up.

We measured $S$ and $W$ directly on one pinned core of an Intel Core i9-10900K. Two controlled runs
covered seven corpora in all. The first tested Wikidata beside three repository corpora: Rust
source, JSON and Markdown. The second tested Wikidata beside mixed prose and code, random bytes and
fixed-width records.

| corpus | compression ratio | symbols / output byte | scanner, ns / sym | worker, ns / byte | balance $k^*$ |
|---|---:|---:|---:|---:|---:|
| Wikidata dump | 17.7× | 0.119 | 1.41 | 1.60 | 9.55 |
| mixed prose and code | 6.5× | 0.339 | 1.48 | 2.56 | 5.09 |
| JSON | 4.9× | 0.432 | 1.43 | 2.59 | 4.19 |
| Rust source | 5.2× | 0.419 | 1.48 | 2.58 | 4.16 |
| random bytes | 1.1× | 1.000 | 2.00 | 7.79 | 3.90 |
| fixed records | 1.0× | 1.000 | 2.08 | 7.62 | 3.68 |
| Markdown docs | 3.9× | 0.553 | 1.44 | 2.72 | 3.43 |

Look at the fourth column first. Across every text and structured corpus, the scanner spends almost
the same time per Huffman symbol: 1.41 to 1.48 ns. On random bytes and uncompressible records,
longer average codewords nudge that cost to 2.00 to 2.08 ns.

What changes dramatically is the third column. Wikidata needs only 0.119 symbols per output byte.
Rust source and JSON need 0.419 and 0.432 — more than three and a half times as many. Markdown
needs 0.553, nearly five times as many. Worker cost per output byte rises too, because less
compressible blocks move to the general-purpose kernel
([faster decoding for ordinary text](2026-11-27-faster-decoding-for-ordinary-text.md)). Yet worker
cost rises by only 1.6 to 1.7 times on the text panel, far less than symbol density does. The
balance point $k^*$ therefore drops by more than half.

<figure class="bf-fig" id="station-balance">
<p class="bf-title">Workers needed to match the serial scanner on ten-core hardware</p>
<!-- fig:station-balance -->
<svg viewBox="0 0 720 326" role="img" aria-label="Horizontal bar chart of single-thread station balance k* on an Intel Core i9-10900K. Wikidata dump 9.55, mixed prose and code 5.09, JSON 4.19, Rust source 4.16, random bytes 3.90, fixed records 3.68, Markdown docs 3.43."><text class="bf-tm" x="150" y="24" text-anchor="end" style="font-size:11px">corpus</text><text class="bf-tm" x="214" y="24" text-anchor="end" style="font-size:11px">ratio</text><text class="bf-tm" x="230" y="24" text-anchor="start" style="font-size:11px">workers needed to match the scanner (k* = W / S)</text><line x1="230.0" y1="36.0" x2="230.0" y2="286.0" style="stroke:var(--bf-vz-axis);stroke-width:1.5"/><text class="bf-tm" x="230.0" y="302.0" text-anchor="middle" style="font-size:11px">0</text><line x1="314.0" y1="36.0" x2="314.0" y2="286.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="314.0" y="302.0" text-anchor="middle" style="font-size:11px">2</text><line x1="398.0" y1="36.0" x2="398.0" y2="286.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="398.0" y="302.0" text-anchor="middle" style="font-size:11px">4</text><line x1="482.0" y1="36.0" x2="482.0" y2="286.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="482.0" y="302.0" text-anchor="middle" style="font-size:11px">6</text><line x1="566.0" y1="36.0" x2="566.0" y2="286.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="566.0" y="302.0" text-anchor="middle" style="font-size:11px">8</text><line x1="650.0" y1="36.0" x2="650.0" y2="286.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="650.0" y="302.0" text-anchor="middle" style="font-size:11px">10</text><text class="bf-t-strong" x="150.0" y="58.0" text-anchor="end" style="font-size:12px">Wikidata dump</text><text class="bf-tm" x="214.0" y="58.0" text-anchor="end" style="font-size:11px">17.7×</text><rect x="230.0" y="45.0" width="401.1" height="18.0" rx="3" style="fill:var(--bf-vz-1)"><title>Wikidata dump: k* = 9.55, compression 17.7×, 0.119 symbols/byte</title></rect><text class="bf-t-strong" x="639.1" y="58.0" text-anchor="start" style="font-size:12px">9.55</text><text class="bf-tm" x="150.0" y="93.0" text-anchor="end" style="font-size:12px">Mixed prose & code</text><text class="bf-tm" x="214.0" y="93.0" text-anchor="end" style="font-size:11px">6.5×</text><rect x="230.0" y="80.0" width="213.8" height="18.0" rx="3" style="fill:var(--bf-vz-3)"><title>Mixed prose & code: k* = 5.09, compression 6.5×, 0.339 symbols/byte</title></rect><text class="bf-tm" x="451.8" y="93.0" text-anchor="start" style="font-size:12px">5.09</text><text class="bf-tm" x="150.0" y="128.0" text-anchor="end" style="font-size:12px">JSON</text><text class="bf-tm" x="214.0" y="128.0" text-anchor="end" style="font-size:11px">4.9×</text><rect x="230.0" y="115.0" width="176.0" height="18.0" rx="3" style="fill:var(--bf-vz-2)"><title>JSON: k* = 4.19, compression 4.9×, 0.432 symbols/byte</title></rect><text class="bf-tm" x="414.0" y="128.0" text-anchor="start" style="font-size:12px">4.19</text><text class="bf-tm" x="150.0" y="163.0" text-anchor="end" style="font-size:12px">Rust source</text><text class="bf-tm" x="214.0" y="163.0" text-anchor="end" style="font-size:11px">5.2×</text><rect x="230.0" y="150.0" width="174.7" height="18.0" rx="3" style="fill:var(--bf-vz-2)"><title>Rust source: k* = 4.16, compression 5.2×, 0.419 symbols/byte</title></rect><text class="bf-tm" x="412.7" y="163.0" text-anchor="start" style="font-size:12px">4.16</text><text class="bf-tm" x="150.0" y="198.0" text-anchor="end" style="font-size:12px">Random bytes</text><text class="bf-tm" x="214.0" y="198.0" text-anchor="end" style="font-size:11px">1.1×</text><rect x="230.0" y="185.0" width="163.8" height="18.0" rx="3" style="fill:var(--bf-vz-4)"><title>Random bytes: k* = 3.90, compression 1.1×, 1.000 symbols/byte</title></rect><text class="bf-tm" x="401.8" y="198.0" text-anchor="start" style="font-size:12px">3.90</text><text class="bf-tm" x="150.0" y="233.0" text-anchor="end" style="font-size:12px">Fixed records</text><text class="bf-tm" x="214.0" y="233.0" text-anchor="end" style="font-size:11px">1.0×</text><rect x="230.0" y="220.0" width="154.6" height="18.0" rx="3" style="fill:var(--bf-vz-4)"><title>Fixed records: k* = 3.68, compression 1.0×, 1.000 symbols/byte</title></rect><text class="bf-tm" x="392.6" y="233.0" text-anchor="start" style="font-size:12px">3.68</text><text class="bf-tm" x="150.0" y="268.0" text-anchor="end" style="font-size:12px">Markdown docs</text><text class="bf-tm" x="214.0" y="268.0" text-anchor="end" style="font-size:11px">3.9×</text><rect x="230.0" y="255.0" width="144.1" height="18.0" rx="3" style="fill:var(--bf-vz-2)"><title>Markdown docs: k* = 3.43, compression 3.9×, 0.553 symbols/byte</title></rect><text class="bf-tm" x="382.1" y="268.0" text-anchor="start" style="font-size:12px">3.43</text><text class="bf-tm" x="650.0" y="320.0" text-anchor="end" style="font-size:11px">k* = worker time / scanner time (1 thread, Intel Core i9-10900K)</text></svg>
<figcaption>Single-thread station balance on an Intel Core i9-10900K (median of three rounds,
fastest of five passes per block). Wikidata, Rust source, JSON and Markdown come from the
text-panel run; mixed prose and code, random bytes and fixed records come from the workstation's
default-build balance sweep on the same machine.</figcaption>
</figure>

A separate eleven-corpus survey on a virtual machine ruled out the encoder as a confound. The
Wikidata dump was compressed by lbzip2, while the text corpora were compressed with standard bzip2
at level 9. Re-encoding Wikidata with standard bzip2 left its balance point within 2% of the
original dump. The difference comes from the data, not the compressor.

## 2. Watching the knee move on ten cores

Single-thread station balance predicts demand, not real wall time. Cache contention, thread wakeups
and memory traffic can shift the true knee. We therefore ran a pre-registered thread sweep on the
same Intel Core i9-10900K. It tested four to ten workers over nine interleaved rounds per thread
count. Each text corpus was concatenated into a multi-stream file of roughly 183 to 190 MB of
output so startup noise stayed small. Wikidata was tested as four concatenated copies totalling
839 MB.

The frozen criterion defined a corpus's scaling knee as the smallest worker count within 2% of its
best nine-round median elapsed time.

| corpus | balance $k^*$ | 4 workers, ms | 5 workers, ms | 6 workers, ms | 8 workers, ms | 9 workers, ms | 10 workers, ms | median knee | fastest-round knee |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Wikidata (839 MB) | 9.55 | 473.9 | 403.8 | 358.1 | 300.7 | 269.6 | 288.5 | 9 | 9 |
| Rust source (183 MB) | 4.16 | 207.2 | 202.9 | 203.2 | 256.5 | 256.2 | 263.9 | 5 | 5 |
| JSON (190 MB) | 4.19 | 218.5 | 217.5 | 221.5 | 259.4 | 270.1 | 257.4 | 4 | 4 |
| Markdown (184 MB) | 3.43 | 296.6 | 298.8 | 303.8 | 203.9 | 253.8 | 240.3 | 8 | 4 |

The pre-registered hypothesis passed cleanly. The text panel's median scaling knee was five
workers, while Wikidata's knee was nine. On Rust source and JSON, adding workers past six did not
merely flatten out. It made decoding measurably slower. Moving from five workers to ten pushed
median time up 30% on Rust source and 18% on JSON. Total processor time roughly doubled as extra
threads contended for shared L3 cache and queue locks.

Markdown looks like an outlier in the median column. It landed at eight workers even though its
single-thread balance point is the lowest at 3.43. The per-round timings explain why. The
workstation ran under Linux's powersave governor, idling at 800 MHz between runs. Individual 200 ms
bursts therefore split cleanly into two speed clusters depending on turbo ramp-up. Full-clock runs
at 4.9 GHz took about 190 to 200 ms, while slower-ramp runs took 260 to 305 ms. On Markdown, four
of nine rounds stayed fast at four and five workers, versus five of nine at eight workers. That
single round flipped which side of the gap the median landed on.

<figure class="bf-fig" id="scaling-curves">
<p class="bf-title">Normalised decode time from four to ten workers on ten-core hardware</p>
<!-- fig:scaling-curves -->
<svg viewBox="0 0 720 344" role="img" aria-label="Two-panel line chart of elapsed decode time from 4 to 10 workers on an Intel Core i9-10900K, normalised to 4 workers. Left panel shows nine-round medians: Wikidata falls to 0.57x at 9 workers while Rust source bottoms at 0.98x at 5 and JSON at 1.00x at 4 to 5 before rising past 1.2x. Right panel shows fastest-of-nine rounds: Wikidata falls to 0.52x at 9 workers, Rust source bottoms at 0.85x at 5, JSON is flat at 1.00x at 4 to 6, and Markdown is already at its floor of 1.00x at 4 workers."><line x1="52.0" y1="20.0" x2="72.0" y2="20.0" style="stroke:var(--bf-vz-1);stroke-width:2.5"/><circle cx="62.0" cy="20.0" r="3.5" style="fill:var(--bf-vz-1)"/><text class="bf-tm" x="78.0" y="24.0" text-anchor="start" style="font-size:12px">Wikidata (17.7×)</text><line x1="218.0" y1="20.0" x2="238.0" y2="20.0" style="stroke:var(--bf-vz-2);stroke-width:2.5"/><circle cx="228.0" cy="20.0" r="3.5" style="fill:var(--bf-vz-2)"/><text class="bf-tm" x="244.0" y="24.0" text-anchor="start" style="font-size:12px">Rust source (5.2×)</text><line x1="384.0" y1="20.0" x2="404.0" y2="20.0" style="stroke:var(--bf-vz-3);stroke-width:2.5"/><circle cx="394.0" cy="20.0" r="3.5" style="fill:var(--bf-vz-3)"/><text class="bf-tm" x="410.0" y="24.0" text-anchor="start" style="font-size:12px">JSON (4.9×)</text><line x1="550.0" y1="20.0" x2="570.0" y2="20.0" style="stroke:var(--bf-vz-4);stroke-width:2.5;stroke-dasharray:4,3"/><circle cx="560.0" cy="20.0" r="3.5" style="fill:var(--bf-vz-4)"/><text class="bf-tm" x="576.0" y="24.0" text-anchor="start" style="font-size:12px">Markdown (3.9×)</text><text class="bf-t-strong" x="200.0" y="54.0" text-anchor="middle" style="font-size:12px">Nine-round median (pre-registered)</text><line x1="56.0" y1="254.0" x2="344.0" y2="254.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="48.0" y="258.0" text-anchor="end" style="font-size:11px">0.6×</text><line x1="56.0" y1="206.0" x2="344.0" y2="206.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="48.0" y="210.0" text-anchor="end" style="font-size:11px">0.8×</text><line x1="56.0" y1="158.0" x2="344.0" y2="158.0" style="stroke:var(--bf-vz-axis);stroke-width:1.4"/><text class="bf-tm" x="48.0" y="162.0" text-anchor="end" style="font-size:11px">1.0×</text><line x1="56.0" y1="110.0" x2="344.0" y2="110.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="48.0" y="114.0" text-anchor="end" style="font-size:11px">1.2×</text><line x1="56.0" y1="74.0" x2="56.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="56.0" y="308.0" text-anchor="middle" style="font-size:11px">4</text><line x1="104.0" y1="74.0" x2="104.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="104.0" y="308.0" text-anchor="middle" style="font-size:11px">5</text><line x1="152.0" y1="74.0" x2="152.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="152.0" y="308.0" text-anchor="middle" style="font-size:11px">6</text><line x1="200.0" y1="74.0" x2="200.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="200.0" y="308.0" text-anchor="middle" style="font-size:11px">7</text><line x1="248.0" y1="74.0" x2="248.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="248.0" y="308.0" text-anchor="middle" style="font-size:11px">8</text><line x1="296.0" y1="74.0" x2="296.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="296.0" y="308.0" text-anchor="middle" style="font-size:11px">9</text><line x1="344.0" y1="74.0" x2="344.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="344.0" y="308.0" text-anchor="middle" style="font-size:11px">10</text><text class="bf-tm" x="200.0" y="326.0" text-anchor="middle" style="font-size:11px">worker threads</text><polyline fill="none" points="56.0,158.0 104.0,193.5 152.0,216.6 200.0,233.1 248.0,245.6 296.0,261.4 344.0,251.8" style="stroke:var(--bf-vz-1);stroke-width:2.3"/><circle cx="56.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 4 workers: 1.00×</title></circle><circle cx="104.0" cy="193.5" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 5 workers: 0.85×</title></circle><circle cx="152.0" cy="216.6" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 6 workers: 0.76×</title></circle><circle cx="200.0" cy="233.1" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 7 workers: 0.69×</title></circle><circle cx="248.0" cy="245.6" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 8 workers: 0.64×</title></circle><circle cx="296.0" cy="261.4" r="5.0" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2.0"><title>Wikidata, 9 workers: 0.57×</title></circle><circle cx="344.0" cy="251.8" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 10 workers: 0.61×</title></circle><polyline fill="none" points="56.0,158.0 104.0,163.0 152.0,162.6 200.0,131.1 248.0,100.9 296.0,101.4 344.0,92.2" style="stroke:var(--bf-vz-2);stroke-width:2.3"/><circle cx="56.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 4 workers: 1.00×</title></circle><circle cx="104.0" cy="163.0" r="5.0" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2.0"><title>Rust source, 5 workers: 0.98×</title></circle><circle cx="152.0" cy="162.6" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 6 workers: 0.98×</title></circle><circle cx="200.0" cy="131.1" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 7 workers: 1.11×</title></circle><circle cx="248.0" cy="100.9" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 8 workers: 1.24×</title></circle><circle cx="296.0" cy="101.4" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 9 workers: 1.24×</title></circle><circle cx="344.0" cy="92.2" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 10 workers: 1.27×</title></circle><polyline fill="none" points="56.0,158.0 104.0,159.2 152.0,154.6 200.0,140.0 248.0,113.1 296.0,101.4 344.0,115.3" style="stroke:var(--bf-vz-3);stroke-width:2.3"/><circle cx="56.0" cy="158.0" r="5.0" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:2.0"><title>JSON, 4 workers: 1.00×</title></circle><circle cx="104.0" cy="159.2" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 5 workers: 0.99×</title></circle><circle cx="152.0" cy="154.6" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 6 workers: 1.01×</title></circle><circle cx="200.0" cy="140.0" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 7 workers: 1.07×</title></circle><circle cx="248.0" cy="113.1" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 8 workers: 1.19×</title></circle><circle cx="296.0" cy="101.4" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 9 workers: 1.24×</title></circle><circle cx="344.0" cy="115.3" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 10 workers: 1.18×</title></circle><polyline fill="none" points="56.0,158.0 104.0,156.3 152.0,152.2 200.0,188.5 248.0,233.1 296.0,192.6 344.0,203.6" style="stroke:var(--bf-vz-4);stroke-width:2.3;stroke-dasharray:4,3"/><circle cx="56.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 4 workers: 1.00×</title></circle><circle cx="104.0" cy="156.3" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 5 workers: 1.01×</title></circle><circle cx="152.0" cy="152.2" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 6 workers: 1.02×</title></circle><circle cx="200.0" cy="188.5" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 7 workers: 0.87×</title></circle><circle cx="248.0" cy="233.1" r="5.0" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:2.0"><title>Markdown, 8 workers: 0.69×</title></circle><circle cx="296.0" cy="192.6" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 9 workers: 0.86×</title></circle><circle cx="344.0" cy="203.6" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 10 workers: 0.81×</title></circle><text class="bf-t-strong" x="552.0" y="54.0" text-anchor="middle" style="font-size:12px">Fastest of nine rounds (full turbo clock)</text><line x1="408.0" y1="254.0" x2="696.0" y2="254.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="400.0" y="258.0" text-anchor="end" style="font-size:11px">0.6×</text><line x1="408.0" y1="206.0" x2="696.0" y2="206.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="400.0" y="210.0" text-anchor="end" style="font-size:11px">0.8×</text><line x1="408.0" y1="158.0" x2="696.0" y2="158.0" style="stroke:var(--bf-vz-axis);stroke-width:1.4"/><text class="bf-tm" x="400.0" y="162.0" text-anchor="end" style="font-size:11px">1.0×</text><line x1="408.0" y1="110.0" x2="696.0" y2="110.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="400.0" y="114.0" text-anchor="end" style="font-size:11px">1.2×</text><line x1="408.0" y1="74.0" x2="408.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="408.0" y="308.0" text-anchor="middle" style="font-size:11px">4</text><line x1="456.0" y1="74.0" x2="456.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="456.0" y="308.0" text-anchor="middle" style="font-size:11px">5</text><line x1="504.0" y1="74.0" x2="504.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="504.0" y="308.0" text-anchor="middle" style="font-size:11px">6</text><line x1="552.0" y1="74.0" x2="552.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="552.0" y="308.0" text-anchor="middle" style="font-size:11px">7</text><line x1="600.0" y1="74.0" x2="600.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="600.0" y="308.0" text-anchor="middle" style="font-size:11px">8</text><line x1="648.0" y1="74.0" x2="648.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="648.0" y="308.0" text-anchor="middle" style="font-size:11px">9</text><line x1="696.0" y1="74.0" x2="696.0" y2="290.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="696.0" y="308.0" text-anchor="middle" style="font-size:11px">10</text><text class="bf-tm" x="552.0" y="326.0" text-anchor="middle" style="font-size:11px">worker threads</text><polyline fill="none" points="408.0,158.0 456.0,205.0 504.0,233.4 552.0,254.7 600.0,265.5 648.0,274.4 696.0,269.4" style="stroke:var(--bf-vz-1);stroke-width:2.3"/><circle cx="408.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 4 workers: 1.00×</title></circle><circle cx="456.0" cy="205.0" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 5 workers: 0.80×</title></circle><circle cx="504.0" cy="233.4" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 6 workers: 0.69×</title></circle><circle cx="552.0" cy="254.7" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 7 workers: 0.60×</title></circle><circle cx="600.0" cy="265.5" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 8 workers: 0.55×</title></circle><circle cx="648.0" cy="274.4" r="5.0" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:2.0"><title>Wikidata, 9 workers: 0.52×</title></circle><circle cx="696.0" cy="269.4" r="3.2" style="fill:var(--bf-vz-1);stroke:var(--bf-surface);stroke-width:1.2"><title>Wikidata, 10 workers: 0.54×</title></circle><polyline fill="none" points="408.0,158.0 456.0,195.0 504.0,182.2 552.0,156.6 600.0,132.6 648.0,154.6 696.0,128.7" style="stroke:var(--bf-vz-2);stroke-width:2.3"/><circle cx="408.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 4 workers: 1.00×</title></circle><circle cx="456.0" cy="195.0" r="5.0" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:2.0"><title>Rust source, 5 workers: 0.85×</title></circle><circle cx="504.0" cy="182.2" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 6 workers: 0.90×</title></circle><circle cx="552.0" cy="156.6" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 7 workers: 1.01×</title></circle><circle cx="600.0" cy="132.6" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 8 workers: 1.11×</title></circle><circle cx="648.0" cy="154.6" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 9 workers: 1.01×</title></circle><circle cx="696.0" cy="128.7" r="3.2" style="fill:var(--bf-vz-2);stroke:var(--bf-surface);stroke-width:1.2"><title>Rust source, 10 workers: 1.12×</title></circle><polyline fill="none" points="408.0,158.0 456.0,156.8 504.0,158.0 552.0,135.9 600.0,94.6 648.0,106.2 696.0,98.0" style="stroke:var(--bf-vz-3);stroke-width:2.3"/><circle cx="408.0" cy="158.0" r="5.0" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:2.0"><title>JSON, 4 workers: 1.00×</title></circle><circle cx="456.0" cy="156.8" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 5 workers: 1.00×</title></circle><circle cx="504.0" cy="158.0" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 6 workers: 1.00×</title></circle><circle cx="552.0" cy="135.9" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 7 workers: 1.09×</title></circle><circle cx="600.0" cy="94.6" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 8 workers: 1.26×</title></circle><circle cx="648.0" cy="106.2" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 9 workers: 1.22×</title></circle><circle cx="696.0" cy="98.0" r="3.2" style="fill:var(--bf-vz-3);stroke:var(--bf-surface);stroke-width:1.2"><title>JSON, 10 workers: 1.25×</title></circle><polyline fill="none" points="408.0,158.0 456.0,153.4 504.0,148.2 552.0,144.8 600.0,144.3 648.0,142.9 696.0,146.0" style="stroke:var(--bf-vz-4);stroke-width:2.3;stroke-dasharray:4,3"/><circle cx="408.0" cy="158.0" r="5.0" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:2.0"><title>Markdown, 4 workers: 1.00×</title></circle><circle cx="456.0" cy="153.4" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 5 workers: 1.02×</title></circle><circle cx="504.0" cy="148.2" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 6 workers: 1.04×</title></circle><circle cx="552.0" cy="144.8" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 7 workers: 1.05×</title></circle><circle cx="600.0" cy="144.3" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 8 workers: 1.06×</title></circle><circle cx="648.0" cy="142.9" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 9 workers: 1.06×</title></circle><circle cx="696.0" cy="146.0" r="3.2" style="fill:var(--bf-vz-4);stroke:var(--bf-surface);stroke-width:1.2"><title>Markdown, 10 workers: 1.05×</title></circle><text class="bf-tm" x="696.0" y="338.0" text-anchor="end" style="font-size:11px">elapsed time relative to 4 workers (lower is faster; enlarged dot marks the knee)</text></svg>
<figcaption>Intel Core i9-10900K, nine interleaved rounds per worker count, each corpus normalised
to its four-worker time (lower is faster). Left: pre-registered nine-round medians, including the
frequency-ramp split on Markdown. Right: fastest of nine rounds, which isolates full-clock runs and
shows Markdown already at its floor at four workers while Wikidata drops to 0.52× (183 ms, 4.58 GB/s)
at nine workers.</figcaption>
</figure>

The right-hand panel isolates full-clock runs by taking the fastest of the nine rounds at each
worker count. Every single corpus then lines up with its single-thread station balance. Wikidata
drops smoothly from 355.5 ms at four workers to 183.2 ms at nine, reaching 4.58 GB/s before ticking
up at ten. Rust source hits its floor at five workers (167.4 ms, 1.09 GB/s). JSON sits flat from
four to six workers (184.6 ms, 1.03 GB/s) before climbing. Markdown is already at its fastest at
four workers (187.5 ms, 0.98 GB/s) and drifts gently upward from there.

## 3. Why a faster scanner disappears on Wikidata at eight threads

Once you know where the balance point sits, a puzzle from mibz2 0.5.15 makes immediate sense.

Version 0.5.15 shipped a redesigned Huffman scanner in the default decoder. Normally a bzip2
decoder switches Huffman tables every 50 symbols. The new scanner coalesces consecutive groups that
reuse the same table into one run. It also packs the primary lookup table into a compact
byte-aligned layout and skips redundant zero-initialisation of scratch arrays. Finally, it
dispatches BMI2 bit-shift instructions once per block rather than checking inside the hot loop.

On the ten-core workstation, isolated phase timings confirmed a clean 10.5% cut in serial scanning
time. Across four copies of Wikidata, scanning fell from 160 ms to 143 ms with non-overlapping
ranges over ten paired rounds. In a construction-free test where workers are artificially fast,
that scanner win translated directly into 7.8% faster eight-thread decoding and 5.8% faster
ten-thread decoding.

Yet in the real decoder on the very same machine, the wall-clock picture was much subtler:

- On four copies of Wikidata, eight-thread decoding sped up by 1.9% and ten-thread decoding by
  4.0%. Thirty guard measurements across six other corpora showed zero regressions.
- On the single-copy Wikidata release check comparing 0.5.15 against 0.5.14, single-thread decoding
  ran 5.7% faster and two-thread decoding 3.4% faster. Elapsed time at four, eight and ten threads
  stayed flat within 0.4%. At twelve unpinned threads a 9.5% gain re-emerged.

Nothing went wrong with the scanner. On Wikidata, $k^*$ is about 9.5. At four or eight threads, the
workers still take longer than the serial scanner. Shaving 10.5% off the scanner therefore just
gives that thread more idle margin while it waits for the admission window. At one and two threads,
every saved microsecond comes straight off the active core's total work. At ten threads, workers
finally catch up to the scanner and the speed-up shows through again. On ordinary text, where $k^*$
is four to five, the scanner already limits throughput well before eight cores.

## 4. Two more shipped improvements in 0.5.16

Two further default-path improvements shipped in mibz2 0.5.16. Both target real workloads that a
Wikidata-only benchmark never exercises.

**Wider branchless run decoding.** When a bzip2 block compresses well, its decoded symbol stream
consists largely of run-length tokens rather than dense literal bytes. mibz2's branchless run
front previously accepted a block only up to 50% of the block length bound. A census across
fourteen corpora found forty compressible blocks sitting just above that line, between 50% and 75%
of the bound. Those blocks came from Markdown, ELF executables, JSON, prose and Python source, and
still held 17% to 49% run symbols.

Version 0.5.16 raises the admission ceiling to 75% of the bound. In the newly widened band, a block
must also carry at least 12.5% run symbols. That guard keeps short, small-alphabet random streams
(near 3.8% runs) on the column path where they belong. Isolated virtual-machine benchmarks across
three compiler profiles measured clear cuts in move-to-front and inverse-BWT processor time.
Markdown ran 13.2% to 32.4% faster, ELF binaries 14.5% to 24.7% faster and JSON 9.0% to 9.9%
faster. Not a single Wikidata, random, records or pattern block changed route. On the ten-core
workstation's 0.5.16 release check, Wikidata elapsed times stayed flat from one to twenty threads,
matching the prediction.

**Chunked whole-file reads in Python.** Profiling the Python bindings turned up an unexpected cost
on the classic `mibz2.open(path).read()` path. Because `BZ2Reader` inherited `RawIOBase.readall()`,
a whole-file read looped through `readinto()` in 8 KiB pieces. Reading one 210 MB file took 25,829
calls across the Python–Rust boundary.

Version 0.5.16 overrides whole-file `readall()` and `read()` in `BZ2Reader` to pull 1 MiB chunks
directly from the Rust reader and join them once. On a virtual machine decoding Wikidata,
`mibz2.open(...).read()` used 6.3% to 6.8% less processor time and finished 5.7% to 12.5% faster.
Direct `BZ2Reader.read()` calls cut processor time by 8.5% and wall time by 20.0%, at 0.99× peak
memory. Because the change lives purely in Python above the existing reader, the single stable-ABI
wheel for Python 3.10 and newer stays unchanged.

## 5. What this means for decoding in practice

Two practical rules fall straight out of these measurements.

First, throwing every core on a machine at a parallel bzip2 decoder only helps when the file
compresses tightly. Highly repetitive dumps (15× to 20× compression) keep eight to ten workers fed.
Ordinary source trees, JSON exports and documentation (4× to 6× compression) saturate the serial
Huffman scanner around four to six workers. Beyond that point, extra worker threads burn processor
time and cache bandwidth for slower wall-clock throughput.

Second, a single benchmark corpus can hide half the optimisation surface. On Wikidata, worker speed
dominates up to eight cores. That masks scanner improvements while leaving ordinary text's run-path
threshold untouched. Measuring station balance alongside full-file scaling exposes both sides of
the pipeline. All three software changes described here are live by default in mibz2 0.5.16 on
crates.io and PyPI.

---

**Sources:**

- Physical ten-core text-panel scaling sweep and station balance: [registration and recipe](https://github.com/lmmx/mibz2/blob/af7ff4829277a1687c692c5eb8c2f9f5208958c4/docs/journal/2026-11-27-text-knee-i9/registration.md) ([#1135](https://github.com/lmmx/mibz2/pull/1135), [#1139](https://github.com/lmmx/mibz2/pull/1139)), archived Intel Core i9-10900K bundle `origin/evidence` `5e3f28cc` (`text-knee/run-20261127T172728Z.tar.gz`).
- Four-corpus single-thread station balance on the same workstation: archived bundle `origin/evidence` `77236472` (`jcc-aligned-release/run-20261127T135104Z.tar.gz`) and [verdict record](https://github.com/lmmx/mibz2/pull/1172) ([#1172](https://github.com/lmmx/mibz2/pull/1172)).
- Eleven-corpus virtual-machine station-balance survey and bzip2-vs-lbzip2 control: [research note](https://github.com/lmmx/mibz2/blob/c5c34377/docs/journal/2026-11-27-explore/station-balance-off-wikidata.md).
- Coalesced Huffman scanner in 0.5.15: [#1034](https://github.com/lmmx/mibz2/pull/1034), i9-10900K hardware verdict ([#1151](https://github.com/lmmx/mibz2/pull/1151), `origin/evidence` `9de7b6e0`), and 0.5.15 release scaling (`origin/evidence` `bce3914d`, [release record](https://github.com/lmmx/mibz2/blob/c5c34377/frontier/deltas/2026-11-27-release-0-5-15-done.md)).
- Wider branchless run admission in 0.5.16: [#1175](https://github.com/lmmx/mibz2/pull/1175), [dissection note](https://github.com/lmmx/mibz2/blob/7cbbb78d/docs/journal/2026-11-27-explore/runs-takes-runs-post-1080.md), and 0.5.16 workstation scaling check ([#1199](https://github.com/lmmx/mibz2/pull/1199), `origin/evidence` `4710ef99`).
- Chunked whole-file Python reads in 0.5.16: [#1176](https://github.com/lmmx/mibz2/pull/1176) and [delta](https://github.com/lmmx/mibz2/blob/9001629a/frontier/deltas/2026-11-27-python-readall-abi3.md).
