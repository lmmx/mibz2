# mibz2

A bzip2 decoder built for memory-level parallelism.

mibz2 decodes bzip2. It is block-parallel, as lbzip2 is. Within a block, its inverse BWT is walked as 12 interleaved chains, so a core keeps a dozen of the walk's cache-missing loads in flight instead of one.

- Its output is byte for byte libbzip2's, and every block CRC and stream CRC is checked.
- It runs on any x86-64 or aarch64 CPU. Its SIMD kernels (AVX2, PCLMULQDQ) are chosen at run time, and each has a portable scalar twin.
- The library depends on std only.

```pycon
>>> import mibz2
>>> data = mibz2.decompress(open("file.bz2", "rb").read())
>>> data[:18]
b'line one\nline two\n'
>>> with mibz2.open("file.bz2") as f:
...     lines = f.readlines()
...     verified = f.raw.verified
>>> len(lines), verified
(2000, True)
```

Its name is for "multi-chain inversion of bzip2". It was written in [wikidata-pq](<https://github.com/lmmx/wikidata-pq>), to decode the Wikidata dump, and knows nothing of Wikidata ([history](<https://lmmx.github.io/mibz2/pr-preview/pr-27/background/history/index.md>)).

## Where to go

- [Installation](<https://lmmx.github.io/mibz2/pr-preview/pr-27/getting-started/installation/index.md>) and the [quickstart](<https://lmmx.github.io/mibz2/pr-preview/pr-27/getting-started/quickstart/index.md>), in Python and in Rust.
- [What is checked, and when](<https://lmmx.github.io/mibz2/pr-preview/pr-27/concepts/checks/index.md>): each block's CRC before its bytes are returned, each stream's at its end, and what an error reports.
- [How it works](<https://lmmx.github.io/mibz2/pr-preview/pr-27/concepts/how-it-works/index.md>): the scanner, the workers and the multi-chain walk.
- [Benchmarks](<https://lmmx.github.io/mibz2/pr-preview/pr-27/benchmarks/index.md>): master's figures over its commits, and mibz2 against pbz2 and lbzip2.
- [Reference](<https://lmmx.github.io/mibz2/pr-preview/pr-27/reference/index.md>): the Python API, and the Rust API on docs.rs.

## Install

```sh
pip install mibz2   # Python 3.10+
cargo add mibz2     # Rust
```
