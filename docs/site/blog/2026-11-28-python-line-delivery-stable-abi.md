---
title: "Twenty-six times faster line iteration in Python without leaving the Stable ABI"
image: assets/social/blog-2026-11-28-python-line-delivery-stable-abi.png
description: "Bypassing Python's redundant buffering layer and scanning newlines in safe Rust speeds up line reads up to twenty-six times while keeping one universal abi3 wheel."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Twenty-six times faster line iteration in Python without leaving the Stable ABI

Bypassing Python's redundant buffering layer and scanning newlines in safe Rust speeds up line reads up to twenty-six times while keeping one universal abi3 wheel.
{: .post-dek }

28 November 2026 · 7 min read · [Research result](#5-where-the-stable-abi-boundary-really-sits)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

When a Rust decoder produces gigabytes per second, the slowest step in Python is often crossing the language boundary. Until recently, mibz2's Python package had optimised whole-file reads. Yet iterating line by line through a compressed log or JSON dump still paid for layers of CPython buffering that repeated work on every call.

We dissected every streaming entry point in mibz2's Python layer. That audit covered raw reader iteration, reading all lines, chunked reads, buffer fills, and whole-stream decompression. Removing extra copies and scanning newlines in safe Rust sped up raw line iteration by 26.1 times. Reading all lines into a list dropped 28%, and line-by-line iteration dropped 13 to 14%. On Wikidata, 64 KiB streaming reads ran 9% faster with 12% less peak memory. Every change stays inside safe Rust and CPython's Stable ABI, keeping one universal wheel across Python 3.10 through 3.15.

The same audit mapped the exact wall that safe Stable ABI code cannot cross on whole-stream reads. There, PyO3's zero-initialisation and glibc's 32 MiB anonymous mapping threshold combine to hold peak memory near twice the decoded output size.

**Takeaways**

- **Raw reader line iteration is 26.1 times faster.** Overriding line iteration directly in Rust
  eliminates Python's default raw-stream fallback, which previously requested one byte at a time.
- **Skipping Python's redundant buffer removes two copies per read.** mibz2's Rust reader already
  decodes into block buffers up to 900 KiB. Stacking a 1 MiB CPython buffer on top copied every
  byte three times instead of once.
- **Direct C-method binding removes Python frame overhead.** Binding Rust line methods directly onto
  the file object kept full buffered-reader compatibility while turning a 66% wrapper slowdown into
  a 4% speedup.
- **Safe eight-byte newline scanning avoids per-line GIL churn.** While bytes remain in the current
  decoded block, Rust scans eight bytes at a time without dropping the Global Interpreter Lock.
- **Whole-stream peak memory hits two external floors.** Safe PyO3 zero-fills a full Stable ABI
  output buffer before copying into it. Meanwhile, glibc keeps 1 MiB chunks on the process heap
  unless chunks exceed 32 MiB.

## 1. Two hidden costs in Python's standard I/O stack

Python's standard I/O module was designed for unbuffered operating-system file descriptors. A
minimal raw stream implements one buffer-fill method, and callers wrap it in a buffered reader so
small reads hit memory instead of the kernel.

That layering breaks down in two ways when the underlying stream is already a block-parallel Rust
decoder.

First, a raw stream that does not override line reading inherits Python's generic fallback. That
fallback reads one byte at a time until it sees a newline. On a 1 MiB text slice holding 26,301
lines, raw iteration crossed into Rust over a million times. It spent nearly a full second of CPU
time to read a single megabyte.

Second, even callers using mibz2's buffered opener paid for the same buffering twice. mibz2's Rust
reader already holds the current decoded bzip2 block in memory (up to 900 KiB) and lends a direct
slice of its unread bytes. Stacking a 1 MiB CPython buffered reader on top copied every byte three
times. First, bytes moved from the Rust block into a temporary Python bytes object. Next, they moved
into the 1 MiB C buffer, and finally into the returned line object.

<figure class="bf-fig" id="copy-chain">
<p class="bf-title">From three copies to one: streaming lines out of a block-buffered Rust reader</p>
<!-- fig:copy-chain -->
<svg viewBox="0 0 720 246" role="img" aria-label="Architecture diagram comparing Python streaming before and after direct Rust delivery. Before: Rust 900 KiB decoded block copies into temporary PyBytes (copy 1), then into io.BufferedReader 1 MiB buffer (copy 2), then into the returned line PyBytes (copy 3), while raw BZ2Reader fell back to 1-byte reads. After: Rust 900 KiB decoded block runs a safe 8-byte SWAR newline scan in place with no GIL release within the block and performs 1 direct copy straight into the returned line PyBytes.">
<text class="bf-t-strong" x="16" y="24" style="font-size:12px">Before: BufferedReader stacked over block-buffered Rust (3 copies per line)</text>
<rect x="16" y="36" width="142" height="46" rx="6" style="fill:var(--bf-amber-tint);stroke:var(--bf-vz-2);stroke-width:1.5"/>
<text class="bf-t-strong" x="87" y="56" text-anchor="middle" style="font-size:11px">Rust block buffer</text>
<text class="bf-tm" x="87" y="72" text-anchor="middle" style="font-size:10px">up to 900 KiB ready</text>
<line x1="158" y1="59" x2="196" y2="59" style="stroke:var(--bf-vz-1);stroke-width:1.8"/>
<polygon points="196,55 204,59 196,63" style="fill:var(--bf-vz-1)"/>
<text class="bf-tm" x="180" y="51" text-anchor="middle" style="font-size:9.5px">copy 1</text>
<rect x="206" y="36" width="136" height="46" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-vz-1);stroke-width:1.5"/>
<text class="bf-t-strong" x="274" y="56" text-anchor="middle" style="font-size:11px">Temp PyBytes</text>
<text class="bf-tm" x="274" y="72" text-anchor="middle" style="font-size:10px">per raw readinto()</text>
<line x1="342" y1="59" x2="380" y2="59" style="stroke:var(--bf-vz-1);stroke-width:1.8"/>
<polygon points="380,55 388,59 380,63" style="fill:var(--bf-vz-1)"/>
<text class="bf-tm" x="364" y="51" text-anchor="middle" style="font-size:9.5px">copy 2</text>
<rect x="390" y="36" width="150" height="46" rx="6" style="fill:var(--bf-blue-tint);stroke:var(--bf-vz-1);stroke-width:1.5"/>
<text class="bf-t-strong" x="465" y="56" text-anchor="middle" style="font-size:11px">io.BufferedReader</text>
<text class="bf-tm" x="465" y="72" text-anchor="middle" style="font-size:10px">extra 1 MiB C buffer</text>
<line x1="540" y1="59" x2="578" y2="59" style="stroke:var(--bf-vz-1);stroke-width:1.8"/>
<polygon points="578,55 586,59 578,63" style="fill:var(--bf-vz-1)"/>
<text class="bf-tm" x="562" y="51" text-anchor="middle" style="font-size:9.5px">copy 3</text>
<rect x="588" y="36" width="116" height="46" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-vz-axis);stroke-width:1.5"/>
<text class="bf-t-strong" x="646" y="56" text-anchor="middle" style="font-size:11px">Returned line</text>
<text class="bf-tm" x="646" y="72" text-anchor="middle" style="font-size:10px">PyBytes to caller</text>
<text class="bf-tm" x="16" y="102" style="font-size:10.5px">Raw BZ2Reader without BufferedReader: inherited io.IOBase.readline() called read(1) for every byte.</text>
<line x1="16" y1="120" x2="704" y2="120" style="stroke:var(--bf-vz-grid);stroke-width:1;stroke-dasharray:4,4"/>
<text class="bf-t-strong" x="16" y="144" style="font-size:12px">After: direct Rust SWAR line framing + bound C methods (1 copy per line, abi3-py310)</text>
<rect x="16" y="156" width="170" height="52" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-vz-3);stroke-width:1.8"/>
<text class="bf-t-strong" x="101" y="177" text-anchor="middle" style="font-size:11px">Rust block buffer</text>
<text class="bf-tm" x="101" y="194" text-anchor="middle" style="font-size:10px">tracks unread bytes in block</text>
<line x1="186" y1="182" x2="232" y2="182" style="stroke:var(--bf-vz-3);stroke-width:1.8"/>
<polygon points="232,178 240,182 232,186" style="fill:var(--bf-vz-3)"/>
<text class="bf-tm" x="212" y="174" text-anchor="middle" style="font-size:9.5px">in place</text>
<rect x="242" y="156" width="230" height="52" rx="6" style="fill:var(--bf-green-tint);stroke:var(--bf-vz-3);stroke-width:1.8"/>
<text class="bf-t-strong" x="357" y="177" text-anchor="middle" style="font-size:11px">8-byte SWAR newline scan</text>
<text class="bf-tm" x="357" y="194" text-anchor="middle" style="font-size:10px">no GIL release until block empties</text>
<line x1="472" y1="182" x2="546" y2="182" style="stroke:var(--bf-vz-3);stroke-width:1.8"/>
<polygon points="546,178 554,182 546,186" style="fill:var(--bf-vz-3)"/>
<text class="bf-t-strong" x="512" y="174" text-anchor="middle" style="font-size:9.5px">1 copy</text>
<rect x="556" y="156" width="148" height="52" rx="6" style="fill:var(--bf-surface);stroke:var(--bf-vz-3);stroke-width:1.8"/>
<text class="bf-t-strong" x="630" y="177" text-anchor="middle" style="font-size:11px">Returned line / chunk</text>
<text class="bf-tm" x="630" y="194" text-anchor="middle" style="font-size:10px">PyBytes or memoryview</text>
<text class="bf-tm" x="16" y="232" style="font-size:10.5px">BufferedReader subclass stays intact for isinstance(), .raw, .detach(), and TextIOWrapper().</text>
</svg>
<figcaption>Top: stacking CPython's 1 MiB BufferedReader over mibz2's already block-buffered Rust
reader copied every decoded byte three times. Bottom: scanning newlines inside the lent Rust block
buffer cuts delivery to one copy while keeping full BufferedReader compatibility.</figcaption>
</figure>

