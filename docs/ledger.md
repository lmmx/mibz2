# The footprint and queue ledger

What the parallel decoder (src/parallel.rs) allocates, what of it holds data still to be
read, what of it the T-vector phase reads at random, and whether the scanner, the workers
or the reader is waiting. It has two halves (#39, #65):

- **the ledger:** arithmetic from the code, `mibz2::trace::ledger` (src/trace.rs). It is
  printed by `just bench-ledger` (no run) and by `just bench-occupancy`;
- **the measurements:** high-water marks and queue states, swept from the decoder's own
  timeline (feature `trace`) by examples/occupancy.rs. They are printed as six lines a
  thread count, with the ledger's bounds checked against them.

## Three quantities, kept apart

- **Reserved** (the ledger's `alloc`) is a `Vec`'s capacity: address space asked of the
  allocator. Pages never written are not resident, so peak RSS can be below it.
- **Live** is the bytes holding data that will still be read: a T-vector until its walk
  ends, a block's decoded bytes until the reader has copied them out.
- **Hot (a model)** is what the T-vector phase touches at random: each worker's T-vector
  (4 bytes a row), written by the build's 256 scattered streams and read by the walk's
  dependent loads, plus 3 KB of counts and segment starts. Everything else is read or
  written in order. This is a model of the walk's working set, not a measured cache
  residency.

**RSS is resident pages, not last-level-cache residency.** None of these figures says
what is in a cache.

## The ledger

M = 100,000 × level is the most rows a block can hold (from the stream header). n is the
longest block's rows, and D its largest decoded size. t is the threads, s the window
(`slots`, at least t) and c the writer's chunk (4 MiB from `BzReader::parallel`).
"Once" means allocated once and reused, so its capacity is a high-water mark: these
buffers never shrink.

| owner | buffer | how many | reserved each | at most | live each | hot |
|---|---|--:|--:|--:|--:|--:|
| worker | `ll`, the last column | t | M | M | n, until the build has read it | – |
| worker | `tt`, the T-vector | t | 4n | 4(2n − 1) | 4n, in the T-vector phase | 4n |
| worker | `rle`, the walk's output | t | n | 2n − 1 | n | – |
| worker | `seg.bufs`, the walk's segments | t | 512 (2n/512 + 64) | ratchets | n | – |
| worker | `seg.starts`, `.next`, `.bufs.outer`, `counts` | t | 21.5 KB | 37.9 KB | 21.5 KB | 3 KB |
| slot | `symbols` | s | 1,800,200 | same | 2(n + 1); the corpus's ≈ 0.2 MB | – |
| slot | `out`, decoded bytes | s + 1 | max(n + 64, D) | 2 × that | D | – |
| input | chunks | 4 | c | c | c | – |
| input | the scanner's window | 1 | c | 2(4 MiB + 2 MiB + c) | 2 MiB + c | – |
| decoder | block tables | 1 | ≈ 47 KB, per block | | | – |
| decoder | thread stacks (virtual) | t + 2 | 2 MiB | | a few KB | – |

Where the counts and bounds come from:

- **The T-vector and the walk's output double** when a block is longer than any that
  worker has decoded before (`Vec` growth: max(2 × capacity, need)). The corpus's full
  -9 blocks run from about 899,930 to 900,000 rows, so most workers end with 7.2 MB
  reserved for a 3.6 MB T-vector. The second half is never written, so it costs address space only.
- **The segment buffers ratchet.** Each starts at twice the mean segment and doubles when
  its segment fills it. So each grows with the longest segment it has held. On the
  corpus they end at 6–8 MB a worker, 3–4× their 1.83 MB floor. No useful bound holds;
  the instrument measures them.
- **Symbols** are reserved at the format's bound (18,002 selectors × 50 symbols × 2 B),
  whatever the level. There are at most `slots` buffers: the scanner's block and the
  blocks issued but not yet decoded, with no new buffer while the pool has one.
- **Output buffers number at most `slots + 1`.** At most `slots` are decoding or in
  `ready`, and one more is the reader's current block. A block that decodes to more than
  n + 64 bytes doubles its buffer.
- **The input chunks:** 2 are queued (`INPUT_QUEUE`), 1 is held by the writer (filling or
  sending it), and 1 is being appended to the window. `BzReader`'s feeder allocates each
  chunk afresh.
- **The scanner's window** keeps up to `COMPACT` (4 MiB) of bytes already read before it
  drops them. So it holds under 4 MiB + its 2 MiB lookahead + one chunk, reserved by
  doubling.

**The ledger at -9 (full blocks, the default window 2t, 4 MiB chunks; `just bench-ledger
--threads 1,2,4,10`):**

| threads | reserved at least | at most, before the segments' ratchet | hot (model) |
|--:|--:|--:|--:|
| 1 | 34.6 MB | 56.7 MB | 3.6 MB |
| 2 | 47.2 MB | 73.9 MB | 7.2 MB |
| 4 | 72.5 MB | 108.2 MB | 14.4 MB |
| 10 | 148.5 MB | 211.0 MB | 36.0 MB |

The segment buffers add at least 1.83 MB a worker (measured: 6–8 MB). A block of runs
only, which decodes to 46.6 MB at -9, would raise each output buffer's bound to 93 MB. No
real text does that.

The tests (`trace::tests`) decode a two-stream synthetic file, a short block and then a
longer one, through the decoder's own calls. They check each of a worker's buffers'
capacities against the ledger: exactly where it is exact (`ll`, `seg.starts`, `symbols`,
and `tt`, `rle` and `out` by the growth rule), within its bounds elsewhere. They also
check that every buffer a worker has is in the ledger.

## The measurements (the occupancy instrument's summary)

These come from one sweep of every thread's marks in time order. Each interval is
credited to the states before the mark that ends it. Shares are of the trace's span, or,
for the workers, of the workers' time.

- **Reader waiting:** from `ReadWait` to `Taken`. Split into:
  - **starved:** the next block in order is not yet ready, so the decoder is behind;
  - **the handoff:** the next block is ready and the reader has not yet woken to take it.

  Neither is producer backpressure.
- **Scanner backpressured:** waiting for a slot (`SlotWait` to `SlotGot`) while the
  earliest untaken block is ready, so the reader binds. Its share during the handoff is
  given apart. It is distinct from waiting for a slot while the earliest untaken block is
  still decoding, which means the workers bind. Waiting for input is given apart.
- **Workers:** decoding (`DecodeStart` to `DecodeEnd`); idle with no job queued; idle
  with a job queued (sent and not yet received: the handoff); the mean number decoding.
- **The T-vector phase:** `decode_block`'s inverse BWT, the build and the walk
  (`InvertStart` to `InvertEnd`). It gives the mean and greatest number of blocks in the
  phase, and their T-vectors' bytes, the hot model's measured counterpart (rows × 4).
- **The window:** blocks issued and not taken, plus the block the scanner reads in the
  slot it got. Its mean and high-water are at most `slots` by construction, which is
  checked.
- **Ready, not taken:** blocks from `Ready` to `Taken`, and their decoded bytes (from
  `DecodeEnd`). Their bytes are split by what held them:
  - head of line: the reader waiting while an earlier block decodes;
  - the reader busy;
  - the handoff.

  It also gives the time from ready to taken against the mean decode.
- **Held:** symbol buffers from `Issued` until the worker returns them, and of those, the
  dead bytes. Those are the bytes from MTF's end (`InvertStart`) on, held but never read
  again. It also gives output buffers' capacity against their bytes over all blocks, and
  the window's live bytes and capacity (`Window`).
- **Worker buffers:** each worker's capacities as it recorded them at its end, its
  high-water marks. They are checked against the ledger's bounds.

## What these figures cannot tell you

- **Not cache residency, and not bandwidth.** "Hot" is a model of which bytes are
  accessed at random. Whether they stay in L2 or L3 depends on the other cores'
  traffic. A streaming buffer such as an output block passes through the caches without
  being "hot" here. Cache and memory-controller counters (bench/imc.py) are the measure
  for that.
- **Not RSS.** The ledger counts `Vec` capacity. It leaves out allocator overhead,
  fragmentation and freed memory the allocator keeps. Untouched capacity isn't
  resident. Peak RSS is printed beside it for comparison: the same process, the `off`
  mode.
- **Not every allocation.** The scanner's per-block tables are approximate: their struct
  size isn't public. The channels' nodes and the shared state's headers, a few KB, are
  left out. So are the caller's own read buffer and the thread stacks' touched pages.
- **Measured figures are this file's.** The corpus has 431 blocks: about 233 full ones of
  about 900,000 rows, and about 200 short ones (a tenth of all its blocks hold 10 rows or
  fewer). So per-block means, such as the mean decode or the time from ready to taken,
  mix both.

## Overhead

- **The default build: none.** Everything new is behind the `trace` feature or `cfg(test)`:
  the marks, the sizes they carry, the buffers' accessors and the ledger. The default
  build's only change is `pub(crate)` on four constants that the ledger reads. The CI
  bench's paired A/B is the check that the default build is unchanged.
- **`trace` compiled in, level 0:** each mark is one relaxed atomic load. A block has two
  more (`InvertStart`, `InvertEnd`), and each input chunk one (`Window`).
- **Levels 1 and 2:** each mark also costs a timestamp and a push to the thread's own
  buffer, now with two sizes (16 more bytes a mark). Reading a `Vec`'s length and
  capacity is two loads. The instrument checks every traced mode against `off` by its
  paired throughput ratio. A mode whose interval excludes no change is reported as
  perturbing, and its trace is not to be interpreted.
