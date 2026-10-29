# Quickstart

## Python

Write a .bz2 file with the standard library's `bz2`, to have one to read:

```python
import bz2

with bz2.open("sample.bz2", "wb") as f:
    for i in range(10_000):
        f.write(f"line {i}\n".encode())
```

A whole file in memory, decoded on every CPU this process may use, or on one thread:

```pycon
>>> import mibz2
>>> compressed = open("sample.bz2", "rb").read()
>>> data = mibz2.decompress(compressed)
>>> data == bz2.decompress(compressed)
True
>>> mibz2.decompress(compressed, threads=1) == data
True
```

A file read as it is decoded: `open` takes a path or a binary file object, and returns a buffered binary file. Its `raw.verified` turns true once a read has reached the end with every block's and stream's CRC matched:

```pycon
>>> with mibz2.open("sample.bz2") as f:
...     first = f.readline()
...     before = f.raw.verified
...     rest = f.read()
...     after = f.raw.verified
>>> first, before, after
(b'line 0\n', False, True)
```

A bad input raises `mibz2.DecodeError`, a `ValueError`, from `decompress` or from the read at which it is found. The message says which block and bit decoding failed at:

```pycon
>>> damaged = bytearray(compressed)
>>> damaged[len(damaged) // 2] ^= 0x10
>>> try:
...     mibz2.decompress(bytes(damaged))
... except mibz2.DecodeError as e:
...     print(type(e).__name__, isinstance(e, ValueError))
DecodeError True
```

## Rust

```rust
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // a whole .bz2 in memory (its streams one after another): on this thread, or on 4
    let data = std::fs::read("file.bz2")?;
    let text = mibz2::decode(&data)?;
    let same = mibz2::decode_parallel(&data, 4)?;
    assert_eq!(text, same);

    // any Read, as a Read: on this thread, or block-parallel on 4 worker threads
    let mut out = Vec::new();
    mibz2::BzReader::parallel(std::fs::File::open("file.bz2")?, 4).read_to_end(&mut out)?;
    Ok(())
}
```

A bad stream is an `mibz2::Error` (its `kind`, `block` and `bit`), or from a `BzReader` an `io::Error` of kind `InvalidData` wrapping it. The [Rust reference](<https://lmmx.github.io/mibz2/pr-preview/pr-115/reference/rust/index.md>) lists the rest: the push decoder, the block scanner and the decoding of a range of blocks.