Even with decoded bytes already in memory, CPython's buffered framing costs twice as much as direct
memory iteration. On 28.7 MB of text with 793,778 lines, wrapping a memory buffer in a 1 MiB
buffered reader took 227.7 ms of CPU. Iterating the memory buffer alone took 110.4 ms. That gap was
pure framing overhead waiting to be removed.

## 2. Scanning lines in safe Rust without releasing the GIL

Moving line framing into Rust helps only if you do not trade one overhead for another. A naive
Rust line reader that drops Python's Global Interpreter Lock on every call would pay 793,778 lock
round-trips on our text file. Yet the entire 28.7 MB stream contains only thirty-two bzip2 blocks.

Simple state tracking removes that churn. Inside the Rust reader wrapper, we track how many unread
bytes remain in the current decoded block. Whenever bytes remain, line reads and fixed-size reads
stay inside the current slice and skip releasing the GIL. The reader drops the GIL only when a block
empties, letting parallel workers refill the buffer just thirty-two times across 28.7 MB.

We locate each newline inside the lent block slice with standard integer operations rather than
unsafe SIMD intrinsics or external crates. XORing an eight-byte word with repeated newline bytes and
applying a zero-byte bitmask checks eight bytes in four instructions. A short scalar loop handles
the remaining tail bytes. Exhaustive unit tests check every offset, boundary, and length against
byte-by-byte scanning.

