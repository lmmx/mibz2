# bunzip

A bzip2 decoder built for memory-level parallelism. It is block-parallel like lbzip2. Within
a block, its inverse BWT is walked as 12 interleaved chains, so a core keeps a dozen of the
walk's cache-missing loads in flight instead of one.

- Its output is byte for byte libbzip2's, and every block CRC and stream CRC is checked.
- It runs on any x86-64 or aarch64 CPU. Its SIMD kernels (AVX2, PCLMULQDQ) are chosen at
  run time, and each has a portable scalar twin.
- It depends on std only.

It lives in [wikidata-pq](https://github.com/lmmx/wikidata-pq), whose split decodes the
Wikidata dump with it (`WIKIDATA_BZ2_DECODER=native`). It knows nothing of Wikidata, and
nothing in this directory depends on the rest of that repository. *The name is
provisional; the crate is not published.*

## Use

```rust
use std::io::Read;

// a whole .bz2 in memory (its streams one after another): on this thread, or on 4
let data = std::fs::read("file.bz2")?;
let text = bunzip::decode(&data)?;
let same = bunzip::decode_parallel(&data, 4)?;

// any Read, as a Read: on this thread, or block-parallel on 4 worker threads
let mut out = Vec::new();
bunzip::BzReader::new(std::fs::File::open("file.bz2")?).read_to_end(&mut out)?;
bunzip::BzReader::parallel(std::fs::File::open("file.bz2")?, 4).read_to_end(&mut out)?;

// the block boundaries (each block's header and Huffman symbols read, nothing decoded),
// then any range of whole blocks decoded on its own, on any thread: a caller's own
// parallelism, or a sweep over block-aligned segments
for span in bunzip::scan(&data) {
    let span = span?;
    let mut part = Vec::new();
    let range = bunzip::decode_blocks(&data, span.start_bit, span.end_bit, span.level,
                                      bunzip::Kernels::detect(), &mut part)?;
    // range.blocks: each block's (start bit, end bit, CRC); bunzip::combine(...) gives a
    // stream's combined CRC from its blocks'
}
```

| API | what |
|---|---|
| `decode`, `decode_parallel(data, threads)` | a whole .bz2 in memory |
| `BzReader::new(r)`, `BzReader::parallel(r, threads)`, `parallel_with(r, threads, slots, kernels)` | a `Read` adaptor; a bad stream is an `io::Error` (`InvalidData`) wrapping `bunzip::Error` |
| `StreamDecoder::new(threads, slots, kernels)` | the push form: `write` compressed chunks from one thread, `read` decoded bytes from another, `close`, `cancel`; at most `slots` blocks in flight |
| `scan(data)` → `BlockSpan { index, start_bit, end_bit, crc, stream, level }` | block boundaries, without decoding |
| `decode_blocks(data, start_bit, end_bit, level, kernels, out)` → `Range` | a range of whole blocks, each CRC checked |
| `Error { kind: ErrorKind, block, bit }` | what is wrong with the input, and where |
| `Kernels::detect()`, `Kernels::scalar()` | the SIMD kernels, or the portable ones |
| `bwt::InverseBwt` (`Segmented`, the default; `Plain`), `decode_with::<I>`, `StreamDecoder::with_inverse::<I>` | the inverse-BWT kernel, swappable and timeable as part of the whole decoder |

**Errors** (`ErrorKind`) cover the following:

- the input: `Empty`, `NotBz2`, `Truncated`, `TrailingData`;
- each of libbzip2's block checks: `BadBlockMagic`, `BadOrigPtr`, `NoSymbols`,
  `BadGroups`, `BadSelectors`, `BadSelector`, `BadCodeLength`, `BadHuffmanCode`,
  `SelectorOverrun`, `RunTooLong`, `BlockTooLong`, `RunUnfinished`;
- `BlockCrc` and `StreamCrc`, with the stored and computed values;
- this decoder's refusals: `Randomised`, `BlockTooLarge`;
- `Io` (from a `BzReader`'s source), `RangeOvershoot`, `Cancelled`, and `Internal`
  (a caught panic: a bug).

The bytes before an error are exactly the stream's up to the bad block.

## Choices

The format is libbzip2 1.0.8's, with every check it makes. Four choices differ from it,
and each has its own error:

- **randomised blocks** (written only by bzip2 0.9.0 and earlier, before 1999) are
  refused;
- **bytes after the last stream** that do not begin another stream are an error. lbzip2
  and bzip2 ignore them. Refusing them means nothing after a stream is ever silently
  dropped;
- an **oversubscribed Huffman code** pointing past the alphabet, where libbzip2 reads a
  stale table entry, is an error;
- a block whose **compressed form passes 64 MiB** is an error. A real block is at most
  about 4.2 MB.

## How it works

- **The scanner.** One thread reads each block's header and Huffman-coded symbols in
  order. This is the one sequential phase: a block's end is known only once they are read.
  It is roughly a tenth of a decode on well-compressed text, more on poorly compressible
  data.
- **The workers.** Each worker finishes a block: move-to-front, the T-vector, the inverse
  BWT walk, RLE1 and the CRC.
- **The reader** takes the blocks in order.
- **Memory is bounded:** at most `slots` blocks are in flight, and the buffers are reused.
- **The walk** (`bwt::walk_segments`):
  - 256 pseudo-random rows, the walk's first among them, are segment starts;
  - a bitmap of the starts ends each segment;
  - 12 lanes walk segments in lockstep;
  - the segments are joined in order.

  It equals the plain walk on every list, lists of several cycles included.

## Prior work

- **The format and its checks are libbzip2's** (Julian Seward, bzip2 1.0.8,
  `decompress.c`). This decoder follows its decoding rules and makes each of its checks,
  in its terms. Its Huffman decode tables are built as `BZ2_hbCreateDecodeTables` builds
  them, so a stream libbzip2 refuses is refused here too. It is a separate implementation
  in Rust, not a port of libbzip2's code. The tests use libbzip2 itself (the `bzip2`
  crate's bundled 1.0.8) as the oracle.
- **Interleaving several traversals of one BWT inversion** on one thread, so that
  out-of-order execution keeps their loads in flight, is Kärkkäinen, Kempa and Puglisi's
  idea: "Slashing the Time for BWT Inversion", DCC 2012, pp. 99–108.
  - They also use a super-alphabet (an LF² table: two characters a dependent lookup) and
    "Precopy" for repetitive data. Neither is used here.
  - What is particular here is only how the traversals are found in an existing bzip2
    file, which carries no metadata for them: the random starts, the stop bitmap, and the
    joining in order.
- **Block-parallel decoding** is lbzip2's (Mikołaj Izdebski) and pbzip2's approach.

## Tests and benchmarks

```sh
cargo test --release      # against libbzip2: the corpora below, crafted and malformed streams
cargo bench               # decode speed on the corpora below
cargo run --release --example fuzz -- 600 1            # a mutation fuzz loop, libbzip2 the oracle
cargo build --profile fuzz --example fuzz              # (with overflow checks and debug assertions)
cargo run --release --example bzcat -- 4 < f.bz2 > f   # the parallel decoder as a command
cargo run --release --example windows -- OUT w.bin...  # blocks found in byte-range windows of a .bz2, checked
```

**On aarch64:** `rustup target add aarch64-unknown-linux-gnu`, then `apt install qemu-user
gcc-aarch64-linux-gnu`, then `cargo test --release --target aarch64-unknown-linux-gnu`
(`.cargo/config.toml` sets the linker and the qemu runner).

**The tests' corpora, none of them Wikidata** (`tests/common/mod.rs`, `corpora`), each
compressed by reference bzip2 (libbzip2 1.0.8):

- a text corpus: this crate's own sources and README, repeated;
- random bytes;
- one byte repeated;
- a 7-byte pattern repeated;
- runs of random lengths;
- binary records.

They are decoded by every entry point. Beside them are crafted blocks (random
permutations, lists of several cycles, block-size bounds at every level 1–9) and malformed
streams (truncation at every point, CRC flips, every header and table bit flipped,
oversized runs, trailing data).

## Benchmark disclosure

`cargo bench` (best of 3, decoded MB/s, cycles a byte at the nominal clock). The machine
is a **shared 4-vCPU cloud VM** (`Intel(R) Xeon(R) Processor @ 2.80GHz`, AVX2 and
PCLMULQDQ) with other jobs running, so the figures are indicative only. Between two runs,
libbzip2's own figure on the text corpus moved from 18 to 34 MB/s. Corpora at the default
scale (8–20 MB each), level 9:

| corpus | ratio | libbzip2, 1 thread | bunzip, 1 thread | scalar kernels, 1 thread | plain walk, 1 thread | bunzip, 4 threads |
|---|--:|--:|--:|--:|--:|--:|
| text (sources) | 19.8× | 34 MB/s | 84 MB/s | 66 MB/s | 17 MB/s | 155 MB/s |
| random | 1.1× | 8 | 31 | 28 | 17 | 89 |
| one byte | 392,157× | 347 | 2,029 | 1,034 | 2,372 | 831 |
| pattern (7 bytes) | 17,563× | 139 | 93 | 64 | 210 | 302 |
| runs | 263× | 250 | 1,434 | 594 | 797 | 666 |
| records | 1.0× | 8 | 32 | 27 | 9 | 31 |

What the table shows:

- **Text and random data:** 2.5–4× libbzip2 on one thread.
- **A short pattern repeated** is the one case slower than libbzip2: 0.67× on one thread.
  Its walk is cache-resident and prefetchable, so the plain walk wins (210 MB/s), and the
  segments' bookkeeping only costs. Choosing the walk per block is open work.
- **Poorly compressible binary records** do not scale past one thread. The sequential
  Huffman scanner is most of their decode. Random bytes scale 2.9×.
- **Data of near-infinite ratio** (one byte, runs) is bound by copying its output, at
  0.7–2 GB/s.
- On the Wikidata dump's own blocks (its JSON, 12–21×), this decoder is 2.0–2.3× lbzip2
  on one thread (the repository's journal, `docs/journal/2026-10-27-bz2-decoder.md`).

## Licence

None declared yet. The repository this crate lives in declares no licence, so this
crate's is for its owner to choose before it is extracted or published.
