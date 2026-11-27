---
title: "Half the memory on bzip2's most compressible files"
image: assets/social/blog-2026-11-27-bounded-memory-on-highly-compressible-input.png
description: "Workers that pause rather than race far ahead of the reader now let mibz2 unpack a tiny file of runs in half the memory, and sooner as well."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Half the memory on bzip2's most compressible files

Workers that pause rather than race far ahead of the reader now let mibz2 unpack a tiny file of runs in half the memory, and sooner as well.
{: .post-dek }

27 November 2026 · 9 min read · [Research synthesis](#5-what-it-means-for-a-caller)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 decodes a bzip2 file the way [lbzip2](https://github.com/kjn/lbzip2) does, many blocks at a
time. One thread, the scanner, reads each compressed block's
[Huffman codes](2026-11-05-huffman-bit-buffer.md) in order. A pool of workers rebuilds the blocks'
text in parallel, and a reader hands the finished blocks to the caller in sequence
([the decoder, drawn as stations](2026-11-25-the-decoder-as-a-queueing-network.md#4-the-decoder-drawn-as-stations)).
Between the scanner and the reader sits an admission window, a fixed number of blocks allowed in
flight at once. Beside it sits a cap of 12 MiB on decoded output that the caller has not yet read.
Together they are meant to keep the decoder's memory small whatever file it is given.

For ordinary files they do. A block of JSON or text decodes to about a megabyte, so a window of
thirty-two blocks holds a few tens of megabytes. But the size of a compressed block says very
little about the size of what it decodes to. Two gigabytes of zeros compress to a file of 1,522
bytes. Decoding that file with the library's default settings for eight threads once took 1.27 GB
of memory. The cap on unread output was there all along; it was simply checked at a moment when it
could not see the danger.

That has now been fixed. Workers consult the cap themselves before they start a block, and the
pool of spare output buffers is bounded in bytes rather than in buffers. Take the same file, four
workers and a thirty-two-block window: peak memory fell from 1,118 MiB to 545 MiB, a cut of 51%.
The decode also finished about 15% sooner. More to the point, memory on such a file no longer
grows with the window at all. It is the same 545 MiB with eight, sixteen or thirty-two blocks in
flight.

**Takeaways**

- **A tiny file can decode to a vast one.** bzip2's first stage writes a run of up to 259 equal
  bytes as five. One block of a single repeated byte therefore decodes to about 46.6 MB, from a
  few dozen compressed bytes.
- **The old cap watched the wrong moment.** It was checked only when the scanner admitted a block.
  Blocks of runs are scanned in microseconds, so the whole window was admitted before any output
  existed. The workers then decoded every admitted block in full.
- **Now a worker waits instead.** While the unread output is over the cap, no worker starts a
  block more than the worker count plus two places ahead of the reader. A bounded pool stops spare
  46 MB buffers from piling up behind it.
- **Memory now follows the workers, not the window.** Four workers need a flat 545 MiB for any
  window from eight to thirty-two blocks. They had needed 589 to 1,118 MiB. Eight workers need
  955 MiB, down from 1,528, and ordinary files decode no slower.
- **It is in the default decoder.** The change is on master, in every decoder the library builds
  by default, and will ship in the next release.

## 1. How can 1,522 bytes become two gigabytes?

bzip2 compresses in stages, and two of them encode runs. The first is applied to the raw input
before anything else. It replaces any run of four to 259 identical bytes by four copies of the
byte, followed by a count. A block at the strongest setting holds at most 900,000 bytes *after*
this stage. If all of them encode maximal runs, the block holds 180,000 runs of 259 bytes, and it
decodes to

$$
\frac{900{,}000}{5} \times 259 = 46{,}620{,}000 \text{ bytes} \approx 46.6 \text{ MB}.
$$

The remaining stages then make that block almost free to store. The
[Burrows–Wheeler transform](2026-11-08-bwt-runs-from-mtf-codes.md) of so regular a text is itself
a handful of long runs, and move-to-front turns them into zeros. The second run-length stage
writes a run of zeros by the binary digits of its length, so the number of symbols grows only with
the logarithm of the run. The whole block's coded symbols fit in a few dozen bytes. Two gigabytes
of zeros make about 46 such blocks, held in a file of 1,522 bytes: an expansion of well over a
million to one.

<aside class="il-gloss" markdown>**Two run-length stages.** bzip2 encodes runs twice. The first
stage, on the raw input, exists to protect the block sort from long runs; the second, after
move-to-front, compresses the zeros that move-to-front produces. The first stage is what fixes a
block's decoded size: at most 900,000 bytes go in, and up to about 52 times as many can come
out.</aside>

None of this is an attack peculiar to mibz2. Any input dominated by long runs behaves this way: a
sparse disk image, a zero-filled log, or a deliberately constructed
[decompression bomb](https://en.wikipedia.org/wiki/Zip_bomb). A decoder that holds several blocks
at once has to be ready for each of them to be 46 MB rather than one.

## 2. Why did the queue fill with giant blocks?

The window counts blocks, not bytes. Library callers get four blocks in flight per thread, so
thirty-two at eight threads. The byte cap was meant to cover the rest. Once 12 MiB of decoded
output was waiting for the reader, the scanner would admit blocks only up to the worker count,
plus two, ahead of the reader.

The flaw was in *when* that rule was applied. The scanner checked the cap only at the moment it
admitted a block, and only the scanner checked it. On a file of runs the scanner finishes each
block in microseconds, long before any worker has finished decoding the first one. When it
admitted the second block, and the tenth, and the thirty-second, no output existed yet, so the
cap's test passed every time. The window filled entirely with admitted blocks, and the workers
then decoded every one of them, each into a buffer of its own of about 46 MB. A pool kept up to
one returned buffer per slot for reuse, so that memory stayed held until the decoder was dropped.

A probe on a cloud machine with four virtual CPUs measured the consequence. With four workers, peak memory rose
with the window, from 457 MiB with four slots to 943 MiB with thirty-two. At the default window
for eight threads it reached 1,273 MiB. At sixteen threads it reached 1,818 MiB, limited only
because the file had run out of blocks. Ordinary text, at comparable settings, used between 99
and 148 MiB. The extra memory also cost time, because the kernel zeroes every new page as it is
first touched. With four workers, the sixteen- and thirty-two-block windows ran 24% to 28% slower
than the eight-block one.

No earlier test had seen this. The most compressible sample in the benchmark set decodes to
20 MB, less than a single block of runs.

<figure class="bf-fig" id="window">
<p class="bf-title">Which blocks in the window may be decoded, on a file of runs</p>
<!-- fig:window -->
<svg viewBox="0 0 720 300" role="img" aria-label="Schematic of a 32-block window with four workers on a file of runs only. Before the fix every slot in the window may end up holding a decoded block of about 46 MB. After it, only the six blocks nearest the reader, four workers plus two, may be decoded while the unread output is over its cap; the rest wait as scanned blocks, holding only their symbols."><text class="bf-tm" x="120" y="18" text-anchor="start" style="font-size:11.5px">reader takes the oldest block here</text><path d="M126,24 L126,34" style="stroke:var(--bf-frame);stroke-width:1.5"/><text class="bf-t-strong" x="16" y="76" text-anchor="start" style="font-size:13px">Before</text><text class="bf-tm" x="16" y="94" text-anchor="start" style="font-size:11.5px">any slot may</text><rect x="120" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 1: a decoded block, about 46 MB</title></rect><rect x="138" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 2: a decoded block, about 46 MB</title></rect><rect x="156" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 3: a decoded block, about 46 MB</title></rect><rect x="174" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 4: a decoded block, about 46 MB</title></rect><rect x="192" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 5: a decoded block, about 46 MB</title></rect><rect x="210" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 6: a decoded block, about 46 MB</title></rect><rect x="228" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 7: a decoded block, about 46 MB</title></rect><rect x="246" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 8: a decoded block, about 46 MB</title></rect><rect x="264" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 9: a decoded block, about 46 MB</title></rect><rect x="282" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 10: a decoded block, about 46 MB</title></rect><rect x="300" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 11: a decoded block, about 46 MB</title></rect><rect x="318" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 12: a decoded block, about 46 MB</title></rect><rect x="336" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 13: a decoded block, about 46 MB</title></rect><rect x="354" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 14: a decoded block, about 46 MB</title></rect><rect x="372" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 15: a decoded block, about 46 MB</title></rect><rect x="390" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 16: a decoded block, about 46 MB</title></rect><rect x="408" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 17: a decoded block, about 46 MB</title></rect><rect x="426" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 18: a decoded block, about 46 MB</title></rect><rect x="444" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 19: a decoded block, about 46 MB</title></rect><rect x="462" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 20: a decoded block, about 46 MB</title></rect><rect x="480" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 21: a decoded block, about 46 MB</title></rect><rect x="498" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 22: a decoded block, about 46 MB</title></rect><rect x="516" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 23: a decoded block, about 46 MB</title></rect><rect x="534" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 24: a decoded block, about 46 MB</title></rect><rect x="552" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 25: a decoded block, about 46 MB</title></rect><rect x="570" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 26: a decoded block, about 46 MB</title></rect><rect x="588" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 27: a decoded block, about 46 MB</title></rect><rect x="606" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 28: a decoded block, about 46 MB</title></rect><rect x="624" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 29: a decoded block, about 46 MB</title></rect><rect x="642" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 30: a decoded block, about 46 MB</title></rect><rect x="660" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 31: a decoded block, about 46 MB</title></rect><rect x="678" y="50" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 32: a decoded block, about 46 MB</title></rect><line x1="116" y1="120" x2="696" y2="120" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="16" y="108" text-anchor="start" style="font-size:11.5px">hold one</text><text class="bf-t-strong" x="16" y="206" text-anchor="start" style="font-size:13px">After</text><text class="bf-tm" x="16" y="224" text-anchor="start" style="font-size:11.5px">only the first six</text><rect x="120" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 1: a decoded block, about 46 MB</title></rect><rect x="138" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 2: a decoded block, about 46 MB</title></rect><rect x="156" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 3: a decoded block, about 46 MB</title></rect><rect x="174" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 4: a decoded block, about 46 MB</title></rect><rect x="192" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 5: a decoded block, about 46 MB</title></rect><rect x="210" y="180" width="16" height="70" rx="2" style="fill:var(--bf-vz-2)"><title>slot 6: a decoded block, about 46 MB</title></rect><rect x="228" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 7: scanned, waiting: its symbols only</title></rect><rect x="246" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 8: scanned, waiting: its symbols only</title></rect><rect x="264" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 9: scanned, waiting: its symbols only</title></rect><rect x="282" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 10: scanned, waiting: its symbols only</title></rect><rect x="300" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 11: scanned, waiting: its symbols only</title></rect><rect x="318" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 12: scanned, waiting: its symbols only</title></rect><rect x="336" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 13: scanned, waiting: its symbols only</title></rect><rect x="354" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 14: scanned, waiting: its symbols only</title></rect><rect x="372" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 15: scanned, waiting: its symbols only</title></rect><rect x="390" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 16: scanned, waiting: its symbols only</title></rect><rect x="408" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 17: scanned, waiting: its symbols only</title></rect><rect x="426" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 18: scanned, waiting: its symbols only</title></rect><rect x="444" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 19: scanned, waiting: its symbols only</title></rect><rect x="462" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 20: scanned, waiting: its symbols only</title></rect><rect x="480" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 21: scanned, waiting: its symbols only</title></rect><rect x="498" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 22: scanned, waiting: its symbols only</title></rect><rect x="516" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 23: scanned, waiting: its symbols only</title></rect><rect x="534" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 24: scanned, waiting: its symbols only</title></rect><rect x="552" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 25: scanned, waiting: its symbols only</title></rect><rect x="570" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 26: scanned, waiting: its symbols only</title></rect><rect x="588" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 27: scanned, waiting: its symbols only</title></rect><rect x="606" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 28: scanned, waiting: its symbols only</title></rect><rect x="624" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 29: scanned, waiting: its symbols only</title></rect><rect x="642" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 30: scanned, waiting: its symbols only</title></rect><rect x="660" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 31: scanned, waiting: its symbols only</title></rect><rect x="678" y="242" width="16" height="8" rx="2" style="fill:var(--bf-vz-1)"><title>slot 32: scanned, waiting: its symbols only</title></rect><line x1="116" y1="250" x2="696" y2="250" style="stroke:var(--bf-vz-axis);stroke-width:1"/><text class="bf-tm" x="16" y="238" text-anchor="start" style="font-size:11.5px">may be decoded</text><path d="M227.0,160 L227.0,258" style="stroke:var(--bf-ink);stroke-width:1.5;stroke-dasharray:4 3"/><text class="bf-tm" x="235.0" y="176" text-anchor="start" style="font-size:11.5px">four workers + two: decoding goes no further</text><text class="bf-tm" x="235.0" y="192" text-anchor="start" style="font-size:11.5px">while the unread output is over the cap</text><rect x="120" y="274" width="14" height="14" rx="2" style="fill:var(--bf-vz-2)"/><text class="bf-tm" x="140" y="286" text-anchor="start" style="font-size:11.5px">decoded block, about 46 MB</text><rect x="350" y="280" width="14" height="8" rx="2" style="fill:var(--bf-vz-1)"/><text class="bf-tm" x="370" y="286" text-anchor="start" style="font-size:11.5px">scanned block, waiting: its symbols only</text></svg>
<figcaption>Schematic, four workers and a window of thirty-two blocks. Before the fix, nothing
stopped any admitted block from being decoded, and each decoded block of runs is about 46 MB.
After it, while the unread output is over its 12 MiB cap, only the blocks within six places of
the reader may be decoded; the rest wait, holding only their scanned symbols.</figcaption>
</figure>

## 3. How does the fix bound the memory?

It has two parts, both in the code that coordinates the scanner, the workers and the reader.

**The workers check the cap.** Before a worker starts decoding a block, it reads the total of
decoded bytes still waiting for the reader. That total is kept in an atomic counter beside the
shared state. On an ordinary file the total never reaches 12 MiB, so the check is a single memory
read and takes no lock. Only at or over the cap does the worker look further. If its block lies
more than the worker count plus two places ahead of the block the reader needs next, it sleeps.
The reader wakes the sleeping workers each time it takes a block, and each re-checks its own
position. Cancelling the decode wakes them too, so none is left waiting on a decode that has
ended.

Waiting threads invite deadlock, and this design avoids it by construction. Blocks are handed to
workers strictly in order. The block the reader needs next, and the few after it, were therefore
handed out before any block far enough ahead to be held back, and they are never made to wait.
They finish, the reader takes them, and its progress releases the workers behind.

**The spare-buffer pool is bounded in bytes.** The pool used to keep up to one buffer per slot. It
now also stops accepting buffers once their total capacity passes the cap plus 2 MiB for each
block that may be in flight: 24 MiB at four workers. Even past that, it keeps up to one buffer per
worker. That exception matters. Without it, every block of runs would free its 46 MB buffer and
allocate a fresh one, paying the kernel's page faults again each time.

Taken together, the two parts change what the memory on a file of runs is proportional to. Before,
it was the window; now it is about two decoded blocks for each worker, whatever the window's size.
The change adds no unsafe code. One new test drives a stream of high-expansion blocks through a
decoder whose cap is forced to bind, and checks the peak. Another cancels a decode while workers
wait behind the cap, and checks that they wake and exit.

## 4. What did it measure?

The change was measured on its author's development machine, against the master branch it was
built on. Each setting decoded the 1,522-byte file of zeros three times, interleaved with the old
build, in one session.

| Workers | Window (blocks) | Peak memory before → after | Change | Elapsed time before → after |
|---:|---:|---:|---:|---:|
| 4 | 8 | 589 → 545 MiB | 7% less | 1.27 → 1.27 s |
| 4 | 16 | 811 → 545 MiB | 33% less | 1.44 → 1.35 s |
| 4 | 32 | 1,118 → 545 MiB | 51% less | 1.38 → 1.18 s |
| 8 | 32 | 1,528 → 955 MiB | 37% less | 2.03 → 1.60 s |

<figure class="bf-fig" id="peak">
<p class="bf-title">Peak memory decoding 2 GiB of zeros from a 1,522-byte file</p>
<!-- fig:peak -->
<svg viewBox="0 0 720 300" role="img" aria-label="Peak memory decoding 2 GiB of zeros compressed to 1,522 bytes, before and after the fix. Four workers with 8 slots: 589 to 545 MiB. With 16 slots: 811 to 545. With 32 slots: 1,118 to 545. Eight workers with 32 slots: 1,528 to 955."><line x1="170.0" y1="28" x2="170.0" y2="252" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="170.0" y="268" text-anchor="middle" style="font-size:11.5px">0</text><line x1="297.5" y1="28" x2="297.5" y2="252" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="297.5" y="268" text-anchor="middle" style="font-size:11.5px">400</text><line x1="425.0" y1="28" x2="425.0" y2="252" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="425.0" y="268" text-anchor="middle" style="font-size:11.5px">800</text><line x1="552.5" y1="28" x2="552.5" y2="252" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="552.5" y="268" text-anchor="middle" style="font-size:11.5px">1,200</text><line x1="680.0" y1="28" x2="680.0" y2="252" style="stroke:var(--bf-vz-grid);stroke-width:1"/><text class="bf-tm" x="680.0" y="268" text-anchor="middle" style="font-size:11.5px">1,600</text><text class="bf-tm" x="425.0" y="288" text-anchor="middle" style="font-size:11.5px">peak memory (MiB)</text><text class="bf-t" x="160" y="65" text-anchor="end" style="font-size:12.5px">4 workers, 8 slots</text><rect x="170" y="40" width="187.7" height="18" rx="2" style="fill:var(--bf-vz-mid)"><title>4 workers, 8 slots, before: 588.9 MiB</title></rect><text class="bf-tm" x="363.7" y="54" text-anchor="start" style="font-size:12px">589</text><rect x="170" y="62" width="173.8" height="18" rx="2" style="fill:var(--bf-vz-1)"><title>4 workers, 8 slots, after: 545.2 MiB</title></rect><text class="bf-t-strong" x="349.8" y="76" text-anchor="start" style="font-size:12px">545</text><text class="bf-t" x="160" y="121" text-anchor="end" style="font-size:12.5px">4 workers, 16 slots</text><rect x="170" y="96" width="258.4" height="18" rx="2" style="fill:var(--bf-vz-mid)"><title>4 workers, 16 slots, before: 810.6 MiB</title></rect><text class="bf-tm" x="434.4" y="110" text-anchor="start" style="font-size:12px">811</text><rect x="170" y="118" width="173.8" height="18" rx="2" style="fill:var(--bf-vz-1)"><title>4 workers, 16 slots, after: 545.2 MiB</title></rect><text class="bf-t-strong" x="349.8" y="132" text-anchor="start" style="font-size:12px">545</text><text class="bf-t" x="160" y="177" text-anchor="end" style="font-size:12.5px">4 workers, 32 slots</text><rect x="170" y="152" width="356.3" height="18" rx="2" style="fill:var(--bf-vz-mid)"><title>4 workers, 32 slots, before: 1,117.9 MiB</title></rect><text class="bf-tm" x="532.3" y="166" text-anchor="start" style="font-size:12px">1,118</text><rect x="170" y="174" width="173.8" height="18" rx="2" style="fill:var(--bf-vz-1)"><title>4 workers, 32 slots, after: 545.4 MiB</title></rect><text class="bf-t-strong" x="349.8" y="188" text-anchor="start" style="font-size:12px">545</text><text class="bf-t" x="160" y="233" text-anchor="end" style="font-size:12.5px">8 workers, 32 slots</text><rect x="170" y="208" width="487.0" height="18" rx="2" style="fill:var(--bf-vz-mid)"><title>8 workers, 32 slots, before: 1,528.0 MiB</title></rect><text class="bf-tm" x="663.0" y="222" text-anchor="start" style="font-size:12px">1,528</text><rect x="170" y="230" width="304.4" height="18" rx="2" style="fill:var(--bf-vz-1)"><title>8 workers, 32 slots, after: 955.1 MiB</title></rect><text class="bf-t-strong" x="480.4" y="244" text-anchor="start" style="font-size:12px">955</text><line x1="170" y1="28" x2="170" y2="252" style="stroke:var(--bf-vz-axis);stroke-width:1"/><rect x="170" y="8" width="14" height="12" rx="2" style="fill:var(--bf-vz-mid)"/><text class="bf-tm" x="190" y="18" text-anchor="start" style="font-size:11.5px">before</text><rect x="250" y="8" width="14" height="12" rx="2" style="fill:var(--bf-vz-1)"/><text class="bf-tm" x="270" y="18" text-anchor="start" style="font-size:11.5px">after</text></svg>
<figcaption>Before and after the fix, on the same machine in one session, three interleaved repeats
of each setting. With four workers, memory no longer depends on the window. The machine differs
from the cloud probe of §2, so the two sets of "before" figures are not directly
comparable.</figcaption>
</figure>

With four workers and thirty-two slots, the decode took half as many page faults as before, about
140,000 instead of 286,000. It also used 23% less processor time. Fewer fresh pages for the kernel
to zero is the likely source of most of the 15% saving in elapsed time. At eight workers the
elapsed time fell by 21%.

Ordinary input had to decode no slower, a condition fixed before any measurement. Text and a
sample of the Wikidata dump were decoded at one and four workers, seven times each, and every 95%
interval included no change. The continuous-integration runner, a Xeon with four CPUs, then ran ten
paired rounds over the benchmark corpora. At four threads it found the Wikidata sample 2.0%
faster, with a 95% interval from 0.3% to 3.7%. The rest were within noise but for one small cell,
random data at four threads, which came out 5.2% slower. That file's whole output is 8 MB, under
the 12 MiB cap, so neither new check can act on it.

## 5. What it means for a caller

A caller of the parallel decoder, from Rust or from Python, chooses how many threads to use. That
choice also sets how many blocks may be in flight. Until now, it also set the worst-case memory on
a highly compressible file. A 1,522-byte input could claim more than a gigabyte at an everyday
setting, and more again with more threads. A file of that kind no longer costs memory in
proportion to the queue. It costs about two blocks per worker. A wider window can be chosen for
throughput without enlarging the worst case, and the decode no longer slows itself by faulting in
memory nobody needs.

The bound is not the 12 MiB of the cap itself. A block that is being decoded must have somewhere
to go. Each block within the worker count plus two of the reader may still hold a full decoded
block of runs, and each worker may keep a spare buffer; 545 MiB at four workers is that floor. A
decoder that wanted to go lower would have to decode a block of runs in pieces, which is a
different design.

The fix is on the master branch and will ship in the next release. It applies to every decoder
the library builds by default: the pool of workers that each take a whole block, and the teams
that share one. It changes nothing about their output. The test suite, a fuzzing run and checksums
of the decoded files all show the same bytes, and the same errors, as before.

---

**Sources:**

- The fix: [pull request #1129](https://github.com/lmmx/mibz2/pull/1129), merged as
  [1429f54f](https://github.com/lmmx/mibz2/commit/1429f54f74fa718d06d9ecda83f3a876fc9716a9); its
  measurements in [the delta record](https://github.com/lmmx/mibz2/blob/1429f54f74fa718d06d9ecda83f3a876fc9716a9/frontier/deltas/2026-11-27-ready-cap-worker-gate.md);
  the question and its stop test, frozen before data, in
  [the claim](https://github.com/lmmx/mibz2/blob/1429f54f74fa718d06d9ecda83f3a876fc9716a9/frontier/claims/ready-cap-worker-gate.toml),
  admitted in [pull request #1121](https://github.com/lmmx/mibz2/pull/1121); the code in
  [src/parallel.rs](https://github.com/lmmx/mibz2/blob/1429f54f74fa718d06d9ecda83f3a876fc9716a9/src/parallel.rs)
  and its tests in [src/parallel/plateau_tests.rs](https://github.com/lmmx/mibz2/blob/1429f54f74fa718d06d9ecda83f3a876fc9716a9/src/parallel/plateau_tests.rs).
- The continuous-integration benchmark: the paired run
  [45963424147](https://github.com/lmmx/mibz2/actions/runs/45963424147), head a1d9b317 against
  base 30479058, Intel Xeon 6973P-C, four CPUs, reported on the pull request.
- The problem: [the memory probe](https://github.com/lmmx/mibz2/blob/0b1c0813c1c43a4c0c538103e317dbde3ac5fec8/docs/journal/2026-11-27-explore/expansive-block-memory.md)
  ([pull request #1094](https://github.com/lmmx/mibz2/pull/1094); a four-vCPU Xeon cloud machine),
  and the [synthesis that raised its priority](https://github.com/lmmx/mibz2/blob/583e9b83293997aa9bcf24945c15fe9f7938ffb8/docs/journal/2026-11-27-synthesis/eleventh-pass.md)
  ([pull request #1103](https://github.com/lmmx/mibz2/pull/1103)).
- The pipeline and its admission rule: [The decoder as a queueing network](2026-11-25-the-decoder-as-a-queueing-network.md).