Reading all lines into a list goes one step further. Rust scans and allocates every complete line in
the current decoded block in one tight loop. It advances the reader cursor once per block instead of
once per line, then appends straight to a Python list. When a line crosses a block boundary, only
that single line gathers bytes across two blocks before the tight loop resumes.

## 3. Two surprises caught by benchmarking

Fast Rust methods were only half the story. Two subtle Python-side interactions showed up right away
under paired benchmarking and would have erased much of the gain if left unchecked.

**A one-line Python wrapper can cost more than the work it avoids.** Our first prototype subclassed
CPython's buffered reader and forwarded line calls through a tiny Python method. Across 793,778
lines, that extra Python call slowed explicit line loops by 66%, jumping from 660 ms to 1,098 ms.
CPython had to build and tear down a Python call frame nearly eight hundred thousand times just to
pass one argument to C.

Direct C-method binding solves the problem cleanly. When mibz2's file opener creates its buffered
reader subclass, it initializes the base buffer to a 1-byte stub, dropping the redundant 1 MiB C
buffer. It binds the Rust reader's C methods directly onto the instance for line reading and
peeking. It also returns the Rust reader directly as the iterator. Calling detach removes those
bindings so a detached stream behaves like stock Python. Standard type checks, raw-stream
attributes, and text wrappers keep working unchanged. Explicit line loops flipped from 66% slower to
4% faster.

