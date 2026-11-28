[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-1363/blog/index.md>)

# Twenty-six times faster line iteration in Python without leaving the Stable ABI

Bypassing Python's redundant buffering layer and scanning newlines in safe Rust speeds up line reads up to twenty-six times while keeping one universal abi3 wheel.

28 November 2026 · 7 min read · [Research result](<#5-where-the-stable-abi-boundary-really-sits>)

When a Rust decoder produces gigabytes per second, the slowest step in Python is often crossing the language boundary. Until recently, mibz2's Python package had optimised whole-file reads. Yet iterating line by line through a compressed log or JSON dump still paid for layers of CPython buffering that repeated work on every call.

We dissected every streaming entry point in mibz2's Python layer. That audit covered raw reader iteration, reading all lines, chunked reads, buffer fills, and whole-stream decompression. Removing extra copies and scanning newlines in safe Rust sped up raw line iteration by 26.1 times. Reading all lines into a list dropped 28%, and line-by-line iteration dropped 13 to 14%. On Wikidata, 64 KiB streaming reads ran 9% faster with 12% less peak memory. Every change stays inside safe Rust and CPython's Stable ABI, keeping one universal wheel across Python 3.10 through 3.15.

The same audit mapped the exact wall that safe Stable ABI code cannot cross on whole-stream reads. There, PyO3's zero-initialisation and glibc's 32 MiB anonymous mapping threshold combine to hold peak memory near twice the decoded output size.

**Takeaways**

- **Raw reader line iteration is 26.1 times faster.** Overriding line iteration directly in Rust eliminates Python's default raw-stream fallback, which previously requested one byte at a time.
- **Skipping Python's redundant buffer removes two copies per read.** mibz2's Rust reader already decodes into block buffers up to 900 KiB. Stacking a 1 MiB CPython buffer on top copied every byte three times instead of once.
- **Direct C-method binding removes Python frame overhead.** Binding Rust line methods directly onto the file object kept full buffered-reader compatibility while turning a 66% wrapper slowdown into a 4% speedup.
- **Safe eight-byte newline scanning avoids per-line GIL churn.** While bytes remain in the current decoded block, Rust scans eight bytes at a time without dropping the Global Interpreter Lock.
- **Whole-stream peak memory hits two external floors.** Safe PyO3 zero-fills a full Stable ABI output buffer before copying into it. Meanwhile, glibc keeps 1 MiB chunks on the process heap unless chunks exceed 32 MiB.

## 1\. Two hidden costs in Python's standard I/O stack

Python's standard I/O module was designed for unbuffered operating-system file descriptors. A minimal raw stream implements one buffer-fill method, and callers wrap it in a buffered reader so small reads hit memory instead of the kernel.

That layering breaks down in two ways when the underlying stream is already a block-parallel Rust decoder.

First, a raw stream that does not override line reading inherits Python's generic fallback. That fallback reads one byte at a time until it sees a newline. On a 1 MiB text slice holding 26,301 lines, raw iteration crossed into Rust over a million times. It spent nearly a full second of CPU time to read a single megabyte.

Second, even callers using mibz2's buffered opener paid for the same buffering twice. mibz2's Rust reader already holds the current decoded bzip2 block in memory (up to 900 KiB) and lends a direct slice of its unread bytes. Stacking a 1 MiB CPython buffered reader on top copied every byte three times. First, bytes moved from the Rust block into a temporary Python bytes object. Next, they moved into the 1 MiB C buffer, and finally into the returned line object.

From three copies to one: streaming lines out of a block-buffered Rust reader

Top: stacking CPython's 1 MiB BufferedReader over mibz2's already block-buffered Rust reader copied every decoded byte three times. Bottom: scanning newlines inside the lent Rust block buffer cuts delivery to one copy while keeping full BufferedReader compatibility.

Even with decoded bytes already in memory, CPython's buffered framing costs twice as much as direct memory iteration. On 28.7 MB of text with 793,778 lines, wrapping a memory buffer in a 1 MiB buffered reader took 227.7 ms of CPU. Iterating the memory buffer alone took 110.4 ms. That gap was pure framing overhead waiting to be removed.

## 2\. Scanning lines in safe Rust without releasing the GIL

Moving line framing into Rust helps only if you do not trade one overhead for another. A naive Rust line reader that drops Python's Global Interpreter Lock on every call would pay 793,778 lock round-trips on our text file. Yet the entire 28.7 MB stream contains only thirty-two bzip2 blocks.

Simple state tracking removes that churn. Inside the Rust reader wrapper, we track how many unread bytes remain in the current decoded block. Whenever bytes remain, line reads and fixed-size reads stay inside the current slice and skip releasing the GIL. The reader drops the GIL only when a block empties, letting parallel workers refill the buffer just thirty-two times across 28.7 MB.

We locate each newline inside the lent block slice with standard integer operations rather than unsafe SIMD intrinsics or external crates. XORing an eight-byte word with repeated newline bytes and applying a zero-byte bitmask checks eight bytes in four instructions. A short scalar loop handles the remaining tail bytes. Exhaustive unit tests check every offset, boundary, and length against byte-by-byte scanning.

Reading all lines into a list goes one step further. Rust scans and allocates every complete line in the current decoded block in one tight loop. It advances the reader cursor once per block instead of once per line, then appends straight to a Python list. When a line crosses a block boundary, only that single line gathers bytes across two blocks before the tight loop resumes.

## 3\. Two surprises caught by benchmarking

Fast Rust methods were only half the story. Two subtle Python-side interactions showed up right away under paired benchmarking and would have erased much of the gain if left unchecked.

**A one-line Python wrapper can cost more than the work it avoids.** Our first prototype subclassed CPython's buffered reader and forwarded line calls through a tiny Python method. Across 793,778 lines, that extra Python call slowed explicit line loops by 66%, jumping from 660 ms to 1,098 ms. CPython had to build and tear down a Python call frame nearly eight hundred thousand times just to pass one argument to C.

Direct C-method binding solves the problem cleanly. When mibz2's file opener creates its buffered reader subclass, it initializes the base buffer to a 1-byte stub, dropping the redundant 1 MiB C buffer. It binds the Rust reader's C methods directly onto the instance for line reading and peeking. It also returns the Rust reader directly as the iterator. Calling detach removes those bindings so a detached stream behaves like stock Python. Standard type checks, raw-stream attributes, and text wrappers keep working unchanged. Explicit line loops flipped from 66% slower to 4% faster.

**Slicing a bytearray silently invokes resizing logic.** During verification, writing decoded bytes into a caller's 1 MiB bytearray via direct slice assignment slowed single-threaded reads by 14.7%. In CPython, bytearray slice assignment runs general sequence-replacement code that checks whether the buffer must grow or shift. Wrapping the target in a byte-cast memory view takes a fixed-size copy path instead, flipping the same workload from 14.7% slower to 2.5% faster.

## 4\. Measured gains across text and Wikidata

We benchmarked the finished safe-Rust code in fresh subprocesses using order-balanced paired trials. The suite covers 28.7 MB of text (793,778 lines) and 210 MB of Wikidata JSON (15,620 long lines). Every workload ran at one and two threads with exact SHA-256 byte parity checks.

Paired CPU time and peak memory change across Python streaming workloads

Order-balanced paired CPU reductions across Python streaming workloads on text and Wikidata, all in safe Rust inside abi3-py310. Peak memory also drops by 6 to 12% on streaming line and 64 KiB reads once the redundant 1 MiB BufferedReader buffer is gone.

Across all twenty-nine workload cells on both corpora, zero cells regressed. Single-threaded peak memory stayed flat or fell by up to 11.8%.

## 5\. Where the Stable ABI boundary really sits

One workload deliberately stayed unchanged. Whole-stream reads and one-shot decompression return the 210 MB Wikidata dump as one bytes object. Peak resident memory sits near 434 MiB, just over twice the decoded output size.

Tracing why uncovered two constraints outside mibz2's own code.

First, PyO3's safe Stable ABI bytes constructor always zero-fills the target buffer before letting safe Rust write into it. For a 210 MB stream, that faults and writes every page twice while the source chunks are still live. Peak memory stays near 2×, and the extra pass adds over a second of work under the GIL. PyO3's newer bytes writer avoids that zero-fill on full CPython builds. Under the Limited API, however, it falls back to an intermediate Rust vector plus a full copy.

Second, glibc's dynamic mapping threshold keeps 1 MiB chunks on the process heap. Copying chunks into a pre-sized buffer and freeing each one right away does not help. Freeing a 1 MiB buffer during startup teaches glibc to raise its dynamic threshold. In a direct allocator probe on 210 MB, 1 MiB chunks stayed on the heap and peaked at 428 MiB. Raising the chunk size past glibc's 32 MiB hard cap put each chunk in its own anonymous mapping. Those mappings returned to the operating system on free, cutting peak resident memory by 41 to 45% down to 251 MiB.

Those two facts draw a clear line between what belongs in a universal wheel and what requires a packaging trade-off. Every streaming and line-oriented gain above ships in safe Rust inside the Python 3.10 Stable ABI, keeping one wheel for Python 3.10 through 3.15. Breaking the 2× peak memory floor on whole-stream reads stays a deliberate choice for later. Doing so means shipping per-version Python wheels for a zero-copy writer, or pairing a small unsafe Limited-API allocation with chunks above 32 MiB.

---

**Sources:**

- Direct Rust Stable ABI line and streaming delivery (merged for the upcoming 0.5.18 release): [\#1335](<https://github.com/lmmx/mibz2/pull/1335>) (Rust Python bindings, Python wrapper, type stubs, and tests).
- Full Python delivery bottleneck dissection, PyO3 `PyBytes::new_with` audit, and glibc `M_MMAP_THRESHOLD` probe: [\#1327](<https://github.com/lmmx/mibz2/pull/1327>), [lab notebook](<https://github.com/lmmx/mibz2/blob/60db3337/docs/notebook/2026-11-28-explore-python-abi3-delivery.md>).
- Paired verification across 29 workload cells on text and Wikidata: [delta record](<https://github.com/lmmx/mibz2/blob/60db3337/frontier/deltas/2026-11-28-python-abi3-delivery-build.md>).
