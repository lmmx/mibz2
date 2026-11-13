# mibz2 (Rust)

The crate's API documentation is on [docs.rs](<https://docs.rs/mibz2>).

| API | what |
| --- | --- |
| `decode`, `decode_parallel(data, threads)` | a whole .bz2 in memory |
| `BzReader::new(r)`, `BzReader::parallel(r, threads)`, `parallel_with(r, threads, slots, kernels)` | a `Read` adaptor; a bad stream is an `io::Error` (`InvalidData`) wrapping `mibz2::Error` |
| `StreamDecoder::new(threads, slots, kernels)` | the push form: `write` compressed chunks from one thread, `read` decoded bytes from another, `close`, `cancel`; at most `slots` blocks in flight |
| `scan(data)` → `BlockSpan { index, start_bit, end_bit, crc, stream, level }` | block boundaries, without decoding |
| `decode_blocks(data, start_bit, end_bit, level, kernels, out)` → `Range` | a range of whole blocks, each CRC checked |
| `Error { kind: ErrorKind, block, bit }` | what is wrong with the input, and where |
| `Kernels::detect()`, `Kernels::scalar()` | the SIMD kernels, or the portable ones |
| `bwt::InverseBwt` (`Segmented`, the default; `Plain`), `decode_with::<I>`, `StreamDecoder::with_inverse::<I>` | the inverse-BWT kernel, swappable and timeable as part of the whole decoder |

```rust
// the block boundaries (each block's header and Huffman symbols read, nothing decoded),
// then any range of whole blocks decoded on its own, on any thread
let data = std::fs::read("file.bz2")?;
for span in mibz2::scan(&data) {
    let span = span?;
    let mut part = Vec::new();
    let range = mibz2::decode_blocks(&data, span.start_bit, span.end_bit, span.level,
                                      mibz2::Kernels::detect(), &mut part)?;
    // range.blocks: each block's (start bit, end bit, CRC); mibz2::combine(...) gives a
    // stream's combined CRC from its blocks'
}
```