**Slicing a bytearray silently invokes resizing logic.** During verification, writing decoded bytes
into a caller's 1 MiB bytearray via direct slice assignment slowed single-threaded reads by 14.7%.
In CPython, bytearray slice assignment runs general sequence-replacement code that checks whether
the buffer must grow or shift. Wrapping the target in a byte-cast memory view takes a fixed-size
copy path instead, flipping the same workload from 14.7% slower to 2.5% faster.

## 4. Measured gains across text and Wikidata

We benchmarked the finished safe-Rust code in fresh subprocesses using order-balanced paired trials.
The suite covers 28.7 MB of text (793,778 lines) and 210 MB of Wikidata JSON (15,620 long lines).
Every workload ran at one and two threads with exact SHA-256 byte parity checks.

<figure class="bf-fig" id="delivery-gains">
<p class="bf-title">Paired CPU time and peak memory change across Python streaming workloads</p>
<!-- fig:delivery-gains -->
<svg viewBox="0 0 720 318" role="img" aria-label="Horizontal bar chart showing paired CPU time reductions for direct Rust Python delivery under abi3-py310 with zero unsafe code. BZ2Reader line iteration on 1 MiB text: minus 96.2% CPU (26.1 times faster). open().readlines() on 28.7 MB text: minus 28.3% CPU on 1 thread, minus 27.6% on 2 threads. for line in open() on 28.7 MB text: minus 13.3% CPU on 1 thread (minus 6.1% peak memory), minus 13.9% on 2 threads (minus 7.8% peak memory). open().read(64 KiB) on 210 MB Wikidata: minus 9.1% CPU (minus 11.8% peak memory) on 1 thread, minus 6.2% on 2 threads. open().read(1 MiB) on 210 MB Wikidata: minus 5.2% CPU on 1 thread. BZ2Reader.readinto(1 MiB) on Wikidata: minus 2.5% CPU on 1 thread.">
<line x1="252.0" y1="22.0" x2="252.0" y2="270.0" style="stroke:var(--bf-vz-axis);stroke-width:1.5"/>
<text class="bf-tm" x="252.0" y="286.0" text-anchor="middle" style="font-size:10.5px">0%</text>
<line x1="342.0" y1="22.0" x2="342.0" y2="270.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/>
<text class="bf-tm" x="342.0" y="286.0" text-anchor="middle" style="font-size:10.5px">−25%</text>
<line x1="432.0" y1="22.0" x2="432.0" y2="270.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/>
<text class="bf-tm" x="432.0" y="286.0" text-anchor="middle" style="font-size:10.5px">−50%</text>
<line x1="522.0" y1="22.0" x2="522.0" y2="270.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/>
<text class="bf-tm" x="522.0" y="286.0" text-anchor="middle" style="font-size:10.5px">−75%</text>
<line x1="612.0" y1="22.0" x2="612.0" y2="270.0" style="stroke:var(--bf-vz-grid);stroke-width:1"/>
<text class="bf-tm" x="612.0" y="286.0" text-anchor="middle" style="font-size:10.5px">−100%</text>

<text class="bf-t-strong" x="242" y="42" text-anchor="end" style="font-size:11px">BZ2Reader line iteration</text>
<text class="bf-tm" x="242" y="56" text-anchor="end" style="font-size:10px">1 MiB text, 1 thread</text>
<rect x="252.0" y="33.0" width="346.3" height="18.0" rx="3" style="fill:var(--bf-vz-3)"><title>BZ2Reader line iteration: -96.2% CPU (26.1x faster)</title></rect>
<text class="bf-t-strong" x="604.0" y="46.0" style="font-size:10.5px">−96.2% (26.1×)</text>

<text class="bf-t-strong" x="242" y="86" text-anchor="end" style="font-size:11px">open().readlines()</text>
<text class="bf-tm" x="242" y="100" text-anchor="end" style="font-size:10px">28.7 MB text, 1T / 2T</text>
<rect x="252.0" y="75.0" width="101.9" height="12.0" rx="2.5" style="fill:var(--bf-vz-3)"><title>open().readlines() 1T: -28.3% CPU, -2.3% peak memory</title></rect>
<text class="bf-t-strong" x="360.0" y="85.0" style="font-size:10px">1T: −28.3% CPU</text>
<rect x="252.0" y="90.0" width="99.4" height="12.0" rx="2.5" style="fill:var(--bf-vz-2)"><title>open().readlines() 2T: -27.6% CPU, -2.8% peak memory</title></rect>
<text class="bf-tm" x="358.0" y="100.0" style="font-size:10px">2T: −27.6% CPU</text>

<text class="bf-t-strong" x="242" y="132" text-anchor="end" style="font-size:11px">for line in open()</text>
<text class="bf-tm" x="242" y="146" text-anchor="end" style="font-size:10px">28.7 MB text, 1T / 2T</text>
<rect x="252.0" y="121.0" width="47.9" height="12.0" rx="2.5" style="fill:var(--bf-vz-3)"><title>for line in open() 1T: -13.3% CPU, -6.1% peak memory</title></rect>
<text class="bf-t-strong" x="306.0" y="131.0" style="font-size:10px">1T: −13.3% CPU (−6.1% mem)</text>
<rect x="252.0" y="136.0" width="50.0" height="12.0" rx="2.5" style="fill:var(--bf-vz-2)"><title>for line in open() 2T: -13.9% CPU, -7.8% peak memory</title></rect>
<text class="bf-tm" x="308.0" y="146.0" style="font-size:10px">2T: −13.9% CPU (−7.8% mem)</text>

<text class="bf-t-strong" x="242" y="178" text-anchor="end" style="font-size:11px">open().read(64 KiB)</text>
<text class="bf-tm" x="242" y="192" text-anchor="end" style="font-size:10px">210 MB Wikidata, 1T / 2T</text>
<rect x="252.0" y="167.0" width="32.8" height="12.0" rx="2.5" style="fill:var(--bf-vz-3)"><title>open().read(64 KiB) 1T: -9.1% CPU, -11.8% peak memory</title></rect>
<text class="bf-t-strong" x="291.0" y="177.0" style="font-size:10px">1T: −9.1% CPU (−11.8% mem)</text>
<rect x="252.0" y="182.0" width="22.3" height="12.0" rx="2.5" style="fill:var(--bf-vz-2)"><title>open().read(64 KiB) 2T: -6.2% CPU, -0.7% peak memory</title></rect>
<text class="bf-tm" x="281.0" y="192.0" style="font-size:10px">2T: −6.2% CPU</text>

<text class="bf-t-strong" x="242" y="220" text-anchor="end" style="font-size:11px">open().read(1 MiB)</text>
<text class="bf-tm" x="242" y="234" text-anchor="end" style="font-size:10px">210 MB Wikidata, 1T</text>
<rect x="252.0" y="213.0" width="18.7" height="14.0" rx="2.5" style="fill:var(--bf-vz-3)"><title>open().read(1 MiB) 1T: -5.2% CPU</title></rect>
<text class="bf-tm" x="277.0" y="224.0" style="font-size:10px">−5.2% CPU</text>

<text class="bf-t-strong" x="242" y="254" text-anchor="end" style="font-size:11px">BZ2Reader.readinto(1 MiB)</text>
<text class="bf-tm" x="242" y="268" text-anchor="end" style="font-size:10px">210 MB Wikidata, 1T</text>
<rect x="252.0" y="247.0" width="9.0" height="14.0" rx="2.5" style="fill:var(--bf-vz-3)"><title>BZ2Reader.readinto(1 MiB) 1T: -2.5% CPU (fixed from +14.7% naive bytearray slice)</title></rect>
<text class="bf-tm" x="267.0" y="258.0" style="font-size:10px">−2.5% CPU (via memoryview)</text>

<text class="bf-tm" x="704.0" y="308.0" text-anchor="end" style="font-size:10.5px">Paired CPU reduction vs previous release (longer bar is faster; 0 regressions across 29 cells)</text>
</svg>
<figcaption>Order-balanced paired CPU reductions across Python streaming workloads on text and
Wikidata, all in safe Rust inside abi3-py310. Peak memory also drops by 6 to 12% on streaming
line and 64 KiB reads once the redundant 1 MiB BufferedReader buffer is gone.</figcaption>
</figure>

Across all twenty-nine workload cells on both corpora, zero cells regressed. Single-threaded peak
memory stayed flat or fell by up to 11.8%.

## 5. Where the Stable ABI boundary really sits

One workload deliberately stayed unchanged. Whole-stream reads and one-shot decompression return
the 210 MB Wikidata dump as one bytes object. Peak resident memory sits near 434 MiB, just over
twice the decoded output size.

Tracing why uncovered two constraints outside mibz2's own code.

First, PyO3's safe Stable ABI bytes constructor always zero-fills the target buffer before letting
safe Rust write into it. For a 210 MB stream, that faults and writes every page twice while the
source chunks are still live. Peak memory stays near 2×, and the extra pass adds over a second of
work under the GIL. PyO3's newer bytes writer avoids that zero-fill on full CPython builds. Under
the Limited API, however, it falls back to an intermediate Rust vector plus a full copy.

Second, glibc's dynamic mapping threshold keeps 1 MiB chunks on the process heap. Copying chunks
into a pre-sized buffer and freeing each one right away does not help. Freeing a 1 MiB buffer during
startup teaches glibc to raise its dynamic threshold. In a direct allocator probe on 210 MB, 1 MiB
chunks stayed on the heap and peaked at 428 MiB. Raising the chunk size past glibc's 32 MiB hard cap
put each chunk in its own anonymous mapping. Those mappings returned to the operating system on
free, cutting peak resident memory by 41 to 45% down to 251 MiB.

Those two facts draw a clear line between what belongs in a universal wheel and what requires a
packaging trade-off. Every streaming and line-oriented gain above ships in safe Rust inside the
Python 3.10 Stable ABI, keeping one wheel for Python 3.10 through 3.15. Breaking the 2× peak memory
floor on whole-stream reads stays a deliberate choice for later. Doing so means shipping per-version
Python wheels for a zero-copy writer, or pairing a small unsafe Limited-API allocation with chunks
above 32 MiB.

---

**Sources:**

- Direct Rust Stable ABI line and streaming delivery (merged for the upcoming 0.5.18 release): [#1335](https://github.com/lmmx/mibz2/pull/1335) (Rust Python bindings, Python wrapper, type stubs, and tests).
- Full Python delivery bottleneck dissection, PyO3 `PyBytes::new_with` audit, and glibc `M_MMAP_THRESHOLD` probe: [#1327](https://github.com/lmmx/mibz2/pull/1327), [lab notebook](https://github.com/lmmx/mibz2/blob/60db3337/docs/notebook/2026-11-28-explore-python-abi3-delivery.md).
- Paired verification across 29 workload cells on text and Wikidata: [delta record](https://github.com/lmmx/mibz2/blob/60db3337/frontier/deltas/2026-11-28-python-abi3-delivery-build.md).
